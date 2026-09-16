// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Conditional publication over Delta's own control-table transaction and history.
//! Commit metadata is a search index only: reconciliation verifies the exact typed row.
use super::{
    layout::DurableLayout,
    publication::{PublicationRoot, read_record},
    write::PhysicalInput,
};
use datafusion::{
    arrow::{
        array::{RecordBatch, UInt64Array},
        datatypes::{DataType, Field, Schema},
    },
    catalog::Session,
    common::{DFSchema, DFSchemaRef, DataFusionError, Result, ScalarValue},
    datasource::{MemTable, provider_as_source},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{
        Expr, Extension, LogicalPlan, LogicalPlanBuilder, UserDefinedLogicalNode,
        UserDefinedLogicalNodeCore, col, lit, physical_planning_context::PhysicalPlanningContext,
    },
    physical_expr::EquivalenceProperties,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PlanProperties,
        SendableRecordBatchStream, execute_stream,
        execution_plan::{Boundedness, EmissionType},
        stream::RecordBatchStreamAdapter,
    },
    physical_planner::{ExtensionPlanner, PhysicalPlanner},
};
use deltalake::{
    DeltaTable,
    delta_datafusion::SessionFallbackPolicy,
    kernel::{Action, transaction::CommitProperties},
    protocol::SaveMode,
};
use futures_util::TryStreamExt;
use pse_relations::generated::runtime::publications;
use std::{
    hash::{Hash, Hasher},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// A rejected or unresolved publication. An unresolved outcome is never rollback.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum PublicationError {
    /// The complete request differs from an already committed attempt or identity.
    #[error("publication identity was reused with a different request")]
    #[diagnostic(code(config::invalid))]
    IdentityReused,
    /// A concurrent publication won, or the expected parent is no longer the head.
    #[error("publication parent changed")]
    #[diagnostic(code(runtime::infrastructure))]
    Conflict,
    /// Storage cannot establish whether the attempt committed. Retry the same request.
    #[error("publication outcome is unresolved: {detail}")]
    #[diagnostic(code(runtime::infrastructure))]
    Unresolved {
        /// Underlying observation failure; retained for reconciliation diagnostics.
        detail: String,
    },
}

#[derive(Debug)]
struct Request {
    location: url::Url,
    registry: Arc<pse_schema::Registry>,
    started: AtomicBool,
}

