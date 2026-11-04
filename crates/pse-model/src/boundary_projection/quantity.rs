// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Trait-owner integration of actual quantity declarations; no copied physical field schema.

use super::*;
use pse_quantity::boundary::{BoundaryField, BoundaryRecord};

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

pse_quantity::quantity_type_key_boundary!(quantity_record);
pse_quantity::quantity_type_boundary!(quantity_record);
pse_quantity::basis_boundary!(quantity_record);
pse_quantity::entity_kind_boundary!(quantity_record);
pse_quantity::reference_condition_boundary!(quantity_record);
pse_quantity::reference_state_boundary!(quantity_record);
pse_quantity::unit_factor_boundary!(quantity_record);
pse_quantity::unit_boundary!(quantity_record);
pse_quantity::quantity_kind_boundary!(quantity_record);
pse_quantity::kind_factor_boundary!(quantity_record);
pse_quantity::kind_definition_boundary!(quantity_record);
pse_quantity::ratio_boundary!(quantity_record);
pse_quantity::dimension_vector_boundary!(quantity_record);

macro_rules! quantity_vocabulary {
    ($($ty:ident),* $(,)?) => { $(
        impl ProjectionType for pse_quantity::$ty {
            fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
                graph.declare::<Self>(stringify!($ty), "crates/pse-quantity/src/enums.rs", 1,
                    Direction::Both, &[], &[], |_| Ok(Shape::StringEnum(
                        Self::ALL.iter().map(|value| value.as_str().to_owned()).collect())))
            }
        }
        impl ValidateProjection for pse_quantity::$ty {
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
    ReferenceStateKind
);

impl<T: ProjectionType, const N: usize> ProjectionType for [T; N] {
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        let element = T::projection(graph)?;
        let element_name = match &element {
            TypeRef::Named(name) => name.clone(),
            TypeRef::Scalar(scalar) => format!("{scalar:?}"),
            _ => T::projection_argument_name(graph)?,
        };
        let name = format!("Array{N}{element_name}");
        let length = u64::try_from(N)
            .map_err(|_| ProjectionError::invalid(&name, "array extent exceeds uint64"))?;
        let root =
            graph.declare::<Self>(&name, file!(), 1, Direction::Both, &[], &[], |graph| {
                Ok(Shape::Record(vec![Field::of::<Vec<T>>(
                    graph,
                    "values",
                    1,
                    Direction::Both,
                    vec![LocalRule::Length {
                        min: length,
                        max: length,
                    }],
                )?]))
            })?;
        graph.array_container::<T, N>(&root, element)?;
        let element = T::projection_rust_path(graph)?;
        graph.public_path::<Self>(&root, &format!("[{element};{N}]"))?;
        Ok(root)
    }
    fn projection_rust_path(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(format!("[{};{N}]", T::projection_rust_path(graph)?))
    }
    fn projection_argument_name(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(format!("Array{N}{}", T::projection_argument_name(graph)?))
    }
}
impl<T, const N: usize> ProjectionRecord for [T; N] {
    const VERSION: u32 = 1;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "values")];
    type Fields = (Vec<T>,);
    fn into_projection_fields(self) -> Self::Fields {
        (self.into_iter().collect(),)
    }
    fn from_projection_fields((values,): Self::Fields) -> Result<Self, AdapterError> {
        values.try_into().map_err(|_: Vec<T>| {
            AdapterError::refused(
                "values",
                AdapterFailure::InvalidValue,
                format!("fixed array requires exactly {N} components"),
            )
        })
    }
}
impl<T: ValidateProjection, const N: usize> ValidateProjection for [T; N] {
    const SUPPORTS_FLOAT64_DOMAIN: bool = T::SUPPORTS_FLOAT64_DOMAIN;
    const NEEDS_RECURSIVE_VALIDATION: bool = T::NEEDS_RECURSIVE_VALIDATION;
    fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
        if !Self::NEEDS_RECURSIVE_VALIDATION {
            return Ok(());
        }
        for (index, value) in self.iter().enumerate() {
            value.validate_at(&format!("{path}[{index}]"))?;
        }
        Ok(())
    }
    fn projection_length(&self) -> Option<usize> {
        Some(N)
    }
    fn projection_float64_domain(&self, domain: Float64Domain) -> Option<bool> {
        Self::SUPPORTS_FLOAT64_DOMAIN.then(|| {
            self.iter()
                .all(|value| value.projection_float64_domain(domain) == Some(true))
        })
    }
}
