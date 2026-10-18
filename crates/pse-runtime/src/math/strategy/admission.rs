// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One synchronized task ledger. Native callbacks never hold its lock while executing.
use super::{Ledger, WorkCharge, WorkLimits, WorkObservation, add_work, check_limits};
use datafusion::execution::memory_pool::{MemoryConsumer, MemoryPool};
use pse_backend_native::{
    ProblemError,
    solve::{NativeStorageScope, WorkAdmission, WorkEvidence},
};
use std::{collections::BTreeMap, sync::{Arc, Mutex}};

pub(crate) struct TaskAdmission {
    state: Mutex<State>,
    parent: Option<(Arc<TaskAdmission>, usize)>,
    scope: pse_kernels::ExecutionScope,
    pool: Option<Arc<dyn MemoryPool>>,
    pressure: Option<Arc<super::super::retention::Pressure>>,
    strict_storage: bool,
    entry_dispatched: std::sync::atomic::AtomicBool,
}
struct State {
    ledger: Ledger,
    pending: WorkObservation,
    unknown_pending: [u64; 4],
    unreported: WorkObservation,
    storage: Vec<Arc<pse_columnar::AllocationLease>>,
    storage_allowance: Option<(Arc<pse_columnar::AllocationLease>, usize)>,
    local: Option<(WorkLimits, WorkObservation)>,
    hooked: bool,
    local_bound: Option<WorkObservation>,
    budget_base: WorkObservation,
    budget_local: WorkObservation,
    next_child: usize,
    children: BTreeMap<usize, ChildWork>,
}
struct ChildWork { bound: Option<WorkObservation>, observed: WorkObservation, pending: WorkObservation }
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
        Self::new_with_pressure(limits, scope, pool, strict_storage, None)
    }
    pub(crate) fn new_for(
        limits: WorkLimits,
        scope: pse_kernels::ExecutionScope,
        service: &super::super::MathService,
        strict_storage: bool,
    ) -> Arc<Self> {
        Self::new_with_pressure(
            limits,
            scope,
            Some(service.pool.clone()),
            strict_storage,
            Some(service.pressure.clone()),
        )
    }
    fn new_with_pressure(
        limits: WorkLimits,
        scope: pse_kernels::ExecutionScope,
        pool: Option<Arc<dyn MemoryPool>>,
        strict_storage: bool,
        pressure: Option<Arc<super::super::retention::Pressure>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            parent: None,
            state: Mutex::new(State {
                ledger: Ledger::new(limits),
                pending: zero(),
                unknown_pending: [0; 4],
                unreported: zero(),
                storage: Vec::new(),
                storage_allowance: None,
                local: None,
                hooked: false,
                local_bound: None,
                budget_base: zero(),
                budget_local: zero(),
                next_child: 0,
                children: BTreeMap::new(),
            }),
            scope,
            pool,
            pressure,
            strict_storage,
            entry_dispatched: std::sync::atomic::AtomicBool::new(false),
        })
    }
    /// Private local strategy state, sharing only original aggregate permission and
    /// actual work. Concurrent children never reset another child's local ledger.
    pub(crate) fn child(self: &Arc<Self>) -> Result<Arc<Self>, ProblemError> {
        let index = { let mut state = self.state()?; let index = state.next_child;
            state.next_child = index.checked_add(1).ok_or_else(|| ProblemError::memory("task child extent"))?; index };
        let mut child = Self::new_with_pressure(self.limits()?, self.scope.clone(), self.pool.clone(), self.strict_storage, self.pressure.clone());
        Arc::get_mut(&mut child).ok_or_else(|| ProblemError::Internal("new task child escaped".into()))?.parent = Some((self.clone(), index));
        Ok(child)
    }
    fn begin_child(&self, index: usize, bound: Option<WorkObservation>, hooked: bool) -> Result<(), ProblemError> {
        self.scope.check().map_err(ProblemError::Provider)?;
        let mut state = self.state()?;
        if state.children.contains_key(&index) { return Err(ProblemError::Contract("task child already dispatched".into())); }
        let strict = [state.ledger.limits.evaluations, state.ledger.limits.iterations, state.ledger.limits.factorizations, state.ledger.limits.proof_steps].iter().any(Option::is_some);
        let bound = if hooked { None } else if strict {
            Some(bound.ok_or_else(|| ProblemError::Unsupported("aggregate child work requires complete coverage or inclusive bound".into()))?)
        } else { bound };
        let mut held = bound.unwrap_or_else(zero); held.attempts = 1;
        check_limits(state.ledger.limits, add_work(state.admission_budget()?, held)?)?;
        state.pending.attempts = state.pending.attempts.checked_add(1).ok_or_else(|| ProblemError::memory("task child attempts"))?;
        state.children.insert(index, ChildWork { bound: bound.map(|mut bound| { bound.attempts = 0; bound }), observed: zero(), pending: zero() });
        Ok(())
    }
    fn complete_child(&self, index: usize, charge: WorkCharge) -> Result<(), ProblemError> {
        let mut state = self.state()?;
        let child = state.children.remove(&index).ok_or_else(|| ProblemError::Contract("unissued child work charge".into()))?;
        state.pending.attempts = state.pending.attempts.checked_sub(1).ok_or_else(|| ProblemError::Contract("missing child attempt reservation".into()))?;
        let remaining = child.bound.map(|bound| subtract_known(bound, child.observed)).transpose()?.unwrap_or_else(zero);
        let effective = WorkObservation { attempts: charge.observed.attempts,
            evaluations: charge.observed.evaluations.or(remaining.evaluations),
            iterations: charge.observed.iterations.or(remaining.iterations),
            factorizations: charge.observed.factorizations.or(remaining.factorizations),
            proof_steps: charge.observed.proof_steps.or(remaining.proof_steps) };
        let upper = add_work(state.current_budget()?, effective)?;
        let ledger = &mut state.ledger;
        for (unknown, reserved, upper) in [
            (ledger.total.evaluations.is_none() || charge.observed.evaluations.is_none(), &mut ledger.reserved.evaluations, upper.evaluations),
            (ledger.total.iterations.is_none() || charge.observed.iterations.is_none(), &mut ledger.reserved.iterations, upper.iterations),
            (ledger.total.factorizations.is_none() || charge.observed.factorizations.is_none(), &mut ledger.reserved.factorizations, upper.factorizations),
            (ledger.total.proof_steps.is_none() || charge.observed.proof_steps.is_none(), &mut ledger.reserved.proof_steps, upper.proof_steps),
        ] { if unknown { *reserved = upper; } }
        if [charge.observed.evaluations, charge.observed.iterations, charge.observed.factorizations, charge.observed.proof_steps].iter().any(Option::is_none) { state.ledger.has_reservation = true; }
        state.ledger.charge(charge)?;
        // An enclosing report may include the complete child. Keep its remainder in
        // the original reconciliation owner, without stealing another child's hooks.
        state.unreported = add_work(state.unreported, charge.observed)?;
        state.budget_local = state.local_effective(add_work(state.budget_local, charge.observed)?);
        if let Some((_, work)) = &mut state.local { *work = add_work(*work, charge.observed)?; }
        state.retain_effective_work()?;
        check_limits(state.ledger.limits, state.admission_budget()?)
    }
    pub(crate) fn matches_scope(&self, scope: &pse_kernels::ExecutionScope) -> bool {
        Arc::ptr_eq(self.scope.cancellation(), scope.cancellation())
            && self.scope.deadline() == scope.deadline()
    }
    pub(crate) fn entry_dispatched(&self) -> bool {
        self.entry_dispatched
            .load(std::sync::atomic::Ordering::Acquire)
    }
    pub(crate) fn mark_entry_dispatched(&self) {
        self.entry_dispatched
            .store(true, std::sync::atomic::Ordering::Release);
    }
    fn state(&self) -> Result<std::sync::MutexGuard<'_, State>, ProblemError> {
        self.state
            .lock()
            .map_err(|_| ProblemError::Internal("task work admission owner panicked".into()))
    }
    pub(crate) fn observation(&self) -> Result<WorkObservation, ProblemError> {
        Ok(self.state()?.ledger.observation())
    }
    /// The existing parent grant remains authoritative for serial child operations.
    pub(crate) fn limits(&self) -> Result<WorkLimits, ProblemError> {
        Ok(self.state()?.ledger.limits)
    }
    pub(crate) fn reserve_attempt(&self) -> Result<(), ProblemError> {
        self.state()?.ledger.reserve_attempt()?;
        if let Some((parent, _)) = &self.parent {
            let state = parent.state()?;
            if state.admission_budget()?.attempts >= state.ledger.limits.attempts {
                return Err(ProblemError::Limit { kind: pse_backend_native::LimitKind::Work, detail: "aggregate child attempt allowance exhausted".into() });
            }
        }
        Ok(())
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
        if state
            .storage_allowance
            .as_ref()
            .is_some_and(|(held, _)| Arc::ptr_eq(held, &owner))
        {
            return Ok(());
        }
        // Native owners hold detached lease tokens for still-live prior storage.
        // The task keeps only the current allowance, rather than a per-step history.
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
        if !hooked { state.ledger.reserve(local, bound)?; }
        if let Some((parent, index)) = &self.parent { parent.begin_child(*index, bound, hooked)?; }
        Ok(())
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
        if let Some((parent, index)) = &self.parent { parent.complete_child(*index, charge)?; }
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
impl TaskAdmission {
    fn admit_inner(&self, work: WorkEvidence, child: Option<usize>) -> Result<(), ProblemError> {
        self.scope.check().map_err(ProblemError::Provider)?;
        let mut state = self.state()?;
        let actual = observation(work);
        let global = state.ledger.limits;
        let local = state.local.map(|(limits, _)| limits);
        let mut known = actual;
        let mut unknown = state.unknown_pending;
        for (index, (unit, global_cap, local_cap)) in [
            (
                &mut known.evaluations,
                global.evaluations,
                local.and_then(|limits| limits.evaluations),
            ),
            (
                &mut known.iterations,
                global.iterations,
                local.and_then(|limits| limits.iterations),
            ),
            (
                &mut known.factorizations,
                global.factorizations,
                local.and_then(|limits| limits.factorizations),
            ),
            (
                &mut known.proof_steps,
                global.proof_steps,
                local.and_then(|limits| limits.proof_steps),
            ),
        ]
        .into_iter()
        .enumerate()
        {
            if unit.is_none() {
                if global_cap.is_some() || local_cap.is_some() {
                    return Err(ProblemError::Unsupported(
                        "hard work allowance requires a known operation count before dispatch"
                            .into(),
                    ));
                }
                unknown[index] = unknown[index]
                    .checked_add(1)
                    .ok_or_else(|| ProblemError::memory("unknown work span counter"))?;
                *unit = Some(0);
            }
        }
        let pending = add_work(state.pending, known)?;
        let covered = child.and_then(|index| state.children.get(&index)).is_some_and(|work| work.bound.is_some());
        if let Some(index) = child {
            let issued = state.children.get(&index).ok_or_else(|| ProblemError::Contract("native child has no dispatch reservation".into()))?;
            if let Some(bound) = issued.bound {
                let next = add_work(issued.observed, add_work(issued.pending, known)?)?;
                check_limits(WorkLimits { attempts: u64::MAX, evaluations: bound.evaluations, iterations: bound.iterations, factorizations: bound.factorizations, proof_steps: bound.proof_steps }, next)?;
            }
        }
        check_limits(global, add_work(state.admission_budget()?, if covered { zero() } else { known })?)?;
        if let Some((limits, _)) = state.local {
            check_limits(limits, add_work(state.budget_local, pending)?)?;
        }
        if let Some((parent, index)) = &self.parent { parent.admit_inner(work, Some(*index))?; }
        if let Some(index) = child { let issued = state.children.get_mut(&index).ok_or_else(|| ProblemError::Contract("missing admitted child".into()))?; issued.pending = add_work(issued.pending, known)?; }
        state.pending = pending;
        state.unknown_pending = unknown;
        Ok(())
    }
    fn observe_inner(&self, work: WorkEvidence, child: Option<usize>) -> Result<(), ProblemError> {
        let mut state = self.state()?;
        let actual = observation(work);
        fn release(
            pending: &mut Option<u64>,
            unknown: &mut u64,
            actual: Option<u64>,
            capped: bool,
        ) -> Result<(), ProblemError> {
            match (*pending, actual) {
                (Some(p), Some(a)) => {
                    *pending = Some(p.checked_sub(a).ok_or_else(|| {
                        ProblemError::Contract("work observation exceeds its admission".into())
                    })?)
                }
                (_, None) if !capped => {
                    *unknown = unknown.checked_sub(1).ok_or_else(|| {
                        ProblemError::Contract(
                            "unknown work observation has no admitted span".into(),
                        )
                    })?
                }
                _ => {
                    return Err(ProblemError::Unsupported(
                        "native operation has unknown inclusive work under a hard cap".into(),
                    ));
                }
            }
            Ok(())
        }
        let mut pending = state.pending;
        let mut unknown = state.unknown_pending;
        let global = state.ledger.limits;
        let local = state.local.map(|(limits, _)| limits);
        release(
            &mut pending.evaluations,
            &mut unknown[0],
            actual.evaluations,
            global.evaluations.is_some()
                || local.is_some_and(|limits| limits.evaluations.is_some()),
        )?;
        release(
            &mut pending.iterations,
            &mut unknown[1],
            actual.iterations,
            global.iterations.is_some() || local.is_some_and(|limits| limits.iterations.is_some()),
        )?;
        release(
            &mut pending.factorizations,
            &mut unknown[2],
            actual.factorizations,
            global.factorizations.is_some()
                || local.is_some_and(|limits| limits.factorizations.is_some()),
        )?;
        release(
            &mut pending.proof_steps,
            &mut unknown[3],
            actual.proof_steps,
            global.proof_steps.is_some()
                || local.is_some_and(|limits| limits.proof_steps.is_some()),
        )?;
        if let Some((parent, index)) = &self.parent { parent.observe_inner(work, Some(*index))?; }
        if let Some(index) = child {
            let issued = state.children.get_mut(&index).ok_or_else(|| ProblemError::Contract("missing observed child".into()))?;
            issued.pending = subtract_known(issued.pending, actual)?;
            issued.observed = add_work(issued.observed, actual)?;
        }
        state.pending = pending;
        state.unknown_pending = unknown;
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
}
impl WorkAdmission for TaskAdmission {
    fn retain_storage(&self) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        let mut state = self.state.lock().ok()?;
        // Transfer these actual reservations to the native owner. The task must not
        // keep a second historical ownership list after the native owner takes them.
        let mut leases = std::mem::take(&mut state.storage);
        if let Some((owner, _)) = &state.storage_allowance
            && !leases.iter().any(|held| Arc::ptr_eq(held, owner))
        {
            leases.push(owner.clone());
        }
        Some(Arc::new(leases))
    }
    fn reserve_retained_storage(
        &self,
        owner: &str,
        known_bytes: usize,
    ) -> Result<Option<Arc<dyn std::any::Any + Send + Sync>>, ProblemError> {
        self.scope.check().map_err(ProblemError::Provider)?;
        let Some(pool) = &self.pool else {
            return Ok(None);
        };
        let reservation =
            MemoryConsumer::new(format!("math:native-retained:{owner}")).register(pool);
        let result = match &self.pressure {
            Some(pressure) => pressure.try_grow(&reservation, known_bytes),
            None => reservation.try_grow(known_bytes),
        };
        // Optional retention never blocks useful full-update work under pressure.
        // Cancellation/deadline retains its typed meaning even if capacity also failed.
        self.scope.check().map_err(ProblemError::Provider)?;
        if result.is_err() {
            return Ok(None);
        }
        Ok(Some(pse_columnar::AllocationLease::new(reservation)))
    }
    fn admit(&self, work: WorkEvidence) -> Result<(), ProblemError> { self.admit_inner(work, None) }
    fn observe(&self, work: WorkEvidence) -> Result<(), ProblemError> { self.observe_inner(work, None) }
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
            match &self.pressure {
                Some(pressure) => pressure.try_grow(&reservation, known_bytes),
                None => reservation.try_grow(known_bytes),
            }
            .map_err(|error| ProblemError::memory(error.to_string()))?;
            self.state()?
                .storage
                .push(pse_columnar::AllocationLease::new(reservation));
        }
        Ok(())
    }
}

