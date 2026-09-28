// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The durable job queue from the runtime's side (ADR-0112 Outcomes 14, 15 and 18; Plan 22
//! O4): versioned job payloads over content-addressed source bundles, and the worker that
//! claims jobs, runs them under its lease and ends each try through the job's retry
//! policy.
//!
//! A job payload names its authored sources by the §6.1 package content hash, never by a
//! path, plus the case and a serialized solver profile. A worker refuses a payload version
//! it does not know. The durable `cancel_requested` flag is the cancellation authority: the
//! claimed attempt's heartbeat returns it, and a `LISTEN` watcher that re-reads it after
//! every reconnect only shortens latency.
use super::{
    Durability, ModelingAnalysis, Operations, PhysicalContext, RunDurability, Runtime,
    WorkflowError, contract,
    durable::{Claim, DurableAttempt, DurableRecord},
};
use crate::authoring_driver::document::{
    OwnedDocumentSet, load_package_texts_owned, package_checksum,
};
use crate::math::solves::SolverProfile;
use pse_backend_native::{
    execution::BackendSettings,
    solve::{Backend, Controls, SolveIntent, SolverSelection},
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_model::generated::enums::ModelingAnalysisRoute;
use pse_operations::{
    attempts::{AttemptKind, NewAttempt},
    jobs::{ClaimedJob, Enqueued, NewJob, RetryPolicy},
    lifecycle::AttemptState,
    sources::{SourceBundle, SourceDocument},
};
use std::{collections::BTreeMap, sync::Arc, time::Duration};

/// The payload version of [`ModelingJob`] this build executes.
pub const MODELING_JOB_VERSION: i32 = 1;

/// The library presolve a job requests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobPresolve {
    /// Preserve source coordinates through an identity transport.
    Off,
    /// Only qualified source-backed passes.
    #[default]
    Auto,
}

/// The shared solver controls a job carries; backend settings are the adapter defaults.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobProfile {
    /// Mathematical purpose.
    pub intent: SolveIntent,
    /// An explicit backend, or deterministic routing when absent.
    pub backend: Option<Backend>,
    /// The library presolve.
    pub presolve: JobPresolve,
    /// Wall-clock allowance of each step, in seconds.
    pub time_limit_seconds: f64,
    /// Native iteration limit.
    pub iterations: u32,
    /// Admitted native threads.
    pub threads: usize,
    /// Progress events retained in memory; the durable stream keeps every event.
    pub history: usize,
}

impl Default for JobProfile {
    fn default() -> Self {
        let controls = Controls::default();
        Self {
            intent: SolveIntent::Root,
            backend: None,
            presolve: JobPresolve::Auto,
            time_limit_seconds: controls.time_limit.as_secs_f64(),
            iterations: controls.iterations,
            threads: controls.threads,
            history: controls.history,
        }
    }
}

impl JobProfile {
    /// The solver profile this job runs under.
    ///
    /// # Errors
    /// A time limit that is not a positive finite number of seconds.
    pub fn solver(&self) -> Result<SolverProfile, WorkflowError> {
        let time_limit = Duration::try_from_secs_f64(self.time_limit_seconds)
            .ok()
            .filter(|d| !d.is_zero())
            .ok_or_else(|| contract("job time limit must be a positive number of seconds"))?;
        Ok(SolverProfile {
            presolve: match self.presolve {
                JobPresolve::Off => pse_backend_native::presolve::Policy::Off,
                JobPresolve::Auto => pse_backend_native::presolve::Policy::Auto,
            },
            numerics: Default::default(),
            convexity: Default::default(),
            intent: self.intent,
            selection: self
                .backend
                .map_or(SolverSelection::Auto, SolverSelection::Explicit),
            controls: Controls {
                time_limit,
                iterations: self.iterations,
                threads: self.threads,
                history: self.history,
                ..Controls::default()
            },
            backend: BackendSettings::Default,
        })
    }
}

