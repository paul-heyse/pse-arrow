// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared, source-aware search interpretation and record delivery.
pub mod composition;
pub mod config;
pub mod graph;
pub(crate) mod lifetime;
mod model;
pub(crate) mod native_path_serde;
pub mod presentation;
pub mod providers;
pub mod rust;
pub mod stream;
pub use model::*;

/// The source owner knows whether failure preceded selection or followed observation.
#[derive(Clone, Copy, Debug)]
pub enum SourceOperation {
    /// Validate an explicitly selected root before source work begins.
    SelectRoot,
    /// Capture metadata for a selected source.
    ObserveSource,
    /// Acquire a selected source's original bytes.
    ReadSource,
    /// Traverse the selected filesystem universe.
    TraverseSources,
}
impl SourceOperation {
    fn as_str(self) -> &'static str {
        match self {
            Self::SelectRoot => "select_root",
            Self::ObserveSource => "observe_source",
            Self::ReadSource => "read_source",
            Self::TraverseSources => "traverse_sources",
        }
    }
}
/// Filesystem cause retained alongside its domain premise and available correction.
#[derive(Debug, thiserror::Error)]
#[error("{operation}: {source}", operation = .recovery.operation)]
pub struct SourceFailure {
    /// Publicly projectable owner facts, redacted at presentation.
    pub recovery: crate::envelope::RecoveryRecord,
    /// Original operating-system error remains in the cause chain.
    #[source]
    pub source: std::io::Error,
}
impl SourceFailure {
    /// Classify using the operation's observed premise, not display text.
    pub fn new(
        operation: SourceOperation,
        operand: std::path::PathBuf,
        source: std::io::Error,
        observed: bool,
    ) -> Self {
        use crate::envelope::RecoveryCause;
        let cause = match source.kind() {
            std::io::ErrorKind::PermissionDenied => RecoveryCause::PermissionDenied,
            std::io::ErrorKind::NotFound if observed => RecoveryCause::SourceDisappeared,
            std::io::ErrorKind::NotFound if matches!(operation, SourceOperation::SelectRoot) => {
                RecoveryCause::MissingRoot
            }
            _ => RecoveryCause::InputOutput,
        };
        Self {
            recovery: crate::envelope::RecoveryRecord {
                operation: operation.as_str().into(),
                operand: Some(operand.into()),
                origin: None,
                cause,
                next: None,
            },
            source,
        }
    }
}

