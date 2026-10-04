// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The durable job queue from the runtime's side (ADR-0114 Outcomes 14, 15 and 18; Plan 22
//! O4): versioned job payloads over content-addressed source bundles, and the worker that
//! claims jobs, runs them under its lease and ends each try through the job's retry
//! policy.
//!
//! A job payload is a typed, versioned document (ADR-0116 Outcome 6): it names its authored
//! sources by the §6.1 package content hash, never by a path, plus the case, the typed
//! solve settings and a start policy. Its request identity is framed from the typed
//! document, never from the text of a JSON value (Outcome 9). A worker refuses a payload
//! version it does not know. The durable `cancel_requested` flag is the cancellation
//! authority: the claimed attempt's heartbeat returns it, and a `LISTEN` watcher that
//! re-reads it after every reconnect only shortens latency.
use super::{
    Durability, Operations, PhysicalContext, RunDurability, Runtime, WorkflowError, contract,
    durable::{Claim, DurableAttempt, DurableRecord},
};
use crate::authoring_driver::document::{
    OwnedDocumentSet, load_package_documents_owned, package_checksum,
};
use crate::math::settings::SolveSettings;
use pse_ids::ContentHash;
use pse_model::{document::Version, generated::enums::ModelingAnalysisRoute};
use pse_operations::{
    attempts::{AttemptKind, NewAttempt},
    jobs::{ClaimedJob, Enqueued, NewJob, RetryPolicy},
    lifecycle::AttemptState,
    solutions::SolutionId,
    sources::{SourceBundle, SourceDocument},
    studies::StudyId,
};
use std::{collections::BTreeMap, sync::Arc, time::Duration};

/// The payload version this build executes: the store's `payload_version` column and the
/// document's own `version` ([`JobPayload`]).
pub const JOB_PAYLOAD_VERSION: i32 = 7;

/// Version admission precedes decoding any nested current scientific contract.
#[derive(serde::Deserialize)]
pub(super) struct DocumentVersion {
    pub(super) version: u32,
}

/// How a job's solve is started.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[schemars(rename = "JobStart")]
pub enum JobStart {
    /// From the case's authored starts.
    #[default]
    Fresh,
    /// From the latest incumbent in the parent attempt chain (Plan 22 G8).
    ResumeFromParent,
    /// From one stored solution (Plan 22 G8).
    StoredSolution {
        /// The stored solution.
        solution: SolutionId,
    },
}

/// Version 6 of a durable job's payload: the one task a job runs. Unknown fields, tasks
/// and versions are refused.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JobPayload {
    /// Document version.
    pub version: Version<{ JOB_PAYLOAD_VERSION as u32 }>,
    /// The task.
    pub task: JobTask,
}

impl JobPayload {
    /// A payload running `task`.
    pub const fn new(task: JobTask) -> Self {
        Self {
            version: Version,
            task,
        }
    }
}

/// What a durable job does.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JobTask {
    /// Solve one authored case once, possibly as one point of a study.
    Modeling(Box<ModelingJob>),
    /// Run one admitted occurrence through its existing operation owner.
    StudyOperation(Box<StudyOperationJob>),
    /// Publish a concluded study: its summary and every completed point's result members,
    /// as the study's one publication (Plan 22 O7).
    StudyFinalization(StudyFinalization),
}

/// A study's finalization task.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyFinalization {
    /// The study to publish.
    pub study_id: StudyId,
}

/// One authored case of a package closure, solved once under typed solve settings; as a
/// study point it carries its binding.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModelingJob {
    /// The source bundle of the physical package.
    pub physical: ContentHash,
    /// The source bundles of the modeling package closure, in load order.
    pub modeling: Vec<ContentHash>,
    /// The authored case to solve.
    pub case: pse_model::generated::identities::DeclarationId,
    /// Its analysis route.
    pub route: ModelingAnalysisRoute,
    /// The solve settings.
    pub settings: SolveSettings,
    /// How the solve starts. A study point with a predecessor starts from the
    /// predecessor's stored solution instead.
    #[serde(default)]
    pub start: JobStart,
}

/// One immutable admitted occurrence and its package source bundles.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyOperationJob {
    /// Physical source bundle.
    pub physical: ContentHash,
    /// Modeling source bundle closure.
    pub modeling: Vec<ContentHash>,
    /// The exact admitted occurrence copied mechanically from StudyDefinition.
    pub point: StudyPointBinding,
}

