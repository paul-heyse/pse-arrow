// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable preparation, effect-owned compilation and worker-local library evaluation.
use crate::{
    MathError,
    guarded::{Comparison, Condition, Stage, validate_dependencies},
    jets::{EvaluationLimits, JetLayout, LiftInput, ProviderLift},
    library::{self, Optimization},
};
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
    pub(crate) smooth: DerivativeOrder,
    support: Support,
    switches: BTreeSet<usize>,
    pub(crate) obligations: Vec<(Option<Atom>, Condition)>,
    input_quantities: Vec<Option<pse_quantity::QuantityTypeId>>,
    output_quantities: Vec<pse_quantity::QuantityTypeId>,
    expressions: Vec<Option<Atom>>,
    providers: Vec<ProviderSpec>,
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
            && self.smooth == other.smooth
            && self.support == other.support
            && self.switches == other.switches
            && self.obligations == other.obligations
            && self.input_quantities == other.input_quantities
            && self.output_quantities == other.output_quantities
            && self.expressions == other.expressions
            && self.providers == other.providers
    }
}
impl PreparedBody {
    /// Known symbolic payload and container contents. Tree/allocator overhead and
    /// library-global interners require a separately labelled foreign allowance.
    pub fn retained_bytes(&self) -> usize {
        fn stages(items: &[Stage]) -> usize {
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
                        } => stages(then) + stages(otherwise),
                        Stage::Provider {
                            inputs,
                            outputs,
                            spec,
                            ..
                        } => {
                            size_of_val(inputs.as_slice())
                                + size_of_val(outputs.as_slice())
                                + size_of_val(spec.inputs.as_slice())
                                + size_of_val(spec.outputs.as_slice())
                                + size_of_val(spec.components.as_slice())
                        }
                        Stage::Require { .. } => 0,
                    })
                    .sum::<usize>()
        }
        size_of::<PreparedBodyData>()
            + stages(&self.stages)
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
            + self
                .support
                .first
                .iter()
                .map(|s| s.len() * size_of::<usize>())
                .sum::<usize>()
            + self
                .support
                .second
                .iter()
                .map(|s| s.len() * size_of::<(usize, usize)>())
                .sum::<usize>()
    }
    /// Retain accounting when a body clone outlives its runtime case plan.
    pub fn with_owner(mut self, owner: Arc<dyn crate::AllocationOwner>) -> Self {
        self.owner = Some(owner);
        self
    }
    pub(crate) fn new(
        inputs: usize,
        slots: usize,
        outputs: Vec<usize>,
        stages: Vec<Stage>,
        smooth: DerivativeOrder,
    ) -> Result<Self, MathError> {
        if slots == 0
            || inputs > slots
            || slots > library::MAX_FORMAL_SYMBOLS
            || outputs.is_empty()
            || outputs.iter().any(|&i| i >= slots)
        {
            return Err(MathError::Contract("prepared body layout".into()));
        }
        let parameters = (0..slots)
            .map(library::formal)
            .collect::<Result<Vec<_>, _>>()?;
        let symbols = symbol_map(&parameters);
        let mut assigned = (0..inputs).collect::<BTreeSet<_>>();
        validate_dependencies(&stages, &symbols, &mut assigned, 0, &mut 16384)?;
        if outputs.iter().any(|i| !assigned.contains(i)) {
            return Err(MathError::Contract(
                "output unassigned on some branch".into(),
            ));
        }
        let mut facts = vec![Fact::default(); slots];
        for i in 0..inputs {
            facts[i] = Fact {
                expression: Some(parameters[i].clone()),
                first: BTreeSet::from([i]),
                second: BTreeSet::new(),
            };
        }
        let mut controls = BTreeSet::new();
        let mut switches = BTreeSet::new();
        let mut providers = BTreeMap::new();
        let mut obligations = vec![];
        analyze(
            &stages,
            &parameters,
            &symbols,
            &mut facts,
            &mut controls,
            &mut switches,
            &mut providers,
            &mut obligations,
        )?;
        let support = Support {
            first: outputs.iter().map(|&i| facts[i].first.clone()).collect(),
            second: outputs.iter().map(|&i| facts[i].second.clone()).collect(),
            controls,
        };
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
                smooth,
                support,
                switches,
                obligations,
                input_quantities: vec![None; inputs],
                output_quantities: vec![],
                expressions,
                providers: providers.into_values().collect(),
            }),
            owner: None,
        })
    }
    /// Smooth derivative capability established by admitted operations and providers.
    pub fn available_order(&self) -> DerivativeOrder {
        if self.switches.is_empty() {
            self.smooth
        } else {
            DerivativeOrder::Value
        }
    }
    /// Fixed external guards do not reduce derivatives with respect to selected free coordinates.
    pub fn available_order_for(&self, coordinates: &[usize]) -> DerivativeOrder {
        if coordinates.iter().any(|c| self.switches.contains(c)) {
            DerivativeOrder::Value
        } else {
            self.smooth
        }
    }
    /// Number of formal inputs.
    pub fn input_count(&self) -> usize {
        self.inputs
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
    /// Complete support, including all branch alternatives.
    pub fn support(&self) -> &Support {
        &self.support
    }
    /// Consumed executable contracts, keyed by full physical/data interpretation.
    pub fn providers(&self) -> &[ProviderSpec] {
        &self.providers
    }
    /// Library expression when no opaque provider or branch prevents exact projection.
    pub fn expression(&self, output: usize) -> Option<&Atom> {
        self.expressions.get(output).and_then(Option::as_ref)
    }
    /// Whether coefficient views must retain a domain/control obligation.
    pub fn has_obligations(&self) -> bool {
        has_obligations(&self.stages)
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
        limits.check()?;
        if cancelled.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        if outputs.is_empty()
            || outputs.iter().any(|&i| i >= self.outputs.len())
            || outputs.iter().collect::<BTreeSet<_>>().len() != outputs.len()
            || coordinates.iter().any(|&i| i >= self.inputs)
        {
            return Err(MathError::Contract(
                "compiled output/coordinate demand".into(),
            ));
        }
        if order > self.smooth
            || (order > DerivativeOrder::Value
                && coordinates.iter().any(|c| self.switches.contains(c)))
        {
            return Err(MathError::Contract(
                "nonsmooth operation lacks a derivative neighborhood".into(),
            ));
        }
        let selected: Vec<_> = outputs.iter().map(|&i| self.outputs[i]).collect();
        let parameters = (0..self.slots)
            .map(library::formal)
            .collect::<Result<Vec<_>, _>>()?;
        let symbols = symbol_map(&parameters);
        let mut needed: BTreeSet<_> = selected.iter().copied().collect();
        if let Some(effects) = &self.output_effects {
            needed.extend(outputs.iter().flat_map(|&i| effects[i].iter().copied()));
        }
        let stages = prune(
            &self.stages,
            &mut needed,
            &symbols,
            self.output_effects.is_none(),
        );
        let mut layouts = vec![];
        let mut programs = vec![];
        let mut used = 0usize;
        let mut retained_numeric = 0usize;
        for requested in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
        ]
        .into_iter()
        .take(order as usize + 1)
        {
            let layout = JetLayout::new(coordinates.to_vec(), requested, limits)?;
            let frame = self
                .slots
                .checked_mul(layout.width())
                .ok_or(MathError::Limit("jet frame"))?;
            limits.allocation(frame)?;
            let mut allowance = BuildAllowance {
                operations: limits.operations,
                providers: limits.provider_calls,
                entries: frame,
            };
            let program = compile_stages(
                &stages,
                &parameters,
                &symbols,
                &layout,
                options,
                limits,
                cancelled,
                &mut allowance,
            )?;
            retained_numeric = retained_numeric
                .checked_add(allowance.entries - frame)
                .ok_or(MathError::Limit("retained numeric storage"))?;
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
            layouts.push(layout);
            programs.push(program);
        }
        Ok(CompiledBody {
            owner: None,
            inputs: self.inputs,
            slots: self.slots,
            scratch_bytes: used * size_of::<f64>(),
            retained_numeric_bytes: retained_numeric * size_of::<f64>(),
            outputs: Arc::new(selected),
            layouts: Arc::new(layouts),
            programs: Arc::new(programs),
            limits,
            support: Arc::new(Support {
                first: outputs
                    .iter()
                    .map(|&i| self.support.first[i].clone())
                    .collect(),
                second: outputs
                    .iter()
                    .map(|&i| self.support.second[i].clone())
                    .collect(),
                controls: self.support.controls.clone(),
            }),
        })
    }
}

