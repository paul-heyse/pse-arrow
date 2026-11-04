// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use crate::{DiagnosticCode, DiagnosticFacts, DiagnosticObservation, TypedDiagnostic};
use pse_contract::boundary_projection::{
    AdapterError, AdapterFailure, ProjectionError, ValidationError,
};
impl TypedDiagnostic for ProjectionError {
    fn diagnostic_code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode::ValidationInvariant)
    }
}
impl TypedDiagnostic for ValidationError {
    fn diagnostic_code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode::WorkflowInput)
    }
    fn diagnostic_facts(&self) -> DiagnosticFacts {
        let Self::Rule { path, rule, reason } = self;
        let mut facts = DiagnosticFacts::default();
        facts.observe("field_path", DiagnosticObservation::Text(path.clone()));
        facts.observe("local_rule", DiagnosticObservation::Text((*rule).into()));
        facts.observe("detail", DiagnosticObservation::Text(reason.clone()));
        facts
    }
}
impl TypedDiagnostic for AdapterError {
    fn diagnostic_code(&self) -> Option<DiagnosticCode> {
        match self {
            Self::Refused {
                kind: AdapterFailure::LayoutMismatch,
                ..
            } => Some(DiagnosticCode::WorkflowInternal),
            Self::Refused { .. } => Some(DiagnosticCode::WorkflowInput),
            Self::Validation(error) => error.diagnostic_code(),
        }
    }
    fn diagnostic_facts(&self) -> DiagnosticFacts {
        match self {
            Self::Validation(error) => error.diagnostic_facts(),
            Self::Refused { path, kind, reason } => {
                let mut facts = DiagnosticFacts::default();
                facts.observe("field_path", DiagnosticObservation::Text(path.clone()));
                facts.observe(
                    "adoption_category",
                    DiagnosticObservation::Text(format!("{kind:?}")),
                );
                facts.observe("detail", DiagnosticObservation::Text(reason.clone()));
                facts
            }
        }
    }
    fn diagnostic_children(&self) -> Option<Box<dyn Iterator<Item = &dyn TypedDiagnostic> + '_>> {
        match self {
            Self::Validation(error) => {
                let child: &dyn TypedDiagnostic = error;
                Some(Box::new(std::iter::once(child)))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_contract_errors_keep_codes_facts_and_original_validation_source() {
        let error = AdapterError::Validation(ValidationError::Rule {
            path: "source.values[2]".into(),
            rule: "finite",
            reason: "nonfinite".into(),
        });
        assert_eq!(error.diagnostic_code(), Some(DiagnosticCode::WorkflowInput));
        assert_eq!(
            error.failure_class(),
            Some(DiagnosticCode::WorkflowInput.class())
        );
        assert_eq!(
            miette::Diagnostic::code(&error).unwrap().to_string(),
            "workflow.input"
        );
        assert_eq!(error.diagnostic_children().unwrap().count(), 1);
        let source = miette::Diagnostic::diagnostic_source(&error).unwrap();
        assert!(source.to_string().contains("source.values[2]"));
        let facts = error.diagnostic_facts();
        assert!(
            matches!(facts.observations.get("local_rule"), Some(DiagnosticObservation::Text(value)) if value == "finite")
        );
    }

    #[test]
    fn layout_mismatch_remains_internal_and_keeps_adoption_category() {
        let error = AdapterError::refused("owner", AdapterFailure::LayoutMismatch, "stale fields");
        assert_eq!(
            error.diagnostic_code(),
            Some(DiagnosticCode::WorkflowInternal)
        );
        assert_eq!(
            miette::Diagnostic::code(&error).unwrap().to_string(),
            "workflow.internal"
        );
        assert!(miette::Diagnostic::diagnostic_source(&error).is_none());
        assert!(
            matches!(error.diagnostic_facts().observations.get("adoption_category"),
            Some(DiagnosticObservation::Text(value)) if value == "LayoutMismatch")
        );
    }
}
