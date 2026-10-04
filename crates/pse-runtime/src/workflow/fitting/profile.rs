// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Profile-likelihood intervals by adaptive pin chains (Plan 22 S3; ADR-0118 items 8 and 9;
//! PS-11, PS-12).
//!
//! **Chains.** Each free parameter has two chains, one per end. A point pins the parameter
//! with [`native::transform::Pinned`] on the fit prepared once and re-solves the fit over
//! the other coordinates through the one NLP runner; its seed is the solution of the
//! chain's latest accepted point below the threshold, or the fit estimate, and is recorded
//! as the point's input (PS-11). Chains run in parallel, as many at once as the fit's job
//! admitted cores for, each pinned fit on the fit's own threads.
//!
//! **Statistic.** With `f = ½·χ²` the objective, the signed root `r = √(2(f - f*))` of the
//! likelihood-ratio statistic is compared with `r* = √χ²₁(level)`. On a linear model `r`
//! is linear in the pinned value, `|θ - θ̂|/σ`, so the end is `θ̂ ± r*·σ`, the Wald end.
//!
//! **Step control** (homotopy style). The predictor extrapolates `r` along the secant of
//! the last two accepted points to reach `r*`, at most doubling the step; a failed pinned
//! fit with a recoverable numerical cause halves it toward the last accepted point.
//! Contract, resource and operational failures end the chain. Once a point passes the threshold, the
//! end is bracketed and the next pin is the secant root in `r` between the bracket's
//! points, exact on a linear model. A chain ends at a point within the tolerance of `r*`
//! (`threshold`), at the parameter's declared bound below it (`bound`), or stops: a failure
//! at the smallest step, the point budget, the fit's deadline or cancellation, or a pinned
//! fit whose objective falls below the estimate's (`stopped`).
use super::*;
use super::{covariance::IntervalBound, oracle::FitOracle};
use native::{
    ProblemError,
    kkt::Withheld,
    solve::{Backend, Compatibility, Execution, Qualification, Termination},
};
use pse_model::diagnostic::{BoundaryDiagnostic, DiagnosticProjection};
use pse_relations::generated::enums::{IntervalEnd, IntervalOutcome};
use statrs::distribution::{ChiSquared, ContinuousCDF};
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

