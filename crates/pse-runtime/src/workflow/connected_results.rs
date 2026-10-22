// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact canonical result selections over protected native files. Original row
//! coordinates select recorded coverage; missing ranges stay missing.

#[cfg(test)]
use super::result_files;
use super::{Runtime, WorkflowError, contract, result_blocks};
use datafusion::arrow::{datatypes::SchemaRef, record_batch::RecordBatch};
use pse_columnar::{CancellationToken, MemoryConsumer, MemoryPool, MemoryReservation};
use pse_model::generated::runtime::canonical_result_blocks::Row as BlockMetadata;
use pse_operations::{canonical::MetadataResponse, canonical_results::ResultRead};
use std::{
    collections::{BTreeMap, VecDeque},
    future::Future,
    io::{BufWriter, Write},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

#[cfg(all(test, feature = "canonical-tests"))]
use std::time::Duration;

// The boxed child drops before its grant, including cancellation and an unpolled
// reader. Keeping the grant outside the child also retains it after Poll::Ready
// until the allocation itself is dropped.
struct PrepaidReaderFuture<'a, T> {
    future: futures_util::future::BoxFuture<'a, T>,
    _owner: MemoryReservation,
}

impl<T> Future for PrepaidReaderFuture<'_, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        self.get_mut().future.as_mut().poll(cx)
    }
}

fn prepaid_reader_future<'a, F>(
    pool: &Arc<dyn MemoryPool>,
    future: F,
) -> Result<PrepaidReaderFuture<'a, F::Output>, WorkflowError>
where
    F: Future + Send + 'a,
{
    let owner = MemoryConsumer::new("canonical:result-reader-future").register(pool);
    owner
        .try_grow(size_of_val(&future))
        .map_err(pse_engine::EngineError::from)?;
    Ok(PrepaidReaderFuture {
        future: Box::pin(future),
        _owner: owner,
    })
}

// Metadata transport and retained storage have independent real lifetimes.
pub(super) fn response_owner(
    runtime: &Runtime,
    name: &'static str,
    profile: MetadataResponse,
) -> Result<Arc<MemoryReservation>, WorkflowError> {
    pooled_response_owner(&runtime.shared.pool(), name, profile)
}

pub(super) fn pooled_response_owner(
    pool: &Arc<dyn MemoryPool>,
    name: &'static str,
    profile: MetadataResponse,
) -> Result<Arc<MemoryReservation>, WorkflowError> {
    let owner = MemoryConsumer::new(name).register(pool);
    owner
        .try_grow(profile.allocation_bytes())
        .map_err(pse_engine::EngineError::from)?;
    Ok(Arc::new(owner))
}

/// Transfer only an actually unique response grant. An outstanding SDK/router
/// hold prevents partitioning its allowance; that real overlap stays prepaid.
pub(super) fn retain_metadata(
    pool: &Arc<dyn MemoryPool>,
    response: Arc<MemoryReservation>,
    bytes: usize,
) -> Result<MemoryReservation, WorkflowError> {
    if bytes > response.size() {
        return Err(contract(
            "retained metadata exceeds admitted endpoint extent",
        ));
    }
    match Arc::try_unwrap(response) {
        Ok(response) => Ok(response.split(bytes)),
        Err(_live_response) => {
            let retained = MemoryConsumer::new("canonical:retained-metadata").register(pool);
            retained
                .try_grow(bytes)
                .map_err(pse_engine::EngineError::from)?;
            Ok(retained)
        }
    }
}

/// Retained protected rows also prepay their final release response. That small
/// grant follows the read into queued cleanup and uncertain-release recovery.
pub(super) fn retain_protected_metadata<F>(
    pool: &Arc<dyn MemoryPool>,
    response: Arc<MemoryReservation>,
    bytes: usize,
    attach_fallback: F,
) -> Result<MemoryReservation, WorkflowError>
where
    F: FnOnce(
        Arc<dyn std::fmt::Debug + Send + Sync>,
    ) -> Result<(), pse_operations::canonical::CanonicalError>,
{
    let Some(bytes) = bytes
        .checked_add(MetadataResponse::Protection.allocation_bytes())
        .filter(|bytes| *bytes <= response.size())
    else {
        attach_fallback(response)?;
        return Err(contract("protected metadata allocation extent"));
    };
    match Arc::try_unwrap(response) {
        Ok(response) => Ok(response.split(bytes)),
        Err(response) => {
            let retained =
                MemoryConsumer::new("canonical:retained-protected-metadata").register(pool);
            if let Err(source) = retained.try_grow(bytes) {
                // Admission can fail while the SDK still owns the original
                // response. Keep that whole grant through refused-read cleanup.
                attach_fallback(response)?;
                return Err(pse_engine::EngineError::from(source).into());
            }
            Ok(retained)
        }
    }
}

#[derive(Debug)]
struct OutputFilter {
    output: String,
    owner: pse_ids::SemanticId,
    field: String,
    partition: String,
    dense: bool,
    minimum: Option<f64>,
    maximum: Option<f64>,
    missing: Option<bool>,
    _owner: MemoryReservation,
}
#[derive(Debug)]
enum OutputRows {
    Scalar,
    Dense(pse_model::generated::runtime::canonical_result_block_outputs::Row),
}

/// Invocation-relative scientific admission for one selected reader. Transport
/// verification and protected read permission remain independent obligations.
#[derive(Debug)]
#[cfg(test)]
struct ResultAdmission(Arc<pse_relations::validate::ValidationContext>);
#[cfg(test)]
impl ResultAdmission {
    fn new(runtime: &Runtime) -> Self {
        Self(runtime.admission_context())
    }
    fn admit(
        &self,
        runtime: &Runtime,
        relation: pse_ids::SemanticId,
        owned: pse_columnar::owned_buffer::OwnedRecordBatch,
        cancel: &CancellationToken,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let spec = runtime
            .registry()
            .relation_by_id(relation)
            .ok_or_else(|| contract("result relation unavailable"))?;
        pse_relations::columnar::FieldCheckedBatch::admit_owned(
            runtime.registry(),
            spec,
            owned,
            &self.0,
            cancel,
        )
        .map_err(pse_engine::EngineError::from)
        .map_err(WorkflowError::from)
    }
}

/// Existing terminal selection alternatives, borrowed until read admission.
enum TerminalResultSelection<'a> {
    Exact {
        run: &'a str,
        attempt: &'a str,
    },
    Latest {
        problem: &'a str,
        classes: &'a [pse_operations::canonical_execution::TerminalClass],
    },
}

/// Full relation/range reader with a synchronous complete frozen inventory.
#[derive(Debug)]
pub struct CanonicalResultReader {
    reader: ResultReader,
    snapshot: super::ResultSnapshot,
}
impl CanonicalResultReader {
    fn from_reader(reader: ResultReader) -> Result<Self, WorkflowError> {
        let snapshot = reader
            .snapshot
            .clone()
            .ok_or_else(|| contract("full result snapshot absent"))?;
        Ok(Self { reader, snapshot })
    }
    /// Complete original full relation/range inventory, independent of progress.
    pub fn snapshot(&self) -> super::ResultSnapshot {
        self.snapshot.clone()
    }
    /// Exact full registry schema of the scientific relation.
    pub fn schema(&self) -> SchemaRef {
        self.reader.schema()
    }
    /// Original protected run/attempt/manifest, including partiality.
    pub fn selection(&self) -> &ResultRead {
        self.reader.selection()
    }
    /// Cancellation shared with this stream.
    pub fn cancellation_token(&self) -> CancellationToken {
        self.reader.cancellation_token()
    }
    /// Stop unread work and release transient metadata.
    pub fn close(&mut self) {
        self.reader.close();
    }
    /// Export all remaining selected rows atomically on success.
    pub async fn export_ipc(&mut self, destination: &std::path::Path) -> Result<(), WorkflowError> {
        self.reader.export_ipc(destination).await
    }
    /// Original complete selection coordinates for a portable review member.
    pub fn pack_selection(
        &self,
        usability: super::PackScientificState,
    ) -> Result<super::PackResultSelection, WorkflowError> {
        self.reader.pack_selection(usability)
    }
    pub(crate) fn validate_pack_selection(
        &self,
        selection: &super::PackResultSelection,
    ) -> Result<(), WorkflowError> {
        self.reader.validate_pack_selection(selection)
    }
    /// Export the unread protected selection with a finite output allowance and
    /// an independent caller cancellation token. No final file on failure.
    pub async fn export_ipc_bounded(
        &mut self,
        destination: &std::path::Path,
        maximum_bytes: u64,
        cancel: &CancellationToken,
    ) -> Result<u64, WorkflowError> {
        self.reader
            .export_ipc_bounded(destination, maximum_bytes, cancel)
            .await
    }
    pub(crate) async fn export_ipc_acknowledged(
        &mut self,
        destination: &std::path::Path,
        maximum_bytes: u64,
        cancel: &CancellationToken,
        acknowledged: &mut u64,
    ) -> Result<u64, WorkflowError> {
        self.reader
            .export_ipc_with_prefix(destination, maximum_bytes, cancel, Some(acknowledged))
            .await
    }
    /// Decode and admit the next selected native Arrow batch.
    pub async fn next_batch(&mut self) -> Result<Option<RecordBatch>, WorkflowError> {
        self.reader.next_batch().await
    }
    /// Decode and admit the next selected registry relation batch.
    pub async fn next_relation_batch(
        &mut self,
    ) -> Result<Option<pse_relations::columnar::FieldCheckedBatch>, WorkflowError> {
        self.reader.next_relation_batch().await
    }
}

/// Explicit selected stream. A reusable selected universe requires asynchronous freezing.
#[derive(Debug)]
pub struct SelectedResultReader {
    reader: ResultReader,
}
impl SelectedResultReader {
    /// Freeze the complete original selected universe, irrespective of stream
    /// progress. A fresh cancellation token controls this independent operation.
    pub async fn selected_snapshot(
        &self,
        cancel: CancellationToken,
    ) -> Result<super::SelectedResultSnapshot, WorkflowError> {
        let activity = self.reader.runtime.admit_activity()?;
        let pool = self.reader.runtime.shared.pool();
        let reader = &self.reader;
        activity
            .scope(prepaid_reader_future(&pool, async move {
                cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                let selection_owner = MemoryConsumer::new("canonical:selected-snapshot-key")
                    .register(&reader.runtime.shared.pool());
                selection_owner
                    .try_grow(reader.set.capacity())
                    .map_err(pse_engine::EngineError::from)?;
                let mut fresh = ResultReader {
                    observation: None,
                    runtime: reader.runtime.clone(),
                    read: reader.read.clone(),
                    snapshot: None,
                    decode: reader.decode.clone(),
                    read_owner: reader.read_owner.clone(),
                    relation: reader.relation,
                    set: reader.set.clone(),
                    schema: reader.schema.clone(),
                    start: reader.start,
                    end: reader.end,
                    after: None,
                    snapshot_position: 0,
                    pending: VecDeque::new(),
                    exhausted: reader.start == reader.end,
                    consumed: false,
                    cancel,
                    output: reader.output.clone(),
                    indexes: BTreeMap::new(),
                    descriptors: BTreeMap::new(),
                    descriptor_owner: None,
                    selected_batch: None,
                    metadata_owner: None,
                    _selection_owner: selection_owner,
                };
                let mut builder =
                    super::result_snapshot::SelectedSnapshotBuilder::new(fresh.decode.clone())?;
                loop {
                    let cancel = fresh.cancel.clone();
                    let next = prepaid_reader_future(
                        &fresh.runtime.shared.pool(),
                        fresh.next_checked_body(Some(&mut builder)),
                    )?;
                    let batch = cancel
                        .until_cancelled(next)
                        .await
                        .map_err(pse_engine::EngineError::from)??;
                    if batch.is_none() {
                        break;
                    }
                }
                Ok(builder.finish())
            })?)
            .await
    }
    /// Exact full registry schema of the scientific relation.
    pub fn schema(&self) -> SchemaRef {
        self.reader.schema()
    }
    /// Original protected run/attempt/manifest, including partiality.
    pub fn selection(&self) -> &ResultRead {
        self.reader.selection()
    }
    /// Cancellation shared with this stream.
    pub fn cancellation_token(&self) -> CancellationToken {
        self.reader.cancellation_token()
    }
    /// Stop unread work and release transient metadata.
    pub fn close(&mut self) {
        self.reader.close();
    }
    /// Export all remaining selected rows atomically on success.
    pub async fn export_ipc(&mut self, destination: &std::path::Path) -> Result<(), WorkflowError> {
        self.reader.export_ipc(destination).await
    }
    /// Original complete selection coordinates for a portable review member.
    pub fn pack_selection(
        &self,
        usability: super::PackScientificState,
    ) -> Result<super::PackResultSelection, WorkflowError> {
        self.reader.pack_selection(usability)
    }
    pub(crate) fn validate_pack_selection(
        &self,
        selection: &super::PackResultSelection,
    ) -> Result<(), WorkflowError> {
        self.reader.validate_pack_selection(selection)
    }
    /// Export the unread protected selection with a finite output allowance and
    /// an independent caller cancellation token. No final file on failure.
    pub async fn export_ipc_bounded(
        &mut self,
        destination: &std::path::Path,
        maximum_bytes: u64,
        cancel: &CancellationToken,
    ) -> Result<u64, WorkflowError> {
        self.reader
            .export_ipc_bounded(destination, maximum_bytes, cancel)
            .await
    }
    pub(crate) async fn export_ipc_acknowledged(
        &mut self,
        destination: &std::path::Path,
        maximum_bytes: u64,
        cancel: &CancellationToken,
        acknowledged: &mut u64,
    ) -> Result<u64, WorkflowError> {
        self.reader
            .export_ipc_with_prefix(destination, maximum_bytes, cancel, Some(acknowledged))
            .await
    }
    /// Decode and admit the next selected native Arrow batch.
    pub async fn next_batch(&mut self) -> Result<Option<RecordBatch>, WorkflowError> {
        self.reader.next_batch().await
    }
    /// Decode and admit the next selected registry relation batch.
    pub async fn next_relation_batch(
        &mut self,
    ) -> Result<Option<pse_relations::columnar::FieldCheckedBatch>, WorkflowError> {
        self.reader.next_relation_batch().await
    }
}

#[derive(Debug)]
struct SelectedBatch {
    block: BlockMetadata,
    descriptor: pse_model::generated::runtime::canonical_result_batches::Row,
    batch: pse_relations::columnar::FieldCheckedBatch,
    _owner: MemoryReservation,
}

