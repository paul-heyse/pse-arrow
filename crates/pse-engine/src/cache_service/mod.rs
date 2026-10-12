// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native caching owned by the shared deployment runtime, below session scopes.
pub mod details;
mod envelope;
pub use details::{CacheEntries, CacheEntryReport};
pub mod flight;
pub(crate) mod inspection;
pub mod load;
pub mod metrics;
mod namespace;
mod policy;
pub mod settings;
pub(crate) mod syntax;
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
                FileMetadataCache, FileStatisticsCache, ListFilesCache,
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
    source_generation: Option<String>,
    store: Arc<dyn object_store::ObjectStore>,
    reservation: MemoryReservation,
}

/// The authoritative cache observation fields, also projected at language boundaries.
#[macro_export]
macro_rules! cache_report_fields {
    ($emit:ident) => {
        $emit! {

        /// Native cache name.
                name: String,
        /// Pool-reserved capacity, including key/node allowance.
                capacity_bytes: usize,
        /// Native retained entry extent; excludes upstream reader Arc clones.
                retained_bytes: usize,
        /// Configured admission ceiling, distinct from a pool capacity reservation.
                policy_limit_bytes: usize,
        /// All live value owners, including values evicted while readers retain them.
                live_bytes: Option<usize>,
        /// Reader-owned bytes are unavailable for native file cache value types.
                pinned_bytes: Option<usize>,
        /// Reserved transient replay/decode staging; native allocator peaks are separate.
                inflight_bytes: Option<usize>,
        /// Currently admitted replay/decode loads under the shared deployment gate.
                active_loads: Option<usize>,
        /// Upstream eviction totals are not exposed by `DefaultCache`.
                evictions: Option<usize>,
        /// Retained entry count (expired entries may await native TTL removal).
                entries: usize,
        /// Successful native lookups.
                hits: usize,
        /// Unsuccessful native lookups.
                misses: usize,
        /// Explicit admission refusals, distinct from native eviction.
                bypasses: usize,
            }
    };
}
macro_rules! report {
    ($( $(#[$meta:meta])* $field:ident: $ty:ty, )*) => {
        /// Cheap aggregate observations without cloning the cached inventory.
        #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
        #[serde(deny_unknown_fields)]
        pub struct CacheReport { $( $(#[$meta])* pub $field: $ty, )* }
    };
}
cache_report_fields!(report);

/// One shared cache owner, carried in the actual native SessionConfig extensions.
#[derive(Debug)]
pub struct NativeCacheService {
    pub(crate) metrics: metrics::Metrics,
    policy: CacheBudget,
    pub(crate) model: Arc<crate::session::cache::model::ModelCache>,
    syntax: Arc<syntax::SyntaxCache>,
    files: Mutex<Arc<FileCacheBinding>>,
    stores: Mutex<Vec<StoreBinding>>,
    store_counters: StoreCounters,
    pool: Arc<dyn MemoryPool>,
    components: Mutex<Vec<std::sync::Weak<dyn CacheComponent>>>,
    loads: tokio::sync::Semaphore,
    queries: Arc<tokio::sync::Semaphore>,
    outputs: Arc<tokio::sync::Semaphore>,
    predicate_live: Arc<AtomicUsize>,
    maintenance: Arc<pse_columnar::retention::RetentionFence>,
}
#[derive(Debug, Default)]
struct FileCacheBinding {
    metadata: Option<Arc<Envelope<Path, CachedFileMetadataEntry>>>,
    statistics: Option<Arc<Envelope<TableScopedPath, CachedFileMetadata>>>,
    listing: Option<Arc<Envelope<TableScopedPath, CachedFileList>>>,
}

#[derive(Clone, Copy)]
enum FileCacheDemand {
    MetadataStatistics,
    WithListing,
}

/// An independently owned cache family sharing the deployment's native resources.
pub trait CacheComponent: std::fmt::Debug + Send + Sync {
    /// Whether storage maintenance invalidates this family's semantic dependencies.
    fn storage_bound(&self) -> bool {
        true
    }
    /// Evict cache references while retaining active readers.
    fn invalidate(&self);
    /// Constant-space aggregate counters.
    fn report(&self) -> Vec<CacheReport>;
    /// Bounded detailed inspection under the caller's existing staging reservation.
    /// # Errors
    /// Inventory staging exceeds the declared inspection allowance.
    fn details(
        &self,
        rows: &mut Vec<CacheEntryReport>,
        limit: usize,
        owner: &MemoryReservation,
        budget: usize,
    ) -> Result<()>;
    /// Family-specific actual event totals.
    fn execution_report(&self) -> Vec<(&'static str, Option<usize>)>;
}
impl NativeCacheService {
    /// Register a cache family without making a resource ownership cycle.
    pub fn register_component(&self, component: &Arc<dyn CacheComponent>) {
        let mut components = self
            .components
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Runtime-scoped families can come and go within one shared deployment.
        // Their weak registration must not retain every historical runtime slot.
        components.retain(|registered| registered.strong_count() != 0);
        components.push(Arc::downgrade(component));
    }
    fn components(&self) -> Vec<Arc<dyn CacheComponent>> {
        self.components
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter_map(std::sync::Weak::upgrade)
            .collect()
    }
    /// Actual deployment pool, shared by source-specific cache adapters.
    pub fn pool(&self) -> &Arc<dyn MemoryPool> {
        &self.pool
    }

    /// Evict service references after a native maintenance fence. Live readers
    /// retain their owners, and pending loads cannot publish across this epoch.
    pub fn invalidate(&self) {
        use datafusion::execution::cache::Cache;
        for component in self.components() {
            if component.storage_bound() {
                component.invalidate();
            }
        }
        self.maintenance.clear(|| {
            let files = self.file_caches();
            if let Some(cache) = &files.metadata {
                cache.clear();
            }
            if let Some(cache) = &files.statistics {
                cache.clear();
            }
            if let Some(cache) = &files.listing {
                cache.clear();
            }
        });
    }
    /// Reserve mandatory native capacity; file-cache families activate on qualified binding.
    /// # Errors
    /// The shared memory pool cannot reserve the configured capacity envelopes.
    pub fn new(policy: CacheBudget, pool: &Arc<dyn MemoryPool>) -> Result<Arc<Self>> {
        let maximum = match pool.memory_limit() {
            datafusion::execution::memory_pool::MemoryLimit::Finite(bytes) => bytes,
            _ => usize::MAX,
        };
        policy.validate(maximum).map_err(pse_columnar::external)?;
        let service = Arc::new(Self {
            maintenance: Arc::default(),
            model: crate::session::cache::model::ModelCache::new(policy.model_result_bytes),
            metrics: metrics::Metrics::default(),
            syntax: Envelope::new("pse.cache.syntax", policy.syntax_bytes, None, pool)?,
            files: Mutex::default(),
            components: Mutex::default(),
            loads: tokio::sync::Semaphore::new(policy.concurrent_loads.get()),
            queries: Arc::new(tokio::sync::Semaphore::new(policy.concurrent_queries.get())),
            outputs: Arc::new(tokio::sync::Semaphore::new(policy.concurrent_outputs.get())),
            predicate_live: Arc::default(),
            policy,
            stores: Mutex::new(Vec::new()),
            store_counters: StoreCounters::default(),
            pool: Arc::clone(pool),
        });
        let model: Arc<dyn CacheComponent> = service.model.clone();
        service.register_component(&model);
        Ok(service)
    }
    /// Cancellable outer-query admission. Hold until the stream settles.
    /// # Errors
    /// Cancellation or a closed deployment gate.
    pub async fn admit_query(
        &self,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<tokio::sync::OwnedSemaphorePermit> {
        admit(self.queries.clone(), cancel).await
    }
    /// Cancellable leaf-output admission. Acquire after its dependencies settle.
    /// # Errors
    /// Cancellation or a closed deployment gate.
    pub async fn admit_output(
        &self,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<tokio::sync::OwnedSemaphorePermit> {
        admit(self.outputs.clone(), cancel).await
    }
    /// The sole policy used to construct this service.
    pub fn policy(&self) -> &CacheBudget {
        &self.policy
    }
    fn file_caches(&self) -> Arc<FileCacheBinding> {
        self.files
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Stage requested capacity and publish all missing families together.
    /// No lock spans native configuration, I/O, planning or an await.
    fn activate(&self, demand: FileCacheDemand) -> Result<Arc<FileCacheBinding>> {
        let mut active = self.files.lock().map_err(|_| {
            datafusion::common::DataFusionError::Internal(
                "file-cache activation lock poisoned".into(),
            )
        })?;
        let with_listing =
            matches!(demand, FileCacheDemand::WithListing) || active.listing.is_some();
        let maximum = match self.pool.memory_limit() {
            datafusion::execution::memory_pool::MemoryLimit::Finite(bytes) => bytes,
            _ => usize::MAX,
        };
        if self
            .policy
            .activation_commitment(true, true, with_listing)
            .is_none_or(|bytes| bytes > maximum)
        {
            return Err(datafusion::common::DataFusionError::ResourcesExhausted("activated file-cache, staging, predicate and working capacities exceed the shared pool".into()));
        }
        if active.metadata.is_some()
            && active.statistics.is_some()
            && (!with_listing || active.listing.is_some())
        {
            return Ok(active.clone());
        }
        let metadata = match &active.metadata {
            Some(cache) => cache.clone(),
            None => Envelope::new(
                "pse.cache.metadata",
                self.policy.metadata_bytes,
                None,
                &self.pool,
            )?,
        };
        let statistics = match &active.statistics {
            Some(cache) => cache.clone(),
            None => Envelope::new(
                "pse.cache.statistics",
                self.policy.statistics_bytes,
                None,
                &self.pool,
            )?,
        };
        let listing = match &active.listing {
            Some(cache) => Some(cache.clone()),
            None if with_listing => Some(Envelope::new(
                "pse.cache.listing",
                self.policy.listing_bytes,
                self.policy.listing_ttl,
                &self.pool,
            )?),
            None => None,
        };
        let binding = Arc::new(FileCacheBinding {
            metadata: Some(metadata),
            statistics: Some(statistics),
            listing,
        });
        *active = binding.clone();
        Ok(binding)
    }

    /// Native configuration retaining only the source's demanded capacity owners.
    fn config(&self, generation: usize, demand: FileCacheDemand) -> Result<CacheManagerConfig> {
        let files = self.activate(demand)?;
        let mut config = Self::unbound_config()
            .with_file_metadata_cache(files.metadata.as_ref().map(
                |cache| -> Arc<FileMetadataCache> {
                    Namespaced::new(cache.clone(), generation, self.maintenance.clone())
                },
            ))
            .with_metadata_cache_limit(self.policy.metadata_bytes)
            .with_file_statistics_cache(files.statistics.as_ref().map(
                |cache| -> Arc<FileStatisticsCache> {
                    Namespaced::new(cache.clone(), generation, self.maintenance.clone())
                },
            ))
            .with_file_statistics_cache_limit(self.policy.statistics_bytes);
        if matches!(demand, FileCacheDemand::WithListing) {
            config = config
                .with_list_files_cache(files.listing.as_ref().map(|cache| -> Arc<ListFilesCache> {
                    Namespaced::new(cache.clone(), generation, self.maintenance.clone())
                }))
                .with_list_files_cache_limit(self.policy.listing_bytes)
                .with_list_files_cache_ttl(self.policy.listing_ttl);
        }
        Ok(config)
    }
    /// Unqualified stores cannot safely use native Path-keyed caches.
    pub fn unbound_config() -> CacheManagerConfig {
        CacheManagerConfig::default()
            .with_metadata_cache_limit(0)
            .with_file_statistics_cache_limit(0)
            .with_list_files_cache_limit(0)
    }
    /// Stable cache namespace for this actual registered store; refusal bypasses reuse.
    pub fn generation(
        &self,
        root: &url::Url,
        store: Arc<dyn object_store::ObjectStore>,
    ) -> Option<usize> {
        let (generation, reused) = self.source_generation(root, store, None)?;
        if reused {
            self.store_counters.hits.fetch_add(1, Ordering::Relaxed);
        }
        Some(generation)
    }
    fn source_generation(
        &self,
        root: &url::Url,
        store: Arc<dyn object_store::ObjectStore>,
        source_generation: Option<&str>,
    ) -> Option<(usize, bool)> {
        let mut stores = self.stores.lock().ok()?;
        if let Some(index) = stores.iter().position(|binding| {
            binding.root == *root
                && binding.source_generation.as_deref() == source_generation
                && Arc::ptr_eq(&binding.store, &store)
        }) {
            return Some((index, true));
        }
        self.store_counters.misses.fetch_add(1, Ordering::Relaxed);
        let result = self.bind_store(root, store, source_generation, &mut stores);
        if result.is_none() {
            self.store_counters.bypasses.fetch_add(1, Ordering::Relaxed);
        }
        result.map(|generation| (generation, false))
    }
    fn bind_store(
        &self,
        root: &url::Url,
        store: Arc<dyn object_store::ObjectStore>,
        source_generation: Option<&str>,
        stores: &mut Vec<StoreBinding>,
    ) -> Option<usize> {
        let bytes = root
            .as_str()
            .len()
            .checked_add(512)?
            .checked_add(source_generation.map_or(0, str::len))?;
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
            source_generation: source_generation.map(str::to_owned),
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
        let namespace = self.source_generation(root, store, None);
        self.configured_source_state(state, namespace, FileCacheDemand::WithListing)
    }
    /// Bind file-cache namespaces to a source owner's established generation.
    /// `None` disables file metadata/statistics/listing reuse. The caller must
    /// establish this generation from its actual source lifecycle, not a hash claim.
    /// Directory listings remain uncached because an append need not change it.
    /// # Errors
    /// Missing object store or native cache configuration failure.
    pub fn bind_state_with_generation(
        &self,
        root: &url::Url,
        state: &SessionState,
        source_generation: Option<&str>,
    ) -> Result<Arc<SessionState>> {
        let store = state.runtime_env().object_store_registry.get_store(root)?;
        let namespace = source_generation
            .and_then(|generation| self.source_generation(root, store, Some(generation)));
        self.configured_source_state(state, namespace, FileCacheDemand::MetadataStatistics)
    }
    fn configured_source_state(
        &self,
        state: &SessionState,
        namespace: Option<(usize, bool)>,
        demand: FileCacheDemand,
    ) -> Result<Arc<SessionState>> {
        let config = match namespace {
            Some((generation, _)) => self.config(generation, demand)?,
            None => Self::unbound_config(),
        };
        let configured = self.configured_state(state, &config)?;
        if namespace.is_some_and(|(_, reused)| reused) {
            self.store_counters.hits.fetch_add(1, Ordering::Relaxed);
        }
        Ok(configured)
    }
    fn configured_state(
        &self,
        state: &SessionState,
        config: &CacheManagerConfig,
    ) -> Result<Arc<SessionState>> {
        let runtime = state.runtime_env();
        let runtime = RuntimeEnv {
            memory_pool: Arc::clone(&runtime.memory_pool),
            disk_manager: Arc::clone(&runtime.disk_manager),
            object_store_registry: Arc::clone(&runtime.object_store_registry),
            cache_manager: datafusion::execution::cache::cache_manager::CacheManager::try_new(
                config,
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
        let files = self.file_caches();
        let mut reports = vec![
            self.syntax.report(),
            files.metadata.as_ref().map_or_else(
                || inactive_report("pse.cache.metadata", self.policy.metadata_bytes),
                |cache| cache.report(),
            ),
            files.statistics.as_ref().map_or_else(
                || inactive_report("pse.cache.statistics", self.policy.statistics_bytes),
                |cache| cache.report(),
            ),
            files.listing.as_ref().map_or_else(
                || inactive_report("pse.cache.listing", self.policy.listing_bytes),
                |cache| cache.report(),
            ),
            self.store_report(),
        ];
        for component in self.components() {
            reports.extend(component.report());
        }
        reports
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
    /// Actual event totals. Native replay action/byte and reader decode counters
    /// have no deployment-wide upstream hook; their values are explicitly absent.
    pub fn execution_report(&self) -> Vec<(&'static str, Option<usize>)> {
        self.metrics
            .report()
            .into_iter()
            .map(|(name, count)| (name, Some(count)))
            .chain([
                ("native_replay_actions", None),
                ("native_replay_bytes", None),
                ("native_decoded_rows", None),
            ])
            .chain(
                self.components()
                    .into_iter()
                    .flat_map(|component| component.execution_report()),
            )
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

async fn admit(
    gate: Arc<tokio::sync::Semaphore>,
    cancel: &pse_columnar::CancellationToken,
) -> Result<tokio::sync::OwnedSemaphorePermit> {
    tokio::select! {
        biased;
        () = cancel.cancelled() => Err(datafusion::common::DataFusionError::Execution("native admission cancelled".into())),
        permit = gate.acquire_owned() => permit.map_err(|error| datafusion::common::DataFusionError::External(Box::new(error))),
    }
}

fn inactive_report(name: &str, policy_limit_bytes: usize) -> CacheReport {
    CacheReport {
        name: name.into(),
        capacity_bytes: 0,
        retained_bytes: 0,
        policy_limit_bytes,
        live_bytes: None,
        pinned_bytes: None,
        inflight_bytes: None,
        active_loads: None,
        evictions: None,
        entries: 0,
        hits: 0,
        misses: 0,
        bypasses: 0,
    }
}
