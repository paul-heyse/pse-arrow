// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Waiter cancellation never takes native ownership away from the existing supervisor.
//!
//! Every run is supervised under its runtime's durability class (ADR-0114 Outcome 16). An
//! ephemeral run is admitted at once or refused, and keeps its record in memory. A durable
//! run registers its attempt before any effect, queues for native admission, runs under a
//! heartbeat lease whose durable cancellation flag stops it, streams its progress, and
//! records its typed termination before its result is published to waiters.
use super::{
    Runtime, WorkflowError, contract,
    durable::{Canceller, Durability, DurableAttempt, RunDurability},
};
use crate::math::{MathRuntimeError, Submission};
use pse_backend_native::solve::{Event, Progress};
use pse_columnar::flight::FlightCancellation;
use pse_ids::SemanticId;
use pse_model::generated::identities::RunId;
use std::{collections::BTreeMap, sync::Arc};

/// Existing native-submit ownership can be held at actual family worker entry by
/// isolated unit controls. No release-build observer or execution policy is added.
#[cfg(test)]
type FamilyWorkerEntry = Arc<dyn Fn(&std::sync::atomic::AtomicBool) + Send + Sync>;
#[cfg(test)]
tokio::task_local! { static FAMILY_WORKER_ENTRY: FamilyWorkerEntry; }
#[cfg(test)]
pub(in crate::workflow) async fn observed_family_worker_entry<T>(
    entry: FamilyWorkerEntry,
    work: impl Future<Output = T>,
) -> T {
    FAMILY_WORKER_ENTRY.scope(entry, work).await
}
#[cfg(test)]
pub(in crate::workflow) fn family_worker_entry() -> Option<FamilyWorkerEntry> {
    FAMILY_WORKER_ENTRY.try_with(Arc::clone).ok()
}

/// Public cancellation lease. Dropping the last public handle requests cancellation;
/// the supervisor retains the actual native handle until its join completes.
#[derive(Debug)]
struct Lease(
    FlightCancellation,
    Option<crate::CancelSource>,
    Option<pse_diagnostics::observation::ObservationContext>,
);
impl Lease {
    fn cancel(&self) {
        if let Some(context) = &self.2
            && context.is_active()
        {
            context.boolean("run.stop_requested", true);
        }
        self.0.cancel();
        if let Some(checks) = &self.1 {
            checks.cancel();
        }
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.cancel();
    }
}
/// A repeatably awaitable, cancellable view of one native job.
#[derive(Clone, Debug)]
pub struct RunHandle {
    observation: Option<pse_diagnostics::observation::OperationReader>,
    lease: Arc<Lease>,
    receiver: tokio::sync::watch::Receiver<Option<Arc<RunResult>>>,
    progress: Arc<Progress>,
    run_id: RunId,
    attempt_id: Option<pse_model::generated::identities::AttemptId>,
    canonical_keys: Option<(String, String)>,
}
impl RunHandle {
    /// Passive current/cumulative/recent operation reader; dropping it never cancels this run.
    pub fn observations(&self) -> Option<pse_diagnostics::observation::OperationReader> {
        self.observation.clone()
    }

    /// Request stop; result ownership remains live until native teardown and join.
    pub fn cancel(&self) {
        self.lease.cancel();
    }
    /// The same terminal result remains available after cancellation of an earlier waiter.
    pub async fn wait(&self) -> Result<Arc<RunResult>, WorkflowError> {
        let mut receiver = self.receiver.clone();
        loop {
            if let Some(result) = receiver.borrow_and_update().clone() {
                return Ok(result);
            }
            receiver
                .changed()
                .await
                .map_err(|_| contract("lost public run supervisor"))?;
        }
    }
    /// Nonblocking immutable completion snapshot.
    pub fn result(&self) -> Option<Arc<RunResult>> {
        self.receiver.borrow().clone()
    }
    /// Bounded native events and actual dropped-event count. A durable run's complete
    /// stream is in the operational store.
    pub fn progress(&self) -> (Vec<Event>, u64) {
        self.progress.snapshot()
    }
    /// The run identity, known before any effect: minted when the run starts, or the run
    /// a claimed job's attempts share.
    pub const fn run_id(&self) -> RunId {
        self.run_id
    }
    /// The durable attempt of this run, minted before any effect; `None` when ephemeral.
    pub const fn attempt_id(&self) -> Option<pse_model::generated::identities::AttemptId> {
        self.attempt_id
    }
    /// Exact canonical lookup identity; independent of scientific lineage identities.
    pub fn canonical_run_key(&self) -> Option<&str> {
        self.canonical_keys.as_ref().map(|keys| keys.0.as_str())
    }
    /// Exact native attempt lookup identity, known before dispatch.
    pub fn canonical_attempt_key(&self) -> Option<&str> {
        self.canonical_keys.as_ref().map(|keys| keys.1.as_str())
    }
    /// The handle of a run supervised as a staged sequence: cancelling it cancels `checks`,
    /// which stops the step in progress and joins its native teardown.
    pub(super) fn staged(
        checks: crate::CancelSource,
        receiver: tokio::sync::watch::Receiver<Option<Arc<RunResult>>>,
        progress: Arc<Progress>,
        run_id: RunId,
        attempt_id: Option<pse_model::generated::identities::AttemptId>,
        canonical_keys: Option<(String, String)>,
    ) -> Self {
        Self {
            observation: observations_for(run_id, canonical_keys.as_ref()),
            lease: Arc::new(Lease(
                FlightCancellation::default(),
                Some(checks),
                pse_diagnostics::observation::current(),
            )),
            receiver,
            progress,
            run_id,
            attempt_id,
            canonical_keys,
        }
    }
}
fn observations_for(
    run: RunId,
    canonical: Option<&(String, String)>,
) -> Option<pse_diagnostics::observation::OperationReader> {
    pse_diagnostics::observation::current().map(|context| {
        context.source_ref("run.identity", run.as_id().to_string().as_bytes());
        if let Some((run, attempt)) = canonical {
            context.source_ref("canonical.run", run.as_bytes());
            context.source_ref("canonical.attempt", attempt.as_bytes());
        }
        context.reader()
    })
}

