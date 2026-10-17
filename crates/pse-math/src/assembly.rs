// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical local gathers, explicit row contributions and library-owned sparse products.
use crate::{
    MathError,
    binding::{CaseStructure, CaseValues, EvalBindings, Target},
    execution::TrialEvaluation,
    guarded::{CompiledBody, PreparedBody, PreparedSupport, Worker},
    index::{Addend, Entry, GlobalCol, GlobalRow, Slot},
    jets::EvaluationLimits,
    library::Optimization,
    sparse::AssemblyMatrix,
};
#[path = "assembly_observation.rs"]
mod observation;
use enum_map::EnumMap;
use observation::CaseObservation;
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
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AssemblyLimits {
    /// Maximum original derivative contributions before duplicate accumulation.
    pub contributions: usize,
    /// Largest dimension/index/entry count representable by the selected native ABI.
    pub native_index: usize,
    /// Complete attempt storage budget: shared evaluator scratch, clones and occurrence caches.
    pub worker_bytes: usize,
}
impl Default for AssemblyLimits {
    fn default() -> Self {
        Self {
            contributions: 1_000_000,
            native_index: i32::MAX as usize,
            worker_bytes: 2 * 1024 * 1024 * 1024,
        }
    }
}
/// Which contributions one compiled local demand evaluates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, enum_map::Enum)]
pub enum EvaluationDemand {
    /// Selected primary objective only.
    Objective,
    /// Selected constraint outputs only.
    Constraints,
    /// All selected objective and constraint outputs.
    All,
    /// One directional constraint action.
    Directional,
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
    groups: EnumMap<EvaluationDemand, Option<Group>>,
    coordinates: Vec<Slot>,
    columns: Vec<GlobalCol>,
}
#[derive(Clone, Debug)]
struct Term {
    instance: usize,
    output: usize,
    result_row: usize,
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
    projection: Arc<crate::factorable::ProjectionTopology>,
    rows: Arc<BTreeMap<SemanticId, GlobalRow>>,
    row_contributions: Arc<BTreeMap<SemanticId, Vec<(usize, usize)>>>,
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
/// Distinct dependency projections of one original contribution. Execution includes
/// branch and validity inputs, including fixed symbols; numerical support does not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContributionDependencies {
    /// Original target, retaining every objective level and inequality identity.
    pub target: Target,
    /// Numerical all-branch first support, without promoting derivative capability.
    pub numerical: BTreeSet<SemanticId>,
    /// Inputs needed to execute and validate this contribution.
    pub execution: BTreeSet<SemanticId>,
}
impl CasePlan {
    /// Complete original dependency projections. Body-wide control attribution is
    /// conservative; unlike solver columns this inventory retains fixed inputs.
    pub fn dependencies(
        &self,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Vec<ContributionDependencies>, MathError> {
        let mut result = Vec::new();
        let _ = self.visit_dependencies(cancel, |dependency| {
            result.push(dependency);
            std::ops::ControlFlow::<std::convert::Infallible>::Continue(())
        })?;
        Ok(result)
    }
    /// Checked peak population for original dependency discovery, without evaluator
    /// compilation or sparse derivative assembly. Opaque bodies retain caller-owned
    /// conservative admission. Source plans and bodies remain borrowed and charged.
    pub fn dependency_allocation_bound(&self) -> Result<Option<usize>, MathError> {
        let overflow = || MathError::Limit("dependency discovery extent");
        let mut bytes = size_of::<Vec<ContributionDependencies>>();
        for binding in self.structure.instances() {
            let body = &self.bodies[&binding.body];
            // First support is contained by the owner's second-support traversal
            // envelope; this does not compile second derivatives or evaluators.
            let Some(scratch) =
                body.arithmetic_analysis_allocation_bound(Some(binding.slots.len()))?
            else {
                return Ok(None);
            };
            let dimensions = binding
                .slots
                .len()
                .checked_add(binding.contributions.len())
                .ok_or_else(overflow)?;
            let metadata = dimensions
                .checked_mul(11 * size_of::<usize>() + 16 * size_of::<usize>())
                .ok_or_else(overflow)?;
            let dependencies = binding
                .contributions
                .len()
                .checked_mul(
                    size_of::<ContributionDependencies>()
                        .checked_add(
                            binding
                                .slots
                                .len()
                                .checked_mul(
                                    2 * (11 * size_of::<SemanticId>() + 16 * size_of::<usize>()),
                                )
                                .ok_or_else(overflow)?,
                        )
                        .ok_or_else(overflow)?,
                )
                .and_then(|n| n.checked_mul(2))
                .ok_or_else(overflow)?;
            bytes = bytes
                .checked_add(scratch)
                .and_then(|n| n.checked_add(metadata))
                .and_then(|n| n.checked_add(dependencies))
                .ok_or_else(overflow)?;
        }
        Ok(Some(bytes))
    }
    /// Capability traversal scratch for precisely the selected row contributors.
    /// The body owner includes compact stage copies and transitive provider demand;
    /// opaque regions use the caller's prepaid construction envelope.
    pub fn conditional_capability_allocation_bound(
        &self,
        rows: &[SemanticId],
    ) -> Result<Option<usize>, MathError> {
        let overflow = || MathError::Limit("conditional capability extent");
        let mut bytes = 0usize;
        for binding in self.structure.instances() {
            if !binding
                .contributions
                .iter()
                .any(|c| matches!(c.target, Target::Row(row) if rows.contains(&row)))
            {
                continue;
            }
            let Some(scratch) =
                self.bodies[&binding.body].arithmetic_analysis_allocation_bound(None)?
            else {
                return Ok(None);
            };
            let metadata = binding
                .slots
                .len()
                .checked_add(binding.contributions.len())
                .and_then(|n| n.checked_mul(11 * size_of::<usize>() + 16 * size_of::<usize>()))
                .ok_or_else(overflow)?;
            bytes = bytes
                .checked_add(scratch)
                .and_then(|n| n.checked_add(metadata))
                .ok_or_else(overflow)?;
        }
        Ok(Some(bytes))
    }
    /// Visit original contributions one at a time, preserving distinct numerical
    /// and conservative execution support. Break refuses further projection
    /// without collecting the rest of a potentially large contribution inventory.
    pub fn visit_dependencies<B>(
        &self,
        cancel: &Arc<AtomicBool>,
        mut visit: impl FnMut(ContributionDependencies) -> std::ops::ControlFlow<B>,
    ) -> Result<std::ops::ControlFlow<B>, MathError> {
        for binding in self.structure.instances() {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            let outputs: Vec<_> = binding
                .contributions
                .iter()
                .map(|c| c.output)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            if outputs.is_empty() {
                continue;
            }
            let body = &self.bodies[&binding.body];
            let coordinates = (0..binding.slots.len()).collect::<Vec<_>>();
            let established = self.supports.iter().find(|support| {
                support.matches_body(body.as_ref())
                    && support.order() >= DerivativeOrder::First
                    && outputs
                        .iter()
                        .all(|output| support.outputs().contains(output))
                    && coordinates
                        .iter()
                        .all(|coordinate| support.coordinates().contains(coordinate))
            });
            let support = if let Some(established) = established {
                established.conditional_support(
                    &outputs,
                    &coordinates,
                    DerivativeOrder::First,
                    cancel,
                )?
            } else {
                body.incidence(&outputs, &coordinates, cancel)?
            };
            for contribution in &binding.contributions {
                if cancel.load(Ordering::Relaxed) {
                    return Err(MathError::Cancelled);
                }
                let numerical: BTreeSet<_> = support
                    .first_for_output(contribution.output)
                    .ok_or_else(|| MathError::Contract("dependency output support absent".into()))?
                    .iter()
                    .map(|slot| binding.slots[*slot].source())
                    .collect();
                let execution = numerical
                    .iter()
                    .copied()
                    .chain(
                        support
                            .support()
                            .controls
                            .iter()
                            .map(|slot| binding.slots[*slot].source()),
                    )
                    .collect();
                let dependency = ContributionDependencies {
                    target: contribution.target,
                    numerical,
                    execution,
                };
                if let std::ops::ControlFlow::Break(value) = visit(dependency) {
                    return Ok(std::ops::ControlFlow::Break(value));
                }
            }
        }
        Ok(std::ops::ControlFlow::Continue(()))
    }
    /// Shared component identities and known payload estimates, excluding body and
    /// support payloads and the shallow plan wrapper. Deduplicate each component
    /// separately when observing live plans of different derivative orders.
    pub fn allocation_components(&self) -> Vec<(&'static str, usize, usize)> {
        let structure = &self.structure;
        let structure_bytes = size_of::<CaseStructure>()
            + structure.selection_bytes()
            + size_of_val(structure.rows())
            + size_of_val(structure.objectives())
            + size_of_val(structure.degradations())
            + size_of_val(structure.requirements())
            + structure
                .instances()
                .iter()
                .map(|i| size_of_val(i) + size_of_val(i.contributions.as_slice()))
                .sum::<usize>();
        let instance_bytes = size_of::<Vec<Instance>>()
            + self.instances.capacity() * size_of::<Instance>()
            + self
                .instances
                .iter()
                .map(|i| {
                    i.coordinates.capacity() * size_of::<Slot>()
                        + i.columns.capacity() * size_of::<GlobalCol>()
                        + i.groups
                            .values()
                            .flatten()
                            .map(|g| g.outputs.capacity() * size_of::<usize>())
                            .sum::<usize>()
                })
                .sum::<usize>();
        let request_bytes = size_of::<Vec<LocalDemand>>()
            + self.requests.capacity() * size_of::<LocalDemand>()
            + self
                .requests
                .iter()
                .map(|r| (r.outputs.capacity() + r.coordinates.capacity()) * size_of::<usize>())
                .sum::<usize>();
        let (coordinate_id, coordinate_bytes) = structure.coordinate_allocation();
        let mut components = vec![
            ("coordinate-universe", coordinate_id, coordinate_bytes),
            (
                "projection-topology",
                Arc::as_ptr(&self.projection) as usize,
                self.projection.retained_bytes(),
            ),
            (
                "structure",
                Arc::as_ptr(&self.structure) as usize,
                structure_bytes,
            ),
            (
                "columns",
                Arc::as_ptr(&self.columns) as usize,
                size_of::<Vec<SemanticId>>() + self.columns.capacity() * size_of::<SemanticId>(),
            ),
            (
                "rows",
                Arc::as_ptr(&self.rows) as usize,
                size_of::<BTreeMap<SemanticId, GlobalRow>>()
                    + self.rows.len() * (size_of::<(SemanticId, GlobalRow)>() + 96),
            ),
            (
                "instances",
                Arc::as_ptr(&self.instances) as usize,
                instance_bytes,
            ),
            (
                "jacobian",
                Arc::as_ptr(&self.jacobian) as usize,
                self.jacobian.retained_bytes(),
            ),
            (
                "hessian",
                Arc::as_ptr(&self.hessian) as usize,
                self.hessian.retained_bytes(),
            ),
            (
                "jacobian_terms",
                Arc::as_ptr(&self.jacobian_terms) as usize,
                size_of::<Vec<Term>>() + self.jacobian_terms.capacity() * size_of::<Term>(),
            ),
            (
                "hessian_terms",
                Arc::as_ptr(&self.hessian_terms) as usize,
                size_of::<Vec<Term>>() + self.hessian_terms.capacity() * size_of::<Term>(),
            ),
            (
                "requests",
                Arc::as_ptr(&self.requests) as usize,
                request_bytes,
            ),
            (
                "supports",
                Arc::as_ptr(&self.supports) as usize,
                size_of::<Vec<Arc<PreparedSupport>>>()
                    + self.supports.capacity() * size_of::<Arc<PreparedSupport>>(),
            ),
        ];
        components.push((
            "row-contributions",
            Arc::as_ptr(&self.row_contributions) as usize,
            size_of_val(self.row_contributions.as_ref())
                + self
                    .row_contributions
                    .values()
                    .map(|indices| {
                        size_of::<SemanticId>()
                            + 96
                            + size_of_val(indices)
                            + indices.capacity() * size_of::<(usize, usize)>()
                    })
                    .sum::<usize>(),
        ));
        for binding in structure.instances() {
            components.push((
                "binding-slots",
                Arc::as_ptr(&binding.slots) as usize,
                size_of::<Vec<crate::binding::SlotBinding>>()
                    + 2 * size_of::<usize>()
                    + binding.slots.capacity() * size_of::<crate::binding::SlotBinding>(),
            ));
            components.push((
                "binding-members",
                Arc::as_ptr(&binding.checked_members) as usize,
                size_of_val(binding.checked_members.as_ref())
                    + 2 * size_of::<usize>()
                    + binding.checked_members.len() * (size_of::<(SemanticId, SemanticId)>() + 96),
            ));
        }
        components
    }
    /// Process-local shared storage identities, independent of owner wrappers and body data.
    pub fn allocation_identity(&self) -> Vec<usize> {
        vec![
            Arc::as_ptr(&self.structure) as usize,
            Arc::as_ptr(&self.columns) as usize,
            Arc::as_ptr(&self.projection) as usize,
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
        Self::retained_group_bytes(std::iter::once(self)).unwrap_or(usize::MAX)
    }
    /// Retained extent of co-owned plans, counting each immutable allocation once.
    pub fn retained_group_bytes<'a>(plans: impl IntoIterator<Item = &'a Self>) -> Option<usize> {
        let mut seen = BTreeSet::new();
        let mut bytes = 0usize;
        for plan in plans {
            bytes = bytes.checked_add(plan.owner_wrapper_bytes())?;
            for (kind, identity, extent) in plan.allocation_components() {
                if seen.insert((kind, identity)) {
                    bytes = bytes.checked_add(extent)?;
                }
            }
            for body in plan.bodies.values() {
                if seen.insert(("body", body.allocation_identity())) {
                    bytes = bytes.checked_add(body.retained_bytes())?;
                }
            }
            for support in plan.supports.iter() {
                if seen.insert(("support", support.allocation_identity())) {
                    bytes = bytes.checked_add(support.retained_bytes())?;
                }
            }
        }
        Some(bytes)
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
        Self::prepare_with_source_support(
            structure, bodies, registry, order, limits, cancel, columns, None,
        )
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "conditional preparation reuses its actual source support under the same case limits"
    )]
    fn prepare_with_source_support(
        structure: Arc<CaseStructure>,
        bodies: BTreeMap<ContentHash, Arc<PreparedBody>>,
        registry: &QuantityRegistry,
        order: DerivativeOrder,
        limits: AssemblyLimits,
        cancel: &Arc<AtomicBool>,
        columns: Vec<SemanticId>,
        source: Option<&Self>,
    ) -> Result<Self, MathError> {
        let _phase = pse_diagnostics::observation::current().map(|context| {
            context.count("math.case_plan_constructions", "count", 1);
            context.phase("math.case_plan_construction")
        });
        if limits.contributions == 0 || limits.native_index == 0 || limits.worker_bytes == 0 {
            return Err(MathError::Limit("zero case assembly budget"));
        }
        if columns.iter().collect::<BTreeSet<_>>().len() != columns.len()
            || columns.iter().any(|c| !structure.contains_coordinate(c))
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
        let mut request_indices = BTreeMap::<_, usize>::new();
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
            for (q, slot) in body.input_quantities().iter().zip(binding.slots.iter()) {
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
                let established = source.and_then(|source| {
                    source
                        .structure
                        .instances()
                        .binary_search_by_key(&binding.instance, |i| i.instance)
                        .ok()
                        .filter(|&index| source.structure.instances()[index].body == binding.body)
                        .and_then(|index| {
                            source.instances[index]
                                .groups
                                .values()
                                .flatten()
                                .map(|group| &source.supports[group.request])
                                .find(|support| {
                                    support.order() >= order
                                        && all_outputs
                                            .iter()
                                            .all(|output| support.outputs().contains(output))
                                        && formal.iter().all(|coordinate| {
                                            support.coordinates().contains(coordinate)
                                        })
                                })
                        })
                });
                let support = if let Some(established) = established {
                    if let Some(context) = pse_diagnostics::observation::current() {
                        context.count("math.source_support_reuses", "count", 1);
                    }
                    established.conditional_support(&all_outputs, &formal, order, cancel)?
                } else if order == DerivativeOrder::First {
                    body.incidence(&all_outputs, &formal, cancel)?
                } else {
                    body.prepare_support(&all_outputs, &formal, order, cancel)?
                };
                shared_supports.insert(shared_key.clone(), support);
            }
            let mut groups = EnumMap::<EvaluationDemand, Option<Group>>::default();
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
                let ceiling = if demand == EvaluationDemand::All {
                    order
                } else {
                    order.min(DerivativeOrder::First)
                };
                let demand = LocalDemand {
                    body: binding.body,
                    outputs: outputs.clone(),
                    coordinates: formal.clone(),
                    order: ceiling,
                    directional: false,
                };
                let key = (
                    demand.body,
                    demand.outputs.clone(),
                    demand.coordinates.clone(),
                );
                let request = if let Some(&request) = request_indices.get(&key) {
                    if requests[request].order < demand.order {
                        requests[request].order = demand.order;
                        let shared = shared_supports.get(&shared_key).ok_or_else(|| {
                            MathError::Contract("missing selected support".into())
                        })?;
                        let support = if demand.outputs == all_outputs {
                            shared.clone()
                        } else {
                            shared.select_outputs(&demand.outputs, cancel)?
                        };
                        supports[request] = Arc::new(support.at_order(demand.order)?);
                    }
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
                    supports.push(Arc::new(support.at_order(ceiling)?));
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
        let mut row_contributions = BTreeMap::<_, Vec<(usize, usize)>>::new();
        for (index, binding) in structure.instances().iter().enumerate() {
            for (output, contribution) in binding.contributions.iter().enumerate() {
                if let Target::Row(row) = contribution.target {
                    row_contributions
                        .entry(row)
                        .or_default()
                        .push((index, output));
                }
            }
        }
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("affine.setup_columns", "count", columns.len() as u64);
            context.count(
                "affine.setup_contributions",
                "count",
                row_contributions.values().map(Vec::len).sum::<usize>() as u64,
            );
        }
        Ok(Self {
            projection: Arc::new(crate::factorable::ProjectionTopology::new(
                &structure, &columns,
            )?),
            row_contributions: Arc::new(row_contributions),
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
        Self::prepare_functions(
            &self.structure,
            |body| self.bodies.get(&body).cloned(),
            selected_rows,
            coordinates,
            registry,
            order,
            self.limits,
            cancel,
        )
    }
    /// Prepare selected function rows directly from admitted structure. Only bodies
    /// contributing to selected rows are acquired; their complete guard and provider
    /// dependencies remain intact. The original coordinate universe is shared.
    ///
    /// Rows retain source order and derivative coordinates retain caller order.
    /// Objectives, native forms and solver requirements are excluded, as in
    /// [`Self::functions`].
    ///
    /// # Errors
    /// Unknown or repeated rows/coordinates, missing selected bodies, or ordinary
    /// preparation failures.
    #[expect(
        clippy::too_many_arguments,
        reason = "selected preparation consumes source, body authority and explicit assembly policy"
    )]
    pub fn prepare_functions(
        source: &CaseStructure,
        mut body: impl FnMut(ContentHash) -> Option<Arc<PreparedBody>>,
        selected_rows: &[SemanticId],
        coordinates: Vec<SemanticId>,
        registry: &QuantityRegistry,
        order: DerivativeOrder,
        limits: AssemblyLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let selected: BTreeSet<_> = selected_rows.iter().copied().collect();
        if selected.len() != selected_rows.len()
            || selected
                .iter()
                .any(|row| source.rows().binary_search_by_key(row, |r| r.id).is_err())
        {
            return Err(MathError::Contract(
                "unknown or repeated function output".into(),
            ));
        }
        // Refuse invalid coordinate requests before acquiring any body products.
        if coordinates.iter().collect::<BTreeSet<_>>().len() != coordinates.len()
            || coordinates.iter().any(|c| !source.contains_coordinate(c))
        {
            return Err(MathError::Contract(
                "unknown or duplicate derivative coordinate".into(),
            ));
        }
        let mut instances = Vec::new();
        for binding in source.instances() {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            let contributions: Vec<_> = binding
                .contributions
                .iter()
                .filter(|c| matches!(c.target, Target::Row(r) if selected.contains(&r)))
                .cloned()
                .collect();
            if !contributions.is_empty() {
                instances.push(crate::binding::InstanceBinding {
                    instance: binding.instance,
                    body: binding.body,
                    slots: binding.slots.clone(),
                    checked_members: binding.checked_members.clone(),
                    contributions,
                });
            }
        }
        let mut bodies = BTreeMap::new();
        for binding in &instances {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            if let std::collections::btree_map::Entry::Vacant(entry) = bodies.entry(binding.body) {
                entry.insert(
                    body(binding.body)
                        .ok_or_else(|| MathError::Contract("missing prepared body".into()))?,
                );
            }
        }
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.selected_function_constructions", "count", 1);
            context.count(
                "math.selected_function_bodies",
                "count",
                bodies.len() as u64,
            );
            context.count(
                "math.selected_function_rows",
                "count",
                selected.len() as u64,
            );
        }
        let structure = Arc::new(
            source.restricted(
                instances,
                source
                    .rows()
                    .iter()
                    .filter(|r| selected.contains(&r.id))
                    .cloned()
                    .collect(),
                None,
            ),
        );
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
            self.limits,
            cancel,
            coordinates,
        )
    }
    /// Exact derivative capability of selected authored rows and formal coordinates,
    /// including provider/transitive output demand. No conditional plan, sparse layout,
    /// evaluator request or native program is constructed by this query.
    pub fn conditional_available_order(
        &self,
        rows: &BTreeSet<SemanticId>,
        columns: &BTreeSet<SemanticId>,
        cancel: &Arc<AtomicBool>,
    ) -> Result<DerivativeOrder, MathError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        if rows.is_empty()
            || rows.len() != columns.len()
            || rows.iter().any(|row| !self.rows.contains_key(row))
            || columns
                .iter()
                .any(|column| self.columns.binary_search(column).is_err())
        {
            return Err(MathError::Contract(
                "conditional capability inventory".into(),
            ));
        }
        let mut available = DerivativeOrder::Second;
        for binding in self.structure.instances() {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            let outputs: Vec<_> = binding
                .contributions
                .iter()
                .filter_map(|contribution| {
                    matches!(contribution.target, Target::Row(row) if rows.contains(&row))
                        .then_some(contribution.output)
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            if outputs.is_empty() {
                continue;
            }
            let coordinates: Vec<_> = binding
                .slots
                .iter()
                .enumerate()
                .filter_map(|(index, slot)| columns.contains(&slot.source()).then_some(index))
                .collect();
            available = available.min(
                self.bodies[&binding.body].available_order_for_outputs(&outputs, &coordinates)?,
            );
        }
        Ok(available)
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
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        if selected_rows.is_empty()
            || selected_rows.len() != selected_columns.len()
            || selected_rows.iter().any(|r| !self.rows.contains_key(r))
            || selected_columns
                .iter()
                .any(|c| self.columns.binary_search(c).is_err())
        {
            return Err(MathError::Contract("conditional block inventory".into()));
        }
        let rows = self
            .structure
            .rows()
            .iter()
            .filter(|r| selected_rows.contains(&r.id))
            .cloned()
            .collect();
        let binding_indices: BTreeSet<_> = selected_rows
            .iter()
            .filter_map(|row| self.row_contributions.get(row))
            .flatten()
            .map(|&(binding, _)| binding)
            .collect();
        let instances: Vec<_> = binding_indices
            .into_iter()
            .map(|index| {
                let binding = &self.structure.instances()[index];
                crate::binding::InstanceBinding {
                    instance: binding.instance,
                    body: binding.body,
                    slots: binding.slots.clone(),
                    checked_members: binding.checked_members.clone(),
                    contributions: binding
                        .contributions
                        .iter()
                        .filter(|c| matches!(c.target,Target::Row(r)if selected_rows.contains(&r)))
                        .cloned()
                        .collect(),
                }
            })
            .collect();
        let bodies = instances
            .iter()
            .map(|i| (i.body, self.bodies[&i.body].clone()))
            .collect();
        let structure = self
            .structure
            .restricted(instances, rows, Some(selected_columns));
        let columns = structure.free_variables().collect();
        Self::prepare_with_source_support(
            Arc::new(structure),
            bodies,
            registry,
            self.order.max(DerivativeOrder::First),
            self.limits,
            cancel,
            columns,
            Some(self),
        )
    }
    /// Append first directional constraint actions after a consumer requests them.
    /// Existing demands, support products and artifact ordinals retain their identity.
    pub fn with_directional_actions(&self, cancel: &Arc<AtomicBool>) -> Result<Self, MathError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let mut instances = self.instances.as_ref().clone();
        let mut requests = self.requests.as_ref().clone();
        let mut supports = self.supports.as_ref().clone();
        let mut indices = BTreeMap::new();
        for (index, request) in requests.iter().enumerate().filter(|(_, r)| r.directional) {
            indices.insert(
                (
                    request.body,
                    request.outputs.clone(),
                    request.coordinates.clone(),
                ),
                index,
            );
        }
        for instance in &mut instances {
            if instance.groups[EvaluationDemand::Directional].is_some() {
                continue;
            }
            let Some(group) = instance.groups[EvaluationDemand::Constraints].as_ref() else {
                continue;
            };
            let original = &requests[group.request];
            let key = (
                original.body,
                group.outputs.clone(),
                original.coordinates.clone(),
            );
            let request = if let Some(&index) = indices.get(&key) {
                index
            } else {
                let support = self.bodies[&original.body].prepare_support(
                    &group.outputs,
                    &original.coordinates,
                    DerivativeOrder::First,
                    cancel,
                )?;
                let index = requests.len();
                requests.push(LocalDemand {
                    body: original.body,
                    outputs: group.outputs.clone(),
                    coordinates: original.coordinates.clone(),
                    order: DerivativeOrder::First,
                    directional: true,
                });
                supports.push(Arc::new(support));
                indices.insert(key, index);
                index
            };
            instance.groups[EvaluationDemand::Directional] = Some(Group {
                outputs: group.outputs.clone(),
                request,
            });
        }
        let mut result = self.clone();
        result.instances = Arc::new(instances);
        result.requests = Arc::new(requests);
        result.supports = Arc::new(supports);
        Ok(result)
    }
    /// Whether independently demanded directional constraint actions are prepared.
    pub fn has_directional_actions(&self) -> bool {
        self.requests.iter().any(|r| r.directional)
    }
    /// Construction extent of immutable support/order or directional-demand
    /// upgrades. Numerical evaluators are deferred; this bounds selected support,
    /// instance/demand copies and actual possible sparse derivative contributions.
    /// Unknown control/provider support retains a conservative caller-owned entry.
    /// # Errors
    /// Overflow before constructing any support or sparse population.
    pub fn support_upgrade_allocation_bound(
        &self,
        evaluation: EvaluationLimits,
        order: DerivativeOrder,
    ) -> Result<Option<usize>, MathError> {
        let overflow = || MathError::Limit("support upgrade construction extent");
        let add = |a: usize, b: usize| a.checked_add(b).ok_or_else(overflow);
        let mul = |a: usize, b: usize| a.checked_mul(b).ok_or_else(overflow);
        let mut support_bytes = 0;
        for (support, request) in self.supports.iter().zip(self.requests.iter()) {
            let requested = if request.directional {
                DerivativeOrder::First
            } else {
                order
            };
            let Some(bytes) = support.support_upgrade_allocation_bound(evaluation, requested)?
            else {
                return Ok(None);
            };
            support_bytes = add(support_bytes, mul(bytes, 2)?)?;
        }
        let mut dimensions = 0;
        let mut copies = 0;
        let mut contributions = 0;
        for (instance, binding) in self.instances.iter().zip(self.structure.instances()) {
            let n = instance.coordinates.len();
            dimensions = add(dimensions, n)?;
            copies = add(copies, mul(n, size_of::<Slot>() + size_of::<GlobalCol>())?)?;
            for group in instance.groups.values().flatten() {
                dimensions = add(dimensions, group.outputs.len())?;
            }
            contributions = add(
                contributions,
                mul(binding.contributions.len(), add(n, mul(n, n)?)?)?,
            )?;
        }
        for demand in self.requests.iter() {
            dimensions = add(
                dimensions,
                add(demand.outputs.len(), demand.coordinates.len())?,
            )?;
        }
        let count = add(self.requests.len(), self.instances.len())?;
        let descriptors = add(
            mul(
                add(mul(count, 2)?, 4)?,
                size_of::<LocalDemand>() + size_of::<Arc<PreparedSupport>>(),
            )?,
            add(
                mul(self.instances.len(), size_of::<Instance>())?,
                add(copies, mul(dimensions, 4 * size_of::<usize>())?)?,
            )?,
        )?;
        let maps = mul(
            count,
            11 * (size_of::<(ContentHash, Vec<usize>, Vec<usize>, usize)>()
                + size_of::<(usize, Arc<PreparedSupport>)>())
                + 32 * size_of::<usize>(),
        )?;
        // case_patterns checks the combined contribution ceiling BEFORE either
        // push. Include Vec growth plus faer's indices/argsort/CSC/refill and zero
        // buffers, preserving duplicate contributions and aliased coordinates.
        let terms = contributions.min(self.limits.contributions);
        let sparse = add(
            mul(
                terms,
                2 * size_of::<Entry<GlobalCol, GlobalCol>>()
                    + 2 * size_of::<Term>()
                    + size_of::<faer::sparse::Pair<usize, usize>>()
                    + 4 * size_of::<usize>()
                    + 2 * size_of::<f64>(),
            )?,
            add(
                8 * (size_of::<Entry<GlobalCol, GlobalCol>>() + size_of::<Term>())
                    + 4 * size_of::<usize>(),
                mul(add(self.columns.len(), 1)?, 2 * size_of::<usize>())?,
            )?,
        )?;
        let bodies = mul(
            self.bodies.len(),
            11 * size_of::<(ContentHash, Arc<PreparedBody>)>() + 16 * size_of::<usize>(),
        )?;
        Ok(Some(add(
            add(add(add(support_bytes, descriptors)?, maps)?, sparse)?,
            add(bodies, size_of::<Self>())?,
        )?))
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
            if let Some(group) = &instance.groups[EvaluationDemand::All]
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
            if self.requests[request].directional {
                supports.push(previous.clone());
                continue;
            }
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
                        .then(|| {
                            instance.groups[EvaluationDemand::All]
                                .as_ref()
                                .map(|all| all.request)
                        })
                        .flatten()
                })
                .ok_or_else(|| {
                    MathError::Contract("support demand has no instance owner".into())
                })?;
            supports.push(Arc::new(
                stronger[&parent]
                    .select_outputs(previous.outputs(), cancel)?
                    .at_order(order.min(DerivativeOrder::First))?,
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
                    if !request.directional {
                        let is_all = self.instances.iter().any(|instance| {
                            instance.groups[EvaluationDemand::All]
                                .as_ref()
                                .is_some_and(|group| {
                                    self.requests[group.request].body == request.body
                                        && self.requests[group.request].outputs == request.outputs
                                        && self.requests[group.request].coordinates
                                            == request.coordinates
                                })
                        });
                        request.order = if is_all {
                            order
                        } else {
                            order.min(DerivativeOrder::First)
                        };
                    }
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
    /// Verify one genuine derivative projection against frozen source mathematics and
    /// quantities, excluding only the admitted derivative demand/order. Actual bodies,
    /// guards, physical bounds/maps and symbolic sparse terms must all match.
    /// # Errors
    /// Cancellation, unavailable support or the source's existing construction allowance.
    pub fn is_derivative_projection_of(
        &self,
        original: &Self,
        quantities: &QuantityRegistry,
        cancel: &Arc<AtomicBool>,
    ) -> Result<bool, MathError> {
        if self.structure != original.structure
            || self.columns != original.columns
            || self.bodies != original.bodies
            || self.requests.iter().any(|d| d.directional)
        {
            return Ok(false);
        }
        let expected = Self::prepare_with_coordinates(
            original.structure.clone(),
            original.bodies.clone(),
            quantities,
            self.order,
            original.limits,
            cancel,
            original.columns.as_ref().clone(),
        )?;
        let same_terms = |left: &[Term], right: &[Term]| {
            left.len() == right.len()
                && left.iter().zip(right).all(|(a, b)| {
                    a.instance == b.instance
                        && a.output == b.output
                        && a.i == b.i
                        && a.j == b.j
                        && a.scale.to_bits() == b.scale.to_bits()
                        && a.target == b.target
                })
        };
        Ok(self.structure == expected.structure
            && self.columns == expected.columns
            && self.bodies == expected.bodies
            && self.requests == expected.requests
            && self.supports == expected.supports
            && same_terms(&self.jacobian_terms, &expected.jacobian_terms)
            && same_terms(&self.hessian_terms, &expected.hessian_terms))
    }
    /// Consumed immutable semantic bodies.
    pub fn bodies(&self) -> &BTreeMap<ContentHash, Arc<PreparedBody>> {
        &self.bodies
    }
    /// Caller-issued per-case admission policy, preserved by derivative and function projections.
    pub fn limits(&self) -> AssemblyLimits {
        self.limits
    }
    /// Stable free-variable order.
    pub fn columns(&self) -> &[SemanticId] {
        &self.columns
    }
    pub(crate) fn projection_topology(&self) -> &crate::factorable::ProjectionTopology {
        &self.projection
    }
    /// Original contribution order, indexed once rather than rediscovered for each row proof.
    pub(crate) fn row_contributions(&self, row: SemanticId) -> &[(usize, usize)] {
        self.row_contributions.get(&row).map_or(&[], Vec::as_slice)
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
    /// Each product retains its selected output map and original free-variable formal
    /// coordinates, independently of the numerical derivative column selection.
    pub fn incidence(
        &self,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Vec<Arc<PreparedSupport>>, MathError> {
        let mut products = Vec::with_capacity(self.instances.len());
        let mut shared = BTreeMap::new();
        let free = self.structure.free_variables().collect::<BTreeSet<_>>();
        for (binding, instance) in self.structure.instances().iter().zip(self.instances.iter()) {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            // Structural incidence covers every original contribution, including
            // later lexicographic objectives that have no primary numerical group.
            let outputs = binding
                .contributions
                .iter()
                .map(|contribution| contribution.output)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let ready = instance.groups[EvaluationDemand::All]
                .as_ref()
                .map(|group| &self.supports[group.request])
                .filter(|ready| ready.outputs() == outputs);
            let coordinates = binding
                .slots
                .iter()
                .enumerate()
                .filter_map(|(slot, binding)| free.contains(&binding.source()).then_some(slot))
                .collect::<Vec<_>>();
            let key = (binding.body, outputs.clone(), coordinates.clone());
            let product = if let Some(product) = shared.get(&key) {
                Arc::clone(product)
            } else {
                let product = match ready {
                    Some(ready)
                        if ready.coordinates() == coordinates
                            && ready.order() >= DerivativeOrder::First =>
                    {
                        Arc::clone(ready)
                    }
                    Some(ready) => Arc::new(ready.incidence(&coordinates, cancel)?),
                    None => Arc::new(self.bodies[&binding.body].incidence(
                        &outputs,
                        &coordinates,
                        cancel,
                    )?),
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
        let group = self.instances.get(instance)?.groups[EvaluationDemand::All].as_ref()?;
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
    /// Preconstruction extent of operation gathers, their wrapper and temporary
    /// source-offset maps. Shared plans, physical slots and programs retain their owners.
    pub fn operation_layout_allocation_bound(&self) -> Result<usize, MathError> {
        let overflow = || MathError::Limit("operation binding layout extent");
        let scalars = self
            .structure
            .variables()
            .len()
            .checked_add(self.structure.parameters().len())
            .ok_or_else(overflow)?;
        let mut bytes = scalars
            .checked_mul(256 + 2 * size_of::<SemanticId>())
            .and_then(|bytes| bytes.checked_add(size_of::<CaseAssembly>() + 4096))
            .and_then(|bytes| {
                bytes.checked_add(self.instances.len().checked_mul(
                    2 * size_of::<EnumMap<EvaluationDemand, Option<EvalBindings>>>(),
                )?)
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    self.requests
                        .len()
                        .checked_mul(2 * size_of::<Arc<CompiledBody>>())?,
                )
            })
            .ok_or_else(overflow)?;
        for (instance, binding) in self.instances.iter().zip(self.structure.instances()) {
            for _ in instance.groups.values().flatten() {
                bytes = bytes
                    .checked_add(
                        EvalBindings::allocation_bound(binding.slots.len()).ok_or_else(overflow)?,
                    )
                    .ok_or_else(overflow)?;
            }
        }
        Ok(bytes)
    }
    /// Bind completed artifacts in the exact compiler demand order.
    pub fn assemble(
        self: &Arc<Self>,
        programs: Vec<Arc<CompiledBody>>,
    ) -> Result<CaseAssembly, MathError> {
        let _phase = pse_diagnostics::observation::current().map(|context| {
            context.count("math.assembly_binds", "count", 1);
            context.phase("math.assembly_bind")
        });
        if programs.len() != self.requests.len() {
            return Err(MathError::Contract("artifact demand count".into()));
        }
        for (index, (request, program)) in self.requests.iter().zip(&programs).enumerate() {
            let support = program.prepared_support();
            if support.outputs() != request.outputs
                || support.coordinates() != request.coordinates
                || program.compiled_order() != request.order
                || program.is_directional() != request.directional
                // Cached compact programs can outlive their weak original body.
                // Compare the complete selected mathematics admitted by this plan;
                // its original bodies remain authoritative for independent checks.
                || support != self.supports[index].as_ref()
            {
                return Err(MathError::Contract(
                    "artifact does not match selected admitted demand".into(),
                ));
            }
        }
        let coordinates: BTreeMap<_, _> = self
            .columns
            .iter()
            .copied()
            .enumerate()
            .map(|(index, id)| (id, index))
            .collect();
        // Only actual compiled formal reads occupy the fixed frame. The complete source
        // universe remains immutable authority for independent original assessment.
        let bound_programs = &programs;
        let fixed: Vec<_> = self
            .instances
            .iter()
            .zip(self.structure.instances())
            .flat_map(|(instance, binding)| {
                instance.groups.values().flatten().flat_map(move |group| {
                    bound_programs[group.request]
                        .input_formals()
                        .iter()
                        .map(move |formal| binding.slots[*formal].source())
                })
            })
            .filter(|id| !coordinates.contains_key(id))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let fixed_offsets = fixed
            .iter()
            .copied()
            .enumerate()
            .map(|(index, id)| (id, index))
            .collect();
        let bindings = self
            .instances
            .iter()
            .zip(self.structure.instances())
            .map(|(instance, binding)| {
                let mut groups = EnumMap::default();
                for (demand, group) in &instance.groups {
                    if let Some(group) = group {
                        groups[demand] = Some(EvalBindings::new(
                            binding,
                            programs[group.request].input_formals(),
                            &coordinates,
                            &fixed_offsets,
                        )?);
                    }
                }
                Ok(groups)
            })
            .collect::<Result<Vec<EnumMap<EvaluationDemand, Option<EvalBindings>>>, MathError>>()?;
        let input_bytes = bindings
            .iter()
            .flat_map(|groups| groups.values().flatten())
            .try_fold(0usize, |bytes, binding| {
                bytes.checked_add(
                    binding
                        .len()
                        .checked_mul(size_of::<f64>() + size_of::<u64>())?,
                )
            })
            .and_then(|bytes| {
                self.instances.iter().try_fold(bytes, |bytes, instance| {
                    if instance.groups[EvaluationDemand::Directional].is_some() {
                        bytes.checked_add(
                            instance
                                .coordinates
                                .len()
                                .checked_mul(size_of::<f64>() + size_of::<u64>())?,
                        )
                    } else {
                        Some(bytes)
                    }
                })
            })
            .and_then(|bytes| bytes.checked_add(fixed.len().checked_mul(size_of::<f64>())?))
            .ok_or(MathError::Limit("case input buffers"))?;
        let scratch = programs
            .iter()
            .try_fold(0usize, |n, p| n.checked_add(p.worker_bytes()))
            .ok_or(MathError::Limit("case worker bytes"))?;
        let bytes = self
            .instances
            .iter()
            .flat_map(|i| i.groups.values().flatten())
            .try_fold(
                scratch
                    .checked_add(input_bytes)
                    .ok_or(MathError::Limit("case worker bytes"))?,
                |n, g| n.checked_add(programs[g.request].evaluation_cache_bytes()),
            )
            .and_then(|n| n.checked_add(self.jacobian.retained_bytes()))
            .and_then(|n| n.checked_add(self.hessian.retained_bytes()))
            .and_then(|n| n.checked_add((self.rows.len() + self.columns.len()).checked_mul(size_of::<f64>())?))
            .and_then(|n| {
                n.checked_add(
                    self.instances
                        .len()
                        .checked_mul(size_of::<EnumMap<EvaluationDemand, Option<GroupCache>>>())?,
                )
            })
            .and_then(|n| n.checked_add(programs.len().checked_mul(size_of::<Worker>())?))
            .and_then(|n| {
                if self.has_directional_actions() {
                    let formals = self
                        .structure
                        .instances()
                        .iter()
                        .map(|i| i.slots.len())
                        .max()
                        .unwrap_or(0);
                    n.checked_add(
                        self.rows
                            .len()
                            .checked_add(formals.checked_mul(4)?)?
                            .checked_mul(size_of::<f64>())?,
                    )
                } else {
                    Some(n)
                }
            })
            .ok_or(MathError::Limit("case worker bytes"))?;
        if bytes > self.limits.worker_bytes {
            return Err(MathError::Limit("case worker bytes"));
        }
        Ok(CaseAssembly {
            plan: Arc::clone(self),
            programs,
            numeric_worker_bytes: bytes,
            bindings,
            fixed,
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
            .zip(self.requests.iter())
            .map(|(support, request)| {
                if request.directional {
                    support
                        .compile_directional(optimization, limits, cancel)
                        .map(Arc::new)
                } else {
                    support.compile(optimization, limits, cancel).map(Arc::new)
                }
            })
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
    let mut support_visits = 0u64;
    for (index, (binding, instance)) in structure.instances().iter().zip(instances).enumerate() {
        let support = instance.groups[EvaluationDemand::All]
            .as_ref()
            .map(|g| &supports[g.request]);
        let offsets = instance.coordinates.iter().enumerate()
            .map(|(i, formal)| (formal.get(), i)).collect::<BTreeMap<_, _>>();
        for contribution in &binding.contributions {
            if order >= DerivativeOrder::First
                && let Target::Row(row) = contribution.target
            {
                let first = support
                    .and_then(|s| s.first_for_output(contribution.output))
                    .ok_or_else(|| {
                        MathError::Contract("missing selected First incidence".into())
                    })?;
                let result_row = instance.groups[EvaluationDemand::Constraints].as_ref()
                    .and_then(|group| group.outputs.binary_search(&contribution.output).ok())
                    .ok_or_else(|| MathError::Contract("missing prepared constraint output".into()))?;
                for formal in first {
                    support_visits = support_visits.checked_add(1).ok_or(MathError::Limit("support contribution count"))?;
                    let Some(&i) = offsets.get(formal) else { continue; };
                    contribution_allowance(jp.len(), hp.len(), limits)?;
                    jp.push(Entry::new(rows[&row], instance.columns[i]));
                    jt.push(Term {
                        instance: index, output: contribution.output, result_row, i, j: 0,
                        scale: contribution.scale * binding.slots[*formal].scale(),
                        target: contribution.target,
                    });
                }
            }
            if order >= DerivativeOrder::Second
                && selected(contribution.target, EvaluationDemand::All)
            {
                let second = support
                    .and_then(|s| s.second_for_output(contribution.output))
                    .ok_or_else(|| MathError::Contract("missing selected Second support".into()))?;
                let result_row = instance.groups[EvaluationDemand::All].as_ref()
                    .and_then(|group| group.outputs.binary_search(&contribution.output).ok())
                    .ok_or_else(|| MathError::Contract("missing prepared Hessian output".into()))?;
                let mut ordered = Vec::new();
                for &(a, b) in second {
                    support_visits = support_visits.checked_add(1).ok_or(MathError::Limit("support contribution count"))?;
                    let (Some(&i), Some(&j)) = (offsets.get(&a), offsets.get(&b)) else { continue; };
                    if instance.columns[i] >= instance.columns[j] { ordered.push((i, j)); }
                    // Distinct local axes can alias the same global diagonal. Both
                    // ordered cross terms contribute, even though support is symmetric.
                    if i != j && instance.columns[j] >= instance.columns[i] { ordered.push((j, i)); }
                }
                ordered.sort_unstable();
                for (i, j) in ordered {
                    contribution_allowance(jp.len(), hp.len(), limits)?;
                    hp.push(Entry::new(instance.columns[i], instance.columns[j]));
                    ht.push(Term {
                        instance: index, output: contribution.output, result_row, i, j,
                        scale: contribution.scale
                            * binding.slots[instance.coordinates[i].get()].scale()
                            * binding.slots[instance.coordinates[j].get()].scale(),
                        target: contribution.target,
                    });
                }
            }
        }
    }
    if let Some(context) = pse_diagnostics::observation::current() {
        context.count("math.assembly.support_candidates", "support-entries", support_visits);
        context.count("math.assembly.emitted_addends", "addends", (jp.len() + hp.len()) as u64);
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
    /// One runtime directional Taylor axis, distinct from assembled partials.
    pub directional: bool,
}
/// A structural plan bound to immutable numeric programs.
#[derive(Clone, Debug)]
pub struct CaseAssembly {
    plan: Arc<CasePlan>,
    programs: Vec<Arc<CompiledBody>>,
    numeric_worker_bytes: usize,
    bindings: Vec<EnumMap<EvaluationDemand, Option<EvalBindings>>>,
    fixed: Vec<SemanticId>,
}

/// Frozen numeric inputs, rebound independently of an immutable operation layout.
#[derive(Debug)]
pub struct BoundValues {
    assembly: Arc<CaseAssembly>,
    fixed: Vec<f64>,
}
impl BoundValues {
    /// Refill this operation's fixed frame through a precompiled dense read map.
    /// Validation completes before any retained input changes; repeated source slots
    /// remain legal aliases of distinct declared inputs.
    pub fn refill_mapped(&mut self, source: &[f64], reads: &[usize]) -> Result<(), MathError> {
        if reads.len() != self.fixed.len() {
            return Err(MathError::Contract("compact read-map extent".into()));
        }
        for (id, &read) in self.assembly.fixed.iter().zip(reads) {
            let value = source
                .get(read)
                .copied()
                .ok_or_else(|| MathError::Contract("compact read-map source".into()))?;
            self.assembly.structure.validate_scalar(*id, value)?;
        }
        for (value, &read) in self.fixed.iter_mut().zip(reads) {
            *value = source[read];
        }
        Ok(())
    }
    /// Exact semantic inputs retained in this operation's compact fixed frame.
    pub fn input_ids(&self) -> &[SemanticId] {
        &self.assembly.fixed
    }
    /// Atomically refill declared changed inputs in an existing compact frame. Unread
    /// declared inputs have no frame slot; no complete semantic map is constructed.
    pub fn refill(&mut self, updates: &[(SemanticId, f64)]) -> Result<(), MathError> {
        for (index, (id, value)) in updates.iter().enumerate() {
            if updates[..index].iter().any(|(previous, _)| previous == id) {
                return Err(MathError::Contract("duplicate compact input update".into()));
            }
            self.assembly.structure.validate_scalar(*id, *value)?;
        }
        for (id, value) in updates {
            if let Ok(index) = self.assembly.fixed.binary_search(id) {
                self.fixed[index] = *value;
            }
        }
        Ok(())
    }
    /// Bit-exact consumed fixed input identity for native numeric compatibility only.
    pub fn input_identity(&self) -> ContentHash {
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::MathCaseValuesV1);
        hash.str("compact-native-inputs");
        for (id, value) in self.assembly.fixed.iter().zip(&self.fixed) {
            hash.id(id).u64(value.to_bits());
        }
        hash.finish_hash()
    }
    /// Borrow a finite native trial in the assembly's declared coordinate order.
    pub fn trial<'a>(&'a self, coordinates: &'a [f64]) -> Result<EvalInputs<'a>, MathError> {
        if coordinates.len() != self.assembly.columns.len()
            || coordinates.iter().any(|v| !v.is_finite())
        {
            return Err(MathError::Contract("finite trial coordinate extent".into()));
        }
        Ok(EvalInputs(InputSource::Trial(self, coordinates)))
    }
}
/// Inputs to an admitted operation: semantic original assessment or dense native trial.
#[derive(Clone, Copy, Debug)]
pub struct EvalInputs<'a>(InputSource<'a>);
#[derive(Clone, Copy, Debug)]
enum InputSource<'a> {
    Case(&'a CaseValues),
    Trial(&'a BoundValues, &'a [f64]),
}
impl<'a> From<&'a CaseValues> for EvalInputs<'a> {
    fn from(values: &'a CaseValues) -> Self {
        Self(InputSource::Case(values))
    }
}
impl EvalInputs<'_> {
    fn check(&self, assembly: &CaseAssembly) -> Result<(), MathError> {
        if let InputSource::Trial(bound, _) = self.0
            && !std::ptr::eq(assembly, bound.assembly.as_ref())
        {
            return Err(MathError::Contract(
                "trial belongs to another operation layout".into(),
            ));
        }
        Ok(())
    }
    fn fill(&self, bindings: &EvalBindings, output: &mut [f64]) -> Result<(), MathError> {
        match self.0 {
            InputSource::Case(values) => bindings.fill(Some(values), &[], &[], output),
            InputSource::Trial(bound, coordinates) => {
                bindings.fill(None, coordinates, &bound.fixed, output)
            }
        }
    }
}
impl std::ops::Deref for CaseAssembly {
    type Target = CasePlan;
    fn deref(&self) -> &Self::Target {
        &self.plan
    }
}
/// Borrowed compact description of one actual prepared template. Supports and
/// selections stay with their immutable owners rather than being copied for telemetry.
#[derive(Clone, Copy, Debug)]
pub struct PreparedDemandCensus<'a> {
    /// Index in this assembly's compiler-issued template order.
    pub template: usize,
    /// Actual immutable demand, including body and ordered selections.
    pub demand: &'a LocalDemand,
    /// Actual selected support, independent of numerical zeros.
    pub support: &'a PreparedSupport,
    /// Actual compiled programs and their compact per-order census.
    pub program: &'a CompiledBody,
}
/// Compact case populations; candidate coordinate pairs are not inferred from nnz.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaseCensus {
    /// Authored occurrences bound to the case.
    pub occurrences: usize,
    /// Deduplicated compiled local templates.
    pub templates: usize,
    /// Selected physical solver columns.
    pub columns: usize,
    /// Selected physical solver rows.
    pub rows: usize,
    /// Emitted ordered Jacobian addends, before duplicate accumulation.
    pub jacobian_addends: usize,
    /// Emitted ordered Hessian addends, preserving aliased cross terms.
    pub hessian_addends: usize,
    /// Canonical stored Jacobian nonzeros.
    pub jacobian_nnz: usize,
    /// Canonical stored Hessian nonzeros.
    pub hessian_nnz: usize,
    /// Admitted complete attempt storage, distinct from current instantiated bytes.
    pub numeric_worker_bytes: usize,
}
impl CaseAssembly {
    /// Counts from actual immutable owners, with no support enumeration or copy.
    pub fn census(&self) -> CaseCensus {
        CaseCensus {
            occurrences: self.instances.len(),
            templates: self.programs.len(),
            columns: self.columns.len(),
            rows: self.rows.len(),
            jacobian_addends: self.jacobian_terms.len(),
            hessian_addends: self.hessian_terms.len(),
            jacobian_nnz: self.jacobian.matrix().val().len(),
            hessian_nnz: self.hessian.matrix().val().len(),
            numeric_worker_bytes: self.numeric_worker_bytes,
        }
    }
    /// Borrow prepared template facts without constructing another support population.
    pub fn demand_census(&self) -> impl Iterator<Item = PreparedDemandCensus<'_>> {
        self.requests
            .iter()
            .zip(self.supports.iter())
            .zip(&self.programs)
            .enumerate()
            .map(
                |(template, ((demand, support), program))| PreparedDemandCensus {
                    template,
                    demand,
                    support,
                    program,
                },
            )
    }
    /// Exact fixed/parameter reads of the admitted compiled programs.
    pub fn input_ids(&self) -> &[SemanticId] {
        &self.fixed
    }
    /// Bind only inputs actually read by this operation. The caller's source universe
    /// is already admitted; each consumed scalar is independently checked here.
    pub fn bind_inputs(
        self: &Arc<Self>,
        value: impl Fn(SemanticId) -> Option<f64>,
    ) -> Result<BoundValues, MathError> {
        let _phase = pse_diagnostics::observation::current().map(|context| {
            context.count("math.input_binds", "count", 1);
            context.phase("math.input_bind")
        });
        let fixed = self
            .fixed
            .iter()
            .map(|id| {
                let value = value(*id)
                    .ok_or_else(|| MathError::Contract("missing compact case input".into()))?;
                self.structure.validate_scalar(*id, value)?;
                Ok(value)
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        Ok(BoundValues {
            assembly: self.clone(),
            fixed,
        })
    }
    /// Validate a rebind once and retain only its fixed and parameter coordinates.
    pub fn bind_values(self: &Arc<Self>, values: &CaseValues) -> Result<BoundValues, MathError> {
        let _phase = pse_diagnostics::observation::current().map(|context| {
            context.count("math.value_binds", "count", 1);
            context.phase("math.value_bind")
        });
        self.structure.validate_values(values)?;
        Ok(BoundValues {
            assembly: Arc::clone(self),
            fixed: self.fixed.iter().map(|id| values.scalars[id]).collect(),
        })
    }
    /// Identity and known extent of the shallow plan wrapper and its body map only.
    pub fn plan_wrapper_allocation(&self) -> (usize, usize) {
        (
            Arc::as_ptr(&self.plan) as usize,
            size_of::<CasePlan>()
                + self.plan.bodies.len()
                    * (size_of::<Arc<PreparedBody>>() + size_of::<ContentHash>() + 96),
        )
    }
    /// Immutable compiled products for deduplicated live allocation observation.
    pub fn programs(&self) -> &[Arc<CompiledBody>] {
        &self.programs
    }
    /// Attempt-local shared evaluator scratch, occurrence caches and sparse refill capacity.
    pub fn numeric_worker_bytes(&self) -> usize {
        self.numeric_worker_bytes
    }
    /// An attempt owns all mutable evaluator and provider state.
    pub fn worker(
        self: &Arc<Self>,
        providers: BTreeMap<ProviderKey, Box<dyn Provider>>,
        cancel: pse_diagnostics::Cancellation,
    ) -> CaseWorker {
        self.worker_scoped(providers, pse_kernels::ExecutionScope::new(cancel, None))
    }
    /// Attach the original enclosing deadline to every primitive and nested callback.
    pub fn worker_scoped(
        self: &Arc<Self>,
        providers: BTreeMap<ProviderKey, Box<dyn Provider>>,
        scope: pse_kernels::ExecutionScope,
    ) -> CaseWorker {
        let groups = self
            .instances
            .iter()
            .enumerate()
            .map(|(index, i)| {
                EnumMap::from_fn(|demand| {
                    i.groups[demand].as_ref().map(|_| {
                        let inputs = self.bindings[index][demand]
                            .as_ref()
                            .map_or(0, EvalBindings::len);
                        GroupCache {
                            inputs: vec![f64::NAN; inputs],
                            bits: vec![
                                0;
                                inputs
                                    + if demand == EvaluationDemand::Directional {
                                        i.coordinates.len()
                                    } else {
                                        0
                                    }
                            ],
                            direction: if demand == EvaluationDemand::Directional {
                                vec![0.; i.coordinates.len()]
                            } else {
                                Vec::new()
                            },
                            cache: None,
                            result: TrialEvaluation::default(),
                        }
                    })
                })
            })
            .collect();
        CaseWorker {
            assembly: Arc::clone(self),
            groups,
            workers: self
                .programs
                .iter()
                .map(|p| p.worker_scoped(scope.clone()))
                .collect(),
            providers,
            cancel: scope.cancellation().clone(),
            scope,
            jacobian: Some(self.jacobian.as_ref().clone()),
            hessian: Some(self.hessian.as_ref().clone()),
            constraint_values: vec![f64::NAN; self.rows.len()],
            gradient_values: vec![f64::NAN; self.columns.len()],
            recycler: None,
            observation: None,
            observation_operation: None,
        }
    }
    /// Exclusively attach reset storage to this exact immutable assembly. The caller
    /// owns its resident charge and supplies freshly constructed attempt providers.
    ///
    /// # Errors
    /// Scratch belongs to another assembly, was not reset, or the new scope stopped.
    pub fn worker_from_scratch(
        self: &Arc<Self>,
        scratch: CaseScratch,
        providers: BTreeMap<ProviderKey, Box<dyn Provider>>,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<CaseWorker, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        if !Arc::ptr_eq(self, &scratch.assembly) || !scratch.reset {
            return Err(MathError::Contract(
                "incompatible or unreset case scratch".into(),
            ));
        }
        let CaseScratch {
            assembly,
            groups,
            mut workers,
            jacobian,
            hessian,
            constraint_values,
            gradient_values,
            ..
        } = scratch;
        for worker in &mut workers {
            worker.set_scope(scope.clone());
        }
        Ok(CaseWorker {
            scope: scope.clone(),
            assembly,
            groups,
            workers,
            providers,
            cancel: scope.cancellation().clone(),
            jacobian,
            hessian,
            constraint_values,
            gradient_values,
            recycler: None,
            observation: None,
            observation_operation: None,
        })
    }
}
#[derive(Debug)]
struct GroupCache {
    inputs: Vec<f64>,
    bits: Vec<u64>,
    direction: Vec<f64>,
    cache: Option<DerivativeOrder>,
    result: TrialEvaluation,
}
/// Independent admission of optional owner-local demand evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemandObservationStatus {
    /// No observation operation was supplied at this execution boundary.
    NotRequested,
    /// Collection was disabled, refused or could not allocate its bounded storage.
    Unavailable,
    /// Aggregate facts are collected, but indexed rows were refused independently.
    AggregateOnly,
    /// Indexed populations are collected under the actual operation identity.
    Collected,
}
/// Attempt-local counters for one occurrence, demand and requested order.
/// A set overflow flag makes the saturated values lower bounds, never exact totals.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DemandCounts {
    /// Calls reaching this occurrence, including binding and evaluation failures.
    pub calls: u64,
    /// Calls whose requested result was complete, including valid cache hits.
    pub completed: u64,
    /// Calls refused during binding, evaluation or the final scope check.
    pub failed: u64,
    /// Actual local evaluator invocations; cache hits are excluded.
    pub evaluations: u64,
    /// Successful reuse under exact consumed input bits and sufficient order.
    pub cache_hits: u64,
    /// Hits served by a strictly richer derivative order in this same group.
    pub cache_projections: u64,
    /// No earlier result existed for this group.
    pub empty_misses: u64,
    /// An earlier trial refusal or explicit invalidation cleared this group.
    pub invalidated_misses: u64,
    /// A complete cached result consumed different input or direction bits.
    pub changed_input_misses: u64,
    /// Same consumed bits, but insufficient completed derivative order.
    pub order_upgrade_misses: u64,
    /// No cached result, but indexed history was unavailable to establish why.
    pub unclassified_misses: u64,
    /// Complete results by actual achieved order, including richer cache reuse.
    achieved: [u64; 3],
    /// At least one local population exceeded its representable bound.
    pub overflowed: bool,
}
impl DemandCounts {
    fn add(field: &mut u64, overflowed: &mut bool) {
        if let Some(value) = field.checked_add(1) {
            *field = value;
        } else {
            *overflowed = true;
        }
    }
    fn called(&mut self) {
        Self::add(&mut self.calls, &mut self.overflowed);
    }
    fn failed(&mut self) {
        Self::add(&mut self.failed, &mut self.overflowed);
    }
    fn completed(&mut self, achieved: DerivativeOrder) {
        Self::add(&mut self.completed, &mut self.overflowed);
        Self::add(&mut self.achieved[achieved as usize], &mut self.overflowed);
    }
    fn hit(&mut self, requested: DerivativeOrder, achieved: DerivativeOrder) {
        Self::add(&mut self.cache_hits, &mut self.overflowed);
        if achieved > requested {
            Self::add(&mut self.cache_projections, &mut self.overflowed);
        }
        self.completed(achieved);
    }
    fn miss(&mut self, previous: Option<DerivativeOrder>, same: bool, invalidated: Option<bool>) {
        let field = if previous.is_some() {
            if same {
                &mut self.order_upgrade_misses
            } else {
                &mut self.changed_input_misses
            }
        } else {
            match invalidated {
                Some(true) => &mut self.invalidated_misses,
                Some(false) => &mut self.empty_misses,
                None => &mut self.unclassified_misses,
            }
        };
        Self::add(field, &mut self.overflowed);
        Self::add(&mut self.evaluations, &mut self.overflowed);
    }
    /// Number of completed results actually carrying this order.
    pub fn achieved(&self, order: DerivativeOrder) -> u64 {
        self.achieved[order as usize]
    }
}
/// Borrowed indexed runtime facts, with source occurrence and actual template identity.
#[derive(Clone, Copy, Debug)]
pub struct DemandStatistics<'a> {
    /// Original authored occurrence, preserving repeated uses of one template.
    pub instance: SemanticId,
    /// Index in the assembly's immutable template order.
    pub template: usize,
    /// Selected local demand; no cross-demand cache projection is implied.
    pub demand: EvaluationDemand,
    /// Order actually requested by the caller.
    pub requested: DerivativeOrder,
    /// Complete attempt-local populations, including explicit overflow.
    pub counts: &'a DemandCounts,
}
/// Isolated attempt scratch. Failed evaluation clears the failed cache and never publishes output.
#[derive(Debug)]
pub struct CaseWorker {
    scope: pse_kernels::ExecutionScope,
    assembly: Arc<CaseAssembly>,
    groups: Vec<EnumMap<EvaluationDemand, Option<GroupCache>>>,
    // Evaluation is serial under &mut self; scratch is shared only inside this attempt.
    workers: Vec<Worker>,
    providers: BTreeMap<ProviderKey, Box<dyn Provider>>,
    cancel: Arc<AtomicBool>,
    jacobian: Option<AssemblyMatrix>,
    hessian: Option<AssemblyMatrix>,
    constraint_values: Vec<f64>,
    gradient_values: Vec<f64>,
    recycler: Option<ScratchRecycler>,
    observation: Option<Box<CaseObservation>>,
    observation_operation: Option<pse_diagnostics::observation::OperationId>,
}
struct ScratchRecycler(Box<dyn FnOnce(CaseScratch) + Send>);
impl std::fmt::Debug for ScratchRecycler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ScratchRecycler")
    }
}
/// Opaque exclusively owned numeric storage. It contains no provider, result,
/// applicability witness, factor, execution scope or cancellation owner.
#[derive(Debug)]
pub struct CaseScratch {
    assembly: Arc<CaseAssembly>,
    groups: Vec<EnumMap<EvaluationDemand, Option<GroupCache>>>,
    workers: Vec<Worker>,
    jacobian: Option<AssemblyMatrix>,
    hessian: Option<AssemblyMatrix>,
    constraint_values: Vec<f64>,
    gradient_values: Vec<f64>,
    reset: bool,
}
impl CaseScratch {
    /// Compatibility requires the exact immutable admitted assembly/program/profile.
    pub fn compatible(&self, assembly: &Arc<CaseAssembly>) -> bool {
        Arc::ptr_eq(&self.assembly, assembly)
    }
    /// Poison every mutable numeric slot while preserving indexed constants and
    /// instructions. The caller precharges the complete reset allocation overlap.
    ///
    /// # Errors
    /// An evaluator contains unsupported external/provider-lift state. The consuming
    /// failure destroys this candidate; no partially reset object can be attached.
    pub fn reset(mut self) -> Result<Self, MathError> {
        for groups in &mut self.groups {
            for group in groups.values_mut().flatten() {
                group.cache = None;
                group.result.clear();
                group.inputs.fill(f64::NAN);
                group.bits.fill(0);
                group.direction.fill(f64::NAN);
            }
        }
        for worker in &mut self.workers {
            worker.reset_scratch()?;
        }
        if let Some(matrix) = &mut self.jacobian {
            matrix.poison();
        }
        if let Some(matrix) = &mut self.hessian {
            matrix.poison();
        }
        self.constraint_values.fill(f64::NAN);
        self.gradient_values.fill(f64::NAN);
        self.reset = true;
        Ok(self)
    }
}
impl Drop for CaseWorker {
    fn drop(&mut self) {
        let Some(recycler) = self.recycler.take() else {
            return;
        };
        // Provider destruction precedes publication of detached storage. No cache or
        // scope/cancellation handle is carried into the lane's pending slot.
        self.providers.clear();
        self.invalidate_trial();
        for worker in &mut self.workers {
            worker.detach_controls();
        }
        (recycler.0)(CaseScratch {
            assembly: self.assembly.clone(),
            groups: std::mem::take(&mut self.groups),
            workers: std::mem::take(&mut self.workers),
            jacobian: self.jacobian.take(),
            hessian: self.hessian.take(),
            constraint_values: std::mem::take(&mut self.constraint_values),
            gradient_values: std::mem::take(&mut self.gradient_values),
            reset: false,
        });
    }
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
    /// Actual current evaluator realizations, in the immutable template order.
    pub fn evaluator_census(
        &self,
    ) -> impl Iterator<Item = (usize, crate::guarded::WorkerCensus)> + '_ {
        self.workers
            .iter()
            .enumerate()
            .map(|(template, worker)| (template, worker.census()))
    }
    /// Whether indexed diagnostic observations were independently admitted.
    /// An unavailable collection is never represented by a zero population.
    pub fn demand_observation_status(&self) -> DemandObservationStatus {
        if let Some(observation) = &self.observation {
            if observation.indexed_rows {
                DemandObservationStatus::Collected
            } else {
                DemandObservationStatus::AggregateOnly
            }
        } else if self.observation_operation.is_some() {
            DemandObservationStatus::Unavailable
        } else {
            DemandObservationStatus::NotRequested
        }
    }
    /// Borrow indexed observations without scanning or copying mathematical supports.
    /// Consult `demand_observation_status` before interpreting an empty iterator.
    pub fn demand_statistics(&self) -> impl Iterator<Item = DemandStatistics<'_>> {
        self.observation.iter().flat_map(|observation| {
            observation
                .rows
                .iter()
                .zip(self.assembly.instances.iter())
                .zip(self.assembly.structure.instances())
                .flat_map(|((groups, local), binding)| {
                    groups
                        .iter()
                        .filter_map(move |(demand, row)| {
                            row.as_ref()
                                .zip(local.groups[demand].as_ref())
                                .map(|(row, group)| (demand, row, group.request))
                        })
                        .flat_map(move |(demand, row, template)| {
                            row.counts
                                .iter()
                                .map(move |(requested, counts)| DemandStatistics {
                                    instance: binding.instance,
                                    template,
                                    demand,
                                    requested,
                                    counts,
                                })
                        })
                })
        })
    }
    fn bind_observation(&mut self) {
        let context = pse_diagnostics::observation::current();
        let operation = context
            .as_ref()
            .map(pse_diagnostics::observation::ObservationContext::id);
        if self.observation_operation != operation {
            self.observation = context
                .as_ref()
                .and_then(|context| CaseObservation::new(context, &self.assembly));
            self.observation_operation = operation;
        }
        if let Some(observation) = &mut self.observation {
            observation.begin();
        }
    }
    /// Return storage exactly once when the evaluator and its providers are destroyed.
    /// This only detaches storage; the lane must witness callback/native drain and
    /// precharge/reset before making it available to another attempt.
    pub fn recycle_on_drop(&mut self, recycler: impl FnOnce(CaseScratch) + Send + 'static) {
        self.recycler = Some(ScratchRecycler(Box::new(recycler)));
    }
    #[cfg(test)]
    pub(crate) fn instantiated_orders(&self) -> Vec<Vec<DerivativeOrder>> {
        self.workers
            .iter()
            .map(Worker::instantiated_orders)
            .collect()
    }
    /// Shared cooperative cancellation owner for bounded library work on this admitted worker.
    pub fn cancellation(&self) -> &Arc<AtomicBool> {
        &self.cancel
    }
    /// Immutable products shared by the attempt.
    pub fn assembly(&self) -> &Arc<CaseAssembly> {
        &self.assembly
    }
    /// Clear numerical results and observation witnesses after a refused trial.
    pub fn invalidate_trial(&mut self) {
        if let Some(observation) = &mut self.observation {
            observation.invalidate();
        }
        for groups in &mut self.groups {
            for group in groups.values_mut().flatten() {
                group.cache = None;
                group.result.clear();
            }
        }
    }
    fn evaluate(
        &mut self,
        values: EvalInputs<'_>,
        demand: EvaluationDemand,
        order: DerivativeOrder,
    ) -> Result<(), MathError> {
        self.bind_observation();
        let _entered = self
            .observation
            .as_ref()
            .map(|observation| observation.producer.context().enter());
        let _phase = self
            .observation
            .as_ref()
            .map(|observation| observation.phase(demand));
        let result = self.evaluate_inner(values, demand, order);
        if result.is_err() {
            self.invalidate_trial();
        }
        if let Some(observation) = &mut self.observation {
            observation.publish(&self.assembly, demand, order);
        }
        result
    }
    fn evaluate_inner(
        &mut self,
        values: EvalInputs<'_>,
        demand: EvaluationDemand,
        order: DerivativeOrder,
    ) -> Result<(), MathError> {
        self.scope.check().map_err(crate::error::scope_error)?;
        values.check(&self.assembly)?;
        for (i, binding) in self.assembly.structure.instances().iter().enumerate() {
            self.scope.check().map_err(crate::error::scope_error)?;
            let Some(worker) = &mut self.groups[i][demand] else {
                continue;
            };
            let context = |cause| MathError::Instance {
                instance: binding.instance,
                checked_members: binding.checked_members.as_ref().clone(),
                cause: Box::new(cause),
            };
            let group = self.assembly.instances[i].groups[demand]
                .as_ref()
                .ok_or_else(|| MathError::Contract("missing compiled demand".into()))?;
            let bindings = self.assembly.bindings[i][demand]
                .as_ref()
                .ok_or_else(|| MathError::Contract("missing compiled bindings".into()))?;
            if let Some(observation) = &mut self.observation {
                observation.called(i, demand, order);
            }
            if let Err(cause) = values.fill(bindings, &mut worker.inputs) {
                if let Some(observation) = &mut self.observation {
                    observation.failed(i, demand, order);
                }
                worker.cache = None;
                worker.result.clear();
                return Err(context(cause));
            }
            let same = worker
                .bits
                .iter()
                .zip(&worker.inputs)
                .all(|(bits, value)| *bits == value.to_bits());
            if let Some(achieved) = worker.cache.filter(|achieved| *achieved >= order && same) {
                if let Some(observation) = &mut self.observation {
                    observation.hit(i, demand, order, achieved);
                }
                continue;
            }
            if let Some(observation) = &mut self.observation {
                observation.miss(i, demand, order, worker.cache, same);
            }
            worker.cache = None;
            for (bits, value) in worker.bits.iter_mut().zip(&worker.inputs) {
                *bits = value.to_bits();
            }
            let evaluated = self.workers[group.request]
                .evaluate_trial_into(
                    &worker.inputs,
                    order,
                    &mut self.providers,
                    &self.cancel,
                    &mut worker.result,
                )
                .and_then(|()| self.scope.check().map_err(crate::error::scope_error));
            if let Err(cause) = evaluated {
                if let Some(observation) = &mut self.observation {
                    observation.failed(i, demand, order);
                }
                return Err(context(cause));
            }
            if let Some(observation) = &mut self.observation {
                observation.completed(i, demand, order);
            }
            worker.cache = Some(order);
        }
        self.scope.check().map_err(crate::error::scope_error)?;
        Ok(())
    }
    fn result(
        &self,
        instance: usize,
        output: usize,
        demand: EvaluationDemand,
    ) -> Result<(&TrialEvaluation, usize), MathError> {
        let group = self.assembly.instances[instance].groups[demand]
            .as_ref()
            .ok_or_else(|| MathError::Contract("missing compiled demand".into()))?;
        let row = group
            .outputs
            .binary_search(&output)
            .map_err(|_| MathError::Contract("missing demand output".into()))?;
        let result = self.groups[instance][demand]
            .as_ref()
            .filter(|g| g.cache.is_some())
            .ok_or_else(|| MathError::Contract("unevaluated demand".into()))?;
        Ok((&result.result, row))
    }
    /// Read evidence from the exact last demanded programs; this never evaluates an
    /// inactive branch or an unrelated output just to create observations.
    pub fn applicability_observations(&self) -> Vec<pse_model::applicability::Observation> {
        let mut observations = Vec::new();
        for (instance, groups) in self.groups.iter().enumerate() {
            for group in groups.values().flatten() {
                if group.cache.is_some() {
                    let evaluation = &group.result;
                    for observation in evaluation
                        .applicability
                        .iter()
                        .flat_map(crate::execution::ApplicabilityWitness::hydrate)
                    {
                        let mut observation = observation;
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
    pub fn objective<'a>(&mut self, values: impl Into<EvalInputs<'a>>) -> Result<f64, MathError> {
        self.evaluate(
            values.into(),
            EvaluationDemand::Objective,
            DerivativeOrder::Value,
        )?;
        let mut value = 0.0;
        for (i, b) in self.assembly.structure.instances().iter().enumerate() {
            for c in &b.contributions {
                if c.target == Target::PRIMARY {
                    let (v, r) = self.result(i, c.output, EvaluationDemand::Objective)?;
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
    pub fn constraints<'a>(
        &mut self,
        values: impl Into<EvalInputs<'a>>,
    ) -> Result<&[f64], MathError> {
        self.evaluate(
            values.into(),
            EvaluationDemand::Constraints,
            DerivativeOrder::Value,
        )?;
        self.constraint_values.fill(0.0);
        for (i, b) in self.assembly.structure.instances().iter().enumerate() {
            for c in &b.contributions {
                if let Target::Row(id) = c.target {
                    let (v, r) = self.result(i, c.output, EvaluationDemand::Constraints)?;
                    let addend = c.scale * v.values[r];
                    self.constraint_values[self.assembly.rows[&id].get()] += addend;
                }
            }
        }
        for &value in &self.constraint_values { finite(value)?; }
        Ok(&self.constraint_values)
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
                let (evaluation, row) = self.result(i, output, EvaluationDemand::Constraints)?;
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
    pub fn gradient<'a>(
        &mut self,
        values: impl Into<EvalInputs<'a>>,
    ) -> Result<&[f64], MathError> {
        self.evaluate(
            values.into(),
            EvaluationDemand::Objective,
            DerivativeOrder::First,
        )?;
        self.gradient_values.fill(0.0);
        for (i, b) in self.assembly.structure.instances().iter().enumerate() {
            let local = &self.assembly.instances[i];
            for c in &b.contributions {
                if c.target == Target::PRIMARY {
                    let r = local.groups[EvaluationDemand::Objective].as_ref()
                        .and_then(|group| group.outputs.binary_search(&c.output).ok())
                        .ok_or_else(|| MathError::Contract("missing objective output".into()))?;
                    let v = &self.groups[i][EvaluationDemand::Objective].as_ref()
                        .filter(|group| group.cache.is_some())
                        .ok_or_else(|| MathError::Contract("unevaluated gradient demand".into()))?.result;
                    let sign = self.assembly.structure.objective().map_or(1.0, |o| o.sense.sign());
                    for (k, &formal) in local.coordinates.iter().enumerate() {
                        let addend = sign
                            * c.scale
                            * b.slots[formal.get()].scale()
                            * v.jacobian[r * local.coordinates.len() + k];
                        self.gradient_values[local.columns[k].get()] += addend;
                    }
                }
            }
        }
        for &value in &self.gradient_values { finite(value)?; }
        Ok(&self.gradient_values)
    }
    /// Refill the canonical constraint Jacobian without rebuilding structure.
    pub fn jacobian<'a>(
        &mut self,
        values: impl Into<EvalInputs<'a>>,
    ) -> Result<&faer::sparse::SparseColMat<usize, f64>, MathError> {
        self.evaluate(
            values.into(),
            EvaluationDemand::Constraints,
            DerivativeOrder::First,
        )?;
        self.jacobian
            .as_mut()
            .ok_or_else(|| MathError::Contract("detached Jacobian".into()))?
            .clear();
        for (k, t) in self.assembly.jacobian_terms.iter().enumerate() {
            let v = &self.groups[t.instance][EvaluationDemand::Constraints].as_ref()
                .filter(|group| group.cache.is_some())
                .ok_or_else(|| MathError::Contract("unevaluated Jacobian demand".into()))?.result;
            let r = t.result_row;
            let n = self.assembly.instances[t.instance].coordinates.len();
            let value = t.scale * v.jacobian[r * n + t.i];
            self.jacobian
                .as_mut()
                .ok_or_else(|| MathError::Contract("detached Jacobian".into()))?
                .add(Addend::new(k), value)?;
        }
        Ok(self
            .jacobian
            .as_ref()
            .ok_or_else(|| MathError::Contract("detached Jacobian".into()))?
            .matrix())
    }
    /// Refill the lower Lagrangian Hessian with mapped row and objective weights.
    pub fn hessian<'a>(
        &mut self,
        values: impl Into<EvalInputs<'a>>,
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
        self.evaluate(
            values.into(),
            EvaluationDemand::All,
            DerivativeOrder::Second,
        )?;
        self.hessian
            .as_mut()
            .ok_or_else(|| MathError::Contract("detached Hessian".into()))?
            .clear();
        for (k, t) in self.assembly.hessian_terms.iter().enumerate() {
            let weight = match t.target {
                Target::Objective(_) => objective_weight * self.objective_sign(),
                Target::Row(id) => multipliers[self.assembly.rows[&id].get()],
            };
            let v = &self.groups[t.instance][EvaluationDemand::All].as_ref()
                .filter(|group| group.cache.is_some())
                .ok_or_else(|| MathError::Contract("unevaluated Hessian demand".into()))?.result;
            let r = t.result_row;
            let value = weight * t.scale * v.second(r, t.i, t.j);
            self.hessian
                .as_mut()
                .ok_or_else(|| MathError::Contract("detached Hessian".into()))?
                .add(Addend::new(k), value)?;
        }
        Ok(self
            .hessian
            .as_ref()
            .ok_or_else(|| MathError::Contract("detached Hessian".into()))?
            .matrix())
    }
    /// Apply demanded directional programs, or the assembled reference when none are prepared.
    pub fn jacobian_product<'a>(
        &mut self,
        values: impl Into<EvalInputs<'a>>,
        direction: &[f64],
        output: &mut [f64],
    ) -> Result<(), MathError> {
        let values = values.into();
        if !self.assembly.has_directional_actions() {
            self.jacobian(values)?;
            return self
                .jacobian
                .as_ref()
                .ok_or_else(|| MathError::Contract("detached Jacobian".into()))?
                .product(direction, output);
        }
        if direction.len() != self.assembly.columns.len()
            || output.len() != self.assembly.rows.len()
            || direction.iter().any(|v| !v.is_finite())
        {
            return Err(MathError::Contract(
                "finite case direction and exact action output dimensions required".into(),
            ));
        }
        let result = self.directional_product(values, direction);
        match result {
            Ok(result) => {
                output.copy_from_slice(&result);
                Ok(())
            }
            Err(cause) => {
                self.invalidate_trial();
                Err(cause)
            }
        }
    }
    fn directional_product(
        &mut self,
        values: EvalInputs<'_>,
        direction: &[f64],
    ) -> Result<Vec<f64>, MathError> {
        self.bind_observation();
        let _entered = self
            .observation
            .as_ref()
            .map(|observation| observation.producer.context().enter());
        let _phase = self
            .observation
            .as_ref()
            .map(|observation| observation.phase(EvaluationDemand::Directional));
        let result = self.directional_product_inner(values, direction);
        if let Some(observation) = &mut self.observation {
            observation.publish(
                &self.assembly,
                EvaluationDemand::Directional,
                DerivativeOrder::First,
            );
        }
        result
    }
    fn directional_product_inner(
        &mut self,
        values: EvalInputs<'_>,
        direction: &[f64],
    ) -> Result<Vec<f64>, MathError> {
        self.scope.check().map_err(crate::error::scope_error)?;
        values.check(&self.assembly)?;
        let mut output = vec![0.0; self.assembly.rows.len()];
        for (index, binding) in self.assembly.structure.instances().iter().enumerate() {
            self.scope.check().map_err(crate::error::scope_error)?;
            let local = &self.assembly.instances[index];
            let Some(group) = &local.groups[EvaluationDemand::Directional] else {
                continue;
            };
            let context = |cause| MathError::Instance {
                instance: binding.instance,
                checked_members: binding.checked_members.as_ref().clone(),
                cause: Box::new(cause),
            };
            let cache = self.groups[index][EvaluationDemand::Directional]
                .as_mut()
                .ok_or_else(|| MathError::Contract("missing directional group".into()))?;
            let bindings = self.assembly.bindings[index][EvaluationDemand::Directional]
                .as_ref()
                .ok_or_else(|| MathError::Contract("missing directional bindings".into()))?;
            if let Some(observation) = &mut self.observation {
                observation.called(index, EvaluationDemand::Directional, DerivativeOrder::First);
            }
            let prepared = values.fill(bindings, &mut cache.inputs).and_then(|()| {
                for (axis, (&formal, &column)) in
                    local.coordinates.iter().zip(&local.columns).enumerate()
                {
                    cache.direction[axis] =
                        finite(binding.slots[formal.get()].scale() * direction[column.get()])?;
                }
                Ok(())
            });
            if let Err(cause) = prepared {
                if let Some(observation) = &mut self.observation {
                    observation.failed(
                        index,
                        EvaluationDemand::Directional,
                        DerivativeOrder::First,
                    );
                }
                return Err(context(cause));
            }
            let same = cache
                .bits
                .iter()
                .zip(cache.inputs.iter().chain(cache.direction.iter()))
                .all(|(bits, value)| *bits == value.to_bits());
            if cache.cache.is_some() && same {
                if let Some(observation) = &mut self.observation {
                    observation.hit(
                        index,
                        EvaluationDemand::Directional,
                        DerivativeOrder::First,
                        DerivativeOrder::First,
                    );
                }
            } else {
                if let Some(observation) = &mut self.observation {
                    observation.miss(
                        index,
                        EvaluationDemand::Directional,
                        DerivativeOrder::First,
                        cache.cache,
                        same,
                    );
                }
                cache.cache = None;
                for (bits, value) in cache
                    .bits
                    .iter_mut()
                    .zip(cache.inputs.iter().chain(cache.direction.iter()))
                {
                    *bits = value.to_bits();
                }
                let evaluated = self.workers[group.request]
                    .evaluate_directional_trial_into(
                        &cache.inputs,
                        &cache.direction,
                        &mut self.providers,
                        &self.cancel,
                        &mut cache.result,
                    )
                    .and_then(|()| self.scope.check().map_err(crate::error::scope_error));
                if let Err(cause) = evaluated {
                    if let Some(observation) = &mut self.observation {
                        observation.failed(
                            index,
                            EvaluationDemand::Directional,
                            DerivativeOrder::First,
                        );
                    }
                    return Err(context(cause));
                }
                if let Some(observation) = &mut self.observation {
                    observation.completed(
                        index,
                        EvaluationDemand::Directional,
                        DerivativeOrder::First,
                    );
                }
                cache.cache = Some(DerivativeOrder::First);
            }
            if cache.cache.is_none() {
                return Err(MathError::Contract("unevaluated directional group".into()));
            }
            let evaluation = &cache.result;
            for contribution in &binding.contributions {
                if let Target::Row(row) = contribution.target {
                    let local_row = group
                        .outputs
                        .binary_search(&contribution.output)
                        .map_err(|_| MathError::Contract("missing directional output".into()))?;
                    let slot = self.assembly.rows[&row].get();
                    output[slot] =
                        finite(output[slot] + contribution.scale * evaluation.jacobian[local_row])
                            .map_err(context)?;
                }
            }
        }
        self.scope.check().map_err(crate::error::scope_error)?;
        Ok(output)
    }
}
fn selected(target: Target, demand: EvaluationDemand) -> bool {
    match demand {
        EvaluationDemand::Objective => target == Target::PRIMARY,
        EvaluationDemand::Constraints => matches!(target, Target::Row(_)),
        EvaluationDemand::Directional => false,
        // A later lexicographic objective is projected to coefficients, never evaluated.
        EvaluationDemand::All => matches!(target, Target::Row(_)) || target == Target::PRIMARY,
    }
}
fn finite(value: f64) -> Result<f64, MathError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(MathError::Contract("nonfinite assembled result".into()))
    }
}

#[cfg(test)]
mod observation_counter_tests {
    use super::*;
    #[test]
    fn demand_counter_overflow_is_explicit_and_never_wraps_to_zero() {
        let mut counts = DemandCounts {
            calls: u64::MAX,
            ..DemandCounts::default()
        };
        counts.called();
        assert_eq!(counts.calls, u64::MAX);
        assert!(counts.overflowed);
        counts.completed(DerivativeOrder::First);
        assert_eq!(counts.completed, 1);
        assert_eq!(counts.achieved(DerivativeOrder::First), 1);
    }
}
