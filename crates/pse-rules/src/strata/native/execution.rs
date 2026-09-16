// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual physical children supply all source and provenance inputs to the iteration.
use super::{FixedPoint, Program};
use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion::{
    catalog::Session,
    common::{DataFusionError, Result},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{
        LogicalPlan, UserDefinedLogicalNode, physical_planning_context::PhysicalPlanningContext,
    },
    physical_expr::EquivalenceProperties,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, ExecutionPlanProperties, Partitioning,
        PlanProperties, SendableRecordBatchStream, execute_stream,
        execution_plan::{Boundedness, EmissionType},
        stream::RecordBatchStreamAdapter,
    },
    physical_planner::{ExtensionPlanner, PhysicalPlanner},
};
use futures_util::TryStreamExt;
use pse_catalog::{
    CatalogError,
    session::{SnapshotSession, execution::NativeExecutionContext},
};
use pse_ids::owned_buffer::OwnedRecordBatch;
use pse_relations::columnar::FieldCheckedBatch;
use std::{collections::BTreeMap, sync::Arc};

/// Composes finite rule execution with Delta and other native extension planners.
#[derive(Debug)]
pub struct RuleExtensionPlanner;
#[async_trait::async_trait]
impl ExtensionPlanner for RuleExtensionPlanner {
    async fn plan_extension(
        &self,
        _: &dyn PhysicalPlanner,
        node: &dyn UserDefinedLogicalNode,
        _: &[&LogicalPlan],
        inputs: &[Arc<dyn ExecutionPlan>],
        session: &dyn Session,
        _: &PhysicalPlanningContext,
    ) -> Result<Option<Arc<dyn ExecutionPlan>>> {
        let Some(node) = node.as_any().downcast_ref::<FixedPoint>() else {
            return Ok(None);
        };
        let services = NativeExecutionContext::from_session(session)?;
        let state = session
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| {
                DataFusionError::Plan("fixed point requires actual SessionState".into())
            })?;
        validate_children(&node.program, inputs)?;
        Ok(Some(Arc::new(FixedPointExec {
            program: Arc::clone(&node.program),
            inputs: inputs.to_vec(),
            services,
            state: Arc::new(state.clone()),
            properties: Arc::new(PlanProperties::new(
                EquivalenceProperties::new(Arc::clone(&node.program.layout.schema)),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Final,
                Boundedness::Bounded,
            )),
        })))
    }
}
fn validate_children(program: &Program, inputs: &[Arc<dyn ExecutionPlan>]) -> Result<()> {
    if inputs.len() != program.children.len()
        || inputs
            .iter()
            .any(|input| input.boundedness().is_unbounded())
    {
        return Err(DataFusionError::Plan(
            "fixed point requires its complete bounded physical inputs".into(),
        ));
    }
    Ok(())
}
struct FixedPointExec {
    program: Arc<Program>,
    inputs: Vec<Arc<dyn ExecutionPlan>>,
    state: Arc<SessionState>,
    services: Arc<NativeExecutionContext>,
    properties: Arc<PlanProperties>,
}
impl std::fmt::Debug for FixedPointExec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_as(DisplayFormatType::Default, f)
    }
}
impl DisplayAs for FixedPointExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PseFixedPointExec: rules={}, children={}, max_rounds={}",
            self.program.rules.len(),
            self.inputs.len(),
            self.program.limits.max_rounds
        )
    }
}
impl ExecutionPlan for FixedPointExec {
    fn name(&self) -> &'static str {
        "PseFixedPointExec"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }
    fn partition_statistics(
        &self,
        partition: Option<usize>,
    ) -> Result<Arc<datafusion::common::Statistics>> {
        if partition.is_some_and(|index| index != 0) {
            return Err(DataFusionError::Plan(
                "fixed point has one output partition".into(),
            ));
        }
        // One tuple exists only after convergence and full conflict admission.
        // An exact count could let a native aggregate eliminate that necessary work.
        Ok(Arc::new(datafusion::common::Statistics::new_unknown(
            &self.schema(),
        )))
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        self.inputs.iter().collect()
    }
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(
            &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
        ) -> Result<datafusion::common::tree_node::TreeNodeRecursion>,
    ) -> Result<datafusion::common::tree_node::TreeNodeRecursion> {
        Ok(datafusion::common::tree_node::TreeNodeRecursion::Continue)
    }
    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        validate_children(&self.program, &children)?;
        Ok(Arc::new(Self {
            inputs: children,
            program: Arc::clone(&self.program),
            state: Arc::clone(&self.state),
            services: Arc::clone(&self.services),
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
                "fixed point has one output partition".into(),
            ));
        }
        let inputs = self.inputs.clone();
        let program = Arc::clone(&self.program);
        let services = Arc::clone(&self.services);
        let state = Arc::clone(&self.state);
        let stream = futures_util::stream::once(async move {
            let session = bind_inputs(&program, &inputs, &state, &services, context)
                .await
                .map_err(native_error)?;
            let settled = super::super::iterate(
                &program.rules,
                &program.bindings,
                &session,
                services.registry(),
                &program.outputs,
                program.limits,
                services.cancellation(),
            )
            .await
            .map_err(native_error)?;
            program.layout.pack(&settled).map_err(native_error)
        });
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            self.schema(),
            stream,
        )))
    }
}

