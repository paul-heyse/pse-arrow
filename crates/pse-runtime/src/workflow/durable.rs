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
        AttemptFence, CanonicalAttempt, CanonicalRun, RESULT_BATCH_BYTES, ResultManifest,
        TerminalClass, execution_attempt_key, result_batch_key, result_payload_digest,
        result_set_key,
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
        let manifest = self.store.reconcile_closed_attempt(&closed).await?;
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
        // SAFETY: current native recovery authority froze and reconciled this exact
        // attempt; the receipt reports cancellation or worker loss without scientific success.
        unsafe {
            self.store
                .seal_attempt(
                    &manifest,
                    &format!("{operation}:terminal"),
                    class,
                    &serde_json::to_vec(&receipt).map_err(|e| contract(e.to_string()))?,
                )
                .await
        }?;
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
    Durable(Operations),
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
        let stream = self.stream.finish().await;
        let lost = self
            .heartbeat
            .as_ref()
            .is_some_and(|h| h.lost.load(Ordering::Acquire));
        let mut outcome = classify(result, cancelled);
        if lost {
            outcome = match self.heartbeat.as_ref().and_then(|heartbeat| {
                heartbeat
                    .failure
                    .lock()
                    .ok()
                    .and_then(|failure| failure.clone())
            }) {
                Some(error) => infrastructure(&error),
                None => infrastructure(&contract("durable lease renewal failed")),
            };
        }
        let mut solutions = Vec::new();
        let ingestion = async {
            stream?;
            self.store_tables(result).await?;
            solutions = store_seeds(&self.operations, self.fence()?, result).await?;
            Ok::<(), WorkflowError>(())
        }
        .await;
        if let Err(error) = &ingestion {
            outcome = infrastructure(error);
        }
        let stored = StoredCompletion {
            version: 1,
            attempt_id: self.attempt,
            completion: result.completion().ok().cloned(),
            termination: outcome.detail.clone(),
            state: outcome.state,
        };
        let terminal = self.terminate(&outcome, &stored).await;
        if let Some(heartbeat) = self.heartbeat.take() {
            heartbeat.stop().await;
        }
        let mut record = self.record(terminal, Some(stored), solutions).await;
        if let Err(error) = ingestion {
            // Cancellation revokes ingestion authority before native drain. An exact
            // acknowledged cancelled terminal settles that lifecycle; its stored
            // termination still retains the ingestion diagnostic. Other persistence
            // errors must remain errors, including an unacknowledged cancellation.
            let settled_cancellation = record.attempt.as_ref().is_ok_and(|attempt| {
                attempt.terminal
                    && attempt.closed
                    && attempt.outcome.as_deref() == Some("cancelled")
                    && record.attempt_key.as_deref() == Some(attempt.key.as_str())
                    && record
                        .run
                        .as_ref()
                        .is_some_and(|run| run.cancelled && run.key == attempt.run)
                    && record.manifest.is_some()
                    && record.completion.as_ref().is_some_and(|completion| {
                        completion.attempt_id == self.attempt
                            && completion.state == AttemptState::Cancelled
                    })
            });
            if !settled_cancellation {
                record.attempt = Err(Arc::new(error));
            }
        }
        record
    }
    pub(crate) async fn abandon(mut self, error: &Arc<WorkflowError>) -> DurableRecord {
        let _ = self.stream.finish().await;
        if self.fence.get().is_none() {
            return DurableRecord {
                attempt_id: self.attempt,
                attempt_key: None,
                run: self.run.get().cloned(),
                attempt: Err(error.clone()),
                manifest: None,
                completion: None,
                solutions: Vec::new(),
            };
        }
        let outcome = failure(error, false);
        let stored = StoredCompletion {
            version: 1,
            attempt_id: self.attempt,
            completion: None,
            termination: outcome.detail.clone(),
            state: outcome.state,
        };
        let terminal = self.terminate(&outcome, &stored).await;
        if let Some(heartbeat) = self.heartbeat.take() {
            heartbeat.stop().await;
        }
        self.record(terminal, Some(stored), Vec::new()).await
    }
    fn fence(&self) -> Result<&AttemptFence, WorkflowError> {
        self.fence
            .get()
            .ok_or_else(|| contract("canonical execution claim absent"))
    }
    async fn store_tables(&self, result: &RunResult) -> Result<(), WorkflowError> {
        let fence = self.fence()?.clone();
        for (relation, table) in result.tables().map_err(WorkflowError::Shared)? {
            super::result_projection::store_result_table(
                &self.operations.store,
                &fence,
                *relation,
                table,
                &self.operations.pool,
            )
            .await?;
        }
        Ok(())
    }
    #[allow(
        unsafe_code,
        reason = "controlled scientific owner admits original completed observations; no pointer or ABI operations"
    )]
    async fn terminate(
        &self,
        outcome: &Outcome,
        stored: &StoredCompletion,
    ) -> Result<(CanonicalAttempt, ResultManifest), WorkflowError> {
        let fence = self.fence()?;
        let current_run = self
            .operations
            .store
            .canonical_run(fence.run())
            .await?
            .ok_or_else(|| contract("completion run absent"))?;
        let mut outcome = outcome.clone();
        let mut stored = stored.clone();
        if current_run.cancelled {
            outcome.state = AttemptState::Cancelled;
            stored.state = AttemptState::Cancelled;
        }
        let bytes = serde_json::to_vec(&stored).map_err(|e| contract(e.to_string()))?;
        let current = self
            .operations
            .store
            .canonical_attempt(fence.attempt())
            .await?
            .ok_or_else(|| contract("completion attempt absent"))?;
        let mut receipt = if current.ingestion_open
            && current.expires_at > chrono::Utc::now().timestamp_micros()
        {
            match write_chunks(&self.operations.store, fence, "__completion", 0, &bytes).await {
                Ok(receipt) => receipt,
                Err(_) if outcome.state != AttemptState::Completed => CompletionReceipt {
                    version: 1,
                    batch_count: 0,
                    payload_bytes: 0,
                    digest: String::new(),
                    inline: Some(Box::new(stored.clone())),
                },
                Err(error) => return Err(error),
            }
        } else {
            if outcome.state == AttemptState::Completed {
                return Err(contract(
                    "scientifically completed run lost live durable authority",
                ));
            }
            CompletionReceipt {
                version: 1,
                batch_count: 0,
                payload_bytes: 0,
                digest: String::new(),
                inline: Some(Box::new(stored.clone())),
            }
        };
        let closed = if outcome.state == AttemptState::Cancelled {
            self.operations
                .store
                .cancel_run(fence.run(), &format!("cancel:{}", fence.attempt()))
                .await?;
            self.operations
                .store
                .recover_closed_attempt(fence.run(), &format!("recovery:{}", fence.attempt()))
                .await?
        } else {
            match self
                .operations
                .store
                .close_result_ingestion(fence, &format!("close:{}", fence.attempt()))
                .await
            {
                Ok(closed) => closed,
                Err(original) => {
                    match self
                        .operations
                        .store
                        .recover_closed_attempt(
                            fence.run(),
                            &format!("recovery:{}", fence.attempt()),
                        )
                        .await
                    {
                        Ok(closed) => closed,
                        Err(_) => return Err(original.into()),
                    }
                }
            }
        };
        let current_run = self
            .operations
            .store
            .canonical_run(fence.run())
            .await?
            .ok_or_else(|| contract("recovered completion run absent"))?;
        if current_run.cancelled {
            outcome.state = AttemptState::Cancelled;
            stored.state = AttemptState::Cancelled;
            receipt = CompletionReceipt {
                version: 1,
                batch_count: 0,
                payload_bytes: 0,
                digest: String::new(),
                inline: Some(Box::new(stored.clone())),
            };
        }
        let manifest = self
            .operations
            .store
            .reconcile_closed_attempt(&closed)
            .await?;
        let class = match outcome.state {
            AttemptState::Completed => TerminalClass::Succeeded,
            AttemptState::Partial => TerminalClass::Partial,
            AttemptState::Cancelled => TerminalClass::Cancelled,
            _ => TerminalClass::Failed,
        };
        // SAFETY: this scientific owner captured the actual completion and admitted
        // table projections; the reconciled manifest covers them exactly. Lost live
        // authority cannot admit success, and current cancellation is reflected above.
        let terminal = unsafe {
            self.operations
                .store
                .seal_attempt(
                    &manifest,
                    &format!("seal:{}", fence.attempt()),
                    class,
                    &serde_json::to_vec(&receipt).map_err(|e| contract(e.to_string()))?,
                )
                .await
        }?;
        Ok((terminal, manifest.row().clone()))
    }
    async fn record(
        &self,
        terminal: Result<(CanonicalAttempt, ResultManifest), WorkflowError>,
        completion: Option<StoredCompletion>,
        solutions: Vec<(usize, SolutionId)>,
    ) -> DurableRecord {
        let mut terminal = terminal;
        let run_key = self
            .fence
            .get()
            .map(AttemptFence::run)
            .or_else(|| self.run.get().map(|row| row.key.as_str()));
        let run = match run_key {
            Some(key) => match self.operations.store.canonical_run(key).await {
                Ok(row) => row,
                Err(error) => {
                    terminal = Err(error.into());
                    None
                }
            },
            None => None,
        };
        let completion = match completion {
            Some(mut completion) => {
                if terminal
                    .as_ref()
                    .is_ok_and(|(attempt, _)| attempt.outcome.as_deref() == Some("cancelled"))
                {
                    completion.state = AttemptState::Cancelled;
                }
                match serde_json::to_vec(&completion)
                    .map_err(|e| contract(e.to_string()))
                    .and_then(|bytes| reserve(&self.operations.pool, bytes.len()))
                {
                    Ok(owner) => Some(Arc::new(pse_columnar::Leased::new(
                        Arc::new(completion),
                        owner,
                    ))),
                    Err(error) => {
                        terminal = Err(error);
                        None
                    }
                }
            }
            None => None,
        };
        let (attempt, manifest) = match terminal {
            Ok((attempt, manifest)) => (Ok(attempt), Some(manifest)),
            Err(error) => (Err(Arc::new(error)), None),
        };
        DurableRecord {
            attempt_id: self.attempt,
            attempt_key: self.fence.get().map(|f| f.attempt().to_owned()),
            run,
            attempt,
            manifest,
            completion,
            solutions,
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
fn infrastructure(error: &WorkflowError) -> Outcome {
    let mut outcome = failure(error, false);
    outcome.detail.cause = TerminationCause::Infrastructure {
        diagnostic: error.boundary_diagnostic(),
    };
    outcome
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
#[derive(Debug)]
struct Heartbeat {
    stop: tokio::sync::oneshot::Sender<()>,
    task: tokio::task::JoinHandle<()>,
    lost: Arc<AtomicBool>,
    failure: Arc<Mutex<Option<Arc<WorkflowError>>>>,
}
impl Heartbeat {
    fn spawn(operations: &Operations, fence: AttemptFence, cancel: Canceller) -> Self {
        let (stop, mut stopped) = tokio::sync::oneshot::channel();
        let operations = operations.clone();
        let lost = Arc::new(AtomicBool::new(false));
        let flag = lost.clone();
        let failure = Arc::new(Mutex::new(None));
        let recorded = failure.clone();
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(operations.policy.heartbeat);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {_=&mut stopped=>break,_=interval.tick()=>{if let Err(error)=operations.store.renew_attempt(&fence,operations.policy.lease).await{if let Ok(mut failure)=recorded.lock(){*failure=Some(Arc::new(WorkflowError::Canonical(error)));}flag.store(true,Ordering::Release);cancel();break;}}}
            }
        });
        Self {
            stop,
            task,
            lost,
            failure,
        }
    }
    async fn stop(self) {
        let _ = self.stop.send(());
        let _ = self.task.await;
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
fn seed_extent(payload: &WarmPayload) -> Result<usize, WorkflowError> {
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
    count
        .checked_mul(32)
        .and_then(|n| n.checked_add(16 * 1024))
        .ok_or_else(|| contract("portable seed serialization extent"))
}
async fn store_seeds(
    operations: &Operations,
    fence: &AttemptFence,
    result: &RunResult,
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
        let crate::math::solves::Outcome::Native(native) = &step.outcome else {
            continue;
        };
        if !step.completion.decision.permits_seed() {
            continue;
        }
        let (Some(seed), Some(preparation)) = (
            native.warm_start.as_ref(),
            request.solve.seed_preparation_identity(),
        ) else {
            continue;
        };
        let prediction_extent = native
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
            .ok_or_else(|| contract("portable prediction extent"))?;
        let extent = seed_extent(&seed.payload)?
            .checked_add(prediction_extent)
            .ok_or_else(|| contract("portable seed document extent"))?;
        let _serialization = reserve(&operations.pool, extent)?;
        let solution: SolutionId = pse_operations::mint_id();
        let name = "__seeds";
        let bytes = serde_json::to_vec(&SeedDocument {
            version: 1,
            warm: seed_payload(&seed.payload),
            prediction: step.portable_prediction()?,
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
            layout: seed.compatibility.layout.to_string(),
            preparation: preparation.to_string(),
            profile: seed.compatibility.profile.to_string(),
            data: seed.compatibility.data.to_string(),
            backend: seed.compatibility.backend.as_str().into(),
            step: index as u64,
            batch_count: receipt.batch_count,
            payload_bytes: receipt.payload_bytes,
            digest: receipt.digest,
            run_sequence: run.sequence,
            attempt_generation: fence.generation(),
        };
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
    use super::*;
    use pse_backend_native::solve::Metric;
    use std::collections::BTreeMap;

    async fn claimed() -> (Operations, AttemptFence) {
        let runtime = super::super::durable_tests::durable_runtime();
        let Durability::Durable(operations) = runtime.durability() else {
            unreachable!()
        };
        let operations = operations.clone();
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