/// Which stored seed a preparation starts from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoredStart {
    /// The newest seed stored for the preparation's coordinates and backend.
    Latest,
    /// This stored solution.
    Solution(pse_model::generated::identities::SolutionId),
}
/// Mathematical report variants share one joined public job lifecycle.
#[derive(Debug)]
pub enum RunReport {
    /// Explicit compact horizon outcome; full modeling history was not requested.
    Horizon(Arc<super::HorizonOutcome>),
    /// Authored steady or simultaneous solve with original-model qualification.
    Modeling(Vec<super::ModelingResult>),
    /// A completed or partial native integration.
    Simulation(Box<super::ModelingTrajectory>),
    /// Native steady/transient parameter fitting.
    Fit(Box<super::FitReport>),
    /// Joined shooting NLP plus original physical trajectory and composed acceptance.
    #[cfg(feature = "solver-diffsol")]
    Shooting(Box<super::ShootingReport>),
}
/// Immutable request representation, with no mutable native objects.
#[derive(Clone, Debug)]
pub enum RunRequest {
    /// Immutable compact horizon identity and original source owners.
    Horizon(super::HorizonRequest),
    /// One immutable authored algebraic case.
    Modeling(Vec<super::ModelingSolvePreparation>),
    /// One already-prepared physical simulation.
    Simulation(Box<super::ModelingSimulation>),
    /// Compiled shared-parameter experiments.
    Fit(Box<super::PreparedFit>),
    /// Admitted shooting problem and the exact requested initial coordinate vector.
    #[cfg(feature = "solver-diffsol")]
    Shooting {
        /// Bound scientific problem admitted before dispatch.
        problem: Arc<super::ShootingProblem>,
        /// Caller-provided original coordinates, or the prepared defaults.
        initial: Option<Vec<f64>>,
    },
}
/// Immutable joined outcome. Table encoding and retention never invoke a solver again.
#[derive(Debug)]
pub struct RunResult {
    pub(super) modeling_continue_independent: bool,
    pub(super) attempt_summary: std::sync::OnceLock<Arc<super::AttemptSummary>>,
    pub(super) modeling_origin: usize,
    pub(super) original_completion_captured: bool,
    pub(super) horizon_output: Option<Arc<super::HorizonOutcome>>,
    /// Original horizon input, distinct from the materialized full-output solve prefix.
    pub(super) original_horizon_request: Option<RunRequest>,
    #[cfg(feature = "solver-diffsol")]
    pub(super) horizon_owners: Vec<Arc<pse_columnar::AllocationLease>>,
    pub(super) horizon_tightenings: Option<Vec<pse_modeling::DomainTightening>>,
    pub(super) horizon_unmerged: Option<Box<RunResult>>,
    /// The run: an execution identity, not mathematical content identity. A retried
    /// job's tries share it; each try is its own attempt.
    pub run_id: RunId,
    pub(crate) runtime: Runtime,
    pub(crate) request: RunRequest,
    pub(crate) _owner: Option<Arc<pse_columnar::AllocationLease>>,
    pub(crate) report: Result<RunReport, Arc<WorkflowError>>,
    /// Completed modeling steps retained when a later step fails.
    pub(super) modeling_failure: Option<ModelingFailure>,
    pub(crate) assessments: Vec<pse_relations::generated::runtime::candidate_assessments::Row>,
    pub(crate) completion: Result<super::completion::Completion, Arc<WorkflowError>>,
    pub(super) encodings: super::result_export::EncodingState,
    pub(super) result_validation: Arc<pse_relations::validate::ValidationContext>,
    pub(super) projection_owner: Option<super::result_export::ProjectionOwner>,
    /// What the run recorded durably; ephemeral runs record nothing.
    pub(crate) durability: RunDurability,
    /// What a rolling horizon did at each sample (Plan 22 Y5c); `None` for any other run.
    pub(crate) horizon: Option<Arc<super::HorizonReport>>,
}
#[derive(Debug)]
pub(super) struct ModelingFailure {
    pub(super) completed: Vec<super::ModelingResult>,
    pub(super) step: Option<usize>,
}
impl RunResult {
    /// A joined result before completion capture and durable recording.
    pub(super) fn joined(
        run_id: RunId,
        runtime: Runtime,
        request: RunRequest,
        owner: Option<Arc<pse_columnar::AllocationLease>>,
        report: Result<RunReport, Arc<WorkflowError>>,
    ) -> Self {
        let result_validation = Arc::new(runtime.validation_context().new_scope());
        let projection_owner =
            super::result_export::ProjectionOwner::new(&runtime, result_validation.clone());
        Self {
            modeling_continue_independent: false,
            attempt_summary: Default::default(),
            result_validation,
            projection_owner,
            modeling_origin: 0,
            original_completion_captured: false,
            horizon_output: None,
            original_horizon_request: None,
            #[cfg(feature = "solver-diffsol")]
            horizon_owners: Vec::new(),
            horizon_tightenings: None,
            horizon_unmerged: None,
            run_id,
            runtime,
            request,
            _owner: owner,
            report,
            modeling_failure: None,
            assessments: vec![],
            completion: Err(Arc::new(contract("completion has not been captured"))),
            encodings: super::result_export::EncodingState::default(),
            durability: RunDurability::Ephemeral,
            horizon: None,
        }
    }
    fn with_modeling_policy(mut self, continue_independent: bool) -> Self {
        self.modeling_continue_independent = continue_independent;
        self
    }
    fn reset_result_projection(&mut self) {
        self.attempt_summary.take();
        self.result_validation = Arc::new(self.runtime.validation_context().new_scope());
        self.projection_owner = super::result_export::ProjectionOwner::new(
            &self.runtime,
            self.result_validation.clone(),
        );
    }
    /// Final completion-owned scientific assessments; exporting tables cannot change them.
    pub fn assessments(&self) -> &[pse_relations::generated::runtime::candidate_assessments::Row] {
        &self.assessments
    }
    /// Every requested candidate is a result under the requested usability policy.
    /// Seed-only and diagnostic-only candidates are never results (ADR-0106).
    pub fn usable(&self) -> bool {
        self.report.is_ok()
            && !self.assessments.is_empty()
            && self.assessments.iter().all(|a| a.permits_result)
    }
    pub(super) fn modeling_result(&self, step: usize) -> Option<&super::ModelingResult> {
        match &self.report {
            Ok(RunReport::Modeling(results)) => results.get(step),
            Err(_) => self.modeling_failure.as_ref()?.completed.get(step),
            _ => None,
        }
    }
    pub(super) fn modeling_step(&self, index: usize) -> usize {
        self.modeling_result(index)
            .map_or(self.modeling_origin + index, |result| {
                result.original_attempt
            })
    }
    pub(super) fn modeling_error(&self, step: usize) -> Option<&WorkflowError> {
        let failed = self
            .modeling_failure
            .as_ref()
            .map_or(Some(0), |failure| failure.step);
        (Some(step) == failed)
            .then(|| self.report.as_ref().err().map(AsRef::as_ref))
            .flatten()
    }
    pub(super) fn completed(mut self) -> Self {
        if self.original_completion_captured || self.horizon_unmerged.is_some() {
            return self;
        }
        if let Ok(RunReport::Fit(r)) = &mut self.report
            && let Some(seed) = r.solve.as_mut().and_then(|s| s.warm_start.as_mut())
        {
            seed.origin = Some(pse_backend_native::solve::SeedOrigin {
                run: Some(self.run_id),
                attempt: 0,
            });
        }
        self.assessments = self.assess_candidates();
        self.completion = self.capture_completion().map_err(Arc::new);
        self.reset_result_projection();
        self
    }
    /// Capture completion, then record the attempt's end durably when the run is durable.
    pub(super) async fn finished(self, durable: Option<DurableAttempt>, cancelled: bool) -> Self {
        let _phase = pse_diagnostics::observation::phase("run.completion.and.publication");
        let mut result = self.completed();
        if let Some(context) = pse_diagnostics::observation::current() {
            context.boolean("run.cancel_requested", cancelled);
            context.boolean("run.execution_failed", result.report.is_err());
            context.boolean("run.original_completion.failed", result.completion.is_err());
            context.source_ref("run.identity", result.run_id.as_id().to_string().as_bytes());
        }
        if let Some(attempt) = durable {
            let record = attempt.finish(&result, cancelled).await;
            if let Some(context) = pse_diagnostics::observation::current() {
                context.boolean("run.publication.failed", record.attempt.is_err());
            }
            if let Err(error) = &record.attempt {
                result.report = Err(error.clone());
                result.reset_result_projection();
            }
            result.durability = RunDurability::Durable(Arc::new(record));
        }
        result
    }
    /// A durable run that failed before it ran: record why, and publish the failure.
    pub(super) async fn refused(mut self, durable: Option<DurableAttempt>) -> Self {
        self = self.completed();
        if let (Some(attempt), Err(error)) = (durable, &self.report) {
            let error = error.clone();
            let record = attempt.abandon(&error).await;
            // A cleanup or sealing failure remains visible in the durable record.
            // It must not replace the refusal that prevented scientific execution.
            if record.attempt.is_err() {
                self.completion = Err(error);
                self.reset_result_projection();
            }
            self.durability = RunDurability::Durable(Arc::new(record));
        }
        self
    }

