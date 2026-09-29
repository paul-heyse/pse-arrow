// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One solver projection over the same shared mathematics and semantic variable identities.
use super::*;
use pse_math::binding::{CaseLimits, CaseStructure, Target, Variable};
/// Per-attempt variable specification in canonical physical units.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelingVariableState {
    /// Override of whether the variable is fixed; `None` keeps the authored state.
    pub fixed: Option<bool>,
    /// Outer option selects an override; inner None removes that endpoint.
    pub lower: Option<Option<f64>>,
    /// Upper endpoint override with the same meaning as `lower`.
    pub upper: Option<Option<f64>>,
}
/// Case values and structural specifications resolve through the modeling language's paths.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelingCaseBindings {
    /// Values by modeling-language path, in canonical physical units.
    pub values: BTreeMap<String, f64>,
    /// Structural variable specifications by modeling-language path.
    pub variables: BTreeMap<String, ModelingVariableState>,
}
impl From<&pse_modeling::specialize::Fixture> for ModelingCaseBindings {
    fn from(fixture: &pse_modeling::specialize::Fixture) -> Self {
        Self {
            values: fixture
                .specifications
                .iter()
                .filter_map(|(path, value)| value.value.map(|v| (path.clone(), v)))
                .collect(),
            variables: fixture
                .specifications
                .iter()
                .filter(|(_, value)| {
                    value.fixed.is_some() || value.lower.is_some() || value.upper.is_some()
                })
                .map(|(path, value)| {
                    (
                        path.clone(),
                        ModelingVariableState {
                            fixed: value.fixed,
                            lower: value.lower.map(Some),
                            upper: value.upper.map(Some),
                        },
                    )
                })
                .collect(),
        }
    }
}
impl CompilerWorkspace {
    /// Prepare an immutable solver view, excluding observation rows and retaining all original model products.
    /// Physical inputs are canonical values. Parameters and fixed variables are required;
    /// free starts are required separately by numerical solve admission. The bound
    /// tightenings admission applied are returned with the view ([`BoundStructure`]).
    #[expect(
        clippy::too_many_arguments,
        reason = "the specialization request (root, instance, bindings, limits) travels with the case, profiles and cancellation as independent inputs"
    )]
    pub fn prepare_modeling_case_cancellable(
        &mut self,
        root: DeclarationId,
        instance: InstanceId,
        mut bindings: Bindings,
        limits: Limits,
        case: &ModelingCaseBindings,
        order: DerivativeOrder,
        profile: Profile,
        cancel: Arc<AtomicBool>,
    ) -> Result<(
        PreparedModeling,
        PreparedCase,
        CaseValues,
        Vec<pse_modeling::DomainTightening>,
    )> {
        bindings
            .demand
            .extend(case.values.keys().chain(case.variables.keys()).cloned());
        bindings.demand.sort();
        bindings.demand.dedup();
        let model =
            self.prepare_modeling_cancellable(root, instance, bindings, limits, cancel.clone())?;
        let values = model.case_values(case)?;
        let states = case
            .variables
            .iter()
            .map(|(path, state)| {
                model
                    .model
                    .paths
                    .get(path)
                    .copied()
                    .map(|id| (id, state.clone()))
                    .ok_or_else(|| CompileError::Missing(format!("variable path {path}")))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        let bound = model.bound_structure(&states)?;
        let prepared =
            self.prepare_modeling_view(&model, bound.structure, &values, order, profile, &cancel)?;
        let values = prepared.complete(&values);
        Ok((model, prepared, values, bound.tightenings))
    }
}

impl PreparedModeling {
    /// Resolve authored defaults and canonical fixture overrides without a runtime.
    pub fn case_values(&self, case: &ModelingCaseBindings) -> Result<CaseValues> {
        let mut values = CaseValues {
            scalars: BTreeMap::new(),
        };
        for id in &self.admitted.inputs {
            if let Some(pse_modeling::specialize::Value::Number { bits, .. }) =
                self.model.symbols[id].initial
            {
                values.scalars.insert(*id, f64::from_bits(bits));
            }
        }
        for (path, value) in &case.values {
            let id = self.model.paths.get(path).ok_or_else(|| {
                CompileError::Missing(format!("case path {path} is not a scalar symbol"))
            })?;
            if !self.admitted.inputs.contains(id) {
                return Err(CompileError::Missing(format!(
                    "case path {path} is computed, not an independent coordinate"
                )));
            }
            values.scalars.insert(*id, *value);
        }
        Ok(values)
    }
    /// A minimal value-only observation view. Unselected expressions and absent inputs
    /// are not evaluated and cannot poison a start or check.
    pub fn observation_structure(&self, rows: &BTreeSet<SemanticId>) -> Result<Arc<CaseStructure>> {
        self.observation_structure_over(rows, self.admitted.case.variables())
    }
    /// An observation view over explicitly bound variables (fixed flags and box).
    pub(super) fn observation_structure_over(
        &self,
        rows: &BTreeSet<SemanticId>,
        variables: &[Variable],
    ) -> Result<Arc<CaseStructure>> {
        if rows
            .iter()
            .any(|id| !self.admitted.case.rows().iter().any(|r| r.id == *id))
        {
            return Err(CompileError::Missing("unknown modeling observation".into()));
        }
        let instances = self
            .admitted
            .case
            .instances()
            .iter()
            .filter_map(|i| {
                let mut i = i.clone();
                i.contributions
                    .retain(|c| matches!(c.target,Target::Row(id) if rows.contains(&id)));
                (!i.contributions.is_empty()).then_some(i)
            })
            .collect::<Vec<_>>();
        let needed = instances
            .iter()
            .flat_map(|i| i.slots.iter().map(|s| s.source()))
            .collect::<BTreeSet<_>>();
        Ok(Arc::new(CaseStructure::new(
            variables
                .iter()
                .filter(|v| needed.contains(&v.port.id))
                .cloned()
                .collect(),
            self.admitted
                .case
                .parameters()
                .iter()
                .filter(|v| needed.contains(&v.id))
                .cloned()
                .collect(),
            instances,
            self.admitted
                .case
                .rows()
                .iter()
                .filter(|r| rows.contains(&r.id))
                .cloned()
                .collect(),
            None,
            CaseLimits::default(),
        )?))
    }
    /// Complete identity of a prepared view of `structure` (A6, DP-09): the bound case
    /// structure (fixed/free state, bounds, instances, rows, objective and native forms),
    /// every admitted body the plan may bind with its source occurrences, the rules of the
    /// derived realization parameters (ADR-0104), the derivative order, the evaluator
    /// profile and the physical context. Equal keys give equal plans, structural analyses,
    /// derivations, artifact requests and provenance, so a view prepared once serves every
    /// value rebind of the same structure. Values are not part of it: they reach only the
    /// derived parameters and the value-dependent products ([`PreparedCase::rebind`]).
    pub fn view_key(
        &self,
        structure: &CaseStructure,
        order: DerivativeOrder,
        profile: Profile,
        context: &ContentHash,
    ) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::CompilerModelingViewV2);
        h.hash(&structure.key())
            .hash(context)
            .u64(order as u64)
            .u64(self.admitted.bodies.len() as u64);
        for (key, body) in &self.admitted.bodies {
            h.hash(key).u64(body.occurrences.len() as u64);
            for o in &body.occurrences {
                h.id(&o.id)
                    .id(&o.definition)
                    .u64(u64::from(o.span.start))
                    .u64(u64::from(o.span.end));
            }
        }
        h.u64(self.model.derived.len() as u64);
        for (id, parameter) in &self.model.derived {
            h.id(id).id(&parameter.source.as_id());
            match parameter.rule {
                pse_modeling::specialize::DerivedRule::Bound { variable, upper } => {
                    h.str("bound").id(&variable).bool(upper);
                }
                pse_modeling::specialize::DerivedRule::Extremum {
                    expression,
                    upper,
                    margin,
                } => {
                    h.str("extremum")
                        .id(&expression)
                        .bool(upper)
                        .u64(pse_ids::canonical_f64_bits(margin));
                }
            }
        }
        for x in [
            profile.optimization.cores,
            profile.optimization.horner_iterations,
            profile.optimization.cpe_iterations,
            profile.evaluation.derivative_components,
            profile.evaluation.operations,
            profile.evaluation.scratch_bytes,
            profile.evaluation.provider_calls,
        ] {
            h.u64(x as u64);
        }
        h.finish_hash()
    }
    /// Source occurrences of the admitted bodies, by definition. They are provenance, not
    /// structure, so a reused view takes them from the current model.
    pub fn occurrences(&self) -> BTreeMap<SemanticId, Vec<Occurrence>> {
        self.admitted
            .bodies
            .values()
            .flat_map(|b| b.occurrences.iter().cloned())
            .fold(BTreeMap::new(), |mut map, o| {
                map.entry(o.definition).or_insert_with(Vec::new).push(o);
                map
            })
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "one view binds a structure to its values, registry, order, profile, environment and cancellation"
    )]
    fn prepare_view(
        &self,
        structure: Arc<CaseStructure>,
        values: &CaseValues,
        quantities: Arc<QuantityRegistry>,
        order: DerivativeOrder,
        profile: Profile,
        environment: &ContentHash,
        cancel: &Arc<AtomicBool>,
    ) -> Result<PreparedCase> {
        let derivation = Arc::new(Derivation::new(self, &structure, &quantities, cancel)?);
        let derived = derivation.derive(values, cancel)?;
        let values = derived.complete(values);
        structure.validate_frozen_values(&values)?;
        let plan = Arc::new(CasePlan::prepare(
            structure,
            self.admitted
                .bodies
                .iter()
                .map(|(k, b)| (*k, b.math.clone()))
                .collect(),
            &quantities,
            order,
            AssemblyLimits::default(),
            cancel,
        )?);
        let bound = ValueProducts::bind(&plan, &values, cancel)?;
        Ok(PreparedCase {
            quantities,
            presolve: bound.presolve,
            coefficient_values: bound.assumptions,
            facts: bound.facts,
            structure: structural_plan(SemanticId::NIL, &plan, cancel)?,
            artifacts: artifact_requests(&plan, profile, environment),
            occurrences: self.occurrences(),
            coefficients: bound.coefficients,
            plan,
            derivation,
            derived,
        })
    }
}
impl PreparedCase {
    /// Whether every value the value-dependent products consumed is unchanged in `values`
    /// (DP-09): the values the derived parameters consumed (ADR-0104), the dependencies the
    /// presolve projection recorded and, with a coefficient snapshot, the fixed and
    /// parameter values it assumed. Free-variable starts are never among them.
    pub fn values_match(&self, values: &CaseValues) -> bool {
        self.derived.matches(values) && self.products_match(&self.derived.complete(values))
    }
    /// `values` completed with the derived parameters of this binding (ADR-0104): the
    /// values every consumer of this view evaluates with.
    pub fn complete(&self, values: &CaseValues) -> CaseValues {
        self.derived.complete(values).into_owned()
    }
    /// Whether the presolve projection and coefficient snapshot hold for completed values.
    fn products_match(&self, values: &CaseValues) -> bool {
        self.presolve.matches(&self.plan, values)
            && (self.coefficients.is_none()
                || self
                    .coefficient_values
                    .iter()
                    .all(|(id, bits)| values.scalars.get(id).map(|v| v.to_bits()) == Some(*bits)))
    }
    /// Value-only rebind (A6). The plan, structural analysis, artifact requests, source
    /// occurrences and derivation rules depend on structure only and are shared. Derived
    /// parameters are recomputed only when a value they consumed changed; the presolve
    /// projection, coefficient snapshot and problem facts are rebuilt only when a value
    /// they consumed, derived or not, changed ([`Self::values_match`]). The recorded
    /// assumptions always follow the completed values.
    ///
    /// # Errors
    /// Values that do not bind this structure, a refused derived parameter, a failed
    /// projection, or cancellation.
    pub fn rebind(&self, values: &CaseValues, cancel: &Arc<AtomicBool>) -> Result<Self> {
        let mut rebound = self.clone();
        if !self.derived.matches(values) {
            rebound.derived = self.derivation.derive(values, cancel)?;
        }
        let values = rebound.derived.complete(values);
        self.plan.structure().validate_frozen_values(&values)?;
        rebound.coefficient_values = fixed_values(&self.plan, &values)?;
        if !self.products_match(&values) {
            let bound = ValueProducts::bind(&self.plan, &values, cancel)?;
            rebound.presolve = bound.presolve;
            rebound.coefficients = bound.coefficients;
            rebound.facts = bound.facts;
        }
        Ok(rebound)
    }
}
/// A case structure admitted for preparation, with the transformations admission applied
/// to the specified case. The transformations depend on the specifications, not only on
/// the structure: two cases that tighten to the same bounds share one view
/// ([`PreparedModeling::view_key`]) but keep their own records.
#[derive(Clone, Debug)]
pub struct BoundStructure {
    /// The solver structure: variable states applied and discrete domains admitted.
    pub structure: Arc<CaseStructure>,
    /// Inward bound tightenings of free integer-valued variables, in variable order
    /// (ADR-0103 item 4).
    pub tightenings: Vec<pse_modeling::DomainTightening>,
}
/// Finite-bound admission of free discrete variables after case binding (ADR-0103 item 4).
/// A binary decision lives in the unit box, and a case bound outside it is refused;
/// integer and semi domains need finite bounds, and a semi domain's active interval is
/// positive. Integer-valued bounds are tightened inward exactly, to the ceiling of the lower
/// and the floor of the upper, and each tightening is returned as a record. Fixed variables
/// are checked for membership by the case structure instead.
fn admit_domains(
    model: &PreparedModeling,
    variables: &mut [Variable],
) -> Result<Vec<pse_modeling::DomainTightening>> {
    use pse_model::generated::enums::ModelingVariableDomain as Domain;
    use pse_modeling::{DomainAnalysis, DomainRefusal};
    let mut tightenings = Vec::new();
    for v in variables
        .iter_mut()
        .filter(|v| !v.fixed && v.domain.is_discrete())
    {
        let refuse = |reason| {
            CompileError::from(model.model.domain_refusal(
                v.port.id,
                DomainAnalysis::Preparation,
                reason,
            ))
        };
        let (lower, upper) = if v.domain == Domain::Binary {
            let (l, u) = (v.lower.unwrap_or(0.0), v.upper.unwrap_or(1.0));
            if !(0.0..=1.0).contains(&l) || !(0.0..=1.0).contains(&u) {
                return Err(refuse(DomainRefusal::ConflictingBound));
            }
            (l, u)
        } else {
            match (v.lower, v.upper) {
                (Some(l), Some(u)) if l.is_finite() && u.is_finite() => (l, u),
                _ => return Err(refuse(DomainRefusal::InfiniteBound)),
            }
        };
        if v.domain.is_semi() && lower <= 0.0 {
            return Err(refuse(DomainRefusal::EmptyDomain));
        }
        // Exact in floating point; adding zero turns a ceiling of -0.5 into +0.
        let tightened = if v.domain.is_integer() {
            (lower.ceil() + 0.0, upper.floor() + 0.0)
        } else {
            (lower, upper)
        };
        if tightened.0 > tightened.1 {
            return Err(refuse(DomainRefusal::EmptyDomain));
        }
        if tightened != (lower, upper) {
            tightenings.push(model.model.domain_tightening(
                v.port.id,
                [lower, upper],
                [tightened.0, tightened.1],
            ));
        }
        v.lower = Some(tightened.0);
        v.upper = Some(tightened.1);
    }
    Ok(tightenings)
}
impl PreparedModeling {
    /// The solver structure of this model under case specifications: variable states
    /// applied, discrete domains admitted and observation rows excluded, with the bound
    /// tightenings admission recorded. It depends on no value and is cheap;
    /// [`Self::view_key`] identifies the view prepared from the structure (A6).
    ///
    /// # Errors
    /// Integrated derivatives, unknown specification targets or refused discrete domains.
    pub fn bound_structure(
        &self,
        states: &BTreeMap<SemanticId, ModelingVariableState>,
    ) -> Result<BoundStructure> {
        let model = self;
        if !model.model.integrated.is_empty() {
            return Err(CompileError::Missing(
                "integrated time derivatives require the dynamic consumer".into(),
            ));
        }
        let mut variables = model.admitted.case.variables().to_vec();
        for (id, state) in states {
            let variable = variables
                .iter_mut()
                .find(|v| v.port.id == *id)
                .ok_or_else(|| {
                    CompileError::Missing(format!(
                        "case specification {id} requires an outer variable"
                    ))
                })?;
            if let Some(fixed) = state.fixed {
                variable.fixed = fixed;
            }
            if let Some(lower) = state.lower {
                variable.lower = lower;
            }
            if let Some(upper) = state.upper {
                variable.upper = upper;
            }
        }
        let tightenings = admit_domains(model, &mut variables)?;
        let equations = model
            .admitted
            .outputs
            .iter()
            .filter_map(|o| o.constraint().map(|(id, _)| id))
            .collect::<BTreeSet<_>>();
        let rows = model
            .admitted
            .case
            .rows()
            .iter()
            .filter(|r| equations.contains(&r.id))
            .cloned()
            .collect::<Vec<_>>();
        let instances = model
            .admitted
            .case
            .instances()
            .iter()
            .filter_map(|i| {
                let mut i = i.clone();
                i.contributions.retain(|c| match c.target {
                    Target::Row(id) => equations.contains(&id),
                    Target::Objective => true,
                });
                (!i.contributions.is_empty()).then_some(i)
            })
            .collect::<Vec<_>>();
        Ok(BoundStructure {
            structure: Arc::new(
                CaseStructure::new(
                    variables,
                    model.admitted.case.parameters().to_vec(),
                    instances,
                    rows,
                    model.admitted.case.objective().cloned(),
                    CaseLimits::default(),
                )?
                .with_native(model.admitted.case.native().to_vec())?
                .with_requirements(model.admitted.case.requirements().iter().copied()),
            ),
            tightenings,
        })
    }
}
impl CompilerWorkspace {
    /// Prepare the solver view of a bound structure ([`PreparedModeling::bound_structure`])
    /// and bind its first values. Later values rebind it ([`PreparedCase::rebind`]).
    pub fn prepare_modeling_view(
        &self,
        model: &PreparedModeling,
        structure: Arc<CaseStructure>,
        values: &CaseValues,
        order: DerivativeOrder,
        profile: Profile,
        cancel: &Arc<AtomicBool>,
    ) -> Result<PreparedCase> {
        model.prepare_view(
            structure,
            values,
            self.inputs.quantities.clone(),
            order,
            profile,
            self.inventory.environment(&self.db),
            cancel,
        )
    }
    /// Compile selected observations under the same library environment and quantity
    /// authority. The program depends on no value: every evaluation binds its own (A6), so
    /// no presolve or coefficient projection is built for it.
    pub fn prepare_modeling_observations(
        &self,
        model: &PreparedModeling,
        rows: &BTreeSet<SemanticId>,
        profile: Profile,
        cancel: &Arc<AtomicBool>,
    ) -> Result<PreparedFunctions> {
        let plan = Arc::new(CasePlan::prepare(
            model.observation_structure(rows)?,
            model
                .admitted
                .bodies
                .iter()
                .map(|(k, b)| (*k, b.math.clone()))
                .collect(),
            &self.inputs.quantities,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            cancel,
        )?);
        Ok(PreparedFunctions {
            artifacts: artifact_requests(&plan, profile, self.inventory.environment(&self.db)),
            plan,
        })
    }
}

