// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable preparation, effect-owned compilation and worker-local library evaluation.
use crate::{
    MathError,
    guarded::{Comparison, Condition, Stage, validate_dependencies},
    jets::{EvaluationLimits, JetLayout, LiftInput, ProviderLift},
    library::{self, Optimization},
};
use enum_map::EnumMap;
use pse_ids::SemanticId;
use pse_kernels::{
    DerivativeOrder, EvaluationContext, Provider, ProviderKey, ProviderRequest, ProviderSpec,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use symbolica::{
    atom::{Atom, AtomCore, Indeterminate, Symbol},
    evaluate::ExpressionEvaluator,
};

/// Conservative complete mathematical support and separate control dependencies.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Support {
    /// Nonzero-possible first derivative input columns for each output.
    pub first: Vec<BTreeSet<usize>>,
    /// Nonzero-possible symmetric second derivative input pairs for each output.
    pub second: Vec<BTreeSet<(usize, usize)>>,
    /// Inputs needed for obligations/branch decisions even when numerical derivatives vanish.
    pub controls: BTreeSet<usize>,
}

/// Immutable demand-indexed mathematical support, separate from the value program.
#[derive(Clone, Debug)]
pub struct PreparedSupport {
    data: Arc<PreparedSupportData>,
    owner: Option<Arc<dyn crate::AllocationOwner>>,
}
#[derive(Debug)]
struct PreparedSupportData {
    body: PreparedBody,
    outputs: Vec<usize>,
    coordinates: Vec<usize>,
    order: DerivativeOrder,
    support: Support,
    remaining_occurrences: usize,
    derivative_operations: usize,
}
impl PartialEq for PreparedSupport {
    fn eq(&self, other: &Self) -> bool {
        // Allocation leases and construction receipts do not change mathematical demand identity.
        self.body() == other.body()
            && self.outputs() == other.outputs()
            && self.coordinates() == other.coordinates()
            && self.order() == other.order()
            && self.support() == other.support()
    }
}
impl PreparedSupport {
    /// Original body output ordinals; support rows use this ordered map.
    pub fn outputs(&self) -> &[usize] {
        &self.data.outputs
    }
    /// Original formal input positions; never renumbered by selection.
    pub fn coordinates(&self) -> &[usize] {
        &self.data.coordinates
    }
    /// Constructed support ceiling, independent of numerical capability/readiness.
    pub fn order(&self) -> DerivativeOrder {
        self.data.order
    }
    /// Requested all-branch support. Value products have no derivative support rows.
    pub fn support(&self) -> &Support {
        &self.data.support
    }
    /// First support by original body output ordinal, through the explicit selected map.
    pub fn first_for_output(&self, output: usize) -> Option<&BTreeSet<usize>> {
        self.outputs()
            .iter()
            .position(|&i| i == output)
            .and_then(|row| self.support().first.get(row))
    }
    /// Second support by original body output ordinal, through the selected map.
    pub fn second_for_output(&self, output: usize) -> Option<&BTreeSet<(usize, usize)>> {
        self.outputs()
            .iter()
            .position(|&i| i == output)
            .and_then(|row| self.support().second.get(row))
    }
    /// Unspent finite authored construction allowance.
    pub fn remaining_occurrences(&self) -> usize {
        self.data.remaining_occurrences
    }
    /// Library derivative calls performed while constructing this immutable support.
    pub fn derivative_operations(&self) -> usize {
        self.data.derivative_operations
    }
    /// Demand's admitted immutable value program.
    pub fn body(&self) -> &PreparedBody {
        &self.data.body
    }
    /// Unique owned support storage identity; body data has its own allocation identity.
    pub fn allocation_identity(&self) -> usize {
        Arc::as_ptr(&self.data) as usize
    }
    /// Known owned demand payload, excluding separately shared body data.
    pub fn retained_bytes(&self) -> usize {
        size_of::<PreparedSupportData>()
            + (self.data.outputs.capacity() + self.data.coordinates.capacity()) * size_of::<usize>()
            + (self.data.support.first.capacity() + self.data.support.second.capacity())
                * size_of::<BTreeSet<usize>>()
            + self
                .data
                .support
                .first
                .iter()
                .map(|s| s.len() * (size_of::<usize>() + 96))
                .sum::<usize>()
            + self
                .data
                .support
                .second
                .iter()
                .map(|s| s.len() * (size_of::<(usize, usize)>() + 96))
                .sum::<usize>()
            + self.data.support.controls.len() * (size_of::<usize>() + 96)
    }
    /// Retain the runtime owner's lease with every escaping support clone.
    pub fn with_owner(mut self, owner: Arc<dyn crate::AllocationOwner>) -> Self {
        self.owner = Some(crate::retain_allocation_owner(self.owner.take(), owner));
        self
    }
    /// Add stronger support under the remaining allowance; failure preserves this product.
    pub fn upgrade(
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
        self.body().support_with_allowance(
            self.outputs(),
            self.coordinates(),
            order,
            self.remaining_occurrences(),
            cancel,
            Some(self),
        )
    }
    /// Restrict outputs without repeating symbolic support construction.
    pub fn select_outputs(
        &self,
        outputs: &[usize],
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self, MathError> {
        self.body().check_selection(outputs, self.coordinates())?;
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let rows = outputs
            .iter()
            .map(|output| {
                self.outputs()
                    .iter()
                    .position(|i| i == output)
                    .ok_or_else(|| {
                        MathError::Contract("support output selection is not a subset".into())
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut remaining = self.remaining_occurrences();
        let mut allowance = SupportAllowance {
            remaining: &mut remaining,
            derivatives: 0,
        };
        allowance.consume(
            outputs
                .len()
                .checked_add(self.coordinates().len())
                .ok_or(MathError::Limit("support maps"))?,
            SemanticId::NIL,
        )?;
        let mut support = Support::default();
        if self.order() > DerivativeOrder::Value {
            allowance.consume(self.support().controls.len(), SemanticId::NIL)?;
            support.controls = self.support().controls.clone();
            for &row in &rows {
                allowance.consume(self.support().first[row].len(), SemanticId::NIL)?;
                support.first.push(self.support().first[row].clone());
                if self.order() >= DerivativeOrder::Second {
                    allowance.consume(self.support().second[row].len(), SemanticId::NIL)?;
                    support.second.push(self.support().second[row].clone());
                }
            }
        }
        Ok(Self {
            data: Arc::new(PreparedSupportData {
                body: self.body().clone(),
                outputs: outputs.to_vec(),
                coordinates: self.coordinates().to_vec(),
                order: self.order(),
                support,
                remaining_occurrences: remaining,
                derivative_operations: 0,
            }),
            owner: self.owner.clone(),
        })
    }
    /// Compile numerical products after independent selected-closure capability admission.
    pub fn compile(
        &self,
        options: Optimization,
        limits: EvaluationLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<CompiledBody, MathError> {
        self.body().compile_scope(
            self.outputs(),
            self.coordinates(),
            self.order(),
            options,
            limits,
            cancel,
            false,
            self,
        )
    }
    /// Compile strict-interior derivatives; every trial retains boundary admission.
    pub fn compile_branch_local(
        &self,
        options: Optimization,
        limits: EvaluationLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<CompiledBody, MathError> {
        self.body().compile_scope(
            self.outputs(),
            self.coordinates(),
            self.order(),
            options,
            limits,
            cancel,
            true,
            self,
        )
    }
}

/// Physically admitted symbolic regions without mutable evaluators or foreign workers.
#[derive(Clone, Debug)]
pub struct PreparedBody {
    data: Arc<PreparedBodyData>,
    owner: Option<Arc<dyn crate::AllocationOwner>>,
}
/// Shared immutable body data; construction and mutation stay inside admission.
#[derive(Clone, Debug)]
pub struct PreparedBodyData {
    pub(crate) inputs: usize,
    pub(crate) slots: usize,
    pub(crate) outputs: Vec<usize>,
    output_effects: Option<Vec<BTreeSet<usize>>>,
    pub(crate) stages: Vec<Stage>,
    /// Unspent authored construction allowance, continued by each demand.
    remaining_occurrences: usize,
    pub(crate) obligations: Vec<(Option<Atom>, Condition)>,
    input_quantities: Vec<Option<pse_quantity::QuantityTypeId>>,
    output_quantities: Vec<pse_quantity::QuantityTypeId>,
    expressions: Vec<Option<Atom>>,
    providers: Vec<ProviderSpec>,
    /// Local checked-member tokens whose actual attribution belongs to each instance.
    checked_members: BTreeSet<SemanticId>,
    /// Construction occurrences its builder counted; accounting, not mathematical identity.
    occurrences: usize,
}
impl std::ops::Deref for PreparedBody {
    type Target = PreparedBodyData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl PartialEq for PreparedBody {
    fn eq(&self, other: &Self) -> bool {
        // Allocation lifetime is runtime metadata, not mathematical identity.
        self.inputs == other.inputs
            && self.slots == other.slots
            && self.outputs == other.outputs
            && self.output_effects == other.output_effects
            && self.stages == other.stages
            && self.obligations == other.obligations
            && self.input_quantities == other.input_quantities
            && self.output_quantities == other.output_quantities
            && self.expressions == other.expressions
            && self.providers == other.providers
            && self.checked_members == other.checked_members
    }
}
impl PreparedBody {
    /// Process-local immutable allocation identity used only for unique live accounting.
    pub fn allocation_identity(&self) -> usize {
        Arc::as_ptr(&self.data) as usize
    }
    /// Known symbolic payload and container contents. The library's global symbol interner
    /// is process state that no product owns or releases, so it is not counted here.
    pub fn retained_bytes(&self) -> usize {
        fn stages(items: &[Stage], seen: &mut BTreeSet<usize>) -> usize {
            fn lineage_bytes(
                lineage: &Arc<pse_model::diagnostic::ValidityLineage>,
                seen: &mut BTreeSet<usize>,
            ) -> usize {
                if seen.insert(Arc::as_ptr(lineage) as usize) {
                    size_of_val(lineage.as_ref()) + lineage.heap_bytes() + 2 * size_of::<usize>()
                } else {
                    0
                }
            }
            size_of_val(items)
                + items
                    .iter()
                    .map(|s| match s {
                        Stage::Block {
                            expressions,
                            outputs,
                            ..
                        } => {
                            size_of_val(expressions.as_slice())
                                + expressions
                                    .iter()
                                    .map(|a| a.as_view().get_byte_size())
                                    .sum::<usize>()
                                + size_of_val(outputs.as_slice())
                        }
                        Stage::Branch {
                            then, otherwise, ..
                        } => stages(then, seen) + stages(otherwise, seen),
                        Stage::Domain {
                            stages: local,
                            lineage,
                            ..
                        } => stages(local, seen) + lineage_bytes(lineage, seen),
                        Stage::Applicability {
                            stages: local,
                            predicates,
                            inputs,
                            plan,
                            ..
                        } => {
                            stages(local, seen)
                                + (predicates.capacity() + inputs.capacity()) * size_of::<usize>()
                                + plan.retained_bytes()
                        }
                        Stage::Provider {
                            inputs,
                            outputs,
                            spec,
                            partial,
                            ..
                        } => {
                            size_of_val(inputs.as_slice())
                                + size_of_val(partial.as_slice())
                                + size_of_val(outputs.as_slice())
                                + size_of_val(spec.inputs.as_slice())
                                + size_of_val(spec.outputs.as_slice())
                                + spec.shapes.retained_bytes()
                        }
                        Stage::Require { lineage, .. } => lineage
                            .as_ref()
                            .map_or(0, |lineage| lineage_bytes(lineage, seen)),
                    })
                    .sum::<usize>()
        }
        size_of::<PreparedBodyData>()
            + self.checked_members.len() * (size_of::<SemanticId>() + 96)
            + stages(&self.stages, &mut BTreeSet::new())
            + self.outputs.capacity() * size_of::<usize>()
            + self.input_quantities.capacity() * size_of::<Option<pse_quantity::QuantityTypeId>>()
            + self.output_quantities.capacity() * size_of::<pse_quantity::QuantityTypeId>()
            + self
                .expressions
                .iter()
                .flatten()
                .chain(self.obligations.iter().filter_map(|(a, _)| a.as_ref()))
                .map(|a| a.as_view().get_byte_size())
                .sum::<usize>()
    }
    /// Declare body-local checked-member attribution after typed closure admission.
    /// Every declared token must occur in an emitted closure validity stage; form/data
    /// lineages keep their original authored identities and are not instance tokens.
    pub fn with_checked_members(mut self, tokens: BTreeSet<SemanticId>) -> Result<Self, MathError> {
        fn collect(stages: &[Stage], members: &mut BTreeSet<SemanticId>) {
            use pse_model::generated::enums::ModelingValidityLayer;
            for stage in stages {
                match stage {
                    Stage::Require {
                        lineage: Some(lineage),
                        ..
                    } => {
                        if lineage.layer == ModelingValidityLayer::Closure {
                            members.extend(&lineage.members);
                        }
                    }
                    Stage::Domain {
                        stages, lineage, ..
                    } => {
                        if lineage.layer == ModelingValidityLayer::Closure {
                            members.extend(&lineage.members);
                        }
                        collect(stages, members);
                    }
                    Stage::Applicability { stages, .. } => collect(stages, members),
                    Stage::Branch {
                        then, otherwise, ..
                    } => {
                        collect(then, members);
                        collect(otherwise, members);
                    }
                    _ => {}
                }
            }
        }
        let mut emitted = BTreeSet::new();
        collect(&self.stages, &mut emitted);
        if emitted != tokens {
            return Err(MathError::Contract(
                "checked-member tokens do not cover emitted closure validity".into(),
            ));
        }
        Arc::make_mut(&mut self.data).checked_members = tokens;
        Ok(self)
    }
    /// Complete attribution demand of this normalized reusable body.
    pub fn checked_members(&self) -> &BTreeSet<SemanticId> {
        &self.checked_members
    }
    /// Retain accounting when a body clone outlives its runtime case plan.
    pub fn with_owner(mut self, owner: Arc<dyn crate::AllocationOwner>) -> Self {
        self.owner = Some(crate::retain_allocation_owner(self.owner.take(), owner));
        self
    }
    #[cfg(test)]
    pub(crate) fn new(
        inputs: usize,
        slots: usize,
        outputs: Vec<usize>,
        stages: Vec<Stage>,
        _smooth: DerivativeOrder,
    ) -> Result<Self, MathError> {
        Self::new_with_allowance(
            inputs,
            slots,
            outputs,
            stages,
            &mut crate::typed::BodyLimits::default().occurrences,
        )
    }
    /// Value admission continues the caller's finite authored construction allowance.
    pub(crate) fn new_with_allowance(
        inputs: usize,
        slots: usize,
        outputs: Vec<usize>,
        stages: Vec<Stage>,
        remaining: &mut usize,
    ) -> Result<Self, MathError> {
        if slots == 0 || inputs > slots || outputs.is_empty() || outputs.iter().any(|&i| i >= slots)
        {
            return Err(MathError::Contract("prepared body layout".into()));
        }
        let mut allowance = SupportAllowance {
            remaining,
            derivatives: 0,
        };
        allowance.consume(slots, SemanticId::NIL)?;
        let parameters = (0..slots)
            .map(library::formal)
            .collect::<Result<Vec<_>, _>>()?;
        let symbols = symbol_map(&parameters);
        let mut assigned = (0..inputs).collect::<BTreeSet<_>>();
        validate_dependencies(&stages, &symbols, &mut assigned, 0, allowance.remaining)?;
        if outputs.iter().any(|i| !assigned.contains(i)) {
            return Err(MathError::Contract(
                "output unassigned on some branch".into(),
            ));
        }
        allowance.consume(slots, SemanticId::NIL)?;
        allowance.consume(1, SemanticId::NIL)?;
        let empty_fact = Arc::new(Fact::default());
        let mut facts = vec![empty_fact; slots];
        for i in 0..inputs {
            allowance.consume(1, SemanticId::NIL)?;
            facts[i] = Arc::new(Fact {
                expression: Some(parameters[i].clone()),
                ..Fact::default()
            });
        }
        let mut providers = BTreeMap::new();
        let mut obligations = vec![];
        analyze(
            &stages,
            &parameters,
            &symbols,
            &mut facts,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
            &mut providers,
            &mut obligations,
            &mut allowance,
            DerivativeOrder::Value,
            None,
            None,
            None,
        )?;
        let expressions = outputs
            .iter()
            .map(|&i| facts[i].expression.clone())
            .collect();
        Ok(Self {
            data: Arc::new(PreparedBodyData {
                inputs,
                slots,
                outputs,
                output_effects: None,
                stages,
                remaining_occurrences: *allowance.remaining,
                obligations,
                input_quantities: vec![None; inputs],
                output_quantities: vec![],
                expressions,
                providers: providers.into_values().collect(),
                occurrences: 0,
                checked_members: BTreeSet::new(),
            }),
            owner: None,
        })
    }
    /// Smooth derivative capability of all admitted outputs and formal coordinates.
    pub fn available_order(&self) -> DerivativeOrder {
        self.available_order_for(&(0..self.inputs).collect::<Vec<_>>())
    }
    /// Order available inside selected active branches; boundaries require trial checks.
    pub fn branch_local_order(&self) -> DerivativeOrder {
        self.selected_order(
            &(0..self.output_count()).collect::<Vec<_>>(),
            &(0..self.inputs).collect::<Vec<_>>(),
            true,
        )
        .unwrap_or(DerivativeOrder::Value)
    }
    /// Capability of all outputs with respect to selected formal coordinates.
    pub fn available_order_for(&self, coordinates: &[usize]) -> DerivativeOrder {
        self.available_order_for_outputs(&(0..self.output_count()).collect::<Vec<_>>(), coordinates)
            .unwrap_or(DerivativeOrder::Value)
    }
    /// Capability of precisely the selected output/effect closure, independent of readiness.
    pub fn available_order_for_outputs(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
    ) -> Result<DerivativeOrder, MathError> {
        self.selected_order(outputs, coordinates, false)
    }
    fn selected_order(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        local_branches: bool,
    ) -> Result<DerivativeOrder, MathError> {
        self.check_selection(outputs, coordinates)?;
        let stages = self.demanded_stages(outputs)?;
        let parameters = (0..self.slots)
            .map(library::formal)
            .collect::<Result<Vec<_>, _>>()?;
        let symbols = symbol_map(&parameters);
        let mut depends = vec![false; self.slots];
        for &i in coordinates {
            depends[i] = true;
        }
        let numeric = numeric_slots(
            &stages,
            &outputs.iter().map(|&i| self.outputs[i]).collect::<Vec<_>>(),
            &symbols,
        )?;
        selected_capability(&stages, &symbols, &mut depends, local_branches, &numeric)
    }
    fn check_selection(&self, outputs: &[usize], coordinates: &[usize]) -> Result<(), MathError> {
        if outputs.is_empty()
            || outputs.iter().any(|&i| i >= self.output_count())
            || outputs.iter().collect::<BTreeSet<_>>().len() != outputs.len()
            || coordinates.iter().any(|&i| i >= self.inputs)
            || coordinates.iter().collect::<BTreeSet<_>>().len() != coordinates.len()
        {
            return Err(MathError::Contract(
                "compiled output/coordinate demand".into(),
            ));
        }
        Ok(())
    }
    /// Construct selected immutable support without constructing an evaluator.
    pub fn prepare_support(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<PreparedSupport, MathError> {
        self.support_with_allowance(
            outputs,
            coordinates,
            order,
            self.remaining_occurrences,
            cancel,
            None,
        )
    }
    /// Conservative all-branch incidence, including opaque value-only providers.
    pub fn incidence(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        cancel: &Arc<AtomicBool>,
    ) -> Result<PreparedSupport, MathError> {
        self.prepare_support(outputs, coordinates, DerivativeOrder::First, cancel)
    }
    fn support_with_allowance(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        order: DerivativeOrder,
        mut remaining: usize,
        cancel: &Arc<AtomicBool>,
        prior: Option<&PreparedSupport>,
    ) -> Result<PreparedSupport, MathError> {
        self.check_selection(outputs, coordinates)?;
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let mut allowance = SupportAllowance {
            remaining: &mut remaining,
            derivatives: 0,
        };
        allowance.consume(
            outputs
                .len()
                .checked_add(coordinates.len())
                .ok_or(MathError::Limit("support demand maps"))?,
            SemanticId::NIL,
        )?;
        let mut support = Support::default();
        if order > DerivativeOrder::Value {
            allowance.consume(self.slots, SemanticId::NIL)?;
            let parameters = (0..self.slots)
                .map(library::formal)
                .collect::<Result<Vec<_>, _>>()?;
            let symbols = symbol_map(&parameters);
            let stages = self.demand(outputs, &symbols);
            allowance.consume(self.slots, SemanticId::NIL)?;
            allowance.consume(1, SemanticId::NIL)?;
            let empty_fact = Arc::new(Fact::default());
            let mut facts = vec![empty_fact; self.slots];
            for &i in coordinates {
                allowance.consume(2, SemanticId::NIL)?;
                facts[i] = Arc::new(Fact {
                    expression: Some(parameters[i].clone()),
                    first: BTreeSet::from([i]),
                    ..Fact::default()
                });
            }
            // Non-coordinate inputs remain symbolic parameters for exact local composition.
            for i in (0..self.inputs).filter(|i| !coordinates.contains(i)) {
                allowance.consume(1, SemanticId::NIL)?;
                facts[i] = Arc::new(Fact {
                    expression: Some(parameters[i].clone()),
                    ..Fact::default()
                });
            }
            let selected = coordinates.iter().copied().collect::<BTreeSet<_>>();
            let numeric = numeric_slots(
                &stages,
                &outputs.iter().map(|&i| self.outputs[i]).collect::<Vec<_>>(),
                &symbols,
            )?;
            analyze(
                &stages,
                &parameters,
                &symbols,
                &mut facts,
                &mut support.controls,
                &mut BTreeSet::new(),
                &mut BTreeMap::new(),
                &mut vec![],
                &mut allowance,
                order,
                Some(&selected),
                Some(cancel),
                Some(&numeric),
            )?;
            allowance.consume(
                outputs
                    .len()
                    .checked_mul(if order >= DerivativeOrder::Second {
                        2
                    } else {
                        1
                    })
                    .ok_or(MathError::Limit("support output extent"))?,
                SemanticId::NIL,
            )?;
            for &output in outputs {
                let fact = &facts[self.outputs[output]];
                let source = fact.source.unwrap_or(SemanticId::NIL);
                allowance.consume(fact.first.len(), source)?;
                support.first.push(fact.first.clone());
                if order >= DerivativeOrder::Second {
                    allowance.consume(fact.second.len(), source)?;
                    support.second.push(fact.second.clone());
                }
            }
        }
        if let Some(prior) = prior
            && order >= DerivativeOrder::First
            && prior.order() >= DerivativeOrder::First
            && support.first != prior.support().first
        {
            return Err(MathError::Contract(
                "support upgrade changed incidence".into(),
            ));
        }
        let derivative_operations = allowance.derivatives;
        Ok(PreparedSupport {
            data: Arc::new(PreparedSupportData {
                body: self.clone(),
                outputs: outputs.to_vec(),
                coordinates: coordinates.to_vec(),
                order,
                support,
                remaining_occurrences: remaining,
                derivative_operations,
            }),
            owner: prior.and_then(|p| p.owner.clone()),
        })
    }
    /// Number of formal inputs.
    pub fn input_count(&self) -> usize {
        self.inputs
    }
    /// Formal slots the body holds: its inputs plus stage results, within its
    /// `BodyLimits::slots` allowance.
    pub fn slot_count(&self) -> usize {
        self.slots
    }
    /// Authored construction occurrences counted by its typed builder, before support
    /// preparation consumes the rest of `BodyLimits::occurrences`; zero without a builder.
    pub fn occurrence_count(&self) -> usize {
        self.occurrences
    }
    pub(crate) fn set_occurrences(&mut self, occurrences: usize) {
        Arc::make_mut(&mut self.data).occurrences = occurrences;
    }
    pub(crate) fn set_effects(&mut self, effects: Vec<BTreeSet<usize>>) {
        Arc::make_mut(&mut self.data).output_effects = Some(effects);
    }
    pub(crate) fn set_quantities(
        &mut self,
        inputs: Vec<Option<pse_quantity::QuantityTypeId>>,
        outputs: Vec<pse_quantity::QuantityTypeId>,
    ) {
        let data = Arc::make_mut(&mut self.data);
        data.input_quantities = inputs;
        data.output_quantities = outputs;
    }
    /// Scalar physical contracts established by the builder; unused inputs may be absent.
    pub fn input_quantities(&self) -> &[Option<pse_quantity::QuantityTypeId>] {
        &self.input_quantities
    }
    /// Physical output contracts in declaration order.
    pub fn output_quantities(&self) -> &[pse_quantity::QuantityTypeId] {
        &self.output_quantities
    }
    /// Number of ordered body outputs.
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
    /// Consumed executable contracts, keyed by full physical/data interpretation.
    pub fn providers(&self) -> &[ProviderSpec] {
        &self.providers
    }
    /// Actual provider orders required by compilation, including authored local partials.
    /// Domain and applicability evidence is evaluated at value order independently of outer jets.
    pub fn provider_demands(
        &self,
        order: DerivativeOrder,
    ) -> Result<BTreeMap<ProviderKey, DerivativeOrder>, MathError> {
        self.provider_demands_for_outputs(&(0..self.output_count()).collect::<Vec<_>>(), order)
    }
    /// Actual provider requirements for selected outputs and their mandatory effects.
    pub fn provider_demands_for_outputs(
        &self,
        outputs: &[usize],
        order: DerivativeOrder,
    ) -> Result<BTreeMap<ProviderKey, DerivativeOrder>, MathError> {
        fn collect(
            stages: &[Stage],
            order: DerivativeOrder,
            demands: &mut BTreeMap<ProviderKey, DerivativeOrder>,
            numeric: &BTreeSet<usize>,
        ) -> Result<(), MathError> {
            for stage in stages {
                match stage {
                    Stage::Provider {
                        spec,
                        partial,
                        outputs,
                        ..
                    } => {
                        let requested = if outputs.iter().any(|i| numeric.contains(i)) {
                            order
                        } else {
                            DerivativeOrder::Value
                        };
                        let required = provider_request_order(requested, partial.len())?;
                        demands
                            .entry(spec.key())
                            .and_modify(|o| *o = (*o).max(required))
                            .or_insert(required);
                    }
                    Stage::Domain { stages, .. } | Stage::Applicability { stages, .. } => {
                        collect(stages, DerivativeOrder::Value, demands, numeric)?
                    }
                    Stage::Branch {
                        then, otherwise, ..
                    } => {
                        collect(then, order, demands, numeric)?;
                        collect(otherwise, order, demands, numeric)?;
                    }
                    Stage::Block { .. } | Stage::Require { .. } => {}
                }
            }
            Ok(())
        }
        let mut demands = BTreeMap::new();
        let stages = self.demanded_stages(outputs)?;
        let parameters = (0..self.slots)
            .map(library::formal)
            .collect::<Result<Vec<_>, _>>()?;
        let numeric = numeric_slots(
            &stages,
            &outputs.iter().map(|&i| self.outputs[i]).collect::<Vec<_>>(),
            &symbol_map(&parameters),
        )?;
        collect(&stages, order, &mut demands, &numeric)?;
        Ok(demands)
    }
    /// Optional flattened library expression of one output. It is absent when a provider or
    /// branch prevents flattening, and also when substituting the shared stage results
    /// would exceed the flattening bound (large but factorable bodies, such as Helmholtz
    /// derivatives). Projections that must not lose such outputs read the stage program
    /// instead ([`crate::factorable`]).
    pub fn expression(&self, output: usize) -> Option<&Atom> {
        self.expressions.get(output).and_then(Option::as_ref)
    }
    /// Whether coefficient views must retain a domain/control obligation.
    pub fn has_obligations(&self) -> bool {
        has_obligations(&self.stages)
    }
    /// The stage program an evaluation of `outputs` executes: every retained obligation and
    /// only the arithmetic those outputs and their effects need. Projections read this
    /// program so that they carry exactly the obligations the evaluator enforces.
    /// # Errors
    /// An output ordinal outside the body, or an exhausted formal vocabulary.
    pub(crate) fn demanded_stages(&self, outputs: &[usize]) -> Result<Vec<Stage>, MathError> {
        if outputs.iter().any(|&i| i >= self.outputs.len()) {
            return Err(MathError::Contract("demanded body output".into()));
        }
        let parameters = (0..self.slots)
            .map(library::formal)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self.demand(outputs, &symbol_map(&parameters)))
    }
    fn demand(&self, outputs: &[usize], symbols: &HashMap<Symbol, usize>) -> Vec<Stage> {
        let mut needed: BTreeSet<_> = outputs.iter().map(|&i| self.outputs[i]).collect();
        if let Some(effects) = &self.output_effects {
            needed.extend(outputs.iter().flat_map(|&i| effects[i].iter().copied()));
        }
        prune(
            &self.stages,
            &mut needed,
            symbols,
            self.output_effects.is_none(),
        )
    }
    /// Compile an exact output/coordinate demand under explicit limits.
    pub fn compile(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        order: DerivativeOrder,
        options: Optimization,
        limits: EvaluationLimits,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<CompiledBody, MathError> {
        self.prepare_support(outputs, coordinates, order, cancelled)?
            .compile(options, limits, cancelled)
    }
    /// Compile derivatives restricted to the strict interior of the selected control path.
    /// Each trial checks separation from every unproved guard boundary, including domain
    /// predicates. This establishes only a local neighborhood, never transition smoothness.
    /// # Errors
    /// Unsupported provider derivatives, resource limits, or invalid demands. A worker
    /// additionally refuses a trial on an unproved control boundary.
    pub fn compile_branch_local(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        order: DerivativeOrder,
        options: Optimization,
        limits: EvaluationLimits,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<CompiledBody, MathError> {
        self.prepare_support(outputs, coordinates, order, cancelled)?
            .compile_branch_local(options, limits, cancelled)
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "The local scope is explicit and the public compilation contracts stay unchanged"
    )]
    fn compile_scope(
        &self,
        outputs: &[usize],
        coordinates: &[usize],
        order: DerivativeOrder,
        options: Optimization,
        limits: EvaluationLimits,
        cancelled: &Arc<AtomicBool>,
        local_branches: bool,
        prepared_support: &PreparedSupport,
    ) -> Result<CompiledBody, MathError> {
        limits.check()?;
        if cancelled.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        self.check_selection(outputs, coordinates)?;
        if order > self.selected_order(outputs, coordinates, local_branches)? {
            return Err(MathError::Contract(
                "nonsmooth operation lacks a derivative neighborhood".into(),
            ));
        }
        let selected: Vec<_> = outputs.iter().map(|&i| self.outputs[i]).collect();
        let parameters = (0..self.slots)
            .map(library::formal)
            .collect::<Result<Vec<_>, _>>()?;
        let symbols = symbol_map(&parameters);
        let stages = self.demand(outputs, &symbols);
        let numeric = numeric_slots(&stages, &selected, &symbols)?;
        // Conservative coordinate reachability includes every branch alternative. It
        // selects Taylor coefficients; Symbolica still owns every derivative operation.
        let reachability_coordinates = if order > DerivativeOrder::Value {
            coordinates.len()
        } else {
            0
        };
        let support_entries = self
            .slots
            .checked_mul(reachability_coordinates.saturating_add(3))
            .ok_or(MathError::Limit("coordinate reachability"))?;
        limits.allocation(support_entries)?;
        let mut coordinate_support = vec![vec![false; reachability_coordinates]; self.slots];
        if order > DerivativeOrder::Value {
            for (coordinate, &slot) in coordinates.iter().enumerate() {
                coordinate_support[slot][coordinate] = true;
            }
            coordinate_reachability(&stages, &symbols, &mut coordinate_support)?;
        }
        // One layout and program per compiled order, `Value` through `order`.
        let mut layouts = EnumMap::<DerivativeOrder, Option<JetLayout>>::default();
        let mut programs = EnumMap::<DerivativeOrder, Option<Vec<CompiledStage>>>::default();
        let mut used = 0usize;
        let mut retained_numeric = 0usize;
        let mut retained_instructions = 0usize;
        let mut remaining_operations = limits.operations;
        let mut remaining_providers = limits.provider_calls;
        for requested in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
        ]
        .into_iter()
        .filter(|requested| *requested <= order)
        {
            let layout = JetLayout::new(coordinates.to_vec(), requested, limits)?;
            let frame = self
                .slots
                .checked_mul(layout.width())
                .ok_or(MathError::Limit("jet frame"))?;
            limits.allocation(frame)?;
            let mut allowance = BuildAllowance {
                operations: remaining_operations,
                providers: remaining_providers,
                entries: frame,
                instructions: 0,
                local_order: if local_branches {
                    requested
                } else {
                    DerivativeOrder::Value
                },
            };
            let program = compile_stages(
                &stages,
                &parameters,
                &symbols,
                &coordinate_support,
                &numeric,
                &layout,
                options,
                limits,
                cancelled,
                &mut allowance,
            )?;
            remaining_operations = allowance.operations;
            remaining_providers = allowance.providers;
            retained_numeric = retained_numeric
                .checked_add(allowance.entries - frame)
                .ok_or(MathError::Limit("retained numeric storage"))?;
            retained_instructions = retained_instructions
                .checked_add(allowance.instructions)
                .ok_or(MathError::Limit("retained instruction storage"))?;
            used = used
                .checked_add(allowance.entries)
                .ok_or(MathError::Limit("compiled demand scratch"))?;
            let n = coordinates.len();
            let output_width = 1usize
                .checked_add(if requested >= DerivativeOrder::First {
                    n
                } else {
                    0
                })
                .and_then(|k| {
                    k.checked_add(if requested >= DerivativeOrder::Second {
                        n * n
                    } else {
                        0
                    })
                })
                .ok_or(MathError::Limit("output derivative buffers"))?;
            used = selected
                .len()
                .checked_mul(output_width)
                .and_then(|entries| used.checked_add(entries))
                .ok_or(MathError::Limit("output derivative buffers"))?;
            limits.allocation(used)?;
            layouts[requested] = Some(layout);
            programs[requested] = Some(program);
        }
        let evidence_bytes = applicability_extent(&stages).saturating_mul(4);
        let scratch_bytes = used
            .checked_mul(size_of::<f64>())
            .and_then(|b| b.checked_add(evidence_bytes))
            .ok_or(MathError::Limit("applicability observation storage"))?;
        if scratch_bytes > limits.scratch_bytes {
            return Err(MathError::Limit("applicability observation storage"));
        }
        Ok(CompiledBody {
            owner: None,
            inputs: self.inputs,
            slots: self.slots,
            scratch_bytes,
            retained_bytes: retained_numeric
                .checked_mul(size_of::<f64>())
                .and_then(|n| n.checked_add(retained_instructions))
                .and_then(|n| n.checked_add(evidence_bytes))
                .and_then(|n| n.checked_add(prepared_support.retained_bytes()))
                .ok_or(MathError::Limit("retained program storage"))?,
            outputs: Arc::new(selected),
            layouts: Arc::new(layouts),
            programs: Arc::new(programs),
            limits,
            support: prepared_support.clone(),
        })
    }
}

#[derive(Clone)]
#[allow(
    clippy::large_enum_variant,
    reason = "Evaluator stages stay inline to avoid an allocation for each compiled stage"
)]
enum CompiledStage {
    Applicability {
        stages: Vec<Self>,
        frame: Vec<f64>,
        layout: JetLayout,
        predicates: Vec<usize>,
        inputs: Vec<usize>,
        token: usize,
        plan: Arc<pse_model::applicability::Node>,
    },
    Domain {
        stages: Vec<Self>,
        frame: Vec<f64>,
        layout: JetLayout,
        argument: usize,
        token: usize,
        lineage: Arc<pse_model::diagnostic::ValidityLineage>,
    },
    Block {
        evaluator: ExpressionEvaluator<f64>,
        components: Vec<usize>,
        inputs: Vec<usize>,
        outputs: Vec<usize>,
        arguments: Vec<f64>,
        values: Vec<f64>,
        source: SemanticId,
    },
    Require {
        argument: usize,
        condition: Condition,
        order: DerivativeOrder,
        source: SemanticId,
        lineage: Option<Arc<pse_model::diagnostic::ValidityLineage>>,
    },
    Branch {
        require_separation: bool,
        comparison: Comparison,
        left: usize,
        right: usize,
        then: Vec<Self>,
        otherwise: Vec<Self>,
    },
    Provider {
        spec: ProviderSpec,
        partial: Vec<usize>,
        inputs: Vec<usize>,
        outputs: Vec<usize>,
        request: ProviderRequest,
        arguments: Vec<f64>,
        lift: Option<ProviderLift>,
        source: SemanticId,
    },
}
/// Immutable library artifact. Clone workers to obtain independent evaluator scratch.
#[derive(Clone)]
pub struct CompiledBody {
    owner: Option<Arc<dyn crate::AllocationOwner>>,
    scratch_bytes: usize,
    retained_bytes: usize,
    inputs: usize,
    slots: usize,
    outputs: Arc<Vec<usize>>,
    layouts: Arc<EnumMap<DerivativeOrder, Option<JetLayout>>>,
    programs: Arc<EnumMap<DerivativeOrder, Option<Vec<CompiledStage>>>>,
    limits: EvaluationLimits,
    support: PreparedSupport,
}
impl std::fmt::Debug for CompiledBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledBody")
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs.len())
            .field("profiles", &self.layouts.values().flatten().count())
            .finish()
    }
}
impl CompiledBody {
    /// Highest residual order actually compiled, distinct from symbolic availability.
    pub fn compiled_order(&self) -> DerivativeOrder {
        if self.layouts[DerivativeOrder::Second].is_some() {
            DerivativeOrder::Second
        } else if self.layouts[DerivativeOrder::First].is_some() {
            DerivativeOrder::First
        } else {
            DerivativeOrder::Value
        }
    }
    /// Attach accounting to the allocation itself so evaluator/worker clones retain it.
    pub fn with_owner(mut self, owner: Arc<dyn crate::AllocationOwner>) -> Self {
        self.support = self.support.with_owner(owner.clone());
        self.owner = Some(crate::retain_allocation_owner(self.owner.take(), owner));
        self
    }
    /// Upper bound on owned numeric frame and stage scratch.
    pub fn scratch_bytes(&self) -> usize {
        self.scratch_bytes
    }
    /// Storage retained by the immutable evaluator templates: their numeric buffers and
    /// library instruction streams, excluding attempt frames and returned derivative
    /// buffers.
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
    /// Independent mutable scratch; caller/attempt owns its provider worker map.
    pub fn worker(&self) -> Worker {
        let width = self
            .layouts
            .values()
            .flatten()
            .last()
            .map_or(1, JetLayout::width);
        Worker {
            body: self.clone(),
            programs: self.programs.as_ref().clone(),
            frame: vec![0.0; self.slots * width],
        }
    }
    /// Exact selected admitted demand retained by this evaluator product.
    pub fn prepared_support(&self) -> &PreparedSupport {
        &self.support
    }
    /// Structural support survives trials and numerical zeros.
    pub fn support(&self) -> &Support {
        self.support.support()
    }
    /// Explicit structural incidence admission from this retained output demand.
    pub fn incidence(&self, cancel: &Arc<AtomicBool>) -> Result<PreparedSupport, MathError> {
        if self.support.order() >= DerivativeOrder::First {
            return Ok(self.support.clone());
        }
        self.support.upgrade(DerivativeOrder::First, cancel)
    }
    /// Differentiation coordinates in formal input order.
    pub fn coordinates(&self) -> &[usize] {
        self.layouts[DerivativeOrder::Value]
            .as_ref()
            .map_or(&[], JetLayout::coordinates)
    }
}
/// Atomic result, with raw derivatives in output-major row-major order.
#[derive(Clone, Debug, PartialEq)]
pub struct Evaluation {
    /// Evidence observed only on active demanded numerical paths.
    pub applicability: Vec<pse_model::applicability::Observation>,
    /// Requested values.
    pub values: Vec<f64>,
    /// Output × coordinate first partials.
    pub jacobian: Vec<f64>,
    /// Output × coordinate × coordinate full symmetric Hessians.
    pub hessians: Vec<f64>,
}
/// Worker-local numeric frame and Symbolica scratch.
#[derive(Clone)]
pub struct Worker {
    body: CompiledBody,
    programs: EnumMap<DerivativeOrder, Option<Vec<CompiledStage>>>,
    frame: Vec<f64>,
}
impl std::fmt::Debug for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Worker")
            .field("body", &self.body)
            .field("frame_entries", &self.frame.len())
            .finish_non_exhaustive()
    }
}
impl Worker {
    /// Evaluate only the requested order, publishing no result on failure.
    pub fn evaluate(
        &mut self,
        inputs: &[f64],
        order: DerivativeOrder,
        providers: &mut BTreeMap<ProviderKey, Box<dyn Provider>>,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<Evaluation, MathError> {
        self.frame.fill(f64::NAN);
        if inputs.len() != self.body.inputs || inputs.iter().any(|v| !v.is_finite()) {
            return Err(MathError::Contract(
                "finite ordered formal inputs required".into(),
            ));
        }
        let (Some(layout), Some(program)) = (
            self.body.layouts[order].as_ref(),
            self.programs[order].as_mut(),
        ) else {
            return Err(MathError::Contract("uncompiled derivative order".into()));
        };
        let width = layout.width();
        let n = layout.coordinates.len();
        for (slot, &value) in inputs.iter().enumerate() {
            let jet = &mut self.frame[slot * width..(slot + 1) * width];
            jet.fill(0.0);
            jet[0] = value;
        }
        if order >= DerivativeOrder::First {
            for (i, &slot) in layout.coordinates.iter().enumerate() {
                self.frame[slot * width + 1 + i] = 1.0;
            }
        }
        let context = EvaluationContext {
            cancelled,
            max_result_bytes: self.body.limits.scratch_bytes,
        };
        let mut applicability = Vec::new();
        evaluate_stages(
            program,
            &mut self.frame,
            layout,
            providers,
            &context,
            &mut applicability,
        )?;
        context.check().map_err(|cause| MathError::Provider {
            source_id: SemanticId::NIL,
            provider: SemanticId::NIL,
            cause,
        })?;
        let mut result = Evaluation {
            applicability,
            values: Vec::with_capacity(self.body.outputs.len()),
            jacobian: vec![],
            hessians: vec![],
        };
        if order >= DerivativeOrder::Second {
            result.hessians.resize(self.body.outputs.len() * n * n, 0.0);
        }
        for (row, &slot) in self.body.outputs.iter().enumerate() {
            let jet = &self.frame[slot * width..(slot + 1) * width];
            if jet.iter().any(|v| !v.is_finite()) {
                return Err(MathError::Contract(
                    "nonfinite or unassigned result jet".into(),
                ));
            }
            result.values.push(jet[0]);
            if order >= DerivativeOrder::First {
                result.jacobian.extend_from_slice(&jet[1..=n]);
            }
            if order >= DerivativeOrder::Second {
                for (k, &(i, j)) in layout.pairs.iter().enumerate() {
                    let value = jet[1 + n + k] * layout.raw_factor(1 + n + k);
                    result.hessians[row * n * n + i * n + j] = value;
                    result.hessians[row * n * n + j * n + i] = value;
                }
            }
        }
        Ok(result)
    }
}

// Stages are in dependency order and scalar destinations are single assignment.
// Union, rather than replacement, covers shared destinations of mutually exclusive
// arms and domain-local slots. Extra reachability is safe; omitted derivatives are not.
fn coordinate_reachability(
    stages: &[Stage],
    symbols: &HashMap<Symbol, usize>,
    support: &mut [Vec<bool>],
) -> Result<(), MathError> {
    for stage in stages {
        match stage {
            Stage::Block {
                expressions,
                outputs,
                ..
            } => {
                for (expression, &output) in expressions.iter().zip(outputs) {
                    let inputs = reads(std::slice::from_ref(expression), symbols)?;
                    for coordinate in 0..support[output].len() {
                        support[output][coordinate] |=
                            inputs.iter().any(|&i| support[i][coordinate]);
                    }
                }
            }
            Stage::Provider {
                inputs, outputs, ..
            } => {
                for &output in outputs.iter().filter(|&&i| i != usize::MAX) {
                    for coordinate in 0..support[output].len() {
                        support[output][coordinate] |=
                            inputs.iter().any(|&i| support[i][coordinate]);
                    }
                }
            }
            Stage::Branch {
                then, otherwise, ..
            } => {
                coordinate_reachability(then, symbols, support)?;
                coordinate_reachability(otherwise, symbols, support)?;
            }
            // Predicate bodies only request values; they cannot contribute derivatives.
            Stage::Domain { .. } | Stage::Applicability { .. } | Stage::Require { .. } => {}
        }
    }
    Ok(())
}

fn numeric_slots(
    stages: &[Stage],
    outputs: &[usize],
    symbols: &HashMap<Symbol, usize>,
) -> Result<BTreeSet<usize>, MathError> {
    fn collect(
        stages: &[Stage],
        needed: &mut BTreeSet<usize>,
        seen: &mut BTreeSet<usize>,
        symbols: &HashMap<Symbol, usize>,
    ) -> Result<(), MathError> {
        for stage in stages.iter().rev() {
            match stage {
                Stage::Block {
                    expressions,
                    outputs,
                    ..
                } => {
                    for (expression, &output) in expressions.iter().zip(outputs) {
                        if needed.remove(&output) {
                            seen.insert(output);
                            let inputs = reads(std::slice::from_ref(expression), symbols)?;
                            needed.extend(&inputs);
                            seen.extend(inputs);
                        }
                    }
                }
                Stage::Provider {
                    inputs, outputs, ..
                } => {
                    if outputs.iter().any(|i| needed.contains(i)) {
                        for output in outputs {
                            if needed.remove(output) {
                                seen.insert(*output);
                            }
                        }
                        needed.extend(inputs);
                        seen.extend(inputs);
                    }
                }
                Stage::Branch {
                    then, otherwise, ..
                } => {
                    let mut a = needed.clone();
                    let mut b = needed.clone();
                    collect(then, &mut a, seen, symbols)?;
                    collect(otherwise, &mut b, seen, symbols)?;
                    *needed = a.union(&b).copied().collect();
                }
                Stage::Require { .. } | Stage::Domain { .. } | Stage::Applicability { .. } => {}
            }
        }
        Ok(())
    }
    let mut needed = outputs.iter().copied().collect();
    let mut seen = BTreeSet::new();
    collect(stages, &mut needed, &mut seen, symbols)?;
    Ok(seen)
}

fn selected_capability(
    stages: &[Stage],
    symbols: &HashMap<Symbol, usize>,
    depends: &mut [bool],
    local_branches: bool,
    numeric: &BTreeSet<usize>,
) -> Result<DerivativeOrder, MathError> {
    let mut available = DerivativeOrder::Second;
    for stage in stages {
        match stage {
            Stage::Block {
                expressions,
                outputs,
                ..
            } => {
                for (expression, &output) in expressions.iter().zip(outputs) {
                    depends[output] |= reads(std::slice::from_ref(expression), symbols)?
                        .iter()
                        .any(|&i| depends[i]);
                }
            }
            Stage::Provider {
                spec,
                partial,
                inputs,
                outputs,
                ..
            } => {
                let remaining = (spec.derivatives.min(spec.smoothness) as usize)
                    .checked_sub(partial.len())
                    .ok_or_else(|| MathError::Contract("provider partial capability".into()))?;
                let dependent = inputs.iter().any(|&i| depends[i]);
                if dependent && outputs.iter().any(|i| numeric.contains(i)) {
                    available = available.min(match remaining {
                        0 => DerivativeOrder::Value,
                        1 => DerivativeOrder::First,
                        _ => DerivativeOrder::Second,
                    });
                }
                for &output in outputs.iter().filter(|&&s| s != usize::MAX) {
                    depends[output] |= dependent;
                }
            }
            Stage::Branch {
                continuity,
                left,
                right,
                then,
                otherwise,
                ..
            } => {
                if depends[*left] || depends[*right] {
                    available =
                        available.min(if local_branches && *continuity == DerivativeOrder::Value {
                            DerivativeOrder::Second
                        } else {
                            *continuity
                        });
                }
                available = available.min(selected_capability(
                    then,
                    symbols,
                    depends,
                    local_branches,
                    numeric,
                )?);
                available = available.min(selected_capability(
                    otherwise,
                    symbols,
                    depends,
                    local_branches,
                    numeric,
                )?);
            }
            Stage::Domain { .. } | Stage::Applicability { .. } | Stage::Require { .. } => {}
        }
    }
    Ok(available)
}

struct BuildAllowance {
    local_order: DerivativeOrder,
    operations: usize,
    providers: usize,
    entries: usize,
    /// Instruction storage of the evaluators built so far, in bytes: retained with the
    /// program, never scratch.
    instructions: usize,
}
fn provider_request_order(
    order: DerivativeOrder,
    partials: usize,
) -> Result<DerivativeOrder, MathError> {
    match order as usize + partials {
        0 => Ok(DerivativeOrder::Value),
        1 => Ok(DerivativeOrder::First),
        2 => Ok(DerivativeOrder::Second),
        _ => Err(MathError::Contract(
            "external derivative order exhausted by explicit partial".into(),
        )),
    }
}
#[allow(
    clippy::too_many_arguments,
    reason = "Compilation receives distinct symbol, budget, and cancellation contexts"
)]
fn compile_stages(
    stages: &[Stage],
    parameters: &[Atom],
    symbols: &HashMap<Symbol, usize>,
    coordinate_support: &[Vec<bool>],
    numeric: &BTreeSet<usize>,
    layout: &JetLayout,
    options: Optimization,
    limits: EvaluationLimits,
    cancelled: &Arc<AtomicBool>,
    allowance: &mut BuildAllowance,
) -> Result<Vec<CompiledStage>, MathError> {
    let mut result = vec![];
    for stage in stages {
        if cancelled.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        result.push(match stage {
            Stage::Applicability {
                stages,
                predicates,
                inputs,
                token,
                plan,
            } => {
                let value_layout = JetLayout::new(vec![], DerivativeOrder::Value, limits)?;
                allowance.entries = allowance
                    .entries
                    .checked_add(parameters.len())
                    .ok_or(MathError::Limit("applicability scratch"))?;
                limits.allocation(allowance.entries)?;
                CompiledStage::Applicability {
                    stages: compile_stages(
                        stages,
                        parameters,
                        symbols,
                        coordinate_support,
                        numeric,
                        &value_layout,
                        options,
                        limits,
                        cancelled,
                        allowance,
                    )?,
                    frame: vec![f64::NAN; parameters.len()],
                    layout: value_layout,
                    predicates: predicates.clone(),
                    inputs: inputs.clone(),
                    token: *token,
                    plan: plan.clone(),
                }
            }
            Stage::Domain {
                stages,
                argument,
                token,
                lineage,
            } => {
                let value_layout = JetLayout::new(vec![], DerivativeOrder::Value, limits)?;
                allowance.entries = allowance
                    .entries
                    .checked_add(parameters.len())
                    .ok_or(MathError::Limit("domain predicate scratch"))?;
                limits.allocation(allowance.entries)?;
                CompiledStage::Domain {
                    stages: compile_stages(
                        stages,
                        parameters,
                        symbols,
                        coordinate_support,
                        numeric,
                        &value_layout,
                        options,
                        limits,
                        cancelled,
                        allowance,
                    )?,
                    frame: vec![f64::NAN; parameters.len()],
                    layout: value_layout,
                    argument: *argument,
                    token: *token,
                    lineage: lineage.clone(),
                }
            }
            Stage::Block {
                expressions,
                outputs,
                source,
            } => {
                let inputs = reads(expressions, symbols)?;
                let params = inputs
                    .iter()
                    .map(|&i| parameters[i].clone())
                    .collect::<Vec<_>>();
                let active: Vec<_> = layout
                    .coordinates
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &slot)| {
                        (layout.order > DerivativeOrder::Value
                            && outputs.iter().any(|i| numeric.contains(i))
                            && inputs.iter().any(|&input| coordinate_support[input][i]))
                        .then_some((i, slot))
                    })
                    .collect();
                let local = JetLayout::new(
                    active.iter().map(|&(_, slot)| slot).collect(),
                    layout.order,
                    limits,
                )?;
                let mut components = vec![0];
                if layout.order >= DerivativeOrder::First {
                    components.extend(active.iter().map(|&(i, _)| 1 + i));
                }
                if layout.order >= DerivativeOrder::Second {
                    for &(i, j) in &local.pairs {
                        let pair = (active[i].0, active[j].0);
                        let position = layout
                            .pairs
                            .iter()
                            .position(|&p| p == pair)
                            .ok_or_else(|| MathError::Contract("Taylor subset pair".into()))?;
                        components.push(1 + layout.coordinates.len() + position);
                    }
                }
                let evaluator = library::bounded_evaluator(
                    *source,
                    expressions,
                    &params,
                    &local,
                    options,
                    cancelled,
                    limits,
                    allowance.operations,
                    allowance.entries,
                )?;
                let operations = operation_count(evaluator.count_operations());
                allowance.operations = allowance
                    .operations
                    .checked_sub(operations)
                    .ok_or(MathError::Limit("compiled operations"))?;
                let input_len = evaluator.get_input_len();
                let output_len = evaluator.get_output_len();
                if input_len != inputs.len() * local.width()
                    || output_len != outputs.len() * local.width()
                {
                    return Err(MathError::Contract("library vectorization layout".into()));
                }
                let storage = library::storage(&evaluator)?;
                allowance.entries = allowance
                    .entries
                    .checked_add(
                        input_len + output_len + components.len() + storage.numeric_entries,
                    )
                    .ok_or(MathError::Limit("evaluator scratch"))?;
                limits.allocation(allowance.entries)?;
                allowance.instructions = allowance
                    .instructions
                    .checked_add(storage.instruction_bytes)
                    .ok_or(MathError::Limit("evaluator instruction storage"))?;
                CompiledStage::Block {
                    evaluator,
                    components,
                    inputs,
                    outputs: outputs.clone(),
                    arguments: vec![0.0; input_len],
                    values: vec![0.0; output_len],
                    source: *source,
                }
            }
            Stage::Require {
                argument,
                condition,
                order,
                source,
                lineage,
            } => CompiledStage::Require {
                argument: *argument,
                condition: *condition,
                order: *order,
                source: *source,
                lineage: lineage.clone(),
            },
            Stage::Branch {
                continuity,
                comparison,
                left,
                right,
                then,
                otherwise,
            } => CompiledStage::Branch {
                require_separation: allowance.local_order > *continuity && left != right,
                comparison: *comparison,
                left: *left,
                right: *right,
                then: compile_stages(
                    then,
                    parameters,
                    symbols,
                    coordinate_support,
                    numeric,
                    layout,
                    options,
                    limits,
                    cancelled,
                    allowance,
                )?,
                otherwise: compile_stages(
                    otherwise,
                    parameters,
                    symbols,
                    coordinate_support,
                    numeric,
                    layout,
                    options,
                    limits,
                    cancelled,
                    allowance,
                )?,
            },
            Stage::Provider {
                spec,
                partial,
                inputs,
                outputs,
                source,
            } => {
                allowance.providers = allowance
                    .providers
                    .checked_sub(1)
                    .ok_or(MathError::Limit("provider calls"))?;
                let differentiated = layout.order > DerivativeOrder::Value
                    && outputs.iter().any(|i| numeric.contains(i))
                    && inputs
                        .iter()
                        .any(|&i| coordinate_support[i].iter().any(|&dependent| dependent));
                let request = ProviderRequest {
                    outputs: outputs
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &slot)| (slot != usize::MAX).then_some(i))
                        .collect(),
                    order: provider_request_order(
                        if differentiated {
                            layout.order
                        } else {
                            DerivativeOrder::Value
                        },
                        partial.len(),
                    )?,
                };
                let destinations = outputs
                    .iter()
                    .copied()
                    .filter(|&s| s != usize::MAX)
                    .collect();
                let lift = if differentiated && layout.width() > 1 {
                    Some(ProviderLift::compile(
                        inputs.len(),
                        layout,
                        options,
                        EvaluationLimits {
                            operations: allowance.operations,
                            ..limits
                        },
                        cancelled,
                    )?)
                } else {
                    None
                };
                if let Some(lift) = &lift {
                    allowance.operations = allowance
                        .operations
                        .checked_sub(operation_count(lift.evaluator.count_operations()))
                        .ok_or(MathError::Limit("provider lift operations"))?;
                }
                let entries = inputs.len()
                    + request.outputs.len() * (1 + inputs.len() + inputs.len() * inputs.len());
                allowance.entries = allowance
                    .entries
                    .checked_add(
                        entries
                            + lift.as_ref().map_or(0, |l| {
                                l.scratch.len()
                                    + l.output.len()
                                    + operation_count(l.evaluator.count_operations())
                            }),
                    )
                    .ok_or(MathError::Limit("provider scratch"))?;
                limits.allocation(allowance.entries)?;
                CompiledStage::Provider {
                    spec: spec.clone(),
                    partial: partial.clone(),
                    inputs: inputs.clone(),
                    outputs: destinations,
                    request,
                    arguments: vec![0.0; inputs.len()],
                    lift,
                    source: *source,
                }
            }
        });
    }
    Ok(result)
}
fn operation_count(c: symbolica::evaluate::OperationCount) -> usize {
    c.additions
        .saturating_add(c.multiplications)
        .saturating_add(c.inversions)
        .saturating_add(c.function_calls)
}
fn evaluate_stages(
    stages: &mut [CompiledStage],
    frame: &mut [f64],
    layout: &JetLayout,
    providers: &mut BTreeMap<ProviderKey, Box<dyn Provider>>,
    context: &EvaluationContext<'_>,
    observations: &mut Vec<pse_model::applicability::Observation>,
) -> Result<(), MathError> {
    let width = layout.width();
    for stage in stages {
        if context.cancelled.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        match stage {
            CompiledStage::Applicability {
                stages,
                frame: local,
                layout: value_layout,
                predicates,
                inputs,
                token,
                plan,
            } => {
                for (slot, value) in local.iter_mut().enumerate() {
                    *value = frame[slot * width];
                }
                evaluate_stages(
                    stages,
                    local,
                    value_layout,
                    providers,
                    context,
                    observations,
                )?;
                if predicates
                    .iter()
                    .chain(inputs.iter())
                    .any(|slot| !local[*slot].is_finite())
                {
                    return Err(MathError::Contract(
                        "nonfinite applicability capture".into(),
                    ));
                }
                let predicates = predicates
                    .iter()
                    .map(|slot| local[*slot] > 0.)
                    .collect::<Vec<_>>();
                let inputs = inputs.iter().map(|slot| local[*slot]).collect::<Vec<_>>();
                let mut assessment = plan.assess(&predicates, &inputs);
                let offset = observations.len();
                observations.append(&mut assessment.observations);
                if !assessment.refused.is_empty() {
                    assessment.observations = observations.clone();
                    for index in &mut assessment.refused {
                        *index += offset;
                    }
                    return Err(MathError::Applicability(Box::new(assessment)));
                }
                frame[*token * width..(*token + 1) * width].fill(0.);
            }
            CompiledStage::Domain {
                stages,
                frame: local,
                layout: value_layout,
                argument,
                token,
                lineage,
            } => {
                for (slot, value) in local.iter_mut().enumerate() {
                    *value = frame[slot * width];
                }
                evaluate_stages(
                    stages,
                    local,
                    value_layout,
                    providers,
                    context,
                    observations,
                )?;
                if !Condition::Positive.permits(local[*argument]) {
                    return Err(MathError::Validity(Box::new(lineage.as_ref().clone())));
                }
                frame[*token * width..(*token + 1) * width].fill(0.0);
            }
            CompiledStage::Block {
                evaluator,
                components,
                inputs,
                outputs,
                arguments,
                values,
                source,
            } => {
                let local_width = components.len();
                for (k, &slot) in inputs.iter().enumerate() {
                    for (j, &component) in components.iter().enumerate() {
                        arguments[k * local_width + j] = frame[slot * width + component];
                    }
                }
                evaluator
                    .try_evaluate(arguments, values)
                    .map_err(|e| MathError::Evaluation {
                        source_id: *source,
                        order: layout.order,
                        detail: e.to_string(),
                    })?;
                if values.iter().any(|v| !v.is_finite()) {
                    return Err(MathError::Domain {
                        source_id: *source,
                        requirement: "finite arithmetic jet",
                    });
                }
                for (k, &slot) in outputs.iter().enumerate() {
                    frame[slot * width..(slot + 1) * width].fill(0.0);
                    for (j, &component) in components.iter().enumerate() {
                        frame[slot * width + component] = values[k * local_width + j];
                    }
                }
            }
            CompiledStage::Require {
                argument,
                condition,
                order,
                source,
                lineage,
            } => {
                if layout.order >= *order && !condition.permits(frame[*argument * width]) {
                    // A closure range names its lineage (Plan 23 H5).
                    return Err(match lineage {
                        Some(lineage) => MathError::Validity(Box::new(lineage.as_ref().clone())),
                        None => MathError::Domain {
                            source_id: *source,
                            requirement: condition.description(),
                        },
                    });
                }
            }
            CompiledStage::Branch {
                require_separation,
                comparison,
                left,
                right,
                then,
                otherwise,
            } => {
                let a = frame[*left * width];
                let b = frame[*right * width];
                if !a.is_finite() || !b.is_finite() {
                    return Err(MathError::Contract("unassigned branch input".into()));
                }
                if *require_separation && a == b {
                    return Err(MathError::Domain {
                        source_id: SemanticId::NIL,
                        requirement: "branch-local derivative trial is on an unproved control boundary",
                    });
                }
                evaluate_stages(
                    if comparison.select(a, b) {
                        then
                    } else {
                        otherwise
                    },
                    frame,
                    layout,
                    providers,
                    context,
                    observations,
                )?;
            }
            CompiledStage::Provider {
                spec,
                partial,
                inputs,
                outputs,
                request,
                arguments,
                lift,
                source,
            } => {
                let provider = providers
                    .get_mut(&spec.key())
                    .ok_or_else(|| MathError::Contract("missing provider worker".into()))?;
                provider
                    .spec()
                    .check_bound(spec, request.order)
                    .map_err(|e| MathError::Contract(e.to_string()))?;
                for (value, &slot) in arguments.iter_mut().zip(inputs.iter()) {
                    *value = frame[slot * width];
                }
                let error = |cause| MathError::Provider {
                    source_id: *source,
                    provider: spec.id,
                    cause,
                };
                request.validate(spec, context).map_err(error)?;
                let values = provider
                    .evaluate(arguments, request, context)
                    .map_err(error)?;
                values.validate(spec, request).map_err(error)?;
                for (row, &slot) in outputs.iter().enumerate() {
                    let n = inputs.len();
                    let selected = match partial.as_slice() {
                        [] => values.values[row],
                        [i] => values.jacobian[row * n + i],
                        [i, j] => values.hessians[(row * n + i) * n + j],
                        _ => return Err(MathError::Contract("external partial order".into())),
                    };
                    if let Some(lift) = lift.as_mut() {
                        let n = inputs.len();
                        for (parameter, value) in lift.inputs.iter().zip(&mut lift.scratch) {
                            *value = match *parameter {
                                LiftInput::Value => selected,
                                LiftInput::First(i) => {
                                    if let Some(j) = partial.first() {
                                        values.hessians[(row * n + j) * n + i]
                                    } else {
                                        values.jacobian[row * n + i]
                                    }
                                }
                                LiftInput::Second(i, j) => values.hessians[row * n * n + i * n + j],
                                LiftInput::Argument(i, k) => {
                                    frame[inputs[i] * width + k] * layout.raw_factor(k)
                                }
                            };
                        }
                        lift.evaluator
                            .try_evaluate(&lift.scratch, &mut lift.output)
                            .map_err(|e| MathError::Evaluation {
                                source_id: *source,
                                order: layout.order,
                                detail: e.to_string(),
                            })?;
                        if lift.output.iter().any(|v| !v.is_finite()) {
                            return Err(MathError::Domain {
                                source_id: *source,
                                requirement: "finite composed provider jet",
                            });
                        }
                        frame[slot * width..(slot + 1) * width].copy_from_slice(&lift.output);
                    } else {
                        frame[slot * width..(slot + 1) * width].fill(0.0);
                        frame[slot * width] = selected;
                    }
                }
            }
        }
    }
    Ok(())
}

