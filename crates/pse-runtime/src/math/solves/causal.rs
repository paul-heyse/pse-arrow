// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Workflow-issued causal realization, bound lazily and assessed in the original case.
use super::*;
use pse_kernels::ExecutionScope;
use pse_model::strategy::WorkObservation;
use std::pin::Pin;

/// An authored topology description; constructing it performs no optional preparation.
pub(crate) trait CausalSupplier: std::fmt::Debug + Send + Sync {
    fn key(&self) -> pse_ids::ContentHash;
    fn prepare<'a>(
        &'a self,
        original: PreparedSolve,
        scope: ExecutionScope,
        cancel: &'a crate::CancelSource,
    ) -> Pin<Box<dyn Future<Output = Result<PreparedCausal, MathRuntimeError>> + Send + 'a>>;
}

/// Actual native map completion and its independently reconstructed original state.
pub(crate) struct CausalResult {
    pub(crate) report: Box<SolveReport>,
    pub(crate) values: Option<CaseValues>,
    /// Inclusive source work; native component subtotals cannot establish this count.
    pub(crate) inclusive_work: WorkEvidence,
}

/// One prepared causal evaluator; execution stays on the admitted session worker.
pub(crate) trait CausalExecution: std::fmt::Debug + Send + Sync {
    fn execute(
        &self,
        service: &MathService,
        execution: Execution,
        budget: &Arc<WorkerBudget>,
        original_start: &[f64],
    ) -> Result<CausalResult, MathRuntimeError>;
}

