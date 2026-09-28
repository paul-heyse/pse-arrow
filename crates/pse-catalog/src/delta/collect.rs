// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Member-table maintenance the catalog instructs (ADR-0114 Outcome 11; Plan 22 X10).
//!
//! The catalog owns what must stay reachable: it advances the workspace's maintenance
//! epoch, then fixes each table's protected ranges (live publications, their change
//! windows, live intents' prefixes). This module executes on one table exactly what that
//! allows: it verifies every protected version opens, commits a maintenance fence,
//! checkpoints, vacuums with those versions kept and cleans the log below the oldest
//! one unless a live intent's prefix covers the table. It also removes the tables of a
//! retired publication and the prefix of a reclaimed intent. None of it holds a lease:
//! the catalog's marks and epoch are the coordination.
use datafusion::{
    common::{DataFusionError, ResolvedTableReference, Result},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{Expr, LogicalPlan},
    physical_plan::{ExecutionPlan, SendableRecordBatchStream},
};
use futures_util::{StreamExt, TryStreamExt};
use pse_engine::{
    EngineError,
    operation::{Body, Definition, Family, Operation},
    session::{EngineSession, PreparedComputation},
};
use pse_relations::generated::{
    enums::RetentionReason,
    runtime::{maintenance_outcomes, retained_versions},
};
use pse_schema::model::provider::{OperationEffect, OperationPurpose, ProviderScope};
use std::{collections::BTreeSet, sync::Arc};

/// What collection does to a table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectAction {
    /// Checkpoint and remove unreferenced data files and log entries.
    Collect,
    /// Compact before checkpointing and collecting.
    Optimize,
}

/// One member table the catalog fixed for collection.
#[derive(Clone, Debug)]
pub struct CollectTarget {
    /// Qualified name governing the operation's policy.
    pub reference: ResolvedTableReference,
    /// The Delta table.
    pub location: url::Url,
    /// What to do.
    pub action: CollectAction,
    /// Millisecond cutoff for log cleanup; the table's own log retention still applies.
    pub log_cutoff_ms: i64,
}

/// Prepare the collection of one table keeping `retained`: the ranges the catalog fixed
/// after advancing the workspace epoch. Ranges of other tables are ignored; an
/// attempt range covers the tables under its prefix.
/// # Errors
/// An invalid cutoff, target policy or planning failure.
pub fn prepare_collect(
    session: &EngineSession,
    target: CollectTarget,
    retained: Vec<retained_versions::Row>,
    cancel: &pse_columnar::CancellationToken,
) -> std::result::Result<PreparedComputation, EngineError> {
    let now = super::maintenance::now_ms().map_err(pse_engine::session::engine)?;
    if target.log_cutoff_ms < 0 || target.log_cutoff_ms > now {
        return Err(pse_engine::session::engine(invalid(
            "log cleanup cutoff must be between the Unix epoch and now",
        )));
    }
    let mut session = session.with_purpose(OperationPurpose::Publish);
    session.bind_target(ProviderScope::Table(
        target.reference.catalog.to_string(),
        target.reference.schema.to_string(),
        target.reference.table.to_string(),
    ));
    let schema = maintenance_outcomes::schema().map_err(EngineError::from)?;
    let plan = Operation::plan(
        Arc::new(Request {
            target,
            retained,
            schema,
        }),
        vec![],
    )
    .map_err(pse_engine::session::engine)?;
    session.prepare(
        pse_engine::session::contract::ExecutionContract::plan(plan, None, effects()),
        cancel,
    )
}

#[derive(Debug)]
struct Request {
    target: CollectTarget,
    retained: Vec<retained_versions::Row>,
    schema: datafusion::arrow::datatypes::SchemaRef,
}
#[async_trait::async_trait]
impl Definition for Request {
    fn name(&self) -> &'static str {
        "DeltaCollect"
    }
    fn schema(&self) -> datafusion::arrow::datatypes::SchemaRef {
        Arc::clone(&self.schema)
    }
    fn family(&self) -> Family {
        Family::Command
    }
    fn effects(&self) -> BTreeSet<OperationEffect> {
        effects()
    }
    async fn prepare(
        self: Arc<Self>,
        _: &[Expr],
        _: &[LogicalPlan],
        _: &[Arc<dyn ExecutionPlan>],
        state: &SessionState,
    ) -> Result<Arc<dyn Body>> {
        Ok(Arc::new(CollectBody {
            request: self,
            state: Arc::new(state.clone()),
        }))
    }
}
#[derive(Debug)]
struct CollectBody {
    request: Arc<Request>,
    state: Arc<SessionState>,
}
impl Body for CollectBody {
    fn execute(
        &self,
        _: Vec<Arc<dyn ExecutionPlan>>,
        _: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let request = self.request.clone();
        let state = self.state.clone();
        Ok(pse_engine::operation::batch(
            Arc::clone(&request.schema),
            async move { collect(&request, state).await },
        ))
    }
}

