// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Waiter cancellation never takes native ownership away from the existing supervisor.
use super::{Runtime, WorkflowError, contract};
use crate::math::MathRuntimeError;
use pse_backend_native::{
    solve::{Event, Progress},
};
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
        if let Some(checks) = &self.1 { checks.cancel(); }
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
    /// Bounded native events and actual dropped-event count.
    pub fn progress(&self) -> (Vec<Event>, u64) {
        self.progress.snapshot()
    }
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
}
impl RunResult {
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
        match &mut self.report {
            Ok(RunReport::Fit(r)) => {
                if let Some(seed) = r.solve.as_mut().and_then(|s| s.warm_start.as_mut()) {
                    seed.origin = Some(pse_backend_native::solve::SeedOrigin {
                        run: Some(self.run_id),
                        attempt: 0,
                    });
                }
            }
            _ => {}
        }
        self.assessments = self.assess_candidates();
        self.completion = self.capture_completion().map_err(Arc::new);
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
}
impl super::ModelingSimulation {
    /// Start the authored simulation through the shared joined run and publication lifecycle.
    pub fn start(&self) -> Result<RunHandle, WorkflowError> {
        let runtime=self.runtime.clone();
        let run_id=pse_authoring::ids::uuid_v7();
        let handle=self.submit(run_id)?;
        let lease=Arc::new(Lease(handle.cancellation(), None));
        let progress=handle.progress_source();
        let (sender,receiver)=tokio::sync::watch::channel(None);
        let prepared=self.clone();
        tokio::spawn(async move {
            let report=match handle.finish().await {
                Ok(((report,checks),owner))=>Ok(RunReport::Simulation(Box::new(prepared.finish(run_id,report,checks,owner)))),
                Err(error)=>Err(Arc::new(WorkflowError::Math(error))),
            };
            sender.send_replace(Some(Arc::new(RunResult {
                run_id,runtime,request:RunRequest::Simulation(Box::new(prepared)),
                _owner:None,report,assessments:vec![],
                completion:Err(Arc::new(contract("completion has not been captured"))),batches:OnceLock::new(),
            }.completed())));
        });
        Ok(RunHandle{lease,receiver,progress})
    }
}

impl super::PreparedFit {
    /// Start one native fitting attempt under the existing joined job lifecycle.
    pub fn start(&self) -> Result<RunHandle, WorkflowError> {
        let runtime = self.problem.runtime.clone();
        let prepared = self.clone();
        let run_id = pse_authoring::ids::uuid_v7();
        let handle = runtime.native().submit(
            self.problem.profile.solver.controls.threads,
            self.problem.bytes,
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
        let lease = Arc::new(Lease(handle.cancellation(), None));
        let progress = handle.progress_source();
        let (sender, receiver) = tokio::sync::watch::channel(None);
        let request = RunRequest::Fit(Box::new(self.clone()));
        tokio::spawn(async move {
            let (report, owner) = match handle.finish().await {
                Ok((r, o)) => (Ok(r), Some(o)),
                Err(e) => (Err(Arc::new(WorkflowError::Math(e))), None),
            };
            sender.send_replace(Some(Arc::new(
                RunResult {
                    run_id,
                    runtime,
                    request,
                    _owner: owner,
                    report,
                    
                    assessments: vec![],
                    completion: Err(Arc::new(contract("completion has not been captured"))),
                    batches: OnceLock::new(),
                }
                .completed(),
            )));
        });
        Ok(RunHandle {
            lease,
            receiver,
            progress,
        })
    }
}

impl super::ModelingSolvePreparation {
    /// Retain a compatible native seed without retaining mutable solver state.
    pub fn with_start(mut self, seed: pse_backend_native::solve::WarmStart) -> Result<Self, WorkflowError> {
        self.solve = self.solve.with_start(seed).map_err(MathRuntimeError::from)?;
        self.profile.controls.start = pse_backend_native::solve::StartPolicy::Explicit;
        Ok(self)
    }
    /// Select the complete original free-coordinate primal seed by semantic identity.
    pub fn with_primal_start(mut self, values: BTreeMap<SemanticId,f64>) -> Result<Self, WorkflowError> {
        self.solve = self.solve.with_primal_start(values).map_err(MathRuntimeError::from)?;
        self.profile.controls.start = pse_backend_native::solve::StartPolicy::Explicit;
        Ok(self)
    }
    /// Start one authored algebraic run: a one-step authored sequence.
    pub fn start(&self) -> Result<RunHandle, WorkflowError> {
        self.source.runtime.launch(vec![self.clone()], false)
    }
}

impl Runtime {
    /// Start a finite authored sequence of prepared steps.
    pub async fn start_modeling(
        &self, steps: Vec<super::ModelingSolvePreparation>, continue_independent: bool,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle,WorkflowError> {
        if steps.is_empty() || steps.len()>4096 || steps.iter().any(|p|!Arc::ptr_eq(&p.source.runtime.shared,&self.shared)) {
            return Err(contract("empty, excessive or mixed-runtime authored sequence"));
        }
        let mut declarations=BTreeMap::new();
        for step in &steps {
            if step.source.physical.identity()!=steps[0].source.physical.identity() {return Err(contract("sequence needs one admitted physical context"));}
            for row in step.source.declarations() {
                if declarations.insert(row.declaration_id,row).is_some_and(|old|old!=row) {return Err(contract("sequence contains conflicting source declarations"));}
            }
        }
        if cancel.token().is_cancelled() {return Err(MathRuntimeError::Cancelled.into());}
        self.launch(steps,continue_independent)
    }
    /// Run prepared steps as one staged sequence (A6) on one native session: each step is
    /// assessed against the original model on its worker, a `PreviousAccepted` step starts
    /// from the previous step's seed when that step was accepted, and an unaccepted step
    /// ends the sequence unless the steps are independent. The handle shares one progress
    /// stream across steps; cancelling it stops the current step and joins native teardown.
    fn launch(
        &self, steps: Vec<super::ModelingSolvePreparation>, continue_independent: bool,
    ) -> Result<RunHandle,WorkflowError> {
        let history=steps.iter().map(|s|s.profile.controls.history).max().unwrap_or(0);
        let progress=Arc::new(Progress::new(history));
        let mut staged=super::staged::Staged::open(self,Some(progress.clone()))?;
        let cancel=crate::CancelSource::new();
        let lease=Arc::new(Lease(FlightCancellation::default(),Some(cancel.clone())));
        let (sender,receiver)=tokio::sync::watch::channel(None);
        let runtime=self.clone();
        tokio::spawn(async move {
            let run_id=pse_authoring::ids::uuid_v7();
            let mut results=Vec::new();
            let mut failure=None;
            for (attempt,step) in steps.iter().enumerate() {
                if cancel.token().is_cancelled() {break;}
                let previous=staged.predecessor();
                match staged.run(step.clone(),super::modeling::assessment::Obligations::Final,run_id,attempt,previous,&cancel).await {
                    Ok(result)=>{
                        let accepted=result.accepted;
                        results.push(result);
                        if !accepted && !continue_independent {break;}
                    }
                    Err(error)=>{failure=Some(error);break;}
                }
            }
            staged.close().await;
            let report=match failure {Some(error)=>Err(error),None=>Ok(RunReport::Modeling(results))};
            sender.send_replace(Some(Arc::new(RunResult{run_id,runtime,request:RunRequest::Modeling(steps),_owner:None,report,assessments:vec![],completion:Err(Arc::new(contract("completion has not been captured"))),batches:OnceLock::new()}.completed())));
        });
        Ok(RunHandle{lease,receiver,progress})
    }
}
