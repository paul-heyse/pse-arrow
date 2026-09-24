// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Delta snapshot and exact-selection caching over shared engine resources.
#[cfg(test)]
mod delta_journeys;
pub(crate) mod resident;
pub mod snapshot;
#[cfg(test)]
mod store_tests;
use datafusion::{
    common::Result,
    execution::memory_pool::{MemoryPool, MemoryReservation},
};
pub mod metrics;
pub mod policy;
pub mod settings;
pub use policy::DeltaCacheBudget;
use pse_engine::cache_service::{
    CacheComponent, CacheEntryReport, CacheReport, NativeCacheService,
};
use std::sync::Arc;

/// Delta-specific adapters; generic cache ownership lives in the engine.
#[derive(Debug)]
pub struct DeltaCacheService {
    metrics: metrics::Metrics,
    native: Arc<NativeCacheService>,
    policy: DeltaCacheBudget,
    pool: Arc<dyn MemoryPool>,
    snapshots: snapshot::SnapshotCache,
    resident: resident::ResidentCache,
}
impl DeltaCacheService {
    /// Construct all cache families against one native pool.
    /// # Errors
    /// Invalid cache policy or resource admission failure.
    pub fn new(policy: DeltaCacheBudget, pool: &Arc<dyn MemoryPool>) -> Result<Arc<Self>> {
        let native = NativeCacheService::new(policy.native.clone(), pool)?;
        Self::with_native(policy, native)
    }
    /// Attach durable caches to the existing deployment resources.
    /// # Errors
    /// Cache policies disagree or their aggregate exceeds the native pool limit.
    pub fn with_native(
        policy: DeltaCacheBudget,
        native: Arc<NativeCacheService>,
    ) -> Result<Arc<Self>> {
        let pool = native.pool().clone();
        if &policy.native != native.policy() {
            return Err(datafusion::common::DataFusionError::Configuration(
                "Delta and native cache policies refer to different resource settings".into(),
            ));
        }
        if let datafusion::execution::memory_pool::MemoryLimit::Finite(bytes) = pool.memory_limit()
        {
            policy.validate(bytes).map_err(pse_columnar::external)?;
        }
        let service = Arc::new(Self {
            metrics: metrics::Metrics::default(),
            native,
            snapshots: snapshot::SnapshotCache::new(&policy),
            resident: resident::ResidentCache::new(&policy),
            policy,
            pool: pool.clone(),
        });
        let component: Arc<dyn CacheComponent> = service.clone();
        service.native.register_component(&component);
        Ok(service)
    }
    /// Shared generic engine resources.
    pub fn native(&self) -> &Arc<NativeCacheService> {
        &self.native
    }
    /// Actual deployment policy.
    pub fn policy(&self) -> &DeltaCacheBudget {
        &self.policy
    }
    /// Snapshot bytes retained by cache entries and live readers.
    pub fn live_snapshot_bytes(&self) -> usize {
        self.snapshots.live_bytes()
    }
}
impl CacheComponent for DeltaCacheService {
    fn invalidate(&self) {
        self.snapshots.invalidate();
        self.resident.invalidate();
    }
    fn report(&self) -> Vec<CacheReport> {
        vec![self.snapshots.report(), self.resident.report()]
    }
    fn details(
        &self,
        rows: &mut Vec<CacheEntryReport>,
        limit: usize,
        owner: &MemoryReservation,
        budget: usize,
    ) -> Result<()> {
        self.snapshots.details(rows, limit, owner, budget)?;
        self.resident.details(rows, limit, owner, budget)
    }
    fn execution_report(&self) -> Vec<(&'static str, Option<usize>)> {
        self.metrics
            .report()
            .into_iter()
            .chain([
                ("snapshot_loads", Some(self.snapshots.loads())),
                ("resident_loads", Some(self.resident.loads())),
            ])
            .collect()
    }
}