/// A frozen authored realization and its complete original numerical target.
#[derive(Clone, Debug)]
pub struct PreparedCausal {
    original: Box<PreparedSolve>,
    scope: ExecutionScope,
    key: pse_ids::ContentHash,
    realization: Arc<dyn CausalExecution>,
}
impl PreparedCausal {
    pub(crate) fn new(
        original: PreparedSolve,
        scope: ExecutionScope,
        key: pse_ids::ContentHash,
        realization: Arc<dyn CausalExecution>,
    ) -> Self {
        Self {
            original: Box::new(original),
            scope,
            key,
            realization,
        }
    }
    /// Frozen complete original numerical target.
    pub fn original(&self) -> &PreparedSolve {
        &self.original
    }
    /// Actual authored realization, effective map profile and original profile identity.
    pub fn key(&self) -> pse_ids::ContentHash {
        self.key
    }
    /// Original task cancellation and absolute deadline.
    pub fn scope(&self) -> &ExecutionScope {
        &self.scope
    }
    /// The same linked root backend admitted by the original source.
    pub fn backend(&self) -> Backend {
        Backend::Kinsol
    }
    pub(crate) fn result_bytes(&self) -> Result<usize, MathRuntimeError> {
        self.original
            .result_bytes()?
            .checked_mul(2)
            .ok_or(MathRuntimeError::Limit("causal result extent"))
    }
}
impl MathService {
    pub(crate) fn causal_step(
        &self,
        prepared: &PreparedCausal,
        mut execution: Execution,
        retained: &mut Retained,
        budget: &Arc<WorkerBudget>,
        original_start: &[f64],
    ) -> Result<Outcome, super::super::strategy::EffectFailure> {
        // The native map is a component, never a report over original coordinates.
        let mut work = WorkEvidence::default();
        let mut failure_cause = None;
        let mut component_observation = None;
        retained.clear();
        let result = (|| -> Result<Outcome, MathRuntimeError> {
            execution.check()?;
            let scope = execution.scope()?;
            if !Arc::ptr_eq(scope.cancellation(), prepared.scope.cancellation())
                || scope.deadline().is_some_and(|deadline| {
                    prepared
                        .scope
                        .deadline()
                        .is_none_or(|outer| deadline > outer)
                })
            {
                return Err(ProblemError::Contract(
                    "causal task differs from prepared original scope".into(),
                )
                .into());
            }
            let Representation::Algebraic(source) = &prepared.original.representation else {
                return Err(ProblemError::Internal(
                    "causal original representation changed".into(),
                )
                .into());
            };
            let columns = source.prepared.prepared.plan.columns();
            if original_start.len() != columns.len()
                || original_start.iter().any(|value| !value.is_finite())
            {
                return Err(ProblemError::Contract(
                    "complete original causal start required".into(),
                )
                .into());
            }
            execution.work_admission = budget
                .admission()
                .map(|owner| -> Arc<dyn WorkAdmission> { owner });
            let result_owner = self.reserve("math:causal-results", prepared.result_bytes()?)?;
            let CausalResult {
                mut report,
                values,
                inclusive_work,
            } = prepared
                .realization
                .execute(self, execution.clone(), budget, original_start)?;
            work = inclusive_work;
            if !crate::workflow::numerics::native_use(&report, &prepared.original.numerics.policy)
                .permits_use()
                || values.is_none()
            {
                component_observation = Some(super::super::strategy::observe_native(&report));
                failure_cause = report
                    .shared_callback_failure()
                    .or_else(|| report.shared_validation_failure());
                return Err(ProblemError::Unsupported(
                    "causal component did not establish a complete usable original candidate"
                        .into(),
                )
                .into());
            }
            let values = values.ok_or_else(|| {
                ProblemError::Internal("checked causal reconstruction disappeared".into())
            })?;
            let _charge = budget.charge(
                values
                    .scalars
                    .len()
                    .checked_mul(96)
                    .ok_or(MathRuntimeError::Limit("causal original coordinates"))?,
            )?;
            if columns.iter().any(|id| {
                values
                    .scalars
                    .get(id)
                    .is_none_or(|value| !value.is_finite())
            }) {
                return Err(ProblemError::Contract(
                    "causal reconstruction lacks original coordinates".into(),
                )
                .into());
            }
            let mut original = source.clone();
            original.values = values.clone();
            let Outcome::Constant(mut evaluated) = self.constant(
                Representation::Algebraic(original),
                &prepared.original.tolerances,
                &scope,
                budget,
            )?
            else {
                return Err(ProblemError::Internal(
                    "original causal assessment unavailable".into(),
                )
                .into());
            };
            let bounds = source
                .prepared
                .prepared
                .plan
                .structure()
                .variables()
                .iter()
                .filter(|variable| !variable.fixed)
                .zip(&prepared.original.tolerances.variables)
                .map(|(variable, tolerance)| {
                    let value = values.scalars[&variable.port.id];
                    Violation {
                        id: variable.port.id,
                        physical: quality::interval(
                            value,
                            variable.lower.unwrap_or(f64::NEG_INFINITY),
                            variable.upper.unwrap_or(f64::INFINITY),
                        ),
                        tolerance: *tolerance,
                    }
                })
                .collect();
            evaluated.quality = Quality::new(evaluated.quality.rows, bounds, vec![])?;
            evaluated.coordinates = columns.iter().map(|id| (*id, values.scalars[id])).collect();
            let add = |a: Option<u64>, b: Option<u64>| a.zip(b).and_then(|(a, b)| a.checked_add(b));
            evaluated.work = WorkEvidence {
                evaluations: add(work.evaluations, evaluated.work.evaluations),
                iterations: add(work.iterations, evaluated.work.iterations),
                factorizations: add(work.factorizations, evaluated.work.factorizations),
                proof_steps: add(work.proof_steps, evaluated.work.proof_steps),
            };
            work = evaluated.work;
            report = Box::new((*report).with_owner(result_owner.clone()));
            evaluated.components = vec![report];
            evaluated.owner = Some(result_owner);
            execution.check()?;
            Ok(Outcome::Constant(evaluated))
        })();
        retained.clear();
        result.map_err(|error| {
            let cause = failure_cause.unwrap_or_else(|| Arc::new(error.into_problem()));
            let observed = WorkObservation {
                attempts: 1,
                evaluations: work.evaluations,
                iterations: work.iterations,
                factorizations: work.factorizations,
                proof_steps: work.proof_steps,
            };
            match component_observation {
                Some(observation) => {
                    super::super::strategy::EffectFailure::component(cause, observed, observation)
                }
                None => super::super::strategy::EffectFailure::observed(cause, observed),
            }
        })
    }
}
