// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "owned IDAS/SUNContext resources and panic-contained C callbacks"
)]
//! IDAS owns residual integration, consistent initialization and trial recovery.
use super::*;
use crate::{
    NativeStatus,
    callback::CallbackState,
    solve::{Assurance, Backend, Execution, NativeTermination, Progress},
};
use std::{ffi::c_void, marker::PhantomData, rc::Rc, time::Instant};
use suitesparse_sys as _;
use sundials_sys as ffi;

struct Context<'a> {
    oracle: &'a mut dyn Oracle,
    contract: Contract,
    parameters: Vec<f64>,
    mode: usize,
    trial_policy: TrialPolicy,
    callback: CallbackState,
    columns: Vec<ffi::sunindextype>,
    rows: Vec<ffi::sunindextype>,
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
        let Some(e) = c.evaluate(Function::Rhs, t, &x, true) else {
            return c.failure();
        };
        let Some(j) = e.jacobian else { return -1 };
        let mut values = Vec::with_capacity(c.rows.len());
        for col in 0..n {
            for k in c.columns[col] as usize..c.columns[col + 1] as usize {
                let row = c.rows[k] as usize;
                values.push(
                    -j.get(row, col).copied().unwrap_or(0.0)
                        + if row == col && c.contract.differential[row] {
                            cj
                        } else {
                            0.0
                        },
                );
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
        if np < 0 || np as usize != c.parameters.len() {
            return -1;
        }
        let x = unsafe { read(y, n) };
        let Some(e) = c.evaluate(Function::Rhs, t, &x, true) else {
            return c.failure();
        };
        let Some(j) = e.jacobian else { return -1 };
        for p in 0..np as usize {
            let s = unsafe { read(*ys.add(p), n) };
            let ds = unsafe { read(*yps.add(p), n) };
            let residual: Vec<_> = (0..n)
                .map(|row| {
                    let mut value = if c.contract.differential[row] {
                        ds[row]
                    } else {
                        0.0
                    };
                    value -= j.get(row, n + p).copied().unwrap_or(0.0);
                    for (col, s) in s.iter().enumerate() {
                        value -= j.get(row, col).copied().unwrap_or(0.0) * s;
                    }
                    value
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
        parameters: &[f64],
        p: &Profile,
        execution: Execution,
    ) -> Result<Self, ProblemError> {
        let c = oracle.contract().clone();
        let n = c.states.len();
        let mut support = BTreeSet::new();
        for mode in 0..c.events.len() {
            support.extend(
                oracle
                    .support(mode, Function::Rhs)
                    .into_iter()
                    .filter(|(_, col)| *col < n),
            );
        }
        support.extend((0..n).map(|i| (i, i)));
        let mut columns = vec![0];
        let mut rows = Vec::new();
        for col in 0..n {
            for &(r, _) in support.iter().filter(|(_, c)| *c == col) {
                rows.push(index(r)?);
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
            callback: Box::new(Context {
                oracle,
                contract: c,
                parameters: parameters.to_vec(),
                mode: 0,
                trial_policy: p.trial_failures,
                callback: CallbackState::new(execution),
                columns,
                rows,
            }),
            _local: PhantomData,
        };
        unsafe {
            check(ffi::SUNContext_Create(0, &raw mut s.ctx), "context")?;
        }
        let Some(initial) =
            s.callback
                .evaluate(Function::Initial, p.start, &vec![0.0; n], p.sensitivities)
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
            s.matrix = ffi::SUNSparseMatrix(
                index(n)?,
                index(n)?,
                index(s.callback.rows.len())?,
                0,
                s.ctx,
            );
            if s.matrix.is_null() {
                return Err(ProblemError::memory("IDAS matrix allocation"));
            }
            s.linear = ffi::SUNLinSol_KLU(s.y, s.matrix, s.ctx);
            if s.linear.is_null() {
                return Err(ProblemError::memory("IDAS KLU allocation"));
            }
            check(ffi::IDASetLinearSolver(s.mem, s.linear, s.matrix), "KLU")?;
            check(ffi::IDASetJacFn(s.mem, Some(jacobian)), "analytic Jacobian")?;
        }
        if p.sensitivities {
            let j = initial
                .jacobian
                .ok_or_else(|| ProblemError::internal("IDAS initial sensitivity missing"))?;
            for k in 0..parameters.len() {
                let values: Vec<_> = (0..n)
                    .map(|r| j.get(r, n + k).copied().unwrap_or(0.0))
                    .collect();
                let v = s.vector(&values)?;
                s.sens.push(v);
                let v = s.vector(&vec![0.0; n])?;
                s.dsens.push(v);
            }
            unsafe {
                check(
                    ffi::IDASensInit(
                        s.mem,
                        parameters
                            .len()
                            .try_into()
                            .map_err(|_| ProblemError::unsupported("IDAS parameter extent"))?,
                        ffi::IDA_SIMULTANEOUS,
                        Some(sensitivities),
                        s.sens.as_mut_ptr(),
                        s.dsens.as_mut_ptr(),
                    ),
                    "forward sensitivities",
                )?;
                let mut atol_s = Vec::new();
                for scale in &p.parameter_scales {
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
        s.consistent(p.start, p)?;
        Ok(s)
    }
    fn initialize_roots(&mut self) -> Result<(), ProblemError> {
        let n = self.callback.contract.events[self.callback.mode].len();
        unsafe {
            check(
                ffi::IDARootInit(
                    self.mem,
                    n.try_into()
                        .map_err(|_| ProblemError::unsupported("IDAS root extent"))?,
                    if n == 0 { None } else { Some(roots) },
                ),
                "roots",
            )
        }
    }
    fn consistent(&mut self, t: f64, p: &Profile) -> Result<(), ProblemError> {
        let flag = unsafe {
            ffi::IDACalcIC(
                self.mem,
                ffi::IDA_YA_YDP_INIT,
                t + (p.end - t).min(p.initial_step),
            )
        };
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
            if p.sensitivities {
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
    fn sample(&mut self, t: f64, p: &Profile, stepped: bool) -> Result<Sample, ProblemError> {
        let n = self.callback.contract.states.len();
        let np = self.callback.parameters.len();
        let x = unsafe { read(self.y, n) };
        let Some(e) = self
            .callback
            .evaluate(Function::Output, t, &x, p.sensitivities)
        else {
            return Err(self.callback.failed("output"));
        };
        let mut state_sensitivities = Vec::new();
        let mut output_sensitivities = Vec::new();
        if p.sensitivities {
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
                .ok_or_else(|| ProblemError::internal("IDAS output derivative missing"))?;
            for row in 0..e.values.len() {
                for (k, column) in columns.iter().enumerate().take(np) {
                    output_sensitivities.push(
                        j.get(row, n + k).copied().unwrap_or(0.0)
                            + (0..n)
                                .map(|i| j.get(row, i).copied().unwrap_or(0.0) * column[i])
                                .sum::<f64>(),
                    );
                }
            }
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
                read(self.quad, self.callback.contract.quadratures.len())
            }
        };
        Ok(Sample {
            mode: 0,
            time: t,
            state: x,
            outputs: e.values,
            state_sensitivities,
            output_sensitivities,
            integrals,
        })
    }
}

/// Execute the smooth IDAS profile on its owning worker.
pub(super) fn integrate_with_progress(
    oracle: &mut dyn Oracle,
    p: &Profile,
    parameters: &[f64],
    cancel: Cancellation,
    progress: Arc<Progress>,
) -> Result<Report, ProblemError> {
    p.validate(oracle.contract(), parameters)?;
    if !p.changes.is_empty() || oracle.contract().events.iter().any(|e| !e.is_empty()) {
        return Err(ProblemError::unsupported(
            "IDAS currently admits smooth fixed-mass systems; use Diffsol for hybrid resets",
        ));
    }
    let execution = Execution {
        cancel,
        started: Instant::now(),
        time_limit: p.time_limit,
        progress,
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
            parameters,
            false,
        )
    });
    let initial = match initial {
        Ok(initial) => initial,
        Err(error) => return Ok(failure(r, error)),
    };
    r.requested_initial = initial.values;
    let mut s = match Session::new(oracle, parameters, p, execution.clone()) {
        Ok(session) => session,
        Err(error) => return Ok(failure(r, error)),
    };
    r.consistent_initial = unsafe { read(s.y, n) };
    let mut steps = 0;
    let mut time = p.start;
    let max_steps: std::ffi::c_long = p
        .max_steps
        .try_into()
        .map_err(|_| ProblemError::unsupported("IDAS step allowance extent"))?;
    let result = (|| -> Result<(), ProblemError> {
        for target in p.samples.iter().copied().chain(std::iter::once(p.end)) {
            if target > time {
                if steps >= max_steps {
                    r.termination = Termination::StepLimit;
                    return Ok(());
                }
                unsafe {
                    check(ffi::IDASetStopTime(s.mem, target), "stop time")?;
                    check(
                        ffi::IDASetMaxNumSteps(s.mem, max_steps - steps),
                        "remaining steps",
                    )?;
                }
                let flag = unsafe {
                    ffi::IDASolve(s.mem, target, &raw mut time, s.y, s.dy, ffi::IDA_NORMAL)
                };
                unsafe {
                    check(ffi::IDAGetNumSteps(s.mem, &raw mut steps), "step count")?;
                }
                r.statistics = vec![
                    serde_json::json!({"backend":"idas", "native_version":crate::sundials_version(), "native_flag":flag,"steps":steps,"derivatives":"analytic state and parameter partials"}),
                ];
                if flag == ffi::IDA_TOO_MUCH_WORK {
                    r.termination = Termination::StepLimit;
                    return Ok(());
                }
                if flag < 0 {
                    return Err(s.callback.native_failure(flag, "integration step"));
                }
                r.completed_time = time;
                s.callback
                    .callback
                    .execution
                    .progress
                    .push(crate::solve::Event {
                        phase: "idas.output".into(),
                        elapsed: s.callback.callback.execution.started.elapsed(),
                        values: std::collections::BTreeMap::from([
                            ("time".into(), crate::solve::Metric::Real(time)),
                            (
                                "steps".into(),
                                crate::solve::Metric::Integer(long_counter(steps)),
                            ),
                        ]),
                    });
            }
            if r.samples.len() < p.samples.len() && p.samples[r.samples.len()] == time {
                r.samples.push(s.sample(time, p, steps > 0)?);
            }
        }
        r.termination = Termination::Completed;
        Ok(())
    })();
    if let Err(e) = result {
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
fn long_counter(value: std::ffi::c_long) -> i64 {
    i64::from(value)
}
