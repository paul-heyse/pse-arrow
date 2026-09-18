// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Retained native Delta snapshots. Lookup validates current log generation;
//! cached entries hold no reader lease. Providers retain the entry's reservation.
use super::{CacheBudget, NativeCacheService};
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
    observed_latest: u64,
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
fn pin(value: Arc<SnapshotValue>) -> Arc<RetainedTable> {
    Arc::new(RetainedTable {
        table: value.table.clone(),
        _owner: SnapshotPin::acquire(value),
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
    flights: super::flight::Flights<Key, SnapshotValue>,
    loading: super::load::LoadCounters,
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

    pub(super) fn new(policy: &CacheBudget) -> Self {
        Self {
            entries: DefaultCache::new(policy.snapshot_bytes).with_name("pse.cache.snapshots"),
            flights: super::flight::Flights::new(policy.concurrent_loads.get()),
            loading: super::load::LoadCounters::default(),
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

impl NativeCacheService {
    /// Open a native version under fresh read ownership held by the calling provider.
    pub(crate) async fn open_snapshot(
        self: &Arc<Self>,
        location: url::Url,
        version: Option<u64>,
        requirement: LoadRequirement,
        state: Arc<SessionState>,
    ) -> Result<Arc<RetainedTable>> {
        let store = state
            .runtime_env()
            .object_store_registry
            .get_store(&location)?;
        let generation = self.generation(&location, store);
        let mut builder = crate::delta::provider::table_builder(location, &state)?
            .with_log_buffer_size(self.policy.concurrent_loads.get())
            .map_err(external)?;
        if requirement == LoadRequirement::Metadata {
            builder = builder.without_files();
        }
        let table = builder.build().map_err(external)?;
        // This lookup bypasses all cached Delta snapshots and directory caches.
        let observed_latest = table
            .log_store()
            .get_latest_version(0)
            .await
            .map_err(external)?;
        let version = version.unwrap_or(observed_latest);
        if version > observed_latest {
            return Err(DataFusionError::Plan(
                "requested Delta version exceeds the observed log generation".into(),
            ));
        }
        let Some(store) = generation else {
            self.snapshots.bypasses.fetch_add(1, Ordering::Relaxed);
            return self.load_snapshot(table, version, None).await.map(pin);
        };
        let key = Key {
            store,
            version,
            observed_latest,
            requirement,
        };
        if let Some(value) = self.snapshots.entries.get(&key) {
            self.snapshots.hits.fetch_add(1, Ordering::Relaxed);
            return Ok(pin(value.0));
        }
        self.snapshots.misses.fetch_add(1, Ordering::Relaxed);
        let seed_key = Key {
            store,
            version: u64::MAX,
            observed_latest: u64::MAX,
            requirement,
        };
        let seed = (version == observed_latest)
            .then(|| self.snapshots.entries.get(&seed_key))
            .flatten()
            .map(|entry| entry.0)
            .filter(|entry| entry.table.version().is_some_and(|old| old < version));
        let service = Arc::clone(self);
        let epoch = self.snapshots.epoch.load(Ordering::Acquire);
        let population_key = key.clone();
        self.snapshots
            .flights
            .load(key, || async move {
                let value = service.load_snapshot(table, version, seed).await?;
                // An uncached native observation guards cross-process maintenance;
                // admission's epoch guard linearizes local invalidation and insertion.
                let current = value
                    .table
                    .log_store()
                    .get_latest_version(0)
                    .await
                    .map_err(external)?;
                if current == population_key.observed_latest {
                    service
                        .snapshots
                        .admit(&population_key, Entry(Arc::clone(&value)), epoch);
                    service
                        .snapshots
                        .admit(&seed_key, Entry(Arc::clone(&value)), epoch);
                }
                Ok(value)
            })
            .await
            .map(pin)
    }

    /// Retain a successfully returned native commit state without replaying its log.
    /// Failure here concerns acceleration only; the caller already owns a commit.
    pub(crate) async fn remember_committed(
        &self,
        table: &DeltaTable,
        state: &SessionState,
    ) -> Result<()> {
        let location = table.log_store().root_url().clone();
        let store = state
            .runtime_env()
            .object_store_registry
            .get_store(&location)?;
        let Some(store) = self.generation(&location, store) else {
            return Ok(());
        };
        let Some(version) = table.version() else {
            return Ok(());
        };
        let epoch = self.snapshots.epoch.load(Ordering::Acquire);
        let observed_latest = table
            .log_store()
            .get_latest_version(0)
            .await
            .map_err(external)?;
        if version != observed_latest {
            return Ok(());
        }
        let reservation = MemoryConsumer::new("pse.cache.committed_snapshot").register(&self.pool);
        let value = self.retain_snapshot(table.clone(), reservation)?;
        let key = Key {
            store,
            version,
            observed_latest,
            requirement: LoadRequirement::Query,
        };
        self.snapshots.admit(&key, Entry(value.clone()), epoch);
        let seed = Key {
            version: u64::MAX,
            observed_latest: u64::MAX,
            ..key
        };
        self.snapshots.admit(&seed, Entry(value), epoch);
        Ok(())
    }

    async fn load_snapshot(
        &self,
        mut table: DeltaTable,
        version: u64,
        seed: Option<Arc<SnapshotValue>>,
    ) -> Result<Arc<SnapshotValue>> {
        let _load = self.admit_load(&self.snapshots.loading).await?;
        let reservation = MemoryConsumer::new("pse.cache.snapshot_owner").register(&self.pool);
        self.snapshots.loads.fetch_add(1, Ordering::Relaxed);
        if let Some(seed) = &seed {
            table = seed.table.clone();
        }
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
    DataFusionError::External(Box::new(error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::execution::memory_pool::{GreedyMemoryPool, MemoryPool};
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
            observed_latest: 2,
            requirement: LoadRequirement::Query,
        }
    }
    #[test]
    fn eviction_and_invalidation_cannot_free_a_live_reader_or_admit_a_late_fill() {
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(4096));
        let mut policy = CacheBudget::disabled(1);
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
    fn load_capability_and_observed_generation_are_lookup_inputs() {
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
                observed_latest: 3,
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
        let cache = SnapshotCache::new(&CacheBudget::disabled(1));
        let value = retained(&cache, &pool);
        assert_eq!(cache.report().pinned_bytes, Some(0));
        let first = pin(value.clone());
        let second = pin(value.clone());
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
