// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent native host capabilities. A manager reply is not physical release.
//!
//! [`HostManager::current_membership`] authenticates the manager root and actual process
//! membership without granting mutation. Original [`OwnedUnit`] handles retain native stop
//! and drain responsibility. Foreground calls inherit authenticated enclosing containment;
//! detached wrappers and their payloads share one native service. Child/capture evidence and
//! later native drain/capacity reconciliation remain distinct owners.

pub mod activity;
pub mod admission;
pub mod affected;
pub mod agent_hooks;
pub mod artifacts;
pub mod boundary;
pub mod catalog;
pub mod cli;
pub mod correctness;
pub mod dependency;
pub mod diagnostic_tools;
pub mod discovery;
pub mod doctor;
pub mod envelope;
pub mod environment;
pub mod extraction;
pub mod fixtures;
pub mod foreground;
pub mod generation;
pub mod invocation;
pub mod invocation_view;
pub mod managed;
mod manager;
pub mod native;
pub mod ownership;
pub mod paths;
pub mod prepared_xtask;
pub mod product_reader;
pub mod repository;
pub mod selection;
pub mod server;
pub mod source_command;
pub mod source_proposal;
pub mod source_read;
pub mod status;
pub mod stream_records;
pub mod study_module;
pub mod test_inventory;
pub mod test_workflow;
pub use manager::{
    BorrowedService, BorrowedUnit, CurrentMembership, HostManager, LaunchIntent, OrdinaryPlacement,
    OwnedUnit, ResourceControls, ServiceFinalizer, ServiceSpec, UnitObservation,
};

