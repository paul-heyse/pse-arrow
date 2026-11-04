// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Synchronous provider placement with original-scope process and I/O custody.
use super::{
    PreparedExecutable, ProcessCustody, SessionHandle, WorkerError, handler, infrastructure,
    protocol as p,
    transport::{DECODE_LIMIT, FRAME_LIMIT, IoOwner, NATIVE_STACK},
};
use pse_backend_native::coolprop::{WaterFactory, WaterRegime};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::{
    EvaluationContext, ExecutionScope, PreparedProviderRequest, Provider, ProviderBinding,
    ProviderDemand, ProviderError, ProviderFactory, ProviderOutput, ProviderRequest,
    ProviderRuntime, ProviderSpec, ProviderValues, Registration,
    ProviderBatchPoint, ProviderBatchResults,
};
use std::{
    sync::{Arc, Weak},
    time::{Duration, Instant},
};

/// Explicit prepared local placement of exactly the selected standard HEOS Water.
#[derive(Clone, Debug)]
pub struct LocalWaterFactory {
    water: WaterFactory,
    regime: WaterRegime,
    executable: PreparedExecutable,
    service: Weak<crate::math::MathService>,
    configuration: ContentHash,
}
impl LocalWaterFactory {
    /// Prepare the actual parent science and select an already deployed executable.
    /// The v1 controlled placement explicitly refuses a custom physical registry.
    pub fn prepare(
        executable: PreparedExecutable,
        registry: &pse_quantity::QuantityRegistry,
        regime: WaterRegime,
        service: Arc<crate::math::MathService>,
    ) -> Result<Self, WorkerError> {
        let standard = pse_quantity::generated::standard_registry()
            .map_err(|error| ProviderError::Contract(error.to_string()))?;
        let prerequisites = pse_quantity::PhysicalPreconditions::new(Vec::new())
            .map_err(|error| ProviderError::Contract(error.to_string()))?;
        if pse_quantity::physical_identity::identity(registry, &prerequisites)
            != pse_quantity::physical_identity::identity(&standard, &prerequisites)
        {
            return Err(ProviderError::Contract(
                "scientific worker v1 requires the actual complete standard physical registry"
                    .into(),
            )
            .into());
        }
        let water = WaterFactory::prepare(registry, regime)?;
        let mut configuration = pse_ids::FramedHasher::new(pse_ids::Frame::ProviderConfigurationV1);
        use std::os::unix::ffi::OsStrExt as _;
        configuration
            .str("prepared-local-scientific-worker-v1")
            .hash(&water.configuration_key())
            .str(&executable.observation().sha256)
            .part(executable.path().as_os_str().as_bytes())
            .u64(executable.opaque_bytes() as u64);
        Ok(Self {
            water,
            regime,
            executable,
            service: Arc::downgrade(&service),
            configuration: configuration.finish_hash(),
        })
    }
    /// Supply this explicitly selected placement through the ordinary provider ingress.
    pub fn registration(
        executable: PreparedExecutable,
        registry: &pse_quantity::QuantityRegistry,
        regime: WaterRegime,
        service: Arc<crate::math::MathService>,
    ) -> Result<Registration, WorkerError> {
        let descriptor = pse_backend_native::coolprop::descriptor(registry, regime)?;
        Ok(Registration::bind(
            descriptor,
            Arc::new(Self::prepare(executable, registry, regime, service)?),
        )?)
    }
}
impl ProviderFactory for LocalWaterFactory {
    fn spec(&self) -> &ProviderSpec {
        self.water.spec()
    }
    fn configuration_key(&self) -> ContentHash {
        self.configuration
    }
    fn create(&self) -> Result<ProviderBinding, ProviderError> {
        self.create_scoped(ExecutionScope::new(
            pse_diagnostics::Cancellation::default(),
            None,
        ))
    }
    fn create_scoped(&self, scope: ExecutionScope) -> Result<ProviderBinding, ProviderError> {
        scope.check()?;
        Ok(ProviderBinding::new(
            self.spec().clone(),
            Box::new(Proxy {
                factory: self.clone(),
                scope,
                attempt: pse_operations::mint_id(),
                remote: None,
                prepared: vec![],
                invocation: 0,
                failed: false,
            }),
        ))
    }
}

