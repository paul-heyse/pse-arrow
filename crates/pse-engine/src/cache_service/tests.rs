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
    let maintenance = Arc::default();
    let left = Namespaced::new(cache.clone(), 1, Arc::clone(&maintenance));
    let right = Namespaced::new(cache, 2, maintenance);
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
fn maintenance_fences_old_views_without_invalidating_escaped_values() {
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(4096));
    let cache = Envelope::<Path, Value>::new("unit", 2048, None, &pool).unwrap();
    let maintenance = Arc::new(pse_columnar::retention::RetentionFence::default());
    let old = Namespaced::new(cache.clone(), 1, maintenance.clone());
    let path = Path::from("part-0.parquet");
    old.put(&path, Value(10));
    let escaped = old.get(&path).unwrap();
    maintenance.clear(|| cache.clear());
    let fresh = Namespaced::new(cache.clone(), 1, maintenance);
    fresh.put(&path, Value(20));
    old.put(&path, Value(99));
    old.remove(&path);
    old.clear();
    old.update_cache_limit(0);
    old.update_cache_ttl(Some(Duration::ZERO));
    old.drop_table_entries(&datafusion::common::TableReference::bare("table"))
        .unwrap();
    assert_eq!(old.get(&path), None);
    assert!(!old.contains_key(&path));
    assert_eq!(old.len(), 0);
    assert!(old.list_entries().is_empty());
    assert_eq!(fresh.get(&path), Some(Value(20)));
    assert_eq!(fresh.cache_limit(), 2048);
    assert_eq!(fresh.cache_ttl(), None);
    assert_eq!(escaped, Value(10));
}

#[test]
fn file_metadata_namespaces_change_with_verified_source_generation() {
    use datafusion::execution::cache::cache_manager::FileMetadata;
    struct Metadata;
    impl FileMetadata for Metadata {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn memory_size(&self) -> usize {
            128
        }
        fn extra_info(&self) -> datafusion::common::HashMap<String, String> {
            datafusion::common::HashMap::new()
        }
    }
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(8192));
    let mut budget = CacheBudget::disabled(1);
    budget.metadata_bytes = 1024;
    budget.inspection_bytes = 4096;
    let service = NativeCacheService::new(budget, &pool).unwrap();
    let root = url::Url::parse("memory://source-generation/table/").unwrap();
    let runtime = Arc::new(RuntimeEnv::default());
    runtime.register_object_store(&root, Arc::new(object_store::memory::InMemory::new()));
    let state = SessionStateBuilder::new().with_runtime_env(runtime).build();
    let old = service
        .bind_state_with_generation(&root, &state, Some("verified-source-one"))
        .unwrap();
    let path = Path::from("same-file.parquet");
    let value = CachedFileMetadataEntry::new(
        object_store::ObjectMeta {
            location: path.clone(),
            size: 12,
            last_modified: std::time::SystemTime::UNIX_EPOCH.into(),
            e_tag: None,
            version: None,
        },
        Arc::new(Metadata),
    );
    old.runtime_env()
        .cache_manager
        .get_file_metadata_cache()
        .put(&path, value);
    let same = service
        .bind_state_with_generation(&root, &state, Some("verified-source-one"))
        .unwrap();
    let replaced = service
        .bind_state_with_generation(&root, &state, Some("verified-source-two"))
        .unwrap();
    let unqualified = service
        .bind_state_with_generation(&root, &state, None)
        .unwrap();
    assert!(
        same.runtime_env()
            .cache_manager
            .get_file_metadata_cache()
            .get(&path)
            .is_some()
    );
    assert!(
        replaced
            .runtime_env()
            .cache_manager
            .get_file_metadata_cache()
            .get(&path)
            .is_none()
    );
    assert!(
        unqualified
            .runtime_env()
            .cache_manager
            .get_file_metadata_cache()
            .get(&path)
            .is_none()
    );
    assert!(
        old.runtime_env()
            .cache_manager
            .get_file_metadata_cache()
            .get(&path)
            .is_some()
    );
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
fn file_cache_activation_checks_headroom_before_reserving() {
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(512));
    let mut policy = CacheBudget::disabled(1);
    policy.metadata_bytes = 256;
    policy.statistics_bytes = 512;
    let service = NativeCacheService::new(policy, &pool).unwrap();
    assert!(matches!(
        service.activate(FileCacheDemand::MetadataStatistics),
        Err(datafusion::common::DataFusionError::ResourcesExhausted(_))
    ));
    assert!(service.file_caches().metadata.is_none());
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
    assert_eq!(
        pool.reserved(),
        0,
        "inspection does not activate file capacity"
    );
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

