// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Schedule pure arithmetic regions over independent rows. Scalar-to-lane
//! packing and scattering remain entirely inside the supplier batch adapter.
use super::*;

const SEGMENT: usize = 256;
pub(super) const fn retained_rows(rows: usize) -> usize {
    if rows < SEGMENT { rows } else { SEGMENT }
}
#[derive(Default, Debug)]
pub(in crate::execution) struct Scratch {
    frames: Vec<Vec<f64>>,
    trials: Vec<TrialEvaluation>,
    failures: Vec<Option<MathError>>,
    active: Vec<usize>,
    arguments: Vec<f64>,
    values: Vec<f64>,
    products: Vec<f64>,
    pub(in crate::execution) admitted: usize,
}
pub(in crate::execution) struct Row<'a> {
    pub(super) trial: &'a TrialEvaluation,
    pub(super) product: Option<&'a [f64]>,
}
/// Select unsupported x4 action/control shapes before any row effect. Scalar
/// profiles retain their existing independent realization; there is no retry.
pub(super) fn validate_profile(
    body: &CompiledBody,
    order: DerivativeOrder,
) -> Result<(), MathError> {
    if !matches!(
        body.optimization().evaluator,
        library::Evaluator::CompiledX4 { .. }
    ) {
        return Ok(());
    }
    let template = &body.template;
    if template.reverse_product.is_some() {
        return Err(MathError::Contract(
            "x4 batch reverse products are unsupported".into(),
        ));
    }
    let actual = if let Some(direct) = &template.direct_product {
        if direct.packed_shape().is_none() {
            return Err(MathError::Contract(
                "x4 batch requires a qualified direct native product".into(),
            ));
        }
        DerivativeOrder::Value
    } else {
        if !matches!(
            template.product,
            ProductKind::SelectedPartials | ProductKind::JacobianAction
        ) {
            return Err(MathError::Contract(
                "x4 batch product has no packed realization".into(),
            ));
        }
        order
    };
    check_stages(
        template.programs[actual]
            .as_ref()
            .ok_or_else(|| MathError::Contract("x4 batch order not compiled".into()))?,
    )
}
impl Scratch {
    pub(super) fn prepaid(bytes: usize) -> Self {
        Self {
            admitted: bytes,
            ..Self::default()
        }
    }
}
pub(super) fn has_packed(stages: &[CompiledStage]) -> bool {
    stages.iter().any(|stage| match stage {
        CompiledStage::Block {
            evaluator: BlockEvaluator::Native(block),
            ..
        } => block.executable.is_packed(),
        CompiledStage::Domain { stages, .. } | CompiledStage::Applicability { stages, .. } => {
            has_packed(stages)
        }
        CompiledStage::Branch {
            then, otherwise, ..
        } => has_packed(then) || has_packed(otherwise),
        _ => false,
    })
}
fn check_stages(stages: &[CompiledStage]) -> Result<(), MathError> {
    if pure_branch_free_stages(stages) {
        Ok(())
    } else {
        Err(MathError::Contract(
            "packed batch requires pure branch-free stages".into(),
        ))
    }
}

