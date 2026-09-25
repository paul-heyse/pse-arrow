// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical local gathers, explicit row contributions and library-owned sparse products.
use crate::{
    MathError,
    binding::{CaseStructure, CaseValues, Target},
    guarded::{CompiledBody, Evaluation, PreparedBody, Worker},
    jets::EvaluationLimits,
    library::Optimization,
    sparse::AssemblyMatrix,
};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::{DerivativeOrder, Provider, ProviderKey};
use pse_quantity::QuantityRegistry;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Per-case bounds, distinct from a local derivative artifact's resource budget.
#[derive(Clone, Copy, Debug)]
pub struct AssemblyLimits {
    /// Maximum original derivative contributions before duplicate accumulation.
    pub contributions: usize,
    /// Largest dimension/index/entry count representable by the selected native ABI.
    pub native_index: usize,
    /// Aggregate numeric scratch budget for compiled case demands.
    pub worker_bytes: usize,
}
impl Default for AssemblyLimits {
    fn default() -> Self {
        Self {
            contributions: 1_000_000,
            native_index: i32::MAX as usize,
            worker_bytes: 256 * 1024 * 1024,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Demand {
    Objective,
    Constraints,
    All,
}
#[derive(Clone, Debug)]
struct Group {
    outputs: Vec<usize>,
    request: usize,
}
#[derive(Clone, Debug)]
struct Instance {
    groups: Vec<Option<Group>>,
    coordinates: Vec<usize>,
    columns: Vec<usize>,
}
#[derive(Clone, Debug)]
struct Term {
    instance: usize,
    output: usize,
    i: usize,
    j: usize,
    scale: f64,
    target: Target,
}
/// Immutable selected model: full semantic inventory survives sparse projection.
#[derive(Clone, Debug)]
pub struct CasePlan {
    structure: Arc<CaseStructure>,
    bodies: BTreeMap<ContentHash, Arc<PreparedBody>>,
    columns: Vec<SemanticId>,
    rows: BTreeMap<SemanticId, usize>,
    instances: Vec<Instance>,
    jacobian: AssemblyMatrix,
    hessian: AssemblyMatrix,
    jacobian_terms: Vec<Term>,
    hessian_terms: Vec<Term>,
    order: DerivativeOrder,
    requests: Vec<LocalDemand>,
    worker_bytes: usize,
    owner: Option<Arc<dyn crate::AllocationOwner>>,
}
impl CasePlan {
    /// Retain runtime accounting on every escaping structural plan clone.
    pub fn with_owner(mut self, owner: Arc<dyn crate::AllocationOwner>) -> Self {
        for body in self.bodies.values_mut() {
            *body = Arc::new(body.as_ref().clone().with_owner(owner.clone()));
        }
        self.owner = Some(owner);
        self
    }
    /// Admit complete structure and sparse ordering without constructing evaluators.
    pub fn prepare(
        structure: Arc<CaseStructure>,
        bodies: BTreeMap<ContentHash, Arc<PreparedBody>>,
        registry: &QuantityRegistry,
        order: DerivativeOrder,
        limits: AssemblyLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        let coordinates = structure.free_variables().collect();
        Self::prepare_with_coordinates(
            structure,
            bodies,
            registry,
            order,
            limits,
            cancel,
            coordinates,
        )
    }
    /// Prepare explicit derivative coordinates without changing fixed/free or parameter roles.
    /// The ordered selection may contain state variables, fixed variables and parameters.
    pub fn prepare_with_coordinates(
        structure: Arc<CaseStructure>,
        bodies: BTreeMap<ContentHash, Arc<PreparedBody>>,
        registry: &QuantityRegistry,
        order: DerivativeOrder,
        limits: AssemblyLimits,
        cancel: &Arc<AtomicBool>,
        columns: Vec<SemanticId>,
    ) -> Result<Self, MathError> {
        if limits.contributions == 0 || limits.native_index == 0 || limits.worker_bytes == 0 {
            return Err(MathError::Limit("zero case assembly budget"));
        }
        let known: BTreeSet<_> = structure
            .variables()
            .iter()
            .map(|v| v.port.id)
            .chain(structure.parameters().iter().map(|p| p.id))
            .collect();
        if columns.iter().collect::<BTreeSet<_>>().len() != columns.len()
            || columns.iter().any(|c| !known.contains(c))
        {
            return Err(MathError::Contract(
                "unknown or duplicate derivative coordinate".into(),
            ));
        }
        let column_map: BTreeMap<_, _> =
            columns.iter().enumerate().map(|(i, &id)| (id, i)).collect();
        let rows: BTreeMap<_, _> = structure
            .rows()
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id, i))
            .collect();
        if columns.len() > limits.native_index || rows.len() > limits.native_index {
            return Err(MathError::Limit("native index width"));
        }
        let mut requests = Vec::<LocalDemand>::new();
        let mut request_indices = BTreeMap::new();
        let mut instances = vec![];
        let mut jp = vec![];
        let mut hp = vec![];
        let mut jt = vec![];
        let mut ht = vec![];
        let mut target_counts = BTreeMap::new();
        for binding in structure.instances() {
            for c in &binding.contributions {
                *target_counts
                    .entry(match c.target {
                        Target::Objective => None,
                        Target::Row(r) => Some(r),
                    })
                    .or_insert(0usize) += 1;
            }
        }
        for (index, binding) in structure.instances().iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            let body = bodies
                .get(&binding.body)
                .ok_or_else(|| MathError::Contract("missing prepared body".into()))?;
            if body.input_count() != binding.slots.len() {
                return Err(MathError::Contract("body binding arity".into()));
            }
            for (q, slot) in body.input_quantities().iter().zip(&binding.slots) {
                if let Some(q) = q {
                    pse_quantity::admission::require_same_contract(*q, slot.quantity(), registry)?;
                }
            }
            for c in &binding.contributions {
                let q = *body
                    .output_quantities()
                    .get(c.output)
                    .ok_or_else(|| MathError::Contract("untyped or missing body output".into()))?;
                let target = match c.target {
                    Target::Row(id) => structure.rows()[rows[&id]].quantity,
                    Target::Objective => {
                        structure
                            .objective()
                            .ok_or_else(|| MathError::Contract("missing objective".into()))?
                            .quantity
                    }
                };
                pse_quantity::admission::require_same_contract(q, target, registry)?;
                let key = match c.target {
                    Target::Objective => None,
                    Target::Row(r) => Some(r),
                };
                if target_counts[&key] > 1 || c.scale != 1.0 {
                    // Physical point values cannot be summed/scaled as residual differences.
                    let ty = registry.quantity_type(q)?;
                    if ty.key.scale_kind == pse_quantity::ScaleKind::Point
                        && registry.kind(ty.key.kind)?.addition_kind
                            != pse_quantity::QuantityAdditionKind::Additive
                    {
                        return Err(MathError::Contract(
                            "point-valued output needs an explicit difference before accumulation"
                                .into(),
                        ));
                    }
                }
            }
            let coordinates: Vec<_> = binding
                .slots
                .iter()
                .enumerate()
                .filter_map(|(i, s)| column_map.contains_key(&s.source()).then_some(i))
                .collect();
            let local_columns: Vec<_> = coordinates
                .iter()
                .map(|&i| column_map[&binding.slots[i].source()])
                .collect();
            let mut groups: Vec<Option<Group>> = vec![];
            for demand in [Demand::Objective, Demand::Constraints, Demand::All] {
                let outputs: Vec<_> = binding
                    .contributions
                    .iter()
                    .filter(|c| selected(c.target, demand))
                    .map(|c| c.output)
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                if outputs.is_empty() {
                    groups.push(None);
                    continue;
                }
                let demand = LocalDemand {
                    body: binding.body,
                    outputs: outputs.clone(),
                    coordinates: coordinates.clone(),
                    order,
                };
                let key = (
                    demand.body,
                    demand.outputs.clone(),
                    demand.coordinates.clone(),
                );
                let request = *request_indices.entry(key).or_insert_with(|| {
                    requests.push(demand);
                    requests.len() - 1
                });
                groups.push(Some(Group { outputs, request }));
            }
            for c in &binding.contributions {
                if let Target::Row(r) = c.target {
                    for (i, &formal) in coordinates.iter().enumerate() {
                        if body.support().first[c.output].contains(&formal) {
                            jp.push((rows[&r], local_columns[i]));
                            jt.push(Term {
                                instance: index,
                                output: c.output,
                                i,
                                j: 0,
                                scale: c.scale * binding.slots[formal].scale(),
                                target: c.target,
                            });
                        }
                    }
                }
                if order >= DerivativeOrder::Second {
                    for (i, &a) in coordinates.iter().enumerate() {
                        for (j, &b) in coordinates.iter().enumerate() {
                            if local_columns[i] >= local_columns[j]
                                && body.support().second[c.output].contains(&(a.min(b), a.max(b)))
                            {
                                // Both ordered local pairs survive when aliases meet on a global diagonal.
                                hp.push((local_columns[i], local_columns[j]));
                                ht.push(Term {
                                    instance: index,
                                    output: c.output,
                                    i,
                                    j,
                                    scale: c.scale
                                        * binding.slots[a].scale()
                                        * binding.slots[b].scale(),
                                    target: c.target,
                                });
                            }
                        }
                    }
                }
                if jp
                    .len()
                    .checked_add(hp.len())
                    .is_none_or(|n| n > limits.contributions)
                {
                    return Err(MathError::Limit("case derivative contributions"));
                }
            }
            instances.push(Instance {
                groups,
                coordinates,
                columns: local_columns,
            });
        }
        let bound = limits.native_index;
        let jacobian = AssemblyMatrix::new(rows.len(), columns.len(), &jp, bound)?;
        let hessian = AssemblyMatrix::new(columns.len(), columns.len(), &hp, bound)?;
        Ok(Self {
            structure,
            bodies,
            columns,
            rows,
            instances,
            jacobian,
            hessian,
            jacobian_terms: jt,
            hessian_terms: ht,
            order,
            requests,
            worker_bytes: limits.worker_bytes,
            owner: None,
        })
    }
    /// Complete selected semantic inventory.
    pub fn structure(&self) -> &CaseStructure {
        &self.structure
    }
    /// Project only selected function outputs; unrelated guards/providers are not evaluated.
    pub fn functions(
        &self,
        selected_rows: &[SemanticId],
        coordinates: Vec<SemanticId>,
        registry: &QuantityRegistry,
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        let selected: BTreeSet<_> = selected_rows.iter().copied().collect();
        if selected.len() != selected_rows.len()
            || selected.iter().any(|r| !self.rows.contains_key(r))
        {
            return Err(MathError::Contract(
                "unknown or repeated function output".into(),
            ));
        }
        let instances = self
            .structure
            .instances()
            .iter()
            .filter_map(|binding| {
                let mut binding = binding.clone();
                binding
                    .contributions
                    .retain(|c| matches!(c.target, Target::Row(r) if selected.contains(&r)));
                (!binding.contributions.is_empty()).then_some(binding)
            })
            .collect();
        let structure = Arc::new(CaseStructure::new(
            self.structure.variables().to_vec(),
            self.structure.parameters().to_vec(),
            instances,
            self.structure
                .rows()
                .iter()
                .filter(|r| selected.contains(&r.id))
                .cloned()
                .collect(),
            None,
            crate::binding::CaseLimits::default(),
        )?);
        Self::prepare_with_coordinates(
            structure,
            self.bodies.clone(),
            registry,
            order,
            AssemblyLimits::default(),
            cancel,
            coordinates,
        )
    }
    /// Compile one explicitly conditional initialization block. Outside variables
    /// become fixed boundary values and unrelated row demands disappear entirely.
    /// This is not an independent optimization decomposition.
    pub fn conditional(
        &self,
        selected_rows: &BTreeSet<SemanticId>,
        selected_columns: &BTreeSet<SemanticId>,
        registry: &QuantityRegistry,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        if selected_rows.is_empty()
            || selected_rows.len() != selected_columns.len()
            || selected_rows.iter().any(|r| !self.rows.contains_key(r))
            || selected_columns
                .iter()
                .any(|c| self.columns.binary_search(c).is_err())
        {
            return Err(MathError::Contract("conditional block inventory".into()));
        }
        let variables = self
            .structure
            .variables()
            .iter()
            .cloned()
            .map(|mut v| {
                v.fixed = !selected_columns.contains(&v.port.id);
                v
            })
            .collect();
        let rows = self
            .structure
            .rows()
            .iter()
            .filter(|r| selected_rows.contains(&r.id))
            .cloned()
            .collect();
        let instances: Vec<_> = self
            .structure
            .instances()
            .iter()
            .cloned()
            .filter_map(|mut i| {
                i.contributions
                    .retain(|c| matches!(c.target,Target::Row(r)if selected_rows.contains(&r)));
                (!i.contributions.is_empty()).then_some(i)
            })
            .collect();
        let bodies = instances
            .iter()
            .map(|i| (i.body, self.bodies[&i.body].clone()))
            .collect();
        let structure = CaseStructure::new(
            variables,
            self.structure.parameters().to_vec(),
            instances,
            rows,
            None,
            crate::binding::CaseLimits::default(),
        )?;
        Self::prepare(
            Arc::new(structure),
            bodies,
            registry,
            DerivativeOrder::First,
            AssemblyLimits::default(),
            cancel,
        )
    }
    /// Consumed immutable semantic bodies.
    pub fn bodies(&self) -> &BTreeMap<ContentHash, Arc<PreparedBody>> {
        &self.bodies
    }
    /// Stable free-variable order.
    pub fn columns(&self) -> &[SemanticId] {
        &self.columns
    }
    /// Compiled derivative ceiling.
    pub fn available_order(&self) -> DerivativeOrder {
        self.requests
            .iter()
            .map(|r| self.bodies[&r.body].available_order_for(&r.coordinates))
            .min()
            .unwrap_or(DerivativeOrder::Second)
    }
    /// Derivative order requested in the prepared artifact demands.
    pub fn order(&self) -> DerivativeOrder {
        self.order
    }
    /// Canonical all-branch constraint support.
    pub fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.jacobian.matrix().symbolic()
    }
    /// Canonical lower-triangle Lagrangian support.
    pub fn hessian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.hessian.matrix().symbolic()
    }
    /// Unique immutable local evaluation demands.
    pub fn demands(&self) -> &[LocalDemand] {
        &self.requests
    }
    /// All-branch objective support in global free-variable order.
    pub fn objective_support(&self) -> BTreeSet<usize> {
        let mut support = BTreeSet::new();
        for (b, i) in self.structure.instances().iter().zip(&self.instances) {
            for c in &b.contributions {
                if c.target == Target::Objective {
                    for (&slot, &col) in i.coordinates.iter().zip(&i.columns) {
                        if self.bodies[&b.body].support().first[c.output].contains(&slot) {
                            support.insert(col);
                        }
                    }
                }
            }
        }
        support
    }
    /// Bind completed artifacts in the exact compiler demand order.
    pub fn assemble(
        self: &Arc<Self>,
        programs: Vec<Arc<CompiledBody>>,
    ) -> Result<CaseAssembly, MathError> {
        if programs.len() != self.requests.len() {
            return Err(MathError::Contract("artifact demand count".into()));
        }
        let bytes = self
            .instances
            .iter()
            .flat_map(|i| i.groups.iter().flatten())
            .try_fold(0usize, |n, g| {
                n.checked_add(programs[g.request].scratch_bytes())
            })
            .ok_or(MathError::Limit("case worker bytes"))?;
        if bytes > self.worker_bytes {
            return Err(MathError::Limit("case worker bytes"));
        }
        Ok(CaseAssembly {
            plan: Arc::clone(self),
            programs,
            numeric_worker_bytes: bytes,
        })
    }
    /// Compile a standalone plan. Runtime consumers use compiler-issued artifact requests.
    pub fn compile(
        self: &Arc<Self>,
        optimization: Optimization,
        limits: EvaluationLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<CaseAssembly, MathError> {
        let programs = self
            .requests
            .iter()
            .map(|r| {
                self.bodies[&r.body]
                    .compile(
                        &r.outputs,
                        &r.coordinates,
                        r.order,
                        optimization,
                        limits,
                        cancel,
                    )
                    .map(Arc::new)
            })
            .collect::<Result<_, _>>()?;
        self.assemble(programs)
    }
}
/// One deduplicated local demand, independent of numeric trial values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalDemand {
    /// Admitted semantic body identity.
    pub body: ContentHash,
    /// Ordered output selection.
    pub outputs: Vec<usize>,
    /// Ordered formal derivative coordinates (aliases remain distinct).
    pub coordinates: Vec<usize>,
    /// Derivative ceiling.
    pub order: DerivativeOrder,
}
/// A structural plan bound to immutable numeric programs.
#[derive(Clone, Debug)]
pub struct CaseAssembly {
    plan: Arc<CasePlan>,
    programs: Vec<Arc<CompiledBody>>,
    numeric_worker_bytes: usize,
}
impl std::ops::Deref for CaseAssembly {
    type Target = CasePlan;
    fn deref(&self) -> &Self::Target {
        &self.plan
    }
}
impl CaseAssembly {
    /// Aggregate per-instance numeric evaluator capacity admitted by the plan.
    pub fn numeric_worker_bytes(&self) -> usize {
        self.numeric_worker_bytes
    }
    /// An attempt owns all mutable evaluator and provider state.
    pub fn worker(
        self: &Arc<Self>,
        providers: BTreeMap<ProviderKey, Box<dyn Provider>>,
        cancel: Arc<AtomicBool>,
    ) -> CaseWorker {
        let groups = self
            .instances
            .iter()
            .map(|i| {
                i.groups
                    .iter()
                    .map(|g| {
                        g.as_ref().map(|g| GroupWorker {
                            worker: self.programs[g.request].worker(),
                            cache: None,
                        })
                    })
                    .collect()
            })
            .collect();
        CaseWorker {
            assembly: Arc::clone(self),
            groups,
            providers,
            cancel,
            jacobian: self.jacobian.clone(),
            hessian: self.hessian.clone(),
        }
    }
}
#[derive(Debug)]
struct GroupWorker {
    worker: Worker,
    cache: Option<(Vec<u64>, DerivativeOrder, Evaluation)>,
}
/// Isolated attempt scratch. Failed evaluation clears the failed cache and never publishes output.
#[derive(Debug)]
pub struct CaseWorker {
    assembly: Arc<CaseAssembly>,
    groups: Vec<Vec<Option<GroupWorker>>>,
    providers: BTreeMap<ProviderKey, Box<dyn Provider>>,
    cancel: Arc<AtomicBool>,
    jacobian: AssemblyMatrix,
    hessian: AssemblyMatrix,
}
/// A library-evaluated source output before signed row aggregation.
#[derive(Clone, Debug)]
pub struct OutputValue {
    /// Original authored instance.
    pub instance: SemanticId,
    /// Original local output ordinal.
    pub output: usize,
    /// Canonical physical value before contribution scaling.
    pub value: f64,
}
impl CaseWorker {
    /// Immutable products shared by the attempt.
    pub fn assembly(&self) -> &Arc<CaseAssembly> {
        &self.assembly
    }
    fn evaluate(
        &mut self,
        values: &CaseValues,
        demand: Demand,
        order: DerivativeOrder,
    ) -> Result<(), MathError> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        for (i, binding) in self.assembly.structure.instances().iter().enumerate() {
            let Some(worker) = &mut self.groups[i][demand as usize] else {
                continue;
            };
            let context = |cause| MathError::Instance {
                instance: binding.instance,
                cause: Box::new(cause),
            };
            let inputs = binding.values(values).map_err(|cause| {
                worker.cache = None;
                context(cause)
            })?;
            let bits: Vec<_> = inputs.iter().map(|v| v.to_bits()).collect();
            if worker
                .cache
                .as_ref()
                .is_some_and(|(x, o, _)| *x == bits && *o >= order)
            {
                continue;
            }
            worker.cache = None;
            let result = worker
                .worker
                .evaluate(&inputs, order, &mut self.providers, &self.cancel)
                .map_err(context)?;
            worker.cache = Some((bits, order, result));
        }
        Ok(())
    }
    fn result(
        &self,
        instance: usize,
        output: usize,
        demand: Demand,
    ) -> Result<(&Evaluation, usize), MathError> {
        let group = self.assembly.instances[instance].groups[demand as usize]
            .as_ref()
            .ok_or_else(|| MathError::Contract("missing compiled demand".into()))?;
        let row = group
            .outputs
            .binary_search(&output)
            .map_err(|_| MathError::Contract("missing demand output".into()))?;
        let result = self.groups[instance][demand as usize]
            .as_ref()
            .and_then(|g| g.cache.as_ref())
            .ok_or_else(|| MathError::Contract("unevaluated demand".into()))?;
        Ok((&result.2, row))
    }
    /// Selected objective only, normalized for a minimization oracle.
    pub fn objective(&mut self, values: &CaseValues) -> Result<f64, MathError> {
        self.evaluate(values, Demand::Objective, DerivativeOrder::Value)?;
        let mut value = 0.0;
        for (i, b) in self.assembly.structure.instances().iter().enumerate() {
            for c in &b.contributions {
                if c.target == Target::Objective {
                    let (v, r) = self.result(i, c.output, Demand::Objective)?;
                    value += c.scale * v.values[r];
                }
            }
        }
        finite(value * self.objective_sign())
    }
    fn objective_sign(&self) -> f64 {
        self.assembly
            .structure
            .objective()
            .map_or(1.0, |o| o.sense.sign())
    }
    /// Constraint values in selected row order; repeated contributions accumulate.
    pub fn constraints(&mut self, values: &CaseValues) -> Result<Vec<f64>, MathError> {
        self.evaluate(values, Demand::Constraints, DerivativeOrder::Value)?;
        let mut out = vec![0.0; self.assembly.rows.len()];
        for (i, b) in self.assembly.structure.instances().iter().enumerate() {
            for c in &b.contributions {
                if let Target::Row(id) = c.target {
                    let (v, r) = self.result(i, c.output, Demand::Constraints)?;
                    out[self.assembly.rows[&id]] += c.scale * v.values[r];
                }
            }
        }
        out.into_iter().map(finite).collect()
    }
    /// Retain independently visible source terms from the most recent constraint evaluation.
    pub fn constraint_sources(&self) -> Result<Vec<OutputValue>, MathError> {
        let mut out = Vec::new();
        for (i, b) in self.assembly.structure.instances().iter().enumerate() {
            let outputs: BTreeSet<_> = b
                .contributions
                .iter()
                .filter(|c| matches!(c.target, Target::Row(_)))
                .map(|c| c.output)
                .collect();
            for output in outputs {
                let (evaluation, row) = self.result(i, output, Demand::Constraints)?;
                out.push(OutputValue {
                    instance: b.instance,
                    output,
                    value: finite(evaluation.values[row])?,
                });
            }
        }
        Ok(out)
    }
    /// Objective gradient in free-variable order and minimization orientation.
    pub fn gradient(&mut self, values: &CaseValues) -> Result<Vec<f64>, MathError> {
        self.evaluate(values, Demand::Objective, DerivativeOrder::First)?;
        let mut out = vec![0.0; self.assembly.columns.len()];
        for (i, b) in self.assembly.structure.instances().iter().enumerate() {
            let local = &self.assembly.instances[i];
            for c in &b.contributions {
                if c.target == Target::Objective {
                    let (v, r) = self.result(i, c.output, Demand::Objective)?;
                    for (k, &formal) in local.coordinates.iter().enumerate() {
                        out[local.columns[k]] += self.objective_sign()
                            * c.scale
                            * b.slots[formal].scale()
                            * v.jacobian[r * local.coordinates.len() + k];
                    }
                }
            }
        }
        out.into_iter().map(finite).collect()
    }
    /// Refill the canonical constraint Jacobian without rebuilding structure.
    pub fn jacobian(
        &mut self,
        values: &CaseValues,
    ) -> Result<&faer::sparse::SparseColMat<usize, f64>, MathError> {
        self.evaluate(values, Demand::Constraints, DerivativeOrder::First)?;
        self.jacobian.clear();
        for (k, t) in self.assembly.jacobian_terms.iter().enumerate() {
            let (v, r) = self.result(t.instance, t.output, Demand::Constraints)?;
            let n = self.assembly.instances[t.instance].coordinates.len();
            let value = t.scale * v.jacobian[r * n + t.i];
            self.jacobian.add(k, value)?;
        }
        Ok(self.jacobian.matrix())
    }
    /// Refill the lower Lagrangian Hessian with mapped row and objective weights.
    pub fn hessian(
        &mut self,
        values: &CaseValues,
        objective_weight: f64,
        multipliers: &[f64],
    ) -> Result<&faer::sparse::SparseColMat<usize, f64>, MathError> {
        if self.assembly.order < DerivativeOrder::Second
            || !objective_weight.is_finite()
            || multipliers.len() != self.assembly.rows.len()
            || multipliers.iter().any(|v| !v.is_finite())
        {
            return Err(MathError::Contract("Lagrangian Hessian demand".into()));
        }
        self.evaluate(values, Demand::All, DerivativeOrder::Second)?;
        self.hessian.clear();
        for (k, t) in self.assembly.hessian_terms.iter().enumerate() {
            let weight = match t.target {
                Target::Objective => objective_weight * self.objective_sign(),
                Target::Row(id) => multipliers[self.assembly.rows[&id]],
            };
            let (v, r) = self.result(t.instance, t.output, Demand::All)?;
            let n = self.assembly.instances[t.instance].coordinates.len();
            let value = weight * t.scale * v.hessians[r * n * n + t.i * n + t.j];
            self.hessian.add(k, value)?;
        }
        Ok(self.hessian.matrix())
    }
    /// Apply the assembled Jacobian through faer sparse multiplication.
    pub fn jacobian_product(
        &mut self,
        values: &CaseValues,
        direction: &[f64],
        output: &mut [f64],
    ) -> Result<(), MathError> {
        self.jacobian(values)?;
        self.jacobian.product(direction, output)
    }
}
fn selected(target: Target, demand: Demand) -> bool {
    match demand {
        Demand::Objective => target == Target::Objective,
        Demand::Constraints => target != Target::Objective,
        Demand::All => true,
    }
}
fn finite(value: f64) -> Result<f64, MathError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(MathError::Contract("nonfinite assembled result".into()))
    }
}
