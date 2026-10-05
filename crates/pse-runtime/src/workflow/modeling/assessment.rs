// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original-model assessment of a step's candidate: prepared once per structure, then
//! evaluated on the step's own worker, so every step of a staged sequence is qualified the
//! same way without a nested job (A6).
use super::results::{AssessmentScope, CertifiedBound, assess_observations, assessment_units};
use super::*;
use crate::math::{ExecutableCase, WorkerBudget, solves::Outcome};
use pse_math::binding::CaseValues;
use pse_model::generated::identities::RunId;
use std::sync::Arc;

/// One assessed candidate: the complete values it implies, its original-model checks and
/// reports, and why checks could not be evaluated.
#[derive(Debug)]
pub(in crate::workflow) struct AssessedPoint {
    pub values: CaseValues,
    pub checks: Vec<ModelingCheck>,
    pub reports: Vec<ModelingReport>,
    pub error: Option<pse_model::diagnostic::BoundaryDiagnostic>,
    /// A candidate existed and was assessed; without one no check applies.
    pub complete: bool,
    pub required_closure: usize,
    pub owner: Arc<pse_columnar::AllocationLease>,
    pub work: Vec<pse_model::strategy::WorkCharge>,
}
impl AssessedPoint {
    /// Preserve the original scientific refusal instead of deriving it from native status.
    pub(in crate::workflow) fn conclusion(
        &self,
        completion: &crate::workflow::numerics::Completed,
    ) -> crate::math::strategy::OriginalConclusion {
        use crate::math::strategy::OriginalConclusion;
        if let Some(error) = &self.error {
            OriginalConclusion::Unavailable {
                cause: crate::math::strategy::target::assessment_failure(error),
            }
        } else if !self.complete {
            OriginalConclusion::Unavailable {
                cause: Arc::new(pse_backend_native::ProblemError::numerical(
                    "native trajectory produced no assessable original candidate",
                )),
            }
        } else if completion.permits_use() {
            OriginalConclusion::Satisfied
        } else {
            OriginalConclusion::Refused {
                cause: Arc::new(pse_backend_native::ProblemError::numerical(
                    completion.decision.reason(),
                )),
            }
        }
    }
    /// Compose original checks with native permission through the shared completion owner.
    pub(in crate::workflow) fn completion(
        &self,
        outcome: &Outcome,
        policy: &pse_model::numerics::NumericalPolicy,
    ) -> crate::workflow::numerics::Completed {
        crate::workflow::numerics::complete(
            outcome.candidate_use(policy),
            crate::workflow::numerics::CompletionEvidence::point(
                &self.checks,
                self.complete && self.error.is_none(),
                self.required_closure,
            ),
            policy,
        )
    }
}
/// Which original-model obligations a step answers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::workflow) enum Obligations {
    /// Intermediate specifications keep their model checks; final-fixture expectations
    /// apply only to the original specification.
    Intermediate,
    /// Every obligation, including final-fixture expectations.
    Final,
}
/// The prepared original-model assessment of one bound case.
#[derive(Debug)]
pub(in crate::workflow) struct Assessment {
    program: Option<Arc<ExecutableCase>>,
    rows: Vec<SemanticId>,
    scope: AssessmentScope,
    pub(in crate::workflow) required_closure: usize,
}
impl ModelingPackage {
    /// Prepare the assessment of `prepared` before its step runs. The observation program
    /// is value-independent and shared by every step of the same structure.
    pub(in crate::workflow) async fn assessment(
        &self,
        prepared: &ModelingSolvePreparation,
        obligations: Obligations,
        cancel: &crate::CancelSource,
    ) -> Result<Assessment, WorkflowError> {
        let product = prepared.model.model.compiled();
        let mut units = assessment_units(product);
        if obligations == Obligations::Intermediate {
            for expectation in product.model.expectations.values() {
                units.remove(&(expectation.id, expectation.lineage.declaration));
            }
        }
        let scope = units.keys().copied().collect();
        let rows = units
            .into_values()
            .flatten()
            .collect::<std::collections::BTreeSet<_>>();
        let program = if rows.is_empty() {
            None
        } else {
            Some(
                self.observation_program(&prepared.model.model, &rows, prepared.compiler, cancel)
                    .await?,
            )
        };
        let rows = program
            .as_ref()
            .map(|p| p.assembly.structure().rows().iter().map(|r| r.id).collect())
            .unwrap_or_default();
        Ok(Assessment {
            program,
            rows,
            scope,
            required_closure: product.model.required_closure_checks(),
        })
    }
}
impl Assessment {
    /// Assess `outcome` against the original model on the step's worker. The evaluator
    /// observes the step's stop flag and is charged to the session's worker share (F31).
    /// Without a candidate nothing is evaluated and no check applies.
    #[expect(
        clippy::too_many_arguments,
        reason = "the step supplies its identity, stop flag, worker share and result owner"
    )]
    pub(in crate::workflow) fn assess(
        &self,
        prepared: &ModelingSolvePreparation,
        run_id: RunId,
        attempt: usize,
        outcome: &Outcome,
        execution: Option<&pse_kernels::ExecutionScope>,
        budget: &Arc<WorkerBudget>,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> AssessedPoint {
        let mut point = AssessedPoint {
            values: prepared.model.values.clone(),
            checks: vec![],
            reports: vec![],
            error: None,
            complete: false,
            required_closure: self.required_closure,
            owner,
            work: Vec::new(),
        };
        match outcome {
            Outcome::Constant(report) => {
                point
                    .values
                    .scalars
                    .extend(report.coordinates.iter().copied());
            }
            Outcome::Native(native) => match &native.candidate {
                Some(candidate) if candidate.primal.len() == native.variables.len() => {
                    point.values.scalars.extend(
                        native
                            .variables
                            .iter()
                            .copied()
                            .zip(candidate.primal.iter().copied()),
                    );
                }
                Some(_) => {
                    point.error =
                        Some(contract("native candidate coordinate extent").boundary_diagnostic());
                    return point;
                }
                None => return point,
            },
            Outcome::Rejected(_) => return point,
        }
        let Some(execution) = execution else {
            point.error =
                Some(contract("candidate assessment has no execution scope").boundary_diagnostic());
            return point;
        };
        point.complete = true;
        let certified = CertifiedBound::of(outcome);
        let mut evaluations = 0_u64;
        match self.evaluate(
            prepared,
            run_id,
            attempt,
            &point.values,
            certified,
            execution,
            budget,
            &mut evaluations,
        ) {
            Ok((checks, reports)) => {
                point.checks = checks;
                point.reports = reports;
            }
            Err(error) => point.error = Some(error.boundary_diagnostic()),
        }
        let mut charging_owner = pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
        if let Ok(original) = prepared.solve.original_identity() {
            charging_owner.hash(&original);
        }
        charging_owner
            .str("original-model-assessment")
            .str(&run_id.to_string())
            .u64(attempt as u64);
        point.work.push(pse_model::strategy::WorkCharge {
            phase: pse_model::strategy::Phase::Assessment,
            scope: pse_model::strategy::Scope::Task,
            charging_owner: charging_owner.finish_hash(),
            observed: pse_model::strategy::WorkObservation {
                attempts: 0,
                evaluations: Some(evaluations),
                iterations: Some(0),
                factorizations: Some(0),
                proof_steps: Some(0),
            },
        });
        point
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "the step supplies its identity, point, certified bound, stop flag and worker share"
    )]
    fn evaluate(
        &self,
        prepared: &ModelingSolvePreparation,
        run_id: RunId,
        attempt: usize,
        values: &CaseValues,
        certified: Option<CertifiedBound>,
        execution: &pse_kernels::ExecutionScope,
        budget: &Arc<WorkerBudget>,
        evaluations: &mut u64,
    ) -> Result<(Vec<ModelingCheck>, Vec<ModelingReport>), WorkflowError> {
        let checkpoint = || {
            execution.check().map_err(|error| {
                crate::math::MathRuntimeError::from(pse_backend_native::ProblemError::Provider(
                    error,
                ))
            })
        };
        checkpoint()?;
        let mut applicability = Vec::new();
        let observed = if let Some(program) = &self.program {
            let mut evaluator = prepared.source.runtime.native().worker(
                program.clone(),
                &prepared.providers,
                execution.clone(),
                budget,
            )?;
            let observed = budget.evaluate(|| {
                *evaluations =
                    evaluations
                        .checked_add(1)
                        .ok_or(crate::math::MathRuntimeError::Limit(
                            "original assessment work counter overflow",
                        ))?;
                evaluator
                    .worker()
                    .constraints(values)
                    .map_err(crate::math::MathRuntimeError::from)
            })?;
            applicability = evaluator.worker().applicability_observations();
            self.rows.iter().copied().zip(observed).collect()
        } else {
            BTreeMap::new()
        };
        let (mut checks, mut reports) = assess_observations(
            run_id,
            prepared.model.model.compiled(),
            values,
            &observed,
            &applicability,
            prepared.solve.numerics(),
            &prepared.source.quantities,
            true,
            Some(&self.scope),
            certified,
        )?;
        for row in &mut checks {
            row.step = attempt as i64;
        }
        for row in &mut reports {
            row.step = attempt as i64;
        }
        checkpoint()?;
        Ok((checks, reports))
    }
}
