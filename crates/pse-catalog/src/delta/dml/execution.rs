// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Effect timing, real physical children and native Delta operation execution.
use super::super::settlement::{committed, unresolved};
use datafusion::{
    arrow::{
        array::{Array, RecordBatch, UInt64Array},
        datatypes::{DataType, Field, Schema},
    },
    common::{DataFusionError, Result},
    datasource::provider_as_source,
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, cast, dml::MergeIntoClause, lit},
    physical_plan::{ExecutionPlan, SendableRecordBatchStream},
};
use deltalake::{
    DeltaTable,
    kernel::{Action, transaction::CommitProperties},
    operations::write::WriteMetrics,
    protocol::SaveMode,
};
use pse_engine::operation::{Body, Execution, Family};
use pse_engine::session::physical_input::PhysicalInput;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(super) enum Command {
    Insert(SaveMode),
    Delete(Vec<Expr>),
    Update {
        assignments: Vec<(String, Expr)>,
        filters: Vec<Expr>,
    },
    Merge {
        on: Expr,
        clauses: Vec<MergeIntoClause>,
    },
}
#[derive(Debug)]
struct MutationBody {
    table: DeltaTable,
    context: Arc<super::super::operation::DeltaOperationContext>,
    command: Command,
}
pub(super) fn plan(
    table: DeltaTable,
    state: Arc<SessionState>,
    commit: CommitProperties,
    contract: Option<super::super::contract::DeclaredCheck>,
    command: Command,
    children: Vec<Arc<dyn ExecutionPlan>>,
) -> Result<Arc<dyn ExecutionPlan>> {
    let context = Arc::new(super::super::operation::DeltaOperationContext::new(
        state,
        contract,
        commit,
        super::super::operation::CommitKind::Data,
    )?);
    let services = context.services.clone();
    Execution::plan(
        "DeltaMutation",
        count_schema(),
        children,
        Arc::new(MutationBody {
            table,
            context,
            command,
        }),
        Family::Command,
        Arc::default(),
        services,
    )
}
fn count_schema() -> datafusion::arrow::datatypes::SchemaRef {
    Arc::new(Schema::new(vec![Field::new(
        "count",
        DataType::UInt64,
        false,
    )]))
}
impl Body for MutationBody {
    fn execute(
        &self,
        children: Vec<Arc<dyn ExecutionPlan>>,
        _: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let table = self.table.clone();
        let context = self.context.clone();
        let command = self.command.clone();
        Ok(pse_engine::operation::batch(count_schema(), async move {
            let count = run(table, context, command, children).await?;
            Ok(RecordBatch::try_new(
                count_schema(),
                vec![Arc::new(UInt64Array::from(vec![count]))],
            )?)
        }))
    }
}
async fn run(
    table: DeltaTable,
    context: Arc<super::super::operation::DeltaOperationContext>,
    command: Command,
    children: Vec<Arc<dyn ExecutionPlan>>,
) -> Result<u64> {
    let state = context.state.clone();
    let _writer = context.begin(&table).await?;
    match command {
        Command::Insert(mode) => insert(table, &context, mode, &children).await,
        Command::Update {
            assignments,
            filters,
        } => {
            let mut builder = context.update(table);
            if let Some(predicate) = filters.into_iter().reduce(Expr::and) {
                builder = builder.with_predicate(predicate);
            }
            for (name, value) in assignments {
                builder = builder.with_update(datafusion::common::Column::from_name(name), value);
            }
            let (after, metrics) = builder.await.map_err(unresolved)?;
            super::super::provider::committed(&after, &state).await;
            super::super::settlement::observed(after.version(), count(metrics.num_updated_rows))
        }
        Command::Delete(filters) => {
            let before = table.clone();
            let mut builder = context.delete(table);
            if let Some(predicate) = filters.into_iter().reduce(Expr::and) {
                builder = builder.with_predicate(predicate);
            }
            let (after, metrics) = builder.await.map_err(unresolved)?;
            super::super::provider::committed(&after, &state).await;
            if after.version() == before.version() {
                return Ok(0);
            }
            if let Some(rows) = metrics.num_deleted_rows {
                return super::super::settlement::observed(after.version(), count(rows));
            }
            let version = after
                .version()
                .ok_or_else(|| invalid("Delta delete omitted its version"))?;
            // File-level deletion can lack row statistics. Count the exact before and
            // after snapshots using native aggregation; never re-evaluate its predicate.
            // Commands do not rebase: an intervening commit fails OCC instead.
            let observed = async {
                let before = row_count(before, Arc::clone(&state)).await?;
                let after = row_count(after, state).await?;
                affected_difference(before, after, true)
            }
            .await;
            observed.map_err(|error| committed(version, error))
        }
        Command::Merge { on, clauses } => {
            super::merge::execute(table, &context, input(&children)?, on, clauses).await
        }
    }
}
async fn insert(
    table: DeltaTable,
    context: &super::super::operation::DeltaOperationContext,
    mode: SaveMode,
    children: &[Arc<dyn ExecutionPlan>],
) -> Result<u64> {
    let state = context.state.clone();
    let exact_input = children
        .first()
        .and_then(|child| {
            datafusion::physical_plan::statistics::StatisticsContext::new()
                .compute(
                    child.as_ref(),
                    &datafusion::physical_plan::statistics::StatisticsArgs::new(),
                )
                .ok()
        })
        .and_then(|stats| match stats.num_rows {
            datafusion::common::stats::Precision::Exact(rows) => u64::try_from(rows).ok(),
            _ => None,
        });
    let input = input(children)?;
    let previous = table.version();
    let before = table.clone();
    let table = context
        .write(table, input, mode)
        .await
        .map_err(unresolved)?;
    if table.version() == previous {
        return Ok(0);
    }
    let version = table
        .version()
        .ok_or_else(|| invalid("Delta write omitted its version"))?;
    super::super::provider::committed(&table, &state).await;
    if let Some(rows) = exact_input {
        return Ok(rows);
    }
    let metrics = write_count(&table, version, &state).await;
    if let Ok(Some(rows)) = metrics {
        return Ok(rows);
    }
    let fallback = async {
        let after = row_count(table, state.clone()).await?;
        let before = if matches!(mode, SaveMode::Overwrite | SaveMode::ErrorIfExists)
            || previous.is_none()
        {
            0
        } else {
            row_count(before, state.clone()).await?
        };
        affected_difference(before, after, false)
    }
    .await;
    fallback.map_err(|error| {
        committed(
            version,
            match metrics {
                Err(primary) => DataFusionError::Collection(vec![primary, error]),
                Ok(_) => error,
            },
        )
    })
}
fn input(children: &[Arc<dyn ExecutionPlan>]) -> Result<LogicalPlan> {
    let [child] = children else {
        return Err(invalid("Delta mutation requires one physical input"));
    };
    LogicalPlanBuilder::scan(
        "prepared_delta_input",
        provider_as_source(Arc::new(PhysicalInput::storage(Arc::clone(child)))),
        None,
    )?
    .build()
}
async fn write_count(
    table: &DeltaTable,
    version: u64,
    state: &SessionState,
) -> Result<Option<u64>> {
    let actions = super::super::actions::read(
        table,
        version,
        state,
        &pse_columnar::CancellationToken::new(),
    )
    .await?;
    let mut count_value = None;
    let mut seen = false;
    actions.visit(|action| {
        if let Action::CommitInfo(info) = action {
            if seen {
                return Err(invalid("duplicate write commit info"));
            }
            seen = true;
            let Some(metrics) = info.info.get("operationMetrics") else {
                return Ok(());
            };
            let metrics: WriteMetrics = serde_json::from_value(metrics.clone())
                .map_err(|e| DataFusionError::External(Box::new(e)))?;
            count_value = Some(count(metrics.num_added_rows)?);
        }
        Ok(())
    })?;
    Ok(count_value)
}
async fn row_count(table: DeltaTable, state: Arc<SessionState>) -> Result<u64> {
    let provider = table
        .table_provider()
        .with_session(Arc::clone(&state))
        .build()
        .await?;
    let plan = LogicalPlanBuilder::scan(
        "counted_version",
        provider_as_source(Arc::new(provider)),
        None,
    )?
    .aggregate(
        Vec::<Expr>::new(),
        vec![
            cast(
                datafusion::functions_aggregate::expr_fn::count(lit(1i64)),
                DataType::UInt64,
            )
            .alias("count"),
        ],
    )?
    .build()?;
    let batches = datafusion::physical_plan::collect(
        state.create_physical_plan(&plan).await?,
        state.task_ctx(),
    )
    .await?;
    let [batch] = batches.as_slice() else {
        return Err(invalid("native count did not return one batch"));
    };
    let array = batch
        .column(0)
        .as_any()
        .downcast_ref::<UInt64Array>()
        .ok_or_else(|| invalid("native count type changed"))?;
    if array.len() != 1 || array.is_null(0) {
        return Err(invalid("native count did not return one value"));
    }
    Ok(array.value(0))
}
pub(super) fn count(value: usize) -> Result<u64> {
    u64::try_from(value).map_err(|e| DataFusionError::External(Box::new(e)))
}
pub(super) fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.to_owned())
}

fn affected_difference(before: u64, after: u64, deleted: bool) -> Result<u64> {
    if deleted {
        before.checked_sub(after)
    } else {
        after.checked_sub(before)
    }
    .ok_or_else(|| invalid("native mutation changed the exact count in the wrong direction"))
}
#[cfg(test)]
mod count_unit {
    use super::*;
    #[test]
    fn exact_snapshot_differences_preserve_zero_and_reject_inconsistent_metrics() {
        assert_eq!(affected_difference(4, 7, false).unwrap(), 3);
        assert_eq!(affected_difference(4, 0, true).unwrap(), 4);
        assert_eq!(affected_difference(0, 0, false).unwrap(), 0);
        assert!(affected_difference(4, 3, false).is_err());
        assert!(affected_difference(3, 4, true).is_err());
    }
}