#[test]
fn lazy_file_caches_default_and_inspection_remain_inactive() {
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(1 << 30));
    let policy = CacheBudget::for_memory(1 << 30);
    let service = NativeCacheService::new(policy.clone(), &pool).unwrap();
    assert_eq!(pool.reserved(), policy.syntax_bytes);
    let before = service.report();
    assert!(service.inspect_entries(1).unwrap().rows().is_empty());
    service.invalidate();
    assert_eq!(service.report(), before);
    for (name, limit) in [
        ("pse.cache.metadata", policy.metadata_bytes),
        ("pse.cache.statistics", policy.statistics_bytes),
        ("pse.cache.listing", policy.listing_bytes),
    ] {
        let row = before.iter().find(|row| row.name == name).unwrap();
        assert_eq!(row.policy_limit_bytes, limit);
        assert_eq!(
            (row.capacity_bytes, row.retained_bytes, row.entries),
            (0, 0, 0)
        );
    }
    assert_eq!(pool.reserved(), policy.syntax_bytes);
}

#[test]
fn lazy_file_caches_qualified_binding_activates_only_demand_once() {
    use datafusion::execution::{runtime_env::RuntimeEnv, session_state::SessionStateBuilder};
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(16384));
    let mut policy = CacheBudget::disabled(1024);
    policy.metadata_bytes = 1024;
    policy.statistics_bytes = 2048;
    policy.listing_bytes = 4096;
    policy.inspection_bytes = 4096;
    let service = NativeCacheService::new(policy, &pool).unwrap();
    let root = url::Url::parse("memory:///lazy").unwrap();
    let runtime = Arc::new(RuntimeEnv::default());
    runtime.register_object_store(&root, Arc::new(object_store::memory::InMemory::new()));
    let state = SessionStateBuilder::new().with_runtime_env(runtime).build();
    let unqualified = service
        .bind_state_with_generation(&root, &state, None)
        .unwrap();
    assert_eq!(pool.reserved(), 0);
    assert!(service.file_caches().metadata.is_none());
    assert!(
        unqualified
            .runtime_env()
            .cache_manager
            .get_metadata_cache_limit()
            == 0
    );
    let first = service
        .bind_state_with_generation(&root, &state, Some("v1"))
        .unwrap();
    let capacity = pool.reserved();
    assert!(first.runtime_env().cache_manager.get_metadata_cache_limit() == 1024);
    assert!(service.file_caches().listing.is_none());
    assert_eq!(
        service
            .report()
            .iter()
            .find(|row| row.name == "pse.cache.listing")
            .unwrap()
            .capacity_bytes,
        0
    );
    let original = service.file_caches();
    let second = service
        .bind_state_with_generation(&root, &state, Some("v1"))
        .unwrap();
    assert_eq!(pool.reserved(), capacity);
    assert!(Arc::ptr_eq(
        original.metadata.as_ref().unwrap(),
        service.file_caches().metadata.as_ref().unwrap()
    ));
    assert!(
        second
            .runtime_env()
            .cache_manager
            .get_metadata_cache_limit()
            == 1024
    );
    let listed = service.bind_state(&root, &state).unwrap();
    assert!(
        listed
            .runtime_env()
            .cache_manager
            .get_list_files_cache()
            .is_some()
    );
    assert_eq!(
        service
            .report()
            .iter()
            .find(|row| row.name == "pse.cache.listing")
            .unwrap()
            .capacity_bytes,
        4096
    );
    let reserved = pool.reserved();
    service.invalidate();
    assert_eq!(
        pool.reserved(),
        reserved,
        "clear evicts entries while retaining activated capacity"
    );
}

