// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Platform reservations use the same finite DataFusion pool as query operators.

use std::sync::Arc;

use datafusion_execution::memory_pool::{
    MemoryConsumer, MemoryLimit, MemoryPool, MemoryReservation,
};
use pse_ids::{MemoryReserver, Reservation, ReserveError};

use crate::RuntimeError;

/// Opens attributed, non-spillable claims against a configured shared memory pool.
#[derive(Debug)]
pub struct PoolReserver {
    pool: Arc<dyn MemoryPool>,
    limit: usize,
}

impl PoolReserver {
    /// Admits a finite pool whose arithmetic fits the platform allocation envelope.
    ///
    /// # Errors
    /// [`RuntimeError::ConfigInvalid`] for zero, unbounded, unknown or oversized limits.
    pub fn new(pool: Arc<dyn MemoryPool>) -> Result<Self, RuntimeError> {
        let MemoryLimit::Finite(limit) = pool.memory_limit() else {
            return Err(RuntimeError::ConfigInvalid {
                key: "datafusion.runtime.memory_limit".to_owned(),
                reason: "platform reservations require a declared finite pool".to_owned(),
            });
        };
        if limit == 0 || isize::try_from(limit).is_err() {
            return Err(RuntimeError::ConfigInvalid {
                key: "datafusion.runtime.memory_limit".to_owned(),
                reason: "the finite memory limit must be in 1..=isize::MAX".to_owned(),
            });
        }
        Ok(Self { pool, limit })
    }

    /// Bytes currently claimed by all consumers of this same pool.
    pub fn reserved(&self) -> usize {
        self.pool.reserved()
    }
}

impl MemoryReserver for PoolReserver {
    fn open(&self, owner: &str) -> Box<dyn Reservation> {
        Box::new(PoolReservation {
            inner: MemoryConsumer::new(owner)
                .with_can_spill(false)
                .register(&self.pool),
            limit: self.limit,
        })
    }
}

#[derive(Debug)]
struct PoolReservation {
    inner: MemoryReservation,
    limit: usize,
}

impl PoolReservation {
    fn refused(&self, bytes: usize, detail: &str) -> ReserveError {
        ReserveError::Exhausted {
            owner: self.inner.consumer().name().to_owned(),
            requested: bytes,
            reserved: self.inner.size(),
            limit_hint: format!(
                "datafusion.runtime.memory_limit={} bytes; {detail}",
                self.limit
            ),
        }
    }
}

impl Reservation for PoolReservation {
    fn try_grow(&mut self, bytes: usize) -> Result<(), ReserveError> {
        if self
            .inner
            .size()
            .checked_add(bytes)
            .is_none_or(|size| size > self.limit)
        {
            return Err(self.refused(
                bytes,
                "the requested reservation exceeds the configured pool",
            ));
        }
        self.inner
            .try_grow(bytes)
            .map_err(|error| self.refused(bytes, &error.to_string()))
    }

    fn shrink(&mut self, bytes: usize) {
        self.inner.shrink(bytes.min(self.inner.size()));
    }

    fn size(&self) -> usize {
        self.inner.size()
    }

    fn release(&mut self) {
        self.inner.free();
    }
}
