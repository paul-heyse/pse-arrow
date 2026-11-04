// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Child-local immutable demand and mutable native attempt state.
use super::WorkerError;
use pse_ids::SemanticId;
use pse_kernels::{
    EvaluationContext, ExecutionScope, PreparedProviderRequest, ProviderBinding, ProviderBuffer,
    ProviderError, ProviderFactory, ProviderRequest, ProviderRuntime, ProviderSpec, ProviderValues,
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
    /// Stop admission and destroy settled state without changing the original stop cause.
    /// The caller must run this on the same native lane after its invocation.
    pub fn close(&mut self) {
        self.state = SessionState::Stopping;
        drop(self.binding.take());
        self.state = SessionState::Closed;
    }
}

#[cfg(feature = "property-coolprop")]
impl PreparedSession {
    /// Reconstruct water science from actual admitted physical declarations.
    pub fn prepare_water(
        handle: SessionHandle,
        registry: &pse_quantity::QuantityRegistry,
        regime: pse_backend_native::coolprop::WaterRegime,
        expected: &ProviderSpec,
        configuration: pse_ids::ContentHash,
        request: ProviderRequest,
        scope: ExecutionScope,
        result_limit: usize,
    ) -> Result<Self, WorkerError> {
        scope.check()?;
        let factory = pse_backend_native::coolprop::WaterFactory::prepare(registry, regime)?;
        Self::prepare(
            handle,
            &factory,
            expected,
            configuration,
            request,
            scope,
            result_limit,
        )
    }
}
