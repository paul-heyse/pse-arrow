// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Child-local immutable demand and mutable native attempt state.
use super::WorkerError;
use pse_ids::SemanticId;
use pse_kernels::{
    EvaluationContext, ExecutionScope, PreparedProviderRequest, ProviderBatchOutcome,
    ProviderBatchPoint, ProviderBinding, ProviderBuffer, ProviderError, ProviderFactory,
    ProviderRequest, ProviderRuntime, ProviderSpec, ProviderValues,
};

/// Attempt-bound context reference, checked on every invocation; never a proof token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionHandle {
    /// Explicit enclosing attempt identity.
    pub attempt: SemanticId,
    /// Fresh launch identity owned by the parent.
    pub session: SemanticId,
    /// Fresh prepared context identity; replacement never retains it.
    pub generation: SemanticId,
}

/// Original attempt state; failure cannot transition back to ready.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionState {
    /// Immutable physical contract and actual products have been admitted.
    Ready,
    /// One coherent native invocation owns the mutable state.
    Evaluating,
    /// No further invocation may enter.
    Stopping,
    /// Native state has been dropped.
    Closed,
    /// Infrastructure failed; no previous completion remains current.
    Failed,
}

/// A prepared native provider and its exact original execution scope.
#[derive(Debug)]
pub struct PreparedSession {
    handle: SessionHandle,
    state: SessionState,
    binding: Option<ProviderBinding>,
    request: PreparedProviderRequest,
    buffer: ProviderBuffer,
    scope: ExecutionScope,
    last_invocation: u64,
    result_limit: usize,
}
impl PreparedSession {
    /// Admit the actual reconstructed physical contract and native factory.
    /// Expected hashes alone cannot substitute for the reconstructed factory.
    pub fn prepare(
        handle: SessionHandle,
        factory: &dyn ProviderFactory,
        expected: &ProviderSpec,
        configuration: pse_ids::ContentHash,
        request: ProviderRequest,
        scope: ExecutionScope,
        result_limit: usize,
    ) -> Result<Self, WorkerError> {
        scope.check()?;
        if [handle.attempt, handle.session, handle.generation].contains(&SemanticId::NIL) {
            return Err(ProviderError::Contract(
                "scientific worker requires fresh nonzero context identities".into(),
            )
            .into());
        }
        if factory.spec() != expected || factory.configuration_key() != configuration {
            return Err(ProviderError::Contract(
                "scientific worker reconstructed physical/data/datum contract differs".into(),
            )
            .into());
        }
        let checked =
            PreparedProviderRequest::new(std::sync::Arc::new(expected.clone()), request.clone())?;
        checked.check_context(&EvaluationContext {
            cancelled: scope.cancellation(),
            max_result_bytes: result_limit,
        })?;
        let mut binding = factory.create_scoped(scope.nested())?;
        if binding.supports_linearization() {
            return Err(ProviderError::Contract(
                "scientific worker has no admitted transferable linearization context".into(),
            )
            .into());
        }
        let request = binding.prepare(request)?;
        let context = EvaluationContext {
            cancelled: scope.cancellation(),
            max_result_bytes: result_limit,
        };
        request.check_context(&context)?;
        // The launcher admits request/result/decoder and opaque storage before
        // this constructor; this check bounds the exact numerical allocation.
        let buffer = ProviderBuffer::new(&request);
        scope.check()?;
        Ok(Self {
            handle,
            state: SessionState::Ready,
            binding: Some(binding),
            request,
            buffer,
            scope,
            last_invocation: 0,
            result_limit,
        })
    }
    /// Exact parent/session/generation reference established during preparation.
    pub fn handle(&self) -> SessionHandle {
        self.handle
    }
    /// Current mutable attempt state.
    pub fn state(&self) -> SessionState {
        self.state
    }
    /// Exact immutable demand, prepared once before callbacks.
    pub fn request(&self) -> &PreparedProviderRequest {
        &self.request
    }
    /// Genuine group capability of this exact native preparation.
    pub fn supports_batch(&self) -> bool {
        self.binding
            .as_ref()
            .is_some_and(|binding| binding.supports_batch(&self.request))
    }
    /// Execute one native group under the original scope and shared scalar/group invocation sequence.
    pub fn evaluate_batch(
        &mut self,
        handle: SessionHandle,
        invocation: u64,
        points: &[ProviderBatchPoint],
    ) -> Result<Vec<ProviderBatchOutcome>, WorkerError> {
        if let Err(error) = self.scope.check() {
            self.close();
            return Err(error.into());
        }
        if self.state != SessionState::Ready
            || handle != self.handle
            || invocation == 0
            || invocation <= self.last_invocation
            || !self.supports_batch()
        {
            return Err(ProviderError::Contract(
                "scientific worker stale or unsupported batch context".into(),
            )
            .into());
        }
        // Frame refusal precedes the mutable invocation transition and native admission.
        self.binding
            .as_ref()
            .ok_or_else(|| {
                ProviderError::Terminal("scientific worker native state is closed".into())
            })?
            .check_batch_frame(
                points,
                &self.request,
                &EvaluationContext {
                    cancelled: self.scope.cancellation(),
                    max_result_bytes: self.result_limit,
                },
            )?;
        self.last_invocation = invocation;
        self.state = SessionState::Evaluating;
        let result: Result<Vec<ProviderBatchOutcome>, ProviderError> = (|| {
            let binding = self.binding.as_mut().ok_or_else(|| {
                ProviderError::Terminal("scientific worker native state is closed".into())
            })?;
            let context = EvaluationContext {
                cancelled: self.scope.cancellation(),
                max_result_bytes: self.result_limit,
            };
            let completed = binding.evaluate_batch(points, &self.request, &context)?;
            self.scope.check()?;
            Ok(completed)
        })();
        self.state = match &result {
            Ok(_) => SessionState::Ready,
            Err(error) if error.recoverable() => SessionState::Ready,
            Err(_) => SessionState::Failed,
        };
        result.map_err(Into::into)
    }

