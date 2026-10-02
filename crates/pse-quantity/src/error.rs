// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Every error this crate returns, with its blueprint §23.2 class.
//!
//! Source variants declare detailed identities; each code derives its coarse failure class.
//! Quantity refusals retain complete operand and free-index contracts as structured evidence.
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
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum DimensionError {
    /// A rational exponent was offered with a zero denominator.
    #[error("a rational exponent cannot have a zero denominator")]
    ZeroDenominator,

    /// A reduced exponent does not fit the `i16` numerator/denominator pair.
    #[error("the rational exponent {num}/{den} does not fit the canonical `i16` pair")]
    Overflow {
        /// The numerator before reduction.
        num: i64,
        /// The denominator before reduction.
        den: i64,
    },

    /// A stored exponent pair is not the canonical form of its value.
    #[error("the rational exponent {num}/{den} is not in canonical reduced form")]
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
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum QuantityError {
    /// A physical numeric boundary received a nonfinite source magnitude.
    #[error("nonfinite magnitude {value} for quantity `{quantity}` in unit `{unit}`")]
    NonfiniteMagnitude {
        /// Admitted target quantity.
        quantity: QuantityTypeId,
        /// Declared source representation.
        unit: UnitId,
        /// Rejected source magnitude.
        value: f64,
    },

    /// A finite source overflowed its admitted multiply-then-add operation.
    #[error(
        "nonfinite converted magnitude for quantity `{quantity}` from `{from}` to `{to}`: ({value} * {scale}) + {offset}"
    )]
    NonfiniteConversion {
        /// Admitted target quantity.
        quantity: QuantityTypeId,
        /// Declared source representation.
        from: UnitId,
        /// Canonical target representation.
        to: UnitId,
        /// Finite source magnitude.
        value: f64,
        /// Admitted scale applied first.
        scale: f64,
        /// Admitted offset applied second.
        offset: f64,
    },

    /// A load-time invariant of the quantity registry was violated.
    #[error("quantity registry rule `{rule}` rejected `{subject}`: {detail}")]
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
    InferencePrecondition {
        /// Named prerequisite that was not established.
        rule: &'static str,
        /// Specific absent or conflicting facts.
        detail: String,
    },

    /// Dimension arithmetic failed.
    #[error(transparent)]
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
    UnregisteredResultType {
        /// The operator whose result could not be resolved.
        opcode: Opcode,
        /// The registered operation that was selected.
        operation: OperationId,
        /// The composed quantity-type key, rendered for the diagnostic.
        requested: String,
    },

    /// Two complete quantity types could not be combined.
    #[error("incompatible quantity types ({reason}): {}", operand_contracts(.operands))]
    Incompatible {
        /// Which component, or which origin-sensitive rule, refused.
        reason: IncompatibilityReason,
        /// Actual operand quantity and free-index contracts, in argument order.
        operands: Vec<(QuantityTypeId, crate::IndexSet)>,
        /// A registered conversion that would make the operands compatible, when one
        /// exists; the author has to ask for it explicitly (§8.4).
        hint: Option<ConversionId>,
    },

    /// A literal's unit does not resolve to exactly one quantity type.
    #[error("the literal unit `{unit}` matches {} registered quantity types", .candidates.len())]
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
    ContractMismatch {
        /// Which component of the §8.1 tuple disagreed.
        component: ContractComponent,
        /// The declared quantity type.
        expected: QuantityTypeId,
        /// The inferred quantity type.
        actual: QuantityTypeId,
    },

    /// A unit literal factor names no atomic unit (ADR-0124). A defined unit is spelled by
    /// its composition, never by its row name.
    #[error("`{symbol}` is not an atomic unit symbol")]
    UnknownUnitSymbol {
        /// The factor symbol as written.
        symbol: String,
    },

    /// An affine or datum-restricted unit appears other than as the sole factor with
    /// exponent one (ADR-0124).
    #[error(
        "the affine or datum-restricted unit `{symbol}` may appear only as the sole factor with exponent one"
    )]
    AffineUnitFactor {
        /// The offending unit.
        unit: UnitId,
        /// Its symbol.
        symbol: String,
    },

    /// An identity does not resolve in the quantity registry.
    #[error("unknown {kind} `{id}`")]
    UnknownId {
        /// What kind of identity it was meant to be (`unit`, `quantity_type`, …).
        kind: &'static str,
        /// The identity that does not resolve.
        id: SemanticId,
    },
}

