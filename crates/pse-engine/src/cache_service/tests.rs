// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use datafusion::execution::{
    cache::{Cache, CacheValue},
    memory_pool::GreedyMemoryPool,
};
use object_store::path::Path;
use std::{sync::atomic::Ordering, time::Duration};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Value(usize);
impl CacheValue for Value {
    fn size(&self) -> usize {
        self.0
    }
}

#[test]
fn native_eviction_resize_clear_and_owner_drop_preserve_the_envelope() {
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(2048));
    let cache = Envelope::<Path, Value>::new("unit", 1024, None, &pool).unwrap();
    assert_eq!(pool.reserved(), 1024);
    let a = Path::from("a");
    let b = Path::from("b");
    cache.put(&a, Value(500));
    cache.put(&b, Value(500));
    assert!(cache.get(&a).is_none());
    assert_eq!(cache.get(&b), Some(Value(500)));
    cache.update_cache_limit(4096);
    assert_eq!(cache.cache_limit(), 1024);
    cache.put(&b, Value(usize::MAX));
    assert!(cache.is_empty());
    assert_eq!(pool.reserved(), 1024);
    cache.update_cache_limit(512);
    cache.put(&a, Value(50));
    cache.clear();
    assert_eq!(cache.report().retained_bytes, 0);
    assert_eq!(pool.reserved(), 1024);
    let retained = cache.clone();
    drop(cache);
    assert_eq!(pool.reserved(), 1024);
    drop(retained);
    assert_eq!(pool.reserved(), 0);
}

#[test]
fn ttl_cannot_be_extended_beyond_declared_freshness() {
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(2048));
    let cache = Envelope::<Path, Value>::new("unit", 1024, Some(Duration::ZERO), &pool).unwrap();
    cache.update_cache_ttl(None);
    assert_eq!(cache.cache_ttl(), Some(Duration::ZERO));
    cache.put(&Path::from("a"), Value(1));
    assert_eq!(cache.get(&Path::from("a")), None);
}

#[tokio::test]
async fn snapshot_and_resident_loads_share_admission_and_release_on_drop() {
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(8192));
    let mut policy = CacheBudget::disabled(1);
    policy.concurrent_loads = 1.try_into().unwrap();
    policy.inflight_bytes = 1024;
    let service = NativeCacheService::new(policy, &pool).unwrap();
    let snapshots = load::LoadCounters::default();
    let residents = load::LoadCounters::default();
    let first = service.admit_load(&snapshots).await.unwrap();
    let pending = service.admit_load(&residents);
    tokio::pin!(pending);
    assert!(futures_util::poll!(&mut pending).is_pending());
    assert_eq!(pool.reserved(), 1024);
    drop(first);
    let second = pending.await.unwrap();
    assert_eq!(snapshots.active.load(Ordering::Acquire), 0);
    assert_eq!(residents.bytes.load(Ordering::Acquire), 1024);
    drop(second);
    assert_eq!(pool.reserved(), 0);
}

#[test]
fn bounded_inspection_does_not_touch_lru_counters_or_allocate_when_refused() {
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(8192));
    let policy = CacheBudget::disabled(1);
    let service = NativeCacheService::new(policy, &pool).unwrap();
    let before = service.report();
    assert!(service.inspect_entries(0).unwrap().rows().is_empty());
    assert!(service.inspect_entries(1).is_err());
    assert_eq!(service.report(), before);
    assert_eq!(pool.reserved(), 0, "inspection releases its staging owner");
    let mut policy = CacheBudget::disabled(1);
    policy.inspection_bytes = 4096;
    let service = NativeCacheService::new(policy, &pool).unwrap();
    let before = service.report();
    assert!(service.inspect_entries(1).unwrap().rows().is_empty());
    assert_eq!(service.report(), before);
}