impl CompilerWorkspace {
    /// Select dynamic/test functions and explicit derivative coordinates from the
    /// same admitted model. Coordinates retain caller order through CasePlan.
    pub fn prepare_modeling_functions(
        &self,
        model: &PreparedModeling,
        rows: Vec<SemanticId>,
        coordinates: Vec<SemanticId>,
        order: DerivativeOrder,
        profile: Profile,
        cancel: &Arc<AtomicBool>,
    ) -> Result<PreparedFunctions> {
        let source = model.admitted.plan(
            &self.inputs.quantities,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            cancel,
        )?;
        let plan = Arc::new(source.functions(
            &rows,
            coordinates,
            &self.inputs.quantities,
            order,
            cancel,
        )?);
        Ok(PreparedFunctions {
            artifacts: artifact_requests(&plan, profile, self.inventory.environment(&self.db)),
            plan,
        })
    }
}

impl PreparedModeling {
    /// Identity of the parametric projection of the view keyed `view` over `parameters`, in
    /// request order (A6, DP-09): the view's complete identity and the ordered parameter
    /// coordinates. Equal keys give equal parametric plans and artifact requests.
    pub fn parametric_key(view: &ContentHash, parameters: &[SemanticId]) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::CompilerModelingParametricV1);
        h.hash(view).u64(parameters.len() as u64);
        for parameter in parameters {
            h.id(parameter);
        }
        h.finish_hash()
    }
}

