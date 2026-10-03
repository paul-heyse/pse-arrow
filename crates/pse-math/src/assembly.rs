// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical local gathers, explicit row contributions and library-owned sparse products.
use crate::{
    MathError,
    binding::{CaseStructure, CaseValues, Target},
    guarded::{CompiledBody, Evaluation, PreparedBody, PreparedSupport, Worker},
    index::{Addend, Entry, GlobalCol, GlobalRow, Slot},
    jets::EvaluationLimits,
    library::Optimization,
    sparse::AssemblyMatrix,
};
use enum_map::EnumMap;
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
/// Which contributions one compiled local demand evaluates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, enum_map::Enum)]
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
/// One binding's compiled demands and its derivative coordinates: `coordinates[k]` is the
/// formal slot of local coordinate `k`, and `columns[k]` its case-global column.
#[derive(Clone, Debug)]
struct Instance {
    groups: EnumMap<Demand, Option<Group>>,
    coordinates: Vec<Slot>,
    columns: Vec<GlobalCol>,
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
    columns: Arc<Vec<SemanticId>>,
    rows: Arc<BTreeMap<SemanticId, GlobalRow>>,
    instances: Arc<Vec<Instance>>,
    jacobian: Arc<AssemblyMatrix>,
    hessian: Arc<AssemblyMatrix>,
    jacobian_terms: Arc<Vec<Term>>,
    hessian_terms: Arc<Vec<Term>>,
    order: DerivativeOrder,
    requests: Arc<Vec<LocalDemand>>,
    supports: Arc<Vec<Arc<PreparedSupport>>>,
    available: DerivativeOrder,
    limits: AssemblyLimits,
    owner: Option<Arc<dyn crate::AllocationOwner>>,
}
impl CasePlan {
    /// Process-local shared storage identities, independent of owner wrappers and body data.
    pub fn allocation_identity(&self) -> Vec<usize> {
        vec![
            Arc::as_ptr(&self.structure) as usize,
            Arc::as_ptr(&self.columns) as usize,
            Arc::as_ptr(&self.rows) as usize,
            Arc::as_ptr(&self.instances) as usize,
            Arc::as_ptr(&self.jacobian) as usize,
            Arc::as_ptr(&self.hessian) as usize,
            Arc::as_ptr(&self.jacobian_terms) as usize,
            Arc::as_ptr(&self.hessian_terms) as usize,
            Arc::as_ptr(&self.requests) as usize,
            Arc::as_ptr(&self.supports) as usize,
        ]
    }
    /// Plan storage without immutable body data, which has its own unique lease.
    pub fn allocation_bytes(&self) -> usize {
        self.retained_bytes()
            .saturating_sub(self.bodies.values().map(|b| b.retained_bytes()).sum())
    }
    /// Owned shallow wrappers created by attaching escaping allocation ownership.
    pub fn owner_wrapper_bytes(&self) -> usize {
        size_of::<Self>()
            + self.bodies.len() * (size_of::<PreparedBody>() + size_of::<ContentHash>() + 96)
    }
    /// Known immutable payload; shared bodies are counted once within this plan.
    /// Opaque library and map allocation overhead is accounted by the runtime policy.
    pub fn retained_bytes(&self) -> usize {
        let structure = &self.structure;
        size_of::<Self>()
            + size_of_val(structure.variables())
            + size_of_val(structure.parameters())
            + size_of_val(structure.rows())
            + structure
                .instances()
                .iter()
                .map(|i| {
                    size_of_val(i)
                        + i.checked_members.len() * (size_of::<(SemanticId, SemanticId)>() + 96)
                        + size_of_val(i.slots.as_slice())
                        + size_of_val(i.contributions.as_slice())
                })
                .sum::<usize>()
            + self
                .bodies
                .values()
                .map(|b| b.retained_bytes())
                .sum::<usize>()
            + self.columns.capacity() * size_of::<SemanticId>()
            + self.rows.len() * size_of::<(SemanticId, GlobalRow)>()
            + self
                .instances
                .iter()
                .map(|i| {
                    size_of_val(i)
                        + i.coordinates.capacity() * size_of::<Slot>()
                        + i.columns.capacity() * size_of::<GlobalCol>()
                        + i.groups
                            .values()
                            .flatten()
                            .map(|g| size_of_val(g) + g.outputs.capacity() * size_of::<usize>())
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self.jacobian.retained_bytes()
            + self.hessian.retained_bytes()
            + (self.jacobian_terms.capacity() + self.hessian_terms.capacity()) * size_of::<Term>()
            + self.requests.capacity() * size_of::<LocalDemand>()
            + self.supports.capacity() * size_of::<Arc<PreparedSupport>>()
            + self
                .supports
                .iter()
                .map(|s| s.retained_bytes())
                .sum::<usize>()
    }
    /// Retain runtime accounting on every escaping structural plan clone.
    pub fn with_owner(mut self, owner: Arc<dyn crate::AllocationOwner>) -> Self {
        for body in self.bodies.values_mut() {
            *body = Arc::new(body.as_ref().clone().with_owner(owner.clone()));
        }
        for support in Arc::make_mut(&mut self.supports) {
            *support = Arc::new(support.as_ref().clone().with_owner(owner.clone()));
        }
        self.owner = Some(crate::retain_allocation_owner(self.owner.take(), owner));
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
        let column_map: BTreeMap<_, _> = columns
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, GlobalCol::new(i)))
            .collect();
        let rows: BTreeMap<_, _> = structure
            .rows()
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id, GlobalRow::new(i)))
            .collect();
        if columns.len() > limits.native_index || rows.len() > limits.native_index {
            return Err(MathError::Limit("native index width"));
        }
        let mut requests = Vec::<LocalDemand>::new();
        let mut supports = Vec::<Arc<PreparedSupport>>::new();
        let mut available = DerivativeOrder::Second;
        let mut shared_supports =
            BTreeMap::<(ContentHash, Vec<usize>, Vec<usize>), PreparedSupport>::new();
        let mut request_indices = BTreeMap::new();
        let mut instances = vec![];
        let mut target_counts = BTreeMap::new();
        for binding in structure.instances() {
            for c in &binding.contributions {
                *target_counts.entry(c.target).or_insert(0usize) += 1;
            }
        }
        for binding in structure.instances() {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            let body = bodies
                .get(&binding.body)
                .ok_or_else(|| MathError::Contract("missing prepared body".into()))?;
            if !body
                .checked_members()
                .iter()
                .eq(binding.checked_members.keys())
                || binding
                    .checked_members
                    .values()
                    .any(|id| *id == SemanticId::NIL)
            {
                return Err(MathError::Contract(
                    "incomplete or invalid checked-member attribution".into(),
                ));
            }
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
                    Target::Row(id) => structure.rows()[rows[&id].get()].quantity,
                    Target::Objective(level) => {
                        structure
                            .objectives()
                            .get(level)
                            .ok_or_else(|| MathError::Contract("missing objective".into()))?
                            .quantity
                    }
                };
                pse_quantity::admission::require_same_contract(q, target, registry)?;
                if target_counts[&c.target] > 1 || c.scale != 1.0 {
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
            let coordinates: Vec<Slot> = binding
                .slots
                .iter()
                .enumerate()
                .filter_map(|(i, s)| column_map.contains_key(&s.source()).then_some(Slot::new(i)))
                .collect();
            let local_columns: Vec<GlobalCol> = coordinates
                .iter()
                .map(|&i| column_map[&binding.slots[i.get()].source()])
                .collect();
            // The library compiles over formal slot positions.
            let formal: Vec<usize> = coordinates.iter().map(|s| s.get()).collect();
            let all_outputs: Vec<_> = binding
                .contributions
                .iter()
                .map(|c| c.output)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let shared_key = (binding.body, all_outputs.clone(), formal.clone());
            if !all_outputs.is_empty() && !shared_supports.contains_key(&shared_key) {
                let support = if order == DerivativeOrder::First {
                    body.incidence(&all_outputs, &formal, cancel)?
                } else {
                    body.prepare_support(&all_outputs, &formal, order, cancel)?
                };
                shared_supports.insert(shared_key.clone(), support);
            }
            let mut groups = EnumMap::<Demand, Option<Group>>::default();
            for (demand, group) in &mut groups {
                let outputs: Vec<_> = binding
                    .contributions
                    .iter()
                    .filter(|c| selected(c.target, demand))
                    .map(|c| c.output)
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                if outputs.is_empty() {
                    continue;
                }
                let demand = LocalDemand {
                    body: binding.body,
                    outputs: outputs.clone(),
                    coordinates: formal.clone(),
                    order,
                };
                let key = (
                    demand.body,
                    demand.outputs.clone(),
                    demand.coordinates.clone(),
                );
                let request = if let Some(&request) = request_indices.get(&key) {
                    request
                } else {
                    available = available.min(
                        body.available_order_for_outputs(&demand.outputs, &demand.coordinates)?,
                    );
                    let shared = shared_supports
                        .get(&shared_key)
                        .ok_or_else(|| MathError::Contract("missing selected support".into()))?;
                    let support = if demand.outputs == all_outputs {
                        shared.clone()
                    } else {
                        shared.select_outputs(&demand.outputs, cancel)?
                    };
                    let request = requests.len();
                    requests.push(demand);
                    supports.push(Arc::new(support));
                    request_indices.insert(key, request);
                    request
                };
                *group = Some(Group { outputs, request });
            }
            instances.push(Instance {
                groups,
                coordinates,
                columns: local_columns,
            });
        }
        let patterns = case_patterns(
            &structure,
            &rows,
            &instances,
            &supports,
            order,
            columns.len(),
            limits,
        )?;
        Ok(Self {
            structure,
            bodies,
            columns: Arc::new(columns),
            rows: Arc::new(rows),
            instances: Arc::new(instances),
            jacobian: Arc::new(patterns.jacobian),
            hessian: Arc::new(patterns.hessian),
            jacobian_terms: Arc::new(patterns.jacobian_terms),
            hessian_terms: Arc::new(patterns.hessian_terms),
            order,
            requests: Arc::new(requests),
            supports: Arc::new(supports),
            available,
            limits,
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
    /// The parametric projection of this plan (Plan 22 S1): the same structure, objective
    /// and rows, with this plan's columns followed by `parameters`, in request order, as
    /// derivative coordinates, at second order. [`Self::functions`] drops the objective; a
    /// parametric sensitivity needs the Lagrangian, so this keeps it. The parameter
    /// coordinates are differentiated, never solved for: the KKT-point analysis pins them
    /// (sIPOPT's pin formulation), so their Hessian and Jacobian entries are what the
    /// parametric step and the reduced Hessian read.
    ///
    /// # Errors
    /// An empty, repeated or undeclared parameter, a parameter already among the columns,
    /// or the ordinary preparation failures.
    pub fn parametric(
        &self,
        parameters: &[SemanticId],
        order: DerivativeOrder,
        registry: &QuantityRegistry,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        let declared: BTreeSet<_> = self.structure.parameters().iter().map(|p| p.id).collect();
        if parameters.is_empty() || parameters.iter().any(|p| !declared.contains(p)) {
            return Err(MathError::Contract(
                "parametric coordinates are declared parameters, at least one".into(),
            ));
        }
        let coordinates = self
            .columns
            .iter()
            .chain(parameters)
            .copied()
            .collect::<Vec<_>>();
        Self::prepare_with_coordinates(
            self.structure.clone(),
            self.bodies.clone(),
            registry,
            order,
            AssemblyLimits {
                worker_bytes: self.limits.worker_bytes,
                ..AssemblyLimits::default()
            },
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
    /// Monotonic immutable support upgrade under the original case and body allowances.
    /// Failed or cancelled stronger construction leaves this plan and its products intact.
    pub fn prepare_order(
        &self,
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        if order <= self.order {
            return Ok(self.clone());
        }
        let mut stronger = BTreeMap::new();
        for instance in self.instances.iter() {
            if let Some(group) = &instance.groups[Demand::All]
                && !stronger.contains_key(&group.request)
            {
                stronger.insert(
                    group.request,
                    Arc::new(self.supports[group.request].upgrade(order, cancel)?),
                );
            }
        }
        let mut supports = Vec::with_capacity(self.supports.len());
        for (request, previous) in self.supports.iter().enumerate() {
            if let Some(support) = stronger.get(&request) {
                supports.push(Arc::clone(support));
                continue;
            }
            let parent = self
                .instances
                .iter()
                .find_map(|instance| {
                    instance
                        .groups
                        .values()
                        .flatten()
                        .any(|group| group.request == request)
                        .then(|| instance.groups[Demand::All].as_ref().map(|all| all.request))
                        .flatten()
                })
                .ok_or_else(|| {
                    MathError::Contract("support demand has no instance owner".into())
                })?;
            supports.push(Arc::new(
                stronger[&parent].select_outputs(previous.outputs(), cancel)?,
            ));
        }
        let patterns = case_patterns(
            &self.structure,
            &self.rows,
            &self.instances,
            &supports,
            order,
            self.columns.len(),
            self.limits,
        )?;
        let mut result = self.clone();
        result.order = order;
        result.requests = Arc::new(
            self.requests
                .iter()
                .cloned()
                .map(|mut request| {
                    request.order = order;
                    request
                })
                .collect(),
        );
        result.supports = Arc::new(supports);
        result.jacobian = Arc::new(patterns.jacobian);
        result.hessian = Arc::new(patterns.hessian);
        result.jacobian_terms = Arc::new(patterns.jacobian_terms);
        result.hessian_terms = Arc::new(patterns.hessian_terms);
        Ok(result)
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
        self.available
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
    /// Immutable support products aligned with the exact local demand order.
    pub fn supports(&self) -> &[Arc<PreparedSupport>] {
        &self.supports
    }
    /// Explicit structural First admission, aligned with original instances.
    /// Each product retains its selected output map and original formal coordinates.
    pub fn incidence(
        &self,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Vec<Arc<PreparedSupport>>, MathError> {
        let mut products = Vec::with_capacity(self.instances.len());
        let mut shared = BTreeMap::new();
        for (binding, instance) in self.structure.instances().iter().zip(self.instances.iter()) {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            let group = instance.groups[Demand::All].as_ref().ok_or_else(|| {
                MathError::Contract("instance has no selected output incidence".into())
            })?;
            let ready = &self.supports[group.request];
            let key = (
                binding.body,
                ready.outputs().to_vec(),
                ready.coordinates().to_vec(),
            );
            let product = if let Some(product) = shared.get(&key) {
                Arc::clone(product)
            } else {
                let product = if ready.order() >= DerivativeOrder::First {
                    Arc::clone(ready)
                } else {
                    Arc::new(ready.upgrade(DerivativeOrder::First, cancel)?)
                };
                shared.insert(key, Arc::clone(&product));
                product
            };
            products.push(product);
        }
        Ok(products)
    }
    /// Conservative body-local incidence for an original contribution and instance.
    /// Value plans hold no derivative support; explicitly prepare First for structure.
    pub fn incidence_for(&self, instance: usize, output: usize) -> Option<&BTreeSet<usize>> {
        let group = self.instances.get(instance)?.groups[Demand::All].as_ref()?;
        let support = self.supports.get(group.request)?;
        let row = support.outputs().iter().position(|&i| i == output)?;
        support.support().first.get(row)
    }
    /// All-branch objective support in global free-variable order.
    pub fn objective_support(&self) -> BTreeSet<GlobalCol> {
        let mut support = BTreeSet::new();
        for (instance, (b, i)) in self
            .structure
            .instances()
            .iter()
            .zip(self.instances.iter())
            .enumerate()
        {
            for c in &b.contributions {
                if c.target == Target::PRIMARY {
                    for (&slot, &col) in i.coordinates.iter().zip(&i.columns) {
                        if self
                            .incidence_for(instance, c.output)
                            .is_some_and(|s| s.contains(&slot.get()))
                        {
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
        for (request, program) in self.requests.iter().zip(&programs) {
            let support = program.prepared_support();
            if support.outputs() != request.outputs
                || support.coordinates() != request.coordinates
                || program.compiled_order() != request.order
                || support.body() != self.bodies[&request.body].as_ref()
            {
                return Err(MathError::Contract(
                    "artifact does not match selected admitted demand".into(),
                ));
            }
        }
        let bytes = self
            .instances
            .iter()
            .flat_map(|i| i.groups.values().flatten())
            .try_fold(0usize, |n, g| {
                n.checked_add(programs[g.request].scratch_bytes())
            })
            .ok_or(MathError::Limit("case worker bytes"))?;
        if bytes > self.limits.worker_bytes {
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
            .supports
            .iter()
            .map(|support| support.compile(optimization, limits, cancel).map(Arc::new))
            .collect::<Result<_, _>>()?;
        self.assemble(programs)
    }
}
struct CasePatterns {
    jacobian: AssemblyMatrix,
    hessian: AssemblyMatrix,
    jacobian_terms: Vec<Term>,
    hessian_terms: Vec<Term>,
}
fn case_patterns(
    structure: &CaseStructure,
    rows: &BTreeMap<SemanticId, GlobalRow>,
    instances: &[Instance],
    supports: &[Arc<PreparedSupport>],
    order: DerivativeOrder,
    columns: usize,
    limits: AssemblyLimits,
) -> Result<CasePatterns, MathError> {
    let mut jp = Vec::<Entry<GlobalRow, GlobalCol>>::new();
    let mut hp = Vec::<Entry<GlobalCol, GlobalCol>>::new();
    let mut jt = vec![];
    let mut ht = vec![];
    for (index, (binding, instance)) in structure.instances().iter().zip(instances).enumerate() {
        let support = instance.groups[Demand::All]
            .as_ref()
            .map(|g| &supports[g.request]);
        for contribution in &binding.contributions {
            if order >= DerivativeOrder::First
                && let Target::Row(row) = contribution.target
            {
                let first = support
                    .and_then(|s| s.first_for_output(contribution.output))
                    .ok_or_else(|| {
                        MathError::Contract("missing selected First incidence".into())
                    })?;
                for (i, &formal) in instance.coordinates.iter().enumerate() {
                    if first.contains(&formal.get()) {
                        contribution_allowance(jp.len(), hp.len(), limits)?;
                        jp.push(Entry::new(rows[&row], instance.columns[i]));
                        jt.push(Term {
                            instance: index,
                            output: contribution.output,
                            i,
                            j: 0,
                            scale: contribution.scale * binding.slots[formal.get()].scale(),
                            target: contribution.target,
                        });
                    }
                }
            }
            if order >= DerivativeOrder::Second && selected(contribution.target, Demand::All) {
                let second = support
                    .and_then(|s| s.second_for_output(contribution.output))
                    .ok_or_else(|| MathError::Contract("missing selected Second support".into()))?;
                for (i, &a) in instance.coordinates.iter().enumerate() {
                    for (j, &b) in instance.coordinates.iter().enumerate() {
                        let (a, b) = (a.get(), b.get());
                        if instance.columns[i] >= instance.columns[j]
                            && second.contains(&(a.min(b), a.max(b)))
                        {
                            contribution_allowance(jp.len(), hp.len(), limits)?;
                            hp.push(Entry::new(instance.columns[i], instance.columns[j]));
                            ht.push(Term {
                                instance: index,
                                output: contribution.output,
                                i,
                                j,
                                scale: contribution.scale
                                    * binding.slots[a].scale()
                                    * binding.slots[b].scale(),
                                target: contribution.target,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(CasePatterns {
        jacobian: AssemblyMatrix::new(rows.len(), columns, &jp, limits.native_index)?,
        hessian: AssemblyMatrix::hessian(columns, &hp, limits.native_index)?,
        jacobian_terms: jt,
        hessian_terms: ht,
    })
}
fn contribution_allowance(
    first: usize,
    second: usize,
    limits: AssemblyLimits,
) -> Result<(), MathError> {
    if first
        .checked_add(second)
        .and_then(|n| n.checked_add(1))
        .is_none_or(|n| n > limits.contributions)
    {
        return Err(MathError::Limit("case derivative contributions"));
    }
    Ok(())
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
                EnumMap::from_fn(|demand| {
                    i.groups[demand].as_ref().map(|g| GroupWorker {
                        worker: self.programs[g.request].worker(),
                        cache: None,
                    })
                })
            })
            .collect();
        CaseWorker {
            assembly: Arc::clone(self),
            groups,
            providers,
            cancel,
            jacobian: self.jacobian.as_ref().clone(),
            hessian: self.hessian.as_ref().clone(),
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
    groups: Vec<EnumMap<Demand, Option<GroupWorker>>>,
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
    /// Shared cooperative cancellation owner for bounded library work on this admitted worker.
    pub fn cancellation(&self) -> &Arc<AtomicBool> {
        &self.cancel
    }
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
            let Some(worker) = &mut self.groups[i][demand] else {
                continue;
            };
            let context = |cause| MathError::Instance {
                instance: binding.instance,
                checked_members: binding.checked_members.clone(),
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
        let group = self.assembly.instances[instance].groups[demand]
            .as_ref()
            .ok_or_else(|| MathError::Contract("missing compiled demand".into()))?;
        let row = group
            .outputs
            .binary_search(&output)
            .map_err(|_| MathError::Contract("missing demand output".into()))?;
        let result = self.groups[instance][demand]
            .as_ref()
            .and_then(|g| g.cache.as_ref())
            .ok_or_else(|| MathError::Contract("unevaluated demand".into()))?;
        Ok((&result.2, row))
    }
    /// Read evidence from the exact last demanded programs; this never evaluates an
    /// inactive branch or an unrelated output just to create observations.
    pub fn applicability_observations(&self) -> Vec<pse_model::applicability::Observation> {
        let mut observations = Vec::new();
        for (instance, groups) in self.groups.iter().enumerate() {
            for group in groups.values().flatten() {
                if let Some((_, _, evaluation)) = &group.cache {
                    for observation in &evaluation.applicability {
                        let mut observation = observation.clone();
                        observation.instance =
                            Some(self.assembly.structure.instances()[instance].instance);
                        if !observations.contains(&observation) {
                            observations.push(observation);
                        }
                    }
                }
            }
        }
        observations
    }
    /// Selected objective only, normalized for a minimization oracle.
    pub fn objective(&mut self, values: &CaseValues) -> Result<f64, MathError> {
        self.evaluate(values, Demand::Objective, DerivativeOrder::Value)?;
        let mut value = 0.0;
        for (i, b) in self.assembly.structure.instances().iter().enumerate() {
            for c in &b.contributions {
                if c.target == Target::PRIMARY {
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
                    out[self.assembly.rows[&id].get()] += c.scale * v.values[r];
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
                if c.target == Target::PRIMARY {
                    let (v, r) = self.result(i, c.output, Demand::Objective)?;
                    for (k, &formal) in local.coordinates.iter().enumerate() {
                        out[local.columns[k].get()] += self.objective_sign()
                            * c.scale
                            * b.slots[formal.get()].scale()
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
            self.jacobian.add(Addend::new(k), value)?;
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
                Target::Objective(_) => objective_weight * self.objective_sign(),
                Target::Row(id) => multipliers[self.assembly.rows[&id].get()],
            };
            let (v, r) = self.result(t.instance, t.output, Demand::All)?;
            let n = self.assembly.instances[t.instance].coordinates.len();
            let value = weight * t.scale * v.hessians[r * n * n + t.i * n + t.j];
            self.hessian.add(Addend::new(k), value)?;
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
        Demand::Objective => target == Target::PRIMARY,
        Demand::Constraints => matches!(target, Target::Row(_)),
        // A later lexicographic objective is projected to coefficients, never evaluated.
        Demand::All => matches!(target, Target::Row(_)) || target == Target::PRIMARY,
    }
}
fn finite(value: f64) -> Result<f64, MathError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(MathError::Contract("nonfinite assembled result".into()))
    }
}
