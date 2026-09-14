// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A sealed list. Catalog membership cannot change during a query.

use datafusion::catalog::{CatalogProvider, CatalogProviderList};
use std::collections::BTreeMap;
use std::sync::Arc;

/// The complete catalog set supplied before the session becomes queryable.
#[derive(Debug)]
pub struct SnapshotCatalogList {
    catalogs: BTreeMap<String, Arc<dyn CatalogProvider>>,
}
impl SnapshotCatalogList {
    /// Seal the supplied inventory. No later registration mutates it.
    pub fn new(catalogs: BTreeMap<String, Arc<dyn CatalogProvider>>) -> Self {
        Self { catalogs }
    }
}
impl CatalogProviderList for SnapshotCatalogList {
    // The upstream signature has no error channel. The immutable implementation leaves
    // membership unchanged; platform entry points reject every mutation plan beforehand.
    fn register_catalog(
        &self,
        _name: String,
        _catalog: Arc<dyn CatalogProvider>,
    ) -> Option<Arc<dyn CatalogProvider>> {
        None
    }
    fn catalog_names(&self) -> Vec<String> {
        self.catalogs.keys().cloned().collect()
    }
    fn catalog(&self, name: &str) -> Option<Arc<dyn CatalogProvider>> {
        self.catalogs.get(name).cloned()
    }
}
