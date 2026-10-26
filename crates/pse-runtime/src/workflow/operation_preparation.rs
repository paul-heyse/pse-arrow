// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared operation admission; hosts supply transport and their original cancellation owner.
use super::{
    AdmittedBinding, AdmittedHorizonValues, DeclaredExecution, DeclaredProcedure,
    ExecutionServices, ModelingPackage, ModelingSolvePreparation, OperationRequest,
    OperationSource, PreparationServices, PreparationSettings, PreparedStudyOperation, RunHandle,
    WorkflowError, contract,
};
use crate::{CancelSource, math::settings::SolveSettings};
use pse_model::generated::{enums::ModelingAnalysisRoute, identities::DeclarationId};

/// Host intent. Prepared shooting handles have no serialized study representation.
#[derive(Clone, Debug)]
pub enum RuntimeOperationRequest {
    /// A declaration whose route is resolved by runtime admission.
    DeclaredCase {
        /// Selected authored case.
        case: DeclarationId,
        /// Complete native controls.
        settings: Box<SolveSettings>,
    },
    /// The unchanged study-admitted operation vocabulary and its contextual horizon values.
    Study {
        /// Scientific request, independent of occurrence identity and output selection.
        operation: Box<OperationRequest>,
        /// Explicit requested result retention, independent of occurrence identity.
        output: super::StudyOutputDemand,
        /// Already admitted physical horizon assignments, when present.
        admitted_horizon: Option<AdmittedHorizonValues>,
    },
    /// Existing host-only shooting capability.
    #[cfg(feature = "solver-diffsol")]
    Shooting {
        /// Admitted immutable problem.
        problem: std::sync::Arc<super::ShootingProblem>,
        /// Explicit original coordinates, or the problem's admitted defaults.
        initial: Option<Vec<f64>>,
    },
}
fn projection_refusal(reason: &str) -> WorkflowError {
    let mut diagnostic = pse_model::diagnostic::BoundaryDiagnostic::new(
        pse_model::diagnostic::BoundaryClass::Unsupported,
        pse_diagnostics::DiagnosticStage::StudyAdmission,
        [],
        pse_diagnostics::DiagnosticRule::StudyOperationUnsupported,
    );
    diagnostic.observations.insert(
        "reason".into(),
        pse_model::diagnostic::Observation::Text(reason.into()),
    );
    diagnostic.into()
}
impl RuntimeOperationRequest {
    /// Project only the admitted persisted vocabulary, before any durable effect.
    /// # Errors
    /// Host-only work and an unresolved declared route have no study descriptor.
    pub fn into_study(self) -> Result<OperationRequest, WorkflowError> {
        match self {
            Self::Study { operation, .. } => Ok(*operation),
            Self::DeclaredCase { .. } => Err(projection_refusal(
                "study projection requires the runtime-admitted declared route",
            )),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { .. } => Err(projection_refusal(
                "shooting is a host capability, outside StudyOperation Version<8>",
            )),
        }
    }
}

/// One prepared host operation, composed from the shared study subset and host-only work.
#[derive(Clone, Debug)]
pub enum PreparedOperation {
    /// Shared declared-case, simulation, fitting or horizon owner.
    Study(PreparedStudyOperation),
    /// A shooting problem retains its exact requested initial vector.
    #[cfg(feature = "solver-diffsol")]
    Shooting {
        /// Existing native problem owner.
        problem: std::sync::Arc<super::ShootingProblem>,
        /// Original coordinates selected before dispatch.
        initial: Option<Vec<f64>>,
    },
}
impl PreparedOperation {
    /// Original scientific input identity, established by its actual owner.
    /// # Errors
    /// Horizon identity requires the contextual horizon admission.
    pub fn identity(&self) -> Result<pse_ids::ContentHash, WorkflowError> {
        match self {
            Self::Study(PreparedStudyOperation::DeclaredCase(prepared)) => prepared
                .solve
                .request_identity()
                .map(|key| key.as_id())
                .map_err(crate::math::MathRuntimeError::from)
                .map_err(Into::into),
            Self::Study(PreparedStudyOperation::Simulation(prepared)) => Ok(prepared.identity()),
            Self::Study(PreparedStudyOperation::Fit(prepared)) => Ok(prepared.identity()),
            Self::Study(PreparedStudyOperation::Horizon(_)) => Err(contract(
                "horizon identity requires its contextual admission",
            )),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { problem, initial } => {
                Ok(problem.request_identity(initial.as_deref()).as_id())
            }
        }
    }
    /// Keep study admission a typed subset of actual prepared capabilities.
    /// # Errors
    /// Shooting cannot enter the persisted Version<8> operation set.
    pub fn into_study(self) -> Result<PreparedStudyOperation, WorkflowError> {
        match self {
            Self::Study(prepared) => Ok(prepared),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { .. } => Err(projection_refusal(
                "shooting has no StudyOperation Version<8> projection",
            )),
        }
    }
    /// Start through the same joined supervisor used by durable workers.
    /// # Errors
    /// Original operation admission and execution refusals retain their causes.
    pub async fn start(
        &self,
        services: &ExecutionServices,
        cancel: &CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        match self {
            Self::Study(prepared) => prepared.start(services, cancel).await,
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { problem, initial } => {
                problem.start_replay_with_cancel(services, initial.clone(), cancel)
            }
        }
    }
}

