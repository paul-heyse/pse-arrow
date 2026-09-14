// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Sealed namespace inventories; lookup performs no I/O.

use super::BoxFut;
use datafusion::catalog::{SchemaProvider, TableProvider};
use datafusion::common::Result;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Tables resolved before session construction. Mutation methods remain unsupported.
#[derive(Debug)]
pub struct SnapshotSchema {
    pub(crate) tables: BTreeMap<String, Arc<dyn TableProvider>>,
}
impl SchemaProvider for SnapshotSchema {
    fn table_names(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }
    fn table_exist(&self, name: &str) -> bool {
        self.tables.contains_key(name)
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
        Box::pin(async move { Ok(self.tables.get(name).cloned()) })
    }
}
