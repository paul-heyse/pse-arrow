// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reference state declarations (blueprint §6.2, §8.4).
use crate::{QuantityTypeId, ReferenceStateId, ReferenceStateKind, UnitId};
use pse_ids::SemanticId;
crate::quantity_boundary_record! {
callback: reference_condition_boundary; name: "ReferenceCondition"; version: 1;
direction: Both; reserved_tags: []; reserved_names: [];
construct: [];
/// A typed reference condition (ADR-0123 Outcome 6): a value in a declared unit of a
/// declared quantity type. Admission checks the type's dimension against the condition it
/// states and the unit against the type.
#[derive(schemars::JsonSchema, serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
#[schemars(deny_unknown_fields)]
pub struct ReferenceCondition {
    /// The value in `unit`.
    pub value: f64 => 1;
    /// The declared quantity type of the condition.
    pub quantity_type: QuantityTypeId => 2;
    /// The unit `value` is written in.
    pub unit: UnitId => 3;
}
}
// Semantic equality preserves every declared IEEE bit, including signed zero.
impl PartialEq for ReferenceCondition {
    fn eq(&self, other: &Self) -> bool {
        self.value.to_bits() == other.value.to_bits()
            && self.quantity_type == other.quantity_type
            && self.unit == other.unit
    }
}
impl Eq for ReferenceCondition {}
crate::quantity_boundary_record! {
callback: reference_state_boundary; name: "ReferenceState"; version: 1;
direction: Both; reserved_tags: []; reserved_names: [];
construct: [];
/// The datum carried by origin-sensitive quantities.
#[derive(
    schemars::JsonSchema, serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq,
)]
#[schemars(deny_unknown_fields)]
pub struct ReferenceState {
    /// Registry identity.
    pub id: ReferenceStateId => 1;
    /// The name packages address it by, declared once in the physical document
    /// (ADR-0123 Outcome 6).
    pub name: String => 2;
    /// The reference convention.
    pub kind: ReferenceStateKind => 3;
    /// Typed reference temperature, when specified.
    pub temperature: Option<ReferenceCondition> => 4;
    /// Typed reference pressure, when specified.
    pub pressure: Option<ReferenceCondition> => 5;
    /// Whether formation enthalpy is included.
    pub include_enthalpy_of_formation: bool => 6;
    /// Optional datum subject; physical source admission requires an authored entity.
    pub subject: Option<SemanticId> => 7;
}
}