async fn bind_inputs(
    program: &Program,
    inputs: &[Arc<dyn ExecutionPlan>],
    state: &SessionState,
    services: &Arc<NativeExecutionContext>,
    context: Arc<TaskContext>,
) -> Result<SnapshotSession, RuleError> {
    let cancel = services.cancellation();
    let mut roles = BTreeMap::new();
    for ((role, key), child) in program.children.iter().zip(inputs) {
        cancel.checkpoint().map_err(CatalogError::from)?;
        let spec = super::super::native_state::spec(services.registry(), *key)?;
        let schema = Arc::new(
            pse_schema::arrow::relation_schema(services.registry(), spec)
                .map_err(|e| internal(e.to_string()))?,
        );
        if child.schema().fields() != schema.fields() {
            return Err(internal("fixed-point child fields changed"));
        }
        let mut stream = execute_stream(Arc::clone(child), Arc::clone(&context)).map_err(engine)?;
        let mut batches = Vec::new();
        let mut scratch = services
            .reserver()
            .open("rules:fixed-point-child-admission");
        while let Some(batch) = stream.try_next().await.map_err(engine)? {
            cancel.checkpoint().map_err(CatalogError::from)?;
            scratch
                .try_grow(pse_ids::validation_extent(&batch).map_err(CatalogError::from)?)
                .map_err(CatalogError::from)?;
            let batch = batch
                .with_schema(Arc::clone(&schema))
                .map_err(|e| internal(e.to_string()))?;
            let owned = OwnedRecordBatch::export(batch, services.reserver().as_ref(), cancel)
                .map_err(CatalogError::from)?;
            batches.push(FieldCheckedBatch::admit_owned(
                services.registry(),
                spec,
                owned,
            )?);
        }
        let rows = FieldCheckedBatch::concat_reserved(
            services.registry(),
            spec,
            &batches,
            services.reserver().as_ref(),
            cancel,
        )?;
        roles.insert(role.clone(), rows);
    }
    // The mapping children must retain the admitted output-to-source relation.
    // Consumers read the executed child below; a rewritten child cannot silently
    // replace its actual provenance owner with invented witnesses.
    for (rule, binding) in &program.bindings {
        for (port, input) in &binding.ports {
            if let super::super::RuleInputLocation::Native(input) = &input.location {
                let role = super::super::native_support::mapping_role(*rule, port);
                if roles
                    .get(&role)
                    .is_none_or(|rows| rows.batch() != input.support_mapping().batch())
                {
                    return Err(internal(
                        "executed support mapping differs from its admitted owner",
                    ));
                }
            }
        }
    }
    Ok(services.candidate_roles(state, roles)?)
}
fn native_error(error: RuleError) -> DataFusionError {
    DataFusionError::External(Box::new(CatalogError::Semantic(Arc::new(error))))
}
