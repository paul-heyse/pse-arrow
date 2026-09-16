// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The native planner, including EXPLAIN, receives the actual deferred operation plan.

use crate::BoxFut;
use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::{Session, TableProvider},
    common::{DFSchemaRef, Result},
    logical_expr::{Expr, MergeIntoClause, TableType},
    physical_plan::ExecutionPlan,
};
use datafusion_expr::dml::InsertOp;
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct DeferredTarget {
    pub(super) source: Arc<dyn TableProvider>,
    pub(super) physical: Arc<dyn ExecutionPlan>,
}
impl TableProvider for DeferredTarget {
    fn schema(&self) -> SchemaRef {
        self.source.schema()
    }
    fn table_type(&self) -> TableType {
        self.source.table_type()
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
        self.source.scan(state, projection, filters, limit)
    }
    fn insert_into<'s, 't, 'future>(
        &'s self,
        _state: &'t dyn Session,
        _input: Arc<dyn ExecutionPlan>,
        _op: InsertOp,
    ) -> BoxFut<'future, Result<Arc<dyn ExecutionPlan>>>
    where
        's: 'future,
        't: 'future,
        Self: 'future,
    {
        Box::pin(async move { Ok(Arc::clone(&self.physical)) })
    }
    fn delete_from<'s, 't, 'future>(
        &'s self,
        _state: &'t dyn Session,
        _filters: Vec<Expr>,
    ) -> BoxFut<'future, Result<Arc<dyn ExecutionPlan>>>
    where
        's: 'future,
        't: 'future,
        Self: 'future,
    {
        Box::pin(async move { Ok(Arc::clone(&self.physical)) })
    }
    fn update<'s, 't, 'future>(
        &'s self,
        _state: &'t dyn Session,
        _assignments: Vec<(String, Expr)>,
        _filters: Vec<Expr>,
    ) -> BoxFut<'future, Result<Arc<dyn ExecutionPlan>>>
    where
        's: 'future,
        't: 'future,
        Self: 'future,
    {
        Box::pin(async move { Ok(Arc::clone(&self.physical)) })
    }
    fn truncate<'s, 't, 'future>(
        &'s self,
        _state: &'t dyn Session,
    ) -> BoxFut<'future, Result<Arc<dyn ExecutionPlan>>>
    where
        's: 'future,
        't: 'future,
        Self: 'future,
    {
        Box::pin(async move { Ok(Arc::clone(&self.physical)) })
    }
    fn merge_into<'s, 't, 'future>(
        &'s self,
        _state: &'t dyn Session,
        _source: Arc<dyn ExecutionPlan>,
        _schema: DFSchemaRef,
        _on: Expr,
        _clauses: Vec<MergeIntoClause>,
    ) -> BoxFut<'future, Result<Arc<dyn ExecutionPlan>>>
    where
        's: 'future,
        't: 'future,
        Self: 'future,
    {
        Box::pin(async move { Ok(Arc::clone(&self.physical)) })
    }
}
