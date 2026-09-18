// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native caching owned by the shared deployment runtime, below session scopes.
#[cfg(test)]
mod delta_journeys;
mod details;
mod envelope;
pub use details::{CacheEntries, CacheEntryReport};
mod flight;
pub(crate) mod inspection;
mod load;
pub(crate) mod metrics;
mod namespace;
mod policy;
pub(crate) mod resident;
pub mod snapshot;
#[cfg(test)]
mod store_tests;
#[cfg(test)]
mod tests;
pub use policy::CacheBudget;

use datafusion::execution::{
    memory_pool::{MemoryConsumer, MemoryReservation},
    runtime_env::RuntimeEnv,
    session_state::{SessionState, SessionStateBuilder},
};
use datafusion::{
    common::Result,
    execution::{
        cache::{
            TableScopedPath,
            cache_manager::{
                CacheManagerConfig, CachedFileList, CachedFileMetadata, CachedFileMetadataEntry,
            },
        },
        memory_pool::MemoryPool,
    },
};
use envelope::Envelope;
use namespace::Namespaced;
use object_store::path::Path;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Debug, Default)]
struct StoreCounters {
    hits: AtomicUsize,
    misses: AtomicUsize,
    bypasses: AtomicUsize,
}

#[derive(Debug)]
struct StoreBinding {
    root: url::Url,
    store: Arc<dyn object_store::ObjectStore>,
    reservation: MemoryReservation,
}

/// Cheap aggregate observations: no cache lookup, file IO or cloned value inventory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheReport {
    /// Native cache name.
    pub name: String,
    /// Pool-reserved capacity, including key/node allowance.
    pub capacity_bytes: usize,
    /// Native retained entry extent; excludes upstream reader Arc clones.
    pub retained_bytes: usize,
    /// Configured admission ceiling, distinct from a pool capacity reservation.
    pub policy_limit_bytes: usize,
    /// All live value owners, including values evicted while readers retain them.
    pub live_bytes: Option<usize>,
    /// Reader-owned bytes are unavailable for native file cache value types.
    pub pinned_bytes: Option<usize>,
    /// Reserved transient replay/decode staging; native allocator peaks are separate.
    pub inflight_bytes: Option<usize>,
    /// Currently admitted replay/decode loads under the shared deployment gate.
    pub active_loads: Option<usize>,
    /// Upstream eviction totals are not exposed by `DefaultCache`.
    pub evictions: Option<usize>,
    /// Retained entry count (expired entries may await native TTL removal).
    pub entries: usize,
    /// Successful native lookups.
    pub hits: usize,
    /// Unsuccessful native lookups.
    pub misses: usize,
    /// Explicit admission refusals, distinct from native eviction.
    pub bypasses: usize,
}

