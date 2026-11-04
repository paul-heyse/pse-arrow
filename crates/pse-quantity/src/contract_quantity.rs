// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Trait-owner integration of actual quantity declarations; no copied physical field schema.

use pse_contract::boundary_projection::*;
use crate::boundary::{BoundaryField, BoundaryRecord};

macro_rules! quantity_record {
    ($owner:path; $($field:ident => $tag:literal;)*) => {
        impl ProjectionType for $owner {
            fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
                graph.declare::<Self>(<Self as BoundaryRecord>::NAME,
                    <Self as BoundaryRecord>::SOURCE, <Self as BoundaryRecord>::VERSION,
                    <Self as BoundaryRecord>::DIRECTION, <Self as BoundaryRecord>::RESERVED_TAGS,
                    <Self as BoundaryRecord>::RESERVED_NAMES, |graph| Ok(Shape::Record(vec![$(
                        Field::of::<<Self as BoundaryField<$tag>>::Type>(graph,
                            stringify!($field), $tag, <Self as BoundaryRecord>::DIRECTION, vec![])?,
                    )*])))
            }
        }
        impl ProjectionRecord for $owner {
            const VERSION: u32 = <Self as BoundaryRecord>::VERSION;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[$(($tag, stringify!($field))),*];
            type Fields = <Self as BoundaryRecord>::Fields;
            fn into_projection_fields(self) -> Self::Fields { self.into_boundary_fields() }
            fn from_projection_fields(fields: Self::Fields) -> Result<Self, AdapterError> {
                Self::from_boundary_fields(fields).map_err(|error| AdapterError::refused("", AdapterFailure::InvalidValue, error.to_string()))
            }
        }
        impl ValidateProjection for $owner {
            fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
                $(<Self as BoundaryField<$tag>>::boundary_field(self)
                    .validate_at(&field_path(path, stringify!($field)))?;)*
                Ok(())
            }
        }
    };
}

quantity_type_key_boundary!(quantity_record);
quantity_type_boundary!(quantity_record);
basis_boundary!(quantity_record);
entity_kind_boundary!(quantity_record);
reference_condition_boundary!(quantity_record);
reference_state_boundary!(quantity_record);
unit_factor_boundary!(quantity_record);
unit_boundary!(quantity_record);
quantity_kind_boundary!(quantity_record);
kind_factor_boundary!(quantity_record);
kind_definition_boundary!(quantity_record);
ratio_boundary!(quantity_record);
dimension_vector_boundary!(quantity_record);

macro_rules! quantity_vocabulary {
    ($($ty:ident),* $(,)?) => { $(
        impl ProjectionType for crate::$ty {
            fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
                graph.declare::<Self>(stringify!($ty), "crates/pse-quantity/src/enums.rs", 1,
                    Direction::Both, &[], &[], |_| Ok(Shape::StringEnum(
                        Self::ALL.iter().map(|value| value.as_str().to_owned()).collect())))
            }
        }
        impl ValidateProjection for crate::$ty {
            const NEEDS_RECURSIVE_VALIDATION: bool = false;
            fn validate_at(&self, _: &str) -> Result<(), ValidationError> { Ok(()) }
        }
    )* };
}
// Only owner selection lives here. Their declared ALL/as_str owns every member.
quantity_vocabulary!(
    ScaleKind,
    QuantityAdditionKind,
    QuantityKindCategory,
    BasisKind,
    CompositionBasis,
    RateBasis,
    ReductionKind,
    ReferenceStateKind,
    ConversionKind,
    Opcode,
    BasisRule,
    ReferenceRule,
    QuantityScaleRule,
    QuantityShapeRule,
    SubjectRule
);
