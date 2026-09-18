// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Retain local read ownership through native view expansion, optimization and IO.
use super::lease::ReadLease;
use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::{Session, TableProvider},
    common::{Constraints, DataFusionError, Result, Statistics, tree_node::TreeNodeRecursion},
    execution::TaskContext,
    logical_expr::{Expr, TableProviderFilterPushDown, TableType},
    physical_expr::PhysicalExpr,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, PlanProperties, SendableRecordBatchStream,
        execution_plan::CardinalityEffect,
        statistics::{ChildStats, StatisticsArgs},
        stream::RecordBatchStreamAdapter,
    },
};
use futures_util::StreamExt;
use std::{fmt, sync::Arc};

trait ReadOwner: fmt::Debug + Send + Sync {}
pub(crate) fn supports_round_reset(plan: &dyn ExecutionPlan) -> bool {
    plan.is::<LeasedExec>()
}
impl<T: fmt::Debug + Send + Sync> ReadOwner for T {}

pub(super) fn retain_reader_budget(
    inner: Arc<dyn TableProvider>,
    state: &datafusion::execution::session_state::SessionState,
) -> Arc<dyn TableProvider> {
    let predicate_bytes = state
        .config_options()
        .execution
        .parquet
        .max_predicate_cache_size
        .unwrap_or(0);
    if predicate_bytes == 0 {
        return inner;
    }
    Arc::new(LeasedProvider {
        inner,
        lease: Arc::new(()),
        predicate_bytes,
    })
}

pub(super) fn retain_snapshot(
    inner: Arc<dyn TableProvider>,
    owner: Arc<crate::cache_service::snapshot::RetainedTable>,
) -> Arc<dyn TableProvider> {
    Arc::new(LeasedProvider {
        inner,
        lease: owner,
        predicate_bytes: 0,
    })
}

pub(super) fn retain(
    inner: Arc<dyn TableProvider>,
    lease: Option<Arc<ReadLease>>,
) -> Arc<dyn TableProvider> {
    match lease {
        Some(lease) => Arc::new(LeasedProvider {
            inner,
            lease,
            predicate_bytes: 0,
        }),
        None => inner,
    }
}

pub(super) fn retain_execution(
    inner: Arc<dyn ExecutionPlan>,
    lease: Option<Arc<ReadLease>>,
) -> Arc<dyn ExecutionPlan> {
    match lease {
        Some(lease) => Arc::new(LeasedExec {
            inner,
            lease,
            predicate_bytes: 0,
        }),
        None => inner,
    }
}

#[derive(Debug)]
struct LeasedProvider {
    inner: Arc<dyn TableProvider>,
    lease: Arc<dyn ReadOwner>,
    predicate_bytes: usize,
}
#[async_trait::async_trait]
impl TableProvider for LeasedProvider {
    fn schema(&self) -> SchemaRef {
        self.inner.schema()
    }
    fn table_type(&self) -> TableType {
        self.inner.table_type()
    }
    fn constraints(&self) -> Option<&Constraints> {
        self.inner.constraints()
    }
    fn statistics(&self) -> Option<Statistics> {
        self.inner.statistics()
    }
    fn get_column_default(&self, column: &str) -> Option<&Expr> {
        self.inner.get_column_default(column)
    }
    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<TableProviderFilterPushDown>> {
        self.inner.supports_filters_pushdown(filters)
    }
    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let inner = self.inner.scan(state, projection, filters, limit).await?;
        Ok(Arc::new(LeasedExec {
            inner,
            lease: Arc::clone(&self.lease),
            predicate_bytes: self.predicate_bytes,
        }))
    }
    async fn scan_with_args<'a>(
        &self,
        state: &dyn Session,
        args: datafusion::catalog::ScanArgs<'a>,
    ) -> Result<datafusion::catalog::ScanResult> {
        let inner = self.inner.scan_with_args(state, args).await?.into_inner();
        Ok(datafusion::catalog::ScanResult::new(Arc::new(LeasedExec {
            inner,
            lease: Arc::clone(&self.lease),
            predicate_bytes: self.predicate_bytes,
        })))
    }
}

