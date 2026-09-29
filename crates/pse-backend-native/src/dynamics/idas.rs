// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "owned IDAS/SUNContext resources and panic-contained C callbacks"
)]
//! IDAS owns residual integration, consistent initialization, trial recovery, roots,
//! scheduled reinitialization, sign constraints and forward sensitivities (ADR-0093,
//! ADR-0110 item 1). Each scheduled change or reset restarts the same native memory with
//! `IDAReInit`, `IDASensReInit`, `IDAQuadReInit` and `IDACalcIC`; nothing is integrated by
//! project code. Sensitivities are taken with respect to the integration parameters, one
//! per scheduled-input interval, so they cross every scheduled change (I6).
use super::*;
use crate::{
    NativeStatus,
    callback::CallbackState,
    solve::{Assurance, Backend, Execution, NativeTermination, Preconditioner, Progress},
};
use std::{
    ffi::{c_long, c_void},
    marker::PhantomData,
    rc::Rc,
    time::Instant,
};
use suitesparse_sys as _;
use sundials_sys as ffi;

type Jacobian = faer::sparse::SparseColMat<usize, f64>;

struct Context<'a> {
    oracle: &'a mut dyn Oracle,
    contract: Contract,
    /// The contract parameter values of the current segment.
    parameters: Vec<f64>,
    /// The integration parameter vector: the sensitivity parameters.
    integration: Vec<f64>,
    /// The integration column of each contract parameter in the current segment.
    map: Vec<usize>,
    mode: usize,
    trial_policy: TrialPolicy,
    callback: CallbackState,
    /// CSC pattern of the Newton matrix: the state columns of every mode's support plus
    /// the diagonal, sorted by row within each column.
    columns: Vec<ffi::sunindextype>,
    rows: Vec<ffi::sunindextype>,
    /// Inverse Jacobi diagonal of the current Newton matrix (Krylov preconditioner).
    inverse_diagonal: Vec<f64>,
}
impl Context<'_> {
    fn evaluate(
        &mut self,
        f: Function,
        t: f64,
        x: &[f64],
        derivatives: bool,
    ) -> Option<Evaluation> {
        let Self {
            oracle,
            contract: c,
            parameters,
            mode,
            callback,
            ..
        } = self;
        let value = callback.evaluate("idas.evaluation", || {
            let e = oracle.evaluate(*mode, f, t, x, parameters, derivatives)?;
            let rows = match f {
                Function::Output => c.outputs.len(),
                Function::QuadratureFlux => c.quadratures.len(),
                Function::Roots => c.events[*mode].len(),
                _ => c.states.len(),
            };
            if e.values.len() != rows
                || derivatives
                    && e.jacobian.as_ref().is_none_or(|j| {
                        j.nrows() != rows || j.ncols() != c.states.len() + c.parameters.len()
                    })
            {
                return Err(ProblemError::internal(
                    "IDAS function value/derivative dimensions",
                ));
            }
            if e.values.iter().any(|v| !v.is_finite())
                || derivatives
                    && e.jacobian
                        .as_ref()
                        .is_some_and(|j| j.val().iter().any(|v| !v.is_finite()))
            {
                return Err(ProblemError::numerical(
                    "IDAS function value or derivative is nonfinite",
                ));
            }
            Ok(e)
        });
        if value.is_none()
            && callback.terminal.is_none()
            && self.trial_policy == TrialPolicy::Terminal
        {
            callback.terminal = Some((
                crate::solve::Termination::Evaluation,
                "trial failure under terminal policy".into(),
            ));
        }
        value
    }
    /// The analytic state-and-parameter partials of the current mode's residual function;
    /// `evaluate` has checked their presence and extent.
    fn partials(&mut self, t: f64, x: &[f64]) -> Option<Jacobian> {
        self.evaluate(Function::Rhs, t, x, true)
            .and_then(|e| e.jacobian)
    }
    fn failure(&self) -> i32 {
        if self.callback.terminal.is_some() {
            -1
        } else {
            1
        }
    }
    /// The typed cause of a failed callback demand, never a message.
    fn failed(&mut self, operation: &str) -> ProblemError {
        self.callback
            .terminal_error()
            .or_else(|| self.callback.last_failure.take())
            .unwrap_or_else(|| ProblemError::internal(format!("IDAS {operation} failed")))
    }
    /// A failed integration or initialization flag: a latched callback cause wins, and
    /// an evaluation flag reports the callback's typed witness; otherwise the native
    /// status is kept (F06).
    fn native_failure(&mut self, flag: i32, operation: &str) -> ProblemError {
        let status = termination(flag);
        if self.callback.terminal.is_some()
            || status.category == crate::solve::Termination::Evaluation
                && self.callback.last_failure.is_some()
        {
            return self.failed(operation);
        }
        ProblemError::native(
            NativeStatus {
                backend: Backend::Idas,
                code: status.code,
                name: status.name,
            },
            status.category,
            format!("IDAS {operation}"),
        )
    }
}
/// Map every pinned IDA/IDAS return flag to the shared stop vocabulary, mirroring
/// `kinsol::termination`. Only an integer undeclared by the pinned header reaches the
/// final arm, and it is never given a confident category.
pub(crate) fn termination(flag: i32) -> NativeTermination {
    use crate::solve::Termination as T;
    let (name, category) = match flag {
        ffi::IDA_SUCCESS => ("IDA_SUCCESS", T::Success),
        ffi::IDA_TSTOP_RETURN => ("IDA_TSTOP_RETURN", T::Success),
        ffi::IDA_ROOT_RETURN => ("IDA_ROOT_RETURN", T::Success),
        ffi::IDA_WARNING => ("IDA_WARNING", T::Inconclusive),
        ffi::IDA_TOO_MUCH_WORK => ("IDA_TOO_MUCH_WORK", T::IterationLimit),
        ffi::IDA_TOO_MUCH_ACC => ("IDA_TOO_MUCH_ACC", T::Numerical),
        ffi::IDA_ERR_FAIL => ("IDA_ERR_FAIL", T::Numerical),
        ffi::IDA_CONV_FAIL => ("IDA_CONV_FAIL", T::Numerical),
        ffi::IDA_LINIT_FAIL => ("IDA_LINIT_FAIL", T::Numerical),
        ffi::IDA_LSETUP_FAIL => ("IDA_LSETUP_FAIL", T::Numerical),
        ffi::IDA_LSOLVE_FAIL => ("IDA_LSOLVE_FAIL", T::Numerical),
        ffi::IDA_RES_FAIL => ("IDA_RES_FAIL", T::Evaluation),
        ffi::IDA_REP_RES_ERR => ("IDA_REP_RES_ERR", T::Evaluation),
        ffi::IDA_RTFUNC_FAIL => ("IDA_RTFUNC_FAIL", T::Evaluation),
        ffi::IDA_CONSTR_FAIL => ("IDA_CONSTR_FAIL", T::Numerical),
        ffi::IDA_FIRST_RES_FAIL => ("IDA_FIRST_RES_FAIL", T::Evaluation),
        ffi::IDA_LINESEARCH_FAIL => ("IDA_LINESEARCH_FAIL", T::Numerical),
        ffi::IDA_NO_RECOVERY => ("IDA_NO_RECOVERY", T::Numerical),
        ffi::IDA_NLS_INIT_FAIL => ("IDA_NLS_INIT_FAIL", T::Numerical),
        ffi::IDA_NLS_SETUP_FAIL => ("IDA_NLS_SETUP_FAIL", T::Numerical),
        ffi::IDA_NLS_FAIL => ("IDA_NLS_FAIL", T::Numerical),
        ffi::IDA_MEM_NULL => ("IDA_MEM_NULL", T::Invalid),
        ffi::IDA_MEM_FAIL => ("IDA_MEM_FAIL", T::ResourceExhausted),
        ffi::IDA_ILL_INPUT => ("IDA_ILL_INPUT", T::Invalid),
        ffi::IDA_NO_MALLOC => ("IDA_NO_MALLOC", T::Invalid),
        ffi::IDA_BAD_EWT => ("IDA_BAD_EWT", T::Numerical),
        ffi::IDA_BAD_K => ("IDA_BAD_K", T::Invalid),
        ffi::IDA_BAD_T => ("IDA_BAD_T", T::Invalid),
        ffi::IDA_BAD_DKY => ("IDA_BAD_DKY", T::Invalid),
        ffi::IDA_VECTOROP_ERR => ("IDA_VECTOROP_ERR", T::Numerical),
        ffi::IDA_CONTEXT_ERR => ("IDA_CONTEXT_ERR", T::Invalid),
        ffi::IDA_NO_QUAD => ("IDA_NO_QUAD", T::Invalid),
        ffi::IDA_QRHS_FAIL => ("IDA_QRHS_FAIL", T::Evaluation),
        ffi::IDA_FIRST_QRHS_ERR => ("IDA_FIRST_QRHS_ERR", T::Evaluation),
        ffi::IDA_REP_QRHS_ERR => ("IDA_REP_QRHS_ERR", T::Evaluation),
        ffi::IDA_NO_SENS => ("IDA_NO_SENS", T::Invalid),
        ffi::IDA_SRES_FAIL => ("IDA_SRES_FAIL", T::Evaluation),
        ffi::IDA_REP_SRES_ERR => ("IDA_REP_SRES_ERR", T::Evaluation),
        ffi::IDA_BAD_IS => ("IDA_BAD_IS", T::Invalid),
        ffi::IDA_NO_QUADSENS => ("IDA_NO_QUADSENS", T::Invalid),
        ffi::IDA_QSRHS_FAIL => ("IDA_QSRHS_FAIL", T::Evaluation),
        ffi::IDA_FIRST_QSRHS_ERR => ("IDA_FIRST_QSRHS_ERR", T::Evaluation),
        ffi::IDA_REP_QSRHS_ERR => ("IDA_REP_QSRHS_ERR", T::Evaluation),
        ffi::IDA_UNRECOGNIZED_ERROR => ("IDA_UNRECOGNIZED_ERROR", T::Inconclusive),
        ffi::IDA_NO_ADJ => ("IDA_NO_ADJ", T::Invalid),
        ffi::IDA_NO_FWD => ("IDA_NO_FWD", T::Invalid),
        ffi::IDA_NO_BCK => ("IDA_NO_BCK", T::Invalid),
        ffi::IDA_BAD_TB0 => ("IDA_BAD_TB0", T::Invalid),
        ffi::IDA_REIFWD_FAIL => ("IDA_REIFWD_FAIL", T::Numerical),
        ffi::IDA_FWD_FAIL => ("IDA_FWD_FAIL", T::Numerical),
        ffi::IDA_GETY_BADT => ("IDA_GETY_BADT", T::Invalid),
        _ => ("IDA_UNKNOWN", T::Inconclusive),
    };
    NativeTermination {
        code: i64::from(flag),
        name: name.into(),
        message: None,
        category,
        assurance: Assurance::None,
    }
}
/// Setup, option and retrieval calls return module-specific flags (IDA, IDALS or
/// SUNErrCode); a nonzero flag there is an adapter invariant failure.
fn check(code: i32, operation: &str) -> Result<(), ProblemError> {
    if code == 0 {
        Ok(())
    } else {
        Err(ProblemError::internal(format!(
            "IDAS {operation}: native flag {code}"
        )))
    }
}
fn index(n: usize) -> Result<ffi::sunindextype, ProblemError> {
    n.try_into()
        .map_err(|_| ProblemError::unsupported("IDAS index extent"))
}
/// Trajectory stop implied by a typed failure; stops are never reported as failures.
fn stopped(error: &ProblemError, execution: &Execution) -> Termination {
    match crate::callback::classify(error) {
        crate::callback::Failure::Stopped(crate::solve::Termination::Cancelled) => {
            Termination::Cancelled
        }
        crate::callback::Failure::Stopped(crate::solve::Termination::TimeLimit) => {
            Termination::TimeLimit
        }
        _ => match execution.stopped() {
            Some(crate::solve::Termination::Cancelled) => Termination::Cancelled,
            Some(crate::solve::Termination::TimeLimit) => Termination::TimeLimit,
            _ => Termination::Failed,
        },
    }
}
unsafe fn read(v: ffi::N_Vector, n: usize) -> Vec<f64> {
    unsafe { std::slice::from_raw_parts(ffi::N_VGetArrayPointer(v), n).to_vec() }
}
unsafe fn write(v: ffi::N_Vector, values: &[f64]) {
    unsafe {
        std::ptr::copy_nonoverlapping(values.as_ptr(), ffi::N_VGetArrayPointer(v), values.len());
    }
}
/// `J · [S; P]`: every function row's derivative along each integration parameter, from
/// the state sensitivities `columns` (one per integration parameter) and the direct
/// partials, where `P` selects each contract parameter's integration column `map[k]` in the
/// current segment. One faer sparse × dense product (F12).
fn chained(j: &Jacobian, columns: &[Vec<f64>], n: usize, map: &[usize]) -> faer::Mat<f64> {
    let np = columns.len();
    let chain = faer::Mat::from_fn(n + map.len(), np, |i, k| {
        if i < n {
            columns[k][i]
        } else {
            f64::from(map[i - n] == k)
        }
    });
    let mut result = faer::Mat::zeros(j.nrows(), np);
    faer::sparse::linalg::matmul::sparse_dense_matmul(
        result.as_mut(),
        faer::Accum::Replace,
        j.as_ref(),
        chain.as_ref(),
        1.0,
        faer::Par::Seq,
    );
    result
}
fn finish_callback(c: &mut Context<'_>, result: std::thread::Result<i32>) -> i32 {
    match result {
        Ok(flag) => flag,
        Err(_) => {
            c.callback.terminal = Some((
                crate::solve::Termination::Panic,
                "panic in IDAS callback".into(),
            ));
            -1
        }
    }
}
// All oracle calls cross CallbackState's catch_unwind boundary; buffers publish only on success.
unsafe extern "C" fn residual(
    t: f64,
    y: ffi::N_Vector,
    dy: ffi::N_Vector,
    out: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        let x = unsafe { read(y, n) };
        let Some(e) = c.evaluate(Function::Rhs, t, &x, false) else {
            return c.failure();
        };
        let yp = unsafe { read(dy, n) };
        let r: Vec<_> = e
            .values
            .iter()
            .enumerate()
            .map(|(i, f)| {
                if c.contract.differential[i] {
                    yp[i] - f
                } else {
                    -f
                }
            })
            .collect();
        unsafe {
            write(out, &r);
        }
        0
    }));
    finish_callback(c, result)
}
/// Newton matrix `cj·M − ∂f/∂y` on the admitted pattern, filled by one traversal of the
/// analytic Jacobian's state columns (F12). A partial outside the declared support is a
/// contract violation, never silently dropped.
unsafe extern "C" fn jacobian(
    t: f64,
    cj: f64,
    y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    _r: ffi::N_Vector,
    matrix: ffi::SUNMatrix,
    data: *mut c_void,
    _a: ffi::N_Vector,
    _b: ffi::N_Vector,
    _d: ffi::N_Vector,
) -> i32 {
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        let x = unsafe { read(y, n) };
        let Some(j) = c.partials(t, &x) else {
            return c.failure();
        };
        let mut values = vec![0.0; c.rows.len()];
        let symbolic = j.symbolic();
        for col in 0..n {
            let range = c.columns[col] as usize..c.columns[col + 1] as usize;
            let pattern = &c.rows[range.clone()];
            let slot = |row: usize| {
                pattern
                    .binary_search(&(row as ffi::sunindextype))
                    .ok()
                    .map(|k| range.start + k)
            };
            if c.contract.differential[col] {
                if let Some(k) = slot(col) {
                    values[k] += cj;
                }
            }
            for k in symbolic.col_range(col) {
                let Some(target) = slot(symbolic.row_idx()[k]) else {
                    c.callback.terminal = Some((
                        crate::solve::Termination::Evaluation,
                        "IDAS residual partial outside its declared support".into(),
                    ));
                    return -1;
                };
                values[target] -= j.val()[k];
            }
        }
        unsafe {
            std::ptr::copy_nonoverlapping(
                c.columns.as_ptr(),
                ffi::SUNSparseMatrix_IndexPointers(matrix),
                c.columns.len(),
            );
            std::ptr::copy_nonoverlapping(
                c.rows.as_ptr(),
                ffi::SUNSparseMatrix_IndexValues(matrix),
                c.rows.len(),
            );
            std::ptr::copy_nonoverlapping(
                values.as_ptr(),
                ffi::SUNSparseMatrix_Data(matrix),
                values.len(),
            );
        }
        0
    }));
    finish_callback(c, result)
}
/// Analytic Newton-matrix product `(cj·M − ∂f/∂y)·v` for the matrix-free Krylov routes.
unsafe extern "C" fn jtimes(
    t: f64,
    y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    _r: ffi::N_Vector,
    v: ffi::N_Vector,
    jv: ffi::N_Vector,
    cj: f64,
    data: *mut c_void,
    _a: ffi::N_Vector,
    _b: ffi::N_Vector,
) -> i32 {
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        let x = unsafe { read(y, n) };
        let Some(j) = c.partials(t, &x) else {
            return c.failure();
        };
        let direction = unsafe { read(v, n) };
        let mut extended = vec![0.0; j.ncols()];
        extended[..n].copy_from_slice(&direction);
        let mut product = vec![0.0; n];
        faer::sparse::linalg::matmul::sparse_dense_matmul(
            faer::MatMut::from_column_major_slice_mut(&mut product, n, 1),
            faer::Accum::Replace,
            j.as_ref(),
            faer::MatRef::from_column_major_slice(&extended, extended.len(), 1),
            1.0,
            faer::Par::Seq,
        );
        let out: Vec<_> = (0..n)
            .map(|i| {
                let mass = if c.contract.differential[i] { cj } else { 0.0 };
                mass * direction[i] - product[i]
            })
            .collect();
        unsafe {
            write(jv, &out);
        }
        0
    }));
    finish_callback(c, result)
}
/// Jacobi preconditioner setup from the compiled Newton-matrix diagonal.
unsafe extern "C" fn precondition_setup(
    t: f64,
    y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    _r: ffi::N_Vector,
    cj: f64,
    data: *mut c_void,
) -> i32 {
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        let x = unsafe { read(y, n) };
        let Some(j) = c.partials(t, &x) else {
            return c.failure();
        };
        let symbolic = j.symbolic();
        for col in 0..n {
            let partial = symbolic
                .col_range(col)
                .find(|k| symbolic.row_idx()[*k] == col)
                .map_or(0.0, |k| j.val()[k]);
            let diagonal = if c.contract.differential[col] {
                cj
            } else {
                0.0
            } - partial;
            c.inverse_diagonal[col] = if diagonal.is_finite() && diagonal.abs() > f64::MIN_POSITIVE
            {
                diagonal.recip()
            } else {
                1.0
            };
        }
        0
    }));
    finish_callback(c, result)
}
/// Jacobi preconditioner solve `P z = r`.
unsafe extern "C" fn precondition_solve(
    _t: f64,
    _y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    _r: ffi::N_Vector,
    rvec: ffi::N_Vector,
    zvec: ffi::N_Vector,
    _cj: f64,
    _delta: f64,
    data: *mut c_void,
) -> i32 {
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let r = unsafe { read(rvec, c.inverse_diagonal.len()) };
        let z: Vec<_> = r
            .iter()
            .zip(&c.inverse_diagonal)
            .map(|(r, d)| r * d)
            .collect();
        unsafe {
            write(zvec, &z);
        }
        0
    }));
    finish_callback(c, result)
}
/// Forward-sensitivity residuals `M·ṡ − ∂f/∂y·s − ∂f/∂p` for every parameter at once: one
/// faer sparse × dense product replaces the dense (row, column) loops (F12).
unsafe extern "C" fn sensitivities(
    np: i32,
    t: f64,
    y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    _r: ffi::N_Vector,
    ys: *mut ffi::N_Vector,
    yps: *mut ffi::N_Vector,
    rs: *mut ffi::N_Vector,
    data: *mut c_void,
    _a: ffi::N_Vector,
    _b: ffi::N_Vector,
    _d: ffi::N_Vector,
) -> i32 {
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        if np < 0 || np as usize != c.integration.len() {
            return -1;
        }
        let np = np as usize;
        let x = unsafe { read(y, n) };
        let Some(j) = c.partials(t, &x) else {
            return c.failure();
        };
        let columns: Vec<_> = (0..np).map(|p| unsafe { read(*ys.add(p), n) }).collect();
        let product = chained(&j, &columns, n, &c.map);
        for p in 0..np {
            let ds = unsafe { read(*yps.add(p), n) };
            let residual: Vec<_> = (0..n)
                .map(|row| {
                    let rate = if c.contract.differential[row] {
                        ds[row]
                    } else {
                        0.0
                    };
                    rate - product[(row, p)]
                })
                .collect();
            unsafe {
                write(*rs.add(p), &residual);
            }
        }
        0
    }));
    finish_callback(c, result)
}
unsafe extern "C" fn quadrature(
    t: f64,
    y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    out: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let x = unsafe { read(y, c.contract.states.len()) };
        let Some(e) = c.evaluate(Function::QuadratureFlux, t, &x, false) else {
            return c.failure();
        };
        unsafe {
            write(out, &e.values);
        }
        0
    }));
    finish_callback(c, result)
}
unsafe extern "C" fn roots(
    t: f64,
    y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    out: *mut f64,
    data: *mut c_void,
) -> i32 {
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let x = unsafe { read(y, c.contract.states.len()) };
        let Some(e) = c.evaluate(Function::Roots, t, &x, false) else {
            return -1;
        };
        unsafe {
            std::ptr::copy_nonoverlapping(e.values.as_ptr(), out, e.values.len());
        }
        0
    }));
    finish_callback(c, result)
}
struct Session<'a> {
    ctx: ffi::SUNContext,
    mem: *mut c_void,
    matrix: ffi::SUNMatrix,
    linear: ffi::SUNLinearSolver,
    vectors: Vec<ffi::N_Vector>,
    y: ffi::N_Vector,
    dy: ffi::N_Vector,
    quad: ffi::N_Vector,
    sens: Vec<ffi::N_Vector>,
    dsens: Vec<ffi::N_Vector>,
    /// Output quadratures of finished segments; IDAS restarts its quadrature at each
    /// reinitialization.
    totals: Vec<f64>,
    callback: Box<Context<'a>>,
    _local: PhantomData<Rc<()>>,
}
impl Drop for Session<'_> {
    fn drop(&mut self) {
        unsafe {
            if !self.mem.is_null() {
                ffi::IDAFree(&raw mut self.mem);
            }
            if !self.linear.is_null() {
                ffi::SUNLinSolFree(self.linear);
            }
            if !self.matrix.is_null() {
                ffi::SUNMatDestroy(self.matrix);
            }
            for v in self.vectors.drain(..) {
                ffi::N_VDestroy(v);
            }
            if !self.ctx.is_null() {
                ffi::SUNContext_Free(&raw mut self.ctx);
            }
        }
    }
}
/// Consistent-initialization option of `IDACalcIC`.
fn initialization_option(start: IdasInitialization) -> i32 {
    match start {
        IdasInitialization::AlgebraicAndRates => ffi::IDA_YA_YDP_INIT,
        IdasInitialization::SteadyStates => ffi::IDA_Y_INIT,
    }
}
impl<'a> Session<'a> {
    fn vector(&mut self, values: &[f64]) -> Result<ffi::N_Vector, ProblemError> {
        let v = unsafe { ffi::N_VNew_Serial(index(values.len())?, self.ctx) };
        if v.is_null() {
            return Err(ProblemError::memory("IDAS vector allocation"));
        }
        self.vectors.push(v);
        unsafe {
            write(v, values);
        }
        Ok(v)
    }
    fn new(
        oracle: &'a mut dyn Oracle,
        integration: &[f64],
        p: &Profile,
        execution: Execution,
    ) -> Result<Self, ProblemError> {
        let c = oracle.contract().clone();
        let n = c.states.len();
        let map = p.columns_at(c.parameters.len(), p.start);
        let mut pattern: Vec<(usize, usize)> = (0..n).map(|i| (i, i)).collect();
        for mode in 0..c.events.len() {
            pattern.extend(
                oracle
                    .support(mode, Function::Rhs)
                    .into_iter()
                    .map(|entry| (entry.row.get(), entry.col.get()))
                    .filter(|&(row, col)| col < n && row < n)
                    .map(|(row, col)| (col, row)),
            );
        }
        pattern.sort_unstable();
        pattern.dedup();
        let mut columns = vec![0];
        let mut rows = Vec::with_capacity(pattern.len());
        let mut next = pattern.iter().peekable();
        for col in 0..n {
            while let Some(&(_, row)) = next.next_if(|(c, _)| *c == col) {
                rows.push(index(row)?);
            }
            columns.push(index(rows.len())?);
        }
        let mut s = Self {
            ctx: std::ptr::null_mut(),
            mem: std::ptr::null_mut(),
            matrix: std::ptr::null_mut(),
            linear: std::ptr::null_mut(),
            y: std::ptr::null_mut(),
            dy: std::ptr::null_mut(),
            quad: std::ptr::null_mut(),
            vectors: vec![],
            sens: vec![],
            dsens: vec![],
            totals: vec![0.0; c.quadratures.len()],
            callback: Box::new(Context {
                oracle,
                contract: c,
                parameters: p.parameters_at(integration, p.start),
                integration: integration.to_vec(),
                map,
                mode: 0,
                trial_policy: p.trial_failures,
                callback: CallbackState::new(execution),
                columns,
                rows,
                inverse_diagonal: vec![1.0; n],
            }),
            _local: PhantomData,
        };
        unsafe {
            check(ffi::SUNContext_Create(0, &raw mut s.ctx), "context")?;
        }
        let Some(initial) =
            s.callback
                .evaluate(Function::Initial, p.start, &vec![0.0; n], p.forward())
        else {
            return Err(s.callback.failed("initial function"));
        };
        s.y = s.vector(&initial.values)?;
        s.dy = s.vector(&vec![0.0; n])?;
        let ids: Vec<_> = s
            .callback
            .contract
            .differential
            .iter()
            .map(|v| f64::from(*v))
            .collect();
        let id = s.vector(&ids)?;
        let atol = s.vector(&p.atol)?;
        unsafe {
            s.mem = ffi::IDACreate(s.ctx);
            if s.mem.is_null() {
                return Err(ProblemError::memory("IDAS memory allocation"));
            }
            check(
                ffi::IDAInit(s.mem, Some(residual), p.start, s.y, s.dy),
                "initialization",
            )?;
            check(
                ffi::IDASetUserData(s.mem, (&raw mut *s.callback).cast()),
                "user data",
            )?;
            check(ffi::IDASetId(s.mem, id), "differential identities")?;
            check(
                ffi::IDASVtolerances(s.mem, p.rtol, atol),
                "state tolerances",
            )?;
            check(ffi::IDASetInitStep(s.mem, p.initial_step), "initial step")?;
        }
        s.linear_solver(p.idas.linear)?;
        if p.idas.constraints.iter().any(|v| *v != StateSign::Free) {
            let codes: Vec<_> = p
                .idas
                .constraints
                .iter()
                .map(|v| state_sign_code(*v))
                .collect();
            let constraints = s.vector(&codes)?;
            unsafe {
                check(
                    ffi::IDASetConstraints(s.mem, constraints),
                    "sign constraints",
                )?;
            }
        }
        if p.forward() {
            let j = initial
                .jacobian
                .ok_or_else(|| ProblemError::internal("IDAS initial sensitivity missing"))?;
            let symbolic = j.symbolic();
            // Each integration parameter starts from its contract parameter's initial
            // partials when the first segment uses it, and from zero otherwise.
            for k in 0..integration.len() {
                let mut values = vec![0.0; n];
                if let Some(contract) = s.callback.map.iter().position(|c| *c == k) {
                    for e in symbolic.col_range(n + contract) {
                        values[symbolic.row_idx()[e]] = j.val()[e];
                    }
                }
                let v = s.vector(&values)?;
                s.sens.push(v);
                let v = s.vector(&vec![0.0; n])?;
                s.dsens.push(v);
            }
            unsafe {
                check(
                    ffi::IDASensInit(
                        s.mem,
                        integration
                            .len()
                            .try_into()
                            .map_err(|_| ProblemError::unsupported("IDAS parameter extent"))?,
                        corrector(p.idas.sensitivity),
                        Some(sensitivities),
                        s.sens.as_mut_ptr(),
                        s.dsens.as_mut_ptr(),
                    ),
                    "forward sensitivities",
                )?;
                let mut atol_s = Vec::new();
                for scale in &p.integration_parameters(&p.parameter_scales) {
                    let tolerances: Vec<_> = p.atol.iter().map(|v| v / scale).collect();
                    atol_s.push(s.vector(&tolerances)?);
                }
                check(
                    ffi::IDASensSVtolerances(s.mem, p.rtol, atol_s.as_mut_ptr()),
                    "sensitivity tolerances",
                )?;
                check(ffi::IDASetSensErrCon(s.mem, 1), "sensitivity error control")?;
            }
        }
        if !s.callback.contract.quadratures.is_empty() {
            s.quad = s.vector(&vec![0.0; s.callback.contract.quadratures.len()])?;
            let atol = s.vector(&p.out_atol)?;
            unsafe {
                check(
                    ffi::IDAQuadInit(s.mem, Some(quadrature), s.quad),
                    "quadrature",
                )?;
                check(
                    ffi::IDAQuadSVtolerances(
                        s.mem,
                        p.out_rtol
                            .ok_or_else(|| contract("quadrature relative tolerance"))?,
                        atol,
                    ),
                    "quadrature tolerances",
                )?;
                check(ffi::IDASetQuadErrCon(s.mem, 1), "quadrature error control")?;
            }
        }
        s.initialize_roots()?;
        s.consistent(p.start, p, initialization_option(p.idas.initialization))?;
        Ok(s)
    }
    /// Direct KLU over the compiled Jacobian, or matrix-free SPGMR/SPFGMR over analytic
    /// products with an optional Jacobi left preconditioner (IDAS supports left only).
    fn linear_solver(&mut self, linear: IdasLinear) -> Result<(), ProblemError> {
        let n = self.callback.contract.states.len();
        unsafe {
            match linear {
                IdasLinear::Klu => {
                    self.matrix = ffi::SUNSparseMatrix(
                        index(n)?,
                        index(n)?,
                        index(self.callback.rows.len())?,
                        0,
                        self.ctx,
                    );
                    if self.matrix.is_null() {
                        return Err(ProblemError::memory("IDAS matrix allocation"));
                    }
                    self.linear = ffi::SUNLinSol_KLU(self.y, self.matrix, self.ctx);
                    if self.linear.is_null() {
                        return Err(ProblemError::memory("IDAS KLU allocation"));
                    }
                    check(
                        ffi::IDASetLinearSolver(self.mem, self.linear, self.matrix),
                        "KLU",
                    )?;
                    check(
                        ffi::IDASetJacFn(self.mem, Some(jacobian)),
                        "analytic Jacobian",
                    )?;
                }
                IdasLinear::Spgmr {
                    dimension,
                    preconditioner,
                }
                | IdasLinear::Spfgmr {
                    dimension,
                    preconditioner,
                } => {
                    let dimension = i32::try_from(dimension.into_inner())
                        .map_err(|_| contract("IDAS Krylov dimension"))?;
                    let side = match preconditioner {
                        Preconditioner::None => ffi::SUN_PREC_NONE,
                        Preconditioner::Jacobi => ffi::SUN_PREC_LEFT,
                    } as i32;
                    self.linear = if matches!(linear, IdasLinear::Spgmr { .. }) {
                        ffi::SUNLinSol_SPGMR(self.y, side, dimension, self.ctx)
                    } else {
                        ffi::SUNLinSol_SPFGMR(self.y, side, dimension, self.ctx)
                    };
                    if self.linear.is_null() {
                        return Err(ProblemError::memory("IDAS Krylov allocation"));
                    }
                    check(
                        ffi::IDASetLinearSolver(self.mem, self.linear, std::ptr::null_mut()),
                        "Krylov linear solver",
                    )?;
                    check(
                        ffi::IDASetJacTimes(self.mem, None, Some(jtimes)),
                        "analytic Jacobian products",
                    )?;
                    if preconditioner == Preconditioner::Jacobi {
                        check(
                            ffi::IDASetPreconditioner(
                                self.mem,
                                Some(precondition_setup),
                                Some(precondition_solve),
                            ),
                            "Jacobi preconditioner",
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
    /// Root functions of the active mode, with their declared crossing directions.
    fn initialize_roots(&mut self) -> Result<(), ProblemError> {
        let events = &self.callback.contract.events[self.callback.mode];
        let mut directions: Vec<i32> = events.iter().map(|e| e.direction.code()).collect();
        let count = events.len();
        unsafe {
            check(
                ffi::IDARootInit(
                    self.mem,
                    count
                        .try_into()
                        .map_err(|_| ProblemError::unsupported("IDAS root extent"))?,
                    if count == 0 { None } else { Some(roots) },
                ),
                "roots",
            )?;
            if directions.iter().any(|d| *d != 0) {
                check(
                    ffi::IDASetRootDirection(self.mem, directions.as_mut_ptr()),
                    "root directions",
                )?;
            }
        }
        Ok(())
    }
    fn consistent(&mut self, t: f64, p: &Profile, option: i32) -> Result<(), ProblemError> {
        // `tout1` only orients and scales the initialization step; nothing is integrated.
        let toward = if p.end > t {
            t + (p.end - t).min(p.initial_step)
        } else {
            t + p.initial_step
        };
        let flag = unsafe { ffi::IDACalcIC(self.mem, option, toward) };
        if flag < 0 {
            return Err(self
                .callback
                .native_failure(flag, "consistent initial conditions"));
        }
        unsafe {
            check(
                ffi::IDAGetConsistentIC(self.mem, self.y, self.dy),
                "consistent state retrieval",
            )?;
            if p.forward() {
                check(
                    ffi::IDAGetSensConsistentIC(
                        self.mem,
                        self.sens.as_mut_ptr(),
                        self.dsens.as_mut_ptr(),
                    ),
                    "consistent sensitivity retrieval",
                )?;
            }
        }
        Ok(())
    }
    /// Restart the same native memory at a scheduled change or an event reset: the
    /// differential states (and their sensitivities) carry over, `IDACalcIC` recomputes
    /// the algebraic states and every rate under the new parameters or mode.
    fn restart(
        &mut self,
        t: f64,
        state: &[f64],
        p: &Profile,
        mode_changed: bool,
    ) -> Result<(), ProblemError> {
        unsafe {
            if p.forward() {
                // The carried sensitivities and their rates (the rates are only guesses
                // for `IDACalcIC`).
                let mut time = t;
                check(
                    ffi::IDAGetSens(self.mem, &raw mut time, self.sens.as_mut_ptr()),
                    "segment sensitivities",
                )?;
                check(
                    ffi::IDAGetSensDky(self.mem, t, 1, self.dsens.as_mut_ptr()),
                    "segment sensitivity rates",
                )?;
            }
            if !self.quad.is_null() {
                let mut time = t;
                check(
                    ffi::IDAGetQuad(self.mem, &raw mut time, self.quad),
                    "segment quadrature",
                )?;
                let finished = read(self.quad, self.totals.len());
                for (total, value) in self.totals.iter_mut().zip(finished) {
                    *total += value;
                }
            }
            write(self.y, state);
            check(
                ffi::IDAReInit(self.mem, t, self.y, self.dy),
                "reinitialization",
            )?;
            if p.forward() {
                check(
                    ffi::IDASensReInit(
                        self.mem,
                        corrector(p.idas.sensitivity),
                        self.sens.as_mut_ptr(),
                        self.dsens.as_mut_ptr(),
                    ),
                    "sensitivity reinitialization",
                )?;
            }
            if !self.quad.is_null() {
                write(self.quad, &vec![0.0; self.totals.len()]);
                check(
                    ffi::IDAQuadReInit(self.mem, self.quad),
                    "quadrature reinitialization",
                )?;
            }
        }
        if mode_changed {
            self.initialize_roots()?;
        }
        self.consistent(t, p, ffi::IDA_YA_YDP_INIT)
    }
    fn sample(&mut self, t: f64, p: &Profile, stepped: bool) -> Result<Sample, ProblemError> {
        let n = self.callback.contract.states.len();
        let x = unsafe { read(self.y, n) };
        let Some(e) = self
            .callback
            .evaluate(Function::Output, t, &x, p.forward())
        else {
            return Err(self.callback.failed("output"));
        };
        let mut state_sensitivities = Vec::new();
        let mut output_sensitivities = Vec::new();
        if p.forward() {
            let mut time = t;
            if stepped {
                unsafe {
                    check(
                        ffi::IDAGetSens(self.mem, &raw mut time, self.sens.as_mut_ptr()),
                        "sensitivity output",
                    )?;
                }
            }
            let columns: Vec<_> = self.sens.iter().map(|v| unsafe { read(*v, n) }).collect();
            state_sensitivities = (0..n)
                .flat_map(|i| columns.iter().map(move |c| c[i]))
                .collect();
            let j = e
                .jacobian
                .as_ref()
                .ok_or_else(|| ProblemError::internal("IDAS output derivative missing"))?;
            let product = chained(j, &columns, n, &self.callback.map);
            output_sensitivities = (0..e.values.len())
                .flat_map(|row| (0..columns.len()).map(move |k| (row, k)))
                .map(|(row, k)| product[(row, k)])
                .collect();
        }
        let integrals = if self.quad.is_null() {
            vec![]
        } else {
            let mut time = t;
            unsafe {
                if stepped {
                    check(
                        ffi::IDAGetQuad(self.mem, &raw mut time, self.quad),
                        "quadrature output",
                    )?;
                }
                read(self.quad, self.totals.len())
                    .iter()
                    .zip(&self.totals)
                    .map(|(v, total)| v + total)
                    .collect()
            }
        };
        Ok(Sample {
            mode: self.callback.mode,
            time: t,
            state: x,
            outputs: e.values,
            state_sensitivities,
            output_sensitivities,
            integrals,
        })
    }
    /// Native counters of the current segment; IDAS resets them at each reinitialization.
    fn statistics(&self, flag: i32, p: &Profile) -> serde_json::Value {
        let mut value = serde_json::json!({
            "backend": "idas",
            "native_version": crate::sundials_version(),
            "native_flag": flag,
            "derivatives": "analytic state and parameter partials",
            "idas": serde_json::to_value(&p.idas).unwrap_or(serde_json::Value::Null),
        });
        macro_rules! count {
            ($($get:ident => $name:literal),* $(,)?) => {$(
                let mut v: c_long = 0;
                if unsafe { ffi::$get(self.mem, &raw mut v) } == 0 {
                    value[$name] = serde_json::json!(long_counter(v));
                }
            )*};
        }
        count!(
            IDAGetNumSteps => "steps",
            IDAGetNumResEvals => "residual_evaluations",
            IDAGetNumNonlinSolvIters => "nonlinear_iterations",
            IDAGetNumNonlinSolvConvFails => "nonlinear_convergence_failures",
            IDAGetNumErrTestFails => "error_test_failures",
            IDAGetNumBacktrackOps => "initialization_backtracks",
            IDAGetNumGEvals => "root_evaluations",
            IDAGetNumLinIters => "linear_iterations",
            IDAGetNumPrecEvals => "preconditioner_evaluations",
            IDAGetNumJtimesEvals => "jacobian_products",
        );
        value
    }
    /// Integrate one segment toward `stop`, sampling every requested time before it.
    /// Returns the found root index, or `None` at `stop`; step and native limits end the
    /// report without an error.
    fn segment(
        &mut self,
        p: &Profile,
        r: &mut Report,
        stop: f64,
        changing: bool,
        budget: c_long,
    ) -> Result<(Option<usize>, c_long), ProblemError> {
        let mut steps: c_long = 0;
        r.statistics.push(self.statistics(0, p));
        loop {
            let target = p
                .samples
                .get(r.samples.len())
                .copied()
                .filter(|t| *t < stop)
                .unwrap_or(stop);
            if steps >= budget {
                r.termination = Termination::StepLimit;
                return Ok((None, steps));
            }
            let mut time = r.completed_time;
            unsafe {
                check(ffi::IDASetStopTime(self.mem, target), "stop time")?;
                check(
                    ffi::IDASetMaxNumSteps(self.mem, budget - steps),
                    "remaining steps",
                )?;
            }
            let flag = unsafe {
                ffi::IDASolve(
                    self.mem,
                    target,
                    &raw mut time,
                    self.y,
                    self.dy,
                    ffi::IDA_NORMAL,
                )
            };
            unsafe {
                check(ffi::IDAGetNumSteps(self.mem, &raw mut steps), "step count")?;
            }
            if let Some(last) = r.statistics.last_mut() {
                *last = self.statistics(flag, p);
            }
            if flag == ffi::IDA_TOO_MUCH_WORK {
                r.termination = Termination::StepLimit;
                return Ok((None, steps));
            }
            if flag < 0 {
                return Err(self.callback.native_failure(flag, "integration step"));
            }
            r.completed_time = time;
            let execution = &self.callback.callback.execution;
            execution.progress.push(crate::solve::Event {
                phase: "idas.output".into(),
                elapsed: execution.started.elapsed(),
                values: std::collections::BTreeMap::from([
                    ("time".into(), crate::solve::Metric::Real(time)),
                    (
                        "steps".into(),
                        crate::solve::Metric::Integer(long_counter(steps)),
                    ),
                ]),
                incumbent: None,
            });
            if flag == ffi::IDA_ROOT_RETURN {
                let events = self.callback.contract.events[self.callback.mode].len();
                let mut found = vec![0; events];
                unsafe {
                    check(
                        ffi::IDAGetRootInfo(self.mem, found.as_mut_ptr()),
                        "root information",
                    )?;
                }
                let index = found
                    .iter()
                    .position(|v| *v != 0)
                    .ok_or_else(|| ProblemError::internal("IDAS root without a root index"))?;
                return Ok((Some(index), steps));
            }
            // A sample at a scheduled change observes the post-change state.
            let deferred = target >= stop && changing;
            if !deferred && p.samples.get(r.samples.len()) == Some(&time) {
                r.samples.push(self.sample(time, p, true)?);
            }
            if target >= stop {
                return Ok((None, steps));
            }
        }
    }
    /// The segment loop: roots, resets and scheduled changes restart the same native
    /// memory; every transition is recorded with its settled post-transition state.
    fn run(&mut self, p: &Profile, r: &mut Report) -> Result<(), ProblemError> {
        let n = self.callback.contract.states.len();
        let max_steps: c_long = p
            .max_steps
            .try_into()
            .map_err(|_| ProblemError::unsupported("IDAS step allowance extent"))?;
        let mut used: c_long = 0;
        let mut time = p.start;
        // Scheduled-input changes bound the segments; each restart applies the next
        // intervals' columns to the same integration parameters (I6).
        let boundaries = p.boundaries();
        let np = self.callback.contract.parameters.len();
        let mut segment = 0;
        loop {
            let state = unsafe { read(self.y, n) };
            settle_transitions(&self.callback.contract, &mut r.events, time, &state)?;
            if !self.callback.contract.events[self.callback.mode].is_empty() {
                let Some(guards) = self.callback.evaluate(Function::Roots, time, &state, false)
                else {
                    return Err(self.callback.failed("root function"));
                };
                if guards_at_zero(
                    &guards.values,
                    &self.callback.contract.events[self.callback.mode],
                ) > 0
                {
                    return Err(contract("initial or post-reset root is ambiguous"));
                }
            }
            // A sample at a transition time observes the restarted state.
            if p.samples.get(r.samples.len()) == Some(&time) {
                r.samples.push(self.sample(time, p, false)?);
            }
            r.completed_time = time;
            if time >= p.end {
                r.termination = Termination::Completed;
                return Ok(());
            }
            let stop = boundaries.get(segment).copied().unwrap_or(p.end);
            let changing = segment < boundaries.len();
            let (root, steps) = self.segment(p, r, stop, changing, max_steps - used)?;
            used += steps;
            if r.termination == Termination::StepLimit {
                return Ok(());
            }
            time = r.completed_time;
            let state = unsafe { read(self.y, n) };
            let mut seed = state.clone();
            if let Some(index) = root {
                let mode = self.callback.mode;
                let Some(guards) = self.callback.evaluate(Function::Roots, time, &state, false)
                else {
                    return Err(self.callback.failed("root function"));
                };
                if guards_at_zero(&guards.values, &self.callback.contract.events[mode]) > 1 {
                    return Err(contract("ambiguous simultaneous dynamic events"));
                }
                let event = self.callback.contract.events[mode][index].clone();
                if r.events.len() >= p.max_events {
                    r.termination = Termination::EventLimit;
                    return Ok(());
                }
                r.events.push(EventRecord {
                    event: Some(event.id),
                    time,
                    before: state.clone(),
                    after: None,
                });
                if event.terminal {
                    if p.samples.get(r.samples.len()) == Some(&time) {
                        r.samples.push(self.sample(time, p, true)?);
                    }
                    r.termination = Termination::Event;
                    return Ok(());
                }
                let Some(reset) =
                    self.callback
                        .evaluate(Function::Reset(index), time, &state, false)
                else {
                    return Err(self.callback.failed("event reset"));
                };
                seed = reset.values;
                self.callback.mode = event.next_mode;
            }
            // Roots precede a scheduled change at the same native stop time.
            let changed = boundaries.get(segment).is_some_and(|b| *b == time);
            if changed {
                if r.events.len() >= p.max_events {
                    r.termination = Termination::EventLimit;
                    return Ok(());
                }
                r.events.push(EventRecord {
                    event: None,
                    time,
                    before: seed.clone(),
                    after: None,
                });
                self.callback.map = p.columns_at(np, time);
                self.callback.parameters = p.parameters_at(&self.callback.integration, time);
                segment += 1;
            }
            if time >= p.end && root.is_none() && !changed {
                r.termination = Termination::Completed;
                return Ok(());
            }
            self.restart(time, &seed, p, root.is_some())?;
        }
    }
}
/// The native sensitivity corrector.
fn corrector(method: SensitivityCorrector) -> i32 {
    match method {
        SensitivityCorrector::Simultaneous => ffi::IDA_SIMULTANEOUS,
        SensitivityCorrector::Staggered => ffi::IDA_STAGGERED,
    }
}

/// Execute the IDAS profile on its owning worker. The caller validated the profile.
pub(super) fn integrate_with_progress(
    oracle: &mut dyn Oracle,
    p: &Profile,
    integration: &[f64],
    cancel: Cancellation,
    progress: Arc<Progress>,
) -> Result<Report, ProblemError> {
    let execution = Execution {
        cancel,
        started: Instant::now(),
        time_limit: p.time_limit,
        progress,
        memory: None,
    };
    let mut r = Report::new(p.start);
    let n = oracle.contract().states.len();
    let failure = |mut report: Report, error: ProblemError| {
        report.termination = stopped(&error, &execution);
        report.error = Some(error);
        (report.progress, report.dropped_progress) = execution.progress.snapshot();
        report
    };
    let initial = crate::quality::contained(|| {
        oracle.evaluate(
            0,
            Function::Initial,
            p.start,
            &vec![0.0; n],
            &p.parameters_at(integration, p.start),
            false,
        )
    });
    let initial = match initial {
        Ok(initial) => initial,
        Err(error) => return Ok(failure(r, error)),
    };
    r.requested_initial = initial.values;
    let mut s = match Session::new(oracle, integration, p, execution.clone()) {
        Ok(session) => session,
        Err(error) => return Ok(failure(r, error)),
    };
    r.consistent_initial = unsafe { read(s.y, n) };
    if let Err(e) = s.run(p, &mut r) {
        r.termination = stopped(&e, &s.callback.callback.execution);
        r.error = Some(e);
    }
    // Terminal-policy and checkpoint stops keep their typed status and cause.
    if let Some((status, _)) = &s.callback.callback.terminal {
        r.termination = match status {
            crate::solve::Termination::Cancelled => Termination::Cancelled,
            crate::solve::Termination::TimeLimit => Termination::TimeLimit,
            crate::solve::Termination::Panic => Termination::Panic,
            _ => Termination::Failed,
        };
        if r.error.is_none() {
            r.error = s.callback.callback.terminal_error();
        }
    }
    (r.progress, r.dropped_progress) = s.callback.callback.execution.progress.snapshot();
    Ok(r)
}

#[allow(
    clippy::useless_conversion,
    reason = "C long width differs between native ABIs"
)]
fn long_counter(value: c_long) -> i64 {
    i64::from(value)
}
