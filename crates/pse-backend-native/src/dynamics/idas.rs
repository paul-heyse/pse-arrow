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
//! per scheduled-input interval, so they cross every scheduled change (I6). The adjoint
//! route (ADR-0110 item 3) records the forward pass with `IDAAdjInit`/`IDASolveF` and
//! integrates one backward problem with `IDASolveB` and its gradient quadrature.
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
/// A second-order adjoint result: the symmetric Hessian over the directions and its
/// largest relative asymmetry before symmetrization.
type SecondOrder = (faer::Mat<f64>, f64);

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
    /// Actual first residual outcome of the current IDACalcIC phase, not a later trial.
    initial_ic_residual: Option<bool>,
    /// CSC pattern of the Newton matrix: the state columns of every mode's support plus
    /// the diagonal, sorted by row within each column.
    columns: Vec<ffi::sunindextype>,
    rows: Vec<ffi::sunindextype>,
    /// CSC pattern of the adjoint Newton matrix `cjB·M + ∂f/∂yᵀ`: the transposed pattern.
    columns_b: Vec<ffi::sunindextype>,
    rows_b: Vec<ffi::sunindextype>,
    /// CSC pattern of the second-order adjoint's block lower-triangular Newton matrix: the
    /// transposed pattern on both diagonal blocks and on the coupling block below them;
    /// empty on the other routes.
    columns_bs: Vec<ffi::sunindextype>,
    rows_bs: Vec<ffi::sunindextype>,
    /// Residual rows grouped so that no two rows of a group depend on a common state: one
    /// weighted Hessian per group yields their coupling columns (a Curtis–Powell–Reid
    /// colouring of the coupling block).
    colors: Vec<Vec<usize>>,
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
                Function::Inventory | Function::Transfer(_) => c.balances.len(),
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
    /// `∇²(Σ weights·f)·d` over the state followed by the contract parameters, from the
    /// oracle's exact weighted Hessian of function `f` (ADR-0110 item 4).
    fn curvature(
        &mut self,
        f: Function,
        t: f64,
        x: &[f64],
        weights: &[f64],
        direction: &[f64],
    ) -> Option<Vec<f64>> {
        let Self {
            oracle,
            contract: c,
            parameters,
            mode,
            callback,
            ..
        } = self;
        let width = c.states.len() + c.parameters.len();
        let value = callback.evaluate("idas.curvature", || {
            let h = oracle.weighted_hessian(*mode, f, t, x, parameters, weights)?;
            if h.nrows() != width || h.ncols() != width {
                return Err(ProblemError::internal("IDAS weighted Hessian dimensions"));
            }
            if h.val().iter().any(|v| !v.is_finite()) {
                return Err(ProblemError::numerical(
                    "IDAS weighted Hessian is nonfinite",
                ));
            }
            symmetric_product(h.as_ref(), direction)
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
    /// The tangent direction of integration column `k` over the state followed by the
    /// contract parameters: the forward state sensitivity `s_k`, then every contract
    /// parameter that takes column `k` in the current segment.
    fn direction(&self, k: usize, sensitivity: &[f64]) -> Vec<f64> {
        sensitivity
            .iter()
            .copied()
            .chain(self.map.iter().map(|column| f64::from(*column == k)))
            .collect()
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
/// Call IDAS or SUNDIALS and require a zero flag, as [`check`] does.
///
/// Every use passes live native objects of one session: its `SUNContext`, its IDAS memory
/// (which owns the adjoint memory and every backward problem), matrices, linear solvers and
/// serial vectors, all freed only when the session drops. Calls run on the session's owning
/// thread, and out-parameters and vector arrays are caller storage that outlives the call.
macro_rules! native {
    ($call:expr, $operation:expr $(,)?) => {{
        // SAFETY: the macro's contract: live session-owned native objects on the owning
        // thread, with caller storage that outlives the call.
        check(unsafe { $call }, $operation)
    }};
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
/// The first `n` values of a serial vector.
///
/// # Safety
/// `v` is a live serial vector of at least `n` values.
unsafe fn read(v: ffi::N_Vector, n: usize) -> Vec<f64> {
    // SAFETY: `v` is a live serial vector (the caller's contract).
    let data = unsafe { ffi::N_VGetArrayPointer(v) };
    // SAFETY: its data array holds at least `n` initialized values.
    unsafe { std::slice::from_raw_parts(data, n) }.to_vec()
}
/// Overwrite the first `values.len()` entries of a serial vector.
///
/// # Safety
/// `v` is a live serial vector of at least `values.len()` values.
unsafe fn write(v: ffi::N_Vector, values: &[f64]) {
    // SAFETY: `v` is a live serial vector (the caller's contract).
    let data = unsafe { ffi::N_VGetArrayPointer(v) };
    // SAFETY: its native data array holds at least `values.len()` values and cannot
    // overlap the Rust slice.
    unsafe { std::ptr::copy_nonoverlapping(values.as_ptr(), data, values.len()) };
}
/// The vector at `k` of a SUNDIALS vector array.
///
/// # Safety
/// `array` holds more than `k` initialized vectors.
unsafe fn at(array: *mut ffi::N_Vector, k: usize) -> ffi::N_Vector {
    // SAFETY: `k` is within the caller's array.
    let entry = unsafe { array.add(k) };
    // SAFETY: as above; the entry is initialized.
    unsafe { *entry }
}
/// Copy a CSC pattern and its values into a SUNDIALS sparse matrix.
///
/// # Safety
/// `matrix` is a live CSC `SUNSparseMatrix` with `columns.len() - 1` columns and room for
/// `rows.len()` nonzeros, and `values` has `rows.len()` entries.
unsafe fn publish(
    matrix: ffi::SUNMatrix,
    columns: &[ffi::sunindextype],
    rows: &[ffi::sunindextype],
    values: &[f64],
) {
    // SAFETY: `matrix` is a live sparse matrix (the caller's contract).
    let pointers = unsafe { ffi::SUNSparseMatrix_IndexPointers(matrix) };
    // SAFETY: the matrix holds one index pointer per column plus one, `columns.len()`, in
    // native storage that cannot overlap the Rust slice.
    unsafe { std::ptr::copy_nonoverlapping(columns.as_ptr(), pointers, columns.len()) };
    // SAFETY: as above.
    let indices = unsafe { ffi::SUNSparseMatrix_IndexValues(matrix) };
    // SAFETY: its nonzero capacity holds the `rows.len()` row indices.
    unsafe { std::ptr::copy_nonoverlapping(rows.as_ptr(), indices, rows.len()) };
    // SAFETY: as above.
    let data = unsafe { ffi::SUNSparseMatrix_Data(matrix) };
    // SAFETY: its nonzero capacity holds the `values.len() == rows.len()` values.
    unsafe { std::ptr::copy_nonoverlapping(values.as_ptr(), data, values.len()) };
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
/// `Jᵀv` over the state followed by the contract parameters: one faer transpose product.
fn transposed(j: &Jacobian, v: &[f64]) -> Vec<f64> {
    let mut out = vec![0.0; j.ncols()];
    faer::sparse::linalg::matmul::sparse_dense_matmul(
        faer::MatMut::from_column_major_slice_mut(&mut out, j.ncols(), 1),
        faer::Accum::Replace,
        j.as_ref().transpose(),
        faer::MatRef::from_column_major_slice(v, v.len(), 1),
        1.0,
        faer::Par::Seq,
    );
    out
}
/// CSC index arrays of `n` columns from sorted `(column, row)` pairs.
#[allow(
    clippy::type_complexity,
    reason = "the two SUNDIALS CSC index arrays, returned together"
)]
fn csc(
    n: usize,
    pairs: &[(usize, usize)],
) -> Result<(Vec<ffi::sunindextype>, Vec<ffi::sunindextype>), ProblemError> {
    let mut columns = vec![0];
    let mut rows = Vec::with_capacity(pairs.len());
    let mut next = pairs.iter().peekable();
    for col in 0..n {
        while let Some(&(_, row)) = next.next_if(|(c, _)| *c == col) {
            rows.push(index(row)?);
        }
        columns.push(index(rows.len())?);
    }
    Ok((columns, rows))
}
/// The position of `row` in CSC column `col`.
fn slot(
    columns: &[ffi::sunindextype],
    rows: &[ffi::sunindextype],
    col: usize,
    row: usize,
) -> Option<usize> {
    let range = columns[col] as usize..columns[col + 1] as usize;
    rows[range.clone()]
        .binary_search(&(row as ffi::sunindextype))
        .ok()
        .map(|k| range.start + k)
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
    // SAFETY: `data` is this session's registered `Context`, live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes live serial vectors of the `n` states for the callback.
        let x = unsafe { read(y, n) };
        let evaluation = c.evaluate(Function::Rhs, t, &x, false);
        c.initial_ic_residual.get_or_insert(evaluation.is_some());
        let Some(e) = evaluation else {
            return c.failure();
        };
        // SAFETY: as above.
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
        // SAFETY: `out` is IDAS's residual vector of the `n` states; `r` has `n` values.
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
    // SAFETY: `data` is this session's registered `Context`, live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes a live serial vector of the `n` states for the callback.
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
            if c.contract.differential[col]
                && let Some(k) = slot(col)
            {
                values[k] += cj;
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
        // SAFETY: IDAS passes the KLU matrix `linear_solver` created with `n` columns and
        // room for `c.rows.len()` nonzeros; `values` has one entry per pattern row.
        unsafe { publish(matrix, &c.columns, &c.rows, &values) };
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
    // SAFETY: `data` is this session's registered `Context`, live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes live serial vectors of the `n` states for the callback.
        let x = unsafe { read(y, n) };
        let Some(j) = c.partials(t, &x) else {
            return c.failure();
        };
        // SAFETY: as above.
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
        // SAFETY: as above; `out` has `n` values.
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
    // SAFETY: `data` is this session's registered `Context`, live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes a live serial vector of the `n` states for the callback.
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
    // SAFETY: `data` is this session's registered `Context`, live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: IDAS passes live serial vectors of the `n` states for the callback, and
        // the inverse diagonal has one entry per state.
        let r = unsafe { read(rvec, c.inverse_diagonal.len()) };
        let z: Vec<_> = r
            .iter()
            .zip(&c.inverse_diagonal)
            .map(|(r, d)| r * d)
            .collect();
        // SAFETY: as above.
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
    // SAFETY: `data` is this session's registered `Context`, live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        if np < 0 || np as usize != c.integration.len() {
            return -1;
        }
        let np = np as usize;
        // SAFETY: IDAS passes live serial vectors of the `n` states for the callback.
        let x = unsafe { read(y, n) };
        let Some(j) = c.partials(t, &x) else {
            return c.failure();
        };
        let columns: Vec<_> = (0..np)
            .map(|p| {
                // SAFETY: IDAS passes arrays of `np` sensitivity vectors.
                let v = unsafe { at(ys, p) };
                // SAFETY: each is a live serial vector of the `n` states.
                unsafe { read(v, n) }
            })
            .collect();
        let product = chained(&j, &columns, n, &c.map);
        for p in 0..np {
            // SAFETY: as above.
            let v = unsafe { at(yps, p) };
            // SAFETY: as above.
            let ds = unsafe { read(v, n) };
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
            // SAFETY: as above.
            let out = unsafe { at(rs, p) };
            // SAFETY: as above; `residual` has `n` values.
            unsafe {
                write(out, &residual);
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
    // SAFETY: `data` is this session's registered `Context`, live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: IDAS passes a live serial vector of the states for the callback.
        let x = unsafe { read(y, c.contract.states.len()) };
        let Some(e) = c.evaluate(Function::QuadratureFlux, t, &x, false) else {
            return c.failure();
        };
        // SAFETY: `out` is IDAS's vector of the quadratures, which `evaluate` checked
        // `e.values` against.
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
    // SAFETY: `data` is this session's registered `Context`, live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: IDAS passes a live serial vector of the states for the callback.
        let x = unsafe { read(y, c.contract.states.len()) };
        let Some(e) = c.evaluate(Function::Roots, t, &x, false) else {
            return -1;
        };
        // SAFETY: `out` holds one value per root function of the active mode, which
        // `evaluate` checked `e.values` against; IDAS's array cannot overlap it.
        unsafe {
            std::ptr::copy_nonoverlapping(e.values.as_ptr(), out, e.values.len());
        }
        0
    }));
    finish_callback(c, result)
}
/// The adjoint residual `M·λ' + ∂f/∂yᵀλ` of the backward problem (ADR-0110 item 3): the
/// forward residual is `M·y' − f`, so its adjoint in IDAS's form `(λᵀF_ẏ)' − λᵀF_y` needs
/// only the transposed state partials, one faer product.
unsafe extern "C" fn residual_b(
    t: f64,
    yy: ffi::N_Vector,
    _yp: ffi::N_Vector,
    yb: ffi::N_Vector,
    ypb: ffi::N_Vector,
    rr: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is this session's registered `Context`, live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes live serial vectors of the `n` states (forward) and of the
        // `n` adjoint values (first-order backward problem) for the callback.
        let x = unsafe { read(yy, n) };
        let Some(j) = c.partials(t, &x) else {
            return c.failure();
        };
        // SAFETY: as above.
        let lambda = unsafe { read(yb, n) };
        // SAFETY: as above.
        let rate = unsafe { read(ypb, n) };
        let product = transposed(&j, &lambda);
        let out: Vec<_> = (0..n)
            .map(|i| {
                let mass = if c.contract.differential[i] {
                    rate[i]
                } else {
                    0.0
                };
                mass + product[i]
            })
            .collect();
        // SAFETY: as above; `out` has `n` values.
        unsafe {
            write(rr, &out);
        }
        0
    }));
    finish_callback(c, result)
}
/// Adjoint Newton matrix `∂f/∂yᵀ + cjB·M` on the transposed pattern.
unsafe extern "C" fn jacobian_b(
    t: f64,
    cj: f64,
    yy: ffi::N_Vector,
    _yp: ffi::N_Vector,
    _yb: ffi::N_Vector,
    _ypb: ffi::N_Vector,
    _rr: ffi::N_Vector,
    matrix: ffi::SUNMatrix,
    data: *mut c_void,
    _a: ffi::N_Vector,
    _b: ffi::N_Vector,
    _d: ffi::N_Vector,
) -> i32 {
    // SAFETY: `data` is this session's registered `Context`, live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes a live serial vector of the `n` states for the callback.
        let x = unsafe { read(yy, n) };
        let Some(j) = c.partials(t, &x) else {
            return c.failure();
        };
        let mut values = vec![0.0; c.rows_b.len()];
        let symbolic = j.symbolic();
        for col in 0..n {
            if c.contract.differential[col]
                && let Some(k) = slot(&c.columns_b, &c.rows_b, col, col)
            {
                values[k] += cj;
            }
            // Forward entry (row, col) is adjoint entry (col, row).
            for k in symbolic.col_range(col) {
                let Some(target) = slot(&c.columns_b, &c.rows_b, symbolic.row_idx()[k], col) else {
                    c.callback.terminal = Some((
                        crate::solve::Termination::Evaluation,
                        "IDAS residual partial outside its declared support".into(),
                    ));
                    return -1;
                };
                values[target] += j.val()[k];
            }
        }
        // SAFETY: IDAS passes the KLU matrix `create_backward` made with `n` columns and
        // room for `c.rows_b.len()` nonzeros; `values` has one entry per pattern row.
        unsafe { publish(matrix, &c.columns_b, &c.rows_b, &values) };
        0
    }));
    finish_callback(c, result)
}
/// The gradient integrand `−∂f/∂pᵀλ` over the segment's integration columns; IDAS
/// integrates it from the backward problem's start toward the segment's start, so the
/// quadrature at the start is `∫ ∂f/∂pᵀλ dt` over the segment.
unsafe extern "C" fn quadrature_b(
    t: f64,
    yy: ffi::N_Vector,
    _yp: ffi::N_Vector,
    yb: ffi::N_Vector,
    _ypb: ffi::N_Vector,
    out: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is this session's registered `Context`, live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let c = unsafe { &mut *data.cast::<Context<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes live serial vectors of the `n` states (forward) and of the
        // `n` adjoint values for the callback.
        let x = unsafe { read(yy, n) };
        let Some(j) = c.partials(t, &x) else {
            return c.failure();
        };
        // SAFETY: as above.
        let product = transposed(&j, &unsafe { read(yb, n) });
        let mut rate = vec![0.0; c.integration.len()];
        for (q, column) in c.map.iter().enumerate() {
            rate[*column] = -product[n + q];
        }
        // SAFETY: `out` is the problem's quadrature vector of one value per integration
        // column (`create_backward`), the extent of `rate`.
        unsafe {
            write(out, &rate);
        }
        0
    }));
    finish_callback(c, result)
}
/// The context and integration column of a second-order backward problem's user data.
///
/// # Safety
/// `data` is the address of a live [`Direction`] whose context outlives the call.
unsafe fn directed<'c>(data: *mut c_void) -> (&'c mut Context<'c>, usize) {
    // SAFETY: `data` addresses a live `Direction` (the caller's contract).
    let direction = unsafe { &*data.cast::<Direction>() };
    (
        // SAFETY: its context is the session's live `Context`, not otherwise borrowed
        // while IDAS calls back (the caller's contract).
        unsafe { &mut *direction.context.cast::<Context<'c>>() },
        direction.column,
    )
}
/// The second-order adjoint residual (ADR-0110 item 4) of `[λ; μ]`, where `μ` is the
/// tangent of `λ` along one integration column `k`: `M·λ' + f_xᵀλ` as on the first-order
/// route, and `M·μ' + f_xᵀμ + [∇²(λᵀf)·d]_x` with `d = [s_k; π_k]`, the forward state
/// sensitivity and the contract parameters in effect for column `k`. The forward
/// sensitivities arrive from IDAS's checkpoints (`IDAInitBS`).
unsafe extern "C" fn residual_bs(
    t: f64,
    yy: ffi::N_Vector,
    _yp: ffi::N_Vector,
    yys: *mut ffi::N_Vector,
    _yps: *mut ffi::N_Vector,
    yb: ffi::N_Vector,
    ypb: ffi::N_Vector,
    rr: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is the problem's boxed `Direction`, registered by `create_backward`
    // and live with the session, whose context IDAS reaches only through this callback.
    let (c, column) = unsafe { directed(data) };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes live serial vectors of the `n` forward states and of the
        // `2n` second-order adjoint values for the callback.
        let x = unsafe { read(yy, n) };
        let Some(j) = c.partials(t, &x) else {
            return c.failure();
        };
        // SAFETY: as above.
        let adjoint = unsafe { read(yb, 2 * n) };
        // SAFETY: as above.
        let rate = unsafe { read(ypb, 2 * n) };
        // SAFETY: IDAS passes one forward sensitivity vector per integration column, and
        // `column` is one of them.
        let sensitivity = unsafe { at(yys, column) };
        // SAFETY: each is a live serial vector of the `n` states.
        let d = c.direction(column, &unsafe { read(sensitivity, n) });
        let Some(v) = c.curvature(Function::Rhs, t, &x, &adjoint[..n], &d) else {
            return c.failure();
        };
        let first = transposed(&j, &adjoint[..n]);
        let second = transposed(&j, &adjoint[n..]);
        let mut out = vec![0.0; 2 * n];
        for i in 0..n {
            let mass = if c.contract.differential[i] { 1.0 } else { 0.0 };
            out[i] = mass * rate[i] + first[i];
            out[n + i] = mass * rate[n + i] + second[i] + v[i];
        }
        // SAFETY: `rr` is the problem's residual vector of `2n` values, the extent of `out`.
        unsafe {
            write(rr, &out);
        }
        0
    }));
    finish_callback(c, result)
}
/// Newton matrix of the second-order adjoint (ADR-0110 item 4): `∂f/∂yᵀ + cjB·M` on both
/// diagonal blocks and the exact coupling `∂(∇²(λᵀf)·d)_x/∂λ` below them, whose column j
/// is `(∇²f_j·d)_x`, one weighted Hessian per colour of rows with disjoint state support.
/// The residual is linear, so Newton converges in one correction. A block-diagonal matrix
/// was measured first: it leaves the algebraic coupling uncorrected (its error does not
/// shrink with the step), and a DAE's backward corrector failed to converge thousands of
/// times until the error test failed at the minimum step.
unsafe extern "C" fn jacobian_bs(
    t: f64,
    cj: f64,
    yy: ffi::N_Vector,
    _yp: ffi::N_Vector,
    ys: *mut ffi::N_Vector,
    _yps: *mut ffi::N_Vector,
    _yb: ffi::N_Vector,
    _ypb: ffi::N_Vector,
    _rr: ffi::N_Vector,
    matrix: ffi::SUNMatrix,
    data: *mut c_void,
    _a: ffi::N_Vector,
    _b: ffi::N_Vector,
    _d: ffi::N_Vector,
) -> i32 {
    // SAFETY: `data` is the problem's boxed `Direction`, registered by `create_backward`
    // and live with the session, whose context IDAS reaches only through this callback.
    let (c, column) = unsafe { directed(data) };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes a live serial vector of the `n` forward states.
        let x = unsafe { read(yy, n) };
        let Some(j) = c.partials(t, &x) else {
            return c.failure();
        };
        let outside = |c: &mut Context<'_>| {
            c.callback.terminal = Some((
                crate::solve::Termination::Evaluation,
                "IDAS residual partial outside its declared support".into(),
            ));
            -1
        };
        let mut values = vec![0.0; c.rows_bs.len()];
        let symbolic = j.symbolic();
        for block in [0, n] {
            for col in 0..n {
                if c.contract.differential[col]
                    && let Some(k) = slot(&c.columns_bs, &c.rows_bs, block + col, block + col)
                {
                    values[k] += cj;
                }
                // Forward entry (row, col) is adjoint entry (col, row) in each block.
                for k in symbolic.col_range(col) {
                    let Some(target) = slot(
                        &c.columns_bs,
                        &c.rows_bs,
                        block + symbolic.row_idx()[k],
                        block + col,
                    ) else {
                        return outside(c);
                    };
                    values[target] += j.val()[k];
                }
            }
        }
        // SAFETY: IDAS passes one forward sensitivity vector per integration column, and
        // `column` is one of them.
        let sensitivity = unsafe { at(ys, column) };
        // SAFETY: each is a live serial vector of the `n` states.
        let d = c.direction(column, &unsafe { read(sensitivity, n) });
        for color in c.colors.clone() {
            let mut weights = vec![0.0; n];
            for row in &color {
                weights[*row] = 1.0;
            }
            let Some(v) = c.curvature(Function::Rhs, t, &x, &weights, &d) else {
                return c.failure();
            };
            // Rows of one colour share no state, so each state entry belongs to one row.
            for row in color {
                for k in c.columns_b[row] as usize..c.columns_b[row + 1] as usize {
                    let state = c.rows_b[k] as usize;
                    let Some(target) = slot(&c.columns_bs, &c.rows_bs, row, n + state) else {
                        return outside(c);
                    };
                    values[target] += v[state];
                }
            }
        }
        // SAFETY: IDAS passes the KLU matrix `create_backward` made with `2n` columns and
        // room for `c.rows_bs.len()` nonzeros; `values` has one entry per pattern row.
        unsafe { publish(matrix, &c.columns_bs, &c.rows_bs, &values) };
        0
    }));
    finish_callback(c, result)
}
/// The second-order quadrature: the gradient integrand `−f_pᵀλ`, then the Hessian
/// column's integrand `−(f_pᵀμ + [∇²(λᵀf)·d]_p)`, each over the segment's integration
/// columns.
unsafe extern "C" fn quadrature_bs(
    t: f64,
    yy: ffi::N_Vector,
    _yp: ffi::N_Vector,
    yys: *mut ffi::N_Vector,
    _yps: *mut ffi::N_Vector,
    yb: ffi::N_Vector,
    _ypb: ffi::N_Vector,
    out: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is the problem's boxed `Direction`, registered by `create_backward`
    // and live with the session, whose context IDAS reaches only through this callback.
    let (c, column) = unsafe { directed(data) };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        let width = c.integration.len();
        // SAFETY: IDAS passes live serial vectors of the `n` forward states and of the
        // `2n` second-order adjoint values for the callback.
        let x = unsafe { read(yy, n) };
        let Some(j) = c.partials(t, &x) else {
            return c.failure();
        };
        // SAFETY: as above.
        let adjoint = unsafe { read(yb, 2 * n) };
        // SAFETY: IDAS passes one forward sensitivity vector per integration column, and
        // `column` is one of them.
        let sensitivity = unsafe { at(yys, column) };
        // SAFETY: each is a live serial vector of the `n` states.
        let d = c.direction(column, &unsafe { read(sensitivity, n) });
        let Some(v) = c.curvature(Function::Rhs, t, &x, &adjoint[..n], &d) else {
            return c.failure();
        };
        let first = transposed(&j, &adjoint[..n]);
        let second = transposed(&j, &adjoint[n..]);
        let mut rate = vec![0.0; 2 * width];
        for (q, target) in c.map.iter().enumerate() {
            rate[*target] = -first[n + q];
            rate[width + *target] = -(second[n + q] + v[n + q]);
        }
        // SAFETY: `out` is the problem's quadrature vector of `2·width` values
        // (`create_backward`), the extent of `rate`.
        unsafe {
            write(out, &rate);
        }
        0
    }));
    finish_callback(c, result)
}
/// The segment a backward pass restarts at: its start time and consistent forward state,
/// with the state sensitivities and their rates when the forward pass integrates them.
struct Start {
    time: f64,
    y: Vec<f64>,
    dy: Vec<f64>,
    sensitivities: Vec<Vec<f64>>,
    rates: Vec<Vec<f64>>,
}
/// The user data of one second-order backward problem: the shared callback context and
/// the integration column whose tangent the problem integrates.
struct Direction {
    context: *mut c_void,
    column: usize,
}
/// Native counters of one backward problem, summed over its reinitializations: steps,
/// nonlinear iterations, nonlinear convergence failures and error-test failures.
type Counters = [i64; 4];
/// One IDAS backward problem: the first-order adjoint (n values, `width` gradient
/// quadratures), or on the second-order route the adjoint with its tangent along one
/// direction (2n values; the gradient and then the Hessian column, 2·width quadratures).
struct Problem {
    which: i32,
    matrix: ffi::SUNMatrix,
    linear: ffi::SUNLinearSolver,
    y: ffi::N_Vector,
    yp: ffi::N_Vector,
    quadrature: ffi::N_Vector,
    /// The second-order user data, boxed so its address is stable while IDAS holds it.
    direction: Option<Box<Direction>>,
    counters: Counters,
}
/// The IDAS adjoint of one attempt: `IDAAdjInit` checkpoints and its backward problems,
/// reinitialized at every segment and every observed sample.
struct Backward {
    /// The second-order directions; empty on the first-order route.
    directions: Vec<usize>,
    /// The backward problems, created at the first restart.
    problems: Vec<Problem>,
    /// Forward state and rates at a restart time, for `IDACalcICB`/`IDACalcICBS`.
    forward: ffi::N_Vector,
    forward_rate: ffi::N_Vector,
    /// Forward state sensitivities at a restart time and their rates, which the
    /// second-order residual does not read.
    forward_sensitivities: Vec<ffi::N_Vector>,
    forward_sensitivity_rates: Vec<ffi::N_Vector>,
    /// Every segment's consistent start, from the forward pass.
    starts: Vec<Start>,
    /// Most checkpoints held at once, and the allowance.
    checkpoints: usize,
    limit: usize,
}
impl Backward {
    fn second_order(&self) -> bool {
        !self.directions.is_empty()
    }
}
/// The adjoint values and quadratures of every backward problem.
type Adjoints = (Vec<Vec<f64>>, Vec<Vec<f64>>);
/// The forward state, its rates and its state sensitivities at a restart time.
type Forward = (Vec<f64>, Vec<f64>, Vec<Vec<f64>>);
struct Session<'a> {
    ctx: ffi::SUNContext,
    mem: *mut c_void,
    matrix: ffi::SUNMatrix,
    linear: ffi::SUNLinearSolver,
    vectors: Vec<ffi::N_Vector>,
    y: ffi::N_Vector,
    dy: ffi::N_Vector,
    quad: ffi::N_Vector,
    /// Forward sensitivities are integrated: requested by the profile, or the tangents
    /// of the second-order route.
    forward: bool,
    sens: Vec<ffi::N_Vector>,
    dsens: Vec<ffi::N_Vector>,
    /// Output quadratures of finished segments; IDAS restarts its quadrature at each
    /// reinitialization.
    totals: Vec<f64>,
    transition: Option<Transition>,
    /// The adjoint route's checkpoints and backward problems.
    adjoint: Option<Backward>,
    callback: Box<Context<'a>>,
    _local: PhantomData<Rc<()>>,
}
impl Drop for Session<'_> {
    fn drop(&mut self) {
        // Every object below was created by this session, is freed exactly once, and the
        // integrator memory goes first so nothing native refers to the rest afterwards.
        // `IDAFree` also frees the adjoint memory and every backward problem.
        if !self.mem.is_null() {
            // SAFETY: the session's live IDAS memory; `IDAFree` nulls the pointer.
            unsafe { ffi::IDAFree(&raw mut self.mem) };
        }
        if !self.linear.is_null() {
            // SAFETY: the session's linear solver, no longer attached to any memory.
            unsafe { ffi::SUNLinSolFree(self.linear) };
        }
        if !self.matrix.is_null() {
            // SAFETY: the session's Newton matrix, freed after the solver that used it.
            unsafe { ffi::SUNMatDestroy(self.matrix) };
        }
        if let Some(b) = &self.adjoint {
            for problem in &b.problems {
                if !problem.linear.is_null() {
                    // SAFETY: as above, for the backward problem's solver.
                    unsafe { ffi::SUNLinSolFree(problem.linear) };
                }
                if !problem.matrix.is_null() {
                    // SAFETY: as above, for the backward problem's matrix.
                    unsafe { ffi::SUNMatDestroy(problem.matrix) };
                }
            }
        }
        for v in self.vectors.drain(..) {
            // SAFETY: every serial vector the session allocated, each listed once.
            unsafe { ffi::N_VDestroy(v) };
        }
        if !self.ctx.is_null() {
            // SAFETY: the session's context, freed last, after every object created in it.
            unsafe { ffi::SUNContext_Free(&raw mut self.ctx) };
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
struct StatePreparation<'a> {
    time: f64,
    parameters: Vec<f64>,
    map: Vec<usize>,
    trial_failures: TrialPolicy,
    rtol: f64,
    atol: &'a [f64],
    initial_step: Option<f64>,
    linear: IdasLinear,
    forward: bool,
}
impl<'a> Session<'a> {
    fn vector(&mut self, values: &[f64]) -> Result<ffi::N_Vector, ProblemError> {
        // SAFETY: the session's live context; the vector is freed with the session.
        let v = unsafe { ffi::N_VNew_Serial(index(values.len())?, self.ctx) };
        if v.is_null() {
            return Err(ProblemError::memory("IDAS vector allocation"));
        }
        self.vectors.push(v);
        // SAFETY: `v` was just allocated with `values.len()` entries.
        unsafe {
            write(v, values);
        }
        Ok(v)
    }
    /// Common native state/role/sign/analytic-linear preparation, before any IC phase.
    fn prepare(
        oracle: &'a mut dyn Oracle,
        integration: &[f64],
        config: StatePreparation<'_>,
        execution: Execution,
    ) -> Result<(Self, Evaluation), ProblemError> {
        let c = oracle.contract().clone();
        let n = c.states.len();
        let map = config.map;
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
        let (columns, rows) = csc(n, &pattern)?;
        let mut transposed: Vec<_> = pattern.iter().map(|&(col, row)| (row, col)).collect();
        transposed.sort_unstable();
        let (columns_b, rows_b) = csc(n, &transposed)?;
        let mut s = Self {
            ctx: std::ptr::null_mut(),
            mem: std::ptr::null_mut(),
            matrix: std::ptr::null_mut(),
            linear: std::ptr::null_mut(),
            y: std::ptr::null_mut(),
            dy: std::ptr::null_mut(),
            quad: std::ptr::null_mut(),
            vectors: vec![],
            forward: config.forward,
            sens: vec![],
            dsens: vec![],
            totals: vec![0.0; c.quadratures.len()],
            transition: None,
            adjoint: None,
            callback: Box::new(Context {
                oracle,
                contract: c,
                parameters: config.parameters,
                integration: integration.to_vec(),
                map,
                mode: 0,
                trial_policy: config.trial_failures,
                callback: CallbackState::new(execution),
                initial_ic_residual: None,
                columns,
                rows,
                columns_b,
                rows_b,
                columns_bs: Vec::new(),
                rows_bs: Vec::new(),
                colors: Vec::new(),
                inverse_diagonal: vec![1.0; n],
            }),
            _local: PhantomData,
        };
        native!(ffi::SUNContext_Create(0, &raw mut s.ctx), "context")?;
        let Some(initial) = s.callback.evaluate(
            Function::Initial,
            config.time,
            &vec![0.0; n],
            config.forward,
        ) else {
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
        let atol = s.vector(config.atol)?;
        // SAFETY: the session's live context; `Drop` frees the memory.
        s.mem = unsafe { ffi::IDACreate(s.ctx) };
        if s.mem.is_null() {
            return Err(ProblemError::memory("IDAS memory allocation"));
        }
        native!(
            ffi::IDAInit(s.mem, Some(residual), config.time, s.y, s.dy),
            "initialization",
        )?;
        // The boxed context has a stable address for the session's lifetime.
        native!(
            ffi::IDASetUserData(s.mem, (&raw mut *s.callback).cast()),
            "user data",
        )?;
        native!(ffi::IDASetId(s.mem, id), "differential identities")?;
        native!(
            ffi::IDASVtolerances(s.mem, config.rtol, atol),
            "state tolerances",
        )?;
        if let Some(step) = config.initial_step {
            native!(ffi::IDASetInitStep(s.mem, step), "initial step")?;
        }
        s.linear_solver(config.linear)?;
        // The authored bounds' signs keep the steps in the domain (ADR-0119 Outcome 4).
        if s.callback
            .contract
            .signs
            .iter()
            .any(|v| *v != StateSign::Free)
        {
            let codes: Vec<_> = s
                .callback
                .contract
                .signs
                .iter()
                .map(|v| state_sign_code(*v))
                .collect();
            let constraints = s.vector(&codes)?;
            native!(
                ffi::IDASetConstraints(s.mem, constraints),
                "sign constraints",
            )?;
        }
        Ok((s, initial))
    }
    fn new(
        oracle: &'a mut dyn Oracle,
        integration: &[f64],
        p: &Profile,
        directions: &[usize],
        execution: Execution,
    ) -> Result<Self, ProblemError> {
        let forward = p.forward() || !directions.is_empty();
        let map = p.columns_at(oracle.contract().parameters.len(), p.start);
        let (mut s, initial) = Self::prepare(
            oracle,
            integration,
            StatePreparation {
                time: p.start,
                parameters: p.parameters_at(integration, p.start),
                map,
                trial_failures: p.trial_failures,
                rtol: p.rtol,
                atol: &p.atol,
                initial_step: Some(p.initial_step),
                linear: p.idas.linear,
                forward,
            },
            execution,
        )?;
        let n = s.callback.contract.states.len();
        if forward {
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
            native!(
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
            native!(
                ffi::IDASensSVtolerances(s.mem, p.rtol, atol_s.as_mut_ptr()),
                "sensitivity tolerances",
            )?;
            native!(ffi::IDASetSensErrCon(s.mem, 1), "sensitivity error control")?;
        }
        if !s.callback.contract.quadratures.is_empty() {
            s.quad = s.vector(&vec![0.0; s.callback.contract.quadratures.len()])?;
            let atol = s.vector(&p.out_atol)?;
            native!(
                ffi::IDAQuadInit(s.mem, Some(quadrature), s.quad),
                "quadrature",
            )?;
            native!(
                ffi::IDAQuadSVtolerances(
                    s.mem,
                    p.out_rtol
                        .ok_or_else(|| contract("quadrature relative tolerance"))?,
                    atol,
                ),
                "quadrature tolerances",
            )?;
            native!(ffi::IDASetQuadErrCon(s.mem, 1), "quadrature error control")?;
        }
        if p.sensitivity == DynamicSensitivity::Adjoint {
            s.adjoint_init(p, directions)?;
        }
        s.initialize_roots()?;
        s.consistent(p.start, p, initialization_option(p.idas.initialization))?;
        Ok(s)
    }
    /// `IDAAdjInit` with Hermite interpolation and `steps_between_checkpoints` steps
    /// between checkpoints; the forward pass then runs through `IDASolveF`, and stores the
    /// state sensitivities at its checkpoints when the second-order route integrates them.
    fn adjoint_init(&mut self, p: &Profile, directions: &[usize]) -> Result<(), ProblemError> {
        let n = self.callback.contract.states.len();
        let width = self.callback.integration.len();
        let steps: c_long = p
            .adjoint
            .steps_between_checkpoints
            .into_inner()
            .try_into()
            .map_err(|_| ProblemError::unsupported("IDAS checkpoint interval extent"))?;
        native!(
            ffi::IDAAdjInit(self.mem, steps, ffi::IDA_HERMITE),
            "adjoint initialization",
        )?;
        let zeros = vec![0.0; n];
        let mut sensitivities = Vec::new();
        let mut rates = Vec::new();
        if !directions.is_empty() {
            // The transposed pattern of `∂f/∂yᵀ` on both diagonal blocks of the 2n Newton
            // matrix and on the coupling block `∂(∇²(λᵀf)·d)_x/∂λ` below them: column j of
            // the coupling holds `(∇²f_j·d)_x`, nonzero only where f_j depends on the state.
            let (columns, rows) = (&self.callback.columns_b, &self.callback.rows_b);
            let mut pairs = Vec::with_capacity(3 * rows.len());
            for (column, row) in [(0, 0), (0, n), (n, n)] {
                for col in 0..n {
                    for &entry in &rows[columns[col] as usize..columns[col + 1] as usize] {
                        pairs.push((column + col, row + entry as usize));
                    }
                }
            }
            pairs.sort_unstable();
            // Greedy colouring of the residual rows by their state support.
            let mut colors: Vec<Vec<usize>> = Vec::new();
            let mut owned: Vec<Vec<bool>> = Vec::new();
            for j in 0..n {
                let support = &rows[columns[j] as usize..columns[j + 1] as usize];
                let color = owned
                    .iter()
                    .position(|states| support.iter().all(|i| !states[*i as usize]))
                    .unwrap_or_else(|| {
                        owned.push(vec![false; n]);
                        colors.push(Vec::new());
                        owned.len() - 1
                    });
                for i in support {
                    owned[color][*i as usize] = true;
                }
                colors[color].push(j);
            }
            self.callback.colors = colors;
            (self.callback.columns_bs, self.callback.rows_bs) = csc(2 * n, &pairs)?;
            for _ in 0..width {
                sensitivities.push(self.vector(&zeros)?);
                rates.push(self.vector(&zeros)?);
            }
        }
        self.adjoint = Some(Backward {
            directions: directions.to_vec(),
            problems: Vec::new(),
            forward: self.vector(&zeros)?,
            forward_rate: self.vector(&zeros)?,
            forward_sensitivities: sensitivities,
            forward_sensitivity_rates: rates,
            starts: Vec::new(),
            checkpoints: 0,
            limit: p.adjoint.max_checkpoints.into_inner(),
        });
        Ok(())
    }
    /// Direct KLU over the compiled Jacobian, or matrix-free SPGMR/SPFGMR over analytic
    /// products with an optional Jacobi left preconditioner (IDAS supports left only).
    fn linear_solver(&mut self, linear: IdasLinear) -> Result<(), ProblemError> {
        let n = self.callback.contract.states.len();
        match linear {
            IdasLinear::Klu => {
                // SAFETY: the session's live context; `Drop` frees the matrix.
                self.matrix = unsafe {
                    ffi::SUNSparseMatrix(
                        index(n)?,
                        index(n)?,
                        index(self.callback.rows.len())?,
                        0,
                        self.ctx,
                    )
                };
                if self.matrix.is_null() {
                    return Err(ProblemError::memory("IDAS matrix allocation"));
                }
                // SAFETY: the session's live state vector, matrix and context; `Drop` frees
                // the solver.
                self.linear = unsafe { ffi::SUNLinSol_KLU(self.y, self.matrix, self.ctx) };
                if self.linear.is_null() {
                    return Err(ProblemError::memory("IDAS KLU allocation"));
                }
                native!(
                    ffi::IDASetLinearSolver(self.mem, self.linear, self.matrix),
                    "KLU",
                )?;
                native!(
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
                    Preconditioner::BlockFactor => {
                        return Err(ProblemError::Unsupported(
                            "IDAS block factor preconditioning is not supplied".into(),
                        ));
                    }
                } as i32;
                self.linear = if matches!(linear, IdasLinear::Spgmr { .. }) {
                    // SAFETY: the session's live state vector and context; `Drop` frees
                    // the solver.
                    unsafe { ffi::SUNLinSol_SPGMR(self.y, side, dimension, self.ctx) }
                } else {
                    // SAFETY: as above.
                    unsafe { ffi::SUNLinSol_SPFGMR(self.y, side, dimension, self.ctx) }
                };
                if self.linear.is_null() {
                    return Err(ProblemError::memory("IDAS Krylov allocation"));
                }
                native!(
                    ffi::IDASetLinearSolver(self.mem, self.linear, std::ptr::null_mut()),
                    "Krylov linear solver",
                )?;
                native!(
                    ffi::IDASetJacTimes(self.mem, None, Some(jtimes)),
                    "analytic Jacobian products",
                )?;
                if preconditioner == Preconditioner::Jacobi {
                    native!(
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
        Ok(())
    }
    /// Root functions of the active mode, with their declared crossing directions.
    fn initialize_roots(&mut self) -> Result<(), ProblemError> {
        let events = &self.callback.contract.events[self.callback.mode];
        let mut directions: Vec<i32> = events.iter().map(|e| root_direction(e.direction)).collect();
        let count = events.len();
        native!(
            ffi::IDARootInit(
                self.mem,
                count
                    .try_into()
                    .map_err(|_| ProblemError::unsupported("IDAS root extent"))?,
                if count == 0 { None } else { Some(roots) },
            ),
            "roots",
        )?;
        // IDAS copies the `count` directions.
        if directions.iter().any(|d| *d != 0) {
            native!(
                ffi::IDASetRootDirection(self.mem, directions.as_mut_ptr()),
                "root directions",
            )?;
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
        let mode = if option == ffi::IDA_YA_YDP_INIT {
            IdasInitialization::AlgebraicAndRates
        } else {
            IdasInitialization::SteadyStates
        };
        let flag = self.calculate_consistent(toward, mode, &p.idas.initial_conditions)?;
        if flag < 0 {
            return Err(self
                .callback
                .native_failure(flag, "consistent initial conditions"));
        }
        native!(
            ffi::IDAGetConsistentIC(self.mem, self.y, self.dy),
            "consistent state retrieval",
        )?;
        if self.forward {
            native!(
                ffi::IDAGetSensConsistentIC(
                    self.mem,
                    self.sens.as_mut_ptr(),
                    self.dsens.as_mut_ptr(),
                ),
                "consistent sensitivity retrieval",
            )?;
        }
        Ok(())
    }
    /// Configure the native IC phase before calling it, preserving the actual return.
    fn calculate_consistent(
        &mut self,
        toward: f64,
        mode: IdasInitialization,
        controls: &IdasInitialConditions,
    ) -> Result<i32, ProblemError> {
        controls.validate_for(
            mode,
            self.callback
                .contract
                .signs
                .iter()
                .any(|sign| *sign != StateSign::Free),
        )?;
        self.callback.callback.execution.check()?;
        native!(
            ffi::IDASetMaxNumJacsIC(self.mem, controls.jacobian_attempts as i32),
            "IC Jacobian attempts"
        )?;
        native!(
            ffi::IDASetMaxNumItersIC(self.mem, controls.newton_iterations as i32),
            "IC Newton iterations"
        )?;
        native!(
            ffi::IDASetNonlinConvCoefIC(self.mem, controls.convergence_coefficient),
            "IC convergence coefficient"
        )?;
        native!(
            ffi::IDASetLineSearchOffIC(self.mem, i32::from(!controls.line_search)),
            "IC line search"
        )?;
        if let Some(value) = controls.step_trials {
            native!(
                ffi::IDASetMaxNumStepsIC(self.mem, value as i32),
                "IC artificial step trials"
            )?;
        }
        if let Some(value) = controls.backtracks {
            native!(
                ffi::IDASetMaxBacksIC(self.mem, value as i32),
                "IC backtracks"
            )?;
        }
        if let Some(value) = controls.step_tolerance {
            native!(
                ffi::IDASetStepToleranceIC(self.mem, value),
                "IC step tolerance"
            )?;
        }
        self.callback.initial_ic_residual = None;
        // SAFETY: live IDAS memory and contained callbacks on this session's thread.
        Ok(unsafe { ffi::IDACalcIC(self.mem, initialization_option(mode), toward) })
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
        if self.forward {
            // The carried sensitivities and their rates (the rates are only guesses
            // for `IDACalcIC`).
            let mut time = t;
            native!(
                ffi::IDAGetSens(self.mem, &raw mut time, self.sens.as_mut_ptr()),
                "segment sensitivities",
            )?;
            native!(
                ffi::IDAGetSensDky(self.mem, t, 1, self.dsens.as_mut_ptr()),
                "segment sensitivity rates",
            )?;
        }
        if !self.quad.is_null() {
            let mut time = t;
            native!(
                ffi::IDAGetQuad(self.mem, &raw mut time, self.quad),
                "segment quadrature",
            )?;
            // SAFETY: the session's quadrature vector holds one value per total.
            let finished = unsafe { read(self.quad, self.totals.len()) };
            for (total, value) in self.totals.iter_mut().zip(finished) {
                *total += value;
            }
        }
        // SAFETY: the session's state vector holds the `n` states `state` carries.
        unsafe { write(self.y, state) };
        native!(
            ffi::IDAReInit(self.mem, t, self.y, self.dy),
            "reinitialization",
        )?;
        // The adjoint route keeps one segment's checkpoints at a time.
        if self.adjoint.is_some() {
            native!(ffi::IDAAdjReInit(self.mem), "adjoint reinitialization")?;
        }
        if self.forward {
            native!(
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
            // SAFETY: the session's quadrature vector holds one value per total.
            unsafe { write(self.quad, &vec![0.0; self.totals.len()]) };
            native!(
                ffi::IDAQuadReInit(self.mem, self.quad),
                "quadrature reinitialization",
            )?;
        }
        if mode_changed {
            self.initialize_roots()?;
        }
        self.consistent(t, p, ffi::IDA_YA_YDP_INIT)
    }
    fn sample(&mut self, t: f64, stepped: bool) -> Result<Sample, ProblemError> {
        let n = self.callback.contract.states.len();
        // SAFETY: the session's state vector holds the `n` states.
        let x = unsafe { read(self.y, n) };
        let Some(e) = self
            .callback
            .evaluate(Function::Output, t, &x, self.forward)
        else {
            return Err(self.callback.failed("output"));
        };
        let mut state_sensitivities = Vec::new();
        let mut output_sensitivities = Vec::new();
        if self.forward {
            let mut time = t;
            if stepped {
                native!(
                    ffi::IDAGetSens(self.mem, &raw mut time, self.sens.as_mut_ptr()),
                    "sensitivity output",
                )?;
            }
            let columns: Vec<_> = self
                .sens
                .iter()
                // SAFETY: each sensitivity vector of the session holds the `n` states.
                .map(|v| unsafe { read(*v, n) })
                .collect();
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
        let integrals = self.integrals(t, stepped)?;
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
    fn capture_endpoint(
        &mut self,
        r: &mut Report,
        time: f64,
        stepped: bool,
        event: Option<SemanticId>,
    ) -> Result<(), ProblemError> {
        let point = self.sample(time, stepped)?;
        r.endpoint = Some(TrajectoryEndpoint {
            point,
            event,
            input_columns: self.callback.map.clone(),
            inputs: self.callback.parameters.clone(),
        });
        Ok(())
    }
    fn integrals(&mut self, t: f64, stepped: bool) -> Result<Vec<f64>, ProblemError> {
        let integrals = if self.quad.is_null() {
            vec![]
        } else {
            let mut time = t;
            if stepped {
                native!(
                    ffi::IDAGetQuad(self.mem, &raw mut time, self.quad),
                    "quadrature output",
                )?;
            }
            // SAFETY: the session's quadrature vector holds one value per total.
            unsafe { read(self.quad, self.totals.len()) }
                .iter()
                .zip(&self.totals)
                .map(|(v, total)| v + total)
                .collect()
        };
        Ok(integrals)
    }
    fn inventories(&mut self, time: f64, state: &[f64]) -> Result<Vec<f64>, ProblemError> {
        if self.callback.contract.balances.is_empty() {
            return Ok(vec![]);
        }
        self.callback
            .evaluate(Function::Inventory, time, state, false)
            .map(|v| v.values)
            .ok_or_else(|| self.callback.failed("conserved inventory"))
    }
    fn conserve(
        &mut self,
        report: &mut Report,
        time: f64,
        state: &[f64],
        stepped: bool,
    ) -> Result<(), ProblemError> {
        let inventories = self.inventories(time, state)?;
        let integrals = self.integrals(time, stepped)?;
        let transfers = cumulative_transfers(&self.callback.contract, report);
        observe_conservation(
            &self.callback.contract,
            report,
            time,
            self.callback.mode,
            inventories,
            integrals,
            transfers,
        )
    }
    fn transition(
        &mut self,
        report: &Report,
        event: Option<usize>,
        time: f64,
        state: &[f64],
    ) -> Result<(), ProblemError> {
        let mut transfers = vec![0.0; self.callback.contract.balances.len()];
        if let Some(index) = event {
            let id = self.callback.contract.events[self.callback.mode][index].id;
            if self
                .callback
                .contract
                .balances
                .iter()
                .any(|b| b.transfers.contains(&id))
            {
                transfers = self
                    .callback
                    .evaluate(Function::Transfer(index), time, state, false)
                    .map(|v| v.values)
                    .ok_or_else(|| self.callback.failed("event transfer"))?;
                for (value, balance) in transfers.iter_mut().zip(&self.callback.contract.balances) {
                    if !balance.transfers.contains(&id) && *value != 0.0 {
                        return Err(contract(
                            "event transfer supplied for an unauthorized conserved subject",
                        ));
                    }
                }
            }
        }
        self.transition = Some(Transition {
            record: report.events.len() - 1,
            inventories: self.inventories(time, state)?,
            transfers,
        });
        Ok(())
    }
    /// One forward `IDASolve` call, or `IDASolveF` on the adjoint route, which stores
    /// checkpoints: more than `max_checkpoints` at once is a typed memory limit.
    fn solve(&mut self, target: f64, time: &mut f64) -> Result<i32, ProblemError> {
        let Some(b) = self.adjoint.as_mut() else {
            // SAFETY: the session's live IDAS memory and vectors on its owning thread;
            // `time` is caller storage, and the callbacks reach the context only through
            // their user data.
            return Ok(unsafe {
                ffi::IDASolve(self.mem, target, time, self.y, self.dy, ffi::IDA_NORMAL)
            });
        };
        let mut stored = 0;
        // SAFETY: as above; `stored` is a local out-parameter.
        let flag = unsafe {
            ffi::IDASolveF(
                self.mem,
                target,
                time,
                self.y,
                self.dy,
                ffi::IDA_NORMAL,
                &raw mut stored,
            )
        };
        // IDAS counts the checkpoints it adds after the segment's first.
        let held = usize::try_from(stored).unwrap_or(0) + 1;
        b.checkpoints = b.checkpoints.max(held);
        if held > b.limit {
            return Err(ProblemError::memory(
                "the adjoint forward pass needs more than max_checkpoints checkpoints",
            ));
        }
        Ok(flag)
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
                // SAFETY: a counter query of the session's live IDAS memory into a local.
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
            native!(ffi::IDASetStopTime(self.mem, target), "stop time")?;
            native!(
                ffi::IDASetMaxNumSteps(self.mem, budget - steps),
                "remaining steps",
            )?;
            let flag = self.solve(target, &mut time)?;
            native!(ffi::IDAGetNumSteps(self.mem, &raw mut steps), "step count")?;
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
                // One entry per root function of the active mode.
                let mut found = vec![0; events];
                native!(
                    ffi::IDAGetRootInfo(self.mem, found.as_mut_ptr()),
                    "root information",
                )?;
                let index = found
                    .iter()
                    .position(|v| *v != 0)
                    .ok_or_else(|| ProblemError::internal("IDAS root without a root index"))?;
                return Ok((Some(index), steps));
            }
            // A sample at a scheduled change observes the post-change state.
            let deferred = target >= stop && changing;
            if !deferred && p.samples.get(r.samples.len()) == Some(&time) {
                let point = self.sample(time, true)?;
                let inventories = self.inventories(time, &point.state)?;
                let transfers = cumulative_transfers(&self.callback.contract, r);
                observe_conservation(
                    &self.callback.contract,
                    r,
                    time,
                    point.mode,
                    inventories,
                    point.integrals.clone(),
                    transfers,
                )?;
                r.samples.push(point);
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
            // SAFETY: the session's state vector holds the `n` states.
            let state = unsafe { read(self.y, n) };
            let inventories = self.inventories(time, &state)?;
            let integrals = self.integrals(time, false)?;
            if let Some(transition) = self.transition.take() {
                settle_transition(
                    &self.callback.contract,
                    r,
                    transition,
                    ConservationPoint {
                        time,
                        mode: self.callback.mode,
                        inventories,
                        integrals,
                        transfers: vec![],
                        defects: vec![],
                    },
                    &state,
                )?;
            } else {
                let transfers = cumulative_transfers(&self.callback.contract, r);
                observe_conservation(
                    &self.callback.contract,
                    r,
                    time,
                    self.callback.mode,
                    inventories,
                    integrals,
                    transfers,
                )?;
            }
            // A coincident event reset settles first under the terminating segment.
            // Only then does the scheduled segment become active and initialize consistently.
            if boundaries.get(segment) == Some(&time) {
                if r.events.len() >= p.max_events {
                    r.termination = Termination::EventLimit;
                    return Ok(());
                }
                r.events.push(EventRecord {
                    event: None,
                    time,
                    before: state.clone(),
                    after: None,
                });
                self.transition(r, None, time, &state)?;
                self.callback.map = p.columns_at(np, time);
                self.callback.parameters = p.parameters_at(&self.callback.integration, time);
                segment += 1;
                self.restart(time, &state, p, false)?;
                continue;
            }
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
                r.samples.push(self.sample(time, false)?);
            }
            r.completed_time = time;
            if time >= p.end {
                self.capture_endpoint(r, time, false, None)?;
                r.termination = Termination::Completed;
                return Ok(());
            }
            if self.adjoint.is_some() {
                let (sensitivities, rates) = if self.forward {
                    (
                        // SAFETY: each sensitivity vector of the session holds the `n`
                        // states.
                        self.sens.iter().map(|v| unsafe { read(*v, n) }).collect(),
                        // SAFETY: as above, for their rates.
                        self.dsens.iter().map(|v| unsafe { read(*v, n) }).collect(),
                    )
                } else {
                    (Vec::new(), Vec::new())
                };
                // SAFETY: the session's rate vector holds the `n` states.
                let dy = unsafe { read(self.dy, n) };
                self.backward_mut()?.starts.push(Start {
                    time,
                    y: state,
                    dy,
                    sensitivities,
                    rates,
                });
            }
            let stop = boundaries.get(segment).copied().unwrap_or(p.end);
            let changing = segment < boundaries.len();
            let (root, steps) = self.segment(p, r, stop, changing, max_steps - used)?;
            used += steps;
            if r.termination == Termination::StepLimit {
                return Ok(());
            }
            time = r.completed_time;
            // SAFETY: the session's state vector holds the `n` states.
            let state = unsafe { read(self.y, n) };
            let mut seed = state.clone();
            self.conserve(r, time, &state, true)?;
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
                    self.capture_endpoint(r, time, true, Some(event.id))?;
                    if p.samples.get(r.samples.len()) == Some(&time) {
                        let point = self.sample(time, true)?;
                        let inventories = self.inventories(time, &point.state)?;
                        let transfers = cumulative_transfers(&self.callback.contract, r);
                        observe_conservation(
                            &self.callback.contract,
                            r,
                            time,
                            point.mode,
                            inventories,
                            point.integrals.clone(),
                            transfers,
                        )?;
                        r.samples.push(point);
                    }
                    r.termination = Termination::Event;
                    return Ok(());
                }
                self.transition(r, Some(index), time, &state)?;
                let Some(reset) =
                    self.callback
                        .evaluate(Function::Reset(index), time, &state, false)
                else {
                    return Err(self.callback.failed("event reset"));
                };
                seed = reset.values;
                self.callback.mode = event.next_mode;
            }
            let changed = boundaries.get(segment) == Some(&time);
            if root.is_none() && changed {
                // The next loop captures the old-segment inventory before restart.
                continue;
            }
            if time >= p.end && root.is_none() && !changed {
                self.capture_endpoint(r, time, true, None)?;
                r.termination = Termination::Completed;
                return Ok(());
            }
            self.restart(time, &seed, p, root.is_some())?;
        }
    }
}
impl Session<'_> {
    fn backward_mut(&mut self) -> Result<&mut Backward, ProblemError> {
        self.adjoint
            .as_mut()
            .ok_or_else(|| ProblemError::internal("IDAS adjoint not initialized"))
    }
    /// The backward pass over every segment from the last (ADR-0110 items 3 and 4). Each
    /// segment before the last reruns its forward pass from its stored start with
    /// `IDAReInit` + `IDAAdjReInit` + `IDASolveF`, because reinitialization frees the
    /// checkpoints; the backward problems restart with `IDAReInitB` at the segment's end
    /// and at every observed sample, after the jump of `eliminate` (and its tangent on
    /// the second-order route), with `IDACalcICB`/`IDACalcICBS` recomputing the algebraic
    /// adjoint. The quadratures accumulate the gradient (and the Hessian columns); the
    /// first segment adds its initial values' partials, and the differential adjoint
    /// carries across scheduled changes. Returns the gradient, and on the second-order
    /// route the symmetric Hessian over the directions with its largest relative
    /// asymmetry before symmetrization.
    fn backward(
        &mut self,
        p: &Profile,
        r: &Report,
        weights: &[f64],
    ) -> Result<(Vec<f64>, Option<SecondOrder>), ProblemError> {
        let n = self.callback.contract.states.len();
        let np = self.callback.contract.parameters.len();
        let width = self.callback.integration.len();
        let boundaries = p.boundaries();
        let b = self.backward_mut()?;
        let starts = std::mem::take(&mut b.starts);
        let directions = b.directions.clone();
        let (count, size, quadratures) = if b.second_order() {
            (directions.len(), 2 * n, 2 * width)
        } else {
            (1, n, width)
        };
        let mut carried: Option<Vec<Vec<f64>>> = None;
        let mut totals = vec![vec![0.0; quadratures]; count];
        for (k, start) in starts.iter().enumerate().rev() {
            let stop = boundaries.get(k).copied().unwrap_or(p.end);
            self.callback.map = p.columns_at(np, start.time);
            self.callback.parameters = p.parameters_at(&self.callback.integration, start.time);
            let last = k + 1 == starts.len();
            if !last {
                self.replay(start, stop, p)?;
            }
            // The segment's samples, latest first; a sample at a change time observes the
            // later segment.
            let mut observed = (0..r.samples.len())
                .rev()
                .filter(|i| {
                    let t = r.samples[*i].time;
                    t >= start.time && (t < stop || (last && t <= stop))
                })
                .peekable();
            let mut states = carried
                .take()
                .unwrap_or_else(|| vec![vec![0.0; size]; count]);
            let mut sums = vec![vec![0.0; quadratures]; count];
            if let Some(i) = observed.next_if(|i| r.samples[*i].time >= stop) {
                self.jump(r, i, weights, &mut states, &mut sums)?;
            }
            // The forward pass ended at the segment's stop.
            let forward = self.forward_at_stop()?;
            self.restart_backward(stop, start.time, (&states, &sums), &forward, p)?;
            let mut first = None;
            for i in observed {
                let t = r.samples[i].time;
                if t <= start.time {
                    first = Some(i);
                    break;
                }
                (states, sums) = self.solve_backward(t)?;
                self.jump(r, i, weights, &mut states, &mut sums)?;
                let forward = self.forward_at(t, &r.samples[i])?;
                self.restart_backward(t, start.time, (&states, &sums), &forward, p)?;
            }
            (states, sums) = self.solve_backward(start.time)?;
            if let Some(i) = first {
                self.jump(r, i, weights, &mut states, &mut sums)?;
            }
            if k == 0 {
                self.initial_values(start.time, &states, &mut sums)?;
            }
            for (total, sum) in totals.iter_mut().zip(&sums) {
                for (t, v) in total.iter_mut().zip(sum) {
                    *t += v;
                }
            }
            carried = Some(states);
            let execution = &self.callback.callback.execution;
            execution.progress.push(crate::solve::Event {
                phase: "idas.adjoint".into(),
                elapsed: execution.started.elapsed(),
                values: std::collections::BTreeMap::from([
                    ("time".into(), crate::solve::Metric::Real(start.time)),
                    ("segment".into(), crate::solve::Metric::Integer(k as i64)),
                ]),
                incumbent: None,
            });
        }
        self.collect_counters()?;
        if totals.iter().flatten().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "nonfinite adjoint gradient or Hessian",
            ));
        }
        let gradient = totals[0][..width].to_vec();
        if directions.is_empty() {
            return Ok((gradient, None));
        }
        // Column `b` of the Hessian is problem `b`'s tangent quadrature; the square block
        // over the directions is symmetric up to integration error.
        let d = directions.len();
        let entry = |row: usize, column: usize| totals[column][width + directions[row]];
        let scale = (0..d)
            .flat_map(|a| (0..d).map(move |b| (a, b)))
            .map(|(a, b)| entry(a, b).abs())
            .fold(0.0_f64, f64::max);
        let asymmetry = (0..d)
            .flat_map(|a| (0..d).map(move |b| (a, b)))
            .map(|(a, b)| (entry(a, b) - entry(b, a)).abs())
            .fold(0.0_f64, f64::max)
            / (1.0 + scale);
        let hessian = faer::Mat::from_fn(d, d, |a, b| 0.5 * (entry(a, b) + entry(b, a)));
        Ok((gradient, Some((hessian, asymmetry))))
    }
    /// Rerun one segment's forward pass from its stored consistent start, with its state
    /// sensitivities when they are integrated, storing its checkpoints again.
    fn replay(&mut self, start: &Start, stop: f64, p: &Profile) -> Result<(), ProblemError> {
        let steps: c_long = p
            .max_steps
            .try_into()
            .map_err(|_| ProblemError::unsupported("IDAS step allowance extent"))?;
        // SAFETY: the session's state vector holds the `n` states the start was read from.
        unsafe { write(self.y, &start.y) };
        // SAFETY: as above, for the rates.
        unsafe { write(self.dy, &start.dy) };
        native!(
            ffi::IDAReInit(self.mem, start.time, self.y, self.dy),
            "adjoint forward reinitialization",
        )?;
        if self.forward {
            for (v, values) in self.sens.iter().zip(&start.sensitivities) {
                // SAFETY: as above, for each state sensitivity.
                unsafe { write(*v, values) };
            }
            for (v, values) in self.dsens.iter().zip(&start.rates) {
                // SAFETY: as above, for each sensitivity rate.
                unsafe { write(*v, values) };
            }
            native!(
                ffi::IDASensReInit(
                    self.mem,
                    corrector(p.idas.sensitivity),
                    self.sens.as_mut_ptr(),
                    self.dsens.as_mut_ptr(),
                ),
                "adjoint forward sensitivity reinitialization",
            )?;
        }
        native!(ffi::IDAAdjReInit(self.mem), "adjoint reinitialization")?;
        native!(ffi::IDASetStopTime(self.mem, stop), "stop time")?;
        native!(
            ffi::IDASetMaxNumSteps(self.mem, steps),
            "adjoint forward steps",
        )?;
        let mut time = start.time;
        let flag = self.solve(stop, &mut time)?;
        if flag < 0 {
            return Err(self.callback.native_failure(flag, "adjoint forward pass"));
        }
        Ok(())
    }
    /// The forward state, rates and state sensitivities where the forward pass stopped.
    fn forward_at_stop(&mut self) -> Result<Forward, ProblemError> {
        let n = self.callback.contract.states.len();
        let sensitivities = if self.adjoint.as_ref().is_some_and(Backward::second_order) {
            let mut time = 0.0;
            native!(
                ffi::IDAGetSens(self.mem, &raw mut time, self.sens.as_mut_ptr()),
                "adjoint forward sensitivities",
            )?;
            // SAFETY: each sensitivity vector of the session holds the `n` states.
            self.sens.iter().map(|v| unsafe { read(*v, n) }).collect()
        } else {
            Vec::new()
        };
        Ok((
            // SAFETY: the session's state and rate vectors hold the `n` states.
            unsafe { read(self.y, n) },
            // SAFETY: as above.
            unsafe { read(self.dy, n) },
            sensitivities,
        ))
    }
    /// The forward state and rates at sample time `t`, interpolated from the checkpoints,
    /// and the sample's own state sensitivities on the second-order route.
    fn forward_at(&mut self, t: f64, sample: &Sample) -> Result<Forward, ProblemError> {
        let n = self.callback.contract.states.len();
        let width = self.callback.integration.len();
        let b = self.backward_mut()?;
        let (forward, rate) = (b.forward, b.forward_rate);
        let sensitivities = if b.second_order() {
            if sample.state_sensitivities.len() != n * width {
                return Err(ProblemError::internal("adjoint sample sensitivity extent"));
            }
            (0..width)
                .map(|k| {
                    (0..n)
                        .map(|i| sample.state_sensitivities[i * width + k])
                        .collect()
                })
                .collect()
        } else {
            Vec::new()
        };
        native!(
            ffi::IDAGetAdjY(self.mem, t, forward, rate),
            "adjoint forward interpolation",
        )?;
        // SAFETY: the adjoint's forward state and rate vectors hold the `n` states.
        let forward = unsafe { read(forward, n) };
        // SAFETY: as above.
        let rate = unsafe { read(rate, n) };
        Ok((forward, rate, sensitivities))
    }
    /// Start every backward problem at `t` with its adjoint values and quadratures:
    /// `IDAInitB`/`IDAInitBS` and their settings once, `IDAReInitB` afterwards, then
    /// `IDACalcICB`/`IDACalcICBS` for the algebraic adjoint and every rate.
    fn restart_backward(
        &mut self,
        t: f64,
        start: f64,
        (states, sums): (&[Vec<f64>], &[Vec<f64>]),
        forward: &Forward,
        p: &Profile,
    ) -> Result<(), ProblemError> {
        let mem = self.mem;
        if self.backward_mut()?.problems.is_empty() {
            self.create_backward(t, p)?;
        }
        let b = self.backward_mut()?;
        let (fy, fyp) = (b.forward, b.forward_rate);
        // SAFETY: the adjoint's forward state vector holds the `n` states `forward` was
        // read with.
        unsafe { write(fy, &forward.0) };
        // SAFETY: as above, for the rates.
        unsafe { write(fyp, &forward.1) };
        for (v, values) in b.forward_sensitivities.iter().zip(&forward.2) {
            // SAFETY: as above, for each state sensitivity.
            unsafe { write(*v, values) };
        }
        for (index, problem) in b.problems.iter_mut().enumerate() {
            let values = states
                .get(index)
                .zip(sums.get(index))
                .ok_or_else(|| ProblemError::internal("IDAS backward problem extent"))?;
            // A problem's counters restart with it; keep the finished interval's.
            // SAFETY: `mem` is the session's live IDAS memory, which owns `problem`.
            unsafe { accumulate(mem, problem)? };
            // SAFETY: the problem's vectors hold its adjoint values and quadratures, the
            // extents the backward pass keeps `values` at.
            unsafe { write(problem.y, values.0) };
            // SAFETY: as above.
            unsafe { write(problem.yp, &vec![0.0; values.0.len()]) };
            // SAFETY: as above.
            unsafe { write(problem.quadrature, values.1) };
            native!(
                ffi::IDAReInitB(mem, problem.which, t, problem.y, problem.yp),
                "adjoint reinitialization",
            )?;
            native!(
                ffi::IDAQuadReInitB(mem, problem.which, problem.quadrature),
                "adjoint quadrature reinitialization",
            )?;
        }
        // `tout1` only orients and scales the initialization step toward the start.
        let toward = t - (t - start).min(p.initial_step);
        let second = b.second_order();
        let problems = b.problems.iter().map(|q| q.which).collect::<Vec<_>>();
        let (mut sensitivities, mut rates) = (
            b.forward_sensitivities.clone(),
            b.forward_sensitivity_rates.clone(),
        );
        for which in problems {
            let flag = if second {
                // SAFETY: the session's live IDAS memory and adjoint vectors on its owning
                // thread; the sensitivity arrays are local copies of the vector handles.
                unsafe {
                    ffi::IDACalcICBS(
                        mem,
                        which,
                        toward,
                        fy,
                        fyp,
                        sensitivities.as_mut_ptr(),
                        rates.as_mut_ptr(),
                    )
                }
            } else {
                // SAFETY: the session's live IDAS memory and adjoint vectors.
                unsafe { ffi::IDACalcICB(mem, which, toward, fy, fyp) }
            };
            if flag < 0 {
                return Err(self
                    .callback
                    .native_failure(flag, "consistent adjoint initial conditions"));
            }
        }
        Ok(())
    }
    /// Every backward problem with its KLU solver and quadrature: one first-order problem
    /// over the transposed pattern, or one 2n problem per second-order direction over the
    /// block lower-triangular pattern, whose callbacks read the forward sensitivities.
    fn create_backward(&mut self, t: f64, p: &Profile) -> Result<(), ProblemError> {
        let n = self.callback.contract.states.len();
        let width = self.callback.integration.len();
        let context: *mut c_void = (&raw mut *self.callback).cast();
        let directions = self.backward_mut()?.directions.clone();
        let second = !directions.is_empty();
        let (size, quadratures, nonzeros) = if second {
            (2 * n, 2 * width, self.callback.rows_bs.len())
        } else {
            (n, width, self.callback.rows_b.len())
        };
        let ids: Vec<_> = self
            .callback
            .contract
            .differential
            .iter()
            .cycle()
            .take(size)
            .map(|v| f64::from(*v))
            .collect();
        let id = self.vector(&ids)?;
        let atol = self.vector(
            &p.atol
                .iter()
                .copied()
                .cycle()
                .take(size)
                .collect::<Vec<_>>(),
        )?;
        let smallest = p.atol.iter().copied().fold(f64::INFINITY, f64::min);
        let steps: c_long = p
            .max_steps
            .try_into()
            .map_err(|_| ProblemError::unsupported("IDAS step allowance extent"))?;
        let targets: Vec<Option<usize>> = if second {
            directions.iter().copied().map(Some).collect()
        } else {
            vec![None]
        };
        let (mem, ctx) = (self.mem, self.ctx);
        for target in targets {
            let y = self.vector(&vec![0.0; size])?;
            let yp = self.vector(&vec![0.0; size])?;
            let quadrature = self.vector(&vec![0.0; quadratures])?;
            let mut problem = Problem {
                which: 0,
                matrix: std::ptr::null_mut(),
                linear: std::ptr::null_mut(),
                y,
                yp,
                quadrature,
                direction: target.map(|column| Box::new(Direction { context, column })),
                counters: [0; 4],
            };
            let data: *mut c_void = problem
                .direction
                .as_mut()
                .map_or(context, |d| (&raw mut **d).cast());
            // The problem joins the session before any fallible native call, so `Drop`
            // frees whatever it allocated.
            let b = self.backward_mut()?;
            b.problems.push(problem);
            let problem = b
                .problems
                .last_mut()
                .ok_or_else(|| ProblemError::internal("IDAS backward problem"))?;
            native!(
                ffi::IDACreateB(mem, &raw mut problem.which),
                "backward problem",
            )?;
            let which = problem.which;
            if second {
                native!(
                    ffi::IDAInitBS(mem, which, Some(residual_bs), t, problem.y, problem.yp),
                    "second-order backward initialization",
                )?;
            } else {
                native!(
                    ffi::IDAInitB(mem, which, Some(residual_b), t, problem.y, problem.yp),
                    "backward initialization",
                )?;
            }
            // The user data is the session's boxed context, or the problem's boxed
            // direction: both keep their addresses while the session lives.
            native!(ffi::IDASetUserDataB(mem, which, data), "backward user data")?;
            native!(
                ffi::IDASetIdB(mem, which, id),
                "backward differential identities",
            )?;
            native!(
                ffi::IDASVtolerancesB(mem, which, p.rtol, atol),
                "backward tolerances",
            )?;
            native!(ffi::IDASetMaxNumStepsB(mem, which, steps), "backward steps")?;
            // SAFETY: the session's live context; `Drop` frees the matrix.
            problem.matrix = unsafe {
                ffi::SUNSparseMatrix(index(size)?, index(size)?, index(nonzeros)?, 0, ctx)
            };
            if problem.matrix.is_null() {
                return Err(ProblemError::memory("IDAS adjoint matrix allocation"));
            }
            // SAFETY: the problem's live adjoint vector and matrix and the session's
            // context; `Drop` frees the solver.
            problem.linear = unsafe { ffi::SUNLinSol_KLU(problem.y, problem.matrix, ctx) };
            if problem.linear.is_null() {
                return Err(ProblemError::memory("IDAS adjoint KLU allocation"));
            }
            native!(
                ffi::IDASetLinearSolverB(mem, which, problem.linear, problem.matrix),
                "backward KLU",
            )?;
            if second {
                // A restart at a sample takes the forward sensitivities from the sample,
                // while the steps interpolate IDAS's recomputed checkpoint data: the
                // algebraic adjoint tangent is consistent with the former and off from
                // the latter by interpolation error, which no step size removes. The
                // index-1 algebraic components follow the differential ones, so they
                // leave the local error test (`IDASetSuppressAlgB`).
                native!(
                    ffi::IDASetSuppressAlgB(mem, which, 1),
                    "second-order algebraic error test",
                )?;
                native!(
                    ffi::IDASetJacFnBS(mem, which, Some(jacobian_bs)),
                    "second-order backward Jacobian",
                )?;
                native!(
                    ffi::IDAQuadInitBS(mem, which, Some(quadrature_bs), problem.quadrature),
                    "second-order backward quadrature",
                )?;
            } else {
                native!(
                    ffi::IDASetJacFnB(mem, which, Some(jacobian_b)),
                    "backward analytic Jacobian",
                )?;
                native!(
                    ffi::IDAQuadInitB(mem, which, Some(quadrature_b), problem.quadrature),
                    "backward quadrature",
                )?;
            }
            native!(
                ffi::IDAQuadSStolerancesB(mem, which, p.rtol, smallest),
                "backward quadrature tolerances",
            )?;
            native!(
                ffi::IDASetQuadErrConB(mem, which, 1),
                "backward quadrature error control",
            )?;
        }
        Ok(())
    }
    /// Integrate every backward problem to `t`; their adjoint values and quadratures there.
    fn solve_backward(&mut self, t: f64) -> Result<Adjoints, ProblemError> {
        let mem = self.mem;
        // SAFETY: the session's live IDAS memory on its owning thread; the backward
        // callbacks reach the context only through their user data.
        let flag = unsafe { ffi::IDASolveB(mem, t, ffi::IDA_NORMAL) };
        if flag < 0 {
            return Err(self.callback.native_failure(flag, "adjoint step"));
        }
        let n = self.callback.contract.states.len();
        let width = self.callback.integration.len();
        let b = self.backward_mut()?;
        let (size, quadratures) = if b.second_order() {
            (2 * n, 2 * width)
        } else {
            (n, width)
        };
        let mut states = Vec::with_capacity(b.problems.len());
        let mut sums = Vec::with_capacity(b.problems.len());
        for problem in &b.problems {
            let mut time = t;
            native!(
                ffi::IDAGetB(mem, problem.which, &raw mut time, problem.y, problem.yp),
                "adjoint state",
            )?;
            native!(
                ffi::IDAGetQuadB(mem, problem.which, &raw mut time, problem.quadrature),
                "adjoint quadrature",
            )?;
            // SAFETY: the problem's adjoint vector holds `size` values (`create_backward`).
            states.push(unsafe { read(problem.y, size) });
            // SAFETY: its quadrature vector holds `quadratures` values.
            sums.push(unsafe { read(problem.quadrature, quadratures) });
        }
        Ok((states, sums))
    }
    /// Add the jump at sample `i` to every backward problem: the first-order jump to the
    /// adjoint and the gradient, in the segment's integration columns, and on the
    /// second-order route its tangent along the problem's direction to `μ` and the Hessian
    /// column: with `d = [s_k; π_k]` and `ŵ` the first-order constraint multipliers,
    /// `v = ∇²(cᵀg)·d − ∇²(ŵᵀf)·d` is eliminated through the same algebraic block. The
    /// forward state and sensitivities are the sample's own.
    fn jump(
        &mut self,
        r: &Report,
        i: usize,
        weights: &[f64],
        states: &mut [Vec<f64>],
        sums: &mut [Vec<f64>],
    ) -> Result<(), ProblemError> {
        let sample = &r.samples[i];
        let n = self.callback.contract.states.len();
        let m = self.callback.contract.outputs.len();
        let width = self.callback.integration.len();
        let cotangent = &weights[i * m..(i + 1) * m];
        let Some(output) = self
            .callback
            .evaluate(Function::Output, sample.time, &sample.state, true)
            .and_then(|e| e.jacobian)
        else {
            return Err(self.callback.failed("adjoint output partials"));
        };
        let rhs = if self.callback.contract.differential.contains(&false) {
            let Some(j) = self.callback.partials(sample.time, &sample.state) else {
                return Err(self.callback.failed("adjoint constraint partials"));
            };
            Some(j)
        } else {
            None
        };
        let differential = self.callback.contract.differential.clone();
        let first = eliminate(
            &differential,
            rhs.as_ref().map(|j| j.as_ref()),
            transposed_product(output.as_ref(), cotangent),
        )?;
        let directions = self.backward_mut()?.directions.clone();
        for (index, (state, sum)) in states.iter_mut().zip(sums.iter_mut()).enumerate() {
            for (a, d) in state.iter_mut().zip(&first.state) {
                *a += d;
            }
            for (q, d) in first.parameters.iter().enumerate() {
                *sum.get_mut(self.callback.map[q])
                    .ok_or_else(|| ProblemError::internal("adjoint jump columns"))? += d;
            }
            let Some(&column) = directions.get(index) else {
                continue;
            };
            if sample.state_sensitivities.len() != n * width {
                return Err(ProblemError::internal("adjoint sample sensitivity extent"));
            }
            let s = (0..n)
                .map(|x| sample.state_sensitivities[x * width + column])
                .collect::<Vec<_>>();
            let d = self.callback.direction(column, &s);
            let Some(mut v) = self.callback.curvature(
                Function::Output,
                sample.time,
                &sample.state,
                cotangent,
                &d,
            ) else {
                return Err(self.callback.failed("adjoint output curvature"));
            };
            if rhs.is_some() {
                let Some(constraint) = self.callback.curvature(
                    Function::Rhs,
                    sample.time,
                    &sample.state,
                    &first.multipliers,
                    &d,
                ) else {
                    return Err(self.callback.failed("adjoint constraint curvature"));
                };
                for (total, c) in v.iter_mut().zip(constraint) {
                    *total -= c;
                }
            }
            let tangent = eliminate(&differential, rhs.as_ref().map(|j| j.as_ref()), v)?;
            for (a, d) in state[n..].iter_mut().zip(&tangent.state) {
                *a += d;
            }
            for (q, d) in tangent.parameters.iter().enumerate() {
                *sum.get_mut(width + self.callback.map[q])
                    .ok_or_else(|| ProblemError::internal("adjoint jump columns"))? += d;
            }
        }
        Ok(())
    }
    /// The first segment's initial differential values depend on the parameters:
    /// `(∂x₀/∂p)ᵀλ_d` joins the gradient, and on the second-order route
    /// `(∂x₀/∂p)ᵀμ_d + ∇²_pp(λ_dᵀx₀)·π_k` joins the Hessian column (the initial values do
    /// not depend on the state). `IDACalcIC` recomputes the algebraic ones.
    fn initial_values(
        &mut self,
        t: f64,
        states: &[Vec<f64>],
        sums: &mut [Vec<f64>],
    ) -> Result<(), ProblemError> {
        let n = self.callback.contract.states.len();
        let width = self.callback.integration.len();
        let zeros = vec![0.0; n];
        let Some(j) = self
            .callback
            .evaluate(Function::Initial, t, &zeros, true)
            .and_then(|e| e.jacobian)
        else {
            return Err(self.callback.failed("initial partials"));
        };
        let directions = self.backward_mut()?.directions.clone();
        let mask = self.callback.contract.differential.clone();
        let differential = |values: &[f64]| -> Vec<f64> {
            values
                .iter()
                .zip(&mask)
                .map(|(l, d)| if *d { *l } else { 0.0 })
                .collect()
        };
        for (index, (state, sum)) in states.iter().zip(sums.iter_mut()).enumerate() {
            let lambda = differential(&state[..n]);
            let product = transposed(&j, &lambda);
            for (q, column) in self.callback.map.iter().enumerate() {
                sum[*column] += product[n + q];
            }
            let Some(&column) = directions.get(index) else {
                continue;
            };
            let mu = differential(&state[n..]);
            let product = transposed(&j, &mu);
            let d = self.callback.direction(column, &zeros);
            let Some(v) = self
                .callback
                .curvature(Function::Initial, t, &zeros, &lambda, &d)
            else {
                return Err(self.callback.failed("initial curvature"));
            };
            for (q, target) in self.callback.map.iter().enumerate() {
                sum[width + *target] += product[n + q] + v[n + q];
            }
        }
        Ok(())
    }
    /// Add every backward problem's native counters since its last reinitialization.
    fn collect_counters(&mut self) -> Result<(), ProblemError> {
        let mem = self.mem;
        for problem in &mut self.backward_mut()?.problems {
            // SAFETY: `mem` is the session's live IDAS memory, which owns `problem`.
            unsafe {
                accumulate(mem, problem)?;
            }
        }
        Ok(())
    }
    /// The backward problems' summed native counters, for the statistics.
    fn backward_statistics(&self) -> serde_json::Value {
        let problems = self.adjoint.as_ref().map_or_else(Vec::new, |b| {
            b.problems
                .iter()
                .map(|q| {
                    serde_json::json!({
                        "direction": q.direction.as_ref().map(|d| d.column),
                        "steps": q.counters[0],
                        "nonlinear_iterations": q.counters[1],
                        "nonlinear_convergence_failures": q.counters[2],
                        "error_test_failures": q.counters[3],
                    })
                })
                .collect()
        });
        serde_json::Value::Array(problems)
    }
}
/// Add a backward problem's native counters since its last reinitialization, which
/// resets them.
///
/// # Safety
/// `mem` is the live IDAS memory that owns `problem`.
unsafe fn accumulate(mem: *mut c_void, problem: &mut Problem) -> Result<(), ProblemError> {
    // SAFETY: `mem` is the live IDAS memory that owns `problem` (the caller's contract).
    let backward = unsafe { ffi::IDAGetAdjIDABmem(mem, problem.which) };
    if backward.is_null() {
        return Err(ProblemError::internal("IDAS backward memory"));
    }
    // The backward memory is owned by `mem`; the counters are written into `values`.
    let mut values: [c_long; 4] = [0; 4];
    native!(
        ffi::IDAGetNumSteps(backward, &raw mut values[0]),
        "backward steps",
    )?;
    native!(
        ffi::IDAGetNumNonlinSolvIters(backward, &raw mut values[1]),
        "backward nonlinear iterations",
    )?;
    native!(
        ffi::IDAGetNumNonlinSolvConvFails(backward, &raw mut values[2]),
        "backward convergence failures",
    )?;
    native!(
        ffi::IDAGetNumErrTestFails(backward, &raw mut values[3]),
        "backward error-test failures",
    )?;
    for (total, value) in problem.counters.iter_mut().zip(values) {
        *total += long_counter(value);
    }
    Ok(())
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
    let (report, _, _) = attempt(oracle, p, integration, &[], cancel, progress, |_, _| Ok(()));
    Ok(report)
}
/// One IDACalcIC call, with no physical step, sensitivity, quadrature or event setup.
pub(super) fn initialize_consistent(
    oracle: &mut dyn Oracle,
    parameters: &[f64],
    request: &ConsistentInitialization,
    execution: Execution,
) -> Result<ConsistentStateReport, ProblemError> {
    oracle.contract().validate()?;
    request.controls.validate_for(
        request.mode,
        oracle
            .contract()
            .signs
            .iter()
            .any(|sign| *sign != StateSign::Free),
    )?;
    let n = oracle.contract().states.len();
    if !request.time.is_finite()
        || !request.toward.is_finite()
        || request.time == request.toward
        || !(request.toward - request.time).is_finite()
        || !positive(request.rtol)
        || request.atol.len() != n
        || request.residual_tolerances.len() != n
        || request
            .atol
            .iter()
            .chain(&request.residual_tolerances)
            .any(|x| !positive(*x))
        || parameters.len() != oracle.contract().parameters.len()
        || parameters.iter().any(|x| !x.is_finite())
    {
        return Err(contract(
            "invalid consistent-state request or original parameter extent",
        ));
    }
    let dimension = match request.linear {
        IdasLinear::Klu => None,
        IdasLinear::Spgmr { dimension, .. } | IdasLinear::Spfgmr { dimension, .. } => {
            Some(dimension.into_inner())
        }
    };
    if dimension.is_some_and(|value| i32::try_from(value).is_err()) {
        return Err(contract("invalid IDAS initialization Krylov dimension"));
    }
    execution.check()?;
    let allowance = execution.memory.ok_or_else(|| {
        contract("consistent initialization requires a finite foreign memory allowance")
    })?;
    // Native KLU fill can be dense. Bound the planned shape before allocating foreign
    // vectors/matrices; deployment owns enforcement of opaque native allocations.
    let vectors = dimension
        .unwrap_or(0)
        .checked_mul(2)
        .and_then(|v| v.checked_add(12));
    let bytes = n
        .checked_mul(n)
        .and_then(|v| v.checked_mul(32))
        .and_then(|v| {
            n.checked_mul(vectors?)
                .and_then(|w| w.checked_mul(8))
                .and_then(|w| v.checked_add(w))
        })
        .ok_or_else(|| {
            ProblemError::memory("consistent initialization workspace shape overflow")
        })?;
    if bytes > allowance {
        return Err(ProblemError::memory(
            "consistent initialization workspace exceeds foreign allowance",
        ));
    }
    let final_execution = execution.clone();
    let (mut session, initial) = Session::prepare(
        oracle,
        parameters,
        StatePreparation {
            time: request.time,
            parameters: parameters.to_vec(),
            map: (0..parameters.len()).collect(),
            trial_failures: request.trial_failures,
            rtol: request.rtol,
            atol: &request.atol,
            initial_step: None,
            linear: request.linear,
            forward: false,
        },
        execution,
    )?;
    let flag = session.calculate_consistent(request.toward, request.mode, &request.controls)?;
    let initial_residual = session.callback.initial_ic_residual;
    if flag == ffi::IDA_FIRST_RES_FAIL
        && initial_residual == Some(false)
        && session.callback.callback.terminal.is_none()
    {
        // Tagged IDAS cannot repair a failed first residual. This is current-start
        // evaluation evidence, not a globally established model contract failure.
        session.callback.callback.trial_rejections =
            session.callback.callback.trial_rejections.saturating_sub(1);
        session.callback.callback.terminal = Some((
            crate::solve::Termination::Evaluation,
            "invalid initial original residual".into(),
        ));
    }
    let mut native = termination(flag);
    let mut error = if flag < 0 {
        Some(
            session
                .callback
                .native_failure(flag, "consistent initial conditions"),
        )
    } else {
        None
    };
    let callback = &mut session.callback.callback;
    if callback.terminal.is_none()
        && let Some(stop) = callback.execution.stopped()
    {
        callback.last_failure = callback.execution.check().err();
        callback.terminal = Some((
            stop,
            "execution checkpoint after consistent initialization".into(),
        ));
    }
    // SAFETY: only a query/copy of live vectors before any physical step. The pinned
    // API allows retrieval after failed IDACalcIC, so these remain diagnostic candidates.
    let retrieval = unsafe { ffi::IDAGetConsistentIC(session.mem, session.y, session.dy) };
    let candidate = if retrieval == ffi::IDA_SUCCESS {
        // SAFETY: both serial vectors have the authored state extent.
        let state = unsafe { read(session.y, n) };
        // SAFETY: the owned rate serial vector has the same authored state extent.
        let rates = unsafe { read(session.dy, n) };
        if state.iter().chain(&rates).all(|x| x.is_finite()) {
            Some(ConsistentState { state, rates })
        } else {
            None
        }
    } else {
        if error.is_none() {
            error = Some(
                session
                    .callback
                    .native_failure(retrieval, "consistent state retrieval"),
            );
        }
        None
    };
    let iterations = ic_counter(session.mem, ffi::IDAGetNumNonlinSolvIters);
    let backtracks = ic_counter(session.mem, ffi::IDAGetNumBacktrackOps);
    let mut assessment = None;
    let mut validation_error = None;
    if session.callback.callback.terminal.is_none()
        && let Some(candidate) = &candidate
    {
        if let Some(rhs) =
            session
                .callback
                .evaluate(Function::Rhs, request.time, &candidate.state, false)
        {
            let residual: Vec<_> = rhs
                .values
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    if session.callback.contract.differential[i] {
                        candidate.rates[i] - f
                    } else {
                        -f
                    }
                })
                .collect();
            let roles_preserved = match request.mode {
                IdasInitialization::AlgebraicAndRates => session
                    .callback
                    .contract
                    .differential
                    .iter()
                    .enumerate()
                    .all(|(i, d)| !*d || candidate.state[i] == initial.values[i]),
                IdasInitialization::SteadyStates => candidate.rates.iter().all(|v| *v == 0.0),
            };
            let signs_satisfied = session
                .callback
                .contract
                .signs
                .iter()
                .zip(&candidate.state)
                .all(|(sign, x)| match sign {
                    StateSign::Free => true,
                    StateSign::NonNegative => *x >= 0.0,
                    StateSign::Positive => *x > 0.0,
                    StateSign::NonPositive => *x <= 0.0,
                    StateSign::Negative => *x < 0.0,
                });
            let residual_satisfied = residual
                .iter()
                .zip(&request.residual_tolerances)
                .all(|(v, t)| v.is_finite() && v.abs() <= *t);
            assessment = Some(ConsistentStateAssessment {
                residual,
                residual_satisfied,
                roles_preserved,
                signs_satisfied,
            });
        } else {
            validation_error = Some(
                session
                    .callback
                    .failed("consistent-state original assessment"),
            );
        }
    }
    let callback = &mut session.callback.callback;
    if let Some((category, _)) = &callback.terminal {
        native.category = *category;
        if error.is_none() {
            error = callback.terminal_error();
        }
    }
    let mut evidence = crate::solve::Evidence {
        work: crate::solve::WorkEvidence {
            evaluations: callback.counts.values().try_fold(0_u64, |sum, count| {
                sum.checked_add(u64::try_from(*count).ok()?)
            }),
            iterations,
            ..Default::default()
        },
        abandoned: callback.execution.abandonment.observation(),
        callback: crate::solve::CallbackEvidence {
            trial_rejections: callback.trial_rejections,
            regime_crossings: callback.regime_crossings,
            terminal_failure: callback.terminal.is_some(),
        },
        start_submitted: true,
        ..Default::default()
    };
    let identity = session.callback.contract.identity;
    drop(session); // Native destruction stays inside the original task deadline.
    if !evidence.callback.terminal_failure
        && let Some(stop) = final_execution.stopped()
    {
        native.category = stop;
        evidence.callback.terminal_failure = true;
        evidence.abandoned = final_execution.abandonment.observation();
        if error.is_none() {
            error = final_execution.check().err();
        }
        assessment = None;
    }
    Ok(ConsistentStateReport {
        identity,
        time: request.time,
        mode: request.mode,
        initial_residual,
        termination: native,
        requested: initial.values,
        candidate,
        assessment,
        evidence,
        backtracks,
        error,
        validation_error,
    })
}
fn ic_counter(
    mem: *mut c_void,
    get: unsafe extern "C" fn(*mut c_void, *mut c_long) -> i32,
) -> Option<u64> {
    let mut count = 0;
    // SAFETY: counter query on the caller's live IDAS memory, into a correctly typed local.
    if unsafe { get(mem, &raw mut count) } == ffi::IDA_SUCCESS {
        u64::try_from(count).ok()
    } else {
        None
    }
}
/// The gradient of the cotangent's functional over the integration parameters, on the
/// IDAS adjoint (ADR-0110 item 3). The caller validated the profile and charged the
/// checkpoint estimate.
pub(super) fn gradient(
    oracle: &mut dyn Oracle,
    p: &Profile,
    integration: &[f64],
    cotangent: Cotangent<'_>,
    cancel: Cancellation,
    progress: Arc<Progress>,
) -> Result<Gradient, ProblemError> {
    backward_route(oracle, p, integration, &[], cotangent, cancel, progress)
}
/// The gradient and the Hessian over `directions` of the cotangent's functional, by
/// forward-over-adjoint second-order sensitivities (ADR-0110 item 4). The caller admitted
/// the second-order route and charged its estimate.
pub(super) fn hessian(
    oracle: &mut dyn Oracle,
    p: &Profile,
    integration: &[f64],
    directions: &[usize],
    cotangent: Cotangent<'_>,
    cancel: Cancellation,
    progress: Arc<Progress>,
) -> Result<Gradient, ProblemError> {
    backward_route(
        oracle,
        p,
        integration,
        directions,
        cotangent,
        cancel,
        progress,
    )
}
/// One forward pass with checkpoints, the cotangent of its samples, and the backward pass
/// of the first-order route or, with `directions`, the second-order route.
fn backward_route(
    oracle: &mut dyn Oracle,
    p: &Profile,
    integration: &[f64],
    directions: &[usize],
    cotangent: Cotangent<'_>,
    cancel: Cancellation,
    progress: Arc<Progress>,
) -> Result<Gradient, ProblemError> {
    let (mut report, outcome, checkpoints) = attempt(
        oracle,
        p,
        integration,
        directions,
        cancel,
        progress,
        |s, r| {
            let weights = cotangent(r)?;
            if weights.len()
                != r.samples
                    .len()
                    .saturating_mul(s.callback.contract.outputs.len())
                || weights.iter().any(|w| !w.is_finite())
            {
                return Err(contract("adjoint cotangent extent or value"));
            }
            let result = s.backward(p, r, &weights)?;
            Ok((result, s.backward_statistics()))
        },
    );
    let completed = report.termination == Termination::Completed;
    let (gradient, hessian) = match outcome {
        Some(((gradient, second), backward)) => {
            report.statistics.push(serde_json::json!({
                "adjoint": {
                    "checkpoints": checkpoints,
                    "steps_between_checkpoints": p.adjoint.steps_between_checkpoints.into_inner(),
                    "segments": p.boundaries().len() + 1,
                    "second_order_directions": directions,
                    "hessian_asymmetry": second.as_ref().map(|(_, a)| *a),
                    "backward": backward,
                }
            }));
            (Some(gradient), second.map(|(h, _)| h))
        }
        None => (None, None),
    };
    Ok(Gradient {
        gradient: gradient.filter(|_| completed),
        hessian: hessian.filter(|_| completed),
        report,
        checkpoints,
        reserved_bytes: 0,
    })
}
/// One IDAS attempt: the forward pass, then `after` once it completed, with every failure
/// and stop kept in the report.
fn attempt<T>(
    oracle: &mut dyn Oracle,
    p: &Profile,
    integration: &[f64],
    directions: &[usize],
    cancel: Cancellation,
    progress: Arc<Progress>,
    after: impl FnOnce(&mut Session<'_>, &Report) -> Result<T, ProblemError>,
) -> (Report, Option<T>, usize) {
    let execution = Execution {
        cancel,
        started: Instant::now(),
        time_limit: p.time_limit,
        progress,
        memory: None,
        enclosing_scope: None,
        abandonment: Arc::default(),
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
        Err(error) => return (failure(r, error), None, 0),
    };
    r.requested_initial = initial.values;
    let mut s = match Session::new(oracle, integration, p, directions, execution.clone()) {
        Ok(session) => session,
        Err(error) => return (failure(r, error), None, 0),
    };
    // SAFETY: the new session's state vector holds the `n` states.
    r.consistent_initial = unsafe { read(s.y, n) };
    let mut outcome = None;
    let result = s.run(p, &mut r).and_then(|()| {
        if r.termination == Termination::Completed {
            outcome = Some(after(&mut s, &r)?);
        }
        Ok(())
    });
    if let Err(e) = result {
        r.termination = stopped(&e, &s.callback.callback.execution);
        r.error = Some(e);
        outcome = None;
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
        outcome = None;
    }
    (r.progress, r.dropped_progress) = s.callback.callback.execution.progress.snapshot();
    let checkpoints = s.adjoint.as_ref().map_or(0, |b| b.checkpoints);
    (r, outcome, checkpoints)
}

#[allow(
    clippy::useless_conversion,
    reason = "C long width differs between native ABIs"
)]
fn long_counter(value: c_long) -> i64 {
    i64::from(value)
}

