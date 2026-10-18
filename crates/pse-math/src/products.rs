// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library-owned requested products. Runtime weights and seeds are ordinary parameters.
use super::*;
#[cfg(test)]
#[path = "product_guard_tests.rs"]
mod product_guard_tests;

#[derive(Clone)]
pub(super) struct DirectProduct {
    evaluator: BlockEvaluator,
    pub(super) order: DerivativeOrder,
    pairs: Arc<Vec<(usize, usize)>>,
    inputs: usize,
    outputs: usize,
    directional_gradient: bool,
    arguments: usize,
    results: usize,
    scratch_bytes: usize,
    pub(super) retained_bytes: usize,
}
pub(super) struct DirectWorker {
    evaluator: BlockEvaluator,
    arguments: Vec<f64>,
    values: Vec<f64>,
}
impl DirectWorker {
    pub(super) fn reset(mut self) -> Result<Self, MathError> {
        self.evaluator = self.evaluator.reset()?;
        self.arguments.fill(f64::NAN);
        self.values.fill(f64::NAN);
        Ok(self)
    }
}
// Selection intentionally drops whole-body expression views. Reconstruct a bounded
// temporary library expression only for this concrete product, never as another model
// authority; original stage execution remains the source/guard authority.
fn flat_expressions(
    body: &PreparedBody,
    limits: EvaluationLimits,
    cancel: &Arc<AtomicBool>,
) -> Result<Option<Vec<Atom>>, MathError> {
    if !body.is_flat_arithmetic() {
        return Ok(None);
    }
    let parameters = body.parameters()?;
    let symbols = symbol_map(&parameters);
    let mut expressions = BTreeMap::<usize, Atom>::new();
    let mut work = 0usize;
    for stage in &body.stages {
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        let Stage::Block {
            expressions: values,
            outputs,
            ..
        } = stage
        else {
            continue;
        };
        for (value, &output) in values.iter().zip(outputs) {
            let mut value = value.clone();
            for input in reads(std::slice::from_ref(&value), &symbols)? {
                if let Some(replacement) = expressions.get(&input) {
                    let bound = value
                        .as_view()
                        .get_byte_size()
                        .checked_mul(replacement.as_view().get_byte_size().saturating_add(1));
                    let Some(bound) = bound.filter(|n| *n <= limits.scratch_bytes.min(1024 * 1024))
                    else {
                        return Ok(None);
                    };
                    crate::construction::additional(bound)?;
                    value = value
                        .replace(parameters[input].clone())
                        .with(replacement.clone());
                    work = work
                        .checked_add(operation_count(value.count_operations()))
                        .ok_or(MathError::Limit("weighted flattening operations"))?;
                    if work > limits.operations {
                        return Ok(None);
                    }
                }
            }
            expressions.insert(output, value);
        }
    }
    Ok(Some(
        body.outputs
            .iter()
            .map(|slot| {
                expressions
                    .get(slot)
                    .cloned()
                    .unwrap_or_else(|| parameters[*slot].clone())
            })
            .collect(),
    ))
}

#[derive(Clone)]
pub(super) struct TransposeProduct {
    program: CompiledBody,
    inputs: usize,
    worker_bytes: usize,
    arguments_map: Vec<usize>,
}
pub(super) struct TransposeWorker {
    worker: Worker,
    arguments: Vec<f64>,
    trial: TrialEvaluation,
}
impl TransposeWorker {
    pub(super) fn reset(mut self) -> Result<Self, MathError> {
        self.worker.detach_controls();
        self.worker.reset_scratch()?;
        self.arguments.fill(f64::NAN);
        self.trial.clear();
        Ok(self)
    }
}
// A temporary weighted library product retains the original host stages and guards.
// Its one first row is the transpose action; no residual-output-by-parameter jet is
// materialized. This is product preparation, not another authored model authority.
fn weighted_stages(body: &PreparedBody, weights: usize, limits: EvaluationLimits)
    -> Result<PreparedBody, MathError> {
    let parameters = body.parameters()?;
    let inputs = body.inputs.checked_add(weights).ok_or(MathError::Limit("transpose inputs"))?;
    let output = body.slots.checked_add(weights).ok_or(MathError::Limit("transpose slots"))?;
    let offsets = (0..body.slots).map(|i| (i, if i < body.inputs { i } else { i + weights }))
        .collect::<BTreeMap<_, _>>();
    let temporary = body.formal_slots.iter().copied().max().unwrap_or(body.slots)
        .checked_add(body.slots).and_then(|v| v.checked_add(weights + 2))
        .ok_or(MathError::Limit("transpose temporary formals"))?;
    let replacement = (0..body.slots).map(|i| Ok((parameters[i].clone(),
        library::formal(temporary + i)?, library::formal(offsets[&i])?)))
        .collect::<Result<Vec<_>, MathError>>()?;
    fn rewrite(stages: &mut [Stage], replacement: &[(Atom, Atom, Atom)],
        remaining: &mut usize) -> Result<(), MathError> {
        for stage in stages {
            match stage {
                Stage::Block { expressions, .. } => {
                    for value in expressions {
                        *remaining = remaining.checked_sub(replacement.len().saturating_mul(2))
                            .ok_or(MathError::Limit("transpose source rewriting"))?;
                        for (old, temporary, _) in replacement.iter() {
                            *value = value.replace(old.clone()).with(temporary.clone());
                        }
                        for (_, temporary, new) in replacement.iter() {
                            *value = value.replace(temporary.clone()).with(new.clone());
                        }
                    }
                }
                Stage::Domain { stages, .. } | Stage::Applicability { stages, .. } => rewrite(stages, replacement, remaining)?,
                Stage::Branch { then, otherwise, .. } => { rewrite(then, replacement, remaining)?; rewrite(otherwise, replacement, remaining)?; }
                _ => {}
            }
        }
        Ok(())
    }
    limits.allocation(output.checked_add(1).ok_or(MathError::Limit("transpose slots"))?)?;
    let mut stages = body.stages.clone();
    let mut remaining = limits.operations;
    rewrite(&mut stages, &replacement, &mut remaining)?;
    lower_stages(&mut stages, &offsets)?;
    let mut weighted = Atom::num(0);
    for (i, &slot) in body.outputs.iter().enumerate() {
        weighted += library::formal(body.inputs + i)? * library::formal(offsets[&slot])?;
    }
    stages.push(Stage::Block { expressions: vec![weighted], outputs: vec![output], source: SemanticId::NIL });
    let mut allowance = remaining;
    let mut prepared = PreparedBody::new_with_allowance(inputs, output + 1, vec![output], stages, &mut allowance)?;
    Arc::make_mut(&mut prepared.data).checked_members = body.checked_members.clone();
    Ok(prepared)
}