    /// Full typed reports and backend-specific metrics; errors preserve their original causes.
    pub fn report(&self) -> Result<&RunReport, &WorkflowError> {
        self.report.as_ref().map_err(AsRef::as_ref)
    }
    /// Original immutable declarations for each requested step, including unattempted steps.
    pub fn request(&self) -> &RunRequest {
        &self.request
    }
    /// What the run recorded durably: nothing for an ephemeral run; the attempt, its stored
    /// progress stream and its stored seeds for a durable one.
    pub const fn durability(&self) -> &RunDurability {
        &self.durability
    }
    /// Exact retained canonical run identity when registration was acknowledged.
    pub fn canonical_run_key(&self) -> Option<&str> {
        match &self.durability {
            RunDurability::Ephemeral => None,
            RunDurability::Durable(record) => record.run.as_ref().map(|run| run.key.as_str()),
        }
    }
    /// Exact native attempt identity; no truncated scientific identity is used for lookup.
    pub fn canonical_attempt_key(&self) -> Option<&str> {
        match &self.durability {
            RunDurability::Ephemeral => None,
            RunDurability::Durable(record) => record.attempt_key.as_deref(),
        }
    }
    /// What a rolling horizon did at each sample: its measurements, the steps that acted
    /// and the inputs it applied (Plan 22 Y5c); `None` for any other run. Its estimator and
    /// controller steps are the run's modeling steps.
    pub fn horizon(&self) -> Option<&super::HorizonReport> {
        self.horizon.as_deref()
    }
}

/// The durable attempt of a run under `runtime`'s class, or none when ephemeral. A worker
/// passes the attempt it claimed with its job, already under its lease.
pub(super) fn attempt_for(
    runtime: &Runtime,
    given: Option<DurableAttempt>,
) -> Result<Option<DurableAttempt>, WorkflowError> {
    match (&runtime.durability, given) {
        (Durability::Ephemeral, None) => Ok(None),
        (Durability::Ephemeral, Some(_)) => {
            Err(contract("a claimed job runs only on a durable runtime"))
        }
        (Durability::Durable(operations), None) => Ok(Some(DurableAttempt::new(operations)?)),
        (Durability::Durable(_), Some(attempt)) => Ok(Some(attempt)),
    }
}
/// The run's event stream: bounded in memory and, for a durable run, tapped by its store
/// stream, which keeps every event.
pub(super) fn progress_for(history: usize, durable: Option<&DurableAttempt>) -> Arc<Progress> {
    Arc::new(match durable {
        Some(attempt) => Progress::tapped(history, attempt.tap()),
        None => Progress::new(history),
    })
}
/// Register a durable attempt (planned, queued), wait for native admission, and enter
/// running under its lease; `admission` resolves once native admission holds.
async fn admit<T>(
    durable: &mut Option<DurableAttempt>,
    run_id: RunId,
    request: &RunRequest,
    admission: impl Future<Output = Result<T, WorkflowError>>,
    cancel: Canceller,
) -> Result<T, WorkflowError> {
    if let Some(attempt) = durable.as_ref() {
        attempt.register(run_id, request).await?;
    }
    if let Some(attempt) = durable.as_mut() {
        attempt.start(cancel).await?;
    }
    admission.await
}
/// Submit a native operation under bounded population admission. Durable provenance
/// observes when CPU and memory are admitted without adding another waiting population.
fn submission(
    cancel: &FlightCancellation,
    progress: &Arc<Progress>,
    durable: bool,
    deadline: std::time::Instant,
) -> (Submission, Option<tokio::sync::oneshot::Receiver<()>>) {
    let (admitted, signal) = if durable {
        let (tx, rx) = tokio::sync::oneshot::channel();
        (Some(tx), Some(rx))
    } else {
        (None, None)
    };
    (
        Submission {
            cancel: cancel.clone(),
            progress: progress.clone(),
            admitted,
            deadline: Some(deadline),
        },
        signal,
    )
}
async fn admitted(signal: Option<tokio::sync::oneshot::Receiver<()>>) -> Result<(), WorkflowError> {
    match signal {
        Some(signal) => signal
            .await
            .map_err(|_| WorkflowError::Math(MathRuntimeError::Cancelled)),
        None => Ok(()),
    }
}
/// The existing supervisor keeps ownership through cancellation and native drain.
fn supervised_with_cancel(
    work: impl Future<Output = ()>,
    stop: impl FnOnce(),
    parent: Option<crate::CancelSource>,
) -> impl Future<Output = ()> {
    // Box before constructing the supervisor future. Keeping the full scientific
    // future inline makes task-local polling/drop exceed ordinary executor stacks.
    let mut work = Box::pin(work);
    async move {
        if let Some(parent) = parent {
            tokio::select! {
                () = &mut work => {},
                () = parent.cancelled() => { stop(); work.await; },
            }
        } else {
            work.await;
        }
    }
}

