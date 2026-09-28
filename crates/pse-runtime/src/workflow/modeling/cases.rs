// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Case interpretation delegates expression execution, numerical policy and solving to their owners.
use super::*;
use crate::math::{
    modeling::ModelingCasePreparation,
    solves::{NumericalInputs, PreparedSolve, SolverProfile},
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
    values: BTreeMap<SemanticId, f64>,
    _owner: std::sync::Arc<pse_columnar::AllocationLease>,
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
}
/// One immutable case plus the numerical declarations selected for this analysis.
#[derive(Clone, Debug)]
pub struct ModelingSolvePreparation {
    pub model: ModelingCasePreparation,
    pub solve: PreparedSolve,
    /// Typed case/default/start provenance for every resolved input.
    pub starts: BTreeMap<SemanticId, StartSource>,
    pub(in crate::workflow) providers:
        BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    pub(in crate::workflow) source: ModelingPackage,
    pub(in crate::workflow) compiler: Profile,
    pub(in crate::workflow) instance: InstanceId,
    pub(in crate::workflow) profile: SolverProfile,
}
/// Numerical resolution precedes native routing so diagnostics can inspect an
/// underdetermined or otherwise ineligible problem without requesting a solver.
pub(in crate::workflow) struct ModelingCaseResolution {
    pub compiler: Profile,
    pub instance: InstanceId,
    pub model: ModelingCasePreparation,
    pub starts: BTreeMap<SemanticId, StartSource>,
    pub providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    pub numerical: NumericalInputs,
    pub solver: SolverProfile,
    pub numerics: std::sync::Arc<pse_model::numerics::ResolvedNumericalPolicy>,
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
                &values,
                &ModelingCaseBindings::default(),
                &NumericalInputs::default(),
                &pse_model::numerics::NumericalPolicy::default(),
                &pse_backend_native::solve::Controls::default(),
                profile,
                cancel,
                Some(&rows),
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
        let (values, owner) = service
            .with_worker(assembly, providers, cancel, move |worker| {
                let observed = worker.constraints(&values)?;
                Ok((ids.into_iter().zip(observed).collect(), retained))
            })
            .await?;
        Ok(ModelingObservations {
            values,
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
        seed: BTreeMap<SemanticId, f64>,
        parameters: BTreeMap<SemanticId, f64>,
        compiler: Profile,
        allow_missing_free: bool,
        cancel: &crate::CancelSource,
    ) -> Result<(CaseValues, BTreeMap<SemanticId, StartSource>), WorkflowError> {
        let product = model.compiled();
        let inner_unknowns = product
            .admitted
            .implicit
            .values()
            .flat_map(|i| i.unknowns.iter().copied())
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
        for (id, value) in seed {
            let fixed = case.variables.iter().any(|(path, state)| {
                state.fixed == Some(true) && product.model.paths.get(path) == Some(&id)
            });
            if !fixed
                && product
                    .admitted
                    .case
                    .variables()
                    .iter()
                    .any(|v| v.port.id == id)
            {
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
        for (id, value) in parameters {
            if !value.is_finite()
                || !product
                    .admitted
                    .case
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
                        .case
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
    /// Resolve starts in dependency order; explicit case values override every hint.
    /// No value is fabricated for a missing coordinate, and starts never fix variables.
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
            BTreeMap::new(),
            BTreeMap::new(),
            cancel,
        )
        .await
    }
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
        seed: BTreeMap<SemanticId, f64>,
        parameters: BTreeMap<SemanticId, f64>,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        let resolution = self
            .resolve_case(
                root, instance, bindings, limits, case, order, compiler, solver, numerical, seed,
                parameters, false, cancel,
            )
            .await?;
        self.finish_case(resolution).await
    }
    pub(in crate::workflow) async fn finish_case(
        &self,
        resolution: ModelingCaseResolution,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        let ModelingCaseResolution {
            compiler,
            instance,
            model,
            starts,
            providers,
            numerical,
            solver,
            ..
        } = resolution;
        let solve = self
            .runtime
            .shared
            .math()
            .prepare_solve(
                model.case.clone(),
                model.values.clone(),
                providers.clone(),
                solver.clone(),
                None,
                numerical,
            )
            .await?;
        Ok(ModelingSolvePreparation {
            source: self.clone(),
            compiler,
            instance,
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
        seed: BTreeMap<SemanticId, f64>,
        parameters: BTreeMap<SemanticId, f64>,
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
        let inner = Inner::of(model.compiled());
        let (values, starts) = self
            .resolve_starts(
                &model,
                &case,
                seed,
                parameters,
                compiler,
                allow_missing_free,
                cancel,
            )
            .await?;
        if !allow_missing_free {
            require_inputs(&model, &values)?;
        }
        let providers = self
            .inner_registrations(
                model.clone(),
                &values,
                &case,
                &numerical,
                &solver.numerics,
                &solver.controls,
                compiler,
                cancel,
                None,
            )
            .await?;
        numerical
            .declarations
            .retain(|r| !inner.contains(&r.declaration.target_id));
        numerical.implicit = implicit::factorable_definitions(&model, &case);
        let mut solver = solver;
        solver
            .numerics
            .requirements
            .retain(|r| !inner.contains(&r.target_id));
        let states = self
            .variable_states(
                &model,
                &inner,
                &case,
                &values,
                instance,
                &mut numerical,
                compiler,
                &providers,
                cancel,
            )
            .await?;
        let prepared = self
            .bound_case(&model, values.clone(), &states, order, compiler, cancel)
            .await?;
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
                instance,
                &mut numerical,
                &solver,
                compiler,
                &providers,
                cancel,
            )
            .await?;
        Ok(ModelingCaseResolution {
            compiler,
            instance,
            numerics,
            model: prepared,
            starts,
            providers,
            numerical,
            solver,
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
        instance: InstanceId,
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
                        ModelId::from_id(instance.as_id()),
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
        instance: InstanceId,
        numerical: &mut NumericalInputs,
        solver: &SolverProfile,
        compiler: Profile,
        providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        cancel: &crate::CancelSource,
    ) -> Result<std::sync::Arc<pse_model::numerics::ResolvedNumericalPolicy>, WorkflowError> {
        let product = model.compiled();
        let variables = variables(product);
        let equations = equations(product);
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
        targets.extend(numerical.targets.clone());
        let resolved = pse_math::numerics::resolve(
            &physical,
            &targets,
            &numerical.declarations,
            &solver.numerics,
        )
        .map_err(crate::math::MathRuntimeError::from)?;
        for t in &resolved.targets {
            if t.kind == NumericalTarget::Variable
                && !prepared
                    .case
                    .compiled()
                    .plan
                    .structure()
                    .variables()
                    .iter()
                    .any(|v| v.port.id == t.id && v.fixed)
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
                    ModelId::from_id(instance.as_id()),
                    id,
                    NumericalTarget::Row,
                    source,
                    NumericalSource::DerivedNominal,
                    None,
                    Some(scale),
                ));
            }
        }
        Ok(std::sync::Arc::new(
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
                .implicit
                .values()
                .flat_map(|i| i.unknowns.iter().copied())
                .collect(),
            rows: product
                .admitted
                .implicit
                .values()
                .flat_map(|i| i.residuals.iter().flat_map(|r| r.rows.iter().copied()))
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
        .case
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
    model_id: ModelId,
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
            model_id,
            case_id: None,
            target_id: target,
            target_kind: kind,
            nominal,
            scaling_factor: scale,
            absolute_tolerance: None,
            relative_tolerance: None,
            unit_id: None,
            coordinates: NumericalCoordinates::Physical,
            priority: 0,
            required: true,
            provenance: format!("modeling annotation {source_id}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::solves::Outcome;
    #[tokio::test]
    async fn kernel_starts_numerics_and_constant_solver_share_the_existing_pipeline() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let names = BTreeMap::from([(
            "Scalar".into(),
            physical.quantities.neutral_dimensionless().unwrap(),
        )]);
        let rows=pse_authoring::language::parse(
            "package p { def Root { param p: Scalar = 2; var x: Scalar; var y: Scalar; let bad: Scalar = log(-1); eq balance: x+y == 6; annotation start x(p); annotation start y(2*x); annotation bounds x(0,10); annotation nominal x(2); annotation nominal y(4); annotation scale balance(inverseSum); annotation check x(x > 2.5); annotation report y(\"computed y\"); } }",
            SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default()).unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(rows, physical, names).unwrap();
        let mut bindings = Bindings::default();
        bindings.demand.push("bad".into());
        let case = ModelingCaseBindings {
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
        let report = package
            .solve_case(
                p,
                super::super::super::tests::compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
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
            .modeling_package(rows, fixture::physical(), fixture::discrete_names())
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
}

#[cfg(all(test, feature = "solver-kinsol"))]
mod native_tests {
    use super::*;
    #[tokio::test]
    async fn kernel_native_root_solves_and_qualifies_the_authored_model() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let names = BTreeMap::from([(
            "Scalar".into(),
            physical.quantities.neutral_dimensionless().unwrap(),
        )]);
        let rows=pse_authoring::language::parse("package p { def Root { var x:Scalar; eq e:x*x==4; annotation start x(1); annotation bounds x(0.5,3); annotation nominal x(2); annotation valid x(0.5,3,reject); annotation check x(x>0); annotation report x(\"root\"); expect x == 2 tolerance 1e-6; } }",SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default()).unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = rt.modeling_package(rows, physical, names).unwrap();
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
        assert!(
            (result
                .reports
                .iter()
                .find(|r| r.label == "root")
                .unwrap()
                .value
                - 2.)
                .abs()
                < 1e-6
        );
        assert!(result.checks.iter().any(|c| c.kind
            == pse_model::generated::enums::ModelingCheckKind::Expectation
            && c.satisfied));
    }
}
