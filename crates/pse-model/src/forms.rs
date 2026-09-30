// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Constraint forms that a lowering leaves to a native constraint handler (ADR-0104).
//!
//! A native realization keeps the authored structure instead of a linear reformulation.
//! Only a backend with the matching handler may execute it; every other route refuses the
//! case. Identities are specialized symbol and row identities. The handler each form needs
//! is the registry enumeration `NativeConstraintForm`, which backend capability records
//! also publish.
use crate::generated::enums::NativeConstraintForm;
use pse_ids::SemanticId;

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
        form: NativeConstraintForm,
        /// Members and their weights.
        members: Vec<(SemanticId, f64)>,
    },
    /// `resultant = op(operands)` for `And`, `Or` or `Xor`.
    Logic {
        /// `And`, `Or` or `Xor`.
        form: NativeConstraintForm,
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
    pub const fn form(&self) -> NativeConstraintForm {
        match self {
            Self::Indicator { .. } => NativeConstraintForm::Indicator,
            Self::Sos { form, .. } | Self::Logic { form, .. } => *form,
            Self::Cardinality { .. } => NativeConstraintForm::Cardinality,
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
