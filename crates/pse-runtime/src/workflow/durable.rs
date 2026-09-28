// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Durability classes and durable attempts (ADR-0114 Outcomes 12–17; Plan 22 O3–O6).
//!
//! A runtime is [`Durability::Ephemeral`] (in memory, as a library call or unit test needs;
//! it cannot publish) or [`Durability::Durable`]: every run is then an attempt registered
//! in the operational store before any effect. The class is an explicit policy of the
//! runtime, never a fallback: a durable run whose store is unreachable fails with the
//! infrastructure class, and an ephemeral run is never silently recorded.
//!
//! A durable attempt moves through the one transition table of `pse-operations`:
//! planned, queued (waiting for native admission, which queues rather than refuses),
//! running under a heartbeat lease, then completed, partial, failed or cancelled with its
//! typed termination. The durable `cancel_requested` flag is the cancellation authority;
//! the heartbeat returns it and a `LISTEN` watcher only shortens latency (T03). Its
//! progress streams to the store without an event cap, its reusable seeds are stored by
//! coordinate-compatibility stamp, and the published relations derive from those records.
use super::{RunReport, RunRequest, RunResult, WorkflowError};
use pse_backend_native::solve::{
    Compatibility, Event, Metric, ProgressTap, WarmPayload, WarmStart,
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_model::generated::enums::CandidateUse;
use pse_operations::{
    OperationsError, Store,
    attempts::{
        AttemptFilter, AttemptKind, AttemptRecord, NewAttempt, RuntimeTermination, Termination,
        TerminationCode, TransitionNote,
    },
    jobs::{Finished, JobOutcome, Requeue},
    lifecycle::AttemptState,
    solutions::{SeedVectors, Solution},
    streams::{ProgressEvent, ProgressValue, Retention},
};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicI32, Ordering},
    },
    time::Duration,
};

/// Lease, heartbeat, stream and retention settings of durable attempts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeasePolicy {
    /// How long a lease lasts without renewal. A process that stops heartbeating loses
    /// its attempts to the stale sweep after this long.
    pub lease: Duration,
    /// How often a running attempt renews its lease and reads its cancellation flag.
    pub heartbeat: Duration,
    /// How long the streams of finished attempts are kept.
    pub retention: Retention,
    /// Progress events per batched insert.
    pub batch: usize,
    /// The longest a progress event waits before its batch is written.
    pub flush: Duration,
}

impl Default for LeasePolicy {
    fn default() -> Self {
        Self {
            lease: Duration::from_secs(30),
            heartbeat: Duration::from_secs(10),
            retention: Retention {
                finished_for: Duration::from_secs(7 * 24 * 3600),
            },
            batch: 512,
            flush: Duration::from_millis(200),
        }
    }
}

/// The operational store as a runtime uses it: this process's worker identity and its
/// lease policy. Cheap to clone.
#[derive(Clone, Debug)]
pub struct Operations {
    store: Store,
    worker: Arc<str>,
    policy: LeasePolicy,
}

/// What start-up recovery did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Recovery {
    /// Jobs whose worker vanished, with the new attempt or final state of each.
    pub requeued: Vec<Requeue>,
    /// Other running attempts whose lease had expired, now stale.
    pub stale: Vec<SemanticId>,
    /// Progress events of finished attempts removed by the retention policy.
    pub pruned_events: u64,
}

impl Operations {
    /// Connect, open the store's schema (creating it on an empty store, refusing a
    /// different one), and recover: expired leases are marked stale (and their jobs
    /// requeued) and old streams are pruned.
    ///
    /// # Errors
    /// The infrastructure class when the store is unreachable; a configuration refusal
    /// ([`OperationsError::SchemaMismatch`]) when its schema is another build's
    /// (`just db-reset`).
    pub async fn connect(
        url: &str,
        worker: impl Into<String>,
        policy: LeasePolicy,
    ) -> Result<Self, WorkflowError> {
        let store = Store::connect(url).await?;
        store.open().await?;
        let operations = Self::from_store(store, worker, policy);
        operations.recover().await?;
        Ok(operations)
    }