struct BoundedExportFile {
    file: std::fs::File,
    remaining: u64,
    written: u64,
    cancel: CancellationToken,
}
impl Write for BoundedExportFile {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.cancel.checkpoint().map_err(std::io::Error::other)?;
        if bytes.len() as u64 > self.remaining {
            return Err(std::io::Error::other(
                "review export output allowance exhausted",
            ));
        }
        let written = self.file.write(bytes)?;
        self.remaining -= written as u64;
        self.written += written as u64;
        Ok(written)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

/// A protected terminal attempt, exact relation and requested row coordinates.
#[derive(Debug)]
struct ResultReader {
    /// Related lazy access; it owns no scientific execution or cancellation authority.
    observation: Option<pse_diagnostics::observation::OperationGuard>,
    runtime: Runtime,
    read: Arc<ResultRead>,
    snapshot: Option<super::ResultSnapshot>,
    decode: Arc<super::result_snapshot::ProtectedResultDecode>,
    read_owner: Arc<MemoryReservation>,
    relation: pse_ids::SemanticId,
    set: String,
    schema: SchemaRef,
    start: u64,
    end: u64,
    after: Option<u64>,
    snapshot_position: usize,
    pending: VecDeque<BlockMetadata>,
    exhausted: bool,
    consumed: bool,
    cancel: CancellationToken,
    output: Option<Arc<OutputFilter>>,
    indexes: BTreeMap<String, OutputRows>,
    descriptors: BTreeMap<String, pse_model::generated::runtime::canonical_result_batches::Row>,
    descriptor_owner: Option<MemoryReservation>,
    selected_batch: Option<SelectedBatch>,
    // Drop all metadata-backed pages and indexes before their shared grant.
    metadata_owner: Option<MemoryReservation>,
    _selection_owner: MemoryReservation,
}
impl ResultReader {
    /// Exact registry schema of the selected scientific relation.
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }
    /// Actual recorded terminal attempt, including failure/partiality class.
    fn selection(&self) -> &ResultRead {
        &self.read
    }
    /// Cancellation shared with this typed consumer.
    fn cancellation_token(&self) -> CancellationToken {
        self.cancel.clone()
    }
    /// Stop unread database work; returned arrays own their decoded buffers.
    fn close(&mut self) {
        if let Some(operation) = self.observation.take() {
            operation.finish(pse_diagnostics::observation::Outcome::Interrupted);
        }
        self.cancel.cancel();
        self.pending = VecDeque::new();
        self.indexes = BTreeMap::new();
        self.descriptors = BTreeMap::new();
        self.descriptor_owner = None;
        self.selected_batch = None;
        self.metadata_owner = None;
        self.exhausted = true;
    }

