// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed DSL errors with byte offsets.

use super::ast::Span;

/// An authored expression that fails the bounded grammar.
#[derive(Debug, thiserror::Error)]
pub enum DslError {
    /// A grammar token or form did not match.
    #[error("{expected} expected at byte {offset}, found {found}")]
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
    /// Unary minus next to power requires explicit parentheses.
    #[error("ambiguous unary power at byte {offset}: use (-x)^2 or -(x^2)")]
    AmbiguousUnaryPower {
        /// Unary minus position.
        offset: u32,
    },
    /// Nonfinite numbers have no admitted authored meaning.
    #[error("nonfinite numeric literal at byte {offset}")]
    NonFiniteNumber {
        /// Numeric token position.
        offset: u32,
    },
    /// Parsing exceeded the declared resource envelope.
    #[error("DSL {limit} budget exceeded: {needed} exceeds {allowed}")]
    Budget {
        /// Resource name.
        limit: &'static str,
        /// Admitted maximum.
        allowed: u64,
        /// Observed requirement.
        needed: u64,
    },
}

pse_diagnostics::impl_diagnostic! {
    DslError,
    code(this) { match this {
            Self::Syntax { .. } => Some(pse_diagnostics::DiagnosticCode::AuthoringParseSyntax),
            Self::AmbiguousUnaryPower { .. } => Some(pse_diagnostics::DiagnosticCode::AuthoringParseAmbiguousUnaryPower),
            Self::NonFiniteNumber { .. } => Some(pse_diagnostics::DiagnosticCode::AuthoringParseNonfiniteNumber),
            Self::Budget { .. } => Some(pse_diagnostics::DiagnosticCode::AuthoringParseBudget),
            _ => None,
        } },
    forward(_this) { None },
    help(_this) { None },
    related(_this) { None },
    source(_this) { None }
}