#[derive(Debug)]
struct LeasedExec {
    inner: Arc<dyn ExecutionPlan>,
    lease: Arc<dyn ReadOwner>,
    predicate_bytes: usize,
}
impl DisplayAs for LeasedExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DeltaReadLease")
    }
}
impl ExecutionPlan for LeasedExec {
    fn name(&self) -> &'static str {
        "DeltaReadLease"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        self.inner.properties()
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        vec![&self.inner]
    }
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(&Arc<dyn PhysicalExpr>) -> Result<TreeNodeRecursion>,
    ) -> Result<TreeNodeRecursion> {
        Ok(TreeNodeRecursion::Continue)
    }
    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let [inner]: [Arc<dyn ExecutionPlan>; 1] = children.try_into().map_err(|_| {
            DataFusionError::Plan("Delta reader lease requires one native child".into())
        })?;
        Ok(Arc::new(Self {
            inner,
            lease: Arc::clone(&self.lease),
            predicate_bytes: self.predicate_bytes,
        }))
    }
    fn maintains_input_order(&self) -> Vec<bool> {
        vec![true]
    }
    fn benefits_from_input_partitioning(&self) -> Vec<bool> {
        vec![false]
    }
    fn cardinality_effect(&self) -> CardinalityEffect {
        CardinalityEffect::Equal
    }
    fn supports_limit_pushdown(&self) -> bool {
        true
    }
    fn gather_filters_for_pushdown(
        &self,
        _: datafusion::physical_plan::filter_pushdown::FilterPushdownPhase,
        filters: Vec<Arc<dyn PhysicalExpr>>,
        _: &datafusion::common::config::ConfigOptions,
    ) -> Result<datafusion::physical_plan::filter_pushdown::FilterDescription> {
        datafusion::physical_plan::filter_pushdown::FilterDescription::from_children(
            filters,
            &self.children(),
        )
    }
    fn handle_child_pushdown_result(
        &self,
        _: datafusion::physical_plan::filter_pushdown::FilterPushdownPhase,
        result: datafusion::physical_plan::filter_pushdown::ChildPushdownResult,
        _: &datafusion::common::config::ConfigOptions,
    ) -> Result<
        datafusion::physical_plan::filter_pushdown::FilterPushdownPropagation<
            Arc<dyn ExecutionPlan>,
        >,
    > {
        Ok(datafusion::physical_plan::filter_pushdown::FilterPushdownPropagation::if_all(result))
    }
    fn child_stats_requests(&self, partition: Option<usize>) -> Vec<ChildStats> {
        vec![ChildStats::At(partition)]
    }
    fn statistics_from_inputs(
        &self,
        inputs: &[Arc<Statistics>],
        _: &StatisticsArgs,
    ) -> Result<Arc<Statistics>> {
        inputs
            .first()
            .cloned()
            .ok_or_else(|| DataFusionError::Internal("reader lease child statistics absent".into()))
    }
    fn execute(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let reservation =
            datafusion::execution::memory_pool::MemoryConsumer::new("pse.parquet.predicate_cache")
                .register(&context.runtime_env().memory_pool);
        reservation.try_grow(self.predicate_bytes)?;
        let stream = self.inner.execute(partition, context)?;
        let lease = Arc::clone(&self.lease);
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            self.schema(),
            stream.map(move |batch| {
                let _retained = (&lease, &reservation);
                batch
            }),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::{arrow::datatypes::Schema, physical_plan::empty::EmptyExec};

    #[tokio::test]
    async fn physical_stream_retains_lease_after_plan_and_provider_drop() {
        let directory = tempfile::tempdir().unwrap();
        let location = url::Url::from_directory_path(directory.path()).unwrap();
        let cancel = pse_ids::CancellationToken::new();
        let lease = super::super::lease::read(&location, &cancel)
            .await
            .unwrap()
            .unwrap();
        let plan: Arc<dyn ExecutionPlan> = Arc::new(LeasedExec {
            inner: Arc::new(EmptyExec::new(Arc::new(Schema::empty()))),
            lease,
            predicate_bytes: 0,
        });
        let stream = plan.execute(0, Arc::new(TaskContext::default())).unwrap();
        drop(plan);
        assert!(super::super::lease::maintenance(&location, &cancel).is_err());
        drop(stream);
        assert!(super::super::lease::maintenance(&location, &cancel).is_ok());
    }
}