#[derive(Clone)]
#[allow(
    clippy::large_enum_variant,
    reason = "Evaluator stages stay inline to avoid an allocation for each compiled stage"
)]
enum CompiledStage {
    Block {
        evaluator: ExpressionEvaluator<f64>,
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
    },
    Branch {
        comparison: Comparison,
        left: usize,
        right: usize,
        then: Vec<Self>,
        otherwise: Vec<Self>,
    },
    Provider {
        spec: ProviderSpec,
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
    retained_numeric_bytes: usize,
    inputs: usize,
    slots: usize,
    outputs: Arc<Vec<usize>>,
    layouts: Arc<Vec<JetLayout>>,
    programs: Arc<Vec<Vec<CompiledStage>>>,
    limits: EvaluationLimits,
    support: Arc<Support>,
}
impl std::fmt::Debug for CompiledBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledBody")
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs.len())
            .field("profiles", &self.layouts.len())
            .finish()
    }
}
impl CompiledBody {
    /// Attach accounting to the allocation itself so evaluator/worker clones retain it.
    pub fn with_owner(mut self, owner: Arc<dyn crate::AllocationOwner>) -> Self {
        self.owner = Some(owner);
        self
    }
    /// Upper bound on owned numeric frame and stage scratch.
    pub fn scratch_bytes(&self) -> usize {
        self.scratch_bytes
    }
    /// Numeric buffers retained in the immutable evaluator templates, excluding
    /// attempt frames and returned derivative buffers. Foreign heaps are separate.
    pub fn retained_numeric_bytes(&self) -> usize {
        self.retained_numeric_bytes
    }
    /// Independent mutable scratch; caller/attempt owns its provider worker map.
    pub fn worker(&self) -> Worker {
        let width = self.layouts.last().map_or(1, JetLayout::width);
        Worker {
            body: self.clone(),
            programs: self.programs.as_ref().clone(),
            frame: vec![0.0; self.slots * width],
        }
    }
    /// Structural support survives trials and numerical zeros.
    pub fn support(&self) -> &Support {
        &self.support
    }
    /// Differentiation coordinates in formal input order.
    pub fn coordinates(&self) -> &[usize] {
        self.layouts[0].coordinates()
    }
}
/// Atomic result, with raw derivatives in output-major row-major order.
#[derive(Clone, Debug, PartialEq)]
pub struct Evaluation {
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
    programs: Vec<Vec<CompiledStage>>,
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
        let layout = self
            .body
            .layouts
            .get(order as usize)
            .ok_or_else(|| MathError::Contract("uncompiled derivative order".into()))?;
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
        evaluate_stages(
            &mut self.programs[order as usize],
            &mut self.frame,
            layout,
            providers,
            &context,
        )?;
        context.check().map_err(|cause| MathError::Provider {
            source_id: SemanticId::NIL,
            provider: SemanticId::NIL,
            cause,
        })?;
        let mut result = Evaluation {
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

struct BuildAllowance {
    operations: usize,
    providers: usize,
    entries: usize,
}
#[allow(
    clippy::too_many_arguments,
    reason = "Compilation receives distinct symbol, budget, and cancellation contexts"
)]
fn compile_stages(
    stages: &[Stage],
    parameters: &[Atom],
    symbols: &HashMap<Symbol, usize>,
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
                let evaluator = library::bounded_evaluator(
                    expressions,
                    &params,
                    layout,
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
                if input_len != inputs.len() * layout.width()
                    || output_len != outputs.len() * layout.width()
                {
                    return Err(MathError::Contract("library vectorization layout".into()));
                }
                allowance.entries = allowance
                    .entries
                    .checked_add(input_len + output_len + library::numeric_entries(&evaluator)?)
                    .ok_or(MathError::Limit("evaluator scratch"))?;
                limits.allocation(allowance.entries)?;
                CompiledStage::Block {
                    evaluator,
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
            } => CompiledStage::Require {
                argument: *argument,
                condition: *condition,
                order: *order,
                source: *source,
            },
            Stage::Branch {
                comparison,
                left,
                right,
                then,
                otherwise,
            } => CompiledStage::Branch {
                comparison: *comparison,
                left: *left,
                right: *right,
                then: compile_stages(
                    then, parameters, symbols, layout, options, limits, cancelled, allowance,
                )?,
                otherwise: compile_stages(
                    otherwise, parameters, symbols, layout, options, limits, cancelled, allowance,
                )?,
            },
            Stage::Provider {
                spec,
                inputs,
                outputs,
                source,
            } => {
                allowance.providers = allowance
                    .providers
                    .checked_sub(1)
                    .ok_or(MathError::Limit("provider calls"))?;
                let request = ProviderRequest {
                    outputs: outputs
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &slot)| (slot != usize::MAX).then_some(i))
                        .collect(),
                    order: layout.order,
                };
                let destinations = outputs
                    .iter()
                    .copied()
                    .filter(|&s| s != usize::MAX)
                    .collect();
                let lift = if layout.width() > 1 {
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
) -> Result<(), MathError> {
    let width = layout.width();
    for stage in stages {
        if context.cancelled.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        match stage {
            CompiledStage::Block {
                evaluator,
                inputs,
                outputs,
                arguments,
                values,
                source,
            } => {
                for (k, &slot) in inputs.iter().enumerate() {
                    arguments[k * width..(k + 1) * width]
                        .copy_from_slice(&frame[slot * width..(slot + 1) * width]);
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
                    frame[slot * width..(slot + 1) * width]
                        .copy_from_slice(&values[k * width..(k + 1) * width]);
                }
            }
            CompiledStage::Require {
                argument,
                condition,
                order,
                source,
            } => {
                if layout.order >= *order && !condition.permits(frame[*argument * width]) {
                    return Err(MathError::Domain {
                        source_id: *source,
                        requirement: condition.description(),
                    });
                }
            }
            CompiledStage::Branch {
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
                )?;
            }
            CompiledStage::Provider {
                spec,
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
                if provider.spec() != spec {
                    return Err(MathError::Contract(
                        "provider worker changed its contract".into(),
                    ));
                }
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
                    if let Some(lift) = lift.as_mut() {
                        let n = inputs.len();
                        for (parameter, value) in lift.inputs.iter().zip(&mut lift.scratch) {
                            *value = match *parameter {
                                LiftInput::Value => values.values[row],
                                LiftInput::First(i) => values.jacobian[row * n + i],
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
                        frame[slot * width] = values.values[row];
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
            Stage::Require { .. } | Stage::Branch { .. } | Stage::Provider { .. }
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
                    inputs: inputs.clone(),
                    outputs: selected,
                    source: *source,
                });
            }
            Stage::Branch {
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

#[derive(Clone, Debug, Default)]
struct Fact {
    expression: Option<Atom>,
    first: BTreeSet<usize>,
    second: BTreeSet<(usize, usize)>,
}
fn dense_second(first: &BTreeSet<usize>) -> BTreeSet<(usize, usize)> {
    first
        .iter()
        .flat_map(|&i| first.range(i..).map(move |&j| (i, j)))
        .collect()
}
#[allow(
    clippy::too_many_arguments,
    reason = "Analysis propagates separate facts and obligation inventories through branches"
)]
fn analyze(
    stages: &[Stage],
    parameters: &[Atom],
    symbols: &HashMap<Symbol, usize>,
    facts: &mut [Fact],
    controls: &mut BTreeSet<usize>,
    switches: &mut BTreeSet<usize>,
    providers: &mut BTreeMap<ProviderKey, ProviderSpec>,
    obligations: &mut Vec<(Option<Atom>, Condition)>,
) -> Result<(), MathError> {
    for stage in stages {
        match stage {
            Stage::Block {
                expressions,
                outputs,
                ..
            } => {
                for (expression, &slot) in expressions.iter().zip(outputs) {
                    let inputs = reads(std::slice::from_ref(expression), symbols)?;
                    let first = inputs
                        .iter()
                        .flat_map(|&i| facts[i].first.iter().copied())
                        .collect::<BTreeSet<_>>();
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
                    };
                    if let Some(expr) = &fact.expression {
                        if operation_count(expr.count_operations()) > 16384 {
                            return Err(MathError::Limit("local symbolic support expansion"));
                        }
                        fact.first = reads(std::slice::from_ref(expr), symbols)?
                            .into_iter()
                            .collect();
                        fact.second.clear();
                        for &i in &fact.first {
                            let d = expr.derivative(
                                Indeterminate::try_from(parameters[i].clone())
                                    .map_err(|e| MathError::Library(e.clone()))?,
                            );
                            for j in reads(std::slice::from_ref(&d), symbols)? {
                                fact.second.insert((i.min(j), i.max(j)));
                            }
                        }
                    }
                    if fact.expression.is_none() {
                        if fact.first.len() > 256 {
                            return Err(MathError::Limit("opaque derivative support"));
                        }
                        fact.second = dense_second(&fact.first);
                    }
                    facts[slot] = fact;
                }
            }
            Stage::Require {
                argument,
                condition,
                order,
                ..
            } => {
                controls.extend(&facts[*argument].first);
                if *order == DerivativeOrder::Value {
                    obligations.push((facts[*argument].expression.clone(), *condition));
                }
            }
            Stage::Provider {
                spec,
                inputs,
                outputs,
                ..
            } => {
                providers.insert(spec.key(), spec.clone());
                let first = inputs
                    .iter()
                    .flat_map(|&i| facts[i].first.iter().copied())
                    .collect::<BTreeSet<_>>();
                if first.len() > 256 {
                    return Err(MathError::Limit("provider support width"));
                }
                for &slot in outputs {
                    facts[slot] = Fact {
                        expression: None,
                        first: first.clone(),
                        second: dense_second(&first),
                    };
                }
            }
            Stage::Branch {
                left,
                right,
                then,
                otherwise,
                ..
            } => {
                controls.extend(&facts[*left].first);
                controls.extend(&facts[*right].first);
                switches.extend(&facts[*left].first);
                switches.extend(&facts[*right].first);
                let mut a = facts.to_vec();
                let mut b = facts.to_vec();
                analyze(
                    then,
                    parameters,
                    symbols,
                    &mut a,
                    controls,
                    switches,
                    providers,
                    obligations,
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
                )?;
                for i in 0..facts.len() {
                    if a[i].expression != b[i].expression
                        || a[i].first != b[i].first
                        || a[i].second != b[i].second
                    {
                        facts[i] = Fact {
                            expression: None,
                            first: a[i].first.union(&b[i].first).copied().collect(),
                            second: a[i].second.union(&b[i].second).copied().collect(),
                        };
                    } else {
                        facts[i] = a[i].clone();
                    }
                }
            }
        }
    }
    Ok(())
}