/// A host boundary failure, never a scientific outcome.
#[derive(Debug, thiserror::Error)]
pub enum HostError {
    /// Native I/O failed.
    #[error("host I/O: {0}")]
    Io(#[from] std::io::Error),
    /// A routed native invocation hint failed its checked identity constructor.
    #[error("native invocation identity: {0}")]
    InvocationIdentity(#[from] uuid::Error),
    /// Original asynchronous task termination, preserving panic/cancellation cause.
    #[error("host task: {0}")]
    Task(#[from] tokio::task::JoinError),
    /// A staged compiler/test operation's original concrete refusal.
    #[error(transparent)]
    Workflow(Box<test_workflow::WorkflowError>),
    /// Fixed helper input, required terminal protocol or original report association failed.
    #[error(transparent)]
    WorkflowOwner(Box<managed::workflow::WorkflowOwnerError>),
    /// Captured product-reader transport retains its original native failure.
    #[error(transparent)]
    ProductReader(Box<product_reader::ProductReaderError>),
    /// Captured development-tool preparation or native forwarding failure.
    #[error(transparent)]
    PreparedXtask(Box<prepared_xtask::PreparedXtaskError>),
    /// Source-change reach preserves native Git and graph/map failures.
    #[error(transparent)]
    Affected(#[from] affected::AffectedError),
    /// D-Bus transport or manager refused an operation.
    #[error("user manager: {0}")]
    Manager(#[from] zbus::Error),
    /// A requested capability has invalid premises.
    #[error("host ownership: {0}")]
    Ownership(String),
    /// History could not establish a global selection; every original row failure survives.
    #[error("invocation history is incomplete; global latest cannot be established")]
    IncompleteInvocationHistory {
        /// Independent unreadable source records from the actual traversal.
        failures: Vec<invocation::HistoryFailure>,
    },
    /// Deadline expired without proving the named fact.
    #[error("host deadline: {0}")]
    Deadline(String),
    /// Creation may have committed; retrying can duplicate effects.
    #[error(
        "launch acknowledgement unknown for {unit}: {detail}; inspect original intent, do not replay"
    )]
    UnknownLaunch {
        /// Exact original unit name.
        unit: String,
        /// Original failure.
        detail: String,
    },
    /// Both the original operation and cleanup failed; neither failure is discarded.
    #[error("host operation failed: {operation}; cleanup also failed: {cleanup}")]
    Settlement {
        /// Original failure.
        operation: Box<HostError>,
        /// Independently attempted cleanup failure.
        cleanup: Box<HostError>,
    },
    /// Environment construction failed.
    #[error(transparent)]
    Environment(#[from] environment::EnvironmentError),
    /// Native Git observation failed, retaining the original source chain.
    #[error(transparent)]
    Repository(#[from] repository::RepositoryError),
    /// Preserve correctness/metadata interpretation and its original native causes.
    #[error(transparent)]
    Correctness(#[from] correctness::CorrectnessError),
    /// Passive source reading retains original file/selection causes.
    #[error(transparent)]
    SourceCommand(#[from] source_command::SourceCommandError),
    /// Native artifact path, range and backend failures retain their original causes.
    #[error(transparent)]
    Artifact(#[from] artifacts::ArtifactError),
    /// Generator/source ownership lookup retains original map and path causes.
    #[error(transparent)]
    SourceOwnership(#[from] ownership::OwnershipError),
}

pse_diagnostics::impl_diagnostic! {
    HostError,
    code(this) { Some(match this {
        Self::Io(_) => pse_diagnostics::DiagnosticCode::HostIo,
        Self::InvocationIdentity(_) => pse_diagnostics::DiagnosticCode::HostOwnership,
        Self::Task(_) => pse_diagnostics::DiagnosticCode::RuntimeInfrastructure,
        Self::Workflow(error) => pse_diagnostics::TypedDiagnostic::diagnostic_code(error.as_ref()).unwrap_or(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
        Self::WorkflowOwner(error) => pse_diagnostics::TypedDiagnostic::diagnostic_code(error.as_ref()).unwrap_or(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
        Self::ProductReader(error) => pse_diagnostics::TypedDiagnostic::diagnostic_code(error.as_ref()).unwrap_or(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
        Self::PreparedXtask(error) => pse_diagnostics::TypedDiagnostic::diagnostic_code(error.as_ref()).unwrap_or(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
        Self::Affected(error) => pse_diagnostics::TypedDiagnostic::diagnostic_code(error).unwrap_or(pse_diagnostics::DiagnosticCode::HostOwnership),
        Self::Manager(_) => pse_diagnostics::DiagnosticCode::HostManager,
        Self::Ownership(_) => pse_diagnostics::DiagnosticCode::HostOwnership,
        Self::IncompleteInvocationHistory {..} => pse_diagnostics::DiagnosticCode::HostOwnership,
        Self::Deadline(_) => pse_diagnostics::DiagnosticCode::HostDeadline,
        Self::UnknownLaunch {..} => pse_diagnostics::DiagnosticCode::HostUnknownLaunch,
        Self::Settlement {..} => pse_diagnostics::DiagnosticCode::HostSettlement,
        Self::Environment(_) => pse_diagnostics::DiagnosticCode::HostEnvironment,
        Self::Repository(error) => pse_diagnostics::TypedDiagnostic::diagnostic_code(error).unwrap_or(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
        Self::Correctness(error) => pse_diagnostics::TypedDiagnostic::diagnostic_code(error).unwrap_or(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
        Self::SourceCommand(error) => pse_diagnostics::TypedDiagnostic::diagnostic_code(error).unwrap_or(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
        Self::Artifact(error) => pse_diagnostics::TypedDiagnostic::diagnostic_code(error).unwrap_or(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
        Self::SourceOwnership(error) => pse_diagnostics::TypedDiagnostic::diagnostic_code(error).unwrap_or(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
    }) },
    forward(this) { match this { Self::Workflow(error) => Some(error.as_ref()), Self::WorkflowOwner(error) => Some(error.as_ref()), Self::ProductReader(error) => Some(error.as_ref()), Self::PreparedXtask(error) => Some(error.as_ref()), Self::Affected(error) => Some(error), Self::Environment(error) => Some(error), Self::Repository(error) => Some(error), Self::Correctness(error) => Some(error), Self::SourceCommand(error) => Some(error), Self::Artifact(error) => Some(error), Self::SourceOwnership(error) => Some(error), _ => None } },
    help(_this) { None },
    related(this) { match this {
        Self::Settlement {operation,cleanup} => { let children:[&dyn pse_diagnostics::TypedDiagnostic;2]=[operation.as_ref(),cleanup.as_ref()]; Some(Box::new(children.into_iter())) },
        Self::IncompleteInvocationHistory {failures} => Some(Box::new(failures.iter().map(|failure| -> &dyn pse_diagnostics::TypedDiagnostic { failure }))),
        _ => None
    } },
    source(_this) { None }
}

/// Passive native process telemetry selection.
pub mod telemetry;
/// Explicit persistent local telemetry service operator.
pub mod telemetry_service;
