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
    maintenance: Arc<pse_columnar::retention::RetentionFence>,
    epoch: u64,
}
impl<K: NamespaceKey, V: CacheValue> Namespaced<K, V> {
    pub(super) fn new(
        inner: Arc<dyn Cache<K, V>>,
        generation: usize,
        maintenance: Arc<pse_columnar::retention::RetentionFence>,
    ) -> Arc<Self> {
        Arc::new(Self {
            inner,
            prefix: format!("pse-store-{generation}/"),
            epoch: maintenance.generation(),
            maintenance,
        })
    }
}
impl<K: NamespaceKey, V: CacheValue> Cache<K, V> for Namespaced<K, V> {
    fn get(&self, key: &K) -> Option<V> {
        self.maintenance
            .admit(self.epoch, || self.inner.get(&key.scoped(&self.prefix)))
            .flatten()
    }
    fn put(&self, key: &K, value: V) -> Option<V> {
        self.maintenance
            .admit(self.epoch, || {
                self.inner.put(&key.scoped(&self.prefix), value)
            })
            .flatten()
    }
    fn remove(&self, key: &K) -> Option<V> {
        self.maintenance
            .admit(self.epoch, || self.inner.remove(&key.scoped(&self.prefix)))
            .flatten()
    }
    fn contains_key(&self, key: &K) -> bool {
        self.maintenance
            .admit(self.epoch, || {
                self.inner.contains_key(&key.scoped(&self.prefix))
            })
            .unwrap_or(false)
    }
    // Counts refer to the shared native cache; counting must not clone its inventory.
    fn len(&self) -> usize {
        self.maintenance
            .admit(self.epoch, || self.inner.len())
            .unwrap_or(0)
    }
    fn clear(&self) {
        self.maintenance.admit(self.epoch, || self.inner.clear());
    }
    fn name(&self) -> String {
        format!("{}:{}", self.inner.name(), self.prefix)
    }
    fn cache_limit(&self) -> usize {
        self.inner.cache_limit()
    }
    fn update_cache_limit(&self, limit: usize) {
        self.maintenance
            .admit(self.epoch, || self.inner.update_cache_limit(limit));
    }
    fn cache_ttl(&self) -> Option<Duration> {
        self.inner.cache_ttl()
    }
    fn update_cache_ttl(&self, ttl: Option<Duration>) {
        self.maintenance
            .admit(self.epoch, || self.inner.update_cache_ttl(ttl));
    }
    fn drop_table_entries(&self, table: &TableReference) -> Result<()> {
        self.maintenance
            .admit(self.epoch, || self.inner.drop_table_entries(table))
            .unwrap_or(Ok(()))
    }
    fn list_entries(&self) -> HashMap<K, CacheEntryInfo<V>> {
        self.maintenance
            .admit(self.epoch, || self.inner.list_entries())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(key, value)| key.unscoped(&self.prefix).map(|key| (key, value)))
            .collect()
    }
}