fn subtract_known(a: WorkObservation, b: WorkObservation) -> Result<WorkObservation, ProblemError> {
    let sub = |a: Option<u64>, b: Option<u64>| -> Result<Option<u64>, ProblemError> {
        match (a, b) { (Some(a), Some(b)) => a.checked_sub(b).map(Some).ok_or_else(|| ProblemError::Contract("child work exceeds its reservation".into())), (a, None) => Ok(a), (None, _) => Ok(None) }
    };
    Ok(WorkObservation { attempts: a.attempts.checked_sub(b.attempts).ok_or_else(|| ProblemError::Contract("child attempts exceed reservation".into()))?, evaluations: sub(a.evaluations, b.evaluations)?, iterations: sub(a.iterations, b.iterations)?, factorizations: sub(a.factorizations, b.factorizations)?, proof_steps: sub(a.proof_steps, b.proof_steps)? })
}

impl State {
    fn admission_budget(&self) -> Result<WorkObservation, ProblemError> {
        let mut held = zero();
        let mut covered_pending = zero();
        for child in self.children.values() {
            if let Some(bound) = child.bound {
                held = add_work(held, subtract_known(bound, child.observed)?)?;
                covered_pending = add_work(covered_pending, child.pending)?;
            }
        }
        add_work(self.current_budget()?, add_work(subtract_known(self.pending, covered_pending)?, held)?)
    }
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
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
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
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
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
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
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
    fn optional_retained_storage_grant_owns_only_allocation_and_falls_back_under_pressure() {
        use datafusion::execution::memory_pool::GreedyMemoryPool;
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(64));
        let cancel = pse_diagnostics::Cancellation::default();
        let task = TaskAdmission::new(
            limits(),
            pse_kernels::ExecutionScope::new(cancel.clone(), None),
            Some(pool.clone()),
            false,
        );
        let weak = Arc::downgrade(&task);
        let token = task
            .reserve_retained_storage("witness", 64)
            .unwrap()
            .unwrap();
        assert_eq!(pool.reserved(), 64);
        assert!(
            task.reserve_retained_storage("replacement", 1)
                .unwrap()
                .is_none()
        );
        cancel.cancel();
        assert!(matches!(
            task.reserve_retained_storage("cancelled", 1),
            Err(ProblemError::Provider(_))
        ));
        drop(task);
        assert!(weak.upgrade().is_none());
        assert_eq!(pool.reserved(), 64);
        drop(token);
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
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
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
    fn proposal_and_screen_work_reduce_later_native_room_on_the_same_owner() {
        let task = TaskAdmission::new(
            WorkLimits {
                evaluations: Some(3),
                ..limits()
            },
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
            None,
            false,
        );
        task.admit(unit()).unwrap();
        task.observe(unit()).unwrap();
        task.admit(unit()).unwrap();
        task.observe(unit()).unwrap();
        assert_eq!(task.observation().unwrap().evaluations, Some(2));
        task.reserve(limits(), None, true).unwrap();
        task.admit(unit()).unwrap();
        task.observe(unit()).unwrap();
        assert!(
            task.admit(unit()).is_err(),
            "native dispatch cannot obtain fresh counters after proposal screening"
        );
        task.complete_charge(charge(Some(1))).unwrap();
        assert_eq!(
            task.observation().unwrap().evaluations,
            Some(3),
            "inclusive hook reconciliation charges each actual evaluation once"
        );
    }
    #[test]
    fn detached_storage_token_releases_completed_ledger_and_replaces_old_allowance() {
        use datafusion::execution::memory_pool::GreedyMemoryPool;
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(128));
        let task = TaskAdmission::new(
            limits(),
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
            Some(pool.clone()),
            true,
        );
        let weak = Arc::downgrade(&task);
        let reserve = || {
            let reservation = MemoryConsumer::new("foreign").register(&pool);
            reservation.try_grow(64).unwrap();
            pse_columnar::AllocationLease::new(reservation)
        };
        task.retain_storage_allowance(reserve()).unwrap();
        let old = task.retain_storage().unwrap();
        task.retain_storage_allowance(reserve()).unwrap();
        assert_eq!(
            pool.reserved(),
            128,
            "old and replacement storage overlap before native retirement"
        );
        let current = task.retain_storage().unwrap();
        drop(old);
        assert_eq!(
            pool.reserved(),
            64,
            "completed allowance is not accumulated in task storage history"
        );
        drop(task);
        assert!(
            weak.upgrade().is_none(),
            "native storage token does not retain the task ledger"
        );
        assert_eq!(pool.reserved(), 64);
        drop(current);
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn uncapped_unknown_nested_iterations_remain_unknown_and_caps_refuse_before_work() {
        let task = TaskAdmission::new(
            limits(),
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
            None,
            false,
        );
        let unknown = WorkEvidence {
            evaluations: Some(0),
            iterations: None,
            factorizations: Some(0),
            proof_steps: Some(0),
        };
        let known = WorkEvidence {
            iterations: Some(1),
            ..unknown
        };
        task.admit(unknown).unwrap();
        task.admit(known).unwrap();
        task.observe(unknown).unwrap();
        task.observe(known).unwrap();
        task.admit(known).unwrap();
        task.observe(known).unwrap();
        assert_eq!(
            task.observation().unwrap().iterations,
            None,
            "unknown inclusive iterations never become observed zero"
        );
        let capped = TaskAdmission::new(
            WorkLimits {
                iterations: Some(10),
                ..limits()
            },
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
            None,
            false,
        );
        assert!(capped.admit(unknown).is_err());
        assert_eq!(capped.observation().unwrap().iterations, Some(0));
        task.reserve(
            WorkLimits {
                iterations: Some(10),
                ..limits()
            },
            None,
            true,
        )
        .unwrap();
        assert!(
            task.admit(unknown).is_err(),
            "local hard caps refuse before effect too"
        );
    }
    #[test]
    fn native_token_takes_individual_reservations_without_task_history() {
        use datafusion::execution::memory_pool::GreedyMemoryPool;
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(128));
        let task = TaskAdmission::new(
            limits(),
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
            Some(pool.clone()),
            false,
        );
        task.admit_storage(NativeStorageScope::Linear, "owner1", 64, false)
            .unwrap();
        let old = task.retain_storage().unwrap();
        task.admit_storage(NativeStorageScope::Linear, "owner2", 64, false)
            .unwrap();
        let current = task.retain_storage().unwrap();
        assert_eq!(pool.reserved(), 128);
        drop(old);
        assert_eq!(pool.reserved(), 64);
        drop(current);
        assert_eq!(
            pool.reserved(),
            0,
            "task retains no historical native storage owners"
        );
    }
    #[test]
    fn unknown_completed_work_keeps_its_bound_and_later_hooks_consume_room() {
        let task = TaskAdmission::new(
            WorkLimits {
                evaluations: Some(4),
                ..limits()
            },
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
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
    #[test]
    fn concurrent_children_have_private_local_reports_and_one_aggregate_cap() {
        let parent = task();
        let one = parent.child().unwrap(); let two = parent.child().unwrap();
        let mut local = limits(); local.evaluations = Some(1);
        one.reserve(local, None, true).unwrap(); two.reserve(limits(), None, true).unwrap();
        one.admit(unit()).unwrap(); two.admit(unit()).unwrap();
        assert!(two.admit(unit()).is_err(), "issued siblings cannot spend the same remaining global unit");
        one.observe(unit()).unwrap();
        let recorded = one.complete_charge(charge(Some(1))).unwrap();
        assert_eq!(recorded.observed.evaluations, Some(1));
        assert!(one.admit(unit()).is_err(), "another child's larger local cap cannot overwrite this child's cap");
        two.observe(unit()).unwrap();
        let mut second = charge(Some(1)); second.charging_owner = pse_ids::ContentHash::from_bytes([2; 32]);
        let recorded = two.complete_charge(second).unwrap();
        assert_eq!(recorded.observed.evaluations, Some(1));
        assert_eq!(parent.observation().unwrap().evaluations, Some(2));
        assert_eq!(parent.observation().unwrap().attempts, 2);
        let third = parent.child().unwrap(); assert!(third.reserve(limits(), None, true).is_err());
    }
    #[test]
    fn concurrent_child_bounds_release_known_surplus_but_never_renew_unknown_permission() {
        let mut caps = limits(); caps.attempts = 8; caps.evaluations = Some(10);
        let parent = TaskAdmission::new(caps, pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None), None, false);
        let bound = |n| WorkObservation { evaluations: Some(n), ..zero() };
        let one = parent.child().unwrap(); let two = parent.child().unwrap(); let three = parent.child().unwrap();
        one.reserve(caps, Some(bound(6)), false).unwrap();
        two.reserve(caps, Some(bound(4)), false).unwrap();
        assert!(three.reserve(caps, Some(bound(1)), false).is_err());
        one.complete_charge(charge(Some(2))).unwrap();
        three.reserve(caps, Some(bound(4)), false).unwrap();
        let mut unknown = charge(None); unknown.charging_owner = pse_ids::ContentHash::from_bytes([2; 32]);
        two.complete_charge(unknown).unwrap();
        let mut known = charge(Some(3)); known.charging_owner = pse_ids::ContentHash::from_bytes([3; 32]);
        three.complete_charge(known).unwrap();
        assert_eq!(parent.observation().unwrap().evaluations, None);
        let four = parent.child().unwrap();
        assert!(four.reserve(caps, Some(bound(2)), false).is_err());
        four.reserve(caps, Some(bound(1)), false).unwrap();
    }
    #[test]
    fn partially_hooked_child_consumes_its_inclusive_bound_once() {
        let mut caps = limits(); caps.attempts = 4; caps.evaluations = Some(6);
        let parent = TaskAdmission::new(caps, pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None), None, false);
        let one = parent.child().unwrap(); let two = parent.child().unwrap();
        one.reserve(caps, Some(WorkObservation { evaluations: Some(5), ..zero() }), false).unwrap();
        two.reserve(caps, None, true).unwrap();
        one.admit(unit()).unwrap(); two.admit(unit()).unwrap();
        assert!(two.admit(unit()).is_err());
        one.observe(unit()).unwrap(); two.observe(unit()).unwrap();
        one.complete_charge(charge(Some(3))).unwrap();
        let mut second = charge(Some(1)); second.charging_owner = pse_ids::ContentHash::from_bytes([2; 32]);
        two.complete_charge(second).unwrap();
        assert_eq!(parent.observation().unwrap().evaluations, Some(4));
    }

}
