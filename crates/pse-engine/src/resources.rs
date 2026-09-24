// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared production resource assembly, also used by isolated development fixtures.
use crate::{
    EngineError,
    cache_service::{CacheBudget, NativeCacheService},
};
use datafusion::execution::{
    memory_pool::{FairSpillPool, MemoryPool, PeakRecordingPool, TrackConsumersPool},
    runtime_env::{RuntimeEnv, RuntimeEnvBuilder},
};
use std::{num::NonZeroUsize, path::PathBuf, sync::Arc};

/// Deployment CPU admission, shared by synchronous compiler jobs and native queries.
/// A query admits at most the deployment worker capacity; nested work borrows it.
#[derive(Debug)]
pub struct CpuAdmission {
    /// Same semaphore retained by blocking compiler workers.
    pub permits: Arc<tokio::sync::Semaphore>,
    /// Total deployment workers, independent of native task partitioning.
    pub workers: std::num::NonZeroU32,
}

/// One native runtime and finite allocator for execution, caches and platform buffers.
#[derive(Debug)]
pub struct EngineResources {
    /// Shared native environment; session construction never replaces its pool.
    pub runtime: Arc<RuntimeEnv>,
    /// Attributed native consumer accounting.
    pub tracked: Arc<TrackConsumersPool<FairSpillPool>>,
    /// Peak accounting for that same pool.
    pub peak: Arc<PeakRecordingPool>,
    /// Platform reserve-before-allocate adapter over that same pool.
    pub pool: Arc<dyn MemoryPool>,
    /// Native caches sharing the allocator.
    pub caches: Arc<NativeCacheService>,
}
impl EngineResources {
    /// Build finite shared resources. No catalog, filesystem data or solver fixture.
    /// # Errors
    /// Invalid capacity, native resource construction or cache admission failure.
    pub fn build(
        memory: NonZeroUsize,
        consumers: NonZeroUsize,
        spill: PathBuf,
        spill_bytes: u64,
        caches: CacheBudget,
    ) -> Result<Self, EngineError> {
        caches.validate(memory.get())?;
        let tracked = Arc::new(TrackConsumersPool::new(
            FairSpillPool::new(memory.get()),
            consumers,
        ));
        let pool: Arc<dyn MemoryPool> = tracked.clone();
        let peak = Arc::new(PeakRecordingPool::new(pool));
        let pool: Arc<dyn MemoryPool> = peak.clone();
        if isize::try_from(memory.get()).is_err() {
            return Err(EngineError::ConfigInvalid {
                key: "datafusion.runtime.memory_limit".into(),
                reason: "finite memory limit exceeds isize::MAX".into(),
            });
        }
        let io_concurrency = caches.concurrent_loads;
        let caches = NativeCacheService::new(caches, &pool).map_err(crate::session::engine)?;
        let runtime = RuntimeEnvBuilder::new()
            .with_memory_pool(pool.clone())
            .with_cache_manager(NativeCacheService::unbound_config())
            .with_object_store_registry(Arc::new(crate::stores::ObservedStores::new(
                io_concurrency,
            )))
            .with_temp_file_path(spill)
            .with_max_temp_directory_size(spill_bytes)
            .build_arc()
            .map_err(crate::session::engine)?;
        Ok(Self {
            runtime,
            tracked,
            peak,
            pool,
            caches,
        })
    }
}
