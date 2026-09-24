// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Conditional publication uses Delta transactions; engine owns command settlement.
use super::{
    layout::DurableLayout,
    publication::{PublicationRoot, read_optional_record, read_record},
};
use datafusion::{
    arrow::array::{Int64Array, RecordBatch},
    common::{DataFusionError, Result, ScalarValue},
    datasource::{MemTable, provider_as_source},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, col, lit},
    physical_plan::{ExecutionPlan, SendableRecordBatchStream, execute_stream},
};
use deltalake::{
    DeltaTable,
    kernel::{Transaction, transaction::CommitProperties},
    protocol::SaveMode,
};
use futures_util::TryStreamExt;
use pse_engine::operation::{Body, Definition, Family, Operation};
use pse_relations::generated::runtime::publications;
use std::sync::Arc;
/// A rejected or unresolved publication. An unresolved outcome is never rollback.
#[derive(Debug, thiserror::Error)]
pub enum PublicationError {
    /// The complete request differs from an already committed attempt or identity.
    #[error("publication identity was reused with a different request")]
    IdentityReused,
    /// A concurrent publication won, or the expected parent is no longer the head.
    #[error("publication parent changed")]
    Conflict,
    /// Storage cannot establish whether the attempt committed. Retry the same request.
    #[error("publication outcome is unresolved: {source}")]
    Unresolved {
        /// Underlying observation failure; retained for reconciliation diagnostics.
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

#[derive(Debug)]
struct Request {
    location: url::Url,
    registry: Arc<pse_schema::Registry>,
}
/// Construct a native publication over an exact control-row input.
#[derive(Debug)]
pub struct DeltaPublish;
impl DeltaPublish {
    /// Publish a declared control row after all native dependencies complete.
    /// # Errors
    /// The input differs from the control relation.
    pub fn plan(
        location: url::Url,
        input: LogicalPlan,
        registry: Arc<pse_schema::Registry>,
    ) -> Result<LogicalPlan> {
        if input.schema().as_arrow().fields() != publications::schema().map_err(external)?.fields()
        {
            return Err(invalid("DeltaPublish input must be runtime.publications"));
        }
        Ok(pse_engine::session::contract::ExecutionContract::plan(
            Operation::plan(Arc::new(Request { location, registry }), vec![input])?,
            None,
            effects(),
        ))
    }
}
#[async_trait::async_trait]
impl Definition for Request {
    fn name(&self) -> &'static str {
        "DeltaPublish"
    }
    fn schema(&self) -> datafusion::arrow::datatypes::SchemaRef {
        super::write::version_schema()
    }
    fn family(&self) -> Family {
        Family::Command
    }
    fn effects(&self) -> std::collections::BTreeSet<pse_schema::model::provider::OperationEffect> {
        effects()
    }
    async fn prepare(
        self: Arc<Self>,
        _: &[Expr],
        _: &[LogicalPlan],
        _: &[Arc<dyn ExecutionPlan>],
        state: &SessionState,
    ) -> Result<Arc<dyn Body>> {
        Ok(Arc::new(PublishBody {
            request: self,
            state: Arc::new(state.clone()),
        }))
    }
}
#[derive(Debug)]
struct PublishBody {
    request: Arc<Request>,
    state: Arc<SessionState>,
}
impl Body for PublishBody {
    fn execute(
        &self,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
        _: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let [input]: [_; 1] = inputs
            .try_into()
            .map_err(|_| invalid("publication requires one child"))?;
        let request = self.request.clone();
        let state = self.state.clone();
        Ok(pse_engine::operation::batch(
            super::write::version_schema(),
            async move { run(request, input, state).await },
        ))
    }
}
async fn run(
    request: Arc<Request>,
    input: Arc<dyn ExecutionPlan>,
    state: Arc<SessionState>,
) -> Result<RecordBatch> {
    let schema = super::write::version_schema();
    let services =
        pse_engine::session::execution::NativeExecutionContext::from_session(state.as_ref())?;
    let _writer = super::lease::write(&request.location, services.cancellation()).await?;
    services.require_settlement();
    let input =
        control_input(input).map_err(|error| error.context("bind publication control input"))?;
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

#[tracing::instrument(name = "pse.delta.commit", skip_all, fields(publication_id = %record.publication_id, parent_publication_id = ?record.parent_publication_id, committed_version = tracing::field::Empty), err)]
async fn commit(
    location: &url::Url,
    record: &publications::Row,
    batch: RecordBatch,
    state: &Arc<SessionState>,
    registry: &Arc<pse_schema::Registry>,
) -> Result<i64> {
    let result = commit_inner(location, record, batch, state, registry).await;
    if let Ok(version) = &result {
        tracing::Span::current().record("committed_version", *version);
    }
    result
}

async fn commit_inner(
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
    let context = super::operation::DeltaOperationContext::new(
        state.clone(),
        Some(contract.clone()),
        CommitProperties::default(),
        super::operation::CommitKind::Publication,
    )?;
    let state = context.state.clone();
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
        let initialized = initialize_control(location, &state, &contract).await?;
        initialized_owner = initialized.0;
        let table = initialized_owner.table.clone();
        contract.verify(&table)?;
        if initialized.1 {
            super::settlement::observed(
                table.version(),
                super::settlement::expected_version(table.version(), 0),
            )?;
            if record.parent_publication_id.is_some() {
                return Err(external(PublicationError::Conflict));
            }
            (table, false)
        } else {
            if let Some(version) = reconcile(&table, record, &state, registry).await? {
                return Ok(version);
            }
            let head = admit_parent(Some(&table), location, record, &state, registry).await?;
            (table, head)
        }
    };
    let next = version(&table)?
        .checked_add(1)
        .ok_or_else(|| invalid("publication version overflows"))?;
    let commit = commit_properties(record, next);
    let context = context.with_commit(commit, super::operation::CommitKind::Publication);
    let result = if has_head {
        let batch = one_row(physical, &state).await?;
        let parent = record
            .parent_publication_id
            .ok_or_else(|| invalid("existing publication needs a parent"))?;
        let mut update = context.update(table).with_predicate(
            col("publication_id").eq(lit(ScalarValue::Binary(Some(parent.as_bytes().to_vec())))),
        );
        for (field, column) in batch.schema().fields().iter().zip(batch.columns()) {
            update = update.with_update(
                field.name().as_str(),
                lit(ScalarValue::try_from_array(column, 0)?),
            );
        }
        update
            .await
            .map(|(table, metrics)| (table, metrics.num_updated_rows))
    } else {
        let input = LogicalPlanBuilder::scan(
            "publication_candidate",
            provider_as_source(Arc::new(
                pse_engine::session::physical_input::PhysicalInput::storage(physical),
            )),
            None,
        )?
        .build()?;
        context
            .write(table, input, SaveMode::Append)
            .await
            .map(|table| (table, 1))
    };
    // Successful builders already return the exact native committed state.
    // Only ambiguous failures require an uncached log observation/reconciliation.
    settle_publication(location, record, next, result, &state, registry).await
}

async fn settle_publication(
    location: &url::Url,
    record: &publications::Row,
    expected: i64,
    result: std::result::Result<(DeltaTable, usize), deltalake::DeltaTableError>,
    state: &Arc<SessionState>,
    registry: &Arc<pse_schema::Registry>,
) -> Result<i64> {
    let error = match result {
        Ok((table, rows)) => {
            let version = control_success(table.version(), rows, expected)?;
            super::provider::committed(&table, state).await;
            return Ok(version);
        }
        Err(error) => Arc::new(error),
    };
    let latest = load(location, state).await.map_err(|observation| {
        unresolved(super::settlement::recovery_failed(
            error.clone(),
            observation,
        ))
    })?;
    if let Some(opened) = latest
        && let Some(version) = reconcile(&opened.table, record, state, registry)
            .await
            .map_err(|observation| {
                unresolved(super::settlement::recovery_failed(
                    error.clone(),
                    observation,
                ))
            })?
    {
        super::provider::committed(&opened.table, state).await;
        return Ok(version);
    }
    Err(unresolved(error))
}

async fn initialize_control(
    location: &url::Url,
    state: &SessionState,
    contract: &super::contract::DeclaredCheck,
) -> Result<(super::provider::Opened, bool)> {
    let empty = super::provider::table_builder(location.clone(), state)?
        .build()
        .map_err(external)?;
    // Schema/CHECK creation has no publication identity. A lost creation response
    // or concurrent initializer is reconciled by opening the actual declared table.
    match contract
        .create(
            empty,
            super::operation::commit_policy(
                CommitProperties::default(),
                super::operation::CommitKind::Publication,
            ),
        )
        .await
    {
        Ok(table) => Ok((super::provider::Opened { table, owner: None }, true)),
        Err(error) => {
            let primary: Arc<deltalake::DeltaTableError> = Arc::new(error.into());
            load(location, state)
                .await
                .map_err(|observation| {
                    unresolved(super::settlement::recovery_failed(
                        primary.clone(),
                        observation,
                    ))
                })?
                .map(|opened| (opened, false))
                .ok_or_else(|| unresolved(primary))
        }
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
    super::operation::commit_policy(
        CommitProperties::default()
            .with_application_transaction(Transaction::new(
                format!("pse.attempt:{}", record.attempt_id),
                version,
            ))
            .with_application_transaction(Transaction::new(
                format!("pse.publication:{}", record.publication_id),
                version,
            ))
            .with_metadata([
                (
                    "pse.attempt".into(),
                    serde_json::json!(record.attempt_id.to_string()),
                ),
                (
                    "pse.publication".into(),
                    serde_json::json!(record.publication_id.to_string()),
                ),
            ]), // Concurrent first-head writers must compete for the same native version.
        // Never rebase an append after another writer has published a head.
        super::operation::CommitKind::Publication,
    )
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
        .map_err(unresolved)?;
    let publication = snapshot
        .transaction_version(
            log.as_ref(),
            format!("pse.publication:{}", requested.publication_id),
        )
        .await
        .map_err(unresolved)?;
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
    .map_err(unresolved)?;
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
fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}
fn unresolved(source: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> DataFusionError {
    external(PublicationError::Unresolved {
        source: source.into(),
    })
}

fn effects() -> std::collections::BTreeSet<pse_schema::model::provider::OperationEffect> {
    use pse_schema::model::provider::OperationEffect::{Publish, Read, Write};
    [Read, Write, Publish].into_iter().collect()
}

pse_diagnostics::impl_diagnostic! {
    PublicationError,
    code(this) { match this {
            Self::IdentityReused => Some(pse_diagnostics::DiagnosticCode::ConfigInvalid),
            Self::Conflict | Self::Unresolved { .. } => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),

            _ => None,
        } },
    forward(_this) { None },
    help(_this) { None },
    related(_this) { None },
    source(_this) { None }
}

fn control_success(actual: Option<u64>, rows: usize, expected: i64) -> Result<i64> {
    match rows {
        0 => Err(external(PublicationError::Conflict)),
        1 => super::settlement::observed(
            actual,
            super::settlement::expected_version(
                actual,
                u64::try_from(expected)
                    .map_err(|error| DataFusionError::External(Box::new(error)))?,
            ),
        )
        .map(|()| expected),
        _ => super::settlement::observed(
            actual,
            Err(invalid(
                "publication update affected more than one control row",
            )),
        ),
    }
}

#[cfg(test)]
mod completion_unit {
    use super::*;
    #[test]
    fn known_success_requires_exact_transition_and_exactly_one_control_row() {
        assert_eq!(control_success(Some(8), 1, 8).unwrap(), 8);
        assert!(control_success(Some(7), 1, 8).is_err());
        assert!(control_success(None, 1, 8).is_err());
        let error = control_success(Some(7), 0, 8).unwrap_err();
        let observations = pse_columnar::observe(&error, pse_columnar::PlanOrigin::Analytics);
        assert_eq!(observations.len(), 1);
        let cause: &dyn std::error::Error = observations[0].domain_cause.unwrap();
        assert!(matches!(
            cause.downcast_ref::<PublicationError>(),
            Some(PublicationError::Conflict)
        ));
        assert!(control_success(Some(8), 2, 8).is_err());
    }
}

pse_columnar::impl_native_error!(PublicationError);
