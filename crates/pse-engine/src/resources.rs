// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared production resource assembly, also used by isolated development fixtures.
use crate::{
    EngineError,
    cache_service::{CacheBudget, NativeCacheService},
};
use datafusion::execution::{
    memory_pool::{
        FairSpillPool, MemoryConsumer, MemoryLimit, MemoryPool, MemoryReservation,
        PeakRecordingPool, TrackConsumersPool,
    },
    runtime_env::{RuntimeEnv, RuntimeEnvBuilder},
};
use std::{num::NonZeroUsize, path::PathBuf, sync::Arc};

/// Release observation around the actual semaphore, without a second capacity count.
#[derive(Debug)]
pub struct ReleaseNotifyingSemaphore {
    inner: Arc<tokio::sync::Semaphore>,
    released: Arc<tokio::sync::Notify>,
    closed: pse_diagnostics::Cancellation,
}
impl ReleaseNotifyingSemaphore {
    /// Construct the sole semaphore for this capacity owner.
    pub fn new(permits: usize) -> Self {
        Self {
            inner: Arc::new(tokio::sync::Semaphore::new(permits)),
            released: Arc::new(tokio::sync::Notify::new()),
            closed: pse_diagnostics::Cancellation::default(),
        }
    }
    /// Register before attempting admission; notifications grant no capacity.
    pub fn released(&self) -> Arc<tokio::sync::Notify> {
        self.released.clone()
    }
    /// Actual currently free permits, never an independently maintained estimate.
    pub fn available_permits(&self) -> usize {
        self.inner.available_permits()
    }
    /// Close the actual semaphore and wake retrying owners to observe closure.
    pub fn close(&self) {
        self.inner.close();
        self.closed.cancel();
        self.released.notify_waiters();
    }
    /// Whether the sole underlying semaphore has closed.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
    /// Sticky closure observation, independent of which capacity blocked a paired retry.
    pub async fn closed(&self) {
        self.closed.cancelled().await;
    }
    /// Attempt complete ownership without retaining a queued partial acquisition.
    pub fn try_acquire_many_owned(
        self: Arc<Self>,
        permits: u32,
    ) -> Result<ReleasedPermit, tokio::sync::TryAcquireError> {
        self.inner
            .clone()
            .try_acquire_many_owned(permits)
            .map(|permit| ReleasedPermit {
                permit: Some(permit),
                released: Some(self.released.clone()),
            })
    }
    /// Attempt one complete permit.
    pub fn try_acquire_owned(
        self: Arc<Self>,
    ) -> Result<ReleasedPermit, tokio::sync::TryAcquireError> {
        self.try_acquire_many_owned(1)
    }
    /// Await an already sanctioned queued acquisition at the existing semaphore.
    pub async fn acquire_owned(
        self: Arc<Self>,
    ) -> Result<ReleasedPermit, tokio::sync::AcquireError> {
        self.acquire_many_owned(1).await
    }
    /// Await an already sanctioned complete acquisition. Admission policy stays with callers.
    pub async fn acquire_many_owned(
        self: Arc<Self>,
        permits: u32,
    ) -> Result<ReleasedPermit, tokio::sync::AcquireError> {
        self.inner
            .clone()
            .acquire_many_owned(permits)
            .await
            .map(|permit| ReleasedPermit {
                permit: Some(permit),
                released: Some(self.released.clone()),
            })
    }
}

/// Actual capacity remains charged through the last owner, then returns before notification.
#[derive(Debug)]
pub struct ReleasedPermit {
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
    released: Option<Arc<tokio::sync::Notify>>,
}
impl ReleasedPermit {
    /// Retain an existing non-CPU cache admission under the same query lifetime.
    pub(crate) fn unobserved(permit: tokio::sync::OwnedSemaphorePermit) -> Self {
        Self {
            permit: Some(permit),
            released: None,
        }
    }
    /// Actual number of held permits.
    pub fn num_permits(&self) -> usize {
        self.permit
            .as_ref()
            .map_or(0, tokio::sync::OwnedSemaphorePermit::num_permits)
    }
}
impl Drop for ReleasedPermit {
    fn drop(&mut self) {
        drop(self.permit.take());
        if let Some(released) = &self.released {
            released.notify_waiters();
        }
    }
}

/// Release observation around the actual shared allocator. Notifications never grant
/// capacity: admission must retry the underlying pool after each observed release.
#[derive(Debug)]
pub struct ReleaseNotifyingPool {
    inner: Arc<dyn MemoryPool>,
    released: Arc<tokio::sync::Notify>,
}
impl ReleaseNotifyingPool {
    /// Attach release observation without replacing or duplicating allocation authority.
    pub fn new(inner: Arc<dyn MemoryPool>) -> Self {
        Self {
            inner,
            released: Arc::new(tokio::sync::Notify::new()),
        }
    }
    /// Observe shrink/free and consumer departure. Create the notification future before
    /// attempting a reservation; an unpolled future observes `notify_waiters` too.
    pub fn released(&self) -> Arc<tokio::sync::Notify> {
        self.released.clone()
    }
}
impl std::fmt::Display for ReleaseNotifyingPool {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, formatter)
    }
}
impl MemoryPool for ReleaseNotifyingPool {
    fn name(&self) -> &str {
        self.inner.name()
    }
    fn register(&self, consumer: &MemoryConsumer) {
        self.inner.register(consumer);
    }
    fn unregister(&self, consumer: &MemoryConsumer) {
        self.inner.unregister(consumer);
        // FairSpillPool's per-consumer allowance can improve even on a zero-byte departure.
        self.released.notify_waiters();
    }
    fn grow(&self, reservation: &MemoryReservation, additional: usize) {
        self.inner.grow(reservation, additional);
    }
    fn shrink(&self, reservation: &MemoryReservation, shrink: usize) {
        self.inner.shrink(reservation, shrink);
        if shrink != 0 {
            self.released.notify_waiters();
        }
    }
    fn try_grow(
        &self,
        reservation: &MemoryReservation,
        additional: usize,
    ) -> datafusion::common::Result<()> {
        self.inner.try_grow(reservation, additional)
    }
    fn reserved(&self) -> usize {
        self.inner.reserved()
    }
    fn memory_limit(&self) -> MemoryLimit {
        self.inner.memory_limit()
    }
}

