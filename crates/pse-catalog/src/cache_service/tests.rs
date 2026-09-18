// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use datafusion::execution::{
    cache::{Cache, CacheValue},
    memory_pool::GreedyMemoryPool,
};
use std::time::Duration;

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
fn store_namespaces_never_alias_identical_paths() {
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(4096));
    let cache = Envelope::<Path, Value>::new("unit", 2048, None, &pool).unwrap();
    let left = Namespaced::new(cache.clone(), 1);
    let right = Namespaced::new(cache, 2);
    let path = Path::from("part-0.parquet");
    left.put(&path, Value(10));
    assert_eq!(left.get(&path), Some(Value(10)));
    assert_eq!(right.get(&path), None);
    right.put(&path, Value(20));
    assert_eq!(left.get(&path), Some(Value(10)));
    assert_eq!(right.get(&path), Some(Value(20)));
    assert_eq!(left.list_entries().keys().collect::<Vec<_>>(), [&path]);
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

#[test]
fn failed_construction_releases_preceding_reservations() {
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(512));
    let mut policy = CacheBudget::disabled(1);
    policy.metadata_bytes = 256;
    policy.statistics_bytes = 512;
    assert!(NativeCacheService::new(policy, &pool).is_err());
    assert_eq!(pool.reserved(), 0);
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
    let mut policy = CacheBudget::disabled(1);
    policy.metadata_bytes = 1024;
    let service = NativeCacheService::new(policy, &pool).unwrap();
    let before = service.report();
    assert!(service.inspect_entries(0).unwrap().rows().is_empty());
    assert!(service.inspect_entries(1).is_err());
    assert_eq!(service.report(), before);
    assert_eq!(pool.reserved(), 1024);
    let mut policy = CacheBudget::disabled(1);
    policy.metadata_bytes = 1024;
    policy.inspection_bytes = 4096;
    let service = NativeCacheService::new(policy, &pool).unwrap();
    let before = service.report();
    assert!(service.inspect_entries(1).unwrap().rows().is_empty());
    assert_eq!(service.report(), before);
}

#[test]
fn store_bindings_report_their_owned_reservations_and_actual_reuse() {
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(8192));
    let mut policy = CacheBudget::disabled(1);
    policy.metadata_bytes = 1024;
    policy.inspection_bytes = 4096;
    let service = NativeCacheService::new(policy, &pool).unwrap();
    let store: Arc<dyn object_store::ObjectStore> = Arc::new(object_store::memory::InMemory::new());
    let root = url::Url::parse("memory:///first").unwrap();
    assert_eq!(service.generation(&root, Arc::clone(&store)), Some(0));
    assert_eq!(service.generation(&root, Arc::clone(&store)), Some(0));
    let second = url::Url::parse("memory:///second").unwrap();
    assert_eq!(service.generation(&second, store), Some(1));
    let reports = service.report();
    assert_eq!(
        reports
            .iter()
            .map(|report| report.capacity_bytes + report.live_bytes.unwrap_or(0))
            .sum::<usize>(),
        pool.reserved()
    );
    let bindings = reports
        .iter()
        .find(|report| report.name == "pse.cache.store_bindings")
        .unwrap();
    assert_eq!(bindings.entries, 2);
    assert_eq!(bindings.hits, 1);
    assert_eq!(bindings.misses, 2);
    assert_eq!(bindings.bypasses, 0);
    assert_eq!(bindings.live_bytes, Some(bindings.retained_bytes));
    assert!(bindings.retained_bytes > 1024);
    drop(service);
    assert_eq!(pool.reserved(), 0);
}
