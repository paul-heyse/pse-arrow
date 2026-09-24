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
    // Capture is an admission boundary: source constraints/statistics must not
    // let an optimizer remove the row checks that establish those claims.
    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<datafusion::logical_expr::TableProviderFilterPushDown>> {
        self.provider.supports_filters_pushdown(filters)
    }
    fn scan_with_args<'a, 'life0, 'life1, 'async_trait>(
        &'life0 self,
        state: &'life1 dyn Session,
        args: datafusion::catalog::ScanArgs<'a>,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<datafusion::catalog::ScanResult>> + Send + 'async_trait>,
    >
    where
        'a: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        self.provider.scan_with_args(state, args)
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
