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
    Durability, ModelingAnalysis, Operations, PhysicalContext, RunDurability, Runtime,
    WorkflowError, contract,
    durable::{Claim, DurableAttempt, DurableRecord},
};
use crate::authoring_driver::document::{
    OwnedDocumentSet, load_package_texts_owned, package_checksum,
};
use crate::math::settings::SolveSettings;
use pse_ids::{ContentHash, SemanticId};
use pse_model::{document::Version, generated::enums::ModelingAnalysisRoute};
use pse_operations::{
    attempts::{AttemptKind, NewAttempt},
    jobs::{ClaimedJob, Enqueued, NewJob, RetryPolicy},
    lifecycle::AttemptState,
    solutions::SolutionId,
    sources::{SourceBundle, SourceDocument},
};
use std::{collections::BTreeMap, sync::Arc, time::Duration};

/// The payload version of [`ModelingJob`] this build executes: the store's
/// `payload_version` column and the document's own `version`.
pub const MODELING_JOB_VERSION: i32 = 2;

/// How a job's solve is started.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
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

/// Version 2 of a modeling job: one authored case of a package closure, solved once under
/// typed solve settings. Unknown fields and versions are refused.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModelingJob {
    /// Document version.
    pub version: Version<2>,
    /// The source bundle of the physical package.
    pub physical: ContentHash,
    /// The source bundles of the modeling package closure, in load order.
    pub modeling: Vec<ContentHash>,
    /// The authored case to solve.
    pub case: SemanticId,
    /// Its analysis route.
    pub route: ModelingAnalysisRoute,
    /// The solve settings.
    pub settings: SolveSettings,
    /// How the solve starts.
    #[serde(default)]
    pub start: JobStart,
}

impl ModelingJob {
    /// The request identity of this job: its typed document framed through the serde
    /// data model, so neither key order nor the build graph's JSON features move it.
    ///
    /// # Errors
    /// A settings serializer refused its value.
    pub fn request_identity(&self) -> Result<ContentHash, WorkflowError> {
        pse_backend_native::identity::of(pse_ids::Frame::DurableJobRequestV2, self).map_err(
            |e| WorkflowError::Math(crate::math::MathRuntimeError::from(e)),
        )
    }
}

