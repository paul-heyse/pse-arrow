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
    ANALYSIS_EDGES, ANALYSIS_NODES, Analysis, AnalysisEdge, AnalysisNode, analysis_key,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
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
type Graph = DiGraph<(String, String), (String, Option<String>)>;
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
    evidence: Option<String>,
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
    if controls.roots.len() > 64 || controls.roots.iter().any(|root| root.len() > 128) {
        return Err(contract("analysis root bound"));
    }
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
        use pse_relations::generated::runtime::canonical_analyses as rows;
        let owner =
            MemoryConsumer::new("canonical:analysis-header").register(&self.runtime.shared.pool());
        owner
            .try_grow(12 * 1024 * 1024)
            .map_err(pse_engine::EngineError::from)?;
        let row = self.runtime.canonical_store().analysis(&self.key).await?;
        let validation = self.runtime.validation_context()?;
        let mut builder = rows::Builder::with_registry(&self.runtime.registry, 1, &validation)
            .map_err(relation)?;
        builder.push(row).map_err(relation)?;
        builder
            .finish()
            .map_err(relation)?
            .retained(&self.runtime.shared.pool(), &CancellationToken::new())
            .map_err(relation)
    }
    /// Native ordered node page, at most 64 rows, retaining actual Arrow buffers.
    pub async fn nodes(
        &self,
        after: Option<&str>,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        use pse_relations::generated::runtime::canonical_analysis_nodes as rows;
        let owner =
            MemoryConsumer::new("canonical:analysis-page").register(&self.runtime.shared.pool());
        owner
            .try_grow(12 * 1024 * 1024)
            .map_err(pse_engine::EngineError::from)?;
        let page = self
            .runtime
            .canonical_store()
            .analysis_node_page(&self.key, after)
            .await?;
        let validation = self.runtime.validation_context()?;
        let mut builder =
            rows::Builder::with_registry(&self.runtime.registry, page.len(), &validation)
                .map_err(relation)?;
        for row in page {
            builder.push(row).map_err(relation)?;
        }
        builder
            .finish()
            .map_err(relation)?
            .retained(&self.runtime.shared.pool(), &CancellationToken::new())
            .map_err(relation)
    }
    /// Native directed graph edge page with original result evidence references.
    pub async fn edges(
        &self,
        after: Option<&str>,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        use pse_relations::generated::runtime::canonical_analysis_edges as rows;
        let owner =
            MemoryConsumer::new("canonical:analysis-page").register(&self.runtime.shared.pool());
        owner
            .try_grow(12 * 1024 * 1024)
            .map_err(pse_engine::EngineError::from)?;
        let page = self
            .runtime
            .canonical_store()
            .analysis_edge_page(&self.key, after)
            .await?;
        let validation = self.runtime.validation_context()?;
        let mut builder =
            rows::Builder::with_registry(&self.runtime.registry, page.len(), &validation)
                .map_err(relation)?;
        for row in page {
            builder.push(row).map_err(relation)?;
        }
        builder
            .finish()
            .map_err(relation)?
            .retained(&self.runtime.shared.pool(), &CancellationToken::new())
            .map_err(relation)
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
        self.operations()?
            .record(run, attempt)
            .await?
            .completion
            .ok_or_else(|| contract("stored completion metadata absent"))
    }
    /// Reopen only an active, retained exact analysis; no latest-run substitution.
    pub async fn analysis(&self, key: &str) -> Result<AnalysisHandle, WorkflowError> {
        self.canonical_store().analysis(key).await?;
        Ok(AnalysisHandle {
            runtime: self.clone(),
            key: key.into(),
        })
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "one analysis admission binds exact source and result lineage, method/configuration, graph membership and selection controls"
    )]
    async fn store_analysis_graph(
        &self,
        revision: &pse_operations::canonical::Revision,
        sources: &[pse_operations::canonical::Revision],
        inputs: &[pse_operations::canonical_results::ResultRead],
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
                .str(&input.run().key)
                .str(&input.attempt().key)
                .str(&input.manifest().digest);
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
                .str(evidence.as_deref().unwrap_or(""));
        }
        let input_digest = input_hash.finish_hash().to_hex();
        let key = analysis_key(
            "pse.analysis.v1",
            &[
                revision.key.as_bytes(),
                method.as_bytes(),
                &configuration,
                input_digest.as_bytes(),
            ],
        );
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
            let edge_key = analysis_key(
                "pse.analysis.edge.v1",
                &[
                    key.as_bytes(),
                    source.as_bytes(),
                    target.as_bytes(),
                    kind.as_bytes(),
                    evidence.as_deref().unwrap_or("").as_bytes(),
                ],
            );
            retained_edges.push(AnalysisEdge {
                key: edge_key,
                analysis: key.clone(),
                source,
                target,
                kind,
                evidence,
            });
        }
        let header = Analysis {
            key: key.clone(),
            revision: revision.key.clone(),
            method: method.into(),
            configuration: configuration.into(),
            input_digest,
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
            node_count: retained_nodes.len() as u64,
            edge_count: retained_edges.len() as u64,
            active: false,
        };
        self.canonical_store()
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
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        let owner = MemoryConsumer::new("canonical:result-analysis").register(&self.shared.pool());
        owner
            .try_grow(128 * 1024 * 1024)
            .map_err(pse_engine::EngineError::from)?;
        let input = self
            .canonical_store()
            .read_results(run, attempt, Duration::from_secs(600))
            .await?;
        let sources = self.canonical_store().analysis_run_sources(run).await?;
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
        for relation_name in [
            "runtime.parametric_sensitivities",
            "runtime.response_sensitivities",
        ] {
            cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
            let spec = self
                .registry
                .relation(relation_name)
                .ok_or_else(|| contract("analysis relation unavailable"))?;
            if !input
                .sets()
                .iter()
                .any(|set| set.name == spec.id.to_string())
            {
                continue;
            }
            // The complete evidence relation must fit this explicit bounded graph
            // campaign; refusal avoids silently treating a prefix as complete.
            let descriptor = input
                .sets()
                .iter()
                .find(|set| set.name == spec.id.to_string())
                .ok_or_else(|| contract("analysis descriptor absent"))?;
            if descriptor.row_count > 4096 {
                return Err(contract(
                    "analysis evidence relation exceeds bounded campaign",
                ));
            }
            let mut reader = self
                .results(run, attempt, relation_name, 0, u64::MAX, cancel.clone())
                .await?;
            while let Some(batch) = reader.next_relation_batch().await? {
                input.renew(Duration::from_secs(600)).await?;
                let (batch_key, start) = reader
                    .last_receipt()
                    .ok_or_else(|| contract("analysis original batch receipt absent"))?
                    .clone();
                if relation_name == "runtime.parametric_sensitivities" {
                    let view=pse_relations::generated::runtime::parametric_sensitivities::View::from_checked(&batch).map_err(relation)?;
                    for index in 0..view.len() {
                        let row = view.row(index).map_err(relation)?;
                        let parameter = vertex(
                            &mut graph,
                            &mut nodes,
                            row.parameter_id.to_string(),
                            "parameter",
                        )?;
                        let target_name = if matches!(
                            row.target_kind,
                            pse_model::generated::enums::NumericalTarget::Objective
                        ) {
                            format!("objective:step:{}", row.step)
                        } else {
                            row.target_id.to_string()
                        };
                        let target = vertex(
                            &mut graph,
                            &mut nodes,
                            target_name,
                            target_kind(row.target_kind),
                        )?;
                        for (field, value) in [("primal", row.primal), ("dual", row.dual)] {
                            if value.is_some() {
                                let evidence = format!(
                                    "ipc:{}:{batch_key}:{}:{field}",
                                    spec.id,
                                    start + index as u64
                                );
                                edge(
                                    &mut graph,
                                    parameter,
                                    target,
                                    &format!("retained_parametric_{field}"),
                                    Some(evidence),
                                )?;
                            }
                        }
                        edge(
                            &mut graph,
                            attempt_node,
                            target,
                            "produced_evidence",
                            Some(format!(
                                "ipc:{}:{batch_key}:{}",
                                spec.id,
                                start + index as u64
                            )),
                        )?;
                    }
                } else {
                    let view=pse_relations::generated::runtime::response_sensitivities::View::from_checked(&batch).map_err(relation)?;
                    for index in 0..view.len() {
                        let row = view.row(index).map_err(relation)?;
                        let parameter = vertex(
                            &mut graph,
                            &mut nodes,
                            row.parameter_id.to_string(),
                            "parameter",
                        )?;
                        let output =
                            vertex(&mut graph, &mut nodes, row.output_id.to_string(), "output")?;
                        edge(
                            &mut graph,
                            parameter,
                            output,
                            "retained_response_sensitivity",
                            Some(format!(
                                "ipc:{}:{batch_key}:{}:value",
                                spec.id,
                                start + index as u64
                            )),
                        )?;
                        edge(
                            &mut graph,
                            attempt_node,
                            output,
                            "produced_evidence",
                            Some(format!(
                                "ipc:{}:{batch_key}:{}",
                                spec.id,
                                start + index as u64
                            )),
                        )?;
                    }
                }
            }
        }
        let configuration = serde_json::to_vec(&(1_u8, controls, run, attempt))
            .map_err(|error| contract(error.to_string()))?;
        self.store_analysis_graph(
            &revision,
            &sources,
            &[input],
            "result-sensitivity-provenance-reachability:v1",
            configuration,
            graph,
            nodes,
            controls,
        )
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
        let runtime = &self.source.runtime;
        let owner =
            MemoryConsumer::new("canonical:dependency-analysis").register(&runtime.shared.pool());
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
                        pse_math::binding::Target::Objective(level) => format!("objective:{level}"),
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
        let revision = self.source.canonical_revision().clone();
        let mut sources = vec![revision.clone()];
        let operations = match runtime.durability() {
            Durability::Durable(operations) => operations.clone(),
            Durability::Ephemeral => super::Operations::from_store(
                runtime.canonical_store().clone(),
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
        let receipt = runtime.canonical_store().analysis(pre.key()).await.unwrap();
        assert!(receipt.active);
        assert_eq!(
            receipt.method,
            "original-incidence-execution-reachability:v1"
        );
        assert_eq!(receipt.revision, package.canonical_revision().key);
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
        assert!(edges.iter().all(|edge| edge.evidence.is_none()));
        let replay = prepared
            .dependency_analysis(&controls, CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(replay.key(), pre.key());
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
            .unwrap()
            .evidence
            .as_ref()
            .unwrap();
        assert!(evidence.ends_with(":0:primal"));
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
        assert!(
            runtime
                .canonical_store()
                .forget_run_results(run)
                .await
                .is_err()
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
                .analysis(pre.key())
                .await
                .unwrap()
                .revision,
            receipt.revision
        );
        assert_eq!(
            runtime
                .canonical_store()
                .analysis(changed.key())
                .await
                .unwrap()
                .revision,
            revised.canonical_revision().key
        );
        runtime.forget_analysis_results(post.key()).await.unwrap();
        assert!(runtime.analysis(post.key()).await.is_err());
        let mut retired = false;
        for _ in 0..32 {
            if runtime
                .canonical_store()
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
