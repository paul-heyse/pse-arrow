// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Loss-aware owned projection of the native diagnostic protocols.
use miette::{Diagnostic, SourceCode};
use pse_diagnostics::{DiagnosticFacts, DiagnosticObservation, TypedDiagnostic};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::BTreeMap;

/// Exact secret-value replacement retains its configuration key provenance.
#[derive(Clone, Debug, Default)]
pub struct RedactionPolicy {
    values: Vec<(String, String)>,
}
impl RedactionPolicy {
    /// Capture secret selections without putting their values in the response.
    pub fn from_environment() -> Self {
        Self::from_inputs(&std::env::vars_os().collect())
    }
    /// Capture classified secrets from the actual payload mapping, independently of the
    /// supervisor's deliberately smaller control environment.
    pub fn from_inputs(environment: &crate::environment::Environment) -> Self {
        Self::new(
            environment
                .iter()
                .map(|(name, value)| {
                    (
                        name.to_string_lossy().into_owned(),
                        value.to_string_lossy().into_owned(),
                    )
                })
                .filter(|(name, value)| {
                    let name = name.to_ascii_uppercase();
                    !value.is_empty()
                        && [
                            "TOKEN",
                            "PASSWORD",
                            "SECRET",
                            "AUTHORIZATION",
                            "PRIVATE_KEY",
                            "LICENSE",
                        ]
                        .iter()
                        .any(|word| name.contains(word))
                }),
        )
    }
    /// Supply explicitly classified secrets, useful for non-environment sources.
    pub fn new(values: impl IntoIterator<Item = (String, String)>) -> Self {
        let mut values: Vec<_> = values
            .into_iter()
            .filter(|(_, value)| !value.is_empty())
            .collect();
        values.sort_by_key(|(_, value)| std::cmp::Reverse(value.len()));
        Self { values }
    }
    /// Replace exact selected values while preserving the secret's category/source.
    pub fn redact(&self, value: &str) -> String {
        self.values
            .iter()
            .fold(value.to_owned(), |text, (name, secret)| {
                text.replace(secret, &format!("<redacted:{name}>"))
            })
    }
}
/// A source label resolved through the source's native span reader.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
pub struct LabelRecord {
    /// Label description, when supplied.
    pub label: Option<String>,
    /// Whether the native diagnostic marks the primary label.
    pub primary: bool,
    /// Original UTF-8 source byte offset.
    pub byte_offset: usize,
    /// Original UTF-8 source byte extent.
    pub byte_length: usize,
    /// Source-owned document name, including inherited related context.
    pub source: Option<String>,
    /// Zero-based source-reader line.
    pub line: Option<usize>,
    /// Zero-based source-reader column; its unit is retained explicitly.
    pub column: Option<usize>,
    /// miette's source-reader coordinate unit.
    pub column_encoding: String,
    /// Original inability to resolve the requested span.
    pub unavailable: Option<String>,
}
/// Source-owned authored location, without a product/model dependency.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
pub struct LocationRecord {
    /// Semantic source occurrence identity.
    pub source: [u8; 16],
    /// Immutable revision when known.
    pub revision: Option<[u8; 32]>,
    /// Actual source document identity when supplied.
    pub document: Option<[u8; 16]>,
    /// Source role when supplied.
    pub occurrence_role: Option<String>,
    /// Original role-local position.
    pub occurrence_position: Option<u32>,
    /// Original authored path.
    pub path: String,
    /// Original display name.
    pub name: Option<String>,
    /// Inclusive UTF-8 byte offset.
    pub start: Option<u32>,
    /// Exclusive UTF-8 byte offset.
    pub end: Option<u32>,
}
/// Numbers never silently turn nonfinite observations into JSON null.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ObservationValue {
    /// Absent evidence.
    Missing,
    /// Finite numerical value.
    Number(f64),
    /// Explicit IEEE nonfinite classification.
    NonFinite(pse_diagnostics::NonFinite),
    /// Exact count.
    Integer(i64),
    /// Checked predicate.
    Boolean(bool),
    /// Native text, redacted according to its source policy.
    Text(String),
    /// Complete registered operand contracts.
    Contracts(Vec<pse_diagnostics::OperandContract>),
}
/// Structured native attribution, separate from rendered text.
#[derive(Clone, Debug, Default, Serialize, serde::Deserialize, JsonSchema)]
pub struct FactsRecord {
    /// Original semantic sources, preserving order and occurrences.
    pub sources: Vec<[u8; 16]>,
    /// Original authored locations.
    pub locations: Vec<LocationRecord>,
    /// Registered rule spelling when supplied.
    pub rule: Option<String>,
    /// Named native observations.
    pub observations: BTreeMap<String, ObservationValue>,
}
impl FactsRecord {
    fn capture(facts: DiagnosticFacts, policy: &RedactionPolicy) -> Self {
        Self {
            sources: facts.sources,
            locations: facts
                .locations
                .into_iter()
                .map(|value| LocationRecord {
                    source: value.source,
                    revision: value.revision,
                    document: value.document,
                    occurrence_role: value.occurrence_role.map(|s| policy.redact(&s)),
                    occurrence_position: value.occurrence_position,
                    path: policy.redact(&value.path),
                    name: value.name.map(|s| policy.redact(&s)),
                    start: value.start,
                    end: value.end,
                })
                .collect(),
            rule: facts.rule.map(|value| value.as_str().to_owned()),
            observations: facts
                .observations
                .into_iter()
                .map(|(key, value)| {
                    (
                        key,
                        match value {
                            DiagnosticObservation::Missing => ObservationValue::Missing,
                            DiagnosticObservation::Number(value) if value.is_finite() => {
                                ObservationValue::Number(value)
                            }
                            DiagnosticObservation::Number(value) => {
                                ObservationValue::NonFinite(if value.is_nan() {
                                    pse_diagnostics::NonFinite::Nan
                                } else if value.is_sign_negative() {
                                    pse_diagnostics::NonFinite::NegativeInfinity
                                } else {
                                    pse_diagnostics::NonFinite::PositiveInfinity
                                })
                            }
                            DiagnosticObservation::Integer(value) => {
                                ObservationValue::Integer(value)
                            }
                            DiagnosticObservation::Boolean(value) => {
                                ObservationValue::Boolean(value)
                            }
                            DiagnosticObservation::Text(value) => {
                                ObservationValue::Text(policy.redact(&value))
                            }
                            DiagnosticObservation::Contracts(value) => {
                                ObservationValue::Contracts(value)
                            }
                        },
                    )
                })
                .collect(),
        }
    }
}
/// Owner-supplied context and safe recovery advice for a classified refusal.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
pub struct RecoveryRecord {
    /// The owner-classified operation that failed.
    pub operation: String,
    /// Safe resolved operand, when known.
    pub operand: Option<crate::investigation::NativePath>,
    /// Effective input's contributing provider, when known.
    pub origin: Option<crate::investigation::config::InputOrigin>,
    /// Cause independent of rendered text and raw operating-system numbers.
    pub cause: RecoveryCause,
    /// Advice only; never automatically executed.
    pub next: Option<super::NextAction>,
}
impl RecoveryRecord {
    /// Apply the original disclosure before downstream consumers can use a correction.
    pub fn redacted(mut self, policy: &RedactionPolicy) -> Self {
        let path = |value: crate::investigation::NativePath| {
            let text = value.to_path_buf().to_string_lossy().into_owned();
            let redacted = policy.redact(&text);
            if text == redacted {
                value
            } else {
                crate::investigation::NativePath::Utf8(redacted)
            }
        };
        self.operation = policy.redact(&self.operation);
        self.operand = self.operand.map(path);
        self.origin = self.origin.map(|mut origin| {
            origin.provider = policy.redact(&origin.provider);
            origin.source = origin.source.map(path);
            origin
        });
        self.next = self.next.and_then(|action| match action {
            super::NextAction::Command { argv, reason }
                if argv.iter().all(|arg| policy.redact(arg) == *arg) =>
            {
                Some(super::NextAction::Command {
                    argv,
                    reason: policy.redact(&reason),
                })
            }
            super::NextAction::ReadArtifact { location, reason }
                if policy.redact(&location) == location =>
            {
                Some(super::NextAction::ReadArtifact {
                    location,
                    reason: policy.redact(&reason),
                })
            }
            _ => None,
        });
        self
    }
}
/// Source and prepared-context causes retain their different premises.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, serde::Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryCause {
    /// An explicitly selected working root was absent before execution.
    MissingRoot,
    /// An observed source disappeared before acquisition finished.
    SourceDisappeared,
    /// Access was denied by the filesystem.
    PermissionDenied,
    /// The observed source changed during acquisition.
    SourceChanged,
    /// Another native filesystem failure, without guessing its domain meaning.
    InputOutput,
    /// Prepared-world owner supplied its specific reason.
    Native(crate::semantic::FailureReason),
}

