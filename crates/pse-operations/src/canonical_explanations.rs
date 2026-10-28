// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded metadata neighborhoods over existing canonical relations, without payload acquisition.

use crate::{
    canonical::transport::{original_deadline, within_clock},
    canonical::{
        CanonicalError, CanonicalStore, MetadataResponse, REQUEST_TIMEOUT, protected_query,
    },
    generated::surreal as wire,
};
use pse_model::study::{ActionKind, PointAction, Refusal, WaitReason};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use surrealdb_types::{Object, Value};

/// The exact selected graph owner and revision, never a moving current pointer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GraphAnchor {
    /// One active managed occurrence at its concurrency revision.
    StudyPoint {
        /// Canonical point identity.
        point: String,
        /// Exact point concurrency revision.
        revision: u64,
        /// Exact study authority generation, including cancellation.
        generation: u64,
    },
    /// One retained scientific run at its original source revision.
    Run {
        /// Canonical run identity.
        run: String,
        /// Exact primary source revision.
        revision: String,
    },
    /// One node of an active immutable derived analysis.
    AnalysisNode {
        /// Canonical analysis identity.
        analysis: String,
        /// Selected node identity, including isolates.
        node: String,
        /// Exact analysis primary source revision.
        revision: String,
    },
    /// A discovered artifact header; content still requires protected acquisition.
    Artifact {
        /// Independent artifact identity.
        artifact: String,
        /// Exact artifact primary source revision.
        revision: String,
    },
}
/// The requested local question; no transitive or all-path completeness is implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GraphScope {
    /// Immediate study prerequisites and the shared policy's observed blocker.
    ImmediateBlockers,
    /// Immediate predecessor/source references.
    RecordedSources,
    /// Immediate dependent/consumer references.
    RetainedConsumers,
}
/// A continuation binds its exact owner, revision and question.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GraphContinuation {
    /// Frozen selection to which the cursor belongs.
    pub anchor: GraphAnchor,
    /// Exact neighborhood question.
    pub scope: GraphScope,
    /// Last retained ordering key; artifact consumers order by analysis identity.
    pub after: String,
}
/// One bounded semantic graph query, separate from native physical-plan observations.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CanonicalGraphRequest {
    /// Required standalone graph-request interpretation.
    pub version: pse_model::document::Version<1>,
    /// Exact selected owner/revision.
    pub anchor: GraphAnchor,
    /// Local neighborhood question.
    pub scope: GraphScope,
    /// Returned edges, from one through 64; the native query retrieves one additional row.
    pub limit: u32,
    /// Optional exact-selection continuation.
    pub continuation: Option<GraphContinuation>,
}
/// A metadata node; absent revision means that endpoint's revision was not fetched.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GraphNode {
    /// Exact table-qualified node identity.
    pub identity: String,
    /// Source declaration/owner role.
    pub role: String,
    /// Selected source or concurrency revision, when actually observed.
    pub revision: Option<String>,
}
/// An existing edge's semantic role, preserving parallel relations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GraphEdgeRole {
    /// Ordering prerequisite.
    Ordering,
    /// Scientific usable-result prerequisite.
    UsableResult,
    /// Authored continuation-seed prerequisite.
    ContinuationSeed,
    /// Recorded run-to-source relation.
    RecordedSource,
    /// Retained analysis-to-artifact input relation.
    AnalysisInput,
    /// Authored analysis relation; connectivity alone establishes no numerical semantics.
    AnalysisRelation {
        /// Original authored classification.
        relation: String,
    },
}
/// Exact recorded evidence locator, not a grant to read protected bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GraphEvidence {
    /// Independent artifact identity.
    pub artifact: String,
    /// Optional original member position.
    pub block: Option<u64>,
    /// Optional original row position.
    pub row: Option<u64>,
    /// Optional original field identity.
    pub field: Option<String>,
}
/// One original edge, without client-side graph reconstruction or collapsed parallel kinds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GraphEdge {
    /// Exact table-qualified retained relation identity.
    pub identity: String,
    /// Original directed source.
    pub source: String,
    /// Original directed target.
    pub target: String,
    /// Original semantic role.
    pub role: GraphEdgeRole,
    /// Recorded protected evidence locator, if any; no acquisition occurs here.
    pub evidence: Option<GraphEvidence>,
}
/// Bounded metadata response with explicit local completeness and revision-bound continuation.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CanonicalGraphDocument {
    /// Required standalone graph-response interpretation.
    pub version: pse_model::document::Version<1>,
    /// Frozen selected owner/revision.
    pub anchor: GraphAnchor,
    /// Requested local question.
    pub scope: GraphScope,
    /// Selected node plus observed neighboring identities, preserving isolates.
    pub nodes: Vec<GraphNode>,
    /// At most the requested count of retained original edges.
    pub edges: Vec<GraphEdge>,
    /// Shared study policy action, when this is a study occurrence.
    pub decision: Option<PointAction>,
    /// One blocker actually selected by shared policy, not an assertion that all blockers were enumerated.
    pub observed_blocker: Option<String>,
    /// Native result rows decoded, including the page-plus-one sentinel; not physical KV work.
    pub examined_rows: u32,
    /// Whether the requested immediate neighborhood was exhausted.
    pub complete: bool,
    /// Explicit next page, never a silent truncated success.
    pub continuation: Option<GraphContinuation>,
    /// Scope and evidence limits, including protected acquisition.
    pub evidence_scope: String,
}

fn invalid(message: &str) -> CanonicalError {
    CanonicalError::Configuration(message.into())
}
fn node(table: &str, key: &str) -> String {
    format!("{table}:{key}")
}

