// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native byte-LRU/TTL under a pool-reserved, non-growing capacity envelope.
//! Native values may escape through Arc clones; this owns retained capacity, not
//! every allocation inside an upstream reader. Reports keep that distinction.

use datafusion::{
    common::{HashMap, Result, TableReference},
    execution::{
        cache::{Cache, CacheEntryInfo, CacheKey, CacheValue, default_cache::DefaultCache},
        memory_pool::{MemoryConsumer, MemoryPool, MemoryReservation},
    },
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct AccountedKey<K>(K);
impl<K: CacheKey> CacheKey for AccountedKey<K> {
    fn size(&self) -> usize {
        // Native LRU + hit map each retain keys. Include node/hash-table slack.
        self.0.size().saturating_mul(2).saturating_add(256)
    }
    fn table_ref(&self) -> Option<&TableReference> {
        self.0.table_ref()
    }
}

pub(super) struct Envelope<K: CacheKey, V: CacheValue> {
    inner: DefaultCache<AccountedKey<K>, V>,
    reservation: MemoryReservation,
    hits: AtomicUsize,
    misses: AtomicUsize,
    bypasses: AtomicUsize,
    ttl_ceiling: Option<Duration>,
    pub(super) admission: std::sync::Mutex<()>,
}
impl<K: CacheKey, V: CacheValue> std::fmt::Debug for Envelope<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeCacheEnvelope")
            .field("name", &self.name())
            .field("reserved", &self.reservation.size())
            .finish_non_exhaustive()
    }
}
impl<K: CacheKey, V: CacheValue> Envelope<K, V> {
    pub(super) fn new(
        name: &str,
        bytes: usize,
        ttl: Option<Duration>,
        pool: &Arc<dyn MemoryPool>,
    ) -> Result<Arc<Self>> {
        let reservation = MemoryConsumer::new(name).register(pool);
        reservation.try_grow(bytes)?;
        Ok(Arc::new(Self {
            inner: DefaultCache::new_with_ttl(bytes, ttl).with_name(name),
            reservation,
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            bypasses: AtomicUsize::new(0),
            ttl_ceiling: ttl,
            admission: std::sync::Mutex::new(()),
        }))
    }
    pub(super) fn report(&self) -> super::CacheReport {
        super::CacheReport {
            name: self.name(),
            capacity_bytes: self.reservation.size(),
            inflight_bytes: None,
            active_loads: None,
            policy_limit_bytes: self.cache_limit(),
            live_bytes: None,
            pinned_bytes: None,
            evictions: None,
            retained_bytes: self.inner.memory_used(),
            entries: self.inner.len(),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            bypasses: self.bypasses.load(Ordering::Relaxed),
        }
    }
}
impl<K: CacheKey, V: CacheValue> Cache<K, V> for Envelope<K, V> {
    fn get(&self, key: &K) -> Option<V> {
        let value = self.inner.get(&AccountedKey(key.clone()));
        if value.is_some() {
            &self.hits
        } else {
            &self.misses
        }
        .fetch_add(1, Ordering::Relaxed);
        value
    }
    fn put(&self, key: &K, value: V) -> Option<V> {
        let _guard = self.admission.lock().ok()?;
        let key = AccountedKey(key.clone());
        if key
            .size()
            .checked_add(value.size())
            .is_none_or(|size| size > self.cache_limit())
        {
            self.bypasses.fetch_add(1, Ordering::Relaxed);
        }
        // Native addition is unchecked: reject overflow before crossing the seam.
        if key.size().checked_add(value.size()).is_none() {
            return self.inner.remove(&key);
        }
        self.inner.put(&key, value)
    }
    fn remove(&self, key: &K) -> Option<V> {
        self.inner.remove(&AccountedKey(key.clone()))
    }
    fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(&AccountedKey(key.clone()))
    }
    fn len(&self) -> usize {
        self.inner.len()
    }
    fn clear(&self) {
        self.inner.clear();
    }
    fn name(&self) -> String {
        self.inner.name()
    }
    fn cache_limit(&self) -> usize {
        self.inner.cache_limit()
    }
    fn update_cache_limit(&self, limit: usize) {
        // Native setter is infallible. An external request cannot enlarge ownership.
        self.inner
            .update_cache_limit(limit.min(self.reservation.size()));
    }
    fn cache_ttl(&self) -> Option<Duration> {
        self.inner.cache_ttl()
    }
    fn update_cache_ttl(&self, ttl: Option<Duration>) {
        self.inner.update_cache_ttl(match self.ttl_ceiling {
            Some(ceiling) => Some(ttl.map_or(ceiling, |ttl| ttl.min(ceiling))),
            None => ttl,
        });
    }
    fn drop_table_entries(&self, table: &TableReference) -> Result<()> {
        self.inner.drop_table_entries(table)
    }
    fn list_entries(&self) -> HashMap<K, CacheEntryInfo<V>> {
        self.inner
            .list_entries()
            .into_iter()
            .map(|(key, value)| (key.0, value))
            .collect()
    }
}