    /// Use an already connected and opened store without recovery.
    pub fn from_store(store: Store, worker: impl Into<String>, policy: LeasePolicy) -> Self {
        Self {
            store,
            worker: Arc::from(worker.into()),
            policy,
        }
    }

    /// The default worker identity of this process: host and process id.
    pub fn process_worker(role: &str) -> String {
        let host = std::env::var("HOSTNAME")
            .ok()
            .filter(|h| !h.is_empty())
            .unwrap_or_else(|| "localhost".to_owned());
        format!("{role}:{host}:{}", std::process::id())
    }

    /// The store.
    pub const fn store(&self) -> &Store {
        &self.store
    }

    /// This process's worker identity, the owner of the leases it takes.
    pub fn worker(&self) -> &str {
        &self.worker
    }

    /// The lease policy.
    pub const fn policy(&self) -> LeasePolicy {
        self.policy
    }

    /// The start-up stale sweep: requeue every job whose lease expired, mark every other
    /// expired running attempt stale, and apply stream retention.
    ///
    /// # Errors
    /// Store failures.
    pub async fn recover(&self) -> Result<Recovery, WorkflowError> {
        const BATCH: i64 = 256;
        let mut recovery = Recovery::default();
        loop {
            let requeued = self
                .store
                .jobs()
                .requeue_expired(BATCH, pse_operations::mint_id)
                .await?;
            let done = requeued.len() < BATCH as usize;
            recovery.requeued.extend(requeued);
            if done {
                break;
            }
        }
        loop {
            let stale = self.store.attempts().sweep_stale(BATCH).await?;
            let done = stale.len() < BATCH as usize;
            recovery.stale.extend(stale);
            if done {
                break;
            }
        }
        recovery.pruned_events = self
            .store
            .streams()
            .apply_retention(self.policy.retention)
            .await?;
        Ok(recovery)
    }

    /// Durable attempts newest first; they survive a restart of the process that ran them.
    ///
    /// # Errors
    /// Store failures.
    pub async fn runs(&self, filter: &AttemptFilter) -> Result<Vec<AttemptRecord>, WorkflowError> {
        Ok(self.store.attempts().list(filter).await?)
    }
}

impl super::Runtime {
    /// The durable attempts of this runtime's operational store, newest first, as the
    /// registry relation `runtime.operational_attempts`. They survive a restart of the
    /// process that ran them (ADR-0114 Outcome 16).
    ///
    /// # Errors
    /// An ephemeral runtime, which records no attempts; store failures; a stored value
    /// outside the registry contract.
    pub async fn runs(
        &self,
        filter: &AttemptFilter,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        use pse_relations::generated::runtime::operational_attempts as attempts;
        let Durability::Durable(operations) = &self.durability else {
            return Err(super::contract(
                "the run listing needs a durable runtime (ADR-0114 Outcome 16)",
            ));
        };
        let records = operations.runs(filter).await?;
        let mut rows = attempts::Builder::with_registry(&self.registry, records.len())
            .map_err(super::relation)?;
        for record in &records {
            rows.push(record.row()).map_err(super::relation)?;
        }
        rows.finish().map_err(super::relation)
    }
}

/// How a runtime keeps its runs (ADR-0114 Outcome 16): an explicit policy.
#[derive(Clone, Debug, Default)]
pub enum Durability {
    /// In memory only: nothing survives the process and nothing may be published.
    #[default]
    Ephemeral,
    /// Every run is an attempt registered in the operational store.
    Durable(Operations),
}

/// What a finished run recorded durably.
#[derive(Clone, Debug)]
pub enum RunDurability {
    /// Nothing was recorded; the run cannot be published.
    Ephemeral,
    /// The run's attempt and what the store holds for it.
    Durable(Box<DurableRecord>),
}

