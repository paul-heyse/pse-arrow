// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native caching owned by the shared deployment runtime, below session scopes.
pub mod details;
mod envelope;
pub use details::{CacheEntries, CacheEntryReport};
pub(crate) mod inspection;
pub mod load;
pub mod metrics;
mod policy;
pub mod settings;
pub(crate) mod syntax;
#[cfg(test)]
mod tests;
pub use policy::CacheBudget;

use datafusion::{
    common::Result,
    execution::{
        cache::cache_manager::CacheManagerConfig,
        memory_pool::{MemoryPool, MemoryReservation},
    },
};
use envelope::Envelope;
use std::sync::{Arc, Mutex, atomic::AtomicUsize};

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
    pool: Arc<dyn MemoryPool>,
    components: Mutex<Vec<std::sync::Weak<dyn CacheComponent>>>,
    loads: tokio::sync::Semaphore,
    queries: Arc<tokio::sync::Semaphore>,
    outputs: Arc<tokio::sync::Semaphore>,
    predicate_live: Arc<AtomicUsize>,
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
        for component in self.components() {
            if component.storage_bound() {
                component.invalidate();
            }
        }
    }
    /// Reserve the actual native syntax/model capacity and shared execution allowances.
    /// # Errors
    /// The shared memory pool cannot reserve the configured capacity envelopes.
    pub fn new(policy: CacheBudget, pool: &Arc<dyn MemoryPool>) -> Result<Arc<Self>> {
        let maximum = match pool.memory_limit() {
            datafusion::execution::memory_pool::MemoryLimit::Finite(bytes) => bytes,
            _ => usize::MAX,
        };
        policy.validate(maximum).map_err(pse_columnar::external)?;
        let service = Arc::new(Self {
            model: crate::session::cache::model::ModelCache::new(policy.model_result_bytes),
            metrics: metrics::Metrics::default(),
            syntax: Envelope::new("pse.cache.syntax", policy.syntax_bytes, None, pool)?,
            components: Mutex::default(),
            loads: tokio::sync::Semaphore::new(policy.concurrent_loads.get()),
            queries: Arc::new(tokio::sync::Semaphore::new(policy.concurrent_queries.get())),
            outputs: Arc::new(tokio::sync::Semaphore::new(policy.concurrent_outputs.get())),
            predicate_live: Arc::default(),
            policy,
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
    /// File caches stay disabled: canonical protected IPC has its own immutable decoder.
    pub fn unbound_config() -> CacheManagerConfig {
        CacheManagerConfig::default()
            .with_metadata_cache_limit(0)
            .with_file_statistics_cache_limit(0)
            .with_list_files_cache_limit(0)
    }
    /// Constant-space aggregate inspection, never an entry enumeration.
    pub fn report(&self) -> Vec<CacheReport> {
        let mut reports = vec![self.syntax.report()];
        for component in self.components() {
            reports.extend(component.report());
        }
        reports
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