/// Native publication command with one real, typed control-row input.
#[derive(Debug, Clone)]
pub struct DeltaPublish {
    request: Arc<Request>,
    input: LogicalPlan,
    schema: DFSchemaRef,
}
impl DeltaPublish {
    /// Publish exactly one generated control row, comparing its expected parent.
    /// The input and all validation execute before the conditional control commit.
    /// # Errors
    /// The input does not have the declared control schema.
    pub fn plan(
        location: url::Url,
        input: LogicalPlan,
        registry: Arc<pse_schema::Registry>,
    ) -> Result<LogicalPlan> {
        let expected = publications::schema().map_err(external)?;
        if input.schema().as_arrow().fields() != expected.fields() {
            return Err(invalid("DeltaPublish input must be runtime.publications"));
        }
        let schema = Arc::new(DFSchema::try_from(Schema::new(vec![Field::new(
            "version",
            DataType::UInt64,
            false,
        )]))?);
        Ok(LogicalPlan::Extension(Extension {
            node: Arc::new(Self {
                request: Arc::new(Request {
                    location,
                    registry,
                    started: AtomicBool::new(false),
                }),
                input,
                schema,
            }),
        }))
    }
}
impl PartialEq for DeltaPublish {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.request, &other.request) && self.input == other.input
    }
}
impl Eq for DeltaPublish {}
impl Hash for DeltaPublish {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.request).hash(state);
        self.input.hash(state);
    }
}
impl PartialOrd for DeltaPublish {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match Arc::as_ptr(&self.request).cmp(&Arc::as_ptr(&other.request)) {
            std::cmp::Ordering::Equal => self.input.partial_cmp(&other.input),
            order => Some(order),
        }
    }
}
impl UserDefinedLogicalNodeCore for DeltaPublish {
    fn name(&self) -> &'static str {
        "DeltaPublish"
    }
    fn inputs(&self) -> Vec<&LogicalPlan> {
        vec![&self.input]
    }
    fn schema(&self) -> &DFSchemaRef {
        &self.schema
    }
    fn expressions(&self) -> Vec<Expr> {
        vec![]
    }
    fn fmt_for_explain(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DeltaPublish: conditional control transaction")
    }
    fn with_exprs_and_inputs(
        &self,
        exprs: Vec<Expr>,
        mut inputs: Vec<LogicalPlan>,
    ) -> Result<Self> {
        if !exprs.is_empty() || inputs.len() != 1 {
            return Err(invalid(
                "DeltaPublish requires one input and no expressions",
            ));
        }
        let input = inputs
            .pop()
            .ok_or_else(|| invalid("publication input absent"))?;
        Ok(Self {
            input,
            ..self.clone()
        })
    }
    fn prevent_predicate_push_down_columns(&self) -> std::collections::HashSet<String> {
        self.schema
            .fields()
            .iter()
            .map(|f| f.name().to_owned())
            .collect()
    }
}
#[derive(Debug)]
pub(crate) struct PublishPlanner;
#[async_trait::async_trait]
impl ExtensionPlanner for PublishPlanner {
    async fn plan_extension(
        &self,
        _: &dyn PhysicalPlanner,
        node: &dyn UserDefinedLogicalNode,
        _: &[&LogicalPlan],
        inputs: &[Arc<dyn ExecutionPlan>],
        session: &dyn Session,
        _: &PhysicalPlanningContext,
    ) -> Result<Option<Arc<dyn ExecutionPlan>>> {
        let Some(node) = node.as_any().downcast_ref::<DeltaPublish>() else {
            return Ok(None);
        };
        let state = session
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| invalid("DeltaPublish requires the actual SessionState"))?;
        let [input] = inputs else {
            return Err(invalid("DeltaPublish requires one physical child"));
        };
        Ok(Some(Arc::new(PublishExec {
            request: Arc::clone(&node.request),
            input: Arc::clone(input),
            state: Arc::new(state.clone()),
            properties: Arc::new(PlanProperties::new(
                EquivalenceProperties::new(Arc::new(node.schema.as_arrow().clone())),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Final,
                Boundedness::Bounded,
            )),
        })))
    }
}
#[derive(Debug)]
struct PublishExec {
    request: Arc<Request>,
    input: Arc<dyn ExecutionPlan>,
    state: Arc<SessionState>,
    properties: Arc<PlanProperties>,
}
impl DisplayAs for PublishExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DeltaPublishExec: expected parent, exact attempt reconciliation")
    }
}
impl ExecutionPlan for PublishExec {
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(
            &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
        ) -> Result<datafusion::common::tree_node::TreeNodeRecursion>,
    ) -> Result<datafusion::common::tree_node::TreeNodeRecursion> {
        Ok(datafusion::common::tree_node::TreeNodeRecursion::Continue)
    }
    fn name(&self) -> &'static str {
        "DeltaPublishExec"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        vec![&self.input]
    }
    fn with_new_children(
        self: Arc<Self>,
        mut children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if children.len() != 1 {
            return Err(invalid("DeltaPublish requires one physical child"));
        }
        Ok(Arc::new(Self {
            request: Arc::clone(&self.request),
            input: children
                .pop()
                .ok_or_else(|| invalid("publication input absent"))?,
            state: Arc::clone(&self.state),
            properties: Arc::clone(&self.properties),
        }))
    }
    fn execute(&self, partition: usize, _: Arc<TaskContext>) -> Result<SendableRecordBatchStream> {
        if partition != 0 || self.request.started.swap(true, Ordering::AcqRel) {
            return Err(invalid(
                "a prepared publication executes once in partition zero",
            ));
        }
        let request = Arc::clone(&self.request);
        let state = Arc::clone(&self.state);
        let input = Arc::clone(&self.input);
        let schema = self.schema();
        let output = Arc::clone(&schema);
        let stream = futures_util::stream::once(async move {
            let input = control_input(input)?;
            let batch = one_row(input, &state).await?;
            let registry = request.registry.as_ref();
            let record = publications::View::try_from_batch_with_registry(registry, &batch)
                .map_err(external)?
                .row(0)
                .map_err(external)?;
            let version =
                commit(&request.location, &record, batch, &state, &request.registry).await?;
            Ok(RecordBatch::try_new(
                schema,
                vec![Arc::new(UInt64Array::from(vec![version]))],
            )?)
        });
        Ok(Box::pin(RecordBatchStreamAdapter::new(output, stream)))
    }
}
fn control_input(input: Arc<dyn ExecutionPlan>) -> Result<Arc<dyn ExecutionPlan>> {
    super::layout::declared_output(input, &publications::schema().map_err(external)?)
}
async fn one_row(input: Arc<dyn ExecutionPlan>, state: &SessionState) -> Result<RecordBatch> {
    let mut stream = execute_stream(input, state.task_ctx())?;
    let mut row = None;
    while let Some(batch) = stream.try_next().await? {
        if batch.num_rows() == 0 {
            continue;
        }
        if batch.num_rows() != 1 || row.is_some() {
            return Err(invalid("publication command requires exactly one row"));
        }
        row = Some(batch);
    }
    row.ok_or_else(|| invalid("publication command requires exactly one row"))
}
async fn load(location: &url::Url, state: &SessionState) -> Result<Option<DeltaTable>> {
    let mut table = super::provider::table_builder(location.clone(), state)?
        .build()
        .map_err(external)?;
    if !table
        .verify_deltatable_existence()
        .await
        .map_err(external)?
    {
        return Ok(None);
    }
    table.load().await.map_err(external)?;
    Ok(Some(table))
}
async fn commit(
    location: &url::Url,
    record: &publications::Row,
    batch: RecordBatch,
    state: &Arc<SessionState>,
    registry: &Arc<pse_schema::Registry>,
) -> Result<u64> {
    let base = load(location, state).await?;
    if let Some(table) = &base {
        if let Some(version) = reconcile(table, record, state, registry).await? {
            return Ok(version);
        }
        let current = read_record(
            &PublicationRoot {
                location: location.clone(),
                version: version(table)?,
            },
            registry,
            Arc::clone(state),
        )
        .await?;
        if current.workspace_id != record.workspace_id
            || Some(current.publication_id) != record.parent_publication_id
        {
            return Err(external(PublicationError::Conflict));
        }
    } else if record.parent_publication_id.is_some() {
        return Err(external(PublicationError::Conflict));
    }

    if record.parent_publication_id == Some(record.publication_id) {
        return Err(invalid("a publication cannot be its own parent"));
    }
    super::publication::verify_inputs(record, registry, Arc::clone(state)).await?;
    let candidate = super::publication::bind_members(record, registry, Arc::clone(state)).await?;
    super::admission::admit(record, Arc::clone(registry), &candidate).await?;
    let physical = encoded_candidate(batch, state).await?;
    let commit = commit_properties(record);
    let result = if let Some(table) = base {
        let batch = one_row(physical, state).await?;
        let parent = record
            .parent_publication_id
            .ok_or_else(|| invalid("existing publication needs a parent"))?;
        let mut update = table
            .update()
            .with_session_state(state.clone())
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .with_predicate(
                col("publication_id")
                    .eq(lit(ScalarValue::Binary(Some(parent.as_bytes().to_vec())))),
            )
            .with_commit_properties(commit);
        for (field, column) in batch.schema().fields().iter().zip(batch.columns()) {
            update = update.with_update(
                field.name().as_str(),
                lit(ScalarValue::try_from_array(column, 0)?),
            );
        }
        update
            .await
            .map(|(table, metrics)| (table, metrics.num_updated_rows == 1))
    } else {
        let input = LogicalPlanBuilder::scan(
            "publication_candidate",
            provider_as_source(Arc::new(PhysicalInput(physical))),
            None,
        )?
        .build()?;
        super::provider::table_builder(location.clone(), state)?
            .build()
            .map_err(external)?
            .write(Vec::<RecordBatch>::new())
            .with_input_plan(input)
            .with_session_state(state.clone())
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .with_save_mode(SaveMode::ErrorIfExists)
            .with_commit_properties(commit)
            .await
            .map(|table| (table, true))
    };
    // Even a successful builder result is reconciled against the recorded row. A
    // post-commit failure and a lost acknowledgment take exactly the same path.
    let latest = load(location, state)
        .await
        .map_err(|error| unresolved(error.to_string()))?;
    if let Some(table) = latest
        && let Some(version) = reconcile(&table, record, state, registry)
            .await
            .map_err(|error| unresolved(error.to_string()))?
    {
        return Ok(version);
    }
    match result {
        Ok((_, false)) => Err(external(PublicationError::Conflict)),
        Ok(_) => Err(unresolved(
            "builder returned success without a recorded publication".into(),
        )),
        Err(error) => Err(unresolved(error.to_string())),
    }
}