/// One pinned fit of a profile chain.
#[derive(Clone, Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProfilePoint {
    /// The pinned parameter value, in the parameter's unit.
    pub value: f64,
    /// The chain point whose solution seeded this fit; `None` for the fit estimate (PS-11).
    pub seed: Option<usize>,
    /// The pinned fit's qualification, when the runner returned a report.
    pub qualification: Option<Qualification>,
    /// Actual native termination, when the runner returned a report.
    pub termination: Option<Termination>,
    /// Original typed cause; serialized through its owner's boundary diagnostic.
    pub failures: Vec<ProfileFailure>,
    /// The native callback owner latched a terminal failure, even for a trial-class cause.
    pub callback_terminal_failure: bool,
    /// The pinned fit's objective, half the weighted residual sum of squares.
    pub objective: Option<f64>,
    /// `√(2·max(f - f*, 0))`.
    pub statistic: Option<f64>,
    /// Qualified stationary or better and feasible, and used by the chain.
    pub accepted: bool,
    /// Why the pinned fit failed or was not used.
    pub detail: Option<String>,
}
/// A retained source failure. Clones share the same typed witness; presentation is derived.
#[derive(Clone, Debug, schemars::JsonSchema)]
#[schemars(with = "BoundaryDiagnostic")]
pub struct ProfileFailure(Arc<ProblemError>);
impl From<ProblemError> for ProfileFailure {
    fn from(error: ProblemError) -> Self {
        Self(Arc::new(error))
    }
}
impl ProfileFailure {
    /// Original native, mathematical or provider failure.
    pub fn cause(&self) -> &ProblemError {
        self.0.as_ref()
    }
    /// Structured boundary projection supplied by the source failure's owner.
    pub fn diagnostic(&self) -> BoundaryDiagnostic {
        self.0
            .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation)
    }
    fn subdivides(&self) -> bool {
        matches!(
            crate::math::strategy::failure(self.cause()),
            pse_model::generated::enums::NumericalAttemptObservation::NumericalFailure
        )
    }
    fn bytes(&self) -> usize {
        self.0.retained_bytes().saturating_add(128)
    }
}
impl PartialEq for ProfileFailure {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for ProfileFailure {}
impl serde::Serialize for ProfileFailure {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.diagnostic().serialize(serializer)
    }
}
/// Infrastructure outcome of one chain; this never becomes a scientific trial rejection.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(
    tag = "kind",
    content = "detail",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ProfileWorkerFailure {
    /// The chain began and its worker unwound. It is never replayed.
    Panic,
    /// A worker could not start or enter its execution environment; an unstarted
    /// chain carries this outcome when no remaining worker completes it.
    Scheduling(String),
    /// Entering the worker's native environment failed with an original typed cause.
    Environment(ProfileFailure),
}
/// The chain of one free parameter toward one end.
#[derive(Clone, Debug, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProfileChain {
    /// Typed worker failure, separate from the scientific interval outcome.
    pub worker_failure: Option<ProfileWorkerFailure>,
    /// Worker start or environment failures for this execution, including partial starts.
    pub scheduling_failures: Vec<ProfileWorkerFailure>,
    /// Successfully started workers in this profile execution.
    pub actual_parallelism: usize,
    /// The parameter.
    pub parameter: SemanticId,
    /// The end the chain moves toward.
    pub end: IntervalEnd,
    /// The fit estimate of the parameter.
    pub estimate: f64,
    /// The end reached.
    pub bound: IntervalBound,
    /// Why the chain stopped.
    pub detail: Option<String>,
    /// Every pinned fit, in solve order.
    pub points: Vec<ProfilePoint>,
}
fn chain_worker<T>(run: impl FnOnce() -> T) -> Result<T, ProfileWorkerFailure> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(run))
        .map_err(|_| ProfileWorkerFailure::Panic)
}
/// Production dispatch seam: tests can refuse starts before the OS spawn while every
/// successful start runs the same atomic claim loop and panic handling as production.
fn dispatch<T: Send>(
    count: usize,
    workers: usize,
    stack: usize,
    run: impl Fn(usize) -> T + Sync,
    failed: impl Fn(usize, ProfileWorkerFailure) -> T + Sync,
    environment: impl Fn(&(dyn Fn() + Sync)) -> Result<(), ProfileWorkerFailure> + Sync,
    spawn: impl Fn(usize) -> Result<(), String>,
) -> (Vec<T>, usize, Vec<ProfileWorkerFailure>) {
    if workers == 1 {
        return (
            (0..count)
                .map(|i| chain_worker(|| run(i)).unwrap_or_else(|e| failed(i, e)))
                .collect(),
            usize::from(count != 0),
            Vec::new(),
        );
    }
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<T>>> = Mutex::new((0..count).map(|_| None).collect());
    let mut scheduling = Vec::new();
    let started = std::thread::scope(|scope| {
        let work = || {
            environment(&|| loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                if index >= count {
                    break;
                }
                let value = chain_worker(|| run(index)).unwrap_or_else(|e| failed(index, e));
                results
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)[index] = Some(value);
            })
        };
        let mut handles = Vec::new();
        for index in 0..workers {
            if let Err(error) = spawn(index) {
                scheduling.push(ProfileWorkerFailure::Scheduling(error));
                continue;
            }
            match std::thread::Builder::new()
                .name("pse-profile".into())
                .stack_size(stack)
                .spawn_scoped(scope, work)
            {
                Ok(handle) => handles.push(handle),
                Err(error) => scheduling.push(ProfileWorkerFailure::Scheduling(error.to_string())),
            }
        }
        let started = handles.len();
        for handle in handles {
            match handle.join() {
                Ok(Ok(())) => {}
                Ok(Err(error)) => scheduling.push(error),
                Err(_) => scheduling.push(ProfileWorkerFailure::Panic),
            }
        }
        started
    });
    let detail = if scheduling.is_empty() {
        "no profile worker completed the unstarted chain".into()
    } else {
        scheduling
            .iter()
            .map(|failure| match failure {
                ProfileWorkerFailure::Panic => "profile worker panicked outside a chain",
                ProfileWorkerFailure::Scheduling(detail) => detail.as_str(),
                ProfileWorkerFailure::Environment(_) => "profile worker environment failed",
            })
            .collect::<Vec<_>>()
            .join("; ")
    };
    let unstarted = if scheduling
        .iter()
        .any(|failure| matches!(failure, ProfileWorkerFailure::Panic))
    {
        ProfileWorkerFailure::Panic
    } else if let Some(failure) = scheduling
        .iter()
        .find(|failure| matches!(failure, ProfileWorkerFailure::Environment(_)))
    {
        failure.clone()
    } else {
        ProfileWorkerFailure::Scheduling(detail)
    };
    (
        results
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .into_iter()
            .enumerate()
            .map(|(i, result)| result.unwrap_or_else(|| failed(i, unstarted.clone())))
            .collect(),
        started,
        scheduling,
    )
}
impl ProfileChain {
    /// Retained bytes.
    pub(super) fn bytes(&self) -> usize {
        size_of::<Self>()
            + self.detail.as_ref().map_or(0, String::capacity)
            + match &self.worker_failure {
                Some(ProfileWorkerFailure::Scheduling(detail)) => detail.capacity(),
                Some(ProfileWorkerFailure::Environment(failure)) => failure.bytes(),
                _ => 0,
            }
            + self.scheduling_failures.capacity() * size_of::<ProfileWorkerFailure>()
            + self
                .scheduling_failures
                .iter()
                .map(|failure| match failure {
                    ProfileWorkerFailure::Scheduling(detail) => detail.capacity(),
                    ProfileWorkerFailure::Environment(failure) => failure.bytes(),
                    ProfileWorkerFailure::Panic => 0,
                })
                .sum::<usize>()
            + self.points.capacity() * size_of::<ProfilePoint>()
            + self
                .points
                .iter()
                .map(|p| {
                    p.detail.as_ref().map_or(0, String::capacity)
                        + p.failures.capacity() * size_of::<ProfileFailure>()
                        + p.failures.iter().map(ProfileFailure::bytes).sum::<usize>()
                })
                .sum::<usize>()
    }
}

