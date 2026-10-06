// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Task-admitted egobox actor. Non-Send library objects never leave their owning worker.
use super::{MathRuntimeError, MathService};
use pse_backend_native::ProblemError;
use pse_columnar::flight::FlightCancellation;
use pse_math::surrogate::{
    FidelityCorrespondence, FidelityEvaluator, RetainedSurrogate, SurrogateFailure,
    SurrogateOptions, SurrogateProposal, SurrogateWork,
};
use std::{
    sync::{
        Arc, Mutex,
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
    Advance(SyncSender<ProposalResult>),
    Proposal(SyncSender<ProposalResult>),
    Work(SyncSender<Result<SurrogateWork, MathRuntimeError>>),
    Stop,
}
struct Actor {
    sender: Sender<Command>,
    join: Mutex<Option<JoinHandle<()>>>,
    gate: Mutex<()>,
    abort: FlightCancellation,
    owner: Mutex<Option<Arc<pse_columnar::AllocationLease>>>,
}
impl std::fmt::Debug for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurrogateActor").finish_non_exhaustive()
    }
}
impl Drop for Actor {
    fn drop(&mut self) {
        self.abort.cancel();
        let _ = self.sender.send(Command::Stop);
        let slot = match self.join.get_mut() {
            Ok(slot) => slot,
            Err(poison) => poison.into_inner(),
        };
        if let Some(join) = slot.take() {
            let _ = join.join();
        }
    }
}
impl Actor {
    fn proposal(&self, advance: bool) -> ProposalResult {
        let _gate = self
            .gate
            .lock()
            .map_err(|_| MathRuntimeError::Panic("surrogate actor gate poisoned".into()))?;
        let (reply, answer) = mpsc::sync_channel(1);
        self.sender
            .send(if advance {
                Command::Advance(reply)
            } else {
                Command::Proposal(reply)
            })
            .map_err(|_| MathRuntimeError::Panic("surrogate owning worker exited".into()))?;
        answer
            .recv()
            .map_err(|_| MathRuntimeError::Panic("surrogate owning worker response lost".into()))?
    }
    fn phase(&self, scope: pse_kernels::ExecutionScope) -> Result<PhaseReply, MathRuntimeError> {
        let _gate = loop {
            scope.check().map_err(ProblemError::Provider)?;
            match self.gate.try_lock() {
                Ok(gate) => break gate,
                Err(std::sync::TryLockError::Poisoned(_)) => {
                    return Err(MathRuntimeError::Panic(
                        "surrogate actor gate poisoned".into(),
                    ));
                }
                Err(std::sync::TryLockError::WouldBlock) => {
                    std::thread::sleep(std::time::Duration::from_millis(1))
                }
            }
        };
        let (reply, answer) = mpsc::sync_channel(1);
        self.sender
            .send(Command::Phase(scope, reply))
            .map_err(|_| MathRuntimeError::Panic("surrogate owning worker exited".into()))?;
        answer.recv().map_err(|_| {
            MathRuntimeError::Panic("surrogate owning worker phase response lost".into())
        })
    }
    fn work(&self) -> Result<SurrogateWork, MathRuntimeError> {
        let _gate = self
            .gate
            .lock()
            .map_err(|_| MathRuntimeError::Panic("surrogate actor gate poisoned".into()))?;
        let (reply, answer) = mpsc::sync_channel(1);
        self.sender
            .send(Command::Work(reply))
            .map_err(|_| MathRuntimeError::Panic("surrogate owning worker exited".into()))?;
        answer
            .recv()
            .map_err(|_| MathRuntimeError::Panic("surrogate owning worker response lost".into()))?
    }
}
/// Send channel owner for one actual non-Send solver/state and its accounted worker/pool lifetime.
/// No global model cache, rebuilt solver or unsafe Send implementation.
#[derive(Debug)]
pub struct SurrogateHandle {
    service: Arc<MathService>,
    actor: Arc<Actor>,
    owner: Arc<pse_columnar::AllocationLease>,
    control: FlightCancellation,
    scope: pse_kernels::ExecutionScope,
    task_scope: pse_kernels::ExecutionScope,
    correspondence: Arc<FidelityCorrespondence>,
    cores: usize,
    workspace: usize,
    task: pse_ids::ContentHash,
}
impl SurrogateHandle {
    /// Complete actual source/correspondence/options task identity.
    pub fn key(&self) -> pse_ids::ContentHash {
        self.task
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
        self.scope.check().map_err(scope_error)?;
        self.actor
            .proposal(false)?
            .map(|proposal| {
                let owner = self
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
        self.owner.size()
    }
    /// Request cooperative interruption; active admitted work joins before releasing CPU.
    pub fn cancel(&self) {
        self.scope
            .cancellation()
            .store(true, std::sync::atomic::Ordering::Release);
        self.control.cancel();
    }
}
fn failure(error: SurrogateFailure<ProblemError>) -> MathRuntimeError {
    match error {
        SurrogateFailure::Callback(e) => e.into(),
        SurrogateFailure::Math(e) => e.into(),
        SurrogateFailure::Panic(message) => MathRuntimeError::Panic(message),
    }
}
fn scope_error(error: pse_kernels::ProviderError) -> MathRuntimeError {
    match error {
        pse_kernels::ProviderError::Cancelled => MathRuntimeError::Cancelled,
        other => MathRuntimeError::Math(pse_math::MathError::Scope(other)),
    }
}
fn actor<E: FidelityEvaluator<Error = ProblemError>>(
    evaluator: E,
    options: SurrogateOptions,
    scope: pse_kernels::ExecutionScope,
    abort: FlightCancellation,
) -> Result<Actor, MathRuntimeError> {
    let (sender, commands) = mpsc::channel();
    let (ready, initialized) = mpsc::sync_channel(1);
    let flag = abort.flag();
    let stack = options.stack_bytes;
    let join = std::thread::Builder::new()
        .name("pse-egobox".into())
        .stack_size(stack)
        .spawn(move || {
            // Construct here: egobox's boxed strategies are Sync but not Send.
            let initialized_task = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut task =
                    RetainedSurrogate::new(evaluator, options, scope).map_err(failure)?;
                task.bind_abort(flag)?;
                Ok::<_, MathRuntimeError>(task)
            }));
            let mut task = match initialized_task {
                Ok(Ok(task)) => task,
                Ok(Err(error)) => {
                    let _ = ready.send(Err(error));
                    return;
                }
                Err(_) => {
                    let _ = ready.send(Err(MathRuntimeError::Panic(
                        "egobox construction unwound".into(),
                    )));
                    return;
                }
            };
            if ready.send(Ok(())).is_err() {
                return;
            }
            while let Ok(command) = commands.recv() {
                match command {
                    Command::Phase(scope, reply) => {
                        let before = task.work();
                        let proposal = task.advance_within(scope).map_err(failure);
                        let after = task.work();
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
                    Command::Stop => break,
                }
            }
            // Full solver/state and local rayon pool drop here before this thread exits.
        })
        .map_err(|e| MathRuntimeError::Infrastructure(e.to_string()))?;
    match initialized.recv() {
        Ok(Ok(())) => Ok(Actor {
            sender,
            join: Mutex::new(Some(join)),
            gate: Mutex::new(()),
            abort,
            owner: Mutex::new(None),
        }),
        Ok(Err(error)) => {
            let _ = join.join();
            Err(error)
        }
        Err(_) => {
            let _ = join.join();
            Err(MathRuntimeError::Panic(
                "egobox construction response lost".into(),
            ))
        }
    }
}
impl MathService {
    /// Construct full library state inside its permanent worker under original task admission.
    /// The finite local clock begins at submission, before waiting for CPU or a job slot.
    pub async fn prepare_surrogate<E: FidelityEvaluator<Error = ProblemError>>(
        self: &Arc<Self>,
        evaluator: E,
        options: SurrogateOptions,
        scope: pse_kernels::ExecutionScope,
        driver: &crate::CancelSource,
    ) -> Result<SurrogateHandle, MathRuntimeError> {
        if options.stack_bytes != self.policy.stack_bytes
            || options.foreign_bytes > self.policy.foreign_bytes
        {
            return Err(MathRuntimeError::Limit(
                "surrogate stack/foreign allocation policy",
            ));
        }
        let workspace = options.workspace_bytes(evaluator.correspondence())?;
        if workspace > self.policy.workspace_bytes {
            return Err(MathRuntimeError::Limit("surrogate retained workspace"));
        }
        let correspondence = Arc::new(evaluator.correspondence().clone());
        let task_scope = scope.clone();
        let task = options.key(&correspondence)?;
        scope.check().map_err(scope_error)?;
        let local = Instant::now()
            .checked_add(options.time)
            .ok_or(MathRuntimeError::Limit("surrogate submission deadline"))?;
        let deadline = scope.deadline().map_or(local, |outer| outer.min(local));
        let scope = pse_kernels::ExecutionScope::new(scope.cancellation().clone(), Some(deadline));
        let cores = options.threads;
        let control = FlightCancellation::default();
        let worker_scope = scope.clone();
        let worker_control = control.clone();
        let operation = self.job_retained_scoped(
            cores,
            workspace,
            control.clone(),
            scope.deadline(),
            move |flag| {
                if flag.load(std::sync::atomic::Ordering::Acquire) {
                    return Err(MathRuntimeError::Cancelled);
                }
                let actor = actor(evaluator, options, worker_scope, worker_control)?;
                Ok((Arc::new(actor), workspace))
            },
        );
        tokio::pin!(operation);
        let (actor, owner) = tokio::select! {result=&mut operation=>result?,()=driver.cancelled()=>{scope.cancellation().store(true,std::sync::atomic::Ordering::Release);control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        // The actor itself owns the lease so its Drop joins library/pool destruction before releasing it,
        // including when the last handle disappears during a detached active job.
        *actor
            .owner
            .lock()
            .map_err(|_| MathRuntimeError::Panic("surrogate allocation owner poisoned".into()))? =
            Some(owner.clone());
        Ok(SurrogateHandle {
            service: self.clone(),
            actor,
            owner,
            control,
            scope,
            task_scope,
            correspondence,
            cores,
            workspace,
            task,
        })
    }
    /// Advance on the same owning thread under the same task deadline and admitted CPU team.
    /// Consumers independently screen/correct proposals against the original scientific problem.
    pub async fn advance_surrogate(
        self: &Arc<Self>,
        handle: &SurrogateHandle,
        driver: &crate::CancelSource,
    ) -> ProposalResult {
        let actor = handle.actor.clone();
        let control = handle.control.clone();
        let owner = handle.owner.clone();
        let operation = self.job_retained_scoped(
            handle.cores,
            handle.workspace,
            control.clone(),
            handle.scope.deadline(),
            move |_| {
                let _owner = owner;
                let proposal = actor.proposal(true)?;
                let bytes = proposal
                    .as_ref()
                    .map_or(Ok(0), SurrogateProposal::retained_bytes)?;
                Ok((proposal, bytes))
            },
        );
        tokio::pin!(operation);
        tokio::select! {result=&mut operation=>result.map(|(proposal,owner)|proposal.map(|p|p.with_owner(owner))),()=driver.cancelled()=>{handle.scope.cancellation().store(true,std::sync::atomic::Ordering::Release);control.cancel();let _=operation.await;Err(MathRuntimeError::Cancelled)}}
    }
}

/// Library surrogate phase bound to one actual original target and retained actor.
#[derive(Clone, Debug)]
pub struct PreparedSurrogate(Arc<PreparedSurrogateData>);
#[derive(Debug)]
struct PreparedSurrogateData {
    handle: Arc<SurrogateHandle>,
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
        self.0.handle.cores
    }
    /// Actual consumed task/options/fidelity/source and original binding identity.
    pub fn key(&self) -> pse_ids::ContentHash {
        self.0.key
    }
    /// Actual source-owned support contracts; none denotes deterministic numerical accuracy.
    pub fn support(&self) -> std::collections::BTreeSet<pse_ids::ContentHash> {
        std::collections::BTreeSet::from([
            self.key(),
            self.0.handle.task,
            self.0.handle.correspondence.source,
            self.0.handle.correspondence.fidelity,
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
            let scope = pse_kernels::ExecutionScope::new(active.cancellation().clone(), deadline);
            let reply = self.0.handle.actor.phase(scope.clone())?;
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
                        &self.0.handle.correspondence,
                        self.0.handle.task,
                        pse_model::strategy::BranchPolicy::any_qualified(),
                    )?
                    .with_owner(owner.clone());
                outcome.statistical = Some(Arc::new(statistical.with_owner(owner.clone())));
                let mut evaluations = 0;
                match self.0.handle.service.screen_start_worker_observed(
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
    /// # Errors
    /// Changed actor service/source/target/inventory, finite extent or scope incompatibility.
    pub fn prepare_surrogate_phase(
        self: &Arc<Self>,
        handle: Arc<SurrogateHandle>,
        target: super::solves::PreparedSolve,
        scope: pse_kernels::ExecutionScope,
        time: std::time::Duration,
    ) -> Result<PreparedSurrogate, MathRuntimeError> {
        scope.check().map_err(ProblemError::Provider)?;
        handle.scope.check().map_err(ProblemError::Provider)?;
        if time.is_zero()
            || scope.deadline().is_none()
            || !Arc::ptr_eq(self, &handle.service)
            || !Arc::ptr_eq(scope.cancellation(), handle.task_scope.cancellation())
            || handle
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
        handle.correspondence.validate()?;
        if target.original_identity()? != handle.correspondence.original_target
            || target.original_coordinates()? != handle.correspondence.coordinates
            || target.threads() != handle.cores
        {
            return Err(ProblemError::Contract("surrogate correspondence original target/coordinates or admitted CPU team mismatch".into()).into());
        }
        let local = Instant::now()
            .checked_add(time)
            .ok_or_else(|| ProblemError::Contract("surrogate phase deadline extent".into()))?;
        let deadline = scope
            .deadline()
            .into_iter()
            .chain(handle.scope.deadline())
            .chain([local])
            .min();
        let operation = pse_kernels::ExecutionScope::new(scope.cancellation().clone(), deadline);
        let result_bytes = handle
            .correspondence
            .coordinates
            .len()
            .checked_mul(size_of::<f64>() * 2 + size_of::<pse_ids::SemanticId>())
            .and_then(|n| {
                n.checked_add(
                    handle
                        .correspondence
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
            .hash(&handle.task)
            .hash(&target.original_identity()?)
            .hash(&target.preparation_identity()?)
            .u64(time.as_secs())
            .u64(u64::from(time.subsec_nanos()));
        let key = h.finish_hash();
        let owner = self.reserve(
            "math:surrogate-phase-metadata",
            size_of::<PreparedSurrogateData>(),
        )?;
        Ok(PreparedSurrogate(Arc::new(PreparedSurrogateData {
            handle,
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
    use pse_ids::{ContentHash, SemanticId};
    use pse_math::surrogate::FidelityCorrespondence;
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
            regression: pse_math::surrogate::RegressionSpec::CONSTANT,
            correlation: pse_math::surrogate::CorrelationSpec::MATERN52,
            theta: pse_math::surrogate::ThetaTuning::Fixed(vec![0.1].into()),
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
    #[test]
    fn actual_actor_retains_non_send_library_and_same_pool_then_joins_destruction() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SurrogateHandle>();
        let (e, o, threads, destroyed) = fixture();
        let control = FlightCancellation::default();
        let task = actor(
            e,
            o,
            pse_kernels::ExecutionScope::new(Arc::new(AtomicBool::new(false)), None),
            control,
        )
        .unwrap();
        assert_eq!(task.work().unwrap().evaluations, 0);
        let first = task.proposal(true).unwrap().unwrap();
        assert_eq!(task.work().unwrap().evaluations, 3);
        let second = task.proposal(true).unwrap().unwrap();
        assert_eq!(first.task, second.task);
        let work = task.work().unwrap();
        assert_eq!(work.initializations, 1);
        assert_eq!(work.iterations, 1);
        let observed = threads.lock().unwrap();
        assert!(observed.len() > 3);
        assert!(observed.iter().all(|thread| *thread == observed[0]));
        drop(observed);
        assert!(!destroyed.load(Ordering::Acquire));
        drop(task);
        assert!(destroyed.load(Ordering::Acquire));
    }
    #[test]
    fn actor_refuses_expired_original_scope_before_evaluation_and_joins_failed_construction() {
        let (e, o, threads, destroyed) = fixture();
        let scope = pse_kernels::ExecutionScope::new(
            Arc::new(AtomicBool::new(false)),
            Some(Instant::now() - std::time::Duration::from_secs(1)),
        );
        assert!(matches!(
            actor(e, o, scope, FlightCancellation::default()),
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
        let cancel = Arc::new(AtomicBool::new(false));
        let scope = pse_kernels::ExecutionScope::new(
            cancel.clone(),
            Some(Instant::now() + std::time::Duration::from_secs(20)),
        );
        let task = actor(
            Failed(e.correspondence.clone()),
            o,
            scope.clone(),
            FlightCancellation::default(),
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
        let cancel = Arc::new(AtomicBool::new(false));
        let scope = pse_kernels::ExecutionScope::new(
            cancel.clone(),
            Some(Instant::now() + std::time::Duration::from_secs(20)),
        );
        let task = actor(
            Slow(e.correspondence.clone()),
            o,
            scope,
            FlightCancellation::default(),
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
        assert!(task.proposal(false).unwrap().is_none());
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
            Arc::new(AtomicBool::new(false)),
            Some(Instant::now() + std::time::Duration::from_secs(20)),
        );
        let handle = Arc::new(
            runtime
                .native()
                .prepare_surrogate(evaluator, options, scope.clone(), &cancel)
                .await
                .unwrap(),
        );
        let unrelated =
            pse_kernels::ExecutionScope::new(Arc::new(AtomicBool::new(false)), scope.deadline());
        assert!(
            runtime
                .native()
                .prepare_surrogate_phase(
                    handle.clone(),
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
                    handle.clone(),
                    original.clone(),
                    scope.clone(),
                    std::time::Duration::ZERO
                )
                .is_err()
        );
        let phase = runtime
            .native()
            .prepare_surrogate_phase(
                handle.clone(),
                original.clone(),
                scope.clone(),
                std::time::Duration::from_secs(10),
            )
            .unwrap();
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
                    backend: Backend::Kinsol,
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
        let result = package
            .solve_case(prepared, workflow_fixture::compiler_profile(), &cancel)
            .await
            .unwrap();
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
        let rows = trace.rows(result.run_id, 0).unwrap();
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
        drop(handle);
        drop(result);
        assert!(destroyed.load(Ordering::Acquire));
    }
}
