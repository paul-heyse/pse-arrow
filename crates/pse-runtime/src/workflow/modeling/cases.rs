// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Case interpretation delegates expression execution, numerical policy and solving to their owners.
use super::*;
use crate::math::{
    modeling::ModelingCasePreparation,
    solves::{NumericalInputs, ParametricPreparation, PreparedSolve, SolverProfile},
};
use pse_compiler::workspace::{
    ModelingCaseBindings, ModelingHint, ModelingOutput, ModelingVariableState, Profile,
};
use pse_kernels::DerivativeOrder;
use pse_math::binding::CaseValues;
use pse_model::generated::enums::{NumericalCoordinates, NumericalSource, NumericalTarget};
use pse_modeling::{DomainAnalysis, annotation::AnnotationValue};
use std::collections::BTreeSet;

/// Selected values retain their allocation owner until the last reader drops them.
#[derive(Clone, Debug)]
pub struct ModelingObservations {
    /// Typed evidence retained on demanded paths at this exact bound point.
    pub applicability: Vec<pse_model::applicability::Observation>,
    values: BTreeMap<SemanticId, f64>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl std::ops::Deref for ModelingObservations {
    type Target = BTreeMap<SemanticId, f64>;
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
/// Where a resolved input's value came from (F14). Workflows branch on this type, never
/// on a label.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StartSource {
    /// The model's declared initial value.
    ModelDefault,
    /// An explicit case value at this path.
    Case {
        /// Authored case path.
        path: String,
    },
    /// A contextually admitted study assignment, independent of source spelling.
    Binding {
        /// Stable selected member.
        member: SemanticId,
    },
    /// A start annotation evaluated through the model.
    Annotation {
        /// The start annotation's declaration.
        declaration: DeclarationId,
    },
    /// The accepted result of a predecessor point.
    Predecessor,
    /// A continuation parameter override.
    Continuation,
    /// A seed read from the operational store's solution store (ADR-0112 Outcome 17).
    Stored {
        /// The stored solution; the seed's content identity enters lineage (F25).
        solution: SemanticId,
    },
    /// A temporary fix of one step, such as an initialization's discrete assignment
    /// (ADR-0103 item 6).
    Fixed,
}
/// What one step composes over its case by identity (A6): a predecessor's solved values,
/// parameter replacements and temporary fixes. Nothing here outlives the step (PS-08).
#[derive(Clone, Debug, Default)]
pub(in crate::workflow) struct CaseOverrides {
    /// Solved values of an earlier step, which start the free variables.
    pub seed: BTreeMap<SemanticId, f64>,
    /// Replacements of declared parameters, such as continuation values.
    pub parameters: BTreeMap<SemanticId, f64>,
    /// Variables the step holds fixed, at these values.
    pub fixes: BTreeMap<SemanticId, f64>,
}
/// One immutable case plus the numerical declarations selected for this analysis.
#[derive(Clone, Debug)]
pub struct ModelingSolvePreparation {
    pub(in crate::workflow) stored_seed_owner: Option<Arc<dyn pse_columnar::PayloadOwner>>,
    /// The case's model view, solver projection and input values.
    pub model: ModelingCasePreparation,
    /// Routed solve with its numerical policy and representation.
    pub solve: PreparedSolve,
    /// Typed case/default/start provenance for every resolved input.
    pub starts: BTreeMap<SemanticId, StartSource>,
    pub(in crate::workflow) providers:
        BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    pub(in crate::workflow) source: ModelingPackage,
    pub(in crate::workflow) compiler: Profile,
    pub(in crate::workflow) profile: SolverProfile,
}
/// Numerical resolution precedes native routing so diagnostics can inspect an
/// underdetermined or otherwise ineligible problem without requesting a solver.
pub(in crate::workflow) struct ModelingCaseResolution {
    pub(super) case_bindings: ModelingCaseBindings,
    pub compiler: Profile,
    pub model: ModelingCasePreparation,
    pub starts: BTreeMap<SemanticId, StartSource>,
    pub providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    pub numerical: NumericalInputs,
    pub solver: SolverProfile,
    pub numerics: Arc<pse_model::numerics::ResolvedNumericalPolicy>,
    /// The parametric program of the solver's sensitivity request (Plan 22 S1).
    pub parametric: Option<ParametricPreparation<Arc<crate::math::ExecutableCase>>>,
}
impl ModelingCaseResolution {
    /// Canonical identity of the bound request and contextual native admission before an attempt exists.
    pub(in crate::workflow) fn admission_identity(
        &self,
        decision: &pse_backend_native::routing::Decision,
    ) -> Result<pse_ids::ContentHash, WorkflowError> {
        admission_identity(&self.model, &self.numerics, &self.solver, decision, None)
    }

