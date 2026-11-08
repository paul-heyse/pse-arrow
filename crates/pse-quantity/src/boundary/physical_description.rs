// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive original quantity, datum, unit and representation correspondence.

use pse_contract::boundary_projection::{Direction, LocalRule};
use serde::{Deserialize, Serialize};

pse_contract::projection_struct! {
/// Original selected quantity and its complete physical interpretation.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PhysicalQuantityDescription {
    projection: { name: "PhysicalQuantityDescription", rust_path: crate::boundary::PhysicalQuantityDescription,
        version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Exact original type including basis, datum, scale, shape and subject.
    pub quantity: crate::QuantityType => { tag: 1, direction: Direction::Output, rules: [LocalRule::Required] };
    /// What is measured, including exact rational dimension and arithmetic convention.
    pub quantity_kind: crate::QuantityKind => { tag: 2, direction: Direction::Output, rules: [LocalRule::Required] };
    /// Original canonical representation.
    pub canonical_unit: crate::Unit => { tag: 3, direction: Direction::Output, rules: [LocalRule::Required] };
    /// Selected representation; raw result values are never changed by inspection.
    pub representation_unit: crate::Unit => { tag: 4, direction: Direction::Output, rules: [LocalRule::Required] };
    /// Admitted conversion multiplier to canonical magnitude.
    pub scale_to_canonical: f64 => { tag: 5, direction: Direction::Output, rules: [] };
    /// Admitted conversion offset; difference quantities retain zero here.
    pub offset_to_canonical: f64 => { tag: 6, direction: Direction::Output, rules: [] };
    /// Exact original basis and its composition/rate/reference conventions.
    pub basis: Option<crate::Basis> => { tag: 7, direction: Direction::Output, rules: [] };
    /// Actual reference conditions required by a standard-volume or similar basis.
    pub basis_reference_state: Option<crate::ReferenceState> => { tag: 8, direction: Direction::Output, rules: [] };
    /// Exact original datum, including typed reference conditions.
    pub reference_state: Option<crate::ReferenceState> => { tag: 9, direction: Direction::Output, rules: [] };
    /// Declared ordered axis kinds.
    pub shape: Vec<crate::EntityKind> => { tag: 10, direction: Direction::Output, rules: [] };
    /// Declared subject kind.
    pub subject: Option<crate::EntityKind> => { tag: 11, direction: Direction::Output, rules: [] };
}
}
