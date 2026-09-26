// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Waiter cancellation never takes native ownership away from the existing supervisor.
use super::{ModelRevision, Runtime, WorkflowError, contract};
use crate::math::{
    MathRuntimeError,
    solves::{PreparedSolve, SequenceReport, SolveSequence, SolverProfile},
};
use pse_backend_native::{
    routing::Route,
    solve::{Event, Progress},
};
use pse_columnar::flight::FlightCancellation;
use pse_ids::SemanticId;
use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};

/// Immutable prepared representation and original semantic metadata.
#[derive(Clone, Debug)]
pub struct PreparedCase {
    pub(crate) revision: ModelRevision,
    pub(crate) case: SemanticId,
    pub(crate) profile: SolverProfile,
    pub(crate) solve: PreparedSolve,
    pub(crate) preparation: crate::math::Preparation,
}
impl ModelRevision {
    /// Prepare the selected immutable revision through the shared Salsa compiler.
    /// Coefficient projection follows admitted facts; callers do not select a compiler path.
    pub async fn prepare(
        &self,
        case: SemanticId,
        profile: SolverProfile,
        compiler: pse_compiler::workspace::Profile,
        cancel: &crate::CancelSource,
    ) -> Result<PreparedCase, WorkflowError> {
        let inputs = self
            .0
            .cases
            .get(&case)
            .ok_or_else(|| contract("unknown selected case"))?;
        let order = if matches!(
            profile.controls.hessian,
            pse_backend_native::solve::HessianMode::Exact
        ) {
            pse_kernels::DerivativeOrder::Second
        } else {
            pse_kernels::DerivativeOrder::First
        };
        let preparation = self
            .0
            .runtime
            .native()
            .prepare_revision(
                self.0.workspace.clone(),
                inputs.as_ref().clone(),
                case,
                order,
                compiler,
                cancel,
            )
            .await?;
        let providers = self
            .0
            .providers
            .values()
            .map(|p| (p.registration.spec().key(), p.registration.clone()))
            .collect();
        let mut declarations: Vec<_> = self
            .0
            .resolved_sources
            .numerics
            .iter()
            .filter(|r| r.case_id.is_none_or(|id| id == case))
            .map(|r| pse_math::numerics::SourcedRequirement {
                source: if r.case_id.is_some() {
                    pse_model::generated::enums::NumericalSource::Case
                } else {
                    pse_model::generated::enums::NumericalSource::Model
                },
                declaration: r.clone(),
            })
            .collect();
        let mut targets = Vec::new();
        for balance in self
            .0
            .resolved_sources
            .balances
            .iter()
            .filter(|b| b.case_id == case)
        {
            let quantity = balance.quantity_id.into();
            targets.push(pse_math::numerics::TargetSpec {
                id: balance.balance_id,
                kind: pse_model::generated::enums::NumericalTarget::Closure,
                quantity,
                unit: self
                    .0
                    .physical
                    .quantities
                    .quantity_type(quantity)
                    .map_err(|e| contract(e.to_string()))?
                    .canonical_unit,
                integer: false,
                declared_tolerance: Some(balance.tolerance),
            });
        }
        let mut property_targets = preparation
            .compiled()
            .plan
            .numerical_targets(&self.0.physical.quantities)
            .map_err(super::math)?;
        property_targets.extend(targets.iter().cloned());
        declarations.extend(self.property_numerics(case, &property_targets)?);
        let solve = self
            .0
            .runtime
            .native()
            .prepare_solve(
                preparation.clone(),
                pse_math::binding::CaseValues {
                    scalars: inputs.values.clone(),
                },
                providers,
                profile.clone(),
                None,
                crate::math::solves::NumericalInputs {
                    declarations,
                    targets,
                },
            )
            .await?;
        Ok(PreparedCase {
            revision: self.clone(),
            case,
            profile,
            solve,
            preparation,
        })
    }
}
impl PreparedCase {
    /// Select a primal-only authored seed by semantic ID, independently of allocation reuse.
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
    /// Select an owned compatible seed independently of native allocation reuse.
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
    /// Contextual backend alternatives assessed during this immutable preparation.
    pub fn eligibility(&self) -> &[pse_backend_native::routing::Eligibility] {
        self.solve.eligibility()
    }
    /// Mathematically admitted backend route, before native execution.
    pub fn route(&self) -> Route {
        self.solve.route()
    }
    /// The original compiler structure, including semantic IDs and decomposition.
    pub fn compiled(&self) -> &pse_compiler::workspace::PreparedCase {
        self.preparation.compiled()
    }
    /// Start a finite native attempt; all later waiters observe its one terminal result.
    pub fn start(&self) -> Result<RunHandle, WorkflowError> {
        self.revision.0.runtime.start(vec![self.clone()], false)
    }
}
/// Public cancellation lease. Dropping the last public handle requests cancellation;
/// the supervisor retains the actual native handle until its join completes.
#[derive(Debug)]
struct Lease(FlightCancellation);
impl Drop for Lease {
    fn drop(&mut self) {
        self.0.cancel();
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
        self.lease.0.cancel();
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
    /// Existing native algebraic solve/sequence report.
    Solves(SequenceReport),
    /// A completed or partial native integration.
    Simulation(Box<pse_backend_native::dynamics::Report>),
    /// Native steady/transient parameter fitting.
    Fit(Box<super::FitReport>),
}
/// Immutable request representation, with no mutable native objects.
#[derive(Clone, Debug)]
pub enum RunRequest {
    /// Finite already-prepared algebraic cases.
    Solves(Vec<PreparedCase>),
    /// One already-prepared physical simulation.
    Simulation(Box<super::PreparedSimulation>),
    /// Compiled shared-parameter experiments.
    Fit(Box<super::PreparedFit>),
}
impl RunRequest {
    pub(crate) fn revisions(&self) -> Vec<&ModelRevision> {
        match self {
            Self::Solves(s) => s.iter().map(|s| &s.revision).collect(),
            Self::Simulation(s) => vec![&s.revision],
            Self::Fit(f) => vec![&f.problem.revision],
        }
    }
}
/// Immutable joined outcome. Table encoding/publication never invokes a solver again.
#[derive(Debug)]
pub struct RunResult {
    /// Unique execution identity, not mathematical content identity.
    pub run_id: SemanticId,
    pub(crate) runtime: Runtime,
    pub(crate) request: RunRequest,
    pub(crate) _owner: Option<Arc<pse_columnar::AllocationLease>>,
    pub(crate) report: Result<RunReport, Arc<MathRuntimeError>>,
    pub(crate) physical:
        Result<Vec<pse_relations::generated::runtime::physical_checks::Row>, Arc<WorkflowError>>,
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
    /// Every requested candidate passed the requested usability policy.
    pub fn usable(&self) -> bool {
        !self.assessments.is_empty()
            && self
                .assessments
                .iter()
                .all(|a| a.usability != pse_model::generated::enums::CandidateUse::Unusable)
    }
    fn completed(mut self) -> Self {
        match &mut self.report {
            Ok(RunReport::Solves(r)) => {
                for (attempt, outcome) in r.outcomes.iter_mut().enumerate() {
                    if let crate::math::solves::Outcome::Native(r) = outcome
                        && let Some(seed) = &mut r.warm_start
                    {
                        seed.origin = Some(pse_backend_native::solve::SeedOrigin {
                            run: Some(self.run_id),
                            attempt,
                        });
                    }
                }
            }
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
        self.physical = self.evaluate_physical_checks().map_err(Arc::new);
        self.assessments = self.assess_candidates();
        self.completion = self.capture_completion().map_err(Arc::new);
        self
    }

    /// Full typed reports and backend-specific metrics; errors preserve their original causes.
    pub fn report(&self) -> Result<&RunReport, &MathRuntimeError> {
        self.report.as_ref().map_err(AsRef::as_ref)
    }
    /// Original immutable declarations for each requested step, including unattempted steps.
    pub fn request(&self) -> &RunRequest {
        &self.request
    }
}
impl Runtime {
    /// Start a bounded sequence of already prepared native cases under one lifecycle.
    pub fn start(
        &self,
        steps: Vec<PreparedCase>,
        continue_independent: bool,
    ) -> Result<RunHandle, WorkflowError> {
        if steps.is_empty()
            || steps
                .iter()
                .any(|p| !Arc::ptr_eq(&p.revision.0.runtime.shared, &self.shared))
        {
            return Err(contract("empty sequence or mixed runtime ownership"));
        }
        // Publication has one complete declaration per model ID. Do not silently
        // choose a revision when a batch intentionally mixes revisions of a model.
        let mut revisions = BTreeMap::new();
        for step in &steps {
            if revisions
                .insert(step.revision.0.row.model_id, step.revision.identity())
                .is_some_and(|v| v != step.revision.identity())
            {
                return Err(contract(
                    "sequence mixes revisions of one model; use separate runs",
                ));
            }
        }
        let handle = self.native().solve(SolveSequence {
            steps: steps.iter().map(|p| p.solve.clone()).collect(),
            continue_independent,
            result_limit: steps.len(),
        })?;
        let lease = Arc::new(Lease(handle.cancellation()));
        let progress = handle.progress_source();
        let (sender, receiver) = tokio::sync::watch::channel(None);
        let runtime = self.clone();
        let run_id = pse_authoring::ids::uuid_v7();
        tokio::spawn(async move {
            let report = handle
                .finish()
                .await
                .map(RunReport::Solves)
                .map_err(Arc::new);
            let result = Arc::new(
                RunResult {
                    run_id,
                    runtime,
                    request: RunRequest::Solves(steps),
                    _owner: None,
                    report,
                    physical: Ok(vec![]),
                    assessments: vec![],
                    completion: Err(Arc::new(contract("completion has not been captured"))),
                    batches: OnceLock::new(),
                }
                .completed(),
            );
            sender.send_replace(Some(result));
        });
        Ok(RunHandle {
            lease,
            receiver,
            progress,
        })
    }
}

impl super::PreparedSimulation {
    /// Start one bounded simulation under the same cancellation and completion owner.
    pub fn start(&self) -> Result<RunHandle, WorkflowError> {
        let runtime = self.revision.0.runtime.clone();
        let prepared = self.clone();
        let handle = runtime
            .native()
            .submit(1, self.bytes, move |flag, progress| {
                #[cfg(feature = "solver-diffsol")]
                {
                    let mut worker = prepared.worker(flag.clone())?;
                    let report = pse_backend_native::dynamics::integrate_with_progress(
                        &mut worker,
                        &prepared.profile,
                        &prepared.parameters,
                        flag,
                        progress,
                    )?;
                    let retained = report
                        .numeric_bytes()
                        .checked_add(4 << 20)
                        .ok_or(MathRuntimeError::Limit("trajectory result extent"))?;
                    Ok((RunReport::Simulation(Box::new(report)), retained))
                }
                #[cfg(not(feature = "solver-diffsol"))]
                {
                    let _ = (prepared, flag, progress);
                    Err(MathRuntimeError::Infrastructure(
                        "Diffsol not linked".into(),
                    ))
                }
            })?;
        let lease = Arc::new(Lease(handle.cancellation()));
        let progress = handle.progress_source();
        let (sender, receiver) = tokio::sync::watch::channel(None);
        let request = RunRequest::Simulation(Box::new(self.clone()));
        let run_id = pse_authoring::ids::uuid_v7();
        tokio::spawn(async move {
            let (report, owner) = match handle.finish().await {
                Ok((r, o)) => (Ok(r), Some(o)),
                Err(e) => (Err(Arc::new(e)), None),
            };
            sender.send_replace(Some(Arc::new(
                RunResult {
                    run_id,
                    runtime,
                    request,
                    _owner: owner,
                    report,
                    physical: Ok(vec![]),
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

impl super::PreparedFit {
    /// Start one native fitting attempt under the existing joined job lifecycle.
    pub fn start(&self) -> Result<RunHandle, WorkflowError> {
        let runtime = self.problem.revision.0.runtime.clone();
        let prepared = self.clone();
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
                let report = prepared.execute(flag, progress)?;
                let retained = report
                    .numeric_bytes()
                    .checked_add(allowance)
                    .ok_or(MathRuntimeError::Limit("fit result extent"))?;
                Ok((RunReport::Fit(Box::new(report)), retained))
            },
        )?;
        let lease = Arc::new(Lease(handle.cancellation()));
        let progress = handle.progress_source();
        let (sender, receiver) = tokio::sync::watch::channel(None);
        let request = RunRequest::Fit(Box::new(self.clone()));
        let run_id = pse_authoring::ids::uuid_v7();
        tokio::spawn(async move {
            let (report, owner) = match handle.finish().await {
                Ok((r, o)) => (Ok(r), Some(o)),
                Err(e) => (Err(Arc::new(e)), None),
            };
            sender.send_replace(Some(Arc::new(
                RunResult {
                    run_id,
                    runtime,
                    request,
                    _owner: owner,
                    report,
                    physical: Ok(vec![]),
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
