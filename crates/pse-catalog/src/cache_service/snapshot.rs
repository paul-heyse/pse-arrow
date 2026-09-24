// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Retained native Delta snapshots. Lookup validates the maintenance generation;
//! cached entries hold no reader lease. Providers retain the entry's reservation.
use super::{DeltaCacheBudget, DeltaCacheService};
use datafusion::{
    common::{DataFusionError, Result, TableReference},
    execution::{
        cache::{Cache, CacheKey, CacheValue, default_cache::DefaultCache},
        memory_pool::{MemoryConsumer, MemoryReservation},
        session_state::SessionState,
    },
};
use deltalake::DeltaTable;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

/// Required native load capability; metadata-only states never satisfy query scans.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoadRequirement {
    /// Protocol/schema/table properties with no active file materialization.
    Metadata,
    /// Files and statistics for native query pruning.
    Query,
    /// Native maintenance explicitly requiring full files and statistics.
    Maintenance,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Key {
    store: usize,
    version: u64,
    maintenance: crate::delta::lease::Generation,
    requirement: LoadRequirement,
}
impl CacheKey for Key {
    fn size(&self) -> usize {
        size_of::<Self>() + 256
    }
    fn table_ref(&self) -> Option<&TableReference> {
        None
    }
}

/// Actual native table plus its retained allocation owner. Never expose an
/// unowned snapshot Arc to a scan: retain this object in the provider and stream.
#[derive(Debug)]
struct SnapshotValue {
    table: DeltaTable,
    readers: AtomicUsize,
    pinned: Arc<AtomicUsize>,
    reservation: MemoryReservation,
    live: Arc<AtomicUsize>,
}
impl Drop for SnapshotValue {
    fn drop(&mut self) {
        self.live
            .fetch_sub(self.reservation.size(), Ordering::AcqRel);
    }
}
/// One live reader of a retained native value. Cache entries never own these pins.
#[derive(Debug)]
pub(crate) struct RetainedTable {
    pub(crate) table: DeltaTable,
    _owner: SnapshotPin,
    _lease: Option<Arc<crate::delta::lease::ReadLease>>,
}
#[derive(Debug)]
struct SnapshotPin(Arc<SnapshotValue>);
impl SnapshotPin {
    fn acquire(value: Arc<SnapshotValue>) -> Self {
        if value.readers.fetch_add(1, Ordering::AcqRel) == 0 {
            value
                .pinned
                .fetch_add(value.reservation.size(), Ordering::AcqRel);
        }
        Self(value)
    }
}
impl Drop for SnapshotPin {
    fn drop(&mut self) {
        if self.0.readers.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.0
                .pinned
                .fetch_sub(self.0.reservation.size(), Ordering::AcqRel);
        }
    }
}
fn pin(
    value: Arc<SnapshotValue>,
    lease: Option<Arc<crate::delta::lease::ReadLease>>,
) -> Arc<RetainedTable> {
    Arc::new(RetainedTable {
        table: value.table.clone(),
        _owner: SnapshotPin::acquire(value),
        _lease: lease,
    })
}
#[derive(Clone)]
struct Entry(Arc<SnapshotValue>);
impl CacheValue for Entry {
    fn size(&self) -> usize {
        self.0.reservation.size()
    }
}

pub(super) struct SnapshotCache {
    entries: DefaultCache<Key, Entry>,
    flights: pse_engine::cache_service::flight::Flights<
        (pse_engine::session::execution::AttemptScope, Key),
        SnapshotValue,
    >,
    loading: pse_engine::cache_service::load::LoadCounters,
    live: Arc<AtomicUsize>,
    pinned: Arc<AtomicUsize>,
    loads: AtomicUsize,
    misses: AtomicUsize,
    bypasses: AtomicUsize,
    hits: AtomicUsize,
    epoch: AtomicUsize,
    admission: Mutex<()>,
}
impl std::fmt::Debug for SnapshotCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeSnapshotCache")
            .field("entries", &self.entries.len())
            .finish_non_exhaustive()
    }
}
impl SnapshotCache {
    pub(super) fn loads(&self) -> usize {
        self.loads.load(Ordering::Relaxed)
    }

