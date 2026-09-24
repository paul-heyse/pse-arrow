// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Transparent native readers retain supplied owners through rewrites and IO.
use crate::provider::witness::ExecutionOwner;
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

/// Retain a supplied resource owner and native predicate-cache budget.
pub fn provider(
    inner: Arc<dyn TableProvider>,
    owner: Arc<dyn ExecutionOwner>,
    predicate_bytes: usize,
) -> Arc<dyn TableProvider> {
    Arc::new(RetainedProvider {
        inner,
        lease: owner,
        predicate_bytes,
    })
}
/// Retain a supplied owner through native execution without changing the relation.
pub fn execution(
    inner: Arc<dyn ExecutionPlan>,
    owner: Arc<dyn ExecutionOwner>,
) -> Arc<dyn ExecutionPlan> {
    Arc::new(RetainedExec {
        inner,
        lease: owner,
        predicate_bytes: 0,
    })
}
/// The wrapper is reset-transparent; eligibility still checks the actual child.
pub fn supports_reset(plan: &dyn ExecutionPlan) -> bool {
    plan.is::<RetainedExec>()
}
#[derive(Debug)]
struct RetainedProvider {
    inner: Arc<dyn TableProvider>,
    lease: Arc<dyn ExecutionOwner>,
    predicate_bytes: usize,
}
#[async_trait::async_trait]
impl TableProvider for RetainedProvider {
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
        Ok(Arc::new(RetainedExec {
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
        Ok(datafusion::catalog::ScanResult::new(Arc::new(
            RetainedExec {
                inner,
                lease: Arc::clone(&self.lease),
                predicate_bytes: self.predicate_bytes,
            },
        )))
    }
}

#[derive(Debug)]
struct RetainedExec {
    inner: Arc<dyn ExecutionPlan>,
    lease: Arc<dyn ExecutionOwner>,
    predicate_bytes: usize,
}
impl DisplayAs for RetainedExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DeltaReadLease")
    }
}
impl ExecutionPlan for RetainedExec {
    fn name(&self) -> &'static str {
        "DeltaReadLease"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        self.inner.properties()
    }
    fn with_fetch(&self, limit: Option<usize>) -> Option<Arc<dyn ExecutionPlan>> {
        self.inner
            .with_fetch(limit)
            .map(|inner| -> Arc<dyn ExecutionPlan> {
                Arc::new(Self {
                    inner,
                    lease: self.lease.clone(),
                    predicate_bytes: self.predicate_bytes,
                })
            })
    }
    fn fetch(&self) -> Option<usize> {
        self.inner.fetch()
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
        args: &StatisticsArgs,
    ) -> Result<Arc<Statistics>> {
        let statistics = inputs.first().cloned().ok_or_else(|| {
            DataFusionError::Internal("reader lease child statistics absent".into())
        })?;
        let Some(fetch) = self.inner.fetch() else {
            return Ok(statistics);
        };
        // At this pin native memory scans retain pre-fetch row statistics. Apply
        // the native statistics operation for one partition; global per-partition
        // fetch estimates must not acquire false exactness.
        if args.partition().is_some()
            || self
                .inner
                .properties()
                .output_partitioning()
                .partition_count()
                == 1
        {
            Ok(Arc::new(statistics.as_ref().clone().with_fetch(
                Some(fetch),
                0,
                1,
            )?))
        } else {
            Ok(Arc::new(statistics.as_ref().clone().to_inexact()))
        }
    }

    fn execute(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let reservation =
            datafusion::execution::memory_pool::MemoryConsumer::new("pse.parquet.predicate_cache")
                .register(&context.runtime_env().memory_pool);
        let bytes = self.predicate_bytes.checked_mul(2).ok_or_else(|| {
            DataFusionError::ResourcesExhausted("predicate reader extent overflow".into())
        })?;
        let predicate = if bytes == 0 {
            None
        } else {
            let service = context
                .session_config()
                .get_extension::<crate::cache_service::NativeCacheService>()
                .ok_or_else(|| {
                    DataFusionError::Plan(
                        "predicate cache requires its aggregate admission owner".into(),
                    )
                })?;
            Some(service.admit_predicates(bytes)?)
        };
        reservation.try_grow(bytes)?;
        let stream = self.inner.execute(partition, context)?;
        let lease = Arc::clone(&self.lease);
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            self.schema(),
            stream.map(move |batch| {
                let _retained = (&lease, &reservation, &predicate);
                batch
            }),
        )))
    }
}
