// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A single deployment admission gate for snapshot replay and decoded fills.
use datafusion::execution::memory_pool::{MemoryConsumer, MemoryReservation};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Default)]
/// Actual live load concurrency and reserved staging bytes.
pub struct LoadCounters {
    /// Number of admitted loads whose guards remain alive.
    pub active: AtomicUsize,
    /// Bytes reserved by the currently admitted loads.
    pub bytes: AtomicUsize,
}
/// Native staging and semaphore ownership released together on drop.
#[derive(Debug)]
pub struct LoadGuard<'a> {
    _permit: tokio::sync::SemaphorePermit<'a>,
    _reservation: MemoryReservation,
    counters: &'a LoadCounters,
    bytes: usize,
}
impl Drop for LoadGuard<'_> {
    fn drop(&mut self) {
        self.counters.bytes.fetch_sub(self.bytes, Ordering::AcqRel);
        self.counters.active.fetch_sub(1, Ordering::AcqRel);
    }
}
impl super::NativeCacheService {
    /// Acquire deployment load capacity and reserve staging from the shared pool.
    /// # Errors
    /// Closed admission semaphore or shared-pool reservation refusal.
    pub async fn admit_load<'a>(
        &'a self,
        counters: &'a LoadCounters,
    ) -> datafusion::common::Result<LoadGuard<'a>> {
        let permit = self
            .loads
            .acquire()
            .await
            .map_err(|error| datafusion::common::DataFusionError::External(Box::new(error)))?;
        let bytes = self.policy.inflight_bytes / self.policy.concurrent_loads.get();
        let reservation = MemoryConsumer::new("pse.cache.load_staging").register(&self.pool);
        reservation.try_grow(bytes)?;
        counters.active.fetch_add(1, Ordering::AcqRel);
        counters.bytes.fetch_add(bytes, Ordering::AcqRel);
        Ok(LoadGuard {
            _permit: permit,
            _reservation: reservation,
            counters,
            bytes,
        })
    }
}

/// An admitted allowance for the active and prefetched Parquet file readers.
#[derive(Debug)]
pub struct PredicateGuard {
    live: std::sync::Arc<AtomicUsize>,
    bytes: usize,
}
impl Drop for PredicateGuard {
    fn drop(&mut self) {
        self.live.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
impl super::NativeCacheService {
    /// Admit the full concurrent-reader allowance before native IO begins.
    /// # Errors
    /// Aggregate predicate capacity would be exceeded.
    pub fn admit_predicates(&self, bytes: usize) -> datafusion::common::Result<PredicateGuard> {
        self.predicate_live
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current
                    .checked_add(bytes)
                    .filter(|next| *next <= self.policy.predicate_total_bytes)
            })
            .map_err(|_| {
                datafusion::common::DataFusionError::ResourcesExhausted(
                    "aggregate predicate cache allowance exhausted".into(),
                )
            })?;
        Ok(PredicateGuard {
            live: self.predicate_live.clone(),
            bytes,
        })
    }
}

#[cfg(test)]
mod integrated_performance_unit {
    #[test]
    fn aggregate_predicate_admission_counts_all_readers_and_releases_on_drop() {
        let pool: std::sync::Arc<dyn datafusion::execution::memory_pool::MemoryPool> =
            std::sync::Arc::new(datafusion::execution::memory_pool::GreedyMemoryPool::new(
                4096,
            ));
        let mut policy = crate::cache_service::CacheBudget::disabled(1);
        policy.predicate_cache_bytes = 64;
        policy.predicate_total_bytes = 256;
        let service = crate::cache_service::NativeCacheService::new(policy, &pool).unwrap();
        let first = service.admit_predicates(128).unwrap();
        let second = service.admit_predicates(128).unwrap();
        assert!(service.admit_predicates(1).is_err());
        drop(first);
        assert!(service.admit_predicates(129).is_err());
        drop(second);
        assert!(service.admit_predicates(256).is_ok());
    }
}
