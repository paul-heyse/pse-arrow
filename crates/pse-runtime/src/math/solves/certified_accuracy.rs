// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored outputs consume the actual complete reconstruction enclosure.
use super::*;
use pse_math::engineering_accuracy::GoalResult;
use pse_model::generated::enums::{
    AccuracyGoalSubject, AccuracyObservation, AccuracyUnavailableReason as U, NumericalTarget,
};

impl PreparedSolve {
    pub(super) fn certified_observable_accuracy(
        &self,
        service: &MathService,
        receipt: &derived::CertifiedReconstructionPoint,
        values: &CaseValues,
        scope: &pse_kernels::ExecutionScope,
        budget: &Arc<WorkerBudget>,
        results: &mut [GoalResult],
    ) -> Result<(), ProblemError> {
        let eligible = |goal: &pse_model::engineering_accuracy::AccuracyGoal| {
            goal.subject == AccuracyGoalSubject::SelectedOutput
                && goal.observation == AccuracyObservation::Steady
                && goal.target_kind == NumericalTarget::Observable
        };
        if !results.iter().any(|result| eligible(&result.goal)) {
            return Ok(());
        }
        for result in results.iter_mut().filter(|result| eligible(&result.goal)) {
            result.classification.unavailable = Some(U::EvaluatorUncertainty);
        }
        #[cfg(feature = "solver-root-isolation")]
        {
            use native::root_isolation::{Ibex, PointArithmeticEvidence};
            use pse_kernels::DerivativeOrder;
            use pse_model::{
                engineering_accuracy::OutputEvidence,
                generated::enums::{AccuracyEvidenceInterpretation, AccuracyEvidenceMethod},
                strategy::{AccuracyClass, AccuracyEvidence},
            };
            let Some(selected) = self.selected_output_program() else {
                return Ok(());
            };
            let plan = &selected.executable.assembly;
            // Matching temporarily materializes the point (8 bytes/coordinate)
            // before cloning the two-sided enclosure (16 bytes/coordinate).
            let box_bytes = plan
                .columns()
                .len()
                .checked_mul(24)
                .ok_or_else(|| ProblemError::memory("certified output box extent"))?;
            let _box = budget
                .charge(box_bytes)
                .map_err(MathRuntimeError::into_problem)?;
            let Some(region) = receipt.coordinate_box(self, values)? else {
                return Ok(());
            };
            if plan.columns().len() != region.len() {
                return Err(ProblemError::Contract(
                    "certified output coordinate inventory".into(),
                ));
            }
            let mut execution = Execution::within(
                scope.cancellation().clone(),
                &self.profile.controls,
                scope.clone(),
            )?;
            execution.work_admission = budget
                .admission()
                .map(|owner| -> Arc<dyn WorkAdmission> { owner });
            execution.check()?;
            let available = budget.capacity().saturating_sub(budget.used());
            let construction = budget
                .charge(available)
                .map_err(MathRuntimeError::into_problem)?;
            let Some(program) = plan
                .point_arithmetic_program_for_order(
                    values,
                    DerivativeOrder::Value,
                    available / 256,
                    &execution.cancel,
                )
                .map_err(|error| match error {
                    pse_math::factorable::FactorableError::Math(error) => ProblemError::Math(error),
                    error => ProblemError::Unsupported(error.to_string()),
                })?
            else {
                return Ok(());
            };
            drop(construction);
            let _program = budget
                .charge(program.retained_bytes())
                .map_err(MathRuntimeError::into_problem)?;
            let bytes =
                Ibex.point_arithmetic_workspace_bytes_for_order(&program, DerivativeOrder::Value)?;
            let _workspace = budget
                .charge(bytes)
                .map_err(MathRuntimeError::into_problem)?;
            let _scalars = budget
                .charge(program.graph.residuals.len().saturating_mul(24))
                .map_err(MathRuntimeError::into_problem)?;
            let enclosed = Ibex.enclose_arithmetic_box_values_with_execution(
                &program, &region, bytes, &execution,
            )?;
            execution.check()?;
            let intervals = match enclosed {
                PointArithmeticEvidence::Enclosed { values, .. } => values,
                PointArithmeticEvidence::Incomplete { .. } => return Ok(()),
                PointArithmeticEvidence::Interrupted { .. } => return Err(ProblemError::Cancelled),
            };
            let mut worker = service
                .case_worker(
                    Some(selected.executable.clone()),
                    BTreeMap::new(),
                    scope,
                    budget,
                )
                .map_err(MathRuntimeError::into_problem)?;
            let actual = budget
                .evaluate(|| {
                    worker
                        .worker()
                        .constraints(values)
                        .map_err(MathRuntimeError::from)
                })
                .map_err(MathRuntimeError::into_problem)?;
            let mut validity = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
            validity
                .str("selected-output-over-certified-coordinate-box")
                .hash(&receipt.validity())
                .hash(&selected.identity)
                .hash(&program.key)
                .hash(&program.graph.identity());
            let validity = validity.finish_hash();
            for result in results.iter_mut().filter(|result| eligible(&result.goal)) {
                let Some(row) = selected
                    .rows
                    .get(&result.goal.target_id)
                    .and_then(|id| plan.structure().rows().iter().position(|row| row.id == *id))
                else {
                    continue;
                };
                let Some(interval) = program
                    .rows
                    .get(row)
                    .and_then(|row| intervals.get(row.value))
                else {
                    continue;
                };
                let Some(value) = actual.get(row).copied().filter(|value| value.is_finite()) else {
                    continue;
                };
                let radius = (value - interval.lower)
                    .abs()
                    .max((interval.upper - value).abs());
                let error = if radius == 0. { 0. } else { radius.next_up() };
                if !error.is_finite() {
                    continue;
                }
                let mut bound = receipt.bound_goal(&result.goal);
                let mut product = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
                product.hash(&bound.product).hash(&validity);
                bound.product = product.finish_hash();
                *result = GoalResult::assess(&bound, Some(OutputEvidence {
                    target: bound.declaration.target_id, target_kind: bound.declaration.target_kind,
                    quantity: bound.declaration.quantity_id, unit: bound.declaration.unit_id,
                    observation: bound.declaration.observation, time: bound.declaration.time,
                    value: Some(value), accuracy: AccuracyEvidence { product: bound.product,
                        normalization: bound.normalization, class: AccuracyClass::Certified, error: Some(error) },
                    source: receipt.source(), validity: Some(validity),
                    interpretation: AccuracyEvidenceInterpretation::OutputError,
                    method: AccuracyEvidenceMethod::CertifiedEnclosure,
                    interval: Some((interval.lower, interval.upper)),
                    limitation: "Authored interval evaluation over the actual complete selected-root enclosure; original checks and selection proof retained.".into(),
                }));
            }
        }
        #[cfg(not(feature = "solver-root-isolation"))]
        let _ = (service, receipt, values, scope, budget);
        Ok(())
    }
}
