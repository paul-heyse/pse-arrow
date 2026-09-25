// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Rust-only Diffsol adapter. All operator failures exit through one owned catch boundary.
use super::*;
use diffsol::{
    ConstantOp, ConstantOpSens, FaerContext, FaerSparseLU, FaerSparseMat, FaerVec, LinearOp,
    Matrix, NonLinearOp, NonLinearOpJacobian, NonLinearOpSens, OdeBuilder, OdeEquations,
    OdeEquationsRef, OdeSolverMethod, OdeSolverStopReason, Op, Vector, VectorHost,
};
use std::{
    cell::{Cell, RefCell},
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    rc::Rc,
    sync::atomic::Ordering,
    time::Instant,
};
type V = FaerVec<f64>;
type M = FaerSparseMat<f64>;
type Pattern = faer::sparse::SymbolicSparseColMat<usize>;
#[derive(Debug)]
struct Abort;
struct Shared<'o> {
    oracle: RefCell<&'o mut dyn Oracle>,
    contract: Contract,
    parameters: RefCell<Vec<f64>>,
    mode: Cell<usize>,
    integrals: RefCell<Vec<f64>>,
    seed: RefCell<Option<Vec<f64>>>,
    seed_sens: RefCell<Option<Vec<V>>>,
    parameter_active: Cell<bool>,
    failure: RefCell<Option<(Termination, ProblemError)>>,
    cancel: Cancellation,
    deadline: Instant,
    started: Instant,
    progress: Arc<crate::solve::Progress>,
    context: FaerContext,
}
impl Shared<'_> {
    fn abort(&self, status: Termination, error: ProblemError) -> ! {
        *self.failure.borrow_mut() = Some((status, error));
        resume_unwind(Box::new(Abort))
    }
    fn check(&self) {
        if self.cancel.load(Ordering::Acquire) {
            self.abort(
                Termination::Cancelled,
                pse_math::MathError::Cancelled.into(),
            );
        }
        if Instant::now() >= self.deadline {
            self.abort(Termination::TimeLimit, contract("dynamic deadline"));
        }
    }
    fn evaluate(&self, f: Function, t: f64, x: &[f64], derivative: bool) -> Evaluation {
        self.evaluate_mode(self.mode.get(), f, t, x, derivative)
    }
    fn evaluate_mode(
        &self,
        mode: usize,
        f: Function,
        t: f64,
        x: &[f64],
        derivative: bool,
    ) -> Evaluation {
        self.check();
        let result =
            self.oracle
                .borrow_mut()
                .evaluate(mode, f, t, x, &self.parameters.borrow(), derivative);
        match result {
            Ok(v) => {
                let n = self.nout_mode(mode, f);
                if v.values.len() != n
                    || v.values.iter().any(|x| !x.is_finite())
                    || derivative
                        && v.jacobian.as_ref().is_none_or(|j| {
                            j.nrows() != n
                                || j.ncols()
                                    != self.contract.states.len() + self.contract.parameters.len()
                                || j.val().iter().any(|v| !v.is_finite())
                        })
                {
                    self.abort(
                        Termination::Failed,
                        contract("dynamic function value/derivative contract"),
                    );
                }
                v
            }
            Err(e) => self.abort(Termination::Failed, e),
        }
    }
    fn nout_mode(&self, mode: usize, f: Function) -> usize {
        match f {
            Function::BalanceFlux => self.contract.balances.len(),
            Function::Output => self.contract.outputs.len(),
            Function::Roots => self.contract.events[mode].len(),
            _ => self.contract.states.len(),
        }
    }
}
#[derive(Clone)]
struct Operator<'o> {
    shared: Rc<Shared<'o>>,
    function: Function,
    mode: usize,
    state_pattern: Pattern,
    parameter_pattern: Pattern,
}
impl<'o> Operator<'o> {
    fn new(shared: Rc<Shared<'o>>, function: Function) -> Result<Self, ProblemError> {
        let mode = shared.mode.get();
        Self::for_mode(shared, function, mode)
    }
    fn for_mode(
        shared: Rc<Shared<'o>>,
        function: Function,
        mode: usize,
    ) -> Result<Self, ProblemError> {
        let n = shared.contract.states.len();
        let np = shared.contract.parameters.len();
        let m = shared.nout_mode(mode, function);
        let mut pairs = shared.oracle.borrow().support(mode, function);
        if function == Function::Initial {
            pairs.extend((0..n).flat_map(|r| (n..n + np).map(move |c| (r, c))));
        }
        if pairs.iter().any(|&(r, c)| r >= m || c >= n + np) {
            return Err(contract("dynamic support bounds"));
        }
        let pattern = |parameter: bool| {
            let indices: Vec<_> = pairs
                .iter()
                .filter_map(|&(r, c)| {
                    if parameter && c >= n {
                        Some(faer::sparse::Pair::new(r, c - n))
                    } else if !parameter && c < n {
                        Some(faer::sparse::Pair::new(r, c))
                    } else {
                        None
                    }
                })
                .collect();
            Pattern::try_new_from_indices(m, if parameter { np } else { n }, &indices)
                .map(|v| v.0)
                .map_err(|e| contract(&e.to_string()))
        };
        Ok(Self {
            state_pattern: pattern(false)?,
            parameter_pattern: pattern(true)?,
            shared,
            function,
            mode,
        })
    }
    fn partials(&self, x: &V, t: f64, matrix: &mut M, parameter: bool) {
        let result = self
            .shared
            .evaluate_mode(self.mode, self.function, t, x.as_slice(), true);
        let Some(j) = result.jacobian else {
            self.shared
                .abort(Termination::Failed, contract("missing dynamic partials"));
        };
        let offset = if parameter { self.nstates() } else { 0 };
        let target = matrix.inner_mut();
        for c in 0..target.ncols() {
            let rows = target.symbolic().row_idx()[target.col_range(c)].to_vec();
            let start = target.col_range(c).start;
            for (k, r) in rows.into_iter().enumerate() {
                target.val_mut()[start + k] = if parameter && !self.shared.parameter_active.get() {
                    0.0
                } else {
                    j.get(r, c + offset).copied().unwrap_or(0.0)
                };
            }
        }
    }
    fn product(&self, x: &V, t: f64, v: &V, y: &mut V, parameter: bool) {
        let pattern = if parameter {
            &self.parameter_pattern
        } else {
            &self.state_pattern
        };
        let mut matrix = M::new_from_sparsity(
            self.nout(),
            v.len(),
            Some(pattern.clone()),
            self.shared.context,
        );
        self.partials(x, t, &mut matrix, parameter);
        matrix.gemv(1.0, v, 0.0, y);
    }
}
impl Op for Operator<'_> {
    type T = f64;
    type V = V;
    type M = M;
    type C = FaerContext;
    fn context(&self) -> &FaerContext {
        &self.shared.context
    }
    fn nstates(&self) -> usize {
        self.shared.contract.states.len()
    }
    fn nparams(&self) -> usize {
        self.shared.contract.parameters.len()
    }
    fn nout(&self) -> usize {
        self.shared.nout_mode(self.mode, self.function)
    }
}
impl NonLinearOp for Operator<'_> {
    fn call_inplace(&self, x: &V, t: f64, y: &mut V) {
        y.as_mut_slice().copy_from_slice(
            &self
                .shared
                .evaluate_mode(self.mode, self.function, t, x.as_slice(), false)
                .values,
        );
    }
}
impl NonLinearOpJacobian for Operator<'_> {
    fn jac_mul_inplace(&self, x: &V, t: f64, v: &V, y: &mut V) {
        self.product(x, t, v, y, false);
    }
    fn jacobian_inplace(&self, x: &V, t: f64, y: &mut M) {
        self.partials(x, t, y, false);
    }
    fn jacobian_sparsity(&self) -> Option<Pattern> {
        Some(self.state_pattern.clone())
    }
}
impl NonLinearOpSens for Operator<'_> {
    fn sens_mul_inplace(&self, x: &V, t: f64, v: &V, y: &mut V) {
        self.product(x, t, v, y, true);
    }
    fn sens_inplace(&self, x: &V, t: f64, y: &mut M) {
        self.partials(x, t, y, true);
    }
    fn sens_sparsity(&self) -> Option<Pattern> {
        Some(self.parameter_pattern.clone())
    }
}
impl ConstantOp for Operator<'_> {
    fn call_inplace(&self, t: f64, y: &mut V) {
        if let Some(seed) = self.shared.seed.borrow().as_ref() {
            y.as_mut_slice().copy_from_slice(seed);
        } else {
            let x = vec![0.0; self.nstates()];
            y.as_mut_slice()
                .copy_from_slice(&self.shared.evaluate(Function::Initial, t, &x, false).values);
        }
    }
}
impl ConstantOpSens for Operator<'_> {
    fn sens_mul_inplace(&self, t: f64, v: &V, y: &mut V) {
        if let Some(seed) = self.shared.seed_sens.borrow().as_ref() {
            for i in 0..self.nstates() {
                y[i] = seed.iter().enumerate().map(|(j, s)| s[i] * v[j]).sum();
            }
            return;
        }
        self.product(
            &V::zeros(self.nstates(), self.shared.context),
            t,
            v,
            y,
            true,
        );
    }
    fn sens_inplace(&self, t: f64, y: &mut M) {
        if let Some(seed) = self.shared.seed_sens.borrow().as_ref() {
            let target = y.inner_mut();
            for (c, col) in seed.iter().enumerate() {
                let rows = target.symbolic().row_idx()[target.col_range(c)].to_vec();
                let start = target.col_range(c).start;
                for (k, r) in rows.into_iter().enumerate() {
                    target.val_mut()[start + k] = col[r];
                }
            }
            return;
        }
        self.partials(&V::zeros(self.nstates(), self.shared.context), t, y, true);
    }
    fn sens_sparsity(&self) -> Option<Pattern> {
        Some(self.parameter_pattern.clone())
    }
}
#[derive(Clone)]
struct Mass<'o>(Rc<Shared<'o>>);
impl Op for Mass<'_> {
    type T = f64;
    type V = V;
    type M = M;
    type C = FaerContext;
    fn context(&self) -> &FaerContext {
        &self.0.context
    }
    fn nstates(&self) -> usize {
        self.0.contract.states.len()
    }
    fn nparams(&self) -> usize {
        self.0.contract.parameters.len()
    }
    fn nout(&self) -> usize {
        self.nstates()
    }
}
impl LinearOp for Mass<'_> {
    fn gemv_inplace(&self, x: &V, _t: f64, beta: f64, y: &mut V) {
        for (i, d) in self.0.contract.differential.iter().enumerate() {
            y[i] = if *d { x[i] } else { 0.0 } + beta * y[i];
        }
    }
    fn sparsity(&self) -> Option<Pattern> {
        Some(
            <Pattern as diffsol::matrix::sparsity::MatrixSparsity<M>>::new_diagonal(self.nstates()),
        )
    }
}
struct Equation<'o> {
    rhs: Operator<'o>,
    init: Operator<'o>,
    out: Operator<'o>,
    root: Operator<'o>,
    mass: Mass<'o>,
    reset: Option<Operator<'o>>,
}
impl Op for Equation<'_> {
    type T = f64;
    type V = V;
    type M = M;
    type C = FaerContext;
    fn context(&self) -> &FaerContext {
        self.rhs.context()
    }
    fn nstates(&self) -> usize {
        self.rhs.nstates()
    }
    fn nparams(&self) -> usize {
        self.rhs.nparams()
    }
    fn nout(&self) -> usize {
        self.out.nout()
    }
}
impl<'a, 'o> OdeEquationsRef<'a> for Equation<'o> {
    type Mass = Mass<'o>;
    type Rhs = Operator<'o>;
    type Root = Operator<'o>;
    type Init = Operator<'o>;
    type Out = Operator<'o>;
    type Reset = Operator<'o>;
}
impl<'o> OdeEquations for Equation<'o> {
    fn rhs(&self) -> Operator<'o> {
        self.rhs.clone()
    }
    fn init(&self) -> Operator<'o> {
        self.init.clone()
    }
    fn out(&self) -> Option<Operator<'o>> {
        Some(self.out.clone())
    }
    fn root(&self) -> Option<Operator<'o>> {
        (self.root.nout() > 0).then(|| self.root.clone())
    }
    fn reset(&self) -> Option<Operator<'o>> {
        self.reset.clone()
    }
    fn mass(&self) -> Option<Mass<'o>> {
        self.mass
            .0
            .contract
            .differential
            .contains(&false)
            .then(|| self.mass.clone())
    }
    fn set_params(&mut self, p: &V) {
        *self.rhs.shared.parameters.borrow_mut() = p.as_slice().to_vec();
    }
    fn get_params(&self, p: &mut V) {
        p.as_mut_slice()
            .copy_from_slice(&self.rhs.shared.parameters.borrow());
    }
}
fn native(error: diffsol::DiffsolError) -> ProblemError {
    pse_math::MathError::Library(error.to_string()).into()
}

