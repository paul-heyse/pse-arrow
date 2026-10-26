// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored source interpretation feeds the common sparse fitting engine.
use super::*;
use crate::math::modeling::ModelingPreparation;
use crate::workflow::modeling::{ModelingPackage, ModelingSimulation, results as checks};
use pse_compiler::workspace::{ModelingCaseBindings, ModelingOutput, Profile};
use pse_model::generated::enums::ModelingAnalysisRoute as Route;
use pse_model::generated::identities::RunId;
use pse_modeling::Bindings;
use pse_modeling::Limits;

struct FitPreparationInput<'a> {
    binding: Option<&'a super::super::AdmittedBinding>,
    pending: Option<&'a BTreeMap<InstanceId, super::super::PhysicalSimulationRequest>>,
}

#[derive(Clone, Debug)]
pub(super) enum Assessment {
    Steady {
        model: Box<ModelingPreparation>,
        program: Option<Arc<ExecutableCase>>,
        numerics: pse_math::SharedAllocation<ResolvedNumericalPolicy>,
    },
    Transient(Box<ModelingSimulation>),
}
fn member(model: &ModelingPreparation, path: &str) -> Result<SemanticId, WorkflowError> {
    model
        .compiled()
        .model
        .paths
        .get(path)
        .copied()
        .ok_or_else(|| contract(format!("unknown fit source path {path}")))
}
fn port(model: &ModelingPreparation, id: SemanticId) -> Result<Port, WorkflowError> {
    let source = &model.compiled().admitted.case();
    source
        .parameters()
        .iter()
        .find(|p| p.id == id)
        .cloned()
        .ok_or_else(|| contract("a shared fit parameter must bind a numerical parameter member"))
}
type FitSource = (
    Bindings,
    ModelingCaseBindings,
    ModelingPreparation,
    BTreeMap<usize, SemanticId>,
);
fn fit_binding_refusal(
    class: pse_model::diagnostic::BoundaryClass,
    members: impl IntoIterator<Item = SemanticId>,
    reason: &str,
) -> WorkflowError {
    let mut diagnostic = pse_model::diagnostic::BoundaryDiagnostic::new(
        class,
        pse_diagnostics::DiagnosticStage::StudyBinding,
        members,
        pse_diagnostics::DiagnosticRule::StudyBindingTarget,
    );
    diagnostic.observations.insert(
        "reason".into(),
        pse_model::diagnostic::Observation::Text(reason.into()),
    );
    diagnostic.into()
}
impl ModelingPackage {
    /// Admit global fit-parameter quantities through every experiment's shared physical port.
    pub(in crate::workflow) async fn admit_fit_overlay(
        &self,
        services: &PreparationServices,
        id: FitId,
        overlay: &super::super::PointOverlay,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<super::super::AdmittedBinding, WorkflowError> {
        let declaration = self.fit_declaration(services, id).await?;
        let (sources, _) = self
            .fit_sources(services, &declaration, limits, cancel)
            .await?;
        let mut entries = BTreeMap::new();
        for assignment in &overlay.assignments {
            let super::super::BindingTarget::Member(member) = assignment.target else {
                return Err(fit_binding_refusal(
                    pse_model::diagnostic::BoundaryClass::Unsupported,
                    [],
                    "fit point targets require an explicit global parameter identity",
                ));
            };
            let index = declaration
                .parameters
                .iter()
                .position(|parameter| parameter.symbol_id == member)
                .ok_or_else(|| {
                    fit_binding_refusal(
                        pse_model::diagnostic::BoundaryClass::InvalidModel,
                        [member],
                        "fit point target is not a declared global parameter",
                    )
                })?;
            let (model, local) = sources
                .iter()
                .find_map(|(_, _, model, local)| local.get(&index).map(|local| (model, *local)))
                .ok_or_else(|| contract("fit parameter has no experiment binding"))?;
            let mut entry =
                self.admit_target_quantity(model.compiled(), local, &assignment.value)?;
            entry.member = member;
            entry.supplied = assignment.target.clone();
            if entries.insert(member, entry).is_some() {
                return Err(fit_binding_refusal(
                    pse_model::diagnostic::BoundaryClass::InvalidModel,
                    [member],
                    "duplicate fit parameter assignment",
                ));
            }
        }
        Ok(super::super::AdmittedBinding {
            revision: self.revision.identity(),
            context: self.physical.identity(),
            entries,
        })
    }
    pub(in crate::workflow) async fn validate_fit_binding(
        &self,
        services: &PreparationServices,
        id: FitId,
        binding: &super::super::AdmittedBinding,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<(), WorkflowError> {
        let declaration = self.fit_declaration(services, id).await?;
        let (sources, _) = self
            .fit_sources(services, &declaration, limits, cancel)
            .await?;
        self.validate_fit_source_binding(&declaration, &sources, binding)
    }
    fn validate_fit_source_binding(
        &self,
        declaration: &FitDeclaration,
        sources: &[FitSource],
        binding: &super::super::AdmittedBinding,
    ) -> Result<(), WorkflowError> {
        if binding.revision != self.revision.identity()
            || binding.context != self.physical.identity()
        {
            return Err(fit_binding_refusal(
                pse_model::diagnostic::BoundaryClass::Incompatible,
                [],
                "fit binding source/context differs",
            ));
        }
        for (member, entry) in &binding.entries {
            let index = declaration
                .parameters
                .iter()
                .position(|parameter| parameter.symbol_id == *member)
                .ok_or_else(|| {
                    fit_binding_refusal(
                        pse_model::diagnostic::BoundaryClass::Incompatible,
                        [*member],
                        "fit binding target is not a global parameter",
                    )
                })?;
            if entry.member != *member
                || !entry.parameter
                || entry.supplied != super::super::BindingTarget::Member(*member)
            {
                return Err(fit_binding_refusal(
                    pse_model::diagnostic::BoundaryClass::Incompatible,
                    [*member],
                    "fit binding must retain its global parameter role",
                ));
            }
            let (model, local) = sources
                .iter()
                .find_map(|(_, _, model, local)| local.get(&index).map(|local| (model, *local)))
                .ok_or_else(|| contract("fit parameter has no experiment binding"))?;
            let mut local_entry = entry.clone();
            local_entry.member = local;
            self.validate_binding(
                model.compiled(),
                &super::super::AdmittedBinding {
                    revision: binding.revision,
                    context: binding.context,
                    entries: BTreeMap::from([(local, local_entry)]),
                },
            )?;
        }
        Ok(())
    }
    async fn fit_sources(
        &self,
        services: &PreparationServices,
        declaration: &FitDeclaration,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<(Vec<FitSource>, Vec<Port>), WorkflowError> {
        let mut sources = Vec::new();
        let mut ports: Vec<Option<Port>> = vec![None; declaration.parameters.len()];
        for experiment in &declaration.experiments {
            let (mut bindings, case) = self
                .declared_case(
                    services,
                    experiment.case_id,
                    experiment.route,
                    limits,
                    cancel,
                )
                .await?;
            bindings.demand.extend(
                experiment
                    .bindings
                    .iter()
                    .map(|binding| binding.path.clone()),
            );
            bindings.demand.extend(
                declaration
                    .observations
                    .iter()
                    .filter(|observation| observation.experiment_id == experiment.experiment_id)
                    .map(|observation| observation.output_path.clone()),
            );
            bindings.demand.sort();
            bindings.demand.dedup();
            let model = self
                .prepare(
                    services,
                    experiment.case_id,
                    experiment.experiment_id,
                    bindings.clone(),
                    limits,
                    cancel,
                )
                .await?;
            let mut local = BTreeMap::new();
            let mut local_ids = BTreeSet::new();
            for binding in &experiment.bindings {
                let parameter = declaration
                    .parameters
                    .iter()
                    .position(|parameter| parameter.symbol_id == binding.parameter_id)
                    .ok_or_else(|| contract("experiment binds an undeclared fit parameter"))?;
                let member = member(&model, &binding.path)?;
                let mut parameter_port = port(&model, member)?;
                parameter_port.id = binding.parameter_id;
                if ports[parameter]
                    .as_ref()
                    .is_some_and(|old| old != &parameter_port)
                {
                    return Err(contract("shared parameter physical contract mismatch"));
                }
                ports[parameter] = Some(parameter_port);
                if local.insert(parameter, member).is_some() || !local_ids.insert(member) {
                    return Err(contract("duplicate experiment parameter binding"));
                }
            }
            sources.push((bindings, case, model, local));
        }
        let ports = ports
            .into_iter()
            .map(|port| port.ok_or_else(|| contract("fit parameter has no experiment binding")))
            .collect::<Result<Vec<_>, _>>()?;
        Ok((sources, ports))
    }
    /// Compile authored case paths once; trial parameter values never enter source queries.
    pub async fn prepare_fit(
        &self,
        services: &PreparationServices,
        id: FitId,
        profile: FitProfile,
        compiler: Profile,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<PreparedFit, WorkflowError> {
        self.prepare_fit_bound(services, id, profile, compiler, limits, None, cancel)
            .await
    }
    /// Build each point's declared values, numerical premises and fresh experiment ICs together.
    pub(in crate::workflow) async fn prepare_fit_bound(
        &self,
        services: &PreparationServices,
        id: FitId,
        profile: FitProfile,
        compiler: Profile,
        limits: Limits,
        binding: Option<&super::super::AdmittedBinding>,
        cancel: &crate::CancelSource,
    ) -> Result<PreparedFit, WorkflowError> {
        self.prepare_fit_input(
            services,
            id,
            profile,
            compiler,
            limits,
            FitPreparationInput {
                binding,
                pending: None,
            },
            cancel,
        )
        .await
    }
    /// Prepare fitting with physically typed experiment controls under one shared owner.
    pub async fn prepare_physical_fit(
        &self,
        services: &PreparationServices,
        id: FitId,
        request: super::super::PhysicalFitRequest,
        compiler: Profile,
        limits: Limits,
        binding: Option<&super::super::AdmittedBinding>,
        cancel: &crate::CancelSource,
    ) -> Result<PreparedFit, WorkflowError> {
        let request = request.retain(services)?;
        let pending = request.settings.simulations.clone();
        let profiles = pending
            .iter()
            .map(|(id, request)| {
                (
                    *id,
                    request
                        .controls
                        .clone()
                        .with_coordinates(Vec::new(), Vec::new(), Vec::new()),
                )
            })
            .collect();
        let profile = request.settings.with_simulations(profiles).profile()?;
        self.prepare_fit_input(
            services,
            id,
            profile,
            compiler,
            limits,
            FitPreparationInput {
                binding,
                pending: Some(&pending),
            },
            cancel,
        )
        .await
    }
    async fn prepare_fit_input(
        &self,
        services: &PreparationServices,
        id: FitId,
        profile: FitProfile,
        compiler: Profile,
        limits: Limits,
        input: FitPreparationInput<'_>,
        cancel: &crate::CancelSource,
    ) -> Result<PreparedFit, WorkflowError> {
        let FitPreparationInput { binding, pending } = input;
        let preparation_driver = services.math().preparation_driver(cancel)?;
        let cancel = &preparation_driver;
        let mut original_profile = profile.clone();
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                let (mut problem, assessments) = self
                    .prepare_fit_problem_bound(services, id, profile, compiler, limits, FitPreparationInput { binding, pending }, cancel)
                    .await?;
                original_profile.simulations = problem.profile.simulations.clone();
                let original_inputs = crate::workflow::replay::OriginalFitInputs::capture(services, id, &original_profile, compiler, limits, binding, problem.residual_lm_criteria)?;
                if !problem.initial.is_empty() && problem.profile.optimizer != FitOptimizer::NativeNlp {
                    if oracle::residual::eligible(&problem, services.math()) {
                        return Ok(PreparedFit { original_inputs, source: self.clone(),
                            problem: Arc::new(problem), route: FitRoute::ResidualLm, assessments });
                    }
                    if problem.profile.optimizer == FitOptimizer::ResidualLm {
                        return Err(contract("residual LM requires unboxed unconstrained exact response fitting with supported settings and finite matrix extent"));
                    }
                }
                let facts = native::routing::oracle_facts(
                    &problem.contract,
                    true,
                    problem.bounds.iter().all(|(a, b)| a.is_finite() && a == b),
                );
                // Contextual interpretations may clone complete inventories, but share the
                // matching witness already owned by the immutable preparation.
                let extent = native::structural::construction_bytes(
                    &problem.contract,
                    problem.layout.constraints.matrix().symbolic(),
                )
                .map_err(crate::math::MathRuntimeError::from)?;
                let reservation =
                    datafusion::execution::memory_pool::MemoryConsumer::new("fit:route-structure")
                        .register(&services.pool());
                reservation
                    .try_grow(extent)
                    .map_err(crate::math::MathRuntimeError::from)?;
                let decision = native::routing::Requirements {
                    table: &native::execution::LINKED,
                    facts: &facts,
                    intent: problem.profile.solver.intent,
                    numerical_psd: false,
                    least_squares: true,
                    controls: &problem.profile.solver.controls,
                    settings: &problem.profile.solver.backend,
                    sensitivity: false,
                    context: native::routing::Context {
                        snapshot: problem.snapshot.clone(),
                        pending_classes: &[],
                        structure: Some(problem.structure.clone()),
                        oracle: Some(&problem.contract),
                        guards: &BTreeMap::new(),
                        budgets: Some(native::execution::Budgets {
                            tolerances: &problem.tolerances,
                            normalization: &problem.normalization,
                            accuracy: &problem.accuracy,
                        }),
                        coefficients: None,
                        cone: None,
                        factorable: None,
                        certificate: None,
                        prepared: &[native::routing::ArtifactDemand::Representation(
                            native::execution::Representation::Nlp,
                        )],
                        refusals: &BTreeMap::new(),
                    },
                }
                .decision(problem.profile.solver.selection);
                let route = decision
                    .route()
                    .map_err(crate::math::MathRuntimeError::from)?;
                problem.structural_assessment = decision.structure;
                if let Some(assessment) = &mut problem.structural_assessment {
                    let retained = assessment
                        .retained_bytes()
                        .checked_sub(assessment.witness.retained_bytes())
                        .filter(|bytes| *bytes <= reservation.size())
                        .ok_or_else(|| contract("fit structural assessment retained allowance"))?;
                    reservation.shrink(reservation.size() - retained);
                    assessment.witness = assessment
                        .witness
                        .clone()
                        .with_owner(pse_columnar::AllocationLease::new(reservation));
                }
                crate::math::solves::admit_profile(
                    &problem.profile.solver,
                    route,
                    &problem.snapshot,
                )
                .map_err(crate::math::MathRuntimeError::from)?;
                Ok(PreparedFit {
                    original_inputs,
                    source: self.clone(),
                    problem: Arc::new(problem),
                    route: match route { native::routing::Route::Native(backend) => FitRoute::Native(backend),
                        native::routing::Route::Constant => FitRoute::Evaluation },
                    assessments,
                })
            })
            .await
    }
    #[cfg(test)]
    pub(in crate::workflow::fitting) async fn prepare_fit_problem(
        &self,
        services: &PreparationServices,
        id: FitId,
        profile: FitProfile,
        compiler: Profile,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<(FitProblem, Vec<Assessment>), WorkflowError> {
        self.prepare_fit_problem_bound(
            services,
            id,
            profile,
            compiler,
            limits,
            FitPreparationInput {
                binding: None,
                pending: None,
            },
            cancel,
        )
        .await
    }
    async fn prepare_fit_problem_bound(
        &self,
        services: &PreparationServices,
        id: FitId,
        mut profile: FitProfile,
        compiler: Profile,
        limits: Limits,
        input: FitPreparationInput<'_>,
        cancel: &crate::CancelSource,
    ) -> Result<(FitProblem, Vec<Assessment>), WorkflowError> {
        let FitPreparationInput { binding, pending } = input;
        let preparation_driver = services.math().preparation_driver(cancel)?;
        let cancel = &preparation_driver;
        let activity = services.admit_activity()?;
        // Preparation contains both steady and transient child state machines. Keep
        // that state on the heap before activity/context composition so callers
        // carry a bounded future rather than every preparation alternative inline.
        let future_storage =
            datafusion::execution::memory_pool::MemoryConsumer::new("fit:preparation-future")
                .register(&services.pool());
        let preparation = {
            let preparation = async {
                // Charged once the layout bound is known, before the layout is built.
                let reservation =
                    datafusion::execution::memory_pool::MemoryConsumer::new("fit:prepared")
                        .register(&services.pool());
                let mut d = self.fit_declaration(services, id).await?;
                profile
                    .solver
                    .controls
                    .validate()
                    .map_err(crate::math::MathRuntimeError::from)?;
                if profile.solver.controls.start != native::solve::StartPolicy::NoPriorStart
                    || profile.solver.controls.reuse == native::solve::ReusePolicy::RequireReuse
                {
                    return Err(contract(
                        "fitting consumes declared guesses and must permit fresh solver allocation",
                    ));
                }
                // A fit's parameters are its unknowns: what it derives from them is its covariance
                // and intervals, never the parametric sensitivities of a modeling solve.
                if profile.solver.sensitivity.is_some() {
                    return Err(contract(
                        "a fit derives its covariance and intervals (FitProfile.uncertainty), not parametric sensitivities",
                    ));
                }
                if let Some(uncertainty) = &profile.uncertainty {
                    uncertainty.admit()?;
                }
                if profile.solver.intent != SolveIntent::Optimize
                    || !profile.rank_tolerance.is_finite()
                    || !(0.0..1.0).contains(&profile.rank_tolerance)
                    || profile.rank_tolerance == 0.0
                    || profile.max_cells == 0
                    || d.experiments.is_empty()
                    || d.parameters.is_empty()
                {
                    return Err(contract("fit intent, rank policy or inventories"));
                }
                if profile.optimizer == FitOptimizer::ResidualLm
                    && d.parameters.iter().any(|parameter| !parameter.fixed)
                    && !oracle::residual::request_eligible(&profile)
                {
                    return Err(contract(
                        "residual LM cannot implement the requested native-only controls or products",
                    ));
                }
                let free_parameters = d.parameters.iter().filter(|p| !p.fixed).count();
                let residual_rows = d.observations.iter().filter(|o| o.included).count();
                let residual_workspace = oracle::residual::workspace_eligible(
                    residual_rows,
                    free_parameters,
                    profile.max_cells,
                    services
                        .math()
                        .policy()
                        .foreign_allowance(&profile.solver.controls),
                );
                if profile.optimizer == FitOptimizer::ResidualLm
                    && free_parameters > 0
                    && !residual_workspace
                {
                    return Err(contract(
                        "residual LM dense workspace exceeds its known foreign allowance or matrix extent",
                    ));
                }
                if d.parameters
                    .iter()
                    .map(|p| p.symbol_id)
                    .collect::<BTreeSet<_>>()
                    .len()
                    != d.parameters.len()
                    || d.experiments
                        .iter()
                        .map(|e| e.experiment_id)
                        .collect::<BTreeSet<_>>()
                        .len()
                        != d.experiments.len()
                    || d.observations
                        .iter()
                        .map(|o| o.observation_id)
                        .collect::<BTreeSet<_>>()
                        .len()
                        != d.observations.len()
                {
                    return Err(contract(
                        "duplicate fit parameter, experiment or observation binding",
                    ));
                }
                // Resolve ownership before observation paths drive selective preparation.
                // An unbound observation must not disappear from that demand and cause an
                // unrelated empty-layout failure before its actual boundary refusal.
                if d.observations.iter().any(|observation| {
                    !d.experiments
                        .iter()
                        .any(|experiment| experiment.experiment_id == observation.experiment_id)
                }) || profile.simulations.keys().any(|id| {
                    !d.experiments.iter().any(|experiment| {
                        experiment.experiment_id == *id && experiment.route == Route::Integrated
                    })
                }) {
                    return Err(contract("fit observation or dynamic profile ownership"));
                }
                // A supplied Hessian (exact or Gauss–Newton) needs second-order steady models for
                // the constraint curvature. Only the exact Hessian also needs the residual
                // curvature, which a transient experiment takes from IDAS forward-over-adjoint
                // second-order sensitivities over dynamic functions compiled to second order
                // (ADR-0110 item 4).
                profile.solver.controls.hessian = pse_model::strategy::resolve_curvature(
                    profile.solver.controls.hessian,
                    pse_model::strategy::CurvatureFacts {
                        exact: false,
                        least_squares_response: profile.derivatives != FitDerivatives::Gradient,
                    },
                );
                let hessian = profile.solver.controls.hessian;
                // An adjoint gradient carries no response Jacobian, so no supplied Hessian can be
                // formed from it (ADR-0110 item 3).
                if profile.derivatives == FitDerivatives::Gradient
                    && matches!(hessian, HessianMode::Exact | HessianMode::GaussNewton)
                {
                    return Err(contract(
                        "gradient-only fitting cannot supply exact or Gauss-Newton curvature",
                    ));
                }
                let mut order = if !matches!(hessian, HessianMode::Exact | HessianMode::GaussNewton)
                {
                    DerivativeOrder::First
                } else {
                    DerivativeOrder::Second
                };
                let q = &self.quantities;
                // One immutable structural/physical inventory serves admission and fresh point construction.
                let (mut sources, parameter_ports) =
                    self.fit_sources(services, &d, limits, cancel).await?;
                if let Some(binding) = binding {
                    self.validate_fit_source_binding(&d, &sources, binding)?;
                    for parameter in &mut d.parameters {
                        if let Some(entry) = binding.entries.get(&parameter.symbol_id) {
                            parameter.value = entry.canonical.into_inner();
                        }
                    }
                }
                for ((_, case, _, local), experiment) in sources.iter_mut().zip(&d.experiments) {
                    for binding in &experiment.bindings {
                        let parameter = d
                            .parameters
                            .iter()
                            .position(|parameter| parameter.symbol_id == binding.parameter_id)
                            .ok_or_else(|| {
                                contract("experiment binds an undeclared fit parameter")
                            })?;
                        if !local.contains_key(&parameter) {
                            return Err(contract("fit parameter binding inventory differs"));
                        }
                        case.values
                            .insert(binding.path.clone(), d.parameters[parameter].value);
                    }
                }
                // The actual admitted steady inventories prove this residual-only shape before
                // numerical artifact allocation. Transient equations remain internal IVP rows.
                if profile.optimizer != FitOptimizer::NativeNlp
                    && oracle::residual::request_eligible(&profile)
                    && free_parameters > 0
                    && residual_rows >= free_parameters
                    && residual_workspace
                    && d.parameters
                        .iter()
                        .all(|p| p.fixed || p.lower.is_none() && p.upper.is_none())
                    && sources
                        .iter()
                        .zip(&d.experiments)
                        .all(|((_, _, model, _), experiment)| {
                            experiment.route == Route::Integrated || {
                                let structure = model.compiled().admitted.case();
                                structure.variables().iter().all(|v| v.fixed)
                                    && structure
                                        .rows()
                                        .iter()
                                        .all(|r| !r.lower.is_finite() && !r.upper.is_finite())
                            }
                        })
                {
                    order = DerivativeOrder::First;
                }
                let lineage = pse_model::lineage::Fitted::new(
                    d.fit_id,
                    sources.iter().map(|(_, _, model, _)| model.solved()),
                );
                let named = lineage.lineage();
                let mut targets = Vec::new();
                let mut declarations = Vec::new();
                let mut vars = Vec::new();
                let mut initial = Vec::new();
                let mut parameter_columns = Vec::new();
                for (p, port) in d.parameters.iter().zip(&parameter_ports) {
                    if !p.value.is_finite()
                        || !p.scale.is_finite()
                        || p.scale <= 0.0
                        || p.lower.is_some_and(|v| !v.is_finite() || v > p.value)
                        || p.upper.is_some_and(|v| !v.is_finite() || v < p.value)
                        || (!p.fixed && p.lower.is_some() && p.lower == p.upper)
                    {
                        return Err(contract("parameter bounds, scale or fixed decision"));
                    }
                    targets.push(TargetSpec {
                        id: p.symbol_id,
                        kind: NumericalTarget::Variable,
                        quantity: port.quantity,
                        unit: port.unit,
                        integer: false,
                        declared_tolerance: None,
                    });
                    declarations.push(SourcedRequirement {
                        source: NumericalSource::Model,
                        declaration: NumericalRequirement {
                            requirement_id: pse_ids::named_id(
                                d.fit_id.as_id(),
                                &format!("nominal.{}", p.symbol_id),
                            ),
                            model_id: named.model_id,
                            case_id: named.case_id,
                            instance_id: named.instance_id,
                            fit_id: named.fit_id,
                            target_id: p.symbol_id,
                            target_kind: NumericalTarget::Variable,
                            nominal: Some(p.scale),
                            scaling_factor: None,
                            absolute_tolerance: None,
                            shared_engineering_allowance: None,
                            engineering_rule_id: None,
                            relative_tolerance: None,
                            unit_id: Some(port.unit.as_id()),
                            coordinates: NumericalCoordinates::Physical,
                            priority: 0,
                            required: true,
                            provenance: "authored fitting parameter nominal".into(),
                        },
                    });
                    parameter_columns.push(if p.fixed {
                        None
                    } else {
                        let col = OriginalCol::new(vars.len());
                        vars.push(Variable {
                            id: p.symbol_id,
                            lower: p.lower.unwrap_or(f64::NEG_INFINITY),
                            upper: p.upper.unwrap_or(f64::INFINITY),
                        });
                        initial.push(p.value);
                        Some(col)
                    });
                }
                let mut experiments = Vec::new();
                let mut assessments = Vec::new();
                let mut bounds = Vec::new();
                let mut rows = Vec::new();
                let mut measurements = Vec::new();
                let mut bytes = profile
                    .solver
                    .controls
                    .report_allowance()
                    .map_err(crate::math::MathRuntimeError::from)?
                    .checked_add(256 * 1024)
                    .ok_or_else(|| contract("fit report extent"))?;
                let mut physical_cells = 0usize;
                let mut identity = FramedHasher::new(pse_ids::Frame::ModelingFitSourceV2);
                identity.hash(&self.physical.key);
                identity.hash(&self.revision.identity().as_id());
                let mut execution_identity =
                    FramedHasher::new(pse_ids::Frame::ModelingFitExecutionV1);
                for (ei, (e, (bindings, case, model, local))) in
                    d.experiments.iter().zip(sources).enumerate()
                {
                    let local_observations = d
                        .observations
                        .iter()
                        .filter(|o| o.experiment_id == e.experiment_id)
                        .collect::<Vec<_>>();
                    let mut bound_times = BTreeMap::new();
                    let experiment = if e.route == Route::Integrated {
                        let (mut integration, provisional_quadratures) = if let Some(profile) =
                            profile.simulations.get(&e.experiment_id)
                        {
                            (profile.clone(), BTreeSet::new())
                        } else {
                            let data=model.compiled().model.fixtures.get(&e.experiment_id).ok_or_else(||contract("transient experiment needs explicit or authored integration controls"))?;
                            let controls =
                                self.integration_profile(&model, data, &profile.solver.numerics)?;
                            (controls.profile, controls.provisional_quadratures)
                        };
                        for o in &local_observations {
                            let raw = o
                                .time
                                .ok_or_else(|| contract("transient observation needs time"))?;
                            let scale = if let Some(id) = o.time_unit_id {
                                let unit = q.unit(UnitId::from_id(id)).map_err(math)?;
                                if unit.is_affine
                                    || unit.dimension
                                        != pse_quantity::DimensionVector::base(
                                            pse_quantity::BaseDimension::Time,
                                        )
                                {
                                    return Err(contract(
                                        "observation time requires a non-affine time unit",
                                    ));
                                }
                                unit.scale_to_canonical
                            } else {
                                1.
                            };
                            // Generated dynamics uses the authored canonical axis directly, without a second time origin.
                            let time = super::super::time::observation(
                                raw,
                                scale,
                                o.time_basis.unwrap_or(
                                    pse_relations::generated::enums::ObservationTimeBasis::Elapsed,
                                ),
                                0.,
                                integration.start,
                            )?;
                            if !time.is_finite()
                                || time < integration.start
                                || time > integration.end
                            {
                                return Err(contract(
                                    "observation time outside the experiment domain",
                                ));
                            }
                            bound_times.insert(o.observation_id, time);
                        }
                        // Keep requested diagnostic/end samples as well as every included measurement.
                        integration.samples.extend(
                            local_observations
                                .iter()
                                .filter(|o| o.included)
                                .map(|o| bound_times[&o.observation_id]),
                        );
                        integration.samples.sort_by(f64::total_cmp);
                        integration.samples.dedup();
                        // Fitted parameters need forward sensitivities for the response Jacobian,
                        // or the adjoint route for the gradient alone.
                        integration.sensitivity =
                            if local.keys().any(|p| parameter_columns[*p].is_some()) {
                                match profile.derivatives {
                                    FitDerivatives::Responses => {
                                        native::dynamics::DynamicSensitivity::Forward
                                    }
                                    FitDerivatives::Gradient => {
                                        native::dynamics::DynamicSensitivity::Adjoint
                                    }
                                }
                            } else {
                                native::dynamics::DynamicSensitivity::None
                            };
                        // Fitting selects semantic free inputs below. A reused resolved
                        // simulation profile may retain positional axes from its previous
                        // preparation; those are not a second fitting selection.
                        integration.forward_axes = None;
                        // Exact transient Hessians need the dynamic functions' second derivatives.
                        let transient_order = if hessian == HessianMode::Exact {
                            DerivativeOrder::Second
                        } else {
                            DerivativeOrder::First
                        };
                        let free_inputs = local
                            .iter()
                            .filter(|(parameter, _)| parameter_columns[**parameter].is_some())
                            .map(|(_, id)| *id)
                            .collect::<BTreeSet<_>>();
                        let forward_parameters = (integration.sensitivity
                            == native::dynamics::DynamicSensitivity::Forward)
                            .then_some(&free_inputs);
                        // The experiment's authored case declares its modes and events.
                        let mut simulation = self
                            .prepare_simulation_with_controls(
                                services,
                                e.case_id,
                                e.experiment_id,
                                bindings,
                                limits,
                                case,
                                compiler,
                                integration,
                                transient_order,
                                &if hessian == HessianMode::Exact {
                                    free_inputs.clone()
                                } else {
                                    BTreeSet::new()
                                },
                                forward_parameters,
                                &provisional_quadratures,
                                None,
                                None,
                                pending.and_then(|requests| requests.get(&e.experiment_id)),
                                cancel,
                            )
                            .await?;
                        if let Some(request) =
                            pending.and_then(|requests| requests.get(&e.experiment_id))
                        {
                            let admitted = request.controls.clone().with_coordinates(
                                simulation.profile().atol.clone(),
                                simulation.profile().out_atol.clone(),
                                simulation.profile().parameter_scales.clone(),
                            );
                            profile.simulations.insert(e.experiment_id, admitted);
                            simulation.retain_physical_request(request);
                        }
                        execution_identity.hash(&simulation.identity());
                        bytes = bytes
                            .checked_add(simulation.bytes)
                            .ok_or_else(|| contract("fit transient extent"))?;
                        // A fit parameter binds the integration column in effect at the start: a
                        // scheduled input's first interval, whose later intervals keep their
                        // scheduled values (I6).
                        let start = simulation.profile().columns_at(
                            simulation.contract().parameters.len(),
                            simulation.profile().start,
                        );
                        let bindings =
                            local
                                .iter()
                                .map(|(parameter, id)| {
                                    let position = simulation
                            .contract()
                            .parameters
                            .iter()
                            .position(|p| p == id)
                            .ok_or_else(|| {
                                contract("fitted dynamic member is not a sensitivity parameter")
                            })?;
                                    Ok(Binding {
                                        local: start[position],
                                        parameter: *parameter,
                                        conversion: pse_quantity::UnitConvertSpec {
                                            from: parameter_ports[*parameter].unit,
                                            to: parameter_ports[*parameter].unit,
                                            scale: 1.,
                                            offset: 0.,
                                        },
                                    })
                                })
                                .collect::<Result<Vec<_>, WorkflowError>>()?;
                        // The exact Hessian's second-order route is admitted before any native
                        // work: the adjoint profile limits, IDAS (Diffsol has no second-order
                        // adjoint), and its estimate against the math allowance.
                        if hessian == HessianMode::Exact {
                            let mut second = simulation.profile().clone();
                            second.sensitivity = native::dynamics::DynamicSensitivity::Adjoint;
                            second.forward_axes = None;
                            let directions = bindings
                                .iter()
                                .filter(|b| parameter_columns[b.parameter].is_some())
                                .map(|b| b.local)
                                .collect::<Vec<_>>();
                            if !directions.is_empty() {
                                second
                                    .validate(simulation.contract(), &simulation.parameters)
                                    .and_then(|_| {
                                        second.admit_second_order(
                                            simulation.contract(),
                                            &directions,
                                            simulation.native_layout(),
                                        )
                                    })
                                    .map_err(|e| WorkflowError::Math(e.into()))?;
                            }
                        }
                        let output_ports = simulation
                            .contract()
                            .outputs
                            .iter()
                            .map(|id| {
                                let row = simulation
                                    .model()
                                    .compiled()
                                    .admitted
                                    .case()
                                    .rows()
                                    .iter()
                                    .find(|r| r.id == *id)
                                    .ok_or_else(|| contract("dynamic output contract absent"))?;
                                Ok(Port {
                                    id: *id,
                                    quantity: row.quantity,
                                    unit: q
                                        .quantity_type(row.quantity)
                                        .map_err(math)?
                                        .canonical_unit,
                                })
                            })
                            .collect::<Result<Vec<_>, WorkflowError>>()?;
                        physical_cells = physical_cells
                            .checked_add(
                                simulation
                                    .profile()
                                    .samples
                                    .len()
                                    .checked_mul(
                                        simulation.model().compiled().admitted.outputs.len(),
                                    )
                                    .ok_or_else(|| contract("fit check extent"))?,
                            )
                            .ok_or_else(|| contract("fit check extent"))?;
                        let result = Experiment::Transient(Box::new(IntegratedExperiment {
                            snapshot: simulation.snapshot().clone(),
                            program: simulation.program(),
                            snapshots: simulation.snapshot_allocator(services.math()),
                            profile: simulation.profile().clone(),
                            parameters: simulation.parameters.clone(),
                            output_ports,
                            bindings,
                        }));
                        assessments.push(Assessment::Transient(Box::new(simulation)));
                        result
                    } else {
                        if profile.simulations.contains_key(&e.experiment_id) {
                            return Err(contract("algebraic experiment has a dynamic profile"));
                        }
                        // The experiment supplies objective-free feasibility callbacks. The
                        // outer fit owns the least-squares objective and its optimization intent.
                        let mut experiment_solver = profile.solver.clone();
                        experiment_solver.intent = SolveIntent::FeasiblePoint;
                        let resolved = self
                            .resolve_case(
                                services,
                                e.case_id,
                                e.experiment_id,
                                bindings,
                                limits,
                                case,
                                order,
                                compiler,
                                experiment_solver,
                                Default::default(),
                                Default::default(),
                                false,
                                cancel,
                            )
                            .await?;
                        let model = resolved.model.model.clone();
                        let source = resolved.model.case.compiled().plan.structure();
                        if source.objective().is_some() {
                            return Err(contract("fit experiment has an authored objective"));
                        }
                        execution_identity.hash(&source.key());
                        let mut values = resolved.model.values.clone();
                        let mut coordinates = TiVec::<GlobalCol, _>::new();
                        // ADR-0103 item 6: estimation refuses a discrete variable the case leaves free.
                        model
                            .compiled()
                            .model
                            .require_fixed_discrete(
                                source
                                    .variables()
                                    .iter()
                                    .filter(|v| !v.fixed)
                                    .map(|v| v.port.id),
                                pse_modeling::DomainAnalysis::Fitting,
                            )
                            .map_err(crate::workflow::modeling_error)?;
                        for v in source.variables().iter().filter(|v| !v.fixed) {
                            targets.push(TargetSpec {
                                id: alias(e.experiment_id, v.port.id),
                                kind: NumericalTarget::Variable,
                                quantity: v.port.quantity,
                                unit: v.port.unit,
                                integer: false,
                                declared_tolerance: None,
                            });
                            let col = vars.len();
                            vars.push(Variable {
                                id: alias(e.experiment_id, v.port.id),
                                lower: v.lower.unwrap_or(f64::NEG_INFINITY),
                                upper: v.upper.unwrap_or(f64::INFINITY),
                            });
                            initial.push(
                                *values
                                    .scalars
                                    .get(&v.port.id)
                                    .ok_or_else(|| contract("missing experiment initial state"))?,
                            );
                            coordinates.push((v.port.id, OriginalCol::new(col)));
                        }
                        let local_states = coordinates.len();
                        for (parameter, id) in &local {
                            values.scalars.insert(*id, d.parameters[*parameter].value);
                            if let Some(col) = parameter_columns[*parameter] {
                                coordinates.push((*id, col));
                            }
                        }
                        for (id, value) in &values.scalars {
                            execution_identity.id(id).u64(value.to_bits());
                        }
                        let mut outputs = source
                            .rows()
                            .iter()
                            .filter(|r| r.lower.is_finite() || r.upper.is_finite())
                            .map(|r| r.id)
                            .collect::<BTreeSet<_>>();
                        for o in &local_observations {
                            if o.included {
                                outputs.insert(
                                    ModelingOutput::Member(member(&model, &o.output_path)?)
                                        .row_id(),
                                );
                            }
                        }
                        let case = services
                            .math()
                            .prepare_modeling_functions(
                                self.numerical_workspace(services)?,
                                model.clone(),
                                outputs.into_iter().collect(),
                                coordinates.iter().map(|v| v.0).collect(),
                                order,
                                compiler,
                                cancel,
                            )
                            .await?;
                        let mut constraints = Vec::new();
                        // Function projection owns derivatives; the admitted case owns row bounds.
                        for (i, r) in case.assembly.structure().rows().iter().enumerate() {
                            if let Some(original) = source.rows().iter().find(|o| {
                                o.id == r.id && (o.lower.is_finite() || o.upper.is_finite())
                            }) {
                                targets.push(TargetSpec {
                                    id: alias(e.experiment_id, r.id),
                                    kind: NumericalTarget::Row,
                                    quantity: r.quantity,
                                    unit: q.quantity_type(r.quantity).map_err(math)?.canonical_unit,
                                    integer: false,
                                    declared_tolerance: None,
                                });
                                constraints.push((GlobalRow::new(i), OriginalRow::new(rows.len())));
                                rows.push(alias(e.experiment_id, r.id));
                                bounds.push((original.lower, original.upper));
                            }
                        }
                        for mut r in resolved.numerical.declarations.clone() {
                            r.declaration.target_id =
                                alias(e.experiment_id, r.declaration.target_id);
                            r.declaration.requirement_id =
                                alias(e.experiment_id, r.declaration.requirement_id);
                            if targets.iter().any(|t| {
                                t.id == r.declaration.target_id
                                    && t.kind == r.declaration.target_kind
                            }) {
                                declarations.push(r);
                            }
                        }
                        let check_rows = checks::observation_rows(model.compiled(), &values)?;
                        let program = if check_rows.is_empty() {
                            None
                        } else {
                            Some(
                                services
                                    .math()
                                    .prepare_modeling_functions(
                                        self.numerical_workspace(services)?,
                                        model.clone(),
                                        check_rows.into_iter().collect(),
                                        vec![],
                                        DerivativeOrder::Value,
                                        compiler,
                                        cancel,
                                    )
                                    .await?,
                            )
                        };
                        let numerics = resolved.numerics;
                        physical_cells = physical_cells
                            .checked_add(model.compiled().admitted.outputs.len())
                            .ok_or_else(|| contract("fit check extent"))?;
                        bytes = bytes
                            .checked_add(case.assembly.numeric_worker_bytes())
                            .and_then(|n| {
                                n.checked_add(
                                    program
                                        .as_ref()
                                        .map_or(0, |p| p.assembly.numeric_worker_bytes()),
                                )
                            })
                            .ok_or_else(|| contract("fit worker extent"))?;
                        assessments.push(Assessment::Steady {
                            model: Box::new(model),
                            program,
                            numerics,
                        });
                        Experiment::Steady(Steady {
                            variables: source.variables().to_vec(),
                            case,
                            values,
                            coordinates,
                            constraints,
                            local_states,
                            providers: resolved.providers,
                        })
                    };
                    for binding in local_observations {
                        let observation_revision = self
                            .selected_revision_roots(
                                services,
                                &[e.case_id, binding.observation_id],
                                cancel,
                            )
                            .await?;
                        let observation = observation_revision
                            .checked()
                            .measurement(
                                e.case_id,
                                binding.observation_id,
                                &binding.value_attribute,
                                binding.standard_deviation_attribute.as_deref(),
                            )
                            .map_err(|error| contract(error.to_string()))?;
                        let output =
                            ModelingOutput::Member(member(&model, &binding.output_path)?).row_id();
                        let (row, port) = match &experiment {
                            Experiment::Steady(s) => {
                                if binding.time.is_some()
                                    || binding.time_basis.is_some()
                                    || binding.time_unit_id.is_some()
                                {
                                    return Err(contract(
                                        "algebraic observation has a time coordinate",
                                    ));
                                }
                                let r = model
                                    .compiled()
                                    .admitted
                                    .case()
                                    .rows()
                                    .iter()
                                    .find(|r| r.id == output)
                                    .ok_or_else(|| contract("unknown observed member"))?;
                                (
                                    s.case
                                        .assembly
                                        .structure()
                                        .rows()
                                        .iter()
                                        .position(|r| r.id == output)
                                        .unwrap_or(usize::MAX),
                                    Port {
                                        id: output,
                                        quantity: r.quantity,
                                        unit: q
                                            .quantity_type(r.quantity)
                                            .map_err(math)?
                                            .canonical_unit,
                                    },
                                )
                            }
                            Experiment::Transient(s) => {
                                let i=s.output_ports.iter().position(|p|p.id==output).ok_or_else(||contract("transient observations require a state or an authored report member"))?;
                                (i, s.output_ports[i].clone())
                            }
                        };
                        if observation.quantity != port.quantity {
                            return Err(contract(format!(
                                "measured attribute {}.{} has quantity {}, observed output {} requires {}",
                                binding.observation_id,
                                binding.value_attribute,
                                observation.quantity.as_id(),
                                binding.output_path,
                                port.quantity.as_id()
                            )));
                        }
                        let value = observation.value;
                        let sigma = observation.standard_deviation;
                        if !binding.importance.is_finite()
                            || binding.importance <= 0.
                            || value.is_some_and(|v| !v.is_finite())
                            || sigma.is_some_and(|s| !s.is_finite() || s <= 0.)
                            || sigma.is_some_and(|s| !(binding.importance.sqrt() / s).is_finite())
                            || (binding.included && (value.is_none() || sigma.is_none()))
                        {
                            return Err(contract(
                                "included observations require finite values, positive difference-unit standard deviations and importance",
                            ));
                        }
                        measurements.push(Measurement {
                            id: binding.observation_id.as_id(),
                            experiment: ei,
                            row,
                            time: bound_times.get(&binding.observation_id).copied(),
                            sample_index: match &experiment {
                                Experiment::Transient(s) if binding.included => Some(
                                    s.profile
                                        .samples
                                        .binary_search_by(|t| {
                                            t.total_cmp(&bound_times[&binding.observation_id])
                                        })
                                        .map_err(|_| contract("unbound observation sample"))?,
                                ),
                                _ => None,
                            },
                            included: binding.included,
                            value,
                            sigma,
                            importance: binding.importance,
                            port,
                        });
                    }
                    experiments.push(experiment);
                }
                // The predictions' propagated covariance is bounded like the dense diagnostics.
                let included = measurements.iter().filter(|o| o.included).count();
                if profile.uncertainty.as_ref().is_some_and(|u| u.predictions)
                    && included
                        .checked_mul(included)
                        .is_none_or(|cells| cells > profile.max_cells)
                {
                    return Err(contract(
                        "the predictions' propagated covariance exceeds the fit's max_cells",
                    ));
                }
                if measurements.len() != d.observations.len()
                    || !measurements.iter().any(|o| o.included)
                {
                    return Err(contract("fit observation or dynamic profile ownership"));
                }
                let prepared = PreparedExperiments {
                    execution_identity: execution_identity.finish_hash(),
                    declaration: d,
                    lineage,
                    profile,
                    order,
                    variables: vars,
                    rows,
                    bounds,
                    initial,
                    parameter_ports,
                    parameter_columns,
                    experiments,
                    measurements,
                    targets,
                    declarations,
                    bytes,
                    physical_cells,
                };
                let control = pse_diagnostics::Cancellation::default();
                let retained_services = services.clone();
                let quantities = q.clone();
                let preconditions = self.physical.preconditions.clone();
                let source = identity.finish_hash();
                let physical_owner: Arc<dyn pse_math::AllocationOwner> =
                    Arc::new(self.physical.clone());
                // The transferred reservation owns construction storage; the canonical job
                // supplies the bounded worker, CPU admission and native cancellation boundary.
                let operation = services.math().job(1, 0, control.clone(), move |flag| {
                    Ok(prepared.finish(
                        retained_services,
                        quantities,
                        preconditions,
                        source,
                        physical_owner,
                        reservation,
                        &flag.flag(),
                    ))
                });
                tokio::pin!(operation);
                let problem = tokio::select! {
                    result = &mut operation => result.map_err(WorkflowError::from)??,
                    () = cancel.cancelled() => {
                        control.cancel();
                        let _ = operation.await;
                        return Err(crate::math::MathRuntimeError::Cancelled.into());
                    }
                };
                Ok((problem, assessments))
            };
            future_storage
                .try_grow(size_of_val(&preparation))
                .map_err(crate::math::MathRuntimeError::from)?;
            Box::pin(preparation)
        };
        let result = activity.scope(preparation).await;
        drop(future_storage);
        result
    }
}
impl PreparedFit {
    pub(crate) fn execute(
        &self,
        services: &ExecutionServices,
        run_id: RunId,
        scope: pse_kernels::ExecutionScope,
        progress: Arc<native::solve::Progress>,
        workers: usize,
        storage: Option<Arc<pse_columnar::AllocationLease>>,
    ) -> Result<FitReport, crate::math::MathRuntimeError> {
        let problem = &self.problem;
        let (mut report, trace) = crate::math::strategy::target::callable(
            crate::math::strategy::target::CallContext {
                service: services.preparation().math(),
                scope: &scope,
                storage,
            },
            self.callable_source(),
            |solver, admission| {
                self.execute_original(
                    services.preparation().math(),
                    run_id,
                    scope.clone(),
                    progress.clone(),
                    workers,
                    solver.unwrap_or(&problem.profile.solver),
                    admission,
                )
            },
            |report| report.solve.as_ref(),
            |report| {
                report.residual.as_ref().map_or_else(
                    || {
                        report.solve.as_ref().map_or(
                            pse_model::generated::enums::NumericalAttemptObservation::Converged,
                            crate::math::strategy::observe_native,
                        )
                    },
                    FitResidualReport::observation,
                )
            },
            |report, observed| {
                let original_failure = report
                    .validation_error
                    .as_ref()
                    .map(crate::math::strategy::target::assessment_failure)
                    .or_else(|| {
                        report
                            .diagnostic
                            .as_ref()
                            .filter(|d| {
                                matches!(
                                    d.rule,
                                    FitRule::FinalEvaluation | FitRule::ObjectiveOverflow
                                )
                            })
                            .map(|d| d.cause.clone())
                    });
                let policy = &problem.numerics.policy;
                let original = super::super::numerics::original_completion(
                    report.candidate_use(policy),
                    super::super::numerics::CompletionEvidence::point(
                        &report.checks,
                        report.checks_complete && report.validation_error.is_none(),
                        self.required_closure_checks(),
                    ),
                    policy,
                );
                let mut assessment = crate::math::strategy::target::original_assessment(
                    original.decision,
                    original_failure,
                    report
                        .solve
                        .as_ref()
                        .and_then(crate::math::strategy::cause_native)
                        .or_else(|| {
                            report
                                .residual
                                .as_ref()
                                .and_then(|r| r.failure.as_ref())
                                .map(ProfileFailure::shared_cause)
                        }),
                    observed,
                );
                assessment.retention.candidate = report.completion.as_ref().map_or_else(
                    || {
                        super::super::numerics::refused(
                            pse_model::generated::enums::CandidateRefusal::ModelChecks,
                        )
                    },
                    |completion| completion.decision.clone(),
                );
                assessment
            },
        )?;
        report.strategy = Some(trace);
        Ok(report)
    }
    fn execute_original(
        &self,
        math: &Arc<MathService>,
        run_id: RunId,
        scope: pse_kernels::ExecutionScope,
        progress: Arc<native::solve::Progress>,
        workers: usize,
        solver: &SolverProfile,
        admission: Arc<crate::math::strategy::admission::TaskAdmission>,
    ) -> Result<FitReport, crate::math::MathRuntimeError> {
        let (mut report, execution) = self.problem.execute_profile(
            math,
            solver,
            self.route,
            scope.clone(),
            progress,
            Some(admission),
        )?;
        let Some(candidate) = report.candidate.as_ref() else {
            report.completion = Some(self.assess_completion(&report));
            return Ok(report);
        };
        match self.assess_candidate(run_id, candidate, &report.trajectories, &scope) {
            Ok((checks, reports)) => {
                report.checks = checks;
                report.reports = reports;
                report.checks_complete = true;
            }
            Err(error) => report.validation_error = Some(error.boundary_diagnostic()),
        }
        report.completion = Some(self.assess_completion(&report));
        if let Some(uncertainty) = &self.problem.profile.uncertainty
            && let Some(controls) = &uncertainty.profile
            && report.covariance.is_some()
        {
            report.profiles = Some(native::execution::original_scoped(&execution, || {
                self.profiles(
                    math,
                    run_id,
                    &report,
                    uncertainty.level.into_inner(),
                    controls,
                    (self.route, workers),
                    &execution,
                )
            }));
        }
        Ok(report)
    }
    /// Assess the authored source obligations at an independently observed candidate.
    /// Profile pins and the original estimate share this exact physical interpretation.
    pub(super) fn assess_candidate(
        &self,
        run_id: RunId,
        candidate: &[f64],
        trajectories: &BTreeMap<InstanceId, Arc<native::dynamics::Report>>,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<
        (
            Vec<super::super::ModelingCheck>,
            Vec<super::super::ModelingReport>,
        ),
        WorkflowError,
    > {
        let assess = || {
            if self.assessments.len() != self.problem.experiments.len() {
                return Err(contract("fit assessment sources are incomplete"));
            }
            let checkpoint = || {
                scope.check().map_err(|error| {
                    WorkflowError::from(crate::math::MathRuntimeError::from(
                        native::ProblemError::Provider(error),
                    ))
                })
            };
            checkpoint()?;
            let mut rows = Vec::new();
            let mut reports = Vec::new();
            for (ei, (source, experiment)) in self
                .assessments
                .iter()
                .zip(&self.problem.experiments)
                .enumerate()
            {
                match (source, experiment) {
                    (
                        Assessment::Steady {
                            model,
                            program,
                            numerics,
                        },
                        Experiment::Steady(s),
                    ) => {
                        let mut values = s.values.clone();
                        for (id, col) in &s.coordinates {
                            values.scalars.insert(*id, candidate[col.get()]);
                        }
                        let mut applicability = Vec::new();
                        let observed = if let Some(program) = program {
                            let providers = crate::math::attempt_providers(&s.providers, scope)
                                .map_err(|e| {
                                    WorkflowError::from(crate::math::MathRuntimeError::from(
                                        native::ProblemError::Provider(e),
                                    ))
                                })?;
                            let mut worker =
                                program.assembly.worker_scoped(providers, scope.clone());
                            let outputs = worker.constraints(&values).map_err(math)?;
                            let observed = program
                                .assembly
                                .structure()
                                .rows()
                                .iter()
                                .map(|r| r.id)
                                .zip(outputs.iter().copied())
                                .collect();
                            applicability = worker.applicability_observations();
                            observed
                        } else {
                            BTreeMap::new()
                        };
                        let (checks, observations) = checks::assess_observations(
                            run_id,
                            model.compiled(),
                            &values,
                            &observed,
                            &applicability,
                            numerics,
                            &self.problem.quantities,
                            true,
                            None,
                            None,
                        )?;
                        rows.extend(checks);
                        reports.extend(observations);
                    }
                    (Assessment::Transient(simulation), Experiment::Transient(s)) => {
                        let mut parameters = s.parameters.clone();
                        for binding in &s.bindings {
                            let value = self.problem.parameter_columns[binding.parameter].map_or(
                                self.problem.declaration.parameters[binding.parameter].value,
                                |col| candidate[col.get()],
                            );
                            parameters[binding.local] =
                                value * binding.conversion.scale + binding.conversion.offset;
                        }
                        let trajectory = trajectories
                            .get(&self.problem.declaration.experiments[ei].experiment_id)
                            .ok_or_else(|| contract("final original trajectory unavailable"))?;
                        let checks =
                            simulation.check_samples(run_id, trajectory, &parameters, scope);
                        if let Some(error) = checks.error {
                            return Err(WorkflowError::Boundary(Box::new(error)));
                        }
                        if !checks.complete {
                            return Err(contract("original trajectory assessment incomplete"));
                        }
                        rows.extend(checks.rows);
                        reports.extend(checks.reports);
                    }
                    _ => return Err(contract("fit assessment source mismatch")),
                }
            }
            checkpoint()?;
            Ok((rows, reports))
        };
        crate::workflow::dynamics::analysis("fit.original_physical_checks", assess)
    }
}

#[cfg(test)]
mod parameter_point_tests {
    use super::*;
    use crate::workflow::tests::{compiler_profile, id};
    use crate::workflow::{BindingAssignment, BindingQuantity, BindingTarget, PointOverlay};

    #[tokio::test]
    async fn fit_parameter_points_preserve_roles_fresh_starts_units_and_replay() {
        let runtime = crate::workflow::tests::runtime();
        let services = runtime.execution_services();
        let original = regression::package(&runtime, regression::declared(), [1.; 4]).await;
        let mut physical = original.physical.clone();
        let quantity = physical.quantities.neutral_dimensionless().unwrap();
        let canonical = physical
            .quantities
            .quantity_type(quantity)
            .unwrap()
            .canonical_unit;
        let mut percent = physical.quantities.unit(canonical).unwrap().clone();
        percent.id = UnitId::from_id(pse_ids::named_id(
            canonical.as_id(),
            "fit-parameter-percent",
        ));
        percent.symbol = "fit-percent".into();
        percent.scale_to_canonical = 0.01;
        // This representation is a new atom, rather than the canonical unit's empty product.
        percent.definition = None;
        let percent_id = percent.id;
        let mut builder = physical.quantities.to_builder();
        builder.unit(percent);
        physical.quantities = Arc::new(builder.build().unwrap());
        physical.key = pse_compiler::workspace::physical_identity(
            &physical.quantities,
            &physical.preconditions,
        );
        let mut declarations = original
            .fit_declarations(services.preparation())
            .await
            .unwrap();
        declarations.fits[0].parameters[0].fixed = true;
        declarations.fits[0].parameters[1].lower = None;
        declarations.fits[0].parameters[1].upper = None;
        let package = runtime
            .modeling_package(
                original
                    .declarations(services.preparation())
                    .await
                    .unwrap()
                    .to_vec(),
                physical,
            )
            .await
            .unwrap()
            .with_fit_declarations(&services.source_services(), declarations.clone())
            .await
            .unwrap();
        let fit = id(32).into();
        let overlay = |fixed, free, unit| PointOverlay {
            assignments: vec![
                BindingAssignment {
                    target: BindingTarget::Member(id(1)),
                    value: BindingQuantity {
                        magnitude: pse_model::scalars::FiniteBound::try_new(fixed).unwrap(),
                        quantity,
                        unit,
                    },
                },
                BindingAssignment {
                    target: BindingTarget::Member(id(2)),
                    value: BindingQuantity {
                        magnitude: pse_model::scalars::FiniteBound::try_new(free).unwrap(),
                        quantity,
                        unit,
                    },
                },
            ],
        };
        let mut profile = regression::profile(HessianMode::GaussNewton, None);
        profile.optimizer = FitOptimizer::ResidualLm;
        profile.solver.selection = native::solve::SolverSelection::Auto;
        let cancel = crate::CancelSource::new();
        let canonical_binding = package
            .admit_fit_overlay(
                services.preparation(),
                fit,
                &overlay(2., 3., canonical),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        let operation = crate::workflow::StudyOperation {
            version: pse_model::document::Version,
            source: crate::workflow::OperationSource::of(&package),
            preparation: crate::workflow::PreparationSettings {
                compiler: compiler_profile(),
                limits: Limits::default(),
            },
            operation: crate::workflow::OperationRequest::Fit(crate::workflow::FitOperation {
                fit,
                settings: crate::workflow::PhysicalFitRequest::new(FitPreparationDocument {
                    optimizer: FitOptimizer::ResidualLm,
                    solver: crate::math::settings::SolveSettings {
                        intent: SolveIntent::Optimize,
                        controls: native::solve::Controls {
                            hessian: HessianMode::GaussNewton,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    simulations: BTreeMap::new(),
                    rank_tolerance: 1e-8,
                    max_cells: 100000,
                    derivatives: FitDerivatives::Responses,
                    experiment_workers: pse_model::scalar!(PositiveCount(1)),
                    uncertainty: None,
                }),
            }),
            output: crate::workflow::StudyOutputDemand::Full {},
            admitted_horizon: None,
        };
        let (converted, seed_need) = package
            .admit_operation_overlay(
                services.preparation(),
                &operation,
                &overlay(200., 300., percent_id),
                &mut None,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(seed_need, pse_model::study::SeedNeed::NotNeeded);
        assert_eq!(converted.identity(), canonical_binding.identity());
        let crate::workflow::PreparedStudyOperation::Fit(first) = package
            .prepare_bound_operation(services.preparation(), &operation, &converted, &cancel)
            .await
            .unwrap()
        else {
            panic!("fit point");
        };
        assert_eq!(first.problem.initial, vec![3.]);
        assert!(first.declaration().parameters[0].fixed);
        assert!(!first.declaration().parameters[1].fixed);
        assert_eq!(
            first.declaration().parameters[0].lower,
            declarations.fits[0].parameters[0].lower
        );
        assert_eq!(
            first.declaration().parameters[1].scale,
            declarations.fits[0].parameters[1].scale
        );
        let mut fresh_declarations = declarations.clone();
        fresh_declarations.fits[0].parameters[0].value = 2.;
        fresh_declarations.fits[0].parameters[1].value = 3.;
        let fresh_source = package
            .clone()
            .with_fit_declarations(&services.source_services(), fresh_declarations)
            .await
            .unwrap();
        let fresh = fresh_source
            .prepare_fit(
                services.preparation(),
                fit,
                profile.clone(),
                compiler_profile(),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(first.problem.initial, fresh.problem.initial);
        assert_eq!(
            serde_json::to_value(first.declaration()).unwrap(),
            serde_json::to_value(fresh.declaration()).unwrap()
        );
        let next_binding = package
            .admit_fit_overlay(
                services.preparation(),
                fit,
                &overlay(4., 5., canonical),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        let next = package
            .prepare_fit_bound(
                services.preparation(),
                fit,
                profile.clone(),
                compiler_profile(),
                Limits::default(),
                Some(&next_binding),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(next.problem.initial, vec![5.]);
        assert_eq!(first.problem.initial, vec![3.]);
        let retry = package
            .prepare_fit_bound(
                services.preparation(),
                fit,
                profile,
                compiler_profile(),
                Limits::default(),
                Some(&converted),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(retry.identity(), first.identity());
        assert_eq!(retry.problem.initial, vec![3.]);
        let constructor = first.original_inputs.document(0);
        assert_eq!(constructor.binding.as_ref(), Some(&converted));
        let replay = constructor
            .reconstruct(services.preparation(), &package, &cancel)
            .await
            .unwrap();
        assert_eq!(replay.identity(), first.identity());
        assert_eq!(replay.problem.initial, first.problem.initial);
        let mut ambiguous = overlay(2., 3., canonical);
        ambiguous.assignments[0].target = BindingTarget::Path("a".into());
        assert_eq!(
            package
                .admit_fit_overlay(
                    services.preparation(),
                    fit,
                    &ambiguous,
                    Limits::default(),
                    &cancel
                )
                .await
                .unwrap_err()
                .boundary_diagnostic()
                .class,
            pse_model::diagnostic::BoundaryClass::Unsupported
        );
    }
}