impl CanonicalGraphRequest {
    fn validate(&self) -> Result<&str, CanonicalError> {
        if !(1..=64).contains(&self.limit) {
            return Err(CanonicalError::PayloadLimit);
        }
        let compatible = matches!(
            (&self.anchor, self.scope),
            (GraphAnchor::StudyPoint { .. }, _)
                | (GraphAnchor::Run { .. }, GraphScope::RecordedSources)
                | (GraphAnchor::Artifact { .. }, GraphScope::RetainedConsumers)
                | (
                    GraphAnchor::AnalysisNode { .. },
                    GraphScope::RecordedSources | GraphScope::RetainedConsumers
                )
        );
        if !compatible {
            return Err(invalid(
                "graph scope is incompatible with its selected owner",
            ));
        }
        if let Some(cursor) = &self.continuation {
            if cursor.anchor != self.anchor || cursor.scope != self.scope {
                return Err(invalid(
                    "graph continuation belongs to another selected revision or scope",
                ));
            }
            if cursor.after.is_empty() || cursor.after.len() > 4096 {
                return Err(CanonicalError::PayloadLimit);
            }
            Ok(&cursor.after)
        } else {
            Ok("")
        }
    }
    /// Actual native query and safe selected bindings; dispatch and plans consume these same builders.
    pub fn description(&self) -> Result<serde_json::Value, CanonicalError> {
        let after = self.validate()?;
        Ok(
            serde_json::json!({"sql":self.sql(),"bindings":self.bindings(after),
            "source":{"path":file!(),"owner":"CanonicalGraphRequest::sql and bindings"},
            "scope":"one bound native relation neighborhood; physical graph plan is separate from semantic completeness"}),
        )
    }
    fn bindings(&self, after: &str) -> BTreeMap<String, Value> {
        let mut bindings = BTreeMap::from([("after".into(), Value::String(after.into()))]);
        match &self.anchor {
            GraphAnchor::StudyPoint { point, .. } => {
                bindings.insert("point".into(), Value::String(point.clone()));
            }
            GraphAnchor::Run { run, .. } => {
                bindings.insert("run".into(), Value::String(run.clone()));
            }
            GraphAnchor::Artifact { artifact, .. } => {
                bindings.insert("artifact".into(), Value::String(artifact.clone()));
            }
            GraphAnchor::AnalysisNode { analysis, node, .. } => {
                bindings.insert("analysis".into(), Value::String(analysis.clone()));
                bindings.insert("node".into(), Value::String(node.clone()));
            }
        }
        bindings
    }
    fn sql(&self) -> String {
        let source = match (&self.anchor, self.scope) {
            (GraphAnchor::StudyPoint { .. }, GraphScope::RetainedConsumers) => {
                "SELECT id,in,out,key,study,dependent,predecessor,kind FROM canonical_study_dependencies WITH INDEX study_dependents WHERE predecessor=$point AND key>$after ORDER BY key"
            }
            (GraphAnchor::StudyPoint { .. }, _) => {
                "SELECT id,in,out,key,study,dependent,predecessor,kind FROM canonical_study_dependencies WITH INDEX study_dependencies WHERE dependent=$point AND key>$after ORDER BY key"
            }
            (GraphAnchor::Run { .. }, _) => {
                "SELECT id,in,out,key,run,revision FROM canonical_run_sources WITH INDEX run_source_cleanup WHERE run=$run AND key>$after ORDER BY key"
            }
            (GraphAnchor::Artifact { .. }, _) => {
                "SELECT id,in,out,key,analysis,artifact FROM canonical_analysis_inputs WITH INDEX analysis_input_artifact WHERE artifact=$artifact AND analysis>$after ORDER BY analysis"
            }
            (GraphAnchor::AnalysisNode { .. }, GraphScope::RecordedSources) => {
                "SELECT id,in,out,key,analysis,source,target,kind,evidence_artifact,evidence_block,evidence_row,evidence_field FROM canonical_analysis_edges WITH INDEX analysis_edge_target WHERE target=$node AND key>$after ORDER BY key"
            }
            (GraphAnchor::AnalysisNode { .. }, _) => {
                "SELECT id,in,out,key,analysis,source,target,kind,evidence_artifact,evidence_block,evidence_row,evidence_field FROM canonical_analysis_edges WITH INDEX analysis_edge_source WHERE source=$node AND key>$after ORDER BY key"
            }
        };
        // Only the validated numeric page cap is rendered; identities and cursor
        // remain bound values. This also makes the pushed cap explicit in EXPLAIN.
        format!("{source} LIMIT {};", u64::from(self.limit) + 1)
    }
}

/// Project native operator facts without turning missing positive-only counters into zeroes.
fn graph_plan_observations(value: &Value) -> Vec<serde_json::Value> {
    fn visit(value: &Value, depth: usize, output: &mut Vec<serde_json::Value>) {
        if depth > 64 || output.len() >= 256 {
            return;
        }
        match value {
            Value::Array(items) => {
                for item in items.iter() {
                    visit(item, depth + 1, output);
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
                            .filter_map(|key| {
                                attrs
                                    .get(key)
                                    .map(|value| (key, value.clone().into_json_value()))
                            })
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
                            .filter_map(|key| {
                                metrics
                                    .get(key)
                                    .map(|value| (key, value.clone().into_json_value()))
                            })
                            .collect::<BTreeMap<_, _>>()
                        })
                        .unwrap_or_default();
                    if operator.contains("Graph")
                        || matches!(
                            operator.as_str(),
                            "IndexScan" | "Filter" | "Sort" | "TopK" | "Limit"
                        )
                        || !selected.is_empty()
                    {
                        output.push(serde_json::json!({"operator":operator,"attributes":selected,"observed_counters":counters,
                            "folded_blocks_observed":counters.contains_key("block_hits"),"delta_entries_observed":counters.contains_key("delta_hits"),
                            "predicate_interpretation":if operator=="Filter" {"downstream Filter; predicate not pushed into graph scan"} else {match attributes.and_then(|attrs|attrs.get("predicate_scope")) {
                                Some(Value::String(scope)) if scope=="key"=>"key predicate before edge record fetch",
                                Some(Value::String(scope)) if scope=="payload"=>"inline payload predicate with record fallback when payload is unusable",
                                Some(Value::String(scope)) if scope=="record"=>"predicate requires edge record evaluation",
                                _=>"no native predicate placement observation"}}}));
                    }
                }
                for child in object.values() {
                    visit(child, depth + 1, output);
                }
            }
            _ => {}
        }
    }
    let mut output = Vec::new();
    visit(value, 0, &mut output);
    output
}

impl CanonicalStore {
    async fn graph_anchor(
        &self,
        anchor: &GraphAnchor,
    ) -> Result<(GraphNode, Option<PointAction>, Option<String>), CanonicalError> {
        match anchor {
            GraphAnchor::StudyPoint {
                point,
                revision,
                generation,
            } => {
                let scope = self.study_scope(point).await?;
                if scope.point().revision != *revision || scope.generation() != *generation {
                    return Err(invalid("selected graph point revision changed"));
                }
                let decision = scope.action(None)?;
                let predecessor = match decision.kind {
                    ActionKind::Wait(WaitReason::Dependency { predecessor })
                    | ActionKind::Refuse(Refusal::DependencyUnusable { predecessor }) => {
                        Some(predecessor)
                    }
                    _ => None,
                };
                let blocker = predecessor.and_then(|key| {
                    scope
                        .predecessors()
                        .iter()
                        .find(|row| row.occurrence == u64::from(key.0))
                        .map(|row| node("canonical_study_points", &row.key))
                });
                Ok((
                    GraphNode {
                        identity: node("canonical_study_points", point),
                        role: "study_point".into(),
                        revision: Some(revision.to_string()),
                    },
                    Some(decision),
                    blocker,
                ))
            }
            GraphAnchor::Run { run, revision } => {
                let row = self
                    .canonical_run(run)
                    .await?
                    .ok_or(CanonicalError::IncompleteResponse)?;
                if row.revision != *revision {
                    return Err(invalid("selected graph run revision changed"));
                }
                Ok((
                    GraphNode {
                        identity: node("canonical_runs", run),
                        role: "run".into(),
                        revision: Some(row.revision),
                    },
                    None,
                    None,
                ))
            }
            GraphAnchor::Artifact { artifact, revision } => {
                let row = self.artifact(artifact).await?;
                if row.revision != *revision {
                    return Err(invalid("selected graph artifact revision changed"));
                }
                Ok((
                    GraphNode {
                        identity: node("canonical_artifacts", artifact),
                        role: "artifact_metadata_only".into(),
                        revision: Some(row.revision),
                    },
                    None,
                    None,
                ))
            }
            GraphAnchor::AnalysisNode {
                analysis,
                node: selected,
                revision,
            } => {
                let row = self.analysis(analysis).await?;
                if row.revision != *revision {
                    return Err(invalid("selected graph analysis revision changed"));
                }
                let mut response=protected_query("canonical_explanations::node",||Ok(self.db.metadata_query(MetadataResponse::Header,"SELECT key,analysis,semantic,kind FROM ONLY type::record('canonical_analysis_nodes',$node);").bind(("node",selected.clone())))).await?;
                let actual = wire::decode_canonical_analysis_nodes(
                    response
                        .take::<Option<Object>>(0)?
                        .ok_or(CanonicalError::IncompleteResponse)?,
                )?;
                if actual.analysis != *analysis {
                    return Err(invalid("selected graph node belongs to another analysis"));
                }
                Ok((
                    GraphNode {
                        identity: node("canonical_analysis_nodes", selected),
                        role: actual.kind,
                        revision: Some(row.revision),
                    },
                    None,
                    None,
                ))
            }
        }
    }

