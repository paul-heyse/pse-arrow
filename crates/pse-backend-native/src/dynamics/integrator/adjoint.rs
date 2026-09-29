// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checkpointed adjoint gradients on Diffsol (ADR-0110 item 3).
//!
//! Every scheduled segment is its own Diffsol problem, whose operators map the contract
//! parameters to the segment's integration columns (I6). The driven forward loop records
//! the checkpoints itself — the reset metadata between Diffsol's own checkpoint segments is
//! crate-private — with at least the two end states of each segment, because
//! `Checkpointing::new` needs two, and it stores the Hermite points of each segment's last
//! interval with the derivative of the scheme's own interpolant.
//!
//! The backward pass runs segment by segment from the end. Diffsol integrates the adjoint
//! between observed samples, replays the forward pass from the checkpoints and applies the
//! initial-value correction at the first segment's start. At every observed sample the
//! adjoint jumps by [`sample_jump`], and the adjoint solver restarts from that state
//! through Diffsol's consistent initialization: the algebraic adjoint is recomputed for the
//! jumped differential adjoint, and a multistep scheme starts from the post-jump rates.
//! A later segment's start adjoint becomes the earlier segment's terminal adjoint, since
//! the differential states are continuous across a scheduled change.
use super::*;
use diffsol::{
    AdjointEquations, AdjointOdeSolverMethod, AugmentedOdeEquations, Checkpointing, DiffsolError,
    HermiteInterpolator, OdeSolverProblem, OdeSolverState, StateRef, StateRefMut,
    error::OdeSolverError, ode_solver::OdeSolverStatistics,
};
use std::cell::Ref;

type Problem<'o> = OdeSolverProblem<Equation<'o>>;
type Adjoint<'a, 'o, F> = AdjointEquations<'a, Equation<'o>, F>;
type Common = diffsol::ode_solver::state::StateCommon<V>;

/// Each forward scheme's statistics, an inherent method of each Diffsol solver.
trait Counted {
    fn statistics(&self) -> &OdeSolverStatistics;
}
impl<'o, LS: LinearSolver<M>> Counted for diffsol::Bdf<'_, Equation<'o>, LS> {
    fn statistics(&self) -> &OdeSolverStatistics {
        self.get_statistics()
    }
}
impl<'o, LS: LinearSolver<M>> Counted for diffsol::Sdirk<'_, Equation<'o>, LS> {
    fn statistics(&self) -> &OdeSolverStatistics {
        self.get_statistics()
    }
}
impl<'o> Counted for diffsol::ExplicitRk<'_, Equation<'o>> {
    fn statistics(&self) -> &OdeSolverStatistics {
        self.get_statistics()
    }
}

/// One Diffsol scheme's constructors: its forward solver, the terminal state of its
/// adjoint, and its adjoint solver from a state.
struct Scheme<Fw, St, Rs> {
    forward: Fw,
    start: St,
    resume: Rs,
}
impl<Fw, St, Rs> Scheme<Fw, St, Rs> {
    /// The constructors, typed against one problem lifetime.
    fn new<'a, 'o: 'a, F, B>(forward: Fw, start: St, resume: Rs) -> Self
    where
        F: OdeSolverMethod<'a, Equation<'o>>,
        B: OdeSolverMethod<'a, Equation<'o>>,
        Fw: Fn(&'a Problem<'o>) -> Result<F, DiffsolError>,
        St: Fn(&'a Problem<'o>, &mut Adjoint<'a, 'o, Replay<F>>) -> Result<B::State, DiffsolError>,
        Rs: Fn(&'a Problem<'o>, B::State, Adjoint<'a, 'o, Replay<F>>) -> Result<B, DiffsolError>,
    {
        Self {
            forward,
            start,
            resume,
        }
    }
}

/// What the passes produced beyond the report.
#[derive(Default)]
struct Outcome {
    gradient: Option<Vec<f64>>,
    checkpoints: usize,
}

