// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Persisted incidence and retained result-evidence graphs. Reachability is
//! structural; quantitative meaning stays in the original scientific Arrow rows.

use super::{Durability, ModelingSolvePreparation, Runtime, WorkflowError, contract, relation};
use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::{Dfs, Reversed},
};
use pse_columnar::{CancellationToken, MemoryConsumer};
use pse_operations::canonical_analyses::{
    ANALYSIS_EDGES, ANALYSIS_NODES, AnalysisEdge, AnalysisNode, analysis_key,
};
#[cfg(all(test, feature = "canonical-tests"))]
use std::time::Duration;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, atomic::AtomicBool},
};

/// Direction in the graph's explicit source-to-dependent scientific edges.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisDirection {
    /// Objects contributing to the selected roots.
    Upstream,
    /// Objects depending on the selected roots.
    Downstream,
}
/// Exact semantic roots and direction; an empty root set selects the whole
/// bounded method graph. Roots are exact semantic names from that method's nodes.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisControls {
    /// Exact selected semantic object names.
    pub roots: Vec<String>,
    /// Traverse contributors or dependents, preserving distinct edge meanings.
    pub direction: AnalysisDirection,
}
impl AnalysisControls {
    fn validate(&self) -> Result<(), WorkflowError> {
        if self.roots.len() > 64 || self.roots.iter().any(|root| root.len() > 128) {
            return Err(contract("analysis root bound"));
        }
        Ok(())
    }
}
fn target_kind(kind: pse_model::generated::enums::NumericalTarget) -> &'static str {
    use pse_model::generated::enums::NumericalTarget;
    match kind {
        NumericalTarget::Variable => "variable",
        NumericalTarget::Row => "row",
        NumericalTarget::Objective => "objective",
        NumericalTarget::Observable => "observable",
        NumericalTarget::Closure => "closure",
    }
}
use pse_operations::canonical::MetadataResponse;
use pse_operations::canonical_artifacts::{ArtifactLocator, ArtifactRead};
type Graph = DiGraph<(String, String), (String, Option<ArtifactLocator>)>;
// Four MiB transport, native/typed copies, a <=128 KiB header configuration
// or <=64 bounded graph rows, and Arrow projection scratch. No IPC verifier.
fn vertex(
    graph: &mut Graph,
    nodes: &mut BTreeMap<String, NodeIndex>,
    semantic: String,
    kind: &str,
) -> Result<NodeIndex, WorkflowError> {
    if let Some(index) = nodes.get(&semantic) {
        return Ok(*index);
    }
    if nodes.len() >= ANALYSIS_NODES {
        return Err(contract("analysis node bound exceeded"));
    }
    let index = graph.add_node((semantic.clone(), kind.into()));
    nodes.insert(semantic, index);
    Ok(index)
}
fn edge(
    graph: &mut Graph,
    source: NodeIndex,
    target: NodeIndex,
    kind: &str,
    evidence: Option<ArtifactLocator>,
) -> Result<(), WorkflowError> {
    // Identical context references can qualify many rows at the same scope.
    // Deduplicate only exact edges; different block/row/field locators survive.
    if graph
        .edges_connecting(source, target)
        .any(|existing| existing.weight() == &(kind.into(), evidence.clone()))
    {
        return Ok(());
    }
    if graph.edge_count() >= ANALYSIS_EDGES {
        return Err(contract("analysis edge bound exceeded"));
    }
    graph.add_edge(source, target, (kind.into(), evidence));
    Ok(())
}
fn selected(
    graph: &Graph,
    nodes: &BTreeMap<String, NodeIndex>,
    controls: &AnalysisControls,
) -> Result<BTreeSet<NodeIndex>, WorkflowError> {
    controls.validate()?;
    if controls.roots.is_empty() {
        return Ok(graph.node_indices().collect());
    }
    let mut result = BTreeSet::new();
    for root in &controls.roots {
        let index = *nodes
            .get(root)
            .ok_or_else(|| contract("analysis root is not a selected scientific object"))?;
        let mut visit = Dfs::new(graph, index);
        match controls.direction {
            AnalysisDirection::Downstream => {
                while let Some(node) = visit.next(graph) {
                    result.insert(node);
                }
            }
            AnalysisDirection::Upstream => {
                while let Some(node) = visit.next(Reversed(graph)) {
                    result.insert(node);
                }
            }
        }
    }
    // A selected claim keeps its admitted context without traversing unrelated
    // responses sharing that immutable evidence owner.
    use petgraph::visit::EdgeRef;
    let context = graph
        .edge_references()
        .filter(|edge| {
            result.contains(&edge.target()) && graph[edge.source()].1 == "retained_context"
        })
        .map(|edge| edge.source())
        .collect::<Vec<_>>();
    result.extend(context);
    Ok(result)
}

