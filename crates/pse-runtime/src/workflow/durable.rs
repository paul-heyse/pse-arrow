// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Canonical claims precede native effects; immutable closed manifests own stored observations.
use super::connected_results::{pooled_response_owner, retain_metadata, retain_protected_metadata};
use super::{RunReport, RunRequest, RunResult, WorkflowError, contract, relation};
use datafusion::execution::memory_pool::{MemoryConsumer, MemoryPool};
use pse_backend_native::solve::{Compatibility, Event, ProgressTap, WarmPayload, WarmStart};
use pse_ids::{ContentHash, Frame, FramedHasher, SemanticId};
use pse_model::generated::{
    enums::{AttemptState, CandidateUse, NativeBackend},
    identities::{AttemptId, RunId, SolutionId},
};
use pse_operations::{
    canonical::{CanonicalOptions, CanonicalStore, Revision},
    canonical_execution::{
        AppendAck, AttemptFence, CanonicalAttempt, CanonicalRun, ClosedAttempt, EmptySetAck,
        RESULT_ACKNOWLEDGMENT_BYTES, RESULT_BATCH_BYTES, ResultAcknowledgments, ResultManifest,
        SeedAck, SeedEligibility, TerminalClass, execution_attempt_key, result_batch_key,
        result_payload_digest, result_set_key,
    },
    canonical_results::ResultRead,
};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering},
    },
    time::Duration,
};

/// A requested immutable seed is absent; storage failures retain their original cause.
#[derive(Debug, thiserror::Error)]
pub enum SeedReadError {
    /// No eligible retained header exists for this explicit scientific artifact.
    #[error("stored seed {solution} is unavailable")]
    Missing {
        /// Exact requested artifact identity.
        solution: SolutionId,
    },
}
pse_diagnostics::impl_diagnostic! {
    SeedReadError,
    code(_this){Some(pse_diagnostics::DiagnosticCode::StudySeedUnavailable)},
    forward(_this){None},help(_this){None},related(_this){None},source(_this){None}
}
/// Finite worker lease and bounded progress queue policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeasePolicy {
    /// Positive server authority lifetime.
    pub lease: Duration,
    /// Positive interval between authority renewals.
    pub heartbeat: Duration,
    /// Bounded observed-event queue capacity.
    pub batch: usize,
    /// Maximum event flush delay.
    pub flush: Duration,
}
impl Default for LeasePolicy {
    fn default() -> Self {
        Self {
            lease: Duration::from_secs(30),
            heartbeat: Duration::from_secs(10),
            batch: 64,
            flush: Duration::from_millis(200),
        }
    }
}
/// The sole canonical store, worker identity and renewal policy.
#[derive(Clone, Debug)]
pub struct Operations {
    store: CanonicalStore,
    worker: Arc<str>,
    policy: LeasePolicy,
    pub(super) pool: Arc<dyn MemoryPool>,
}
/// Explicit expiry reconciliation receipts; recovery never invents scientific success.
#[derive(Clone, Debug, Default)]
pub struct Recovery {
    /// Exact attempt keys reconciled without scientific rerun.
    pub recovered: Vec<String>,
}
impl Operations {
    /// Connect and open the sole canonical deployment with an accounted memory pool.
    pub async fn connect(
        options: &CanonicalOptions,
        worker: impl Into<String>,
        policy: LeasePolicy,
        pool: Arc<dyn MemoryPool>,
    ) -> Result<Self, WorkflowError> {
        let store = CanonicalStore::connect(options).await?;
        store.open().await?;
        Ok(Self::from_store(store, worker, policy, pool))
    }
    /// Bind an already opened deployment, worker and finite lease policy.
    pub fn from_store(
        store: CanonicalStore,
        worker: impl Into<String>,
        policy: LeasePolicy,
        pool: Arc<dyn MemoryPool>,
    ) -> Self {
        Self {
            store,
            worker: Arc::from(worker.into()),
            policy,
            pool,
        }
    }
    fn admitted(&self) -> Result<Self, WorkflowError> {
        Ok(Self {
            store: self.store.activity_store()?,
            ..self.clone()
        })
    }
    /// Exact canonical deployment used by executions and connected readers.
    pub fn store(&self) -> &CanonicalStore {
        &self.store
    }
    pub(super) fn same_physical_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.worker, &other.worker) && Arc::ptr_eq(&self.pool, &other.pool)
    }
    pub(super) fn physical_owner_identity(&self) -> (usize, usize) {
        (
            Arc::as_ptr(&self.worker) as *const () as usize,
            Arc::as_ptr(&self.pool) as *const () as usize,
        )
    }
    pub(super) fn physical_worker_owner(&self) -> Arc<str> {
        self.worker.clone()
    }
    /// Stable native worker identity for this runtime owner.
    pub fn worker(&self) -> &str {
        &self.worker
    }
    /// Finite authority and progress policy selected for this owner.
    pub const fn policy(&self) -> LeasePolicy {
        self.policy
    }
    /// Process-scoped default worker identity.
    pub fn process_worker(role: &str) -> String {
        let host = std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".into());
        format!("{role}:{host}:{}", std::process::id())
    }
    /// Persist cancellation under an explicit immutable effect identity.
    pub async fn cancel(&self, run: &str, operation: &str) -> Result<CanonicalRun, WorkflowError> {
        let activity = self.store().admit_activity()?;
        activity
            .scope(async { Ok(self.store.cancel_run(run, operation).await?) })
            .await
    }
    /// Bounded per-problem recorded execution summaries, excluding scientific payloads.
    pub async fn runs(
        &self,
        problem: &str,
        after: Option<u64>,
        limit: usize,
    ) -> Result<
        pse_columnar::Leased<Vec<pse_operations::canonical_execution::RunSummary>>,
        WorkflowError,
    > {
        let activity = self.store().admit_activity()?;
        activity
            .scope(async {
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:run-page-response",
                    pse_operations::canonical::MetadataResponse::Page,
                )?;
                let rows = pse_operations::canonical::response_ownership_scope(
                    response.clone(),
                    self.store.execution_run_page(problem, after, limit),
                )
                .await?;
                let bytes = rows.iter().try_fold(
                    rows.capacity()
                        .checked_mul(size_of::<pse_operations::canonical_execution::RunSummary>())
                        .and_then(|bytes| {
                            bytes.checked_add(
                                size_of::<
                                    pse_columnar::Leased<
                                        Vec<pse_operations::canonical_execution::RunSummary>,
                                    >,
                                >() + 2 * size_of::<usize>(),
                            )
                        })
                        .ok_or_else(|| contract("run summary retained extent"))?,
                    |bytes, row| {
                        [
                            Some(&row.key),
                            Some(&row.problem),
                            Some(&row.revision),
                            Some(&row.interpretation),
                            row.current_attempt.as_ref(),
                            row.terminal_attempt.as_ref(),
                            row.terminal_class.as_ref(),
                        ]
                        .into_iter()
                        .flatten()
                        .try_fold(bytes, |bytes, value| bytes.checked_add(value.capacity()))
                        .ok_or_else(|| contract("run summary retained extent"))
                    },
                )?;
                let retained = Arc::new(retain_metadata(&self.pool, response, bytes)?);
                Ok(pse_columnar::Leased::new(Arc::new(rows), retained))
            })
            .await
    }
    /// Reopen the exact historical admitted completion and eligible seed inventory.
    pub async fn record(&self, run: &str, attempt: &str) -> Result<DurableRecord, WorkflowError> {
        let activity = self.store().admit_activity()?;
        activity
            .scope(async {
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:record-selection-response",
                    pse_operations::canonical::MetadataResponse::Selection,
                )?;
                let read = Arc::new(
                    pse_operations::canonical::response_ownership_scope(
                        response.clone(),
                        self.store.read_results(run, attempt),
                    )
                    .await?,
                );
                let read_owner = Arc::new(retain_protected_metadata(
                    &self.pool,
                    response,
                    read.retained_bytes()?,
                    |owner| read.retain_metadata_owner(owner),
                )?);
                read.retain_metadata_owner(read_owner.clone())?;
                let completion = pse_operations::canonical::response_ownership_scope(
                    Arc::new((read.clone(), read_owner.clone())),
                    self.decode_completion(CompletionSelection::Full(&read)),
                )
                .await?;
                let mut solutions = Vec::new();
                let mut after = None;
                loop {
                    let response = pooled_response_owner(
                        &self.pool,
                        "canonical:record-seed-page-response",
                        pse_operations::canonical::MetadataResponse::Page,
                    )?;
                    let page = pse_operations::canonical::response_ownership_scope(
                        Arc::new((read.clone(), read_owner.clone(), response)),
                        self.store.result_seed_page(attempt, after),
                    )
                    .await?;
                    if page.is_empty() {
                        break;
                    }
                    for seed in page {
                        after = Some(seed.step);
                        if seed.run != run || seed.attempt != attempt {
                            return Err(contract("seed inventory escaped its retained attempt"));
                        }
                        if completion
                            .completion
                            .as_ref()
                            .and_then(|c| c.assessments.get(seed.step as usize))
                            .is_some_and(|a| a.permits_seed)
                        {
                            let solution = SolutionId::from_id(
                                SemanticId::parse_hex(&seed.key)
                                    .map_err(|error| contract(error.to_string()))?,
                            );
                            solutions.push((
                                usize::try_from(seed.step)
                                    .map_err(|_| contract("stored step extent"))?,
                                solution,
                            ));
                        }
                    }
                }
                let metadata_owner = durable_metadata_owner(
                    &self.pool,
                    Some(read.run()),
                    Some(read.attempt()),
                    Some(read.manifest()),
                    read.attempt().key.len(),
                    solutions
                        .capacity()
                        .checked_mul(size_of::<(usize, SolutionId)>())
                        .ok_or_else(|| contract("durable solutions extent"))?,
                )?;
                Ok(DurableRecord {
                    attempt_id: completion.attempt_id,
                    attempt_key: Some(read.attempt().key.clone()),
                    run: Some(read.run().clone()),
                    attempt: Ok(read.attempt().clone()),
                    manifest: Some(read.manifest().clone()),
                    completion: Some(completion),
                    solutions,
                    _solutions_owner: None,
                    _metadata_owner: Some(metadata_owner),
                })
            })
            .await
    }
    /// Reopen only the original completion under fresh exact terminal permission.
    pub async fn completion(
        &self,
        run: &str,
        attempt: &str,
    ) -> Result<Arc<pse_columnar::Leased<StoredCompletion>>, WorkflowError> {
        let activity = self.store().admit_activity()?;
        activity
            .scope(async {
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:completion-selection-response",
                    pse_operations::canonical::MetadataResponse::Selection,
                )?;
                let read = pse_operations::canonical::response_ownership_scope(
                    response.clone(),
                    self.store.read_completion(run, attempt),
                )
                .await?;
                let read_owner = Arc::new(retain_protected_metadata(
                    &self.pool,
                    response,
                    read.retained_bytes()?,
                    |owner| read.retain_metadata_owner(owner),
                )?);
                read.retain_metadata_owner(read_owner)?;
                self.decode_completion(CompletionSelection::Slim(&read))
                    .await
            })
            .await
    }

    pub(super) async fn progress_coverage(
        &self,
        read: &ResultRead,
    ) -> Result<super::ProgressCoverage, WorkflowError> {
        Ok(self
            .decode_completion(CompletionSelection::Full(read))
            .await?
            .progress
            .clone())
    }

    async fn decode_completion(
        &self,
        read: CompletionSelection<'_>,
    ) -> Result<Arc<pse_columnar::Leased<StoredCompletion>>, WorkflowError> {
        let header: CompletionReceipt = serde_json::from_slice(
            read.attempt()
                .completion
                .as_ref()
                .ok_or_else(|| contract("terminal completion absent"))?
                .as_slice(),
        )
        .map_err(|e| contract(e.to_string()))?;
        if header.version != 1 {
            return Err(contract("stored completion receipt interpretation"));
        }
        let (mut completion, owner) = match header.inline {
            Some(completion) => {
                if header.batch_count != 0 || header.payload_bytes != 0 || !header.digest.is_empty()
                {
                    return Err(contract("inline completion carries chunk coverage"));
                }
                let bytes = serde_json::to_vec(&completion).map_err(|e| contract(e.to_string()))?;
                (*completion, reserve(&self.pool, bytes.len())?)
            }
            None => {
                let (bytes, owner) = read_chunks_selection(
                    &self.store,
                    &self.pool,
                    read,
                    "__completion",
                    0,
                    header.batch_count,
                    header.payload_bytes,
                    &header.digest,
                    false,
                )
                .await?;
                (
                    serde_json::from_slice(&bytes).map_err(|e| contract(e.to_string()))?,
                    owner,
                )
            }
        };
        if !matches!(completion.version, 1 | 2) {
            return Err(contract("stored completion interpretation"));
        }
        if completion.version == 1 {
            completion.progress = super::ProgressCoverage::Unknown {};
        }
        Ok(Arc::new(pse_columnar::Leased::new(
            Arc::new(completion),
            owner,
        )))
    }

    /// Admit this exact closed selection and reopen the acknowledged immutable receipt.
    #[allow(
        unsafe_code,
        reason = "scientific settlement supplies the prepared original completion or explicit unavailable science"
    )]
    async fn settle(
        &self,
        closed: &ClosedAttempt,
        operation: &str,
        class: TerminalClass,
        receipt: &CompletionReceipt,
        seeds: SeedEligibility,
        heartbeat: Option<Heartbeat>,
    ) -> Result<DurableRecord, WorkflowError> {
        let prepared = async {
            let completion = serde_json::to_vec(receipt).map_err(|e| contract(e.to_string()))?;
            let manifest = self
                .store
                .reconcile_closed_attempt(closed)
                .await
                .map_err(|error| {
                    super::diagnostics::operation_context(error.into(), "durable.settle.reconcile")
                })?;
            Ok::<_, WorkflowError>((manifest, completion))
        }
        .await;
        // Healthy renewal remains valid after ingestion closes. Keep it through
        // every reconciliation page, then join and inspect it before terminal admission.
        let heartbeat = match heartbeat {
            Some(heartbeat) => heartbeat.stop().await,
            None => None,
        };
        let (manifest, completion) = match prepared {
            Ok(prepared) => {
                if let Some(error) = heartbeat {
                    return Err(WorkflowError::Shared(error));
                }
                prepared
            }
            Err(error) => {
                let error = Arc::new(error);
                return Err(WorkflowError::Shared(
                    export_failure(Some(error.clone()), heartbeat, Some(closed.fence()))
                        .unwrap_or(error),
                ));
            }
        };
        // SAFETY: normal execution supplies its joined assessment, recovery supplies
        // no scientific completion. Both refer to this exact frozen manifest.
        unsafe {
            self.store
                .seal_attempt(&manifest, operation, class, &completion, seeds)
                .await
        }
        .map_err(|error| {
            super::diagnostics::operation_context(error.into(), "durable.settle.seal")
        })?;
        self.record(closed.fence().run(), closed.fence().attempt())
            .await
            .map_err(|error| super::diagnostics::operation_context(error, "durable.settle.record"))
    }
    /// Recover only explicitly selected expired/cancelled run authority. No numerical dispatch.
    #[allow(
        unsafe_code,
        reason = "current recovery authority records truthful worker loss; never promotes arbitrary rows to scientific success"
    )]
    pub async fn recover(&self, run: &str, operation: &str) -> Result<Recovery, WorkflowError> {
        self.recover_with_study_point(run, operation, None).await
    }

    /// Admit occurrence-specific absence only from its refreshed post-fence dispatch facts.
    pub(super) async fn recover_study_point(
        &self,
        run: &str,
        operation: &str,
        point: &str,
    ) -> Result<Recovery, WorkflowError> {
        self.recover_with_study_point(run, operation, Some(point))
            .await
    }

    async fn recover_with_study_point(
        &self,
        run: &str,
        operation: &str,
        point_key: Option<&str>,
    ) -> Result<Recovery, WorkflowError> {
        let activity = self.store().admit_activity()?;
        activity
            .scope(async {
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:recovery-header-response",
                    pse_operations::canonical::MetadataResponse::Header,
                )?;
                let current = pse_operations::canonical::response_ownership_scope(
                    response,
                    self.store.canonical_run(run),
                )
                .await?
                .ok_or_else(|| contract("recovery run absent"))?;
                let attempt = current
                    .current_attempt
                    .as_ref()
                    .ok_or_else(|| contract("recovery attempt absent"))?;
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:recovery-header-response",
                    pse_operations::canonical::MetadataResponse::Header,
                )?;
                let actual = pse_operations::canonical::response_ownership_scope(
                    response,
                    self.store.canonical_attempt(attempt),
                )
                .await?
                .ok_or_else(|| contract("recovery attempt header absent"))?;
                if actual.terminal() {
                    return Ok(Recovery {
                        recovered: vec![actual.key],
                    });
                }
                let closed = self.store.recover_closed_attempt(run, operation).await?;
                if closed.fence().run() != run
                    || closed.fence().attempt() != actual.key.as_str()
                    || closed.fence().generation() != actual.generation
                {
                    return Err(contract(
                        "recovered writer differs from the observed exact attempt",
                    ));
                }
                let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
                hash.str("scientific.attempt.lineage.v1")
                    .str(closed.fence().attempt());
                let attempt_id = AttemptId::from_id(hash.finish_id());
                let error = WorkflowError::Canonical(
                    pse_operations::canonical::CanonicalError::WorkerLost {
                        run: run.to_owned(),
                        attempt: actual.key.clone(),
                        generation: actual.generation,
                    },
                );
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:recovery-header-response",
                    pse_operations::canonical::MetadataResponse::Header,
                )?;
                let current = pse_operations::canonical::response_ownership_scope(
                    response,
                    self.store.canonical_run(run),
                )
                .await?
                .ok_or_else(|| contract("recovered run absent"))?;
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:recovery-study-response",
                    pse_operations::canonical::MetadataResponse::Selection,
                )?;
                let summary = pse_operations::canonical::response_ownership_scope(
                    response,
                    self.store.canonical_study_for_run(run),
                )
                .await?;
                let cancelled =
                    current.cancelled || summary.as_ref().is_some_and(|study| study.cancelled);
                let mut outcome = failure(&error, cancelled);
                if let Some(owner) = actual.finalization_study.as_deref() {
                    if !summary
                        .as_ref()
                        .is_some_and(|study| study.key == owner && study.run == run)
                    {
                        return Err(contract("recovery summary authority association differs"));
                    }
                    // This immutable capability admits only summary publication,
                    // never a scientific dispatch. Its recovery fence is now closed.
                    outcome.detail.effect = pse_model::study::EffectState::Absent;
                } else if let Some(point_key) = point_key {
                    let response = pooled_response_owner(
                        &self.pool,
                        "canonical:recovery-point-response",
                        pse_operations::canonical::MetadataResponse::Selection,
                    )?;
                    let (point, frozen, _study, owner) =
                        pse_operations::canonical::response_ownership_scope(
                            response,
                            self.store.recovery_snapshot(point_key),
                        )
                        .await?;
                    if point.run != run
                        || point.attempt.as_deref() != Some(actual.key.as_str())
                        || frozen.key != actual.key
                        || frozen.run != run
                        || frozen.generation != actual.generation
                        || frozen.ingestion_open()
                        || owner.current_attempt.as_deref() != Some(actual.key.as_str())
                        || frozen.close_generation != Some(owner.current_generation)
                        || owner.current_generation <= actual.generation
                    {
                        return Err(contract(
                            "recovery point does not name the exact fenced attempt",
                        ));
                    }
                    // Refresh after freezing the writer: a native-start transaction
                    // racing before the fence must be visible here; one racing after
                    // it cannot commit under the old execution-run generation.
                    if !pse_operations::canonical_studies::point_facts(&point)?.native_started {
                        outcome.detail.effect = pse_model::study::EffectState::Absent;
                    }
                }
                outcome.retryable = true;
                outcome.detail.retry_failure = Some(pse_model::study::RetryFailure::Transient);
                let stored = StoredCompletion {
                    version: 2,
                    attempt_id,
                    completion: None,
                    termination: outcome.detail,
                    state: outcome.state,
                    progress: super::ProgressCoverage::Unknown {},
                };
                let receipt = CompletionReceipt {
                    version: 1,
                    batch_count: 0,
                    payload_bytes: 0,
                    digest: String::new(),
                    inline: Some(Box::new(stored)),
                };
                let class = if cancelled {
                    TerminalClass::Cancelled
                } else {
                    TerminalClass::Failed
                };
                self.settle(
                    &closed,
                    &format!("{operation}:terminal"),
                    class,
                    &receipt,
                    SeedEligibility::Unavailable,
                    None,
                )
                .await?;
                Ok(Recovery {
                    recovered: vec![closed.fence().attempt().into()],
                })
            })
            .await
    }
}
/// Whether application execution persists its complete scientific result.
#[derive(Clone, Debug, Default)]
pub enum Durability {
    /// Explicit local numerical adapter without persisted executions.
    #[default]
    Ephemeral,
    /// Application execution uses native canonical claims and immutable results.
    Durable(Box<Operations>),
}
/// Persistence receipt of one joined execution.
#[derive(Clone, Debug, Default)]
pub enum RunDurability {
    /// Explicit local numerical execution.
    #[default]
    Ephemeral,
    /// Exact retained execution, including observable write failure.
    Durable(Arc<DurableRecord>),
}
/// A completed canonical attempt and its exact immutable selection, or an observable write failure.
#[derive(Debug)]
pub struct DurableRecord {
    /// Scientific lineage identity, distinct from the opaque native key.
    pub attempt_id: AttemptId,
    /// Exact native key, when registration established the attempt.
    pub attempt_key: Option<String>,
    /// Actual acknowledged run header; absent before successful run registration.
    pub run: Option<CanonicalRun>,
    /// Actual terminal receipt or the original persistence failure.
    pub attempt: Result<CanonicalAttempt, Arc<WorkflowError>>,
    /// Exact reconciled admitted selection, when settlement succeeded.
    pub manifest: Option<ResultManifest>,
    /// Accounted original scientific completion and truthful lifecycle facts.
    pub completion: Option<Arc<pse_columnar::Leased<StoredCompletion>>>,
    /// Eligible final seeds by their scientific step and artifact identity.
    pub solutions: Vec<(usize, SolutionId)>,
    _solutions_owner: Option<Arc<pse_columnar::AllocationLease>>,
    _metadata_owner: Option<Arc<pse_columnar::MemoryReservation>>,
}
// One extent owner for the continuing native lifecycle metadata. Completion and
// already-leased seed identities retain their independent scientific owners.
fn durable_metadata_owner(
    pool: &Arc<dyn MemoryPool>,
    run: Option<&CanonicalRun>,
    attempt: Option<&CanonicalAttempt>,
    manifest: Option<&ResultManifest>,
    attempt_key_bytes: usize,
    solutions_bytes: usize,
) -> Result<Arc<pse_columnar::MemoryReservation>, WorkflowError> {
    use pse_model::HeapUsage;
    let bytes = [
        size_of::<DurableRecord>(),
        2 * size_of::<usize>(),
        attempt_key_bytes,
        solutions_bytes,
        run.map_or(0, HeapUsage::heap_bytes),
        attempt.map_or(0, HeapUsage::heap_bytes),
        manifest.map_or(0, HeapUsage::heap_bytes),
    ]
    .into_iter()
    .try_fold(0usize, usize::checked_add)
    .ok_or_else(|| contract("durable metadata extent"))?;
    let owner = MemoryConsumer::new("canonical:durable-record-metadata").register(pool);
    owner
        .try_grow(bytes)
        .map_err(pse_engine::EngineError::from)?;
    Ok(Arc::new(owner))
}

