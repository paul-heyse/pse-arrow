// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The native half of the staged-sequence primitive (A6; architecture §4): one
//! worker-owned session that keeps native state across the steps of a sequence.
//!
//! A session owns one native thread for its whole life, and the sequence's [`Retained`]
//! native state never leaves it. Each step is admitted on its own: it takes its CPU permits
//! for the step only, so the driver that plans the next step, prepares structure or rebinds
//! values in between never waits on a permit the idle session holds. The session keeps its
//! job slot, stack and worker share until it closes; closing joins the thread, which
//! witnesses native and thread-local destruction.
use super::{MathRuntimeError, MathService, WorkerBudget};
mod driver;
#[cfg(any(test, feature = "solver-kinsol"))]
pub(crate) use driver::LocalOperation;
use pse_backend_native::{
    ProblemError,
    execution::{self, BackendExecution, Retained},
    solve::{Backend, Progress},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

#[cfg(any(all(test, feature = "solver-ipopt"), feature = "canonical-tests"))]
type NativeEntry = Arc<dyn Fn(&AtomicBool) + Send + Sync>;
#[cfg(any(all(test, feature = "solver-ipopt"), feature = "canonical-tests"))]
tokio::task_local! { static NATIVE_ENTRY: NativeEntry; }
#[cfg(feature = "canonical-tests")]
static RECEIVER_ENTRY: std::sync::OnceLock<NativeEntry> = std::sync::OnceLock::new();
/// Install one qualified receiver observer before any scientific work is issued.
/// The process association spans spawned run owners; local controls take precedence.
///
/// # Errors
/// An existing installation belongs to the process's original qualification controller.
#[cfg(feature = "canonical-tests")]
#[doc(hidden)]
pub fn install_qualified_native_entry(
    observer: Arc<dyn Fn(&AtomicBool) + Send + Sync>,
) -> Result<(), MathRuntimeError> {
    RECEIVER_ENTRY.set(observer).map_err(|_| {
        MathRuntimeError::Infrastructure("native qualification observer already installed".into())
    })
}
/// Observe actual admitted native session entries for composition controls. The
/// callback crosses into the owner thread; it must cooperate with the stop flag.
#[cfg(any(all(test, feature = "solver-ipopt"), feature = "canonical-tests"))]
#[doc(hidden)]
#[cfg_attr(
    not(feature = "canonical-tests"),
    allow(
        unreachable_pub,
        reason = "the same observer is publicly re-exported only for canonical qualification controllers"
    )
)]
pub async fn observed_native_entry<T>(observer: NativeEntry, work: impl Future<Output = T>) -> T {
    NATIVE_ENTRY.scope(observer, work).await
}

struct AccuracySignal<T> {
    original_satisfied: bool,
    goals: Vec<pse_math::engineering_accuracy::GoalResult>,
    stop: Option<super::strategy::AccuracyStop<T>>,
}

fn accuracy_assessor<T: Send + 'static, F>(
    owner: Arc<std::sync::Mutex<F>>,
    signal: Arc<std::sync::Mutex<AccuracySignal<T>>>,
) -> impl FnMut(
    &super::solves::Outcome,
    Option<&pse_kernels::ExecutionScope>,
    &Arc<WorkerBudget>,
) -> super::strategy::Assessed<T>
+ Send
+ 'static
where
    F: FnMut(
            &super::solves::Outcome,
            Option<&pse_kernels::ExecutionScope>,
            &Arc<WorkerBudget>,
        ) -> super::strategy::Assessed<T>
        + Send
        + 'static,
{
    move |outcome, scope, budget| {
        let assessed = match owner.lock() {
            Ok(mut assess) => assess(outcome, scope, budget),
            Err(poisoned) => {
                let mut assess = poisoned.into_inner();
                let rejected = super::solves::Outcome::Rejected(Arc::new(MathRuntimeError::Panic(
                    "scientific accuracy assessment owner panicked".into(),
                )));
                assess(&rejected, scope, budget)
            }
        };
        // A poisoned signal is explicitly refused by the session before further
        // work. Its stale contents can never authorize another native operation.
        if let Ok(mut signal) = signal.lock() {
            signal.original_satisfied = assessed.original.satisfied();
            signal.goals = assessed.accuracy.clone();
            signal.stop = assessed.accuracy_stop;
        }
        assessed
    }
}

/// A completed native resource stop remains an observable failed result. An expired
/// driver scope can retain its current assessed native time stop, but never revive an
/// earlier scientific result or grant candidate permission.
fn retains_stopped_native_report(
    current_assessment: bool,
    last: Option<&super::strategy::AutoObservation>,
    cause: &Arc<ProblemError>,
    report: &pse_backend_native::solve::SolveReport,
    events: &[super::strategy::Event],
) -> bool {
    use pse_model::{
        generated::enums::{CandidateUse, NumericalEventKind},
        strategy::Phase,
    };
    let driver_deadline = matches!(
        cause.as_ref(),
        ProblemError::Provider(pse_kernels::ProviderError::Deadline)
    ) && report.termination.category
        == pse_backend_native::solve::Termination::TimeLimit;
    if !current_assessment
        || (!driver_deadline
            && report.termination.category
                != pse_backend_native::solve::Termination::ResourceExhausted)
    {
        return false;
    }
    let Some(last) = last else {
        return false;
    };
    let Some(original_cause) = last
        .original
        .as_ref()
        .and_then(super::strategy::OriginalConclusion::cause)
    else {
        return false;
    };
    if !matches!(
        last.permission,
        Some(CandidateUse::Unusable | CandidateUse::SeedOnly)
    ) || last.native != super::strategy::observe_native(report)
        || (!driver_deadline && !Arc::ptr_eq(&original_cause, cause))
    {
        return false;
    }
    let Some(assessed) = events.last() else {
        return false;
    };
    assessed.kind == NumericalEventKind::Finished
        && assessed.phase == Phase::Assessment
        && assessed.observation == Some(last.native)
        && assessed.permission == last.permission
        && assessed
            .original
            .as_ref()
            .and_then(super::strategy::OriginalConclusion::cause)
            .is_some_and(|original| Arc::ptr_eq(&original, &original_cause))
        && events.iter().any(|event| {
            event.mechanism == assessed.mechanism && event.kind == NumericalEventKind::Started
        })
}

/// Finalize a stopped automatic driver from already observed evidence only.
fn finish_automatic_stop<T>(
    cause: Arc<ProblemError>,
    current_assessment: bool,
    last: Option<&super::strategy::AutoObservation>,
    last_result: Option<(super::solves::Outcome, T, super::solves::StrategyTrace)>,
    trace: super::solves::StrategyTrace,
) -> Result<(super::solves::Outcome, T, super::solves::StrategyTrace), MathRuntimeError> {
    let preserve = last_result.as_ref().is_some_and(|(outcome, _, actual)| {
        matches!(outcome, super::solves::Outcome::Native(report)
            if retains_stopped_native_report(current_assessment, last, &cause, report, &actual.events))
    });
    if preserve {
        let (outcome, product, _) = last_result.ok_or_else(|| {
            ProblemError::Internal("assessed native stop result disappeared".into())
        })?;
        return Ok((outcome, product, trace));
    }
    Err(MathRuntimeError::Strategy {
        cause: Arc::new(
            ProblemError::Math(pse_math::MathError::Typed {
                retained: cause.retained_bytes(),
                cause: pse_model::diagnostic::DiagnosticCause::from_shared(cause),
            })
            .into(),
        ),
        trace,
    })
}

/// Admission of one escaping native session, separate from scientific candidate use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionDisposition {
    /// The adapter's keyed session may remain available for a compatible next attempt.
    RetainCompatible,
    /// No session product is admitted, even when the candidate can be used.
    Discard,
}
/// Assessment supplies composed permission and an independent artifact decision.
#[derive(Clone, Debug)]
pub(crate) struct StepRetention {
    pub(crate) candidate: crate::workflow::numerics::CandidateDecision,
    pub(crate) session: SessionDisposition,
}
impl StepRetention {
    pub(crate) const fn permits_session(&self) -> bool {
        self.candidate.permits_use() && matches!(self.session, SessionDisposition::RetainCompatible)
    }
    fn apply(&self, retained: &mut Retained) {
        if !self.permits_session() {
            // Independently admitted response factors have their own validity and owner.
            retained.clear();
        }
    }
}

