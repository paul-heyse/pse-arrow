// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Composition of library-owned local pullbacks and owner-qualified provider actions.
use super::*;
use pse_kernels::{ProviderDemand, ProviderError};

#[derive(Clone)]
pub(super) struct ReverseProduct {
    stages: Vec<Pullback>,
    pub(super) order: DerivativeOrder,
    bytes: usize,
}
pub(super) struct ReverseWorker {
    stages: Vec<Pullback>,
    adjoints: Vec<f64>,
    tangents: Vec<f64>,
    zero_direction: Vec<f64>,
}
#[derive(Clone)]
enum Pullback {
    Arithmetic {
        evaluator: BlockEvaluator,
        inputs: Vec<usize>,
        outputs: Vec<usize>,
        arguments: Vec<f64>,
        values: Vec<f64>,
        source: SemanticId,
    },
    Provider {
        transpose: PreparedProviderRequest,
        transpose_buffer: ProviderBuffer,
        second: Option<(PreparedProviderRequest, ProviderBuffer)>,
        weights: Vec<f64>,
        weight_tangents: Vec<f64>,
    },
    Branch {
        then: Vec<Self>,
        otherwise: Vec<Self>,
    },
    Control,
}
#[derive(Default)]
struct Cost {
    bytes: usize,
    modules: usize,
    operations: usize,
    instructions: usize,
    instruction_bytes: usize,
    inputs: usize,
    outputs: usize,
    blocks: usize,
    providers: usize,
}
impl Cost {
    fn add_bytes(&mut self, bytes: usize, limits: EvaluationLimits) -> Result<(), MathError> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or(MathError::Limit("local pullback storage"))?;
        if self.bytes > limits.scratch_bytes {
            return Err(MathError::Limit("local pullback storage"));
        }
        Ok(())
    }
}
impl ReverseProduct {
    pub(super) fn prepare(&self, providers: &mut ProviderBindings) -> Result<(), MathError> {
        fn prepare(stages: &[Pullback], providers: &mut ProviderBindings) -> Result<(), MathError> {
            for stage in stages {
                match stage {
                    Pullback::Provider {
                        transpose, second, ..
                    } => {
                        if !providers.supports_linearization(transpose.key()) {
                            return Err(MathError::Contract("composed provider action requires an owner-qualified linearization".into()));
                        }
                        providers
                            .prepare_request(transpose)
                            .map_err(crate::error::scope_error)?;
                        if let Some((request, _)) = second {
                            providers
                                .prepare_request(request)
                                .map_err(crate::error::scope_error)?;
                        }
                    }
                    Pullback::Branch { then, otherwise } => {
                        prepare(then, providers)?;
                        prepare(otherwise, providers)?;
                    }
                    _ => {}
                }
            }
            Ok(())
        }
        prepare(&self.stages, providers)
    }
}
impl ReverseWorker {
    pub(super) fn reset(mut self) -> Result<Self, MathError> {
        fn reset(stages: Vec<Pullback>) -> Result<Vec<Pullback>, MathError> {
            stages
                .into_iter()
                .map(|stage| {
                    Ok(match stage {
                        Pullback::Arithmetic {
                            evaluator,
                            inputs,
                            outputs,
                            mut arguments,
                            mut values,
                            source,
                        } => {
                            arguments.fill(f64::NAN);
                            values.fill(f64::NAN);
                            Pullback::Arithmetic {
                                evaluator: evaluator.reset()?,
                                inputs,
                                outputs,
                                arguments,
                                values,
                                source,
                            }
                        }
                        Pullback::Branch { then, otherwise } => Pullback::Branch {
                            then: reset(then)?,
                            otherwise: reset(otherwise)?,
                        },
                        stage => stage,
                    })
                })
                .collect()
        }
        self.stages = reset(self.stages)?;
        self.adjoints.fill(0.);
        self.tangents.fill(0.);
        self.zero_direction.fill(0.);
        Ok(self)
    }
}
#[allow(
    clippy::too_many_arguments,
    reason = "pullback compilation preserves separate prepared stages, forward association, library cache, work allowance and original scope"
)]
fn compile_pullbacks(
    prepared: &PreparedStages,
    forward: &[CompiledStage],
    second: bool,
    options: Optimization,
    limits: EvaluationLimits,
    scope: &pse_kernels::ExecutionScope,
    scalars: &mut library::ScalarCache,
    cost: &mut Cost,
    next_formal: usize,
    #[cfg(feature = "compiled-evaluator")] native_modules: &mut crate::native_compiled::Batch,
) -> Result<Vec<Pullback>, MathError> {
    if prepared.stages.len() != forward.len() {
        return Err(MathError::Contract(
            "local pullback stage association".into(),
        ));
    }
    cost.add_bytes(
        prepared
            .stages
            .len()
            .checked_mul(size_of::<Pullback>())
            .ok_or(MathError::Limit("local pullback stages"))?,
        limits,
    )?;
    prepared
        .stages
        .iter()
        .zip(forward)
        .map(|(prepared, forward)| {
            scope.check().map_err(crate::error::scope_error)?;
            Ok(match (&prepared.stage, forward) {
                (
                    Stage::Block {
                        expressions,
                        outputs,
                        source,
                    },
                    CompiledStage::Block { .. },
                ) => {
                    let region = prepared
                        .region
                        .as_ref()
                        .ok_or_else(|| MathError::Contract("pullback arithmetic region".into()))?;
                    if region.inputs.is_empty() {
                        return Ok(Pullback::Control);
                    }
                    let weights = (0..outputs.len())
                        .map(|i| library::formal(next_formal + i))
                        .collect::<Result<Vec<_>, _>>()?;
                    let weighted = expressions
                        .iter()
                        .zip(&weights)
                        .fold(Atom::num(0), |sum, (expression, weight)| {
                            sum + expression * weight
                        });
                    let mut gradients = Vec::with_capacity(region.inputs.len());
                    for parameter in &region.parameters {
                        scope.check().map_err(crate::error::scope_error)?;
                        cost.operations = cost
                            .operations
                            .checked_add(1)
                            .ok_or(MathError::Limit("local symbolic derivative calls"))?;
                        if cost.operations > limits.operations {
                            return Err(MathError::Limit("local symbolic derivative calls"));
                        }
                        crate::construction::additional(
                            weighted
                                .as_view()
                                .get_byte_size()
                                .checked_mul(4)
                                .ok_or(MathError::Limit("local pullback construction"))?,
                        )?;
                        gradients.push(
                            weighted.derivative(
                                Indeterminate::try_from(parameter.clone())
                                    .map_err(MathError::Library)?,
                            ),
                        );
                    }
                    let parameters = region
                        .parameters
                        .iter()
                        .chain(&weights)
                        .cloned()
                        .collect::<Vec<_>>();
                    let width = if second { 2 } else { 1 };
                    let remaining = limits
                        .operations
                        .checked_sub(cost.operations)
                        .ok_or(MathError::Limit("local pullback operations"))?;
                    let inspected = if second {
                        let local = JetLayout::new(vec![0], DerivativeOrder::First, limits)?;
                        library::bounded_evaluator(
                            *source,
                            &gradients,
                            &parameters,
                            &local,
                            &[],
                            options,
                            scope.cancellation(),
                            limits,
                            remaining,
                            cost.bytes / size_of::<f64>(),
                            scalars,
                        )?
                    } else {
                        library::bounded_scalar_evaluator(
                            &gradients,
                            &parameters,
                            options,
                            scope.cancellation(),
                            EvaluationLimits {
                                operations: remaining,
                                ..limits
                            },
                            scalars,
                        )?
                    };
                    let storage = inspected.facts.storage;
                    cost.operations = cost
                        .operations
                        .checked_add(operation_count(inspected.facts.operations))
                        .ok_or(MathError::Limit("local pullback operations"))?;
                    cost.instructions += storage.instruction_count;
                    cost.instruction_bytes += storage.instruction_bytes;
                    cost.blocks += 1;
                    let arguments = parameters
                        .len()
                        .checked_mul(width)
                        .ok_or(MathError::Limit("local pullback arguments"))?;
                    let results = gradients
                        .len()
                        .checked_mul(width)
                        .ok_or(MathError::Limit("local pullback results"))?;
                    cost.inputs += arguments;
                    cost.outputs += results;
                    let bytes = arguments
                        .checked_add(results)
                        .and_then(|n| n.checked_add(storage.numeric_entries))
                        .and_then(|n| n.checked_mul(size_of::<f64>()))
                        .and_then(|n| n.checked_add(storage.instruction_bytes))
                        .and_then(|n| {
                            n.checked_add(
                                (region.inputs.len() + outputs.len()) * size_of::<usize>(),
                            )
                        })
                        .ok_or(MathError::Limit("local pullback storage"))?;
                    cost.add_bytes(bytes, limits)?;
                    let evaluator = match options.evaluator {
                        library::Evaluator::Interpreted => {
                            BlockEvaluator::Interpreted(inspected.evaluator)
                        }
                        library::Evaluator::CompiledScalar { module_bytes } => {
                            #[cfg(feature = "compiled-evaluator")]
                            {
                                crate::construction::additional(module_bytes)?;
                                cost.modules = cost
                                    .modules
                                    .checked_add(module_bytes)
                                    .ok_or(MathError::Limit("local pullback modules"))?;
                                BlockEvaluator::Native(native_modules.push(
                                    &inspected.evaluator,
                                    &storage,
                                    &inspected.export,
                                    module_bytes,
                                    scope,
                                )?)
                            }
                            #[cfg(not(feature = "compiled-evaluator"))]
                            {
                                let _ = module_bytes;
                                return Err(MathError::Contract(
                                    "compiled pullback requires compiled-evaluator".into(),
                                ));
                            }
                        }
                    };
                    Pullback::Arithmetic {
                        evaluator,
                        inputs: region.inputs.clone(),
                        outputs: outputs.clone(),
                        arguments: vec![f64::NAN; arguments],
                        values: vec![f64::NAN; results],
                        source: *source,
                    }
                }
                (Stage::Provider { partial, .. }, CompiledStage::Provider { request, .. })
                    if request.order > DerivativeOrder::Value =>
                {
                    if !partial.is_empty()
                        || !matches!(request.demand, ProviderDemand::JacobianAction { .. })
                    {
                        return Err(MathError::Contract(
                            "composed provider partial action unavailable".into(),
                        ));
                    }
                    let prepare = |demand, order| {
                        PreparedProviderRequest::new(
                            Arc::new(request.spec().clone()),
                            ProviderRequest {
                                outputs: request.outputs.clone(),
                                order,
                                demand,
                            },
                        )
                        .map_err(|e| MathError::Contract(e.to_string()))
                    };
                    let transpose = prepare(
                        ProviderDemand::TransposeAction { cotangents: 1 },
                        DerivativeOrder::First,
                    )?;
                    let second_request = if second {
                        Some(prepare(
                            ProviderDemand::WeightedSecondAction { directions: 1 },
                            DerivativeOrder::Second,
                        )?)
                    } else {
                        None
                    };
                    let bytes = transpose
                        .extents()
                        .bytes()
                        .map_err(crate::error::scope_error)?
                        .checked_add(
                            second_request
                                .as_ref()
                                .map(|r| r.extents().bytes())
                                .transpose()
                                .map_err(crate::error::scope_error)?
                                .unwrap_or(0),
                        )
                        .and_then(|n| n.checked_add(2 * request.outputs.len() * size_of::<f64>()))
                        .and_then(|n| {
                            n.checked_add(
                                2 * size_of::<PreparedProviderRequest>()
                                    + 2 * request.outputs.capacity() * size_of::<usize>(),
                            )
                        })
                        .ok_or(MathError::Limit("provider pullback storage"))?;
                    cost.add_bytes(bytes, limits)?;
                    cost.providers += 1 + usize::from(second);
                    Pullback::Provider {
                        transpose_buffer: ProviderBuffer::new(&transpose),
                        transpose,
                        second: second_request.map(|r| {
                            let b = ProviderBuffer::new(&r);
                            (r, b)
                        }),
                        weights: vec![0.; request.outputs.len()],
                        weight_tangents: vec![0.; request.outputs.len()],
                    }
                }
                (
                    Stage::Branch { .. },
                    CompiledStage::Branch {
                        then, otherwise, ..
                    },
                ) => Pullback::Branch {
                    then: compile_pullbacks(
                        &prepared.children[0],
                        then,
                        second,
                        options,
                        limits,
                        scope,
                        scalars,
                        cost,
                        next_formal,
                        #[cfg(feature = "compiled-evaluator")]
                        native_modules,
                    )?,
                    otherwise: compile_pullbacks(
                        &prepared.children[1],
                        otherwise,
                        second,
                        options,
                        limits,
                        scope,
                        scalars,
                        cost,
                        next_formal,
                        #[cfg(feature = "compiled-evaluator")]
                        native_modules,
                    )?,
                },
                _ => Pullback::Control,
            })
        })
        .collect()
}
impl PreparedSupport {
    pub(super) fn compile_reverse_product(
        &self,
        kind: ProductKind,
        options: Optimization,
        limits: EvaluationLimits,
        scope: &pse_kernels::ExecutionScope,
        retained: Option<&library::ScalarProducts>,
    ) -> Result<CompiledBody, MathError> {
        let second = kind == ProductKind::WeightedHessianAction;
        let order = if second {
            DerivativeOrder::Second
        } else {
            DerivativeOrder::First
        };
        let coordinates = self.dense_coordinates()?;
        let selected = (0..self.outputs().len()).collect::<Vec<_>>();
        let available = self.body().selected_order(&selected, &coordinates, false)?;
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
        let mut compiled = self.body().compile_scope(
            &selected,
            &coordinates,
            DerivativeOrder::First,
            options,
            limits,
            scope,
            false,
            ProductKind::JacobianAction,
            self,
            retained,
        )?;
        let mut scalars = library::ScalarCache::from_products(&compiled.scalar_products);
        let mut cost = Cost::default();
        let next_formal = self
            .body()
            .formal_slots
            .iter()
            .copied()
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(MathError::Limit("pullback temporary formals"))?;
        #[cfg(feature = "compiled-evaluator")]
        let mut native_modules = crate::native_compiled::Batch::default();
        let stages = compile_pullbacks(
            &self.program,
            compiled.programs[DerivativeOrder::First]
                .as_ref()
                .ok_or_else(|| MathError::Contract("missing pullback forward program".into()))?,
            second,
            options,
            limits,
            scope,
            &mut scalars,
            &mut cost,
            next_formal,
            #[cfg(feature = "compiled-evaluator")]
            &mut native_modules,
        )?;
        #[cfg(feature = "compiled-evaluator")]
        native_modules.finish(scope)?;
        // Original Second eligibility checks retain their position in the forward program.
        fn require_order(stages: &mut [CompiledStage]) {
            for stage in stages {
                match stage {
                    CompiledStage::Require { order, .. } if *order > DerivativeOrder::First => {
                        *order = DerivativeOrder::First
                    }
                    CompiledStage::Branch {
                        then, otherwise, ..
                    } => {
                        require_order(then);
                        require_order(otherwise);
                    }
                    _ => {}
                }
            }
        }
        if second {
            require_order(
                Arc::make_mut(&mut compiled.programs)[DerivativeOrder::First]
                    .as_mut()
                    .ok_or_else(|| {
                        MathError::Contract("missing mutable pullback forward program".into())
                    })?,
            );
        }
        let buffers = (2 * compiled.slots + coordinates.len())
            .checked_mul(size_of::<f64>())
            .ok_or(MathError::Limit("pullback frame storage"))?;
        cost.add_bytes(buffers, limits)?;
        let forward = compiled.census[DerivativeOrder::First]
            .ok_or_else(|| MathError::Contract("missing pullback forward census".into()))?;
        if forward
            .library_operations
            .checked_add(cost.operations)
            .is_none_or(|n| n > limits.operations)
            || forward
                .provider_stages
                .checked_add(cost.providers)
                .is_none_or(|n| n > limits.provider_calls)
        {
            return Err(MathError::Limit("composed action work"));
        }
        compiled.scratch_bytes = compiled
            .scratch_bytes
            .checked_add(cost.bytes)
            .ok_or(MathError::Limit("composed action scratch"))?;
        if compiled.scratch_bytes > limits.scratch_bytes {
            return Err(MathError::Limit("composed action scratch"));
        }
        compiled.worker_bytes = compiled
            .worker_bytes
            .checked_add(cost.bytes)
            .ok_or(MathError::Limit("composed action worker"))?;
        compiled.retained_bytes = compiled
            .retained_bytes
            .checked_add(cost.bytes)
            .and_then(|n| n.checked_add(cost.modules))
            .and_then(|n| n.checked_add(size_of::<ReverseProduct>()))
            .ok_or(MathError::Limit("composed action retained storage"))?;
        let replaced_scalar_bytes = compiled
            .scalar_products
            .unowned_bytes()
            .checked_add(compiled.scalar_products.descriptor_bytes())
            .ok_or(MathError::Limit("replaced scalar storage"))?;
        let scalar_products = scalars.freeze();
        compiled.retained_bytes = compiled
            .retained_bytes
            .checked_sub(replaced_scalar_bytes)
            .and_then(|n| n.checked_add(scalar_products.unowned_bytes()))
            .and_then(|n| n.checked_add(scalar_products.descriptor_bytes()))
            .ok_or(MathError::Limit("pullback scalar storage"))?;
        compiled.scalar_products = scalar_products;
        let mut census = forward;
        census.product = kind;
        census.order = order;
        census.result_entries = coordinates.len();
        census.arithmetic_blocks += cost.blocks;
        census.arithmetic_input_entries += cost.inputs;
        census.arithmetic_output_entries += cost.outputs;
        census.native_arithmetic_blocks +=
            if matches!(options.evaluator, library::Evaluator::CompiledScalar { .. }) {
                cost.blocks
            } else {
                0
            };
        census.provider_stages += cost.providers;
        census.library_operations += cost.operations;
        census.library_instructions += cost.instructions;
        census.instruction_storage_bytes += cost.instruction_bytes;
        census.worker_bytes += cost.bytes;
        compiled.census[DerivativeOrder::First] = None;
        compiled.census[order] = Some(census);
        compiled.product = kind;
        compiled.reverse_product = Some(Arc::new(ReverseProduct {
            stages,
            order,
            bytes: cost.bytes,
        }));
        scope.check().map_err(crate::error::scope_error)?;
        Ok(compiled)
    }
}
fn instantiate(stages: &mut [Pullback]) -> Result<(), MathError> {
    for stage in stages {
        match stage {
            Pullback::Arithmetic { evaluator, .. } => evaluator.instantiate()?,
            Pullback::Branch { then, otherwise } => {
                instantiate(then)?;
                instantiate(otherwise)?;
            }
            _ => {}
        }
    }
    Ok(())
}
#[allow(
    clippy::too_many_arguments,
    reason = "reverse execution borrows independent primal, cotangent, tangent and provider buffers under the original scope"
)]
fn evaluate_pullbacks(
    stages: &mut [Pullback],
    forward: &mut [CompiledStage],
    frame: &[f64],
    layout: &JetLayout,
    adjoints: &mut [f64],
    tangents: &mut [f64],
    providers: &mut ProviderBindings,
    context: &EvaluationContext<'_>,
    second_order: bool,
) -> Result<(), MathError> {
    for (stage, forward) in stages.iter_mut().zip(forward).rev() {
        context.check().map_err(crate::error::scope_error)?;
        match (stage, forward) {
            (
                Pullback::Arithmetic {
                    evaluator,
                    inputs,
                    outputs,
                    arguments,
                    values,
                    source,
                },
                CompiledStage::Block { .. },
            ) => {
                let width = if second_order { 2 } else { 1 };
                for (i, &slot) in inputs.iter().enumerate() {
                    arguments[width * i] = layout.frame_value(frame, slot, 0);
                    if second_order {
                        arguments[width * i + 1] = layout.frame_value(frame, slot, 1);
                    }
                }
                for (i, &slot) in outputs.iter().enumerate() {
                    arguments[width * (inputs.len() + i)] = adjoints[slot];
                    if second_order {
                        arguments[width * (inputs.len() + i) + 1] = tangents[slot];
                    }
                    adjoints[slot] = 0.;
                    tangents[slot] = 0.;
                }
                evaluator
                    .evaluate(arguments, values)
                    .map_err(|detail| MathError::Evaluation {
                        source_id: *source,
                        order: if second_order {
                            DerivativeOrder::Second
                        } else {
                            DerivativeOrder::First
                        },
                        detail,
                    })?;
                if values.iter().any(|value| !value.is_finite()) {
                    return Err(MathError::Domain {
                        source_id: *source,
                        requirement: "finite library local pullback",
                    });
                }
                for (i, &slot) in inputs.iter().enumerate() {
                    adjoints[slot] += values[width * i];
                    if second_order {
                        tangents[slot] += values[width * i + 1];
                    }
                }
            }
            (
                Pullback::Provider {
                    transpose,
                    transpose_buffer,
                    second,
                    weights,
                    weight_tangents,
                },
                CompiledStage::Provider {
                    inputs,
                    outputs,
                    binding,
                    arguments,
                    direction,
                    linearization,
                    source,
                    ..
                },
            ) => {
                let token = linearization.ok_or_else(|| MathError::Provider {
                    source_id: *source,
                    provider: transpose.spec().id,
                    cause: ProviderError::Contract(
                        "provider did not complete its declared linearization".into(),
                    ),
                })?;
                let (bank, slot) = binding.ok_or_else(|| {
                    MathError::Contract("missing forward provider binding".into())
                })?;
                if bank != providers.identity() {
                    return Err(MathError::Contract(
                        "provider bank changed during composition".into(),
                    ));
                }
                for ((weight, tangent), &slot) in weights
                    .iter_mut()
                    .zip(weight_tangents.iter_mut())
                    .zip(outputs.iter())
                {
                    *weight = adjoints[slot];
                    *tangent = tangents[slot];
                    adjoints[slot] = 0.;
                    tangents[slot] = 0.;
                }
                let provider_id = transpose.spec().id;
                let error = |cause| MathError::Provider {
                    source_id: *source,
                    provider: provider_id,
                    cause,
                };
                if let Some((request, buffer)) = second {
                    providers
                        .bind_prepared_request(slot, request)
                        .map_err(error)?;
                    let complete = providers
                        .fill(
                            slot,
                            buffer,
                            arguments,
                            request,
                            ProviderRuntime {
                                directions: direction,
                                weights,
                                expected_linearization: Some(token),
                                ..Default::default()
                            },
                            context,
                        )
                        .map_err(error)?;
                    for (i, &slot) in inputs.iter().enumerate() {
                        adjoints[slot] += complete.jacobian[i];
                        tangents[slot] += complete.hessians[i];
                    }
                }
                providers
                    .bind_prepared_request(slot, transpose)
                    .map_err(error)?;
                let complete = providers
                    .fill(
                        slot,
                        transpose_buffer,
                        arguments,
                        transpose,
                        ProviderRuntime {
                            cotangents: if second_order {
                                weight_tangents
                            } else {
                                weights
                            },
                            expected_linearization: Some(token),
                            ..Default::default()
                        },
                        context,
                    )
                    .map_err(error)?;
                for (i, &slot) in inputs.iter().enumerate() {
                    if second_order {
                        tangents[slot] += complete.jacobian[i];
                    } else {
                        adjoints[slot] += complete.jacobian[i];
                    }
                }
            }
            (
                Pullback::Branch { then, otherwise },
                CompiledStage::Branch {
                    comparison,
                    left,
                    right,
                    then: forward_then,
                    otherwise: forward_otherwise,
                    ..
                },
            ) => {
                if comparison.select(
                    layout.frame_value(frame, *left, 0),
                    layout.frame_value(frame, *right, 0),
                ) {
                    evaluate_pullbacks(
                        then,
                        forward_then,
                        frame,
                        layout,
                        adjoints,
                        tangents,
                        providers,
                        context,
                        second_order,
                    )?;
                } else {
                    evaluate_pullbacks(
                        otherwise,
                        forward_otherwise,
                        frame,
                        layout,
                        adjoints,
                        tangents,
                        providers,
                        context,
                        second_order,
                    )?;
                }
            }
            (Pullback::Control, _) => {}
            _ => {
                return Err(MathError::Contract(
                    "local pullback executable association".into(),
                ));
            }
        }
    }
    Ok(())
}
impl Worker {
    #[allow(
        clippy::too_many_arguments,
        reason = "product evaluation keeps caller inputs, weights, direction, providers, scope and atomic trial output explicit"
    )]
    pub(super) fn evaluate_reverse_product(
        &mut self,
        inputs: &[f64],
        weights: &[f64],
        direction: Option<&[f64]>,
        providers: &mut ProviderBindings,
        cancel: &Arc<AtomicBool>,
        output: &mut [f64],
        trial: &mut TrialEvaluation,
    ) -> Result<(), MathError> {
        let template = self
            .body
            .reverse_product
            .clone()
            .ok_or_else(|| MathError::Contract("missing composed action product".into()))?;
        self.prepare_providers(providers)?;
        if self.reverse_worker.is_none() {
            let used = self
                .used_bytes
                .checked_add(template.bytes)
                .filter(|&n| n <= self.prepaid_bytes)
                .ok_or(MathError::Limit("composed action worker charge"))?;
            let mut stages = template.stages.clone();
            instantiate(&mut stages)?;
            self.reverse_worker = Some(ReverseWorker {
                stages,
                adjoints: vec![0.; self.body.slots],
                tangents: vec![0.; self.body.slots],
                zero_direction: vec![0.; self.body.coordinate_offsets.len()],
            });
            self.used_bytes = used;
        }
        let mut worker = self
            .reverse_worker
            .take()
            .ok_or_else(|| MathError::Contract("missing composed action worker".into()))?;
        let evaluated = (|| {
            let direction = direction.unwrap_or(&worker.zero_direction);
            self.evaluate_seeded(
                inputs,
                DerivativeOrder::First,
                Some(direction),
                providers,
                cancel,
                trial,
            )?;
            worker.adjoints.fill(0.);
            worker.tangents.fill(0.);
            for (&slot, &weight) in self.body.outputs.iter().zip(weights) {
                worker.adjoints[slot] += weight;
            }
            let layout = self.body.layouts[DerivativeOrder::First]
                .as_ref()
                .ok_or_else(|| MathError::Contract("missing action forward layout".into()))?;
            let forward = self.programs[DerivativeOrder::First]
                .as_mut()
                .ok_or_else(|| MathError::Contract("missing action forward program".into()))?;
            let context = EvaluationContext {
                cancelled: cancel,
                max_result_bytes: self.body.limits.scratch_bytes,
            };
            evaluate_pullbacks(
                &mut worker.stages,
                &mut forward.stages,
                &self.frame,
                layout,
                &mut worker.adjoints,
                &mut worker.tangents,
                providers,
                &context,
                template.order == DerivativeOrder::Second,
            )?;
            for (value, &slot) in output.iter_mut().zip(self.body.coordinate_offsets.iter()) {
                *value = if template.order == DerivativeOrder::Second {
                    worker.tangents[slot]
                } else {
                    worker.adjoints[slot]
                };
            }
            context.check().map_err(crate::error::scope_error)?;
            Ok(())
        })();
        if evaluated.is_err() {
            worker.adjoints.fill(0.);
            worker.tangents.fill(0.);
            trial.clear();
        }
        self.reverse_worker = Some(worker);
        evaluated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_ids::ContentHash;
    use pse_kernels::{
        DerivativeSource, Port, Provider, ProviderBinding, ProviderOutput, ProviderShapes,
    };
    use std::sync::atomic::AtomicUsize;
    #[derive(Debug)]
    struct Actions {
        calls: Arc<[AtomicUsize; 3]>,
        coefficient: Arc<AtomicUsize>,
        drift: bool,
        key: Option<ContentHash>,
    }
    impl Provider for Actions {
        fn supports(&self, demand: &ProviderDemand) -> bool {
            matches!(
                demand,
                ProviderDemand::CompleteJets
                    | ProviderDemand::JacobianAction { directions: 1 }
                    | ProviderDemand::TransposeAction { cotangents: 1 }
                    | ProviderDemand::WeightedSecondAction { directions: 1 }
            )
        }
        fn supports_linearization(&self) -> bool {
            true
        }
        fn completed_linearization(&self) -> Option<ContentHash> {
            self.key
        }
        fn fill(
            &mut self,
            x: &[f64],
            request: &PreparedProviderRequest,
            runtime: ProviderRuntime<'_>,
            output: ProviderOutput<'_>,
            _: &EvaluationContext<'_>,
        ) -> Result<(), ProviderError> {
            let n = x.len();
            if self.drift && matches!(request.demand, ProviderDemand::WeightedSecondAction { .. }) {
                self.coefficient.store(3, Ordering::Relaxed);
            }
            let coefficient = self.coefficient.load(Ordering::Relaxed);
            let c = coefficient as f64;
            let (a, b) = (x[0], x[1]);
            for (row, &original) in request.outputs.iter().enumerate() {
                output.values[row] = if original == 0 {
                    c * a * b
                } else {
                    a * a + b * b * b
                };
            }
            let first = |output: usize, axis: usize| match (output, axis) {
                (0, 0) => c * b,
                (0, 1) => c * a,
                (1, 0) => 2. * a,
                (1, 1) => 3. * b * b,
                _ => 0.,
            };
            let hessian = |output: usize, i: usize, j: usize| match (output, i, j) {
                (0, 0, 1) | (0, 1, 0) => c,
                (1, 0, 0) => 2.,
                (1, 1, 1) => 6. * b,
                _ => 0.,
            };
            match request.demand {
                ProviderDemand::CompleteJets => assert_eq!(
                    request.order,
                    DerivativeOrder::Value,
                    "reverse never requests full partial arrays"
                ),
                ProviderDemand::JacobianAction { .. } => {
                    self.calls[0].fetch_add(1, Ordering::Relaxed);
                    for (row, &original) in request.outputs.iter().enumerate() {
                        output.jacobian[row] = (0..n)
                            .map(|i| first(original, i) * runtime.directions[i])
                            .sum();
                    }
                }
                ProviderDemand::TransposeAction { .. } => {
                    self.calls[1].fetch_add(1, Ordering::Relaxed);
                    for (i, value) in output.jacobian.iter_mut().enumerate() {
                        *value = request
                            .outputs
                            .iter()
                            .enumerate()
                            .map(|(row, &original)| runtime.cotangents[row] * first(original, i))
                            .sum();
                    }
                }
                ProviderDemand::WeightedSecondAction { .. } => {
                    self.calls[2].fetch_add(1, Ordering::Relaxed);
                    for i in 0..n {
                        output.jacobian[i] = request
                            .outputs
                            .iter()
                            .enumerate()
                            .map(|(row, &original)| runtime.weights[row] * first(original, i))
                            .sum();
                        output.hessians[i] = request
                            .outputs
                            .iter()
                            .enumerate()
                            .map(|(row, &original)| {
                                runtime.weights[row]
                                    * (0..n)
                                        .map(|j| hessian(original, i, j) * runtime.directions[j])
                                        .sum::<f64>()
                            })
                            .sum();
                    }
                }
                _ => {
                    return Err(ProviderError::Contract(
                        "fixture narrow product unavailable".into(),
                    ));
                }
            }
            self.key = Some(ContentHash::from_bytes([coefficient as u8; 32]));
            Ok(())
        }
    }
    fn fixture(n: usize) -> (PreparedBody, ProviderSpec) {
        crate::initialize().unwrap();
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let quantity = pse_quantity::standard::ids::quantity("neutral");
        let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
        let port = |i: usize| Port {
            id: pse_ids::named_id(SemanticId::NIL, &i.to_string()),
            quantity,
            unit,
        };
        let spec = ProviderSpec {
            id: SemanticId::from_bytes([78; 16]),
            revision: ContentHash::from_bytes([1; 32]),
            data: ContentHash::from_bytes([2; 32]),
            inputs: (0..n).map(port).collect(),
            outputs: vec![port(n), port(n + 1)],
            shapes: ProviderShapes::default(),
            derivative_source: DerivativeSource::Analytic,
            derivatives: DerivativeOrder::Second,
            smoothness: DerivativeOrder::Second,
        };
        let z = library::formal(n).unwrap();
        let q = library::formal(n + 1).unwrap();
        let x = library::formal(2).unwrap();
        let body = PreparedBody::new_with_allowance(
            n,
            n + 3,
            vec![n + 2],
            vec![
                Stage::Require {
                    argument: 2,
                    condition: Condition::Positive,
                    order: DerivativeOrder::Second,
                    source: SemanticId::from_bytes([81; 16]),
                    lineage: None,
                },
                Stage::Provider {
                    spec: spec.clone(),
                    partial: vec![],
                    inputs: (0..n).collect(),
                    outputs: vec![n, n + 1],
                    source: SemanticId::from_bytes([79; 16]),
                },
                Stage::Block {
                    expressions: vec![&z * &z + &q * &x],
                    outputs: vec![n + 2],
                    source: SemanticId::from_bytes([80; 16]),
                },
            ],
            &mut 1_000_000,
        )
        .unwrap();
        (body, spec)
    }
    #[test]
    fn composed_actions_use_owner_qualified_narrow_provider_products_and_library_pullbacks() {
        composed_actions(Optimization::default());
    }
    #[cfg(feature = "compiled-evaluator")]
    #[test]
    fn composed_actions_compiled_local_pullbacks_preserve_owner_qualified_provider_actions() {
        composed_actions(Optimization {
            evaluator: library::Evaluator::CompiledScalar {
                module_bytes: 1 << 20,
            },
            ..Default::default()
        });
    }
    fn composed_actions(options: Optimization) {
        let n = 128;
        let (body, spec) = fixture(n);
        let stop = pse_diagnostics::Cancellation::new(false);
        let scope = pse_kernels::ExecutionScope::new(stop.clone(), None);
        let mut point = vec![0.; n];
        point[..3].copy_from_slice(&[2., 3., 4.]);
        let mut direction = vec![0.; n];
        direction[..3].copy_from_slice(&[5., 7., 11.]);
        for kind in [
            ProductKind::TransposeJacobianAction,
            ProductKind::WeightedHessianAction,
        ] {
            let order = if kind == ProductKind::TransposeJacobianAction {
                DerivativeOrder::First
            } else {
                DerivativeOrder::Second
            };
            let support = body
                .prepare_support(&[0], &(0..n).collect::<Vec<_>>(), order, &stop)
                .unwrap();
            let program = support
                .compile_product(
                    kind,
                    options,
                    EvaluationLimits {
                        scratch_bytes: 128 * 1024,
                        ..Default::default()
                    },
                    &scope,
                )
                .unwrap();
            assert!(program.census().all(|c| c.derivative_components == 2));
            if matches!(options.evaluator, library::Evaluator::CompiledScalar { .. }) {
                assert!(
                    program
                        .census()
                        .all(|c| c.native_arithmetic_blocks == c.arithmetic_blocks)
                );
            }
            let calls = Arc::new(std::array::from_fn(|_| AtomicUsize::new(0)));
            let provider = ProviderBinding::new(
                spec.clone(),
                Box::new(Actions {
                    calls: calls.clone(),
                    coefficient: Arc::new(AtomicUsize::new(1)),
                    drift: false,
                    key: None,
                }),
            );
            let mut bank = ProviderBindings::from([(spec.key(), provider)]);
            let mut worker = program.worker_scoped(scope.clone());
            worker.prepare_providers(&mut bank).unwrap();
            let mut output = vec![99.; n];
            for weight in [1., -2.] {
                let values = if kind == ProductKind::TransposeJacobianAction {
                    ProductValues::TransposeJacobianAction {
                        weights: std::slice::from_ref(&weight),
                    }
                } else {
                    ProductValues::WeightedHessianAction {
                        weights: std::slice::from_ref(&weight),
                        direction: &direction,
                    }
                };
                worker
                    .evaluate_product_into(&point, values, &mut bank, &stop, &mut output)
                    .unwrap();
                let expected = if kind == ProductKind::TransposeJacobianAction {
                    [52., 132., 31.]
                } else {
                    [
                        5. * 26. + 7. * 24. + 11. * 4.,
                        5. * 24. + 7. * 80. + 11. * 27.,
                        5. * 4. + 7. * 27.,
                    ]
                };
                for (actual, expected) in output[..3].iter().zip(expected) {
                    assert!(
                        (*actual - weight * expected).abs() < 1e-10,
                        "{kind:?}: {actual} != {}",
                        weight * expected
                    );
                }
                assert!(output[3..].iter().all(|&value| value == 0.));
            }
            assert_eq!(calls[0].load(Ordering::Relaxed), 2);
            assert_eq!(calls[1].load(Ordering::Relaxed), 2);
            assert_eq!(
                calls[2].load(Ordering::Relaxed),
                if order == DerivativeOrder::Second {
                    2
                } else {
                    0
                }
            );
        }
    }
    #[test]
    fn composed_action_changed_followup_premise_publishes_no_result_and_recovers() {
        let (body, spec) = fixture(3);
        let stop = pse_diagnostics::Cancellation::new(false);
        let scope = pse_kernels::ExecutionScope::new(stop.clone(), None);
        let support = body
            .prepare_support(&[0], &[0, 1, 2], DerivativeOrder::Second, &stop)
            .unwrap();
        let program = support
            .compile_product(
                ProductKind::WeightedHessianAction,
                Optimization::default(),
                EvaluationLimits::default(),
                &scope,
            )
            .unwrap();
        let calls = Arc::new(std::array::from_fn(|_| AtomicUsize::new(0)));
        let coefficient = Arc::new(AtomicUsize::new(1));
        let provider = ProviderBinding::new(
            spec.clone(),
            Box::new(Actions {
                calls,
                coefficient: coefficient.clone(),
                drift: true,
                key: None,
            }),
        );
        let mut bank = ProviderBindings::from([(spec.key(), provider)]);
        let mut worker = program.worker_scoped(scope);
        let mut output = [91.; 3];
        let mut trial = TrialEvaluation::default();
        let values = ProductValues::WeightedHessianAction {
            weights: &[1.],
            direction: &[1., 2., 3.],
        };
        assert!(matches!(
            worker.evaluate_product_trial_into(
                &[0., 3., 4.],
                values,
                &mut bank,
                &stop,
                &mut output,
                &mut trial
            ),
            Err(MathError::Provider {
                cause: ProviderError::LinearizationChanged,
                ..
            })
        ));
        assert_eq!(output, [91.; 3]);
        assert!(trial.provider_visits().is_empty());
        // The next invocation establishes the now-current scientific context afresh.
        worker
            .evaluate_product_trial_into(
                &[0., 3., 4.],
                values,
                &mut bank,
                &stop,
                &mut output,
                &mut trial,
            )
            .unwrap();
        assert_eq!(
            trial.provider_visits().len(),
            1,
            "internal followups do not duplicate original call occurrences"
        );
        assert!(trial.provider_visits()[0].linearization.is_some());
    }
    #[test]
    fn composed_action_unqualified_owner_refuses_before_effects_and_second_guard_survives_zero_weights()
     {
        #[derive(Debug)]
        struct Unknown(Actions);
        impl Provider for Unknown {
            fn supports(&self, demand: &ProviderDemand) -> bool {
                self.0.supports(demand)
            }
            fn fill(
                &mut self,
                x: &[f64],
                r: &PreparedProviderRequest,
                runtime: ProviderRuntime<'_>,
                out: ProviderOutput<'_>,
                c: &EvaluationContext<'_>,
            ) -> Result<(), ProviderError> {
                self.0.fill(x, r, runtime, out, c)
            }
        }
        let (body, spec) = fixture(3);
        let stop = pse_diagnostics::Cancellation::new(false);
        let scope = pse_kernels::ExecutionScope::new(stop.clone(), None);
        let support = body
            .prepare_support(&[0], &[0, 1, 2], DerivativeOrder::Second, &stop)
            .unwrap();
        let program = support
            .compile_product(
                ProductKind::WeightedHessianAction,
                Optimization::default(),
                EvaluationLimits::default(),
                &scope,
            )
            .unwrap();
        let calls = Arc::new(std::array::from_fn(|_| AtomicUsize::new(0)));
        let make = || Actions {
            calls: calls.clone(),
            coefficient: Arc::new(AtomicUsize::new(1)),
            drift: false,
            key: None,
        };
        let unknown = ProviderBinding::new(spec.clone(), Box::new(Unknown(make())));
        let mut bank = ProviderBindings::from([(spec.key(), unknown)]);
        let mut worker = program.worker_scoped(scope.clone());
        let mut output = [91.; 3];
        let values = ProductValues::WeightedHessianAction {
            weights: &[0.],
            direction: &[1., 2., 3.],
        };
        assert!(matches!(
            worker.evaluate_product_into(&[2., 3., 4.], values, &mut bank, &stop, &mut output),
            Err(MathError::Contract(_))
        ));
        assert_eq!(output, [91.; 3]);
        assert!(calls.iter().all(|count| count.load(Ordering::Relaxed) == 0));
        let known = ProviderBinding::new(spec.clone(), Box::new(make()));
        let mut bank = ProviderBindings::from([(spec.key(), known)]);
        assert!(
            matches!(worker.evaluate_product_into(&[2.,3.,-1.],values,&mut bank,&stop,&mut output),
            Err(MathError::Domain{source_id,..}) if source_id==SemanticId::from_bytes([81;16]))
        );
        assert_eq!(output, [91.; 3]);
        assert!(calls.iter().all(|count| count.load(Ordering::Relaxed) == 0));
    }
    #[test]
    fn composed_action_mixed_branch_provider_preserves_active_visits_and_inactive_guards() {
        let (source, spec) = fixture(4);
        let a = library::formal(0).unwrap();
        let body = PreparedBody::new(
            4,
            7,
            vec![6],
            vec![Stage::Branch {
                continuity: DerivativeOrder::Value,
                comparison: Comparison::Lt,
                left: 2,
                right: 3,
                then: source.stages.clone(),
                otherwise: vec![Stage::Block {
                    expressions: vec![&a * &a],
                    outputs: vec![6],
                    source: SemanticId::from_bytes([82; 16]),
                }],
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let stop = pse_diagnostics::Cancellation::new(false);
        let scope = pse_kernels::ExecutionScope::new(stop.clone(), None);
        let support = body
            .prepare_support(&[0], &[0, 1], DerivativeOrder::Second, &stop)
            .unwrap();
        let program = support
            .compile_product(
                ProductKind::WeightedHessianAction,
                Optimization::default(),
                EvaluationLimits::default(),
                &scope,
            )
            .unwrap();
        let calls = Arc::new(std::array::from_fn(|_| AtomicUsize::new(0)));
        let provider = ProviderBinding::new(
            spec.clone(),
            Box::new(Actions {
                calls: calls.clone(),
                coefficient: Arc::new(AtomicUsize::new(1)),
                drift: false,
                key: None,
            }),
        );
        let mut bank = ProviderBindings::from([(spec.key(), provider)]);
        let mut worker = program.worker_scoped(scope);
        let mut trial = TrialEvaluation::default();
        let mut output = [91.; 2];
        let values = ProductValues::WeightedHessianAction {
            weights: &[1.],
            direction: &[5., 7.],
        };
        worker
            .evaluate_product_trial_into(
                &[2., 3., 4., 5.],
                values,
                &mut bank,
                &stop,
                &mut output,
                &mut trial,
            )
            .unwrap();
        assert_eq!(output, [298., 680.]);
        assert_eq!(trial.provider_visits().len(), 1);
        let before = calls.each_ref().map(|count| count.load(Ordering::Relaxed));
        worker
            .evaluate_product_trial_into(
                &[2., 3., -1., -2.],
                values,
                &mut bank,
                &stop,
                &mut output,
                &mut trial,
            )
            .unwrap();
        assert_eq!(output, [10., 0.]);
        assert!(trial.provider_visits().is_empty());
        assert_eq!(
            calls.each_ref().map(|count| count.load(Ordering::Relaxed)),
            before,
            "inactive provider and its guard remain unvisited"
        );
    }
}