#[cfg(test)]
mod initialization_tests {
    use super::*;
    #[derive(Debug)]
    struct IcOracle {
        contract: Contract,
        residual_calls: usize,
        fail: bool,
        nonlinear: bool,
    }
    impl IcOracle {
        fn new() -> Self {
            let id = |n: u8| SemanticId::from_bytes([n; 16]);
            Self {
                contract: Contract {
                    identity: ContentHash::from_bytes([91; 32]),
                    states: vec![id(1), id(2)],
                    differential: vec![true, false],
                    parameters: vec![],
                    outputs: vec![id(3)],
                    events: vec![vec![]],
                    signs: vec![],
                    derivatives: pse_kernels::DerivativeOrder::First,
                    quadratures: vec![],
                    balances: vec![],
                },
                residual_calls: 0,
                fail: false,
                nonlinear: false,
            }
        }
    }
    impl Oracle for IcOracle {
        fn contract(&self) -> &Contract {
            &self.contract
        }
        fn support(&self, _: usize, function: Function) -> Vec<SupportEntry> {
            if function == Function::Rhs {
                [(0, 0), (1, 0), (1, 1)]
                    .map(|(r, c)| SupportEntry::new(OriginalRow::new(r), OriginalCol::new(c)))
                    .to_vec()
            } else {
                vec![]
            }
        }
        fn evaluate(
            &mut self,
            _: usize,
            function: Function,
            _: f64,
            x: &[f64],
            _: &[f64],
            derivatives: bool,
        ) -> Result<Evaluation, ProblemError> {
            if function == Function::Initial {
                return Ok(Evaluation {
                    values: vec![2.0, if self.nonlinear { 1.0 } else { 0.0 }],
                    jacobian: None,
                });
            }
            assert_eq!(
                function,
                Function::Rhs,
                "IC-only call does not evaluate outputs, events or quadratures"
            );
            self.residual_calls += 1;
            if self.fail {
                return Err(pse_math::MathError::Domain {
                    source_id: SemanticId::from_bytes([92; 16]),
                    requirement: "IC original-domain witness",
                }
                .into());
            }
            let rhs = if self.nonlinear {
                x[0] * x[0] - x[1] * x[1]
            } else {
                2.0 * x[0] - x[1]
            };
            let jacobian = if derivatives {
                Some(
                    faer::sparse::SparseColMat::try_new_from_triplets(
                        2,
                        2,
                        &[
                            faer::sparse::Triplet::new(0, 0, -1.0),
                            faer::sparse::Triplet::new(
                                1,
                                0,
                                if self.nonlinear { 2.0 * x[0] } else { 2.0 },
                            ),
                            faer::sparse::Triplet::new(
                                1,
                                1,
                                if self.nonlinear { -2.0 * x[1] } else { -1.0 },
                            ),
                        ],
                    )
                    .unwrap(),
                )
            } else {
                None
            };
            Ok(Evaluation {
                values: vec![-x[0], rhs],
                jacobian,
            })
        }
    }
    fn request() -> ConsistentInitialization {
        ConsistentInitialization {
            time: 0.0,
            toward: 0.1,
            rtol: 1e-9,
            atol: vec![1e-11; 2],
            residual_tolerances: vec![1e-8; 2],
            mode: IdasInitialization::AlgebraicAndRates,
            linear: IdasLinear::Klu,
            controls: Default::default(),
            trial_failures: TrialPolicy::Terminal,
        }
    }
    fn execution() -> Execution {
        let mut e = Execution::new(Arc::default(), &crate::solve::Controls::default());
        e.memory = Some(1 << 20);
        e
    }
    #[test]
    fn public_ic_preserves_original_roles_and_has_no_physical_trajectory() {
        let mut oracle = IcOracle::new();
        let report = initialize_consistent(&mut oracle, &[], &request(), execution()).unwrap();
        assert_eq!(report.termination.name, "IDA_SUCCESS");
        assert!(report.error.is_none());
        let point = report.candidate.unwrap();
        assert_eq!(point.state[0], 2.0);
        assert!((point.state[1] - 4.0).abs() < 1e-8);
        assert!((point.rates[0] + 2.0).abs() < 1e-8);
        let assessment = report.assessment.unwrap();
        assert!(
            assessment.roles_preserved
                && assessment.residual_satisfied
                && assessment.signs_satisfied
        );
        assert!(report.evidence.work.iterations.is_some_and(|n| n > 0));
        assert!(report.evidence.work.evaluations.is_some_and(|n| n > 0));
        assert_eq!(report.evidence.work.factorizations, None);
        assert_eq!(report.evidence.work.proof_steps, None);
        assert!(report.backtracks.is_some());
        let mut all_states = request();
        all_states.mode = IdasInitialization::SteadyStates;
        let report =
            initialize_consistent(&mut IcOracle::new(), &[], &all_states, execution()).unwrap();
        assert_eq!(report.termination.name, "IDA_SUCCESS");
        let point = report.candidate.unwrap();
        assert!(point.state.iter().all(|v| v.abs() < 1e-8));
        assert!(point.rates.iter().all(|v| *v == 0.0));
        assert!(report.assessment.unwrap().roles_preserved);
    }
    #[test]
    fn public_ic_native_cap_keeps_failed_status_counters_and_original_assessment() {
        let mut oracle = IcOracle {
            nonlinear: true,
            ..IcOracle::new()
        };
        let mut request = request();
        request.controls.newton_iterations = 1;
        request.controls.jacobian_attempts = 1;
        request.controls.step_trials = Some(1);
        let report = initialize_consistent(&mut oracle, &[], &request, execution()).unwrap();
        assert_eq!(report.termination.name, "IDA_CONV_FAIL");
        assert!(report.error.is_some());
        assert!(!report.evidence.callback.terminal_failure);
        assert!(
            report
                .evidence
                .work
                .iterations
                .is_some_and(|n| n > 0 && n <= 2)
        );
        assert!(report.assessment.is_some());
        assert!(report.backtracks.is_some());
    }
    #[test]
    fn public_ic_terminal_domain_preserves_typed_cause_and_prevents_fresh_assessment() {
        let mut oracle = IcOracle {
            fail: true,
            ..IcOracle::new()
        };
        let report = initialize_consistent(&mut oracle, &[], &request(), execution()).unwrap();
        assert_eq!(report.termination.name, "IDA_RES_FAIL");
        assert_eq!(
            report.termination.category,
            crate::solve::Termination::Evaluation
        );
        assert!(report.evidence.callback.terminal_failure);
        assert_eq!(oracle.residual_calls, 1);
        assert!(report.assessment.is_none());
        assert!(matches!(
            report.error,
            Some(ProblemError::Math(pse_math::MathError::Domain {
                requirement: "IC original-domain witness",
                ..
            }))
        ));
        assert_eq!(report.evidence.work.iterations, Some(0));
        let mut oracle = IcOracle {
            fail: true,
            ..IcOracle::new()
        };
        let mut r = request();
        r.trial_failures = TrialPolicy::Recoverable;
        let report = initialize_consistent(&mut oracle, &[], &r, execution()).unwrap();
        assert_eq!(report.termination.name, "IDA_FIRST_RES_FAIL");
        assert_eq!(report.initial_residual, Some(false));
        assert!(report.evidence.callback.terminal_failure);
        assert_eq!(
            report.evidence.callback.trial_rejections, 0,
            "failed first residual is not a later Newton trial rejection"
        );
        assert_eq!(oracle.residual_calls, 1);
        assert!(report.assessment.is_none());
        assert!(matches!(
            report.error,
            Some(ProblemError::Math(pse_math::MathError::Domain {
                requirement: "IC original-domain witness",
                ..
            }))
        ));
    }
    #[test]
    fn public_ic_rejects_inert_and_nonfinite_controls_before_callbacks() {
        let mut oracle = IcOracle::new();
        let mut r = request();
        r.mode = IdasInitialization::SteadyStates;
        r.controls.step_trials = Some(3);
        assert!(matches!(
            initialize_consistent(&mut oracle, &[], &r, execution()),
            Err(ProblemError::Contract(_))
        ));
        r = request();
        r.controls.line_search = false;
        r.controls.backtracks = Some(3);
        assert!(matches!(
            initialize_consistent(&mut oracle, &[], &r, execution()),
            Err(ProblemError::Contract(_))
        ));
        r = request();
        r.controls.line_search = false;
        r.controls.step_tolerance = Some(1e-12);
        assert!(matches!(
            initialize_consistent(&mut oracle, &[], &r, execution()),
            Err(ProblemError::Contract(_))
        ));
        r = request();
        r.toward = r.time;
        assert!(matches!(
            initialize_consistent(&mut oracle, &[], &r, execution()),
            Err(ProblemError::Contract(_))
        ));
        let mut e = execution();
        e.memory = Some(1);
        assert!(matches!(
            initialize_consistent(&mut oracle, &[], &request(), e),
            Err(ProblemError::Limit {
                kind: crate::LimitKind::Memory,
                ..
            })
        ));
        assert_eq!(oracle.residual_calls, 0);
        let mut constrained = IcOracle::new();
        constrained.contract.signs = vec![StateSign::NonNegative; 2];
        r = request();
        r.controls.line_search = false;
        r.controls.step_tolerance = Some(1e-12);
        let report = initialize_consistent(&mut constrained, &[], &r, execution()).unwrap();
        assert_eq!(
            report.termination.name, "IDA_SUCCESS",
            "an explicit constraint step floor still acts without line search"
        );
        let e = execution();
        e.cancel.store(true, std::sync::atomic::Ordering::Release);
        assert!(matches!(
            initialize_consistent(&mut oracle, &[], &request(), e),
            Err(ProblemError::Cancelled)
        ));
        assert_eq!(oracle.residual_calls, 0);
    }
}
