// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable insertion weights for service-local modeling retention.
use datafusion::execution::cache::{CacheKey, CacheValue};
use quick_cache::{OptionsBuilder, Weighter, unsync::Cache};
use std::sync::{Mutex, MutexGuard};

// Account conservatively for the library's resident entry and indexing metadata.
const ENTRY_OVERHEAD: usize = 128;

struct Snapshot<V> {
    value: V,
    weight: u64,
}

struct InsertionWeighter;
impl<K, V> Weighter<K, Snapshot<V>> for InsertionWeighter {
    fn weight(&self, _key: &K, value: &Snapshot<V>) -> u64 {
        value.weight
    }
}

type Entries<K, V> = Cache<K, Snapshot<V>, InsertionWeighter>;
struct State<K, V> {
    limit: usize,
    entries: Option<Entries<K, V>>,
}

/// Equal keys may own different allocation capacities. Lookups never replace a
/// resident key, and eviction always reads its immutable insertion weight.
pub(crate) struct WeightedCache<K, V> {
    state: Mutex<State<K, V>>,
    name: &'static str,
}

pub(crate) fn entry_weight<K: CacheKey, V: CacheValue>(key: &K, value: &V) -> Option<usize> {
    key.size()
        .checked_add(value.size())?
        .checked_add(size_of::<Snapshot<V>>())?
        .checked_add(ENTRY_OVERHEAD)
}

impl<K: CacheKey, V: CacheValue> WeightedCache<K, V> {
    pub(crate) fn new(limit: usize, name: &'static str) -> Self {
        Self {
            state: Mutex::new(State {
                limit,
                entries: Self::backing(limit),
            }),
            name,
        }
    }

    fn backing(limit: usize) -> Option<Entries<K, V>> {
        if limit == 0 {
            return None;
        }
        // A full hot allocation admits any entry within the byte limit, including
        // after restoring a zero-capacity cache. Ghosts own no retained products.
        let options = OptionsBuilder::new()
            .shards(1)
            .estimated_items_capacity(0)
            .weight_capacity(u64::try_from(limit).ok()?)
            .hot_allocation(1.0)
            .ghost_allocation(0.0)
            .build()
            .ok()?;
        Some(Cache::with_options(
            options,
            InsertionWeighter,
            Default::default(),
            Default::default(),
        ))
    }