/// The durable records of one finished attempt.
#[derive(Clone, Debug)]
pub struct DurableRecord {
    /// The attempt identity, minted before any effect.
    pub attempt_id: SemanticId,
    /// The attempt as stored after its terminal transition, or why it was not recorded.
    pub attempt: Result<AttemptRecord, Arc<WorkflowError>>,
    /// The complete progress stream as stored: the snapshot publication derives from.
    pub progress: Result<Vec<ProgressEvent>, Arc<WorkflowError>>,
    /// Seeds stored from accepted steps, by step.
    pub solutions: Vec<(usize, SemanticId)>,
}

/// Cancels the supervised run; shared with the heartbeat.
pub(crate) type Canceller = Arc<dyn Fn() + Send + Sync>;

/// The typed end of an attempt.
#[derive(Clone, Debug)]
pub(super) struct Outcome {
    state: AttemptState,
    termination: Termination,
    reason: String,
    /// A retry could succeed: the failure was infrastructure, not the science.
    retryable: bool,
}

/// Where a claimed attempt ends: through its job, under the job's retry policy.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Claim {
    pub(crate) job: SemanticId,
    pub(crate) attempt: SemanticId,
}

/// One durable attempt of a run: registration, lease, stream and termination.
#[derive(Debug)]
pub(crate) struct DurableAttempt {
    operations: Operations,
    attempt: SemanticId,
    claim: Option<Claim>,
    stream: Streamer,
    heartbeat: Option<Heartbeat>,
}

impl DurableAttempt {
    /// A new attempt for a run of this process. Nothing is stored until [`Self::register`].
    pub(super) fn new(operations: &Operations) -> Self {
        let attempt = pse_operations::mint_id();
        Self {
            stream: Streamer::spawn(operations, attempt),
            operations: operations.clone(),
            attempt,
            claim: None,
            heartbeat: None,
        }
    }

    /// The attempt a worker claimed with its job: already running under the worker's lease.
    pub(crate) fn claimed(operations: &Operations, claim: Claim) -> Self {
        Self {
            stream: Streamer::spawn(operations, claim.attempt),
            operations: operations.clone(),
            attempt: claim.attempt,
            claim: Some(claim),
            heartbeat: None,
        }
    }

    pub(super) const fn attempt_id(&self) -> SemanticId {
        self.attempt
    }

    /// The durable progress tap; every event reaches the store, whatever the in-memory cap.
    pub(super) fn tap(&self) -> Arc<dyn ProgressTap> {
        self.stream.tap.clone()
    }

    /// The step later progress events belong to.
    pub(super) fn set_step(&self, step: usize) {
        self.stream
            .tap
            .step
            .store(i32::try_from(step).unwrap_or(i32::MAX), Ordering::Release);
    }

    /// Register a new attempt before any effect: planned, then queued for admission. A
    /// claimed attempt is already registered and running.
    pub(super) async fn register(
        &self,
        run_id: SemanticId,
        request: &RunRequest,
    ) -> Result<(), WorkflowError> {
        if self.claim.is_some() {
            return Ok(());
        }
        let (kind, request_identity, preparation_identity) = identities(request)?;
        let attempts = self.operations.store.attempts();
        attempts
            .create(
                &NewAttempt {
                    attempt_id: self.attempt,
                    run_id,
                    kind,
                    request_identity,
                    preparation_identity,
                    parent_attempt: None,
                },
                Some(self.operations.worker()),
            )
            .await?;
        attempts
            .transition(
                self.attempt,
                AttemptState::Queued,
                &TransitionNote::by(self.operations.worker()).because("awaiting native admission"),
            )
            .await?;
        Ok(())
    }

    /// Enter running under this process's lease (a claimed attempt already is), then renew
    /// the lease and watch the durable cancellation flag until the attempt finishes.
    pub(crate) async fn start(&mut self, cancel: Canceller) -> Result<(), WorkflowError> {
        if self.heartbeat.is_some() {
            // A worker started its claimed attempt's lease before preparing the job.
            return Ok(());
        }
        if self.claim.is_none() {
            self.operations
                .store
                .attempts()
                .start(
                    self.attempt,
                    self.operations.worker(),
                    self.operations.policy.lease,
                )
                .await?;
        }
        self.heartbeat = Some(Heartbeat::spawn(&self.operations, self.attempt, cancel));
        Ok(())
    }

