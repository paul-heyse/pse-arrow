// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Every error this crate returns, with its blueprint §23.2 class.
//!
//! The classes are spelled as `#[diagnostic(code(...))]` in Rust path form, which is what
//! `tests/governance/tests/error_taxonomy.rs` and the §23.2 table agree on. The variants
//! split along those classes on purpose:
//!
//! - a registry that does not load, an unknown identity or a broken rational exponent is
//!   `validation::invariant` — the data is wrong before any expression is typed;
//! - no rule, or more than one rule, for a composition is
//!   `compile::math::quantity_operation_unsupported`, never a dimension-only fallback
//!   (§8.3);
//! - a mismatch between two complete quantity types is `compile::math::unit_inconsistent`;
//! - a literal that violates an operator's domain restriction before any solver runs is
//!   `compile::math::domain_violation_static`.
//!
//! Messages name identities through `Display` (lowercase hexadecimal) and never through a
//! `Debug` rendering, so nothing here can grow a hash-container iteration order (§5.3).

use pse_ids::SemanticId;

use crate::enums::Opcode;
use crate::ids::{ConversionId, OperationId, QuantityKindId, QuantityTypeId, UnitId};

/// A rational exponent could not be formed or read (blueprint §4.4, §6.2).
///
/// Every variant is `validation::invariant`: the exponent pair is data, and a pair that
/// cannot be reduced into the `pse.dimension_vector` storage is invalid data rather than a
/// failure of the expression being compiled.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, miette::Diagnostic)]
#[non_exhaustive]
pub enum DimensionError {
    /// A rational exponent was offered with a zero denominator.
    #[error("a rational exponent cannot have a zero denominator")]
    #[diagnostic(code(validation::invariant))]
    ZeroDenominator,

    /// A reduced exponent does not fit the `i16` numerator/denominator pair.
    #[error("the rational exponent {num}/{den} does not fit the canonical `i16` pair")]
    #[diagnostic(code(validation::invariant))]
    Overflow {
        /// The numerator before reduction.
        num: i64,
        /// The denominator before reduction.
        den: i64,
    },

    /// A stored exponent pair is not the canonical form of its value.
    #[error("the rational exponent {num}/{den} is not in canonical reduced form")]
    #[diagnostic(code(validation::invariant))]
    NotReduced {
        /// The stored numerator.
        num: i16,
        /// The stored denominator.
        den: i16,
    },
}

crate::closed_enum! {
    /// Why two complete quantity types could not be combined (blueprint §8.1, §8.3).
    ///
    /// The reason is the diagnostic: "unit mismatch" is what a dimension-only checker can
    /// say, and it is exactly the answer §8.1 rejects. Each member names the component of
    /// the §8.1 tuple that disagreed, or the origin-sensitive rule that refused.
    pub enum IncompatibilityReason {
        /// The quantity kinds differ.
        KindMismatch => "kind_mismatch",
        /// The bases differ, and no registered conversion applies.
        BasisMismatch => "basis_mismatch",
        /// The reference states differ, and no registered conversion applies.
        ReferenceMismatch => "reference_mismatch",
        /// The subjects differ.
        SubjectMismatch => "subject_mismatch",
        /// The free index identities differ.
        IndexMismatch => "index_mismatch",
        /// Two points on an origin-sensitive scale were added.
        PointPlusPoint => "point_plus_point",
        /// A point was subtracted from a difference.
        DifferenceMinusPoint => "difference_minus_point",
        /// Two points were subtracted, but they are measured from different datums.
        DatumMismatch => "datum_mismatch",
        /// An affine unit offset was applied to a difference.
        AffineOffsetOnDifference => "affine_offset_on_difference",
        /// A reduction summed points; averaging points needs `WeightedMean`.
        SumOfPoints => "sum_of_points",
    }
}

crate::closed_enum! {
    /// Which component of the §8.1 quantity-type tuple a contract check compared.
    pub enum ContractComponent {
        /// The quantity kind.
        Kind => "kind",
        /// The basis.
        Basis => "basis",
        /// The reference state.
        ReferenceState => "reference_state",
        /// The point/difference scale.
        ScaleKind => "scale_kind",
        /// The index shape.
        Shape => "shape",
        /// The subject.
        SubjectKind => "subject_kind",
        /// The canonical unit.
        Unit => "unit",
    }
}

