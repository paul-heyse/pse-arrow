// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native ownership ends after join, including thread-local foreign destructors.
use super::{MathRuntimeError, MathService};
use pse_diagnostics::Cancellation;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
// Independent spawned tasks do not inherit a resident lane.
tokio::task_local! { static POPULATION: Arc<PopulationLane>; }
/// One resident population owner, with an exclusive loan through actual child join.
#[derive(Debug)]
pub(super) struct PopulationLane {
    jobs: Arc<pse_engine::resources::ReleaseNotifyingSemaphore>,
    _ticket: Arc<pse_engine::resources::ReleasedPermit>,
    in_use: std::sync::atomic::AtomicBool,
    closed: std::sync::atomic::AtomicBool,
    drained: tokio::sync::Notify,
}
impl PopulationLane {
    pub(super) fn new(
        jobs: Arc<pse_engine::resources::ReleaseNotifyingSemaphore>,
        ticket: Arc<pse_engine::resources::ReleasedPermit>,
    ) -> Arc<Self> {
        Arc::new(Self {
            jobs,
            _ticket: ticket,
            in_use: false.into(),
            closed: false.into(),
            drained: tokio::sync::Notify::new(),
        })
    }
    pub(super) fn claim(self: &Arc<Self>) -> Result<PopulationUse, MathRuntimeError> {
        if self.closed.load(Ordering::Acquire) {
            return Err(MathRuntimeError::Limit("native population lane closed"));
        }
        self.in_use
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| MathRuntimeError::Limit("concurrent native population loan"))?;
        let usage = PopulationUse(self.clone());
        if self.closed.load(Ordering::Acquire) {
            return Err(MathRuntimeError::Limit("native population lane closed"));
        }
        Ok(usage)
    }
    pub(super) fn idle(&self) -> bool {
        !self.in_use.load(Ordering::Acquire) && !self.closed.load(Ordering::Acquire)
    }
    pub(super) fn close(&self) {
        self.closed.store(true, Ordering::Release);
    }
    pub(super) async fn drain(&self) {
        loop {
            let notified = self.drained.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if !self.in_use.load(Ordering::Acquire) {
                return;
            }
            notified.await;
        }
    }
}
#[derive(Debug)]
pub(super) struct PopulationUse(Arc<PopulationLane>);
impl Drop for PopulationUse {
    fn drop(&mut self) {
        self.0.in_use.store(false, Ordering::Release);
        self.0.drained.notify_waiters();
    }
}
/// Explicit propagation into a sanctioned dependent loader. Concurrent children refuse;
/// captured scopes close with the resident owner and cannot start later independent work.
#[derive(Clone)]
pub(crate) struct PopulationScope(Option<Arc<PopulationLane>>);
impl PopulationScope {
    pub(crate) fn capture() -> Self {
        Self(POPULATION.try_with(Clone::clone).ok())
    }
    pub(super) fn belongs_to(&self, service: &MathService) -> bool {
        self.0
            .as_ref()
            .is_some_and(|lane| Arc::ptr_eq(&lane.jobs, &service.jobs))
    }
    pub(crate) async fn run<T>(self, work: impl Future<Output = T>) -> T {
        match self.0 {
            Some(lane) => population_scope(lane, work).await,
            None => work.await,
        }
    }
}
pub(super) async fn population_scope<T>(
    lane: Arc<PopulationLane>,
    work: impl Future<Output = T>,
) -> T {
    POPULATION.scope(lane, work).await
}
#[derive(Debug)]
enum PopulationEntry {
    Owned {
        _ticket: pse_engine::resources::ReleasedPermit,
    },
    Borrowed {
        _usage: PopulationUse,
    },
}
/// Both active resources are acquired tentatively, never held while awaiting another.
#[derive(Debug)]
pub(super) struct ActivePermits {
    _job: pse_engine::resources::ReleasedPermit,
    _cpu: pse_engine::resources::ReleasedPermit,
}
/// The actual failed resource selects the wake source; tentative rollback must not
/// wake its own retry loop through the other semaphore.
pub(super) enum ActiveAttempt {
    Ready(ActivePermits),
    JobsBusy,
    CpuBusy,
}
impl ActiveAttempt {
    #[cfg(test)]
    pub(super) fn acquired(self) -> Option<ActivePermits> {
        match self {
            Self::Ready(permit) => Some(permit),
            Self::JobsBusy | Self::CpuBusy => None,
        }
    }
}
impl MathService {
    /// Reserve one resident constructor population before creating its owner-affine
    /// worker. Opaque reports consume this prepaid envelope instead of charging it again.
    #[cfg(any(test, feature = "solver-diffsol", feature = "solver-idas"))]
    pub(crate) fn reserve_constructor(
        &self,
        name: &str,
        construction: ConstructionEstimate,
        execution: &pse_backend_native::solve::Execution,
    ) -> Result<Arc<dyn pse_math::construction::ConstructionAdmission>, MathRuntimeError> {
        execution.check()?;
        if construction.bytes() > self.policy.worker_bytes {
            return Err(MathRuntimeError::Limit("constructor worker capacity"));
        }
        let reservation = Arc::new(
            datafusion::execution::memory_pool::MemoryConsumer::new(name).register(&self.pool),
        );
        self.pressure.try_grow(&reservation, construction.bytes())?;
        execution.check()?;
        Ok(Arc::new(ConstructionOwner::new(
            reservation,
            construction,
            self.policy.worker_bytes,
            execution.cancel.clone(),
            self.pressure.clone(),
        )))
    }
    /// A genuine synchronous phase boundary lends its population to the nested team.
    /// Parent resident memory remains charged, but active resources are released before
    /// complete additional construction admission. Failed full-team attempts release
    /// temporary memory and permits so smaller ready stages remain eligible.
    pub(crate) fn quiescent_phase<T>(
        &self,
        cores: usize,
        bytes: usize,
        execution: &pse_backend_native::solve::Execution,
        work: impl FnOnce() -> T,
    ) -> Result<T, MathRuntimeError> {
        self.quiescent_phase_owned(cores, bytes, execution, |_| work())
    }
    pub(super) fn quiescent_phase_owned<T>(
        &self,
        cores: usize,
        bytes: usize,
        execution: &pse_backend_native::solve::Execution,
        work: impl FnOnce(Arc<datafusion::execution::memory_pool::MemoryReservation>) -> T,
    ) -> Result<T, MathRuntimeError> {
        self.quiescent_phase_owned_construction(
            cores,
            ConstructionEstimate::Known(bytes),
            execution,
            work,
        )
    }
    pub(super) fn quiescent_phase_owned_construction<T>(
        &self,
        cores: usize,
        construction: ConstructionEstimate,
        execution: &pse_backend_native::solve::Execution,
        work: impl FnOnce(Arc<datafusion::execution::memory_pool::MemoryReservation>) -> T,
    ) -> Result<T, MathRuntimeError> {
        let bytes = construction.bytes();
        if matches!(construction, ConstructionEstimate::Opaque(_))
            && bytes > self.policy.worker_bytes
        {
            return Err(MathRuntimeError::Limit("constructor worker capacity"));
        }
        if cores == 0 || cores > self.cores || cores > u32::MAX as usize {
            return Err(MathRuntimeError::Limit("optimizer cores"));
        }
        self.check_entry_extent(bytes)?;
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|error| MathRuntimeError::Infrastructure(error.to_string()))?;
        pse_backend_native::execution::pause_compute();
        let admitted = runtime.block_on(async {
            let lease = Arc::new(
                datafusion::execution::memory_pool::MemoryConsumer::new("math:phase")
                    .register(&self.pool),
            );
            let mut pressure = None;
            let job_releases = self.active_jobs.released();
            let cpu_releases = self.cpu.released();
            let task_scope = execution.scope()?;
            loop {
                let abandoned = execution.abandonment.notified();
                execution.check()?;
                let released = self.released.notified();
                tokio::pin!(released);
                released.as_mut().enable();
                let jobs_released = job_releases.notified();
                let cpu_released = cpu_releases.notified();
                tokio::pin!(jobs_released, cpu_released);
                jobs_released.as_mut().enable();
                cpu_released.as_mut().enable();
                let reserved = match lease.try_grow(bytes) {
                    Ok(()) => true,
                    Err(datafusion::common::DataFusionError::ResourcesExhausted(_)) => {
                        pressure.get_or_insert_with(|| self.pressure.demand());
                        self.trim_compiler_lanes();
                        self.retire_idle_native_lanes();
                        match self.pressure.retry_growth(&lease, bytes) {
                            Ok(()) => true,
                            Err(datafusion::common::DataFusionError::ResourcesExhausted(_)) => {
                                false
                            }
                            Err(error) => return Err(error.into()),
                        }
                    }
                    Err(error) => return Err(MathRuntimeError::from(error)),
                };
                let mut cpu_busy = false;
                if reserved {
                    match try_active(&self.active_jobs, &self.cpu, cores as u32)? {
                        ActiveAttempt::Ready(active) => return Ok((lease, active)),
                        ActiveAttempt::CpuBusy => cpu_busy = true,
                        ActiveAttempt::JobsBusy => {},
                    }
                    lease.shrink(bytes);
                }
                tokio::select! {
                    // An active-capacity retry must not consume its own shrink
                    // notification; retain release wakeups for pool-pressure waits.
                    () = released, if !reserved => {},
                    () = cpu_released, if reserved && cpu_busy => {},
                    () = jobs_released, if reserved && !cpu_busy => {},
                    () = abandoned => {},
                    () = self.active_jobs.closed() => return Err(MathRuntimeError::Limit("active jobs closed")),
                    () = self.cpu.closed() => return Err(MathRuntimeError::Limit("CPU admission closed")),
                    () = task_scope.cancellation_control().cancelled() => {},
                    () = async { match task_scope.deadline() { Some(at) => tokio::time::sleep_until(at.into()).await, None => std::future::pending().await } } => {},
                }
            }
        });
        let result = admitted.and_then(|(lease, active)| {
            execution.check()?;
            let compute = super::staged::ComputeOwner::new(
                self.active_jobs.clone(),
                self.cpu.clone(),
                cores as u32,
                active,
                runtime,
            );
            let owner = Arc::new(ConstructionOwner::new(
                lease.clone(),
                construction,
                self.policy.worker_bytes,
                execution.cancel.clone(),
                self.pressure.clone(),
            ));
            let result = pse_backend_native::execution::compute_scoped(Box::new(compute), || {
                pse_math::construction::scoped(owner, || work(lease.clone()))
            });
            drop(lease);
            Ok(result)
        });
        // All nested workers/scopes have joined before the estimate's allowance returns.
        let resumed = pse_backend_native::execution::resume_compute(execution);
        match result {
            Err(error) => Err(error),
            Ok(result) => {
                resumed?;
                Ok(result)
            }
        }
    }
    pub(super) async fn reserve_active(
        &self,
        name: &str,
        cores: u32,
        bytes: usize,
        cancel: &Cancellation,
        at: std::time::Instant,
    ) -> Result<
        (
            Arc<datafusion::execution::memory_pool::MemoryReservation>,
            ActivePermits,
        ),
        MathRuntimeError,
    > {
        self.check_entry_extent(bytes)?;
        let lease = Arc::new(
            datafusion::execution::memory_pool::MemoryConsumer::new(name).register(&self.pool),
        );
        let mut pressure = None;
        let job_releases = self.active_jobs.released();
        let cpu_releases = self.cpu.released();
        let active = loop {
            let released = self.released.notified();
            tokio::pin!(released);
            released.as_mut().enable();
            let jobs_released = job_releases.notified();
            let cpu_released = cpu_releases.notified();
            tokio::pin!(jobs_released, cpu_released);
            jobs_released.as_mut().enable();
            cpu_released.as_mut().enable();
            if cancel.flag().load(Ordering::Acquire) {
                return Err(MathRuntimeError::Cancelled);
            }
            if std::time::Instant::now() >= at {
                return Err(Self::admission_timeout());
            }
            // Complete construction demand precedes both active resources.
            let reserved = match lease.try_grow(bytes) {
                Ok(()) => true,
                Err(datafusion::common::DataFusionError::ResourcesExhausted(_)) => {
                    pressure.get_or_insert_with(|| self.pressure.demand());
                    self.trim_compiler_lanes();
                    self.retire_idle_native_lanes();
                    match self.pressure.retry_growth(&lease, bytes) {
                        Ok(()) => true,
                        Err(datafusion::common::DataFusionError::ResourcesExhausted(_)) => false,
                        Err(error) => return Err(error.into()),
                    }
                }
                Err(error) => return Err(error.into()),
            };
            let mut cpu_busy = false;
            if reserved {
                match try_active(&self.active_jobs, &self.cpu, cores)? {
                    ActiveAttempt::Ready(active) => break active,
                    ActiveAttempt::CpuBusy => cpu_busy = true,
                    ActiveAttempt::JobsBusy => {}
                }
                lease.shrink(bytes);
            }
            // No tentative memory, job or CPU owner survives this retry wait.
            tokio::select! {
                biased;
                () = cancel.cancelled() => return Err(MathRuntimeError::Cancelled),
                () = self.active_jobs.closed() => return Err(MathRuntimeError::Limit("active jobs closed")),
                () = self.cpu.closed() => return Err(MathRuntimeError::Limit("CPU admission closed")),
                () = tokio::time::sleep_until(at.into()) => return Err(Self::admission_timeout()),
                // A successful tentative reservation was just shrunk when active
                // capacity was unavailable. Its own release notification must not
                // turn this retry into a ready loop that starves cancellation tasks.
                () = released, if !reserved => {},
                () = cpu_released, if reserved && cpu_busy => {},
                () = jobs_released, if reserved && !cpu_busy => {},
            }
        };
        Ok((lease, active))
    }
    #[cfg(test)]
    pub(super) fn try_active(&self, cores: u32) -> Result<Option<ActivePermits>, MathRuntimeError> {
        try_active(&self.active_jobs, &self.cpu, cores).map(ActiveAttempt::acquired)
    }
}
pub(super) fn try_active(
    jobs: &Arc<pse_engine::resources::ReleaseNotifyingSemaphore>,
    cpu: &Arc<pse_engine::resources::ReleaseNotifyingSemaphore>,
    cores: u32,
) -> Result<ActiveAttempt, MathRuntimeError> {
    use tokio::sync::TryAcquireError;
    // Closure is terminal even when the other capacity owner is busy.
    if jobs.is_closed() {
        return Err(MathRuntimeError::Limit("active jobs closed"));
    }
    if cpu.is_closed() {
        return Err(MathRuntimeError::Limit("CPU admission closed"));
    }
    let job = match jobs.clone().try_acquire_owned() {
        Ok(job) => job,
        Err(TryAcquireError::NoPermits) => return Ok(ActiveAttempt::JobsBusy),
        Err(TryAcquireError::Closed) => return Err(MathRuntimeError::Limit("active jobs closed")),
    };
    match cpu.clone().try_acquire_many_owned(cores) {
        Ok(cpu) => Ok(ActiveAttempt::Ready(ActivePermits {
            _job: job,
            _cpu: cpu,
        })),
        Err(TryAcquireError::NoPermits) => Ok(ActiveAttempt::CpuBusy),
        Err(TryAcquireError::Closed) => Err(MathRuntimeError::Limit("CPU admission closed")),
    }
}
/// Source-issued construction demand, or a prepaid envelope for a producer whose
/// complete population is opaque. The envelope is a charge, not an observed peak.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ConstructionEstimate {
    /// Initial source-bound populations are charged; reports reserve excess or
    /// newly discovered populations in addition to that initial demand.
    Known(usize),
    /// The complete opaque population is already charged; reports consume its
    /// finite envelope without charging the same capacity for a second time.
    Opaque(usize),
}
impl ConstructionEstimate {
    pub(crate) fn from_bound(bound: Option<usize>, capacity: usize) -> Self {
        bound.map_or(Self::Opaque(capacity), Self::Known)
    }
    pub(crate) fn bytes(self) -> usize {
        match self {
            Self::Known(bytes) | Self::Opaque(bytes) => bytes,
        }
    }
}
/// Known constructor excess grows the same admitted pool reservation immediately;
/// reports within an explicitly prepaid opaque envelope consume its existing charge.
/// No CPU/native owner waits for memory at this boundary.
#[derive(Debug)]
struct ConstructionOwner {
    reservation: Arc<datafusion::execution::memory_pool::MemoryReservation>,
    initial: usize,
    prepaid: usize,
    capacity: usize,
    additional: std::sync::Mutex<usize>,
    cancelled: Arc<std::sync::atomic::AtomicBool>,
    pressure: Arc<super::retention::Pressure>,
}
impl ConstructionOwner {
    fn new(
        reservation: Arc<datafusion::execution::memory_pool::MemoryReservation>,
        construction: ConstructionEstimate,
        capacity: usize,
        cancelled: Arc<std::sync::atomic::AtomicBool>,
        pressure: Arc<super::retention::Pressure>,
    ) -> Self {
        let (initial, prepaid, capacity) = match construction {
            ConstructionEstimate::Known(bytes) => (bytes, 0, capacity),
            ConstructionEstimate::Opaque(bytes) => (0, bytes, bytes.min(capacity)),
        };
        Self {
            reservation,
            initial,
            prepaid,
            capacity,
            additional: std::sync::Mutex::new(0),
            cancelled,
            pressure,
        }
    }
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
        let next = *additional + bytes;
        let growth = next.saturating_sub(self.prepaid) - (*additional).saturating_sub(self.prepaid);
        if growth > 0 {
            self.pressure
                .try_grow(&self.reservation, growth)
                .map_err(|_| pse_math::MathError::Limit("constructor common pool"))?;
        }
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
    pressure: Option<Arc<super::retention::Pressure>>,
}
impl WorkerBudget {
    /// A budget of `capacity` bytes, the worker share of the job's reservation.
    pub(crate) fn new(capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            capacity,
            used: Arc::new(AtomicUsize::new(0)),
            pool: None,
            admission: None,
            pressure: None,
        })
    }
    /// A budget of up to `capacity` bytes charged to `pool` as workers are built.
    #[cfg(test)]
    pub(crate) fn drawing(
        capacity: usize,
        pool: &Arc<dyn datafusion::execution::memory_pool::MemoryPool>,
    ) -> Arc<Self> {
        Self::drawing_with(capacity, pool, None)
    }
    pub(crate) fn drawing_for(capacity: usize, service: &MathService) -> Arc<Self> {
        Self::drawing_with(capacity, &service.pool, Some(service.pressure.clone()))
    }
    fn drawing_with(
        capacity: usize,
        pool: &Arc<dyn datafusion::execution::memory_pool::MemoryPool>,
        pressure: Option<Arc<super::retention::Pressure>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            capacity,
            used: Arc::new(AtomicUsize::new(0)),
            pool: Some(Arc::new(
                datafusion::execution::memory_pool::MemoryConsumer::new("math:session-workers")
                    .register(pool),
            )),
            admission: None,
            pressure,
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
            pressure: self.pressure.clone(),
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
        if let Some(pool) = &self.pool {
            let result = match &self.pressure {
                Some(pressure) => pressure.try_grow(pool, bytes),
                None => pool.try_grow(bytes),
            };
            if let Err(error) = result {
                self.used.fetch_sub(bytes, Ordering::AcqRel);
                return Err(error.into());
            }
        }
        Ok(WorkerCharge {
            allocation: Arc::new(WorkerAllocation {
                capacity: self.capacity,
                used: self.used.clone(),
                pool: self.pool.clone(),
                pressure: self.pressure.clone(),
                bytes,
            }),
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
    allocation: Arc<WorkerAllocation>,
}
/// Passive residency only: an idle allocation must not retain an old task ledger,
/// cancellation owner or deadline through the budget that admitted its construction.
#[derive(Debug)]
struct WorkerAllocation {
    capacity: usize,
    used: Arc<AtomicUsize>,
    pool: Option<Arc<datafusion::execution::memory_pool::MemoryReservation>>,
    pressure: Option<Arc<super::retention::Pressure>>,
    bytes: usize,
}
impl WorkerCharge {
    pub(super) fn share(&self) -> Self {
        Self {
            allocation: self.allocation.clone(),
        }
    }
    pub(super) fn exclusive(&self) -> bool {
        Arc::strong_count(&self.allocation) == 1
    }
    pub(super) fn belongs_to(&self, budget: &WorkerBudget) -> bool {
        Arc::ptr_eq(&self.allocation.used, &budget.used)
            && match (&self.allocation.pool, &budget.pool) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
    pub(super) fn compatible(&self, budget: &WorkerBudget, bytes: usize) -> bool {
        self.belongs_to(budget)
            && self.allocation.bytes == bytes
            && self.allocation.used.load(Ordering::Acquire) <= budget.capacity
    }
    /// Reset's public consuming map temporarily owns both numeric stacks. Charge
    /// that overlap against the same resident lane, without an attempt admission.
    pub(super) fn reset_overlap(&self) -> Result<Self, MathRuntimeError> {
        Arc::new(WorkerBudget {
            capacity: self.allocation.capacity,
            used: self.allocation.used.clone(),
            pool: self.allocation.pool.clone(),
            pressure: self.allocation.pressure.clone(),
            admission: None,
        })
        .charge(self.allocation.bytes)
    }
}
impl Drop for WorkerAllocation {
    fn drop(&mut self) {
        if let Some(pool) = &self.pool {
            pool.shrink(self.bytes);
        }
        self.used.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
/// Cancels on future abandonment while the detached supervisor retains all resource owners.
struct Caller(Option<Cancellation>);
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
    pub(crate) cancel: Cancellation,
    /// Its event stream: bounded in memory, and tapped by a durable stream when durable.
    pub(crate) progress: Arc<pse_backend_native::solve::Progress>,
    /// Signalled once the operation holds its job slot, CPU permits and reservation.
    pub(crate) admitted: Option<tokio::sync::oneshot::Sender<()>>,
    /// Absolute task deadline, including durable job and CPU waiting.
    pub(crate) deadline: Option<std::time::Instant>,
}
/// Current admission lifetime, including who owns expiration and its ready signal.
struct AdmissionControl {
    cancel: Cancellation,
    admitted: Option<tokio::sync::oneshot::Sender<()>>,
    deadline: Option<std::time::Instant>,
    source_owns_deadline: bool,
}
impl Submission {
    /// In-memory work: a fresh cancellation, 256 retained events, refused when full.
    pub(crate) fn ephemeral() -> Self {
        Self {
            cancel: Cancellation::default(),
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
    construction: ConstructionEstimate,
    slot: PopulationEntry,
    compiler: Option<Arc<std::sync::Mutex<Option<super::CompilerOwner>>>>,
}
impl AdmissionEntry {
    /// Set the constructor portion explicitly when the entry also covers other
    /// resources, or when a source retains its opaque construction envelope.
    pub(super) fn with_construction(mut self, construction: ConstructionEstimate) -> Self {
        self.construction = construction;
        self
    }
}
impl MathService {
    /// Submit one finite operation under the existing admission, teardown and join owner.
    pub(crate) fn submit<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        work: impl FnOnce(
            Cancellation,
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
            Cancellation,
            Arc<pse_backend_native::solve::Progress>,
        ) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<super::solves::SolveHandle<(T, Arc<pse_columnar::AllocationLease>)>, MathRuntimeError>
    {
        self.submit_construction(cores, ConstructionEstimate::Known(bytes), submission, work)
    }
    /// Detach the job's already admitted foreign allowance before dispatch. The
    /// explicit consumer may retain it independently of the joined task ledger.
    pub(crate) fn submit_with_storage<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        submission: Submission,
        work: impl FnOnce(
            Cancellation,
            Arc<pse_backend_native::solve::Progress>,
            Arc<pse_columnar::AllocationLease>,
        ) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<super::solves::SolveHandle<(T, Arc<pse_columnar::AllocationLease>)>, MathRuntimeError>
    {
        self.submit_construction_storage(
            cores,
            ConstructionEstimate::Known(bytes),
            submission,
            false,
            true,
            move |flag, progress, storage| {
                let storage =
                    storage.ok_or(MathRuntimeError::Limit("missing prepaid foreign allowance"))?;
                work(flag, progress, storage)
            },
        )
    }
    /// Submit construction whose source distinguishes an initial bound from a
    /// prepaid opaque envelope before any dispatch task is created.
    pub(crate) fn submit_construction<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        construction: ConstructionEstimate,
        submission: Submission,
        work: impl FnOnce(
            Cancellation,
            Arc<pse_backend_native::solve::Progress>,
        ) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<super::solves::SolveHandle<(T, Arc<pse_columnar::AllocationLease>)>, MathRuntimeError>
    {
        self.submit_construction_mode(cores, construction, submission, false, work)
    }
    /// Trusted closed child driver enforces its absolute deadline, kills and reaps.
    /// Expiry must not mutate the public user-cancellation source to wake it.
    #[cfg(all(unix, feature = "solver-kinsol"))]
    pub(super) fn submit_source_timed<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        submission: Submission,
        work: impl FnOnce(
            Cancellation,
            Arc<pse_backend_native::solve::Progress>,
        ) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<super::solves::SolveHandle<(T, Arc<pse_columnar::AllocationLease>)>, MathRuntimeError>
    {
        self.submit_construction_mode(
            cores,
            ConstructionEstimate::Opaque(bytes),
            submission,
            true,
            work,
        )
    }
    fn submit_construction_mode<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        construction: ConstructionEstimate,
        submission: Submission,
        source_owns_deadline: bool,
        work: impl FnOnce(
            Cancellation,
            Arc<pse_backend_native::solve::Progress>,
        ) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<super::solves::SolveHandle<(T, Arc<pse_columnar::AllocationLease>)>, MathRuntimeError>
    {
        self.submit_construction_storage(
            cores,
            construction,
            submission,
            source_owns_deadline,
            false,
            move |flag, progress, _| work(flag, progress),
        )
    }
    fn submit_construction_storage<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        construction: ConstructionEstimate,
        submission: Submission,
        source_owns_deadline: bool,
        retain_foreign: bool,
        work: impl FnOnce(
            Cancellation,
            Arc<pse_backend_native::solve::Progress>,
            Option<Arc<pse_columnar::AllocationLease>>,
        ) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<super::solves::SolveHandle<(T, Arc<pse_columnar::AllocationLease>)>, MathRuntimeError>
    {
        let bytes = construction.bytes();
        if matches!(construction, ConstructionEstimate::Opaque(_))
            && bytes > self.policy.worker_bytes
        {
            return Err(MathRuntimeError::Limit("constructor worker capacity"));
        }
        let service = self.clone();
        let Submission {
            cancel,
            progress,
            admitted,
            deadline,
        } = submission;
        let entry = self
            .admit_entry(cores, bytes, &cancel, deadline, None)?
            .with_construction(construction);
        let flag = cancel.clone();
        let events = progress.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(crate::invocation::inherit(crate::observation::inherit(
            crate::observation::child(
                "math.submission",
                pse_operations::canonical::inherit_activity(async move {
                    let result = service
                        .admitted_storage(
                            cores,
                            bytes,
                            AdmissionControl {
                                cancel: flag,
                                admitted,
                                deadline,
                                source_owns_deadline,
                            },
                            Some(entry),
                            retain_foreign,
                            move |flag, storage| work(flag, events, storage),
                        )
                        .await;
                    let _ = sender.send(result);
                }),
            ),
        )));
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
        cancel: Cancellation,
        work: impl FnOnce(Cancellation) -> Result<T, MathRuntimeError> + Send + 'static,
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
        cancel: Cancellation,
        deadline: Option<std::time::Instant>,
        work: impl FnOnce(Cancellation) -> Result<T, MathRuntimeError> + Send + 'static,
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
        cancel: Cancellation,
        work: impl FnOnce(Cancellation) -> Result<(T, usize), MathRuntimeError> + Send + 'static,
    ) -> Result<(T, Arc<pse_columnar::AllocationLease>), MathRuntimeError> {
        self.admitted(cores, bytes, cancel, (None, None, false), None, work)
            .await
    }
    /// Prepare/advance a product under the caller's original task clock. Waiting for
    /// capacity and execution share this deadline; it is never interpreted as cancellation.
    pub(super) async fn job_retained_scoped<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        cancel: Cancellation,
        deadline: Option<std::time::Instant>,
        work: impl FnOnce(Cancellation) -> Result<(T, usize), MathRuntimeError> + Send + 'static,
    ) -> Result<(T, Arc<pse_columnar::AllocationLease>), MathRuntimeError> {
        self.admitted(cores, bytes, cancel, (None, deadline, false), None, work)
            .await
    }
    /// Construct under a source-issued bound or its explicitly prepaid opaque
    /// fallback, preserving the same admission, thread, join and retention owner.
    pub(super) async fn job_retained_construction<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        construction: ConstructionEstimate,
        cancel: Cancellation,
        deadline: Option<std::time::Instant>,
        work: impl FnOnce(Cancellation) -> Result<(T, usize), MathRuntimeError> + Send + 'static,
    ) -> Result<(T, Arc<pse_columnar::AllocationLease>), MathRuntimeError> {
        let bytes = construction.bytes();
        if bytes > self.policy.worker_bytes {
            return Err(MathRuntimeError::Limit("constructor worker capacity"));
        }
        let entry = self
            .admit_entry(cores, bytes, &cancel, deadline, None)?
            .with_construction(construction);
        self.job_retained_on_entry(cores, bytes, cancel, deadline, entry, work)
            .await
    }
    pub(super) async fn job_construction<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        construction: ConstructionEstimate,
        cancel: Cancellation,
        deadline: Option<std::time::Instant>,
        work: impl FnOnce(Cancellation) -> Result<T, MathRuntimeError> + Send + 'static,
    ) -> Result<T, MathRuntimeError> {
        self.job_retained_construction(cores, construction, cancel, deadline, move |flag| {
            work(flag).map(|value| (value, 0))
        })
        .await
        .map(|(value, _)| value)
    }
    /// Admit population, await exclusive compiler ownership, then dispatch under one cutoff.
    /// No active CPU or native job is held while another compiler operation runs.
    pub(crate) async fn compiler_job_retained<T: Send + 'static>(
        self: &Arc<Self>,
        workspace: super::Workspace,
        construction: ConstructionEstimate,
        cancel: Cancellation,
        deadline: Option<std::time::Instant>,
        work: impl FnOnce(
            super::CompilerAuthority,
            Cancellation,
        ) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<(T, Arc<pse_columnar::AllocationLease>), MathRuntimeError> {
        let bytes = construction.bytes();
        if matches!(construction, ConstructionEstimate::Opaque(_))
            && bytes > self.policy.worker_bytes
        {
            return Err(MathRuntimeError::Limit("constructor worker capacity"));
        }
        let mut entry = self
            .admit_entry(1, bytes, &cancel, deadline, None)?
            .with_construction(construction);
        let authority = Self::compiler_authority(workspace, &cancel, entry.at).await?;
        entry.compiler = Some(authority.retention.clone());
        #[cfg(test)]
        let effects = self.preparation_effects.clone();
        self.job_retained_on_entry(1, bytes, cancel, deadline, entry, move |flag| {
            #[cfg(test)]
            let gate = effects
                .compiler_entry_gate
                .lock()
                .map_err(|_| {
                    MathRuntimeError::Infrastructure("test compiler gate lock poisoned".into())
                })?
                .take();
            #[cfg(test)]
            if let Some(gate) = gate {
                let _ = gate.entered.send(());
                let watchdog = std::time::Instant::now() + std::time::Duration::from_secs(5);
                while !flag.load(Ordering::Acquire) {
                    if std::time::Instant::now() >= watchdog {
                        return Err(MathRuntimeError::Infrastructure(
                            "test compiler stop not observed".into(),
                        ));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                let _ = gate.stopped.send(());
                gate.exit
                    .recv_timeout(std::time::Duration::from_secs(5))
                    .map_err(|error| MathRuntimeError::Infrastructure(error.to_string()))?;
            }
            work(authority, flag)
        })
        .await
    }
    pub(crate) async fn compiler_job<T: Send + 'static>(
        self: &Arc<Self>,
        workspace: super::Workspace,
        construction: ConstructionEstimate,
        cancel: Cancellation,
        deadline: Option<std::time::Instant>,
        work: impl FnOnce(super::CompilerAuthority, Cancellation) -> Result<T, MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<T, MathRuntimeError> {
        self.compiler_job_retained(
            workspace,
            construction,
            cancel,
            deadline,
            move |authority, flag| work(authority, flag).map(|value| (value, 0)),
        )
        .await
        .map(|(value, _)| value)
    }
    async fn compiler_authority(
        workspace: super::Workspace,
        cancel: &Cancellation,
        cutoff: std::time::Instant,
    ) -> Result<super::CompilerAuthority, MathRuntimeError> {
        if cancel.flag().load(Ordering::Acquire) {
            return Err(MathRuntimeError::Cancelled);
        }
        if std::time::Instant::now() >= cutoff {
            return Err(Self::admission_timeout());
        }
        let stage = pse_diagnostics::observation::phase("math.compiler_admission");
        let permit = tokio::select! {
            biased;
            () = cancel.cancelled() => return Err(MathRuntimeError::Cancelled),
            () = tokio::time::sleep_until(cutoff.into()) => return Err(Self::admission_timeout()),
            permit = workspace.admission.clone().acquire_owned() => permit.map_err(|_| {
                MathRuntimeError::Infrastructure("compiler admission closed".into())
            })?,
        };
        drop(stage);
        if cancel.flag().load(Ordering::Acquire) {
            return Err(MathRuntimeError::Cancelled);
        }
        if std::time::Instant::now() >= cutoff {
            return Err(Self::admission_timeout());
        }
        Ok(super::CompilerAuthority::new(workspace, permit))
    }
    /// Consume a ticket acquired before a synchronous submission creates any task.
    pub(super) async fn job_scoped_on_entry<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        cancel: Cancellation,
        deadline: Option<std::time::Instant>,
        entry: AdmissionEntry,
        work: impl FnOnce(Cancellation) -> Result<T, MathRuntimeError> + Send + 'static,
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
        cancel: Cancellation,
        deadline: Option<std::time::Instant>,
        entry: AdmissionEntry,
        work: impl FnOnce(Cancellation) -> Result<(T, usize), MathRuntimeError> + Send + 'static,
    ) -> Result<(T, Arc<pse_columnar::AllocationLease>), MathRuntimeError> {
        self.admitted(
            cores,
            bytes,
            cancel,
            (None, deadline, false),
            Some(entry),
            work,
        )
        .await
    }
    /// Refuse a synchronous burst before its handle/task joins the bounded population.
    /// A multi-phase operation supplies one unchanged admission cutoff; the separate
    /// enclosing deadline remains the only post-dispatch execution clock.
    pub(super) fn admit_entry(
        &self,
        cores: usize,
        bytes: usize,
        cancel: &Cancellation,
        deadline: Option<std::time::Instant>,
        cutoff: Option<std::time::Instant>,
    ) -> Result<AdmissionEntry, MathRuntimeError> {
        if cores == 0 || cores > self.cores || cores > u32::MAX as usize {
            return Err(MathRuntimeError::Limit("optimizer cores"));
        }
        let construction = ConstructionEstimate::Known(bytes);
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
        let slot = match POPULATION
            .try_with(|lane| Arc::ptr_eq(&lane.jobs, &self.jobs).then(|| lane.clone()))
            .ok()
            .flatten()
        {
            Some(lane) => PopulationEntry::Borrowed {
                _usage: lane.claim()?,
            },
            None => match self.jobs.clone().try_acquire_owned() {
                Ok(ticket) => PopulationEntry::Owned { _ticket: ticket },
                Err(tokio::sync::TryAcquireError::Closed) => {
                    return Err(MathRuntimeError::Limit("native population closed"));
                }
                Err(tokio::sync::TryAcquireError::NoPermits) => PopulationEntry::Borrowed {
                    _usage: self
                        .borrow_idle_native_population()
                        .ok_or(MathRuntimeError::Limit("native jobs"))?,
                },
            },
        };
        Ok(AdmissionEntry {
            at,
            bytes,
            construction,
            slot,
            compiler: None,
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
        cancel: &Cancellation,
        deadline: Option<std::time::Instant>,
    ) -> Result<Arc<pse_columnar::AllocationLease>, MathRuntimeError> {
        self.check_entry_extent(bytes)?;
        let at = self.admission_deadline(deadline)?;
        let reservation =
            datafusion::execution::memory_pool::MemoryConsumer::new(name).register(&self.pool);
        let mut pressure = None;
        loop {
            // notify_waiters is observed even before the first poll of this future.
            let released = self.released.notified();
            tokio::pin!(released);
            released.as_mut().enable();
            if cancel.flag().load(Ordering::Acquire) {
                return Err(MathRuntimeError::Cancelled);
            }
            if std::time::Instant::now() >= at {
                return Err(Self::admission_timeout());
            }
            match reservation.try_grow(bytes) {
                Ok(()) => return Ok(pse_columnar::AllocationLease::new(reservation)),
                Err(datafusion::common::DataFusionError::ResourcesExhausted(_)) => {
                    pressure.get_or_insert_with(|| self.pressure.demand());
                    self.trim_compiler_lanes();
                    self.retire_idle_native_lanes();
                    match self.pressure.retry_growth(&reservation, bytes) {
                        Ok(()) => return Ok(pse_columnar::AllocationLease::new(reservation)),
                        Err(datafusion::common::DataFusionError::ResourcesExhausted(_)) => {}
                        Err(error) => return Err(error.into()),
                    }
                }
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
        cancel: Cancellation,
        (admitted, deadline, source_owns_deadline): (
            Option<tokio::sync::oneshot::Sender<()>>,
            Option<std::time::Instant>,
            bool,
        ),
        entry: Option<AdmissionEntry>,
        work: impl FnOnce(Cancellation) -> Result<(T, usize), MathRuntimeError> + Send + 'static,
    ) -> Result<(T, Arc<pse_columnar::AllocationLease>), MathRuntimeError> {
        self.admitted_storage(
            cores,
            bytes,
            AdmissionControl {
                cancel,
                admitted,
                deadline,
                source_owns_deadline,
            },
            entry,
            false,
            move |flag, _| work(flag),
        )
        .await
    }
    async fn admitted_storage<T: Send + 'static>(
        self: &Arc<Self>,
        cores: usize,
        bytes: usize,
        control: AdmissionControl,
        entry: Option<AdmissionEntry>,
        retain_foreign: bool,
        work: impl FnOnce(
            Cancellation,
            Option<Arc<pse_columnar::AllocationLease>>,
        ) -> Result<(T, usize), MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<(T, Arc<pse_columnar::AllocationLease>), MathRuntimeError> {
        let AdmissionControl {
            cancel,
            admitted,
            deadline,
            source_owns_deadline,
        } = control;
        let entry = match entry {
            Some(entry) => entry,
            None => self.admit_entry(cores, bytes, &cancel, deadline, None)?,
        };
        let AdmissionEntry {
            at,
            bytes,
            construction,
            slot,
            compiler,
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
        tokio::spawn(crate::invocation::inherit(crate::observation::inherit(
            crate::observation::child(
                "math.job",
                pse_operations::canonical::inherit_activity(async move {
                    let result = async {
                let (lease, active) = {
                    let _waiting = pse_diagnostics::observation::phase("math.admission.wait");
                    service.reserve_active("math:native-job", cores as u32, bytes, &cancel, at).await?
                };
                if cancel.flag().load(Ordering::Acquire) {
                    return Err(MathRuntimeError::Cancelled);
                }
                if std::time::Instant::now() >= at {
                    return Err(Self::admission_timeout());
                }
                if let Some(admitted) = admitted {
                    let _ = admitted.send(());
                }
                let flag = cancel.clone();
                let storage = if retain_foreign {
                    let foreign = service.policy.foreign_bytes;
                    if foreign == 0 || foreign > lease.size() {
                        return Err(MathRuntimeError::Limit("prepaid foreign allowance extent"));
                    }
                    Some(pse_columnar::AllocationLease::new(lease.split(foreign)))
                } else { None };
                let compute = super::staged::ComputeOwner::new(
                    service.active_jobs.clone(),
                    service.cpu.clone(),
                    cores as u32,
                    active,
                    tokio::runtime::Handle::current(),
                );
                let native_compute = compute.clone();
                let construction = Arc::new(ConstructionOwner::new(
                    lease.clone(),
                    construction,
                    service.policy.worker_bytes,
                    flag.flag(),
                    service.pressure.clone(),
                ));
                let sessions = service.policy.inner_session_bytes;
                let runtime = tokio::runtime::Handle::current();
                let observation = pse_diagnostics::observation::current();
                let invocation = crate::invocation::TransferredContext::capture();
                let dispatch = tracing::dispatcher::get_default(Clone::clone);
                let span = tracing::Span::current();
                let handle = std::thread::Builder::new()
                    .name("pse-math".into())
                    .stack_size(service.policy.stack_bytes)
                    .spawn(move || {
                        let _runtime = runtime.enter();
                        let _invocation = invocation.enter();
                        let _observation = observation.as_ref().map(pse_diagnostics::observation::ObservationContext::enter);
                        let _execution = crate::observation::native_execution_scope();
                        let _dispatch = tracing::dispatcher::set_default(&dispatch);
                        let _span = span.enter();
                        let _phase = pse_diagnostics::observation::phase("native.worker");
                        #[cfg(feature = "solver-kinsol")]
                        pse_backend_native::implicit::budget_sessions(sessions);
                        #[cfg(not(feature = "solver-kinsol"))]
                        let _ = sessions;
                        let interval = crate::invocation::stage("native.worker_execution");
                        let result = pse_backend_native::execution::compute_scoped(
                            Box::new(native_compute),
                            || pse_math::construction::scoped(construction, || work(flag, storage)),
                        );
                        interval.finish(match &result {
                            Ok(_) => crate::invocation::InvocationOutcome::Completed,
                            Err(MathRuntimeError::Cancelled) => crate::invocation::InvocationOutcome::Cancelled,
                            Err(_) => crate::invocation::InvocationOutcome::Failed,
                        });
                        result
                    })
                    .map_err(|e| MathRuntimeError::Infrastructure(e.to_string()))?;
                // Joining witnesses thread-local and native destructor completion.
                let mut joining = tokio::task::spawn_blocking(move || handle.join());
                // Inclusive supervisor wait overlaps worker execution; it is not exclusive CPU time.
                let joining_phase = pse_diagnostics::observation::phase("native.join.wait");
                let result = match deadline {
                    Some(deadline) => tokio::select! {
                        biased;
                        () = cancel.cancelled() => {
                            if let Some(context) = pse_diagnostics::observation::current() { context.boolean("native.stop_requested", true); }
                            let _ = joining.await;
                            Err(MathRuntimeError::Cancelled)
                        },
                        () = tokio::time::sleep_until(deadline.into()) => {
                            // Signal checkpoints immediately, but retain every native owner
                            // until the actual thread and its foreign destructors have joined.
                            if let Some(context) = pse_diagnostics::observation::current() { context.boolean("native.deadline_expired", true); }
                            if !source_owns_deadline { cancel.cancel(); }
                            let _ = joining.await;
                            Err(Self::admission_timeout())
                        },
                        joined = &mut joining => joined
                            .map_err(|e| MathRuntimeError::Infrastructure(e.to_string()))?
                            .map_err(|_| MathRuntimeError::Panic("native worker panic".into()))?,
                    },
                    None => joining
                        .await
                        .map_err(|e| MathRuntimeError::Infrastructure(e.to_string()))?
                        .map_err(|_| MathRuntimeError::Panic("native worker panic".into()))?,
                };
                drop(joining_phase);
                if let Some(context) = pse_diagnostics::observation::current() {
                    context.boolean("native.joined", true);
                    context.boolean("native.return.failed", result.is_err());
                }
                // Native thread, TLS and foreign destruction have joined. Release CPU
                // before optional retention yields for the escaped result's growth.
                drop(compute);
                let result = result.and_then(|(value, retained)| {
                    if cancel.flag().load(Ordering::Acquire) {
                        return Err(MathRuntimeError::Cancelled);
                    }
                    // The admission-only clock ends at dispatch. Preserve actual task expiry.
                    if deadline.is_some_and(|at| std::time::Instant::now() >= at) {
                        return Err(Self::admission_timeout());
                    }
                    if let Some(more) = retained.checked_sub(lease.size()).filter(|n| *n > 0) {
                        service.pressure.try_grow(&lease, more)?;
                    }
                    Ok((
                        value,
                        pse_columnar::AllocationLease::new(lease.split(retained)),
                    ))
                });
                drop(lease);
                result
            }
            .await;
                    // Actual native/TLS join precedes release of compiler exclusion and storage.
                    drop(compiler);
                    drop(slot);
                    let _ = tx.send(result);
                }),
            ),
        )));
        let result = rx
            .await
            .map_err(|_| MathRuntimeError::Infrastructure("lost native supervisor".into()))?;
        caller.0.take();
        result
    }
}

#[cfg(all(test, feature = "solver-idas"))]
mod idas_storage_tests;

#[cfg(test)]
mod construction_tests {
    use super::*;
    use datafusion::execution::memory_pool::{FairSpillPool, MemoryConsumer, MemoryPool};
    use pse_math::construction::ConstructionAdmission;
    #[tokio::test]
    async fn storage_aware_submission_detaches_only_prepaid_foreign_through_actual_join() {
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy {
                foreign_bytes: 1 << 20,
                ..Default::default()
            },
        );
        let baseline = service.pool.reserved();
        let handle = service
            .submit_with_storage(1, 0, Submission::ephemeral(), |_, _, storage| {
                assert_eq!(storage.size(), 1 << 20);
                Ok((storage, 32))
            })
            .unwrap();
        let (storage, result) = handle.finish().await.unwrap();
        assert_eq!(storage.size(), 1 << 20);
        assert_eq!(result.size(), 32);
        assert_eq!(
            service.pool.reserved(),
            baseline + (1 << 20) + 32,
            "joined stack/constructor ownership ends without reclaiming detached foreign bytes"
        );
        drop(result);
        assert_eq!(service.pool.reserved(), baseline + (1 << 20));
        drop(storage);
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[test]
    fn closed_cpu_refuses_even_when_jobs_are_saturated() {
        let jobs = Arc::new(pse_engine::resources::ReleaseNotifyingSemaphore::new(1));
        let cpu = Arc::new(pse_engine::resources::ReleaseNotifyingSemaphore::new(1));
        let held = jobs.clone().try_acquire_owned().unwrap();
        cpu.close();
        assert!(matches!(
            try_active(&jobs, &cpu, 1),
            Err(MathRuntimeError::Limit("CPU admission closed"))
        ));
        assert_eq!(jobs.available_permits(), 0);
        drop(held);
        assert_eq!(jobs.available_permits(), 1);
    }
    #[tokio::test]
    async fn closed_admission_refuses_while_memory_is_unavailable() {
        for close_cpu in [false, true] {
            let service = super::super::tests::service();
            let baseline = service.pool.reserved();
            let held =
                MemoryConsumer::new("test:closed-admission-held-memory").register(&service.pool);
            held.try_grow((512 << 20) - baseline).unwrap();
            let full = service.pool.reserved();
            let control = Cancellation::default();
            let worker_control = control.clone();
            let worker_service = service.clone();
            let runtime = tokio::runtime::Handle::current();
            let (registered, registration) = std::sync::mpsc::sync_channel(1);
            let (send, receive) = std::sync::mpsc::sync_channel(1);
            let join = std::thread::spawn(move || {
                use std::future::Future;
                let pending = worker_service.reserve_active(
                    "test:closed-admission",
                    1,
                    1024,
                    &worker_control,
                    std::time::Instant::now() + std::time::Duration::from_secs(10),
                );
                let mut pending = std::pin::pin!(pending);
                let mut registered = Some(registered);
                let result = runtime.block_on(futures_util::future::poll_fn(|context| {
                    let result = pending.as_mut().poll(context);
                    if result.is_pending()
                        && let Some(registered) = registered.take()
                    {
                        registered.send(()).unwrap();
                    }
                    result
                }));
                send.send(result.map(|_| ())).unwrap();
            });
            let registered = registration.recv_timeout(std::time::Duration::from_secs(1));
            if close_cpu {
                service.cpu.close();
            } else {
                service.active_jobs.close();
            }
            let result = receive.recv_timeout(std::time::Duration::from_secs(1));
            if result.is_err() {
                control.cancel();
            }
            join.join().unwrap();
            assert!(
                registered.is_ok(),
                "memory wait must be registered before closure"
            );
            assert!(
                matches!(result.unwrap(), Err(MathRuntimeError::Limit(reason))
                if reason == if close_cpu { "CPU admission closed" } else { "active jobs closed" })
            );
            assert!(!control.is_cancelled());
            assert_eq!(service.pool.reserved(), full);
            assert_eq!(service.active_jobs.available_permits(), service.policy.jobs);
            assert_eq!(service.cpu.available_permits(), service.cores);
            drop(held);
            assert_eq!(service.pool.reserved(), baseline);
        }
    }
    #[tokio::test]
    async fn quiescent_closed_admission_refuses_while_memory_is_unavailable() {
        for close_cpu in [false, true] {
            let service = super::super::tests::service();
            let baseline = service.pool.reserved();
            let held =
                MemoryConsumer::new("test:quiescent-closed-held-memory").register(&service.pool);
            held.try_grow((512 << 20) - baseline).unwrap();
            let full = service.pool.reserved();
            if close_cpu {
                service.cpu.close();
            } else {
                service.active_jobs.close();
            }
            let control = Cancellation::default();
            let worker_control = control.clone();
            let worker_service = service.clone();
            let runtime = tokio::runtime::Handle::current();
            let (send, receive) = std::sync::mpsc::sync_channel(1);
            let join = std::thread::spawn(move || {
                let _entered = runtime.enter();
                let execution =
                    pse_backend_native::solve::Execution::new(worker_control, &Default::default());
                send.send(worker_service.quiescent_phase_owned_construction(
                    1,
                    ConstructionEstimate::Known(1024),
                    &execution,
                    |_| -> () { panic!("closed admission must not execute construction") },
                ))
                .unwrap();
            });
            let result = receive.recv_timeout(std::time::Duration::from_secs(1));
            if result.is_err() {
                control.cancel();
            }
            join.join().unwrap();
            assert!(
                matches!(result.unwrap(), Err(MathRuntimeError::Limit(reason))
                if reason == if close_cpu { "CPU admission closed" } else { "active jobs closed" })
            );
            assert!(!control.is_cancelled());
            assert_eq!(service.pool.reserved(), full);
            assert_eq!(service.active_jobs.available_permits(), service.policy.jobs);
            assert_eq!(service.cpu.available_permits(), service.cores);
            drop(held);
            assert_eq!(service.pool.reserved(), baseline);
        }
    }
    #[tokio::test]
    async fn compute_resume_refuses_closed_cpu_while_jobs_remain_owned() {
        use pse_backend_native::execution::ComputeAdmission;
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let mut owner = super::super::staged::ComputeOwner::new(
            service.active_jobs.clone(),
            service.cpu.clone(),
            1,
            service.try_active(1).unwrap().unwrap(),
            tokio::runtime::Handle::current(),
        );
        owner.pause();
        let held = service
            .active_jobs
            .clone()
            .try_acquire_many_owned(service.policy.jobs as u32)
            .unwrap();
        service.cpu.close();
        let control = Cancellation::default();
        let worker_control = control.clone();
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        let join = std::thread::spawn(move || {
            let execution =
                pse_backend_native::solve::Execution::new(worker_control, &Default::default());
            send.send(owner.resume(&execution)).unwrap();
        });
        let result = receive.recv_timeout(std::time::Duration::from_secs(1));
        if result.is_err() {
            control.cancel();
        }
        join.join().unwrap();
        assert!(
            matches!(result.unwrap(), Err(pse_backend_native::ProblemError::Limit {
            kind: pse_backend_native::LimitKind::Memory, detail,
        }) if detail == "CPU admission closed")
        );
        assert!(!control.is_cancelled());
        assert_eq!(service.active_jobs.available_permits(), 0);
        assert_eq!(service.cpu.available_permits(), service.cores);
        assert_eq!(service.pool.reserved(), baseline);
        drop(held);
        assert_eq!(service.active_jobs.available_permits(), service.policy.jobs);
    }
    async fn gated_scoped_job_stops_after_native_join(explicit_cancel: bool) {
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy::default(),
        );
        let baseline = service.pool.reserved();
        let jobs = service.jobs.available_permits();
        let cores = service.cpu.available_permits();
        let cancel = Cancellation::default();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
        let (started, start) = tokio::sync::oneshot::channel();
        let (observed, observation) = tokio::sync::oneshot::channel();
        let (release, gate) = std::sync::mpsc::channel();
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let parent = hub.start("native.join.fixture");
        let worker_service = service.clone();
        let worker_cancel = cancel.clone();
        let task = tokio::spawn(pse_diagnostics::observation::scope(
            Some(parent.context()),
            async move {
                worker_service
                    .job_scoped(1, 1024, worker_cancel, Some(deadline), move |flag| {
                        let _ = started.send(());
                        let cutoff = std::time::Instant::now() + std::time::Duration::from_secs(5);
                        while !flag.load(Ordering::Acquire) {
                            if std::time::Instant::now() >= cutoff {
                                return Err(MathRuntimeError::Infrastructure(
                                    "native cancellation was not observed".into(),
                                ));
                            }
                            std::thread::sleep(std::time::Duration::from_millis(1));
                        }
                        let _ = observed.send(());
                        // Cancellation has been observed, but native teardown is deliberately held.
                        gate.recv_timeout(std::time::Duration::from_secs(10))
                            .map_err(|error| MathRuntimeError::Infrastructure(error.to_string()))?;
                        Err::<(), _>(MathRuntimeError::Cancelled)
                    })
                    .await
            },
        ));
        tokio::time::timeout(std::time::Duration::from_secs(5), start)
            .await
            .unwrap()
            .unwrap();
        if explicit_cancel {
            cancel.cancel();
        }
        tokio::time::timeout(std::time::Duration::from_secs(5), observation)
            .await
            .unwrap()
            .unwrap();
        // An earlier cancellation must remain cancellation even if join crosses the deadline.
        tokio::time::sleep_until((deadline + std::time::Duration::from_millis(20)).into()).await;
        assert!(!task.is_finished());
        let job = hub
            .list(0, 128)
            .operations
            .into_iter()
            .find(|operation| operation.kind == "math.job")
            .unwrap();
        let reader = hub.reader(job.id).unwrap();
        let stopping = reader.snapshot(std::time::Duration::from_secs(60));
        assert!(
            stopping.outcome.is_none(),
            "stop request is not native drain"
        );
        assert!(
            stopping
                .phases
                .iter()
                .any(|phase| phase.kind == "native.join.wait")
        );
        assert_eq!(service.jobs.available_permits(), jobs - 1);
        assert_eq!(service.cpu.available_permits(), cores - 1);
        assert!(service.pool.reserved() > baseline);
        release.send(()).unwrap();
        let result = task.await.unwrap();
        if explicit_cancel {
            assert!(matches!(result, Err(MathRuntimeError::Cancelled)));
        } else {
            assert!(matches!(
                result,
                Err(MathRuntimeError::Solve(
                    pse_backend_native::ProblemError::Limit {
                        kind: pse_backend_native::LimitKind::Time,
                        ..
                    }
                ))
            ));
        }
        let joined = reader.snapshot(std::time::Duration::from_secs(60));
        assert!(
            joined
                .metrics
                .iter()
                .any(|metric| metric.name == "native.joined"
                    && metric.latest == pse_diagnostics::observation::Value::Boolean(true))
        );
        assert_eq!(service.jobs.available_permits(), jobs);
        assert_eq!(service.cpu.available_permits(), cores);
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[tokio::test(flavor = "current_thread")]
    async fn scoped_deadline_signals_native_worker_and_retains_owners_until_join() {
        gated_scoped_job_stops_after_native_join(false).await;
    }
    #[tokio::test(flavor = "current_thread")]
    async fn scoped_cancellation_before_deadline_preserves_cause_through_native_join() {
        gated_scoped_job_stops_after_native_join(true).await;
    }
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
    #[tokio::test]
    async fn saturated_residents_lend_population_only_to_direct_preparation() {
        let service = super::super::tests::service();
        let mut sessions = (0..service.policy.jobs)
            .map(|_| service.open_session().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(service.jobs.available_permits(), 0);
        assert_eq!(service.active_jobs.available_permits(), service.policy.jobs);
        assert_eq!(service.cpu.available_permits(), service.cores);
        let independent = service.clone();
        sessions[0]
            .prepare(async move {
                let task =
                    tokio::spawn(async move { independent.submit(1, 0, |_, _| Ok(((), 0))) });
                assert!(matches!(
                    task.await.unwrap(),
                    Err(MathRuntimeError::Limit("native jobs"))
                ));
            })
            .await
            .unwrap();
        assert_eq!(
            service.artifact_width(),
            service.cores.min(service.policy.flights).max(1)
        );
        let borrowed = service.clone();
        let answer = sessions[0]
            .prepare(async move {
                assert_eq!(
                    borrowed.artifact_width(),
                    1,
                    "dependent cold artifact preparation owns one exclusive population loan"
                );
                borrowed
                    .job(1, 0, Cancellation::default(), |_| Ok(42))
                    .await
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(answer, 42);
        assert_eq!(service.jobs.available_permits(), 0);
        assert_eq!(service.active_jobs.available_permits(), service.policy.jobs);
        for session in sessions {
            session.close().await;
        }
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
    }
    #[tokio::test]
    async fn borrowed_population_survives_lender_close_and_abandoned_child_until_join() {
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy {
                jobs: 1,
                foreign_bytes: 1 << 20,
                ..Default::default()
            },
        );
        let mut session = service.open_session().unwrap();
        let (entered, observed) = tokio::sync::oneshot::channel();
        let (release, wait) = std::sync::mpsc::channel();
        let child_service = service.clone();
        let handle = session
            .prepare(async move {
                // Explicit single-flight propagation is the same dependent-loader seam as
                // selected admission, frontier, basis and program artifact construction.
                let population = PopulationScope::capture();
                tokio::spawn(population.run(async move {
                    child_service.submit(1, 0, move |_, _| {
                        entered.send(()).unwrap();
                        wait.recv().unwrap();
                        Ok(((), 0))
                    })
                }))
                .await
                .unwrap()
            })
            .await
            .unwrap()
            .unwrap();
        observed.await.unwrap();
        let captured = session.prepare(async { PopulationScope::capture() }).await;
        assert!(captured.is_err()); // A live child still owns this lane.
        let mut close = Box::pin(session.close());
        assert!(futures_util::poll!(close.as_mut()).is_pending());
        assert_eq!(service.jobs.available_permits(), 0);
        drop(handle);
        assert_eq!(service.jobs.available_permits(), 0);
        release.send(()).unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(1), close)
            .await
            .unwrap();
        let drained = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            service.jobs.clone().acquire_owned(),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(service.active_jobs.available_permits(), service.policy.jobs);
        assert_eq!(service.cpu.available_permits(), service.cores);
        drop(drained);
    }
    #[tokio::test]
    async fn population_loan_is_exclusive_and_closed_capture_cannot_restart() {
        let service = super::super::tests::service();
        let mut session = service.open_session().unwrap();
        let scope = session
            .prepare(async { PopulationScope::capture() })
            .await
            .unwrap();
        let lane = scope.0.as_ref().unwrap();
        let usage = lane.claim().unwrap();
        assert!(matches!(
            lane.claim(),
            Err(MathRuntimeError::Limit("concurrent native population loan"))
        ));
        drop(usage);
        session.close().await;
        let child = service.clone();
        let refused = scope
            .run(async move { child.submit(1, 0, |_, _| Ok(((), 0))) })
            .await;
        assert!(matches!(
            refused,
            Err(MathRuntimeError::Limit("native population lane closed"))
        ));
    }
    #[tokio::test]
    async fn unavailable_active_stage_releases_tentative_memory_and_population_on_cancel() {
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let active = service
            .active_jobs
            .clone()
            .acquire_many_owned(service.policy.jobs as u32)
            .await
            .unwrap();
        let control = Cancellation::default();
        let stop = control.clone();
        let released = service.released.notified();
        tokio::pin!(released);
        released.as_mut().enable();
        let pending_service = service.clone();
        let pending = tokio::spawn(async move {
            pending_service
                .job(1, 1024, control, |_| -> Result<(), MathRuntimeError> {
                    panic!("undispatched operation")
                })
                .await
        });
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            while service.jobs.available_permits() == service.policy.jobs {
                assert!(
                    !pending.is_finished(),
                    "operation finished before population admission"
                );
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("population admission must become observable");
        tokio::time::timeout(std::time::Duration::from_secs(1), released)
            .await
            .expect("the unavailable stage must release its tentative memory");
        assert!(!pending.is_finished());
        assert_eq!(service.pool.reserved(), baseline);
        assert_eq!(service.cpu.available_permits(), service.cores);
        stop.cancel();
        assert!(matches!(
            tokio::time::timeout(std::time::Duration::from_secs(1), pending)
                .await
                .expect("active-stage waiting must observe cancellation")
                .unwrap(),
            Err(MathRuntimeError::Cancelled)
        ));
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
        drop(active);
    }
    #[tokio::test]
    async fn unavailable_active_stage_preserves_original_deadline_without_cancelling() {
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let active = service
            .active_jobs
            .clone()
            .try_acquire_many_owned(service.policy.jobs as u32)
            .unwrap();
        let control = Cancellation::default();
        let at = std::time::Instant::now() + std::time::Duration::from_millis(20);
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            service.reserve_active("test:blocked-active-deadline", 1, 1024, &control, at),
        )
        .await
        .expect("active-stage retries must retain the original deadline");
        assert!(matches!(
            result,
            Err(MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Limit {
                    kind: pse_backend_native::LimitKind::Time,
                    ..
                }
            ))
        ));
        assert!(!control.flag().load(Ordering::Acquire));
        assert_eq!(service.pool.reserved(), baseline);
        assert_eq!(service.cpu.available_permits(), service.cores);
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
        assert_eq!(service.active_jobs.available_permits(), 0);
        drop(active);
    }
    #[tokio::test]
    async fn contending_quiescent_fit_teams_release_estimation_and_preserve_population() {
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let phase_service = service.clone();
            let barrier = barrier.clone();
            handles.push(
                service
                    .submit(1, 0, move |flag, _| {
                        let execution =
                            pse_backend_native::solve::Execution::new(flag, &Default::default());
                        barrier.wait();
                        let observed =
                            phase_service.quiescent_phase(2, 4096, &execution, || {
                                assert_eq!(phase_service.cpu.available_permits(), 0);
                                assert!(phase_service.pool.reserved() >= baseline + 4096);
                                // Both fit populations remain finite while the nested team borrows its parent's ticket.
                                assert!((1..=2).contains(
                                    &(phase_service.policy.jobs
                                        - phase_service.jobs.available_permits())
                                ));
                                7
                            })?;
                        Ok((observed, 0))
                    })
                    .unwrap(),
            );
        }
        let results = tokio::time::timeout(std::time::Duration::from_secs(2), async {
            let mut results = Vec::new();
            for handle in handles {
                results.push(handle.finish().await.unwrap().0);
            }
            results
        })
        .await
        .unwrap();
        assert_eq!(results, vec![7, 7]);
        assert_eq!(service.pool.reserved(), baseline);
        assert_eq!(service.cpu.available_permits(), service.cores);
        assert_eq!(service.active_jobs.available_permits(), service.policy.jobs);
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
    }
    #[tokio::test]
    async fn quiescent_full_team_retry_allows_smaller_ready_work() {
        let service = super::super::tests::service();
        let occupied = service.cpu.clone().acquire_owned().await.unwrap();
        let phase = service.clone();
        let (entered, observed) = tokio::sync::oneshot::channel();
        let parent = service
            .submit(1, 0, move |flag, _| {
                entered.send(()).unwrap();
                let execution =
                    pse_backend_native::solve::Execution::new(flag, &Default::default());
                phase.quiescent_phase(2, 4096, &execution, || ())?;
                Ok(((), 0))
            })
            .unwrap();
        let mut parent = Box::pin(parent.finish());
        assert!(futures_util::poll!(parent.as_mut()).is_pending());
        observed.await.unwrap();
        let small = service.submit(1, 0, |_, _| Ok((19, 0))).unwrap();
        assert_eq!(
            tokio::time::timeout(std::time::Duration::from_secs(1), small.finish())
                .await
                .unwrap()
                .unwrap()
                .0,
            19
        );
        assert!(futures_util::poll!(parent.as_mut()).is_pending());
        drop(occupied);
        tokio::time::timeout(std::time::Duration::from_secs(1), parent)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(service.cpu.available_permits(), service.cores);
        assert_eq!(service.active_jobs.available_permits(), service.policy.jobs);
    }
    #[test]
    fn passive_worker_allocation_retains_idle_charge_without_original_task_admission() {
        let pool: Arc<dyn MemoryPool> = Arc::new(FairSpillPool::new(4096));
        let lane = WorkerBudget::drawing(1024, &pool);
        let stop = Cancellation::default();
        let old_stop = Arc::downgrade(&stop);
        let admission = super::super::strategy::admission::TaskAdmission::new(
            pse_model::strategy::WorkLimits {
                attempts: 1,
                evaluations: None,
                iterations: None,
                factorizations: None,
                proof_steps: None,
            },
            pse_kernels::ExecutionScope::new(stop.clone(), None),
            None,
            false,
        );
        let original = Arc::downgrade(&admission);
        let budget = lane.with_admission(admission.clone());
        let charge = budget.charge(256).unwrap();
        let callback = charge.share();
        assert!(!callback.exclusive());
        drop(charge);
        drop(budget);
        drop(admission);
        drop(stop);
        assert!(original.upgrade().is_none());
        assert!(old_stop.upgrade().is_none());
        assert!(callback.exclusive());
        assert_eq!(lane.used(), 256);
        assert_eq!(pool.reserved(), 256);
        assert!(callback.compatible(&lane, 256));
        let other = WorkerBudget::drawing(1024, &pool);
        assert!(!callback.compatible(&other, 256));
        let overlap = callback.reset_overlap().unwrap();
        assert_eq!(pool.reserved(), 512);
        drop(overlap);
        assert_eq!(pool.reserved(), 256);
        drop(callback);
        assert_eq!(lane.used(), 0);
        assert_eq!(pool.reserved(), 0);
        let small = WorkerBudget::drawing(300, &pool);
        let held = small.charge(256).unwrap();
        assert!(held.reset_overlap().is_err());
        assert_eq!(small.used(), 256);
        drop(held);
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn constructor_growth_uses_same_reservation_and_checks_worker_capacity() {
        let pool: Arc<dyn MemoryPool> = Arc::new(FairSpillPool::new(32));
        let reservation = Arc::new(MemoryConsumer::new("constructor test").register(&pool));
        reservation.try_grow(8).unwrap();
        let owner = ConstructionOwner {
            reservation: reservation.clone(),
            initial: 4,
            prepaid: 0,
            capacity: 12,
            additional: std::sync::Mutex::new(0),
            cancelled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pressure: super::super::retention::Pressure::new(0),
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
    fn constructor_opaque_envelope_covers_reports_without_double_charging() {
        // The entire pool is already reserved: another growth cannot succeed.
        let pool: Arc<dyn MemoryPool> = Arc::new(FairSpillPool::new(16));
        let reservation = Arc::new(MemoryConsumer::new("opaque constructor test").register(&pool));
        reservation.try_grow(16).unwrap();
        let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let owner = ConstructionOwner {
            reservation: reservation.clone(),
            initial: 0,
            prepaid: 16,
            capacity: 16,
            additional: std::sync::Mutex::new(0),
            cancelled: cancelled.clone(),
            pressure: super::super::retention::Pressure::new(0),
        };
        owner.try_grow(7).unwrap();
        owner.try_grow(9).unwrap();
        assert_eq!(reservation.size(), 16);
        assert_eq!(pool.reserved(), 16);
        assert!(matches!(
            owner.try_grow(1),
            Err(pse_math::MathError::ByteLimit {
                resource: "constructor worker bytes",
                required: 17,
                available: 16,
            })
        ));
        assert_eq!(*owner.additional.lock().unwrap(), 16);
        cancelled.store(true, Ordering::Release);
        assert!(matches!(
            owner.try_grow(0),
            Err(pse_math::MathError::Cancelled)
        ));
        assert_eq!(pool.reserved(), 16);
        drop(owner);
        drop(reservation);
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn resident_opaque_constructor_precharges_and_follows_last_worker_owner() {
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let execution = pse_backend_native::solve::Execution::new(
            Cancellation::default(),
            &pse_backend_native::solve::Controls::default(),
        );
        let owner = service
            .reserve_constructor(
                "test:resident-constructor",
                ConstructionEstimate::Opaque(16),
                &execution,
            )
            .unwrap();
        assert_eq!(service.pool.reserved(), baseline + 16);
        owner.try_grow(7).unwrap();
        owner.try_grow(9).unwrap();
        assert_eq!(service.pool.reserved(), baseline + 16);
        assert!(owner.try_grow(1).is_err());
        let worker = owner.clone();
        drop(owner);
        assert_eq!(service.pool.reserved(), baseline + 16);
        drop(worker);
        assert_eq!(service.pool.reserved(), baseline);
        let held = MemoryConsumer::new("test:resident-refusal").register(&service.pool);
        held.try_grow((512 << 20) - baseline).unwrap();
        assert!(
            service
                .reserve_constructor(
                    "test:refused-constructor",
                    ConstructionEstimate::Opaque(16),
                    &execution
                )
                .is_err()
        );
        assert_eq!(service.pool.reserved(), 512 << 20);
        drop(held);
        assert_eq!(service.pool.reserved(), baseline);
        execution.cancel.store(true, Ordering::Release);
        assert!(matches!(
            service.reserve_constructor(
                "test:cancelled-constructor",
                ConstructionEstimate::Opaque(16),
                &execution
            ),
            Err(MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Cancelled
            ))
        ));
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[tokio::test]
    async fn phase_constructor_uses_its_reservation_and_restores_parent_owner() {
        use pse_kernels::DerivativeOrder;
        use pse_math::typed::{Binary, BodyBuilder, BodyLimits};
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let scalar = match registry.physical_name("Scalar").unwrap() {
            pse_quantity::PhysicalName::QuantityType(id) => id,
            other => panic!("Scalar must resolve to the admitted quantity type: {other:?}"),
        };
        let mut builder = BodyBuilder::new(
            pse_math::initialize().unwrap(),
            &registry,
            &pse_quantity::standard::StandardInvariantChecker,
            1,
            BodyLimits::default(),
        )
        .unwrap();
        let input = builder
            .input(
                0,
                scalar,
                pse_quantity::IndexSet::new(),
                pse_ids::SemanticId::NIL,
            )
            .unwrap();
        let output = builder
            .binary(
                Binary::Add,
                input.clone(),
                input,
                None,
                pse_ids::SemanticId::NIL,
            )
            .unwrap();
        let body = builder.prepare(&[output]).unwrap();
        let limits = pse_math::jets::EvaluationLimits::default();
        let known = body
            .arithmetic_compilation_allocation_bound(DerivativeOrder::Value, limits)
            .unwrap()
            .unwrap();
        let capacity = 1 << 20;
        assert!(known < capacity);
        let (service, _cache) = super::super::tests::service_with_policy(
            64 << 20,
            super::super::MathPolicy {
                worker_bytes: capacity,
                foreign_bytes: 1 << 20,
                inner_session_bytes: 0,
                ..Default::default()
            },
        );
        let baseline = service.pool.reserved();
        let phase_service = service.clone();
        service
            .job(1, capacity, Cancellation::default(), move |flag| {
                let execution =
                    pse_backend_native::solve::Execution::new(flag.clone(), &Default::default());
                let parent = phase_service.pool.reserved();
                let compile = || {
                    body.compile(
                        &[0],
                        &[0],
                        DerivativeOrder::Value,
                        pse_math::library::Optimization::default(),
                        limits,
                        &pse_kernels::ExecutionScope::new(flag.clone(), None),
                    )
                };
                for construction in [
                    ConstructionEstimate::Known(known),
                    ConstructionEstimate::Opaque(capacity),
                ] {
                    phase_service.quiescent_phase_owned_construction(
                        1,
                        construction,
                        &execution,
                        |reservation| {
                            let compiled = compile().unwrap();
                            match construction {
                                ConstructionEstimate::Known(bytes) => {
                                    assert!(reservation.size() > bytes);
                                }
                                ConstructionEstimate::Opaque(bytes) => {
                                    assert_eq!(reservation.size(), bytes);
                                }
                            }
                            drop(compiled);
                        },
                    )?;
                    assert_eq!(phase_service.pool.reserved(), parent);
                }
                // The parent has no growth headroom. A restored parent must refuse
                // the exact scalar cache report instead of using the departed phase.
                let refused = compile().unwrap_err();
                assert!(refused.to_string().contains("constructor worker bytes"));
                Ok(())
            })
            .await
            .unwrap();
        assert_eq!(service.pool.reserved(), baseline);
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
            prepaid: 0,
            capacity: 64,
            additional: std::sync::Mutex::new(0),
            cancelled: cancelled.clone(),
            pressure: super::super::retention::Pressure::new(0),
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
