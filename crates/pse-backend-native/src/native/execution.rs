// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::Solve;
use datafusion::{
    catalog::Session,
    common::{DataFusionError, Result},
    logical_expr::{
        LogicalPlan, UserDefinedLogicalNode, physical_planning_context::PhysicalPlanningContext,
    },
    physical_plan::ExecutionPlan,
    physical_planner::{ExtensionPlanner, PhysicalPlanner},
};
use pse_catalog::session::execution::NativeExecutionContext;
use std::{num::NonZeroUsize, sync::Arc};

/// Native physical planner with an explicit shared solver concurrency allowance.
/// Clone/reuse this planner across sessions to retain the same admission semaphore.
/// Ipopt/MUMPS must use the qualified single-thread libraries; permits bound solver
/// invocations, not the number of threads in an unqualified BLAS installation.
#[derive(Debug)]
pub struct NativeSolverPlanner {
    #[cfg(feature = "ipopt")]
    permits: Arc<tokio::sync::Semaphore>,
    #[cfg(feature = "ipopt")]
    foreign_bytes: usize,
}
impl NativeSolverPlanner {
    /// Select concurrent solver slots and a per-run foreign allocation allowance.
    /// The allowance is charged to the actual session budget before constructing
    /// Ipopt. It does not claim to intercept all C allocations or limit process RSS.
    pub fn new(concurrency: NonZeroUsize, foreign_bytes: NonZeroUsize) -> Self {
        #[cfg(not(feature = "ipopt"))]
        let _ = (concurrency, foreign_bytes);
        Self {
            #[cfg(feature = "ipopt")]
            permits: Arc::new(tokio::sync::Semaphore::new(concurrency.get())),
            #[cfg(feature = "ipopt")]
            foreign_bytes: foreign_bytes.get(),
        }
    }
}
#[async_trait::async_trait]
impl ExtensionPlanner for NativeSolverPlanner {
    async fn plan_extension(
        &self,
        _: &dyn PhysicalPlanner,
        node: &dyn UserDefinedLogicalNode,
        _: &[&LogicalPlan],
        inputs: &[Arc<dyn ExecutionPlan>],
        session: &dyn Session,
        _: &PhysicalPlanningContext,
    ) -> Result<Option<Arc<dyn ExecutionPlan>>> {
        let Some(node) = node.as_any().downcast_ref::<Solve>() else {
            return Ok(None);
        };
        let services = NativeExecutionContext::from_session(session)?;
        services
            .admit_effects(&std::collections::BTreeSet::from([
                pse_schema::model::provider::OperationEffect::Read,
                pse_schema::model::provider::OperationEffect::Nondeterministic,
            ]))
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        #[cfg(not(feature = "ipopt"))]
        {
            let _ = (node, inputs);
            Err(DataFusionError::External(Box::new(
                pse_catalog::CatalogError::Semantic(Arc::new(crate::NativeError::Capability(
                    "native solver requires the ipopt build capability".into(),
                ))),
            )))
        }
        #[cfg(feature = "ipopt")]
        linked::prepare(self, node, inputs, session, services).map(Some)
    }
}

#[cfg(feature = "ipopt")]
mod linked {
    use super::{
        Arc, DataFusionError, ExecutionPlan, NativeExecutionContext, NativeSolverPlanner, Result,
        Session, Solve,
    };
    use crate::{driver, native::Specification};
    use datafusion::{
        execution::{TaskContext, session_state::SessionState},
        logical_expr::UserDefinedLogicalNodeCore,
        physical_expr::EquivalenceProperties,
        physical_plan::{
            DisplayAs, DisplayFormatType, ExecutionPlanProperties, Partitioning, PlanProperties,
            SendableRecordBatchStream, execute_stream,
            execution_plan::{Boundedness, EmissionType},
            stream::RecordBatchStreamAdapter,
        },
    };
    use futures_util::TryStreamExt;
    use pse_ids::CancellationToken;
    use pse_numerics::EvaluationProgram;