fn symbol_map(parameters: &[Atom]) -> HashMap<Symbol, usize> {
    parameters
        .iter()
        .enumerate()
        .flat_map(|(i, a)| a.get_all_symbols(false).into_iter().map(move |s| (s, i)))
        .collect()
}
fn reads(expressions: &[Atom], symbols: &HashMap<Symbol, usize>) -> Result<Vec<usize>, MathError> {
    expressions
        .iter()
        .flat_map(|e| e.get_all_symbols(false))
        .filter(|symbol| !Atom::var(*symbol).is_constant())
        .map(|s| {
            symbols
                .get(&s)
                .copied()
                .ok_or_else(|| MathError::Contract("unknown block parameter".into()))
        })
        .collect::<Result<BTreeSet<_>, _>>()
        .map(|s| s.into_iter().collect())
}
fn has_obligations(stages: &[Stage]) -> bool {
    stages.iter().any(|s| {
        matches!(
            s,
            Stage::Require { .. }
                | Stage::Branch { .. }
                | Stage::Provider { .. }
                | Stage::Domain { .. }
                | Stage::Applicability { .. }
        )
    })
}
// Backward demand retains every authored obligation. Dead properties are excluded without hoisting.
fn prune(
    stages: &[Stage],
    needed: &mut BTreeSet<usize>,
    symbols: &HashMap<Symbol, usize>,
    all_effects: bool,
) -> Vec<Stage> {
    let mut result = vec![];
    for stage in stages.iter().rev() {
        match stage {
            Stage::Applicability {
                stages,
                predicates,
                inputs,
                token,
                plan,
            } => {
                if !needed.remove(token) && !all_effects {
                    continue;
                }
                let mut local = predicates
                    .iter()
                    .chain(inputs)
                    .copied()
                    .collect::<BTreeSet<_>>();
                let stages = prune(stages, &mut local, symbols, true);
                needed.extend(local);
                result.push(Stage::Applicability {
                    stages,
                    predicates: predicates.clone(),
                    inputs: inputs.clone(),
                    token: *token,
                    plan: plan.clone(),
                });
            }
            Stage::Domain {
                stages,
                argument,
                token,
                lineage,
            } => {
                if !needed.remove(token) && !all_effects {
                    continue;
                }
                let mut local = BTreeSet::from([*argument]);
                let stages = prune(stages, &mut local, symbols, true);
                needed.extend(local);
                result.push(Stage::Domain {
                    stages,
                    argument: *argument,
                    token: *token,
                    lineage: lineage.clone(),
                });
            }
            Stage::Block {
                expressions,
                outputs,
                source,
            } => {
                let chosen: Vec<_> = outputs
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| needed.contains(s))
                    .map(|(i, &s)| (expressions[i].clone(), s))
                    .collect();
                if !chosen.is_empty() {
                    for (_, s) in &chosen {
                        needed.remove(s);
                    }
                    for (e, _) in &chosen {
                        for symbol in e.get_all_symbols(false) {
                            if let Some(&slot) = symbols.get(&symbol) {
                                needed.insert(slot);
                            }
                        }
                    }
                    result.push(Stage::Block {
                        expressions: chosen.iter().map(|(e, _)| e.clone()).collect(),
                        outputs: chosen.iter().map(|(_, s)| *s).collect(),
                        source: *source,
                    });
                }
            }
            Stage::Require { argument, .. } => {
                if all_effects || needed.contains(argument) {
                    needed.insert(*argument);
                    result.push(stage.clone());
                }
            }
            Stage::Provider {
                spec,
                partial,
                inputs,
                outputs,
                source,
            } => {
                // The call itself is a fallible domain obligation, even after algebraic cancellation.
                let used = outputs.iter().any(|s| needed.contains(s));
                if !used && !all_effects {
                    continue;
                }
                let selected = outputs
                    .iter()
                    .map(|s| {
                        if needed.remove(s) || !used {
                            *s
                        } else {
                            usize::MAX
                        }
                    })
                    .collect();
                needed.extend(inputs);
                result.push(Stage::Provider {
                    spec: spec.clone(),
                    partial: partial.clone(),
                    inputs: inputs.clone(),
                    outputs: selected,
                    source: *source,
                });
            }
            Stage::Branch {
                continuity,
                comparison,
                left,
                right,
                then,
                otherwise,
            } => {
                let mut a = needed.clone();
                let mut b = needed.clone();
                let then = prune(then, &mut a, symbols, all_effects);
                let otherwise = prune(otherwise, &mut b, symbols, all_effects);
                if then.is_empty() && otherwise.is_empty() {
                    continue;
                }
                *needed = a.union(&b).copied().collect();
                needed.insert(*left);
                needed.insert(*right);
                result.push(Stage::Branch {
                    continuity: *continuity,
                    comparison: *comparison,
                    left: *left,
                    right: *right,
                    then,
                    otherwise,
                });
            }
        }
    }
    result.reverse();
    result
}

