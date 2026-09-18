// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Coordinated local maintenance, driven by a typed native retention query. Delta
//! owns optimization, checkpointing, data-file vacuum and transaction-log cleanup.
use super::publication::PublicationRoot;
use crate::{
    CatalogError,
    session::{PreparedComputation, RelationPlan, SnapshotSession},
};
use datafusion::{
    catalog::Session,
    common::{DFSchema, DFSchemaRef, DataFusionError, ResolvedTableReference, Result},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{
        Expr, Extension, LogicalPlan, UserDefinedLogicalNode, UserDefinedLogicalNodeCore,
        physical_planning_context::PhysicalPlanningContext,
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
    delta_datafusion::SessionFallbackPolicy, kernel::transaction::CommitProperties,
    operations::vacuum::VacuumMode, protocol::checkpoints,
};
use futures_util::TryStreamExt;
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::generated::{
    enums::RetentionReason,
    runtime::{maintenance_outcomes, retained_versions},
};
use pse_schema::model::provider::{OperationEffect, OperationPurpose, ProviderScope};
use std::{
    collections::BTreeSet,
    hash::{Hash, Hasher},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

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

impl SnapshotSession {
    /// Prepare a lazy native maintenance command. Use detached retained-version
    /// facts in an administrative session: an active publication reader deliberately
    /// excludes destructive maintenance, including readers owned by this caller.
    /// # Errors
    /// Undeclared retention fields, foreign source, target policy or planning failure.
    pub fn prepare_maintenance(
        &self,
        target: MaintenanceTarget,
        retention: &RelationPlan,
        cancel: &CancellationToken,
    ) -> std::result::Result<PreparedComputation, CatalogError> {
        if retention.relation_id() != retained_versions::RELATION_ID {
            return Err(crate::session::engine(invalid(
                "maintenance requires declared retained_versions",
            )));
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| crate::session::engine(external(error)))?;
        let now = i64::try_from(now.as_millis())
            .map_err(|error| crate::session::engine(external(error)))?;
        if target.log_cutoff_ms < 0 || target.log_cutoff_ms > now {
            return Err(crate::session::engine(invalid(
                "log cleanup cutoff must be between the Unix epoch and now",
            )));
        }
        let mut session = self.with_purpose(OperationPurpose::Publish);
        session.bindings.target(ProviderScope::Table(
            target.reference.catalog.to_string(),
            target.reference.schema.to_string(),
            target.reference.table.to_string(),
        ));
        let schema = maintenance_outcomes::schema().map_err(CatalogError::from)?;
        let plan = LogicalPlan::Extension(Extension {
            node: Arc::new(Maintenance {
                request: Arc::new(Request {
                    target,
                    started: AtomicBool::new(false),
                }),
                input: retention.plan().clone(),
                schema: Arc::new(DFSchema::try_from(schema).map_err(crate::session::engine)?),
            }),
        });
        session.prepare(
            crate::session::contract::ExecutionContract::plan(plan, None, effects()),
            cancel,
        )
    }
}

#[derive(Debug)]
struct Request {
    target: MaintenanceTarget,
    started: AtomicBool,
}
#[derive(Clone)]
struct Maintenance {
    request: Arc<Request>,
    input: LogicalPlan,
    schema: DFSchemaRef,
}
impl PartialEq for Maintenance {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.request, &other.request) && self.input == other.input
    }
}
impl Eq for Maintenance {}
impl Hash for Maintenance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.request).hash(state);
        self.input.hash(state);
    }
}
impl PartialOrd for Maintenance {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match Arc::as_ptr(&self.request).cmp(&Arc::as_ptr(&other.request)) {
            std::cmp::Ordering::Equal => self.input.partial_cmp(&other.input),
            order => Some(order),
        }
    }
}
// Native plan renderers visit children separately. Debug describes this
// node without recursively duplicating complete subgraphs in JSON.
impl std::fmt::Debug for Maintenance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UserDefinedLogicalNodeCore::fmt_for_explain(self, f)
    }
}
impl UserDefinedLogicalNodeCore for Maintenance {
    fn name(&self) -> &'static str {
        "DeltaMaintenance"
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
        write!(f, "DeltaMaintenance: {}", self.request.target.location)
    }
    fn with_exprs_and_inputs(&self, exprs: Vec<Expr>, inputs: Vec<LogicalPlan>) -> Result<Self> {
        if !exprs.is_empty() || inputs.len() != 1 {
            return Err(invalid("maintenance requires one real retention child"));
        }
        let input = inputs
            .into_iter()
            .next()
            .ok_or_else(|| invalid("retention child absent"))?;
        Ok(Self {
            input,
            ..self.clone()
        })
    }
    fn prevent_predicate_push_down_columns(&self) -> std::collections::HashSet<String> {
        self.schema
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .collect()
    }
}