    /// Explain the same bound graph query; experimental ANALYZE executes and drains its inner plan.
    pub async fn inspect_graph_explanation(
        &self,
        request: &CanonicalGraphRequest,
        experiment: bool,
    ) -> Result<serde_json::Value, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async {
            let after=request.validate()?;
            let before=self.graph_anchor(&request.anchor).await?.0;
            let mut plans=BTreeMap::new();
            let mut observations=BTreeMap::new();
            let mut modes=vec![("static","EXPLAIN FORMAT JSON")];
            if experiment {modes.push(("analyze","EXPLAIN ANALYZE FORMAT JSON"));}
            for (name,prefix) in modes {
                let sql=format!("{prefix} {}",request.sql());
                let mut response=protected_query("canonical_explanations::plan",||Ok(self.db.metadata_query(MetadataResponse::DiagnosticPlan,sql.clone()).bind(request.bindings(after)))).await?;
                let plan=response.take::<Value>(0)?;
                observations.insert(name,graph_plan_observations(&plan));
                plans.insert(name,plan);
            }
            if self.graph_anchor(&request.anchor).await?.0!=before {return Err(invalid("graph selection changed during explanation"));}
            Ok(serde_json::json!({"query":request.description()?,"namespace":self.namespace(),"database":self.database(),"experimental_execution":experiment,"raw_plans":plans,"operator_observations":observations,"operator_projection_limits":{"operators":256,"nesting":64},
                "scope":"outer operator evidence only; missing cache counters do not establish an instrumentation gap or a miss; no scientific or result-completeness inference"}))
        })).await
    }

    /// Read one original relation neighborhood under current receiving authority; never acquires payloads.
    pub async fn graph_explanation(
        &self,
        request: &CanonicalGraphRequest,
    ) -> Result<CanonicalGraphDocument, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let after=request.validate()?;
            let (anchor,decision,blocker)=self.graph_anchor(&request.anchor).await?;
            let bindings=request.bindings(after);
            let mut response=protected_query("canonical_explanations::neighborhood",||Ok(self.db.metadata_query(MetadataResponse::Page,request.sql()).bind(bindings.clone()))).await?;
            let rows=response.take::<Vec<Object>>(0)?;
            if rows.len()>request.limit as usize+1 { return Err(CanonicalError::PayloadLimit); }
            let examined_rows=u32::try_from(rows.len()).map_err(|_|CanonicalError::PayloadLimit)?;
            let mut entries=decode_page(request, after, rows)?;
            let more=entries.len()>request.limit as usize;
            entries.truncate(request.limit as usize);
            let mut nodes=BTreeMap::from([(anchor.identity.clone(),anchor.clone())]);
            for (_,edge) in &entries {
                for endpoint in [&edge.source,&edge.target] {
                    nodes.entry(endpoint.clone()).or_insert_with(||GraphNode{identity:endpoint.clone(),role:"recorded_neighbor".into(),revision:None});
                }
            }
            // An endpoint observed under a changed selection is never promoted to a coherent page.
            let (again,_,_)=self.graph_anchor(&request.anchor).await?;
            if again!=anchor { return Err(invalid("graph selection changed during observation")); }
            let continuation=if more {Some(GraphContinuation{anchor:request.anchor.clone(),scope:request.scope,after:entries.last().ok_or(CanonicalError::IncompleteResponse)?.0.clone()})}else{None};
            let document=CanonicalGraphDocument{version:pse_model::document::Version,anchor:request.anchor.clone(),scope:request.scope,nodes:nodes.into_values().collect(),edges:entries.into_iter().map(|(_,edge)|edge).collect(),decision,observed_blocker:blocker,examined_rows,complete:!more,continuation,evidence_scope:"one immediate neighborhood only; recorded connectivity is not numerical dependence or scientific usability; current metadata is rechecked per page, not an atomic graph snapshot; artifact/evidence bytes require existing protected acquisition".into()};
            if serde_json::to_vec(&document).map_err(|error|CanonicalError::Configuration(error.to_string()))?.len()>256*1024 { return Err(CanonicalError::PayloadLimit); }
            Ok(document)
        })).await
    }
}

// Validate the full native page before dropping its sentinel. A malformed sentinel cannot
// turn a corrupt range into an apparently complete or resumable neighborhood.
fn decode_page(
    request: &CanonicalGraphRequest,
    after: &str,
    rows: Vec<Object>,
) -> Result<Vec<(String, GraphEdge)>, CanonicalError> {
    let mut previous = after.to_owned();
    rows.into_iter()
        .map(|row| {
            let entry = decode_edge(request, row)?;
            if entry.0.as_str() <= previous.as_str() {
                return Err(invalid(
                    "stored graph ordering is not strictly beyond its cursor",
                ));
            }
            previous.clone_from(&entry.0);
            Ok(entry)
        })
        .collect()
}