#[derive(Debug, Default)]
struct Fact {
    expression: Option<Atom>,
    first: BTreeSet<usize>,
    second: BTreeSet<(usize, usize)>,
    /// Actual producer occurrence, without inventing a source for formal inputs.
    source: Option<SemanticId>,
}
struct SupportAllowance<'a> {
    remaining: &'a mut usize,
    derivatives: usize,
}
impl SupportAllowance<'_> {
    fn consume(&mut self, required: usize, source_id: SemanticId) -> Result<(), MathError> {
        if required > *self.remaining {
            return Err(MathError::WorkLimit {
                source_id,
                resource: "derivative support construction",
                required,
                available: *self.remaining,
                // Support preparation precedes selection of a Taylor layout.
                components: 0,
            });
        }
        *self.remaining -= required;
        Ok(())
    }
    fn derivative(&mut self, source: SemanticId) -> Result<(), MathError> {
        self.consume(1, source)?;
        self.derivatives = self
            .derivatives
            .checked_add(1)
            .ok_or(MathError::Limit("support derivative calls"))?;
        Ok(())
    }
    fn insert(
        &mut self,
        target: &mut BTreeSet<(usize, usize)>,
        pair: (usize, usize),
        source: SemanticId,
    ) -> Result<(), MathError> {
        self.consume(1, source)?;
        target.insert(pair);
        Ok(())
    }
    fn extend(
        &mut self,
        target: &mut BTreeSet<(usize, usize)>,
        values: &BTreeSet<(usize, usize)>,
        source: SemanticId,
    ) -> Result<(), MathError> {
        self.consume(values.len(), source)?;
        target.extend(values);
        Ok(())
    }
    fn clone_facts(
        &mut self,
        facts: &[Arc<Fact>],
        source: SemanticId,
    ) -> Result<Vec<Arc<Fact>>, MathError> {
        // Snapshot immutable slot handles; no accumulated support set is copied.
        self.consume(facts.len(), source)?;
        Ok(facts.to_vec())
    }
    fn equal_facts(&mut self, a: &Fact, b: &Fact, source: SemanticId) -> Result<bool, MathError> {
        // A distinct allocation can still denote exactly the same symbolic fact. Charge
        // a conservative byte-comparison bound before the library equality operation,
        // then charge each support entry comparison that is actually performed.
        let bytes = match (&a.expression, &b.expression) {
            (Some(a), Some(b)) => a
                .as_view()
                .get_byte_size()
                .checked_add(b.as_view().get_byte_size())
                .ok_or(MathError::Limit("support comparison extent"))?,
            _ => 1,
        };
        self.consume(bytes, source)?;
        if a.expression != b.expression
            || a.first.len() != b.first.len()
            || a.second.len() != b.second.len()
        {
            return Ok(false);
        }
        for (a, b) in a.first.iter().zip(&b.first) {
            self.consume(1, source)?;
            if a != b {
                return Ok(false);
            }
        }
        for (a, b) in a.second.iter().zip(&b.second) {
            self.consume(1, source)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
fn dense_second(
    first: &BTreeSet<usize>,
    allowance: &mut SupportAllowance<'_>,
    source: SemanticId,
) -> Result<BTreeSet<(usize, usize)>, MathError> {
    let n = first.len();
    let count = n
        .checked_add(1)
        .and_then(|next| n.checked_mul(next))
        .map(|count| count / 2)
        .ok_or(MathError::Limit("derivative support cardinality"))?;
    allowance.consume(count, source)?;
    Ok(first
        .iter()
        .flat_map(|&i| first.range(i..).map(move |&j| (i, j)))
        .collect())
}
#[allow(
    clippy::too_many_arguments,
    reason = "Analysis propagates separate facts and obligation inventories through branches"
)]
fn analyze(
    stages: &[Stage],
    parameters: &[Atom],
    symbols: &HashMap<Symbol, usize>,
    facts: &mut [Arc<Fact>],
    controls: &mut BTreeSet<usize>,
    switches: &mut BTreeSet<usize>,
    providers: &mut BTreeMap<ProviderKey, ProviderSpec>,
    obligations: &mut Vec<(Option<Atom>, Condition)>,
    allowance: &mut SupportAllowance<'_>,
    requested: DerivativeOrder,
    coordinates: Option<&BTreeSet<usize>>,
    cancel: Option<&Arc<AtomicBool>>,
    numeric: Option<&BTreeSet<usize>>,
) -> Result<(), MathError> {
    for stage in stages {
        if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
            return Err(MathError::Cancelled);
        }
        match stage {
            Stage::Applicability {
                stages,
                predicates,
                inputs,
                token,
                plan,
            } => {
                let source = plan.claim.form;
                let mut local = allowance.clone_facts(facts, source)?;
                analyze(
                    stages,
                    parameters,
                    symbols,
                    &mut local,
                    controls,
                    &mut BTreeSet::new(),
                    providers,
                    obligations,
                    allowance,
                    requested.min(DerivativeOrder::First),
                    coordinates,
                    cancel,
                    numeric,
                )?;
                for slot in predicates.iter().chain(inputs) {
                    allowance.consume(local[*slot].first.len(), source)?;
                    controls.extend(&local[*slot].first);
                }
                allowance.consume(1, source)?;
                facts[*token] = Arc::new(Fact {
                    expression: Some(Atom::num(0)),
                    source: Some(source),
                    ..Fact::default()
                });
            }
            Stage::Domain {
                stages,
                argument,
                token,
                lineage,
            } => {
                let source = lineage.source;
                let mut local = allowance.clone_facts(facts, source)?;
                // Domain boundaries do not constitute branch transitions of a numerical output.
                analyze(
                    stages,
                    parameters,
                    symbols,
                    &mut local,
                    controls,
                    &mut BTreeSet::new(),
                    providers,
                    obligations,
                    allowance,
                    requested.min(DerivativeOrder::First),
                    coordinates,
                    cancel,
                    numeric,
                )?;
                allowance.consume(local[*argument].first.len(), source)?;
                controls.extend(&local[*argument].first);
                obligations.push((local[*argument].expression.clone(), Condition::Positive));
                allowance.consume(1, source)?;
                facts[*token] = Arc::new(Fact {
                    expression: Some(Atom::num(0)),
                    source: Some(source),
                    ..Fact::default()
                });
            }
            Stage::Block {
                expressions,
                outputs,
                source,
            } => {
                for (expression, &slot) in expressions.iter().zip(outputs) {
                    let inputs = reads(std::slice::from_ref(expression), symbols)?;
                    for &i in &inputs {
                        allowance.consume(facts[i].first.len(), *source)?;
                    }
                    let first = if requested > DerivativeOrder::Value {
                        inputs
                            .iter()
                            .flat_map(|&i| facts[i].first.iter().copied())
                            .collect::<BTreeSet<_>>()
                    } else {
                        BTreeSet::new()
                    };
                    // Flattening is only an optional support/coefficient optimization.
                    // Bound substitution BEFORE allocating the expanded tree. Every symbol
                    // occurrence occupies at least one source byte; the byte product is a
                    // conservative replacement bound. Larger DAGs retain complete support.
                    let expanded = inputs.iter().try_fold(expression.clone(), |e, &i| {
                        let value = facts[i].expression.as_ref()?;
                        let bytes = e
                            .as_view()
                            .get_byte_size()
                            .checked_mul(value.as_view().get_byte_size().checked_add(1)?)?;
                        if bytes > 1024 * 1024 {
                            return None;
                        }
                        let result = e.replace(parameters[i].clone()).with(value.clone());
                        (operation_count(result.count_operations()) <= 16384).then_some(result)
                    });
                    let mut fact = Fact {
                        expression: expanded,
                        second: BTreeSet::new(),
                        first,
                        source: Some(*source),
                    };
                    if requested > DerivativeOrder::Value
                        && let Some(expr) = &fact.expression
                    {
                        if operation_count(expr.count_operations()) > 16384 {
                            return Err(MathError::Limit("local symbolic support expansion"));
                        }
                        let first = reads(std::slice::from_ref(expr), symbols)?;
                        allowance.consume(first.len(), *source)?;
                        fact.first = first
                            .into_iter()
                            .filter(|i| coordinates.is_none_or(|c| c.contains(i)))
                            .collect();
                        fact.second.clear();
                        if requested >= DerivativeOrder::Second
                            && numeric.is_none_or(|n| n.contains(&slot))
                        {
                            for &i in &fact.first {
                                allowance.derivative(*source)?;
                                let d = expr.derivative(
                                    Indeterminate::try_from(parameters[i].clone())
                                        .map_err(|e| MathError::Library(e.clone()))?,
                                );
                                for j in reads(std::slice::from_ref(&d), symbols)?
                                    .into_iter()
                                    .filter(|j| coordinates.is_none_or(|c| c.contains(j)))
                                {
                                    allowance.insert(
                                        &mut fact.second,
                                        (i.min(j), i.max(j)),
                                        *source,
                                    )?;
                                }
                            }
                        }
                    }
                    if requested >= DerivativeOrder::Second
                        && fact.expression.is_none()
                        && numeric.is_none_or(|n| n.contains(&slot))
                    {
                        // Compose support through the shared program. Losing the optional
                        // flattened expression does not make unrelated coordinates nonlinear.
                        // Symbolica owns each local derivative; only its dependency sets
                        // are propagated here (the two terms of the Hessian chain rule).
                        for &i in &inputs {
                            allowance.derivative(*source)?;
                            let derivative = expression.derivative(
                                Indeterminate::try_from(parameters[i].clone())
                                    .map_err(|e| MathError::Library(e.clone()))?,
                            );
                            if derivative == Atom::num(0) {
                                continue;
                            }
                            allowance.extend(&mut fact.second, &facts[i].second, *source)?;
                            for j in reads(std::slice::from_ref(&derivative), symbols)? {
                                for &a in &facts[i].first {
                                    for &b in &facts[j].first {
                                        allowance.insert(
                                            &mut fact.second,
                                            (a.min(b), a.max(b)),
                                            *source,
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                    allowance.consume(1, *source)?;
                    facts[slot] = Arc::new(fact);
                }
            }
            Stage::Require {
                argument,
                condition,
                order,
                source,
                ..
            } => {
                if requested >= *order {
                    allowance.consume(facts[*argument].first.len(), *source)?;
                    controls.extend(&facts[*argument].first);
                }
                if *order == DerivativeOrder::Value {
                    obligations.push((facts[*argument].expression.clone(), *condition));
                }
            }
            Stage::Provider {
                spec,
                inputs,
                outputs,
                source,
                ..
            } => {
                providers.insert(spec.key(), spec.clone());
                for &i in inputs {
                    allowance.consume(facts[i].first.len(), *source)?;
                }
                let first = inputs
                    .iter()
                    .flat_map(|&i| facts[i].first.iter().copied())
                    .collect::<BTreeSet<_>>();
                for &slot in outputs.iter().filter(|&&s| s != usize::MAX) {
                    allowance.consume(first.len(), *source)?;
                    allowance.consume(1, *source)?;
                    facts[slot] = Arc::new(Fact {
                        expression: None,
                        first: first.clone(),
                        second: if requested >= DerivativeOrder::Second
                            && numeric.is_none_or(|n| n.contains(&slot))
                        {
                            dense_second(&first, allowance, *source)?
                        } else {
                            BTreeSet::new()
                        },
                        source: Some(*source),
                    });
                }
            }
            Stage::Branch {
                continuity,
                left,
                right,
                then,
                otherwise,
                ..
            } => {
                let source = facts[*left]
                    .source
                    .or(facts[*right].source)
                    .unwrap_or(SemanticId::NIL);
                allowance.consume(facts[*left].first.len(), source)?;
                allowance.consume(facts[*right].first.len(), source)?;
                controls.extend(&facts[*left].first);
                controls.extend(&facts[*right].first);
                if *continuity == DerivativeOrder::Value {
                    switches.extend(&facts[*left].first);
                    switches.extend(&facts[*right].first);
                }
                let mut a = allowance.clone_facts(facts, source)?;
                let mut b = allowance.clone_facts(facts, source)?;
                analyze(
                    then,
                    parameters,
                    symbols,
                    &mut a,
                    controls,
                    switches,
                    providers,
                    obligations,
                    allowance,
                    requested,
                    coordinates,
                    cancel,
                    numeric,
                )?;
                analyze(
                    otherwise,
                    parameters,
                    symbols,
                    &mut b,
                    controls,
                    switches,
                    providers,
                    obligations,
                    allowance,
                    requested,
                    coordinates,
                    cancel,
                    numeric,
                )?;
                for i in 0..facts.len() {
                    if Arc::ptr_eq(&a[i], &b[i]) {
                        if !Arc::ptr_eq(&facts[i], &a[i]) {
                            allowance.consume(1, source)?;
                            facts[i] = a[i].clone();
                        }
                        continue;
                    }
                    if !allowance.equal_facts(&a[i], &b[i], source)? {
                        let mut second = BTreeSet::new();
                        allowance.extend(&mut second, &a[i].second, source)?;
                        allowance.extend(&mut second, &b[i].second, source)?;
                        allowance.consume(a[i].first.len(), source)?;
                        allowance.consume(b[i].first.len(), source)?;
                        allowance.consume(1, source)?;
                        facts[i] = Arc::new(Fact {
                            expression: None,
                            first: a[i].first.union(&b[i].first).copied().collect(),
                            second,
                            source: a[i].source.or(b[i].source),
                        });
                    } else {
                        allowance.consume(1, source)?;
                        facts[i] = a[i].clone();
                    }
                }
            }
        }
    }
    Ok(())
}

fn applicability_extent(stages: &[Stage]) -> usize {
    stages.iter().fold(0usize, |sum, stage| {
        sum.saturating_add(match stage {
            Stage::Applicability { stages, plan, .. } => plan
                .observation_bytes()
                .saturating_add(applicability_extent(stages)),
            Stage::Domain { stages, .. } => applicability_extent(stages),
            Stage::Branch {
                then, otherwise, ..
            } => applicability_extent(then).saturating_add(applicability_extent(otherwise)),
            _ => 0,
        })
    })
}
