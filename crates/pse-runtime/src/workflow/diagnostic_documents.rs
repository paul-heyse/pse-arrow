// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable observation of native diagnostic evidence, independent of Python transport.

/// One retained original message in an error's source chain.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticCauseDocument {
    /// Original source message, with no reclassification.
    pub message: String,
}
/// Query source coordinates under the native engine's line and column convention.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticSpanDocument {
    /// Starting source line.
    pub start_line: u64,
    /// Starting source column.
    pub start_column: u64,
    /// Ending source line.
    pub end_line: u64,
    /// Ending source column.
    pub end_column: u64,
}
impl From<datafusion::common::Span> for DiagnosticSpanDocument {
    fn from(span: datafusion::common::Span) -> Self {
        Self {
            start_line: span.start.line,
            start_column: span.start.column,
            end_line: span.end.line,
            end_column: span.end.column,
        }
    }
}
/// One native query note or help, retaining its optional span.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticNoteDocument {
    /// Original native text.
    pub message: String,
    /// Original query source span when supplied.
    pub span: Option<DiagnosticSpanDocument>,
}
/// One original query annotation, including every grouped note and help.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticAnnotationDocument {
    /// Canonical error or warning severity of the native annotation.
    pub kind: pse_model::generated::enums::DiagnosticSeverity,
    /// Original native annotation message.
    pub message: String,
    /// Original annotation span when supplied.
    pub span: Option<DiagnosticSpanDocument>,
    /// Original ordered notes.
    pub notes: Vec<DiagnosticNoteDocument>,
    /// Original ordered help suggestions.
    pub helps: Vec<DiagnosticNoteDocument>,
}
impl From<&datafusion::common::diagnostic::Diagnostic> for DiagnosticAnnotationDocument {
    fn from(diagnostic: &datafusion::common::diagnostic::Diagnostic) -> Self {
        use pse_model::generated::enums::DiagnosticSeverity;
        Self {
            kind: match diagnostic.kind {
                datafusion::common::diagnostic::DiagnosticKind::Error => DiagnosticSeverity::Error,
                datafusion::common::diagnostic::DiagnosticKind::Warning => {
                    DiagnosticSeverity::Warning
                }
            },
            message: diagnostic.message.clone(),
            span: diagnostic.span.map(Into::into),
            notes: diagnostic
                .notes
                .iter()
                .map(|note| DiagnosticNoteDocument {
                    message: note.message.clone(),
                    span: note.span.map(Into::into),
                })
                .collect(),
            helps: diagnostic
                .helps
                .iter()
                .map(|help| DiagnosticNoteDocument {
                    message: help.message.clone(),
                    span: help.span.map(Into::into),
                })
                .collect(),
        }
    }
}
/// One typed native leaf and its ordered execution contexts.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticContextDocument {
    /// The authoritative typed leaf code.
    pub code: pse_diagnostics::DiagnosticCode,
    /// Original leaf message.
    pub message: String,
    /// Ordered native execution wrappers.
    pub contexts: Vec<String>,
    /// Original grouped query annotations.
    pub diagnostics: Vec<DiagnosticAnnotationDocument>,
}
impl From<pse_columnar::Observation<'_>> for DiagnosticContextDocument {
    fn from(observation: pse_columnar::Observation<'_>) -> Self {
        Self {
            code: observation.code,
            message: observation
                .domain_cause
                .map_or_else(|| observation.cause.to_string(), ToString::to_string),
            contexts: observation
                .contexts
                .into_iter()
                .map(str::to_owned)
                .collect(),
            diagnostics: observation
                .diagnostics
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[cfg(test)]
mod boundary_unit {
    use super::*;
    #[test]
    fn query_annotations_retain_grouped_notes_and_help() {
        let native =
            datafusion::common::diagnostic::Diagnostic::new_warning("native warning", None)
                .with_note("first occurrence", None)
                .with_note("second occurrence", None)
                .with_help("native suggestion", None);
        let document = DiagnosticAnnotationDocument::from(&native);
        let encoded = serde_json::to_vec(&document).unwrap();
        let retained: DiagnosticAnnotationDocument = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(
            retained.kind,
            pse_model::generated::enums::DiagnosticSeverity::Warning
        );
        assert_eq!(
            retained
                .notes
                .iter()
                .map(|note| note.message.as_str())
                .collect::<Vec<_>>(),
            ["first occurrence", "second occurrence"]
        );
        assert_eq!(retained.helps[0].message, "native suggestion");
        assert!(retained.span.is_none());
    }
}
