// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed retention and native Delta maintenance under engine command settlement.
use super::publication::PublicationRoot;
use datafusion::{
    common::{DataFusionError, ResolvedTableReference, Result},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{Expr, LogicalPlan},
    physical_plan::{ExecutionPlan, SendableRecordBatchStream, execute_stream},
};
use deltalake::{
    kernel::transaction::CommitProperties, operations::vacuum::VacuumMode, protocol::checkpoints,
};
use futures_util::TryStreamExt;
use pse_columnar::CancellationToken;
use pse_engine::{
    EngineError,
    operation::{Body, Definition, Family, Operation},
    session::{EngineSession, PreparedComputation, RelationPlan},
};
use pse_ids::SemanticId;
use pse_relations::generated::{
    enums::RetentionReason,
    runtime::{maintenance_outcomes, retained_versions},
};
use pse_schema::model::provider::{OperationEffect, OperationPurpose, ProviderScope};
use std::{collections::BTreeSet, sync::Arc};
/// Explicit current publication and declared target. Historical roots, attempts,
/// outputs and CDF consumers supply additional ranges through the real query input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaintenanceAction {
    /// Checkpoint and remove expired unreferenced native data/log files.
    Collect,
    /// Compact before native checkpoint and collection.
    Optimize,
    /// Retire an unreferenced member attempt belonging to this control root.
    /// Native DELETE tombstones its data; VACUUM honors declared expiration.
    ReclaimUnpublished,
}

/// Declared target and native lifecycle operation under reader exclusion.
#[derive(Clone, Debug)]
pub struct MaintenanceTarget {
    /// The expected current control root. Its members are protected automatically.
    pub head: PublicationRoot,
    /// Qualified name governing operation policy.
    pub reference: ResolvedTableReference,
    /// Native Delta table to maintain.
    pub location: url::Url,
    /// Exact relation declaration to verify before any effects.
    pub relation_id: SemanticId,
    /// Native lifecycle operation.
    pub action: MaintenanceAction,
    /// Millisecond cutoff for native log cleanup. Table data retention remains the
    /// declared Delta retention duration; no option disables that safety check.
    pub log_cutoff_ms: i64,
}