/// One shared cache owner, carried in the actual native SessionConfig extensions.
#[derive(Debug)]
pub struct NativeCacheService {
    pub(crate) metrics: metrics::Metrics,
    policy: CacheBudget,
    metadata: Arc<Envelope<Path, CachedFileMetadataEntry>>,
    statistics: Arc<Envelope<TableScopedPath, CachedFileMetadata>>,
    listing: Arc<Envelope<TableScopedPath, CachedFileList>>,
    stores: Mutex<Vec<StoreBinding>>,
    store_counters: StoreCounters,
    pool: Arc<dyn MemoryPool>,
    snapshots: snapshot::SnapshotCache,
    resident: resident::ResidentCache,
    loads: tokio::sync::Semaphore,
}
impl NativeCacheService {
    /// Evict service references after a native maintenance fence. Live readers
    /// retain their owners, and pending loads cannot publish across this epoch.
    pub fn invalidate(&self) {
        use datafusion::execution::cache::Cache;
        self.snapshots.invalidate();
        self.resident.invalidate();
        self.metadata.clear();
        self.statistics.clear();
        self.listing.clear();
    }
    /// Reserve every native file-cache capacity before creating its cache.
    /// # Errors
    /// The shared memory pool cannot reserve the configured capacity envelopes.
    pub fn new(policy: CacheBudget, pool: &Arc<dyn MemoryPool>) -> Result<Arc<Self>> {
        let maximum = match pool.memory_limit() {
            datafusion::execution::memory_pool::MemoryLimit::Finite(bytes) => bytes,
            _ => usize::MAX,
        };
        policy
            .validate(maximum)
            .map_err(|error| datafusion::common::DataFusionError::External(Box::new(error)))?;
        Ok(Arc::new(Self {
            metrics: metrics::Metrics::default(),
            metadata: Envelope::new("pse.cache.metadata", policy.metadata_bytes, None, pool)?,
            statistics: Envelope::new("pse.cache.statistics", policy.statistics_bytes, None, pool)?,
            listing: Envelope::new(
                "pse.cache.listing",
                policy.listing_bytes,
                policy.listing_ttl,
                pool,
            )?,
            snapshots: snapshot::SnapshotCache::new(&policy),
            resident: resident::ResidentCache::new(&policy),
            loads: tokio::sync::Semaphore::new(policy.concurrent_loads.get()),
            policy,
            stores: Mutex::new(Vec::new()),
            store_counters: StoreCounters::default(),
            pool: Arc::clone(pool),
        }))
    }
    /// The sole policy used to construct this service.
    pub fn policy(&self) -> &CacheBudget {
        &self.policy
    }
    /// Native configuration retaining all capacity owners through cache Arcs.
    fn config(&self, generation: usize) -> CacheManagerConfig {
        CacheManagerConfig::default()
            .with_file_metadata_cache(Some(Namespaced::new(self.metadata.clone(), generation)))
            .with_metadata_cache_limit(self.policy.metadata_bytes)
            .with_file_statistics_cache(Some(Namespaced::new(self.statistics.clone(), generation)))
            .with_file_statistics_cache_limit(self.policy.statistics_bytes)
            .with_list_files_cache(Some(Namespaced::new(self.listing.clone(), generation)))
            .with_list_files_cache_limit(self.policy.listing_bytes)
            .with_list_files_cache_ttl(self.policy.listing_ttl)
    }
    /// Unqualified stores cannot safely use native Path-keyed caches.
    pub fn unbound_config() -> CacheManagerConfig {
        CacheManagerConfig::default()
            .with_metadata_cache_limit(0)
            .with_file_statistics_cache_limit(0)
            .with_list_files_cache_limit(0)
    }
    fn generation(
        &self,
        root: &url::Url,
        store: Arc<dyn object_store::ObjectStore>,
    ) -> Option<usize> {
        let mut stores = self.stores.lock().ok()?;
        if let Some(index) = stores
            .iter()
            .position(|binding| binding.root == *root && Arc::ptr_eq(&binding.store, &store))
        {
            self.store_counters.hits.fetch_add(1, Ordering::Relaxed);
            return Some(index);
        }
        self.store_counters.misses.fetch_add(1, Ordering::Relaxed);
        let result = self.bind_store(root, store, &mut stores);
        if result.is_none() {
            self.store_counters.bypasses.fetch_add(1, Ordering::Relaxed);
        }
        result
    }
    fn bind_store(
        &self,
        root: &url::Url,
        store: Arc<dyn object_store::ObjectStore>,
        stores: &mut Vec<StoreBinding>,
    ) -> Option<usize> {
        let bytes = root.as_str().len().checked_add(512)?;
        let retained = stores.iter().try_fold(0usize, |total, binding| {
            total.checked_add(binding.reservation.size())
        })?;
        if retained.checked_add(bytes)? > self.policy.inspection_bytes / 2 {
            return None;
        }
        let reservation = MemoryConsumer::new("pse.cache.store_bindings").register(&self.pool);
        reservation.try_grow(bytes).ok()?;
        let generation = stores.len();
        stores.push(StoreBinding {
            root: root.clone(),
            store,
            reservation,
        });
        Some(generation)
    }
    /// Bind cache keys to the actual registration and canonical table root.
    /// All other native runtime state and session implementations are retained.
    /// # Errors
    /// The root cannot be resolved in the caller registry or native configuration fails.
    pub fn bind_state(&self, root: &url::Url, state: &SessionState) -> Result<Arc<SessionState>> {
        let runtime = state.runtime_env();
        let store = runtime.object_store_registry.get_store(root)?;
        let config = self
            .generation(root, store)
            .map_or_else(Self::unbound_config, |generation| self.config(generation));
        let runtime = RuntimeEnv {
            memory_pool: Arc::clone(&runtime.memory_pool),
            disk_manager: Arc::clone(&runtime.disk_manager),
            object_store_registry: Arc::clone(&runtime.object_store_registry),
            cache_manager: datafusion::execution::cache::cache_manager::CacheManager::try_new(
                &config,
            )?,
        };
        let mut session_config = state.config().clone();
        session_config
            .options_mut()
            .execution
            .parquet
            .max_predicate_cache_size = Some(
            session_config
                .options()
                .execution
                .parquet
                .max_predicate_cache_size
                .unwrap_or(self.policy.predicate_cache_bytes)
                .min(self.policy.predicate_cache_bytes),
        );
        Ok(Arc::new(
            SessionStateBuilder::new_from_existing(state.clone())
                .with_config(session_config)
                .with_runtime_env(Arc::new(runtime))
                .build(),
        ))
    }
    /// Constant-space aggregate inspection, never an entry enumeration.
    pub fn report(&self) -> Vec<CacheReport> {
        vec![
            self.metadata.report(),
            self.statistics.report(),
            self.listing.report(),
            self.snapshots.report(),
            self.resident.report(),
            self.store_report(),
        ]
    }
    fn store_report(&self) -> CacheReport {
        // Reporting remains read-only even if mutation of this registry was poisoned.
        let stores = self
            .stores
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let bytes = stores
            .iter()
            .map(|binding| binding.reservation.size())
            .sum();
        CacheReport {
            name: "pse.cache.store_bindings".into(),
            capacity_bytes: 0,
            retained_bytes: bytes,
            policy_limit_bytes: self.policy.inspection_bytes / 2,
            live_bytes: Some(bytes),
            pinned_bytes: None,
            inflight_bytes: None,
            active_loads: None,
            evictions: Some(0),
            entries: stores.len(),
            hits: self.store_counters.hits.load(Ordering::Relaxed),
            misses: self.store_counters.misses.load(Ordering::Relaxed),
            bypasses: self.store_counters.bypasses.load(Ordering::Relaxed),
        }
    }
    /// Snapshot owners retained by both cache entries and active readers.
    pub fn live_snapshot_bytes(&self) -> usize {
        self.snapshots.live_bytes()
    }
    /// Actual event totals. Native replay action/byte and reader decode counters
    /// have no deployment-wide upstream hook; their values are explicitly absent.
    pub fn execution_report(&self) -> Vec<(&'static str, Option<usize>)> {
        self.metrics
            .report()
            .into_iter()
            .map(|(name, count)| (name, Some(count)))
            .chain([
                ("snapshot_loads", Some(self.snapshots.loads())),
                ("resident_loads", Some(self.resident.loads())),
                ("native_replay_actions", None),
                ("native_replay_bytes", None),
                ("native_decoded_rows", None),
            ])
            .collect()
    }
}

/// Use the deployment's qualified cache view if its service is bound.
/// # Errors
/// Native storage resolution or cache configuration fails.
pub fn bind_state(root: &url::Url, state: &Arc<SessionState>) -> Result<Arc<SessionState>> {
    match state.config().get_extension::<NativeCacheService>() {
        Some(service) => service.bind_state(root, state),
        None => {
            NativeCacheService::new(CacheBudget::disabled(1), &state.runtime_env().memory_pool)?
                .bind_state(root, state)
        }
    }
}
