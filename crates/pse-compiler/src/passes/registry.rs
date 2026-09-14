// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The pass registry: the implementations behind the declared `PassSpec`s
//! (blueprint §14.1).
//!
//! A pass is a `reference.pass_specs` row first and a Rust type second; this module is
//! where the two are bound together and where a spec without an implementation is caught.
//!
use super::{Pass, dag::invalid};
use crate::CompilerError;
use pse_schema::Registry;
use std::{collections::BTreeMap, sync::Arc};

/// Implementations bound to exact registry declarations, without implicit fallback passes.
#[derive(Default)]
pub struct PassRegistry {
    passes: BTreeMap<&'static str, Arc<dyn Pass>>,
}
impl PassRegistry {
    /// An empty implementation inventory.
    pub fn new() -> Self {
        Self::default()
    }
    /// Register one implementation only when its complete specification matches authority.
    /// # Errors
    /// Unknown/mismatched specification or duplicate implementation.
    pub fn register(
        &mut self,
        pass: Arc<dyn Pass>,
        registry: &Registry,
    ) -> Result<(), CompilerError> {
        let spec = pass.spec();
        if registry.pass(&spec.qualified_name()) != Some(spec) {
            return Err(invalid("implementation differs from declared pass"));
        }
        if self.passes.contains_key(spec.name) {
            return Err(invalid("pass implementation already registered"));
        }
        self.passes.insert(spec.name, pass);
        Ok(())
    }
    /// Resolve an available implementation explicitly.
    /// # Errors
    /// A declaration has no executable implementation in this registry.
    pub fn get(&self, name: &str) -> Result<&Arc<dyn Pass>, CompilerError> {
        self.passes
            .get(name)
            .ok_or_else(|| invalid(format!("pass {name} has no executable implementation")))
    }
}
impl std::fmt::Debug for PassRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PassRegistry")
            .field("passes", &self.passes.keys())
            .finish()
    }
}
