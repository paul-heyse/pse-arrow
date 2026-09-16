// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Effect timing, real physical children and native Delta operation execution.
use crate::delta::write::PhysicalInput;
use datafusion::{
    arrow::{
        array::{Array, RecordBatch, UInt64Array},
        datatypes::{DataType, Field, Schema},
    },
    common::{DataFusionError, Result},
    datasource::provider_as_source,
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, cast, dml::MergeIntoClause, lit},
    physical_expr::EquivalenceProperties,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PlanProperties,
        SendableRecordBatchStream,
        execution_plan::{Boundedness, EmissionType},
        stream::RecordBatchStreamAdapter,
    },
};
use deltalake::{
    DeltaTable,
    delta_datafusion::SessionFallbackPolicy,
    kernel::{Action, transaction::CommitProperties},
    operations::write::WriteMetrics,
    protocol::SaveMode,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// A native mutation could have committed even when outcome delivery failed.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum MutationError {
    /// Inspect the native Delta log before retrying an uncertain command.
    #[error("Delta mutation outcome is unresolved: {source}")]
    #[diagnostic(code(runtime::infrastructure))]
    Unresolved {
        /// Original operation or outcome observation error.
        source: deltalake::DeltaTableError,
    },
    /// The native commit completed; only result observation failed.
    #[error("Delta mutation committed version {version}; subsequent work failed: {source}")]
    #[diagnostic(code(runtime::infrastructure))]
    Committed {
        /// Actual committed Delta version.
        version: u64,
        /// Result observation error.
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

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
pub(super) struct MutationExec {
    table: DeltaTable,
    state: Arc<SessionState>,
    commit: CommitProperties,
    command: Command,
    children: Vec<Arc<dyn ExecutionPlan>>,
    properties: Arc<PlanProperties>,
    started: Arc<AtomicBool>,
}
impl MutationExec {
    pub(super) fn new(
        table: DeltaTable,
        state: Arc<SessionState>,
        commit: CommitProperties,
        command: Command,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Self {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "count",
            DataType::UInt64,
            false,
        )]));
        Self {
            table,
            state,
            commit,
            command,
            children,
            started: Arc::default(),
            properties: Arc::new(PlanProperties::new(
                EquivalenceProperties::new(schema),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Final,
                Boundedness::Bounded,
            )),
        }
    }
}
impl DisplayAs for MutationExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeltaMutationExec: {:?}", self.command)
    }
}
impl ExecutionPlan for MutationExec {
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(
            &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
        ) -> Result<datafusion::common::tree_node::TreeNodeRecursion>,
    ) -> Result<datafusion::common::tree_node::TreeNodeRecursion> {
        Ok(datafusion::common::tree_node::TreeNodeRecursion::Continue)
    }
    fn name(&self) -> &'static str {
        "DeltaMutationExec"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        self.children.iter().collect()
    }
    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if children.len() != self.children.len() {
            return Err(invalid("Delta mutation child count changed"));
        }
        Ok(Arc::new(Self {
            table: self.table.clone(),
            state: Arc::clone(&self.state),
            commit: self.commit.clone(),
            command: self.command.clone(),
            children,
            properties: Arc::clone(&self.properties),
            started: Arc::clone(&self.started),
        }))
    }
    fn execute(&self, partition: usize, _: Arc<TaskContext>) -> Result<SendableRecordBatchStream> {
        if partition != 0 || self.started.swap(true, Ordering::AcqRel) {
            return Err(invalid(
                "a prepared Delta mutation executes once in partition zero",
            ));
        }
        let table = self.table.clone();
        let state = Arc::clone(&self.state);
        let commit = self.commit.clone();
        let command = self.command.clone();
        let children = self.children.clone();
        let schema = self.schema();
        let output = Arc::clone(&schema);
        let stream = futures_util::stream::once(async move {
            let count = run(table, state, commit, command, children).await?;
            Ok(RecordBatch::try_new(
                schema,
                vec![Arc::new(UInt64Array::from(vec![count]))],
            )?)
        });
        Ok(Box::pin(RecordBatchStreamAdapter::new(output, stream)))
    }
}
async fn run(
    table: DeltaTable,
    state: Arc<SessionState>,
    commit: CommitProperties,
    command: Command,
    children: Vec<Arc<dyn ExecutionPlan>>,
) -> Result<u64> {
    match command {
        Command::Insert(mode) => {
            let input = input(&children)?;
            let previous = table.version();
            let table = table
                .write(Vec::<RecordBatch>::new())
                .with_input_plan(input)
                .with_save_mode(mode)
                .with_session_state(state)
                .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
                .with_cast_safety(false)
                .with_commit_properties(commit)
                .await
                .map_err(unresolved)?;
            if table.version() == previous {
                return Ok(0);
            }
            let version = table
                .version()
                .ok_or_else(|| invalid("Delta write omitted its version"))?;
            write_count(&table, version)
                .await
                .map_err(|error| committed(version, error))
        }
        Command::Update {
            assignments,
            filters,
        } => {
            let mut builder = table
                .update()
                .with_session_state(state)
                .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
                .with_safe_cast(false)
                .with_commit_properties(commit);
            if let Some(predicate) = filters.into_iter().reduce(Expr::and) {
                builder = builder.with_predicate(predicate);
            }
            for (name, value) in assignments {
                builder = builder.with_update(datafusion::common::Column::from_name(name), value);
            }
            let (_, metrics) = builder.await.map_err(unresolved)?;
            count(metrics.num_updated_rows)
        }
        Command::Delete(filters) => {
            let before = table.clone();
            let mut builder = table
                .delete()
                .with_session_state(state.clone())
                .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
                .with_commit_properties(commit);
            if let Some(predicate) = filters.into_iter().reduce(Expr::and) {
                builder = builder.with_predicate(predicate);
            }
            let (after, metrics) = builder.await.map_err(unresolved)?;
            if let Some(rows) = metrics.num_deleted_rows {
                return count(rows);
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
                before
                    .checked_sub(after)
                    .ok_or_else(|| invalid("delete increased the row count"))
            }
            .await;
            observed.map_err(|error| committed(version, error))
        }
        Command::Merge { on, clauses } => {
            super::merge::execute(table, state, commit, input(&children)?, on, clauses).await
        }
    }
}
fn input(children: &[Arc<dyn ExecutionPlan>]) -> Result<LogicalPlan> {
    let [child] = children else {
        return Err(invalid("Delta mutation requires one physical input"));
    };
    LogicalPlanBuilder::scan(
        "prepared_delta_input",
        provider_as_source(Arc::new(PhysicalInput(Arc::clone(child)))),
        None,
    )?
    .build()
}
async fn write_count(table: &DeltaTable, version: u64) -> Result<u64> {
    let bytes = table
        .log_store()
        .read_commit_entry(version)
        .await
        .map_err(unresolved)?
        .ok_or_else(|| invalid("committed Delta log entry unavailable"))?;
    for line in bytes.split(|b| *b == b'\n').filter(|line| !line.is_empty()) {
        let action: Action =
            serde_json::from_slice(line).map_err(|e| DataFusionError::External(Box::new(e)))?;
        if let Action::CommitInfo(info) = action {
            let metrics: WriteMetrics = serde_json::from_value(
                info.info
                    .get("operationMetrics")
                    .cloned()
                    .ok_or_else(|| invalid("write metrics unavailable"))?,
            )
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
            return count(metrics.num_added_rows);
        }
    }
    Err(invalid("write commit info unavailable"))
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
pub(super) fn unresolved(source: deltalake::DeltaTableError) -> DataFusionError {
    DataFusionError::External(Box::new(MutationError::Unresolved { source }))
}
fn committed(
    version: u64,
    error: impl std::error::Error + Send + Sync + 'static,
) -> DataFusionError {
    DataFusionError::External(Box::new(MutationError::Committed {
        version,
        source: Box::new(error),
    }))
}
pub(super) fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.to_owned())
}
