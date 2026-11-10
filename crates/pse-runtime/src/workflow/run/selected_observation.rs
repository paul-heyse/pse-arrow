// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure observations retain the same original run/attempt and durable completion owner.
use super::*;
use crate::workflow::{PreparedSelectedObservation, CompletedSelectedObservation};

impl PreparedSelectedObservation {
    /// Start an ordinary selected physical observation under the joined run lifecycle.
    pub fn start(&self, services: &ExecutionServices, cancel: &crate::CancelSource) -> Result<RunHandle, WorkflowError> {
        RunSupervisor::launch_unclaimed(services, None, 64, Some(cancel), || self.source.check_service_bindings(services.preparation()).map_err(WorkflowError::from), |supervisor, ()| Ok(self.start_owned(supervisor)))
    }
    pub(crate) async fn start_attempt(&self, services: &ExecutionServices, attempt: DurableAttempt, cancel: &crate::CancelSource) -> Result<RunHandle, WorkflowError> {
        RunSupervisor::launch_claimed(services, None, 64, cancel, attempt, || self.source.check_service_bindings(services.preparation()).map_err(WorkflowError::from), |supervisor, ()| Ok(self.start_owned(supervisor))).await
    }
    fn start_owned(&self, supervisor: RunSupervisor) -> RunHandle {
        start_selected_owned(supervisor, Box::new(self.clone()), None)
    }
}
impl CompletedSelectedObservation {
    /// Publish this original batch occurrence without recomputing its mathematics.
    pub fn start(self, services: &ExecutionServices, cancel: &crate::CancelSource) -> Result<RunHandle, WorkflowError> {
        let source = self.request.source.clone();
        RunSupervisor::launch_unclaimed(services, None, 64, Some(cancel), || source.check_service_bindings(services.preparation()).map_err(WorkflowError::from), |supervisor, ()| Ok(start_selected_owned(supervisor, self.request, Some((self.result, self._population)))))
    }
}
type PopulationCompletion = (Result<Box<crate::workflow::SelectedObservationValues>, Arc<WorkflowError>>, Arc<pse_columnar::Leased<pse_math::assembly::CaseBatchOutcome>>);
fn start_selected_owned(supervisor: RunSupervisor, request: Box<PreparedSelectedObservation>, completed: Option<PopulationCompletion>) -> RunHandle {
        supervisor.spawn(move |mut supervisor| async move {
            let started = async {
                if let Some(attempt) = supervisor.durable.as_mut() {
                    attempt.register(&supervisor.services.source_services(), supervisor.run_id, &RunRequest::SelectedObservation(&request)).await?;
                    let checks = supervisor.checks.clone();
                    attempt.start(Arc::new(move || checks.cancel())).await?;
                }
                match completed {
                    Some((result, _population)) => result.map_err(WorkflowError::Shared),
                    None => request.evaluate(supervisor.services.preparation(), &supervisor.checks).await.map(Box::new),
                }
            }.await;
            let result = started.map_err(Arc::new);
            RunResult::joined(supervisor.run_id, supervisor.services, RunOutcome::SelectedObservation { request, result }, None)
                .finished(supervisor.durable, supervisor.checks.token().is_cancelled()).await
        })
}

impl RunResult {
    pub(in crate::workflow) async fn project_selected_observations(&self, projection: &crate::workflow::result_export::Projection) -> Result<(), WorkflowError> {
        use pse_relations::generated::runtime::selected_observations::Row;
        let pool = self.services.preparation().pool();
        let validation = self.result_validation_context();
        let mut rows = crate::workflow::result_export::Rows::<Row>::new(projection, self.services.preparation().registry(), &pool, &projection.cancel, &validation).map_err(crate::workflow::relation)?;
        if let RunOutcome::SelectedObservation { request, result: Ok(result) } = &self.outcome {
            for (ordinal, (output, (&member, &canonical))) in request.operation.outputs.iter().zip(result.members.iter().zip(&result.canonical)).enumerate() {
                let plan = pse_quantity::CanonicalConversionPlan::registered(&request.source.quantities, output.quantity, output.unit).map_err(|error| crate::workflow::math(pse_math::MathError::Quantity(error)))?;
                let value = (canonical - plan.offset()) / plan.scale();
                if !value.is_finite() { return Err(contract("selected observation representation is nonfinite")); }
                rows.push(Row { run_id: self.run_id, ordinal: i64::try_from(ordinal).map_err(|_| contract("selected observation ordinal"))?, member_id: member, quantity_id: output.quantity, unit_id: output.unit, canonical, value }).await.map_err(crate::workflow::relation)?;
            }
        }
        rows.finish().await.map_err(crate::workflow::relation)
    }
}
