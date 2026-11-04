// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicitly prepared local placement of an existing scientific provider.
//!
//! Physical admission and numerical meaning remain with the provider owner. A
//! session never restores a failed attempt or replays an ambiguous invocation.

#[cfg(feature = "property-coolprop")]
pub mod generated;
#[cfg(feature = "property-coolprop")]
mod handler;
mod process;
pub mod protocol;
#[cfg(feature = "property-coolprop")]
mod proxy;
mod session;
#[cfg(feature = "property-coolprop")]
mod transport;
#[cfg(feature = "property-coolprop")]
mod wire;
#[cfg(feature = "property-coolprop")]
pub use proxy::LocalWaterFactory;
#[cfg(feature = "property-coolprop")]
pub use transport::serve;

pub use process::{PreparedExecutable, ProcessCustody};
pub use session::{PreparedSession, SessionHandle, SessionState};

use pse_kernels::ProviderError;

/// Attributed placement failure; ambiguous dispatch is distinct from a refusal.
#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    /// Original scientific owner refusal.
    #[error(transparent)]
    Provider(#[from] ProviderError),
    /// An authenticated local process or transport failed before dispatch.
    #[error("scientific worker infrastructure: {0}")]
    Infrastructure(String),
    /// Dispatch occurred and no current authenticated completion was obtained.
    #[error("scientific worker completion is unknown: {0}")]
    UnknownCompletion(String),
    /// Original remote allocation refusal, including its owner-provided detail.
    #[error("scientific worker resource refusal: {0}")]
    RemoteResource(String),
    /// Original stop cause plus an independently retained cleanup failure.
    #[error("{cause}; scientific worker cleanup failed: {cleanup}")]
    Cleanup {
        /// Original failure is retained without text-based reclassification.
        #[source]
        cause: Box<Self>,
        /// Actual kill/wait or I/O-owner settlement failure.
        cleanup: String,
    },
}

pse_diagnostics::impl_diagnostic! {
    WorkerError,
    code(this) { match this {
        Self::Provider(_) | Self::Cleanup { .. } => None,
        Self::Infrastructure(_) | Self::UnknownCompletion(_) => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
        Self::RemoteResource(_) => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
    } },
    forward(this) { match this {
        Self::Provider(error) => Some(error), Self::Cleanup { cause, .. } => Some(cause.as_ref()), _ => None,
    } },
    help(_this) { None }, related(_this) { None }, source(_this) { None },
    facts(this) {
        let mut facts = pse_diagnostics::DiagnosticFacts::default();
        if matches!(this, Self::UnknownCompletion(_)) {
            facts.observe("scientific_worker_completion_unknown", pse_diagnostics::DiagnosticObservation::Boolean(true));
        }
        if let Self::Infrastructure(detail) | Self::UnknownCompletion(detail) | Self::RemoteResource(detail) = this {
            facts.observe("detail", pse_diagnostics::DiagnosticObservation::Text(detail.clone()));
        }
        facts
    }
}
impl pse_model::diagnostic::DiagnosticProjection for WorkerError {
    fn boundary_diagnostic(
        &self,
        stage: pse_diagnostics::DiagnosticStage,
    ) -> pse_model::diagnostic::BoundaryDiagnostic {
        use pse_diagnostics::TypedDiagnostic as _;
        match self {
            Self::Provider(error) => error.boundary_diagnostic(stage),
            Self::Cleanup { cause, cleanup } => {
                let mut value = cause.boundary_diagnostic(stage);
                value
                    .causes
                    .push(Self::Infrastructure(cleanup.clone()).boundary_diagnostic(stage));
                value
            }
            _ => pse_model::diagnostic::project_facts(
                self.diagnostic_code(),
                self.diagnostic_facts(),
                stage,
            ),
        }
    }
}

impl WorkerError {
    #[cfg(feature = "property-coolprop")]
    fn retained_bytes(&self) -> usize {
        size_of_val(self).saturating_add(match self {
            Self::Provider(error) => error.retained_bytes(),
            Self::Infrastructure(detail)
            | Self::UnknownCompletion(detail)
            | Self::RemoteResource(detail) => detail.capacity(),
            Self::Cleanup { cause, cleanup } => {
                cause.retained_bytes().saturating_add(cleanup.capacity())
            }
        })
    }
    #[cfg(feature = "property-coolprop")]
    fn into_provider(self) -> ProviderError {
        match self {
            Self::Provider(error) => error,
            error => ProviderError::Nested {
                retained: error.retained_bytes(),
                cause: pse_model::diagnostic::DiagnosticCause::new(error),
            },
        }
    }
}

fn infrastructure(error: impl std::fmt::Display) -> WorkerError {
    WorkerError::Infrastructure(error.to_string())
}

#[cfg(all(test, feature = "property-coolprop"))]
mod measurements;
#[cfg(test)]
mod tests;