impl CompilerWorkspace {
    /// The parametric projection of a prepared solver view (Plan 22 S1): the view's plan
    /// with its objective and rows, its free columns followed by `parameters` as derivative
    /// coordinates at second order ([`CasePlan::parametric`]), and the artifact requests
    /// that compile it. It depends on no value, like the view it projects, so one program
    /// serves every value rebind of that view; the runtime caches it under
    /// [`PreparedModeling::parametric_key`].
    ///
    /// # Errors
    /// An empty, repeated or undeclared parameter, or cancellation.
    pub fn prepare_modeling_parametric(
        &self,
        view: &PreparedCase,
        parameters: &[SemanticId],
        profile: Profile,
        cancel: &Arc<AtomicBool>,
    ) -> Result<PreparedFunctions> {
        let plan = Arc::new(
            view.plan
                .parametric(parameters, &self.inputs.quantities, cancel)?,
        );
        Ok(PreparedFunctions {
            artifacts: artifact_requests(&plan, profile, self.inventory.environment(&self.db)),
            plan,
        })
    }
}

impl CompilerWorkspace {
    /// Structural index-one eligibility uses the existing library matching owner;
    /// a successful matching is not a numerical nonsingularity claim.
    pub fn analyze_modeling_partition(
        &self,
        model: &PreparedModeling,
        rows: Vec<SemanticId>,
        columns: Vec<SemanticId>,
        profile: Profile,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Arc<StructuralAnalysis>> {
        let p = self.prepare_modeling_functions(
            model,
            rows.clone(),
            columns.clone(),
            DerivativeOrder::First,
            profile,
            cancel,
        )?;
        analyze_partition(SemanticId::NIL, &p.plan, &rows, &columns, cancel)
    }
}
