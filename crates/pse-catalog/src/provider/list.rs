// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Faithful native registration in private operation-scoped catalog state.

use datafusion::catalog::{CatalogProvider, CatalogProviderList};
use std::collections::BTreeMap;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicU64, Ordering},
};

/// The complete catalog set supplied before the session becomes queryable.
#[derive(Debug)]
pub struct SnapshotCatalogList {
    catalogs: RwLock<BTreeMap<String, Arc<dyn CatalogProvider>>>,
    generation: Arc<AtomicU64>,
}
impl SnapshotCatalogList {
    /// Create private native state from an already resolved inventory.
    pub fn new(catalogs: BTreeMap<String, Arc<dyn CatalogProvider>>) -> Self {
        Self {
            catalogs: RwLock::new(catalogs),
            generation: Arc::new(AtomicU64::new(0)),
        }
    }
    pub(crate) fn with_generation(
        catalogs: BTreeMap<String, Arc<dyn CatalogProvider>>,
        generation: Arc<AtomicU64>,
    ) -> Self {
        Self {
            catalogs: RwLock::new(catalogs),
            generation,
        }
    }
    /// Namespace mutation witness. It never substitutes for a source revision.
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }
    pub(crate) fn generation_owner(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.generation)
    }
}
impl CatalogProviderList for SnapshotCatalogList {
    fn register_catalog(
        &self,
        name: String,
        catalog: Arc<dyn CatalogProvider>,
    ) -> Option<Arc<dyn CatalogProvider>> {
        let mut catalogs = self
            .catalogs
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let previous = catalogs.insert(name, catalog);
        self.generation.fetch_add(1, Ordering::AcqRel);
        previous
    }
    fn catalog_names(&self) -> Vec<String> {
        self.catalogs
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .keys()
            .cloned()
            .collect()
    }
    fn catalog(&self, name: &str) -> Option<Arc<dyn CatalogProvider>> {
        self.catalogs
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(name)
            .cloned()
    }
}
