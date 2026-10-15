// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Borrowed lexical lookup and occurrence-owned bindings.
use super::Type;
use std::{collections::BTreeMap, sync::Arc};

/// Read-only access to the current complete lexical type environment.
pub trait TypeLookup {
    /// Resolve a binding, applying the most local scope first.
    fn get(&self, name: &str) -> Option<&Type>;
}
impl TypeLookup for BTreeMap<String, Type> {
    fn get(&self, name: &str) -> Option<&Type> {
        BTreeMap::get(self, name)
    }
}
enum Parent<'a> {
    Borrowed(&'a dyn TypeLookup),
    Shared(Arc<BTreeMap<String, Type>>),
}
/// A lexical frame owns only bindings introduced or refined at this occurrence.
pub struct TypeScope<'a> {
    parent: Parent<'a>,
    upper: Option<&'a dyn TypeLookup>,
    bindings: BTreeMap<String, Type>,
}
impl<'a> TypeScope<'a> {
    /// Borrow an enclosing environment without copying its bindings.
    pub fn new(parent: &'a dyn TypeLookup) -> Self {
        Self {
            parent: Parent::Borrowed(parent),
            upper: None,
            bindings: BTreeMap::new(),
        }
    }
    /// Retain an immutable operation-owned lexical base.
    pub(crate) fn shared(parent: Arc<BTreeMap<String, Type>>) -> Self {
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("modeling.type_overlays", "frames", 1);
        }
        Self {
            parent: Parent::Shared(parent),
            upper: None,
            bindings: BTreeMap::new(),
        }
    }
    /// Borrow an authoritative current overlay above this frame's local bindings.
    pub(crate) fn with_upper(mut self, upper: &'a dyn TypeLookup) -> Self {
        self.upper = Some(upper);
        self
    }
    /// Introduce or replace one occurrence-local binding.
    pub fn insert(&mut self, name: String, ty: Type) -> Option<Type> {
        self.bindings.insert(name, ty)
    }
    /// Introduce bindings with ordinary local shadowing.
    pub fn extend(&mut self, bindings: impl IntoIterator<Item = (String, Type)>) {
        self.bindings.extend(bindings);
    }
    /// Introduce a fallback only when the enclosing environment has no binding.
    pub(crate) fn insert_absent(&mut self, name: String, ty: Type) {
        if self.get(&name).is_none() {
            self.insert(name, ty);
        }
    }
}
impl TypeLookup for TypeScope<'_> {
    fn get(&self, name: &str) -> Option<&Type> {
        self.upper
            .and_then(|upper| upper.get(name))
            .or_else(|| self.bindings.get(name))
            .or_else(|| match &self.parent {
                Parent::Borrowed(parent) => parent.get(name),
                Parent::Shared(parent) => parent.get(name),
            })
    }
}

impl std::fmt::Debug for TypeScope<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TypeScope")
            .field("bindings", &self.bindings)
            .finish_non_exhaustive()
    }
}