impl super::ModelingSimulation {
    /// Start the authored simulation through the shared joined run and durable lifecycle.
    pub fn start(&self) -> Result<RunHandle, WorkflowError> {
        self.start_owned(None, self.admission_deadline()?, None)
    }
    /// Bind a bounded live subscription before any native effect. Dropping the cursor
    /// detaches only that subscription; the run retains cancellation and drain ownership.
    pub fn start_live(
        &self,
        options: super::LiveSimulationOptions,
    ) -> Result<(RunHandle, super::LiveSimulationCursor), WorkflowError> {
        let (handle, cursor) =
            self.start_owned_live(None, self.admission_deadline()?, None, Some(options))?;
        Ok((
            handle,
            cursor.ok_or_else(|| contract("live subscription absent"))?,
        ))
    }
    /// Start with caller cancellation linked through pending admission and native drain.
    pub fn start_with_cancel(
        &self,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        if cancel.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled.into());
        }
        let deadline = self.admission_deadline()?;
        self.start_owned(
            None,
            cancel
                .deadline()
                .map_or(deadline, |parent| deadline.min(parent)),
            Some(cancel.clone()),
        )
    }
    pub(crate) async fn start_attempt(
        &self,
        attempt: DurableAttempt,
    ) -> Result<RunHandle, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                let admission = self.admission_deadline().and_then(|deadline| {
                    if matches!(self.runtime.durability(), Durability::Ephemeral) {
                        Err(contract(
                            "a claimed simulation runs only on a durable runtime",
                        ))
                    } else {
                        Ok(deadline)
                    }
                });
                let deadline = match admission {
                    Ok(deadline) => deadline,
                    Err(error) => {
                        let error = Arc::new(error);
                        let _ = attempt.abandon(&error).await;
                        return Err(WorkflowError::Shared(error));
                    }
                };
                self.start_owned(Some(attempt), deadline, None)
            })
            .await
    }
    fn admission_deadline(&self) -> Result<std::time::Instant, WorkflowError> {
        std::time::Instant::now()
            .checked_add(self.profile().time_limit)
            .ok_or_else(|| contract("submitted task deadline extent"))
    }
    fn start_owned(
        &self,
        attempt: Option<DurableAttempt>,
        deadline: std::time::Instant,
        parent: Option<crate::CancelSource>,
    ) -> Result<RunHandle, WorkflowError> {
        self.start_owned_live(attempt, deadline, parent, None)
            .map(|(handle, _)| handle)
    }
    fn start_owned_live(
        &self,
        attempt: Option<DurableAttempt>,
        deadline: std::time::Instant,
        parent: Option<crate::CancelSource>,
        live: Option<super::LiveSimulationOptions>,
    ) -> Result<(RunHandle, Option<super::LiveSimulationCursor>), WorkflowError> {
        if let Some(options) = live {
            options.admit(self.profile())?;
        }
        let activity = self.runtime.admit_activity()?;
        activity.sync_scope(|| {
            let runtime = self.runtime.clone();
            let run_id: RunId = pse_operations::mint_id();
            let mut durable = attempt_for(&runtime, attempt)?;
            let run_id = durable
                .as_ref()
                .and_then(DurableAttempt::claimed_run)
                .unwrap_or(run_id);
            let progress = progress_for(256, durable.as_ref());
            let cancel = FlightCancellation::default();
            let forwarding = cancel.clone();
            let (submission, signal) = submission(&cancel, &progress, durable.is_some(), deadline);
            let canonical_attempt = durable.as_ref().map(|attempt| attempt.canonical_keys(run_id).1);
            let live = live.map(|options| super::live_simulation::LiveDispatch::new(
                self.clone(), options, run_id, canonical_attempt,
            )).transpose()?;
            let (observer, mut live_receiver, subscriber, mut cursor) = match live {
                Some(live) => (Some((live.retention, live.observer)), Some(live.receiver), Some(live.subscriber), Some(live.cursor)),
                None => (None, None, None, None),
            };
            let dispatch_source = self.clone();
            let mut dispatch = Some(move || match observer {
                Some((retention, observer)) => dispatch_source.submit_with_samples(run_id, submission, retention, observer),
                None => dispatch_source.submit(run_id, submission),
            });
            let handle = if durable.is_none() {
                Some(dispatch
                    .take()
                    .ok_or_else(|| contract("native dispatch absent"))?(
                )?)
            } else {
                None
            };
            let lease = Arc::new(Lease(cancel.clone(), None, pse_diagnostics::observation::current()));
            let attempt_id = durable.as_ref().map(DurableAttempt::attempt_id);
            let canonical_keys = durable
                .as_ref()
                .map(|attempt| attempt.canonical_keys(run_id));
            let (sender, receiver) = tokio::sync::watch::channel(None);
            if let Some(cursor) = cursor.as_mut() { cursor.bind_completion(receiver.clone()); }
            let prepared = self.clone();
            tokio::spawn(crate::workflow::inherit_runtime_activity(
                supervised_with_cancel(
                    async move {
                        let request = RunRequest::Simulation(Box::new(prepared.clone()));
                        let stop = cancel.clone();
                        let registered = async {
                            if let Some(attempt) = durable.as_mut() {
                                attempt.register(run_id, &request).await?;
                                let stop = stop.clone();
                                attempt.start(Arc::new(move || stop.cancel())).await?;
                            }
                            Ok::<(), WorkflowError>(())
                        }
                        .await;
                        if let Err(error) = registered {
                            if let Some(handle) = handle {
                                handle.cancel();
                                let _ = handle.finish().await;
                            }
                            let refused = RunResult::joined(
                                run_id,
                                runtime,
                                request,
                                None,
                                Err(Arc::new(error)),
                            )
                            .refused(durable)
                            .await;
                            sender.send_replace(Some(Arc::new(refused)));
                            return;
                        }
                        let handle = match handle {
                            Some(handle) => Ok(handle),
                            None => match dispatch.take() {
                                Some(dispatch) => dispatch(),
                                None => Err(contract("native dispatch absent")),
                            },
                        };
                        let handle = match handle {
                            Ok(handle) => handle,
                            Err(error) => {
                                let refused = RunResult::joined(
                                    run_id,
                                    runtime,
                                    request,
                                    None,
                                    Err(Arc::new(error)),
                                )
                                .refused(durable)
                                .await;
                                sender.send_replace(Some(Arc::new(refused)));
                                return;
                            }
                        };
                        if let Err(error) = admit(
                            &mut durable,
                            run_id,
                            &request,
                            admitted(signal),
                            Arc::new(move || stop.cancel()),
                        )
                        .await
                        {
                            handle.cancel();
                            let _ = handle.finish().await;
                            let refused = RunResult::joined(
                                run_id,
                                runtime,
                                request,
                                None,
                                Err(Arc::new(error)),
                            )
                            .refused(durable)
                            .await;
                            sender.send_replace(Some(Arc::new(refused)));
                            return;
                        }
                        if live_receiver.is_some() && let Some(attempt) = durable.as_mut()
                            && let Err(error) = attempt.begin_live_samples()
                                && let Some(subscriber) = &subscriber {
                                    super::live_simulation::deliver(subscriber, super::LiveSimulationEvent::RecordingUnknown(Arc::new(error)), &cancel, deadline).await;
                                }
                        let finished = handle.finish();
                        tokio::pin!(finished);
                        let mut joined = None;
                        let mut selected = false;
                        if let Some(receiver) = live_receiver.as_mut() {
                            loop {
                                tokio::select! {
                                    result = &mut finished, if joined.is_none() => { joined = Some(result); }
                                    event = receiver.recv() => {
                                        let Some(event) = event else { break; };
                                        let append = match &event {
                                            super::LiveSimulationEvent::Sample(batch) => Some(batch.table.clone()),
                                            super::LiveSimulationEvent::Selected { .. } => { selected = true; None },
                                            _ => None,
                                        };
                                        if let Some(subscriber) = &subscriber {
                                            super::live_simulation::deliver(subscriber, event, &cancel, deadline).await;
                                        }
                                        if let (Some(table), Some(attempt)) = (append, durable.as_mut()) {
                                            let event = match attempt.publish_live_samples(&table).await {
                                                Ok(rows) => super::LiveSimulationEvent::Recorded { rows },
                                                Err(error) => super::LiveSimulationEvent::RecordingUnknown(Arc::new(error)),
                                            };
                                            if let Some(subscriber) = &subscriber {
                                                super::live_simulation::deliver(subscriber, event, &cancel, deadline).await;
                                            }
                                        }
                                    }
                                }
                            }
                            if selected && let Some(attempt) = durable.as_mut() {
                                let event = match attempt.close_live_samples().await {
                                    Ok(()) => super::LiveSimulationEvent::RecordingClosed,
                                    Err(error) => super::LiveSimulationEvent::RecordingUnknown(Arc::new(error)),
                                };
                                if let Some(subscriber) = &subscriber {
                                    super::live_simulation::deliver(subscriber, event, &cancel, deadline).await;
                                }
                            }
                        }
                        let joined = match joined { Some(result) => result, None => finished.await };
                        let report = match joined {
                            Ok(((report, checks, accuracy), owner)) => prepared
                                .finish(run_id, report, checks, accuracy, owner)
                                .map(|trajectory| RunReport::Simulation(Box::new(trajectory)))
                                .map_err(Arc::new),
                            Err(error) => Err(Arc::new(WorkflowError::Math(error))),
                        };
                        let cancelled = cancel.flag().load(std::sync::atomic::Ordering::Acquire);
                        let result = RunResult::joined(run_id, runtime, request, None, report)
                            .finished(durable, cancelled)
                            .await;
                        let result = Arc::new(result);
                        sender.send_replace(Some(result));
                    },
                    move || forwarding.cancel(),
                    parent,
                ),
            ));
            Ok((RunHandle {
                observation: observations_for(run_id, canonical_keys.as_ref()),
                lease,
                receiver,
                progress,
                run_id,
                attempt_id,
                canonical_keys,
            }, cursor))
        })
    }
}

