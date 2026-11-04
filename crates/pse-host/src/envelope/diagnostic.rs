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
    /// Preserve exact native bytes unless disclosure requires a redacted display path.
    pub(crate) fn redact_path(
        &self,
        value: crate::investigation::NativePath,
    ) -> crate::investigation::NativePath {
        let text = value.to_path_buf().to_string_lossy().into_owned();
        let redacted = self.redact(&text);
        if text == redacted {
            value
        } else {
            crate::investigation::NativePath::Utf8(redacted)
        }
    }
}
pse_contract::projection_struct! {
/// A source label resolved through the source's native span reader.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
pub struct LabelRecord {
    projection: { name: "DiagnosticLabel", rust_path: crate::envelope::LabelRecord, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Label description, when supplied.
    pub label: Option<String> => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Whether the native diagnostic marks the primary label.
    pub primary: bool => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original UTF-8 source byte offset.
    pub byte_offset: usize => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original UTF-8 source byte extent.
    pub byte_length: usize => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Source-owned document name, including inherited related context.
    pub source: Option<String> => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Zero-based source-reader line.
    pub line: Option<usize> => { tag: 6, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Zero-based source-reader column; its unit is retained explicitly.
    pub column: Option<usize> => { tag: 7, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// miette's source-reader coordinate unit.
    pub column_encoding: String => { tag: 8, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original inability to resolve the requested span.
    pub unavailable: Option<String> => { tag: 9, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
pse_contract::projection_struct! {
/// Source-owned authored location, without a product/model dependency.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
pub struct LocationRecord {
    projection: { name: "DiagnosticLocation", rust_path: crate::envelope::LocationRecord, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Semantic source occurrence identity.
    pub source: [u8; 16] => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Immutable revision when known.
    pub revision: Option<[u8; 32]> => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Actual source document identity when supplied.
    pub document: Option<[u8; 16]> => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Source role when supplied.
    pub occurrence_role: Option<String> => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original role-local position.
    pub occurrence_position: Option<u32> => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original authored path.
    pub path: String => { tag: 6, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original display name.
    pub name: Option<String> => { tag: 7, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Inclusive UTF-8 byte offset.
    pub start: Option<u32> => { tag: 8, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Exclusive UTF-8 byte offset.
    pub end: Option<u32> => { tag: 9, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
pse_contract::projection_enum! {
/// Numbers never silently turn nonfinite observations into JSON null.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ObservationValue {
    projection: { name: "DiagnosticObservation", rust_path: crate::envelope::ObservationValue, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Absent evidence.
    Missing => 1;
    /// Finite numerical value.
    Number(f64) => 2;
    /// Explicit IEEE nonfinite classification.
    NonFinite(pse_diagnostics::NonFinite) => 3;
    /// Exact count.
    Integer(i64) => 4;
    /// Checked predicate.
    Boolean(bool) => 5;
    /// Native text, redacted according to its source policy.
    Text(String) => 6;
    /// Complete registered operand contracts.
    Contracts(Vec<pse_diagnostics::OperandContract>) => 7;
}
}
pse_contract::projection_struct! {
/// Structured native attribution, separate from rendered text.
#[derive(Clone, Debug, Default, Serialize, serde::Deserialize, JsonSchema)]
pub struct FactsRecord {
    projection: { name: "DiagnosticFacts", rust_path: crate::envelope::FactsRecord, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Original semantic sources, preserving order and occurrences.
    pub sources: Vec<[u8; 16]> => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original authored locations.
    pub locations: Vec<LocationRecord> => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Registered rule spelling when supplied.
    pub rule: Option<String> => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Named native observations.
    pub observations: BTreeMap<String, ObservationValue> => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
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
pse_contract::projection_struct! {
/// Owner-supplied context and safe recovery advice for a classified refusal.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
pub struct RecoveryRecord {
    projection: { name: "DiagnosticRecovery", rust_path: crate::envelope::RecoveryRecord, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// The owner-classified operation that failed.
    pub operation: String => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Safe resolved operand, when known.
    pub operand: Option<crate::investigation::NativePath> => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Effective input's contributing provider, when known.
    pub origin: Option<crate::investigation::config::InputOrigin> => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Cause independent of rendered text and raw operating-system numbers.
    pub cause: RecoveryCause => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Advice only; never automatically executed.
    pub next: Option<super::NextAction> => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    @local {
        /// Internal selector context; decoding public observations never restores it.
        #[serde(skip)]
        #[schemars(skip)]
        pub selection: Option<Box<crate::investigation::SelectionContext>> = None => check:
            |_: &Option<Box<crate::investigation::SelectionContext>>, _: &str| Ok::<(), pse_contract::boundary_projection::ValidationError>(());
    }
}
}
impl RecoveryRecord {
    /// Apply the original disclosure before downstream consumers can use a correction.
    pub fn redacted(mut self, policy: &RedactionPolicy) -> Self {
        self.operation = policy.redact(&self.operation);
        self.operand = self.operand.map(|path| policy.redact_path(path));
        self.origin = self.origin.map(|mut origin| {
            origin.provider = policy.redact(&origin.provider);
            origin.source = origin.source.map(|path| policy.redact_path(path));
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
pse_contract::projection_enum! {
/// Source and prepared-context causes retain their different premises.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, serde::Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryCause {
    projection: { name: "DiagnosticRecoveryCause", rust_path: crate::envelope::RecoveryCause, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// An explicitly selected working root was absent before execution.
    MissingRoot => 1;
    /// An observed source disappeared before acquisition finished.
    SourceDisappeared => 2;
    /// Access was denied by the filesystem.
    PermissionDenied => 3;
    /// The observed source changed during acquisition.
    SourceChanged => 4;
    /// Another native filesystem failure, without guessing its domain meaning.
    InputOutput => 5;
    /// Prepared-world owner supplied its specific reason.
    Native(crate::semantic::FailureReason) => 6;
}
}

pse_contract::projection_struct! {
/// Owned versioned projection. Independent relationships preserve every occurrence.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
pub struct ErrorRecord {
    projection: { name: "ErrorRecord", rust_path: crate::envelope::ErrorRecord, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Diagnostic record version.
    pub version: u32 => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Native message after source-aware redaction.
    pub message: String => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Registered semantic code or external diagnostic's original code.
    pub code: Option<String> => { tag: 3, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Native registered failure class, when known.
    pub class: Option<String> => { tag: 4, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Severity from the native protocol.
    pub severity: Option<String> => { tag: 5, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Native actionable help.
    pub help: Option<String> => { tag: 6, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Native URL, serialized with standard JSON escaping.
    pub url: Option<String> => { tag: 7, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Native structured attribution.
    pub facts: FactsRecord => { tag: 8, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Ordered typed diagnostic children, without deduplication.
    pub children: Vec<ErrorRecord> => { tag: 9, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Ordinary std::error causes, independently of diagnostic delegation.
    pub causes: Vec<String> => { tag: 10, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// miette diagnostic_source delegation, independently of ordinary sources.
    pub diagnostic_source: Option<Box<ErrorRecord>> => { tag: 11, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Related diagnostics, with inherited source context.
    pub related: Vec<ErrorRecord> => { tag: 12, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Resolved original labels.
    pub labels: Vec<LabelRecord> => { tag: 13, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Parser's structured ErrorKind/context rather than only its rendered string.
    pub parser_context: BTreeMap<String, String> => { tag: 14, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Optional recovery facts from the owner that knows the failed operation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery: Option<Box<RecoveryRecord>> => { tag: 15, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Source-owned semantic RPC observation, independent of native verdict authority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_rpc: Option<Box<crate::semantic::RpcObservation>> => { tag: 16, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
impl ErrorRecord {
    fn semantic_rpc<'error>(
        error: &'error (dyn std::error::Error + 'static),
    ) -> Option<&'error crate::semantic::RpcFailure> {
        let mut current: &(dyn std::error::Error + 'static) = error;
        loop {
            if let Some(crate::semantic::Error::Rpc(rpc)) =
                current.downcast_ref::<crate::semantic::Error>()
            {
                return Some(rpc);
            }
            if let Some(error) = current.downcast_ref::<crate::HostError>() {
                match error {
                    crate::HostError::Semantic(error) => {
                        current = error.as_ref();
                        continue;
                    }
                    crate::HostError::Investigation(error) => {
                        current = error;
                        continue;
                    }
                    _ => {}
                }
            }
            if let Some(crate::investigation::Error::Semantic(error)) =
                current.downcast_ref::<crate::investigation::Error>()
            {
                current = error.as_ref();
                continue;
            }
            let Some(source) = current.source() else {
                return None;
            };
            current = source;
        }
    }
    fn contains_semantic_rpc(error: &dyn TypedDiagnostic) -> bool {
        Self::semantic_rpc(error).is_some()
            || error
                .diagnostic_children()
                .is_some_and(|mut children| children.any(Self::contains_semantic_rpc))
    }
    /// Capture a typed native diagnostic without flattening its aggregate structure.
    pub fn from_typed(error: &dyn TypedDiagnostic, policy: &RedactionPolicy) -> Self {
        let rpc = Self::semantic_rpc(error);
        // miette erases typed child identities. A mixed graph omits flattened raw causes
        // throughout its derived diagnostic relationships; independently captured typed
        // children below preserve each sibling's facts, causes and individual RPC facet.
        let mut record =
            Self::capture_diagnostic(error, None, policy, !Self::contains_semantic_rpc(error));
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
        if let Some(rpc) = rpc {
            record.semantic_rpc = Some(Box::new(rpc.observation_with_policy(policy)));
            // Full original ConnectRPC/validator sources remain available locally.
            // Publicly rendering them would restore intentionally omitted input.
            if let Some(conversion) = rpc.conversion_error() {
                record.related.push(Self::from_typed(conversion, policy));
            }
        }
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
        Self::capture_diagnostic(error, inherited, policy, true)
    }
    fn capture_diagnostic(
        error: &dyn Diagnostic,
        inherited: Option<&dyn SourceCode>,
        policy: &RedactionPolicy,
        include_causes: bool,
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
        let mut cause = include_causes
            .then(|| std::error::Error::source(error))
            .flatten();
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
            diagnostic_source: error.diagnostic_source().map(|child| {
                Box::new(Self::capture_diagnostic(
                    child,
                    source,
                    policy,
                    include_causes,
                ))
            }),
            related: error
                .related()
                .map(|related| {
                    related
                        .map(|child| {
                            Self::capture_diagnostic(child, source, policy, include_causes)
                        })
                        .collect()
                })
                .unwrap_or_default(),
            labels,
            parser_context: BTreeMap::new(),
            recovery: None,
            semantic_rpc: None,
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
            semantic_rpc: None,
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
            selection: None,
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
            selection: None,
            cause: RecoveryCause::InputOutput,
            next: None,
        }
        .redacted(&RedactionPolicy::default());
        assert_eq!(value.operand.unwrap(), raw);
    }
}
