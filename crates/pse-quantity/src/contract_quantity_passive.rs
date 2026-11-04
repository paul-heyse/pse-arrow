// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Consume source-owned passive quantity metadata beside the admitted data records.
//! These implementations expose documents; native physical receipt restoration stays
//! exclusively with the original quantity owner.

use pse_contract::boundary_projection::*;
use crate::boundary::{
    BoundaryAlternative, BoundaryAlternativeField, BoundaryAlternatives, BoundaryDocument,
    BoundaryField, BoundaryFieldEncoding, BoundaryRecord, PassiveFieldEncoding,
};

macro_rules! passive_quantity_record {
    ($owner:path; $($field:ident => $tag:literal;)*) => {
        impl ProjectionType for $owner {
            fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
                graph.declare::<Self>(
                    <Self as BoundaryRecord>::NAME,
                    <Self as BoundaryRecord>::SOURCE,
                    <Self as BoundaryRecord>::VERSION,
                    <Self as BoundaryRecord>::DIRECTION,
                    <Self as BoundaryRecord>::RESERVED_TAGS,
                    <Self as BoundaryRecord>::RESERVED_NAMES,
                    |graph| Ok(Shape::Record(vec![$({
                        let field = Field::of::<<Self as BoundaryField<$tag>>::Type>(
                            graph, stringify!($field), $tag,
                            <Self as BoundaryRecord>::DIRECTION, vec![],
                        )?;
                        match <Self as BoundaryFieldEncoding<$tag>>::ENCODING {
                            PassiveFieldEncoding::Native => field,
                            PassiveFieldEncoding::Binary64Bits => field.representation(Scalar::F64Bits)?,
                        }
                    },)*])),
                )
            }
        }
        impl ProjectionRecord for $owner {
            const VERSION: u32 = <Self as BoundaryRecord>::VERSION;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[$(($tag, stringify!($field))),*];
            type Fields = <Self as BoundaryRecord>::Fields;
            fn into_projection_fields(self) -> Self::Fields { self.into_boundary_fields() }
            fn from_projection_fields(fields: Self::Fields) -> Result<Self, AdapterError> {
                Self::from_boundary_fields(fields).map_err(|error|
                    AdapterError::refused("", AdapterFailure::InvalidValue, error.to_string()))
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

macro_rules! passive_quantity_alternatives {
    ($owner:path; $($variant:ident => $alternative:literal {
        $($field:ident => $tag:literal;)+
    };)*) => {
        impl ProjectionType for $owner {
            fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
                graph.declare::<Self>(
                    <Self as BoundaryAlternatives>::NAME,
                    <Self as BoundaryAlternatives>::SOURCE,
                    <Self as BoundaryAlternatives>::VERSION,
                    <Self as BoundaryAlternatives>::DIRECTION,
                    &[], &[],
                    |graph| Ok(Shape::Alternatives(vec![$(
                        Alternative {
                            name: stringify!($variant).into(), tag: $alternative,
                            fields: vec![$(
                                Field::of::<<Self as BoundaryAlternativeField<$alternative, $tag>>::Type>(
                                    graph, stringify!($field), $tag,
                                    <Self as BoundaryAlternatives>::DIRECTION, vec![],
                                )?,
                            )+],
                        },
                    )*])),
                )
            }
        }
        $(impl ProjectionAlternative<$alternative> for $owner {
            const VERSION: u32 = <Self as BoundaryAlternatives>::VERSION;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[$(($tag, stringify!($field))),+];
            type Fields = <Self as BoundaryAlternative<$alternative>>::Fields;
            fn into_projection_alternative(self) -> Result<Self::Fields, Self> {
                <Self as BoundaryAlternative<$alternative>>::into_boundary_alternative(self)
            }
            fn from_projection_alternative(fields: Self::Fields) -> Self {
                <Self as BoundaryAlternative<$alternative>>::from_boundary_alternative(fields)
            }
        })*
        impl ValidateProjection for $owner {
            fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
                $(if let Some(($($field,)+)) =
                    <Self as BoundaryAlternative<$alternative>>::boundary_alternative_fields(self)
                {
                    let alternative = field_path(path, stringify!($variant));
                    $($field.validate_at(&field_path(&alternative, stringify!($field)))?;)+
                })*
                Ok(())
            }
        }
    };
}

// Selection only: every original field, tag, type and exact-bit interpretation
// comes from its lower source callback, including fields private to that owner.
physical_contract_record_boundary!(passive_quantity_record);
physical_scheme_record_boundary!(passive_quantity_record);
physical_contract_facts_boundary!(passive_quantity_record);
physical_qualified_factor_boundary!(passive_quantity_record);
physical_formula_facts_boundary!(passive_quantity_record);
bound_index_reference_boundary!(passive_quantity_record);
physical_scheme_shape_boundary!(passive_quantity_alternatives);
physical_inference_record_boundary!(passive_quantity_record);

// The sole intrinsic document-view field is a typed link to the original record,
// rather than a second declaration of Scheme alternatives or physical facts.
const SCHEME_DOCUMENT: (u32, &str) = (1, "document");
impl ProjectionType for crate::scheme::Scheme {
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        graph.declare::<Self>(
            <Self as BoundaryDocument>::NAME,
            <Self as BoundaryDocument>::SOURCE,
            <Self as BoundaryDocument>::VERSION,
            <Self as BoundaryDocument>::DIRECTION,
            &[],
            &[],
            |graph| {
                Ok(Shape::Record(vec![Field::of::<
                    <Self as BoundaryDocument>::Document,
                >(
                    graph,
                    SCHEME_DOCUMENT.1,
                    SCHEME_DOCUMENT.0,
                    <Self as BoundaryDocument>::DIRECTION,
                    vec![],
                )?]))
            },
        )
    }
}
impl ProjectionRecord for crate::scheme::Scheme {
    const VERSION: u32 = <Self as BoundaryDocument>::VERSION;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[SCHEME_DOCUMENT];
    type Fields = (<Self as BoundaryDocument>::Document,);
    fn into_projection_fields(self) -> Self::Fields {
        (self.boundary_document(),)
    }
    fn from_projection_fields((document,): Self::Fields) -> Result<Self, AdapterError> {
        Self::from_boundary_document(document).map_err(|error| {
            AdapterError::refused(
                SCHEME_DOCUMENT.1,
                AdapterFailure::InvalidValue,
                error.to_string(),
            )
        })
    }
}
impl ValidateProjection for crate::scheme::Scheme {
    fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
        // Capture is passive. In particular, outgoing Resolved contracts need not
        // satisfy a new recipe-membership check merely to publish their facts.
        self.boundary_document()
            .validate_at(&field_path(path, SCHEME_DOCUMENT.1))
    }
}

#[cfg(test)]
#[path = "contract_quantity_passive_tests.rs"]
mod tests;
