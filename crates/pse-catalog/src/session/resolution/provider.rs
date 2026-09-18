// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Observable metadata is a native source with bounded, lazy execution.
use super::{Resolve, SnapshotSession};
use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::{Session, TableProvider},
    common::{DataFusionError, Result, tree_node::TreeNodeRecursion},
    execution::{
        TaskContext,
        context::SessionContext,
        session_state::{SessionState, SessionStateBuilder},
    },
    logical_expr::{Expr, TableType},
    physical_expr::EquivalenceProperties,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PlanProperties,
        SendableRecordBatchStream,
        execution_plan::{Boundedness, EmissionType},
        stream::RecordBatchStreamAdapter,
    },
};
use std::sync::{Arc, atomic::Ordering};

#[derive(Debug)]
pub(super) struct ResolutionProvider {
    pub(super) operation: Arc<Resolve>,
    pub(super) session: SnapshotSession,
}
#[async_trait::async_trait]
impl TableProvider for ResolutionProvider {
    fn schema(&self) -> SchemaRef {
        Resolve::schema()
    }
    fn table_type(&self) -> TableType {
        TableType::Temporary
    }
    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let services = super::super::execution::NativeExecutionContext::from_session(state)?;
        services
            .admit_effects(&Resolve::effects())
            .map_err(external)?;
        let state = state
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| {
                DataFusionError::Plan("metadata resolution requires caller state".into())
            })?;
        let mut session = self.session.clone();
        let defaults = &state.config_options().catalog;
        // Retain actual caller configuration and planners, with the original source
        // hierarchy. The result must never retain its own resolving provider.
        let captured = SessionStateBuilder::new_from_existing(state.clone())
            .with_catalog_list(Arc::new(
                session
                    .bindings
                    .catalogs(&defaults.default_catalog, &defaults.default_schema)?,
            ))
            .build();
        session.context = SessionContext::new_with_state(captured);
        let physical: Arc<dyn ExecutionPlan> = Arc::new(ResolutionExec {
            operation: Arc::clone(&self.operation),
            session,
            cancel: services.cancellation().clone(),
            properties: Arc::new(PlanProperties::new(
                EquivalenceProperties::new(self.schema()),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Final,
                Boundedness::Bounded,
            )),
        });
        super::super::physical_input::PhysicalInput::native(physical)
            .scan(state, projection, filters, limit)
            .await
    }
}

#[derive(Debug)]
struct ResolutionExec {
    operation: Arc<Resolve>,
    session: SnapshotSession,
    cancel: pse_ids::CancellationToken,
    properties: Arc<PlanProperties>,
}
impl DisplayAs for ResolutionExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CatalogResolutionExec: {:?} {:?}",
            self.operation.consistency, self.operation.request
        )
    }
}
impl ExecutionPlan for ResolutionExec {
    fn name(&self) -> &'static str {
        "CatalogResolutionExec"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        vec![]
    }
    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if !children.is_empty() {
            return Err(DataFusionError::Plan(
                "metadata source has no data children".into(),
            ));
        }
        Ok(self)
    }
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(
            &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
        ) -> Result<TreeNodeRecursion>,
    ) -> Result<TreeNodeRecursion> {
        Ok(TreeNodeRecursion::Continue)
    }
    fn execute(&self, partition: usize, _: Arc<TaskContext>) -> Result<SendableRecordBatchStream> {
        if partition != 0 || self.operation.started.swap(true, Ordering::AcqRel) {
            return Err(DataFusionError::Execution(
                "metadata resolution executes once in partition zero".into(),
            ));
        }
        let operation = Arc::clone(&self.operation);
        let session = self.session.clone();
        let cancel = self.cancel.clone();
        let stream = futures_util::stream::once(async move {
            cancel.checkpoint().map_err(|e| external(e.into()))?;
            operation.execute(&session, &cancel).await.map_err(external)
        });
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            self.schema(),
            stream,
        )))
    }
}
fn external(error: crate::CatalogError) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
