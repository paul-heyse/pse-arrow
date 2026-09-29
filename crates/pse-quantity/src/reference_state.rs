// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reference state declarations (blueprint §6.2, §8.4).
use crate::{QuantityTypeId, ReferenceStateId, ReferenceStateKind, UnitId};
use pse_ids::SemanticId;
/// A typed reference condition (ADR-0123 Outcome 6): a value in a declared unit of a
/// declared quantity type. Admission checks the type's dimension against the condition it
/// states and the unit against the type.
#[derive(Clone, Copy, Debug)]
pub struct ReferenceCondition {
    /// The value in `unit`.
    pub value: f64,
    /// The declared quantity type of the condition.
    pub quantity_type: QuantityTypeId,
    /// The unit `value` is written in.
    pub unit: UnitId,
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
/// The datum carried by origin-sensitive quantities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceState {
    /// Registry identity.
    pub id: ReferenceStateId,
    /// The name packages address it by, declared once in the physical document
    /// (ADR-0123 Outcome 6).
    pub name: String,
    /// The reference convention.
    pub kind: ReferenceStateKind,
    /// Typed reference temperature, when specified.
    pub temperature: Option<ReferenceCondition>,
    /// Typed reference pressure, when specified.
    pub pressure: Option<ReferenceCondition>,
    /// Whether formation enthalpy is included.
    pub include_enthalpy_of_formation: bool,
    /// Optional datum subject; physical source admission requires an authored entity.
    pub subject: Option<SemanticId>,
}