struct Remote {
    io: Option<IoOwner>,
    child: ProcessCustody,
    _endpoint: tempfile::TempDir,
    session: SemanticId,
    closed: bool,
}
impl std::fmt::Debug for Remote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScientificRemote")
            .field("pid", &self.child.pid())
            .field("session", &self.session)
            .finish_non_exhaustive()
    }
}
impl Remote {
    fn exchange(
        &self,
        request: p::Request,
        scope: &ExecutionScope,
        invocation_cancel: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<p::Response, WorkerError> {
        self.io
            .as_ref()
            .ok_or_else(|| infrastructure("scientific worker I/O absent"))?
            .exchange(request, scope, invocation_cancel)
    }
    fn fail(mut self, cause: WorkerError) -> WorkerError {
        // The aggregate reservation also covers parent I/O buffers and its
        // executor. Reaping the child settles only one of those owners.
        let admission = self.child.admission_guard();
        let cause = self.child.settle_failure(cause);
        self.io.take();
        drop(admission);
        cause
    }
}
impl Drop for Remote {
    fn drop(&mut self) {
        #[cfg(test)]
        self.child.observe_close(self.closed);
        // Dropping I/O after an unclosed channel causes the helper to terminate
        // native work. Child custody still retains admission through actual reap.
        self.io.take();
        if let Err(error) = self.child.drain(if self.closed {
            Duration::from_millis(100)
        } else {
            Duration::ZERO
        }) {
            tracing::error!(error = %error, "scientific worker drop could not establish reap");
        }
    }
}

#[derive(Debug)]
struct Proxy {
    factory: LocalWaterFactory,
    scope: ExecutionScope,
    attempt: SemanticId,
    remote: Option<Remote>,
    prepared: Vec<(ProviderRequest, p::Handle, bool)>,
    invocation: u64,
    failed: bool,
}
impl Proxy {
    fn launch(&mut self) -> Result<(), WorkerError> {
        if self.failed {
            return Err(infrastructure("scientific worker binding is invalidated"));
        }
        if self.remote.is_some() {
            return Ok(());
        }
        self.scope.check()?;
        let service = self
            .factory
            .service
            .upgrade()
            .ok_or_else(|| infrastructure("scientific worker runtime retired"))?;
        let known = 8 * FRAME_LIMIT + 4 * DECODE_LIMIT + NATIVE_STACK + (6 << 20) + 64 * 4096;
        let bytes = self
            .factory
            .executable
            .opaque_bytes()
            .checked_add(known)
            .ok_or(ProviderError::Limit("scientific worker admitted bytes"))?;
        let admission = service
            .reserve("math:prepared-scientific-worker", bytes)
            .map_err(|error| {
                WorkerError::Provider(ProviderError::Nested {
                    retained: error.retained_bytes().unwrap_or(usize::MAX),
                    cause: pse_model::diagnostic::DiagnosticCause::new(error),
                })
            })?;
        let directory = tempfile::Builder::new()
            .prefix("pse-science-")
            .tempdir()
            .map_err(infrastructure)?;
        let endpoint = directory.path().join("worker.sock");
        let session = pse_operations::mint_id();
        let mut child =
            self.factory
                .executable
                .launch(&endpoint, session, &self.scope, admission, false)?;
        let startup = Instant::now()
            .checked_add(self.scope.remaining(Duration::from_secs(30))?)
            .ok_or(ProviderError::Limit("scientific worker startup extent"))?;
        let connected = (|| {
            while !endpoint.exists() {
                self.scope.check()?;
                child.check_running()?;
                if Instant::now() >= startup {
                    return Err(infrastructure(
                        "scientific worker startup allowance expired",
                    ));
                }
                std::thread::sleep(Duration::from_millis(2));
            }
            child.authenticate_image()?;
            IoOwner::connect(endpoint, child.pid(), &self.scope)
        })();
        let io = match connected {
            Ok(io) => io,
            Err(error) => return Err(child.settle_failure(error)),
        };
        self.remote = Some(Remote {
            io: Some(io),
            child,
            _endpoint: directory,
            session,
            closed: false,
        });
        Ok(())
    }
    fn fail(&mut self, error: WorkerError) -> WorkerError {
        self.failed = true;
        self.prepared.clear();
        match self.remote.take() {
            Some(remote) => remote.fail(error),
            None => error,
        }
    }
    fn handle_for(&self, request: &PreparedProviderRequest) -> Option<p::Handle> {
        self.prepared
            .iter()
            .find(|(actual, _, _)| {
                actual.outputs == request.outputs
                    && actual.order == request.order
                    && actual.demand == request.demand
            })
            .map(|(_, handle, _)| handle.clone())
    }
    fn prepare(&mut self, request: &PreparedProviderRequest) -> Result<(), WorkerError> {
        self.scope.check()?;
        if self.handle_for(request).is_some() {
            return Ok(());
        }
        if self.prepared.len() >= 64 {
            return Err(ProviderError::Limit("scientific worker prepared demand count").into());
        }
        self.launch()?;
        let remote = self
            .remote
            .as_ref()
            .ok_or_else(|| infrastructure("scientific worker process absent"))?;
        let handle = p::Handle::from(SessionHandle {
            attempt: self.attempt,
            session: remote.session,
            generation: pse_operations::mint_id(),
        });
        let remaining_ns = scientific_remaining_ns(&self.scope, Instant::now())?;
        let prepare = p::Prepare {
            version: 1,
            handle: handle.clone(),
            spec: p::Spec::capture(self.factory.spec())?,
            configuration: self.factory.water.configuration_key(),
            physical_registry: "pse.standard.v1".into(),
            regime: match self.factory.regime {
                WaterRegime::Liquid => "liquid",
                WaterRegime::Vapor => "vapor",
                WaterRegime::Supercritical => "supercritical",
            }
            .into(),
            outputs: request.outputs.iter().map(|&n| n as u64).collect(),
            order: request.order as u32,
            demand: p::Demand::capture(&request.demand),
            remaining_ns,
            // Transient grouped numerical storage is admitted by the launch reservation's frame
            // allowance. Preparation still allocates only this demand's scalar scratch.
            result_limit: FRAME_LIMIT as u64,
        };
        match remote.exchange(p::Request::Prepare(prepare), &self.scope, None)? {
            p::Response::Prepared(actual) if actual.handle == handle => {
                self.prepared.push((request.request().clone(), handle, actual.batch_supported));
                Ok(())
            }
            p::Response::Failure(value) => Err(handler::refusal(value)),
            _ => Err(WorkerError::UnknownCompletion(
                "scientific worker preparation identity differs".into(),
            )),
        }
    }
}
// Startup and drain have their own finite operational allowances. They never
// become the lifetime of this scientific binding.
fn scientific_remaining_ns(
    scope: &ExecutionScope,
    now: Instant,
) -> Result<Option<u64>, ProviderError> {
    scope.check()?;
    scope
        .deadline()
        .map(|deadline| {
            let remaining = deadline
                .checked_duration_since(now)
                .filter(|value| !value.is_zero())
                .ok_or(ProviderError::Deadline)?;
            u64::try_from(remaining.as_nanos())
                .map_err(|_| ProviderError::Limit("scientific worker remaining time extent"))
        })
        .transpose()
}
impl Provider for Proxy {
    fn supports_batch(&self, request: &PreparedProviderRequest) -> bool {
        self.prepared.iter().any(|(actual, _, batch)| *batch && actual == request.request())
    }
    fn fill_batch(&mut self, points: &[&ProviderBatchPoint], request: &PreparedProviderRequest,
        context: &EvaluationContext<'_>) -> Result<ProviderBatchResults, ProviderError> {
        if let Err(error) = self.scope.check().and_then(|()| context.check()) {
            return Err(self.fail(error.into()).into_provider());
        }
        if !self.supports_batch(request) { return Err(ProviderError::Contract("scientific worker batch demand unavailable".into())); }
        let handle = self.handle_for(request).ok_or_else(|| ProviderError::Contract("scientific worker demand was not prepared".into()))?;
        self.invocation = self.invocation.checked_add(1).ok_or(ProviderError::Limit("scientific worker invocation identity"))?;
        let bits = |values: &[f64]| values.iter().map(|value| value.to_bits()).collect();
        let result = (|| {
            let remote = self.remote.as_ref().ok_or_else(|| infrastructure("scientific worker binding is invalidated"))?;
            match remote.exchange(p::Request::EvaluateBatch(p::EvaluateBatch {
                handle: handle.clone(), invocation: self.invocation,
                points: points.iter().map(|point| p::BatchPoint { id: point.id,
                    inputs: bits(&point.inputs), directions: bits(&point.directions),
                    cotangents: bits(&point.cotangents), weights: bits(&point.weights),
                }).collect(),
            }), &self.scope, Some(context.cancelled))? {
                p::Response::Batch(batch) if batch.handle == handle && batch.invocation == self.invocation => {
                    if batch.points.len() != points.len() { return Err(WorkerError::UnknownCompletion("scientific worker group completion extent differs".into())); }
                    let extent = request.extents();
                    let mut results = Vec::with_capacity(points.len());
                    for (actual, original) in batch.points.into_iter().zip(points) {
                        if actual.id != original.id { return Err(WorkerError::UnknownCompletion("scientific worker group point identity differs".into())); }
                        results.push(match actual.result {
                            p::BatchResult::Product(product) => {
                                if product.values.len() != extent.values || product.jacobian.len() != extent.jacobian
                                    || product.hessians.len() != extent.hessians || product.values.iter().chain(&product.jacobian)
                                        .chain(&product.hessians).any(|value| !value.is_finite()) {
                                    return Err(WorkerError::UnknownCompletion("scientific worker group product is malformed".into()));
                                }
                                Ok(ProviderValues { values: product.values, jacobian: product.jacobian, hessians: product.hessians })
                            }
                            p::BatchResult::Refused(refusal) => Err(handler::refusal(refusal).into_provider()),
                        });
                    }
                    self.scope.check()?;
                    Ok(results)
                }
                p::Response::Failure(value) => Err(handler::refusal(value)),
                _ => Err(WorkerError::UnknownCompletion("scientific worker group completion context differs".into())),
            }
        })();
        match result {
            Ok(results) => Ok(results),
            Err(WorkerError::Provider(error)) if error.recoverable() => Err(error),
            Err(error) => Err(self.fail(error).into_provider()),
        }
    }
    fn supports(&self, demand: &ProviderDemand) -> bool {
        matches!(demand, ProviderDemand::CompleteJets)
    }
    fn prepare_demand(&mut self, request: &PreparedProviderRequest) -> Result<(), ProviderError> {
        self.prepare(request)
            .map_err(|error| self.fail(error).into_provider())
    }
    fn fill(
        &mut self,
        inputs: &[f64],
        request: &PreparedProviderRequest,
        runtime: ProviderRuntime<'_>,
        mut output: ProviderOutput<'_>,
        context: &EvaluationContext<'_>,
    ) -> Result<(), ProviderError> {
        if let Err(error) = self.scope.check().and_then(|()| context.check()) {
            return Err(self.fail(error.into()).into_provider());
        }
        let handle = self.handle_for(request).ok_or_else(|| {
            ProviderError::Contract("scientific worker demand was not prepared".into())
        })?;
        self.invocation = self.invocation.checked_add(1).ok_or(ProviderError::Limit(
            "scientific worker invocation identity",
        ))?;
        let result = (|| {
            let remote = self
                .remote
                .as_ref()
                .ok_or_else(|| infrastructure("scientific worker binding is invalidated"))?;
            let response = remote.exchange(
                p::Request::Evaluate(p::Evaluate {
                    handle: handle.clone(),
                    invocation: self.invocation,
                    inputs: inputs.to_vec(),
                    directions: runtime.directions.to_vec(),
                    cotangents: runtime.cotangents.to_vec(),
                    weights: runtime.weights.to_vec(),
                }),
                &self.scope,
                Some(context.cancelled),
            )?;
            match response {
                p::Response::Values(value)
                    if value.handle == handle && value.invocation == self.invocation =>
                {
                    let expected = request.extents();
                    if value.values.len() != expected.values
                        || value.jacobian.len() != expected.jacobian
                        || value.hessians.len() != expected.hessians
                        || value
                            .values
                            .iter()
                            .chain(&value.jacobian)
                            .chain(&value.hessians)
                            .any(|cell| !cell.is_finite())
                    {
                        return Err(WorkerError::UnknownCompletion(
                            "scientific worker returned malformed or nonfinite extents".into(),
                        ));
                    }
                    let values = ProviderValues {
                        values: value.values,
                        jacobian: value.jacobian,
                        hessians: value.hessians,
                    };
                    output.copy_from(&values)?;
                    self.scope.check()?;
                    Ok(())
                }
                p::Response::Failure(value) => Err(handler::refusal(value)),
                _ => Err(WorkerError::UnknownCompletion(
                    "scientific worker invocation completion identity differs".into(),
                )),
            }
        })();
        match result {
            Ok(()) => Ok(()),
            Err(WorkerError::Provider(error)) if error.recoverable() => Err(error),
            Err(error) => Err(self.fail(error).into_provider()),
        }
    }
}
impl Drop for Proxy {
    fn drop(&mut self) {
        if let Some(mut remote) = self.remote.take() {
            if remote.closed {
                return;
            }
            let cleanup = ExecutionScope::new(
                pse_diagnostics::Cancellation::default(),
                Instant::now().checked_add(Duration::from_millis(100)),
            );
            remote.closed = matches!(
                remote.exchange(p::Request::Shutdown, &cleanup, None),
                Ok(p::Response::Shutdown)
            );
            drop(remote);
        }
    }
}

#[cfg(test)]
#[path = "proxy_tests.rs"]
mod tests;