/// The gradient of the cotangent's functional over the integration parameters.
pub(in crate::dynamics) fn gradient(
    oracle: &mut dyn Oracle,
    profile: &Profile,
    parameters: &[f64],
    cotangent: Cotangent<'_>,
    cancel: Cancellation,
    progress: Arc<crate::solve::Progress>,
) -> Result<Gradient, ProblemError> {
    let shared = Shared::new(oracle, profile, parameters, cancel, progress)?;
    let mut report = Report::new(profile.start);
    let mut outcome = Outcome::default();
    let result = catch_unwind(AssertUnwindSafe(|| match profile.diffsol.linear {
        DiffsolLinear::FaerLu => {
            schemes::<linear::FaerLu>(&shared, profile, &mut report, cotangent, &mut outcome)
        }
        DiffsolLinear::Klu => {
            schemes::<linear::Klu>(&shared, profile, &mut report, cotangent, &mut outcome)
        }
    }));
    shared.contain(result, &mut report);
    if report.termination != Termination::Completed {
        outcome.gradient = None;
    }
    Ok(Gradient {
        report,
        gradient: outcome.gradient,
        hessian: None,
        checkpoints: outcome.checkpoints,
        reserved_bytes: 0,
    })
}

/// The horizon's scheduled segments: each starts at the start or a change time and stops
/// at the next change time or the end.
struct Segments {
    starts: Vec<f64>,
    stops: Vec<f64>,
    boundaries: Vec<f64>,
}

/// One scheme's forward and backward passes over every segment problem.
fn schemes<LS: LinearSolver<M>>(
    shared: &Rc<Shared<'_>>,
    p: &Profile,
    r: &mut Report,
    cotangent: Cotangent<'_>,
    out: &mut Outcome,
) -> Result<(), ProblemError> {
    let np = shared.contract.parameters.len();
    let boundaries = p.boundaries();
    let segments = Segments {
        starts: std::iter::once(p.start)
            .chain(boundaries.iter().copied())
            .collect(),
        stops: boundaries
            .iter()
            .copied()
            .chain(std::iter::once(p.end))
            .collect(),
        boundaries,
    };
    let scales = p.integration_parameters(&p.parameter_scales);
    let mut problems = Vec::with_capacity(segments.starts.len());
    for start in &segments.starts {
        *shared.columns.borrow_mut() = p.columns_at(np, *start);
        problems.push(build_problem(shared, p, *start, &scales)?);
    }
    let passes = Passes {
        shared,
        p,
        segments: &segments,
    };
    match p.diffsol.method {
        DiffsolMethod::Bdf => passes.run(
            &problems,
            r,
            cotangent,
            out,
            Scheme::new(
                |problem| problem.bdf::<LS>(),
                |problem, adjoint| {
                    adjoint_start::<LS, diffsol::BdfState<V>, _>(problem, adjoint, true)
                },
                |problem, state, adjoint| {
                    problem.bdf_solver_adjoint_from_state::<LS, _>(state, adjoint)
                },
            ),
        ),
        DiffsolMethod::TrBdf2 => passes.run(
            &problems,
            r,
            cotangent,
            out,
            Scheme::new(
                |problem| problem.tr_bdf2::<LS>(),
                |problem, adjoint| {
                    adjoint_start::<LS, diffsol::RkState<V>, _>(problem, adjoint, true)
                },
                |problem, state, adjoint| {
                    problem.tr_bdf2_solver_adjoint_from_state::<LS, _>(state, adjoint)
                },
            ),
        ),
        DiffsolMethod::Esdirk34 => passes.run(
            &problems,
            r,
            cotangent,
            out,
            Scheme::new(
                |problem| problem.esdirk34::<LS>(),
                |problem, adjoint| {
                    adjoint_start::<LS, diffsol::RkState<V>, _>(problem, adjoint, true)
                },
                |problem, state, adjoint| {
                    problem.esdirk34_solver_adjoint_from_state::<LS, _>(state, adjoint)
                },
            ),
        ),
        DiffsolMethod::Tsit45 => passes.run(
            &problems,
            r,
            cotangent,
            out,
            Scheme::new(
                |problem| problem.tsit45(),
                |problem, adjoint| {
                    adjoint_start::<LS, diffsol::RkState<V>, _>(problem, adjoint, false)
                },
                |problem, state, adjoint| problem.tsit45_solver_adjoint_from_state(state, adjoint),
            ),
        ),
    }
}