    /// Record the end of the attempt: flush its stream, store its reusable seeds, stop the
    /// lease and apply the terminal transition (through the job for a claimed attempt),
    /// then read back what the store holds.
    pub(super) async fn finish(mut self, result: &RunResult, cancelled: bool) -> DurableRecord {
        let lost = self.heartbeat.as_ref().is_some_and(Heartbeat::lost);
        let flushed = self.stream.finish().await;
        let solutions = match (&flushed, self.heartbeat.is_some()) {
            (Ok(()), true) => store_seeds(&self.operations, self.attempt, result).await,
            _ => Ok(Vec::new()),
        };
        if let Some(heartbeat) = self.heartbeat.take() {
            heartbeat.stop().await;
        }
        let mut outcome = classify(result, cancelled || lost);
        if let Err(error) = &flushed {
            outcome = infrastructure(error);
        }
        let solutions = solutions.unwrap_or_else(|error| {
            outcome = infrastructure(&error);
            Vec::new()
        });
        let attempt = self.terminate(&outcome).await.map_err(Arc::new);
        let progress = self
            .operations
            .store
            .streams()
            .snapshot(self.attempt)
            .await
            .map_err(|e| Arc::new(e.into()));
        DurableRecord {
            attempt_id: self.attempt,
            attempt,
            progress,
            solutions,
        }
    }

    /// Record an attempt that never ran because preparation, registration or admission
    /// failed: a claimed attempt ends through its job; an unstarted one is cancelled.
    pub(crate) async fn abandon(mut self, error: &WorkflowError) -> DurableRecord {
        let _ = self.stream.finish().await;
        if let Some(heartbeat) = self.heartbeat.take() {
            heartbeat.stop().await;
        }
        let outcome = failure(error, false);
        let attempt = match self.claim {
            Some(_) => self.terminate(&outcome).await,
            None => self.cancel_unstarted(&outcome).await,
        }
        .map_err(Arc::new);
        DurableRecord {
            attempt_id: self.attempt,
            attempt,
            progress: Ok(Vec::new()),
            solutions: Vec::new(),
        }
    }

    async fn cancel_unstarted(&self, outcome: &Outcome) -> Result<AttemptRecord, WorkflowError> {
        let attempts = self.operations.store.attempts();
        match attempts.get(self.attempt).await {
            Ok(record) if matches!(record.state, AttemptState::Planned | AttemptState::Queued) => {
                let note = TransitionNote::by(self.operations.worker())
                    .because(outcome.reason.clone())
                    .terminated(outcome.termination.clone());
                Ok(attempts
                    .transition(self.attempt, AttemptState::Cancelled, &note)
                    .await?)
            }
            Ok(record) => Ok(record),
            Err(error) => Err(error.into()),
        }
    }

    async fn terminate(&self, outcome: &Outcome) -> Result<AttemptRecord, WorkflowError> {
        let note = TransitionNote::by(self.operations.worker())
            .because(outcome.reason.clone())
            .terminated(outcome.termination.clone());
        let store = &self.operations.store;
        match self.claim {
            None => Ok(store
                .attempts()
                .transition(self.attempt, outcome.state, &note)
                .await?),
            Some(claim) => {
                let retry = (outcome.state == AttemptState::Failed && outcome.retryable)
                    .then(pse_operations::mint_id);
                let finished = store
                    .jobs()
                    .finish(
                        claim.job,
                        self.operations.worker(),
                        &JobOutcome {
                            state: outcome.state,
                            note,
                            retry_as: retry,
                        },
                    )
                    .await?;
                if let Finished::Requeued { attempt_id, .. } = finished {
                    tracing::info!(job = %claim.job, next = %attempt_id, "failed try requeued");
                }
                Ok(store.attempts().get(self.attempt).await?)
            }
        }
    }
}