/// Deployment CPU admission, shared by synchronous compiler jobs and native queries.
/// A query admits at most the deployment worker capacity; nested work borrows it.
#[derive(Debug)]
pub struct CpuAdmission {
    /// Same semaphore retained by blocking compiler workers.
    pub permits: Arc<ReleaseNotifyingSemaphore>,
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
        let pool: Arc<dyn MemoryPool> = Arc::new(ReleaseNotifyingPool::new(peak.clone()));
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

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn semaphore_release_returns_capacity_before_wake_and_last_owner() {
        let semaphore = Arc::new(ReleaseNotifyingSemaphore::new(2));
        let permit = Arc::new(semaphore.clone().try_acquire_many_owned(2).unwrap());
        let retained = permit.clone();
        let released = semaphore.released();
        let notification = released.notified();
        drop(permit);
        assert_eq!(semaphore.available_permits(), 0);
        drop(retained);
        notification.await;
        assert_eq!(semaphore.available_permits(), 2);
        let all = semaphore.clone().try_acquire_many_owned(2).unwrap();
        assert_eq!(all.num_permits(), 2);
    }
    #[tokio::test]
    async fn semaphore_close_wakes_unpolled_retry_and_refuses_new_ownership() {
        let semaphore = Arc::new(ReleaseNotifyingSemaphore::new(0));
        let released = semaphore.released();
        let notification = released.notified();
        semaphore.close();
        notification.await;
        assert!(matches!(
            semaphore.try_acquire_owned(),
            Err(tokio::sync::TryAcquireError::Closed)
        ));
    }
    #[tokio::test]
    async fn semaphore_closure_is_sticky_without_a_release_or_active_runtime_waiter() {
        let semaphore = Arc::new(ReleaseNotifyingSemaphore::new(1));
        let held = semaphore.clone().try_acquire_owned().unwrap();
        semaphore.close();
        semaphore.closed().await;
        assert_eq!(held.num_permits(), 1);
        assert!(matches!(
            semaphore.clone().try_acquire_owned(),
            Err(tokio::sync::TryAcquireError::Closed)
        ));
        drop(held);
    }
    #[tokio::test]
    async fn release_notification_covers_unpolled_shrink_free_and_unregister() {
        let pool = Arc::new(ReleaseNotifyingPool::new(Arc::new(FairSpillPool::new(64))));
        let erased: Arc<dyn MemoryPool> = pool.clone();
        let released = pool.released();
        let reservation = MemoryConsumer::new("query").register(&erased);
        reservation.try_grow(64).unwrap();
        let notification = released.notified();
        reservation.shrink(16);
        notification.await;
        assert_eq!(erased.reserved(), 48);
        let notification = released.notified();
        reservation.free();
        notification.await;
        let notification = released.notified();
        drop(reservation);
        notification.await;
        assert_eq!(erased.reserved(), 0);
        assert!(matches!(erased.memory_limit(), MemoryLimit::Finite(64)));
    }
    #[tokio::test]
    async fn zero_byte_spillable_departure_wakes_actual_fair_share_grower() {
        let pool = Arc::new(ReleaseNotifyingPool::new(Arc::new(FairSpillPool::new(64))));
        let erased: Arc<dyn MemoryPool> = pool.clone();
        let growing = MemoryConsumer::new("growing-spiller")
            .with_can_spill(true)
            .register(&erased);
        let departing = MemoryConsumer::new("zero-byte-spiller")
            .with_can_spill(true)
            .register(&erased);
        growing.try_grow(32).unwrap();
        assert_eq!(departing.size(), 0);
        let released = pool.released();
        let notification = released.notified();
        assert!(growing.try_grow(16).is_err()); // Two registered spillers: share is32.
        drop(departing); // Reserved bytes remain32, but share becomes64.
        tokio::time::timeout(std::time::Duration::from_secs(1), notification)
            .await
            .unwrap();
        growing.try_grow(16).unwrap();
        assert_eq!(erased.reserved(), 48);
        drop(growing);
        assert_eq!(erased.reserved(), 0);
    }
    #[tokio::test]
    async fn production_runtime_cache_and_platform_use_notifying_pool() {
        let resources = EngineResources::build(
            NonZeroUsize::new(1024).unwrap(),
            NonZeroUsize::new(8).unwrap(),
            std::env::temp_dir(),
            1024,
            CacheBudget::disabled(1024),
        )
        .unwrap();
        assert!(Arc::ptr_eq(&resources.pool, &resources.runtime.memory_pool));
        assert!(resources.pool.is::<ReleaseNotifyingPool>());
        let pool = resources
            .pool
            .downcast_ref::<ReleaseNotifyingPool>()
            .unwrap();
        let released = pool.released();
        let owner = MemoryConsumer::new("escaped-result").register(&resources.pool);
        owner.try_grow(128).unwrap();
        let notification = released.notified();
        drop(owner);
        notification.await;
        assert_eq!(resources.pool.reserved(), 0);
    }
}