#[derive(Debug)]
pub(crate) struct MaintenancePlanner;
#[async_trait::async_trait]
impl ExtensionPlanner for MaintenancePlanner {
    async fn plan_extension(
        &self,
        _: &dyn PhysicalPlanner,
        node: &dyn UserDefinedLogicalNode,
        _: &[&LogicalPlan],
        inputs: &[Arc<dyn ExecutionPlan>],
        session: &dyn Session,
        _: &PhysicalPlanningContext,
    ) -> Result<Option<Arc<dyn ExecutionPlan>>> {
        let Some(node) = node.as_any().downcast_ref::<Maintenance>() else {
            return Ok(None);
        };
        let services = crate::session::execution::NativeExecutionContext::from_session(session)?;
        services.admit_effects(&effects()).map_err(external)?;
        let state = session
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| invalid("maintenance requires actual caller state"))?;
        let [input] = inputs else {
            return Err(invalid("maintenance physical child absent"));
        };
        let declared = pse_schema::arrow::relation_schema(
            services.registry(),
            retained_versions::spec(services.registry()).map_err(external)?,
        )
        .map_err(external)?;
        let input = super::layout::declared_output(Arc::clone(input), &Arc::new(declared))?;
        Ok(Some(Arc::new(MaintenanceExec {
            request: Arc::clone(&node.request),
            input,
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
struct MaintenanceExec {
    request: Arc<Request>,
    input: Arc<dyn ExecutionPlan>,
    state: Arc<SessionState>,
    properties: Arc<PlanProperties>,
}
impl DisplayAs for MaintenanceExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DeltaMaintenanceExec: exclusive local coordination, native retained versions")
    }
}
impl ExecutionPlan for MaintenanceExec {
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(
            &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
        ) -> Result<datafusion::common::tree_node::TreeNodeRecursion>,
    ) -> Result<datafusion::common::tree_node::TreeNodeRecursion> {
        Ok(datafusion::common::tree_node::TreeNodeRecursion::Continue)
    }
    fn name(&self) -> &'static str {
        "DeltaMaintenanceExec"
    }
    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }
    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        vec![&self.input]
    }
    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if children.len() != 1 {
            return Err(invalid("maintenance requires one physical child"));
        }
        Ok(Arc::new(Self {
            input: children
                .into_iter()
                .next()
                .ok_or_else(|| invalid("retention child absent"))?,
            request: Arc::clone(&self.request),
            state: Arc::clone(&self.state),
            properties: Arc::clone(&self.properties),
        }))
    }
    fn execute(&self, partition: usize, _: Arc<TaskContext>) -> Result<SendableRecordBatchStream> {
        if partition != 0 || self.request.started.swap(true, Ordering::AcqRel) {
            return Err(invalid("maintenance executes once in partition zero"));
        }
        let request = Arc::clone(&self.request);
        let input = Arc::clone(&self.input);
        let state = Arc::clone(&self.state);
        let stream =
            futures_util::stream::once(
                async move { maintain(&request.target, input, state).await },
            );
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            self.schema(),
            stream,
        )))
    }
}

async fn maintain(
    target: &MaintenanceTarget,
    input: Arc<dyn ExecutionPlan>,
    state: Arc<SessionState>,
) -> Result<datafusion::arrow::array::RecordBatch> {
    let services = crate::session::execution::NativeExecutionContext::from_session(state.as_ref())?;
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
    let versions = selected_versions(target, &current, &table_path, &control_path)?;
    let mut retained = Retention {
        versions,
        changes: false,
        attempts: false,
        reservation: services.reserver().open("delta:retained-versions"),
    };
    retained
        .reservation
        .try_grow(retained.versions.len() * 64)
        .map_err(external)?;
    collect_versions(input, &state, &table_path, &mut retained).await?;
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
    // A crash after this point still changes the uncached log generation observed
    // by a subsequent cache consumer in any cooperating process.
    if table_path != control_path {
        maintenance_fence(latest, &state).await?;
    }
    let table = maintenance_fence(table, &state).await?;
    apply_maintenance(target, &state, table, &contract, retained, reclamation).await
}

