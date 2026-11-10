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
    pub(in crate::execution) admitted: usize,
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
        _ => false,
    })
}
fn check_stages(stages: &[CompiledStage]) -> Result<(), MathError> {
    for stage in stages {
        match stage {
            CompiledStage::Branch { .. } | CompiledStage::Provider { .. } => {
                return Err(MathError::Contract(
                    "packed batch requires pure branch-free stages".into(),
                ));
            }
            CompiledStage::Domain { stages, .. } | CompiledStage::Applicability { stages, .. } => {
                check_stages(stages)?
            }
            _ => {}
        }
    }
    Ok(())
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
        self.layout.product() == ProductKind::SelectedPartials
            && self.worker.numeric.template.programs[self.layout.order]
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
            request.inputs,
            request.points.len(),
            self.layout.order,
            cancel,
            |index, result| {
                let (outcome, applicability) = match result {
                    Err(MathError::Cancelled) => {
                        output.canceled = true;
                        (PointOutcome::NotExecuted, Vec::new())
                    }
                    Err(error) => (PointOutcome::Failed(error), Vec::new()),
                    Ok(trial) => match copy_trial(
                        trial,
                        &self.layout,
                        &mut output.cells
                            [index * self.layout.cells..(index + 1) * self.layout.cells],
                    ) {
                        Ok(()) => (
                            PointOutcome::Succeeded,
                            trial
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
                    .checked_mul(size_of::<f64>())?,
            )
        })
        .and_then(|n| n.checked_mul(scratch_rows))
        .ok_or(MathError::Limit("packed batch scratch extent"))?;
    Ok(backing)
}

pub(in crate::execution) fn run(
    numeric: &mut NumericFrame,
    scratch: &mut Scratch,
    inputs: &[f64],
    rows: usize,
    order: DerivativeOrder,
    cancel: &Arc<AtomicBool>,
    mut receive: impl FnMut(usize, Result<&TrialEvaluation, MathError>),
) -> Result<(), MathError> {
    let mut canceled = false;
    let template = numeric.template.clone();
    let program = template.programs[order]
        .as_ref()
        .ok_or_else(|| MathError::Contract("packed batch order".into()))?;
    check_stages(program)?;
    numeric.prepare_order(order)?;
    let scope = numeric
        .scope
        .as_ref()
        .ok_or_else(|| MathError::Contract("detached packed batch".into()))?
        .clone();
    let layout = template.layouts[order]
        .as_ref()
        .ok_or_else(|| MathError::Contract("packed batch layout".into()))?;
    let entries = layout.frame_entries(template.slots)?;
    let scratch_rows = rows.min(SEGMENT);
    let backing = backing_bytes(&template, rows, order)?;
    let lane_allowance = backing
        .checked_add(template.order_worker_bytes[order])
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
        scratch.failures.clear();
        scratch.failures.resize_with(count, || None);
        for row in 0..count {
            resize(&mut scratch.frames[row], entries, f64::NAN)?;
            scratch.frames[row].fill(f64::NAN);
            scratch.trials[row].clear();
            let inputs =
                &inputs[(start + row) * template.inputs..(start + row + 1) * template.inputs];
            if inputs.iter().any(|value| !value.is_finite()) {
                scratch.failures[row] = Some(MathError::Contract(
                    "finite ordered formal inputs required".into(),
                ));
            } else {
                seed_frame(&template, &mut scratch.frames[row], layout, inputs, None);
            }
        }
        let stages = &mut numeric.programs[order]
            .as_mut()
            .ok_or_else(|| MathError::Contract("packed worker order".into()))?
            .stages;
        for stage in stages {
            scope.check().map_err(crate::error::scope_error)?;
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
                block.executable.evaluate_batch(
                    active,
                    &scratch.arguments,
                    &mut scratch.values,
                    &scope,
                    lane_allowance,
                )?;
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
        for row in 0..count {
            let index = start + row;
            let trial = &mut scratch.trials[row];
            if cancel.load(Ordering::Acquire) || canceled {
                canceled = true;
                receive(index, Err(MathError::Cancelled));
            } else if let Some(error) = scratch.failures[row].take() {
                receive(index, Err(error));
            } else {
                let result = publish_trial(&template, &scratch.frames[row], layout, trial);
                receive(index, result.map(|()| &*trial));
            }
            trial.clear();
            scratch.frames[row].fill(f64::NAN);
        }
    }
    Ok(())
}
