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

/// Native table registration in an attempt's private namespace generation.
#[derive(Debug)]
pub struct SnapshotSchema {
    tables: RwLock<BTreeMap<String, Arc<dyn TableProvider>>>,
    generation: Arc<AtomicU64>,
}
impl SnapshotSchema {
    pub(crate) fn new(
        tables: BTreeMap<String, Arc<dyn TableProvider>>,
        generation: Arc<AtomicU64>,
    ) -> Self {
        Self {
            tables: RwLock::new(tables),
            generation,
        }
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
                .map(|table| table.table_type()))
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
                .cloned())
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
        tables.insert(name, table);
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
        Ok(previous)
    }
}
