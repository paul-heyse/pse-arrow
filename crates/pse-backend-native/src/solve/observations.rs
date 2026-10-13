// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Synchronous native facts; telemetry retention never supplies decision evidence.
use super::{Backend, ContentHash, SemanticId, Termination};
use std::sync::Arc;
fn serialize_number<S: serde::Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    serde::Serialize::serialize(&pse_model::diagnostic::Observation::number(*value), serializer)
}
fn serialize_optional_number<S: serde::Serializer>(value: &Option<f64>, serializer: S)
    -> Result<S::Ok, S::Error>
{
    serde::Serialize::serialize(&value.map(pse_model::diagnostic::Observation::number), serializer)
}

/// Checkpoint coverage of the actual adapter, not a promise of native preemption.
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub enum StopFidelity {
    /// Evaluations and native iteration callbacks poll the original execution scope.
    EvaluationAndIterationCheckpoints,
    /// Evaluations poll; an indivisible native linear operation must finish first.
    EvaluationCheckpoints,
}
/// Coordinate authority of a captured point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum ObservationCoordinates {
    /// Coordinates submitted to the oracle; an outer formulation may still transform them.
    SubmittedOracle,
    /// The causal producer explicitly supplies physical tear destination quantities.
    PhysicalTears,
}
/// Native phase, without inventing internal restoration variables or slacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum NativePhase {
    /// Ordinary native optimization.
    Main,
    /// Native restoration phase.
    Restoration,
    /// The callback did not establish the phase.
    Unknown,
}
/// Latest callback point in its submitted coordinate authority, not a resumable state.
#[derive(Clone, Debug, serde::Serialize)]
pub struct NativeSnapshot {
    /// Actual native iteration ordinal.
    pub iteration: Option<u64>,
    /// Actual callback phase.
    pub phase: NativePhase,
    /// Captured coordinate prefix, with source IDs.
    pub values: Vec<(SemanticId, f64)>,
    /// Full submitted coordinate count, including any omitted suffix.
    pub total_coordinates: usize,
    /// A suffix was omitted by the fixed capture allocation.
    pub partial: bool,
}
/// Family-specific native meanings; these counters are not interchangeable cost units.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
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
        cpu_seconds: Option<f64>,
        /// System time including callbacks; never summed with wall time.
        system_seconds: Option<f64>,
        /// Native algorithm wall time including callbacks.
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
/// Exact profile and problem witness for native facts; unknown counters remain absent.
#[derive(Clone, Debug, serde::Serialize)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum RawMapRole {
    /// Actual fixed-point callback; not evidence of accepted outer iterate status.
    FixedPointEvaluation,
    /// Fresh original residual assessment, separate from native map convergence.
    OriginalAssessment,
    /// A producer cannot establish the role.
    Unknown,
}
/// Typed failure class; the complete cause remains with the report's callback failure owner.
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub enum RawMapFailure {
    /// Recoverable trial refusal in the scientific producer.
    Trial,
    /// Fatal map failure or nonfinite map output.
    Fatal,
    /// Actual cooperative stop.
    Stopped(Termination),
}
/// One physical tear quantity, or an explicitly submitted-oracle quantity.
#[derive(Clone, Debug, serde::Serialize)]
pub struct RawMapCoordinate {
    /// Tear destination variable ID.
    pub variable: SemanticId,
    /// Original connection residual ID.
    pub row: SemanticId,
    /// Actual map input.
    #[serde(serialize_with = "serialize_number")]
    pub input: f64,
    /// Actual G output; absent for failed calls and residual-only assessments.
    pub mapped: Option<f64>,
    /// Raw G(x)-x, or fresh original residual for OriginalAssessment.
    #[serde(serialize_with = "serialize_optional_number")]
    pub residual: Option<f64>,
    /// Original physical tolerance, present only with physical coordinate authority.
    pub physical_tolerance: Option<f64>,
    /// Physical abs(residual)/tolerance; never native scaling or accelerated step norm.
    #[serde(serialize_with = "serialize_optional_number")]
    pub physical_error_ratio: Option<f64>,
}
/// A bounded diagnostic map sample; missing fields are unknown, never zero.
#[derive(Clone, Debug, serde::Serialize)]
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
    /// Inner tolerance is absent when a compound map has no single observed value.
    pub inner_tolerance: Option<f64>,
    /// Observed inner start identity, when supplied by the scientific producer.
    pub inner_start: Option<ContentHash>,
    /// Observed inner branch identity, when supplied by the scientific producer.
    pub inner_branch: Option<ContentHash>,
}
/// Shared bounded history, independent of lossy progress events.
#[derive(Clone, Debug, serde::Serialize)]
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
const _: () = assert!(size_of::<RawMapCoordinate>() * SAMPLE_COORDINATES
    + size_of::<RawMapSample>() + size_of::<Arc<RawMapSample>>() + 128 <= SAMPLE_BYTES);

