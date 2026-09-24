// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Observable metadata is a native source with bounded, lazy execution.
use super::{EngineSession, Resolve};
use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::{Session, TableProvider},
    common::{DataFusionError, Result},
    execution::{TaskContext, context::SessionContext, session_state::SessionState},
    logical_expr::{Expr, TableType},
    physical_plan::{ExecutionPlan, SendableRecordBatchStream},
};
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct ResolutionProvider {
    pub(super) operation: Arc<Resolve>,
    pub(super) session: EngineSession,
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
        let mut captured = state.clone();
        captured.register_catalog_list(Arc::new(
            session
                .bindings
                .catalogs(&defaults.default_catalog, &defaults.default_schema)?,
        ));
        Arc::make_mut(&mut session.native).context = SessionContext::new_with_state(captured);
        let physical = crate::operation::Execution::plan(
            "CatalogResolution",
            self.schema(),
            vec![],
            Arc::new(ResolutionBody {
                operation: self.operation.clone(),
                session,
                cancel: services.cancellation().clone(),
            }),
            crate::operation::Family::Observation,
            self.operation.started.clone(),
            services,
        )?;
        super::super::physical_input::PhysicalInput::native(physical)
            .scan(state, projection, filters, limit)
            .await
    }
}

#[derive(Debug)]
struct ResolutionBody {
    operation: Arc<Resolve>,
    session: EngineSession,
    cancel: pse_columnar::CancellationToken,
}
impl crate::operation::Body for ResolutionBody {
    fn execute(
        &self,
        _: Vec<Arc<dyn ExecutionPlan>>,
        _: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let operation = self.operation.clone();
        let session = self.session.clone();
        let cancel = self.cancel.clone();
        Ok(crate::operation::batch(Resolve::schema(), async move {
            cancel.checkpoint().map_err(|e| external(e.into()))?;
            operation.execute(&session, &cancel).await.map_err(external)
        }))
    }
}
fn external(error: crate::EngineError) -> DataFusionError {
    pse_columnar::external(error)
}