    pub(super) fn new(policy: &DeltaCacheBudget) -> Self {
        Self {
            entries: DefaultCache::new(policy.snapshot_bytes).with_name("pse.cache.snapshots"),
            flights: pse_engine::cache_service::flight::Flights::new(
                policy.native.concurrent_loads.get(),
            ),
            loading: pse_engine::cache_service::load::LoadCounters::default(),
            live: Arc::new(AtomicUsize::new(0)),
            pinned: Arc::new(AtomicUsize::new(0)),
            loads: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            bypasses: AtomicUsize::new(0),
            hits: AtomicUsize::new(0),
            epoch: AtomicUsize::new(0),
            admission: Mutex::new(()),
        }
    }
    pub(super) fn details(
        &self,
        rows: &mut Vec<super::CacheEntryReport>,
        limit: usize,
        owner: &MemoryReservation,
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
            inflight_bytes: Some(self.loading.bytes.load(Ordering::Acquire)),
            active_loads: Some(self.loading.active.load(Ordering::Acquire)),
            policy_limit_bytes: self.entries.cache_limit(),
            live_bytes: Some(self.live_bytes()),
            pinned_bytes: Some(self.pinned.load(Ordering::Acquire)),
            evictions: None,
            retained_bytes: self.entries.memory_used(),
            entries: self.entries.len(),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            bypasses: self.bypasses.load(Ordering::Relaxed),
        }
    }
    pub(super) fn live_bytes(&self) -> usize {
        self.live.load(Ordering::Acquire)
    }
    pub(super) fn invalidate(&self) {
        let _guard = self.admission.lock();
        self.epoch.fetch_add(1, Ordering::AcqRel);
        self.entries.clear();
    }
    fn admit(&self, key: &Key, value: Entry, epoch: usize) {
        if let Ok(_guard) = self.admission.lock()
            && self.epoch.load(Ordering::Acquire) == epoch
            && key.size().saturating_add(value.size()) <= self.entries.cache_limit()
        {
            self.entries.put(key, value);
        } else {
            self.bypasses.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl DeltaCacheService {
    /// Open a native version under fresh read ownership held by the calling provider.
    pub(crate) async fn open_snapshot(
        self: &Arc<Self>,
        location: url::Url,
        version: Option<u64>,
        requirement: LoadRequirement,
        state: Arc<SessionState>,
    ) -> Result<Arc<RetainedTable>> {
        let cancel = state
            .config()
            .get_extension::<pse_engine::session::execution::NativeExecutionContext>()
            .map_or_else(pse_columnar::CancellationToken::new, |owner| {
                owner.cancellation().clone()
            });
        let lease = if requirement == LoadRequirement::Maintenance {
            None
        } else {
            crate::delta::lease::read(&location, &cancel).await?
        };
        let maintenance = lease.as_ref().map(|lease| lease.generation.clone());
        let store = state
            .runtime_env()
            .object_store_registry
            .get_store(&location)?;
        let generation = self.native().generation(&location, store);
        let mut builder = crate::delta::provider::table_builder(location, &state)?
            .with_log_buffer_size(1)
            .map_err(external)?;
        if requirement == LoadRequirement::Metadata {
            builder = builder.without_files();
        }
        let table = builder.build().map_err(external)?;
        let version = resolve_version(version, || async {
            table
                .log_store()
                .get_latest_version(0)
                .await
                .map_err(external)
        })
        .await?;
        let (Some(store), Some(maintenance)) = (generation, maintenance) else {
            self.snapshots.bypasses.fetch_add(1, Ordering::Relaxed);
            return self
                .load_snapshot(table, version)
                .await
                .map(|value| pin(value, lease));
        };
        let key = Key {
            store,
            version,
            maintenance,
            requirement,
        };
        if let Some(value) = self.snapshots.entries.get(&key) {
            self.snapshots.hits.fetch_add(1, Ordering::Relaxed);
            return Ok(pin(value.0, lease));
        }
        self.snapshots.misses.fetch_add(1, Ordering::Relaxed);
        let service = Arc::clone(self);
        let epoch = self.snapshots.epoch.load(Ordering::Acquire);
        let population_key = key.clone();
        self.snapshots
            .flights
            .load(
                (
                    state
                        .config()
                        .get_extension::<pse_engine::session::execution::NativeExecutionContext>()
                        .map(|services| services.attempt_scope())
                        .or_else(|| {
                            state
                                .config()
                                .get_extension::<pse_engine::session::execution::AttemptScope>()
                                .map(|scope| scope.as_ref().clone())
                        })
                        .unwrap_or_default(),
                    key,
                ),
                move || async move {
                    let value = service.load_snapshot(table, version).await?;
                    service
                        .snapshots
                        .admit(&population_key, Entry(Arc::clone(&value)), epoch);
                    Ok(value)
                },
            )
            .await
            .map(|value| pin(value, lease))
    }

    /// Retain a successfully returned native commit state without replaying its log.
    /// Failure here concerns acceleration only; the caller already owns a commit.
    pub(crate) fn remember_committed(
        &self,
        table: &DeltaTable,
        state: &SessionState,
    ) -> Result<()> {
        let location = table.log_store().root_url().clone();
        let store = state
            .runtime_env()
            .object_store_registry
            .get_store(&location)?;
        let Some(store) = self.native().generation(&location, store) else {
            return Ok(());
        };
        let Some(version) = table.version() else {
            return Ok(());
        };
        let epoch = self.snapshots.epoch.load(Ordering::Acquire);
        let Some(maintenance) = crate::delta::lease::generation(&location)? else {
            return Ok(());
        };
        let reservation = MemoryConsumer::new("pse.cache.committed_snapshot").register(&self.pool);
        let value = self.retain_snapshot(table.clone(), reservation)?;
        let key = Key {
            store,
            version,
            maintenance,
            requirement: if table.config.require_files {
                LoadRequirement::Query
            } else {
                LoadRequirement::Metadata
            },
        };
        self.snapshots.admit(&key, Entry(value.clone()), epoch);
        Ok(())
    }

    async fn load_snapshot(
        &self,
        mut table: DeltaTable,
        version: u64,
    ) -> Result<Arc<SnapshotValue>> {
        let _load = self.native.admit_load(&self.snapshots.loading).await?;
        let reservation = MemoryConsumer::new("pse.cache.snapshot_owner").register(&self.pool);
        self.snapshots.loads.fetch_add(1, Ordering::Relaxed);
        table.load_version(version).await.map_err(external)?;
        self.retain_snapshot(table, reservation)
    }
    fn retain_snapshot(
        &self,
        table: DeltaTable,
        reservation: MemoryReservation,
    ) -> Result<Arc<SnapshotValue>> {
        let bytes = table
            .snapshot()
            .map_err(external)?
            .snapshot()
            .snapshot_ref()
            .estimated_owned_heap_size_bytes()
            .saturating_add(size_of::<SnapshotValue>())
            .saturating_add(512);
        // Native replay lacks an allocation callback. The staging reservation and
        // semaphore bound fanout; the public native estimate governs retention.
        reservation.try_resize(bytes)?;
        self.snapshots.live.fetch_add(bytes, Ordering::AcqRel);
        Ok(Arc::new(SnapshotValue {
            table,
            readers: AtomicUsize::new(0),
            pinned: self.snapshots.pinned.clone(),
            reservation,
            live: Arc::clone(&self.snapshots.live),
        }))
    }
}
fn external(error: deltalake::DeltaTableError) -> DataFusionError {
    error.into()
}

/// Current-head I/O belongs exclusively to an unpinned request. Keep the callback
/// lazy so even constructing a current-head operation is absent on the exact path.
async fn resolve_version<F, Fut>(version: Option<u64>, current: F) -> Result<u64>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<u64>>,
{
    match version {
        Some(version) => Ok(version),
        None => current().await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::execution::memory_pool::{GreedyMemoryPool, MemoryPool};
    #[tokio::test]
    async fn pinned_cache_hit_never_reads_or_lists_the_fake_log_store() {
        use datafusion::execution::{
            runtime_env::RuntimeEnvBuilder, session_state::SessionStateBuilder,
        };
        let directory = tempfile::tempdir().unwrap();
        let root = url::Url::from_directory_path(directory.path()).unwrap();
        let store = Arc::new(super::super::store_tests::CountingStore::default());
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(32 << 20));
        let runtime = RuntimeEnvBuilder::new()
            .with_memory_pool(pool.clone())
            .build_arc()
            .unwrap();
        runtime.register_object_store(&root, store.clone());
        let service =
            DeltaCacheService::new(DeltaCacheBudget::for_memory(32 << 20), &pool).unwrap();
        let state = Arc::new(
            SessionStateBuilder::new()
                .with_default_features()
                .with_runtime_env(runtime)
                .build(),
        );
        let lease = crate::delta::lease::read(&root, &pse_columnar::CancellationToken::new())
            .await
            .unwrap()
            .unwrap();
        let key = Key {
            store: service.native().generation(&root, store.clone()).unwrap(),
            version: 7,
            maintenance: lease.generation.clone(),
            requirement: LoadRequirement::Query,
        };
        // A fake retained value isolates cache routing. No Delta commit/replay is
        // performed by this unit; real snapshot semantics belong to I18 journeys.
        let expected = retained(&service.snapshots, &pool);
        service.snapshots.admit(&key, Entry(expected.clone()), 0);
        let hit = service
            .open_snapshot(root, Some(7), LoadRequirement::Query, state)
            .await
            .unwrap();
        #[expect(
            clippy::used_underscore_binding,
            reason = "test compares the actual cache value owner without interpreting the fake table"
        )]
        let actual_owner = &hit._owner.0;
        assert!(Arc::ptr_eq(actual_owner, &expected));
        assert_eq!(store.gets.load(Ordering::SeqCst), 0);
        assert_eq!(store.lists.load(Ordering::SeqCst), 0);
        assert_eq!(service.snapshots.loads(), 0);
    }
    #[tokio::test]
    async fn exact_versions_never_observe_head_and_unpinned_requests_observe_it_once() {
        let heads = AtomicUsize::new(0);
        for requested in [Some(0), Some(5), Some(5), Some(1), None, None] {
            let actual = resolve_version(requested, || {
                heads.fetch_add(1, Ordering::SeqCst);
                std::future::ready(Ok(42))
            })
            .await
            .unwrap();
            assert_eq!(actual, requested.unwrap_or(42));
        }
        assert_eq!(heads.load(Ordering::SeqCst), 2);
        assert!(
            resolve_version(None, || std::future::ready(Err(
                DataFusionError::Execution("head unavailable".into())
            )))
            .await
            .is_err()
        );
    }
    fn retained(cache: &SnapshotCache, pool: &Arc<dyn MemoryPool>) -> Arc<SnapshotValue> {
        let reservation = MemoryConsumer::new("unit.snapshot").register(pool);
        reservation.try_grow(512).unwrap();
        cache.live.fetch_add(512, Ordering::AcqRel);
        Arc::new(SnapshotValue {
            table: DeltaTable::new_in_memory(),
            readers: AtomicUsize::new(0),
            pinned: cache.pinned.clone(),
            reservation,
            live: cache.live.clone(),
        })
    }
    fn key(version: u64) -> Key {
        Key {
            store: 0,
            version,
            maintenance: crate::delta::lease::test_generation(2),
            requirement: LoadRequirement::Query,
        }
    }
    #[test]
    fn eviction_and_invalidation_cannot_free_a_live_reader_or_admit_a_late_fill() {
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(4096));
        let mut policy = DeltaCacheBudget::disabled(1);
        policy.snapshot_bytes = 1024;
        let cache = SnapshotCache::new(&policy);
        let reader = retained(&cache, &pool);
        cache.admit(&key(1), Entry(reader.clone()), 0);
        cache.admit(&key(2), Entry(retained(&cache, &pool)), 0);
        assert!(cache.entries.get(&key(1)).is_none());
        assert_eq!(pool.reserved(), 1024);
        cache.invalidate();
        assert_eq!(pool.reserved(), 512);
        cache.admit(&key(1), Entry(reader.clone()), 0);
        assert!(cache.entries.is_empty());
        drop(reader);
        assert_eq!(pool.reserved(), 0);
        assert_eq!(cache.live_bytes(), 0);
    }
    #[test]
    fn load_capability_and_maintenance_generation_are_lookup_inputs() {
        let original = key(1);
        assert_ne!(
            original,
            Key {
                requirement: LoadRequirement::Metadata,
                ..original.clone()
            }
        );
        assert_ne!(
            original,
            Key {
                maintenance: crate::delta::lease::test_generation(3),
                ..original.clone()
            }
        );
        assert_ne!(
            original,
            Key {
                store: 1,
                ..original.clone()
            }
        );
    }
    #[test]
    fn pins_count_unique_native_owners_until_the_last_reader_drops() {
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(4096));
        let cache = SnapshotCache::new(&DeltaCacheBudget::disabled(1));
        let value = retained(&cache, &pool);
        assert_eq!(cache.report().pinned_bytes, Some(0));
        let first = pin(value.clone(), None);
        let second = pin(value.clone(), None);
        assert_eq!(cache.report().pinned_bytes, Some(512));
        drop(first);
        assert_eq!(cache.report().pinned_bytes, Some(512));
        drop(value);
        assert_eq!(cache.report().live_bytes, Some(512));
        drop(second);
        assert_eq!(cache.report().pinned_bytes, Some(0));
        assert_eq!(pool.reserved(), 0);
    }
}
