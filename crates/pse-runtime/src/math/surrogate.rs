// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Task-admitted egobox actor. Non-Send library objects never leave their owning worker.
use super::{MathRuntimeError, MathService};
use pse_backend_native::ProblemError;
use pse_diagnostics::Cancellation;
mod library;
use library::{CallbackOwner, RetainedSurrogate};
/// Runtime-owned retained library surrogate contracts.
pub use library::{
    CorrelationSpec, FidelityCorrespondence, FidelityEvaluator, RegressionSpec, SurrogateAdvance,
    SurrogateFailure, SurrogateOptions, SurrogateProposal, SurrogateSnapshot,
    SurrogateStateSummary, SurrogateWork, ThetaTuning,
};
use std::{
    sync::{
        Arc, Mutex, Weak,
        mpsc::{self, Sender, SyncSender},
    },
    thread::JoinHandle,
    time::Instant,
};

type ProposalResult = Result<Option<SurrogateProposal>, MathRuntimeError>;
struct PhaseReply {
    proposal: ProposalResult,
    work: Result<SurrogateWork, MathRuntimeError>,
    callback_terminal: bool,
}
enum Command {
    Phase(pse_kernels::ExecutionScope, SyncSender<PhaseReply>),
    Advance(SyncSender<Result<SurrogateAdvance, MathRuntimeError>>),
    Proposal(SyncSender<ProposalResult>),
    Work(SyncSender<Result<SurrogateWork, MathRuntimeError>>),
    Snapshot(
        Arc<pse_columnar::AllocationLease>,
        pse_columnar::CancellationToken,
        SyncSender<Result<SurrogateSnapshot, MathRuntimeError>>,
    ),
    Stop,
}
/// The existing serialization mutex owns exclusion; notification owns no queue or permit.
#[derive(Default)]
struct ActorGate {
    mutex: Mutex<()>,
    released: pse_diagnostics::WakeSignal,
}
struct ActorGateGuard<'a> {
    guard: Option<std::sync::MutexGuard<'a, ()>>,
    released: &'a pse_diagnostics::WakeSignal,
}
impl Drop for ActorGateGuard<'_> {
    fn drop(&mut self) {
        self.guard.take(); // Publish actual unlock before waking blocked callers.
        self.released.notify();
    }
}
impl ActorGate {
    fn poison() -> MathRuntimeError {
        MathRuntimeError::Internal("surrogate actor gate poisoned".into())
    }
    fn lock(&self) -> Result<ActorGateGuard<'_>, MathRuntimeError> {
        Ok(ActorGateGuard {
            guard: Some(self.mutex.lock().map_err(|_| Self::poison())?),
            released: &self.released,
        })
    }
    fn lock_wait(
        &self,
        check: impl Fn() -> Result<(), MathRuntimeError>,
        stop: impl Future<Output = ()>,
        deadline: Option<Instant>,
    ) -> Result<ActorGateGuard<'_>, MathRuntimeError> {
        struct WakeThread(std::thread::Thread);
        impl std::task::Wake for WakeThread {
            fn wake(self: Arc<Self>) {
                self.0.unpark();
            }
            fn wake_by_ref(self: &Arc<Self>) {
                self.0.unpark();
            }
        }
        let waker = std::task::Waker::from(Arc::new(WakeThread(std::thread::current())));
        let mut context = std::task::Context::from_waker(&waker);
        let mut stop = std::pin::pin!(stop);
        loop {
            // Register both wake sources before the original predicates/lock attempt.
            let _release = self.released.watch(&waker);
            let _ = stop.as_mut().poll(&mut context);
            check()?;
            match self.mutex.try_lock() {
                Ok(guard) => {
                    return Ok(ActorGateGuard {
                        guard: Some(guard),
                        released: &self.released,
                    });
                }
                Err(std::sync::TryLockError::Poisoned(_)) => return Err(Self::poison()),
                Err(std::sync::TryLockError::WouldBlock) => {}
            }
            match deadline {
                Some(at) => std::thread::park_timeout(at.saturating_duration_since(Instant::now())),
                None => std::thread::park(),
            }
        }
    }
}
struct Actor {
    observation: Option<pse_diagnostics::observation::ObservationContext>,
    sender: Sender<Command>,
    join: Mutex<Option<JoinHandle<Result<(), MathRuntimeError>>>>,
    gate: ActorGate,
    abort: Cancellation,
    owner: Mutex<Option<Arc<pse_columnar::AllocationLease>>>,
    _shell_owner: Option<Arc<pse_columnar::AllocationLease>>,
}
impl std::fmt::Debug for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurrogateActor").finish_non_exhaustive()
    }
}
impl Drop for Actor {
    fn drop(&mut self) {
        let _ = self.retire();
    }
}
impl Actor {
    fn retire(&self) -> Result<(), MathRuntimeError> {
        let mut result = Ok(());
        let mut slot = match self.join.lock() {
            Ok(slot) => slot,
            Err(poison) => {
                result = Err(MathRuntimeError::Internal(
                    "surrogate actor join owner poisoned".into(),
                ));
                poison.into_inner()
            }
        };
        // Taking the existing physical join owner closes command admission before
        // waiting for an already admitted command to release its serialization gate.
        let Some(join) = slot.take() else {
            return result;
        };
        drop(slot);
        self.abort.cancel();
        let guard = match self.gate.mutex.lock() {
            Ok(guard) => guard,
            Err(poison) => {
                result = MathRuntimeError::settle(result, Err(ActorGate::poison()));
                poison.into_inner()
            }
        };
        let gate = ActorGateGuard {
            guard: Some(guard),
            released: &self.gate.released,
        };
        let stopped = self.sender.send(Command::Stop).map_err(|_| {
            MathRuntimeError::Infrastructure("surrogate actor stop submission lost".into())
        });
        result = MathRuntimeError::settle(result, stopped);
        // Admission is already closed. TLS and panic-payload destruction may
        // reenter a borrowed actor, so they must see Retiring without this gate.
        drop(gate);
        let owner = match self.owner.lock() {
            Ok(mut slot) => slot.take(),
            Err(poison) => {
                result = MathRuntimeError::settle(
                    result,
                    Err(MathRuntimeError::Internal(
                        "surrogate allocation owner poisoned".into(),
                    )),
                );
                poison.into_inner().take()
            }
        };
        let result = MathRuntimeError::settle(result, join_actor(join));
        // The performing retirement owns the native charge until the actual
        // worker, team, TLS and join-payload destruction have all settled.
        drop(owner);
        result
    }
    fn check_admission(&self) -> Result<(), MathRuntimeError> {
        let slot = self.join.lock().map_err(|_| {
            MathRuntimeError::Internal("surrogate actor join owner poisoned".into())
        })?;
        if slot.is_some() {
            Ok(())
        } else {
            Err(MathRuntimeError::Retiring)
        }
    }
}
fn join_actor(join: JoinHandle<Result<(), MathRuntimeError>>) -> Result<(), MathRuntimeError> {
    join.join()
        .map_err(|payload| super::joined_pool::worker_panic("surrogate actor", payload))?
}
impl Actor {
    fn snapshot(
        &self,
        owner: Arc<pse_columnar::AllocationLease>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<SurrogateSnapshot, MathRuntimeError> {
        let _gate = self.gate.lock_wait(
            || cancel.checkpoint().map_err(|_| MathRuntimeError::Cancelled),
            cancel.cancelled(),
            None,
        )?;
        self.check_admission()?;
        let (reply, answer) = mpsc::sync_channel(1);
        self.sender
            .send(Command::Snapshot(owner, cancel.clone(), reply))
            .map_err(|_| MathRuntimeError::Panic("surrogate owning worker exited".into()))?;
        answer.recv().map_err(|_| {
            MathRuntimeError::Panic("surrogate owning worker snapshot response lost".into())
        })?
    }
    fn advance(&self) -> Result<SurrogateAdvance, MathRuntimeError> {
        let _gate = self.gate.lock()?;
        self.check_admission()?;
        let (reply, answer) = mpsc::sync_channel(1);
        self.sender
            .send(Command::Advance(reply))
            .map_err(|_| MathRuntimeError::Panic("surrogate owning worker exited".into()))?;
        answer
            .recv()
            .map_err(|_| MathRuntimeError::Panic("surrogate owning worker response lost".into()))?
    }
    fn proposal(&self) -> ProposalResult {
        let _gate = self.gate.lock()?;
        self.check_admission()?;
        let (reply, answer) = mpsc::sync_channel(1);
        self.sender
            .send(Command::Proposal(reply))
            .map_err(|_| MathRuntimeError::Panic("surrogate owning worker exited".into()))?;
        answer
            .recv()
            .map_err(|_| MathRuntimeError::Panic("surrogate owning worker response lost".into()))?
    }
    fn phase(&self, scope: pse_kernels::ExecutionScope) -> Result<PhaseReply, MathRuntimeError> {
        let _gate = self.gate.lock_wait(
            || {
                scope
                    .check()
                    .map_err(ProblemError::Provider)
                    .map_err(Into::into)
            },
            scope.cancellation_control().cancelled(),
            scope.deadline(),
        )?;
        self.check_admission()?;
        let (reply, answer) = mpsc::sync_channel(1);
        if let (Some(caller), Some(producer)) =
            (pse_diagnostics::observation::current(), &self.observation)
        {
            caller.link(
                producer,
                pse_diagnostics::observation::Relationship::DependencyWait,
            );
        }
        self.sender
            .send(Command::Phase(scope, reply))
            .map_err(|_| MathRuntimeError::Panic("surrogate owning worker exited".into()))?;
        answer.recv().map_err(|_| {
            MathRuntimeError::Panic("surrogate owning worker phase response lost".into())
        })
    }
    fn work(&self) -> Result<SurrogateWork, MathRuntimeError> {
        let _gate = self.gate.lock()?;
        self.check_admission()?;
        let (reply, answer) = mpsc::sync_channel(1);
        self.sender
            .send(Command::Work(reply))
            .map_err(|_| MathRuntimeError::Panic("surrogate owning worker exited".into()))?;
        answer
            .recv()
            .map_err(|_| MathRuntimeError::Panic("surrogate owning worker response lost".into()))?
    }
}
/// Unique retirement authority for one actual non-Send solver/state and its worker/team.
/// Await every enclosing advance/staged solve before consuming `finish`; cloned
/// phases only borrow actor access and retain independently accounted metadata.
#[derive(Debug)]
pub struct SurrogateHandle {
    actor: Arc<Actor>,
    control: Cancellation,
    metadata: Arc<SurrogateTaskMetadata>,
}
/// Compact immutable binding data survives idle phase aliases independently of
/// the native workspace. The weak actor conveys no retirement authority.
#[derive(Debug)]
struct SurrogateTaskMetadata {
    service: Arc<MathService>,
    actor: Weak<Actor>,
    scope: pse_kernels::ExecutionScope,
    task_scope: pse_kernels::ExecutionScope,
    correspondence: FidelityCorrespondence,
    cores: usize,
    workspace: usize,
    task: pse_ids::ContentHash,
    _owner: Arc<pse_columnar::AllocationLease>,
    _actor_shell_owner: Arc<pse_columnar::AllocationLease>,
}
impl Drop for SurrogateHandle {
    fn drop(&mut self) {
        // Abandonment containment only; successful users consume finish instead.
        self.cancel();
        let _ = self.actor.retire();
    }
}
impl SurrogateHandle {
    /// Close command admission, interrupt this handle's work and join its actual
    /// actor, library team and thread-local destruction under the retained charge.
    /// # Errors
    /// Original stop, solver destruction and physical worker settlement failures.
    pub fn finish(self) -> Result<(), MathRuntimeError> {
        self.cancel();
        self.actor.retire()
    }
    /// Passive actor observations; reading does not queue behind its scientific work.
    pub fn observations(&self) -> Option<pse_diagnostics::observation::OperationReader> {
        self.actor
            .observation
            .as_ref()
            .map(|context| context.reader())
    }