/// One step's work on the session thread; it reports its own result.
type Work = Box<dyn FnOnce(Result<&mut Retained, ProblemError>) + Send>;
/// The same runtime CPU allowance may be relinquished only during native exclusion
/// admission. It stays on the native owner thread and is reacquired against the
/// actual execution clock before the guarded native operation begins.
#[derive(Clone, Debug)]
pub(super) struct ComputeOwner {
    jobs: Arc<tokio::sync::Semaphore>,
    cpu: Arc<tokio::sync::Semaphore>,
    cores: u32,
    permit: Arc<std::sync::Mutex<Option<super::jobs::ActivePermits>>>,
    runtime: tokio::runtime::Handle,
}
impl ComputeOwner {
    pub(super) fn new(
        jobs: Arc<tokio::sync::Semaphore>,
        cpu: Arc<tokio::sync::Semaphore>,
        cores: u32,
        permit: super::jobs::ActivePermits,
        runtime: tokio::runtime::Handle,
    ) -> Self {
        Self {
            jobs,
            cpu,
            cores,
            permit: Arc::new(std::sync::Mutex::new(Some(permit))),
            runtime,
        }
    }
}
impl execution::ComputeAdmission for ComputeOwner {
    fn pause(&mut self) {
        self.permit
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }
    fn try_resume(
        &mut self,
        execution: &pse_backend_native::solve::Execution,
    ) -> Result<execution::ComputeAvailability, ProblemError> {
        execution.check()?;
        let mut held = self
            .permit
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if held.is_some() {
            return Ok(execution::ComputeAvailability::Acquired);
        }
        *held = super::jobs::try_active(&self.jobs, &self.cpu, self.cores)
            .map_err(MathRuntimeError::into_problem)?;
        execution.check().inspect_err(|_| {
            held.take();
        })?;
        Ok(if held.is_some() {
            execution::ComputeAvailability::Acquired
        } else {
            execution::ComputeAvailability::Unavailable
        })
    }
    fn resume(
        &mut self,
        execution: &pse_backend_native::solve::Execution,
    ) -> Result<(), ProblemError> {
        // Retry complete pairs, without a queued large CPU request blocking smaller work.
        let runtime = self.runtime.clone();
        runtime.block_on(async {
            loop {
                if self.try_resume(execution)? == execution::ComputeAvailability::Acquired {
                    return Ok(());
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
    }
}
struct OperationContext {
    #[cfg(any(test, feature = "solver-kinsol"))]
    lane: Arc<super::jobs::PopulationLane>,
    _usage: Option<super::jobs::PopulationUse>,
    #[cfg(any(test, feature = "solver-kinsol"))]
    runtime: tokio::runtime::Handle,
}
// The only reusable scratch owner is the actual native session thread. Nothing
// inherits this association through Tokio tasks or crosses a worker-thread boundary.
thread_local! {
    static SCRATCH_LANE: std::cell::RefCell<Option<ScratchLane>> = const { std::cell::RefCell::new(None) };
}
#[derive(Clone)]
pub(super) struct ScratchOrigin {
    lane: Arc<()>,
    request: u64,
    thread: std::thread::ThreadId,
}
struct ScratchEntry {
    origin: ScratchOrigin,
    case: Arc<super::ExecutableCase>,
    scratch: pse_math::assembly::CaseScratch,
    charge: super::WorkerCharge,
}
struct ScratchLane {
    identity: Arc<()>,
    request: u64,
    completed: u64,
    active: bool,
    enabled: bool,
    pending: Option<ScratchEntry>,
    ready: Option<ScratchEntry>,
}
impl ScratchLane {
    fn new() -> Self {
        Self {
            identity: Arc::new(()),
            request: 0,
            completed: 0,
            active: false,
            enabled: true,
            pending: None,
            ready: None,
        }
    }
    fn clear(&mut self) {
        self.enabled = false;
        self.pending = None;
        self.ready = None;
    }
    fn begin(&mut self) {
        // A finite session cannot realistically exhaust this counter; refusing
        // further recycling at saturation preserves ownership without wrapping.
        if let Some(next) = self.request.checked_add(1) {
            self.request = next;
            self.active = true;
            self.enabled = true;
        } else {
            self.clear();
        }
    }
    fn finish(&mut self) {
        self.active = false;
        self.completed = self.request;
        let promotable = self.pending.as_ref().is_some_and(|entry| {
            self.enabled && entry.origin.request <= self.completed && entry.charge.exclusive()
        });
        if !promotable {
            return;
        }
        let Some(entry) = self.pending.take() else {
            return;
        };
        let ScratchEntry {
            origin,
            case,
            scratch,
            charge,
        } = entry;
        // Reserve all reset overlap before Symbolica allocates a replacement stack.
        // Refusal/panic destroys the candidate and leaves no old controls alive.
        let Ok(overlap) = charge.reset_overlap() else {
            return;
        };
        let reset = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| scratch.reset()));
        drop(overlap);
        if let Ok(Ok(scratch)) = reset {
            self.ready = Some(ScratchEntry {
                origin,
                case,
                scratch,
                charge,
            });
        }
    }
}
struct ScratchLaneGuard;
impl ScratchLaneGuard {
    fn install() -> Self {
        SCRATCH_LANE.with(|slot| *slot.borrow_mut() = Some(ScratchLane::new()));
        Self
    }
}
impl Drop for ScratchLaneGuard {
    fn drop(&mut self) {
        SCRATCH_LANE.with(|slot| {
            slot.borrow_mut().take();
        });
    }
}
pub(super) fn scratch_origin() -> Option<ScratchOrigin> {
    SCRATCH_LANE.with(|slot| {
        let lane = slot.borrow();
        lane.as_ref()
            .filter(|lane| lane.active && lane.enabled)
            .map(|lane| ScratchOrigin {
                lane: lane.identity.clone(),
                request: lane.request,
                thread: std::thread::current().id(),
            })
    })
}
pub(super) fn take_scratch(
    origin: &ScratchOrigin,
    case: &Arc<super::ExecutableCase>,
    budget: &WorkerBudget,
    bytes: usize,
) -> Option<(pse_math::assembly::CaseScratch, super::WorkerCharge)> {
    SCRATCH_LANE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let lane = slot.as_mut()?;
        if !lane.active
            || !lane.enabled
            || !Arc::ptr_eq(&lane.identity, &origin.lane)
            || origin.request != lane.request
            || origin.thread != std::thread::current().id()
        {
            return None;
        }
        let entry = lane.ready.take()?;
        if entry.origin.request >= origin.request
            || !entry.charge.exclusive()
            || !Arc::ptr_eq(case, &entry.case)
            || !entry.scratch.compatible(&case.assembly)
            || !entry.charge.compatible(budget, bytes)
        {
            return None;
        }
        Some((entry.scratch, entry.charge))
    })
}
pub(super) fn yield_idle_scratch(budget: &WorkerBudget) -> bool {
    SCRATCH_LANE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(lane) = slot.as_mut() else {
            return false;
        };
        if lane
            .ready
            .as_ref()
            .is_some_and(|entry| entry.charge.belongs_to(budget))
        {
            lane.ready = None;
            true
        } else {
            false
        }
    })
}
pub(super) fn return_scratch(
    origin: ScratchOrigin,
    case: Arc<super::ExecutableCase>,
    scratch: pse_math::assembly::CaseScratch,
    charge: super::WorkerCharge,
) {
    if origin.thread != std::thread::current().id() {
        return;
    }
    // A teardown after lane retirement or on another thread destroys storage. The
    // recycler never creates a new lane, task registration or permission source.
    let _ = SCRATCH_LANE.try_with(|slot| {
        if let Ok(mut slot) = slot.try_borrow_mut()
            && let Some(lane) = slot
                .as_mut()
                .filter(|lane| lane.enabled && Arc::ptr_eq(&lane.identity, &origin.lane))
        {
            lane.pending = Some(ScratchEntry {
                origin,
                case,
                scratch,
                charge,
            });
        }
    });
}
struct Request {
    /// Admitted native threads of this step, which size adapter scopes.
    threads: usize,
    /// Adapter whose scope the step's native state must live in.
    backends: Vec<Backend>,
    scope_owner: Option<Arc<pse_columnar::AllocationLease>>,
    work: Work,
}
/// Adapter scopes currently entered on the session thread.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Scope {
    backends: Vec<Backend>,
    threads: usize,
}
impl Scope {
    fn admits(&self, request: &Request) -> bool {
        request.threads == self.threads && request.backends == self.backends
    }
    fn extended(&self, request: &Request) -> Self {
        // Scope affinity is exact. A previous profile's pools must not accumulate
        // behind an unrelated request or inflate its admitted construction extent.
        Self { backends: request.backends.clone(), threads: request.threads }
    }
}
/// Serve steps until the session closes. Native state is built and torn down inside the
/// scopes of the adapters that own it; a step needing another scope ends the current one
/// (dropping retained state there) and re-enters with the extended set.
fn serve(
    receiver: &std::sync::Mutex<mpsc::Receiver<Request>>,
    pool: &Arc<dyn datafusion::execution::memory_pool::MemoryPool>,
    pressure: &super::retention::Pressure,
    stack: usize,
    current: &std::sync::Mutex<Scope>,
) {
    let mut pending: Option<Request> = None;
    let mut scope = Scope {
        backends: Vec::new(),
        threads: 1,
    };
    loop {
        *current
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = scope.clone();
        let adapters: Vec<&dyn BackendExecution> = scope
            .backends
            .iter()
            .map(|b| execution::adapter(*b))
            .collect();
        let entered = scope.clone();
        // These scopes survive while the session is idle. Backend identity deduplication
        // does not merge separately created nested teams. Hold their stack allowance
        // outside `scoped`, whose scoped pools join all threads before it returns.
        let team_owner =
            execution::scope_stack_bytes(&adapters, scope.threads, stack).and_then(|bytes| {
                match pending
                    .as_mut()
                    .and_then(|request| request.scope_owner.take())
                {
                    Some(owner) if owner.size() >= bytes => Ok(owner),
                    Some(_) => Err(ProblemError::Contract(
                        "native scope exceeds pre-admitted stacks".into(),
                    )),
                    None if bytes == 0 => Ok(pse_columnar::AllocationLease::new(
                        datafusion::execution::memory_pool::MemoryConsumer::new("math:native-scope-stacks").register(pool))),
                    None => Err(ProblemError::Contract(
                        "native scope has no pre-admitted stacks".into(),
                    )),
                }
            });
        let exit = match team_owner {
            Ok(_team_owner) => execution::scoped(&adapters, scope.threads, stack, || {
                let _scratch_lane = ScratchLaneGuard::install();
                let mut retained = Retained::default();
                loop {
                    if pending.is_none() && pressure.active() {
                        SCRATCH_LANE.with(|slot| {
                            if let Some(lane) = slot.borrow_mut().as_mut() {
                                lane.clear();
                            }
                        });
                        retained.clear();
                        retained.release();
                        if entered.threads != 1 || !entered.backends.is_empty() {
                            return Ok(Some(Scope {
                                backends: Vec::new(),
                                threads: 1,
                            }));
                        }
                    }
                    let mut request = match pending.take() {
                        Some(request) => request,
                        // Only this thread receives; the lock lends the receiver to the scope.
                        None => match receiver
                            .lock()
                            .map(|r| r.recv_timeout(std::time::Duration::from_millis(10)))
                        {
                            Ok(Ok(request)) => request,
                            Ok(Err(mpsc::RecvTimeoutError::Timeout)) => {
                                if execution::scheduler_waiting() {
                                    retained.relinquish_scheduler();
                                }
                                continue;
                            }
                            Ok(Err(mpsc::RecvTimeoutError::Disconnected)) | Err(_) => {
                                return Ok::<_, ProblemError>(None);
                            }
                        },
                    };
                    if !entered.admits(&request) {
                        let next = entered.extended(&request);
                        pending = Some(request);
                        return Ok(Some(next));
                    }
                    // A ready next request must not indefinitely retain the previous
                    // step's idle reader ahead of an already waiting Uno owner.
                    if execution::scheduler_waiting() {
                        retained.relinquish_scheduler();
                    }
                    drop(request.scope_owner.take());
                    SCRATCH_LANE.with(|slot| {
                        if let Some(lane) = slot.borrow_mut().as_mut() {
                            lane.begin();
                            if pressure.active() {
                                lane.clear();
                            }
                        }
                    });
                    (request.work)(Ok(&mut retained));
                    SCRATCH_LANE.with(|slot| {
                        if let Some(lane) = slot.borrow_mut().as_mut() {
                            if pressure.active() {
                                lane.clear();
                            } else {
                                lane.finish();
                            }
                        }
                    });
                }
            }),
            Err(error) => Err(error),
        };
        match exit {
            Ok(None) => return,
            Ok(Some(next)) => scope = next,
            Err(error) => {
                // The scope could not be entered: refuse the step that needed it and serve
                // on outside every adapter scope, which cannot fail to enter.
                if let Some(request) = pending.take() {
                    (request.work)(Err(error));
                }
                scope = Scope {
                    backends: Vec::new(),
                    threads: 1,
                };
            }
        }
    }
}
/// Stops an admitted step when its caller stops waiting for it.
struct Stop(Option<Arc<AtomicBool>>);
impl Drop for Stop {
    fn drop(&mut self) {
        if let Some(flag) = &self.0 {
            flag.store(true, Ordering::Release);
        }
    }
}
/// A worker-owned native session shared by the steps of one staged sequence.
#[derive(Debug)]
pub(crate) struct NativeSession {
    population: Arc<super::jobs::PopulationLane>,
    dispatch: Arc<tokio::sync::Semaphore>,
    scope: Arc<std::sync::Mutex<Scope>>,
    service: Arc<MathService>,
    sender: Option<mpsc::Sender<Request>>,
    joined: Option<tokio::sync::oneshot::Receiver<()>>,
    budget: Arc<WorkerBudget>,
    parked: bool,
}
/// A resident native owner without a strong reference to its service or any request.
/// Dropping it closes the channel; the existing supervisor releases residency after join.
#[derive(Debug)]
pub(super) struct ParkedSession {
    affinity: pse_ids::ContentHash,
    population: Arc<super::jobs::PopulationLane>,
    dispatch: Arc<tokio::sync::Semaphore>,
    scope: Arc<std::sync::Mutex<Scope>>,
    sender: Option<mpsc::Sender<Request>>,
    joined: Option<tokio::sync::oneshot::Receiver<()>>,
    budget: Arc<WorkerBudget>,
}
impl Drop for ParkedSession {
    fn drop(&mut self) { if self.sender.is_some() { self.population.close(); self.sender.take(); } }
}
impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("threads", &self.threads)
            .field("backends", &self.backends)
            .finish_non_exhaustive()
    }
}
impl MathService {
    /// Open a native session: one job slot, the session thread's stack, the deployment's
    /// foreign allowance and the inner-session cache, held until the session closes. Its
    /// workers draw worker storage from the pool as they are built.
    ///
    /// # Errors
    /// No job slot or pool capacity, or the thread could not start.
    pub(crate) fn open_session(self: &Arc<Self>) -> Result<NativeSession, MathRuntimeError> {
        if let Some(session) = self.take_native_lane(None) { return Ok(session) }
        let slot = self
            .jobs
            .clone()
            .try_acquire_owned()
            .map_err(|error| match error {
                tokio::sync::TryAcquireError::Closed => {
                    MathRuntimeError::Limit("native population closed")
                }
                tokio::sync::TryAcquireError::NoPermits => MathRuntimeError::Limit("native jobs"),
            })?;
        self.session_on(Arc::new(slot))
    }
    /// Open a native session for durable work. One population ticket bounds its wait
    /// for memory; no admission waiter sits outside the deployment's job limit.
    ///
    /// # Errors
    /// Pool capacity, a closed admission, or the thread could not start.
    pub(crate) async fn open_session_queued(
        self: &Arc<Self>,
        cancel: &crate::CancelSource,
        deadline: Option<std::time::Instant>,
    ) -> Result<NativeSession, MathRuntimeError> {
        cancel.checkpoint().map_err(|_| MathRuntimeError::Cancelled)?;
        if deadline.is_some_and(|deadline| std::time::Instant::now() >= deadline) {
            return Err(Self::admission_timeout());
        }
        if let Some(session) = self.take_native_lane(None) { return Ok(session) }
        let slot = self
            .jobs
            .clone()
            .try_acquire_owned()
            .map_err(|error| match error {
                tokio::sync::TryAcquireError::Closed => {
                    MathRuntimeError::Limit("native population closed")
                }
                tokio::sync::TryAcquireError::NoPermits => MathRuntimeError::Limit("native jobs"),
            })?;
        let bytes = self.session_bytes()?;
        let control = pse_columnar::flight::FlightCancellation::default();
        let lease = tokio::select! {
            lease = self.reserve_entry("math:native-session", bytes, &control, deadline) => lease?,
            () = cancel.cancelled() => {
                control.cancel();
                return Err(MathRuntimeError::Cancelled);
            }
        };
        cancel
            .checkpoint()
            .map_err(|_| MathRuntimeError::Cancelled)?;
        self.session_on_reserved(Arc::new(slot), lease)
    }
    fn session_bytes(&self) -> Result<usize, MathRuntimeError> {
        // The session thread's inner-solve session cache is held within its lease (I14);
        // its workers draw from the pool as they are built.
        self.policy
            .stack_bytes
            .checked_add(self.policy.foreign_bytes)
            .and_then(|n| n.checked_add(self.policy.inner_session_bytes))
            .ok_or(MathRuntimeError::Limit("native allowance overflow"))
    }
    fn session_on(
        self: &Arc<Self>,
        slot: Arc<tokio::sync::OwnedSemaphorePermit>,
    ) -> Result<NativeSession, MathRuntimeError> {
        let bytes = self.session_bytes()?;
        let lease = self.reserve("math:native-session", bytes)?;
        self.session_on_reserved(slot, lease)
    }
    fn session_on_reserved(
        self: &Arc<Self>,
        slot: Arc<tokio::sync::OwnedSemaphorePermit>,
        lease: Arc<pse_columnar::AllocationLease>,
    ) -> Result<NativeSession, MathRuntimeError> {
        let sessions = self.policy.inner_session_bytes;
        let (sender, receiver) = mpsc::channel::<Request>();
        let stack = self.policy.stack_bytes;
        let pool = self.pool.clone();
        let pressure = self.pressure.clone();
        let scope = Arc::new(std::sync::Mutex::new(Scope {
            backends: Vec::new(),
            threads: 1,
        }));
        let native_scope = scope.clone();
        let thread = std::thread::Builder::new()
            .name("pse-math".into())
            .stack_size(stack)
            .spawn(move || {
                #[cfg(feature = "solver-kinsol")]
                pse_backend_native::implicit::budget_sessions(sessions);
                #[cfg(not(feature = "solver-kinsol"))]
                let _ = sessions;
                serve(&std::sync::Mutex::new(receiver), &pool, &pressure, stack, &native_scope)
            })
            .map_err(|e| MathRuntimeError::Infrastructure(e.to_string()))?;
        let (joined, receiver) = tokio::sync::oneshot::channel();
        let population = super::jobs::PopulationLane::new(self.jobs.clone(), slot.clone());
        let joined_population = population.clone();
        tokio::spawn(pse_operations::canonical::inherit_activity(async move {
            // Joining, not the last reply, witnesses thread-local destruction.
            let _ = tokio::task::spawn_blocking(move || thread.join()).await;
            drop(lease);
            joined_population.drain().await;
            drop(joined_population);
            drop(slot);
            let _ = joined.send(());
        }));
        Ok(NativeSession {
            population,
            dispatch: Arc::new(tokio::sync::Semaphore::new(1)),
            scope,
            service: self.clone(),
            sender: Some(sender),
            joined: Some(receiver),
            budget: WorkerBudget::drawing_for(self.policy.worker_bytes, self),
            parked: false,
        })
    }
}
impl MathService {
    fn take_native_lane(self: &Arc<Self>, affinity: Option<pse_ids::ContentHash>) -> Option<NativeSession> {
        if self.jobs.is_closed() { return None }
        let mut lanes = self.native_lanes.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let index = affinity.and_then(|key| lanes.iter().position(|lane| lane.affinity == key))
            .or_else(|| (!lanes.is_empty()).then_some(0))?;
        let mut lane = lanes.remove(index);
        Some(NativeSession {
            population: lane.population.clone(), dispatch: lane.dispatch.clone(), scope: lane.scope.clone(),
            service: self.clone(), sender: lane.sender.take(), joined: lane.joined.take(),
            budget: lane.budget.clone(), parked: false,
        })
        // Drop cannot close the transferred population; the channel is empty now.
    }
    pub(crate) fn independent_session(self: &Arc<Self>, affinity: pse_ids::ContentHash) -> Result<NativeSession, MathRuntimeError> {
        self.take_native_lane(Some(affinity)).map_or_else(|| self.open_session(), Ok)
    }
    pub(crate) fn retire_idle_native_lanes(&self) {
        let lanes = std::mem::take(&mut *self.native_lanes.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        drop(lanes);
    }
    /// Stop admission and drain all service-owned native lanes, including native TLS and
    /// thread joins. Active request supervisors keep their owners until they complete.
    pub async fn shutdown_native_lanes(&self) {
        self.jobs.close();
        let lanes = std::mem::take(&mut *self.native_lanes.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        for mut lane in lanes {
            lane.population.close(); lane.sender.take();
            if let Some(joined) = lane.joined.take() { let _ = joined.await; }
        }
        while self.jobs.available_permits() < self.policy.jobs {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
}
impl NativeSession {
    /// Lend this idle lane's ticket to directly awaited preparation. Independent spawned
    /// tasks cannot inherit it; an exclusive loan survives through actual child join.
    pub(crate) async fn prepare<T>(
        &mut self,
        work: impl Future<Output = T>,
    ) -> Result<T, MathRuntimeError> {
        if !self.population.idle() {
            return Err(MathRuntimeError::Limit("native session is not quiescent"));
        }
        Ok(super::jobs::population_scope(self.population.clone(), work).await)
    }
    /// Run one step on the session thread with the retained native state, under its own
    /// CPU admission of `cores` permits. `backend` names the adapter whose scope the step's
    /// native state belongs to. Cancelling `cancel` (or dropping the returned future) sets
    /// the step's stop flag; the session stays usable for later steps.
    ///
    /// # Errors
    /// Admission failure, cancellation before admission, a closed session or the step's own
    /// error.
    #[cfg(any(test, feature = "solver-diffsol"))]
    pub(crate) async fn run<T: Send + 'static>(
        &self,
        cores: usize,
        backend: Option<Backend>,
        cancel: &crate::CancelSource,
        work: impl FnOnce(
            &mut Retained,
            &Arc<AtomicBool>,
            &Arc<WorkerBudget>,
        ) -> Result<T, MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<T, MathRuntimeError> {
        self.run_scoped(cores, backend.into_iter().collect(), None, cancel, work)
            .await
    }
    /// Queue native work under the already admitted task's cancellation owner and clock.
    #[cfg(feature = "solver-diffsol")]
    pub(crate) async fn run_in_task<T: Send + 'static>(
        &self,
        cores: usize,
        scope: pse_kernels::ExecutionScope,
        cancel: &crate::CancelSource,
        work: impl FnOnce(
            &mut Retained,
            &Arc<AtomicBool>,
            &Arc<WorkerBudget>,
        ) -> Result<T, MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<T, MathRuntimeError> {
        scope.check().map_err(ProblemError::Provider)?;
        self.run_task(
            cores,
            Vec::new(),
            scope.deadline(),
            Some(scope),
            cancel,
            work,
        )
        .await
    }
    async fn run_scoped<T: Send + 'static>(
        &self,
        cores: usize,
        backends: Vec<Backend>,
        deadline: Option<std::time::Instant>,
        cancel: &crate::CancelSource,
        work: impl FnOnce(
            &mut Retained,
            &Arc<AtomicBool>,
            &Arc<WorkerBudget>,
        ) -> Result<T, MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<T, MathRuntimeError> {
        self.run_task(cores, backends, deadline, None, cancel, work)
            .await
    }
    /// A prepared task supplies the cancellation owner and clock consumed by every rung.
    async fn run_task<T: Send + 'static>(
        &self,
        cores: usize,
        backends: Vec<Backend>,
        deadline: Option<std::time::Instant>,
        enclosing: Option<pse_kernels::ExecutionScope>,
        cancel: &crate::CancelSource,
        work: impl FnOnce(
            &mut Retained,
            &Arc<AtomicBool>,
            &Arc<WorkerBudget>,
        ) -> Result<T, MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<T, MathRuntimeError> {
        self.run_task_context(
            cores,
            backends,
            deadline,
            enclosing,
            cancel,
            move |retained, flag, budget, _| work(retained, flag, budget),
        )
        .await
    }
    /// A prepared task supplies the cancellation owner and clock consumed by every rung.
    async fn run_task_context<T: Send + 'static>(
        &self,
        cores: usize,
        backends: Vec<Backend>,
        deadline: Option<std::time::Instant>,
        enclosing: Option<pse_kernels::ExecutionScope>,
        cancel: &crate::CancelSource,
        work: impl FnOnce(
            &mut Retained,
            &Arc<AtomicBool>,
            &Arc<WorkerBudget>,
            &mut OperationContext,
        ) -> Result<T, MathRuntimeError>
        + Send
        + 'static,
    ) -> Result<T, MathRuntimeError> {
        if cores == 0 || cores > self.service.cores || cores > u32::MAX as usize {
            return Err(MathRuntimeError::Limit("optimizer cores"));
        }
        let sender = self
            .sender
            .as_ref()
            .ok_or_else(|| MathRuntimeError::Infrastructure("closed native session".into()))?;
        let admission_deadline = self.service.admission_deadline(deadline)?;
        let expires = async {
            tokio::time::sleep_until(admission_deadline.into()).await;
        };
        let task_stop = async {
            match &enclosing {
                Some(scope) => loop {
                    if let Err(error) = scope.check() {
                        break error;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                },
                None => std::future::pending::<pse_kernels::ProviderError>().await,
            }
        };
        tokio::pin!(expires, task_stop);
        let dispatch = tokio::select! {
            dispatch = self.dispatch.clone().acquire_owned() => dispatch.map_err(|_| MathRuntimeError::Limit("native dispatch closed"))?,
            () = cancel.cancelled() => return Err(MathRuntimeError::Cancelled),
            () = &mut expires => return Err(MathService::admission_timeout()),
            error = &mut task_stop => return Err(ProblemError::Provider(error).into()),
        };
        let usage = self.population.claim()?;
        // A dispatch owner observes one stable lane. Idle pressure may only shrink the
        // entered scope, so a stale union is a conservative construction reservation.
        let next = Scope { backends: backends.clone(), threads: cores };
        let adapters = next
            .backends
            .iter()
            .map(|backend| execution::adapter(*backend))
            .collect::<Vec<_>>();
        let stacks =
            execution::scope_stack_bytes(&adapters, next.threads, self.service.policy.stack_bytes)?;
        let control = pse_columnar::flight::FlightCancellation::default();
        let (reservation, permits) = tokio::select! {
            permits = self.service.reserve_active("math:native-scope-stacks", cores as u32, stacks, &control, admission_deadline) => permits?,
            () = cancel.cancelled() => {
                if let Some(scope)=&enclosing {scope.cancellation().store(true,Ordering::Release);}
                return Err(MathRuntimeError::Cancelled);
            },
            () = &mut expires => return Err(ProblemError::Limit {
                kind:pse_backend_native::LimitKind::Time,
                detail:"native CPU admission deadline".into(),
            }.into()),
            error=&mut task_stop=>return Err(ProblemError::Provider(error).into()),
        };
        cancel
            .checkpoint()
            .map_err(|_| MathRuntimeError::Cancelled)?;
        if std::time::Instant::now() >= admission_deadline {
            return Err(MathService::admission_timeout());
        }
        if let Some(scope) = &enclosing {
            scope.check().map_err(ProblemError::Provider)?;
        }
        let flag = enclosing.as_ref().map_or_else(
            || Arc::new(AtomicBool::new(false)),
            |scope| scope.cancellation().clone(),
        );
        let mut stop = Stop(Some(flag.clone()));
        let budget = self.budget.clone();
        let (reply, result) = tokio::sync::oneshot::channel();
        let compute = ComputeOwner::new(
            self.service.active_jobs.clone(),
            self.service.cpu.clone(),
            cores as u32,
            permits,
            tokio::runtime::Handle::current(),
        );
        #[cfg(any(all(test, feature = "solver-ipopt"), feature = "canonical-tests"))]
        let entry = NATIVE_ENTRY.try_with(Arc::clone).ok();
        #[cfg(feature = "canonical-tests")]
        let entry = entry.or_else(|| RECEIVER_ENTRY.get().cloned());
        let scope_owner = pse_columnar::AllocationLease::new(reservation.split(stacks));
        #[cfg(any(test, feature = "solver-kinsol"))]
        let lane = self.population.clone();
        #[cfg(any(test, feature = "solver-kinsol"))]
        let runtime = tokio::runtime::Handle::current();
        let request = Request {
            threads: cores,
            backends,
            scope_owner: Some(scope_owner),
            work: Box::new(move |retained| {
                let mut context = OperationContext {
                    #[cfg(any(test, feature = "solver-kinsol"))]
                    lane,
                    _usage: Some(usage),
                    #[cfg(any(test, feature = "solver-kinsol"))]
                    runtime,
                };
                let _dispatch = dispatch;
                let outcome = execution::compute_scoped(Box::new(compute), || match retained {
                    Ok(retained) => {
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            #[cfg(any(
                                all(test, feature = "solver-ipopt"),
                                feature = "canonical-tests"
                            ))]
                            if let Some(entry) = entry {
                                entry(&flag);
                            }
                            work(retained, &flag, &budget, &mut context)
                        })) {
                            Ok(outcome) => outcome,
                            Err(_) => {
                                // The request still owns its CPU guard. Destroy poisoned
                                // foreign state inside its adapter scope before releasing it.
                                retained.clear();
                                retained.release();
                                Err(MathRuntimeError::Panic("native session step panic".into()))
                            }
                        }
                    }
                    Err(error) => Err(error.into()),
                });
                drop(context);
                drop(_dispatch);
                let _ = reply.send(outcome);
            }),
        };
        if sender.send(request).is_err() {
            stop.0 = None;
            return Err(MathRuntimeError::Infrastructure(
                "lost native session".into(),
            ));
        }
        tokio::pin!(result);
        let outcome = tokio::select! {
            outcome = &mut result => outcome,
            () = cancel.cancelled() => {
                if let Some(flag)=&stop.0 {flag.store(true, Ordering::Release);}
                result.await
            }
        };
        stop.0 = None;
        outcome.map_err(|_| MathRuntimeError::Infrastructure("lost native session step".into()))?
    }
    /// Run a complete serial sequence on this native owner. Child preparation may
    /// borrow its population only at an explicit quiescent boundary; no block dispatch
    /// crosses the owner channel, and retained state dies before serial scope guards.
    #[cfg(any(test, feature = "solver-kinsol"))]
    pub(crate) async fn sequence<T: Send + 'static>(
        &self,
        scope: pse_kernels::ExecutionScope,
        controls: pse_backend_native::solve::Controls,
        cancel: &crate::CancelSource,
        work: impl FnOnce(&LocalOperation<'_>) -> Result<T, MathRuntimeError> + Send + 'static,
    ) -> Result<T, MathRuntimeError> {
        let service = self.service.clone();
        let local_scope = scope.clone();
        self.run_task_context(
            1,
            Vec::new(),
            scope.deadline(),
            Some(scope),
            cancel,
            move |retained, flag, budget, context| {
                retained.clear();
                retained.release();
                let local = LocalOperation::new(
                    service,
                    retained,
                    context,
                    flag.clone(),
                    budget.clone(),
                    local_scope,
                    controls,
                )?;
                work(&local)
            },
        )
        .await
    }
    /// Resolve the existing full strategy on this session's dispatch lane.
    #[allow(
        clippy::too_many_arguments,
        reason = "the step protocol keeps predecessor, attempt, progress, storage, cancellation and original assessment authorities explicit"
    )]
    pub(crate) async fn step<T: Send + 'static>(
        &self,
        step: super::solves::PreparedSolve,
        previous: Option<super::solves::Predecessor>,
        attempt: usize,
        progress: Arc<Progress>,
        owner: Arc<pse_columnar::AllocationLease>,
        cancel: &crate::CancelSource,
        assess: impl FnMut(
            &super::solves::Outcome,
            Option<&pse_kernels::ExecutionScope>,
            &Arc<WorkerBudget>,
        ) -> super::strategy::Assessed<T>
        + Send
        + 'static,
    ) -> Result<(super::solves::Outcome, T, super::solves::StrategyTrace), MathRuntimeError> {
        driver::StepDriver {
            service: self.service.clone(),
            dispatcher: self,
        }
        .step(step, previous, attempt, progress, owner, cancel, assess)
        .await
    }
    /// Execute the independent steps of one batch on the retained native state under one
    /// admission of the first step's threads (Plan 22 N5), and `assess` each outcome by its
    /// position on the same worker. The retained state survives only when every member ends
    /// with a usable, accepted candidate.
    ///
    /// # Errors
    /// Admission, cancellation before admission or a lost session.
    pub(crate) async fn batch<T: Send + 'static>(
        &self,
        members: Vec<super::solves::BatchMember>,
        progress: Arc<Progress>,
        cancel: &crate::CancelSource,
        mut assess: impl FnMut(
            usize,
            &super::solves::Outcome,
            Option<&pse_kernels::ExecutionScope>,
            &Arc<WorkerBudget>,
        ) -> super::strategy::Assessed<T>
        + Send
        + 'static,
    ) -> Result<
        Vec<Result<(super::solves::Outcome, T, super::solves::StrategyTrace), MathRuntimeError>>,
        MathRuntimeError,
    > {
        let Some(first) = members.first() else {
            return Ok(Vec::new());
        };
        // Batch only a composition the common resolver has admitted as one ready
        // direct operation. Other members retain the exact individual task semantics,
        // including hard caps, optional preparations and observation-dependent profiles.
        let batch_profile = first.step.strategy_profile().ok();
        let batch_threads = first.step.threads();
        let mut direct_batch = batch_profile.is_some();
        let mut decisions = Vec::with_capacity(members.len());
        for member in &members {
            let request = member.step.composition_request();
            if request.policy != pse_model::strategy::CompositionPolicy::Auto
                || request.limits.is_some()
                || request.branch.connected.is_some()
                || member.step.task_scope().is_some()
                || member.step.composition_is_declared()
            {
                direct_batch = false;
                continue;
            }
            let operations = match member.step.automatic_operations(
                None,
                &Arc::new(AtomicBool::new(cancel.token().is_cancelled())),
            ) {
                Ok(operations) => operations,
                Err(_) => {
                    direct_batch = false;
                    continue;
                }
            };
            direct_batch &= member.step.strategy_profile().ok() == batch_profile
                && member.step.threads() == batch_threads
                && member.step.task_scope().is_none()
                && !member.step.composition_is_declared()
                && request.policy == pse_model::strategy::CompositionPolicy::Auto
                && request.limits.is_none()
                && request.branch.connected.is_none()
                && member.step.backend() != Some(Backend::Pounce)
                && operations.len() == 1
                && operations[0].candidate.kind == pse_model::strategy::MechanismKind::Direct
                && operations[0].candidate.prepared;
            let candidates = operations
                .iter()
                .map(|operation| operation.candidate.clone())
                .collect::<Vec<_>>();
            let declaration = member.step.numerical_strategy();
            let work = pse_model::strategy::WorkObservation {
                attempts: 0,
                evaluations: Some(0),
                iterations: Some(0),
                factorizations: Some(0),
                proof_steps: Some(0),
            };
            let decision = super::strategy::next_automatic(
                request,
                &declaration.start,
                &candidates,
                &Default::default(),
                None,
                work,
                false,
                None,
            );
            direct_batch &= matches!(
                decision,
                super::strategy::AutoDecision::Dispatch { candidate: 0 }
            );
            match super::strategy::automatic_decision_key(
                request,
                &candidates,
                &decision,
                None,
                work,
            ) {
                Ok(key) => decisions.push(key),
                Err(_) => direct_batch = false,
            };
        }
        if !direct_batch {
            let assessor = Arc::new(std::sync::Mutex::new(assess));
            let mut outcomes = Vec::with_capacity(members.len());
            for (index, member) in members.into_iter().enumerate() {
                let assessor = assessor.clone();
                outcomes.push(
                    self.step(
                        member.step,
                        None,
                        member.attempt,
                        progress.clone(),
                        member.owner,
                        cancel,
                        move |outcome, scope, budget| {
                            let mut owner = assessor
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            owner(index, outcome, scope, budget)
                        },
                    )
                    .await,
                );
            }
            return Ok(outcomes);
        }
        let service = self.service.clone();
        let deadline = std::time::Instant::now()
            .checked_add(first.step.time_limit())
            .ok_or_else(|| ProblemError::Contract("batch deadline extent".into()))?;
        let threads = first.step.threads();
        if members
            .iter()
            .any(|m| m.step.threads() != threads || m.step.composition_is_declared())
        {
            return Err(ProblemError::Unsupported(
                "native batching requires one thread extent and direct member declarations".into(),
            )
            .into());
        }
        let backends = members.iter().filter_map(|m| m.step.backend()).collect();
        let originals = members
            .iter()
            .map(|m| {
                Ok((
                    m.step.numerical_strategy(),
                    m.step.original_identity()?,
                    m.step.backend(),
                    m.step.strategy_profile()?,
                    m.step.entry_origin(false),
                    m.attempt,
                ))
            })
            .collect::<Result<Vec<_>, ProblemError>>()?;
        let declared = members
            .iter()
            .try_fold(0usize, |n, m| {
                n.checked_add(m.step.declared_foreign_bytes())
            })
            .ok_or(MathRuntimeError::Limit("native allowance overflow"))?;
        let allowance = service.reserve("math:step-foreign", declared)?;
        self.run_scoped(threads,backends,Some(deadline),cancel,move |retained,flag,budget| {
            let enclosing=pse_kernels::ExecutionScope::new(flag.clone(),Some(deadline));
            let outcomes=service.execute_batch(members,retained,flag,&progress,budget,&enclosing);
            let mut kept=true;
            let assessed=outcomes.into_iter().zip(originals).zip(decisions).enumerate().map(|(index,((outcome,(declaration,original,backend,profile,start,attempt)),decision))| {
                let mut value=Some(outcome.unwrap_or_else(|cause| (super::solves::Outcome::Rejected(Arc::new(cause)),Some(enclosing.clone()))));
                let mut product=None;
                let mut result=super::strategy::run_observed(&declaration,&enclosing,
                    |_|super::strategy::Facts {support:Default::default(),accuracy:Vec::new(),consumption:Vec::new(),reservation:None,work_admitted:false,start,inherited:false,connected:false,refusal:None},
                    |_,_| {
                        let value=value.take().ok_or_else(||Arc::new(ProblemError::Internal("batch outcome consumed twice".into())))?;
                        let mut charge=pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
                        charge.hash(&original).u64(attempt as u64).u64(index as u64);
                        Ok(super::strategy::Attempt {evidence:Vec::new(),observation:super::strategy::observe(&value.0),work:pse_model::strategy::WorkCharge {phase:pse_model::strategy::Phase::Native,scope:pse_model::strategy::Scope::Task,charging_owner:charge.finish_hash(),observed:super::strategy::work(&value.0)},value})
                    },
                    |(outcome,scope),observation| {
                        let assessment=assess(index,outcome,scope.as_ref(),budget);
                        product=Some(assessment.product);
                        super::strategy::Assessment {auxiliary:false,retention:assessment.retention,original:assessment.original,work:assessment.work,observation,cause:super::strategy::cause(outcome)}
                    });
                for event in &mut result.events {if event.decision.is_none() {event.decision=Some(decision);}}
                let trace=super::strategy::Trace {owner:None,publication_request:None,declaration,original,backend,profile,start,starts:vec![start],products:vec![Default::default()],events:result.events};
                let bytes=trace.retained_bytes()?;
                let trace=Arc::new(trace.with_owner(service.reserve("math:strategy-trace",bytes)?));
                match (result.value,product,result.terminal) {
                    (Some((outcome,_)),Some(assessed),None)=>{
                        kept &= result.assessment.as_ref().is_some_and(|a|a.retention.permits_session());
                        Ok((outcome,assessed,trace))
                    }
                    (_,_,terminal)=>{
                        kept=false;
                        let cause=terminal.map_or_else(||ProblemError::Internal("batch ended without original assessment".into()),|cause|ProblemError::Math(pse_math::MathError::Typed {retained:cause.retained_bytes(),cause:pse_model::diagnostic::DiagnosticCause::from_shared(cause)}));
                        let outcome=super::solves::Outcome::Rejected(Arc::new(cause.into()));
                        let assessed=assess(index,&outcome,Some(&enclosing),budget).product;
                        Ok((outcome,assessed,trace))
                    }
                }
            }).collect();
            if kept {retained.charge_session(Box::new(allowance));} else {retained.clear();}
            Ok(assessed)
        }).await
    }
    /// Reset on the owning thread before parking. This request has exclusive dispatch;
    /// no request ledger, callback borrower or numerical response factor may survive.
    pub(crate) async fn park_independent(mut self, affinity: pse_ids::ContentHash) -> Result<(), MathRuntimeError> {
        let scope = self.scope.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
        let cancel = crate::CancelSource::new();
        #[cfg(feature = "solver-kinsol")]
        let inner_bytes = self.service.policy.inner_session_bytes;
        let reset = self.run_scoped(scope.threads, scope.backends, None, &cancel, move |retained, _, _| {
            retained.reset_independent()?;
            #[cfg(feature = "solver-kinsol")]
            { // Cached inner sessions still own their submitted functions and controls.
                pse_backend_native::implicit::budget_sessions(0);
                pse_backend_native::implicit::budget_sessions(inner_bytes);
            }
            SCRATCH_LANE.with(|slot| { if let Some(lane) = slot.borrow_mut().as_mut() { lane.clear(); } });
            Ok(())
        }).await;
        if let Err(cause) = reset { self.close().await; return Err(cause) }
        let service = self.service.clone();
        if service.jobs.is_closed() || service.pressure.active() { self.close().await; return Ok(()) }
        let closing = {
            let mut lanes = service.native_lanes.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if service.jobs.is_closed() { true } else {
                let lane = ParkedSession { affinity, population: self.population.clone(), dispatch: self.dispatch.clone(),
                    scope: self.scope.clone(), sender: self.sender.take(), joined: self.joined.take(), budget: self.budget.clone() };
                self.parked = true;
                lanes.push(lane);
                false
            }
        };
        if closing { self.close().await; }
        Ok(())
    }
    /// Close the session and wait until its thread has joined.
    pub(crate) async fn close(mut self) {
        self.population.close();
        self.sender.take();
        if let Some(joined) = self.joined.take() {
            let _ = joined.await;
        }
    }
}
impl Drop for NativeSession {
    fn drop(&mut self) {
        if self.parked { return; }
        self.population.close();
        // Without an explicit close the thread still exits after its last step and the
        // supervisor joins it before releasing the admission.
        self.sender.take();
    }
}

