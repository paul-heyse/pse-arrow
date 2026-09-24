// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Cache-key namespace views. `ObjectMeta` and storage paths are never rewritten.
use datafusion::{
    common::{HashMap, Result, TableReference},
    execution::cache::{Cache, CacheEntryInfo, CacheKey, CacheValue, TableScopedPath},
};
use object_store::path::Path;
use std::{sync::Arc, time::Duration};

pub(super) trait NamespaceKey: CacheKey {
    fn scoped(&self, prefix: &str) -> Self;
    fn unscoped(&self, prefix: &str) -> Option<Self>;
}
impl NamespaceKey for Path {
    fn scoped(&self, prefix: &str) -> Self {
        Path::from(format!("{prefix}{self}"))
    }
    fn unscoped(&self, prefix: &str) -> Option<Self> {
        self.as_ref().strip_prefix(prefix).map(Path::from)
    }
}
impl NamespaceKey for TableScopedPath {
    fn scoped(&self, prefix: &str) -> Self {
        Self {
            table: self.table.clone(),
            path: self.path.scoped(prefix),
        }
    }
    fn unscoped(&self, prefix: &str) -> Option<Self> {
        Some(Self {
            table: self.table.clone(),
            path: self.path.unscoped(prefix)?,
        })
    }
}

pub(super) struct Namespaced<K: NamespaceKey, V: CacheValue> {
    inner: Arc<dyn Cache<K, V>>,
    prefix: String,
}
impl<K: NamespaceKey, V: CacheValue> Namespaced<K, V> {
    pub(super) fn new(inner: Arc<dyn Cache<K, V>>, generation: usize) -> Arc<Self> {
        Arc::new(Self {
            inner,
            prefix: format!("pse-store-{generation}/"),
        })
    }
}
impl<K: NamespaceKey, V: CacheValue> Cache<K, V> for Namespaced<K, V> {
    fn get(&self, key: &K) -> Option<V> {
        self.inner.get(&key.scoped(&self.prefix))
    }
    fn put(&self, key: &K, value: V) -> Option<V> {
        self.inner.put(&key.scoped(&self.prefix), value)
    }
    fn remove(&self, key: &K) -> Option<V> {
        self.inner.remove(&key.scoped(&self.prefix))
    }
    fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(&key.scoped(&self.prefix))
    }
    // Counts refer to the shared native cache; counting must not clone its inventory.
    fn len(&self) -> usize {
        self.inner.len()
    }
    fn clear(&self) {
        self.inner.clear();
    }
    fn name(&self) -> String {
        format!("{}:{}", self.inner.name(), self.prefix)
    }
    fn cache_limit(&self) -> usize {
        self.inner.cache_limit()
    }
    fn update_cache_limit(&self, limit: usize) {
        self.inner.update_cache_limit(limit);
    }
    fn cache_ttl(&self) -> Option<Duration> {
        self.inner.cache_ttl()
    }
    fn update_cache_ttl(&self, ttl: Option<Duration>) {
        self.inner.update_cache_ttl(ttl);
    }
    fn drop_table_entries(&self, table: &TableReference) -> Result<()> {
        self.inner.drop_table_entries(table)
    }
    fn list_entries(&self) -> HashMap<K, CacheEntryInfo<V>> {
        self.inner
            .list_entries()
            .into_iter()
            .filter_map(|(key, value)| key.unscoped(&self.prefix).map(|key| (key, value)))
            .collect()
    }
}