#[cfg(feature = "solver-diffsol")]
impl super::ShootingProblem {
    /// Start under the runtime's shared resource, cancellation and joined run lifecycle.
    pub fn start(self: &Arc<Self>) -> Result<RunHandle, WorkflowError> {
        self.start_requested(None, None)
    }
    /// Admit an explicit original-coordinate initial vector before native dispatch.
    pub fn start_with_initial(
        self: &Arc<Self>,
        initial: Vec<f64>,
    ) -> Result<RunHandle, WorkflowError> {
        if initial.len() != self.initial_extent() || initial.iter().any(|v| !v.is_finite()) {
            return Err(contract(
                "shooting initial coordinate extent or nonfinite value",
            ));
        }
        self.start_requested(Some(initial), None)
    }
    /// Preserve caller cancellation through the existing shooting supervisor.
    pub fn start_replay_with_cancel(
        self: &Arc<Self>,
        initial: Option<Vec<f64>>,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        if cancel.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled.into());
        }
        if initial.as_ref().is_some_and(|initial| {
            initial.len() != self.initial_extent() || initial.iter().any(|v| !v.is_finite())
        }) {
            return Err(contract(
                "shooting initial coordinate extent or nonfinite value",
            ));
        }
        self.start_requested(initial, Some(cancel.clone()))
    }
    fn start_requested(
        self: &Arc<Self>,
        initial: Option<Vec<f64>>,
        parent: Option<crate::CancelSource>,
    ) -> Result<RunHandle, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity.sync_scope(|| {
            let runtime = self.runtime.clone();
            let run_id: RunId = pse_operations::mint_id();
            let mut durable = attempt_for(&runtime, None)?;
            let progress = progress_for(256, durable.as_ref());
            let cancel = FlightCancellation::default();
            let forwarding = cancel.clone();
            let deadline = std::time::Instant::now()
                .checked_add(self.solver.controls.time_limit)
                .ok_or_else(|| contract("submitted task deadline extent"))?;
            let deadline = parent
                .as_ref()
                .and_then(crate::CancelSource::deadline)
                .map_or(deadline, |outer| deadline.min(outer));
            let (submission, signal) = submission(&cancel, &progress, durable.is_some(), deadline);
            let prepared = self.clone();
            let mut declared_profile = self.solver.clone();
            if initial.is_some() {
                declared_profile.controls.start = pse_backend_native::solve::StartPolicy::Explicit;
            }
            let start = initial.clone();
            let native = runtime.native().clone();
            let job_bytes = self.job_bytes()?;
            let mut dispatch = Some(move || {
                native.submit_with(
                    declared_profile.controls.threads,
                    job_bytes,
                    submission,
                    move |flag, progress| {
                        let report = prepared.solve(
                            run_id,
                            pse_kernels::ExecutionScope::new(flag, Some(deadline)),
                            progress,
                            start.as_deref(),
                            &declared_profile,
                        )?;
                        let retained = report
                            .numeric_bytes()
                            .checked_add(prepared.solver.controls.report_allowance()?)
                            .ok_or(MathRuntimeError::Limit("shooting result extent"))?;
                        Ok((RunReport::Shooting(Box::new(report)), retained))
                    },
                )
            });
            let handle = if durable.is_none() {
                Some(dispatch
                    .take()
                    .ok_or_else(|| contract("native dispatch absent"))?(
                )?)
            } else {
                None
            };
            let lease = Arc::new(Lease(
                cancel.clone(),
                None,
                pse_diagnostics::observation::current(),
            ));
            let attempt_id = durable.as_ref().map(DurableAttempt::attempt_id);
            let canonical_keys = durable
                .as_ref()
                .map(|attempt| attempt.canonical_keys(run_id));
            let (sender, receiver) = tokio::sync::watch::channel(None);
            let request = RunRequest::Shooting {
                problem: self.clone(),
                initial,
            };
            tokio::spawn(crate::workflow::inherit_runtime_activity(
                supervised_with_cancel(
                    async move {
                        let stop = cancel.clone();
                        let registered = async {
                            if let Some(attempt) = durable.as_mut() {
                                attempt.register(run_id, &request).await?;
                                let stop = stop.clone();
                                attempt.start(Arc::new(move || stop.cancel())).await?;
                            }
                            Ok::<(), WorkflowError>(())
                        }
                        .await;
                        if let Err(error) = registered {
                            if let Some(handle) = handle {
                                handle.cancel();
                                let _ = handle.finish().await;
                            }
                            let refused = RunResult::joined(
                                run_id,
                                runtime,
                                request,
                                None,
                                Err(Arc::new(error)),
                            )
                            .refused(durable)
                            .await;
                            sender.send_replace(Some(Arc::new(refused)));
                            return;
                        }
                        let handle = match handle {
                            Some(handle) => Ok(handle),
                            None => match dispatch.take() {
                                Some(dispatch) => dispatch().map_err(WorkflowError::from),
                                None => Err(contract("native dispatch absent")),
                            },
                        };
                        let handle = match handle {
                            Ok(handle) => handle,
                            Err(error) => {
                                let refused = RunResult::joined(
                                    run_id,
                                    runtime,
                                    request,
                                    None,
                                    Err(Arc::new(error)),
                                )
                                .refused(durable)
                                .await;
                                sender.send_replace(Some(Arc::new(refused)));
                                return;
                            }
                        };
                        if let Err(error) = admit(
                            &mut durable,
                            run_id,
                            &request,
                            admitted(signal),
                            Arc::new(move || stop.cancel()),
                        )
                        .await
                        {
                            handle.cancel();
                            let _ = handle.finish().await;
                            let refused = RunResult::joined(
                                run_id,
                                runtime,
                                request,
                                None,
                                Err(Arc::new(error)),
                            )
                            .refused(durable)
                            .await;
                            sender.send_replace(Some(Arc::new(refused)));
                            return;
                        }
                        let (report, owner) = match handle.finish().await {
                            Ok((report, owner)) => (Ok(report), Some(owner)),
                            Err(error) => (Err(Arc::new(WorkflowError::Math(error))), None),
                        };
                        let cancelled = cancel.flag().load(std::sync::atomic::Ordering::Acquire);
                        let result = RunResult::joined(run_id, runtime, request, owner, report)
                            .finished(durable, cancelled)
                            .await;
                        sender.send_replace(Some(Arc::new(result)));
                    },
                    move || forwarding.cancel(),
                    parent,
                ),
            ));
            Ok(RunHandle {
                observation: observations_for(run_id, canonical_keys.as_ref()),
                lease,
                receiver,
                progress,
                run_id,
                attempt_id,
                canonical_keys,
            })
        })
    }
}

