// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native scans before and after actual capture. Neither wrapper exposes mutation.
use crate::BoxFut;
use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::{Session, TableProvider},
    common::Result,
    logical_expr::{Expr, TableType},
    physical_plan::ExecutionPlan,
};
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct ObservationTable {
    pub(super) provider: Arc<dyn TableProvider>,
    pub(super) schema: SchemaRef,
}
impl TableProvider for ObservationTable {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
    fn table_type(&self) -> TableType {
        self.provider.table_type()
    }
    fn scan<'s, 't, 'p, 'f, 'future>(
        &'s self,
        state: &'t dyn Session,
        projection: Option<&'p Vec<usize>>,
        filters: &'f [Expr],
        limit: Option<usize>,
    ) -> BoxFut<'future, Result<Arc<dyn ExecutionPlan>>>
    where
        's: 'future,
        't: 'future,
        'p: 'future,
        'f: 'future,
        Self: 'future,
    {
        self.provider.scan(state, projection, filters, limit)
    }
}