/// Typed scientific completion; data lives in bounded immutable chunks.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredCompletion {
    /// Portable completion interpretation.
    pub version: u8,
    /// Original scientific attempt lineage.
    pub attempt_id: AttemptId,
    /// Original scientific assessments; worker loss has no invented conclusion.
    pub completion: Option<super::Completion>,
    /// Original terminal cause and effect knowledge.
    pub termination: TerminationDetail,
    /// Truthful execution state, distinct from individual candidate permissions.
    pub state: AttemptState,
    /// Original sampled telemetry coverage; absence never means complete coverage.
    #[serde(default)]
    pub progress: super::ProgressCoverage,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompletionReceipt {
    version: u8,
    batch_count: u64,
    payload_bytes: u64,
    digest: String,
    inline: Option<Box<StoredCompletion>>,
}
pub(crate) type Canceller = Arc<dyn Fn() + Send + Sync>;
#[derive(Clone, Debug)]
pub(super) struct Outcome {
    pub(super) state: AttemptState,
    pub(super) detail: TerminationDetail,
    pub(super) retryable: bool,
}
// Task-local export control: absent from production and never shared across attempts.
#[cfg(all(test, feature = "canonical-tests"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FinishExportPhase {
    Progress,
    Table,
    Seed,
    Completion,
    AcknowledgedSeal,
}
#[cfg(all(test, feature = "canonical-tests"))]
#[derive(Debug)]
struct FinishExportGate {
    phase: FinishExportPhase,
    reached: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
    release: Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
    failure: Option<Arc<WorkflowError>>,
}
#[cfg(all(test, feature = "canonical-tests"))]
impl FinishExportGate {
    async fn at(gate: &OnceLock<Self>, phase: FinishExportPhase) -> Result<(), WorkflowError> {
        let Some(gate) = gate.get().filter(|gate| gate.phase == phase) else {
            return Ok(());
        };
        let reached = gate
            .reached
            .lock()
            .map_err(|_| contract("export control lock"))?
            .take();
        let Some(reached) = reached else {
            return Ok(());
        };
        let release = gate
            .release
            .lock()
            .map_err(|_| contract("export control lock"))?
            .take()
            .ok_or_else(|| contract("export control release absent"))?;
        let _ = reached.send(());
        release
            .await
            .map_err(|_| contract("export control release dropped"))?;
        match &gate.failure {
            Some(error) => Err(WorkflowError::Shared(error.clone())),
            None => Ok(()),
        }
    }
}
#[cfg(all(test, feature = "canonical-tests"))]
tokio::task_local! {
    static WORKER_EXPORT_CONTROL: Arc<Mutex<Option<FinishExportGate>>>;
}
#[cfg(all(test, feature = "canonical-tests"))]
pub(super) async fn gated_worker_export<T>(
    reached: tokio::sync::oneshot::Sender<()>,
    release: tokio::sync::oneshot::Receiver<()>,
    failure: bool,
    work: impl Future<Output = T>,
) -> T {
    let gate = FinishExportGate {
        phase: if failure {
            FinishExportPhase::Completion
        } else {
            FinishExportPhase::Table
        },
        reached: Mutex::new(Some(reached)),
        release: Mutex::new(Some(release)),
        failure: failure.then(|| {
            Arc::new(WorkflowError::Canonical(
                pse_operations::canonical::CanonicalError::Timeout,
            ))
        }),
    };
    WORKER_EXPORT_CONTROL
        .scope(Arc::new(Mutex::new(Some(gate))), work)
        .await
}
fn uncertain_progress_queue(progress: &super::ProgressCoverage) -> Option<u64> {
    use super::{
        ProgressCoverage, ProgressWriteStage, ProgressWriterFailure, ProgressWriterOutcome,
    };
    match progress {
        ProgressCoverage::Sampled {
            queued,
            unacknowledged,
            writer,
            ..
        } if *unacknowledged > 0
            && matches!(
                writer,
                ProgressWriterOutcome::JoinFailed
                    | ProgressWriterOutcome::Failed(ProgressWriterFailure {
                        stage: ProgressWriteStage::CanonicalAppend {}
                            | ProgressWriteStage::Acknowledgment {}
                    })
            ) =>
        {
            Some(*queued)
        }
        _ => None,
    }
}
/// Attempt-owned precharged receipt storage. Only the asynchronous publication
/// owners enter this short mutex; native Tap callbacks only try their queue.
#[derive(Debug)]
pub(super) struct PublicationAcks {
    state: Mutex<Option<ResultAcknowledgments>>,
    frozen: AtomicBool,
    memory: Result<Arc<pse_columnar::AllocationLease>, Arc<WorkflowError>>,
}
impl PublicationAcks {
    pub(super) fn new(pool: &Arc<dyn MemoryPool>) -> Arc<Self> {
        let reservation =
            MemoryConsumer::new("canonical:publication-acknowledgments").register(pool);
        let memory = reservation
            .try_grow(RESULT_ACKNOWLEDGMENT_BYTES)
            .map(|()| pse_columnar::AllocationLease::new(reservation))
            .map_err(|error| Arc::new(WorkflowError::from(pse_engine::EngineError::from(error))));
        Arc::new(Self {
            state: Mutex::new(None),
            frozen: AtomicBool::new(false),
            memory,
        })
    }
    fn admitted(&self) -> Result<(), WorkflowError> {
        self.memory
            .as_ref()
            .map(|_| ())
            .map_err(|error| WorkflowError::Shared(error.clone()))
    }
    fn update<T>(
        &self,
        fence: &AttemptFence,
        action: impl FnOnce(
            &mut ResultAcknowledgments,
        ) -> Result<T, pse_operations::canonical::CanonicalError>,
    ) -> Result<T, WorkflowError> {
        self.admitted()?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| contract("publication acknowledgment lock poisoned"))?;
        if self.frozen.load(Ordering::Acquire) {
            return Err(contract("publication acknowledgment owner already frozen"));
        }
        let acknowledgments = state.get_or_insert_with(|| ResultAcknowledgments::new(fence));
        if acknowledgments.fence() != fence {
            return Err(contract("publication acknowledgment attempt changed"));
        }
        Ok(action(acknowledgments)?)
    }
    pub(super) fn expect(&self, fence: &AttemptFence, name: &str) -> Result<(), WorkflowError> {
        self.update(fence, |acks| acks.expect(name))
    }
    pub(super) fn append(&self, fence: &AttemptFence, ack: AppendAck) -> Result<(), WorkflowError> {
        self.update(fence, |acks| acks.append(ack))
    }
    pub(super) fn empty(
        &self,
        fence: &AttemptFence,
        ack: EmptySetAck,
    ) -> Result<(), WorkflowError> {
        self.update(fence, |acks| acks.empty(ack))
    }
    pub(super) fn complete(&self, fence: &AttemptFence, name: &str) -> Result<(), WorkflowError> {
        self.update(fence, |acks| acks.complete(name))
    }
    fn complete_present(&self, fence: &AttemptFence, name: &str) -> Result<(), WorkflowError> {
        self.update(fence, |acks| acks.complete_present(name))
    }
    fn seed(&self, fence: &AttemptFence, ack: SeedAck) -> Result<(), WorkflowError> {
        self.update(fence, |acks| acks.seed(ack))
    }
    fn rows(&self, fence: &AttemptFence, name: &str) -> Result<u64, WorkflowError> {
        self.update(fence, |acks| Ok(acks.rows(name)))
    }
    pub(super) fn freeze(
        &self,
        fence: &AttemptFence,
    ) -> Result<ResultAcknowledgments, WorkflowError> {
        self.admitted()?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| contract("publication acknowledgment lock poisoned"))?;
        if self.frozen.swap(true, Ordering::AcqRel) {
            return Err(contract("publication acknowledgment owner already frozen"));
        }
        Ok(state
            .take()
            .unwrap_or_else(|| ResultAcknowledgments::new(fence)))
    }
}
#[derive(Debug)]
pub(crate) struct DurableAttempt {
    worker_publication: Option<super::worker::PublicationHandoff>,
    live_prefix: super::result_projection::ResultStreamPosition,
    live_prefix_failed: bool,
    live_append_control: Option<Arc<pse_columnar::AllocationLease>>,
    horizon_prefix: Option<
        std::collections::BTreeMap<SemanticId, super::result_projection::ResultStreamPosition>,
    >,
    #[cfg(any(
        feature = "solver-diffsol",
        all(test, feature = "canonical-tests", feature = "solver-kinsol")
    ))]
    horizon_seed_next: u64,
    horizon_prefix_failure: Option<Arc<WorkflowError>>,
    output: super::StudyOutputDemand,
    operations: Operations,
    attempt: AttemptId,
    run_id: Option<RunId>,
    claim_operation: String,
    fence: Arc<OnceLock<AttemptFence>>,
    run: OnceLock<CanonicalRun>,
    metadata_owner: Option<Arc<pse_columnar::MemoryReservation>>,
    stream: Streamer,
    acknowledgments: Arc<PublicationAcks>,
    solutions: Vec<(usize, SolutionId)>,
    solutions_owner: Option<Arc<pse_columnar::AllocationLease>>,
    completion_owner: Option<Arc<pse_columnar::AllocationLease>>,
    heartbeat: Option<Heartbeat>,
}
impl DurableAttempt {
    pub(super) fn set_worker_publication(
        &mut self,
        handoff: Option<super::worker::PublicationHandoff>,
    ) {
        self.worker_publication = handoff;
    }
    pub(super) async fn begin_worker_publication(&mut self) -> Result<bool, WorkflowError> {
        match self.worker_publication.take() {
            Some(handoff) => handoff.ready().await,
            None => Ok(true),
        }
    }
    pub(super) fn new(operations: &Operations) -> Result<Self, WorkflowError> {
        let operations = operations.admitted()?;
        let metadata_owner = durable_metadata_owner(
            &operations.pool,
            None,
            None,
            None,
            pse_schema::catalog::substrate::METADATA_ID_BYTES,
            0,
        )?;
        let attempt: AttemptId = pse_operations::mint_id();
        let fence = Arc::new(OnceLock::new());
        let acknowledgments = PublicationAcks::new(&operations.pool);
        Ok(Self {
            worker_publication: None,
            live_prefix: Default::default(),
            live_prefix_failed: false,
            live_append_control: None,
            horizon_prefix: None,
            #[cfg(any(
                feature = "solver-diffsol",
                all(test, feature = "canonical-tests", feature = "solver-kinsol")
            ))]
            horizon_seed_next: 0,
            horizon_prefix_failure: None,
            output: super::StudyOutputDemand::Full {},
            operations: operations.clone(),
            attempt,
            run_id: None,
            claim_operation: format!("claim:{attempt}"),
            stream: Streamer::new(&operations, fence.clone(), acknowledgments.clone()),
            acknowledgments,
            solutions: Vec::new(),
            solutions_owner: None,
            completion_owner: None,
            fence,
            run: OnceLock::new(),
            metadata_owner: Some(metadata_owner),
            heartbeat: None,
        })
    }
    pub(crate) fn claimed(
        operations: &Operations,
        fence: AttemptFence,
        run_id: RunId,
        attempt: AttemptId,
    ) -> Result<Self, WorkflowError> {
        let operations = operations.admitted()?;
        let metadata_owner = durable_metadata_owner(
            &operations.pool,
            None,
            None,
            None,
            pse_schema::catalog::substrate::METADATA_ID_BYTES,
            0,
        )?;
        let shared = Arc::new(OnceLock::new());
        let _ = shared.set(fence);
        let acknowledgments = PublicationAcks::new(&operations.pool);
        let stream = Streamer::new(&operations, shared.clone(), acknowledgments.clone());
        #[cfg(all(test, feature = "canonical-tests"))]
        if let Ok(Some(gate)) = WORKER_EXPORT_CONTROL
            .try_with(|control| control.lock().ok().and_then(|mut gate| gate.take()))
        {
            let _ = stream.export_gate.set(gate);
        }
        Ok(Self {
            worker_publication: None,
            live_prefix: Default::default(),
            live_prefix_failed: false,
            live_append_control: None,
            horizon_prefix: None,
            #[cfg(any(
                feature = "solver-diffsol",
                all(test, feature = "canonical-tests", feature = "solver-kinsol")
            ))]
            horizon_seed_next: 0,
            horizon_prefix_failure: None,
            output: super::StudyOutputDemand::Full {},
            operations: operations.clone(),
            attempt,
            run_id: Some(run_id),
            claim_operation: format!("claimed:{attempt}"),
            stream,
            acknowledgments,
            solutions: Vec::new(),
            solutions_owner: None,
            completion_owner: None,
            fence: shared,
            run: OnceLock::new(),
            metadata_owner: Some(metadata_owner),
            heartbeat: None,
        })
    }
    fn retain_run(&self, row: CanonicalRun) -> Result<(), WorkflowError> {
        use pse_model::HeapUsage;
        if self.run.get().is_some() {
            return Ok(());
        }
        let bytes = row.heap_bytes();
        let owner = self
            .metadata_owner
            .as_ref()
            .ok_or_else(|| contract("durable metadata owner absent"))?;
        owner
            .try_grow(bytes)
            .map_err(pse_engine::EngineError::from)?;
        if self.run.set(row).is_err() {
            owner.shrink(bytes);
        }
        Ok(())
    }
    pub(super) const fn attempt_id(&self) -> AttemptId {
        self.attempt
    }
    pub(super) fn claimed_run(&self) -> Option<RunId> {
        self.run_id
    }
    pub(super) fn canonical_keys(&self, run_id: RunId) -> (String, String) {
        match self.fence.get() {
            Some(f) => (f.run().into(), f.attempt().into()),
            None => {
                let run = format!("run:{run_id}");
                let attempt = execution_attempt_key(&run, &self.claim_operation);
                (run, attempt)
            }
        }
    }
    pub(super) fn tap(&self) -> Arc<dyn ProgressTap> {
        self.stream.tap.clone()
    }
    pub(super) fn set_step(&self, step: usize) {
        self.stream
            .tap
            .step
            .store(i32::try_from(step).unwrap_or(i32::MAX), Ordering::Release);
    }
    pub(super) async fn register(
        &self,
        run_id: RunId,
        request: &RunRequest,
    ) -> Result<(), WorkflowError> {
        if let Some(fence) = self.fence.get() {
            let row = self
                .operations
                .store
                .canonical_run(fence.run())
                .await?
                .ok_or_else(|| contract("claimed run absent"))?;
            self.retain_run(row)?;
            self.stream.ready.notify_one();
            return Ok(());
        }
        let inputs = request_inputs(request)?;
        let (runtime, physical) = request_context(request)?;
        let identity = identities(request)?;
        let selected = serde_json::to_vec(&(
            1_u8,
            identity,
            physical.identity(),
            request_provenance(request)?,
        ))
        .map_err(|e| contract(e.to_string()))?;
        self.register_selection(
            run_id,
            runtime,
            physical,
            &inputs,
            serde_json::to_vec(&(1_u8, run_id, identity)).map_err(|e| contract(e.to_string()))?,
            selected,
        )
        .await
    }
    #[cfg(feature = "solver-diffsol")]
    pub(super) async fn register_horizon(
        &self,
        run_id: RunId,
        runtime: &super::Runtime,
        packages: &[super::ModelingPackage],
        identity: ContentHash,
        provenance: &[u8],
    ) -> Result<(), WorkflowError> {
        if self.fence.get().is_some() {
            let row = self
                .operations
                .store
                .canonical_run(self.fence()?.run())
                .await?
                .ok_or_else(|| contract("claimed horizon run absent"))?;
            self.retain_run(row)?;
            self.stream.ready.notify_one();
            return Ok(());
        }
        let physical = &packages
            .first()
            .ok_or_else(|| contract("horizon source absent"))?
            .physical;
        let mut inputs = packages
            .iter()
            .map(|p| p.recorded_revision().cloned())
            .collect::<Result<Vec<_>, _>>()?;
        inputs.sort_by(|a, b| a.key.cmp(&b.key));
        inputs.dedup_by(|a, b| a.key == b.key);
        self.register_selection(
            run_id,
            runtime,
            physical,
            &inputs,
            serde_json::to_vec(&(1_u8, "horizon", run_id, identity))
                .map_err(|e| contract(e.to_string()))?,
            serde_json::to_vec(&(
                1_u8,
                identity,
                physical.identity(),
                serde_json::from_slice::<serde_json::Value>(provenance)
                    .map_err(|error| contract(error.to_string()))?,
            ))
            .map_err(|e| contract(e.to_string()))?,
        )
        .await
    }
    async fn register_selection(
        &self,
        run_id: RunId,
        runtime: &super::Runtime,
        physical: &super::PhysicalContext,
        inputs: &[Revision],
        request: Vec<u8>,
        selected: Vec<u8>,
    ) -> Result<(), WorkflowError> {
        let primary = inputs
            .first()
            .ok_or_else(|| contract("durable execution needs an exact scientific revision"))?;
        let physical_rows = self.operations.put_physical_rows(runtime, physical).await?;
        let mut sources = inputs.iter().skip(1).cloned().collect::<Vec<_>>();
        sources.extend(physical_rows.revisions.iter().cloned());
        let (run, _) = self.canonical_keys(run_id);
        let attestation = runtime.canonical()?.attestation();
        let row = self
            .operations
            .store
            .begin_run(&pse_operations::canonical_execution::RunRequest {
                key: run.clone(),
                revision: primary.clone(),
                sources,
                request,
                source_selection: selected,
                attestation: serde_json::to_vec(&(attestation.source, attestation.build))
                    .map_err(|e| contract(e.to_string()))?,
            })
            .await?;
        drop(physical_rows);
        self.retain_run(row)?;
        let fence = self
            .operations
            .store
            .claim_run(
                &run,
                &self.claim_operation,
                self.operations.worker(),
                self.operations.policy.lease,
            )
            .await?;
        self.fence
            .set(fence)
            .map_err(|_| contract("native fence already recorded"))?;
        self.stream.ready.notify_one();
        Ok(())
    }
    pub(crate) fn set_output(&mut self, output: super::StudyOutputDemand) {
        self.output = output;
    }
    pub(super) fn begin_live_samples(&self) -> Result<(), WorkflowError> {
        self.acknowledgments.expect(
            self.fence()?,
            &pse_relations::generated::runtime::live_simulation_samples::RELATION_ID.to_string(),
        )
    }
    pub(super) async fn publish_live_samples(
        &mut self,
        table: &pse_relations::columnar::FieldCheckedBatch,
    ) -> Result<u64, WorkflowError> {
        if self.live_prefix_failed {
            return Err(contract(
                "live sample recording is incomplete after an earlier refusal",
            ));
        }
        let fence = self.fence()?.clone();
        let result = super::result_projection::append_live_sample_batch(
            &self.operations.store,
            &fence,
            &self.operations.pool,
            &self.acknowledgments,
            &mut self.live_prefix,
            &mut self.live_append_control,
            table,
        )
        .await;
        if let Err(error) = result {
            self.live_prefix_failed = true;
            return Err(error);
        }
        Ok(self.live_prefix.origin)
    }
    pub(super) fn fail_live_samples(&mut self) {
        self.live_prefix_failed = true;
        self.live_append_control = None;
    }
    pub(super) async fn close_live_samples(&mut self) -> Result<(), WorkflowError> {
        self.live_append_control = None;
        if self.live_prefix_failed {
            return Err(contract("live sample recording remains incomplete"));
        }
        let fence = self.fence()?.clone();
        super::result_projection::finish_result_table_prefix(
            &self.operations.store,
            &fence,
            pse_relations::generated::runtime::live_simulation_samples::RELATION_ID,
            &self.acknowledgments,
            self.live_prefix,
        )
        .await
    }
    #[cfg(any(
        feature = "solver-diffsol",
        all(test, feature = "canonical-tests", feature = "solver-kinsol")
    ))]
    pub(super) fn begin_horizon_prefix(
        &mut self,
        output: super::StudyOutputDemand,
        maximum_solves: usize,
    ) -> Result<(), WorkflowError> {
        if self.horizon_prefix.is_some() {
            return Err(contract("horizon prefix owner already established"));
        }
        let owner = reserve(
            &self.operations.pool,
            maximum_solves
                .checked_mul(size_of::<(usize, SolutionId)>())
                .ok_or_else(|| contract("horizon seed identity extent"))?,
        )?;
        self.output = output;
        self.solutions_owner = Some(owner);
        self.horizon_prefix = Some(Default::default());
        Ok(())
    }
    #[cfg(any(
        feature = "solver-diffsol",
        all(test, feature = "canonical-tests", feature = "solver-kinsol")
    ))]
    pub(super) fn refuse_horizon_prefix(&mut self, error: Arc<WorkflowError>) {
        if self.horizon_prefix_failure.is_none() {
            self.horizon_prefix_failure = Some(error);
        }
    }
    #[cfg(any(
        feature = "solver-diffsol",
        all(test, feature = "canonical-tests", feature = "solver-kinsol")
    ))]
    pub(super) async fn publish_horizon_step(
        &mut self,
        result: &RunResult,
    ) -> Result<(), WorkflowError> {
        let fence = self.fence()?.clone();
        let full = matches!(self.output, super::StudyOutputDemand::Full {});
        let relations = if full {
            result.table_ids()
        } else {
            result.summary_table_ids()
        };
        let positions = self
            .horizon_prefix
            .as_mut()
            .ok_or_else(|| contract("horizon prefix owner absent"))?;
        for id in relations {
            if id == pse_relations::generated::runtime::modeling_findings::RELATION_ID {
                continue;
            }
            let position = positions.entry(id).or_default();
            let cursor = result
                .cursor_by_id(id, 1024, super::ResultOrder::Canonical)
                .map_err(WorkflowError::Shared)?;
            super::result_projection::append_result_table_prefix(
                &self.operations.store,
                &fence,
                id,
                cursor,
                &self.operations.pool,
                &self.acknowledgments,
                position,
                None,
            )
            .await?;
        }
        let (solutions, _owner) = append_seeds(
            &self.operations,
            &fence,
            result,
            &self.acknowledgments,
            &mut self.horizon_seed_next,
            #[cfg(all(test, feature = "canonical-tests"))]
            &self.stream.export_gate,
        )
        .await?;
        self.solutions.extend(solutions);
        Ok(())
    }
    async fn finish_horizon_prefix(
        &mut self,
        result: &RunResult,
    ) -> Result<(Vec<(usize, SolutionId)>, Arc<pse_columnar::AllocationLease>), WorkflowError> {
        if let Some(error) = &self.horizon_prefix_failure {
            return Err(WorkflowError::Shared(error.clone()));
        }
        let fence = self.fence()?.clone();
        let mut positions = self
            .horizon_prefix
            .take()
            .ok_or_else(|| contract("horizon prefix owner absent"))?;
        let findings = pse_relations::generated::runtime::modeling_findings::RELATION_ID;
        let position = positions.entry(findings).or_default();
        let cursor = result
            .cursor_by_id(findings, 1024, super::ResultOrder::Canonical)
            .map_err(WorkflowError::Shared)?;
        super::result_projection::append_result_table_prefix(
            &self.operations.store,
            &fence,
            findings,
            cursor,
            &self.operations.pool,
            &self.acknowledgments,
            position,
            None,
        )
        .await?;
        // A zero-solve/refused horizon still closes each advertised mandatory
        // relation coherently. Relations already appended are never reopened.
        let advertised = if matches!(self.output, super::StudyOutputDemand::Full {}) {
            result.table_ids()
        } else {
            result.summary_table_ids()
        };
        for id in advertised {
            if positions.contains_key(&id) {
                continue;
            }
            let cursor = result
                .cursor_by_id(id, 1024, super::ResultOrder::Canonical)
                .map_err(WorkflowError::Shared)?;
            let position = positions.entry(id).or_default();
            super::result_projection::append_result_table_prefix(
                &self.operations.store,
                &fence,
                id,
                cursor,
                &self.operations.pool,
                &self.acknowledgments,
                position,
                None,
            )
            .await?;
        }
        if let Some(report) = &result.horizon_output {
            let detail = report
                .detail
                .as_ref()
                .map_err(|error| WorkflowError::Shared(error.clone()))?;
            for (id, batches) in detail {
                // Mandatory summaries were already streamed in original step order.
                if positions.contains_key(id) {
                    continue;
                }
                let spec = result
                    .runtime
                    .registry()
                    .relation_by_id(*id)
                    .ok_or_else(|| contract("horizon selected relation absent"))?;
                let schema =
                    pse_schema::arrow::relation_schema_ref(result.runtime.registry(), spec)
                        .map_err(|error| relation(error.into()))?;
                let cursor = super::ResultCursor::from_retained(schema, *id, batches)?;
                let position = positions.entry(*id).or_default();
                super::result_projection::append_result_table_prefix(
                    &self.operations.store,
                    &fence,
                    *id,
                    cursor,
                    &self.operations.pool,
                    &self.acknowledgments,
                    position,
                    None,
                )
                .await?;
            }
        }
        for (id, position) in positions {
            super::result_projection::finish_result_table_prefix(
                &self.operations.store,
                &fence,
                id,
                &self.acknowledgments,
                position,
            )
            .await?;
        }
        self.acknowledgments.complete_present(&fence, "__seeds")?;
        Ok((
            std::mem::take(&mut self.solutions),
            self.solutions_owner
                .take()
                .ok_or_else(|| contract("horizon seed identity owner absent"))?,
        ))
    }
    pub(crate) async fn start(&mut self, cancel: Canceller) -> Result<(), WorkflowError> {
        self.acknowledgments.admitted()?;
        if self.operations.policy.heartbeat.is_zero() || self.operations.policy.lease.is_zero() {
            return Err(contract(
                "durable heartbeat and lease intervals must be positive",
            ));
        }
        if self.heartbeat.is_some() {
            return Ok(());
        }
        let fence = self
            .fence
            .get()
            .ok_or_else(|| contract("no native claim before admission"))?
            .clone();
        self.heartbeat = Some(Heartbeat::spawn(&self.operations, fence, cancel));
        Ok(())
    }
    pub(super) async fn finish(mut self, result: &RunResult, cancelled: bool) -> DurableRecord {
        // Keep renewal alive through every export phase, including progress drain.
        let progress = self.stream.finish().await;
        let ingestion = async {
            if self.horizon_prefix.is_some() {
                return self.finish_horizon_prefix(result).await;
            }
            self.store_tables(result).await.map_err(|error| {
                super::diagnostics::operation_context(error, "durable.finish.tables")
            })?;
            store_seeds(
                &self.operations,
                self.fence()?,
                result,
                &self.acknowledgments,
                #[cfg(all(test, feature = "canonical-tests"))]
                &self.stream.export_gate,
            )
            .await
            .map_err(|error| super::diagnostics::operation_context(error, "durable.finish.seeds"))
        }
        .await;
        let tables_complete = ingestion.is_ok();
        let ingestion_error = match ingestion {
            Ok((solutions, owner)) => {
                self.solutions = solutions;
                self.solutions_owner = Some(owner);
                None
            }
            Err(error) => Some(Arc::new(error)),
        };
        let error = ingestion_error;
        let mut outcome = classify(result, cancelled);
        if let Some(error) = &error {
            if !self.cancelled_refusal(error) {
                // Cleanup cannot convert an unrelated or unacknowledged error into
                // successful cancellation, even if the study later becomes cancelled.
                return self.reject_export(error.clone()).await;
            }
            if tables_complete {
                outcome.state = AttemptState::Cancelled;
            } else {
                outcome = failure(error, true);
            }
        }
        let original = if tables_complete {
            result.completion().ok()
        } else {
            None
        };
        let (completion, owner) = match copy_original_completion(
            &self.operations.pool,
            original,
            &(&outcome.detail, &progress),
        ) {
            Ok(prepared) => prepared,
            Err(error) => return self.reject_export(Arc::new(error)).await,
        };
        self.completion_owner = Some(owner);
        let stored = StoredCompletion {
            version: 2,
            attempt_id: self.attempt,
            completion,
            termination: outcome.detail,
            state: outcome.state,
            progress,
        };
        match self.terminate(stored).await {
            Ok(record) => record,
            Err(error) => self.reject_export(Arc::new(error)).await,
        }
    }
    pub(crate) async fn abandon(self, error: &Arc<WorkflowError>) -> DurableRecord {
        self.abandon_with_cancellation(error, false).await
    }
    /// Settle an unadmitted cancelled publication without emitting result or
    /// completion payloads, retaining an earlier typed scientific failure.
    pub(super) async fn abandon_cancelled(self, error: &Arc<WorkflowError>) -> DurableRecord {
        self.abandon_with_cancellation(error, true).await
    }
    /// A lost capacity acknowledgement is uncertain, not proof of cancellation.
    /// Drain existing progress and reject through the existing fenced owner;
    /// never launch a completion payload without a publication slot.
    pub(super) async fn reject_worker_publication(
        mut self,
        error: Arc<WorkflowError>,
    ) -> DurableRecord {
        let _ = self.stream.finish().await;
        self.reject_export(error).await
    }
    async fn abandon_with_cancellation(
        mut self,
        error: &Arc<WorkflowError>,
        cancelled: bool,
    ) -> DurableRecord {
        let progress = self.stream.finish().await;
        if self.fence.get().is_none() {
            return self.failed_record(error.clone());
        }
        let export_error = if is_canonical_failure(error) {
            export_failure(Some(error.clone()), None, self.fence.get())
        } else {
            None
        };
        if let Some(error) = &export_error
            && !self.cancelled_refusal(error)
        {
            return self.reject_export(error.clone()).await;
        }
        let outcome = failure(error, cancelled || export_error.is_some());
        let stored = StoredCompletion {
            version: 2,
            attempt_id: self.attempt,
            completion: None,
            termination: outcome.detail,
            state: outcome.state,
            progress,
        };
        match self.terminate(stored).await {
            Ok(record) => record,
            Err(error) => self.reject_export(Arc::new(error)).await,
        }
    }
    async fn stop_heartbeat(&mut self) -> Option<Arc<WorkflowError>> {
        match self.heartbeat.take() {
            Some(heartbeat) => heartbeat.stop().await,
            None => None,
        }
    }
    async fn reject_export(&mut self, error: Arc<WorkflowError>) -> DurableRecord {
        let heartbeat = self.stop_heartbeat().await;
        let error =
            export_failure(Some(error.clone()), heartbeat, self.fence.get()).unwrap_or(error);
        if let Some(fence) = self.fence.get()
            && self
                .operations
                .store
                .close_result_ingestion(fence, &format!("export-close:{}", fence.attempt()))
                .await
                .is_err()
        {
            let _ = self
                .operations
                .store
                .recover_closed_attempt(fence.run(), &format!("export-cleanup:{}", fence.attempt()))
                .await;
        }
        self.failed_record(error)
    }
    pub(crate) fn cancelled_refusal(&self, error: &WorkflowError) -> bool {
        self.fence
            .get()
            .is_some_and(|fence| acknowledged_cancellation(error, fence))
    }
    fn failed_record(&mut self, error: Arc<WorkflowError>) -> DurableRecord {
        DurableRecord {
            attempt_id: self.attempt,
            attempt_key: self.fence.get().map(|f| f.attempt().to_owned()),
            run: self.run.take(),
            attempt: Err(error),
            manifest: None,
            completion: None,
            solutions: Vec::new(),
            _solutions_owner: None,
            _metadata_owner: self.metadata_owner.take(),
        }
    }
    fn fence(&self) -> Result<&AttemptFence, WorkflowError> {
        self.fence
            .get()
            .ok_or_else(|| contract("canonical execution claim absent"))
    }
    async fn store_tables(&self, result: &RunResult) -> Result<(), WorkflowError> {
        let fence = self.fence()?.clone();
        let detail = self.output.detail(result.usable());
        let mut allowance = detail.map(super::study_demand::DetailAllowance::new);
        let summary = result.summary_table_ids();
        let selected = detail
            .map(|detail| {
                detail
                    .relation_names()
                    .map(|name| {
                        result
                            .runtime
                            .registry()
                            .relation(name)
                            .map(|spec| spec.id)
                            .ok_or_else(|| contract("study output relation disappeared"))
                    })
                    .collect::<Result<std::collections::BTreeSet<_>, _>>()
            })
            .transpose()?;
        let advertised = result.table_ids();
        if selected
            .as_ref()
            .is_some_and(|selected| selected.iter().any(|id| !advertised.contains(id)))
        {
            return Err(contract(
                "selected study relation is unavailable for this operation",
            ));
        }
        let mut queue = super::result_projection::ResultAppendQueue::new(
            &self.operations.store,
            &fence,
            &self.operations.pool,
            &self.acknowledgments,
        );
        for relation in advertised.into_iter().filter(|id| {
            selected
                .as_ref()
                .is_none_or(|selected| summary.contains(id) || selected.contains(id))
        }) {
            #[cfg(all(test, feature = "canonical-tests"))]
            FinishExportGate::at(&self.stream.export_gate, FinishExportPhase::Table).await?;
            let selected_detail = selected
                .as_ref()
                .is_some_and(|selected| selected.contains(&relation));
            let selector = if selected_detail {
                let name = result
                    .runtime
                    .registry()
                    .relation_by_id(relation)
                    .ok_or_else(|| contract("selected study relation disappeared"))?
                    .key
                    .qualified_name();
                let detail =
                    detail.ok_or_else(|| contract("selected study detail demand absent"))?;
                Some(
                    match detail.selector(&name, relation, self.operations.pool.clone()) {
                        Ok(selector) => selector,
                        Err(error)
                            if queue.has_optional_retention()
                                && super::result_projection::projection_memory_refusal(&error) =>
                        {
                            queue.yield_optional().await?;
                            detail.selector(&name, relation, self.operations.pool.clone())?
                        }
                        Err(error) => return Err(error),
                    },
                )
            } else {
                None
            };
            super::result_projection::store_run_result_table_queued(
                &mut queue,
                result,
                relation,
                selector.as_ref(),
                if selected_detail {
                    allowance.as_mut()
                } else {
                    None
                },
            )
            .await?;
        }
        queue.flush().await
    }
    async fn terminate(
        &mut self,
        mut stored: StoredCompletion,
    ) -> Result<DurableRecord, WorkflowError> {
        let fence = self.fence()?.clone();
        let current = self
            .operations
            .store
            .canonical_attempt(fence.attempt())
            .await
            .map_err(|error| {
                super::diagnostics::operation_context(error.into(), "durable.terminate.attempt")
            })?
            .ok_or_else(|| contract("completion attempt absent"))?;
        if current.terminal() {
            let _ = self.stop_heartbeat().await;
            return self
                .operations
                .record(fence.run(), fence.attempt())
                .await
                .map_err(|error| {
                    super::diagnostics::operation_context(
                        error,
                        "durable.terminate.record_terminal",
                    )
                });
        }
        let current_run = self
            .operations
            .store
            .canonical_run(fence.run())
            .await
            .map_err(|error| {
                super::diagnostics::operation_context(error.into(), "durable.terminate.run")
            })?
            .ok_or_else(|| contract("completion run absent"))?;
        if current_run.cancelled {
            stored.state = AttemptState::Cancelled;
        }
        let owner = match self.completion_owner.take() {
            Some(owner) => owner,
            None => completion_copy_owner(&self.operations.pool, &stored)?,
        };
        let bytes = serde_json::to_vec(&stored).map_err(|e| contract(e.to_string()))?;
        let mut receipt = if stored.state == AttemptState::Cancelled || !current.ingestion_open() {
            inline_completion(stored.clone())
        } else {
            #[cfg(all(test, feature = "canonical-tests"))]
            FinishExportGate::at(&self.stream.export_gate, FinishExportPhase::Completion).await?;
            match write_chunks(
                &self.operations.store,
                &fence,
                "__completion",
                0,
                &bytes,
                &self.acknowledgments,
                owner.clone(),
            )
            .await
            .map_err(|error| {
                super::diagnostics::operation_context(error, "durable.terminate.completion")
            }) {
                Ok(receipt) => receipt,
                Err(error) if self.cancelled_refusal(&error) => {
                    stored.state = AttemptState::Cancelled;
                    inline_completion(stored.clone())
                }
                Err(error) => return Err(error),
            }
        };
        let mut acknowledged_manifest = None;
        let closed = if stored.state == AttemptState::Cancelled {
            if let Some(error) = self.stop_heartbeat().await
                && !self.cancelled_refusal(&error)
            {
                return Err(WorkflowError::Shared(error));
            }
            if !current_run.cancelled {
                self.operations
                    .store
                    .cancel_run(fence.run(), &format!("cancel:{}", fence.attempt()))
                    .await
                    .map_err(|error| {
                        super::diagnostics::operation_context(
                            error.into(),
                            "durable.terminate.cancel",
                        )
                    })?;
            }
            self.operations
                .store
                .recover_closed_attempt(fence.run(), &format!("recovery:{}", fence.attempt()))
                .await
                .map_err(|error| {
                    super::diagnostics::operation_context(
                        error.into(),
                        "durable.terminate.recover_cancelled",
                    )
                })?
        } else {
            if let super::ProgressCoverage::Sampled { acknowledged, .. } = &stored.progress
                && *acknowledged != self.acknowledgments.rows(&fence, "__progress")?
            {
                return Err(contract(
                    "original progress coverage differs from actual append acknowledgments",
                ));
            }
            let acknowledgments = self.acknowledgments.freeze(&fence)?;
            let (admission, admission_operation) = match uncertain_progress_queue(&stored.progress)
            {
                Some(queued) => match self
                    .operations
                    .store
                    .close_result_ingestion(&fence, &format!("close-readback:{}", fence.attempt()))
                    .await
                {
                    Ok(closed) => (
                        self.operations
                            .store
                            .reconcile_progress_uncertainty(&closed, acknowledgments, queued)
                            .await,
                        "durable.terminate.reconcile_progress",
                    ),
                    Err(error) => (Err(error), "durable.terminate.close_progress"),
                },
                None => (
                    self.operations
                        .store
                        .close_acknowledged(
                            &fence,
                            &format!("close:{}", fence.attempt()),
                            acknowledgments,
                        )
                        .await,
                    "durable.terminate.close_acknowledged",
                ),
            };
            match admission {
                Ok(manifest) => {
                    let closed = manifest.closed().clone();
                    acknowledged_manifest = Some(manifest);
                    closed
                }
                Err(error) if canonical_cancellation(&error, &fence) => {
                    if let Some(error) = self.stop_heartbeat().await
                        && !self.cancelled_refusal(&error)
                    {
                        return Err(WorkflowError::Shared(error));
                    }
                    stored.state = AttemptState::Cancelled;
                    receipt = inline_completion(stored.clone());
                    self.operations
                        .store
                        .recover_closed_attempt(
                            fence.run(),
                            &format!("recovery:{}", fence.attempt()),
                        )
                        .await
                        .map_err(|error| {
                            super::diagnostics::operation_context(
                                error.into(),
                                "durable.terminate.recover_after_close",
                            )
                        })?
                }
                Err(error) => {
                    return Err(super::diagnostics::operation_context(
                        error.into(),
                        admission_operation,
                    ));
                }
            }
        };
        let class = match stored.state {
            AttemptState::Completed => TerminalClass::Succeeded,
            AttemptState::Partial => TerminalClass::Partial,
            AttemptState::Cancelled => TerminalClass::Cancelled,
            _ => TerminalClass::Failed,
        };
        let seeds = if stored.completion.is_some() {
            SeedEligibility::CompleteOriginalInventory
        } else {
            SeedEligibility::Unavailable
        };
        if let Some(manifest) = acknowledged_manifest {
            // The owner keeps original completion and seed identities. No protected
            // result reopen, payload reread or inventory reconstruction follows success.
            let completion =
                serde_json::to_vec(&receipt).map_err(|error| contract(error.to_string()))?;
            if let Some(error) = self.stop_heartbeat().await {
                if self.cancelled_refusal(&error) {
                    return self
                        .settle_cancelled_completion(&fence, stored, seeds)
                        .await;
                }
                return Err(WorkflowError::Shared(error));
            }
            #[allow(
                unsafe_code,
                reason = "joined scientific owner admits original complete export"
            )]
            // SAFETY: native work and all publication children joined; the original
            // scientific assessments and exact actual ACK inventory are retained here.
            let sealed = unsafe {
                self.operations
                    .store
                    .seal_acknowledged(
                        &manifest,
                        &format!("seal:{}", fence.attempt()),
                        class,
                        &completion,
                        seeds,
                    )
                    .await
            };
            match sealed {
                Ok(ack) => {
                    let (attempt, run) = ack.into_parts();
                    #[cfg(all(test, feature = "canonical-tests"))]
                    FinishExportGate::at(
                        &self.stream.export_gate,
                        FinishExportPhase::AcknowledgedSeal,
                    )
                    .await?;
                    let metadata_owner = durable_metadata_owner(
                        &self.operations.pool,
                        Some(&run),
                        Some(&attempt),
                        Some(manifest.row()),
                        attempt.key.len(),
                        0,
                    )?;
                    return Ok(DurableRecord {
                        attempt_id: self.attempt,
                        attempt_key: Some(attempt.key.clone()),
                        run: Some(run),
                        attempt: Ok(attempt),
                        manifest: Some(manifest.row().clone()),
                        completion: Some(Arc::new(pse_columnar::Leased::new(
                            Arc::new(stored),
                            owner,
                        ))),
                        solutions: if seeds == SeedEligibility::CompleteOriginalInventory {
                            std::mem::take(&mut self.solutions)
                        } else {
                            Vec::new()
                        },
                        _solutions_owner: self.solutions_owner.take(),
                        _metadata_owner: Some(metadata_owner),
                    });
                }
                Err(error) if canonical_cancellation(&error, &fence) => {
                    return self
                        .settle_cancelled_completion(&fence, stored, seeds)
                        .await;
                }
                Err(error) => {
                    return Err(super::diagnostics::operation_context(
                        error.into(),
                        "durable.terminate.seal_acknowledged",
                    ));
                }
            }
        }
        let heartbeat = self.heartbeat.take();
        match self
            .operations
            .settle(
                &closed,
                &format!("seal:{}", fence.attempt()),
                class,
                &receipt,
                seeds,
                heartbeat,
            )
            .await
        {
            Ok(record) => Ok(record),
            Err(error) if self.cancelled_refusal(&error) => {
                // Cancellation after intentional close has an acknowledged refusal.
                // Preserve the fully exported original assessments under recovery authority.
                self.settle_cancelled_completion(&fence, stored, seeds)
                    .await
            }
            Err(error) => Err(error),
        }
    }
    async fn settle_cancelled_completion(
        &self,
        fence: &AttemptFence,
        mut stored: StoredCompletion,
        seeds: SeedEligibility,
    ) -> Result<DurableRecord, WorkflowError> {
        stored.state = AttemptState::Cancelled;
        let closed = self
            .operations
            .store
            .recover_closed_attempt(fence.run(), &format!("recovery:{}", fence.attempt()))
            .await
            .map_err(|error| {
                super::diagnostics::operation_context(
                    error.into(),
                    "durable.settle_cancelled.recover",
                )
            })?;
        self.operations
            .settle(
                &closed,
                &format!("seal-cancel:{}", fence.attempt()),
                TerminalClass::Cancelled,
                &inline_completion(stored),
                seeds,
                None,
            )
            .await
    }
}
/// Version 1 of the typed detail of an attempt's termination, stored as the attempt's
/// termination-detail document (ADR-0116 Outcome 6): the typed values that explain why the
/// try ended as it did.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TerminationDetail {
    /// Document version.
    pub version: pse_model::document::Version<2>,
    /// Why the try ended.
    pub cause: TerminationCause,
    /// Typed occurrence try, including failures before native admission.
    pub point: Option<pse_model::study::PointAttemptOutcome>,
    /// Purpose-specific retry classification; severity does not grant retries.
    pub retry_failure: Option<pse_model::study::RetryFailure>,
    /// Effect knowledge is independent of attempt lifecycle.
    pub effect: pse_model::study::EffectState,
}

