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
