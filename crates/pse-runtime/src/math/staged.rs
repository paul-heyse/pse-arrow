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

/// One step's work on the session thread; it reports its own result.
type Work = Box<dyn FnOnce(Result<&mut Retained, ProblemError>) + Send>;
struct Request {
    /// Admitted native threads of this step, which size adapter scopes.
    threads: usize,
    /// Adapter whose scope the step's native state must live in.
    backend: Option<Backend>,
    work: Work,
    /// Successful dispatch transfers CPU admission to the native request, including
    /// scope entry and required state cleanup. A waiter never owns dispatched capacity.
    _permits: tokio::sync::OwnedSemaphorePermit,
}
/// Adapter scopes currently entered on the session thread.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Scope {
    backends: Vec<Backend>,
    threads: usize,
}
impl Scope {
    fn admits(&self, request: &Request) -> bool {
        request.threads == self.threads
            && request.backend.is_none_or(|b| self.backends.contains(&b))
    }
    fn extended(&self, request: &Request) -> Self {
        let mut backends = self.backends.clone();
        if let Some(backend) = request.backend
            && !backends.contains(&backend)
        {
            backends.push(backend);
        }
        Self {
            backends,
            threads: request.threads,
        }
    }
}
/// Serve steps until the session closes. Native state is built and torn down inside the
/// scopes of the adapters that own it; a step needing another scope ends the current one
/// (dropping retained state there) and re-enters with the extended set.
fn serve(receiver: &std::sync::Mutex<mpsc::Receiver<Request>>, stack: usize) {
    let mut pending: Option<Request> = None;
    let mut scope = Scope {
        backends: Vec::new(),
        threads: 1,
    };
    loop {
        let adapters: Vec<&dyn BackendExecution> = scope
            .backends
            .iter()
            .map(|b| execution::adapter(*b))
            .collect();
        let entered = scope.clone();
        let exit = execution::scoped(&adapters, scope.threads, stack, || {
            let mut retained = Retained::default();
            loop {
                let request = match pending.take() {
                    Some(request) => request,
                    // Only this thread receives; the lock lends the receiver to the scope.
                    None => match receiver.lock().map(|r| r.recv()) {
                        Ok(Ok(request)) => request,
                        Ok(Err(_)) | Err(_) => return Ok::<_, ProblemError>(None),
                    },
                };
                if !entered.admits(&request) {
                    let next = entered.extended(&request);
                    pending = Some(request);
                    return Ok(Some(next));
                }
                (request.work)(Ok(&mut retained));
            }
        });
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
struct Stop(Arc<AtomicBool>);
impl Drop for Stop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}
/// A worker-owned native session shared by the steps of one staged sequence.
#[derive(Debug)]
pub(crate) struct NativeSession {
    service: Arc<MathService>,
    sender: Option<mpsc::Sender<Request>>,
    joined: Option<tokio::sync::oneshot::Receiver<()>>,
    budget: Arc<WorkerBudget>,
}
impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("threads", &self.threads)
            .field("backend", &self.backend)
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
        let slot = self
            .jobs
            .clone()
            .try_acquire_owned()
            .map_err(|_| MathRuntimeError::Limit("native jobs"))?;
        self.session_on(slot)
    }
    /// Open a native session for durable work: wait for a job slot instead of refusing
    /// (ADR-0112 Outcome 14). The attempt stays queued while it waits.
    ///
    /// # Errors
    /// Pool capacity, a closed admission, or the thread could not start.
    pub(crate) async fn open_session_queued(
        self: &Arc<Self>,
    ) -> Result<NativeSession, MathRuntimeError> {
        let slot = self
            .jobs
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| MathRuntimeError::Limit("native job admission closed"))?;
        self.session_on(slot)
    }
    fn session_on(
        self: &Arc<Self>,
        slot: tokio::sync::OwnedSemaphorePermit,
    ) -> Result<NativeSession, MathRuntimeError> {
        // The session thread's inner-solve session cache is held within its lease (I14);
        // its workers draw from the pool as they are built.
        let sessions = self.policy.inner_session_bytes;
        let bytes = self
            .policy
            .stack_bytes
            .checked_add(self.policy.foreign_bytes)
            .and_then(|n| n.checked_add(sessions))
            .ok_or(MathRuntimeError::Limit("native allowance overflow"))?;
        let lease = self.reserve("math:native-session", bytes)?;
        let (sender, receiver) = mpsc::channel::<Request>();
        let stack = self.policy.stack_bytes;
        let thread = std::thread::Builder::new()
            .name("pse-math".into())
            .stack_size(stack)
            .spawn(move || {
                #[cfg(feature = "solver-kinsol")]
                pse_backend_native::implicit::budget_sessions(sessions);
                #[cfg(not(feature = "solver-kinsol"))]
                let _ = sessions;
                serve(&std::sync::Mutex::new(receiver), stack)
            })
            .map_err(|e| MathRuntimeError::Infrastructure(e.to_string()))?;
        let (joined, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            // Joining, not the last reply, witnesses thread-local destruction.
            let _ = tokio::task::spawn_blocking(move || thread.join()).await;
            drop(lease);
            drop(slot);
            let _ = joined.send(());
        });
        Ok(NativeSession {
            service: self.clone(),
            sender: Some(sender),
            joined: Some(receiver),
            budget: WorkerBudget::drawing(self.policy.worker_bytes, &self.pool),
        })
    }
}
impl NativeSession {
    /// Run one step on the session thread with the retained native state, under its own
    /// CPU admission of `cores` permits. `backend` names the adapter whose scope the step's
    /// native state belongs to. Cancelling `cancel` (or dropping the returned future) sets
    /// the step's stop flag; the session stays usable for later steps.
    ///
    /// # Errors
    /// Admission failure, cancellation before admission, a closed session or the step's own
    /// error.
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
        if cores == 0 || cores > self.service.cores || cores > u32::MAX as usize {
            return Err(MathRuntimeError::Limit("optimizer cores"));
        }
        let sender = self
            .sender
            .as_ref()
            .ok_or_else(|| MathRuntimeError::Infrastructure("closed native session".into()))?;
        let permits = tokio::select! {
            permits = self.service.cpu.clone().acquire_many_owned(cores as u32) => {
                permits.map_err(|_| MathRuntimeError::Limit("CPU admission closed"))?
            }
            () = cancel.cancelled() => return Err(MathRuntimeError::Cancelled),
        };
        let flag = Arc::new(AtomicBool::new(false));
        let stop = Stop(flag.clone());
        let budget = self.budget.clone();
        let (reply, result) = tokio::sync::oneshot::channel();
        let request = Request {
            threads: cores,
            backend,
            work: Box::new(move |retained| {
                let outcome = match retained {
                    Ok(retained) => {
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            work(retained, &flag, &budget)
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
                };
                let _ = reply.send(outcome);
            }),
            _permits: permits,
        };
        sender
            .send(request)
            .map_err(|_| MathRuntimeError::Infrastructure("lost native session".into()))?;
        tokio::pin!(result);
        let outcome = tokio::select! {
            outcome = &mut result => outcome,
            () = cancel.cancelled() => {
                stop.0.store(true, Ordering::Release);
                result.await
            }
        };
        drop(stop);
        outcome.map_err(|_| MathRuntimeError::Infrastructure("lost native session step".into()))?
    }
    /// Execute one bound solve step on the retained native state and `assess` its outcome
    /// on the same worker, charged to the session's worker share (A6). A foreign allowance
    /// the step declares is reserved on admission and released when its work ends on the
    /// session thread; the session holds the deployment's. Native events go to
    /// `progress`: one stream per step, or one shared by an authored sequence and its run
    /// handle. `assess` returns its product and whether the original-model assessment
    /// accepted the candidate. A step that does not end with a usable, accepted candidate
    /// drops the retained native state, so it cannot poison a later step; the sequence
    /// itself continues.
    ///
    /// # Errors
    /// Admission, cancellation before admission or a lost session.
    #[expect(
        clippy::too_many_arguments,
        reason = "one staged step binds its solve, predecessor, attempt, progress, lease, cancellation and assessment"
    )]
    pub(crate) async fn step<T: Send + 'static>(
        &self,
        step: super::solves::PreparedSolve,
        previous: Option<super::solves::Predecessor>,
        attempt: usize,
        progress: Arc<Progress>,
        owner: Arc<pse_columnar::AllocationLease>,
        cancel: &crate::CancelSource,
        assess: impl FnOnce(
            &super::solves::Outcome,
            Option<&pse_kernels::ExecutionScope>,
            &Arc<WorkerBudget>,
        ) -> (T, bool)
        + Send
        + 'static,
    ) -> Result<(super::solves::Outcome, T), MathRuntimeError> {
        let service = self.service.clone();
        let (threads, backend) = (step.threads(), step.backend());
        let allowance = service.reserve("math:step-foreign", step.declared_foreign_bytes())?;
        self.run(threads, backend, cancel, move |retained, flag, budget| {
            let _allowance = allowance;
            let (outcome, scope) = service.execute(
                step, previous, attempt, retained, flag, &progress, budget, &owner,
            )?;
            let (assessed, accepted) = assess(&outcome, scope.as_ref(), budget);
            if !accepted {
                retained.clear();
            }
            Ok((outcome, assessed))
        })
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
        ) -> (T, bool)
        + Send
        + 'static,
    ) -> Result<Vec<Result<(super::solves::Outcome, T), MathRuntimeError>>, MathRuntimeError> {
        let Some(first) = members.first() else {
            return Ok(Vec::new());
        };
        let service = self.service.clone();
        let (threads, backend) = (first.step.threads(), first.step.backend());
        let declared = members
            .iter()
            .try_fold(0usize, |n, m| {
                n.checked_add(m.step.declared_foreign_bytes())
            })
            .ok_or(MathRuntimeError::Limit("native allowance overflow"))?;
        let allowance = service.reserve("math:step-foreign", declared)?;
        self.run(threads, backend, cancel, move |retained, flag, budget| {
            let _allowance = allowance;
            let outcomes = service.execute_batch(members, retained, flag, &progress, budget);
            let mut kept = true;
            let assessed = outcomes
                .into_iter()
                .enumerate()
                .map(|(i, outcome)| {
                    outcome.map(|(outcome, scope)| {
                        let (assessed, accepted) = assess(i, &outcome, scope.as_ref(), budget);
                        kept &= accepted;
                        (outcome, assessed)
                    })
                })
                .collect::<Vec<_>>();
            if !kept {
                retained.clear();
            }
            Ok(assessed)
        })
        .await
    }
    /// Close the session and wait until its thread has joined.
    pub(crate) async fn close(mut self) {
        self.sender.take();
        if let Some(joined) = self.joined.take() {
            let _ = joined.await;
        }
    }
}
impl Drop for NativeSession {
    fn drop(&mut self) {
        // Without an explicit close the thread still exits after its last step and the
        // supervisor joins it before releasing the admission.
        self.sender.take();
    }
}

#[cfg(test)]
mod admission_tests {
    use super::*;
    use std::sync::{Condvar, Mutex};

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
                        |_: &mut NativeState| Ok(true),
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