/// The attempt kind and the identities it is registered with: the request identity of
/// every step (blueprint §5.1) and, for one prepared step, its preparation identity.
fn identities(
    request: &RunRequest,
) -> Result<(AttemptKind, ContentHash, Option<ContentHash>), WorkflowError> {
    let math = |e: pse_backend_native::ProblemError| {
        WorkflowError::Math(crate::math::MathRuntimeError::from(e))
    };
    Ok(match request {
        RunRequest::Modeling(steps) => {
            let mut h = FramedHasher::new("pse.durable.modeling_request.v1");
            h.u64(steps.len() as u64);
            for step in steps {
                h.hash(&step.solve.request_identity().map_err(math)?);
            }
            let preparation = match steps.as_slice() {
                [one] => Some(one.solve.preparation_identity().map_err(math)?),
                _ => None,
            };
            (AttemptKind::Modeling, h.finish_hash(), preparation)
        }
        RunRequest::Simulation(s) => (AttemptKind::Simulation, s.identity(), Some(s.identity())),
        RunRequest::Fit(f) => (AttemptKind::Fit, f.problem.key, Some(f.problem.key)),
    })
}

const fn termination(code: TerminationCode, detail: serde_json::Value) -> Termination {
    Termination {
        code,
        detail: Some(detail),
    }
}

/// A run-level failure: cancelled when cancellation stopped it, otherwise failed with the
/// error's diagnostic code; retryable only for infrastructure.
fn failure(error: &WorkflowError, cancelled: bool) -> Outcome {
    use crate::math::MathRuntimeError as M;
    let is_cancel = matches!(error, WorkflowError::Math(M::Cancelled));
    // The violated named contract when the error names one; its diagnostic code otherwise.
    let diagnostic = error.boundary_diagnostic();
    let code = if diagnostic.rule == "workflow.unclassified" {
        // The registry's dotted spelling of the §23.2 code (miette renders `a::b`).
        miette::Diagnostic::code(error)
            .map_or_else(|| diagnostic.rule, |c| c.to_string().replace("::", "."))
    } else {
        diagnostic.rule
    };
    let retryable = match error {
        WorkflowError::Operations(e) => e.is_retryable(),
        WorkflowError::Math(M::Infrastructure(_)) => true,
        _ => false,
    };
    let detail = serde_json::json!({ "error": error.to_string() });
    if cancelled || is_cancel {
        Outcome {
            state: AttemptState::Cancelled,
            termination: termination(
                TerminationCode::Runtime(RuntimeTermination::Cancelled),
                detail,
            ),
            reason: "cancellation requested".to_owned(),
            retryable: false,
        }
    } else {
        Outcome {
            state: AttemptState::Failed,
            termination: termination(TerminationCode::Rule(code), detail),
            reason: error.to_string(),
            retryable,
        }
    }
}

fn infrastructure(error: &WorkflowError) -> Outcome {
    Outcome {
        state: AttemptState::Failed,
        termination: termination(
            TerminationCode::Runtime(RuntimeTermination::Infrastructure),
            serde_json::json!({ "error": error.to_string() }),
        ),
        reason: error.to_string(),
        retryable: true,
    }
}

