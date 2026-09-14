// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed DSL errors with byte offsets.

use super::ast::Span;

/// An authored expression that fails the bounded grammar.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum DslError {
    /// A grammar token or form did not match.
    #[error("{expected} expected at byte {offset}, found {found}")]
    #[diagnostic(code(authoring::parse::syntax))]
    Syntax {
        /// First byte that did not match.
        offset: u32,
        /// Its exact source range.
        span: Span,
        /// Required token or form.
        expected: String,
        /// Encountered token or end of input.
        found: String,
    },
    /// Weighted means require ordered pairs and at least one pair.
    #[error("weighted_mean requires a nonempty even argument list at byte {offset}")]
    #[diagnostic(code(authoring::parse::weighted_mean_arity))]
    WeightedMeanArity {
        /// Start of the call.
        offset: u32,
    },
    /// Unary minus next to power requires explicit parentheses.
    #[error("ambiguous unary power at byte {offset}: use (-x)^2 or -(x^2)")]
    #[diagnostic(code(authoring::parse::ambiguous_unary_power))]
    AmbiguousUnaryPower {
        /// Unary minus position.
        offset: u32,
    },
    /// Nonfinite numbers have no admitted authored meaning.
    #[error("nonfinite numeric literal at byte {offset}")]
    #[diagnostic(code(authoring::parse::nonfinite_number))]
    NonFiniteNumber {
        /// Numeric token position.
        offset: u32,
    },
    /// Parsing exceeded the declared resource envelope.
    #[error("DSL {limit} budget exceeded: {needed} exceeds {allowed}")]
    #[diagnostic(code(authoring::parse::budget))]
    Budget {
        /// Resource name.
        limit: &'static str,
        /// Admitted maximum.
        allowed: u64,
        /// Observed requirement.
        needed: u64,
    },
}