#[tracing::instrument(name = "pse.delta.collect", skip_all, fields(table = %request.target.location, action = ?request.target.action), err)]
async fn collect(
    request: &Request,
    state: Arc<SessionState>,
) -> Result<datafusion::arrow::array::RecordBatch> {
    let services =
        pse_engine::session::execution::NativeExecutionContext::from_session(state.as_ref())?;
    let cancel = services.cancellation();
    let target = &request.target;
    let latest_owner = super::provider::open_native(
        target.location.clone(),
        None,
        crate::cache_service::snapshot::LoadRequirement::Maintenance,
        &state,
    )
    .await?;
    let table = latest_owner.table.clone();
    let latest = table
        .version()
        .ok_or_else(|| invalid("a collected table has a version"))?;
    let contract = super::contract::DeclaredCheck::open(&table, &state)?;
    contract.verify(&table)?;
    let mut retained = super::maintenance::EffectiveRetention::new(services.pool());
    for row in &request.retained {
        cancel.checkpoint().map_err(external)?;
        if row.from_version > row.through_version {
            return Err(invalid("retention interval is reversed"));
        }
        let covers = match row.reason {
            RetentionReason::Attempt => {
                target.location.as_str().starts_with(row.table_uri.as_str())
            }
            RetentionReason::Publication | RetentionReason::Changes => {
                same_table(&row.table_uri, &target.location)
            }
        };
        if !covers {
            continue;
        }
        retained.changes |= row.reason == RetentionReason::Changes;
        retained.attempts |= row.reason == RetentionReason::Attempt;
        // Ranges beyond the table's history (an attempt's whole history) keep what exists.
        let start = super::provider::delta_version(row.from_version)?;
        if start > latest {
            continue;
        }
        let end = super::provider::delta_version(row.through_version)?.min(latest);
        retained.protect(start, end)?;
    }
    retained.expand(cancel)?;
    // Materialize every protected version before destructive work; missing history is
    // a refusal, including versions older than a retained checkpoint boundary.
    for version in &retained.versions {
        let selected = super::provider::open_native(
            target.location.clone(),
            Some(*version),
            crate::cache_service::snapshot::LoadRequirement::Maintenance,
            &state,
        )
        .await?;
        contract.verify(&selected.table)?;
    }
    cancel.checkpoint().map_err(external)?;
    services.require_settlement();
    let table = super::maintenance::maintenance_fence(table, &state).await?;
    let fence_version = table
        .version()
        .ok_or_else(|| invalid("maintenance fence has no version"))?;
    super::maintenance::apply_maintenance(
        &target.location,
        target.action == CollectAction::Optimize,
        target.log_cutoff_ms,
        &state,
        table,
        &contract,
        retained,
    )
    .await
    .map_err(|source| super::settlement::maintenance_interrupted(fence_version, source))
}

/// Whether a recorded table URI names this table.
fn same_table(uri: &str, table: &url::Url) -> bool {
    let normalize = |text: &str| text.trim_end_matches('/').to_owned();
    if normalize(uri) == normalize(table.as_str()) {
        return true;
    }
    // Local aliases of one directory are one table.
    match (url::Url::parse(uri), table.to_file_path()) {
        (Ok(recorded), Ok(path)) if recorded.scheme() == "file" => recorded
            .to_file_path()
            .ok()
            .and_then(|recorded| recorded.canonicalize().ok())
            .zip(path.canonicalize().ok())
            .is_some_and(|(left, right)| left == right),
        _ => false,
    }
}

/// Remove every object of each table: a retired publication's tables the catalog
/// planned. Idempotent — a missing or partially removed table removes what remains —
/// and every cached snapshot and file-metadata entry is invalidated. Returns the objects
/// removed.
/// # Errors
/// An unregistered store or a failed listing or deletion (a rerun resumes).
pub async fn remove_tables(tables: &[url::Url], state: &SessionState) -> Result<u64> {
    let mut removed = 0;
    for table in tables {
        removed += remove_prefix(table, state).await?;
    }
    Ok(removed)
}

/// Remove every object under a prefix: the member prefix of a reclaimed intent.
/// Idempotent, and invalidates every cached snapshot and file-metadata entry.
/// # Errors
/// An unregistered store or a failed listing or deletion (a rerun resumes).
pub async fn remove_prefix(prefix: &url::Url, state: &SessionState) -> Result<u64> {
    let store = state
        .runtime_env()
        .object_store_registry
        .get_store(prefix)?;
    let path = object_store::path::Path::from_url_path(prefix.path()).map_err(external)?;
    let locations = store.list(Some(&path)).map_ok(|meta| meta.location).boxed();
    let mut removed = 0;
    let mut deleted = store.delete_stream(locations);
    let result = async {
        while let Some(outcome) = deleted.next().await {
            match outcome {
                Ok(_) => removed += 1,
                // Another remover (or an earlier interrupted run) got there first.
                Err(object_store::Error::NotFound { .. }) => {}
                Err(error) => return Err(external(error)),
            }
        }
        Ok(())
    }
    .await;
    // Even a partial removal invalidates: no cached entry may outlive its files.
    if let Some(service) = state
        .config()
        .get_extension::<crate::cache_service::DeltaCacheService>()
    {
        service.native().invalidate();
    }
    result.map(|()| removed)
}

fn effects() -> BTreeSet<OperationEffect> {
    [
        OperationEffect::Read,
        OperationEffect::Write,
        OperationEffect::Publish,
    ]
    .into_iter()
    .collect()
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}