/// The typed end of a joined run: cancelled when cancellation stopped it; completed when
/// every requested candidate is a result; partial when some are; failed otherwise. The
/// termination code is the last native termination (or the run error's diagnostic code).
fn classify(result: &RunResult, cancelled: bool) -> Outcome {
    let report = match result.report() {
        Ok(report) => report,
        Err(error) => return failure(error, cancelled),
    };
    let uses: Vec<&str> = result
        .assessments()
        .iter()
        .map(|a| a.usability.as_str())
        .collect();
    let usable = |a: &&pse_model::generated::runtime::candidate_assessments::Row| {
        matches!(
            a.usability,
            CandidateUse::Usable | CandidateUse::QualifiedUnclosed
        )
    };
    let any = result.assessments().iter().any(|a| usable(&a));
    let code = match (report, result.completion()) {
        (RunReport::Modeling(_), Ok(c)) => c
            .solves
            .iter()
            .rev()
            .find_map(|s| s.termination.map(TerminationCode::Native))
            .unwrap_or_else(|| {
                c.solves.last().map_or(
                    TerminationCode::Runtime(RuntimeTermination::Unattempted),
                    |s| TerminationCode::RunState(s.state),
                )
            }),
        (RunReport::Simulation(t), _) => TerminationCode::Trajectory(t.report.termination),
        (RunReport::Fit(_), Ok(c)) => c
            .computation
            .as_ref()
            .and_then(|r| r.termination)
            .map_or(
                TerminationCode::Runtime(RuntimeTermination::ConstantEvaluation),
                TerminationCode::Native,
            ),
        (_, Err(_)) => TerminationCode::Runtime(RuntimeTermination::Unassessed),
    };
    let detail = serde_json::json!({ "usable": result.usable(), "candidate_use": uses });
    let (state, reason) = if cancelled {
        (AttemptState::Cancelled, "cancellation requested")
    } else if result.usable() {
        (
            AttemptState::Completed,
            "every requested candidate is a result",
        )
    } else if any {
        (
            AttemptState::Partial,
            "some requested candidates are results",
        )
    } else {
        (AttemptState::Failed, "no requested candidate is a result")
    };
    Outcome {
        state,
        termination: termination(
            if cancelled {
                TerminationCode::Runtime(RuntimeTermination::Cancelled)
            } else {
                code
            },
            detail,
        ),
        reason: reason.to_owned(),
        retryable: false,
    }
}

// ------------------------------------------------------------------ heartbeat --

/// Renews the lease and watches the durable cancellation flag of one running attempt.
#[derive(Debug)]
struct Heartbeat {
    stop: tokio::sync::oneshot::Sender<()>,
    task: tokio::task::JoinHandle<()>,
    lost: Arc<AtomicBool>,
}

impl Heartbeat {
    fn spawn(operations: &Operations, attempt: SemanticId, cancel: Canceller) -> Self {
        let (stop, mut stopped) = tokio::sync::oneshot::channel::<()>();
        let lost = Arc::new(AtomicBool::new(false));
        let lease_lost = lost.clone();
        let operations = operations.clone();
        let task = tokio::spawn(async move {
            // LISTEN first, then read the authority: a request made before the watcher
            // existed is still observed (the listen-then-inspect rule).
            let mut watcher = operations.store.watch_cancellation(attempt).await.ok();
            let mut ticks = tokio::time::interval(operations.policy.heartbeat);
            loop {
                tokio::select! {
                    _ = &mut stopped => break,
                    _ = ticks.tick() => {
                        match operations
                            .store
                            .attempts()
                            .heartbeat(attempt, operations.worker(), operations.policy.lease)
                            .await
                        {
                            Ok(ack) if ack.cancel_requested => cancel(),
                            Ok(_) => {}
                            Err(OperationsError::LeaseLost { .. }) => {
                                lease_lost.store(true, Ordering::Release);
                                cancel();
                                break;
                            }
                            Err(error) => {
                                tracing::warn!(%attempt, %error, "lease renewal failed; retrying");
                            }
                        }
                    }
                    requested = async {
                        match watcher.as_mut() {
                            Some(w) => w.requested().await,
                            None => std::future::pending().await,
                        }
                    } => {
                        if requested.is_ok() {
                            cancel();
                        }
                        // Observed or failed: the heartbeat keeps reading the flag.
                        watcher = None;
                    }
                }
            }
        });
        Self { stop, task, lost }
    }

    fn lost(&self) -> bool {
        self.lost.load(Ordering::Acquire)
    }

    async fn stop(self) {
        let _ = self.stop.send(());
        let _ = self.task.await;
    }
}

// --------------------------------------------------------------------- stream --

/// One observed native event and the step it belongs to.
type Observed = (i32, chrono::DateTime<chrono::Utc>, Event);

