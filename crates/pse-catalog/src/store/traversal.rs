// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Algorithm owners shared only within one explicit cold admission traversal.

use crate::CatalogError;
use datafusion::common::extensions::Extensions;
use std::{
    any::Any,
    sync::{Arc, Mutex},
};

/// Native type-keyed extension storage for current admission implementations.
/// A cached value is an optimization, never an admission certificate. Implementations
/// key their entries by actual immutable input owners and retain accounted payloads.
/// Snapshots do not retain this traversal, so temporary caches release on completion.
#[derive(Debug, Default)]
pub struct AdmissionTraversal(Mutex<Extensions>);

impl AdmissionTraversal {
    /// Get the shared implementation-local cache, constructing its empty state once.
    /// This does not execute an operation or establish facts about any input.
    /// # Errors
    /// A previous owner panicked while holding the extension inventory.
    pub fn owner<T: Any + Send + Sync + Default>(&self) -> Result<Arc<T>, CatalogError> {
        let mut owners = self.0.lock().map_err(|_| {
            super::verify::admission("admission traversal", "extension owner lock poisoned")
        })?;
        if let Some(owner) = owners.get_arc::<T>() {
            return Ok(owner);
        }
        let owner = Arc::new(T::default());
        owners.insert_arc(Arc::clone(&owner));
        Ok(owner)
    }
}
