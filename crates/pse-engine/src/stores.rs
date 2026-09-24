// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Instrument each registered native store once, before cache identities are assigned.
use datafusion::{
    common::Result,
    execution::object_store::{DefaultObjectStoreRegistry, ObjectStoreRegistry},
};
use object_store::ObjectStore;
use std::sync::Arc;
use url::Url;
mod admission;

/// Native registry containing the same instrumented stores for DataFusion and Delta.
#[derive(Debug)]
pub struct ObservedStores {
    inner: DefaultObjectStoreRegistry,
    gate: Arc<tokio::sync::Semaphore>,
    wrappers: std::sync::Mutex<Vec<std::sync::Weak<dyn ObjectStore>>>,
}
impl Default for ObservedStores {
    fn default() -> Self {
        Self::new(std::num::NonZeroUsize::MIN)
    }
}
impl ObservedStores {
    /// Share one finite I/O gate across local and subsequently registered stores.
    pub fn new(concurrency: std::num::NonZeroUsize) -> Self {
        let inner = DefaultObjectStoreRegistry::new();
        let gate = Arc::new(tokio::sync::Semaphore::new(concurrency.get()));
        let mut wrappers = Vec::new();
        // The native default registry eagerly installs its one local filesystem.
        let root = datafusion::execution::object_store::ObjectStoreUrl::local_filesystem();
        if let Ok(store) = inner.get_store(root.as_ref()) {
            let store = instrumented_object_store::instrument_object_store(
                Arc::new(admission::SharedIo {
                    inner: store,
                    gate: gate.clone(),
                }),
                "file",
            );
            wrappers.push(Arc::downgrade(&store));
            inner.register_store(root.as_ref(), store);
        }
        Self {
            inner,
            gate,
            wrappers: std::sync::Mutex::new(wrappers),
        }
    }
}
impl ObjectStoreRegistry for ObservedStores {
    fn register_store(
        &self,
        url: &Url,
        store: Arc<dyn ObjectStore>,
    ) -> Option<Arc<dyn ObjectStore>> {
        // Native registry semantics define replacement, so a replacement gets a new
        // actual identity. Reads never manufacture new wrappers or cache namespaces.
        let mut wrappers = self
            .wrappers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        wrappers.retain(|value| value.strong_count() != 0);
        if wrappers
            .iter()
            .any(|value| std::sync::Weak::ptr_eq(value, &Arc::downgrade(&store)))
        {
            return self.inner.register_store(url, store);
        }
        let store = instrumented_object_store::instrument_object_store(
            Arc::new(admission::SharedIo {
                inner: store,
                gate: self.gate.clone(),
            }),
            url.scheme(),
        );
        wrappers.push(Arc::downgrade(&store));
        self.inner.register_store(url, store)
    }
    fn deregister_store(&self, url: &Url) -> Result<Arc<dyn ObjectStore>> {
        self.inner.deregister_store(url)
    }
    fn get_store(&self, url: &Url) -> Result<Arc<dyn ObjectStore>> {
        self.inner.get_store(url)
    }
}

#[cfg(test)]
mod durability_unit {
    use super::*;
    #[tokio::test]
    async fn aliases_and_replacements_retain_exact_store_identity_without_double_gating() {
        let registry = ObservedStores::default();
        let original = Url::parse("memory://one").unwrap();
        let alias = Url::parse("memory://two").unwrap();
        registry.register_store(&original, Arc::new(object_store::memory::InMemory::new()));
        let old = registry.get_store(&original).unwrap();
        registry.register_store(&alias, old.clone());
        assert!(Arc::ptr_eq(&old, &registry.get_store(&alias).unwrap()));
        registry.register_store(&original, Arc::new(object_store::memory::InMemory::new()));
        assert!(!Arc::ptr_eq(&old, &registry.get_store(&original).unwrap()));
        let path = object_store::path::Path::from("test");
        old.put_opts(&path, "a".into(), object_store::PutOptions::default())
            .await
            .unwrap();
        assert_eq!(
            old.get_opts(&path, object_store::GetOptions::default())
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap(),
            "a"
        );
        assert!(
            registry
                .get_store(&original)
                .unwrap()
                .get_opts(&path, object_store::GetOptions::default())
                .await
                .is_err()
        );
    }
}