/// Same owned integration using the caller's bounded progress source.
pub(super) fn integrate_with_progress(
    oracle: &mut dyn Oracle,
    profile: &Profile,
    parameters: &[f64],
    cancel: Cancellation,
    progress: Arc<crate::solve::Progress>,
) -> Result<Report, ProblemError> {
    profile.validate(oracle.contract(), parameters)?;
    let contract_value = oracle.contract().clone();
    let shared = Rc::new(Shared {
        oracle: RefCell::new(oracle),
        contract: contract_value.clone(),
        parameters: RefCell::new(parameters.to_vec()),
        integrals: RefCell::new(vec![0.0; contract_value.balances.len()]),
        mode: Cell::new(0),
        seed: RefCell::new(None),
        seed_sens: RefCell::new(None),
        parameter_active: Cell::new(true),
        failure: RefCell::new(None),
        cancel,
        deadline: Instant::now()
            .checked_add(profile.time_limit)
            .ok_or_else(|| contract("dynamic deadline overflow"))?,
        started: Instant::now(),
        progress,
        context: FaerContext {
            par: faer::Par::Seq,
        },
    });
    let mut report = Report::new(profile.start);
    let result = catch_unwind(AssertUnwindSafe(|| {
        run(shared.clone(), profile, &mut report)
    }));
    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            report.termination = Termination::Failed;
            report.error = Some(e);
        }
        Err(payload) => {
            if payload.is::<Abort>() {
                if let Some((status, error)) = shared.failure.borrow_mut().take() {
                    report.termination = status;
                    report.error = Some(error);
                } else {
                    report.termination = Termination::Panic;
                    report.error = Some(contract("unattributed dynamic abort"));
                }
            } else {
                report.termination = Termination::Panic;
                report.error = Some(contract("panic inside Diffsol operation"));
            }
        }
    }
    (report.progress, report.dropped_progress) = shared.progress.snapshot();
    Ok(report)
}
fn run(shared: Rc<Shared<'_>>, p: &Profile, r: &mut Report) -> Result<(), ProblemError> {
    let mut time = p.start;
    let mut change = 0;
    let mut steps = 0usize;
    loop {
        shared.check();
        let eq = Equation {
            rhs: Operator::new(shared.clone(), Function::Rhs)?,
            init: Operator::new(shared.clone(), Function::Initial)?,
            out: Operator::new(
                shared.clone(),
                if shared.contract.balances.is_empty() {
                    Function::Output
                } else {
                    Function::BalanceFlux
                },
            )?,
            root: Operator::new(shared.clone(), Function::Roots)?,
            mass: Mass(shared.clone()),
            reset: None,
        };
        let params = shared.parameters.borrow().clone();
        let mut builder = OdeBuilder::<M>::new()
            .context(shared.context)
            .t0(time)
            .h0(p.initial_step)
            .rtol(p.rtol)
            .atol(p.atol.clone())
            .p(params)
            .param_scales(p.parameter_scales.clone());
        if let Some(tolerance) = p.out_rtol {
            builder = builder
                .integrate_out(true)
                .out_rtol(tolerance)
                .out_atol(p.out_atol.clone());
        }
        if p.sensitivities {
            builder = builder.sens_rtol(p.rtol).sens_atol(p.atol.clone());
        }
        let mut problem = builder.build_from_eqn(eq).map_err(native)?;
        problem.ic_options = copy_initial(&p.initialization);
        problem.ode_options = copy_native(&p.native);
        let requested = ConstantOp::call(&problem.eqn.init(), time)
            .as_slice()
            .to_vec();
        if r.requested_initial.is_empty() {
            r.requested_initial = requested;
        }
        let stop = p.changes.get(change).map_or(p.end, |v| v.time);
        let (event, state) = if p.sensitivities {
            let mut solver = problem.bdf_sens::<FaerSparseLU<f64>>().map_err(native)?;
            record_start(&shared, &solver, p, r, time)?;
            if time >= p.end {
                r.termination = Termination::Completed;
                return Ok(());
            }
            let attempt = catch_unwind(AssertUnwindSafe(|| {
                drive(&shared, &mut solver, p, r, stop, &mut steps)
            }));
            r.statistics.push(
                serde_json::to_value(solver.get_statistics())
                    .map_err(|e| contract(&e.to_string()))?,
            );
            if let Some(statistics) = r
                .statistics
                .last_mut()
                .and_then(serde_json::Value::as_object_mut)
            {
                statistics.insert("sensitivity_partials".into(), serde_json::json!({"state":"analytic", "parameter":"analytic", "event_time":"Diffsol finite-difference root/reset time partials", "reset":"Diffsol event-time correction and consistent mass reset"}));
            }
            let result = match attempt {
                Ok(result) => result?,
                Err(payload) => resume_unwind(payload),
            };
            if let Some(index) = result.0 {
                let event = &shared.contract.events[shared.mode.get()][index];
                if !event.terminal {
                    reset_sens(&shared, &mut solver, p, index)?;
                    *shared.seed.borrow_mut() = Some(solver.state().y.as_slice().to_vec());
                }
            }
            *shared.seed_sens.borrow_mut() = Some(solver.state().s.to_vec());
            result
        } else {
            let mut solver = problem.bdf::<FaerSparseLU<f64>>().map_err(native)?;
            record_start(&shared, &solver, p, r, time)?;
            if time >= p.end {
                r.termination = Termination::Completed;
                return Ok(());
            }
            let attempt = catch_unwind(AssertUnwindSafe(|| {
                drive(&shared, &mut solver, p, r, stop, &mut steps)
            }));
            r.statistics.push(
                serde_json::to_value(solver.get_statistics())
                    .map_err(|e| contract(&e.to_string()))?,
            );
            match attempt {
                Ok(result) => result?,
                Err(payload) => resume_unwind(payload),
            }
        };
        if matches!(
            r.termination,
            Termination::StepLimit | Termination::EventLimit
        ) {
            return Ok(());
        }
        time = r.completed_time;
        if let Some(index) = event {
            let guards = shared.evaluate(Function::Roots, time, &state, false).values;
            let events = &shared.contract.events[shared.mode.get()];
            if guards
                .iter()
                .zip(events)
                .filter(|(g, e)| g.abs() <= e.tolerance)
                .count()
                > 1
            {
                return Err(contract("ambiguous simultaneous dynamic events"));
            }
            let e = events
                .get(index)
                .ok_or_else(|| contract("native root index"))?
                .clone();
            if r.events.len() >= p.max_events {
                r.termination = Termination::EventLimit;
                return Ok(());
            }
            r.events.push(EventRecord {
                event: Some(e.id),
                time,
                before: state.clone(),
                after: None,
            });
            if e.terminal {
                if p.samples.get(r.samples.len()).is_some_and(|t| *t == time) {
                    let outputs = shared
                        .evaluate(Function::Output, time, &state, false)
                        .values;
                    r.samples.push(Sample {
                        balance_integrals: shared.integrals.borrow().clone(),
                        time,
                        state: state.clone(),
                        outputs,
                        state_sensitivities: vec![],
                        output_sensitivities: vec![],
                    });
                }
                r.termination = Termination::Event;
                return Ok(());
            }
            if !p.sensitivities {
                *shared.seed.borrow_mut() = Some(
                    shared
                        .evaluate(Function::Reset(index), time, &state, false)
                        .values,
                );
            }
            shared.mode.set(e.next_mode);
        } else {
            *shared.seed.borrow_mut() = Some(state);
        }
        // Roots precede a scheduled change at the same native stop time.
        let changed = p.changes.get(change).is_some_and(|c| c.time == time);
        if changed {
            if r.events.len() >= p.max_events {
                r.termination = Termination::EventLimit;
                return Ok(());
            }
            r.events.push(EventRecord {
                event: None,
                time,
                before: shared.seed.borrow().clone().unwrap_or_default(),
                after: None,
            });
            *shared.parameters.borrow_mut() = p.changes[change].parameters.clone();
            shared.parameter_active.set(false);
            change += 1;
        }
        if time >= p.end && event.is_none() && !changed {
            r.termination = Termination::Completed;
            return Ok(());
        }
    }
}
// The transition equation combines pre-event root/reset derivatives with post-event rates.
// Diffsol owns the saltation and mass-matrix consistency operations.
fn reset_sens<'p, 'o: 'p, S: OdeSolverMethod<'p, Equation<'o>>>(
    shared: &Rc<Shared<'o>>,
    solver: &mut S,
    p: &Profile,
    index: usize,
) -> Result<(), ProblemError> {
    let mode = shared.mode.get();
    let event = &shared.contract.events[mode][index];
    let state = solver.state();
    let roots = shared.evaluate(Function::Roots, state.t, state.y.as_slice(), true);
    if roots
        .values
        .iter()
        .zip(&shared.contract.events[mode])
        .filter(|(g, e)| g.abs() <= e.tolerance)
        .count()
        != 1
    {
        return Err(contract("ambiguous simultaneous sensitivity events"));
    }
    if p.samples
        .iter()
        .any(|t| (*t - state.t).abs() <= 8.0 * f64::EPSILON * (1.0 + state.t.abs()))
    {
        let j = roots
            .jacobian
            .ok_or_else(|| contract("missing root derivatives"))?;
        for (k, s) in state.s.iter().enumerate() {
            let moving = (0..state.y.len())
                .map(|i| j.get(index, i).copied().unwrap_or(0.0) * s[i])
                .sum::<f64>()
                + if shared.parameter_active.get() {
                    j.get(index, state.y.len() + k).copied().unwrap_or(0.0)
                } else {
                    0.0
                };
            if moving.abs() > 100.0 * f64::EPSILON {
                return Err(contract(
                    "fixed-time observation coincides with a parameter-dependent jump",
                ));
            }
        }
    }
    let eq = Equation {
        rhs: Operator::for_mode(shared.clone(), Function::Rhs, event.next_mode)?,
        root: Operator::for_mode(shared.clone(), Function::Roots, mode)?,
        reset: Some(Operator::for_mode(
            shared.clone(),
            Function::Reset(index),
            mode,
        )?),
        init: Operator::for_mode(shared.clone(), Function::Initial, mode)?,
        out: Operator::for_mode(shared.clone(), Function::Output, event.next_mode)?,
        mass: Mass(shared.clone()),
    };
    let parameters = shared.parameters.borrow().clone();
    let mut transition = OdeBuilder::<M>::new()
        .context(shared.context)
        .t0(state.t)
        .h0(p.initial_step)
        .rtol(p.rtol)
        .atol(p.atol.clone())
        .sens_rtol(p.rtol)
        .sens_atol(p.atol.clone())
        .p(parameters)
        .param_scales(p.parameter_scales.clone())
        .build_from_eqn(eq)
        .map_err(native)?;
    transition.ic_options = copy_initial(&p.initialization);
    if shared.contract.differential.contains(&false) {
        solver
            .state_mut()
            .apply_reset_with_sens_mass::<FaerSparseLU<f64>, _>(&transition, index)
            .map_err(native)?;
    } else {
        solver
            .state_mut()
            .apply_reset_with_sens(&transition, index)
            .map_err(native)?;
    }
    Ok(())
}
fn record_start<'p, 'o: 'p, S: OdeSolverMethod<'p, Equation<'o>>>(
    shared: &Rc<Shared<'o>>,
    s: &S,
    p: &Profile,
    r: &mut Report,
    time: f64,
) -> Result<(), ProblemError> {
    let state = s.state().y.as_slice().to_vec();
    if r.consistent_initial.is_empty() {
        r.consistent_initial = state.clone();
    }
    for event in r
        .events
        .iter_mut()
        .rev()
        .take_while(|e| e.time == time && e.after.is_none())
    {
        for balance in &shared.contract.balances {
            let jump = (state[balance.state] - event.before[balance.state]) * balance.scale;
            let declared = event
                .event
                .and_then(|id| balance.impulses.get(&id).copied())
                .unwrap_or(0.0);
            if !jump.is_finite() || (jump - declared).abs() > balance.tolerance {
                return Err(contract(
                    "conserved state jump differs from its declared event impulse",
                ));
            }
        }
        event.after = Some(state.clone());
    }
    {
        let roots = shared.evaluate(Function::Roots, time, &state, false).values;
        if roots
            .iter()
            .zip(&shared.contract.events[shared.mode.get()])
            .any(|(g, e)| g.abs() <= e.tolerance)
        {
            return Err(contract("initial or post-reset root is ambiguous"));
        }
    }
    if p.samples.get(r.samples.len()).is_some_and(|t| *t == time) {
        r.samples.push(sample(shared, s, time, p.sensitivities)?);
    }
    r.completed_time = time;
    Ok(())
}
fn drive<'p, 'o: 'p, S: OdeSolverMethod<'p, Equation<'o>>>(
    shared: &Rc<Shared<'o>>,
    s: &mut S,
    p: &Profile,
    r: &mut Report,
    stop: f64,
    steps: &mut usize,
) -> Result<(Option<usize>, Vec<f64>), ProblemError> {
    s.set_stop_time(stop).map_err(native)?;
    loop {
        shared.check();
        if *steps >= p.max_steps {
            r.termination = Termination::StepLimit;
            return Ok((None, s.state().y.as_slice().to_vec()));
        }
        *steps += 1;
        let reason = s.step().map_err(native)?;
        let (time, root) = match reason {
            OdeSolverStopReason::RootFound(t, i) => (t, Some(i)),
            _ => (s.state().t, None),
        };
        while let Some(&t) = p.samples.get(r.samples.len()) {
            if t > time || (t == time && (root.is_some() || p.changes.iter().any(|c| c.time == t)))
            {
                break;
            }
            r.samples.push(sample(shared, s, t, p.sensitivities)?);
        }
        r.completed_time = time;
        shared.progress.push(crate::solve::Event {
            phase: "simulation.step".into(),
            elapsed: shared.started.elapsed(),
            values: std::collections::BTreeMap::from([
                ("time".into(), crate::solve::Metric::Real(time)),
                ("steps".into(), crate::solve::Metric::Integer(*steps as i64)),
            ]),
        });
        if root.is_some() {
            s.state_mut_back(time).map_err(native)?;
        }
        if root.is_some() || matches!(reason, OdeSolverStopReason::TstopReached) {
            if !shared.contract.balances.is_empty() {
                let g = s.state().g;
                for (total, value) in shared.integrals.borrow_mut().iter_mut().zip(g.as_slice()) {
                    *total += value;
                }
            }
            return Ok((root, s.state().y.as_slice().to_vec()));
        }
    }
}
fn sample<'p, 'o: 'p, S: OdeSolverMethod<'p, Equation<'o>>>(
    shared: &Rc<Shared<'o>>,
    s: &S,
    t: f64,
    sens: bool,
) -> Result<Sample, ProblemError> {
    let y = if t == s.state().t {
        s.state().y.clone()
    } else {
        s.interpolate(t).map_err(native)?
    };
    let eval = shared.evaluate(Function::Output, t, y.as_slice(), sens);
    let np = shared.contract.parameters.len();
    let n = shared.contract.states.len();
    let mut dy = Vec::new();
    let mut dh = Vec::new();
    if sens {
        let vectors = if t == s.state().t {
            s.state().s.to_vec()
        } else {
            s.interpolate_sens(t).map_err(native)?
        };
        if vectors.len() != np {
            return Err(contract("native sensitivity count"));
        }
        let states = faer::Mat::from_fn(n, np, |i, j| vectors[j][i]);
        let Some(jac) = eval.jacobian else {
            return Err(contract("output sensitivity partials"));
        };
        let chain = faer::Mat::from_fn(n + np, np, |i, j| {
            if i < n {
                states[(i, j)]
            } else {
                f64::from(shared.parameter_active.get() && i - n == j)
            }
        });
        let mut result = faer::Mat::zeros(jac.nrows(), chain.ncols());
        faer::sparse::linalg::matmul::sparse_dense_matmul(
            result.as_mut(),
            faer::Accum::Replace,
            jac.as_ref(),
            chain.as_ref(),
            1.0,
            faer::Par::Seq,
        );
        for i in 0..n {
            for j in 0..np {
                dy.push(states[(i, j)]);
            }
        }
        for i in 0..eval.values.len() {
            for j in 0..np {
                dh.push(result[(i, j)]);
            }
        }
        if dy.iter().chain(&dh).any(|v| !v.is_finite()) {
            return Err(contract("nonfinite output sensitivity"));
        }
    }
    let balance_integrals = if shared.contract.balances.is_empty() {
        vec![]
    } else {
        let g = s.interpolate_out(t).map_err(native)?;
        g.as_slice()
            .iter()
            .zip(shared.integrals.borrow().iter())
            .map(|(v, total)| v + total)
            .collect()
    };
    Ok(Sample {
        balance_integrals,
        time: t,
        state: y.as_slice().to_vec(),
        outputs: eval.values,
        state_sensitivities: dy,
        output_sensitivities: dh,
    })
}