/// One chain's task: the parameter's declaration index and column, its end and the first
/// step.
#[derive(Clone, Copy, Debug)]
struct Task {
    parameter: usize,
    column: OriginalCol,
    end: IntervalEnd,
    step: f64,
}
/// An accepted pinned fit.
struct Solution {
    objective: f64,
    primal: Vec<f64>,
    qualification: Qualification,
    termination: Termination,
}
/// A failed pinned solve retains report state independently from its original causes.
struct PinFailure {
    qualification: Option<Qualification>,
    termination: Option<Termination>,
    failures: Vec<ProfileFailure>,
    callback_terminal_failure: bool,
}
impl From<ProblemError> for PinFailure {
    fn from(error: ProblemError) -> Self {
        Self {
            qualification: None,
            termination: None,
            failures: vec![ProfileFailure(Arc::new(error))],
            callback_terminal_failure: false,
        }
    }
}
impl PinFailure {
    fn subdivides(&self) -> bool {
        !self.callback_terminal_failure
            && !self.failures.is_empty()
            && self.failures.iter().all(ProfileFailure::subdivides)
    }
    fn detail(&self) -> String {
        self.failures
            .iter()
            .map(|f| f.cause().to_string())
            .collect::<Vec<_>>()
            .join("; ")
    }
    /// Extract the original callback/validation witnesses before any statistical decision.
    fn assess(report: native::solve::SolveReport) -> Result<Solution, Self> {
        let qualification = report.qualification;
        let termination = report.termination.category;
        let mut failure = Self {
            qualification: Some(qualification),
            termination: Some(termination),
            failures: [
                report.shared_callback_failure(),
                report.shared_validation_failure(),
            ]
            .into_iter()
            .flatten()
            .map(ProfileFailure)
            .collect(),
            callback_terminal_failure: report.evidence.callback.terminal_failure,
        };
        if failure.callback_terminal_failure && failure.failures.is_empty() {
            failure.failures.push(
                ProblemError::Internal("terminal callback failure has no original cause".into())
                    .into(),
            );
        }
        // A native operational stop stays terminal even if its incumbent is stationary.
        if !matches!(
            termination,
            Termination::Success | Termination::Acceptable | Termination::FeasibleOnly
        ) || failure.failures.is_empty()
            && (!matches!(
                qualification,
                Qualification::Stationary
                    | Qualification::OptimalWithinTolerance
                    | Qualification::GapQualified
            ) || !report
                .quality
                .as_ref()
                .is_some_and(native::quality::Quality::feasible))
        {
            failure
                .failures
                .push(ProfileFailure(Arc::new(ProblemError::native(
                    native::NativeStatus {
                        backend: report.backend,
                        code: report.termination.code,
                        name: report.termination.name.clone(),
                    },
                    termination,
                    format!(
                        "the pinned fit is qualified {} ({})",
                        qualification.as_str(),
                        report.termination.name
                    ),
                ))));
        }
        if !failure.failures.is_empty() {
            return Err(failure);
        }
        let objective = report
            .observation
            .as_ref()
            .and_then(|o| o.objective)
            .filter(|f| f.is_finite());
        match (report.candidate, objective) {
            (Some(candidate), Some(objective)) => Ok(Solution {
                objective,
                primal: candidate.primal,
                qualification,
                termination,
            }),
            _ => {
                failure
                    .failures
                    .push(ProfileFailure(Arc::new(ProblemError::Internal(
                        "the qualified pinned fit observed no finite objective or candidate".into(),
                    ))));
                Err(failure)
            }
        }
    }
}
/// What every chain of one fit shares.
struct Chains<'a> {
    problem: &'a Arc<FitProblem>,
    backend: Backend,
    execution: &'a Execution,
    estimate: &'a [f64],
    /// The estimate's objective `f*`.
    optimum: f64,
    /// `r* = √χ²₁(level)`.
    threshold: f64,
    tolerance: f64,
    points: usize,
}
impl Chains<'_> {
    /// One pinned fit at `value`, seeded from `seed`.
    fn solve(&self, column: OriginalCol, value: f64, seed: &[f64]) -> Result<Solution, PinFailure> {
        if let Some(stop) = self.execution.stopped() {
            return Err(ProblemError::stopped(stop, "the fit stopped").into());
        }
        let p = self.problem;
        let mut controls = p.profile.solver.controls.clone();
        controls.time_limit = self
            .execution
            .time_limit
            .saturating_sub(self.execution.started.elapsed());
        let failed = PinFailure::from;
        let oracle = FitOracle::new(p.clone(), self.execution.clone()).map_err(failed)?;
        let pinned = native::transform::Pinned::new(Box::new(oracle), &[(column.get(), value)])
            .map_err(failed)?;
        let mut initial = seed.to_vec();
        initial[column.get()] = value;
        let report = native::execution::nlp(
            native::execution::Step {
                adapter: native::execution::adapter(self.backend),
                snapshot: &p.snapshot,
                structure: p.structural_assessment.as_ref(),
                settings: &p.profile.solver.backend,
                controls: &controls,
                accuracy: &p.accuracy,
                execution: self.execution.clone(),
                tolerances: &p.tolerances,
                normalization: &p.normalization,
                compatibility: Compatibility {
                    layout: p.key,
                    profile: p.profile_key.as_id(),
                    data: p.source_identity,
                    backend: self.backend,
                },
                warm: None,
            },
            &mut native::execution::Retained::default(),
            native::execution::Nlp {
                oracle: Box::new(pinned),
                initial: &initial,
                presolve: &p.profile.solver.presolve,
                intent: SolveIntent::Optimize,
                sense: pse_math::binding::ObjectiveSense::Minimize,
                limit: p.profile.max_cells,
                analysis: native::execution::Analysis::NONE,
            },
        )
        .map_err(failed)?;
        PinFailure::assess(report)
    }
    fn failed(&self, task: Task, failure: ProfileWorkerFailure) -> ProfileChain {
        ProfileChain {
            parameter: self.problem.declaration.parameters[task.parameter].symbol_id,
            end: task.end,
            estimate: self.estimate[task.column.get()],
            bound: IntervalBound {
                value: None,
                outcome: IntervalOutcome::Stopped,
            },
            detail: Some(match &failure {
                ProfileWorkerFailure::Panic => "profile chain worker panic".into(),
                ProfileWorkerFailure::Scheduling(reason) => reason.clone(),
                ProfileWorkerFailure::Environment(failure) => failure.cause().to_string(),
            }),
            points: Vec::new(),
            worker_failure: Some(failure),
            scheduling_failures: Vec::new(),
            actual_parallelism: 0,
        }
    }
    /// Run one chain to its end.
    fn run(&self, task: Task) -> ProfileChain {
        let p = self.problem;
        let column = task.column.get();
        let variable = &p.contract.variables[column];
        ChainPolicy {
            parameter: p.declaration.parameters[task.parameter].symbol_id,
            column,
            end: task.end,
            step: task.step,
            estimate: self.estimate,
            limit: if task.end == IntervalEnd::Lower {
                variable.lower
            } else {
                variable.upper
            },
            bound_tolerance: p.tolerances.variables[column],
            scale: p.declaration.parameters[task.parameter].scale,
            optimum: self.optimum,
            threshold: self.threshold,
            tolerance: self.tolerance,
            points: self.points,
            execution: self.execution,
        }
        .run(|value, seed| self.solve(task.column, value, seed))
    }
}
/// Statistical target and bracketing policy, independent of problem/native startup.
struct ChainPolicy<'a> {
    parameter: SemanticId,
    column: usize,
    end: IntervalEnd,
    step: f64,
    estimate: &'a [f64],
    limit: f64,
    bound_tolerance: f64,
    scale: f64,
    optimum: f64,
    threshold: f64,
    tolerance: f64,
    points: usize,
    execution: &'a Execution,
}
impl ChainPolicy<'_> {
    fn run(
        &self,
        mut solve: impl FnMut(f64, &[f64]) -> Result<Solution, PinFailure>,
    ) -> ProfileChain {
        let direction = if self.end == IntervalEnd::Lower {
            -1.0
        } else {
            1.0
        };
        let estimate = self.estimate[self.column];
        let smallest = 1e-9 * self.scale.max(estimate.abs());
        let mut chain = ProfileChain {
            worker_failure: None,
            scheduling_failures: Vec::new(),
            actual_parallelism: 1,
            parameter: self.parameter,
            end: self.end,
            estimate,
            bound: IntervalBound {
                value: None,
                outcome: IntervalOutcome::Stopped,
            },
            detail: None,
            points: Vec::new(),
        };
        let end = |mut chain: ProfileChain, value: Option<f64>, outcome, detail| {
            chain.bound = IntervalBound { value, outcome };
            chain.detail = detail;
            chain
        };
        if self.limit.is_finite() && (self.limit - estimate).abs() <= self.bound_tolerance {
            return end(chain, Some(self.limit), IntervalOutcome::Bound, None);
        }
        // The latest accepted point below the threshold: value, statistic, solution and
        // chain index (`None` for the estimate).
        let mut base = (estimate, 0.0, self.estimate.to_vec(), None::<usize>);
        let mut over: Option<(f64, f64)> = None;
        let mut ceiling: Option<f64> = None;
        let mut step = self.step.max(smallest);
        // An objective this far below the estimate's is a lower minimum, not noise.
        let lower = self.tolerance * self.threshold * self.threshold / 2.0;
        while chain.points.len() < self.points {
            let (at, statistic) = (base.0, base.1);
            let value = match over {
                // The secant root in r between the bracket's points.
                Some((beyond, above)) => {
                    let fraction =
                        ((self.threshold - statistic) / (above - statistic)).clamp(0.01, 0.99);
                    at + fraction * (beyond - at)
                }
                None => {
                    let mut value = at + direction * step;
                    if let Some(ceiling) = ceiling
                        && direction * (value - ceiling) >= 0.0
                    {
                        value = 0.5 * (at + ceiling);
                    }
                    if self.limit.is_finite() && direction * (value - self.limit) > 0.0 {
                        value = self.limit;
                    }
                    value
                }
            };
            let index = chain.points.len();
            let seed = base.3;
            let result = self.execution.stopped().map_or_else(
                || solve(value, &base.2),
                |stop| Err(ProblemError::stopped(stop, "the profile chain stopped").into()),
            );
            match result {
                Err(failure) => {
                    let subdivides = failure.subdivides();
                    let detail = failure.detail();
                    chain.points.push(ProfilePoint {
                        value,
                        seed,
                        qualification: failure.qualification,
                        termination: failure.termination,
                        failures: failure.failures,
                        callback_terminal_failure: failure.callback_terminal_failure,
                        objective: None,
                        statistic: None,
                        accepted: false,
                        detail: Some(detail.clone()),
                    });
                    // Halve the step toward the last accepted point.
                    let gap = (value - at).abs() / 2.0;
                    if !subdivides || gap < smallest || self.execution.stopped().is_some() {
                        return end(chain, None, IntervalOutcome::Stopped, Some(detail));
                    }
                    over = None;
                    ceiling = Some(value);
                    step = gap;
                }
                Ok(solution) => {
                    let increase = solution.objective - self.optimum;
                    let r = (2.0 * increase.max(0.0)).sqrt();
                    let below = increase < -lower;
                    chain.points.push(ProfilePoint {
                        value,
                        seed,
                        qualification: Some(solution.qualification),
                        termination: Some(solution.termination),
                        failures: Vec::new(),
                        callback_terminal_failure: false,
                        objective: Some(solution.objective),
                        statistic: Some(r),
                        accepted: !below,
                        detail: below
                            .then(|| "the pinned fit's objective is below the estimate's".into()),
                    });
                    if below {
                        let detail = format!(
                            "a pinned fit at {value} reached an objective {} below the estimate's",
                            -increase
                        );
                        return end(chain, None, IntervalOutcome::Stopped, Some(detail));
                    }
                    if (r - self.threshold).abs() <= self.tolerance * self.threshold {
                        return end(chain, Some(value), IntervalOutcome::Threshold, None);
                    }
                    if r > self.threshold {
                        over = Some((value, r));
                        continue;
                    }
                    let advanced = (value - at).abs();
                    let slope = (r - statistic) / advanced;
                    base = (value, r, solution.primal, Some(index));
                    if self.limit.is_finite() && value == self.limit {
                        return end(chain, Some(self.limit), IntervalOutcome::Bound, None);
                    }
                    if over.is_none() {
                        let predicted = if slope > 0.0 {
                            ((self.threshold - r) / slope).min(2.0 * advanced)
                        } else {
                            2.0 * advanced
                        };
                        step = predicted.max(smallest);
                    }
                }
            }
        }
        let detail = format!("the chain's budget of {} pinned fits ran out", self.points);
        end(chain, None, IntervalOutcome::Stopped, Some(detail))
    }
}

