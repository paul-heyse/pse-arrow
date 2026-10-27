// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical local gathers, explicit row contributions and library-owned sparse products.
use crate::{
    MathError,
    binding::{CaseStructure, CaseValues, EvalBindings, Target},
    execution::TrialEvaluation,
    guarded::{CompiledBody, PreparedBody, PreparedSupport, ProductKind, ProductValues, Worker},
    index::{Addend, Entry, GlobalCol, GlobalRow, Slot},
    jets::EvaluationLimits,
    library::Optimization,
    sparse::AssemblyMatrix,
};
#[path = "assembly_entries.rs"]
mod entries;
#[path = "assembly_projection.rs"]
mod projection;
use projection::{InstanceReuse, ProjectionRelationship, project_compatible};
#[path = "assembly_completion.rs"]
mod completion;
#[path = "assembly_observation.rs"]
mod observation;
use completion::{AssembledProduct, AssemblyCompletion};
use enum_map::EnumMap;
use observation::CaseObservation;
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::DerivativeOrder;
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
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    enum_map::Enum,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationDemand {
    /// Selected primary objective only.
    Objective,
    /// Selected constraint outputs only.
    Constraints,
    /// All selected objective and constraint outputs.
    All,
    /// One directional constraint action.
    Directional,
    /// Constraint cotangent pulled back to global coordinates.
    TransposeDirectional,
    /// Weighted second derivative applied directly to one global direction.
    HessianAction,
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
    objective_terms: Vec<OutputTerm>,
    constraint_terms: Vec<ConstraintTerm>,
    gradient_coordinates: Vec<GradientCoordinate>,
}
/// Immutable output/derivative addresses in original contribution order.
#[derive(Clone, Debug)]
struct OutputTerm {
    result_row: usize,
    scale: f64,
}
#[derive(Clone, Debug)]
struct ConstraintTerm {
    result_row: usize,
    destination: usize,
    scale: f64,
}
#[derive(Clone, Debug)]
struct GradientCoordinate {
    destination: usize,
    scale: f64,
}
#[derive(Clone, Debug)]
struct Term {
    instance: usize,
    output: usize,
    result_offset: usize,
    i: usize,
    j: usize,
    scale: f64,
    target: Target,
}
#[derive(Clone, Debug)]
struct HessianTerm {
    instance: usize,
    component: usize,
    i: usize,
    j: usize,
    scale: f64,
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
    ownership: Arc<PlanOwnership>,
    product_weights: Arc<Vec<EnumMap<EvaluationDemand, Vec<ProductWeight>>>>,
    jacobian: Arc<AssemblyMatrix>,
    hessian: Arc<AssemblyMatrix>,
    jacobian_terms: Arc<Vec<Term>>,
    hessian_terms: Arc<Vec<HessianTerm>>,
    order: DerivativeOrder,
    requests: Arc<Vec<LocalDemand>>,
    supports: Arc<Vec<Arc<PreparedSupport>>>,
    numerical_supports: Option<Arc<Vec<Arc<PreparedSupport>>>>,
    entries: Option<Arc<entries::EntryPlan>>,
    available: DerivativeOrder,
    actions: PreparedActions,
    limits: AssemblyLimits,
    owner: Option<Arc<dyn crate::AllocationOwner>>,
}
/// Derived access paths over the exact immutable binding and demand population.
#[derive(Clone, Debug)]
struct PlanOwnership {
    consumers: Vec<Vec<(usize, EvaluationDemand)>>,
    slot_sources: Vec<SemanticId>,
    source_bindings: BTreeMap<SemanticId, Vec<usize>>,
    slots: usize,
    contributions: usize,
}
impl PlanOwnership {
    fn new(
        structure: &CaseStructure,
        instances: &[Instance],
        requests: usize,
    ) -> Result<Self, MathError> {
        let overflow = || MathError::Limit("case ownership index extent");
        let slots = structure
            .instances()
            .iter()
            .try_fold(0usize, |n, binding| n.checked_add(binding.slots.len()))
            .ok_or_else(overflow)?;
        let groups = instances
            .iter()
            .try_fold(0usize, |n, instance| {
                n.checked_add(instance.groups.values().flatten().count())
            })
            .ok_or_else(overflow)?;
        let bytes = requests
            .checked_mul(size_of::<Vec<(usize, EvaluationDemand)>>())
            .and_then(|bytes| {
                bytes.checked_add(groups.checked_mul(2 * size_of::<(usize, EvaluationDemand)>())?)
            })
            .and_then(|bytes| {
                bytes.checked_add(slots.checked_mul(192 + 2 * size_of::<SemanticId>())?)
            })
            .and_then(|bytes| bytes.checked_add(size_of::<Self>()))
            .ok_or_else(overflow)?;
        crate::construction::additional(bytes)?;
        let mut consumers = vec![Vec::new(); requests];
        for (index, instance) in instances.iter().enumerate() {
            for (demand, group) in instance.groups.iter() {
                if let Some(group) = group {
                    consumers[group.request].push((index, demand));
                }
            }
        }
        let mut source_bindings = BTreeMap::<_, Vec<_>>::new();
        for (index, binding) in structure.instances().iter().enumerate() {
            for source in binding
                .slots
                .iter()
                .map(|slot| slot.source())
                .collect::<BTreeSet<_>>()
            {
                source_bindings.entry(source).or_default().push(index);
            }
        }
        Ok(Self {
            consumers,
            slot_sources: source_bindings.keys().copied().collect(),
            source_bindings,
            slots,
            contributions: structure
                .instances()
                .iter()
                .map(|binding| binding.contributions.len())
                .sum(),
        })
    }
    fn bytes(&self) -> usize {
        size_of::<Self>()
            + self.consumers.capacity() * size_of::<Vec<(usize, EvaluationDemand)>>()
            + self
                .consumers
                .iter()
                .map(|consumers| consumers.capacity() * size_of::<(usize, EvaluationDemand)>())
                .sum::<usize>()
            + self.slot_sources.capacity() * size_of::<SemanticId>()
            + self
                .source_bindings
                .values()
                .map(|bindings| {
                    96 + size_of::<SemanticId>()
                        + size_of::<Vec<usize>>()
                        + bindings.capacity() * size_of::<usize>()
                })
                .sum::<usize>()
    }
}
// Preparation permission survives an empty/constant projection, where there is
// no local program to carry it. Requests below still own actual compiled work.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PreparedActions {
    jacobian: bool,
    transpose: bool,
    hessian: bool,
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
    /// Refresh only current source attribution after exact numerical compatibility.
    /// Prepared supports, sparse layouts and immutable numerical programs stay shared.
    pub fn receive_source_context(&self, current: &CaseStructure) -> Result<Self, MathError> {
        let structure = self.structure.receive_source_context(current)?;
        for binding in structure.instances() {
            let body = self.bodies.get(&binding.body).ok_or_else(|| {
                MathError::Contract("receiving source body is absent from cached plan".into())
            })?;
            crate::SourceBindings::check_inventory(
                binding.instance,
                body,
                &binding.checked_members,
                &binding.checked_sources,
                &binding.occurrences,
            )?;
        }
        let mut received = self.clone();
        received.structure = Arc::new(structure);
        Ok(received)
    }
    /// Conservative receiving-shell copy allowance, with original numerical payloads shared.
    pub fn source_context_allocation_bound(&self) -> Result<usize, MathError> {
        self.structure
            .source_context_allocation_bound()?
            .checked_add(
                self.bodies
                    .len()
                    .checked_mul(size_of::<(ContentHash, Arc<PreparedBody>)>() + 96)
                    .ok_or(MathError::Limit("receiving plan source body wrappers"))?,
            )
            .and_then(|bytes| bytes.checked_add(size_of::<Self>() + 4096))
            .ok_or(MathError::Limit("receiving plan source context extent"))
    }

    /// Validate actual typed body/slot/output contracts without preparing arithmetic.
    /// # Errors
    /// Missing or incompatible contracts, ambiguous accumulation, or cancellation.
    #[doc(hidden)]
    pub fn check_physical_bindings(
        structure: &CaseStructure,
        bodies: &BTreeMap<ContentHash, Arc<PreparedBody>>,
        registry: &QuantityRegistry,
        cancel: &Arc<AtomicBool>,
    ) -> Result<(), MathError> {
        let rows: BTreeMap<_, _> = structure
            .rows()
            .iter()
            .enumerate()
            .map(|(i, row)| (row.id, GlobalRow::new(i)))
            .collect();
        let mut target_counts = BTreeMap::new();
        for binding in structure.instances() {
            for contribution in &binding.contributions {
                *target_counts.entry(contribution.target).or_insert(0usize) += 1;
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
            if !body
                .checked_sources()
                .iter()
                .eq(binding.checked_sources.keys())
                || binding
                    .checked_sources
                    .values()
                    .any(|id| *id == SemanticId::NIL)
            {
                return Err(MathError::Contract(
                    "incomplete or invalid checked-source attribution".into(),
                ));
            }
            if !body
                .occurrence_tokens()
                .iter()
                .eq(binding.occurrences.keys())
                || binding.occurrences.iter().any(|(token, location)| {
                    *token == SemanticId::NIL || location.source == SemanticId::NIL
                })
            {
                return Err(MathError::Contract(
                    "incomplete or invalid original occurrence attribution".into(),
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
        }
        Ok(())
    }
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
            let metadata = pse_model::allocation::btree_allocation_bound::<usize, ()>(dimensions)
                .ok_or_else(overflow)?;
            let dependencies = binding
                .contributions
                .len()
                .checked_mul(
                    size_of::<ContributionDependencies>()
                        .checked_add(
                            pse_model::allocation::btree_allocation_bound::<SemanticId, ()>(
                                binding.slots.len(),
                            )
                            .and_then(|bytes| bytes.checked_mul(2))
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
        for index in self.selected_contributions(rows.iter().copied()).keys() {
            let binding = &self.structure.instances()[*index];
            let Some(scratch) =
                self.bodies[&binding.body].arithmetic_analysis_allocation_bound(None)?
            else {
                return Ok(None);
            };
            let metadata = binding
                .slots
                .len()
                .checked_add(binding.contributions.len())
                .and_then(pse_model::allocation::btree_allocation_bound::<usize, ()>)
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
        visit: impl FnMut(ContributionDependencies) -> std::ops::ControlFlow<B>,
    ) -> Result<std::ops::ControlFlow<B>, MathError> {
        self.visit_binding_dependencies(0..self.structure.instances().len(), cancel, visit)
    }
    /// Complete obligations of bindings relevant to a selected conditional boundary.
    /// Candidate slots are conservative: fixed/inactive inputs and every original
    /// objective or outside row in a selected binding remain visible to admission.
    pub fn boundary_dependencies(
        &self,
        rows: &BTreeSet<SemanticId>,
        unknowns: &BTreeSet<SemanticId>,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Vec<ContributionDependencies>, MathError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let bindings = self.boundary_bindings(rows, unknowns);
        let mut result = Vec::new();
        let _ = self.visit_binding_dependencies(bindings.into_iter(), cancel, |dependency| {
            result.push(dependency);
            std::ops::ControlFlow::<std::convert::Infallible>::Continue(())
        })?;
        Ok(result)
    }
    fn boundary_bindings(
        &self,
        rows: &BTreeSet<SemanticId>,
        unknowns: &BTreeSet<SemanticId>,
    ) -> BTreeSet<usize> {
        rows.iter()
            .flat_map(|row| {
                self.row_contributions(*row)
                    .iter()
                    .map(|&(binding, _)| binding)
            })
            .chain(
                unknowns
                    .iter()
                    .filter_map(|id| self.ownership.source_bindings.get(id))
                    .flatten()
                    .copied(),
            )
            .collect()
    }
    fn visit_binding_dependencies<B>(
        &self,
        bindings: impl Iterator<Item = usize>,
        cancel: &Arc<AtomicBool>,
        mut visit: impl FnMut(ContributionDependencies) -> std::ops::ControlFlow<B>,
    ) -> Result<std::ops::ControlFlow<B>, MathError> {
        for index in bindings {
            #[cfg(test)]
            ownership_tests::DEPENDENCY_VISITS.with(|visits| {
                let (bindings, supports) = visits.get();
                visits.set((bindings + 1, supports));
            });
            let binding = &self.structure.instances()[index];
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
            let established = self.instances[index]
                .groups
                .values()
                .flatten()
                .map(|group| &self.supports[group.request])
                .find(|support| {
                    #[cfg(test)]
                    ownership_tests::DEPENDENCY_VISITS.with(|visits| {
                        let (bindings, supports) = visits.get();
                        visits.set((bindings, supports + 1));
                    });
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
                    i.objective_terms.capacity() * size_of::<OutputTerm>()
                        + i.constraint_terms.capacity() * size_of::<ConstraintTerm>()
                        + i.gradient_coordinates.capacity() * size_of::<GradientCoordinate>()
                        + i.coordinates.capacity() * size_of::<Slot>()
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
                "product_weights",
                Arc::as_ptr(&self.product_weights) as usize,
                size_of::<Vec<EnumMap<EvaluationDemand, Vec<ProductWeight>>>>()
                    + self.product_weights.capacity()
                        * size_of::<EnumMap<EvaluationDemand, Vec<ProductWeight>>>()
                    + self
                        .product_weights
                        .iter()
                        .flat_map(|g| g.values())
                        .map(|v| v.capacity() * size_of::<ProductWeight>())
                        .sum::<usize>(),
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
                size_of::<Vec<HessianTerm>>()
                    + self.hessian_terms.capacity() * size_of::<HessianTerm>(),
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
            "plan-ownership",
            Arc::as_ptr(&self.ownership) as usize,
            self.ownership.bytes(),
        ));
        if let Some(entries) = &self.entries {
            components.push(("entry-plan", Arc::as_ptr(entries) as usize, entries.bytes()));
        }
        if let Some(supports) = &self.numerical_supports {
            components.push((
                "numerical-supports",
                Arc::as_ptr(supports) as usize,
                size_of_val(supports.as_ref())
                    + supports.capacity() * size_of::<Arc<PreparedSupport>>(),
            ));
        }
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
            components.push((
                "binding-sources",
                Arc::as_ptr(&binding.checked_sources) as usize,
                size_of_val(binding.checked_sources.as_ref())
                    + 2 * size_of::<usize>()
                    + binding.checked_sources.len() * (size_of::<(SemanticId, SemanticId)>() + 96),
            ));
            components.push((
                "binding-occurrences",
                Arc::as_ptr(&binding.occurrences) as usize,
                size_of_val(binding.occurrences.as_ref())
                    + 2 * size_of::<usize>()
                    + binding.occurrences.len()
                        * (size_of::<(SemanticId, pse_model::diagnostic::SourceLocation)>() + 96)
                    + binding
                        .occurrences
                        .values()
                        .map(pse_model::diagnostic::SourceLocation::heap_bytes)
                        .sum::<usize>(),
            ));
        }
        components
    }
    /// Process-local shared storage identities, independent of owner wrappers and body data.
    pub fn allocation_identity(&self) -> Vec<usize> {
        let mut identities = vec![
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
        ];
        identities.extend(
            self.entries
                .iter()
                .map(|entries| Arc::as_ptr(entries) as usize),
        );
        identities.extend(
            self.numerical_supports
                .iter()
                .map(|supports| Arc::as_ptr(supports) as usize),
        );
        identities
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
            for support in plan
                .supports
                .iter()
                .chain(plan.numerical_supports.iter().flat_map(|s| s.iter()))
            {
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
        if let Some(supports) = &mut self.numerical_supports {
            for support in Arc::make_mut(supports) {
                *support = Arc::new(support.as_ref().clone().with_owner(owner.clone()));
            }
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
        Self::check_physical_bindings(&structure, &bodies, registry, cancel)?;
        let mut requests = Vec::<LocalDemand>::new();
        let mut supports = Vec::<Arc<PreparedSupport>>::new();
        let mut available = DerivativeOrder::Second;
        let mut shared_supports =
            BTreeMap::<(ContentHash, Vec<usize>, Vec<usize>), PreparedSupport>::new();
        let mut request_indices = BTreeMap::<_, usize>::new();
        let mut instances = vec![];
        for binding in structure.instances() {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            let body = bodies
                .get(&binding.body)
                .ok_or_else(|| MathError::Contract("missing prepared body".into()))?;
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
                    product: if ceiling == DerivativeOrder::Second {
                        ProductKind::WeightedHessian
                    } else {
                        ProductKind::SelectedPartials
                    },
                };
                let key = (
                    demand.body,
                    demand.outputs.clone(),
                    demand.coordinates.clone(),
                    demand.product,
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
            let mut objective_terms = Vec::new();
            let mut constraint_terms = Vec::new();
            for contribution in &binding.contributions {
                let demand = match contribution.target {
                    Target::PRIMARY => EvaluationDemand::Objective,
                    Target::Row(_) => EvaluationDemand::Constraints,
                    _ => continue,
                };
                let result_row = groups[demand]
                    .as_ref()
                    .and_then(|group| group.outputs.binary_search(&contribution.output).ok())
                    .ok_or_else(|| {
                        MathError::Contract("missing prepared contribution output".into())
                    })?;
                match contribution.target {
                    Target::PRIMARY => objective_terms.push(OutputTerm {
                        result_row,
                        scale: contribution.scale,
                    }),
                    Target::Row(row) => constraint_terms.push(ConstraintTerm {
                        result_row,
                        destination: rows[&row].get(),
                        scale: contribution.scale,
                    }),
                    _ => {}
                }
            }
            let gradient_coordinates = if !objective_terms.is_empty() {
                coordinates
                    .iter()
                    .enumerate()
                    .map(|(offset, formal)| GradientCoordinate {
                        destination: local_columns[offset].get(),
                        scale: binding.slots[formal.get()].scale(),
                    })
                    .collect()
            } else {
                Vec::new()
            };
            instances.push(Instance {
                groups,
                coordinates,
                columns: local_columns,
                objective_terms,
                constraint_terms,
                gradient_coordinates,
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
        let product_weights = Arc::new(prepare_product_weights(&instances, &structure, &rows));
        let ownership = Arc::new(PlanOwnership::new(&structure, &instances, requests.len())?);
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
            ownership,
            product_weights,
            jacobian: Arc::new(patterns.jacobian),
            hessian: Arc::new(patterns.hessian),
            jacobian_terms: Arc::new(patterns.jacobian_terms),
            hessian_terms: Arc::new(patterns.hessian_terms),
            order,
            requests: Arc::new(requests),
            supports: Arc::new(supports),
            numerical_supports: None,
            entries: None,
            actions: PreparedActions::default(),
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
                    occurrences: binding.occurrences.clone(),
                    checked_sources: binding.checked_sources.clone(),
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
        for (index, ordinals) in self.selected_contributions(rows.iter().copied()) {
            if cancel.load(Ordering::Relaxed) {
                return Err(MathError::Cancelled);
            }
            let binding = &self.structure.instances()[index];
            let outputs: Vec<_> = ordinals
                .iter()
                .map(|&ordinal| binding.contributions[ordinal].output)
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
        let rows = selected_rows
            .iter()
            .map(|row| self.structure.rows()[self.rows[row].get()].clone())
            .collect();
        let binding_indices = self.selected_contributions(selected_rows.iter().copied());
        let instances: Vec<_> = binding_indices
            .into_iter()
            .map(|(index, ordinals)| {
                let binding = &self.structure.instances()[index];
                crate::binding::InstanceBinding {
                    instance: binding.instance,
                    body: binding.body,
                    slots: binding.slots.clone(),
                    occurrences: binding.occurrences.clone(),
                    checked_sources: binding.checked_sources.clone(),
                    checked_members: binding.checked_members.clone(),
                    contributions: ordinals
                        .into_iter()
                        .map(|ordinal| binding.contributions[ordinal].clone())
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
        self.with_first_action(
            ProductKind::JacobianAction,
            EvaluationDemand::Directional,
            cancel,
        )
    }
    /// Admit constraint pullbacks only for consumers requesting transpose actions.
    pub fn with_transpose_actions(&self, cancel: &Arc<AtomicBool>) -> Result<Self, MathError> {
        self.with_first_action(
            ProductKind::TransposeJacobianAction,
            EvaluationDemand::TransposeDirectional,
            cancel,
        )
    }
    fn with_first_action(
        &self,
        product: ProductKind,
        demand: EvaluationDemand,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let mut instances = self.instances.as_ref().clone();
        let mut requests = self.requests.as_ref().clone();
        let mut supports = self.supports.as_ref().clone();
        let mut indices = BTreeMap::new();
        for (index, request) in requests
            .iter()
            .enumerate()
            .filter(|(_, r)| r.product == product)
        {
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
            if instance.groups[demand].is_some() {
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
                    product,
                });
                supports.push(Arc::new(support));
                indices.insert(key, index);
                index
            };
            instance.groups[demand] = Some(Group {
                outputs: group.outputs.clone(),
                request,
            });
        }
        let mut result = self.clone();
        match product {
            ProductKind::JacobianAction => result.actions.jacobian = true,
            ProductKind::TransposeJacobianAction => result.actions.transpose = true,
            _ => return Err(MathError::Contract("first action preparation kind".into())),
        }
        result.product_weights = Arc::new(prepare_product_weights(
            &instances,
            &self.structure,
            &self.rows,
        ));
        result.ownership = Arc::new(PlanOwnership::new(
            &self.structure,
            &instances,
            requests.len(),
        )?);
        result.instances = Arc::new(instances);
        result.requests = Arc::new(requests);
        result.supports = Arc::new(supports);
        result.refresh_numerical_supports()?;
        Ok(result)
    }
    /// Admit actual Hessian actions only for consumers that request them.
    pub fn with_hessian_actions(&self, cancel: &Arc<AtomicBool>) -> Result<Self, MathError> {
        if self.order < DerivativeOrder::Second {
            return Err(MathError::Contract(
                "Hessian actions require Second support".into(),
            ));
        }
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        let mut result = self.clone();
        result.actions.hessian = true;
        let mut instances = self.instances.as_ref().clone();
        let mut requests = self.requests.as_ref().clone();
        let mut supports = self.supports.as_ref().clone();
        let mut prepared = BTreeMap::new();
        for instance in &mut instances {
            if instance.groups[EvaluationDemand::HessianAction].is_some() {
                continue;
            }
            let Some(all) = instance.groups[EvaluationDemand::All].as_ref() else {
                continue;
            };
            let request = *prepared.entry(all.request).or_insert_with(|| {
                let index = requests.len();
                let mut request = requests[all.request].clone();
                request.product = ProductKind::WeightedHessianAction;
                requests.push(request);
                supports.push(supports[all.request].clone());
                index
            });
            instance.groups[EvaluationDemand::HessianAction] = Some(Group {
                outputs: all.outputs.clone(),
                request,
            });
        }
        result.product_weights = Arc::new(prepare_product_weights(
            &instances,
            &self.structure,
            &self.rows,
        ));
        result.ownership = Arc::new(PlanOwnership::new(
            &self.structure,
            &instances,
            requests.len(),
        )?);
        result.instances = Arc::new(instances);
        result.requests = Arc::new(requests);
        result.supports = Arc::new(supports);
        result.refresh_numerical_supports()?;
        Ok(result)
    }
    /// Whether a direct weighted action program was explicitly admitted.
    pub fn has_hessian_actions(&self) -> bool {
        self.actions.hessian
    }
    /// Whether directional constraint actions were explicitly admitted, including
    /// mathematically empty projections requiring no local arithmetic program.
    pub fn has_directional_actions(&self) -> bool {
        self.actions.jacobian
    }
    /// Whether constraint cotangent actions were explicitly admitted.
    pub fn has_transpose_actions(&self) -> bool {
        self.actions.transpose
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
            let requested = if matches!(
                request.product,
                ProductKind::JacobianAction | ProductKind::TransposeJacobianAction
            ) {
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
            copies = add(copies, mul(n, 2 * size_of::<GradientCoordinate>())?)?;
            copies = add(
                copies,
                mul(
                    add(binding.contributions.len(), 8)?,
                    2 * (size_of::<OutputTerm>() + size_of::<ConstraintTerm>()),
                )?,
            )?;
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
        let ownership_bytes = add(
            self.ownership.bytes(),
            mul(self.ownership.slots, 192 + 2 * size_of::<SemanticId>())?,
        )?;
        let descriptors = add(
            ownership_bytes,
            add(
                mul(
                    add(mul(count, 2)?, 4)?,
                    size_of::<LocalDemand>() + size_of::<Arc<PreparedSupport>>(),
                )?,
                add(
                    mul(self.instances.len(), size_of::<Instance>())?,
                    add(copies, mul(dimensions, 4 * size_of::<usize>())?)?,
                )?,
            )?,
        )?;
        let maps = add(
            pse_model::allocation::btree_allocation_bound::<
                (ContentHash, Vec<usize>, Vec<usize>, usize),
                Arc<PreparedSupport>,
            >(count)
            .ok_or_else(overflow)?,
            pse_model::allocation::btree_allocation_bound::<usize, Arc<PreparedSupport>>(count)
                .ok_or_else(overflow)?,
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
        let bodies =
            pse_model::allocation::btree_allocation_bound::<ContentHash, Arc<PreparedBody>>(
                self.bodies.len(),
            )
            .ok_or_else(overflow)?;
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
        if order <= self.order() {
            return Ok(self.clone());
        }
        if self.numerical_supports.is_some() {
            let mut restored = self.clone();
            restored.numerical_supports = None;
            for (request, support) in Arc::make_mut(&mut restored.requests)
                .iter_mut()
                .zip(restored.supports.iter())
            {
                request.order = support.order();
            }
            return restored.prepare_order(order, cancel);
        }
        let mut supports = Vec::with_capacity(self.supports.len());
        for (request, previous) in self.supports.iter().enumerate() {
            if matches!(
                self.requests[request].product,
                ProductKind::JacobianAction | ProductKind::TransposeJacobianAction
            ) {
                supports.push(previous.clone());
                continue;
            }
            let requested = if self
                .request_consumers(request)
                .iter()
                .any(|&(_, demand)| demand == EvaluationDemand::All)
            {
                order
            } else {
                order.min(DerivativeOrder::First)
            };
            // The exact shared request owns its current support and finite allowance.
            // Different All consumers may retain different conservative controls;
            // none supplies permission or a replacement premise for another request.
            supports.push(Arc::new(previous.upgrade(requested, cancel)?));
        }
        let mut requests = self.requests.as_ref().clone();
        let mut instances = self.instances.as_ref().clone();
        let all_requests = instances
            .iter()
            .filter_map(|i| i.groups[EvaluationDemand::All].as_ref().map(|g| g.request))
            .collect::<BTreeSet<_>>();
        for &original in &all_requests {
            if order == DerivativeOrder::Second {
                let shared = instances.iter().any(|i| {
                    i.groups.iter().any(|(d, g)| {
                        d != EvaluationDemand::All
                            && g.as_ref().is_some_and(|g| g.request == original)
                    })
                });
                let target = if shared {
                    let target = requests.len();
                    requests.push(requests[original].clone());
                    supports.push(supports[original].clone());
                    supports[original] =
                        Arc::new(supports[original].at_order(DerivativeOrder::First)?);
                    requests[original].order = DerivativeOrder::First;
                    target
                } else {
                    original
                };
                requests[target].order = order;
                requests[target].product = ProductKind::WeightedHessian;
                for instance in &mut instances {
                    if let Some(group) = &mut instance.groups[EvaluationDemand::All]
                        && group.request == original
                    {
                        group.request = target;
                    }
                }
            } else {
                requests[original].order = order;
            }
        }
        for (index, request) in requests.iter_mut().enumerate() {
            if !all_requests.contains(&index) && request.product == ProductKind::SelectedPartials {
                request.order = order.min(DerivativeOrder::First);
            }
        }
        let patterns = case_patterns(
            &self.structure,
            &self.rows,
            &instances,
            &supports,
            order,
            self.columns.len(),
            self.limits,
        )?;
        let mut result = self.clone();
        result.order = order;
        result.product_weights = Arc::new(prepare_product_weights(
            &instances,
            &self.structure,
            &self.rows,
        ));
        result.ownership = Arc::new(PlanOwnership::new(
            &self.structure,
            &instances,
            requests.len(),
        )?);
        result.instances = Arc::new(instances);
        result.requests = Arc::new(requests);
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
            || self.actions != PreparedActions::default()
            || self.columns != original.columns
            || self.bodies != original.bodies
            || self.requests.iter().any(|d| {
                matches!(
                    d.product,
                    ProductKind::JacobianAction | ProductKind::TransposeJacobianAction
                )
            })
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
            && self.hessian_terms.len() == expected.hessian_terms.len()
            && self
                .hessian_terms
                .iter()
                .zip(expected.hessian_terms.iter())
                .all(|(a, b)| {
                    a.instance == b.instance
                        && a.component == b.component
                        && a.i == b.i
                        && a.j == b.j
                        && a.scale.to_bits() == b.scale.to_bits()
                }))
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
    /// Original selected row declaration, addressed through the prepared row index.
    pub fn row(&self, id: SemanticId) -> Option<&crate::binding::Row> {
        self.rows
            .get(&id)
            .map(|row| &self.structure.rows()[row.get()])
    }
    pub(crate) fn projection_topology(&self) -> &crate::factorable::ProjectionTopology {
        &self.projection
    }
    /// Original contribution order, indexed once rather than rediscovered for each row proof.
    pub(crate) fn row_contributions(&self, row: SemanticId) -> &[(usize, usize)] {
        self.row_contributions.get(&row).map_or(&[], Vec::as_slice)
    }
    /// Every original local slot source, including fixed and inactive execution inputs.
    pub fn binding_sources(&self) -> &[SemanticId] {
        &self.ownership.slot_sources
    }
    /// Original slot population, retaining repeated bindings for allocation bounds.
    pub fn binding_slot_count(&self) -> usize {
        self.ownership.slots
    }
    /// Original contribution population, retaining duplicates and objective levels.
    pub fn contribution_count(&self) -> usize {
        self.ownership.contributions
    }
    /// Every original instance/demand consumer in source and demand order.
    pub fn request_consumers(&self, request: usize) -> &[(usize, EvaluationDemand)] {
        self.ownership
            .consumers
            .get(request)
            .map_or(&[], Vec::as_slice)
    }
    fn selected_contributions(
        &self,
        rows: impl IntoIterator<Item = SemanticId>,
    ) -> BTreeMap<usize, Vec<usize>> {
        let mut selected = BTreeMap::<_, Vec<_>>::new();
        for row in rows {
            for &(instance, ordinal) in self.row_contributions(row) {
                selected.entry(instance).or_default().push(ordinal);
            }
        }
        for ordinals in selected.values_mut() {
            ordinals.sort_unstable();
            ordinals.dedup();
        }
        selected
    }
    /// Compiled derivative ceiling.
    pub fn available_order(&self) -> DerivativeOrder {
        self.available
    }
    /// Complete numerical derivative ceiling; selected entries/actions do not promote it.
    pub fn order(&self) -> DerivativeOrder {
        if self.numerical_supports.is_some() {
            DerivativeOrder::Value
        } else {
            self.order
        }
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
    /// Exact numerical artifact supports; complete structural support remains independent.
    pub fn numerical_supports(&self) -> &[Arc<PreparedSupport>] {
        self.numerical_supports.as_deref().unwrap_or(&self.supports)
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
        for group in self
            .entries
            .iter()
            .flat_map(|entries| entries.groups.iter())
        {
            bytes = bytes
                .checked_add(
                    EvalBindings::allocation_bound(
                        self.structure.instances()[group.instance].slots.len(),
                    )
                    .ok_or_else(overflow)?,
                )
                .ok_or_else(overflow)?;
        }
        bytes = bytes
            .checked_add(
                self.entries
                    .as_ref()
                    .map_or(0, |entries| entries.provider_visits)
                    .checked_mul(512)
                    .ok_or_else(overflow)?,
            )
            .ok_or_else(overflow)?;
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
                || program.product_kind() != request.product
                // Cached compact programs can outlive their weak original body.
                // Compare the complete selected mathematics admitted by this plan;
                // its original bodies remain authoritative for independent checks.
                || support != self.numerical_supports()[index].as_ref()
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
        let mut fixed: Vec<_> = self
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
        for group in self
            .entries
            .iter()
            .flat_map(|entries| entries.groups.iter())
        {
            let binding = &self.structure.instances()[group.instance];
            fixed.extend(
                programs[group.request]
                    .input_formals()
                    .iter()
                    .map(|formal| binding.slots[*formal].source())
                    .filter(|id| !coordinates.contains_key(id)),
            );
        }
        fixed.sort_unstable();
        fixed.dedup();
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
        let entry_bindings = self
            .entries
            .iter()
            .flat_map(|entries| entries.groups.iter())
            .map(|group| {
                EvalBindings::new(
                    &self.structure.instances()[group.instance],
                    programs[group.request].input_formals(),
                    &coordinates,
                    &fixed_offsets,
                )
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        let repeated_entry_providers = CaseAssembly::repeated_entry_providers(self, &programs);
        let entry_bytes = self
            .entries
            .iter()
            .flat_map(|entries| entries.groups.iter())
            .zip(&entry_bindings)
            .try_fold(
                size_of::<entries::EntryScratch>(),
                |bytes, (group, bindings)| {
                    bytes
                        .checked_add(programs[group.request].evaluation_cache_bytes())?
                        .checked_add(bindings.len().checked_mul(size_of::<f64>())?)?
                        .checked_add(size_of::<TrialEvaluation>() + size_of::<Vec<f64>>())
                },
            )
            .and_then(|n| {
                n.checked_add(
                    self.jacobian_entries()
                        .len()
                        .checked_mul(size_of::<f64>())?,
                )
            })
            .and_then(|n| {
                n.checked_add(
                    repeated_entry_providers
                        .len()
                        .checked_mul(size_of::<Option<pse_kernels::ProviderLinearizationToken>>())?,
                )
            })
            .ok_or(MathError::Limit("selected entry worker extent"))?;
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
        let projection_bytes = projection::allocation_bound(self, &programs)?;
        let bytes = self
            .instances
            .iter()
            .flat_map(|i| i.groups.values().flatten())
            .try_fold(
                scratch
                    .checked_add(input_bytes)
                    .and_then(|n| n.checked_add(entry_bytes))
                    .and_then(|n| n.checked_add(projection_bytes))
                    .ok_or(MathError::Limit("case worker bytes"))?,
                |n, g| n.checked_add(programs[g.request].evaluation_cache_bytes()),
            )
            .and_then(|n| n.checked_add(self.jacobian.retained_bytes()))
            .and_then(|n| n.checked_add(self.hessian.retained_bytes()))
            .and_then(|n| {
                n.checked_add(
                    (self.rows.len()
                        + self.columns.len()
                        + self.rows.len().max(self.columns.len()))
                    .checked_mul(size_of::<f64>())?,
                )
            })
            .and_then(|n| {
                n.checked_add(
                    self.instances
                        .len()
                        .checked_mul(size_of::<EnumMap<EvaluationDemand, Option<GroupCache>>>())?,
                )
            })
            .and_then(|n| n.checked_add(programs.len().checked_mul(size_of::<Worker>())?))
            .and_then(|n| n.checked_add(size_of::<AssemblyCompletion>()))
            .and_then(|n| {
                n.checked_add(
                    self.columns
                        .len()
                        .checked_mul(2)?
                        .checked_add(self.rows.len())?
                        .checked_add(2)?
                        .checked_mul(size_of::<u64>())?,
                )
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
            entry_bindings,
            repeated_entry_providers,
        })
    }
    /// Compile a standalone plan. Runtime consumers use compiler-issued artifact requests.
    pub fn compile(
        self: &Arc<Self>,
        optimization: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<CaseAssembly, MathError> {
        let programs = self
            .numerical_supports()
            .iter()
            .zip(self.requests.iter())
            .map(|(support, request)| {
                support
                    .compile_product(request.product, optimization, limits, scope)
                    .map(Arc::new)
            })
            .collect::<Result<_, _>>()?;
        self.assemble(programs)
    }
}
struct CasePatterns {
    jacobian: AssemblyMatrix,
    hessian: AssemblyMatrix,
    jacobian_terms: Vec<Term>,
    hessian_terms: Vec<HessianTerm>,
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
        let offsets = instance
            .coordinates
            .iter()
            .enumerate()
            .map(|(i, formal)| (formal.get(), i))
            .collect::<BTreeMap<_, _>>();
        let first_layout = instance.groups[EvaluationDemand::Constraints]
            .as_ref()
            .map(|group| crate::execution::FirstLayout::from_support(&supports[group.request]));
        for contribution in &binding.contributions {
            if order >= DerivativeOrder::First
                && let Target::Row(row) = contribution.target
            {
                let first = support
                    .and_then(|s| s.first_for_output(contribution.output))
                    .ok_or_else(|| {
                        MathError::Contract("missing selected First incidence".into())
                    })?;
                let result_row = instance.groups[EvaluationDemand::Constraints]
                    .as_ref()
                    .and_then(|group| group.outputs.binary_search(&contribution.output).ok())
                    .ok_or_else(|| {
                        MathError::Contract("missing prepared constraint output".into())
                    })?;
                for formal in first {
                    support_visits = support_visits
                        .checked_add(1)
                        .ok_or(MathError::Limit("support contribution count"))?;
                    let Some(&i) = offsets.get(formal) else {
                        continue;
                    };
                    contribution_allowance(jp.len(), hp.len(), limits)?;
                    jp.push(Entry::new(rows[&row], instance.columns[i]));
                    jt.push(Term {
                        instance: index,
                        output: contribution.output,
                        result_offset: first_layout
                            .as_ref()
                            .and_then(|layout| layout.offset(result_row, i))
                            .ok_or_else(|| {
                                MathError::Contract("missing compact Jacobian address".into())
                            })?,
                        i,
                        j: 0,
                        scale: contribution.scale * binding.slots[*formal].scale(),
                        target: contribution.target,
                    });
                }
            }
        }
        if order >= DerivativeOrder::Second
            && let Some(support) = support
        {
            let mut union = BTreeSet::new();
            for &(a, b) in support.support().second.iter().flatten() {
                support_visits = support_visits
                    .checked_add(1)
                    .ok_or(MathError::Limit("support contribution count"))?;
                let (Some(&i), Some(&j)) = (offsets.get(&a), offsets.get(&b)) else {
                    continue;
                };
                union.insert((i.min(j), i.max(j)));
            }
            let mut ordered = Vec::new();
            for (component, &(i, j)) in union.iter().enumerate() {
                if instance.columns[i] >= instance.columns[j] {
                    ordered.push((i, j, component));
                }
                if i != j && instance.columns[j] >= instance.columns[i] {
                    ordered.push((j, i, component));
                }
            }
            ordered.sort_unstable();
            for (i, j, component) in ordered {
                contribution_allowance(jp.len(), hp.len(), limits)?;
                hp.push(Entry::new(instance.columns[i], instance.columns[j]));
                ht.push(HessianTerm {
                    instance: index,
                    component,
                    i,
                    j,
                    scale: binding.slots[instance.coordinates[i].get()].scale()
                        * binding.slots[instance.coordinates[j].get()].scale(),
                });
            }
        }
    }
    if let Some(context) = pse_diagnostics::observation::current() {
        context.count(
            "math.assembly.support_candidates",
            "support-entries",
            support_visits,
        );
        context.count(
            "math.assembly.emitted_addends",
            "addends",
            (jp.len() + hp.len()) as u64,
        );
    }
    Ok(CasePatterns {
        jacobian: AssemblyMatrix::new(rows.len(), columns, &jp, limits.native_index)?,
        hessian: AssemblyMatrix::lower_hessian(columns, &hp, limits.native_index)?,
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
    /// Mathematical product, excluding numerical runtime weight/direction bindings.
    pub product: ProductKind,
}
#[derive(Clone, Debug)]
enum ProductTarget {
    Objective,
    Row(usize),
}
#[derive(Clone, Debug)]
struct ProductWeight {
    output: usize,
    target: ProductTarget,
    scale: f64,
}
/// A structural plan bound to immutable numeric programs.
#[derive(Clone, Debug)]
pub struct CaseAssembly {
    plan: Arc<CasePlan>,
    programs: Vec<Arc<CompiledBody>>,
    numeric_worker_bytes: usize,
    bindings: Vec<EnumMap<EvaluationDemand, Option<EvalBindings>>>,
    fixed: Vec<SemanticId>,
    entry_bindings: Vec<EvalBindings>,
    repeated_entry_providers: Vec<entries::RepeatedProvider>,
}

/// Frozen numeric inputs, rebound independently of an immutable operation layout.
#[derive(Debug)]
pub struct BoundValues {
    assembly: Arc<CaseAssembly>,
    fixed: Vec<f64>,
    revision: u64,
}
fn fixed_revision() -> Result<u64, MathError> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.try_update(Ordering::Relaxed, Ordering::Relaxed, |revision| {
        revision.checked_add(1)
    })
    .map_err(|_| MathError::Limit("fixed value revision"))
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
        if self
            .fixed
            .iter()
            .zip(reads)
            .any(|(value, &read)| value.to_bits() != source[read].to_bits())
        {
            self.revision = fixed_revision()?;
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
        if updates.iter().any(|(id, value)| {
            self.assembly
                .fixed
                .binary_search(id)
                .is_ok_and(|index| self.fixed[index].to_bits() != value.to_bits())
        }) {
            self.revision = fixed_revision()?;
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
/// Borrowed original selectors of one physical occurrence's prepared demand.
#[derive(Clone, Copy, Debug)]
pub struct OccurrenceDemand<'a> {
    /// Index into CaseStructure::instances, whose identity and checked members remain authoritative.
    pub instance: usize,
    /// Actual consumed operation category.
    pub demand: EvaluationDemand,
    /// Shared mathematical template index.
    pub template: usize,
    /// Original output ordinals, in the prepared product's order.
    pub outputs: &'a [usize],
    /// Original formal derivative axes, preserving local independence and aliases.
    pub coordinates: &'a [usize],
}
impl CaseAssembly {
    /// Use a validated receiving plan shell while retaining these exact compiled programs.
    pub fn receive_source_context(&self, plan: Arc<CasePlan>) -> Result<Self, MathError> {
        if !self.plan.structure.same_numerical(&plan.structure)
            || self.plan.allocation_identity()[1..] != plan.allocation_identity()[1..]
            || self.plan.order != plan.order
            || self.plan.available != plan.available
            || self.plan.actions != plan.actions
            || self.plan.limits != plan.limits
            || !Arc::ptr_eq(&self.plan.row_contributions, &plan.row_contributions)
            || !Arc::ptr_eq(&self.plan.ownership, &plan.ownership)
            || !Arc::ptr_eq(&self.plan.product_weights, &plan.product_weights)
            || self.plan.bodies.len() != plan.bodies.len()
            || !self
                .plan
                .bodies
                .iter()
                .zip(&plan.bodies)
                .all(|((a, body), (b, received))| a == b && Arc::ptr_eq(body, received))
        {
            return Err(MathError::Contract(
                "receiving assembly is not the same numerical plan".into(),
            ));
        }
        let mut received = self.clone();
        received.plan = plan;
        Ok(received)
    }
    /// Conservative copied assembly metadata; all numerical artifacts retain their own owners.
    pub fn source_context_allocation_bound(&self) -> Result<usize, MathError> {
        self.plan
            .operation_layout_allocation_bound()?
            .checked_add(size_of::<Self>() + 4096)
            .ok_or(MathError::Limit("receiving assembly source context extent"))
    }

    /// Borrow occurrence/template selectors without diagnostic admission or support copies.
    pub fn occurrence_demands(&self) -> impl Iterator<Item = OccurrenceDemand<'_>> {
        let requests = &self.requests;
        self.instances
            .iter()
            .enumerate()
            .flat_map(move |(instance, local)| {
                local.groups.iter().filter_map(move |(demand, group)| {
                    let group = group.as_ref()?;
                    let request = &requests[group.request];
                    Some(OccurrenceDemand {
                        instance,
                        demand,
                        template: group.request,
                        outputs: &request.outputs,
                        coordinates: &request.coordinates,
                    })
                })
            })
    }
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
            revision: fixed_revision()?,
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
            revision: fixed_revision()?,
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
        providers: pse_kernels::ProviderBindings,
        cancel: pse_diagnostics::Cancellation,
    ) -> CaseWorker {
        self.worker_scoped(providers, pse_kernels::ExecutionScope::new(cancel, None))
    }
    /// Attach the original enclosing deadline to every primitive and nested callback.
    pub fn worker_scoped(
        self: &Arc<Self>,
        providers: pse_kernels::ProviderBindings,
        scope: pse_kernels::ExecutionScope,
    ) -> CaseWorker {
        let groups = self
            .instances
            .iter()
            .enumerate()
            .map(|(index, i)| {
                EnumMap::from_fn(|demand| {
                    i.groups[demand].as_ref().map(|group| {
                        let inputs = self.bindings[index][demand]
                            .as_ref()
                            .map_or(0, EvalBindings::len);
                        let program = &self.programs[group.request];
                        let weighted = matches!(
                            program.product_kind(),
                            ProductKind::WeightedHessian
                                | ProductKind::WeightedHessianAction
                                | ProductKind::TransposeJacobianAction
                        );
                        let directed = matches!(
                            program.product_kind(),
                            ProductKind::JacobianAction | ProductKind::WeightedHessianAction
                        );
                        let weights = if weighted { group.outputs.len() } else { 0 };
                        let directions = if directed { i.coordinates.len() } else { 0 };
                        GroupCache {
                            inputs: vec![f64::NAN; inputs],
                            bits: vec![0; inputs + weights + directions],
                            weights: vec![f64::NAN; weights],
                            direction: vec![f64::NAN; directions],
                            product: vec![f64::NAN; program.product_len()],
                            spare_product: vec![f64::NAN; program.product_len()],
                            cache: None,
                            visible: false,
                            result: TrialEvaluation::default(),
                            spare: TrialEvaluation::default(),
                        }
                    })
                })
            })
            .collect();
        CaseWorker {
            entries: entries::EntryScratch::new(self),
            instance_reuse: InstanceReuse::new(self.requests.len()),
            assembly: Arc::clone(self),
            groups,
            projections: Vec::new(),
            workers: self
                .programs
                .iter()
                .map(|p| p.worker_scoped(scope.clone()))
                .collect(),
            providers,
            cancel: scope.cancellation().clone(),
            scope,
            jacobian: (self.order() >= DerivativeOrder::First)
                .then(|| self.jacobian.as_ref().clone()),
            hessian: Some(self.hessian.as_ref().clone()),
            constraint_values: vec![f64::NAN; self.rows.len()],
            gradient_values: vec![f64::NAN; self.columns.len()],
            action_values: vec![f64::NAN; self.rows.len().max(self.columns.len())],
            recycler: None,
            observation: None,
            observation_operation: None,
            completion: AssemblyCompletion::new(self.columns.len(), self.rows.len()),
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
        mut providers: pse_kernels::ProviderBindings,
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
            projections,
            jacobian,
            hessian,
            constraint_values,
            gradient_values,
            action_values,
            entries,
            ..
        } = scratch;
        for worker in &mut workers {
            worker.set_scope(scope.clone());
            worker.prepare_providers(&mut providers)?;
        }
        Ok(CaseWorker {
            entries,
            instance_reuse: InstanceReuse::new(self.requests.len()),
            scope: scope.clone(),
            assembly,
            groups,
            workers,
            projections,
            providers,
            cancel: scope.cancellation().clone(),
            jacobian,
            hessian,
            constraint_values,
            gradient_values,
            action_values,
            recycler: None,
            observation: None,
            observation_operation: None,
            completion: AssemblyCompletion::new(self.columns.len(), self.rows.len()),
        })
    }
}
#[derive(Debug)]
struct GroupCache {
    inputs: Vec<f64>,
    bits: Vec<u64>,
    direction: Vec<f64>,
    cache: Option<DerivativeOrder>,
    visible: bool,
    weights: Vec<f64>,
    product: Vec<f64>,
    spare_product: Vec<f64>,
    result: TrialEvaluation,
    spare: TrialEvaluation,
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
    /// Hits served by a richer order or an explicitly mapped compatible demand.
    pub cache_projections: u64,
    /// No earlier result existed for this group.
    pub empty_misses: u64,
    /// An earlier trial refusal or explicit invalidation cleared this group.
    pub invalidated_misses: u64,
    /// A complete cached result consumed different input, weight or direction bits.
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
    fn projected(&mut self, achieved: DerivativeOrder) {
        Self::add(&mut self.cache_hits, &mut self.overflowed);
        Self::add(&mut self.cache_projections, &mut self.overflowed);
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
    /// Selected local demand; compatible cross-demand projections are counted explicitly.
    pub demand: EvaluationDemand,
    /// Order actually requested by the caller.
    pub requested: DerivativeOrder,
    /// Complete attempt-local populations, including explicit overflow.
    pub counts: &'a DemandCounts,
}
/// Isolated attempt scratch. Failed evaluation clears the failed cache and never publishes output.
#[derive(Debug)]
pub struct CaseWorker {
    completion: AssemblyCompletion,
    scope: pse_kernels::ExecutionScope,
    assembly: Arc<CaseAssembly>,
    groups: Vec<EnumMap<EvaluationDemand, Option<GroupCache>>>,
    // Evaluation is serial under &mut self; scratch is shared only inside this attempt.
    workers: Vec<Worker>,
    projections: Vec<ProjectionRelationship>,
    instance_reuse: InstanceReuse,
    providers: pse_kernels::ProviderBindings,
    cancel: Arc<AtomicBool>,
    jacobian: Option<AssemblyMatrix>,
    hessian: Option<AssemblyMatrix>,
    constraint_values: Vec<f64>,
    gradient_values: Vec<f64>,
    action_values: Vec<f64>,
    entries: entries::EntryScratch,
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
    projections: Vec<ProjectionRelationship>,
    jacobian: Option<AssemblyMatrix>,
    hessian: Option<AssemblyMatrix>,
    constraint_values: Vec<f64>,
    gradient_values: Vec<f64>,
    action_values: Vec<f64>,
    entries: entries::EntryScratch,
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
                group.spare.clear();
                group.visible = false;
                group.weights.fill(f64::NAN);
                group.product.fill(f64::NAN);
                group.spare_product.fill(f64::NAN);
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
        self.action_values.fill(f64::NAN);
        self.entries.clear();
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
        drop(std::mem::take(&mut self.providers));
        self.invalidate_trial();
        for worker in &mut self.workers {
            worker.detach_controls();
        }
        (recycler.0)(CaseScratch {
            assembly: self.assembly.clone(),
            groups: std::mem::take(&mut self.groups),
            workers: std::mem::take(&mut self.workers),
            projections: std::mem::take(&mut self.projections),
            jacobian: self.jacobian.take(),
            hessian: self.hessian.take(),
            constraint_values: std::mem::take(&mut self.constraint_values),
            gradient_values: std::mem::take(&mut self.gradient_values),
            action_values: std::mem::take(&mut self.action_values),
            entries: std::mem::take(&mut self.entries),
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
    /// Prepare the actual bound request set before native callbacks are attached.
    pub fn prepare_providers(&mut self) -> Result<(), MathError> {
        self.scope.check().map_err(crate::error::scope_error)?;
        self.check_entry_providers()?;
        for worker in &mut self.workers {
            worker.prepare_providers(&mut self.providers)?;
        }
        Ok(())
    }
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
    #[cfg(test)]
    pub(crate) fn assembled_counts(&self) -> [u64; 7] {
        std::array::from_fn(|index| self.completion.assemblies.as_slice()[index])
    }
    #[cfg(test)]
    pub(crate) fn local_evaluation_counts(&self) -> [u64; 5] {
        std::array::from_fn(|index| self.completion.evaluations.as_slice()[index])
    }
    /// Shared cooperative cancellation owner for bounded library work on this admitted worker.
    pub fn cancellation(&self) -> &Arc<AtomicBool> {
        &self.cancel
    }
    /// Original admitted execution scope, including its notifying stop owner and clock.
    pub fn execution_scope(&self) -> &pse_kernels::ExecutionScope {
        &self.scope
    }
    /// Immutable products shared by the attempt.
    pub fn assembly(&self) -> &Arc<CaseAssembly> {
        &self.assembly
    }
    /// Clear numerical results and observation witnesses after a refused trial.
    pub fn invalidate_trial(&mut self) {
        self.entries.clear();
        self.completion.invalidate();
        if let Some(observation) = &mut self.observation {
            observation.invalidate();
        }
        for groups in &mut self.groups {
            for group in groups.values_mut().flatten() {
                group.cache = None;
                group.result.clear();
                group.spare.clear();
                group.visible = false;
                group.weights.fill(f64::NAN);
                group.product.fill(f64::NAN);
                group.spare_product.fill(f64::NAN);
            }
        }
    }
    fn refuse_trial(&mut self, cause: &MathError) {
        if independent_completion_survives(cause) {
            for groups in &mut self.groups {
                for group in groups.values_mut().flatten() {
                    group.visible = false;
                    group.spare.clear();
                    group.spare_product.fill(f64::NAN);
                }
            }
        } else {
            self.invalidate_trial();
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
        if let Err(cause) = &result {
            self.refuse_trial(cause);
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
        if order > self.assembly.order()
            && matches!(
                demand,
                EvaluationDemand::Objective | EvaluationDemand::Constraints | EvaluationDemand::All
            )
        {
            return Err(MathError::Contract(
                "full numerical derivative demand was not prepared".into(),
            ));
        }
        self.instance_reuse.begin();
        for (i, binding) in self.assembly.structure.instances().iter().enumerate() {
            self.scope.check().map_err(crate::error::scope_error)?;
            let Some(worker) = &mut self.groups[i][demand] else {
                continue;
            };
            let context = |cause| MathError::Instance {
                instance: binding.instance,
                occurrences: binding.occurrences.as_ref().clone(),
                checked_sources: binding.checked_sources.as_ref().clone(),
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
                worker.visible = false;
                return Err(context(cause));
            }
            let same = worker
                .bits
                .iter()
                .zip(&worker.inputs)
                .all(|(bits, value)| *bits == value.to_bits());
            // A numerical completion does not authorize skipping current provider
            // effects. The provider owner may reuse its own qualified products.
            let pure = self.assembly.programs[group.request]
                .census()
                .all(|order| order.provider_stages == 0);
            if let Some(achieved) = worker
                .cache
                .filter(|achieved| *achieved >= order && same && pure)
            {
                worker.visible = true;
                if let Some(observation) = &mut self.observation {
                    observation.hit(i, demand, order, achieved);
                }
                self.instance_reuse.record(group.request, i);
                continue;
            }
            if let Some(achieved) = self
                .instance_reuse
                .project(
                    &mut self.groups,
                    i,
                    group.request,
                    demand,
                    order,
                    &self.assembly.programs[group.request],
                )
                .map_err(context)?
            {
                self.completion.replaced(demand)?;
                self.instance_reuse.record(group.request, i);
                if let Some(observation) = &mut self.observation {
                    observation.projected(i, demand, order, achieved);
                }
                continue;
            }
            match project_compatible(
                &mut self.groups[i],
                demand,
                order,
                &self.assembly,
                i,
                &mut self.projections,
            ) {
                Ok(Some(achieved)) => {
                    self.completion.replaced(demand)?;
                    self.instance_reuse.record(group.request, i);
                    if let Some(observation) = &mut self.observation {
                        observation.projected(i, demand, order, achieved);
                    }
                    continue;
                }
                Ok(None) => {}
                Err(cause) => {
                    if let Some(observation) = &mut self.observation {
                        observation.failed(i, demand, order);
                    }
                    return Err(context(cause));
                }
            }
            let worker = self.groups[i][demand]
                .as_mut()
                .ok_or_else(|| MathError::Contract("missing projected group".into()))?;
            if let Some(observation) = &mut self.observation {
                observation.miss(i, demand, order, worker.cache, same);
            }
            worker.visible = false;
            #[cfg(test)]
            {
                self.completion.evaluations[demand] += 1;
            }
            let actual_evaluation_started = self
                .observation
                .as_ref()
                .and_then(|observation| observation.evaluation_started());
            let evaluated = self.workers[group.request]
                .evaluate_trial_into(
                    &worker.inputs,
                    order,
                    &mut self.providers,
                    &self.cancel,
                    &mut worker.spare,
                )
                .and_then(|()| self.scope.check().map_err(crate::error::scope_error));
            if let Some(observation) = &mut self.observation {
                observation.evaluation_finished(actual_evaluation_started, evaluated.is_ok());
            }
            if let Err(cause) = evaluated {
                if let Some(observation) = &mut self.observation {
                    observation.failed(i, demand, order);
                }
                return Err(context(cause));
            }
            if let Some(observation) = &mut self.observation {
                observation.completed(i, demand, order);
            }
            std::mem::swap(&mut worker.result, &mut worker.spare);
            worker.spare.clear();
            for (bits, value) in worker.bits.iter_mut().zip(&worker.inputs) {
                *bits = value.to_bits();
            }
            worker.cache = Some(order);
            self.completion.replaced(demand)?;
            worker.visible = true;
            self.instance_reuse.record(group.request, i);
        }
        self.scope.check().map_err(crate::error::scope_error)?;
        Ok(())
    }
    fn evaluated_result(
        &self,
        instance: usize,
        demand: EvaluationDemand,
    ) -> Result<&TrialEvaluation, MathError> {
        self.groups[instance][demand]
            .as_ref()
            .filter(|group| group.cache.is_some())
            .map(|group| &group.result)
            .ok_or_else(|| MathError::Contract("unevaluated demand".into()))
    }
    /// Read evidence from the exact last demanded programs; this never evaluates an
    /// inactive branch or an unrelated output just to create observations.
    pub fn applicability_observations(&self) -> Vec<pse_model::applicability::Observation> {
        let mut observations = Vec::new();
        for (instance, groups) in self.groups.iter().enumerate() {
            for group in groups.values().flatten() {
                if group.cache.is_some() && group.visible {
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
        if self.entries.visible {
            for (group, evaluation) in self
                .assembly
                .entries
                .iter()
                .flat_map(|entries| &entries.groups)
                .zip(&self.entries.trials)
            {
                for mut observation in evaluation
                    .applicability
                    .iter()
                    .flat_map(crate::execution::ApplicabilityWitness::hydrate)
                {
                    observation.instance =
                        Some(self.assembly.structure.instances()[group.instance].instance);
                    if !observations.contains(&observation) {
                        observations.push(observation);
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
        if self.completion.reusable(
            AssembledProduct::Objective,
            EvaluationDemand::Objective,
            DerivativeOrder::Value,
        ) {
            return Ok(self.completion.objective);
        }
        self.completion.begin(AssembledProduct::Objective);
        let assembled = (|| {
            let mut value = 0.0;
            for (i, instance) in self.assembly.instances.iter().enumerate() {
                for term in &instance.objective_terms {
                    let result = self.evaluated_result(i, EvaluationDemand::Objective)?;
                    value += term.scale * result.values[term.result_row];
                }
            }
            self.completion.objective = finite(value * self.objective_sign())?;
            Ok(())
        })();
        self.finish_assembly(
            assembled,
            AssembledProduct::Objective,
            EvaluationDemand::Objective,
            DerivativeOrder::Value,
        )?;
        Ok(self.completion.objective)
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
        if self.completion.reusable(
            AssembledProduct::Constraints,
            EvaluationDemand::Constraints,
            DerivativeOrder::Value,
        ) {
            return Ok(&self.constraint_values);
        }
        self.completion.begin(AssembledProduct::Constraints);
        let assembled = (|| {
            self.constraint_values.fill(0.0);
            for (i, instance) in self.assembly.instances.iter().enumerate() {
                for term in &instance.constraint_terms {
                    let result = self.evaluated_result(i, EvaluationDemand::Constraints)?;
                    let addend = term.scale * result.values[term.result_row];
                    self.constraint_values[term.destination] += addend;
                }
            }
            for &value in &self.constraint_values {
                finite(value)?;
            }
            Ok(())
        })();
        self.finish_assembly(
            assembled,
            AssembledProduct::Constraints,
            EvaluationDemand::Constraints,
            DerivativeOrder::Value,
        )?;
        Ok(&self.constraint_values)
    }
    /// Retain independently visible source terms from the most recent constraint evaluation.
    pub fn constraint_sources(&self) -> Result<Vec<OutputValue>, MathError> {
        let mut out = Vec::new();
        for (i, binding) in self.assembly.structure.instances().iter().enumerate() {
            if let Some(group) = &self.assembly.instances[i].groups[EvaluationDemand::Constraints] {
                let evaluation = self.evaluated_result(i, EvaluationDemand::Constraints)?;
                for (row, &output) in group.outputs.iter().enumerate() {
                    out.push(OutputValue {
                        instance: binding.instance,
                        output,
                        value: finite(evaluation.values[row])?,
                    });
                }
            }
        }
        Ok(out)
    }
    /// Objective gradient in free-variable order and minimization orientation.
    pub fn gradient<'a>(&mut self, values: impl Into<EvalInputs<'a>>) -> Result<&[f64], MathError> {
        self.evaluate(
            values.into(),
            EvaluationDemand::Objective,
            DerivativeOrder::First,
        )?;
        if self.completion.reusable(
            AssembledProduct::Gradient,
            EvaluationDemand::Objective,
            DerivativeOrder::First,
        ) {
            return Ok(&self.gradient_values);
        }
        self.completion.begin(AssembledProduct::Gradient);
        let assembled = (|| {
            self.gradient_values.fill(0.0);
            let sign = self.objective_sign();
            for (i, instance) in self.assembly.instances.iter().enumerate() {
                for term in &instance.objective_terms {
                    let result = &self.groups[i][EvaluationDemand::Objective]
                        .as_ref()
                        .filter(|group| group.cache.is_some())
                        .ok_or_else(|| MathError::Contract("unevaluated gradient demand".into()))?
                        .result;
                    for (axis, value) in result.first_entries(term.result_row) {
                        let coordinate = &instance.gradient_coordinates[axis];
                        let addend = sign * term.scale * coordinate.scale * value;
                        self.gradient_values[coordinate.destination] += addend;
                    }
                }
            }
            for &value in &self.gradient_values {
                finite(value)?;
            }
            Ok(())
        })();
        self.finish_assembly(
            assembled,
            AssembledProduct::Gradient,
            EvaluationDemand::Objective,
            DerivativeOrder::First,
        )?;
        Ok(&self.gradient_values)
    }
    /// Refill the canonical constraint Jacobian without rebuilding structure.
    pub fn jacobian<'a>(
        &mut self,
        values: impl Into<EvalInputs<'a>>,
    ) -> Result<&faer::sparse::SparseColMat<usize, f64>, MathError> {
        if self.assembly.order() < DerivativeOrder::First {
            return Err(MathError::Contract(
                "full numerical Jacobian was not prepared".into(),
            ));
        }
        let values = values.into();
        let point = self.trial_point(values)?;
        self.evaluate(
            values,
            EvaluationDemand::Constraints,
            DerivativeOrder::First,
        )?;
        self.completion.jacobian_point = point;
        if self.completion.reusable(
            AssembledProduct::Jacobian,
            EvaluationDemand::Constraints,
            DerivativeOrder::First,
        ) {
            return Ok(self
                .jacobian
                .as_ref()
                .ok_or_else(|| MathError::Contract("detached Jacobian".into()))?
                .matrix());
        }
        self.completion.begin(AssembledProduct::Jacobian);
        let assembled = (|| {
            self.jacobian
                .as_mut()
                .ok_or_else(|| MathError::Contract("detached Jacobian".into()))?
                .clear();
            for (k, t) in self.assembly.jacobian_terms.iter().enumerate() {
                let v = &self.groups[t.instance][EvaluationDemand::Constraints]
                    .as_ref()
                    .filter(|group| group.cache.is_some())
                    .ok_or_else(|| MathError::Contract("unevaluated Jacobian demand".into()))?
                    .result;
                let value = t.scale * v.first[t.result_offset];
                self.jacobian
                    .as_mut()
                    .ok_or_else(|| MathError::Contract("detached Jacobian".into()))?
                    .add(Addend::new(k), value)?;
            }
            Ok(())
        })();
        self.finish_assembly(
            assembled,
            AssembledProduct::Jacobian,
            EvaluationDemand::Constraints,
            DerivativeOrder::First,
        )?;
        self.completion.jacobian_point = point;
        Ok(self
            .jacobian
            .as_ref()
            .ok_or_else(|| MathError::Contract("detached Jacobian".into()))?
            .matrix())
    }
    /// Refill the lower Lagrangian Hessian from actual weighted local products.
    pub fn hessian<'a>(
        &mut self,
        values: impl Into<EvalInputs<'a>>,
        objective_weight: f64,
        multipliers: &[f64],
    ) -> Result<&faer::sparse::SparseColMat<usize, f64>, MathError> {
        self.check_weights(objective_weight, multipliers)?;
        let values = values.into();
        let point = self.trial_point(values)?;
        self.evaluate_bound_product(
            values,
            EvaluationDemand::All,
            objective_weight,
            multipliers,
            None,
        )?;
        self.completion.hessian_point = point;
        if !self.completion.same_weights(objective_weight, multipliers) {
            self.completion.forget(AssembledProduct::Hessian);
        }
        if self.completion.reusable(
            AssembledProduct::Hessian,
            EvaluationDemand::All,
            DerivativeOrder::Second,
        ) {
            return Ok(self
                .hessian
                .as_ref()
                .ok_or_else(|| MathError::Contract("detached Hessian".into()))?
                .matrix());
        }
        self.completion.begin(AssembledProduct::Hessian);
        let assembled = (|| {
            let matrix = self
                .hessian
                .as_mut()
                .ok_or_else(|| MathError::Contract("detached Hessian".into()))?;
            matrix.clear();
            for (index, term) in self.assembly.hessian_terms.iter().enumerate() {
                let group = self.groups[term.instance][EvaluationDemand::All]
                    .as_ref()
                    .filter(|g| g.visible && g.cache.is_some())
                    .ok_or_else(|| MathError::Contract("unevaluated weighted Hessian".into()))?;
                matrix.add(
                    Addend::new(index),
                    term.scale * group.product[term.component],
                )?;
            }
            self.scope.check().map_err(crate::error::scope_error)?;
            Ok(())
        })();
        self.finish_assembly(
            assembled,
            AssembledProduct::Hessian,
            EvaluationDemand::All,
            DerivativeOrder::Second,
        )?;
        self.completion.hessian_point = point;
        self.completion.set_weights(objective_weight, multipliers);
        Ok(self
            .hessian
            .as_ref()
            .ok_or_else(|| MathError::Contract("detached Hessian".into()))?
            .matrix())
    }
    /// Evaluate a weighted Hessian action directly; no full matrix is assembled.
    pub fn hessian_product<'a>(
        &mut self,
        values: impl Into<EvalInputs<'a>>,
        objective_weight: f64,
        multipliers: &[f64],
        direction: &[f64],
        output: &mut [f64],
    ) -> Result<(), MathError> {
        self.check_weights(objective_weight, multipliers)?;
        if direction.len() != self.assembly.columns.len()
            || output.len() != self.assembly.columns.len()
            || direction.iter().any(|v| !v.is_finite())
        {
            return Err(MathError::Contract(
                "prepared Hessian action and finite global dimensions required".into(),
            ));
        }
        let values = values.into();
        let point = self.trial_point(values)?;
        if self.matrix_eligible(
            AssembledProduct::Hessian,
            EvaluationDemand::All,
            DerivativeOrder::Second,
            point,
        ) && self.completion.same_weights(objective_weight, multipliers)
        {
            self.scope.check().map_err(crate::error::scope_error)?;
            if !self.completion.reusable(
                AssembledProduct::HessianAction,
                EvaluationDemand::All,
                DerivativeOrder::Second,
            ) || !self.completion.same_action(point, direction)
            {
                self.completion.begin(AssembledProduct::HessianAction);
                let assembled = self
                    .hessian
                    .as_ref()
                    .ok_or_else(|| MathError::Contract("detached hessian".into()))?
                    .symmetric_product(direction, &mut self.action_values[..output.len()]);
                self.finish_assembly(
                    assembled,
                    AssembledProduct::HessianAction,
                    EvaluationDemand::All,
                    DerivativeOrder::Second,
                )?;
                self.completion.set_action(point, direction);
            }
            for groups in &mut self.groups {
                if let Some(group) = &mut groups[EvaluationDemand::All] {
                    group.visible = true;
                }
            }
            let current = self.scope.check().map_err(crate::error::scope_error);
            if let Err(cause) = &current {
                self.refuse_trial(cause);
            }
            current?;
            output.copy_from_slice(&self.action_values[..output.len()]);
            return Ok(());
        }
        if !self.assembly.has_hessian_actions() {
            return Err(MathError::Contract(
                "no completed compatible matrix or prepared action".into(),
            ));
        }
        self.evaluate_bound_product(
            values,
            EvaluationDemand::HessianAction,
            objective_weight,
            multipliers,
            Some(direction),
        )?;
        if !self.completion.reusable(
            AssembledProduct::HessianAction,
            EvaluationDemand::HessianAction,
            DerivativeOrder::Second,
        ) {
            self.completion.begin(AssembledProduct::HessianAction);
            let assembled = (|| {
                self.action_values[..output.len()].fill(0.0);
                for (index, (local, binding)) in self
                    .assembly
                    .instances
                    .iter()
                    .zip(self.assembly.structure.instances())
                    .enumerate()
                {
                    let Some(cache) = &self.groups[index][EvaluationDemand::HessianAction] else {
                        continue;
                    };
                    for (axis, (&formal, &column)) in
                        local.coordinates.iter().zip(&local.columns).enumerate()
                    {
                        let destination = &mut self.action_values[column.get()];
                        *destination = finite(
                            *destination
                                + binding.slots[formal.get()].scale() * cache.product[axis],
                        )?;
                    }
                }
                self.scope.check().map_err(crate::error::scope_error)?;
                Ok(())
            })();
            self.finish_assembly(
                assembled,
                AssembledProduct::HessianAction,
                EvaluationDemand::HessianAction,
                DerivativeOrder::Second,
            )?;
        }
        output.copy_from_slice(&self.action_values[..output.len()]);
        Ok(())
    }
    /// Evaluate a first action from its admitted directional program.
    pub fn jacobian_product<'a>(
        &mut self,
        values: impl Into<EvalInputs<'a>>,
        direction: &[f64],
        output: &mut [f64],
    ) -> Result<(), MathError> {
        if direction.len() != self.assembly.columns.len()
            || output.len() != self.assembly.rows.len()
            || direction.iter().any(|v| !v.is_finite())
        {
            return Err(MathError::Contract(
                "prepared Jacobian action and finite global dimensions required".into(),
            ));
        }
        let values = values.into();
        let point = self.trial_point(values)?;
        if self.matrix_eligible(
            AssembledProduct::Jacobian,
            EvaluationDemand::Constraints,
            DerivativeOrder::First,
            point,
        ) {
            self.scope.check().map_err(crate::error::scope_error)?;
            if !self.completion.reusable(
                AssembledProduct::JacobianAction,
                EvaluationDemand::Constraints,
                DerivativeOrder::First,
            ) || !self.completion.same_action(point, direction)
            {
                self.completion.begin(AssembledProduct::JacobianAction);
                let assembled = self
                    .jacobian
                    .as_ref()
                    .ok_or_else(|| MathError::Contract("detached jacobian".into()))?
                    .product(direction, &mut self.action_values[..output.len()]);
                self.finish_assembly(
                    assembled,
                    AssembledProduct::JacobianAction,
                    EvaluationDemand::Constraints,
                    DerivativeOrder::First,
                )?;
                self.completion.set_action(point, direction);
            }
            for groups in &mut self.groups {
                if let Some(group) = &mut groups[EvaluationDemand::Constraints] {
                    group.visible = true;
                }
            }
            let current = self.scope.check().map_err(crate::error::scope_error);
            if let Err(cause) = &current {
                self.refuse_trial(cause);
            }
            current?;
            output.copy_from_slice(&self.action_values[..output.len()]);
            return Ok(());
        }
        if !self.assembly.has_directional_actions() {
            return Err(MathError::Contract(
                "no completed compatible matrix or prepared action".into(),
            ));
        }
        self.evaluate_bound_product(
            values,
            EvaluationDemand::Directional,
            0.0,
            &[],
            Some(direction),
        )?;
        if !self.completion.reusable(
            AssembledProduct::JacobianAction,
            EvaluationDemand::Directional,
            DerivativeOrder::First,
        ) {
            self.completion.begin(AssembledProduct::JacobianAction);
            let assembled = (|| {
                self.action_values[..output.len()].fill(0.0);
                for (index, weights) in self.assembly.product_weights.iter().enumerate() {
                    let Some(cache) = &self.groups[index][EvaluationDemand::Directional] else {
                        continue;
                    };
                    for weight in &weights[EvaluationDemand::Directional] {
                        if let ProductTarget::Row(row) = weight.target {
                            self.action_values[row] = finite(
                                self.action_values[row]
                                    + weight.scale * cache.product[weight.output],
                            )?;
                        }
                    }
                }
                self.scope.check().map_err(crate::error::scope_error)?;
                Ok(())
            })();
            self.finish_assembly(
                assembled,
                AssembledProduct::JacobianAction,
                EvaluationDemand::Directional,
                DerivativeOrder::First,
            )?;
        }
        output.copy_from_slice(&self.action_values[..output.len()]);
        Ok(())
    }
    /// Pull back constraint weights without constructing unconsumed partials.
    /// A completed compatible pure matrix is consumed when already available.
    pub fn jacobian_transpose_product<'a>(
        &mut self,
        values: impl Into<EvalInputs<'a>>,
        weights: &[f64],
        output: &mut [f64],
    ) -> Result<(), MathError> {
        if weights.len() != self.assembly.rows.len()
            || output.len() != self.assembly.columns.len()
            || weights.iter().any(|v| !v.is_finite())
        {
            return Err(MathError::Contract(
                "finite constraint cotangent and global dimensions required".into(),
            ));
        }
        let values = values.into();
        let point = self.trial_point(values)?;
        // Transpose products share the action staging buffer, so invalidate any
        // previous action receipt before writing even when using a completed matrix.
        self.completion
            .begin(AssembledProduct::TransposeJacobianAction);
        let matrix = self.matrix_eligible(
            AssembledProduct::Jacobian,
            EvaluationDemand::Constraints,
            DerivativeOrder::First,
            point,
        );
        let demand = if matrix {
            EvaluationDemand::Constraints
        } else {
            EvaluationDemand::TransposeDirectional
        };
        if !matrix {
            if !self.assembly.has_transpose_actions() {
                return Err(MathError::Contract(
                    "no completed compatible matrix or prepared transpose action".into(),
                ));
            }
            self.evaluate_bound_product(values, demand, 0., weights, None)?;
        }
        let assembled = (|| {
            if matrix {
                self.jacobian
                    .as_ref()
                    .ok_or_else(|| MathError::Contract("detached jacobian".into()))?
                    .transpose_product(weights, &mut self.action_values[..output.len()])?;
            } else {
                self.action_values[..output.len()].fill(0.);
                for (index, (local, binding)) in self
                    .assembly
                    .instances
                    .iter()
                    .zip(self.assembly.structure.instances())
                    .enumerate()
                {
                    let Some(cache) = &self.groups[index][demand] else {
                        continue;
                    };
                    for (axis, (&formal, &column)) in
                        local.coordinates.iter().zip(&local.columns).enumerate()
                    {
                        let destination = &mut self.action_values[column.get()];
                        *destination = finite(
                            *destination
                                + binding.slots[formal.get()].scale() * cache.product[axis],
                        )?;
                    }
                }
            }
            self.scope.check().map_err(crate::error::scope_error)
        })();
        self.finish_assembly(
            assembled,
            AssembledProduct::TransposeJacobianAction,
            demand,
            DerivativeOrder::First,
        )?;
        output.copy_from_slice(&self.action_values[..output.len()]);
        Ok(())
    }
    fn trial_point(&mut self, values: EvalInputs<'_>) -> Result<Option<u64>, MathError> {
        let result = values
            .check(&self.assembly)
            .and_then(|()| self.completion.point(values));
        if let Err(cause) = &result {
            self.refuse_trial(cause);
        }
        result
    }
    fn matrix_eligible(
        &self,
        product: AssembledProduct,
        demand: EvaluationDemand,
        order: DerivativeOrder,
        point: Option<u64>,
    ) -> bool {
        let matrix_point = match product {
            AssembledProduct::Jacobian => self.completion.jacobian_point,
            AssembledProduct::Hessian => self.completion.hessian_point,
            _ => return false,
        };
        // Provider completions alone do not authorize skipping current effects.
        // Until an enclosing owner admits all current linearization premises,
        // provider-bearing matrices retain the prepared action route.
        point.is_some()
            && matrix_point == point
            && self.completion.reusable(product, demand, order)
            && self
                .assembly
                .instances
                .iter()
                .filter_map(|instance| instance.groups[demand].as_ref())
                .all(|group| {
                    self.assembly.programs[group.request]
                        .census()
                        .all(|order| order.provider_stages == 0)
                })
    }
    fn finish_assembly(
        &mut self,
        assembled: Result<(), MathError>,
        product: AssembledProduct,
        demand: EvaluationDemand,
        order: DerivativeOrder,
    ) -> Result<(), MathError> {
        let result = assembled.and_then(|()| self.scope.check().map_err(crate::error::scope_error));
        if let Err(cause) = &result {
            self.refuse_trial(cause);
        } else {
            self.completion.completed(product, demand, order);
        }
        result
    }
    fn check_weights(&self, objective_weight: f64, multipliers: &[f64]) -> Result<(), MathError> {
        if self.assembly.order < DerivativeOrder::Second
            || !objective_weight.is_finite()
            || multipliers.len() != self.assembly.rows.len()
            || multipliers.iter().any(|v| !v.is_finite())
        {
            return Err(MathError::Contract(
                "finite Lagrangian coefficients and Second demand required".into(),
            ));
        }
        Ok(())
    }
    fn evaluate_bound_product(
        &mut self,
        values: EvalInputs<'_>,
        demand: EvaluationDemand,
        objective_weight: f64,
        multipliers: &[f64],
        direction: Option<&[f64]>,
    ) -> Result<(), MathError> {
        self.bind_observation();
        let _entered = self
            .observation
            .as_ref()
            .map(|o| o.producer.context().enter());
        let _phase = self.observation.as_ref().map(|o| o.phase(demand));
        let order = if matches!(
            demand,
            EvaluationDemand::Directional | EvaluationDemand::TransposeDirectional
        ) {
            DerivativeOrder::First
        } else {
            DerivativeOrder::Second
        };
        let evaluated = self.evaluate_bound_product_inner(
            values,
            demand,
            order,
            objective_weight,
            multipliers,
            direction,
        );
        if let Err(cause) = &evaluated {
            self.refuse_trial(cause);
        }
        if let Some(observation) = &mut self.observation {
            observation.publish(&self.assembly, demand, order);
        }
        evaluated
    }
    fn evaluate_bound_product_inner(
        &mut self,
        values: EvalInputs<'_>,
        demand: EvaluationDemand,
        order: DerivativeOrder,
        objective_weight: f64,
        multipliers: &[f64],
        direction: Option<&[f64]>,
    ) -> Result<(), MathError> {
        self.scope.check().map_err(crate::error::scope_error)?;
        values.check(&self.assembly)?;
        let objective_weight = finite(objective_weight * self.objective_sign())?;
        for (index, (local, binding)) in self
            .assembly
            .instances
            .iter()
            .zip(self.assembly.structure.instances())
            .enumerate()
        {
            let Some(group) = &local.groups[demand] else {
                continue;
            };
            let cache = self.groups[index][demand]
                .as_mut()
                .ok_or_else(|| MathError::Contract("missing product cache".into()))?;
            let inputs = self.assembly.bindings[index][demand]
                .as_ref()
                .ok_or_else(|| MathError::Contract("missing product gather map".into()))?;
            if let Some(observation) = &mut self.observation {
                observation.called(index, demand, order);
            }
            let evaluated = (|| {
                values.fill(inputs, &mut cache.inputs)?;
                if !cache.weights.is_empty() {
                    cache.weights.fill(0.0);
                    for weight in &self.assembly.product_weights[index][demand] {
                        let coefficient = match weight.target {
                            ProductTarget::Objective => objective_weight,
                            ProductTarget::Row(row) => multipliers[row],
                        };
                        cache.weights[weight.output] =
                            finite(cache.weights[weight.output] + weight.scale * coefficient)?;
                    }
                }
                if let Some(direction) = direction {
                    for (axis, (&formal, &column)) in
                        local.coordinates.iter().zip(&local.columns).enumerate()
                    {
                        cache.direction[axis] =
                            finite(binding.slots[formal.get()].scale() * direction[column.get()])?;
                    }
                }
                let same = cache
                    .bits
                    .iter()
                    .zip(
                        cache
                            .inputs
                            .iter()
                            .chain(&cache.weights)
                            .chain(&cache.direction),
                    )
                    .all(|(bits, value)| *bits == value.to_bits());
                let pure = self.assembly.programs[group.request]
                    .census()
                    .all(|order| order.provider_stages == 0);
                if cache.cache.is_some() && same && pure {
                    cache.visible = true;
                    if let Some(observation) = &mut self.observation {
                        observation.hit(index, demand, order, order);
                    }
                    return Ok(());
                }
                if let Some(observation) = &mut self.observation {
                    observation.miss(index, demand, order, cache.cache, same);
                }
                cache.visible = false;
                let product = match self.assembly.requests[group.request].product {
                    ProductKind::JacobianAction => ProductValues::JacobianAction {
                        direction: &cache.direction,
                    },
                    ProductKind::TransposeJacobianAction => {
                        ProductValues::TransposeJacobianAction {
                            weights: &cache.weights,
                        }
                    }
                    ProductKind::WeightedHessian => ProductValues::WeightedHessian {
                        weights: &cache.weights,
                    },
                    ProductKind::WeightedHessianAction => ProductValues::WeightedHessianAction {
                        weights: &cache.weights,
                        direction: &cache.direction,
                    },
                    ProductKind::SelectedPartials => {
                        return Err(MathError::Contract(
                            "non-product demand in product execution".into(),
                        ));
                    }
                };
                #[cfg(test)]
                {
                    self.completion.evaluations[demand] += 1;
                }
                let actual_evaluation_started = self
                    .observation
                    .as_ref()
                    .and_then(|observation| observation.evaluation_started());
                let actual_evaluated = self.workers[group.request]
                    .evaluate_product_trial_into(
                        &cache.inputs,
                        product,
                        &mut self.providers,
                        &self.cancel,
                        &mut cache.spare_product,
                        &mut cache.spare,
                    )
                    .and_then(|()| self.scope.check().map_err(crate::error::scope_error));
                if let Some(observation) = &mut self.observation {
                    observation
                        .evaluation_finished(actual_evaluation_started, actual_evaluated.is_ok());
                }
                actual_evaluated?;
                std::mem::swap(&mut cache.product, &mut cache.spare_product);
                std::mem::swap(&mut cache.result, &mut cache.spare);
                cache.spare.clear();
                cache.spare_product.fill(f64::NAN);
                for (bits, value) in cache.bits.iter_mut().zip(
                    cache
                        .inputs
                        .iter()
                        .chain(&cache.weights)
                        .chain(&cache.direction),
                ) {
                    *bits = value.to_bits();
                }
                cache.cache = Some(order);
                self.completion.replaced(demand)?;
                cache.visible = true;
                if let Some(observation) = &mut self.observation {
                    observation.completed(index, demand, order);
                }
                Ok(())
            })();
            if let Err(cause) = evaluated {
                if let Some(observation) = &mut self.observation {
                    observation.failed(index, demand, order);
                }
                return Err(MathError::Instance {
                    instance: binding.instance,
                    occurrences: binding.occurrences.as_ref().clone(),
                    checked_sources: binding.checked_sources.as_ref().clone(),
                    checked_members: binding.checked_members.as_ref().clone(),
                    cause: Box::new(cause),
                });
            }
        }
        self.scope.check().map_err(crate::error::scope_error)
    }
}
fn prepare_product_weights(
    instances: &[Instance],
    structure: &CaseStructure,
    rows: &BTreeMap<SemanticId, GlobalRow>,
) -> Vec<EnumMap<EvaluationDemand, Vec<ProductWeight>>> {
    instances
        .iter()
        .zip(structure.instances())
        .map(|(instance, binding)| {
            EnumMap::from_fn(|demand| {
                let Some(group) = &instance.groups[demand] else {
                    return Vec::new();
                };
                if !matches!(
                    demand,
                    EvaluationDemand::All
                        | EvaluationDemand::Directional
                        | EvaluationDemand::TransposeDirectional
                        | EvaluationDemand::HessianAction
                ) {
                    return Vec::new();
                }
                binding
                    .contributions
                    .iter()
                    .filter_map(|contribution| {
                        let output = group.outputs.binary_search(&contribution.output).ok()?;
                        let target = match contribution.target {
                            Target::Objective(_)
                                if contribution.target == Target::PRIMARY
                                    && !matches!(
                                        demand,
                                        EvaluationDemand::Directional
                                            | EvaluationDemand::TransposeDirectional
                                    ) =>
                            {
                                ProductTarget::Objective
                            }
                            Target::Row(row) => ProductTarget::Row(rows[&row].get()),
                            _ => return None,
                        };
                        Some(ProductWeight {
                            output,
                            target,
                            scale: contribution.scale,
                        })
                    })
                    .collect()
            })
        })
        .collect()
}
fn selected(target: Target, demand: EvaluationDemand) -> bool {
    match demand {
        EvaluationDemand::Objective => target == Target::PRIMARY,
        EvaluationDemand::Constraints => matches!(target, Target::Row(_)),
        EvaluationDemand::Directional
        | EvaluationDemand::TransposeDirectional
        | EvaluationDemand::HessianAction => false,
        // A later lexicographic objective is projected to coefficients, never evaluated.
        EvaluationDemand::All => matches!(target, Target::Row(_)) || target == Target::PRIMARY,
    }
}
// Only typed refusals whose semantics do not contradict scientific premises retain
// an independently complete old result. Unknown provider/library failures are conservative.
fn independent_completion_survives(cause: &MathError) -> bool {
    match cause {
        MathError::Instance { cause, .. } => independent_completion_survives(cause),
        MathError::Cancelled
        | MathError::Limit(_)
        | MathError::ByteLimit { .. }
        | MathError::WorkLimit { .. }
        | MathError::SlotLimit { .. }
        | MathError::DerivativeDemand { .. } => true,
        MathError::Provider { cause, .. } | MathError::Scope(cause) => matches!(
            cause,
            pse_kernels::ProviderError::Cancelled
                | pse_kernels::ProviderError::Deadline
                | pse_kernels::ProviderError::Limit(_)
                | pse_kernels::ProviderError::DerivativeUnavailable { .. }
        ),
        _ => false,
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
    fn selected_actual_evaluator_timing_excludes_cache_hits_and_is_absent_when_off() {
        use pse_diagnostics::observation::{CoherentRecord, ObservationHub, RecordTap, Value};
        #[derive(Debug, Default)]
        struct Tap(std::sync::Mutex<Vec<CoherentRecord>>);
        impl RecordTap for Tap {
            fn selected(&self, family: &str) -> bool {
                family == "evaluator_demand_batch.v1"
            }
            fn observe(&self, record: &CoherentRecord) {
                if self.selected(&record.family) {
                    self.0.lock().unwrap().push(record.clone());
                }
            }
        }
        let (assembly, values) = crate::assembly_tests::fixture(false, false);
        let hub = ObservationHub::new(Default::default());
        let operation = hub.start("selected actual evaluator timing");
        let tap = Arc::new(Tap::default());
        let selected = operation.context().with_record_tap(tap.clone());
        let entered = selected.enter();
        let mut worker = assembly.worker(
            pse_kernels::ProviderBindings::new(),
            pse_diagnostics::Cancellation::new(false),
        );
        let first = worker.objective(&values).unwrap();
        assert_eq!(worker.objective(&values).unwrap(), first);
        drop(entered);
        let records = tap.0.lock().unwrap();
        assert_eq!(records.len(), 2);
        let count = |record: &CoherentRecord, name: &str| match record
            .values
            .iter()
            .find(|field| field.name == name)
            .unwrap()
            .value
        {
            Value::Count(value) => value,
            _ => panic!("selected count missing"),
        };
        assert!(count(&records[0], "actual_evaluations_completed") > 0);
        assert_eq!(
            count(&records[0], "actual_evaluations_completed"),
            count(&records[0], "math.demand.evaluations")
        );
        assert_eq!(count(&records[1], "actual_evaluations_completed"), 0);
        assert_eq!(
            count(&records[1], "actual_evaluations_completed_nanoseconds"),
            0
        );
        assert!(count(&records[1], "math.demand.cache_hits") > 0);
        drop(records);
        let off = hub.start("ordinary observation without selected timing");
        let off_context = off.context();
        let entered = off_context.enter();
        let mut ordinary = assembly.worker(
            pse_kernels::ProviderBindings::new(),
            pse_diagnostics::Cancellation::new(false),
        );
        ordinary.objective(&values).unwrap();
        drop(entered);
        let snapshot = off.reader().snapshot(std::time::Duration::from_secs(60));
        assert!(
            snapshot
                .producers
                .iter()
                .filter_map(|producer| producer.current.as_ref())
                .all(|record| record
                    .values
                    .iter()
                    .all(|field| !field.name.starts_with("actual_evaluation")))
        );
    }
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

#[cfg(test)]
mod cotangent_tests {
    use super::*;
    use crate::{assembly_tests::fixture, library::Optimization};

    #[test]
    fn constraint_cotangents_use_direct_products_or_completed_matrix_and_preserve_aliases() {
        for alias in [false, true] {
            let (original, values) = fixture(alias, false);
            let cancel = pse_diagnostics::Cancellation::new(false);
            let plan = original
                .plan
                .with_transpose_actions(&cancel.flag())
                .unwrap();
            assert!(plan.has_transpose_actions());
            let assembly = Arc::new(
                Arc::new(plan)
                    .compile(
                        Optimization::default(),
                        EvaluationLimits::default(),
                        &pse_kernels::ExecutionScope::new(cancel.clone(), None),
                    )
                    .unwrap(),
            );
            let mut worker = assembly.worker(pse_kernels::ProviderBindings::new(), cancel.clone());
            let weights = vec![2.; assembly.rows.len()];
            let mut output = vec![f64::NAN; assembly.columns.len()];
            worker
                .jacobian_transpose_product(&values, &weights, &mut output)
                .unwrap();
            assert_eq!(worker.completion.assemblies[AssembledProduct::Jacobian], 0);
            let first = worker.completion.evaluations[EvaluationDemand::TransposeDirectional];
            assert!(first > 0);
            let expected = output.clone();
            worker
                .jacobian_transpose_product(&values, &weights, &mut output)
                .unwrap();
            assert_eq!(output, expected);
            assert_eq!(
                worker.completion.evaluations[EvaluationDemand::TransposeDirectional],
                first
            );
            let bound = assembly.bind_values(&values).unwrap();
            let point = assembly
                .columns
                .iter()
                .map(|id| values.scalars[id])
                .collect::<Vec<_>>();
            let trial = bound.trial(&point).unwrap();
            worker.jacobian(trial).unwrap();
            worker
                .jacobian_transpose_product(bound.trial(&point).unwrap(), &weights, &mut output)
                .unwrap();
            assert_eq!(output, expected);
            assert_eq!(
                worker.completion.evaluations[EvaluationDemand::TransposeDirectional],
                first
            );
            let mut doubled = weights.clone();
            doubled.iter_mut().for_each(|w| *w *= 2.);
            worker
                .jacobian_transpose_product(bound.trial(&point).unwrap(), &doubled, &mut output)
                .unwrap();
            for (actual, value) in output.iter().zip(expected) {
                assert_eq!(*actual, 2. * value);
            }
            let intact = output.clone();
            assert!(
                worker
                    .jacobian_transpose_product(&values, &[f64::NAN], &mut output)
                    .is_err()
            );
            assert_eq!(output, intact);
            cancel.cancel();
            assert!(
                worker
                    .jacobian_transpose_product(&values, &weights, &mut output)
                    .is_err()
            );
            assert_eq!(output, intact);
        }
    }
}

#[cfg(test)]
mod empty_action_tests {
    use super::*;
    #[test]
    fn explicitly_prepared_empty_actions_keep_capability_without_unused_programs() {
        let (source, values) = crate::assembly_tests::fixture(false, false);
        let cancel = pse_diagnostics::Cancellation::default();
        let plan = CasePlan::prepare_functions(
            source.structure(),
            |id| source.bodies().get(&id).cloned(),
            &[],
            source.columns().to_vec(),
            &pse_quantity::standard::standard_registry().unwrap(),
            DerivativeOrder::First,
            AssemblyLimits::default(),
            &cancel.flag(),
        )
        .unwrap();
        assert!(!plan.has_directional_actions() && !plan.has_transpose_actions());
        let plan = plan
            .with_directional_actions(&cancel.flag())
            .unwrap()
            .with_transpose_actions(&cancel.flag())
            .unwrap();
        assert!(plan.has_directional_actions() && plan.has_transpose_actions());
        assert!(plan.requests.is_empty());
        let assembly = Arc::new(
            Arc::new(plan)
                .compile(
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &pse_kernels::ExecutionScope::new(cancel.clone(), None),
                )
                .unwrap(),
        );
        let mut worker = assembly.worker(pse_kernels::ProviderBindings::new(), cancel);
        worker
            .jacobian_product(&values, &vec![1.; assembly.columns().len()], &mut [])
            .unwrap();
        let mut output = vec![f64::NAN; assembly.columns().len()];
        worker
            .jacobian_transpose_product(&values, &[], &mut output)
            .unwrap();
        assert!(output.iter().all(|v| *v == 0.));
    }
}

#[cfg(test)]
mod ownership_tests {
    use super::*;
    thread_local! {
        pub(super) static DEPENDENCY_VISITS: std::cell::Cell<(usize, usize)> =
            const { std::cell::Cell::new((0, 0)) };
    }
    #[test]
    fn late_boundary_visits_only_relevant_supports_with_unrelated_parent_growth() {
        use crate::binding::{
            CaseLimits, Contribution, InstanceBinding, Row, SlotBinding, Variable,
        };
        use crate::typed::{Binary, BodyBuilder, BodyLimits};
        use pse_quantity::standard::{StandardInvariantChecker, ids, standard_registry};

        let id = |n| SemanticId::from_bytes([n; 16]);
        let registry = standard_registry().unwrap();
        let quantity = ids::quantity("neutral");
        let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
        let port = |n| pse_kernels::Port {
            id: id(n),
            quantity,
            unit,
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let (base, _) = crate::assembly_tests::fixture(false, false);
        for unrelated in [1u8, 16, 64] {
            let mut bodies = base.bodies().clone();
            let mut variables = base.structure().variables().to_vec();
            let mut bindings = base.structure().instances().to_vec();
            bindings[0].instance = id(250);
            for contribution in &mut bindings[0].contributions {
                if matches!(contribution.target, Target::Row(_)) {
                    contribution.target = Target::Row(id(249));
                }
            }
            bindings[0].contributions.push(Contribution {
                output: 0,
                target: Target::Row(id(250)),
                scale: 1.,
            });
            let mut rows = vec![
                Row {
                    id: id(249),
                    quantity,
                    lower: 0.,
                    upper: 0.,
                },
                Row {
                    id: id(250),
                    quantity,
                    lower: 0.,
                    upper: 0.,
                },
            ];
            for n in 0..unrelated {
                let mut builder = BodyBuilder::new(
                    crate::initialize().unwrap(),
                    &registry,
                    &StandardInvariantChecker,
                    1,
                    BodyLimits::default(),
                )
                .unwrap();
                let input = builder
                    .input(0, quantity, Default::default(), id(230))
                    .unwrap();
                let constant = builder
                    .literal(
                        f64::from(n + 1),
                        registry.unit(unit).unwrap(),
                        pse_quantity::literal::LiteralContext::Explicit {
                            quantity_type: quantity,
                        },
                        id(231),
                    )
                    .unwrap();
                let output = builder
                    .binary(Binary::Add, input, constant, None, id(232))
                    .unwrap();
                let key = ContentHash::from_bytes([n + 10; 32]);
                bodies.insert(key, Arc::new(builder.prepare(&[output]).unwrap()));
                variables.push(Variable {
                    port: port(n + 30),
                    fixed: false,
                    domain: pse_model::generated::enums::ModelingVariableDomain::Continuous,
                    lower: None,
                    upper: None,
                });
                rows.push(Row {
                    id: id(n + 100),
                    quantity,
                    lower: 0.,
                    upper: 0.,
                });
                bindings.push(InstanceBinding {
                    instance: id(n + 10),
                    body: key,
                    occurrences: Default::default(),
                    checked_sources: Default::default(),
                    checked_members: Default::default(),
                    slots: vec![SlotBinding::new(&port(n + 30), &port(230), &registry).unwrap()]
                        .into(),
                    contributions: vec![Contribution {
                        output: 0,
                        target: Target::Row(id(n + 100)),
                        scale: 1.,
                    }],
                });
            }
            for fixed in [false, true] {
                variables
                    .iter_mut()
                    .find(|v| v.port.id == id(2))
                    .unwrap()
                    .fixed = fixed;
                let structure = Arc::new(
                    CaseStructure::new(
                        variables.clone(),
                        vec![],
                        bindings.clone(),
                        rows.clone(),
                        base.structure().objective().cloned(),
                        CaseLimits::default(),
                    )
                    .unwrap(),
                );
                for order in [DerivativeOrder::Value, DerivativeOrder::First] {
                    let plan = CasePlan::prepare(
                        structure.clone(),
                        bodies.clone(),
                        &registry,
                        order,
                        base.limits(),
                        &cancel,
                    )
                    .unwrap();
                    assert_eq!(
                        plan.structure().instances().last().unwrap().instance,
                        id(250)
                    );
                    assert!(plan.supports().len() > usize::from(unrelated));
                    DEPENDENCY_VISITS.with(|visits| visits.set((0, 0)));
                    let dependencies = plan
                        .boundary_dependencies(
                            &BTreeSet::from([id(249)]),
                            &BTreeSet::from([id(1)]),
                            &cancel,
                        )
                        .unwrap();
                    let (binding_visits, support_visits) =
                        DEPENDENCY_VISITS.with(std::cell::Cell::get);
                    assert_eq!(binding_visits, 1);
                    assert!(support_visits > 0);
                    assert!(
                        support_visits
                            <= plan
                                .instances
                                .last()
                                .unwrap()
                                .groups
                                .values()
                                .flatten()
                                .count()
                    );
                    assert_eq!(
                        dependencies,
                        [
                            Target::PRIMARY,
                            Target::Row(id(249)),
                            Target::Row(id(249)),
                            Target::Row(id(250))
                        ]
                        .into_iter()
                        .map(|target| ContributionDependencies {
                            target,
                            numerical: BTreeSet::from([id(1), id(2)]),
                            execution: BTreeSet::from([id(1), id(2)]),
                        })
                        .collect::<Vec<_>>()
                    );
                    assert_eq!(plan.row(id(249)).unwrap().id, id(249));
                    assert!(plan.row(SemanticId::NIL).is_none());
                }
            }
        }
    }
    #[test]
    fn indexed_contributors_and_all_shared_request_consumers_survive_demand_changes() {
        let (source, _) = crate::assembly_tests::fixture(false, false);
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let mut bindings = source.structure().instances().to_vec();
        let first_row = source.structure().rows()[0].id;
        let second_row = source.structure().rows()[1].id;
        let mut second = bindings[0].clone();
        second.instance = SemanticId::from_bytes([99; 16]);
        second
            .contributions
            .retain(|c| matches!(c.target, Target::Row(_)));
        for contribution in &mut second.contributions {
            contribution.target = Target::Row(second_row);
        }
        bindings.push(second);
        let structure = Arc::new(
            CaseStructure::new(
                source.structure().variables().to_vec(),
                source.structure().parameters().to_vec(),
                bindings,
                source.structure().rows().to_vec(),
                source.structure().objectives().first().cloned(),
                crate::binding::CaseLimits::default(),
            )
            .unwrap(),
        );
        let value = CasePlan::prepare(
            structure,
            source.bodies().clone(),
            &registry,
            DerivativeOrder::Value,
            source.limits(),
            &cancel,
        )
        .unwrap();
        assert_eq!(
            value.selected_contributions([second_row, first_row, first_row]),
            BTreeMap::from([(0, vec![1, 2]), (1, vec![0, 1])])
        );
        assert!(value.selected_contributions([SemanticId::NIL]).is_empty());
        assert_eq!(value.binding_slot_count(), 4);
        assert_eq!(value.contribution_count(), 5);
        let boundary = value
            .boundary_dependencies(
                &BTreeSet::from([first_row]),
                &BTreeSet::from([source.columns()[0]]),
                &cancel,
            )
            .unwrap();
        assert_eq!(
            boundary,
            value.dependencies(&cancel).unwrap(),
            "shared slots keep the outside row and objective in the candidate closure"
        );
        assert_eq!(value.binding_sources(), source.binding_sources());
        let shared = value
            .requests
            .iter()
            .enumerate()
            .find_map(|(index, _)| {
                (value
                    .request_consumers(index)
                    .iter()
                    .filter(|(_, demand)| *demand == EvaluationDemand::Constraints)
                    .count()
                    == 2)
                    .then_some(index)
            })
            .unwrap();
        assert_eq!(
            value
                .request_consumers(shared)
                .iter()
                .filter(|(_, demand)| *demand == EvaluationDemand::Constraints)
                .map(|(instance, _)| *instance)
                .collect::<Vec<_>>(),
            [0, 1]
        );
        let first = value
            .with_directional_actions(&cancel)
            .unwrap()
            .with_transpose_actions(&cancel)
            .unwrap();
        let second = first
            .prepare_order(DerivativeOrder::Second, &cancel)
            .unwrap()
            .with_hessian_actions(&cancel)
            .unwrap();
        for plan in [&value, &first, &second] {
            for request in 0..plan.demands().len() {
                let exhaustive = plan
                    .instances
                    .iter()
                    .enumerate()
                    .flat_map(|(instance, local)| {
                        local.groups.iter().filter_map(move |(demand, group)| {
                            group
                                .as_ref()
                                .filter(|g| g.request == request)
                                .map(|_| (instance, demand))
                        })
                    })
                    .collect::<Vec<_>>();
                assert_eq!(plan.request_consumers(request), exhaustive);
            }
        }
        assert_eq!(value.order(), DerivativeOrder::Value);
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            value.prepare_order(DerivativeOrder::Second, &cancel),
            Err(MathError::Cancelled)
        ));
        assert_eq!(value.request_consumers(shared).len(), 3);
    }
}

#[cfg(test)]
mod instance_reuse_tests {
    use super::*;
    use crate::binding::{CaseLimits, Contribution, InstanceBinding, Row, SlotBinding, Variable};
    use pse_quantity::standard::{StandardInvariantChecker, ids, standard_registry};
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    pub(super) fn fixture(
        sqrt: bool,
    ) -> (Arc<CaseAssembly>, CaseValues, pse_diagnostics::Cancellation) {
        let standard = standard_registry().unwrap();
        let quantity = ids::quantity("neutral");
        let canonical = standard.quantity_type(quantity).unwrap().canonical_unit;
        let mut registry = standard.to_builder();
        let mut triple = standard.unit(canonical).unwrap().clone();
        triple.id = pse_quantity::UnitId::from_id(id(220));
        triple.symbol = "triple-neutral-reuse".into();
        triple.scale_to_canonical = 3.;
        triple.definition = None;
        let scaled = triple.id;
        registry.unit(triple);
        let registry = registry.build().unwrap();
        let port = |n, unit| pse_kernels::Port {
            id: id(n),
            quantity,
            unit,
        };
        let mut builder = crate::typed::BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            1,
            crate::typed::BodyLimits::default(),
        )
        .unwrap();
        let input = builder
            .input(0, quantity, pse_quantity::IndexSet::new(), id(30))
            .unwrap();
        let output = if sqrt {
            builder.unary(crate::Function::Sqrt, input, id(31)).unwrap()
        } else {
            builder
                .binary(
                    crate::typed::Binary::Mul,
                    input.clone(),
                    input,
                    None,
                    id(31),
                )
                .unwrap()
        };
        let body = Arc::new(builder.prepare(&[output]).unwrap());
        let key = ContentHash::from_bytes([30; 32]);
        let structure = Arc::new(
            CaseStructure::new(
                vec![(1, scaled), (2, canonical)]
                    .into_iter()
                    .map(|(n, unit)| Variable {
                        port: port(n, unit),
                        fixed: false,
                        domain: pse_model::generated::enums::ModelingVariableDomain::Continuous,
                        lower: None,
                        upper: None,
                    })
                    .collect(),
                vec![],
                (0..2)
                    .map(|i| InstanceBinding {
                        instance: id(i + 10),
                        body: key,
                        occurrences: Default::default(),
                        checked_sources: Default::default(),
                        checked_members: Default::default(),
                        slots: vec![
                            SlotBinding::new(
                                &port(i + 1, if i == 0 { scaled } else { canonical }),
                                &port(3, canonical),
                                &registry,
                            )
                            .unwrap(),
                        ]
                        .into(),
                        contributions: vec![Contribution {
                            output: 0,
                            target: Target::Row(id(i + 20)),
                            scale: 1.,
                        }],
                    })
                    .collect(),
                (20..22)
                    .map(|n| Row {
                        id: id(n),
                        quantity,
                        lower: 0.,
                        upper: 0.,
                    })
                    .collect(),
                None,
                CaseLimits::default(),
            )
            .unwrap(),
        );
        let control = pse_diagnostics::Cancellation::default();
        let plan = CasePlan::prepare(
            structure,
            BTreeMap::from([(key, body)]),
            &registry,
            DerivativeOrder::First,
            AssemblyLimits::default(),
            &control.flag(),
        )
        .unwrap();
        let assembly = Arc::new(
            Arc::new(plan)
                .compile(
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &pse_kernels::ExecutionScope::new(control.clone(), None),
                )
                .unwrap(),
        );
        (
            assembly,
            CaseValues {
                scalars: BTreeMap::from([(id(1), 1.), (id(2), 3.)]),
            },
            control,
        )
    }
    #[test]
    fn equal_normalized_gathers_reuse_arithmetic_but_keep_each_instance_scatter_and_bits() {
        let (assembly, mut values, control) = fixture(false);
        let mut worker = assembly.worker(pse_kernels::ProviderBindings::new(), control.clone());
        assert_eq!(worker.constraints(&values).unwrap(), [9., 9.]);
        assert_eq!(
            worker.completion.evaluations[EvaluationDemand::Constraints],
            1
        );
        assert_eq!(worker.jacobian(&values).unwrap().val(), [18., 6.]);
        assert_eq!(
            worker.completion.evaluations[EvaluationDemand::Constraints],
            2
        );
        assert_eq!(worker.constraints(&values).unwrap(), [9., 9.]);
        assert_eq!(
            worker.completion.evaluations[EvaluationDemand::Constraints],
            2
        );
        values.scalars.insert(id(2), 4.);
        assert_eq!(worker.constraints(&values).unwrap(), [9., 16.]);
        assert_eq!(
            worker.completion.evaluations[EvaluationDemand::Constraints],
            3
        );
        let mut signed = assembly.worker(pse_kernels::ProviderBindings::new(), control.clone());
        values.scalars.insert(id(1), -0.);
        values.scalars.insert(id(2), 0.);
        assert_eq!(signed.constraints(&values).unwrap(), [0., 0.]);
        // Existing affine gathers add their canonical zero offset. Donation uses
        // those actual normalized bits, rather than unnormalized source encodings.
        assert_eq!(
            signed.groups[0][EvaluationDemand::Constraints]
                .as_ref()
                .unwrap()
                .inputs[0]
                .to_bits(),
            signed.groups[1][EvaluationDemand::Constraints]
                .as_ref()
                .unwrap()
                .inputs[0]
                .to_bits()
        );
        assert_eq!(
            signed.completion.evaluations[EvaluationDemand::Constraints],
            1
        );
        let mut opposite = assembly.worker(pse_kernels::ProviderBindings::new(), control.clone());
        values.scalars.insert(id(1), -1.);
        values.scalars.insert(id(2), 3.);
        assert_eq!(opposite.constraints(&values).unwrap(), [9., 9.]);
        assert_eq!(
            opposite.completion.evaluations[EvaluationDemand::Constraints],
            2,
            "equal outputs cannot replace the complete gathered input identity"
        );
        assert_eq!(opposite.jacobian(&values).unwrap().val(), [-18., 6.]);
        let mut failing = assembly.worker(pse_kernels::ProviderBindings::new(), control.clone());
        values.scalars.insert(id(1), 1e200);
        assert!(
            matches!(failing.constraints(&values), Err(MathError::Instance { instance, .. }) if instance == id(10))
        );
        assert_eq!(
            failing.completion.evaluations[EvaluationDemand::Constraints],
            1
        );
        control.cancel();
        assert!(worker.constraints(&values).is_err());
    }
    #[test]
    fn receiver_context_guards_are_excluded_from_instance_donation_and_value_does_not_grant_first()
    {
        let (assembly, mut values, control) = fixture(true);
        values.scalars.insert(id(1), 0.);
        values.scalars.insert(id(2), 0.);
        let mut worker = assembly.worker(pse_kernels::ProviderBindings::new(), control);
        assert_eq!(worker.constraints(&values).unwrap(), [0., 0.]);
        assert_eq!(
            worker.completion.evaluations[EvaluationDemand::Constraints],
            2
        );
        assert!(
            matches!(worker.jacobian(&values), Err(MathError::Instance { instance, .. }) if instance == id(10))
        );
        assert_eq!(
            worker.completion.evaluations[EvaluationDemand::Constraints],
            3
        );
    }
}