/// The cause of a try's end, beside its typed termination code.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TerminationCause {
    /// The run failed with an error, or cancellation stopped it.
    Error {
        /// The error's message.
        diagnostic: pse_model::diagnostic::BoundaryDiagnostic,
    },
    /// Infrastructure failed the try; authored point policy decides whether retry is permitted.
    Infrastructure {
        /// Preserved original typed cause and observations.
        diagnostic: pse_model::diagnostic::BoundaryDiagnostic,
    },
    /// The joined run was assessed.
    Assessment {
        /// Every requested candidate is a result.
        usable: bool,
        /// The use of each requested candidate, in request order.
        candidate_use: Vec<CandidateUse>,
        /// Original scientific diagnostic envelopes, retained without rendered-string reconstruction.
        diagnostics: Vec<pse_model::diagnostic::BoundaryDiagnostic>,
    },
}

fn detail(cause: TerminationCause, retryable: bool) -> TerminationDetail {
    TerminationDetail {
        version: pse_model::document::Version,
        cause,
        point: None,
        retry_failure: Some(if retryable {
            pse_model::study::RetryFailure::Transient
        } else {
            pse_model::study::RetryFailure::Deterministic
        }),
        effect: pse_model::study::EffectState::Absent,
    }
}
fn failure(error: &WorkflowError, cancelled: bool) -> Outcome {
    let cancelled = cancelled
        || matches!(
            error,
            WorkflowError::Math(crate::math::MathRuntimeError::Cancelled)
        );
    let retryable = matches!(error, WorkflowError::Canonical(error) if error.retryable())
        || matches!(
            error,
            WorkflowError::Math(crate::math::MathRuntimeError::Infrastructure(_))
        );
    let infrastructure = matches!(error, WorkflowError::Canonical(error) if error.is_infrastructure())
        || matches!(
            error,
            WorkflowError::Math(crate::math::MathRuntimeError::Infrastructure(_))
        );
    let cause = if infrastructure {
        TerminationCause::Infrastructure {
            diagnostic: error.boundary_diagnostic(),
        }
    } else {
        TerminationCause::Error {
            diagnostic: error.boundary_diagnostic(),
        }
    };
    let mut detail = detail(cause, retryable);
    if matches!(error, WorkflowError::Canonical(_)) {
        detail.effect = pse_model::study::EffectState::Unknown;
    }
    Outcome {
        state: if cancelled {
            AttemptState::Cancelled
        } else {
            AttemptState::Failed
        },
        detail,
        retryable,
    }
}
fn classify(result: &RunResult, cancelled: bool) -> Outcome {
    if let Err(error) = result.report() {
        return failure(error, cancelled);
    }
    let any = result.assessments().iter().any(|a| a.permits_result);
    Outcome {
        state: if cancelled {
            AttemptState::Cancelled
        } else if result.usable() {
            AttemptState::Completed
        } else if any {
            AttemptState::Partial
        } else {
            AttemptState::Failed
        },
        detail: detail(
            TerminationCause::Assessment {
                usable: result.usable(),
                candidate_use: result.assessments().iter().map(|a| a.usability).collect(),
                diagnostics: result
                    .completion()
                    .map_or_else(|e| vec![e.boundary_diagnostic()], |c| c.diagnostics.clone()),
            },
            false,
        ),
        retryable: false,
    }
}
fn identities(request: &RunRequest) -> Result<(ContentHash, Option<ContentHash>), WorkflowError> {
    let math = |e: pse_backend_native::ProblemError| {
        WorkflowError::Math(crate::math::MathRuntimeError::from(e))
    };
    Ok(match request {
        RunRequest::Horizon(horizon) => (horizon.identity, None),
        RunRequest::Modeling(steps) => {
            let mut h = FramedHasher::new(Frame::DurableModelingRequestV3);
            h.u64(steps.len() as u64);
            for step in steps {
                h.hash(&step.solve.request_identity().map_err(math)?.as_id());
            }
            let preparation = match steps.as_slice() {
                [one] => Some(one.solve.preparation_identity().map_err(math)?),
                _ => None,
            };
            (h.finish_hash(), preparation)
        }
        RunRequest::Simulation(s) => (s.identity(), Some(s.identity())),
        RunRequest::Fit(f) => (f.problem.key, Some(f.problem.key)),
        #[cfg(feature = "solver-diffsol")]
        RunRequest::Shooting { problem, initial } => (
            problem.request_identity(initial.as_deref()).as_id(),
            Some(problem.request_identity(None).as_id()),
        ),
    })
}
fn request_inputs(request: &RunRequest) -> Result<Vec<Revision>, WorkflowError> {
    let mut revisions = match request {
        RunRequest::Horizon(horizon) => horizon
            .sources
            .iter()
            .map(|source| source.recorded_revision().cloned())
            .collect::<Result<Vec<_>, _>>()?,
        RunRequest::Modeling(steps) => steps
            .iter()
            .map(|s| s.source.recorded_revision().cloned())
            .collect::<Result<Vec<_>, _>>()?,
        RunRequest::Simulation(s) => vec![s.source.recorded_revision()?.clone()],
        RunRequest::Fit(f) => vec![f.source.recorded_revision()?.clone()],
        #[cfg(feature = "solver-diffsol")]
        RunRequest::Shooting { problem, .. } => {
            vec![problem.simulation.source.recorded_revision()?.clone()]
        }
    };
    revisions.sort_by(|a: &Revision, b| a.key.cmp(&b.key));
    revisions.dedup_by(|a, b| a.key == b.key);
    Ok(revisions)
}
fn request_context(
    request: &RunRequest,
) -> Result<(&super::Runtime, &super::PhysicalContext), WorkflowError> {
    let source = match request {
        RunRequest::Horizon(horizon) => horizon
            .sources
            .first()
            .ok_or_else(|| contract("horizon source absent"))?,
        RunRequest::Modeling(steps) => {
            &steps
                .first()
                .ok_or_else(|| contract("empty scientific request"))?
                .source
        }
        RunRequest::Simulation(s) => &s.source,
        RunRequest::Fit(f) => &f.source,
        #[cfg(feature = "solver-diffsol")]
        RunRequest::Shooting { problem, .. } => &problem.simulation.source,
    };
    Ok((&source.runtime, &source.physical))
}
fn request_provenance(request: &RunRequest) -> Result<Vec<serde_json::Value>, WorkflowError> {
    let encode = |value| serde_json::to_value(value).map_err(|e| contract(e.to_string()));
    match request {
        RunRequest::Horizon(horizon) => Ok(vec![
            serde_json::from_slice(&horizon.provenance)
                .map_err(|error| contract(error.to_string()))?,
        ]),
        RunRequest::Modeling(steps) => steps
            .iter()
            .map(|step| {
                serde_json::to_value((
                    &step.starts,
                    step.solve.numerical_strategy(),
                    step.bound_source.as_deref().map(|bound| &**bound),
                ))
                .map_err(|e| contract(e.to_string()))
            })
            .collect(),
        RunRequest::Simulation(simulation) => Ok(vec![encode(simulation.profile())?]),
        RunRequest::Fit(fit) => Ok(vec![
            serde_json::to_value(fit.numerical_strategy()).map_err(|e| contract(e.to_string()))?,
        ]),
        #[cfg(feature = "solver-diffsol")]
        RunRequest::Shooting { problem, .. } => Ok(vec![
            serde_json::to_value(
                problem
                    .numerical_strategy()
                    .map_err(crate::math::MathRuntimeError::from)?,
            )
            .map_err(|e| contract(e.to_string()))?,
        ]),
    }
}
fn inline_completion(stored: StoredCompletion) -> CompletionReceipt {
    CompletionReceipt {
        version: 1,
        batch_count: 0,
        payload_bytes: 0,
        digest: String::new(),
        inline: Some(Box::new(stored)),
    }
}
fn canonical_cancellation(
    error: &pse_operations::canonical::CanonicalError,
    fence: &AttemptFence,
) -> bool {
    matches!(error, pse_operations::canonical::CanonicalError::StudyCancellation { run, attempt, generation, .. }
        if run == fence.run() && attempt == fence.attempt() && *generation == fence.generation())
}
fn acknowledged_cancellation(error: &WorkflowError, fence: &AttemptFence) -> bool {
    match error {
        WorkflowError::Canonical(error) => canonical_cancellation(error, fence),
        WorkflowError::Shared(error) => acknowledged_cancellation(error, fence),
        _ => false,
    }
}
fn is_canonical_failure(error: &WorkflowError) -> bool {
    match error {
        WorkflowError::Canonical(_) => true,
        WorkflowError::Shared(error) => is_canonical_failure(error),
        _ => false,
    }
}
fn export_failure(
    ingestion: Option<Arc<WorkflowError>>,
    heartbeat: Option<Arc<WorkflowError>>,
    fence: Option<&AttemptFence>,
) -> Option<Arc<WorkflowError>> {
    // Any unrelated error takes precedence over an acknowledged cancellation.
    let cancelled =
        |error: &WorkflowError| fence.is_some_and(|f| acknowledged_cancellation(error, f));
    match (ingestion, heartbeat) {
        (Some(ingestion), Some(heartbeat)) if cancelled(&ingestion) && !cancelled(&heartbeat) => {
            Some(heartbeat)
        }
        (Some(error), _) | (None, Some(error)) => Some(error),
        (None, None) => None,
    }
}
#[derive(Debug)]
struct Heartbeat {
    stop: tokio::sync::oneshot::Sender<()>,
    task: tokio::task::JoinHandle<()>,
    failure: Arc<Mutex<Option<Arc<WorkflowError>>>>,
}
impl Heartbeat {
    fn spawn(operations: &Operations, fence: AttemptFence, cancel: Canceller) -> Self {
        let (stop, mut stopped) = tokio::sync::oneshot::channel();
        let operations = operations.clone();
        let failure = Arc::new(Mutex::new(None));
        let recorded = failure.clone();
        let task = tokio::spawn(crate::workflow::inherit_runtime_activity(async move {
            let mut interval = tokio::time::interval(operations.policy.heartbeat);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {_=&mut stopped=>break,_=interval.tick()=>{if let Err(error)=operations.store.renew_attempt(&fence,operations.policy.lease).await{let error=super::diagnostics::operation_context(WorkflowError::Canonical(error),"durable.heartbeat.renew");if let Ok(mut failure)=recorded.lock(){*failure=Some(Arc::new(error));}cancel();break;}}}
            }
        }));
        Self {
            stop,
            task,
            failure,
        }
    }
    async fn stop(self) -> Option<Arc<WorkflowError>> {
        let _ = self.stop.send(());
        if let Err(error) = self.task.await {
            return Some(Arc::new(contract(format!("heartbeat join: {error}"))));
        }
        match self.failure.lock() {
            Ok(mut failure) => failure.take(),
            Err(_) => Some(Arc::new(contract("heartbeat failure observation poisoned"))),
        }
    }
}
const PROGRESS_EVENT_BYTES: usize = 16 * 1024;

fn progress_memory(
    pool: &Arc<dyn MemoryPool>,
    capacity: usize,
) -> Option<Arc<pse_columnar::AllocationLease>> {
    let extent = (2 * capacity + 1) * PROGRESS_EVENT_BYTES
        + 8 * capacity
            .saturating_mul(PROGRESS_EVENT_BYTES)
            .min(RESULT_BATCH_BYTES);
    let reservation = MemoryConsumer::new("canonical:ordinary-progress").register(pool);
    reservation
        .try_grow(extent)
        .ok()
        .map(|()| pse_columnar::AllocationLease::new(reservation))
}

