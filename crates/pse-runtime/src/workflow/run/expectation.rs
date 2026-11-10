// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One ordinary parent run publishes statistical products of original child studies.
use super::*;
use crate::workflow::expectation::PreparedExpectationOperation;
impl PreparedExpectationOperation {
    /// Start a source-bound parent under ordinary original run/attempt supervision.
    pub fn start(
        &self,
        services: &ExecutionServices,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        if matches!(services.durability(), Durability::Ephemeral) {
            return Err(WorkflowError::RecordingUnavailable);
        }
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
            |supervisor, ()| {
                let request = Box::new(self.clone());
                Ok(supervisor.spawn(move |mut supervisor| async move {
                    let result = async {
                        if let Some(attempt) = supervisor.durable.as_mut() {
                            attempt
                                .register(
                                    &supervisor.services.source_services(),
                                    supervisor.run_id,
                                    &RunRequest::Expectation(&request),
                                )
                                .await?;
                            let checks = supervisor.checks.clone();
                            attempt.start(Arc::new(move || checks.cancel())).await?;
                        }
                        request.evaluate(&supervisor.checks).await.map(Box::new)
                    }
                    .await
                    .map_err(Arc::new);
                    RunResult::joined(
                        supervisor.run_id,
                        supervisor.services,
                        RunOutcome::Expectation { request, result },
                        None,
                    )
                    .finished(supervisor.durable, supervisor.checks.token().is_cancelled())
                    .await
                }))
            },
        )
    }
}
