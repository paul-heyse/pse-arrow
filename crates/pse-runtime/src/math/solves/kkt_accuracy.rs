// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One workflow point owns a bounded pending KKT output operation until original checks.
use super::*;
mod arithmetic;
use native::engineering_accuracy::{DeferredKktAccuracy, KktOutputObserver};
#[cfg(feature = "solver-root-isolation")]
use native::engineering_accuracy::{KktOutput, KktOutputs, KktValidity};
use pse_math::engineering_accuracy::GoalResult;
use pse_model::{
    engineering_accuracy::BoundGoal,
    generated::enums::{
        AccuracyGoalSubject, AccuracyObservation, AccuracyUnavailableReason as U, NumericalTarget,
    },
    strategy::SemanticProductKey,
};
use std::sync::Mutex;

fn same_frozen_context(expected: SemanticProductKey, mut actual: SemanticProductKey) -> bool {
    actual.point = expected.point;
    actual.accuracy = expected.accuracy;
    actual == expected
}

#[derive(Debug)]
struct Pending {
    artifact: DeferredKktAccuracy,
    _charge: super::super::jobs::WorkerCharge,
}
#[derive(Debug)]
pub(super) struct PointAccuracySink {
    /// Frozen scientific/numerical context; the actual point and work demand are separate.
    source: SemanticProductKey,
    numerics: Arc<ResolvedNumericalPolicy>,
    pending: Mutex<Option<Pending>>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl PointAccuracySink {
    fn state(&self) -> Result<std::sync::MutexGuard<'_, Option<Pending>>, ProblemError> {
        self.pending
            .lock()
            .map_err(|_| ProblemError::Internal("point accuracy owner panicked".into()))
    }
    fn compatible(&self, source: SemanticProductKey) -> bool {
        same_frozen_context(self.source, source)
    }
    fn take(&self) -> Result<Option<Pending>, ProblemError> {
        Ok(self.state()?.take())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hash(value: u64) -> pse_ids::ContentHash {
        let mut hash = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
        hash.u64(value);
        hash.finish_hash()
    }
    #[test]
    fn engineering_accuracy_kkt_pending_context_freezes_scientific_binding_and_allows_actual_work_point()
     {
        let frozen = SemanticProductKey {
            structure: hash(1),
            binding: hash(2),
            numerical_policy: Some(hash(3)),
            normalization: Some(hash(4)),
            point: None,
            parameters: Some(hash(5)),
            derivation: None,
            branch: None,
            accuracy: Some(hash(6)),
        };
        let mut actual = frozen;
        actual.point = Some(hash(7));
        actual.accuracy = Some(hash(8));
        assert!(same_frozen_context(frozen, actual));
        let mut changed = actual;
        changed.binding = hash(9);
        assert!(!same_frozen_context(frozen, changed));
        changed = actual;
        changed.numerical_policy = Some(hash(9));
        assert!(!same_frozen_context(frozen, changed));
        changed = actual;
        changed.parameters = Some(hash(9));
        assert!(!same_frozen_context(frozen, changed));
        changed = actual;
        changed.branch = Some(hash(9));
        assert!(!same_frozen_context(frozen, changed));
        changed = actual;
        changed.normalization = Some(hash(9));
        assert!(!same_frozen_context(frozen, changed));
    }
}
#[derive(Debug)]
struct Observer {
    source: SemanticProductKey,
    goals: Vec<BoundGoal>,
    sink: Arc<PointAccuracySink>,
    budget: Arc<WorkerBudget>,
    _charge: super::super::jobs::WorkerCharge,
}
impl KktOutputObserver for Observer {
    fn source(&self) -> SemanticProductKey {
        self.source
    }
    fn goals(&self) -> &[BoundGoal] {
        &self.goals
    }
    fn defer(
        &mut self,
        artifact: DeferredKktAccuracy,
        execution: &Execution,
    ) -> Result<(), ProblemError> {
        execution.check()?;
        if !self.sink.compatible(artifact.source()) {
            return Err(ProblemError::Contract(
                "pending KKT factor changes frozen point context".into(),
            ));
        }
        // A point has one pending artifact; replacement releases its old charge
        // before admitting the newly produced factor and request metadata.
        self.sink.take()?;
        let charge = self
            .budget
            .charge(artifact.bytes())
            .map_err(MathRuntimeError::into_problem)?;
        *self.sink.state()? = Some(Pending {
            artifact,
            _charge: charge,
        });
        Ok(())
    }
}

impl PreparedSolve {
    /// Install a fresh point-local owner before any strategy/native work. Clones
    /// within this same point share it; another workflow point installs a new one.
    pub(crate) fn with_point_accuracy(
        mut self,
        service: &MathService,
    ) -> Result<Self, ProblemError> {
        self.point_accuracy = None;
        if self.numerics.policy.goals.is_empty() {
            return Ok(self);
        }
        let Representation::Algebraic(case) = &self.representation else {
            return Ok(self);
        };
        let point = case
            .prepared
            .compiled()
            .plan
            .columns()
            .iter()
            .map(|id| case.values.scalars.get(id).copied())
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| ProblemError::Contract("accuracy anchor coordinate missing".into()))?;
        let mut source = self.semantic_point_key(&point)?;
        source.point = None;
        let owner = service
            .reserve(
                "math:point-accuracy-owner",
                size_of::<PointAccuracySink>().saturating_add(128),
            )
            .map_err(MathRuntimeError::into_problem)?;
        self.point_accuracy = Some(Arc::new(PointAccuracySink {
            source,
            numerics: self.numerics.clone(),
            pending: Mutex::new(None),
            _owner: owner,
        }));
        Ok(self)
    }
    /// Carry the same ephemeral owner through a serial strategy's admitted profile.
    /// Frozen scientific binding and goal declarations are checked on dispatch.
    pub(crate) fn inherit_point_accuracy(&mut self, original: &PreparedSolve) {
        self.point_accuracy = original.point_accuracy.clone();
        self.selected_outputs = original.selected_outputs.clone();
    }
    /// Drop pending factor storage on an original refusal or before another attempt.
    pub(crate) fn clear_point_accuracy(&self) -> Result<(), ProblemError> {
        if let Some(sink) = &self.point_accuracy {
            sink.take()?;
        }
        Ok(())
    }
    pub(super) fn kkt_observer(
        &self,
        budget: &Arc<WorkerBudget>,
    ) -> Result<Option<Box<dyn KktOutputObserver>>, MathRuntimeError> {
        let Some(sink) = &self.point_accuracy else {
            return Ok(None);
        };
        if self.profile.intent != SolveIntent::Optimize
            || self.backend().is_none_or(|backend| {
                execution::adapter(backend).representation() != execution::Representation::Nlp
            })
        {
            return Ok(None);
        }
        let Representation::Algebraic(case) = &self.representation else {
            return Ok(None);
        };
        if !case.providers.is_empty() {
            return Ok(None);
        }
        let plan = &case.prepared.compiled().plan;
        let point = plan
            .columns()
            .iter()
            .map(|id| case.values.scalars.get(id).copied())
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                ProblemError::Contract("KKT accuracy start coordinate missing".into())
            })?;
        let mut source = self.semantic_point_key(&point)?;
        source.point = None;
        source.accuracy = self.work_precision.or(Some(self.numerics.key));
        if !sink.compatible(source) || self.numerics.policy.goals != sink.numerics.policy.goals {
            return Err(ProblemError::Contract(
                "KKT goal dispatch changes frozen numerical context or goal declarations".into(),
            )
            .into());
        }
        let selected = self.selected_output_program();
        let supported = self.numerics.policy.goals.iter().filter(|goal| {
            goal.subject == AccuracyGoalSubject::SelectedOutput
                && goal.observation == AccuracyObservation::Steady
                && ((goal.target_kind == NumericalTarget::Variable
                    && plan.columns().contains(&goal.target_id))
                    || (goal.target_kind == NumericalTarget::Observable
                        && selected
                            .is_some_and(|program| program.rows.contains_key(&goal.target_id))))
        });
        let bytes = self
            .numerics
            .policy
            .goals
            .iter()
            .try_fold(size_of::<Observer>().saturating_add(128), |bytes, goal| {
                bytes
                    .checked_add(size_of::<BoundGoal>())?
                    .checked_add(goal.provenance.len())
            })
            .ok_or_else(|| ProblemError::memory("KKT goal observer metadata extent"))?;
        let charge = budget.charge(bytes)?;
        let goals = supported
            .map(|goal| {
                let mut product = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
                product
                    .str("direct-original-kkt-output")
                    .id(&goal.target_id)
                    .hash(&source.binding)
                    .hash(&source.accuracy.unwrap_or(self.numerics.key));
                if goal.target_kind == NumericalTarget::Observable
                    && let Some(selected) = selected
                    && let Some(row) = selected.rows.get(&goal.target_id)
                {
                    product.hash(&selected.identity).id(row);
                }
                let mut units = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
                units
                    .str("physical-output-error")
                    .id(&goal.quantity_id)
                    .id(&goal.unit_id);
                BoundGoal {
                    declaration: goal.clone(),
                    source,
                    product: product.finish_hash(),
                    normalization: units.finish_hash(),
                }
            })
            .collect::<Vec<_>>();
        if goals.is_empty() {
            return Ok(None);
        }
        Ok(Some(Box::new(Observer {
            source,
            goals,
            sink: sink.clone(),
            budget: budget.clone(),
            _charge: charge,
        })))
    }

    /// Consume the actual native step factor only after the caller's completed
    /// original-model admission. Direct coordinate projection adds no opaque evaluator.
    pub(super) fn kkt_coordinate_accuracy(
        &self,
        _service: &MathService,
        outcome: &Outcome,
        values: &CaseValues,
        scope: &pse_kernels::ExecutionScope,
        budget: &Arc<WorkerBudget>,
    ) -> Result<Option<Vec<GoalResult>>, ProblemError> {
        let pending = self
            .point_accuracy
            .as_ref()
            .map(|sink| sink.take())
            .transpose()?
            .flatten();
        let Outcome::Native(report) = outcome else {
            return Ok(None);
        };
        let Some(pending) = pending else {
            if let Some(withheld) = &report.evidence.output_accuracy {
                if let Some(error) = &withheld.failure {
                    return Err(ProblemError::Math(pse_math::MathError::Typed {
                        retained: error.retained_bytes(),
                        cause: pse_model::diagnostic::DiagnosticCause::from_shared(error.clone()),
                    }));
                }
                let mut results = withheld
                    .goals
                    .iter()
                    .zip(&withheld.outputs)
                    .map(|(goal, output)| match output {
                        Ok(evidence) => GoalResult::assess(goal, Some(evidence.clone())),
                        Err(reason) => GoalResult::unavailable(goal.declaration.clone(), *reason),
                    })
                    .collect::<Vec<_>>();
                for goal in &self.numerics.policy.goals {
                    if !results
                        .iter()
                        .any(|result| result.goal.goal_id == goal.goal_id)
                    {
                        results.push(GoalResult::unavailable(
                            goal.clone(),
                            U::UnsupportedObservation,
                        ));
                    }
                }
                return Ok(Some(results));
            }
            return Ok(None);
        };
        let Some(sink) = &self.point_accuracy else {
            return Ok(None);
        };
        let source = pending.artifact.source();
        let Representation::Algebraic(case) = &self.representation else {
            return Err(ProblemError::Contract(
                "pending KKT factor lacks original algebraic case".into(),
            ));
        };
        let plan = &case.prepared.compiled().plan;
        let point = plan
            .columns()
            .iter()
            .map(|id| values.scalars.get(id).copied())
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| ProblemError::Contract("KKT output coordinate missing".into()))?;
        if !sink.compatible(source)
            || report
                .candidate
                .as_ref()
                .is_none_or(|candidate| candidate.primal != point)
            || source.point != Some(native::square_response::point_key(&point))
            || pending
                .artifact
                .goals()
                .iter()
                .any(|goal| !self.numerics.policy.goals.contains(&goal.declaration))
        {
            return Ok(Some(
                self.numerics
                    .policy
                    .goals
                    .iter()
                    .cloned()
                    .map(|goal| GoalResult::unavailable(goal, U::InvalidValidity))
                    .collect(),
            ));
        }
        scope.check().map_err(ProblemError::from)?;
        let mut execution = Execution::within(
            scope.cancellation().clone(),
            &self.profile.controls,
            scope.clone(),
        )?;
        execution.work_admission = budget
            .admission()
            .map(|owner| -> Arc<dyn WorkAdmission> { owner });
        let arithmetic =
            arithmetic::original_point(plan, values, &point, source, &execution, budget)?;
        #[cfg(not(feature = "solver-root-isolation"))]
        {
            let arithmetic::PointArithmetic::Unavailable(reason) = arithmetic;
            let mut results = bound_results(pending.artifact.goals(), reason);
            append_unobserved_goals(&mut results, &self.numerics.policy.goals);
            Ok(Some(results))
        }
        #[cfg(feature = "solver-root-isolation")]
        {
            let count = pending.artifact.goals().len();
            let point_proof = match arithmetic {
                arithmetic::PointArithmetic::Enclosed(proof) => proof,
                arithmetic::PointArithmetic::Unavailable(reason) => {
                    let mut results = bound_results(pending.artifact.goals(), reason);
                    append_unobserved_goals(&mut results, &self.numerics.policy.goals);
                    return Ok(Some(results));
                }
            };
            let observable_requested = pending
                .artifact
                .goals()
                .iter()
                .any(|goal| goal.declaration.target_kind == NumericalTarget::Observable);
            let observable = if observable_requested {
                match self
                    .evaluated_observable_outputs(_service, values, scope, budget, &execution)?
                {
                    Some(outputs) => Some(outputs),
                    None => {
                        let mut results =
                            bound_results(pending.artifact.goals(), U::EvaluatorUncertainty);
                        append_unobserved_goals(&mut results, &self.numerics.policy.goals);
                        return Ok(Some(results));
                    }
                }
            } else {
                None
            };
            let gradient_bytes = count
                .checked_mul(point.len())
                .and_then(|extent| extent.checked_mul(size_of::<f64>()));
            let dimension = point
                .len()
                .checked_mul(2)
                .and_then(|extent| extent.checked_add(plan.structure().rows().len()));
            let action_bytes = dimension
                .and_then(|extent| extent.checked_mul(8 * size_of::<f64>()))
                .and_then(|bytes| {
                    dimension?
                        .checked_mul(dimension?)?
                        .checked_mul(size_of::<f64>())?
                        .checked_add(bytes)
                });
            let metadata = pending
                .artifact
                .goals()
                .iter()
                .try_fold(0usize, |bytes, goal| {
                    bytes
                        .checked_add(
                            size_of::<KktOutput>()
                                + size_of::<GoalResult>()
                                + size_of::<BoundGoal>()
                                + 1024,
                        )?
                        .checked_add(goal.declaration.provenance.capacity().checked_mul(3)?)
                });
            let bytes = gradient_bytes
                .and_then(|extent| extent.checked_add(action_bytes?))
                .and_then(|extent| extent.checked_add(metadata?))
                .ok_or_else(|| ProblemError::memory("KKT coordinate output extent"))?;
            let _output_charge = budget
                .charge(bytes)
                .map_err(MathRuntimeError::into_problem)?;
            let bound = pending.artifact.goals().to_vec();
            // Full original checks already ran in Assessment. This witness identifies
            // that admitted original candidate and the completed direct projection.
            let mut witness = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
            witness
                .str("admitted-original-stationary-coordinate-output")
                .hash(&source.structure)
                .hash(&source.binding)
                .hash(&self.normalization.key())
                .hash(&native::square_response::point_key(&point))
                .hash(&point_proof.proof.projection);
            if let Some(observable) = &observable {
                witness
                    .hash(&observable.projection)
                    .hash(&observable.witness);
            }
            let validity = KktValidity {
                source,
                witness: witness.finish_hash(),
            };
            let outputs = bound
                .iter()
                .map(|goal| {
                    let (value, gradient, gradient_uncertainty, uncertainty) =
                        match goal.declaration.target_kind {
                            NumericalTarget::Variable => {
                                let index =
                                    plan.columns()
                                        .iter()
                                        .position(|id| *id == goal.declaration.target_id)
                                        .ok_or_else(|| {
                                            ProblemError::Contract(
                                    "KKT selected coordinate outside original inventory".into(),
                                )
                                        })?;
                                let value = point[index];
                                let spacing = [value.next_up(), value.next_down()]
                                    .into_iter()
                                    .filter(|neighbor| neighbor.is_finite())
                                    .map(|neighbor| (neighbor - value).abs())
                                    .fold(0., f64::max);
                                let mut gradient = vec![0.; point.len()];
                                gradient[index] = 1.;
                                (value, gradient, vec![0.; point.len()], spacing)
                            }
                            NumericalTarget::Observable => {
                                let output = observable
                                    .as_ref()
                                    .and_then(|outputs| {
                                        outputs.outputs.get(&goal.declaration.target_id)
                                    })
                                    .ok_or_else(|| {
                                        ProblemError::Unsupported(
                                            "selected KKT observable was not evaluated".into(),
                                        )
                                    })?;
                                (
                                    output.value,
                                    output.gradient.clone(),
                                    output.gradient_uncertainty.clone(),
                                    output.uncertainty,
                                )
                            }
                            _ => {
                                return Err(ProblemError::Unsupported(
                                    "KKT output target is not a selected variable or observable"
                                        .into(),
                                ));
                            }
                        };
                    Ok(KktOutput {
                        goal: goal.clone(),
                        derivative_source: source,
                        value,
                        gradient,
                        gradient_uncertainty: Some(gradient_uncertainty),
                        uncertainty: Some(uncertainty),
                    })
                })
                .collect::<Result<Vec<_>, ProblemError>>()?;
            let evidence = pending.artifact.estimate(
                report,
                KktOutputs {
                    validity: Some(validity),
                    arithmetic: Some(point_proof.proof),
                    outputs,
                },
                &execution,
            );
            let failure = match evidence.failure {
                Some(error) if engineering_accuracy::optional_failure(&error) => {
                    Some(engineering_accuracy::retained_failure(error, budget)?)
                }
                Some(error) => {
                    return Err(ProblemError::Math(pse_math::MathError::Typed {
                        retained: error.retained_bytes(),
                        cause: pse_model::diagnostic::DiagnosticCause::from_shared(error),
                    }));
                }
                None => None,
            };
            let mut results = bound
                .into_iter()
                .zip(evidence.outputs)
                .map(|(goal, output)| match output {
                    Ok(evidence) => GoalResult::assess(&goal, Some(evidence)),
                    Err(reason) => GoalResult::unavailable(goal.declaration, reason),
                })
                .collect::<Vec<_>>();
            for result in &mut results {
                if result.classification.status
                    == pse_model::generated::enums::AccuracyGoalStatus::Unresolved
                {
                    result.failure = failure.clone();
                }
            }
            for goal in &self.numerics.policy.goals {
                if !results
                    .iter()
                    .any(|result| result.goal.goal_id == goal.goal_id)
                {
                    results.push(GoalResult::unavailable(
                        goal.clone(),
                        U::UnsupportedObservation,
                    ));
                }
            }
            Ok(Some(results))
        }
    }
}

fn bound_results(goals: &[BoundGoal], reason: U) -> Vec<GoalResult> {
    goals
        .iter()
        .map(|goal| GoalResult::unavailable(goal.declaration.clone(), reason))
        .collect()
}

fn append_unobserved_goals(
    results: &mut Vec<GoalResult>,
    goals: &[pse_model::engineering_accuracy::AccuracyGoal],
) {
    for goal in goals {
        if !results
            .iter()
            .any(|result| result.goal.goal_id == goal.goal_id)
        {
            results.push(GoalResult::unavailable(
                goal.clone(),
                U::UnsupportedObservation,
            ));
        }
    }
}
