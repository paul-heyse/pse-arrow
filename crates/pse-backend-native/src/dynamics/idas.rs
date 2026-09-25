// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "owned IDAS/SUNContext resources and panic-contained C callbacks"
)]
//! IDAS owns residual integration, consistent initialization and trial recovery.
use super::*;
use crate::{
    callback::CallbackState,
    solve::{Execution, Progress},
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
                Function::BalanceFlux => c.balances.len(),
                Function::Roots => c.events[*mode].len(),
                _ => c.states.len(),
            };
            if e.values.len() != rows
                || e.values.iter().any(|v| !v.is_finite())
                || derivatives
                    && e.jacobian.as_ref().is_none_or(|j| {
                        j.nrows() != rows
                            || j.ncols() != c.states.len() + c.parameters.len()
                            || j.val().iter().any(|v| !v.is_finite())
                    })
            {
                return Err(contract(
                    "IDAS function value/derivative dimensions or nonfinite values",
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
}
fn check(code: i32, operation: &str) -> Result<(), ProblemError> {
    if code == 0 {
        Ok(())
    } else {
        Err(contract(&format!("IDAS {operation}: native flag {code}")))
    }
}
fn index(n: usize) -> Result<ffi::sunindextype, ProblemError> {
    n.try_into().map_err(|_| contract("IDAS index extent"))
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
        let Some(e) = c.evaluate(Function::BalanceFlux, t, &x, false) else {
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
            return Err(contract("IDAS vector allocation failed"));
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
        let initial = s
            .callback
            .evaluate(Function::Initial, p.start, &vec![0.0; n], p.sensitivities)
            .ok_or_else(|| contract("IDAS initial function failed"))?;
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
                return Err(contract("IDAS memory allocation failed"));
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
                return Err(contract("IDAS matrix allocation failed"));
            }
            s.linear = ffi::SUNLinSol_KLU(s.y, s.matrix, s.ctx);
            if s.linear.is_null() {
                return Err(contract("IDAS KLU allocation failed"));
            }
            check(ffi::IDASetLinearSolver(s.mem, s.linear, s.matrix), "KLU")?;
            check(ffi::IDASetJacFn(s.mem, Some(jacobian)), "analytic Jacobian")?;
        }
        if p.sensitivities {
            let j = initial
                .jacobian
                .ok_or_else(|| contract("IDAS initial sensitivity missing"))?;
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
                            .map_err(|_| contract("IDAS parameter extent"))?,
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
        if !s.callback.contract.balances.is_empty() {
            s.quad = s.vector(&vec![0.0; s.callback.contract.balances.len()])?;
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
                    n.try_into().map_err(|_| contract("IDAS root extent"))?,
                    if n == 0 { None } else { Some(roots) },
                ),
                "roots",
            )
        }
    }
    fn consistent(&mut self, t: f64, p: &Profile) -> Result<(), ProblemError> {
        unsafe {
            check(
                ffi::IDACalcIC(
                    self.mem,
                    ffi::IDA_YA_YDP_INIT,
                    t + (p.end - t).min(p.initial_step),
                ),
                "consistent initial conditions",
            )?;
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
        let e = self
            .callback
            .evaluate(Function::Output, t, &x, p.sensitivities)
            .ok_or_else(|| contract("IDAS output failed"))?;
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
                .ok_or_else(|| contract("IDAS output derivative missing"))?;
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
        let balance_integrals = if self.quad.is_null() {
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
                read(self.quad, self.callback.contract.balances.len())
            }
        };
        Ok(Sample {
            time: t,
            state: x,
            outputs: e.values,
            state_sensitivities,
            output_sensitivities,
            balance_integrals,
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
        return Err(contract(
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
    let failure = |mut report: Report, error| {
        report.termination = match execution.stopped() {
            Some(crate::solve::Termination::Cancelled) => Termination::Cancelled,
            Some(crate::solve::Termination::TimeLimit) => Termination::TimeLimit,
            _ => Termination::Failed,
        };
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
        .map_err(|_| contract("IDAS step allowance extent"))?;
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
                    return Err(contract(&format!("IDAS native flag {flag}")));
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
        r.error = Some(e);
        r.termination = Termination::Failed;
    }
    if let Some((status, message)) = &s.callback.callback.terminal {
        r.termination = match status {
            crate::solve::Termination::Cancelled => Termination::Cancelled,
            crate::solve::Termination::TimeLimit => Termination::TimeLimit,
            crate::solve::Termination::Panic => Termination::Panic,
            _ => Termination::Failed,
        };
        r.error = Some(contract(message));
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
