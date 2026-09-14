// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! An immutable inventory of the seven declared namespaces.

use super::{schema::SnapshotSchema, table::RelationTable};
use crate::{CatalogError, Snapshot};
use datafusion::catalog::{CatalogProvider, SchemaProvider, TableProvider};
use pse_ids::MemoryReserver;
use pse_schema::{Registry, model::Namespace};
use std::collections::BTreeMap;
use std::sync::Arc;

/// A catalog whose exact relation handles are fixed for the session lifetime.
#[derive(Debug)]
pub struct SnapshotCatalog {
    schemas: BTreeMap<String, Arc<dyn SchemaProvider>>,
}
impl SnapshotCatalog {
    /// Construct providers only from admitted members, rechecking their declarations.
    ///
    /// # Errors
    /// Incompatible registry/schema/row contracts at the provider boundary.
    pub fn new(
        snapshot: &Arc<Snapshot>,
        registry: &Registry,
        reserver: &Arc<dyn MemoryReserver>,
    ) -> Result<Self, CatalogError> {
        let mut tables = BTreeMap::new();
        for relation in snapshot.relations().values() {
            let table = RelationTable::new(
                Arc::clone(snapshot),
                relation.contract().canonical.relation_id,
                registry,
                Arc::clone(reserver),
            )?;
            let table: Arc<dyn TableProvider> = Arc::new(table);
            if tables
                .insert(
                    (
                        relation.contract().namespace.clone(),
                        relation.contract().name.clone(),
                    ),
                    table,
                )
                .is_some()
            {
                return Err(CatalogError::Admission {
                    path: "snapshot provider".to_owned(),
                    reason: "several producer ports supply one query table name; bind one explicit output port".to_owned(),
                });
            }
        }
        Ok(Self::from_tables(&tables))
    }
    pub(crate) fn from_tables(tables: &BTreeMap<(String, String), Arc<dyn TableProvider>>) -> Self {
        let schemas = Namespace::ALL
            .into_iter()
            .map(|namespace| {
                let selected = tables
                    .iter()
                    .filter(|((ns, _), _)| ns == namespace.as_str())
                    .map(|((_, name), table)| (name.clone(), Arc::clone(table)))
                    .collect();
                let schema: Arc<dyn SchemaProvider> = Arc::new(SnapshotSchema { tables: selected });
                (namespace.as_str().to_owned(), schema)
            })
            .collect();
        Self { schemas }
    }
}
impl CatalogProvider for SnapshotCatalog {
    fn schema_names(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }
    fn schema(&self, name: &str) -> Option<Arc<dyn SchemaProvider>> {
        self.schemas.get(name).cloned()
    }
}
