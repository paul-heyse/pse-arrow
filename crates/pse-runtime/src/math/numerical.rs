// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Execution-only mathematical artifacts use the existing flights, cache and epoch.
use super::*;
use datafusion::execution::cache::Cache;
use pse_columnar::flight::FlightError;
use pse_compiler::workspace::{NumericalObservationBuildRecipe, NumericalObservationRequest};
use pse_math::numerical_observation::NumericalObservationAssembly;

fn transfer_failure<E: std::fmt::Debug + std::fmt::Display + Into<MathRuntimeError>>(
    error: pse_math::transfer::TransferError<E>,
) -> MathRuntimeError {
    use pse_math::transfer::TransferError as T;
    match error {
        T::Admission(error) => error.into(),
        T::Math(error) => error.into(),
        T::Overflow(_) => MathRuntimeError::Limit("numerical transfer extent"),
        T::Allocation(source) => MathRuntimeError::Allocation {
            operation: "numerical transfer",
            source,
        },
        T::Layout(pse_math::jets::LayoutSnapshotError::Math(error)) => error.into(),
        other => pse_math::MathError::Contract(other.to_string()).into(),
    }
}

/// Immutable original observation binding and checked numerical realization.
/// No source mathematical body or scalar preparation is necessary to execute it.
#[derive(Debug)]
pub struct NumericalArtifact {
    pub(crate) execution: portable::AcquiredExecution,
    pub(crate) assembly: Arc<NumericalObservationAssembly>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl NumericalArtifact {
    /// Exact original compiler-issued demand and custody.
    pub fn request(&self) -> &Arc<NumericalObservationRequest> {
        self.execution.request()
    }
    /// Shared checked observation program; attempts bind their own scope and backing.
    pub fn assembly(&self) -> &Arc<NumericalObservationAssembly> {
        &self.assembly
    }
    pub(super) fn retained_bytes(&self) -> usize {
        self.execution.retained_bytes() + self._owner.size()
    }
}

impl MathService {
    /// Run the same execution-only assembly over an ordered Value population.
    pub(crate) async fn evaluate_numerical_observation_batch(
        self: &Arc<Self>,
        artifact: Arc<NumericalArtifact>,
        points: Vec<(pse_ids::SemanticId, pse_math::binding::CaseValues)>,
        driver: &crate::CancelSource,
    ) -> Result<Arc<pse_columnar::Leased<pse_math::assembly::CaseBatchOutcome>>, MathRuntimeError>
    {
        let driver = self.preparation_driver(driver)?;
        let workers = artifact.assembly.batch_worker_bytes(points.len())?;
        let bytes = workers
            .checked_mul(2)
            .and_then(|n| {
                n.checked_add(
                    points
                        .len()
                        .checked_mul(size_of::<pse_math::assembly::CaseBatchPoint<'_>>())?,
                )
            })
            .ok_or(MathRuntimeError::Limit("numerical observation population"))?;
        let control = Cancellation::default();
        let deadline = driver.deadline();
        let work = self.job_retained_construction(
            1,
            ConstructionEstimate::Opaque(bytes),
            control.clone(),
            deadline,
            move |flag| {
                let scope = pse_kernels::ExecutionScope::new(flag, deadline);
                let construction = pse_math::construction::current().ok_or_else(|| {
                    MathRuntimeError::Internal("numerical population owner absent".into())
                })?;
                let mut worker = artifact
                    .assembly
                    .batch_worker_scoped(scope, points.len(), |bytes| {
                        construction.try_grow(bytes)?;
                        Ok::<Arc<dyn pse_math::AllocationOwner>, pse_math::MathError>(Arc::new(
                            construction.clone(),
                        ))
                    })
                    .map_err(transfer_failure)?;
                let points = points
                    .iter()
                    .map(|(id, values)| pse_math::assembly::CaseBatchPoint { id: *id, values })
                    .collect::<Vec<_>>();
                let result = Arc::new(worker.constraints(&points)?);
                let retained = result
                    .retained_bytes()
                    .checked_add(2 * size_of::<usize>())
                    .ok_or(MathRuntimeError::Limit("numerical population result"))?;
                Ok((result, retained))
            },
        );
        tokio::pin!(work);
        let (result, owner) = tokio::select! {
            result = &mut work => result?,
            () = driver.cancelled() => { control.cancel(); let _ = work.await; return Err(MathRuntimeError::Cancelled); }
        };
        Ok(Arc::new(pse_columnar::Leased::new(result, owner)))
    }

    /// Execute under a fresh private worker and return independently owned results.
    pub(crate) async fn evaluate_numerical_observation(
        self: &Arc<Self>,
        artifact: Arc<NumericalArtifact>,
        values: pse_math::binding::CaseValues,
        driver: &crate::CancelSource,
    ) -> Result<Arc<pse_columnar::Leased<pse_math::guarded::Evaluation>>, MathRuntimeError> {
        let driver = self.preparation_driver(driver)?;
        let bytes = artifact
            .assembly
            .worker_bytes()
            .checked_mul(2)
            .and_then(|n| n.checked_add(4096))
            .ok_or(MathRuntimeError::Limit("numerical observation worker"))?;
        let control = Cancellation::default();
        let deadline = driver.deadline();
        let work = self.job_retained_construction(
            1,
            ConstructionEstimate::Opaque(bytes),
            control.clone(),
            deadline,
            move |flag| {
                let scope = pse_kernels::ExecutionScope::new(flag, deadline);
                let construction = pse_math::construction::current().ok_or_else(|| {
                    MathRuntimeError::Internal("numerical worker construction owner absent".into())
                })?;
                let mut worker = artifact
                    .assembly
                    .worker_scoped(scope.clone(), |bytes| {
                        construction.try_grow(bytes)?;
                        Ok::<Arc<dyn pse_math::AllocationOwner>, pse_math::MathError>(Arc::new(
                            construction.clone(),
                        ))
                    })
                    .map_err(transfer_failure)?;
                let observed = worker.evaluate(&values)?;
                let retained = size_of::<pse_math::guarded::Evaluation>()
                    + 2 * size_of::<usize>()
                    + (observed.values.len() + observed.jacobian.len() + observed.hessians.len())
                        * size_of::<f64>()
                    + observed.applicability.len()
                        * size_of::<pse_model::applicability::Observation>()
                    + observed
                        .applicability
                        .iter()
                        .map(pse_model::applicability::Observation::retained_bytes)
                        .sum::<usize>();
                construction.try_grow(retained)?;
                scope.check().map_err(pse_math::MathError::Scope)?;
                Ok((Arc::new(observed.clone()), retained))
            },
        );
        tokio::pin!(work);
        let (result, owner) = tokio::select! {
            result = &mut work => result?,
            () = driver.cancelled() => { control.cancel(); let _ = work.await; return Err(MathRuntimeError::Cancelled); }
        };
        Ok(Arc::new(pse_columnar::Leased::new(result, owner)))
    }

    /// Acquire one passive execution request through the ordinary cache and flight
    /// owners. `received` is a currently protected numerical hit, or a genuine miss.
    /// Only a miss consumes the separate original compiler build recipe.
    pub(crate) async fn numerical_artifact_for(
        self: &Arc<Self>,
        workspace: Workspace,
        request: Arc<NumericalObservationRequest>,
        recipe: NumericalObservationBuildRecipe,
        received: Option<portable::AcquiredExecution>,
        driver: &crate::CancelSource,
    ) -> Result<Arc<NumericalArtifact>, MathRuntimeError> {
        let driver = self.preparation_driver(driver)?;
        let key = Key::Numerical(request.key(), request.profile().optimization.evaluator);
        if let Some(Value::Numerical(artifact)) = self.entries.get(&key) {
            return Ok(artifact);
        }
        let _ = self.preparation_driver(&driver)?;
        let descriptor_owner = self.reserve(
            "math:numerical-flight-descriptor",
            request
                .retained_bytes()
                .checked_add(recipe.retained_bytes())
                .ok_or(MathRuntimeError::Limit("numerical request descriptor"))?,
        )?;
        let service = self.clone();
        let epoch = self.retention.generation();
        let flight_key = key.clone();
        let population = PopulationScope::capture();
        let attribution = crate::ExecutionContext::capture();
        let deadline = self.preparation_deadline(&driver)?;
        let wait = async {
            tokio::select! { biased; () = driver.cancelled() => {}, () = tokio::time::sleep_until(deadline.into()) => {} }
        };
        let loaded = self.numerical_flights.load_owned_until(flight_key, move |cancel| {
            attribution.shared_producer_at("math.numerical.producer", pse_diagnostics::observation::current())
                .instrument(population.run(async move {
                    let _descriptor_owner = descriptor_owner;
                    if let Some(Value::Numerical(artifact)) = service.entries.get(&key) { return Ok(artifact); }
                    let execution = if let Some(received) = received {
                        if received.request().request_bytes() != request.request_bytes() {
                            return Err(MathRuntimeError::Internal("numerical acquisition request changed".into()));
                        }
                        received
                    } else {
                        let build_request = request.clone();
                        let (records, _records_owner) = service.compiler_job_retained(workspace,
                            ConstructionEstimate::Known(WITHIN_WORKSPACE), cancel.clone(), Some(deadline),
                            move |authority, flag| {
                                let mut compiler = authority.lock()?;
                                let records = compiler.compile_numerical_observation(&build_request, &recipe, flag.flag())?;
                                if records.len() != build_request.instances().len()
                                    || records.iter().zip(build_request.instances()).any(|(record, instance)|
                                        record.semantic_identity() != instance.semantic_identity()
                                        || record.instance() != instance.attribution().instance()) {
                                    return Err(MathRuntimeError::Internal("numerical original semantic seal changed".into()));
                                }
                                let bytes = records.iter().try_fold(0usize, |n,record|
                                    n.checked_add(record.request().descriptor_bytes()))
                                    .ok_or(MathRuntimeError::Limit("numerical warm requests"))?;
                                Ok((records, bytes))
                            }).await?;
                        let producer_driver = crate::CancelSource::new().with_deadline(Some(deadline));
                        let requests = records.iter().map(|record| record.request().clone()).collect::<Vec<_>>();
                        let pending = service.prepare_artifacts(requests.iter(), &producer_driver);
                        tokio::pin!(pending);
                        let artifacts = tokio::select! {
                            result = &mut pending => result?,
                            () = cancel.cancelled() => {
                                producer_driver.cancel();
                                let _ = pending.await;
                                return Err(MathRuntimeError::Cancelled);
                            }
                        };
                        let capture_service = service.clone();
                        let capture_request = request.clone();
                        service.job_scoped(1, WITHIN_WORKSPACE, cancel.clone(), Some(deadline), move |flag| {
                            let scope = pse_kernels::ExecutionScope::new(flag, Some(deadline));
                            portable::capture_execution(&capture_service, capture_request, &artifacts, &scope)
                                .map_err(portable::runtime_error)
                        }).await?
                    };
                    let finalize = service.clone();
                    let (execution, assembly, owner) = service.job_scoped(1, WITHIN_WORKSPACE,
                        cancel.clone(), Some(deadline), move |flag| {
                            let scope = pse_kernels::ExecutionScope::new(flag, Some(deadline));
                            let mut execution = execution;
                            match request.profile().optimization.evaluator {
                                pse_math::library::Evaluator::Interpreted => {},
                                evaluator @ (pse_math::library::Evaluator::CompiledScalar { module_bytes }
                                    | pse_math::library::Evaluator::CompiledX4 { module_bytes }) => {
                                    #[cfg(feature = "compiled-evaluator")]
                                    {
                                        let mut templates = Vec::with_capacity(execution.templates.len());
                                        for template in &execution.templates {
                                            let admit = |bytes| finalize.reserve("math:numerical-native-realization", bytes)
                                                .map(|owner| -> Arc<dyn pse_math::AllocationOwner> { owner });
                                            let compiled = if matches!(evaluator, pse_math::library::Evaluator::CompiledX4 { .. }) {
                                                template.compile_x4(module_bytes, &scope, admit)
                                            } else {
                                                template.compile_scalar(module_bytes, &scope, admit)
                                            }
                                                .map_err(transfer_failure)?;
                                            templates.push(Arc::new(compiled));
                                        }
                                        execution.templates = templates;
                                    }
                                    #[cfg(not(feature = "compiled-evaluator"))]
                                    { let _ = evaluator;
                                    return Err(MathRuntimeError::Math(pse_math::MathError::Contract(format!(
                                        "native numerical profile ({module_bytes} bytes) requires compiled-evaluator"))));
                                    }
                                },
                            }
                            let request_owner = finalize.reserve("math:numerical-original-request",
                                request.retained_bytes().checked_mul(2)
                                    .ok_or(MathRuntimeError::Limit("numerical request ownership"))?)?;
                            let request = request.with_owner(request_owner.clone());
                            execution.request = request.clone();
                            let bound = request.binding().assembly_allocation_bytes(&execution.templates)?
                                .checked_add(size_of::<NumericalArtifact>() + 8 * size_of::<usize>())
                                .ok_or(MathRuntimeError::Limit("numerical assembly shell"))?;
                            let owner = finalize.reserve("math:numerical-artifact-binding", bound)?;
                            let assembly = request.binding().assemble(execution.templates.clone())?;
                            let retained = assembly.retained_bytes()
                                .checked_add(size_of::<NumericalArtifact>() + 8 * size_of::<usize>())
                                .ok_or(MathRuntimeError::Limit("numerical retained assembly"))?;
                            let owner = owner.partition(&[retained])
                                .map_err(|_| MathRuntimeError::Limit("numerical assembly exceeds construction bound"))?
                                .into_iter().next().ok_or(MathRuntimeError::Limit("numerical assembly lease absent"))?;
                            let assembly = assembly.with_owner(Arc::new((request_owner, owner.clone())));
                            scope.check().map_err(pse_math::MathError::Scope)?;
                            Ok((execution, assembly, owner))
                        }).await?;
                    if cancel.is_cancelled() { return Err(MathRuntimeError::Cancelled); }
                    let artifact = Arc::new(NumericalArtifact { execution, assembly, _owner:owner });
                    service.pressure.publish_artifact(epoch, &key, Value::Numerical(artifact.clone()));
                    Ok(artifact)
                }))
        }, wait).await.map_err(|cause| match cause {
            FlightError::Load(error) => MathRuntimeError::Shared(error),
            FlightError::Capacity => MathRuntimeError::Limit("numerical artifact flights"),
            FlightError::Retiring => MathRuntimeError::Retiring,
            FlightError::Panicked => MathRuntimeError::Panic("numerical artifact producer".into()),
        })?;
        let _ = self.preparation_driver(&driver)?;
        loaded.ok_or(MathRuntimeError::Cancelled)
    }
}
