// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Accounted pure flow preparation and native/library tear selection.
use super::{MathRuntimeError, MathService, solves::SolveHandle};
use pse_backend_native::{ProblemError, solve::*, tears};
use pse_columnar::flight::FlightCancellation;
/// Authored topology selection and explicit tear policies for public workflow callers.
pub use pse_compiler::workspace::{
    FlowConnectionDocument, FlowSelectionDocument, ModelingFlowSelection,
};
use pse_ids::SemanticId;
pub use pse_structural::flowsheet::{Decision, Policy};
use std::sync::Arc;
/// Immutable physically admitted compiler product and its allocation owner.
#[derive(Clone, Debug)]
pub struct PreparedFlow {
    graph: Arc<pse_structural::flowsheet::FlowGraph>,
    _owner: Arc<super::products::ProductOwner>,
}
impl PreparedFlow {
    /// Complete physical graph, SCCs, bindings and source identities.
    pub fn graph(&self) -> &pse_structural::flowsheet::FlowGraph {
        &self.graph
    }
}
/// Tear policy is explicit; a heuristic never substitutes for an unavailable optimizer. A
/// registry vocabulary whose spelling is its boundary name (ADR-0115 Outcome 3).
pub use pse_model::generated::enums::TearMethod;
/// Native attempt and independently checked graph result retain accounted ownership.
#[derive(Debug)]
pub struct TearResult {
    /// Available independently checked tear set; absent if no native incumbent exists.
    pub selected: Option<tears::TearReport>,
    /// Actual native report, including attempts without usable graph results.
    pub attempt: Option<SolveReport>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl MathService {
    /// Project an immutable authored model with explicitly selected nodes and tear policies.
    pub async fn prepare_modeling_flow(
        self: &Arc<Self>,
        model: pse_compiler::workspace::SemanticModeling,
        quantities: Arc<pse_quantity::QuantityRegistry>,
        selection: ModelingFlowSelection,
        driver: &crate::CancelSource,
    ) -> Result<PreparedFlow, MathRuntimeError> {
        let demand = model.flow_allocation_bound(&selection, &quantities)?;
        if demand > self.policy.workspace_bytes {
            return Err(MathRuntimeError::Limit("flow construction capacity"));
        }
        let control = FlightCancellation::default();
        let operation = self.job_retained(1, demand, control.clone(), move |flag| {
            if flag.load(std::sync::atomic::Ordering::Relaxed) {
                return Err(MathRuntimeError::Cancelled);
            }
            let graph = Arc::new(model.flow_graph(&selection, &quantities)?);
            let bytes = graph
                .retained_allocation_bound()
                .map_err(pse_compiler::workspace::CompileError::from)?;
            Ok((graph, bytes))
        });
        tokio::pin!(operation);
        let (graph, lease) = tokio::select! {r=&mut operation=>r?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        let owner = self.shared_product(
            vec![2, Arc::as_ptr(&graph) as usize],
            graph.clone(),
            lease,
            Vec::new(),
        )?;
        Ok(PreparedFlow {
            graph,
            _owner: owner,
        })
    }
    /// Run the selected tear method with the complete native lifetime inside admission.
    pub fn select_tears(
        self: &Arc<Self>,
        flow: PreparedFlow,
        method: TearMethod,
        controls: Controls,
    ) -> Result<SolveHandle<TearResult>, MathRuntimeError> {
        controls.validate()?;
        // A tear selection is one fresh attempt: there is no retained seed or allocation.
        if controls.start != StartPolicy::NoPriorStart || controls.reuse != ReusePolicy::Fresh {
            return Err(ProblemError::Contract(
                "tear selection has no retained seed or allocation".into(),
            )
            .into());
        }
        if method == TearMethod::Highs && !cfg!(feature = "solver-highs") {
            return Err(ProblemError::Unavailable {
                backend: Backend::Highs,
                alternatives: vec![],
            }
            .into());
        }
        let d = flow.graph.declaration();
        let bytes = d
            .nodes
            .len()
            .checked_add(d.connections.len())
            .and_then(|v| v.checked_add(d.decisions.len()))
            .and_then(|v| v.checked_mul(1024))
            .and_then(|v| v.checked_add(controls.report_allowance().ok()?))
            .ok_or(MathRuntimeError::Limit("tear result allowance"))?;
        let owner = self.reserve("math:tear-results", bytes)?;
        let worker_bytes = if method == TearMethod::UnweightedHeuristic || d.connections.is_empty()
        {
            flow.graph
                .tear_allocation_bound()
                .map_err(pse_compiler::workspace::CompileError::from)?
        } else {
            tears::construction_allocation_bound(&flow.graph)?
        };
        if worker_bytes > self.policy.worker_bytes {
            return Err(MathRuntimeError::Limit("tear construction capacity"));
        }
        let cancel = FlightCancellation::default();
        let deadline = std::time::Instant::now()
            .checked_add(controls.time_limit)
            .ok_or(MathRuntimeError::Limit("tear task deadline"))?;
        let entry = self.admit_entry(
            controls.threads,
            worker_bytes,
            &cancel,
            Some(deadline),
            None,
        )?;
        let scope = pse_kernels::ExecutionScope::new(cancel.flag(), Some(deadline));
        let control = cancel.clone();
        let progress = Arc::new(Progress::new(controls.history));
        let events = progress.clone();
        let service = self.clone();
        let (tx, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let result = service
                .job_scoped_on_entry(
                    controls.threads,
                    worker_bytes,
                    control,
                    Some(deadline),
                    entry,
                    move |flag| {
                        let mut execution = Execution::within(flag, &controls, scope.clone())?;
                        execution.progress = events;
                        execution.check()?;
                        let (selected, attempt) = match method {
                            TearMethod::UnweightedHeuristic => {
                                (Some(tears::heuristic(&flow.graph)?), None)
                            }
                            TearMethod::Highs
                                if flow.graph.declaration().connections.is_empty() =>
                            {
                                (Some(tears::heuristic(&flow.graph)?), None)
                            }
                            #[cfg(feature = "solver-highs")]
                            TearMethod::Highs => {
                                let (s, r) = tears::solve(&flow.graph, &controls, execution)?;
                                (s, Some(r.with_owner(owner.clone())))
                            }
                            #[cfg(not(feature = "solver-highs"))]
                            TearMethod::Highs => {
                                return Err(ProblemError::Unavailable {
                                    backend: Backend::Highs,
                                    alternatives: vec![],
                                }
                                .into());
                            }
                        };
                        scope.check().map_err(ProblemError::from)?;
                        Ok(TearResult {
                            selected: selected.map(|v| v.with_owner(owner.clone())),
                            attempt,
                            _owner: owner,
                        })
                    },
                )
                .await;
            let _ = tx.send(result);
        });
        Ok(SolveHandle {
            cancel,
            receiver: Some(receiver),
            progress,
        })
    }
}

/// Public immutable projection of the selected, physically admitted flow graph.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FlowGraphDocument {
    #[doc = "Identity of the exact admitted flow graph."]
    pub identity: pse_ids::ContentHash,
    #[doc = "Selected graph nodes."]
    pub nodes: Vec<FlowNodeDocument>,
    #[doc = "Selected authored connections."]
    pub connections: Vec<FlowEdgeDocument>,
    #[doc = "Grouped tear decisions."]
    pub decisions: Vec<FlowDecisionDocument>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[doc = "One admitted node and its physical ports."]
pub struct FlowNodeDocument {
    #[doc = "Identity retained from the admitted source."]
    pub id: SemanticId,
    #[doc = "Selected authored ports."]
    pub ports: Vec<FlowPortDocument>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[doc = "One admitted port with its physical meaning."]
pub struct FlowPortDocument {
    #[doc = "Identity retained from the admitted source."]
    pub id: SemanticId,
    #[doc = "Physical quantity identity."]
    pub quantity_id: SemanticId,
    #[doc = "Physical unit identity."]
    pub unit_id: SemanticId,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[doc = "One admitted directed edge and its port bindings."]
pub struct FlowEdgeDocument {
    #[doc = "Identity retained from the admitted source."]
    pub id: SemanticId,
    #[doc = "Source endpoint identity."]
    pub from: SemanticId,
    #[doc = "Destination endpoint identity."]
    pub to: SemanticId,
    #[doc = "Grouped tear decision identity."]
    pub decision: SemanticId,
    #[doc = "Admitted source and destination port bindings."]
    pub bindings: Vec<(SemanticId, SemanticId)>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[doc = "One grouped tear decision and canonical permission."]
pub struct FlowDecisionDocument {
    #[doc = "Identity retained from the admitted source."]
    pub id: SemanticId,
    #[doc = "Native tear objective cost."]
    pub cost: f64,
    #[doc = "Canonical tear permission."]
    pub policy: Policy,
}
impl PreparedFlow {
    /// Observe its admitted topology without decoding or rebuilding semantic meaning.
    pub fn document(&self) -> FlowGraphDocument {
        let graph = self.graph();
        let declaration = graph.declaration();
        FlowGraphDocument {
            identity: graph.key(),
            nodes: declaration
                .nodes
                .iter()
                .map(|node| FlowNodeDocument {
                    id: node.id,
                    ports: node
                        .ports
                        .iter()
                        .map(|port| FlowPortDocument {
                            id: port.id,
                            quantity_id: port.quantity.as_id(),
                            unit_id: port.unit.as_id(),
                        })
                        .collect(),
                })
                .collect(),
            connections: declaration
                .connections
                .iter()
                .map(|edge| FlowEdgeDocument {
                    id: edge.id,
                    from: edge.from,
                    to: edge.to,
                    decision: edge.decision,
                    bindings: edge.bindings.clone(),
                })
                .collect(),
            decisions: declaration
                .decisions
                .iter()
                .map(|decision| FlowDecisionDocument {
                    id: decision.id,
                    cost: decision.cost,
                    policy: decision.policy,
                })
                .collect(),
        }
    }
}
/// Available independently verified tear evidence; absence remains separate from native attempts.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TearSelectionDocument {
    #[doc = "Grouped tear decisions."]
    pub decisions: Vec<SemanticId>,
    #[doc = "Selected authored connections."]
    pub connections: Vec<SemanticId>,
    #[doc = "Admitted evaluation order."]
    pub order: Vec<SemanticId>,
    #[doc = "Native tear objective cost."]
    pub cost: f64,
    #[doc = "Native method and evidence description."]
    pub method: String,
}
impl TearResult {
    /// Project retained selected tears without executing another native operation.
    pub fn document(&self) -> Option<TearSelectionDocument> {
        self.selected
            .as_ref()
            .map(|selected| TearSelectionDocument {
                decisions: selected.decisions.iter().copied().collect(),
                connections: selected.connections.clone(),
                order: selected.order.clone(),
                cost: selected.cost,
                method: selected.method.into(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn structural_tear_deadline_includes_cpu_wait_without_cancellation() {
        let service = super::super::tests::service();
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let graph = Arc::new(
            pse_structural::flowsheet::FlowGraph::admit(
                pse_structural::flowsheet::Declaration {
                    nodes: vec![pse_structural::flowsheet::Node {
                        id: SemanticId::from_bytes([1; 16]),
                        ports: vec![],
                    }],
                    connections: vec![],
                    decisions: vec![],
                },
                &registry,
                pse_structural::projection::GraphLimits { nodes: 1, edges: 0 },
            )
            .unwrap(),
        );
        let lease = service.reserve("test:structural-flow", 1024).unwrap();
        let owner = service
            .shared_product(
                vec![2, Arc::as_ptr(&graph) as usize],
                graph.clone(),
                lease,
                vec![],
            )
            .unwrap();
        let flow = PreparedFlow {
            graph,
            _owner: owner,
        };
        let permit = service.cpu.clone().acquire_many_owned(2).await.unwrap();
        let controls = Controls {
            time_limit: std::time::Duration::from_millis(25),
            ..Controls::default()
        };
        let handle = service
            .select_tears(flow.clone(), TearMethod::UnweightedHeuristic, controls)
            .unwrap();
        let cancellation = handle.cancel.flag();
        let result = tokio::time::timeout(std::time::Duration::from_secs(1), handle.finish())
            .await
            .unwrap();
        assert!(matches!(
            result,
            Err(MathRuntimeError::Solve(ProblemError::Limit {
                kind: pse_backend_native::LimitKind::Time,
                ..
            }))
        ));
        assert!(!cancellation.load(std::sync::atomic::Ordering::Acquire));
        assert_eq!(service.cpu.available_permits(), 0);
        drop(permit);
        let result = service
            .select_tears(flow, TearMethod::UnweightedHeuristic, Controls::default())
            .unwrap()
            .finish()
            .await
            .unwrap();
        assert!(result.selected.is_some());
        assert!(result.attempt.is_none());
        assert_eq!(service.cpu.available_permits(), 2);
    }
}

#[cfg(test)]
mod demand_tests {
    use super::*;
    #[tokio::test(flavor = "current_thread")]
    async fn tear_submission_burst_owns_ticket_before_spawning_attempt() {
        use pse_structural::{
            flowsheet::{Declaration, FlowGraph, Node},
            projection::GraphLimits,
        };
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy {
                jobs: 32,
                foreign_bytes: 1 << 20,
                ..Default::default()
            },
        );
        let registry = pse_quantity::QuantityRegistryBuilder::new()
            .build()
            .unwrap();
        let graph = Arc::new(
            FlowGraph::admit(
                Declaration {
                    nodes: vec![Node {
                        id: SemanticId::from_bytes([1; 16]),
                        ports: vec![],
                    }],
                    connections: vec![],
                    decisions: vec![],
                },
                &registry,
                GraphLimits { nodes: 1, edges: 0 },
            )
            .unwrap(),
        );
        let lease = service
            .reserve("flow:burst-source", graph.tear_allocation_bound().unwrap())
            .unwrap();
        let owner = service
            .shared_product(vec![92], graph.clone(), lease, vec![])
            .unwrap();
        let flow = PreparedFlow {
            graph,
            _owner: owner,
        };
        let baseline = service.pool.reserved();
        let cpu = service.cpu.clone().acquire_many_owned(2).await.unwrap();
        let mut handles = Vec::new();
        for _ in 0..32 {
            handles.push(
                service
                    .select_tears(
                        flow.clone(),
                        TearMethod::UnweightedHeuristic,
                        Controls::default(),
                    )
                    .unwrap(),
            );
        }
        assert_eq!(service.jobs.available_permits(), 0);
        assert!(matches!(
            service.select_tears(
                flow.clone(),
                TearMethod::UnweightedHeuristic,
                Controls::default()
            ),
            Err(MathRuntimeError::Limit("native jobs"))
        ));
        for handle in &handles {
            handle.cancel();
        }
        for handle in handles {
            assert!(matches!(
                handle.finish().await,
                Err(MathRuntimeError::Cancelled)
            ));
        }
        assert_eq!(service.jobs.available_permits(), 32);
        assert_eq!(service.pool.reserved(), baseline);
        drop(cpu);
        assert_eq!(service.cpu.available_permits(), 2);
        let result = service
            .select_tears(flow, TearMethod::UnweightedHeuristic, Controls::default())
            .unwrap()
            .finish()
            .await
            .unwrap();
        assert_eq!(result.selected.as_ref().unwrap().order.len(), 1);
        drop(result);
        assert_eq!(service.pool.reserved(), 0);
    }
    #[tokio::test]
    async fn heuristic_flow_uses_source_demand_under_generous_worker_capacity() {
        use pse_structural::{
            flowsheet::{Declaration, FlowGraph, Node},
            projection::GraphLimits,
        };
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy {
                worker_bytes: 16usize << 30,
                foreign_bytes: 1 << 20,
                ..Default::default()
            },
        );
        let registry = pse_quantity::QuantityRegistryBuilder::new()
            .build()
            .unwrap();
        let graph = Arc::new(
            FlowGraph::admit(
                Declaration {
                    nodes: vec![
                        Node {
                            id: SemanticId::from_bytes([1; 16]),
                            ports: vec![],
                        },
                        Node {
                            id: SemanticId::from_bytes([2; 16]),
                            ports: vec![],
                        },
                    ],
                    connections: vec![],
                    decisions: vec![],
                },
                &registry,
                GraphLimits { nodes: 8, edges: 8 },
            )
            .unwrap(),
        );
        let lease = service
            .reserve("flow:test-source", graph.tear_allocation_bound().unwrap())
            .unwrap();
        let owner = service
            .shared_product(vec![91], graph.clone(), lease, vec![])
            .unwrap();
        let flow = PreparedFlow {
            graph,
            _owner: owner,
        };
        let result = service
            .select_tears(flow, TearMethod::UnweightedHeuristic, Controls::default())
            .unwrap()
            .finish()
            .await
            .unwrap();
        assert_eq!(result.selected.as_ref().unwrap().order.len(), 2);
        assert!(result.attempt.is_none());
        drop(result);
        assert_eq!(service.pool.reserved(), 0);
    }
}
