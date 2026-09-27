// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One solver projection over the same shared mathematics and semantic variable identities.
use super::*;
use pse_math::binding::{CaseLimits, CaseStructure, Target, Variable};
/// Per-attempt variable specification in canonical physical units.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelingVariableState {
    pub fixed: Option<bool>,
    /// Outer option selects an override; inner None removes that endpoint.
    pub lower: Option<Option<f64>>,
    pub upper: Option<Option<f64>>,
}
/// Case values and structural specifications resolve through the modeling language's paths.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelingCaseBindings {
    pub values: BTreeMap<String, f64>,
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
    /// free starts are required separately by numerical solve admission.
    pub fn prepare_modeling_case_cancellable(
        &mut self,
        root: SemanticId,
        instance: SemanticId,
        mut bindings: Bindings,
        limits: Limits,
        case: &ModelingCaseBindings,
        order: DerivativeOrder,
        profile: Profile,
        cancel: Arc<AtomicBool>,
    ) -> Result<(PreparedModeling, PreparedCase, CaseValues)> {
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
        let prepared =
            self.prepare_modeling_bound_case(&model, &values, &states, order, profile, &cancel)?;
        Ok((model, prepared, values))
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
            self.admitted
                .case
                .variables()
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
            &cancel,
        )?);
        let presolve = Arc::new(plan.presolve_facts(&values, 100_000, &cancel)?);
        let facts = pse_math::facts::ProblemFacts::from_plan(&plan, None, &presolve)?;
        let coefficients = if facts.affine_rows.iter().all(|v| *v)
            && facts.objective_degree.is_some_and(|d| d <= 2)
            && presolve
                .obligations
                .values()
                .all(|s| *s == pse_math::presolve::ObligationStatus::Discharged)
        {
            Some(Arc::new(plan.coefficients_with_facts(
                &values, &presolve, 100_000, &cancel,
            )?))
        } else {
            None
        };
        let facts =
            pse_math::facts::ProblemFacts::from_plan(&plan, coefficients.as_deref(), &presolve)?;
        let assumptions = plan
            .structure()
            .parameters()
            .iter()
            .map(|p| p.id)
            .chain(
                plan.structure()
                    .variables()
                    .iter()
                    .filter(|v| v.fixed)
                    .map(|v| v.port.id),
            )
            .map(|id| (id, values.scalars[&id].to_bits()))
            .collect();
        let prepared = PreparedCase {
            quantities: quantities.clone(),
            presolve,
            coefficient_values: assumptions,
            facts,
            structure: structural_plan(SemanticId::NIL, &plan, &cancel)?,
            artifacts: artifact_requests(&plan, profile, environment),
            occurrences: self
                .admitted
                .bodies
                .values()
                .flat_map(|b| b.occurrences.iter().cloned())
                .fold(BTreeMap::new(), |mut map, o| {
                    map.entry(o.definition).or_insert_with(Vec::new).push(o);
                    map
                }),
            coefficients,
            plan,
        };
        Ok(prepared)
    }
}
/// Finite-bound admission of free discrete variables after case binding (ADR-0103 item 4).
/// A binary decision narrows to the unit box; integer and semi domains need finite bounds,
/// and a semi domain's active interval is positive. Fixed variables are checked for
/// membership by the case structure instead.
fn admit_domains(model: &PreparedModeling, variables: &mut [Variable]) -> Result<()> {
    use pse_model::generated::enums::ModelingVariableDomain as Domain;
    use pse_modeling::{DomainAnalysis, DomainRefusal};
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
            (
                v.lower.unwrap_or(0.0).max(0.0),
                v.upper.unwrap_or(1.0).min(1.0),
            )
        } else {
            match (v.lower, v.upper) {
                (Some(l), Some(u)) if l.is_finite() && u.is_finite() => (l, u),
                _ => return Err(refuse(DomainRefusal::InfiniteBound)),
            }
        };
        if lower > upper
            || v.domain.is_integer() && lower.ceil() > upper.floor()
            || v.domain.is_semi() && lower <= 0.0
        {
            return Err(refuse(DomainRefusal::EmptyDomain));
        }
        v.lower = Some(lower);
        v.upper = Some(upper);
    }
    Ok(())
}
impl CompilerWorkspace {
    /// Apply case specifications to an already resolved, immutable kernel revision.
    pub fn prepare_modeling_bound_case(
        &self,
        model: &PreparedModeling,
        values: &CaseValues,
        states: &BTreeMap<SemanticId, ModelingVariableState>,
        order: DerivativeOrder,
        profile: Profile,
        cancel: &Arc<AtomicBool>,
    ) -> Result<PreparedCase> {
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
        admit_domains(model, &mut variables)?;
        let equations = model
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
        let structure = Arc::new(CaseStructure::new(
            variables,
            model.admitted.case.parameters().to_vec(),
            instances,
            rows,
            model.admitted.case.objective().cloned(),
            CaseLimits::default(),
        )?);
        let prepared = model.prepare_view(
            structure,
            &values,
            self.inputs.quantities.clone(),
            order,
            profile,
            self.inventory.environment(&self.db),
            &cancel,
        )?;
        Ok(prepared)
    }
    /// Compile selected observations under the same library environment and quantity authority.
    pub fn prepare_modeling_observations(
        &self,
        model: &PreparedModeling,
        rows: &BTreeSet<SemanticId>,
        values: &CaseValues,
        profile: Profile,
        cancel: &Arc<AtomicBool>,
    ) -> Result<PreparedCase> {
        let structure = model.observation_structure(rows)?;
        let ids = structure
            .parameters()
            .iter()
            .map(|p| p.id)
            .chain(structure.variables().iter().map(|v| v.port.id))
            .collect::<BTreeSet<_>>();
        let values = CaseValues {
            scalars: values
                .scalars
                .iter()
                .filter(|(id, _)| ids.contains(id))
                .map(|(id, v)| (*id, *v))
                .collect(),
        };
        model.prepare_view(
            structure,
            &values,
            self.inputs.quantities.clone(),
            DerivativeOrder::Value,
            profile,
            self.inventory.environment(&self.db),
            cancel,
        )
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
