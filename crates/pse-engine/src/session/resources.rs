// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Child accounting against the same parent allocations, native pool and I/O owners.

use super::EngineSession;
use crate::EngineError;
use datafusion::execution::{
    memory_pool::{MemoryConsumer, MemoryLimit, MemoryPool, MemoryReservation},
    runtime_env::RuntimeEnv,
};
use pse_columnar::ReserveError;
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
impl EngineSession {
    pub(crate) fn execution_scope(&self) -> Result<Self, EngineError> {
        let Some(limit) = self.effective_policy()?.max_bytes else {
            return Ok(self.clone());
        };
        let quota = Arc::new(Quota {
            limit,
            used: AtomicUsize::new(0),
        });
        let mut result = self.clone();
        let model_runtime = self.native.context.runtime_env();
        let parent = self.execution_runtime.as_ref().unwrap_or(&model_runtime);
        let runtime = RuntimeEnv {
            memory_pool: Arc::new(ScopedPool {
                parent: Arc::clone(&parent.memory_pool),
                quota,
            }),
            disk_manager: Arc::clone(&parent.disk_manager),
            cache_manager: Arc::clone(&parent.cache_manager),
            object_store_registry: Arc::clone(&parent.object_store_registry),
        };
        result.pool = Arc::clone(&runtime.memory_pool);
        result.execution_runtime = Some(Arc::new(runtime));
        Ok(result)
    }
}

#[cfg(test)]
mod pivot_unit {
    use super::*;
    use datafusion::execution::memory_pool::GreedyMemoryPool;
    #[test]
    fn native_consumers_share_child_and_parent_limits() {
        let parent: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(128));
        let quota = Arc::new(Quota {
            limit: 96,
            used: AtomicUsize::new(0),
        });
        let pool: Arc<dyn MemoryPool> = Arc::new(ScopedPool {
            parent: parent.clone(),
            quota: quota.clone(),
        });
        let a = MemoryConsumer::new("application").register(&pool);
        let b = MemoryConsumer::new("operator").register(&pool);
        a.try_grow(64).unwrap();
        b.try_grow(32).unwrap();
        assert!(a.try_grow(1).is_err());
        assert_eq!(parent.reserved(), 96);
        drop(a);
        assert_eq!(quota.used.load(Ordering::Acquire), 32);
        drop(b);
        assert_eq!(parent.reserved(), 0);
    }
    #[test]
    fn ancestor_refusal_rolls_back_child_admission() {
        let parent: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(16));
        let quota = Arc::new(Quota {
            limit: 128,
            used: AtomicUsize::new(0),
        });
        let pool: Arc<dyn MemoryPool> = Arc::new(ScopedPool {
            parent: parent.clone(),
            quota: quota.clone(),
        });
        let owner = MemoryConsumer::new("application").register(&pool);
        assert!(owner.try_grow(17).is_err());
        assert_eq!(quota.used.load(Ordering::Acquire), 0);
        assert_eq!(parent.reserved(), 0);
    }
}