/// Prepare a lazy native maintenance command. Use detached retained-version
/// facts in an administrative session: an active publication reader deliberately
/// excludes destructive maintenance, including readers owned by this caller.
/// # Errors
/// Undeclared retention fields, foreign source, target policy or planning failure.
pub fn prepare_maintenance(
    source_session: &EngineSession,
    target: MaintenanceTarget,
    retention: &RelationPlan,
    cancel: &CancellationToken,
) -> std::result::Result<PreparedComputation, EngineError> {
    if retention.relation_id() != retained_versions::RELATION_ID {
        return Err(pse_engine::session::engine(invalid(
            "maintenance requires declared retained_versions",
        )));
    }
    let now = now_ms().map_err(pse_engine::session::engine)?;
    if target.log_cutoff_ms < 0 || target.log_cutoff_ms > now {
        return Err(pse_engine::session::engine(invalid(
            "log cleanup cutoff must be between the Unix epoch and now",
        )));
    }
    let mut session = source_session.with_purpose(OperationPurpose::Publish);
    session.bind_target(ProviderScope::Table(
        target.reference.catalog.to_string(),
        target.reference.schema.to_string(),
        target.reference.table.to_string(),
    ));
    let schema = maintenance_outcomes::schema().map_err(EngineError::from)?;
    let plan = Operation::plan(
        Arc::new(Request { target, schema }),
        vec![retention.plan().clone()],
    )
    .map_err(pse_engine::session::engine)?;
    session.prepare(
        pse_engine::session::contract::ExecutionContract::plan(plan, None, effects()),
        cancel,
    )
}
#[derive(Debug)]
struct Request {
    target: MaintenanceTarget,
    schema: datafusion::arrow::datatypes::SchemaRef,
}
#[async_trait::async_trait]
impl Definition for Request {
    fn name(&self) -> &'static str {
        "DeltaMaintenance"
    }
    fn schema(&self) -> datafusion::arrow::datatypes::SchemaRef {
        self.schema.clone()
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
        Ok(Arc::new(MaintenanceBody {
            request: self,
            state: Arc::new(state.clone()),
        }))
    }
}
#[derive(Debug)]
struct MaintenanceBody {
    request: Arc<Request>,
    state: Arc<SessionState>,
}
impl Body for MaintenanceBody {
    fn execute(
        &self,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
        _: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let [input]: [_; 1] = inputs
            .try_into()
            .map_err(|_| invalid("maintenance requires one retention child"))?;
        let state = self.state.clone();
        let request = self.request.clone();
        let services =
            pse_engine::session::execution::NativeExecutionContext::from_session(state.as_ref())?;
        let declared = pse_schema::arrow::relation_schema(
            services.registry(),
            retained_versions::spec(services.registry()).map_err(external)?,
        )
        .map_err(external)?;
        let input = super::layout::declared_output(input, &Arc::new(declared))?;
        Ok(pse_engine::operation::batch(
            request.schema.clone(),
            async move { maintain(&request.target, input, state).await },
        ))
    }
}
#[tracing::instrument(name = "pse.delta.maintain", skip_all, fields(action = ?target.action, delta_version = tracing::field::Empty, files_deleted = tracing::field::Empty), err)]
async fn maintain(
    target: &MaintenanceTarget,
    input: Arc<dyn ExecutionPlan>,
    state: Arc<SessionState>,
) -> Result<datafusion::arrow::array::RecordBatch> {
    let services =
        pse_engine::session::execution::NativeExecutionContext::from_session(state.as_ref())?;
    let cancel = services.cancellation();
    // Acquire control first, matching publication readers and writers. Both paths
    // are canonicalized so aliases cannot acquire an independent exclusion token.
    let control = super::lease::maintenance(&target.head.location, cancel)?;
    let control_path = local_path(&target.head.location)?;
    let table_path = local_path(&target.location)?;
    let _table = if table_path == control_path {
        None
    } else {
        Some(super::lease::maintenance(&target.location, cancel)?)
    };
    let latest_owner = super::provider::open_native(
        target.head.location.clone(),
        None,
        crate::cache_service::snapshot::LoadRequirement::Maintenance,
        &state,
    )
    .await?;
    let latest = latest_owner.table.clone();
    let latest_version = latest
        .version()
        .ok_or_else(|| invalid("publication control has no version"))?;
    let current_root = PublicationRoot {
        location: target.head.location.clone(),
        version: super::provider::signed_version(latest_version)?,
    };
    let current = super::publication::read_maintained_record(
        &current_root,
        services.registry(),
        Arc::clone(&state),
        &control,
    )
    .await?;
    let expected = super::publication::read_maintained_record(
        &target.head,
        services.registry(),
        Arc::clone(&state),
        &control,
    )
    .await?;
    if current != expected {
        return Err(invalid("maintenance publication head changed"));
    }
    let mut retained = selected_versions(
        target,
        &current,
        &table_path,
        &control_path,
        services.pool(),
    )?;
    collect_versions(input, &state, &table_path, &mut retained).await?;
    retained.expand(cancel)?;
    let table_owner = super::provider::open_native(
        target.location.clone(),
        None,
        crate::cache_service::snapshot::LoadRequirement::Maintenance,
        &state,
    )
    .await?;
    let table = table_owner.table.clone();
    let contract = super::contract::DeclaredCheck::new(services.registry(), target.relation_id)?;
    contract.verify(&table)?;
    let reclamation = if target.action == MaintenanceAction::ReclaimUnpublished {
        if table_path == control_path || !retained.versions.is_empty() {
            return Err(invalid(
                "referenced publications, outputs, attempts or CDF ranges cannot be reclaimed",
            ));
        }
        Some(super::attempt::admit_reclamation(&table, &target.head.location, &state).await?)
    } else {
        None
    };
    // Materialize every protected version before destructive work; missing history
    // is a refusal, including versions older than a retained checkpoint boundary.
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
    // Commit native coordination evidence on every root before deleting anything.
    // The exclusive leases already persisted new retention generations. This
    // native transaction additionally records the maintenance operation itself.
    if table_path != control_path {
        maintenance_fence(latest, &state).await?;
    }
    let table = maintenance_fence(table, &state).await?;
    let fence_version = table
        .version()
        .ok_or_else(|| invalid("maintenance fence has no version"))?;
    apply_maintenance(target, &state, table, &contract, retained, reclamation)
        .await
        .map_err(|source| super::settlement::maintenance_interrupted(fence_version, source))
}

