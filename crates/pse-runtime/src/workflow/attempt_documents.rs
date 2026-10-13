// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded projections of retained producer evidence. No evaluation or native call occurs here.
use super::{RunReport, RunRequest, RunResult, WorkflowError, contract};
use crate::math::strategy::{Event, OriginalConclusion, Trace};
use pse_ids::{ContentHash, SemanticId};
use pse_model::{diagnostic::{BoundaryClass, DiagnosticCode}, generated::{enums::{CandidateUse,
    NumericalAttemptObservation as Observation, NumericalEventKind}, identities::RunId},
    strategy::{Phase, Transition, WorkCharge, WorkObservation}};
use std::sync::Arc;
use pse_model::diagnostic::Observation as NumberObservation;

/// Actual submitted-start provenance. Full portable coordinates belong to replay.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttemptStartSummary {
    /// Prior attempt selected by the original start owner.
    pub previous_attempt: Option<usize>,
    /// Actual compatible source layout, when a complete seed was supplied.
    pub layout: Option<ContentHash>,
    /// Actual seed numeric data identity, distinct from native session compatibility.
    pub data: Option<ContentHash>,
    /// Transformations actually traversed before native submission.
    pub transformations: Vec<pse_backend_native::solve::SeedTransformation>,
    /// Number of explicitly supplied sparse start variables.
    pub sparse_variables: usize,
    /// Whether the adapter submitted a seed to the native API.
    pub submitted: bool,
}

/// Disjoint phase observations are absent until their actual owner measures them.
#[derive(Clone, Debug, Default, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttemptIntervals {
    /// Preparation wall seconds; unavailable at this reporting boundary.
    pub preparation_seconds: Option<NumberObservation>,
    /// Admission queue wall seconds; unavailable at this reporting boundary.
    pub queue_seconds: Option<NumberObservation>,
    /// Native algorithm wall seconds, including callbacks; overlaps callback work.
    pub native_including_callbacks_seconds: Option<NumberObservation>,
    /// Independent original assessment wall seconds; unavailable unless separately observed.
    pub assessment_seconds: Option<NumberObservation>,
    /// Publication wall seconds; unavailable at this reporting boundary.
    pub publication_seconds: Option<NumberObservation>,
}

