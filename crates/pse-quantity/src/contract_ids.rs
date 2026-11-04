// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use pse_contract::boundary_projection::*;
macro_rules! physical_identity_projection { ($($ty:ty => $name:literal),* $(,)?) => { $(
    impl ProjectionType for $ty {
        fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
            graph.declare::<Self>($name, "crates/pse-quantity/src/ids.rs", 1, Direction::Both, &[], &[], |graph| {
                Ok(identity_shape(pse_ids::SemanticId::projection(graph)?))
            })
        }
    }
    impl ValidateProjection for $ty {
        const NEEDS_RECURSIVE_VALIDATION: bool = false;
        fn validate_at(&self, _: &str) -> Result<(), ValidationError> { Ok(()) }
        fn projection_length(&self) -> Option<usize> { Some(16) }
    }
    impl ProjectionRecord for $ty {
        const VERSION: u32 = 1;
        const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "id")];
        type Fields = (pse_ids::SemanticId,);
        fn into_projection_fields(self) -> Self::Fields { (self.as_id(),) }
        fn from_projection_fields((id,): Self::Fields) -> Result<Self, AdapterError> { Ok(Self::from_id(id)) }
    }
)* }; }
// The trait owner supplies legal implementations across the quantity -> model dependency
// boundary. These actual nominal types remain distinct named identity projections.
physical_identity_projection!(
    crate::EntityKindId => "EntityKindId",
    crate::UnitId => "UnitId",
    crate::UnitSetId => "UnitSetId",
    crate::ConversionId => "ConversionId",
    crate::OperationId => "OperationId",
    crate::InvariantId => "InvariantId",
    crate::QuantityKindId => "QuantityKindId",
    crate::BasisId => "BasisId",
    crate::QuantityTypeId => "QuantityTypeId",
    crate::ReferenceStateId => "ReferenceStateId",
    crate::DomainId => "DomainId",
    crate::BoundIndexId => "BoundIndexId",
);
