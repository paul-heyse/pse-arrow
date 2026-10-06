// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owned progress observations preserve integral counters, unavailable evidence and provenance.
use pse_backend_native::solve::{Event, IncumbentEvent, Metric};
use pse_model::diagnostic::Observation;
use std::collections::BTreeMap;

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
    fn progress_preserves_exact_counters_and_nonfinite_evidence() {
        let event = Event {
            phase: "native callback".into(),
            elapsed: std::time::Duration::from_millis(5),
            values: [
                ("large".into(), Metric::Integer((1i64 << 53) + 1)),
                ("nan".into(), Metric::Real(f64::NAN)),
            ]
            .into(),
            incumbent: Some(IncumbentEvent {objective:-0.0,dual_bound:None,gap:None,nodes:(1i64<<53)+3,seconds:0.005,primal:Some(vec![2.0;32_768])}),
        };
        let document = ProgressEventDocument::from(&event);
        let encoded = serde_json::to_vec(&document).unwrap();
        assert!(encoded.len()<1024,"progress DTO carries incumbent scalars rather than the native primal vector");
        assert_eq!(event.incumbent.as_ref().unwrap().primal.as_ref().unwrap().len(),32_768);
        assert_eq!(serde_json::to_vec(&ProgressEventDocument::from(event)).unwrap(),encoded);
        let retained: ProgressEventDocument = serde_json::from_slice(&encoded).unwrap();
        assert!(
            matches!(retained.values["large"], ProgressMetricDocument::Integer(value) if value == (1i64<<53)+1)
        );
        assert!(matches!(
            retained.values["nan"],
            ProgressMetricDocument::Real(Observation::Nonfinite(_))
        ));
        assert!(retained.at.is_none());
        assert_eq!(retained.incumbent.as_ref().unwrap().objective.to_bits(),(-0.0f64).to_bits());
        assert_eq!(retained.incumbent.as_ref().unwrap().nodes,Some((1i64<<53)+3));
    }
}
