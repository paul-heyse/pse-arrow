// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Host requests compose the existing scientific owners under runtime-owned admission.
use super::*;
use crate::{
    CancelSource,
    math::{settings::SolveSettings, solves::SolverProfile},
};
use pse_model::generated::identities::DeclarationId;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

impl ModelingPackage {
    /// Run admitted conformance controls with one runtime-owned policy composition.
    /// # Errors
    /// Invalid controls, native settings and original conformance failures propagate.
    pub async fn conform_request(
        &self,
        services: &ExecutionServices,
        preparation: &PreparationSettings,
        settings: SolveSettings,
        controls: ConformanceControls,
        cancel: &CancelSource,
    ) -> Result<ModelingConformanceReport, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        let fixtures = controls.selection()?;
        let ConformanceControls {
            maximum_fixtures,
            maximum_checks,
            derivative_cells,
            derivative_step,
            derivative_tolerance,
            diagnostics,
            ..
        } = controls;
        self.conform(
            services,
            ModelingConformancePolicy {
                compiler: preparation.compiler,
                solver: settings.profile()?,
                numerical: Default::default(),
                limits: preparation.limits,
                derivatives: pse_backend_native::derivative_diagnostics::Policy {
                    perturbation: derivative_step,
                    relative_tolerance: derivative_tolerance,
                    maximum_cells: derivative_cells,
                },
                maximum_fixtures,
                maximum_checks,
                fixtures,
                diagnostics,
            },
            cancel,
        )
        .await
    }
    /// Admit an authored procedure with the shared preparation and settings declarations.
    /// # Errors
    /// Original settings, source, route and preparation refusals propagate unchanged.
    pub async fn admit_declared_request(
        &self,
        services: &PreparationServices,
        case: DeclarationId,
        preparation: &PreparationSettings,
        settings: SolveSettings,
        cancel: &CancelSource,
    ) -> Result<DeclaredExecution, WorkflowError> {
        self.check_service_bindings(services)?;
        self.declared_execution(
            services,
            case,
            preparation.compiler,
            settings.profile()?,
            Default::default(),
            preparation.limits,
            cancel,
        )
        .await
    }
    /// Prepare and run the admitted simulation under the original caller owner.
    /// # Errors
    /// Shared operation admission and the original physical integration can refuse.
    pub async fn simulate_case_request(
        &self,
        services: &ExecutionServices,
        case: DeclarationId,
        preparation: &PreparationSettings,
        request: PhysicalSimulationRequest,
        cancel: &CancelSource,
    ) -> Result<ModelingTrajectory, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        let prepared = self
            .prepare_operation(
                services.preparation(),
                &OperationSource::of(self),
                preparation,
                RuntimeOperationRequest::Study {
                    operation: Box::new(OperationRequest::Simulation(SimulationOperation {
                        case,
                        settings: Some(request),
                    })),
                    output: StudyOutputDemand::Full {},
                    admitted_horizon: None,
                },
                None,
                cancel,
            )
            .await?;
        let PreparedOperation::Study(PreparedStudyOperation::Simulation(prepared)) = prepared
        else {
            return Err(contract("simulation operation required"));
        };
        prepared.run(services, cancel).await
    }
    /// Execute the actual authored solve or initialization procedure.
    /// # Errors
    /// Admission and the selected procedure retain their original typed failures.
    pub async fn execute_case_request(
        &self,
        services: &ExecutionServices,
        case: DeclarationId,
        preparation: &PreparationSettings,
        settings: SolveSettings,
        cancel: &CancelSource,
    ) -> Result<ModelingResult, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        let execution = self
            .admit_declared_request(services.preparation(), case, preparation, settings, cancel)
            .await?;
        self.execute_declared(services, &execution, cancel).await
    }
    /// Execute typed transient values using the existing bound scientific owner.
    /// # Errors
    /// Invalid physical inputs, admission and original solve failures propagate.
    pub async fn execute_bound_request(
        &self,
        services: &ExecutionServices,
        case: DeclarationId,
        overlay: &PointOverlay,
        preparation: &PreparationSettings,
        settings: SolveSettings,
        cancel: &CancelSource,
    ) -> Result<ModelingResult, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        self.solve_bound(
            services,
            case,
            overlay,
            preparation.compiler,
            settings.profile()?,
            Default::default(),
            preparation.limits,
            cancel,
        )
        .await
    }
    /// Inspect an admitted procedure without dispatching it.
    /// # Errors
    /// Original contextual admission failures propagate.
    pub async fn inspect_case_request(
        &self,
        services: &PreparationServices,
        request: &ModelingInspectionRequest,
        cancel: &CancelSource,
    ) -> Result<ModelingInspection, WorkflowError> {
        self.check_service_bindings(services)?;
        if request.limit == 0 || request.limit > 128 {
            return Err(contract("inspection page limit must be between one and128"));
        }
        modeling::documents::bound_observation(
            self.admit_declared_request(
                services,
                request.case,
                &request.preparation,
                request.settings.clone(),
                cancel,
            )
            .await?
            .inspection(
                OperationSource::of(self),
                request.preparation.clone(),
                request.offset,
                request.limit,
            ),
        )
    }
    /// Prepare authored topology using the same declared admission as other hosts.
    /// # Errors
    /// Admission and the existing flow owner's refusals propagate.
    pub async fn prepare_flow_request(
        &self,
        services: &PreparationServices,
        case: DeclarationId,
        preparation: &PreparationSettings,
        settings: SolveSettings,
        selection: crate::math::flows::ModelingFlowSelection,
        cancel: &CancelSource,
    ) -> Result<crate::math::flows::PreparedFlow, WorkflowError> {
        self.check_service_bindings(services)?;
        let execution = self
            .admit_declared_request(services, case, preparation, settings, cancel)
            .await?;
        self.prepare_flow(services, &execution.analysis, selection, cancel)
            .await
    }
    /// Prepare the existing fresh-session map strategy with its actual selected profile.
    /// # Errors
    /// Declared admission and the recycle owner's route/control refusals propagate.
    #[cfg(feature = "solver-kinsol")]
    #[allow(
        clippy::too_many_arguments,
        reason = "original request coordinates and controls remain explicit alongside the receiving effect capability"
    )]
    pub async fn prepare_recycle_request(
        &self,
        services: &PreparationServices,
        case: DeclarationId,
        preparation: &PreparationSettings,
        settings: SolveSettings,
        selection: crate::math::flows::ModelingFlowSelection,
        request: RecycleRequest,
        cancel: &CancelSource,
    ) -> Result<PreparedRecycle, WorkflowError> {
        self.check_service_bindings(services)?;
        let execution = self
            .admit_declared_request(services, case, preparation, settings, cancel)
            .await?;
        self.prepare_recycle(services, &execution.analysis, selection, request, cancel)
            .await
    }
    /// Prepare explicit continuation stages without replacing their original profile.
    /// # Errors
    /// Declared admission and original initialization refusals propagate.
    #[cfg(feature = "solver-kinsol")]
    pub async fn prepare_initialization_request(
        &self,
        services: &PreparationServices,
        case: DeclarationId,
        preparation: &PreparationSettings,
        settings: SolveSettings,
        stages: InitializationStages,
        cancel: &CancelSource,
    ) -> Result<PreparedInitializationStrategy, WorkflowError> {
        self.check_service_bindings(services)?;
        let execution = self
            .admit_declared_request(services, case, preparation, settings, cancel)
            .await?;
        self.prepare_block_initialization_physical(services, &execution.analysis, &stages, cancel)
            .await
    }
    /// Execute the admitted authored initialization and explicit overrides.
    /// # Errors
    /// Original admission, overrides and initialization failures propagate.
    pub async fn initialize_case_request(
        &self,
        services: &ExecutionServices,
        case: DeclarationId,
        preparation: &PreparationSettings,
        settings: SolveSettings,
        overrides: InitializationOverrides,
        cancel: &CancelSource,
    ) -> Result<ModelingInitializationReport, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        let execution = self
            .admit_declared_request(services.preparation(), case, preparation, settings, cancel)
            .await?;
        self.initialize_declared(services, &execution, overrides, cancel)
            .await
    }
    /// Run the existing nonlinear explanation after contextual declared admission.
    /// # Errors
    /// Admission and the explanation owner's policy failures propagate.
    pub async fn explain_case_request(
        &self,
        services: &ExecutionServices,
        case: DeclarationId,
        preparation: &PreparationSettings,
        settings: SolveSettings,
        policy: ModelingNonlinearPolicy,
        cancel: &CancelSource,
    ) -> Result<ModelingNonlinearExplanation, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        let execution = self
            .admit_declared_request(services.preparation(), case, preparation, settings, cancel)
            .await?;
        self.explain_nonlinear(services, &execution.analysis, policy, cancel)
            .await
    }
    async fn prepare_diagnostic_request(
        &self,
        services: &PreparationServices,
        case: DeclarationId,
        preparation: &PreparationSettings,
        profile: SolverProfile,
        operation: &DiagnosticOperation,
    ) -> Result<
        (
            ModelingDiagnosticPreparation,
            pse_compiler::workspace::Profile,
        ),
        WorkflowError,
    > {
        let execution = operation
            .within(self.declared_execution(
                services,
                case,
                preparation.compiler,
                profile,
                Default::default(),
                preparation.limits,
                operation.driver(),
            ))
            .await?;
        let prepared = operation
            .within(self.prepare_declared_diagnostics(services, &execution, operation.driver()))
            .await?;
        Ok((prepared, execution.analysis.compiler))
    }
    /// Diagnose original case values under one inherited caller clock.
    /// # Errors
    /// Original admission, diagnostic policy and deadline failures propagate.
    #[allow(
        clippy::too_many_arguments,
        reason = "original request coordinates and controls remain explicit alongside the receiving effect capability"
    )]
    pub async fn diagnose_case_request(
        &self,
        services: &ExecutionServices,
        case: DeclarationId,
        preparation: &PreparationSettings,
        settings: SolveSettings,
        policy: ModelingDiagnosticPolicy,
        origin: Option<DiagnosticOrigin>,
        cancel: &CancelSource,
    ) -> Result<ModelingDiagnostics, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        let profile = settings.profile()?;
        let operation = DiagnosticOperation::new(cancel, profile.controls.time_limit, origin)?;
        let (prepared, compiler) = self
            .prepare_diagnostic_request(
                services.preparation(),
                case,
                preparation,
                profile,
                &operation,
            )
            .await?;
        let values = prepared.model.values.clone();
        self.diagnose_case(services, prepared, values, policy, compiler, &operation)
            .await
    }
    /// Diagnose supplied scalar points without inventing starts or renewing clocks.
    /// # Errors
    /// Invalid allowances, sample identity, contextual admission or diagnostics refuse.
    #[expect(
        clippy::too_many_arguments,
        reason = "source, preparation, settings, sample values, diagnostic policy, campaign controls and original cancellation are independent request meanings"
    )]
    pub async fn diagnose_samples_request(
        &self,
        services: &ExecutionServices,
        case: DeclarationId,
        preparation: &PreparationSettings,
        settings: SolveSettings,
        samples: DiagnosticSamples,
        policy: ModelingDiagnosticPolicy,
        controls: DiagnosticSamplesControls,
        origin: Option<DiagnosticOrigin>,
        cancel: &CancelSource,
    ) -> Result<ModelingDiagnosticSamples, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        let profile = settings.profile()?;
        let allowance = Duration::try_from_secs_f64(controls.time_limit)
            .map_err(|_| contract("diagnostic time limit must be finite and positive"))?;
        let operation =
            DiagnosticOperation::new(cancel, profile.controls.time_limit.min(allowance), origin)?;
        let (prepared, compiler) = self
            .prepare_diagnostic_request(
                services.preparation(),
                case,
                preparation,
                profile,
                &operation,
            )
            .await?;
        let structure = prepared.model.case.compiled().plan.structure();
        let ports = structure
            .variables()
            .iter()
            .map(|variable| variable.port.clone())
            .chain(structure.parameters().iter().cloned())
            .collect::<Vec<_>>();
        let values = samples.admit(&self.quantities, &ports)?;
        let values = values
            .into_iter()
            .map(|(name, overrides)| {
                let mut values = prepared.model.values.clone();
                values.scalars.extend(overrides);
                (name, values)
            })
            .collect();
        self.diagnose_samples(
            services,
            prepared,
            values,
            policy,
            compiler,
            controls.maximum_samples,
            &operation,
        )
        .await
    }
    /// Bind diagnostic coordinates and retain the native analysis through its owner.
    /// # Errors
    /// Original contextual admission, coordinate maps and native diagnostics refuse.
    #[cfg(feature = "solver-highs")]
    #[allow(
        clippy::too_many_arguments,
        reason = "original request coordinates and controls remain explicit alongside the receiving effect capability"
    )]
    pub async fn diagnose_linear_request(
        &self,
        services: &ExecutionServices,
        case: DeclarationId,
        preparation: &PreparationSettings,
        settings: SolveSettings,
        controls: LinearDiagnosticControls,
        origin: Option<DiagnosticOrigin>,
        cancel: &CancelSource,
    ) -> Result<ModelingNativeAnalysis, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        let profile = settings.profile()?;
        let operation = DiagnosticOperation::new(cancel, profile.controls.time_limit, origin)?;
        let native_controls = profile.controls.clone();
        let (prepared, _) = self
            .prepare_diagnostic_request(
                services.preparation(),
                case,
                preparation,
                profile,
                &operation,
            )
            .await?;
        let plan = &prepared.model.case.compiled().plan;
        let rows = plan
            .structure()
            .rows()
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>();
        let maximum_entries = controls.maximum_entries;
        let request = controls.request(plan.columns(), &rows)?;
        let exported = self
            .diagnose_linear(
                services,
                prepared,
                request,
                native_controls,
                maximum_entries,
                &operation,
            )
            .await?
            .into_export()?;
        operation.check()?;
        Ok(exported)
    }
    /// Retain the existing scaled-Jacobian search and its original diagnostic clock.
    /// # Errors
    /// Contextual admission, policy and native diagnostic failures propagate.
    #[cfg(feature = "solver-highs")]
    #[allow(
        clippy::too_many_arguments,
        reason = "original request coordinates and controls remain explicit alongside the receiving effect capability"
    )]
    pub async fn diagnose_jacobian_request(
        &self,
        services: &ExecutionServices,
        case: DeclarationId,
        preparation: &PreparationSettings,
        settings: SolveSettings,
        controls: JacobianDiagnosticControls,
        origin: Option<DiagnosticOrigin>,
        cancel: &CancelSource,
    ) -> Result<ModelingNativeAnalysis, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        let profile = settings.profile()?;
        let operation = DiagnosticOperation::new(cancel, profile.controls.time_limit, origin)?;
        let native_controls = profile.controls.clone();
        let (prepared, _) = self
            .prepare_diagnostic_request(
                services.preparation(),
                case,
                preparation,
                profile,
                &operation,
            )
            .await?;
        let values = prepared.model.values.clone();
        let exported = self
            .diagnose_jacobian_optimization(
                services,
                prepared,
                values,
                controls.policy(),
                native_controls,
                &operation,
            )
            .await?
            .into_export()?;
        operation.check()?;
        Ok(exported)
    }
    /// Inspect prepared source artifacts under one finite clock including admission.
    /// # Errors
    /// Nonpositive allowances, original admission and source-map allocation refuse.
    #[allow(
        clippy::too_many_arguments,
        reason = "original request coordinates and controls remain explicit alongside the receiving effect capability"
    )]
    pub async fn source_map_request(
        &self,
        services: &PreparationServices,
        case: DeclarationId,
        preparation: &PreparationSettings,
        settings: SolveSettings,
        maximum_bytes: usize,
        time_limit: f64,
        cancel: &CancelSource,
    ) -> Result<ModelingSourceMap, WorkflowError> {
        self.check_service_bindings(services)?;
        let time_limit = Duration::try_from_secs_f64(time_limit)
            .map_err(|_| contract("source map time limit must be finite and positive"))?;
        if maximum_bytes == 0 || time_limit.is_zero() {
            return Err(contract(
                "source map requires a positive finite allocation and clock",
            ));
        }
        let deadline = Instant::now()
            .checked_add(time_limit)
            .ok_or_else(|| deadline_limit("source map deadline extent"))?;
        let cancel = cancel.with_deadline(Some(deadline));
        let execution = self
            .admit_declared_request(services, case, preparation, settings, &cancel)
            .await?;
        let prepared = self
            .prepare_declared_diagnostics(services, &execution, &cancel)
            .await?;
        self.modeling_diagnostic_source_map(services, &prepared, maximum_bytes, time_limit, &cancel)
            .await
    }
}