/// Every failure of physical typing (blueprint §8.1, §8.3, §23.2).
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[non_exhaustive]
pub enum QuantityError {
    /// A load-time invariant of the quantity registry was violated.
    #[error("quantity registry rule `{rule}` rejected `{subject}`: {detail}")]
    #[diagnostic(code(validation::invariant))]
    Registry {
        /// The load-time rule that refused, named as it is in §8.2.
        rule: &'static str,
        /// The identity the rule was applied to.
        subject: SemanticId,
        /// What specifically was wrong.
        detail: String,
    },

    /// An operation lacks an established semantic prerequisite.
    #[error("quantity inference rule `{rule}` rejected the operation: {detail}")]
    #[diagnostic(code(compile::math::quantity_operation_unsupported))]
    InferencePrecondition {
        /// Named prerequisite that was not established.
        rule: &'static str,
        /// Specific absent or conflicting facts.
        detail: String,
    },

    /// Dimension arithmetic failed.
    #[error(transparent)]
    #[diagnostic(code(validation::invariant))]
    Dimension(#[from] DimensionError),

    /// No registered `quantity_operations` rule, or more than one, matched the operands.
    ///
    /// §8.3 refuses to fall back on dimension equality here: two operands whose dimensions
    /// combine arithmetically may still be torque and energy.
    #[error(
        "`{opcode}` has no unique registered quantity operation over {} operand kinds \
         ({ordered_matches} ordered and {swapped_matches} swapped matches)",
        .input_kinds.len()
    )]
    #[diagnostic(code(compile::math::quantity_operation_unsupported))]
    OperationUnsupported {
        /// The operator whose composition failed.
        opcode: Opcode,
        /// The operand kinds, in argument order.
        input_kinds: Vec<QuantityKindId>,
        /// How many registered rules matched the operands in argument order.
        ordered_matches: usize,
        /// How many matched only after swapping the operands.
        swapped_matches: usize,
    },

    /// A rule resolved, but the quantity type it composes is not registered.
    #[error(
        "`{opcode}` resolved operation `{operation}`, whose result type `{requested}` is not registered"
    )]
    #[diagnostic(code(compile::math::quantity_operation_unsupported))]
    UnregisteredResultType {
        /// The operator whose result could not be resolved.
        opcode: Opcode,
        /// The registered operation that was selected.
        operation: OperationId,
        /// The composed quantity-type key, rendered for the diagnostic.
        requested: String,
    },

    /// Two complete quantity types could not be combined.
    #[error("incompatible quantity types ({reason}) across {} operands", .operands.len())]
    #[diagnostic(code(compile::math::unit_inconsistent))]
    Incompatible {
        /// Which component, or which origin-sensitive rule, refused.
        reason: IncompatibilityReason,
        /// The operand types, in argument order.
        operands: Vec<QuantityTypeId>,
        /// A registered conversion that would make the operands compatible, when one
        /// exists; the author has to ask for it explicitly (§8.4).
        hint: Option<ConversionId>,
    },

    /// A literal's unit does not resolve to exactly one quantity type.
    #[error("the literal unit `{unit}` matches {} registered quantity types", .candidates.len())]
    #[diagnostic(code(compile::math::unit_inconsistent))]
    AmbiguousLiteral {
        /// The unit written on the literal.
        unit: UnitId,
        /// The quantity types the unit could belong to.
        candidates: Vec<QuantityTypeId>,
    },

    /// A declared `UnitConvert` edge disagrees with the operand it converts.
    #[error(
        "`UnitConvert` declares `{declared_from}` → `{to}`, but the operand carries `{operand_unit}`"
    )]
    #[diagnostic(code(compile::math::unit_inconsistent))]
    UnitConvertMismatch {
        /// The source unit the conversion edge declares.
        declared_from: UnitId,
        /// The unit the operand actually carries.
        operand_unit: UnitId,
        /// The target unit the conversion edge declares.
        to: UnitId,
    },

    /// A literal operand violates an operator's domain restriction before any solve.
    #[error("`{opcode}` requires {restriction}, but the literal operand is {value}")]
    #[diagnostic(code(compile::math::domain_violation_static))]
    StaticDomain {
        /// The operator whose restriction was violated.
        opcode: Opcode,
        /// The restriction, spelled as §7.2 spells it (`a positive argument`, `eps > 0`).
        restriction: &'static str,
        /// The literal value that violates it.
        value: f64,
    },

    /// A declared contract and the inferred type disagree in one component.
    #[error("{component} mismatch: expected `{expected}`, inferred `{actual}`")]
    #[diagnostic(code(compile::math::unit_inconsistent))]
    ContractMismatch {
        /// Which component of the §8.1 tuple disagreed.
        component: ContractComponent,
        /// The declared quantity type.
        expected: QuantityTypeId,
        /// The inferred quantity type.
        actual: QuantityTypeId,
    },

    /// An identity does not resolve in the quantity registry.
    #[error("unknown {kind} `{id}`")]
    #[diagnostic(code(validation::invariant))]
    UnknownId {
        /// What kind of identity it was meant to be (`unit`, `quantity_type`, …).
        kind: &'static str,
        /// The identity that does not resolve.
        id: SemanticId,
    },
}

