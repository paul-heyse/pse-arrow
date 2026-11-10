// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-independent, operation-owned ordered numerical populations.
use super::*;
use crate::execution::transfer::CheckedExecution;

/// A current scoped numerical worker and its independently admitted reusable
/// batch scratch. Row results are borrowed only for the supplied callback.
pub struct CheckedBatchExecution {
    frame: NumericFrame,
    trial: TrialEvaluation,
    result: Evaluation,
    rows: usize,
    order: DerivativeOrder,
    #[cfg(feature = "compiled-evaluator")]
    scratch: packed::Scratch,
}

pub(super) fn diagnostic_backing(stages: &[CompiledStage]) -> Result<usize, MathError> {
    stages.iter().try_fold(0usize, |sum, stage| {
        let lineage = |value: &pse_model::diagnostic::ValidityLineage| {
            value
                .sets
                .capacity()
                .checked_add(value.members.capacity())
                .and_then(|n| n.checked_mul(size_of::<SemanticId>()))
                .and_then(|n| {
                    n.checked_add(value.variables.capacity().checked_mul(size_of::<u32>())?)
                })
                .and_then(|n| n.checked_add(size_of_val(value)))
                .ok_or(MathError::Limit("batch validity diagnostic"))
        };
        let bytes = match stage {
            CompiledStage::Require {
                lineage: Some(value),
                ..
            } => lineage(value)?,
            CompiledStage::Domain {
                lineage: value,
                stages,
                ..
            } => lineage(value)?
                .checked_add(diagnostic_backing(stages)?)
                .ok_or(MathError::Limit("batch domain diagnostic"))?,
            CompiledStage::Applicability { plan, stages, .. } => plan
                .retained_bytes()
                .checked_mul(2)
                .and_then(|n| n.checked_add(diagnostic_backing(stages).ok()?))
                .ok_or(MathError::Limit("batch applicability diagnostic"))?,
            CompiledStage::Branch {
                then, otherwise, ..
            } => diagnostic_backing(then)?
                .checked_add(diagnostic_backing(otherwise)?)
                .ok_or(MathError::Limit("batch branch diagnostic"))?,
            _ => 0,
        };
        sum.checked_add(bytes)
            .ok_or(MathError::Limit("batch diagnostic extent"))
    })
}
pub(crate) fn numerical_batch_backing(
    template: &ExecutionTemplate,
    rows: usize,
    order: DerivativeOrder,
) -> Result<usize, MathError> {
    let layout = template.layouts[order]
        .as_ref()
        .ok_or_else(|| MathError::Contract("batch order not compiled".into()))?;
    let coordinates = layout.coordinates.len();
    let width = 1usize
        .checked_add(if order >= DerivativeOrder::First {
            coordinates
        } else {
            0
        })
        .and_then(|n| {
            n.checked_add(if order >= DerivativeOrder::Second {
                coordinates.checked_mul(coordinates)?
            } else {
                0
            })
        })
        .ok_or(MathError::Limit("batch dense callback width"))?;
    let dense = template
        .outputs
        .len()
        .checked_mul(width)
        .and_then(|n| n.checked_mul(size_of::<f64>()))
        .ok_or(MathError::Limit("batch dense callback extent"))?;
    let stages = template.programs[order]
        .as_ref()
        .ok_or_else(|| MathError::Contract("batch program missing".into()))?;
    #[cfg(feature = "compiled-evaluator")]
    let (scratch, live_rows) = if packed::has_packed(stages) {
        (
            packed::backing_bytes(template, rows, order)?,
            packed::retained_rows(rows),
        )
    } else {
        (0, usize::from(rows > 0))
    };
    #[cfg(not(feature = "compiled-evaluator"))]
    let (scratch, live_rows) = (0, usize::from(rows > 0));
    let diagnostics = diagnostic_backing(stages)?
        .checked_add(size_of::<MathError>())
        .and_then(|n| n.checked_mul(live_rows))
        .ok_or(MathError::Limit("batch failed row extent"))?;
    let callback = template
        .evaluation_cache_bytes
        .checked_mul(2)
        .and_then(|n| n.checked_add(dense))
        .ok_or(MathError::Limit("batch callback overlap"))?;
    let numeric = template
        .scratch_bytes
        .checked_add(scratch)
        .and_then(|n| n.checked_add(callback))
        .ok_or(MathError::Limit("numerical batch scratch extent"))?;
    if numeric > template.limits.scratch_bytes {
        return Err(MathError::Limit("numerical batch scratch bytes"));
    }
    template
        .worker_bytes
        .checked_add(scratch)
        .and_then(|n| n.checked_add(diagnostics))
        .and_then(|n| n.checked_add(callback))
        .and_then(|n| n.checked_add(size_of::<CheckedBatchExecution>()))
        .ok_or(MathError::Limit("complete numerical batch backing"))
}

