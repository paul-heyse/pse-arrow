// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit native container facts for recursively declared floating-point domains.

use super::{
    Definition, LocalRule, ProjectionError, ProjectionGraph, ProjectionRecord, ProjectionType,
    Scalar, Shape, TypeRef,
};
use std::collections::{BTreeMap, BTreeSet};

/// Intrinsic native layout recorded only by its checked standard-container owner.
/// Arbitrary record shapes never acquire numeric collection meaning by resemblance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeContainer {
    /// The actual checked binary64 nominal scalar, declared by its source scalar macro.
    NominalFloat64,
    /// The actual `[T; N]` declaration; the existing wire record stores `values`.
    FixedArray {
        /// Actual element's recursively resolved projection.
        element: TypeRef,
        /// Exact source array extent.
        length: u64,
    },
}

/// Implemented only alongside actual checked f64 nominal declarations in this crate.
pub(crate) trait NominalFloat64: ProjectionRecord<Fields = (f64,)> {}

impl ProjectionGraph {
    /// Attach scalar-value meaning from its actual checked source declaration.
    pub(crate) fn nominal_float64<T: NominalFloat64>(
        &mut self,
        root: &TypeRef,
    ) -> Result<(), ProjectionError> {
        let TypeRef::Named(name) = root else {
            return Err(ProjectionError::invalid(
                "nominal scalar",
                "scalar owner is not named",
            ));
        };
        let definition = self
            .definitions
            .get_mut(name)
            .ok_or_else(|| ProjectionError::invalid(name, "nominal scalar owner is unresolved"))?;
        if definition.rust_type != std::any::type_name::<T>()
            || !matches!(&definition.shape, Shape::Record(fields)
                if fields.len() == 1 && fields[0].name == "value" && fields[0].tag == 1
                    && fields[0].ty == TypeRef::Scalar(Scalar::F64Bits))
        {
            return Err(ProjectionError::invalid(
                name,
                "nominal scalar hook must identify its actual f64 declaration",
            ));
        }
        definition.native_container = Some(NativeContainer::NominalFloat64);
        Ok(())
    }
    /// Record intrinsic array meaning only for the actual Rust array declaration.
    /// The generic owner proves the native type; an arbitrary similarly shaped
    /// record can never opt itself into numeric-container traversal.
    pub(super) fn array_container<T: ProjectionType, const N: usize>(
        &mut self,
        root: &TypeRef,
        element: TypeRef,
    ) -> Result<(), ProjectionError> {
        let TypeRef::Named(name) = root else {
            return Err(ProjectionError::invalid(
                "array",
                "array projection is not named",
            ));
        };
        let length = u64::try_from(N)
            .map_err(|_| ProjectionError::invalid(name, "array extent exceeds uint64"))?;
        // The actual generic container owner selects its representation. In
        // particular Vec<u8> is Bytes, while numeric vectors remain repeated.
        let expected = Vec::<T>::projection(self)?;
        let definition = self
            .definitions
            .get_mut(name)
            .ok_or_else(|| ProjectionError::invalid(name, "array owner is unresolved"))?;
        if definition.rust_type != std::any::type_name::<[T; N]>() {
            return Err(ProjectionError::invalid(
                name,
                "intrinsic container must be its actual native array owner",
            ));
        }
        if !matches!(&definition.shape, Shape::Record(fields)
            if fields.len() == 1 && fields[0].name == "values" && fields[0].tag == 1
            && fields[0].ty == expected
            && fields[0].rules == [LocalRule::Length { min: length, max: length }])
        {
            return Err(ProjectionError::invalid(
                name,
                "intrinsic array layout differs from its owner declaration",
            ));
        }
        definition.native_container = Some(NativeContainer::FixedArray { element, length });
        Ok(())
    }
}

pub(super) fn supports_float64_domain(
    ty: &TypeRef,
    definitions: &BTreeMap<String, Definition>,
) -> bool {
    fn visit(
        ty: &TypeRef,
        definitions: &BTreeMap<String, Definition>,
        active: &mut BTreeSet<String>,
    ) -> bool {
        match ty {
            TypeRef::Scalar(Scalar::F64Bits) => true,
            TypeRef::Optional(inner) | TypeRef::Repeated(inner) | TypeRef::Set(inner) => {
                visit(inner, definitions, active)
            }
            TypeRef::Map(_, value) => visit(value, definitions, active),
            TypeRef::Named(name) => {
                if !active.insert(name.clone()) {
                    return false;
                }
                let supported = definitions
                    .get(name)
                    .and_then(|definition| definition.native_container.as_ref())
                    .is_some_and(|container| match container {
                        NativeContainer::NominalFloat64 => true,
                        NativeContainer::FixedArray { element, .. } => {
                            visit(element, definitions, active)
                        }
                    });
                active.remove(name);
                supported
            }
            TypeRef::Scalar(_) => false,
        }
    }
    visit(ty, definitions, &mut BTreeSet::new())
}
