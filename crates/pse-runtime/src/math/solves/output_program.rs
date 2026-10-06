// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected source outputs share the existing compiler, evaluator and arithmetic owners.
use super::*;
use pse_ids::{ContentHash, SemanticId};
use std::collections::BTreeMap;

#[derive(Debug)]
pub(crate) struct SelectedOutputProgram {
    pub(crate) executable: Arc<ExecutableCase>,
    pub(crate) rows: BTreeMap<SemanticId, SemanticId>,
    pub(crate) identity: ContentHash,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl SelectedOutputProgram {
    pub(crate) fn new(
        service: &MathService,
        executable: Arc<ExecutableCase>,
        rows: BTreeMap<SemanticId, SemanticId>,
    ) -> Result<Arc<Self>, MathRuntimeError> {
        let mut identity = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
        identity
            .str("selected-authored-output-first")
            .hash(&executable.assembly.structure().key());
        for (target, row) in &rows {
            identity.id(target).id(row);
        }
        for coordinate in executable.assembly.columns() {
            identity.id(coordinate);
        }
        let bytes = rows
            .len()
            .checked_mul(128)
            .and_then(|bytes| bytes.checked_add(size_of::<Self>()))
            .ok_or_else(|| ProblemError::memory("selected output inventory extent"))?;
        let owner = service.reserve("math:selected-output-program", bytes)?;
        Ok(Arc::new(Self {
            executable,
            rows,
            identity: identity.finish_hash(),
            _owner: owner,
        }))
    }
}
#[derive(Debug)]
pub(crate) struct EvaluatedOutput {
    pub(crate) value: f64,
    pub(crate) gradient: Vec<f64>,
    pub(crate) gradient_uncertainty: Vec<f64>,
    pub(crate) uncertainty: f64,
}
pub(crate) struct EvaluatedOutputs {
    pub(crate) outputs: BTreeMap<SemanticId, EvaluatedOutput>,
    pub(crate) projection: ContentHash,
    pub(crate) witness: ContentHash,
    _owner: super::super::jobs::WorkerCharge,
}
impl PreparedSolve {
    pub(crate) fn with_selected_outputs(mut self, program: Arc<SelectedOutputProgram>) -> Self {
        self.selected_outputs = Some(program);
        self
    }
    pub(crate) fn selected_output_program(&self) -> Option<&SelectedOutputProgram> {
        self.selected_outputs.as_deref()
    }
    /// Call only after original candidate checks; work is scoped to requested source rows.
    pub(crate) fn evaluated_observable_outputs(
        &self,
        service: &MathService,
        values: &CaseValues,
        scope: &pse_kernels::ExecutionScope,
        budget: &Arc<WorkerBudget>,
        execution: &Execution,
    ) -> Result<Option<EvaluatedOutputs>, ProblemError> {
        let Some(selected) = &self.selected_outputs else {
            return Ok(None);
        };
        let plan = &selected.executable.assembly;
        let n = plan.columns().len();
        let count = plan.structure().rows().len();
        let entries = count
            .checked_mul(n)
            .ok_or_else(|| ProblemError::memory("selected output gradient extent"))?;
        let bytes = entries
            .checked_mul(3)
            .and_then(|extent| extent.checked_add(count.checked_mul(2)?))
            .and_then(|extent| extent.checked_add(n))
            .and_then(|n| n.checked_mul(size_of::<f64>()))
            .and_then(|n| n.checked_add(count.checked_mul(256)?))
            .ok_or_else(|| ProblemError::memory("selected output extent"))?;
        let owner = budget
            .charge(bytes)
            .map_err(MathRuntimeError::into_problem)?;
        let point = plan
            .columns()
            .iter()
            .map(|id| values.scalars.get(id).copied())
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                ProblemError::Contract("selected output point coordinate missing".into())
            })?;
        let mut worker = service
            .case_worker(
                Some(selected.executable.clone()),
                BTreeMap::new(),
                scope,
                budget,
            )
            .map_err(MathRuntimeError::into_problem)?;
        let output_values = budget
            .evaluate(|| {
                worker
                    .worker()
                    .constraints(values)
                    .map_err(MathRuntimeError::from)
            })
            .map_err(MathRuntimeError::into_problem)?;
        let jacobian = budget
            .evaluate(|| {
                worker
                    .worker()
                    .jacobian(values)
                    .cloned()
                    .map_err(MathRuntimeError::from)
            })
            .map_err(MathRuntimeError::into_problem)?;
        let Some(arithmetic) = engineering_accuracy::arithmetic::selected_point(
            plan,
            values,
            &point,
            &output_values,
            &jacobian,
            execution,
            budget,
        )?
        else {
            return Ok(None);
        };
        let mut gradients = vec![vec![0.; point.len()]; count];
        for (column, _) in point.iter().enumerate() {
            for entry in jacobian.symbolic().col_range(column) {
                gradients[jacobian.symbolic().row_idx()[entry]][column] = jacobian.val()[entry];
            }
        }
        let mut outputs = BTreeMap::new();
        for (target, row) in &selected.rows {
            let index = plan
                .structure()
                .rows()
                .iter()
                .position(|output| output.id == *row)
                .ok_or_else(|| {
                    ProblemError::Internal("selected output inventory changed".into())
                })?;
            outputs.insert(
                *target,
                EvaluatedOutput {
                    value: output_values[index],
                    gradient: std::mem::take(&mut gradients[index]),
                    gradient_uncertainty: arithmetic.jacobian
                        [index * point.len()..(index + 1) * point.len()]
                        .to_vec(),
                    uncertainty: arithmetic.residual[index],
                },
            );
        }
        Ok(Some(EvaluatedOutputs {
            outputs,
            projection: selected.identity,
            witness: arithmetic.witness,
            _owner: owner,
        }))
    }
}