#[cfg(any(feature = "kinsol", test))]
pub(crate) struct RawMapCapture {
    history: RawMapHistory,
    limit: usize,
    ordinal: u64,
    coordinates: ObservationCoordinates,
    tolerances: Vec<f64>,
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
        history.checked_mul(SAMPLE_BYTES).and_then(|n| n.checked_add(3 * SAMPLE_BYTES))
            .ok_or_else(|| crate::ProblemError::memory("raw map history extent"))?;
        Ok(Self {
            history: RawMapHistory { problem: contract.identity, profile,
                samples: Vec::with_capacity(history), omitted: 0, last_successful: None,
                last_evaluation: None },
            limit: history,
            ordinal: 0,
            coordinates: if physical { ObservationCoordinates::PhysicalTears }
                else { ObservationCoordinates::SubmittedOracle },
            tolerances: tolerances.iter().copied().take(SAMPLE_COORDINATES).collect(),
        })
    }
    pub(crate) fn record(
        &mut self,
        contract: &crate::OracleContract,
        x: &[f64],
        values: Option<&[f64]>,
        role: RawMapRole,
        failure: Option<RawMapFailure>,
    ) {
        self.ordinal = self.ordinal.saturating_add(1);
        let retain = self.history.samples.len() < self.limit;
        if !retain { self.history.omitted = self.history.omitted.saturating_add(1); }
        let successful_map = failure.is_none() && values.is_some()
            && role == RawMapRole::FixedPointEvaluation;
        let length = x.len().min(SAMPLE_COORDINATES);
        let mut captured = Vec::with_capacity(length);
        for (i, input) in x.iter().copied().take(length).enumerate() {
            let observed = values.and_then(|v| v.get(i)).copied().filter(|v| v.is_finite());
            let mapped = (role == RawMapRole::FixedPointEvaluation).then_some(observed).flatten();
            let residual = if role == RawMapRole::OriginalAssessment { observed }
                else { mapped.map(|v| v - input) };
            let tolerance = (self.coordinates == ObservationCoordinates::PhysicalTears)
                .then(|| self.tolerances.get(i).copied()).flatten();
            captured.push(RawMapCoordinate { variable: contract.variables[i].id,
                row: contract.rows[i], input, mapped, residual, physical_tolerance: tolerance,
                physical_error_ratio: residual.zip(tolerance).map(|(r,t)| r.abs()/t) });
        }
        let sample = Arc::new(RawMapSample { ordinal: self.ordinal, role,
            coordinates: self.coordinates, values: captured, total_coordinates: x.len(),
            partial: length < x.len(), failure, inner_tolerance: None,
            inner_start: None, inner_branch: None });
        if successful_map { self.history.last_successful = Some(sample.clone()); }
        self.history.last_evaluation = Some(sample.clone());
        if retain { self.history.samples.push(sample); }
    }
    pub(crate) fn finish(self) -> Arc<RawMapHistory> { Arc::new(self.history) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_map_capture_retains_physical_residuals_and_failed_last_point_with_bounded_history() {
        let contract = crate::solver_tests::contract();
        let mut capture = RawMapCapture::new(&contract, contract.identity, 2, true, &[0.25]).unwrap();
        capture.record(&contract, &[4.], Some(&[1.]), RawMapRole::FixedPointEvaluation, None);
        capture.record(&contract, &[1.], Some(&[2.]), RawMapRole::FixedPointEvaluation, None);
        capture.record(&contract, &[2.], Some(&[1.5]), RawMapRole::FixedPointEvaluation, None);
        capture.record(&contract, &[1.5], None, RawMapRole::FixedPointEvaluation, Some(RawMapFailure::Trial));
        let history = capture.finish();
        assert_eq!(history.samples.len(), 2);
        assert_eq!(history.omitted, 2);
        assert!(matches!(history.last_evaluation.as_ref().unwrap().failure, Some(RawMapFailure::Trial)));
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
        let mut capture = RawMapCapture::new(&contract, contract.identity, 0, false, &[1.; 1000]).unwrap();
        capture.record(&contract, &[1.; 1000], Some(&[2.; 1000]), RawMapRole::FixedPointEvaluation, None);
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
        capture.record(&contract, &[f64::MAX], Some(&[-f64::MAX]), RawMapRole::FixedPointEvaluation, None);
        let history = capture.finish();
        let value = serde_json::to_value(&history).unwrap();
        assert_eq!(value["samples"][0]["values"][0]["residual"]["kind"], "nonfinite");
        assert_eq!(value["samples"][0]["values"][0]["residual"]["value"], "negative_infinity");
        assert_eq!(value["samples"][0]["values"][0]["physical_error_ratio"]["kind"], "nonfinite");
    }
}