impl ModelingPackage {
    /// Prepare one host request under explicit source/context, controls and admitted values.
    /// # Errors
    /// Invalid context, unsupported bindings or the selected scientific owner's refusal.
    pub async fn prepare_operation(
        &self,
        services: &PreparationServices,
        source: &OperationSource,
        preparation: &PreparationSettings,
        request: RuntimeOperationRequest,
        binding: Option<&AdmittedBinding>,
        cancel: &CancelSource,
    ) -> Result<PreparedOperation, WorkflowError> {
        self.check_service_bindings(services)?;
        source.check(self)?;
        let driver = services.math().preparation_driver(cancel)?;
        match request {
            RuntimeOperationRequest::DeclaredCase { case, settings } => Ok(
                PreparedOperation::Study(PreparedStudyOperation::DeclaredCase(Box::new(
                    self.prepare_case_operation(
                        services,
                        case,
                        &settings,
                        preparation,
                        None,
                        binding,
                        &driver,
                    )
                    .await?,
                ))),
            ),
            RuntimeOperationRequest::Study {
                operation,
                output,
                admitted_horizon,
            } => {
                let admitted_horizon = match admitted_horizon {
                    Some(values) => Some(values),
                    None => {
                        super::study_operations::admit_horizon_values(
                            services,
                            self,
                            source,
                            preparation,
                            &operation,
                            &driver,
                        )
                        .await?
                    }
                };
                super::study_operations::OperationPreparation {
                    source,
                    preparation,
                    operation: &operation,
                    output: &output,
                    admitted_horizon: admitted_horizon.as_ref(),
                    binding,
                }
                .prepare(services, self, &driver)
                .await
                .map(PreparedOperation::Study)
            }
            #[cfg(feature = "solver-diffsol")]
            RuntimeOperationRequest::Shooting { problem, initial } => {
                OperationSource::of(&problem.simulation.source).check(self)?;
                if binding.is_some_and(|binding| !binding.entries.is_empty()) {
                    return Err(contract("shooting request has unsupported bindings"));
                }
                Ok(PreparedOperation::Shooting { problem, initial })
            }
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "original request coordinates and controls remain explicit alongside the receiving effect capability"
    )]
    pub(in crate::workflow) async fn prepare_case_operation(
        &self,
        services: &PreparationServices,
        case: DeclarationId,
        settings: &SolveSettings,
        preparation: &PreparationSettings,
        expected_route: Option<ModelingAnalysisRoute>,
        binding: Option<&AdmittedBinding>,
        cancel: &CancelSource,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        self.check_service_bindings(services)?;
        #[cfg(test)]
        super::study_operations::count_declared_admission();
        let execution = self
            .admit_declared_request(services, case, preparation, settings.clone(), cancel)
            .await?;
        if !matches!(execution.procedure, DeclaredProcedure::Solve)
            || expected_route.is_some_and(|route| execution.route != route)
        {
            return Err(pse_model::diagnostic::BoundaryDiagnostic::new(
                pse_model::diagnostic::BoundaryClass::Unsupported,
                pse_diagnostics::DiagnosticStage::StudyAdmission,
                [case.as_id()],
                pse_diagnostics::DiagnosticRule::StudyOperationUnsupported,
            )
            .into());
        }
        #[cfg(test)]
        super::study_operations::observe_declared_admission();
        self.prepare_operation_execution(services, execution, binding, None, cancel)
            .await
    }

    /// Existing lane admissions and immediate requests share the same binding lowering.
    pub(in crate::workflow) async fn prepare_operation_execution(
        &self,
        services: &PreparationServices,
        mut execution: DeclaredExecution,
        binding: Option<&AdmittedBinding>,
        current_view: Option<&mut super::study_operations::CurrentNumericalView>,
        cancel: &CancelSource,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        self.check_service_bindings(services)?;
        let Some(binding) = binding else {
            return self.prepare_declared(services, &execution, cancel).await;
        };
        execution.prepared_model(self)?;
        self.validate_binding(execution.model.compiled(), binding)?;
        let bound_source = self.retain_bound_source(services, binding)?;
        execution.analysis.case.members = binding.values();
        let analysis = execution.analysis;
        let resolution = self
            .resolve_prepared_case_with_current_view(
                services,
                execution.model,
                analysis.case,
                analysis.order,
                analysis.compiler,
                analysis.solver,
                analysis.numerical,
                super::modeling::cases::CaseOverrides::default(),
                false,
                current_view,
                cancel,
            )
            .await?;
        let mut prepared = self.finish_case(services, resolution, cancel).await?;
        prepared.bound_source = Some(bound_source);
        Ok(prepared)
    }
}
