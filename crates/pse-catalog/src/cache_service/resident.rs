// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Lazy decoded exact selections, owned by the same native cache service as files
//! and snapshots. Idle entries contain only Arrow values and allocation owners.
use super::NativeCacheService;
use crate::session::cache::Cached;
use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::{Session, TableProvider},
    common::{DataFusionError, Result, TableReference},
    execution::{
        TaskContext,
        cache::{Cache, CacheKey, CacheValue, default_cache::DefaultCache},
        session_state::SessionState,
    },
    logical_expr::{Expr, TableType},
    physical_expr::EquivalenceProperties,
    physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PlanProperties,
        SendableRecordBatchStream,
        execution_plan::{Boundedness, EmissionType},
        metrics::ExecutionPlanMetricsSet,
        stream::RecordBatchStreamAdapter,
    },
};
use futures_util::TryStreamExt;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

/// Actual frozen engine and policy assembly, retained through native scopes.
#[derive(Debug, Clone)]
pub(crate) struct CacheIdentity {
    pub(crate) generation: pse_ids::SemanticId,
    pub(crate) policies: Arc<Vec<pse_schema::model::provider::ProviderPolicy>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Key {
    store: usize,
    latest: u64,
    selection: String,
    schema: SchemaRef,
    settings: pse_ids::ContentHash,
    implementation: pse_ids::SemanticId,
    policies: String,
}
impl CacheKey for Key {
    fn size(&self) -> usize {
        512 + self.selection.len() * 2
            + self.policies.len() * 2
            + self.schema.fields().size()
            + self
                .schema
                .metadata()
                .iter()
                .map(|(key, value)| key.capacity() + value.capacity() + 64)
                .sum::<usize>()
    }
    fn table_ref(&self) -> Option<&TableReference> {
        None
    }
}
#[derive(Clone)]
struct Value(Arc<Cached>);
impl CacheValue for Value {
    fn size(&self) -> usize {
        self.0.retained_bytes().saturating_add(512)
    }
}
pub(super) struct ResidentCache {
    entries: DefaultCache<Key, Value>,
    flights: super::flight::Flights<Key, Cached>,
    loading: super::load::LoadCounters,
    epoch: AtomicUsize,
    admission: Mutex<()>,
    live: Arc<AtomicUsize>,
    pinned: Arc<AtomicUsize>,
    hits: AtomicUsize,
    loads: AtomicUsize,
    misses: AtomicUsize,
    bypasses: AtomicUsize,
}
impl std::fmt::Debug for ResidentCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeResidentCache")
            .field("entries", &self.entries.len())
            .finish_non_exhaustive()
    }
}
impl ResidentCache {
    pub(super) fn loads(&self) -> usize {
        self.loads.load(Ordering::Relaxed)
    }