impl QuantityError {
    /// Conservative owned diagnostic extent, including variable operand witnesses.
    pub fn retained_bytes(&self) -> usize {
        let heap = match self {
            Self::Registry { detail, .. } | Self::InferencePrecondition { detail, .. } => {
                detail.capacity()
            }
            Self::UnregisteredResultType { requested, .. } => requested.capacity(),
            Self::UnknownUnitSymbol { symbol } | Self::AffineUnitFactor { symbol, .. } => {
                symbol.capacity()
            }
            Self::OperationUnsupported { input_kinds, .. } => input_kinds
                .capacity()
                .saturating_mul(size_of::<QuantityKindId>()),
            Self::AmbiguousLiteral { candidates, .. } => candidates
                .capacity()
                .saturating_mul(size_of::<QuantityTypeId>()),
            Self::Incompatible { operands, .. } => operands.iter().fold(
                operands
                    .capacity()
                    .saturating_mul(size_of::<(QuantityTypeId, crate::IndexSet)>()),
                |bytes, (_, axes)| bytes.saturating_add(axes.len().saturating_mul(2048)),
            ),
            Self::Dimension(_)
            | Self::NonfiniteMagnitude { .. }
            | Self::NonfiniteConversion { .. }
            | Self::UnitConvertMismatch { .. }
            | Self::StaticDomain { .. }
            | Self::ContractMismatch { .. }
            | Self::UnknownId { .. } => 0,
        };
        size_of::<Self>().saturating_add(heap)
    }
}

fn operand_contracts(operands: &[(QuantityTypeId, crate::IndexSet)]) -> String {
    operands
        .iter()
        .map(|(quantity, indices)| {
            let axes = indices
                .iter()
                .map(|index| format!("{}@{}:{}", index.bound_index, index.domain, index.kind))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{quantity}[{axes}]")
        })
        .collect::<Vec<_>>()
        .join("; ")
}

pse_diagnostics::impl_diagnostic! {
    DimensionError,
    code(this) { match this {
            Self::ZeroDenominator | Self::Overflow { .. } | Self::NotReduced { .. } => Some(pse_diagnostics::DiagnosticCode::ValidationInvariant),


            _ => None,
        } },
    forward(_this) { None },
    help(_this) { None },
    related(_this) { None },
    source(_this) { None }
}