fn decode_edge(
    request: &CanonicalGraphRequest,
    row: Object,
) -> Result<(String, GraphEdge), CanonicalError> {
    Ok(match &request.anchor {
        GraphAnchor::StudyPoint { point, .. } => {
            let edge = wire::decode_canonical_study_dependencies(row)?;
            let endpoint = if request.scope == GraphScope::RetainedConsumers {
                &edge.predecessor
            } else {
                &edge.dependent
            };
            if endpoint != point {
                return Err(invalid(
                    "stored study edge is outside selected neighborhood",
                ));
            }
            let role = match edge.kind.as_str() {
                "ordering" => GraphEdgeRole::Ordering,
                "usable" => GraphEdgeRole::UsableResult,
                "seed" => GraphEdgeRole::ContinuationSeed,
                _ => return Err(invalid("unrecognized stored study edge role")),
            };
            (
                edge.key.clone(),
                GraphEdge {
                    identity: node("canonical_study_dependencies", &edge.key),
                    source: node("canonical_study_points", &edge.dependent),
                    target: node("canonical_study_points", &edge.predecessor),
                    role,
                    evidence: None,
                },
            )
        }
        GraphAnchor::Run { run, .. } => {
            let edge = wire::decode_canonical_run_sources(row)?;
            if edge.run != *run {
                return Err(invalid("stored source edge belongs to another run"));
            }
            (
                edge.key.clone(),
                GraphEdge {
                    identity: node("canonical_run_sources", &edge.key),
                    source: node("canonical_runs", &edge.run),
                    target: node("canonical_revisions", &edge.revision),
                    role: GraphEdgeRole::RecordedSource,
                    evidence: None,
                },
            )
        }
        GraphAnchor::Artifact { artifact, .. } => {
            let edge = wire::decode_canonical_analysis_inputs(row)?;
            if edge.artifact != *artifact {
                return Err(invalid("stored input edge belongs to another artifact"));
            }
            (
                edge.analysis.clone(),
                GraphEdge {
                    identity: node("canonical_analysis_inputs", &edge.key),
                    source: node("canonical_analyses", &edge.analysis),
                    target: node("canonical_artifacts", &edge.artifact),
                    role: GraphEdgeRole::AnalysisInput,
                    evidence: Some(GraphEvidence {
                        artifact: edge.artifact,
                        block: None,
                        row: None,
                        field: None,
                    }),
                },
            )
        }
        GraphAnchor::AnalysisNode {
            analysis,
            node: selected,
            ..
        } => {
            let edge = wire::decode_canonical_analysis_edges(row)?;
            let endpoint = if request.scope == GraphScope::RecordedSources {
                &edge.target
            } else {
                &edge.source
            };
            if edge.analysis != *analysis || endpoint != selected {
                return Err(invalid(
                    "stored analysis edge is outside selected analysis neighborhood",
                ));
            }
            (
                edge.key.clone(),
                GraphEdge {
                    identity: node("canonical_analysis_edges", &edge.key),
                    source: node("canonical_analysis_nodes", &edge.source),
                    target: node("canonical_analysis_nodes", &edge.target),
                    role: GraphEdgeRole::AnalysisRelation {
                        relation: edge.kind,
                    },
                    evidence: edge.evidence_artifact.map(|artifact| GraphEvidence {
                        artifact,
                        block: edge.evidence_block,
                        row: edge.evidence_row,
                        field: edge.evidence_field,
                    }),
                },
            )
        }
    })
}

#[cfg(all(test, feature = "canonical-tests"))]
#[allow(
    clippy::expect_used,
    reason = "selected optional evidence must fail on missing prerequisites"
)]
pub(crate) fn operator_evidence_directory() -> String {
    let directory = std::env::var("PSE_DIAGNOSTIC_EVIDENCE")
        .expect("selected operator evidence requires PSE_DIAGNOSTIC_EVIDENCE");
    assert!(
        std::env::var_os("PSE_CANONICAL_TOOLS").is_some(),
        "selected operator evidence requires its captured PSE_CANONICAL_TOOLS executable"
    );
    directory
}