// A locator belongs to the independently retained artifact, never the producer's
// reclaimed result file. Family is admitted context; validity only qualifies a claim.
#[derive(Clone, Debug)]
struct Located<R> {
    row: R,
    locator: ArtifactLocator,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResponseFamily {
    Root,
    Optimal,
}
impl ResponseFamily {
    fn name(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Optimal => "optimal",
        }
    }
}
#[derive(Default)]
struct ResponseEvidence {
    contexts: Vec<Located<pse_model::generated::runtime::scientific_contexts::Row>>,
    routes: Vec<Located<pse_model::generated::runtime::route_decisions::Row>>,
    validity: Vec<Located<pse_model::generated::runtime::local_validity::Row>>,
    computations: Vec<Located<pse_model::generated::runtime::computation_runs::Row>>,
    endpoints: Vec<Located<pse_model::generated::runtime::trajectory_endpoints::Row>>,
    samples: Vec<Located<pse_model::generated::runtime::simulation_samples::Row>>,
    modes: Vec<Located<pse_model::generated::runtime::modeling_trajectory_modes::Row>>,
    events: Vec<Located<pse_model::generated::runtime::simulation_events::Row>>,
    parametric: Vec<Located<pse_model::generated::runtime::parametric_sensitivities::Row>>,
    responses: Vec<Located<pse_model::generated::runtime::response_sensitivities::Row>>,
    // Other interpretation rows (including original scaling) remain addressable.
    interpretation: Vec<ArtifactLocator>,
}
fn field_locator(locator: &ArtifactLocator, field: &str) -> ArtifactLocator {
    let mut locator = locator.clone();
    locator.field = Some(field.into());
    locator
}
fn located_rows<R: pse_relations::columnar::RelationRow>(
    batch: &pse_relations::columnar::FieldCheckedBatch,
    artifact: &str,
    block: u64,
) -> Result<Vec<Located<R>>, WorkflowError> {
    Ok(R::rows(batch)
        .map_err(relation)?
        .into_iter()
        .enumerate()
        .map(|(row, value)| Located {
            row: value,
            locator: ArtifactLocator {
                artifact: artifact.into(),
                block,
                row: Some(row as u64),
                field: None,
            },
        })
        .collect())
}
impl ResponseEvidence {
    fn push(
        &mut self,
        name: &str,
        batch: &pse_relations::columnar::FieldCheckedBatch,
        artifact: &str,
        block: u64,
    ) -> Result<(), WorkflowError> {
        // Refuse complete relations rather than silently graphing a prefix.
        macro_rules! rows {
            ($field:ident) => {{
                let rows = located_rows(batch, artifact, block)?;
                if self.$field.len().saturating_add(rows.len()) > ANALYSIS_NODES {
                    return Err(contract("analysis evidence relation exceeds bound"));
                }
                self.$field.extend(rows);
            }};
        }
        match name {
            "runtime.scientific_contexts" => rows!(contexts),
            "runtime.route_decisions" => rows!(routes),
            "runtime.local_validity" => rows!(validity),
            "runtime.computation_runs" => rows!(computations),
            "runtime.trajectory_endpoints" => rows!(endpoints),
            "runtime.simulation_samples" => rows!(samples),
            "runtime.modeling_trajectory_modes" => rows!(modes),
            "runtime.simulation_events" => rows!(events),
            "runtime.parametric_sensitivities" => rows!(parametric),
            "runtime.response_sensitivities" => rows!(responses),
            _ => {
                self.interpretation.push(ArtifactLocator {
                    artifact: artifact.into(),
                    block,
                    row: None,
                    field: None,
                });
            }
        }
        Ok(())
    }
    fn scalar_family(
        &self,
        run: pse_model::generated::identities::RunId,
        step: i64,
    ) -> Option<ResponseFamily> {
        use pse_model::generated::enums::NativeSolveIntent;
        let contexts = self
            .contexts
            .iter()
            .filter(|context| context.row.step == step && context.row.time.is_none())
            .collect::<Vec<_>>();
        // Route rows have no run_id. Ambiguous step ownership cannot borrow a
        // different run's route, even when its numerical rows happen to agree.
        if contexts.is_empty()
            || contexts.iter().any(|context| {
                context.row.run_id != run
                    || context.row.model_id.is_none()
                    || context.row != contexts[0].row
            })
        {
            return None;
        }
        let mut routes = self.routes.iter().filter(|route| route.row.step == step);
        let first = &routes.next()?.row;
        if routes.any(|route| {
            route.row.request_identity != first.request_identity || route.row.intent != first.intent
        }) {
            return None;
        }
        match first.intent {
            NativeSolveIntent::Root => Some(ResponseFamily::Root),
            NativeSolveIntent::Optimize => Some(ResponseFamily::Optimal),
            _ => None,
        }
    }
    fn scalar_certified(
        &self,
        run: pse_model::generated::identities::RunId,
        step: i64,
        family: ResponseFamily,
    ) -> bool {
        use pse_model::generated::enums::DerivedQuantity;
        let rows = self
            .validity
            .iter()
            .filter(|row| {
                row.row.run_id == run
                    && row.row.step == step
                    && row.row.quantity == DerivedQuantity::ParametricSensitivity
            })
            .collect::<Vec<_>>();
        !rows.is_empty()
            && rows.iter().all(|row| {
                let v = &row.row.validity;
                if !v.certified || v.reason.is_some() {
                    return false;
                }
                match family {
                    ResponseFamily::Root => {
                        v.root_rank.is_some()
                            && v.root_rank_cutoff.is_some()
                            && v.root_rank_relative_cutoff.is_some()
                            && v.root_backward_error.is_some()
                            && v.root_backward_error_limit.is_some()
                            && v.root_neighborhood
                                .as_ref()
                                .is_some_and(|value| !value.is_empty())
                            && v.licq.is_none()
                            && v.strict_complementarity.is_none()
                            && v.second_order.is_none()
                    }
                    ResponseFamily::Optimal => {
                        v.licq == Some(true)
                            && v.strict_complementarity == Some(true)
                            && v.second_order == Some(true)
                            && v.weakly_active == Some(0)
                            && v.root_rank.is_none()
                            && v.root_rank_cutoff.is_none()
                            && v.root_rank_relative_cutoff.is_none()
                            && v.root_backward_error.is_none()
                            && v.root_backward_error_limit.is_none()
                            && v.root_neighborhood.is_none()
                    }
                }
            })
    }
    // Simulation publication records sensitivity availability, not a scalar
    // local certificate. Partial/event trajectories retain this weaker meaning.
    fn transient_recorded(
        &self,
        row: &pse_model::generated::runtime::response_sensitivities::Row,
    ) -> bool {
        use pse_model::generated::enums::ComputationKind;
        let Some(time) = row.time else {
            return false;
        };
        let headers = self
            .computations
            .iter()
            .filter(|header| header.row.run_id == row.run_id)
            .collect::<Vec<_>>();
        let contexts = self
            .contexts
            .iter()
            .filter(|context| context.row.run_id == row.run_id)
            .collect::<Vec<_>>();
        let endpoints = self
            .endpoints
            .iter()
            .filter(|endpoint| endpoint.row.run_id == row.run_id)
            .collect::<Vec<_>>();
        !headers.is_empty()
            && headers.iter().all(|header| {
                header.row == headers[0].row
                    && header.row.kind == ComputationKind::Simulation
                    && header.row.response_available == Some(true)
                    && header.row.completed_time.is_some_and(|end| time <= end)
                    && header
                        .row
                        .completed_samples
                        .is_some_and(|count| row.sample >= 0 && row.sample < count)
            })
            && !contexts.is_empty()
            && contexts.iter().all(|context| {
                context.row == contexts[0].row
                    && context.row.time.is_some()
                    && context.row.instance_id == Some(row.experiment_id)
                    && context
                        .row
                        .inputs
                        .iter()
                        .any(|coordinate| coordinate.id == row.parameter_id)
            })
            && !endpoints.is_empty()
            && endpoints
                .iter()
                .all(|endpoint| endpoint.row == endpoints[0].row && time <= endpoint.row.time)
            && self.samples.iter().any(|sample| {
                sample.row.run_id == row.run_id
                    && sample.row.sample == row.sample
                    && sample.row.time == time
                    && sample.row.symbol_id == row.output_id
                    && sample.row.unit_id == row.output_unit_id
            })
    }
}
fn evidence_context(
    graph: &mut Graph,
    nodes: &mut BTreeMap<String, NodeIndex>,
    attempt: NodeIndex,
    target: NodeIndex,
    kind: &str,
    locator: &ArtifactLocator,
) -> Result<(), WorkflowError> {
    let mut identity = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
    identity
        .str("pse.analysis.evidence-coordinate.v2")
        .str(&locator.artifact)
        .u64(locator.block);
    // Fields are distinct edges over the same retained row/block owner.
    identity.part(&serde_json::to_vec(&locator.row).map_err(|error| contract(error.to_string()))?);
    let context = vertex(
        graph,
        nodes,
        format!("evidence:{}", identity.finish_hash().to_hex()),
        "retained_context",
    )?;
    if target == attempt {
        return edge(graph, attempt, context, kind, Some(locator.clone()));
    }
    edge(
        graph,
        attempt,
        context,
        "produced_context",
        Some(locator.clone()),
    )?;
    edge(graph, context, target, kind, Some(locator.clone()))
}
fn project_response_evidence(
    evidence: &ResponseEvidence,
    graph: &mut Graph,
    nodes: &mut BTreeMap<String, NodeIndex>,
    attempt: NodeIndex,
) -> Result<(), WorkflowError> {
    use pse_model::generated::enums::NumericalTarget;
    // Interpretation/qualification evidence survives even with no derivative
    // rows, including original withheld reasons and partial event coverage.
    macro_rules! retain {
        ($rows:expr, $kind:expr) => {
            for item in $rows {
                evidence_context(graph, nodes, attempt, attempt, $kind, &item.locator)?;
            }
        };
    }
    retain!(&evidence.contexts, "scientific_context");
    retain!(&evidence.routes, "admitted_route");
    retain!(&evidence.validity, "local_validity");
    retain!(&evidence.computations, "computation_context");
    retain!(&evidence.endpoints, "trajectory_endpoint");
    retain!(&evidence.events, "trajectory_event");
    retain!(&evidence.modes, "trajectory_mode");
    for locator in &evidence.interpretation {
        evidence_context(
            graph,
            nodes,
            attempt,
            attempt,
            "retained_interpretation",
            locator,
        )?;
    }
    for item in &evidence.parametric {
        let row = &item.row;
        let family = evidence.scalar_family(row.run_id, row.step);
        let family_name = family.map_or("unclassified", ResponseFamily::name);
        let parameter = vertex(graph, nodes, row.parameter_id.to_string(), "parameter")?;
        let target_name = if row.target_kind == NumericalTarget::Objective {
            "objective".into()
        } else {
            row.target_id.to_string()
        };
        let original = vertex(
            graph,
            nodes,
            target_name.clone(),
            target_kind(row.target_kind),
        )?;
        let target = vertex(
            graph,
            nodes,
            format!(
                "{family_name}:{}:step:{}:{target_name}",
                row.run_id, row.step
            ),
            "scoped_response",
        )?;
        edge(
            graph,
            target,
            original,
            "scientific_coordinate",
            Some(item.locator.clone()),
        )?;
        edge(
            graph,
            attempt,
            target,
            "produced_evidence",
            Some(item.locator.clone()),
        )?;
        for context in evidence
            .contexts
            .iter()
            .filter(|c| c.row.run_id == row.run_id && c.row.step == row.step)
        {
            evidence_context(
                graph,
                nodes,
                attempt,
                target,
                "response_coordinates",
                &context.locator,
            )?;
        }
        for route in evidence.routes.iter().filter(|r| r.row.step == row.step) {
            evidence_context(
                graph,
                nodes,
                attempt,
                target,
                "response_family",
                &field_locator(&route.locator, "intent"),
            )?;
        }
        for validity in evidence
            .validity
            .iter()
            .filter(|v| v.row.run_id == row.run_id && v.row.step == row.step)
        {
            evidence_context(
                graph,
                nodes,
                attempt,
                target,
                "response_validity",
                &field_locator(&validity.locator, "validity"),
            )?;
        }
        for (field, value) in [("primal", row.primal), ("dual", row.dual)] {
            if value.is_none() {
                continue;
            }
            let certified = family.is_some_and(|family| {
                evidence.scalar_certified(row.run_id, row.step, family)
                    && (family != ResponseFamily::Root
                        || (field == "primal" && row.target_kind == NumericalTarget::Variable))
            });
            let kind = if certified {
                format!("retained_{family_name}_parametric_{field}")
            } else {
                format!("withheld_{family_name}_parametric_{field}")
            };
            edge(
                graph,
                if certified { parameter } else { attempt },
                target,
                &kind,
                Some(field_locator(&item.locator, field)),
            )?;
        }
    }
    for item in &evidence.responses {
        let row = &item.row;
        let recorded = evidence.transient_recorded(row);
        let parameter = vertex(graph, nodes, row.parameter_id.to_string(), "parameter")?;
        // Each sample retains its own quantitative row locator even when the
        // semantic output node is shared. Sample/time/unit scope is in that row.
        let target = vertex(graph, nodes, row.output_id.to_string(), "output")?;
        // Production provenance describes the output's membership in this
        // retained block. Quantitative claims below still identify every row.
        let mut produced = item.locator.clone();
        produced.row = None;
        produced.field = None;
        edge(graph, attempt, target, "produced_evidence", Some(produced))?;
        edge(
            graph,
            if recorded { parameter } else { attempt },
            target,
            if recorded {
                "recorded_transient_response"
            } else {
                "withheld_response_claim"
            },
            Some(field_locator(&item.locator, "value")),
        )?;
        for context in evidence
            .contexts
            .iter()
            .filter(|c| c.row.run_id == row.run_id)
        {
            evidence_context(
                graph,
                nodes,
                attempt,
                target,
                "response_coordinates",
                &context.locator,
            )?;
        }
        for header in evidence
            .computations
            .iter()
            .filter(|h| h.row.run_id == row.run_id)
        {
            evidence_context(
                graph,
                nodes,
                attempt,
                target,
                "response_completion",
                &header.locator,
            )?;
        }
        for endpoint in evidence
            .endpoints
            .iter()
            .filter(|e| e.row.run_id == row.run_id)
        {
            evidence_context(
                graph,
                nodes,
                attempt,
                target,
                "response_endpoint",
                &endpoint.locator,
            )?;
        }
        // The quantitative row itself preserves exact sample/time/output/unit
        // coordinates. Its matching sample and mode remain in the same retained
        // trajectory; avoid duplicating a graph edge per output/sample metadata.
        for event in evidence
            .events
            .iter()
            .filter(|e| e.row.run_id == row.run_id)
        {
            evidence_context(
                graph,
                nodes,
                attempt,
                target,
                "response_event_context",
                &event.locator,
            )?;
        }
    }
    Ok(())
}