struct Tap {
    step: AtomicI32,
    sender: Mutex<Option<tokio::sync::mpsc::Sender<super::ProgressEventDocument>>>,
    observed: AtomicU64,
    queued: AtomicU64,
    dropped: AtomicU64,
    acknowledged: AtomicU64,
    counter_overflow: AtomicBool,
    // Escaped taps and the owned writer share the original admission lifetime.
    _memory: Option<Arc<pse_columnar::AllocationLease>>,
}
impl std::fmt::Debug for Tap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tap")
            .field("observed", &self.observed)
            .finish_non_exhaustive()
    }
}
impl Tap {
    fn close(&self) {
        // A poisoned callback-control mutex cannot keep the writer channel alive.
        match self.sender.lock() {
            Ok(mut sender) => {
                sender.take();
            }
            Err(poisoned) => {
                poisoned.into_inner().take();
            }
        }
    }
    fn new(
        sender: Option<tokio::sync::mpsc::Sender<super::ProgressEventDocument>>,
        memory: Option<Arc<pse_columnar::AllocationLease>>,
    ) -> Self {
        Self {
            step: AtomicI32::new(0),
            sender: Mutex::new(sender),
            observed: AtomicU64::new(0),
            queued: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
            acknowledged: AtomicU64::new(0),
            counter_overflow: AtomicBool::new(false),
            _memory: memory,
        }
    }
    fn count(&self, counter: &AtomicU64, amount: u64) -> Option<u64> {
        match counter.try_update(Ordering::AcqRel, Ordering::Acquire, |value| {
            value.checked_add(amount)
        }) {
            Ok(previous) => Some(previous),
            Err(_) => {
                self.counter_overflow.store(true, Ordering::Release);
                None
            }
        }
    }
    fn coverage(&self, writer: super::ProgressWriterOutcome) -> super::ProgressCoverage {
        if self.counter_overflow.load(Ordering::Acquire) {
            return super::ProgressCoverage::Unknown {};
        }
        let queued = self.queued.load(Ordering::Acquire);
        let acknowledged = self.acknowledged.load(Ordering::Acquire);
        super::ProgressCoverage::Sampled {
            observed: self.observed.load(Ordering::Acquire),
            queued,
            dropped_before_queue: self.dropped.load(Ordering::Acquire),
            acknowledged,
            unacknowledged: queued.saturating_sub(acknowledged),
            writer,
        }
    }
}
impl ProgressTap for Tap {
    fn observe(&self, event: &Event) {
        // No callback waits for a mutex, channel, runtime or canonical request.
        // The successful producer retains this short exclusion through send, so
        // its observed sequence cannot be overtaken by another queued producer.
        let Ok(sender_guard) = self.sender.try_lock() else {
            let _ = self.count(&self.observed, 1);
            let _ = self.count(&self.dropped, 1);
            return;
        };
        let sequence = self.count(&self.observed, 1);
        let Some(sender) = sender_guard.as_ref() else {
            let _ = self.count(&self.dropped, 1);
            return;
        };
        let Ok(permit) = sender.clone().try_reserve_owned() else {
            let _ = self.count(&self.dropped, 1);
            return;
        };
        let Some(sequence) = sequence.and_then(|value| i64::try_from(value).ok()) else {
            self.counter_overflow.store(true, Ordering::Release);
            let _ = self.count(&self.dropped, 1);
            return;
        };
        if !super::progress_documents::bounded_event(event, PROGRESS_EVENT_BYTES) {
            let _ = self.count(&self.dropped, 1);
            return;
        }
        let mut document = super::ProgressEventDocument::from(event);
        document.step = Some(self.step.load(Ordering::Acquire));
        document.sequence = Some(sequence);
        document.at = Some(chrono::Utc::now().timestamp_micros());
        let _ = self.count(&self.queued, 1);
        permit.send(document);
    }
}
#[derive(Debug)]
struct Streamer {
    tap: Arc<Tap>,
    ready: Arc<tokio::sync::Notify>,
    task: Option<tokio::task::JoinHandle<Result<(), super::ProgressWriteStage>>>,
    outcome: super::ProgressWriterOutcome,
    acknowledgments: Arc<PublicationAcks>,
    fence: Arc<OnceLock<AttemptFence>>,
    #[cfg(all(test, feature = "canonical-tests"))]
    export_gate: Arc<OnceLock<FinishExportGate>>,
}
impl Streamer {
    fn new(
        operations: &Operations,
        fence: Arc<OnceLock<AttemptFence>>,
        acknowledgments: Arc<PublicationAcks>,
    ) -> Self {
        let capacity = operations.policy.batch.clamp(1, 512);
        // Queue + retained batch + one producer permit, including DTO tree nodes,
        // and bounded serialization/append/receipt scratch are admitted up front.
        let memory = progress_memory(&operations.pool, capacity);
        let ready = Arc::new(tokio::sync::Notify::new());
        #[cfg(all(test, feature = "canonical-tests"))]
        let export_gate = Arc::new(OnceLock::new());
        if memory.is_none() {
            return Self {
                tap: Arc::new(Tap::new(None, None)),
                ready,
                task: None,
                outcome: super::ProgressWriterOutcome::MemoryRefused,
                acknowledgments,
                fence,
                #[cfg(all(test, feature = "canonical-tests"))]
                export_gate,
            };
        }
        let (sender, mut receiver) =
            tokio::sync::mpsc::channel::<super::ProgressEventDocument>(capacity);
        let tap = Arc::new(Tap::new(Some(sender), memory));
        let writer_tap = tap.clone();
        let signal = ready.clone();
        let operations = operations.clone();
        #[cfg(all(test, feature = "canonical-tests"))]
        let writer_gate = export_gate.clone();
        let writer_acknowledgments = acknowledgments.clone();
        let writer_fence = fence.clone();
        let task = tokio::spawn(crate::workflow::inherit_runtime_activity(async move {
            let fence = writer_fence;
            while fence.get().is_none() {
                tokio::select! {_=signal.notified()=>{},event=receiver.recv()=>{if event.is_some(){return Err(super::ProgressWriteStage::Claim {});}return Ok(());}}
            }
            let fence = fence
                .get()
                .ok_or(super::ProgressWriteStage::Claim {})?
                .clone();
            let mut ordinal = 0_u64;
            while let Some(event) = receiver.recv().await {
                let mut events = Vec::with_capacity(capacity);
                events.push(event);
                let deadline = std::time::Instant::now()
                    .checked_add(operations.policy.flush)
                    .ok_or(super::ProgressWriteStage::Encoding {})?;
                while events.len() < capacity {
                    let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                    let limit = capacity - events.len();
                    match tokio::time::timeout(remaining, receiver.recv_many(&mut events, limit))
                        .await
                    {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {}
                    }
                }
                let mut ranges = std::collections::VecDeque::from([(0, events.len())]);
                while let Some((start, end)) = ranges.pop_front() {
                    let mut output = ProgressBuffer {
                        bytes: Vec::new(),
                        full: false,
                    };
                    output
                        .bytes
                        .try_reserve_exact(
                            capacity
                                .saturating_mul(PROGRESS_EVENT_BYTES)
                                .min(RESULT_BATCH_BYTES),
                        )
                        .map_err(|_| super::ProgressWriteStage::Allocation {})?;
                    match serde_json::to_writer(&mut output, &events[start..end]) {
                        Ok(()) => {
                            #[cfg(all(test, feature = "canonical-tests"))]
                            FinishExportGate::at(&writer_gate, FinishExportPhase::Progress)
                                .await
                                .map_err(|_| super::ProgressWriteStage::Publication {})?;
                            let acknowledged = operations
                                .store
                                .append_result_batch(
                                    &fence,
                                    &format!("progress:{}:{ordinal}", fence.attempt()),
                                    "__progress",
                                    ordinal,
                                    pse_operations::canonical_execution::ResultPayload::new_owned(
                                        output.bytes,
                                        writer_tap
                                            ._memory
                                            .clone()
                                            .ok_or(super::ProgressWriteStage::Allocation {})?,
                                    )
                                    .map_err(|_| super::ProgressWriteStage::Encoding {})?,
                                    (end - start) as u64,
                                )
                                .await
                                .map_err(|_| super::ProgressWriteStage::CanonicalAppend {})?;
                            if acknowledged.row_count != (end - start) as u64 {
                                return Err(super::ProgressWriteStage::Acknowledgment {});
                            }
                            let rows = acknowledged.row_count;
                            writer_acknowledgments
                                .append(&fence, acknowledged)
                                .map_err(|_| super::ProgressWriteStage::Acknowledgment {})?;
                            let _ = writer_tap.count(&writer_tap.acknowledged, rows);
                            ordinal = ordinal
                                .checked_add(1)
                                .ok_or(super::ProgressWriteStage::Acknowledgment {})?;
                        }
                        Err(_) if output.full && end - start > 1 => {
                            let middle = start + (end - start) / 2;
                            ranges.push_front((middle, end));
                            ranges.push_front((start, middle));
                        }
                        Err(_) if output.full => {
                            return Err(super::ProgressWriteStage::Encoding {});
                        }
                        Err(_) => return Err(super::ProgressWriteStage::Encoding {}),
                    }
                }
            }
            Ok(())
        }));
        Self {
            tap,
            ready,
            task: Some(task),
            outcome: super::ProgressWriterOutcome::Complete,
            acknowledgments,
            fence,
            #[cfg(all(test, feature = "canonical-tests"))]
            export_gate,
        }
    }
    async fn finish(&mut self) -> super::ProgressCoverage {
        self.tap.close();
        if let Some(task) = self.task.take() {
            // Each append already settles under its original bounded canonical
            // request clock. Joining does not start a new request or cancel science.
            self.outcome = match task.await {
                Ok(Ok(())) => super::ProgressWriterOutcome::Complete,
                Ok(Err(stage)) => {
                    super::ProgressWriterOutcome::Failed(super::ProgressWriterFailure { stage })
                }
                Err(_) => super::ProgressWriterOutcome::JoinFailed,
            };
        }
        if let Some(fence) = self.fence.get()
            && self
                .acknowledgments
                .complete_present(fence, "__progress")
                .is_err()
        {
            self.outcome = super::ProgressWriterOutcome::Failed(super::ProgressWriterFailure {
                stage: super::ProgressWriteStage::Acknowledgment {},
            });
        }
        self.tap.coverage(self.outcome)
    }
}
impl Drop for Streamer {
    fn drop(&mut self) {
        // A dropped workflow still closes the owned queue; the detached writer
        // retains its admitted memory only through its bounded pending appends.
        self.tap.close();
    }
}
struct ProgressBuffer {
    bytes: Vec<u8>,
    full: bool,
}
impl std::io::Write for ProgressBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > RESULT_BATCH_BYTES.saturating_sub(self.bytes.len()) {
            self.full = true;
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "progress batch extent",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(crate) fn decode_progress(
    bytes: &[u8],
) -> Result<Vec<super::ProgressEventDocument>, WorkflowError> {
    let events: Vec<super::ProgressEventDocument> =
        serde_json::from_slice(bytes).map_err(|e| contract(e.to_string()))?;
    if events.len() > 512 {
        return Err(contract("progress block event bound"));
    }
    Ok(events)
}
async fn write_chunks(
    store: &CanonicalStore,
    fence: &AttemptFence,
    name: &str,
    first: u64,
    bytes: &[u8],
    acknowledgments: &PublicationAcks,
    owner: Arc<pse_columnar::AllocationLease>,
) -> Result<CompletionReceipt, WorkflowError> {
    acknowledgments.expect(fence, name)?;
    let mut count = 0;
    for (ordinal, chunk) in bytes.chunks(RESULT_BATCH_BYTES).enumerate() {
        let acknowledged = store
            .append_result_batch(
                fence,
                &format!(
                    "chunks:{}:{name}:{}",
                    fence.attempt(),
                    first + ordinal as u64
                ),
                name,
                first + ordinal as u64,
                pse_operations::canonical_execution::ResultPayload::new_owned(
                    chunk.to_vec(),
                    owner.clone(),
                )?,
                chunk.len() as u64,
            )
            .await?;
        acknowledgments.append(fence, acknowledged)?;
        count += 1;
    }
    if name != "__seeds" {
        acknowledgments.complete(fence, name)?;
    }
    Ok(CompletionReceipt {
        version: 1,
        batch_count: count,
        payload_bytes: bytes.len() as u64,
        digest: result_payload_digest(bytes),
        inline: None,
    })
}
#[derive(Clone, Copy)]
enum CompletionSelection<'a> {
    Full(&'a ResultRead),
    Slim(&'a pse_operations::canonical_results::CompletionRead),
}
impl CompletionSelection<'_> {
    fn attempt(&self) -> &CanonicalAttempt {
        match self {
            Self::Full(read) => read.attempt(),
            Self::Slim(read) => read.attempt(),
        }
    }
}
#[allow(
    clippy::too_many_arguments,
    reason = "exact chunk identity, coverage, and extent are independently checked"
)]
async fn read_chunks(
    store: &CanonicalStore,
    pool: &Arc<dyn MemoryPool>,
    read: &ResultRead,
    name: &str,
    first: u64,
    count: u64,
    length: u64,
    digest: &str,
    subset: bool,
) -> Result<(Vec<u8>, Arc<pse_columnar::AllocationLease>), WorkflowError> {
    read_chunks_selection(
        store,
        pool,
        CompletionSelection::Full(read),
        name,
        first,
        count,
        length,
        digest,
        subset,
    )
    .await
}
#[allow(
    clippy::too_many_arguments,
    reason = "the protected result selection, chunk window, extent and digest are separate admission premises checked before pooled allocation"
)]
async fn read_chunks_selection(
    store: &CanonicalStore,
    pool: &Arc<dyn MemoryPool>,
    read: CompletionSelection<'_>,
    name: &str,
    first: u64,
    count: u64,
    length: u64,
    digest: &str,
    subset: bool,
) -> Result<(Vec<u8>, Arc<pse_columnar::AllocationLease>), WorkflowError> {
    match read {
        CompletionSelection::Full(full) => {
            let set = result_set_key(&full.attempt().key, name);
            let descriptor = full
                .sets()
                .iter()
                .find(|s| s.key == set)
                .ok_or_else(|| contract("stored chunk set absent"))?;
            if (!subset && (descriptor.batch_count != count || descriptor.row_count != length))
                || first
                    .checked_add(count)
                    .is_none_or(|end| end > descriptor.batch_count)
                || count != length.div_ceil(RESULT_BATCH_BYTES as u64)
            {
                return Err(contract("stored chunk descriptor differs"));
            }
        }
        CompletionSelection::Slim(slim) => {
            if name != "__completion"
                || first != 0
                || subset
                || slim.chunk_coverage() != Some((count, length))
                || count != length.div_ceil(RESULT_BATCH_BYTES as u64)
            {
                return Err(contract("stored completion chunk coverage differs"));
            }
        }
    }
    let extent = usize::try_from(length).map_err(|_| contract("stored extent overflow"))?;
    let owner = reserve(pool, extent)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(extent)
        .map_err(|_| contract("stored chunk allocation refused"))?;
    // Hydrated opaque file access allocates one extra chunk concurrently with
    // the accumulated JSON document; reserve before any physical file read.
    let staging = MemoryConsumer::new("canonical:opaque-file-read").register(pool);
    staging
        .try_grow(
            extent
                .min(RESULT_BATCH_BYTES)
                .saturating_mul(2)
                .saturating_add(64 * 1024),
        )
        .map_err(pse_engine::EngineError::from)?;
    let response = pooled_response_owner(
        pool,
        "canonical:opaque-file-response",
        pse_operations::canonical::MetadataResponse::Header,
    )?;
    let response: Arc<dyn std::fmt::Debug + Send + Sync> = match read {
        CompletionSelection::Full(full) => Arc::new((full.clone(), response)),
        CompletionSelection::Slim(slim) => Arc::new((slim.clone(), response)),
    };
    pse_operations::canonical::response_ownership_scope(response, async {
        for ordinal in 0..count {
            let payload = match read {
                CompletionSelection::Full(full) => {
                    let set = result_set_key(&full.attempt().key, name);
                    store.result_payload(full, &set, first + ordinal).await?
                }
                CompletionSelection::Slim(slim) => {
                    store.completion_payload(slim, first + ordinal).await?
                }
            };
            let expected =
                (length - ordinal * RESULT_BATCH_BYTES as u64).min(RESULT_BATCH_BYTES as u64);
            if payload.batch.payload.len() as u64 != expected || payload.batch.row_count != expected
            {
                return Err(contract("stored chunk coverage differs"));
            }
            bytes.extend_from_slice(payload.batch.payload.as_slice());
        }
        Ok::<_, WorkflowError>(())
    })
    .await?;
    if result_payload_digest(&bytes) != digest {
        return Err(contract("stored chunk digest differs"));
    }
    Ok((bytes, owner))
}