    pub(super) fn prepare(
        planner: &NativeSolverPlanner,
        node: &Solve,
        inputs: &[Arc<dyn ExecutionPlan>],
        session: &dyn Session,
        services: Arc<NativeExecutionContext>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        driver::qualify_version().map_err(error)?;
        let [input] = inputs else {
            return Err(DataFusionError::Plan("solve requires one child".into()));
        };
        if input.boundedness().is_unbounded()
            || input.schema().as_ref() != node.input.schema().as_arrow()
        {
            return Err(DataFusionError::Plan(
                "solve needs the exact bounded numerical child".into(),
            ));
        }
        let state = session
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| DataFusionError::Plan("solve requires actual SessionState".into()))?;
        let cancel = services.cancellation().child_token();
        let program = EvaluationProgram::compile(
            &UserDefinedLogicalNodeCore::expressions(node),
            &node
                .specification
                .variables
                .iter()
                .map(|v| v.column.clone())
                .collect::<Vec<_>>(),
            input.schema(),
            state,
            Arc::clone(services.reserver()),
            cancel.clone(),
        )
        .map_err(|e| error(e.into()))?;
        Ok(Arc::new(SolveExec {
            input: Arc::clone(input),
            specification: Arc::clone(&node.specification),
            program: Arc::new(program),
            services,
            cancel,
            permits: Arc::clone(&planner.permits),
            foreign_bytes: planner.foreign_bytes,
            properties: Arc::new(PlanProperties::new(
                EquivalenceProperties::new(super::super::output::schema()),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Final,
                Boundedness::Bounded,
            )),
        }))
    }
    struct SolveExec {
        input: Arc<dyn ExecutionPlan>,
        specification: Arc<Specification>,
        program: Arc<EvaluationProgram>,
        services: Arc<NativeExecutionContext>,
        cancel: CancellationToken,
        permits: Arc<tokio::sync::Semaphore>,
        foreign_bytes: usize,
        properties: Arc<PlanProperties>,
    }
    impl std::fmt::Debug for SolveExec {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.fmt_as(DisplayFormatType::Default, f)
        }
    }
    impl DisplayAs for SolveExec {
        fn fmt_as(
            &self,
            _: DisplayFormatType,
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            write!(
                f,
                "PseSolveExec: variables={}, constraints={}, hessian={:?}",
                self.specification.variables.len(),
                self.specification.constraints.len(),
                self.specification.options.hessian
            )
        }
    }
    impl ExecutionPlan for SolveExec {
        fn name(&self) -> &'static str {
            "PseSolveExec"
        }
        fn properties(&self) -> &Arc<PlanProperties> {
            &self.properties
        }
        fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
            vec![&self.input]
        }
        fn partition_statistics(
            &self,
            partition: Option<usize>,
        ) -> Result<Arc<datafusion::common::Statistics>> {
            if partition.is_some_and(|index| index != 0) {
                return Err(DataFusionError::Plan(
                    "solve has one output partition".into(),
                ));
            }
            Ok(Arc::new(datafusion::common::Statistics::new_unknown(
                &self.schema(),
            )))
        }
        fn apply_expressions(
            &self,
            visitor: &mut dyn FnMut(
                &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
            )
                -> Result<datafusion::common::tree_node::TreeNodeRecursion>,
        ) -> Result<datafusion::common::tree_node::TreeNodeRecursion> {
            for expression in self.program.physical_expressions() {
                if visitor(expression)? == datafusion::common::tree_node::TreeNodeRecursion::Stop {
                    return Ok(datafusion::common::tree_node::TreeNodeRecursion::Stop);
                }
            }
            Ok(datafusion::common::tree_node::TreeNodeRecursion::Continue)
        }
        fn with_new_children(
            self: Arc<Self>,
            children: Vec<Arc<dyn ExecutionPlan>>,
        ) -> Result<Arc<dyn ExecutionPlan>> {
            let [input] = children.as_slice() else {
                return Err(DataFusionError::Plan("solve requires one input".into()));
            };
            if input.schema() != self.input.schema() || input.boundedness().is_unbounded() {
                return Err(DataFusionError::Plan("solve child contract changed".into()));
            }
            Ok(Arc::new(Self {
                input: Arc::clone(input),
                specification: Arc::clone(&self.specification),
                program: Arc::clone(&self.program),
                services: Arc::clone(&self.services),
                cancel: self.cancel.clone(),
                permits: Arc::clone(&self.permits),
                foreign_bytes: self.foreign_bytes,
                properties: Arc::clone(&self.properties),
            }))
        }
        fn execute(
            &self,
            partition: usize,
            context: Arc<TaskContext>,
        ) -> Result<SendableRecordBatchStream> {
            if partition != 0 {
                return Err(DataFusionError::Execution(
                    "solve has one output partition".into(),
                ));
            }
            let input = Arc::clone(&self.input);
            let spec = Arc::clone(&self.specification);
            let program = Arc::clone(&self.program);
            let services = Arc::clone(&self.services);
            let cancel = self.cancel.clone();
            let permits = Arc::clone(&self.permits);
            let foreign_bytes = self.foreign_bytes;
            // Guard exists before first poll, so dropping even an unpolled stream
            // signals its private callback token without cancelling its parent.
            let guard = CancelOnDrop(cancel.clone());
            let stream = futures_util::stream::once(async move {
                let _guard = guard;
                let mut stream = execute_stream(input, context)?;
                let mut row = None;
                while let Some(batch) = stream.try_next().await? {
                    cancel.checkpoint().map_err(|e| error(e.into()))?;
                    if batch.num_rows() == 0 {
                        continue;
                    }
                    if batch.num_rows() != 1 || row.is_some() {
                        return Err(DataFusionError::Execution(
                            "solve input must contain exactly one numerical row".into(),
                        ));
                    }
                    row = Some(batch);
                }
                let row = row.ok_or_else(|| {
                    DataFusionError::Execution("solve numerical input is absent".into())
                })?;
                let permit = cancel
                    .until_cancelled(permits.acquire_owned())
                    .await
                    .map_err(|e| error(e.into()))?
                    .map_err(|e| DataFusionError::Execution(e.to_string()))?;
                cancel.checkpoint().map_err(|e| error(e.into()))?;
                let request = driver::Request {
                    program,
                    input: row,
                    variables: spec.variables.clone(),
                    constraints: spec
                        .constraints
                        .iter()
                        .map(|c| (c.lower, c.upper, c.scale))
                        .collect(),
                    options: spec.options.clone(),
                    cancel,
                    reserver: Arc::clone(services.reserver()),
                    foreign_bytes,
                };
                services.require_settlement();
                // Native task wrapper prevents accidental task-handle detachment.
                // Running C work cannot be aborted: the private token is signalled
                // on stream drop and the task retains the permit/workspace until return.
                let task = datafusion::common::runtime::SpawnedTask::spawn_blocking(move || {
                    let _permit = permit;
                    driver::solve(&request)
                });
                let outcome = task
                    .await
                    .map_err(|e| DataFusionError::Execution(format!("solver worker: {e}")))?
                    .map_err(error)?;
                super::super::output::pack(outcome)
            });
            Ok(Box::pin(RecordBatchStreamAdapter::new(
                self.schema(),
                stream,
            )))
        }
    }
    struct CancelOnDrop(CancellationToken);
    impl Drop for CancelOnDrop {
        fn drop(&mut self) {
            self.0.cancel();
        }
    }
    fn error(error: crate::NativeError) -> DataFusionError {
        DataFusionError::External(Box::new(pse_catalog::CatalogError::Semantic(Arc::new(
            error,
        ))))
    }
}
