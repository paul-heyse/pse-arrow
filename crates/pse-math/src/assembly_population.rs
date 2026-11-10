// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordered instance frontiers across independently admitted physical cases.
use super::*;
use crate::{
    ExecutionAttribution,
    batch::{CheckedBatchExecution, PreparedBatch},
    guarded::Evaluation,
};

#[derive(Clone, Copy, Debug)]
struct Occurrence {
    case: usize,
    instance: usize,
}
#[derive(Debug)]
struct PopulationGroup {
    prepared: [Option<PreparedBatch>; 2],
    occurrences: Vec<Occurrence>,
}

/// An immutable population of pure, branch-free cases. Exact existing numerical
/// program identity qualifies sharing; original case and instance order qualify
/// physical transport and diagnostics independently.
#[derive(Debug)]
pub struct CrossCasePopulation {
    cases: Vec<Arc<CaseAssembly>>,
    groups: Vec<PopulationGroup>,
    ceiling: DerivativeOrder,
}
fn order_index(order: DerivativeOrder) -> usize {
    usize::from(order >= DerivativeOrder::First)
}
fn selected(case: &CaseAssembly, ordinal: usize) -> Option<(usize, &Group)> {
    case.instances
        .iter()
        .enumerate()
        .filter_map(|(instance, local)| {
            local.groups[EvaluationDemand::Constraints]
                .as_ref()
                .map(|group| (instance, group))
        })
        .nth(ordinal)
}
impl CrossCasePopulation {
    /// Whether every selected constraint stage can execute in this pure
    /// population at the exact requested Value or First ceiling.
    pub fn eligible(case: &CaseAssembly, ceiling: DerivativeOrder) -> bool {
        ceiling <= DerivativeOrder::First
            && case.order() >= ceiling
            && case.instances.iter().all(|instance| {
                instance.groups[EvaluationDemand::Constraints]
                    .as_ref()
                    .is_none_or(|group| {
                        [DerivativeOrder::Value, ceiling]
                            .into_iter()
                            .all(|order| case.programs[group.request].pure_branch_free(order))
                    })
            })
    }
    /// Conservative passive constructor extent, before grouping or source-map
    /// attachment. This counts original instances rather than an invented body
    /// complexity estimate; instruction allocations retain their original owners.
    pub fn construction_bytes(cases: &[Arc<CaseAssembly>]) -> Result<usize, MathError> {
        let overflow = || MathError::Limit("cross-case population metadata");
        let instances = cases.iter().try_fold(0usize, |sum, case| {
            sum.checked_add(case.instances.len()).ok_or_else(overflow)
        })?;
        size_of::<Self>()
            .checked_add(2 * size_of::<usize>())
            .and_then(|bytes| {
                bytes.checked_add(cases.len().checked_mul(size_of::<Arc<CaseAssembly>>())?)
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    instances
                        .checked_mul(size_of::<PopulationGroup>() + 2 * size_of::<Occurrence>())?,
                )
            })
            .ok_or_else(overflow)
    }
    /// Prepare compatible current occurrences one instance frontier at a time.
    /// A later local instance never executes after an earlier local refusal.
    pub fn new(
        cases: Vec<Arc<CaseAssembly>>,
        ceiling: DerivativeOrder,
    ) -> Result<Arc<Self>, MathError> {
        if cases.iter().any(|case| !Self::eligible(case, ceiling)) {
            return Err(MathError::Contract(
                "cross-case population requires pure branch-free selected partials".into(),
            ));
        }
        crate::construction::additional(Self::construction_bytes(&cases)?)?;
        let capacity = cases.iter().map(|case| case.instances.len()).sum();
        let mut groups = Vec::with_capacity(capacity);
        let waves = cases
            .iter()
            .map(|case| {
                case.instances
                    .iter()
                    .filter(|local| local.groups[EvaluationDemand::Constraints].is_some())
                    .count()
            })
            .max()
            .unwrap_or(0);
        for wave in 0..waves {
            for (case_index, case) in cases.iter().enumerate() {
                let Some((_, local)) = selected(case, wave) else {
                    continue;
                };
                let body = &case.programs[local.request];
                if cases[..case_index].iter().any(|previous| {
                    selected(previous, wave).is_some_and(|(_, local)| {
                        Arc::ptr_eq(body, &previous.programs[local.request])
                    })
                }) {
                    continue;
                }
                let count = cases
                    .iter()
                    .filter(|candidate| {
                        selected(candidate, wave).is_some_and(|(_, local)| {
                            Arc::ptr_eq(body, &candidate.programs[local.request])
                        })
                    })
                    .count();
                let mut occurrences = Vec::with_capacity(count);
                for (case, candidate) in cases.iter().enumerate() {
                    if let Some((instance, local)) = selected(candidate, wave)
                        && Arc::ptr_eq(body, &candidate.programs[local.request])
                    {
                        occurrences.push(Occurrence { case, instance });
                    }
                }
                groups.push(PopulationGroup {
                    prepared: [
                        Some(PreparedBatch::new(
                            Arc::clone(body),
                            DerivativeOrder::Value,
                        )?),
                        (ceiling >= DerivativeOrder::First)
                            .then(|| PreparedBatch::new(Arc::clone(body), DerivativeOrder::First))
                            .transpose()?,
                    ],
                    occurrences,
                });
            }
        }
        Ok(Arc::new(Self {
            cases,
            groups,
            ceiling,
        }))
    }
    /// Native metadata retained by this projection, excluding the existing cases.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + 2 * size_of::<usize>()
            + self.cases.capacity() * size_of::<Arc<CaseAssembly>>()
            + self.groups.capacity() * size_of::<PopulationGroup>()
            + self
                .groups
                .iter()
                .map(|group| group.occurrences.capacity() * size_of::<Occurrence>())
                .sum::<usize>()
    }
    /// Exact case population in original caller order.
    pub fn cases(&self) -> &[Arc<CaseAssembly>] {
        &self.cases
    }
    /// Complete reusable worker allowance, including both requested orders and
    /// physical results. The caller retains its actual enclosing allocation owner.
    pub fn worker_bytes(&self) -> Result<usize, MathError> {
        let overflow = || MathError::Limit("cross-case population worker");
        let mut bytes = size_of::<CrossCasePopulationWorker>();
        let mut max_inputs = 0usize;
        let mut max_rows = 0usize;
        for group in &self.groups {
            for prepared in group.prepared.iter().flatten() {
                bytes = bytes
                    .checked_add(prepared.reusable_backing_bytes(group.occurrences.len())?)
                    .ok_or_else(overflow)?;
                max_inputs = max_inputs.max(prepared.layout().inputs());
                max_rows = max_rows.max(group.occurrences.len());
            }
        }
        bytes = bytes
            .checked_add(
                max_inputs
                    .checked_mul(max_rows.checked_add(1).ok_or_else(overflow)?)
                    .and_then(|n| n.checked_mul(size_of::<f64>()))
                    .ok_or_else(overflow)?,
            )
            .and_then(|n| n.checked_add(max_rows.checked_mul(size_of::<Occurrence>())?))
            .and_then(|n| {
                n.checked_add(
                    self.groups
                        .len()
                        .checked_mul(size_of::<[Option<CheckedBatchExecution>; 2]>())?,
                )
            })
            .ok_or_else(overflow)?;
        for case in &self.cases {
            // Original input/diagnostic payload bounds also cover one attributed
            // refusal per case; no per-point source maps are prepared eagerly.
            let source = case
                .structure
                .instances()
                .iter()
                .map(|binding| ExecutionAttribution::from_case_instance(binding).retained_bytes())
                .max()
                .unwrap_or(0);
            bytes = bytes
                .checked_add(source)
                .and_then(|n| n.checked_add(case.jacobian.retained_bytes()))
                .and_then(|n| n.checked_add(case.rows.len().checked_mul(size_of::<f64>())?))
                .and_then(|n| {
                    n.checked_add(
                        size_of::<CasePopulationEvaluation>()
                            + size_of::<Option<MathError>>()
                            + size_of::<bool>(),
                    )
                })
                .ok_or_else(overflow)?;
        }
        Ok(bytes)
    }
    /// Bind one current attempt or existing experiment lane to reusable packed
    /// arithmetic, without changing that lane's placement or scientific scope.
    pub fn worker_scoped(
        self: &Arc<Self>,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<CrossCasePopulationWorker, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        crate::construction::additional(self.worker_bytes()?)?;
        let mut locals = Vec::with_capacity(self.groups.len());
        let mut inputs = 0;
        let mut rows = 0;
        for group in &self.groups {
            let mut workers = [None, None];
            for (index, prepared) in group.prepared.iter().enumerate() {
                if let Some(prepared) = prepared {
                    let backing = prepared.reusable_backing_bytes(group.occurrences.len())?;
                    workers[index] = Some(prepared.reusable_worker_scoped(
                        scope.clone(),
                        group.occurrences.len(),
                        backing,
                    )?);
                    inputs = inputs.max(prepared.layout().inputs());
                    rows = rows.max(group.occurrences.len());
                }
            }
            locals.push(workers);
        }
        let results = self
            .cases
            .iter()
            .map(|case| CasePopulationEvaluation {
                values: vec![0.; case.rows.len()],
                jacobian: (self.ceiling >= DerivativeOrder::First)
                    .then(|| case.jacobian.as_ref().clone()),
                first: false,
            })
            .collect();
        Ok(CrossCasePopulationWorker {
            population: Arc::clone(self),
            locals,
            scope,
            active: Vec::with_capacity(rows),
            packed: Vec::with_capacity(rows * inputs),
            gather: vec![0.; inputs],
            results,
            errors: (0..self.cases.len()).map(|_| None).collect(),
            complete: vec![false; self.cases.len()],
        })
    }
}

