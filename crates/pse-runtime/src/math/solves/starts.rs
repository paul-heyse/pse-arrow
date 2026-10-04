// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original compiled-model screening of common physical start proposals.
use super::*;
use crate::math::prediction::{Proposal, Screened};
use native::NlpOracle;
use pse_model::strategy::BranchPolicy;
use std::sync::atomic::{AtomicBool, Ordering};

/// Abandonment stops the enclosing task; ordinary deadline exhaustion never changes its flag.
struct AbandonedTask(Option<Arc<AtomicBool>>);
impl Drop for AbandonedTask {
    fn drop(&mut self) {
        if let Some(flag) = &self.0 {
            flag.store(true, Ordering::Release);
        }
    }
}
impl PreparedSolve {
    /// Bind a declared fidelity proposal to this original target for common screening.
    /// Statistical infill evidence never supplies numerical accuracy or result permission.
    /// # Errors
    /// Changed target, fidelity, task, physical inventory or required connected branch.
    pub fn surrogate_start(
        &self,
        point: &pse_math::surrogate::SurrogateProposal,
        correspondence: &pse_math::surrogate::FidelityCorrespondence,
        task: pse_ids::ContentHash,
        branch: BranchPolicy,
    ) -> Result<Proposal, ProblemError> {
        let source = self.semantic_point_key(&point.coordinates)?;
        Proposal::surrogate(
            point,
            correspondence,
            task,
            source,
            self.original_identity()?,
            branch,
        )
    }
    /// Consume an original-screened physical proposal as an explicit primal-only start.
    /// Attach starts before composing a strategy so its admitted rungs remain immutable.
    /// # Errors
    /// Wrong original target, composed request or incompatible native coordinate inventory.
    pub fn with_screened_start(self, start: &Screened) -> Result<Self, ProblemError> {
        if self.composition.is_some() || start.proposal().target() != self.original_identity()? {
            return Err(ProblemError::Contract(
                "screened start must bind the original target before strategy composition".into(),
            ));
        }
        self.with_primal_start(start.proposal().values().collect())
    }
}
impl MathService {
    /// The same original screening operation on an already admitted strategy worker.
    pub(crate) fn screen_start_worker(
        &self,
        target: &PreparedSolve,
        proposal: Proposal,
        branch: BranchPolicy,
        scope: pse_kernels::ExecutionScope,
        budget: &Arc<WorkerBudget>,
    ) -> Result<Screened, MathRuntimeError> {
        self.screen_start_worker_observed(target, proposal, branch, scope, budget, &mut 0)
    }
    /// Count actual attempted original callback invocations, including partial failures.
    pub(crate) fn screen_start_worker_observed(
        &self,
        target: &PreparedSolve,
        proposal: Proposal,
        branch: BranchPolicy,
        scope: pse_kernels::ExecutionScope,
        budget: &Arc<WorkerBudget>,
        evaluations: &mut u64,
    ) -> Result<Screened, MathRuntimeError> {
        scope.check().map_err(ProblemError::Provider)?;
        let Representation::Algebraic(case) = &target.representation else {
            return Err(ProblemError::Unsupported(
                "original proposal screening requires compiled algebraic source".into(),
            )
            .into());
        };
        let executable = case
            .case
            .clone()
            .ok_or_else(|| ProblemError::Contract("original screening evaluator missing".into()))?;
        let rows = case.prepared.prepared.plan.structure().rows().len();
        let _scratch = budget.charge(
            rows.checked_mul(size_of::<f64>())
                .ok_or(MathRuntimeError::Limit("original screening rows"))?,
        )?;
        let ExecutionWorker {
            worker,
            _case,
            _charge,
        } = self.worker(executable, &case.providers, scope.clone(), budget)?;
        let mut oracle = native::assembled::AlgebraicOracle::new(worker, case.values.clone())?;
        let contract = NlpOracle::contract(&oracle).clone();
        let execution =
            Execution::within(scope.cancellation().clone(), &Controls::default(), scope)?;
        Ok(proposal.screen(
            &contract,
            target.original_identity()?,
            branch,
            &execution,
            |x| {
                *evaluations = evaluations
                    .checked_add(1)
                    .ok_or_else(|| ProblemError::memory("original screening evaluation counter"))?;
                let objective = oracle.objective(x)?;
                *evaluations = evaluations
                    .checked_add(1)
                    .ok_or_else(|| ProblemError::memory("original screening evaluation counter"))?;
                let mut values = vec![0.; rows];
                oracle.constraints(x, &mut values)?;
                if !objective.is_finite() || values.iter().any(|v| !v.is_finite()) {
                    return Err(ProblemError::numerical(
                        "nonfinite original start evaluation",
                    ));
                }
                Ok(())
            },
        )?)
    }

    /// Screen actual original bounds, guards, objective and rows on one admitted worker.
    /// Infeasible equations remain lawful for a start; no result permission is produced.
    /// # Errors
    /// A proposal has wrong meaning, a source guard fails, or the original scope/resources stop.
    pub async fn screen_start(
        self: &Arc<Self>,
        target: PreparedSolve,
        proposal: Proposal,
        branch: BranchPolicy,
        scope: pse_kernels::ExecutionScope,
        driver: &crate::CancelSource,
    ) -> Result<Screened, MathRuntimeError> {
        scope.check().map_err(ProblemError::Provider)?;
        let Representation::Algebraic(case) = &target.representation else {
            return Err(ProblemError::Unsupported(
                "compiled original start screening requires an algebraic source".into(),
            )
            .into());
        };
        let executable = case.case.as_ref().ok_or_else(|| {
            ProblemError::Contract("no original compiled evaluator for proposal screening".into())
        })?;
        let worker_bytes = executable.assembly.numeric_worker_bytes();
        let rows = case.prepared.prepared.plan.structure().rows().len();
        let result_bytes = proposal
            .values()
            .count()
            .checked_mul(size_of::<f64>() + size_of::<pse_ids::SemanticId>())
            .and_then(|n| n.checked_add(size_of::<Screened>()))
            .ok_or(MathRuntimeError::Limit("screened start extent"))?;
        let bytes = worker_bytes
            .checked_add(result_bytes)
            .and_then(|n| n.checked_add(rows.checked_mul(size_of::<f64>())?))
            .ok_or(MathRuntimeError::Limit("screening worker extent"))?;
        let budget = WorkerBudget::new(bytes);
        let control = FlightCancellation::default();
        let service = self.clone();
        let worker_scope = scope.clone();
        let mut abandoned = AbandonedTask(Some(scope.cancellation().clone()));
        let operation =
            self.job_retained_scoped(1, bytes, control.clone(), scope.deadline(), move |_| {
                let screened = service.screen_start_worker(
                    &target,
                    proposal,
                    branch,
                    worker_scope,
                    &budget,
                )?;
                let retained = screened.retained_bytes()?;
                Ok((screened, retained))
            });
        tokio::pin!(operation);
        let result = tokio::select! {
            result=&mut operation=>result,
            ()=driver.cancelled()=>{
                scope.cancellation().store(true,Ordering::Release);control.cancel();
                let _=operation.await;Err(MathRuntimeError::Cancelled)
            }
        };
        abandoned.0 = None;
        result.map(|(screened, owner)| screened.with_owner(owner))
    }
}
