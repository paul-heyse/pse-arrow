// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One synchronized task ledger. Native callbacks never hold its lock while executing.
use super::{Ledger, WorkCharge, WorkLimits, WorkObservation, add_work, check_limits};
use datafusion::execution::memory_pool::{MemoryConsumer, MemoryPool};
use pse_backend_native::{
    ProblemError,
    solve::{NativeStorageScope, WorkAdmission, WorkEvidence},
};
use std::sync::{Arc, Mutex};

pub(crate) struct TaskAdmission {
    state: Mutex<State>,
    scope: pse_kernels::ExecutionScope,
    pool: Option<Arc<dyn MemoryPool>>,
    strict_storage: bool,
}
struct State {
    ledger: Ledger,
    pending: WorkObservation,
    unreported: WorkObservation,
    storage: Vec<Arc<pse_columnar::AllocationLease>>,
    storage_allowance: Option<(Arc<pse_columnar::AllocationLease>, usize)>,
    local: Option<(WorkLimits, WorkObservation)>,
    hooked: bool,
    local_bound: Option<WorkObservation>,
    budget_base: WorkObservation,
    budget_local: WorkObservation,
}
impl std::fmt::Debug for TaskAdmission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskAdmission")
            .field("strict_storage", &self.strict_storage)
            .finish_non_exhaustive()
    }
}
fn zero() -> WorkObservation {
    WorkObservation {
        attempts: 0,
        evaluations: Some(0),
        iterations: Some(0),
        factorizations: Some(0),
        proof_steps: Some(0),
    }
}
fn observation(work: WorkEvidence) -> WorkObservation {
    WorkObservation {
        attempts: 0,
        evaluations: work.evaluations,
        iterations: work.iterations,
        factorizations: work.factorizations,
        proof_steps: work.proof_steps,
    }
}
impl TaskAdmission {
    pub(crate) fn new(
        limits: WorkLimits,
        scope: pse_kernels::ExecutionScope,
        pool: Option<Arc<dyn MemoryPool>>,
        strict_storage: bool,
    ) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State {
                ledger: Ledger::new(limits),
                pending: zero(),
                unreported: zero(),
                storage: Vec::new(),
                storage_allowance: None,
                local: None,
                hooked: false,
                local_bound: None,
                budget_base: zero(),
                budget_local: zero(),
            }),
            scope,
            pool,
            strict_storage,
        })
    }
    fn state(&self) -> Result<std::sync::MutexGuard<'_, State>, ProblemError> {
        self.state
            .lock()
            .map_err(|_| ProblemError::Internal("task work admission owner panicked".into()))
    }
    pub(crate) fn observation(&self) -> Result<WorkObservation, ProblemError> {
        Ok(self.state()?.ledger.observation())
    }
    pub(crate) fn reserve_attempt(&self) -> Result<(), ProblemError> {
        self.state()?.ledger.reserve_attempt()
    }
    /// Reuse the allocation owner's already acquired foreign reservation. Known
    /// linear storage consumes its capacity without charging the same bytes twice.
    pub(crate) fn retain_storage_allowance(
        &self,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> Result<(), ProblemError> {
        if owner.size() == 0 {
            return Ok(());
        }
        let mut state = self.state()?;
        state.storage.push(owner.clone());
        state.storage_allowance = Some((owner, 0));
        Ok(())
    }
    pub(crate) fn reserve(
        &self,
        local: WorkLimits,
        bound: Option<WorkObservation>,
        hooked: bool,
    ) -> Result<(), ProblemError> {
        let mut state = self.state()?;
        state.budget_base = state.ledger.effective();
        state.budget_local = zero();
        state.local = Some((local, zero()));
        state.hooked = hooked;
        state.local_bound = bound;
        if hooked {
            // The owner has established complete pre-operation coverage for every capped
            // counter; no synthetic whole-operation measurement is needed.
            Ok(())
        } else {
            state.ledger.reserve(local, bound)
        }
    }
    #[cfg(test)]
    pub(crate) fn charge(&self, charge: WorkCharge) -> Result<(), ProblemError> {
        self.complete_charge(charge).map(|_| ())
    }
    pub(crate) fn complete_charge(
        &self,
        mut charge: WorkCharge,
    ) -> Result<WorkCharge, ProblemError> {
        let mut state = self.state()?;
        // A complete hook owner can report even a failed operation whose native
        // report was never constructed. This is the observed inclusive count for
        // that covered counter, not an uncapped native subtotal or a reservation.
        if state.hooked {
            let local = state.local.map(|(limits, _)| limits);
            for (actual, hooked, global, local) in [
                (
                    &mut charge.observed.evaluations,
                    state.unreported.evaluations,
                    state.ledger.limits.evaluations,
                    local.and_then(|l| l.evaluations),
                ),
                (
                    &mut charge.observed.iterations,
                    state.unreported.iterations,
                    state.ledger.limits.iterations,
                    local.and_then(|l| l.iterations),
                ),
                (
                    &mut charge.observed.factorizations,
                    state.unreported.factorizations,
                    state.ledger.limits.factorizations,
                    local.and_then(|l| l.factorizations),
                ),
                (
                    &mut charge.observed.proof_steps,
                    state.unreported.proof_steps,
                    state.ledger.limits.proof_steps,
                    local.and_then(|l| l.proof_steps),
                ),
            ] {
                if actual.is_none() && (global.is_some() || local.is_some()) {
                    *actual = hooked;
                }
            }
        }
        // Native hooks already charged actual units. The report/assessment event retains
        // its inclusive observation; only its unhooked remainder enters the ledger.
        fn remainder(value: Option<u64>, hooked: &mut Option<u64>) -> Option<u64> {
            match (value, *hooked) {
                (Some(value), Some(observed)) => {
                    *hooked = Some(0);
                    Some(value.saturating_sub(observed))
                }
                (None, _) => {
                    *hooked = Some(0);
                    None
                }
                (value, None) => value,
            }
        }
        let mut emitted = charge;
        fn inclusive(value: Option<u64>, hooked: Option<u64>) -> Option<u64> {
            value.map(|n| n.max(hooked.unwrap_or(0)))
        }
        emitted.observed.evaluations =
            inclusive(charge.observed.evaluations, state.unreported.evaluations);
        emitted.observed.iterations =
            inclusive(charge.observed.iterations, state.unreported.iterations);
        emitted.observed.factorizations = inclusive(
            charge.observed.factorizations,
            state.unreported.factorizations,
        );
        emitted.observed.proof_steps =
            inclusive(charge.observed.proof_steps, state.unreported.proof_steps);
        charge.observed.evaluations = remainder(
            charge.observed.evaluations,
            &mut state.unreported.evaluations,
        );
        charge.observed.iterations =
            remainder(charge.observed.iterations, &mut state.unreported.iterations);
        charge.observed.factorizations = remainder(
            charge.observed.factorizations,
            &mut state.unreported.factorizations,
        );
        charge.observed.proof_steps = remainder(
            charge.observed.proof_steps,
            &mut state.unreported.proof_steps,
        );
        state.ledger.charge(charge)?;
        state.budget_local = state.local_effective(add_work(state.budget_local, charge.observed)?);
        if let Some((_, work)) = &mut state.local {
            *work = add_work(*work, charge.observed)?;
        }
        if let Some((limits, _)) = state.local {
            check_limits(limits, state.budget_local)?;
        }
        state.retain_effective_work()?;
        Ok(emitted)
    }
}
impl WorkAdmission for TaskAdmission {
    fn admit(&self, work: WorkEvidence) -> Result<(), ProblemError> {
        self.scope.check().map_err(ProblemError::Provider)?;
        let mut state = self.state()?;
        let pending = add_work(state.pending, observation(work))?;
        check_limits(
            state.ledger.limits,
            add_work(state.current_budget()?, pending)?,
        )?;
        if let Some((limits, _)) = state.local {
            check_limits(limits, add_work(state.budget_local, pending)?)?;
        }
        state.pending = pending;
        Ok(())
    }
    fn observe(&self, work: WorkEvidence) -> Result<(), ProblemError> {
        let mut state = self.state()?;
        let actual = observation(work);
        fn release(pending: &mut Option<u64>, actual: Option<u64>) -> Result<(), ProblemError> {
            match (*pending, actual) {
                (Some(p), Some(a)) => {
                    *pending = Some(p.checked_sub(a).ok_or_else(|| {
                        ProblemError::Contract("work observation exceeds its admission".into())
                    })?)
                }
                _ => {
                    return Err(ProblemError::Unsupported(
                        "native operation has unknown inclusive work".into(),
                    ));
                }
            }
            Ok(())
        }
        let mut pending = state.pending;
        release(&mut pending.evaluations, actual.evaluations)?;
        release(&mut pending.iterations, actual.iterations)?;
        release(&mut pending.factorizations, actual.factorizations)?;
        release(&mut pending.proof_steps, actual.proof_steps)?;
        state.pending = pending;
        state.ledger.total = add_work(state.ledger.total, actual)?;
        state.unreported = add_work(state.unreported, actual)?;
        state.budget_local = add_work(state.budget_local, actual)?;
        if let Some((_, work)) = &mut state.local {
            *work = add_work(*work, actual)?;
        }
        state.retain_effective_work()?;
        if let Some((limits, _)) = state.local {
            check_limits(limits, state.budget_local)?;
        }
        check_limits(state.ledger.limits, state.current_budget()?)
    }
    fn admit_storage(
        &self,
        scope: NativeStorageScope,
        owner: &str,
        known_bytes: usize,
        opaque: bool,
    ) -> Result<(), ProblemError> {
        self.scope.check().map_err(ProblemError::Provider)?;
        if scope == NativeStorageScope::Application {
            return Ok(());
        }
        if opaque && self.strict_storage {
            return Err(ProblemError::Unsupported(
                "strict linear storage requires a complete producer-owned bound".into(),
            ));
        }
        if !opaque && known_bytes > 0 {
            let mut state = self.state()?;
            if let Some((owner, used)) = &mut state.storage_allowance {
                let next = used
                    .checked_add(known_bytes)
                    .ok_or_else(|| ProblemError::memory("native linear storage extent"))?;
                if next > owner.size() {
                    return Err(ProblemError::memory(
                        "native linear storage exceeds its retained allowance",
                    ));
                }
                *used = next;
                return Ok(());
            }
            drop(state);
            let pool = self
                .pool
                .as_ref()
                .ok_or_else(|| ProblemError::memory("native storage has no allocation pool"))?;
            let reservation =
                MemoryConsumer::new(format!("math:native-linear:{owner}")).register(pool);
            reservation
                .try_grow(known_bytes)
                .map_err(|error| ProblemError::memory(error.to_string()))?;
            self.state()?
                .storage
                .push(pse_columnar::AllocationLease::new(reservation));
        }
        Ok(())
    }
}