impl super::PreparedFit {
    /// Start one native fitting attempt under the existing joined job lifecycle.
    pub fn start(&self) -> Result<RunHandle, WorkflowError> {
        self.start_owned(None, self.admission_deadline()?, None)
    }
    /// Start with caller cancellation linked through pending admission and native drain.
    pub fn start_with_cancel(
        &self,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        if cancel.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled.into());
        }
        let deadline = self.admission_deadline()?;
        self.start_owned(
            None,
            cancel
                .deadline()
                .map_or(deadline, |parent| deadline.min(parent)),
            Some(cancel.clone()),
        )
    }
    pub(crate) async fn start_attempt(
        &self,
        attempt: DurableAttempt,
    ) -> Result<RunHandle, WorkflowError> {
        let activity = self.problem.runtime.admit_activity()?;
        activity
            .scope(async {
                let admission = self.admission_deadline().and_then(|deadline| {
                    if matches!(self.problem.runtime.durability(), Durability::Ephemeral) {
                        Err(contract("a claimed fit runs only on a durable runtime"))
                    } else {
                        Ok(deadline)
                    }
                });
                let deadline = match admission {
                    Ok(deadline) => deadline,
                    Err(error) => {
                        let error = Arc::new(error);
                        let _ = attempt.abandon(&error).await;
                        return Err(WorkflowError::Shared(error));
                    }
                };
                self.start_owned(Some(attempt), deadline, None)
            })
            .await
    }
    fn admission_deadline(&self) -> Result<std::time::Instant, WorkflowError> {
        std::time::Instant::now()
            .checked_add(self.problem.profile.solver.controls.time_limit)
            .ok_or_else(|| contract("submitted task deadline extent"))
    }
    fn start_owned(
        &self,
        attempt: Option<DurableAttempt>,
        deadline: std::time::Instant,
        parent: Option<crate::CancelSource>,
    ) -> Result<RunHandle, WorkflowError> {
        let activity = self.problem.runtime.admit_activity()?;
        activity.sync_scope(|| {
            let runtime = self.problem.runtime.clone();
            let prepared = self.clone();
            let run_id: RunId = pse_operations::mint_id();
            let mut durable = attempt_for(&runtime, attempt)?;
            let run_id = durable
                .as_ref()
                .and_then(DurableAttempt::claimed_run)
                .unwrap_or(run_id);
            let progress = progress_for(256, durable.as_ref());
            let cancel = FlightCancellation::default();
            let forwarding = cancel.clone();
            let (submission, signal) = submission(&cancel, &progress, durable.is_some(), deadline);
            // Estimation admits only its own demand. Profile teams acquire their next
            // stage after the estimate reaches the quiescent phase boundary.
            let workers = self.problem.profile_workers(runtime.native().cores());
            let cores = self.problem.profile.solver.controls.threads;
            let bytes = self.problem.bytes;
            let native = runtime.native().clone();
            #[cfg(test)]
            let entry = family_worker_entry();
            let mut dispatch = Some(move || {
                native.submit_with(cores, bytes, submission, move |flag, progress| {
                    #[cfg(test)]
                    if let Some(entry) = entry {
                        entry(&flag);
                    }
                    let allowance = prepared
                        .problem
                        .profile
                        .solver
                        .controls
                        .report_allowance()?;
                    let report = prepared.execute(
                        run_id,
                        pse_kernels::ExecutionScope::new(flag, Some(deadline)),
                        progress,
                        workers,
                    )?;
                    let retained = report
                        .numeric_bytes()
                        .checked_add(allowance)
                        .ok_or(MathRuntimeError::Limit("fit result extent"))?;
                    Ok((RunReport::Fit(Box::new(report)), retained))
                })
            });
            let handle = if durable.is_none() {
                Some(dispatch
                    .take()
                    .ok_or_else(|| contract("native dispatch absent"))?(
                )?)
            } else {
                None
            };
            let lease = Arc::new(Lease(
                cancel.clone(),
                None,
                pse_diagnostics::observation::current(),
            ));
            let attempt_id = durable.as_ref().map(DurableAttempt::attempt_id);
            let canonical_keys = durable
                .as_ref()
                .map(|attempt| attempt.canonical_keys(run_id));
            let (sender, receiver) = tokio::sync::watch::channel(None);
            let request = RunRequest::Fit(Box::new(self.clone()));
            tokio::spawn(crate::workflow::inherit_runtime_activity(
                supervised_with_cancel(
                    async move {
                        let stop = cancel.clone();
                        let registered = async {
                            if let Some(attempt) = durable.as_mut() {
                                attempt.register(run_id, &request).await?;
                                let stop = stop.clone();
                                attempt.start(Arc::new(move || stop.cancel())).await?;
                            }
                            Ok::<(), WorkflowError>(())
                        }
                        .await;
                        if let Err(error) = registered {
                            if let Some(handle) = handle {
                                handle.cancel();
                                let _ = handle.finish().await;
                            }
                            let refused = RunResult::joined(
                                run_id,
                                runtime,
                                request,
                                None,
                                Err(Arc::new(error)),
                            )
                            .refused(durable)
                            .await;
                            sender.send_replace(Some(Arc::new(refused)));
                            return;
                        }
                        let handle = match handle {
                            Some(handle) => Ok(handle),
                            None => match dispatch.take() {
                                Some(dispatch) => dispatch().map_err(WorkflowError::from),
                                None => Err(contract("native dispatch absent")),
                            },
                        };
                        let handle = match handle {
                            Ok(handle) => handle,
                            Err(error) => {
                                let refused = RunResult::joined(
                                    run_id,
                                    runtime,
                                    request,
                                    None,
                                    Err(Arc::new(error)),
                                )
                                .refused(durable)
                                .await;
                                sender.send_replace(Some(Arc::new(refused)));
                                return;
                            }
                        };
                        if let Err(error) = admit(
                            &mut durable,
                            run_id,
                            &request,
                            admitted(signal),
                            Arc::new(move || stop.cancel()),
                        )
                        .await
                        {
                            handle.cancel();
                            let _ = handle.finish().await;
                            let refused = RunResult::joined(
                                run_id,
                                runtime,
                                request,
                                None,
                                Err(Arc::new(error)),
                            )
                            .refused(durable)
                            .await;
                            sender.send_replace(Some(Arc::new(refused)));
                            return;
                        }
                        let (report, owner) = match handle.finish().await {
                            Ok((r, o)) => (Ok(r), Some(o)),
                            Err(e) => (Err(Arc::new(WorkflowError::Math(e))), None),
                        };
                        let cancelled = cancel.flag().load(std::sync::atomic::Ordering::Acquire);
                        let result = RunResult::joined(run_id, runtime, request, owner, report)
                            .finished(durable, cancelled)
                            .await;
                        sender.send_replace(Some(Arc::new(result)));
                    },
                    move || forwarding.cancel(),
                    parent,
                ),
            ));
            Ok(RunHandle {
                observation: observations_for(run_id, canonical_keys.as_ref()),
                lease,
                receiver,
                progress,
                run_id,
                attempt_id,
                canonical_keys,
            })
        })
    }
}