/// The progress tap of a durable attempt; never blocks the native thread.
#[derive(Debug)]
struct Tap {
    step: AtomicI32,
    sender: Mutex<Option<tokio::sync::mpsc::UnboundedSender<Observed>>>,
}

impl ProgressTap for Tap {
    fn observe(&self, event: &Event) {
        let step = self.step.load(Ordering::Acquire);
        if let Ok(sender) = self.sender.lock()
            && let Some(sender) = sender.as_ref()
        {
            let _ = sender.send((step, chrono::Utc::now(), event.clone()));
        }
    }
}

/// Writes a durable attempt's progress in batched inserts, numbering events in order.
#[derive(Debug)]
struct Streamer {
    tap: Arc<Tap>,
    task: Option<tokio::task::JoinHandle<Result<(), WorkflowError>>>,
}

fn value(metric: &Metric) -> ProgressValue {
    match metric {
        Metric::Real(v) => ProgressValue::real(*v),
        Metric::Integer(v) => ProgressValue::Integer(*v),
        Metric::Text(v) => ProgressValue::Text(v.clone()),
        Metric::Bool(v) => ProgressValue::Boolean(*v),
        Metric::Unavailable(reason) => ProgressValue::Unavailable(*reason),
    }
}

impl Streamer {
    fn spawn(operations: &Operations, attempt: SemanticId) -> Self {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Observed>();
        let tap = Arc::new(Tap {
            step: AtomicI32::new(0),
            sender: Mutex::new(Some(sender)),
        });
        let operations = operations.clone();
        let task = tokio::spawn(async move {
            let policy = operations.policy;
            let mut seq = 0_i64;
            let mut batch: Vec<ProgressEvent> = Vec::with_capacity(policy.batch);
            let mut open = true;
            while open {
                // Wait for the first event, then gather until the batch is full, the flush
                // interval passed or the stream closed.
                match receiver.recv().await {
                    Some(observed) => batch.push(event(&mut seq, observed)),
                    None => open = false,
                }
                let deadline = tokio::time::Instant::now() + policy.flush;
                while open && batch.len() < policy.batch {
                    match tokio::time::timeout_at(deadline, receiver.recv()).await {
                        Ok(Some(observed)) => batch.push(event(&mut seq, observed)),
                        Ok(None) => open = false,
                        Err(_) => break,
                    }
                }
                if !batch.is_empty() {
                    append(&operations, attempt, &batch).await?;
                    batch.clear();
                }
            }
            Ok(())
        });
        Self {
            tap,
            task: Some(task),
        }
    }

    /// Close the stream and wait until every observed event is stored. Events observed
    /// after this are not part of the attempt.
    async fn finish(&mut self) -> Result<(), WorkflowError> {
        if let Ok(mut sender) = self.tap.sender.lock() {
            sender.take();
        }
        match self.task.take() {
            Some(task) => task.await.map_err(|e| {
                WorkflowError::Math(crate::math::MathRuntimeError::Infrastructure(format!(
                    "progress stream task: {e}"
                )))
            })?,
            None => Ok(()),
        }
    }
}

fn event(seq: &mut i64, (step, at, event): Observed) -> ProgressEvent {
    let this = *seq;
    *seq += 1;
    ProgressEvent {
        seq: this,
        step,
        at,
        elapsed_seconds: event.elapsed.as_secs_f64(),
        phase: event.phase,
        values: event
            .values
            .iter()
            .map(|(name, metric)| (name.clone(), value(metric)))
            .collect(),
    }
}

