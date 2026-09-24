// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Resolved schema scopes derived from the bound provider inventory.

use super::schema::SnapshotSchema;
use datafusion::catalog::{CatalogProvider, SchemaProvider, TableProvider};
use std::collections::BTreeMap;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

/// A private catalog projected from captured bindings for one operation attempt.
#[derive(Debug)]
pub struct SnapshotCatalog {
    schemas: RwLock<BTreeMap<String, Arc<dyn SchemaProvider>>>,
    generation: Arc<AtomicU64>,
    frozen: AtomicBool,
}
impl SnapshotCatalog {
    pub(crate) fn from_tables<'a>(
        tables: &BTreeMap<(String, String), Arc<dyn TableProvider>>,
        default_schema: Option<&str>,
        declared_schemas: impl Iterator<Item = &'a str>,
        generation: Arc<AtomicU64>,
    ) -> Self {
        let namespaces: std::collections::BTreeSet<_> = tables
            .keys()
            .map(|(schema, _)| schema.clone())
            .chain(default_schema.map(str::to_owned))
            .chain(declared_schemas.map(str::to_owned))
            .collect();
        let schemas = namespaces
            .into_iter()
            .map(|namespace| {
                let selected = tables
                    .iter()
                    .filter(|((ns, _), _)| ns == &namespace)
                    .map(|((_, name), table)| (name.clone(), Arc::clone(table)))
                    .collect();
                let schema: Arc<dyn SchemaProvider> =
                    Arc::new(SnapshotSchema::new(selected, Arc::clone(&generation)));
                (namespace.clone(), schema)
            })
            .collect();
        Self {
            schemas: RwLock::new(schemas),
            generation,
            frozen: AtomicBool::new(false),
        }
    }
    pub(crate) fn fork(&self, generation: Arc<AtomicU64>) -> Self {
        let schemas = self
            .schemas
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|(name, schema)| {
                let schema: Arc<dyn SchemaProvider> =
                    schema.downcast_ref::<SnapshotSchema>().map_or_else(
                        || Arc::clone(schema),
                        |schema| Arc::new(schema.fork(Arc::clone(&generation))),
                    );
                (name.clone(), schema)
            })
            .collect();
        Self {
            schemas: RwLock::new(schemas),
            generation,
            frozen: AtomicBool::new(false),
        }
    }
    pub(crate) fn freeze(&self) {
        self.frozen.store(true, Ordering::Release);
        for schema in self
            .schemas
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
        {
            if let Some(schema) = schema.downcast_ref::<SnapshotSchema>() {
                schema.freeze();
            }
        }
    }
    pub(crate) fn ensure_schema(&self, name: &str) -> Arc<dyn SchemaProvider> {
        Arc::clone(
            self.schemas
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .entry(name.to_owned())
                .or_insert_with(|| {
                    Arc::new(SnapshotSchema::new(
                        BTreeMap::new(),
                        Arc::clone(&self.generation),
                    ))
                }),
        )
    }
}
impl CatalogProvider for SnapshotCatalog {
    fn schema_names(&self) -> Vec<String> {
        self.schemas
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .keys()
            .cloned()
            .collect()
    }
    fn schema(&self, name: &str) -> Option<Arc<dyn SchemaProvider>> {
        self.schemas
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(name)
            .cloned()
    }
    fn register_schema(
        &self,
        name: &str,
        schema: Arc<dyn SchemaProvider>,
    ) -> datafusion::common::Result<Option<Arc<dyn SchemaProvider>>> {
        if self.frozen.load(Ordering::Acquire) {
            return Err(datafusion::common::DataFusionError::Plan(
                "immutable selected catalog".into(),
            ));
        }
        let previous = self
            .schemas
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(name.to_owned(), schema);
        self.generation.fetch_add(1, Ordering::AcqRel);
        Ok(previous)
    }
    fn deregister_schema(
        &self,
        name: &str,
        cascade: bool,
    ) -> datafusion::common::Result<Option<Arc<dyn SchemaProvider>>> {
        if self.frozen.load(Ordering::Acquire) {
            return Err(datafusion::common::DataFusionError::Plan(
                "immutable selected catalog".into(),
            ));
        }
        let mut schemas = self
            .schemas
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !cascade
            && schemas
                .get(name)
                .is_some_and(|schema| !schema.table_names().is_empty())
        {
            return Err(datafusion::common::DataFusionError::Execution(format!(
                "Schema {name} is not empty"
            )));
        }
        let previous = schemas.remove(name);
        if previous.is_some() {
            self.generation.fetch_add(1, Ordering::AcqRel);
        }
        Ok(previous)
    }
}