/// Binding, occurrence policy and operation share one immutable authority.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyPointBinding {
    /// Study identity.
    pub study_id: StudyId,
    /// Occurrence key, independent of binding identity.
    pub point_index: u32,
    /// Canonical binding identity.
    pub binding_hash: pse_ids::roles::BindingHash,
    /// Physically admitted assignments.
    pub binding: super::AdmittedBinding,
    /// Shared typed dependencies and start policy.
    pub policy: pse_model::study::PointPolicy,
    /// Reconstructable operation owned by the immutable definition.
    pub operation: super::StudyOperation,
}

impl ModelingJob {
    /// The submitted logical job: its kind, idempotency scope and typed document.
    /// Scientific request identity remains independent of that operational scope.
    ///
    /// # Errors
    /// A settings serializer refused its value.
    pub fn operational_job_identity(
        &self,
        idempotency_key: &str,
    ) -> Result<pse_ids::roles::OperationalJobHash, WorkflowError> {
        pse_ids::document::of(
            pse_ids::Frame::DurableJobRequestV4,
            &(AttemptKind::Modeling, idempotency_key, self),
        )
        .map(pse_ids::roles::OperationalJobHash::from)
        .map_err(|e| contract(e.to_string()))
    }
}

/// Version 1 of a source bundle's manifest: the path of every document, in order.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SourceManifest {
    /// Document version.
    pub version: Version<1>,
    /// The document paths, relative to the package root.
    pub paths: Vec<String>,
}
/// What one pass of a worker did.
#[derive(Clone, Debug)]
pub enum Processed {
    /// No job was available.
    Idle,
    /// A claimed job's try ended; the record says how.
    Ran {
        /// The job.
        job: pse_operations::jobs::JobId,
        /// The try's durable record.
        record: Box<DurableRecord>,
    },
}

impl Processed {
    /// The terminal state of the try, when a job was processed and its end recorded.
    pub fn state(&self) -> Option<AttemptState> {
        match self {
            Self::Idle => None,
            Self::Ran { record, .. } => record.attempt.as_ref().ok().map(|a| a.state),
        }
    }
}

/// How a worker serves the queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkerSettings {
    /// How long an idle worker waits before claiming again.
    pub poll: Duration,
    /// How often the worker runs the stale sweep (expired leases requeue their jobs).
    pub recovery: Duration,
    /// Stop after this many processed jobs.
    pub jobs: Option<usize>,
    /// Stop as soon as no job is available.
    pub until_idle: bool,
}

impl Default for WorkerSettings {
    fn default() -> Self {
        Self {
            poll: Duration::from_millis(500),
            recovery: Duration::from_secs(30),
            jobs: None,
            until_idle: false,
        }
    }
}

fn operations(runtime: &Runtime) -> Result<&Operations, WorkflowError> {
    match &runtime.durability {
        Durability::Durable(operations) => Ok(operations),
        Durability::Ephemeral => Err(contract(
            "the job queue needs a durable runtime (ADR-0114 Outcome 16)",
        )),
    }
}

impl Operations {
    /// Store authored documents, each the exact bytes of the kind its path declares
    /// (ADR-0125), as a content-addressed source bundle and return its §6.1 package content
    /// hash. Storing the same documents again is a no-op.
    ///
    /// # Errors
    /// Store failures.
    pub async fn put_sources(
        &self,
        sources: &BTreeMap<String, Vec<u8>>,
    ) -> Result<ContentHash, WorkflowError> {
        let bundle_hash = package_checksum(sources);
        let documents = sources
            .iter()
            .map(|(path, bytes)| SourceDocument {
                path: path.clone(),
                content_hash: pse_ids::encoding_checksum(bytes).content_hash(),
                content: bytes.clone(),
            })
            .collect();
        self.store()
            .sources()
            .put(&SourceBundle {
                bundle_hash: bundle_hash.into(),
                manifest: serde_json::to_value(SourceManifest {
                    version: Version,
                    paths: sources.keys().cloned().collect(),
                })
                .map_err(|e| contract(format!("source manifest: {e}")))?,
                documents,
            })
            .await?;
        Ok(bundle_hash)
    }

