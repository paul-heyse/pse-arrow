// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed invariant planning and execution failures (blueprint §23.2).
/// An invariant that cannot be planned or executed.
#[derive(Debug, thiserror::Error)]
pub enum RuleError {
    /// Original classified native engine failure.
    #[error(transparent)]
    Engine(#[from] pse_columnar::EngineError),
    /// Checked relation, cancellation or allocation failure.
    #[error(transparent)]
    Relation(#[from] pse_relations::RelationError),
    /// Session failure with its original diagnostic classification.
    #[error(transparent)]
    Catalog(#[from] pse_engine::EngineError),
    /// A platform postcondition failed.
    #[error("internal invariant: {what}")]
    Internal {
        /// What did not hold.
        what: String,
    },
}
pse_diagnostics::impl_diagnostic! {
    RuleError,
    code(this) { match this {
        Self::Internal { .. } => Some(pse_diagnostics::DiagnosticCode::InternalInvariant),
        _ => None,
    } },
    forward(this) { match this {
        Self::Engine(value) => Some(value),
        Self::Relation(value) => Some(value),
        Self::Catalog(value) => Some(value),
        Self::Internal { .. } => None,
    } },
    help(_this) { None },
    related(_this) { None },
    source(_this) { None }
}
pse_columnar::impl_native_error!(RuleError);
