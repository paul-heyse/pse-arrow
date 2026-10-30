// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent native host capabilities. A manager reply is not physical release.

pub mod activity;
pub mod admission;
pub mod agent_hooks;
pub mod boundary;
pub mod cli;
pub mod correctness;
pub mod diagnostic_tools;
pub mod discovery;
pub mod doctor;
pub mod envelope;
pub mod environment;
pub mod foreground;
pub mod generation;
pub mod invocation;
mod manager;
pub mod native;
pub mod paths;
pub mod repository;
pub mod selection;
pub mod server;
pub mod study_module;
pub use manager::{
    BorrowedService, BorrowedUnit, HostManager, LaunchIntent, OrdinaryPlacement, OwnedUnit,
    ResourceControls, ServiceSpec, UnitObservation,
};

/// A host boundary failure, never a scientific outcome.
#[derive(Debug, thiserror::Error)]
pub enum HostError {
    /// Native I/O failed.
    #[error("host I/O: {0}")]
    Io(#[from] std::io::Error),
    /// D-Bus transport or manager refused an operation.
    #[error("user manager: {0}")]
    Manager(#[from] zbus::Error),
    /// A requested capability has invalid premises.
    #[error("host ownership: {0}")]
    Ownership(String),
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
}

pse_diagnostics::impl_diagnostic! {
    HostError,
    code(this) { Some(match this {
        Self::Io(_) => pse_diagnostics::DiagnosticCode::HostIo,
        Self::Manager(_) => pse_diagnostics::DiagnosticCode::HostManager,
        Self::Ownership(_) => pse_diagnostics::DiagnosticCode::HostOwnership,
        Self::Deadline(_) => pse_diagnostics::DiagnosticCode::HostDeadline,
        Self::UnknownLaunch {..} => pse_diagnostics::DiagnosticCode::HostUnknownLaunch,
        Self::Settlement {..} => pse_diagnostics::DiagnosticCode::HostSettlement,
        Self::Environment(_) => pse_diagnostics::DiagnosticCode::HostEnvironment,
        Self::Repository(error) => pse_diagnostics::TypedDiagnostic::diagnostic_code(error).unwrap_or(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
    }) },
    forward(this) { match this { Self::Environment(error) => Some(error), Self::Repository(error) => Some(error), _ => None } },
    help(_this) { None },
    related(this) { match this { Self::Settlement {operation,cleanup} => { let children:[&dyn pse_diagnostics::TypedDiagnostic;2]=[operation.as_ref(),cleanup.as_ref()]; Some(Box::new(children.into_iter())) }, _ => None } },
    source(_this) { None }
}

/// Passive native process telemetry selection.
pub mod telemetry;
/// Explicit persistent local telemetry service operator.
pub mod telemetry_service;