    /// Read a source bundle and verify, byte for byte, its package content hash and every
    /// document hash; a data document is verified like a text one (ADR-0125).
    ///
    /// # Errors
    /// An unknown bundle, a store failure, or content that does not match its hashes.
    pub async fn sources(
        &self,
        bundle: &ContentHash,
    ) -> Result<BTreeMap<String, Vec<u8>>, WorkflowError> {
        let stored = self.store().sources().get(&(*bundle).into()).await?;
        let mut sources = BTreeMap::new();
        for document in stored.documents {
            if pse_ids::encoding_checksum(&document.content).content_hash() != document.content_hash
            {
                return Err(contract(format!(
                    "source document `{}` of bundle {} does not match its content hash",
                    document.path,
                    bundle.to_prefixed()
                )));
            }
            sources.insert(document.path, document.content);
        }
        if package_checksum(&sources) != *bundle {
            return Err(contract(format!(
                "source bundle {} does not match its package content hash",
                bundle.to_prefixed()
            )));
        }
        Ok(sources)
    }

    /// Enqueue a modeling job with its first attempt. The idempotency key names the logical
    /// request: enqueuing the same key again returns the existing job. Its operational
    /// identity includes that scope ([`ModelingJob::operational_job_identity`]).
    ///
    /// # Errors
    /// Store failures, an invalid retry policy, or a payload that has no document form.
    pub async fn enqueue(
        &self,
        job: &ModelingJob,
        idempotency_key: &str,
        retry: RetryPolicy,
        priority: i32,
    ) -> Result<Enqueued, WorkflowError> {
        let operational_job_identity = pse_ids::roles::RecordedOperationalJobIdentity::current(
            job.operational_job_identity(idempotency_key)?,
        );
        let payload =
            serde_json::to_value(JobPayload::new(JobTask::Modeling(Box::new(job.clone()))))
                .map_err(|e| contract(format!("job payload: {e}")))?;
        Ok(self
            .store()
            .jobs()
            .enqueue(&NewJob {
                attempt: NewAttempt {
                    attempt_id: pse_operations::mint_id(),
                    run_id: pse_operations::mint_id(),
                    kind: AttemptKind::Modeling,
                    operational_job_identity,
                    preparation_identity: None,
                    parent_attempt: None,
                },
                idempotency_key: idempotency_key.to_owned(),
                payload_version: JOB_PAYLOAD_VERSION,
                payload,
                priority,
                retry,
            })
            .await?)
    }
}

impl Runtime {
    /// Claim one available job and run it under this worker's lease; `Idle` when no job
    /// is available. The claimed attempt's lease is renewed and its cancellation flag
    /// watched from the claim on, through preparation and the run.
    ///
    /// # Errors
    /// A non-durable runtime, or a store failure while claiming. A failure of the job
    /// itself ends its try instead and is reported in the record.
    pub async fn work_once(&self) -> Result<Processed, WorkflowError> {
        Ok(self.work_once_with_result().await?.0)
    }