/// The adjoint's terminal state at a segment's stop, built as Diffsol's `*_state_adjoint`
/// builds it (with consistent algebraic values for the implicit schemes) but without its
/// step-size probe. That probe takes an explicit step of 0.01·|y|/|y'| over the state's
/// forward slot and evaluates the adjoint there, which leaves the segment's checkpoints —
/// a Diffsol error — whenever the forward state is nearly steady at the stop. The backward
/// pass starts with the forward pass's last step instead, and the scheme's error control
/// adapts it.
fn adjoint_start<'a, 'o: 'a, LS, S, F>(
    problem: &'a Problem<'o>,
    adjoint: &mut Adjoint<'a, 'o, F>,
    implicit: bool,
) -> Result<S, DiffsolError>
where
    LS: LinearSolver<M>,
    S: OdeSolverState<V>,
    F: OdeSolverMethod<'a, Equation<'o>>,
{
    let t = adjoint.last_t();
    let mut state = S::new_without_initialise_augmented_at(problem, adjoint, t)?;
    *state.as_mut().t = t;
    *state.as_mut().h = -adjoint.last_h().unwrap_or(problem.h0);
    if implicit {
        let mut newton = diffsol::NewtonNonlinearSolver::new(LS::default(), diffsol::NoLineSearch);
        state.as_mut().set_consistent(problem, &mut newton)?;
        state
            .as_mut()
            .set_consistent_augmented(problem, adjoint, &mut newton)?;
    }
    Ok(state)
}

