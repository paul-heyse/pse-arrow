// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Structured boundary failures retain model identity and observed values.
pub use crate::generated::enums::NativeBoundaryClass as BoundaryClass;
use pse_ids::SemanticId;

/// An observed value is distinct from absent evidence and human-readable prose.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Observation {
    /// A finite or explicitly nonfinite observed floating value.
    Real(f64),
    /// An exact count or code.
    Integer(i64),
    /// A checked predicate.
    Boolean(bool),
    /// A native text observation.
    Text(String),
}
/// Stable structured cause shared by Rust and the public error projection.
#[derive(Clone, Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
#[error("{stage}: {rule} ({class:?}; sources {sources:?})")]
pub struct BoundaryDiagnostic {
    /// Machine-readable disposition; callers never parse the message.
    pub class: BoundaryClass,
    /// Operation boundary, such as selected admission or provider registration.
    pub stage: String,
    /// All affected authored identities, in deterministic order.
    pub sources: Vec<SemanticId>,
    /// The violated named contract, not a replacement for source identities.
    pub rule: String,
    /// Values observed while checking the contract.
    pub observations: std::collections::BTreeMap<String, Observation>,
}
impl BoundaryDiagnostic {
    /// Construct an attributable boundary error; add observations when available.
    pub fn new(
        class: BoundaryClass,
        stage: impl Into<String>,
        sources: impl IntoIterator<Item = SemanticId>,
        rule: impl Into<String>,
    ) -> Self {
        let mut sources: Vec<_> = sources.into_iter().collect();
        sources.sort_unstable();
        sources.dedup();
        Self {
            class,
            stage: stage.into(),
            sources,
            rule: rule.into(),
            observations: Default::default(),
        }
    }
}
pse_diagnostics::impl_diagnostic! {
    BoundaryDiagnostic,
    code(this) { Some(match this.class {
        BoundaryClass::InvalidModel => pse_diagnostics::DiagnosticCode::ValidationInvariant,
        BoundaryClass::Unsupported => pse_diagnostics::DiagnosticCode::CapabilityBackend,
        BoundaryClass::ResourceLimit => pse_diagnostics::DiagnosticCode::RuntimeResourceLimit,
        BoundaryClass::TrialRejected | BoundaryClass::Nonfinite => pse_diagnostics::DiagnosticCode::SolveEvaluationError,
        BoundaryClass::Infrastructure | BoundaryClass::Conflict | BoundaryClass::Incompatible => pse_diagnostics::DiagnosticCode::RuntimeInfrastructure,
        BoundaryClass::Cancelled => pse_diagnostics::DiagnosticCode::RuntimeCancelled,
        BoundaryClass::Internal => pse_diagnostics::DiagnosticCode::InternalInvariant,
    }) },
    forward(_this) {None}, help(_this) {None}, related(_this) {None}, source(_this) {None}
}
