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
struct Request {
    /// Admitted native threads of this step, which size adapter scopes.
    threads: usize,
    /// Adapter whose scope the step's native state must live in.
    backends: Vec<Backend>,
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
            && request.backends.iter().all(|b| self.backends.contains(b))
    }
    fn extended(&self, request: &Request) -> Self {
        let mut backends = self.backends.clone();
        for backend in &request.backends {
            if !backends.contains(backend) {
                backends.push(*backend);
            }
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
    service: Arc<MathService>,
    sender: Option<mpsc::Sender<Request>>,
    joined: Option<tokio::sync::oneshot::Receiver<()>>,
    budget: Arc<WorkerBudget>,
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
        if cores == 0 || cores > self.service.cores || cores > u32::MAX as usize {
            return Err(MathRuntimeError::Limit("optimizer cores"));
        }
        let sender = self
            .sender
            .as_ref()
            .ok_or_else(|| MathRuntimeError::Infrastructure("closed native session".into()))?;
        let expires = async {
            match deadline {
                Some(at) => tokio::time::sleep_until(at.into()).await,
                None => std::future::pending::<()>().await,
            }
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
        let permits = tokio::select! {
            permits = self.service.cpu.clone().acquire_many_owned(cores as u32) => {
                permits.map_err(|_| MathRuntimeError::Limit("CPU admission closed"))?
            }
            () = cancel.cancelled() => {
                if let Some(scope)=&enclosing {scope.cancellation().store(true,Ordering::Release);}
                return Err(MathRuntimeError::Cancelled);
            },
            () = expires => return Err(ProblemError::Limit {
                kind:pse_backend_native::LimitKind::Time,
                detail:"task deadline while waiting for native CPU admission".into(),
            }.into()),
            error=task_stop=>return Err(ProblemError::Provider(error).into()),
        };
        let flag = enclosing.as_ref().map_or_else(
            || Arc::new(AtomicBool::new(false)),
            |scope| scope.cancellation().clone(),
        );
        let mut stop = Stop(Some(flag.clone()));
        let budget = self.budget.clone();
        let (reply, result) = tokio::sync::oneshot::channel();
        let request = Request {
            threads: cores,
            backends,
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
    /// Resolve one owner-bound operation at a time. Async preparation never runs inside
    /// the native worker, and no retry obtains a new task clock or work allowance.
    #[expect(
        clippy::too_many_arguments,
        reason = "the common step binds its admitted scientific task"
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
        use pse_model::strategy::{CompositionPolicy, Mechanism, Position, Transition};
        let request = step.composition_request().clone();
        request
            .validate()
            .map_err(|error| ProblemError::Contract(error.to_string()))?;
        if step.composition_is_declared() {
            return self
                .step_bound(
                    step, previous, attempt, progress, owner, cancel, assess, None, None, None,
                    false, None,
                )
                .await;
        }
        if request.policy == CompositionPolicy::Declared {
            return Err(ProblemError::Contract(
                "declared composition requires an actually bound declaration".into(),
            )
            .into());
        }
        let deadline = std::time::Instant::now()
            .checked_add(step.time_limit())
            .ok_or_else(|| ProblemError::Contract("automatic task deadline extent".into()))?;
        let scope = step
            .task_scope()
            .unwrap_or_else(|| pse_kernels::ExecutionScope::new(Arc::default(), Some(deadline)));
        let step = step.within_task(scope.clone())?;
        let mut declaration = step.numerical_strategy();
        declaration.branch = request.branch;
        declaration.start.recovery = request.recovery.clone();
        if let Some(limits) = request.limits {
            declaration.limits = limits;
        }
        if request.limits.is_none() {
            declaration.limits.attempts = step.automatic_attempt_capacity(scope.cancellation())?;
        }
        let ledger = super::strategy::admission::TaskAdmission::new(
            declaration.limits,
            scope.clone(),
            Some(self.service.pool.clone()),
            step.declared_foreign_bytes() > 0,
        );
        let capacity = usize::try_from(declaration.limits.attempts)
            .map_err(|_| ProblemError::memory("automatic product state capacity"))?;
        let product_owner = self.service.reserve(
            "math:automatic-product-state",
            super::strategy::ProductState::extent(capacity)?,
        )?;
        let shared_products = Arc::new(std::sync::Mutex::new(
            super::strategy::ProductState::owned(capacity, product_owner),
        ));
        let assess = Arc::new(std::sync::Mutex::new(assess));
        let initial = step.entry_origin(previous.is_some());
        let inherited = previous.is_some() && initial == pse_model::strategy::StartOrigin::Accepted;
        let mut original_operations = step.automatic_operations(None, scope.cancellation())?;
        for operation in &mut original_operations {
            if !operation.candidate.replacement {
                operation.candidate.start = initial;
            }
        }
        let mut operations = original_operations;
        let mut bound_operations = std::collections::BTreeMap::new();
        let mut attempted = std::collections::BTreeSet::new();
        let mut attempted_bindings = std::collections::BTreeSet::new();
        let mut last_observation = None;
        let mut last_result = None;
        let mut opinions_bound = false;
        let mut collected = super::strategy::Trace {
            owner: None,
            declaration: declaration.clone(),
            original: step.original_identity()?,
            backend: step.backend(),
            profile: step.strategy_profile()?,
            start: initial,
            starts: Vec::new(),
            products: Vec::new(),
            events: Vec::new(),
        };
        collected.declaration.mechanisms.clear();
        loop {
            scope.check().map_err(ProblemError::Provider)?;
            let candidates = operations
                .iter()
                .map(|operation| operation.candidate.clone())
                .collect::<Vec<_>>();
            let mut constrained = request.clone();
            constrained.limits = Some(declaration.limits);
            let decision = super::strategy::next_automatic(
                &constrained,
                &declaration.start,
                &candidates,
                &attempted,
                last_observation.as_ref(),
                ledger.observation()?,
                inherited,
            );
            let resolved = super::strategy::automatic_decision_key(
                &request,
                &candidates,
                &decision,
                last_observation.as_ref(),
                ledger.observation()?,
            )?;
            let mut decision_identity =
                pse_ids::FramedHasher::new(pse_ids::Frame::NumericalDecisionV1);
            decision_identity
                .hash(&collected.original)
                .hash(&step.preparation_identity()?)
                .hash(&resolved);
            let decision_key = decision_identity.finish_hash();
            match decision {
                super::strategy::AutoDecision::Finish => break,
                super::strategy::AutoDecision::Assess => {
                    return Err(ProblemError::Internal(
                        "original assessment must execute on its admitted worker".into(),
                    )
                    .into());
                }
                super::strategy::AutoDecision::Stop { cause } => {
                    if let Some(cause) = cause
                        && let Some(index) = collected.declaration.mechanisms.len().checked_sub(1)
                    {
                        collected.events.push(super::strategy::Event {
                            mechanism: index,
                            kind: pse_model::generated::enums::NumericalEventKind::Abandoned,
                            phase: pse_model::strategy::Phase::Assessment,
                            original: None,
                            decision: Some(decision_key),
                            observation: Some(super::strategy::failure(&cause)),
                            transition: Some(Transition::Stop),
                            permission: None,
                            work: None,
                            cause: Some(cause),
                        });
                    }
                    break;
                }
                super::strategy::AutoDecision::Prepare { candidate }
                | super::strategy::AutoDecision::Dispatch { candidate } => {
                    let preparing =
                        matches!(decision, super::strategy::AutoDecision::Prepare { .. });
                    if !preparing {
                        attempted.insert(candidate);
                    }
                    let operation = operations[candidate].clone();
                    if !preparing && !attempted_bindings.insert(operation.candidate.identity) {
                        continue;
                    }
                    let description = operation.candidate.clone();
                    let mechanism = Mechanism {
                        kind: description.kind,
                        position: if preparing {
                            Position::Preparation
                        } else {
                            Position::Execution
                        },
                        required: true,
                        profile: None,
                        support: description.support.iter().copied().collect(),
                        limits: declaration.limits,
                        starts: vec![description.start],
                        transitions: vec![
                            Transition::Finish,
                            Transition::Recover,
                            Transition::Stop,
                        ],
                    };
                    let index = collected.declaration.mechanisms.len();
                    collected.declaration.mechanisms.push(mechanism.clone());
                    // Binding preparations reserve before effects; execution admission
                    // belongs to run_admitted and must not reserve the same work twice.
                    let admitted = if preparing {
                        ledger.reserve_attempt().and_then(|()| {
                            ledger.reserve(mechanism.limits, description.reservation, false)
                        })
                    } else {
                        Ok(())
                    };
                    let prepared = match admitted {
                        Ok(()) if bound_operations.contains_key(&candidate) => {
                            Ok(bound_operations.remove(&candidate).ok_or_else(|| {
                                ProblemError::Internal("selected preparation disappeared".into())
                            })?)
                        }
                        Ok(()) => {
                            self.service
                                .prepare_automatic_operation(operation, scope.clone(), cancel)
                                .await
                        }
                        Err(error) => Err(error.into()),
                    };
                    let prepared = match prepared {
                        Ok(prepared) => prepared,
                        Err(error) => {
                            let observation = super::strategy::runtime_failure(&error);
                            let recoverable=matches!(observation,pse_model::generated::enums::NumericalAttemptObservation::CapabilityRefusal);
                            let cause = Arc::new(error.into_problem());
                            collected.starts.push(description.start);
                            collected.products.push(Default::default());
                            collected.events.push(super::strategy::Event {
                                mechanism: index,
                                kind: pse_model::generated::enums::NumericalEventKind::Refused,
                                phase: pse_model::strategy::Phase::Preparation,
                                original: None,
                                observation: Some(observation),
                                transition: Some(if recoverable {
                                    Transition::Continue
                                } else {
                                    Transition::Stop
                                }),
                                permission: None,
                                work: None,
                                cause: Some(cause.clone()),
                                decision: Some(decision_key),
                            });
                            if recoverable {
                                attempted.insert(candidate);
                                attempted_bindings.insert(description.identity);
                                continue;
                            } else {
                                let bytes = collected.retained_bytes()?;
                                let trace = Arc::new(
                                    collected.with_owner(
                                        self.service
                                            .reserve("math:automatic-refusal-trace", bytes)?,
                                    ),
                                );
                                return Err(MathRuntimeError::Strategy {
                                    cause: Arc::new(
                                        ProblemError::Math(pse_math::MathError::Typed {
                                            retained: cause.retained_bytes(),
                                            cause:
                                                pse_model::diagnostic::DiagnosticCause::from_shared(
                                                    cause,
                                                ),
                                        })
                                        .into(),
                                    ),
                                    trace,
                                });
                            }
                        }
                    };
                    if preparing {
                        let mut work_identity =
                            pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
                        work_identity
                            .hash(&decision_key)
                            .str("automatic-preparation");
                        let work = pse_model::strategy::WorkCharge {
                            phase: pse_model::strategy::Phase::Preparation,
                            scope: pse_model::strategy::Scope::Task,
                            charging_owner: work_identity.finish_hash(),
                            observed: pse_model::strategy::WorkObservation {
                                attempts: 1,
                                evaluations: None,
                                iterations: None,
                                factorizations: None,
                                proof_steps: None,
                            },
                        };
                        let work = ledger.complete_charge(work)?;
                        operations[candidate].candidate.prepared = true;
                        bound_operations.insert(candidate, prepared);
                        collected.starts.push(description.start);
                        collected.products.push(Default::default());
                        collected.events.push(super::strategy::Event {
                            mechanism: index,
                            kind: pse_model::generated::enums::NumericalEventKind::Finished,
                            phase: pse_model::strategy::Phase::Preparation,
                            original: None,
                            observation: None,
                            transition: Some(Transition::Continue),
                            permission: None,
                            work: Some(work),
                            cause: None,
                            decision: Some(decision_key),
                        });
                        continue;
                    }
                    let assessor = assess.clone();
                    let one = self
                        .step_bound(
                            step.clone(),
                            previous.clone(),
                            attempt
                                .checked_add(index)
                                .ok_or_else(|| ProblemError::memory("automatic attempt ordinal"))?,
                            progress.clone(),
                            owner.clone(),
                            cancel,
                            move |outcome, scope, budget| {
                                match assessor.lock() {
                                    Ok(mut assess) => assess(outcome, scope, budget),
                                    Err(poisoned) => {
                                        // A poisoned scientific owner cannot be retried. The original
                                        // task propagates panic before further numerical work.
                                        let mut assess = poisoned.into_inner();
                                        let rejected = super::solves::Outcome::Rejected(Arc::new(
                                            MathRuntimeError::Panic(
                                                "scientific assessment owner panicked".into(),
                                            ),
                                        ));
                                        assess(&rejected, scope, budget)
                                    }
                                }
                            },
                            Some(prepared),
                            Some(mechanism),
                            Some(shared_products.clone()),
                            true,
                            Some(ledger.clone()),
                        )
                        .await?;
                    last_observation = one.2.events.iter().rev().find_map(|event| {
                        event
                            .observation
                            .map(|native| super::strategy::AutoObservation {
                                awaiting_assessment: false,
                                native,
                                original: event.original.clone(),
                                permission: event.permission,
                            })
                    });
                    let report = match &one.0 {
                        super::solves::Outcome::Native(report) => Some(report.as_ref()),
                        _ => None,
                    };
                    if !opinions_bound
                        && report.is_some()
                        && last_observation.as_ref().is_some_and(|last| {
                            !last
                                .original
                                .as_ref()
                                .is_some_and(super::strategy::OriginalConclusion::satisfied)
                        })
                    {
                        opinions_bound = true;
                        let more = step.automatic_operations(report, scope.cancellation())?;
                        operations.extend(more.into_iter().filter(|operation| {
                            !attempted_bindings.contains(&operation.candidate.identity)
                        }));
                    }
                    collected.starts.extend(one.2.starts.iter().copied());
                    collected.products.extend(one.2.products.iter().cloned());
                    collected
                        .events
                        .extend(one.2.events.iter().cloned().map(|mut event| {
                            event.mechanism = index;
                            if event.decision.is_none() {
                                event.decision = Some(decision_key);
                            }
                            event
                        }));
                    last_result = Some(one);
                }
            }
        }
        let bytes = collected.retained_bytes()?;
        let trace = Arc::new(
            collected.with_owner(
                self.service
                    .reserve("math:automatic-strategy-trace", bytes)?,
            ),
        );
        match last_result {
            Some((outcome, product, _)) => Ok((outcome, product, trace)),
            None => Err(MathRuntimeError::Strategy {
                cause: Arc::new(
                    ProblemError::Unsupported(
                        "no automatic operation satisfies the admitted task constraints".into(),
                    )
                    .into(),
                ),
                trace,
            }),
        }
    }

    /// Execute one bound solve step on the retained native state and `assess` its outcome
    /// on the same worker, charged to the session's worker share (A6). A foreign allowance
    /// the step declares is reserved on admission and retained with escaping native state;
    /// otherwise its work releases it on the session thread. The session also holds the deployment's. Native events go to
    /// `progress`: one stream per step, or one shared by an authored sequence and its run
    /// handle. `assess` returns its product, composed candidate permission and session
    /// admission. A step without both original permission and artifact admission
    /// drops the retained native state, so it cannot poison a later step; the sequence
    /// itself continues.
    ///
    /// # Errors
    /// Admission, cancellation before admission or a lost session.
    #[expect(
        clippy::too_many_arguments,
        reason = "one staged step binds its solve, predecessor, attempt, progress, lease, cancellation and assessment"
    )]
    async fn step_bound<T: Send + 'static>(
        &self,
        mut step: super::solves::PreparedSolve,
        previous: Option<super::solves::Predecessor>,
        attempt: usize,
        progress: Arc<Progress>,
        owner: Arc<pse_columnar::AllocationLease>,
        cancel: &crate::CancelSource,
        mut assess: impl FnMut(
            &super::solves::Outcome,
            Option<&pse_kernels::ExecutionScope>,
            &Arc<WorkerBudget>,
        ) -> super::strategy::Assessed<T>
        + Send
        + 'static,
        operation: Option<super::solves::PreparedRung>,
        mechanism: Option<pse_model::strategy::Mechanism>,
        shared_products: Option<super::strategy::SharedProducts>,
        original_completion: bool,
        shared_admission: Option<Arc<super::strategy::admission::TaskAdmission>>,
    ) -> Result<(super::solves::Outcome, T, super::solves::StrategyTrace), MathRuntimeError> {
        let service = self.service.clone();
        let (threads, backend) = (step.threads(), step.backend());
        let deadline = std::time::Instant::now()
            .checked_add(step.time_limit())
            .ok_or_else(|| ProblemError::Contract("task deadline extent".into()))?;
        let mut declaration = step.numerical_strategy();
        if let Some(mechanism) = mechanism {
            declaration.mechanisms = vec![mechanism];
            declaration.branch = step.composition_request().branch;
            declaration.start.recovery = step.composition_request().recovery.clone();
            if let Some(limits) = step.composition_request().limits {
                declaration.limits = limits;
            }
        }
        let original = step.original_identity()?;
        let profile = step.strategy_profile()?;
        let start = match declaration.start.policy {
            pse_model::strategy::StartPolicy::Explicit => {
                pse_model::strategy::StartOrigin::Explicit
            }
            pse_model::strategy::StartPolicy::PreviousAccepted if previous.is_some() => {
                pse_model::strategy::StartOrigin::Accepted
            }
            _ => pse_model::strategy::StartOrigin::Specification,
        };

        let declared_foreign = step.declared_foreign_bytes();
        let composition = step.take_composition();
        use super::solves::PreparedRung;
        let rungs = operation.map_or_else(
            || composition.map_or_else(|| vec![step.clone().into()], |c| c.rungs.clone()),
            |operation| vec![operation],
        );
        let start = rungs
            .first()
            .map_or(start, |rung| rung.entry_origin(previous.is_some()));
        let inherited = start == pse_model::strategy::StartOrigin::Accepted;
        let backends = rungs.iter().filter_map(PreparedRung::backend).collect();
        let threads = rungs
            .iter()
            .map(PreparedRung::threads)
            .max()
            .unwrap_or(threads);
        let task_scope = step
            .task_scope()
            .or_else(|| rungs.iter().find_map(PreparedRung::task_scope));
        let deadline = task_scope
            .as_ref()
            .and_then(pse_kernels::ExecutionScope::deadline)
            .map_or(deadline, |outer| outer.min(deadline));
        let products = rungs
            .iter()
            .map(|rung| super::strategy::RungProducts {
                provider: match rung {
                    PreparedRung::Surrogate(p) => {
                        Some(super::strategy::ProviderEvidence::Library(p.key()))
                    }
                    _ => None,
                },
                derived: match rung {
                    PreparedRung::Multistart(p) => Some(p.key()),
                    PreparedRung::Surrogate(p) => Some(p.key()),
                    PreparedRung::Derived(p) => Some(p.family().key()),
                    #[cfg(feature = "solver-kinsol")]
                    PreparedRung::Blocks(p) => Some(p.key()),
                    #[cfg(feature = "solver-kinsol")]
                    PreparedRung::Causal(p) => Some(p.key()),
                    PreparedRung::Path { prepared, .. } => Some(prepared.family().key()),
                    #[cfg(feature = "solver-petsc")]
                    PreparedRung::Petsc(p) => Some(p.key()),
                    PreparedRung::Original(_) => None,
                },
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let supports = rungs
            .iter()
            .map(PreparedRung::support)
            .collect::<Result<Vec<_>, _>>()?;

        let starts: Vec<_> = rungs
            .iter()
            .enumerate()
            .map(|(index, rung)| {
                if index == 0 {
                    start
                } else {
                    rung.entry_origin(false)
                }
            })
            .collect();
        let initial = if rungs
            .iter()
            .any(|rung| !matches!(rung, PreparedRung::Original(_) | PreparedRung::Path { .. }))
        {
            Some(step.source_start(previous.as_ref())?)
        } else {
            None
        };
        let product_owner = service.reserve(
            "math:actual-product-state",
            super::strategy::ProductState::extent(rungs.len())?,
        )?;
        let allowance = service.reserve("math:step-foreign", declared_foreign)?;
        self.run_task(threads, backends, Some(deadline), task_scope.clone(), cancel, move |retained, flag, budget| {
            enum RungOutcome { Original(Box<super::solves::ScopedOutcome>), Derived(Box<super::solves::DerivedAttempt>), Path(Box<super::solves::paths::PathOutcome>), Surrogate(Box<super::surrogate::SurrogatePhaseOutcome>) }
            let _allowance = allowance;
            let enclosing = pse_kernels::ExecutionScope::new(flag.clone(), Some(deadline));
            let admission=shared_admission.unwrap_or_else(||super::strategy::admission::TaskAdmission::new(declaration.limits,enclosing.clone(),Some(service.pool.clone()),declared_foreign>0));
            admission.retain_storage_allowance(_allowance.clone())?;
            let scoped_budget=budget.with_admission(admission.clone());
            let budget=&scoped_budget;
            let mut assessed = None;
            let product_state=shared_products.unwrap_or_else(||Arc::new(std::sync::Mutex::new(super::strategy::ProductState::owned(rungs.len(),product_owner))));
            let prepared=std::cell::RefCell::new(rungs.into_iter().map(Some).collect::<Vec<_>>());
            let mut previous = previous;
            let point=std::cell::RefCell::new(initial);
            let fresh_start=std::cell::Cell::new(None::<pse_model::strategy::StartOrigin>);
            let connected=std::cell::Cell::new(false);
            let completion_witness=std::cell::RefCell::new(None::<super::solves::paths::PathCompletion>);
            let actual_starts=std::cell::RefCell::new(starts);
            let actual_products=std::cell::RefCell::new(products);
            let result = super::strategy::run_admitted(
                &declaration, &enclosing, admission.clone(),
                |index| {
                    let declared=actual_starts.borrow()[index];
                    let fresh=fresh_start.get().filter(|origin|index>0 && !matches!(prepared.borrow()[index].as_ref(),Some(PreparedRung::Multistart(_))) && declared!=pse_model::strategy::StartOrigin::Explicit
                        && declaration.start.permits_recovery(*origin,false)
                        && declaration.mechanisms[index].starts.contains(origin));
                    let actual=fresh.unwrap_or(declared);
                    actual_starts.borrow_mut()[index]=actual;
                    let origin=prepared.borrow()[index].as_ref().and_then(|rung|match rung {PreparedRung::Path{prepared,start}=>Some(prepared.origin_connected(start)),_=>None});
                    let (bound,refusal)=match origin {Some(Ok(bound))=>(bound,None),Some(Err(error))=>(false,Some(Arc::new(error))),None=>(connected.get(),None)};
                    let consumed=if matches!(prepared.borrow()[index].as_ref(),Some(PreparedRung::Original(_))) && !declaration.accuracy.is_empty() {
                        let candidate=point.borrow();
                        match candidate.as_ref() {
                            Some(point)=>super::strategy::products(&product_state).and_then(|state|state.bind_inputs(&declaration.accuracy,original,pse_backend_native::square_response::point_key(point),declaration.branch,0).and_then(|contract|state.consume(&contract))),
                            None=>Err(ProblemError::Unsupported("original correction lacks an accuracy-qualified point".into())),
                        }
                    } else {Ok(Vec::new())};
                    let (accuracy,accuracy_refusal)=match consumed {Ok(accuracy)=>(accuracy,None),Err(error)=>(Vec::new(),Some(Arc::new(error)))};
                    let local_refusal=prepared.borrow()[index].as_ref().and_then(|rung|match rung {PreparedRung::Derived(p)=>p.local_scope_refusal(&enclosing),_=>None});
                    let work_admitted=prepared.borrow()[index].as_ref().is_some_and(|rung|match rung {PreparedRung::Original(target)=>target.work_admitted(declaration.limits,&enclosing,admission.clone()) && target.work_admitted(declaration.mechanisms[index].limits,&enclosing,admission.clone()),_=>false});
                    super::strategy::Facts { support:supports[index].clone(), accuracy, consumption:if matches!(prepared.borrow()[index].as_ref(),Some(PreparedRung::Original(_))) { declaration.accuracy.clone() } else {Vec::new()}, reservation:None, work_admitted, start:actual, inherited:index==0 && inherited, connected:bound, refusal:refusal.or(local_refusal).or(accuracy_refusal) }
                },
                |index,mechanism| {
                    let rung = prepared.borrow_mut()[index].take().ok_or_else(|| Arc::new(ProblemError::Internal("strategy mechanism executed twice".into())))?;
                    let input=match &rung {
                        PreparedRung::Original(p)=>if matches!(actual_starts.borrow()[index],pse_model::strategy::StartOrigin::Auxiliary|pse_model::strategy::StartOrigin::Surrogate) {point.borrow().clone()} else {p.source_start(previous.as_ref()).ok()},
                        PreparedRung::Multistart(p)=>Some(p.point()),
                        PreparedRung::Surrogate(_)=>point.borrow().clone(),
                        PreparedRung::Derived(p)=>if Some(actual_starts.borrow()[index])==fresh_start.get() {point.borrow().clone()} else {Some(p.original().source_start(previous.as_ref()).map_err(Arc::new)?)},
                        #[cfg(feature="solver-kinsol")]
                        PreparedRung::Blocks(p)=>Some(p.original().source_start(previous.as_ref()).map_err(Arc::new)?),
                        #[cfg(feature="solver-kinsol")]
                        PreparedRung::Causal(p)=>Some(p.original().source_start(previous.as_ref()).map_err(Arc::new)?),
                        PreparedRung::Path{start,..}=>Some(start.point().to_vec()),
                        #[cfg(feature="solver-petsc")] PreparedRung::Petsc(p)=>if Some(actual_starts.borrow()[index])==fresh_start.get() {point.borrow().clone()} else {Some(p.source().original().source_start(previous.as_ref()).map_err(Arc::new)?)},
                    };
                    let actual_input=input.clone();
                    if let Some(input)=input {
                        let mut identity=pse_ids::FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
                        identity.str("actual-original-coordinate-start").hash(&original);
                        for x in input {identity.f64(x);}
                        actual_products.borrow_mut()[index].start=Some(identity.finish_hash());
                    }
                    let (value,observation,observed)=match rung {
                        PreparedRung::Original(step)=>{
                            let mut step=*step;
                            if matches!(actual_starts.borrow()[index],pse_model::strategy::StartOrigin::Auxiliary|pse_model::strategy::StartOrigin::Surrogate) {
                                let current=point.borrow();
                                step=step.with_recovery_point(current.as_deref().ok_or_else(||Arc::new(ProblemError::Internal("screened auxiliary point absent".into())))?).map_err(Arc::new)?;
                            }
                            if let Some(backend)=step.backend() {
                                actual_products.borrow_mut()[index].provider=Some(super::strategy::ProviderEvidence::Native(pse_model::strategy::ProfileRef {backend,key:step.strategy_profile().map_err(Arc::new)?}));
                            }
                            let target=step.clone();
                            let mut value = service.execute(step, previous.take(), attempt, retained, flag, &progress, budget, &owner, &enclosing)
                                .unwrap_or_else(|error| (super::solves::Outcome::Rejected(Arc::new(error)), Some(enclosing.clone())));
                            if let Some(witness)=completion_witness.borrow().as_ref()
                                && let super::solves::Outcome::Native(report)=&mut value.0
                                && let Some(candidate)=&report.candidate {
                                let execution=pse_backend_native::solve::Execution::within(flag.clone(),&pse_backend_native::solve::Controls::default(),enclosing.clone()).map_err(Arc::new)?;
                                if let Err(cause)=witness.validate_original_candidate(&target,&candidate.primal,&execution) {
                                    report.record_validation_failure(cause);
                                    let failure_owner=service.reserve("math:path-completion-failure",report.failure_bytes()).map_err(|error|Arc::new(error.into_problem()))?;
                                    report.retain_failure_owner(failure_owner);
                                }
                            }
                            let observation=super::strategy::observe(&value.0);
                            let observed=target.inclusive_work(&value.0);
                            (RungOutcome::Original(Box::new(value)),observation,observed)
                        },
                        #[cfg(feature="solver-kinsol")]
                        PreparedRung::Blocks(prepared)=>{
                            let mut execution=pse_backend_native::solve::Execution::within(flag.clone(),prepared.original().controls(),enclosing.clone()).map_err(Arc::new)?;
                            execution.work_admission=budget.admission().map(|owner|->Arc<dyn pse_backend_native::solve::WorkAdmission>{owner});
                            execution.progress=progress.clone();execution.memory=Some(service.policy.foreign_allowance(prepared.original().controls()));
                            let outcome=service.blocks_step(&prepared,execution,retained,budget,actual_input.as_deref().ok_or_else(||Arc::new(ProblemError::Internal("complete original block start absent".into())))?)?;
                            let observed=super::strategy::work(&outcome);let observation=super::strategy::observe(&outcome);
                            (RungOutcome::Original(Box::new((outcome,Some(enclosing.clone())))),observation,observed)
                        },
                        #[cfg(feature="solver-kinsol")]
                        PreparedRung::Causal(prepared)=>{
                            let mut execution=pse_backend_native::solve::Execution::within(flag.clone(),prepared.original().controls(),enclosing.clone()).map_err(Arc::new)?;
                            execution.work_admission=budget.admission().map(|owner|->Arc<dyn pse_backend_native::solve::WorkAdmission>{owner});
                            execution.progress=progress.clone();execution.memory=Some(service.policy.foreign_allowance(prepared.original().controls()));
                            let outcome=service.causal_step(&prepared,execution,retained,budget,actual_input.as_deref().ok_or_else(||Arc::new(ProblemError::Internal("complete original causal start absent".into())))?)?;
                            let observed=super::strategy::work(&outcome);let observation=super::strategy::observe(&outcome);
                            (RungOutcome::Original(Box::new((outcome,Some(enclosing.clone())))),observation,observed)
                        },
                        PreparedRung::Derived(prepared)=>{
                            let mut execution=pse_backend_native::solve::Execution::within(flag.clone(),prepared.controls(),enclosing.clone()).map_err(Arc::new)?;
                            execution.work_admission=budget.admission().map(|owner|->Arc<dyn pse_backend_native::solve::WorkAdmission>{owner});execution.progress=progress.clone();
                            execution.memory=Some(service.policy.foreign_allowance(prepared.controls()));
                            let value=match service.derived_step(&prepared,execution.clone(),retained,budget,actual_input.as_deref().ok_or_else(||Arc::new(ProblemError::Internal("original source point absent".into())))?) {
                                Ok(value)=>value,
                                Err(error)=>{
                                    retained.clear();
                                    return Err(error.effect(&prepared,&enclosing));
                                }
                            };
                            if let Some(failed)=prepared.local_attempt_failure(&value,&enclosing) {retained.clear();return Err(failed);}
                            if original_completion && value.proposal.is_some() {
                                if let Some(proposal)=&value.proposal && let Some(accuracy)=proposal.reconstruction_accuracy {
                                    let mut source=prepared.original().semantic_point_key(&proposal.coordinates).map_err(Arc::new)?;
                                    source.normalization=Some(accuracy.normalization);source.derivation=Some(proposal.family);source.accuracy=Some(accuracy.product);
                                    let evidence=pse_model::strategy::ProductEvidence {source,derivative_order:0,branch:declaration.branch,accuracy};
                                    super::strategy::products(&product_state).and_then(|mut state|state.publish(evidence.clone())).map_err(Arc::new)?;
                                    actual_products.borrow_mut()[index].accuracy=Some(accuracy);actual_products.borrow_mut()[index].evidence=Some(evidence);
                                }
                                let outcome=service.derived_original_outcome(&prepared,&value,&execution,budget).map_err(|error|Arc::new(error.into_problem()))?;
                                let observed=super::strategy::work(&outcome);
                                let observation=super::strategy::observe(&outcome);
                                (RungOutcome::Original(Box::new((outcome,Some(enclosing.clone())))),observation,observed)
                            } else {
                                let observed=value.work();
                                let observation=super::strategy::observe(&value.outcome);
                                (RungOutcome::Derived(Box::new(value)),observation,observed)
                            }
                        },
                        PreparedRung::Multistart(prepared)=>{
                            let mut screening_work=0;
                            let super::solves::multistart::BoundMultistart {step,_bindings,screening_evaluations}=service.screen_multistart_worker_observed(&prepared,enclosing.clone(),budget,&mut screening_work).map_err(|error|super::strategy::EffectFailure::observed(Arc::new(error.into_problem()),pse_model::strategy::WorkObservation {attempts:1,evaluations:Some(screening_work),iterations:None,factorizations:None,proof_steps:None}))?;
                            let value=service.execute(step,None,attempt,retained,flag,&progress,budget,&owner,&enclosing)
                                .unwrap_or_else(|error|(super::solves::Outcome::Rejected(Arc::new(error)),Some(enclosing.clone())));
                            let observation=super::strategy::observe(&value.0);
                            let mut observed=super::strategy::work(&value.0);
                            observed.evaluations=observed.evaluations.and_then(|n|n.checked_add(screening_evaluations));
                            (RungOutcome::Original(Box::new(value)),observation,observed)
                        },
                        PreparedRung::Surrogate(prepared)=>{
                            let mut execution=pse_backend_native::solve::Execution::within(flag.clone(),&pse_backend_native::solve::Controls::default(),enclosing.clone()).map_err(Arc::new)?;
                            let phase_budget=WorkerBudget::drawing(prepared.worker_bytes().max(service.policy.worker_bytes),&service.pool);
                            execution.work_admission=budget.admission().map(|owner|->Arc<dyn pse_backend_native::solve::WorkAdmission>{owner});
                            let phase_budget=phase_budget.with_admission(admission.clone());
                            let value=prepared.advance(&execution,&phase_budget,owner.clone());
                            let observed=value.work;
                            let observation=value.terminal.as_deref().map_or(pse_model::generated::enums::NumericalAttemptObservation::Auxiliary,super::strategy::failure);
                            (RungOutcome::Surrogate(Box::new(value)),observation,observed)
                        },
                        PreparedRung::Path{prepared,start}=>{
                            let mut execution=pse_backend_native::solve::Execution::within(flag.clone(),&prepared.profile().controls,enclosing.clone()).map_err(Arc::new)?;
                            execution.work_admission=budget.admission().map(|owner|->Arc<dyn pse_backend_native::solve::WorkAdmission>{owner});execution.progress=progress.clone();execution.memory=Some(service.policy.foreign_allowance(&prepared.profile().controls));
                            let path_budget=WorkerBudget::drawing(prepared.worker_bytes().max(service.policy.worker_bytes),&service.pool);
                            let path_budget=path_budget.with_admission(admission.clone());
                            let mut value=super::solves::paths::run_path(&service,&prepared,*start,execution,&path_budget,owner.clone());
                            let mut observed=value.work();
                            if let Some(proposal)=value.proposal.take() {
                                let mut screening_evaluations=0;
                                match service.screen_start_worker_observed(prepared.original_target(),proposal.clone(),proposal.branch(),enclosing.clone(),budget,&mut screening_evaluations) {
                                    Ok(screened)=>{value.proposal=Some(screened.proposal().clone());},
                                    Err(error)=>value.terminal=Some(Arc::new(error.into_problem())),
                                }
                                observed.evaluations=observed.evaluations.and_then(|n|n.checked_add(screening_evaluations));
                            }
                            let observation=value.terminal.as_deref().map_or(pse_model::generated::enums::NumericalAttemptObservation::Auxiliary,super::strategy::failure);
                            (RungOutcome::Path(Box::new(value)),observation,observed)
                        },
                        #[cfg(feature="solver-petsc")]
                        PreparedRung::Petsc(prepared)=>{
                            let mut execution=pse_backend_native::solve::Execution::within(flag.clone(),prepared.controls(),enclosing.clone()).map_err(Arc::new)?;
                            execution.work_admission=budget.admission().map(|owner|->Arc<dyn pse_backend_native::solve::WorkAdmission>{owner});execution.progress=progress.clone();execution.memory=Some(service.policy.foreign_allowance(prepared.controls()));
                            let value=service.execute_petsc(&prepared,execution,budget,actual_input.as_deref().ok_or_else(||Arc::new(ProblemError::internal("original PETSc start missing")))?).map_err(|error|Arc::new(error.into_problem()))?;
                            let observed=value.work();
                            let observation=super::strategy::observe(&value.outcome);
                            (RungOutcome::Derived(Box::new(value)),observation,observed)
                        },
                    };
                    let mut charge=pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
                    charge.hash(&original).u64(attempt as u64).u64(index as u64);
                    let phase=if mechanism.position==pse_model::strategy::Position::Preparation {pse_model::strategy::Phase::Preparation} else {pse_model::strategy::Phase::Native};
                    Ok(super::strategy::Attempt { value, observation, work:pse_model::strategy::WorkCharge {phase, scope:pse_model::strategy::Scope::Task, charging_owner:charge.finish_hash(), observed } })
                },
                |value,observation| match value {
                    RungOutcome::Original(value)=>{
                        let (outcome,scope)=value.as_ref();
                        let mut assessment = assess(outcome,scope.as_ref(),budget);
                        for charge in &mut assessment.work {
                            let mut occurrence=pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
                            occurrence.hash(&charge.charging_owner).u64(attempt as u64);
                            charge.charging_owner=occurrence.finish_hash();
                        }
                        assessed = Some(assessment.product);
                        super::strategy::Assessment { auxiliary:false, retention:assessment.retention, original:assessment.original, work:assessment.work, observation, cause:super::strategy::cause(outcome) }
                    },
                    RungOutcome::Derived(value)=>{
                        let cause=value.screening_failure.clone().or_else(||super::strategy::cause(&value.outcome));
                        if let Some(proposal)=&value.proposal {
                            *point.borrow_mut()=Some(proposal.coordinates.clone());
                            fresh_start.set(Some(pse_model::strategy::StartOrigin::Auxiliary));
                            // This actual reconstruction accuracy belongs to the producing
                            // family; immutable preparation supplied no point certificate.
                            let product_index=actual_products.borrow().iter().position(|p|p.derived==Some(proposal.family));
                            if let Some(index)=product_index
                                && let Some(accuracy)=proposal.reconstruction_accuracy {
                                    let source=pse_model::strategy::SemanticProductKey { structure:original,binding:proposal.original,numerical_policy:None,normalization:Some(accuracy.normalization),point:Some(pse_backend_native::square_response::point_key(&proposal.coordinates)),parameters:None,derivation:Some(proposal.family),branch:None,accuracy:Some(accuracy.product) };
                                    let evidence=pse_model::strategy::ProductEvidence {source,derivative_order:0,branch:declaration.branch,accuracy};
                                    if let Err(error)=super::strategy::products(&product_state).and_then(|mut state|state.publish(evidence.clone())) {
                                        return super::strategy::Assessment {auxiliary:true,original:super::strategy::OriginalConclusion::Unavailable {cause:Arc::new(error)},work:Vec::new(),retention:StepRetention {candidate:crate::workflow::numerics::auxiliary_start(false),session:SessionDisposition::Discard},observation,cause:None};
                                    }
                                    actual_products.borrow_mut()[index].accuracy=Some(accuracy);
                                    actual_products.borrow_mut()[index].evidence=Some(evidence);
                            }
                        }
                        let decision=crate::workflow::numerics::auxiliary_start(value.proposal.is_some());
                        let observation=cause.as_deref().map_or_else(||if value.proposal.is_some() {pse_model::generated::enums::NumericalAttemptObservation::Auxiliary} else {observation},super::strategy::failure);
                        super::strategy::Assessment {auxiliary:true,original:super::strategy::OriginalConclusion::Unavailable {cause:Arc::new(ProblemError::Unsupported("auxiliary result requires original correction".into()))},work:Vec::new(),retention:StepRetention {candidate:decision,session:SessionDisposition::Discard},observation,cause}
                    },
                    RungOutcome::Surrogate(value)=>{
                        if let Some(statistical)=&value.statistical {
                            let product_index=actual_products.borrow().iter().position(|p|matches!(p.provider,Some(super::strategy::ProviderEvidence::Library(_))) && p.statistical.is_none());
                            if let Some(index)=product_index {actual_products.borrow_mut()[index].statistical=Some(statistical.clone());}
                        }
                        if let Some(proposal)=&value.proposal {
                            *point.borrow_mut()=Some(proposal.values().map(|(_,v)|v).collect());
                            fresh_start.set(Some(pse_model::strategy::StartOrigin::Surrogate));
                        }
                        let observation=if value.callback_terminal && super::strategy::permits_numerical_continuation(observation) {pse_model::generated::enums::NumericalAttemptObservation::ContractFailure} else {observation};
                        super::strategy::Assessment {auxiliary:true,original:super::strategy::OriginalConclusion::Unavailable {cause:Arc::new(ProblemError::Unsupported("auxiliary result requires original correction".into()))},work:Vec::new(),retention:StepRetention {candidate:crate::workflow::numerics::auxiliary_start(value.proposal.is_some() && !value.callback_terminal),session:SessionDisposition::Discard},observation,cause:value.terminal.clone()}
                    },
                    RungOutcome::Path(value)=>{
                        if let Some(event)=value.events.first() {
                            let product_index=actual_products.borrow().iter().position(|p|p.derived==Some(event.family));
                            if let Some(index)=product_index {actual_products.borrow_mut()[index].path_events=Some(value.events.clone());}
                        }
                        if let Some(proposal)=&value.proposal {
                            *completion_witness.borrow_mut()=value.completion_witness();
                            connected.set(completion_witness.borrow().is_some());
                            *point.borrow_mut()=Some(proposal.values().map(|(_,v)|v).collect());fresh_start.set(Some(pse_model::strategy::StartOrigin::Auxiliary));
                            let product_index=actual_products.borrow().iter().position(|p|p.derived==proposal.source().derivation);
                            if let Some(index)=product_index {actual_products.borrow_mut()[index].transport=value.connected.map(|p|p.transport);}
                        }
                        super::strategy::Assessment {auxiliary:true,original:super::strategy::OriginalConclusion::Unavailable {cause:Arc::new(ProblemError::Unsupported("auxiliary result requires original correction".into()))},work:Vec::new(),retention:StepRetention {candidate:crate::workflow::numerics::auxiliary_start(value.proposal.is_some()),session:SessionDisposition::Discard},observation,cause:value.terminal.clone()}
                    },
                },
            );
            let trace = super::strategy::Trace { owner:None, declaration, original, backend, profile, start, starts:actual_starts.into_inner(), products:actual_products.into_inner(), events:result.events };
            let bytes=trace.retained_bytes()?;
            let trace=Arc::new(trace.with_owner(service.reserve("math:strategy-trace",bytes)?));
            match (result.value, assessed, result.terminal) {
                (Some(RungOutcome::Original(value)), Some(product), None) => {
                    let (outcome,_)=*value;
                    if let Some(assessment) = result.assessment { assessment.retention.apply(retained); }
                    retained.charge_session(Box::new(_allowance.clone()));
                    Ok((outcome,product,trace))
                }
                (_, _, terminal) => {
                    retained.clear();
                    let error = terminal.map_or_else(|| ProblemError::Internal("strategy ended without an assessed original".into()), |error| ProblemError::Math(pse_math::MathError::Typed {retained:error.retained_bytes(),cause:pse_model::diagnostic::DiagnosticCause::from_shared(error)}));
                    let outcome = super::solves::Outcome::Rejected(Arc::new(error.into()));
                    let product = assess(&outcome,Some(&enclosing),budget).product;
                    Ok((outcome,product,trace))
                }
            }
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
        if members.iter().any(|member| {
            let request = member.step.composition_request();
            request.policy == pse_model::strategy::CompositionPolicy::Declared
                || request.limits.is_some()
                || request.branch.connected.is_some()
        }) {
            return Err(ProblemError::Unsupported("constrained numerical composition requires individually admitted steps before native dispatch".into()).into());
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
            let assessed=outcomes.into_iter().zip(originals).enumerate().map(|(index,(outcome,(declaration,original,backend,profile,start,attempt)))| {
                let mut value=Some(outcome.unwrap_or_else(|cause| (super::solves::Outcome::Rejected(Arc::new(cause)),Some(enclosing.clone()))));
                let mut product=None;
                let result=super::strategy::run_observed(&declaration,&enclosing,
                    |_|super::strategy::Facts {support:Default::default(),accuracy:Vec::new(),consumption:Vec::new(),reservation:None,work_admitted:false,start,inherited:false,connected:false,refusal:None},
                    |_,_| {
                        let value=value.take().ok_or_else(||Arc::new(ProblemError::Internal("batch outcome consumed twice".into())))?;
                        let mut charge=pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
                        charge.hash(&original).u64(attempt as u64).u64(index as u64);
                        Ok(super::strategy::Attempt {observation:super::strategy::observe(&value.0),work:pse_model::strategy::WorkCharge {phase:pse_model::strategy::Phase::Native,scope:pse_model::strategy::Scope::Task,charging_owner:charge.finish_hash(),observed:super::strategy::work(&value.0)},value})
                    },
                    |(outcome,scope),observation| {
                        let assessment=assess(index,outcome,scope.as_ref(),budget);
                        product=Some(assessment.product);
                        super::strategy::Assessment {auxiliary:false,retention:assessment.retention,original:assessment.original,work:assessment.work,observation,cause:super::strategy::cause(outcome)}
                    });
                let trace=super::strategy::Trace {owner:None,declaration,original,backend,profile,start,starts:vec![start],products:vec![Default::default()],events:result.events};
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
                    |_: &mut u64| Ok(true),
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