/// Owned versioned projection. Independent relationships preserve every occurrence.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
pub struct ErrorRecord {
    /// Diagnostic record version.
    pub version: u32,
    /// Native message after source-aware redaction.
    pub message: String,
    /// Registered semantic code or external diagnostic's original code.
    pub code: Option<String>,
    /// Native registered failure class, when known.
    pub class: Option<String>,
    /// Severity from the native protocol.
    pub severity: Option<String>,
    /// Native actionable help.
    pub help: Option<String>,
    /// Native URL, serialized with standard JSON escaping.
    pub url: Option<String>,
    /// Native structured attribution.
    pub facts: FactsRecord,
    /// Ordered typed diagnostic children, without deduplication.
    pub children: Vec<ErrorRecord>,
    /// Ordinary std::error causes, independently of diagnostic delegation.
    pub causes: Vec<String>,
    /// miette diagnostic_source delegation, independently of ordinary sources.
    pub diagnostic_source: Option<Box<ErrorRecord>>,
    /// Related diagnostics, with inherited source context.
    pub related: Vec<ErrorRecord>,
    /// Resolved original labels.
    pub labels: Vec<LabelRecord>,
    /// Parser's structured ErrorKind/context rather than only its rendered string.
    pub parser_context: BTreeMap<String, String>,
    /// Optional recovery facts from the owner that knows the failed operation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery: Option<RecoveryRecord>,
}
impl ErrorRecord {
    /// Capture a typed native diagnostic without flattening its aggregate structure.
    pub fn from_typed(error: &dyn TypedDiagnostic, policy: &RedactionPolicy) -> Self {
        let mut record = Self::from_diagnostic(error, None, policy);
        record.code = error
            .diagnostic_code()
            .map(|code| code.as_str().to_owned())
            .or(record.code);
        record.class = error.failure_class().map(|class| class.as_str().to_owned());
        record.facts = FactsRecord::capture(error.diagnostic_facts(), policy);
        record.children = error
            .diagnostic_children()
            .map(|children| {
                children
                    .map(|child| Self::from_typed(child, policy))
                    .collect()
            })
            .unwrap_or_default();
        record
    }
    /// Recover a source-owned presentation cause without interpreting its display string.
    pub fn from_host(error: &crate::HostError, policy: &RedactionPolicy) -> Self {
        if let crate::HostError::Investigation(error) = error {
            return error.record(policy);
        }
        if let crate::HostError::Io(error) = error
            && let Some(original) = super::PresentationError::from_io(error)
        {
            let mut record = Self::from_typed(original, policy);
            record
                .parser_context
                .insert("valid_fields".into(), original.choices.join(","));
            record.parser_context.insert(
                "field_grammar".into(),
                "comma-separated literal top-level payload names".into(),
            );
            return record;
        }
        Self::from_typed(error, policy)
    }
    /// Capture miette's protocol, inheriting the parent's source for related labels.
    pub fn from_diagnostic(
        error: &dyn Diagnostic,
        inherited: Option<&dyn SourceCode>,
        policy: &RedactionPolicy,
    ) -> Self {
        let source = error.source_code().or(inherited);
        let labels = error
            .labels()
            .map(|labels| {
                labels
                    .map(|label| {
                        let mut record = LabelRecord {
                            label: label.label().map(|text| policy.redact(text)),
                            primary: label.primary(),
                            byte_offset: label.offset(),
                            byte_length: label.len(),
                            source: None,
                            line: None,
                            column: None,
                            column_encoding: "miette_source_column".into(),
                            unavailable: None,
                        };
                        match source.map(|source| source.read_span(label.inner(), 0, 0)) {
                            Some(Ok(span)) => {
                                record.source = span.name().map(|name| policy.redact(name));
                                record.line = Some(span.line());
                                record.column = Some(span.column());
                            }
                            Some(Err(error)) => {
                                record.unavailable = Some(policy.redact(&error.to_string()))
                            }
                            None => record.unavailable = Some("source context unavailable".into()),
                        }
                        record
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut causes = Vec::new();
        let mut cause = std::error::Error::source(error);
        while let Some(value) = cause {
            causes.push(policy.redact(&value.to_string()));
            cause = value.source();
        }
        Self {
            version: 1,
            message: policy.redact(&error.to_string()),
            code: error.code().map(|value| policy.redact(&value.to_string())),
            class: None,
            severity: error
                .severity()
                .map(|value| format!("{value:?}").to_ascii_lowercase()),
            help: error.help().map(|value| policy.redact(&value.to_string())),
            url: error.url().map(|value| policy.redact(&value.to_string())),
            facts: FactsRecord::default(),
            children: Vec::new(),
            causes,
            diagnostic_source: error
                .diagnostic_source()
                .map(|child| Box::new(Self::from_diagnostic(child, source, policy))),
            related: error
                .related()
                .map(|related| {
                    related
                        .map(|child| Self::from_diagnostic(child, source, policy))
                        .collect()
                })
                .unwrap_or_default(),
            labels,
            parser_context: BTreeMap::new(),
            recovery: None,
        }
    }
    /// Convert clap's native context into the common structured diagnostic record.
    pub fn from_clap(error: &clap::Error, policy: &RedactionPolicy) -> Self {
        Self {
            version: 1,
            message: policy.redact(&error.to_string()),
            code: Some("cli.usage".into()),
            class: Some("authoring.parse".into()),
            severity: Some("error".into()),
            help: Some("use --help for this command's accepted arguments".into()),
            url: None,
            facts: FactsRecord::default(),
            children: Vec::new(),
            causes: Vec::new(),
            diagnostic_source: None,
            related: Vec::new(),
            labels: Vec::new(),
            parser_context: std::iter::once(("error_kind".into(), format!("{:?}", error.kind())))
                .chain(
                    error.context().map(|(key, value)| {
                        (format!("{key:?}"), policy.redact(&value.to_string()))
                    }),
                )
                .collect(),
            recovery: None,
        }
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    #[test]
    fn recovery_redaction_withholds_unsafe_actions_and_preserves_safe_native_paths() {
        use crate::investigation::{NativePath, config::InputOrigin};
        let value = RecoveryRecord {
            operation: "select_root".into(),
            operand: Some(NativePath::Utf8("/tmp/secret/root".into())),
            origin: Some(InputOrigin {
                provider: "request file".into(),
                source: Some(NativePath::Utf8("/tmp/secret/request.json".into())),
            }),
            cause: RecoveryCause::MissingRoot,
            next: Some(super::super::NextAction::Command {
                argv: vec![
                    "pse".into(),
                    "search".into(),
                    "--path".into(),
                    "secret".into(),
                    "--explain".into(),
                ],
                reason: "read".into(),
            }),
        }
        .redacted(&RedactionPolicy::new([("TOKEN".into(), "secret".into())]));
        assert!(value.next.is_none());
        let json = serde_json::to_string(&value).unwrap();
        assert!(!json.contains("secret"));
        assert!(json.contains("redacted:TOKEN"));
        let raw = NativePath::Raw {
            unix: b"/tmp/\xff".to_vec(),
        };
        let value = RecoveryRecord {
            operation: "read_source".into(),
            operand: Some(raw.clone()),
            origin: None,
            cause: RecoveryCause::InputOutput,
            next: None,
        }
        .redacted(&RedactionPolicy::default());
        assert_eq!(value.operand.unwrap(), raw);
    }
}
