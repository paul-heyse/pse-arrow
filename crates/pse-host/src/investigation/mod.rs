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
    /// Retain native query-construction metadata through the existing local IO wrapper.
    pub fn query_error_record(&self) -> Option<Box<crate::envelope::ErrorRecord>> {
        let Self::Io(error) = self else {
            return None;
        };
        let original = error
            .get_ref()?
            .downcast_ref::<crate::source_query::QueryError>()?;
        match original {
            crate::source_query::QueryError::TreeSitter(error) => Some(Box::new(error.record())),
            _ => None,
        }
    }
    /// Classify the boundary without interpreting diagnostic code or display text.
    pub fn phase(&self) -> DiagnosticPhase {
        match self {
            Self::Invalid(_) | Self::Reference(_) => DiagnosticPhase::Constraint,
            Self::Json(_) | Self::Configuration(_) => DiagnosticPhase::Schema,
            Self::Io(_) | Self::Stale(_) | Self::Unavailable(_) => DiagnosticPhase::Context,
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