    /// Explicitly copy the actual library history under an independent conservative charge.
    /// Compact advancement never requests or retains this history.
    /// # Errors
    /// Cancellation, original task deadline, allocation refusal or owning-worker failure.
    pub fn snapshot(
        &self,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<SurrogateSnapshot, MathRuntimeError> {
        cancel
            .checkpoint()
            .map_err(|_| MathRuntimeError::Cancelled)?;
        self.metadata.scope.check().map_err(scope_error)?;
        let owner = self
            .metadata
            .service
            .reserve("math:surrogate-snapshot", self.metadata.workspace)?;
        self.actor.snapshot(owner, cancel)
    }
    /// Complete actual source/correspondence/options task identity.
    pub fn key(&self) -> pse_ids::ContentHash {
        self.metadata.task
    }
    /// Latest model proposal, never original result permission.
    pub fn proposal(&self) -> ProposalResult {
        if self
            .control
            .flag()
            .load(std::sync::atomic::Ordering::Acquire)
        {
            return Err(MathRuntimeError::Cancelled);
        }
        self.metadata.scope.check().map_err(scope_error)?;
        self.actor
            .proposal()?
            .map(|proposal| {
                let owner = self
                    .metadata
                    .service
                    .reserve("math:surrogate-proposal", proposal.retained_bytes()?)?;
                Ok(proposal.with_owner(owner))
            })
            .transpose()
    }
    /// Inclusive actual callback/iteration work, including partial failed attempts.
    pub fn work(&self) -> Result<SurrogateWork, MathRuntimeError> {
        self.actor.work()
    }
    /// Accounted retained solver/state, persistent worker stack and pool reservation.
    pub fn retained_bytes(&self) -> usize {
        self.metadata.workspace
    }
    /// Request cooperative interruption; active admitted work joins before releasing CPU.
    pub fn cancel(&self) {
        self.metadata.scope.cancellation_control().cancel();
        self.control.cancel();
    }
}
fn failure(error: SurrogateFailure<ProblemError>) -> MathRuntimeError {
    match error {
        SurrogateFailure::Callback(e) => e.into(),
        SurrogateFailure::Math(e) => e.into(),
        SurrogateFailure::Panic(message) => MathRuntimeError::Panic(message),
        SurrogateFailure::Runtime(error) => error,
        SurrogateFailure::Settlement { operation, cleanup } => MathRuntimeError::Settlement {
            operation: Box::new(failure(*operation)),
            cleanup,
        },
    }
}
fn scope_error(error: pse_kernels::ProviderError) -> MathRuntimeError {
    match error {
        pse_kernels::ProviderError::Cancelled => MathRuntimeError::Cancelled,
        other => MathRuntimeError::Math(pse_math::MathError::Scope(other)),
    }
}
fn actor<E: FidelityEvaluator<Error = ProblemError>>(
    mut callback: CallbackOwner<E>,
    options: SurrogateOptions,
    scope: pse_kernels::ExecutionScope,
    abort: Cancellation,
) -> Result<Actor, MathRuntimeError> {
    let constructed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        actor_inner(callback.clone(), options, scope, abort)
    }))
    .unwrap_or_else(|payload| {
        Err(super::joined_pool::worker_panic(
            "surrogate actor construction",
            payload,
        ))
    });
    MathRuntimeError::settle(constructed, callback.finish())
}
fn actor_inner<E: FidelityEvaluator<Error = ProblemError>>(
    callback: CallbackOwner<E>,
    options: SurrogateOptions,
    scope: pse_kernels::ExecutionScope,
    abort: Cancellation,
) -> Result<Actor, MathRuntimeError> {
    let (sender, commands) = mpsc::channel();
    let (ready, initialized) = mpsc::sync_channel(1);
    let flag = abort.flag();
    let stack = options.stack_bytes;
    let diagnostic = pse_diagnostics::observation::current().map(|context| {
        context.child(
            "surrogate.actor",
            pse_diagnostics::observation::Relationship::Nested,
        )
    });
    let observation = diagnostic.as_ref().map(|guard| guard.context());
    let worker_observation = observation.clone();
    let join = std::thread::Builder::new()
        .name("pse-egobox".into())
        .stack_size(stack)
        .spawn(move || {
            let _entered = worker_observation
                .as_ref()
                .map(pse_diagnostics::observation::ObservationContext::enter);
            // Construct here: egobox's boxed strategies are Sync but not Send.
            let initialized_task = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut task =
                    RetainedSurrogate::new_with_owner(callback, options, scope).map_err(failure)?;
                if let Err(error) = task.bind_abort(flag) {
                    return MathRuntimeError::settle(Err(error.into()), task.finish());
                }
                Ok::<_, MathRuntimeError>(task)
            }));
            let mut task = match initialized_task {
                Ok(Ok(task)) => task,
                Ok(Err(error)) => {
                    let _ = ready.send(Err(error));
                    return Ok(());
                }
                Err(payload) => {
                    let _ = ready.send(Err(super::joined_pool::worker_panic(
                        "egobox construction",
                        payload,
                    )));
                    return Ok(());
                }
            };
            if ready.send(Ok(())).is_err() {
                return task.finish();
            }
            while let Ok(command) = commands.recv() {
                match command {
                    Command::Phase(scope, reply) => {
                        let _phase =
                            pse_diagnostics::observation::phase("surrogate.fidelity.advance");
                        let before = task.work();
                        let advance = task.advance_within(scope).map_err(failure);
                        let after = match &advance {
                            Ok(advance) => Ok(advance.work),
                            Err(_) => task.work(),
                        };
                        let proposal = advance.map(|advance| advance.proposal);
                        let work = before
                            .and_then(|before| {
                                after.and_then(|after| {
                                    Ok(SurrogateWork {
                                        evaluations: after
                                            .evaluations
                                            .checked_sub(before.evaluations)
                                            .ok_or_else(|| {
                                                pse_math::MathError::Contract(
                                                    "surrogate work counter regressed".into(),
                                                )
                                            })?,
                                        iterations: after
                                            .iterations
                                            .checked_sub(before.iterations)
                                            .ok_or_else(|| {
                                                pse_math::MathError::Contract(
                                                    "surrogate work counter regressed".into(),
                                                )
                                            })?,
                                        initializations: after
                                            .initializations
                                            .checked_sub(before.initializations)
                                            .ok_or_else(|| {
                                                pse_math::MathError::Contract(
                                                    "surrogate work counter regressed".into(),
                                                )
                                            })?,
                                    })
                                })
                            })
                            .map_err(Into::into);
                        if let Some(context) = &worker_observation {
                            context.boolean(
                                "surrogate.proposal.available",
                                proposal.as_ref().is_ok_and(Option::is_some),
                            );
                            context.boolean("surrogate.phase.failed", proposal.is_err());
                            if let Ok(work) = &work {
                                context.count("surrogate.evaluations", "calls", work.evaluations);
                                context.count(
                                    "surrogate.iterations",
                                    "iterations",
                                    work.iterations,
                                );
                                context.count(
                                    "surrogate.initializations",
                                    "constructions",
                                    work.initializations,
                                );
                            }
                        }
                        let callback_terminal = proposal.is_err() && task.is_finished();
                        let _ = reply.send(PhaseReply {
                            proposal,
                            work,
                            callback_terminal,
                        });
                    }
                    Command::Advance(reply) => {
                        let _ = reply.send(task.advance().map_err(failure));
                    }
                    Command::Proposal(reply) => {
                        let _ = reply.send(Ok(task.proposal()));
                    }
                    Command::Work(reply) => {
                        let _ = reply.send(task.work().map_err(Into::into));
                    }
                    Command::Snapshot(owner, cancel, reply) => {
                        let _ = reply.send(task.snapshot(owner, &cancel).map_err(failure));
                    }
                    Command::Stop => break,
                }
            }
            let result = task.finish();
            if let Some(diagnostic) = diagnostic {
                diagnostic.finish(if result.is_ok() {
                    pse_diagnostics::observation::Outcome::Completed
                } else {
                    pse_diagnostics::observation::Outcome::Failed
                });
            }
            result
        })
        .map_err(|e| MathRuntimeError::Infrastructure(e.to_string()))?;
    match initialized.recv() {
        Ok(Ok(())) => Ok(Actor {
            observation,
            sender,
            join: Mutex::new(Some(join)),
            gate: ActorGate::default(),
            abort,
            owner: Mutex::new(None),
            _shell_owner: None,
        }),
        Ok(Err(error)) => MathRuntimeError::settle(Err(error), join_actor(join)),
        Err(_) => MathRuntimeError::settle(
            Err(MathRuntimeError::Panic(
                "egobox construction response lost".into(),
            )),
            join_actor(join),
        ),
    }
}
impl MathService {
    /// Construct full library state inside its permanent worker under original task admission.
    /// The finite local clock begins at submission, before waiting for CPU or a job slot.
    pub fn prepare_surrogate<'a, E: FidelityEvaluator<Error = ProblemError>>(
        self: &'a Arc<Self>,
        evaluator: E,
        options: SurrogateOptions,
        scope: pse_kernels::ExecutionScope,
        driver: &'a crate::CancelSource,
    ) -> impl Future<Output = Result<SurrogateHandle, MathRuntimeError>> + Send + 'a {
        let mut callback = CallbackOwner::new(evaluator, scope.clone(), options.evaluations);
        // Establish custody eagerly so even an unpolled abandoned future cannot
        // destroy the evaluator while another failure is already unwinding.
        async move {
            let prepared = futures_util::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {
        if options.stack_bytes != self.policy.stack_bytes
            || options.foreign_bytes > self.policy.foreign_bytes
        {
            return Err(MathRuntimeError::Limit(
                "surrogate stack/foreign allocation policy",
            ));
        }
        let (correspondence, workspace, task, metadata_owner) = callback.with_correspondence(|correspondence| {
            let workspace = options.workspace_bytes(correspondence)?;
            if workspace > self.policy.workspace_bytes {
                return Err(MathRuntimeError::Limit("surrogate retained workspace"));
            }
            let task = options.key(correspondence)?;
            let metadata_bytes = correspondence.coordinates.capacity()
                .checked_add(correspondence.outputs.capacity())
                .and_then(|n| n.checked_mul(size_of::<pse_ids::SemanticId>()))
                .and_then(|n| n.checked_add(size_of::<SurrogateTaskMetadata>() + 2 * size_of::<usize>()))
                .ok_or_else(|| ProblemError::memory("surrogate task metadata extent"))?;
            let owner = self.reserve("math:surrogate-task-metadata", metadata_bytes)?;
            // Admission precedes the vector copy, from this same locked source.
            Ok((correspondence.clone(), workspace, task, owner))
        })?;
        let task_scope = scope.clone();
        let shell_owner = self.reserve(
            "math:surrogate-actor-shell",
            size_of::<Actor>() + 2 * size_of::<usize>(),
        )?;
        scope.check().map_err(scope_error)?;
        let local = Instant::now()
            .checked_add(options.time)
            .ok_or(MathRuntimeError::Limit("surrogate submission deadline"))?;
        let deadline = scope.deadline().map_or(local, |outer| outer.min(local));
        let scope =
            pse_kernels::ExecutionScope::new(scope.cancellation_control().clone(), Some(deadline));
        let cores = options.threads;
        let control = Cancellation::default();
        let worker_scope = scope.clone();
        let worker_control = control.clone();
        let worker_callback = callback.clone();
        let actor_shell_owner = shell_owner.clone();
        let operation = self.job_retained_scoped_with_finish(
            cores,
            workspace,
            control.clone(),
            scope.deadline(),
            move |flag| {
                worker_callback.retain_construction()?;
                if flag.load(std::sync::atomic::Ordering::Acquire) {
                    return Err(MathRuntimeError::Cancelled);
                }
                let mut actor = actor(worker_callback, options, worker_scope, worker_control)?;
                actor._shell_owner = Some(actor_shell_owner);
                Ok((Arc::new(actor), workspace))
            },
            |actor: Arc<Actor>| actor.retire(),
        );
        tokio::pin!(operation);
        let (actor, owner) = tokio::select! {
            result=&mut operation=>result?,
            ()=driver.cancelled()=>{
                scope.cancellation_control().cancel();control.cancel();
                let cleanup = match operation.await {
                    Ok((actor, _owner)) => actor.retire(),
                    Err(MathRuntimeError::Cancelled) => Ok(()),
                    Err(error) => Err(error),
                };
                return MathRuntimeError::settle(Err(MathRuntimeError::Cancelled), cleanup);
            }
        };
        // Transfer the native lease to the actor; retirement takes it into the
        // performing join, independently of idle metadata and actor aliases.
        let attached = match actor.owner.lock() {
            Ok(mut slot) => { *slot = Some(owner); Ok(()) }
            Err(poison) => {
                *poison.into_inner() = Some(owner);
                Err(MathRuntimeError::Internal("surrogate allocation owner poisoned".into()))
            }
        };
        if let Err(error) = attached {
            return MathRuntimeError::settle(Err(error), actor.retire());
        }
        if let Err(error) = callback.release_construction() {
            return MathRuntimeError::settle(Err(error), actor.retire());
        }
        let metadata = Arc::new(SurrogateTaskMetadata {
            service: self.clone(),
            actor: Arc::downgrade(&actor),
            scope,
            task_scope,
            correspondence,
            cores,
            workspace,
            task,
            _owner: metadata_owner,
            _actor_shell_owner: shell_owner,
        });
        Ok(SurrogateHandle { actor, control, metadata })
        })).await.unwrap_or_else(|payload| Err(super::joined_pool::worker_panic("surrogate preparation", payload)));
            MathRuntimeError::settle(prepared, callback.finish())
        }
    }
    /// Advance on the same owning thread under the same task deadline and admitted CPU team.
    /// Consumers independently screen/correct proposals against the original scientific problem.
    pub async fn advance_surrogate(
        self: &Arc<Self>,
        handle: &SurrogateHandle,
        driver: &crate::CancelSource,
    ) -> Result<SurrogateAdvance, MathRuntimeError> {
        let actor = handle.actor.clone();
        let control = handle.control.clone();
        let operation = self.job_retained_scoped(
            handle.metadata.cores,
            handle.metadata.workspace,
            control.clone(),
            handle.metadata.scope.deadline(),
            move |_| {
                let advance = actor.advance()?;
                let bytes = advance
                    .proposal
                    .as_ref()
                    .map_or(Ok(0), SurrogateProposal::retained_bytes)?;
                Ok((advance, bytes))
            },
        );
        tokio::pin!(operation);
        tokio::select! {
            result=&mut operation=>result.map(|(mut advance,owner)|{
                advance.proposal=advance.proposal.map(|proposal|proposal.with_owner(owner));advance
            }),
            ()=driver.cancelled()=>{
                handle.metadata.scope.cancellation_control().cancel();control.cancel();
                let settled = match operation.await {
                    Ok(_) | Err(MathRuntimeError::Cancelled) => Ok(()),
                    Err(error) => Err(error),
                };
                MathRuntimeError::settle(Err(MathRuntimeError::Cancelled), settled)
            }
        }
    }
}

