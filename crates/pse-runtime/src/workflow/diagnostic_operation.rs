// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One diagnostic caller clock and cancellation owner, from preparation through publication.
use super::{WorkflowError, contract, deadline_limit};
use crate::{CancelSource, math::MathService};
use pse_diagnostics::observation::OperationId;
use pse_model::{
    diagnostic::BoundaryDiagnostic,
    generated::identities::{AttemptId, RunId},
};
use std::{
    future::Future,
    time::{Duration, Instant},
};

/// Optional link to retained original evidence; never authority to use a scientific result.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticOrigin {
    /// Original retained run identity.
    pub run_id: RunId,
    /// Original durable attempt, absent for ephemeral execution.
    pub attempt_id: Option<AttemptId>,
    /// Original requested step, when one is selected.
    pub step: Option<usize>,
    /// Original zero-based native attempt ordinal.
    pub native_attempt: Option<usize>,
    /// Original published diagnostic operation, when observation was enabled.
    pub operation: Option<OperationId>,
    /// Retained original failure or scientific refusal; never reevaluated.
    pub failure: Option<BoundaryDiagnostic>,
}
impl DiagnosticOrigin {
    pub(super) fn retained_bytes(&self) -> usize {
        use pse_model::HeapUsage;
        size_of::<Self>().saturating_add(self.failure.as_ref().map_or(0, HeapUsage::heap_bytes))
    }
}

/// Explicit nonserialized execution owner. Create it before declaration/preparation; reuse it
/// across all requested samples and native analyses instead of granting each stage a new clock.
#[derive(Clone, Debug)]
pub struct DiagnosticOperation {
    driver: CancelSource,
    scope: pse_kernels::ExecutionScope,
    /// Optional original evidence link, independent of scientific permission.
    pub origin: Option<DiagnosticOrigin>,
}
impl DiagnosticOperation {
    /// Start the caller's finite allowance, narrowed by any enclosing original clock.
    pub fn new(
        parent: &CancelSource,
        allowance: Duration,
        origin: Option<DiagnosticOrigin>,
    ) -> Result<Self, WorkflowError> {
        if allowance.is_zero() {
            return Err(contract("diagnostic time allowance must be positive"));
        }
        let deadline = Instant::now()
            .checked_add(allowance)
            .ok_or_else(|| deadline_limit("diagnostic deadline extent"))?;
        let driver = parent.child_with_deadline(Some(deadline));
        let scope = pse_kernels::ExecutionScope::new(
            pse_diagnostics::Cancellation::default(),
            driver.deadline(),
        );
        Ok(Self {
            driver,
            scope,
            origin,
        })
    }
    /// Driver used by this caller's preparation waiters.
    pub fn driver(&self) -> &CancelSource {
        &self.driver
    }
    /// Original numerical clock and notifying cancellation owner.
    pub fn scope(&self) -> &pse_kernels::ExecutionScope {
        &self.scope
    }
    /// Stop owned numerical work as well as asynchronous preparation waiters.
    pub fn cancel(&self) {
        self.driver.cancel();
        self.scope.cancellation_control().cancel();
    }
    /// Refuse late publication even after an opaque numerical library returns.
    pub fn check(&self) -> Result<(), WorkflowError> {
        if self.driver.token().is_cancelled() {
            return Err(crate::math::MathRuntimeError::Cancelled.into());
        }
        self.scope
            .check()
            .map_err(pse_backend_native::ProblemError::Provider)
            .map_err(crate::math::MathRuntimeError::from)?;
        Ok(())
    }
    /// Bound a shared preparation wait without cancelling its independently owned producer.
    pub async fn within<T>(
        &self,
        operation: impl Future<Output = Result<T, WorkflowError>>,
    ) -> Result<T, WorkflowError> {
        MathService::within_task(&self.scope, &self.driver, operation).await
    }
    pub(super) fn retained_bytes(&self) -> usize {
        self.origin
            .as_ref()
            .map_or(0, DiagnosticOrigin::retained_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn diagnostic_original_clock_covers_preparation_and_refuses_late_publication() {
        let parent =
            CancelSource::new().with_deadline(Some(Instant::now() + Duration::from_millis(20)));
        let task = DiagnosticOperation::new(&parent, Duration::from_secs(60), None).unwrap();
        assert_eq!(task.scope().deadline(), parent.deadline());
        assert!(
            task.within(std::future::pending::<Result<(), WorkflowError>>())
                .await
                .is_err()
        );
        assert!(task.check().is_err());
        assert!(!parent.token().is_cancelled());
    }
    #[tokio::test]
    async fn diagnostic_cancel_stops_owned_work_without_cancelling_parent() {
        let parent = CancelSource::new();
        let task = DiagnosticOperation::new(&parent, Duration::from_secs(10), None).unwrap();
        task.cancel();
        assert!(task.check().is_err());
        assert!(
            task.scope()
                .cancellation()
                .load(std::sync::atomic::Ordering::Acquire)
        );
        assert!(!parent.token().is_cancelled());
    }
}
