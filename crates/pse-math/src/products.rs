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
    #[cfg(feature = "compiled-evaluator")]
    resettable: bool,
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
impl DirectProduct {
    /// Shared scalar/Taylor parameter layout; runtime weights remain ordinary
    /// inputs and never become per-lane compiled constants.
    pub(super) fn bind_arguments(
        &self,
        inputs: &[f64],
        weights: &[f64],
        direction: Option<&[f64]>,
        coordinates: &[usize],
        arguments: &mut [f64],
    ) -> Result<(), MathError> {
        if inputs.len() != self.inputs
            || weights.len() != self.outputs
            || arguments.len() != self.arguments
        {
            return Err(MathError::Contract(
                "direct product parameter extent".into(),
            ));
        }
        if self.directional_gradient {
            let direction =
                direction.ok_or_else(|| MathError::Contract("direction missing".into()))?;
            if direction.len() != coordinates.len() {
                return Err(MathError::Contract(
                    "direct product direction extent".into(),
                ));
            }
            for (index, &value) in inputs.iter().chain(weights).enumerate() {
                arguments[2 * index] = value;
                arguments[2 * index + 1] = 0.;
            }
            for (&coordinate, &seed) in coordinates.iter().zip(direction) {
                arguments[2 * coordinate + 1] = seed;
            }
        } else {
            arguments[..self.inputs].copy_from_slice(inputs);
            arguments[self.inputs..self.inputs + self.outputs].copy_from_slice(weights);
            if let Some(direction) = direction {
                arguments[self.inputs + self.outputs..].copy_from_slice(direction);
            }
        }
        Ok(())
    }
    /// Project only the requested direct entries. A directional weighted
    /// gradient reads its tangent, never an unrequested dense Second product.
    pub(super) fn write_product(
        &self,
        values: &[f64],
        output: &mut [f64],
    ) -> Result<(), MathError> {
        if values.len() != self.results
            || output
                .len()
                .checked_mul(if self.directional_gradient { 2 } else { 1 })
                .is_none_or(|extent| extent > values.len())
        {
            return Err(MathError::Contract("direct product result extent".into()));
        }
        if self.directional_gradient {
            for (index, value) in output.iter_mut().enumerate() {
                *value = values[2 * index + 1];
            }
        } else {
            output.copy_from_slice(&values[..output.len()]);
        }
        Ok(())
    }
    #[cfg(feature = "compiled-evaluator")]
    pub(super) fn packed_shape(&self) -> Option<(usize, usize, usize)> {
        match &self.evaluator {
            BlockEvaluator::Native(block) if self.resettable && block.executable.is_packed() => {
                Some((self.arguments, self.results, self.scratch_bytes))
            }
            _ => None,
        }
    }
}
impl DirectWorker {
    #[cfg(feature = "compiled-evaluator")]
    pub(super) fn evaluate_batch(
        &mut self,
        rows: usize,
        arguments: &[f64],
        values: &mut [f64],
        scope: &pse_kernels::ExecutionScope,
        allowance: usize,
    ) -> Result<(), MathError> {
        match &mut self.evaluator {
            BlockEvaluator::Native(block) => {
                let packed = block.executable.is_packed();
                block
                    .executable
                    .evaluate_batch(rows, arguments, values, scope, allowance)?;
                if packed && let Some(context) = pse_diagnostics::observation::current() {
                    context.count(
                        "math.native_x4_direct_action_rows",
                        "count",
                        (rows / 4 * 4) as u64,
                    );
                }
                Ok(())
            }
            _ => Err(MathError::Contract(
                "direct packed product requires a qualified native evaluator".into(),
            )),
        }
    }
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
) -> Result<Vec<Atom>, MathError> {
    if !body.is_flat_arithmetic() {
        return Err(MathError::Contract(
            "direct product preflight/source mismatch".into(),
        ));
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
                        return Err(MathError::Limit("direct product flattening bytes"));
                    };
                    crate::construction::additional(bound)?;
                    value = value
                        .replace(parameters[input].clone())
                        .with(replacement.clone());
                    work = work
                        .checked_add(operation_count(value.count_operations()))
                        .ok_or(MathError::Limit("weighted flattening operations"))?;
                    if work > limits.operations {
                        return Err(MathError::Limit("direct product flattening operations"));
                    }
                }
            }
            expressions.insert(output, value);
        }
    }
    Ok(body
        .outputs
        .iter()
        .map(|slot| {
            expressions
                .get(slot)
                .cloned()
                .unwrap_or_else(|| parameters[*slot].clone())
        })
        .collect())
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
        // The immutable source/profile decision precedes substitutions, derivatives,
        // native optimization and transient construction admission. Errors after this
        // point are terminal for this selected arithmetic implementation.
        let preflight = self.preflight_product(kind, options, limits)?;
        let support = preflight.support();
        let kind = preflight.kind;
        let options = preflight.options;
        let limits = preflight.limits;
        let required = support.order();
        let outputs = (0..support.outputs().len()).collect::<Vec<_>>();
        let coordinates = support.dense_coordinates()?;
        match preflight.route() {
            ProductRoute::Direct => {
                let expressions = flat_expressions(support.body(), limits, cancel)?;
                return support.compile_direct_product(
                    kind,
                    expressions,
                    options,
                    limits,
                    scope,
                    retained,
                );
            }
            ProductRoute::Reverse => {
                return support.compile_reverse_product(kind, options, limits, scope, retained);
            }
            ProductRoute::Regional => {}
        }
        support.body().compile_scope(
            &outputs,
            &coordinates,
            required,
            options,
            limits,
            scope,
            false,
            kind,
            support,
            retained,
            RegionStrategy::Forward,
        )
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
        let body = self.body();
        let selected = (0..self.outputs().len()).collect::<Vec<_>>();
        let coordinates = self.dense_coordinates()?;
        let available = body.selected_order(&selected, &coordinates, false)?;
        let order = if kind == ProductKind::TransposeJacobianAction {
            DerivativeOrder::First
        } else {
            DerivativeOrder::Second
        };
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
        let directional_gradient = kind == ProductKind::WeightedHessianAction;
        // Exact-entry consumers retain canonical pairs. An action consumes a
        // directional weighted gradient, independent of support density, and
        // does not construct an action-only population of second entries.
        let pairs = if kind == ProductKind::WeightedHessian {
            let axes = self
                .coordinates()
                .iter()
                .enumerate()
                .map(|(axis, &formal)| (formal, axis))
                .collect::<BTreeMap<_, _>>();
            self.support()
                .second
                .iter()
                .flatten()
                .filter_map(|&(i, j)| Some((*axes.get(&i)?, *axes.get(&j)?)))
                .map(|(i, j)| (i.min(j), i.max(j)))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        // Differentiate each participating first derivative once. Exact-entry callers
        // consume supported pairs; action callers differentiate the weighted gradient
        // along one library Taylor axis instead of constructing every second pair.
        let needed_first = if directional_gradient || kind == ProductKind::TransposeJacobianAction {
            (0..n).collect::<BTreeSet<_>>()
        } else {
            pairs.iter().map(|&(i, _)| i).collect()
        };
        let calls = needed_first
            .len()
            .checked_add(
                if directional_gradient || kind == ProductKind::TransposeJacobianAction {
                    0
                } else {
                    pairs.len()
                },
            )
            .ok_or(MathError::Limit("weighted symbolic derivative calls"))?;
        if calls > limits.operations {
            return Err(MathError::Limit("weighted symbolic derivative calls"));
        }
        let mut firsts = BTreeMap::new();
        for i in needed_first {
            scope.check().map_err(crate::error::scope_error)?;
            crate::construction::additional(
                weighted
                    .as_view()
                    .get_byte_size()
                    .checked_mul(4)
                    .ok_or(MathError::Limit("weighted derivative construction"))?,
            )?;
            firsts.insert(
                i,
                weighted.derivative(
                    Indeterminate::try_from(parameters[coordinates[i]].clone())
                        .map_err(MathError::Library)?,
                ),
            );
        }
        let mut expressions =
            if directional_gradient || kind == ProductKind::TransposeJacobianAction {
                firsts.into_values().collect::<Vec<_>>()
            } else {
                let mut expressions = Vec::with_capacity(pairs.len());
                for &(i, j) in &pairs {
                    scope.check().map_err(crate::error::scope_error)?;
                    let first = firsts.get(&i).ok_or_else(|| {
                        MathError::Contract("weighted first partial missing".into())
                    })?;
                    crate::construction::additional(
                        first
                            .as_view()
                            .get_byte_size()
                            .checked_mul(4)
                            .ok_or(MathError::Limit("weighted derivative construction"))?,
                    )?;
                    let partial = first.derivative(
                        Indeterminate::try_from(parameters[coordinates[j]].clone())
                            .map_err(MathError::Library)?,
                    );
                    expressions.push(partial);
                }
                expressions
            };
        let product_len = if matches!(
            kind,
            ProductKind::WeightedHessianAction | ProductKind::TransposeJacobianAction
        ) {
            n
        } else {
            pairs.len()
        };
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
                .map(|i| (i, 1))
                .collect::<Vec<_>>();
            library::bounded_evaluator(
                SemanticId::NIL,
                &expressions,
                &parameters,
                &layout,
                &zeros,
                options,
                scope,
                limits,
                limits.operations,
                0,
                &mut scalars,
            )?
        } else {
            library::bounded_scalar_evaluator(
                &expressions,
                &parameters,
                options,
                scope,
                limits,
                &mut scalars,
            )?
        };
        let storage = inspected.facts.storage;
        #[cfg(feature = "compiled-evaluator")]
        let mut storage = storage;
        let operations = operation_count(inspected.facts.operations);
        let export = inspected.export;
        let evaluator = inspected.evaluator;
        let modules = options.evaluator.module_bytes();
        let evaluator = match options.evaluator {
            library::Evaluator::Interpreted => BlockEvaluator::Interpreted(evaluator),
            library::Evaluator::CompiledScalar { module_bytes }
            | library::Evaluator::CompiledX4 { module_bytes } => {
                #[cfg(feature = "compiled-evaluator")]
                {
                    crate::construction::additional(module_bytes)?;
                    let executable = crate::native_compiled::compile_profile(
                        &evaluator,
                        &storage,
                        options.evaluator,
                        scope,
                    )?;
                    let shell = size_of::<ExpressionEvaluator<f64>>() + 2 * size_of::<usize>();
                    crate::construction::additional(shell)?;
                    storage.auxiliary_bytes = storage
                        .auxiliary_bytes
                        .checked_add(shell)
                        .ok_or(MathError::Limit("native product canonical owner"))?;
                    BlockEvaluator::Native(NativeBlock {
                        canonical: Arc::new(evaluator),
                        executable,
                    })
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
            .checked_mul(options.evaluator.numeric_stack_factor())
            .and_then(|n| {
                n.checked_add(parameters.len().checked_mul(if directional_gradient {
                    2
                } else {
                    1
                })?)
            })
            .and_then(|n| {
                n.checked_add(expressions.len() * if directional_gradient { 2 } else { 1 })
            })
            .and_then(|n| n.checked_add(product_len))
            .ok_or(MathError::Limit("weighted product scratch"))?;
        limits.allocation(numeric)?;
        let scratch_bytes = numeric
            .checked_mul(size_of::<f64>())
            .and_then(|n| n.checked_add(storage.auxiliary_bytes))
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
            #[cfg(feature = "compiled-evaluator")]
            resettable: inspected.facts.resettable,
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
            RegionStrategy::Forward,
        )?;
        let template = Arc::make_mut(&mut compiled.template);
        // Derivative-order obligations retain their original execution position and are
        // checked even though this semantic route carries only values through the stages.
        for stage in Arc::make_mut(&mut template.programs)
            .values_mut()
            .flatten()
            .flatten()
        {
            if let CompiledStage::Require { order: minimum, .. } = stage
                && *minimum <= order
            {
                *minimum = DerivativeOrder::Value;
            }
        }
        template.census[order] = Some(CompiledOrderCensus {
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
            instruction_storage_bytes: storage.auxiliary_bytes,
            frame_entries: 0,
            derivative_components: if directional_gradient { 2 } else { 1 },
            second_pairs: direct.pairs.len(),
            result_entries: product_len,
            worker_bytes: scratch_bytes,
        });
        template.evaluation_cache_bytes = template
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
        template.product = kind;
        template.direct_product = Some(Arc::new(direct));
        template.worker_bytes = template
            .worker_bytes
            .checked_add(scratch_bytes)
            .ok_or(MathError::Limit("weighted product worker storage"))?;
        template.scratch_bytes = template
            .scratch_bytes
            .checked_add(numeric * size_of::<f64>())
            .ok_or(MathError::Limit("weighted product numeric storage"))?;
        if template.scratch_bytes > limits.scratch_bytes {
            return Err(MathError::Limit("weighted product numeric storage"));
        }
        compiled.retained_bytes = compiled
            .retained_bytes
            .checked_add(retained_bytes)
            .ok_or(MathError::Limit("weighted product retained storage"))?;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.weighted_direct_constructions", "count", 1);
            context.count(
                "math.weighted_symbolic_derivative_calls",
                "calls",
                calls as u64,
            );
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
        self.template.product_kind()
    }
    /// Number of entries consumed by this product's actual caller.
    pub fn product_len(&self) -> usize {
        self.template.product_len()
    }
    /// Ordered selected-axis symmetric pairs of a weighted matrix product.
    pub fn product_pairs(&self) -> &[(usize, usize)] {
        self.template.product_pairs()
    }
}
impl ExecutionTemplate {
    /// Immutable mathematical product, independent of runtime coefficient values.
    pub(crate) fn product_kind(&self) -> ProductKind {
        self.product
    }
    /// Number of entries consumed by this product's actual caller.
    pub(crate) fn product_len(&self) -> usize {
        match self.product {
            ProductKind::SelectedPartials => 0,
            ProductKind::JacobianAction => self.outputs.len(),
            ProductKind::WeightedHessian => self.product_pairs().len(),
            ProductKind::WeightedHessianAction | ProductKind::TransposeJacobianAction => {
                self.coordinate_offsets.len()
            }
        }
    }
    /// Ordered selected-axis symmetric pairs of a weighted matrix product.
    pub(crate) fn product_pairs(&self) -> &[(usize, usize)] {
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
impl NumericFrame {
    /// Materialize the existing direct product exactly once in private scratch.
    pub(super) fn prepare_direct_worker(
        &mut self,
        template: &DirectProduct,
    ) -> Result<&mut DirectWorker, MathError> {
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
        self.direct_product
            .as_mut()
            .ok_or_else(|| MathError::Contract("weighted product worker missing".into()))
    }
    /// Evaluate into private storage, then publish a complete finite requested product.
    pub(crate) fn evaluate_product_into(
        &mut self,
        inputs: &[f64],
        bindings: ProductValues<'_>,
        providers: &mut ProviderBindings,
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
        providers: &mut ProviderBindings,
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
            ProductValues::WeightedHessian { weights }
            | ProductValues::TransposeJacobianAction { weights } => (Some(weights), None),
            ProductValues::WeightedHessianAction { weights, direction } => {
                (Some(weights), Some(direction))
            }
        };
        if bindings.kind() != self.template.product
            || output.len() != self.template.product_len()
            || weights.is_some_and(|w| {
                w.len() != self.template.outputs.len() || w.iter().any(|v| !v.is_finite())
            })
            || direction.is_some_and(|d| {
                d.len() != self.template.coordinate_offsets.len()
                    || d.iter().any(|v| !v.is_finite())
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
            if self.template.reverse_product.is_some() {
                self.evaluate_reverse_product(
                    inputs,
                    weights.ok_or_else(|| MathError::Contract("weights missing".into()))?,
                    direction,
                    providers,
                    cancel,
                    &mut scratch,
                    trial,
                )?;
            } else if let Some(template) = self.template.direct_product.clone() {
                self.evaluate_seeded(
                    inputs,
                    DerivativeOrder::Value,
                    None,
                    providers,
                    cancel,
                    trial,
                    None,
                )?;
                let coordinates = Arc::clone(&self.template.coordinate_offsets);
                let worker = self.prepare_direct_worker(&template)?;
                let weights = weights
                    .ok_or_else(|| MathError::Contract("weighted coefficients missing".into()))?;
                template.bind_arguments(
                    inputs,
                    weights,
                    direction,
                    &coordinates,
                    &mut worker.arguments,
                )?;
                worker.values.fill(f64::NAN);
                worker
                    .evaluator
                    .evaluate(&worker.arguments, &mut worker.values)
                    .map_err(|detail| MathError::Evaluation {
                        source_id: SemanticId::NIL,
                        order: DerivativeOrder::Second,
                        detail,
                    })?;
                template.write_product(&worker.values, &mut scratch)?;
            } else {
                if self.template.product == ProductKind::JacobianAction {
                    self.evaluate_directional_trial_into(
                        inputs,
                        direction.ok_or_else(|| MathError::Contract("direction missing".into()))?,
                        providers,
                        cancel,
                        trial,
                    )?;
                    for (row, value) in scratch.iter_mut().enumerate() {
                        *value = trial.first(row, 0);
                    }
                } else if self.template.product == ProductKind::WeightedHessian {
                    self.evaluate_seeded(
                        inputs,
                        DerivativeOrder::Second,
                        None,
                        providers,
                        cancel,
                        trial,
                        None,
                    )?;
                    scratch.fill(0.0);
                    let weights =
                        weights.ok_or_else(|| MathError::Contract("weights missing".into()))?;
                    for (row, &weight) in weights.iter().enumerate() {
                        for (out, &(i, j)) in scratch.iter_mut().zip(self.template.product_pairs())
                        {
                            *out += weight * trial.second(row, i, j);
                        }
                    }
                } else {
                    return Err(MathError::Contract(
                        "requested action program missing".into(),
                    ));
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
            matrix.template.census[DerivativeOrder::Second]
                .unwrap()
                .result_entries,
            3
        );
        assert_eq!(
            matrix.template.census[DerivativeOrder::Second]
                .unwrap()
                .derivative_components,
            1
        );
        assert!(
            matrix.template.programs[DerivativeOrder::Second].is_none(),
            "no per-output second-jet program"
        );
        let mut worker = matrix.worker();
        let mut providers = ProviderBindings::new();
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
            action.template.census[DerivativeOrder::Second]
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
        let support = body
            .prepare_support(&[0, 1], &[1, 0], DerivativeOrder::First, &cancel)
            .unwrap();
        let compiled = support
            .compile_product(
                ProductKind::TransposeJacobianAction,
                Optimization::default(),
                EvaluationLimits::default(),
                &scope,
            )
            .unwrap();
        assert_eq!(compiled.compiled_order(), DerivativeOrder::First);
        assert_eq!(compiled.product_len(), 2);
        let mut worker = compiled.worker_scoped(scope.clone());
        let mut result = [f64::NAN; 2];
        for (weights, expected) in [([2., 3.], [18., 17.]), ([0., -1.], [-2., -3.])] {
            worker
                .evaluate_product_into(
                    &[2., 3.],
                    ProductValues::TransposeJacobianAction { weights: &weights },
                    &mut ProviderBindings::default(),
                    &cancel,
                    &mut result,
                )
                .unwrap();
            assert_eq!(result, expected);
        }
        let x = library::formal(0).unwrap();
        let y = library::formal(1).unwrap();
        let branch = PreparedBody::new(
            3,
            5,
            vec![4],
            vec![
                Stage::Block {
                    expressions: vec![Atom::num(0)],
                    outputs: vec![3],
                    source: SemanticId::NIL,
                },
                Stage::Branch {
                    continuity: DerivativeOrder::Value,
                    comparison: Comparison::Lt,
                    left: 3,
                    right: 2,
                    then: vec![Stage::Block {
                        expressions: vec![(&x * &y).pow(2)],
                        outputs: vec![4],
                        source: SemanticId::NIL,
                    }],
                    otherwise: vec![Stage::Block {
                        expressions: vec![x.pow(3) + y.pow(2)],
                        outputs: vec![4],
                        source: SemanticId::NIL,
                    }],
                },
            ],
            DerivativeOrder::Second,
        )
        .unwrap();
        let support = branch
            .prepare_support(&[0], &[1, 0], DerivativeOrder::First, &cancel)
            .unwrap();
        let compiled = support
            .compile_product(
                ProductKind::TransposeJacobianAction,
                Optimization::default(),
                EvaluationLimits::default(),
                &scope,
            )
            .unwrap();
        assert!(compiled.template.reverse_product.is_some());
        let mut worker = compiled.worker_scoped(scope);
        for (guard, expected) in [(1., [48., 72.]), (-1., [12., 24.]), (1., [48., 72.])] {
            worker
                .evaluate_product_into(
                    &[2., 3., guard],
                    ProductValues::TransposeJacobianAction { weights: &[2.] },
                    &mut ProviderBindings::default(),
                    &cancel,
                    &mut result,
                )
                .unwrap();
            assert_eq!(result, expected);
        }
        let saved = result;
        assert!(
            worker
                .evaluate_product_into(
                    &[1e200, 3., 1.],
                    ProductValues::TransposeJacobianAction { weights: &[0.] },
                    &mut ProviderBindings::default(),
                    &cancel,
                    &mut result
                )
                .is_err()
        );
        assert_eq!(result, saved);
        cancel.cancel();
        assert!(matches!(
            worker.evaluate_product_into(
                &[2., 3., 1.],
                ProductValues::TransposeJacobianAction { weights: &[2.] },
                &mut ProviderBindings::default(),
                &cancel,
                &mut result
            ),
            Err(MathError::Cancelled)
        ));
        assert_eq!(result, saved);
    }

    #[test]
    fn shared_flat_actions_use_staged_products_in_requested_order() {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let y = library::formal(1).unwrap();
        let shared = library::formal(2).unwrap();
        let body = PreparedBody::new(
            2,
            5,
            vec![3, 4],
            vec![
                Stage::Block {
                    expressions: vec![&x * &y],
                    outputs: vec![2],
                    source: SemanticId::NIL,
                },
                Stage::Block {
                    expressions: vec![shared.pow(2), &shared + &x],
                    outputs: vec![3, 4],
                    source: SemanticId::NIL,
                },
            ],
            DerivativeOrder::Second,
        )
        .unwrap();
        let cancel = pse_diagnostics::Cancellation::new(false);
        let scope = pse_kernels::ExecutionScope::new(cancel.clone(), None);
        // In coordinates [y, x], 2(xy+x) + 3(xy)^2 has gradient [76, 116]
        // and Hessian [[24, 74], [74, 54]] at (x, y) = (2, 3).
        for (order, binding, expected) in [
            (
                DerivativeOrder::First,
                ProductValues::TransposeJacobianAction { weights: &[2., 3.] },
                [76., 116.],
            ),
            (
                DerivativeOrder::Second,
                ProductValues::WeightedHessianAction {
                    weights: &[2., 3.],
                    direction: &[5., 7.],
                },
                [638., 748.],
            ),
        ] {
            let compiled = body
                .prepare_support(&[1, 0], &[1, 0], order, &cancel)
                .unwrap()
                .compile_product(
                    binding.kind(),
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &scope,
                )
                .unwrap();
            assert!(compiled.template.reverse_product.is_some());
            assert!(compiled.template.direct_product.is_none());
            assert!(compiled.template.programs[DerivativeOrder::Second].is_none());
            assert!(compiled.product_pairs().is_empty());
            let mut actual = [f64::NAN; 2];
            compiled
                .worker_scoped(scope.clone())
                .evaluate_product_into(
                    &[2., 3.],
                    binding,
                    &mut ProviderBindings::default(),
                    &cancel,
                    &mut actual,
                )
                .unwrap();
            assert_eq!(actual, expected);
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
            if kind == ProductKind::WeightedHessian {
                assert!(census.direct_weighted);
                assert_eq!(census.native_arithmetic_blocks, 1);
                assert!(matches!(
                    compiled.template.direct_product.as_ref().unwrap().evaluator,
                    BlockEvaluator::Native(_)
                ));
            } else {
                assert!(!census.direct_weighted);
                assert!(census.native_arithmetic_blocks > 0);
                assert!(compiled.template.reverse_product.is_some());
            }
            assert!(compiled.template.programs[DerivativeOrder::Second].is_none());
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
                    &mut ProviderBindings::new(),
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
                    &mut ProviderBindings::new(),
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
                        &mut ProviderBindings::new(),
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
                    &mut ProviderBindings::new(),
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
            matches!(worker.evaluate_product_into(&[0.],ProductValues::WeightedHessian{weights:&[1.]},&mut ProviderBindings::new(),&cancel,&mut out),Err(MathError::Domain{source_id,..}) if source_id==source)
        );
        assert_eq!(out, [93.]);
        cancel.cancel();
        assert!(matches!(
            worker.evaluate_product_into(
                &[2.],
                ProductValues::WeightedHessian { weights: &[1.] },
                &mut ProviderBindings::new(),
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
        assert_eq!(
            census.second_pairs, 0,
            "contracted branch action uses local pullbacks without a complete Hessian"
        );
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
                &mut ProviderBindings::new(),
                &cancel,
                &mut output,
            )
            .unwrap();
        assert_eq!(output, [416., 492.]);
        worker
            .evaluate_product_into(
                &[2., 3., -1.],
                binding,
                &mut ProviderBindings::new(),
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
        assert!(compiled.template.programs[DerivativeOrder::Second].is_none());
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
                &mut ProviderBindings::new(),
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
                &mut ProviderBindings::new(),
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
#[cfg(all(test, feature = "compiled-evaluator"))]
#[path = "product_realization_probe.rs"]
mod realization_probe;
