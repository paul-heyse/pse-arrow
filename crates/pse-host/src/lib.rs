// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent native host capabilities. A manager reply is not physical release.
//!
//! [`HostManager::current_membership`] keeps the immutable unit root separate from the
//! authenticated actual member. Only a live [`OwnedUnit`] or that actual native workload
//! membership can construct a [`PayloadParent`]. Fresh child creation yields an opaque
//! [`OwnedSubgroup`]; inert identity snapshots cannot adopt existing directories.
//!
//! Callers keep children unreaped and gated while attaching/revalidating. They also serialize
//! parent launch closure against recursive kill, positive drain and original-inode removal.
//! These APIs report kernel facts, not that lifecycle cutoff, reservation release or managed
//! terminal settlement. Root limits remain aggregate; borrowing never grants root stop.
//! Dropping a capability establishes no cleanup. Production recovery from protected ledger
//! records and the shared service lifecycle remain pending; recovery probes are test-only.

pub mod activity;
pub mod admission;
pub mod agent_hooks;
pub mod artifacts;
pub mod boundary;
pub mod catalog;
pub mod cli;
pub mod correctness;
pub mod diagnostic_tools;
pub mod dependency;
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
pub mod repository;
pub mod selection;
pub mod server;
pub mod source_command;
pub mod source_proposal;
pub mod source_read;
pub mod status;
pub mod stream_records;
pub mod study_module;
pub mod test_workflow;
pub use manager::{
    AttachedChild, BorrowedService, BorrowedUnit, CreationOutcome, CurrentMembership,
    DirectoryBinding, HostManager, LaunchIntent, OpenedSubgroupCustody, OrdinaryPlacement,
    OwnedSubgroup, OwnedUnit, PayloadParent, PayloadParentBinding, RemovalReceipt,
    ResourceControls, ServiceFinalizer, ServiceSpec, SubgroupCreationIntent, UnitObservation,
};

/// A host boundary failure, never a scientific outcome.
#[derive(Debug, thiserror::Error)]
pub enum HostError {
    /// Native I/O failed.
    #[error("host I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Original asynchronous task termination, preserving panic/cancellation cause.
    #[error("host task: {0}")]
    Task(#[from] tokio::task::JoinError),
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
        Self::Task(_) => pse_diagnostics::DiagnosticCode::RuntimeInfrastructure,
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
    forward(this) { match this { Self::Environment(error) => Some(error), Self::Repository(error) => Some(error), Self::Correctness(error) => Some(error), Self::SourceCommand(error) => Some(error), Self::Artifact(error) => Some(error), Self::SourceOwnership(error) => Some(error), _ => None } },
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