/// The forward pass and its checkpoints over one scheme.
struct Passes<'s, 'o> {
    shared: &'s Rc<Shared<'o>>,
    p: &'s Profile,
    segments: &'s Segments,
}
impl<'o> Passes<'_, 'o> {
    /// The forward pass stores its checkpoints; the cotangent reads its samples; the
    /// backward pass runs from the last segment to the first.
    fn run<'a, F, B, Fw, St, Rs>(
        &self,
        problems: &'a [Problem<'o>],
        r: &mut Report,
        cotangent: Cotangent<'_>,
        out: &mut Outcome,
        scheme: Scheme<Fw, St, Rs>,
    ) -> Result<(), ProblemError>
    where
        F: OdeSolverMethod<'a, Equation<'o>> + Counted,
        B: AdjointOdeSolverMethod<'a, Equation<'o>, Replay<F>>,
        Fw: Fn(&'a Problem<'o>) -> Result<F, DiffsolError>,
        St: Fn(
            &'a Problem<'o>,
            &mut Adjoint<'a, 'o, Replay<F>>,
        ) -> Result<<B as OdeSolverMethod<'a, Equation<'o>>>::State, DiffsolError>,
        Rs: Fn(
            &'a Problem<'o>,
            <B as OdeSolverMethod<'a, Equation<'o>>>::State,
            Adjoint<'a, 'o, Replay<F>>,
        ) -> Result<B, DiffsolError>,
    {
        let (shared, p, segments) = (self.shared, self.p, self.segments);
        let np = shared.contract.parameters.len();
        let mut stored = Vec::with_capacity(problems.len());
        let mut steps = 0usize;
        for (k, problem) in problems.iter().enumerate() {
            shared.check();
            let start = segments.starts[k];
            *shared.columns.borrow_mut() = p.columns_at(np, start);
            let seed = shared.seed.borrow().clone();
            let mut solver = Replay::new((scheme.forward)(problem).map_err(native)?);
            if r.requested_initial.is_empty() {
                r.requested_initial = ConstantOp::call(&problem.eqn.init(), start)
                    .as_slice()
                    .to_vec();
            }
            record_start(shared, &solver, p, r, start)?;
            // The segment's own unwind catch records its statistics before an abort
            // continues to the outer boundary.
            let attempt = catch_unwind(AssertUnwindSafe(|| {
                self.record(&mut solver, r, k, &mut steps, &mut out.checkpoints)
            }));
            r.statistics
                .push(segment_statistics(solver.statistics(), &p.diffsol)?);
            let checkpointing = match attempt {
                Ok(result) => result?,
                Err(payload) => resume_unwind(payload),
            };
            let Some(checkpointing) = checkpointing else {
                return Ok(());
            };
            let time = r.completed_time;
            let state = solver.state().y.as_slice().to_vec();
            *shared.seed.borrow_mut() = Some(state.clone());
            if segments.boundaries.get(k) == Some(&time) {
                if r.events.len() >= p.max_events {
                    r.termination = Termination::EventLimit;
                    return Ok(());
                }
                r.events.push(EventRecord {
                    event: None,
                    time,
                    before: state,
                    after: None,
                });
            }
            stored.push((checkpointing, solver, seed));
        }
        r.termination = Termination::Completed;
        let weights = cotangent(r)?;
        if weights.len()
            != r.samples
                .len()
                .saturating_mul(shared.contract.outputs.len())
            || weights.iter().any(|w| !w.is_finite())
        {
            return Err(contract("adjoint cotangent extent or value"));
        }
        let mut carried: Option<Vec<f64>> = None;
        let mut total = vec![0.0; shared.width()];
        for (k, (checkpointing, solver, seed)) in stored.into_iter().enumerate().rev() {
            shared.check();
            let problem = &problems[k];
            let (start, stop) = (segments.starts[k], segments.stops[k]);
            *shared.columns.borrow_mut() = p.columns_at(np, start);
            *shared.seed.borrow_mut() = seed;
            // The segment's samples, latest first; a sample at a change time observes the
            // later segment.
            let last = k + 1 == problems.len();
            let mut observed = (0..r.samples.len())
                .rev()
                .filter(|i| {
                    let t = r.samples[*i].time;
                    t >= start && (t < stop || (last && t <= stop))
                })
                .peekable();
            let mut adjoint = problem.adjoint_equations(vec![checkpointing], Some(solver), Some(1));
            let mut state = (scheme.start)(problem, &mut adjoint).map_err(native)?;
            if let (Some(lambda), Some(channel)) = (&carried, state.as_mut().s.first_mut()) {
                channel.as_mut_slice().copy_from_slice(lambda);
            }
            // A sample at the end jumps before the first backward step.
            if let Some(i) = observed.next_if(|i| r.samples[*i].time >= stop) {
                state = self.jumped::<B>(state, r, i, &weights)?;
            }
            let mut solver = resumed(&scheme, problem, state, adjoint)?;
            let mut first = None;
            for i in observed {
                let t = r.samples[i].time;
                if t <= start {
                    first = Some(i);
                    break;
                }
                self.step_to(&mut solver, t)?;
                let (state, adjoint) = split(solver)?;
                let state = self.jumped::<B>(state, r, i, &weights)?;
                solver = resumed(&scheme, problem, state, adjoint)?;
            }
            self.step_to(&mut solver, start)?;
            let (state, adjoint) = split(solver)?;
            let mut common = state.into_common();
            if let Some(i) = first {
                self.jump(r, i, &weights, &mut common)?;
            }
            // The first segment's initial values depend on the parameters; a later
            // segment's are carried by the adjoint instead.
            adjoint.correct_sg_for_init(start, &common.s, &mut common.sg);
            let (Some(lambda), Some(partial)) = (common.s.first(), common.sg.first()) else {
                return Err(ProblemError::internal("adjoint channel"));
            };
            for (sum, value) in total.iter_mut().zip(partial.as_slice()) {
                *sum += value;
            }
            carried = Some(lambda.as_slice().to_vec());
            shared.progress.push(crate::solve::Event {
                phase: "simulation.adjoint".into(),
                elapsed: shared.started.elapsed(),
                values: std::collections::BTreeMap::from([
                    ("time".into(), crate::solve::Metric::Real(start)),
                    ("segment".into(), crate::solve::Metric::Integer(k as i64)),
                ]),
                incumbent: None,
            });
        }
        if total.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite adjoint gradient"));
        }
        r.statistics.push(serde_json::json!({
            "adjoint": {
                "checkpoints": out.checkpoints,
                "steps_between_checkpoints": p.adjoint.steps_between_checkpoints.into_inner(),
                "segments": problems.len(),
            }
        }));
        out.gradient = Some(total);
        Ok(())
    }
    /// The adjoint state after the jump at sample `i`. The adjoint solver never integrates
    /// its forward slot, whose content is arbitrary after backward steps; the restart's
    /// consistent initialization solves the forward constraints once more, so the slot
    /// takes the sample's consistent forward state.
    fn jumped<'a, B: OdeSolverMethod<'a, Equation<'o>>>(
        &self,
        state: B::State,
        r: &Report,
        i: usize,
        weights: &[f64],
    ) -> Result<B::State, ProblemError>
    where
        'o: 'a,
    {
        let mut common = state.into_common();
        self.jump(r, i, weights, &mut common)?;
        common.y.as_mut_slice().copy_from_slice(&r.samples[i].state);
        common.dy.as_mut_slice().fill(0.0);
        Ok(B::State::new_from_common(common))
    }
    /// Add the jump at sample `i` ([`sample_jump`]) to the adjoint and to the gradient, in
    /// the segment's integration columns. The forward state is the sample's own.
    fn jump(
        &self,
        r: &Report,
        i: usize,
        weights: &[f64],
        state: &mut Common,
    ) -> Result<(), ProblemError> {
        let shared = self.shared;
        let sample = &r.samples[i];
        let m = shared.contract.outputs.len();
        let partials = |f: Function| {
            shared
                .evaluate(f, sample.time, &sample.state, true)
                .jacobian
                .ok_or_else(|| ProblemError::internal("adjoint jump partials"))
        };
        let output = partials(Function::Output)?;
        let rhs = if shared.contract.differential.contains(&false) {
            Some(partials(Function::Rhs)?)
        } else {
            None
        };
        let (lambda, parameters) = sample_jump(
            &shared.contract.differential,
            output.as_ref(),
            rhs.as_ref().map(|j| j.as_ref()),
            &weights[i * m..(i + 1) * m],
        )?;
        let (Some(adjoint), Some(partial)) = (state.s.first_mut(), state.sg.first_mut()) else {
            return Err(ProblemError::internal("adjoint channel"));
        };
        for (a, d) in adjoint.as_mut_slice().iter_mut().zip(&lambda) {
            *a += d;
        }
        let columns = shared.columns.borrow();
        let partial = partial.as_mut_slice();
        for (k, d) in parameters.iter().enumerate() {
            let slot = columns
                .get(k)
                .and_then(|c| partial.get_mut(*c))
                .ok_or_else(|| ProblemError::internal("adjoint jump columns"))?;
            *slot += d;
        }
        Ok(())
    }
    /// Integrate the adjoint back to `t`.
    fn step_to<'a, B: OdeSolverMethod<'a, Equation<'o>>>(
        &self,
        solver: &mut B,
        t: f64,
    ) -> Result<(), ProblemError>
    where
        'o: 'a,
    {
        match solver.set_stop_time(t) {
            Ok(()) => loop {
                self.shared.check();
                if solver.step().map_err(native)? == OdeSolverStopReason::TstopReached {
                    return Ok(());
                }
            },
            Err(DiffsolError::OdeSolverError(OdeSolverError::StopTimeAtCurrentTime)) => Ok(()),
            Err(e) => Err(native(e)),
        }
    }
    /// Drive one segment to its stop, storing a checkpoint every
    /// `steps_between_checkpoints` steps and the Hermite points since the last one, as
    /// Diffsol's own recorder does; `None` when the step allowance ran out.
    fn record<'a, F>(
        &self,
        s: &mut Replay<F>,
        r: &mut Report,
        k: usize,
        steps: &mut usize,
        count: &mut usize,
    ) -> Result<Option<Checkpointing<Equation<'o>, Point<F::State>>>, ProblemError>
    where
        F: OdeSolverMethod<'a, Equation<'o>>,
        'o: 'a,
    {
        let (shared, p) = (self.shared, self.p);
        let stop = self.segments.stops[k];
        let between = p.adjoint.steps_between_checkpoints.into_inner();
        let limit = p.adjoint.max_checkpoints.into_inner();
        let mut store = |state: Point<F::State>| {
            *count += 1;
            if *count > limit {
                return Err(ProblemError::memory(
                    "the adjoint forward pass needs more than max_checkpoints checkpoints",
                ));
            }
            Ok(state)
        };
        let mut checkpoints = vec![store(s.state_clone())?];
        let mut points = Hermite::default();
        points.push(s);
        let mut since = 0usize;
        let start = s.state().t;
        let mut first = true;
        s.set_stop_time(stop).map_err(native)?;
        loop {
            shared.check();
            if *steps >= p.max_steps {
                r.termination = Termination::StepLimit;
                return Ok(None);
            }
            *steps += 1;
            let reason = s.step().map_err(native)?;
            if std::mem::take(&mut first) {
                // A fresh solver's start rates leave the algebraic ones unsolved; the
                // first step's interpolant has them.
                let rate = s.inner.interpolate_dy(start).map_err(native)?;
                checkpoints[0].dy.copy_from(&rate);
                points.ydots[0] = rate;
            }
            let time = s.state().t;
            while let Some(&t) = p.samples.get(r.samples.len()) {
                // A sample at a scheduled change observes the post-transition state.
                if t > time || (t == time && self.segments.boundaries.contains(&t)) {
                    break;
                }
                r.samples.push(sample(shared, s, t, false)?);
            }
            r.completed_time = time;
            shared.progress.push(crate::solve::Event {
                phase: "simulation.step".into(),
                elapsed: shared.started.elapsed(),
                values: std::collections::BTreeMap::from([
                    ("time".into(), crate::solve::Metric::Real(time)),
                    ("steps".into(), crate::solve::Metric::Integer(*steps as i64)),
                ]),
                incumbent: None,
            });
            match reason {
                OdeSolverStopReason::TstopReached => break,
                OdeSolverStopReason::RootFound(..) => {
                    return Err(ProblemError::internal("root on the adjoint forward pass"));
                }
                OdeSolverStopReason::InternalTimestep => {
                    points.push(s);
                    since += 1;
                    if since > between {
                        checkpoints.push(store(s.checkpoint())?);
                        points = Hermite::default();
                        points.push(s);
                        since = 0;
                    }
                }
            }
        }
        points.push(s);
        checkpoints.push(store(s.state_clone())?);
        let start = checkpoints.len() - 2;
        Ok(Some(Checkpointing::new::<Replay<F>>(
            None,
            start,
            checkpoints,
            Some(points.finish()),
        )))
    }
}

