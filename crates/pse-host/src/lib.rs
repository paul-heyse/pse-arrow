// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent native host capabilities. A manager reply is not physical release.

pub mod environment;
mod manager;
pub use manager::{
    BorrowedUnit, HostManager, LaunchIntent, OwnedUnit, ServiceSpec, UnitObservation,
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
}

pse_diagnostics::impl_diagnostic! {
    HostError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
