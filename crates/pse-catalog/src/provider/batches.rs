// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable operation inputs. Keys remain unavailable until admission establishes them.

use crate::BoxFut;
use datafusion::{
    arrow::{array::RecordBatch, datatypes::SchemaRef},
    catalog::{Session, TableProvider},
    common::{DataFusionError, Result},
    datasource::memory::MemorySourceConfig,
    logical_expr::{Expr, TableType},
    physical_plan::ExecutionPlan,
};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct BatchInput {
    schema: SchemaRef,
    batches: Vec<RecordBatch>,
}
impl BatchInput {
    pub(crate) fn try_new(schema: SchemaRef, batches: Vec<RecordBatch>) -> Result<Self> {
        if batches.iter().any(|batch| batch.schema() != schema) {
            return Err(DataFusionError::Plan(
                "operation input differs from its exact declared schema".into(),
            ));
        }
        Ok(Self { schema, batches })
    }
}
impl TableProvider for BatchInput {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
    fn table_type(&self) -> TableType {
        TableType::Base
    }
    fn scan<'s, 't, 'p, 'f, 'future>(
        &'s self,
        _state: &'t dyn Session,
        projection: Option<&'p Vec<usize>>,
        _filters: &'f [Expr],
        _limit: Option<usize>,
    ) -> BoxFut<'future, Result<Arc<dyn ExecutionPlan>>>
    where
        's: 'future,
        't: 'future,
        'p: 'future,
        'f: 'future,
        Self: 'future,
    {
        Box::pin(async move {
            let plan: Arc<dyn ExecutionPlan> = MemorySourceConfig::try_new_exec(
                std::slice::from_ref(&self.batches),
                self.schema(),
                projection.cloned(),
            )?;
            Ok(plan)
        })
    }
}
