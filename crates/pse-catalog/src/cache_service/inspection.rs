// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native read-only table functions. Binding/EXPLAIN never observes the counters.
use super::NativeCacheService;
use datafusion::{
    arrow::{array::RecordBatch, datatypes::SchemaRef},
    catalog::{Session, TableFunction, TableFunctionArgs, TableFunctionImpl, TableProvider},
    common::{DataFusionError, Result},
    datasource::memory::MemorySourceConfig,
    execution::{
        TaskContext,
        session_state::{SessionState, SessionStateBuilder},
    },
    logical_expr::{Expr, TableType},
    physical_expr::EquivalenceProperties,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PlanProperties,
        SendableRecordBatchStream,
        execution_plan::{Boundedness, EmissionType},
    },
};
use pse_relations::generated::runtime::{
    cache_entry_statistics, cache_statistics, execution_statistics,
};
use std::sync::Arc;

pub(crate) fn bind(
    state: SessionState,
    registry: &Arc<pse_schema::Registry>,
) -> Result<SessionState> {
    let Some(service) = state.config().get_extension::<NativeCacheService>() else {
        return Ok(state);
    };
    let mut functions = state.table_functions().clone();
    for (name, kind, relation) in [
        ("pse_cache_statistics", 0, "runtime.cache_statistics"),
        (
            "pse_execution_statistics",
            1,
            "runtime.execution_statistics",
        ),
        ("pse_cache_entries", 2, "runtime.cache_entry_statistics"),
    ] {
        let Some(spec) = registry.relation(relation) else {
            continue;
        };
        let schema =
            Arc::new(pse_schema::arrow::relation_schema(registry, spec).map_err(external)?);
        functions.insert(
            name.into(),
            Arc::new(TableFunction::new(
                name.into(),
                Arc::new(Statistics {
                    service: service.clone(),
                    registry: registry.clone(),
                    schema,
                    kind,
                    limit: 0,
                }),
            )),
        );
    }
    Ok(SessionStateBuilder::new_from_existing(state)
        .with_table_functions(functions)
        .build())
}
#[derive(Debug, Clone)]
struct Statistics {
    service: Arc<NativeCacheService>,
    registry: Arc<pse_schema::Registry>,
    schema: SchemaRef,
    kind: u8,
    limit: usize,
}
impl TableFunctionImpl for Statistics {
    fn call_with_args(&self, args: TableFunctionArgs<'_, '_>) -> Result<Arc<dyn TableProvider>> {
        let mut provider = self.clone();
        if self.kind == 2 {
            let [Expr::Literal(datafusion::common::ScalarValue::Int64(Some(limit)), _)] =
                args.exprs()
            else {
                return Err(DataFusionError::Plan(
                    "cache entry inspection requires one nonnegative integer limit".into(),
                ));
            };
            provider.limit = usize::try_from(*limit).map_err(external)?;
        } else if !args.exprs().is_empty() {
            return Err(DataFusionError::Plan(
                "cache statistics take no arguments".into(),
            ));
        }
        Ok(Arc::new(provider))
    }
}
#[async_trait::async_trait]
impl TableProvider for Statistics {
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
    fn table_type(&self) -> TableType {
        TableType::Temporary
    }
    async fn scan(
        &self,
        _: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        _: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if !filters.is_empty() {
            return Err(DataFusionError::Plan(
                "statistics filters remain native residuals".into(),
            ));
        }
        let schema = projection.map_or_else(
            || Ok(self.schema.clone()),
            |indices| self.schema.project(indices).map(Arc::new),
        )?;
        Ok(Arc::new(StatisticsExec {
            source: self.clone(),
            projection: projection.cloned(),
            properties: Arc::new(PlanProperties::new(
                EquivalenceProperties::new(schema),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Incremental,
                Boundedness::Bounded,
            )),
        }))
    }
}
impl Statistics {
    fn batch(&self) -> Result<RecordBatch> {
        if self.kind == 2 {
            let entries = self.service.inspect_entries(self.limit)?;
            let mut builder = cache_entry_statistics::Builder::with_registry(
                &self.registry,
                entries.rows().len(),
            )
            .map_err(external)?;
            for entry in entries.rows() {
                builder
                    .push(cache_entry_statistics::Row {
                        cache: entry.cache.clone(),
                        key: entry.key.clone(),
                        bytes: signed(entry.bytes)?,
                        hits: signed(entry.hits)?,
                    })
                    .map_err(external)?;
            }
            Ok(builder.finish().map_err(external)?.into_batch())
        } else if self.kind == 1 {
            let mut builder = execution_statistics::Builder::with_registry(&self.registry, 8)
                .map_err(external)?;
            for (name, count) in self.service.execution_report() {
                builder
                    .push(execution_statistics::Row {
                        name: name.into(),
                        count: count.map(signed).transpose()?,
                    })
                    .map_err(external)?;
            }
            Ok(builder.finish().map_err(external)?.into_batch())
        } else {
            let reports = self.service.report();
            let mut builder =
                cache_statistics::Builder::with_registry(&self.registry, reports.len())
                    .map_err(external)?;
            for report in reports {
                builder
                    .push(cache_statistics::Row {
                        name: report.name,
                        policy_limit_bytes: signed(report.policy_limit_bytes)?,
                        capacity_bytes: signed(report.capacity_bytes)?,
                        retained_bytes: signed(report.retained_bytes)?,
                        live_bytes: report.live_bytes.map(signed).transpose()?,
                        pinned_bytes: report.pinned_bytes.map(signed).transpose()?,
                        entries: signed(report.entries)?,
                        inflight_bytes: report.inflight_bytes.map(signed).transpose()?,
                        active_loads: report.active_loads.map(signed).transpose()?,
                        hits: signed(report.hits)?,
                        misses: signed(report.misses)?,
                        bypasses: signed(report.bypasses)?,
                        evictions: report.evictions.map(signed).transpose()?,
                    })
                    .map_err(external)?;
            }
            Ok(builder.finish().map_err(external)?.into_batch())
        }
    }
}
#[derive(Debug)]
struct StatisticsExec {
    source: Statistics,
    projection: Option<Vec<usize>>,
    properties: Arc<PlanProperties>,
}
impl DisplayAs for StatisticsExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NativeCacheStatisticsExec: observation only")
    }
}
impl ExecutionPlan for StatisticsExec {
    fn name(&self) -> &'static str {
        "NativeCacheStatisticsExec"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        vec![]
    }
    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if !children.is_empty() {
            return Err(DataFusionError::Plan("statistics have no children".into()));
        }
        Ok(self)
    }
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(
            &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
        ) -> Result<datafusion::common::tree_node::TreeNodeRecursion>,
    ) -> Result<datafusion::common::tree_node::TreeNodeRecursion> {
        Ok(datafusion::common::tree_node::TreeNodeRecursion::Continue)
    }
    fn execute(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        if partition != 0 {
            return Err(DataFusionError::Execution(
                "statistics have one partition".into(),
            ));
        }
        let batch = self.source.batch()?;
        MemorySourceConfig::try_new_exec(
            &[vec![batch]],
            self.source.schema.clone(),
            self.projection.clone(),
        )?
        .execute(0, context)
    }
}
fn signed(value: usize) -> Result<i64> {
    i64::try_from(value).map_err(external)
}
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
