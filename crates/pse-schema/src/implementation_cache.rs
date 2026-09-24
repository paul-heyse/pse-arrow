// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registry-lifetime memoization, independent of any execution library or global registry.
use crate::SchemaError;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex},
};
#[derive(Default)]
pub(crate) struct ImplementationCache {
    owner: Arc<()>,
    values: Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}
impl std::fmt::Debug for ImplementationCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImplementationCache")
            .finish_non_exhaustive()
    }
}
impl ImplementationCache {
    pub(crate) fn owner(&self) -> Arc<()> {
        Arc::clone(&self.owner)
    }
    pub(crate) fn get<T: Any + Send + Sync>(
        &self,
        build: impl FnOnce() -> Result<T, SchemaError>,
    ) -> Result<Arc<T>, SchemaError> {
        let mut values = self
            .values
            .lock()
            .map_err(|_| invalid("implementation owner lock poisoned"))?;
        if let Some(value) = values.get(&TypeId::of::<T>()) {
            return Arc::clone(value)
                .downcast()
                .map_err(|_| invalid("implementation key/type mismatch"));
        }
        let value = Arc::new(build()?);
        values.insert(TypeId::of::<T>(), value.clone());
        Ok(value)
    }
}
fn invalid(reason: &str) -> SchemaError {
    SchemaError::InvalidDeclaration {
        context: "derived implementation".into(),
        reason: reason.into(),
    }
}
