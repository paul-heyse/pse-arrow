// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Conditional publication over Delta's own control-table transaction and history.
//! Commit metadata is a search index only: reconciliation verifies the exact typed row.
use super::{
    layout::DurableLayout,
    publication::{PublicationRoot, read_optional_record, read_record},
};
use datafusion::{
    arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::Schema,
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
    kernel::{Transaction, transaction::CommitProperties},
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
#[derive(Clone)]
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
        let schema = Arc::new(DFSchema::try_from(Schema::new(vec![
            pse_schema::model::IntegerRange::NONNEGATIVE.field("version"),
        ]))?);
        Ok(crate::session::contract::ExecutionContract::plan(
            LogicalPlan::Extension(Extension {
                node: Arc::new(Self {
                    request: Arc::new(Request {
                        location,
                        registry,
                        started: AtomicBool::new(false),
                    }),
                    input,
                    schema,
                }),
            }),
            None,
            effects(),
        ))
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
// Native plan renderers visit children separately. Debug describes this
// node without recursively duplicating complete subgraphs in JSON.
impl std::fmt::Debug for DeltaPublish {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UserDefinedLogicalNodeCore::fmt_for_explain(self, f)
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
        crate::session::execution::NativeExecutionContext::from_session(session)?
            .admit_effects(&effects())
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
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
            let services =
                crate::session::execution::NativeExecutionContext::from_session(state.as_ref())?;
            let _writer = super::lease::write(&request.location, services.cancellation()).await?;
            services.require_settlement();
            let input = control_input(input)
                .map_err(|error| error.context("bind publication control input"))?;
            let batch = one_row(input, &state)
                .await
                .map_err(|error| error.context("collect publication control input"))?;
            let registry = request.registry.as_ref();
            let record = publications::View::try_from_batch_with_registry(registry, &batch)
                .map_err(external)?
                .row(0)
                .map_err(external)?;
            let version = commit(&request.location, &record, batch, &state, &request.registry)
                .await
                .map_err(|error| error.context("commit publication control record"))?;
            Ok(RecordBatch::try_new(
                schema,
                vec![Arc::new(Int64Array::from(vec![version]))],
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
async fn load(
    location: &url::Url,
    state: &SessionState,
) -> Result<Option<super::provider::Opened>> {
    let table = super::provider::table_builder(location.clone(), state)?
        .build()
        .map_err(external)?;
    if !table
        .verify_deltatable_existence()
        .await
        .map_err(external)?
    {
        return Ok(None);
    }
    Ok(Some(
        super::provider::open_native(
            location.clone(),
            None,
            crate::cache_service::snapshot::LoadRequirement::Query,
            &Arc::new(state.clone()),
        )
        .await?,
    ))
}

async fn commit(
    location: &url::Url,
    record: &publications::Row,
    batch: RecordBatch,
    state: &Arc<SessionState>,
    registry: &Arc<pse_schema::Registry>,
) -> Result<i64> {
    let contract = super::contract::DeclaredCheck::new(
        registry,
        publications::spec(registry).map_err(external)?.id,
    )?;
    let state = Arc::new(
        contract
            .bind(state)
            .map_err(|error| error.context("bind publication control contract"))?,
    );
    let base_owner = load(location, &state).await?;
    let base = base_owner.as_ref().map(|opened| opened.table.clone());
    if let Some(table) = &base {
        contract.verify(table)?;
        if let Some(version) = reconcile(table, record, &state, registry).await? {
            return Ok(version);
        }
    }
    let has_head = admit_parent(base.as_ref(), location, record, &state, registry).await?;

    if record.parent_publication_id == Some(record.publication_id) {
        return Err(invalid("a publication cannot be its own parent"));
    }
    super::publication::verify_inputs(record, registry, Arc::clone(&state)).await?;
    let candidate = super::publication::bind_members(record, registry, Arc::clone(&state))
        .await
        .map_err(|error| error.context("bind publication members"))?;
    super::admission::admit(record, Arc::clone(registry), &candidate)
        .await
        .map_err(|error| error.context("admit publication members"))?;
    let physical = encoded_candidate(batch, &state)
        .await
        .map_err(|error| error.context("encode publication control record"))?;
    let initialized_owner;
    let (table, has_head) = if let Some(table) = base {
        (table, has_head)
    } else {
        initialized_owner = initialize_control(location, &state, &contract).await?;
        let table = initialized_owner.table.clone();
        contract.verify(&table)?;
        if let Some(version) = reconcile(&table, record, &state, registry).await? {
            return Ok(version);
        }
        let head = admit_parent(Some(&table), location, record, &state, registry).await?;
        (table, head)
    };
    let next = version(&table)?
        .checked_add(1)
        .ok_or_else(|| invalid("publication version overflows"))?;
    let commit = commit_properties(record, next);
    let result = if has_head {
        let batch = one_row(physical, &state).await?;
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
            provider_as_source(Arc::new(
                crate::session::physical_input::PhysicalInput::storage(physical),
            )),
            None,
        )?
        .build()?;
        table
            .write(Vec::<RecordBatch>::new())
            .with_input_plan(input)
            .with_session_state(state.clone())
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .with_save_mode(SaveMode::Append)
            .with_commit_properties(commit)
            .await
            .map(|table| (table, true))
    };
    // Successful builders already return the exact native committed state.
    // Only ambiguous failures require an uncached log observation/reconciliation.
    settle_publication(location, record, result, &state, registry).await
}

async fn settle_publication(
    location: &url::Url,
    record: &publications::Row,
    result: std::result::Result<(DeltaTable, bool), deltalake::DeltaTableError>,
    state: &Arc<SessionState>,
    registry: &Arc<pse_schema::Registry>,
) -> Result<i64> {
    let error = match result {
        Ok((table, true)) => {
            if let Some(version) = reconcile(&table, record, state, registry).await? {
                super::provider::committed(&table, state).await;
                return Ok(version);
            }
            return Err(unresolved(
                "native commit has no matching publication receipt".into(),
            ));
        }
        Ok((_, false)) => return Err(external(PublicationError::Conflict)),
        Err(error) => error,
    };
    let latest = load(location, state)
        .await
        .map_err(|error| unresolved(error.to_string()))?;
    if let Some(opened) = latest
        && let Some(version) = reconcile(&opened.table, record, state, registry)
            .await
            .map_err(|error| unresolved(error.to_string()))?
    {
        super::provider::committed(&opened.table, state).await;
        return Ok(version);
    }
    Err(unresolved(error.to_string()))
}

async fn initialize_control(
    location: &url::Url,
    state: &SessionState,
    contract: &super::contract::DeclaredCheck,
) -> Result<super::provider::Opened> {
    let empty = super::provider::table_builder(location.clone(), state)?
        .build()
        .map_err(external)?;
    // Schema/CHECK creation has no publication identity. A lost creation response
    // or concurrent initializer is reconciled by opening the actual declared table.
    match contract
        .create(
            empty,
            CommitProperties::default()
                .with_max_retries(0)
                .with_create_checkpoint(true)
                .with_cleanup_expired_logs(Some(false)),
        )
        .await
    {
        Ok(table) => Ok(super::provider::Opened { table, owner: None }),
        Err(error) => load(location, state)
            .await?
            .ok_or_else(|| unresolved(error.to_string())),
    }
}

async fn admit_parent(
    table: Option<&DeltaTable>,
    location: &url::Url,
    requested: &publications::Row,
    state: &Arc<SessionState>,
    registry: &Arc<pse_schema::Registry>,
) -> Result<bool> {
    let current = match table {
        Some(table) => {
            read_optional_record(
                &PublicationRoot {
                    location: location.clone(),
                    version: version(table)?,
                },
                registry,
                Arc::clone(state),
            )
            .await?
        }
        None => None,
    };
    match current {
        Some(current)
            if current.workspace_id == requested.workspace_id
                && Some(current.publication_id) == requested.parent_publication_id =>
        {
            Ok(true)
        }
        None if requested.parent_publication_id.is_none() => Ok(false),
        _ => Err(external(PublicationError::Conflict)),
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
fn commit_properties(record: &publications::Row, version: i64) -> CommitProperties {
    CommitProperties::default()
        .with_application_transaction(Transaction::new(format!("pse.attempt:{}", record.attempt_id), version))
        .with_application_transaction(Transaction::new(format!("pse.publication:{}", record.publication_id), version))
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
        // Concurrent first-head writers must compete for the same native version.
        // Never rebase an append after another writer has published a head.
        .with_max_retries(0)
        .with_create_checkpoint(true)
        .with_cleanup_expired_logs(Some(false))
}

/// Find a committed attempt in retained Delta history and compare its complete row.
/// Missing history fails closed: absence is not evidence that a retry is safe.
async fn reconcile(
    table: &DeltaTable,
    requested: &publications::Row,
    state: &Arc<SessionState>,
    registry: &Arc<pse_schema::Registry>,
) -> Result<Option<i64>> {
    let snapshot = table.snapshot().map_err(external)?;
    let log = table.log_store();
    let attempt = snapshot
        .transaction_version(
            log.as_ref(),
            format!("pse.attempt:{}", requested.attempt_id),
        )
        .await
        .map_err(|error| unresolved(error.to_string()))?;
    let publication = snapshot
        .transaction_version(
            log.as_ref(),
            format!("pse.publication:{}", requested.publication_id),
        )
        .await
        .map_err(|error| unresolved(error.to_string()))?;
    let selected = match (attempt, publication) {
        (None, None) => return Ok(None),
        (Some(left), Some(right)) if left == right => left,
        _ => return Err(external(PublicationError::IdentityReused)),
    };
    let actual = read_record(
        &PublicationRoot {
            location: table.table_url().clone(),
            version: selected,
        },
        registry,
        Arc::clone(state),
    )
    .await
    .map_err(|error| unresolved(error.to_string()))?;
    if actual != *requested {
        return Err(external(PublicationError::IdentityReused));
    }
    Ok(Some(selected))
}

fn version(table: &DeltaTable) -> Result<i64> {
    let version = table
        .version()
        .ok_or_else(|| invalid("control table has no version"))?;
    super::provider::signed_version(version)
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

fn effects() -> std::collections::BTreeSet<pse_schema::model::provider::OperationEffect> {
    use pse_schema::model::provider::OperationEffect::{Publish, Read, Write};
    [Read, Write, Publish].into_iter().collect()
}