/// Cloneable binding to an original target and a borrowed retained actor.
/// Its creator keeps the unique `SurrogateHandle` until enclosing work departs,
/// then consumes its fallible finish; idle bindings cannot retain native custody.
#[derive(Clone, Debug)]
pub struct PreparedSurrogate(Arc<PreparedSurrogateData>);
#[derive(Debug)]
struct PreparedSurrogateData {
    task: Arc<SurrogateTaskMetadata>,
    target: super::solves::PreparedSolve,
    scope: pse_kernels::ExecutionScope,
    operation: pse_kernels::ExecutionScope,
    key: pse_ids::ContentHash,
    result_bytes: usize,
    worker_bytes: usize,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl PreparedSurrogate {
    /// Original target whose completion independently assesses every physical proposal.
    pub fn original_target(&self) -> &super::solves::PreparedSolve {
        &self.0.target
    }
    /// Original caller scope, distinct from this operation's narrower deadline.
    pub fn scope(&self) -> &pse_kernels::ExecutionScope {
        &self.0.scope
    }
    /// Exact finite CPU team retained by the active actor's local rayon pool.
    pub fn threads(&self) -> usize {
        self.0.task.cores
    }
    /// Actual consumed task/options/fidelity/source and original binding identity.
    pub fn key(&self) -> pse_ids::ContentHash {
        self.0.key
    }
    /// Actual source-owned support contracts; none denotes deterministic numerical accuracy.
    pub fn support(&self) -> std::collections::BTreeSet<pse_ids::ContentHash> {
        std::collections::BTreeSet::from([
            self.key(),
            self.0.task.task,
            self.0.task.correspondence.source,
            self.0.task.correspondence.fidelity,
        ])
    }
    /// Library surrogate phases have no native Backend profile.
    pub fn profile_ref(&self) -> Option<pse_model::strategy::ProfileRef> {
        None
    }
    /// Complete finite copied sample/proposal/observation extent.
    pub(crate) fn result_bytes(&self) -> usize {
        self.0.result_bytes
    }
    /// Original screening callbacks use this actual finite worker extent.
    pub(crate) fn worker_bytes(&self) -> usize {
        self.0.worker_bytes
    }
    /// Advance on the actor's same owner while the shared native worker holds CPU admission.
    /// No asynchronous admission, actor reconstruction or additional task scope occurs here.
    pub(crate) fn advance(
        &self,
        execution: &pse_backend_native::solve::Execution,
        budget: &Arc<super::WorkerBudget>,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> SurrogatePhaseOutcome {
        let mut outcome = SurrogatePhaseOutcome {
            statistical: None,
            proposal: None,
            screened: None,
            work: pse_model::strategy::WorkObservation {
                attempts: 0,
                evaluations: None,
                iterations: None,
                factorizations: None,
                proof_steps: None,
            },
            terminal: None,
            callback_terminal: false,
            _owner: owner.clone(),
        };
        let result = (|| -> Result<(), MathRuntimeError> {
            execution.check()?;
            if !Arc::ptr_eq(&execution.cancel, self.scope().cancellation())
                || execution
                    .scope()?
                    .deadline()
                    .is_none_or(|d| self.scope().deadline().is_some_and(|outer| d > outer))
            {
                return Err(
                    ProblemError::Contract("surrogate active task scope mismatch".into()).into(),
                );
            }
            self.0.operation.check().map_err(ProblemError::Provider)?;
            let active = execution.scope()?;
            let deadline = active
                .deadline()
                .into_iter()
                .chain(self.0.operation.deadline())
                .min();
            let scope =
                pse_kernels::ExecutionScope::new(active.cancellation_control().clone(), deadline);
            let reply = self
                .0
                .task
                .actor
                .upgrade()
                .ok_or(MathRuntimeError::Retiring)?
                .phase(scope.clone())?;
            outcome.callback_terminal = reply.callback_terminal;
            match reply.work {
                Ok(work) => {
                    outcome.work = work.observation();
                    outcome.work.attempts = work
                        .initializations
                        .checked_add(work.iterations)
                        .ok_or_else(|| ProblemError::memory("surrogate operation count extent"))?;
                }
                Err(error) => {
                    outcome.terminal = Some(Arc::new(error.into_problem()));
                }
            }
            let statistical = match reply.proposal {
                Ok(value) => value,
                Err(error) => {
                    outcome.terminal = Some(Arc::new(error.into_problem()));
                    return Ok(());
                }
            };
            if outcome.terminal.is_some() {
                return Ok(());
            }
            scope.check().map_err(ProblemError::Provider)?;
            if let Some(statistical) = statistical {
                if statistical.retained_bytes()? > self.result_bytes() {
                    return Err(ProblemError::memory(
                        "surrogate copied sample extent exceeds admission",
                    )
                    .into());
                }
                let source = self
                    .0
                    .target
                    .surrogate_start(
                        &statistical,
                        &self.0.task.correspondence,
                        self.0.task.task,
                        pse_model::strategy::BranchPolicy::any_qualified(),
                    )?
                    .with_owner(owner.clone());
                outcome.statistical = Some(Arc::new(statistical.with_owner(owner.clone())));
                let mut evaluations = 0;
                match self.0.task.service.screen_start_worker_observed(
                    &self.0.target,
                    source,
                    pse_model::strategy::BranchPolicy::any_qualified(),
                    active.clone(),
                    budget,
                    &mut evaluations,
                ) {
                    Ok(screened) => {
                        outcome.proposal =
                            Some(screened.proposal().clone().with_owner(owner.clone()));
                        outcome.screened = Some(screened.with_owner(owner));
                    }
                    Err(error) => outcome.terminal = Some(Arc::new(error.into_problem())),
                }
                outcome.work.evaluations = outcome
                    .work
                    .evaluations
                    .and_then(|n| n.checked_add(evaluations));
            }
            scope.check().map_err(ProblemError::Provider)?;
            execution.check()?;
            Ok(())
        })();
        if let Err(error) = result {
            outcome.terminal = Some(Arc::new(error.into_problem()));
            outcome.proposal = None;
            outcome.screened = None;
        }
        outcome
    }
}
/// Actual statistical observations and physical start from one admitted actor command.
#[derive(Debug)]
pub struct SurrogatePhaseOutcome {
    /// Library-selected sample and uncertainty/infill statistic; no deterministic accuracy.
    pub statistical: Option<Arc<SurrogateProposal>>,
    /// Independently original-screened physical start, never original result permission.
    pub proposal: Option<super::prediction::Proposal>,
    pub(crate) screened: Option<super::prediction::Screened>,
    /// Atomic actual work delta including partial failed model evaluations.
    pub work: pse_model::strategy::WorkObservation,
    /// Original retained typed cause, including callback panic/deadline/contract stops.
    pub terminal: Option<Arc<ProblemError>>,
    /// Actual actor failure disables the retained task; its cause alone never permits a trial retry.
    pub callback_terminal: bool,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl MathService {
    /// Bind a genuine retained surrogate task to its original compiled source and caller scope.
    /// Metadata preparation is synchronous and performs no model callbacks or native iteration.
    /// Keep `handle` until all enclosing phase uses finish, then consume its `finish`.
    /// # Errors
    /// Changed actor service/source/target/inventory, finite extent or scope incompatibility.
    pub fn prepare_surrogate_phase(
        self: &Arc<Self>,
        handle: &SurrogateHandle,
        target: super::solves::PreparedSolve,
        scope: pse_kernels::ExecutionScope,
        time: std::time::Duration,
    ) -> Result<PreparedSurrogate, MathRuntimeError> {
        scope.check().map_err(ProblemError::Provider)?;
        handle.actor.check_admission()?;
        let task = &handle.metadata;
        task.scope.check().map_err(ProblemError::Provider)?;
        if time.is_zero()
            || scope.deadline().is_none()
            || !Arc::ptr_eq(self, &task.service)
            || !Arc::ptr_eq(scope.cancellation(), task.task_scope.cancellation())
            || task
                .task_scope
                .deadline()
                .is_some_and(|d| scope.deadline() != Some(d))
            || target.task_scope().as_ref().is_some_and(|original| {
                !Arc::ptr_eq(original.cancellation(), scope.cancellation())
                    || original.deadline() != scope.deadline()
            })
        {
            return Err(ProblemError::Contract(
                "surrogate phase requires its original finite task/service/scope".into(),
            )
            .into());
        }
        task.correspondence.validate()?;
        if target.original_identity()? != task.correspondence.original_target
            || target.original_coordinates()? != task.correspondence.coordinates
            || target.threads() != task.cores
        {
            return Err(ProblemError::Contract("surrogate correspondence original target/coordinates or admitted CPU team mismatch".into()).into());
        }
        let local = Instant::now()
            .checked_add(time)
            .ok_or_else(|| ProblemError::Limit {
                kind: pse_backend_native::LimitKind::Time,
                detail: "surrogate phase deadline extent".into(),
            })?;
        let deadline = scope
            .deadline()
            .into_iter()
            .chain(task.scope.deadline())
            .chain([local])
            .min();
        let operation =
            pse_kernels::ExecutionScope::new(scope.cancellation_control().clone(), deadline);
        let result_bytes = task
            .correspondence
            .coordinates
            .len()
            .checked_mul(size_of::<f64>() * 2 + size_of::<pse_ids::SemanticId>())
            .and_then(|n| {
                n.checked_add(
                    task.correspondence
                        .outputs
                        .len()
                        .checked_mul(size_of::<f64>())?,
                )
            })
            .and_then(|n| {
                n.checked_add(
                    size_of::<SurrogatePhaseOutcome>()
                        + size_of::<SurrogateProposal>()
                        + size_of::<super::prediction::Proposal>()
                        + 4096,
                )
            })
            .ok_or_else(|| ProblemError::memory("surrogate phase result extent"))?;
        let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("actual-retained-surrogate-original-phase")
            .hash(&task.task)
            .hash(&target.original_identity()?)
            .hash(&target.preparation_identity()?)
            .u64(time.as_secs())
            .u64(u64::from(time.subsec_nanos()));
        let key = h.finish_hash();
        let owner = self.reserve(
            "math:surrogate-phase-metadata",
            size_of::<PreparedSurrogateData>() + 2 * size_of::<usize>(),
        )?;
        Ok(PreparedSurrogate(Arc::new(PreparedSurrogateData {
            task: task.clone(),
            target,
            scope,
            operation,
            key,
            result_bytes,
            worker_bytes: self.policy.worker_bytes,
            _owner: owner,
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::surrogate::FidelityCorrespondence;
    use pse_ids::{ContentHash, SemanticId};
    use std::sync::atomic::{AtomicBool, Ordering};
    #[derive(Debug)]
    struct Evaluator {
        correspondence: FidelityCorrespondence,
        threads: Arc<Mutex<Vec<std::thread::ThreadId>>>,
        destroyed: Arc<AtomicBool>,
    }
    impl Drop for Evaluator {
        fn drop(&mut self) {
            self.destroyed.store(true, Ordering::Release);
        }
    }
    impl FidelityEvaluator for Evaluator {
        type Error = ProblemError;
        fn correspondence(&self) -> &FidelityCorrespondence {
            &self.correspondence
        }
        fn evaluate(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            self.threads
                .lock()
                .unwrap()
                .push(std::thread::current().id());
            out[0] = (x[0] - 0.37).powi(2);
            Ok(())
        }
    }
    fn fixture() -> (
        Evaluator,
        SurrogateOptions,
        Arc<Mutex<Vec<std::thread::ThreadId>>>,
        Arc<AtomicBool>,
    ) {
        let threads = Arc::new(Mutex::new(Vec::new()));
        let destroyed = Arc::new(AtomicBool::new(false));
        let c = FidelityCorrespondence {
            original_target: ContentHash::from_bytes([1; 32]),
            model_target: ContentHash::from_bytes([1; 32]),
            source: ContentHash::from_bytes([2; 32]),
            fidelity: ContentHash::from_bytes([3; 32]),
            coordinates: vec![SemanticId::from_bytes([4; 16])],
            outputs: vec![SemanticId::from_bytes([5; 16])],
        };
        let o = SurrogateOptions {
            bounds: vec![(0.0, 1.0)],
            initial: vec![vec![0.0], vec![0.5], vec![1.0]],
            iterations: 2,
            evaluations: 20,
            infill_starts: 1,
            regression: RegressionSpec::CONSTANT,
            correlation: CorrelationSpec::MATERN52,
            theta: ThetaTuning::Fixed(vec![0.1].into()),
            gp_starts: 0,
            gp_evaluations: 5,
            seed: 74,
            phase_steps: (1, 1),
            radius: 0.2,
            contraction: 0.5,
            threads: 1,
            stack_bytes: 4 * 1024 * 1024,
            foreign_bytes: 1024 * 1024,
            time: std::time::Duration::from_secs(20),
        };
        (
            Evaluator {
                correspondence: c,
                threads: threads.clone(),
                destroyed: destroyed.clone(),
            },
            o,
            threads,
            destroyed,
        )
    }
    #[derive(Debug)]
    struct DestructivePreparation {
        evaluator: Evaluator,
        panic_correspondence: bool,
    }
    impl Drop for DestructivePreparation {
        fn drop(&mut self) {
            panic!("public evaluator destructor failed");
        }
    }
    impl FidelityEvaluator for DestructivePreparation {
        type Error = ProblemError;
        fn correspondence(&self) -> &FidelityCorrespondence {
            assert!(!self.panic_correspondence, "public correspondence panic");
            self.evaluator.correspondence()
        }
        fn evaluate(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), Self::Error> {
            self.evaluator.evaluate(x, out)
        }
    }
    #[tokio::test]
    async fn public_pre_admission_failure_survives_evaluator_destructor_panic() {
        for panic_correspondence in [false, true] {
            let (evaluator, mut options, threads, destroyed) = fixture();
            let (service, _cache) = super::super::tests::service_with_policy(
                128 << 20,
                super::super::MathPolicy {
                    stack_bytes: options.stack_bytes,
                    foreign_bytes: options.foreign_bytes,
                    workspace_bytes: 16 << 20,
                    worker_bytes: 8 << 20,
                    inner_session_bytes: 0,
                    ..Default::default()
                },
            );
            let baseline = service.pool.reserved();
            if !panic_correspondence {
                options.stack_bytes += 1;
            }
            let result = service
                .prepare_surrogate(
                    DestructivePreparation {
                        evaluator,
                        panic_correspondence,
                    },
                    options,
                    pse_kernels::ExecutionScope::new(Cancellation::default(), None),
                    &crate::CancelSource::new(),
                )
                .await;
            assert!(
                matches!(result, Err(MathRuntimeError::Settlement { operation, cleanup })
                if (matches!(operation.as_ref(), MathRuntimeError::Limit("surrogate stack/foreign allocation policy"))
                    || matches!(operation.as_ref(), MathRuntimeError::Panic(message) if message.contains("public correspondence panic")))
                && matches!(cleanup.as_ref(), MathRuntimeError::Panic(message) if message.contains("public evaluator destructor failed")))
            );
            assert!(threads.lock().unwrap().is_empty());
            assert!(destroyed.load(Ordering::Acquire));
            assert_eq!(service.pool.reserved(), baseline);
        }
    }
    #[tokio::test]
    async fn metadata_reservation_refusal_preserves_evaluator_retirement_failure() {
        let (evaluator, options, threads, destroyed) = fixture();
        let capacity = 128 << 20;
        let (service, _cache) = super::super::tests::service_with_policy(
            capacity,
            super::super::MathPolicy {
                stack_bytes: options.stack_bytes,
                foreign_bytes: options.foreign_bytes,
                workspace_bytes: 16 << 20,
                ..Default::default()
            },
        );
        let baseline = service.pool.reserved();
        let occupied = service
            .reserve("test:metadata-refusal", capacity - baseline)
            .unwrap();
        let result = service
            .prepare_surrogate(
                DestructivePreparation {
                    evaluator,
                    panic_correspondence: false,
                },
                options,
                pse_kernels::ExecutionScope::new(Cancellation::default(), None),
                &crate::CancelSource::new(),
            )
            .await;
        assert!(
            matches!(result, Err(MathRuntimeError::Settlement { operation, cleanup })
            if matches!(*operation, MathRuntimeError::Pool(_))
                && matches!(*cleanup, MathRuntimeError::Panic(ref message)
                    if message.contains("public evaluator destructor failed")))
        );
        assert!(destroyed.load(Ordering::Acquire));
        assert!(
            threads.lock().unwrap().is_empty(),
            "metadata refusal must precede native evaluation"
        );
        drop(occupied);
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[test]
    fn actor_spawn_refusal_survives_evaluator_destructor_panic() {
        let (evaluator, mut options, _, destroyed) = fixture();
        // An aligned stack exceeding the process address space forces the actual
        // native thread builder's refusal without replacing its production path.
        options.stack_bytes = usize::MAX / 4 + 1;
        let scope = pse_kernels::ExecutionScope::new(Cancellation::default(), None);
        let owner = CallbackOwner::new(
            DestructivePreparation {
                evaluator,
                panic_correspondence: false,
            },
            scope.clone(),
            options.evaluations,
        );
        let result = actor(owner, options, scope, Cancellation::default());
        assert!(
            matches!(result, Err(MathRuntimeError::Settlement { operation, cleanup })
            if matches!(operation.as_ref(), MathRuntimeError::Infrastructure(_))
            && matches!(cleanup.as_ref(), MathRuntimeError::Panic(message) if message.contains("public evaluator destructor failed")))
        );
        assert!(destroyed.load(Ordering::Acquire));
    }
    #[test]
    fn abandoning_unpolled_preparation_contains_evaluator_destructor_panic() {
        let (evaluator, options, threads, destroyed) = fixture();
        let (service, _cache) =
            super::super::tests::service_with_policy(128 << 20, Default::default());
        let driver = crate::CancelSource::new();
        let preparation = service.prepare_surrogate(
            DestructivePreparation {
                evaluator,
                panic_correspondence: false,
            },
            options,
            pse_kernels::ExecutionScope::new(Cancellation::default(), None),
            &driver,
        );
        let abandoned =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(preparation)));
        assert!(abandoned.is_ok());
        assert!(destroyed.load(Ordering::Acquire));
        assert!(threads.lock().unwrap().is_empty());
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn failed_or_cancelled_construction_keeps_original_memory_through_final_evaluator_destruction()
     {
        use std::time::Duration;
        #[derive(Debug)]
        struct GatedPreparation {
            evaluator: Evaluator,
            calls: std::sync::atomic::AtomicUsize,
            cancel: bool,
            constructed: Sender<()>,
            construction_gate: Mutex<mpsc::Receiver<()>>,
            destroying: Sender<()>,
            destruction_gate: mpsc::Receiver<()>,
        }
        impl FidelityEvaluator for GatedPreparation {
            type Error = ProblemError;
            fn correspondence(&self) -> &FidelityCorrespondence {
                if self.calls.fetch_add(1, Ordering::SeqCst) == 2 {
                    self.constructed.send(()).unwrap();
                    let _ = self
                        .construction_gate
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_secs(10));
                    assert!(self.cancel, "actual retained constructor failed");
                }
                self.evaluator.correspondence()
            }
            fn evaluate(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), Self::Error> {
                self.evaluator.evaluate(x, out)
            }
        }
        impl Drop for GatedPreparation {
            fn drop(&mut self) {
                let _ = self.destroying.send(());
                let _ = self.destruction_gate.recv_timeout(Duration::from_secs(10));
                panic!("final evaluator destruction failed");
            }
        }
        for cancelled in [false, true] {
            let (evaluator, options, _, destroyed) = fixture();
            let (service, _cache) = super::super::tests::service_with_policy(
                128 << 20,
                super::super::MathPolicy {
                    stack_bytes: options.stack_bytes,
                    foreign_bytes: options.foreign_bytes,
                    workspace_bytes: 16 << 20,
                    worker_bytes: 8 << 20,
                    inner_session_bytes: 0,
                    ..Default::default()
                },
            );
            let baseline = service.pool.reserved();
            let (constructed, constructing) = mpsc::channel();
            let (release_construction, construction_gate) = mpsc::channel();
            let (destroying, destruction) = mpsc::channel();
            let (release_destruction, destruction_gate) = mpsc::channel();
            let evaluator = GatedPreparation {
                evaluator,
                calls: 0.into(),
                cancel: cancelled,
                constructed,
                construction_gate: Mutex::new(construction_gate),
                destroying,
                destruction_gate,
            };
            let driver = Arc::new(crate::CancelSource::new());
            let scope = pse_kernels::ExecutionScope::new(Cancellation::default(), None);
            let task_scope = scope.clone();
            let task_driver = driver.clone();
            let task_service = service.clone();
            let task = tokio::spawn(async move {
                task_service
                    .prepare_surrogate(evaluator, options, task_scope, &task_driver)
                    .await
            });
            constructing.recv_timeout(Duration::from_secs(5)).unwrap();
            if cancelled {
                driver.cancel();
                tokio::time::timeout(
                    Duration::from_secs(5),
                    scope.cancellation_control().cancelled(),
                )
                .await
                .unwrap();
            }
            release_construction.send(()).unwrap();
            destruction.recv_timeout(Duration::from_secs(5)).unwrap();
            let charged = service.pool.reserved();
            let pending = !task.is_finished();
            release_destruction.send(()).unwrap();
            let result = task.await.unwrap();
            assert!(pending);
            assert!(
                charged > baseline,
                "the original construction reservation must survive the last evaluator destructor"
            );
            let error = result.unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("final evaluator destruction failed")
            );
            assert!(error.to_string().contains(if cancelled {
                "cancel"
            } else {
                "actual retained constructor failed"
            }));
            assert!(destroyed.load(Ordering::Acquire));
            assert_eq!(service.pool.reserved(), baseline);
        }
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn failed_construction_keeps_original_memory_through_evaluator_panic_payload_destruction()
    {
        use std::time::Duration;
        struct Payload {
            entered: Sender<()>,
            release: mpsc::Receiver<()>,
        }
        impl Drop for Payload {
            fn drop(&mut self) {
                let _ = self.entered.send(());
                let _ = self.release.recv_timeout(Duration::from_secs(10));
            }
        }
        #[derive(Debug)]
        struct Panicking {
            evaluator: Evaluator,
            calls: std::sync::atomic::AtomicUsize,
            entered: Sender<()>,
            release: Option<mpsc::Receiver<()>>,
        }
        impl FidelityEvaluator for Panicking {
            type Error = ProblemError;
            fn correspondence(&self) -> &FidelityCorrespondence {
                assert!(
                    self.calls.fetch_add(1, Ordering::SeqCst) != 2,
                    "payload control constructor failed"
                );
                self.evaluator.correspondence()
            }
            fn evaluate(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), Self::Error> {
                self.evaluator.evaluate(x, out)
            }
        }
        impl Drop for Panicking {
            fn drop(&mut self) {
                std::panic::panic_any(Payload {
                    entered: self.entered.clone(),
                    release: self.release.take().unwrap(),
                });
            }
        }
        let (evaluator, options, _, destroyed) = fixture();
        let (service, _cache) = super::super::tests::service_with_policy(
            128 << 20,
            super::super::MathPolicy {
                stack_bytes: options.stack_bytes,
                foreign_bytes: options.foreign_bytes,
                workspace_bytes: 16 << 20,
                worker_bytes: 8 << 20,
                inner_session_bytes: 0,
                ..Default::default()
            },
        );
        let baseline = service.pool.reserved();
        let (entered, entering) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        let evaluator = Panicking {
            evaluator,
            calls: 0.into(),
            entered,
            release: Some(gate),
        };
        let task_service = service.clone();
        let task = tokio::spawn(async move {
            task_service
                .prepare_surrogate(
                    evaluator,
                    options,
                    pse_kernels::ExecutionScope::new(Cancellation::default(), None),
                    &crate::CancelSource::new(),
                )
                .await
        });
        entering.recv_timeout(Duration::from_secs(5)).unwrap();
        let charged = service.pool.reserved();
        let pending = !task.is_finished();
        release.send(()).unwrap();
        let result = task.await.unwrap();
        assert!(pending);
        assert!(
            charged > baseline,
            "original reservation must outlive the caught payload's destructor"
        );
        assert!(destroyed.load(Ordering::Acquire));
        assert!(
            matches!(result, Err(MathRuntimeError::Settlement { operation, cleanup })
            if matches!(operation.as_ref(), MathRuntimeError::Panic(message) if message.contains("payload control constructor failed"))
            && matches!(cleanup.as_ref(), MathRuntimeError::Panic(_)))
        );
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn cancellation_after_constructed_actor_preserves_retirement_error_and_tls_custody() {
        use std::time::Duration;
        struct Departure {
            entered: Sender<()>,
            release: mpsc::Receiver<()>,
        }
        impl Drop for Departure {
            fn drop(&mut self) {
                let _ = self.entered.send(());
                let _ = self.release.recv_timeout(Duration::from_secs(10));
            }
        }
        thread_local! {
            static DEPARTURE: std::cell::RefCell<Option<Departure>> = const { std::cell::RefCell::new(None) };
        }
        #[derive(Debug)]
        struct CleanupFailure {
            evaluator: Evaluator,
            entered: Sender<()>,
            release: Option<mpsc::Receiver<()>>,
        }
        impl FidelityEvaluator for CleanupFailure {
            type Error = ProblemError;
            fn correspondence(&self) -> &FidelityCorrespondence {
                self.evaluator.correspondence()
            }
            fn evaluate(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), Self::Error> {
                if let Some(release) = self.release.take() {
                    DEPARTURE.with(|slot| {
                        *slot.borrow_mut() = Some(Departure {
                            entered: self.entered.clone(),
                            release,
                        })
                    });
                }
                self.evaluator.evaluate(x, out)
            }
        }
        impl Drop for CleanupFailure {
            fn drop(&mut self) {
                panic!("actual constructed actor cleanup failed");
            }
        }
        let (evaluator, options, _, destroyed) = fixture();
        let workspace = options.workspace_bytes(evaluator.correspondence()).unwrap();
        let (service, _cache) = super::super::tests::service_with_policy(
            128 << 20,
            super::super::MathPolicy {
                stack_bytes: options.stack_bytes,
                foreign_bytes: options.foreign_bytes,
                workspace_bytes: 16 << 20,
                worker_bytes: 8 << 20,
                inner_session_bytes: 0,
                ..Default::default()
            },
        );
        let baseline = service.pool.reserved();
        let cores = service.cpu.available_permits();
        let (entered, entering) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        let (ready, constructed) = tokio::sync::oneshot::channel();
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let worker_service = service.clone();
        let task = tokio::spawn(async move {
            worker_service
                .job_retained_scoped_with_finish(
                    1,
                    workspace,
                    worker_cancel,
                    None,
                    move |flag| {
                        let scope = pse_kernels::ExecutionScope::new(Cancellation::default(), None);
                        let callback = CallbackOwner::new(
                            CleanupFailure {
                                evaluator,
                                entered,
                                release: Some(gate),
                            },
                            scope.clone(),
                            options.evaluations,
                        );
                        callback.retain_construction()?;
                        let actor =
                            Arc::new(actor(callback, options, scope, Cancellation::default())?);
                        actor.advance()?;
                        let _ = ready.send(());
                        let until = Instant::now() + Duration::from_secs(5);
                        while !flag.is_cancelled() && Instant::now() < until {
                            std::thread::yield_now();
                        }
                        Ok((actor, workspace))
                    },
                    |actor: Arc<Actor>| actor.retire(),
                )
                .await
        });
        tokio::time::timeout(Duration::from_secs(5), constructed)
            .await
            .unwrap()
            .unwrap();
        cancel.cancel();
        entering.recv_timeout(Duration::from_secs(5)).unwrap();
        let charged = service.pool.reserved();
        let available = service.cpu.available_permits();
        let pending = !task.is_finished();
        release.send(()).unwrap();
        let result = task.await.unwrap();
        assert!(pending);
        assert!(
            charged > baseline,
            "actor retirement must retain the original job reservation"
        );
        assert_eq!(
            available,
            cores - 1,
            "actor retirement still owns the original job compute"
        );
        assert!(
            matches!(result, Err(MathRuntimeError::Settlement { operation, cleanup })
            if matches!(operation.as_ref(), MathRuntimeError::Cancelled)
            && matches!(cleanup.as_ref(), MathRuntimeError::Panic(message) if message.contains("actual constructed actor cleanup failed")))
        );
        assert!(destroyed.load(Ordering::Acquire));
        assert_eq!(service.pool.reserved(), baseline);
        assert_eq!(service.cpu.available_permits(), cores);
    }
    #[test]
    fn actual_actor_retains_non_send_library_and_same_pool_then_joins_destruction() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SurrogateHandle>();
        let (e, o, threads, destroyed) = fixture();
        let control = Cancellation::default();
        let scope = pse_kernels::ExecutionScope::new(Cancellation::new(false), None);
        let task = actor(
            CallbackOwner::new(e, scope.clone(), o.evaluations),
            o,
            scope,
            control,
        )
        .unwrap();
        assert_eq!(task.work().unwrap().evaluations, 0);
        let first = task.advance().unwrap().proposal.unwrap();
        assert_eq!(task.work().unwrap().evaluations, 3);
        let second = task.advance().unwrap().proposal.unwrap();
        assert_eq!(first.task, second.task);
        let work = task.work().unwrap();
        assert_eq!(work.initializations, 1);
        assert_eq!(work.iterations, 1);
        let observed = threads.lock().unwrap();
        assert!(observed.len() > 3);
        assert!(observed.iter().all(|thread| *thread == observed[0]));
        drop(observed);
        assert!(!destroyed.load(Ordering::Acquire));
        task.retire().unwrap();
        assert!(matches!(task.work(), Err(MathRuntimeError::Retiring)));
        drop(task);
        assert!(destroyed.load(Ordering::Acquire));
    }
    #[test]
    fn actor_retirement_preserves_poison_stop_and_join_failures() {
        let (sender, receiver) = mpsc::channel();
        drop(receiver);
        let actor = Actor {
            observation: None,
            sender,
            join: Mutex::new(Some(std::thread::spawn(|| {
                panic!("actor worker exit failure")
            }))),
            gate: ActorGate::default(),
            abort: Cancellation::default(),
            owner: Mutex::new(None),
            _shell_owner: None,
        };
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = actor.join.lock().unwrap();
            panic!("injected join mutex poison");
        }));
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = actor.gate.mutex.lock().unwrap();
            panic!("injected gate poison");
        }));
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = actor.owner.lock().unwrap();
            panic!("injected allocation owner poison");
        }));
        let error = actor.retire().unwrap_err();
        let detail = error.to_string();
        for cause in [
            "join owner poisoned",
            "actor gate poisoned",
            "allocation owner poisoned",
            "stop submission lost",
            "actor worker exit failure",
        ] {
            assert!(
                detail.contains(cause),
                "missing independent failure: {cause}: {detail}"
            );
        }
        assert!(matches!(error, MathRuntimeError::Settlement { cleanup, .. }
            if matches!(*cleanup, MathRuntimeError::Panic(ref message) if message.contains("actor worker exit failure"))));
    }
    #[tokio::test]
    async fn consuming_handle_finish_fences_borrowed_actor_and_keeps_charge_through_tls_exit() {
        use std::time::Duration;
        struct Departure {
            entered: Sender<()>,
            release: mpsc::Receiver<()>,
            actor: Arc<Mutex<Option<Weak<Actor>>>>,
            reentered: Sender<bool>,
        }
        impl Drop for Departure {
            fn drop(&mut self) {
                let _ = self.entered.send(());
                let _ = self.release.recv_timeout(Duration::from_secs(10));
                let actor = self
                    .actor
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap();
                let (send, receive) = mpsc::channel();
                // Bound a regression: if retirement still holds the gate, TLS
                // can depart after the timeout and let the waiting call settle.
                std::thread::spawn(move || {
                    let _ = send.send(matches!(actor.work(), Err(MathRuntimeError::Retiring)));
                });
                let _ = self.reentered.send(
                    receive
                        .recv_timeout(Duration::from_secs(5))
                        .unwrap_or(false),
                );
            }
        }
        thread_local! {
            static DEPARTURE: std::cell::RefCell<Option<Departure>> = const { std::cell::RefCell::new(None) };
        }
        #[derive(Debug)]
        struct Gated {
            evaluator: Evaluator,
            entered: Sender<()>,
            release: Option<mpsc::Receiver<()>>,
            actor: Arc<Mutex<Option<Weak<Actor>>>>,
            reentered: Sender<bool>,
        }
        impl FidelityEvaluator for Gated {
            type Error = ProblemError;
            fn correspondence(&self) -> &FidelityCorrespondence {
                self.evaluator.correspondence()
            }
            fn evaluate(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), Self::Error> {
                if let Some(release) = self.release.take() {
                    DEPARTURE.with(|slot| {
                        *slot.borrow_mut() = Some(Departure {
                            entered: self.entered.clone(),
                            release,
                            actor: self.actor.clone(),
                            reentered: self.reentered.clone(),
                        })
                    });
                }
                self.evaluator.evaluate(x, out)
            }
        }
        let (evaluator, options, _, destroyed) = fixture();
        let (entered, departing) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        let actor_slot = Arc::new(Mutex::new(None));
        let (reentered, reentry) = mpsc::channel();
        let evaluator = Gated {
            evaluator,
            entered,
            release: Some(gate),
            actor: actor_slot.clone(),
            reentered,
        };
        let (service, _cache) = super::super::tests::service_with_policy(
            128 << 20,
            super::super::MathPolicy {
                stack_bytes: options.stack_bytes,
                foreign_bytes: options.foreign_bytes,
                workspace_bytes: 16 << 20,
                worker_bytes: 8 << 20,
                inner_session_bytes: 0,
                ..Default::default()
            },
        );
        let baseline = service.pool.reserved();
        let handle = service
            .prepare_surrogate(
                evaluator,
                options,
                pse_kernels::ExecutionScope::new(Cancellation::default(), None),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        *actor_slot.lock().unwrap() = Some(Arc::downgrade(&handle.actor));
        service
            .advance_surrogate(&handle, &crate::CancelSource::new())
            .await
            .unwrap();
        let retained = handle.retained_bytes();
        let metadata = handle.metadata.clone();
        let compact = metadata._owner.size() + metadata._actor_shell_owner.size();
        let actor = handle.actor.clone();
        let (send, finished) = mpsc::channel();
        let finisher = std::thread::spawn(move || send.send(handle.finish()).unwrap());
        departing.recv_timeout(Duration::from_secs(5)).unwrap();
        let pending = matches!(finished.try_recv(), Err(mpsc::TryRecvError::Empty));
        let charged = service.pool.reserved();
        let queued_actor = actor.clone();
        let queued = std::thread::spawn(move || queued_actor.work());
        release.send(()).unwrap();
        let result = finished.recv_timeout(Duration::from_secs(10)).unwrap();
        finisher.join().unwrap();
        let refused = queued.join().unwrap();
        assert!(
            pending,
            "consuming finish must await actual TLS destruction"
        );
        assert!(
            charged >= baseline + retained,
            "retained charge must survive physical TLS exit"
        );
        result.unwrap();
        assert!(matches!(refused, Err(MathRuntimeError::Retiring)));
        assert!(
            reentry.recv_timeout(Duration::from_secs(1)).unwrap(),
            "TLS reentry must observe closed admission before its destructor departs"
        );
        assert!(destroyed.load(Ordering::Acquire));
        assert!(actor.owner.lock().unwrap().is_none());
        assert_eq!(
            service.pool.reserved(),
            baseline + compact,
            "native workspace must release before idle actor/metadata aliases"
        );
        drop(metadata);
        assert_eq!(
            service.pool.reserved(),
            baseline + actor._shell_owner.as_ref().unwrap().size()
        );
        drop(actor);
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[tokio::test]
    async fn consuming_finish_waits_for_active_command_then_refuses_later_evaluation() {
        use std::time::Duration;
        #[derive(Debug)]
        struct Blocked {
            evaluator: Evaluator,
            entered: Option<Sender<()>>,
            release: mpsc::Receiver<()>,
        }
        impl FidelityEvaluator for Blocked {
            type Error = ProblemError;
            fn correspondence(&self) -> &FidelityCorrespondence {
                self.evaluator.correspondence()
            }
            fn evaluate(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), Self::Error> {
                if let Some(entered) = self.entered.take() {
                    entered.send(()).unwrap();
                    self.release.recv_timeout(Duration::from_secs(10)).unwrap();
                }
                self.evaluator.evaluate(x, out)
            }
        }
        let (evaluator, options, threads, destroyed) = fixture();
        let (entered, entering) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        let (service, _cache) = super::super::tests::service_with_policy(
            128 << 20,
            super::super::MathPolicy {
                stack_bytes: options.stack_bytes,
                foreign_bytes: options.foreign_bytes,
                workspace_bytes: 16 << 20,
                worker_bytes: 8 << 20,
                inner_session_bytes: 0,
                ..Default::default()
            },
        );
        let baseline = service.pool.reserved();
        let handle = service
            .prepare_surrogate(
                Blocked {
                    evaluator,
                    entered: Some(entered),
                    release: gate,
                },
                options,
                pse_kernels::ExecutionScope::new(Cancellation::default(), None),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let actor = handle.actor.clone();
        let active = actor.clone();
        let command = std::thread::spawn(move || active.advance());
        entering.recv_timeout(Duration::from_secs(5)).unwrap();
        let native_bytes = handle.retained_bytes();
        let (send, receive) = mpsc::channel();
        let finisher = std::thread::spawn(move || {
            let _ = send.send(handle.finish());
        });
        let until = Instant::now() + Duration::from_secs(5);
        while actor.join.lock().unwrap().is_some() && Instant::now() < until {
            std::thread::yield_now();
        }
        let closed = actor.join.lock().unwrap().is_none();
        let pending = matches!(receive.try_recv(), Err(mpsc::TryRecvError::Empty));
        let charged = service.pool.reserved();
        release.send(()).unwrap();
        let command_result = command.join().unwrap();
        let finished = receive.recv_timeout(Duration::from_secs(5)).unwrap();
        finisher.join().unwrap();
        assert!(
            closed && pending,
            "finish closes admission but fences the active command"
        );
        assert!(charged >= baseline + native_bytes);
        assert!(
            command_result.is_err(),
            "active callback sees original cancellation after departure"
        );
        finished.unwrap();
        let calls = threads.lock().unwrap().len();
        assert_eq!(calls, 1);
        assert!(matches!(actor.advance(), Err(MathRuntimeError::Retiring)));
        assert_eq!(threads.lock().unwrap().len(), calls);
        assert!(destroyed.load(Ordering::Acquire));
        assert_eq!(
            service.pool.reserved(),
            baseline + actor._shell_owner.as_ref().unwrap().size()
        );
        drop(actor);
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[test]
    fn actor_join_payload_can_reenter_closed_actor_without_holding_retirement_gate() {
        use std::time::Duration;
        struct Payload {
            actor: Arc<Mutex<Option<Weak<Actor>>>>,
            observed: Sender<bool>,
        }
        impl Drop for Payload {
            fn drop(&mut self) {
                let actor = self
                    .actor
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap();
                let (send, receive) = mpsc::channel();
                std::thread::spawn(move || {
                    let _ = send.send(matches!(actor.work(), Err(MathRuntimeError::Retiring)));
                });
                let _ = self.observed.send(
                    receive
                        .recv_timeout(Duration::from_secs(5))
                        .unwrap_or(false),
                );
            }
        }
        let (sender, commands) = mpsc::channel();
        let slot = Arc::new(Mutex::new(None));
        let (observed, observation) = mpsc::channel();
        let payload = Payload {
            actor: slot.clone(),
            observed,
        };
        let join = std::thread::spawn(move || -> Result<(), MathRuntimeError> {
            assert!(matches!(
                commands.recv_timeout(Duration::from_secs(5)),
                Ok(Command::Stop)
            ));
            std::panic::panic_any(payload);
        });
        let actor = Arc::new(Actor {
            observation: None,
            sender,
            join: Mutex::new(Some(join)),
            gate: ActorGate::default(),
            abort: Cancellation::default(),
            owner: Mutex::new(None),
            _shell_owner: None,
        });
        *slot.lock().unwrap() = Some(Arc::downgrade(&actor));
        let result = actor.retire();
        assert!(
            observation.recv_timeout(Duration::from_secs(1)).unwrap(),
            "join-payload destruction must observe Retiring before it departs"
        );
        assert!(matches!(result, Err(MathRuntimeError::Panic(_))));
    }
    #[tokio::test]
    async fn handle_cancel_wakes_scope_waiter_before_actor_gate_release() {
        use std::{future::Future, time::Duration};
        let (evaluator, options, _, destroyed) = fixture();
        let (service, _cache) = super::super::tests::service_with_policy(
            128 << 20,
            super::super::MathPolicy {
                stack_bytes: options.stack_bytes,
                foreign_bytes: options.foreign_bytes,
                workspace_bytes: 16 << 20,
                worker_bytes: 8 << 20,
                inner_session_bytes: 0,
                ..Default::default()
            },
        );
        let baseline = service.pool.reserved();
        let scope = pse_kernels::ExecutionScope::new(Cancellation::default(), None);
        let handle = service
            .prepare_surrogate(evaluator, options, scope, &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(!Arc::ptr_eq(
            &handle.metadata.scope.cancellation_control().flag(),
            &handle.control.flag()
        ));
        let held = handle.actor.gate.lock().unwrap();
        let waiter = handle.metadata.clone();
        let waiter_actor = handle.actor.clone();
        let (registered, registration) = mpsc::sync_channel(1);
        let (send, receive) = mpsc::sync_channel(1);
        let join = std::thread::spawn(move || {
            let control = waiter.scope.cancellation_control().clone();
            let mut stop = std::pin::pin!(control.cancelled());
            let mut registered = Some(registered);
            let stop = futures_util::future::poll_fn(move |context| {
                let polled = stop.as_mut().poll(context);
                if polled.is_pending()
                    && let Some(registered) = registered.take()
                {
                    registered.send(()).unwrap();
                }
                polled
            });
            let result = waiter_actor.gate.lock_wait(
                || {
                    waiter
                        .scope
                        .check()
                        .map_err(ProblemError::Provider)
                        .map_err(Into::into)
                },
                stop,
                waiter.scope.deadline(),
            );
            send.send(matches!(
                result,
                Err(MathRuntimeError::Solve(ProblemError::Provider(
                    pse_kernels::ProviderError::Cancelled
                )))
            ))
            .unwrap();
        });
        let registration = registration.recv_timeout(Duration::from_secs(1));
        handle.cancel();
        let result = receive.recv_timeout(Duration::from_secs(1));
        // Release and join even on regression: a failed notification must not leak
        // the waiter or the actor's actual library/pool lifetime out of this test.
        drop(held);
        join.join().unwrap();
        assert!(
            registration.is_ok(),
            "waiter must register before cancellation"
        );
        assert!(
            result.unwrap(),
            "scope cancellation must wake the held-gate waiter"
        );
        assert!(handle.metadata.scope.cancellation_control().is_cancelled());
        assert!(handle.control.is_cancelled());
        handle.finish().unwrap();
        assert!(destroyed.load(Ordering::Acquire));
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[test]
    fn actor_refuses_expired_original_scope_before_evaluation_and_joins_failed_construction() {
        let (e, o, threads, destroyed) = fixture();
        let scope = pse_kernels::ExecutionScope::new(
            Cancellation::new(false),
            Some(Instant::now() - std::time::Duration::from_secs(1)),
        );
        assert!(matches!(
            actor(
                CallbackOwner::new(e, scope.clone(), o.evaluations),
                o,
                scope,
                Cancellation::default()
            ),
            Err(MathRuntimeError::Math(pse_math::MathError::Scope(
                pse_kernels::ProviderError::Deadline
            )))
        ));
        assert!(threads.lock().unwrap().is_empty());
        assert!(destroyed.load(Ordering::Acquire));
    }
    #[test]
    fn actor_phase_retains_actual_partial_failure_work_and_terminal_cause() {
        #[derive(Debug)]
        struct Failed(FidelityCorrespondence);
        impl FidelityEvaluator for Failed {
            type Error = ProblemError;
            fn correspondence(&self) -> &FidelityCorrespondence {
                &self.0
            }
            fn evaluate(&mut self, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
                Err(ProblemError::Math(pse_math::MathError::Domain {
                    source_id: SemanticId::from_bytes([34; 16]),
                    requirement: "actual fidelity terminal",
                }))
            }
        }
        let (e, o, _, _) = fixture();
        let cancel = Cancellation::default();
        let scope = pse_kernels::ExecutionScope::new(
            cancel.clone(),
            Some(Instant::now() + std::time::Duration::from_secs(20)),
        );
        let task = actor(
            CallbackOwner::new(
                Failed(e.correspondence.clone()),
                scope.clone(),
                o.evaluations,
            ),
            o,
            scope.clone(),
            Cancellation::default(),
        )
        .unwrap();
        let reply = task.phase(scope.clone()).unwrap();
        assert!(matches!(
            reply.proposal,
            Err(MathRuntimeError::Solve(ProblemError::Math(
                pse_math::MathError::Domain {
                    requirement: "actual fidelity terminal",
                    ..
                }
            )))
        ));
        assert!(reply.callback_terminal);
        let work = reply.work.unwrap();
        assert_eq!(work.evaluations, 1);
        assert_eq!(work.initializations, 1);
        assert_eq!(work.iterations, 0);
        assert!(!cancel.load(Ordering::Acquire));
        let reply = task.phase(scope).unwrap();
        assert!(reply.proposal.is_err());
        assert_eq!(reply.work.unwrap().evaluations, 0);
    }

    #[test]
    fn actor_phase_late_callback_retains_deadline_and_partial_work_without_cancelling_outer_task() {
        #[derive(Debug)]
        struct Slow(FidelityCorrespondence);
        impl FidelityEvaluator for Slow {
            type Error = ProblemError;
            fn correspondence(&self) -> &FidelityCorrespondence {
                &self.0
            }
            fn evaluate(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
                std::thread::sleep(std::time::Duration::from_millis(100));
                out[0] = 0.;
                Ok(())
            }
        }
        let (e, o, _, _) = fixture();
        let cancel = Cancellation::default();
        let scope = pse_kernels::ExecutionScope::new(
            cancel.clone(),
            Some(Instant::now() + std::time::Duration::from_secs(20)),
        );
        let task = actor(
            CallbackOwner::new(Slow(e.correspondence.clone()), scope.clone(), o.evaluations),
            o,
            scope,
            Cancellation::default(),
        )
        .unwrap();
        let phase = pse_kernels::ExecutionScope::new(
            cancel.clone(),
            Some(Instant::now() + std::time::Duration::from_millis(50)),
        );
        let reply = task.phase(phase).unwrap();
        assert!(matches!(
            reply.proposal,
            Err(MathRuntimeError::Math(pse_math::MathError::Scope(
                pse_kernels::ProviderError::Deadline
            )))
        ));
        assert!(reply.callback_terminal);
        let work = reply.work.unwrap();
        assert_eq!(work.evaluations, 1);
        assert_eq!(work.initializations, 1);
        assert_eq!(work.iterations, 0);
        assert!(!cancel.load(Ordering::Acquire));
        assert!(task.proposal().unwrap().is_none());
    }

    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn retained_surrogate_phase_runs_in_shared_driver_then_original_correction_grants_permission()
     {
        retained_surrogate_corrector(false).await;
    }
    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn automatic_supplied_surrogate_runs_original_corrector_and_preserves_phase_permissions()
    {
        retained_surrogate_corrector(true).await;
    }
    #[cfg(feature = "solver-kinsol")]
    async fn retained_surrogate_corrector(automatic: bool) {
        use super::super::solves::{NumericalInputs, PreparedRung, SolverProfile};
        use crate::workflow::tests as workflow_fixture;
        use pse_backend_native::solve::{Backend, SolveIntent, SolverSelection, StartPolicy};
        use pse_model::strategy::{
            Mechanism, MechanismKind, NumericalStrategy, Position, ProfileRef, StartOrigin,
            Transition, WorkLimits,
        };
        let runtime = workflow_fixture::runtime_with(128 << 20, 1 << 20, 512 << 20);
        let rows = pse_authoring::language::parse(
            "package p { def Root { var x:Scalar; annotation start x(0); eq balance:x==0.8; } def Other { var x:Scalar; annotation start x(0); eq balance:x==0.7; } }",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let other = rows
            .iter()
            .find(|row| row.name == "Other")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, workflow_fixture::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let profile = SolverProfile {
            intent: SolveIntent::Root,
            selection: SolverSelection::Explicit(Backend::Kinsol),
            composition: pse_model::strategy::CompositionRequest {
                recovery: vec![StartOrigin::Surrogate],
                ..Default::default()
            },
            ..Default::default()
        };
        let mut prepared = package
            .prepare_solve(
                &runtime.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                Default::default(),
                pse_kernels::DerivativeOrder::First,
                workflow_fixture::compiler_profile(),
                profile,
                NumericalInputs::default(),
                &cancel,
            )
            .await
            .unwrap();
        let original = prepared.solve.clone();
        let original_profile = original.strategy_profile().unwrap();
        let (mut evaluator, mut options, threads, destroyed) = fixture();
        options.stack_bytes = runtime.native().stack_bytes();
        let identity = original.original_identity().unwrap();
        evaluator.correspondence.original_target = identity;
        evaluator.correspondence.model_target = identity;
        evaluator.correspondence.coordinates = original.original_coordinates().unwrap();
        let scope = pse_kernels::ExecutionScope::new(
            Cancellation::new(false),
            Some(Instant::now() + std::time::Duration::from_secs(20)),
        );
        let handle = runtime
            .native()
            .prepare_surrogate(evaluator, options, scope.clone(), &cancel)
            .await
            .unwrap();
        let unrelated =
            pse_kernels::ExecutionScope::new(Cancellation::new(false), scope.deadline());
        assert!(
            runtime
                .native()
                .prepare_surrogate_phase(
                    &handle,
                    original.clone(),
                    unrelated.clone(),
                    std::time::Duration::from_secs(10)
                )
                .is_err()
        );
        assert!(
            runtime
                .native()
                .prepare_surrogate_phase(
                    &handle,
                    original.clone(),
                    scope.clone(),
                    std::time::Duration::ZERO
                )
                .is_err()
        );
        let phase = runtime
            .native()
            .prepare_surrogate_phase(
                &handle,
                original.clone(),
                scope.clone(),
                std::time::Duration::from_secs(10),
            )
            .unwrap();
        let idle_phase = phase.clone();
        let idle_rung = PreparedRung::Surrogate(phase.clone());
        let actor_alias = handle.actor.clone();
        let metadata_charge = Arc::downgrade(&handle.metadata._owner);
        let phase_charge = Arc::downgrade(&phase.0._owner);
        let binding_key = phase.key();
        let support = phase.support();
        assert_eq!(handle.work().unwrap().evaluations, 0);
        let limits = WorkLimits {
            attempts: 4,
            evaluations: None,
            iterations: None,
            factorizations: None,
            proof_steps: None,
        };
        let mut declaration = NumericalStrategy::direct(StartPolicy::NoPriorStart, limits);
        declaration.start.recovery = vec![StartOrigin::Surrogate];
        declaration.mechanisms = vec![
            Mechanism {
                operation: Default::default(),
                kind: MechanismKind::Surrogate,
                position: Position::Preparation,
                required: true,
                profile: None,
                support: phase.support().into_iter().collect(),
                limits,
                starts: vec![StartOrigin::Specification],
                transitions: vec![Transition::Continue, Transition::Stop],
            },
            Mechanism {
                operation: Default::default(),
                kind: MechanismKind::Direct,
                position: Position::Execution,
                required: true,
                profile: Some(ProfileRef {
                    method: pse_model::strategy::NumericalMethod::Backend(Backend::Kinsol),
                    key: original.strategy_profile().unwrap(),
                }),
                support: vec![],
                limits,
                starts: vec![StartOrigin::Surrogate, StartOrigin::Specification],
                transitions: vec![Transition::Finish, Transition::Stop],
            },
        ];
        if automatic {
            let denied_profile = SolverProfile {
                intent: SolveIntent::Root,
                selection: SolverSelection::Explicit(Backend::Kinsol),
                ..Default::default()
            };
            let denied = package
                .prepare_solve(
                    &runtime.preparation_services(),
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    pse_kernels::DerivativeOrder::First,
                    workflow_fixture::compiler_profile(),
                    denied_profile,
                    NumericalInputs::default(),
                    &cancel,
                )
                .await
                .unwrap();
            assert!(
                denied
                    .solve
                    .clone()
                    .within_task(scope.clone())
                    .unwrap()
                    .with_automatic_products(vec![PreparedRung::Surrogate(phase.clone())])
                    .is_err(),
                "producer-bound proposal requires declared recovery permission"
            );
            let mismatch = package
                .prepare_solve(
                    &runtime.preparation_services(),
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    pse_kernels::DerivativeOrder::First,
                    workflow_fixture::compiler_profile(),
                    SolverProfile {
                        intent: SolveIntent::Root,
                        selection: SolverSelection::Explicit(Backend::Kinsol),
                        composition: pse_model::strategy::CompositionRequest {
                            recovery: vec![StartOrigin::Surrogate],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    NumericalInputs::default(),
                    &cancel,
                )
                .await
                .unwrap();
            // A different task has no authority to consume this actual actor.
            assert!(
                mismatch
                    .solve
                    .within_task(unrelated.clone())
                    .unwrap()
                    .with_automatic_products(vec![PreparedRung::Surrogate(phase.clone())])
                    .is_err()
            );
            let wrong_original = package
                .prepare_solve(
                    &runtime.preparation_services(),
                    other,
                    pse_modeling::specialize::root_instance(other),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    pse_kernels::DerivativeOrder::First,
                    workflow_fixture::compiler_profile(),
                    SolverProfile {
                        intent: SolveIntent::Root,
                        selection: SolverSelection::Explicit(Backend::Kinsol),
                        composition: pse_model::strategy::CompositionRequest {
                            recovery: vec![StartOrigin::Surrogate],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    NumericalInputs::default(),
                    &cancel,
                )
                .await
                .unwrap();
            assert!(
                wrong_original
                    .solve
                    .within_task(scope.clone())
                    .unwrap()
                    .with_automatic_products(vec![PreparedRung::Surrogate(phase.clone())])
                    .is_err()
            );
            prepared.solve = original
                .clone()
                .within_task(scope)
                .unwrap()
                .with_automatic_products(vec![PreparedRung::Surrogate(phase)])
                .unwrap();
        } else {
            prepared.solve = original
                .clone()
                .within_task(scope)
                .unwrap()
                .with_strategy(
                    declaration,
                    vec![PreparedRung::Surrogate(phase), original.into()],
                )
                .unwrap();
        }
        let action = async {
        let result = package
            .solve_case(
                &runtime.execution_services(),
                prepared,
                workflow_fixture::compiler_profile(),
                &cancel,
            )
            .await?;
        assert!(result.accepted, "diagnostic={:?}", result.diagnostic());
        assert_eq!(handle.work().unwrap().evaluations, 3);
        assert_eq!(handle.work().unwrap().initializations, 1);
        let trace = result.strategy.as_ref().unwrap();
        assert_eq!(trace.declaration.start.policy, StartPolicy::NoPriorStart);
        assert_eq!(trace.starts[1], StartOrigin::Surrogate);
        assert!(
            matches!(&trace.products[1].provider,Some(crate::math::strategy::ProviderEvidence::Native(profile)) if profile.key==original_profile),
            "corrector retains its frozen native profile and entry policy"
        );
        let rows = trace
            .rows(result.run_id, 0, runtime.native(), 0..usize::MAX)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let statistical = rows
            .iter()
            .find(|row| {
                row.mechanism == MechanismKind::Surrogate && row.surrogate_coordinates.is_some()
            })
            .unwrap();
        if automatic {
            assert_eq!(trace.declaration.mechanisms.len(), 2);
            assert_eq!(
                trace.declaration.mechanisms[0].kind,
                MechanismKind::Surrogate
            );
            assert_eq!(trace.declaration.mechanisms[1].kind, MechanismKind::Direct);
            assert!(trace.events.iter().any(|event| event.mechanism == 0
                && event.permission == Some(pse_model::generated::enums::CandidateUse::SeedOnly)));
            assert!(trace.events.iter().any(|event| {
                event.mechanism == 1
                    && event
                        .original
                        .as_ref()
                        .is_some_and(crate::math::strategy::OriginalConclusion::satisfied)
            }));
            assert_eq!(trace.starts[1], StartOrigin::Surrogate);
        }
        let sample = handle.proposal().unwrap().unwrap();
        assert_eq!(statistical.backend, None);
        assert_eq!(statistical.evaluations, Some(5));
        assert_eq!(
            statistical.surrogate_coordinates.as_deref(),
            Some(sample.coordinates.as_slice())
        );
        assert_eq!(statistical.infill_statistic, sample.infill_statistic);
        assert_eq!(statistical.infill_statistic, None);
        assert_eq!(statistical.surrogate_radius, Some(sample.radius));
        assert_eq!(
            statistical.surrogate_model_values.as_deref(),
            Some(sample.model_values.as_slice())
        );
        let point = result.path_start(vec![0., 1.]).unwrap();
        assert!((point.point()[0] - 0.8).abs() < 1e-7);
        assert!((handle.proposal().unwrap().unwrap().coordinates[0] - 0.8).abs() > 0.1);
        let observed = threads.lock().unwrap();
        assert!(observed.iter().all(|thread| *thread == observed[0]));
        drop(observed);
        Ok::<_, crate::workflow::WorkflowError>(result)
        }.await;
        let charged = runtime.native().pool.reserved();
        let native_charge = handle.actor.owner.lock().unwrap().as_ref().unwrap().size();
        let retirement = handle
            .finish()
            .map_err(crate::workflow::WorkflowError::from);
        let result = crate::workflow::WorkflowError::settle(action, retirement).unwrap();
        assert!(matches!(
            actor_alias.work(),
            Err(MathRuntimeError::Retiring)
        ));
        assert_eq!(
            runtime.native().pool.reserved() + native_charge,
            charged,
            "idle phase/rung/actor aliases must release the physical native charge at finish"
        );
        assert_eq!(idle_phase.key(), binding_key);
        assert_eq!(idle_phase.support(), support);
        assert!(metadata_charge.upgrade().is_some());
        assert!(phase_charge.upgrade().is_some());
        drop(result);
        drop(idle_phase);
        assert!(
            metadata_charge.upgrade().is_some(),
            "the final rung alias still retains compact metadata"
        );
        drop(idle_rung);
        assert!(phase_charge.upgrade().is_none());
        assert!(metadata_charge.upgrade().is_none());
        drop(actor_alias);
        assert!(destroyed.load(Ordering::Acquire));
    }
}

#[cfg(test)]
mod gate_wait_tests {
    use super::*;
    #[test]
    fn poisoned_surrogate_owner_is_internal_and_never_input_or_resource_refusal() {
        use pse_model::diagnostic::DiagnosticProjection;
        let gate = ActorGate::default();
        assert!(
            std::panic::catch_unwind(|| {
                let _held = gate.lock().unwrap();
                panic!("injected owned callback unwind");
            })
            .is_err()
        );
        let error = gate.lock().err().unwrap();
        assert!(matches!(error, MathRuntimeError::Internal(_)));
        let diagnostic = error.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation);
        assert_eq!(
            diagnostic.class,
            pse_model::diagnostic::BoundaryClass::Internal
        );
        assert_eq!(
            diagnostic.rule,
            pse_diagnostics::DiagnosticRule::WorkflowInternal
        );
    }
    #[test]
    fn surrogate_gate_release_and_cancel_wake_original_waiters() {
        for cancel in [false, true] {
            let gate = Arc::new(ActorGate::default());
            let held = gate.lock().unwrap();
            let stop = pse_columnar::CancellationToken::new();
            let caller = stop.clone();
            let other = gate.clone();
            let (send, recv) = mpsc::sync_channel(1);
            let join = std::thread::spawn(move || {
                send.send(()).unwrap();
                other
                    .lock_wait(
                        || caller.checkpoint().map_err(|_| MathRuntimeError::Cancelled),
                        caller.cancelled(),
                        None,
                    )
                    .is_ok()
            });
            recv.recv().unwrap();
            if cancel {
                stop.cancel();
            } else {
                drop(held);
            }
            assert_eq!(join.join().unwrap(), !cancel);
        }
    }
    #[test]
    fn surrogate_gate_deadline_remains_original_and_does_not_request_stop() {
        let gate = ActorGate::default();
        let _held = gate.lock().unwrap();
        let stop = Cancellation::default();
        let at = Instant::now();
        let scope = pse_kernels::ExecutionScope::new(stop.clone(), Some(at));
        assert!(
            gate.lock_wait(
                || scope
                    .check()
                    .map_err(ProblemError::Provider)
                    .map_err(Into::into),
                stop.cancelled(),
                Some(at)
            )
            .is_err()
        );
        assert!(!stop.is_cancelled());
    }
}
