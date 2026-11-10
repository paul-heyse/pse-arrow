// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mathematical observations use the ordinary original run and publication boundary.
use super::*;
use crate::workflow::PreparedMathematicalOperation;
impl PreparedMathematicalOperation {
    /// Start the original selected analysis, retaining unsupported observations without promotion.
    pub fn start(
        &self,
        services: &ExecutionServices,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        RunSupervisor::launch_unclaimed(
            services,
            None,
            64,
            Some(cancel),
            || {
                self.source
                    .check_service_bindings(services.preparation())
                    .map_err(WorkflowError::from)
            },
            |supervisor, ()| self.start_with_supervisor(supervisor, false),
        )
    }
    fn start_with_supervisor(
        &self,
        supervisor: RunSupervisor,
        registered: bool,
    ) -> Result<RunHandle, WorkflowError> {
        let request = Box::new(self.clone());
        Ok(supervisor.spawn(move |mut supervisor| async move {
            let result = async {
                if let Some(attempt) = supervisor.durable.as_mut() {
                    if !registered {
                        attempt
                            .register(
                                &supervisor.services.source_services(),
                                supervisor.run_id,
                                &RunRequest::Mathematical(&request),
                            )
                            .await?;
                    }
                    let checks = supervisor.checks.clone();
                    attempt.start(Arc::new(move || checks.cancel())).await?;
                }
                request
                    .evaluate(supervisor.services.preparation(), &supervisor.checks)
                    .await
                    .map(Box::new)
            }
            .await
            .map_err(Arc::new);
            RunResult::joined(
                supervisor.run_id,
                supervisor.services,
                RunOutcome::Mathematical { request, result },
                None,
            )
            .finished(supervisor.durable, supervisor.checks.token().is_cancelled())
            .await
        }))
    }
    /// Start or observe exactly the caller-issued original mathematical operation.
    /// Repetition checks the complete source/request envelope before returning a
    /// retained attempt; protected readback never reruns this method.
    pub async fn start_issued(
        &self,
        services: &ExecutionServices,
        identity: IssuedRunIdentity,
        cancel: &crate::CancelSource,
    ) -> Result<IssuedRunStart, WorkflowError> {
        pse_contract::boundary_projection::validate_field(
            &identity.operation,
            "operation",
            &[pse_model::engineering::OPERATION_ID_RULE],
        )
        .map_err(|error| {
            WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
        })?;
        let activity = services.preparation().admit_activity()?;
        activity
            .scope(async {
                self.source.check_service_bindings(services.preparation())?;
                self.identity()?;
                RunSupervisor::launch_deadline(None, Some(cancel))?;
                let attempt = match services.durability() {
                    Durability::Ephemeral => None,
                    Durability::Durable(operations) => {
                        let (run, attempt_key) = identity.canonical_keys();
                        let extent =
                            pse_columnar::MemoryConsumer::new("issued-mathematics:original-header")
                                .register(&services.preparation().pool());
                        extent
                            .try_grow(
                                pse_operations::canonical::MetadataResponse::Header
                                    .allocation_bytes(),
                            )
                            .map_err(pse_engine::EngineError::from)?;
                        let previous = pse_operations::canonical::response_ownership_scope(
                            Arc::new(extent),
                            operations.store().canonical_run(&run),
                        )
                        .await?;
                        let attempt = DurableAttempt::issued(operations, identity.clone())?;
                        if previous.is_some() {
                            attempt
                                .observe_original(
                                    &services.source_services(),
                                    identity.run,
                                    &RunRequest::Mathematical(self),
                                )
                                .await?;
                            return Ok(IssuedRunStart::Retained {
                                identity,
                                run,
                                attempt: attempt_key,
                            });
                        }
                        if let Err(error) = attempt
                            .register(
                                &services.source_services(),
                                identity.run,
                                &RunRequest::Mathematical(self),
                            )
                            .await
                        {
                            return Err(RunSupervisor::abandon_claim(attempt, error).await);
                        }
                        Some(attempt)
                    }
                };
                crate::invocation::start_scope(|| {
                    let mut supervisor =
                        RunSupervisor::new(services.clone(), attempt, Some(cancel), None, 64)?;
                    supervisor.run_id = identity.run;
                    self.start_with_supervisor(supervisor, true)
                        .map(IssuedRunStart::Started)
                })()
            })
            .await
    }
}
