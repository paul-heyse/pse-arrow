// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Read-only native memory scans over completed Arrow buffers. No producer graph is
//! retained. Unlike `MemTable`, this provider exposes no mutable partition storage or
//! DML hooks, so admitted facts cannot change through another provider handle.

use crate::BoxFut;
use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::{Session, TableProvider},
    common::{Constraints, Result},
    logical_expr::{Expr, TableType},
    physical_plan::ExecutionPlan,
};
use pse_ids::owned_buffer::OwnedRecordBatch;
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct MaterializedTable {
    pub(super) defaults: std::collections::BTreeMap<String, Expr>,
    pub(super) schema: SchemaRef,
    pub(super) batches: Vec<OwnedRecordBatch>,
    pub(super) constraints: Constraints,
}
impl TableProvider for MaterializedTable {
    fn get_column_default(&self, name: &str) -> Option<&Expr> {
        self.defaults.get(name)
    }
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
    fn table_type(&self) -> TableType {
        TableType::Base
    }
    fn constraints(&self) -> Option<&Constraints> {
        Some(&self.constraints)
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
            let plan: Arc<dyn ExecutionPlan> =
                datafusion::datasource::memory::MemorySourceConfig::try_new_exec(
                    &[self
                        .batches
                        .iter()
                        .map(|batch| batch.batch().clone())
                        .collect()],
                    self.schema(),
                    projection.cloned(),
                )?;
            Ok(plan)
        })
    }
}