/// Compact restartable exact analysis handle. Graph pages come from native
/// relations; this owner never retains a hydrated run report or trajectory.
#[derive(Clone, Debug)]
pub struct AnalysisHandle {
    runtime: Runtime,
    key: String,
}
impl AnalysisHandle {
    /// Immutable method/configuration and graph identity.
    pub fn key(&self) -> &str {
        &self.key
    }
    /// Registry-declared method and input-lineage header.
    pub async fn header(
        &self,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                use pse_relations::generated::runtime::canonical_analyses as rows;
                let owner = super::connected_results::response_owner(
                    &self.runtime,
                    "canonical:analysis-header",
                    MetadataResponse::Header,
                )?;
                let row = pse_operations::canonical::response_ownership_scope(
                    owner.clone(),
                    self.runtime.canonical_store()?.analysis(&self.key),
                )
                .await?;
                let validation = self.runtime.validation_context();
                let mut builder =
                    rows::Builder::with_registry(self.runtime.registry(), 1, &validation)
                        .map_err(relation)?;
                builder.push(row).map_err(relation)?;
                builder
                    .finish()
                    .map_err(relation)?
                    .retained(&self.runtime.shared.pool(), &CancellationToken::new())
                    .map_err(relation)
            })
            .await
    }
    /// Native ordered node page, at most 64 rows, retaining actual Arrow buffers.
    pub async fn nodes(
        &self,
        after: Option<&str>,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                use pse_relations::generated::runtime::canonical_analysis_nodes as rows;
                let owner = super::connected_results::response_owner(
                    &self.runtime,
                    "canonical:analysis-page",
                    MetadataResponse::Page,
                )?;
                let page = pse_operations::canonical::response_ownership_scope(
                    owner.clone(),
                    self.runtime
                        .canonical_store()?
                        .analysis_node_page(&self.key, after),
                )
                .await?;
                let validation = self.runtime.validation_context();
                let mut builder =
                    rows::Builder::with_registry(self.runtime.registry(), page.len(), &validation)
                        .map_err(relation)?;
                for row in page {
                    builder.push(row).map_err(relation)?;
                }
                builder
                    .finish()
                    .map_err(relation)?
                    .retained(&self.runtime.shared.pool(), &CancellationToken::new())
                    .map_err(relation)
            })
            .await
    }
    /// Native directed graph edge page with original result evidence references.
    pub async fn edges(
        &self,
        after: Option<&str>,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                use pse_relations::generated::runtime::canonical_analysis_edges as rows;
                let owner = super::connected_results::response_owner(
                    &self.runtime,
                    "canonical:analysis-page",
                    MetadataResponse::Page,
                )?;
                let page = pse_operations::canonical::response_ownership_scope(
                    owner.clone(),
                    self.runtime
                        .canonical_store()?
                        .analysis_edge_page(&self.key, after),
                )
                .await?;
                let validation = self.runtime.validation_context();
                let mut builder =
                    rows::Builder::with_registry(self.runtime.registry(), page.len(), &validation)
                        .map_err(relation)?;
                for row in page {
                    builder.push(row).map_err(relation)?;
                }
                builder
                    .finish()
                    .map_err(relation)?
                    .retained(&self.runtime.shared.pool(), &CancellationToken::new())
                    .map_err(relation)
            })
            .await
    }
}
impl Runtime {
    /// Read bounded original completion metadata from one exact retained attempt.
    /// Scientific reports and trajectories remain in their canonical result sets.
    pub async fn stored_completion(
        &self,
        run: &str,
        attempt: &str,
    ) -> Result<Arc<pse_columnar::Leased<super::durable::StoredCompletion>>, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async { self.operations()?.completion(run, attempt).await })
            .await
    }
    /// Reopen only an active, retained exact analysis; no latest-run substitution.
    pub async fn analysis(&self, key: &str) -> Result<AnalysisHandle, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let response = super::connected_results::response_owner(
                    self,
                    "canonical:analysis-selection-response",
                    MetadataResponse::Header,
                )?;
                pse_operations::canonical::response_ownership_scope(
                    response,
                    self.canonical_store()?.analysis(key),
                )
                .await?;
                Ok(AnalysisHandle {
                    runtime: self.clone(),
                    key: key.into(),
                })
            })
            .await
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "one analysis admission binds exact source and result lineage, method/configuration, graph membership and selection controls"
    )]
    async fn store_analysis_graph(
        &self,
        revision: &pse_operations::canonical::Revision,
        sources: &[pse_operations::canonical::Revision],
        inputs: &[ArtifactRead],
        method: &str,
        configuration: Vec<u8>,
        graph: Graph,
        nodes: BTreeMap<String, NodeIndex>,
        controls: &AnalysisControls,
    ) -> Result<AnalysisHandle, WorkflowError> {
        use petgraph::visit::EdgeRef;
        let selected = selected(&graph, &nodes, controls)?;
        let mut input_hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
        input_hash.str("pse.analysis.inputs.v1");
        for source in sources {
            input_hash.str(&source.key);
        }
        for input in inputs {
            input_hash
                .str(&input.header().key)
                .str(&input.header().creation_request_digest);
        }
        for index in &selected {
            let (semantic, kind) = &graph[*index];
            input_hash.str(semantic).str(kind);
        }
        let mut included = graph
            .edge_references()
            .filter(|edge| selected.contains(&edge.source()) && selected.contains(&edge.target()))
            .map(|edge| {
                let (kind, evidence) = edge.weight();
                (
                    graph[edge.source()].0.clone(),
                    graph[edge.target()].0.clone(),
                    kind.clone(),
                    evidence.clone(),
                )
            })
            .collect::<Vec<_>>();
        included.sort();
        included.dedup();
        for (source, target, kind, evidence) in &included {
            input_hash
                .str(source)
                .str(target)
                .str(kind)
                .part(&serde_json::to_vec(evidence).map_err(|error| contract(error.to_string()))?);
        }
        let input_digest = input_hash.finish_hash().to_hex();
        let mut header = self
            .canonical_store()?
            .new_analysis(
                revision,
                method,
                configuration,
                input_digest,
                selected.len() as u64,
                included.len() as u64,
            )
            .await?;
        let key = header.key.clone();
        let mut mapping = BTreeMap::new();
        let mut retained_nodes = Vec::with_capacity(selected.len());
        for index in selected {
            let (semantic, kind) = &graph[index];
            let node_key = analysis_key(
                "pse.analysis.node.v1",
                &[key.as_bytes(), semantic.as_bytes()],
            );
            mapping.insert(semantic.clone(), node_key.clone());
            retained_nodes.push(AnalysisNode {
                key: node_key,
                analysis: key.clone(),
                semantic: semantic.clone(),
                kind: kind.clone(),
            });
        }
        let mut retained_edges = Vec::with_capacity(included.len());
        for (source, target, kind, evidence) in included {
            let source = mapping
                .get(&source)
                .ok_or_else(|| contract("analysis source absent"))?
                .clone();
            let target = mapping
                .get(&target)
                .ok_or_else(|| contract("analysis target absent"))?
                .clone();
            let evidence_bytes =
                serde_json::to_vec(&evidence).map_err(|error| contract(error.to_string()))?;
            let edge_key = analysis_key(
                "pse.analysis.edge.v1",
                &[
                    key.as_bytes(),
                    source.as_bytes(),
                    target.as_bytes(),
                    kind.as_bytes(),
                    &evidence_bytes,
                ],
            );
            retained_edges.push(AnalysisEdge {
                key: edge_key,
                analysis: key.clone(),
                source,
                target,
                kind,
                evidence_artifact: evidence.as_ref().map(|locator| locator.artifact.clone()),
                evidence_block: evidence.as_ref().map(|locator| locator.block),
                evidence_row: evidence.as_ref().and_then(|locator| locator.row),
                evidence_field: evidence.and_then(|locator| locator.field),
            });
        }
        pse_operations::canonical_analyses::seal_analysis_request(
            &mut header,
            sources,
            inputs,
            &retained_nodes,
            &retained_edges,
        );
        self.canonical_store()?
            .persist_analysis(&header, sources, inputs, &retained_nodes, &retained_edges)
            .await?;
        Ok(AnalysisHandle {
            runtime: self.clone(),
            key,
        })
    }

    /// Persist family-qualified response evidence and its exact provenance.
    /// Scalar derivatives require their admitted route and original validity;
    /// transient recordings retain their weaker sample/endpoint meaning. Graph
    /// reachability never invents a derivative or promotes outcome-class success.
    pub async fn result_analysis(
        &self,
        run: &str,
        attempt: &str,
        controls: &AnalysisControls,
        cancel: CancellationToken,
    ) -> Result<AnalysisHandle, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                controls.validate()?;
                cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                let owner =
                    MemoryConsumer::new("canonical:result-analysis").register(&self.shared.pool());
                owner
                    .try_grow(128 * 1024 * 1024)
                    .map_err(pse_engine::EngineError::from)?;
                let response = super::connected_results::response_owner(
                    self,
                    "canonical:result-analysis-selection-response",
                    MetadataResponse::Selection,
                )?;
                let input = Arc::new(
                    pse_operations::canonical::response_ownership_scope(
                        response.clone(),
                        self.canonical_store()?.read_results(run, attempt),
                    )
                    .await?,
                );
                let input_owner = Arc::new(super::connected_results::retain_protected_metadata(
                    &self.shared.pool(),
                    response,
                    input.retained_bytes()?,
                    |owner| input.retain_metadata_owner(owner),
                )?);
                input.retain_metadata_owner(input_owner.clone())?;
                let response = super::connected_results::response_owner(
                    self,
                    "canonical:result-analysis-sources-response",
                    MetadataResponse::Selection,
                )?;
                let sources = pse_operations::canonical::response_ownership_scope(
                    Arc::new((input.clone(), input_owner.clone(), response)),
                    self.canonical_store()?.analysis_run_sources(run),
                )
                .await?;
                let revision = sources
                    .iter()
                    .find(|source| source.key == input.run().revision)
                    .ok_or_else(|| contract("analysis primary result source absent"))?
                    .clone();
                let mut graph = Graph::new();
                let mut nodes = BTreeMap::new();
                let attempt_node = vertex(
                    &mut graph,
                    &mut nodes,
                    format!("attempt:{attempt}"),
                    "attempt",
                )?;
                for source in &sources {
                    let node = vertex(
                        &mut graph,
                        &mut nodes,
                        format!("revision:{}", source.key),
                        "source_revision",
                    )?;
                    edge(&mut graph, node, attempt_node, "input_provenance", None)?;
                }
                let mut dispatch = ResponseEvidence::default();
                for name in ["runtime.scientific_contexts", "runtime.computation_runs"] {
                    let spec = self
                        .registry()
                        .relation(name)
                        .ok_or_else(|| contract("response dispatch relation absent"))?;
                    let Some(set) = input
                        .sets()
                        .iter()
                        .find(|set| set.name == spec.id.to_string())
                    else {
                        continue;
                    };
                    let schema = pse_schema::arrow::relation_schema_ref(self.registry(), spec)
                        .map_err(|error| contract(error.to_string()))?;
                    let snapshot = super::ResultSnapshot::capture(
                        self,
                        input.clone(),
                        input_owner.clone(),
                        spec.id,
                        &set.key,
                        schema,
                        0,
                        u64::MAX,
                    )
                    .await?;
                    for block in snapshot.blocks() {
                        let batch = snapshot.read_block(block, &cancel).await?;
                        owner
                            .try_grow(batch.batch().get_array_memory_size().saturating_mul(4))
                            .map_err(pse_engine::EngineError::from)?;
                        dispatch.push(name, &batch, "dispatch", block.ordinal)?;
                    }
                }
                let kind = super::artifacts::response_artifact_kind(
                    dispatch.contexts.iter().map(|item| &item.row),
                    dispatch.computations.iter().map(|item| &item.row),
                )?;
                drop(dispatch);
                let staging_owner = format!(
                    "analysis-staging:{}",
                    pse_operations::mint_id::<pse_model::generated::identities::RunId>()
                );
                let artifact = match self
                    .promote_scientific_artifact(
                        run,
                        attempt,
                        kind,
                        "audit",
                        &staging_owner,
                        cancel.clone(),
                    )
                    .await
                {
                    Ok(artifact) => artifact,
                    Err(source) => {
                        if let WorkflowError::Canonical(
                            pse_operations::canonical::CanonicalError::ArtifactUnsettled {
                                intent,
                                ..
                            },
                        ) = &source
                        {
                            return Err(WorkflowError::ArtifactTransferUnsettled {
                                analysis: None,
                                artifact: intent.clone(),
                                owner_kind: "audit".into(),
                                owner: staging_owner,
                                read: None,
                                source: Box::new(source),
                            });
                        }
                        return Err(source);
                    }
                };
                let transfer_error =
                    |source: WorkflowError, analysis: Option<String>, read: Option<String>| {
                        WorkflowError::ArtifactTransferUnsettled {
                            analysis,
                            artifact: Box::new(artifact.header().clone()),
                            owner_kind: "audit".into(),
                            owner: staging_owner.clone(),
                            read,
                            source: Box::new(source),
                        }
                    };
                let response = super::connected_results::response_owner(
                    self,
                    "canonical:analysis-artifact-selection-response",
                    MetadataResponse::Selection,
                )
                .map_err(|source| transfer_error(source, None, None))?;
                let artifact_read = pse_operations::canonical::response_ownership_scope(
                    response.clone(),
                    self.canonical_store()?
                        .read_artifact(&artifact.header().key),
                )
                .await
                .map_err(|source| transfer_error(source.into(), None, None))?;
                let read_owner = Arc::new(
                    super::connected_results::retain_protected_metadata(
                        &self.shared.pool(),
                        response,
                        artifact_read.retained_bytes().map_err(|source| {
                            transfer_error(
                                source.into(),
                                None,
                                Some(artifact_read.read_key().to_owned()),
                            )
                        })?,
                        |owner| artifact_read.retain_metadata_owner(owner),
                    )
                    .map_err(|source| {
                        transfer_error(source, None, Some(artifact_read.read_key().to_owned()))
                    })?,
                );
                artifact_read
                    .retain_metadata_owner(read_owner.clone())
                    .map_err(|source| {
                        transfer_error(
                            source.into(),
                            None,
                            Some(artifact_read.read_key().to_owned()),
                        )
                    })?;
                let read_key = artifact_read.read_key().to_owned();
                let graph_result: Result<AnalysisHandle, WorkflowError> = async {
                    let mut evidence = ResponseEvidence::default();
                    for ordinal in 0..artifact.header().block_count {
                        let response = super::connected_results::response_owner(
                            self,
                            "canonical:analysis-artifact-response",
                            MetadataResponse::Header,
                        )?;
                        let payload = Arc::new(
                            MemoryConsumer::new("canonical:analysis-artifact-payload")
                                .register(&self.shared.pool()),
                        );
                        payload
                            .try_grow(4 * super::result_files::MAX_FILE_BYTES)
                            .map_err(pse_engine::EngineError::from)?;
                        let block = pse_operations::canonical::response_ownership_scope(
                            Arc::new((response, payload, read_owner.clone())),
                            artifact_read.block(ordinal),
                        )
                        .await?;
                        let relation_name = block.relation.as_str();
                        let spec = self
                            .registry()
                            .relation(relation_name)
                            .ok_or_else(|| contract("analysis copied relation unavailable"))?;
                        let schema = pse_schema::arrow::relation_schema_ref(self.registry(), spec)
                            .map_err(|error| contract(error.to_string()))?;
                        let decoded = super::result_files::decode_result_file(
                            block.payload.as_slice(),
                            schema,
                            usize::try_from(block.rows)
                                .map_err(|_| contract("analysis row extent"))?,
                        )?;
                        let batch = pse_relations::columnar::FieldCheckedBatch::admit(
                            self.registry(),
                            spec,
                            decoded,
                            &self.validation_context(),
                            &cancel,
                        )
                        .map_err(pse_engine::EngineError::from)?;
                        owner
                            .try_grow(batch.batch().get_array_memory_size().saturating_mul(4))
                            .map_err(pse_engine::EngineError::from)?;
                        evidence.push(relation_name, &batch, &artifact.header().key, ordinal)?;
                    }
                    project_response_evidence(&evidence, &mut graph, &mut nodes, attempt_node)?;
                    let configuration = serde_json::to_vec(&(2_u8, controls, run, attempt))
                        .map_err(|error| contract(error.to_string()))?;
                    let handle = self
                        .store_analysis_graph(
                            &revision,
                            &sources,
                            std::slice::from_ref(&artifact_read),
                            "result-sensitivity-provenance-reachability:v2",
                            configuration,
                            graph,
                            nodes,
                            controls,
                        )
                        .await?;
                    Ok(handle)
                }
                .await;
                let handle = graph_result.map_err(|source| {
                    let analysis = match &source {
                        WorkflowError::Canonical(
                            pse_operations::canonical::CanonicalError::AnalysisUnsettled {
                                intent,
                                ..
                            },
                        ) => Some(intent.key.clone()),
                        _ => None,
                    };
                    transfer_error(source, analysis, Some(read_key.clone()))
                })?;
                self.canonical_store()?
                    .release_artifact_intent(artifact.header(), "audit", &staging_owner)
                    .await
                    .map_err(|source| {
                        transfer_error(
                            source.into(),
                            Some(handle.key().to_owned()),
                            Some(read_key.clone()),
                        )
                    })?;
                artifact_read.release().await.map_err(|source| {
                    transfer_error(source.into(), Some(handle.key().to_owned()), Some(read_key))
                })?;
                Ok(handle)
            })
            .await
    }
}
impl ModelingSolvePreparation {
    /// Persist original all-branch numerical incidence and conservative execution
    /// dependencies before dispatch. Neither edge kind certifies derivatives.
    pub async fn dependency_analysis(
        &self,
        controls: &AnalysisControls,
        cancel: CancellationToken,
    ) -> Result<AnalysisHandle, WorkflowError> {
        let activity = self.source.runtime.admit_activity()?;
        activity
            .scope(async {
                controls.validate()?;
                let runtime = &self.source.runtime;
                let owner = MemoryConsumer::new("canonical:dependency-analysis")
                    .register(&runtime.shared.pool());
                owner
                    .try_grow(128 * 1024 * 1024)
                    .map_err(pse_engine::EngineError::from)?;
                cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                let plan = &self.model.case.compiled().plan;
                let structure = plan.structure();
                if structure.variables().len()
                    + structure.parameters().len()
                    + structure.rows().len()
                    + structure.objectives().len()
                    > ANALYSIS_NODES
                {
                    return Err(contract("analysis original structure exceeds bound"));
                }
                let mut graph = Graph::new();
                let mut nodes = BTreeMap::new();
                for variable in structure.variables() {
                    vertex(
                        &mut graph,
                        &mut nodes,
                        variable.port.id.to_string(),
                        "variable",
                    )?;
                }
                for parameter in structure.parameters() {
                    vertex(
                        &mut graph,
                        &mut nodes,
                        parameter.id.to_string(),
                        "parameter",
                    )?;
                }
                for row in structure.rows() {
                    vertex(&mut graph, &mut nodes, row.id.to_string(), "row")?;
                }
                for level in 0..structure.objectives().len() {
                    vertex(
                        &mut graph,
                        &mut nodes,
                        format!("objective:{level}"),
                        "objective",
                    )?;
                }
                let flow = plan
                    .visit_dependencies(&Arc::new(AtomicBool::new(false)), |dependency| {
                        let projected = (|| {
                            cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                            let target_name = match dependency.target {
                                pse_math::binding::Target::Row(id) => id.to_string(),
                                pse_math::binding::Target::Objective(level) => {
                                    format!("objective:{level}")
                                }
                            };
                            let target = *nodes.get(&target_name).ok_or_else(|| {
                                contract("analysis target absent from original structure")
                            })?;
                            for (kind, symbols) in [
                                ("numerical_incidence", dependency.numerical),
                                ("execution_dependency", dependency.execution),
                            ] {
                                for symbol in symbols {
                                    let source = *nodes.get(&symbol.to_string()).ok_or_else(|| {
                                contract("analysis dependency absent from original selected values")
                            })?;
                                    edge(&mut graph, source, target, kind, None)?;
                                }
                            }
                            Ok::<(), WorkflowError>(())
                        })();
                        match projected {
                            Ok(()) => std::ops::ControlFlow::Continue(()),
                            Err(error) => std::ops::ControlFlow::Break(error),
                        }
                    })
                    .map_err(super::math)?;
                if let std::ops::ControlFlow::Break(error) = flow {
                    return Err(error);
                }
                let revision = self.source.recorded_revision()?.clone();
                let mut sources = vec![revision.clone()];
                let operations = match runtime.durability() {
                    Durability::Durable(operations) => operations.as_ref().clone(),
                    Durability::Ephemeral => super::Operations::from_store(
                        runtime.canonical_store()?.clone(),
                        "analysis",
                        super::LeasePolicy::default(),
                        runtime.shared.pool(),
                    ),
                };
                let physical_rows = operations
                    .put_physical_rows(runtime, &self.source.physical)
                    .await?;
                sources.extend(physical_rows.revisions.iter().cloned());
                sources.sort_by(|a, b| a.key.cmp(&b.key));
                sources.dedup_by(|a, b| a.key == b.key);
                let configuration =
                    serde_json::to_vec(&(1_u8, controls, self.admission_identity()?.to_hex()))
                        .map_err(|error| contract(error.to_string()))?;
                let analysis = runtime
                    .store_analysis_graph(
                        &revision,
                        &sources,
                        &[],
                        "original-incidence-execution-reachability:v1",
                        configuration,
                        graph,
                        nodes,
                        controls,
                    )
                    .await?;
                // Keep the admitted physical source protection until analysis roots acknowledge.
                drop(physical_rows);
                Ok(analysis)
            })
            .await
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_analyses_server_unit {
    use super::*;
    use pse_relations::columnar::RelationRow;
    use pse_relations::generated::runtime::{
        canonical_analysis_edges, canonical_analysis_nodes, parametric_sensitivities,
    };
    #[tokio::test]
    async fn canonical_original_dependencies_and_retained_sensitivity_persist_exact_sources() {
        let runtime = super::super::durable_tests::durable_runtime();
        let (package, mut analysis) =
            super::super::durable_tests::package_on(&runtime, super::super::durable_tests::LINEAR)
                .await;
        analysis.bindings.demand.push("t".into());
        let cancel = crate::CancelSource::new();
        let original = package.prepare_analysis(&analysis, &cancel).await.unwrap();
        let parameter = original.model.case.compiled().plan.structure().parameters()[0].id;
        let variable = original.model.case.compiled().plan.structure().variables()[0]
            .port
            .id;
        analysis.solver.sensitivity = Some(crate::math::settings::SensitivityRequest {
            parameters: vec![parameter],
            reduced_hessian: false,
            propagation: None,
        });
        let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
        let controls = AnalysisControls {
            roots: vec![parameter.to_string()],
            direction: AnalysisDirection::Downstream,
        };
        let pre = prepared
            .dependency_analysis(&controls, CancellationToken::new())
            .await
            .unwrap();
        let receipt = runtime
            .canonical_store()
            .unwrap()
            .analysis(pre.key())
            .await
            .unwrap();
        assert!(receipt.active);
        assert_eq!(
            receipt.method,
            "original-incidence-execution-reachability:v1"
        );
        assert_eq!(receipt.revision, package.canonical_revision().unwrap().key);
        let nodes = canonical_analysis_nodes::Row::rows(&pre.nodes(None).await.unwrap()).unwrap();
        assert!(
            nodes
                .iter()
                .any(|node| node.semantic == parameter.to_string())
        );
        assert!(
            !nodes
                .iter()
                .any(|node| node.semantic == variable.to_string())
        );
        let edges = canonical_analysis_edges::Row::rows(&pre.edges(None).await.unwrap()).unwrap();
        assert!(edges.iter().any(|edge| edge.kind == "numerical_incidence"));
        assert!(edges.iter().any(|edge| edge.kind == "execution_dependency"));
        assert!(edges.iter().all(|edge| edge.evidence_artifact.is_none()));
        let replay = prepared
            .dependency_analysis(&controls, CancellationToken::new())
            .await
            .unwrap();
        assert_ne!(replay.key(), pre.key());
        let replay_receipt = runtime
            .canonical_store()
            .unwrap()
            .analysis(replay.key())
            .await
            .unwrap();
        assert_eq!(replay_receipt.revision, receipt.revision);
        assert_eq!(replay_receipt.method, receipt.method);
        assert_eq!(replay_receipt.configuration, receipt.configuration);
        assert_eq!(replay_receipt.input_digest, receipt.input_digest);
        assert_eq!(replay_receipt.node_count, receipt.node_count);
        assert_eq!(replay_receipt.edge_count, receipt.edge_count);
        let result = prepared.start().unwrap().wait().await.unwrap();
        assert!(result.usable(), "{:?}", result.report());
        let quantified = parametric_sensitivities::Row::rows(
            &result.table("runtime.parametric_sensitivities").unwrap(),
        )
        .unwrap();
        assert_eq!(quantified.len(), 1);
        assert_eq!(quantified[0].parameter_id, parameter);
        assert_eq!(quantified[0].target_id, variable);
        assert!((quantified[0].primal.unwrap() - 1.0).abs() < 1e-10);
        assert!(quantified[0].dual.is_none());
        let run = result.canonical_run_key().unwrap();
        let attempt = result.canonical_attempt_key().unwrap();
        let final_state = runtime
            .promote_scientific_artifact(
                run,
                attempt,
                super::super::artifacts::ScientificArtifactKind::FinalState,
                "user",
                "retained-final-state",
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let post = runtime
            .result_analysis(
                run,
                attempt,
                &AnalysisControls {
                    roots: vec![],
                    direction: AnalysisDirection::Downstream,
                },
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let sources = runtime
            .canonical_store()
            .unwrap()
            .analysis_run_sources(run)
            .await
            .unwrap();
        assert!(sources.len() >= 2);
        let nodes = canonical_analysis_nodes::Row::rows(&post.nodes(None).await.unwrap()).unwrap();
        assert_eq!(
            nodes
                .iter()
                .filter(|node| node.kind == "source_revision")
                .count(),
            sources.len()
        );
        let edges = canonical_analysis_edges::Row::rows(&post.edges(None).await.unwrap()).unwrap();
        let evidence = edges
            .iter()
            .find(|edge| edge.kind == "retained_root_parametric_primal")
            .unwrap();
        assert!(evidence.evidence_artifact.is_some());
        assert_eq!(evidence.evidence_row, Some(0));
        assert_eq!(evidence.evidence_field.as_deref(), Some("primal"));
        assert!(
            !edges
                .iter()
                .any(|edge| edge.kind == "retained_root_parametric_dual")
        );
        let reopened = runtime.analysis(post.key()).await.unwrap();
        assert_eq!(reopened.key(), post.key());
        assert_eq!(
            canonical_analysis_edges::Row::rows(&reopened.edges(None).await.unwrap()).unwrap(),
            edges
        );
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        assert!(
            runtime
                .result_analysis(run, attempt, &controls, cancelled)
                .await
                .is_err()
        );
        let mut retired = false;
        for _ in 0..32 {
            if runtime
                .canonical_store()
                .unwrap()
                .forget_run_results(run)
                .await
                .is_ok()
            {
                retired = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(
            retired,
            "independent graph evidence does not retain the producer run"
        );
        let final_read = runtime
            .canonical_store()
            .unwrap()
            .read_artifact(&final_state.header().key)
            .await
            .unwrap();
        let mut preserved_variables = false;
        for ordinal in 0..final_state.header().block_count {
            let block = final_read.block(ordinal).await.unwrap();
            if block.relation == "runtime.solve_variables" {
                preserved_variables |= block.rows > 0;
                assert!(
                    final_state
                        .block(ordinal, CancellationToken::new())
                        .await
                        .is_ok()
                );
            }
        }
        final_read.release().await.unwrap();
        assert!(preserved_variables);
        let evidence_artifact = runtime
            .scientific_artifact(evidence.evidence_artifact.as_deref().unwrap())
            .await
            .unwrap();
        let retained = evidence_artifact
            .evidence(
                &ArtifactLocator {
                    artifact: evidence.evidence_artifact.clone().unwrap(),
                    block: evidence.evidence_block.unwrap(),
                    row: evidence.evidence_row,
                    field: evidence.evidence_field.clone(),
                },
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let retained = parametric_sensitivities::Row::rows(&retained).unwrap();
        assert_eq!(
            retained, quantified,
            "actual quantitative evidence remains identical after producer retirement"
        );
        // An independently retained evidence artifact survives removal of its
        // graph owner as well as the producer's result retirement.
        runtime
            .canonical_store()
            .unwrap()
            .retain_artifact(
                &evidence_artifact.header().key,
                "user",
                "independent-response-consumer",
            )
            .await
            .unwrap();
        let mut declarations = pse_authoring::language::parse(
            super::super::durable_tests::LINEAR,
            pse_ids::SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        declarations
            .iter_mut()
            .find(|row| row.name == "t" && row.value.binding.is_some())
            .unwrap()
            .value
            .binding
            .as_mut()
            .unwrap()
            .expression = Some("2".into());
        let revised = package.with_declarations(declarations).await.unwrap();
        let changed = revised
            .prepare_analysis(&analysis, &cancel)
            .await
            .unwrap()
            .dependency_analysis(&controls, CancellationToken::new())
            .await
            .unwrap();
        assert_ne!(changed.key(), pre.key());
        assert_eq!(
            runtime
                .canonical_store()
                .unwrap()
                .analysis(pre.key())
                .await
                .unwrap()
                .revision,
            receipt.revision
        );
        assert_eq!(
            runtime
                .canonical_store()
                .unwrap()
                .analysis(changed.key())
                .await
                .unwrap()
                .revision,
            revised.canonical_revision().unwrap().key
        );
        for _ in 0..5 {
            if runtime.forget_analysis_results(post.key()).await.unwrap() {
                break;
            }
        }
        assert!(runtime.analysis(post.key()).await.is_err());
        assert!(
            runtime
                .scientific_artifact(&evidence_artifact.header().key)
                .await
                .unwrap()
                .evidence(
                    &ArtifactLocator {
                        artifact: evidence_artifact.header().key.clone(),
                        block: evidence.evidence_block.unwrap(),
                        row: evidence.evidence_row,
                        field: evidence.evidence_field.clone()
                    },
                    CancellationToken::new(),
                )
                .await
                .is_ok(),
            "graph retirement cannot release another consumer's retained artifact"
        );

        let mut retired = false;
        for _ in 0..32 {
            if runtime
                .canonical_store()
                .unwrap()
                .forget_run_results(run)
                .await
                .is_ok()
            {
                retired = true;
                break;
            }
            tokio::task::yield_now().await;
        }
        assert!(
            retired,
            "retirement remains blocked after exact analysis withdrawal and dropped read owners"
        );
    }
}

#[cfg(test)]
mod analysis_bounds_unit {
    use super::*;

    #[test]
    fn analysis_controls_refuse_before_graph_preparation() {
        let mut controls = AnalysisControls {
            roots: vec!["r".repeat(128); 64],
            direction: AnalysisDirection::Upstream,
        };
        assert!(controls.validate().is_ok());
        controls.roots.push("extra".into());
        assert!(controls.validate().is_err());
        controls.roots = vec!["r".repeat(129)];
        assert!(controls.validate().is_err());
    }

    #[test]
    fn analysis_exact_caps_preserve_complete_membership_and_refuse_overflow() {
        let mut graph = Graph::new();
        let mut nodes = BTreeMap::new();
        for ordinal in 0..ANALYSIS_NODES {
            vertex(&mut graph, &mut nodes, format!("v:{ordinal}"), "variable").unwrap();
        }
        assert_eq!(graph.node_count(), ANALYSIS_NODES);
        // An existing vertex remains valid at the cap; a small selection cannot
        // turn an incompletely admitted source graph into complete evidence.
        assert!(vertex(&mut graph, &mut nodes, "v:0".into(), "variable").is_ok());
        assert!(vertex(&mut graph, &mut nodes, "extra".into(), "variable").is_err());
        let root = nodes["v:0"];
        for _ in 0..ANALYSIS_EDGES {
            edge(&mut graph, root, root, "incidence", None).unwrap();
        }
        assert_eq!(graph.edge_count(), ANALYSIS_EDGES);
        assert!(edge(&mut graph, root, root, "incidence", None).is_err());
        let controls = AnalysisControls {
            roots: vec!["v:0".into()],
            direction: AnalysisDirection::Upstream,
        };
        assert_eq!(
            selected(&graph, &nodes, &controls).unwrap(),
            BTreeSet::from([root])
        );
        assert_eq!(graph.edge_count(), ANALYSIS_EDGES);
    }
}

#[cfg(test)]
#[path = "analyses-response-tests.rs"]
mod response_evidence_unit;
