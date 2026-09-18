// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A single deployment admission gate for snapshot replay and decoded fills.
use datafusion::execution::memory_pool::{MemoryConsumer, MemoryReservation};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Default)]
pub(super) struct LoadCounters {
    pub(super) active: AtomicUsize,
    pub(super) bytes: AtomicUsize,
}
pub(super) struct LoadGuard<'a> {
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
    pub(super) async fn admit_load<'a>(
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
