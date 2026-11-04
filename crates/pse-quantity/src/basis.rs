// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Basis declarations (blueprint §6.2, §8.4).
use crate::{BasisId, BasisKind, CompositionBasis, RateBasis, ReferenceStateId};
crate::quantity_boundary_record! {
callback: basis_boundary; name: "PhysicalBasis"; version: 1;
direction: Both; reserved_tags: []; reserved_names: [];
construct: [];
/// A declared material or specific-quantity basis.
#[derive(
    schemars::JsonSchema, serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq,
)]
#[expect(
    clippy::struct_field_names,
    reason = "fields retain the reference.bases contract names"
)]
#[schemars(rename = "PhysicalBasis")]
#[schemars(deny_unknown_fields)]
pub struct Basis {
    /// Registry identity.
    pub id: BasisId => 1;
    /// What this quantity is per.
    pub kind: BasisKind => 2;
    /// Optional composition convention.
    pub composition_basis: Option<CompositionBasis> => 3;
    /// Optional rate convention.
    pub rate_basis: Option<RateBasis> => 4;
    /// Conditions required for a standard-volume basis.
    pub reference_conditions: Option<ReferenceStateId> => 5;
}
}
