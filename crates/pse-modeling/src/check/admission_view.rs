// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! A single borrowed authority for source and synthesized function admission.
use super::{CheckedPackage, Function};
use crate::{DeclarationId, PhysicalOperation, Type};
use pse_authoring::dsl::PathSegment;
use pse_ids::SemanticId;
use std::{borrow::Cow, collections::BTreeMap, ops::Deref};

/// Read-only package authority with operation-owned finite function signatures.
pub trait FunctionAdmission {
    /// Immutable source package and its non-function inventories.
    fn package(&self) -> &CheckedPackage;
    /// Current function owner, including finite synthesized functions.
    fn function(&self, id: DeclarationId) -> Option<&Function>;
    /// Resolve an unindexed lexical or synthesized function name.
    fn resolve(&self, owner: DeclarationId, name: &str) -> Option<DeclarationId> {
        self.package().resolve(owner, name)
    }
    /// Resolve parsed atomic segments through the same function authority.
    fn resolve_segments(
        &self,
        owner: DeclarationId,
        segments: &[PathSegment],
    ) -> Option<DeclarationId> {
        self.package().resolve_segments(owner, segments)
    }
    /// Result contract seen by this body; reconstruction overrides are local.
    fn function_result<'a>(&self, function: &'a Function) -> Cow<'a, Type> {
        Cow::Borrowed(&function.result)
    }
}
impl Deref for dyn FunctionAdmission + '_ {
    type Target = CheckedPackage;
    fn deref(&self) -> &Self::Target {
        self.package()
    }
}
impl FunctionAdmission for CheckedPackage {
    fn package(&self) -> &CheckedPackage {
        self
    }
    fn function(&self, id: DeclarationId) -> Option<&Function> {
        self.functions.get(&id)
    }
}
/// Borrowed source package plus the current specialization's finite function owner.
#[derive(Debug)]
pub struct FunctionContracts<'a> {
    package: &'a CheckedPackage,
    functions: &'a BTreeMap<String, Function>,
    reconstruction: Option<&'a PhysicalOperation>,
}
impl<'a> FunctionContracts<'a> {
    pub(crate) fn new(
        package: &'a CheckedPackage,
        functions: &'a BTreeMap<String, Function>,
    ) -> Self {
        Self {
            package,
            functions,
            reconstruction: None,
        }
    }
    pub(crate) fn for_reconstruction(&self, operation: Option<&'a PhysicalOperation>) -> Self {
        Self {
            package: self.package,
            functions: self.functions,
            reconstruction: operation
                .filter(|operation| matches!(operation, PhysicalOperation::Reconstruction { .. })),
        }
    }
    /// Resolve against this specialization's current finite function owner.
    pub fn resolve(&self, owner: DeclarationId, name: &str) -> Option<DeclarationId> {
        FunctionAdmission::resolve(self, owner, name)
    }
    /// Resolve atomic path segments against the same finite function owner.
    pub fn resolve_segments(
        &self,
        owner: DeclarationId,
        segments: &[PathSegment],
    ) -> Option<DeclarationId> {
        FunctionAdmission::resolve_segments(self, owner, segments)
    }
    /// Read a source or synthesized function without copying its package.
    pub fn function(&self, id: DeclarationId) -> Option<&Function> {
        FunctionAdmission::function(self, id)
    }
    fn synthesized(&self, name: &str) -> Option<DeclarationId> {
        name.strip_prefix("f_")
            .and_then(|hex| SemanticId::parse_hex(hex).ok())
            .map(DeclarationId::from)
            .filter(|id| {
                self.functions
                    .contains_key(&format!("f_{}", id.as_id().to_hex()))
            })
    }
}
impl Deref for FunctionContracts<'_> {
    type Target = CheckedPackage;
    fn deref(&self) -> &Self::Target {
        self.package
    }
}
impl FunctionAdmission for FunctionContracts<'_> {
    fn package(&self) -> &CheckedPackage {
        self.package
    }
    fn function(&self, id: DeclarationId) -> Option<&Function> {
        self.functions
            .get(&format!("f_{}", id.as_id().to_hex()))
            .or_else(|| self.package.functions.get(&id))
    }
    fn resolve(&self, owner: DeclarationId, name: &str) -> Option<DeclarationId> {
        self.synthesized(name)
            .or_else(|| self.package.resolve(owner, name))
    }
    fn resolve_segments(
        &self,
        owner: DeclarationId,
        segments: &[PathSegment],
    ) -> Option<DeclarationId> {
        if let [segment] = segments
            && segment.indices.is_empty()
            && let Some(id) = self.synthesized(&segment.name)
        {
            return Some(id);
        }
        self.package.resolve_segments(owner, segments)
    }
    fn function_result<'a>(&self, function: &'a Function) -> Cow<'a, Type> {
        if let Some(operation) = self.reconstruction {
            return operation.body_function_result(&function.result);
        }
        Cow::Borrowed(&function.result)
    }
}