/// A fallible search boundary preserves native causes and explicit unavailable facts.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The request contradicts the selected operation.
    #[error("invalid investigation request: {0}")]
    Invalid(String),
    /// A requested capability or preparation is absent.
    #[error("investigation capability unavailable: {0}")]
    Unavailable(String),
    /// An addressed observation no longer has its original premises.
    #[error("stale investigation reference: {0}")]
    Stale(String),
    /// Cooperative cancellation was observed.
    #[error("investigation cancelled")]
    Cancelled,
    /// Preserve original filesystem or writer failure.
    #[error("investigation input/output: {0}")]
    Io(#[from] std::io::Error),
    /// Source selection/acquisition preserves its operand and premise.
    #[error("investigation source: {0}")]
    SourcePath(#[source] Box<SourceFailure>),
    /// Preserve a writer failure separately from source acquisition.
    #[error("investigation delivery: {0}")]
    Delivery(#[source] std::io::Error),
    /// Preserve typed document decoding failure.
    #[error("investigation JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Preserve malformed reference encoding.
    #[error("investigation reference encoding: {0}")]
    Reference(#[from] base64::DecodeError),
    /// Preserve configuration source and key diagnostics.
    #[error("investigation configuration: {0}")]
    Configuration(Box<figment::Error>),
    /// Preserve native semantic preparation, protocol, source and cancellation causes.
    #[error("investigation semantics: {0}")]
    Semantic(Box<crate::semantic::Error>),
    /// Preserve original acquisition/source-equivalence failure from the API owner.
    #[error("investigation API: {0}")]
    Api(Box<crate::api::ApiError>),
}
impl From<figment::Error> for Error {
    fn from(error: figment::Error) -> Self {
        Self::Configuration(Box::new(error))
    }
}
impl From<crate::semantic::Error> for Error {
    fn from(error: crate::semantic::Error) -> Self {
        Self::Semantic(Box::new(error))
    }
}
impl From<crate::api::ApiError> for Error {
    fn from(error: crate::api::ApiError) -> Self {
        Self::Api(Box::new(error))
    }
}
/// Owner-classified native disposition, independent of prose or a positive candidate's presence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureDisposition {
    /// Missing preparation or unsupported/unproved capability.
    Unavailable,
    /// Native cancellation was observed.
    Cancelled,
    /// Original source/context premises changed.
    Stale,
    /// The native/protocol/input operation failed.
    Failed,
}
impl Error {
    /// Project owner facts consistently for command failures and in-query diagnostics.
    pub fn record(
        &self,
        policy: &crate::envelope::RedactionPolicy,
    ) -> crate::envelope::ErrorRecord {
        let mut record = crate::envelope::ErrorRecord::from_typed(self, policy);
        // Keep the parser's richer context under the original source disclosure policy.
        if let Self::Io(error) = self
            && let Some(crate::source_query::QueryError::TreeSitter(error)) = error
                .get_ref()
                .and_then(|error| error.downcast_ref::<crate::source_query::QueryError>())
        {
            return error.record();
        }
        let recovery = match self {
            Self::SourcePath(failure) => Some(failure.recovery.clone()),
            Self::Semantic(error) => match error.as_ref() {
                crate::semantic::Error::Refusal(failure) => failure.context.as_ref().map(|context| crate::envelope::RecoveryRecord {
                    operation: context.operation.clone(), operand: context.operand.clone(), origin: None,
                    cause: crate::envelope::RecoveryCause::Native(context.reason),
                    next: match (context.reason, context.operation.as_str(), context.operand.as_ref()) {
                        (crate::semantic::FailureReason::SessionUnavailable, "native_query", Some(root)) => root.to_path_buf().to_str().map(|root| crate::envelope::NextAction::Command {
                            argv: vec!["pse".into(), "-C".into(), root.into(), "code".into(), "session".into(), "prepare".into(), "--backend".into(), "native-snapshot".into()],
                            reason: "explicitly prepare the selected native project before repeating the query; preparation is an advised effect".into(),
                        }),
                        _ => None,
                    },
                }),
                _ => None,
            },
            _ => None,
        };
        record.recovery = recovery.map(|recovery| recovery.redacted(policy));
        record
    }
    /// Retain native query-construction and source recovery metadata in query events.
    pub fn query_error_record(&self) -> Option<Box<crate::envelope::ErrorRecord>> {
        Some(Box::new(self.record(
            &crate::envelope::RedactionPolicy::from_environment(),
        )))
    }
    /// Classify the boundary without interpreting diagnostic code or display text.
    pub fn phase(&self) -> DiagnosticPhase {
        match self {
            Self::Invalid(_) | Self::Reference(_) => DiagnosticPhase::Constraint,
            Self::Json(_) | Self::Configuration(_) => DiagnosticPhase::Schema,
            Self::Io(_) | Self::Stale(_) | Self::Unavailable(_) => DiagnosticPhase::Context,
            Self::SourcePath(error)
                if matches!(
                    error.recovery.cause,
                    crate::envelope::RecoveryCause::MissingRoot
                ) =>
            {
                DiagnosticPhase::Constraint
            }
            Self::SourcePath(_) => DiagnosticPhase::Context,
            Self::Delivery(_) => DiagnosticPhase::Delivery,
            Self::Semantic(error) => match error.as_ref() {
                crate::semantic::Error::Schema(_) => DiagnosticPhase::Schema,
                crate::semantic::Error::InvalidRequest(_) => DiagnosticPhase::Constraint,
                crate::semantic::Error::Validator(_) => DiagnosticPhase::Validator,
                crate::semantic::Error::Transport(_) => DiagnosticPhase::Delivery,
                crate::semantic::Error::Stale | crate::semantic::Error::Unavailable(_) => {
                    DiagnosticPhase::Context
                }
                crate::semantic::Error::CancelledObserved { .. }
                | crate::semantic::Error::CancellationUnobserved { .. } => {
                    DiagnosticPhase::Settlement
                }
                crate::semantic::Error::Refusal(failure) => match failure.phase {
                    crate::semantic::FailurePhase::Schema => DiagnosticPhase::Schema,
                    crate::semantic::FailurePhase::Constraint => DiagnosticPhase::Constraint,
                    crate::semantic::FailurePhase::Context => DiagnosticPhase::Context,
                    crate::semantic::FailurePhase::Validator => DiagnosticPhase::Validator,
                    crate::semantic::FailurePhase::Provider => DiagnosticPhase::Provider,
                    crate::semantic::FailurePhase::Transport => DiagnosticPhase::Delivery,
                    crate::semantic::FailurePhase::Settlement => DiagnosticPhase::Settlement,
                },
                _ => DiagnosticPhase::Provider,
            },
            Self::Api(_) | Self::Cancelled => DiagnosticPhase::Provider,
        }
    }
    /// Preserve a genuine remote refusal without manufacturing one from a local error.
    pub fn native_refusal(&self) -> Option<crate::semantic::Failure> {
        match self {
            Self::Semantic(error) => match error.as_ref() {
                crate::semantic::Error::Refusal(failure) => Some(failure.as_ref().clone()),
                crate::semantic::Error::CancelledObserved { .. } => {
                    Some(crate::semantic::Failure::from_error(error))
                }
                _ => None,
            },
            _ => None,
        }
    }
    /// Interpret local and serialized native causes through one authority.
    pub fn disposition(&self) -> FailureDisposition {
        match self {
            Self::Unavailable(_) => FailureDisposition::Unavailable,
            Self::Cancelled => FailureDisposition::Cancelled,
            Self::Stale(_) => FailureDisposition::Stale,
            Self::SourcePath(error)
                if matches!(
                    error.recovery.cause,
                    crate::envelope::RecoveryCause::SourceDisappeared
                        | crate::envelope::RecoveryCause::SourceChanged
                ) =>
            {
                FailureDisposition::Stale
            }
            Self::Api(error) if matches!(error.as_ref(), crate::api::ApiError::Invalid(_)) => {
                FailureDisposition::Unavailable
            }
            Self::Semantic(error) => match error.as_ref() {
                crate::semantic::Error::Unavailable(_) => FailureDisposition::Unavailable,
                crate::semantic::Error::Cancelled
                | crate::semantic::Error::CancelledObserved { .. } => FailureDisposition::Cancelled,
                crate::semantic::Error::Stale => FailureDisposition::Stale,
                crate::semantic::Error::Refusal(failure) => match failure.state {
                    crate::semantic::FailureState::Unavailable
                    | crate::semantic::FailureState::Unsupported => FailureDisposition::Unavailable,
                    crate::semantic::FailureState::Cancelled => FailureDisposition::Cancelled,
                    crate::semantic::FailureState::Stale => FailureDisposition::Stale,
                    crate::semantic::FailureState::Failed => FailureDisposition::Failed,
                },
                _ => FailureDisposition::Failed,
            },
            _ => FailureDisposition::Failed,
        }
    }
}
pse_diagnostics::impl_diagnostic! {Error,
    code(this){Some(match this {Self::Semantic(error)=>pse_diagnostics::TypedDiagnostic::diagnostic_code(error.as_ref()).unwrap_or(pse_diagnostics::DiagnosticCode::HostOwnership),Self::Api(error)=>pse_diagnostics::TypedDiagnostic::diagnostic_code(error.as_ref()).unwrap_or(pse_diagnostics::DiagnosticCode::HostOwnership),_=>pse_diagnostics::DiagnosticCode::HostOwnership})},
    forward(this){match this {Self::Semantic(error)=>Some(error.as_ref()),Self::Api(error)=>Some(error.as_ref()),_=>None}},help(_this){None},related(_this){None},source(_this){None}
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    #[test]
    fn native_path_operands_are_redacted_before_public_projection() {
        use std::os::unix::ffi::OsStringExt as _;
        let path = std::path::PathBuf::from(std::ffi::OsString::from_vec(
            b"/selected/secret/\xff".to_vec(),
        ));
        let error = Error::SourcePath(Box::new(SourceFailure::new(
            SourceOperation::SelectRoot,
            path,
            std::io::Error::from(std::io::ErrorKind::NotFound),
            false,
        )));
        let record = error.record(&crate::envelope::RedactionPolicy::new([(
            "TOKEN".into(),
            "secret".into(),
        )]));
        assert!(
            !record.message.contains("[47,"),
            "debug native bytes bypass string redaction"
        );
        let encoded = serde_json::to_string(&record).unwrap();
        assert!(!encoded.contains("secret"));
        assert!(encoded.contains("redacted:TOKEN"));
    }
    #[test]
    fn native_context_recovery_preserves_reason_and_preparation_scope() {
        for reason in [
            crate::semantic::FailureReason::SessionUnavailable,
            crate::semantic::FailureReason::OutsidePreparedWorld,
            crate::semantic::FailureReason::InactiveSource,
            crate::semantic::FailureReason::OriginalMappingUnavailable,
        ] {
            let mut failure = crate::semantic::Failure::from_error(
                &crate::semantic::Error::Unavailable("native context".into()),
            );
            failure.context = Some(crate::semantic::FailureContext {
                reason,
                operation: "native_query".into(),
                operand: Some(std::path::PathBuf::from("/selected/project").into()),
            });
            let error: Error = crate::semantic::Error::Refusal(Box::new(failure)).into();
            let record = error.record(&crate::envelope::RedactionPolicy::default());
            let recovery = record.recovery.unwrap();
            assert_eq!(
                recovery.cause,
                crate::envelope::RecoveryCause::Native(reason)
            );
            assert!(recovery.origin.is_none());
            if reason == crate::semantic::FailureReason::SessionUnavailable {
                let Some(crate::envelope::NextAction::Command { argv, .. }) = recovery.next else {
                    panic!("missing explicit preparation advice")
                };
                assert_eq!(
                    argv,
                    [
                        "pse",
                        "-C",
                        "/selected/project",
                        "code",
                        "session",
                        "prepare",
                        "--backend",
                        "native-snapshot"
                    ]
                );
            } else {
                assert!(recovery.next.is_none());
            }
        }
    }
    #[test]
    fn command_and_query_records_share_selection_cause_and_origin() {
        let mut failure = SourceFailure::new(
            SourceOperation::SelectRoot,
            "/selected/missing".into(),
            std::io::Error::from(std::io::ErrorKind::NotFound),
            false,
        );
        failure.recovery.origin = Some(config::InputOrigin {
            provider: "request file".into(),
            source: Some(std::path::PathBuf::from("/input/request.json").into()),
        });
        let error = Error::SourcePath(Box::new(failure));
        assert_eq!(error.phase(), DiagnosticPhase::Constraint);
        let query = error.query_error_record().unwrap();
        let host: crate::HostError = error.into();
        let command = crate::envelope::ErrorRecord::from_host(
            &host,
            &crate::envelope::RedactionPolicy::default(),
        );
        assert_eq!(
            serde_json::to_value(query.recovery).unwrap(),
            serde_json::to_value(command.recovery).unwrap()
        );
    }
}