impl Runtime {
    /// Admit one finite prepared algebraic sequence through the shared joined owner.
    /// # Errors
    /// Non-algebraic operations and original sequence admission refuse before dispatch.
    pub async fn start_prepared_sequence(
        &self,
        operations: Vec<PreparedOperation>,
        controls: RunControls,
        cancel: &CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        let steps = operations
            .into_iter()
            .map(|operation| match operation {
                PreparedOperation::Study(PreparedStudyOperation::DeclaredCase(prepared)) => {
                    Ok(*prepared)
                }
                _ => Err(contract(
                    "finite solve sequences require authored algebraic cases",
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.execution_services()
            .start_modeling(steps, controls.continue_independent, cancel)
            .await
    }
    /// Admit encoded solve settings before preparing the existing non-durable cone strategy.
    /// # Errors
    /// Invalid settings and the original cone admission failures propagate.
    pub async fn prepare_conic_request(
        &self,
        request: ConicRequest,
        physical: &PhysicalContext,
        settings: SolveSettings,
    ) -> Result<PreparedConic, WorkflowError> {
        self.prepare_conic(request, physical, settings.profile()?)
            .await
    }
}
impl crate::math::MathService {
    /// Admit shared settings before the existing non-durable tear-selection operation.
    /// # Errors
    /// Settings and the original tear selector retain their typed failures.
    pub fn select_tears_request(
        self: &Arc<Self>,
        flow: crate::math::flows::PreparedFlow,
        method: crate::math::flows::TearMethod,
        settings: SolveSettings,
    ) -> Result<crate::math::solves::SolveHandle<crate::math::flows::TearResult>, MathRuntimeError>
    {
        self.select_tears(flow, method, settings.profile()?.controls)
    }
}

/// Admit pure conformance selection through the same controls used by every host.
/// # Errors
/// Invalid selection, bounds and the original document admission failures propagate.
pub async fn conform_pure_request(
    documents: Vec<BTreeMap<String, Vec<u8>>>,
    physical_documents: BTreeMap<String, Vec<u8>>,
    budget: crate::ResourceBudget,
    controls: PureConformanceControls,
    preparation: PreparationSettings,
    cancel: &CancelSource,
) -> Result<ModelingConformanceReport, WorkflowError> {
    let selection = controls.selection()?;
    conform_pure_documents(
        documents,
        physical_documents,
        budget,
        selection,
        controls.maximum_fixtures,
        controls.maximum_checks,
        preparation,
        cancel,
    )
    .await
}