fn resize<T: Clone>(values: &mut Vec<T>, length: usize, value: T) -> Result<(), MathError> {
    if length > values.len() {
        values
            .try_reserve_exact(length - values.len())
            .map_err(|_| MathError::Limit("packed batch scratch allocation"))?;
    }
    values.resize(length, value);
    Ok(())
}
impl BatchWorker {
    pub(super) fn uses_packed(&self) -> bool {
        let template = &self.worker.numeric.template;
        template
            .direct_product
            .as_ref()
            .is_some_and(|direct| direct.packed_shape().is_some())
            || matches!(
                self.layout.product(),
                ProductKind::SelectedPartials | ProductKind::JacobianAction
            ) && template.programs[self.layout.order]
                .as_ref()
                .is_some_and(|program| has_packed(program))
    }
    pub(super) fn evaluate_packed(
        &mut self,
        request: BatchRequest<'_>,
        cancel: &Arc<AtomicBool>,
        mut output: BatchOutcome,
        mut completed: impl FnMut(usize),
    ) -> Result<BatchOutcome, MathError> {
        run(
            &mut self.worker.numeric,
            &mut self.packed,
            Request {
                inputs: request.inputs,
                rows: request.points.len(),
                order: self.layout.order,
                bindings: request.bindings,
            },
            cancel,
            |index, result| {
                let (outcome, applicability) = match result {
                    Err(MathError::Cancelled) => {
                        output.canceled = true;
                        (PointOutcome::NotExecuted, Vec::new())
                    }
                    Err(error) => (PointOutcome::Failed(error), Vec::new()),
                    Ok(row) => match {
                        let target = &mut output.cells
                            [index * self.layout.cells..(index + 1) * self.layout.cells];
                        if let Some(product) = row.product {
                            target.copy_from_slice(product);
                            Ok(())
                        } else if self.layout.product() == ProductKind::JacobianAction {
                            for (output, value) in target.iter_mut().enumerate() {
                                *value = row.trial.first(output, 0);
                            }
                            Ok(())
                        } else {
                            copy_trial(row.trial, &self.layout, target)
                        }
                    } {
                        Ok(()) => (
                            PointOutcome::Succeeded,
                            row.trial
                                .applicability
                                .iter()
                                .flat_map(ApplicabilityWitness::hydrate)
                                .collect(),
                        ),
                        Err(error) => (PointOutcome::Failed(error), Vec::new()),
                    },
                };
                output.points.push(PointResult {
                    id: request.points[index],
                    outcome,
                    applicability,
                });
                completed(index);
            },
        )?;
        output.canceled |= cancel.load(Ordering::Acquire);
        Ok(output)
    }
}
pub(in crate::execution) fn backing_bytes(
    template: &ExecutionTemplate,
    rows: usize,
    order: DerivativeOrder,
) -> Result<usize, MathError> {
    let order = if template.direct_product.is_some() {
        DerivativeOrder::Value
    } else {
        order
    };
    let program = template.programs[order]
        .as_ref()
        .ok_or_else(|| MathError::Contract("batch order not compiled".into()))?;
    let scratch_rows = rows.min(SEGMENT);
    let (mut max_inputs, mut max_outputs, mut lane_entries) = (0usize, 0usize, 0usize);
    for stage in program {
        if let CompiledStage::Block {
            arguments, values, ..
        } = stage
        {
            max_inputs = max_inputs.max(arguments.len());
            max_outputs = max_outputs.max(values.len());
            lane_entries = lane_entries
                .checked_add(arguments.len())
                .and_then(|n| n.checked_add(values.len()))
                .ok_or(MathError::Limit("retained lane buffers"))?;
        }
    }
    let product_cells = if let Some(direct) = &template.direct_product {
        let (inputs, outputs, _) = direct
            .packed_shape()
            .ok_or_else(|| MathError::Contract("packed direct product missing".into()))?;
        max_inputs = max_inputs.max(inputs);
        max_outputs = max_outputs.max(outputs);
        lane_entries = lane_entries
            .checked_add(inputs)
            .and_then(|n| n.checked_add(outputs))
            .ok_or(MathError::Limit("direct lane buffers"))?;
        template.product_len()
    } else {
        0
    };
    let layout = template.layouts[order]
        .as_ref()
        .ok_or_else(|| MathError::Contract("batch layout missing".into()))?;
    let frames = layout
        .frame_entries(template.slots)?
        .checked_mul(size_of::<f64>())
        .ok_or(MathError::Limit("packed frames"))?;
    let backing = frames
        .checked_add(
            template
                .evaluation_cache_bytes
                .checked_mul(2)
                .ok_or(MathError::Limit("packed trial overlap"))?,
        )
        .and_then(|n| {
            n.checked_add(
                size_of::<Vec<f64>>()
                    + size_of::<TrialEvaluation>()
                    + size_of::<Option<MathError>>()
                    + size_of::<usize>(),
            )
        })
        .and_then(|n| {
            n.checked_add(
                max_inputs
                    .checked_add(max_outputs)?
                    .checked_add(lane_entries)?
                    .checked_add(product_cells)?
                    .checked_mul(size_of::<f64>())?,
            )
        })
        .and_then(|n| n.checked_mul(scratch_rows))
        .ok_or(MathError::Limit("packed batch scratch extent"))?;
    Ok(backing)
}