    /// As [`Self::work_once`], also returning the joined result of a try that ran: its
    /// typed reports, start sources and native metrics, for a caller that inspects them.
    /// The result holds its reservation until dropped.
    ///
    /// # Errors
    /// As for [`Self::work_once`].
    pub async fn work_once_with_result(
        &self,
    ) -> Result<(Processed, Option<Arc<super::RunResult>>), WorkflowError> {
        let operations = operations(self)?;
        self.reconcile_study_receipts().await?;
        let Some(claimed) = operations
            .store()
            .jobs()
            .claim(operations.worker(), operations.policy().lease)
            .await?
        else {
            return Ok((Processed::Idle, None));
        };
        let job = claimed.job_id;
        let cancel = crate::CancelSource::new();
        let mut attempt = DurableAttempt::claimed(
            operations,
            Claim {
                job,
                attempt: claimed.attempt_id,
                run: claimed.run_id,
            },
        );
        let stop = cancel.clone();
        let ran = |record| {
            Ok((
                Processed::Ran {
                    job,
                    record: Box::new(record),
                },
                None,
            ))
        };
        if let Err(error) = attempt.start(Arc::new(move || stop.cancel())).await {
            return ran(attempt.abandon(&error).await);
        }
        let task = match decode(&claimed) {
            Ok(task) => task,
            Err(error) => return ran(attempt.abandon(&error).await),
        };
        let handle = match task {
            JobTask::StudyFinalization(task) => {
                let published = tokio::select! { published = self.finalize_study(operations,task.study_id,&cancel) => published, () = cancel.cancelled() => Err(crate::math::MathRuntimeError::Cancelled.into()) };
                return ran(attempt
                    .end_task(published.map(|published| {
                        format!(
                            "study {} published as {}",
                            task.study_id, published.publication_id
                        )
                    }))
                    .await);
            }
            JobTask::Modeling(modeling) => {
                let preparation = tokio::select! { prepared = self.prepare_job(operations,&claimed,*modeling,&cancel) => prepared, () = cancel.cancelled() => Err(crate::math::MathRuntimeError::Cancelled.into()) };
                let (prepared, applied, _) = match preparation {
                    Ok(prepared) => prepared,
                    Err(error) => return ran(attempt.abandon(&error).await),
                };
                attempt.tap().observe(&applied.event());
                self.start_attempt(vec![prepared], attempt)?
            }
            JobTask::StudyOperation(task) => {
                let mut point = match operations.point_context(&task.point).await {
                    Ok(point) => point,
                    Err(error) => return ran(attempt.abandon(&error).await),
                };
                attempt.set_point(point.clone());
                let preparation = tokio::select! { prepared = self.prepare_study_job(operations,&task,&cancel) => prepared, () = cancel.cancelled() => Err(crate::math::MathRuntimeError::Cancelled.into()) };
                let (prepared, seed, acquisition_error) = match preparation {
                    Ok(prepared) => prepared,
                    Err(error) => return ran(attempt.abandon(&error).await),
                };
                let action = match operations
                    .store()
                    .studies()
                    .admit_dispatch(
                        pse_operations::studies::DispatchFence {
                            study: task.point.study_id,
                            key: task.point.policy.key,
                            job,
                            attempt: claimed.attempt_id,
                            worker: operations.worker(),
                            expected_revision: point.revision,
                        },
                        seed,
                    )
                    .await
                {
                    Ok(action) => action,
                    Err(error) => return ran(attempt.abandon(&error.into()).await),
                };
                use pse_model::study::ActionKind;
                match action {
                    ActionKind::Start(start) => { point.revision += 1; point.start = Some(start); attempt.set_point(point); prepared.start_attempt(self,&cancel,attempt).await? }
                    ActionKind::Refuse(refusal) => {
                        let mut diagnostic = super::study_execution::policy_refusal(&refusal);
                        if let Some(error) = acquisition_error { diagnostic.causes.push(error.boundary_diagnostic()); }
                        return ran(attempt.abandon(&diagnostic.into()).await);
                    }
                    ActionKind::Cancel => return ran(attempt.abandon(&crate::math::MathRuntimeError::Cancelled.into()).await),
                    ActionKind::Reconcile | ActionKind::Wait(_) => return ran(attempt.abandon(&contract("study policy requires reconciliation or input acquisition before native dispatch")).await),
                }
            }
        };
        let result = tokio::select! {
            result = handle.wait() => result?,
            () = cancel.cancelled() => {
                handle.cancel();
                handle.wait().await?
            }
        };
        let record = match result.durability() {
            RunDurability::Durable(record) => record.clone(),
            RunDurability::Ephemeral => {
                return Err(contract("a claimed job ran without its durable attempt"));
            }
        };
        Ok((Processed::Ran { job, record }, Some(result)))
    }

    /// Serve the queue until `stop` fires, `settings.jobs` jobs were processed, or (with
    /// `until_idle`) no job is available. Expired leases are swept on start and every
    /// `settings.recovery`.
    ///
    /// # Errors
    /// As for [`Runtime::work_once`], and store failures of the sweep.
    pub async fn serve(
        &self,
        settings: WorkerSettings,
        stop: &crate::CancelSource,
    ) -> Result<Vec<Processed>, WorkflowError> {
        let operations = operations(self)?;
        let mut processed = Vec::new();
        let mut swept = tokio::time::Instant::now();
        operations.recover().await?;
        while !stop.token().is_cancelled()
            && settings.jobs.is_none_or(|limit| processed.len() < limit)
        {
            if swept.elapsed() >= settings.recovery {
                operations.recover().await?;
                swept = tokio::time::Instant::now();
            }
            match self.work_once().await? {
                Processed::Idle if settings.until_idle => break,
                Processed::Idle => {
                    tokio::select! {
                        () = tokio::time::sleep(settings.poll) => {}
                        () = stop.cancelled() => {}
                    }
                }
                ran => processed.push(ran),
            }
        }
        Ok(processed)
    }