impl CheckedExecution {
    /// Attach a complete previously admitted batch allowance. The original
    /// worker retains that allocation owner and the current operation scope.
    pub fn into_batch(
        self,
        max_rows: usize,
        order: DerivativeOrder,
        backing_bytes: usize,
    ) -> Result<CheckedBatchExecution, MathError> {
        CheckedBatchExecution::from_frame(self.frame, max_rows, order, backing_bytes)
    }
}
impl std::fmt::Debug for CheckedBatchExecution {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("CheckedBatchExecution")
            .field("rows", &self.rows)
            .field("order", &self.order)
            .finish_non_exhaustive()
    }
}
impl CheckedBatchExecution {
    pub(super) fn from_frame(
        frame: NumericFrame,
        max_rows: usize,
        order: DerivativeOrder,
        backing_bytes: usize,
    ) -> Result<Self, MathError> {
        if numerical_batch_backing(&frame.template, max_rows, order)? > backing_bytes {
            return Err(MathError::Limit("prepaid numerical batch backing"));
        }
        frame.check_scope()?;
        Ok(Self {
            frame,
            rows: max_rows,
            order,
            trial: TrialEvaluation::default(),
            result: Evaluation {
                applicability: Vec::new(),
                values: Vec::new(),
                jacobian: Vec::new(),
                hessians: Vec::new(),
            },
            #[cfg(feature = "compiled-evaluator")]
            scratch: packed::Scratch::prepaid(backing_bytes),
        })
    }
    /// Execute ordered point-major compact inputs, yielding each original index
    /// once. Successful rows borrow reusable storage; failures move to the receiver.
    pub fn evaluate_observing(
        &mut self,
        rows: usize,
        inputs: &[f64],
        mut receive: impl FnMut(usize, Result<&Evaluation, MathError>),
    ) -> Result<(), MathError> {
        if rows > self.rows || rows.checked_mul(self.frame.template.inputs) != Some(inputs.len()) {
            return Err(MathError::Contract(
                "admitted numerical batch input extent".into(),
            ));
        }
        self.frame.check_scope()?;
        #[cfg(feature = "compiled-evaluator")]
        if self.frame.template.programs[self.order]
            .as_ref()
            .is_some_and(|program| packed::has_packed(program))
        {
            let cancel = self
                .frame
                .scope
                .as_ref()
                .ok_or_else(|| MathError::Contract("detached numerical batch".into()))?
                .cancellation()
                .clone();
            return packed::run(
                &mut self.frame,
                &mut self.scratch,
                packed::Request {
                    inputs,
                    rows,
                    order: self.order,
                    bindings: BatchBindings::SelectedPartials,
                },
                &cancel,
                |index, result| match result {
                    Ok(row) => match row.trial.write_evaluation(&mut self.result) {
                        Ok(()) => receive(index, Ok(&self.result)),
                        Err(error) => receive(index, Err(error)),
                    },
                    Err(error) => receive(index, Err(error)),
                },
            );
        }
        let stride = self.frame.template.inputs;
        let cancel = self
            .frame
            .scope
            .as_ref()
            .ok_or_else(|| MathError::Contract("detached numerical batch".into()))?
            .cancellation()
            .clone();
        let mut providers = ProviderBindings::new();
        for row in 0..rows {
            let result = self
                .frame
                .evaluate_trial_into(
                    &inputs[row * stride..(row + 1) * stride],
                    self.order,
                    &mut providers,
                    &cancel,
                    &mut self.trial,
                )
                .and_then(|()| self.trial.write_evaluation(&mut self.result));
            match result {
                Ok(()) => receive(row, Ok(&self.result)),
                Err(error) => receive(row, Err(error)),
            }
        }
        Ok(())
    }
}