pse_diagnostics::impl_diagnostic! {
    QuantityError,
    code(this) { match this {
            Self::Registry { .. } | Self::Dimension(..) => Some(pse_diagnostics::DiagnosticCode::ValidationInvariant),
            Self::UnknownId { .. } => Some(pse_diagnostics::DiagnosticCode::QuantityUnknownId),
            Self::Incompatible { .. } => Some(pse_diagnostics::DiagnosticCode::QuantityIncompatible),
            Self::UnitConvertMismatch { .. } => Some(pse_diagnostics::DiagnosticCode::QuantityUnitConversion),
            Self::ContractMismatch { .. } => Some(pse_diagnostics::DiagnosticCode::QuantityContractMismatch),
            Self::NonfiniteMagnitude { .. } | Self::NonfiniteConversion { .. } => Some(pse_diagnostics::DiagnosticCode::QuantityNonfinite),
            Self::InferencePrecondition { .. } | Self::OperationUnsupported { .. } | Self::UnregisteredResultType { .. } => Some(pse_diagnostics::DiagnosticCode::CompileMathQuantityOperationUnsupported),



            Self::AmbiguousLiteral { .. } | Self::UnknownUnitSymbol { .. } | Self::AffineUnitFactor { .. } => Some(pse_diagnostics::DiagnosticCode::CompileMathUnitInconsistent),


            Self::StaticDomain { .. } => Some(pse_diagnostics::DiagnosticCode::CompileMathDomainViolationStatic),


            _ => None,
        } },
    forward(_this) { None },
    help(_this) { None },
    related(_this) { None },
    source(_this) { None },
    facts(this) {
        use pse_diagnostics::{DiagnosticFacts, DiagnosticObservation as O, DiagnosticRule as R, OperandContract};
        let mut facts = DiagnosticFacts::default();
        facts.observe("detail", O::Text(this.to_string()));
        match this {
            Self::Incompatible { operands, hint, reason } => {
                facts.rule = Some(R::QuantityIncompatible);
                facts.observe("reason", O::Text(reason.to_string()));
                facts.observe("operands", O::Contracts(operands.iter().map(|(quantity, axes)| OperandContract {
                    quantity: *quantity.as_bytes(), indices: axes.iter().map(|axis| [*axis.bound_index.as_bytes(), *axis.domain.as_bytes(), *axis.kind.as_bytes()]).collect()
                }).collect()));
                facts.sources.extend(operands.iter().map(|(quantity,_)| *quantity.as_bytes()));
                if let Some(hint) = hint { facts.sources.push(*hint.as_bytes()); }
            }
            Self::ContractMismatch { expected, actual, component } => {
                facts.rule = Some(R::QuantityContractMismatch);
                facts.observe("component", O::Text(component.to_string()));
                facts.observe("operands", O::Contracts([expected, actual].into_iter().map(|quantity| OperandContract {quantity: *quantity.as_bytes(), indices: Vec::new()}).collect()));
                facts.sources.extend([*expected.as_bytes(), *actual.as_bytes()]);
            }
            Self::UnitConvertMismatch {declared_from, operand_unit, to} => {
                facts.rule = Some(R::QuantityUnitConversion);
                facts.sources.extend([*declared_from.as_bytes(), *operand_unit.as_bytes(), *to.as_bytes()]);
                for (key, value) in [("declared_from",declared_from),("operand_unit",operand_unit),("to",to)] { facts.observe(key, O::Text(value.to_string())); }
            }
            Self::NonfiniteMagnitude {quantity, unit, value} => {
                facts.rule = Some(R::QuantityNonfinite); facts.sources.extend([*quantity.as_bytes(),*unit.as_bytes()]); facts.observe("value", O::Number(*value));
            }
            Self::NonfiniteConversion {quantity,from,to,value,scale,offset} => {
                facts.rule = Some(R::QuantityNonfinite); facts.sources.extend([*quantity.as_bytes(),*from.as_bytes(),*to.as_bytes()]);
                for (key,value) in [("value",value),("scale",scale),("offset",offset)] { facts.observe(key,O::Number(*value)); }
            }
            Self::UnknownId {id,kind} => { facts.rule=Some(R::QuantityUnknownId);facts.sources.push(*id.as_bytes()); facts.observe("kind",O::Text((*kind).into())); }
            Self::StaticDomain {value,..} => {facts.rule=Some(R::MathDomain);facts.observe("value",O::Number(*value));}
            Self::Registry {subject,..} => {facts.rule=Some(R::MathQuantity); facts.sources.push(*subject.as_bytes());}
            Self::InferencePrecondition {..} | Self::Dimension(_) | Self::OperationUnsupported {..} | Self::UnregisteredResultType {..} | Self::AmbiguousLiteral {..} | Self::UnknownUnitSymbol {..} | Self::AffineUnitFactor {..} => facts.rule=Some(R::MathQuantity),
        }
        facts
    }
}

#[cfg(test)]
mod tests {
    use pse_diagnostics::TypedDiagnostic;
    use pse_ids::SemanticId;

    use super::{ContractComponent, DimensionError, IncompatibilityReason, QuantityError};
    use crate::enums::Opcode;
    use crate::ids::{QuantityTypeId, UnitId};