/// A forward solver whose states carry the derivative of the scheme's interpolant.
/// Diffsol's checkpoint replay records `state().dy` as each Hermite point's rate, which a
/// BDF state keeps as its first backward difference (first-order accurate) and a fresh
/// implicit start leaves unsolved for algebraic states; interpolating the forward state
/// with those rates bends the adjoint's coefficients between steps. The wrapper delegates
/// every step to the scheme, so the replay takes the same steps.
#[derive(Clone)]
struct Replay<F> {
    inner: F,
    dy: V,
}
impl<'a, 'o: 'a, F: OdeSolverMethod<'a, Equation<'o>>> Replay<F> {
    fn new(inner: F) -> Self {
        let dy = rate(&inner);
        Self { inner, dy }
    }
    fn refresh(&mut self) {
        self.dy = rate(&self.inner);
    }
}
/// The derivative of the scheme's interpolant at its current time, or its state's own
/// rate when the scheme has no interpolant there yet.
fn rate<'a, 'o: 'a, F: OdeSolverMethod<'a, Equation<'o>>>(solver: &F) -> V {
    solver
        .interpolate_dy(solver.state().t)
        .unwrap_or_else(|_| solver.state().dy.clone())
}
impl<F: Counted> Counted for Replay<F> {
    fn statistics(&self) -> &OdeSolverStatistics {
        self.inner.statistics()
    }
}
impl<'a, 'o: 'a, F: OdeSolverMethod<'a, Equation<'o>>> OdeSolverMethod<'a, Equation<'o>>
    for Replay<F>
{
    type State = Point<F::State>;
    type Config = F::Config;
    fn problem(&self) -> &'a Problem<'o> {
        self.inner.problem()
    }
    fn checkpoint(&mut self) -> Self::State {
        let dy = self.dy.clone();
        Point {
            state: self.inner.checkpoint(),
            dy,
        }
    }
    fn state_clone(&self) -> Self::State {
        Point {
            state: self.inner.state_clone(),
            dy: self.dy.clone(),
        }
    }
    fn set_state(&mut self, state: Self::State) {
        self.inner.set_state(state.state);
        self.dy = state.dy;
    }
    fn into_state(self) -> Self::State {
        Point {
            state: self.inner.into_state(),
            dy: self.dy,
        }
    }
    fn state(&self) -> StateRef<'_, V> {
        StateRef {
            dy: &self.dy,
            ..self.inner.state()
        }
    }
    fn state_mut(&mut self) -> StateRefMut<'_, V> {
        self.inner.state_mut()
    }
    fn config(&self) -> &Self::Config {
        self.inner.config()
    }
    fn config_mut(&mut self) -> &mut Self::Config {
        self.inner.config_mut()
    }
    fn jacobian(&self) -> Option<Ref<'_, M>> {
        self.inner.jacobian()
    }
    fn mass(&self) -> Option<Ref<'_, M>> {
        self.inner.mass()
    }
    fn step(&mut self) -> Result<OdeSolverStopReason<f64>, DiffsolError> {
        let reason = self.inner.step()?;
        self.refresh();
        Ok(reason)
    }
    fn set_stop_time(&mut self, tstop: f64) -> Result<(), DiffsolError> {
        self.inner.set_stop_time(tstop)
    }
    fn interpolate_inplace(&self, t: f64, y: &mut V) -> Result<(), DiffsolError> {
        self.inner.interpolate_inplace(t, y)
    }
    fn interpolate_dy_inplace(&self, t: f64, dy: &mut V) -> Result<(), DiffsolError> {
        self.inner.interpolate_dy_inplace(t, dy)
    }
    fn interpolate_out_inplace(&self, t: f64, g: &mut V) -> Result<(), DiffsolError> {
        self.inner.interpolate_out_inplace(t, g)
    }
    fn interpolate_sens_inplace(&self, t: f64, sens: &mut [V]) -> Result<(), DiffsolError> {
        self.inner.interpolate_sens_inplace(t, sens)
    }
    fn state_mut_back(&mut self, t: f64) -> Result<(), DiffsolError> {
        self.inner.state_mut_back(t)?;
        self.refresh();
        Ok(())
    }
    fn order(&self) -> usize {
        self.inner.order()
    }
}
/// A checkpoint with the interpolant's rate at its time, which the replay's first Hermite
/// point reads; the scheme's own state is restored unchanged, so the replay repeats the
/// forward steps.
#[derive(Clone)]
struct Point<S> {
    state: S,
    dy: V,
}
impl<S: OdeSolverState<V>> OdeSolverState<V> for Point<S> {
    fn as_ref(&self) -> StateRef<'_, V> {
        StateRef {
            dy: &self.dy,
            ..self.state.as_ref()
        }
    }
    fn as_mut(&mut self) -> StateRefMut<'_, V> {
        self.state.as_mut()
    }
    fn into_common(self) -> diffsol::ode_solver::state::StateCommon<V> {
        let mut common = self.state.into_common();
        common.dy = self.dy;
        common
    }
    fn new_from_common(state: diffsol::ode_solver::state::StateCommon<V>) -> Self {
        let dy = state.dy.clone();
        Self {
            state: S::new_from_common(state),
            dy,
        }
    }
    fn set_problem<E: OdeEquations>(
        &mut self,
        problem: &OdeSolverProblem<E>,
    ) -> Result<(), DiffsolError> {
        self.state.set_problem(problem)
    }
    fn set_augmented_problem<E: OdeEquations, A: AugmentedOdeEquations<E>>(
        &mut self,
        problem: &OdeSolverProblem<E>,
        augmented: &A,
    ) -> Result<(), DiffsolError> {
        self.state.set_augmented_problem(problem, augmented)
    }
}

