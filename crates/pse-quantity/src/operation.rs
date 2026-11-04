// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registered complete-quantity operation declarations (blueprint §6.2, §8.3).
use crate::{
    BasisId, BasisRule, ConversionId, EntityKindId, InvariantId, Opcode, OperationId,
    QuantityKindId, QuantityScaleRule, QuantityShapeRule, ReferenceRule, ReferenceStateId,
    SubjectRule,
};
pse_contract::projection_struct! {
/// A conversion applied once to the named operand before rule matching.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InputConversion {
    projection: { name: "PhysicalInputConversionDeclaration", rust_path: crate::InputConversion, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Zero-based operand position.
    pub operand: u16 => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Registered conversion identity.
    pub conversion: ConversionId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Data declaring a physical composition; no dimension-only fallback is implied.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct QuantityOperation {
    projection: { name: "PhysicalOperationDeclaration", rust_path: crate::QuantityOperation, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Registry identity.
    pub id: OperationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Mathematical operator.
    pub opcode: Opcode => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Operand kinds in argument order.
    pub input_kinds: Vec<QuantityKindId> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Declared result kind.
    pub result_kind: QuantityKindId => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Result basis policy.
    pub basis_rule: BasisRule => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Result datum policy.
    pub reference_rule: ReferenceRule => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Result point/difference policy.
    pub scale_rule: QuantityScaleRule => { tag: 7, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Result index-shape policy.
    pub shape_rule: QuantityShapeRule => { tag: 8, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Operand supplying a preserved basis.
    pub basis_source: Option<u16> => { tag: 9, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Operand supplying a preserved datum.
    pub reference_source: Option<u16> => { tag: 10, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Operand supplying a preserved scale kind.
    pub scale_source: Option<u16> => { tag: 11, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Operand supplying a preserved shape.
    pub shape_source: Option<u16> => { tag: 12, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Result subject policy.
    pub subject_rule: SubjectRule => { tag: 13, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Operand supplying a preserved subject.
    pub subject_source: Option<u16> => { tag: 14, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Explicit result subject for `declared_result`.
    pub result_subject_kind: Option<EntityKindId> => { tag: 15, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Explicit result basis for `declared_result`.
    pub result_basis: Option<BasisId> => { tag: 16, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Explicit result datum for `declared_result`.
    pub result_reference_state: Option<ReferenceStateId> => { tag: 17, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Conversions performed before matching.
    pub input_conversions: Vec<InputConversion> => { tag: 18, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Invariants that must be established at application time.
    pub precondition_invariants: Vec<InvariantId> => { tag: 19, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