async fn encoded_candidate(
    batch: RecordBatch,
    state: &SessionState,
) -> Result<Arc<dyn ExecutionPlan>> {
    let layout = DurableLayout::new(batch.schema())?;
    let input = LogicalPlanBuilder::scan(
        "publication_candidate",
        provider_as_source(Arc::new(MemTable::try_new(
            batch.schema(),
            vec![vec![batch]],
        )?)),
        None,
    )?
    .build()?;
    let encoded = layout.encode(input)?;
    state.create_physical_plan(&encoded).await
}
fn commit_properties(record: &publications::Row) -> CommitProperties {
    CommitProperties::default()
        .with_metadata([
            (
                "pse.attempt".into(),
                serde_json::json!(record.attempt_id.to_string()),
            ),
            (
                "pse.publication".into(),
                serde_json::json!(record.publication_id.to_string()),
            ),
        ])
        // An empty-table writer has no existing read set. Rebasing a second
        // creator can append a second head despite ErrorIfExists's initial check.
        .with_max_retries(0)
        .with_create_checkpoint(false)
        .with_cleanup_expired_logs(Some(false))
}

/// Find a committed attempt in retained Delta history and compare its complete row.
/// Missing history fails closed: absence is not evidence that a retry is safe.
async fn reconcile(
    table: &DeltaTable,
    requested: &publications::Row,
    state: &Arc<SessionState>,
    registry: &Arc<pse_schema::Registry>,
) -> Result<Option<u64>> {
    for version in (0..=version(table)?).rev() {
        let bytes = table
            .log_store()
            .read_commit_entry(version)
            .await
            .map_err(external)?
            .ok_or_else(|| unresolved(format!("control commit {version} is unavailable")))?;
        for action in serde_json::Deserializer::from_slice(&bytes).into_iter::<Action>() {
            let Action::CommitInfo(info) = action.map_err(external)? else {
                continue;
            };
            let attempt = serde_json::json!(requested.attempt_id.to_string());
            let publication = serde_json::json!(requested.publication_id.to_string());
            if info.info.get("pse.attempt") == Some(&attempt)
                || info.info.get("pse.publication") == Some(&publication)
            {
                let location = table.table_url().clone();
                let actual = read_record(
                    &PublicationRoot { location, version },
                    registry,
                    Arc::clone(state),
                )
                .await?;
                return if actual == *requested {
                    Ok(Some(version))
                } else {
                    Err(external(PublicationError::IdentityReused))
                };
            }
        }
    }
    Ok(None)
}
fn version(table: &DeltaTable) -> Result<u64> {
    table
        .version()
        .ok_or_else(|| invalid("control table has no version"))
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
fn unresolved(detail: String) -> DataFusionError {
    external(PublicationError::Unresolved { detail })
}