impl State {
    fn local_effective(&self, work: WorkObservation) -> WorkObservation {
        self.local_bound.map_or(work, |bound| WorkObservation {
            attempts: work.attempts,
            evaluations: work.evaluations.or(bound.evaluations),
            iterations: work.iterations.or(bound.iterations),
            factorizations: work.factorizations.or(bound.factorizations),
            proof_steps: work.proof_steps.or(bound.proof_steps),
        })
    }
    fn current_budget(&self) -> Result<WorkObservation, ProblemError> {
        self.local.map_or(Ok(self.ledger.effective()), |_| {
            add_work(self.budget_base, self.budget_local)
        })
    }
    fn retain_effective_work(&mut self) -> Result<(), ProblemError> {
        // Keep observed totals unknown when appropriate, while retaining both an
        // earlier inclusive reservation and subsequent independently observed work.
        let effective = self.current_budget()?;
        for (actual, reserved, now) in [
            (
                self.ledger.total.evaluations,
                &mut self.ledger.reserved.evaluations,
                effective.evaluations,
            ),
            (
                self.ledger.total.iterations,
                &mut self.ledger.reserved.iterations,
                effective.iterations,
            ),
            (
                self.ledger.total.factorizations,
                &mut self.ledger.reserved.factorizations,
                effective.factorizations,
            ),
            (
                self.ledger.total.proof_steps,
                &mut self.ledger.reserved.proof_steps,
                effective.proof_steps,
            ),
        ] {
            if actual.is_none() {
                *reserved = now;
                self.ledger.has_reservation = true;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn unit() -> WorkEvidence {
        WorkEvidence {
            evaluations: Some(1),
            iterations: Some(0),
            factorizations: Some(0),
            proof_steps: Some(0),
        }
    }
    fn limits() -> WorkLimits {
        WorkLimits {
            attempts: 2,
            evaluations: Some(2),
            iterations: None,
            factorizations: None,
            proof_steps: None,
        }
    }
    fn task() -> Arc<TaskAdmission> {
        TaskAdmission::new(
            limits(),
            pse_kernels::ExecutionScope::new(Arc::default(), None),
            None,
            false,
        )
    }
    fn charge(evaluations: Option<u64>) -> WorkCharge {
        WorkCharge {
            phase: super::super::Phase::Native,
            scope: pse_model::strategy::Scope::Task,
            charging_owner: pse_ids::ContentHash::from_bytes([1; 32]),
            observed: WorkObservation {
                attempts: 1,
                evaluations,
                iterations: Some(0),
                factorizations: Some(0),
                proof_steps: Some(0),
            },
        }
    }
    #[test]
    fn hooks_refuse_before_work_and_charge_failed_operations_once() {
        let task = task();
        task.reserve(limits(), None, true).unwrap();
        task.admit(unit()).unwrap();
        task.observe(unit()).unwrap();
        task.admit(unit()).unwrap();
        task.observe(unit()).unwrap();
        assert!(task.admit(unit()).is_err());
        let recorded = task.complete_charge(charge(Some(2))).unwrap();
        assert_eq!(recorded.observed.evaluations, Some(2));
        assert_eq!(task.observation().unwrap().evaluations, Some(2));
        assert!(task.charge(charge(Some(2))).is_err());
    }
    #[test]
    fn unbounded_work_refuses_and_unknown_actual_retains_reservation() {
        let task = task();
        assert!(task.reserve(limits(), None, false).is_err());
        let mut bound = zero();
        bound.evaluations = Some(2);
        task.reserve(limits(), Some(bound), false).unwrap();
        task.charge(charge(None)).unwrap();
        assert_eq!(task.observation().unwrap().evaluations, None);
        assert!(task.reserve(limits(), Some(bound), false).is_err());
    }
    #[test]
    fn unknown_strict_linear_storage_and_unadmitted_observation_refuse() {
        let task = TaskAdmission::new(
            limits(),
            pse_kernels::ExecutionScope::new(Arc::default(), None),
            None,
            true,
        );
        assert!(
            task.admit_storage(NativeStorageScope::Linear, "factor", 8, true)
                .is_err()
        );
        assert!(task.observe(unit()).is_err());
    }
    #[test]
    fn native_validation_and_independent_assessment_share_the_cap() {
        let task = task();
        task.reserve(limits(), None, true).unwrap();
        task.admit(unit()).unwrap();
        task.observe(unit()).unwrap();
        let native = task.complete_charge(charge(Some(0))).unwrap();
        assert_eq!(native.observed.evaluations, Some(1));
        task.admit(unit()).unwrap();
        task.observe(unit()).unwrap();
        let mut assessment = charge(Some(1));
        assessment.phase = super::super::Phase::Assessment;
        assessment.charging_owner = pse_ids::ContentHash::from_bytes([2; 32]);
        assessment.observed.attempts = 0;
        task.complete_charge(assessment).unwrap();
        assert_eq!(task.observation().unwrap().evaluations, Some(2));
        assert!(task.admit(unit()).is_err());
    }
    #[test]
    fn complete_linear_storage_is_pool_owned_until_last_native_owner_drops() {
        use datafusion::execution::memory_pool::GreedyMemoryPool;
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(64));
        let task = TaskAdmission::new(
            limits(),
            pse_kernels::ExecutionScope::new(Arc::default(), None),
            Some(pool.clone()),
            true,
        );
        task.admit_storage(NativeStorageScope::Linear, "factor", 64, false)
            .unwrap();
        assert_eq!(pool.reserved(), 64);
        assert!(
            task.admit_storage(NativeStorageScope::Linear, "factor", 1, false)
                .is_err()
        );
        let native: Arc<dyn WorkAdmission> = task.clone();
        drop(task);
        assert_eq!(pool.reserved(), 64);
        drop(native);
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn preacquired_storage_is_not_reserved_again() {
        use datafusion::execution::memory_pool::GreedyMemoryPool;
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(64));
        let reservation = MemoryConsumer::new("foreign").register(&pool);
        reservation.try_grow(64).unwrap();
        let owner = pse_columnar::AllocationLease::new(reservation);
        let task = TaskAdmission::new(
            limits(),
            pse_kernels::ExecutionScope::new(Arc::default(), None),
            Some(pool.clone()),
            true,
        );
        task.retain_storage_allowance(owner.clone()).unwrap();
        task.admit_storage(NativeStorageScope::Linear, "factor", 64, false)
            .unwrap();
        assert_eq!(pool.reserved(), 64);
        drop(owner);
        drop(task);
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn unknown_completed_work_keeps_its_bound_and_later_hooks_consume_room() {
        let task = TaskAdmission::new(
            WorkLimits {
                evaluations: Some(4),
                ..limits()
            },
            pse_kernels::ExecutionScope::new(Arc::default(), None),
            None,
            false,
        );
        let mut bound = zero();
        bound.evaluations = Some(2);
        task.reserve(limits(), Some(bound), false).unwrap();
        task.complete_charge(charge(None)).unwrap();
        task.reserve(limits(), None, true).unwrap();
        task.admit(unit()).unwrap();
        task.observe(unit()).unwrap();
        task.admit(unit()).unwrap();
        task.observe(unit()).unwrap();
        assert!(task.admit(unit()).is_err());
        let mut next = charge(Some(2));
        next.charging_owner = pse_ids::ContentHash::from_bytes([3; 32]);
        task.complete_charge(next).unwrap();
        assert_eq!(task.observation().unwrap().evaluations, None);
        task.reserve(limits(), None, true).unwrap();
        assert!(task.admit(unit()).is_err());
    }
    #[test]
    fn invalid_observation_does_not_consume_an_admitted_unit() {
        let task = task();
        task.reserve(limits(), None, true).unwrap();
        task.admit(unit()).unwrap();
        let mut invalid = unit();
        invalid.factorizations = Some(1);
        assert!(task.observe(invalid).is_err());
        task.observe(unit()).unwrap();
        assert_eq!(task.observation().unwrap().evaluations, Some(1));
    }
}