    /// Execute exactly once, with ordered numerical products and no proof export.
    pub fn evaluate(
        &mut self,
        handle: SessionHandle,
        invocation: u64,
        inputs: &[f64],
        runtime: ProviderRuntime<'_>,
    ) -> Result<ProviderValues, WorkerError> {
        if let Err(error) = self.scope.check() {
            self.close();
            return Err(error.into());
        }
        if self.state != SessionState::Ready
            || handle != self.handle
            || invocation == 0
            || invocation <= self.last_invocation
            || runtime.expected_linearization.is_some()
        {
            return Err(ProviderError::Contract(
                "scientific worker stale session, invocation or linearization context".into(),
            )
            .into());
        }
        self.last_invocation = invocation;
        self.state = SessionState::Evaluating;
        let result: Result<ProviderValues, ProviderError> = (|| {
            let binding = self.binding.as_mut().ok_or_else(|| {
                ProviderError::Terminal("scientific worker native state is closed".into())
            })?;
            let context = EvaluationContext {
                cancelled: self.scope.cancellation(),
                max_result_bytes: self.result_limit,
            };
            let completed = self
                .buffer
                .fill(binding, inputs, &self.request, runtime, &context)?;
            self.scope.check()?;
            Ok(ProviderValues {
                values: completed.values.to_vec(),
                jacobian: completed.jacobian.to_vec(),
                hessians: completed.hessians.to_vec(),
            })
        })();
        self.state = match &result {
            Ok(_) => SessionState::Ready,
            Err(error) if error.recoverable() => SessionState::Ready,
            Err(_) => SessionState::Failed,
        };
        result.map_err(Into::into)
    }
    /// Request cancellation for this handle's scope, then retire its settled native state.
    /// The caller must run this on the same native lane after its invocation has joined;
    /// requesting cancellation alone does not establish physical settlement.
    pub fn stop(&mut self) {
        self.scope.cancellation_control().cancel();
        self.close();
    }
    /// Stop admission and destroy settled state without changing the original stop cause.
    /// The caller must run this on the same native lane after its invocation.
    pub fn close(&mut self) {
        self.state = SessionState::Stopping;
        drop(self.binding.take());
        self.state = SessionState::Closed;
    }
}
