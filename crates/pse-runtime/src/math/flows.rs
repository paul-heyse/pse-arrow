// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Accounted pure flow preparation and native/library tear selection.
use super::{MathRuntimeError, MathService, Workspace, solves::SolveHandle};
use pse_backend_native::{ProblemError, solve::*, tears};
use pse_engine::cache_service::flight::FlightCancellation;
use pse_ids::SemanticId;
use std::sync::Arc;
/// Immutable physically admitted compiler product and its allocation owner.
#[derive(Clone, Debug)]
pub struct PreparedFlow {
    graph: Arc<pse_structural::flowsheet::FlowGraph>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl PreparedFlow {
    /// Complete physical graph, SCCs, bindings and source identities.
    pub fn graph(&self) -> &pse_structural::flowsheet::FlowGraph {
        &self.graph
    }
}
/// Tear policy is explicit; a heuristic never substitutes for an unavailable optimizer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TearMethod {
    /// Exact weighted feedback-edge MILP with native incumbent/bound/gap reporting.
    Highs,
    /// Explicit unweighted petgraph greedy feedback arc set.
    UnweightedHeuristic,
}
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
    /// Project physical flow declarations through the same pure Salsa workspace.
    pub async fn prepare_flow(
        self: &Arc<Self>,
        workspace: Workspace,
        revision: pse_compiler::workspace::Inputs,
        id: SemanticId,
    ) -> Result<PreparedFlow, MathRuntimeError> {
        let owner = self.reserve("math:flow-product", self.policy.workspace_bytes)?;
        let graph = self
            .job(
                1,
                self.policy.stack_bytes,
                FlightCancellation::default(),
                move |_| {
                    let _lease = workspace.lease;
                    let mut compiler = workspace.compiler.lock().map_err(|_| {
                        MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                    })?;
                    compiler.publish(revision)?;
                    compiler.prepare_flow(id).map_err(Into::into)
                },
            )
            .await?;
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
        let worker_bytes = controls
            .threads
            .saturating_sub(1)
            .checked_mul(self.policy.stack_bytes)
            .and_then(|v| v.checked_add(self.policy.worker_bytes))
            .ok_or(MathRuntimeError::Limit("tear worker stack allowance"))?;
        let cancel = FlightCancellation::default();
        let control = cancel.clone();
        let progress = Arc::new(Progress::new(controls.history));
        let events = progress.clone();
        let service = self.clone();
        let (tx, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let result = service
                .job(controls.threads, worker_bytes, control, move |flag| {
                    let mut execution = Execution::new(flag, &controls);
                    execution.progress = events;
                    if execution.stopped().is_some() {
                        return Err(MathRuntimeError::Cancelled);
                    }
                    let (selected, attempt) = match method {
                        TearMethod::UnweightedHeuristic => {
                            (Some(tears::heuristic(&flow.graph)?), None)
                        }
                        TearMethod::Highs if flow.graph.declaration().connections.is_empty() => {
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
                    Ok(TearResult {
                        selected: selected.map(|v| v.with_owner(owner.clone())),
                        attempt,
                        _owner: owner,
                    })
                })
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
