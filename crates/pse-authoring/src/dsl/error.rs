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
    source(_this) { None },
    facts(this) {
        use pse_diagnostics::{DiagnosticFacts, DiagnosticRule as R, DiagnosticObservation as O};
        let mut facts = DiagnosticFacts {rule:Some(match this {Self::Budget{..}=>R::AuthoringBudget, Self::Syntax{..} | Self::AmbiguousUnaryPower{..} | Self::NonFiniteNumber{..}=>R::AuthoringSyntax}), ..Default::default()};
        facts.observe("detail",O::Text(this.to_string()));
        match this {
            Self::Syntax{offset,expected,found,..} => {facts.observe("offset",O::Integer(i64::from(*offset)));facts.observe("expected",O::Text(expected.clone()));facts.observe("found",O::Text(found.clone()));}
            Self::AmbiguousUnaryPower{offset}|Self::NonFiniteNumber{offset}=>facts.observe("offset",O::Integer(i64::from(*offset))),
            Self::Budget{limit,allowed,needed}=>{facts.observe("limit",O::Text((*limit).into()));for (key,value) in [("allowed",allowed),("needed",needed)]{facts.observe(key,i64::try_from(*value).map_or_else(|_|O::Text(value.to_string()),O::Integer));}}
        }
        facts
    }
}