pub(in crate::execution) struct Request<'a> {
    pub(super) inputs: &'a [f64],
    pub(super) rows: usize,
    pub(super) order: DerivativeOrder,
    pub(super) bindings: BatchBindings<'a>,
}
pub(in crate::execution) fn run(
    numeric: &mut NumericFrame,
    scratch: &mut Scratch,
    request: Request<'_>,
    cancel: &Arc<AtomicBool>,
    mut receive: impl FnMut(usize, Result<Row<'_>, MathError>),
) -> Result<(), MathError> {
    let Request {
        inputs,
        rows,
        order,
        bindings,
    } = request;
    let mut canceled = false;
    let template = numeric.template.clone();
    let order = if template.direct_product.is_some() {
        DerivativeOrder::Value
    } else {
        order
    };
    let (weights, directions) = bindings.parts();
    let program = template.programs[order]
        .as_ref()
        .ok_or_else(|| MathError::Contract("packed batch order".into()))?;
    check_stages(program)?;
    let scope = numeric
        .scope
        .as_ref()
        .ok_or_else(|| MathError::Contract("detached packed batch".into()))?
        .clone();
    match scope.check().map_err(crate::error::scope_error) {
        Ok(()) => {}
        Err(MathError::Cancelled) => canceled = true,
        Err(error) => return Err(error),
    }
    if canceled || cancel.load(Ordering::Acquire) {
        for index in 0..rows {
            receive(index, Err(MathError::Cancelled));
        }
        return Ok(());
    }
    numeric.prepare_order(order)?;
    let layout = template.layouts[order]
        .as_ref()
        .ok_or_else(|| MathError::Contract("packed batch layout".into()))?;
    let entries = layout.frame_entries(template.slots)?;
    let scratch_rows = rows.min(SEGMENT);
    let backing = backing_bytes(&template, rows, order)?;
    let lane_allowance = backing
        .checked_add(template.order_worker_bytes[order])
        .and_then(|n| {
            n.checked_add(
                template
                    .direct_product
                    .as_ref()
                    .and_then(|direct| direct.packed_shape())
                    .map_or(0, |(_, _, bytes)| bytes),
            )
        })
        .ok_or(MathError::Limit("prepaid native lane backing"))?;
    if backing > template.limits.scratch_bytes {
        return Err(MathError::Limit("packed batch scratch bytes"));
    }
    if backing > scratch.admitted {
        crate::construction::additional(backing - scratch.admitted)?;
        scratch.admitted = backing;
    }
    while scratch.frames.len() < scratch_rows {
        scratch
            .frames
            .try_reserve_exact(1)
            .map_err(|_| MathError::Limit("packed row frames"))?;
        scratch.frames.push(Vec::new());
        scratch
            .trials
            .try_reserve_exact(1)
            .map_err(|_| MathError::Limit("packed row trials"))?;
        scratch.trials.push(TrialEvaluation::default());
    }
    scratch
        .failures
        .try_reserve_exact(scratch_rows.saturating_sub(scratch.failures.len()))
        .map_err(|_| MathError::Limit("packed row failures"))?;
    scratch
        .active
        .try_reserve_exact(scratch_rows.saturating_sub(scratch.active.len()))
        .map_err(|_| MathError::Limit("packed row map"))?;
    let mut providers = ProviderBindings::new();
    for start in (0..rows).step_by(SEGMENT) {
        let count = (rows - start).min(SEGMENT);
        if canceled || cancel.load(Ordering::Acquire) {
            canceled = true;
            for row in 0..count {
                receive(start + row, Err(MathError::Cancelled));
            }
            continue;
        }
        scratch.failures.clear();
        scratch.failures.resize_with(count, || None);
        for row in 0..count {
            resize(&mut scratch.frames[row], entries, f64::NAN)?;
            scratch.frames[row].fill(f64::NAN);
            scratch.trials[row].clear();
            let inputs =
                &inputs[(start + row) * template.inputs..(start + row + 1) * template.inputs];
            let direction = directions.map(|values| {
                &values[(start + row) * template.coordinate_offsets.len()
                    ..(start + row + 1) * template.coordinate_offsets.len()]
            });
            let weights = weights.map(|values| {
                &values[(start + row) * template.outputs.len()
                    ..(start + row + 1) * template.outputs.len()]
            });
            if inputs.iter().any(|value| !value.is_finite()) {
                scratch.failures[row] = Some(MathError::Contract(
                    "finite ordered formal inputs required".into(),
                ));
            } else if direction.is_some_and(|values| values.iter().any(|value| !value.is_finite()))
                || weights.is_some_and(|values| values.iter().any(|value| !value.is_finite()))
            {
                scratch.failures[row] = Some(MathError::Contract(
                    "finite exact product binding extent/kind".into(),
                ));
            } else {
                seed_frame(
                    &template,
                    &mut scratch.frames[row],
                    layout,
                    inputs,
                    direction,
                );
            }
        }
        let stages = &mut numeric.programs[order]
            .as_mut()
            .ok_or_else(|| MathError::Contract("packed worker order".into()))?
            .stages;
        for stage in stages {
            match scope.check().map_err(crate::error::scope_error) {
                Ok(()) => {}
                Err(MathError::Cancelled) => {
                    canceled = true;
                    break;
                }
                Err(error) => return Err(error),
            }
            if cancel.load(Ordering::Acquire) {
                canceled = true;
                break;
            }
            scratch.active.clear();
            scratch
                .active
                .extend((0..count).filter(|&row| scratch.failures[row].is_none()));
            if let CompiledStage::Block {
                evaluator: BlockEvaluator::Native(block),
                gather,
                scatter,
                arguments,
                values,
                checks,
                checked_ends,
                ..
            } = stage
            {
                let inputs = arguments.len();
                let outputs = values.len();
                let active = scratch.active.len();
                resize(
                    &mut scratch.arguments,
                    active
                        .checked_mul(inputs)
                        .ok_or(MathError::Limit("packed argument extent"))?,
                    0.,
                )?;
                resize(
                    &mut scratch.values,
                    active
                        .checked_mul(outputs)
                        .ok_or(MathError::Limit("packed result extent"))?,
                    f64::NAN,
                )?;
                for (position, &row) in scratch.active.iter().enumerate() {
                    scratch.arguments[position * inputs..(position + 1) * inputs]
                        .copy_from_slice(arguments);
                    for &(argument, slot) in gather.iter() {
                        scratch.arguments[position * inputs + argument] = scratch.frames[row][slot];
                    }
                }
                match block.executable.evaluate_batch(
                    active,
                    &scratch.arguments,
                    &mut scratch.values,
                    &scope,
                    lane_allowance,
                ) {
                    Ok(()) => {}
                    Err(MathError::Cancelled) => {
                        canceled = true;
                        break;
                    }
                    Err(error) => return Err(error),
                }
                for (position, &row) in scratch.active.iter().enumerate() {
                    if let Err(error) = publish_block(
                        &mut scratch.frames[row],
                        scatter,
                        &scratch.values[position * outputs..(position + 1) * outputs],
                        checks,
                        checked_ends,
                    ) {
                        scratch.failures[row] = Some(error);
                    }
                }
            } else {
                for &row in &scratch.active {
                    let trial = &mut scratch.trials[row];
                    let context = EvaluationContext {
                        cancelled: cancel,
                        max_result_bytes: template.limits.scratch_bytes,
                    };
                    if let Err(error) = evaluate_stages(
                        std::slice::from_mut(stage),
                        &mut scratch.frames[row],
                        layout,
                        &mut providers,
                        &context,
                        &mut trial.applicability,
                        &mut trial.spare_witnesses,
                        &mut trial.provider_visits,
                        &mut None,
                    ) {
                        scratch.failures[row] = Some(error);
                    }
                }
            }
        }
        if let Some(direct) = &template.direct_product
            && !canceled
            && !cancel.load(Ordering::Acquire)
        {
            scratch.active.clear();
            for row in 0..count {
                if scratch.failures[row].is_none() {
                    match publish_trial(
                        &template,
                        &scratch.frames[row],
                        layout,
                        &mut scratch.trials[row],
                    ) {
                        Ok(()) => scratch.active.push(row),
                        Err(error) => scratch.failures[row] = Some(error),
                    }
                }
            }
            let (arguments, results, _) = direct
                .packed_shape()
                .ok_or_else(|| MathError::Contract("packed direct product missing".into()))?;
            resize(
                &mut scratch.arguments,
                scratch
                    .active
                    .len()
                    .checked_mul(arguments)
                    .ok_or(MathError::Limit("direct batch arguments"))?,
                f64::NAN,
            )?;
            resize(
                &mut scratch.values,
                scratch
                    .active
                    .len()
                    .checked_mul(results)
                    .ok_or(MathError::Limit("direct batch values"))?,
                f64::NAN,
            )?;
            resize(
                &mut scratch.products,
                count
                    .checked_mul(template.product_len())
                    .ok_or(MathError::Limit("direct batch products"))?,
                f64::NAN,
            )?;
            for (ordinal, &row) in scratch.active.iter().enumerate() {
                let index = start + row;
                let weights = weights
                    .ok_or_else(|| MathError::Contract("direct batch weights missing".into()))?;
                direct.bind_arguments(
                    &inputs[index * template.inputs..(index + 1) * template.inputs],
                    &weights[index * template.outputs.len()..(index + 1) * template.outputs.len()],
                    directions.map(|values| {
                        &values[index * template.coordinate_offsets.len()
                            ..(index + 1) * template.coordinate_offsets.len()]
                    }),
                    &template.coordinate_offsets,
                    &mut scratch.arguments[ordinal * arguments..(ordinal + 1) * arguments],
                )?;
            }
            if !canceled && !cancel.load(Ordering::Acquire) {
                match numeric.prepare_direct_worker(direct)?.evaluate_batch(
                    scratch.active.len(),
                    &scratch.arguments,
                    &mut scratch.values,
                    &scope,
                    lane_allowance,
                ) {
                    Ok(()) => {}
                    Err(MathError::Cancelled) => canceled = true,
                    Err(error) => return Err(error),
                }
                if !canceled {
                    for (ordinal, &row) in scratch.active.iter().enumerate() {
                        let target = &mut scratch.products
                            [row * template.product_len()..(row + 1) * template.product_len()];
                        direct.write_product(
                            &scratch.values[ordinal * results..(ordinal + 1) * results],
                            target,
                        )?;
                        if target.iter().any(|value| !value.is_finite()) {
                            scratch.failures[row] = Some(MathError::Domain {
                                source_id: SemanticId::NIL,
                                requirement: "finite requested product",
                            });
                        }
                    }
                }
            }
        }
        for row in 0..count {
            let index = start + row;
            let trial = &mut scratch.trials[row];
            if cancel.load(Ordering::Acquire) || canceled {
                canceled = true;
                receive(index, Err(MathError::Cancelled));
            } else if let Some(error) = scratch.failures[row].take() {
                receive(index, Err(error));
            } else {
                if template.direct_product.is_some() {
                    receive(
                        index,
                        Ok(Row {
                            trial,
                            product: Some(
                                &scratch.products[row * template.product_len()
                                    ..(row + 1) * template.product_len()],
                            ),
                        }),
                    );
                } else {
                    let result = publish_trial(&template, &scratch.frames[row], layout, trial);
                    receive(
                        index,
                        result.map(|()| Row {
                            trial: &*trial,
                            product: None,
                        }),
                    );
                }
            }
            trial.clear();
            scratch.frames[row].fill(f64::NAN);
        }
    }
    Ok(())
}