#[cfg(test)]
mod tests {
    use miette::Diagnostic as _;
    use pse_ids::SemanticId;

    use super::{ContractComponent, DimensionError, IncompatibilityReason, QuantityError};
    use crate::enums::Opcode;
    use crate::ids::{QuantityTypeId, UnitId};

    /// The §23.2 class of a variant, as a string, so the mapping is asserted and not
    /// merely written down.
    fn code_of(error: &QuantityError) -> String {
        error
            .code()
            .map(|code| code.to_string())
            .unwrap_or_default()
    }

    #[test]
    fn each_variant_carries_its_section_23_2_class() {
        let id = SemanticId::NIL;
        assert_eq!(
            code_of(&QuantityError::UnknownId { kind: "unit", id }),
            "validation::invariant"
        );
        assert_eq!(
            code_of(&QuantityError::Dimension(DimensionError::ZeroDenominator)),
            "validation::invariant"
        );
        assert_eq!(
            code_of(&QuantityError::OperationUnsupported {
                opcode: Opcode::Mul,
                input_kinds: Vec::new(),
                ordered_matches: 0,
                swapped_matches: 0,
            }),
            "compile::math::quantity_operation_unsupported"
        );
        assert_eq!(
            code_of(&QuantityError::Incompatible {
                reason: IncompatibilityReason::PointPlusPoint,
                operands: Vec::new(),
                hint: None,
            }),
            "compile::math::unit_inconsistent"
        );
        assert_eq!(
            code_of(&QuantityError::StaticDomain {
                opcode: Opcode::Log,
                restriction: "a positive argument",
                value: 0.0,
            }),
            "compile::math::domain_violation_static"
        );
    }

    /// A diagnostic names identities in hexadecimal, never through `Debug`.
    #[test]
    fn messages_render_identities_not_debug() {
        let unit = UnitId::from_id(SemanticId::from_bytes([0x0c; 16]));
        let error = QuantityError::AmbiguousLiteral {
            unit,
            candidates: vec![QuantityTypeId::from_id(SemanticId::NIL)],
        };
        let rendered = error.to_string();
        assert!(
            rendered.contains("0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c"),
            "{rendered}"
        );
        assert!(!rendered.contains("UnitId("), "{rendered}");
    }

    /// The reason and the component are part of the message, so a mismatch says which
    /// component of the §8.1 tuple disagreed.
    #[test]
    fn a_contract_mismatch_names_its_component() {
        let error = QuantityError::ContractMismatch {
            component: ContractComponent::ReferenceState,
            expected: QuantityTypeId::from_id(SemanticId::NIL),
            actual: QuantityTypeId::from_id(SemanticId::from_bytes([1; 16])),
        };
        assert!(error.to_string().starts_with("reference_state mismatch"));
    }

    #[test]
    fn a_dimension_error_converts_into_a_quantity_error() {
        let error: QuantityError = DimensionError::Overflow { num: 1, den: 2 }.into();
        assert!(matches!(error, QuantityError::Dimension(_)));
        assert_eq!(code_of(&error), "validation::invariant");
    }
}
