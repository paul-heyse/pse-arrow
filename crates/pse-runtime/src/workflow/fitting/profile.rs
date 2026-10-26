// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Profile-likelihood intervals by adaptive pin chains (Plan 22 S3; ADR-0118 items 8 and 9;
//! PS-11, PS-12).
//!
//! **Chains.** Each free parameter has two chains, one per end. A point pins the parameter
//! with [`native::transform::Pinned`] on the fit prepared once and re-solves the fit over
//! the other coordinates through the one NLP runner; its seed is the solution of the
//! chain's latest accepted point below the threshold, or the fit estimate, and is recorded
//! as the point's input (PS-11). After estimation relinquishes active admission, the
//! profile phase admits its team, each pinned fit on the fit's own threads.
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
use crate::workflow::numerics::{CandidateDecision, CompletionEvidence, NativeEvidence};
#[cfg(test)]
use native::solve::Backend;
use native::{
    ProblemError,
    kkt::Withheld,
    solve::{Compatibility, Execution, Qualification, Termination},
};
use pse_model::diagnostic::{BoundaryDiagnostic, DiagnosticProjection};
use pse_model::generated::enums::{CandidateQualifier, CandidateRefusal};
use pse_model::{HeapUsage, generated::identities::RunId};
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
    /// Applicability and closure limitations of this independently checked pin.
    pub qualifiers: Vec<CandidateQualifier>,
    /// Actual residual-native evidence for this pin; absent for a native NLP pin.
    pub residual: Option<FitResidualReport>,
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
    pub(super) fn shared(cause: Arc<ProblemError>) -> Self {
        Self(cause)
    }
    pub(super) fn shared_cause(&self) -> Arc<ProblemError> {
        self.0.clone()
    }
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
    /// Limitations retained from the estimate and accepted pins through this interval end.
    pub qualifiers: Vec<CandidateQualifier>,
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

/// A profile product requires every requested parameter end, rather than merely
/// a successfully returned diagnostic collection of chains.
fn interval_validity(
    expected: impl IntoIterator<Item = SemanticId>,
    chains: &[ProfileChain],
) -> Result<(), (pse_model::generated::enums::WithheldReason, String)> {
    use pse_model::generated::enums::WithheldReason;
    use std::collections::BTreeSet;
    let expected = expected.into_iter().collect::<BTreeSet<_>>();
    let unavailable = |detail: &str| Err((WithheldReason::AnalysisUnavailable, detail.into()));
    if expected.is_empty() || expected.len().checked_mul(2) != Some(chains.len()) {
        return unavailable(
            "profile intervals lack the complete requested parameter-end inventory",
        );
    }
    let mut observed = BTreeSet::new();
    for chain in chains {
        let end = match chain.end {
            IntervalEnd::Lower => 0_u8,
            IntervalEnd::Upper => 1_u8,
        };
        if !expected.contains(&chain.parameter) || !observed.insert((chain.parameter, end)) {
            return unavailable(
                "profile intervals contain an unexpected or duplicate parameter end",
            );
        }
        if chain.worker_failure.is_some() {
            return unavailable("a requested profile chain has a worker failure");
        }
        if !matches!(
            chain.bound.outcome,
            IntervalOutcome::Bound | IntervalOutcome::Threshold
        ) || !chain.bound.value.is_some_and(f64::is_finite)
        {
            return unavailable(
                "a requested profile chain stopped without a finite interval endpoint",
            );
        }
    }
    Ok(())
}