#[cfg(test)]
mod admission_tests {
    use super::*;
    use std::sync::{Condvar, Mutex};

    #[tokio::test]
    async fn independent_lane_reuses_owner_releases_request_state_and_has_no_service_cycle() {
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let affinity = pse_ids::ContentHash::from_bytes([9; 32]);
        let lane = service.independent_session(affinity).unwrap();
        let scope_owner = lane.scope.clone();
        let budget = lane.budget.clone();
        let cancel = crate::CancelSource::new();
        lane.run(1, None, &cancel, |retained, _, _| {
            retained.session(Backend::Kinsol, pse_backend_native::solve::ReusePolicy::AllowRebuild,
                |_: &mut u64| Ok(execution::SessionUpdate::Compatible), || Ok(3u64))?;
            Ok(())
        }).await.unwrap();
        lane.park_independent(affinity).await.unwrap();
        assert_eq!(service.native_lanes.lock().unwrap().len(), 1);
        assert_eq!(service.cpu.available_permits(), service.cores);
        assert_eq!(service.active_jobs.available_permits(), service.policy.jobs);
        assert!(service.pool.reserved() > baseline);
        let reused = service.independent_session(affinity).unwrap();
        assert!(Arc::ptr_eq(&scope_owner, &reused.scope));
        reused.run(1, None, &cancel, |retained, _, _| { assert!(retained.is_empty()); assert!(retained.advance().is_none()); Ok(()) }).await.unwrap();
        reused.park_independent(affinity).await.unwrap();
        drop(budget); drop(scope_owner);
        let pool = service.pool.clone();
        let weak = Arc::downgrade(&service);
        service.shutdown_native_lanes().await;
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
        assert_eq!(pool.reserved(), baseline);
        drop(service);
        assert!(weak.upgrade().is_none(), "resident workers cannot strongly retain their service");
    }
    #[tokio::test]
    async fn dropping_service_retires_parked_native_owner_without_an_arc_cycle() {
        let service = super::super::tests::service();
        let pool = service.pool.clone();
        let baseline = pool.reserved();
        let weak = Arc::downgrade(&service);
        let affinity = pse_ids::ContentHash::from_bytes([8; 32]);
        service.open_session().unwrap().park_independent(affinity).await.unwrap();
        assert!(pool.reserved() > baseline);
        drop(service);
        assert!(weak.upgrade().is_none());
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while pool.reserved() != baseline {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
        }).await.unwrap();
    }
    #[tokio::test]
    async fn resident_population_is_reused_by_unrelated_session_before_starvation() {
        let service = super::super::tests::service();
        let mut sessions = Vec::new();
        for _ in 0..service.policy.jobs { sessions.push(service.open_session().unwrap()); }
        let affinity = pse_ids::ContentHash::from_bytes([7; 32]);
        for session in sessions { session.park_independent(affinity).await.unwrap(); }
        assert_eq!(service.jobs.available_permits(), 0);
        let session = service.open_session().unwrap();
        assert_eq!(service.native_lanes.lock().unwrap().len(), service.policy.jobs - 1);
        session.close().await;
        service.shutdown_native_lanes().await;
    }
    #[test]
    fn native_scope_affinity_replaces_old_profile_pools() {
        let current = Scope { backends: vec![Backend::Pounce], threads: 2 };
        let request = Request { threads: 1, backends: vec![Backend::Highs], scope_owner: None,
            work: Box::new(|_| {}) };
        assert!(!current.admits(&request));
        let next = current.extended(&request);
        assert_eq!(next.backends, vec![Backend::Highs]);
        assert_eq!(next.threads, 1);
        assert!(next.admits(&request));
    }
    #[tokio::test]
    async fn exclusive_session_scratch_waits_for_request_return_and_native_sibling_drain() {
        let service = super::super::tests::service();
        let (prepared, mut values, parameter) = super::super::tests::component_fixture(&service);
        let case = service.assemble(prepared).await.unwrap();
        values.scalars.insert(case.assembly.columns()[0], 1.0);
        let session = service.open_session().unwrap();
        let cancel = crate::CancelSource::new();
        let bytes = case.assembly.numeric_worker_bytes() + 1024;
        let work_service = service.clone();
        let first_case = case.clone();
        let first_values = values.clone();
        session
            .run(1, None, &cancel, move |retained, _, budget| {
                let stop = Arc::new(AtomicBool::new(false));
                let weak = Arc::downgrade(&stop);
                let super::super::ExecutionWorker {
                    mut worker,
                    _case,
                    _charge,
                } = work_service.reusable_worker(
                    first_case.clone(),
                    &Default::default(),
                    pse_kernels::ExecutionScope::new(stop.clone(), None),
                    budget,
                )?;
                worker.constraints(&first_values)?;
                drop(worker);
                drop(stop);
                assert!(
                    weak.upgrade().is_none(),
                    "pending scratch owns no old cancellation"
                );
                let origin = scratch_origin().unwrap();
                assert!(
                    take_scratch(&origin, &first_case, budget, bytes).is_none(),
                    "work has not returned"
                );
                retained.session(
                    Backend::Kinsol,
                    pse_backend_native::solve::ReusePolicy::AllowRebuild,
                    |_: &mut super::super::WorkerCharge| Ok(execution::SessionUpdate::Incompatible),
                    || Ok(_charge),
                )?;
                drop(_case);
                assert_eq!(budget.used(), bytes);
                Ok(())
            })
            .await
            .unwrap();
        session
            .run(1, None, &cancel, move |retained, _, budget| {
                assert_eq!(budget.used(), bytes);
                SCRATCH_LANE.with(|slot| {
                    let slot = slot.borrow();
                    let lane = slot.as_ref().unwrap();
                    assert!(lane.pending.is_some());
                    assert!(lane.ready.is_none());
                });
                retained.clear();
                SCRATCH_LANE.with(|slot| assert!(slot.borrow().as_ref().unwrap().ready.is_none()));
                Ok(())
            })
            .await
            .unwrap();
        let work_service = service.clone();
        let next_case = case.clone();
        let mut next_values = values.clone();
        next_values.scalars.insert(parameter, 2.0);
        session
            .run(1, None, &cancel, move |_, flag, budget| {
                assert_eq!(budget.used(), bytes, "idle storage stays charged");
                SCRATCH_LANE.with(|slot| assert!(slot.borrow().as_ref().unwrap().ready.is_some()));
                let mut reused = work_service.reusable_worker(
                    next_case.clone(),
                    &Default::default(),
                    pse_kernels::ExecutionScope::new(flag.clone(), None),
                    budget,
                )?;
                assert_eq!(
                    budget.used(),
                    bytes,
                    "exclusive attach neither releases nor charges again"
                );
                let actual = reused.worker.constraints(&next_values)?;
                let mut independent = work_service.worker(
                    next_case,
                    &Default::default(),
                    pse_kernels::ExecutionScope::new(flag.clone(), None),
                    budget,
                )?;
                assert_eq!(
                    budget.used(),
                    bytes + bytes - 1024,
                    "independent original worker is cold"
                );
                assert_eq!(actual, independent.worker.constraints(&next_values)?);
                drop(independent);
                drop(reused);
                Ok(())
            })
            .await
            .unwrap();
        let budget = session.budget.clone();
        session
            .run(1, None, &cancel, move |_, _, budget| {
                assert_eq!(budget.used(), bytes);
                Ok(())
            })
            .await
            .unwrap();
        session.close().await;
        assert_eq!(budget.used(), 0);
    }

    #[tokio::test]
    async fn exclusive_session_scratch_refuses_foreign_budget_and_pressure_destroys_idle_storage() {
        let service = super::super::tests::service();
        let (prepared, _, _) = super::super::tests::component_fixture(&service);
        let case = service.assemble(prepared).await.unwrap();
        let session = service.open_session().unwrap();
        let cancel = crate::CancelSource::new();
        let bytes = case.assembly.numeric_worker_bytes() + 1024;
        let work_service = service.clone();
        let first = case.clone();
        session
            .run(1, None, &cancel, move |_, flag, budget| {
                drop(work_service.reusable_worker(
                    first,
                    &Default::default(),
                    pse_kernels::ExecutionScope::new(flag.clone(), None),
                    budget,
                )?);
                Ok(())
            })
            .await
            .unwrap();
        let next = case.clone();
        session
            .run(1, None, &cancel, move |_, _, budget| {
                assert_eq!(budget.used(), bytes);
                let origin = scratch_origin().unwrap();
                let mut wrong_lane = origin.clone();
                wrong_lane.lane = Arc::new(());
                assert!(take_scratch(&wrong_lane, &next, budget, bytes).is_none());
                let mut old_request = origin.clone();
                old_request.request -= 1;
                assert!(take_scratch(&old_request, &next, budget, bytes).is_none());
                SCRATCH_LANE.with(|slot| assert!(slot.borrow().as_ref().unwrap().ready.is_some()));
                let foreign = WorkerBudget::new(bytes * 4);
                assert!(take_scratch(&origin, &next, &foreign, bytes).is_none());
                assert_eq!(budget.used(), 0, "incompatible idle candidate destroyed");
                Ok(())
            })
            .await
            .unwrap();
        let work_service = service.clone();
        let first = case.clone();
        session
            .run(1, None, &cancel, move |_, flag, budget| {
                drop(work_service.reusable_worker(
                    first,
                    &Default::default(),
                    pse_kernels::ExecutionScope::new(flag.clone(), None),
                    budget,
                )?);
                Ok(())
            })
            .await
            .unwrap();
        session
            .run(1, None, &cancel, move |_, _, budget| {
                assert_eq!(budget.used(), bytes);
                Ok(())
            })
            .await
            .unwrap();
        let pressure = service.pressure.demand();
        // A real request boundary also witnesses pressure, without a guessed delay
        // or a second permission poll. All idle transfer charges must then depart.
        session
            .run(1, None, &cancel, move |_, _, _| Ok(()))
            .await
            .unwrap();
        assert_eq!(session.budget.used(), 0);
        drop(pressure);
        session.close().await;
    }

    #[tokio::test]
    async fn exclusive_session_reset_overlap_refusal_destroys_candidate_before_next_attempt() {
        let service = super::super::tests::service();
        let (prepared, mut values, _) = super::super::tests::component_fixture(&service);
        let case = service.assemble(prepared).await.unwrap();
        values.scalars.insert(case.assembly.columns()[0], 1.0);
        let bytes = case.assembly.numeric_worker_bytes() + 1024;
        let mut session = service.open_session().unwrap();
        session.budget = WorkerBudget::drawing_for(bytes, &service);
        let cancel = crate::CancelSource::new();
        let work_service = service.clone();
        let first = case.clone();
        session
            .run(1, None, &cancel, move |_, flag, budget| {
                let mut worker = work_service.reusable_worker(
                    first,
                    &Default::default(),
                    pse_kernels::ExecutionScope::new(flag.clone(), None),
                    budget,
                )?;
                worker.worker.constraints(&values)?;
                drop(worker);
                assert_eq!(budget.used(), bytes);
                Ok(())
            })
            .await
            .unwrap();
        session
            .run(1, None, &cancel, move |_, _, budget| {
                assert_eq!(
                    budget.used(),
                    0,
                    "reset overlap refuses before allocation and destroys candidate"
                );
                SCRATCH_LANE.with(|slot| {
                    let slot = slot.borrow();
                    let lane = slot.as_ref().unwrap();
                    assert!(lane.ready.is_none());
                    assert!(lane.pending.is_none());
                });
                Ok(())
            })
            .await
            .unwrap();
        session.close().await;
    }

    #[tokio::test]
    async fn cold_independent_worker_can_yield_optional_idle_scratch_for_required_admission() {
        let service = super::super::tests::service();
        let (prepared, _, _) = super::super::tests::component_fixture(&service);
        let case = service.assemble(prepared).await.unwrap();
        let session = service.open_session().unwrap();
        let cancel = crate::CancelSource::new();
        let work_service = service.clone();
        let first = case.clone();
        session
            .run(1, None, &cancel, move |_, flag, budget| {
                drop(work_service.reusable_worker(
                    first,
                    &Default::default(),
                    pse_kernels::ExecutionScope::new(flag.clone(), None),
                    budget,
                )?);
                Ok(())
            })
            .await
            .unwrap();
        let work_service = service.clone();
        session
            .run(1, None, &cancel, move |_, flag, budget| {
                let cold = case.assembly.numeric_worker_bytes();
                let used = budget.used();
                assert!(used > cold);
                let fill = budget.charge(budget.capacity() - used - cold + 1)?;
                let worker = work_service.worker(
                    case,
                    &Default::default(),
                    pse_kernels::ExecutionScope::new(flag.clone(), None),
                    budget,
                )?;
                SCRATCH_LANE.with(|slot| assert!(slot.borrow().as_ref().unwrap().ready.is_none()));
                drop(worker);
                drop(fill);
                assert_eq!(budget.used(), 0);
                Ok(())
            })
            .await
            .unwrap();
        session.close().await;
    }

    #[tokio::test]
    async fn optional_scratch_metadata_does_not_reduce_cold_policy_capacity() {
        let original = super::super::tests::service();
        let (prepared, _, _) = super::super::tests::component_fixture(&original);
        let case = original.assemble(prepared).await.unwrap();
        let cold = case.assembly.numeric_worker_bytes();
        let (service, _cache) = super::super::tests::service_with_policy(
            512 << 20,
            super::super::MathPolicy {
                worker_bytes: cold,
                ..original.policy.clone()
            },
        );
        let session = service.open_session().unwrap();
        let worker_service = service.clone();
        session
            .run(
                1,
                None,
                &crate::CancelSource::new(),
                move |_, flag, budget| {
                    let worker = worker_service.reusable_worker(
                        case,
                        &Default::default(),
                        pse_kernels::ExecutionScope::new(flag.clone(), None),
                        budget,
                    )?;
                    assert_eq!(budget.used(), cold);
                    drop(worker);
                    assert_eq!(budget.used(), 0);
                    Ok(())
                },
            )
            .await
            .unwrap();
        session.close().await;
    }

    #[tokio::test]
    async fn optional_scratch_metadata_does_not_reduce_cold_live_budget_capacity() {
        let service = super::super::tests::service();
        let (prepared, _, _) = super::super::tests::component_fixture(&service);
        let case = service.assemble(prepared).await.unwrap();
        let cold = case.assembly.numeric_worker_bytes();
        let session = service.open_session().unwrap();
        let worker_service = service.clone();
        session
            .run(
                1,
                None,
                &crate::CancelSource::new(),
                move |_, flag, budget| {
                    let fill = budget.charge(budget.capacity() - cold)?;
                    let worker = worker_service.reusable_worker(
                        case,
                        &Default::default(),
                        pse_kernels::ExecutionScope::new(flag.clone(), None),
                        budget,
                    )?;
                    assert_eq!(budget.used(), budget.capacity());
                    drop(worker);
                    drop(fill);
                    assert_eq!(budget.used(), 0);
                    Ok(())
                },
            )
            .await
            .unwrap();
        session.close().await;
    }

    #[test]
    fn expired_driver_retains_actual_assessed_native_time_stop_and_trace() {
        use super::super::strategy::{self, AutoObservation, OriginalConclusion};
        use pse_backend_native::solve::{
            Assurance, Controls, Execution, NativeTermination, SolveReport, Termination,
        };
        use pse_model::{
            generated::enums::{CandidateUse, NumericalEventKind},
            strategy::{Phase, Scope, StartOrigin, WorkCharge, WorkObservation},
        };
        let controls = Controls::default();
        let scope = pse_kernels::ExecutionScope::new(Arc::default(), None);
        let declaration = strategy::direct(&controls);
        let cause = Arc::new(ProblemError::stopped(
            Termination::TimeLimit,
            "scripted native time stop",
        ));
        let contract = pse_backend_native::OracleContract {
            identity: pse_ids::ContentHash::from_bytes([2; 32]),
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
            variables: Vec::new(),
            rows: Vec::new(),
        };
        let report = SolveReport::new(
            Backend::Ipopt,
            &contract,
            NativeTermination {
                code: 0,
                name: "scripted.time.stop".into(),
                message: None,
                category: Termination::TimeLimit,
                assurance: Assurance::None,
            },
            &Execution::new(scope.cancellation().clone(), &controls),
        );
        let work = WorkObservation {
            attempts: 1,
            evaluations: Some(0),
            iterations: Some(0),
            factorizations: Some(0),
            proof_steps: Some(0),
        };
        let attempts = std::cell::Cell::new(0);
        let assessments = std::cell::Cell::new(0);
        let result = strategy::run(
            &declaration,
            &scope,
            |_| strategy::Facts {
                support: Default::default(),
                accuracy: Vec::new(),
                consumption: Vec::new(),
                reservation: Some(work),
                work_admitted: false,
                start: StartOrigin::Specification,
                inherited: false,
                connected: false,
                refusal: None,
            },
            |_, _| {
                attempts.set(attempts.get() + 1);
                Ok(strategy::Attempt {
                    evidence: Vec::new(),
                    observation: strategy::observe_native(&report),
                    value: report.clone(),
                    work: WorkCharge {
                        phase: Phase::Native,
                        scope: Scope::Task,
                        charging_owner: contract.identity,
                        observed: work,
                    },
                })
            },
            |report, observation| {
                assessments.set(assessments.get() + 1);
                strategy::Assessment {
                    auxiliary: false,
                    original: OriginalConclusion::Unavailable {
                        cause: cause.clone(),
                    },
                    work: Vec::new(),
                    retention: StepRetention {
                        candidate: crate::workflow::numerics::native_use(
                            report,
                            &pse_model::numerics::NumericalPolicy::default(),
                        ),
                        session: SessionDisposition::Discard,
                    },
                    observation,
                    cause: None,
                }
            },
        );
        let assessment = result.assessment.as_ref().unwrap();
        let mut last = AutoObservation {
            awaiting_assessment: false,
            native: assessment.observation,
            original: Some(assessment.original.clone()),
            permission: Some(assessment.retention.candidate.usability),
        };
        assert_eq!(last.permission, Some(CandidateUse::Unusable));
        let report = result.value.as_ref().unwrap();
        let trace_with_events = |events| {
            Arc::new(strategy::Trace {
                owner: None,
                publication_request: None,
                declaration: declaration.clone(),
                original: contract.identity,
                backend: Some(Backend::Ipopt),
                profile: contract.identity,
                start: StartOrigin::Specification,
                starts: vec![StartOrigin::Specification],
                products: vec![Default::default()],
                events,
            })
        };
        let trace = trace_with_events(result.events.clone());
        assert_eq!(
            trace
                .events
                .iter()
                .filter(|event| event.kind == NumericalEventKind::Started)
                .count(),
            1
        );
        assert_eq!(
            trace
                .events
                .iter()
                .filter(|event| event.kind == NumericalEventKind::Finished
                    && event.phase == Phase::Assessment)
                .count(),
            1
        );
        // Deterministically observe the boundary after the actual engine assessment;
        // neither sleeping nor a second numerical attempt is needed to expire it.
        let expired = pse_kernels::ExecutionScope::new(
            scope.cancellation().clone(),
            Some(std::time::Instant::now()),
        );
        let deadline = Arc::new(ProblemError::Provider(expired.check().unwrap_err()));
        assert!(matches!(
            deadline.as_ref(),
            ProblemError::Provider(pse_kernels::ProviderError::Deadline)
        ));
        let actual = || {
            Some((
                super::super::solves::Outcome::Native(Box::new(report.clone())),
                (),
                trace.clone(),
            ))
        };
        let (outcome, (), retained) =
            finish_automatic_stop(deadline.clone(), true, Some(&last), actual(), trace.clone())
                .unwrap();
        assert!(Arc::ptr_eq(&retained, &trace));
        let super::super::solves::Outcome::Native(stopped) = outcome else {
            panic!("expected actual native time stop")
        };
        assert_eq!(stopped.termination.category, Termination::TimeLimit);
        assert!(stopped.candidate.is_none() && stopped.quality.is_none());
        assert!(
            !crate::workflow::numerics::native_use(&stopped, &Default::default()).permits_use()
        );
        assert_eq!((attempts.get(), assessments.get()), (1, 1));
        for result in [
            finish_automatic_stop(
                deadline.clone(),
                false,
                Some(&last),
                actual(),
                trace.clone(),
            ),
            finish_automatic_stop::<()>(deadline.clone(), true, Some(&last), None, trace.clone()),
        ] {
            let error = result.unwrap_err();
            assert_eq!(
                strategy::runtime_failure(&error),
                pse_model::generated::enums::NumericalAttemptObservation::ResourceExhausted
            );
            let MathRuntimeError::Strategy { trace: failed, .. } = error else {
                panic!("expected retained stop trace")
            };
            assert!(Arc::ptr_eq(&failed, &trace));
        }
        last.permission = Some(CandidateUse::Usable);
        assert!(
            finish_automatic_stop(deadline.clone(), true, Some(&last), actual(), trace.clone())
                .is_err()
        );
        last.permission = Some(CandidateUse::Unusable);
        let mut earlier = report.clone();
        earlier.termination.category = Termination::Success;
        assert!(
            finish_automatic_stop(
                deadline.clone(),
                true,
                Some(&last),
                Some((
                    super::super::solves::Outcome::Native(Box::new(earlier)),
                    (),
                    trace.clone()
                )),
                trace.clone()
            )
            .is_err()
        );
        let mut incomplete = trace.events.clone();
        incomplete.retain(|event| event.kind != NumericalEventKind::Started);
        assert!(
            finish_automatic_stop(
                deadline.clone(),
                true,
                Some(&last),
                Some((
                    super::super::solves::Outcome::Native(Box::new(report.clone())),
                    (),
                    trace_with_events(incomplete)
                )),
                trace.clone()
            )
            .is_err()
        );
        let mut disagreement = trace.events.clone();
        disagreement.last_mut().unwrap().original = Some(OriginalConclusion::Unavailable {
            cause: Arc::new(ProblemError::stopped(
                Termination::TimeLimit,
                "another assessment",
            )),
        });
        assert!(
            finish_automatic_stop(
                deadline,
                true,
                Some(&last),
                Some((
                    super::super::solves::Outcome::Native(Box::new(report.clone())),
                    (),
                    trace_with_events(disagreement)
                )),
                trace.clone()
            )
            .is_err()
        );
        expired.cancellation().store(true, Ordering::Release);
        let cancelled = Arc::new(ProblemError::Provider(expired.check().unwrap_err()));
        assert!(matches!(
            cancelled.as_ref(),
            ProblemError::Provider(pse_kernels::ProviderError::Cancelled)
        ));
        let error = finish_automatic_stop(cancelled, true, Some(&last), actual(), trace.clone())
            .unwrap_err();
        assert_eq!(
            strategy::runtime_failure(&error),
            pse_model::generated::enums::NumericalAttemptObservation::Cancelled
        );
        let MathRuntimeError::Strategy { trace: failed, .. } = error else {
            panic!("expected cancellation trace")
        };
        assert!(Arc::ptr_eq(&failed, &trace));
        assert_eq!((attempts.get(), assessments.get()), (1, 1));
    }

    #[test]
    fn actual_resource_stop_retains_current_report_but_never_a_stale_or_usable_result() {
        use super::super::strategy::{self, AutoDecision, AutoObservation, OriginalConclusion};
        use pse_backend_native::solve::{
            Assurance, Controls, Execution, NativeTermination, SolveReport, Termination,
        };
        use pse_model::{
            generated::enums::{CandidateRefusal, CandidateUse, NumericalEventKind},
            strategy::{Phase, Scope, StartOrigin, WorkCharge, WorkObservation},
        };
        let controls = Controls::default();
        let scope = pse_kernels::ExecutionScope::new(Arc::default(), None);
        let declaration = strategy::direct(&controls);
        let cause = Arc::new(ProblemError::memory("scripted native memory stop"));
        let contract = pse_backend_native::OracleContract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
            variables: Vec::new(),
            rows: Vec::new(),
        };
        let mut report = Some(SolveReport::new(
            Backend::Scip,
            &contract,
            NativeTermination {
                code: 0,
                name: "scripted.memory.stop".into(),
                message: None,
                category: Termination::ResourceExhausted,
                assurance: Assurance::None,
            },
            &Execution::new(scope.cancellation().clone(), &controls),
        ));
        let work = WorkObservation {
            attempts: 1,
            evaluations: Some(0),
            iterations: Some(0),
            factorizations: Some(0),
            proof_steps: Some(0),
        };
        let result = strategy::run(
            &declaration,
            &scope,
            |_| strategy::Facts {
                support: Default::default(),
                accuracy: Vec::new(),
                consumption: Vec::new(),
                reservation: Some(work),
                work_admitted: false,
                start: StartOrigin::Specification,
                inherited: false,
                connected: false,
                refusal: None,
            },
            |_, _| {
                let report = report.take().unwrap();
                Ok(strategy::Attempt {
                    evidence: Vec::new(),
                    observation: strategy::observe_native(&report),
                    value: report,
                    work: WorkCharge {
                        phase: Phase::Native,
                        scope: Scope::Task,
                        charging_owner: contract.identity,
                        observed: work,
                    },
                })
            },
            |report, observation| strategy::Assessment {
                auxiliary: false,
                original: OriginalConclusion::Unavailable {
                    cause: cause.clone(),
                },
                work: Vec::new(),
                retention: StepRetention {
                    candidate: crate::workflow::numerics::native_use(
                        report,
                        &pse_model::numerics::NumericalPolicy::default(),
                    ),
                    session: SessionDisposition::Discard,
                },
                observation,
                cause: None,
            },
        );
        assert!(result.terminal.is_none());
        let assessment = result.assessment.as_ref().unwrap();
        let mut last = AutoObservation {
            awaiting_assessment: false,
            native: assessment.observation,
            original: Some(assessment.original.clone()),
            permission: Some(assessment.retention.candidate.usability),
        };
        let report = result.value.as_ref().unwrap();
        assert!(report.candidate.is_none());
        assert!(report.quality.is_none());
        assert_eq!(last.permission, Some(CandidateUse::Unusable));
        assert!(
            assessment
                .retention
                .candidate
                .refusals
                .contains(&CandidateRefusal::NoCandidate)
        );
        assert!(
            assessment
                .retention
                .candidate
                .refusals
                .contains(&CandidateRefusal::FeasibilityUnavailable)
        );
        assert!(
            !assessment
                .retention
                .candidate
                .refusals
                .contains(&CandidateRefusal::Infeasible)
        );
        let request = pse_model::strategy::CompositionRequest::default();
        let AutoDecision::Stop { cause: Some(stop) } = strategy::next_automatic(
            &request,
            &declaration.start,
            &[],
            &Default::default(),
            Some(&last),
            result.work,
            false,
            None,
        ) else {
            panic!("actual native resource report must stop");
        };
        assert!(retains_stopped_native_report(
            true,
            Some(&last),
            &stop,
            report,
            &result.events
        ));
        assert!(
            !retains_stopped_native_report(false, Some(&last), &stop, report, &result.events),
            "a later binding cannot revive an earlier report"
        );
        let cutoff = Arc::new(ProblemError::memory("independent later task cutoff"));
        assert!(!retains_stopped_native_report(
            true,
            Some(&last),
            &cutoff,
            report,
            &result.events
        ));
        last.permission = Some(CandidateUse::Usable);
        assert!(!retains_stopped_native_report(
            true,
            Some(&last),
            &stop,
            report,
            &result.events
        ));
        last.permission = Some(CandidateUse::Unusable);
        let mut incomplete = result.events.clone();
        incomplete.retain(|event| event.kind != NumericalEventKind::Started);
        assert!(!retains_stopped_native_report(
            true,
            Some(&last),
            &stop,
            report,
            &incomplete
        ));
    }

    #[test]
    fn session_retention_requires_composed_permission_and_artifact_admission() {
        use pse_model::generated::enums::CandidateUse;
        for (use_kind, disposition, expected) in [
            (
                CandidateUse::Usable,
                SessionDisposition::RetainCompatible,
                true,
            ),
            (
                CandidateUse::QualifiedUnclosed,
                SessionDisposition::RetainCompatible,
                true,
            ),
            (
                CandidateUse::SeedOnly,
                SessionDisposition::RetainCompatible,
                false,
            ),
            (
                CandidateUse::Unusable,
                SessionDisposition::RetainCompatible,
                false,
            ),
            (CandidateUse::Usable, SessionDisposition::Discard, false),
        ] {
            let mut retained = Retained::default();
            retained
                .session(
                    Backend::Kinsol,
                    pse_backend_native::solve::ReusePolicy::AllowRebuild,
                    |_: &mut u64| Ok(execution::SessionUpdate::Compatible),
                    || Ok(42u64),
                )
                .unwrap();
            StepRetention {
                candidate: crate::workflow::numerics::CandidateDecision {
                    usability: use_kind,
                    qualifiers: vec![],
                    refusals: vec![],
                    bound: None,
                },
                session: disposition,
            }
            .apply(&mut retained);
            assert_eq!(
                !retained.is_empty(),
                expected,
                "{use_kind:?}/{disposition:?}"
            );
        }
    }

    /// Release the worker even if a test assertion unwinds.
    struct Gate(Arc<(Mutex<bool>, Condvar)>);
    impl Gate {
        fn new() -> Self {
            Self(Arc::new((Mutex::new(false), Condvar::new())))
        }
        fn release(&self) {
            *self.0.0.lock().unwrap() = true;
            self.0.1.notify_all();
        }
    }
    impl Drop for Gate {
        fn drop(&mut self) {
            self.release();
        }
    }
    fn wait_gate(gate: &Arc<(Mutex<bool>, Condvar)>) {
        let mut released = gate.0.lock().unwrap();
        while !*released {
            released = gate.1.wait(released).unwrap();
        }
    }
    async fn stopped_waiter_keeps_dispatched_capacity(abandon: bool) {
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let first = service.open_session().unwrap();
        let second = service.open_session().unwrap();
        let entered = Arc::new(tokio::sync::Notify::new());
        let gate = Gate::new();
        let stopped = Arc::new(AtomicBool::new(false));
        let cancel = crate::CancelSource::new();
        let seen = entered.clone();
        let held = gate.0.clone();
        let observed = stopped.clone();
        let mut waiting = Some(Box::pin(first.run(2, None, &cancel, move |_, flag, _| {
            seen.notify_one();
            wait_gate(&held);
            observed.store(flag.load(Ordering::Acquire), Ordering::Release);
            Ok(())
        })));
        assert!(futures_util::poll!(waiting.as_mut().unwrap().as_mut()).is_pending());
        entered.notified().await;
        if abandon {
            drop(waiting.take());
        } else {
            cancel.cancel();
            assert!(futures_util::poll!(waiting.as_mut().unwrap().as_mut()).is_pending());
        }
        assert_eq!(service.cpu.available_permits(), 0);
        let cancel_second = crate::CancelSource::new();
        let began = Arc::new(AtomicBool::new(false));
        let observed = began.clone();
        let mut next = Box::pin(second.run(2, None, &cancel_second, move |_, _, _| {
            observed.store(true, Ordering::Release);
            Ok(())
        }));
        assert!(futures_util::poll!(next.as_mut()).is_pending());
        assert!(!began.load(Ordering::Acquire));
        gate.release();
        if let Some(waiting) = waiting.take() {
            waiting.await.unwrap();
        }
        drop(waiting);
        next.await.unwrap();
        assert!(stopped.load(Ordering::Acquire));
        first.close().await;
        second.close().await;
        assert_eq!(service.cpu.available_permits(), service.cores);
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn serial_operation_retains_native_state_and_loans_saturated_population_once() {
        use driver::Dispatch;
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let session = service.open_session().unwrap();
        let mut idle = Vec::new();
        for _ in 1..service.policy.jobs {
            idle.push(service.open_session().unwrap());
        }
        assert_eq!(service.jobs.available_permits(), 0);
        let cancel = crate::CancelSource::new();
        let flag = Arc::new(AtomicBool::new(false));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let scope = pse_kernels::ExecutionScope::new(flag.clone(), Some(deadline));
        let entries = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let seen = entries.clone();
        let worker_service = service.clone();
        let worker_cancel = cancel.clone();
        let worker_scope = scope.clone();
        observed_native_entry(
            Arc::new(move |_| {
                seen.fetch_add(1, Ordering::AcqRel);
            }),
            session.sequence(scope, Default::default(), &cancel, move |local| {
                assert!(Arc::ptr_eq(&local.execution().cancel, &flag));
                assert_eq!(local.execution().scope()?.deadline(), Some(deadline));
                let started = local.execution().started;
                for expected in 1..=3 {
                    let actual = local.run_ready(Dispatch::run_task(
                        local,
                        1,
                        vec![Backend::Kinsol],
                        Some(deadline),
                        Some(worker_scope.clone()),
                        &worker_cancel,
                        move |retained, _, _| {
                            let (count, reused) = retained.session(
                                Backend::Kinsol,
                                pse_backend_native::solve::ReusePolicy::AllowRebuild,
                                |count: &mut usize| {
                                    *count += 1;
                                    Ok(execution::SessionUpdate::Compatible)
                                },
                                || Ok(1usize),
                            )?;
                            assert_eq!(reused, expected > 1);
                            Ok(*count)
                        },
                    ))?;
                    assert_eq!(actual, expected);
                    // The direct child receives this operation's ticket even when all
                    // four finite population slots belong to resident sessions.
                    let child_service = worker_service.clone();
                    let child_flag = flag.clone();
                    let child = local.preparation(worker_service.job_scoped(
                        1,
                        0,
                        pse_columnar::flight::FlightCancellation::default(),
                        Some(deadline),
                        move |_| {
                            assert_eq!(child_service.jobs.available_permits(), 0);
                            assert!(!child_flag.load(Ordering::Acquire));
                            Ok(expected)
                        },
                    ))?;
                    assert_eq!(child, expected);
                    assert_eq!(local.execution().started, started);
                    assert_eq!(local.execution().scope()?.deadline(), Some(deadline));
                }
                assert!(matches!(
                    local.run_ready(std::future::pending::<Result<(), MathRuntimeError>>()),
                    Err(MathRuntimeError::Infrastructure(_))
                ));
                Ok(())
            }),
        )
        .await
        .unwrap();
        assert_eq!(entries.load(Ordering::Acquire), 1);
        session.close().await;
        for resident in idle {
            resident.close().await;
        }
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
        assert_eq!(service.cpu.available_permits(), service.cores);
        assert_eq!(service.pool.reserved(), baseline);
    }

    #[tokio::test]
    async fn cancelled_serial_preparation_keeps_loan_and_storage_until_child_drain() {
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let session = service.open_session().unwrap();
        let mut idle = Vec::new();
        for _ in 1..service.policy.jobs {
            idle.push(service.open_session().unwrap());
        }
        let entered = Arc::new(tokio::sync::Notify::new());
        let gate = Gate::new();
        let held = gate.0.clone();
        let observed = entered.clone();
        let worker_service = service.clone();
        let cancel = crate::CancelSource::new();
        let flag = Arc::new(AtomicBool::new(false));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let scope = pse_kernels::ExecutionScope::new(flag.clone(), Some(deadline));
        let mut sequence =
            Box::pin(
                session.sequence(scope, Default::default(), &cancel, move |local| {
                    local.preparation(worker_service.job_scoped(
                        1,
                        4096,
                        pse_columnar::flight::FlightCancellation::default(),
                        Some(deadline),
                        move |_| {
                            observed.notify_one();
                            wait_gate(&held);
                            Ok(())
                        },
                    ))
                }),
            );
        assert!(futures_util::poll!(sequence.as_mut()).is_pending());
        entered.notified().await;
        let reserved = service.pool.reserved();
        cancel.cancel();
        assert!(futures_util::poll!(sequence.as_mut()).is_pending());
        assert!(flag.load(Ordering::Acquire));
        assert_eq!(service.jobs.available_permits(), 0);
        assert_eq!(
            service.active_jobs.available_permits(),
            service.policy.jobs - 1
        );
        assert_eq!(service.pool.reserved(), reserved);
        gate.release();
        assert!(sequence.await.is_err());
        session.close().await;
        for resident in idle {
            resident.close().await;
        }
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
        assert_eq!(service.cpu.available_permits(), service.cores);
        assert_eq!(service.pool.reserved(), baseline);
    }

    #[tokio::test]
    async fn abandoned_session_waiter_keeps_dispatched_cpu() {
        stopped_waiter_keeps_dispatched_capacity(true).await;
    }
    #[tokio::test]
    async fn cancelled_session_waiter_keeps_dispatched_cpu() {
        stopped_waiter_keeps_dispatched_capacity(false).await;
    }
    #[tokio::test]
    async fn failed_session_dispatch_returns_undispatched_cpu() {
        let service = super::super::tests::service();
        let mut session = service.open_session().unwrap();
        let (sender, receiver) = mpsc::channel();
        drop(receiver);
        session.sender = Some(sender);
        let cancelled = crate::CancelSource::new();
        assert!(matches!(
            session.run(2, None, &cancelled, |_, _, _| Ok(())).await,
            Err(MathRuntimeError::Infrastructure(_))
        ));
        assert_eq!(service.cpu.available_permits(), service.cores);
        session.close().await;
    }
    #[tokio::test]
    async fn compute_owner_wait_uses_original_clock_and_shared_permit() {
        use execution::ComputeAdmission;
        let service = super::super::tests::service();
        let permit = service.try_active(1).unwrap().unwrap();
        let owner = ComputeOwner::new(
            service.active_jobs.clone(),
            service.cpu.clone(),
            1,
            permit,
            tokio::runtime::Handle::current(),
        );
        let retained = owner.clone();
        let (ready, waiting) = tokio::sync::oneshot::channel();
        let gate = Gate::new();
        let held = gate.0.clone();
        let thread = std::thread::spawn(move || {
            let mut owner = owner;
            owner.pause();
            ready.send(()).unwrap();
            wait_gate(&held);
            let controls = pse_backend_native::solve::Controls {
                time_limit: std::time::Duration::from_millis(20),
                ..Default::default()
            };
            owner.resume(&pse_backend_native::solve::Execution::new(
                Arc::default(),
                &controls,
            ))
        });
        waiting.await.unwrap();
        assert_eq!(service.cpu.available_permits(), service.cores);
        let occupied = service
            .cpu
            .clone()
            .acquire_many_owned(service.cores as u32)
            .await
            .unwrap();
        gate.release();
        let result = tokio::task::spawn_blocking(move || thread.join().unwrap())
            .await
            .unwrap();
        assert!(matches!(
            result,
            Err(ProblemError::Limit {
                kind: pse_backend_native::LimitKind::Time,
                ..
            })
        ));
        assert_eq!(service.cpu.available_permits(), 0);
        drop(occupied);
        drop(retained);
        assert_eq!(service.cpu.available_permits(), service.cores);
    }
    #[tokio::test]
    async fn same_session_dispatch_waiter_takes_no_compute_before_parent_resume() {
        let service = super::super::tests::service();
        let session = service.open_session().unwrap();
        let cancel = crate::CancelSource::new();
        let entered = Arc::new(tokio::sync::Notify::new());
        let seen = entered.clone();
        let gate = Gate::new();
        let held = gate.0.clone();
        let mut first = Box::pin(session.run(2, None, &cancel, move |_, flag, _| {
            execution::pause_compute();
            seen.notify_one();
            wait_gate(&held);
            let execution =
                pse_backend_native::solve::Execution::new(flag.clone(), &Default::default());
            execution::resume_compute(&execution)?;
            Ok(())
        }));
        assert!(futures_util::poll!(first.as_mut()).is_pending());
        entered.notified().await;
        let mut next = Box::pin(session.run(2, None, &cancel, |_, _, _| Ok(())));
        assert!(futures_util::poll!(next.as_mut()).is_pending());
        assert_eq!(service.cpu.available_permits(), service.cores);
        assert_eq!(service.active_jobs.available_permits(), service.policy.jobs);
        gate.release();
        tokio::time::timeout(std::time::Duration::from_secs(1), first)
            .await
            .unwrap()
            .unwrap();
        next.await.unwrap();
        session.close().await;
    }
    #[tokio::test]
    async fn idle_pressure_destroys_optional_state_before_capacity_can_progress() {
        use datafusion::execution::memory_pool::{MemoryConsumer, MemoryLimit};
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let session = service.open_session().unwrap();
        let gate = Gate::new();
        let held = gate.0.clone();
        let entered = Arc::new(tokio::sync::Notify::new());
        let seen = entered.clone();
        let optional = service
            .reserve("optional-session-control", 64 << 20)
            .unwrap();
        session
            .run(
                1,
                None,
                &crate::CancelSource::new(),
                move |retained, _, _| {
                    retained.session(
                        Backend::Ipopt,
                        pse_backend_native::solve::ReusePolicy::AllowRebuild,
                        |_: &mut NativeState| Ok(execution::SessionUpdate::Compatible),
                        || {
                            Ok(NativeState {
                                entered: seen,
                                gate: held,
                            })
                        },
                    )?;
                    retained.charge_session(Box::new(optional));
                    Ok(())
                },
            )
            .await
            .unwrap();
        let MemoryLimit::Finite(capacity) = service.pool.memory_limit() else {
            panic!("finite pool control");
        };
        let fill = MemoryConsumer::new("required-capacity-control").register(&service.pool);
        fill.try_grow(capacity - service.pool.reserved()).unwrap();
        let pending = service.submit(1, 0, |_, _| Ok(((), 0))).unwrap();
        let mut pending = Box::pin(pending.finish());
        assert!(futures_util::poll!(pending.as_mut()).is_pending());
        tokio::time::timeout(std::time::Duration::from_secs(1), entered.notified())
            .await
            .unwrap();
        assert_eq!(service.pool.reserved(), capacity);
        assert_eq!(service.cpu.available_permits(), service.cores);
        assert_eq!(service.active_jobs.available_permits(), service.policy.jobs);
        gate.release();
        let result = tokio::time::timeout(std::time::Duration::from_secs(1), pending)
            .await
            .unwrap()
            .unwrap();
        drop(result);
        drop(fill);
        session.close().await;
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[tokio::test]
    async fn compute_try_resume_never_holds_partial_active_admission() {
        use execution::{ComputeAdmission, ComputeAvailability};
        let service = super::super::tests::service();
        let active = service.try_active(2).unwrap().unwrap();
        let mut owner = ComputeOwner::new(
            service.active_jobs.clone(),
            service.cpu.clone(),
            2,
            active,
            tokio::runtime::Handle::current(),
        );
        owner.pause();
        assert_eq!(service.active_jobs.available_permits(), service.policy.jobs);
        let one_cpu = service.cpu.clone().acquire_owned().await.unwrap();
        let execution =
            pse_backend_native::solve::Execution::new(Arc::default(), &Default::default());
        assert_eq!(
            owner.try_resume(&execution).unwrap(),
            ComputeAvailability::Unavailable
        );
        assert_eq!(service.active_jobs.available_permits(), service.policy.jobs);
        assert_eq!(service.cpu.available_permits(), service.cores - 1);
        drop(one_cpu);
        assert_eq!(
            owner.try_resume(&execution).unwrap(),
            ComputeAvailability::Acquired
        );
        assert_eq!(
            service.active_jobs.available_permits(),
            service.policy.jobs - 1
        );
        owner.pause();
        execution.cancel.store(true, Ordering::Release);
        assert!(matches!(
            owner.try_resume(&execution),
            Err(ProblemError::Cancelled)
        ));
        service.active_jobs.close();
        let execution =
            pse_backend_native::solve::Execution::new(Arc::default(), &Default::default());
        assert!(owner.try_resume(&execution).is_err());
        assert_eq!(service.cpu.available_permits(), service.cores);
    }
    #[cfg(feature = "solver-pounce")]
    #[tokio::test]
    async fn persistent_scope_stacks_survive_idle_and_nested_scope_replacement() {
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let session = service.open_session().unwrap();
        let base = service.pool.reserved();
        let cancel = crate::CancelSource::new();
        session
            .run(2, Some(Backend::Pounce), &cancel, |_, _, _| Ok(()))
            .await
            .unwrap();
        assert_eq!(
            service.pool.reserved(),
            base + 2 * service.policy.stack_bytes
        );
        session
            .run(2, Some(Backend::PounceConvex), &cancel, |_, _, _| Ok(()))
            .await
            .unwrap();
        assert_eq!(
            service.pool.reserved(),
            base + 4 * service.policy.stack_bytes
        );
        session
            .run(1, Some(Backend::Pounce), &cancel, |_, _, _| Ok(()))
            .await
            .unwrap();
        assert_eq!(service.pool.reserved(), base);
        session.close().await;
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[cfg(feature = "solver-pounce")]
    #[tokio::test]
    async fn persistent_scope_reserves_before_active_and_restores_session() {
        use datafusion::execution::memory_pool::MemoryConsumer;
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let session = service.open_session().unwrap();
        let base = service.pool.reserved();
        let datafusion::execution::memory_pool::MemoryLimit::Finite(capacity) =
            service.pool.memory_limit()
        else {
            panic!("scope control requires its finite pool");
        };
        let held = MemoryConsumer::new("scope-refusal-control").register(&service.pool);
        held.try_grow(capacity - base).unwrap();
        let entered = Arc::new(AtomicBool::new(false));
        let observed = entered.clone();
        let cancel = crate::CancelSource::new();
        let mut waiting =
            Box::pin(
                session.run(2, Some(Backend::Pounce), &cancel, move |_, _, _| {
                    observed.store(true, Ordering::Release);
                    Ok(())
                }),
            );
        assert!(futures_util::poll!(waiting.as_mut()).is_pending());
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        assert!(!entered.load(Ordering::Acquire));
        assert_eq!(service.cpu.available_permits(), service.cores);
        assert_eq!(service.active_jobs.available_permits(), service.policy.jobs);
        drop(held);
        waiting.await.unwrap();
        assert!(entered.load(Ordering::Acquire));
        session.close().await;
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[cfg(feature = "solver-pounce")]
    #[tokio::test]
    async fn persistent_scope_stacks_survive_cancel_panic_and_owning_join() {
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let session = service.open_session().unwrap();
        let base = service.pool.reserved();
        let entered = Arc::new(tokio::sync::Notify::new());
        let seen = entered.clone();
        let gate = Gate::new();
        let held = gate.0.clone();
        let cancel = crate::CancelSource::new();
        let mut waiting =
            Box::pin(
                session.run::<()>(2, Some(Backend::Pounce), &cancel, move |_, flag, _| {
                    seen.notify_one();
                    wait_gate(&held);
                    assert!(flag.load(Ordering::Acquire));
                    panic!("persistent team panic after cancellation");
                }),
            );
        assert!(futures_util::poll!(waiting.as_mut()).is_pending());
        entered.notified().await;
        cancel.cancel();
        assert!(futures_util::poll!(waiting.as_mut()).is_pending());
        assert_eq!(
            service.pool.reserved(),
            base + 2 * service.policy.stack_bytes
        );
        assert_eq!(service.cpu.available_permits(), 0);
        gate.release();
        assert!(matches!(waiting.await, Err(MathRuntimeError::Panic(_))));
        assert_eq!(
            service.pool.reserved(),
            base + 2 * service.policy.stack_bytes
        );
        session.close().await;
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[tokio::test]
    async fn supplied_task_scope_keeps_owner_and_normal_completion_does_not_cancel() {
        let service = super::super::tests::service();
        let session = service.open_session().unwrap();
        let flag = Arc::new(AtomicBool::new(false));
        let scope = pse_kernels::ExecutionScope::new(
            flag.clone(),
            Some(std::time::Instant::now() + std::time::Duration::from_secs(2)),
        );
        let original = flag.clone();
        let cancel = crate::CancelSource::new();
        session
            .run_task(
                1,
                Vec::new(),
                scope.deadline(),
                Some(scope.clone()),
                &cancel,
                move |_, actual, _| {
                    assert!(Arc::ptr_eq(actual, &original));
                    Ok(())
                },
            )
            .await
            .unwrap();
        assert!(!flag.load(Ordering::Acquire));
        let held = service
            .cpu
            .clone()
            .acquire_many_owned(service.cores as u32)
            .await
            .unwrap();
        let entered = Arc::new(AtomicBool::new(false));
        let observed = entered.clone();
        {
            let waiting = session.run_task(
                1,
                Vec::new(),
                scope.deadline(),
                Some(scope),
                &cancel,
                move |_, _, _| {
                    observed.store(true, Ordering::Release);
                    Ok(())
                },
            );
            tokio::pin!(waiting);
            tokio::select! {
                result=&mut waiting=>panic!("task dispatched without available CPU: {result:?}"),
                ()=tokio::time::sleep(std::time::Duration::from_millis(20))=>{},
            }
            flag.store(true, Ordering::Release);
            assert!(matches!(
                waiting.await,
                Err(MathRuntimeError::Solve(ProblemError::Provider(
                    pse_kernels::ProviderError::Cancelled
                )))
            ));
        }
        assert!(!entered.load(Ordering::Acquire));
        drop(held);
        session.close().await;
        assert_eq!(service.cpu.available_permits(), service.cores);
    }
    #[tokio::test]
    async fn queued_native_step_expires_without_dispatch_or_user_cancellation() {
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let held = service
            .cpu
            .clone()
            .acquire_many_owned(service.cores as u32)
            .await
            .unwrap();
        let session = service.open_session().unwrap();
        let cancel = crate::CancelSource::new();
        let began = Arc::new(AtomicBool::new(false));
        let observed = began.clone();
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(20);
        let result = session
            .run_scoped(1, Vec::new(), Some(deadline), &cancel, move |_, _, _| {
                observed.store(true, Ordering::Release);
                Ok(())
            })
            .await;
        assert!(matches!(
            result,
            Err(MathRuntimeError::Solve(ProblemError::Limit {
                kind: pse_backend_native::LimitKind::Time,
                ..
            }))
        ));
        assert!(!began.load(Ordering::Acquire));
        assert!(cancel.checkpoint().is_ok());
        assert_eq!(service.cpu.available_permits(), 0);
        drop(held);
        session.close().await;
        assert_eq!(service.pool.reserved(), baseline);
    }
    struct NativeState {
        entered: Arc<tokio::sync::Notify>,
        gate: Arc<(Mutex<bool>, Condvar)>,
    }
    impl Drop for NativeState {
        fn drop(&mut self) {
            self.entered.notify_one();
            wait_gate(&self.gate);
        }
    }
    #[tokio::test]
    async fn panicked_session_step_keeps_cpu_through_state_cleanup_and_can_reuse() {
        let service = super::super::tests::service();
        let session = service.open_session().unwrap();
        let entered = Arc::new(tokio::sync::Notify::new());
        let gate = Gate::new();
        let seen = entered.clone();
        let held = gate.0.clone();
        let cancelled = crate::CancelSource::new();
        let mut waiting =
            Box::pin(
                session.run::<()>(2, None, &cancelled, move |retained, _, _| {
                    // Only the retained-state owner is exercised; no adapter or solver is invoked.
                    retained.session(
                        Backend::Ipopt,
                        pse_backend_native::solve::ReusePolicy::AllowRebuild,
                        |_: &mut NativeState| Ok(execution::SessionUpdate::Compatible),
                        || {
                            Ok(NativeState {
                                entered: seen,
                                gate: held,
                            })
                        },
                    )?;
                    panic!("gated staged-worker panic control");
                }),
            );
        assert!(futures_util::poll!(waiting.as_mut()).is_pending());
        entered.notified().await;
        assert_eq!(service.cpu.available_permits(), 0);
        gate.release();
        assert!(matches!(waiting.await, Err(MathRuntimeError::Panic(_))));
        session
            .run(2, None, &cancelled, |retained, _, _| {
                assert!(retained.is_empty());
                Ok(())
            })
            .await
            .unwrap();
        session.close().await;
        assert_eq!(service.cpu.available_permits(), service.cores);
    }
}
