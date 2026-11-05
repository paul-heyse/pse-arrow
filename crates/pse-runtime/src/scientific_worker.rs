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
    /// All original scientific and independent retirement failures, in order.
    #[error("scientific worker retirement failed: {causes:?}")]
    Retirement {
        /// Original action followed by every independent failed teardown.
        causes: Vec<Self>,
    },
    /// Exact failed physical child operation.
    #[error("scientific worker process {action}: {source}")]
    Process {
        /// Exact operation attempted on the owned process or local endpoint.
        action: &'static str,
        /// Original operating-system error.
        #[source]
        source: std::io::Error,
    },
    /// The owned executable exited unsuccessfully before an intentional kill,
    /// or failed to exit successfully after an acknowledged Shutdown.
    #[error("scientific worker exit after shutdown: {0}")]
    Exit(std::process::ExitStatus),
    /// Lifecycle request failure, retaining its original typed cause.
    #[error("scientific worker {action} for {handle:?}: {cause}")]
    Lifecycle {
        /// Requested handle or session operation.
        action: &'static str,
        /// Exact prepared handle, absent only for process-wide Shutdown.
        handle: Option<protocol::Handle>,
        /// Original typed refusal, dispatch or physical settlement failure.
        #[source]
        cause: Box<Self>,
    },
    /// An acknowledgement did not identify the requested lifecycle operation.
    #[error("scientific worker {action} acknowledgement differs for {handle:?}")]
    Acknowledgement {
        /// Operation whose response did not acknowledge the requested identity.
        action: &'static str,
        /// Exact prepared handle, absent only for process-wide Shutdown.
        handle: Option<protocol::Handle>,
    },
    /// The explicitly owned Hyper connection completed with an error.
    #[cfg(feature = "property-coolprop")]
    #[error("scientific worker H2 connection: {0}")]
    Connection(#[source] hyper::Error),
    /// Generated RPC failure after dispatch, preserving Connect's cause chain.
    #[cfg(feature = "property-coolprop")]
    #[error("scientific worker completion is unknown: {0}")]
    Exchange(#[source] connectrpc::ConnectError),
    /// An explicitly owned async task failed to join.
    #[cfg(feature = "property-coolprop")]
    #[error("scientific worker I/O task: {0}")]
    IoTask(#[source] tokio::task::JoinError),
    /// I/O thread panicked after its runtime and owned tasks departed.
    #[error("scientific worker I/O thread panicked: {0}")]
    IoThread(String),
    /// A child-native provider or its lane panicked during settlement.
    #[error("scientific worker native retirement panicked: {0}")]
    NativePanic(String),
    /// An opaque remote source observation, without reconstructed native authority.
    #[error("scientific worker remote diagnostic observation: {0}")]
    RemoteDiagnostic(Box<pse_model::diagnostic::BoundaryDiagnostic>),
}

pse_diagnostics::impl_diagnostic! {
    WorkerError,
    code(this) { match this {
        Self::Provider(_) | Self::Lifecycle { .. } => None,
        Self::RemoteResource(_) => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
        Self::NativePanic(_) | Self::IoThread(_) => Some(pse_diagnostics::DiagnosticCode::WorkflowPanic),
        _ => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
    } },
    forward(this) { match this {
        Self::Provider(error) => Some(error), Self::Lifecycle { cause, .. } => Some(cause.as_ref()),
        _ => None,
    } },
    help(_this) { None }, related(this) { match this {
        Self::Retirement { causes } => Some(Box::new(causes.iter().map(|cause| { let cause: &dyn pse_diagnostics::TypedDiagnostic = cause; cause }))),
        _ => None,
    } }, source(_this) { None },
    facts(this) {
        let mut facts = pse_diagnostics::DiagnosticFacts::default();
        if matches!(this, Self::UnknownCompletion(_)) {
            facts.observe("scientific_worker_completion_unknown", pse_diagnostics::DiagnosticObservation::Boolean(true));
        }
        #[cfg(feature = "property-coolprop")]
        if matches!(this, Self::Exchange(_)) {
            facts.observe("scientific_worker_completion_unknown", pse_diagnostics::DiagnosticObservation::Boolean(true));
        }
        if let Self::Infrastructure(detail) | Self::UnknownCompletion(detail) | Self::RemoteResource(detail) = this {
            facts.observe("detail", pse_diagnostics::DiagnosticObservation::Text(detail.clone()));
        }
        match this {
            Self::Process { action, .. } | Self::Lifecycle { action, .. } | Self::Acknowledgement { action, .. } => facts.observe("action", pse_diagnostics::DiagnosticObservation::Text((*action).into())),
            _ => {},
        }
        if let Self::Lifecycle { handle: Some(handle), .. } | Self::Acknowledgement { handle: Some(handle), .. } = this {
            facts.observe("generation", pse_diagnostics::DiagnosticObservation::Text(handle.generation.to_string()));
        }
        if let Self::Process { source, .. } = this {
            facts.observe("detail", pse_diagnostics::DiagnosticObservation::Text(source.to_string()));
            facts.observe("io_error_kind", pse_diagnostics::DiagnosticObservation::Text(format!("{:?}", source.kind())));
            if let Some(code) = source.raw_os_error() {
                facts.observe("os_error", pse_diagnostics::DiagnosticObservation::Integer(i64::from(code)));
            }
        }
        if let Self::IoThread(detail) | Self::NativePanic(detail) = this {
            facts.observe("detail", pse_diagnostics::DiagnosticObservation::Text(detail.clone()));
        }
        #[cfg(feature = "property-coolprop")]
        {
            let detail = match this {
                Self::Connection(error) => Some(error.to_string()),
                Self::Exchange(error) => Some(error.to_string()),
                Self::IoTask(error) => Some(error.to_string()),
                _ => None,
            };
            if let Some(detail) = detail { facts.observe("detail", pse_diagnostics::DiagnosticObservation::Text(detail)); }
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
            Self::Retirement { causes } => {
                let mut value = pse_model::diagnostic::project_facts(
                    self.diagnostic_code(),
                    self.diagnostic_facts(),
                    stage,
                );
                value
                    .causes
                    .extend(causes.iter().map(|cause| cause.boundary_diagnostic(stage)));
                value
            }
            Self::RemoteDiagnostic(diagnostic) => {
                let mut value = pse_model::diagnostic::project_facts(
                    self.diagnostic_code(),
                    self.diagnostic_facts(),
                    stage,
                );
                value.causes.push((**diagnostic).clone());
                value
            }
            Self::Lifecycle { cause, .. } => {
                let mut value = pse_model::diagnostic::project_facts(
                    Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
                    self.diagnostic_facts(),
                    stage,
                );
                value.causes.push(cause.boundary_diagnostic(stage));
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
    fn retained_bytes(&self) -> Option<usize> {
        use pse_model::HeapUsage as _;
        size_of_val(self).checked_add(match self {
            Self::Provider(error) => Some(error.retained_bytes()),
            Self::Infrastructure(detail)
            | Self::UnknownCompletion(detail)
            | Self::RemoteResource(detail) => Some(detail.capacity()),
            Self::Retirement { causes } => causes.iter().try_fold(
                causes.capacity().checked_mul(size_of::<Self>())?,
                |bytes, cause| bytes.checked_add(cause.retained_bytes()?),
            ),
            Self::Lifecycle { cause, .. } => cause.retained_bytes(),
            Self::IoThread(detail) | Self::NativePanic(detail) => Some(detail.capacity()),
            Self::RemoteDiagnostic(diagnostic) => {
                size_of_val(diagnostic.as_ref()).checked_add(diagnostic.heap_bytes())
            }
            // These library-owned causes expose no complete retained-allocation contract.
            Self::Process { .. } | Self::Connection(_) | Self::Exchange(_) | Self::IoTask(_) => {
                None
            }
            Self::Acknowledgement { .. } | Self::Exit(_) => Some(0),
        }?)
    }
    #[cfg(feature = "property-coolprop")]
    fn into_provider(self) -> ProviderError {
        match self {
            Self::Provider(error) => error,
            error => ProviderError::Nested {
                // Existing finite retention refuses this conservative sentinel;
                // it is neither a measured extent nor a zero-allocation claim.
                retained: error.retained_bytes().unwrap_or(usize::MAX),
                cause: pse_model::diagnostic::DiagnosticCause::new(error),
            },
        }
    }
}

fn retirement_result(mut causes: Vec<WorkerError>) -> Result<(), WorkerError> {
    match causes.len() {
        0 => Ok(()),
        1 => Err(causes.remove(0)),
        _ => Err(WorkerError::Retirement { causes }),
    }
}

fn infrastructure(error: impl std::fmt::Display) -> WorkerError {
    WorkerError::Infrastructure(error.to_string())
}

#[cfg(all(test, feature = "property-coolprop"))]
mod measurements;
#[cfg(test)]
mod tests;