/// Physical values and the original canonical sparse First layout for one case.
#[derive(Debug)]
pub struct CasePopulationEvaluation {
    values: Vec<f64>,
    jacobian: Option<AssemblyMatrix>,
    first: bool,
}
impl CasePopulationEvaluation {
    /// Original case constraint-row order.
    pub fn values(&self) -> &[f64] {
        &self.values
    }
    /// Canonical original sparse Jacobian, only when First was requested.
    pub fn jacobian(&self) -> Option<&faer::sparse::SparseColMat<usize, f64>> {
        self.first
            .then(|| self.jacobian.as_ref().map(AssemblyMatrix::matrix))
            .flatten()
    }
}
/// One lane's mutable cross-case gathers, arithmetic and sparse result buffers.
#[derive(Debug)]
pub struct CrossCasePopulationWorker {
    population: Arc<CrossCasePopulation>,
    locals: Vec<[Option<CheckedBatchExecution>; 2]>,
    scope: pse_kernels::ExecutionScope,
    active: Vec<Occurrence>,
    packed: Vec<f64>,
    gather: Vec<f64>,
    results: Vec<CasePopulationEvaluation>,
    errors: Vec<Option<MathError>>,
    complete: Vec<bool>,
}
fn scatter_case(
    case: &CaseAssembly,
    instance: usize,
    local: &Evaluation,
    result: &mut CasePopulationEvaluation,
    order: DerivativeOrder,
) -> Result<(), MathError> {
    let binding = &case.instances[instance];
    let group = binding.groups[EvaluationDemand::Constraints]
        .as_ref()
        .ok_or_else(|| MathError::Contract("population constraint group".into()))?;
    for term in &binding.constraint_terms {
        let cell = &mut result.values[term.destination];
        *cell = finite(*cell + term.scale * local.values[term.result_row])?;
    }
    if order >= DerivativeOrder::First {
        let jacobian = result
            .jacobian
            .as_mut()
            .ok_or_else(|| MathError::Contract("population First layout".into()))?;
        let width = case.programs[group.request].coordinates().len();
        for (index, term) in case
            .jacobian_terms
            .iter()
            .enumerate()
            .filter(|(_, term)| term.instance == instance)
        {
            let row = group
                .outputs
                .binary_search(&term.output)
                .map_err(|_| MathError::Contract("population First output".into()))?;
            jacobian.add(
                Addend::new(index),
                finite(term.scale * local.jacobian[row * width + term.i])?,
            )?;
        }
    }
    Ok(())
}
impl CrossCasePopulationWorker {
    /// Execute original ordered instance frontiers. Pure cases may share the
    /// same arithmetic wave; each failed case skips every later local instance.
    pub fn evaluate(
        &mut self,
        values: &[CaseValues],
        order: DerivativeOrder,
    ) -> Result<(), MathError> {
        if values.len() != self.results.len() || order > self.population.ceiling {
            return Err(MathError::Contract("cross-case population demand".into()));
        }
        self.complete.fill(false);
        self.scope.check().map_err(crate::error::scope_error)?;
        for (index, case) in self.population.cases.iter().enumerate() {
            self.errors[index] = case.structure.validate_values(&values[index]).err();
            let result = &mut self.results[index];
            result.values.fill(0.);
            result.first = order >= DerivativeOrder::First;
            if let Some(jacobian) = &mut result.jacobian {
                jacobian.clear();
            }
        }
        for (index, group) in self.population.groups.iter().enumerate() {
            self.active.clear();
            self.packed.clear();
            for &occurrence in &group.occurrences {
                if self.errors[occurrence.case].is_some() {
                    continue;
                }
                self.scope.check().map_err(crate::error::scope_error)?;
                let case = &self.population.cases[occurrence.case];
                let gather = case.bindings[occurrence.instance][EvaluationDemand::Constraints]
                    .as_ref()
                    .ok_or_else(|| MathError::Contract("population gather".into()))?;
                match EvalInputs::from(&values[occurrence.case])
                    .fill(gather, &mut self.gather[..gather.len()])
                {
                    Ok(()) => {
                        self.active.push(occurrence);
                        self.packed.extend_from_slice(&self.gather[..gather.len()]);
                    }
                    Err(error) => {
                        self.errors[occurrence.case] = Some(
                            ExecutionAttribution::from_case_instance(
                                &case.structure.instances()[occurrence.instance],
                            )
                            .evaluation_error(error),
                        )
                    }
                }
            }
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count(
                    "math.case_population_rows",
                    "count",
                    self.active.len() as u64,
                );
                if self.active.len() > 1 {
                    context.count(
                        "math.case_population_shared_rows",
                        "count",
                        self.active.len() as u64,
                    );
                }
            }
            let active = &self.active;
            let cases = &self.population.cases;
            let results = &mut self.results;
            let errors = &mut self.errors;
            self.locals[index][order_index(order)]
                .as_mut()
                .ok_or_else(|| MathError::Contract("population order worker".into()))?
                .evaluate_observing(active.len(), &self.packed, |ordinal, local| {
                    let occurrence = active[ordinal];
                    let case = &cases[occurrence.case];
                    let result = local.and_then(|local| {
                        scatter_case(
                            case,
                            occurrence.instance,
                            local,
                            &mut results[occurrence.case],
                            order,
                        )
                    });
                    if let Err(error) = result {
                        errors[occurrence.case] = Some(
                            ExecutionAttribution::from_case_instance(
                                &case.structure.instances()[occurrence.instance],
                            )
                            .evaluation_error(error),
                        );
                    }
                })?;
        }
        self.scope.check().map_err(crate::error::scope_error)?;
        for (index, complete) in self.complete.iter_mut().enumerate() {
            *complete = self.errors[index].is_none();
        }
        Ok(())
    }
    /// Borrow completed numerical buffers or move the original typed failure.
    /// A consumed failure cannot subsequently expose a partial successful case.
    pub fn result(&mut self, index: usize) -> Result<&CasePopulationEvaluation, MathError> {
        let error = self
            .errors
            .get_mut(index)
            .ok_or_else(|| MathError::Contract("population result index".into()))?;
        if let Some(error) = error.take() {
            return Err(error);
        }
        if !self.complete[index] {
            return Err(MathError::Contract("population result incomplete".into()));
        }
        Ok(&self.results[index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binding::*,
        typed::{Binary, BodyBuilder, BodyLimits},
    };
    use pse_model::generated::enums::ModelingVariableDomain;
    use pse_quantity::{
        IndexSet,
        standard::{StandardInvariantChecker, ids, standard_registry},
    };
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    #[test]
    fn cross_case_population_packs_exact_shared_programs_and_keeps_ordered_instance_failures() {
        let registry = standard_registry().unwrap();
        let quantity = ids::quantity("neutral");
        let port = |n| pse_kernels::Port {
            id: id(n),
            quantity,
            unit: registry.quantity_type(quantity).unwrap().canonical_unit,
        };
        let mut builder = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            2,
            BodyLimits::default(),
        )
        .unwrap();
        let a = builder.input(0, quantity, IndexSet::new(), id(60)).unwrap();
        let b = builder.input(1, quantity, IndexSet::new(), id(61)).unwrap();
        let sum = builder.binary(Binary::Add, a, b, None, id(62)).unwrap();
        let square = builder
            .binary(Binary::Mul, sum.clone(), sum, None, id(63))
            .unwrap();
        let body = Arc::new(builder.prepare(&[square]).unwrap());
        let key = ContentHash::from_bytes([90; 32]);
        let scope =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
        let mut programs: Option<Vec<Arc<CompiledBody>>> = None;
        let mut cases = Vec::new();
        for ordinal in 0_u8..5 {
            let instance = |number, source, scale| InstanceBinding {
                instance: id(number),
                body: key,
                checked_members: Default::default(),
                checked_sources: Default::default(),
                occurrences: Default::default(),
                slots: vec![
                    SlotBinding::new(&port(source), &port(1), &registry).unwrap(),
                    SlotBinding::new(&port(source), &port(2), &registry).unwrap(),
                ]
                .into(),
                contributions: vec![Contribution {
                    output: 0,
                    target: Target::Row(id(10)),
                    scale,
                }],
            };
            let structure = Arc::new(
                CaseStructure::new(
                    vec![1, 2]
                        .into_iter()
                        .map(|number| Variable {
                            port: port(number),
                            fixed: false,
                            domain: ModelingVariableDomain::Continuous,
                            lower: None,
                            upper: None,
                        })
                        .collect(),
                    vec![],
                    vec![
                        instance(20 + 2 * ordinal, 1, 1.),
                        instance(21 + 2 * ordinal, 2, 2.),
                    ],
                    vec![Row {
                        id: id(10),
                        quantity,
                        lower: f64::NEG_INFINITY,
                        upper: f64::INFINITY,
                    }],
                    None,
                    CaseLimits::default(),
                )
                .unwrap(),
            );
            let plan = Arc::new(
                CasePlan::prepare(
                    structure,
                    BTreeMap::from([(key, Arc::clone(&body))]),
                    &registry,
                    DerivativeOrder::First,
                    AssemblyLimits::default(),
                    scope.cancellation(),
                )
                .unwrap(),
            );
            let assembly = if let Some(programs) = &programs {
                plan.assemble(programs.clone()).unwrap()
            } else {
                let assembly = plan
                    .compile(Optimization::default(), EvaluationLimits::default(), &scope)
                    .unwrap();
                programs = Some(assembly.programs.clone());
                assembly
            };
            cases.push(Arc::new(assembly));
        }
        let bound = CrossCasePopulation::construction_bytes(&cases).unwrap();
        let population = CrossCasePopulation::new(cases, DerivativeOrder::First).unwrap();
        assert!(bound >= population.retained_bytes());
        assert_eq!(population.groups.len(), 2);
        assert!(
            population
                .groups
                .iter()
                .all(|group| group.occurrences.len() == 5)
        );
        let mut worker = population.worker_scoped(scope).unwrap();
        let mut values = (0_u32..5)
            .map(|i| CaseValues {
                scalars: BTreeMap::from([(id(1), f64::from(i) + 1.), (id(2), f64::from(i) + 2.)]),
            })
            .collect::<Vec<_>>();
        for order in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Value,
        ] {
            worker.evaluate(&values, order).unwrap();
            for (index, value) in values.iter().enumerate() {
                let x = value.scalars[&id(1)];
                let y = value.scalars[&id(2)];
                let result = worker.result(index).unwrap();
                assert_eq!(result.values(), [4. * x * x + 8. * y * y]);
                if order == DerivativeOrder::First {
                    assert_eq!(result.jacobian().unwrap().val(), [8. * x, 16. * y]);
                } else {
                    assert!(result.jacobian().is_none());
                }
            }
        }
        values[1].scalars.insert(id(1), 1e200);
        values[1].scalars.insert(id(2), 1e200);
        values[3].scalars.insert(id(2), 1e200);
        worker.evaluate(&values, DerivativeOrder::First).unwrap();
        assert!(
            matches!(worker.result(1),Err(MathError::Instance{instance,..}) if instance==id(22))
        );
        assert!(
            worker.result(1).is_err(),
            "consumed failure must not expose a partial row"
        );
        assert!(
            matches!(worker.result(3),Err(MathError::Instance{instance,..}) if instance==id(27))
        );
        assert!(worker.result(0).is_ok());
        assert!(worker.result(2).is_ok());
        assert!(worker.result(4).is_ok());
    }
}
