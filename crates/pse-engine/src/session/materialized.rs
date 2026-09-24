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
use pse_columnar::owned_buffer::OwnedRecordBatch;
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct ImmutableTable<T> {
    pub(super) defaults: std::collections::BTreeMap<String, Expr>,
    pub(super) schema: SchemaRef,
    pub(super) data: T,
    pub(super) constraints: Option<Constraints>,
}
impl<T: Batches> TableProvider for ImmutableTable<T> {
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
        self.constraints.as_ref()
    }
    fn scan<'s, 't, 'p, 'f, 'future>(
        &'s self,
        _state: &'t dyn Session,
        projection: Option<&'p Vec<usize>>,
        _filters: &'f [Expr],
        limit: Option<usize>,
    ) -> BoxFut<'future, Result<Arc<dyn ExecutionPlan>>>
    where
        's: 'future,
        't: 'future,
        'p: 'future,
        'f: 'future,
        Self: 'future,
    {
        Box::pin(async move {
            let plan = datafusion::datasource::memory::MemorySourceConfig::try_new_exec(
                &[self.data.native(&self.schema)?],
                self.schema(),
                projection.cloned(),
            )?;
            let plan = match limit {
                Some(limit) => plan.with_fetch(Some(limit)).unwrap_or(plan),
                None => plan,
            };
            Ok(plan)
        })
    }
}

/// Private immutable batch ownership; constraints are admitted separately.
pub(super) trait Batches: std::fmt::Debug + Send + Sync + 'static {
    fn native(&self, schema: &SchemaRef) -> Result<Vec<datafusion::arrow::array::RecordBatch>>;
}
impl Batches for Vec<OwnedRecordBatch> {
    fn native(&self, _: &SchemaRef) -> Result<Vec<datafusion::arrow::array::RecordBatch>> {
        Ok(self.iter().map(|batch| batch.batch().clone()).collect())
    }
}
impl Batches for pse_relations::columnar::FieldCheckedBatch {
    fn native(&self, schema: &SchemaRef) -> Result<Vec<datafusion::arrow::array::RecordBatch>> {
        let batch = super::query_schema::batch(self.batch())?;
        if batch.schema().as_ref() != schema.as_ref() {
            return Err(datafusion::common::DataFusionError::Internal(
                "candidate query schema changed".into(),
            ));
        }
        Ok(vec![batch])
    }
}
impl ImmutableTable<pse_relations::columnar::FieldCheckedBatch> {
    pub(super) fn candidate(input: pse_relations::columnar::FieldCheckedBatch) -> Self {
        Self {
            schema: super::query_schema::schema(input.batch().schema().as_ref()),
            data: input,
            defaults: std::collections::BTreeMap::new(),
            constraints: None,
        }
    }
}
