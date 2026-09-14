// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Private unpublished rule inputs. No key constraints or semantic admission claims.

use crate::provider::BoxFut;
use datafusion::arrow::{array::RecordBatch, datatypes::SchemaRef};
use datafusion::catalog::{Session, TableProvider};
use datafusion::common::Result;
use datafusion::datasource::memory::MemorySourceConfig;
use datafusion::logical_expr::{Expr, TableType};
use datafusion::physical_plan::ExecutionPlan;
use pse_schema::model::RelationKey;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct CandidateTable {
    pub key: RelationKey,
    pub batch: RecordBatch,
}
impl TableProvider for CandidateTable {
    fn schema(&self) -> SchemaRef {
        self.batch.schema()
    }
    fn table_type(&self) -> TableType {
        TableType::Base
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
            let batch = limit.map_or_else(
                || self.batch.clone(),
                |limit| self.batch.slice(0, limit.min(self.batch.num_rows())),
            );
            let plan: Arc<dyn ExecutionPlan> = MemorySourceConfig::try_new_exec(
                &[vec![batch]],
                self.schema(),
                projection.cloned(),
            )?;
            Ok(plan)
        })
    }
}
