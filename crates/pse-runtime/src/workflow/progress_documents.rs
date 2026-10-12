// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owned progress observations preserve integral counters, unavailable evidence and provenance.
use pse_backend_native::solve::{Event, IncumbentEvent, Metric};
use pse_model::diagnostic::Observation;
use std::collections::BTreeMap;

/// Original ordinary telemetry coverage, independent of scientific completion.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProgressCoverage {
    /// Historical or recovered executions did not retain original counters.
    #[default]
    Unknown,
    /// Finite sampling counters retained by the original execution owner.
    Sampled {
        /// Callback observations, including events rejected before cloning.
        observed: u64,
        /// Events admitted to the finite writer queue.
        queued: u64,
        /// Observations rejected before queue admission.
        dropped_before_queue: u64,
        /// Events in batches with actual canonical append acknowledgments.
        acknowledged: u64,
        /// Queued events without an observed append acknowledgment; storage may be uncertain.
        unacknowledged: u64,
        /// Writer completion, separate from the scientific outcome.
        writer: ProgressWriterOutcome,
    },
}

/// Bounded optional writer outcome; none of these authorizes scientific interruption.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(
    tag = "kind",
    content = "failure",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ProgressWriterOutcome {
    /// Every queued event has an actual append acknowledgment.
    Complete,
    /// Up-front memory admission refused; no event queue was allocated.
    MemoryRefused,
    /// The writer refused or failed; unacknowledged coverage remains explicit.
    Failed(ProgressWriterFailure),
    /// The owned writer task failed to join normally.
    JoinFailed,
}

/// Original bounded failure of the optional progress writer.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ProgressWriterFailure {
    /// Exact bounded stage which did not return its promised acknowledgment.
    pub stage: ProgressWriteStage,
}

/// Optional writer failure boundary; queued unacknowledged events remain uncertain.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProgressWriteStage {
    /// No original native claim was available for the observed event.
    Claim,
    /// Bounded scratch could not be allocated.
    Allocation,
    /// An admitted DTO could not be encoded within its immutable bound.
    Encoding,
    /// The original canonical append did not return an acknowledgment.
    CanonicalAppend,
    /// An acknowledgment disagreed with original immutable coverage.
    Acknowledgment,
    /// Publication stopped before an original canonical append could be issued.
    Publication,
}

/// Conservative bound for the retained DTO and its escaped JSON representation.
/// Inspect borrowed strings/maps only; native incumbent primal storage is never traversed.
pub(crate) fn bounded_event(event: &Event, maximum: usize) -> bool {
    let Some(mut extent) = 1024usize.checked_add(event.phase.len().saturating_mul(6)) else {
        return false;
    };
    if extent > maximum {
        return false;
    }
    for (key, value) in &event.values {
        let text = match value {
            Metric::Text(value) => value.len(),
            _ => 0,
        };
        let Some(next) = key
            .len()
            .checked_add(text)
            .and_then(|bytes| bytes.checked_mul(6))
            .and_then(|bytes| bytes.checked_add(512))
            .and_then(|bytes| extent.checked_add(bytes))
        else {
            return false;
        };
        extent = next;
        if extent > maximum {
            return false;
        }
    }
    true
}