/// Admission bounds for automatic retained-only explanation.
#[derive(Clone, Copy, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttemptSummaryLimits {
    /// Retained bounded original steps.
    pub steps: usize,
    /// Maximum retained decisions across all steps.
    pub events: usize,
    /// Retained original-coordinate violations.
    pub findings: usize,
}
impl Default for AttemptSummaryLimits {
    fn default() -> Self { Self { steps: 16, events: 64, findings: 32 } }
}
impl AttemptSummaryLimits {
    pub(super) fn validate(self) -> Result<Self, WorkflowError> {
        if self.steps > 1024 || self.events > 4096 || self.findings > 4096 {
            return Err(contract("attempt summary exceeds bounded explanation capacity"));
        }
        Ok(self)
    }
}
/// Independent original acceptance; never inferred from a native success code.
#[derive(Clone, Copy, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AttemptOriginalConclusion {
    /// Original owner established its requested scientific conclusion.
    Satisfied,
    /// Original owner refused the candidate.
    Refused,
    /// Original assessment could not establish a conclusion.
    Unavailable,
}
/// Compact synchronous decision record. Charging identity preserves inclusive work meaning.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttemptDecision {
    /// Declared mechanism ordinal.
    pub mechanism: usize,
    /// Actual decision event kind.
    pub kind: NumericalEventKind,
    /// Actual executing or assessing phase.
    pub phase: Phase,
    /// Recorded governing decision identity.
    pub decision: Option<ContentHash>,
    /// Typed native or boundary observation.
    pub observation: Option<Observation>,
    /// Recorded governing transition.
    pub transition: Option<Transition>,
    /// Original candidate-use permission.
    pub permission: Option<CandidateUse>,
    /// Independent original scientific conclusion.
    pub original: Option<AttemptOriginalConclusion>,
    /// Actual owner-issued work; unknown counters remain absent.
    pub work: Option<WorkCharge>,
}
impl From<&Event> for AttemptDecision {
    fn from(event: &Event) -> Self {
        Self { mechanism: event.mechanism, kind: event.kind, phase: event.phase,
            decision: event.decision, observation: event.observation, transition: event.transition,
            permission: event.permission, work: event.work,
            original: event.original.as_ref().map(|original| match original {
                OriginalConclusion::Satisfied => AttemptOriginalConclusion::Satisfied,
                OriginalConclusion::Refused { .. } => AttemptOriginalConclusion::Refused,
                OriginalConclusion::Unavailable { .. } => AttemptOriginalConclusion::Unavailable,
            }) }
    }
}
/// A retained original-coordinate violation. IDs and physical tolerance belong to the producer.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttemptFinding {
    /// Original authored row or variable identity.
    pub source: SemanticId,
    /// Physical violation in original declared units.
    pub violation: NumberObservation,
    /// Explicit tolerance in the same original units.
    pub tolerance: NumberObservation,
}
/// Compact failure explanation over final accepted evidence only.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttemptFailureSummary {
    /// Typed native or boundary observation.
    pub observation: Observation,
    /// Detailed source-owned diagnostic code.
    pub code: DiagnosticCode,
    /// Source-owned boundary disposition.
    pub class: BoundaryClass,
    /// No diagnostic model/native work was admitted or performed by this projection.
    pub retained_only: bool,
}
/// One original requested step; missing native facts remain absent.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttemptStepSummary {
    /// Original zero-based step ordinal.
    pub step: usize,
    /// Actual executed native backend, when available.
    pub backend: Option<pse_backend_native::solve::Backend>,
    /// Typed native or boundary observation.
    pub observation: Option<Observation>,
    /// Independent original permission, when assessed.
    pub accepted: Option<bool>,
    /// Actual owner-issued work; unknown counters remain absent.
    pub work: Option<WorkObservation>,
    /// Whether the native owner submitted a start.
    pub start_submitted: Option<bool>,
    /// Whether actual native state was reused.
    pub reused_native_state: Option<bool>,
    /// Maximum original physical violation divided by its declared tolerance.
    pub normalized_physical_violation: Option<NumberObservation>,
    /// Actual options submitted by the adapter, separate from queried native defaults.
    pub effective_options: pse_backend_native::solve::Options,
    /// Defaults only when the native library supplies a readback API.
    pub native_defaults: pse_backend_native::solve::Options,
    /// Actual submitted seed transformations and source identity.
    pub start: Option<AttemptStartSummary>,
    /// Available interval observations; overlapping native/callback costs are never added.
    pub intervals: AttemptIntervals,
    /// Retained synchronous decision prefix.
    pub decisions: Vec<AttemptDecision>,
    /// Decisions outside the requested prefix.
    pub omitted_decisions: usize,
    /// Retained original-coordinate violations.
    pub findings: Vec<AttemptFinding>,
    /// Original violations outside the requested bound.
    pub omitted_findings: usize,
    /// Actual lossy progress count, independent of synchronous decisions.
    pub dropped_progress: Option<u64>,
    /// Actual family-specific counters, phase and cooperative stop coverage.
    pub native_observations: Option<Arc<pse_backend_native::solve::NativeObservations>>,
    /// Bounded raw physical map samples, independently of progress loss.
    pub raw_map: Option<pse_backend_native::solve::RawMapHistory>,
}
impl AttemptStepSummary {
    fn empty(step: usize) -> Self {
        Self { step, backend: None, observation: None, accepted: None, work: None,
            start_submitted: None, reused_native_state: None, normalized_physical_violation: None,
            effective_options: Default::default(), native_defaults: Default::default(), start: None,
            intervals: Default::default(),
            decisions: Vec::new(), omitted_decisions: 0, findings: Vec::new(),
            omitted_findings: 0, dropped_progress: None, native_observations: None, raw_map: None }
    }
    fn trace(&mut self, trace: &Trace, remaining: &mut usize) {
        let count = trace.events.len().min(*remaining);
        self.decisions.extend(trace.events.iter().take(count).map(AttemptDecision::from));
        self.omitted_decisions = trace.events.len() - count;
        *remaining -= count;
    }
    fn native(&mut self, native: &pse_backend_native::solve::SolveReport, remaining: &mut usize) {
        self.backend = Some(native.backend);
        self.observation = Some(crate::math::strategy::observe_native(native));
        self.work = Some(crate::math::strategy::work_native(native));
        self.start_submitted = Some(native.evidence.start_submitted);
        self.reused_native_state = Some(native.evidence.reused_native_state);
        self.dropped_progress = Some(native.dropped_events);
        self.native_observations = native.evidence.native_observations.clone();
        self.effective_options = native.options.clone();
        self.native_defaults = native.native_defaults.clone();
        self.start = native.start_receipt.as_ref().map(|start| AttemptStartSummary {
            previous_attempt: start.previous_attempt,
            layout: start.seed.as_ref().map(|seed| seed.compatibility.layout),
            data: start.seed.as_ref().map(|seed| seed.compatibility.data),
            transformations: start.transformations.clone(),
            sparse_variables: start.sparse_seed.as_ref().map_or(0, |seed| seed.len()),
            submitted: start.submitted,
        });
        if let Some(observations) = &native.evidence.native_observations
            && let pse_backend_native::solve::NativeCounters::Ipopt { wall_seconds, .. } = &observations.counters {
            self.intervals.native_including_callbacks_seconds = wall_seconds.map(NumberObservation::number);
        }
        if let Some(raw) = &native.evidence.raw_map {
            let count = raw.samples.len().min(*remaining);
            self.raw_map = Some(pse_backend_native::solve::RawMapHistory {
                problem: raw.problem, profile: raw.profile,
                samples: raw.samples.iter().take(count).cloned().collect(),
                omitted: raw.omitted.saturating_add((raw.samples.len() - count) as u64),
                last_successful: raw.last_successful.clone(), last_evaluation: raw.last_evaluation.clone(),
            });
            *remaining -= count;
        }
        if let Some(quality) = &native.quality {
            self.normalized_physical_violation = Some(NumberObservation::number(quality.normalized_max));
            let findings = quality.rows.iter().chain(&quality.bounds).chain(&quality.integrality)
                .filter(|finding| finding.physical > finding.tolerance);
            for finding in findings {
                if *remaining > 0 {
                    self.findings.push(AttemptFinding { source: finding.id, violation: NumberObservation::number(finding.physical),
                        tolerance: NumberObservation::number(finding.tolerance) });
                    *remaining -= 1;
                } else { self.omitted_findings += 1; }
            }
        }
    }
}
/// Bounded automatic summary. Partial evidence and truncation are explicit.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttemptSummary {
    /// Original execution identity.
    pub run_id: RunId,
    /// Full original requested step count.
    pub requested_steps: usize,
    /// Retained bounded original steps.
    pub steps: Vec<AttemptStepSummary>,
    /// Steps omitted by the requested bound.
    pub omitted_steps: usize,
    /// Actual terminal failure when retained.
    pub failure: Option<AttemptFailureSummary>,
    /// These are original recorded reports; a partial trajectory is never a full horizon.
    pub usable: bool,
    #[serde(skip)]
    #[schemars(skip)]
    _owner: Arc<pse_columnar::AllocationLease>,
}
fn native_copy_extent(native: &pse_backend_native::solve::SolveReport) -> Option<usize> {
    let mut extent = 0usize;
    for options in [&native.options, &native.native_defaults] {
        // Bound every occupied BTree node plus owned option key/value strings.
        extent = extent.checked_add(options.len().checked_mul(2048)?)?;
        for (key, value) in options {
            extent = extent.checked_add(key.len())?;
            if let pse_backend_native::solve::OptionValue::Text(value) = value {
                extent = extent.checked_add(value.len())?;
            }
        }
    }
    if let Some(start) = &native.start_receipt {
        extent = extent.checked_add(start.transformations.len().checked_mul(size_of::<pse_backend_native::solve::SeedTransformation>())?)?;
        for transform in &start.transformations {
            if let pse_backend_native::solve::SeedTransformation::Presolve { passes, .. } = transform {
                extent = extent.checked_add(passes.len().checked_mul(size_of::<pse_backend_native::presolve::Pass>())?)?;
            }
        }
    }
    Some(extent)
}
impl RunResult {
    /// Automatic compact retained-only diagnosis, including after cancellation/deadline.
    pub fn attempt_summary(&self) -> Result<Arc<AttemptSummary>, WorkflowError> {
        if let Some(summary) = self.attempt_summary.get() { return Ok(summary.clone()); }
        let summary = Arc::new(self.attempt_summary_with_limits(AttemptSummaryLimits::default())?);
        let _ = self.attempt_summary.set(summary.clone());
        Ok(self.attempt_summary.get().cloned().unwrap_or(summary))
    }
    /// Explicit bounded detail. This never evaluates a model, prepares a solver or queries history.
    pub fn attempt_summary_with_limits(&self, limits: AttemptSummaryLimits) -> Result<AttemptSummary, WorkflowError> {
        let limits = limits.validate()?;
        let mut extent = limits.steps.saturating_mul(size_of::<AttemptStepSummary>() + 16384)
            .saturating_add(limits.events.saturating_mul(size_of::<AttemptDecision>() + 64))
            .saturating_add(limits.findings.saturating_mul(size_of::<AttemptFinding>() + 4096))
            .saturating_add(size_of::<AttemptSummary>() + 512);
        let requested_steps = match &self.request { RunRequest::Modeling(steps) => steps.len(), _ => 1 };
        // Admit copies of actual variable-size option/start payloads before cloning them.
        for step in 0..requested_steps.min(limits.steps) {
            let native = self.modeling_result(step).and_then(|result| match &result.outcome {
                crate::math::solves::Outcome::Native(native) => Some(native.as_ref()), _ => None,
            }).or_else(|| match &self.report {
                Ok(RunReport::Fit(report)) => report.solve.as_ref(),
                #[cfg(feature = "solver-diffsol")]
                Ok(RunReport::Shooting(report)) => report.solve.as_ref(),
                _ => None,
            });
            if let Some(native) = native {
                extent = native_copy_extent(native).and_then(|copied| extent.checked_add(copied))
                    .ok_or_else(|| contract("attempt summary option/start extent overflow"))?;
            }
        }
        let owner = self.runtime.shared.math().reserve("attempt:retained-summary", extent)?;
        let mut summary = AttemptSummary { run_id: self.run_id, requested_steps, steps: Vec::new(),
            omitted_steps: requested_steps.saturating_sub(limits.steps), failure: None,
            usable: self.usable(), _owner: owner };
        let mut events = limits.events;
        let mut findings = limits.findings;
        for step in 0..requested_steps.min(limits.steps) {
            let mut projected = AttemptStepSummary::empty(step);
            if let Some(result) = self.modeling_result(step) {
                projected.accepted = Some(result.accepted);
                projected.observation = Some(crate::math::strategy::observe(&result.outcome));
                projected.work = Some(crate::math::strategy::work(&result.outcome));
                if let crate::math::solves::Outcome::Native(native) = &result.outcome {
                    projected.native(native, &mut findings);
                }
                if let Some(trace) = &result.strategy { projected.trace(trace, &mut events); }
            } else { match &self.report {
                Ok(RunReport::Fit(report)) => {
                    projected.accepted = Some(self.usable());
                    if let Some(native) = &report.solve { projected.native(native, &mut findings); }
                    if let Some(trace) = &report.strategy { projected.trace(trace, &mut events); }
                }
                #[cfg(feature = "solver-diffsol")]
                Ok(RunReport::Shooting(report)) => {
                    projected.accepted = Some(self.usable());
                    if let Some(native) = &report.solve { projected.native(native, &mut findings); }
                    if let Some(trace) = &report.strategy { projected.trace(trace, &mut events); }
                }
                Ok(RunReport::Simulation(_)) | Ok(RunReport::Horizon(_)) => {
                    projected.accepted = Some(self.usable());
                }
                Err(error) => {
                    projected.observation = Some(crate::math::strategy::workflow_failure(error));
                    if let Some(trace) = error.strategy_trace() { projected.trace(trace, &mut events); }
                }
                _ => {}
            }}
            summary.steps.push(projected);
        }
        if let Err(error) = &self.report {
            let diagnostic = error.boundary_diagnostic();
            summary.failure = Some(AttemptFailureSummary {
                observation: crate::math::strategy::workflow_failure(error),
                code: diagnostic.code, class: diagnostic.class, retained_only: true });
        }
        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_summary_is_one_shared_product_and_custom_bounds_are_explicit() {
        let result = RunResult::joined(RunId::from_bytes([91; 16]), crate::workflow::tests::runtime(),
            RunRequest::Modeling(vec![]), None, Ok(RunReport::Modeling(vec![]))).completed();
        let first = result.attempt_summary().unwrap();
        let second = result.attempt_summary().unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(first.requested_steps, 0);
        assert!(result.attempt_summary_with_limits(AttemptSummaryLimits {
            steps: 1, events: usize::MAX, findings: 0,
        }).is_err());
    }
    #[test]
    fn retained_nonfinite_physical_violation_is_observed_not_missing() {
        use pse_backend_native::solve::{Assurance, Backend, Controls, Execution, NativeTermination, SolveReport, Termination};
        let contract = pse_backend_native::OracleContract {
            identity: ContentHash::from_bytes([92; 32]), variables: vec![], rows: vec![],
            derivatives: pse_kernels::DerivativeOrder::Value, smoothness: pse_kernels::DerivativeOrder::Value,
        };
        let execution = Execution::new(Arc::new(std::sync::atomic::AtomicBool::new(false)), &Controls::default());
        let mut report = SolveReport::new(Backend::Kinsol, &contract, NativeTermination {
            code: -1, name: "retained_failure".into(), message: None,
            category: Termination::Numerical, assurance: Assurance::None,
        }, &execution);
        report.quality = Some(pse_backend_native::quality::Quality {
            rows: vec![pse_backend_native::quality::Violation { id: SemanticId::NIL,
                physical: f64::INFINITY, tolerance: 1.0 }], bounds: vec![], integrality: vec![], normalized_max: f64::INFINITY,
        });
        let mut step = AttemptStepSummary::empty(0);
        step.native(&report, &mut 1);
        let json = serde_json::to_value(step).unwrap();
        assert_eq!(json["normalized_physical_violation"]["kind"], "nonfinite");
        assert_eq!(json["findings"][0]["violation"]["value"], "positive_infinity");
    }
    #[test]
    fn explanation_admission_is_finite_and_zero_is_explicit() {
        assert!(AttemptSummaryLimits { steps: 0, events: 0, findings: 0 }.validate().is_ok());
        assert!(AttemptSummaryLimits { steps: 1, events: usize::MAX, findings: 0 }.validate().is_err());
    }
    #[test]
    fn decision_projection_preserves_original_refusal_despite_native_success() {
        let event = Event { mechanism: 2, kind: NumericalEventKind::Finished, phase: Phase::Assessment,
            original: Some(OriginalConclusion::Unavailable { cause: Arc::new(pse_backend_native::ProblemError::Cancelled) }),
            decision: None, observation: Some(Observation::Converged), transition: Some(Transition::Stop),
            permission: None, work: None, cause: None };
        let projected = AttemptDecision::from(&event);
        assert!(matches!(projected.original, Some(AttemptOriginalConclusion::Unavailable)));
        assert_eq!(projected.observation, Some(Observation::Converged));
        assert_eq!(projected.transition, Some(Transition::Stop));
    }
}