#[cfg(all(test, feature = "canonical-tests"))]
#[allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "bounded actual CLI fixture controls fail on any missing input, failed child or timeout"
)]
pub(crate) async fn qualify_operator(
    store: &CanonicalStore,
    kind: &str,
    input: &serde_json::Value,
) {
    let directory = operator_evidence_directory();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let request =
        std::path::Path::new(&directory).join(format!("{}-{kind}-request.json", store.database()));
    let evidence =
        std::path::Path::new(&directory).join(format!("{}-{kind}-operator.json", store.database()));
    std::fs::write(&request, serde_json::to_vec(input).unwrap()).unwrap();
    let mut child = tokio::process::Command::new(root.join("scripts/pse-env"))
        .current_dir(root)
        .arg("--native=")
        .arg("--")
        .arg(root.join(".venv/bin/python"))
        .args([
            "-m",
            "scripts.tests.canonical_inspection_check",
            "--kind",
            kind,
            "--database",
            store.database(),
            "--state",
        ])
        .arg(store.deployment_state())
        .arg("--request")
        .arg(&request)
        .arg("--evidence")
        .arg(&evidence)
        .kill_on_drop(true)
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    match tokio::time::timeout(std::time::Duration::from_secs(45), child.wait()).await {
        Ok(status) => assert!(
            status.unwrap().success(),
            "actual operator failed; evidence {evidence:?}"
        ),
        Err(_) => {
            child.kill().await.unwrap();
            panic!("actual operator exceeded its finite child budget");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical_codec as codec;

    #[test]
    fn graph_requests_bind_values_and_refuse_cross_revision_continuations() {
        let anchor = GraphAnchor::StudyPoint {
            point: "point';DELETE canonical_studies;".into(),
            revision: 7,
            generation: 3,
        };
        let mut request = CanonicalGraphRequest {
            version: pse_model::document::Version,
            anchor: anchor.clone(),
            scope: GraphScope::RecordedSources,
            limit: 1,
            continuation: None,
        };
        assert!(!request.sql().contains("DELETE"));
        assert!(request.sql().ends_with(" LIMIT 2;"));
        assert!(!request.bindings("").contains_key("maximum"));
        request.continuation = Some(GraphContinuation {
            anchor: GraphAnchor::StudyPoint {
                point: "point';DELETE canonical_studies;".into(),
                revision: 8,
                generation: 3,
            },
            scope: request.scope,
            after: "edge".into(),
        });
        assert!(request.validate().is_err());
        request.continuation = None;
        request.limit = 0;
        assert!(request.validate().is_err());
    }

    #[test]
    fn graph_plan_keeps_payload_record_fallback_and_missing_counters_distinct() {
        let mut attrs = Object::new();
        attrs.insert("direction", Value::String("out".into()));
        attrs.insert("predicate_scope", Value::String("payload".into()));
        let mut metrics = Object::new();
        metrics.insert("block_hits", codec::encode_int(3).unwrap());
        let mut object = Object::new();
        object.insert("operator", Value::String("GraphEdgeScan".into()));
        object.insert("attributes", Value::Object(attrs));
        object.insert("metrics", Value::Object(metrics));
        let observed = graph_plan_observations(&Value::Object(object.clone()));
        assert_eq!(observed[0]["folded_blocks_observed"], true);
        assert!(
            !observed[0]["observed_counters"]
                .as_object()
                .unwrap()
                .contains_key("cache_misses")
        );
        assert!(
            observed[0]["predicate_interpretation"]
                .as_str()
                .unwrap()
                .contains("record fallback")
        );
        object.insert("operator", Value::String("Filter".into()));
        let filtered = graph_plan_observations(&Value::Object(object));
        assert!(
            filtered[0]["predicate_interpretation"]
                .as_str()
                .unwrap()
                .contains("downstream Filter")
        );
    }

    #[test]
    fn graph_seek_shapes_use_named_owner_prefixes_without_offset() {
        for (anchor, scope, index, predicate) in [
            (
                GraphAnchor::StudyPoint {
                    point: "p".into(),
                    revision: 0,
                    generation: 0,
                },
                GraphScope::ImmediateBlockers,
                "study_dependencies",
                "dependent=$point",
            ),
            (
                GraphAnchor::StudyPoint {
                    point: "p".into(),
                    revision: 0,
                    generation: 0,
                },
                GraphScope::RecordedSources,
                "study_dependencies",
                "dependent=$point",
            ),
            (
                GraphAnchor::StudyPoint {
                    point: "p".into(),
                    revision: 0,
                    generation: 0,
                },
                GraphScope::RetainedConsumers,
                "study_dependents",
                "predecessor=$point",
            ),
            (
                GraphAnchor::Run {
                    run: "r".into(),
                    revision: "v".into(),
                },
                GraphScope::RecordedSources,
                "run_source_cleanup",
                "run=$run",
            ),
            (
                GraphAnchor::Artifact {
                    artifact: "a".into(),
                    revision: "v".into(),
                },
                GraphScope::RetainedConsumers,
                "analysis_input_artifact",
                "artifact=$artifact",
            ),
            (
                GraphAnchor::AnalysisNode {
                    analysis: "a".into(),
                    node: "n".into(),
                    revision: "v".into(),
                },
                GraphScope::RecordedSources,
                "analysis_edge_target",
                "target=$node",
            ),
            (
                GraphAnchor::AnalysisNode {
                    analysis: "a".into(),
                    node: "n".into(),
                    revision: "v".into(),
                },
                GraphScope::RetainedConsumers,
                "analysis_edge_source",
                "source=$node",
            ),
        ] {
            let request = CanonicalGraphRequest {
                version: pse_model::document::Version,
                anchor,
                scope,
                limit: 64,
                continuation: None,
            };
            let sql = request.sql();
            assert!(
                sql.contains(&format!("WITH INDEX {index} WHERE {predicate}")),
                "{sql}"
            );
            assert!(sql.ends_with(" LIMIT 65;"));
            assert!(
                !sql.contains("START")
                    && !sql.contains("RETURN")
                    && !sql.contains("analysis=$analysis")
            );
            assert!(!request.bindings("").contains_key("maximum"));
        }
    }

    fn analysis_edge_row(key: &str, analysis: &str, source: &str, target: &str) -> Object {
        let mut row = Object::new();
        for (name, value) in [
            ("key", key),
            ("analysis", analysis),
            ("source", source),
            ("target", target),
            ("kind", "incidence"),
        ] {
            row.insert(name, Value::String(value.into()));
        }
        row.insert("id", codec::record_link("canonical_analysis_edges", key));
        row.insert("in", codec::record_link("canonical_analysis_nodes", source));
        row.insert(
            "out",
            codec::record_link("canonical_analysis_nodes", target),
        );
        row
    }

    #[test]
    fn graph_page_refuses_corrupt_sentinel_cross_analysis_endpoints_and_ordering() {
        for scope in [GraphScope::RecordedSources, GraphScope::RetainedConsumers] {
            let selected = if scope == GraphScope::RecordedSources {
                "target"
            } else {
                "source"
            };
            let request = CanonicalGraphRequest {
                version: pse_model::document::Version,
                anchor: GraphAnchor::AnalysisNode {
                    analysis: "analysis".into(),
                    node: selected.into(),
                    revision: "revision".into(),
                },
                scope,
                limit: 1,
                continuation: None,
            };
            let valid = analysis_edge_row("a", "analysis", "source", "target");
            let mut wrong_link = analysis_edge_row("b", "analysis", "source", "target");
            wrong_link.insert(
                "out",
                codec::record_link("canonical_analysis_nodes", "other"),
            );
            let mut wrong_id = analysis_edge_row("b", "analysis", "source", "target");
            wrong_id.insert(
                "id",
                codec::record_link("canonical_analysis_edges", "other"),
            );
            for corrupt in [
                analysis_edge_row("b", "other-analysis", "source", "target"),
                analysis_edge_row("b", "analysis", "other-source", "other-target"),
                analysis_edge_row("a", "analysis", "source", "target"),
                wrong_link,
                wrong_id,
            ] {
                assert!(decode_page(&request, "", vec![valid.clone(), corrupt]).is_err());
            }
            assert!(decode_page(&request, "a", vec![valid.clone()]).is_err());
            assert!(decode_page(&request, "z", vec![valid.clone()]).is_err());
            assert!(
                decode_page(
                    &request,
                    "",
                    vec![
                        valid,
                        analysis_edge_row("b", "analysis", "source", "target")
                    ]
                )
                .is_ok()
            );
        }
    }

    #[test]
    fn graph_index_observations_retain_native_access_offset_and_real_counters() {
        let mut attrs = Object::new();
        for (name, value) in [
            ("index", "analysis_edge_source"),
            ("access", "['source'] MoreThan 'a'"),
            ("direction", "Forward"),
            ("limit", "2"),
            ("offset", "7"),
        ] {
            attrs.insert(name, Value::String(value.into()));
        }
        let mut metrics = Object::new();
        for (name, value) in [
            ("output_rows", 2),
            ("output_batches", 1),
            ("elapsed_ns", 30),
            ("records_fetched", 0),
        ] {
            metrics.insert(name, codec::encode_int(value).unwrap());
        }
        let mut plan = Object::new();
        plan.insert("operator", Value::String("IndexScan".into()));
        plan.insert("attributes", Value::Object(attrs));
        plan.insert("metrics", Value::Object(metrics));
        let observations = graph_plan_observations(&Value::Object(plan));
        assert_eq!(
            observations[0]["attributes"]["index"],
            "analysis_edge_source"
        );
        assert_eq!(observations[0]["attributes"]["offset"], "7");
        assert_eq!(
            observations[0]["attributes"]["access"],
            "['source'] MoreThan 'a'"
        );
        assert_eq!(observations[0]["observed_counters"]["output_rows"], 2);
        assert_eq!(observations[0]["observed_counters"]["records_fetched"], 0);
        assert!(
            observations[0]["observed_counters"]
                .get("edges_scanned")
                .is_none()
        );
    }

    #[test]
    fn graph_parallel_roles_and_protected_locators_are_preserved() {
        let anchor = GraphAnchor::StudyPoint {
            point: "target".into(),
            revision: 0,
            generation: 0,
        };
        let edge = |key: &str, kind: &str| {
            let mut row = Object::new();
            for (name, value) in [
                ("key", key),
                ("study", "study"),
                ("dependent", "target"),
                ("predecessor", "source"),
                ("kind", kind),
            ] {
                row.insert(name, Value::String(value.into()));
            }
            row.insert("in", codec::record_link("canonical_study_points", "target"));
            row.insert(
                "out",
                codec::record_link("canonical_study_points", "source"),
            );
            decode_edge(
                &CanonicalGraphRequest {
                    version: pse_model::document::Version,
                    anchor: anchor.clone(),
                    scope: GraphScope::RecordedSources,
                    limit: 1,
                    continuation: None,
                },
                row,
            )
            .unwrap()
            .1
        };
        let ordering = edge("one", "ordering");
        let usable = edge("two", "usable");
        assert_eq!(ordering.source, usable.source);
        assert_eq!(ordering.target, usable.target);
        assert_ne!(ordering.role, usable.role);
        assert_ne!(ordering.identity, usable.identity);
        let artifact = GraphAnchor::Artifact {
            artifact: "protected".into(),
            revision: "revision".into(),
        };
        let mut row = Object::new();
        for (name, value) in [
            ("key", "input"),
            ("analysis", "consumer"),
            ("artifact", "protected"),
        ] {
            row.insert(name, Value::String(value.into()));
        }
        row.insert("in", codec::record_link("canonical_analyses", "consumer"));
        row.insert(
            "out",
            codec::record_link("canonical_artifacts", "protected"),
        );
        let (_, edge) = decode_edge(
            &CanonicalGraphRequest {
                version: pse_model::document::Version,
                anchor: artifact,
                scope: GraphScope::RetainedConsumers,
                limit: 1,
                continuation: None,
            },
            row,
        )
        .unwrap();
        assert_eq!(edge.source, "canonical_analyses:consumer");
        assert_eq!(edge.target, "canonical_artifacts:protected");
        assert_eq!(edge.evidence.unwrap().artifact, "protected");
    }
}

/// Independent expected edge lists drive first, selective, late and exhausted page controls.
#[cfg(all(test, feature = "canonical-tests"))]
#[allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "native plan and semantic controls fail on any mismatch"
)]
pub(crate) async fn qualify_graph_seek_pages(
    store: &CanonicalStore,
    request: &CanonicalGraphRequest,
    mut expected: Vec<(String, GraphEdge)>,
) {
    use surrealdb_types::ToSql;
    expected.sort_by(|left, right| left.0.cmp(&right.0));
    let (owner, field, index, ordering) = match (&request.anchor, request.scope) {
        (GraphAnchor::StudyPoint { point, .. }, GraphScope::RetainedConsumers) => {
            (point, "predecessor", "study_dependents", "key")
        }
        (GraphAnchor::StudyPoint { point, .. }, _) => {
            (point, "dependent", "study_dependencies", "key")
        }
        (GraphAnchor::Run { run, .. }, _) => (run, "run", "run_source_cleanup", "key"),
        (GraphAnchor::Artifact { artifact, .. }, _) => {
            (artifact, "artifact", "analysis_input_artifact", "analysis")
        }
        (GraphAnchor::AnalysisNode { node, .. }, GraphScope::RecordedSources) => {
            (node, "target", "analysis_edge_target", "key")
        }
        (GraphAnchor::AnalysisNode { node, .. }, _) => {
            (node, "source", "analysis_edge_source", "key")
        }
    };
    let mut cursors = vec![String::new(), "~~~~exhausted".into()];
    if !expected.is_empty() {
        for ordinal in [0, expected.len() / 2, expected.len() - 1] {
            cursors.push(expected[ordinal].0.clone());
        }
    }
    cursors.sort();
    cursors.dedup();
    for limit in [1, 64] {
        for after in &cursors {
            let mut selected = request.clone();
            selected.limit = limit;
            selected.continuation = (!after.is_empty()).then(|| GraphContinuation {
                anchor: selected.anchor.clone(),
                scope: selected.scope,
                after: after.clone(),
            });
            let remaining = expected
                .iter()
                .filter(|entry| entry.0 > *after)
                .collect::<Vec<_>>();
            let observed = store.graph_explanation(&selected).await.unwrap();
            assert_eq!(
                observed.edges,
                remaining
                    .iter()
                    .take(limit as usize)
                    .map(|entry| entry.1.clone())
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                observed.examined_rows as usize,
                remaining.len().min(limit as usize + 1)
            );
            assert_eq!(observed.complete, remaining.len() <= limit as usize);
            assert_eq!(
                observed
                    .continuation
                    .as_ref()
                    .map(|cursor| cursor.after.as_str()),
                if remaining.len() > limit as usize {
                    Some(remaining[limit as usize - 1].0.as_str())
                } else {
                    None
                }
            );
            let physical = store
                .inspect_graph_explanation(&selected, true)
                .await
                .unwrap();
            for mode in ["static", "analyze"] {
                let operators = physical["operator_observations"][mode].as_array().unwrap();
                assert!(
                    !operators.iter().any(|operator| matches!(
                        operator["operator"].as_str(),
                        Some("Filter" | "Sort" | "TopK" | "TableScan" | "GraphEdgeScan")
                    )),
                    "{physical}"
                );
                let scans = operators
                    .iter()
                    .filter(|operator| operator["operator"] == "IndexScan")
                    .collect::<Vec<_>>();
                assert_eq!(scans.len(), 1, "{physical}");
                let scan = scans[0];
                assert_eq!(scan["attributes"]["index"], index, "{physical}");
                assert_eq!(scan["attributes"]["direction"], "Forward", "{physical}");
                assert!(scan["attributes"].get("offset").is_none(), "{physical}");
                let owner_sql = Value::String(owner.clone()).to_sql();
                let after_sql = Value::String(after.clone()).to_sql();
                assert_eq!(
                    scan["attributes"]["access"],
                    format!("[{owner_sql}] MoreThan {after_sql}"),
                    "{physical}"
                );
                assert_eq!(
                    scan["attributes"]["limit"],
                    (limit + 1).to_string(),
                    "{physical}"
                );
                // The scan may re-evaluate the exactly covered scalar conditions. It may
                // not retain another predicate that can reject arbitrarily many entries.
                if let Some(predicate) = scan["attributes"]["predicate"].as_str() {
                    let normalize = |value: &str| {
                        value
                            .chars()
                            .filter(|c| !c.is_whitespace() && !matches!(c, '(' | ')' | '`'))
                            .collect::<String>()
                    };
                    assert_eq!(
                        normalize(predicate),
                        normalize(&format!("{field}={owner_sql} AND {ordering}>{after_sql}")),
                        "{physical}"
                    );
                }
                if mode == "analyze" {
                    assert_eq!(
                        scan["observed_counters"]["output_rows"], observed.examined_rows,
                        "{physical}"
                    );
                }
            }
            if let Ok(directory) = std::env::var("PSE_DIAGNOSTIC_EVIDENCE") {
                let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
                hash.str(owner).str(after).str(index).u64(u64::from(limit));
                let path = std::path::Path::new(&directory)
                    .join(format!("graph-seek-{}.json", hash.finish_hash().to_hex()));
                std::fs::write(path,serde_json::to_vec_pretty(&serde_json::json!({"request":selected,"document":observed,"physical":physical,"bound_scope":"neighborhood index entries only; study anchors additionally perform two fixed study_scope validations, each at most 193 dependencies and 64 predecessor headers"})).unwrap()).unwrap();
            }
        }
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod server_tests {
    #![allow(
        clippy::unwrap_used,
        reason = "isolated graph fixtures and assertions fail the test on unexpected setup or query results"
    )]
    use super::*;
    use crate::canonical::CanonicalOptions;
    use crate::canonical_analyses::{AnalysisEdge, AnalysisNode, seal_analysis_request};
    use crate::canonical_artifacts::{artifact_block, seal_artifact_request};

    #[tokio::test]
    async fn canonical_graph_explanation_root_readers_bypass_conditional_row_permissions() {
        let state = std::env::var("PSE_SURREAL_STATE").unwrap();
        let mut options = CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_graph_auth_{}",
            uuid::Uuid::new_v4().simple()
        );
        let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
        // A false conditional permission would defeat a finite scan cap for record
        // authentication. Both supported root accounts must bypass it in reality.
        crate::canonical::bounded_query(store.db.query("DEFINE TABLE graph_auth_probe SCHEMALESS PERMISSIONS FOR select WHERE allowed=true; DEFINE INDEX graph_auth_seek ON graph_auth_probe FIELDS owner,key; INSERT INTO graph_auth_probe [{key:'a',owner:'owner',allowed:false},{key:'b',owner:'owner',allowed:false}];")).await.unwrap();
        let address = url::Url::parse(&options.endpoint)
            .unwrap()
            .socket_addrs(|| None)
            .unwrap()[0];
        for (username, password, role) in [
            (&options.username, &options.password, "OWNER"),
            (
                &options.selection_username,
                &options.selection_password,
                "VIEWER",
            ),
        ] {
            let deadline = original_deadline(REQUEST_TIMEOUT).into_std();
            let context = || crate::transport::client::RequestContext::control(deadline);
            let db =
                crate::transport::client::Client::connect(address, crate::canonical::MESSAGE_BYTES)
                    .await
                    .unwrap();
            db.signin(crate::transport::client::Root {
                username: username.clone(),
                password: password.clone(),
            })
            .request_context(context())
            .await
            .unwrap();
            db.use_ns(&options.namespace)
                .use_db(&options.database)
                .request_context(context())
                .await
                .unwrap();
            let mut info = db
                .query(format!(
                    "INFO FOR USER `{}` ON ROOT;",
                    username.replace('`', "\\`")
                ))
                .request_context(context())
                .await
                .unwrap()
                .check()
                .unwrap();
            let info = info.take::<Value>(0).unwrap().into_json_value().to_string();
            assert!(
                info.to_uppercase().contains(&format!("ROLES {role}")),
                "configured root role did not match {role}"
            );
            let mut rows=db.query("SELECT key FROM graph_auth_probe WITH INDEX graph_auth_seek WHERE owner='owner' AND key>'' ORDER BY key LIMIT 2;").request_context(context()).await.unwrap().check().unwrap();
            assert_eq!(rows.take::<Vec<Object>>(0).unwrap().len(), 2);
            db.disconnect().await.unwrap();
        }
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn canonical_graph_explanation_preserves_isolates_parallel_edges_and_protected_inputs() {
        graph_explanation_fixture(false).await;
    }

    #[tokio::test]
    #[ignore = "explicit operator evidence requires its captured native executor"]
    async fn canonical_operator_graph_evidence() {
        operator_evidence_directory();
        graph_explanation_fixture(true).await;
    }

    async fn graph_explanation_fixture(operator: bool) {
        let state = std::env::var("PSE_SURREAL_STATE").unwrap();
        let mut options = CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
        options.database = format!("canonical_test_graph_{}", uuid::Uuid::new_v4().simple());
        let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
        let revision = store
            .edit("graph-source", None, "source", &[])
            .await
            .unwrap();
        let bytes = b"protected-evidence-bytes".to_vec();
        let mut artifact = store
            .new_artifact(&revision, "graph-evidence", vec![], 1, bytes.len() as u64)
            .await
            .unwrap();
        let blocks = vec![artifact_block(
            &artifact.key,
            0,
            "capsule",
            bytes,
            "original",
            1,
        )];
        seal_artifact_request(
            &mut artifact,
            std::slice::from_ref(&revision),
            &[],
            &blocks,
            "audit",
            "graph",
        );
        store
            .persist_artifact(
                &artifact,
                std::slice::from_ref(&revision),
                &[],
                &blocks,
                "audit",
                "graph",
            )
            .await
            .unwrap();
        let read = store.read_artifact(&artifact.key).await.unwrap();
        let mut analysis = store
            .new_analysis(
                &revision,
                "graph-inspection",
                vec![],
                "graph-input".into(),
                3,
                130,
            )
            .await
            .unwrap();
        let nodes = (0..3)
            .map(|ordinal| AnalysisNode {
                key: format!("node-{ordinal}"),
                analysis: analysis.key.clone(),
                semantic: format!("semantic-{ordinal}"),
                kind: "variable".into(),
            })
            .collect::<Vec<_>>();
        let edges = (0..130)
            .map(|ordinal| AnalysisEdge {
                key: format!("edge-{ordinal:03}"),
                analysis: analysis.key.clone(),
                source: nodes[0].key.clone(),
                target: nodes[1].key.clone(),
                kind: if ordinal % 2 == 0 {
                    "incidence"
                } else {
                    "reported-relation"
                }
                .into(),
                evidence_artifact: Some(artifact.key.clone()),
                evidence_block: Some(0),
                evidence_row: Some(0),
                evidence_field: Some("field".into()),
            })
            .collect::<Vec<_>>();
        seal_analysis_request(
            &mut analysis,
            std::slice::from_ref(&revision),
            std::slice::from_ref(&read),
            &nodes,
            &edges,
        );
        store
            .persist_analysis(
                &analysis,
                std::slice::from_ref(&revision),
                std::slice::from_ref(&read),
                &nodes,
                &edges,
            )
            .await
            .unwrap();
        // Compare both directed neighborhoods against the authored edge list, never
        // against a second read of the relation being qualified.
        for (selected, scope) in [
            (&nodes[0], GraphScope::RetainedConsumers),
            (&nodes[1], GraphScope::RecordedSources),
            (&nodes[2], GraphScope::RetainedConsumers),
            (&nodes[2], GraphScope::RecordedSources),
        ] {
            let request = CanonicalGraphRequest {
                version: pse_model::document::Version,
                anchor: GraphAnchor::AnalysisNode {
                    analysis: analysis.key.clone(),
                    node: selected.key.clone(),
                    revision: revision.key.clone(),
                },
                scope,
                limit: 1,
                continuation: None,
            };
            let expected = edges
                .iter()
                .filter(|edge| {
                    if scope == GraphScope::RecordedSources {
                        edge.target == selected.key
                    } else {
                        edge.source == selected.key
                    }
                })
                .map(|edge| {
                    (
                        edge.key.clone(),
                        GraphEdge {
                            identity: node("canonical_analysis_edges", &edge.key),
                            source: node("canonical_analysis_nodes", &edge.source),
                            target: node("canonical_analysis_nodes", &edge.target),
                            role: GraphEdgeRole::AnalysisRelation {
                                relation: edge.kind.clone(),
                            },
                            evidence: Some(GraphEvidence {
                                artifact: artifact.key.clone(),
                                block: Some(0),
                                row: Some(0),
                                field: Some("field".into()),
                            }),
                        },
                    )
                })
                .collect();
            qualify_graph_seek_pages(&store, &request, expected).await;
        }
        let mut input_consumers = vec![analysis.key.clone()];
        // Populate enough genuine immutable consumers to cross the maximum page size.
        for ordinal in 0..65 {
            let mut consumer = store
                .new_analysis(
                    &revision,
                    &format!("graph-consumer-{ordinal:03}"),
                    vec![],
                    format!("input-{ordinal}"),
                    0,
                    0,
                )
                .await
                .unwrap();
            seal_analysis_request(
                &mut consumer,
                std::slice::from_ref(&revision),
                std::slice::from_ref(&read),
                &[],
                &[],
            );
            store
                .persist_analysis(
                    &consumer,
                    std::slice::from_ref(&revision),
                    std::slice::from_ref(&read),
                    &[],
                    &[],
                )
                .await
                .unwrap();
            input_consumers.push(consumer.key);
        }
        let expected = input_consumers
            .iter()
            .map(|key| {
                let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
                hash.str("pse.analysis.input.v1")
                    .part(key.as_bytes())
                    .part(artifact.key.as_bytes());
                (
                    key.clone(),
                    GraphEdge {
                        identity: node("canonical_analysis_inputs", &hash.finish_hash().to_hex()),
                        source: node("canonical_analyses", key),
                        target: node("canonical_artifacts", &artifact.key),
                        role: GraphEdgeRole::AnalysisInput,
                        evidence: Some(GraphEvidence {
                            artifact: artifact.key.clone(),
                            block: None,
                            row: None,
                            field: None,
                        }),
                    },
                )
            })
            .collect();
        qualify_graph_seek_pages(
            &store,
            &CanonicalGraphRequest {
                version: pse_model::document::Version,
                anchor: GraphAnchor::Artifact {
                    artifact: artifact.key.clone(),
                    revision: revision.key.clone(),
                },
                scope: GraphScope::RetainedConsumers,
                limit: 1,
                continuation: None,
            },
            expected,
        )
        .await;
        read.release().await.unwrap();
        let mut request = CanonicalGraphRequest {
            version: pse_model::document::Version,
            anchor: GraphAnchor::AnalysisNode {
                analysis: analysis.key.clone(),
                node: nodes[0].key.clone(),
                revision: revision.key.clone(),
            },
            scope: GraphScope::RetainedConsumers,
            limit: 1,
            continuation: None,
        };
        let first = store.graph_explanation(&request).await.unwrap();
        let physical = store
            .inspect_graph_explanation(&request, true)
            .await
            .unwrap();
        assert!(
            !physical["operator_observations"]["analyze"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        if operator {
            qualify_operator(&store, "graph", &serde_json::to_value(&request).unwrap()).await;
        }
        if let Ok(directory) = std::env::var("PSE_DIAGNOSTIC_EVIDENCE") {
            std::fs::write(std::path::Path::new(&directory).join("analysis-graph.json"),serde_json::to_vec_pretty(&serde_json::json!({"conditions":{"nodes":3,"parallel_edges":130,"isolates":1,"page_limit":1,"protected_artifact_bytes":24},"semantic":first,"physical":physical})).unwrap()).unwrap();
        }
        assert!(!first.complete);
        assert_eq!(first.examined_rows, 2);
        assert_eq!(first.edges.len(), 1);
        request.continuation = first.continuation.clone();
        let next = store.graph_explanation(&request).await.unwrap();
        assert!(!next.complete);
        assert_ne!(first.edges[0].role, next.edges[0].role);
        assert!(
            !serde_json::to_string(&first)
                .unwrap()
                .contains("protected-evidence-bytes")
        );
        request.continuation = None;
        request.anchor = GraphAnchor::AnalysisNode {
            analysis: analysis.key.clone(),
            node: nodes[2].key.clone(),
            revision: revision.key.clone(),
        };
        let isolate = store.graph_explanation(&request).await.unwrap();
        assert!(isolate.complete);
        assert!(isolate.edges.is_empty());
        assert_eq!(isolate.nodes.len(), 1);
        request.anchor = GraphAnchor::Artifact {
            artifact: artifact.key.clone(),
            revision: revision.key.clone(),
        };
        let consumers = store.graph_explanation(&request).await.unwrap();
        assert!(!consumers.complete);
        assert_eq!(consumers.edges.len(), 1);
        assert_eq!(consumers.edges[0].role, GraphEdgeRole::AnalysisInput);
        request.anchor = GraphAnchor::Artifact {
            artifact: artifact.key.clone(),
            revision: "changed".into(),
        };
        assert!(store.graph_explanation(&request).await.is_err());
        request.anchor = GraphAnchor::AnalysisNode {
            analysis: input_consumers[1].clone(),
            node: nodes[0].key.clone(),
            revision: revision.key.clone(),
        };
        assert!(
            store.graph_explanation(&request).await.is_err(),
            "an existing node cannot be selected under another analysis"
        );
        request.anchor = GraphAnchor::AnalysisNode {
            analysis: analysis.key.clone(),
            node: nodes[0].key.clone(),
            revision: "changed".into(),
        };
        assert!(store.graph_explanation(&request).await.is_err());
        request.anchor = GraphAnchor::AnalysisNode {
            analysis: analysis.key.clone(),
            node: nodes[0].key.clone(),
            revision: revision.key.clone(),
        };
        // Deliberately corrupt the sentinel through the fixture owner's raw database
        // connection. The immutable writer never creates this cross-analysis row.
        crate::canonical::bounded_query(store.db.query("UPDATE type::record('canonical_analysis_edges',$key) SET analysis=$analysis RETURN NONE;").bind(("key",edges[1].key.clone())).bind(("analysis",input_consumers[1].clone()))).await.unwrap();
        assert!(
            store.graph_explanation(&request).await.is_err(),
            "a corrupt sentinel must refuse the whole page"
        );
        store.remove_isolated_fixture().await.unwrap();
    }
}
