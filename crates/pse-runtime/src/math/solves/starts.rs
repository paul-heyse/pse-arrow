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
    pub(crate) fn within_admitted_task(
        mut self,
        scope: pse_kernels::ExecutionScope,
        admission: Arc<super::super::strategy::admission::TaskAdmission>,
    ) -> Result<Self, ProblemError> {
        if !admission.matches_scope(&scope) {
            return Err(ProblemError::Contract(
                "task admission and prepared operation scopes differ".into(),
            ));
        }
        self = self.within_task(scope)?;
        self.task_admission = Some(admission);
        Ok(self)
    }
    pub(crate) fn task_admission(
        &self,
    ) -> Option<Arc<super::super::strategy::admission::TaskAdmission>> {
        self.task_admission.clone()
    }
    pub(crate) fn has_screened_start(&self) -> bool {
        self.proposal_start.is_some()
    }
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
    /// Consume an original-screened physical proposal with its producer origin and policy.
    /// Attach starts before composing a strategy so its admitted rungs remain immutable.
    /// # Errors
    /// Wrong original target, composed request or incompatible native coordinate inventory.
    pub fn with_screened_start(self, start: &Screened) -> Result<Self, ProblemError> {
        self.bind_screened_start(start, false, None)
    }
    #[cfg(feature = "solver-pounce")]
    pub(crate) fn with_recovery_start(self, start: &Screened) -> Result<Self, ProblemError> {
        if !self
            .task_admission
            .as_ref()
            .is_some_and(|owner| owner.entry_dispatched() && owner.matches_scope(start.scope()))
        {
            return Err(ProblemError::Contract(
                "recovery proposal requires an actually dispatched task entry".into(),
            ));
        }
        self.bind_screened_start(start, true, None)
    }
    pub(crate) fn with_composed_recovery_start(
        self,
        start: &Screened,
        rules: &pse_model::strategy::StartRules,
        branch: BranchPolicy,
    ) -> Result<Self, ProblemError> {
        if !self
            .task_admission
            .as_ref()
            .is_some_and(|owner| owner.entry_dispatched() && owner.matches_scope(start.scope()))
            || !rules.permits_recovery(start.proposal().origin(), false)
        {
            return Err(ProblemError::Contract(
                "composed recovery requires actual entry and the admitted origin".into(),
            ));
        }
        self.bind_screened_start(start, true, Some((branch, &rules.recovery)))
    }
    fn bind_screened_start(
        mut self,
        start: &Screened,
        recovery: bool,
        admitted: Option<(BranchPolicy, &[pse_model::strategy::StartOrigin])>,
    ) -> Result<Self, ProblemError> {
        let proposal = start.proposal();
        if self.composition.is_some() || proposal.target() != self.original_identity()? {
            return Err(ProblemError::Contract(
                "screened start must bind the original target before strategy composition".into(),
            ));
        }
        start.scope().check().map_err(ProblemError::Provider)?;
        if !recovery
            && (self.profile.controls.start == StartPolicy::Explicit
                || self.explicit_start.is_some() && self.proposal_start.is_none())
        {
            return Err(ProblemError::Contract(
                "a screened recovery proposal cannot replace an incoming explicit start".into(),
            ));
        }
        let (branch, origins) = admitted.unwrap_or((
            self.profile.composition.branch,
            &self.profile.composition.recovery,
        ));
        if proposal.branch() != branch || !origins.contains(&proposal.origin()) {
            return Err(ProblemError::Contract(
                "screened proposal origin or branch is not admitted by the original request".into(),
            ));
        }
        if let Some(scope) = &self.task_scope {
            if !Arc::ptr_eq(scope.cancellation(), start.scope().cancellation())
                || scope.deadline() != start.scope().deadline()
            {
                return Err(ProblemError::Contract(
                    "screened proposal task differs from the original scope".into(),
                ));
            }
        }
        let compatibility = self.compatibility.clone().ok_or_else(|| {
            ProblemError::Contract("constant evaluation has no numerical proposal start".into())
        })?;
        let ids: Vec<_> = match &self.representation {
            Representation::Algebraic(source) => source.prepared.prepared.plan.columns().to_vec(),
            Representation::Conic { problem, .. } => problem
                .contract
                .variables
                .iter()
                .map(|variable| variable.id)
                .collect(),
        };
        let values = proposal.values().collect::<BTreeMap<_, _>>();
        if values.len() != ids.len()
            || ids
                .iter()
                .any(|id| values.get(id).is_none_or(|value| !value.is_finite()))
        {
            return Err(ProblemError::Contract(
                "screened start must preserve the exact original coordinate inventory".into(),
            ));
        }
        let payload = execution::adapter(compatibility.backend)
            .primal_start(ids.iter().map(|id| values[id]).collect())?;
        let seed = WarmStart {
            origin: None,
            compatibility,
            payload,
        };
        self.explicit_start = Some(seed);
        self.proposal_start = Some(start.clone());
        self.task_scope = Some(start.scope().clone());
        Ok(self)
    }
}
impl MathService {
    /// One admission owner for proposal production, original screening and the native step.
    pub(crate) fn admit_proposal_task(
        &self,
        mut target: PreparedSolve,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<PreparedSolve, ProblemError> {
        if target
            .task_admission
            .as_ref()
            .is_some_and(|owner| !owner.matches_scope(&scope))
        {
            return Err(ProblemError::Contract(
                "proposal admission cannot replace the original task scope".into(),
            ));
        }
        target = target.within_task(scope.clone())?;
        if target.task_admission.is_none() {
            let mut limits = target.numerical_strategy().limits;
            if let Some(requested) = target.composition_request().limits {
                limits = requested;
            } else if target.composition_request().policy
                == pse_model::strategy::CompositionPolicy::Auto
            {
                limits.attempts = target.automatic_attempt_capacity(scope.cancellation())?;
            }
            target.task_admission = Some(super::super::strategy::admission::TaskAdmission::new(
                limits,
                scope,
                Some(self.pool.clone()),
                target.declared_foreign_bytes() > 0,
            ));
        }
        Ok(target)
    }
    pub(crate) fn screen_auxiliary_start_worker_observed(
        &self,
        target: &PreparedSolve,
        proposal: &OriginalProposal,
        branch: BranchPolicy,
        scope: pse_kernels::ExecutionScope,
        budget: &Arc<WorkerBudget>,
        evaluations: &mut u64,
    ) -> Result<Screened, MathRuntimeError> {
        if proposal.original != target.original_identity()? || branch.connected.is_some() {
            return Err(ProblemError::Contract("auxiliary reconstruction must preserve original source and cannot establish connected transport".into()).into());
        }
        let mut source = target.semantic_point_key(&proposal.coordinates)?;
        source.derivation = Some(proposal.family);
        source.accuracy = proposal
            .reconstruction_accuracy
            .map(|accuracy| accuracy.product);
        let coordinates = target.original_coordinates()?;
        let bytes = size_of::<Screened>()
            .checked_add(
                coordinates
                    .capacity()
                    .checked_mul(size_of::<pse_ids::SemanticId>())
                    .ok_or_else(|| ProblemError::memory("auxiliary screened inventory extent"))?,
            )
            .and_then(|n| n.checked_add(proposal.coordinates.len().checked_mul(size_of::<f64>())?))
            .ok_or_else(|| ProblemError::memory("auxiliary screened start extent"))?;
        let owner = self.reserve("math:auxiliary-screened-start", bytes)?;
        let point = Proposal::auxiliary_path(
            coordinates,
            proposal.coordinates.clone(),
            source,
            proposal.original,
            branch,
        )?
        .with_owner(owner.clone());
        let screened =
            self.screen_start_worker_observed(target, point, branch, scope, budget, evaluations)?;
        Ok(screened.with_owner(owner))
    }
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
                let objective = budget
                    .evaluate(|| {
                        *evaluations = evaluations.checked_add(1).ok_or(
                            MathRuntimeError::Limit("original screening evaluation counter"),
                        )?;
                        oracle.objective(x).map_err(Into::into)
                    })
                    .map_err(MathRuntimeError::into_problem)?;
                let mut values = vec![0.; rows];
                budget
                    .evaluate(|| {
                        *evaluations = evaluations.checked_add(1).ok_or(
                            MathRuntimeError::Limit("original screening evaluation counter"),
                        )?;
                        oracle.constraints(x, &mut values).map_err(Into::into)
                    })
                    .map_err(MathRuntimeError::into_problem)?;
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
        if target
            .task_admission
            .as_ref()
            .is_some_and(|owner| !owner.matches_scope(&scope))
        {
            return Err(ProblemError::Contract(
                "original screening differs from the admitted task scope".into(),
            )
            .into());
        }
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
        let budget = target.task_admission.clone().map_or_else(
            || budget.clone(),
            |admission| budget.with_admission(admission),
        );
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