impl FitProblem {
    /// The two profile-likelihood chains of every free parameter, or why they are withheld:
    /// they need the estimate's own conditions, not full rank, which a flat profile shows.
    pub(super) fn profiles(
        self: &Arc<Self>,
        report: &FitReport,
        level: f64,
        controls: &ProfileControls,
        (route, workers): (native::routing::Route, usize),
        execution: &Execution,
    ) -> Result<Vec<ProfileChain>, FitWithheld> {
        self.estimate_valid(report)?;
        let (Some(estimate), Some(optimum)) = (report.candidate.as_deref(), report.objective)
        else {
            return Err(FitWithheld::Local(Withheld::NoCandidate));
        };
        let native::routing::Route::Native(backend) = route else {
            return Err(FitWithheld::Local(Withheld::NoCandidate));
        };
        let threshold = ChiSquared::new(1.0)
            .map(|c| c.inverse_cdf(level).sqrt())
            .ok()
            .filter(|r| r.is_finite() && *r > 0.0)
            .ok_or(FitWithheld::Local(Withheld::Backsolve))?;
        // A certified covariance sizes the first step at half the Wald half-width; without
        // one, a tenth of the parameter's declared scale.
        let errors = report
            .covariance
            .as_ref()
            .and_then(Covariance::standard_errors);
        let tasks: Vec<Task> = self
            .free()
            .enumerate()
            .flat_map(|(j, (parameter, column))| {
                let scale = self.declaration.parameters[parameter].scale;
                let step = errors
                    .as_ref()
                    .map(|e| 0.5 * threshold * e[j])
                    .filter(|s| s.is_finite() && *s > 0.0)
                    .unwrap_or(0.1 * scale);
                [IntervalEnd::Lower, IntervalEnd::Upper].map(|end| Task {
                    parameter,
                    column,
                    end,
                    step,
                })
            })
            .collect();
        let threads = self.profile.solver.controls.threads;
        let workers = workers.min(tasks.len()).max(1);
        let chains = Chains {
            problem: self,
            backend,
            execution,
            estimate,
            optimum,
            threshold,
            tolerance: controls.tolerance.into_inner(),
            points: controls.points.into_inner(),
        };
        let stack = self.runtime.native().stack_bytes();
        let (mut result, started, scheduling) = dispatch(
            tasks.len(),
            workers,
            stack,
            |index| chains.run(tasks[index]),
            |index, failure| chains.failed(tasks[index], failure),
            |work| {
                native::execution::scoped(
                    &[native::execution::adapter(backend)],
                    threads,
                    stack,
                    || -> Result<(), ProblemError> {
                        work();
                        Ok(())
                    },
                )
                .map_err(|error| ProfileWorkerFailure::Environment(ProfileFailure(Arc::new(error))))
            },
            |_| Ok(()),
        );
        for chain in &mut result {
            chain.actual_parallelism = started;
            chain.scheduling_failures = scheduling.clone();
        }
        Ok(result)
    }
}

