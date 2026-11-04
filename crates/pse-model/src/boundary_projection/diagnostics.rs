// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Local trait integration of the original lower-level diagnostic declarations.

use super::*;
use pse_diagnostics::boundary::{BoundaryDeclaration, BoundaryField};

macro_rules! diagnostic_record {
    ($owner:path; $public:expr; $name:literal; $version:literal; $direction:ident;
     [$($reserved:literal),*]; [$($reserved_name:literal),*]; $($field:ident => $tag:literal;)*) => {
        impl ProjectionType for $owner {
            fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
                let root = graph.declare::<Self>($name, <Self as BoundaryDeclaration>::SOURCE, $version, Direction::$direction,
                    &[$($reserved),*], &[$($reserved_name),*], |graph| Ok(Shape::Record(vec![$(
                        Field::of::<<Self as BoundaryField<$tag>>::Type>(graph, stringify!($field), $tag, Direction::$direction, vec![])?,
                    )*])))?;
                graph.public_path::<Self>(&root, $public)?;
                Ok(root)
            }
        }
        impl ValidateProjection for $owner {
            fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
                $(<Self as BoundaryField<$tag>>::boundary_field(self).validate_at(&field_path(path, stringify!($field)))?;)*
                Ok(())
            }
        }
        impl ProjectionRecord for $owner {
            const VERSION: u32 = $version;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[$(($tag, stringify!($field))),*];
            type Fields = ($(<Self as BoundaryField<$tag>>::Type,)*);
            fn into_projection_fields(self) -> Self::Fields { ($(self.$field,)*) }
            fn from_projection_fields(($($field,)*): Self::Fields) -> Result<Self, AdapterError> { Ok(Self { $($field,)* }) }
        }
    };
}
macro_rules! diagnostic_enum {
    ($owner:path; $public:expr; $name:literal; $version:literal; $direction:ident;
     [$($reserved:literal),*]; [$($reserved_name:literal),*]; $($variant:ident => $tag:literal;)*) => {
        impl ProjectionType for $owner {
            fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
                let root = graph.declare::<Self>($name, <Self as BoundaryDeclaration>::SOURCE, $version, Direction::$direction,
                    &[$($reserved),*], &[$($reserved_name),*], |_| Ok(Shape::Alternatives(vec![$(
                        Alternative { name: stringify!($variant).into(), tag: $tag, fields: vec![] },
                    )*])))?;
                graph.public_path::<Self>(&root, $public)?;
                Ok(root)
            }
        }
        impl ValidateProjection for $owner {
            const NEEDS_RECURSIVE_VALIDATION: bool = false;
            fn validate_at(&self, _: &str) -> Result<(), ValidationError> { Ok(()) }
        }
        $(impl ProjectionAlternative<$tag> for $owner {
            const VERSION: u32 = $version;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[];
            type Fields = ();
            fn into_projection_alternative(self) -> Result<Self::Fields, Self> {
                match self { Self::$variant => Ok(()), other => Err(other) }
            }
            fn from_projection_alternative((): Self::Fields) -> Self { Self::$variant }
        })*
    };
}

pse_diagnostics::diagnostic_operand_contract_boundary!(diagnostic_record);
pse_diagnostics::diagnostic_operation_id_boundary!(diagnostic_record);
pse_diagnostics::diagnostic_producer_id_boundary!(diagnostic_record);
pse_diagnostics::diagnostic_producer_role_boundary!(diagnostic_enum);