/// Portable scientific seed encoding stores every IEEE bit; native working sets stay local.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum SeedPayload {
    Root {
        primal: Vec<u64>,
    },
    Nlp {
        primal: Vec<u64>,
        bounds: Option<(Vec<u64>, Vec<u64>)>,
        rows: Option<Vec<u64>>,
        barrier: Option<u64>,
    },
    Highs {
        primal: Option<Vec<u64>>,
        dual: Option<(Vec<u64>, Vec<u64>)>,
        basis: Option<(Vec<i32>, Vec<i32>)>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SeedDocument {
    version: u8,
    warm: SeedPayload,
    prediction: Option<super::modeling::results::PortablePrediction>,
}
/// Sufficient copied seed meaning; no unrelated completion/report tables are retained.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SeedCapsule {
    version: u8,
    header: pse_model::generated::runtime::canonical_result_seeds::Row,
    assessment: pse_model::generated::runtime::candidate_assessments::Row,
    document: SeedDocument,
    manifest: pse_model::generated::runtime::canonical_result_manifests::Row,
}
/// One freshly admitted original seed, sharing its decoded payload and allocation.
/// Retaining this value permits consuming its immutable bytes; a later acquisition
/// still has to establish current original-manifest and scientific permission.
#[derive(Clone, Debug)]
pub struct AdmittedSeed {
    solution: SolutionId,
    header: pse_model::generated::runtime::canonical_result_seeds::Row,
    assessment:
        Arc<pse_columnar::Leased<pse_model::generated::runtime::candidate_assessments::Row>>,
    pub(crate) start: Arc<pse_columnar::Leased<WarmStart>>,
    pub(crate) prediction:
        Option<Arc<pse_columnar::Leased<super::modeling::results::PortablePrediction>>>,
}
impl AdmittedSeed {
    /// The selected original solution identity.
    pub fn solution(&self) -> SolutionId {
        self.solution
    }
    /// Original compatibility and occurrence coordinates checked at acquisition.
    pub fn header(&self) -> &pse_model::generated::runtime::canonical_result_seeds::Row {
        &self.header
    }
    /// The immutable warm start, retaining its decode allocation through clones.
    pub fn start(&self) -> &Arc<pse_columnar::Leased<WarmStart>> {
        &self.start
    }
    /// Copied original assessment at acquisition, not authority for a future read.
    pub fn assessment(&self) -> Option<&pse_model::generated::runtime::candidate_assessments::Row> {
        Some(&self.assessment)
    }
}
fn decode_admitted_seed(
    solution: SolutionId,
    row: pse_model::generated::runtime::canonical_result_seeds::Row,
    payload: SeedDocument,
    assessment: Arc<
        pse_columnar::Leased<pse_model::generated::runtime::candidate_assessments::Row>,
    >,
    owner: Arc<pse_columnar::AllocationLease>,
) -> Result<AdmittedSeed, WorkflowError> {
    let backend = NativeBackend::try_from(row.backend.as_str())
        .map_err(|error| contract(error.to_string()))?;
    let start = WarmStart {
        origin: None,
        compatibility: Compatibility {
            layout: ContentHash::parse_hex(&row.layout)
                .map_err(|error| contract(format!("seed layout: {error}")))?,
            profile: ContentHash::parse_hex(&row.profile)
                .map_err(|error| contract(format!("seed profile: {error}")))?,
            data: ContentHash::parse_hex(&row.data)
                .map_err(|error| contract(format!("seed data: {error}")))?,
            backend,
        },
        payload: warm_payload(payload.warm),
    };
    Ok(AdmittedSeed {
        solution,
        header: row,
        assessment,
        start: Arc::new(pse_columnar::Leased::new(Arc::new(start), owner.clone())),
        prediction: payload
            .prediction
            .map(|point| Arc::new(pse_columnar::Leased::new(Arc::new(point), owner.clone()))),
    })
}
fn bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|v| v.to_bits()).collect()
}
fn floats(values: Vec<u64>) -> Vec<f64> {
    values.into_iter().map(f64::from_bits).collect()
}
fn seed_payload(payload: &WarmPayload) -> SeedPayload {
    match payload {
        WarmPayload::Root(primal) => SeedPayload::Root {
            primal: bits(primal),
        },
        WarmPayload::Nlp {
            primal,
            bounds,
            rows,
            barrier,
            ..
        } => SeedPayload::Nlp {
            primal: bits(primal),
            bounds: bounds
                .as_ref()
                .map(|(lower, upper)| (bits(lower), bits(upper))),
            rows: rows.as_ref().map(|v| bits(v)),
            barrier: barrier.map(f64::to_bits),
        },
        WarmPayload::Highs {
            primal,
            dual,
            basis,
        } => SeedPayload::Highs {
            primal: primal.as_ref().map(|v| bits(v)),
            dual: dual
                .as_ref()
                .map(|(columns, rows)| (bits(columns), bits(rows))),
            basis: basis.as_ref().map(|b| (b.columns.clone(), b.rows.clone())),
        },
    }
}
fn warm_payload(payload: SeedPayload) -> WarmPayload {
    match payload {
        SeedPayload::Root { primal } => WarmPayload::Root(floats(primal)),
        SeedPayload::Nlp {
            primal,
            bounds,
            rows,
            barrier,
        } => WarmPayload::Nlp {
            primal: floats(primal),
            bounds: bounds.map(|(lower, upper)| (floats(lower), floats(upper))),
            rows: rows.map(floats),
            barrier: barrier.map(f64::from_bits),
            working: None,
        },
        SeedPayload::Highs {
            primal,
            dual,
            basis,
        } => WarmPayload::Highs {
            primal: primal.map(floats),
            dual: dual.map(|(columns, rows)| (floats(columns), floats(rows))),
            basis: basis.map(|(columns, rows)| pse_backend_native::solve::Basis { columns, rows }),
        },
    }
}
pub(in crate::workflow) fn seed_extent(payload: &WarmPayload) -> Result<usize, WorkflowError> {
    let count = match payload {
        WarmPayload::Root(primal) => primal.len(),
        WarmPayload::Nlp {
            primal,
            bounds,
            rows,
            ..
        } => primal
            .len()
            .checked_add(
                bounds
                    .as_ref()
                    .map_or(0, |(lower, upper)| lower.len() + upper.len()),
            )
            .and_then(|n| n.checked_add(rows.as_ref().map_or(0, Vec::len)))
            .ok_or_else(|| contract("portable seed extent"))?,
        WarmPayload::Highs {
            primal,
            dual,
            basis,
        } => primal
            .as_ref()
            .map_or(0, Vec::len)
            .checked_add(
                dual.as_ref()
                    .map_or(0, |(columns, rows)| columns.len() + rows.len()),
            )
            .and_then(|n| {
                n.checked_add(
                    basis
                        .as_ref()
                        .map_or(0, |basis| basis.columns.len() + basis.rows.len()),
                )
            })
            .ok_or_else(|| contract("portable seed extent"))?,
    };
    seed_values_extent(count)
}
pub(in crate::workflow) fn seed_values_extent(count: usize) -> Result<usize, WorkflowError> {
    // A portable u64 bit value has at most20 decimal digits plus its separator.
    // Keep the live8-byte bit vector and the JSON Vec's geometric capacity
    // (up to twice its written extent) together:64 bytes per numeric member
    // covers both, with fixed room for the envelope and initial128-byte writer.
    // The original n-f64 materialization has a separate grant below.
    count
        .checked_mul(64)
        .and_then(|n| n.checked_add(16 * 1024))
        .ok_or_else(|| contract("portable seed serialization extent"))
}
async fn store_seeds(
    operations: &Operations,
    fence: &AttemptFence,
    result: &RunResult,
    acknowledgments: &PublicationAcks,
    #[cfg(all(test, feature = "canonical-tests"))] export_gate: &OnceLock<FinishExportGate>,
) -> Result<(Vec<(usize, SolutionId)>, Arc<pse_columnar::AllocationLease>), WorkflowError> {
    let mut next = 0;
    let result = append_seeds(
        operations,
        fence,
        result,
        acknowledgments,
        &mut next,
        #[cfg(all(test, feature = "canonical-tests"))]
        export_gate,
    )
    .await?;
    acknowledgments.complete_present(fence, "__seeds")?;
    Ok(result)
}
async fn append_seeds(
    operations: &Operations,
    fence: &AttemptFence,
    result: &RunResult,
    acknowledgments: &PublicationAcks,
    next: &mut u64,
    #[cfg(all(test, feature = "canonical-tests"))] export_gate: &OnceLock<FinishExportGate>,
) -> Result<(Vec<(usize, SolutionId)>, Arc<pse_columnar::AllocationLease>), WorkflowError> {
    let reservation =
        MemoryConsumer::new("canonical:acknowledged-seed-identities").register(&operations.pool);
    let (Ok(RunReport::Modeling(steps)), RunRequest::Modeling(requests)) =
        (result.report(), result.request())
    else {
        return Ok((Vec::new(), pse_columnar::AllocationLease::new(reservation)));
    };
    reservation
        .try_grow(
            steps
                .len()
                .checked_mul(size_of::<(usize, SolutionId)>())
                .ok_or_else(|| contract("seed identity extent overflow"))?,
        )
        .map_err(pse_engine::EngineError::from)?;
    let owner = pse_columnar::AllocationLease::new(reservation);
    let mut stored = Vec::with_capacity(steps.len());
    let run = operations
        .store
        .canonical_run(fence.run())
        .await?
        .ok_or_else(|| contract("seed run absent"))?;
    for (index, (step, request)) in steps.iter().zip(requests).enumerate() {
        let (Some(seed), Some(preparation)) = (
            super::study_execution::completed_modeling_seed(step),
            request.solve.seed_preparation_identity(),
        ) else {
            continue;
        };
        // Retain the original native producer's serialization bound. Complete
        // structural evaluations currently issue no portable prediction product.
        let prediction_extent = match &step.outcome {
            crate::math::solves::Outcome::Native(native) => native
                .candidate
                .as_ref()
                .map_or(0, |candidate| candidate.primal.len())
                .checked_add(native.variables.len())
                .and_then(|n| {
                    n.checked_add(
                        native
                            .observation
                            .as_ref()
                            .map_or(0, |observation| observation.values.len()),
                    )
                })
                .and_then(|n| n.checked_mul(32))
                .ok_or_else(|| contract("portable prediction extent"))?,
            crate::math::solves::Outcome::Constant(_) => 0,
            crate::math::solves::Outcome::Rejected(_) => continue,
        };
        let materialization_extent = match &step.outcome {
            crate::math::solves::Outcome::Constant(_) => seed.owned_extent()?,
            // The native seed is borrowed from its independently retained report.
            crate::math::solves::Outcome::Native(_) | crate::math::solves::Outcome::Rejected(_) => {
                0
            }
        };
        let extent = seed
            .serialization_extent()?
            .checked_add(prediction_extent)
            .and_then(|n| n.checked_add(materialization_extent))
            .ok_or_else(|| contract("portable seed document extent"))?;
        let serialization = reserve(&operations.pool, extent)?;
        let seed = seed.materialize(serialization.clone())?;
        let prediction = step.portable_prediction()?;
        if matches!(&step.outcome, crate::math::solves::Outcome::Constant(_))
            && prediction.is_some()
        {
            return Err(contract(
                "complete original seed has an unbounded prediction product",
            ));
        }
        let solution: SolutionId = pse_operations::mint_id();
        let name = "__seeds";
        let bytes = serde_json::to_vec(&SeedDocument {
            version: 1,
            warm: seed_payload(&seed.warm().payload),
            prediction,
        })
        .map_err(|e| contract(e.to_string()))?;
        let receipt = write_chunks(
            &operations.store,
            fence,
            name,
            *next,
            &bytes,
            acknowledgments,
            serialization,
        )
        .await?;
        let set = result_set_key(fence.attempt(), name);
        let row = pse_model::generated::runtime::canonical_result_seeds::Row {
            key: solution.to_string(),
            batch: result_batch_key(fence.attempt(), &set, *next),
            first_ordinal: *next,
            result_set: set,
            attempt: fence.attempt().into(),
            run: fence.run().into(),
            layout: seed.warm().compatibility.layout.to_string(),
            preparation: preparation.to_string(),
            profile: seed.warm().compatibility.profile.to_string(),
            data: seed.warm().compatibility.data.to_string(),
            backend: seed.warm().compatibility.backend.as_str().into(),
            step: result.modeling_step(index) as u64,
            batch_count: receipt.batch_count,
            payload_bytes: receipt.payload_bytes,
            digest: receipt.digest,
            run_sequence: run.sequence,
            attempt_generation: fence.generation(),
        };
        #[cfg(all(test, feature = "canonical-tests"))]
        FinishExportGate::at(export_gate, FinishExportPhase::Seed).await?;
        let acknowledged = operations
            .store
            .register_result_seed(fence, &format!("seed:{}:{solution}", fence.attempt()), &row)
            .await?;
        acknowledgments.seed(fence, acknowledged)?;
        stored.push((result.modeling_step(index), solution));
        *next += receipt.batch_count;
    }
    Ok((stored, owner))
}
impl Operations {
    async fn admit_seed(
        &self,
        solution: SolutionId,
        row: pse_model::generated::runtime::canonical_result_seeds::Row,
    ) -> Result<AdmittedSeed, WorkflowError> {
        self.load_seed(solution, row).await.map(|(seed, _, _)| seed)
    }
    async fn load_seed(
        &self,
        solution: SolutionId,
        row: pse_model::generated::runtime::canonical_result_seeds::Row,
    ) -> Result<(AdmittedSeed, Arc<ResultRead>, SeedCapsule), WorkflowError> {
        if row.key != solution.to_string() {
            return Err(contract("stored seed identity differs from selection"));
        }
        let response = pooled_response_owner(
            &self.pool,
            "canonical:seed-selection-response",
            pse_operations::canonical::MetadataResponse::Selection,
        )?;
        let read = Arc::new(
            pse_operations::canonical::response_ownership_scope(
                response.clone(),
                self.store.read_results(&row.run, &row.attempt),
            )
            .await?,
        );
        let read_owner = Arc::new(retain_protected_metadata(
            &self.pool,
            response,
            read.retained_bytes()?,
            |owner| read.retain_metadata_owner(owner),
        )?);
        read.retain_metadata_owner(read_owner)?;
        if read.attempt().seed_manifest.as_deref() != Some(read.manifest().key.as_str())
            || row.attempt_generation != read.attempt().generation
            || row.run_sequence != read.run().sequence
        {
            return Err(contract(
                "stored seed is outside the admitted original inventory",
            ));
        }
        let completed = pse_operations::canonical::response_ownership_scope(
            read.clone(),
            self.decode_completion(CompletionSelection::Full(&read)),
        )
        .await?;
        let assessment = completed
            .completion
            .as_ref()
            .and_then(|c| c.assessments.get(row.step as usize))
            .ok_or_else(|| contract("stored seed scientific completion absent"))?;
        if !assessment.permits_seed {
            return Err(contract(
                "stored seed scientific completion did not permit reuse",
            ));
        }
        let name = "__seeds";
        if result_set_key(&row.attempt, name) != row.result_set
            || row.batch != result_batch_key(&row.attempt, &row.result_set, row.first_ordinal)
        {
            return Err(contract("stored seed set coordinates differ"));
        }
        let (bytes, owner) = pse_operations::canonical::response_ownership_scope(
            read.clone(),
            read_chunks(
                &self.store,
                &self.pool,
                &read,
                name,
                row.first_ordinal,
                row.batch_count,
                row.payload_bytes,
                &row.digest,
                true,
            ),
        )
        .await?;
        let payload: SeedDocument =
            serde_json::from_slice(&bytes).map_err(|error| contract(error.to_string()))?;
        if payload.version != 1 {
            return Err(contract("stored seed interpretation differs"));
        }
        if let Some(point) = &payload.prediction
            && (!assessment.permits_result
                || point.permission.usability != assessment.usability
                || !point.permission.permits_use())
        {
            return Err(contract(
                "retained prediction differs from admitted scientific permission",
            ));
        }
        use pse_model::HeapUsage;
        let assessment_owner = reserve(
            &self.pool,
            assessment
                .heap_bytes()
                .checked_add(size_of_val(assessment))
                .ok_or_else(|| contract("seed assessment extent"))?,
        )?;
        let assessment = Arc::new(pse_columnar::Leased::new(
            Arc::new(assessment.clone()),
            assessment_owner,
        ));
        let capsule = SeedCapsule {
            version: 1,
            header: row.clone(),
            assessment: (**assessment).clone(),
            document: payload.clone(),
            manifest: read.manifest().clone(),
        };
        let seed = decode_admitted_seed(solution, row, payload, assessment, owner)?;
        Ok((seed, read, capsule))
    }
    /// Promote one freshly qualified original seed to an independent copied capsule.
    pub async fn promote_seed(
        &self,
        solution: SolutionId,
        owner_kind: &str,
        owner: &str,
    ) -> Result<pse_operations::canonical_artifacts::Artifact, WorkflowError> {
        let activity = self.store.admit_activity()?;
        activity
            .scope(async {
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:seed-header-response",
                    pse_operations::canonical::MetadataResponse::Header,
                )?;
                let row = pse_operations::canonical::response_ownership_scope(
                    response,
                    self.store.result_seed(&solution.to_string()),
                )
                .await?
                .ok_or(SeedReadError::Missing { solution })?;
                let (_admitted, read, capsule) = self.load_seed(solution, row).await?;
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:seed-promotion-sources-response",
                    pse_operations::canonical::MetadataResponse::Selection,
                )?;
                let sources = pse_operations::canonical::response_ownership_scope(
                    Arc::new((read.clone(), response)),
                    self.store.analysis_run_sources(&capsule.header.run),
                )
                .await?;
                let revision = sources
                    .iter()
                    .find(|source| source.key == read.run().revision)
                    .ok_or_else(|| contract("seed artifact primary source absent"))?;
                let payload =
                    serde_json::to_vec(&capsule).map_err(|error| contract(error.to_string()))?;
                let provenance = serde_json::to_vec(&(1_u8, &capsule.header, &capsule.manifest))
                    .map_err(|error| contract(error.to_string()))?;
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:seed-promotion-header-response",
                    pse_operations::canonical::MetadataResponse::Header,
                )?;
                let mut artifact = pse_operations::canonical::response_ownership_scope(
                    Arc::new((read.clone(), response)),
                    self.store
                        .new_artifact(revision, "seed", provenance, 1, payload.len() as u64),
                )
                .await?;
                let block = pse_operations::canonical_artifacts::artifact_block(
                    &artifact.key,
                    0,
                    "seed-capsule:v1",
                    payload,
                    &capsule.header.digest,
                    1,
                );
                pse_operations::canonical_artifacts::seal_artifact_request(
                    &mut artifact,
                    &sources,
                    std::slice::from_ref(read.as_ref()),
                    std::slice::from_ref(&block),
                    owner_kind,
                    owner,
                );
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:seed-promotion-publication-response",
                    pse_operations::canonical::MetadataResponse::Selection,
                )?;
                Ok(pse_operations::canonical::response_ownership_scope(
                    Arc::new((read.clone(), response)),
                    self.store.persist_artifact(
                        &artifact,
                        &sources,
                        std::slice::from_ref(read.as_ref()),
                        std::slice::from_ref(&block),
                        owner_kind,
                        owner,
                    ),
                )
                .await?)
            })
            .await
    }
    /// Acquire a retained independent seed. Producer result metadata is never consulted.
    pub async fn artifact_seed(&self, artifact: &str) -> Result<AdmittedSeed, WorkflowError> {
        let activity = self.store.admit_activity()?;
        activity
            .scope(async {
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:seed-artifact-response",
                    pse_operations::canonical::MetadataResponse::Header,
                )?;
                let read = pse_operations::canonical::response_ownership_scope(
                    response.clone(),
                    self.store.read_artifact(artifact),
                )
                .await?;
                let read_owner = Arc::new(retain_protected_metadata(
                    &self.pool,
                    response,
                    read.retained_bytes()?,
                    |owner| read.retain_metadata_owner(owner),
                )?);
                read.retain_metadata_owner(read_owner.clone())?;
                if read.header().kind != "seed" || read.header().block_count != 1 {
                    return Err(contract("artifact is not a seed capsule"));
                }
                let file_owner = Arc::new(
                    MemoryConsumer::new("canonical:seed-artifact-file").register(&self.pool),
                );
                file_owner
                    .try_grow(4 * super::result_files::MAX_FILE_BYTES)
                    .map_err(pse_engine::EngineError::from)?;
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:seed-artifact-block-response",
                    pse_operations::canonical::MetadataResponse::Header,
                )?;
                let block = pse_operations::canonical::response_ownership_scope(
                    Arc::new((read_owner, file_owner, response)),
                    read.block(0),
                )
                .await?;
                let owner = reserve(
                    &self.pool,
                    block
                        .payload
                        .len()
                        .checked_mul(4)
                        .ok_or_else(|| contract("seed capsule decode extent"))?,
                )?;
                let capsule: SeedCapsule = serde_json::from_slice(&block.payload)
                    .map_err(|error| contract(error.to_string()))?;
                if capsule.version != 1
                    || capsule.document.version != 1
                    || !capsule.assessment.permits_seed
                    || capsule.header.digest != block.original_digest
                    || capsule.manifest.attempt != capsule.header.attempt
                    || capsule.manifest.generation != capsule.header.attempt_generation
                {
                    return Err(contract("retained seed scientific witness differs"));
                }
                if let Some(point) = &capsule.document.prediction
                    && (!capsule.assessment.permits_result
                        || point.permission.usability != capsule.assessment.usability
                        || !point.permission.permits_use())
                {
                    return Err(contract("retained prediction permission differs"));
                }
                let solution = SemanticId::parse_hex(&capsule.header.key)
                    .map(SolutionId::from_id)
                    .map_err(|error| contract(error.to_string()))?;
                let assessment = Arc::new(pse_columnar::Leased::new(
                    Arc::new(capsule.assessment),
                    owner.clone(),
                ));
                read.release().await?;
                decode_admitted_seed(
                    solution,
                    capsule.header,
                    capsule.document,
                    assessment,
                    owner,
                )
            })
            .await
    }
    /// Acquire one explicitly selected original seed and its optional prediction once.
    pub async fn seed(&self, solution: SolutionId) -> Result<AdmittedSeed, WorkflowError> {
        let activity = self.store().admit_activity()?;
        activity
            .scope(async {
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:seed-header-response",
                    pse_operations::canonical::MetadataResponse::Header,
                )?;
                let row = pse_operations::canonical::response_ownership_scope(
                    response,
                    self.store.result_seed(&solution.to_string()),
                )
                .await?
                .ok_or(SeedReadError::Missing { solution })?;
                self.admit_seed(solution, row).await
            })
            .await
    }
    /// Bounded eligible index pages use recorded order and then exact scientific admission.
    pub async fn latest_seed(
        &self,
        target: &Compatibility,
        preparation: &ContentHash,
        attempt: Option<&str>,
    ) -> Result<Option<AdmittedSeed>, WorkflowError> {
        let activity = self.store().admit_activity()?;
        activity
            .scope(async {
                let response = pooled_response_owner(
                    &self.pool,
                    "canonical:seed-candidates-response",
                    pse_operations::canonical::MetadataResponse::Page,
                )?;
                let rows = pse_operations::canonical::response_ownership_scope(
                    response,
                    self.store.result_seed_candidates(
                        &target.layout.to_string(),
                        &preparation.to_string(),
                        target.backend.as_str(),
                        attempt,
                    ),
                )
                .await?;
                let Some(row) = rows.into_iter().next() else {
                    return Ok(None);
                };
                let solution: SolutionId = SemanticId::parse_hex(&row.key)
                    .map(SolutionId::from_id)
                    .map_err(|e| contract(format!("seed identity: {e}")))?;
                self.admit_seed(solution, row).await.map(Some)
            })
            .await
    }
}

/// One exact source object, whose kind separates worker documents from typed physical rows.
pub(super) fn source_edit(
    logical: String,
    scope: String,
    name: String,
    kind: &str,
    bytes: Vec<u8>,
) -> pse_operations::canonical::ObjectEdit {
    let mut hash = FramedHasher::new(Frame::CanonicalSourceObjectV1);
    hash.str(pse_operations::generated::surreal::INTERPRETATION)
        .str(kind)
        .str(&logical)
        .part(&bytes)
        .u64(0);
    let key = hash.finish_hash().to_string();
    pse_operations::canonical::ObjectEdit {
        logical: logical.clone(),
        scope,
        name,
        version: Some(pse_model::generated::runtime::canonical_versions::Row {
            key,
            logical,
            kind: kind.into(),
            payload: bytes.into(),
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        }),
        references: Vec::new(),
    }
}
// Keep each physical edit's construction temporaries out of the publication
// poll frame. Admission precedes construction; out-of-line construction and
// polling preserve ordinary executor stacks without changing task ownership.
async fn physical_publication_action<T, F, Fut>(
    pool: &Arc<dyn MemoryPool>,
    action: F,
) -> Result<T, WorkflowError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, WorkflowError>>,
{
    fn action_bytes<F, Fut>(_: &F) -> usize
    where
        F: FnOnce() -> Fut,
        Fut: Future,
    {
        size_of::<Fut>()
    }
    let owner = MemoryConsumer::new("physical:publication-action").register(pool);
    owner
        .try_grow(action_bytes(&action))
        .map_err(pse_engine::EngineError::from)?;
    let mut work = super::worker::boxed_worker_action(action);
    futures_util::future::poll_fn(|context| {
        super::worker::poll_worker_action(work.as_mut(), context)
    })
    .await
}

impl Operations {
    async fn publish_physical_group(
        &self,
        relation: pse_schema::model::RelationKey,
        identity: ContentHash,
        parent: Option<&Revision>,
        group: &mut super::physical_spool::PhysicalGroup,
    ) -> Result<Revision, WorkflowError> {
        let problem = format!("physical:rows:grouped-v2:{identity}");
        let operation = format!(
            "physical:grouped-v2:{identity}:{:020}:{:020}",
            group.first, group.end
        );
        let edits = group
            .blocks
            .drain(..)
            .enumerate()
            .map(|(offset, payload)| {
                let ordinal = group.first + offset as u64;
                source_edit(
                    format!("rows:{ordinal:020}"),
                    relation.to_string(),
                    format!("{ordinal:020}"),
                    "physical:rows:v1",
                    payload,
                )
            })
            .collect::<Vec<_>>();
        // edit binds the complete original request and settles unknown activation
        // ACKs using its immutable operation receipt. No guessed parent advances.
        physical_publication_action(&self.pool, || async {
            Ok(pse_operations::canonical::response_ownership_scope(
                group.owner.clone(),
                self.store.edit(
                    &problem,
                    parent.map(|row| row.key.as_str()),
                    &operation,
                    &edits,
                ),
            )
            .await?)
        })
        .await
    }
    async fn publish_physical_groups(
        &self,
        relation: pse_schema::model::RelationKey,
        identity: ContentHash,
        reader: super::physical_spool::PhysicalSpoolReader,
    ) -> Result<Option<Revision>, WorkflowError> {
        let mut groups = super::physical_spool::PhysicalGroups::new(reader, &self.pool)?;
        let mut saved = None;
        while let Some(mut group) = groups.next().await? {
            saved = Some(
                self.publish_physical_group(relation, identity, saved.as_ref(), &mut group)
                    .await?,
            );
        }
        Ok(saved)
    }
    /// Persist each registry physical relation independently; each immutable IPC object
    /// is bounded, and no unrelated relation is hydrated on replay.
    pub(super) async fn put_physical_rows(
        &self,
        runtime: &super::Runtime,
        physical: &super::PhysicalContext,
    ) -> Result<super::physical_cache::PhysicalRows, WorkflowError> {
        let activity = self.store().admit_activity()?;
        activity
            .scope(async {
                let generation = runtime.physical_cache.rows_generation();
                if let Some(rows) = runtime.physical_cache.rows(runtime, self, physical) {
                    return runtime
                        .physical_cache
                        .protected_rows(generation, runtime, self, physical, rows.revisions())
                        .await;
                }
                let generation = runtime.physical_cache.begin_rows();
                let encoding =
                    MemoryConsumer::new("canonical:physical-source-encoding").register(&self.pool);
                // Even an empty source map publishes its context receipt.
                encoding
                    .try_grow(16 * pse_operations::canonical::PAYLOAD_BYTES)
                    .map_err(pse_engine::EngineError::from)?;
                let encoding = pse_columnar::AllocationLease::new(encoding);
                let mut revisions = Vec::new();
                for (relation, table) in &physical.sources {
                    let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
                    hash.str("physical.rows.v1").str(&relation.to_string());
                    let mut spool = super::physical_spool::PhysicalSpool::new(
                        runtime.shared.runtime_env(),
                        pse_operations::canonical::PAYLOAD_BYTES,
                    )?;
                    super::result_blocks::visit_source_blocks_async(
                        table.batch(),
                        |_, _, payload| {
                            hash.part(&payload);
                            std::future::ready(spool.push(&payload))
                        },
                    )
                    .await?;
                    let identity = hash.finish_hash();
                    let saved = pse_operations::canonical::response_ownership_scope(
                        encoding.clone(),
                        self.publish_physical_groups(*relation, identity, spool.finish()?),
                    )
                    .await?;
                    if let Some(revision) = saved {
                        revisions.push(revision);
                    }
                }
                // Even built-in contexts carry their exact compiler interpretation receipt.
                let bytes = serde_json::to_vec(&(1_u8, physical.identity()))
                    .map_err(|e| contract(e.to_string()))?;
                let edit = source_edit(
                    "physical:context".into(),
                    "physical".into(),
                    "context".into(),
                    "physical:context:v1",
                    bytes,
                );
                let problem = format!("physical:context:{}", physical.identity());
                revisions.push(
                    physical_publication_action(&self.pool, || async {
                        Ok(pse_operations::canonical::response_ownership_scope(
                            encoding.clone(),
                            self.store.edit(
                                &problem,
                                None,
                                &format!("physical:context:{}", physical.identity()),
                                &[edit],
                            ),
                        )
                        .await?)
                    })
                    .await?,
                );
                let result = runtime
                    .physical_cache
                    .protect_rows(self, &revisions)
                    .await?;
                runtime
                    .physical_cache
                    .retain_rows(generation, runtime, self, physical, &result);
                Ok(result)
            })
            .await
    }
}

/// Count borrowed serialization without a temporary completion tree or payload.
struct CompletionExtent(usize);
impl std::io::Write for CompletionExtent {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self.0.checked_add(bytes.len()).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::OutOfMemory,
                "completion extent overflow",
            )
        })?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn copy_original_completion<T: Serialize + Clone, U: Serialize>(
    pool: &Arc<dyn MemoryPool>,
    original: Option<&T>,
    context: &U,
) -> Result<(Option<T>, Arc<pse_columnar::AllocationLease>), WorkflowError> {
    let owner = completion_copy_owner(pool, &(original, context))?;
    Ok((original.cloned(), owner))
}
pub(super) fn completion_retention_owner<T: Serialize>(
    pool: &Arc<dyn MemoryPool>,
    original: &T,
) -> Result<Arc<pse_columnar::AllocationLease>, WorkflowError> {
    let mut extent = CompletionExtent(0);
    serde_json::to_writer(&mut extent, original).map_err(|error| contract(error.to_string()))?;
    let bytes = extent
        .0
        .checked_mul(16)
        .and_then(|n| n.checked_add(1024))
        .ok_or_else(|| contract("completion retention extent"))?;
    let reservation = MemoryConsumer::new("study:original-summary").register(pool);
    reservation
        .try_grow(bytes)
        .map_err(pse_engine::EngineError::from)?;
    Ok(pse_columnar::AllocationLease::new(reservation))
}
fn completion_copy_owner<T: Serialize>(
    pool: &Arc<dyn MemoryPool>,
    original: &T,
) -> Result<Arc<pse_columnar::AllocationLease>, WorkflowError> {
    let mut extent = CompletionExtent(0);
    serde_json::to_writer(&mut extent, original).map_err(|error| contract(error.to_string()))?;
    // Field names and the owned completion envelope are additional to the exact
    // borrowed payload count. reserve includes owned tree and encoding scratch.
    reserve(
        pool,
        extent
            .0
            .checked_add(1024)
            .ok_or_else(|| contract("completion copy extent overflow"))?,
    )
}

fn reserve(
    pool: &Arc<dyn MemoryPool>,
    bytes: usize,
) -> Result<Arc<pse_columnar::AllocationLease>, WorkflowError> {
    let extent = bytes
        .checked_mul(16)
        .and_then(|n| n.checked_add(2 * bytes.min(RESULT_BATCH_BYTES)))
        .and_then(|n| n.checked_add(1024))
        .ok_or_else(|| contract("scientific decode allocation extent overflow"))?;
    let reservation = MemoryConsumer::new("canonical:scientific-decode").register(pool);
    reservation
        .try_grow(extent)
        .map_err(pse_engine::EngineError::from)?;
    Ok(pse_columnar::AllocationLease::new(reservation))
}

#[cfg(all(test, feature = "canonical-tests"))]
#[path = "physical-publication-tests.rs"]
mod physical_publication_unit;

#[cfg(test)]
mod physical_publication_action_tests {
    use super::*;

    struct Action {
        cancel: crate::CancelSource,
        dropped: Arc<AtomicU64>,
        _payload: [u8; 1024],
    }
    impl Future for Action {
        type Output = Result<(), WorkflowError>;
        fn poll(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            if self.cancel.token().is_cancelled() {
                std::task::Poll::Ready(Ok(()))
            } else {
                std::task::Poll::Pending
            }
        }
    }
    impl Drop for Action {
        fn drop(&mut self) {
            self.dropped.fetch_add(1, Ordering::AcqRel);
        }
    }