#[cfg(test)]
mod worker_tests {
    use super::*;
    fn policy(execution: &Execution) -> ChainPolicy<'_> {
        ChainPolicy {
            parameter: SemanticId::NIL,
            column: 0,
            end: IntervalEnd::Upper,
            step: 1.0,
            estimate: &[0.0],
            limit: 10.0,
            bound_tolerance: 1e-8,
            scale: 1.0,
            optimum: 0.0,
            threshold: 1.0,
            tolerance: 1e-2,
            points: 8,
            execution,
        }
    }
    fn execution() -> Execution {
        Execution::new(
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
            &Default::default(),
        )
    }
    fn solution(value: f64) -> Solution {
        Solution {
            objective: value * value / 2.0,
            primal: vec![value],
            qualification: Qualification::Stationary,
            termination: Termination::Success,
        }
    }
    fn report(termination: Termination, execution: &Execution) -> native::solve::SolveReport {
        native::solve::SolveReport::new(
            Backend::Ipopt,
            &OracleContract {
                identity: ContentHash::from_bytes([1; 32]),
                variables: Vec::new(),
                rows: Vec::new(),
                derivatives: DerivativeOrder::Second,
                smoothness: DerivativeOrder::Second,
            },
            native::solve::NativeTermination {
                code: 123,
                name: "injected native stop".into(),
                message: None,
                category: termination,
                assurance: native::solve::Assurance::None,
            },
            execution,
        )
    }
    #[test]
    fn numerical_failure_subdivides_then_brackets_without_replacing_statistical_policy() {
        let execution = execution();
        let mut pins = Vec::new();
        let mut seeds = Vec::new();
        let chain = policy(&execution).run(|value, seed| {
            pins.push(value);
            seeds.push(seed.to_vec());
            if pins.len() == 1 {
                Err(ProblemError::numerical("injected trajectory failure").into())
            } else {
                Ok(solution(value))
            }
        });
        assert_eq!(pins[0..2], [1.0, 0.5]);
        assert_eq!(seeds[0], vec![0.0]);
        assert_eq!(seeds[1], vec![0.0]);
        assert_eq!(seeds[2], vec![0.5]);
        assert_eq!(chain.bound.outcome, IntervalOutcome::Threshold);
        assert!((chain.bound.value.unwrap() - 1.0).abs() <= 1e-2);
        assert!(matches!(
            chain.points[0].failures[0].cause(),
            ProblemError::Numerical { .. }
        ));
        assert!(!chain.points[0].accepted);
        assert_eq!(chain.points[1].seed, None);
        assert_eq!(chain.points[2].seed, Some(1));
    }
    #[test]
    fn terminal_profile_causes_never_subdivide_and_keep_the_original_error() {
        let source_id = SemanticId::from_bytes([9; 16]);
        let terminal = vec![
            ProblemError::Contract("numerical-looking contract".into()),
            ProblemError::Unsupported("required representation".into()),
            ProblemError::memory("allocation refused"),
            ProblemError::Limit {
                kind: native::LimitKind::Work,
                detail: "attempt cap".into(),
            },
            ProblemError::Cancelled,
            ProblemError::Internal("infrastructure invariant".into()),
            ProblemError::Provider(pse_kernels::ProviderError::Terminal(
                "provider stopped".into(),
            )),
            ProblemError::Math(pse_math::MathError::Instance {
                instance: source_id,
                checked_members: Default::default(),
                cause: Box::new(pse_math::MathError::Provider {
                    source_id,
                    provider: source_id,
                    cause: pse_kernels::ProviderError::DerivativeUnavailable {
                        capability: pse_kernels::DerivativeCapability::SelectorNeighborhood,
                        requested: DerivativeOrder::Second,
                        available: DerivativeOrder::First,
                        members: vec![source_id],
                    },
                }),
            }),
        ];
        for cause in terminal {
            let execution = execution();
            let mut attempt = Some(cause);
            let mut pins = Vec::new();
            let chain = policy(&execution).run(|value, _| {
                pins.push(value);
                Err(attempt
                    .take()
                    .expect("terminal cause must never run another pin")
                    .into())
            });
            assert_eq!(pins, vec![1.0]);
            assert_eq!(chain.bound.outcome, IntervalOutcome::Stopped);
            assert!(chain.bound.value.is_none());
            assert_eq!(chain.points.len(), 1);
            assert!(!chain.points[0].failures[0].subdivides());
            assert!(chain.worker_failure.is_none());
        }
    }
    #[test]
    fn nested_provider_trial_can_subdivide_but_native_terminal_latch_cannot() {
        let trial = || {
            ProblemError::Math(pse_math::MathError::Instance {
                instance: SemanticId::NIL,
                checked_members: Default::default(),
                cause: Box::new(pse_math::MathError::Provider {
                    source_id: SemanticId::NIL,
                    provider: SemanticId::NIL,
                    cause: pse_kernels::ProviderError::Trial("injected trial".into()),
                }),
            })
        };
        let execution = execution();
        let mut pins = Vec::new();
        let chain = policy(&execution).run(|value, _| {
            pins.push(value);
            if pins.len() == 1 {
                Err(trial().into())
            } else {
                Ok(solution(value))
            }
        });
        assert_eq!(pins[0..2], [1.0, 0.5]);
        assert_eq!(chain.bound.outcome, IntervalOutcome::Threshold);
        let mut pins = Vec::new();
        let chain = policy(&execution).run(|value, _| {
            pins.push(value);
            Err(PinFailure {
                qualification: Some(Qualification::Unqualified),
                termination: Some(Termination::Evaluation),
                failures: vec![trial().into()],
                callback_terminal_failure: true,
            })
        });
        assert_eq!(pins, vec![1.0]);
        assert_eq!(chain.bound.outcome, IntervalOutcome::Stopped);
        assert!(chain.points[0].callback_terminal_failure);
        assert!(chain.points[0].failures[0].subdivides());
        let mut latched = report(Termination::Evaluation, &execution);
        latched.record_validation_failure(trial());
        latched.evidence.callback.terminal_failure = true;
        let mut latched = Some(latched);
        let mut pins = Vec::new();
        let chain = policy(&execution).run(|value, _| {
            pins.push(value);
            PinFailure::assess(latched.take().expect("terminal report must never repeat"))
        });
        assert_eq!(pins, vec![1.0]);
        assert!(chain.points[0].callback_terminal_failure);
        assert_eq!(chain.bound.outcome, IntervalOutcome::Stopped);
    }
    #[test]
    fn native_limits_stop_stationary_profile_reports_and_validation_cause_is_retained() {
        let execution = execution();
        for termination in [
            Termination::IterationLimit,
            Termination::TimeLimit,
            Termination::ResourceExhausted,
            Termination::Cancelled,
            Termination::Panic,
            Termination::Invalid,
        ] {
            let mut observed = report(termination, &execution);
            observed.qualification = Qualification::Stationary;
            let mut observed = Some(observed);
            let mut attempts = 0;
            let chain = policy(&execution).run(|_, _| {
                attempts += 1;
                PinFailure::assess(observed.take().expect("native stop must never repeat"))
            });
            assert_eq!(attempts, 1);
            assert_eq!(
                chain.points[0].qualification,
                Some(Qualification::Stationary)
            );
            assert_eq!(chain.points[0].termination, Some(termination));
            assert_eq!(chain.bound.outcome, IntervalOutcome::Stopped);
        }
        let mut observed = report(Termination::Success, &execution);
        observed.record_validation_failure(ProblemError::Provider(
            pse_kernels::ProviderError::Terminal("validation provider".into()),
        ));
        let original = observed.shared_validation_failure().unwrap();
        let failed = PinFailure::assess(observed).err().unwrap();
        assert!(!failed.subdivides());
        assert!(Arc::ptr_eq(&original, &failed.failures[0].0));
    }
    #[test]
    fn stopped_scope_never_starts_a_scripted_pin() {
        let execution = execution();
        execution.cancel.store(true, Ordering::Release);
        let chain = policy(&execution).run(|_, _| panic!("cancelled profile started a pin"));
        assert_eq!(chain.points.len(), 1);
        assert!(matches!(
            chain.points[0].failures[0].cause(),
            ProblemError::Cancelled
        ));
        assert_eq!(chain.bound.outcome, IntervalOutcome::Stopped);
    }
    #[test]
    fn worker_environment_retains_typed_failure_without_starting_a_chain() {
        let attempts = AtomicUsize::new(0);
        let failure =
            ProfileWorkerFailure::Environment(ProblemError::memory("worker native scope").into());
        let (results, started, failures) = dispatch(
            4,
            2,
            2 * 1024 * 1024,
            |_| {
                attempts.fetch_add(1, Ordering::Relaxed);
                Ok(())
            },
            |_, failure| Err(failure),
            |_| Err(failure.clone()),
            |_| Ok(()),
        );
        assert_eq!(started, 2);
        assert_eq!(attempts.load(Ordering::Relaxed), 0);
        assert_eq!(failures, vec![failure.clone(); 2]);
        assert_eq!(results, vec![Err(failure); 4]);
    }
    #[test]
    fn worker_panic_is_typed_and_never_replays_a_started_chain() {
        let attempts = AtomicUsize::new(0);
        let failed: Result<(), _> = chain_worker(|| {
            attempts.fetch_add(1, Ordering::Relaxed);
            panic!("injected chain failure");
        });
        assert_eq!(failed, Err(ProfileWorkerFailure::Panic));
        assert_eq!(attempts.load(Ordering::Relaxed), 1);
        assert_eq!(chain_worker(|| 7), Ok(7));
    }
    #[test]
    fn production_dispatch_covers_serial_pool_partial_and_zero_starts_without_replay() {
        for (workers, allowed, expected) in [(1, 1, 1), (4, 4, 4), (4, 2, 2), (4, 0, 0)] {
            let attempts: Vec<_> = (0..8).map(|_| AtomicUsize::new(0)).collect();
            let (result, started, scheduling) = dispatch(
                8,
                workers,
                2 * 1024 * 1024,
                |index| {
                    attempts[index].fetch_add(1, Ordering::Relaxed);
                    if index == 2 {
                        panic!("injected chain panic");
                    }
                    Ok(index)
                },
                |_, failure| Err(failure),
                |work| {
                    work();
                    Ok(())
                },
                |worker| {
                    if worker < allowed {
                        Ok(())
                    } else {
                        Err("injected spawn refusal".into())
                    }
                },
            );
            assert_eq!(started, expected);
            assert_eq!(scheduling.len(), workers - allowed);
            assert!(
                scheduling
                    .iter()
                    .all(|failure| matches!(failure, ProfileWorkerFailure::Scheduling(_)))
            );
            for (index, value) in result.into_iter().enumerate() {
                assert_eq!(
                    attempts[index].load(Ordering::Relaxed),
                    usize::from(allowed != 0)
                );
                if allowed == 0 {
                    assert!(matches!(value, Err(ProfileWorkerFailure::Scheduling(_))));
                } else if index == 2 {
                    assert_eq!(value, Err(ProfileWorkerFailure::Panic));
                } else {
                    assert_eq!(value, Ok(index));
                }
            }
        }
    }

    #[test]
    fn scheduling_report_counts_retained_string_capacity() {
        let mut detail = String::with_capacity(1024);
        detail.push_str("refused");
        let chain = ProfileChain {
            parameter: SemanticId::NIL,
            end: IntervalEnd::Lower,
            estimate: 0.0,
            bound: IntervalBound {
                value: None,
                outcome: IntervalOutcome::Stopped,
            },
            detail: None,
            points: Vec::new(),
            actual_parallelism: 0,
            worker_failure: Some(ProfileWorkerFailure::Scheduling(detail)),
            scheduling_failures: Vec::new(),
        };
        assert!(chain.bytes() >= size_of::<ProfileChain>() + 1024);
    }
    #[test]
    fn production_dispatch_records_environment_panics_without_starting_or_replaying_chains() {
        let attempts = AtomicUsize::new(0);
        let (results, started, failures) = dispatch(
            4,
            2,
            2 * 1024 * 1024,
            |_| {
                attempts.fetch_add(1, Ordering::Relaxed);
                Ok(())
            },
            |_, failure| Err(failure),
            |_| panic!("injected environment panic"),
            |_| Ok(()),
        );
        assert_eq!(started, 2);
        assert_eq!(attempts.load(Ordering::Relaxed), 0);
        assert_eq!(failures, vec![ProfileWorkerFailure::Panic; 2]);
        assert_eq!(results, vec![Err(ProfileWorkerFailure::Panic); 4]);
    }
}