async fn maintenance_fence(
    table: deltalake::DeltaTable,
    state: &SessionState,
) -> Result<deltalake::DeltaTable> {
    let commit = CommitProperties::default()
        .with_max_retries(0)
        .with_create_checkpoint(false)
        .with_cleanup_expired_logs(Some(false))
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
        .map_err(external);
    // Even a lost response may have committed. Invalidate on both outcomes.
    if let Some(service) = state
        .config()
        .get_extension::<crate::cache_service::NativeCacheService>()
    {
        service.invalidate();
    }
    fenced
}

struct Retention {
    versions: BTreeSet<u64>,
    changes: bool,
    attempts: bool,
    reservation: Box<dyn pse_ids::Reservation>,
}

async fn collect_versions(
    input: Arc<dyn ExecutionPlan>,
    state: &SessionState,
    table_path: &std::path::Path,
    retained: &mut Retention,
) -> Result<()> {
    let services = crate::session::execution::NativeExecutionContext::from_session(state)?;
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
            for version in row.from_version..=row.through_version {
                cancel.checkpoint().map_err(external)?;
                let version = super::provider::delta_version(version)?;
                if !retained.versions.contains(&version) {
                    retained.reservation.try_grow(64).map_err(external)?;
                    retained.versions.insert(version);
                }
            }
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
) -> Result<BTreeSet<u64>> {
    let mut versions = BTreeSet::new();
    if table_path == control_path {
        versions.insert(super::provider::delta_version(target.head.version)?);
    }
    for member in &current.members {
        if same_location(&member.table_uri, table_path)? {
            versions.insert(super::provider::delta_version(member.delta_version)?);
        }
    }
    for member in &current.inputs {
        if same_location(&member.table_uri, table_path)? {
            versions.insert(super::provider::delta_version(member.delta_version)?);
        }
    }
    Ok(versions)
}

async fn apply_maintenance(
    target: &MaintenanceTarget,
    state: &SessionState,
    mut table: deltalake::DeltaTable,
    contract: &super::contract::DeclaredCheck,
    retained: Retention,
    reclamation: Option<(
        CommitProperties,
        datafusion::execution::memory_pool::MemoryReservation,
    )>,
) -> Result<datafusion::arrow::array::RecordBatch> {
    let commit = || {
        CommitProperties::default()
            .with_create_checkpoint(false)
            .with_cleanup_expired_logs(Some(false))
    };
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
        let caller = Arc::new(contract.bind(state)?);
        table = table
            .delete()
            .with_session_state(caller)
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .with_commit_properties(reclamation)
            .await
            .map_err(external)?
            .0;
    }
    if target.action == MaintenanceAction::Optimize {
        let caller = Arc::new(contract.bind(state)?);
        table = table
            .optimize()
            .with_session_state(caller)
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .with_commit_properties(commit())
            .await
            .map_err(external)?
            .0;
    }
    checkpoints::create_checkpoint(&table, None)
        .await
        .map_err(external)?;
    let keep = retained.versions.iter().copied().collect::<Vec<_>>();
    // Full vacuum treats CDF files as ordinary unreferenced files. Lite only walks
    // Remove tombstones, leaving AddCDCFile payloads intact for retained CDF windows.
    let mode = if retained.changes {
        VacuumMode::Lite
    } else {
        VacuumMode::Full
    };
    let (table, metrics) = table
        .vacuum()
        .with_keep_versions(&keep)
        .with_mode(mode)
        .with_commit_properties(commit())
        .await
        .map_err(external)?;
    let version = table
        .version()
        .ok_or_else(|| invalid("maintained table has no version"))?;
    let minimum = retained.versions.first().copied().unwrap_or(version);
    // Live member attempts require complete commit evidence. Native control-table
    // transaction identities survive checkpoints, so retired control logs can go.
    let deleted_logs = if retained.attempts {
        0
    } else {
        checkpoints::cleanup_expired_logs_for(
            minimum,
            table.log_store().as_ref(),
            target.log_cutoff_ms,
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
            deleted_logs: i64::try_from(deleted_logs).map_err(external)?,
        })
        .map_err(external)?;
    Ok(output.finish().map_err(external)?.into_batch())
}

fn same_location(uri: &str, target: &std::path::Path) -> Result<bool> {
    let location = url::Url::parse(uri).map_err(external)?;
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
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
