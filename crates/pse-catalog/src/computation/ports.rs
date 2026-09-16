// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared private output providers. A scan observes completion and never drives it.

use super::ProductionOperation;
use crate::{BoxFut, CatalogError, session::SnapshotSession};
use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::{Session, TableProvider},
    common::{DataFusionError, TableReference},
    datasource::MemTable,
    logical_expr::{Expr, TableType},
    physical_plan::ExecutionPlan,
};
use std::sync::Arc;

pub(super) fn bind(
    mut session: SnapshotSession,
    operation: &Arc<ProductionOperation>,
) -> Result<SnapshotSession, CatalogError> {
    for port in &operation.producer.spec().outputs {
        let spec = operation
            .catalog
            .registry
            .relation(&port.relation)
            .ok_or_else(|| super::membership::refused("output declaration missing"))?;
        let schema = Arc::new(
            pse_schema::arrow::relation_schema(&operation.catalog.registry, spec)
                .map_err(pse_relations::RelationError::from)?,
        );
        let provider: Arc<dyn TableProvider> = Arc::new(OutputPort {
            operation: Arc::clone(operation),
            port: port.port.to_owned(),
            schema,
        });
        session = session.with_output_provider(
            port.port.to_owned(),
            TableReference::full(
                "outputs",
                operation.producer.spec().qualified_name(),
                port.port,
            ),
            provider,
        )?;
    }
    Ok(session)
}
#[derive(Debug)]
struct OutputPort {
    operation: Arc<ProductionOperation>,
    port: String,
    schema: SchemaRef,
}
impl TableProvider for OutputPort {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
    fn table_type(&self) -> TableType {
        TableType::Temporary
    }
    fn scan<'s, 't, 'p, 'f, 'future>(
        &'s self,
        state: &'t dyn Session,
        projection: Option<&'p Vec<usize>>,
        filters: &'f [Expr],
        limit: Option<usize>,
    ) -> BoxFut<'future, datafusion::common::Result<Arc<dyn ExecutionPlan>>>
    where
        's: 'future,
        't: 'future,
        'p: 'future,
        'f: 'future,
        Self: 'future,
    {
        Box::pin(async move {
            let output = self
                .operation
                .output
                .lock()
                .map_err(|_| DataFusionError::Internal("output catalog lock poisoned".to_owned()))?
                .as_ref()
                .cloned()
                .ok_or_else(|| {
                    DataFusionError::Execution("operation output is not completed".to_owned())
                })?;
            let value = output.outputs.get(&self.port).ok_or_else(|| {
                DataFusionError::Internal("completed operation lacks a declared port".to_owned())
            })?;
            let table =
                MemTable::try_new(Arc::clone(&self.schema), vec![vec![value.batch().clone()]])?;
            table.scan(state, projection, filters, limit).await
        })
    }
}
