// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The boolean and scalar expressions a rule plan is written in
//! (blueprint §6.11 `reference.rule_expr_*`, §14.2).
//!
//! Deliberately smaller than the math IR of §7: a rule expression decides membership, it
//! does not model physics. The four-valued shape of §14.2 rule 3 is visible here —
//! [`RuleExpr::IsUnknown`], [`RuleExpr::IsDistinctFrom`] and their negations exist because
//! a null-versus-null candidate must be neither dropped nor matched by accident.

use core::fmt;

use crate::model::cell::Cell;

/// A comparison operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CmpOp {
    /// `=`.
    Eq,
    /// `<>`.
    NotEq,
    /// `<`.
    Lt,
    /// `<=`.
    LtEq,
    /// `>`.
    Gt,
    /// `>=`.
    GtEq,
}

impl CmpOp {
    /// The wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eq => "eq",
            Self::NotEq => "not_eq",
            Self::Lt => "lt",
            Self::LtEq => "lt_eq",
            Self::Gt => "gt",
            Self::GtEq => "gt_eq",
        }
    }
}

impl fmt::Display for CmpOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An expression over a rule plan's columns (blueprint §14.2).
#[derive(Clone, Debug, PartialEq)]
pub enum RuleExpr {
    /// A column of the input.
    Col(&'static str),
    /// A literal.
    Lit(Cell),
    /// Kleene conjunction of every operand.
    And(Vec<RuleExpr>),
    /// Kleene disjunction of every operand.
    Or(Vec<RuleExpr>),
    /// Kleene negation.
    Not(Box<RuleExpr>),
    /// A comparison.
    Cmp {
        /// The operator.
        op: CmpOp,
        /// The left operand.
        l: Box<RuleExpr>,
        /// The right operand.
        r: Box<RuleExpr>,
    },
    /// `IS NULL`.
    IsNull(Box<RuleExpr>),
    /// `IS NOT NULL`.
    IsNotNull(Box<RuleExpr>),
    /// `IS DISTINCT FROM`: null-safe inequality.
    IsDistinctFrom(Box<RuleExpr>, Box<RuleExpr>),
    /// `IS NOT DISTINCT FROM`: null-safe equality.
    IsNotDistinctFrom(Box<RuleExpr>, Box<RuleExpr>),
    /// Membership in a literal list.
    InList {
        /// The probe.
        expr: Box<RuleExpr>,
        /// The literal members.
        list: Vec<Cell>,
    },
    /// A struct field.
    Field {
        /// The struct-valued operand.
        expr: Box<RuleExpr>,
        /// The child name.
        name: &'static str,
    },
    /// The length of a list.
    ListLen(Box<RuleExpr>),
    /// `IS TRUE`.
    IsTrue(Box<RuleExpr>),
    /// `IS FALSE`.
    IsFalse(Box<RuleExpr>),
    /// `IS UNKNOWN`: the third truth value, explicit (blueprint §14.2 rule 3).
    IsUnknown(Box<RuleExpr>),
}

impl RuleExpr {
    /// A column reference.
    pub const fn col(name: &'static str) -> Self {
        Self::Col(name)
    }

    /// A comparison of two expressions.
    pub fn cmp(op: CmpOp, l: Self, r: Self) -> Self {
        Self::Cmp {
            op,
            l: Box::new(l),
            r: Box::new(r),
        }
    }

    /// The node's operator name, for the `reference.rule_expr_*` rows.
    pub const fn op(&self) -> &'static str {
        match self {
            Self::Col(_) => "col",
            Self::Lit(_) => "lit",
            Self::And(_) => "and",
            Self::Or(_) => "or",
            Self::Not(_) => "not",
            Self::Cmp { .. } => "cmp",
            Self::IsNull(_) => "is_null",
            Self::IsNotNull(_) => "is_not_null",
            Self::IsDistinctFrom(_, _) => "is_distinct_from",
            Self::IsNotDistinctFrom(_, _) => "is_not_distinct_from",
            Self::InList { .. } => "in_list",
            Self::Field { .. } => "field",
            Self::ListLen(_) => "list_len",
            Self::IsTrue(_) => "is_true",
            Self::IsFalse(_) => "is_false",
            Self::IsUnknown(_) => "is_unknown",
        }
    }

    /// Every column this expression reads, in first-seen order.
    pub fn columns(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        self.collect_columns(&mut out);
        out
    }

    /// Appends this expression's columns to `out`, skipping repeats.
    fn collect_columns(&self, out: &mut Vec<&'static str>) {
        match self {
            Self::Col(name) => {
                if !out.contains(name) {
                    out.push(name);
                }
            }
            Self::Lit(_) => {}
            Self::And(operands) | Self::Or(operands) => {
                for operand in operands {
                    operand.collect_columns(out);
                }
            }
            Self::Not(inner)
            | Self::IsNull(inner)
            | Self::IsNotNull(inner)
            | Self::ListLen(inner)
            | Self::IsTrue(inner)
            | Self::IsFalse(inner)
            | Self::IsUnknown(inner)
            | Self::Field { expr: inner, .. }
            | Self::InList { expr: inner, .. } => inner.collect_columns(out),
            Self::Cmp { l, r, .. } | Self::IsDistinctFrom(l, r) | Self::IsNotDistinctFrom(l, r) => {
                l.collect_columns(out);
                r.collect_columns(out);
            }
        }
    }
}
