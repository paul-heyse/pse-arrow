// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Runtime adapters preserve their owner's optional original absolute clock.
use pse_diagnostics::stop::{StopReason, StopSignal};
use std::time::Instant;
async fn stopped(control: impl Future<Output = ()>, deadline: Option<Instant>) -> StopReason {
    match pse_diagnostics::stop::stop_first(
        async { control.await; StopReason::Cancelled },
        async { match deadline {
            Some(at) => tokio::time::sleep_until(at.into()).await,
            None => std::future::pending().await,
        } },
    ).await {
        Err(reason) => reason,
        Ok(()) => StopReason::Deadline,
    }
}
impl StopSignal for crate::CancelSource {
    fn check(&self) -> Option<StopReason> {
        if self.token().is_cancelled() { Some(StopReason::Cancelled) }
        else if self.deadline().is_some_and(|at| Instant::now() >= at) { Some(StopReason::Deadline) }
        else { None }
    }
    async fn stopped(&self) -> StopReason { if let Some(reason) = self.check() { return reason; } stopped(self.cancelled(), self.deadline()).await }
}
pub(crate) struct KernelStop<'a>(pub(crate) &'a pse_kernels::ExecutionScope);
impl StopSignal for KernelStop<'_> {
    fn check(&self) -> Option<StopReason> {
        if self.0.cancellation_control().is_cancelled() { Some(StopReason::Cancelled) }
        else if self.0.deadline().is_some_and(|at| Instant::now() >= at) { Some(StopReason::Deadline) }
        else { None }
    }
    async fn stopped(&self) -> StopReason { if let Some(reason) = self.check() { return reason; } stopped(self.0.cancellation_control().cancelled(), self.0.deadline()).await }
}

/// A native flat cancellation owner with this performing operation's explicit clock.
pub(crate) struct NativeStop<'a> {
    pub(crate) control: &'a pse_diagnostics::Cancellation,
    pub(crate) deadline: Option<Instant>,
}
impl StopSignal for NativeStop<'_> {
    fn check(&self) -> Option<StopReason> {
        if self.control.is_cancelled() { Some(StopReason::Cancelled) }
        else if self.deadline.is_some_and(|at| Instant::now() >= at) { Some(StopReason::Deadline) }
        else { None }
    }
    async fn stopped(&self) -> StopReason {
        if let Some(reason) = self.check() { return reason; }
        stopped(self.control.cancelled(), self.deadline).await
    }
}