impl super::ModelingSolvePreparation {
    /// Retain a compatible native seed without retaining mutable solver state.
    pub fn with_start(
        mut self,
        seed: pse_backend_native::solve::WarmStart,
    ) -> Result<Self, WorkflowError> {
        self.solve = self
            .solve
            .with_start(seed)
            .map_err(MathRuntimeError::from)?;
        self.profile.controls.start = pse_backend_native::solve::StartPolicy::Explicit;
        Ok(self)
    }
    /// Select the complete original free-coordinate primal seed by semantic identity.
    pub fn with_primal_start(
        mut self,
        values: BTreeMap<SemanticId, f64>,
    ) -> Result<Self, WorkflowError> {
        self.solve = self
            .solve
            .with_primal_start(values)
            .map_err(MathRuntimeError::from)?;
        self.profile.controls.start = pse_backend_native::solve::StartPolicy::Explicit;
        Ok(self)
    }
    /// Start from a seed in the operational store's solution store (ADR-0114 Outcome 17):
    /// the newest seed stored for this preparation's coordinates and backend, or an
    /// explicit solution. The seed must match the coordinate-compatibility (layout) stamp
    /// and backend exactly; numeric data and native options may differ (F24). Every free
    /// coordinate's start source records the solution, and the seed's content identity
    /// enters the lineage of the result (F25).
    ///
    /// # Errors
    /// A constant evaluation (no seed), no compatible stored seed, an unknown solution, a
    /// store failure, or an incompatible seed, which is refused.
    pub async fn with_stored_start(
        self,
        operations: &super::Operations,
        which: StoredStart,
    ) -> Result<Self, WorkflowError> {
        let target = self
            .solve
            .compatibility()
            .cloned()
            .ok_or_else(|| contract("a constant evaluation consumes no stored seed"))?;
        let admitted = match which {
            StoredStart::Latest => {
                let preparation = self
                    .solve
                    .seed_preparation_identity()
                    .ok_or_else(|| contract("a constant evaluation consumes no stored seed"))?;
                operations
                    .latest_seed(&target, &preparation, None)
                    .await?
                    .ok_or_else(|| contract("no retained seed matches preparation"))?
            }
            StoredStart::Solution(id) => operations.seed(id).await?,
        };
        self.with_admitted_start(&admitted)
    }
    pub(crate) fn with_admitted_start(
        mut self,
        admitted: &super::AdmittedSeed,
    ) -> Result<Self, WorkflowError> {
        let solution = admitted.solution().as_id();
        let columns: Vec<SemanticId> = self.model.case.compiled().plan.columns().to_vec();
        let owner = admitted.start.owner();
        self.solve = self
            .solve
            .with_start_shared(admitted.start.clone())
            .map_err(MathRuntimeError::from)?;
        self.profile.controls.start = pse_backend_native::solve::StartPolicy::Explicit;
        self.stored_seed_owner = Some(owner);
        for column in columns {
            self.starts
                .insert(column, super::StartSource::Stored { solution });
        }
        Ok(self)
    }
    /// Start one authored algebraic run: a one-step authored sequence.
    pub fn start(&self) -> Result<RunHandle, WorkflowError> {
        self.source
            .runtime
            .launch(vec![self.clone()], false, None, None)
    }
}

impl Runtime {
    /// Start a finite authored sequence of prepared steps.
    pub async fn start_modeling(
        &self,
        steps: Vec<super::ModelingSolvePreparation>,
        continue_independent: bool,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                self.check_sequence(&steps)?;
                if cancel.token().is_cancelled() {
                    return Err(MathRuntimeError::Cancelled.into());
                }
                self.launch(steps, continue_independent, None, Some(cancel.clone()))
            })
            .await
    }
    /// Run a claimed job's prepared steps as its attempt, which the worker already holds
    /// under its lease; the attempt ends through the job's retry policy.
    pub(crate) async fn start_attempt(
        &self,
        steps: Vec<super::ModelingSolvePreparation>,
        attempt: DurableAttempt,
    ) -> Result<RunHandle, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let admission = self.check_sequence(&steps).and_then(|()| {
                    if matches!(self.durability(), Durability::Ephemeral) {
                        Err(contract("a claimed solve runs only on a durable runtime"))
                    } else {
                        Ok(())
                    }
                });
                if let Err(error) = admission {
                    let error = Arc::new(error);
                    let _ = attempt.abandon(&error).await;
                    return Err(WorkflowError::Shared(error));
                }
                self.launch(steps, false, Some(attempt), None)
            })
            .await
    }
    fn check_sequence(
        &self,
        steps: &[super::ModelingSolvePreparation],
    ) -> Result<(), WorkflowError> {
        if steps.is_empty()
            || steps.len() > 4096
            || steps
                .iter()
                .any(|p| !Arc::ptr_eq(&p.source.runtime.shared, &self.shared))
        {
            return Err(contract(
                "empty, excessive or mixed-runtime authored sequence",
            ));
        }
        let mut declarations = BTreeMap::new();
        for step in steps {
            if step.source.physical.identity() != steps[0].source.physical.identity() {
                return Err(contract("sequence needs one admitted physical context"));
            }
            for (logical, version) in step.model.model.consumed_source_versions() {
                if declarations
                    .insert(logical, version)
                    .is_some_and(|old| old != version)
                {
                    return Err(contract(
                        "sequence contains conflicting source declarations",
                    ));
                }
            }
        }
        Ok(())
    }
    /// Run prepared steps as one staged sequence (A6) on one native session: each step is
    /// assessed against the original model on its worker, a `PreviousAccepted` step starts
    /// from the previous step's seed when that step was accepted, and an unaccepted step
    /// ends the sequence unless the steps are independent. The handle shares one progress
    /// stream across steps; cancelling it stops the current step and joins native teardown.
    ///
    /// Both routes require a bounded population ticket. A durable run registers its
    /// attempt, waits for CPU/memory under its original clock, and runs under its lease.
    fn launch(
        &self,
        steps: Vec<super::ModelingSolvePreparation>,
        continue_independent: bool,
        attempt: Option<DurableAttempt>,
        parent: Option<crate::CancelSource>,
    ) -> Result<RunHandle, WorkflowError> {
        let activity = self.admit_activity()?;
        activity.sync_scope(|| {
        let history = steps
            .iter()
            .map(|s| s.profile.controls.history)
            .max()
            .unwrap_or(0);
        let mut durable = attempt_for(self, attempt)?;
        let progress = progress_for(history, durable.as_ref());
        let opened = match durable {
            None => Some(super::staged::Staged::open(self, Some(progress.clone()))?),
            Some(_) => None,
        };
        let cancel = crate::CancelSource::new().with_deadline(parent.as_ref().and_then(crate::CancelSource::deadline));
        let forwarding = cancel.clone();
        let checks = cancel.clone();
        let (sender, receiver) = tokio::sync::watch::channel(None);
        let runtime = self.clone();
        // A claimed try runs as the run its job's attempts share; a new run is minted.
        let run_id: RunId = durable
            .as_ref()
            .and_then(DurableAttempt::claimed_run)
            .unwrap_or_else(pse_operations::mint_id);
        let attempt_id = durable.as_ref().map(DurableAttempt::attempt_id);
        let canonical_keys = durable
            .as_ref()
            .map(|attempt| attempt.canonical_keys(run_id));
        let shared = progress.clone();
        tokio::spawn(crate::workflow::inherit_runtime_activity(supervised_with_cancel(async move {
            let request = RunRequest::Modeling(steps.clone());
            let stop = cancel.clone();
            let session = async {
                match opened {
                    Some(staged) => Ok(staged),
                    None => tokio::select! {
                        staged=super::staged::Staged::open_queued(&runtime,Some(shared), &cancel,
                            steps.iter().filter_map(|step| step.solve.task_scope().and_then(|scope| scope.deadline()))
                                .chain(cancel.deadline()).min())=>staged,
                        ()=cancel.cancelled()=>Err(MathRuntimeError::Cancelled.into()),
                    },
                }
            };
            let mut staged = match admit(
                &mut durable,
                run_id,
                &request,
                session,
                Arc::new(move || stop.cancel()),
            )
            .await
            {
                Ok(staged) => staged,
                Err(error) => {
                    let refused =
                        RunResult::joined(run_id, runtime, request, None, Err(Arc::new(error)))
                            .with_modeling_policy(continue_independent)
                            .refused(durable)
                            .await;
                    sender.send_replace(Some(Arc::new(refused)));
                    return;
                }
            };
            let mut results = Vec::new();
            let mut failure = None;
            for (attempt, step) in steps.iter().enumerate() {
                if cancel.token().is_cancelled() {
                    break;
                }
                if let Some(durable) = durable.as_ref() {
                    durable.set_step(attempt);
                }
                let previous = staged.predecessor();
                match staged
                    .run(
                        step.clone(),
                        super::modeling::assessment::Obligations::Final,
                        run_id,
                        attempt,
                        previous,
                        &cancel,
                    )
                    .await
                {
                    Ok(result) => {
                        let accepted = result.accepted;
                        results.push(result);
                        if !accepted && !continue_independent {
                            break;
                        }
                    }
                    Err(error) => {
                        failure = Some((attempt, error));
                        break;
                    }
                }
            }
            staged.close().await;
            let (report, modeling_failure) = match failure {
                Some((step, error)) => (
                    Err(error),
                    Some(ModelingFailure {
                        completed: results,
                        step: Some(step),
                    }),
                ),
                None => (Ok(RunReport::Modeling(results)), None),
            };
            let cancelled = cancel.token().is_cancelled();
            let mut result = RunResult::joined(run_id, runtime, request, None, report);
            result.modeling_continue_independent = continue_independent;
            result.modeling_failure = modeling_failure;
            let result = result.finished(durable, cancelled).await;
            sender.send_replace(Some(Arc::new(result)));
        }, move || forwarding.cancel(), parent)));
        Ok(RunHandle::staged(
            checks,
            receiver,
            progress,
            run_id,
            attempt_id,
            canonical_keys,
        ))

        })
    }
}