/// Portable observation of one native metric, independent of its backend-specific key.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ProgressMetricDocument {
    /// Exact integral count or identifier.
    Integer(i64),
    /// Every IEEE real value, with explicit nonfinite evidence when necessary.
    Real(Observation),
    /// Original native explanatory text.
    Text(String),
    /// Original native Boolean.
    Boolean(bool),
    /// Canonical reason that no observation was available.
    Unavailable(pse_model::generated::enums::EvidenceUnavailableReason),
}
impl From<Metric> for ProgressMetricDocument {
    fn from(metric: Metric) -> Self {
        match metric {
            Metric::Integer(value) => Self::Integer(value),
            Metric::Real(value) => Self::Real(Observation::number(value)),
            Metric::Text(value) => Self::Text(value),
            Metric::Bool(value) => Self::Boolean(value),
            Metric::Unavailable(reason) => Self::Unavailable(reason),
        }
    }
}
impl From<&Metric> for ProgressMetricDocument {
    fn from(metric: &Metric) -> Self {
        match metric {
            Metric::Integer(value) => Self::Integer(*value),
            Metric::Real(value) => Self::Real(Observation::number(*value)),
            Metric::Text(value) => Self::Text(value.clone()),
            Metric::Bool(value) => Self::Boolean(*value),
            Metric::Unavailable(reason) => Self::Unavailable(*reason),
        }
    }
}
/// An incumbent in original objective units and its retained native search evidence.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IncumbentDocument {
    /// Original objective value.
    pub objective: f64,
    /// Finite native global dual bound, when available.
    pub dual_bound: Option<f64>,
    /// Finite native relative gap, when available.
    pub gap: Option<f64>,
    /// Native nodes explored by this observation.
    pub nodes: Option<i64>,
    /// Native elapsed seconds by this observation.
    pub seconds: Option<f64>,
    /// Identity of the stored solution retained for resumption.
    pub solution_id: Option<pse_ids::SemanticId>,
}
impl From<&IncumbentEvent> for IncumbentDocument {
    fn from(incumbent: &IncumbentEvent) -> Self {
        Self {
            objective: incumbent.objective,
            dual_bound: incumbent.dual_bound,
            gap: incumbent.gap,
            nodes: Some(incumbent.nodes),
            seconds: Some(incumbent.seconds),
            solution_id: None,
        }
    }
}
/// An owned event from memory or from a durable attempt's selected stream.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProgressEventDocument {
    /// Original native phase or callback name.
    pub phase: String,
    /// Seconds since admitted step execution began.
    pub elapsed_seconds: f64,
    /// Typed original metric values by backend-specific key.
    pub values: BTreeMap<String, ProgressMetricDocument>,
    /// Typed incumbent evidence when this event reported an incumbent.
    pub incumbent: Option<IncumbentDocument>,
    /// Durable producing step; absent for observations retained only in memory.
    pub step: Option<i32>,
    /// Sequence in this selected durable stream; absent in memory.
    pub sequence: Option<i64>,
    /// Durable observation time in Unix microseconds; absent in memory.
    pub at: Option<i64>,
}
impl From<Event> for ProgressEventDocument {
    fn from(event: Event) -> Self {
        Self::from(&event)
    }
}
impl From<&Event> for ProgressEventDocument {
    fn from(event: &Event) -> Self {
        Self {
            phase: event.phase.clone(),
            elapsed_seconds: event.elapsed.as_secs_f64(),
            values: event
                .values
                .iter()
                .map(|(key, value)| (key.clone(), value.into()))
                .collect(),
            incumbent: event.incumbent.as_ref().map(Into::into),
            step: None,
            sequence: None,
            at: None,
        }
    }
}

#[cfg(test)]
mod boundary_unit {
    use super::*;
    #[test]
    fn progress_coverage_preserves_original_failure_and_refuses_unknown_boundary_fields() {
        let original = ProgressCoverage::Sampled {
            observed: 7,
            queued: 5,
            dropped_before_queue: 2,
            acknowledged: 3,
            unacknowledged: 2,
            writer: ProgressWriterOutcome::Failed(ProgressWriterFailure {
                stage: ProgressWriteStage::CanonicalAppend,
            }),
        };
        let mut encoded = serde_json::to_value(&original).unwrap();
        let decoded: ProgressCoverage = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(decoded, original);
        assert_eq!(
            encoded["writer"]["failure"]["stage"]["kind"],
            "canonical_append"
        );
        encoded["writer"]["failure"]["stage"]["invented"] = serde_json::json!(true);
        assert!(serde_json::from_value::<ProgressCoverage>(encoded).is_err());
    }
    #[test]
    fn progress_preserves_exact_counters_and_nonfinite_evidence() {
        let event = Event {
            phase: "native callback".into(),
            elapsed: std::time::Duration::from_millis(5),
            values: [
                ("large".into(), Metric::Integer((1i64 << 53) + 1)),
                ("nan".into(), Metric::Real(f64::NAN)),
            ]
            .into(),
            incumbent: Some(IncumbentEvent {
                objective: -0.0,
                dual_bound: None,
                gap: None,
                nodes: (1i64 << 53) + 3,
                seconds: 0.005,
                primal: Some(vec![2.0; 32_768]),
            }),
        };
        let document = ProgressEventDocument::from(&event);
        let encoded = serde_json::to_vec(&document).unwrap();
        assert!(
            encoded.len() < 1024,
            "progress DTO carries incumbent scalars rather than the native primal vector"
        );
        assert_eq!(
            event
                .incumbent
                .as_ref()
                .unwrap()
                .primal
                .as_ref()
                .unwrap()
                .len(),
            32_768
        );
        assert_eq!(
            serde_json::to_vec(&ProgressEventDocument::from(event)).unwrap(),
            encoded
        );
        let retained: ProgressEventDocument = serde_json::from_slice(&encoded).unwrap();
        assert!(
            matches!(retained.values["large"], ProgressMetricDocument::Integer(value) if value == (1i64<<53)+1)
        );
        assert!(matches!(
            retained.values["nan"],
            ProgressMetricDocument::Real(Observation::Nonfinite(_))
        ));
        assert!(retained.at.is_none());
        assert_eq!(
            retained.incumbent.as_ref().unwrap().objective.to_bits(),
            (-0.0f64).to_bits()
        );
        assert_eq!(
            retained.incumbent.as_ref().unwrap().nodes,
            Some((1i64 << 53) + 3)
        );
    }
}