    fn pack_selection(
        &self,
        usability: super::PackScientificState,
    ) -> Result<super::PackResultSelection, WorkflowError> {
        let spec = self
            .runtime
            .registry()
            .relation_by_id(self.relation)
            .ok_or_else(|| contract("selected relation absent"))?;
        Ok(super::PackResultSelection {
            run: self.read.run().key.clone(),
            attempt: self.read.attempt().key.clone(),
            relation: spec.key.qualified_name(),
            schema_version: spec.key.version,
            row_start: self.start,
            row_end: self.end,
            field: self.output.as_ref().map(|o| o.field.clone()),
            terminal_class: self
                .read
                .attempt()
                .outcome
                .map(|outcome| outcome.as_str().to_owned())
                .ok_or_else(|| contract("terminal class absent"))?,
            predicate: self.output.as_ref().map(|o| super::PackResultPredicate {
                output: o.output.clone(),
                partition: o.partition.clone(),
                minimum_bits: o.minimum.map(f64::to_bits),
                maximum_bits: o.maximum.map(f64::to_bits),
                missing: o.missing,
            }),
            usability,
        })
    }
    fn validate_pack_selection(
        &self,
        selection: &super::PackResultSelection,
    ) -> Result<(), WorkflowError> {
        let original = self.pack_selection(selection.usability)?;
        if original.run != selection.run
            || original.attempt != selection.attempt
            || original.relation != selection.relation
            || original.schema_version != selection.schema_version
            || original.row_start != selection.row_start
            || original.row_end != selection.row_end
            || original.field != selection.field
            || original.terminal_class != selection.terminal_class
            || original.predicate != selection.predicate
        {
            return Err(contract(
                "declared review selection differs from protected result coordinates",
            ));
        }
        Ok(())
    }
    /// Publish a local Arrow IPC export only after every selected database read
    /// and final IPC write succeeds. Interrupted staging remains `.incomplete`;
    /// creating the final hard link refuses an existing destination atomically.
    async fn export_ipc(&mut self, destination: &std::path::Path) -> Result<(), WorkflowError> {
        let cancel = self.cancel.clone();
        self.export_ipc_bounded(destination, u64::MAX, &cancel)
            .await
            .map(|_| ())
    }
    async fn export_ipc_bounded(
        &mut self,
        destination: &std::path::Path,
        maximum_bytes: u64,
        cancel: &CancellationToken,
    ) -> Result<u64, WorkflowError> {
        self.export_ipc_with_prefix(destination, maximum_bytes, cancel, None)
            .await
    }
    async fn export_ipc_with_prefix(
        &mut self,
        destination: &std::path::Path,
        maximum_bytes: u64,
        cancel: &CancellationToken,
        mut acknowledged: Option<&mut u64>,
    ) -> Result<u64, WorkflowError> {
        if self.consumed {
            return Err(contract("export requires an unread complete selection"));
        }
        let mut staging = destination.as_os_str().to_os_string();
        staging.push(".incomplete");
        let staging = std::path::PathBuf::from(staging);
        let io = |source| pse_engine::EngineError::Infrastructure {
            op: "publish canonical result IPC export".into(),
            source: Box::new(source),
        };
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging)
            .map_err(io)?;
        let mut metadata = self.schema.metadata().clone();
        let relation = self
            .runtime
            .registry()
            .relation_by_id(self.relation)
            .ok_or_else(|| contract("selected relation absent"))?;
        metadata.insert(
            "pse.canonical.relation".into(),
            relation.key.qualified_name(),
        );
        metadata.insert(
            "pse.canonical.relation-version".into(),
            relation.key.version.to_string(),
        );
        metadata.insert(
            "pse.canonical.problem".into(),
            self.read.run().problem.clone(),
        );
        metadata.insert(
            "pse.canonical.revision".into(),
            self.read.run().revision.clone(),
        );
        metadata.insert("pse.canonical.run".into(), self.read.run().key.clone());
        metadata.insert(
            "pse.canonical.attempt".into(),
            self.read.attempt().key.clone(),
        );
        metadata.insert(
            "pse.canonical.terminal-class".into(),
            self.read
                .attempt()
                .outcome
                .map(|outcome| outcome.as_str().to_owned())
                .ok_or_else(|| contract("terminal result class absent"))?,
        );
        metadata.insert(
            "pse.canonical.manifest".into(),
            self.read.manifest().digest.clone(),
        );
        metadata.insert(
            "pse.canonical.interpretation".into(),
            self.read.run().interpretation.clone(),
        );
        metadata.insert("pse.canonical.row-start".into(), self.start.to_string());
        metadata.insert("pse.canonical.row-end".into(), self.end.to_string());
        if let Some(filter) = &self.output {
            metadata.insert("pse.canonical.output".into(), filter.output.clone());
            metadata.insert("pse.canonical.field".into(), filter.field.clone());
            metadata.insert("pse.canonical.partition".into(), filter.partition.clone());
            if let Some(value) = filter.minimum {
                metadata.insert(
                    "pse.canonical.minimum-bits".into(),
                    value.to_bits().to_string(),
                );
            }
            if let Some(value) = filter.maximum {
                metadata.insert(
                    "pse.canonical.maximum-bits".into(),
                    value.to_bits().to_string(),
                );
            }
            if let Some(value) = filter.missing {
                metadata.insert("pse.canonical.missing".into(), value.to_string());
            }
        }
        let schema = Arc::new(self.schema.as_ref().clone().with_metadata(metadata));
        let mut writer = datafusion::arrow::ipc::writer::StreamWriter::try_new(
            BufWriter::new(BoundedExportFile {
                file,
                remaining: maximum_bytes,
                written: 0,
                cancel: cancel.clone(),
            }),
            schema.as_ref(),
        )
        .map_err(result_blocks::ResultBlockError::from)?;
        if let Some(prefix) = acknowledged.as_deref_mut() {
            writer.get_mut().flush().map_err(io)?;
            *prefix = writer.get_ref().get_ref().written;
        }
        loop {
            cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
            let Some(batch) = cancel
                .until_cancelled(self.next_batch())
                .await
                .map_err(pse_engine::EngineError::from)??
            else {
                break;
            };
            let batch = batch
                .with_schema(schema.clone())
                .map_err(result_blocks::ResultBlockError::from)?;
            writer
                .write(&batch)
                .map_err(result_blocks::ResultBlockError::from)?;
            if let Some(prefix) = acknowledged.as_deref_mut() {
                writer.get_mut().flush().map_err(io)?;
                *prefix = writer.get_ref().get_ref().written;
            }
        }
        self.cancel
            .checkpoint()
            .map_err(pse_engine::EngineError::from)?;
        writer
            .finish()
            .map_err(result_blocks::ResultBlockError::from)?;
        writer.get_mut().flush().map_err(io)?;
        writer.get_ref().get_ref().file.sync_all().map_err(io)?;
        let bytes = writer.get_ref().get_ref().written;
        if let Some(prefix) = acknowledged {
            *prefix = bytes;
        }
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        drop(writer);
        std::fs::hard_link(&staging, destination).map_err(io)?;
        std::fs::remove_file(staging).map_err(io)?;
        Ok(bytes)
    }

    /// Decode one admitted member from its exact protected native file.
    /// A late failure is surfaced to the stream; it cannot become normal exhaustion.
    async fn next_batch(&mut self) -> Result<Option<RecordBatch>, WorkflowError> {
        let result = self
            .next_relation_batch()
            .await?
            .map(|batch| {
                batch
                    .checked_export(&self.cancel)
                    .map_err(pse_engine::EngineError::from)
                    .map_err(Into::into)
            })
            .transpose();
        if result.is_err()
            && let Some(operation) = self.observation.take()
        {
            operation.finish(pse_diagnostics::observation::Outcome::Failed);
        }
        result
    }
    /// Return one admitted registry relation batch for native columnar algorithms.
    /// Shared snapshots retain exact protection for future file reads. Copied
    /// decoded buffers retain allocation owners independently of the reader.
    async fn next_relation_batch(
        &mut self,
    ) -> Result<Option<pse_relations::columnar::FieldCheckedBatch>, WorkflowError> {
        if let Err(error) = self.cancel.checkpoint() {
            if let Some(operation) = self.observation.take() {
                operation.finish(pse_diagnostics::observation::Outcome::Cancelled);
            }
            self.close();
            return Err(pse_engine::EngineError::from(error).into());
        }
        self.consumed = true;
        if self.pending.is_empty()
            && self.exhausted
            && self.indexes.is_empty()
            && self.descriptors.is_empty()
        {
            if let Some(operation) = self.observation.take() {
                operation.finish(pse_diagnostics::observation::Outcome::Completed);
            }
            self.pending = VecDeque::new();
            self.indexes = BTreeMap::new();
            self.descriptors = BTreeMap::new();
            self.metadata_owner = None;
            self.descriptor_owner = None;
            self.selected_batch = None;
            return Ok(None);
        }
        // Interrupt the original RPC future, rather than waiting for another
        // between-block checkpoint. The SDK's CancelOnDrop cancels its exact
        // RequestContext; its driver retains uncertain dispatched work through
        // physical drain. This asserts no rollback of a late pin renewal.
        let cancel = self.cancel.clone();
        let outcome = match self.next_checked() {
            Ok(future) => cancel
                .until_cancelled(future)
                .await
                .map_err(pse_engine::EngineError::from)
                .map_err(WorkflowError::from)
                .and_then(std::convert::identity),
            Err(error) => Err(error),
        };
        if outcome.is_err() {
            if let Some(operation) = self.observation.take() {
                operation.finish(pse_diagnostics::observation::Outcome::Failed);
            }
            self.close();
        }
        outcome
    }
    fn retained_page_bytes(&self) -> Result<usize, WorkflowError> {
        use pse_model::HeapUsage;
        let mut bytes = self
            .pending
            .capacity()
            .checked_mul(size_of::<BlockMetadata>())
            .ok_or_else(|| contract("result page extent"))?;
        for block in &self.pending {
            bytes = bytes
                .checked_add(block.heap_bytes())
                .ok_or_else(|| contract("result page extent"))?;
        }
        for (key, rows) in &self.indexes {
            let heap = match rows {
                OutputRows::Scalar => Some(0),
                OutputRows::Dense(row) => Some(row.heap_bytes()),
            }
            .ok_or_else(|| contract("result index extent"))?;
            bytes = bytes
                .checked_add(size_of::<(String, OutputRows)>() + 128)
                .and_then(|bytes| bytes.checked_add(key.capacity()))
                .and_then(|bytes| bytes.checked_add(heap))
                .ok_or_else(|| contract("result index extent"))?;
        }
        Ok(bytes)
    }
    #[inline(never)]
    fn next_checked(
        &mut self,
    ) -> Result<
        PrepaidReaderFuture<
            '_,
            Result<Option<pse_relations::columnar::FieldCheckedBatch>, WorkflowError>,
        >,
        WorkflowError,
    > {
        // Construct and erase the large borrowed file-read state here, before
        // polling. Cancellation wrappers then retain only this small owner;
        // their forwarding futures cannot embed another concrete child state.
        let pool = self.runtime.shared.pool();
        prepaid_reader_future(&pool, self.next_checked_body(None))
    }

    async fn next_checked_body(
        &mut self,
        mut frozen: Option<&mut super::result_snapshot::SelectedSnapshotBuilder>,
    ) -> Result<Option<pse_relations::columnar::FieldCheckedBatch>, WorkflowError> {
        // Full reads consume the already admitted immutable inventory. Borrow its
        // descriptors directly: aliases share metadata and its original charge.
        while self.output.is_none() {
            let snapshot = self
                .snapshot
                .as_ref()
                .ok_or_else(|| contract("full result snapshot absent"))?;
            let Some(metadata) = snapshot.blocks().get(self.snapshot_position) else {
                self.exhausted = true;
                return Ok(None);
            };
            if metadata.columns != self.schema.fields().len() as u64 {
                return Err(contract("recorded result block shape differs"));
            }
            let checked = snapshot.read_block(metadata, &self.cancel).await?;
            let lower = self.start.max(metadata.start);
            let upper = self.end.min(metadata.end);
            self.snapshot_position += 1;
            self.exhausted = self.snapshot_position == snapshot.blocks().len();
            if lower >= upper {
                continue;
            }
            let offset = usize::try_from(lower - metadata.start)
                .map_err(|_| contract("result coordinate exceeds platform"))?;
            let length = usize::try_from(upper - lower)
                .map_err(|_| contract("result coordinate exceeds platform"))?;
            let checked = checked.slice(offset, length).map_err(super::relation)?;
            return Ok(Some(checked));
        }
        loop {
            if self.pending.is_empty() {
                if !self.indexes.is_empty() || !self.descriptors.is_empty() {
                    return Err(contract("selected metadata has unconsumed coordinates"));
                }
                self.pending = VecDeque::new();
                self.indexes = BTreeMap::new();
                self.descriptors = BTreeMap::new();
                self.metadata_owner = None;
                if self.exhausted {
                    self.descriptor_owner = None;
                    self.selected_batch = None;
                    return Ok(None);
                }
                let owner = response_owner(
                    &self.runtime,
                    "canonical:result-metadata",
                    if self.output.is_some() {
                        MetadataResponse::OutputPage
                    } else {
                        MetadataResponse::BlockPage
                    },
                )?;
                let page = pse_operations::canonical::response_ownership_scope(
                    Arc::new((self.read.clone(), self.read_owner.clone(), owner.clone())),
                    async {
                        let page = if let Some(filter) = &self.output {
                            if let Some(context) = pse_diagnostics::observation::current() {
                                context.count("canonical.result.output_page_requests", "count", 1);
                            }
                            if filter.dense {
                                let page = self
                                    .runtime
                                    .canonical_store()?
                                    .result_output_page(
                                        &self.read,
                                        &self.set,
                                        &filter.output,
                                        &filter.partition,
                                        self.start,
                                        self.end,
                                        self.after,
                                        filter.minimum,
                                        filter.maximum,
                                    )
                                    .await?;
                                self.exhausted = page.outputs.len() < 64;
                                self.after = page
                                    .outputs
                                    .last()
                                    .map(|output| output.start)
                                    .or(self.after);
                                for output in page.outputs {
                                    if self
                                        .indexes
                                        .insert(output.batch.clone(), OutputRows::Dense(output))
                                        .is_some()
                                    {
                                        return Err(contract("duplicate dense output index"));
                                    }
                                }
                                page.blocks
                            } else {
                                let page = self
                                    .runtime
                                    .canonical_store()?
                                    .result_block_page(
                                        &self.read,
                                        &self.set,
                                        &self.relation.to_string(),
                                        "0",
                                        self.start,
                                        self.end,
                                        self.after,
                                    )
                                    .await?;
                                self.exhausted = page.len() < 64;
                                self.after = page.last().map(|block| block.ordinal).or(self.after);
                                for block in &page {
                                    if self
                                        .indexes
                                        .insert(block.batch.clone(), OutputRows::Scalar)
                                        .is_some()
                                    {
                                        return Err(contract("duplicate scalar block inventory"));
                                    }
                                }
                                page
                            }
                        } else {
                            return Err(contract("selected result filter absent"));
                        };
                        Ok::<_, WorkflowError>(page)
                    },
                )
                .await?;
                self.descriptors.clear();
                self.descriptor_owner = None;
                if !page.is_empty() {
                    let ordinals = page.iter().map(|block| block.ordinal).collect::<Vec<_>>();
                    let descriptor_response = response_owner(
                        &self.runtime,
                        "canonical:selected-result-descriptors",
                        MetadataResponse::Page,
                    )?;
                    if let Some(context) = pse_diagnostics::observation::current() {
                        context.count("canonical.result.descriptor_page_requests", "count", 1);
                    }
                    let descriptors = pse_operations::canonical::response_ownership_scope(
                        Arc::new((
                            self.read.clone(),
                            self.read_owner.clone(),
                            descriptor_response.clone(),
                        )),
                        self.runtime
                            .canonical_store()?
                            .result_file_batch_descriptors(&self.read, &self.set, &ordinals),
                    )
                    .await?;
                    if descriptors.len() != page.len() {
                        return Err(contract("selected descriptor coverage differs"));
                    }
                    let mut bytes = 0usize;
                    for (block, descriptor) in page.iter().zip(descriptors) {
                        use pse_model::HeapUsage;
                        if descriptor.key != block.batch || descriptor.ordinal != block.ordinal {
                            return Err(contract("selected descriptor coordinate differs"));
                        }
                        bytes = bytes
                            .checked_add(
                                size_of_val(&descriptor)
                                    + descriptor.heap_bytes()
                                    + descriptor.key.capacity()
                                    + 128,
                            )
                            .ok_or_else(|| contract("selected descriptor extent"))?;
                        if self
                            .descriptors
                            .insert(descriptor.key.clone(), descriptor)
                            .is_some()
                        {
                            return Err(contract("duplicate selected batch descriptor"));
                        }
                    }
                    self.descriptor_owner = Some(retain_metadata(
                        &self.runtime.shared.pool(),
                        descriptor_response,
                        bytes,
                    )?);
                }
                self.pending = page.into();
                self.metadata_owner = Some(retain_metadata(
                    &self.runtime.shared.pool(),
                    owner,
                    self.retained_page_bytes()?,
                )?);
                if self.pending.is_empty() {
                    continue;
                }
            }
            let metadata = self
                .pending
                .pop_front()
                .ok_or_else(|| contract("result metadata page empty"))?;
            self.cancel
                .checkpoint()
                .map_err(pse_engine::EngineError::from)?;
            if metadata.columns != self.schema.fields().len() as u64 {
                return Err(contract("recorded result block shape differs"));
            }
            let descriptor = self
                .descriptors
                .remove(&metadata.batch)
                .ok_or_else(|| contract("selected batch descriptor absent"))?;
            let expression = self
                .output
                .as_ref()
                .filter(|filter| !filter.dense)
                .map(|filter| {
                    super::result_projection::scalar_selection_expression(
                        self.relation,
                        filter.owner,
                        &filter.partition,
                        &filter.field,
                        filter.minimum,
                        filter.maximum,
                        filter.missing,
                    )
                })
                .transpose()?;
            if let Some(expression) = &expression
                && !super::result_snapshot::block_may_match(
                    &self.runtime,
                    self.relation,
                    self.schema.clone(),
                    &metadata,
                    &descriptor,
                    expression,
                )?
            {
                self.indexes.remove(&metadata.batch);
                self.selected_batch = None;
                self.decode
                    .validate_pruned_block(&metadata, &descriptor, &self.cancel)
                    .await?;
                continue;
            }
            let cached = self
                .selected_batch
                .as_ref()
                .filter(|cached| cached.block == metadata && cached.descriptor == descriptor)
                .map(|cached| cached.batch.clone());
            let checked = if let Some(checked) = cached {
                checked
            } else {
                use pse_model::HeapUsage;
                self.selected_batch = None;
                let owner = MemoryConsumer::new("canonical:selected-current-batch")
                    .register(&self.runtime.shared.pool());
                owner
                    .try_grow(
                        size_of::<SelectedBatch>()
                            + metadata.heap_bytes()
                            + descriptor.heap_bytes(),
                    )
                    .map_err(pse_engine::EngineError::from)?;
                let checked = self
                    .decode
                    .read_block(&metadata, &descriptor, &self.cancel)
                    .await?;
                self.selected_batch = Some(SelectedBatch {
                    block: metadata.clone(),
                    descriptor: descriptor.clone(),
                    batch: checked.clone(),
                    _owner: owner,
                });
                checked
            };
            let lower = self.start.max(metadata.start);
            let upper = self.end.min(metadata.end);
            if lower >= upper {
                continue;
            }
            let checked = if let Some(filter) = &self.output {
                let indexes = self
                    .indexes
                    .remove(&metadata.batch)
                    .ok_or_else(|| contract("selected output block lacks index"))?;
                let offsets_owner = MemoryConsumer::new("canonical:selected-source-offsets")
                    .register(&self.runtime.shared.pool());
                let maximum_offsets = match &indexes {
                    OutputRows::Scalar => checked.batch().num_rows(),
                    OutputRows::Dense(_) => checked.batch().num_rows(),
                };
                offsets_owner
                    .try_grow(
                        maximum_offsets
                            .checked_mul(size_of::<u32>())
                            .and_then(|bytes| {
                                let predicate = if matches!(&indexes, OutputRows::Scalar) {
                                    // Native Boolean/CASE/comparison temporaries span the
                                    // original complete batch, even for one selected owner.
                                    checked.batch().num_rows().checked_mul(64)?
                                } else {
                                    0
                                };
                                bytes.checked_add(predicate)
                            })
                            .ok_or_else(|| contract("selected offset allocation extent"))?,
                    )
                    .map_err(pse_engine::EngineError::from)?;
                let indices = match indexes {
                    OutputRows::Scalar => {
                        let expression = expression.ok_or_else(|| {
                            contract("original scalar selection expression absent")
                        })?;
                        let mask = super::result_projection::scalar_selection_mask(
                            &self.runtime.bound_model().native_state(),
                            &checked,
                            expression,
                        )?;
                        let mut indices = Vec::with_capacity(checked.batch().num_rows());
                        for offset in 0..checked.batch().num_rows() {
                            use datafusion::arrow::array::Array;
                            let coordinate = metadata
                                .start
                                .checked_add(offset as u64)
                                .ok_or_else(|| contract("original scalar coordinate extent"))?;
                            if coordinate >= lower
                                && coordinate < upper
                                && mask.is_valid(offset)
                                && mask.value(offset)
                            {
                                indices.push(u32::try_from(offset).map_err(|_| {
                                    contract("selected row exceeds Arrow take extent")
                                })?);
                            }
                        }
                        indices
                    }
                    OutputRows::Dense(output) => {
                        let view=pse_relations::generated::runtime::simulation_samples::View::from_checked(&checked).map_err(super::relation)?;
                        let first = output
                            .start
                            .checked_sub(metadata.start)
                            .and_then(|n| usize::try_from(n).ok())
                            .ok_or_else(|| {
                                contract("dense output starts outside original block")
                            })?;
                        let last = output
                            .end
                            .checked_sub(metadata.start)
                            .and_then(|n| usize::try_from(n).ok())
                            .filter(|last| first < *last && *last <= view.len())
                            .ok_or_else(|| contract("dense output ends outside original block"))?;
                        for neighbor in [first.checked_sub(1), (last < view.len()).then_some(last)]
                            .into_iter()
                            .flatten()
                        {
                            let value = view.row(neighbor).map_err(super::relation)?;
                            if format!("{}:{}:value", self.relation, value.symbol_id)
                                == filter.output
                            {
                                return Err(contract(
                                    "dense output omitted original adjacent rows",
                                ));
                            }
                        }
                        let mut indices = Vec::with_capacity(last - first);
                        let mut min = f64::INFINITY;
                        let mut max = f64::NEG_INFINITY;
                        for row in first..last {
                            let value = view.row(row).map_err(super::relation)?;
                            if format!("{}:{}:value", self.relation, value.symbol_id)
                                != filter.output
                            {
                                return Err(contract(
                                    "dense output range differs from original scientific rows",
                                ));
                            }
                            min = min.min(value.time);
                            max = max.max(value.time);
                            let coordinate = metadata.start + row as u64;
                            if coordinate >= self.start
                                && coordinate < self.end
                                && filter.minimum.is_none_or(|lower| value.time >= lower)
                                && filter.maximum.is_none_or(|upper| value.time <= upper)
                            {
                                indices.push(u32::try_from(row).map_err(|_| {
                                    contract("selected trajectory row exceeds Arrow take extent")
                                })?);
                            }
                        }
                        if output.coordinate_min != Some(min) || output.coordinate_max != Some(max)
                        {
                            return Err(contract(
                                "dense coordinate projection differs from exact recorded time",
                            ));
                        }
                        indices
                    }
                };
                if indices.is_empty() {
                    continue;
                }
                let rows =
                    super::result_snapshot::SelectedRows::from_offsets(indices, offsets_owner)?;
                let selected = rows.apply(&checked, &self.runtime.shared.pool(), &self.cancel)?;
                if let Some(builder) = frozen.as_deref_mut() {
                    builder.push(&metadata, &descriptor, rows)?;
                }
                selected
            } else {
                checked
            };
            return Ok(Some(checked));
        }
    }
}
impl Runtime {
    /// Bounded exact-attempt metadata preflight for review publication. It never polls rows.
    pub(crate) async fn review_result_selections(
        &self,
        requests: &[super::ReviewScientificSelection],
        cancel: &CancellationToken,
    ) -> Result<(Vec<super::PackResultSelection>, MemoryReservation), WorkflowError> {
        use super::{PackResultPredicate, PackResultSelection, PackScientificState};
        if requests.len() > 1024 {
            return Err(contract("review selects more than 1024 scientific members"));
        }
        let reservation = MemoryConsumer::new("review:scientific-selection-preflight")
            .register(&self.shared.pool());
        let extent = requests
            .iter()
            .try_fold(0usize, |total, request| {
                let output = request.output.as_ref().map_or(0, |output| {
                    output.field.len().saturating_add(output.partition.len())
                });
                total.checked_add(2048).and_then(|n| {
                    n.checked_add(
                        request
                            .run
                            .len()
                            .saturating_add(request.attempt.len())
                            .saturating_add(request.relation.len())
                            .saturating_add(request.member.len())
                            .saturating_add(output)
                            .saturating_mul(4),
                    )
                })
            })
            .ok_or_else(|| contract("review selection extent overflow"))?;
        reservation
            .try_grow(extent)
            .map_err(pse_engine::EngineError::from)?;
        let mut attempts =
            BTreeMap::<(String, String), (String, std::collections::BTreeSet<String>)>::new();
        let mut selected = Vec::with_capacity(requests.len());
        for request in requests {
            cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
            if request.row_start > request.row_end {
                return Err(contract("review result row range is reversed"));
            }
            let key = (request.run.clone(), request.attempt.clone());
            if !attempts.contains_key(&key) {
                let response = response_owner(
                    self,
                    "review:exact-result-metadata",
                    MetadataResponse::Selection,
                )?;
                let read = pse_operations::canonical::response_ownership_scope(
                    response.clone(),
                    self.canonical_store()?
                        .read_results(&request.run, &request.attempt),
                )
                .await?;
                let _owner = retain_protected_metadata(
                    &self.shared.pool(),
                    response,
                    read.retained_bytes()?,
                    |owner| read.retain_metadata_owner(owner),
                )?;
                let class = read
                    .attempt()
                    .outcome
                    .map(|outcome| outcome.as_str().to_owned())
                    .ok_or_else(|| contract("review requires an exact terminal attempt"))?;
                let copied_metadata = read
                    .sets()
                    .iter()
                    .try_fold(class.len(), |total, set| {
                        total.checked_add(256)?.checked_add(set.name.len())
                    })
                    .ok_or_else(|| contract("review relation metadata extent overflow"))?;
                reservation
                    .try_grow(copied_metadata)
                    .map_err(pse_engine::EngineError::from)?;
                let relations = read.sets().iter().map(|set| set.name.clone()).collect();
                attempts.insert(key.clone(), (class, relations));
                // ResultRead's protected metadata/borrower is dropped here, before the next attempt.
            }
            let (terminal_class, relations) = attempts
                .get(&key)
                .ok_or_else(|| contract("review preflight attempt absent"))?;
            let spec = self
                .registry()
                .relation(&request.relation)
                .ok_or_else(|| contract("unknown selected review relation"))?;
            if !relations.contains(&spec.id.to_string()) {
                return Err(contract("review relation is not part of the exact attempt"));
            }
            let (field, predicate) = if let Some(output) = &request.output {
                output.bounds()?;
                super::result_projection::check_output_field(spec.id, &output.field)?;
                let dense =
                    spec.id == pse_relations::generated::runtime::simulation_samples::RELATION_ID;
                if dense && output.missing.is_some() {
                    return Err(contract("dense review output has no missing predicate"));
                }
                let mut identity = String::with_capacity(66 + output.field.len());
                write_output_identity(&mut identity, &spec.id, output.owner, &output.field, dense)?;
                (
                    Some(output.field.clone()),
                    Some(PackResultPredicate {
                        output: identity,
                        partition: output.partition.clone(),
                        minimum_bits: output.minimum_bits,
                        maximum_bits: output.maximum_bits,
                        missing: output.missing,
                    }),
                )
            } else {
                (None, None)
            };
            selected.push(PackResultSelection {
                run: request.run.clone(),
                attempt: request.attempt.clone(),
                relation: spec.key.qualified_name(),
                schema_version: spec.key.version,
                row_start: request.row_start,
                row_end: request.row_end,
                field,
                terminal_class: terminal_class.clone(),
                predicate,
                usability: PackScientificState::Unknown,
            });
        }
        Ok((selected, reservation))
    }
    /// Read the exact canonical run header as its registry-declared Arrow row.
    pub async fn run_record(
        &self,
        run: &str,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("result.read");
                use pse_relations::generated::runtime::canonical_runs as rows;
                let extent =
                    response_owner(self, "canonical:result-header", MetadataResponse::Header)?;
                let row = pse_operations::canonical::response_ownership_scope(
                    extent.clone(),
                    self.canonical_store()?.canonical_run(run),
                )
                .await?
                .ok_or_else(|| contract("canonical run absent"))?;
                let validation = self.validation_context();
                let mut builder = rows::Builder::with_registry(self.registry(), 1, &validation)
                    .map_err(super::relation)?;
                builder.push(row).map_err(super::relation)?;
                builder
                    .finish()
                    .map_err(super::relation)?
                    .retained(&self.shared.pool(), &CancellationToken::new())
                    .map_err(super::relation)
            })
            .await
    }
    /// Read the actual attempt's recorded lifecycle and completion without substitution.
    pub async fn attempt_record(
        &self,
        attempt: &str,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("result.read");
                use pse_relations::generated::runtime::canonical_attempts as rows;
                let extent =
                    response_owner(self, "canonical:result-header", MetadataResponse::Header)?;
                let row = pse_operations::canonical::response_ownership_scope(
                    extent.clone(),
                    self.canonical_store()?.canonical_attempt(attempt),
                )
                .await?
                .ok_or_else(|| contract("canonical attempt absent"))?;
                let validation = self.validation_context();
                let mut builder = rows::Builder::with_registry(self.registry(), 1, &validation)
                    .map_err(super::relation)?;
                builder.push(row).map_err(super::relation)?;
                builder
                    .finish()
                    .map_err(super::relation)?
                    .retained(&self.shared.pool(), &CancellationToken::new())
                    .map_err(super::relation)
            })
            .await
    }
    /// Read one admitted terminal attempt's exact closed manifest as a typed row.
    pub async fn result_manifest(
        &self,
        attempt: &str,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("result.read");
                use pse_relations::generated::runtime::canonical_result_manifests as rows;
                let extent =
                    response_owner(self, "canonical:result-header", MetadataResponse::Header)?;
                let store = self.canonical_store()?;
                let read =
                    pse_operations::canonical::response_ownership_scope(extent.clone(), async {
                        let selected = store
                            .canonical_attempt(attempt)
                            .await?
                            .ok_or_else(|| contract("canonical attempt absent"))?;
                        store
                            .read_results(&selected.run, attempt)
                            .await
                            .map_err(WorkflowError::from)
                    })
                    .await?;
                let read_owner = Arc::new(retain_protected_metadata(
                    &self.shared.pool(),
                    extent.clone(),
                    read.retained_bytes()?,
                    |owner| read.retain_metadata_owner(owner),
                )?);
                read.retain_metadata_owner(read_owner)?;
                let validation = self.validation_context();
                let mut builder = rows::Builder::with_registry(self.registry(), 1, &validation)
                    .map_err(super::relation)?;
                builder
                    .push(read.manifest().clone())
                    .map_err(super::relation)?;
                builder
                    .finish()
                    .map_err(super::relation)?
                    .retained(&self.shared.pool(), &CancellationToken::new())
                    .map_err(super::relation)
            })
            .await
    }
    /// Select a problem's newest admitted terminal run under explicit outcome classes.
    pub async fn latest_results(
        &self,
        problem: &str,
        classes: &[pse_operations::canonical_execution::TerminalClass],
        relation: &str,
        start: u64,
        end: u64,
        cancel: CancellationToken,
    ) -> Result<CanonicalResultReader, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("result.read");
                cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                let extent = response_owner(
                    self,
                    "canonical:result-selection",
                    MetadataResponse::Selection,
                )?;
                self.result_reader_body(
                    TerminalResultSelection::Latest { problem, classes },
                    relation,
                    start,
                    end,
                    cancel,
                    extent,
                    None,
                )?
                .await
                .and_then(CanonicalResultReader::from_reader)
            })
            .await
    }
    /// Reopen one exact terminal run/attempt and registry relation after restart.
    /// `start..end` are half-open recorded row coordinates; partial/failure
    /// outcomes keep their own observations and are never substituted by another run.
    pub async fn results(
        &self,
        run: &str,
        attempt: &str,
        relation: &str,
        start: u64,
        end: u64,
        cancel: CancellationToken,
    ) -> Result<CanonicalResultReader, WorkflowError> {
        let access = pse_diagnostics::observation::current().map(|parent| {
            parent.child(
                "result.reopen_and_access",
                pse_diagnostics::observation::Relationship::Continuation,
            )
        });
        let context = access.as_ref().map(|operation| operation.context());
        if let Some(context) = &context {
            context.source_ref("canonical.run", run);
            context.source_ref("canonical.attempt", attempt);
            context.text("result.relation", relation);
        }
        let activity = self.admit_activity()?;
        let result = activity
            .scope(pse_diagnostics::observation::scope(context, async {
                let _observation = pse_diagnostics::observation::phase("result.read");
                cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                if start > end {
                    return Err(contract("result row range is reversed"));
                }
                let extent = response_owner(
                    self,
                    "canonical:result-selection",
                    MetadataResponse::Selection,
                )?;
                self.result_reader_body(
                    TerminalResultSelection::Exact { run, attempt },
                    relation,
                    start,
                    end,
                    cancel,
                    extent,
                    None,
                )?
                .await
            }))
            .await;
        match result {
            Ok(mut reader) => {
                reader.observation = access;
                CanonicalResultReader::from_reader(reader)
            }
            Err(error) => {
                if let Some(access) = access {
                    access.finish(pse_diagnostics::observation::Outcome::Failed);
                }
                Err(error)
            }
        }
    }

    /// Construct the large native selection state before its caller polls it.
    /// The existing activity scope owns this boxed body through cancellation. Its
    /// independently prepaid heap state drops under its own grant. The response
    /// grant moves in without an extra alias, so protected
    /// metadata transfer can still unwrap a unique grant after transport drains.
    #[allow(
        clippy::too_many_arguments,
        reason = "the reader factory keeps terminal selection, row coordinates, cancellation and continuing response ownership explicit"
    )]
    #[inline(never)]
    fn result_reader_body<'a>(
        &'a self,
        selection: TerminalResultSelection<'a>,
        relation: &'a str,
        start: u64,
        end: u64,
        cancel: CancellationToken,
        extent: Arc<MemoryReservation>,
        output: Option<Arc<OutputFilter>>,
    ) -> Result<PrepaidReaderFuture<'a, Result<ResultReader, WorkflowError>>, WorkflowError> {
        prepaid_reader_future(&self.shared.pool(), async move {
            let read = match selection {
                TerminalResultSelection::Exact { run, attempt } => {
                    pse_operations::canonical::response_ownership_scope(
                        extent.clone(),
                        self.canonical_store()?.read_results(run, attempt),
                    )
                    .await?
                }
                TerminalResultSelection::Latest { problem, classes } => {
                    pse_operations::canonical::response_ownership_scope(
                        extent.clone(),
                        self.canonical_store()?.latest_results(problem, classes),
                    )
                    .await?
                }
            };
            self.result_selection(read, relation, start..end, cancel, extent, output)
                .await
        })
    }

    async fn result_selection(
        &self,
        read: ResultRead,
        relation: &str,
        rows: std::ops::Range<u64>,
        cancel: CancellationToken,
        extent: Arc<MemoryReservation>,
        output: Option<Arc<OutputFilter>>,
    ) -> Result<ResultReader, WorkflowError> {
        let std::ops::Range { start, end } = rows;
        let read_owner = Arc::new(retain_protected_metadata(
            &self.shared.pool(),
            extent,
            read.retained_bytes()?,
            |owner| read.retain_metadata_owner(owner),
        )?);
        read.retain_metadata_owner(read_owner.clone())?;
        if start > end {
            return Err(contract("result row range is reversed"));
        }
        let spec = self
            .registry()
            .relation(relation)
            .ok_or_else(|| contract("unknown result relation"))?;
        let name = spec.id.to_string();
        let schema = pse_schema::arrow::relation_schema_ref(self.registry(), spec)
            .map_err(|error| pse_engine::EngineError::Semantic(Arc::new(error)))?;
        let set = read
            .sets()
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| contract("relation is not part of selected attempt"))?
            .key
            .as_str();
        // The schema is borrowed from the registry's charged immutable arena.
        // The snapshot owns the shared ResultRead allocation. This reader only
        // owns its additional set key; Arc aliases do not charge read metadata twice.
        let selection_owner =
            MemoryConsumer::new("canonical:result-selection-key").register(&self.shared.pool());
        selection_owner
            .try_grow(set.len())
            .map_err(pse_engine::EngineError::from)?;
        let set = set.to_owned();
        let read = Arc::new(read);
        let snapshot = if output.is_none() {
            Some(
                super::ResultSnapshot::capture(
                    self,
                    read.clone(),
                    read_owner.clone(),
                    spec.id,
                    &set,
                    schema.clone(),
                    start,
                    end,
                )
                .await?,
            )
        } else {
            None
        };
        let decode = match &snapshot {
            Some(snapshot) => snapshot.decoder(),
            None => super::result_snapshot::ProtectedResultDecode::new(
                self,
                read.clone(),
                read_owner.clone(),
                spec.id,
                schema.clone(),
            )?,
        };
        let exhausted = start == end || snapshot.as_ref().is_some_and(|s| s.blocks().is_empty());
        Ok(ResultReader {
            observation: None,
            runtime: self.clone(),
            snapshot,
            decode,
            read,
            read_owner,
            relation: spec.id,
            set,
            schema,
            start,
            end,
            after: None,
            snapshot_position: 0,
            pending: VecDeque::new(),
            metadata_owner: None,
            exhausted,
            consumed: false,
            cancel,
            output,
            indexes: BTreeMap::new(),
            descriptors: BTreeMap::new(),
            descriptor_owner: None,
            selected_batch: None,
            _selection_owner: selection_owner,
        })
    }
    /// Select one explicitly declared scientific output. Scalar numeric bounds
    /// filter finite values; dense bounds select time. Original rows retain their
    /// units, basis and quality fields, and every index is checked against IPC.
    #[allow(
        clippy::too_many_arguments,
        reason = "one output-read operation exposes exact attempt, scientific output, stored row range and finite/null predicate coordinates"
    )]
    pub async fn output_results(
        &self,
        run: &str,
        attempt: &str,
        relation: &str,
        owner: pse_ids::SemanticId,
        field: &str,
        partition: &str,
        start: u64,
        end: u64,
        minimum: Option<f64>,
        maximum: Option<f64>,
        missing: Option<bool>,
        cancel: CancellationToken,
    ) -> Result<SelectedResultReader, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let _observation = pse_diagnostics::observation::phase("result.read");
                if minimum.is_some_and(|value| !value.is_finite())
                    || maximum.is_some_and(|value| !value.is_finite())
                    || minimum.zip(maximum).is_some_and(|(min, max)| min > max)
                    || missing == Some(true) && (minimum.is_some() || maximum.is_some())
                {
                    return Err(contract("invalid finite output predicate"));
                }
                if start > end {
                    return Err(contract("result row range is reversed"));
                }
                let spec = self
                    .registry()
                    .relation(relation)
                    .ok_or_else(|| contract("unknown result relation"))?;
                super::result_projection::check_output_field(spec.id, field)?;
                let dense =
                    spec.id == pse_relations::generated::runtime::simulation_samples::RELATION_ID;
                if dense && missing.is_some() {
                    return Err(contract(
                        "finite dense outputs have no missing-value predicate",
                    ));
                }
                // Two exact 128-bit hexadecimal identities plus separators; reserve
                // owned strings and temporary formatting before materializing them.
                let output_length = 65usize
                    .checked_add(if dense {
                        1usize
                            .checked_add(field.len())
                            .ok_or_else(|| contract("output identity extent"))?
                    } else {
                        0
                    })
                    .ok_or_else(|| contract("output identity extent"))?;
                let admitted = output_length
                    .checked_add(field.len())
                    .and_then(|bytes| bytes.checked_add(partition.len()))
                    .and_then(|bytes| bytes.checked_add(128))
                    .ok_or_else(|| contract("output selection allocation extent"))?;
                let filter_owner =
                    MemoryConsumer::new("canonical:output-selection").register(&self.shared.pool());
                filter_owner
                    .try_grow(admitted + size_of::<OutputFilter>() + 2 * size_of::<usize>())
                    .map_err(pse_engine::EngineError::from)?;
                let mut output = String::with_capacity(output_length);
                write_output_identity(&mut output, &spec.id, owner, field, dense)?;
                let filter = OutputFilter {
                    output,
                    owner,
                    field: field.into(),
                    partition: partition.into(),
                    dense,
                    minimum,
                    maximum,
                    missing,
                    _owner: filter_owner,
                };
                let retained = filter
                    .output
                    .capacity()
                    .checked_add(filter.field.capacity())
                    .and_then(|bytes| bytes.checked_add(filter.partition.capacity()))
                    .ok_or_else(|| contract("output selection allocation extent"))?;
                if retained > admitted {
                    return Err(contract(
                        "output selection exceeds admitted string capacity",
                    ));
                }
                filter._owner.shrink(admitted - retained);
                // The immutable typed predicate exists before any block/descriptor RPC.
                let extent = response_owner(
                    self,
                    "canonical:selected-result-selection",
                    MetadataResponse::Selection,
                )?;
                let reader = self
                    .result_reader_body(
                        TerminalResultSelection::Exact { run, attempt },
                        relation,
                        start,
                        end,
                        cancel,
                        extent,
                        Some(Arc::new(filter)),
                    )?
                    .await?;
                Ok(SelectedResultReader { reader })
            })
            .await
    }
}

