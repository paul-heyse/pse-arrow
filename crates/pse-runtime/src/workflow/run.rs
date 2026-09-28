// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Waiter cancellation never takes native ownership away from the existing supervisor.
//!
//! Every run is supervised under its runtime's durability class (ADR-0112 Outcome 16). An
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
use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};

/// Public cancellation lease. Dropping the last public handle requests cancellation;
/// the supervisor retains the actual native handle until its join completes.
#[derive(Debug)]
struct Lease(FlightCancellation, Option<crate::CancelSource>);
impl Lease {
    fn cancel(&self) {
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
    lease: Arc<Lease>,
    receiver: tokio::sync::watch::Receiver<Option<Arc<RunResult>>>,
    progress: Arc<Progress>,
    run_id: SemanticId,
    attempt_id: Option<pse_operations::attempts::AttemptId>,
}
impl RunHandle {
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
    /// The run identity, minted before any effect.
    pub const fn run_id(&self) -> SemanticId {
        self.run_id
    }
    /// The durable attempt of this run, minted before any effect; `None` when ephemeral.
    pub const fn attempt_id(&self) -> Option<pse_operations::attempts::AttemptId> {
        self.attempt_id
    }
}
/// Which stored seed a preparation starts from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoredStart {
    /// The newest seed stored for the preparation's coordinates and backend.
    Latest,
    /// This stored solution.
    Solution(pse_operations::solutions::SolutionId),
}
/// Mathematical report variants share one joined public job lifecycle.
#[derive(Debug)]
pub enum RunReport {
    /// Authored steady or simultaneous solve with original-model qualification.
    Modeling(Vec<super::ModelingResult>),
    /// A completed or partial native integration.
    Simulation(Box<super::ModelingTrajectory>),
    /// Native steady/transient parameter fitting.
    Fit(Box<super::FitReport>),
}
/// Immutable request representation, with no mutable native objects.
#[derive(Clone, Debug)]
pub enum RunRequest {
    /// One immutable authored algebraic case.
    Modeling(Vec<super::ModelingSolvePreparation>),
    /// One already-prepared physical simulation.
    Simulation(Box<super::ModelingSimulation>),
    /// Compiled shared-parameter experiments.
    Fit(Box<super::PreparedFit>),
}
/// Immutable joined outcome. Table encoding/publication never invokes a solver again.
#[derive(Debug)]
pub struct RunResult {
    /// Unique execution identity, not mathematical content identity.
    pub run_id: SemanticId,
    pub(crate) runtime: Runtime,
    pub(crate) request: RunRequest,
    pub(crate) _owner: Option<Arc<pse_columnar::AllocationLease>>,
    pub(crate) report: Result<RunReport, Arc<WorkflowError>>,
    pub(crate) assessments: Vec<pse_relations::generated::runtime::candidate_assessments::Row>,
    pub(crate) completion: Result<super::completion::Completion, Arc<WorkflowError>>,
    pub(crate) batches: OnceLock<
        Result<
            BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>,
            Arc<WorkflowError>,
        >,
    >,
    /// What the run recorded durably; ephemeral runs record nothing.
    pub(crate) durability: RunDurability,
}
impl RunResult {
    /// A joined result before completion capture and durable recording.
    fn joined(
        run_id: SemanticId,
        runtime: Runtime,
        request: RunRequest,
        owner: Option<Arc<pse_columnar::AllocationLease>>,
        report: Result<RunReport, Arc<WorkflowError>>,
    ) -> Self {
        Self {
            run_id,
            runtime,
            request,
            _owner: owner,
            report,
            assessments: vec![],
            completion: Err(Arc::new(contract("completion has not been captured"))),
            batches: OnceLock::new(),
            durability: RunDurability::Ephemeral,
        }
    }
    /// Final completion-owned scientific assessments; exporting tables cannot change them.
    pub fn assessments(&self) -> &[pse_relations::generated::runtime::candidate_assessments::Row] {
        &self.assessments
    }
    /// Every requested candidate is a result under the requested usability policy.
    /// Seed-only and diagnostic-only candidates are never results (ADR-0106).
    pub fn usable(&self) -> bool {
        use pse_model::generated::enums::CandidateUse;
        !self.assessments.is_empty()
            && self.assessments.iter().all(|a| {
                matches!(
                    a.usability,
                    CandidateUse::Usable | CandidateUse::QualifiedUnclosed
                )
            })
    }
    fn completed(mut self) -> Self {
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
        self
    }
    /// Capture completion, then record the attempt's end durably when the run is durable.
    async fn finished(self, durable: Option<DurableAttempt>, cancelled: bool) -> Self {
        let mut result = self.completed();
        if let Some(attempt) = durable {
            result.durability =
                RunDurability::Durable(Box::new(attempt.finish(&result, cancelled).await));
        }
        result
    }
    /// A durable run that failed before it ran: record why, and publish the failure.
    async fn refused(mut self, durable: Option<DurableAttempt>) -> Self {
        self = self.completed();
        if let (Some(attempt), Err(error)) = (durable, &self.report) {
            let error = error.clone();
            self.durability = RunDurability::Durable(Box::new(attempt.abandon(&error).await));
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
}

/// The durable attempt of a run under `runtime`'s class, or none when ephemeral. A worker
/// passes the attempt it claimed with its job, already under its lease.
fn attempt_for(
    runtime: &Runtime,
    given: Option<DurableAttempt>,
) -> Result<Option<DurableAttempt>, WorkflowError> {
    match (&runtime.durability, given) {
        (Durability::Ephemeral, None) => Ok(None),
        (Durability::Ephemeral, Some(_)) => {
            Err(contract("a claimed job runs only on a durable runtime"))
        }
        (Durability::Durable(operations), None) => Ok(Some(DurableAttempt::new(operations))),
        (Durability::Durable(_), Some(attempt)) => Ok(Some(attempt)),
    }
}
/// The run's event stream: bounded in memory and, for a durable run, tapped by its store
/// stream, which keeps every event.
fn progress_for(history: usize, durable: Option<&DurableAttempt>) -> Arc<Progress> {
    Arc::new(match durable {
        Some(attempt) => Progress::tapped(history, attempt.tap()),
        None => Progress::new(history),
    })
}
/// Register a durable attempt (planned, queued), wait for native admission, and enter
/// running under its lease; `admission` resolves once native admission holds.
async fn admit<T>(
    durable: &mut Option<DurableAttempt>,
    run_id: SemanticId,
    request: &RunRequest,
    admission: impl Future<Output = Result<T, WorkflowError>>,
    cancel: Canceller,
) -> Result<T, WorkflowError> {
    if let Some(attempt) = durable.as_ref() {
        attempt.register(run_id, request).await?;
    }
    let admitted = admission.await?;
    if let Some(attempt) = durable.as_mut() {
        attempt.start(cancel).await?;
    }
    Ok(admitted)
}
/// Submit a native operation under the run's durability: an ephemeral run is refused when
/// the service is full; a durable run queues for admission and is running once admitted.
fn submission(
    cancel: &FlightCancellation,
    progress: &Arc<Progress>,
    durable: bool,
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
            queue: durable,
            admitted,
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

impl super::ModelingSimulation {
    /// Start the authored simulation through the shared joined run and publication lifecycle.
    pub fn start(&self) -> Result<RunHandle, WorkflowError> {
        let runtime = self.runtime.clone();
        let run_id = pse_authoring::ids::uuid_v7();
        let mut durable = attempt_for(&runtime, None)?;
        let progress = progress_for(256, durable.as_ref());
        let cancel = FlightCancellation::default();
        let (submission, signal) = submission(&cancel, &progress, durable.is_some());
        // An ephemeral submission is admitted or refused here; a durable one queues.
        let handle = self.submit(run_id, submission)?;
        let lease = Arc::new(Lease(cancel.clone(), None));
        let attempt_id = durable.as_ref().map(DurableAttempt::attempt_id);
        let (sender, receiver) = tokio::sync::watch::channel(None);
        let prepared = self.clone();
        tokio::spawn(async move {
            let request = RunRequest::Simulation(Box::new(prepared.clone()));
            let stop = cancel.clone();
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
                let refused =
                    RunResult::joined(run_id, runtime, request, None, Err(Arc::new(error)))
                        .refused(durable)
                        .await;
                sender.send_replace(Some(Arc::new(refused)));
                return;
            }
            let report = match handle.finish().await {
                Ok(((report, checks), owner)) => Ok(RunReport::Simulation(Box::new(
                    prepared.finish(run_id, report, checks, owner),
                ))),
                Err(error) => Err(Arc::new(WorkflowError::Math(error))),
            };
            let cancelled = cancel.flag().load(std::sync::atomic::Ordering::Acquire);
            let result = RunResult::joined(run_id, runtime, request, None, report)
                .finished(durable, cancelled)
                .await;
            sender.send_replace(Some(Arc::new(result)));
        });
        Ok(RunHandle {
            lease,
            receiver,
            progress,
            run_id,
            attempt_id,
        })
    }
}

impl super::PreparedFit {
    /// Start one native fitting attempt under the existing joined job lifecycle.
    pub fn start(&self) -> Result<RunHandle, WorkflowError> {
        let runtime = self.problem.runtime.clone();
        let prepared = self.clone();
        let run_id = pse_authoring::ids::uuid_v7();
        let mut durable = attempt_for(&runtime, None)?;
        let progress = progress_for(256, durable.as_ref());
        let cancel = FlightCancellation::default();
        let (submission, signal) = submission(&cancel, &progress, durable.is_some());
        let handle = runtime.native().submit_with(
            self.problem.profile.solver.controls.threads,
            self.problem.bytes,
            submission,
            move |flag, progress| {
                let allowance = prepared
                    .problem
                    .profile
                    .solver
                    .controls
                    .report_allowance()?;
                let report = prepared.execute(run_id, flag, progress)?;
                let retained = report
                    .numeric_bytes()
                    .checked_add(allowance)
                    .ok_or(MathRuntimeError::Limit("fit result extent"))?;
                Ok((RunReport::Fit(Box::new(report)), retained))
            },
        )?;
        let lease = Arc::new(Lease(cancel.clone(), None));
        let attempt_id = durable.as_ref().map(DurableAttempt::attempt_id);
        let (sender, receiver) = tokio::sync::watch::channel(None);
        let request = RunRequest::Fit(Box::new(self.clone()));
        tokio::spawn(async move {
            let stop = cancel.clone();
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
                let refused =
                    RunResult::joined(run_id, runtime, request, None, Err(Arc::new(error)))
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
        });
        Ok(RunHandle {
            lease,
            receiver,
            progress,
            run_id,
            attempt_id,
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
    /// Start from a seed in the operational store's solution store (ADR-0112 Outcome 17):
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
        let solutions = operations.store().solutions();
        let stored = match which {
            StoredStart::Latest => {
                let preparation = self
                    .solve
                    .seed_preparation_identity()
                    .ok_or_else(|| contract("a constant evaluation consumes no stored seed"))?;
                solutions
                    .latest_compatible(&target.layout, &preparation, target.backend)
                    .await?
                    .ok_or_else(|| {
                        contract(
                            "no stored seed matches this preparation's coordinates and backend",
                        )
                    })?
            }
            StoredStart::Solution(id) => solutions.get(id).await?.ok_or_else(|| {
                WorkflowError::Operations(pse_operations::OperationsError::NotFound {
                    entity: "stored solution",
                    id: id.to_string(),
                })
            })?,
        };
        let solution = stored.solution_id.as_id();
        let columns: Vec<SemanticId> = self.model.case.compiled().plan.columns().to_vec();
        let mut seeded = self.with_start(super::durable::warm_start(&stored)?)?;
        for column in columns {
            seeded
                .starts
                .insert(column, super::StartSource::Stored { solution });
        }
        Ok(seeded)
    }
    /// Start one authored algebraic run: a one-step authored sequence.
    pub fn start(&self) -> Result<RunHandle, WorkflowError> {
        self.source.runtime.launch(vec![self.clone()], false, None)
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
        self.check_sequence(&steps)?;
        if cancel.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled.into());
        }
        self.launch(steps, continue_independent, None)
    }
    /// Run a claimed job's prepared steps as its attempt, which the worker already holds
    /// under its lease; the attempt ends through the job's retry policy.
    pub(crate) fn start_attempt(
        &self,
        steps: Vec<super::ModelingSolvePreparation>,
        attempt: DurableAttempt,
    ) -> Result<RunHandle, WorkflowError> {
        self.check_sequence(&steps)?;
        self.launch(steps, false, Some(attempt))
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
            for row in step.source.declarations() {
                if declarations
                    .insert(row.declaration_id, row)
                    .is_some_and(|old| old != row)
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
    /// An ephemeral run opens its session now and is refused when none is free. A durable
    /// run registers its attempt, queues for a session, and runs under its lease.
    fn launch(
        &self,
        steps: Vec<super::ModelingSolvePreparation>,
        continue_independent: bool,
        attempt: Option<DurableAttempt>,
    ) -> Result<RunHandle, WorkflowError> {
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
        let cancel = crate::CancelSource::new();
        let lease = Arc::new(Lease(FlightCancellation::default(), Some(cancel.clone())));
        let (sender, receiver) = tokio::sync::watch::channel(None);
        let runtime = self.clone();
        let run_id = pse_authoring::ids::uuid_v7();
        let attempt_id = durable.as_ref().map(DurableAttempt::attempt_id);
        let shared = progress.clone();
        tokio::spawn(async move {
            let request = RunRequest::Modeling(steps.clone());
            let stop = cancel.clone();
            let session = async {
                match opened {
                    Some(staged) => Ok(staged),
                    None => tokio::select! {
                        staged=super::staged::Staged::open_queued(&runtime,Some(shared))=>staged,
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
                        failure = Some(error);
                        break;
                    }
                }
            }
            staged.close().await;
            let report = match failure {
                Some(error) => Err(error),
                None => Ok(RunReport::Modeling(results)),
            };
            let cancelled = cancel.token().is_cancelled();
            let result = RunResult::joined(run_id, runtime, request, None, report)
                .finished(durable, cancelled)
                .await;
            sender.send_replace(Some(Arc::new(result)));
        });
        Ok(RunHandle {
            lease,
            receiver,
            progress,
            run_id,
            attempt_id,
        })
    }
}