fn copy_initial(
    p: &diffsol::InitialConditionSolverOptions<f64>,
) -> diffsol::InitialConditionSolverOptions<f64> {
    diffsol::InitialConditionSolverOptions {
        use_linesearch: p.use_linesearch,
        max_linesearch_iterations: p.max_linesearch_iterations,
        max_newton_iterations: p.max_newton_iterations,
        max_linear_solver_setups: p.max_linear_solver_setups,
        step_reduction_factor: p.step_reduction_factor,
        armijo_constant: p.armijo_constant,
    }
}

fn copy_native(p: &diffsol::OdeSolverOptions<f64>) -> diffsol::OdeSolverOptions<f64> {
    diffsol::OdeSolverOptions {
        max_nonlinear_solver_iterations: p.max_nonlinear_solver_iterations,
        max_error_test_failures: p.max_error_test_failures,
        max_nonlinear_solver_failures: p.max_nonlinear_solver_failures,
        nonlinear_solver_tolerance: p.nonlinear_solver_tolerance,
        min_timestep: p.min_timestep,
        max_timestep_growth: p.max_timestep_growth,
        min_timestep_growth: p.min_timestep_growth,
        max_timestep_shrink: p.max_timestep_shrink,
        min_timestep_shrink: p.min_timestep_shrink,
        update_jacobian_after_steps: p.update_jacobian_after_steps,
        update_rhs_jacobian_after_steps: p.update_rhs_jacobian_after_steps,
        threshold_to_update_jacobian: p.threshold_to_update_jacobian,
        threshold_to_update_rhs_jacobian: p.threshold_to_update_rhs_jacobian,
        pi_control_proportional: p.pi_control_proportional,
        pi_control_integral: p.pi_control_integral,
    }
}