    pub(super) fn new(policy: &super::CacheBudget) -> Self {
        Self {
            entries: DefaultCache::new(policy.resident_bytes).with_name("pse.cache.resident"),
            flights: super::flight::Flights::new(policy.concurrent_loads.get()),
            loading: super::load::LoadCounters::default(),
            live: Arc::new(AtomicUsize::new(0)),
            pinned: Arc::new(AtomicUsize::new(0)),
            epoch: AtomicUsize::new(0),
            admission: Mutex::new(()),
            hits: AtomicUsize::new(0),
            loads: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            bypasses: AtomicUsize::new(0),
        }
    }
    pub(super) fn invalidate(&self) {
        let _guard = self.admission.lock();
        self.epoch.fetch_add(1, Ordering::AcqRel);
        self.entries.clear();
    }
    pub(super) fn details(
        &self,
        rows: &mut Vec<super::CacheEntryReport>,
        limit: usize,
        owner: &datafusion::execution::memory_pool::MemoryReservation,
        budget: usize,
    ) -> Result<()> {
        let _guard = self
            .admission
            .lock()
            .map_err(|_| DataFusionError::Internal("cache admission lock poisoned".into()))?;
        super::details::reserve_inventory(owner, self.entries.memory_used(), budget)?;
        for (key, value) in self.entries.list_entries() {
            if rows.len() == limit {
                break;
            }
            rows.push(super::CacheEntryReport {
                cache: self.entries.name(),
                key: format!("{key:?}"),
                bytes: value.size_bytes,
                hits: value.hits,
            });
        }
        Ok(())
    }
    pub(super) fn report(&self) -> super::CacheReport {
        super::CacheReport {
            name: self.entries.name(),
            capacity_bytes: 0,
            inflight_bytes: Some(self.loading.bytes.load(Ordering::Acquire)),
            active_loads: Some(self.loading.active.load(Ordering::Acquire)),
            policy_limit_bytes: self.entries.cache_limit(),
            live_bytes: Some(self.live.load(Ordering::Acquire)),
            pinned_bytes: Some(self.pinned.load(Ordering::Acquire)),
            evictions: None,
            retained_bytes: self.entries.memory_used(),
            entries: self.entries.len(),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            bypasses: self.bypasses.load(Ordering::Relaxed),
        }
    }
}
#[derive(Debug)]
struct SelectedTable {
    inner: Arc<dyn TableProvider>,
    service: Arc<NativeCacheService>,
    state: Arc<SessionState>,
    location: url::Url,
    selection: String,
    identity: Arc<CacheIdentity>,
}
/// Wrap the full decoded selection, below query projection/filtering. The outer
/// common execution contract still re-admits policy and requirements on every read.
pub(crate) fn selected(
    inner: Arc<dyn TableProvider>,
    member: &pse_relations::generated::runtime::publications::RuntimePublicationsFieldMembersItem,
    state: &Arc<SessionState>,
) -> Result<Arc<dyn TableProvider>> {
    let Some(service) = state.config().get_extension::<NativeCacheService>() else {
        return Ok(inner);
    };
    let Some(identity) = state.config().get_extension::<CacheIdentity>() else {
        return Ok(inner);
    };
    if service.policy.resident_bytes == 0 {
        return Ok(inner);
    }
    Ok(Arc::new(SelectedTable {
        inner,
        service,
        state: state.clone(),
        location: url::Url::parse(&member.table_uri).map_err(external)?,
        selection: serde_json::to_string(member).map_err(external)?,
        identity,
    }))
}
#[async_trait::async_trait]
impl TableProvider for SelectedTable {
    fn schema(&self) -> SchemaRef {
        self.inner.schema()
    }
    fn table_type(&self) -> TableType {
        self.inner.table_type()
    }
    async fn scan(
        &self,
        session: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        _: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        if !filters.is_empty() {
            return Err(DataFusionError::Plan(
                "resident selection does not consume pushed filters".into(),
            ));
        }
        // Plan construction is pure. No lookup, read, collection or cache population.
        let input = self.inner.scan(session, None, &[], None).await?;
        if !matches!(input.properties().boundedness, Boundedness::Bounded) {
            return Err(DataFusionError::Plan(
                "resident selection must be finite".into(),
            ));
        }
        let plan: Arc<dyn ExecutionPlan> = Arc::new(SelectedExec {
            input,
            service: self.service.clone(),
            state: self.state.clone(),
            location: self.location.clone(),
            selection: self.selection.clone(),
            identity: self.identity.clone(),
            properties: Arc::new(PlanProperties::new(
                EquivalenceProperties::new(self.schema()),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Final,
                Boundedness::Bounded,
            )),
            metrics: ExecutionPlanMetricsSet::new(),
        });
        crate::session::physical_input::PhysicalInput::native(plan)
            .scan(session, projection, &[], None)
            .await
    }
}
#[derive(Debug)]
struct SelectedExec {
    input: Arc<dyn ExecutionPlan>,
    service: Arc<NativeCacheService>,
    state: Arc<SessionState>,
    location: url::Url,
    selection: String,
    identity: Arc<CacheIdentity>,
    properties: Arc<PlanProperties>,
    metrics: ExecutionPlanMetricsSet,
}
pub(crate) fn supports_round_reset(plan: &dyn ExecutionPlan) -> bool {
    plan.is::<SelectedExec>()
}
impl DisplayAs for SelectedExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NativeSelectedCacheExec: exact selection, lazy, lease-free idle entries")
    }
}
impl ExecutionPlan for SelectedExec {
    fn name(&self) -> &'static str {
        "NativeSelectedCacheExec"
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
        let input = children
            .pop()
            .filter(|_| children.is_empty())
            .ok_or_else(|| DataFusionError::Plan("resident cache needs one input".into()))?;
        Ok(Arc::new(Self {
            input,
            service: self.service.clone(),
            state: self.state.clone(),
            location: self.location.clone(),
            selection: self.selection.clone(),
            identity: self.identity.clone(),
            properties: self.properties.clone(),
            metrics: ExecutionPlanMetricsSet::new(),
        }))
    }
    fn apply_expressions(
        &self,
        _: &mut dyn FnMut(
            &Arc<dyn datafusion::physical_expr::PhysicalExpr>,
        ) -> Result<datafusion::common::tree_node::TreeNodeRecursion>,
    ) -> Result<datafusion::common::tree_node::TreeNodeRecursion> {
        Ok(datafusion::common::tree_node::TreeNodeRecursion::Continue)
    }
    fn metrics(&self) -> Option<datafusion::physical_plan::metrics::MetricsSet> {
        Some(self.metrics.clone_inner())
    }
    fn execute(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        if partition != 0 {
            return Err(DataFusionError::Execution(
                "resident selection has one partition".into(),
            ));
        }
        let service = self.service.clone();
        let location = self.location.clone();
        let state = self.state.clone();
        let selection = self.selection.clone();
        let identity = self.identity.clone();
        let input = self.input.clone();
        let metrics = self.metrics.clone();
        let schema = self.schema();
        let stream = futures_util::stream::once(async move {
            let cancel = context
                .session_config()
                .get_extension::<crate::session::execution::NativeExecutionContext>()
                .map_or_else(pse_ids::CancellationToken::new, |owner| {
                    owner.cancellation().clone()
                });
            let lease = crate::delta::lease::read(&location, &cancel).await?;
            let table = crate::delta::provider::table_builder(location.clone(), &state)?
                .build()
                .map_err(external)?;
            let log = table.log_store();
            let latest = log.get_latest_version(0).await.map_err(external)?;
            let store = state
                .runtime_env()
                .object_store_registry
                .get_store(&location)?;
            let generation = service.generation(&location, store);
            let settings = crate::session::config::semantic_settings(
                &crate::session::config::inventory(&state),
            )
            .map_err(external)?;
            let policies = identity
                .policies
                .iter()
                .map(crate::session::config::semantic_policy)
                .collect::<Vec<_>>();
            let policies = serde_json::to_string(&policies).map_err(external)?;
            let key = generation.map(|store| Key {
                store,
                latest,
                selection,
                schema: input.schema(),
                settings: crate::session::config::settings_hash(&settings),
                implementation: identity.generation,
                policies,
            });
            let cached = service
                .resident_value(key, input, context, &metrics, log)
                .await?;
            let source = cached.stream()?;
            let stream: SendableRecordBatchStream = Box::pin(RecordBatchStreamAdapter::new(
                source.schema(),
                source.map_ok(move |batch| {
                    let _lease = &lease;
                    batch
                }),
            ));
            Ok::<_, DataFusionError>(stream)
        })
        .try_flatten();
        Ok(Box::pin(RecordBatchStreamAdapter::new(schema, stream)))
    }
}
impl NativeCacheService {
    async fn resident_value(
        self: &Arc<Self>,
        key: Option<Key>,
        input: Arc<dyn ExecutionPlan>,
        context: Arc<TaskContext>,
        metrics: &ExecutionPlanMetricsSet,
        log: deltalake::logstore::LogStoreRef,
    ) -> Result<Arc<Cached>> {
        let metrics = metrics.clone();
        let cached = if let Some(key) = key {
            if let Some(value) = self.resident.entries.get(&key) {
                self.resident.hits.fetch_add(1, Ordering::Relaxed);
                value.0
            } else {
                self.resident.misses.fetch_add(1, Ordering::Relaxed);
                let owner = self.clone();
                let epoch = owner.resident.epoch.load(Ordering::Acquire);
                let population_key = key.clone();
                let latest = key.latest;
                self.resident
                    .flights
                    .load(key, || async move {
                        let _load = owner.admit_load(&owner.resident.loading).await?;
                        owner.resident.loads.fetch_add(1, Ordering::Relaxed);
                        let value = Cached::collect_bounded(
                            input,
                            context,
                            &metrics,
                            owner
                                .policy
                                .resident_bytes
                                .saturating_sub(population_key.size()),
                        )
                        .await?;
                        let accounted_key = value.reserve_identity(population_key.size());
                        let value =
                            Arc::new(value.account(
                                owner.resident.live.clone(),
                                owner.resident.pinned.clone(),
                            ));
                        let current = log.get_latest_version(0).await.map_err(external)?;
                        if current == latest
                            && accounted_key
                            && value.resident()
                            && let Ok(_guard) = owner.resident.admission.lock()
                            && owner.resident.epoch.load(Ordering::Acquire) == epoch
                        {
                            owner
                                .resident
                                .entries
                                .put(&population_key, Value(value.clone()));
                        } else {
                            owner.resident.bypasses.fetch_add(1, Ordering::Relaxed);
                        }
                        Ok(value)
                    })
                    .await?
            }
        } else {
            self.resident.bypasses.fetch_add(1, Ordering::Relaxed);
            let _load = self.admit_load(&self.resident.loading).await?;
            self.resident.loads.fetch_add(1, Ordering::Relaxed);
            Arc::new(
                Cached::collect_bounded(input, context, &metrics, self.policy.resident_bytes)
                    .await?,
            )
        };
        Ok(cached)
    }
}
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
