// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Synchronous native facts; telemetry retention never supplies decision evidence.
use super::{Backend, ContentHash, SemanticId, Termination};
use std::sync::Arc;
fn serialize_number<S: serde::Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    serde::Serialize::serialize(
        &pse_model::diagnostic::Observation::number(*value),
        serializer,
    )
}
fn serialize_optional_number<S: serde::Serializer>(
    value: &Option<f64>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serde::Serialize::serialize(
        &value.map(pse_model::diagnostic::Observation::number),
        serializer,
    )
}

/// Checkpoint coverage of the actual adapter, not a promise of native preemption.
#[derive(Clone, Copy, Debug, serde::Serialize, schemars::JsonSchema)]
pub enum StopFidelity {
    /// Evaluations and native iteration callbacks poll the original execution scope.
    EvaluationAndIterationCheckpoints,
    /// Evaluations poll; an indivisible native linear operation must finish first.
    EvaluationCheckpoints,
}
/// Coordinate authority of a captured point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub enum ObservationCoordinates {
    /// Coordinates submitted to the oracle; an outer formulation may still transform them.
    SubmittedOracle,
    /// The causal producer explicitly supplies physical tear destination quantities.
    PhysicalTears,
}
/// Native phase, without inventing internal restoration variables or slacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub enum NativePhase {
    /// Ordinary native optimization.
    Main,
    /// Native restoration phase.
    Restoration,
    /// The callback did not establish the phase.
    Unknown,
}
/// One submitted-oracle coordinate with lossless numeric observation encoding.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NativeCoordinate {
    /// Submitted variable identity.
    pub variable: SemanticId,
    /// Observed IEEE value; nonfinite values retain their explicit tag.
    #[serde(serialize_with = "serialize_number")]
    #[schemars(with = "pse_model::diagnostic::Observation")]
    pub value: f64,
}
/// Latest callback point in its submitted coordinate authority, not a resumable state.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NativeSnapshot {
    /// Actual native iteration ordinal.
    pub iteration: Option<u64>,
    /// Actual callback phase.
    pub phase: NativePhase,
    /// Captured coordinate prefix, with source IDs.
    pub values: Vec<NativeCoordinate>,
    /// Full submitted coordinate count, including any omitted suffix.
    pub total_coordinates: usize,
    /// A suffix was omitted by the fixed capture allocation.
    pub partial: bool,
}
/// Family-specific native meanings; these counters are not interchangeable cost units.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum NativeCounters {
    /// Ipopt's structured SolveStatistics, absent after unsupported/serious exits.
    Ipopt {
        /// Native iteration count.
        iterations: Option<u64>,
        /// Native objective evaluations, distinct from adapter-inclusive callback work.
        objective_evaluations: Option<u64>,
        /// Native constraint evaluations.
        constraint_evaluations: Option<u64>,
        /// Native objective gradient evaluations.
        gradient_evaluations: Option<u64>,
        /// Native constraint Jacobian evaluations.
        jacobian_evaluations: Option<u64>,
        /// Native Lagrangian Hessian evaluations.
        hessian_evaluations: Option<u64>,
        /// CPU time including callbacks; overlaps wall and system time.
        #[serde(serialize_with = "serialize_optional_number")]
        #[schemars(with = "Option<pse_model::diagnostic::Observation>")]
        cpu_seconds: Option<f64>,
        /// System time including callbacks; never summed with wall time.
        #[serde(serialize_with = "serialize_optional_number")]
        #[schemars(with = "Option<pse_model::diagnostic::Observation>")]
        system_seconds: Option<f64>,
        /// Native algorithm wall time including callbacks.
        #[serde(serialize_with = "serialize_optional_number")]
        #[schemars(with = "Option<pse_model::diagnostic::Observation>")]
        wall_seconds: Option<f64>,
    },
    /// KINSOL getters for this attempt, excluding independent original assessment.
    Kinsol {
        /// Fixed-point mode increments before G, so a failed map counts an attempted iteration.
        fixed_point_iterations: bool,
        /// Native nonlinear iteration count; not accepted outer-state count.
        iterations: Option<u64>,
        /// Native function evaluations; fixed-point mode means G evaluations.
        function_evaluations: Option<u64>,
        /// Native Jacobian evaluations, unavailable for fixed-point mode.
        jacobian_evaluations: Option<u64>,
        /// Native linear iterations, unavailable for fixed-point mode.
        linear_iterations: Option<u64>,
        /// Native preconditioner setup evaluations.
        preconditioner_evaluations: Option<u64>,
        /// Native preconditioner solves.
        preconditioner_solves: Option<u64>,
        /// Native J*v evaluations.
        jacobian_products: Option<u64>,
    },
}
/// Native integration state at an actual solver return, separate from work counters.
/// These observations describe method configuration and mesh, never output accuracy.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NativeStepObservation {
    /// IDAS 7.1.1 optional outputs from the live forward session.
    Idas {
        /// Order used by the last accepted internal step; absent before any step.
        last_order: Option<u32>,
        /// Current order configured for the next internal attempt, not the last order.
        current_order: Option<u32>,
        /// Last accepted internal step size; never IDACalcIC's artificial step.
        #[serde(serialize_with = "serialize_optional_number")]
        #[schemars(with = "Option<pse_model::diagnostic::Observation>")]
        last_step: Option<f64>,
        /// Step size configured for the next internal attempt.
        #[serde(serialize_with = "serialize_optional_number")]
        #[schemars(with = "Option<pse_model::diagnostic::Observation>")]
        current_step: Option<f64>,
        /// Native internal mesh time, which may be beyond an interpolated/root return.
        #[serde(serialize_with = "serialize_optional_number")]
        #[schemars(with = "Option<pse_model::diagnostic::Observation>")]
        internal_time: Option<f64>,
        /// Successful IDASolve/IDASolveF returned time; absent before a solve or on error.
        #[serde(serialize_with = "serialize_optional_number")]
        #[schemars(with = "Option<pse_model::diagnostic::Observation>")]
        returned_time: Option<f64>,
    },
    /// Diffsol 0.16.2 method/state getters captured before any event rewind or reset.
    Diffsol {
        /// Current method order; BDF may already have changed it for the next attempt.
        /// Fixed-order RK schemes report their tableau order. No last-order getter exists.
        current_order: Option<u32>,
        /// StateRef.h: proposed next step after success, current configuration on error.
        /// It is not the step just accepted; no last-step getter is exposed here.
        #[serde(serialize_with = "serialize_optional_number")]
        #[schemars(with = "Option<pse_model::diagnostic::Observation>")]
        current_step: Option<f64>,
        /// Solver mesh time before state_mut_back, independent of requested samples.
        #[serde(serialize_with = "serialize_optional_number")]
        #[schemars(with = "Option<pse_model::diagnostic::Observation>")]
        internal_time: Option<f64>,
        /// Actual successful step return time, including an interpolated root time.
        #[serde(serialize_with = "serialize_optional_number")]
        #[schemars(with = "Option<pse_model::diagnostic::Observation>")]
        returned_time: Option<f64>,
    },
}
/// Exact profile and problem witness for native facts; unknown counters remain absent.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NativeObservations {
    /// Actual native adapter.
    pub backend: Backend,
    /// Submitted oracle contract identity.
    pub problem: ContentHash,
    /// Executed profile identity.
    pub profile: ContentHash,
    /// Point coordinates still require verified outer transport for authored diagnostics.
    pub coordinates: ObservationCoordinates,
    /// Actual cooperative checkpoint coverage.
    pub stop_fidelity: StopFidelity,
    /// Native family-specific getters.
    pub counters: NativeCounters,
    /// Latest available point, shared with consumers instead of copied into every record.
    pub current: Option<Arc<NativeSnapshot>>,
}
/// What the producer knows about a map call, never inferred from numeric proximity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub enum RawMapRole {
    /// Actual fixed-point callback; not evidence of accepted outer iterate status.
    FixedPointEvaluation,
    /// Fresh original residual assessment, separate from native map convergence.
    OriginalAssessment,
    /// A producer cannot establish the role.
    Unknown,
    /// An explicitly authorized restart start screened against the actual map.
    RestartScreening,
}
/// Typed failure class; the complete cause remains with the report's callback failure owner.
#[derive(Clone, Copy, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum RawMapFailure {
    /// Recoverable trial refusal in the scientific producer.
    Trial,
    /// Fatal map failure or nonfinite map output.
    Fatal,
    /// Actual cooperative stop.
    Stopped(Termination),
}
/// One physical tear quantity, or an explicitly submitted-oracle quantity.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawMapCoordinate {
    /// Tear destination variable ID.
    pub variable: SemanticId,
    /// Original connection residual ID.
    pub row: SemanticId,
    /// Actual map input.
    #[serde(serialize_with = "serialize_number")]
    #[schemars(with = "pse_model::diagnostic::Observation")]
    pub input: f64,
    /// Actual G output; absent for failed calls and residual-only assessments.
    pub mapped: Option<f64>,
    /// Raw G(x)-x, or fresh original residual for OriginalAssessment.
    #[serde(serialize_with = "serialize_optional_number")]
    #[schemars(with = "Option<pse_model::diagnostic::Observation>")]
    pub residual: Option<f64>,
    /// Original physical tolerance, present only with physical coordinate authority.
    pub physical_tolerance: Option<f64>,
    /// Physical abs(residual)/tolerance; never native scaling or accelerated step norm.
    #[serde(serialize_with = "serialize_optional_number")]
    #[schemars(with = "Option<pse_model::diagnostic::Observation>")]
    pub physical_error_ratio: Option<f64>,
}
/// A bounded diagnostic map sample; missing fields are unknown, never zero.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawMapSample {
    /// Evaluation ordinal within this attempt, including failures.
    pub ordinal: u64,
    /// Actual callback role.
    pub role: RawMapRole,
    /// Coordinate authority supplied by the scientific producer.
    pub coordinates: ObservationCoordinates,
    /// Bounded coordinate prefix.
    pub values: Vec<RawMapCoordinate>,
    /// Full map dimension.
    pub total_coordinates: usize,
    /// Coordinate suffix was not retained.
    pub partial: bool,
    /// Failure of this evaluation; partial native outputs are never captured as G.
    pub failure: Option<RawMapFailure>,
    /// Availability of actual inner-solve context, distinct from no inner solve.
    pub inner_context: RawMapContextKind,
    /// Inner tolerance is absent when a compound map has no single observed value.
    pub inner_tolerance: Option<f64>,
    /// Observed inner start identity, when supplied by the scientific producer.
    pub inner_start: Option<ContentHash>,
    /// Observed inner branch identity, when supplied by the scientific producer.
    pub inner_branch: Option<ContentHash>,
    /// Exact inner accuracy identity, including compound heterogeneous budgets.
    pub inner_accuracy: Option<ContentHash>,
}
/// Whether an actual inner solve has a supplied frozen scientific context.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RawMapContextKind {
    /// An opaque producer did not supply the fact.
    #[default]
    Unavailable,
    /// The producer has no inner numerical solve.
    Inapplicable,
    /// Actual authored starts, branch permission and effective accuracy are frozen.
    Frozen,
}
/// Actual scientific inner context supplied by a composed map owner. Identities
/// describe frozen authored permission/settings, not a uniqueness proof or achieved branch.
#[derive(Clone, Copy, Debug, Default)]
pub struct RawMapInnerContext {
    /// Distinguish an opaque map from a map without inner solves.
    pub kind: RawMapContextKind,
    /// A scalar inner tolerance only when the producer has exactly one such budget.
    pub tolerance: Option<f64>,
    /// Frozen authored inner starts, independent of previous solved roots.
    pub start: Option<ContentHash>,
    /// Frozen authored branch/start permission and model identity.
    pub branch: Option<ContentHash>,
    /// Exact composite inner effective accuracy/settings identity.
    pub accuracy: Option<ContentHash>,
}
/// Shared bounded history, independent of lossy progress events.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawMapHistory {
    /// Submitted map contract.
    pub problem: ContentHash,
    /// Executed native profile.
    pub profile: ContentHash,
    /// Retained prefix of actual evaluations, shared with last_successful when possible.
    pub samples: Vec<Arc<RawMapSample>>,
    /// Evaluations omitted from the history prefix (latest successful capture may remain).
    pub omitted: u64,
    /// Most recent successful G evaluation, preserved when the next map fails.
    pub last_successful: Option<Arc<RawMapSample>>,
    /// Latest actual evaluation, including a failed point omitted from the prefix.
    pub last_evaluation: Option<Arc<RawMapSample>>,
}

