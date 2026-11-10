// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared bounded projection of supplier plan observations; unavailable counters remain unknown.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use surrealdb_types::Value;

/// Source-owned context for one exact selected native query. Unknown supplier facts stay absent.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NativeExplanationIdentity {
    /// Issuer from the same admitted store, not a caller's display alias.
    pub issuer: String,
    /// Exact generation/revision/continuation selection supplied by the query owner.
    pub data_cut: serde_json::Value,
    /// Actual supplied server version; an unobserved version is unknown.
    pub server_version: Option<String>,
    /// Exact prepared recipe when the physical owner supplies it.
    pub server_recipe: Option<String>,
    /// Original physical server invocation when supplied by its owner.
    pub server_invocation: Option<String>,
}
/// Bounded projection of a selected query and supplier facts, not a permanent supplier plan schema.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NativeQueryExplanation {
    /// Selected issuer, cut and available incarnation facts.
    pub identity: NativeExplanationIdentity,
    /// Actual checked source query description and bindings used by this owner.
    pub query: serde_json::Value,
    /// True only for explicitly admitted ANALYZE; ordinary inspection does not execute the query.
    pub analyzed: bool,
    /// Bounded supplier operator observations with missing counters omitted.
    pub observations: Vec<serde_json::Value>,
    /// Complete original inspection/profile response, when actually received.
    pub response: Option<NativeExplanationResponse>,
}
/// Original complete response facts. Durations are never combined with caller-side clocks.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NativeExplanationResponse {
    /// Whether these facts describe EXPLAIN inspection or the explicitly profiled operation.
    pub kind: NativeExplanationResponseKind,
    /// Supplied per-statement facts, retaining statement positions and absent durations.
    pub statements: Vec<NativeExplanationStatement>,
}
/// Distinguish passive inspection work from executing the selected profiled operation.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NativeExplanationResponseKind {
    /// Original response to a nonexecuting EXPLAIN inspection.
    Inspection,
    /// Original response to an explicitly admitted EXPLAIN ANALYZE operation.
    ProfiledOperation,
}
/// Facts from one original response slot.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NativeExplanationStatement {
    /// Actual original statement position.
    pub statement: usize,
    /// Original statement status, not inferred from a plan or output count.
    pub succeeded: bool,
    /// Supplied database execution time only, in nanoseconds; absent remains unknown.
    pub database_execution_ns: Option<u64>,
}
impl NativeExplanationResponse {
    /// Capture complete original statistics before extracting any statement value.
    /// # Errors
    /// Empty or oversized response populations cannot become complete explanations.
    pub fn capture(
        response: &crate::transport::client::IndexedResults,
        kind: NativeExplanationResponseKind,
    ) -> Result<Self, crate::canonical::CanonicalError> {
        if response.num_statements() == 0 || response.num_statements() > 256 {
            return Err(crate::canonical::CanonicalError::Configuration(
                "native explanation response is incomplete or exceeds bounds".into(),
            ));
        }
        Ok(Self {
            kind,
            statements: response
                .statement_statistics()
                .map(|(statement, stats, succeeded)| NativeExplanationStatement {
                    statement,
                    succeeded,
                    database_execution_ns: stats
                        .execution_time
                        .and_then(|duration| u64::try_from(duration.as_nanos()).ok()),
                })
                .collect(),
        })
    }
}
impl NativeQueryExplanation {
    /// Attach one checked owner description to bounded plan facts.
    /// # Errors
    /// Invalid context or a description/plan beyond the diagnostic byte bound is refused.
    pub fn new(
        identity: NativeExplanationIdentity,
        query: serde_json::Value,
        plan: &Value,
        analyzed: bool,
    ) -> Result<Self, crate::canonical::CanonicalError> {
        if identity.issuer.is_empty()
            || identity.issuer.len() > 256
            || serde_json::to_vec(&identity).map_or(true, |value| value.len() > 64 * 1024)
            || serde_json::to_vec(&query).map_or(true, |value| value.len() > 256 * 1024)
            || serde_json::to_vec(&plan.clone().into_json_value())
                .map_or(true, |value| value.len() > 1024 * 1024)
        {
            return Err(crate::canonical::CanonicalError::Configuration(
                "native explanation exceeds selected diagnostic bounds".into(),
            ));
        }
        Ok(Self {
            identity,
            query,
            analyzed,
            observations: native_plan_observations(plan),
            response: None,
        })
    }
    /// Attach facts from the original complete response, never from a progress frame or timeout.
    /// # Errors
    /// Empty/oversized responses, or profiling facts on a passive explanation, are refused.
    pub fn attach_response(
        &mut self,
        response: &crate::transport::client::IndexedResults,
        kind: NativeExplanationResponseKind,
    ) -> Result<(), crate::canonical::CanonicalError> {
        if !self.analyzed && matches!(kind, NativeExplanationResponseKind::ProfiledOperation) {
            return Err(crate::canonical::CanonicalError::Configuration(
                "native explanation response is incomplete or mismatched".into(),
            ));
        }
        self.response = Some(NativeExplanationResponse::capture(response, kind)?);
        Ok(())
    }
}