/// Version 1 of a modeling job: one authored case of a package closure, solved once.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelingJob {
    /// The source bundle of the physical package.
    pub physical: ContentHash,
    /// The source bundles of the modeling package closure, in load order.
    pub modeling: Vec<ContentHash>,
    /// The authored case to solve.
    pub case: SemanticId,
    /// Its analysis route.
    pub route: ModelingAnalysisRoute,
    /// The solver profile.
    pub profile: JobProfile,
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
            "the job queue needs a durable runtime (ADR-0112 Outcome 16)",
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
                bundle_hash,
                manifest: serde_json::json!({ "paths": texts.keys().collect::<Vec<_>>() }),
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
        let stored = self.store().sources().get(bundle).await?;
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
    /// request: enqueuing the same key again returns the existing job.
    ///
    /// # Errors
    /// Store failures and an invalid retry policy.
    pub async fn enqueue_modeling(
        &self,
        job: &ModelingJob,
        idempotency_key: &str,
        retry: RetryPolicy,
        priority: i32,
    ) -> Result<Enqueued, WorkflowError> {
        let payload =
            serde_json::to_value(job).map_err(|e| contract(format!("job payload: {e}")))?;
        self.enqueue(
            MODELING_JOB_VERSION,
            payload,
            idempotency_key,
            retry,
            priority,
        )
        .await
    }

    /// Enqueue a job payload of an explicit version, which a worker may refuse.
    ///
    /// # Errors
    /// Store failures and an invalid retry policy or version.
    pub async fn enqueue(
        &self,
        payload_version: i32,
        payload: serde_json::Value,
        idempotency_key: &str,
        retry: RetryPolicy,
        priority: i32,
    ) -> Result<Enqueued, WorkflowError> {
        let mut identity = FramedHasher::new(pse_ids::Frame::DurableJobRequestV1);
        identity
            .u64(u64::try_from(payload_version).unwrap_or(0))
            .str(&payload.to_string());
        Ok(self
            .store()
            .jobs()
            .enqueue(&NewJob {
                attempt: NewAttempt {
                    attempt_id: pse_operations::mint_id(),
                    run_id: pse_operations::mint_id(),
                    kind: AttemptKind::Modeling,
                    request_identity: identity.finish_hash(),
                    preparation_identity: None,
                    parent_attempt: None,
                },
                idempotency_key: idempotency_key.to_owned(),
                payload_version,
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
        let operations = operations(self)?;
        let Some(claimed) = operations
            .store()
            .jobs()
            .claim(operations.worker(), operations.policy().lease)
            .await?
        else {
            return Ok(Processed::Idle);
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
            return Ok(Processed::Ran {
                job,
                record: Box::new(record),
            });
        }
        // A cancellation request stops the try in any phase, including source loading.
        let preparation = tokio::select! {
            prepared = self.prepare_job(operations, &claimed, &cancel) => prepared,
            () = cancel.cancelled() => Err(WorkflowError::Math(
                crate::math::MathRuntimeError::Cancelled,
            )),
        };
        let prepared = match preparation {
            Ok(prepared) => prepared,
            Err(error) => {
                let record = attempt.abandon(&error).await;
                return Ok(Processed::Ran {
                    job,
                    record: Box::new(record),
                });
            }
        };
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
        Ok(Processed::Ran { job, record })
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

    /// Decode a claimed payload, load its sources from the store and prepare its case.
    async fn prepare_job(
        &self,
        operations: &Operations,
        claimed: &ClaimedJob,
        cancel: &crate::CancelSource,
    ) -> Result<super::ModelingSolvePreparation, WorkflowError> {
        if claimed.payload_version != MODELING_JOB_VERSION {
            return Err(WorkflowError::UnknownPayloadVersion {
                version: claimed.payload_version,
                supported: MODELING_JOB_VERSION,
            });
        }
        let job: ModelingJob = serde_json::from_value(claimed.payload.clone())
            .map_err(|e| contract(format!("job payload version {MODELING_JOB_VERSION}: {e}")))?;
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
                job.case,
                job.route,
                Default::default(),
                job.profile.solver()?,
                Default::default(),
                pse_modeling::Limits::default(),
                cancel,
            )
            .await?;
        package.prepare_analysis(&analysis, cancel).await
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
