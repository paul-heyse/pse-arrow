// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Registered complete-quantity operation declarations (blueprint §6.2, §8.3).
use crate::{
    BasisId, BasisRule, ConversionId, InvariantId, Opcode, OperationId, QuantityKindId,
    QuantityScaleRule, QuantityShapeRule, ReferenceRule, ReferenceStateId, SubjectKind,
    SubjectRule,
};
/// A conversion applied once to the named operand before rule matching.
#[derive(Clone, Debug)]
pub struct InputConversion {
    /// Zero-based operand position.
    pub operand: u16,
    /// Registered conversion identity.
    pub conversion: ConversionId,
}
/// Data declaring a physical composition; no dimension-only fallback is implied.
#[derive(Clone, Debug)]
pub struct QuantityOperation {
    /// Registry identity.
    pub id: OperationId,
    /// Mathematical operator.
    pub opcode: Opcode,
    /// Operand kinds in argument order.
    pub input_kinds: Vec<QuantityKindId>,
    /// Declared result kind.
    pub result_kind: QuantityKindId,
    /// Result basis policy.
    pub basis_rule: BasisRule,
    /// Result datum policy.
    pub reference_rule: ReferenceRule,
    /// Result point/difference policy.
    pub scale_rule: QuantityScaleRule,
    /// Result index-shape policy.
    pub shape_rule: QuantityShapeRule,
    /// Operand supplying a preserved basis.
    pub basis_source: Option<u16>,
    /// Operand supplying a preserved datum.
    pub reference_source: Option<u16>,
    /// Operand supplying a preserved scale kind.
    pub scale_source: Option<u16>,
    /// Operand supplying a preserved shape.
    pub shape_source: Option<u16>,
    /// Result subject policy.
    pub subject_rule: SubjectRule,
    /// Operand supplying a preserved subject.
    pub subject_source: Option<u16>,
    /// Explicit result subject for `declared_result`.
    pub result_subject_kind: Option<SubjectKind>,
    /// Explicit result basis for `declared_result`.
    pub result_basis: Option<BasisId>,
    /// Explicit result datum for `declared_result`.
    pub result_reference_state: Option<ReferenceStateId>,
    /// Conversions performed before matching.
    pub input_conversions: Vec<InputConversion>,
    /// Invariants that must be established at application time.
    pub precondition_invariants: Vec<InvariantId>,
}
