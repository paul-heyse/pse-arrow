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
    ExecutionServices, RunOutcome, RunReport, RunRequest, Runtime, WorkflowError, contract,
    deadline_limit,
    durable::{Canceller, Durability, DurableAttempt, RunDurability},
    run_outcome::PairedOperation,
};
use crate::math::{MathRuntimeError, Submission};
use futures_util::FutureExt;
use pse_backend_native::solve::{Event, Progress};
use pse_diagnostics::Cancellation;
use pse_ids::SemanticId;
use pse_model::generated::identities::RunId;
use std::{collections::BTreeMap, sync::Arc};

/// Existing native-submit ownership can be held at actual family worker entry by
/// isolated unit controls. No release-build observer or execution policy is added.
#[cfg(test)]
type FamilyWorkerEntry = Arc<dyn Fn(&std::sync::atomic::AtomicBool) + Send + Sync>;
#[cfg(test)]
tokio::task_local! { static FAMILY_WORKER_ENTRY: FamilyWorkerEntry; }
#[cfg(all(
    test,
    feature = "canonical-tests",
    any(feature = "solver-diffsol", feature = "solver-ipopt")
))]
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

/// Public cancellation lease. The library drop guard stops an immediate run when its
/// last caller leaves; durable work remains owned by its worker through native join.
#[derive(Debug)]
struct Lease {
    checks: crate::CancelSource,
    observation: Option<pse_diagnostics::observation::ObservationContext>,
    // Durable caller handles observe a worker-owned run; only an immediate caller
    // owns cancellation on departure. Explicit cancel remains available for both.
    _guard: Option<pse_columnar::resource::CancellationGuard>,
}
impl Lease {
    fn new(checks: crate::CancelSource, durable: bool) -> Self {
        Self {
            _guard: (!durable).then(|| checks.token().drop_guard()),
            checks,
            observation: pse_diagnostics::observation::current(),
        }
    }
    fn cancel(&self) {
        if let Some(context) = &self.observation
            && context.is_active()
        {
            context.boolean("run.stop_requested", true);
        }
        self.checks.cancel();
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
    /// Passive study observation borrows only telemetry, never a cancellation lease.
    pub(super) fn study_progress_source(&self) -> Option<(String, String, Arc<Progress>)> {
        self.canonical_keys
            .as_ref()
            .map(|(run, attempt)| (run.clone(), attempt.clone(), self.progress.clone()))
    }
    /// Passive current/cumulative/recent operation reader; dropping it never cancels this run.
    pub fn observations(&self) -> Option<pse_diagnostics::observation::OperationReader> {
        self.observation.clone()
    }

    /// Request stop; result ownership remains live until native teardown and join.
    pub fn cancel(&self) {
        self.lease.cancel();
    }
    #[cfg(test)]
    pub(in crate::workflow) fn cancellation_scope(&self) -> &crate::CancelSource {
        &self.lease.checks
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
    /// Copy the whole current native event even with history zero. The subsequent
    /// source head/loss read is independent; a racing publication can make it stale.
    pub fn current_progress(&self) -> super::ProgressCheckpointDocument {
        let current = self
            .progress
            .current()
            .map(super::ProgressEventDocument::from);
        let (generation, highest_issued_sequence) = self.progress.source_identity();
        let current_is_latest = current
            .as_ref()
            .and_then(|event| event.source.as_ref())
            .is_some_and(|source| {
                source.generation == generation && source.sequence == highest_issued_sequence
            });
        super::ProgressCheckpointDocument {
            version: pse_model::document::Version,
            current,
            generation,
            highest_issued_sequence,
            dropped: self.progress.dropped_count(),
            current_is_latest,
        }
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
/// Immutable joined outcome. Table encoding and retention never invoke a solver again.
#[derive(Debug)]
pub struct RunResult {
    /// Supervisor-owned joined evidence clock; absent when constructed outside a supervised invocation.
    pub(super) invocation: Option<Arc<crate::invocation::Recorder>>,
    pub(super) invocation_cancelled: bool,
    pub(super) invocation_timing_unavailable:
        Option<pse_diagnostics::observation::RecordAvailability>,
    pub(super) modeling_continue_independent: bool,
    pub(super) attempt_summary: std::sync::OnceLock<Arc<super::AttemptSummary>>,
    pub(super) modeling_origin: usize,
    pub(super) original_completion_captured: bool,
    pub(super) horizon_output: Option<Arc<super::HorizonOutcome>>,
    #[cfg(feature = "solver-diffsol")]
    pub(super) horizon_owners: Vec<Arc<pse_columnar::AllocationLease>>,
    pub(super) horizon_tightenings: Option<Vec<pse_modeling::DomainTightening>>,
    pub(super) horizon_unmerged: Option<Box<RunResult>>,
    /// The run: an execution identity, not mathematical content identity. A retried
    /// job's tries share it; each try is its own attempt.
    pub run_id: RunId,
    pub(crate) services: ExecutionServices,
    pub(crate) outcome: RunOutcome,
    pub(crate) _owner: Option<Arc<pse_columnar::AllocationLease>>,
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
        services: ExecutionServices,
        outcome: RunOutcome,
        owner: Option<Arc<pse_columnar::AllocationLease>>,
    ) -> Self {
        let result_validation = Arc::new(services.preparation().validation_context().new_scope());
        let projection_owner = super::result_export::ProjectionOwner::new(
            services.preparation(),
            result_validation.clone(),
        );
        Self {
            invocation: crate::invocation::current(),
            invocation_cancelled: false,
            invocation_timing_unavailable: crate::invocation::current_unavailability(),
            modeling_continue_independent: false,
            attempt_summary: Default::default(),
            result_validation,
            projection_owner,
            modeling_origin: 0,
            original_completion_captured: false,
            horizon_output: None,
            #[cfg(feature = "solver-diffsol")]
            horizon_owners: Vec::new(),
            horizon_tightenings: None,
            horizon_unmerged: None,
            run_id,
            services,
            outcome,
            _owner: owner,
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
        self.result_validation =
            Arc::new(self.services.preparation().validation_context().new_scope());
        self.projection_owner = super::result_export::ProjectionOwner::new(
            self.services.preparation(),
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
        self.completion
            .as_ref()
            .is_ok_and(super::Completion::permits_result)
    }
    pub(super) fn modeling_result(&self, step: usize) -> Option<&super::ModelingResult> {
        match self.report() {
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
            .then(|| self.error().map(AsRef::as_ref))
            .flatten()
    }
    pub(super) fn completed(mut self) -> Self {
        if self.original_completion_captured || self.horizon_unmerged.is_some() {
            return self;
        }
        if let RunOutcome::Fit { result: Ok(r), .. } = &mut self.outcome
            && let Some(seed) = r.solve.as_mut().and_then(|s| s.warm_start.as_mut())
        {
            seed.origin = Some(pse_backend_native::solve::SeedOrigin {
                run: Some(self.run_id),
                attempt: 0,
            });
        }
        let stage = crate::invocation::stage("original_qualification_and_completion");
        self.assessments = self.assess_candidates();
        self.completion = self.capture_completion().map_err(Arc::new);
        let qualified = self.completion.is_ok() && self.usable();
        if let Some(recorder) = &self.invocation {
            recorder.qualified(qualified);
        }
        stage.finish(if qualified {
            crate::invocation::InvocationOutcome::Completed
        } else {
            crate::invocation::InvocationOutcome::Failed
        });
        self.reset_result_projection();
        self
    }
    /// Capture completion, then record the attempt's end durably when the run is durable.
    pub(super) async fn finished(self, durable: Option<DurableAttempt>, cancelled: bool) -> Self {
        let _phase = pse_diagnostics::observation::phase("run.completion.and.publication");
        let mut result = self.completed();
        if let Some(recorder) = &result.invocation {
            recorder.terminal();
        }
        result.invocation_cancelled = cancelled;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.boolean("run.cancel_requested", cancelled);
            context.boolean("run.execution_failed", result.report().is_err());
            context.boolean("run.original_completion.failed", result.completion.is_err());
            context.source_ref("run.identity", result.run_id.as_id().to_string().as_bytes());
        }
        if let Some(mut attempt) = durable {
            // Every callsite has completed its scientific handle's owning join,
            // reset/teardown and original assessment before this handoff. The
            // attempt and caller's private source lane remain retained through ACK.
            let admission = attempt.begin_worker_publication().await;
            let publish = matches!(&admission, Ok(true));
            let preserve_scientific_failure =
                !publish && (result.report().is_err() || result.completion.is_err());
            let publication = crate::invocation::stage("required_durable_publication");
            let record = if publish {
                attempt.finish(&result, cancelled).await
            } else {
                // Keep the original failure when science already failed. An
                // unadmitted cancelled owner only settles its existing attempt;
                // it must not start a second payload publisher past capacity.
                let prior = result
                    .error()
                    .or_else(|| result.completion.as_ref().err())
                    .cloned();
                let interrupted = match admission {
                    Err(error) => Some(Arc::new(error)),
                    Ok(_) => None,
                };
                let error = prior
                    .clone()
                    .or_else(|| interrupted.clone())
                    .unwrap_or_else(|| Arc::new(MathRuntimeError::Cancelled.into()));
                if prior.is_none() {
                    result.fail_report(error.clone());
                    result.reset_result_projection();
                }
                if let Some(interrupted) = interrupted {
                    attempt.reject_worker_publication(interrupted).await
                } else {
                    result.invocation_cancelled = true;
                    if let Some(context) = pse_diagnostics::observation::current() {
                        context.boolean("run.cancel_requested", true);
                    }
                    attempt.abandon_cancelled(&error).await
                }
            };
            publication.finish(if record.attempt.is_ok() {
                crate::invocation::InvocationOutcome::Completed
            } else {
                crate::invocation::InvocationOutcome::Failed
            });
            if let Some(context) = pse_diagnostics::observation::current() {
                context.boolean("run.publication.failed", record.attempt.is_err());
            }
            if let Err(error) = &record.attempt
                && !preserve_scientific_failure
            {
                result.fail_report(error.clone());
                result.reset_result_projection();
            }
            result.durability = RunDurability::Durable(Arc::new(record));
        }
        if let Some(recorder) = &result.invocation {
            recorder.ready();
        }
        result
    }
    /// A durable run that failed before it ran: record why, and publish the failure.
    pub(super) async fn refused(mut self, durable: Option<DurableAttempt>) -> Self {
        self.invocation_cancelled = self.error().is_some_and(|error| {
            pse_diagnostics::TypedDiagnostic::diagnostic_code(error.as_ref())
                == Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled)
        });
        self = self.completed();
        if let Some(recorder) = &self.invocation {
            recorder.terminal();
        }
        if let (Some(attempt), Some(error)) = (durable, self.error()) {
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
        if let Some(recorder) = &self.invocation {
            recorder.ready();
        }
        self
    }

    /// Link a follow-up diagnostic to retained original facts without granting result permission.
    pub fn diagnostic_origin(
        &self,
        step: Option<usize>,
    ) -> Result<super::DiagnosticOrigin, WorkflowError> {
        let modeling = step.and_then(|step| self.modeling_result(step));
        if let Some(step) = step {
            let count = match self.request() {
                RunRequest::Modeling(request) => request.len(),
                _ => 0,
            };
            if step >= count {
                return Err(contract("diagnostic origin step outside original request"));
            }
        }
        let failure = match step {
            Some(step) => modeling
                .and_then(super::ModelingResult::diagnostic)
                .or_else(|| {
                    self.modeling_error(step)
                        .map(WorkflowError::boundary_diagnostic)
                }),
            None => self
                .error()
                .map(|error| error.boundary_diagnostic())
                .or_else(|| {
                    (!self.usable())
                        .then(|| self.completion.as_ref().ok()?.diagnostics.first().cloned())
                        .flatten()
                }),
        };
        Ok(super::DiagnosticOrigin {
            run_id: self.run_id,
            attempt_id: match &self.durability {
                RunDurability::Ephemeral => None,
                RunDurability::Durable(record) => Some(record.attempt_id),
            },
            step,
            native_attempt: modeling.map(|result| result.original_attempt),
            operation: self
                .invocation
                .as_ref()
                .and_then(|recorder| recorder.operation()),
            failure,
        })
    }

    /// Joined invocation facts without performing new mathematics or materializing result tables.
    pub fn invocation_evidence(
        &self,
    ) -> Result<crate::invocation::InvocationEvidence, WorkflowError> {
        use crate::invocation::{
            InvocationEndpoints, InvocationEvidenceDocument, InvocationOutcome, InvocationProduct,
            ProductAvailability,
        };
        let (operation, endpoints, intervals, omitted_intervals, preparations) = self
            .invocation
            .as_ref()
            .map(|recorder| recorder.projection())
            .unwrap_or((None, InvocationEndpoints::default(), Vec::new(), 0, None));
        let family = match self.original_request() {
            RunRequest::Horizon(_) => "horizon",
            RunRequest::Modeling(_) => "modeling",
            RunRequest::Simulation(_) => "simulation",
            RunRequest::Fit(_) => "fit",
            #[cfg(feature = "solver-diffsol")]
            RunRequest::Shooting { .. } => "shooting",
        };
        let mut products = vec![InvocationProduct {
            step: None,
            role: "original_scientific_result".into(),
            availability: if self.usable() {
                ProductAvailability::Available
            } else {
                ProductAvailability::Unavailable
            },
        }];
        if let RunOutcome::Fit {
            request: prepared,
            result,
        } = &self.outcome
        {
            let report = result.as_ref().ok().map(Box::as_ref);
            let availability = |requested: bool, available: bool| {
                if !requested {
                    ProductAvailability::NotRequested
                } else if available {
                    ProductAvailability::Available
                } else {
                    ProductAvailability::Unavailable
                }
            };
            products.push(InvocationProduct {
                step: None,
                role: "response_jacobian".into(),
                availability: availability(true, report.is_some_and(|r| r.responses.is_some())),
            });
            products.push(InvocationProduct {
                step: None,
                role: "identifiability".into(),
                availability: availability(true, report.is_some_and(|r| r.rank.is_some())),
            });
            // Completed computation remains in the report; estimate-derived products
            // require the actual governing original qualification as well.
            let product_permission = |quantity| {
                self.completion()
                    .ok()
                    .and_then(|c| c.product_permission(quantity))
                    .is_some_and(super::ProductPermission::permits_result)
            };
            let covariance = report.and_then(|r| r.covariance.as_ref());
            products.push(InvocationProduct {
                step: None,
                role: "parameter_covariance".into(),
                availability: availability(
                    covariance.is_some()
                        || prepared
                            .problem
                            .declaration
                            .parameters
                            .iter()
                            .any(|p| !p.fixed),
                    product_permission(
                        pse_model::generated::enums::DerivedQuantity::ParameterCovariance,
                    ),
                ),
            });
            let uncertainty = prepared.problem.profile.uncertainty.as_ref();
            products.push(InvocationProduct {
                step: None,
                role: "wald_intervals".into(),
                availability: availability(
                    uncertainty.is_some(),
                    product_permission(pse_model::generated::enums::DerivedQuantity::WaldInterval),
                ),
            });
            products.push(InvocationProduct {
                step: None,
                role: "profile_intervals".into(),
                availability: availability(
                    uncertainty.is_some_and(|u| u.profile.is_some()),
                    product_permission(
                        pse_model::generated::enums::DerivedQuantity::ProfileInterval,
                    ),
                ),
            });
        }
        match self.report() {
            Ok(RunReport::Simulation(_)) => products.push(InvocationProduct {
                role: "trajectory".into(),
                step: None,
                availability: if self.usable() {
                    ProductAvailability::Available
                } else {
                    ProductAvailability::Unavailable
                },
            }),
            #[cfg(feature = "solver-diffsol")]
            Ok(RunReport::Shooting(report)) => products.push(InvocationProduct {
                role: "shooting_trajectory".into(),
                step: None,
                availability: if self.usable() && report.trajectory.is_some() {
                    ProductAvailability::Available
                } else {
                    ProductAvailability::Unavailable
                },
            }),
            _ => {}
        }
        let mut omitted_product_steps = 0;
        if let RunRequest::Modeling(requests) = self.request() {
            let limit = super::AttemptSummaryLimits::default().steps;
            omitted_product_steps = requests.len().saturating_sub(limit);
            for (step, request) in requests.iter().take(limit).enumerate() {
                let result = self.modeling_result(step);
                // Each original completion governs its own scientific products. A
                // later refused/failed sequence member cannot erase accepted work.
                let original_permission = self
                    .completion()
                    .ok()
                    .and_then(|completion| completion.step_permission(step))
                    .is_some_and(super::ScientificPermission::permits_result);
                let native = result.and_then(|result| match &result.outcome {
                    crate::math::solves::Outcome::Native(native) => Some(native.as_ref()),
                    _ => None,
                });
                let wanted = request.profile.sensitivity.as_ref();
                let evidence = native.and_then(|report| report.evidence.sensitivity.as_ref());
                let root = native.and_then(|report| report.evidence.root_response.as_ref());
                let status = |requested: bool, produced: bool| {
                    if !requested {
                        ProductAvailability::NotRequested
                    } else if original_permission && produced {
                        ProductAvailability::Available
                    } else {
                        ProductAvailability::Unavailable
                    }
                };
                products.push(InvocationProduct {
                    role: "parametric_sensitivity".into(),
                    step: Some(step),
                    availability: status(
                        wanted.is_some(),
                        evidence.is_some_and(|e| e.sensitivities.is_ok())
                            || root.is_some_and(Result::is_ok),
                    ),
                });
                products.push(InvocationProduct {
                    role: "reduced_hessian".into(),
                    step: Some(step),
                    availability: status(
                        wanted.is_some_and(|w| w.reduced_hessian),
                        evidence
                            .and_then(|e| e.reduced_hessian.as_ref())
                            .is_some_and(Result::is_ok),
                    ),
                });
                // Propagation is computed by the explicitly requested lazy projection owner;
                // the immutable joined response must not invent its later completion.
                products.push(InvocationProduct {
                    role: "propagated_covariance".into(),
                    step: Some(step),
                    availability: status(wanted.is_some_and(|w| w.propagation.is_some()), false),
                });
            }
        }
        let attempt_id = match &self.durability {
            RunDurability::Ephemeral => None,
            RunDurability::Durable(record) => Some(record.attempt_id),
        };
        let owner = self.attempt_summary()?;
        Ok(crate::invocation::InvocationEvidence { document: InvocationEvidenceDocument { version: Default::default(), run_id: self.run_id, attempt_id, operation, family: family.into(),
            clock_scope: "wall seconds from invocation establishment before dispatch; earlier preparation excluded".into(),
            timing_unavailable: self.invocation_timing_unavailable,
            canonical_run: self.canonical_run_key().map(str::to_owned), canonical_attempt: self.canonical_attempt_key().map(str::to_owned),
            outcome: if self.invocation_cancelled { InvocationOutcome::Cancelled } else if self.report().is_err() { InvocationOutcome::Failed } else { InvocationOutcome::Completed },
            endpoints, intervals, omitted_intervals, preparations,
            products, omitted_product_steps,
            attempts: owner.document(), }, _owner: owner })
    }
    /// Full typed reports and backend-specific metrics; errors preserve their original causes.
    pub fn report(&self) -> Result<RunReport<'_>, &WorkflowError> {
        self.outcome.report().map_err(AsRef::as_ref)
    }
    /// The authoritative paired input and typed result.
    pub fn outcome(&self) -> &RunOutcome {
        &self.outcome
    }
    pub(super) fn error(&self) -> Option<&Arc<WorkflowError>> {
        self.outcome.error()
    }
    pub(super) fn fail_report(&mut self, error: Arc<WorkflowError>) {
        self.outcome.fail(error);
    }
    pub(super) fn original_request(&self) -> RunRequest<'_> {
        self.outcome.original_request()
    }
    /// Original immutable declarations for each requested step, including unattempted steps.
    pub fn request(&self) -> RunRequest<'_> {
        self.outcome.request()
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
fn attempt_for(
    services: &ExecutionServices,
    given: Option<DurableAttempt>,
) -> Result<Option<DurableAttempt>, WorkflowError> {
    match (&services.durability, given) {
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
fn progress_for(history: usize, durable: Option<&DurableAttempt>) -> Arc<Progress> {
    Arc::new(match durable {
        Some(attempt) => Progress::tapped(history, attempt.tap()),
        None => Progress::new(history),
    })
}
/// Register a durable attempt (planned, queued), wait for native admission, and enter
/// running under its lease; `admission` resolves once native admission holds.
async fn admit<T>(
    services: &super::SourceServices,
    durable: &mut Option<DurableAttempt>,
    run_id: RunId,
    request: &RunRequest<'_>,
    admission: impl Future<Output = Result<T, WorkflowError>>,
    cancel: Canceller,
) -> Result<T, WorkflowError> {
    if let Some(attempt) = durable.as_ref() {
        attempt.register(services, run_id, request).await?;
    }
    if let Some(attempt) = durable.as_mut() {
        attempt.start(cancel).await?;
    }
    admission.await
}
/// Submit a native operation under bounded population admission. Durable provenance
/// observes when CPU and memory are admitted without adding another waiting population.
fn submission(
    cancel: &Cancellation,
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
/// One owner for execution identity, cancellation, progress and joined publication.
/// Operation code retains its typed scientific finish, including partial prefixes.
pub(in crate::workflow) struct RunSupervisor {
    pub(in crate::workflow) services: ExecutionServices,
    pub(in crate::workflow) run_id: RunId,
    pub(in crate::workflow) durable: Option<DurableAttempt>,
    pub(in crate::workflow) progress: Arc<Progress>,
    pub(in crate::workflow) checks: crate::CancelSource,
    native: Cancellation,
}
impl RunSupervisor {
    pub(in crate::workflow) fn new(
        services: ExecutionServices,
        attempt: Option<DurableAttempt>,
        parent: Option<&crate::CancelSource>,
        deadline: Option<std::time::Instant>,
        history: usize,
    ) -> Result<Self, WorkflowError> {
        let durable = attempt_for(&services, attempt)?;
        let run_id = durable
            .as_ref()
            .and_then(DurableAttempt::claimed_run)
            .unwrap_or_else(pse_operations::mint_id);
        Ok(Self {
            services,
            run_id,
            progress: progress_for(history, durable.as_ref()),
            durable,
            checks: parent.map_or_else(
                || crate::CancelSource::new().with_deadline(deadline),
                |parent| parent.child_with_deadline(deadline),
            ),
            native: Cancellation::default(),
        })
    }
    fn public_handle(
        &self,
        receiver: tokio::sync::watch::Receiver<Option<Arc<RunResult>>>,
    ) -> RunHandle {
        let canonical_keys = self
            .durable
            .as_ref()
            .map(|attempt| attempt.canonical_keys(self.run_id));
        RunHandle {
            observation: observations_for(self.run_id, canonical_keys.as_ref()),
            lease: Arc::new(Lease::new(self.checks.clone(), self.durable.is_some())),
            receiver,
            progress: self.progress.clone(),
            run_id: self.run_id,
            attempt_id: self.durable.as_ref().map(DurableAttempt::attempt_id),
            canonical_keys,
        }
    }
    pub(in crate::workflow) fn spawn<F, Fut>(self, work: F) -> RunHandle
    where
        F: FnOnce(Self) -> Fut + Send + 'static,
        Fut: Future<Output = RunResult> + Send + 'static,
    {
        let (sender, receiver) = tokio::sync::watch::channel(None);
        let handle = self.public_handle(receiver);
        let checks = self.checks.clone();
        let native = self.native.clone();
        tokio::spawn(crate::tooling::runtime_metrics::instrument(
            crate::tooling::runtime_metrics::NativeTaskClass::Workflow,
            crate::workflow::inherit_runtime_activity(
                joined_with_cancel(move || work(self), native, checks).map(move |result| {
                    sender.send_replace(Some(Arc::new(result)));
                }),
            ),
        ));
        handle
    }
    /// Dispatch and typed finish travel together. This is the single registration,
    /// admission, refusal/drain and durable completion skeleton for native families.
    fn start_native<Operation, T, Dispatch, Finish, Finished>(
        self,
        operation: Operation,
        dispatch: Dispatch,
        finish: Finish,
    ) -> Result<RunHandle, WorkflowError>
    where
        Operation: PairedOperation,
        T: Send + 'static,
        Dispatch: FnOnce(Submission) -> Result<crate::math::solves::SolveHandle<T>, WorkflowError>
            + Send
            + 'static,
        Finish: FnOnce(crate::math::solves::SolveHandle<T>, Option<DurableAttempt>) -> Finished
            + Send
            + 'static,
        Finished: Future<Output = (NativeFinished<Operation::Report>, Option<DurableAttempt>)>
            + Send
            + 'static,
    {
        let deadline = self
            .checks
            .deadline()
            .ok_or_else(|| contract("native run deadline absent"))?;
        let (submission, signal) = submission(
            &self.native,
            &self.progress,
            self.durable.is_some(),
            deadline,
        );
        let mut dispatch = Some(move || dispatch(submission));
        let immediate = if self.durable.is_none() {
            Some(dispatch
                .take()
                .ok_or_else(|| contract("native dispatch absent"))?(
            )?)
        } else {
            None
        };
        Ok(self.spawn(move |mut supervisor| async move {
            let registered = async {
                if let Some(attempt) = supervisor.durable.as_mut() {
                    attempt
                        .register(
                            &supervisor.services.source_services(),
                            supervisor.run_id,
                            &operation.request(),
                        )
                        .await?;
                    let checks = supervisor.checks.clone();
                    attempt.start(Arc::new(move || checks.cancel())).await?;
                }
                Ok::<(), WorkflowError>(())
            }
            .await;
            let handle = match registered {
                Err(error) => {
                    if let Some(handle) = immediate {
                        handle.cancel();
                        let _ = handle.finish().await;
                    }
                    return RunResult::joined(
                        supervisor.run_id,
                        supervisor.services,
                        operation.finish(Err(Arc::new(error))),
                        None,
                    )
                    .refused(supervisor.durable)
                    .await;
                }
                Ok(()) => match immediate {
                    Some(handle) => Ok(handle),
                    None => dispatch
                        .take()
                        .ok_or_else(|| contract("native dispatch absent"))
                        .and_then(|dispatch| dispatch()),
                },
            };
            let handle = match handle {
                Ok(handle) => handle,
                Err(error) => {
                    return RunResult::joined(
                        supervisor.run_id,
                        supervisor.services,
                        operation.finish(Err(Arc::new(error))),
                        None,
                    )
                    .refused(supervisor.durable)
                    .await;
                }
            };
            if let Err(error) = admitted(signal).await {
                handle.cancel();
                let _ = handle.finish().await;
                return RunResult::joined(
                    supervisor.run_id,
                    supervisor.services,
                    operation.finish(Err(Arc::new(error))),
                    None,
                )
                .refused(supervisor.durable)
                .await;
            }
            let join = crate::invocation::stage("native_join_and_workflow_postsolve");
            let ((report, owner), durable) = finish(handle, supervisor.durable).await;
            join.finish(if report.is_ok() {
                crate::invocation::InvocationOutcome::Completed
            } else {
                crate::invocation::InvocationOutcome::Failed
            });
            RunResult::joined(
                supervisor.run_id,
                supervisor.services,
                operation.finish(report),
                owner,
            )
            .finished(durable, supervisor.checks.token().is_cancelled())
            .await
        }))
    }
}
type NativeFinished<T> = (
    Result<T, Arc<WorkflowError>>,
    Option<Arc<pse_columnar::AllocationLease>>,
);
async fn finish_native_report<T: Send + 'static>(
    handle: crate::math::solves::SolveHandle<(T, Arc<pse_columnar::AllocationLease>)>,
    durable: Option<DurableAttempt>,
) -> (NativeFinished<T>, Option<DurableAttempt>) {
    let finished = match handle.finish().await {
        Ok((report, owner)) => (Ok(report), Some(owner)),
        Err(error) => (Err(Arc::new(WorkflowError::Math(error))), None),
    };
    (finished, durable)
}
/// One tree-to-native bridge. Stop sets the callback's cheap leaf and then joins
/// the complete operation; no cancellation branch drops owned native work.
#[inline(never)]
fn joined_with_cancel<F, Fut, T>(
    work: F,
    native: Cancellation,
    checks: crate::CancelSource,
) -> impl Future<Output = T>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let mut inner: futures_util::future::BoxFuture<'static, T> =
        super::worker::boxed_worker_action(work);
    let work = futures_util::future::poll_fn(move |context| {
        super::worker::poll_worker_action(inner.as_mut(), context)
    });
    crate::invocation::TransferredContext::capture().scope(async move {
        checks
            .join_native(&native, work, || {
                if let Some(context) =
                    pse_diagnostics::observation::current().filter(|context| context.is_active())
                {
                    context.boolean("run.stop_requested", true);
                }
            })
            .await
    })
}

impl super::ModelingSimulation {
    /// Start the authored simulation through the shared joined run and durable lifecycle.
    pub fn start(&self, services: &ExecutionServices) -> Result<RunHandle, WorkflowError> {
        self.start_owned(services, None, self.admission_deadline()?, None)
    }
    /// Bind a bounded live subscription before any native effect. Dropping the cursor
    /// detaches only that subscription; the run retains cancellation and drain ownership.
    pub fn start_live(
        &self,
        services: &ExecutionServices,
        options: super::LiveSimulationOptions,
    ) -> Result<(RunHandle, super::LiveSimulationCursor), WorkflowError> {
        let (handle, cursor) = self.start_owned_live(
            services,
            None,
            self.admission_deadline()?,
            None,
            Some(options),
        )?;
        Ok((
            handle,
            cursor.ok_or_else(|| contract("live subscription absent"))?,
        ))
    }
    /// Start with caller cancellation linked through pending admission and native drain.
    pub fn start_with_cancel(
        &self,
        services: &ExecutionServices,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        if cancel.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled.into());
        }
        let deadline = self.admission_deadline()?;
        self.start_owned(
            services,
            None,
            cancel
                .deadline()
                .map_or(deadline, |parent| deadline.min(parent)),
            Some(cancel.clone()),
        )
    }
    pub(crate) async fn start_attempt(
        &self,
        services: &ExecutionServices,
        attempt: DurableAttempt,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        let activity = services.preparation().admit_activity()?;
        activity
            .scope(async {
                let admission = self.admission_deadline().and_then(|deadline| {
                    if matches!(services.durability(), Durability::Ephemeral) {
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
                self.start_owned(services, Some(attempt), deadline, Some(cancel.clone()))
            })
            .await
    }
    fn admission_deadline(&self) -> Result<std::time::Instant, WorkflowError> {
        std::time::Instant::now()
            .checked_add(self.profile().time_limit)
            .ok_or_else(|| deadline_limit("submitted task deadline extent"))
    }
    fn start_owned(
        &self,
        services: &ExecutionServices,
        attempt: Option<DurableAttempt>,
        deadline: std::time::Instant,
        parent: Option<crate::CancelSource>,
    ) -> Result<RunHandle, WorkflowError> {
        self.start_owned_live(services, attempt, deadline, parent, None)
            .map(|(handle, _)| handle)
    }
    fn start_owned_live(
        &self,
        services: &ExecutionServices,
        attempt: Option<DurableAttempt>,
        deadline: std::time::Instant,
        parent: Option<crate::CancelSource>,
        live: Option<super::LiveSimulationOptions>,
    ) -> Result<(RunHandle, Option<super::LiveSimulationCursor>), WorkflowError> {
        self.source.check_service_bindings(services.preparation())?;
        if let Some(options) = live {
            options.admit(self.profile())?;
        }
        let activity = services.preparation().admit_activity()?;
        activity.sync_scope(crate::invocation::start_scope(|| {
            let supervisor = RunSupervisor::new(services.clone(), attempt, parent.as_ref(), Some(deadline), 256)?;
            let run_id = supervisor.run_id;
            let deadline = supervisor.checks.deadline().ok_or_else(|| contract("simulation deadline absent"))?;
            let cancel = supervisor.native.clone();
            let canonical_attempt = supervisor.durable.as_ref().map(|attempt| attempt.canonical_keys(run_id).1);
            let live = live.map(|options| super::live_simulation::LiveDispatch::new(
                services.preparation(), self.clone(), options, run_id, canonical_attempt, &cancel, deadline,
            )).transpose()?;
            let (observer, mut live_channel, mut cursor) = match live {
                Some(live) => (Some((live.retention, live.observer)), Some(live.channel), Some(live.cursor)),
                None => (None, None, None),
            };
            let dispatch_source = self.clone();
            let live_refusal = live_channel.as_ref().and_then(|channel| channel.projection.as_ref().err()).cloned();
            let prepared = self.clone();
            let execution = services.clone();
            let projection_services = services.preparation().clone();
            let handle = supervisor.start_native(
                Box::new(self.clone()),
                move |submission| {
                    if let Some(error) = live_refusal { return Err(WorkflowError::Shared(error)); }
                    match observer {
                        Some((retention, observer)) => dispatch_source.submit_with_samples(&execution, run_id, submission, retention, observer),
                        None => dispatch_source.submit(&execution, run_id, submission),
                    }
                },
                move |handle, mut durable| async move {
                        if let Some(channel) = &live_channel && let Some(attempt) = durable.as_mut()
                            && let Err(error) = attempt.begin_live_samples() {
                                super::live_simulation::deliver(&channel.subscriber, super::LiveSimulationEvent::RecordingUnknown(Arc::new(error)), &cancel, deadline).await;
                            }
                        let finished = handle.finish();
                        tokio::pin!(finished);
                        let mut joined = None;
                        let mut selected = false;
                        let mut live_encoding_error = None;
                        if let Some(channel) = live_channel.as_mut() {
                            loop {
                                tokio::select! {
                                    result = &mut finished, if joined.is_none() => { joined = Some(result); }
                                    event = channel.receiver.recv() => {
                                        let Some(event) = event else { break; };
                                        let encoded = channel.projection.as_ref()
                                            .map_err(|error| WorkflowError::Shared(error.clone()))
                                            .and_then(|projection| event.encode(&projection_services, &prepared, projection, &cancel, deadline));
                                        let event = match encoded {
                                            Ok(event) => event,
                                            Err(error) => {
                                                let error = Arc::new(error);
                                                cancel.cancel();
                                                channel.receiver.close();
                                                while channel.receiver.try_recv().is_ok() {}
                                                if let Some(attempt) = durable.as_mut() { attempt.fail_live_samples(); }
                                                super::live_simulation::deliver(&channel.subscriber, super::LiveSimulationEvent::RecordingUnknown(error.clone()), &cancel, deadline).await;
                                                live_encoding_error = Some(error);
                                                break;
                                            }
                                        };
                                        let append = match &event {
                                            super::LiveSimulationEvent::Sample(batch) => Some(batch.table.clone()),
                                            super::LiveSimulationEvent::Selected { .. } => { selected = true; None },
                                            _ => None,
                                        };
                                        super::live_simulation::deliver(&channel.subscriber, event, &cancel, deadline).await;
                                        if let (Some(table), Some(attempt)) = (append, durable.as_mut()) {
                                            let event = match attempt.publish_live_samples(&table).await {
                                                Ok(rows) => super::LiveSimulationEvent::Recorded { rows },
                                                Err(error) => super::LiveSimulationEvent::RecordingUnknown(Arc::new(error)),
                                            };
                                            super::live_simulation::deliver(&channel.subscriber, event, &cancel, deadline).await;
                                        }
                                    }
                                }
                            }
                            if selected && let Some(attempt) = durable.as_mut() {
                                let event = match attempt.close_live_samples().await {
                                    Ok(()) => super::LiveSimulationEvent::RecordingClosed,
                                    Err(error) => super::LiveSimulationEvent::RecordingUnknown(Arc::new(error)),
                                };
                                super::live_simulation::deliver(&channel.subscriber, event, &cancel, deadline).await;
                            }
                        }
                        let joined = match joined { Some(result) => result, None => finished.await };
                        drop(live_channel);
                        let report = match (live_encoding_error, joined) {
                            (Some(error), _) => Err(error),
                            (None, Ok(((report, checks, accuracy), owner))) => prepared
                                .finish(&projection_services, run_id, report, checks, accuracy, owner)
                                .map(Box::new)
                                .map_err(Arc::new),
                            (None, Err(error)) => Err(Arc::new(WorkflowError::Math(error))),
                        };
                    ((report, None), durable)
                },
            )?;
            if let Some(cursor) = cursor.as_mut() { cursor.bind_completion(handle.receiver.clone()); }
            Ok((handle, cursor))
        }))
    }
}

#[cfg(feature = "solver-diffsol")]
impl super::ShootingProblem {
    /// Start under the runtime's shared resource, cancellation and joined run lifecycle.
    pub fn start(
        self: &Arc<Self>,
        services: &ExecutionServices,
    ) -> Result<RunHandle, WorkflowError> {
        self.start_requested(services, None, None)
    }
    /// Admit an explicit original-coordinate initial vector before native dispatch.
    pub fn start_with_initial(
        self: &Arc<Self>,
        services: &ExecutionServices,
        initial: Vec<f64>,
    ) -> Result<RunHandle, WorkflowError> {
        if initial.len() != self.initial_extent() || initial.iter().any(|v| !v.is_finite()) {
            return Err(contract(
                "shooting initial coordinate extent or nonfinite value",
            ));
        }
        self.start_requested(services, Some(initial), None)
    }
    /// Preserve caller cancellation through the existing shooting supervisor.
    pub fn start_replay_with_cancel(
        self: &Arc<Self>,
        services: &ExecutionServices,
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
        self.start_requested(services, initial, Some(cancel.clone()))
    }
    fn start_requested(
        self: &Arc<Self>,
        services: &ExecutionServices,
        initial: Option<Vec<f64>>,
        parent: Option<crate::CancelSource>,
    ) -> Result<RunHandle, WorkflowError> {
        self.simulation
            .source
            .check_service_bindings(services.preparation())?;
        let activity = services.preparation().admit_activity()?;
        activity.sync_scope(crate::invocation::start_scope(|| {
            let deadline = std::time::Instant::now()
                .checked_add(self.solver.controls.time_limit)
                .ok_or_else(|| deadline_limit("submitted task deadline extent"))?;
            let supervisor =
                RunSupervisor::new(services.clone(), None, parent.as_ref(), Some(deadline), 256)?;
            let run_id = supervisor.run_id;
            let deadline = supervisor
                .checks
                .deadline()
                .ok_or_else(|| contract("shooting deadline absent"))?;
            let prepared = self.clone();
            let mut declared_profile = self.solver.clone();
            if initial.is_some() {
                declared_profile.controls.start = pse_backend_native::solve::StartPolicy::Explicit;
            }
            let start = initial.clone();
            let native = services.preparation().math().clone();
            let job_bytes = self.job_bytes()?;
            let execution = services.clone();
            let dispatch = move |submission| {
                native
                    .submit_with_storage(
                        declared_profile.controls.threads,
                        job_bytes,
                        submission,
                        move |flag, progress, storage| {
                            let report = prepared.solve(
                                &execution,
                                run_id,
                                pse_kernels::ExecutionScope::new(flag, Some(deadline)),
                                progress,
                                start.as_deref(),
                                &declared_profile,
                                Some(storage),
                            )?;
                            let retained = report
                                .numeric_bytes()
                                .checked_add(prepared.solver.controls.report_allowance()?)
                                .ok_or(MathRuntimeError::Limit("shooting result extent"))?;
                            Ok((Box::new(report), retained))
                        },
                    )
                    .map_err(WorkflowError::from)
            };
            supervisor.start_native(
                super::run_outcome::ShootingOperation {
                    problem: self.clone(),
                    initial,
                },
                dispatch,
                finish_native_report,
            )
        }))
    }
}

impl super::PreparedFit {
    /// Start one native fitting attempt under the existing joined job lifecycle.
    pub fn start(&self, services: &ExecutionServices) -> Result<RunHandle, WorkflowError> {
        self.start_owned(services, None, self.admission_deadline()?, None)
    }
    /// Start with caller cancellation linked through pending admission and native drain.
    pub fn start_with_cancel(
        &self,
        services: &ExecutionServices,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        if cancel.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled.into());
        }
        let deadline = self.admission_deadline()?;
        self.start_owned(
            services,
            None,
            cancel
                .deadline()
                .map_or(deadline, |parent| deadline.min(parent)),
            Some(cancel.clone()),
        )
    }
    pub(crate) async fn start_attempt(
        &self,
        services: &ExecutionServices,
        attempt: DurableAttempt,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        let activity = services.preparation().admit_activity()?;
        activity
            .scope(async {
                let admission = self.admission_deadline().and_then(|deadline| {
                    if matches!(services.durability(), Durability::Ephemeral) {
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
                self.start_owned(services, Some(attempt), deadline, Some(cancel.clone()))
            })
            .await
    }
    fn admission_deadline(&self) -> Result<std::time::Instant, WorkflowError> {
        std::time::Instant::now()
            .checked_add(self.problem.profile.solver.controls.time_limit)
            .ok_or_else(|| deadline_limit("submitted task deadline extent"))
    }
    fn start_owned(
        &self,
        services: &ExecutionServices,
        attempt: Option<DurableAttempt>,
        deadline: std::time::Instant,
        parent: Option<crate::CancelSource>,
    ) -> Result<RunHandle, WorkflowError> {
        self.source.check_service_bindings(services.preparation())?;
        let activity = services.preparation().admit_activity()?;
        activity.sync_scope(crate::invocation::start_scope(|| {
            let supervisor = RunSupervisor::new(
                services.clone(),
                attempt,
                parent.as_ref(),
                Some(deadline),
                256,
            )?;
            let run_id = supervisor.run_id;
            let deadline = supervisor
                .checks
                .deadline()
                .ok_or_else(|| contract("fit deadline absent"))?;
            let prepared = self.clone();
            // Estimation admits only its own demand. Profile teams acquire their next
            // stage after the estimate reaches the quiescent phase boundary.
            let workers = self
                .problem
                .profile_workers(services.preparation().math().cores());
            let cores = self.problem.profile.solver.controls.threads;
            let bytes = self.problem.bytes;
            let native = services.preparation().math().clone();
            #[cfg(test)]
            let entry = family_worker_entry();
            let execution = services.clone();
            let dispatch = move |submission| {
                native
                    .submit_with_storage(
                        cores,
                        bytes,
                        submission,
                        move |flag, progress, storage| {
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
                                &execution,
                                run_id,
                                pse_kernels::ExecutionScope::new(flag, Some(deadline)),
                                progress,
                                workers,
                                Some(storage),
                            )?;
                            let retained = report
                                .numeric_bytes()
                                .checked_add(allowance)
                                .ok_or(MathRuntimeError::Limit("fit result extent"))?;
                            Ok((Box::new(report), retained))
                        },
                    )
                    .map_err(WorkflowError::from)
            };
            supervisor.start_native(Box::new(self.clone()), dispatch, finish_native_report)
        }))
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
    /// Select a complete physically typed original seed from its registered Arrow relation.
    pub fn with_primal_start(
        mut self,
        services: &super::PreparationServices,
        batches: impl datafusion::arrow::record_batch::RecordBatchReader,
    ) -> Result<Self, WorkflowError> {
        let snapshot = services.snapshot_primal_values(batches)?;
        let plan = &self.model.case.compiled().plan;
        let columns = plan
            .columns()
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let ports = plan
            .structure()
            .variables()
            .iter()
            .filter(|variable| columns.contains(&variable.port.id))
            .map(|variable| variable.port.clone())
            .collect::<Vec<_>>();
        let values = snapshot.admit(&self.source.quantities, &ports)?;
        self.solve = self
            .solve
            .with_primal_start(values)
            .map_err(MathRuntimeError::from)?;
        self.profile.controls.start = pse_backend_native::solve::StartPolicy::Explicit;
        self.stored_seed_owner = Some(Arc::new(snapshot));
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
    pub fn start(&self, services: &ExecutionServices) -> Result<RunHandle, WorkflowError> {
        services.launch(vec![self.clone()], false, None, None)
    }
}

impl Runtime {
    /// Execute prepared original steps with explicitly supplied run capabilities.
    pub async fn start_modeling(
        &self,
        steps: Vec<super::ModelingSolvePreparation>,
        continue_independent: bool,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        self.execution_services()
            .start_modeling(steps, continue_independent, cancel)
            .await
    }
}

impl ExecutionServices {
    /// Start a finite authored sequence of prepared steps.
    pub async fn start_modeling(
        &self,
        steps: Vec<super::ModelingSolvePreparation>,
        continue_independent: bool,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        let activity = self.preparation().admit_activity()?;
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
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        let activity = self.preparation().admit_activity()?;
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
                self.launch(steps, false, Some(attempt), Some(cancel.clone()))
            })
            .await
    }
    fn check_sequence(
        &self,
        steps: &[super::ModelingSolvePreparation],
    ) -> Result<(), WorkflowError> {
        if steps.is_empty() || steps.len() > 4096 {
            return Err(contract("empty or excessive authored sequence"));
        }
        let mut declarations = BTreeMap::new();
        for step in steps {
            step.source.check_service_bindings(self.preparation())?;
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
        self.check_sequence(&steps)?;
        let activity = self.preparation().admit_activity()?;
        activity.sync_scope(crate::invocation::start_scope(|| {
        let history = steps
            .iter()
            .map(|s| s.profile.controls.history)
            .max()
            .unwrap_or(0);
        let supervisor = RunSupervisor::new(self.clone(), attempt, parent.as_ref(), None, history)?;
        let opened = match supervisor.durable {
            None => Some(super::staged::Staged::open(self, Some(supervisor.progress.clone()))?),
            Some(_) => None,
        };
        Ok(supervisor.spawn(move |supervisor| async move {
            let RunSupervisor { services, run_id, mut durable, progress: shared, checks: cancel, .. } = supervisor;
            let request = steps.clone();
            let stop = cancel.clone();
            let session = async {
                match opened {
                    Some(staged) => Ok(staged),
                    None => tokio::select! {
                        staged=super::staged::Staged::open_queued(&services,Some(shared), &cancel,
                            steps.iter().filter_map(|step| step.solve.task_scope().and_then(|scope| scope.deadline()))
                                .chain(cancel.deadline()).min())=>staged,
                        ()=cancel.cancelled()=>Err(MathRuntimeError::Cancelled.into()),
                    },
                }
            };
            let mut staged = match admit(
                &services.source_services(),
                &mut durable,
                run_id,
                &request.request(),
                session,
                Arc::new(move || stop.cancel()),
            )
            .await
            {
                Ok(staged) => staged,
                Err(error) => {
                    let refused =
                        RunResult::joined(run_id, services, request.finish(Err(Arc::new(error))), None)
                            .with_modeling_policy(continue_independent)
                            .refused(durable)
                            .await;
                    return refused;
                }
            };
            let sequence = crate::invocation::stage("staged_modeling_execution_and_original_assessment");
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
            let release_failure = if failure.is_some() {
                staged.close().await;
                None
            } else {
                staged.finish_independent(&cancel).await.err()
            };
            let (report, modeling_failure) = match (failure, release_failure) {
                (Some((step, error)), _) => (
                    Err(error),
                    Some(ModelingFailure {
                        completed: results,
                        step: Some(step),
                    }),
                ),
                (None, Some(error)) => (
                    Err(Arc::new(error)),
                    Some(ModelingFailure { completed: results, step: None }),
                ),
                (None, None) => (Ok(results), None),
            };
            sequence.finish(if report.is_ok() { crate::invocation::InvocationOutcome::Completed } else { crate::invocation::InvocationOutcome::Failed });
            let cancelled = cancel.token().is_cancelled();
            let mut result = RunResult::joined(run_id, services, request.finish(report), None);
            result.modeling_continue_independent = continue_independent;
            result.modeling_failure = modeling_failure;
            result.finished(durable, cancelled).await
        }))

        }))
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
            SemanticId::from_bytes([175; 16]), pse_authoring::language::IdentityPolicy::Named, Default::default()).unwrap();
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
                &runtime.preparation_services(),
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
                &runtime.preparation_services(),
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
        let earlier_origin = result.diagnostic_origin(Some(0)).unwrap();
        assert_eq!(earlier_origin.run_id, result.run_id);
        assert_eq!(earlier_origin.native_attempt, Some(0));
        assert!(
            earlier_origin.failure.is_none(),
            "later failure must not be attributed to an accepted earlier step"
        );
        assert!(result.diagnostic_origin(Some(1)).unwrap().failure.is_some());
        assert!(
            result.diagnostic_origin(Some(2)).unwrap().failure.is_none(),
            "unattempted step has no originating failure"
        );
        assert!(result.diagnostic_origin(Some(3)).is_err());
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
    fn current_progress_remains_whole_without_retained_history() {
        use pse_backend_native::solve::{Event, Metric, Progress};
        let progress = std::sync::Arc::new(Progress::new(0));
        progress.push(Event {
            source: None,
            phase: "ordinary".into(),
            elapsed: std::time::Duration::from_secs(3),
            values: std::collections::BTreeMap::from([("iteration".into(), Metric::Integer(7))]),
            incumbent: None,
        });
        let (_sender, receiver) = tokio::sync::watch::channel(None);
        let handle = super::RunHandle {
            lease: std::sync::Arc::new(super::Lease::new(crate::CancelSource::new(), false)),
            observation: None,
            receiver,
            progress,
            run_id: pse_model::generated::identities::RunId::from_bytes([1; 16]),
            attempt_id: None,
            canonical_keys: None,
        };
        assert!(!handle.cancellation_scope().token().is_cancelled());
        assert!(handle.progress().0.is_empty());
        let checkpoint = handle.current_progress();
        assert_eq!(checkpoint.highest_issued_sequence, 1);
        assert!(checkpoint.current_is_latest);
        let current = checkpoint.current.unwrap();
        assert_eq!(current.phase, "ordinary");
        assert_eq!(current.elapsed_seconds, 3.0);
        assert!(matches!(
            current.values["iteration"],
            super::super::ProgressMetricDocument::Integer(7)
        ));
        assert!(
            current.sequence.is_none(),
            "native source sequence is not a durable append sequence"
        );
    }
    #[test]
    fn completed_handle_drop_does_not_fabricate_diagnostic_loss() {
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("completed.run");
        let context = operation.context();
        let reader = operation.reader();
        let mut lease = super::Lease::new(crate::CancelSource::new(), false);
        lease.observation = Some(context);
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
    #[test]
    fn caller_departure_cancels_only_immediate_child_and_keeps_durable_owner_live() {
        let caller = crate::CancelSource::new();
        let sibling = caller.child_with_deadline(None);
        let immediate = caller.child_with_deadline(None);
        let lease = super::Lease::new(immediate.clone(), false);
        drop(lease);
        assert!(immediate.token().is_cancelled());
        assert!(!caller.token().is_cancelled());
        assert!(!sibling.token().is_cancelled());
        let worker = crate::CancelSource::new();
        let durable = worker.child_with_deadline(None);
        drop(super::Lease::new(durable.clone(), true));
        assert!(
            !durable.token().is_cancelled(),
            "caller disconnect does not stop durable work"
        );
        let explicit = worker.child_with_deadline(None);
        let lease = super::Lease::new(explicit.clone(), true);
        lease.cancel();
        assert!(explicit.token().is_cancelled());
        assert!(!worker.token().is_cancelled());
        assert!(!durable.token().is_cancelled());
        worker.cancel();
        assert!(durable.token().is_cancelled());
    }
    #[tokio::test]
    async fn cancellation_bridge_publishes_stop_before_a_simultaneously_ready_candidate() {
        let source = crate::CancelSource::new();
        let native = pse_diagnostics::Cancellation::default();
        let observed = native.clone();
        source.cancel();
        let candidate = super::joined_with_cancel(
            move || async move {
                assert!(observed.is_cancelled());
                7
            },
            native,
            source,
        )
        .await;
        assert_eq!(
            candidate, 7,
            "drain retains its typed candidate for original assessment"
        );
    }
    #[tokio::test]
    async fn replay_parent_cancellation_reaches_pending_supervisor_and_keeps_drain_owned() {
        let parent = crate::CancelSource::new();
        let native = pse_diagnostics::Cancellation::default();
        let flight = native.clone();
        let (observed, cancellation) = tokio::sync::oneshot::channel();
        let (release, drain) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(super::joined_with_cancel(
            move || async move {
                flight.cancelled().await;
                observed.send(()).unwrap();
                drain.await.unwrap();
            },
            native,
            parent.child_with_deadline(None),
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

#[cfg(all(test, feature = "solver-ipopt", feature = "solver-highs"))]
mod scientific_product_tests {
    use crate::invocation::ProductAvailability;
    use crate::workflow::modeling::sensitivity_tests as fixture;
    use pse_backend_native::solve::{Backend, SolverSelection};

    #[tokio::test]
    async fn joined_scientific_products_use_each_original_completion_and_keep_native_evidence() {
        let source = fixture::QUADRATIC.replace(
            "annotation objective cost(minimize);",
            "annotation objective cost(minimize); annotation check a(a > 0);",
        );
        let (runtime, package, root) = fixture::package(&source).await;
        let cancel = crate::CancelSource::new();
        let mut analysis = fixture::analysis(root, SolverSelection::Explicit(Backend::Ipopt));
        analysis.bindings.demand = vec!["a".into(), "b".into()];
        let plain = package
            .prepare_analysis(&runtime.preparation_services(), &analysis, &cancel)
            .await
            .unwrap();
        let paths = &plain.model.model.compiled().model.paths;
        analysis.solver.sensitivity = Some(crate::math::settings::SensitivityRequest {
            parameters: vec![paths["a"], paths["b"]],
            reduced_hessian: true,
            propagation: None,
        });
        let good = package
            .prepare_analysis(&runtime.preparation_services(), &analysis, &cancel)
            .await
            .unwrap();
        // This remains a well-posed quadratic with computable native derivatives;
        // its separate authored original-model check is deliberately false.
        analysis.case.values.insert("a".into(), -1.0);
        let refused = package
            .prepare_analysis(&runtime.preparation_services(), &analysis, &cancel)
            .await
            .unwrap();
        analysis.case.values.insert("a".into(), 1.0);
        analysis.solver.sensitivity = None;
        let not_requested = package
            .prepare_analysis(&runtime.preparation_services(), &analysis, &cancel)
            .await
            .unwrap();
        let result = runtime
            .start_modeling(vec![good, refused, not_requested], false, &cancel)
            .await
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert!(
            !result.usable(),
            "a later original refusal governs the joined result"
        );
        let accepted = result.modeling_result(0).unwrap();
        let refused = result.modeling_result(1).unwrap();
        assert!(accepted.completion.permits_use());
        assert!(!refused.completion.permits_use());
        assert!(refused.checks.iter().any(|check| !check.satisfied));
        let accepted_origin = result.diagnostic_origin(Some(0)).unwrap();
        assert_eq!(accepted_origin.run_id, result.run_id);
        assert_eq!(accepted_origin.native_attempt, Some(0));
        assert!(accepted_origin.failure.is_none());
        assert!(
            refused.validation_error.is_none(),
            "an authored refusal is distinct from evaluator failure"
        );
        assert!(result.diagnostic_origin(None).unwrap().failure.is_some());
        let refused_origin = result.diagnostic_origin(Some(1)).unwrap();
        assert_eq!(refused_origin.run_id, result.run_id);
        assert_eq!(refused_origin.native_attempt, Some(1));
        assert!(
            refused_origin.failure.is_some(),
            "retain the authoritative authored refusal diagnostic"
        );
        assert!(result.diagnostic_origin(Some(3)).is_err());
        for step in [accepted, refused] {
            let crate::math::solves::Outcome::Native(native) = &step.outcome else {
                panic!("the control must exercise actual native scientific products");
            };
            let sensitivity = native.evidence.sensitivity.as_ref().unwrap();
            assert!(
                sensitivity.sensitivities.is_ok(),
                "raw native derivatives remain diagnostic evidence"
            );
            assert!(sensitivity.reduced_hessian.as_ref().unwrap().is_ok());
        }
        assert!(result.modeling_result(2).is_none());
        let evidence = result.invocation_evidence().unwrap();
        for (step, expected) in [
            (0, ProductAvailability::Available),
            (1, ProductAvailability::Unavailable),
            (2, ProductAvailability::NotRequested),
        ] {
            for role in ["parametric_sensitivity", "reduced_hessian"] {
                let product = evidence
                    .products
                    .iter()
                    .find(|product| product.step == Some(step) && product.role == role)
                    .unwrap();
                assert_eq!(product.availability, expected, "step {step}: {role}");
            }
        }
        // Joined original-result availability remains distinct from accepted step products.
        assert!(
            evidence
                .products
                .iter()
                .any(|product| product.role == "original_scientific_result"
                    && product.availability == ProductAvailability::Unavailable)
        );
    }
}
