// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared boundary vocabulary. Performing owners retain populations and release authority.
/// Operational lifecycle facts do not establish scientific acceptance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleBoundary {
    /// Waiting for original admission.
    AdmissionWait,
    /// Original admission acquired.
    Admitted,
    /// Actual dedicated work dispatched.
    Dispatched,
    /// Cancellation requested; work remains owned.
    StopRequested,
    /// Original deadline expired; work remains owned.
    DeadlineExpired,
    /// Actual owned drain settled.
    DrainSettled,
    /// Actual thread join including TLS/foreign destruction.
    Joined,
    /// The performing owner returned a failure.
    ReturnFailed,
}
impl LifecycleBoundary {
    /// Existing native-job hub key, independent of tracing/exporter presence.
    pub const fn native_job_key(self) -> &'static str {
        match self {
            Self::AdmissionWait => "native.admission.wait",
            Self::Admitted => "native.admitted",
            Self::Dispatched => "native.dispatched",
            Self::StopRequested => "native.stop_requested",
            Self::DeadlineExpired => "native.deadline_expired",
            Self::DrainSettled => "native.drain.settled",
            Self::Joined => "native.joined",
            Self::ReturnFailed => "native.return.failed",
        }
    }
    /// Python native pulls retain their own observation population and names.
    pub const fn python_pull_key(self) -> &'static str {
        match self {
            Self::AdmissionWait => "python.native_pull.admission.wait",
            Self::Admitted => "python.native_pull.admitted",
            Self::Dispatched => "python.native_pull.dispatched",
            Self::StopRequested => "python.native_pull.stop_requested",
            Self::DeadlineExpired => "python.native_pull.deadline_expired",
            Self::DrainSettled => "python.native_pull.drain.settled",
            Self::Joined => "python.native_pull.joined",
            Self::ReturnFailed => "python.native_pull.return.failed",
        }
    }
    /// Existing session hub key. Session joins are a separate population.
    pub const fn native_session_key(self) -> &'static str {
        match self {
            Self::AdmissionWait => "native.session.admission.wait",
            Self::Admitted => "native.session.admitted",
            Self::Dispatched => "native.session.dispatched",
            Self::StopRequested => "native.session.stop_requested",
            Self::DeadlineExpired => "native.session.deadline_expired",
            Self::DrainSettled => "native.session.drain.settled",
            Self::Joined => "native.session.joined",
            Self::ReturnFailed => "native.session.return.failed",
        }
    }
}