    /// Load a modeling job's sources from the store, prepare its case (a study point's with
    /// its overlay) and apply its start policy (a study point with a predecessor starts
    /// from the predecessor's stored solution). A study point also returns where its try
    /// writes its result members.
    async fn prepare_job(
        &self,
        operations: &Operations,
        claimed: &ClaimedJob,
        job: ModelingJob,
        cancel: &crate::CancelSource,
    ) -> Result<
        (
            super::ModelingSolvePreparation,
            AppliedStart,
            Option<super::study::PointContext>,
        ),
        WorkflowError,
    > {
        let solver = job.settings.profile().map_err(WorkflowError::Math)?;
        let physical = self
            .physical_from_sources(&operations.sources(&job.physical).await?, cancel)
            .await?;
        let mut modeling = Vec::with_capacity(job.modeling.len());
        for bundle in &job.modeling {
            modeling.push(operations.sources(bundle).await?);
        }
        let package = self.package_from_sources(&modeling, physical)?;
        let execution = package
            .declared_execution(
                job.case,
                Default::default(),
                solver,
                Default::default(),
                pse_modeling::Limits::default(),
                cancel,
            )
            .await?;
        if execution.route != job.route {
            return Err(contract(
                "durable job route differs from its authored execution",
            ));
        }
        if !matches!(
            execution.procedure,
            super::modeling::DeclaredProcedure::Solve
        ) {
            return Err(contract(
                "durable algebraic job requires the authored solve procedure",
            ));
        }
        let analysis = execution.analysis;
        let prepared = package.prepare_analysis(&analysis, cancel).await?;
        let (prepared, applied) = start(operations, claimed, job.start, prepared).await?;
        Ok((prepared, applied, None))
    }

    async fn prepare_study_job(
        &self,
        operations: &Operations,
        job: &StudyOperationJob,
        cancel: &crate::CancelSource,
    ) -> Result<
        (
            super::PreparedStudyOperation,
            Option<pse_model::study::SeedFact>,
            Option<WorkflowError>,
        ),
        WorkflowError,
    > {
        use pse_model::study::{SeedAvailability, SeedFact, SeedNeed, StartPolicy};
        if job.point.binding.identity() != job.point.binding_hash
            || job.point.policy.key.0 != job.point.point_index
        {
            return Err(contract("study job binding or occurrence identity differs"));
        }
        let stored = operations.store().studies().row(job.point.study_id).await?;
        let definition = super::StudyDefinition::readmission(&stored.definition)?;
        definition.validate_roles()?;
        let defined = definition
            .points
            .iter()
            .find(|point| point.policy.key == job.point.policy.key)
            .ok_or_else(|| contract("study occurrence is absent from its immutable definition"))?;
        let replay = super::StudyPointDefinition {
            operation: job.point.operation.clone(),
            binding_hash: job.point.binding_hash,
            binding: job.point.binding.clone(),
            policy: job.point.policy.clone(),
        };
        let checksum = |point: &super::StudyPointDefinition| {
            pse_ids::document::of(pse_ids::Frame::DurableJobRequestV4, point)
                .map_err(|error| contract(error.to_string()))
        };
        if checksum(defined)? != checksum(&replay)?
            || definition.physical != job.physical
            || definition.modeling != job.modeling
        {
            return Err(contract("study job differs from the immutable definition"));
        }
        let physical = self
            .physical_from_sources(&operations.sources(&job.physical).await?, cancel)
            .await?;
        let mut modeling = Vec::with_capacity(job.modeling.len());
        for bundle in &job.modeling {
            modeling.push(operations.sources(bundle).await?);
        }
        let package = self.package_from_sources(&modeling, physical)?;
        let prepared = package
            .prepare_bound_operation(&job.point.operation, &job.point.binding, cancel)
            .await?;
        if prepared.seed_need() != job.point.policy.seed_need {
            return Err(contract("immutable study operation seed need differs"));
        }
        if prepared.seed_need() == SeedNeed::NotNeeded
            || matches!(job.point.policy.start, StartPolicy::Fresh)
        {
            return Ok((prepared, None, None));
        }
        let super::PreparedStudyOperation::DeclaredCase(case) = prepared else {
            return Err(contract("operation has no admitted seed input"));
        };
        let (role, found) = match &job.point.policy.start {
            StartPolicy::Explicit { role, seed } => (*role, Some(*seed)),
            StartPolicy::Continuation(edge) => {
                let predecessor = operations
                    .store()
                    .studies()
                    .point(job.point.study_id, edge.predecessor.0)
                    .await?;
                let target = case
                    .solve
                    .compatibility()
                    .ok_or_else(|| contract("missing seed compatibility"))?;
                let preparation = case
                    .solve
                    .seed_preparation_identity()
                    .ok_or_else(|| contract("missing seed preparation"))?;
                let stored = operations
                    .store()
                    .solutions()
                    .latest_of_attempt(
                        predecessor.attempt_id,
                        &target.layout,
                        &preparation,
                        target.backend,
                    )
                    .await?;
                (edge.role, stored.map(|stored| stored.solution_id))
            }
            StartPolicy::Fresh => {
                return Ok((
                    super::PreparedStudyOperation::DeclaredCase(case),
                    None,
                    None,
                ));
            }
        };
        let Some(seed) = found else {
            return Ok((
                super::PreparedStudyOperation::DeclaredCase(case),
                Some(SeedFact {
                    role,
                    availability: SeedAvailability::Absent,
                }),
                None,
            ));
        };
        match case
            .as_ref()
            .clone()
            .with_stored_start(operations, super::StoredStart::Solution(seed))
            .await
        {
            Ok(seeded) => Ok((
                super::PreparedStudyOperation::DeclaredCase(Box::new(seeded)),
                Some(SeedFact {
                    role,
                    availability: SeedAvailability::Compatible { seed },
                }),
                None,
            )),
            Err(error) => {
                let availability = if matches!(
                    error,
                    WorkflowError::Operations(pse_operations::OperationsError::NotFound { .. })
                ) {
                    SeedAvailability::Absent
                } else if error.boundary_diagnostic().class
                    == pse_model::diagnostic::BoundaryClass::Incompatible
                {
                    SeedAvailability::Incompatible
                } else {
                    SeedAvailability::InternalFailure
                };
                Ok((
                    super::PreparedStudyOperation::DeclaredCase(case),
                    Some(SeedFact { role, availability }),
                    Some(error),
                ))
            }
        }
    }