#[cfg(all(test, feature = "solver-kinsol", feature = "solver-root-isolation"))]
mod modeling_failure_tests {
    use super::*;
    use crate::workflow::tests as fixture;
    use pse_relations::{
        columnar::RelationRow,
        generated::runtime::{solve_runs, solve_strategy_events, solve_strategy_products},
    };

    #[tokio::test]
    async fn failed_modeling_sequence_publishes_completed_products_and_failed_attempt_trace() {
        let runtime = fixture::runtime_on(
            512 << 20,
            crate::math::MathPolicy {
                worker_bytes: 128 << 20,
                workspace_bytes: 128 << 20,
                foreign_bytes: 32 << 20,
                ..Default::default()
            },
        );
        let declarations = pse_authoring::language::parse(
            "package p {def Root {param p:Scalar=2;implicit a {var y:Scalar;eq ey:y==p+1;annotation start y(2.5);annotation bounds y(1,8);}realize ra on a using nested;implicit b {var z:Scalar;eq ez:z==2*p;annotation start z(3.5);annotation bounds z(1,9);}realize rb on b using nested;}}",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named, Default::default()).unwrap();
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, fixture::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let mut solver = fixture::profile();
        solver.presolve = pse_backend_native::presolve::Policy::Off;
        let good = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                Default::default(),
                pse_kernels::DerivativeOrder::First,
                fixture::compiler_profile(),
                solver.clone(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        solver.composition.limits = Some(pse_model::strategy::WorkLimits {
            attempts: 4,
            evaluations: Some(0),
            iterations: None,
            factorizations: None,
            proof_steps: None,
        });
        let bad = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                Default::default(),
                pse_kernels::DerivativeOrder::First,
                fixture::compiler_profile(),
                solver,
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        let result = runtime
            .start_modeling(vec![good.clone(), bad, good], false, &cancel)
            .await
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert!(
            result.report().is_err(),
            "zero evaluation cap must stop the failed target"
        );
        assert!(!result.usable());
        assert!(result.modeling_result(0).unwrap().accepted);
        let trace = result.modeling_error(1).unwrap().strategy_trace().unwrap();
        assert!(!trace.events.is_empty());
        let tables = result.tables().unwrap();
        let events =
            solve_strategy_events::Row::rows(&tables[&solve_strategy_events::RELATION_ID]).unwrap();
        assert!(events.iter().any(|row| row.step == 0));
        assert!(events.iter().any(|row| row.step == 1));
        assert!(events.iter().all(|row| row.step < 2));
        let products =
            solve_strategy_products::Row::rows(&tables[&solve_strategy_products::RELATION_ID])
                .unwrap();
        assert!(
            products
                .iter()
                .any(|row| row.step == 0 && row.derivative_order == 0)
        );
        assert!(products.iter().all(|row| row.step < 2));
        let runs = solve_runs::Row::rows(&tables[&solve_runs::RELATION_ID]).unwrap();
        assert_eq!(
            runs[0].state,
            pse_model::generated::enums::NativeRunState::ConstantEvaluation
        );
        assert!(runs[0].error.is_none());
        assert_eq!(
            runs[1].state,
            pse_model::generated::enums::NativeRunState::Rejected
        );
        assert!(runs[1].error.is_some());
        assert_eq!(
            runs[2].state,
            pse_model::generated::enums::NativeRunState::Unattempted
        );
        assert!(runs[2].error.is_none());
    }
}

#[cfg(test)]
mod cancellation_tests {
    #[test]
    fn completed_handle_drop_does_not_fabricate_diagnostic_loss() {
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("completed.run");
        let context = operation.context();
        let reader = operation.reader();
        let lease = super::Lease(
            pse_columnar::flight::FlightCancellation::default(),
            None,
            Some(context),
        );
        operation.finish(pse_diagnostics::observation::Outcome::Completed);
        drop(lease);
        let snapshot = reader.snapshot(std::time::Duration::from_secs(60));
        assert!(!snapshot.coverage.curtailed);
        assert_eq!(snapshot.coverage.dropped_updates, 0);
        assert!(
            !snapshot
                .metrics
                .iter()
                .any(|metric| metric.name == "run.stop_requested")
        );
    }
    #[tokio::test]
    async fn replay_parent_cancellation_reaches_pending_supervisor_and_keeps_drain_owned() {
        let parent = crate::CancelSource::new();
        let native = pse_columnar::flight::FlightCancellation::default();
        let flight = native.clone();
        let (observed, cancellation) = tokio::sync::oneshot::channel();
        let (release, drain) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(super::supervised_with_cancel(
            async move {
                flight.cancelled().await;
                observed.send(()).unwrap();
                drain.await.unwrap();
            },
            move || native.cancel(),
            Some(parent.clone()),
        ));
        parent.cancel();
        cancellation.await.unwrap();
        assert!(
            !task.is_finished(),
            "the supervisor must own drain after cancellation"
        );
        release.send(()).unwrap();
        task.await.unwrap();
    }
}
