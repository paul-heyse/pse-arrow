// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Child accounting against the same parent allocations, native pool and I/O owners.

use super::SnapshotSession;
use crate::CatalogError;
use datafusion::execution::{
    context::SessionContext,
    memory_pool::{MemoryConsumer, MemoryLimit, MemoryPool, MemoryReservation},
    runtime_env::RuntimeEnv,
    session_state::SessionStateBuilder,
};
use pse_ids::{MemoryReserver, Reservation, ReserveError};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
struct Quota {
    limit: usize,
    used: AtomicUsize,
}
impl Quota {
    fn claim(&self, owner: &str, bytes: usize) -> Result<(), ReserveError> {
        self.used
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                used.checked_add(bytes).filter(|next| *next <= self.limit)
            })
            .map(|_| ())
            .map_err(|used| ReserveError::Exhausted {
                owner: owner.to_owned(),
                requested: bytes,
                reserved: used,
                limit_hint: format!("provider scope limit_bytes={}", self.limit),
            })
    }
    fn release(&self, bytes: usize) {
        self.used.fetch_sub(bytes, Ordering::AcqRel);
    }
}
#[derive(Debug)]
struct ScopedReserver {
    parent: Arc<dyn MemoryReserver>,
    quota: Arc<Quota>,
}
impl MemoryReserver for ScopedReserver {
    fn open(&self, owner: &str) -> Box<dyn Reservation> {
        Box::new(ScopedReservation {
            parent: self.parent.open(owner),
            quota: Arc::clone(&self.quota),
            owner: owner.to_owned(),
        })
    }
}
#[derive(Debug)]
struct ScopedReservation {
    parent: Box<dyn Reservation>,
    quota: Arc<Quota>,
    owner: String,
}
impl Reservation for ScopedReservation {
    fn try_grow(&mut self, bytes: usize) -> Result<(), ReserveError> {
        self.quota.claim(&self.owner, bytes)?;
        if let Err(error) = self.parent.try_grow(bytes) {
            self.quota.release(bytes);
            return Err(error);
        }
        Ok(())
    }
    fn shrink(&mut self, bytes: usize) {
        let bytes = bytes.min(self.parent.size());
        self.parent.shrink(bytes);
        self.quota.release(bytes);
    }
    fn size(&self) -> usize {
        self.parent.size()
    }
    fn release(&mut self) {
        self.shrink(self.size());
    }
}
impl Drop for ScopedReservation {
    fn drop(&mut self) {
        self.release();
    }
}
#[derive(Debug)]
struct ScopedPool {
    parent: Arc<dyn MemoryPool>,
    quota: Arc<Quota>,
}
impl std::fmt::Display for ScopedPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProviderScope({})", self.parent)
    }
}
impl MemoryPool for ScopedPool {
    fn name(&self) -> &'static str {
        "ProviderScope"
    }
    fn register(&self, c: &MemoryConsumer) {
        self.parent.register(c);
    }
    fn unregister(&self, c: &MemoryConsumer) {
        self.parent.unregister(c);
    }
    fn grow(&self, r: &MemoryReservation, bytes: usize) {
        // Native infallible accounting must remain infallible. Subsequent fallible
        // reservations see this usage; this is not an allocator-level hard limit.
        self.parent.grow(r, bytes);
        self.quota.used.fetch_add(bytes, Ordering::AcqRel);
    }
    fn shrink(&self, r: &MemoryReservation, bytes: usize) {
        self.parent.shrink(r, bytes);
        self.quota.release(bytes);
    }
    fn try_grow(&self, r: &MemoryReservation, bytes: usize) -> datafusion::common::Result<()> {
        self.quota
            .claim(r.consumer().name(), bytes)
            .map_err(|error| {
                datafusion::common::DataFusionError::ResourcesExhausted(error.to_string())
            })?;
        if let Err(error) = self.parent.try_grow(r, bytes) {
            self.quota.release(bytes);
            return Err(error);
        }
        Ok(())
    }
    fn reserved(&self) -> usize {
        self.quota.used.load(Ordering::Acquire)
    }
    fn memory_limit(&self) -> MemoryLimit {
        MemoryLimit::Finite(self.quota.limit)
    }
}
impl SnapshotSession {
    pub(crate) fn execution_scope(&self) -> Result<Self, CatalogError> {
        let Some(limit) = self.effective_policy()?.max_bytes else {
            return Ok(self.clone());
        };
        let quota = Arc::new(Quota {
            limit,
            used: AtomicUsize::new(0),
        });
        let mut result = self.clone();
        result.reserver = Arc::new(ScopedReserver {
            parent: Arc::clone(&self.reserver),
            quota: Arc::clone(&quota),
        });
        let state = self.context.state();
        let parent = state.runtime_env();
        let runtime = RuntimeEnv {
            memory_pool: Arc::new(ScopedPool {
                parent: Arc::clone(&parent.memory_pool),
                quota,
            }),
            disk_manager: Arc::clone(&parent.disk_manager),
            cache_manager: Arc::clone(&parent.cache_manager),
            object_store_registry: Arc::clone(&parent.object_store_registry),
        };
        result.context = SessionContext::new_with_state(
            SessionStateBuilder::new_from_existing(state)
                .with_runtime_env(Arc::new(runtime))
                .build(),
        );
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::execution::memory_pool::GreedyMemoryPool;
    use pse_ids::FixedBudget;

    #[test]
    fn native_and_platform_claims_share_the_child_ceiling_and_release_on_drop() {
        let platform = FixedBudget::new(1024);
        let native: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(1024));
        let quota = Arc::new(Quota {
            limit: 128,
            used: AtomicUsize::new(0),
        });
        let reserver = ScopedReserver {
            parent: platform.clone(),
            quota: quota.clone(),
        };
        let pool: Arc<dyn MemoryPool> = Arc::new(ScopedPool {
            parent: native.clone(),
            quota: quota.clone(),
        });
        let mut a = reserver.open("platform");
        let b = MemoryConsumer::new("native").register(&pool);
        a.try_grow(80).unwrap();
        b.try_grow(48).unwrap();
        assert!(a.try_grow(1).is_err());
        assert!(b.try_grow(1).is_err());
        assert_eq!(quota.used.load(Ordering::Acquire), 128);
        drop(a);
        b.try_grow(80).unwrap();
        drop(b);
        assert_eq!(quota.used.load(Ordering::Acquire), 0);
        assert_eq!(platform.reserved(), 0);
        assert_eq!(native.reserved(), 0);
    }

    #[test]
    fn ancestor_refusal_rolls_back_the_child_claim() {
        let platform = FixedBudget::new(16);
        let native: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(16));
        let quota = Arc::new(Quota {
            limit: 128,
            used: AtomicUsize::new(0),
        });
        let reserver = ScopedReserver {
            parent: platform.clone(),
            quota: quota.clone(),
        };
        let pool: Arc<dyn MemoryPool> = Arc::new(ScopedPool {
            parent: native.clone(),
            quota: quota.clone(),
        });
        let mut a = reserver.open("platform");
        let b = MemoryConsumer::new("native").register(&pool);
        assert!(a.try_grow(17).is_err());
        assert!(b.try_grow(17).is_err());
        assert_eq!(quota.used.load(Ordering::Acquire), 0);
        a.try_grow(16).unwrap();
        b.try_grow(16).unwrap();
        assert_eq!(quota.used.load(Ordering::Acquire), 32);
        drop(a);
        drop(b);
        assert_eq!(quota.used.load(Ordering::Acquire), 0);
    }
}
