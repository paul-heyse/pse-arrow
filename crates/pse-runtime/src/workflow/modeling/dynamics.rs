// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Integrated time consumes generated coordinates and the existing function oracle.
#[path = "dynamics_checks.rs"]
pub(in crate::workflow) mod checks;
#[path = "dynamics_events.rs"]
mod events;
#[path = "trajectory.rs"]
mod trajectory;
use super::*;
use crate::workflow::dynamics::{
    CoordinateBinding, DynamicCoordinates, DynamicMode, DynamicWorker, FunctionProgram,
    GuardProgram, RangeCheck, RangeValue,
};
pub use events::{ModelingDynamicEvent, ModelingDynamicMode};
use pse_backend_native::{
    ProblemError,
    dynamics::{self as native, Function},
};
use pse_compiler::workspace::{ModelingCaseBindings, ModelingHint, ModelingOutput, Profile};
use pse_ids::{ContentHash, FramedHasher};
use pse_kernels::DerivativeOrder;
use pse_math::binding::CaseValues;
use pse_model::generated::enums::{NumericalSource, NumericalTarget};
use std::{collections::BTreeSet, sync::Arc};

/// Immutable generated simulation. No authored state/RHS declaration is introduced.
#[derive(Clone, Debug)]
pub struct ModelingSimulation {
    pub(in crate::workflow) instance: InstanceId,
    quantities: Arc<pse_quantity::QuantityRegistry>,
    modes: Vec<SimulationMode>,
    contract: native::Contract,
    profile: native::Profile,
    pub(in crate::workflow) runtime: Runtime,
    pub(in crate::workflow) source: ModelingPackage,
    programs: Arc<[FunctionProgram]>,
    coordinates: DynamicCoordinates,
    pub(in crate::workflow) parameters: Vec<f64>,
    key: ContentHash,
    pub(in crate::workflow) bytes: usize,
}
#[derive(Clone, Debug)]
struct SimulationMode {
    name: String,
    model: ModelingPreparation,
    numerics: Arc<pse_model::numerics::ResolvedNumericalPolicy>,
    context: DynamicMode,
    check_program: Option<Arc<crate::math::ExecutableCase>>,
    terminal_check_program: Option<Arc<crate::math::ExecutableCase>>,
    sample_scope: Option<results::AssessmentScope>,
}
/// The native trajectory and all partial outcomes retain their resource owner.
#[derive(Clone, Debug)]
pub struct ModelingTrajectory {
    pub run_id: SemanticId,
    pub report: Arc<native::Report>,
    pub prepared: ModelingSimulation,
    /// Physical obligations evaluated at the explicitly requested sample times.
    pub checks: Vec<ModelingCheck>,
    /// Whole-domain integral reports exist only after completing their full domain.
    pub reports: Vec<ModelingReport>,
    pub checks_complete: bool,
    pub accepted: bool,
    pub validation_error: Option<pse_model::diagnostic::BoundaryDiagnostic>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl ModelingTrajectory {
    /// Native trajectory decision owned by `pse-backend-native`.
    pub(crate) fn candidate_use(&self) -> crate::workflow::numerics::CandidateDecision {
        crate::workflow::numerics::trajectory_use(&self.report)
    }
    pub(super) fn sample_context(
        &self,
        sample: &native::Sample,
    ) -> Result<
        (
            ModelingPreparation,
            CaseValues,
            BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        ),
        WorkflowError,
    > {
        let mode = self
            .prepared
            .modes
            .get(sample.mode)
            .ok_or_else(|| contract("trajectory sample mode absent"))?;
        let mut values = mode.context.values.clone();
        self.prepared
            .update_sample_values(sample, &self.prepared.parameters, &mut values)?;
        Ok((mode.model.clone(), values, mode.context.providers.clone()))
    }
    pub(super) async fn derivative_samples(
        &self,
        policy: pse_backend_native::derivative_diagnostics::Policy,
        controls: pse_backend_native::solve::Controls,
        cancel: &crate::CancelSource,
    ) -> Result<
        (
            Vec<(i64, f64, bool, bool, String)>,
            Arc<pse_columnar::AllocationLease>,
        ),
        WorkflowError,
    > {
        let trajectory = self.clone();
        let bytes = policy
            .allowance()
            .map_err(crate::math::MathRuntimeError::from)?
            .checked_add(
                self.report
                    .samples
                    .len()
                    .checked_mul(1024)
                    .ok_or_else(|| contract("derivative sample extent"))?,
            )
            .ok_or_else(|| contract("derivative sample extent"))?;
        let handle = self.prepared.runtime.shared.math().submit(1, bytes, move |flag, _| {
            let mut outcomes = Vec::new();
            for (index, sample) in trajectory.report.samples.iter().enumerate() {
                let p = &trajectory.prepared;
                let parameters = p.profile.changes.iter().rev().find(|c| c.time <= sample.time)
                    .map_or(p.parameters.as_slice(), |c| c.parameters.as_slice()).to_vec();
                let mut functions = vec![(Function::Rhs, p.contract.states.len()), (Function::Output, p.contract.outputs.len())];
                if !p.contract.quadratures.is_empty() { functions.push((Function::QuadratureFlux, p.contract.quadratures.len())); }
                let mut complete = true; let mut passed = true; let mut checked = 0; let mut suspicious = 0; let mut missing = 0; let mut details = Vec::new();
                for (function, rows) in functions {
                    let mut normalization = pse_math::normalization::Normalization::identity(p.contract.states.len() + p.contract.parameters.len(), rows);
                    normalization.variables[p.contract.states.len()..].copy_from_slice(&p.profile.parameter_scales);
                    let result = pse_backend_native::derivative_diagnostics::analyze_dynamic(
                        Box::new(p.worker(flag.clone())?),
                        pse_backend_native::derivative_diagnostics::DynamicSample { mode: sample.mode, function, time: sample.time, state: sample.state.clone(), parameters: parameters.clone() },
                        normalization, policy, pse_backend_native::solve::Execution::new(flag.clone(), &controls),
                    )?;
                    if !result.passed() { details.push(format!("{function:?}: {}", result.summary())); }
                    complete &= result.complete; passed &= result.passed();
                    if let Some(sample) = &result.sample { checked += sample.checked; suspicious += sample.suspicious; missing += sample.missing_structure; }
                }
                outcomes.push((index as i64, sample.time, complete, passed, format!("{checked} comparisons; {suspicious} suspicious entries; {missing} missing sparsity entries; fixed time and mode\n{}", details.join("\n"))));
            }
            let retained = outcomes.capacity() * size_of::<(i64, f64, bool, bool, String)>() + outcomes.iter().map(|o| o.4.capacity()).sum::<usize>();
            Ok((outcomes, retained))
        })?;
        let control = handle.cancellation();
        let finish = handle.finish();
        tokio::pin!(finish);
        Ok(
            tokio::select! { result = &mut finish => result?, () = cancel.cancelled() => { control.cancel(); finish.await? } },
        )
    }
    /// The typed failed qualification or native interruption, without changing termination.
    pub fn diagnostic(&self) -> Option<pse_model::diagnostic::BoundaryDiagnostic> {
        use pse_model::diagnostic::{BoundaryClass as C, BoundaryDiagnostic as D, Observation};
        if self.accepted {
            return None;
        }
        if let Some(error) = &self.validation_error {
            return Some(error.clone());
        }
        if let Some(error) = &self.report.error {
            return Some(super::super::diagnostics::observed(
                error,
                "modeling-trajectory",
            ));
        }
        let class = match self.report.termination {
            native::Termination::Cancelled => C::Cancelled,
            native::Termination::StepLimit | native::Termination::TimeLimit => C::ResourceLimit,
            _ => C::TrialRejected,
        };
        let mut error = D::new(
            class,
            "modeling-trajectory",
            self.checks
                .iter()
                .filter(|c| !c.satisfied)
                .map(|c| c.source_id.as_id()),
            "modeling.trajectory.rejected",
        );
        error.observations.insert(
            "termination".into(),
            Observation::Text(self.report.termination.as_str().into()),
        );
        error.observations.insert(
            "completed_time".into(),
            Observation::Real(self.report.completed_time),
        );
        Some(error)
    }
}
impl ModelingSimulation {
    pub fn identity(&self) -> ContentHash {
        self.key
    }
    pub fn model(&self) -> &ModelingPreparation {
        &self.modes[0].model
    }
    /// Declared mode names in native execution order; callers never persist native indices.
    pub fn mode_names(&self) -> impl Iterator<Item = &str> {
        self.modes.iter().map(|m| m.name.as_str())
    }
    pub fn contract(&self) -> &native::Contract {
        &self.contract
    }
    /// Physical parameter values in the contract's declared order.
    pub fn parameters(&self) -> &[f64] {
        &self.parameters
    }
    pub fn profile(&self) -> &native::Profile {
        &self.profile
    }
    pub fn numerics(&self) -> &pse_model::numerics::ResolvedNumericalPolicy {
        &self.modes[0].numerics
    }
    pub(crate) fn worker(
        &self,
        cancel: Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<DynamicWorker, ProblemError> {
        self.program().worker(cancel)
    }
    pub(crate) fn program(&self) -> crate::workflow::dynamics::DynamicProgram {
        crate::workflow::dynamics::DynamicProgram {
            contract: self.contract.clone(),
            programs: self.programs.clone(),
            coordinates: self.coordinates.clone(),
            modes: self
                .modes
                .iter()
                .map(|m| m.context.clone())
                .collect::<Vec<_>>(),
            max_cells: self.profile.max_cells,
        }
    }
    pub(in crate::workflow) fn submit(
        &self,
        run_id: SemanticId,
        submission: crate::math::Submission,
    ) -> Result<
        crate::math::solves::SolveHandle<(
            (native::Report, checks::SampleChecks),
            Arc<pse_columnar::AllocationLease>,
        )>,
        WorkflowError,
    > {
        let service = self.runtime.shared.math();
        let prepared = self.clone();
        let handle = service.submit_with(1, self.bytes, submission, move |flag, progress| {
            #[cfg(any(feature = "solver-diffsol", feature = "solver-idas"))]
            {
                let started = std::time::Instant::now();
                let mut worker = prepared.worker(flag.clone())?;
                let report = native::integrate_with_progress(
                    &mut worker,
                    &prepared.profile,
                    &prepared.parameters,
                    flag.clone(),
                    progress,
                )?;
                let checks =
                    prepared.check_samples(run_id, &report, &prepared.parameters, &flag, started);
                let bytes = report
                    .numeric_bytes()
                    .checked_add(checks.rows.capacity() * size_of::<ModelingCheck>())
                    .and_then(|n| {
                        n.checked_add(
                            checks
                                .reports
                                .iter()
                                .map(pse_model::HeapUsage::owned_bytes)
                                .sum::<usize>(),
                        )
                    })
                    .and_then(|n| n.checked_add(4 << 20))
                    .ok_or(crate::math::MathRuntimeError::Limit(
                        "modeling trajectory extent",
                    ))?;
                Ok(((report, checks), bytes))
            }
            #[cfg(not(any(feature = "solver-diffsol", feature = "solver-idas")))]
            {
                let _ = (prepared, flag, progress);
                Err(crate::math::MathRuntimeError::Solve(
                    ProblemError::unsupported("dynamic backend is not linked"),
                ))
            }
        })?;
        Ok(handle)
    }
    pub(in crate::workflow) fn finish(
        &self,
        run_id: SemanticId,
        report: native::Report,
        checks: checks::SampleChecks,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> ModelingTrajectory {
        let completion = crate::workflow::numerics::complete(
            crate::workflow::numerics::trajectory_use(&report),
            &checks.rows,
            checks.complete && checks.error.is_none(),
            self.numerics().policy.closure,
        );
        ModelingTrajectory {
            run_id,
            accepted: completion.permits_use(),
            checks: checks.rows,
            reports: checks.reports,
            checks_complete: checks.complete,
            validation_error: checks.error,
            report: Arc::new(report),
            prepared: self.clone(),
            _owner: owner,
        }
    }
    /// Await one joined run. Cancellation still waits for native teardown.
    pub async fn run(
        &self,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingTrajectory, WorkflowError> {
        let handle = self.start()?;
        let wait = handle.wait();
        tokio::pin!(wait);
        let result = tokio::select! { r=&mut wait=>r?, ()=cancel.cancelled()=>{handle.cancel();wait.await?} };
        match &result.report {
            Ok(crate::workflow::RunReport::Simulation(trajectory)) => {
                Ok(trajectory.as_ref().clone())
            }
            Err(error) => Err(WorkflowError::Shared(error.clone())),
            _ => Err(contract("simulation report mismatch")),
        }
    }
}
/// One owner for integrated state/parameter layout, shared with data-authored fixtures.
pub(super) fn dynamic_ports(
    product: &pse_compiler::workspace::PreparedModeling,
    case: &ModelingCaseBindings,
) -> Result<(Vec<pse_kernels::Port>, Vec<SemanticId>), WorkflowError> {
    let axis = product
        .model
        .integrated
        .values()
        .next()
        .ok_or_else(|| contract("missing integrated axis"))?;
    let integral_ids = product
        .model
        .integrals
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    let rates = product
        .model
        .derivatives
        .values()
        .map(|d| d.rate)
        .collect::<BTreeSet<_>>();
    let fixed = case
        .variables
        .iter()
        .map(|(p, s)| {
            let id = *product
                .model
                .paths
                .get(p)
                .ok_or_else(|| contract(format!("unknown dynamic case path {p}")))?;
            if !product
                .admitted
                .case
                .variables()
                .iter()
                .any(|v| v.port.id == id)
            {
                return Err(contract(
                    "dynamic fixedness requires an independent variable",
                ));
            }
            Ok((id, s.fixed))
        })
        .collect::<Result<BTreeMap<_, _>, WorkflowError>>()?;
    let state = product
        .admitted
        .case
        .variables()
        .iter()
        .filter(|v| {
            !rates.contains(&v.port.id)
                && !fixed.get(&v.port.id).copied().flatten().unwrap_or(v.fixed)
        })
        .map(|v| v.port.clone())
        .collect::<Vec<_>>();
    let states = state.iter().map(|p| p.id).collect::<Vec<_>>();
    // ADR-0103 item 6: integration holds discrete variables fixed per segment.
    product
        .model
        .require_fixed_discrete(
            states.iter().copied(),
            pse_modeling::DomainAnalysis::IntegratedDynamics,
        )
        .map_err(crate::workflow::modeling_error)?;
    if product
        .model
        .derivatives
        .keys()
        .any(|id| !states.contains(id))
    {
        return Err(contract(
            "a differentiated state cannot be fixed for integration",
        ));
    }
    let parameters = product
        .admitted
        .inputs
        .iter()
        .filter(|id| {
            !rates.contains(id)
                && !integral_ids.contains(id)
                && **id != axis.time
                && !states.contains(id)
        })
        .copied()
        .collect::<Vec<_>>();
    Ok((state, parameters))
}
impl ModelingPackage {
    /// Resolve authored integration controls against the generated coordinate layout.
    pub(in crate::workflow) fn integration_profile(
        &self,
        model: &ModelingPreparation,
        data: &pse_modeling::specialize::Fixture,
        numerics: &pse_model::numerics::NumericalPolicy,
    ) -> Result<native::Profile, WorkflowError> {
        let integration = data
            .integration
            .as_ref()
            .ok_or_else(|| contract("integration controls absent"))?;
        let product = model.compiled();
        let axis = product
            .model
            .integrated
            .values()
            .next()
            .ok_or_else(|| contract("integrated fixture axis absent"))?;
        let time_scale = self
            .quantities
            .unit(
                self.quantities
                    .quantity_type(axis.quantity)
                    .map_err(super::super::math)?
                    .canonical_unit,
            )
            .map_err(super::super::math)?
            .scale_to_canonical;
        let case = ModelingCaseBindings::from(data);
        let (states, parameters) = dynamics::dynamic_ports(product, &case)?;
        let profile = pse_backend_native::dynamics::Profile {
            start: axis.lower * time_scale,
            end: integration
                .samples
                .last()
                .copied()
                .ok_or_else(|| contract("integration samples empty"))?
                * time_scale,
            samples: integration.samples.iter().map(|t| t * time_scale).collect(),
            rtol: integration.relative_tolerance,
            out_rtol: integration.quadrature_relative_tolerance,
            out_atol: product
                .model
                .integrals
                .keys()
                .map(|id| {
                    integration
                        .quadratures
                        .get(id)
                        .copied()
                        .ok_or_else(|| contract("integral tolerance absent"))
                })
                .collect::<Result<_, _>>()?,
            atol: vec![integration.normalized_absolute_tolerance; states.len()],
            initial_step: integration.initial_step * time_scale,
            parameter_scales: vec![1.; parameters.len()],
            numerics: numerics.clone(),
            ..pse_backend_native::dynamics::Profile::default()
        };
        Ok(profile)
    }
}
impl ModelingPackage {
    /// Bind an authored case directly to the integrated route and its integration controls.
    pub async fn declared_simulation(
        &self,
        root: DeclarationId,
        compiler: Profile,
        profile: Option<native::Profile>,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSimulation, WorkflowError> {
        let (bindings, case) = self
            .declared_case(
                root,
                pse_model::generated::enums::ModelingAnalysisRoute::Integrated,
                limits,
                cancel,
            )
            .await?;
        let instance = pse_modeling::specialize::root_instance(root);
        let profile = if let Some(profile) = profile {
            profile
        } else {
            let model = self
                .prepare(root, instance, bindings.clone(), limits, cancel)
                .await?;
            let fixture = model
                .compiled()
                .model
                .fixtures
                .get(&instance)
                .ok_or_else(|| contract("authored integration controls absent"))?;
            self.integration_profile(&model, fixture, &Default::default())?
        };
        self.prepare_simulation(
            root, instance, bindings, limits, case, compiler, profile, cancel,
        )
        .await
    }
    /// Prepare integrated dynamics from one specialized definition. Rates are compiler
    /// projections; all original equations, hints and source identities remain inspectable.
    async fn prepare_simulation_mode(
        &self,
        root: DeclarationId,
        instance: InstanceId,
        mut bindings: Bindings,
        limits: Limits,
        case: ModelingCaseBindings,
        compiler: Profile,
        profile: native::Profile,
        mode: &ModelingDynamicMode,
        mode_names: &[String],
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSimulation, WorkflowError> {
        mode.extend_bindings(&mut bindings)?;
        if bindings.facts.keys().any(|k| k.starts_with("analysis."))
            && bindings
                .analysis_route()
                .map_err(|e| contract(e.to_string()))?
                != pse_modeling::analysis::Route::Integrated
        {
            return Err(contract(
                "simulation requires the integrated analysis route",
            ));
        }
        bindings = bindings.with_analysis(pse_modeling::analysis::Route::Integrated);
        bindings
            .demand
            .extend(case.values.keys().chain(case.variables.keys()).cloned());
        bindings.demand.sort();
        bindings.demand.dedup();
        let model = self
            .prepare(root, instance, bindings, limits, cancel)
            .await?;
        let product = model.compiled();
        let axis = product
            .model
            .integrated
            .values()
            .next()
            .ok_or_else(|| contract("simulation requires an integrated time axis"))?;
        if product.model.integrated.len() != 1 || product.admitted.case.objective().is_some() {
            return Err(contract(
                "integrated consumer requires one time axis and no optimization objective",
            ));
        }
        let time_unit = self
            .quantities
            .unit(
                self.quantities
                    .quantity_type(axis.quantity)
                    .map_err(super::super::math)?
                    .canonical_unit,
            )
            .map_err(super::super::math)?;
        let time_scale = time_unit.scale_to_canonical;
        if profile.start != axis.lower * time_scale || profile.end > axis.upper * time_scale {
            return Err(contract(
                "integration horizon must start at the domain lower bound and remain inside the domain",
            ));
        }
        let integral_ids = product
            .model
            .integrals
            .keys()
            .copied()
            .collect::<BTreeSet<_>>();
        if !integral_ids.is_empty() {
            if profile.end != axis.upper * time_scale
                || profile.samples.last().copied() != Some(profile.end)
            {
                return Err(contract(
                    "definite integrals require the complete domain and an upper-endpoint sample",
                ));
            }
            // Quadratures coexist with state/output sensitivities. Terminal
            // expressions are excluded from native outputs below, so this does
            // not claim derivatives of the integral itself.
        }
        let terminal_rows = product
            .admitted
            .case
            .instances()
            .iter()
            .filter(|i| i.slots.iter().any(|s| integral_ids.contains(&s.source())))
            .map(|i| i.instance)
            .collect::<BTreeSet<_>>();
        let terminal_targets = product
            .admitted
            .outputs
            .iter()
            .filter_map(|o| match o {
                ModelingOutput::Member(id) if terminal_rows.contains(&o.row_id()) => Some(*id),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        if product.model.annotations.iter().any(|a| {
            terminal_targets.contains(&a.target)
                && matches!(
                    a.value,
                    pse_modeling::annotation::AnnotationValue::Bounds(..)
                        | pse_modeling::annotation::AnnotationValue::Start(..)
                )
        }) {
            return Err(contract(
                "terminal integral bounds or starts require a terminal check, not an integration constraint",
            ));
        }
        let (state, parameters) = dynamic_ports(product, &case)?;
        let rates = product
            .model
            .derivatives
            .values()
            .map(|d| d.rate)
            .collect::<BTreeSet<_>>();
        let states = state.iter().map(|p| p.id).collect::<Vec<_>>();
        let (mut values, starts) = self
            .resolve_starts(
                &model,
                &case,
                BTreeMap::new(),
                BTreeMap::new(),
                compiler,
                false,
                cancel,
            )
            .await?;
        // Terminal placeholders are excluded from every trial program below. Their
        // real values are supplied only by completed native whole-domain quadrature.
        for id in &integral_ids {
            values.scalars.insert(*id, 0.);
        }
        let initial_outputs = product
            .admitted
            .outputs
            .iter()
            .filter_map(|o| {
                if let ModelingOutput::InitialState { state, equation } = o {
                    Some((*state, *equation, o.row_id()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if initial_outputs.len() != product.model.derivatives.len()
            || initial_outputs.len() != product.model.initial_equations.len()
        {
            return Err(contract(
                "every differential state requires one explicit isolated lower-endpoint initial condition",
            ));
        }
        let initial_values = self
            .observe_registered(
                model.clone(),
                initial_outputs.iter().map(|(_, _, row)| *row).collect(),
                values.clone(),
                compiler,
                self.registrations(),
                cancel,
            )
            .await?;
        for (state, _, row) in &initial_outputs {
            values.scalars.insert(*state, initial_values[row]);
        }
        if let Some(id) = states
            .iter()
            .chain(&parameters)
            .find(|id| !values.scalars.contains_key(id))
        {
            return Err(contract(format!(
                "dynamic coordinate {id} requires a value or deterministic start"
            )));
        }
        let providers = self
            .inner_registrations(
                model.clone(),
                &values,
                &case,
                &crate::math::solves::NumericalInputs::default(),
                &profile.numerics,
                &pse_backend_native::solve::Controls::default(),
                compiler,
                cancel,
                None,
            )
            .await?;
        let mut differential = Vec::new();
        let mut rhs = Vec::new();
        let mut used = product.model.initial_equations.clone();
        for id in &states {
            if product.model.derivatives.contains_key(id) {
                let (output, equations) = product
                    .admitted
                    .outputs
                    .iter()
                    .find_map(|o| {
                        if let ModelingOutput::DynamicRate { state, equations } = o {
                            (*state == *id).then_some((o.row_id(), equations))
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| contract("integrated derivative system is absent"))?;
                differential.push(true);
                rhs.push(output);
                used.extend(equations);
            } else {
                differential.push(false);
                rhs.push(SemanticId::NIL);
            }
        }
        let algebraic = product
            .admitted
            .outputs
            .iter()
            .filter_map(|o| match o {
                ModelingOutput::Equation { id, sense } if !used.contains(id) => Some((*id, *sense)),
                _ => None,
            })
            .collect::<Vec<_>>();
        if algebraic.len() != differential.iter().filter(|v| !**v).count()
            || algebraic
                .iter()
                .any(|(_, s)| *s != pse_authoring::dsl::EquationSense::Eq)
        {
            return Err(contract(
                "dynamic algebraic partition must be square and contain equalities only",
            ));
        }
        for (row, (id, _)) in rhs
            .iter_mut()
            .filter(|r| **r == SemanticId::NIL)
            .zip(&algebraic)
        {
            *row = *id;
        }
        self.runtime
            .shared
            .math()
            .validate_modeling_partition(
                self.workspace.clone(),
                model.clone(),
                algebraic.iter().map(|(id, _)| *id).collect(),
                states
                    .iter()
                    .zip(&differential)
                    .filter_map(|(id, d)| (!d).then_some(*id))
                    .collect(),
                compiler,
                cancel,
            )
            .await?;
        let mut initial = Vec::new();
        let mut constants = BTreeMap::new();
        for (i, id) in states.iter().enumerate() {
            if let Some((_, _, row)) = initial_outputs.iter().find(|(state, _, _)| state == id) {
                initial.push(*row);
                continue;
            }
            match start_row(&product.admitted.outputs, *id, starts.get(id))? {
                Some(row) => initial.push(row),
                None => {
                    initial.push(SemanticId::NIL);
                    constants.insert(i, values.scalars[id]);
                }
            }
        }
        let mut outputs = states
            .iter()
            .map(|id| ModelingOutput::Member(*id).row_id())
            .collect::<Vec<_>>();
        for a in &product.model.annotations {
            if matches!(
                a.value,
                pse_modeling::annotation::AnnotationValue::Report(_)
            ) {
                let id = ModelingOutput::Member(a.target).row_id();
                if !outputs.contains(&id) {
                    outputs.push(id);
                }
            }
        }
        outputs.retain(|id| !terminal_rows.contains(id));
        let nominal_rows = product
            .admitted
            .outputs
            .iter()
            .filter(|o| {
                matches!(
                    o,
                    ModelingOutput::Hint {
                        kind: ModelingHint::Nominal,
                        ..
                    }
                )
            })
            .map(ModelingOutput::row_id)
            .collect();
        let observed = self
            .observe_registered(
                model.clone(),
                nominal_rows,
                values.clone(),
                compiler,
                providers.clone(),
                cancel,
            )
            .await?;
        let mut targets = state
            .iter()
            .map(|p| pse_math::numerics::TargetSpec {
                id: p.id,
                kind: NumericalTarget::Variable,
                quantity: p.quantity,
                unit: p.unit,
                integer: false,
                declared_tolerance: None,
            })
            .collect::<Vec<_>>();
        for (id, _) in &algebraic {
            let row = product
                .admitted
                .case
                .rows()
                .iter()
                .find(|r| r.id == *id)
                .ok_or_else(|| contract("algebraic row missing"))?;
            targets.push(pse_math::numerics::TargetSpec {
                id: *id,
                kind: NumericalTarget::Row,
                quantity: row.quantity,
                unit: self
                    .quantities
                    .quantity_type(row.quantity)
                    .map_err(super::super::math)?
                    .canonical_unit,
                integer: false,
                declared_tolerance: None,
            });
        }
        let mut declarations = Vec::new();
        for o in &product.admitted.outputs {
            if let ModelingOutput::Hint {
                target,
                declaration,
                kind: ModelingHint::Nominal,
            } = o
                && let Some(t) = targets.iter().find(|t| t.id == *target)
            {
                declarations.push(cases::requirement(
                    ModelId::from_id(root.as_id()),
                    *target,
                    t.kind,
                    *declaration,
                    NumericalSource::ModelHint,
                    Some(observed[&o.row_id()]),
                    None,
                ));
            }
        }
        let numerics = Arc::new(
            pse_math::numerics::resolve(
                &self.quantities,
                &targets,
                &declarations,
                &profile.numerics,
            )
            .map_err(super::super::math)?,
        );
        let scale = |id, kind| {
            numerics
                .targets
                .iter()
                .find(|t| t.id == id && t.kind == kind)
                .map(|t| t.coordinate_scale)
                .ok_or_else(|| contract("dynamic numerical target absent"))
        };
        let coordinates = DynamicCoordinates {
            time: axis.time,
            time_scale,
            time_origin: 0.,
            state: states
                .iter()
                .map(|id| {
                    Ok(CoordinateBinding {
                        id: *id,
                        scale: scale(*id, NumericalTarget::Variable)?,
                        offset: 0.,
                    })
                })
                .collect::<Result<_, WorkflowError>>()?,
            parameters: parameters
                .iter()
                .map(|id| CoordinateBinding {
                    id: *id,
                    scale: 1.,
                    offset: 0.,
                })
                .collect(),
        };
        let derivative_coordinates = states
            .iter()
            .chain(&parameters)
            .copied()
            .collect::<Vec<_>>();
        let mut programs = Vec::new();
        let (event_contracts, event_roles) =
            events::resolve_events(mode, mode_names, product, &states)?;
        let quadrature_rows = product
            .model
            .integrals
            .values()
            .map(|q| ModelingOutput::Member(q.integrand).row_id())
            .collect::<Vec<_>>();
        let mut roles = vec![
            (Function::Rhs, rhs.clone(), BTreeMap::new()),
            (Function::Initial, initial, constants),
            (Function::Output, outputs.clone(), BTreeMap::new()),
        ];
        if !quadrature_rows.is_empty() {
            roles.push((Function::QuadratureFlux, quadrature_rows, BTreeMap::new()));
        }
        roles.extend(event_roles);
        for (function, rows, mut constants) in roles {
            let selected = rows
                .iter()
                .enumerate()
                .filter(|(i, _)| !constants.contains_key(i))
                .map(|(_, r)| *r)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let compiled = self
                .runtime
                .shared
                .math()
                .prepare_modeling_functions(
                    self.workspace.clone(),
                    model.clone(),
                    selected,
                    derivative_coordinates.clone(),
                    DerivativeOrder::First,
                    compiler,
                    cancel,
                )
                .await?;
            // No derivative-coordinate feedback may leak into an explicit RHS; an
            // implicit DAE would need a residual adapter rather than a guessed rate.
            for i in compiled.assembly.structure().instances() {
                if i.slots.iter().any(|s| integral_ids.contains(&s.source())) {
                    return Err(contract(
                        "definite integral has a noncausal dependency in an integrated trial function; use simultaneous realization",
                    ));
                }
                if i.slots.iter().any(|s| rates.contains(&s.source())) {
                    return Err(contract(
                        "dynamic function depends on an unresolved derivative coordinate",
                    ));
                }
                if function == Function::Initial
                    && i.slots.iter().any(|s| states.contains(&s.source()))
                {
                    return Err(contract(
                        "dynamic initial values must be independent of state guesses",
                    ));
                }
            }
            let row_indices = rows
                .iter()
                .enumerate()
                .map(|(i, id)| {
                    if constants.contains_key(&i) {
                        Ok(0)
                    } else {
                        compiled
                            .assembly
                            .structure()
                            .rows()
                            .iter()
                            .position(|r| r.id == *id)
                            .ok_or_else(|| contract("dynamic output missing"))
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut scales = vec![1.; rows.len()];
            if matches!(
                function,
                Function::Rhs | Function::Initial | Function::Reset(_)
            ) {
                for (i, id) in states.iter().enumerate() {
                    let state_scale = coordinates.state[i].scale;
                    scales[i] = if matches!(function, Function::Initial | Function::Reset(_)) {
                        1. / state_scale
                    } else if differential[i] {
                        let row = compiled
                            .assembly
                            .structure()
                            .rows()
                            .iter()
                            .find(|r| r.id == rows[i])
                            .ok_or_else(|| contract("rate row missing"))?;
                        let rate = self
                            .quantities
                            .quantity_type(row.quantity)
                            .map_err(super::super::math)?;
                        self.quantities
                            .unit(rate.canonical_unit)
                            .map_err(super::super::math)?
                            .scale_to_canonical
                            / self
                                .quantities
                                .unit(state[i].unit)
                                .map_err(super::super::math)?
                                .scale_to_canonical
                            / state_scale
                    } else {
                        1. / scale(rows[i], NumericalTarget::Row)?
                    };
                    if let Some(v) = constants.get_mut(&i) {
                        *v /= state_scale;
                    }
                    let _ = id;
                }
            }
            if function == Function::QuadratureFlux {
                for (i, integral) in product.model.integrals.values().enumerate() {
                    let quantity = |id| -> Result<_, WorkflowError> {
                        let pse_modeling::Type::Quantity(q) = &product.model.symbols[&id].ty else {
                            return Err(contract("integral physical type absent"));
                        };
                        let q = q
                            .resolve(&self.quantities, &BTreeMap::new())
                            .map_err(|e| contract(e.to_string()))?;
                        self.quantities
                            .quantity_type(q)
                            .map_err(|e| contract(e.to_string()))
                    };
                    scales[i] = self
                        .quantities
                        .unit(quantity(integral.integrand)?.canonical_unit)
                        .map_err(super::super::math)?
                        .scale_to_canonical
                        / self
                            .quantities
                            .unit(quantity(integral.result)?.canonical_unit)
                            .map_err(super::super::math)?
                            .scale_to_canonical;
                }
            }
            programs.push(FunctionProgram {
                mode: 0,
                function,
                case: compiled,
                rows: row_indices,
                offsets: vec![0.; scales.len()],
                scales,
                constants,
            });
        }
        let guard = self
            .prepare_dynamic_guards(&model, &case, &terminal_targets, compiler, cancel)
            .await?;
        let units = results::assessment_units(product);
        let sample_scope = (!integral_ids.is_empty()).then(|| {
            units
                .iter()
                .filter(|(_, rows)| rows.is_disjoint(&terminal_rows))
                .map(|(key, _)| *key)
                .collect::<results::AssessmentScope>()
        });
        let mut all_check_rows = units.values().flatten().copied().collect::<BTreeSet<_>>();
        all_check_rows.extend(algebraic.iter().map(|(id, _)| *id));
        let mut sample_rows = units
            .iter()
            .filter(|(key, _)| sample_scope.as_ref().is_none_or(|s| s.contains(key)))
            .flat_map(|(_, rows)| rows.iter().copied())
            .collect::<BTreeSet<_>>();
        sample_rows.extend(algebraic.iter().map(|(id, _)| *id));
        let check_program = if sample_rows.is_empty() {
            None
        } else {
            Some(
                self.runtime
                    .shared
                    .math()
                    .prepare_modeling_functions(
                        self.workspace.clone(),
                        model.clone(),
                        sample_rows.into_iter().collect(),
                        derivative_coordinates.clone(),
                        DerivativeOrder::Value,
                        compiler,
                        cancel,
                    )
                    .await?,
            )
        };
        let terminal_check_program = if integral_ids.is_empty() || all_check_rows.is_empty() {
            None
        } else {
            Some(
                self.runtime
                    .shared
                    .math()
                    .prepare_modeling_functions(
                        self.workspace.clone(),
                        model.clone(),
                        all_check_rows.into_iter().collect(),
                        derivative_coordinates.clone(),
                        DerivativeOrder::Value,
                        compiler,
                        cancel,
                    )
                    .await?,
            )
        };
        let parameter_values = parameters
            .iter()
            .map(|id| values.scalars[id])
            .collect::<Vec<_>>();
        let mut hash = FramedHasher::new(pse_ids::Frame::ModelingDynamicV1);
        hash.id(&root.as_id())
            .id(&instance.as_id())
            .hash(&numerics.key)
            .hash(&super::super::dynamics::profile_identity(&profile));
        for p in &programs {
            for id in p.case.assembly.bodies().keys() {
                hash.hash(id);
            }
            for row in &p.rows {
                hash.u64(*row as u64);
            }
            for (i, value) in &p.constants {
                hash.u64(*i as u64).u64(value.to_bits());
            }
        }
        for (id, v) in &values.scalars {
            hash.id(id).u64(v.to_bits());
        }
        for r in providers.values() {
            hash.hash(&r.configuration_key());
        }
        if let Some(guard) = &guard {
            for id in guard.case.assembly.bodies().keys() {
                hash.hash(id);
            }
            for check in &guard.checks {
                hash.id(&check.source)
                    .bool(check.lower.is_some())
                    .bool(check.upper.is_some());
                for v in [&check.value]
                    .into_iter()
                    .chain(check.lower.as_ref())
                    .chain(check.upper.as_ref())
                {
                    match v {
                        RangeValue::Input(id) => {
                            hash.str("input").id(id);
                        }
                        RangeValue::Output(id) => {
                            hash.str("output").id(id);
                        }
                        RangeValue::Constant(v) => {
                            hash.str("constant").u64(v.to_bits());
                        }
                    }
                }
            }
        }
        for program in check_program.iter().chain(&terminal_check_program) {
            for id in program.assembly.bodies().keys() {
                hash.hash(id);
            }
            for row in program.assembly.structure().rows() {
                hash.id(&row.id);
            }
        }
        hash.str(&mode.name);
        for event in &event_contracts {
            hash.id(&event.id)
                .bool(event.terminal)
                .u64(event.next_mode as u64)
                .u64(event.tolerance.to_bits());
        }
        let key = hash.finish_hash();
        let contract = native::Contract {
            identity: key,
            states,
            differential,
            parameters,
            outputs,
            events: std::iter::once(event_contracts)
                .chain((1..mode_names.len()).map(|_| vec![]))
                .collect(),
            quadratures: product.model.integrals.keys().copied().collect(),
            balances: vec![],
        };
        let cells = profile
            .validate(&contract, &parameter_values)
            .map_err(|e| WorkflowError::Math(e.into()))?;
        let bytes = programs.iter().try_fold(
            cells
                .checked_mul(64)
                .and_then(|n| n.checked_add(4 << 20))
                .ok_or_else(|| contract_error("dynamic result extent"))?,
            |n, p| {
                n.checked_add(p.case.assembly.numeric_worker_bytes())
                    .ok_or_else(|| contract_error("dynamic worker extent"))
            },
        )?;
        let bytes = bytes
            .checked_add(guard.as_ref().map_or(0, |g| {
                g.case.assembly.numeric_worker_bytes() + g.checks.len() * 256 + g.rows.len() * 128
            }))
            .ok_or_else(|| contract_error("dynamic guard extent"))?;
        let maximum_checks = product
            .admitted
            .outputs
            .len()
            .checked_add(product.model.annotations.len())
            .and_then(|n| n.checked_add(product.model.closures.len()))
            .and_then(|n| n.checked_mul(profile.samples.len()))
            .ok_or_else(|| contract_error("dynamic check extent"))?;
        if maximum_checks > profile.max_cells {
            return Err(contract_error("dynamic check cell budget"));
        }
        let bytes = bytes
            .checked_add(
                maximum_checks
                    .checked_mul(size_of::<ModelingCheck>() * 2)
                    .ok_or_else(|| contract_error("dynamic check storage"))?,
            )
            .and_then(|n| {
                n.checked_add(
                    check_program
                        .iter()
                        .chain(&terminal_check_program)
                        .map(|p| p.assembly.numeric_worker_bytes())
                        .sum::<usize>(),
                )
            })
            .and_then(|n| {
                n.checked_add(product.admitted.outputs.len() * 256 + values.scalars.len() * 128)
            })
            .and_then(|n| {
                n.checked_add(
                    product
                        .model
                        .annotations
                        .iter()
                        .map(|a| match &a.value {
                            pse_modeling::annotation::AnnotationValue::Report(label) => {
                                label.len() * 2
                                    + product.model.symbols[&a.target].lineage.path.len() * 2
                                    + 256
                            }
                            _ => 0,
                        })
                        .sum::<usize>(),
                )
            })
            .ok_or_else(|| contract_error("dynamic check storage"))?;
        Ok(ModelingSimulation {
            instance,
            quantities: self.quantities.clone(),
            modes: vec![SimulationMode {
                name: mode.name.clone(),
                model,
                numerics,
                context: DynamicMode {
                    values,
                    providers,
                    guard,
                },
                check_program,
                terminal_check_program,
                sample_scope,
            }],
            contract,
            profile,
            runtime: self.runtime.clone(),
            source: self.clone(),
            programs: programs.into(),
            coordinates,
            parameters: parameter_values,
            key,
            bytes,
        })
    }
    async fn prepare_dynamic_guards(
        &self,
        model: &ModelingPreparation,
        case: &ModelingCaseBindings,
        terminal_targets: &BTreeSet<SemanticId>,
        compiler: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<Option<GuardProgram>, WorkflowError> {
        use pse_modeling::annotation::AnnotationValue;
        let product = model.compiled();
        let mut checks = Vec::new();
        let mut bounded = BTreeSet::new();
        let mut valid = BTreeSet::new();
        let overrides = case
            .variables
            .iter()
            .map(|(path, state)| (product.model.paths[path], state))
            .collect::<BTreeMap<_, _>>();
        let endpoint = |target, source, kind| {
            RangeValue::Output(
                ModelingOutput::Hint {
                    target,
                    declaration: source,
                    kind,
                }
                .row_id(),
            )
        };
        for a in &product.model.annotations {
            if terminal_targets.contains(&a.target) {
                continue;
            }
            let value = if product.admitted.inputs.contains(&a.target) {
                RangeValue::Input(a.target)
            } else {
                RangeValue::Output(ModelingOutput::Member(a.target).row_id())
            };
            let (mut lower, mut upper) = match &a.value {
                AnnotationValue::Bounds(..) => {
                    if !bounded.insert(a.target) {
                        return Err(contract("competing dynamic bounds"));
                    }
                    if !product
                        .admitted
                        .case
                        .variables()
                        .iter()
                        .any(|v| v.port.id == a.target)
                    {
                        return Err(contract("bounds require an independent dynamic variable"));
                    }
                    (
                        Some(endpoint(
                            a.target,
                            a.lineage.declaration,
                            ModelingHint::Lower,
                        )),
                        Some(endpoint(
                            a.target,
                            a.lineage.declaration,
                            ModelingHint::Upper,
                        )),
                    )
                }
                AnnotationValue::Valid {
                    policy: pse_model::generated::enums::ExtrapolationPolicy::Reject,
                    ..
                } => {
                    if !valid.insert(a.target) {
                        return Err(contract("competing dynamic validity ranges"));
                    }
                    (
                        Some(endpoint(
                            a.target,
                            a.lineage.declaration,
                            ModelingHint::ValidLower,
                        )),
                        Some(endpoint(
                            a.target,
                            a.lineage.declaration,
                            ModelingHint::ValidUpper,
                        )),
                    )
                }
                _ => continue,
            };
            if matches!(a.value, AnnotationValue::Bounds(..))
                && let Some(state) = overrides.get(&a.target)
            {
                if let Some(v) = state.lower {
                    lower = v.map(RangeValue::Constant);
                }
                if let Some(v) = state.upper {
                    upper = v.map(RangeValue::Constant);
                }
            }
            checks.push(RangeCheck {
                source: a.lineage.declaration.into(),
                target: a.target,
                value,
                lower,
                upper,
            });
        }
        for (id, state) in overrides {
            if !bounded.contains(&id) && (state.lower.is_some() || state.upper.is_some()) {
                checks.push(RangeCheck {
                    source: id,
                    target: id,
                    value: RangeValue::Input(id),
                    lower: state.lower.flatten().map(RangeValue::Constant),
                    upper: state.upper.flatten().map(RangeValue::Constant),
                });
            }
        }
        if checks.is_empty() {
            return Ok(None);
        }
        let rows = checks
            .iter()
            .flat_map(|c| [Some(&c.value), c.lower.as_ref(), c.upper.as_ref()])
            .flatten()
            .filter_map(|v| {
                if let RangeValue::Output(id) = v {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();
        let compiled = self
            .runtime
            .shared
            .math()
            .prepare_modeling_functions(
                self.workspace.clone(),
                model.clone(),
                rows.into_iter().collect(),
                vec![],
                DerivativeOrder::Value,
                compiler,
                cancel,
            )
            .await?;
        let rows = compiled
            .assembly
            .structure()
            .rows()
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id, i))
            .collect();
        Ok(Some(GuardProgram {
            case: compiled,
            rows,
            checks,
        }))
    }
}
/// The lower-endpoint start of a state without an isolated initial equation: an annotation
/// start is evaluated through the model at the endpoint (its row); any other source is the
/// resolved constant. The typed source decides; no label is parsed (F14).
fn start_row(
    outputs: &[ModelingOutput],
    state: SemanticId,
    source: Option<&StartSource>,
) -> Result<Option<SemanticId>, WorkflowError> {
    let Some(StartSource::Annotation { declaration }) = source else {
        return Ok(None);
    };
    outputs
        .iter()
        .find(|o| {
            matches!(o, ModelingOutput::Hint { target, declaration: d, kind: ModelingHint::Start }
                if *target == state && d == declaration)
        })
        .map(|o| Some(o.row_id()))
        .ok_or_else(|| contract("initial annotation output missing"))
}
fn contract_error(message: &str) -> WorkflowError {
    contract(message)
}

#[cfg(all(test, feature = "solver-diffsol"))]
mod tests {
    use super::*;
    use pse_backend_native::dynamics::Oracle;
    #[test]
    fn start_source_drives_initial_conditions() {
        let state = SemanticId::from_bytes([1; 16]);
        let [declaration, other] = [2, 3].map(|n| DeclarationId::from_bytes([n; 16]));
        let outputs = vec![
            ModelingOutput::Hint {
                target: state,
                declaration,
                kind: ModelingHint::Start,
            },
            ModelingOutput::Hint {
                target: state,
                declaration: other,
                kind: ModelingHint::Lower,
            },
        ];
        // An annotation start is evaluated through the model: its own hint row.
        assert_eq!(
            start_row(
                &outputs,
                state,
                Some(&StartSource::Annotation { declaration })
            )
            .unwrap(),
            Some(outputs[0].row_id())
        );
        // A start annotation without its model output is refused, not frozen.
        assert!(
            start_row(
                &outputs,
                state,
                Some(&StartSource::Annotation { declaration: other })
            )
            .is_err()
        );
        // Every other source is the resolved constant, whatever its case path says.
        for source in [
            StartSource::ModelDefault,
            StartSource::Case {
                path: "annotation:x".into(),
            },
            StartSource::Predecessor,
            StartSource::Continuation,
            StartSource::Stored {
                solution: SemanticId::NIL,
            },
        ] {
            assert_eq!(start_row(&outputs, state, Some(&source)).unwrap(), None);
        }
        assert_eq!(start_row(&outputs, state, None).unwrap(), None);
    }
    fn physical() -> (PhysicalContext, BTreeMap<String, QuantityTypeId>) {
        let mut physical = super::super::super::tests::physical();
        physical.preconditions = Arc::new(
            pse_quantity::PhysicalPreconditions::new(
                pse_quantity::generated::standard_preconditions(),
            )
            .unwrap(),
        );
        physical.key = pse_compiler::workspace::physical_identity(
            &physical.quantities,
            &physical.preconditions,
        );
        let names = BTreeMap::from([
            (
                "Scalar".into(),
                physical.quantities.neutral_dimensionless().unwrap(),
            ),
            (
                "Time".into(),
                QuantityTypeId::from_id(
                    SemanticId::parse_hex("e2ccf6d0a394403db967f4f35b83cb7c").unwrap(),
                ),
            ),
        ]);
        (physical, names)
    }
    #[tokio::test]
    async fn kernel_conformance_integrates_authored_samples_and_retains_each_check() {
        let runtime = super::super::super::tests::runtime();
        let (physical, names) = physical();
        let text = "package p { def Root {domain t:Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]:Time; eq rate[i in t]:d(x[i])/di==2; eq initial:x[0{s}]==1{s}; annotation check x(x[i]<=4{s}); annotation valid x(0{s},4{s},reject); let area:Time=integral(i in t | 2); annotation check area(area>1.9{s});} test dynamic fixture {dof 0; run integrated; integrate samples(0{s},0.5{s},1{s}) relative(1e-6) normalized_absolute(1e-8) step(1e-4{s}) quadrature_relative(1e-6) quadrature_absolute(root.area=1e-7{s});} {child root:Root=Root();} }";
        let rows = pse_authoring::language::parse(
            text,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let rendered = pse_authoring::language::render(&rows).unwrap();
        let reparsed = pse_authoring::language::parse(
            &rendered,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        assert_eq!(
            rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
            reparsed.iter().map(|r| &r.value).collect::<Vec<_>>()
        );
        let package = runtime.modeling_package(rows, physical, names).unwrap();
        let policy = ModelingConformancePolicy {
            fixture_policies: BTreeMap::new(),
            compiler: super::super::super::tests::compiler_profile(),
            solver: super::super::super::tests::profile(),
            numerical: Default::default(),
            limits: Limits::default(),
            maximum_fixtures: 4,
            maximum_checks: 64,
            derivatives: pse_backend_native::derivative_diagnostics::Policy {
                perturbation: 1e-6,
                relative_tolerance: 1e-4,
                maximum_cells: 100,
            },
        };
        let report = package
            .conform(policy, &crate::CancelSource::new())
            .await
            .unwrap();
        assert_eq!(report.trajectories.len(), 1, "{:?}", report.checks);
        let trajectory = report.trajectories.values().next().unwrap();
        assert!(trajectory.accepted, "{:?}", trajectory.diagnostic());
        assert_eq!(trajectory.report.samples.len(), 3);
        let checks = report
            .checks
            .iter()
            .filter(|c| c.kind == pse_model::generated::enums::ModelingConformanceKind::Check)
            .collect::<Vec<_>>();
        assert_eq!(checks.len(), 4);
        let state_target = checks
            .iter()
            .find(|c| c.sample_index == 0)
            .unwrap()
            .target_id;
        let checks = checks
            .into_iter()
            .filter(|c| c.target_id == state_target)
            .collect::<Vec<_>>();
        assert_eq!(checks.len(), 3);
        assert_eq!(
            checks.iter().map(|c| c.sample_index).collect::<Vec<_>>(),
            [0, 1, 2]
        );
        assert_eq!(
            checks.iter().map(|c| c.time).collect::<Vec<_>>(),
            [Some(0.), Some(0.5), Some(1.)]
        );
        assert!(report.passed(), "{:?}", report.checks);
        assert_eq!(
            report
                .checks
                .iter()
                .filter(
                    |c| c.kind == pse_model::generated::enums::ModelingConformanceKind::Derivatives
                )
                .count(),
            3
        );
        report.table().unwrap();
    }
    #[tokio::test]
    async fn kernel_integrated_events_bind_source_resets_and_same_layout_modes() {
        let runtime = super::super::super::tests::runtime();
        let (physical, names) = physical();
        let source = "package p { def Root { domain t: Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); param p: Scalar = 2; var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == p; eq initial: x[0{s}] == 1{s}; let hit[i in t]: Time = x[i]-2{s}; let jump[i in t]: Time = 2*x[i]; let wrong: Scalar = 0; stage coast { override eq rate[i in t]: d(x[i])/di == 0; } annotation check x(x[i] <= 4.01{s}); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical, names).unwrap();
        let cancel = crate::CancelSource::new();
        let event = ModelingDynamicEvent {
            guard: "hit[0{s}]".into(),
            reset: BTreeMap::from([("x[0{s}]".into(), "jump[0{s}]".into())]),
            terminal: false,
            next_mode: Some("coast".into()),
            tolerance: 1e-8,
        };
        let modes = vec![
            ModelingDynamicMode {
                name: "rise".into(),
                facts: BTreeMap::new(),
                events: vec![event.clone()],
            },
            ModelingDynamicMode {
                name: "coast".into(),
                facts: BTreeMap::from([("stage.coast".into(), true)]),
                events: vec![],
            },
        ];
        let profile = native::Profile {
            end: 2.,
            samples: vec![0., 0.25, 0.75, 2.],
            parameter_scales: vec![1.],
            sensitivities: true,
            ..Default::default()
        };
        let compiler = super::super::super::tests::compiler_profile();
        let prepared = package
            .prepare_simulation_modes(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                compiler,
                profile.clone(),
                modes.clone(),
                &cancel,
            )
            .await
            .unwrap();
        let result = prepared.run(&cancel).await.unwrap();
        assert!(
            result.accepted,
            "{:?} {:?}",
            result.report.error, result.validation_error
        );
        assert_eq!(result.report.events.len(), 1);
        assert!((result.report.events[0].time - 0.5).abs() < 1e-5);
        assert_eq!(
            result
                .report
                .samples
                .iter()
                .map(|s| s.mode)
                .collect::<Vec<_>>(),
            vec![0, 0, 1, 1]
        );
        assert!((result.report.samples[3].outputs[0] - 4.).abs() < 1e-6);
        assert!(
            result.report.samples[3].output_sensitivities[0].abs() < 1e-5,
            "moving root sensitivity must cancel at the threshold: {:?}",
            result.report.samples[3]
        );
        assert_eq!(result.checks.len(), 4);
        let tables = result.tables().unwrap();
        assert!(
            tables.contains_key(&pse_relations::generated::runtime::simulation_events::RELATION_ID)
        );
        let mut bad = modes.clone();
        bad[0].events[0]
            .reset
            .insert("x[0{s}]".into(), "wrong".into());
        assert!(
            package
                .prepare_simulation_modes(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    compiler,
                    profile.clone(),
                    bad,
                    &cancel
                )
                .await
                .unwrap_err()
                .to_string()
                .contains("physical types")
        );
        let terminal = vec![ModelingDynamicMode {
            name: "stop".into(),
            facts: BTreeMap::new(),
            events: vec![ModelingDynamicEvent {
                terminal: true,
                reset: BTreeMap::new(),
                next_mode: None,
                ..event
            }],
        }];
        let mut terminal_profile = profile;
        terminal_profile.sensitivities = false;
        let stopped = package
            .prepare_simulation_modes(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                compiler,
                terminal_profile,
                terminal,
                &cancel,
            )
            .await
            .unwrap()
            .run(&cancel)
            .await
            .unwrap();
        assert_eq!(stopped.report.termination, native::Termination::Event);
        assert!(!stopped.accepted);
        assert!(!stopped.checks_complete);
        assert_eq!(stopped.report.samples.len(), 2);
    }
    #[tokio::test]
    async fn kernel_integrated_definite_integrals_are_terminal_native_quadratures() {
        terminal_quadratures(native::Method::Diffsol).await;
        #[cfg(feature = "solver-idas")]
        terminal_quadratures(native::Method::Idas).await;
    }
    async fn terminal_quadratures(method: native::Method) {
        let runtime = super::super::super::tests::runtime();
        let (physical, names) = physical();
        let source = "package p { def Root { domain t: Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); param p: Scalar = 2; var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == p; eq initial: x[0{s}] == 1{s}; let area: Time = integral(i in t | p); let score: Scalar = log(area/1{s}); annotation report area(\"integral\"); annotation report score(\"log-integral\"); annotation check area(area > 3{s}); annotation check x(x[i] <= 6{s}); } }";
        let cancel = crate::CancelSource::new();
        let compiler = super::super::super::tests::compiler_profile();
        for (text, noncausal) in [
            (source.to_string(), false),
            (
                source.replace("d(x[i])/di == p", "d(x[i])/di == area/1{s}"),
                true,
            ),
        ] {
            let rows = pse_authoring::language::parse(
                &text,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let root = rows
                .iter()
                .find(|r| r.name == "Root")
                .unwrap()
                .declaration_id;
            let package = runtime
                .modeling_package(rows, physical.clone(), names.clone())
                .unwrap();
            let profile = native::Profile {
                method,
                end: 2.,
                samples: vec![0., 1., 2.],
                parameter_scales: vec![1.],
                out_rtol: Some(1e-9),
                out_atol: vec![1e-10],
                ..Default::default()
            };
            let preparation = package
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    compiler,
                    profile.clone(),
                    &cancel,
                )
                .await;
            if noncausal {
                let error = preparation.unwrap_err().to_string();
                assert!(error.contains("noncausal"), "{error}");
                continue;
            }
            let prepared = preparation.unwrap();
            assert_eq!(prepared.contract.quadratures.len(), 1);
            assert!(prepared.contract.balances.is_empty());
            assert_eq!(
                prepared.contract.outputs.len(),
                1,
                "terminal outputs must not be prefix samples"
            );
            let result = prepared.run(&cancel).await.unwrap();
            assert!(
                result.accepted,
                "{:?} {:?}",
                result.report.error, result.validation_error
            );
            assert_eq!(result.checks.len(), 4);
            assert_eq!(
                result.checks.iter().filter(|c| c.sample_index == 2).count(),
                2
            );
            assert!(
                (result
                    .reports
                    .iter()
                    .find(|r| r.label == "integral")
                    .unwrap()
                    .value
                    - 4.)
                    .abs()
                    < 1e-6
            );
            assert!(
                (result
                    .reports
                    .iter()
                    .find(|r| r.label == "log-integral")
                    .unwrap()
                    .value
                    - 4_f64.ln())
                .abs()
                    < 1e-6
            );
            assert!((result.report.samples[1].integrals[0] - 2.).abs() < 1e-6);
            let tables = result.tables().unwrap();
            use pse_relations::{columnar::RelationRow, generated::runtime::modeling_reports};
            assert_eq!(
                modeling_reports::Row::rows(&tables[&modeling_reports::RELATION_ID])
                    .unwrap()
                    .len(),
                2
            );
            let mut short = profile.clone();
            short.end = 1.;
            short.samples = vec![0., 1.];
            assert!(
                package
                    .prepare_simulation(
                        root,
                        pse_modeling::specialize::root_instance(root),
                        Bindings::default(),
                        Limits::default(),
                        ModelingCaseBindings::default(),
                        compiler,
                        short,
                        &cancel
                    )
                    .await
                    .unwrap_err()
                    .to_string()
                    .contains("complete domain")
            );
            let mut derivatives = profile;
            derivatives.sensitivities = true;
            let sensitive = package
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    compiler,
                    derivatives,
                    &cancel,
                )
                .await
                .unwrap();
            let sensitive = sensitive.run(&cancel).await.unwrap();
            assert!(sensitive.accepted, "{:?}", sensitive.diagnostic());
            for sample in &sensitive.report.samples {
                assert_eq!(sample.output_sensitivities.len(), 1);
                assert!((sample.output_sensitivities[0] - sample.time).abs() < 1e-6);
                assert!((sample.integrals[0] - 2. * sample.time).abs() < 1e-6);
            }
        }
    }
    #[tokio::test]
    async fn kernel_integrated_sample_checks_share_model_semantics_and_owned_rows() {
        let runtime = super::super::super::tests::runtime();
        let (physical, names) = physical();
        let source = "package p { def Root { domain t: Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == 2; eq initial: x[0{s}] == 1{s}; annotation check x(x[i] <= 4{s}); annotation valid x(0{s},4{s},extrapolate); } }";
        for (text, accepted) in [
            (source.to_string(), false),
            (source.replace("x[i] <= 4{s}", "x[i] <= 6{s}"), true),
        ] {
            let declarations = pse_authoring::language::parse(
                &text,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let root = declarations
                .iter()
                .find(|r| r.name == "Root")
                .unwrap()
                .declaration_id;
            let package = runtime
                .modeling_package(declarations, physical.clone(), names.clone())
                .unwrap();
            let cancel = crate::CancelSource::new();
            let prepared = package
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    super::super::super::tests::compiler_profile(),
                    native::Profile {
                        end: 2.,
                        samples: vec![0., 1., 2.],
                        ..Default::default()
                    },
                    &cancel,
                )
                .await
                .unwrap();
            let result = prepared.run(&cancel).await.unwrap();
            assert_eq!(result.report.termination, native::Termination::Completed);
            assert!(result.checks_complete, "{:?}", result.validation_error);
            assert_eq!(result.accepted, accepted);
            assert_eq!(result.checks.len(), 6);
            assert!(result.checks.iter().any(|c| c.sample_index == 2
                && c.time == Some(2.)
                && c.within_validity == Some(false)
                && c.extrapolation_allowed == Some(true)
                && c.satisfied));
            let tables = result.tables().unwrap();
            use pse_relations::{
                columnar::RelationRow,
                generated::runtime::{computation_runs, modeling_checks},
            };
            let checks = tables[&modeling_checks::RELATION_ID].clone();
            let header =
                computation_runs::Row::rows(&tables[&computation_runs::RELATION_ID]).unwrap();
            assert_eq!(header[0].feasible, Some(accepted));
            assert_eq!(
                header[0].qualification,
                if accepted {
                    pse_model::generated::enums::NativeQualification::Feasible
                } else {
                    pse_model::generated::enums::NativeQualification::Unqualified
                }
            );
            drop(result);
            drop(prepared);
            drop(package);
            drop(tables);
            let rows = modeling_checks::Row::rows(&checks).unwrap();
            assert_eq!(rows.len(), 6);
            assert!(rows.iter().all(|r| r.time.is_some()));
        }
    }
    #[tokio::test]
    async fn kernel_integrated_coupled_rate_matrix_uses_original_residual_and_parameter_jets() {
        let runtime = super::super::super::tests::runtime();
        let (physical, names) = physical();
        let source = "package p { def Root { domain t: Time from 0{s} to 2{s}; discretize mesh on t using integrated(elements=1,order=1); param p: Scalar = 2; var x[i in t]: Time; var y[i in t]: Time; eq first[i in t]: p*d(x[i])/di+d(y[i])/di == p; eq second[i in t]: d(x[i])/di+2*d(y[i])/di == 0; eq ix: x[0{s}] == 1{s}; eq iy: y[0{s}] == 3{s}; } }";
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        for (text, nonlinear, singular) in [
            (source.to_string(), false, false),
            (
                source.replace("p*d(x[i])/di", "d(x[i])/di*d(x[i])/di"),
                true,
                false,
            ),
            (
                source.replace("param p: Scalar = 2", "param p: Scalar = 0.5"),
                false,
                true,
            ),
        ] {
            let rows = pse_authoring::language::parse(
                &text,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap();
            let root = rows
                .iter()
                .find(|r| r.name == "Root")
                .unwrap()
                .declaration_id;
            let package = runtime
                .modeling_package(rows, physical.clone(), names.clone())
                .unwrap();
            let result = package
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    compiler,
                    native::Profile {
                        end: 2.,
                        samples: vec![0., 1., 2.],
                        atol: vec![1e-8; 2],
                        parameter_scales: vec![1.],
                        sensitivities: true,
                        ..Default::default()
                    },
                    &cancel,
                )
                .await;
            if nonlinear {
                let error = result.unwrap_err().to_string();
                assert!(error.contains("affine residual"), "{error}");
                continue;
            }
            let prepared = result.unwrap();
            let product = prepared.model().compiled();
            let inner = product.admitted.implicit.values().next().unwrap();
            assert_eq!(
                inner.algorithm,
                pse_compiler::workspace::ImplicitAlgorithm::AffineRates
            );
            assert_eq!(inner.residuals[0].rows.len(), 2);
            assert!(
                product
                    .admitted
                    .outputs
                    .iter()
                    .filter_map(|o| match o {
                        ModelingOutput::DynamicRate { equations, .. } => Some(equations),
                        _ => None,
                    })
                    .all(|rows| rows.len() == 2)
            );
            let x = product
                .model
                .symbols
                .values()
                .find(|s| s.lineage.path.ends_with(".x"))
                .unwrap()
                .id;
            let y = product
                .model
                .symbols
                .values()
                .find(|s| s.lineage.path.ends_with(".y"))
                .unwrap()
                .id;
            let xi = prepared
                .contract
                .states
                .iter()
                .position(|s| *s == x)
                .unwrap();
            let yi = prepared
                .contract
                .states
                .iter()
                .position(|s| *s == y)
                .unwrap();
            let result = prepared.run(&cancel).await.unwrap();
            if singular {
                assert_ne!(result.report.termination, native::Termination::Completed);
                assert!(result.report.error.is_some());
                continue;
            }
            assert_eq!(
                result.report.termination,
                native::Termination::Completed,
                "{:?}",
                result.report.error
            );
            for sample in &result.report.samples {
                assert!((sample.outputs[xi] - (1. + 4. * sample.time / 3.)).abs() < 1e-6);
                assert!((sample.outputs[yi] - (3. - 2. * sample.time / 3.)).abs() < 1e-6);
                assert!((sample.output_sensitivities[xi] + 2. * sample.time / 9.).abs() < 1e-6);
                assert!((sample.output_sensitivities[yi] - sample.time / 9.).abs() < 1e-6);
            }
        }
    }
    #[tokio::test]
    async fn dynamics_refuses_free_integer() {
        let runtime = super::super::super::tests::runtime();
        let (physical, mut names) = physical();
        names.extend(super::super::super::tests::discrete_names());
        let source = "package p { def Root { domain t: Time from 0{s} to 1{s}; discretize mesh on t using integrated(elements=1,order=1); var x[i in t]: Time; var units: Count in integer; eq ode[i in t]: d(x[i])/di == 2; eq initial: x[0{s}] == 1{s}; annotation start x(0{s}); annotation start units(1{1}); annotation bounds units(0{1}, 3{1}); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical, names).unwrap();
        let profile = native::Profile {
            end: 1.,
            samples: vec![0., 1.],
            parameter_scales: vec![1.],
            ..Default::default()
        };
        let cancel = crate::CancelSource::new();
        let simulate = |case| {
            package.prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                case,
                super::super::super::tests::compiler_profile(),
                profile.clone(),
                &cancel,
            )
        };
        let error = simulate(ModelingCaseBindings::default())
            .await
            .err()
            .unwrap();
        assert_eq!(
            super::super::super::tests::free_discrete_refusal(&error),
            ("units".into(), "integrated_dynamics".into())
        );
        // Fixed for the segment, the discrete input is an ordinary parameter of integration.
        let fixed = ModelingCaseBindings {
            values: BTreeMap::from([("units".into(), 1.)]),
            variables: BTreeMap::from([(
                "units".into(),
                pse_compiler::workspace::ModelingVariableState {
                    fixed: Some(true),
                    ..Default::default()
                },
            )]),
        };
        let prepared = simulate(fixed).await.unwrap();
        assert_eq!(prepared.contract.states.len(), 1);
    }
    #[tokio::test]
    async fn kernel_integrated_time_uses_generated_rates_native_solver_and_initial_sensitivities() {
        let runtime = super::super::super::tests::runtime();
        let (physical, names) = physical();
        let source = "package p { def Root { domain t: Time from 0{s} to 2{s}; discretize mesh on t using integrated(elements=1,order=1); param p: Scalar = 2; param offset: Time = 1{s}; var x[i in t]: Time; eq ode[i in t]: d(x[i])/di == p; eq initial: x[0{s}] == offset; annotation start x(0{s}); annotation nominal x(10{s}); annotation check x(abs(x[i] - offset - p*i) < 1e-5{s}); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical, names).unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let profile = native::Profile {
            end: 2.,
            samples: vec![0., 0.5, 1., 2.],
            parameter_scales: vec![1., 1.],
            sensitivities: true,
            ..Default::default()
        };
        let prepared = package
            .prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                compiler,
                profile,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(prepared.contract.states.len(), 1);
        assert_eq!(prepared.coordinates.state[0].scale, 10.);
        let named = |name: &str| {
            prepared
                .model()
                .compiled()
                .model
                .symbols
                .values()
                .find(|s| s.lineage.path.ends_with(&format!(".{name}")))
                .unwrap()
                .id
        };
        let p = named("p");
        let offset = named("offset");
        let pi = prepared
            .contract
            .parameters
            .iter()
            .position(|id| *id == p)
            .unwrap();
        let oi = prepared
            .contract
            .parameters
            .iter()
            .position(|id| *id == offset)
            .unwrap();
        let result = prepared.run(&cancel).await.unwrap();
        assert_eq!(
            result.report.termination,
            native::Termination::Completed,
            "{:?}",
            result.report.error
        );
        for sample in &result.report.samples {
            assert!((sample.outputs[0] - (1. + 2. * sample.time)).abs() < 1e-6);
            assert!((sample.state[0] - (1. + 2. * sample.time) / 10.).abs() < 1e-6);
            assert!((sample.output_sensitivities[pi] - sample.time).abs() < 1e-6);
            assert!((sample.output_sensitivities[oi] - 1.).abs() < 1e-6);
        }
        // @start is only a guess: the original endpoint equation determines x(0).
        assert!((result.report.requested_initial[0] - 0.1).abs() < 1e-12);
        let tables = result.tables().unwrap();
        use pse_relations::{
            columnar::RelationRow,
            generated::runtime::{computation_runs, response_sensitivities, simulation_samples},
        };
        let samples = tables[&simulation_samples::RELATION_ID].clone();
        let header = computation_runs::Row::rows(&tables[&computation_runs::RELATION_ID]).unwrap();
        assert_eq!(
            header[0].trajectory_termination,
            Some(native::Termination::Completed)
        );
        assert_eq!(header[0].completed_samples, Some(4));
        assert_eq!(
            response_sensitivities::Row::rows(&tables[&response_sensitivities::RELATION_ID])
                .unwrap()
                .len(),
            8
        );
        drop(result);
        let rows = simulation_samples::Row::rows(&samples).unwrap();
        assert_eq!(rows.len(), 4);
        assert!((rows.last().unwrap().value - 5.).abs() < 1e-6);
        let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut worker = prepared.worker(flag.clone()).unwrap();
        assert!(
            worker
                .evaluate(0, Function::Rhs, 0., &[0.1], &prepared.parameters, true)
                .is_ok()
        );
        // Fitting trials reuse compilation, but their checks must consume the
        // actual trial parameters, including the initial-condition parameter.
        let mut trial_parameters = prepared.parameters.clone();
        trial_parameters[pi] = 3.;
        trial_parameters[oi] = 4.;
        let trial = native::integrate(
            &mut worker,
            &prepared.profile,
            &trial_parameters,
            flag.clone(),
        )
        .unwrap();
        assert_eq!(trial.termination, native::Termination::Completed);
        let checks = prepared.check_samples(
            root.as_id(),
            &trial,
            &trial_parameters,
            &flag,
            std::time::Instant::now(),
        );
        assert!(checks.complete && checks.error.is_none());
        assert!(checks.rows.iter().all(|c| c.satisfied));
        let wrong = prepared.check_samples(
            root.as_id(),
            &trial,
            &prepared.parameters,
            &flag,
            std::time::Instant::now(),
        );
        assert!(wrong.rows.iter().any(|c| !c.satisfied));
        assert_eq!(prepared.parameters[pi], 2.);
        assert_eq!(prepared.parameters[oi], 1.);
        flag.store(true, std::sync::atomic::Ordering::Release);
        assert!(matches!(
            worker.evaluate(0, Function::Rhs, 0., &[0.1], &prepared.parameters, true),
            Err(ProblemError::Math(pse_math::MathError::Cancelled))
        ));
    }
    #[tokio::test]
    async fn kernel_nonlinear_dae_initial_parameter_sensitivities_and_quadrature_agree() {
        let runtime = super::super::super::tests::runtime();
        let (physical, names) = physical();
        let source = "package p { def Root { domain t:Time from 0{s} to 1{s}; discretize mesh on t using integrated(elements=1,order=1); param p:Scalar=2; param offset:Time=1{s}; var x[i in t]:Time; var y[i in t]:Scalar; eq rate[i in t]:d(x[i])/di==p; eq initial:x[0{s}]==offset; eq algebraic[i in t]:y[i]*y[i]==x[i]/1{s}; annotation start x(1{s}); annotation start y(1); let area:Time=integral(i in t | p); annotation check area(abs(area-2{s})<1e-5{s}); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let y_declaration = rows.iter().find(|r| r.name == "y").unwrap().declaration_id;
        let package = runtime.modeling_package(rows, physical, names).unwrap();
        let cancel = crate::CancelSource::new();
        let mut methods = vec![native::Method::Diffsol];
        #[cfg(feature = "solver-idas")]
        methods.push(native::Method::Idas);
        for method in methods {
            let profile = native::Profile {
                method,
                end: 1.,
                samples: vec![0., 0.5, 1.],
                rtol: 1e-8,
                atol: vec![1e-10; 2],
                parameter_scales: vec![1.; 2],
                sensitivities: true,
                out_rtol: Some(1e-8),
                out_atol: vec![1e-9],
                ..Default::default()
            };
            let prepared = package
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Default::default(),
                    Default::default(),
                    super::super::super::tests::compiler_profile(),
                    profile,
                    &cancel,
                )
                .await
                .unwrap();
            let y = prepared
                .model()
                .compiled()
                .model
                .symbols
                .values()
                .find(|s| s.lineage.declaration == y_declaration)
                .unwrap()
                .id;
            let yi = prepared
                .contract
                .states
                .iter()
                .position(|id| *id == y)
                .unwrap();
            let parameter = |name: &str| {
                prepared
                    .model()
                    .compiled()
                    .model
                    .symbols
                    .values()
                    .find(|s| s.lineage.path.ends_with(&format!(".{name}")))
                    .unwrap()
                    .id
            };
            let pi = prepared
                .contract
                .parameters
                .iter()
                .position(|id| *id == parameter("p"))
                .unwrap();
            let oi = prepared
                .contract
                .parameters
                .iter()
                .position(|id| *id == parameter("offset"))
                .unwrap();
            let result = prepared.run(&cancel).await.unwrap();
            assert!(result.accepted, "{method:?}: {:?}", result.diagnostic());
            for sample in &result.report.samples {
                let y = (1. + 2. * sample.time).sqrt();
                assert!((sample.state[yi] - y).abs() < 1e-6);
                assert!(
                    (sample.state_sensitivities[yi * 2 + pi] - sample.time / (2. * y)).abs() < 1e-5
                );
                assert!((sample.state_sensitivities[yi * 2 + oi] - 1. / (2. * y)).abs() < 1e-5);
            }
        }
    }
    #[tokio::test]
    async fn kernel_integrated_dae_checks_partition_and_retains_completed_samples_on_failure() {
        let runtime = super::super::super::tests::runtime();
        let (physical, names) = physical();
        let source = "package p { def Root { domain t: Time from 0{s} to 2{s}; discretize mesh on t using integrated(elements=1,order=1); param p: Scalar = 2; param offset: Time = 1{s}; var x[i in t]: Time; var y: Scalar; eq ode[i in t]: d(x[i])/di == y; eq initial: x[0{s}] == offset; eq algebraic: y == 2*p; annotation start x(0{s}); annotation start y(999); } }";
        let parse = |source: &str| {
            pse_authoring::language::parse(
                source,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap()
        };
        let rows = parse(source);
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, physical, names.clone())
            .unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let profile = native::Profile {
            end: 1.,
            samples: vec![0., 0.1, 0.5, 1.],
            atol: vec![1e-8; 2],
            parameter_scales: vec![1., 1.],
            sensitivities: true,
            ..Default::default()
        };
        let prepared = package
            .prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                compiler,
                profile.clone(),
                &cancel,
            )
            .await
            .unwrap();
        let xi = prepared
            .contract
            .differential
            .iter()
            .position(|d| *d)
            .unwrap();
        let yi = 1 - xi;
        let result = prepared.run(&cancel).await.unwrap();
        assert_eq!(
            result.report.termination,
            native::Termination::Completed,
            "{:?}",
            result.report.error
        );
        assert!((result.report.consistent_initial[yi] - 4.).abs() < 1e-7);
        for sample in &result.report.samples {
            assert!((sample.outputs[xi] - (1. + 4. * sample.time)).abs() < 1e-6);
            assert!((sample.outputs[yi] - 4.).abs() < 1e-6);
        }
        let singular = package
            .revised(parse(&source.replace("y == 2*p", "p == 2")), names.clone())
            .unwrap();
        assert!(
            singular
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    compiler,
                    profile.clone(),
                    &cancel
                )
                .await
                .is_err()
        );
        let no_initial = package
            .revised(
                parse(&source.replace("eq initial: x[0{s}] == offset;", "")),
                names.clone(),
            )
            .unwrap();
        assert!(
            no_initial
                .prepare_simulation(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    compiler,
                    profile.clone(),
                    &cancel
                )
                .await
                .unwrap_err()
                .to_string()
                .contains("initial condition")
        );
        let guarded = package
            .revised(
                parse(&source.replace(
                    "annotation start x(0{s});",
                    "annotation start x(0{s}); annotation valid x(0{s},2{s},reject);",
                )),
                names.clone(),
            )
            .unwrap();
        let prepared = guarded
            .prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                compiler,
                profile,
                &cancel,
            )
            .await
            .unwrap();
        let mut worker = prepared
            .worker(Arc::new(std::sync::atomic::AtomicBool::new(false)))
            .unwrap();
        let mut outside = vec![4.; 2];
        outside[xi] = 3.;
        let error = worker
            .evaluate(0, Function::Rhs, 0., &outside, &prepared.parameters, false)
            .unwrap_err();
        assert!(matches!(
            error,
            ProblemError::Math(pse_math::MathError::OutsideRange {
                value: 3.,
                lower: Some(0.),
                upper: Some(2.),
                ..
            })
        ));
        assert_eq!(
            pse_backend_native::callback::classify(&error),
            pse_backend_native::callback::Failure::Trial
        );
        let diagnostic = crate::workflow::diagnostics::observed(&error, "dynamic-test");
        assert_eq!(
            diagnostic.class,
            pse_model::diagnostic::BoundaryClass::TrialRejected
        );
        assert_eq!(diagnostic.rule, "math.range");
        assert_eq!(diagnostic.sources.len(), 2);
        for (name, value) in [("value", 3_f64), ("lower", 0.), ("upper", 2.)] {
            assert!(
                matches!(diagnostic.observations[name], pse_model::diagnostic::Observation::Real(actual) if actual.to_bits() == value.to_bits())
            );
        }
        let result = prepared.run(&cancel).await.unwrap();
        assert_ne!(result.report.termination, native::Termination::Completed);
        assert!(result.report.error.is_some());
        assert!(!result.report.samples.is_empty());
        assert!(result.report.samples.iter().all(|s| s.outputs[xi] <= 2.));
        let bounded = package
            .revised(
                parse(&source.replace(
                    "annotation start x(0{s});",
                    "annotation start x(0{s}); annotation bounds x(0{s},2{s});",
                )),
                names,
            )
            .unwrap();
        let profile = prepared.profile.clone();
        let case = ModelingCaseBindings {
            values: BTreeMap::new(),
            variables: BTreeMap::from([(
                "x[0{s}]".into(),
                pse_compiler::workspace::ModelingVariableState {
                    upper: Some(Some(10.)),
                    ..Default::default()
                },
            )]),
        };
        let override_bounds = bounded
            .prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                case,
                compiler,
                profile,
                &cancel,
            )
            .await
            .unwrap();
        let result = override_bounds.run(&cancel).await.unwrap();
        assert_eq!(
            result.report.termination,
            native::Termination::Completed,
            "{:?}",
            result.report.error
        );
    }

    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn kernel_same_definition_selects_steady_integrated_and_simultaneous() {
        use pse_modeling::analysis::Route;
        let runtime = super::super::super::tests::runtime();
        let (physical, names) = physical();
        let source = "package p { difference backward order(1) offsets(-1,0) weights(-1,1) quadrature(0.5,0.5); def Root { domain t: Time from 0{s} to 1{s}; when analysis.route == analysis.integrated { discretize native on t using integrated(elements=1,order=1); } when analysis.route != analysis.integrated { discretize grid on t using backward(elements=4,order=1); } param target: Time = 2{s}; param tau: Time = 1{s}; var x[i in t]: Time; when analysis.dynamic { eq transient[i in t]: d(x[i])/di == (target-x[i])/tau; eq initial: x[0{s}] == 0{s}; } when not analysis.dynamic { eq stationary[i in t]: 0 == (target-x[i])/tau; } annotation start x(0{s}); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical, names).unwrap();
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        let simulation = package
            .prepare_simulation(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                compiler,
                native::Profile {
                    parameter_scales: vec![1.; 2],
                    ..Default::default()
                },
                &cancel,
            )
            .await
            .unwrap();
        let trajectory = simulation.run(&cancel).await.unwrap();
        assert_eq!(
            trajectory.report.termination,
            native::Termination::Completed,
            "{:?}",
            trajectory.report.error
        );
        assert!(
            (trajectory.report.samples.last().unwrap().outputs[0] - 2. * (1. - (-1.0f64).exp()))
                .abs()
                < 1e-5
        );
        for route in [Route::Steady, Route::Simultaneous] {
            let mut solver = super::super::super::tests::profile();
            solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
                pse_backend_native::solve::Backend::Kinsol,
            );
            let prepared = package
                .prepare_solve(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default().with_analysis(route),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    DerivativeOrder::First,
                    compiler,
                    solver,
                    crate::math::solves::NumericalInputs::default(),
                    &cancel,
                )
                .await
                .unwrap();
            let ids = prepared
                .model
                .model
                .compiled()
                .admitted
                .case
                .variables()
                .iter()
                .map(|v| v.port.id)
                .collect::<Vec<_>>();
            let result = package
                .solve_case(prepared, compiler, &cancel)
                .await
                .unwrap();
            assert!(result.accepted, "{result:?}");
            let values = ids
                .iter()
                .map(|id| result.values.scalars[id])
                .collect::<Vec<_>>();
            if route == Route::Steady {
                assert!(values.iter().all(|v| (*v - 2.).abs() < 1e-7));
            } else {
                let end = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                assert!((end - 2. * (1. - 1.25f64.powi(-4))).abs() < 1e-7);
            }
        }
    }
}
