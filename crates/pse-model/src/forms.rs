// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Constraint forms that a lowering leaves to a native constraint handler (ADR-0104).
//!
//! A native realization keeps the authored structure instead of a linear reformulation.
//! Only a backend with the matching handler may execute it; every other route refuses the
//! case. Identities are specialized symbol and row identities.
use pse_ids::SemanticId;

/// The constraint handler a native form requires.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NativeForm {
    /// A row enforced only while a binary variable takes a declared value.
    Indicator,
    /// At most one nonzero member.
    Sos1,
    /// At most two consecutive nonzero members.
    Sos2,
    /// A resultant equal to the conjunction of its operands.
    And,
    /// A resultant equal to the disjunction of its operands.
    Or,
    /// A resultant equal to the exclusive disjunction of two operands.
    Xor,
    /// At most a bounded number of nonzero members.
    Cardinality,
}
impl NativeForm {
    /// Stable diagnostic spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Indicator => "indicator",
            Self::Sos1 => "sos1",
            Self::Sos2 => "sos2",
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            Self::Cardinality => "cardinality",
        }
    }
}
/// A logical operand: a binary variable, possibly negated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicOperand {
    /// Binary variable identity.
    pub variable: SemanticId,
    /// The operand is the variable's complement.
    pub negated: bool,
}
/// One native constraint over specialized identities.
#[derive(Clone, Debug, PartialEq)]
pub enum NativeConstraint {
    /// `row` holds only while `variable` equals `active`.
    Indicator {
        /// Conditional row; a backend without the handler would enforce it always.
        row: SemanticId,
        /// Binary indicator.
        variable: SemanticId,
        /// The indicator value that activates the row.
        active: bool,
    },
    /// A special ordered set of type 1 or 2, ordered by strictly increasing weights.
    Sos {
        /// `Sos1` or `Sos2`.
        form: NativeForm,
        /// Members and their weights.
        members: Vec<(SemanticId, f64)>,
    },
    /// `resultant = op(operands)` for `And`, `Or` or `Xor`.
    Logic {
        /// `And`, `Or` or `Xor`.
        form: NativeForm,
        /// Binary resultant.
        resultant: SemanticId,
        /// Binary operands.
        operands: Vec<LogicOperand>,
    },
    /// At most `bound` of `members` are nonzero.
    Cardinality {
        /// Members with finite bounds.
        members: Vec<SemanticId>,
        /// Largest nonzero count.
        bound: u32,
    },
}
impl NativeConstraint {
    /// The handler this constraint needs.
    pub const fn form(&self) -> NativeForm {
        match self {
            Self::Indicator { .. } => NativeForm::Indicator,
            Self::Sos { form, .. } | Self::Logic { form, .. } => *form,
            Self::Cardinality { .. } => NativeForm::Cardinality,
        }
    }
    /// Every row and variable identity the constraint refers to.
    pub fn identities(&self) -> Vec<SemanticId> {
        match self {
            Self::Indicator { row, variable, .. } => vec![*row, *variable],
            Self::Sos { members, .. } => members.iter().map(|(id, _)| *id).collect(),
            Self::Logic {
                resultant,
                operands,
                ..
            } => std::iter::once(*resultant)
                .chain(operands.iter().map(|o| o.variable))
                .collect(),
            Self::Cardinality { members, .. } => members.clone(),
        }
    }
    /// Canonical framing for structure identity.
    pub fn frame(&self, h: &mut pse_ids::FramedHasher) {
        h.str(self.form().as_str());
        match self {
            Self::Indicator {
                row,
                variable,
                active,
            } => {
                h.id(row).id(variable).bool(*active);
            }
            Self::Sos { members, .. } => {
                h.u64(members.len() as u64);
                for (id, weight) in members {
                    h.id(id).u64(pse_ids::canonical_f64_bits(*weight));
                }
            }
            Self::Logic {
                resultant,
                operands,
                ..
            } => {
                h.id(resultant).u64(operands.len() as u64);
                for operand in operands {
                    h.id(&operand.variable).bool(operand.negated);
                }
            }
            Self::Cardinality { members, bound } => {
                h.u64(u64::from(*bound)).u64(members.len() as u64);
                for id in members {
                    h.id(id);
                }
            }
        }
    }
}
