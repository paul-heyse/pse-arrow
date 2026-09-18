// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Private attempt namespace; lookup performs no I/O or producer execution.

use super::BoxFut;
use datafusion::catalog::{SchemaProvider, TableProvider};
use datafusion::common::Result;
use std::collections::BTreeMap;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicU64, Ordering},
};

/// A table and its established semantic facts have one owner in the native schema.
#[derive(Clone, Debug)]
enum TableEntry {
    Native(Arc<dyn TableProvider>),
    Bound(Arc<super::binding::TableBinding>),
}
impl TableEntry {
    fn provider(&self) -> &Arc<dyn TableProvider> {
        match self {
            Self::Native(provider) => provider,
            Self::Bound(binding) => &binding.provider,
        }
    }
}

/// Native table registration in an attempt's private namespace generation.
#[derive(Debug)]
pub struct SnapshotSchema {
    tables: RwLock<BTreeMap<String, TableEntry>>,
    generation: Arc<AtomicU64>,
}
impl SnapshotSchema {
    pub(crate) fn new(
        tables: BTreeMap<String, Arc<dyn TableProvider>>,
        generation: Arc<AtomicU64>,
    ) -> Self {
        Self {
            tables: RwLock::new(
                tables
                    .into_iter()
                    .map(|(name, table)| (name, TableEntry::Native(table)))
                    .collect(),
            ),
            generation,
        }
    }
    pub(crate) fn fork(&self, generation: Arc<AtomicU64>) -> Self {
        Self {
            tables: RwLock::new(
                self.tables
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone(),
            ),
            generation,
        }
    }
    pub(crate) fn binding(&self, name: &str) -> Option<Arc<super::binding::TableBinding>> {
        match self
            .tables
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(name)
        {
            Some(TableEntry::Bound(binding)) => Some(Arc::clone(binding)),
            _ => None,
        }
    }
    pub(crate) fn bind(&self, binding: super::binding::TableBinding) {
        self.tables
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                binding.reference.table().to_owned(),
                TableEntry::Bound(Arc::new(binding)),
            );
    }
    pub(crate) fn retain(&self, mut keep: impl FnMut(&str) -> bool) {
        self.tables
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|name, _| keep(name));
    }
}
impl SchemaProvider for SnapshotSchema {
    fn owner_name(&self) -> Option<&str> {
        Some("pse")
    }
    fn table_type<'s, 'n, 'future>(
        &'s self,
        name: &'n str,
    ) -> BoxFut<'future, Result<Option<datafusion::logical_expr::TableType>>>
    where
        's: 'future,
        'n: 'future,
        Self: 'future,
    {
        Box::pin(async move {
            Ok(self
                .tables
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(name)
                .map(|table| table.provider().table_type()))
        })
    }
    fn table_names(&self) -> Vec<String> {
        self.tables
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .keys()
            .cloned()
            .collect()
    }
    fn table_exist(&self, name: &str) -> bool {
        self.tables
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(name)
    }
    fn table<'s, 'n, 'future>(
        &'s self,
        name: &'n str,
    ) -> BoxFut<'future, Result<Option<Arc<dyn TableProvider>>>>
    where
        's: 'future,
        'n: 'future,
        Self: 'future,
    {
        Box::pin(async move {
            Ok(self
                .tables
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(name)
                .map(|table| Arc::clone(table.provider())))
        })
    }
    fn register_table(
        &self,
        name: String,
        table: Arc<dyn TableProvider>,
    ) -> Result<Option<Arc<dyn TableProvider>>> {
        let mut tables = self
            .tables
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if tables.contains_key(&name) {
            return Err(datafusion::common::DataFusionError::Execution(format!(
                "Table already exists: {name}"
            )));
        }
        tables.insert(name, TableEntry::Native(table));
        self.generation.fetch_add(1, Ordering::AcqRel);
        Ok(None)
    }
    fn deregister_table(&self, name: &str) -> Result<Option<Arc<dyn TableProvider>>> {
        let previous = self
            .tables
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(name);
        if previous.is_some() {
            self.generation.fetch_add(1, Ordering::AcqRel);
        }
        Ok(previous.map(|entry| Arc::clone(entry.provider())))
    }
}