    #[tokio::test]
    async fn physical_publication_action_admits_before_construction_and_keeps_exact_owner_until_drop()
     {
        let extent = size_of::<Action>();
        let entered = Arc::new(AtomicU64::new(0));
        let dropped = Arc::new(AtomicU64::new(0));
        let tiny: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(extent - 1));
        let refused = physical_publication_action(&tiny, || {
            entered.fetch_add(1, Ordering::AcqRel);
            Action {
                cancel: crate::CancelSource::new(),
                dropped: dropped.clone(),
                _payload: [0; 1024],
            }
        })
        .await;
        assert!(refused.is_err());
        assert_eq!(entered.load(Ordering::Acquire), 0);
        assert_eq!(dropped.load(Ordering::Acquire), 0);
        assert_eq!(tiny.reserved(), 0);

        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(extent));
        for cancelled in [false, true] {
            let cancel = crate::CancelSource::new();
            let dropped = Arc::new(AtomicU64::new(0));
            let mut work = Box::pin(physical_publication_action(&pool, || {
                assert_eq!(
                    pool.reserved(),
                    extent,
                    "exact future admission precedes construction"
                );
                Action {
                    cancel: cancel.clone(),
                    dropped: dropped.clone(),
                    _payload: [0; 1024],
                }
            }));
            assert!(futures_util::poll!(work.as_mut()).is_pending());
            assert_eq!(pool.reserved(), extent);
            if cancelled {
                cancel.cancel();
                assert!(
                    work.await.is_ok(),
                    "the original phase observes its original cancellation"
                );
            } else {
                drop(work);
            }
            assert_eq!(dropped.load(Ordering::Acquire), 1);
            assert_eq!(pool.reserved(), 0);
        }
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_durable_codec {
    #![allow(
        clippy::unwrap_used,
        clippy::unreachable,
        reason = "canonical codec fixtures require durable setup and the exact root payload variant under test"
    )]
    use super::*;
    use pse_backend_native::solve::Metric;
    use std::collections::BTreeMap;

    async fn claimed() -> (Operations, AttemptFence) {
        let runtime = super::super::durable_tests::durable_runtime();
        let Durability::Durable(operations) = runtime.durability() else {
            unreachable!()
        };
        let operations = operations.as_ref().clone();
        let revision = operations
            .store
            .edit("codec-fixture", None, "source-fixture", &[])
            .await
            .unwrap();
        operations
            .store
            .begin_run(&pse_operations::canonical_execution::RunRequest {
                key: "codec-run".into(),
                revision,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = operations
            .store
            .claim_run(
                "codec-run",
                "codec-claim",
                "codec-worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        (operations, fence)
    }
    #[test]
    fn durable_record_aliases_retain_one_actual_metadata_allocation() {
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(64 * 1024));
        let owner = durable_metadata_owner(&pool, None, None, None, 0, 0).unwrap();
        let charged = pool.reserved();
        let record = Arc::new(DurableRecord {
            attempt_id: pse_operations::mint_id(),
            attempt_key: None,
            run: None,
            attempt: Err(Arc::new(contract("observable failure"))),
            manifest: None,
            completion: None,
            solutions: Vec::new(),
            _solutions_owner: None,
            _metadata_owner: Some(owner),
        });
        let original = RunDurability::Durable(record.clone());
        let alias = original.clone();
        assert_eq!(pool.reserved(), charged);
        let RunDurability::Durable(alias_record) = &alias else {
            unreachable!()
        };
        assert!(Arc::ptr_eq(&record, alias_record));
        drop(record);
        drop(original);
        assert_eq!(pool.reserved(), charged);
        drop(alias);
        assert_eq!(pool.reserved(), 0);
    }

    #[tokio::test]
    async fn canonical_standalone_read_entrypoints_refuse_unprepaid_metadata() {
        let runtime = super::super::durable_tests::durable_runtime();
        let store = runtime
            .canonical_store()
            .unwrap()
            .independent_caller()
            .unwrap();
        // No Python recording_operation, Runtime activity scope, or inherited
        // response grant can cover these independently constructed Operations.
        runtime.close();
        drop(runtime);
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(
            pse_operations::canonical::MetadataResponse::Header
                .allocation_bytes()
                .min(pse_operations::canonical::MetadataResponse::Page.allocation_bytes())
                - 1,
        ));
        let operations = Operations::from_store(
            store,
            "standalone-read-control",
            LeasePolicy::default(),
            pool.clone(),
        );
        fn refused<T: std::fmt::Debug>(result: Result<T, WorkflowError>) {
            assert!(
                matches!(result, Err(WorkflowError::Engine(_))),
                "metadata must refuse before missing-row or empty-page RPC: {result:?}"
            );
        }
        let solution = pse_operations::mint_id::<SolutionId>();
        let identity = ContentHash::from_bytes([1; 32]);
        let target = Compatibility {
            layout: identity,
            profile: identity,
            data: identity,
            backend: NativeBackend::Kinsol,
        };
        refused(operations.runs("unissued-problem", None, 1).await);
        refused(operations.record("unissued-run", "unissued-attempt").await);
        refused(
            operations
                .completion("unissued-run", "unissued-attempt")
                .await,
        );
        refused(operations.seed(solution).await);
        refused(operations.latest_seed(&target, &identity, None).await);
        refused(
            operations
                .promote_seed(solution, "user", "unissued-owner")
                .await,
        );
        refused(
            operations
                .recover("unissued-run", "unissued-recovery")
                .await,
        );
        // A previously selected header exercises the distinct load_seed entry,
        // without claiming that these deliberately absent coordinates are science.
        let row = pse_model::generated::runtime::canonical_result_seeds::Row {
            key: solution.to_string(),
            batch: "unissued-batch".into(),
            first_ordinal: 0,
            result_set: "unissued-set".into(),
            attempt: "unissued-attempt".into(),
            run: "unissued-run".into(),
            layout: identity.to_string(),
            preparation: identity.to_string(),
            profile: identity.to_string(),
            data: identity.to_string(),
            backend: "kinsol".into(),
            step: 0,
            batch_count: 1,
            payload_bytes: 1,
            digest: identity.to_string(),
            run_sequence: 1,
            attempt_generation: 1,
        };
        refused(operations.load_seed(solution, row).await);
        assert_eq!(pool.reserved(), 0, "all refused grants must roll back");
    }

    #[tokio::test]
    async fn canonical_acknowledged_finish_returns_original_without_protected_reopen() {
        let (operations, fence) = claimed().await;
        let attempt_id = pse_operations::mint_id();
        let mut attempt = DurableAttempt::claimed(
            &operations,
            fence.clone(),
            pse_operations::mint_id(),
            attempt_id,
        )
        .unwrap();
        attempt.stream.finish().await;
        let detail = failure(&Arc::new(contract("synthetic partial original")), false).detail;
        let stored = StoredCompletion {
            version: 2,
            attempt_id,
            completion: Some(super::super::Completion::default()),
            termination: detail,
            state: AttemptState::Partial,
            progress: super::super::ProgressCoverage::Sampled {
                observed: 0,
                queued: 0,
                dropped_before_queue: 0,
                acknowledged: 0,
                unacknowledged: 0,
                writer: super::super::ProgressWriterOutcome::Complete,
            },
        };
        let (reached, waiting) = tokio::sync::oneshot::channel();
        let (release, resume) = tokio::sync::oneshot::channel();
        attempt
            .stream
            .export_gate
            .set(FinishExportGate {
                phase: FinishExportPhase::AcknowledgedSeal,
                reached: Mutex::new(Some(reached)),
                release: Mutex::new(Some(resume)),
                failure: None,
            })
            .unwrap();
        let finished = tokio::spawn(async move { attempt.terminate(stored).await });
        waiting.await.unwrap();
        // The actual seal already committed. Explicit fixture retirement removes
        // permission for any protected result reopen before the local return.
        operations
            .store
            .forget_run_results(fence.run())
            .await
            .unwrap();
        assert!(
            operations
                .record(fence.run(), fence.attempt())
                .await
                .is_err()
        );
        release.send(()).unwrap();
        let record = finished.await.unwrap().unwrap();
        assert_eq!(record.attempt_id, attempt_id);
        assert_eq!(
            record.completion.as_ref().unwrap().state,
            AttemptState::Partial
        );
        assert!(record.completion.as_ref().unwrap().completion.is_some());
        assert_eq!(
            record.run.as_ref().unwrap().terminal_attempt.as_deref(),
            Some(fence.attempt())
        );
        assert_eq!(
            record.attempt.as_ref().unwrap().closed_manifest.as_deref(),
            record
                .manifest
                .as_ref()
                .map(|manifest| manifest.key.as_str())
        );
    }
    #[tokio::test]
    async fn canonical_progress_uncertainty_retains_original_sampled_counters() {
        use super::super::{
            ProgressCoverage, ProgressWriteStage, ProgressWriterFailure, ProgressWriterOutcome,
        };
        let (operations, fence) = claimed().await;
        let attempt_id = pse_operations::mint_id();
        let mut attempt = DurableAttempt::claimed(
            &operations,
            fence.clone(),
            pse_operations::mint_id(),
            attempt_id,
        )
        .unwrap();
        attempt.stream.finish().await;
        let event = Event {
            source: None,
            phase: "uncertain".into(),
            elapsed: Duration::ZERO,
            values: Default::default(),
            incumbent: None,
        };
        let mut document = super::super::ProgressEventDocument::from(&event);
        document.sequence = Some(0);
        let bytes = serde_json::to_vec(&[document]).unwrap();
        // Deliberately discard the actual producer ACK to model a committed
        // operation whose response was never observed by the original writer.
        operations
            .store
            .append_result_batch(
                &fence,
                "unobserved-progress",
                "__progress",
                0,
                pse_operations::canonical_execution::ResultPayload::new(bytes.to_vec()).unwrap(),
                1,
            )
            .await
            .unwrap();
        let progress = ProgressCoverage::Sampled {
            observed: 1,
            queued: 1,
            dropped_before_queue: 0,
            acknowledged: 0,
            unacknowledged: 1,
            writer: ProgressWriterOutcome::Failed(ProgressWriterFailure {
                stage: ProgressWriteStage::CanonicalAppend {},
            }),
        };
        let stored = StoredCompletion {
            version: 2,
            attempt_id,
            completion: None,
            termination: failure(&Arc::new(contract("original failed science")), false).detail,
            state: AttemptState::Failed,
            progress: progress.clone(),
        };
        let record = attempt.terminate(stored).await.unwrap();
        assert_eq!(record.completion.as_ref().unwrap().progress, progress);
        let descriptors = pse_operations::canonical_execution::decode_result_descriptors(
            record.manifest.as_ref().unwrap(),
        )
        .unwrap();
        assert_eq!(
            descriptors
                .iter()
                .find(|set| set.name == "__progress")
                .unwrap()
                .row_count,
            1
        );
        let reopened = operations
            .record(fence.run(), fence.attempt())
            .await
            .unwrap();
        assert_eq!(reopened.completion.as_ref().unwrap().progress, progress);
        assert_eq!(
            reopened.completion.as_ref().unwrap().state,
            AttemptState::Failed
        );
    }
    async fn study_claimed() -> (Operations, AttemptFence) {
        let (operations, claim) = study_claim().await;
        (operations, claim.fence)
    }
    async fn study_claim() -> (Operations, pse_operations::canonical_studies::StudyClaim) {
        let (runtime, claim) = study_claim_runtime().await;
        (runtime.operations().unwrap().clone(), claim)
    }
    async fn study_claim_runtime() -> (
        super::super::Runtime,
        pse_operations::canonical_studies::StudyClaim,
    ) {
        let runtime = super::super::durable_tests::durable_runtime();
        let claim = study_claim_on(&runtime, "cancel").await;
        (runtime, claim)
    }
    async fn study_claim_on(
        runtime: &super::super::Runtime,
        prefix: &str,
    ) -> pse_operations::canonical_studies::StudyClaim {
        use pse_model::study::{OccurrenceKey, PointPolicy, SeedNeed, StartPolicy};
        use pse_operations::canonical_studies::{NewOccurrence, point_key};
        let operations = runtime.operations().unwrap();
        let study = format!("{prefix}-study");
        let revision = operations
            .store
            .edit(
                &format!("{prefix}-fixture"),
                None,
                &format!("{prefix}-source"),
                &[],
            )
            .await
            .unwrap();
        let request = |key: &str| pse_operations::canonical_execution::RunRequest {
            key: key.into(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        };
        operations
            .store
            .create_study(
                &study,
                &request(&format!("{prefix}-summary")),
                &[9],
                &[NewOccurrence {
                    policy: PointPolicy {
                        key: OccurrenceKey(1),
                        dependencies: vec![],
                        start: StartPolicy::Fresh,
                        seed_need: SeedNeed::NotNeeded,
                        attempt_limit: 1,
                    },
                    descriptor: vec![9],
                    run: request(&format!("{prefix}-point")),
                }],
                &|| false,
            )
            .await
            .unwrap();
        let scope = operations
            .store
            .study_scope(&point_key(&study, OccurrenceKey(1)))
            .await
            .unwrap();
        operations
            .store
            .claim_study_point(
                &scope,
                None,
                &format!("{prefix}-claim"),
                "cancel-worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap()
    }
    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    #[allow(
        clippy::print_stderr,
        reason = "bounded I30 inquiry reports complete ordinary export and native durations, separately from deliberately gated publication"
    )]
    async fn canonical_joined_publication_progress_and_ordinary_cost_probe() {
        use super::super::durable_tests::{LINEAR, durable_runtime_on, package_on};
        let runtime = super::super::tests::runtime_with_workspace(32 << 20);
        let (package, analysis) = package_on(&runtime, LINEAR).await;
        let prepared = package
            .prepare_analysis(&analysis, &crate::CancelSource::new())
            .await
            .unwrap();
        let publication = durable_runtime_on(runtime.clone());
        let operations = publication.operations().unwrap().clone();
        let mut samples = Vec::new();
        let mut retained = None;
        for index in 0..5 {
            let started = std::time::Instant::now();
            let result = prepared.start().unwrap().wait().await.unwrap();
            let native_us = started.elapsed().as_micros();
            assert!(result.usable());
            let claim =
                study_claim_on(&publication, &format!("ordinary-publication-{index}")).await;
            operations.store.mark_study_started(&claim).await.unwrap();
            let mut attempt = DurableAttempt::claimed(
                &operations,
                claim.fence.clone(),
                result.run_id,
                pse_operations::mint_id(),
            )
            .unwrap();
            attempt.start(Arc::new(|| {})).await.unwrap();
            let started = std::time::Instant::now();
            let record = attempt.finish(&result, false).await;
            let export_us = started.elapsed().as_micros();
            assert!(record.attempt.as_ref().unwrap().terminal() && record.manifest.is_some());
            assert!(record.completion.as_ref().unwrap().completion.is_some());
            samples.push((native_us, export_us));
            retained = Some(result);
        }
        let result = retained.unwrap();
        for (index, phase) in [FinishExportPhase::Table, FinishExportPhase::Completion]
            .into_iter()
            .enumerate()
        {
            let claim = study_claim_on(&publication, &format!("gated-publication-{index}")).await;
            operations.store.mark_study_started(&claim).await.unwrap();
            let mut attempt = DurableAttempt::claimed(
                &operations,
                claim.fence.clone(),
                result.run_id,
                pse_operations::mint_id(),
            )
            .unwrap();
            let (reached, reached_rx) = tokio::sync::oneshot::channel();
            let (release, release_rx) = tokio::sync::oneshot::channel();
            attempt
                .stream
                .export_gate
                .set(FinishExportGate {
                    phase,
                    reached: Mutex::new(Some(reached)),
                    release: Mutex::new(Some(release_rx)),
                    failure: (index == 1).then(|| {
                        Arc::new(WorkflowError::Canonical(
                            pse_operations::canonical::CanonicalError::Timeout,
                        ))
                    }),
                })
                .unwrap();
            attempt.start(Arc::new(|| {})).await.unwrap();
            let finish = attempt.finish(&result, false);
            let control = async {
                let reached = matches!(
                    tokio::time::timeout(Duration::from_secs(10), reached_rx).await,
                    Ok(Ok(()))
                );
                // Exercise the same actual math/pool owner while export retains
                // its original source, heartbeat, fence and selected output.
                let sibling = if reached {
                    Some(match prepared.start() {
                        Err(error) => Err(error),
                        Ok(handle) => {
                            match tokio::time::timeout(Duration::from_secs(20), handle.wait()).await
                            {
                                Ok(result) => result,
                                Err(_) => {
                                    handle.cancel();
                                    let _ = handle.wait().await;
                                    Err(contract(
                                        "publication probe sibling budget elapsed after actual drain",
                                    ))
                                }
                            }
                        }
                    })
                } else {
                    None
                };
                let pending = operations
                    .store
                    .canonical_attempt(claim.fence.attempt())
                    .await;
                // Always release issued publication before asserting the control.
                let _ = release.send(());
                (reached, sibling, pending)
            };
            let (record, (reached, sibling, pending)) = tokio::join!(finish, control);
            assert!(reached);
            assert!(
                sibling.unwrap().unwrap().usable(),
                "useful native work can finish while another joined export waits"
            );
            assert!(
                !pending.unwrap().unwrap().terminal(),
                "a gated export is not an acknowledged terminal attempt"
            );
            if index == 0 {
                assert!(record.attempt.as_ref().unwrap().terminal());
            } else {
                assert!(record.attempt.is_err() && record.manifest.is_none());
            }
            assert!(
                result.usable(),
                "publication refusal does not replace original scientific completion"
            );
        }
        eprintln!(
            "I30 same shared math/pool, five ordinary KINSOL runs: native_wait_us/export_through_ACK_us={samples:?}; gated Table and failing Completion both permitted independent native completion before release; no separate commit stage was installed"
        );
    }

    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn canonical_finish_cancellation_at_each_export_boundary_keeps_exact_prefix_and_science()
    {
        use super::super::durable_tests::{LINEAR, package_on};
        use super::super::{RunDurability, tests};
        let runtime = tests::runtime_with_workspace(32 << 20);
        let (package, analysis) = package_on(&runtime, LINEAR).await;
        let prepared = package
            .prepare_analysis(&analysis, &crate::CancelSource::new())
            .await
            .unwrap();
        let handle = prepared.start().unwrap();
        // wait publishes only after the actual native owner has joined. Exporting
        // this immutable seed-bearing result cannot perform another native solve.
        let result = handle.wait().await.unwrap();
        assert!(matches!(result.durability(), RunDurability::Ephemeral));
        assert!(result.usable(), "{:?}", result.report());
        let RunReport::Modeling(steps) = result.report().unwrap() else {
            unreachable!()
        };
        assert!(matches!(
            steps[0].outcome,
            crate::math::solves::Outcome::Native(_)
        ));
        assert!(super::super::study_execution::completed_modeling_seed(&steps[0]).is_some());
        let RunRequest::Modeling(requests) = result.request() else {
            unreachable!()
        };
        assert!(requests[0].solve.seed_preparation_identity().is_some());
        let original = serde_json::to_value(result.completion().unwrap()).unwrap();
        assert!(result.completion().unwrap().assessments[0].permits_seed);
        let expected_tables = result
            .tables()
            .unwrap()
            .into_iter()
            .map(|(relation, table)| (relation.to_string(), table.batch().num_rows() as u64))
            .collect::<BTreeMap<_, _>>();

        // Each case has its own exact study/attempt identities in one installed
        // fixture, avoiding six repeated complete-schema installations.
        let point_runtime = super::super::durable_tests::durable_runtime();
        let operations = point_runtime.operations().unwrap().clone();
        for (index, (phase, unrelated_export, unrelated_heartbeat)) in [
            (FinishExportPhase::Progress, false, false),
            (FinishExportPhase::Table, false, false),
            (FinishExportPhase::Seed, false, false),
            (FinishExportPhase::Completion, false, false),
            (FinishExportPhase::Table, true, false),
            (FinishExportPhase::Table, false, true),
        ]
        .into_iter()
        .enumerate()
        {
            let claim = study_claim_on(&point_runtime, &format!("cancel-{index}")).await;
            let fence = claim.fence.clone();
            operations.store.mark_study_started(&claim).await.unwrap();
            operations
                .store
                .append_result_batch(
                    &fence,
                    &format!("accepted-prefix:{}", fence.attempt()),
                    "__prefix",
                    0,
                    pse_operations::canonical_execution::ResultPayload::new([7].to_vec()).unwrap(),
                    1,
                )
                .await
                .unwrap();
            let attempt_id = pse_operations::mint_id();
            let mut attempt =
                DurableAttempt::claimed(&operations, fence.clone(), result.run_id, attempt_id)
                    .unwrap();
            let (reached, reached_rx) = tokio::sync::oneshot::channel();
            let (release, release_rx) = tokio::sync::oneshot::channel();
            let unrelated = Arc::new(WorkflowError::Canonical(
                pse_operations::canonical::CanonicalError::Timeout,
            ));
            attempt
                .stream
                .export_gate
                .set(FinishExportGate {
                    phase,
                    reached: Mutex::new(Some(reached)),
                    release: Mutex::new(Some(release_rx)),
                    failure: unrelated_export.then(|| unrelated.clone()),
                })
                .unwrap();
            if unrelated_export || unrelated_heartbeat {
                // Compose both unrelated-error precedences through finish and the
                // final heartbeat join, after real acknowledged study cancellation.
                let (stop, stopped) = tokio::sync::oneshot::channel();
                let failure = Arc::new(Mutex::new(None));
                let recorded = failure.clone();
                let joined_operations = operations.clone();
                let joined_fence = fence.clone();
                let unrelated = unrelated.clone();
                let task = tokio::spawn(async move {
                    stopped.await.unwrap();
                    let refusal = joined_operations
                        .store
                        .renew_attempt(&joined_fence, Duration::from_secs(60))
                        .await
                        .unwrap_err();
                    assert!(canonical_cancellation(&refusal, &joined_fence));
                    *recorded.lock().unwrap() = Some(if unrelated_heartbeat {
                        unrelated
                    } else {
                        Arc::new(WorkflowError::Canonical(refusal))
                    });
                });
                attempt.heartbeat = Some(Heartbeat {
                    stop,
                    task,
                    failure,
                });
            } else {
                attempt.start(Arc::new(|| {})).await.unwrap();
            }
            attempt.stream.tap.observe(&Event {
                source: None,
                phase: "joined-export".into(),
                elapsed: Duration::ZERO,
                values: BTreeMap::from([("counter".into(), Metric::Integer(1))]),
                incumbent: None,
            });
            let joined = result.clone();
            let mut finishing = tokio::spawn(async move { attempt.finish(&joined, false).await });
            tokio::select! {
                reached = reached_rx => reached.unwrap(),
                early = &mut finishing => { unreachable!("finish returned before {phase:?}: {early:?}"); }
            }
            if phase == FinishExportPhase::Progress {
                // Establish an acknowledged baseline using the installed lease,
                // then observe the real independent heartbeat extend it while
                // the progress writer remains blocked. No test clock is advanced.
                let baseline = operations
                    .store
                    .renew_attempt(&fence, operations.policy.lease)
                    .await
                    .unwrap();
                let deadline = tokio::time::Instant::now() + operations.policy.lease;
                loop {
                    let renewed = operations
                        .store
                        .canonical_attempt(fence.attempt())
                        .await
                        .unwrap()
                        .unwrap();
                    if renewed.expires_at > baseline.expires_at {
                        break;
                    }
                    assert!(
                        tokio::time::Instant::now() < deadline,
                        "heartbeat did not extend its original lease while export was gated"
                    );
                    tokio::task::yield_now().await;
                }
            }
            // These are acknowledged real writes before the selected export boundary.
            assert_eq!(
                operations
                    .store
                    .result_seed_page(fence.attempt(), None)
                    .await
                    .unwrap()
                    .len(),
                usize::from(phase == FinishExportPhase::Completion)
            );
            operations
                .store
                .cancel_study(&claim.point.study)
                .await
                .unwrap();
            release.send(()).unwrap();
            let record = finishing.await.unwrap();
            assert_eq!(record.attempt_id, attempt_id);
            assert_eq!(record.attempt_key.as_deref(), Some(fence.attempt()));
            let retained = operations
                .store
                .canonical_attempt(fence.attempt())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(retained.generation, fence.generation());
            assert!(retained.closed() && !retained.ingestion_open());
            assert!(
                operations.store.mark_study_started(&claim).await.is_err(),
                "cancellation cannot redispatch this native start"
            );
            let point = operations
                .store
                .canonical_study_point(&claim.point.key)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(point.attempt.as_deref(), Some(fence.attempt()));
            let facts = pse_operations::canonical_studies::point_facts(&point).unwrap();
            assert_eq!(facts.attempt_count, 1);
            assert!(facts.native_started);
            if unrelated_export || unrelated_heartbeat {
                let WorkflowError::Shared(cause) = record.attempt.as_ref().unwrap_err().as_ref()
                else {
                    unreachable!()
                };
                assert!(Arc::ptr_eq(cause, &unrelated));
                assert!(
                    !retained.terminal()
                        && record.manifest.is_none()
                        && record.completion.is_none()
                );
                assert!(record.solutions.is_empty());
                let error = point_runtime
                    .record_study_attempt(&point, &claim.start, &record)
                    .await
                    .unwrap_err();
                let WorkflowError::Shared(retained_error) = error else {
                    unreachable!()
                };
                assert!(Arc::ptr_eq(
                    &retained_error,
                    record.attempt.as_ref().unwrap_err()
                ));
                let unchanged = operations
                    .store
                    .canonical_study_point(&point.key)
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(unchanged, point);
                assert!(
                    pse_operations::canonical_studies::point_outcome(&unchanged)
                        .unwrap()
                        .is_none()
                );
            } else {
                assert!(
                    retained.terminal(),
                    "case {index}, phase {phase:?}: {:?}",
                    record.attempt
                );
                assert_eq!(record.attempt.as_ref().unwrap().key, fence.attempt());
                assert_eq!(
                    record
                        .attempt
                        .as_ref()
                        .unwrap()
                        .outcome
                        .as_ref()
                        .map(|value| value.as_str()),
                    Some("cancelled")
                );
                let stored = record.completion.as_ref().unwrap();
                assert_eq!(stored.state, AttemptState::Cancelled);
                let descriptors = pse_operations::canonical_execution::decode_result_descriptors(
                    record.manifest.as_ref().unwrap(),
                )
                .unwrap();
                let prefix = descriptors
                    .iter()
                    .find(|set| set.name == "__prefix")
                    .unwrap();
                assert_eq!((prefix.batch_count, prefix.row_count), (1, 1));
                let selection = operations
                    .store
                    .read_results(fence.run(), fence.attempt())
                    .await
                    .unwrap();
                let payload = operations
                    .store
                    .result_payload(&selection, &prefix.key, 0)
                    .await
                    .unwrap();
                assert_eq!(&payload.batch.payload[..], &[7]);
                drop(payload);
                drop(selection);
                assert_eq!(
                    descriptors.iter().any(|set| set.name == "__progress"),
                    phase != FinishExportPhase::Progress
                );
                assert_eq!(
                    descriptors.iter().any(|set| set.name == "__seeds"),
                    matches!(
                        phase,
                        FinishExportPhase::Seed | FinishExportPhase::Completion
                    )
                );
                let has_tables = descriptors.iter().any(|set| !set.name.starts_with("__"));
                assert_eq!(
                    has_tables,
                    matches!(
                        phase,
                        FinishExportPhase::Seed | FinishExportPhase::Completion
                    )
                );
                if has_tables {
                    let actual_tables = descriptors
                        .iter()
                        .filter(|set| !set.name.starts_with("__"))
                        .map(|set| (set.name.clone(), set.row_count))
                        .collect::<BTreeMap<_, _>>();
                    assert_eq!(actual_tables, expected_tables);
                }
                if phase == FinishExportPhase::Completion {
                    assert_eq!(
                        serde_json::to_value(stored.completion.as_ref().unwrap()).unwrap(),
                        original
                    );
                    assert_eq!(record.solutions.len(), 1);
                    let seed = operations
                        .store
                        .result_seed_page(fence.attempt(), None)
                        .await
                        .unwrap()
                        .remove(0);
                    let operation = format!("seed:{}:{}", fence.attempt(), seed.key);
                    // Replay a definitely acknowledged registration after cancellation.
                    // This is receipt-first replay, not a simulated lost response.
                    operations
                        .store
                        .register_result_seed(&fence, &operation, &seed)
                        .await
                        .unwrap();
                    assert_eq!(
                        operations
                            .store
                            .result_seed_page(fence.attempt(), None)
                            .await
                            .unwrap(),
                        [seed]
                    );
                } else {
                    assert!(stored.completion.is_none());
                    assert!(record.solutions.is_empty());
                }
                let reopened = operations
                    .record(fence.run(), fence.attempt())
                    .await
                    .unwrap();
                assert_eq!(reopened.attempt_id, attempt_id);
                assert_eq!(reopened.manifest, record.manifest);
                assert_eq!(
                    serde_json::to_value(&***reopened.completion.as_ref().unwrap()).unwrap(),
                    serde_json::to_value(&***stored).unwrap()
                );
                assert_eq!(reopened.solutions, record.solutions);
                let actual = operations
                    .store
                    .canonical_run(fence.run())
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(actual.terminal_attempt.as_deref(), Some(fence.attempt()));
                // Exercise both owning projections on the same settled attempt.
                // Recovery reads its existing terminal receipt and never dispatches.
                let projected = if matches!(
                    phase,
                    FinishExportPhase::Seed | FinishExportPhase::Completion
                ) {
                    assert!(point_runtime.recover_study_point(&point.key).await.unwrap());
                    operations
                        .store
                        .canonical_study_point(&point.key)
                        .await
                        .unwrap()
                        .unwrap()
                } else {
                    point_runtime
                        .record_study_attempt(&point, &claim.start, &record)
                        .await
                        .unwrap()
                };
                let expected_science = if phase == FinishExportPhase::Completion {
                    let assessment = &result.completion().unwrap().assessments[0];
                    pse_model::study::ScientificFacts {
                        usable: result.usable(),
                        candidate_use: Some(assessment.usability),
                        seed_permission: assessment.permits_seed,
                    }
                } else {
                    pse_model::study::ScientificFacts::default()
                };
                assert_eq!(projected.attempt.as_deref(), Some(fence.attempt()));
                assert!(projected.settled && !projected.assigned);
                // Recovery discovery may have retained this key before the
                // normal terminal observation won. Its fresh snapshot must
                // leave that exact admitted observation untouched.
                assert!(!point_runtime.recover_study_point(&point.key).await.unwrap());
                let unchanged = operations
                    .store
                    .canonical_study_point(&point.key)
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(unchanged.revision, projected.revision);
                assert_eq!(unchanged.facts, projected.facts);
                assert_eq!(unchanged.outcome, projected.outcome);
                let projected_facts =
                    pse_operations::canonical_studies::point_facts(&projected).unwrap();
                assert_eq!(
                    projected_facts.lifecycle,
                    pse_model::generated::enums::StudyPointState::Cancelled
                );
                assert_eq!(projected_facts.scientific, expected_science);
                assert_eq!(projected_facts.attempt_count, 1);
                assert!(projected_facts.native_started);
                assert_eq!(
                    projected_facts.retry_failure,
                    stored.termination.retry_failure
                );
                assert_eq!(projected_facts.effect, stored.termination.effect);
                let outcome = pse_operations::canonical_studies::point_outcome(&projected)
                    .unwrap()
                    .unwrap();
                assert_eq!(outcome.lifecycle, projected_facts.lifecycle);
                assert_eq!(outcome.scientific, expected_science);
                assert_eq!(outcome.start.as_ref(), Some(&claim.start));
                assert_eq!(outcome.attempts.len(), 1);
                let observed_attempt = &outcome.attempts[0];
                assert_eq!(observed_attempt.attempt_id, Some(attempt_id));
                assert_eq!(observed_attempt.lifecycle, Some(AttemptState::Cancelled));
                assert_eq!(observed_attempt.scientific, expected_science);
                assert_eq!(observed_attempt.start.as_ref(), Some(&claim.start));
                assert_eq!(observed_attempt.effect, stored.termination.effect);
            }
        }
        drop(point_runtime);
        operations.store.remove_isolated_fixture().await.unwrap();
        // The originally joined scientific owner retains its original conclusion.
        assert_eq!(
            serde_json::to_value(result.completion().unwrap()).unwrap(),
            original
        );
        drop(result);
        drop(handle);
        drop(package);
    }
    #[tokio::test]
    async fn canonical_cancelled_prefix_and_late_heartbeat_reopen_acknowledged_unavailable_science()
    {
        let (operations, fence) = study_claimed().await;
        operations
            .store
            .append_result_batch(
                &fence,
                "prefix",
                "observations",
                0,
                pse_operations::canonical_execution::ResultPayload::new([7].to_vec()).unwrap(),
                1,
            )
            .await
            .unwrap();
        let attempt_id = pse_operations::mint_id();
        let mut attempt = DurableAttempt::claimed(
            &operations,
            fence.clone(),
            pse_operations::mint_id(),
            attempt_id,
        )
        .unwrap();
        // The earlier heartbeat observation is healthy. Its final joined result
        // observes the cancellation acknowledged after export has already begun.
        let (noticed, notice) = tokio::sync::oneshot::channel();
        let noticed = Mutex::new(Some(noticed));
        attempt
            .start(Arc::new(move || {
                if let Some(noticed) = noticed.lock().unwrap().take() {
                    let _ = noticed.send(());
                }
            }))
            .await
            .unwrap();
        operations
            .store
            .renew_attempt(&fence, Duration::from_secs(60))
            .await
            .unwrap();
        operations.store.cancel_study("cancel-study").await.unwrap();
        notice.await.unwrap();
        let failure = attempt.heartbeat.take().unwrap().stop().await.unwrap();
        assert!(attempt.cancelled_refusal(&failure));
        let unrelated = Arc::new(WorkflowError::Canonical(
            pse_operations::canonical::CanonicalError::Timeout,
        ));
        assert!(Arc::ptr_eq(
            &export_failure(Some(failure.clone()), Some(unrelated.clone()), Some(&fence)).unwrap(),
            &unrelated
        ));
        assert!(Arc::ptr_eq(
            &export_failure(Some(unrelated.clone()), Some(failure.clone()), Some(&fence)).unwrap(),
            &unrelated
        ));
        let wrong = Arc::new(WorkflowError::Canonical(
            pse_operations::canonical::CanonicalError::StudyCancellation {
                run: fence.run().into(),
                attempt: fence.attempt().into(),
                generation: fence.generation() + 1,
                operation: "renew".into(),
            },
        ));
        assert!(!attempt.cancelled_refusal(&wrong));
        let outcome = super::failure(&failure, true);
        let stored = StoredCompletion {
            version: 1,
            attempt_id,
            completion: None,
            termination: outcome.detail,
            state: AttemptState::Cancelled,
            progress: super::super::ProgressCoverage::Unknown {},
        };
        attempt.stream.finish().await;
        let record = attempt.terminate(stored.clone()).await.unwrap();
        assert_eq!(record.attempt_id, attempt_id);
        assert_eq!(
            record
                .attempt
                .as_ref()
                .unwrap()
                .outcome
                .as_ref()
                .map(|value| value.as_str()),
            Some("cancelled")
        );
        assert!(record.completion.as_ref().unwrap().completion.is_none());
        assert!(record.solutions.is_empty());
        assert_eq!(
            pse_operations::canonical_execution::decode_result_descriptors(
                record.manifest.as_ref().unwrap()
            )
            .unwrap()[0]
                .row_count,
            1
        );
        // Existing acknowledged terminal data remains immutable on repeat settlement.
        let mut changed = stored;
        changed.attempt_id = pse_operations::mint_id();
        changed.state = AttemptState::Failed;
        changed.completion = Some(super::super::Completion::default());
        let repeated = attempt.terminate(changed).await.unwrap();
        assert_eq!(repeated.attempt_id, attempt_id);
        assert!(repeated.completion.as_ref().unwrap().completion.is_none());
        assert_eq!(
            repeated.completion.as_ref().unwrap().state,
            AttemptState::Cancelled
        );
    }
    #[tokio::test]
    async fn canonical_slim_completion_matches_inline_and_chunked_originals() {
        for chunked in [false, true] {
            let (operations, fence) = claimed().await;
            let original = StoredCompletion {
                version: 1,
                attempt_id: pse_operations::mint_id(),
                completion: Some(super::super::Completion::default()),
                termination: detail(
                    TerminationCause::Assessment {
                        usable: false,
                        candidate_use: vec![],
                        diagnostics: vec![],
                    },
                    false,
                ),
                state: AttemptState::Failed,
                progress: super::super::ProgressCoverage::Unknown {},
            };
            let bytes = serde_json::to_vec(&original).unwrap();
            let receipt = if chunked {
                write_chunks(
                    &operations.store,
                    &fence,
                    "__completion",
                    0,
                    &bytes,
                    &PublicationAcks::new(&operations.pool),
                    reserve(&operations.pool, bytes.len()).unwrap(),
                )
                .await
                .unwrap()
            } else {
                inline_completion(original)
            };
            // An unrelated result set is not part of a completion property read.
            write_chunks(
                &operations.store,
                &fence,
                "unrelated",
                0,
                &[7; 31],
                &PublicationAcks::new(&operations.pool),
                reserve(&operations.pool, 31).unwrap(),
            )
            .await
            .unwrap();
            let closed = operations
                .store
                .close_result_ingestion(&fence, "slim-close")
                .await
                .unwrap();
            let record = operations
                .settle(
                    &closed,
                    "slim-seal",
                    TerminalClass::Failed,
                    &receipt,
                    SeedEligibility::Unavailable,
                    None,
                )
                .await
                .unwrap();
            let slim = operations
                .completion(fence.run(), fence.attempt())
                .await
                .unwrap();
            assert_eq!(serde_json::to_vec(&**slim).unwrap(), bytes);
            let full: &StoredCompletion = record.completion.as_ref().unwrap();
            assert_eq!(serde_json::to_vec(full).unwrap(), bytes);
            operations.store.remove_isolated_fixture().await.unwrap();
            assert!(
                operations
                    .completion(fence.run(), fence.attempt())
                    .await
                    .is_err()
            );
            assert_eq!(
                serde_json::to_vec(&**slim).unwrap(),
                bytes,
                "previously copied completion does not grant a fresh read"
            );
        }
    }

    #[tokio::test]
    async fn canonical_complete_export_cancelled_at_seal_preserves_original_completion() {
        let (operations, fence) = study_claimed().await;
        let closed = operations
            .store
            .close_result_ingestion(&fence, "completed-close")
            .await
            .unwrap();
        let attempt_id = pse_operations::mint_id();
        let original = StoredCompletion {
            version: 1,
            attempt_id,
            completion: Some(super::super::Completion::default()),
            termination: detail(
                TerminationCause::Assessment {
                    usable: false,
                    candidate_use: vec![],
                    diagnostics: vec![],
                },
                false,
            ),
            state: AttemptState::Failed,
            progress: super::super::ProgressCoverage::Unknown {},
        };
        operations.store.cancel_study("cancel-study").await.unwrap();
        let refusal = operations
            .settle(
                &closed,
                "original-seal",
                TerminalClass::Failed,
                &inline_completion(original.clone()),
                SeedEligibility::CompleteOriginalInventory,
                None,
            )
            .await
            .unwrap_err();
        assert!(acknowledged_cancellation(&refusal, &fence));
        let mut attempt =
            DurableAttempt::claimed(&operations, fence, pse_operations::mint_id(), attempt_id)
                .unwrap();
        attempt.stream.finish().await;
        let record = attempt.terminate(original.clone()).await.unwrap();
        assert_eq!(
            record.completion.as_ref().unwrap().state,
            AttemptState::Cancelled
        );
        assert!(record.completion.as_ref().unwrap().completion.is_some());
        assert_eq!(record.attempt_id, attempt_id);
    }
    #[tokio::test]
    async fn canonical_cancelled_file_reservation_settles_the_exact_study_attempt() {
        let (runtime, claim) = study_claim_runtime().await;
        let operations = runtime.operations().unwrap();
        let fence = &claim.fence;
        let point = operations.store.mark_study_started(&claim).await.unwrap();
        let attempt_id = pse_operations::mint_id();
        let mut attempt = DurableAttempt::claimed(
            operations,
            fence.clone(),
            pse_operations::mint_id(),
            attempt_id,
        )
        .unwrap();
        attempt.start(Arc::new(|| {})).await.unwrap();
        operations.store.cancel_study(&point.study).await.unwrap();
        let operation = format!("cancelled-file:{}", fence.attempt());
        let refusal = operations
            .store
            .reserve_result_file(fence, &operation, &result_payload_digest(b"[1]"))
            .await
            .unwrap_err();
        assert!(matches!(
            &refusal,
            pse_operations::canonical::CanonicalError::StudyCancellation {
                run, attempt, generation, operation: original,
            } if run == fence.run() && attempt == fence.attempt()
                && *generation == fence.generation() && original == &operation
        ));
        let record = attempt.abandon(&Arc::new(refusal.into())).await;
        let terminal = record.attempt.as_ref().unwrap();
        assert_eq!(terminal.key, fence.attempt());
        assert!(terminal.terminal() && terminal.closed() && !terminal.ingestion_open());
        assert_eq!(
            terminal.outcome.as_ref().map(|value| value.as_str()),
            Some("cancelled")
        );
        assert_eq!(record.attempt_id, attempt_id);
        assert!(record.completion.as_ref().unwrap().completion.is_none());
        assert!(record.solutions.is_empty());
        assert!(
            pse_operations::canonical_execution::decode_result_descriptors(
                record.manifest.as_ref().unwrap()
            )
            .unwrap()
            .is_empty()
        );
        let settled = runtime
            .record_study_attempt(&point, &claim.start, &record)
            .await
            .unwrap();
        assert!(settled.settled && !settled.assigned);
        let facts = pse_operations::canonical_studies::point_facts(&settled).unwrap();
        assert_eq!(
            facts.lifecycle,
            pse_model::generated::enums::StudyPointState::Cancelled
        );
        assert!(facts.native_started && !facts.scientific.usable);
        assert_eq!(facts.attempt_count, 1);
        drop(runtime);
    }
    #[tokio::test]
    async fn canonical_preentry_refusal_settles_only_acknowledged_cancellation() {
        let (operations, fence) = study_claimed().await;
        operations.store.cancel_study("cancel-study").await.unwrap();
        let refused = Arc::new(WorkflowError::Canonical(
            operations
                .store
                .renew_attempt(&fence, Duration::from_secs(60))
                .await
                .unwrap_err(),
        ));
        let attempt = DurableAttempt::claimed(
            &operations,
            fence.clone(),
            pse_operations::mint_id(),
            pse_operations::mint_id(),
        )
        .unwrap();
        assert!(attempt.cancelled_refusal(&refused));
        // No heartbeat or scientific owner has entered: the direct response alone
        // must retain the exact cancellation and unavailable scientific facts.
        let record = attempt.abandon(&refused).await;
        assert!(record.attempt.is_ok());
        assert_eq!(
            record.completion.as_ref().unwrap().state,
            AttemptState::Cancelled
        );
        assert!(record.completion.as_ref().unwrap().completion.is_none());
        let reopened = operations
            .record(fence.run(), fence.attempt())
            .await
            .unwrap();
        assert_eq!(reopened.attempt_id, record.attempt_id);
        assert!(reopened.completion.as_ref().unwrap().completion.is_none());

        let (operations, fence) = study_claimed().await;
        operations.store.cancel_study("cancel-study").await.unwrap();
        let attempt = DurableAttempt::claimed(
            &operations,
            fence.clone(),
            pse_operations::mint_id(),
            pse_operations::mint_id(),
        )
        .unwrap();
        let unrelated = Arc::new(WorkflowError::Shared(Arc::new(WorkflowError::Canonical(
            pse_operations::canonical::CanonicalError::Timeout,
        ))));
        let record = attempt.abandon(&unrelated).await;
        assert!(Arc::ptr_eq(
            record.attempt.as_ref().unwrap_err(),
            &unrelated
        ));
        assert!(record.manifest.is_none());
        assert!(record.completion.is_none());
        assert!(
            !operations
                .store
                .canonical_attempt(fence.attempt())
                .await
                .unwrap()
                .unwrap()
                .terminal()
        );
    }
    #[tokio::test]
    async fn canonical_completion_export_and_reconciliation_precede_final_heartbeat_join() {
        for final_failure in 0..3 {
            let (operations, fence) = study_claimed().await;
            let attempt_id = pse_operations::mint_id();
            let mut attempt = DurableAttempt::claimed(
                &operations,
                fence.clone(),
                pse_operations::mint_id(),
                attempt_id,
            )
            .unwrap();
            let original = StoredCompletion {
                version: 1,
                attempt_id,
                completion: Some(super::super::Completion::default()),
                termination: detail(
                    TerminationCause::Assessment {
                        usable: false,
                        candidate_use: vec![],
                        diagnostics: vec![],
                    },
                    false,
                ),
                state: AttemptState::Failed,
                progress: super::super::ProgressCoverage::Unknown {},
            };
            let (stop, stopped) = tokio::sync::oneshot::channel();
            let failure = Arc::new(Mutex::new(None));
            let recorded = failure.clone();
            let final_operations = operations.clone();
            let final_fence = fence.clone();
            let task = tokio::spawn(async move {
                stopped.await.unwrap();
                // The stop/join boundary must see the actual completion chunks
                // frozen in a reconciled manifest, with terminal admission pending.
                let actual = final_operations
                    .store
                    .canonical_attempt(final_fence.attempt())
                    .await
                    .unwrap()
                    .unwrap();
                assert!(actual.closed() && !actual.ingestion_open() && !actual.terminal());
                assert!(actual.closed_manifest.is_some());
                if final_failure != 0 {
                    final_operations
                        .store
                        .cancel_study("cancel-study")
                        .await
                        .unwrap();
                    let error = if final_failure == 1 {
                        final_operations
                            .store
                            .renew_attempt(&final_fence, Duration::from_secs(60))
                            .await
                            .unwrap_err()
                    } else {
                        pse_operations::canonical::CanonicalError::Timeout
                    };
                    *recorded.lock().unwrap() = Some(Arc::new(WorkflowError::Canonical(error)));
                }
            });
            attempt.heartbeat = Some(Heartbeat {
                stop,
                task,
                failure,
            });
            attempt.stream.finish().await;
            let settled = attempt.terminate(original.clone()).await;
            if final_failure == 2 {
                let error = settled.unwrap_err();
                assert!(!acknowledged_cancellation(&error, &fence));
                assert!(
                    !operations
                        .store
                        .canonical_attempt(fence.attempt())
                        .await
                        .unwrap()
                        .unwrap()
                        .terminal()
                );
            } else {
                let record = settled.unwrap();
                assert_eq!(record.attempt_id, attempt_id);
                assert_eq!(
                    record.attempt.as_ref().unwrap().generation,
                    fence.generation()
                );
                assert!(record.attempt.as_ref().unwrap().terminal());
                assert!(record.completion.as_ref().unwrap().completion.is_some());
                assert_eq!(
                    serde_json::to_value(&record.completion.as_ref().unwrap().completion).unwrap(),
                    serde_json::to_value(&original.completion).unwrap(),
                    "late cancellation preserves the fully exported original science"
                );
                assert_eq!(
                    record.completion.as_ref().unwrap().state,
                    if final_failure == 1 {
                        AttemptState::Cancelled
                    } else {
                        AttemptState::Failed
                    }
                );
                assert!(
                    pse_operations::canonical_execution::decode_result_descriptors(
                        record.manifest.as_ref().unwrap()
                    )
                    .unwrap()
                    .iter()
                    .any(|d| d.name == "__completion")
                );
            }
        }
    }
    #[allow(
        unsafe_code,
        reason = "native mechanism fixture records truthful failed observations only"
    )]
    async fn admit(operations: &Operations, fence: &AttemptFence) -> ResultRead {
        let closed = operations
            .store
            .close_result_ingestion(fence, "codec-close")
            .await
            .unwrap();
        let manifest = operations
            .store
            .reconcile_closed_attempt(&closed)
            .await
            .unwrap();
        // SAFETY: this codec fixture reconciled its exact frozen observations and
        // seals only a failed attempt, without granting scientific success or usability.
        unsafe {
            operations
                .store
                .seal_attempt(
                    &manifest,
                    "codec-seal",
                    TerminalClass::Failed,
                    &[9],
                    SeedEligibility::Unavailable,
                )
                .await
        }
        .unwrap();
        operations
            .store
            .read_results(fence.run(), fence.attempt())
            .await
            .unwrap()
    }
    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn canonical_private_horizon_prefix_does_not_admit_a_seed_before_final_coverage() {
        use super::super::durable_tests::{LINEAR, package_on};
        let runtime = super::super::tests::runtime_with_workspace(32 << 20);
        let (package, analysis) = package_on(&runtime, LINEAR).await;
        let result = package
            .prepare_analysis(&analysis, &crate::CancelSource::new())
            .await
            .unwrap()
            .start()
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert!(result.usable());
        let (operations, fence) = claimed().await;
        let mut attempt = DurableAttempt::claimed(
            &operations,
            fence.clone(),
            result.run_id,
            pse_operations::mint_id(),
        )
        .unwrap();
        attempt.start(Arc::new(|| {})).await.unwrap();
        attempt
            .begin_horizon_prefix(super::super::StudyOutputDemand::Full {}, 1)
            .unwrap();
        attempt.publish_horizon_step(&result).await.unwrap();
        let solution = attempt.solutions[0].1;
        assert!(
            operations
                .store
                .read_results(fence.run(), fence.attempt())
                .await
                .is_err()
        );
        assert!(operations.seed(solution).await.is_err());
        let refusal = Arc::new(contract("exact horizon prefix publication refusal"));
        attempt.refuse_horizon_prefix(refusal.clone());
        let record = attempt.finish(&result, false).await;
        assert!(record.attempt.is_err());
        assert!(record.completion.is_none());
        let retained = operations
            .store
            .canonical_attempt(fence.attempt())
            .await
            .unwrap()
            .unwrap();
        assert!(
            !retained.terminal()
                && retained.closed_manifest.is_none()
                && retained.seed_manifest.is_none()
        );
        assert!(operations.seed(solution).await.is_err());
    }

    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn canonical_detail_refusal_preserves_original_scientific_completion() {
        use super::super::durable_tests::{LINEAR, package_on};
        let runtime = super::super::tests::runtime_with_workspace(32 << 20);
        let (package, analysis) = package_on(&runtime, LINEAR).await;
        let original = package
            .prepare_analysis(&analysis, &crate::CancelSource::new())
            .await
            .unwrap()
            .start()
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert!(original.usable());
        let RunRequest::Modeling(requests) = original.request() else {
            unreachable!()
        };
        let RunReport::Modeling(steps) = original.report().unwrap() else {
            unreachable!()
        };
        let result = RunResult::joined(
            original.run_id,
            runtime,
            RunRequest::Modeling(requests.clone()),
            None,
            Ok(RunReport::Modeling(steps.clone())),
        )
        .completed();
        let captured = serde_json::to_value(result.completion().unwrap()).unwrap();
        let (operations, fence) = claimed().await;
        let mut attempt = DurableAttempt::claimed(
            &operations,
            fence.clone(),
            result.run_id,
            pse_operations::mint_id(),
        )
        .unwrap();
        let detail = super::super::StudyDetailDemand {
            relations: vec!["runtime.solve_variables".into()],
            members: Vec::new(),
            maximum_rows: 1,
            maximum_bytes: 1,
        };
        attempt.set_output(super::super::StudyOutputDemand::Compact {
            detail,
            failed_detail: Default::default(),
        });
        attempt.start(Arc::new(|| {})).await.unwrap();
        let result = result.finished(Some(attempt), false).await;
        assert!(result.report().is_err() && !result.usable());
        assert_eq!(
            serde_json::to_value(result.completion().unwrap()).unwrap(),
            captured
        );
        assert!(
            result
                .completion()
                .unwrap()
                .assessments
                .iter()
                .all(|assessment| assessment.permits_result)
        );
        let RunDurability::Durable(record) = result.durability() else {
            unreachable!()
        };
        assert!(record.attempt.is_err() && record.completion.is_none());
        let retained = operations
            .store
            .canonical_attempt(fence.attempt())
            .await
            .unwrap()
            .unwrap();
        assert!(
            !retained.terminal()
                && retained.closed_manifest.is_none()
                && retained.seed_manifest.is_none()
        );
        assert!(
            operations
                .store
                .read_results(fence.run(), fence.attempt())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn canonical_failed_export_prefix_remains_unsealed_and_unreadable() {
        use pse_relations::generated::{enums::NativeMetricKind, runtime::solve_metrics};
        let (operations, fence) = claimed().await;
        let runtime = super::super::durable_tests::durable_runtime();
        let registry = runtime.registry();
        let validation = runtime.validation_context();
        let pool = operations.pool.clone();
        let schema = pse_schema::arrow::relation_schema_ref(
            registry,
            registry.relation_by_id(solve_metrics::RELATION_ID).unwrap(),
        )
        .unwrap();
        let cursor = super::super::ResultCursor::new(
            schema,
            solve_metrics::RELATION_ID,
            1,
            super::super::ResultOrder::Public,
            |request| async move {
                let cancel = pse_columnar::CancellationToken::new();
                let mut rows = super::super::result_export::Rows::<solve_metrics::Row>::new(
                    &request,
                    registry,
                    &pool,
                    &cancel,
                    &validation,
                )
                .map_err(|error| Arc::new(relation(error)))?;
                rows.push(solve_metrics::Row {
                    run_id: pse_operations::mint_id(),
                    step: 0,
                    namespace: "independent".into(),
                    name: "prefix".into(),
                    kind: NativeMetricKind::Real,
                    real: Some(-0.0),
                    integer: None,
                    boolean: None,
                    text: None,
                    unavailable: None,
                })
                .await
                .map_err(|error| Arc::new(relation(error)))?;
                Err(Arc::new(contract(
                    "deliberate producer failure after one acknowledged chunk",
                )))
            },
        )
        .unwrap();
        let failure = super::super::result_projection::store_result_table(
            &operations.store,
            &fence,
            solve_metrics::RELATION_ID,
            cursor,
            &operations.pool,
            &PublicationAcks::new(&operations.pool),
        )
        .await
        .unwrap_err();
        assert!(failure.to_string().contains("deliberate producer failure"));
        assert_eq!(
            pse_operations::testing::pending_result_batch_rows(
                &operations.store,
                &fence,
                &solve_metrics::RELATION_ID.to_string(),
                0
            )
            .await
            .unwrap(),
            Some(1)
        );
        let attempt_id = pse_operations::mint_id();
        let mut attempt = DurableAttempt::claimed(
            &operations,
            fence.clone(),
            pse_operations::mint_id(),
            attempt_id,
        )
        .unwrap();
        attempt.stream.finish().await;
        let record = attempt.reject_export(Arc::new(failure)).await;
        assert!(record.attempt.is_err());
        assert!(record.completion.is_none());
        let retained = operations
            .store
            .canonical_attempt(fence.attempt())
            .await
            .unwrap()
            .unwrap();
        assert!(!retained.ingestion_open());
        assert!(!retained.terminal());
        assert!(retained.closed_manifest.is_none());
        assert!(
            operations
                .store
                .read_results(fence.run(), fence.attempt())
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn canonical_seed_chunks_exact_ieee_bits_and_accounted_join() {
        let (operations, fence) = claimed().await;
        // Size the scientific seed against the current physical block limit.
        // Every four-value cycle below encodes at least this many JSON bytes
        // (replace the opening array bracket with the next cycle's comma).
        // Keep an uneven tail so joining also checks the final partial chunk.
        let minimum_cycle = [
            0x8000000000000000_u64,
            0x7ff8000000000091,
            0xfff0000000000000,
            0,
        ];
        let minimum_cycle_bytes = serde_json::to_vec(&minimum_cycle).unwrap().len() - 1;
        let values = 4 * (RESULT_BATCH_BYTES / minimum_cycle_bytes + 1) + 19;
        let original = (0..values)
            .map(|i| {
                f64::from_bits(match i % 4 {
                    0 => 0x8000000000000000,
                    1 => 0x7ff8000000000091,
                    2 => 0xfff0000000000000,
                    _ => i as u64,
                })
            })
            .collect::<Vec<_>>();
        let bytes =
            serde_json::to_vec(&seed_payload(&WarmPayload::Root(original.clone()))).unwrap();
        assert!(bytes.len() > RESULT_BATCH_BYTES);
        assert_ne!(bytes.len() % RESULT_BATCH_BYTES, 0);
        let expected_chunks = bytes.len().div_ceil(RESULT_BATCH_BYTES) as u64;
        let acknowledgments = PublicationAcks::new(&operations.pool);
        let first = write_chunks(
            &operations.store,
            &fence,
            "__seeds",
            0,
            &bytes,
            &acknowledgments,
            reserve(&operations.pool, bytes.len()).unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(first.batch_count, expected_chunks);
        let second = write_chunks(
            &operations.store,
            &fence,
            "__seeds",
            first.batch_count,
            &bytes,
            &acknowledgments,
            reserve(&operations.pool, bytes.len()).unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(second.batch_count, expected_chunks);
        let read = admit(&operations, &fence).await;
        let before = operations.pool.reserved();
        let (joined, owner) = read_chunks(
            &operations.store,
            &operations.pool,
            &read,
            "__seeds",
            first.batch_count,
            second.batch_count,
            second.payload_bytes,
            &second.digest,
            true,
        )
        .await
        .unwrap();
        assert!(operations.pool.reserved() > before);
        assert_eq!(joined, bytes);
        let decoded: SeedPayload = serde_json::from_slice(&joined).unwrap();
        let WarmPayload::Root(restored) = warm_payload(decoded) else {
            unreachable!()
        };
        assert_eq!(bits(&restored), bits(&original));
        drop(owner);
        assert_eq!(operations.pool.reserved(), before);
        let tiny: Arc<dyn MemoryPool> = Arc::new(
            datafusion::execution::memory_pool::GreedyMemoryPool::new(1024),
        );
        assert!(
            read_chunks(
                &operations.store,
                &tiny,
                &read,
                "__seeds",
                0,
                first.batch_count,
                first.payload_bytes,
                &first.digest,
                true
            )
            .await
            .is_err()
        );
        assert_eq!(tiny.reserved(), 0);
        assert!(
            read_chunks(
                &operations.store,
                &operations.pool,
                &read,
                "__seeds",
                0,
                first.batch_count,
                first.payload_bytes,
                "wrong-digest",
                true
            )
            .await
            .is_err()
        );
    }
    #[tokio::test]
    async fn canonical_progress_writer_refusal_retains_original_science_and_uncertain_coverage() {
        use super::super::durable_tests::{LINEAR, package_on};
        use super::super::{RunDurability, tests};
        let runtime = tests::runtime_with_workspace(32 << 20);
        let (package, analysis) = package_on(&runtime, LINEAR).await;
        let prepared = package
            .prepare_analysis(&analysis, &crate::CancelSource::new())
            .await
            .unwrap();
        let handle = prepared.start().unwrap();
        let result = handle.wait().await.unwrap();
        assert!(matches!(result.durability(), RunDurability::Ephemeral));
        assert!(result.usable(), "{:?}", result.report());
        let scientific = serde_json::to_value(result.completion().unwrap()).unwrap();
        let (operations, fence) = claimed().await;
        // An actual immutable original request already exists. Reusing its
        // operation with different bytes must refuse the optional append.
        let old = super::super::ProgressEventDocument::from(&Event {
            source: None,
            phase: "prior-original".into(),
            elapsed: Duration::ZERO,
            values: BTreeMap::new(),
            incumbent: None,
        });
        operations
            .store
            .append_result_batch(
                &fence,
                &format!("progress:{}:0", fence.attempt()),
                "__progress",
                0,
                pse_operations::canonical_execution::ResultPayload::new(
                    serde_json::to_vec(&[old]).unwrap(),
                )
                .unwrap(),
                1,
            )
            .await
            .unwrap();
        let mut attempt = DurableAttempt::claimed(
            &operations,
            fence.clone(),
            result.run_id,
            pse_operations::mint_id(),
        )
        .unwrap();
        attempt.start(Arc::new(|| {})).await.unwrap();
        attempt.stream.tap.observe(&Event {
            source: None,
            phase: "counter".into(),
            elapsed: Duration::ZERO,
            values: BTreeMap::from([("counter".into(), Metric::Integer((1_i64 << 53) + 1))]),
            incumbent: None,
        });
        let record = attempt.finish(&result, false).await;
        let stored = record.completion.as_ref().unwrap();
        assert_eq!(stored.version, 2);
        assert_eq!(
            serde_json::to_value(stored.completion.as_ref().unwrap()).unwrap(),
            scientific
        );
        assert_eq!(stored.state, classify(&result, false).state);
        assert_eq!(
            stored.progress,
            super::super::ProgressCoverage::Sampled {
                observed: 1,
                queued: 1,
                dropped_before_queue: 0,
                acknowledged: 0,
                unacknowledged: 1,
                writer: super::super::ProgressWriterOutcome::Failed(
                    super::super::ProgressWriterFailure {
                        stage: super::super::ProgressWriteStage::CanonicalAppend {}
                    }
                )
            }
        );
        let reopened = operations
            .record(fence.run(), fence.attempt())
            .await
            .unwrap();
        assert_eq!(
            reopened.completion.as_ref().unwrap().progress,
            stored.progress
        );
        assert_eq!(
            serde_json::to_value(result.completion().unwrap()).unwrap(),
            scientific
        );
        drop(reopened);
        drop(record);
        operations.store.remove_isolated_fixture().await.unwrap();
        drop(result);
        drop(handle);
        drop(package);
    }

    #[tokio::test]
    async fn canonical_progress_batches_exact_integral_events_under_one_receipt() {
        let (mut operations, fence) = claimed().await;
        operations.policy.batch = 8;
        operations.policy.flush = Duration::from_millis(10);
        let admitted = Arc::new(OnceLock::new());
        admitted.set(fence.clone()).unwrap();
        let mut stream = Streamer::new(
            &operations,
            admitted,
            PublicationAcks::new(&operations.pool),
        );
        for index in 0..3 {
            stream.tap.observe(&Event {
                source: None,
                phase: "counter".into(),
                elapsed: Duration::from_millis(index as u64),
                values: BTreeMap::from([(
                    "counter".into(),
                    Metric::Integer((1_i64 << 53) + index),
                )]),
                incumbent: None,
            });
        }
        assert_eq!(
            stream.finish().await,
            super::super::ProgressCoverage::Sampled {
                observed: 3,
                queued: 3,
                dropped_before_queue: 0,
                acknowledged: 3,
                unacknowledged: 0,
                writer: super::super::ProgressWriterOutcome::Complete
            }
        );
        let read = admit(&operations, &fence).await;
        let descriptor = read
            .sets()
            .iter()
            .find(|set| set.name == "__progress")
            .unwrap();
        assert_eq!(descriptor.batch_count, 1);
        assert_eq!(descriptor.row_count, 3);
        let payload = operations
            .store
            .result_payload(&read, &descriptor.key, 0)
            .await
            .unwrap();
        let events = decode_progress(&payload.batch.payload).unwrap();
        assert_eq!(events.len(), 3);
        for (index, event) in events.iter().enumerate() {
            assert_eq!(event.sequence, Some(index as i64));
            assert!(
                matches!(event.values["counter"],super::super::ProgressMetricDocument::Integer(value) if value==(1_i64<<53)+index as i64)
            );
        }
    }
    #[tokio::test]
    async fn canonical_progress_native_burst_samples_without_waiting_for_writer() {
        let (mut operations, fence) = claimed().await;
        operations.policy.batch = 2;
        operations.policy.flush = Duration::from_millis(1);
        let admitted = Arc::new(OnceLock::new());
        admitted.set(fence.clone()).unwrap();
        let mut stream = Streamer::new(
            &operations,
            admitted,
            PublicationAcks::new(&operations.pool),
        );
        let tap = stream.tap.clone();
        let producer = std::thread::spawn(move || {
            for index in 0..12 {
                tap.observe(&Event {
                    source: None,
                    phase: "counter".into(),
                    elapsed: Duration::from_millis(index),
                    values: BTreeMap::from([(
                        "counter".into(),
                        Metric::Integer((1_i64 << 53) + index as i64),
                    )]),
                    incumbent: None,
                });
            }
        });
        tokio::task::spawn_blocking(move || producer.join().unwrap())
            .await
            .unwrap();
        let coverage = stream.finish().await;
        let super::super::ProgressCoverage::Sampled {
            observed,
            queued,
            dropped_before_queue,
            acknowledged,
            unacknowledged,
            writer,
        } = coverage
        else {
            unreachable!()
        };
        assert_eq!(observed, 12);
        assert_eq!(queued + dropped_before_queue, 12);
        assert_eq!(acknowledged, queued);
        assert_eq!(unacknowledged, 0);
        assert_eq!(writer, super::super::ProgressWriterOutcome::Complete);
        let read = admit(&operations, &fence).await;
        let descriptor = read
            .sets()
            .iter()
            .find(|set| set.name == "__progress")
            .unwrap();
        let mut events = Vec::new();
        for ordinal in 0..descriptor.batch_count {
            let payload = operations
                .store
                .result_payload(&read, &descriptor.key, ordinal)
                .await
                .unwrap();
            events.extend(decode_progress(&payload.batch.payload).unwrap());
        }
        assert_eq!(events.len() as u64, queued);
        for event in &events {
            let index = event.sequence.unwrap();
            assert!(
                matches!(event.values["counter"], super::super::ProgressMetricDocument::Integer(value) if value == (1_i64 << 53) + index)
            );
        }
    }
    #[tokio::test]
    async fn canonical_progress_saturation_records_loss_and_retains_integral_observation() {
        let (mut operations, fence) = claimed().await;
        operations.policy.batch = 1;
        let admitted = Arc::new(OnceLock::new());
        admitted.set(fence.clone()).unwrap();
        let mut stream = Streamer::new(
            &operations,
            admitted,
            PublicationAcks::new(&operations.pool),
        );
        let event = Event {
            source: None,
            phase: "counter".into(),
            elapsed: Duration::from_millis(1),
            values: BTreeMap::from([("counter".into(), Metric::Integer((1_i64 << 53) + 17))]),
            incumbent: None,
        };
        stream.tap.observe(&event);
        stream.tap.observe(&event);
        assert_eq!(
            stream.finish().await,
            super::super::ProgressCoverage::Sampled {
                observed: 2,
                queued: 1,
                dropped_before_queue: 1,
                acknowledged: 1,
                unacknowledged: 0,
                writer: super::super::ProgressWriterOutcome::Complete
            }
        );
        let read = admit(&operations, &fence).await;
        let descriptor = read
            .sets()
            .iter()
            .find(|set| set.name == "__progress")
            .unwrap();
        assert_eq!(descriptor.batch_count, 1);
        let payload = operations
            .store
            .result_payload(&read, &descriptor.key, 0)
            .await
            .unwrap();
        let restored = decode_progress(&payload.batch.payload).unwrap();
        assert!(
            matches!(restored[0].values["counter"],super::super::ProgressMetricDocument::Integer(value) if value==(1_i64<<53)+17)
        );
    }
}

#[cfg(test)]
mod ordinary_progress_tests {
    #![allow(
        clippy::unwrap_used,
        reason = "bounded telemetry controls inspect exact owned queue state"
    )]
    use super::super::{ProgressCoverage, ProgressWriterOutcome};
    use super::*;
    use pse_backend_native::solve::{IncumbentEvent, Metric};

    #[test]
    fn original_completion_precharge_refusal_precedes_clone_and_retains_owner() {
        #[derive(Serialize)]
        struct Original {
            payload: String,
            #[serde(skip)]
            clones: Arc<AtomicU64>,
        }
        impl Clone for Original {
            fn clone(&self) -> Self {
                self.clones.fetch_add(1, Ordering::AcqRel);
                Self {
                    payload: self.payload.clone(),
                    clones: self.clones.clone(),
                }
            }
        }
        let clones = Arc::new(AtomicU64::new(0));
        let original = Original {
            payload: "exact original".repeat(128),
            clones: clones.clone(),
        };
        let tiny: Arc<dyn MemoryPool> = Arc::new(
            datafusion::execution::memory_pool::GreedyMemoryPool::new(1024),
        );
        assert!(copy_original_completion(&tiny, Some(&original), &()).is_err());
        assert_eq!(clones.load(Ordering::Acquire), 0);
        assert_eq!(tiny.reserved(), 0);
        let pool: Arc<dyn MemoryPool> = Arc::new(
            datafusion::execution::memory_pool::GreedyMemoryPool::new(16 << 20),
        );
        let (copy, owner) = copy_original_completion(&pool, Some(&original), &()).unwrap();
        assert_eq!(clones.load(Ordering::Acquire), 1);
        assert_eq!(copy.as_ref().unwrap().payload, original.payload);
        assert!(pool.reserved() > original.payload.len());
        drop(copy);
        assert!(pool.reserved() > 0);
        drop(owner);
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn publication_acknowledgment_owner_precharges_before_set_allocation() {
        let tiny: Arc<dyn MemoryPool> =
            Arc::new(datafusion::execution::memory_pool::GreedyMemoryPool::new(
                RESULT_ACKNOWLEDGMENT_BYTES - 1,
            ));
        let refused = PublicationAcks::new(&tiny);
        assert!(refused.admitted().is_err());
        assert!(refused.state.lock().unwrap().is_none());
        assert_eq!(tiny.reserved(), 0);
        let pool: Arc<dyn MemoryPool> = Arc::new(
            datafusion::execution::memory_pool::GreedyMemoryPool::new(RESULT_ACKNOWLEDGMENT_BYTES),
        );
        let admitted = PublicationAcks::new(&pool);
        assert_eq!(pool.reserved(), RESULT_ACKNOWLEDGMENT_BYTES);
        assert!(admitted.state.lock().unwrap().is_none());
        let writer = admitted.clone();
        drop(admitted);
        assert_eq!(pool.reserved(), RESULT_ACKNOWLEDGMENT_BYTES);
        drop(writer);
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn progress_readback_is_reserved_for_uncertain_publication_not_known_queue_loss() {
        use super::super::{ProgressWriteStage, ProgressWriterFailure};
        let coverage = |writer| ProgressCoverage::Sampled {
            observed: 3,
            queued: 2,
            dropped_before_queue: 1,
            acknowledged: 1,
            unacknowledged: 1,
            writer,
        };
        assert_eq!(
            uncertain_progress_queue(&coverage(ProgressWriterOutcome::Failed(
                ProgressWriterFailure {
                    stage: ProgressWriteStage::CanonicalAppend {}
                }
            ))),
            Some(2)
        );
        assert_eq!(
            uncertain_progress_queue(&coverage(ProgressWriterOutcome::JoinFailed)),
            Some(2)
        );
        assert_eq!(
            uncertain_progress_queue(&coverage(ProgressWriterOutcome::Failed(
                ProgressWriterFailure {
                    stage: ProgressWriteStage::Encoding {}
                }
            ))),
            None
        );
        assert_eq!(
            uncertain_progress_queue(&ProgressCoverage::Sampled {
                observed: 1,
                queued: 1,
                dropped_before_queue: 0,
                acknowledged: 1,
                unacknowledged: 0,
                writer: ProgressWriterOutcome::Complete
            }),
            None
        );
        assert_eq!(
            uncertain_progress_queue(&ProgressCoverage::Unknown {}),
            None
        );
    }
    fn event() -> Event {
        Event {
            source: None,
            phase: "counter".into(),
            elapsed: Duration::ZERO,
            values: [("counter".into(), Metric::Integer((1_i64 << 53) + 7))].into(),
            incumbent: None,
        }
    }
    fn tap() -> (
        Arc<Tap>,
        tokio::sync::mpsc::Receiver<super::super::ProgressEventDocument>,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        (Arc::new(Tap::new(Some(sender), None)), receiver)
    }
    fn loss(tap: &Tap, observed: u64, queued: u64, dropped: u64) {
        assert_eq!(
            tap.coverage(ProgressWriterOutcome::Complete),
            ProgressCoverage::Sampled {
                observed,
                queued,
                dropped_before_queue: dropped,
                acknowledged: 0,
                unacknowledged: queued,
                writer: ProgressWriterOutcome::Complete
            }
        );
    }
    #[test]
    fn ordinary_progress_synchronous_saturation_never_waits_for_receiver() {
        let (tap, mut receiver) = tap();
        let native = tap.clone();
        let (finished, received) = std::sync::mpsc::channel();
        let producer = std::thread::spawn(move || {
            for _ in 0..1000 {
                native.observe(&event());
            }
            finished.send(()).unwrap();
        });
        received.recv_timeout(Duration::from_secs(1)).unwrap();
        producer.join().unwrap();
        loss(&tap, 1000, 1, 999);
        let retained = receiver.try_recv().unwrap();
        assert_eq!(retained.sequence, Some(0));
        assert!(
            matches!(retained.values["counter"], super::super::ProgressMetricDocument::Integer(value) if value == (1_i64 << 53) + 7)
        );
    }
    #[test]
    fn ordinary_progress_concurrent_producers_preserve_recorded_sequence_order() {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(512);
        let tap = Arc::new(Tap::new(Some(sender), None));
        let barrier = Arc::new(std::sync::Barrier::new(8));
        let producers = (0..8)
            .map(|_| {
                let tap = tap.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    for _ in 0..1000 {
                        tap.observe(&event());
                    }
                })
            })
            .collect::<Vec<_>>();
        for producer in producers {
            producer.join().unwrap();
        }
        let mut sequences = Vec::new();
        while let Ok(event) = receiver.try_recv() {
            sequences.push(event.sequence.unwrap());
        }
        assert!(!sequences.is_empty());
        assert!(
            sequences.windows(2).all(|pair| pair[0] < pair[1]),
            "concurrent callbacks cannot overtake a retained observation"
        );
        let queued = sequences.len() as u64;
        loss(&tap, 8000, queued, 8000 - queued);
    }
    #[tokio::test]
    async fn ordinary_progress_async_saturation_and_closed_receiver_are_sampled_loss() {
        let (tap, receiver) = tap();
        tap.observe(&event());
        tap.observe(&event());
        drop(receiver);
        tap.observe(&event());
        loss(&tap, 3, 1, 2);
    }
    #[test]
    fn ordinary_progress_oversized_strings_maps_and_lock_contention_drop_before_clone() {
        let (tap, mut receiver) = tap();
        let mut large = event();
        large.phase = "x".repeat(PROGRESS_EVENT_BYTES);
        tap.observe(&large);
        large = event();
        large.values.insert(
            "text".into(),
            Metric::Text("x".repeat(PROGRESS_EVENT_BYTES)),
        );
        tap.observe(&large);
        large = event();
        large.values = (0..1024)
            .map(|index| (index.to_string(), Metric::Integer(index)))
            .collect();
        tap.observe(&large);
        let guard = tap.sender.lock().unwrap();
        tap.observe(&event());
        drop(guard);
        loss(&tap, 4, 0, 4);
        assert!(receiver.try_recv().is_err());
        // Rejected borrowed events do not consume a slot or an owned DTO.
        let mut scalar = event();
        scalar.incumbent = Some(IncumbentEvent {
            objective: -0.,
            dual_bound: None,
            gap: None,
            nodes: (1_i64 << 53) + 9,
            seconds: 0.,
            primal: Some(vec![1.; 100_000]),
        });
        tap.observe(&scalar);
        let retained = receiver.try_recv().unwrap();
        assert_eq!(retained.sequence, Some(4));
        assert!(serde_json::to_vec(&retained).unwrap().len() < 2048);
        assert_eq!(
            scalar
                .incumbent
                .as_ref()
                .unwrap()
                .primal
                .as_ref()
                .unwrap()
                .len(),
            100_000
        );
    }
    #[test]
    fn ordinary_progress_refused_memory_disables_queue_and_escaped_tap_retains_admission() {
        let tiny: Arc<dyn MemoryPool> = Arc::new(
            datafusion::execution::memory_pool::GreedyMemoryPool::new(1024),
        );
        assert!(progress_memory(&tiny, 1).is_none());
        assert_eq!(tiny.reserved(), 0);
        let disabled = Tap::new(None, None);
        disabled.observe(&event());
        assert_eq!(
            disabled.coverage(ProgressWriterOutcome::MemoryRefused),
            ProgressCoverage::Sampled {
                observed: 1,
                queued: 0,
                dropped_before_queue: 1,
                acknowledged: 0,
                unacknowledged: 0,
                writer: ProgressWriterOutcome::MemoryRefused
            }
        );
        let pool: Arc<dyn MemoryPool> = Arc::new(
            datafusion::execution::memory_pool::GreedyMemoryPool::new(16 << 20),
        );
        let memory = progress_memory(&pool, 1).unwrap();
        let admitted = pool.reserved();
        let tap = Arc::new(Tap::new(None, Some(memory)));
        let escaped = tap.clone();
        drop(tap);
        assert_eq!(pool.reserved(), admitted);
        drop(escaped);
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn ordinary_progress_counter_exhaustion_does_not_invent_complete_coverage() {
        let (tap, _receiver) = tap();
        tap.observed.store(u64::MAX, Ordering::Release);
        tap.observe(&event());
        assert_eq!(
            tap.coverage(ProgressWriterOutcome::Complete),
            ProgressCoverage::Unknown {}
        );
    }
    #[test]
    fn ordinary_progress_historical_completion_without_coverage_is_unknown() {
        let stored = StoredCompletion {
            version: 1,
            attempt_id: pse_operations::mint_id(),
            completion: None,
            termination: detail(
                TerminationCause::Assessment {
                    usable: false,
                    candidate_use: vec![],
                    diagnostics: vec![],
                },
                false,
            ),
            state: AttemptState::Failed,
            progress: ProgressCoverage::Unknown {},
        };
        let mut historical = serde_json::to_value(stored).unwrap();
        historical.as_object_mut().unwrap().remove("progress");
        let decoded: StoredCompletion = serde_json::from_value(historical).unwrap();
        assert_eq!(decoded.progress, ProgressCoverage::Unknown {});
    }
}

#[cfg(test)]
mod seed_allocation_tests {
    #![allow(
        clippy::unwrap_used,
        reason = "codec allocation controls require exact bounded round trips"
    )]
    use super::*;
    #[test]
    fn portable_seed_encoding_bound_covers_long_bit_patterns_and_vector_capacity() {
        const COUNT: usize = 10_000;
        let cases = [
            WarmPayload::Root(vec![-2.; COUNT]),
            WarmPayload::Nlp {
                primal: vec![-2.; COUNT],
                bounds: Some((vec![-2.; COUNT], vec![-2.; COUNT])),
                rows: Some(vec![-2.; COUNT]),
                barrier: Some(-2.),
                working: None,
            },
            WarmPayload::Highs {
                primal: Some(vec![-2.; COUNT]),
                dual: Some((vec![-2.; COUNT], vec![-2.; COUNT])),
                basis: Some(pse_backend_native::solve::Basis {
                    columns: vec![i32::MIN; COUNT],
                    rows: vec![i32::MIN; COUNT],
                }),
            },
        ];
        for original in cases {
            let extent = seed_extent(&original).unwrap();
            let document = SeedDocument {
                version: 1,
                warm: seed_payload(&original),
                prediction: None,
            };
            let numeric_capacity = match &document.warm {
                SeedPayload::Root { primal } => primal.capacity() * size_of::<u64>(),
                SeedPayload::Nlp {
                    primal,
                    bounds,
                    rows,
                    ..
                } => {
                    (primal.capacity()
                        + bounds
                            .as_ref()
                            .map_or(0, |(lower, upper)| lower.capacity() + upper.capacity())
                        + rows.as_ref().map_or(0, Vec::capacity))
                        * size_of::<u64>()
                }
                SeedPayload::Highs {
                    primal,
                    dual,
                    basis,
                } => {
                    (primal.as_ref().map_or(0, Vec::capacity)
                        + dual
                            .as_ref()
                            .map_or(0, |(columns, rows)| columns.capacity() + rows.capacity()))
                        * size_of::<u64>()
                        + basis.as_ref().map_or(0, |(columns, rows)| {
                            (columns.capacity() + rows.capacity()) * size_of::<i32>()
                        })
                }
            };
            let bytes = serde_json::to_vec(&document).unwrap();
            assert!(
                extent >= numeric_capacity + bytes.capacity(),
                "grant{extent} must cover simultaneous bit arrays{numeric_capacity} and JSON Vec capacity{}",
                bytes.capacity()
            );
            let reopened: SeedDocument = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(
                serde_json::to_value(seed_payload(&warm_payload(reopened.warm))).unwrap(),
                serde_json::to_value(&document.warm).unwrap()
            );
        }
    }
}

#[cfg(test)]
mod typed_store_failure_unit {
    use super::*;
    #[test]
    fn lost_worker_is_infrastructure_and_never_fabricates_effect_absence() {
        let error =
            WorkflowError::Canonical(pse_operations::canonical::CanonicalError::WorkerLost {
                run: "run".into(),
                attempt: "attempt".into(),
                generation: 7,
            });
        for cancelled in [false, true] {
            let outcome = failure(&error, cancelled);
            assert_eq!(
                outcome.state,
                if cancelled {
                    AttemptState::Cancelled
                } else {
                    AttemptState::Failed
                }
            );
            assert!(matches!(
                outcome.detail.cause,
                TerminationCause::Infrastructure { .. }
            ));
            assert!(outcome.retryable);
            assert_eq!(
                outcome.detail.effect,
                pse_model::study::EffectState::Unknown
            );
        }
    }
    #[test]
    fn a_refused_store_statement_does_not_prove_global_scientific_effect_absence() {
        use pse_operations::canonical::{CanonicalError, FailureEffect, RefusalCode};
        let code = RefusalCode::ExecutionAttemptFenceRevoked;
        let error = WorkflowError::Canonical(CanonicalError::Refused {
            code,
            class: code.class(),
            stage: "seal",
            effect: FailureEffect::StatementRefused,
        });
        let outcome = failure(&error, false);
        assert!(matches!(
            outcome.detail.cause,
            TerminationCause::Error { .. }
        ));
        assert!(!outcome.retryable);
        assert_eq!(
            outcome.detail.effect,
            pse_model::study::EffectState::Unknown
        );
    }
}