impl PreparedFit {
    pub(in crate::workflow) fn profile_interval_validity(
        &self,
        chains: &[ProfileChain],
    ) -> Result<(), (pse_model::generated::enums::WithheldReason, String)> {
        interval_validity(
            self.problem
                .free()
                .map(|(index, _)| self.problem.declaration.parameters[index].symbol_id),
            chains,
        )
    }
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
    run: impl Fn(usize, usize) -> T + Sync,
    failed: impl Fn(usize, ProfileWorkerFailure) -> T + Sync,
    environment: impl Fn(&(dyn Fn() + Sync)) -> Result<(), ProfileWorkerFailure> + Sync,
    spawn: impl Fn(usize) -> Result<(), String>,
) -> (Vec<T>, usize, Vec<ProfileWorkerFailure>) {
    let observation = pse_diagnostics::observation::current();
    let _phase = pse_diagnostics::observation::phase("profile.dispatch");
    if workers == 1 {
        if count != 0
            && let Some(context) = &observation
        {
            context.count("profile.workers_started", "count", 1);
        }
        return (
            (0..count)
                .map(|i| chain_worker(|| run(0, i)).unwrap_or_else(|e| failed(i, e)))
                .collect(),
            usize::from(count != 0),
            Vec::new(),
        );
    }
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<T>>> = Mutex::new((0..count).map(|_| None).collect());
    let mut scheduling = Vec::new();
    let work = |worker| {
        let _entered = observation.as_ref().map(|context| context.enter());
        if let Some(context) = &observation {
            context.count("profile.workers_started", "count", 1);
        }
        environment(&|| loop {
            let index = next.fetch_add(1, Ordering::Relaxed);
            if index >= count {
                break;
            }
            let value = chain_worker(|| run(worker, index)).unwrap_or_else(|e| failed(index, e));
            results
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)[index] = Some(value);
        })
    };
    let started = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for index in 0..workers {
            if let Err(error) = spawn(index) {
                scheduling.push(ProfileWorkerFailure::Scheduling(error));
                continue;
            }
            let work = &work;
            match std::thread::Builder::new()
                .name("pse-profile".into())
                .stack_size(stack)
                .spawn_scoped(scope, move || work(index))
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
    if let Some(context) = &observation {
        context.count("profile.worker_failures", "count", scheduling.len() as u64);
        context.gauge("profile.actual_parallelism", "count", started as f64);
    }
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
            + self.qualifiers.capacity() * size_of::<CandidateQualifier>()
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
                    p.qualifiers.capacity() * size_of::<CandidateQualifier>()
                        + p.detail.as_ref().map_or(0, String::capacity)
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
/// Profile statistics require successful stationary pins in addition to every declared
/// original check. Accepting a limited feasible incumbent for ordinary use does not
/// establish a likelihood statistic.
fn pin_acceptance(
    native: NativeEvidence<'_>,
    original: CompletionEvidence<'_>,
    policy: &pse_model::numerics::NumericalPolicy,
) -> CandidateDecision {
    let mut decision = crate::workflow::numerics::original_completion(
        crate::workflow::numerics::native_evidence_use(native, policy),
        original,
        policy,
    )
    .decision;
    if !matches!(
        native.termination,
        Termination::Success | Termination::Acceptable | Termination::FeasibleOnly
    ) {
        decision.refuse(CandidateRefusal::NativeOutcome);
    }
    if !matches!(
        native.qualification,
        Qualification::Stationary
            | Qualification::OptimalWithinTolerance
            | Qualification::GapQualified
    ) {
        decision.refuse(CandidateRefusal::Unqualified);
    }
    decision
}
fn original_failure(error: WorkflowError) -> PinFailure {
    match error {
        WorkflowError::Math(error) => error.into_problem().into(),
        other => {
            let diagnostic = other.boundary_diagnostic();
            let retained = size_of::<BoundaryDiagnostic>() + diagnostic.heap_bytes();
            ProblemError::Math(pse_math::MathError::Typed {
                retained,
                cause: pse_model::diagnostic::DiagnosticCause::new(diagnostic),
            })
            .into()
        }
    }
}
/// An accepted pinned fit.
struct Solution {
    residual: Option<FitResidualReport>,
    objective: f64,
    primal: Vec<f64>,
    qualification: Qualification,
    termination: Termination,
    qualifiers: Vec<CandidateQualifier>,
}
/// A failed pinned solve retains report state independently from its original causes.
struct PinFailure {
    residual: Option<Box<FitResidualReport>>,
    qualification: Option<Qualification>,
    termination: Option<Termination>,
    failures: Vec<ProfileFailure>,
    callback_terminal_failure: bool,
}
impl From<ProblemError> for PinFailure {
    fn from(error: ProblemError) -> Self {
        Self {
            residual: None,
            qualification: None,
            termination: None,
            failures: vec![ProfileFailure(Arc::new(error))],
            callback_terminal_failure: false,
        }
    }
}
impl PinFailure {
    fn with_residual(mut self, report: &FitResidualReport) -> Self {
        self.qualification = Some(report.qualification);
        self.termination = Some(report.termination_category());
        self.residual = Some(Box::new(report.clone()));
        self
    }
    fn with_native(mut self, report: &native::solve::SolveReport) -> Self {
        self.qualification = Some(report.qualification);
        self.termination = Some(report.termination.category);
        self.callback_terminal_failure = report.evidence.callback.terminal_failure;
        self
    }
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
    fn assess(
        report: native::solve::SolveReport,
        original: CompletionEvidence<'_>,
        policy: &pse_model::numerics::NumericalPolicy,
    ) -> Result<Solution, Self> {
        let qualification = report.qualification;
        let termination = report.termination.category;
        let mut failure = Self {
            residual: None,
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
        let decision = pin_acceptance(NativeEvidence::nlp(&report), original, policy);
        if !decision.permits_use() && failure.failures.is_empty() {
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
                        "the pinned fit is qualified {} ({}): {}",
                        qualification.as_str(),
                        report.termination.name,
                        decision.reason()
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
                residual: None,
                qualifiers: decision.qualifiers,
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
#[derive(Clone, Copy)]
struct Chains<'a> {
    problem: &'a Arc<FitProblem>,
    math: &'a Arc<MathService>,
    prepared: &'a PreparedFit,
    run_id: RunId,
    qualifiers: &'a [CandidateQualifier],
    route: FitRoute,
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
    fn solve(
        &self,
        retained: &mut native::execution::Retained,
        column: OriginalCol,
        value: f64,
        seed: &[f64],
        scratch: Option<Arc<oracle::neutral::FitScratch>>,
    ) -> Result<Solution, PinFailure> {
        if let Some(stop) = self.execution.stopped() {
            return Err(ProblemError::stopped(stop, "the fit stopped").into());
        }
        let p = self.problem;
        if self.route == FitRoute::ResidualLm {
            let (mut evidence, candidate) = oracle::residual::solve(
                p,
                self.math,
                self.execution.clone(),
                seed,
                Some((column.get(), value)),
                scratch.clone(),
            )
            .map_err(PinFailure::from)?;
            // A latched callback stop is terminal for this native attempt. Retain
            // its literal report before attempting any independent source evaluation.
            if evidence.failure.is_some() || !evidence.termination.successful() {
                let category = evidence.termination_category();
                let terminal = evidence.failure.is_some();
                let failure = evidence.failure.clone().unwrap_or_else(|| {
                    ProfileFailure::from(match category {
                        Termination::IterationLimit => ProblemError::Limit {
                            kind: native::LimitKind::Work,
                            detail: "residual LM evaluation allowance".into(),
                        },
                        _ => ProblemError::numerical(format!(
                            "residual LM pin {:?} did not converge",
                            evidence.termination
                        )),
                    })
                });
                return Err(PinFailure {
                    qualification: Some(Qualification::Unqualified),
                    termination: Some(category),
                    residual: Some(Box::new(evidence)),
                    failures: vec![failure],
                    callback_terminal_failure: terminal,
                });
            }
            let observed = oracle::observe_pin(
                p,
                self.math,
                self.execution.clone(),
                &candidate,
                Some(column.get()),
                scratch.clone(),
            )
            .map_err(|error| PinFailure::from(error).with_residual(&evidence))?;
            let gradient = observed.gradient.unwrap_or(f64::INFINITY);
            evidence.stationarity = Some(gradient);
            evidence.qualification = if observed.quality.feasible()
                && gradient.is_finite()
                && gradient <= p.accuracy.stationarity
            {
                Qualification::Stationary
            } else if observed.quality.feasible() {
                Qualification::Feasible
            } else {
                Qualification::Unqualified
            };
            let (checks, _) = self
                .prepared
                .assess_candidate(
                    self.run_id,
                    &candidate,
                    &observed.trajectories,
                    &self.execution.scope().map_err(PinFailure::from)?,
                )
                .map_err(|error| original_failure(error).with_residual(&evidence))?;
            let decision = pin_acceptance(
                NativeEvidence::residual(&evidence, Some(&candidate), Some(&observed.quality)),
                CompletionEvidence::point(&checks, true, self.prepared.required_closure_checks()),
                &p.numerics.policy,
            );
            if !decision.permits_use() {
                let failure = ProfileFailure::from(ProblemError::numerical(format!(
                    "residual LM pin {:?}: {}",
                    evidence.termination,
                    decision.reason()
                )));
                return Err(PinFailure {
                    qualification: Some(evidence.qualification),
                    termination: Some(evidence.termination_category()),
                    residual: Some(Box::new(evidence)),
                    failures: vec![failure],
                    callback_terminal_failure: false,
                });
            }
            evidence.stationarity = Some(gradient);
            evidence.qualification = Qualification::Stationary;
            return Ok(Solution {
                objective: observed.objective,
                qualifiers: decision.qualifiers,
                primal: candidate,
                qualification: Qualification::Stationary,
                termination: evidence.termination_category(),
                residual: Some(evidence),
            });
        }
        let FitRoute::Native(backend) = self.route else {
            return Err(
                ProblemError::internal("nonconstant profile chain has no optimizer").into(),
            );
        };
        let mut controls = p.profile.solver.controls.clone();
        controls.time_limit = self
            .execution
            .time_limit
            .saturating_sub(self.execution.started.elapsed());
        let failed = PinFailure::from;
        let oracle = FitOracle::with_scratch(
            p.clone(),
            self.math.clone(),
            self.execution.clone(),
            scratch.clone(),
        )
        .map_err(failed)?;
        let pinned = native::transform::Pinned::new(Box::new(oracle), &[(column.get(), value)])
            .map_err(failed)?;
        let mut initial = seed.to_vec();
        initial[column.get()] = value;
        let report = native::execution::nlp(
            native::execution::Step {
                adapter: native::execution::adapter(backend),
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
                    backend,
                },
                warm: None,
            },
            retained,
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
        .map_err(failed);
        let report = match report {
            Ok(report) => report,
            Err(failure) => {
                retained.clear();
                return Err(failure);
            }
        };
        let native = NativeEvidence::nlp(&report);
        if !pin_acceptance(
            native,
            CompletionEvidence::point(&[], true, 0),
            &p.numerics.policy,
        )
        .permits_use()
        {
            retained.clear();
            return PinFailure::assess(
                report,
                CompletionEvidence::point(&[], false, self.prepared.required_closure_checks()),
                &p.numerics.policy,
            );
        }
        let assessed = (|| {
            let original = if let Some(candidate) = &report.candidate {
                let observed = oracle::observe_pin(
                    p,
                    self.math,
                    self.execution.clone(),
                    &candidate.primal,
                    None,
                    scratch,
                )
                .map_err(|error| PinFailure::from(error).with_native(&report))?;
                if !observed.quality.feasible() {
                    retained.clear();
                    return Err(PinFailure::from(ProblemError::numerical(
                        "fresh original pin is infeasible",
                    ))
                    .with_native(&report));
                }
                let (checks, _) = self
                    .prepared
                    .assess_candidate(
                        self.run_id,
                        &candidate.primal,
                        &observed.trajectories,
                        &self.execution.scope().map_err(PinFailure::from)?,
                    )
                    .map_err(|error| original_failure(error).with_native(&report))?;
                Some(checks)
            } else {
                None
            };
            PinFailure::assess(
                report,
                CompletionEvidence::point(
                    original.as_deref().unwrap_or(&[]),
                    original.is_some(),
                    self.prepared.required_closure_checks(),
                ),
                &p.numerics.policy,
            )
        })();
        if assessed.is_err() {
            retained.clear();
        }
        assessed
    }
    fn failed(&self, task: Task, failure: ProfileWorkerFailure) -> ProfileChain {
        ProfileChain {
            qualifiers: self.qualifiers.to_vec(),
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
        // Each finite chain owns and destroys its native state on this worker.
        // The adapter checks current pin bounds, layout and profile on every step.
        let mut retained = native::execution::Retained::default();
        let p = self.problem;
        // Construct/drop this neutral population on the actual chain owner, including
        // width-one inline dispatch. No provider or numerical point crosses pins.
        let scratch = oracle::neutral::FitScratch::new(p, self.math);
        let column = task.column.get();
        let variable = &p.contract.variables[column];
        let mut chain = ChainPolicy {
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
        .run(|value, seed| {
            let result = self.solve(&mut retained, task.column, value, seed, scratch.clone());
            if result.is_err()
                && let Some(scratch) = &scratch
            {
                scratch.discard();
            }
            result
        });
        for qualifier in self.qualifiers {
            if !chain.qualifiers.contains(qualifier) {
                chain.qualifiers.push(*qualifier);
            }
        }
        chain
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
        let mut operation = pse_diagnostics::observation::current().map(|parent| {
            parent.child(
                "profile.chain",
                pse_diagnostics::observation::Relationship::Nested,
            )
        });
        let context = operation.as_ref().map(|operation| operation.context());
        let _entered = context.as_ref().map(|context| context.enter());
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("profile.chain_attempts", "count", 1);
            context.source_ref("profile.parameter", self.parameter.to_string());
            context.text(
                "profile.end",
                if self.end == IntervalEnd::Lower {
                    "lower"
                } else {
                    "upper"
                },
            );
        }
        let direction = if self.end == IntervalEnd::Lower {
            -1.0
        } else {
            1.0
        };
        let estimate = self.estimate[self.column];
        let smallest = 1e-9 * self.scale.max(estimate.abs());
        let mut chain = ProfileChain {
            qualifiers: Vec::new(),
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
        let mut end = |mut chain: ProfileChain, value: Option<f64>, outcome, detail| {
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("profile.chains_returned", "count", 1);
                context.text("profile.interval_outcome", &format!("{outcome:?}"));
                context.boolean("profile.interval_available", value.is_some());
            }
            chain.bound = IntervalBound { value, outcome };
            chain.detail = detail;
            if let Some(operation) = operation.take() {
                operation.finish(pse_diagnostics::observation::Outcome::Completed);
            }
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
                || {
                    let _phase = pse_diagnostics::observation::phase("profile.pinned_fit");
                    if let Some(context) = pse_diagnostics::observation::current() {
                        context.count("profile.pinned_fit_attempts", "count", 1);
                        context.gauge("profile.pinned_value", "parameter coordinate", value);
                        context.boolean("profile.seed_from_chain", seed.is_some());
                    }
                    solve(value, &base.2)
                },
                |stop| Err(ProblemError::stopped(stop, "the profile chain stopped").into()),
            );
            match result {
                Err(failure) => {
                    let subdivides = failure.subdivides();
                    if let Some(context) = pse_diagnostics::observation::current() {
                        context.count("profile.failed_points", "count", 1);
                        context.boolean("profile.point_accepted", false);
                        context.unavailable(
                            "profile.statistic",
                            "failed point supplied no qualified statistic",
                        );
                        context.boolean("profile.failure_subdivides", subdivides);
                        context.boolean(
                            "profile.callback_terminal_failure",
                            failure.callback_terminal_failure,
                        );
                    }
                    let detail = failure.detail();
                    chain.points.push(ProfilePoint {
                        qualifiers: Vec::new(),
                        residual: failure.residual.map(|residual| *residual),
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
                    if let Some(context) = pse_diagnostics::observation::current() {
                        context.count("profile.solved_points", "count", 1);
                        context.gauge("profile.statistic", "signed-root likelihood ratio", r);
                        context.boolean("profile.point_accepted", !below);
                    }
                    chain.points.push(ProfilePoint {
                        qualifiers: solution.qualifiers.clone(),
                        residual: solution.residual.clone(),
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
                    if !below {
                        for qualifier in &solution.qualifiers {
                            if !chain.qualifiers.contains(qualifier) {
                                chain.qualifiers.push(*qualifier);
                            }
                        }
                    }
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

impl PreparedFit {
    /// The two profile-likelihood chains of every free parameter, or why they are withheld:
    /// they need the estimate's own conditions, not full rank, which a flat profile shows.
    pub(super) fn profiles(
        &self,
        math: &Arc<MathService>,
        run_id: RunId,
        report: &FitReport,
        level: f64,
        controls: &ProfileControls,
        (route, workers): (FitRoute, usize),
        execution: &Execution,
    ) -> Result<Vec<ProfileChain>, FitWithheld> {
        let p = &self.problem;
        crate::workflow::dynamics::analysis("fit.profile_likelihood_analysis", || {
            p.estimate_valid(report)?;
            let (Some(estimate), Some(optimum)) = (report.candidate.as_deref(), report.objective)
            else {
                return Err(FitWithheld::Local(Withheld::NoCandidate));
            };
            let adapters: Vec<&dyn native::execution::BackendExecution> = match route {
                FitRoute::Native(backend) => vec![native::execution::adapter(backend)],
                FitRoute::ResidualLm => vec![],
                FitRoute::Evaluation => return Err(FitWithheld::Local(Withheld::NoCandidate)),
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
            let tasks: Vec<Task> = p
                .free()
                .enumerate()
                .flat_map(|(j, (parameter, column))| {
                    let scale = p.declaration.parameters[parameter].scale;
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
            let threads = p.profile.solver.controls.threads;
            let experiment_cores = p
                .profile
                .experiment_workers
                .into_inner()
                .min(p.experiments.len())
                .min(math.cores());
            // Native fitting and its mutable callback execute in phases. A chain's
            // admitted envelope covers either its NLP team or its experiment lanes.
            let chain_cores = threads.max(experiment_cores);
            let workers = workers
                .min(tasks.len())
                .min(math.cores() / chain_cores)
                .max(1);
            let chains = Chains {
                problem: p,
                math,
                prepared: self,
                run_id,
                qualifiers: report
                    .completion
                    .as_ref()
                    .map_or(&[], |completion| completion.decision.qualifiers.as_slice()),
                route,
                execution,
                estimate,
                optimum,
                threshold,
                tolerance: controls.tolerance.into_inner(),
                points: controls.points.into_inner(),
            };
            let stack = math.stack_bytes();
            let core_extent = |workers: usize| {
                chain_cores.checked_mul(workers).ok_or_else(|| {
                    FitWithheld::Local(Withheld::Analysis(native::kkt::Unavailable::from(
                        ProblemError::memory("profile team cores"),
                    )))
                })
            };
            // Parent estimation storage stays resident. Guard each chain's additional
            // scratch and native thread/team stacks before the team is created.
            let adapter_stacks = native::execution::scope_stack_bytes(&adapters, threads, stack)
                .map_err(|error| {
                    FitWithheld::Local(Withheld::Analysis(native::kkt::Unavailable::from(error)))
                })?;
            let byte_extent = |workers: usize| {
                p.bytes
                    .checked_mul(workers.saturating_sub(1))
                    .and_then(|bytes| {
                        stack
                            .checked_add(adapter_stacks)
                            .and_then(|per_worker| workers.checked_mul(per_worker))
                            .and_then(|stacks| bytes.checked_add(stacks))
                    })
                    .and_then(|bytes| {
                        bytes.checked_add(workers.checked_mul(foreign::chain_metadata_bytes()?)?)
                    })
                    .ok_or_else(|| {
                        FitWithheld::Local(Withheld::Analysis(native::kkt::Unavailable::from(
                            ProblemError::memory("profile team extent"),
                        )))
                    })
            };
            let constructor = pse_math::construction::current();
            // Profile chains run on owner-affine standard threads. Nested experiment
            // teams must inherit this actual executor, just as they inherit the native
            // environment, construction owner and already admitted capacity envelope.
            let runtime = tokio::runtime::Handle::try_current().map_err(|error| {
                FitWithheld::Local(Withheld::Analysis(native::kkt::Unavailable::from(
                    ProblemError::internal(format!("fit profile runtime: {error}")),
                )))
            })?;
            let mut actual = workers;
            let (mut result, started, scheduling) = loop {
                execution.check().map_err(|error| {
                    FitWithheld::Local(Withheld::Analysis(native::kkt::Unavailable::from(error)))
                })?;
                let attempted = math.quiescent_phase(
                    core_extent(actual)?,
                    byte_extent(actual)?,
                    execution,
                    || {
                        let Some(chain_executions) = foreign::chains(p, math, execution, actual)?
                        else {
                            return Ok(None);
                        };
                        Ok::<_, ProblemError>(Some(dispatch(
                            tasks.len(),
                            actual,
                            stack,
                            |worker, index| {
                                let current = Chains {
                                    execution: &chain_executions[worker],
                                    ..chains
                                };
                                current.run(tasks[index])
                            },
                            |index, failure| chains.failed(tasks[index], failure),
                            |work| {
                                let _runtime = runtime.enter();
                                native::execution::scoped(
                                    &adapters,
                                    threads,
                                    stack,
                                    || -> Result<(), ProblemError> {
                                        experiments::borrowed_capacity(chain_cores, || {
                                            match &constructor {
                                                Some(owner) => pse_math::construction::scoped(
                                                    owner.clone(),
                                                    work,
                                                ),
                                                None => work(),
                                            }
                                        });
                                        Ok(())
                                    },
                                )
                                .map_err(|error| {
                                    ProfileWorkerFailure::Environment(ProfileFailure(Arc::new(
                                        error,
                                    )))
                                })
                            },
                            |_| Ok(()),
                        )))
                    },
                );
                match attempted {
                    Ok(Ok(Some(result))) => break result,
                    Ok(Ok(None)) => {}
                    Ok(Err(error)) => {
                        return Err(FitWithheld::Local(Withheld::Analysis(
                            native::kkt::Unavailable::from(error),
                        )));
                    }
                    Err(error) if actual > 1 && foreign::capacity(&error) => {}
                    Err(error) => {
                        return Err(FitWithheld::Local(Withheld::Analysis(
                            native::kkt::Unavailable::from(error.into_problem()),
                        )));
                    }
                }
                if actual == 1 {
                    return Err(FitWithheld::Local(Withheld::Analysis(
                        native::kkt::Unavailable::from(ProblemError::memory(
                            "serial profile foreign coverage unavailable",
                        )),
                    )));
                }
                actual -= 1;
            };
            for chain in &mut result {
                chain.actual_parallelism = started;
                chain.scheduling_failures = scheduling.clone();
            }
            Ok(result)
        })
    }
}

#[cfg(test)]
mod worker_tests {
    use super::*;
    #[test]
    fn interval_product_requires_complete_unique_finite_ends_without_worker_failure() {
        let first = SemanticId::from_bytes([1; 16]);
        let second = SemanticId::from_bytes([2; 16]);
        let endpoint = |parameter, end| ProfileChain {
            parameter,
            end,
            estimate: 0.0,
            bound: IntervalBound {
                value: Some(if end == IntervalEnd::Lower { -1.0 } else { 1.0 }),
                outcome: if end == IntervalEnd::Lower {
                    IntervalOutcome::Bound
                } else {
                    IntervalOutcome::Threshold
                },
            },
            qualifiers: Vec::new(),
            worker_failure: None,
            scheduling_failures: Vec::new(),
            actual_parallelism: 1,
            detail: None,
            points: Vec::new(),
        };
        let complete = [first, second]
            .into_iter()
            .flat_map(|parameter| {
                [IntervalEnd::Lower, IntervalEnd::Upper].map(|end| endpoint(parameter, end))
            })
            .collect::<Vec<_>>();
        assert!(interval_validity([first, second], &complete).is_ok());
        assert!(interval_validity([first, second], &complete[..3]).is_err());
        assert!(interval_validity([], &[]).is_err());
        for value in [
            None,
            Some(f64::INFINITY),
            Some(f64::NEG_INFINITY),
            Some(f64::NAN),
        ] {
            let mut invalid = complete.clone();
            invalid[0].bound.value = value;
            assert!(interval_validity([first, second], &invalid).is_err());
        }
        let mut stopped = complete.clone();
        stopped[0].bound.outcome = IntervalOutcome::Stopped;
        assert!(interval_validity([first, second], &stopped).is_err());
        let mut duplicate = complete.clone();
        duplicate[3] = duplicate[2].clone();
        assert!(interval_validity([first, second], &duplicate).is_err());
        let mut unexpected = complete.clone();
        unexpected[3].parameter = SemanticId::NIL;
        assert!(interval_validity([first, second], &unexpected).is_err());
        for failure in [
            ProfileWorkerFailure::Panic,
            ProfileWorkerFailure::Scheduling("refused".into()),
        ] {
            let mut failed = complete.clone();
            failed[0].worker_failure = Some(failure);
            assert!(interval_validity([first, second], &failed).is_err());
        }
        let mut recovered_scheduling = complete;
        recovered_scheduling[0]
            .scheduling_failures
            .push(ProfileWorkerFailure::Scheduling(
                "one worker refused; another completed its chains".into(),
            ));
        assert!(interval_validity([first, second], &recovered_scheduling).is_ok());
    }

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
            pse_diagnostics::Cancellation::new(false),
            &Default::default(),
        )
    }
    fn solution(value: f64) -> Solution {
        Solution {
            residual: None,
            objective: value * value / 2.0,
            primal: vec![value],
            qualification: Qualification::Stationary,
            termination: Termination::Success,
            qualifiers: Vec::new(),
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
    fn check(
        kind: pse_model::generated::enums::ModelingCheckKind,
        satisfied: bool,
    ) -> crate::workflow::ModelingCheck {
        crate::workflow::ModelingCheck {
            run_id: RunId::from_bytes([1; 16]),
            step: 0,
            sample_index: 0,
            time: None,
            target_id: SemanticId::from_bytes([4; 16]),
            source_id: pse_modeling::DeclarationId::from_bytes([3; 16]),
            kind,
            value: 0.0,
            tolerance: None,
            satisfied,
            within_validity: None,
            extrapolation_allowed: None,
            basis: pse_model::generated::enums::ModelingCheckBasis::Point,
            layer: None,
            claim_id: None,
            claim_owner: None,
            claim_owner_lineage: Vec::new(),
            coverage_id: None,
            evidence_id: None,
            form_id: None,
            call_id: None,
            selected_records: Vec::new(),
            dependencies: Vec::new(),
            input_values: Vec::new(),
            applicability_outcome: None,
            applicability_basis: None,
            permission_ids: Vec::new(),
            unknown_allowed: None,
            observation_instance: None,
            applicability_required: None,
            applicability_reason: None,
            applicability_permissions: Vec::new(),
        }
    }
    #[test]
    fn pure_pin_policy_requires_original_checks_and_preserves_interval_qualifiers() {
        use pse_model::generated::enums::{
            ModelingApplicabilityOutcome as A, ModelingCheckKind as K,
        };
        let quality = native::quality::Quality::new(vec![], vec![], vec![]).unwrap();
        let evidence = NativeEvidence {
            candidate: Some(&[0.]),
            termination: Termination::Success,
            quality: Some(&quality),
            callback_failure: None,
            validation_failure: None,
            qualification: Qualification::Stationary,
            bound: None,
            relaxed_incumbent: false,
        };
        let policy = pse_model::numerics::NumericalPolicy::default();
        let mut applicability = check(K::Applicability, true);
        applicability.applicability_outcome = Some(A::UnknownEvidence);
        applicability.unknown_allowed = Some(true);
        applicability.applicability_required = Some(true);
        let checks = [check(K::Closure, true), applicability];
        let original = CompletionEvidence::point(&checks, true, 1);
        let decision = pin_acceptance(evidence, original, &policy);
        assert!(decision.permits_use());
        assert_eq!(
            decision.qualifiers,
            vec![CandidateQualifier::ApplicabilityUnknownAllowed]
        );
        let execution = execution();
        let chain = self::policy(&execution).run(|value, _| {
            let mut point = solution(value);
            point.qualifiers = decision.qualifiers.clone();
            Ok(point)
        });
        assert_eq!(chain.bound.outcome, IntervalOutcome::Threshold);
        assert!(chain.bound.value.is_some());
        assert_eq!(chain.qualifiers, decision.qualifiers);
        assert!(
            chain
                .points
                .iter()
                .all(|point| point.qualifiers == chain.qualifiers)
        );
        for original in [
            CompletionEvidence::point(&checks, false, 1),
            CompletionEvidence::point(&checks, true, 2),
        ] {
            assert!(!pin_acceptance(evidence, original, &policy).permits_use());
        }
        let mut denied = checks.clone();
        denied[1].satisfied = false;
        assert!(
            !pin_acceptance(
                evidence,
                CompletionEvidence::point(&denied, true, 1),
                &policy
            )
            .permits_use()
        );
        let mut extrapolated = checks.clone();
        extrapolated[1].applicability_outcome = Some(A::OutsideRegion);
        extrapolated[1].extrapolation_allowed = Some(true);
        let decision = pin_acceptance(
            evidence,
            CompletionEvidence::point(&extrapolated, true, 1),
            &policy,
        );
        assert!(decision.permits_use());
        assert!(
            decision
                .qualifiers
                .contains(&CandidateQualifier::ApplicabilityExtrapolationAllowed)
        );
        let limited = NativeEvidence {
            termination: Termination::IterationLimit,
            qualification: Qualification::Feasible,
            ..evidence
        };
        let policy = pse_model::numerics::NumericalPolicy {
            incumbent: pse_model::generated::enums::IncumbentPolicy::AcceptFeasible,
            ..policy
        };
        assert!(crate::workflow::numerics::native_evidence_use(limited, &policy).permits_use());
        let pin = pin_acceptance(limited, original, &policy);
        assert!(!pin.permits_use());
        assert!(pin.refusals.contains(&CandidateRefusal::Unqualified));
        assert!(pin.refusals.contains(&CandidateRefusal::NativeOutcome));
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
                occurrences: Default::default(),
                checked_sources: Default::default(),
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
                occurrences: Default::default(),
                checked_sources: Default::default(),
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
                residual: None,
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
            PinFailure::assess(
                latched.take().expect("terminal report must never repeat"),
                CompletionEvidence::point(&[], true, 0),
                &Default::default(),
            )
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
                PinFailure::assess(
                    observed.take().expect("native stop must never repeat"),
                    CompletionEvidence::point(&[], true, 0),
                    &Default::default(),
                )
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
        let failed = PinFailure::assess(
            observed,
            CompletionEvidence::point(&[], true, 0),
            &Default::default(),
        )
        .err()
        .unwrap();
        assert!(!failed.subdivides());
        assert!(Arc::ptr_eq(&original, &failed.failures[0].0));
    }
    #[test]
    fn stopped_scope_never_starts_a_scripted_pin() {
        let execution = execution();
        execution.cancellation_control().cancel();
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
            |_, _| {
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
                |_, index| {
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
            qualifiers: Vec::new(),
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
            |_, _| {
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

    #[test]
    fn observation_profile_dispatch_transports_context_and_counts_actual_started_workers() {
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("profile workers");
        let context = operation.context();
        let _entered = context.enter();
        let (results, started, failures) = dispatch(
            4,
            2,
            2 * 1024 * 1024,
            |_, index| {
                assert_eq!(
                    pse_diagnostics::observation::current().unwrap().id(),
                    context.id()
                );
                index
            },
            |_, _| usize::MAX,
            |run| {
                run();
                Ok(())
            },
            |index| {
                if index == 0 {
                    Err("injected unavailable worker".into())
                } else {
                    Ok(())
                }
            },
        );
        assert_eq!(results, vec![0, 1, 2, 3]);
        assert_eq!(started, 1);
        assert_eq!(failures.len(), 1);
        assert_eq!(
            pse_diagnostics::observation::current().unwrap().id(),
            context.id()
        );
        let snapshot = context
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        let count = |name: &str| {
            snapshot
                .metrics
                .iter()
                .find(|metric| metric.name == name)
                .map(|metric| metric.cumulative.clone())
        };
        assert_eq!(count("profile.workers_started"), Some(Value::Count(1)));
        assert_eq!(count("profile.worker_failures"), Some(Value::Count(1)));
    }

    #[test]
    fn observation_stopped_profile_point_does_not_invent_an_attempted_pin() {
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("stopped profile chain");
        let context = operation.context();
        let _entered = context.enter();
        let execution = execution();
        execution.cancellation_control().cancel();
        let chain = policy(&execution).run(|_, _| panic!("cancelled chain called a pin"));
        assert_eq!(chain.bound.outcome, IntervalOutcome::Stopped);
        let parent = context
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        assert_eq!(parent.children.len(), 1);
        let snapshot = hub
            .reader(parent.children[0].id)
            .unwrap()
            .snapshot(std::time::Duration::from_secs(60));
        let metric = |name: &str| snapshot.metrics.iter().find(|metric| metric.name == name);
        assert!(metric("profile.pinned_fit_attempts").is_none());
        assert_eq!(
            metric("profile.failed_points").unwrap().cumulative,
            Value::Count(1)
        );
        assert_eq!(
            metric("profile.interval_available").unwrap().latest,
            Value::Boolean(false)
        );
        assert_eq!(
            metric("profile.failure_subdivides").unwrap().latest,
            Value::Boolean(false)
        );
    }
}

#[cfg(all(test, feature = "solver-ipopt"))]
#[tokio::test]
async fn residual_profile_pin_preserves_literal_callback_failure_before_independent_assessment() {
    let runtime = crate::workflow::tests::runtime();
    let services = runtime.execution_services();
    let seed = oracle::lm_inquiry::fixture(&runtime, "domain", [1., 2.])
        .await
        .unwrap();
    let mut profile = seed.problem.profile.clone();
    profile.optimizer = FitOptimizer::ResidualLm;
    profile.solver.selection = native::solve::SolverSelection::Auto;
    profile.solver.controls.hessian = HessianMode::GaussNewton;
    let prepared = seed
        .source
        .prepare_fit(
            services.preparation(),
            seed.declaration().fit_id,
            profile,
            crate::workflow::tests::compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let problem = prepared.problem.clone();
    let math = services.preparation().math().clone();
    let service = math.clone();
    service.job_scoped(1,problem.bytes+(1<<20),Default::default(),None,move |flag| {
        let mut execution=Execution::new(flag,&problem.profile.solver.controls);
        execution.memory=Some(1<<20);
        let estimate=problem.initial.clone();
        let chains=Chains { problem:&problem,math:&math,prepared:&prepared,
            run_id:RunId::from_bytes([71;16]),qualifiers:&[],route:FitRoute::ResidualLm,execution:&execution,
            estimate:&estimate,optimum:0.,threshold:1.,tolerance:0.001,points:4 };
        let Err(failed)=chains.solve(&mut native::execution::Retained::default(),OriginalCol::new(0),-1.,
            &estimate,oracle::neutral::FitScratch::new(&problem,&math)) else {panic!("invalid current pin unexpectedly succeeded");};
        assert!(failed.callback_terminal_failure);
        assert!(!failed.subdivides(),"a latched callback owner cannot masquerade as an unlatched numerical trial");
        let literal=failed.residual.as_ref().unwrap();
        assert!(matches!(literal.termination,FitResidualTermination::User {..}));
        let source=literal.failure.as_ref().unwrap();
        assert!(failed.failures.iter().any(|failure|Arc::ptr_eq(&failure.0,&source.0)),
            "the original typed source cause survives instead of a second assessment failure");
        assert_eq!(literal.stationarity,None);
        assert_eq!(literal.jacobian_evaluations,0,"the invalid current Value never reaches derivative/final assessment");
        Ok(())
    }).await.unwrap();
}