// Partition Controls::report_allowance before capturing any vector: at most 4 KiB
// per retained history item, plus the latest successful/failed samples and one
// temporary replacement sample. A native snapshot is also bounded. The
// allowance already reserves 8 KiB/history item and a fixed 4 MiB envelope. The
// native producer never allocates n*history diagnostic vectors. Actual solver
// vectors and complete candidate ownership remain with their existing owners.
#[cfg(feature = "ipopt")]
pub(crate) const SNAPSHOT_COORDINATES: usize = 64;
const SAMPLE_COORDINATES: usize = 24;
const SAMPLE_BYTES: usize = 4096;
const _: () = assert!(
    size_of::<RawMapCoordinate>() * SAMPLE_COORDINATES
        + size_of::<RawMapSample>()
        + size_of::<Arc<RawMapSample>>()
        + 128
        <= SAMPLE_BYTES
);

#[cfg(any(feature = "kinsol", test))]
pub(crate) struct RawMapCapture {
    history: RawMapHistory,
    limit: usize,
    ordinal: u64,
    coordinates: ObservationCoordinates,
    tolerances: Vec<f64>,
    inner: RawMapInnerContext,
}
#[cfg(any(feature = "kinsol", test))]
impl RawMapCapture {
    pub(crate) fn new(
        contract: &crate::OracleContract,
        profile: ContentHash,
        history: usize,
        physical: bool,
        tolerances: &[f64],
    ) -> Result<Self, crate::ProblemError> {
        // Check the finite partition before allocating even the prefix's pointer array.
        history
            .checked_mul(SAMPLE_BYTES)
            .and_then(|n| n.checked_add(3 * SAMPLE_BYTES))
            .ok_or_else(|| crate::ProblemError::memory("raw map history extent"))?;
        Ok(Self {
            history: RawMapHistory {
                problem: contract.identity,
                profile,
                samples: Vec::with_capacity(history),
                omitted: 0,
                last_successful: None,
                last_evaluation: None,
            },
            inner: RawMapInnerContext::default(),
            limit: history,
            ordinal: 0,
            coordinates: if physical {
                ObservationCoordinates::PhysicalTears
            } else {
                ObservationCoordinates::SubmittedOracle
            },
            tolerances: tolerances
                .iter()
                .copied()
                .take(SAMPLE_COORDINATES)
                .collect(),
        })
    }
    pub(crate) fn with_inner_context(mut self, inner: RawMapInnerContext) -> Self {
        self.inner = inner;
        self
    }
    pub(crate) fn record(
        &mut self,
        contract: &crate::OracleContract,
        x: &[f64],
        values: Option<&[f64]>,
        role: RawMapRole,
        mut failure: Option<RawMapFailure>,
    ) {
        if role == RawMapRole::OriginalAssessment
            && values.is_some_and(|values| values.iter().any(|value| !value.is_finite()))
            && failure.is_none()
        {
            failure = Some(RawMapFailure::Fatal);
        }
        self.ordinal = self.ordinal.saturating_add(1);
        let retain = self.history.samples.len() < self.limit;
        if !retain {
            self.history.omitted = self.history.omitted.saturating_add(1);
        }
        let map_role = matches!(role, RawMapRole::FixedPointEvaluation | RawMapRole::RestartScreening);
        let successful_map = failure.is_none() && values.is_some() && map_role;
        let length = x.len().min(SAMPLE_COORDINATES);
        let mut captured = Vec::with_capacity(length);
        for (i, input) in x.iter().copied().take(length).enumerate() {
            let observed = values.and_then(|v| v.get(i)).copied();
            let mapped = map_role
                .then_some(observed)
                .flatten()
                .filter(|value| value.is_finite());
            let residual = if role == RawMapRole::OriginalAssessment {
                observed
            } else {
                mapped.map(|v| v - input)
            };
            let tolerance = (self.coordinates == ObservationCoordinates::PhysicalTears)
                .then(|| self.tolerances.get(i).copied())
                .flatten();
            captured.push(RawMapCoordinate {
                variable: contract.variables[i].id,
                row: contract.rows[i],
                input,
                mapped,
                residual,
                physical_tolerance: tolerance,
                physical_error_ratio: residual.zip(tolerance).map(|(r, t)| r.abs() / t),
            });
        }
        let sample = Arc::new(RawMapSample {
            ordinal: self.ordinal,
            role,
            coordinates: self.coordinates,
            values: captured,
            total_coordinates: x.len(),
            partial: length < x.len(),
            failure,
            inner_context: self.inner.kind,
            inner_tolerance: self.inner.tolerance,
            inner_start: self.inner.start,
            inner_branch: self.inner.branch,
            inner_accuracy: self.inner.accuracy,
        });
        if successful_map {
            self.history.last_successful = Some(sample.clone());
        }
        self.history.last_evaluation = Some(sample.clone());
        if retain {
            self.history.samples.push(sample);
        }
    }
    pub(crate) fn finish(self) -> Arc<RawMapHistory> {
        Arc::new(self.history)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_map_context_survives_truncation_and_failed_latest_sample() {
        let contract = crate::solver_tests::contract();
        let context = RawMapInnerContext {
            kind: RawMapContextKind::Frozen,
            tolerance: None,
            start: Some(ContentHash::from_bytes([31; 32])),
            branch: Some(ContentHash::from_bytes([32; 32])),
            accuracy: Some(ContentHash::from_bytes([33; 32])),
        };
        let mut capture = RawMapCapture::new(&contract, contract.identity, 0, true, &[0.25])
            .unwrap().with_inner_context(context);
        capture.record(&contract, &[1.0], Some(&[2.0]), RawMapRole::FixedPointEvaluation, None);
        capture.record(&contract, &[3.0], None, RawMapRole::FixedPointEvaluation, Some(RawMapFailure::Trial));
        let history = capture.finish();
        assert_eq!(history.omitted, 2);
        for sample in [&history.last_successful, &history.last_evaluation] {
            let sample = sample.as_ref().unwrap();
            assert_eq!(sample.inner_context, RawMapContextKind::Frozen);
            assert_eq!(sample.inner_start, context.start);
            assert_eq!(sample.inner_branch, context.branch);
            assert_eq!(sample.inner_accuracy, context.accuracy);
            assert_eq!(sample.inner_tolerance, None);
        }
        assert_eq!(history.last_successful.as_ref().unwrap().values[0].input, 1.0);
        assert_eq!(history.last_evaluation.as_ref().unwrap().values[0].input, 3.0);
    }
    #[test]
    fn raw_map_capture_retains_physical_residuals_and_failed_last_point_with_bounded_history() {
        let contract = crate::solver_tests::contract();
        let mut capture =
            RawMapCapture::new(&contract, contract.identity, 2, true, &[0.25]).unwrap();
        capture.record(
            &contract,
            &[4.],
            Some(&[1.]),
            RawMapRole::FixedPointEvaluation,
            None,
        );
        capture.record(
            &contract,
            &[1.],
            Some(&[2.]),
            RawMapRole::FixedPointEvaluation,
            None,
        );
        capture.record(
            &contract,
            &[2.],
            Some(&[1.5]),
            RawMapRole::FixedPointEvaluation,
            None,
        );
        capture.record(
            &contract,
            &[1.5],
            None,
            RawMapRole::FixedPointEvaluation,
            Some(RawMapFailure::Trial),
        );
        let history = capture.finish();
        assert_eq!(history.samples.len(), 2);
        assert_eq!(history.omitted, 2);
        assert!(matches!(
            history.last_evaluation.as_ref().unwrap().failure,
            Some(RawMapFailure::Trial)
        ));
        let last = history.last_successful.as_ref().unwrap();
        assert_eq!(last.ordinal, 3);
        assert_eq!(last.values[0].mapped, Some(1.5));
        assert_eq!(last.values[0].residual, Some(-0.5));
        assert_eq!(last.values[0].physical_error_ratio, Some(2.));
        assert_eq!(history.samples[0].values[0].residual, Some(-3.));
        assert_eq!(history.samples[0].values[0].physical_error_ratio, Some(12.));
    }
    #[test]
    fn raw_map_capture_marks_partial_coordinates_and_unknown_physical_authority() {
        let mut contract = crate::solver_tests::contract();
        contract.variables = vec![contract.variables[0].clone(); 1000];
        contract.rows = vec![contract.rows[0]; 1000];
        let mut capture =
            RawMapCapture::new(&contract, contract.identity, 0, false, &[1.; 1000]).unwrap();
        capture.record(
            &contract,
            &[1.; 1000],
            Some(&[2.; 1000]),
            RawMapRole::FixedPointEvaluation,
            None,
        );
        let history = capture.finish();
        assert!(history.samples.is_empty());
        assert_eq!(history.omitted, 1);
        let last = history.last_successful.as_ref().unwrap();
        assert!(last.partial);
        assert_eq!(last.total_coordinates, 1000);
        assert_eq!(last.values.len(), SAMPLE_COORDINATES);
        assert_eq!(last.values[0].physical_tolerance, None);
        assert_eq!(last.values[0].physical_error_ratio, None);
    }
    #[test]
    fn raw_map_capture_preserves_nonfinite_observations_in_json() {
        let contract = crate::solver_tests::contract();
        let mut capture = RawMapCapture::new(&contract, contract.identity, 1, true, &[1.]).unwrap();
        capture.record(
            &contract,
            &[f64::MAX],
            Some(&[-f64::MAX]),
            RawMapRole::FixedPointEvaluation,
            None,
        );
        let history = capture.finish();
        let value = serde_json::to_value(&history).unwrap();
        assert_eq!(
            value["samples"][0]["values"][0]["residual"]["kind"],
            "nonfinite"
        );
        assert_eq!(
            value["samples"][0]["values"][0]["residual"]["value"],
            "negative_infinity"
        );
        assert_eq!(
            value["samples"][0]["values"][0]["physical_error_ratio"]["kind"],
            "nonfinite"
        );
    }
    #[test]
    fn native_step_observation_preserves_absence_and_nonfinite_mesh_without_last_step_invention() {
        let idas = NativeStepObservation::Idas {
            last_order: None,
            current_order: Some(1),
            last_step: None,
            current_step: Some(0.01),
            internal_time: Some(f64::NAN),
            returned_time: None,
        };
        let value = serde_json::to_value(idas).unwrap();
        assert_eq!(value["kind"], "idas");
        assert!(value["last_order"].is_null() && value["last_step"].is_null());
        assert!(value["returned_time"].is_null());
        assert_eq!(value["internal_time"]["kind"], "nonfinite");
        let diffsol = NativeStepObservation::Diffsol {
            current_order: Some(3),
            current_step: Some(0.2),
            internal_time: Some(0.5),
            returned_time: Some(0.4),
        };
        let value = serde_json::to_value(diffsol).unwrap();
        assert_eq!(value["internal_time"]["value"], 0.5);
        assert_eq!(value["returned_time"]["value"], 0.4);
        assert!(value.get("last_order").is_none() && value.get("last_step").is_none());
        let schema = serde_json::to_value(schemars::schema_for!(NativeStepObservation)).unwrap();
        assert!(
            schema
                .to_string()
                .contains("DiagnosticNonfiniteObservation")
        );
    }
    #[test]
    fn native_observations_encode_nonfinite_coordinates_and_times_with_matching_schema() {
        let snapshot = NativeSnapshot {
            iteration: Some(1),
            phase: NativePhase::Restoration,
            values: vec![NativeCoordinate {
                variable: SemanticId::NIL,
                value: f64::NAN,
            }],
            total_coordinates: 1,
            partial: false,
        };
        let value = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(value["values"][0]["value"]["kind"], "nonfinite");
        let schema = serde_json::to_value(schemars::schema_for!(NativeSnapshot)).unwrap();
        assert!(
            schema
                .to_string()
                .contains("DiagnosticNonfiniteObservation")
        );
        let counters = NativeCounters::Ipopt {
            iterations: None,
            objective_evaluations: None,
            constraint_evaluations: None,
            gradient_evaluations: None,
            jacobian_evaluations: None,
            hessian_evaluations: None,
            cpu_seconds: Some(f64::INFINITY),
            system_seconds: None,
            wall_seconds: Some(0.),
        };
        let value = serde_json::to_value(&counters).unwrap();
        assert_eq!(value["cpu_seconds"]["kind"], "nonfinite");
        assert_eq!(value["system_seconds"], serde_json::Value::Null);
        assert_eq!(value["wall_seconds"]["kind"], "real");
    }
    #[test]
    fn raw_map_nonfinite_original_assessment_keeps_failure_and_last_success() {
        for (value, tag) in [
            (f64::NAN, "nan"),
            (f64::INFINITY, "positive_infinity"),
            (f64::NEG_INFINITY, "negative_infinity"),
        ] {
            let contract = crate::solver_tests::contract();
            let mut capture =
                RawMapCapture::new(&contract, contract.identity, 1, true, &[1.]).unwrap();
            capture.record(
                &contract,
                &[0.],
                Some(&[1.]),
                RawMapRole::FixedPointEvaluation,
                None,
            );
            capture.record(
                &contract,
                &[1.],
                Some(&[value]),
                RawMapRole::OriginalAssessment,
                None,
            );
            let history = capture.finish();
            let last = history.last_evaluation.as_ref().unwrap();
            assert!(matches!(last.failure, Some(RawMapFailure::Fatal)));
            assert_eq!(
                history.last_successful.as_ref().unwrap().values[0].mapped,
                Some(1.)
            );
            let json = serde_json::to_value(last).unwrap();
            assert_eq!(json["values"][0]["residual"]["kind"], "nonfinite");
            assert_eq!(json["values"][0]["residual"]["value"], tag);
        }
    }
}