    fn state(&self) -> MutexGuard<'_, State<K, V>> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(crate) fn get(&self, key: &K) -> Option<V> {
        self.state()
            .entries
            .as_ref()?
            .get(key)
            .map(|snapshot| snapshot.value.clone())
    }

    /// Reject before library insertion: an overweight replacement must preserve
    /// the old entry, and even the library's transient weight sums must not overflow.
    pub(crate) fn put(&self, key: K, value: V) -> bool {
        let Some(bytes) = entry_weight(&key, &value) else {
            return false;
        };
        let Ok(weight) = u64::try_from(bytes) else {
            return false;
        };
        let mut state = self.state();
        if bytes > state.limit {
            return false;
        }
        let Some(entries) = state.entries.as_mut() else {
            return false;
        };
        if usize::try_from(entries.weight())
            .ok()
            .and_then(|current| current.checked_add(bytes))
            .is_none()
            || entries.weight().checked_add(weight).is_none()
        {
            return false;
        }
        entries.insert(key, Snapshot { value, weight });
        true
    }

    pub(crate) fn len(&self) -> usize {
        self.state().entries.as_ref().map_or(0, Cache::len)
    }

    pub(crate) fn memory_used(&self) -> usize {
        self.state().entries.as_ref().map_or(0, |entries| {
            usize::try_from(entries.weight()).unwrap_or(usize::MAX)
        })
    }

    pub(crate) fn cache_limit(&self) -> usize {
        self.state().limit
    }

    pub(crate) fn name(&self) -> String {
        self.name.into()
    }

    pub(crate) fn clear(&self) {
        if let Some(entries) = self.state().entries.as_mut() {
            entries.clear();
        }
    }

    pub(crate) fn update_cache_limit(&self, limit: usize) {
        let mut state = self.state();
        state.limit = limit;
        if limit == 0 {
            // Dropping the backing also synchronously releases all resident and
            // metadata storage; no zero-capacity ratio survives restoration.
            state.entries = None;
        } else if let Some(entries) = state.entries.as_mut() {
            if let Ok(capacity) = u64::try_from(limit) {
                entries.set_capacity(capacity);
            }
        } else {
            state.entries = Self::backing(limit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::common::TableReference;
    use std::sync::Arc;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct CapacityKey(Arc<Vec<u8>>);
    impl CacheKey for CapacityKey {
        fn size(&self) -> usize {
            size_of::<Self>() + self.0.capacity()
        }
        fn table_ref(&self) -> Option<&TableReference> {
            None
        }
    }
    #[derive(Clone, Debug)]
    struct Value {
        owner: Arc<Vec<u8>>,
        bytes: usize,
    }
    impl CacheValue for Value {
        fn size(&self) -> usize {
            self.bytes
        }
    }
    fn key(id: u8, capacity: usize) -> CapacityKey {
        let mut data = Vec::with_capacity(capacity);
        data.push(id);
        CapacityKey(Arc::new(data))
    }
    fn value(bytes: usize) -> Value {
        Value {
            owner: Arc::new(vec![7]),
            bytes,
        }
    }
    fn cache(limit: usize) -> WeightedCache<CapacityKey, Value> {
        WeightedCache::new(limit, "test:weighted-modeling")
    }

    #[test]
    fn equal_lookup_and_removal_use_the_resident_key_and_captured_weight() {
        let cache = cache(4096);
        let resident = key(1, 8);
        let lookup = key(1, 512);
        let product = value(64);
        let weight = entry_weight(&resident, &product).unwrap();
        assert!(cache.put(resident.clone(), product));
        for _ in 0..8 {
            assert!(cache.get(&lookup).is_some());
            assert_eq!(cache.memory_used(), weight);
        }
        let mut state = cache.state();
        let entries = state.entries.as_mut().unwrap();
        let (removed, snapshot) = entries.remove(&lookup).unwrap();
        assert!(Arc::ptr_eq(&removed.0, &resident.0));
        assert_eq!(snapshot.weight, weight as u64);
        assert_eq!(entries.weight(), 0);
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn equal_replacement_captures_the_new_key_and_value_weight() {
        let cache = cache(4096);
        let first = key(1, 8);
        let second = key(1, 512);
        let product = value(96);
        assert!(cache.put(first.clone(), value(64)));
        assert!(cache.get(&second).is_some());
        let weight = entry_weight(&second, &product).unwrap();
        assert!(cache.put(second.clone(), product));
        assert_eq!(cache.memory_used(), weight);
        assert_eq!(cache.len(), 1);
        let mut state = cache.state();
        let entries = state.entries.as_mut().unwrap();
        let (removed, _) = entries.remove(&first).unwrap();
        assert!(Arc::ptr_eq(&removed.0, &second.0));
        assert_eq!(entries.weight(), 0);
    }

    #[test]
    fn equal_lookup_then_eviction_releases_the_original_key_weight() {
        let resident = key(1, 8);
        let lookup = key(1, 512);
        let product = value(64);
        let weight = entry_weight(&resident, &product).unwrap();
        let cache = cache(weight);
        assert!(cache.put(resident, product.clone()));
        assert!(cache.get(&lookup).is_some());
        assert!(cache.put(key(2, 8), product));
        assert_eq!(cache.memory_used(), weight);
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&lookup).is_none());
        cache.update_cache_limit(0);
        assert_eq!(cache.memory_used(), 0);
    }

    #[test]
    fn oversized_or_overflowing_replacement_preserves_the_old_entry() {
        let resident = key(1, 8);
        let product = value(64);
        let weight = entry_weight(&resident, &product).unwrap();
        let cache = cache(weight);
        assert!(cache.put(resident.clone(), product.clone()));
        assert!(!cache.put(key(1, 512), value(64)));
        assert!(!cache.put(resident.clone(), value(usize::MAX)));
        assert_eq!(cache.memory_used(), weight);
        assert_eq!(cache.len(), 1);
        assert!(Arc::ptr_eq(
            &cache.get(&resident).unwrap().owner,
            &product.owner,
        ));
    }

    #[test]
    fn transient_weight_overflow_is_rejected_before_library_insertion() {
        let resident = key(1, 8);
        let overhead = entry_weight(&resident, &value(0)).unwrap();
        let product = value(usize::MAX - overhead);
        let cache = cache(usize::MAX);
        assert!(cache.put(resident.clone(), product.clone()));
        assert_eq!(cache.memory_used(), usize::MAX);
        assert!(!cache.put(key(2, 8), value(1)));
        assert!(!cache.put(resident.clone(), value(1)));
        assert_eq!(cache.memory_used(), usize::MAX);
        assert!(Arc::ptr_eq(
            &cache.get(&resident).unwrap().owner,
            &product.owner,
        ));
        cache.clear();
        assert_eq!(cache.memory_used(), 0);
    }

    #[test]
    fn partial_and_zero_trim_release_entries_and_restore_full_capacity() {
        let one = key(1, 8);
        let two = key(2, 8);
        let product = value(64);
        let weight = entry_weight(&one, &product).unwrap();
        let cache = cache(2 * weight);
        assert!(cache.put(one, product.clone()));
        assert!(cache.put(two, product.clone()));
        cache.update_cache_limit(weight);
        assert_eq!(cache.cache_limit(), weight);
        assert_eq!(cache.memory_used(), weight);
        assert_eq!(cache.len(), 1);
        cache.update_cache_limit(0);
        assert_eq!(cache.cache_limit(), 0);
        assert_eq!(cache.memory_used(), 0);
        assert_eq!(cache.len(), 0);
        assert!(cache.state().entries.is_none());
        assert!(!cache.put(key(3, 8), product.clone()));
        cache.update_cache_limit(weight);
        // An entry using exactly the restored limit must fit (no 97% hot target).
        assert!(cache.put(key(3, 8), product));
        assert_eq!(cache.memory_used(), weight);
        assert_eq!(cache.len(), 1);
        cache.clear();
        assert_eq!(cache.memory_used(), 0);
        assert_eq!(cache.cache_limit(), weight);
    }

    #[test]
    fn trim_and_clear_release_only_cache_references_to_escaped_products() {
        for zero_trim in [false, true] {
            let cache = cache(4096);
            let lookup = key(1, 8);
            let product = value(64);
            let weak = Arc::downgrade(&product.owner);
            assert!(cache.put(lookup.clone(), product));
            let escaped = cache.get(&lookup).unwrap();
            if zero_trim {
                cache.update_cache_limit(0);
            } else {
                cache.clear();
            }
            assert_eq!(cache.memory_used(), 0);
            assert_eq!(Arc::strong_count(&escaped.owner), 1);
            assert!(weak.upgrade().is_some());
            assert_eq!(*escaped.owner, vec![7]);
            drop(escaped);
            assert!(weak.upgrade().is_none());
        }
    }
}