    /// The §23.2 class of a variant, as a string, so the mapping is asserted and not
    /// merely written down.
    fn code_of(error: &QuantityError) -> Option<pse_diagnostics::DiagnosticCode> {
        error.diagnostic_code()
    }

    #[test]
    fn each_variant_carries_its_section_23_2_class() {
        let id = SemanticId::NIL;
        assert_eq!(
            code_of(&QuantityError::UnknownId { kind: "unit", id }),
            Some(pse_diagnostics::DiagnosticCode::QuantityUnknownId)
        );
        assert_eq!(
            code_of(&QuantityError::Dimension(DimensionError::ZeroDenominator)),
            Some(pse_diagnostics::DiagnosticCode::ValidationInvariant)
        );
        assert_eq!(
            code_of(&QuantityError::OperationUnsupported {
                opcode: Opcode::Mul,
                input_kinds: Vec::new(),
                ordered_matches: 0,
                swapped_matches: 0,
            }),
            Some(pse_diagnostics::DiagnosticCode::CompileMathQuantityOperationUnsupported)
        );
        assert_eq!(
            code_of(&QuantityError::Incompatible {
                reason: IncompatibilityReason::PointPlusPoint,
                operands: Vec::new(),
                hint: None,
            }),
            Some(pse_diagnostics::DiagnosticCode::QuantityIncompatible)
        );
        assert_eq!(
            code_of(&QuantityError::StaticDomain {
                opcode: Opcode::Log,
                restriction: "a positive argument",
                value: 0.0,
            }),
            Some(pse_diagnostics::DiagnosticCode::CompileMathDomainViolationStatic)
        );
        assert_eq!(
            code_of(&QuantityError::NonfiniteMagnitude {
                quantity: id.into(),
                unit: id.into(),
                value: f64::NAN,
            }),
            Some(pse_diagnostics::DiagnosticCode::QuantityNonfinite)
        );
        assert_eq!(
            code_of(&QuantityError::NonfiniteConversion {
                quantity: id.into(),
                from: id.into(),
                to: id.into(),
                value: f64::MAX,
                scale: 2.0,
                offset: 0.0,
            }),
            Some(pse_diagnostics::DiagnosticCode::QuantityNonfinite)
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
    fn source_facts_preserve_complete_operand_contract_and_free_index_identity() {
        let quantity = QuantityTypeId::from_bytes([1; 16]);
        let index = crate::BoundIndexRef::new(
            crate::BoundIndexId::from_bytes([2; 16]),
            crate::DomainId::from_bytes([3; 16]),
            crate::EntityKindId::from_bytes([4; 16]),
        );
        let error = QuantityError::Incompatible {
            reason: IncompatibilityReason::PointPlusPoint,
            operands: vec![(quantity, crate::IndexSet::try_from_iter([index]).unwrap())],
            hint: None,
        };
        let facts = error.diagnostic_facts();
        assert_eq!(
            facts.rule,
            Some(pse_diagnostics::DiagnosticRule::QuantityIncompatible)
        );
        assert_eq!(
            error.diagnostic_code(),
            facts.rule.map(pse_diagnostics::DiagnosticRule::code)
        );
        let pse_diagnostics::DiagnosticObservation::Contracts(contracts) =
            &facts.observations["operands"]
        else {
            panic!("operand contract missing");
        };
        assert_eq!(contracts[0].quantity, [1; 16]);
        assert_eq!(contracts[0].indices, vec![[[2; 16], [3; 16], [4; 16]]]);
    }

    #[test]
    fn a_dimension_error_converts_into_a_quantity_error() {
        let error: QuantityError = DimensionError::Overflow { num: 1, den: 2 }.into();
        assert!(matches!(error, QuantityError::Dimension(_)));
        assert_eq!(
            code_of(&error),
            Some(pse_diagnostics::DiagnosticCode::ValidationInvariant)
        );
    }
}
