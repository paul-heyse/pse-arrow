// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native ownership ends after join, including thread-local foreign destructors.
use super::{MathRuntimeError, MathService};
use pse_columnar::flight::FlightCancellation;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
/// Additional known numeric constructor payload grows the same admitted pool
/// reservation immediately. No CPU/native owner waits for memory at this boundary.
#[derive(Debug)]
struct ConstructionOwner {
    reservation: Arc<datafusion::execution::memory_pool::MemoryReservation>,
    initial: usize,
    capacity: usize,
    additional: std::sync::Mutex<usize>,
    cancelled: Arc<std::sync::atomic::AtomicBool>,
}
impl pse_math::construction::ConstructionAdmission for ConstructionOwner {
    fn try_grow(&self, bytes: usize) -> Result<(), pse_math::MathError> {
        if self.cancelled.load(Ordering::Acquire) {
            return Err(pse_math::MathError::Cancelled);
        }
        let mut additional = self.additional.lock().map_err(|_| {
            pse_math::MathError::Contract("construction admission lock poisoned".into())
        })?;
        let required = self
            .initial
            .checked_add(*additional)
            .and_then(|total| total.checked_add(bytes))
            .ok_or(pse_math::MathError::Limit(
                "construction allocation overflow",
            ))?;
        if required > self.capacity {
            return Err(pse_math::MathError::ByteLimit {
                resource: "constructor worker bytes",
                required,
                available: self.capacity,
            });
        }
        self.reservation
            .try_grow(bytes)
            .map_err(|_| pse_math::MathError::Limit("constructor common pool"))?;
        *additional += bytes;
        Ok(())
    }
}
/// The worker capacity of one admitted job, carved from the job's reservation. Every
/// evaluator the job builds charges its numeric storage here and releases the charge when
/// it is dropped, so the reservation covers all of the job's live workers together, never
/// only the first; a worker that does not fit is refused (F31).
///
/// A native session's budget instead draws each charge from the deployment pool as it is
/// made, up to its capacity, so an idle session holds no worker storage.
#[derive(Debug)]
pub struct WorkerBudget {
    capacity: usize,
    used: Arc<AtomicUsize>,
    pool: Option<Arc<datafusion::execution::memory_pool::MemoryReservation>>,
    admission: Option<Arc<super::strategy::admission::TaskAdmission>>,
}
impl WorkerBudget {
    /// A budget of `capacity` bytes, the worker share of the job's reservation.
    pub(crate) fn new(capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            capacity,
            used: Arc::new(AtomicUsize::new(0)),
            pool: None,
            admission: None,
        })
    }
    /// A budget of up to `capacity` bytes charged to `pool` as workers are built.
    pub(crate) fn drawing(
        capacity: usize,
        pool: &Arc<dyn datafusion::execution::memory_pool::MemoryPool>,
    ) -> Arc<Self> {
        Arc::new(Self {
            capacity,
            used: Arc::new(AtomicUsize::new(0)),
            pool: Some(Arc::new(
                datafusion::execution::memory_pool::MemoryConsumer::new("math:session-workers")
                    .register(pool),
            )),
            admission: None,
        })
    }
    pub(crate) fn with_admission(
        self: &Arc<Self>,
        admission: Arc<super::strategy::admission::TaskAdmission>,
    ) -> Arc<Self> {
        Arc::new(Self {
            capacity: self.capacity,
            used: self.used.clone(),
            pool: self.pool.clone(),
            admission: Some(admission),
        })
    }
    pub(crate) fn admission(&self) -> Option<Arc<super::strategy::admission::TaskAdmission>> {
        self.admission.clone()
    }
    /// Admit a scientific evaluation before entering its evaluator, preserving failed work.
    pub(crate) fn evaluate<T>(
        &self,
        work: impl FnOnce() -> Result<T, MathRuntimeError>,
    ) -> Result<T, MathRuntimeError> {
        use pse_backend_native::solve::WorkAdmission;
        let unit = pse_backend_native::solve::WorkEvidence {
            evaluations: Some(1),
            iterations: Some(0),
            factorizations: Some(0),
            proof_steps: Some(0),
        };
        if let Some(admission) = &self.admission {
            admission.admit(unit)?;
        }
        // Preserve an attempted evaluation even when the evaluator unwinds. The
        // worker's enclosing panic boundary still owns conversion to a typed cause.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
        let observed = self
            .admission
            .as_ref()
            .map(|admission| admission.observe(unit));
        match result {
            Ok(result) => {
                if let Some(observed) = observed {
                    observed?;
                }
                result
            }
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }
    /// Reserve `bytes` for one worker until the returned charge is dropped.
    ///
    /// # Errors
    /// The job's live workers and this one exceed its worker capacity, or the pool cannot
    /// admit a session's charge.
    pub fn charge(self: &Arc<Self>, bytes: usize) -> Result<WorkerCharge, MathRuntimeError> {
        self.used
            .try_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                used.checked_add(bytes).filter(|n| *n <= self.capacity)
            })
            .map_err(|_| MathRuntimeError::Limit("worker storage"))?;
        if let Some(pool) = &self.pool
            && let Err(error) = pool.try_grow(bytes)
        {
            self.used.fetch_sub(bytes, Ordering::AcqRel);
            return Err(error.into());
        }
        Ok(WorkerCharge {
            budget: self.clone(),
            bytes,
        })
    }
    /// The worker share of the job's reservation.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    /// Bytes charged by the job's live workers.
    pub fn used(&self) -> usize {
        self.used.load(Ordering::Acquire)
    }
}
/// One live worker's share of its job's reservation.
#[derive(Debug)]
pub struct WorkerCharge {
    budget: Arc<WorkerBudget>,
    bytes: usize,
}
impl Drop for WorkerCharge {
    fn drop(&mut self) {
        if let Some(pool) = &self.budget.pool {
            pool.shrink(self.bytes);
        }
        self.budget.used.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
/// Cancels on future abandonment while the detached supervisor retains all resource owners.
struct Caller(Option<FlightCancellation>);
impl Drop for Caller {
    fn drop(&mut self) {
        if let Some(cancel) = &self.0 {
            cancel.cancel();
        }
    }
}
/// How a submitted operation is admitted, cancelled and observed.
#[derive(Debug)]
pub(crate) struct Submission {
    /// The operation's cancellation, shared with its public handle.
    pub(crate) cancel: FlightCancellation,
    /// Its event stream: bounded in memory, and tapped by a durable stream when durable.
    pub(crate) progress: Arc<pse_backend_native::solve::Progress>,
    /// Signalled once the operation holds its job slot, CPU permits and reservation.
    pub(crate) admitted: Option<tokio::sync::oneshot::Sender<()>>,
    /// Absolute task deadline, including durable job and CPU waiting.
    pub(crate) deadline: Option<std::time::Instant>,
}
impl Submission {
    /// In-memory work: a fresh cancellation, 256 retained events, refused when full.
    pub(crate) fn ephemeral() -> Self {
        Self {
            cancel: FlightCancellation::default(),
            progress: Arc::new(pse_backend_native::solve::Progress::new(256)),
            admitted: None,
            deadline: None,
        }
    }
}
/// One synchronously owned member of the pending/running/retained population.
/// It is transferred into the existing dispatch/join owner without reacquisition.
#[derive(Debug)]
pub(super) struct AdmissionEntry {
    at: std::time::Instant,
    bytes: usize,
    construction_bytes: usize,
    slot: tokio::sync::OwnedSemaphorePermit,
}
impl MathService {
    /// Submit one finite operation under the existing admission, teardown and join owner.
    pub(crate) fn submit<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        work: impl FnOnce(
            Arc<std::sync::atomic::AtomicBool>,
            Arc<pse_backend_native::solve::Progress>,
        ) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<super::solves::SolveHandle<(T, Arc<pse_columnar::AllocationLease>)>, MathRuntimeError>
    {
        self.submit_with(cores, bytes, Submission::ephemeral(), work)
    }
    /// Submit one finite operation with an explicit admission policy, cancellation and
    /// event stream.
    pub(crate) fn submit_with<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        submission: Submission,
        work: impl FnOnce(
            Arc<std::sync::atomic::AtomicBool>,
            Arc<pse_backend_native::solve::Progress>,
        ) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<super::solves::SolveHandle<(T, Arc<pse_columnar::AllocationLease>)>, MathRuntimeError>
    {
        let service = self.clone();
        let Submission {
            cancel,
            progress,
            admitted,
            deadline,
        } = submission;
        let entry = self.admit_entry(cores, bytes, &cancel, deadline, None)?;
        let flag = cancel.clone();
        let events = progress.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let result = service
                .admitted(
                    cores,
                    bytes,
                    flag,
                    (admitted, deadline),
                    Some(entry),
                    move |flag| work(flag, events),
                )
                .await;
            let _ = sender.send(result);
        });
        Ok(super::solves::SolveHandle {
            cancel,
            receiver: Some(receiver),
            progress,
        })
    }
    pub(crate) async fn job<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        cancel: FlightCancellation,
        work: impl FnOnce(Arc<std::sync::atomic::AtomicBool>) -> Result<T, MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<T, MathRuntimeError> {
        self.job_retained(cores, bytes, cancel, move |flag| work(flag).map(|r| (r, 0)))
            .await
            .map(|(r, _)| r)
    }
    /// Run an attempt with queueing, execution and join under its original deadline.
    pub(crate) async fn job_scoped<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        cancel: FlightCancellation,
        deadline: Option<std::time::Instant>,
        work: impl FnOnce(Arc<std::sync::atomic::AtomicBool>) -> Result<T, MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<T, MathRuntimeError> {
        self.job_retained_scoped(cores, bytes, cancel, deadline, move |flag| {
            work(flag).map(|value| (value, 0))
        })
        .await
        .map(|(value, _)| value)
    }
    /// Transfer retained capacity from the active job reservation after native join.
    /// The returned owner is acquired without releasing/reacquiring pool capacity; a
    /// product larger than `bytes` grows the reservation to its extent first.
    pub(super) async fn job_retained<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        cancel: FlightCancellation,
        work: impl FnOnce(Arc<std::sync::atomic::AtomicBool>) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<(T, Arc<pse_columnar::AllocationLease>), MathRuntimeError> {
        self.admitted(cores, bytes, cancel, (None, None), None, work)
            .await
    }
    /// Prepare/advance a product under the caller's original task clock. Waiting for
    /// capacity and execution share this deadline; it is never interpreted as cancellation.
    pub(super) async fn job_retained_scoped<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        cancel: FlightCancellation,
        deadline: Option<std::time::Instant>,
        work: impl FnOnce(Arc<std::sync::atomic::AtomicBool>) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<(T, Arc<pse_columnar::AllocationLease>), MathRuntimeError> {
        self.admitted(cores, bytes, cancel, (None, deadline), None, work)
            .await
    }
    /// Consume a ticket acquired before a synchronous submission creates any task.
    pub(super) async fn job_scoped_on_entry<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        cancel: FlightCancellation,
        deadline: Option<std::time::Instant>,
        entry: AdmissionEntry,
        work: impl FnOnce(Arc<std::sync::atomic::AtomicBool>) -> Result<T, MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<T, MathRuntimeError> {
        self.job_retained_on_entry(cores, bytes, cancel, deadline, entry, move |flag| {
            work(flag).map(|value| (value, 0))
        })
        .await
        .map(|(value, _)| value)
    }
    /// Transfer the same admitted entry through dispatch, teardown and retention.
    pub(super) async fn job_retained_on_entry<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        cancel: FlightCancellation,
        deadline: Option<std::time::Instant>,
        entry: AdmissionEntry,
        work: impl FnOnce(Arc<std::sync::atomic::AtomicBool>) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<(T, Arc<pse_columnar::AllocationLease>), MathRuntimeError> {
        self.admitted(cores, bytes, cancel, (None, deadline), Some(entry), work)
            .await
    }
    /// Refuse a synchronous burst before its handle/task joins the bounded population.
    /// A multi-phase operation supplies one unchanged admission cutoff; the separate
    /// enclosing deadline remains the only post-dispatch execution clock.
    pub(super) fn admit_entry(
        &self,
        cores: usize,
        bytes: usize,
        cancel: &FlightCancellation,
        deadline: Option<std::time::Instant>,
        cutoff: Option<std::time::Instant>,
    ) -> Result<AdmissionEntry, MathRuntimeError> {
        if cores == 0 || cores > self.cores || cores > u32::MAX as usize {
            return Err(MathRuntimeError::Limit("optimizer cores"));
        }
        let construction_bytes = bytes;
        let stacks = if cores > 1 {
            cores.checked_add(1)
        } else {
            Some(1)
        };
        let bytes = stacks
            .and_then(|n| n.checked_mul(self.policy.stack_bytes))
            .and_then(|n| n.checked_add(bytes))
            .and_then(|n| n.checked_add(self.policy.foreign_bytes))
            .and_then(|n| n.checked_add(self.policy.inner_session_bytes))
            .ok_or(MathRuntimeError::Limit("native allowance overflow"))?;
        self.check_entry_extent(bytes)?;
        let at = match cutoff {
            Some(at) => at,
            None => self.admission_deadline(deadline)?,
        };
        if cancel.flag().load(Ordering::Acquire) {
            return Err(MathRuntimeError::Cancelled);
        }
        if std::time::Instant::now() >= at {
            return Err(Self::admission_timeout());
        }
        let slot = self
            .jobs
            .clone()
            .try_acquire_owned()
            .map_err(|_| MathRuntimeError::Limit("native jobs"))?;
        Ok(AdmissionEntry {
            at,
            bytes,
            construction_bytes,
            slot,
        })
    }
    /// The enclosing deadline is authoritative. Unscoped callers receive a finite
    /// admission-only clock; it is never used to classify post-dispatch execution.
    pub(crate) fn admission_deadline(
        &self,
        deadline: Option<std::time::Instant>,
    ) -> Result<std::time::Instant, MathRuntimeError> {
        deadline.map_or_else(
            || {
                std::time::Instant::now()
                    .checked_add(self.policy.admission_wait)
                    .ok_or(MathRuntimeError::Limit("admission wait overflow"))
            },
            Ok,
        )
    }
    pub(crate) fn admission_timeout() -> MathRuntimeError {
        MathRuntimeError::Solve(pse_backend_native::ProblemError::Limit {
            kind: pse_backend_native::LimitKind::Time,
            detail: "native admission deadline".into(),
        })
    }
    /// Bound before dispatch; a request larger than the pool cannot improve on release.
    pub(crate) fn check_entry_extent(&self, bytes: usize) -> Result<(), MathRuntimeError> {
        if matches!(self.pool.memory_limit(), datafusion::execution::memory_pool::MemoryLimit::Finite(limit) if bytes > limit)
        {
            return Err(MathRuntimeError::Limit(
                "native entry exceeds deployment memory",
            ));
        }
        Ok(())
    }
    /// Memory-only entry for a not-yet-created native session. The caller owns the
    /// single population ticket and does not hold CPU/native exclusion while waiting.
    pub(crate) async fn reserve_entry(
        &self,
        name: &str,
        bytes: usize,
        cancel: &FlightCancellation,
        deadline: Option<std::time::Instant>,
    ) -> Result<Arc<pse_columnar::AllocationLease>, MathRuntimeError> {
        self.check_entry_extent(bytes)?;
        let at = self.admission_deadline(deadline)?;
        let reservation =
            datafusion::execution::memory_pool::MemoryConsumer::new(name).register(&self.pool);
        loop {
            // notify_waiters is observed even before the first poll of this future.
            let released = self.released.notified();
            if cancel.flag().load(Ordering::Acquire) {
                return Err(MathRuntimeError::Cancelled);
            }
            if std::time::Instant::now() >= at {
                return Err(Self::admission_timeout());
            }
            match reservation.try_grow(bytes) {
                Ok(()) => return Ok(pse_columnar::AllocationLease::new(reservation)),
                Err(datafusion::common::DataFusionError::ResourcesExhausted(_)) => {}
                Err(error) => return Err(error.into()),
            }
            tokio::select! {
                biased;
                () = cancel.cancelled() => return Err(MathRuntimeError::Cancelled),
                () = tokio::time::sleep_until(at.into()) => return Err(Self::admission_timeout()),
                () = released => {},
            }
        }
    }
    /// One ticket bounds pending, running and retained owners together. Temporary
    /// pool pressure waits without holding CPU capacity, under the original clock.
    async fn admitted<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        cancel: FlightCancellation,
        (admitted, deadline): (
            Option<tokio::sync::oneshot::Sender<()>>,
            Option<std::time::Instant>,
        ),
        entry: Option<AdmissionEntry>,
        work: impl FnOnce(Arc<std::sync::atomic::AtomicBool>) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<(T, Arc<pse_columnar::AllocationLease>), MathRuntimeError> {
        let entry = match entry {
            Some(entry) => entry,
            None => self.admit_entry(cores, bytes, &cancel, deadline, None)?,
        };
        let AdmissionEntry {
            at,
            bytes,
            construction_bytes,
            slot,
        } = entry;
        if cancel.flag().load(Ordering::Acquire) {
            return Err(MathRuntimeError::Cancelled);
        }
        if std::time::Instant::now() >= at {
            return Err(Self::admission_timeout());
        }
        let mut caller = Caller(Some(cancel.clone()));
        let service = self.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let result = async {
                let lease = Arc::new(datafusion::execution::memory_pool::MemoryConsumer::new("math:native-job").register(&service.pool));
                let cpu = loop {
                    let released = service.released.notified();
                    let cpu = tokio::select! {
                        biased;
                        () = cancel.cancelled() => return Err(MathRuntimeError::Cancelled),
                        () = tokio::time::sleep_until(at.into()) => return Err(Self::admission_timeout()),
                        p = service.cpu.clone().acquire_many_owned(cores as u32) =>
                            p.map_err(|_| MathRuntimeError::Limit("CPU admission closed"))?,
                    };
                    match lease.try_grow(bytes) {
                        Ok(()) => break cpu,
                        Err(datafusion::common::DataFusionError::ResourcesExhausted(_)) => {},
                        Err(error) => return Err(error.into()),
                    }
                    drop(cpu);
                    tokio::select! {
                        biased;
                        () = cancel.cancelled() => return Err(MathRuntimeError::Cancelled),
                        () = tokio::time::sleep_until(at.into()) => return Err(Self::admission_timeout()),
                        () = released => {},
                    }
                };
                if cancel.flag().load(Ordering::Acquire) { return Err(MathRuntimeError::Cancelled); }
                if std::time::Instant::now() >= at { return Err(Self::admission_timeout()); }
                if let Some(admitted) = admitted { let _ = admitted.send(()); }
                let flag = cancel.flag();
                let compute = super::staged::ComputeOwner::new(
                    service.cpu.clone(), cores as u32, cpu, tokio::runtime::Handle::current(),
                );
                let native_compute = compute.clone();
                let construction = Arc::new(ConstructionOwner {
                    reservation: lease.clone(), initial: construction_bytes,
                    capacity: service.policy.worker_bytes, additional: std::sync::Mutex::new(0),
                    cancelled: flag.clone(),
                });
                let sessions = service.policy.inner_session_bytes;
                let handle = std::thread::Builder::new().name("pse-math".into())
                    .stack_size(service.policy.stack_bytes).spawn(move || {
                        #[cfg(feature = "solver-kinsol")]
                        pse_backend_native::implicit::budget_sessions(sessions);
                        #[cfg(not(feature = "solver-kinsol"))]
                        let _ = sessions;
                        pse_backend_native::execution::compute_scoped(Box::new(native_compute), ||
                            pse_math::construction::scoped(construction, || work(flag)))
                    }).map_err(|e| MathRuntimeError::Infrastructure(e.to_string()))?;
                // Joining witnesses thread-local and native destructor completion.
                let result = tokio::task::spawn_blocking(move || handle.join()).await
                    .map_err(|e| MathRuntimeError::Infrastructure(e.to_string()))?
                    .map_err(|_| MathRuntimeError::Panic("native worker panic".into()))?;
                let result = result.and_then(|(value, retained)| {
                    if cancel.flag().load(Ordering::Acquire) { return Err(MathRuntimeError::Cancelled); }
                    // The admission-only clock ends at dispatch. Preserve actual task expiry.
                    if deadline.is_some_and(|at| std::time::Instant::now() >= at) {
                        return Err(Self::admission_timeout());
                    }
                    if let Some(more) = retained.checked_sub(lease.size()).filter(|n| *n > 0) {
                        lease.try_grow(more)?;
                    }
                    Ok((value, pse_columnar::AllocationLease::new(lease.split(retained))))
                });
                drop(lease);
                // This shared owner still retains the actual permit through native TLS
                // destruction and join, even after the thread's compute scope leaves.
                drop(compute);
                result
            }.await;
            drop(slot);
            let _ = tx.send(result);
        });
        let result = rx
            .await
            .map_err(|_| MathRuntimeError::Infrastructure("lost native supervisor".into()))?;
        caller.0.take();
        result
    }
}