/// Project native operator facts without turning missing positive-only counters into zeroes.
pub fn native_plan_observations(value: &Value) -> Vec<serde_json::Value> {
    fn fact(value: &Value) -> serde_json::Value {
        let projected = value.clone().into_json_value();
        if serde_json::to_vec(&projected).map_or(true, |value| value.len() > 4096) {
            serde_json::json!({"unavailable":"exceeds_projection_bound"})
        } else {
            projected
        }
    }
    fn visit(
        value: &Value,
        depth: usize,
        remaining: &mut usize,
        output: &mut Vec<serde_json::Value>,
    ) {
        if depth > 64 || output.len() >= 256 || *remaining == 0 {
            return;
        }
        *remaining -= 1;
        match value {
            Value::Array(items) => {
                for item in items.iter() {
                    visit(item, depth + 1, remaining, output);
                }
            }
            Value::Object(object) => {
                if let Some(Value::String(operator)) = object.get("operator") {
                    let attributes = match object.get("attributes") {
                        Some(Value::Object(attributes)) => Some(attributes),
                        _ => None,
                    };
                    let metrics = match object.get("metrics") {
                        Some(Value::Object(metrics)) => Some(metrics),
                        _ => None,
                    };
                    let selected = attributes
                        .map(|attrs| {
                            [
                                "direction",
                                "index",
                                "access",
                                "offset",
                                "tables",
                                "output",
                                "limit",
                                "order_pushdown",
                                "predicate",
                                "predicate_scope",
                            ]
                            .into_iter()
                            .filter_map(|key| attrs.get(key).map(|value| (key, fact(value))))
                            .collect::<BTreeMap<_, _>>()
                        })
                        .unwrap_or_default();
                    let counters = metrics
                        .map(|metrics| {
                            [
                                "output_rows",
                                "output_batches",
                                "elapsed_ns",
                                "skipped_rows",
                                "edges_scanned",
                                "cache_hits",
                                "cache_misses",
                                "block_hits",
                                "delta_hits",
                                "records_fetched",
                                "props_evals",
                                "props_fallbacks",
                            ]
                            .into_iter()
                            .filter_map(|key| metrics.get(key).map(|value| (key, fact(value))))
                            .collect::<BTreeMap<_, _>>()
                        })
                        .unwrap_or_default();
                    let graph_counters_observed = [
                        "edges_scanned",
                        "cache_hits",
                        "cache_misses",
                        "block_hits",
                        "delta_hits",
                        "records_fetched",
                        "props_evals",
                        "props_fallbacks",
                    ]
                    .into_iter()
                    .any(|key| counters.contains_key(key));
                    if !matches!(
                        operator.as_str(),
                        "Values"
                            | "Scan"
                            | "Filter"
                            | "Sort"
                            | "TopK"
                            | "Limit"
                            | "Project"
                            | "Output"
                            | "IndexScan"
                    ) || operator.contains("Graph")
                        || matches!(
                            operator.as_str(),
                            "IndexScan" | "Filter" | "Sort" | "TopK" | "Limit"
                        )
                        || !selected.is_empty()
                        || (operator == "Values" && graph_counters_observed)
                    {
                        let bounded_operator = operator.chars().take(128).collect::<String>();
                        output.push(serde_json::json!({"operator":bounded_operator,"attributes":selected,"observed_counters":counters,
                            "folded_blocks_observed":counters.contains_key("block_hits"),"delta_entries_observed":counters.contains_key("delta_hits"),
                            "predicate_interpretation":if operator=="Filter" {"downstream Filter; predicate not pushed into graph scan"} else {match attributes.and_then(|attrs|attrs.get("predicate_scope")) {
                                Some(Value::String(scope)) if scope=="key"=>"key predicate before edge record fetch",
                                Some(Value::String(scope)) if scope=="payload"=>"inline payload predicate with record fallback when payload is unusable",
                                Some(Value::String(scope)) if scope=="record"=>"predicate requires edge record evaluation",
                                _=>"no native predicate placement observation"}}}));
                    }
                }
                for child in object.values() {
                    visit(child, depth + 1, remaining, output);
                }
            }
            _ => {}
        }
    }
    let mut output = Vec::new();
    visit(value, 0, &mut 2048, &mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_plan_unknown_operators_preserve_missing_counters() {
        let mut operator = surrealdb_types::Object::new();
        operator.insert("operator", Value::String("FutureSupplierOperator".into()));
        let projected = native_plan_observations(&Value::Object(operator));
        assert_eq!(projected.len(), 1);
        assert_eq!(projected[0]["operator"], "FutureSupplierOperator");
        assert_eq!(projected[0]["observed_counters"], serde_json::json!({}));
    }
    #[test]
    fn native_explanation_keeps_unknown_identity_and_refuses_incomplete_outcome() {
        let identity = NativeExplanationIdentity {
            issuer: "selected-issuer".into(),
            data_cut: serde_json::json!({"generation":17}),
            server_version: None,
            server_recipe: None,
            server_invocation: None,
        };
        let mut explanation=NativeQueryExplanation::new(identity,serde_json::json!({"sql":"SELECT * FROM run WHERE generation=$generation","bindings":{"generation":17}}),&Value::None,false).unwrap();
        assert!(explanation.identity.server_version.is_none());
        assert!(explanation.response.is_none());
        assert!(!explanation.analyzed);
        assert!(
            explanation
                .attach_response(
                    &crate::transport::client::IndexedResults::default(),
                    NativeExplanationResponseKind::Inspection
                )
                .is_err()
        );
        assert!(explanation.response.is_none());
        explanation.query = serde_json::json!({"oversized":"x".repeat(256*1024)});
        assert!(
            NativeQueryExplanation::new(
                explanation.identity,
                explanation.query,
                &Value::None,
                false
            )
            .is_err()
        );
    }
}
