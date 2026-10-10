// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Canonical claims precede native effects; immutable closed manifests own stored observations.
use super::{RunReport, RunRequest, RunResult, WorkflowError, contract};
use datafusion::execution::memory_pool::{MemoryConsumer, MemoryPool};
use pse_backend_native::solve::{Compatibility, Event, ProgressTap, WarmPayload, WarmStart};
use pse_ids::{ContentHash, Frame, FramedHasher};
use pse_model::generated::{
    enums::{AttemptState, CandidateUse, NativeBackend},
    identities::{AttemptId, RunId, SolutionId},
};
use pse_operations::{
    canonical::{CanonicalOptions, CanonicalStore, Revision},
    canonical_execution::{
        AttemptFence, CanonicalAttempt, CanonicalRun, ClosedAttempt, RESULT_BATCH_BYTES,
        ResultManifest, TerminalClass, execution_attempt_key, result_batch_key,
        result_payload_digest, result_set_key,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicI32, Ordering},
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
        Ok(self.store.cancel_run(run, operation).await?)
    }
    /// Bounded per-problem recorded execution summaries, excluding scientific payloads.
    pub async fn runs(
        &self,
        problem: &str,
        after: Option<u64>,
        limit: usize,
    ) -> Result<Vec<pse_operations::canonical_execution::RunSummary>, WorkflowError> {
        Ok(self.store.execution_run_page(problem, after, limit).await?)
    }
    /// Reopen the exact historical admitted completion and eligible seed inventory.
    pub async fn record(&self, run: &str, attempt: &str) -> Result<DurableRecord, WorkflowError> {
        let read = self
            .store
            .read_results(run, attempt, Duration::from_secs(60))
            .await?;
        let header: CompletionReceipt = serde_json::from_slice(
            read.attempt()
                .completion
                .as_ref()
                .ok_or_else(|| contract("terminal completion absent"))?
                .as_slice(),
        )
        .map_err(|e| contract(e.to_string()))?;
        let (completion, owner) = match header.inline {
            Some(completion) => {
                let bytes = serde_json::to_vec(&completion).map_err(|e| contract(e.to_string()))?;
                (*completion, reserve(&self.pool, bytes.len())?)
            }
            None => {
                let (bytes, owner) = read_chunks(
                    &self.store,
                    &self.pool,
                    &read,
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
        let completion = Arc::new(pse_columnar::Leased::new(Arc::new(completion), owner));
        let mut solutions = Vec::new();
        let mut after = None;
        loop {
            read.renew(Duration::from_secs(60)).await?;
            let page = self.store.result_seed_page(attempt, after).await?;
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
                        pse_ids::SemanticId::parse_hex(&seed.key)
                            .map_err(|error| contract(error.to_string()))?,
                    );
                    solutions.push((
                        usize::try_from(seed.step).map_err(|_| contract("stored step extent"))?,
                        solution,
                    ));
                }
            }
        }
        Ok(DurableRecord {
            attempt_id: completion.attempt_id,
            attempt_key: Some(read.attempt().key.clone()),
            run: Some(read.run().clone()),
            attempt: Ok(read.attempt().clone()),
            manifest: Some(read.manifest().clone()),
            completion: Some(completion),
            solutions,
        })
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
        heartbeat: Option<Heartbeat>,
    ) -> Result<DurableRecord, WorkflowError> {
        let prepared = async {
            let completion = serde_json::to_vec(receipt).map_err(|e| contract(e.to_string()))?;
            let manifest = self.store.reconcile_closed_attempt(closed).await?;
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
                .seal_attempt(&manifest, operation, class, &completion)
                .await
        }?;
        self.record(closed.fence().run(), closed.fence().attempt())
            .await
    }
    /// Recover only explicitly selected expired/cancelled run authority. No numerical dispatch.
    #[allow(
        unsafe_code,
        reason = "current recovery authority records truthful worker loss; never promotes arbitrary rows to scientific success"
    )]
    pub async fn recover(&self, run: &str, operation: &str) -> Result<Recovery, WorkflowError> {
        let current = self
            .store
            .canonical_run(run)
            .await?
            .ok_or_else(|| contract("recovery run absent"))?;
        let attempt = current
            .current_attempt
            .as_ref()
            .ok_or_else(|| contract("recovery attempt absent"))?;
        let actual = self
            .store
            .canonical_attempt(attempt)
            .await?
            .ok_or_else(|| contract("recovery attempt header absent"))?;
        if actual.terminal {
            return Ok(Recovery {
                recovered: vec![actual.key],
            });
        }
        let closed = self.store.recover_closed_attempt(run, operation).await?;
        let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
        hash.str("scientific.attempt.lineage.v1")
            .str(closed.fence().attempt());
        let attempt_id = AttemptId::from_id(hash.finish_id());
        let error =
            WorkflowError::Canonical(pse_operations::canonical::CanonicalError::Configuration(
                "worker authority expired before durable scientific completion".into(),
            ));
        let current = self
            .store
            .canonical_run(run)
            .await?
            .ok_or_else(|| contract("recovered run absent"))?;
        let cancelled = current.cancelled
            || self
                .store
                .canonical_study_for_run(run)
                .await?
                .is_some_and(|study| study.cancelled);
        let mut outcome = failure(&error, cancelled);
        outcome.retryable = true;
        outcome.detail.retry_failure = Some(pse_model::study::RetryFailure::Transient);
        let stored = StoredCompletion {
            version: 1,
            attempt_id,
            completion: None,
            termination: outcome.detail,
            state: outcome.state,
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
            None,
        )
        .await?;
        Ok(Recovery {
            recovered: vec![closed.fence().attempt().into()],
        })
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
    Durable(Box<DurableRecord>),
}
/// A completed canonical attempt and its exact immutable selection, or an observable write failure.
#[derive(Clone, Debug)]
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
#[derive(Debug)]
pub(crate) struct DurableAttempt {
    operations: Operations,
    attempt: AttemptId,
    run_id: Option<RunId>,
    claim_operation: String,
    fence: Arc<OnceLock<AttemptFence>>,
    run: OnceLock<CanonicalRun>,
    stream: Streamer,
    heartbeat: Option<Heartbeat>,
}
impl DurableAttempt {
    pub(super) fn new(operations: &Operations) -> Self {
        let attempt: AttemptId = pse_operations::mint_id();
        let fence = Arc::new(OnceLock::new());
        Self {
            operations: operations.clone(),
            attempt,
            run_id: None,
            claim_operation: format!("claim:{attempt}"),
            stream: Streamer::new(operations, fence.clone()),
            fence,
            run: OnceLock::new(),
            heartbeat: None,
        }
    }
    pub(crate) fn claimed(
        operations: &Operations,
        fence: AttemptFence,
        run_id: RunId,
        attempt: AttemptId,
    ) -> Self {
        let shared = Arc::new(OnceLock::new());
        let _ = shared.set(fence);
        Self {
            operations: operations.clone(),
            attempt,
            run_id: Some(run_id),
            claim_operation: format!("claimed:{attempt}"),
            stream: Streamer::new(operations, shared.clone()),
            fence: shared,
            run: OnceLock::new(),
            heartbeat: None,
        }
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
            let _ = self.run.set(row);
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
            let _ = self.run.set(row);
            self.stream.ready.notify_one();
            return Ok(());
        }
        let physical = &packages
            .first()
            .ok_or_else(|| contract("horizon source absent"))?
            .physical;
        let mut inputs = packages
            .iter()
            .map(|p| p.revision.canonical.clone())
            .collect::<Vec<_>>();
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
        let attestation = runtime.canonical.attestation();
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
        let _ = self.run.set(row);
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
    pub(crate) async fn start(&mut self, cancel: Canceller) -> Result<(), WorkflowError> {
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
        self.stream.tap.set_cancel(cancel.clone());
        self.heartbeat = Some(Heartbeat::spawn(&self.operations, fence, cancel));
        Ok(())
    }
    pub(super) async fn finish(mut self, result: &RunResult, cancelled: bool) -> DurableRecord {
        // Keep renewal alive through every export phase, including progress drain.
        let ingestion = async {
            self.stream.finish().await?;
            self.store_tables(result).await?;
            store_seeds(
                &self.operations,
                self.fence()?,
                result,
                #[cfg(all(test, feature = "canonical-tests"))]
                &self.stream.export_gate,
            )
            .await?;
            Ok::<(), WorkflowError>(())
        }
        .await;
        let tables_complete = ingestion.is_ok();
        let ingestion_error = ingestion.err().map(Arc::new);
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
        let stored = StoredCompletion {
            version: 1,
            attempt_id: self.attempt,
            completion: if tables_complete {
                result.completion().ok().cloned()
            } else {
                None
            },
            termination: outcome.detail.clone(),
            state: outcome.state,
        };
        match self.terminate(&stored).await {
            Ok(record) => record,
            Err(error) => self.reject_export(Arc::new(error)).await,
        }
    }
    pub(crate) async fn abandon(mut self, error: &Arc<WorkflowError>) -> DurableRecord {
        let stream = self.stream.finish().await;
        if self.fence.get().is_none() {
            return self.failed_record(error.clone());
        }
        let export_error = stream.err().map(Arc::new);
        let export_error = if is_canonical_failure(error) {
            export_failure(Some(error.clone()), export_error, self.fence.get())
        } else {
            export_error
        };
        if let Some(error) = &export_error
            && !self.cancelled_refusal(error)
        {
            return self.reject_export(error.clone()).await;
        }
        let outcome = failure(error, export_error.is_some());
        let stored = StoredCompletion {
            version: 1,
            attempt_id: self.attempt,
            completion: None,
            termination: outcome.detail.clone(),
            state: outcome.state,
        };
        match self.terminate(&stored).await {
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
    fn failed_record(&self, error: Arc<WorkflowError>) -> DurableRecord {
        DurableRecord {
            attempt_id: self.attempt,
            attempt_key: self.fence.get().map(|f| f.attempt().to_owned()),
            run: self.run.get().cloned(),
            attempt: Err(error),
            manifest: None,
            completion: None,
            solutions: Vec::new(),
        }
    }
    fn fence(&self) -> Result<&AttemptFence, WorkflowError> {
        self.fence
            .get()
            .ok_or_else(|| contract("canonical execution claim absent"))
    }
    async fn store_tables(&self, result: &RunResult) -> Result<(), WorkflowError> {
        let fence = self.fence()?.clone();
        for relation in result.table_ids() {
            let cursor = result
                .cursor_by_id(relation, 1024, super::ResultOrder::Canonical)
                .map_err(WorkflowError::Shared)?;
            #[cfg(all(test, feature = "canonical-tests"))]
            FinishExportGate::at(&self.stream.export_gate, FinishExportPhase::Table).await?;
            super::result_projection::store_result_table(
                &self.operations.store,
                &fence,
                relation,
                cursor,
                &self.operations.pool,
            )
            .await?;
        }
        Ok(())
    }
    async fn terminate(
        &mut self,
        stored: &StoredCompletion,
    ) -> Result<DurableRecord, WorkflowError> {
        let fence = self.fence()?.clone();
        let current = self
            .operations
            .store
            .canonical_attempt(fence.attempt())
            .await?
            .ok_or_else(|| contract("completion attempt absent"))?;
        if current.terminal {
            let _ = self.stop_heartbeat().await;
            return self.operations.record(fence.run(), fence.attempt()).await;
        }
        let current_run = self
            .operations
            .store
            .canonical_run(fence.run())
            .await?
            .ok_or_else(|| contract("completion run absent"))?;
        let mut stored = stored.clone();
        if current_run.cancelled {
            stored.state = AttemptState::Cancelled;
        }
        let bytes = serde_json::to_vec(&stored).map_err(|e| contract(e.to_string()))?;
        let mut receipt = if stored.state == AttemptState::Cancelled || !current.ingestion_open {
            inline_completion(stored.clone())
        } else {
            #[cfg(all(test, feature = "canonical-tests"))]
            FinishExportGate::at(&self.stream.export_gate, FinishExportPhase::Completion).await?;
            match write_chunks(&self.operations.store, &fence, "__completion", 0, &bytes).await {
                Ok(receipt) => receipt,
                Err(error) if self.cancelled_refusal(&error) => {
                    stored.state = AttemptState::Cancelled;
                    inline_completion(stored.clone())
                }
                Err(error) => return Err(error),
            }
        };
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
                    .await?;
            }
            self.operations
                .store
                .recover_closed_attempt(fence.run(), &format!("recovery:{}", fence.attempt()))
                .await?
        } else {
            match self
                .operations
                .store
                .close_result_ingestion(&fence, &format!("close:{}", fence.attempt()))
                .await
            {
                Ok(closed) => closed,
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
                        .await?
                }
                Err(error) => return Err(error.into()),
            }
        };
        let class = match stored.state {
            AttemptState::Completed => TerminalClass::Succeeded,
            AttemptState::Partial => TerminalClass::Partial,
            AttemptState::Cancelled => TerminalClass::Cancelled,
            _ => TerminalClass::Failed,
        };
        let heartbeat = self.heartbeat.take();
        match self
            .operations
            .settle(
                &closed,
                &format!("seal:{}", fence.attempt()),
                class,
                &receipt,
                heartbeat,
            )
            .await
        {
            Ok(record) => Ok(record),
            Err(error) if self.cancelled_refusal(&error) => {
                // Cancellation after intentional close has an acknowledged refusal.
                // Preserve the fully exported original assessments under recovery authority.
                stored.state = AttemptState::Cancelled;
                let closed = self
                    .operations
                    .store
                    .recover_closed_attempt(fence.run(), &format!("recovery:{}", fence.attempt()))
                    .await?;
                self.operations
                    .settle(
                        &closed,
                        &format!("seal-cancel:{}", fence.attempt()),
                        TerminalClass::Cancelled,
                        &inline_completion(stored),
                        None,
                    )
                    .await
            }
            Err(error) => Err(error),
        }
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
    let retryable = matches!(
        error,
        WorkflowError::Canonical(_)
            | WorkflowError::Math(crate::math::MathRuntimeError::Infrastructure(_))
    );
    Outcome {
        state: if cancelled {
            AttemptState::Cancelled
        } else {
            AttemptState::Failed
        },
        detail: detail(
            TerminationCause::Error {
                diagnostic: error.boundary_diagnostic(),
            },
            retryable,
        ),
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
        RunRequest::Modeling(steps) => steps
            .iter()
            .map(|s| s.source.revision.canonical.clone())
            .collect(),
        RunRequest::Simulation(s) => vec![s.source.revision.canonical.clone()],
        RunRequest::Fit(f) => vec![f.source.revision.canonical.clone()],
        #[cfg(feature = "solver-diffsol")]
        RunRequest::Shooting { problem, .. } => {
            vec![problem.simulation.source.revision.canonical.clone()]
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
        RunRequest::Modeling(steps) => steps
            .iter()
            .map(|step| {
                serde_json::to_value((&step.starts, step.solve.numerical_strategy()))
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
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(operations.policy.heartbeat);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {_=&mut stopped=>break,_=interval.tick()=>{if let Err(error)=operations.store.renew_attempt(&fence,operations.policy.lease).await{if let Ok(mut failure)=recorded.lock(){*failure=Some(Arc::new(WorkflowError::Canonical(error)));}cancel();break;}}}
            }
        });
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
struct Tap {
    step: AtomicI32,
    runtime: tokio::runtime::Handle,
    wait: Duration,
    sender: Mutex<Option<tokio::sync::mpsc::Sender<super::ProgressEventDocument>>>,
    overflow: AtomicBool,
    cancel: Mutex<Option<Canceller>>,
}
impl std::fmt::Debug for Tap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tap")
            .field("overflow", &self.overflow)
            .finish_non_exhaustive()
    }
}
impl Tap {
    fn set_cancel(&self, cancel: Canceller) {
        if self.overflow.load(Ordering::Acquire) {
            cancel();
        }
        if let Ok(mut slot) = self.cancel.lock() {
            *slot = Some(cancel);
        }
    }
}
impl ProgressTap for Tap {
    fn observe(&self, event: &Event) {
        let mut document = super::ProgressEventDocument::from(event);
        document.step = Some(self.step.load(Ordering::Acquire));
        document.at = Some(chrono::Utc::now().timestamp_micros());
        // Clone the bounded sender under its lock, then release every lock before waiting.
        let sender = self.sender.lock().ok().and_then(|sender| sender.clone());
        let Some(sender) = sender else {
            return;
        };
        let failed = match sender.try_send(document) {
            Ok(()) => false,
            Err(tokio::sync::mpsc::error::TrySendError::Full(document))
                if tokio::runtime::Handle::try_current().is_err() =>
            {
                // NativeMathService's isolated synchronous producer can yield to the
                // asynchronous writer without growing its queue. A stalled receiver
                // must not keep native cancellation waiting indefinitely.
                self.runtime.block_on(async {
                    !matches!(
                        tokio::time::timeout(self.wait, sender.send(document)).await,
                        Ok(Ok(()))
                    )
                })
            }
            Err(_) => true,
        };
        if failed {
            self.overflow.store(true, Ordering::Release);
            let cancel = self.cancel.lock().ok().and_then(|cancel| cancel.clone());
            if let Some(cancel) = cancel {
                cancel();
            }
        }
    }
}
#[derive(Debug)]
struct Streamer {
    tap: Arc<Tap>,
    ready: Arc<tokio::sync::Notify>,
    task: Option<tokio::task::JoinHandle<Result<(), WorkflowError>>>,
    #[cfg(all(test, feature = "canonical-tests"))]
    export_gate: Arc<OnceLock<FinishExportGate>>,
}
impl Streamer {
    fn new(operations: &Operations, fence: Arc<OnceLock<AttemptFence>>) -> Self {
        let capacity = operations.policy.batch.clamp(1, 512);
        let (sender, mut receiver) =
            tokio::sync::mpsc::channel::<super::ProgressEventDocument>(capacity);
        let tap = Arc::new(Tap {
            step: AtomicI32::new(0),
            runtime: tokio::runtime::Handle::current(),
            wait: operations.policy.heartbeat,
            sender: Mutex::new(Some(sender)),
            overflow: AtomicBool::new(false),
            cancel: Mutex::new(None),
        });
        let ready = Arc::new(tokio::sync::Notify::new());
        let signal = ready.clone();
        let operations = operations.clone();
        #[cfg(all(test, feature = "canonical-tests"))]
        let export_gate = Arc::new(OnceLock::new());
        #[cfg(all(test, feature = "canonical-tests"))]
        let writer_gate = export_gate.clone();
        let task = tokio::spawn(async move {
            while fence.get().is_none() {
                tokio::select! {_=signal.notified()=>{},event=receiver.recv()=>{if event.is_some(){return Err(contract("native progress observed before claim"));}return Ok(());}}
            }
            let fence = fence
                .get()
                .ok_or_else(|| contract("progress native claim absent"))?
                .clone();
            let (mut ordinal, mut sequence) = (0_u64, 0_i64);
            while let Some(event) = receiver.recv().await {
                let _owner = reserve(&operations.pool, RESULT_BATCH_BYTES)?;
                let mut events = Vec::with_capacity(capacity);
                events.push(event);
                let deadline = std::time::Instant::now()
                    .checked_add(operations.policy.flush)
                    .ok_or_else(|| contract("progress flush deadline extent"))?;
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
                for event in &mut events {
                    event.sequence = Some(sequence);
                    sequence = sequence
                        .checked_add(1)
                        .ok_or_else(|| contract("progress sequence overflow"))?;
                }
                let mut ranges = std::collections::VecDeque::from([(0, events.len())]);
                while let Some((start, end)) = ranges.pop_front() {
                    let mut output = ProgressBuffer {
                        bytes: Vec::new(),
                        full: false,
                    };
                    output
                        .bytes
                        .try_reserve_exact(RESULT_BATCH_BYTES)
                        .map_err(|_| contract("progress payload allocation refused"))?;
                    match serde_json::to_writer(&mut output, &events[start..end]) {
                        Ok(()) => {
                            #[cfg(all(test, feature = "canonical-tests"))]
                            FinishExportGate::at(&writer_gate, FinishExportPhase::Progress).await?;
                            operations
                                .store
                                .append_result_batch(
                                    &fence,
                                    &format!("progress:{}:{ordinal}", fence.attempt()),
                                    "__progress",
                                    ordinal,
                                    &output.bytes,
                                    (end - start) as u64,
                                )
                                .await?;
                            ordinal = ordinal
                                .checked_add(1)
                                .ok_or_else(|| contract("progress batch ordinal overflow"))?;
                        }
                        Err(_) if output.full && end - start > 1 => {
                            let middle = start + (end - start) / 2;
                            ranges.push_front((middle, end));
                            ranges.push_front((start, middle));
                        }
                        Err(_) if output.full => {
                            return Err(contract(
                                "one progress observation exceeds immutable batch extent",
                            ));
                        }
                        Err(error) => return Err(contract(error.to_string())),
                    }
                }
            }
            Ok(())
        });
        Self {
            tap,
            ready,
            task: Some(task),
            #[cfg(all(test, feature = "canonical-tests"))]
            export_gate,
        }
    }
    async fn finish(&mut self) -> Result<(), WorkflowError> {
        if let Ok(mut sender) = self.tap.sender.lock() {
            sender.take();
        }
        if let Some(task) = self.task.take() {
            task.await
                .map_err(|error| contract(format!("progress writer: {error}")))??;
        }
        if self.tap.overflow.load(Ordering::Acquire) {
            return Err(contract(
                "durable progress admission failed; scientific run cancelled",
            ));
        }
        Ok(())
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
) -> Result<CompletionReceipt, WorkflowError> {
    let mut count = 0;
    for (ordinal, chunk) in bytes.chunks(RESULT_BATCH_BYTES).enumerate() {
        store
            .append_result_batch(
                fence,
                &format!(
                    "chunks:{}:{name}:{}",
                    fence.attempt(),
                    first + ordinal as u64
                ),
                name,
                first + ordinal as u64,
                chunk,
                chunk.len() as u64,
            )
            .await?;
        count += 1;
    }
    Ok(CompletionReceipt {
        version: 1,
        batch_count: count,
        payload_bytes: bytes.len() as u64,
        digest: result_payload_digest(bytes),
        inline: None,
    })
}
#[allow(
    clippy::too_many_arguments,
    reason = "the protected result selection, chunk window, extent and digest are separate admission premises checked before pooled allocation"
)]
async fn read_chunks(
    store: &CanonicalStore,
    pool: &Arc<dyn MemoryPool>,
    read: &pse_operations::canonical_results::ResultRead,
    name: &str,
    first: u64,
    count: u64,
    length: u64,
    digest: &str,
    subset: bool,
) -> Result<(Vec<u8>, Arc<pse_columnar::AllocationLease>), WorkflowError> {
    let set = result_set_key(&read.attempt().key, name);
    let descriptor = read
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
    let extent = usize::try_from(length).map_err(|_| contract("stored extent overflow"))?;
    let owner = reserve(pool, extent)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(extent)
        .map_err(|_| contract("stored chunk allocation refused"))?;
    for ordinal in 0..count {
        read.renew(Duration::from_secs(60)).await?;
        let payload = store.result_payload(read, &set, first + ordinal).await?;
        let expected =
            (length - ordinal * RESULT_BATCH_BYTES as u64).min(RESULT_BATCH_BYTES as u64);
        if payload.batch.payload.len() as u64 != expected || payload.batch.row_count != expected {
            return Err(contract("stored chunk coverage differs"));
        }
        bytes.extend_from_slice(payload.batch.payload.as_slice());
    }
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
    #[cfg(all(test, feature = "canonical-tests"))] export_gate: &OnceLock<FinishExportGate>,
) -> Result<Vec<(usize, SolutionId)>, WorkflowError> {
    let (Ok(RunReport::Modeling(steps)), RunRequest::Modeling(requests)) =
        (result.report(), result.request())
    else {
        return Ok(Vec::new());
    };
    let mut stored = Vec::new();
    let mut next = 0_u64;
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
        let seed = seed.materialize(serialization)?;
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
        let receipt = write_chunks(&operations.store, fence, name, next, &bytes).await?;
        let set = result_set_key(fence.attempt(), name);
        let row = pse_model::generated::runtime::canonical_result_seeds::Row {
            key: solution.to_string(),
            batch: result_batch_key(fence.attempt(), &set, next),
            first_ordinal: next,
            result_set: set,
            attempt: fence.attempt().into(),
            run: fence.run().into(),
            layout: seed.warm().compatibility.layout.to_string(),
            preparation: preparation.to_string(),
            profile: seed.warm().compatibility.profile.to_string(),
            data: seed.warm().compatibility.data.to_string(),
            backend: seed.warm().compatibility.backend.as_str().into(),
            step: index as u64,
            batch_count: receipt.batch_count,
            payload_bytes: receipt.payload_bytes,
            digest: receipt.digest,
            run_sequence: run.sequence,
            attempt_generation: fence.generation(),
        };
        #[cfg(all(test, feature = "canonical-tests"))]
        FinishExportGate::at(export_gate, FinishExportPhase::Seed).await?;
        operations
            .store
            .register_result_seed(fence, &format!("seed:{}:{solution}", fence.attempt()), &row)
            .await?;
        stored.push((index, solution));
        next += receipt.batch_count;
    }
    Ok(stored)
}
impl Operations {
    async fn seed_document(
        &self,
        solution: SolutionId,
    ) -> Result<
        (
            pse_model::generated::runtime::canonical_result_seeds::Row,
            SeedDocument,
            Arc<pse_columnar::AllocationLease>,
        ),
        WorkflowError,
    > {
        let row = self
            .store
            .result_seed(&solution.to_string())
            .await?
            .ok_or(SeedReadError::Missing { solution })?;
        let read = self
            .store
            .read_results(&row.run, &row.attempt, Duration::from_secs(60))
            .await?;
        let completed = self.record(&row.run, &row.attempt).await?;
        let assessment = completed
            .completion
            .as_ref()
            .and_then(|c| c.completion.as_ref())
            .and_then(|c| c.assessments.get(row.step as usize))
            .ok_or_else(|| contract("stored seed scientific completion absent"))?;
        if !assessment.permits_seed {
            return Err(contract(
                "stored seed scientific completion did not permit reuse",
            ));
        }
        let name = "__seeds";
        if result_set_key(&row.attempt, name) != row.result_set {
            return Err(contract("stored seed set coordinates differ"));
        }
        let (bytes, owner) = read_chunks(
            &self.store,
            &self.pool,
            &read,
            name,
            row.first_ordinal,
            row.batch_count,
            row.payload_bytes,
            &row.digest,
            true,
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
        Ok((row, payload, owner))
    }
    /// One explicitly selected retained seed and its original compatibility stamps.
    pub async fn seed(
        &self,
        solution: SolutionId,
    ) -> Result<
        (
            pse_model::generated::runtime::canonical_result_seeds::Row,
            Arc<pse_columnar::Leased<WarmStart>>,
        ),
        WorkflowError,
    > {
        let (row, payload, owner) = self.seed_document(solution).await?;
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
        Ok((
            row,
            Arc::new(pse_columnar::Leased::new(Arc::new(start), owner)),
        ))
    }
    pub(crate) async fn prediction(
        &self,
        solution: SolutionId,
    ) -> Result<
        Option<(
            pse_model::generated::runtime::canonical_result_seeds::Row,
            Arc<pse_columnar::Leased<super::modeling::results::PortablePrediction>>,
        )>,
        WorkflowError,
    > {
        let (row, payload, owner) = self.seed_document(solution).await?;
        Ok(payload.prediction.map(|point| {
            (
                row,
                Arc::new(pse_columnar::Leased::new(Arc::new(point), owner)),
            )
        }))
    }
    /// Bounded eligible index pages use recorded order and then exact scientific admission.
    pub async fn latest_seed(
        &self,
        target: &Compatibility,
        preparation: &ContentHash,
        attempt: Option<&str>,
    ) -> Result<Option<SolutionId>, WorkflowError> {
        let rows = self
            .store
            .result_seed_candidates(
                &target.layout.to_string(),
                &preparation.to_string(),
                target.backend.as_str(),
                attempt,
            )
            .await?;
        for row in rows {
            let solution: SolutionId = pse_ids::SemanticId::parse_hex(&row.key)
                .map(SolutionId::from_id)
                .map_err(|e| contract(format!("seed identity: {e}")))?;
            let attempt = self
                .store
                .canonical_attempt(&row.attempt)
                .await?
                .ok_or_else(|| contract("seed attempt header absent"))?;
            if !attempt.terminal {
                continue;
            }
            self.seed(solution).await?;
            return Ok(Some(solution));
        }
        Ok(None)
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
impl Operations {
    /// Persist each registry physical relation independently; each immutable IPC object
    /// is bounded, and no unrelated relation is hydrated on replay.
    pub(super) async fn put_physical_rows(
        &self,
        runtime: &super::Runtime,
        physical: &super::PhysicalContext,
    ) -> Result<super::physical_cache::PhysicalRows, WorkflowError> {
        let generation = runtime.physical_cache.rows_generation();
        if let Some(revisions) = runtime.physical_cache.rows(runtime, self, physical) {
            return runtime
                .physical_cache
                .protected_rows(generation, runtime, self, physical, &revisions)
                .await;
        }
        let generation = runtime.physical_cache.begin_rows();
        let encoding =
            MemoryConsumer::new("canonical:physical-source-encoding").register(&self.pool);
        if !physical.sources.is_empty() {
            encoding
                .try_grow(16 * pse_operations::canonical::PAYLOAD_BYTES)
                .map_err(pse_engine::EngineError::from)?;
        }
        let mut revisions = Vec::new();
        for (relation, table) in &physical.sources {
            let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
            hash.str("physical.rows.v1").str(&relation.to_string());
            super::result_blocks::visit_source_blocks(table.batch(), |_, _, payload| {
                hash.part(&payload);
                Ok(())
            })?;
            let identity = hash.finish_hash();
            let problem = format!("physical:rows:{identity}");
            let saved = Arc::new(Mutex::new(None::<Revision>));
            let mut ordinal = 0_u64;
            super::result_blocks::visit_source_blocks_async(
                table.batch(),
                |start, rows, payload| {
                    let ordinal_here = ordinal;
                    ordinal += 1;
                    let store = self.store.clone();
                    let problem = problem.clone();
                    let saved = saved.clone();
                    let edit = source_edit(
                        format!("rows:{ordinal_here:020}"),
                        relation.to_string(),
                        format!("{ordinal_here:020}"),
                        "physical:rows:v1",
                        payload,
                    );
                    async move {
                        let _ = (start, rows);
                        let parent = saved
                            .lock()
                            .map_err(|_| contract("physical source receipt lock"))?
                            .as_ref()
                            .map(|r| r.key.clone());
                        let row = store
                            .edit(
                                &problem,
                                parent.as_deref(),
                                &format!("physical:{identity}:{ordinal_here}"),
                                &[edit],
                            )
                            .await?;
                        *saved
                            .lock()
                            .map_err(|_| contract("physical source receipt lock"))? = Some(row);
                        Ok::<(), WorkflowError>(())
                    }
                },
            )
            .await?;
            if let Some(revision) = saved
                .lock()
                .map_err(|_| contract("physical source receipt lock"))?
                .take()
            {
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
            self.store
                .edit(
                    &problem,
                    None,
                    &format!("physical:context:{}", physical.identity()),
                    &[edit],
                )
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
    }
}

fn reserve(
    pool: &Arc<dyn MemoryPool>,
    bytes: usize,
) -> Result<Arc<pse_columnar::AllocationLease>, WorkflowError> {
    let extent = bytes
        .checked_mul(16)
        .and_then(|n| n.checked_add(2 * RESULT_BATCH_BYTES))
        .ok_or_else(|| contract("scientific decode allocation extent overflow"))?;
    let reservation = MemoryConsumer::new("canonical:scientific-decode").register(pool);
    reservation
        .try_grow(extent)
        .map_err(pse_engine::EngineError::from)?;
    Ok(pse_columnar::AllocationLease::new(reservation))
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
                    &[7],
                    1,
                )
                .await
                .unwrap();
            let attempt_id = pse_operations::mint_id();
            let mut attempt =
                DurableAttempt::claimed(&operations, fence.clone(), result.run_id, attempt_id);
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
            assert!(retained.closed && !retained.ingestion_open);
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
                    !retained.terminal && record.manifest.is_none() && record.completion.is_none()
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
                assert!(retained.terminal);
                assert_eq!(record.attempt.as_ref().unwrap().key, fence.attempt());
                assert_eq!(
                    record.attempt.as_ref().unwrap().outcome.as_deref(),
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
                    .read_results(fence.run(), fence.attempt(), Duration::from_secs(60))
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
        runtime
            .canonical_store()
            .remove_isolated_fixture()
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn canonical_cancelled_prefix_and_late_heartbeat_reopen_acknowledged_unavailable_science()
    {
        let (operations, fence) = study_claimed().await;
        operations
            .store
            .append_result_batch(&fence, "prefix", "observations", 0, &[7], 1)
            .await
            .unwrap();
        let attempt_id = pse_operations::mint_id();
        let mut attempt = DurableAttempt::claimed(
            &operations,
            fence.clone(),
            pse_operations::mint_id(),
            attempt_id,
        );
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
        };
        attempt.stream.finish().await.unwrap();
        let record = attempt.terminate(&stored).await.unwrap();
        assert_eq!(record.attempt_id, attempt_id);
        assert_eq!(
            record.attempt.as_ref().unwrap().outcome.as_deref(),
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
        let repeated = attempt.terminate(&changed).await.unwrap();
        assert_eq!(repeated.attempt_id, attempt_id);
        assert!(repeated.completion.as_ref().unwrap().completion.is_none());
        assert_eq!(
            repeated.completion.as_ref().unwrap().state,
            AttemptState::Cancelled
        );
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
        };
        operations.store.cancel_study("cancel-study").await.unwrap();
        let refusal = operations
            .settle(
                &closed,
                "original-seal",
                TerminalClass::Failed,
                &inline_completion(original.clone()),
                None,
            )
            .await
            .unwrap_err();
        assert!(acknowledged_cancellation(&refusal, &fence));
        let mut attempt =
            DurableAttempt::claimed(&operations, fence, pse_operations::mint_id(), attempt_id);
        attempt.stream.finish().await.unwrap();
        let record = attempt.terminate(&original).await.unwrap();
        assert_eq!(
            record.completion.as_ref().unwrap().state,
            AttemptState::Cancelled
        );
        assert!(record.completion.as_ref().unwrap().completion.is_some());
        assert_eq!(record.attempt_id, attempt_id);
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
        );
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
        );
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
                .terminal
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
            );
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
                assert!(actual.closed && !actual.ingestion_open && !actual.terminal);
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
            attempt.stream.finish().await.unwrap();
            let settled = attempt.terminate(&original).await;
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
                        .terminal
                );
            } else {
                let record = settled.unwrap();
                assert_eq!(record.attempt_id, attempt_id);
                assert!(record.completion.as_ref().unwrap().completion.is_some());
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
    async fn admit(
        operations: &Operations,
        fence: &AttemptFence,
    ) -> pse_operations::canonical_results::ResultRead {
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
                .seal_attempt(&manifest, "codec-seal", TerminalClass::Failed, &[9])
                .await
        }
        .unwrap();
        operations
            .store
            .read_results(fence.run(), fence.attempt(), Duration::from_secs(60))
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn canonical_failed_export_prefix_remains_unsealed_and_unreadable() {
        use pse_relations::generated::{enums::NativeMetricKind, runtime::solve_metrics};
        let (operations, fence) = claimed().await;
        let runtime = super::super::durable_tests::durable_runtime();
        let registry = &runtime.registry;
        let validation = runtime.validation_context().unwrap();
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
                .map_err(|error| Arc::new(super::super::relation(error)))?;
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
                .map_err(|error| Arc::new(super::super::relation(error)))?;
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
        );
        attempt.stream.finish().await.unwrap();
        let record = attempt.reject_export(Arc::new(failure)).await;
        assert!(record.attempt.is_err());
        assert!(record.completion.is_none());
        let retained = operations
            .store
            .canonical_attempt(fence.attempt())
            .await
            .unwrap()
            .unwrap();
        assert!(!retained.ingestion_open);
        assert!(!retained.terminal);
        assert!(retained.closed_manifest.is_none());
        assert!(
            operations
                .store
                .read_results(fence.run(), fence.attempt(), Duration::from_secs(60))
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn canonical_seed_chunks_exact_ieee_bits_and_accounted_join() {
        let (operations, fence) = claimed().await;
        let original = (0..70_019)
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
        let first = write_chunks(&operations.store, &fence, "__seeds", 0, &bytes)
            .await
            .unwrap();
        let second = write_chunks(
            &operations.store,
            &fence,
            "__seeds",
            first.batch_count,
            &bytes,
        )
        .await
        .unwrap();
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
    async fn canonical_progress_batches_exact_integral_events_under_one_receipt() {
        let (mut operations, fence) = claimed().await;
        operations.policy.batch = 8;
        operations.policy.flush = Duration::from_millis(10);
        let admitted = Arc::new(OnceLock::new());
        admitted.set(fence.clone()).unwrap();
        let mut stream = Streamer::new(&operations, admitted);
        for index in 0..3 {
            stream.tap.observe(&Event {
                phase: "counter".into(),
                elapsed: Duration::from_millis(index as u64),
                values: BTreeMap::from([(
                    "counter".into(),
                    Metric::Integer((1_i64 << 53) + index),
                )]),
                incumbent: None,
            });
        }
        stream.finish().await.unwrap();
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
    async fn canonical_progress_native_burst_waits_for_writer_and_preserves_order() {
        let (mut operations, fence) = claimed().await;
        operations.policy.batch = 2;
        operations.policy.flush = Duration::from_millis(1);
        let admitted = Arc::new(OnceLock::new());
        admitted.set(fence.clone()).unwrap();
        let mut stream = Streamer::new(&operations, admitted);
        let tap = stream.tap.clone();
        let producer = std::thread::spawn(move || {
            for index in 0..12 {
                tap.observe(&Event {
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
        stream.finish().await.unwrap();
        assert!(!stream.tap.overflow.load(Ordering::Acquire));
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
        assert_eq!(events.len(), 12);
        for (index, event) in events.iter().enumerate() {
            assert_eq!(event.sequence, Some(index as i64));
            assert!(
                matches!(event.values["counter"], super::super::ProgressMetricDocument::Integer(value) if value == (1_i64 << 53) + index as i64)
            );
        }
    }
    fn isolated_tap(
        wait: Duration,
    ) -> (
        Arc<Tap>,
        tokio::sync::mpsc::Receiver<super::super::ProgressEventDocument>,
        Arc<AtomicBool>,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        let tap = Arc::new(Tap {
            step: AtomicI32::new(0),
            runtime: tokio::runtime::Handle::current(),
            wait,
            sender: Mutex::new(Some(sender)),
            overflow: AtomicBool::new(false),
            cancel: Mutex::new(None),
        });
        let flag = cancelled.clone();
        tap.set_cancel(Arc::new(move || {
            flag.store(true, Ordering::Release);
        }));
        (tap, receiver, cancelled)
    }
    #[tokio::test]
    async fn canonical_progress_stalled_receiver_bounds_native_cancellation_wait() {
        let (tap, receiver, cancelled) = isolated_tap(Duration::from_millis(20));
        let native = tap.clone();
        let producer = std::thread::spawn(move || {
            let event = Event {
                phase: "counter".into(),
                elapsed: Duration::ZERO,
                values: BTreeMap::new(),
                incumbent: None,
            };
            native.observe(&event);
            native.observe(&event);
        });
        // Keep the saturated receiver alive and unscheduled: receiver liveness alone
        // cannot hang a native run (including one cancelled while waiting).
        tokio::time::timeout(
            Duration::from_secs(1),
            tokio::task::spawn_blocking(move || producer.join().unwrap()),
        )
        .await
        .unwrap()
        .unwrap();
        assert!(cancelled.load(Ordering::Acquire));
        assert!(tap.overflow.load(Ordering::Acquire));
        drop(receiver);
    }
    #[tokio::test]
    async fn canonical_progress_closed_receiver_releases_native_producer() {
        let (tap, receiver, cancelled) = isolated_tap(Duration::from_secs(10));
        drop(receiver);
        let native = tap.clone();
        let producer = std::thread::spawn(move || {
            native.observe(&Event {
                phase: "counter".into(),
                elapsed: Duration::ZERO,
                values: BTreeMap::new(),
                incumbent: None,
            })
        });
        tokio::time::timeout(
            Duration::from_secs(1),
            tokio::task::spawn_blocking(move || producer.join().unwrap()),
        )
        .await
        .unwrap()
        .unwrap();
        assert!(cancelled.load(Ordering::Acquire));
        assert!(tap.overflow.load(Ordering::Acquire));
    }

    #[tokio::test]
    async fn canonical_progress_backpressure_cancels_and_retains_integral_observation() {
        let (mut operations, fence) = claimed().await;
        operations.policy.batch = 1;
        let admitted = Arc::new(OnceLock::new());
        admitted.set(fence.clone()).unwrap();
        let mut stream = Streamer::new(&operations, admitted);
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = cancelled.clone();
        stream.tap.set_cancel(Arc::new(move || {
            flag.store(true, Ordering::Release);
        }));
        let event = Event {
            phase: "counter".into(),
            elapsed: Duration::from_millis(1),
            values: BTreeMap::from([("counter".into(), Metric::Integer((1_i64 << 53) + 17))]),
            incumbent: None,
        };
        stream.tap.observe(&event);
        stream.tap.observe(&event);
        assert!(cancelled.load(Ordering::Acquire));
        assert!(stream.finish().await.is_err());
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