/// The dense points since the last checkpoint: the segment of the final interval, which
/// the backward pass starts in without replaying it.
#[derive(Default)]
struct Hermite {
    ys: Vec<V>,
    ydots: Vec<V>,
    ts: Vec<f64>,
}
impl Hermite {
    fn push<'a, 'o: 'a, F: OdeSolverMethod<'a, Equation<'o>>>(&mut self, s: &F) {
        self.ys.push(s.state().y.clone());
        self.ydots.push(s.state().dy.clone());
        self.ts.push(s.state().t);
    }
    fn finish(self) -> HermiteInterpolator<V> {
        HermiteInterpolator::new(self.ys, self.ydots, self.ts)
    }
}

/// A restarted adjoint changes the rate of the parameter integral: the solver's first step
/// starts from the current rate rather than the one of the state it was started from.
fn refresh<'a, 'o: 'a, F, B>(mut solver: B) -> B
where
    F: OdeSolverMethod<'a, Equation<'o>>,
    B: AdjointOdeSolverMethod<'a, Equation<'o>, F>,
{
    let t = solver.state().t;
    let rate = solver.state().s.first().cloned().and_then(|lambda| {
        solver
            .augmented_eqn()
            .and_then(|equations| equations.out())
            .map(|out| out.call(&lambda, t))
    });
    if let Some(rate) = rate
        && let Some(slot) = solver.state_mut().dsg.first_mut()
    {
        slot.copy_from(&rate);
    }
    solver
}