/// Append one batch, retrying transient store failures a bounded number of times.
async fn append(
    operations: &Operations,
    attempt: SemanticId,
    batch: &[ProgressEvent],
) -> Result<(), WorkflowError> {
    let mut delay = Duration::from_millis(50);
    for _ in 0..5 {
        match operations
            .store
            .streams()
            .append_progress(attempt, batch)
            .await
        {
            Ok(_) => return Ok(()),
            Err(error) if error.is_retryable() => {
                tokio::time::sleep(delay).await;
                delay = delay.saturating_mul(2);
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(operations
        .store
        .streams()
        .append_progress(attempt, batch)
        .await
        .map(|_| ())?)
}

// ---------------------------------------------------------------------- seeds --

/// The stored vectors of a portable seed: source-coordinate vectors and, for an NLP seed,
/// the producer's final barrier (authored objective units). An SQP working set is keyed
/// by its native transformation, so it stays with its worker and is not stored.
pub(super) fn seed_vectors(payload: &WarmPayload) -> SeedVectors {
    match payload {
        WarmPayload::Root(primal) => SeedVectors::Root {
            primal: primal.clone(),
        },
        WarmPayload::Nlp {
            primal,
            bounds,
            rows,
            barrier,
            working: _,
        } => SeedVectors::Nlp {
            primal: primal.clone(),
            bounds: bounds.clone(),
            rows: rows.clone(),
            barrier: *barrier,
        },
        WarmPayload::Highs {
            primal,
            dual,
            basis,
        } => SeedVectors::Highs {
            primal: primal.clone(),
            dual: dual.clone(),
            basis: basis.as_ref().map(|b| (b.columns.clone(), b.rows.clone())),
        },
    }
}

/// The owned warm start a stored solution describes, with its original compatibility
/// stamps. Whether it may seed a given preparation is decided by `WarmStart::validate`.
pub(super) fn warm_start(solution: &Solution) -> WarmStart {
    let payload = match &solution.vectors {
        SeedVectors::Root { primal } => WarmPayload::Root(primal.clone()),
        SeedVectors::Nlp {
            primal,
            bounds,
            rows,
            barrier,
        } => WarmPayload::Nlp {
            primal: primal.clone(),
            bounds: bounds.clone(),
            rows: rows.clone(),
            barrier: *barrier,
            working: None,
        },
        SeedVectors::Highs {
            primal,
            dual,
            basis,
        } => WarmPayload::Highs {
            primal: primal.clone(),
            dual: dual.clone(),
            basis: basis
                .as_ref()
                .map(|(columns, rows)| pse_backend_native::solve::Basis {
                    columns: columns.clone(),
                    rows: rows.clone(),
                }),
        },
    };
    WarmStart {
        origin: None,
        compatibility: Compatibility {
            layout: solution.compatibility_stamp,
            profile: solution.profile_stamp,
            data: solution.data_stamp,
            backend: solution.backend,
        },
        payload,
    }
}

/// Store the output seed of every accepted step whose candidate may be used (ADR-0106),
/// keyed by its layout stamp and seed preparation identity.
async fn store_seeds(
    operations: &Operations,
    attempt: SemanticId,
    result: &RunResult,
) -> Result<Vec<(usize, SemanticId)>, WorkflowError> {
    let (Ok(RunReport::Modeling(steps)), RunRequest::Modeling(requests)) =
        (result.report(), result.request())
    else {
        return Ok(Vec::new());
    };
    let mut stored = Vec::new();
    for (index, (step, request)) in steps.iter().zip(requests).enumerate() {
        let crate::math::solves::Outcome::Native(native) = &step.outcome else {
            continue;
        };
        if !(step.accepted && step.outcome.candidate_use().permits_use()) {
            continue;
        }
        let (Some(seed), Some(preparation)) = (
            native.warm_start.as_ref(),
            request.solve.seed_preparation_identity(),
        ) else {
            continue;
        };
        let vectors = seed_vectors(&seed.payload);
        let solution_id = pse_operations::mint_id();
        operations
            .store
            .solutions()
            .put(&Solution {
                solution_id,
                compatibility_stamp: seed.compatibility.layout,
                preparation_identity: preparation,
                backend: seed.compatibility.backend,
                profile_stamp: seed.compatibility.profile,
                data_stamp: seed.compatibility.data,
                vectors,
                created_by: Some(attempt),
            })
            .await?;
        stored.push((index, solution_id));
    }
    Ok(stored)
}
