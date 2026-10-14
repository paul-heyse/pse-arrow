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
    Ok(result)
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
                let owner = Arc::new(
                    MemoryConsumer::new("canonical:analysis-header")
                        .register(&self.runtime.shared.pool()),
                );
                owner
                    .try_grow(12 * 1024 * 1024)
                    .map_err(pse_engine::EngineError::from)?;
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
                let owner = Arc::new(
                    MemoryConsumer::new("canonical:analysis-page")
                        .register(&self.runtime.shared.pool()),
                );
                owner
                    .try_grow(12 * 1024 * 1024)
                    .map_err(pse_engine::EngineError::from)?;
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
                let owner = Arc::new(
                    MemoryConsumer::new("canonical:analysis-page")
                        .register(&self.runtime.shared.pool()),
                );
                owner
                    .try_grow(12 * 1024 * 1024)
                    .map_err(pse_engine::EngineError::from)?;
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

    /// Persist reported response/parametric sensitivity evidence and its exact
    /// provenance. Missing quantitative fields produce no derivative edge; graph
    /// reachability never invents a derivative or promotes outcome-class success.
    pub async fn result_analysis(
        &self,
        run: &str,
        attempt: &str,
        controls: &AnalysisControls,
        cancel: CancellationToken,
    ) -> Result<AnalysisHandle, WorkflowError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        controls.validate()?;
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        let owner = MemoryConsumer::new("canonical:result-analysis").register(&self.shared.pool());
        owner
            .try_grow(128 * 1024 * 1024)
            .map_err(pse_engine::EngineError::from)?;
        let response=super::connected_results::response_owner(self,"canonical:result-analysis-selection-response")?;
        let input = Arc::new(pse_operations::canonical::response_ownership_scope(
            response.clone(), self.canonical_store()?.read_results(run, attempt),
        ).await?);
        let sources = pse_operations::canonical::response_ownership_scope(
            Arc::new((input.clone(),response)),self.canonical_store()?.analysis_run_sources(run),
        ).await?;
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
        let staging_owner=format!("analysis-staging:{}",pse_operations::mint_id::<pse_model::generated::identities::RunId>());
        let artifact=match self.promote_scientific_artifact(run,attempt,super::artifacts::ScientificArtifactKind::LocalAnalysis,"audit",&staging_owner,cancel.clone()).await {
            Ok(artifact)=>artifact,
            Err(source)=>{
                if let WorkflowError::Canonical(pse_operations::canonical::CanonicalError::ArtifactUnsettled{intent,..})=&source {
                    return Err(WorkflowError::ArtifactTransferUnsettled {analysis:None,artifact:intent.clone(),owner_kind:"audit".into(),owner:staging_owner,read:None,source:Box::new(source)});
                }
                return Err(source);
            }
        };
        let transfer_error=|source:WorkflowError,analysis:Option<String>,read:Option<String>|WorkflowError::ArtifactTransferUnsettled {analysis,artifact:Box::new(artifact.header().clone()),owner_kind:"audit".into(),owner:staging_owner.clone(),read,source:Box::new(source)};
        let response=super::connected_results::response_owner(self,"canonical:analysis-artifact-selection-response")?;
        let artifact_read=pse_operations::canonical::response_ownership_scope(response,self.canonical_store()?.read_artifact(&artifact.header().key)).await.map_err(|source|transfer_error(source.into(),None,None))?;
        let read_key=artifact_read.read_key().to_owned();
        let graph_result:Result<AnalysisHandle,WorkflowError>=async {
        for ordinal in 0..artifact.header().block_count {
            let response=super::connected_results::response_owner(self,"canonical:analysis-artifact-response")?;
            let block=pse_operations::canonical::response_ownership_scope(response,artifact_read.block(ordinal)).await?;
            let relation_name=block.relation.as_str();
            if !["runtime.parametric_sensitivities","runtime.response_sensitivities"].contains(&relation_name) {continue;}
            let spec=self.registry().relation(relation_name).ok_or_else(||contract("analysis copied relation unavailable"))?;
            let schema=pse_schema::arrow::relation_schema_ref(self.registry(),spec).map_err(|error|contract(error.to_string()))?;
            let decoded=super::result_files::decode_result_file(block.payload.as_slice(),schema,usize::try_from(block.rows).map_err(|_|contract("analysis row extent"))?)?;
            let batch=pse_relations::columnar::FieldCheckedBatch::admit(self.registry(),spec,decoded,&self.validation_context(),&cancel).map_err(pse_engine::EngineError::from)?;
            let locator=|row,field:Option<&str>|Some(ArtifactLocator {artifact:artifact.header().key.clone(),block:ordinal,row:Some(row),field:field.map(str::to_owned)});
            if relation_name=="runtime.parametric_sensitivities" {
                let view=pse_relations::generated::runtime::parametric_sensitivities::View::from_checked(&batch).map_err(relation)?;
                for index in 0..view.len() {
                    let row=view.row(index).map_err(relation)?;
                    let parameter=vertex(&mut graph,&mut nodes,row.parameter_id.to_string(),"parameter")?;
                    let target_name=if matches!(row.target_kind,pse_model::generated::enums::NumericalTarget::Objective) {format!("objective:step:{}",row.step)} else {row.target_id.to_string()};
                    let target=vertex(&mut graph,&mut nodes,target_name,target_kind(row.target_kind))?;
                    for (field,value) in [("primal",row.primal),("dual",row.dual)] {
                        if value.is_some() { edge(&mut graph,parameter,target,&format!("retained_parametric_{field}"),locator(index as u64,Some(field)))?; }
                    }
                    edge(&mut graph,attempt_node,target,"produced_evidence",locator(index as u64,None))?;
                }
            } else {
                let view=pse_relations::generated::runtime::response_sensitivities::View::from_checked(&batch).map_err(relation)?;
                for index in 0..view.len() {
                    let row=view.row(index).map_err(relation)?;
                    let parameter=vertex(&mut graph,&mut nodes,row.parameter_id.to_string(),"parameter")?;
                    let output=vertex(&mut graph,&mut nodes,row.output_id.to_string(),"output")?;
                    edge(&mut graph,parameter,output,"retained_response_sensitivity",locator(index as u64,Some("value")))?;
                    edge(&mut graph,attempt_node,output,"produced_evidence",locator(index as u64,None))?;
                }
            }
        }
        let configuration = serde_json::to_vec(&(1_u8, controls, run, attempt))
            .map_err(|error| contract(error.to_string()))?;
        let handle=self.store_analysis_graph(
            &revision,
            &sources,
            std::slice::from_ref(&artifact_read),
            "result-sensitivity-provenance-reachability:v1",
            configuration,
            graph,
            nodes,
            controls,
        )
        .await?;
        Ok(handle)
        }.await;
        let handle=graph_result.map_err(|source|{
            let analysis=match &source {WorkflowError::Canonical(pse_operations::canonical::CanonicalError::AnalysisUnsettled{intent,..})=>Some(intent.key.clone()),_=>None};
            transfer_error(source,analysis,Some(read_key.clone()))
        })?;
        self.canonical_store()?.release_artifact_intent(artifact.header(),"audit",&staging_owner).await.map_err(|source|transfer_error(source.into(),Some(handle.key().to_owned()),Some(read_key.clone())))?;
        artifact_read.release().await.map_err(|source|transfer_error(source.into(),Some(handle.key().to_owned()),Some(read_key)))?;
        Ok(handle)

        }).await
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
            .find(|edge| edge.kind == "retained_parametric_primal")
            .unwrap();
        assert!(evidence.evidence_artifact.is_some());
        assert_eq!(evidence.evidence_row, Some(0));
        assert_eq!(evidence.evidence_field.as_deref(), Some("primal"));
        assert!(
            !edges
                .iter()
                .any(|edge| edge.kind == "retained_parametric_dual")
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