    /// Assess the same complete original compiler witness used by solve admission, without executing a solver.
    pub(in crate::workflow) fn route_decision(
        &self,
    ) -> Result<pse_backend_native::routing::Decision, WorkflowError> {
        self.assess_route(None)
    }
    pub(in crate::workflow) fn assess_route(
        &self,
        lexicographic: Option<bool>,
    ) -> Result<pse_backend_native::routing::Decision, WorkflowError> {
        let compiled = self.model.case.compiled();
        let oracle = pse_backend_native::assembled::contract(&compiled.plan);
        let rows: Vec<_> = compiled
            .plan
            .structure()
            .rows()
            .iter()
            .map(|row| row.id)
            .collect();
        let normalization = pse_math::normalization::Normalization::from_policy(
            &self.numerics,
            compiled.plan.columns(),
            &rows,
        )
        .map_err(crate::math::MathRuntimeError::from)?;
        let tolerances = pse_backend_native::quality::Tolerances::from_policy(
            &self.numerics,
            compiled.plan.columns(),
            &rows,
        )
        .map_err(crate::math::MathRuntimeError::from)?;
        let accuracy = pse_backend_native::solve::ResolvedAccuracy::resolve(
            &self.numerics.policy,
            &tolerances,
            &normalization,
        )
        .map_err(crate::math::MathRuntimeError::from)?;
        let coefficients = compiled
            .coefficients
            .as_ref()
            .map(|coefficients| {
                pse_backend_native::CoefficientProblem::from_plan(
                    &compiled.plan,
                    coefficients.as_ref().clone(),
                )
            })
            .transpose()
            .map_err(crate::math::MathRuntimeError::from)?;
        let pending_classes = pse_backend_native::routing::pending_class_evidence(
            &compiled.facts,
            self.solver.intent,
            self.solver.selection,
        );
        let requirements = pse_backend_native::routing::Requirements {
            table: &pse_backend_native::execution::LINKED,
            facts: &compiled.facts,
            intent: self.solver.intent,
            numerical_psd: false,
            least_squares: false,
            controls: &self.solver.controls,
            settings: &self.solver.backend,
            sensitivity: self.solver.sensitivity.is_some(),
            context: pse_backend_native::routing::Context {
                snapshot: pse_backend_native::execution::Snapshot::observe(
                    &pse_backend_native::execution::LINKED,
                ),
                pending_classes: &pending_classes,
                structure: Some(pse_backend_native::routing::Structure {
                    variables: compiled.plan.columns().to_vec(),
                    equations: compiled
                        .plan
                        .structure()
                        .rows()
                        .iter()
                        .map(|row| pse_structural::incidence::Constraint {
                            id: row.id,
                            lower: row.lower.is_finite().then_some(row.lower),
                            upper: row.upper.is_finite().then_some(row.upper),
                        })
                        .collect(),
                    witness: self.model.case.structural_witness(),
                }),
                oracle: Some(&oracle),
                guards: &compiled.presolve.signs,
                budgets: Some(pse_backend_native::execution::Budgets {
                    tolerances: &tolerances,
                    normalization: &normalization,
                    accuracy: &accuracy,
                }),
                coefficients: coefficients.as_ref(),
                cone: None,
                factorable: None,
                certificate: None,
                prepared: &[],
                refusals: &BTreeMap::new(),
            },
        };
        Ok(match lexicographic {
            Some(single_nonzero) => {
                requirements.lexicographic(self.solver.selection, single_nonzero)
            }
            None => requirements.decision(self.solver.selection),
        })
    }
}
impl ModelingSolvePreparation {
    /// Identity of the bound model, numerical policy and retained native admission.
    pub fn admission_identity(&self) -> Result<pse_ids::ContentHash, WorkflowError> {
        let decision = self
            .solve
            .route_decision()
            .ok_or_else(|| contract("algebraic preparation admission facts absent"))?;
        admission_identity(
            &self.model,
            self.solve.numerics(),
            &self.profile,
            decision,
            Some(
                self.solve
                    .preparation_identity()
                    .map_err(crate::math::MathRuntimeError::from)?,
            ),
        )
    }
}
fn admission_identity(
    model: &ModelingCasePreparation,
    numerics: &pse_model::numerics::ResolvedNumericalPolicy,
    solver: &SolverProfile,
    decision: &pse_backend_native::routing::Decision,
    selected_preparation: Option<pse_ids::ContentHash>,
) -> Result<pse_ids::ContentHash, WorkflowError> {
    use pse_model::SemanticFrame;
    let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::ModelingAdmissionV2);
    hash.hash(&model.case.compiled().plan.structure().key())
        .hash(&numerics.key)
        .hash(
            &crate::math::solves::profile_key(solver)
                .map_err(crate::math::MathRuntimeError::from)?
                .as_id(),
        );
    hash.bool(selected_preparation.is_some());
    if let Some(preparation) = selected_preparation {
        hash.hash(&preparation);
    }
    for artifact in model.case.compiled().artifacts.iter() {
        hash.hash(&artifact.key());
    }
    for adapter in pse_backend_native::execution::LINKED.adapters() {
        adapter.capability().row(adapter.backend()).frame(&mut hash);
    }
    let mut row = decision.row(pse_ids::ContentHash::from_bytes([0; 32]), 0);
    row.detail = None;
    row.frame(&mut hash);
    Ok(hash.finish_hash())
}
impl ModelingPackage {
    /// Evaluate exactly the selected source observations through shared compiler artifacts.
    pub async fn observe(
        &self,
        model: ModelingPreparation,
        rows: BTreeSet<SemanticId>,
        values: CaseValues,
        profile: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingObservations, WorkflowError> {
        let providers = self
            .inner_registrations(
                model.clone(),
                &ModelingCaseBindings::default(),
                &NumericalInputs::default(),
                &pse_model::numerics::NumericalPolicy::default(),
                &pse_backend_native::solve::Controls::default(),
                DerivativeOrder::Value,
                profile,
                cancel,
                implicit::ProviderDemand::Observations(Some(&rows)),
            )
            .await?;
        self.observe_registered(model, rows, values, profile, providers, cancel)
            .await
    }
    pub(in crate::workflow) async fn observe_registered(
        &self,
        model: ModelingPreparation,
        rows: BTreeSet<SemanticId>,
        values: CaseValues,
        profile: Profile,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingObservations, WorkflowError> {
        let service = self.runtime.shared.math();
        let bytes = rows
            .len()
            .checked_mul(128)
            .ok_or_else(|| contract("observation result extent"))?;
        let owner = service.reserve("modeling:observations", bytes)?;
        if rows.is_empty() {
            return Ok(ModelingObservations {
                values: BTreeMap::new(),
                applicability: Vec::new(),
                _owner: owner,
            });
        }
        // One value-independent program per observed structure; values bind per call (A6).
        let assembly = self
            .observation_program(&model, &rows, profile, cancel)
            .await?;
        let ids = assembly
            .assembly
            .structure()
            .rows()
            .iter()
            .map(|r| r.id)
            .collect::<Vec<_>>();
        let retained = owner.clone();
        let (values, applicability, _previous_owner) = service
            .with_worker(assembly, providers, cancel, move |worker| {
                let observed = worker.constraints(&values)?;
                Ok((
                    ids.into_iter().zip(observed).collect(),
                    worker.applicability_observations(),
                    retained,
                ))
            })
            .await?;
        let evidence_bytes = applicability.capacity()
            * size_of::<pse_model::applicability::Observation>()
            + applicability
                .iter()
                .map(pse_model::applicability::Observation::retained_bytes)
                .sum::<usize>();
        let owner = service.reserve(
            "modeling:observations",
            bytes.saturating_add(evidence_bytes),
        )?;
        Ok(ModelingObservations {
            values,
            applicability,
            _owner: owner,
        })
    }
    pub(in crate::workflow) fn registrations(
        &self,
    ) -> BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration> {
        self.providers
            .values()
            .map(|r| (r.spec().key(), r.clone()))
            .collect()
    }
    pub(in crate::workflow) async fn resolve_starts(
        &self,
        model: &ModelingPreparation,
        case: &ModelingCaseBindings,
        overrides: &CaseOverrides,
        compiler: Profile,
        allow_missing_free: bool,
        cancel: &crate::CancelSource,
    ) -> Result<(CaseValues, BTreeMap<SemanticId, StartSource>), WorkflowError> {
        let product = model.compiled();
        let inner_unknowns = product
            .admitted
            .implicit_systems()
            .flat_map(|i| i.unknowns.iter().copied())
            .filter(|id| !product.admitted.inputs.contains(id))
            .collect::<BTreeSet<_>>();
        let mut values = CaseValues {
            scalars: BTreeMap::new(),
        };
        let mut starts = BTreeMap::new();
        for id in &product.admitted.inputs {
            if let Some(pse_modeling::specialize::Value::Number { bits, .. }) =
                product.model.symbols[id].initial
            {
                values.scalars.insert(*id, f64::from_bits(bits));
                starts.insert(*id, StartSource::ModelDefault);
            }
        }
        let is_variable = |id: &SemanticId| {
            product
                .admitted
                .case()
                .variables()
                .iter()
                .any(|v| v.port.id == *id)
        };
        for (&id, &value) in &overrides.seed {
            let fixed = overrides.fixes.contains_key(&id)
                || case.variables.iter().any(|(path, state)| {
                    state.fixed == Some(true) && product.model.paths.get(path) == Some(&id)
                });
            if !fixed && is_variable(&id) {
                if !value.is_finite() {
                    return Err(contract("nonfinite accepted predecessor seed"));
                }
                values.scalars.insert(id, value);
                starts.insert(id, StartSource::Predecessor);
            }
        }
        for (path, value) in &case.values {
            let id = *product
                .model
                .paths
                .get(path)
                .ok_or_else(|| contract(format!("unknown scalar case path {path}")))?;
            if inner_unknowns.contains(&id) {
                continue;
            }
            if !product.admitted.inputs.contains(&id) || !value.is_finite() {
                return Err(contract(format!(
                    "case value {path} requires a finite independent coordinate"
                )));
            }
            values.scalars.insert(id, *value);
            starts.insert(id, StartSource::Case { path: path.clone() });
        }
        for (&id, &value) in &overrides.parameters {
            if !value.is_finite()
                || !product
                    .admitted
                    .case()
                    .parameters()
                    .iter()
                    .any(|p| p.id == id)
            {
                return Err(contract(
                    "continuation override must name a finite independent parameter",
                ));
            }
            values.scalars.insert(id, value);
            starts.insert(id, StartSource::Continuation);
        }
        for (&id, &value) in &case.members {
            if !product.admitted.inputs.contains(&id) || !value.is_finite() {
                return Err(contract(
                    "admitted binding requires a finite independent coordinate",
                ));
            }
            values.scalars.insert(id, value);
            starts.insert(id, StartSource::Binding { member: id });
        }
        for (&id, &value) in &overrides.fixes {
            if !value.is_finite() || !is_variable(&id) {
                return Err(contract(
                    "a temporary fix must name a variable at a finite value",
                ));
            }
            values.scalars.insert(id, value);
            starts.insert(id, StartSource::Fixed);
        }
        let hints = product
            .admitted
            .outputs
            .iter()
            .filter_map(|o| {
                if let ModelingOutput::Hint {
                    target,
                    declaration,
                    kind,
                } = o
                {
                    Some((*target, *declaration, *kind, o.row_id()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let mut pending = hints
            .iter()
            .filter(|(id, _, k, _)| {
                *k == ModelingHint::Start
                    && !values.scalars.contains_key(id)
                    && product.admitted.inputs.contains(id)
            })
            .copied()
            .collect::<Vec<_>>();
        let mut targets = BTreeSet::new();
        if pending.iter().any(|(id, _, _, _)| !targets.insert(*id)) {
            return Err(contract(
                "ambiguous start annotations for the same coordinate",
            ));
        }
        while !pending.is_empty() {
            let ready = pending
                .iter()
                .filter(|(_, _, _, row)| {
                    product
                        .admitted
                        .case()
                        .instances()
                        .iter()
                        .filter(|i| i.instance == *row)
                        .all(|i| {
                            i.slots
                                .iter()
                                .all(|s| values.scalars.contains_key(&s.source()))
                        })
                })
                .copied()
                .collect::<Vec<_>>();
            if ready.is_empty() {
                if allow_missing_free {
                    break;
                }
                return Err(contract(format!(
                    "unresolved or cyclic start dependencies: {:?}",
                    pending.iter().map(|(id, _, _, _)| id).collect::<Vec<_>>()
                )));
            }
            let observed = self
                .observe(
                    model.clone(),
                    ready.iter().map(|(_, _, _, r)| *r).collect(),
                    values.clone(),
                    compiler,
                    cancel,
                )
                .await?;
            for (target, declaration, _, row) in &ready {
                let value = observed[row];
                if !value.is_finite() {
                    return Err(contract("nonfinite start result"));
                }
                values.scalars.insert(*target, value);
                starts.insert(
                    *target,
                    StartSource::Annotation {
                        declaration: *declaration,
                    },
                );
            }
            pending.retain(|v| !ready.contains(v));
        }
        Ok((values, starts))
    }
    /// The assignment an initialization's stage and homotopy steps fix (ADR-0103 item 6):
    /// every discrete variable the case leaves free, at its specification start value or
    /// at the value declared for its case path. Each value is refused before any attempt
    /// unless it is a member of the variable's domain; the case bounds are checked again
    /// when each step binds. Empty under [`DiscreteInitialization::Refuse`].
    pub(in crate::workflow) async fn discrete_assignment(
        &self,
        analysis: &ModelingAnalysis,
        policy: &DiscreteInitialization,
        cancel: &crate::CancelSource,
    ) -> Result<BTreeMap<SemanticId, f64>, WorkflowError> {
        use DiscreteInitialization as Policy;
        use pse_modeling::DomainRefusal;
        let declared = match policy {
            Policy::Refuse => return Ok(BTreeMap::new()),
            Policy::FixAtStart => None,
            Policy::FixAt(values) => Some(values),
        };
        let case = &analysis.case;
        // The specialization a step resolves its case paths with (`resolve_case`), which
        // also resolves the declared paths.
        let mut bindings = analysis.bindings.clone();
        bindings.demand.extend(
            case.values
                .keys()
                .chain(case.variables.keys())
                .chain(declared.into_iter().flat_map(BTreeMap::keys))
                .cloned(),
        );
        bindings.demand.sort();
        bindings.demand.dedup();
        let model = self
            .prepare(
                analysis.root,
                analysis.instance,
                bindings,
                analysis.limits,
                cancel,
            )
            .await?;
        let product = model.compiled();
        let specialized = &product.model;
        let inner = Inner::of(product);
        let id = |path: &str| specialized.paths.get(path).copied();
        let mut free = BTreeMap::new();
        for declared in product.admitted.case().variables() {
            if !declared.domain.is_discrete() || inner.contains(&declared.port.id) {
                continue;
            }
            let mut variable = declared.clone();
            for (path, state) in &case.variables {
                if id(path) == Some(variable.port.id) {
                    variable.fixed = state.fixed.unwrap_or(variable.fixed);
                    variable.lower = state.lower.unwrap_or(variable.lower);
                    variable.upper = state.upper.unwrap_or(variable.upper);
                }
            }
            if !variable.fixed {
                free.insert(variable.port.id, variable);
            }
        }
        let values: BTreeMap<SemanticId, f64> = match declared {
            None => {
                self.resolve_starts(
                    &model,
                    case,
                    &CaseOverrides::default(),
                    analysis.compiler,
                    true,
                    cancel,
                )
                .await?
                .0
                .scalars
            }
            Some(declared) => declared
                .iter()
                .map(|(path, value)| {
                    id(path)
                        .filter(|id| free.contains_key(id))
                        .map(|id| (id, *value))
                        .ok_or_else(|| {
                            contract(format!(
                                "discrete initialization value {path} requires a free discrete variable"
                            ))
                        })
                })
                .collect::<Result<_, _>>()?,
        };
        free.values()
            .map(|variable| {
                let refuse = |reason| {
                    crate::workflow::modeling_error(specialized.domain_refusal(
                        variable.port.id,
                        DomainAnalysis::Initialization,
                        reason,
                    ))
                };
                let value = *values
                    .get(&variable.port.id)
                    .ok_or_else(|| refuse(DomainRefusal::NoFixValue))?;
                if !variable.domain.contains(
                    value,
                    variable.lower.unwrap_or(f64::NEG_INFINITY),
                    variable.upper.unwrap_or(f64::INFINITY),
                ) {
                    return Err(refuse(DomainRefusal::NotMember));
                }
                Ok((variable.port.id, value))
            })
            .collect()
    }
    /// Resolve starts in dependency order; explicit case values override every hint.
    /// No value is fabricated for a missing coordinate, and starts never fix variables.
    #[expect(
        clippy::too_many_arguments,
        reason = "the specialization request (root, instance, bindings, limits) travels with the case, profiles and cancellation as independent inputs"
    )]
    pub async fn prepare_solve(
        &self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        case: ModelingCaseBindings,
        order: DerivativeOrder,
        compiler: Profile,
        solver: SolverProfile,
        numerical: NumericalInputs,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        self.prepare_solve_seed(
            root,
            instance,
            bindings,
            limits,
            case,
            order,
            compiler,
            solver,
            numerical,
            CaseOverrides::default(),
            cancel,
        )
        .await
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "the specialization request (root, instance, bindings, limits) travels with the case, profiles and cancellation as independent inputs"
    )]
    pub(in crate::workflow) async fn prepare_solve_seed(
        &self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        case: ModelingCaseBindings,
        order: DerivativeOrder,
        compiler: Profile,
        solver: SolverProfile,
        numerical: NumericalInputs,
        overrides: CaseOverrides,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        let resolution = self
            .resolve_case(
                root, instance, bindings, limits, case, order, compiler, solver, numerical,
                overrides, false, cancel,
            )
            .await?;
        self.finish_case(resolution, cancel).await
    }
    pub(in crate::workflow) async fn finish_case(
        &self,
        resolution: ModelingCaseResolution,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        let ModelingCaseResolution {
            compiler,
            case_bindings,
            model,
            starts,
            mut providers,
            numerical,
            solver,
            numerics,
            parametric,
            ..
        } = resolution;
        let preparation = self.runtime.shared.math().prepare_solve(
            model.case.clone(),
            model.values.clone(),
            providers.clone(),
            solver.clone(),
            numerical.clone(),
        );
        let mut solve = tokio::select! {
            biased;
            () = cancel.cancelled() => Err(crate::math::MathRuntimeError::Cancelled),
            result = preparation => result,
        }
        .map_err(|cause| {
            let mut diagnostic = crate::workflow::diagnostics::observed(
                &cause,
                pse_diagnostics::DiagnosticStage::ModelingAdmission,
            );
            if diagnostic.rule == pse_diagnostics::DiagnosticRule::NativeStructural {
                diagnostics::attribute(&mut diagnostic, model.model.compiled());
                WorkflowError::ModelingAdmission {
                    diagnostic: Box::new(diagnostic),
                    cause: Box::new(cause),
                }
            } else {
                WorkflowError::Math(cause)
            }
        })?;
        if solve.required_order() > model.case.compiled().plan.order() {
            let response = parametric.as_ref().and_then(|program| match program {
                ParametricPreparation::Available(program) => Some(program),
                ParametricPreparation::Unavailable(_) => None,
            });
            let plan = response.map_or(model.case.compiled().plan.as_ref(), |program| {
                &program.assembly
            });
            let order = solve.required_order().max(plan.order());
            providers = self
                .inner_registrations(
                    model.model.clone(),
                    &case_bindings,
                    &numerical,
                    &solver.numerics,
                    &solver.controls,
                    order,
                    compiler,
                    cancel,
                    implicit::ProviderDemand::Case(plan),
                )
                .await?;
            solve = solve
                .with_providers(providers.clone())
                .map_err(crate::math::MathRuntimeError::from)?;
        }
        if let Some(program) = parametric {
            solve = solve.with_sensitivity(program)?;
        }
        // Accuracy requests select their own source functions; reports do not authorize
        // this work. Ordinary solves never construct or execute this projection.
        if !solve.numerics().policy.goals.is_empty() {
            use pse_model::generated::enums::{AccuracyGoalSubject, AccuracyObservation};
            let source = model.model.compiled();
            let mut selected = BTreeMap::new();
            for goal in &solve.numerics().policy.goals {
                if goal.target_kind != NumericalTarget::Observable
                    || goal.subject != AccuracyGoalSubject::SelectedOutput
                    || goal.observation != AccuracyObservation::Steady
                {
                    continue;
                }
                let row_id = ModelingOutput::Member(goal.target_id).row_id();
                let Some(row) = source
                    .admitted
                    .case()
                    .rows()
                    .iter()
                    .find(|row| row.id == row_id)
                else {
                    continue;
                };
                let quantity = self
                    .quantities
                    .quantity_type(row.quantity)
                    .map_err(|cause| contract(cause.to_string()))?;
                if row.quantity.as_id() != goal.quantity_id
                    || quantity.canonical_unit.as_id() != goal.unit_id
                {
                    return Err(contract(
                        "selected accuracy output changes its full physical quantity or canonical unit",
                    ));
                }
                selected.insert(goal.target_id, row_id);
            }
            if !selected.is_empty() {
                let executable = self
                    .runtime
                    .shared
                    .math()
                    .prepare_modeling_functions(
                        self.numerical_workspace()?,
                        model.model.clone(),
                        selected.values().copied().collect(),
                        model.case.compiled().plan.columns().to_vec(),
                        DerivativeOrder::First,
                        compiler,
                        cancel,
                    )
                    .await;
                match executable {
                    Ok(executable)
                        if source
                            .admitted
                            .provider_demands_for_plan(&executable.assembly, DerivativeOrder::First)
                            .map_err(crate::math::MathRuntimeError::from)?
                            .is_empty() =>
                    {
                        solve = solve.with_selected_outputs(
                            crate::math::solves::output_program::SelectedOutputProgram::new(
                                self.runtime.shared.math(),
                                executable,
                                selected,
                            )?,
                        )
                    }
                    Ok(_) => {}
                    Err(cause) if selected_output_derivative_unavailable(&cause) => {}
                    Err(cause) => return Err(cause.into()),
                }
            }
        }
        #[cfg(feature = "solver-kinsol")]
        if let Some(supplier) = self
            .automatic_causal_supplier(&model, &providers, &numerics, &solver, compiler, &solve)?
        {
            solve = solve.with_causal_supplier(supplier);
        }
        #[cfg(not(feature = "solver-kinsol"))]
        let _ = numerics;
        Ok(ModelingSolvePreparation {
            stored_seed_owner: None,
            source: self.clone(),
            compiler,
            profile: solver,
            model,
            solve,
            starts,
            providers,
        })
    }
    /// Resolve an authored case into its bound solver view and numerical policy. The
    /// responsibilities are separate steps: specification values and starts, variable
    /// states from bound hints and case overrides, the bound structure (prepared once per
    /// structure and rebound per values, A6), and the numerical policy.
    #[expect(
        clippy::too_many_arguments,
        reason = "the specialization request (root, instance, bindings, limits) travels with the case, profiles and cancellation as independent inputs"
    )]
    pub(in crate::workflow) async fn resolve_case(
        &self,
        root: DeclarationId,
        instance: InstanceId,
        mut bindings: Bindings,
        limits: Limits,
        case: ModelingCaseBindings,
        order: DerivativeOrder,
        compiler: Profile,
        solver: SolverProfile,
        mut numerical: NumericalInputs,
        overrides: CaseOverrides,
        allow_missing_free: bool,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingCaseResolution, WorkflowError> {
        bindings
            .demand
            .extend(case.values.keys().chain(case.variables.keys()).cloned());
        bindings.demand.sort();
        bindings.demand.dedup();
        let model = self
            .prepare(root, instance, bindings, limits, cancel)
            .await?;
        let model = if (solver.composition.policy == pse_model::strategy::CompositionPolicy::Auto
            || solver.reconstruction.is_some()
            || implicit::declared_accuracy(&model, &solver.numerics))
            && solver.sensitivity.is_none()
            && model
                .compiled()
                .admitted
                .implicit_systems()
                .any(|supplier| {
                    supplier.selection.neighborhood_evidence
                        != pse_compiler::workspace::SelectionNeighborhood::Unestablished
                        && !matches!(
                            supplier.selection.meaning,
                            pse_compiler::workspace::ImplicitMeaning::Relation
                        )
                        && supplier.selection.restriction.is_none()
                }) {
            model.original_equations().unwrap_or(model)
        } else {
            model
        };
        let inner = Inner::of(model.compiled());
        let (values, starts) = self
            .resolve_starts(
                &model,
                &case,
                &overrides,
                compiler,
                allow_missing_free,
                cancel,
            )
            .await?;
        if !allow_missing_free {
            require_inputs(&model, &values)?;
        }
        // Starts and fixed-state hints consume values before the bound case owns its
        // derivative coordinates. Authored partials retain their independent demand.
        let mut providers = self
            .inner_registrations(
                model.clone(),
                &case,
                &numerical,
                &solver.numerics,
                &solver.controls,
                DerivativeOrder::Value,
                compiler,
                cancel,
                implicit::ProviderDemand::Observations(None),
            )
            .await?;
        numerical
            .declarations
            .retain(|r| !inner.contains(&r.declaration.target_id));
        numerical.implicit = implicit::factorable_definitions(&model, &case);
        numerical.implicit.retain(|key, _| {
            model
                .compiled()
                .admitted
                .implicit_systems()
                .find(|supplier| supplier.descriptor.spec().key() == *key)
                .is_some_and(|supplier| {
                    supplier
                        .unknowns
                        .iter()
                        .all(|id| inner.unknowns.contains(id))
                })
        });
        let mut solver = solver;
        solver
            .numerics
            .requirements
            .retain(|r| !inner.contains(&r.target_id));
        let mut states = self
            .variable_states(
                &model,
                &inner,
                &case,
                &values,
                &mut numerical,
                compiler,
                &providers,
                cancel,
            )
            .await?;
        for id in overrides.fixes.keys() {
            states.entry(*id).or_default().fixed = Some(true);
        }
        let prepared = self
            .bound_case(&model, values.clone(), &states, order, compiler, cancel)
            .await?;
        // Upgrade from the actual bound output/coordinate maps, not every formal
        // input of the original source. Native inner residual minima remain separate.
        if order > DerivativeOrder::Value {
            providers = self
                .inner_registrations(
                    model.clone(),
                    &case,
                    &numerical,
                    &solver.numerics,
                    &solver.controls,
                    order,
                    compiler,
                    cancel,
                    implicit::ProviderDemand::Case(&prepared.case.compiled().plan),
                )
                .await?;
        }
        // A sensitivity request differentiates the view's parametric program, and each
        // parameter resolves its coordinate scale as a variable coordinate (Plan 22 S1).
        let parametric = match &solver.sensitivity {
            Some(request) => {
                request
                    .admit(solver.intent)
                    .map_err(crate::math::MathRuntimeError::from)?;
                // A propagation's outputs are columns the step solves (Plan 22 S4).
                let columns = prepared.case.compiled().plan.columns();
                if let Some(propagation) = &request.propagation
                    && let Some(output) = propagation.outputs.iter().find(|o| !columns.contains(o))
                {
                    return Err(contract(format!(
                        "propagation output {output} is not a variable the case solves"
                    )));
                }
                // Validate request roles before response compilation; malformed requests
                // refuse rather than becoming an unavailable numerical quantity.
                let declared = prepared
                    .case
                    .compiled()
                    .plan
                    .structure()
                    .parameters()
                    .iter()
                    .map(|p| p.id)
                    .collect::<BTreeSet<_>>();
                if let Some(id) = request.parameters.iter().find(|id| {
                    !declared.contains(id)
                        || columns.contains(id)
                        || values
                            .scalars
                            .get(id)
                            .is_none_or(|value| !value.is_finite())
                }) {
                    return Err(contract(format!(
                        "sensitivity parameter {id} is not a declared fixed parameter with a finite value"
                    )));
                }
                let derivative_order =
                    if solver.intent == pse_backend_native::solve::SolveIntent::Root {
                        DerivativeOrder::First
                    } else {
                        DerivativeOrder::Second
                    };
                let result = self
                    .parametric_program(
                        &model,
                        &prepared,
                        &request.parameters,
                        derivative_order,
                        compiler,
                        cancel,
                    )
                    .await;
                let preparation = match result {
                    Ok(program) => {
                        providers = self
                            .inner_registrations(
                                model.clone(),
                                &case,
                                &numerical,
                                &solver.numerics,
                                &solver.controls,
                                derivative_order,
                                compiler,
                                cancel,
                                implicit::ProviderDemand::Case(&program.assembly),
                            )
                            .await?;
                        numerical
                            .targets
                            .extend(program.assembly.parameter_targets());
                        ParametricPreparation::Available(program)
                    }
                    Err(WorkflowError::Math(cause))
                        if solver.intent == pse_backend_native::solve::SolveIntent::Root =>
                    {
                        let Some(withheld) = root_parametric_failure(&cause) else {
                            return Err(WorkflowError::Math(cause));
                        };
                        ParametricPreparation::Unavailable(withheld)
                    }
                    Err(cause) => return Err(cause),
                };
                Some(preparation)
            }
            None => None,
        };
        // ADR-0103 item 6: a root or initialization solve cannot decide a discrete variable.
        let analysis = match solver.intent {
            pse_backend_native::solve::SolveIntent::Root => Some(DomainAnalysis::Root),
            pse_backend_native::solve::SolveIntent::Initialize => {
                Some(DomainAnalysis::Initialization)
            }
            _ => None,
        };
        if let Some(analysis) = analysis {
            model
                .compiled()
                .model
                .require_fixed_discrete(
                    prepared.case.compiled().plan.columns().iter().copied(),
                    analysis,
                )
                .map_err(crate::workflow::modeling_error)?;
        }
        let numerics = self
            .resolve_numerics(
                &model,
                &inner,
                &prepared,
                &mut numerical,
                &mut solver,
                compiler,
                &providers,
                cancel,
            )
            .await?;
        Ok(ModelingCaseResolution {
            case_bindings: case,
            compiler,
            numerics,
            model: prepared,
            starts,
            providers,
            numerical,
            solver,
            parametric,
        })
    }
    /// Variable states of the solver view: evaluated bound hints, nominal declarations for
    /// the numerical policy, and the case's fixed/free and bound overrides.
    #[expect(
        clippy::too_many_arguments,
        reason = "hints evaluate at the resolved values with the case's providers and profile"
    )]
    async fn variable_states(
        &self,
        model: &ModelingPreparation,
        inner: &Inner,
        case: &ModelingCaseBindings,
        values: &CaseValues,
        numerical: &mut NumericalInputs,
        compiler: Profile,
        providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        cancel: &crate::CancelSource,
    ) -> Result<BTreeMap<SemanticId, ModelingVariableState>, WorkflowError> {
        let product = model.compiled();
        let hints = hints(product)
            .into_iter()
            .filter(|(id, _, _, _)| !inner.contains(id))
            .collect::<Vec<_>>();
        let bound_rows = hints
            .iter()
            .filter(|(_, _, k, _)| {
                matches!(
                    k,
                    ModelingHint::Lower | ModelingHint::Upper | ModelingHint::Nominal
                )
            })
            .map(|(_, _, _, r)| *r)
            .collect::<BTreeSet<_>>();
        let observed = self
            .observe_registered(
                model.clone(),
                bound_rows,
                values.clone(),
                compiler,
                providers.clone(),
                cancel,
            )
            .await?;
        let variables = variables(product);
        let equations = equations(product);
        let mut states = BTreeMap::<SemanticId, ModelingVariableState>::new();
        let mut hint_keys = BTreeSet::new();
        for (target, declaration, kind, row) in &hints {
            let Some(value) = observed.get(row).copied() else {
                continue;
            };
            if !hint_keys.insert((*target, *kind)) {
                return Err(contract("ambiguous numerical annotations for a coordinate"));
            }
            match kind {
                ModelingHint::Lower | ModelingHint::Upper => {
                    if !variables.contains(target) {
                        return Err(contract("bound annotation requires a variable"));
                    }
                    let state = states.entry(*target).or_default();
                    if *kind == ModelingHint::Lower {
                        state.lower = Some(Some(value));
                    } else {
                        state.upper = Some(Some(value));
                    }
                }
                ModelingHint::Nominal => {
                    let kind = if variables.contains(target) {
                        NumericalTarget::Variable
                    } else if equations.contains(target) {
                        NumericalTarget::Row
                    } else {
                        NumericalTarget::Observable
                    };
                    numerical.declarations.push(requirement(
                        model.solved().lineage(),
                        *target,
                        kind,
                        *declaration,
                        NumericalSource::ModelHint,
                        Some(value),
                        None,
                    ));
                }
                _ => {}
            }
        }
        for (path, overrides) in &case.variables {
            let id = *product
                .model
                .paths
                .get(path)
                .ok_or_else(|| contract(format!("unknown case specification {path}")))?;
            if inner.unknowns.contains(&id) {
                continue;
            }
            let state = states.entry(id).or_default();
            if let Some(v) = overrides.fixed {
                state.fixed = Some(v);
            }
            if let Some(v) = overrides.lower {
                state.lower = Some(v);
            }
            if let Some(v) = overrides.upper {
                state.upper = Some(v);
            }
        }
        Ok(states)
    }
    /// The resolved numerical policy of the bound view: observable nominal targets and
    /// scaling-scheme row scales evaluated at the physical nominal point, which starts from
    /// the view's values, derived realization parameters included (ADR-0104).
    #[expect(
        clippy::too_many_arguments,
        reason = "scales evaluate at the nominal point with the case's providers and profile"
    )]
    async fn resolve_numerics(
        &self,
        model: &ModelingPreparation,
        inner: &Inner,
        prepared: &ModelingCasePreparation,
        numerical: &mut NumericalInputs,
        solver: &mut SolverProfile,
        compiler: Profile,
        providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        cancel: &crate::CancelSource,
    ) -> Result<Arc<pse_model::numerics::ResolvedNumericalPolicy>, WorkflowError> {
        let product = model.compiled();
        let variables = variables(product);
        let equations = equations(product);
        let authored = hints(product)
            .into_iter()
            .filter(|(target, _, kind, _)| {
                matches!(
                    kind,
                    ModelingHint::AccuracyGoalTime
                        | ModelingHint::AccuracyGoalResolution
                        | ModelingHint::AccuracyGoalLower
                        | ModelingHint::AccuracyGoalUpper
                        | ModelingHint::EngineeringScaleValue
                ) && !inner.contains(target)
            })
            .collect::<Vec<_>>();
        let mut frozen = prepared
            .case
            .compiled()
            .plan
            .structure()
            .variables()
            .iter()
            .filter(|variable| !variable.fixed)
            .map(|variable| variable.port.id)
            .collect::<BTreeSet<_>>();
        frozen.extend(inner.unknowns.iter().copied());
        for annotation in &product.model.annotations {
            if !matches!(
                annotation.value,
                AnnotationValue::AccuracyGoal(_) | AnnotationValue::EngineeringScale(_)
            ) {
                continue;
            }
            if inner.contains(&annotation.target) {
                return Err(contract(
                    "authored accuracy targets inside a nested realization are not bound by the outer solve",
                ));
            }
            match &annotation.value {
                AnnotationValue::AccuracyGoal(goal) => {
                    for expression in [
                        goal.time.as_ref(),
                        goal.resolution.as_ref(),
                        goal.criterion_lower.as_ref(),
                        goal.criterion_upper.as_ref(),
                    ]
                    .into_iter()
                    .flatten()
                    {
                        require_frozen_expression(expression, &product.model, &frozen)?;
                    }
                }
                AnnotationValue::EngineeringScale(scale) => {
                    require_frozen_expression(&scale.value, &product.model, &frozen)?;
                }
                _ => {}
            }
        }
        let mut authored_values = BTreeMap::new();
        if !authored.is_empty() {
            let rows = authored.iter().map(|(_, _, _, row)| *row).collect();
            let observed = self
                .observe_registered(
                    model.clone(),
                    rows,
                    prepared.values.clone(),
                    compiler,
                    providers.clone(),
                    cancel,
                )
                .await?;
            authored_values.clone_from(&*observed);
        }
        let mut nominal_point = prepared.values.clone();
        // Resolve source precedence before selecting the physical nominal point.
        let physical = prepared.case.compiled().quantities.clone();
        let mut targets = prepared
            .case
            .compiled()
            .plan
            .numerical_targets(&physical)
            .map_err(crate::math::MathRuntimeError::from)?;
        // Observable hints retain their own typed targets, outside the solver row inventory.
        for annotation in &product.model.annotations {
            if !matches!(annotation.value, AnnotationValue::Nominal(_))
                || inner.contains(&annotation.target)
                || variables.contains(&annotation.target)
                || equations.contains(&annotation.target)
            {
                continue;
            }
            let symbol = product
                .model
                .symbols
                .get(&annotation.target)
                .ok_or_else(|| contract("nominal target absent"))?;
            let pse_modeling::Type::Quantity(q) = &symbol.ty else {
                return Err(contract("nominal target physical type"));
            };
            let q = q
                .resolve(&physical, &BTreeMap::new())
                .map_err(|e| contract(e.to_string()))?;
            let unit = physical
                .quantity_type(q)
                .map_err(|e| contract(e.to_string()))?
                .canonical_unit;
            numerical.targets.push(pse_math::numerics::TargetSpec {
                id: annotation.target,
                kind: NumericalTarget::Observable,
                quantity: q,
                unit,
                integer: false,
                declared_tolerance: None,
            });
        }
        // A requested authored scalar is itself a selected observation. Admit its
        // physical type before policy resolution; reporting annotations are optional.
        for goal in &solver.numerics.goals {
            if goal.subject != pse_model::generated::enums::AccuracyGoalSubject::SelectedOutput
                || goal.observation != pse_model::generated::enums::AccuracyObservation::Steady
                || goal.target_kind != NumericalTarget::Observable
                || targets
                    .iter()
                    .chain(&numerical.targets)
                    .any(|target| target.id == goal.target_id && target.kind == goal.target_kind)
            {
                continue;
            }
            let symbol = product
                .model
                .symbols
                .get(&goal.target_id)
                .ok_or_else(|| contract("accuracy goal target absent from selected model"))?;
            let pse_modeling::Type::Quantity(quantity) = &symbol.ty else {
                return Err(contract("accuracy goal requires a scalar physical target"));
            };
            let quantity = quantity
                .resolve(&physical, &BTreeMap::new())
                .map_err(|error| contract(error.to_string()))?;
            let unit = physical
                .quantity_type(quantity)
                .map_err(|error| contract(error.to_string()))?
                .canonical_unit;
            numerical.targets.push(pse_math::numerics::TargetSpec {
                id: goal.target_id,
                kind: NumericalTarget::Observable,
                quantity,
                unit,
                integer: false,
                declared_tolerance: None,
            });
        }
        targets.extend(numerical.targets.clone());
        lower_authored_accuracy(
            product,
            model.solved().lineage(),
            &physical,
            numerical,
            &mut solver.numerics,
            &mut targets,
            &authored_values,
        )?;
        let reconstruction_requirements = state_reconstruction_requirements(
            product,
            &targets,
            model.solved().lineage(),
            &physical,
        )?;
        let conservation_requirements =
            conservation_row_requirements(product, &targets, model.solved().lineage(), &physical)?;
        // These source-issued declarations must survive finish_case and every
        // subsequent numerical re-resolution. Replace this producer's own IDs
        // idempotently, including removal when its reconstruction row is absent.
        let mut declared_row_ids = product
            .model
            .state_specifications
            .values()
            .flat_map(|specification| specification.reconstructions.iter())
            .map(|(row, _)| state_reconstruction_requirement_id(row))
            .collect::<BTreeSet<_>>();
        declared_row_ids.extend(
            product
                .model
                .closures
                .values()
                .map(conservation_requirement_id),
        );
        numerical.declarations.retain(|requirement| {
            !declared_row_ids.contains(&requirement.declaration.requirement_id)
        });
        numerical.declarations.extend(reconstruction_requirements);
        numerical.declarations.extend(conservation_requirements);
        let resolved = pse_math::numerics::resolve(
            &physical,
            &targets,
            &numerical.declarations,
            &solver.numerics,
        )
        .map_err(crate::math::MathRuntimeError::from)?;
        // Free variables move to their nominal; fixed variables and the parameter
        // coordinates of a sensitivity request keep their values.
        for t in &resolved.targets {
            if t.kind == NumericalTarget::Variable
                && prepared
                    .case
                    .compiled()
                    .plan
                    .structure()
                    .variables()
                    .iter()
                    .any(|v| v.port.id == t.id && !v.fixed)
            {
                nominal_point.scalars.insert(t.id, t.nominal);
            }
        }
        let schemes = product
            .model
            .annotations
            .iter()
            .filter_map(|a| {
                if let AnnotationValue::Scale(s) = a.value
                    && !inner.rows.contains(&a.target)
                {
                    Some((a.target, s, a.lineage.declaration))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let term_rows = schemes
            .iter()
            .flat_map(|(id, _, _)| {
                product
                    .admitted
                    .term_outputs
                    .get(id)
                    .into_iter()
                    .flatten()
                    .map(|(r, _)| *r)
            })
            .collect::<BTreeSet<_>>();
        if !schemes.is_empty() {
            let terms = self
                .observe_registered(
                    model.clone(),
                    term_rows,
                    nominal_point,
                    compiler,
                    providers.clone(),
                    cancel,
                )
                .await?;
            for (id, scheme, source) in schemes {
                let inputs = product
                    .admitted
                    .term_outputs
                    .get(&id)
                    .ok_or_else(|| contract("scaling scheme requires an equation"))?;
                let terms = inputs.iter().map(|(r, s)| s * terms[r]).collect::<Vec<_>>();
                let scale = pse_math::numerics::term_scale(scheme, &terms)
                    .map_err(crate::math::MathRuntimeError::from)?;
                numerical.declarations.push(requirement(
                    model.solved().lineage(),
                    id,
                    NumericalTarget::Row,
                    source,
                    NumericalSource::DerivedNominal,
                    None,
                    Some(scale),
                ));
            }
        }
        Ok(Arc::new(
            pse_math::numerics::resolve(
                &physical,
                &targets,
                &numerical.declarations,
                &solver.numerics,
            )
            .map_err(crate::math::MathRuntimeError::from)?,
        ))
    }
}
fn selected_output_derivative_unavailable(cause: &crate::math::MathRuntimeError) -> bool {
    fn derivative(cause: &pse_math::MathError) -> bool {
        match cause {
            pse_math::MathError::DerivativeDemand { .. } => true,
            pse_math::MathError::Instance { cause, .. } => derivative(cause),
            _ => false,
        }
    }
    use crate::math::MathRuntimeError as E;
    match cause {
        E::Math(cause) | E::Solve(pse_backend_native::ProblemError::Math(cause)) => {
            derivative(cause)
        }
        E::Compile(pse_compiler::workspace::CompileError::Math(cause)) => derivative(cause),
        E::Shared(cause) => selected_output_derivative_unavailable(cause),
        _ => false,
    }
}
/// Lower selected authored accuracy declarations into the shared numerical policy.
/// Expressions have already been evaluated by the compiler's typed modeling program;
/// this function only binds those values to their existing target and registry owners.
pub(in crate::workflow) fn lower_authored_accuracy(
    product: &pse_compiler::workspace::PreparedModeling,
    scope: pse_model::lineage::Lineage,
    registry: &pse_quantity::QuantityRegistry,
    numerical: &mut NumericalInputs,
    policy: &mut pse_model::numerics::NumericalPolicy,
    targets: &mut Vec<pse_math::numerics::TargetSpec>,
    values: &BTreeMap<SemanticId, f64>,
) -> Result<(), WorkflowError> {
    fn ensure_target(
        numerical: &mut NumericalInputs,
        targets: &mut Vec<pse_math::numerics::TargetSpec>,
        registry: &pse_quantity::QuantityRegistry,
        id: SemanticId,
        kind: pse_model::generated::enums::NumericalTarget,
        quantity: pse_quantity::QuantityTypeId,
    ) -> Result<pse_math::numerics::TargetSpec, WorkflowError> {
        if let Some(target) = targets
            .iter()
            .find(|target| target.id == id && target.kind == kind)
        {
            return Ok(target.clone());
        }
        let unit = registry
            .quantity_type(quantity)
            .map_err(|error| contract(error.to_string()))?
            .canonical_unit;
        let target = pse_math::numerics::TargetSpec {
            id,
            kind,
            quantity,
            unit,
            integer: false,
            declared_tolerance: None,
        };
        if !numerical
            .targets
            .iter()
            .any(|known| known.id == id && known.kind == kind)
        {
            numerical.targets.push(target.clone());
        }
        targets.push(target.clone());
        Ok(target)
    }
    use pse_model::generated::enums::{AccuracyGoalSubject, NumericalSource, NumericalTarget};
    use pse_modeling::specialize::Value;

    let row_lineage = |instance| pse_model::lineage::Lineage {
        model_id: scope.model_id,
        case_id: scope.case_id,
        instance_id: Some(instance),
        fit_id: scope.fit_id,
    };
    let find_value = |target: SemanticId,
                      declaration: DeclarationId,
                      kind: ModelingHint|
     -> Result<f64, WorkflowError> {
        let row = hints(product)
            .into_iter()
            .find(|(hint_target, hint_declaration, hint_kind, _)| {
                *hint_target == target && *hint_declaration == declaration && *hint_kind == kind
            })
            .map(|(_, _, _, row)| row)
            .ok_or_else(|| contract("authored accuracy expression has no typed program output"))?;
        let value = values
            .get(&row)
            .copied()
            .ok_or_else(|| contract("authored accuracy expression was not evaluated"))?;
        if !value.is_finite() {
            return Err(contract("authored accuracy expression is nonfinite"));
        }
        Ok(value)
    };

    for rule in &product.model.engineering_rules {
        let Value::Number { bits, quantity } = &rule.value else {
            return Err(contract("shared engineering rule is not a typed number"));
        };
        let quantity_type = registry
            .quantity_type(*quantity)
            .map_err(|error| contract(error.to_string()))?;
        let row = pse_model::numerics::EngineeringRule {
            rule_id: rule.id.into(),
            quantity_id: quantity.as_id(),
            unit_id: quantity_type.canonical_unit.as_id(),
            physical_allowance: Some(f64::from_bits(*bits)),
            relative_fraction: None,
            provenance: format!(
                "shared engineering rule constant {} marked by {}",
                rule.id, rule.marker
            ),
        };
        if let Some(previous) = policy
            .engineering_rules
            .iter()
            .find(|item| item.rule_id == row.rule_id)
        {
            if previous != &row {
                return Err(contract(
                    "conflicting materialization of a shared engineering rule",
                ));
            }
        } else {
            policy.engineering_rules.push(row);
        }
    }

    let annotations = product.model.annotations.iter().collect::<Vec<_>>();
    let mut declarations = Vec::new();
    for annotation in &annotations {
        match &annotation.value {
            AnnotationValue::AccuracyGoal(goal) => {
                let (kind, quantity) = if goal.subject == AccuracyGoalSubject::OptimalObjective {
                    let selected = product.model.objectives.solved().ok_or_else(|| {
                        contract("optimal-objective accuracy requires one selected objective level")
                    })?;
                    let (_, authored_quantity) = authored_target_semantics(
                        product,
                        numerical,
                        targets,
                        registry,
                        annotation.target,
                    )?;
                    if authored_quantity != selected.quantity {
                        return Err(contract(
                            "optimal-objective accuracy target quantity differs from the selected objective quantity",
                        ));
                    }
                    if !product
                        .model
                        .objectives
                        .members_of(selected)
                        .any(|member| member.target == annotation.target)
                    {
                        return Err(contract(
                            "optimal-objective accuracy target is not a member of the selected objective",
                        ));
                    }
                    (NumericalTarget::Objective, selected.quantity)
                } else {
                    authored_target_semantics(
                        product,
                        numerical,
                        targets,
                        registry,
                        annotation.target,
                    )?
                };
                let target_id = if kind == NumericalTarget::Objective {
                    SemanticId::from_bytes([0; 16])
                } else {
                    annotation.target
                };
                let target =
                    ensure_target(numerical, targets, registry, target_id, kind, quantity)?;
                let time = goal
                    .time
                    .as_ref()
                    .map(|_| {
                        find_value(
                            annotation.target,
                            annotation.lineage.declaration,
                            ModelingHint::AccuracyGoalTime,
                        )
                    })
                    .transpose()?;
                let resolution = goal
                    .resolution
                    .as_ref()
                    .map(|_| {
                        find_value(
                            annotation.target,
                            annotation.lineage.declaration,
                            ModelingHint::AccuracyGoalResolution,
                        )
                    })
                    .transpose()?;
                let criterion_lower = goal
                    .criterion_lower
                    .as_ref()
                    .map(|_| {
                        find_value(
                            annotation.target,
                            annotation.lineage.declaration,
                            ModelingHint::AccuracyGoalLower,
                        )
                    })
                    .transpose()?;
                let criterion_upper = goal
                    .criterion_upper
                    .as_ref()
                    .map(|_| {
                        find_value(
                            annotation.target,
                            annotation.lineage.declaration,
                            ModelingHint::AccuracyGoalUpper,
                        )
                    })
                    .transpose()?;
                let lineage = row_lineage(annotation.lineage.instance);
                let source = goal.source;
                let row = pse_model::numerics::AccuracyGoal {
                    goal_id: goal.id.into(),
                    model_id: lineage.model_id,
                    case_id: lineage.case_id,
                    instance_id: lineage.instance_id,
                    fit_id: lineage.fit_id,
                    target_id,
                    target_kind: kind,
                    quantity_id: target.quantity.as_id(),
                    unit_id: target.unit.as_id(),
                    subject: goal.subject,
                    observation: goal.observation,
                    time,
                    resolution,
                    criterion_lower,
                    criterion_upper,
                    required_class: goal.required_class,
                    use_policy: goal.use_policy,
                    refine: goal.refine,
                    source,
                    priority: 0,
                    provenance: format!(
                        "accuracy goal {} on {}",
                        annotation.lineage.declaration, annotation.target
                    ),
                };
                declarations.push(row);
            }
            AnnotationValue::EngineeringScale(scale) => {
                let (kind, quantity) = authored_target_semantics(
                    product,
                    numerical,
                    targets,
                    registry,
                    annotation.target,
                )?;
                let target = ensure_target(
                    numerical,
                    targets,
                    registry,
                    annotation.target,
                    kind,
                    quantity,
                )?;
                let value = find_value(
                    annotation.target,
                    annotation.lineage.declaration,
                    ModelingHint::EngineeringScaleValue,
                )?;
                let row = pse_model::numerics::EngineeringScale {
                    scale_id: scale.id.into(),
                    model_id: scope.model_id,
                    case_id: scope.case_id,
                    instance_id: Some(annotation.lineage.instance),
                    fit_id: scope.fit_id,
                    target_id: annotation.target,
                    target_kind: kind,
                    quantity_id: target.quantity.as_id(),
                    unit_id: target.unit.as_id(),
                    kind: scale.kind,
                    value,
                    source: scale.source,
                    priority: 0,
                    provenance: format!(
                        "engineering scale {} on {}",
                        annotation.lineage.declaration, annotation.target
                    ),
                };
                if let Some(previous) = policy
                    .engineering_scales
                    .iter()
                    .find(|item| item.scale_id == row.scale_id)
                {
                    if previous != &row {
                        return Err(contract(
                            "conflicting materialization of an engineering scale",
                        ));
                    }
                } else {
                    policy.engineering_scales.push(row);
                }
            }
            AnnotationValue::EngineeringDefault { rule_id, source } => {
                let (kind, quantity) = authored_target_semantics(
                    product,
                    numerical,
                    targets,
                    registry,
                    annotation.target,
                )?;
                let target = ensure_target(
                    numerical,
                    targets,
                    registry,
                    annotation.target,
                    kind,
                    quantity,
                )?;
                let mut sourced = requirement(
                    row_lineage(annotation.lineage.instance),
                    annotation.target,
                    kind,
                    annotation.lineage.declaration,
                    *source,
                    None,
                    None,
                );
                sourced.declaration.shared_engineering_allowance = Some(true);
                sourced.declaration.engineering_rule_id = Some((*rule_id).into());
                sourced.declaration.unit_id = Some(target.unit.as_id());
                let id = sourced.declaration.requirement_id;
                numerical
                    .declarations
                    .retain(|existing| existing.declaration.requirement_id != id);
                numerical.declarations.push(sourced);
            }
            _ => {}
        }
    }

    // Bind against the final selected targets now, so authored and request-local goals
    // share the same unit conversion, identity conflict and target-admission path.
    let requested = policy
        .goals
        .iter()
        .filter(|goal| goal.source == NumericalSource::Analysis)
        .cloned()
        .collect::<Vec<_>>();
    policy.goals =
        pse_math::engineering_accuracy::bind_goals(registry, targets, &declarations, &requested)
            .map_err(|error| contract(error.to_string()))?;
    Ok(())
}

fn authored_target_semantics(
    product: &pse_compiler::workspace::PreparedModeling,
    numerical: &NumericalInputs,
    targets: &[pse_math::numerics::TargetSpec],
    registry: &pse_quantity::QuantityRegistry,
    id: SemanticId,
) -> Result<(NumericalTarget, pse_quantity::QuantityTypeId), WorkflowError> {
    if let Some(target) = targets
        .iter()
        .chain(numerical.targets.iter())
        .find(|target| target.id == id && target.kind != NumericalTarget::Objective)
    {
        return Ok((target.kind, target.quantity));
    }
    let symbol = product
        .model
        .symbols
        .get(&id)
        .ok_or_else(|| contract("authored accuracy target is not a specialized scalar"))?;
    let scheme = symbol
        .ty
        .quantity_scheme()
        .ok_or_else(|| contract("authored accuracy target has no physical quantity"))?;
    let quantity = scheme
        .resolve(registry, &Default::default())
        .map_err(|error| contract(error.to_string()))?;
    Ok((NumericalTarget::Observable, quantity))
}

/// Refuse accuracy supports that vary with the current nonlinear solve coordinates.
/// Derived symbols are expanded transitively using their already-specialized expressions.
pub(in crate::workflow) fn require_frozen_expression(
    expression: &pse_authoring::dsl::Expr,
    model: &pse_modeling::specialize::SpecializedModel,
    forbidden: &BTreeSet<SemanticId>,
) -> Result<(), WorkflowError> {
    fn visit(
        expression: &pse_authoring::dsl::Expr,
        model: &pse_modeling::specialize::SpecializedModel,
        forbidden: &BTreeSet<SemanticId>,
        seen: &mut BTreeSet<SemanticId>,
    ) -> Result<(), WorkflowError> {
        for path in expression.free_paths() {
            let Some(segment) = path.segments.first() else {
                continue;
            };
            if path.segments.len() != 1 || !segment.indices.is_empty() {
                continue;
            }
            let Some(id) = model
                .symbols
                .keys()
                .copied()
                .find(|id| pse_modeling::specialize::symbol_name(*id) == segment.name)
            else {
                continue;
            };
            if forbidden.contains(&id) {
                return Err(contract(
                    "accuracy support depends on a changing solve coordinate",
                ));
            }
            if seen.insert(id)
                && let Some(derived) = model
                    .symbols
                    .get(&id)
                    .and_then(|symbol| symbol.expression.as_ref())
            {
                visit(derived, model, forbidden, seen)?;
            }
        }
        Ok(())
    }
    visit(expression, model, forbidden, &mut BTreeSet::new())
}

fn state_reconstruction_requirement_id(row: &pse_modeling::specialize::Row) -> SemanticId {
    pse_ids::named_id(
        pse_ids::named_id(row.lineage.declaration.as_id(), &row.id.to_string()),
        "state-reconstruction-tolerance",
    )
}
fn conservation_requirement_id(closure: &pse_modeling::specialize::Closure) -> SemanticId {
    pse_ids::named_id(
        pse_ids::named_id(closure.lineage.declaration.as_id(), &closure.id.to_string()),
        "conservation-row-tolerance",
    )
}
/// Lower an already specialized physical tolerance through the existing numerical owner.
fn physical_row_requirement(
    target: &pse_math::numerics::TargetSpec,
    tolerance: &pse_modeling::specialize::Value,
    lineage: pse_model::lineage::Lineage,
    declaration: DeclarationId,
    registry: &pse_quantity::QuantityRegistry,
) -> Result<pse_math::numerics::SourcedRequirement, WorkflowError> {
    use pse_modeling::specialize::Value;
    // Static numbers are already canonical; attach their canonical unit rather
    // than applying the authored storage conversion a second time.
    let (magnitude, quantity) = match tolerance {
        Value::Number { bits, quantity } | Value::Coordinate { bits, quantity, .. } => {
            (f64::from_bits(*bits), *quantity)
        }
        Value::Integer(value) => (
            *value as f64,
            registry
                .neutral_dimensionless()
                .ok_or_else(|| contract("integer row tolerance has no scalar contract"))?,
        ),
        _ => return Err(contract("declared row tolerance is not numeric")),
    };
    pse_quantity::admission::require_same_contract(target.quantity, quantity, registry)
        .map_err(pse_math::MathError::from)
        .map_err(crate::math::MathRuntimeError::from)?;
    let unit = registry
        .quantity_type(quantity)
        .map_err(pse_math::MathError::from)
        .map_err(crate::math::MathRuntimeError::from)?
        .canonical_unit;
    if !magnitude.is_finite() || magnitude <= 0.0 {
        return Err(contract(
            "declared row tolerance must be finite and positive",
        ));
    }
    let mut declared = requirement(
        lineage,
        target.id,
        NumericalTarget::Row,
        declaration,
        NumericalSource::Model,
        None,
        None,
    );
    declared.declaration.absolute_tolerance = Some(magnitude);
    declared.declaration.unit_id = Some(unit.as_id());
    Ok(declared)
}
/// Only source conservation equalities retained in this numerical view own row budgets.
/// Connection transport and supplied-state consistency closures remain independent checks.
fn conservation_row_requirements(
    product: &pse_compiler::workspace::PreparedModeling,
    targets: &[pse_math::numerics::TargetSpec],
    lineage: pse_model::lineage::Lineage,
    registry: &pse_quantity::QuantityRegistry,
) -> Result<Vec<pse_math::numerics::SourcedRequirement>, WorkflowError> {
    let mut requirements = Vec::new();
    for closure in product.model.closures.values().filter(|closure| {
        closure.mode == pse_model::generated::enums::ModelingAccumulatorMode::Conservation
            && !closure.observation_only
    }) {
        let row = pse_ids::named_id(closure.id, "conservation");
        let Some(target) = targets
            .iter()
            .find(|target| target.kind == NumericalTarget::Row && target.id == row)
        else {
            continue;
        };
        let mut source = lineage;
        source.instance_id = Some(closure.lineage.instance);
        let mut declared = physical_row_requirement(
            target,
            &closure.tolerance,
            source,
            closure.lineage.declaration,
            registry,
        )?;
        declared.declaration.requirement_id = conservation_requirement_id(closure);
        declared.declaration.provenance = format!(
            "conservation row {} declared by {}",
            row, closure.lineage.declaration,
        );
        requirements.push(declared);
    }
    Ok(requirements)
}
/// Only original reconstruction equations present in this solved view impose row budgets.
/// Supplied states still keep their independent consistency checks; absent equations
/// cannot introduce phantom numerical requirements into a projected or fixed view.
fn state_reconstruction_requirements(
    product: &pse_compiler::workspace::PreparedModeling,
    targets: &[pse_math::numerics::TargetSpec],
    lineage: pse_model::lineage::Lineage,
    registry: &pse_quantity::QuantityRegistry,
) -> Result<Vec<pse_math::numerics::SourcedRequirement>, WorkflowError> {
    let mut requirements: BTreeMap<SemanticId, pse_math::numerics::SourcedRequirement> =
        BTreeMap::new();
    for specification in product.model.state_specifications.values() {
        for (row, tolerance) in &specification.reconstructions {
            let Some(target) = targets
                .iter()
                .find(|target| target.kind == NumericalTarget::Row && target.id == row.id)
            else {
                continue;
            };
            let mut source = lineage;
            source.instance_id = Some(row.lineage.instance);
            let mut requirement = physical_row_requirement(
                target,
                tolerance,
                source,
                row.lineage.declaration,
                registry,
            )?;
            requirement.declaration.requirement_id = state_reconstruction_requirement_id(row);
            requirement.declaration.provenance = format!(
                "state reconstruction {} declared by {}",
                row.id, row.lineage.declaration,
            );
            if let Some(existing) = requirements.get(&row.id) {
                if existing.declaration.requirement_id != requirement.declaration.requirement_id
                    || existing.declaration.absolute_tolerance
                        != requirement.declaration.absolute_tolerance
                    || existing.declaration.unit_id != requirement.declaration.unit_id
                    || existing.declaration.instance_id != requirement.declaration.instance_id
                {
                    return Err(contract("conflicting state reconstruction row tolerances"));
                }
            } else {
                requirements.insert(row.id, requirement);
            }
        }
    }
    Ok(requirements.into_values().collect())
}
/// Coordinates owned by nested implicit realizations; the outer case never binds them.
struct Inner {
    unknowns: BTreeSet<SemanticId>,
    rows: BTreeSet<SemanticId>,
}
impl Inner {
    fn of(product: &pse_compiler::workspace::PreparedModeling) -> Self {
        Self {
            unknowns: product
                .admitted
                .implicit_systems()
                .flat_map(|i| i.unknowns.iter().copied())
                .filter(|id| !product.admitted.inputs.contains(id))
                .collect(),
            rows: product
                .admitted
                .implicit_systems()
                .flat_map(|i| i.residuals.iter().flat_map(|r| r.rows.iter().copied()))
                .filter(|id| {
                    !product
                        .admitted
                        .case()
                        .rows()
                        .iter()
                        .any(|row| row.id == *id)
                })
                .collect(),
        }
    }
    fn contains(&self, id: &SemanticId) -> bool {
        self.unknowns.contains(id) || self.rows.contains(id)
    }
}
/// Every numerical hint: target, declaration, purpose and observation row.
fn hints(
    product: &pse_compiler::workspace::PreparedModeling,
) -> Vec<(SemanticId, DeclarationId, ModelingHint, SemanticId)> {
    product
        .admitted
        .outputs
        .iter()
        .filter_map(|o| {
            if let ModelingOutput::Hint {
                target,
                declaration,
                kind,
            } = o
            {
                Some((*target, *declaration, *kind, o.row_id()))
            } else {
                None
            }
        })
        .collect()
}
fn variables(product: &pse_compiler::workspace::PreparedModeling) -> BTreeSet<SemanticId> {
    product
        .admitted
        .case()
        .variables()
        .iter()
        .map(|v| v.port.id)
        .collect()
}
fn equations(product: &pse_compiler::workspace::PreparedModeling) -> BTreeSet<SemanticId> {
    product
        .admitted
        .outputs
        .iter()
        .filter_map(|o| {
            if let ModelingOutput::Equation { id, .. } = o {
                Some(*id)
            } else {
                None
            }
        })
        .collect()
}
/// Every independent input needs a case value or a resolvable start, except the derived
/// realization parameters the bound structure determines (ADR-0104).
fn require_inputs(model: &ModelingPreparation, values: &CaseValues) -> Result<(), WorkflowError> {
    let product = model.compiled();
    if let Some(id) = product
        .admitted
        .inputs
        .iter()
        .find(|id| !values.scalars.contains_key(id) && !product.model.derived.contains_key(id))
    {
        let path = product
            .model
            .symbols
            .get(id)
            .map(|symbol| symbol.lineage.path.as_str())
            .unwrap_or("<unattributed>");
        return Err(contract(format!(
            "missing case value or resolvable start for {id} ({path})"
        )));
    }
    Ok(())
}
/// A modeling-sourced numerical requirement on `target` of `model`, declared by `source_id`.
pub(in crate::workflow) fn requirement(
    lineage: pse_model::lineage::Lineage,
    target: SemanticId,
    kind: NumericalTarget,
    source_id: DeclarationId,
    source: NumericalSource,
    nominal: Option<f64>,
    scale: Option<f64>,
) -> pse_math::numerics::SourcedRequirement {
    pse_math::numerics::SourcedRequirement {
        source,
        declaration: pse_model::numerics::NumericalRequirement {
            requirement_id: pse_ids::named_id(source_id.as_id(), &target.to_string()),
            model_id: lineage.model_id,
            case_id: lineage.case_id,
            instance_id: lineage.instance_id,
            fit_id: lineage.fit_id,
            target_id: target,
            target_kind: kind,
            nominal,
            scaling_factor: scale,
            absolute_tolerance: None,
            shared_engineering_allowance: None,
            engineering_rule_id: None,
            relative_tolerance: None,
            unit_id: None,
            coordinates: NumericalCoordinates::Physical,
            priority: 0,
            required: true,
            provenance: format!("modeling annotation {source_id}"),
        },
    }
}

/// Separate optional response capability/resource failure from cancellation and
/// infrastructure failure, which still stop the requested operation.
fn root_parametric_failure(
    cause: &crate::math::MathRuntimeError,
) -> Option<pse_backend_native::square_response::Withheld> {
    use crate::math::MathRuntimeError as E;
    use pse_backend_native::square_response::Withheld;
    match cause {
        E::Shared(cause)
        | E::Strategy { cause, .. }
        | E::StrategyTraceUnavailable { cause, .. } => root_parametric_failure(cause),
        E::Math(cause) | E::Solve(pse_backend_native::ProblemError::Math(cause)) => {
            root_math_response_failure(cause)
        }
        E::Compile(pse_compiler::workspace::CompileError::Math(cause)) => {
            root_math_response_failure(cause)
        }
        E::Cancelled
        | E::Solve(pse_backend_native::ProblemError::Cancelled)
        | E::Compile(pse_compiler::workspace::CompileError::Cancelled)
        | E::Retiring
        | E::Infrastructure(_) => None,
        E::Limit(_)
        | E::Pool(_)
        | E::Compile(pse_compiler::workspace::CompileError::Limit(_))
        | E::Solve(pse_backend_native::ProblemError::Limit { .. }) => Some(Withheld::Memory),
        _ => Some(Withheld::Neighborhood(cause.to_string())),
    }
}

fn root_math_response_failure(
    cause: &pse_math::MathError,
) -> Option<pse_backend_native::square_response::Withheld> {
    use pse_backend_native::square_response::Withheld;
    use pse_math::MathError;
    match cause {
        MathError::Instance { cause, .. } => root_math_response_failure(cause),
        MathError::Cancelled => None,
        MathError::Limit(_)
        | MathError::ByteLimit { .. }
        | MathError::SlotLimit { .. }
        | MathError::WorkLimit { .. } => Some(Withheld::Memory),
        _ => Some(Withheld::Neighborhood(cause.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::solves::Outcome;
    use std::sync::Arc;
    #[tokio::test]
    async fn authored_objective_accuracy_rejects_member_quantity_mismatch() {
        use super::super::super::tests as fixture;
        let declarations = pse_authoring::language::parse(
            "package p { def Root { var x:Length; eq root:x==1{m}; annotation start x(1{m}); let cost:Scalar=x/1{m}; annotation objective cost(minimize); annotation accuracy_goal x(optimal_objective, steady, resolution=0.1{m}); } }",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let runtime = fixture::runtime();
        let package = runtime
            .modeling_package(declarations, fixture::physical())
            .await
            .unwrap();
        let mut profile = fixture::profile();
        profile.intent = pse_backend_native::solve::SolveIntent::Optimize;
        let error = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                profile,
                NumericalInputs::default(),
                &crate::CancelSource::new(),
            )
            .await
            .expect_err("physical member quantity cannot describe a dimensionless objective");
        assert!(
            error
                .to_string()
                .contains("target quantity differs from the selected objective quantity"),
            "unexpected refusal: {error}"
        );
    }
    #[cfg(all(feature = "solver-kinsol", feature = "solver-root-isolation"))]
    fn authored_symbol(
        product: &pse_compiler::workspace::PreparedModeling,
        path: &str,
    ) -> SemanticId {
        let suffix = format!(".{path}");
        product
            .model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path == path || symbol.lineage.path.ends_with(&suffix))
            .unwrap()
            .id
    }
    #[cfg(all(
        feature = "solver-kinsol",
        feature = "solver-ipopt",
        feature = "solver-root-isolation"
    ))]
    #[tokio::test]
    async fn automatic_authored_suppliers_preserve_complete_original_case() {
        use super::super::super::tests as fixture;
        use pse_math::implicit::reconstruction::ReconstructionFactory;
        let runtime = fixture::runtime_on(
            512 << 20,
            crate::math::MathPolicy {
                worker_bytes: 128 << 20,
                workspace_bytes: 128 << 20,
                foreign_bytes: 32 << 20,
                ..Default::default()
            },
        );
        let declarations = pse_authoring::language::parse(
            "package p {def Root {param p:Scalar=1;var x:Scalar;annotation start x(1.5);annotation bounds x(0.5,3);implicit a {var y:Scalar;eq ey:y==2*x+p;annotation start y(4);annotation bounds y(1,8);}realize ra on a using nested;implicit b {var z:Scalar;eq ez:z==3*x-p;annotation start z(3.5);annotation bounds z(0.1,9);}realize rb on b using nested;eq floor:a.y+b.z>=3;let cost:Scalar=(x-2)*(x-2);annotation objective cost(minimize);annotation report cost(\"cost\");annotation report a.y(\"y\");annotation report b.z(\"z\");}}",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default()).unwrap();
        let root = declarations
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, fixture::physical())
            .await
            .unwrap();
        let defaults = SolverProfile::default();
        let solver = SolverProfile {
            presolve: pse_backend_native::presolve::Policy::Off,
            controls: pse_backend_native::solve::Controls {
                time_limit: std::time::Duration::from_secs(30),
                ..defaults.controls
            },
            ..defaults
        };
        assert!(solver.reconstruction.is_none());
        let cancel = crate::CancelSource::new();
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                solver,
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let product = prepared.model.model.compiled();
        let x = authored_symbol(product, "x");
        let y = authored_symbol(product, "a.y");
        let z = authored_symbol(product, "b.z");
        let p = authored_symbol(product, "p");
        let plan = &prepared.model.case.compiled().plan;
        assert_eq!(plan.columns().len(), 3);
        assert!(plan.columns().contains(&y) && plan.columns().contains(&z));
        assert_eq!(plan.structure().rows().len(), 3);
        assert!(!plan.columns().contains(&p));
        assert_eq!(prepared.model.values.scalars[&p], 1.0);
        assert_eq!(
            product
                .automatic_reduced_suppliers(plan)
                .unwrap()
                .iter()
                .filter(|s| matches!(s, pse_compiler::workspace::Alternative::Available(_)))
                .count(),
            2
        );
        assert_eq!(
            prepared
                .providers
                .values()
                .filter(|registration| registration
                    .source::<ReconstructionFactory>()
                    .is_some_and(|source| source.supports_reconstruction()))
                .count(),
            2
        );
        // Compare the original coupled equations/actions directly at the ordinary authored start.
        let assembly = runtime
            .shared
            .math()
            .assemble(prepared.model.case.clone())
            .await
            .unwrap();
        let start = prepared.model.values.clone();
        let (values, jacobian, objective) = runtime
            .shared
            .math()
            .with_worker(
                assembly,
                prepared.providers.clone(),
                &cancel,
                move |worker| {
                    Ok((
                        worker.constraints(&start)?,
                        worker.jacobian(&start)?.val().to_vec(),
                        worker.objective(&start)?,
                    ))
                },
            )
            .await
            .unwrap();
        assert!(values.iter().filter(|v| v.abs() < 1e-9).count() >= 2);
        assert!(jacobian.iter().any(|v| *v == -2.0));
        assert!(jacobian.iter().any(|v| *v == -3.0));
        assert!((objective - 0.25).abs() < 1e-12);
        let result = package
            .solve_case(prepared, fixture::compiler_profile(), &cancel)
            .await
            .unwrap();
        assert!(result.accepted, "diagnostic={:?}", result.diagnostic());
        let actual_x = result.values.scalars[&x];
        let actual_y = result.values.scalars[&y];
        let actual_z = result.values.scalars[&z];
        let actual_p = result.values.scalars[&p];
        let numerics = result.prepared.solve.numerics();
        let objective =
            fixture::engineering_target(numerics, NumericalTarget::Objective, SemanticId::NIL);
        let cost = (actual_x - 2.0).powi(2);
        assert!(cost <= objective.budget, "actual original cost={cost}");
        let published_cost = result
            .reports
            .iter()
            .find(|r| r.label == "cost")
            .unwrap()
            .value;
        assert!((published_cost - cost).abs() <= objective.budget);
        let original_cost = match &result.outcome {
            Outcome::Native(native) => native.candidate.as_ref().unwrap().objective.unwrap(),
            Outcome::Constant(constant) => constant.objective.unwrap(),
            Outcome::Rejected(error) => panic!("{error}"),
        };
        assert_eq!(published_cost, original_cost);
        let row_budget = |suffix: &str| {
            let id = result
                .prepared
                .model
                .model
                .compiled()
                .model
                .equations
                .iter()
                .find(|row| row.lineage.path.ends_with(suffix))
                .unwrap()
                .id;
            fixture::engineering_target(numerics, NumericalTarget::Row, id).budget
        };
        assert!((actual_y - (2.0 * actual_x + actual_p)).abs() <= row_budget(".a.ey"));
        assert!((actual_z - (3.0 * actual_x - actual_p)).abs() <= row_budget(".b.ez"));
        assert!(actual_y + actual_z >= 3.0 - row_budget(".floor"));
        let trace = result.strategy.as_ref().unwrap();
        assert!(
            trace
                .declaration
                .mechanisms
                .iter()
                .any(|m| m.kind == pse_model::strategy::MechanismKind::ReducedSpace),
            "actual mechanisms={:?}",
            trace
                .declaration
                .mechanisms
                .iter()
                .map(|mechanism| mechanism.kind)
                .collect::<Vec<_>>()
        );
        assert!(
            trace.products.iter().any(|p| p.accuracy.is_some()),
            "actual accuracy receipt count={}",
            trace
                .products
                .iter()
                .flat_map(|product| &product.evidence)
                .count()
        );
        assert!(
            trace
                .products
                .iter()
                .flat_map(|p| &p.evidence)
                .any(|e| e.derivative_order == 1),
            "actual composed action must be published; derivative orders={:?}",
            trace
                .products
                .iter()
                .flat_map(|product| &product.evidence)
                .map(|evidence| evidence.derivative_order)
                .collect::<Vec<_>>()
        );
        use pse_relations::columnar::RelationRow;
        use pse_relations::generated::runtime::solve_strategy_products;
        let tables = result.tables().unwrap();
        let rows =
            solve_strategy_products::Row::rows(&tables[&solve_strategy_products::RELATION_ID])
                .unwrap();
        let actual = trace
            .products
            .iter()
            .flat_map(|products| &products.evidence)
            .find(|evidence| evidence.derivative_order == 1)
            .unwrap();
        assert!(
            rows.iter().any(|row| row.derivative_order == 1
                && row.product_identity == actual.accuracy.product
                && row.source_structure == actual.source.structure
                && row.source_binding == actual.source.binding
                && Some(row.point) == actual.source.point
                && row.accuracy_class == actual.accuracy.class
                && row.error == actual.accuracy.error),
            "public rows must retain actual action provenance"
        );
        assert!(rows.iter().any(|row| row.derivative_order == 0));
    }
    #[cfg(all(feature = "solver-kinsol", feature = "solver-root-isolation"))]
    #[tokio::test]
    async fn automatic_complete_authored_reconstruction_uses_no_outer_kinsol() {
        use super::super::super::tests as fixture;
        let runtime = fixture::runtime_on(
            512 << 20,
            crate::math::MathPolicy {
                worker_bytes: 128 << 20,
                workspace_bytes: 128 << 20,
                foreign_bytes: 32 << 20,
                ..Default::default()
            },
        );
        let declarations = pse_authoring::language::parse(
            "package p {def Root {param p:Scalar=2;implicit a {var y:Scalar;eq ey:y==p+1;annotation start y(2.5);annotation bounds y(1,8);}realize ra on a using nested;implicit b {var z:Scalar;eq ez:z==2*p;annotation start z(3.5);annotation bounds z(1,9);}realize rb on b using nested;}}",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default()).unwrap();
        let root = declarations
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, fixture::physical())
            .await
            .unwrap();
        for selection in [
            pse_backend_native::solve::SolverSelection::Explicit(
                pse_backend_native::solve::Backend::Kinsol,
            ),
            pse_backend_native::solve::SolverSelection::Auto,
        ] {
            let mut solver = fixture::profile();
            solver.selection = selection;
            solver.presolve = pse_backend_native::presolve::Policy::Off;
            let cancel = crate::CancelSource::new();
            let prepared = package
                .prepare_solve(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    DerivativeOrder::First,
                    fixture::compiler_profile(),
                    solver,
                    NumericalInputs::default(),
                    &cancel,
                )
                .await
                .unwrap();
            assert_eq!(prepared.model.case.compiled().plan.columns().len(), 2);
            let y = authored_symbol(prepared.model.model.compiled(), "a.y");
            let z = authored_symbol(prepared.model.model.compiled(), "b.z");
            if selection
                == pse_backend_native::solve::SolverSelection::Explicit(
                    pse_backend_native::solve::Backend::Kinsol,
                )
            {
                let decision = prepared.solve.route_decision().unwrap();
                assert_eq!(
                    decision.state,
                    pse_backend_native::routing::AssessmentState::Refused
                );
                assert!(
                    decision.selected.is_none(),
                    "complete reconstruction must not fabricate native admission"
                );
                let mut declaration = prepared.solve.numerical_strategy();
                declaration.mechanisms[0].profile = Some(pse_model::strategy::ProfileRef {
                    backend: pse_backend_native::solve::Backend::Kinsol,
                    key: prepared.solve.strategy_profile().unwrap(),
                });
                let direct = prepared
                    .solve
                    .clone()
                    .with_strategy(declaration, vec![prepared.solve.clone().into()])
                    .unwrap();
                let refused = runtime
                    .native()
                    .solve(direct)
                    .unwrap()
                    .finish()
                    .await
                    .unwrap();
                assert!(
                    matches!(refused.outcome, Outcome::Rejected(ref cause)
                if matches!(cause.as_ref(), crate::math::MathRuntimeError::Solve(
                    pse_backend_native::ProblemError::RouteRefused(_)))),
                    "the refused original route cannot dispatch a native attempt"
                );
            }
            let result = package
                .solve_case(prepared, fixture::compiler_profile(), &cancel)
                .await
                .unwrap();
            assert!(result.accepted, "{result:?}");
            assert!(matches!(result.outcome, Outcome::Constant(_)), "{result:?}");
            assert!((result.values.scalars[&y] - 3.0).abs() < 1e-7);
            assert!((result.values.scalars[&z] - 4.0).abs() < 1e-7);
            assert!(
                result
                    .strategy
                    .as_ref()
                    .unwrap()
                    .declaration
                    .mechanisms
                    .iter()
                    .any(|m| m.kind == pse_model::strategy::MechanismKind::ReducedSpace)
            );
            let actual = result
                .strategy
                .as_ref()
                .unwrap()
                .rows(result.run_id, 0)
                .unwrap();
            let completed = actual
                .iter()
                .find(|row| {
                    row.mechanism == pse_model::strategy::MechanismKind::ReducedSpace
                        && row.kind == pse_model::generated::enums::NumericalEventKind::Finished
                })
                .unwrap();
            assert!(completed.backend.is_none());
            assert!(
                completed.profile_identity.is_none(),
                "complete reconstruction has no outer native execution profile"
            );
        }
    }
    #[cfg(all(feature = "solver-kinsol", feature = "solver-root-isolation"))]
    #[tokio::test]
    async fn automatic_complete_authored_reconstruction_refuses_eliminated_original_bounds() {
        use super::super::super::tests as fixture;
        let declarations = pse_authoring::language::parse(
            "package p {def Root {param p:Scalar=2;implicit a {var y:Scalar;eq ey:y==p+1;annotation start y(1);annotation bounds y(0,2);}realize ra on a using nested;}}",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default()).unwrap();
        let root = declarations
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = fixture::runtime()
            .modeling_package(declarations, fixture::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let mut solver = fixture::profile();
        solver.presolve = pse_backend_native::presolve::Policy::Off;
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                solver,
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let y = authored_symbol(prepared.model.model.compiled(), "a.y");
        let variable = prepared
            .model
            .case
            .compiled()
            .plan
            .structure()
            .variables()
            .iter()
            .find(|variable| variable.port.id == y)
            .unwrap();
        assert_eq!(variable.upper, Some(2.0));
        match package
            .solve_case(prepared, fixture::compiler_profile(), &cancel)
            .await
        {
            Err(_) => {}
            Ok(result) => assert!(
                !result.accepted,
                "a complete supplied root outside the original bound cannot be accepted: {result:?}"
            ),
        }
    }
    #[cfg(all(
        feature = "solver-kinsol",
        feature = "solver-ipopt",
        feature = "solver-root-isolation"
    ))]
    #[tokio::test]
    async fn automatic_authored_supplier_empty_original_rows_preserves_optimize_and_feasibility() {
        use super::super::super::tests as fixture;
        for objective in [true, false] {
            let cost = if objective {
                "let cost:Scalar=(x-2)*(x-2);annotation objective cost(minimize);annotation report cost(\"cost\");"
            } else {
                ""
            };
            let source = format!(
                "package p {{def Root {{var x:Scalar;annotation start x(1.5);annotation bounds x(0.5,3);implicit a {{var y:Scalar;eq ey:y==2*x;annotation start y(3);annotation bounds y(1,8);}}realize ra on a using nested;{cost}}}}}"
            );
            let declarations = pse_authoring::language::parse(
                &source,
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
            let runtime = fixture::runtime_on(
                512 << 20,
                crate::math::MathPolicy {
                    worker_bytes: 128 << 20,
                    workspace_bytes: 128 << 20,
                    foreign_bytes: 32 << 20,
                    ..Default::default()
                },
            );
            let package = runtime
                .modeling_package(declarations, fixture::physical())
                .await
                .unwrap();
            let cancel = crate::CancelSource::new();
            let mut solver = SolverProfile::default();
            if !objective {
                solver.intent = pse_backend_native::solve::SolveIntent::Initialize;
            }
            solver.presolve = pse_backend_native::presolve::Policy::Off;
            solver.controls.time_limit = std::time::Duration::from_secs(30);
            let prepared = package
                .prepare_solve(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    DerivativeOrder::First,
                    fixture::compiler_profile(),
                    solver,
                    NumericalInputs::default(),
                    &cancel,
                )
                .await
                .unwrap();
            let product = prepared.model.model.compiled();
            let x = authored_symbol(product, "x");
            let y = authored_symbol(product, "a.y");
            let plan = &prepared.model.case.compiled().plan;
            let suppliers = product.automatic_reduced_suppliers(plan).unwrap();
            let supplier = suppliers
                .iter()
                .find_map(|supplier| {
                    if let pse_compiler::workspace::Alternative::Available(supplier) = supplier {
                        Some(supplier)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_eq!(plan.structure().rows().len(), 1);
            assert_eq!(
                supplier.residuals[0].rows.len(),
                1,
                "every original row is supplied; only the original objective and coordinate bounds remain"
            );
            let result = package
                .solve_case(prepared, fixture::compiler_profile(), &cancel)
                .await
                .unwrap();
            assert!(result.accepted, "{result:?}");
            let actual_x = result.values.scalars[&x];
            let actual_y = result.values.scalars[&y];
            assert!((actual_y - 2.0 * actual_x).abs() < 1e-6);
            assert!((0.5..=3.0).contains(&actual_x));
            assert!((1.0..=8.0).contains(&actual_y));
            if objective {
                let budget = fixture::engineering_target(
                    result.prepared.solve.numerics(),
                    NumericalTarget::Objective,
                    SemanticId::NIL,
                )
                .budget;
                let cost = (actual_x - 2.0).powi(2);
                assert!(cost <= budget);
                let published_cost = result
                    .reports
                    .iter()
                    .find(|r| r.label == "cost")
                    .unwrap()
                    .value;
                assert!((published_cost - cost).abs() <= budget);
                let original_cost = match &result.outcome {
                    Outcome::Native(native) => {
                        native.candidate.as_ref().unwrap().objective.unwrap()
                    }
                    Outcome::Constant(constant) => constant.objective.unwrap(),
                    Outcome::Rejected(error) => panic!("{error}"),
                };
                assert_eq!(published_cost, original_cost);
            }
            assert!(
                result
                    .strategy
                    .as_ref()
                    .unwrap()
                    .declaration
                    .mechanisms
                    .iter()
                    .any(|mechanism| mechanism.kind
                        == pse_model::strategy::MechanismKind::ReducedSpace),
                "{result:?}"
            );
        }
    }
    #[cfg(all(feature = "solver-kinsol", feature = "solver-root-isolation"))]
    #[tokio::test]
    async fn automatic_supplier_promotion_preserves_restricted_peer() {
        use super::super::super::tests as fixture;
        let runtime = fixture::runtime_on(
            512 << 20,
            crate::math::MathPolicy {
                worker_bytes: 128 << 20,
                workspace_bytes: 128 << 20,
                foreign_bytes: 32 << 20,
                ..Default::default()
            },
        );
        let declarations = pse_authoring::language::parse(
            "package p {def Root {param p:Scalar=2;var z:Scalar;annotation start z(1);implicit a {var y:Scalar;eq ey:y==2*p;annotation start y(3);annotation bounds y(1,8);}realize ra on a using nested;implicit restricted select branch(y>0) {var y:Scalar;eq ey:y*y==p;annotation start y(1);annotation bounds y(-3,3);}realize rb on restricted using nested;eq pin:z==restricted.y;}}",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default()).unwrap();
        let root = declarations
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, fixture::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let mut solver = fixture::profile();
        solver.presolve = pse_backend_native::presolve::Policy::Off;
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                solver.clone(),
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let product = prepared.model.model.compiled();
        let a = authored_symbol(product, "a.y");
        let restricted = authored_symbol(product, "restricted.y");
        let plan = &prepared.model.case.compiled().plan;
        assert!(plan.columns().contains(&a));
        assert!(
            !plan.columns().contains(&restricted),
            "selected restricted peer must retain its nested authority"
        );
        assert_eq!(
            product
                .automatic_reduced_suppliers(plan)
                .unwrap()
                .iter()
                .filter(|supplier| matches!(
                    supplier,
                    pse_compiler::workspace::Alternative::Available(_)
                ))
                .count(),
            1
        );
        let restricted_path = product.model.symbols[&restricted].lineage.path.clone();
        let negative = ModelingCaseBindings {
            values: BTreeMap::from([(restricted_path.clone(), -1.0)]),
            variables: BTreeMap::from([(
                restricted_path,
                ModelingVariableState {
                    lower: Some(Some(-3.0)),
                    upper: Some(Some(-0.1)),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let forced = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                negative,
                DerivativeOrder::First,
                fixture::compiler_profile(),
                solver,
                NumericalInputs::default(),
                &cancel,
            )
            .await;
        match forced {
            Err(_) => {}
            Ok(prepared) => match package
                .solve_case(prepared, fixture::compiler_profile(), &cancel)
                .await
            {
                Err(_) => {}
                Ok(result) => assert!(
                    !result.accepted,
                    "negative raw branch cannot bypass the restricted peer: {result:?}"
                ),
            },
        }
    }
    #[cfg(all(feature = "solver-kinsol", feature = "solver-root-isolation"))]
    #[tokio::test]
    async fn automatic_supplier_promotion_preserves_actual_regime_peer() {
        use super::super::super::tests as fixture;
        let runtime = fixture::runtime_on(
            512 << 20,
            crate::math::MathPolicy {
                worker_bytes: 128 << 20,
                workspace_bytes: 128 << 20,
                foreign_bytes: 32 << 20,
                ..Default::default()
            },
        );
        let declarations = pse_authoring::language::parse(
            "package p {def Root {param target:Scalar=2;var z:Scalar;annotation start z(0.5);implicit a {var y:Scalar;eq ey:y==2*target;annotation start y(3);annotation bounds y(1,8);}realize ra on a using nested;implicit roots select minimum((y-target)*(y-target),1e-8) {var y:Scalar;regime negative eligible(y<0) {eq ey:y==-1;annotation start y(-0.5);annotation bounds y(-2,-0.1);}regime positive eligible(y>0) {eq ey:y==1;annotation start y(0.5);annotation bounds y(0.1,2);}}realize rb on roots using nested;eq pin:z==roots.y;annotation report roots.y(\"selected\");}}",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default()).unwrap();
        let root = declarations
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, fixture::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let mut solver = fixture::profile();
        solver.presolve = pse_backend_native::presolve::Policy::Off;
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                solver,
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let product = prepared.model.model.compiled();
        let plan = &prepared.model.case.compiled().plan;
        assert!(plan.columns().contains(&authored_symbol(product, "a.y")));
        assert!(
            !plan
                .columns()
                .contains(&authored_symbol(product, "roots.y"))
        );
        assert_eq!(
            product
                .automatic_reduced_suppliers(plan)
                .unwrap()
                .iter()
                .filter(|supplier| matches!(
                    supplier,
                    pse_compiler::workspace::Alternative::Available(_)
                ))
                .count(),
            1
        );
        assert!(
            product
                .admitted
                .implicit_systems()
                .any(|supplier| supplier.residuals.len() == 2)
        );
        let result = package
            .solve_case(prepared, fixture::compiler_profile(), &cancel)
            .await
            .unwrap();
        assert!(result.accepted, "{result:?}");
        assert!(
            (result
                .reports
                .iter()
                .find(|report| report.label == "selected")
                .unwrap()
                .value
                - 1.0)
                .abs()
                < 1e-7
        );
    }
    #[cfg(all(feature = "solver-kinsol", feature = "solver-root-isolation"))]
    #[tokio::test]
    async fn automatic_authored_suppliers_refuse_positive_dimensional_root_remainder() {
        use super::super::super::tests as fixture;
        let declarations = pse_authoring::language::parse(
            "package p {def Root {param p:Scalar=2;var x:Scalar;annotation start x(1);implicit a {var y:Scalar;eq ey:y==p;annotation start y(1);annotation bounds y(0,4);}realize ra on a using nested;}}",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default()).unwrap();
        let root = declarations
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = fixture::runtime()
            .modeling_package(declarations, fixture::physical())
            .await
            .unwrap();
        let result = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                fixture::profile(),
                NumericalInputs::default(),
                &crate::CancelSource::new(),
            )
            .await;
        assert!(
            result.is_err(),
            "underdetermined original root cannot acquire a selected inverse"
        );
    }
    #[tokio::test]
    #[cfg(feature = "solver-kinsol")]
    async fn conservation_tolerance_reaches_native_accuracy_and_original_quality() {
        use super::super::super::tests as fixture;
        use pse_backend_native::{NlpOracle, quality::Tolerances};
        use pse_model::generated::enums::NumericalProvenanceField;
        let rt = fixture::runtime();
        let mut physical = fixture::physical();
        let mut centimetre = physical
            .quantities
            .compose(&pse_quantity::UnitProduct::symbol("m"))
            .unwrap();
        centimetre.id = pse_ids::named_id(SemanticId::NIL, "conservation-test-centimetre").into();
        centimetre.symbol = "cm".into();
        centimetre.scale_to_canonical *= 0.01;
        centimetre.definition = None;
        let mut quantities = physical.quantities.to_builder();
        quantities.unit(centimetre);
        physical.quantities = Arc::new(quantities.build().unwrap());
        physical.key = pse_compiler::workspace::physical_identity(
            &physical.quantities,
            &physical.preconditions,
        );
        let declarations = pse_authoring::language::parse(
            "package p {def Root {var x:Scalar;var length:Length;annotation start x(1);annotation start length(1{m});accumulate total:Scalar conservation tolerance 1e-7;contribute total role inflow=x;contribute total role outflow=1;accumulate distance:Length conservation tolerance 0.00002{cm};contribute distance role inflow=length;contribute distance role outflow=1{m};state consistency supplied(true) {coordinate amount=x;reconstruct agreement:x==1 tolerance 1e-11;transport amount=x tolerance 1e-8;}}}",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        ).unwrap();
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(declarations, physical).await.unwrap();
        let cancel = crate::CancelSource::new();
        let mut solver = fixture::profile();
        solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
            pse_backend_native::solve::Backend::Kinsol,
        );
        let resolved = package
            .resolve_case(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                solver,
                NumericalInputs::default(),
                CaseOverrides::default(),
                false,
                &cancel,
            )
            .await
            .unwrap();
        let product = resolved.model.model.compiled();
        let plan = &resolved.model.case.compiled().plan;
        let registry = &resolved.model.case.compiled().quantities;
        let targets = plan.numerical_targets(registry).unwrap();
        let sources = conservation_row_requirements(
            product,
            &targets,
            resolved.model.model.solved().lineage(),
            registry,
        )
        .unwrap();
        assert_eq!(sources.len(), 2);
        assert!(
            product
                .model
                .closures
                .values()
                .any(|closure| closure.observation_only)
        );
        assert!(
            state_reconstruction_requirements(
                product,
                &targets,
                resolved.model.model.solved().lineage(),
                registry
            )
            .unwrap()
            .is_empty()
        );
        let scalar = sources
            .iter()
            .find(|source| source.declaration.absolute_tolerance == Some(1e-7))
            .unwrap();
        let scalar_id = scalar.declaration.target_id;
        let dimensional = sources
            .iter()
            .find(|source| source.declaration.target_id != scalar_id)
            .unwrap();
        assert!((dimensional.declaration.absolute_tolerance.unwrap() - 2e-7).abs() < 1e-21);
        for source in &sources {
            let closure = product
                .model
                .closures
                .values()
                .find(|closure| {
                    pse_ids::named_id(closure.id, "conservation") == source.declaration.target_id
                })
                .unwrap();
            assert_eq!(
                source.declaration.instance_id,
                Some(closure.lineage.instance)
            );
            assert_eq!(
                source.declaration.requirement_id,
                conservation_requirement_id(closure)
            );
            assert_eq!(source.source, NumericalSource::Model);
        }
        // Real selected-output projections cannot inherit an inactive equality's budget.
        let construction = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let projection = plan
            .functions(
                &[scalar_id],
                plan.columns().to_vec(),
                registry,
                DerivativeOrder::First,
                &construction,
            )
            .unwrap();
        let projected = conservation_row_requirements(
            product,
            &projection.numerical_targets(registry).unwrap(),
            resolved.model.model.solved().lineage(),
            registry,
        )
        .unwrap();
        assert_eq!(projected.len(), 1);
        assert_eq!(projected[0].declaration.target_id, scalar_id);
        let empty = plan
            .functions(
                &[],
                plan.columns().to_vec(),
                registry,
                DerivativeOrder::Value,
                &construction,
            )
            .unwrap();
        assert!(
            conservation_row_requirements(
                product,
                &empty.numerical_targets(registry).unwrap(),
                resolved.model.model.solved().lineage(),
                registry
            )
            .unwrap()
            .is_empty()
        );
        let mut policy = resolved.solver.numerics.clone();
        let mut tighter = scalar.declaration.clone();
        tighter.requirement_id = pse_ids::named_id(tighter.requirement_id, "analysis-override");
        tighter.absolute_tolerance = Some(5e-8);
        policy.requirements.push(tighter);
        let overridden =
            pse_math::numerics::resolve(registry, &targets, &sources, &policy).unwrap();
        assert_eq!(
            overridden
                .targets
                .iter()
                .find(|target| target.id == scalar_id && target.kind == NumericalTarget::Row)
                .unwrap()
                .budget,
            5e-8
        );
        let second = package
            .resolve_case(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                resolved.solver.clone(),
                resolved.numerical.clone(),
                CaseOverrides::default(),
                false,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(
            second.numerical.declarations.len(),
            resolved.numerical.declarations.len()
        );
        let variable = *product
            .model
            .state_specifications
            .values()
            .next()
            .unwrap()
            .coordinates
            .values()
            .next()
            .unwrap();
        let prepared = package.finish_case(resolved, &cancel).await.unwrap();
        let frozen = prepared.solve.numerics();
        let actual = frozen
            .targets
            .iter()
            .find(|target| target.id == scalar_id && target.kind == NumericalTarget::Row)
            .unwrap();
        assert_eq!(actual.budget, 1e-7);
        assert!(
            actual
                .provenance
                .iter()
                .any(|source| source.source == NumericalSource::Model
                    && source.selected
                    && source.field == NumericalProvenanceField::AbsoluteTolerance)
        );
        let executable = package
            .runtime
            .native()
            .assemble(prepared.model.case.clone())
            .await
            .unwrap();
        let plan = &prepared.model.case.compiled().plan;
        let row_ids = plan
            .structure()
            .rows()
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>();
        let row = row_ids.iter().position(|id| *id == scalar_id).unwrap();
        let column = plan
            .columns()
            .iter()
            .position(|id| *id == variable)
            .unwrap();
        let normalization =
            pse_math::normalization::Normalization::from_policy(frozen, plan.columns(), &row_ids)
                .unwrap();
        let tolerances = prepared.solve.tolerances();
        assert_eq!(tolerances.rows[row], 1e-7);
        let normalized = tolerances.normalized(&normalization).unwrap();
        assert_eq!(
            prepared.solve.accuracy().feasibility,
            normalized
                .variables
                .iter()
                .chain(&normalized.rows)
                .copied()
                .reduce(f64::min)
                .unwrap()
        );
        assert!(prepared.solve.accuracy().feasibility <= 1e-7 / normalization.rows[row]);
        let budget =
            crate::math::WorkerBudget::new(package.runtime.shared.budget().math.worker_bytes);
        let _charge = budget
            .charge(executable.assembly.numeric_worker_bytes())
            .unwrap();
        let execution =
            pse_backend_native::solve::Execution::new(construction, &prepared.profile.controls);
        let worker = executable
            .assembly
            .worker_scoped(BTreeMap::new(), execution.scope().unwrap());
        let mut oracle = pse_backend_native::assembled::AlgebraicOracle::new(
            worker,
            prepared.model.values.clone(),
        )
        .unwrap();
        let mut point = plan
            .columns()
            .iter()
            .map(|id| prepared.model.values.scalars[id])
            .collect::<Vec<_>>();
        let mut values = vec![0.; row_ids.len()];
        for (error, feasible) in [(0.5e-7, true), (1.5e-7, false)] {
            point[column] = 1. + error;
            oracle.constraints(&point, &mut values).unwrap();
            let quality = pse_backend_native::quality::observed(
                oracle.contract(),
                oracle.constraint_bounds(),
                &point,
                &values,
                tolerances,
            )
            .unwrap();
            assert_eq!(quality.feasible(), feasible);
            assert_eq!(quality.rows[row].tolerance, 1e-7);
        }
        // The frozen native tolerance and an independent lowering agree on row order.
        assert_eq!(
            Tolerances::from_policy(frozen, plan.columns(), &row_ids)
                .unwrap()
                .rows,
            tolerances.rows
        );
    }
    #[tokio::test]
    #[cfg(feature = "solver-kinsol")]
    async fn fixed_implicit_inputs_keep_value_provider_and_actual_free_jacobian() {
        use super::super::super::tests as fixture;
        use pse_backend_native::solve::{Backend, SolverSelection};
        let rows = pse_authoring::language::parse(
            "package p {def Root {var p:Scalar;var x:Scalar;implicit root select operational(y=1) settings(\"native.kinsol.v1\") {var y:Scalar;eq e:y*y==p;annotation start y(1);annotation bounds y(0.1,10);}realize policy on root using nested;eq e:x==root.y;annotation start x(2);}}",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        ).unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let rt = fixture::runtime_with(128 << 20, 16 << 20, 1 << 30);
        let package = rt
            .modeling_package(rows, fixture::physical())
            .await
            .unwrap();
        let mut analysis = ModelingAnalysis {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings: Bindings::default(),
            limits: Limits::default(),
            case: ModelingCaseBindings {
                values: BTreeMap::from([("p".into(), 4.)]),
                variables: BTreeMap::from([(
                    "p".into(),
                    ModelingVariableState {
                        fixed: Some(true),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            },
            order: DerivativeOrder::First,
            compiler: fixture::compiler_profile(),
            solver: fixture::profile(),
            numerical: NumericalInputs::default(),
        };
        analysis.solver.selection = SolverSelection::Explicit(Backend::Kinsol);
        let cancel = crate::CancelSource::new();
        let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
        let product = prepared.model.model.compiled();
        let inner = product.admitted.implicit_systems().next().unwrap();
        assert_eq!(inner.descriptor.spec().derivatives, DerivativeOrder::Value);
        let key = inner.descriptor.spec().key();
        assert_eq!(
            prepared.providers[&key].descriptor().spec().derivatives,
            DerivativeOrder::Value
        );
        let plan = &prepared.model.case.compiled().plan;
        assert_eq!(plan.columns().len(), 1);
        assert_eq!(plan.jacobian_pattern().compute_nnz(), 1);
        assert_eq!(
            product
                .admitted
                .provider_demands_for_plan(plan, DerivativeOrder::First)
                .unwrap()[&key],
            DerivativeOrder::Value
        );

        // An explicit response coordinate consumes the otherwise fixed input. It
        // requires the selected provider's genuine derivative neighborhood.
        let input = product.model.paths["p"];
        let response = plan
            .functions(
                &plan
                    .structure()
                    .rows()
                    .iter()
                    .map(|row| row.id)
                    .collect::<Vec<_>>(),
                vec![input],
                &package.quantities,
                DerivativeOrder::First,
                &Arc::new(std::sync::atomic::AtomicBool::new(false)),
            )
            .unwrap();
        assert_eq!(
            product
                .admitted
                .provider_demands_for_plan(&response, DerivativeOrder::First)
                .unwrap()[&key],
            DerivativeOrder::First
        );
        let refused = package
            .inner_registrations(
                prepared.model.model.clone(),
                &analysis.case,
                &analysis.numerical,
                &analysis.solver.numerics,
                &analysis.solver.controls,
                DerivativeOrder::First,
                analysis.compiler,
                &cancel,
                implicit::ProviderDemand::Case(&response),
            )
            .await
            .unwrap_err();
        assert!(
            refused
                .boundary_diagnostic()
                .observations
                .contains_key("requested_derivative_order")
        );

        let result = package
            .solve_case(prepared, analysis.compiler, &cancel)
            .await
            .unwrap();
        assert!(result.completion.decision.permits_use());
        let Outcome::Native(report) = &result.outcome else {
            panic!("actual native original corrector required");
        };
        assert!((report.candidate.as_ref().unwrap().primal[0] - 2.).abs() < 1e-8);

        // The same source with a genuinely free input cannot borrow the fixed
        // callback's Value admission to qualify a derivative request.
        analysis.case.variables.get_mut("p").unwrap().fixed = Some(false);
        let refused = package
            .prepare_analysis(&analysis, &cancel)
            .await
            .unwrap_err();
        let diagnostic = refused.boundary_diagnostic();
        assert_eq!(
            diagnostic.rule,
            pse_diagnostics::DiagnosticRule::MathProvider
        );
        assert!(matches!(
            diagnostic.observations["requested_derivative_order"],
            pse_model::diagnostic::Observation::Integer(1)
        ));
        assert!(matches!(
            diagnostic.observations["available_derivative_order"],
            pse_model::diagnostic::Observation::Integer(0)
        ));
        assert!(!diagnostic.sources.is_empty());
    }
    #[tokio::test]
    #[cfg(feature = "solver-kinsol")]
    async fn state_reconstruction_tolerance_reaches_native_accuracy_and_original_quality() {
        use pse_backend_native::{NlpOracle, quality::Tolerances, solve::ResolvedAccuracy};
        use pse_model::generated::enums::NumericalProvenanceField;
        let rt = super::super::super::tests::runtime();
        let mut physical = super::super::super::tests::physical();
        let mut centimetre = physical
            .quantities
            .compose(&pse_quantity::UnitProduct::symbol("m"))
            .unwrap();
        centimetre.id = pse_ids::named_id(SemanticId::NIL, "accuracy-test-centimetre").into();
        centimetre.symbol = "cm".into();
        centimetre.scale_to_canonical *= 0.01;
        centimetre.definition = None;
        let mut quantities = physical.quantities.to_builder();
        quantities.unit(centimetre);
        physical.quantities = Arc::new(quantities.build().unwrap());
        physical.key = pse_compiler::workspace::physical_identity(
            &physical.quantities,
            &physical.preconditions,
        );
        let declarations = pse_authoring::language::parse(
            "package p { def Root { var x:Scalar; var length:Length; annotation start x(1); annotation start length(1{m}); state s supplied(false) { coordinate amount=x; reconstruct normalization:x==1 tolerance 1e-9; reconstruct extent:length==1{m} tolerance 0.0000002{cm}; transport amount=x tolerance 1e-8; } } }",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        ).unwrap();
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(declarations, physical).await.unwrap();
        let cancel = crate::CancelSource::new();
        let mut solver = super::super::super::tests::profile();
        solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
            pse_backend_native::solve::Backend::Kinsol,
        );
        let resolved = package
            .resolve_case(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                super::super::super::tests::compiler_profile(),
                solver,
                NumericalInputs::default(),
                CaseOverrides::default(),
                false,
                &cancel,
            )
            .await
            .unwrap();
        let product = resolved.model.model.compiled();
        let specification = product.model.state_specifications.values().next().unwrap();
        assert_eq!(specification.reconstructions.len(), 2);
        let row_ids = resolved
            .model
            .case
            .compiled()
            .plan
            .structure()
            .rows()
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>();
        assert_eq!(row_ids.len(), 2);
        assert_eq!(specification.coordinates.len(), 1);
        let x = *specification.coordinates.values().next().unwrap();
        // Follow the exact authored reconstruction identity, independent of native row order.
        let normalization_row = specification
            .reconstructions
            .iter()
            .find(|(_, tolerance)| {
                matches!(tolerance, pse_modeling::specialize::Value::Number { bits, .. }
                if *bits == 1e-9_f64.to_bits())
            })
            .unwrap()
            .0
            .id;
        let registry = &resolved.model.case.compiled().quantities;
        let target = resolved
            .numerics
            .targets
            .iter()
            .find(|target| target.kind == NumericalTarget::Row && target.id == normalization_row)
            .unwrap();
        assert_eq!(target.absolute, 1e-9);
        assert_eq!(target.budget, 1e-9);
        assert!(target.provenance.iter().any(|provenance| provenance.source
            == NumericalSource::Model
            && provenance.field == NumericalProvenanceField::AbsoluteTolerance
            && provenance.selected
            && provenance.declaration.is_some()));
        let dimensional = resolved
            .numerics
            .targets
            .iter()
            .find(|other| other.kind == NumericalTarget::Row && other.id != target.id)
            .unwrap();
        assert!(
            (dimensional.budget - 2e-9).abs() < 1e-23,
            "authored centimetre tolerance is canonical metres: {}",
            dimensional.budget
        );
        let variables = resolved.model.case.compiled().plan.columns().to_vec();
        let tolerances = Tolerances::from_policy(&resolved.numerics, &variables, &row_ids).unwrap();
        let normalization = pse_math::normalization::Normalization::from_policy(
            &resolved.numerics,
            &variables,
            &row_ids,
        )
        .unwrap();
        let accuracy =
            ResolvedAccuracy::resolve(&resolved.solver.numerics, &tolerances, &normalization)
                .unwrap();
        let scalar_row = row_ids.iter().position(|id| *id == target.id).unwrap();
        assert_eq!(tolerances.rows[scalar_row], 1e-9);
        let normalized = tolerances.normalized(&normalization).unwrap();
        assert_eq!(
            accuracy.feasibility,
            normalized
                .variables
                .iter()
                .chain(&normalized.rows)
                .copied()
                .reduce(f64::min)
                .unwrap()
        );
        assert!(accuracy.feasibility <= 1e-9 / normalization.rows[scalar_row]);
        let service = package.runtime.native();
        let executable = service.assemble(resolved.model.case.clone()).await.unwrap();
        assert!(resolved.providers.is_empty());
        let budget =
            crate::math::WorkerBudget::new(package.runtime.shared.budget().math.worker_bytes);
        let _charge = budget
            .charge(executable.assembly.numeric_worker_bytes())
            .unwrap();
        let execution = pse_backend_native::solve::Execution::new(
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
            &resolved.solver.controls,
        );
        let worker = executable
            .assembly
            .worker_scoped(BTreeMap::new(), execution.scope().unwrap());
        let mut oracle = pse_backend_native::assembled::AlgebraicOracle::new(
            worker,
            resolved.model.values.clone(),
        )
        .unwrap();
        let mut point = variables
            .iter()
            .map(|id| resolved.model.values.scalars[id])
            .collect::<Vec<_>>();
        let x_column = variables.iter().position(|id| *id == x).unwrap();
        let mut rows = vec![0.; row_ids.len()];
        for (error, feasible) in [(0.5e-9, true), (1.5e-9, false)] {
            point[x_column] = 1. + error;
            oracle.constraints(&point, &mut rows).unwrap();
            let quality = pse_backend_native::quality::observed(
                oracle.contract(),
                oracle.constraint_bounds(),
                &point,
                &rows,
                &tolerances,
            )
            .unwrap();
            assert_eq!(quality.feasible(), feasible);
            assert_eq!(quality.rows[scalar_row].tolerance, 1e-9);
        }
        let targets = resolved
            .model
            .case
            .compiled()
            .plan
            .numerical_targets(registry)
            .unwrap();
        let sources = state_reconstruction_requirements(
            product,
            &targets,
            resolved.model.model.solved().lineage(),
            registry,
        )
        .unwrap();
        assert_eq!(sources.len(), 2);
        assert!(
            sources.iter().all(
                |source| source.declaration.instance_id == Some(specification.lineage.instance)
            )
        );
        let absent = targets
            .iter()
            .filter(|target| target.kind != NumericalTarget::Row)
            .cloned()
            .collect::<Vec<_>>();
        assert!(
            state_reconstruction_requirements(
                product,
                &absent,
                resolved.model.model.solved().lineage(),
                registry
            )
            .unwrap()
            .is_empty()
        );
        let mut policy = resolved.solver.numerics.clone();
        let mut tighter = sources
            .iter()
            .find(|source| source.declaration.target_id == target.id)
            .unwrap()
            .declaration
            .clone();
        tighter.requirement_id = pse_ids::named_id(tighter.requirement_id, "analysis-override");
        tighter.absolute_tolerance = Some(5e-10);
        policy.requirements.push(tighter);
        let overridden =
            pse_math::numerics::resolve(registry, &targets, &sources, &policy).unwrap();
        assert_eq!(
            overridden
                .targets
                .iter()
                .find(|other| other.kind == NumericalTarget::Row && other.id == target.id)
                .unwrap()
                .budget,
            5e-10
        );
        // Reusing returned inputs must not multiply this producer's declarations.
        let second_resolution = package
            .resolve_case(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                super::super::super::tests::compiler_profile(),
                resolved.solver.clone(),
                resolved.numerical.clone(),
                CaseOverrides::default(),
                false,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(
            second_resolution.numerical.declarations.len(),
            resolved.numerical.declarations.len()
        );
        let target_id = target.id;
        let prepared = package.finish_case(resolved, &cancel).await.unwrap();
        let actual = prepared
            .solve
            .numerics()
            .targets
            .iter()
            .find(|target| target.kind == NumericalTarget::Row && target.id == target_id)
            .unwrap();
        assert_eq!(actual.budget, 1e-9);
        assert!(
            actual
                .provenance
                .iter()
                .any(|source| source.source == NumericalSource::Model
                    && source.selected
                    && source.field == NumericalProvenanceField::AbsoluteTolerance)
        );
        assert_eq!(prepared.solve.accuracy().feasibility, accuracy.feasibility);
        assert_eq!(prepared.solve.tolerances().rows[scalar_row], 1e-9);
    }
    #[tokio::test]
    async fn supplied_state_consistency_does_not_create_a_phantom_row_requirement() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let declarations = pse_authoring::language::parse(
            "package p { def Root { var x:Scalar; annotation start x(1); eq other:x==1; state s supplied(true) { coordinate amount=x; reconstruct normalization:x==1 tolerance 1e-9; transport amount=x tolerance 1e-8; } } }",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        ).unwrap();
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(declarations, physical).await.unwrap();
        let resolved = package
            .resolve_case(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                super::super::super::tests::compiler_profile(),
                super::super::super::tests::profile(),
                NumericalInputs::default(),
                CaseOverrides::default(),
                false,
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let product = resolved.model.model.compiled();
        let specification = product.model.state_specifications.values().next().unwrap();
        assert!(specification.supplied);
        let (row, _) = &specification.reconstructions[0];
        assert!(
            product
                .model
                .closures
                .contains_key(&pse_ids::named_id(row.id, "consistency"))
        );
        let registry = &resolved.model.case.compiled().quantities;
        let targets = resolved
            .model
            .case
            .compiled()
            .plan
            .numerical_targets(registry)
            .unwrap();
        assert!(
            !targets
                .iter()
                .any(|target| target.kind == NumericalTarget::Row && target.id == row.id)
        );
        assert!(
            state_reconstruction_requirements(
                product,
                &targets,
                resolved.model.model.solved().lineage(),
                registry
            )
            .unwrap()
            .is_empty()
        );
        let unrelated = resolved
            .numerics
            .targets
            .iter()
            .find(|target| target.kind == NumericalTarget::Row)
            .unwrap();
        assert_eq!(
            unrelated.budget,
            pse_model::numerics::DEFAULT_ENGINEERING_ACCURACY
        );
        assert!(unrelated.engineering.as_ref().unwrap().canonical_fallback);
        assert!(!unrelated.provenance.iter().any(|source| source.selected
            && source.declaration == Some(state_reconstruction_requirement_id(row))));
    }
    #[tokio::test]
    async fn kernel_starts_numerics_and_constant_solver_share_the_existing_pipeline() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let rows=pse_authoring::language::parse(
            "package p { def Root { param p: Scalar = 2; var x: Scalar; var y: Scalar; let bad: Scalar = log(-1); eq balance: x+y == 6; annotation start x(p); annotation start y(2*x); annotation bounds x(0,10); annotation nominal x(2); annotation nominal y(4); annotation scale balance(inverseSum); annotation check x(x > 2.5); annotation report y(\"computed y\"); } }",
            SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default()).unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(rows, physical).await.unwrap();
        let mut bindings = Bindings::default();
        bindings.demand.push("bad".into());
        let case = ModelingCaseBindings {
            members: BTreeMap::new(),
            values: BTreeMap::new(),
            variables: BTreeMap::from([
                (
                    "x".into(),
                    ModelingVariableState {
                        fixed: Some(true),
                        ..Default::default()
                    },
                ),
                (
                    "y".into(),
                    ModelingVariableState {
                        fixed: Some(true),
                        ..Default::default()
                    },
                ),
            ]),
        };
        let p = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                bindings,
                Limits::default(),
                case.clone(),
                DerivativeOrder::Second,
                super::super::super::tests::compiler_profile(),
                super::super::super::tests::profile(),
                NumericalInputs::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(p.model.case.compiled().facts.variables, 0);
        let x = p.model.model.compiled().model.paths["x"];
        let y = p.model.model.compiled().model.paths["y"];
        assert_eq!(p.model.values.scalars[&x], 2.);
        assert_eq!(p.model.values.scalars[&y], 4.);
        assert!(matches!(p.starts[&x], StartSource::Annotation { .. }));
        assert_eq!(
            p.model
                .case
                .compiled()
                .plan
                .structure()
                .variables()
                .iter()
                .find(|v| v.port.id == x)
                .unwrap()
                .lower,
            Some(0.)
        );
        let row = p
            .solve
            .numerics()
            .targets
            .iter()
            .find(|t| t.kind == NumericalTarget::Row)
            .unwrap();
        assert!((row.nominal - 12.).abs() < 1e-12);
        assert!(
            row.provenance
                .iter()
                .any(|p| p.source == NumericalSource::DerivedNominal && p.selected)
        );
        let admission_identity = p.admission_identity().unwrap();
        let run = p.start().unwrap().wait().await.unwrap();
        let super::super::super::RunReport::Modeling(reports) = run.report().unwrap() else {
            panic!("modeling result expected");
        };
        let report = &reports[0];
        use pse_relations::columnar::RelationRow;
        let admission = pse_relations::generated::runtime::route_decisions::Row::rows(
            &run.table("runtime.route_decisions").unwrap(),
        )
        .unwrap();
        let structure = pse_relations::generated::runtime::structural_assessments::Row::rows(
            &run.table("runtime.structural_assessments").unwrap(),
        )
        .unwrap();
        assert_eq!(admission.len(), 1);
        assert_eq!(admission[0].request_identity, admission_identity);
        assert_eq!(
            admission[0].selected,
            Some(pse_model::generated::enums::NativeRouteKind::Constant)
        );
        assert!(admission[0].refusal.is_none());
        assert_eq!(structure.len(), 1);
        assert_eq!(structure[0].request_identity, admission_identity);
        assert!(structure[0].admitted);
        assert!(structure[0].variables.is_empty());
        assert!(!run.usable()); // The model check refuses the result independently of native admission.
        assert!(matches!(&report.outcome,Outcome::Constant(r) if r.quality.feasible()));
        assert!(!report.accepted);
        assert!(report.validation_error.is_none());
        assert!(report.checks.iter().any(|c| c.kind
            == pse_model::generated::enums::ModelingCheckKind::Check
            && !c.satisfied));
        assert_eq!(
            report
                .reports
                .iter()
                .find(|r| r.label == "computed y")
                .unwrap()
                .value,
            4.
        );
        let mut override_case = case;
        override_case.values.insert("x".into(), 3.);
        let p = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                override_case,
                DerivativeOrder::Second,
                super::super::super::tests::compiler_profile(),
                super::super::super::tests::profile(),
                NumericalInputs::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(p.model.values.scalars[&y], 6.);
        assert_eq!(p.starts[&x], StartSource::Case { path: "x".into() });
    }
    #[tokio::test]
    async fn root_refuses_free_integer() {
        use super::super::super::tests as fixture;
        let rows = pse_authoring::language::parse(
            "package p { def Root { var n: Count in integer; eq e: n == 2{1}; annotation start n(0{1}); annotation bounds n(0{1}, 5{1}); } }",
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
        let package = fixture::runtime()
            .modeling_package(rows, fixture::physical())
            .await
            .unwrap();
        let error = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                fixture::profile(),
                NumericalInputs::default(),
                &crate::CancelSource::new(),
            )
            .await
            .err()
            .unwrap();
        assert_eq!(
            fixture::free_discrete_refusal(&error),
            ("n".into(), "root".into())
        );
        // Fixed by the case, the same model is an admitted continuous square system.
        let fixed = ModelingCaseBindings {
            members: BTreeMap::new(),
            values: BTreeMap::from([("n".into(), 2.)]),
            variables: BTreeMap::from([(
                "n".into(),
                ModelingVariableState {
                    fixed: Some(true),
                    ..Default::default()
                },
            )]),
        };
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                fixed,
                DerivativeOrder::First,
                fixture::compiler_profile(),
                fixture::profile(),
                NumericalInputs::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(prepared.model.case.compiled().facts.variables, 0);
    }
    /// A report in a composite unit carries the unit's spelling-independent product
    /// identity (ADR-0124): the composed literal, the canonical unit and the report agree.
    #[tokio::test]
    async fn report_in_a_composite_unit_carries_its_identity() {
        use super::super::super::tests as fixture;
        let physical = fixture::physical();
        let quantities = Arc::clone(&physical.quantities);
        let molar_cp = pse_quantity::QuantityTypeId::from_id(
            SemanticId::parse_hex("cd653ba98fa94d16b5d66b363f21c3d6").unwrap(),
        );
        let rows = pse_authoring::language::parse(
            "package p { def Root { param cp: MolarCp = 75.3{J/(K*mol)}; var y: MolarCp; eq e: y == cp; annotation start y(cp); annotation report y(\"cp\"); } }",
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
        let package = fixture::runtime()
            .modeling_package(rows, physical)
            .await
            .unwrap();
        let fixed = ModelingCaseBindings {
            members: BTreeMap::new(),
            values: BTreeMap::new(),
            variables: BTreeMap::from([(
                "y".into(),
                ModelingVariableState {
                    fixed: Some(true),
                    ..Default::default()
                },
            )]),
        };
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                fixed,
                DerivativeOrder::First,
                fixture::compiler_profile(),
                fixture::profile(),
                NumericalInputs::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let result = package
            .solve_case(
                prepared,
                fixture::compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let report = result.reports.iter().find(|r| r.label == "cp").unwrap();
        assert_eq!(report.value, 75.3);
        assert_eq!(report.quantity_id, molar_cp.as_id());
        let spelled = |text: &str| {
            let pse_authoring::dsl::ExprKind::Number(number) =
                pse_authoring::dsl::parse_expr(text).unwrap().kind
            else {
                panic!("{text}");
            };
            quantities
                .compose(&number.unit.unwrap())
                .unwrap()
                .id
                .as_id()
        };
        let canonical = quantities.quantity_type(molar_cp).unwrap().canonical_unit;
        assert_eq!(report.unit_id, canonical.as_id());
        for text in ["1{J/(mol*K)}", "1{J/(K*mol)}", "1{J*K^-1*mol^-1}"] {
            assert_eq!(report.unit_id, spelled(text), "{text}");
        }
        let atomic = |symbol: &str| {
            quantities
                .compose(&pse_quantity::UnitProduct::symbol(symbol))
                .unwrap()
                .id
        };
        let minus_one = pse_quantity::Ratio::new(-1, 1).unwrap();
        let factors = pse_quantity::unit::canonical_factors([
            pse_quantity::UnitFactor {
                unit: atomic("J"),
                exponent: pse_quantity::Ratio::ONE,
            },
            pse_quantity::UnitFactor {
                unit: atomic("K"),
                exponent: minus_one,
            },
            pse_quantity::UnitFactor {
                unit: atomic("mol"),
                exponent: minus_one,
            },
        ])
        .unwrap();
        assert_eq!(
            report.unit_id,
            pse_quantity::unit_product_id(&factors).as_id()
        );
    }
}

#[cfg(test)]
#[cfg(feature = "solver-kinsol")]
mod native_tests {
    use super::*;
    #[tokio::test]
    async fn kernel_native_root_solves_and_qualifies_the_authored_model() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let allowance = pse_model::numerics::DEFAULT_ENGINEERING_ACCURACY;
        let source = format!(
            "package p {{ def Root {{ var x:Scalar; eq e:x*x==4; annotation start x(1); annotation bounds x(0.5,3); annotation nominal x(2); annotation valid x(0.5,3); annotation check x(x>0); annotation report x(\"root\"); expect x == 2 tolerance {allowance}; }} }}"
        );
        let rows = pse_authoring::language::parse(
            &source,
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
        let package = rt.modeling_package(rows, physical).await.unwrap();
        let mut profile = super::super::super::tests::profile();
        profile.selection = pse_backend_native::solve::SolverSelection::Explicit(
            pse_backend_native::solve::Backend::Kinsol,
        );
        let compiler = super::super::super::tests::compiler_profile();
        let cancel = crate::CancelSource::new();
        assert!(
            package
                .prepare_solve(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    ModelingCaseBindings::default(),
                    DerivativeOrder::Second,
                    compiler,
                    profile.clone(),
                    NumericalInputs::default(),
                    &cancel
                )
                .await
                .is_err()
        );
        let case = ModelingCaseBindings {
            members: BTreeMap::new(),
            values: BTreeMap::new(),
            variables: BTreeMap::from([(
                "x".into(),
                ModelingVariableState {
                    fixed: None,
                    lower: Some(Some(0.)),
                    upper: Some(None),
                },
            )]),
        };
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                case,
                DerivativeOrder::Second,
                compiler,
                profile,
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let result = package
            .solve_case(prepared, compiler, &cancel)
            .await
            .unwrap();
        assert!(
            result.accepted,
            "{:?}; {:?}",
            result.validation_error, result.outcome
        );
        let model = &result.prepared.model.model.compiled().model;
        let x = model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path.ends_with(".x"))
            .unwrap()
            .id;
        let row = model
            .equations
            .iter()
            .find(|row| row.lineage.path.ends_with(".e"))
            .unwrap()
            .id;
        let target = super::super::super::tests::engineering_target(
            result.prepared.solve.numerics(),
            NumericalTarget::Variable,
            x,
        );
        assert_eq!(target.budget, allowance);
        let value = result
            .reports
            .iter()
            .find(|r| r.label == "root")
            .unwrap()
            .value;
        assert!((value - 2.0).abs() <= target.budget);
        let row_budget = super::super::super::tests::engineering_target(
            result.prepared.solve.numerics(),
            NumericalTarget::Row,
            row,
        )
        .budget;
        assert!((value * value - 4.0).abs() <= row_budget);
        assert!(result.prepared.profile.sensitivity.is_none());
        assert_eq!(
            result.prepared.profile.intent,
            pse_backend_native::solve::SolveIntent::Root
        );
        assert!(result.checks.iter().any(|c| c.kind
            == pse_model::generated::enums::ModelingCheckKind::Expectation
            && c.satisfied));
    }
}

#[cfg(all(test, feature = "solver-kinsol"))]
mod root_unavailable_tests {
    use super::*;
    use crate::math::settings::SensitivityRequest;
    use crate::workflow::tests as fixture;
    use pse_backend_native::solve::{Backend, SolveIntent, SolverSelection};
    use pse_relations::{
        columnar::RelationRow,
        generated::{
            enums::{NativeQualification, WithheldReason},
            runtime::{local_validity, parametric_sensitivities, solve_runs},
        },
    };

    async fn fixed_guard() -> Result<
        (ModelingPackage, ModelingAnalysis, SemanticId),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let text = "package p { def Root { param p: Scalar=0; var x: Scalar; eq root: x==(if p>0 then 1 else -1); annotation start x(-1); } }";
        let rows = pse_authoring::language::parse(
            text,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )?;
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .ok_or_else(|| std::io::Error::other("Root declaration absent"))?
            .declaration_id;
        let package = fixture::runtime_with(16 << 20, 1 << 20, 1 << 30)
            .modeling_package(rows, fixture::physical())
            .await?;
        let mut solver = fixture::profile();
        solver.intent = SolveIntent::Root;
        solver.selection = SolverSelection::Explicit(Backend::Kinsol);
        let mut analysis = ModelingAnalysis {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings: Bindings::default(),
            limits: Limits::default(),
            case: Default::default(),
            order: DerivativeOrder::First,
            compiler: fixture::compiler_profile(),
            solver,
            numerical: NumericalInputs::default(),
        };
        analysis.bindings.demand.push("p".into());
        let cancel = crate::CancelSource::new();
        let prepared = package.prepare_analysis(&analysis, &cancel).await?;
        let parameter = prepared.model.model.compiled().model.paths["p"];
        // A fixed parameter guard is valid for the base x derivative. Making p a
        // derivative coordinate crosses a discontinuous selector and refuses compilation.
        assert!(
            package
                .parametric_program(
                    &prepared.model.model,
                    &prepared.model,
                    &[parameter],
                    DerivativeOrder::First,
                    analysis.compiler,
                    &cancel
                )
                .await
                .is_err()
        );
        Ok((package, analysis, parameter))
    }
    #[tokio::test]
    async fn unavailable_parametric_guard_keeps_public_base_root() {
        let (package, mut analysis, parameter) = fixed_guard().await.unwrap();
        analysis.solver.sensitivity = Some(SensitivityRequest {
            parameters: vec![parameter],
            reduced_hessian: false,
            propagation: None,
        });
        let result = package
            .prepare_analysis(&analysis, &crate::CancelSource::new())
            .await
            .unwrap()
            .start()
            .unwrap()
            .wait()
            .await
            .unwrap();
        let validity =
            local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
        assert_eq!(validity.len(), 1);
        assert!(!validity[0].validity.certified);
        assert_eq!(
            validity[0].validity.reason,
            Some(WithheldReason::NeighborhoodUnavailable)
        );
        assert!(
            parametric_sensitivities::Row::rows(
                &result.table("runtime.parametric_sensitivities").unwrap()
            )
            .unwrap()
            .is_empty()
        );
        assert!(
            solve_runs::Row::rows(&result.table("runtime.solve_runs").unwrap())
                .unwrap()
                .iter()
                .any(|run| run.qualification == NativeQualification::Feasible)
        );
    }
    #[tokio::test]
    async fn unavailable_response_memory_keeps_public_base_root() {
        let (package, mut analysis, parameter) = fixed_guard().await.unwrap();
        analysis.solver.sensitivity = Some(SensitivityRequest {
            parameters: vec![parameter],
            reduced_hessian: false,
            propagation: None,
        });
        let mut prepared = package
            .prepare_analysis(&analysis, &crate::CancelSource::new())
            .await
            .unwrap();
        // The common attachment also transports a typed resource refusal, rather than
        // requiring a fabricated compiled program or dropping the requested quantity.
        prepared.solve = prepared
            .solve
            .with_sensitivity(ParametricPreparation::Unavailable(
                pse_backend_native::square_response::Withheld::Memory,
            ))
            .unwrap();
        let result = prepared.start().unwrap().wait().await.unwrap();
        let validity =
            local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
        assert_eq!(
            validity[0].validity.reason,
            Some(WithheldReason::AnalysisUnavailable)
        );
        assert!(!validity[0].validity.certified);
        assert!(
            solve_runs::Row::rows(&result.table("runtime.solve_runs").unwrap())
                .unwrap()
                .iter()
                .any(|run| run.qualification == NativeQualification::Feasible)
        );
    }
    #[tokio::test]
    async fn malformed_root_parameter_request_still_refuses_preparation() {
        let (package, mut analysis, parameter) = fixed_guard().await.unwrap();
        for parameters in [vec![SemanticId::NIL], vec![parameter, parameter]] {
            analysis.solver.sensitivity = Some(SensitivityRequest {
                parameters,
                reduced_hessian: false,
                propagation: None,
            });
            assert!(
                package
                    .prepare_analysis(&analysis, &crate::CancelSource::new())
                    .await
                    .is_err()
            );
        }
    }
}
