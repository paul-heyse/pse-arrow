// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete physical type keys (blueprint §6.2, §8.1).
use crate::registry::QuantityRegistry;
use crate::{
    BasisId, EntityKindId, QuantityKindId, QuantityTypeId, ReferenceStateId, ScaleKind, UnitId,
};
crate::quantity_boundary_record! {
callback: quantity_type_key_boundary; name: "QuantityTypeKey"; version: 1;
direction: Both; reserved_tags: []; reserved_names: [];
construct: [];
/// Components that determine one registered physical type.
#[derive(
    schemars::JsonSchema,
    serde::Serialize,
    serde::Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[schemars(deny_unknown_fields)]
pub struct QuantityTypeKey {
    /// The measured quantity kind.
    pub kind: QuantityKindId => 1;
    /// Optional material basis.
    pub basis: Option<BasisId> => 2;
    /// Optional datum.
    pub reference_state: Option<ReferenceStateId> => 3;
    /// Point or difference semantics.
    pub scale_kind: ScaleKind => 4;
    /// Ordered domain kinds.
    pub shape: Vec<EntityKindId> => 5;
    /// Optional subject obligation; absent and explicit `none` stay distinct.
    pub subject_kind: Option<EntityKindId> => 6;
}
}
crate::quantity_boundary_record! {
callback: quantity_type_boundary; name: "QuantityType"; version: 1;
direction: Both; reserved_tags: []; reserved_names: [];
construct: [];
/// A registered physical type and its canonical storage unit.
#[derive(schemars::JsonSchema, serde::Serialize, serde::Deserialize, Clone, Debug)]
#[schemars(deny_unknown_fields)]
pub struct QuantityType {
    /// Registry identity.
    pub id: QuantityTypeId => 1;
    /// The name packages address it by, declared once in the physical document and unique
    /// within it (ADR-0123 Outcome 6).
    pub name: Option<String> => 2;
    /// The complete semantic key.
    pub key: QuantityTypeKey => 3;
    /// Canonical unit of the package.
    pub canonical_unit: UnitId => 4;
    /// Positive finite default scaling magnitude, if declared.
    pub nominal_magnitude: Option<f64> => 5;
}
}
impl QuantityType {
    /// Whether the package explicitly designated this complete type as neutral.
    pub fn is_neutral_scalar(&self, registry: &QuantityRegistry) -> bool {
        registry.neutral_dimensionless() == Some(self.id)
    }
}

// Semantic equality preserves every declared IEEE bit, including signed zero.
impl PartialEq for QuantityType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.key == other.key
            && self.canonical_unit == other.canonical_unit
            && self.nominal_magnitude.map(f64::to_bits) == other.nominal_magnitude.map(f64::to_bits)
    }
}
impl Eq for QuantityType {}