/// Version 1 of a source bundle's manifest: the path of every document, in order.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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
    /// Store authored documents as a content-addressed source bundle and return its §6.1
    /// package content hash. Storing the same documents again is a no-op.
    ///
    /// # Errors
    /// Store failures.
    pub async fn put_sources(
        &self,
        texts: &BTreeMap<String, String>,
    ) -> Result<ContentHash, WorkflowError> {
        let bundle_hash = package_checksum(texts);
        let documents = texts
            .iter()
            .map(|(path, text)| SourceDocument {
                path: path.clone(),
                content_hash: pse_ids::encoding_checksum(text.as_bytes()).content_hash(),
                content: text.clone(),
            })
            .collect();
        self.store()
            .sources()
            .put(&SourceBundle {
                bundle_hash: bundle_hash.into(),
                manifest: serde_json::to_value(SourceManifest {
                    version: Version,
                    paths: texts.keys().cloned().collect(),
                })
                .map_err(|e| contract(format!("source manifest: {e}")))?,
                documents,
            })
            .await?;
        Ok(bundle_hash)
    }

    /// Read a source bundle and verify its package content hash and every document hash.
    ///
    /// # Errors
    /// An unknown bundle, a store failure, or content that does not match its hashes.
    pub async fn sources(
        &self,
        bundle: &ContentHash,
    ) -> Result<BTreeMap<String, String>, WorkflowError> {
        let stored = self.store().sources().get(&(*bundle).into()).await?;
        let mut texts = BTreeMap::new();
        for document in stored.documents {
            if pse_ids::encoding_checksum(document.content.as_bytes()).content_hash()
                != document.content_hash
            {
                return Err(contract(format!(
                    "source document `{}` of bundle {} does not match its content hash",
                    document.path,
                    bundle.to_prefixed()
                )));
            }
            texts.insert(document.path, document.content);
        }
        if package_checksum(&texts) != *bundle {
            return Err(contract(format!(
                "source bundle {} does not match its package content hash",
                bundle.to_prefixed()
            )));
        }
        Ok(texts)
    }

    /// Enqueue a modeling job with its first attempt. The idempotency key names the logical
    /// request: enqueuing the same key again returns the existing job. The request identity
    /// is the typed payload's ([`ModelingJob::request_identity`]).
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
        let request_identity = job.request_identity()?;
        let payload =
            serde_json::to_value(job).map_err(|e| contract(format!("job payload: {e}")))?;
        Ok(self
            .store()
            .jobs()
            .enqueue(&NewJob {
                attempt: NewAttempt {
                    attempt_id: pse_operations::mint_id(),
                    run_id: pse_operations::mint_id(),
                    kind: AttemptKind::Modeling,
                    request_identity,
                    preparation_identity: None,
                    parent_attempt: None,
                },
                idempotency_key: idempotency_key.to_owned(),
                payload_version: MODELING_JOB_VERSION,
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
            },
        );
        let stop = cancel.clone();
        if let Err(error) = attempt.start(Arc::new(move || stop.cancel())).await {
            let record = attempt.abandon(&error).await;
            return Ok((
                Processed::Ran {
                    job,
                    record: Box::new(record),
                },
                None,
            ));
        }
        // A cancellation request stops the try in any phase, including source loading.
        let preparation = tokio::select! {
            prepared = self.prepare_job(operations, &claimed, &cancel) => prepared,
            () = cancel.cancelled() => Err(WorkflowError::Math(
                crate::math::MathRuntimeError::Cancelled,
            )),
        };
        let (prepared, applied) = match preparation {
            Ok(prepared) => prepared,
            Err(error) => {
                let record = attempt.abandon(&error).await;
                return Ok((
                    Processed::Ran {
                        job,
                        record: Box::new(record),
                    },
                    None,
                ));
            }
        };
        // The attempt's stream records how the solve started, before its native events.
        attempt.tap().observe(&applied.event());
        let handle = self.start_attempt(vec![prepared], attempt)?;
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

    /// Decode a claimed payload, load its sources from the store, prepare its case and
    /// apply its start policy.
    async fn prepare_job(
        &self,
        operations: &Operations,
        claimed: &ClaimedJob,
        cancel: &crate::CancelSource,
    ) -> Result<(super::ModelingSolvePreparation, AppliedStart), WorkflowError> {
        if claimed.payload_version != MODELING_JOB_VERSION {
            return Err(WorkflowError::UnknownPayloadVersion {
                version: claimed.payload_version,
                supported: MODELING_JOB_VERSION,
            });
        }
        let job: ModelingJob = serde_json::from_value(claimed.payload.clone())
            .map_err(|e| contract(format!("job payload version {MODELING_JOB_VERSION}: {e}")))?;
        let solver = job.settings.profile().map_err(WorkflowError::Math)?;
        let physical = self
            .physical_from_sources(&operations.sources(&job.physical).await?, cancel)
            .await?;
        let mut modeling = Vec::with_capacity(job.modeling.len());
        for bundle in &job.modeling {
            modeling.push(operations.sources(bundle).await?);
        }
        let package = self.package_from_sources(&modeling, physical)?;
        let analysis: ModelingAnalysis = package
            .declared_analysis(
                job.case.into(),
                job.route,
                Default::default(),
                solver,
                Default::default(),
                pse_modeling::Limits::default(),
                cancel,
            )
            .await?;
        let prepared = package.prepare_analysis(&analysis, cancel).await?;
        start(operations, claimed, job.start, prepared).await
    }

    pub(super) async fn physical_from_sources(
        &self,
        texts: &BTreeMap<String, String>,
        cancel: &crate::CancelSource,
    ) -> Result<PhysicalContext, WorkflowError> {
        let pool = self.shared.pool();
        let token = cancel.token();
        let bundle = load_package_texts_owned(
            texts,
            &self.registry,
            pse_authoring::ParseBudget::default(),
            &pool,
            &token,
        )?;
        let documents = OwnedDocumentSet::try_from_bundles(vec![bundle], &pool, &token)?;
        self.physical_from_documents(&documents, &token).await
    }

    pub(super) fn package_from_sources(
        &self,
        bundles: &[BTreeMap<String, String>],
        physical: PhysicalContext,
    ) -> Result<super::ModelingPackage, WorkflowError> {
        let pool = self.shared.pool();
        let token = pse_columnar::CancellationToken::new();
        let bundles = bundles
            .iter()
            .map(|texts| {
                load_package_texts_owned(
                    texts,
                    &self.registry,
                    pse_authoring::ParseBudget::default(),
                    &pool,
                    &token,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let documents = OwnedDocumentSet::try_from_bundles(bundles, &pool, &token)?;
        self.modeling_from_documents(&documents, physical)
    }
}

/// How a claimed job's solve started: the policy its payload requested and the stored
/// solution it started from, if any. It is recorded as the attempt's first progress event
/// (`job.start`), beside the start receipt and lineage of the result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AppliedStart {
    requested: JobStart,
    solution: Option<SolutionId>,
    /// Why a resume started fresh instead.
    fresh: Option<&'static str>,
}

impl AppliedStart {
    fn event(&self) -> pse_backend_native::solve::Event {
        use pse_backend_native::solve::{Metric, UnavailableReason};
        let requested = match self.requested {
            JobStart::Fresh => "fresh",
            JobStart::ResumeFromParent => "resume_from_parent",
            JobStart::StoredSolution { .. } => "stored_solution",
        };
        let mut values = BTreeMap::from([
            ("requested".to_owned(), Metric::Text(requested.to_owned())),
            (
                "solution".to_owned(),
                self.solution.map_or(
                    Metric::Unavailable(UnavailableReason::NotApplicable),
                    |solution| Metric::Text(solution.to_string()),
                ),
            ),
        ]);
        if let Some(reason) = self.fresh {
            values.insert("fresh".to_owned(), Metric::Text(reason.to_owned()));
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
    let applied = |solution, fresh| AppliedStart {
        requested,
        solution,
        fresh,
    };
    let solution = match requested {
        JobStart::Fresh => return Ok((prepared, applied(None, None))),
        JobStart::StoredSolution { solution } => solution,
        JobStart::ResumeFromParent => {
            let Some(parent) = claimed.parent_attempt else {
                return Ok((prepared, applied(None, Some("first try: no parent attempt"))));
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
