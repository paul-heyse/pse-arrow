// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Runtime presentation of an original shared crossing refusal.
use pse_contract_proto::support::BoundaryProjectionError;

/// Consumer-owned diagnostic view retaining the original generic crossing error.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ProjectionDiagnosticError(#[from] BoundaryProjectionError);
impl ProjectionDiagnosticError {
    /// Retain the original shared failure's actual owned allocation extent.
    pub fn retained_bytes(&self) -> Result<usize, pse_contract_proto::support::RetainedExtentError> {
        self.0.retained_bytes()
    }
}

pse_diagnostics::impl_diagnostic! {
    ProjectionDiagnosticError,
    code(this) {
        // Stable declared diagnostic identifiers are a finite vocabulary bridge;
        // rendered error messages never determine failure classification.
        miette::Diagnostic::code(&this.0).and_then(|code| pse_diagnostics::DiagnosticCode::parse(&code.to_string()))
    },
    forward(this) { match &this.0 {
        BoundaryProjectionError::Adoption(error) | BoundaryProjectionError::OutgoingAdoption(error) => Some(error),
        BoundaryProjectionError::OutgoingNative(error) => Some(error),
        _ => None,
    } },
    help(_this) { None }, related(_this) { None }, source(this) { Some(&this.0) },
    facts(this) {
        let mut facts = pse_diagnostics::DiagnosticFacts::default();
        if let Some(error) = this.0.wire_validation() {
            if let Some(detail) = &error.compile_error { facts.observe("rule_compilation_error", pse_diagnostics::DiagnosticObservation::Text(detail.clone())); }
            if let Some(detail) = &error.runtime_error { facts.observe("rule_evaluation_error", pse_diagnostics::DiagnosticObservation::Text(detail.clone())); }
            for (index, violation) in error.violations.iter().enumerate() {
                facts.observe(format!("violation_{index}_field_path"), pse_diagnostics::DiagnosticObservation::Text(violation.field.to_string()));
                facts.observe(format!("violation_{index}_rule_id"), pse_diagnostics::DiagnosticObservation::Text(violation.rule_id.to_string()));
                facts.observe(format!("violation_{index}_detail"), pse_diagnostics::DiagnosticObservation::Text(violation.message.to_string()));
            }
        }
        facts
    }
}
impl pse_model::diagnostic::DiagnosticProjection for ProjectionDiagnosticError {}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_diagnostics::TypedDiagnostic;

    #[test]
    fn original_validator_infrastructure_and_facts_survive_runtime_presentation() {
        let original = protovalidate_buffa::ValidationError { compile_error: Some("actual original compile failure".into()), ..Default::default() };
        let error = ProjectionDiagnosticError::from(BoundaryProjectionError::WireValidation(original));
        assert_eq!(error.diagnostic_code(), Some(pse_diagnostics::DiagnosticCode::WorkflowInternal));
        assert!(matches!(error.diagnostic_facts().observations.get("rule_compilation_error"), Some(pse_diagnostics::DiagnosticObservation::Text(detail)) if detail == "actual original compile failure"));
        let mut cause: Option<&dyn std::error::Error> = Some(&error);
        let mut preserved = false;
        while let Some(current) = cause {
            preserved |= current.downcast_ref::<protovalidate_buffa::ValidationError>().is_some();
            cause = current.source();
        }
        assert!(preserved, "the typed original validator failure remains in the cause chain");
    }
}