async fn maintenance_fence(
    table: deltalake::DeltaTable,
    state: &SessionState,
) -> Result<deltalake::DeltaTable> {
    let commit = super::operation::commit_policy(
        CommitProperties::default(),
        super::operation::CommitKind::Maintenance,
    )
    .with_metadata([(
        "pse.maintenance_fence".into(),
        serde_json::json!(uuid::Uuid::now_v7().to_string()),
    )]);
    // Native metadata commit with unchanged properties: no application state store
    // and no reconstruction of Delta's transaction or replay algorithms.
    let fenced = table
        .set_tbl_properties()
        .with_properties(std::collections::HashMap::new())
        .with_commit_properties(commit)
        .await
        .map_err(super::settlement::unresolved);
    // Even a lost response may have committed. Invalidate on both outcomes.
    if let Some(service) = state
        .config()
        .get_extension::<crate::cache_service::DeltaCacheService>()
    {
        service.native().invalidate();
    }
    fenced
}

struct EffectiveRetention {
    ranges: Vec<(u64, u64)>,
    versions: BTreeSet<u64>,
    changes: bool,
    attempts: bool,
    reservation: pse_columnar::MemoryReservation,
}

impl EffectiveRetention {
    fn protect(&mut self, start: u64, end: u64) -> Result<()> {
        self.reservation.try_grow(32).map_err(external)?;
        self.ranges.push((start, end));
        Ok(())
    }
    // Keep compact intervals through admission, then expand once for Delta keep_versions.
    fn expand(&mut self, cancel: &CancellationToken) -> Result<()> {
        self.ranges.sort_unstable();
        let mut through = None;
        for (start, end) in &self.ranges {
            let start = through.map_or(*start, |previous: u64| {
                (*start).max(previous.saturating_add(1))
            });
            if through == Some(u64::MAX) {
                break;
            }
            for version in start..=*end {
                cancel.checkpoint().map_err(external)?;
                if !self.versions.contains(&version) {
                    self.reservation.try_grow(64).map_err(external)?;
                    self.versions.insert(version);
                }
            }
            through = Some(through.map_or(*end, |previous| previous.max(*end)));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct MaintenancePolicy {
    data_age: chrono::TimeDelta,
    log_cutoff_ms: i64,
    mode: VacuumMode,
    retain_attempt_logs: bool,
}
impl MaintenancePolicy {
    fn resolve(
        data_age: std::time::Duration,
        log_age: std::time::Duration,
        requested_cutoff: i64,
        now: i64,
        changes: bool,
        attempts: bool,
    ) -> Result<Self> {
        if requested_cutoff < 0 || requested_cutoff > now {
            return Err(invalid("invalid log cleanup cutoff"));
        }
        let rounded_age = log_age
            .as_millis()
            .checked_add(u128::from(
                !log_age.subsec_nanos().is_multiple_of(1_000_000),
            ))
            .ok_or_else(|| invalid("log retention age overflows"))?;
        let log_age = i64::try_from(rounded_age)
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        let floor = now.checked_sub(log_age).unwrap_or(0).max(0);
        Ok(Self {
            data_age: chrono::TimeDelta::from_std(data_age)
                .map_err(|error| DataFusionError::External(Box::new(error)))?,
            log_cutoff_ms: requested_cutoff.min(floor),
            mode: if changes {
                VacuumMode::Lite
            } else {
                VacuumMode::Full
            },
            retain_attempt_logs: attempts,
        })
    }
}
fn now_ms() -> Result<i64> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| DataFusionError::External(Box::new(error)))?;
    i64::try_from(now.as_millis()).map_err(|error| DataFusionError::External(Box::new(error)))
}

async fn collect_versions(
    input: Arc<dyn ExecutionPlan>,
    state: &SessionState,
    table_path: &std::path::Path,
    retained: &mut EffectiveRetention,
) -> Result<()> {
    let services = pse_engine::session::execution::NativeExecutionContext::from_session(state)?;
    let cancel = services.cancellation();
    // Detached native query output can protect additional published/attempt/output
    // versions and complete CDF intervals, including update preimages.
    let mut rows = execute_stream(input, state.task_ctx())?;
    while let Some(batch) = rows.try_next().await? {
        cancel.checkpoint().map_err(external)?;
        let view =
            retained_versions::View::try_from_batch_with_registry(services.registry(), &batch)
                .map_err(|error| invalid(&format!("retention relation admission: {error:?}")))?;
        for index in 0..batch.num_rows() {
            let row = view.row(index).map_err(external)?;
            if row.from_version > row.through_version {
                return Err(invalid("retention interval is reversed"));
            }
            if !same_location(&row.table_uri, table_path)? {
                continue;
            }
            retained.changes |= row.reason == RetentionReason::Changes;
            retained.attempts |= row.reason == RetentionReason::Attempt;
            let start = super::provider::delta_version(row.from_version)?;
            let end = super::provider::delta_version(row.through_version)?;
            retained.protect(start, end)?;
        }
    }
    Ok(())
}

fn local_path(location: &url::Url) -> Result<std::path::PathBuf> {
    location
        .to_file_path()
        .map_err(|()| invalid("remote maintenance coordination is unqualified"))?
        .canonicalize()
        .map_err(external)
}

fn selected_versions(
    target: &MaintenanceTarget,
    current: &pse_relations::generated::runtime::publications::Row,
    table_path: &std::path::Path,
    control_path: &std::path::Path,
    pool: &Arc<dyn pse_columnar::MemoryPool>,
) -> Result<EffectiveRetention> {
    let mut retained = EffectiveRetention {
        ranges: Vec::new(),
        versions: BTreeSet::new(),
        changes: false,
        attempts: false,
        reservation: pse_columnar::MemoryConsumer::new("delta:retained-versions").register(pool),
    };
    if table_path == control_path {
        let version = super::provider::delta_version(target.head.version)?;
        retained.protect(version, version)?;
    }
    for member in &current.members {
        if same_location(&member.table_uri, table_path)? {
            let version = super::provider::delta_version(member.delta_version)?;
            retained.protect(version, version)?;
        }
    }
    for member in &current.inputs {
        if same_location(&member.table_uri, table_path)? {
            let version = super::provider::delta_version(member.delta_version)?;
            retained.protect(version, version)?;
        }
    }
    Ok(retained)
}

async fn apply_maintenance(
    target: &MaintenanceTarget,
    state: &SessionState,
    mut table: deltalake::DeltaTable,
    contract: &super::contract::DeclaredCheck,
    retained: EffectiveRetention,
    reclamation: Option<(
        CommitProperties,
        datafusion::execution::memory_pool::MemoryReservation,
    )>,
) -> Result<datafusion::arrow::array::RecordBatch> {
    use deltalake::table::config::TablePropertiesExt;
    let properties = table.snapshot().map_err(external)?.table_config();
    let policy = MaintenancePolicy::resolve(
        properties.deleted_file_retention_duration(),
        properties.log_retention_duration(),
        target.log_cutoff_ms,
        now_ms()?,
        retained.changes,
        retained.attempts,
    )?;
    let caller = Arc::new(state.clone());
    let context = super::operation::DeltaOperationContext::new(
        caller.clone(),
        Some(contract.clone()),
        CommitProperties::default(),
        super::operation::CommitKind::Maintenance,
    )?;
    if let Some((reclamation, _receipt_owner)) = reclamation {
        // DELETE may be a no-op for an empty/provisioned attempt. A native metadata
        // transaction first records retirement without changing any declaration,
        // so an interrupted cleanup can never let the old member writer resume.
        table = table
            .set_tbl_properties()
            .with_properties(std::collections::HashMap::new())
            .with_commit_properties(reclamation.clone())
            .await
            .map_err(external)?;
        let reclaim = context.with_commit(reclamation, super::operation::CommitKind::Maintenance);
        table = reclaim.delete(table).await.map_err(external)?.0;
    }
    if target.action == MaintenanceAction::Optimize {
        table = context.optimize(table).await.map_err(external)?.0;
    }
    checkpoints::create_checkpoint(&table, None)
        .await
        .map_err(external)?;
    let keep = retained.versions.iter().copied().collect::<Vec<_>>();
    let (table, metrics) = context
        .vacuum(table)
        .with_keep_versions(&keep)
        .with_retention_period(policy.data_age)
        .with_mode(policy.mode)
        .await
        .map_err(external)?;
    let version = table
        .version()
        .ok_or_else(|| invalid("maintained table has no version"))?;
    tracing::Span::current().record("delta_version", version);
    tracing::Span::current().record("files_deleted", metrics.files_deleted.len());
    let minimum = retained.versions.first().copied().unwrap_or(version);
    // Live member attempts require complete commit evidence. Native control-table
    // transaction identities survive checkpoints, so retired control logs can go.
    let deleted_logs = if policy.retain_attempt_logs {
        0
    } else {
        checkpoints::cleanup_expired_logs_for(
            minimum,
            table.log_store().as_ref(),
            policy.log_cutoff_ms,
            None,
        )
        .await
        .map_err(external)?
    };
    drop(retained);
    let mut output = maintenance_outcomes::Builder::new().map_err(external)?;
    output
        .push(maintenance_outcomes::Row {
            table_uri: target.location.to_string(),
            delta_version: super::provider::signed_version(version)?,
            deleted_files: metrics.files_deleted,
            deleted_logs: i64::try_from(deleted_logs)
                .map_err(|error| DataFusionError::External(Box::new(error)))?,
        })
        .map_err(external)?;
    Ok(output.finish().map_err(external)?.into_batch())
}

fn same_location(uri: &str, target: &std::path::Path) -> Result<bool> {
    let location =
        url::Url::parse(uri).map_err(|error| DataFusionError::External(Box::new(error)))?;
    if location.scheme() != "file" {
        return Ok(false);
    }
    Ok(location
        .to_file_path()
        .map_err(|()| invalid("invalid local retention URI"))?
        .canonicalize()
        .map_err(external)?
        == target)
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

#[cfg(test)]
mod delta_boundary_unit {
    use super::*;
    use std::time::Duration;
    #[test]
    fn native_retention_floor_changes_and_attempts_form_one_effective_policy() {
        for changes in [false, true] {
            for attempts in [false, true] {
                let policy = MaintenancePolicy::resolve(
                    Duration::from_secs(10),
                    Duration::from_millis(30),
                    99,
                    100,
                    changes,
                    attempts,
                )
                .unwrap();
                assert_eq!(policy.log_cutoff_ms, 70);
                assert_eq!(policy.data_age.num_seconds(), 10);
                assert_eq!(
                    policy.mode,
                    if changes {
                        VacuumMode::Lite
                    } else {
                        VacuumMode::Full
                    }
                );
                assert_eq!(policy.retain_attempt_logs, attempts);
            }
        }
        assert_eq!(
            MaintenancePolicy::resolve(
                Duration::ZERO,
                Duration::from_secs(1),
                20,
                100,
                false,
                false
            )
            .unwrap()
            .log_cutoff_ms,
            0
        );
        for cutoff in [-1, 101] {
            assert!(
                MaintenancePolicy::resolve(
                    Duration::ZERO,
                    Duration::ZERO,
                    cutoff,
                    100,
                    false,
                    false
                )
                .is_err()
            );
        }
        assert!(
            MaintenancePolicy::resolve(Duration::MAX, Duration::ZERO, 0, 100, false, false)
                .is_err()
        );
        assert!(
            MaintenancePolicy::resolve(Duration::ZERO, Duration::MAX, 0, 100, false, false)
                .is_err()
        );
    }
    #[test]
    fn overlapping_protection_ranges_expand_once_under_a_finite_pool() {
        use datafusion::execution::memory_pool::{GreedyMemoryPool, MemoryPool};
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(4096));
        let mut retained = EffectiveRetention {
            ranges: vec![(5, 7), (1, 3), (2, 6)],
            versions: BTreeSet::new(),
            changes: true,
            attempts: true,
            reservation: pse_columnar::MemoryConsumer::new("unit").register(&pool),
        };
        retained.expand(&CancellationToken::new()).unwrap();
        assert_eq!(retained.versions, (1..=7).collect());
        assert_eq!(pool.reserved(), 7 * 64);
        drop(retained);
        assert_eq!(pool.reserved(), 0);
    }
}
