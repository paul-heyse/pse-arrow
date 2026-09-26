// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Lazy decoded exact selections, owned by the same native cache service as files
//! and snapshots. Idle entries contain only Arrow values and allocation owners.
use super::DeltaCacheService;
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
use pse_engine::session::cache::Cached;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

#[cfg(test)]
mod improvement_unit;

/// Actual frozen engine and policy assembly, retained through native scopes.

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Key {
    store: usize,
    maintenance: crate::delta::lease::Generation,
    selection: Arc<MemberSelection>,
    schema: SchemaRef,
    interpretation: Arc<Interpretation>,
}
#[derive(Debug, PartialEq, Eq)]
struct Interpretation {
    settings: Arc<std::collections::BTreeMap<String, Option<String>>>,
    implementation: pse_ids::SemanticId,
    policies: Arc<Vec<pse_schema::model::provider::ProviderPolicy>>,
}
#[derive(Clone, Debug, PartialEq)]
struct MemberSelection(
    pse_relations::generated::runtime::publications::RuntimePublicationsFieldMembersItem,
);
// The generated member consists only of strings, integer/identity fields and a
// typed optional revision. Its equality is reflexive; no floats participate.
impl Eq for MemberSelection {}
impl std::hash::Hash for MemberSelection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let member = &self.0;
        (
            &member.catalog_name,
            &member.schema_name,
            &member.table_name,
            member.relation_id,
            member.relation_version,
            member.contract_fingerprint,
            &member.table_uri,
            member.delta_version,
        )
            .hash(state);
        member.selection.kind.as_str().hash(state);
        member
            .selection
            .revision
            .as_ref()
            .map(|revision| (&revision.column, revision.revision_id))
            .hash(state);
    }
}
impl MemberSelection {
    fn size(&self) -> usize {
        size_of::<Self>()
            + [
                &self.0.catalog_name,
                &self.0.schema_name,
                &self.0.table_name,
                &self.0.table_uri,
            ]
            .into_iter()
            .map(String::capacity)
            .sum::<usize>()
            + self
                .0
                .selection
                .revision
                .as_ref()
                .map_or(0, |revision| revision.column.capacity())
    }
}
impl std::hash::Hash for Interpretation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hashing selects a bucket; full typed policy equality proves a hit.
        self.implementation.hash(state);
        self.settings.hash(state);
    }
}
impl CacheKey for Key {
    fn size(&self) -> usize {
        512 + self.selection.size()
            + self
                .interpretation
                .policies
                .iter()
                .map(|policy| {
                    512 + policy
                        .defaults
                        .iter()
                        .chain(&policy.required_settings)
                        .map(|(key, value)| key.capacity() + value.capacity() + 128)
                        .sum::<usize>()
                        + policy.requirements.len() * 128
                        + match &policy.scope {
                            pse_schema::model::provider::ProviderScope::Catalog(a) => a.capacity(),
                            pse_schema::model::provider::ProviderScope::Schema(a, b) => {
                                a.capacity() + b.capacity()
                            }
                            pse_schema::model::provider::ProviderScope::Table(a, b, c) => {
                                a.capacity() + b.capacity() + c.capacity()
                            }
                            _ => 0,
                        }
                })
                .sum::<usize>()
            + self
                .interpretation
                .settings
                .iter()
                .map(|(key, value)| {
                    key.capacity() + value.as_ref().map_or(0, String::capacity) + 64
                })
                .sum::<usize>()
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
    flights: pse_engine::cache_service::flight::Flights<
        (pse_engine::session::execution::AttemptScope, Key),
        Cached,
    >,
    loading: AtomicUsize,
    retention: pse_columnar::retention::RetentionFence,
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

    pub(super) fn new(policy: &super::DeltaCacheBudget) -> Self {
        Self {
            entries: DefaultCache::new(policy.resident_bytes).with_name("pse.cache.resident"),
            flights: pse_engine::cache_service::flight::Flights::new(
                policy.native.concurrent_loads.get(),
            ),
            loading: AtomicUsize::new(0),
            live: Arc::new(AtomicUsize::new(0)),
            pinned: Arc::new(AtomicUsize::new(0)),
            retention: Default::default(),
            admission: Mutex::new(()),
            hits: AtomicUsize::new(0),
            loads: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            bypasses: AtomicUsize::new(0),
        }
    }
    pub(super) fn invalidate(&self) {
        let _guard = self.admission.lock();
        self.retention.clear(|| self.entries.clear());
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
        pse_engine::cache_service::details::reserve_inventory(
            owner,
            self.entries.memory_used(),
            budget,
        )?;
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
            // Decoded batches charge the shared query pool as they arrive.
            inflight_bytes: None,
            active_loads: Some(self.loading.load(Ordering::Acquire)),
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
    service: Arc<DeltaCacheService>,
    state: Arc<SessionState>,
    location: url::Url,
    selection: Arc<MemberSelection>,
    interpretation: Arc<Interpretation>,
}
/// Wrap the full decoded selection, below query projection/filtering. The outer
/// common execution contract still re-admits policy and requirements on every read.
pub(crate) fn selected(
    inner: Arc<dyn TableProvider>,
    member: &pse_relations::generated::runtime::publications::RuntimePublicationsFieldMembersItem,
    state: &Arc<SessionState>,
) -> Result<Arc<dyn TableProvider>> {
    let Some(service) = state.config().get_extension::<DeltaCacheService>() else {
        return Ok(inner);
    };
    let Some(identity) = state
        .config()
        .get_extension::<pse_engine::session::assembly::AssemblyIdentity>()
    else {
        return Ok(inner);
    };
    if service.policy.resident_bytes == 0 {
        return Ok(inner);
    }
    Ok(Arc::new(SelectedTable {
        inner,
        service,
        state: state.clone(),
        location: url::Url::parse(&member.table_uri)
            .map_err(|error| DataFusionError::External(Box::new(error)))?,
        selection: Arc::new(MemberSelection(member.clone())),
        interpretation: Arc::new(Interpretation {
            settings: match &identity.settings {
                Some(settings) => settings.clone(),
                None => Arc::new(
                    pse_engine::session::config::semantic_settings(
                        &pse_engine::session::config::inventory(state),
                    )
                    .map_err(external)?,
                ),
            },
            implementation: identity.generation,
            policies: identity.policies.clone(),
        }),
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
    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<datafusion::logical_expr::TableProviderFilterPushDown>> {
        // Both cache hits and misses retain native residual evaluation. Unsupported
        // inner filters still reach this wrapper so they cannot trigger a full fill.
        Ok(vec![datafusion::logical_expr::TableProviderFilterPushDown::Inexact; filters.len()])
    }
    async fn scan(
        &self,
        session: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let supported = self
            .inner
            .supports_filters_pushdown(&filters.iter().collect::<Vec<_>>())?;
        if supported.len() != filters.len() {
            return Err(DataFusionError::Internal(
                "provider filter capability arity changed".into(),
            ));
        }
        let pruning: Vec<_> = filters
            .iter()
            .zip(supported)
            .filter_map(|(filter, support)| {
                (support != datafusion::logical_expr::TableProviderFilterPushDown::Unsupported)
                    .then(|| filter.clone())
            })
            .collect();
        let limit = filters.is_empty().then_some(limit).flatten();
        let full_schema = self.schema();
        let populate = filters.is_empty()
            && limit.is_none()
            && projection
                .is_none_or(|columns| columns.iter().copied().eq(0..full_schema.fields().len()));
        // Plan construction does not inspect the cache or execute input rows.
        let input = self
            .inner
            .scan(session, projection, &pruning, limit)
            .await?;
        if !matches!(input.properties().boundedness, Boundedness::Bounded) {
            return Err(DataFusionError::Plan(
                "resident selection must be finite".into(),
            ));
        }
        Ok(Arc::new(SelectedExec {
            properties: Arc::new(PlanProperties::new(
                EquivalenceProperties::new(input.schema()),
                Partitioning::UnknownPartitioning(1),
                if populate {
                    EmissionType::Final
                } else {
                    EmissionType::Incremental
                },
                Boundedness::Bounded,
            )),
            input,
            full_schema,
            projection: projection.cloned(),
            limit,
            populate,
            service: self.service.clone(),
            state: self.state.clone(),
            location: self.location.clone(),
            selection: self.selection.clone(),
            interpretation: self.interpretation.clone(),
            reuse: true,
            metrics: ExecutionPlanMetricsSet::new(),
        }))
    }
}

#[derive(Debug)]
struct SelectedExec {
    input: Arc<dyn ExecutionPlan>,
    full_schema: SchemaRef,
    projection: Option<Vec<usize>>,
    limit: Option<usize>,
    populate: bool,
    service: Arc<DeltaCacheService>,
    state: Arc<SessionState>,
    location: url::Url,
    selection: Arc<MemberSelection>,
    interpretation: Arc<Interpretation>,
    // An arbitrary physical rewrite is not evidence for the original exact source.
    reuse: bool,
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
        let reuse =
            self.reuse && pse_engine::operation::ports::same_physical_input(&input, &self.input);
        Ok(Arc::new(Self {
            input,
            full_schema: self.full_schema.clone(),
            projection: self.projection.clone(),
            limit: self.limit,
            populate: self.populate,
            service: self.service.clone(),
            state: self.state.clone(),
            location: self.location.clone(),
            selection: self.selection.clone(),
            interpretation: self.interpretation.clone(),
            reuse,
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
        let interpretation = self.interpretation.clone();
        let reuse = self.reuse;
        let input = self.input.clone();
        let metrics = self.metrics.clone();
        let schema = self.schema();
        let full_schema = self.full_schema.clone();
        let projection = self.projection.clone();
        let limit = self.limit;
        let populate = self.populate;
        let stream = futures_util::stream::once(async move {
            let cancel = context
                .session_config()
                .get_extension::<pse_engine::session::execution::NativeExecutionContext>()
                .map_or_else(pse_columnar::CancellationToken::new, |owner| {
                    owner.cancellation().clone()
                });
            let lease = crate::delta::lease::read(&location, &cancel).await?;
            let store = state
                .runtime_env()
                .object_store_registry
                .get_store(&location)?;
            let generation = service.native().generation(&location, store);
            let key = generation
                .zip(lease.as_ref())
                .filter(|_| reuse)
                .map(|(store, lease)| Key {
                    store,
                    maintenance: lease.generation.clone(),
                    selection,
                    schema: full_schema,
                    interpretation,
                });
            let source = if populate {
                service
                    .resident_value(key, input, context, &metrics)
                    .await?
                    .stream()?
            } else if let Some(value) = key
                .as_ref()
                .and_then(|key| service.resident.entries.get(key))
            {
                service.resident.hits.fetch_add(1, Ordering::Relaxed);
                let mut plan = pse_engine::session::physical_input::PhysicalInput::native(
                    value.0.reader_plan()?,
                )
                .scan(state.as_ref(), projection.as_ref(), &[], None)
                .await?;
                if let Some(limit) = limit {
                    plan = Arc::new(datafusion::physical_plan::limit::GlobalLimitExec::new(
                        plan,
                        0,
                        Some(limit),
                    ));
                }
                datafusion::physical_plan::execute_stream(plan, context)?
            } else {
                service.resident.misses.fetch_add(1, Ordering::Relaxed);
                service.resident.bypasses.fetch_add(1, Ordering::Relaxed);
                datafusion::physical_plan::execute_stream(input, context)?
            };
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
impl DeltaCacheService {
    async fn resident_value(
        self: &Arc<Self>,
        key: Option<Key>,
        input: Arc<dyn ExecutionPlan>,
        context: Arc<TaskContext>,
        metrics: &ExecutionPlanMetricsSet,
    ) -> Result<Arc<Cached>> {
        let metrics = metrics.clone();
        let cached = if let Some(key) = key {
            if let Some(value) = self.resident.entries.get(&key) {
                self.resident.hits.fetch_add(1, Ordering::Relaxed);
                value.0
            } else {
                self.resident.misses.fetch_add(1, Ordering::Relaxed);
                let owner = self.clone();
                let epoch = owner.resident.retention.generation();
                let population_key = key.clone();
                self.resident
                    .flights
                    .load((context.session_config().get_extension::<pse_engine::session::execution::NativeExecutionContext>()
                    .map(|services| services.attempt_scope()).or_else(|| context.session_config().get_extension::<pse_engine::session::execution::AttemptScope>().map(|scope| scope.as_ref().clone())).unwrap_or_default(), key), move || async move {
                        // The outer query owns admission. Do not hold a replay
                        // permit while its children may need that same load gate.
                        let _active = ActiveDecode::new(&owner.resident.loading);
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
                        if accounted_key
                            && value.resident()
                            && let Ok(_guard) = owner.resident.admission.lock()
                        {
                            if owner.resident.retention.admit(epoch, || owner.resident.entries.put(&population_key, Value(value.clone()))).is_none() {
                                owner.resident.bypasses.fetch_add(1, Ordering::Relaxed);
                            }
                        } else {
                            owner.resident.bypasses.fetch_add(1, Ordering::Relaxed);
                        }
                        Ok(value)
                    })
                    .await?
            }
        } else {
            self.resident.bypasses.fetch_add(1, Ordering::Relaxed);

            let _active = ActiveDecode::new(&self.resident.loading);
            self.resident.loads.fetch_add(1, Ordering::Relaxed);
            Arc::new(
                Cached::collect_bounded(input, context, &metrics, self.policy.resident_bytes)
                    .await?,
            )
        };
        Ok(cached)
    }
}
fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}

struct ActiveDecode<'a>(&'a AtomicUsize);
impl<'a> ActiveDecode<'a> {
    fn new(counter: &'a AtomicUsize) -> Self {
        counter.fetch_add(1, Ordering::AcqRel);
        Self(counter)
    }
}
impl Drop for ActiveDecode<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}