#[cfg(test)]
mod construction_tests {
    use super::*;
    use datafusion::execution::memory_pool::{FairSpillPool, MemoryConsumer, MemoryPool};
    use pse_math::construction::ConstructionAdmission;
    #[tokio::test(flavor = "current_thread")]
    async fn synchronous_submission_burst_refuses_thirty_third_before_spawn() {
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy {
                jobs: 32,
                foreign_bytes: 1 << 20,
                ..Default::default()
            },
        );
        let baseline = service.pool.reserved();
        let cpu = service.cpu.clone().acquire_many_owned(2).await.unwrap();
        let called = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();
        for _ in 0..32 {
            let observed = called.clone();
            handles.push(
                service
                    .submit(1, 0, move |_, _| {
                        observed.fetch_add(1, Ordering::Relaxed);
                        Ok(((), 0))
                    })
                    .unwrap(),
            );
        }
        assert_eq!(service.jobs.available_permits(), 0);
        assert!(matches!(
            service.submit(1, 0, |_, _| Ok(((), 0))),
            Err(MathRuntimeError::Limit("native jobs"))
        ));
        assert_eq!(called.load(Ordering::Relaxed), 0);
        for handle in &handles {
            handle.cancel();
        }
        for handle in handles {
            assert!(matches!(
                handle.finish().await,
                Err(MathRuntimeError::Cancelled)
            ));
        }
        assert_eq!(called.load(Ordering::Relaxed), 0);
        assert_eq!(service.jobs.available_permits(), 32);
        assert_eq!(service.pool.reserved(), baseline);
        drop(cpu);
        assert_eq!(service.cpu.available_permits(), 2);
    }
    #[test]
    fn constructor_growth_uses_same_reservation_and_checks_worker_capacity() {
        let pool: Arc<dyn MemoryPool> = Arc::new(FairSpillPool::new(32));
        let reservation = Arc::new(MemoryConsumer::new("constructor test").register(&pool));
        reservation.try_grow(8).unwrap();
        let owner = ConstructionOwner {
            reservation: reservation.clone(),
            initial: 4,
            capacity: 12,
            additional: std::sync::Mutex::new(0),
            cancelled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };
        owner.try_grow(4).unwrap();
        assert_eq!(reservation.size(), 12);
        assert_eq!(pool.reserved(), 12);
        assert!(matches!(
            owner.try_grow(5),
            Err(pse_math::MathError::ByteLimit {
                resource: "constructor worker bytes",
                required: 13,
                available: 12,
            })
        ));
        assert_eq!(reservation.size(), 12);
        drop(owner);
        drop(reservation);
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn constructor_pool_pressure_refuses_immediately_without_charging_or_waiting() {
        let pool: Arc<dyn MemoryPool> = Arc::new(FairSpillPool::new(16));
        let reservation = Arc::new(MemoryConsumer::new("constructor test").register(&pool));
        reservation.try_grow(8).unwrap();
        let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let owner = ConstructionOwner {
            reservation: reservation.clone(),
            initial: 8,
            capacity: 64,
            additional: std::sync::Mutex::new(0),
            cancelled: cancelled.clone(),
        };
        assert!(matches!(
            owner.try_grow(9),
            Err(pse_math::MathError::Limit("constructor common pool"))
        ));
        assert_eq!(reservation.size(), 8);
        assert_eq!(*owner.additional.lock().unwrap(), 0);
        cancelled.store(true, Ordering::Release);
        assert!(matches!(
            owner.try_grow(1),
            Err(pse_math::MathError::Cancelled)
        ));
        assert_eq!(pool.reserved(), 8);
    }
}