#[test]
fn lazy_file_caches_partial_activation_rolls_back_and_retries() {
    use datafusion::execution::memory_pool::MemoryConsumer;
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(4096));
    let mut policy = CacheBudget::disabled(1024);
    policy.metadata_bytes = 1024;
    policy.statistics_bytes = 1024;
    policy.listing_bytes = 1024;
    let service = NativeCacheService::new(policy, &pool).unwrap();
    let pressure = MemoryConsumer::new("lazy:pressure").register(&pool);
    pressure.try_grow(2500).unwrap();
    assert!(
        service
            .activate(FileCacheDemand::MetadataStatistics)
            .is_err()
    );
    assert_eq!(
        pool.reserved(),
        2500,
        "a staged metadata grant rolls back when statistics refuses"
    );
    assert!(service.file_caches().metadata.is_none());
    assert!(service.file_caches().statistics.is_none());
    drop(pressure);
    let first = service
        .activate(FileCacheDemand::MetadataStatistics)
        .unwrap();
    assert_eq!(pool.reserved(), 2048);
    let pressure = MemoryConsumer::new("lazy:upgrade-pressure").register(&pool);
    pressure.try_grow(1500).unwrap();
    assert!(service.activate(FileCacheDemand::WithListing).is_err());
    assert!(service.file_caches().listing.is_none());
    assert!(Arc::ptr_eq(
        first.metadata.as_ref().unwrap(),
        service.file_caches().metadata.as_ref().unwrap()
    ));
    assert_eq!(pool.reserved(), 3548);
    drop(pressure);
    service.activate(FileCacheDemand::WithListing).unwrap();
    assert_eq!(pool.reserved(), 3072);
}

#[test]
fn lazy_file_caches_racing_first_activation_share_capacity_owners() {
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(8192));
    let mut policy = CacheBudget::disabled(1024);
    policy.metadata_bytes = 1024;
    policy.statistics_bytes = 1024;
    let service = NativeCacheService::new(policy, &pool).unwrap();
    let barrier = Arc::new(std::sync::Barrier::new(8));
    let threads = (0..8)
        .map(|_| {
            let service = service.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                service
                    .activate(FileCacheDemand::MetadataStatistics)
                    .unwrap()
            })
        })
        .collect::<Vec<_>>();
    let results = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(pool.reserved(), 2048);
    assert!(results.iter().all(|binding| Arc::ptr_eq(
        results[0].metadata.as_ref().unwrap(),
        binding.metadata.as_ref().unwrap()
    )));
    assert!(service.file_caches().listing.is_none());
}

#[test]
fn lazy_file_caches_failed_qualified_bind_retains_namespace_without_reporting_reuse() {
    use datafusion::execution::{
        memory_pool::MemoryConsumer, runtime_env::RuntimeEnv, session_state::SessionStateBuilder,
    };
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(8192));
    let mut policy = CacheBudget::disabled(1024);
    policy.metadata_bytes = 1024;
    policy.statistics_bytes = 1024;
    policy.inspection_bytes = 4096;
    let service = NativeCacheService::new(policy, &pool).unwrap();
    let root = url::Url::parse("memory:///refused").unwrap();
    let runtime = Arc::new(RuntimeEnv::default());
    runtime.register_object_store(&root, Arc::new(object_store::memory::InMemory::new()));
    let state = SessionStateBuilder::new().with_runtime_env(runtime).build();
    let pressure = MemoryConsumer::new("lazy:qualified-refusal").register(&pool);
    pressure.try_grow(6500).unwrap();
    for _ in 0..2 {
        assert!(matches!(
            service.bind_state_with_generation(&root, &state, Some("v1")),
            Err(datafusion::common::DataFusionError::ResourcesExhausted(_))
        ));
        assert!(service.file_caches().metadata.is_none());
    }
    let namespace = service.store_report();
    assert_eq!(namespace.entries, 1);
    assert_eq!(
        namespace.hits, 0,
        "failed state construction is not successful cache reuse"
    );
    assert_eq!(namespace.misses, 1);
    assert_eq!(pool.reserved(), 6500 + namespace.retained_bytes);
    drop(pressure);
    service
        .bind_state_with_generation(&root, &state, Some("v1"))
        .unwrap();
    assert_eq!(service.store_report().hits, 1);
    assert_eq!(pool.reserved(), 2048 + namespace.retained_bytes);
}