    pub(super) async fn physical_from_sources(
        &self,
        sources: &BTreeMap<String, Vec<u8>>,
        cancel: &crate::CancelSource,
    ) -> Result<PhysicalContext, WorkflowError> {
        let validation = self.sessions.validation_context(&self.registry)?;
        let pool = self.shared.pool();
        let token = cancel.token();
        let bundle = load_package_documents_owned(
            sources,
            &self.registry,
            pse_authoring::ParseBudget::default(),
            &pool,
            &token,
            &validation,
        )?;
        let documents = OwnedDocumentSet::try_from_bundles(vec![bundle], &pool, &token)?;
        self.physical_from_documents(&documents, &token).await
    }

    pub(super) fn package_from_sources(
        &self,
        bundles: &[BTreeMap<String, Vec<u8>>],
        physical: PhysicalContext,
    ) -> Result<super::ModelingPackage, WorkflowError> {
        // Source bytes and load order define the complete immutable admission closure.
        // The cache owns only admitted values; every attempt has its own mutable workspace.
        let service = self.shared.math();
        let generation = service.modeling_cache.generation();
        let mut framed = pse_ids::FramedHasher::new(pse_ids::Frame::ModelingPackageAdmissionV1);
        framed.u64(bundles.len() as u64).hash(&physical.key)
            // Local cache authority is the exact immutable validation assembly and registry.
            // Its owner is retained with the admission, preventing pointer reuse. These
            // process-local slots never enter source/scientific/operational identities.
            .u64(Arc::as_ptr(&self.sessions) as usize as u64)
            .u64(Arc::as_ptr(&self.registry) as usize as u64);
        for sources in bundles {
            framed.hash(&package_checksum(sources));
        }
        match &physical.package {
            Some(package) => {
                use pse_model::SemanticFrame;
                framed.u64(1);
                package.header.frame(&mut framed);
            }
            None => {
                framed.u64(0);
            }
        }
        let identity = pse_ids::roles::AdmittedClosureHash::from_id(framed.finish_hash());
        if let Some(admitted) = service.modeling_cache.package(identity) {
            return self.package_from_admission(admitted, physical);
        }
        let validation = self.sessions.validation_context(&self.registry)?;
        let pool = self.shared.pool();
        let token = pse_columnar::CancellationToken::new();
        let bundles = bundles
            .iter()
            .map(|sources| {
                load_package_documents_owned(
                    sources,
                    &self.registry,
                    pse_authoring::ParseBudget::default(),
                    &pool,
                    &token,
                    &validation,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let documents = OwnedDocumentSet::try_from_bundles(bundles, &pool, &token)?;
        let package = self.modeling_from_documents(&documents, physical)?;
        service
            .modeling_cache
            .retain_package(generation, identity, package.admission()?);
        Ok(package)
    }
}

/// Decode a claimed job's payload: the task of a known payload version.
///
/// # Errors
/// An unknown payload version, or a document this version does not describe.
fn decode(claimed: &ClaimedJob) -> Result<JobTask, WorkflowError> {
    if claimed.payload_version != JOB_PAYLOAD_VERSION {
        return Err(WorkflowError::UnknownPayloadVersion {
            version: claimed.payload_version,
            supported: JOB_PAYLOAD_VERSION,
        });
    }
    let header = <DocumentVersion as serde::Deserialize>::deserialize(&claimed.payload)
        .map_err(|error| contract(format!("job payload version: {error}")))?;
    if header.version != Version::<{ JOB_PAYLOAD_VERSION as u32 }>::NUMBER {
        let version = i32::try_from(header.version)
            .map_err(|error| contract(format!("job payload version: {error}")))?;
        return Err(WorkflowError::UnknownPayloadVersion {
            version,
            supported: JOB_PAYLOAD_VERSION,
        });
    }
    let payload: JobPayload = serde_json::from_value(claimed.payload.clone())
        .map_err(|e| contract(format!("job payload version {JOB_PAYLOAD_VERSION}: {e}")))?;
    Ok(payload.task)
}

/// How a claimed job's solve started: the policy its payload requested (`fresh`,
/// `resume_from_parent`, `stored_solution`, or `predecessor` for a study point) and the
/// stored solution it started from, if any. It is recorded as the attempt's first progress
/// event (`job.start`), beside the start receipt and lineage of the result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AppliedStart {
    pub(super) requested: &'static str,
    pub(super) solution: Option<SolutionId>,
    /// Why the solve started fresh instead of from a stored solution.
    pub(super) fresh: Option<String>,
}

impl AppliedStart {
    fn event(&self) -> pse_backend_native::solve::Event {
        use pse_backend_native::solve::{Metric, UnavailableReason};
        let mut values = BTreeMap::from([
            (
                "requested".to_owned(),
                Metric::Text(self.requested.to_owned()),
            ),
            (
                "solution".to_owned(),
                self.solution.map_or(
                    Metric::Unavailable(UnavailableReason::NotApplicable),
                    |solution| Metric::Text(solution.to_string()),
                ),
            ),
        ]);
        if let Some(reason) = &self.fresh {
            values.insert("fresh".to_owned(), Metric::Text(reason.clone()));
        }
        pse_backend_native::solve::Event {
            phase: "job.start".into(),
            elapsed: Duration::ZERO,
            values,
            incumbent: None,
        }
    }
}

/// Apply a job's start policy to its prepared case (Plan 22 G8). `ResumeFromParent`
/// starts from the newest solution captured in the parent attempt chain for this
/// preparation's coordinates and backend (an incumbent stored while a superseded try ran),
/// and starts fresh, recorded, when there is none; `StoredSolution` starts from that
/// solution. A stored start reaches the backend as an explicit warm start: SCIP injects its
/// primal as an incumbent, HiGHS takes it as a MIP start.
///
/// # Errors
/// A stored solution that is unknown or whose coordinates or backend differ; store
/// failures.
async fn start(
    operations: &Operations,
    claimed: &ClaimedJob,
    requested: JobStart,
    prepared: super::ModelingSolvePreparation,
) -> Result<(super::ModelingSolvePreparation, AppliedStart), WorkflowError> {
    let applied = |solution, fresh: Option<&str>| AppliedStart {
        requested: match requested {
            JobStart::Fresh => "fresh",
            JobStart::ResumeFromParent => "resume_from_parent",
            JobStart::StoredSolution { .. } => "stored_solution",
        },
        solution,
        fresh: fresh.map(str::to_owned),
    };
    let solution = match requested {
        JobStart::Fresh => return Ok((prepared, applied(None, None))),
        JobStart::StoredSolution { solution } => solution,
        JobStart::ResumeFromParent => {
            let Some(parent) = claimed.parent_attempt else {
                return Ok((
                    prepared,
                    applied(None, Some("first try: no parent attempt")),
                ));
            };
            let (Some(target), Some(preparation)) = (
                prepared.solve.compatibility(),
                prepared.solve.seed_preparation_identity(),
            ) else {
                return Ok((
                    prepared,
                    applied(None, Some("a constant evaluation consumes no seed")),
                ));
            };
            let found = operations
                .store()
                .solutions()
                .latest_in_attempt_chain(parent, &target.layout, &preparation, target.backend)
                .await?;
            let Some(found) = found else {
                tracing::info!(%parent, "no compatible incumbent in the parent chain; fresh start");
                return Ok((
                    prepared,
                    applied(
                        None,
                        Some("no compatible incumbent solution in the parent attempt chain"),
                    ),
                ));
            };
            found.solution_id
        }
    };
    let seeded = prepared
        .with_stored_start(operations, super::StoredStart::Solution(solution))
        .await?;
    Ok((seeded, applied(Some(solution), None)))
}

/// Payload decoding is pure: a claimed job's version column and its document decide the
/// task, with no store, package or solver (Plan 22 S25).
#[cfg(test)]
mod decode_tests {
    use super::*;

    fn claimed(version: i32, payload: serde_json::Value) -> ClaimedJob {
        ClaimedJob {
            job_id: pse_operations::mint_id(),
            attempt_id: pse_operations::mint_id(),
            run_id: pse_operations::mint_id(),
            parent_attempt: None,
            payload_version: version,
            payload,
            try_number: 1,
            lease_expires_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn payload_decoding_is_pure() {
        let study: StudyId = pse_operations::mint_id();
        let payload = serde_json::to_value(JobPayload::new(JobTask::StudyFinalization(
            StudyFinalization { study_id: study },
        )))
        .unwrap();
        // The current version decodes its task.
        let Ok(JobTask::StudyFinalization(task)) =
            decode(&claimed(JOB_PAYLOAD_VERSION, payload.clone()))
        else {
            panic!("the current payload decodes")
        };
        assert_eq!(task.study_id, study);
        // Another version column is refused by version, before its document is read.
        let error = decode(&claimed(JOB_PAYLOAD_VERSION + 1, serde_json::json!({})));
        assert!(
            matches!(
                error,
                Err(WorkflowError::UnknownPayloadVersion { version, supported })
                    if version == JOB_PAYLOAD_VERSION + 1 && supported == JOB_PAYLOAD_VERSION
            ),
            "{error:?}"
        );
        // A known column cannot grant admission to a document of another version.
        let mut restated = payload;
        restated["version"] = serde_json::json!(JOB_PAYLOAD_VERSION + 1);
        assert!(matches!(
            decode(&claimed(JOB_PAYLOAD_VERSION, restated)),
            Err(WorkflowError::UnknownPayloadVersion { .. })
        ));
        let error = decode(&claimed(
            JOB_PAYLOAD_VERSION,
            serde_json::json!({ "from": "a newer build" }),
        ));
        assert!(matches!(error, Err(WorkflowError::Input(_))), "{error:?}");
    }

    #[test]
    fn historical_payload_refusal_precedes_nested_scientific_decode() {
        let historical = serde_json::json!({
            "version": 5,
            "task": {
                "kind": "study_operation",
                "point": {
                    "operation": {
                        "version": 1,
                        "preparation": {"compiler": {"evaluation": {}, "optimization": {}}}
                    }
                }
            }
        });
        for column in [5, JOB_PAYLOAD_VERSION] {
            let claim = claimed(column, historical.clone());
            let error = decode(&claim).unwrap_err();
            assert!(matches!(
                error,
                WorkflowError::UnknownPayloadVersion { version: 5, supported }
                    if supported == JOB_PAYLOAD_VERSION
            ));
            assert_eq!(claim.payload, historical);
        }
    }
}