impl PreparedSupport {
    /// Compile the mathematical product consumed by the caller. Numeric bindings do not
    /// participate in immutable program identity.
    pub fn compile_product(
        &self,
        kind: ProductKind,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<CompiledBody, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        self.compile_product_inner(kind, options, limits, scope, None)
    }
    /// Reuse exact scalar preparations while constructing a distinct requested product.
    pub fn compile_product_reusing(
        &self,
        kind: ProductKind,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
        scalars: &library::ScalarProducts,
    ) -> Result<CompiledBody, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        self.compile_product_inner(kind, options, limits, scope, Some(scalars))
    }
    fn compile_product_inner(
        &self,
        kind: ProductKind,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
        retained: Option<&library::ScalarProducts>,
    ) -> Result<CompiledBody, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        let cancel = scope.cancellation();
        let required = match kind {
            ProductKind::SelectedPartials => self.order(),
            ProductKind::JacobianAction | ProductKind::TransposeJacobianAction => DerivativeOrder::First,
            ProductKind::WeightedHessian | ProductKind::WeightedHessianAction => {
                DerivativeOrder::Second
            }
        };
        if self.order() != required {
            return Err(MathError::Contract("product support order mismatch".into()));
        }
        let outputs = (0..self.outputs().len()).collect::<Vec<_>>();
        let coordinates = self.dense_coordinates()?;
        if matches!(
            kind,
            ProductKind::WeightedHessian | ProductKind::WeightedHessianAction | ProductKind::TransposeJacobianAction
        ) && let Some(expressions) = flat_expressions(self.body(), limits, cancel)?
        {
            return self.compile_direct_product(
                kind,
                expressions,
                options,
                limits,
                scope,
                retained,
            );
        }
        if kind == ProductKind::TransposeJacobianAction {
            return self.compile_transpose_product(options, limits, scope, retained);
        }
        self.body().compile_scope(
            &outputs,
            &coordinates,
            required,
            options,
            limits,
            scope,
            false,
            kind,
            self,
            retained,
        )
    }
    fn compile_transpose_product(&self, options: Optimization, limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope, retained: Option<&library::ScalarProducts>) -> Result<CompiledBody, MathError> {
        let body = self.body();
        let coordinates = self.dense_coordinates()?;
        let weighted = weighted_stages(body, body.outputs.len(), limits)?;
        let support = weighted.prepare_support(&[0], &coordinates, DerivativeOrder::First, scope.cancellation())?;
        let program = support.body().compile_scope(&[0], &support.dense_coordinates()?, DerivativeOrder::First,
            options, limits, scope, false, ProductKind::SelectedPartials, &support, retained)?;
        let mut compiled = body.compile_scope(&(0..self.outputs().len()).collect::<Vec<_>>(),
            &coordinates, DerivativeOrder::Value, options, limits, scope, false,
            ProductKind::SelectedPartials, self, retained)?;
        let argument_bytes = (body.inputs + body.outputs.len()).checked_mul(size_of::<f64>())
            .ok_or(MathError::Limit("transpose arguments"))?;
        let worker_bytes = program.worker_bytes.checked_add(program.evaluation_cache_bytes)
            .and_then(|n| n.checked_add(argument_bytes)).ok_or(MathError::Limit("transpose worker"))?;
        compiled.worker_bytes = compiled.worker_bytes.checked_add(worker_bytes).ok_or(MathError::Limit("transpose worker"))?;
        compiled.scratch_bytes = compiled.scratch_bytes.checked_add(program.scratch_bytes)
            .and_then(|n| n.checked_add(argument_bytes)).ok_or(MathError::Limit("transpose scratch"))?;
        if compiled.scratch_bytes > limits.scratch_bytes { return Err(MathError::Limit("transpose scratch")); }
        compiled.retained_bytes = compiled.retained_bytes.checked_add(program.retained_bytes)
            .and_then(|n| n.checked_add(size_of::<TransposeProduct>())).ok_or(MathError::Limit("transpose program"))?;
        compiled.evaluation_cache_bytes = compiled.evaluation_cache_bytes.checked_add(coordinates.len()*size_of::<f64>())
            .ok_or(MathError::Limit("transpose cache"))?;
        let mut census = program.census[DerivativeOrder::First].ok_or_else(|| MathError::Contract("transpose first program missing".into()))?;
        census.product = ProductKind::TransposeJacobianAction;
        census.result_entries = coordinates.len();
        compiled.census[DerivativeOrder::First] = Some(census);
        compiled.product = ProductKind::TransposeJacobianAction;
        let arguments_map = program.input_formals().to_vec();
        compiled.transpose_product = Some(Arc::new(TransposeProduct { arguments_map, program, inputs: body.inputs,
            worker_bytes }));
        scope.check().map_err(crate::error::scope_error)?;
        Ok(compiled)
    }
    fn compile_direct_product(
        &self,
        kind: ProductKind,
        flattened: Vec<Atom>,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
        retained: Option<&library::ScalarProducts>,
    ) -> Result<CompiledBody, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        let cancel = scope.cancellation();
        let body = self.body();
        let selected = (0..self.outputs().len()).collect::<Vec<_>>();
        let coordinates = self.dense_coordinates()?;
        let available = body.selected_order(&selected, &coordinates, false)?;
        let order = if kind == ProductKind::TransposeJacobianAction { DerivativeOrder::First } else { DerivativeOrder::Second };
        if available < order {
            return Err(MathError::DerivativeDemand {
                source_id: None,
                body: None,
                outputs: self.outputs().to_vec(),
                coordinates: self.coordinates().to_vec(),
                requested: order,
                available,
            });
        }
        let mut parameters = body.parameters()?;
        parameters.truncate(body.inputs);
        let next_formal = body
            .formal_slots
            .iter()
            .copied()
            .max()
            .unwrap_or(body.slots)
            .checked_add(1)
            .ok_or(MathError::Limit("product formal identity"))?;
        let weighted_extent = flattened
            .iter()
            .try_fold(0usize, |n, expression| {
                n.checked_add(expression.as_view().get_byte_size())
            })
            .ok_or(MathError::Limit("weighted symbolic input extent"))?;
        crate::construction::additional(
            weighted_extent
                .checked_mul(4)
                .ok_or(MathError::Limit("weighted symbolic input extent"))?,
        )?;
        let mut weighted = Atom::num(0);
        for expression in &flattened {
            let weight = library::formal(next_formal + parameters.len() - body.inputs)?;
            parameters.push(weight.clone());
            weighted += weight * expression;
        }
        let n = coordinates.len();
        let axes = self
            .coordinates()
            .iter()
            .enumerate()
            .map(|(axis, &formal)| (formal, axis))
            .collect::<BTreeMap<_, _>>();
        let pairs = self
            .support()
            .second
            .iter()
            .flatten()
            .filter_map(|&(i, j)| Some((*axes.get(&i)?, *axes.get(&j)?)))
            .map(|(i, j)| (i.min(j), i.max(j)))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        // The bounded I02 controls qualify the directional-gradient construction for
        // complete pair support. Sparse support retains pair construction with shared
        // firsts; no unmeasured density crossover or latency estimator is introduced.
        let directional_gradient = kind == ProductKind::WeightedHessianAction
            && pairs.len() == n.checked_mul(n.saturating_add(1))
                .ok_or(MathError::Limit("weighted pair population"))? / 2;
        let directions = if kind == ProductKind::WeightedHessianAction && !directional_gradient {
            let directions = (0..n).map(|i| library::formal(next_formal + body.outputs.len() + i))
                .collect::<Result<Vec<_>, _>>()?;
            parameters.extend(directions.iter().cloned());
            directions
        } else { Vec::new() };
        // Differentiate each participating first derivative once. Exact-entry callers
        // consume supported pairs; action callers differentiate the weighted gradient
        // along one library Taylor axis instead of constructing every second pair.
        let needed_first = if directional_gradient || kind == ProductKind::TransposeJacobianAction {
            (0..n).collect::<BTreeSet<_>>()
        } else {
            pairs.iter().map(|&(i, _)| i).collect()
        };
        let calls = needed_first.len().checked_add(if directional_gradient || kind == ProductKind::TransposeJacobianAction { 0 } else { pairs.len() })
            .ok_or(MathError::Limit("weighted symbolic derivative calls"))?;
        if calls > limits.operations {
            return Err(MathError::Limit("weighted symbolic derivative calls"));
        }
        let mut firsts = BTreeMap::new();
        for i in needed_first {
            scope.check().map_err(crate::error::scope_error)?;
            crate::construction::additional(weighted.as_view().get_byte_size()
                .checked_mul(4).ok_or(MathError::Limit("weighted derivative construction"))?)?;
            firsts.insert(i, weighted.derivative(
                Indeterminate::try_from(parameters[coordinates[i]].clone())
                    .map_err(MathError::Library)?,
            ));
        }
        let mut expressions = if directional_gradient || kind == ProductKind::TransposeJacobianAction {
            firsts.into_values().collect::<Vec<_>>()
        } else {
            let mut expressions = if kind == ProductKind::WeightedHessianAction { vec![Atom::num(0); n] } else { Vec::with_capacity(pairs.len()) };
            for &(i, j) in &pairs {
                scope.check().map_err(crate::error::scope_error)?;
                let first = firsts.get(&i).ok_or_else(|| MathError::Contract("weighted first partial missing".into()))?;
                crate::construction::additional(first.as_view().get_byte_size()
                    .checked_mul(4).ok_or(MathError::Limit("weighted derivative construction"))?)?;
                let partial = first.derivative(
                    Indeterminate::try_from(parameters[coordinates[j]].clone())
                        .map_err(MathError::Library)?,
                );
                if kind == ProductKind::WeightedHessianAction {
                    expressions[i] += &partial * &directions[j];
                    if i != j { expressions[j] += &partial * &directions[i]; }
                } else { expressions.push(partial); }
            }
            expressions
        };
        let product_len = if matches!(kind, ProductKind::WeightedHessianAction | ProductKind::TransposeJacobianAction) { n } else { pairs.len() };
        if expressions.is_empty() {
            expressions.push(Atom::num(0));
        }
        let mut scalars = retained
            .map(library::ScalarCache::from_products)
            .unwrap_or_default();
        let inspected = if directional_gradient {
            let layout = JetLayout::new(vec![0], DerivativeOrder::First, limits)?;
            // Runtime weights are constant along the action; only selected physical
            // coordinates receive directional seeds. Other inputs remain complete.
            let selected = coordinates.iter().copied().collect::<BTreeSet<_>>();
            let zeros = (0..parameters.len())
                .filter(|i| !selected.contains(i))
                .map(|i| (i, 1)).collect::<Vec<_>>();
            library::bounded_evaluator(SemanticId::NIL, &expressions, &parameters,
                &layout, &zeros, options, cancel, limits, limits.operations, 0, &mut scalars)?
        } else {
            library::bounded_scalar_evaluator(&expressions, &parameters, options,
                cancel, limits, &mut scalars)?
        };
        let storage = inspected.facts.storage;
        let operations = operation_count(inspected.facts.operations);
        let export = inspected.export;
        let evaluator = inspected.evaluator;
        let modules = options.evaluator.module_bytes();
        let evaluator = match options.evaluator {
            library::Evaluator::Interpreted => BlockEvaluator::Interpreted(evaluator),
            library::Evaluator::CompiledScalar { module_bytes } => {
                #[cfg(feature = "compiled-evaluator")]
                {
                    crate::construction::additional(module_bytes)?;
                    BlockEvaluator::Native(crate::native_compiled::compile(
                        &evaluator,
                        &storage,
                        &export,
                        module_bytes,
                        scope,
                    )?)
                }
                #[cfg(not(feature = "compiled-evaluator"))]
                {
                    let _ = (module_bytes, export);
                    return Err(MathError::Contract(
                        "compiled product requires compiled-evaluator feature".into(),
                    ));
                }
            }
        };
        drop(export);
        let numeric = storage
            .numeric_entries
            .checked_add(parameters.len().checked_mul(if directional_gradient { 2 } else { 1 })
                .ok_or(MathError::Limit("weighted product arguments"))?)
            .and_then(|n| n.checked_add(expressions.len() * if directional_gradient { 2 } else { 1 }))
            .and_then(|n| n.checked_add(product_len))
            .ok_or(MathError::Limit("weighted product scratch"))?;
        limits.allocation(numeric)?;
        let scratch_bytes = numeric
            .checked_mul(size_of::<f64>())
            .and_then(|n| n.checked_add(storage.instruction_bytes))
            .ok_or(MathError::Limit("weighted product scratch"))?;
        let retained_bytes = scratch_bytes
            .checked_add(modules)
            .and_then(|bytes| {
                bytes.checked_add(size_of::<DirectProduct>() + size_of::<Vec<(usize, usize)>>())
            })
            .and_then(|bytes| {
                pairs
                    .capacity()
                    .checked_mul(size_of::<(usize, usize)>())
                    .and_then(|pairs| bytes.checked_add(pairs))
            })
            .ok_or(MathError::Limit("weighted product retained storage"))?;
        let direct = DirectProduct {
            evaluator,
            order,
            pairs: Arc::new(pairs),
            inputs: body.inputs,
            outputs: body.outputs.len(),
            directional_gradient,
            arguments: parameters.len() * if directional_gradient { 2 } else { 1 },
            results: expressions.len() * if directional_gradient { 2 } else { 1 },
            scratch_bytes,
            retained_bytes,
        };
        let direct_scalars = scalars.freeze();
        let mut compiled = body.compile_scope(
            &selected,
            &coordinates,
            DerivativeOrder::Value,
            options,
            limits,
            scope,
            false,
            ProductKind::SelectedPartials,
            self,
            Some(&direct_scalars),
        )?;
        // Derivative-order obligations retain their original execution position and are
        // checked even though this semantic route carries only values through the stages.
        for stage in Arc::make_mut(&mut compiled.programs)
            .values_mut()
            .flatten()
            .flatten()
        {
            if let CompiledStage::Require { order, .. } = stage {
                *order = DerivativeOrder::Value;
            }
        }
        compiled.census[order] = Some(CompiledOrderCensus {
            order,
            directional: kind == ProductKind::WeightedHessianAction,
            product: kind,
            direct_weighted: true,
            stages: 1,
            arithmetic_blocks: 1,
            arithmetic_input_entries: direct.arguments,
            arithmetic_output_entries: direct.results,
            native_arithmetic_blocks: usize::from(modules > 0),
            provider_stages: 0,
            control_stages: 0,
            library_operations: operations,
            library_instructions: storage.instruction_count,
            instruction_storage_bytes: storage.instruction_bytes,
            frame_entries: 0,
            derivative_components: if directional_gradient { 2 } else { 1 },
            second_pairs: direct.pairs.len(),
            result_entries: product_len,
            worker_bytes: scratch_bytes,
        });
        compiled.evaluation_cache_bytes = compiled
            .evaluation_cache_bytes
            .checked_add(
                (product_len
                    + body.outputs.len()
                    + if kind == ProductKind::WeightedHessianAction {
                        n
                    } else {
                        0
                    })
                .checked_mul(2 * size_of::<f64>())
                .ok_or(MathError::Limit("weighted point cache"))?,
            )
            .ok_or(MathError::Limit("weighted point cache"))?;
        compiled.product = kind;
        compiled.direct_product = Some(Arc::new(direct));
        compiled.worker_bytes = compiled
            .worker_bytes
            .checked_add(scratch_bytes)
            .ok_or(MathError::Limit("weighted product worker storage"))?;
        compiled.scratch_bytes = compiled
            .scratch_bytes
            .checked_add(numeric * size_of::<f64>())
            .ok_or(MathError::Limit("weighted product numeric storage"))?;
        if compiled.scratch_bytes > limits.scratch_bytes {
            return Err(MathError::Limit("weighted product numeric storage"));
        }
        compiled.retained_bytes = compiled
            .retained_bytes
            .checked_add(retained_bytes)
            .ok_or(MathError::Limit("weighted product retained storage"))?;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.weighted_direct_constructions", "count", 1);
            context.count("math.weighted_symbolic_derivative_calls", "calls", calls as u64);
            context.integer(
                "math.weighted_direct_outputs",
                "components",
                product_len as i64,
            );
            context.integer(
                "math.weighted_direct_operations",
                "operations",
                operations as i64,
            );
        }
        scope.check().map_err(crate::error::scope_error)?;
        Ok(compiled)
    }
}
impl CompiledBody {
    /// Immutable mathematical product, independent of runtime coefficient values.
    pub fn product_kind(&self) -> ProductKind {
        self.product
    }
    /// Number of entries consumed by this product's actual caller.
    pub fn product_len(&self) -> usize {
        match self.product {
            ProductKind::SelectedPartials => 0,
            ProductKind::JacobianAction => self.outputs.len(),
            ProductKind::WeightedHessian => self.product_pairs().len(),
            ProductKind::WeightedHessianAction | ProductKind::TransposeJacobianAction => self.coordinate_offsets.len(),
        }
    }
    /// Ordered selected-axis symmetric pairs of a weighted matrix product.
    pub fn product_pairs(&self) -> &[(usize, usize)] {
        self.direct_product
            .as_ref()
            .map(|d| d.pairs.as_slice())
            .unwrap_or_else(|| {
                self.layouts[DerivativeOrder::Second]
                    .as_ref()
                    .map_or(&[], |l| l.pairs.as_slice())
            })
    }
}
impl Worker {
    /// Evaluate into private storage, then publish a complete finite requested product.
    pub fn evaluate_product_into(
        &mut self,
        inputs: &[f64],
        bindings: ProductValues<'_>,
        providers: &mut pse_kernels::ProviderBindings,
        cancel: &Arc<AtomicBool>,
        output: &mut [f64],
    ) -> Result<(), MathError> {
        let mut trial = TrialEvaluation::default();
        self.evaluate_product_trial_into(inputs, bindings, providers, cancel, output, &mut trial)
    }
    pub(crate) fn evaluate_product_trial_into(
        &mut self,
        inputs: &[f64],
        bindings: ProductValues<'_>,
        providers: &mut pse_kernels::ProviderBindings,
        cancel: &Arc<AtomicBool>,
        output: &mut [f64],
        trial: &mut TrialEvaluation,
    ) -> Result<(), MathError> {
        trial.clear();
        self.check_scope()?;
        let (weights, direction) = match bindings {
            ProductValues::SelectedPartials => {
                return Err(MathError::Contract(
                    "selected partials use ordered evaluation".into(),
                ));
            }
            ProductValues::JacobianAction { direction } => (None, Some(direction)),
            ProductValues::WeightedHessian { weights } | ProductValues::TransposeJacobianAction { weights } => (Some(weights), None),
            ProductValues::WeightedHessianAction { weights, direction } => {
                (Some(weights), Some(direction))
            }
        };
        if bindings.kind() != self.body.product
            || output.len() != self.body.product_len()
            || weights.is_some_and(|w| {
                w.len() != self.body.outputs.len() || w.iter().any(|v| !v.is_finite())
            })
            || direction.is_some_and(|d| {
                d.len() != self.body.coordinate_offsets.len() || d.iter().any(|v| !v.is_finite())
            })
        {
            return Err(MathError::Contract(
                "finite exact product binding extent/kind".into(),
            ));
        }
        let mut scratch = std::mem::take(&mut self.product_scratch);
        scratch.resize(output.len(), f64::NAN);
        scratch.fill(f64::NAN);
        let evaluated = (|| {
            if let Some(template) = self.body.transpose_product.clone() {
                if self.transpose_worker.is_none() {
                    self.transpose_worker = Some(Box::new(TransposeWorker {
                        worker: template.program.worker_scoped(self.scope.clone().ok_or_else(|| MathError::Contract("detached transpose controls".into()))?),
                        arguments: vec![f64::NAN; template.arguments_map.len()],
                        trial: TrialEvaluation::default(),
                    }));
                    self.used_bytes = self.used_bytes.checked_add(template.worker_bytes)
                        .ok_or(MathError::Limit("transpose worker charge"))?;
                    if self.used_bytes > self.prepaid_bytes { return Err(MathError::Limit("transpose worker charge")); }
                }
                let worker = self.transpose_worker.as_mut().ok_or_else(|| MathError::Contract("missing transpose worker".into()))?;
                let weights = weights.ok_or_else(|| MathError::Contract("transpose weights missing".into()))?;
                for (value, &formal) in worker.arguments.iter_mut().zip(&template.arguments_map) {
                    *value = if formal < template.inputs { inputs[formal] } else { weights[formal - template.inputs] };
                }
                worker.worker.set_scope(self.scope.clone().ok_or_else(|| MathError::Contract("detached transpose controls".into()))?);
                worker.worker.evaluate_trial_into(&worker.arguments, DerivativeOrder::First, providers, cancel, &mut worker.trial)?;
                for (axis, value) in scratch.iter_mut().enumerate() { *value = worker.trial.first(0, axis); }
            } else if let Some(template) = self.body.direct_product.clone() {
                self.evaluate_seeded(
                    inputs,
                    DerivativeOrder::Value,
                    None,
                    providers,
                    cancel,
                    trial,
                )?;
                if self.direct_product.is_none() {
                    let used = self
                        .used_bytes
                        .checked_add(template.scratch_bytes)
                        .ok_or(MathError::Limit("weighted product worker charge"))?;
                    if used > self.prepaid_bytes {
                        return Err(MathError::Limit("weighted product worker charge"));
                    }
                    let mut evaluator = template.evaluator.clone();
                    evaluator.instantiate()?;
                    self.direct_product = Some(DirectWorker {
                        evaluator,
                        arguments: vec![f64::NAN; template.arguments],
                        values: vec![f64::NAN; template.results],
                    });
                    self.used_bytes = used;
                }
                let worker = self
                    .direct_product
                    .as_mut()
                    .ok_or_else(|| MathError::Contract("weighted product worker missing".into()))?;
                let weights = weights.ok_or_else(|| MathError::Contract("weighted coefficients missing".into()))?;
                if template.directional_gradient {
                    for (i, &value) in inputs.iter().chain(weights).enumerate() {
                        worker.arguments[2 * i] = value;
                        worker.arguments[2 * i + 1] = 0.0;
                    }
                    let direction = direction.ok_or_else(|| MathError::Contract("direction missing".into()))?;
                    for (&coordinate, &seed) in self.body.coordinate_offsets.iter().zip(direction) {
                        worker.arguments[2 * coordinate + 1] = seed;
                    }
                } else {
                    worker.arguments[..template.inputs].copy_from_slice(inputs);
                    worker.arguments[template.inputs..template.inputs + template.outputs].copy_from_slice(weights);
                    if let Some(direction) = direction {
                        worker.arguments[template.inputs + template.outputs..].copy_from_slice(direction);
                    }
                }
                worker.values.fill(f64::NAN);
                worker
                    .evaluator
                    .evaluate(&worker.arguments, &mut worker.values)
                    .map_err(|detail| MathError::Evaluation {
                        source_id: SemanticId::NIL,
                        order: DerivativeOrder::Second,
                        detail,
                    })?;
                let count = scratch.len();
                if template.directional_gradient {
                    for (i, value) in scratch.iter_mut().enumerate() {
                        *value = worker.values[2 * i + 1];
                    }
                } else {
                    scratch.copy_from_slice(&worker.values[..count]);
                }
            } else {
                if self.body.product == ProductKind::JacobianAction {
                    self.evaluate_directional_trial_into(
                        inputs,
                        direction.ok_or_else(|| MathError::Contract("direction missing".into()))?,
                        providers,
                        cancel,
                        trial,
                    )?;
                    for (row, value) in scratch.iter_mut().enumerate() { *value = trial.first(row, 0); }
                } else {
                    self.evaluate_seeded(
                        inputs,
                        DerivativeOrder::Second,
                        direction,
                        providers,
                        cancel,
                        trial,
                    )?;
                    scratch.fill(0.0);
                    let weights =
                        weights.ok_or_else(|| MathError::Contract("weights missing".into()))?;
                    for (row, &weight) in weights.iter().enumerate() {
                        if self.body.product == ProductKind::WeightedHessianAction {
                            let axis = self.body.coordinate_offsets.len();
                            for (i, out) in scratch.iter_mut().enumerate() {
                                *out += weight * trial.second(row, i, axis);
                            }
                        } else {
                            for (out, &(i, j)) in scratch.iter_mut().zip(self.body.product_pairs())
                            {
                                *out += weight * trial.second(row, i, j);
                            }
                        }
                    }
                }
            }
            if scratch.iter().any(|v| !v.is_finite()) {
                return Err(MathError::Domain {
                    source_id: SemanticId::NIL,
                    requirement: "finite requested product",
                });
            }
            if cancel.load(Ordering::Acquire) {
                return Err(MathError::Cancelled);
            }
            self.check_scope()?;
            output.copy_from_slice(&scratch);
            Ok(())
        })();
        if evaluated.is_err() {
            scratch.fill(f64::NAN);
            trial.clear();
        }
        self.product_scratch = scratch;
        evaluated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn weighted_body() -> PreparedBody {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let y = library::formal(1).unwrap();
        PreparedBody::new(
            2,
            4,
            vec![2, 3],
            vec![Stage::Block {
                expressions: vec![x.pow(2) + y.pow(2), &x * &y],
                outputs: vec![2, 3],
                source: SemanticId::NIL,
            }],
            DerivativeOrder::Second,
        )
        .unwrap()
    }
    #[test]
    fn weighted_library_products_bind_runtime_coefficients_without_per_output_hessians() {
        let body = weighted_body();
        let cancel = pse_diagnostics::Cancellation::new(false);
        let support = body
            .prepare_support(&[0, 1], &[0, 1], DerivativeOrder::Second, &cancel)
            .unwrap();
        let matrix = support
            .compile_product(
                ProductKind::WeightedHessian,
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap();
        assert_eq!(matrix.product_pairs(), &[(0, 0), (0, 1), (1, 1)]);
        assert_eq!(matrix.product_len(), 3);
        assert_eq!(
            matrix.census[DerivativeOrder::Second]
                .unwrap()
                .result_entries,
            3
        );
        assert_eq!(
            matrix.census[DerivativeOrder::Second]
                .unwrap()
                .derivative_components,
            1
        );
        assert!(
            matrix.programs[DerivativeOrder::Second].is_none(),
            "no per-output second-jet program"
        );
        let mut worker = matrix.worker();
        let mut providers = BTreeMap::new();
        let mut output = [f64::NAN; 3];
        for (weights, expected) in [
            ([2., 3.], [4., 3., 4.]),
            ([-1., 0.5], [-2., 0.5, -2.]),
            ([2., 3.], [4., 3., 4.]),
        ] {
            worker
                .evaluate_product_into(
                    &[2., 3.],
                    ProductValues::WeightedHessian { weights: &weights },
                    &mut providers,
                    &cancel,
                    &mut output,
                )
                .unwrap();
            assert_eq!(output, expected);
        }
        output = [99.; 3];
        assert!(
            worker
                .evaluate_product_into(
                    &[1e200, 3.],
                    ProductValues::WeightedHessian { weights: &[0., 0.] },
                    &mut providers,
                    &cancel,
                    &mut output
                )
                .is_err()
        );
        assert_eq!(
            output, [99.; 3],
            "invalid original values cannot publish a valid weighted cancellation"
        );
        assert!(
            worker
                .evaluate_product_into(
                    &[1e200, 1e200],
                    ProductValues::WeightedHessian {
                        weights: &[1., -2.]
                    },
                    &mut providers,
                    &cancel,
                    &mut output
                )
                .is_err()
        );
        assert_eq!(
            output, [99.; 3],
            "finite Hessian of (x-y)^2 does not erase overflowing original values"
        );
        let action = support
            .compile_product(
                ProductKind::WeightedHessianAction,
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap();
        assert_eq!(action.product_len(), 2);
        assert_eq!(
            action.census[DerivativeOrder::Second]
                .unwrap()
                .result_entries,
            2
        );
        let mut worker = action.worker();
        let mut action_output = [f64::NAN; 2];
        worker
            .evaluate_product_into(
                &[2., 3.],
                ProductValues::WeightedHessianAction {
                    weights: &[2., 3.],
                    direction: &[5., 7.],
                },
                &mut providers,
                &cancel,
                &mut action_output,
            )
            .unwrap();
        assert_eq!(action_output, [41., 43.]);
        worker
            .evaluate_product_into(
                &[2., 3.],
                ProductValues::WeightedHessianAction {
                    weights: &[-1., 0.5],
                    direction: &[7., 5.],
                },
                &mut providers,
                &cancel,
                &mut action_output,
            )
            .unwrap();
        assert_eq!(action_output, [-11.5, -6.5]);
        assert!(
            worker
                .evaluate_product_into(
                    &[2., 3.],
                    ProductValues::WeightedHessian { weights: &[2., 3.] },
                    &mut providers,
                    &cancel,
                    &mut action_output
                )
                .is_err()
        );
    }
    #[test]
    fn transpose_actions_keep_output_weights_coordinate_order_and_host_branches() {
        crate::initialize().unwrap();
        let cancel = pse_diagnostics::Cancellation::new(false);
        let scope = pse_kernels::ExecutionScope::new(cancel.clone(), None);
        let body = weighted_body();
        let support = body.prepare_support(&[0, 1], &[1, 0], DerivativeOrder::First, &cancel).unwrap();
        let compiled = support.compile_product(ProductKind::TransposeJacobianAction,
            Optimization::default(), EvaluationLimits::default(), &scope).unwrap();
        assert_eq!(compiled.compiled_order(), DerivativeOrder::First);
        assert_eq!(compiled.product_len(), 2);
        let mut worker = compiled.worker_scoped(scope.clone());
        let mut result = [f64::NAN; 2];
        for (weights, expected) in [([2., 3.], [18., 17.]), ([0., -1.], [-2., -3.])] {
            worker.evaluate_product_into(&[2., 3.], ProductValues::TransposeJacobianAction { weights: &weights },
                &mut pse_kernels::ProviderBindings::default(), &cancel, &mut result).unwrap();
            assert_eq!(result, expected);
        }
        let x = library::formal(0).unwrap();
        let y = library::formal(1).unwrap();
        let branch = PreparedBody::new(3, 5, vec![4], vec![
            Stage::Block { expressions: vec![Atom::num(0)], outputs: vec![3], source: SemanticId::NIL },
            Stage::Branch { continuity: DerivativeOrder::Value, comparison: Comparison::Lt, left: 3, right: 2,
                then: vec![Stage::Block { expressions: vec![(&x * &y).pow(2)], outputs: vec![4], source: SemanticId::NIL }],
                otherwise: vec![Stage::Block { expressions: vec![x.pow(3) + y.pow(2)], outputs: vec![4], source: SemanticId::NIL }],
            }], DerivativeOrder::Second).unwrap();
        let support = branch.prepare_support(&[0], &[1, 0], DerivativeOrder::First, &cancel).unwrap();
        let compiled = support.compile_product(ProductKind::TransposeJacobianAction,
            Optimization::default(), EvaluationLimits::default(), &scope).unwrap();
        assert!(compiled.transpose_product.is_some());
        let mut worker = compiled.worker_scoped(scope);
        for (guard, expected) in [(1., [48., 72.]), (-1., [12., 24.]), (1., [48., 72.])] {
            worker.evaluate_product_into(&[2., 3., guard], ProductValues::TransposeJacobianAction { weights: &[2.] },
                &mut pse_kernels::ProviderBindings::default(), &cancel, &mut result).unwrap();
            assert_eq!(result, expected);
        }
        let saved = result;
        assert!(worker.evaluate_product_into(&[1e200, 3., 1.],
            ProductValues::TransposeJacobianAction { weights: &[0.] }, &mut pse_kernels::ProviderBindings::default(), &cancel, &mut result).is_err());
        assert_eq!(result, saved);
        cancel.cancel();
        assert!(matches!(worker.evaluate_product_into(&[2., 3., 1.],
            ProductValues::TransposeJacobianAction { weights: &[2.] }, &mut pse_kernels::ProviderBindings::default(), &cancel, &mut result), Err(MathError::Cancelled)));
        assert_eq!(result, saved);
    }

    #[test]
    fn flat_action_construction_comparison_keeps_directional_work_and_exact_entries_distinct() {
        crate::initialize().unwrap();
        let n = 6;
        let x = (0..n).map(|i| library::formal(i).unwrap()).collect::<Vec<_>>();
        let sum = x.iter().cloned().fold(Atom::num(0), |a, b| a + b);
        let chain = (0..n - 1).fold(Atom::num(0), |a, i| a + x[i].pow(2) * &x[i + 1]);
        let shared = (&sum * &sum + &x[0]).pow(2);
        let cancel = pse_diagnostics::Cancellation::new(false);
        let scope = pse_kernels::ExecutionScope::new(cancel.clone(), None);
        let limits = EvaluationLimits::default();
        let point = (0..n).map(|i| 0.2 + i as f64 * 0.1).collect::<Vec<_>>();
        let direction = (0..n).map(|i| if i % 2 == 0 { 0.5 } else { -0.3 }).collect::<Vec<_>>();
        for (name, expression) in [("dense", sum.pow(3)), ("sparse", chain), ("shared", shared)] {
            let body = PreparedBody::new(n, n + 1, vec![n], vec![Stage::Block {
                expressions: vec![expression.clone()], outputs: vec![n], source: SemanticId::NIL,
            }], DerivativeOrder::Second).unwrap();
            let axes = (0..n).collect::<Vec<_>>();
            let support = body.prepare_support(&[0], &axes, DerivativeOrder::Second, &cancel).unwrap();
            let start = std::time::Instant::now();
            let matrix = support.compile_product(ProductKind::WeightedHessian,
                Optimization::default(), limits, &scope).unwrap();
            let matrix_prepare = start.elapsed();
            let start = std::time::Instant::now();
            let action = support.compile_product(ProductKind::WeightedHessianAction,
                Optimization::default(), limits, &scope).unwrap();
            let action_prepare = start.elapsed();
            assert_eq!(action.direct_product.as_ref().unwrap().directional_gradient, name != "sparse");
            assert_eq!(action.census[DerivativeOrder::Second].unwrap().derivative_components,
                if name == "sparse" { 1 } else { 2 });
            let mut matrix_values = vec![f64::NAN; matrix.product_len()];
            matrix.worker_scoped(scope.clone()).evaluate_product_into(&point,
                ProductValues::WeightedHessian { weights: &[1.] }, &mut pse_kernels::ProviderBindings::default(),
                &cancel, &mut matrix_values).unwrap();
            let mut expected = vec![0.; n];
            for (&(i, j), &value) in matrix.product_pairs().iter().zip(&matrix_values) {
                expected[i] += value * direction[j];
                if i != j { expected[j] += value * direction[i]; }
            }
            // Independent contracted symbolic candidate: gradient of directional scalar.
            let seeds = (n..2*n).map(|i| library::formal(i).unwrap()).collect::<Vec<_>>();
            let start = std::time::Instant::now();
            let firsts = x.iter().map(|atom| expression.derivative(
                Indeterminate::try_from(atom.clone()).unwrap())).collect::<Vec<_>>();
            let contracted = firsts.iter().zip(&seeds).fold(Atom::num(0), |a, (g, v)| a + g * v);
            let scalar_gradient = x.iter().map(|atom| contracted.derivative(
                Indeterminate::try_from(atom.clone()).unwrap())).collect::<Vec<_>>();
            let parameters = x.iter().chain(&seeds).cloned().collect::<Vec<_>>();
            let mut alternative = library::bounded_scalar_evaluator(&scalar_gradient,
                &parameters, Optimization::default(), &cancel, limits,
                &mut library::ScalarCache::default()).unwrap();
            let alternative_prepare = start.elapsed();
            let mut alternative_values = vec![f64::NAN; n];
            alternative.evaluator.try_evaluate(&point.iter().chain(&direction).copied().collect::<Vec<_>>(),
                &mut alternative_values).unwrap();
            let mut worker = action.worker_scoped(scope.clone());
            let mut actual = vec![f64::NAN; n];
            let start = std::time::Instant::now();
            for _ in 0..16 {
                worker.evaluate_product_into(&point, ProductValues::WeightedHessianAction {
                    weights: &[1.], direction: &direction }, &mut pse_kernels::ProviderBindings::default(), &cancel,
                    &mut actual).unwrap();
            }
            let execute = start.elapsed();
            for ((&actual, &alternative), &expected) in actual.iter().zip(&alternative_values).zip(&expected) {
                assert!((actual - expected).abs() <= 1e-10 * expected.abs().max(1.), "{name}: {actual} != {expected}");
                assert!((alternative - expected).abs() <= 1e-10 * expected.abs().max(1.));
            }
            eprintln!("I02 {name}: pairs={} matrix_prepare={matrix_prepare:?} directional_prepare={action_prepare:?} scalar_gradient_prepare={alternative_prepare:?} directional_16_calls={execute:?} action_ops={} alternative_ops={} action_scratch={}",
                matrix.product_len(), action.census[DerivativeOrder::Second].unwrap().library_operations,
                operation_count(alternative.facts.operations), action.scratch_bytes());
        }
    }

    #[cfg(feature = "compiled-evaluator")]
    #[test]
    fn compiled_weighted_products_execute_derivatives_and_retain_private_workers() {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let y = library::formal(1).unwrap();
        let body = PreparedBody::new(
            2,
            4,
            vec![2, 3],
            vec![Stage::Block {
                expressions: vec![x.pow(2) + Atom::num(3) * y.pow(2), &x * &y],
                outputs: vec![2, 3],
                source: SemanticId::NIL,
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let cancel = pse_diagnostics::Cancellation::new(false);
        let support = body
            .prepare_support(&[0, 1], &[1, 0], DerivativeOrder::Second, &cancel)
            .unwrap();
        let options = Optimization {
            evaluator: library::Evaluator::CompiledScalar {
                module_bytes: 1 << 20,
            },
            ..Default::default()
        };
        for kind in [
            ProductKind::WeightedHessian,
            ProductKind::WeightedHessianAction,
        ] {
            let compiled = support
                .compile_product(
                    kind,
                    options,
                    EvaluationLimits::default(),
                    &pse_kernels::ExecutionScope::new(cancel.clone(), None),
                )
                .unwrap();
            let census = compiled
                .census()
                .find(|entry| entry.order == DerivativeOrder::Second)
                .unwrap();
            assert!(census.direct_weighted);
            assert_eq!(census.native_arithmetic_blocks, 1);
            assert!(matches!(
                compiled.direct_product.as_ref().unwrap().evaluator,
                BlockEvaluator::Native(_)
            ));
            assert!(compiled.programs[DerivativeOrder::Second].is_none());
            let mut first = compiled.worker();
            let weights = [2., 3.];
            let direction = [5., 7.];
            let binding = if kind == ProductKind::WeightedHessian {
                ProductValues::WeightedHessian { weights: &weights }
            } else {
                ProductValues::WeightedHessianAction {
                    weights: &weights,
                    direction: &direction,
                }
            };
            let expected = if kind == ProductKind::WeightedHessian {
                vec![12., 3., 4.]
            } else {
                vec![81., 43.]
            };
            let mut output = vec![f64::NAN; compiled.product_len()];
            first
                .evaluate_product_into(
                    &[2., 3.],
                    binding,
                    &mut pse_kernels::ProviderBindings::new(),
                    &cancel,
                    &mut output,
                )
                .unwrap();
            assert_eq!(output, expected);
            let mut independent = first.clone();
            first.detach_controls();
            first.reset_scratch().unwrap();
            drop(first);
            drop(compiled);
            let changed_weights = [-1., 0.5];
            let changed_direction = [7., 5.];
            let changed = if kind == ProductKind::WeightedHessian {
                ProductValues::WeightedHessian {
                    weights: &changed_weights,
                }
            } else {
                ProductValues::WeightedHessianAction {
                    weights: &changed_weights,
                    direction: &changed_direction,
                }
            };
            independent
                .evaluate_product_into(
                    &[4., 2.],
                    changed,
                    &mut pse_kernels::ProviderBindings::new(),
                    &cancel,
                    &mut output,
                )
                .unwrap();
            assert_eq!(
                output,
                if kind == ProductKind::WeightedHessian {
                    vec![-6., 0.5, -2.]
                } else {
                    vec![-39.5, -6.5]
                }
            );
            let completed = output.clone();
            assert!(
                independent
                    .evaluate_product_into(
                        &[1e200, 1e200],
                        binding,
                        &mut pse_kernels::ProviderBindings::new(),
                        &cancel,
                        &mut output
                    )
                    .is_err()
            );
            assert_eq!(output, completed);
            cancel.cancel();
            assert!(matches!(
                independent.evaluate_product_into(
                    &[2., 3.],
                    binding,
                    &mut pse_kernels::ProviderBindings::new(),
                    &cancel,
                    &mut output
                ),
                Err(MathError::Cancelled)
            ));
            assert_eq!(output, completed);
            cancel.store(false, Ordering::Release);
        }
    }
    #[test]
    fn weighted_products_retain_derivative_only_original_guard_and_cancel() {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let source = pse_ids::named_id(SemanticId::NIL, "weighted original guard");
        let body = PreparedBody::new(
            1,
            2,
            vec![1],
            vec![
                Stage::Require {
                    argument: 0,
                    condition: Condition::Nonzero,
                    order: DerivativeOrder::Second,
                    source,
                    lineage: None,
                },
                Stage::Block {
                    expressions: vec![x.pow(2)],
                    outputs: vec![1],
                    source,
                },
            ],
            DerivativeOrder::Second,
        )
        .unwrap();
        let cancel = pse_diagnostics::Cancellation::new(false);
        let support = body
            .prepare_support(&[0], &[0], DerivativeOrder::Second, &cancel)
            .unwrap();
        let compiled = support
            .compile_product(
                ProductKind::WeightedHessian,
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap();
        let mut worker = compiled.worker();
        let mut out = [93.];
        assert!(
            matches!(worker.evaluate_product_into(&[0.],ProductValues::WeightedHessian{weights:&[1.]},&mut pse_kernels::ProviderBindings::new(),&cancel,&mut out),Err(MathError::Domain{source_id,..}) if source_id==source)
        );
        assert_eq!(out, [93.]);
        cancel.cancel();
        assert!(matches!(
            worker.evaluate_product_into(
                &[2.],
                ProductValues::WeightedHessian { weights: &[1.] },
                &mut pse_kernels::ProviderBindings::new(),
                &cancel,
                &mut out
            ),
            Err(MathError::Cancelled)
        ));
        assert_eq!(out, [93.]);
    }
    #[test]
    fn branch_weighted_actions_keep_host_control_and_selected_coordinate_chain() {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let y = library::formal(1).unwrap();
        let source = pse_ids::named_id(SemanticId::NIL, "weighted branch");
        let body = PreparedBody::new(
            3,
            5,
            vec![4],
            vec![
                Stage::Block {
                    expressions: vec![Atom::num(0)],
                    outputs: vec![3],
                    source,
                },
                Stage::Branch {
                    continuity: DerivativeOrder::Value,
                    comparison: Comparison::Lt,
                    left: 3,
                    right: 2,
                    then: vec![Stage::Block {
                        expressions: vec![(&x * &y).pow(2)],
                        outputs: vec![4],
                        source,
                    }],
                    otherwise: vec![Stage::Block {
                        expressions: vec![x.pow(3) + y.pow(2)],
                        outputs: vec![4],
                        source,
                    }],
                },
            ],
            DerivativeOrder::Second,
        )
        .unwrap();
        let cancel = pse_diagnostics::Cancellation::new(false);
        let support = body
            .prepare_support(&[0], &[1, 0], DerivativeOrder::Second, &cancel)
            .unwrap();
        let compiled = support
            .compile_product(
                ProductKind::WeightedHessianAction,
                Optimization::default(),
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap();
        let census = compiled
            .census()
            .find(|c| c.order == DerivativeOrder::Second)
            .unwrap();
        assert!(
            !census.direct_weighted,
            "a control barrier is not flattened into one weighted expression"
        );
        assert_eq!(census.second_pairs, 2);
        assert_eq!(census.control_stages, 1);
        let mut worker = compiled.worker();
        let mut output = [f64::NAN; 2];
        let binding = ProductValues::WeightedHessianAction {
            weights: &[2.],
            direction: &[5., 7.],
        };
        worker
            .evaluate_product_into(
                &[2., 3., 1.],
                binding,
                &mut pse_kernels::ProviderBindings::new(),
                &cancel,
                &mut output,
            )
            .unwrap();
        assert_eq!(output, [416., 492.]);
        worker
            .evaluate_product_into(
                &[2., 3., -1.],
                binding,
                &mut pse_kernels::ProviderBindings::new(),
                &cancel,
                &mut output,
            )
            .unwrap();
        assert_eq!(output, [20., 168.]);
    }
    #[cfg(feature = "compiled-evaluator")]
    #[test]
    fn compiled_first_only_product_does_not_construct_unused_second_programs() {
        let body = weighted_body();
        let cancel = pse_diagnostics::Cancellation::new(false);
        let support = body
            .prepare_support(&[0, 1], &[1, 0], DerivativeOrder::First, &cancel)
            .unwrap();
        let options = Optimization {
            evaluator: library::Evaluator::CompiledScalar {
                module_bytes: 1 << 20,
            },
            ..Default::default()
        };
        let compiled = support
            .compile_product(
                ProductKind::SelectedPartials,
                options,
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap();
        assert_eq!(compiled.compiled_order(), DerivativeOrder::First);
        assert!(compiled.programs[DerivativeOrder::Second].is_none());
        assert_eq!(
            compiled
                .census()
                .map(|order| order.native_arithmetic_blocks)
                .sum::<usize>(),
            2
        );
        let mut worker = compiled.worker();
        let result = worker
            .evaluate(
                &[2., 3.],
                DerivativeOrder::First,
                &mut pse_kernels::ProviderBindings::new(),
                &cancel,
            )
            .unwrap();
        assert_eq!(result.values, [13., 6.]);
        assert_eq!(result.jacobian, [6., 4., 2., 3.]);
        assert!(result.hessians.is_empty());
        assert!(!worker.census().instantiated_orders[2]);
        assert!(matches!(
            worker.evaluate(
                &[2., 3.],
                DerivativeOrder::Second,
                &mut pse_kernels::ProviderBindings::new(),
                &cancel
            ),
            Err(MathError::Contract(_))
        ));
        assert!(!worker.census().instantiated_orders[2]);
    }
}

#[cfg(test)]
#[path = "product_contract_tests.rs"]
mod contract_tests;