/// The adjoint solver's state and equations, for a restart.
fn split<'a, 'o: 'a, F, B>(solver: B) -> Result<(B::State, Adjoint<'a, 'o, F>), ProblemError>
where
    F: OdeSolverMethod<'a, Equation<'o>>,
    B: AdjointOdeSolverMethod<'a, Equation<'o>, F>,
{
    let (state, equations) = solver.into_state_and_eqn();
    Ok((
        state,
        equations.ok_or_else(|| ProblemError::internal("adjoint equations"))?,
    ))
}
/// The adjoint solver from a state, through Diffsol's consistent initialization.
fn resumed<'a, 'o: 'a, F, B, Fw, St, Rs>(
    scheme: &Scheme<Fw, St, Rs>,
    problem: &'a Problem<'o>,
    state: B::State,
    adjoint: Adjoint<'a, 'o, F>,
) -> Result<B, ProblemError>
where
    F: OdeSolverMethod<'a, Equation<'o>>,
    B: AdjointOdeSolverMethod<'a, Equation<'o>, F>,
    Rs: Fn(&'a Problem<'o>, B::State, Adjoint<'a, 'o, F>) -> Result<B, DiffsolError>,
{
    (scheme.resume)(problem, state, adjoint)
        .map(refresh)
        .map_err(native)
}