fn write_output_identity(
    output: &mut String,
    relation: &impl std::fmt::Display,
    owner: pse_ids::SemanticId,
    field: &str,
    dense: bool,
) -> Result<(), WorkflowError> {
    use std::fmt::Write as _;
    write!(output, "{relation}:{owner}").map_err(|_| contract("output identity formatting"))?;
    if dense {
        output.push(':');
        output.push_str(field);
    }
    Ok(())
}

#[cfg(test)]
mod result_admission_unit {
    use super::*;
    use datafusion::arrow::{array::Int64Array, datatypes::DataType};
    use pse_schema::model::{
        Authority, DerivationGranularity, FieldContract, Namespace, RelationDecl, SnapshotClass,
    };

    #[test]
    fn reader_future_refuses_unfunded_state_before_polling() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1));
        let polled = Arc::new(AtomicBool::new(false));
        let observed = polled.clone();
        let payload = [0_u8; 4096];
        let future = async move {
            observed.store(true, Ordering::Release);
            std::hint::black_box(payload)
        };
        assert!(size_of_val(&future) > 1);
        assert!(prepaid_reader_future(&pool, future).is_err());
        assert!(!polled.load(Ordering::Acquire));
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn reader_future_grant_survives_ready_and_child_drop() {
        use std::sync::atomic::{AtomicBool, Ordering};

        struct ChildDrop {
            pool: Arc<dyn MemoryPool>,
            dropped: Arc<AtomicBool>,
        }
        impl Drop for ChildDrop {
            fn drop(&mut self) {
                assert!(self.pool.reserved() > 0, "child drops under its grant");
                self.dropped.store(true, Ordering::Release);
            }
        }

        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 10));
        let dropped = Arc::new(AtomicBool::new(false));
        let guard = ChildDrop {
            pool: pool.clone(),
            dropped: dropped.clone(),
        };
        let payload = [0_u8; 4096];
        let mut first = true;
        let future = std::future::poll_fn(move |_| {
            std::hint::black_box((&guard, &payload));
            if std::mem::take(&mut first) {
                Poll::Pending
            } else {
                Poll::Ready(7)
            }
        });
        let bytes = size_of_val(&future);
        let mut future = prepaid_reader_future(&pool, future).unwrap();
        assert_eq!(pool.reserved(), bytes);
        let mut cx = Context::from_waker(futures_util::task::noop_waker_ref());
        assert_eq!(Pin::new(&mut future).poll(&mut cx), Poll::Pending);
        assert_eq!(pool.reserved(), bytes);
        assert_eq!(Pin::new(&mut future).poll(&mut cx), Poll::Ready(7));
        assert_eq!(pool.reserved(), bytes);
        assert!(!dropped.load(Ordering::Acquire));
        drop(future);
        assert!(dropped.load(Ordering::Acquire));
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn metadata_transfer_never_shrinks_live_sdk_or_alias_grants() {
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
        let response =
            pooled_response_owner(&pool, "test:metadata-response", MetadataResponse::Header)
                .unwrap();
        let original = response.size();
        let sdk = response.clone();
        let retained = Arc::new(retain_metadata(&pool, response, 4096).unwrap());
        assert_eq!(
            sdk.size(),
            original,
            "actual SDK suffix retains its entire allowance"
        );
        assert_eq!(pool.reserved(), original + 4096);
        let alias = retained.clone();
        assert_eq!(
            pool.reserved(),
            original + 4096,
            "immutable aliases add no charge"
        );
        drop(retained);
        drop(sdk);
        assert_eq!(pool.reserved(), 4096);
        drop(alias);
        assert_eq!(pool.reserved(), 0);
        let response = pooled_response_owner(
            &pool,
            "test:unique-metadata-response",
            MetadataResponse::Header,
        )
        .unwrap();
        let retained = retain_metadata(&pool, response, 4096).unwrap();
        assert_eq!(
            pool.reserved(),
            4096,
            "a proven unique grant transfers without an overlapping allocation"
        );
        drop(retained);
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn protected_metadata_prepays_cleanup_without_shrinking_live_sdk_grant() {
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
        let bytes = 4096 + MetadataResponse::Protection.allocation_bytes();
        let response = pooled_response_owner(
            &pool,
            "test:protected-metadata-response",
            MetadataResponse::Selection,
        )
        .unwrap();
        let original = response.size();
        let sdk = response.clone();
        let retained = Arc::new(
            retain_protected_metadata(&pool, response, 4096, |_| {
                panic!("successful overlap must not attach the original grant")
            })
            .unwrap(),
        );
        assert_eq!(sdk.size(), original);
        assert_eq!(retained.size(), bytes);
        assert_eq!(pool.reserved(), original + bytes);
        let cleanup = retained.clone();
        drop(retained);
        drop(sdk);
        assert_eq!(
            pool.reserved(),
            bytes,
            "cleanup keeps metadata and its response allowance"
        );
        drop(cleanup);
        assert_eq!(pool.reserved(), 0);

        let response = pooled_response_owner(
            &pool,
            "test:unique-protected-response",
            MetadataResponse::Selection,
        )
        .unwrap();
        let retained = retain_protected_metadata(&pool, response, 4096, |_| {
            panic!("unique transfer must not attach the original grant")
        })
        .unwrap();
        assert_eq!(pool.reserved(), bytes);
        drop(retained);
        assert_eq!(pool.reserved(), 0);
        let response = pooled_response_owner(
            &pool,
            "test:protected-overflow",
            MetadataResponse::Selection,
        )
        .unwrap();
        let mut fallback = None;
        assert!(
            retain_protected_metadata(&pool, response, usize::MAX, |owner| {
                fallback = Some(owner);
                Ok(())
            })
            .is_err()
        );
        assert!(
            fallback.is_some(),
            "checked extent failure retains the original grant through cleanup"
        );
        drop(fallback);
        assert_eq!(pool.reserved(), 0);
    }

    #[tokio::test]
    async fn protected_metadata_refusal_keeps_original_grant_through_late_sdk_and_queued_cleanup() {
        let original = MetadataResponse::Selection.allocation_bytes();
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(original));
        let response = pooled_response_owner(
            &pool,
            "test:refused-protected-response",
            MetadataResponse::Selection,
        )
        .unwrap();
        let sdk = response.clone();
        let mut cleanup_owner = None;
        assert!(
            retain_protected_metadata(&pool, response, 4096, |owner| {
                cleanup_owner = Some(owner);
                Ok(())
            })
            .is_err()
        );
        assert_eq!(
            sdk.size(),
            original,
            "failed admission never shrinks the live SDK grant"
        );
        assert!(
            cleanup_owner.is_some(),
            "refused read transfers its original grant to cleanup"
        );
        let (sdk_drain, sdk_drained) = tokio::sync::oneshot::channel();
        let (cleanup_drain, cleanup_drained) = tokio::sync::oneshot::channel();
        let late_sdk = tokio::spawn(async move {
            let _ = sdk_drained.await;
            drop(sdk);
        });
        let queued_cleanup = tokio::spawn(async move {
            let _ = cleanup_drained.await;
            drop(cleanup_owner);
        });
        assert_eq!(
            pool.reserved(),
            original,
            "SDK and cleanup share one original allocation"
        );
        sdk_drain.send(()).unwrap();
        late_sdk.await.unwrap();
        assert_eq!(
            pool.reserved(),
            original,
            "SDK drain alone cannot retire queued cleanup's grant"
        );
        let refused = MemoryConsumer::new("test:refused-cleanup-reuse").register(&pool);
        assert!(refused.try_grow(1).is_err());
        cleanup_drain.send(()).unwrap();
        queued_cleanup.await.unwrap();
        assert_eq!(pool.reserved(), 0);
    }

    #[tokio::test]
    async fn selected_reader_admits_decoded_stable_checks_in_one_actual_operation_scope() {
        let base = super::super::tests::runtime_on(
            64 << 20,
            crate::math::MathPolicy {
                artifact_bytes: 4 << 20,
                workspace_bytes: 4 << 20,
                foreign_bytes: 1 << 20,
                worker_bytes: 4 << 20,
                ..Default::default()
            },
        );
        let mut declarations = pse_schema::RegistryBuilder::new();
        declarations.declare_relation(
            RelationDecl::new(
                Namespace::Normalized,
                "selected_clock",
                1,
                Authority::Derived,
                SnapshotClass::Model,
                "generic decoded results retain the selected reader's native clock",
            )
            .granularity(DerivationGranularity::Rule)
            .pk(&["ts"])
            .columns(vec![FieldContract::key(
                "ts",
                FieldContract::native(DataType::Int64),
                "timestamp nanoseconds",
            )])
            .checks(BTreeMap::from([(
                "clock".into(),
                "ts = CAST(now() AS BIGINT)".into(),
            )])),
        );
        let registry = Arc::new(declarations.build().unwrap());
        let runtime = Runtime::from_shared_local(
            base.shared.clone(),
            base.model.factory().bind_model(registry.clone()).unwrap(),
        )
        .unwrap();
        let spec = registry.relation("normalized.selected_clock").unwrap();
        let stale = runtime.validation_context();
        let stale_contract = stale.relation(&registry, spec).unwrap();
        let first = runtime
            .scope_activity(async { Ok(ResultAdmission::new(&runtime)) })
            .await
            .unwrap();
        let timestamp = first
            .0
            .admission_scope()
            .execution_props()
            .query_execution_start_time
            .unwrap()
            .timestamp_nanos_opt()
            .unwrap();
        let schema = pse_schema::arrow::relation_schema_ref(&registry, spec).unwrap();
        let raw = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int64Array::from(vec![timestamp]))],
        )
        .unwrap();
        let bytes = result_files::encode_result_file(&raw).unwrap();
        let decoded = result_files::decode_result_file(&bytes, schema, 1).unwrap();
        let cancel = CancellationToken::new();
        assert!(
            !stale_contract
                .evaluate(&decoded, 8, &cancel)
                .unwrap()
                .is_valid(),
            "a pre-warmed runtime owner is not the selected read operation's clock"
        );
        let own = || {
            pse_columnar::owned_buffer::OwnedRecordBatch::copy(
                &decoded,
                &runtime.shared.pool(),
                &cancel,
            )
            .unwrap()
        };
        first.admit(&runtime, spec.id, own(), &cancel).unwrap();
        let second = runtime
            .scope_activity(async { Ok(ResultAdmission::new(&runtime)) })
            .await
            .unwrap();
        assert!(
            !first
                .0
                .admission_scope()
                .same_invocation(second.0.admission_scope())
        );
        assert!(
            second.admit(&runtime, spec.id, own(), &cancel).is_err(),
            "an independent reader cannot reuse old Stable validity"
        );
        first.admit(&runtime, spec.id, own(), &cancel).unwrap();
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_connected_results_server_unit {
    #![allow(
        unsafe_code,
        reason = "isolated generated finite-field fixtures explicitly assert a partial terminal class"
    )]
    use super::super::durable::PublicationAcks;
    use super::*;
    use crate::workflow::PackScientificState;
    use datafusion::arrow::array::{Array, Float64Array};
    use pse_operations::canonical_execution::{
        RunRequest, TerminalClass, result_batch_key, result_payload_digest, result_set_key,
    };
    use pse_relations::generated::{enums::NativeMetricKind, runtime::solve_metrics};
    #[tokio::test]
    async fn canonical_nonempty_multipage_reads_share_256mib_with_real_compiler_leases() {
        use pse_compiler::workspace::{CompilerContext, WorkspaceLimits};
        let local = super::super::tests::runtime_on(
            256 << 20,
            crate::math::MathPolicy {
                workspace_bytes: 64 << 20,
                worker_bytes: 64 << 20,
                artifact_bytes: 16 << 20,
                foreign_bytes: 8 << 20,
                inner_session_bytes: 0,
                ..Default::default()
            },
        );
        let runtime = super::super::durable_tests::durable_runtime_on(local);
        let pool = runtime.shared.pool();
        assert!(
            matches!(pool.memory_limit(), datafusion::execution::memory_pool::MemoryLimit::Finite(limit) if limit == 256 << 20)
        );
        let store = runtime.canonical_store().unwrap();
        let source = store
            .edit("multipage-pressure", None, "source", &[])
            .await
            .unwrap();
        store
            .begin_run(&RunRequest {
                key: "multipage-pressure".into(),
                revision: source,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run(
                "multipage-pressure",
                "claim",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let run_id = pse_operations::mint_id();
        let validation = runtime.validation_context();
        let mut batches = Vec::new();
        for step in 0..65 {
            let mut builder =
                solve_metrics::Builder::with_registry(runtime.registry(), 1, &validation).unwrap();
            builder
                .push(solve_metrics::Row {
                    run_id,
                    step,
                    namespace: "original".into(),
                    name: "temperature".into(),
                    kind: NativeMetricKind::Real,
                    real: Some(273.15 + step as f64),
                    integer: None,
                    boolean: None,
                    text: None,
                    unavailable: None,
                })
                .unwrap();
            batches.push(builder.finish().unwrap());
        }
        let cursor = super::super::ResultCursor::from_retained(
            batches[0].batch().schema(),
            solve_metrics::RELATION_ID,
            &batches,
        )
        .unwrap();
        super::super::result_projection::store_result_table(
            store,
            &fence,
            solve_metrics::RELATION_ID,
            cursor,
            &pool,
            &PublicationAcks::new(&pool),
        )
        .await
        .unwrap();
        drop(batches);
        let closed = store.close_result_ingestion(&fence, "close").await.unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: this fixture publishes original finite observations and claims partial availability only.
        unsafe {
            store
                .seal_attempt(
                    &manifest,
                    "seal",
                    TerminalClass::Partial,
                    &[1],
                    pse_operations::canonical_execution::SeedEligibility::Unavailable,
                )
                .await
                .unwrap();
        }
        let compiler_baseline = pool.reserved();
        let physical = super::super::tests::physical();
        let workspace = || {
            runtime
                .native()
                .workspace(
                    CompilerContext {
                        quantities: physical.quantities.clone(),
                        preconditions: physical.preconditions.clone(),
                        providers: BTreeMap::new(),
                    },
                    WorkspaceLimits::default(),
                )
                .unwrap()
        };
        let first_compiler = workspace();
        let second_compiler = workspace();
        assert_eq!(
            pool.reserved() - compiler_baseline,
            128 << 20,
            "two actual compiler generations retain their configured64MiB leases"
        );
        let baseline = pool.reserved();
        let cancel = CancellationToken::new();
        let mut reader = runtime
            .results(
                fence.run(),
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                65,
                cancel.clone(),
            )
            .await
            .unwrap();
        let snapshot = reader.snapshot();
        assert_eq!(
            snapshot.blocks().len(),
            65,
            "capture crosses the64-row metadata page boundary"
        );
        let selected = pool.reserved();
        let alias = snapshot.clone();
        assert_eq!(
            pool.reserved(),
            selected,
            "snapshot aliases share inventory and read ownership"
        );
        let mut rows = 0;
        while let Some(batch) = reader.next_batch().await.unwrap() {
            assert_eq!(batch.num_rows(), 1);
            let values = batch
                .column_by_name("real")
                .unwrap()
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap();
            assert_eq!(values.value(0), 273.15 + rows as f64);
            rows += 1;
            assert_eq!(reader.reader.snapshot_position, rows);
            assert!(reader.reader.pending.is_empty());
            assert!(
                reader.reader.metadata_owner.is_none(),
                "full consumption uses frozen metadata only"
            );
            assert!(
                reader.reader.after.is_none(),
                "no second unrestricted metadata cursor"
            );
        }
        assert_eq!(rows, 65, "every nonempty block opened and decoded");
        let pressure = MemoryConsumer::new("test:exhausted-reader-pressure").register(&pool);
        pressure.try_grow((256 << 20) - pool.reserved()).unwrap();
        assert!(reader.next_relation_batch().await.unwrap().is_none());
        assert!(reader.next_batch().await.unwrap().is_none());
        drop(pressure);
        drop(reader);
        drop(snapshot);
        assert!(
            pool.reserved() > baseline,
            "independent snapshot keeps its metadata/footer owners"
        );
        drop(alias);
        pse_operations::testing::drain_result_releases(store)
            .await
            .unwrap();
        assert_eq!(
            pool.reserved(),
            baseline,
            "last retained read releases exact resident storage"
        );
        let pressure = MemoryConsumer::new("test:required-result-pressure").register(&pool);
        let remaining = (256 << 20) - pool.reserved();
        pressure
            .try_grow(remaining - MetadataResponse::Selection.allocation_bytes() + 1)
            .unwrap();
        assert!(matches!(
            runtime
                .results(
                    fence.run(),
                    fence.attempt(),
                    "runtime.solve_metrics",
                    0,
                    65,
                    CancellationToken::new()
                )
                .await,
            Err(WorkflowError::Engine(_))
        ));
        drop(pressure);
        let mut cancelled = runtime
            .results(
                fence.run(),
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                65,
                cancel.clone(),
            )
            .await
            .unwrap();
        cancel.cancel();
        assert!(cancelled.next_batch().await.is_err());
        drop(cancelled);
        pse_operations::testing::drain_result_releases(store)
            .await
            .unwrap();
        assert_eq!(pool.reserved(), baseline);
        drop(first_compiler);
        drop(second_compiler);
        assert_eq!(pool.reserved(), compiler_baseline);
    }

    #[tokio::test]
    async fn canonical_empty_demand_has_registry_schema_without_ipc_or_absent_membership() {
        use super::super::durable::{StoredCompletion, TerminationCause, TerminationDetail};
        let runtime = super::super::durable_tests::durable_runtime();
        let store = runtime.canonical_store().unwrap().clone();
        let pool = runtime.shared.pool();
        let source = store
            .edit("empty-demand", None, "source", &[])
            .await
            .unwrap();
        store
            .begin_run(&RunRequest {
                key: "empty-demand".into(),
                revision: source,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run("empty-demand", "claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let validation = runtime.validation_context();
        let empty = solve_metrics::Builder::with_registry(runtime.registry(), 0, &validation)
            .unwrap()
            .finish()
            .unwrap();
        let schema = empty.batch().schema();
        let acknowledgments = PublicationAcks::new(&pool);
        super::super::result_projection::store_result_table(
            &store,
            &fence,
            solve_metrics::RELATION_ID,
            crate::workflow::ResultCursor::from_checked(empty),
            &pool,
            &acknowledgments,
        )
        .await
        .unwrap();
        assert_eq!(
            pse_operations::testing::pending_result_batch_rows(
                &store,
                &fence,
                &solve_metrics::RELATION_ID.to_string(),
                0
            )
            .await
            .unwrap(),
            None
        );
        let original = StoredCompletion {
            version: 2,
            attempt_id: pse_operations::mint_id(),
            completion: None,
            termination: TerminationDetail {
                version: pse_model::document::Version,
                cause: TerminationCause::Assessment {
                    usable: false,
                    candidate_use: vec![],
                    diagnostics: vec![],
                },
                point: None,
                retry_failure: Some(pse_model::study::RetryFailure::Deterministic),
                effect: pse_model::study::EffectState::Absent,
            },
            state: pse_model::generated::enums::AttemptState::Partial,
            progress: super::super::ProgressCoverage::Unknown {},
        };
        let bytes = serde_json::to_vec(&original).unwrap();
        acknowledgments
            .append(
                &fence,
                store
                    .append_result_batch(
                        &fence,
                        "completion",
                        "__completion",
                        0,
                        pse_operations::canonical_execution::ResultPayload::new(bytes.to_vec())
                            .unwrap(),
                        bytes.len() as u64,
                    )
                    .await
                    .unwrap(),
            )
            .unwrap();
        acknowledgments.complete(&fence, "__completion").unwrap();
        let receipt=serde_json::to_vec(&serde_json::json!({"version":1,"batch_count":1,"payload_bytes":bytes.len(),"digest":result_payload_digest(&bytes),"inline":null})).unwrap();
        let manifest = store
            .close_acknowledged(&fence, "close", acknowledgments.freeze(&fence).unwrap())
            .await
            .unwrap();
        // SAFETY: the fixture owns a completed zero-row relation and asserts partial availability only.
        unsafe {
            store
                .seal_acknowledged(
                    &manifest,
                    "seal",
                    TerminalClass::Partial,
                    &receipt,
                    pse_operations::canonical_execution::SeedEligibility::Unavailable,
                )
                .await
                .unwrap();
        }
        let before_selection = pool.reserved();
        let mut selected = runtime
            .results(
                fence.run(),
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                0,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let reader_metadata = selected.reader.set.capacity();
        assert_eq!(selected.reader._selection_owner.size(), reader_metadata);
        let snapshot = selected.snapshot();
        let snapshot_metadata = snapshot.inventory_reserved_bytes();
        assert!(snapshot_metadata > selected.selection().retained_bytes().unwrap());
        assert!(
            snapshot.blocks().is_empty(),
            "requested empty relation retains no physical file inventory"
        );
        let retained = reader_metadata + snapshot_metadata;
        assert!(
            retained - MetadataResponse::Protection.allocation_bytes()
                < MetadataResponse::Selection.allocation_bytes() / 16,
            "idle reader retains actual metadata plus its prepaid final cleanup response"
        );
        assert_eq!(pool.reserved() - before_selection, retained);
        assert_eq!(selected.schema().fields(), schema.fields());
        let demanded = selected
            .selection()
            .sets()
            .iter()
            .find(|set| set.name == solve_metrics::RELATION_ID.to_string())
            .unwrap();
        assert_eq!((demanded.batch_count, demanded.row_count), (0, 0));
        let datafusion::execution::memory_pool::MemoryLimit::Finite(limit) = pool.memory_limit()
        else {
            panic!("empty-demand pressure control requires its finite runtime pool");
        };
        let pressure = MemoryConsumer::new("test:empty-reader-pressure").register(&pool);
        pressure.try_grow(limit - pool.reserved()).unwrap();
        assert_eq!(
            pool.reserved(),
            limit,
            "no boxed reader state can be admitted"
        );
        assert!(selected.next_relation_batch().await.unwrap().is_none());
        assert!(selected.next_batch().await.unwrap().is_none());
        assert!(selected.next_batch().await.unwrap().is_none());
        selected.cancellation_token().cancel();
        assert!(
            selected.next_relation_batch().await.is_err(),
            "cancellation precedes exhaustion"
        );
        drop(pressure);
        drop(selected);
        assert_eq!(
            pool.reserved() - before_selection,
            snapshot_metadata,
            "an independent snapshot alias retains its one metadata owner"
        );
        drop(snapshot);
        pse_operations::testing::drain_result_releases(&store)
            .await
            .unwrap();
        assert_eq!(pool.reserved(), before_selection);
        assert!(
            runtime
                .results(
                    fence.run(),
                    fence.attempt(),
                    "runtime.solve_variables",
                    0,
                    0,
                    CancellationToken::new()
                )
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn canonical_decoded_arrow_survives_reader_drop_and_result_reclamation() {
        let runtime = super::super::durable_tests::durable_runtime();
        let store = runtime.canonical_store().unwrap().clone();
        let pool = runtime.shared.pool();
        let source = store
            .edit("decoded-owner", None, "source", &[])
            .await
            .unwrap();
        store
            .begin_run(&RunRequest {
                key: "decoded-owner".into(),
                revision: source,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run("decoded-owner", "claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let run_id = pse_operations::mint_id();
        let expected = [Some(-0.0), None, Some(273.15)]
            .into_iter()
            .enumerate()
            .map(|(step, real)| solve_metrics::Row {
                run_id,
                step: step as i64,
                namespace: "original".into(),
                name: "temperature".into(),
                kind: if real.is_some() {
                    NativeMetricKind::Real
                } else {
                    NativeMetricKind::Unavailable
                },
                real,
                integer: None,
                boolean: None,
                text: None,
                unavailable: if real.is_none() {
                    Some(pse_model::generated::enums::EvidenceUnavailableReason::Nonfinite)
                } else {
                    None
                },
            })
            .collect::<Vec<_>>();
        let validation = runtime.validation_context();
        let mut builder =
            solve_metrics::Builder::with_registry(runtime.registry(), expected.len(), &validation)
                .unwrap();
        for row in &expected {
            builder.push(row.clone()).unwrap();
        }
        let original = builder.finish().unwrap();
        super::super::result_projection::store_result_table(
            &store,
            &fence,
            solve_metrics::RELATION_ID,
            crate::workflow::ResultCursor::from_checked(original.clone()),
            &pool,
            &PublicationAcks::new(&pool),
        )
        .await
        .unwrap();
        drop(original);
        let closed = store.close_result_ingestion(&fence, "close").await.unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: independently authored observations assert partial availability only.
        unsafe {
            store.seal_attempt(
                &manifest,
                "seal",
                TerminalClass::Partial,
                &[1],
                pse_operations::canonical_execution::SeedEligibility::Unavailable,
            )
        }
        .await
        .unwrap();
        let baseline = pool.reserved();
        let mut reader = runtime
            .results(
                "decoded-owner",
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                3,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let query_reader = runtime
            .results(
                "decoded-owner",
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                3,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let snapshot = query_reader.snapshot();
        drop(query_reader);
        let state = snapshot.session_state();
        assert!(
            state.scalar_functions().contains_key("pse_checked_value"),
            "query retains registry-bound native functions"
        );
        let context = datafusion::execution::context::SessionContext::new_with_state(state);
        // Model state intentionally creates no default catalog/schema. Bind
        // this provider directly without mutating the model's shared catalogs.
        let dataframe = context
            .read_table(snapshot.table_provider())
            .unwrap()
            .filter(datafusion::prelude::col("real").gt(datafusion::prelude::lit(0.0)))
            .unwrap()
            .select_columns(&["real", "step"])
            .unwrap()
            .sort(vec![datafusion::prelude::col("step").sort(true, false)])
            .unwrap()
            .limit(0, Some(1))
            .unwrap();
        let plan = dataframe.create_physical_plan().await.unwrap();
        drop(dataframe); // The unexecuted physical plan now owns this query's snapshot.
        let task = context.task_ctx();
        drop(snapshot);
        drop(context);
        let cleanup = store.independent_caller().unwrap();
        runtime.close();
        let batch = reader.next_batch().await.unwrap().unwrap();
        {
            let view = solve_metrics::View::try_from_batch_with_registry(
                runtime.registry(),
                &batch,
                &validation,
            )
            .unwrap();
            assert_eq!(
                (0..view.len())
                    .map(|index| view.row(index).unwrap())
                    .collect::<Vec<_>>(),
                expected
            );
        }
        let escaped = batch.column_by_name("real").unwrap().clone();
        assert!(cleanup.forget_run_results("decoded-owner").await.is_err());
        drop(reader);
        assert!(
            cleanup.forget_run_results("decoded-owner").await.is_err(),
            "unexecuted native plan retains future unread files"
        );
        let queried = datafusion::physical_plan::collect(plan.clone(), task)
            .await
            .unwrap();
        assert_eq!(
            queried.iter().map(|batch| batch.num_rows()).sum::<usize>(),
            1
        );
        let values = queried[0]
            .column(0)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(values.value(0).to_bits(), 273.15f64.to_bits());
        drop(queried);
        drop(plan);
        drop(batch);
        assert!(pool.reserved() > baseline);
        // Fully decoded buffers require no storage access. The reader's final
        // protection release can settle while these accounted arrays remain live.
        let mut retired = None;
        for _ in 0..32 {
            if let Ok(page) = cleanup.forget_run_results("decoded-owner").await {
                retired = Some(page);
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let retired = retired.expect("final protected-reader release settles");
        let mut reclaimed = retired.batches;
        let mut complete = retired.complete;
        for _ in 0..8 {
            if complete {
                break;
            }
            let page = cleanup.reclaim_result_page("decoded-owner").await.unwrap();
            reclaimed += page.batches;
            if page.complete {
                complete = true;
                break;
            }
        }
        assert!(complete);
        assert_eq!(reclaimed, 1);
        assert!(
            runtime
                .results(
                    "decoded-owner",
                    fence.attempt(),
                    "runtime.solve_metrics",
                    0,
                    3,
                    CancellationToken::new()
                )
                .await
                .is_err()
        );
        cleanup.remove_isolated_fixture().await.unwrap();
        pse_operations::testing::drain_result_releases(&store)
            .await
            .unwrap();
        drop(cleanup);
        drop(store);
        drop(runtime);
        let values = escaped.as_any().downcast_ref::<Float64Array>().unwrap();
        assert_eq!(values.value(0).to_bits(), (-0.0f64).to_bits());
        assert!(values.is_null(1));
        assert_eq!(values.value(2), 273.15);
        assert!(pool.reserved() > baseline);
        drop(escaped);
        assert_eq!(pool.reserved(), baseline);
    }
    #[tokio::test]
    async fn canonical_output_indexes_read_original_rows_and_skip_unrelated_blocks() {
        use pse_relations::generated::runtime::{fit_parameters, simulation_samples};
        let runtime = super::super::durable_tests::durable_runtime();
        let store = runtime.canonical_store().unwrap();
        let source = store.edit("projection", None, "source", &[]).await.unwrap();
        store
            .begin_run(&RunRequest {
                key: "projection".into(),
                revision: source,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run("projection", "claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let validation = runtime.validation_context();
        let first: pse_ids::SemanticId = pse_operations::mint_id();
        let second: pse_ids::SemanticId = pse_operations::mint_id();
        let absent: pse_ids::SemanticId = pse_operations::mint_id();
        let paged: pse_ids::SemanticId = pse_operations::mint_id();
        let unit: pse_ids::SemanticId = pse_operations::mint_id();
        let run_id = pse_operations::mint_id();
        let mut parameters =
            fit_parameters::Builder::with_registry(runtime.registry(), 800, &validation).unwrap();
        for index in 0..800 {
            parameters
                .push(fit_parameters::Row {
                    // Distinct full keys deliberately exercise metadata paging
                    // for one derived owner selector in this storage fixture.
                    run_id: if index >= 100 {
                        pse_operations::mint_id()
                    } else {
                        run_id
                    },
                    parameter_id: if index == 0 {
                        first
                    } else if index == 41 {
                        second
                    } else if index == 42 {
                        absent
                    } else if index >= 100 {
                        paged
                    } else {
                        pse_operations::mint_id()
                    },
                    fixed: false,
                    value: if index == 42 {
                        None
                    } else if index == 41 {
                        Some(-0.0)
                    } else {
                        Some(index as f64)
                    },
                    unit_id: unit,
                    scale: 1.0,
                    at_bound: None,
                })
                .unwrap();
        }
        let parameters = parameters.finish().unwrap();
        super::super::result_projection::store_result_table(
            store,
            &fence,
            fit_parameters::RELATION_ID,
            crate::workflow::ResultCursor::from_checked(parameters.clone()),
            &runtime.shared.pool(),
            &PublicationAcks::new(&runtime.shared.pool()),
        )
        .await
        .unwrap();
        let mut samples =
            simulation_samples::Builder::with_registry(runtime.registry(), 200, &validation)
                .unwrap();
        let mut outputs = [first, second];
        outputs.sort_unstable();
        for symbol in outputs {
            for sample in 0..100 {
                samples
                    .push(simulation_samples::Row {
                        run_id,
                        sample,
                        time: sample as f64,
                        symbol_id: symbol,
                        quantity_id: unit,
                        unit_id: unit,
                        value: if sample == 0 { -0.0 } else { sample as f64 },
                    })
                    .unwrap();
            }
        }
        let samples = samples.finish().unwrap();
        super::super::result_projection::store_result_table(
            store,
            &fence,
            simulation_samples::RELATION_ID,
            crate::workflow::ResultCursor::from_checked(samples.clone()),
            &runtime.shared.pool(),
            &PublicationAcks::new(&runtime.shared.pool()),
        )
        .await
        .unwrap();
        let closed = store.close_result_ingestion(&fence, "close").await.unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: fixture observations deliberately assert partial availability, not usable success.
        unsafe {
            store.seal_attempt(
                &manifest,
                "seal",
                TerminalClass::Partial,
                &[1],
                pse_operations::canonical_execution::SeedEligibility::Unavailable,
            )
        }
        .await
        .unwrap();
        let read = store
            .read_results("projection", fence.attempt())
            .await
            .unwrap();
        let mut missing = runtime
            .output_results(
                "projection",
                fence.attempt(),
                "runtime.fit_parameters",
                absent,
                "value",
                "0",
                0,
                u64::MAX,
                None,
                None,
                Some(true),
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let batch = missing.next_batch().await.unwrap().unwrap();
        assert_eq!(batch.num_rows(), 1);
        assert!(batch.column_by_name("value").unwrap().is_null(0));
        assert!(missing.next_batch().await.unwrap().is_none());
        let mut zero = runtime
            .output_results(
                "projection",
                fence.attempt(),
                "runtime.fit_parameters",
                second,
                "value",
                "0",
                0,
                u64::MAX,
                Some(0.0),
                Some(0.0),
                None,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert!(
            zero.reader.snapshot.is_none(),
            "selected construction never captures full inventory"
        );
        let zero_before = zero
            .selected_snapshot(CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(zero_before.row_count(), 1);
        let batch = zero.next_batch().await.unwrap().unwrap();
        assert_eq!(
            batch
                .column_by_name("value")
                .unwrap()
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap()
                .value(0)
                .to_bits(),
            (-0.0f64).to_bits()
        );
        let zero_partial = zero
            .selected_snapshot(CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(zero_partial.row_count(), 1);
        assert!(zero.next_batch().await.unwrap().is_none());
        let zero_after = zero
            .selected_snapshot(CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(
            zero_after.row_count(),
            1,
            "snapshot is the original universe, never the unread tail"
        );
        let context = datafusion::execution::context::SessionContext::new_with_state(
            zero_after.session_state(),
        );
        let outside = context
            .read_table(zero_after.table_provider())
            .unwrap()
            .filter(datafusion::prelude::col("value").gt(datafusion::prelude::lit(0.0)))
            .unwrap()
            .select_columns(&["value"])
            .unwrap()
            .limit(0, Some(1))
            .unwrap()
            .collect()
            .await
            .unwrap();
        assert_eq!(
            outside.iter().map(|batch| batch.num_rows()).sum::<usize>(),
            0
        );
        let zero_rows = context
            .read_table(zero_after.table_provider())
            .unwrap()
            .filter(datafusion::prelude::col("value").eq(datafusion::prelude::lit(0.0)))
            .unwrap()
            .select_columns(&["value"])
            .unwrap()
            .limit(0, Some(1))
            .unwrap()
            .collect()
            .await
            .unwrap();
        assert_eq!(
            zero_rows
                .iter()
                .map(|batch| batch.num_rows())
                .sum::<usize>(),
            1
        );
        let values = zero_rows
            .iter()
            .find(|b| b.num_rows() > 0)
            .unwrap()
            .column(0)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(values.value(0).to_bits(), (-0.0f64).to_bits());
        drop((
            zero_before,
            zero_partial,
            zero_after,
            context,
            outside,
            zero_rows,
        ));
        let mut late = runtime
            .output_results(
                "projection",
                fence.attempt(),
                "runtime.fit_parameters",
                paged,
                "value",
                "0",
                0,
                u64::MAX,
                Some(798.0),
                None,
                None,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let mut selected_values = Vec::new();
        while let Some(batch) = late.next_batch().await.unwrap() {
            let values = batch
                .column_by_name("value")
                .unwrap()
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap();
            selected_values.extend(values.values().iter().copied());
        }
        assert_eq!(
            selected_values,
            vec![798.0, 799.0],
            "cursor advances across metadata pages whose decoded predicate selects no rows"
        );
        let mut none = runtime
            .output_results(
                "projection",
                fence.attempt(),
                "runtime.fit_parameters",
                paged,
                "value",
                "0",
                0,
                u64::MAX,
                Some(900.0),
                None,
                None,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert!(none.next_batch().await.unwrap().is_none());
        let empty = none
            .selected_snapshot(CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(empty.row_count(), 0);
        assert_eq!(empty.schema(), none.schema());
        drop(empty);
        assert!(
            runtime
                .output_results(
                    "projection",
                    fence.attempt(),
                    "runtime.fit_parameters",
                    paged,
                    "fixed",
                    "0",
                    0,
                    u64::MAX,
                    None,
                    None,
                    None,
                    CancellationToken::new(),
                )
                .await
                .is_err()
        );
        let trajectory_set = result_set_key(
            fence.attempt(),
            &simulation_samples::RELATION_ID.to_string(),
        );
        let page = store
            .result_output_page(
                &read,
                &trajectory_set,
                &format!("{}:{second}:value", simulation_samples::RELATION_ID),
                "0",
                0,
                u64::MAX,
                None,
                Some(20.0),
                Some(22.0),
            )
            .await
            .unwrap();
        assert_eq!(page.outputs.len(), 1);
        assert_eq!(page.blocks.len(), 1);
        assert_eq!(
            read.sets()
                .iter()
                .find(|descriptor| descriptor.key == trajectory_set)
                .unwrap()
                .batch_count,
            1,
            "two exact symbol ranges share one original IPC payload"
        );
        let mut trajectory = runtime
            .output_results(
                "projection",
                fence.attempt(),
                "runtime.simulation_samples",
                second,
                "value",
                "0",
                0,
                u64::MAX,
                Some(20.0),
                Some(22.0),
                None,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let selected_trajectory = trajectory
            .selected_snapshot(CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(selected_trajectory.row_count(), 3);
        let context = datafusion::execution::context::SessionContext::new_with_state(
            selected_trajectory.session_state(),
        );
        let outside = context
            .read_table(selected_trajectory.table_provider())
            .unwrap()
            .filter(datafusion::prelude::col("time").lt(datafusion::prelude::lit(20.0)))
            .unwrap()
            .select_columns(&["time"])
            .unwrap()
            .limit(0, Some(1))
            .unwrap()
            .collect()
            .await
            .unwrap();
        assert_eq!(
            outside.iter().map(|batch| batch.num_rows()).sum::<usize>(),
            0
        );
        drop((selected_trajectory, context, outside));
        let batch = trajectory.next_batch().await.unwrap().unwrap();
        assert_eq!(batch.num_rows(), 3);
        let view = simulation_samples::View::try_from_batch_with_registry(
            runtime.registry(),
            &batch,
            &validation,
        )
        .unwrap();
        for index in 0..3 {
            let row = view.row(index).unwrap();
            assert_eq!(row.symbol_id, second);
            assert_eq!(row.unit_id, unit);
            assert_eq!(row.sample, 20 + index as i64);
            assert_eq!(row.time, (20 + index) as f64);
        }
        assert!(trajectory.next_batch().await.unwrap().is_none());
        // A selected field/time predicate reaches the same portable exporter,
        // retaining only its original matching rows and full typed schema.
        let selected_reader = runtime
            .output_results(
                "projection",
                fence.attempt(),
                "runtime.simulation_samples",
                second,
                "value",
                "0",
                0,
                u64::MAX,
                Some(20.0),
                Some(22.0),
                None,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let selected_borrower = Arc::downgrade(&selected_reader.reader.read);
        let selected_coordinates = selected_reader
            .pack_selection(PackScientificState::Partial)
            .unwrap();
        assert_eq!(selected_coordinates.field.as_deref(), Some("value"));
        assert_eq!(
            selected_coordinates
                .predicate
                .as_ref()
                .unwrap()
                .minimum_bits,
            Some(20.0_f64.to_bits())
        );
        let mut pack_selection = crate::tooling::review_pack::PackSelection::default();
        pack_selection
            .members
            .push(crate::tooling::review_pack::PackMemberSelection {
                id: "selected".into(),
                role: crate::tooling::review_pack::PackMemberRole::Result,
                required: true,
                format: crate::tooling::review_pack::PackFormat::ArrowIpcStream,
                document_version: None,
                source: None,
                references: vec![],
                result: Some(selected_coordinates),
                diagnostic_table: None,
                capture_scope: None,
            });
        let pack_directory = tempfile::tempdir().unwrap();
        let pack_path = pack_directory.path().join("selected-pack");
        let mut pack = crate::tooling::review_pack::PackSession::create(
            &pack_path,
            pack_selection,
            CancellationToken::new(),
        )
        .unwrap();
        let evidence = crate::tooling::frozen_inspection::FrozenEvidenceSet::default();
        let analysis_pool: Arc<dyn MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(128 << 20));
        let analysis = crate::tooling::review_analysis::analyze(
            &evidence,
            &crate::tooling::review_analysis::AnalysisRequest::new(
                pse_diagnostics::observation::OperationId {
                    process_instance: [1, 2],
                    sequence: 3,
                },
            ),
            &analysis_pool,
            &CancellationToken::new(),
        )
        .unwrap();
        // The public composition shares one result-owner path for multiple selected fields.
        let composed_path = pack_directory.path().join("composed-pack");
        let make_selection =
            |member: &str, minimum: f64, maximum: f64| super::super::ReviewScientificSelection {
                member: member.into(),
                run: "projection".into(),
                attempt: fence.attempt().into(),
                relation: "runtime.simulation_samples".into(),
                row_start: 0,
                row_end: u64::MAX,
                output: Some(super::super::ReviewOutputSelection {
                    owner: second,
                    field: "value".into(),
                    partition: "0".into(),
                    minimum_bits: Some(minimum.to_bits()),
                    maximum_bits: Some(maximum.to_bits()),
                    missing: None,
                }),
            };
        let request = crate::tooling::review_pack::PackExportRequest {
            scientific_results: vec![
                make_selection("early", 20.0, 21.0),
                make_selection("late", 22.0, 22.0),
            ],
            ..Default::default()
        };
        let composed = crate::tooling::review_export::export_review_pack(
            &runtime,
            &composed_path,
            &evidence,
            &analysis,
            request,
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            composed.publication,
            crate::tooling::review_pack::PackPublication::Sealed
        );
        let portable =
            crate::tooling::review_pack::PackReader::open(&composed_path, 8 << 20).unwrap();
        for (member, expected) in [("early", 2), ("late", 1)] {
            let stream = datafusion::arrow::ipc::reader::StreamReader::try_new(
                portable.open_member(member).unwrap(),
                None,
            )
            .unwrap();
            assert_eq!(
                stream.map(|batch| batch.unwrap().num_rows()).sum::<usize>(),
                expected
            );
            let declared = composed
                .members
                .iter()
                .find(|item| item.selection.id == member)
                .unwrap()
                .selection
                .result
                .as_ref()
                .unwrap();
            assert_eq!(declared.attempt, fence.attempt());
            assert_eq!(declared.usability, PackScientificState::Unknown);
        }
        pack.add_frozen(&evidence).unwrap();
        pack.add_analysis(&analysis).unwrap();
        pack.export_selected_result("selected", selected_reader)
            .await
            .unwrap();
        assert!(selected_borrower.upgrade().is_none());
        // Other original fixture selections remain independent live borrowers.
        assert!(read.sets().iter().any(|set| set.row_count > 0));
        pack.seal().unwrap();
        let portable = crate::tooling::review_pack::PackReader::open(&pack_path, 1 << 20).unwrap();
        let stream = datafusion::arrow::ipc::reader::StreamReader::try_new(
            portable.open_member("selected").unwrap(),
            None,
        )
        .unwrap();
        let mut times = Vec::new();
        for batch in stream {
            let batch = batch.unwrap();
            let time = batch
                .column_by_name("time")
                .unwrap()
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap();
            for i in 0..batch.num_rows() {
                times.push(time.value(i));
            }
            // Portable streams retain canonical transport metadata in addition
            // to the original relation fields. Inspect those fields directly.
            assert_eq!(
                batch
                    .schema()
                    .metadata()
                    .get("pse.canonical.field")
                    .map(String::as_str),
                Some("value")
            );
            let symbols = batch
                .column_by_name("symbol_id")
                .unwrap()
                .as_any()
                .downcast_ref::<datafusion::arrow::array::FixedSizeBinaryArray>()
                .unwrap();
            for i in 0..batch.num_rows() {
                assert_eq!(symbols.value(i), second.as_bytes());
            }
        }
        assert_eq!(times, vec![20.0, 21.0, 22.0]);
        drop(trajectory);
        drop(zero);
        drop(missing);
        drop(late);
        drop(none);
        drop(read);
        pse_operations::testing::drain_result_releases(store)
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn canonical_connected_results_reopens_declared_arrow_exact_range_and_cancellation() {
        let runtime = super::super::durable_tests::durable_runtime();
        let store = runtime.canonical_store().unwrap();
        let source = store.edit("problem", None, "source", &[]).await.unwrap();
        store
            .begin_run(&RunRequest {
                key: "run".into(),
                revision: source.clone(),
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run("run", "claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let validation = runtime.validation_context();
        let mut builder =
            solve_metrics::Builder::with_registry(runtime.registry(), 4, &validation).unwrap();
        for (step, value) in [Some(1.0), Some(-0.0), None, Some(4.0)]
            .into_iter()
            .enumerate()
        {
            builder
                .push(solve_metrics::Row {
                    run_id: pse_operations::mint_id(),
                    step: step as i64,
                    namespace: "fixture".into(),
                    name: "temperature".into(),
                    kind: if value.is_some() {
                        NativeMetricKind::Real
                    } else {
                        NativeMetricKind::Unavailable
                    },
                    real: value,
                    integer: None,
                    boolean: None,
                    text: None,
                    unavailable: if value.is_none() {
                        Some(pse_model::generated::enums::EvidenceUnavailableReason::Nonfinite)
                    } else {
                        None
                    },
                })
                .unwrap();
        }
        let checked = builder.finish().unwrap();
        let payload = result_files::encode_result_file(checked.batch()).unwrap();
        let name = solve_metrics::RELATION_ID.to_string();
        let set = result_set_key(fence.attempt(), &name);
        let key = result_batch_key(fence.attempt(), &set, 0);
        let metadata = BlockMetadata {
            key: key.clone(),
            result_set: set,
            batch: key,
            output: name.clone(),
            partition: "0".into(),
            ordinal: 0,
            start: 0,
            end: 4,
            rows: 4,
            columns: checked.batch().num_columns() as u64,
            coordinate_min: None,
            coordinate_max: None,
            payload_bytes: payload.len() as u64,
            payload_digest: result_payload_digest(&payload),
            statistics: None,
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        };
        let intent = store
            .reserve_result_file(&fence, "fixture-file", &result_payload_digest(&payload))
            .await
            .unwrap();
        let file = store
            .seal_file(
                &intent,
                &payload,
                pse_operations::canonical_files::ARROW_IPC_FILE,
                1,
                &name,
            )
            .await
            .unwrap();
        store
            .append_result_block(
                &fence,
                "write",
                &name,
                0,
                pse_operations::canonical_execution::ResultPayload::from_sealed(file, 0).unwrap(),
                4,
                &metadata,
            )
            .await
            .unwrap();
        let progress = super::super::ProgressEventDocument {
            version: pse_model::document::Version,
            source: None,
            phase: "fixture".into(),
            elapsed_seconds: 0.25,
            values: [(
                "exact_counter".into(),
                super::super::ProgressMetricDocument::Integer((1i64 << 53) + 1),
            )]
            .into(),
            incumbent: None,
            step: Some(0),
            sequence: Some(0),
            at: Some(1),
        };
        let progress_payload = serde_json::to_vec(&vec![progress]).unwrap();
        store
            .append_result_batch(
                &fence,
                "progress",
                "__progress",
                0,
                pse_operations::canonical_execution::ResultPayload::new(progress_payload.to_vec())
                    .unwrap(),
                1,
            )
            .await
            .unwrap();
        let closed = store.close_result_ingestion(&fence, "close").await.unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        let completion = super::super::durable::StoredCompletion {
            version: 1,
            attempt_id: pse_operations::mint_id(),
            completion: None,
            termination: super::super::durable::TerminationDetail {
                version: pse_model::document::Version,
                cause: super::super::durable::TerminationCause::Assessment {
                    usable: false,
                    candidate_use: vec![],
                    diagnostics: vec![],
                },
                point: None,
                retry_failure: Some(pse_model::study::RetryFailure::Deterministic),
                effect: pse_model::study::EffectState::Absent,
            },
            state: pse_model::generated::enums::AttemptState::Failed,
            progress: super::super::ProgressCoverage::Unknown {},
        };
        let completion = serde_json::to_vec(&serde_json::json!({ "version": 1, "batch_count": 0, "payload_bytes": 0, "digest": "", "inline": completion })).unwrap();
        // SAFETY: finite generated rows are explicitly classified as a partial fixture, not usable success.
        unsafe {
            store.seal_attempt(
                &manifest,
                "seal",
                TerminalClass::Partial,
                &completion,
                pse_operations::canonical_execution::SeedEligibility::Unavailable,
            )
        }
        .await
        .unwrap();
        assert_eq!(
            runtime.run_record("run").await.unwrap().batch().num_rows(),
            1
        );
        assert_eq!(
            runtime
                .attempt_record(fence.attempt())
                .await
                .unwrap()
                .batch()
                .num_rows(),
            1
        );
        assert_eq!(
            runtime
                .result_manifest(fence.attempt())
                .await
                .unwrap()
                .batch()
                .num_rows(),
            1
        );
        let mut latest = runtime
            .latest_results(
                "problem",
                &[TerminalClass::Partial],
                "runtime.solve_metrics",
                0,
                4,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(latest.selection().attempt().key, fence.attempt());
        assert_eq!(latest.next_batch().await.unwrap().unwrap().num_rows(), 4);
        let mut history = runtime
            .progress("run", fence.attempt(), CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(history.run(), "run");
        assert_eq!(history.attempt(), fence.attempt());
        let events = history.next_page().await.unwrap().unwrap();
        assert_eq!(events.len(), 1);
        assert!(
            matches!(events[0].values["exact_counter"],super::super::ProgressMetricDocument::Integer(value) if value==(1i64<<53)+1)
        );
        assert!(history.next_page().await.unwrap().is_none());
        let cancel = CancellationToken::new();
        let mut reader = runtime
            .results(
                "run",
                fence.attempt(),
                "runtime.solve_metrics",
                1,
                3,
                cancel.clone(),
            )
            .await
            .unwrap();
        assert_eq!(
            reader
                .selection()
                .attempt()
                .outcome
                .as_ref()
                .map(|value| value.as_str()),
            Some("partial")
        );
        let batch = reader.next_batch().await.unwrap().unwrap();
        assert_eq!(batch.num_rows(), 2);
        let values = batch
            .column_by_name("real")
            .unwrap()
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(values.value(0).to_bits(), (-0.0f64).to_bits());
        assert!(values.is_null(1));
        assert!(reader.next_batch().await.unwrap().is_none());
        let mut cancelled = runtime
            .results(
                "run",
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                4,
                cancel.clone(),
            )
            .await
            .unwrap();
        cancel.cancel();
        assert!(cancelled.next_batch().await.is_err());
        // Already yielded Arrow buffers remain independent of stream cancellation.
        assert_eq!(values.value(0).to_bits(), (-0.0f64).to_bits());
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("results.arrow");
        let mut export = runtime
            .results(
                "run",
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                4,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        export.export_ipc(&destination).await.unwrap();
        assert!(destination.is_file());
        assert!(!directory.path().join("results.arrow.incomplete").exists());
        let mut file = datafusion::arrow::ipc::reader::StreamReader::try_new(
            std::fs::File::open(&destination).unwrap(),
            None,
        )
        .unwrap();
        assert_eq!(
            file.schema().metadata().get("pse.canonical.attempt"),
            Some(&fence.attempt().to_owned())
        );
        assert_eq!(file.next().unwrap().unwrap().num_rows(), 4);
        assert!(file.next().is_none());
        // The pack uses this actual protected selected exporter, with exact
        // row coordinates, terminal lineage and scientific usability unchanged.
        use crate::tooling::review_pack::{
            PackAvailability, PackFormat, PackMemberRole, PackMemberSelection, PackPublication,
            PackReader, PackSelection, PackSession,
        };
        let evidence = crate::tooling::frozen_inspection::FrozenEvidenceSet::default();
        let analysis_pool: Arc<dyn MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(128 << 20));
        let analysis = crate::tooling::review_analysis::analyze(
            &evidence,
            &crate::tooling::review_analysis::AnalysisRequest::new(
                pse_diagnostics::observation::OperationId {
                    process_instance: [1, 2],
                    sequence: 3,
                },
            ),
            &analysis_pool,
            &CancellationToken::new(),
        )
        .unwrap();
        let mut report_bytes = 0;
        for limited in [false, true] {
            let reader = runtime
                .results(
                    "run",
                    fence.attempt(),
                    "runtime.solve_metrics",
                    1,
                    3,
                    CancellationToken::new(),
                )
                .await
                .unwrap();
            let protected_borrower = Arc::downgrade(&reader.reader.read);
            let original_selection = reader
                .pack_selection(PackScientificState::Unusable)
                .unwrap();
            assert_eq!(original_selection.row_start, 1);
            assert_eq!(original_selection.row_end, 3);
            let mut selection = PackSelection::default();
            if limited {
                selection.maximum_output_bytes = report_bytes + 16;
            }
            selection.members.push(PackMemberSelection {
                id: "scientific".into(),
                role: PackMemberRole::Result,
                required: true,
                format: PackFormat::ArrowIpcStream,
                document_version: None,
                source: None,
                references: vec![],
                result: Some(original_selection),
                diagnostic_table: None,
                capture_scope: None,
            });
            let pack_path = directory.path().join(if limited {
                "limited-pack"
            } else {
                "sealed-pack"
            });
            let mut pack =
                PackSession::create(&pack_path, selection, CancellationToken::new()).unwrap();
            pack.add_frozen(&evidence).unwrap();
            pack.add_analysis(&analysis).unwrap();
            report_bytes = pack.index().members.iter().map(|m| m.readable_bytes).sum();
            if limited {
                // Actual protected IPC writer refuses before exceeding allowance.
                assert!(pack.export_result("scientific", reader).await.is_err());
                assert!(protected_borrower.upgrade().is_none());
                assert!(pack.seal().is_err());
                assert_eq!(
                    PackReader::open(&pack_path, 1 << 20)
                        .unwrap()
                        .index()
                        .publication,
                    PackPublication::Incomplete
                );
                continue;
            }
            pack.export_result("scientific", reader).await.unwrap();
            assert!(protected_borrower.upgrade().is_none());
            let sealed = pack.seal().unwrap();
            assert_eq!(sealed.publication, PackPublication::Sealed);
            let scientific = sealed
                .members
                .iter()
                .find(|member| member.selection.id == "scientific")
                .unwrap();
            assert_eq!(scientific.availability, PackAvailability::Available);
            assert_eq!(
                scientific.selection.result.as_ref().unwrap().usability,
                PackScientificState::Unusable
            );
            assert_eq!(
                scientific.selection.result.as_ref().unwrap().terminal_class,
                "partial"
            );
            let reopened = PackReader::open(&pack_path, 1 << 20).unwrap();
            let mut stream = datafusion::arrow::ipc::reader::StreamReader::try_new(
                reopened.open_member("scientific").unwrap(),
                None,
            )
            .unwrap();
            assert_eq!(
                stream
                    .schema()
                    .metadata()
                    .get("pse.canonical.row-start")
                    .map(String::as_str),
                Some("1")
            );
            assert_eq!(
                stream
                    .schema()
                    .metadata()
                    .get("pse.canonical.row-end")
                    .map(String::as_str),
                Some("3")
            );
            let batch = stream.next().unwrap().unwrap();
            assert_eq!(batch.num_rows(), 2);
            let selected = batch
                .column_by_name("real")
                .unwrap()
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap();
            assert_eq!(selected.value(0).to_bits(), (-0.0_f64).to_bits());
            assert!(selected.is_null(1));
            assert!(stream.next().is_none());
        }
        let token = CancellationToken::new();
        let mut interrupted = runtime
            .results(
                "run",
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                4,
                token.clone(),
            )
            .await
            .unwrap();
        token.cancel();
        let missing = directory.path().join("cancelled.arrow");
        assert!(interrupted.export_ipc(&missing).await.is_err());
        assert!(!missing.exists());
        assert!(
            directory
                .path()
                .join("cancelled.arrow.incomplete")
                .is_file()
        );
        let original = std::fs::read(&destination).unwrap();
        let mut overwrite = runtime
            .results(
                "run",
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                4,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert!(overwrite.export_ipc(&destination).await.is_err());
        assert_eq!(std::fs::read(&destination).unwrap(), original);
        // An admitted transport receipt can still be corrupt scientific IPC.
        // Failure after an already yielded prefix must not publish a complete file.
        store
            .begin_run(&RunRequest {
                key: "malformed".into(),
                revision: source,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let broken = store
            .claim_run(
                "malformed",
                "malformed-claim",
                "worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let broken_set = result_set_key(broken.attempt(), &name);
        for (ordinal, start, end, bytes) in [(0, 0, 4, payload.clone()), (1, 4, 5, vec![8])] {
            let key = result_batch_key(broken.attempt(), &broken_set, ordinal);
            let block = BlockMetadata {
                key: key.clone(),
                result_set: broken_set.clone(),
                batch: key,
                output: name.clone(),
                partition: "0".into(),
                ordinal,
                start,
                end,
                rows: end - start,
                columns: checked.batch().num_columns() as u64,
                coordinate_min: None,
                coordinate_max: None,
                payload_bytes: bytes.len() as u64,
                payload_digest: result_payload_digest(&bytes),
                statistics: None,
                interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
            };
            let intent = store
                .reserve_result_file(
                    &broken,
                    &format!("malformed-file-{ordinal}"),
                    &result_payload_digest(&bytes),
                )
                .await
                .unwrap();
            let file = store
                .seal_file(
                    &intent,
                    &bytes,
                    pse_operations::canonical_files::ARROW_IPC_FILE,
                    1,
                    &name,
                )
                .await
                .unwrap();
            store
                .append_result_block(
                    &broken,
                    &format!("malformed-write-{ordinal}"),
                    &name,
                    ordinal,
                    pse_operations::canonical_execution::ResultPayload::from_sealed(file, 0)
                        .unwrap(),
                    end - start,
                    &block,
                )
                .await
                .unwrap();
        }
        let closed = store
            .close_result_ingestion(&broken, "malformed-close")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        // SAFETY: deliberately corrupt transport evidence is explicitly partial.
        unsafe {
            store.seal_attempt(
                &manifest,
                "malformed-seal",
                TerminalClass::Partial,
                &[1],
                pse_operations::canonical_execution::SeedEligibility::Unavailable,
            )
        }
        .await
        .unwrap();
        let mut late = runtime
            .results(
                "malformed",
                broken.attempt(),
                "runtime.solve_metrics",
                0,
                5,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(late.next_batch().await.unwrap().unwrap().num_rows(), 4);
        assert!(late.next_batch().await.is_err());
        let mut late_export = runtime
            .results(
                "malformed",
                broken.attempt(),
                "runtime.solve_metrics",
                0,
                5,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let partial = directory.path().join("partial.arrow");
        assert!(late_export.export_ipc(&partial).await.is_err());
        assert!(!partial.exists());
        assert!(directory.path().join("partial.arrow.incomplete").is_file());
        let broken_reader = runtime
            .results(
                "malformed",
                broken.attempt(),
                "runtime.solve_metrics",
                0,
                5,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let broken_borrower = Arc::downgrade(&broken_reader.reader.read);
        let mut broken_selection = PackSelection::default();
        broken_selection.members.push(PackMemberSelection {
            id: "broken".into(),
            role: PackMemberRole::Result,
            required: true,
            format: PackFormat::ArrowIpcStream,
            document_version: None,
            source: None,
            references: vec![],
            result: Some(
                broken_reader
                    .pack_selection(PackScientificState::Unusable)
                    .unwrap(),
            ),
            diagnostic_table: None,
            capture_scope: None,
        });
        let broken_pack_path = directory.path().join("broken-pack");
        let mut broken_pack = PackSession::create(
            &broken_pack_path,
            broken_selection,
            CancellationToken::new(),
        )
        .unwrap();
        broken_pack.add_frozen(&evidence).unwrap();
        broken_pack.add_analysis(&analysis).unwrap();
        assert!(
            broken_pack
                .export_result("broken", broken_reader)
                .await
                .is_err()
        );
        assert!(broken_borrower.upgrade().is_none());
        assert!(broken_pack.seal().is_err());
        let incomplete = PackReader::open(&broken_pack_path, 1 << 20).unwrap();
        assert_eq!(incomplete.index().publication, PackPublication::Incomplete);
        let member = incomplete
            .index()
            .members
            .iter()
            .find(|m| m.selection.id == "broken")
            .unwrap();
        assert!(!member.finalized);
        assert!(member.readable_bytes > 0);
        let mut prefix = datafusion::arrow::ipc::reader::StreamReader::try_new(
            incomplete.open_member("broken").unwrap(),
            None,
        )
        .unwrap();
        assert_eq!(prefix.next().unwrap().unwrap().num_rows(), 4);
        assert!(prefix.next().is_none());
        // Drain is a whole-client boundary only after the independent original
        // fixture readers have completed their own selected work and dropped.
        drop((
            latest,
            history,
            reader,
            cancelled,
            export,
            interrupted,
            overwrite,
            late,
            late_export,
        ));
        pse_operations::testing::drain_result_releases(store)
            .await
            .unwrap();
    }
}
