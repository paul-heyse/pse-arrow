// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact immutable result inventories shared by typed readers and native queries.

use super::{RetainedDataServices, WorkflowError, contract, result_files};
use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::{Session, TableProvider, streaming::StreamingTable},
    common::{DataFusionError, Result as NativeResult},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{Expr, TableProviderFilterPushDown, TableType},
    physical_plan::{
        ExecutionPlan, SendableRecordBatchStream, stream::RecordBatchStreamAdapter,
        streaming::PartitionStream,
    },
};
use pse_columnar::{CancellationToken, MemoryConsumer, MemoryReservation};
use pse_model::generated::runtime::{
    canonical_result_batches::Row as FileBatch, canonical_result_blocks::Row as Block,
};
use pse_operations::{canonical::MetadataResponse, canonical_results::ResultRead};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug)]
struct SnapshotInner {
    decode: Arc<ProtectedResultDecode>,
    batches: BTreeMap<String, FileBatch>,
    blocks: Vec<Block>,
    positions: BTreeMap<String, usize>,
    start: u64,
    end: u64,
    _owner: MemoryReservation,
}

/// One protected byte decoder and receiving admission scope. Frozen full
/// inventories and bounded selected pages supply their own exact descriptors.
#[derive(Debug)]
pub(super) struct ProtectedResultDecode {
    pub(super) read: Arc<ResultRead>,
    pub(super) read_owner: Arc<MemoryReservation>,
    capture_custody: std::sync::OnceLock<pse_operations::canonical_study_workbench::StudyCapture>,
    decode: Arc<ResultBlockDecode>,
}
impl std::ops::Deref for ProtectedResultDecode {
    type Target = ResultBlockDecode;
    fn deref(&self) -> &Self::Target {
        &self.decode
    }
}
/// One actual byte decoder shared by ordinary terminal reads and exact captures.
#[derive(Debug)]
pub(super) struct ResultBlockDecode {
    pub(super) services: RetainedDataServices,
    relation: pse_ids::SemanticId,
    pub(super) schema: SchemaRef,
    admission: Arc<pse_relations::validate::ValidationContext>,
    cache: tokio::sync::Mutex<Option<CachedFile>>,
    _owner: MemoryReservation,
}
/// An opaque original read grant or the exact captured occurrence, never an ID alone.
#[derive(Clone, Debug)]
pub(super) enum ResultPayloadAuthority {
    Terminal {
        read: Arc<ResultRead>,
        owner: Arc<MemoryReservation>,
        capture: Option<pse_operations::canonical_study_workbench::StudyCapture>,
    },
    Capture {
        read: Arc<pse_operations::canonical_capture_results::CapturedResultRead>,
    },
}
impl ResultPayloadAuthority {
    fn custody(&self) -> pse_operations::transport::client::RetainedRequest {
        use pse_operations::transport::client::RetainedRequest;
        match self {
            Self::Terminal { read, capture, .. } => {
                let custody = RetainedRequest::from(read.clone());
                match capture {
                    Some(capture) => RetainedRequest::combine(custody, capture.clone()),
                    None => custody,
                }
            }
            Self::Capture { read } => RetainedRequest::from(read.clone()),
        }
    }
    fn allocations(
        &self,
        response: Arc<MemoryReservation>,
    ) -> pse_operations::transport::client::ResponseAllocation {
        use pse_operations::transport::client::ResponseAllocation;
        match self {
            Self::Terminal { owner, .. } => ResponseAllocation::combine(owner.clone(), response),
            Self::Capture { .. } => ResponseAllocation::from(response),
        }
    }
}
impl ProtectedResultDecode {
    pub(super) fn new(
        services: &RetainedDataServices,
        read: Arc<ResultRead>,
        read_owner: Arc<MemoryReservation>,
        relation: pse_ids::SemanticId,
        schema: SchemaRef,
    ) -> Result<Arc<Self>, WorkflowError> {
        let required = read
            .retained_bytes()?
            .checked_add(MetadataResponse::Protection.allocation_bytes())
            .ok_or_else(|| contract("result protected metadata extent"))?;
        if read_owner.size() < required {
            return Err(contract("result read metadata is not prepaid"));
        }
        read.retain_metadata_owner(read_owner.clone())?;
        let decode = ResultBlockDecode::new(services, relation, schema)?;
        decode
            ._owner
            .try_grow(size_of::<Self>() + 2 * size_of::<usize>())
            .map_err(pse_engine::EngineError::from)?;
        Ok(Arc::new(Self {
            read,
            read_owner,
            capture_custody: std::sync::OnceLock::new(),
            decode,
        }))
    }
    pub(super) fn retain_capture(
        &self,
        capture: pse_operations::canonical_study_workbench::StudyCapture,
    ) -> Result<(), WorkflowError> {
        if self
            .capture_custody
            .get()
            .is_some_and(|old| old.key() != capture.key())
        {
            return Err(contract("result capture custody differs"));
        }
        let _ = self.capture_custody.set(capture);
        Ok(())
    }
    fn authority(&self) -> ResultPayloadAuthority {
        ResultPayloadAuthority::Terminal {
            read: self.read.clone(),
            owner: self.read_owner.clone(),
            capture: self.capture_custody.get().cloned(),
        }
    }
    pub(super) async fn read_block(
        &self,
        block: &Block,
        descriptor: &FileBatch,
        cancel: &CancellationToken,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        self.decode
            .read_block(self.authority(), block, descriptor, cancel)
            .await
    }
    pub(super) async fn validate_pruned_block(
        &self,
        block: &Block,
        descriptor: &FileBatch,
        cancel: &CancellationToken,
    ) -> Result<(), WorkflowError> {
        self.decode
            .access_block(self.authority(), block, descriptor, cancel, false)
            .await
            .map(|_| ())
    }
}
impl ResultBlockDecode {
    pub(super) fn new(
        services: &RetainedDataServices,
        relation: pse_ids::SemanticId,
        schema: SchemaRef,
    ) -> Result<Arc<Self>, WorkflowError> {
        let owner = MemoryConsumer::new("canonical:protected-result-decoder")
            .register(&services.pool());
        owner
            .try_grow(size_of::<Self>() + 2 * size_of::<usize>())
            .map_err(pse_engine::EngineError::from)?;
        Ok(Arc::new(Self {
            services: services.clone(),
            relation,
            schema,
            admission: services.admission_context(),
            cache: tokio::sync::Mutex::new(None),
            _owner: owner,
        }))
    }
}

#[derive(Debug)]
struct CachedFile {
    instance: String,
    reader: pse_columnar::ipc_file::AdmittedFile<result_files::ResultFileSource>,
    _source: Arc<pse_columnar::AllocationLease>,
    _owner: MemoryReservation,
}

async fn decoder_turn<'a>(
    cache: &'a tokio::sync::Mutex<Option<CachedFile>>,
    cancel: &CancellationToken,
) -> Result<tokio::sync::MutexGuard<'a, Option<CachedFile>>, WorkflowError> {
    let turn = cancel
        .until_cancelled(cache.lock())
        .await
        .map_err(pse_engine::EngineError::from)?;
    cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
    Ok(turn)
}

/// One terminal selection and a frozen explicit file/batch inventory. Clones,
/// physical plans and partition streams retain the same nonexpiring read ticket.
#[derive(Clone, Debug)]
pub struct ResultSnapshot(Arc<SnapshotInner>);
impl ResultSnapshot {
    #[expect(
        clippy::too_many_arguments,
        reason = "snapshot admission keeps the prepaid read owner, relation schema and selected set bounds explicit"
    )]
    pub(super) async fn capture(
        services: &RetainedDataServices,
        read: Arc<ResultRead>,
        read_owner: Arc<MemoryReservation>,
        relation: pse_ids::SemanticId,
        set: &str,
        schema: SchemaRef,
        start: u64,
        end: u64,
    ) -> Result<Self, WorkflowError> {
        use pse_model::HeapUsage;
        let required = read
            .retained_bytes()?
            .checked_add(MetadataResponse::Protection.allocation_bytes())
            .ok_or_else(|| contract("snapshot protected metadata extent"))?;
        if read_owner.size() < required {
            return Err(contract("snapshot read metadata is not prepaid"));
        }
        read.retain_metadata_owner(read_owner.clone())?;
        let owner = MemoryConsumer::new("canonical:result-snapshot-inventory")
            .register(&services.pool());
        owner
            .try_grow(size_of::<SnapshotInner>())
            .map_err(pse_engine::EngineError::from)?;
        let mut batches = BTreeMap::new();
        let mut blocks = Vec::new();
        let mut positions = BTreeMap::new();
        let mut after = None;
        let mut previous = None;
        if start < end {
            loop {
                // Each bounded RPC page has its own prepaid temporary response scope.
                // Transfer its exact retained strings/rows before growing inventories.
                let response = super::connected_results::pooled_response_owner(
                    &services.pool(),
                    "canonical:result-snapshot-joined-page",
                    MetadataResponse::JoinedBlockPage,
                )?;
                if let Some(context) = pse_diagnostics::observation::current() {
                    context.count("canonical.result.joined_page_requests", "count", 1);
                }
                let joined = super::connected_results::protected_response_scope(
                    read.clone(),
                    pse_operations::transport::client::ResponseAllocation::combine(
                        read_owner.clone(),
                        response.clone(),
                    ),
                    services.canonical_store()?.result_joined_block_page(
                        &read,
                        set,
                        &relation.to_string(),
                        "0",
                        start,
                        end,
                        after,
                    ),
                )
                .await?;
                let finished = joined.exhausted;
                after = joined.next.or(after);
                let page = joined.blocks;
                let descriptors = joined.batches;
                let bytes =
                    page.iter()
                        .zip(&descriptors)
                        .try_fold(0usize, |bytes, (block, batch)| {
                            bytes
                                .checked_add(
                                    size_of::<Block>()
                                        + block.heap_bytes()
                                        + block.key.capacity()
                                        + 128
                                        + size_of::<FileBatch>()
                                        + batch.heap_bytes()
                                        + batch.key.capacity()
                                        + 128,
                                )
                                .ok_or_else(|| contract("snapshot inventory extent"))
                        })?;
                owner
                    .try_grow(bytes)
                    .map_err(pse_engine::EngineError::from)?;
                blocks
                    .try_reserve_exact(page.len())
                    .map_err(|_| contract("snapshot inventory allocation refused"))?;
                for (block, batch) in page.into_iter().zip(descriptors) {
                    if batch.key != block.batch
                        || batch.ordinal != block.ordinal
                        || batch.row_count != block.rows
                        || batch.digest != block.payload_digest
                        || batch.payload_bytes != block.payload_bytes
                        || previous.is_some_and(|end| block.start < end)
                    {
                        return Err(contract("snapshot logical coverage differs"));
                    }
                    previous = Some(block.end);
                    positions.insert(block.key.clone(), blocks.len());
                    if batches.insert(batch.key.clone(), batch).is_some() {
                        return Err(contract("duplicate frozen file batch"));
                    }
                    blocks.push(block);
                }
                drop(response);
                if finished {
                    break;
                }
            }
        }
        Ok(Self(Arc::new(SnapshotInner {
            decode: ProtectedResultDecode::new(services, read, read_owner, relation, schema)?,
            batches,
            blocks,
            positions,
            start,
            end,
            _owner: owner,
        })))
    }
    /// Exact registry relation schema, including its original metadata.
    pub fn schema(&self) -> SchemaRef {
        self.0.decode.schema.clone()
    }
    /// Actual selected attempt and immutable closure, including partiality class.
    pub fn selection(&self) -> &ResultRead {
        &self.0.decode.read
    }
    pub(super) fn blocks(&self) -> &[Block] {
        &self.0.blocks
    }
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(super) fn inventory_reserved_bytes(&self) -> usize {
        self.0._owner.size() + self.0.decode.read_owner.size() + self.0.decode._owner.size()
    }
    /// Existing configured native state, including its functions and shared pool.
    pub fn session_state(&self) -> SessionState {
        self.0
            .decode
            .services
            .session_state()
    }
    /// A native provider over this frozen inventory; no directory discovery.
    pub fn table_provider(&self) -> Arc<dyn TableProvider> {
        Arc::new(SnapshotProvider(FrozenSnapshot::Full(self.clone())))
    }

    pub(super) async fn read_block(
        &self,
        block: &Block,
        cancel: &CancellationToken,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        if self
            .0
            .positions
            .get(&block.key)
            .and_then(|index| self.0.blocks.get(*index))
            != Some(block)
        {
            return Err(contract("result block differs from frozen snapshot"));
        }
        let descriptor = self
            .0
            .batches
            .get(&block.batch)
            .ok_or_else(|| contract("block outside frozen snapshot"))?;
        self.0.decode.read_block(block, descriptor, cancel).await
    }
    pub(super) fn decoder(&self) -> Arc<ProtectedResultDecode> {
        self.0.decode.clone()
    }
}

impl ResultBlockDecode {
    pub(super) async fn read_block(
        &self,
        authority: ResultPayloadAuthority,
        block: &Block,
        descriptor: &FileBatch,
        cancel: &CancellationToken,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        self.access_block(authority, block, descriptor, cancel, true)
            .await?
            .ok_or_else(|| contract("protected native batch decode absent"))
    }
    async fn access_block(
        &self,
        authority: ResultPayloadAuthority,
        block: &Block,
        descriptor: &FileBatch,
        cancel: &CancellationToken,
        decode: bool,
    ) -> Result<Option<pse_relations::columnar::FieldCheckedBatch>, WorkflowError> {
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        if descriptor.key != block.batch
            || descriptor.ordinal != block.ordinal
            || descriptor.row_count != block.rows
            || descriptor.digest != block.payload_digest
            || descriptor.payload_bytes != block.payload_bytes
        {
            return Err(contract("protected result descriptor coverage differs"));
        }
        let locator = pse_operations::canonical_execution::result_payload_locator(descriptor)?;
        // Waiters own no file-shaped grant. Reserve only once this decoder can
        // actually execute, preserving headroom for its current reader/aliases.
        let mut cache = decoder_turn(&self.cache, cancel).await?;
        if cache
            .as_ref()
            .is_none_or(|cached| cached.instance != locator.payload_identity())
        {
            // The framing hash uses a fixed 64 KiB stack reader; source/control
            // allocations coexist with their separately prepaid RPC response.
            let metadata = MemoryConsumer::new("canonical:result-file-cache-control")
                .register(&self.services.pool());
            metadata
                .try_grow(
                    64 * 1024 + size_of::<CachedFile>() + locator.payload_identity().len() * 2,
                )
                .map_err(pse_engine::EngineError::from)?;
            // Both sequential catalog RPCs inside open_result_file must retain
            // the exact read ticket and a separately prepaid metadata response.
            // The file-shaped decode/footer grants cannot bound native response metadata,
            // and connection activity alone does not retain this ResultRead.
            let response = super::connected_results::pooled_response_owner(
                &self.services.pool(),
                "canonical:result-file-open-response",
                match locator {
                    pse_operations::canonical_execution::PayloadLocator::Inline { .. } => {
                        MetadataResponse::InlinePayload
                    }
                    pse_operations::canonical_execution::PayloadLocator::Sealed { .. } => {
                        MetadataResponse::Header
                    }
                },
            )?;
            let (source, count, retained_source) = match &locator {
                pse_operations::canonical_execution::PayloadLocator::Sealed { .. } => {
                    let file = super::connected_results::protected_response_scope(
                        authority.custody(),
                        authority.allocations(response.clone()),
                        async {
                            match &authority {
                                ResultPayloadAuthority::Terminal {
                                    read,
                                    owner,
                                    capture,
                                } => {
                                    let _custody = (owner, capture);
                                    self.services
                                        .canonical_store()?
                                        .open_result_file(read, descriptor)
                                        .await
                                        .map_err(WorkflowError::from)
                                }
                                ResultPayloadAuthority::Capture { read } => read
                                    .open_result_file(descriptor)
                                    .await
                                    .map_err(WorkflowError::from),
                            }
                        },
                    )
                    .await?;
                    let count = usize::try_from(file.descriptor().batch_count)
                        .map_err(|_| contract("file batch count extent"))?;
                    let retained = file.retained_bytes()?;
                    (
                        result_files::ResultFileSource::Sealed(Box::new(file)),
                        count,
                        retained,
                    )
                }
                pse_operations::canonical_execution::PayloadLocator::Inline { .. } => {
                    let body = super::connected_results::protected_response_scope(
                        authority.custody(),
                        authority.allocations(response.clone()),
                        async {
                            match &authority {
                                ResultPayloadAuthority::Terminal {
                                    read,
                                    owner,
                                    capture,
                                } => {
                                    let _custody = (owner, capture);
                                    self.services
                                        .canonical_store()?
                                        .read_inline_payload(read, descriptor)
                                        .await
                                        .map_err(WorkflowError::from)
                                }
                                ResultPayloadAuthority::Capture { read } => read
                                    .read_inline_payload(descriptor)
                                    .await
                                    .map_err(WorkflowError::from),
                            }
                        },
                    )
                    .await?;
                    let count = usize::try_from(body.batch_count)
                        .map_err(|_| contract("inline batch count extent"))?;
                    let bytes = body.body.into_vec();
                    let retained = bytes.capacity();
                    (
                        result_files::ResultFileSource::Inline(std::io::Cursor::new(bytes)),
                        count,
                        retained,
                    )
                }
            };
            if retained_source > response.size() {
                return Err(contract("file source metadata exceeds prepaid response"));
            }
            let source_owner = pse_columnar::AllocationLease::new(response.split(retained_source));
            let reader = result_files::open_file_reader(
                source,
                &self.schema,
                Some(count),
                self.services.pool(),
                cancel,
            )?;
            let instance = locator.payload_identity().to_owned();
            let retained = size_of::<CachedFile>() + instance.capacity();
            metadata.shrink(metadata.size() - retained);
            *cache = Some(CachedFile {
                instance,
                reader,
                _source: source_owner,
                _owner: metadata,
            });
        }
        let cached = cache
            .as_mut()
            .ok_or_else(|| contract("file cache absent"))?;
        let index = usize::try_from(descriptor.payload_index)
            .map_err(|_| contract("file batch index extent"))?;
        cached
            .reader
            .check_index(index)
            .map_err(pse_engine::EngineError::from)?;
        if !decode {
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("canonical.result.pruned_batches", "count", 1);
            }
            return Ok(None);
        }
        let decoded = cached
            .reader
            .read_batch(index, cancel)
            .map_err(pse_engine::EngineError::from)?;
        let batch = decoded.batch;
        drop(cache);
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("canonical.result.native_decodes", "count", 1);
            context.count(
                "canonical.result.decoded_array_bytes",
                "bytes",
                batch.get_array_memory_size() as u64,
            );
        }
        if batch.num_rows() as u64 != block.rows || batch.schema() != self.schema.clone() {
            return Err(contract("frozen file batch rows/schema differ"));
        }
        let spec = self
            .services
            .registry()
            .relation_by_id(self.relation)
            .ok_or_else(|| contract("snapshot relation unavailable"))?;
        let checked = pse_relations::columnar::FieldCheckedBatch::admit_owned(
            self.services.registry(),
            spec,
            batch,
            &self.admission,
            cancel,
        )
        .map_err(pse_engine::EngineError::from)?
        .with_export_owner(decoded.working_owner);
        pse_relations::result_batch::validate_scalar_keys(
            self.services.registry(),
            &checked,
            &self.services.pool(),
            cancel,
        )
        .map_err(pse_engine::EngineError::from)?;
        Ok(Some(checked))
    }
}

/// Exact selected row universe, frozen independently of a streaming cursor.
/// It retains the full relation schema and protected future-file descriptors.
#[derive(Clone, Debug)]
pub struct SelectedResultSnapshot(Arc<SelectedSnapshotInner>);
#[derive(Debug)]
struct SelectedSnapshotInner {
    decode: Arc<ProtectedResultDecode>,
    entries: Vec<SelectedBlock>,
    rows: u64,
    _owner: MemoryReservation,
}
#[derive(Debug)]
struct SelectedBlock {
    block: Block,
    descriptor: FileBatch,
    rows: SelectedRows,
}
/// A dense interval uses constant metadata; sparse offsets own their charged
/// actual allocation, never the complete decoded trajectory.
#[derive(Debug)]
pub(super) enum SelectedRows {
    Range {
        offset: usize,
        length: usize,
    },
    Sparse {
        offsets: Vec<u32>,
        _owner: MemoryReservation,
    },
}
impl SelectedRows {
    pub(super) fn from_mask(
        mask: datafusion::arrow::buffer::BooleanBuffer,
        base: usize,
        owner: MemoryReservation,
    ) -> Result<Option<Self>, WorkflowError> {
        // Drop the bitmap before its scratch grant on every success/error path.
        struct Scratch {
            mask: datafusion::arrow::buffer::BooleanBuffer,
            owner: MemoryReservation,
        }
        let scratch = Scratch { mask, owner };
        if scratch.mask.inner().capacity() > scratch.owner.size() {
            return Err(contract("selected bitmap is not prepaid"));
        }
        let spans = {
            let mut spans = scratch.mask.set_slices();
            (spans.next(), spans.next())
        };
        let Some((start, end)) = spans.0 else {
            return Ok(None);
        };
        if spans.1.is_none() {
            let offset = base
                .checked_add(start)
                .ok_or_else(|| contract("selected row extent"))?;
            let last = base
                .checked_add(end - 1)
                .ok_or_else(|| contract("selected row extent"))?;
            u32::try_from(last).map_err(|_| contract("selected row exceeds Arrow take extent"))?;
            drop(scratch);
            return Ok(Some(Self::Range {
                offset,
                length: end - start,
            }));
        }
        let mut offsets = Vec::with_capacity(scratch.mask.count_set_bits());
        for index in scratch.mask.set_indices() {
            offsets.push(
                u32::try_from(
                    base.checked_add(index)
                        .ok_or_else(|| contract("selected row extent"))?,
                )
                .map_err(|_| contract("selected row exceeds Arrow take extent"))?,
            );
        }
        drop(scratch.mask);
        Ok(Some(Self::from_offsets(offsets, scratch.owner)?))
    }
    pub(super) fn from_offsets(
        offsets: Vec<u32>,
        owner: MemoryReservation,
    ) -> Result<Self, WorkflowError> {
        let bytes = offsets
            .capacity()
            .checked_mul(size_of::<u32>())
            .ok_or_else(|| contract("selected offset extent"))?;
        if bytes > owner.size() {
            return Err(contract("selected offsets are not prepaid"));
        }
        if offsets.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(contract("selected offsets repeat or reverse source rows"));
        }
        if offsets
            .windows(2)
            .all(|pair| pair[0].checked_add(1) == Some(pair[1]))
        {
            let offset = offsets.first().copied().unwrap_or(0) as usize;
            let length = offsets.len();
            drop(offsets);
            drop(owner);
            Ok(Self::Range { offset, length })
        } else {
            owner.shrink(owner.size() - bytes);
            Ok(Self::Sparse {
                offsets,
                _owner: owner,
            })
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::Range { length, .. } => *length,
            Self::Sparse { offsets, .. } => offsets.len(),
        }
    }
    pub(super) fn apply(
        &self,
        batch: &pse_relations::columnar::FieldCheckedBatch,
        pool: &Arc<dyn pse_columnar::MemoryPool>,
        cancel: &CancellationToken,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        match self {
            Self::Range { offset, length } => {
                batch.slice(*offset, *length).map_err(super::relation)
            }
            Self::Sparse { offsets, .. } => {
                let scratch = MemoryConsumer::new("canonical:selected-take-offsets").register(pool);
                scratch
                    .try_grow(offsets.len().saturating_mul(size_of::<u32>()))
                    .map_err(pse_engine::EngineError::from)?;
                let indices = datafusion::arrow::array::UInt32Array::from(offsets.clone());
                let batch = batch
                    .take_reserved(&indices, pool, cancel)
                    .map_err(super::relation);
                drop(indices);
                drop(scratch);
                batch
            }
        }
    }
}
pub(super) struct SelectedSnapshotBuilder {
    decode: Arc<ProtectedResultDecode>,
    entries: Vec<SelectedBlock>,
    rows: u64,
    owner: MemoryReservation,
}
impl SelectedSnapshotBuilder {
    pub(super) fn new(decode: Arc<ProtectedResultDecode>) -> Result<Self, WorkflowError> {
        let owner = MemoryConsumer::new("canonical:selected-snapshot-inventory")
            .register(&decode.services.pool());
        owner
            .try_grow(size_of::<SelectedSnapshotInner>() + 2 * size_of::<usize>())
            .map_err(pse_engine::EngineError::from)?;
        Ok(Self {
            decode,
            entries: Vec::new(),
            rows: 0,
            owner,
        })
    }
    pub(super) fn push(
        &mut self,
        block: &Block,
        descriptor: &FileBatch,
        rows: SelectedRows,
    ) -> Result<(), WorkflowError> {
        use pse_model::HeapUsage;
        let extent = 2 * size_of::<SelectedBlock>() + block.heap_bytes() + descriptor.heap_bytes();
        self.owner
            .try_grow(extent)
            .map_err(pse_engine::EngineError::from)?;
        self.entries
            .try_reserve_exact(1)
            .map_err(|_| contract("selected snapshot inventory allocation refused"))?;
        self.rows = self
            .rows
            .checked_add(rows.len() as u64)
            .ok_or_else(|| contract("selected snapshot row count extent"))?;
        self.entries.push(SelectedBlock {
            block: block.clone(),
            descriptor: descriptor.clone(),
            rows,
        });
        Ok(())
    }
    pub(super) fn finish(self) -> SelectedResultSnapshot {
        SelectedResultSnapshot(Arc::new(SelectedSnapshotInner {
            decode: self.decode,
            entries: self.entries,
            rows: self.rows,
            _owner: self.owner,
        }))
    }
}
impl SelectedResultSnapshot {
    /// Full original registry schema; row restrictions are owned by this snapshot.
    pub fn schema(&self) -> SchemaRef {
        self.0.decode.schema.clone()
    }
    /// Exact original terminal run/attempt/manifest, including partiality.
    pub fn selection(&self) -> &ResultRead {
        &self.0.decode.read
    }
    /// Number of rows in the complete frozen selected universe.
    pub fn row_count(&self) -> u64 {
        self.0.rows
    }
    /// Original configured native functions, registry and pool.
    pub fn session_state(&self) -> SessionState {
        self.0
            .decode
            .services
            .session_state()
    }
    /// Native provider restricting rows before any residual, limit or projection.
    pub fn table_provider(&self) -> Arc<dyn TableProvider> {
        Arc::new(SnapshotProvider(FrozenSnapshot::Selected(self.clone())))
    }
}

#[derive(Clone, Debug)]
enum FrozenSnapshot {
    Full(ResultSnapshot),
    Selected(SelectedResultSnapshot),
}
impl FrozenSnapshot {
    fn decode(&self) -> &Arc<ProtectedResultDecode> {
        match self {
            Self::Full(s) => &s.0.decode,
            Self::Selected(s) => &s.0.decode,
        }
    }
    fn schema(&self) -> SchemaRef {
        self.decode().schema.clone()
    }
    fn len(&self) -> usize {
        match self {
            Self::Full(snapshot) => snapshot.0.blocks.len(),
            Self::Selected(snapshot) => snapshot.0.entries.len(),
        }
    }
    fn member(&self, index: usize) -> Option<(&Block, &FileBatch)> {
        match self {
            Self::Full(snapshot) => {
                let block = snapshot.0.blocks.get(index)?;
                Some((block, snapshot.0.batches.get(&block.batch)?))
            }
            Self::Selected(snapshot) => snapshot
                .0
                .entries
                .get(index)
                .map(|entry| (&entry.block, &entry.descriptor)),
        }
    }
    async fn next(
        &self,
        index: &mut usize,
        cache: &mut Option<(usize, pse_relations::columnar::FieldCheckedBatch)>,
        cancel: &CancellationToken,
        pruning: Option<&PrunedSelection>,
    ) -> Result<Option<pse_relations::columnar::FieldCheckedBatch>, WorkflowError> {
        match self {
            Self::Full(snapshot) => {
                while let Some(block) = snapshot.0.blocks.get(*index) {
                    let position = *index;
                    *index += 1;
                    let lower = snapshot.0.start.max(block.start);
                    let upper = snapshot.0.end.min(block.end);
                    if lower >= upper {
                        continue;
                    }
                    if pruning.is_some_and(|selection| !selection.keeps(position)) {
                        let (_, descriptor) = self
                            .member(position)
                            .ok_or_else(|| contract("pruned batch outside frozen inventory"))?;
                        self.decode()
                            .validate_pruned_block(block, descriptor, cancel)
                            .await?;
                        continue;
                    }
                    let batch = snapshot.read_block(block, cancel).await?;
                    let offset = usize::try_from(lower - block.start)
                        .map_err(|_| contract("snapshot row offset extent"))?;
                    let count = usize::try_from(upper - lower)
                        .map_err(|_| contract("snapshot row count extent"))?;
                    return batch
                        .slice(offset, count)
                        .map(Some)
                        .map_err(super::relation);
                }
                Ok(None)
            }
            Self::Selected(snapshot) => loop {
                let Some(entry) = snapshot.0.entries.get(*index) else {
                    *cache = None;
                    return Ok(None);
                };
                if pruning.is_some_and(|selection| !selection.keeps(*index)) {
                    self.decode()
                        .validate_pruned_block(&entry.block, &entry.descriptor, cancel)
                        .await?;
                    *cache = None;
                    *index += 1;
                    continue;
                }
                let cached = cache.as_ref().and_then(|(position, batch)| {
                    snapshot
                        .0
                        .entries
                        .get(*position)
                        .filter(|prior| {
                            prior.block == entry.block && prior.descriptor == entry.descriptor
                        })
                        .map(|_| batch.clone())
                });
                let checked = match cached {
                    Some(batch) => batch,
                    None => {
                        snapshot
                            .0
                            .decode
                            .read_block(&entry.block, &entry.descriptor, cancel)
                            .await?
                    }
                };
                *cache = Some((*index, checked.clone()));
                *index += 1;
                return entry
                    .rows
                    .apply(
                        &checked,
                        &snapshot.0.decode.services.pool(),
                        cancel,
                    )
                    .map(Some);
            },
        }
    }
}

#[derive(Debug)]
pub(super) struct PrunedSelection {
    keep: Vec<bool>,
    _owner: MemoryReservation,
}
impl PrunedSelection {
    pub(super) fn keeps(&self, index: usize) -> bool {
        self.keep.get(index).copied().unwrap_or(true)
    }
}

fn prepare_pruning(
    snapshot: &FrozenSnapshot,
    state: &SessionState,
    filters: &[Expr],
) -> NativeResult<Option<Arc<PrunedSelection>>> {
    prepare_pruning_members(
        &snapshot.decode().services,
        snapshot.decode().relation,
        snapshot.schema(),
        state,
        filters,
        snapshot.len(),
        |index| snapshot.member(index),
    )
}
/// One selection-local native plan, bound to its actual schema and session.
/// The same physical expression supplies both conservative pruning and residuals.
#[derive(Debug)]
pub(super) struct NativeResultSelection {
    relation: pse_ids::SemanticId,
    schema: SchemaRef,
    physical: Arc<dyn datafusion::physical_expr::PhysicalExpr>,
    pruning: Option<datafusion_pruning::PruningPredicate>,
    _owner: MemoryReservation,
}
impl NativeResultSelection {
    pub(super) fn new(
        services: &RetainedDataServices,
        relation: pse_ids::SemanticId,
        schema: SchemaRef,
        state: &SessionState,
        expression: Expr,
    ) -> NativeResult<Arc<Self>> {
        use datafusion::common::DFSchema;
        let owner = MemoryConsumer::new("canonical:result-selection-plan")
            .register(&services.pool());
        // The registry predicate has bounded shape; retain the native expression,
        // pruning rewrite and schema handles for the full selection lifetime.
        owner.try_grow(
            pse_columnar::schema_working_extent(schema.as_ref())
                .map_err(pse_columnar::external)?
                .checked_add(64 * 1024)
                .ok_or_else(|| {
                    DataFusionError::ResourcesExhausted("selection plan extent".into())
                })?,
        )?;
        let native = DFSchema::try_from(schema.as_ref().clone())?;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("canonical.result.physical_predicate_plans", "count", 1);
        }
        let physical = state.create_physical_expr(expression, &native)?;
        let pruning = datafusion_pruning::PruningPredicateBuilder::new()
            .with_file_schema(schema.clone())
            .try_build(physical.clone())
            .ok()
            .filter(|predicate| !predicate.always_true());
        Ok(Arc::new(Self {
            relation,
            schema,
            physical,
            pruning,
            _owner: owner,
        }))
    }

    pub(super) fn mask(
        &self,
        batch: &datafusion::arrow::record_batch::RecordBatch,
    ) -> NativeResult<datafusion::arrow::array::BooleanArray> {
        if batch.schema().as_ref() != self.schema.as_ref() {
            return Err(DataFusionError::Execution(
                "selection native schema differs".into(),
            ));
        }
        let mask = self
            .physical
            .evaluate(batch)?
            .into_array(batch.num_rows())?;
        mask.as_any()
            .downcast_ref::<datafusion::arrow::array::BooleanArray>()
            .cloned()
            .ok_or_else(|| DataFusionError::Execution("scientific predicate is not Boolean".into()))
    }

    pub(super) fn prune_members<'a>(
        &self,
        services: &RetainedDataServices,
        count: usize,
        member: impl Fn(usize) -> Option<(&'a Block, &'a FileBatch)>,
    ) -> NativeResult<Option<Arc<PrunedSelection>>> {
        use datafusion::common::{Statistics, pruning::PrunableStatistics, stats::Precision};
        let Some(predicate) = &self.pruning else {
            return Ok(None);
        };
        if count == 0
            || !(0..count)
                .any(|index| member(index).is_some_and(|(block, _)| block.statistics.is_some()))
        {
            return Ok(None);
        }
        let extent = super::result_selection::pruning_working_extent(
            self.schema.fields().len(),
            count,
            (0..count).map(|index| {
                member(index)
                    .and_then(|(block, _)| block.statistics.as_ref())
                    .map_or(0, |bytes| bytes.len())
            }),
        )
        .ok_or_else(|| {
            DataFusionError::ResourcesExhausted("result pruning metadata extent".into())
        })?;
        let owner = MemoryConsumer::new("canonical:result-pruning-page")
            .register(&services.pool());
        owner.try_grow(extent)?;
        let mut statistics = Vec::with_capacity(count);
        for index in 0..count {
            let exact = member(index).and_then(|(block, descriptor)| {
                let bytes = block.statistics.as_ref()?;
                let binding = pse_relations::result_batch::StatisticsBinding {
                    payload: pse_operations::canonical_execution::result_payload_locator(
                        descriptor,
                    )
                    .ok()?
                    .payload_identity()
                    .to_owned(),
                    digest: descriptor.digest.clone(),
                    index: descriptor.payload_index,
                    interpretation: block.interpretation.clone(),
                };
                let facts = pse_relations::result_batch::recorded_statistics(
                    bytes.as_slice(),
                    services.registry(),
                    self.relation,
                    self.schema.as_ref(),
                    &binding,
                )?;
                (facts.num_rows == Precision::Exact(usize::try_from(block.rows).ok()?))
                    .then_some(facts)
            });
            statistics.push(Arc::new(
                exact.unwrap_or_else(|| Statistics::new_unknown(self.schema.as_ref())),
            ));
        }
        let statistics = PrunableStatistics::new(statistics, self.schema.clone());
        let keep = match predicate.prune(&statistics) {
            Ok(keep) if keep.len() == count => keep,
            _ => return Ok(None),
        };
        drop(statistics);
        let retained = keep
            .capacity()
            .checked_add(size_of::<PrunedSelection>())
            .ok_or_else(|| {
                DataFusionError::ResourcesExhausted("result pruning mask extent".into())
            })?;
        if retained > owner.size() {
            owner.try_grow(retained - owner.size())?;
        }
        owner.shrink(owner.size() - retained);
        Ok(Some(Arc::new(PrunedSelection {
            keep,
            _owner: owner,
        })))
    }

    #[cfg(all(test, feature = "canonical-tests"))]
    pub(super) fn block_may_match(
        &self,
        services: &RetainedDataServices,
        block: &Block,
        descriptor: &FileBatch,
    ) -> NativeResult<bool> {
        Ok(self
            .prune_members(services, 1, |_| Some((block, descriptor)))?
            .is_none_or(|selection| selection.keeps(0)))
    }
}

fn prepare_pruning_members<'a>(
    services: &RetainedDataServices,
    relation: pse_ids::SemanticId,
    schema: SchemaRef,
    state: &SessionState,
    filters: &[Expr],
    count: usize,
    member: impl Fn(usize) -> Option<(&'a Block, &'a FileBatch)>,
) -> NativeResult<Option<Arc<PrunedSelection>>> {
    let Some(expression) = filters
        .iter()
        .cloned()
        .reduce(|left, right| left.and(right))
    else {
        return Ok(None);
    };
    let selection = NativeResultSelection::new(services, relation, schema, state, expression)?;
    selection.prune_members(services, count, member)
}

#[derive(Debug)]
struct SnapshotProvider(FrozenSnapshot);
#[async_trait::async_trait]
impl TableProvider for SnapshotProvider {
    fn schema(&self) -> SchemaRef {
        self.0.schema()
    }
    fn table_type(&self) -> TableType {
        TableType::Temporary
    }
    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> NativeResult<Vec<TableProviderFilterPushDown>> {
        Ok(vec![TableProviderFilterPushDown::Inexact; filters.len()])
    }
    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> NativeResult<Arc<dyn ExecutionPlan>> {
        let state = state
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| {
                DataFusionError::Plan("snapshot requires configured native session state".into())
            })?;
        if !Arc::ptr_eq(
            &state.runtime_env().memory_pool,
            &self.0.decode().services.pool(),
        ) {
            return Err(DataFusionError::Plan(
                "snapshot query uses another deployment pool".into(),
            ));
        }
        let pruning = prepare_pruning(&self.0, state, filters)?;
        StreamingTable::try_new(
            self.schema(),
            vec![Arc::new(SnapshotPartition { snapshot: self.0.clone(), pruning })],
        )?
        // Inexact filters remain native residuals above this provider. An early
        // limit could count false positives before the residual rejects them.
        .scan(state, projection, &[], if filters.is_empty() { limit } else { None })
        .await
    }
}

#[derive(Debug)]
struct SnapshotPartition {
    snapshot: FrozenSnapshot,
    pruning: Option<Arc<PrunedSelection>>,
}
impl PartitionStream for SnapshotPartition {
    fn schema(&self) -> &SchemaRef {
        &self.snapshot.decode().schema
    }
    fn execute(&self, context: Arc<TaskContext>) -> SendableRecordBatchStream {
        let pool_matches = Arc::ptr_eq(
            &context.runtime_env().memory_pool,
            &self.snapshot.decode().services.pool(),
        );
        let snapshot = self.snapshot.clone();
        let pruning = self.pruning.clone();
        let cancel = CancellationToken::new();
        let stream = futures_util::stream::try_unfold(
            (snapshot, 0usize, None, cancel, pool_matches, pruning),
            |(snapshot, mut index, mut cache, cancel, pool_matches, pruning)| async move {
                if !pool_matches {
                    return Err(DataFusionError::Execution(
                        "snapshot execution uses another deployment pool".into(),
                    ));
                }
                let batch = snapshot
                    .next(&mut index, &mut cache, &cancel, pruning.as_deref())
                    .await
                    .map_err(pse_columnar::external)?;
                let Some(batch) = batch else {
                    return Ok(None);
                };
                let batch = batch
                    .checked_export(&cancel)
                    .map_err(pse_columnar::external)?;
                Ok::<_, DataFusionError>(Some((
                    batch,
                    (snapshot, index, cache, cancel, pool_matches, pruning),
                )))
            },
        );
        Box::pin(RecordBatchStreamAdapter::new(
            self.snapshot.schema(),
            stream,
        ))
    }
}

#[cfg(test)]
mod selected_snapshot_unit {
    use super::*;
    use pse_columnar::{GreedyMemoryPool, MemoryPool};
    #[test]
    fn bitmap_selection_handles_empty_ranges_sparse_offsets_and_u32_overflow() {
        use datafusion::arrow::buffer::BooleanBuffer;
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(4096));
        let make = |values: Vec<bool>, base| {
            let owner = MemoryConsumer::new("fixture:selected-mask").register(&pool);
            owner.try_grow(1024).unwrap();
            SelectedRows::from_mask(BooleanBuffer::from(values), base, owner)
        };
        assert!(make(vec![false; 4], usize::MAX).unwrap().is_none());
        assert!(matches!(
            make(vec![true; 4], 6).unwrap(),
            Some(SelectedRows::Range {
                offset: 6,
                length: 4
            })
        ));
        assert!(matches!(
            make(vec![false, true, true, false], 10).unwrap(),
            Some(SelectedRows::Range {
                offset: 11,
                length: 2
            })
        ));
        let sparse = make(vec![true, false, true], 7).unwrap().unwrap();
        assert!(matches!(&sparse, SelectedRows::Sparse { offsets, .. } if offsets == &[7, 9]));
        drop(sparse);
        assert_eq!(pool.reserved(), 0);
        assert!(make(vec![true, true], u32::MAX as usize).is_err());
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn selected_offsets_require_admission_coalesce_dense_ranges_and_release_sparse_storage() {
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(4096));
        let grant = || MemoryConsumer::new("test:selected-offsets").register(&pool);
        let owner = grant();
        owner.try_grow(12).unwrap();
        let dense = SelectedRows::from_offsets(vec![2, 3, 4], owner).unwrap();
        assert!(matches!(
            dense,
            SelectedRows::Range {
                offset: 2,
                length: 3
            }
        ));
        assert_eq!(pool.reserved(), 0);
        let owner = grant();
        owner.try_grow(12).unwrap();
        let sparse = SelectedRows::from_offsets(vec![0, 2, u32::MAX], owner).unwrap();
        assert_eq!(sparse.len(), 3);
        assert_eq!(pool.reserved(), 12);
        drop(sparse);
        assert_eq!(pool.reserved(), 0);
        for offsets in [vec![2, 2], vec![2, 1]] {
            let owner = grant();
            owner.try_grow(8).unwrap();
            assert!(SelectedRows::from_offsets(offsets, owner).is_err());
            assert_eq!(pool.reserved(), 0);
        }
        assert!(SelectedRows::from_offsets(vec![1, 2], grant()).is_err());
        assert_eq!(pool.reserved(), 0);
    }
}

#[cfg(test)]
mod decoder_lane_inquiry {
    use super::*;
    use datafusion::arrow::{
        array::Float64Array,
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use pse_columnar::{GreedyMemoryPool, MemoryPool};
    use std::sync::Arc;

    #[tokio::test]
    async fn decoder_wait_cancellation_owns_no_working_grant() {
        let cache = Arc::new(tokio::sync::Mutex::new(None));
        let first = cache.lock().await;
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(1024));
        let cancel = CancellationToken::new();
        let waiting = {
            let cache = cache.clone();
            let pool = pool.clone();
            let cancel = cancel.clone();
            tokio::spawn(async move {
                let _turn = decoder_turn(&cache, &cancel).await?;
                let owner = MemoryConsumer::new("test:decoder-working").register(&pool);
                owner
                    .try_grow(1024)
                    .map_err(pse_engine::EngineError::from)?;
                Ok::<_, WorkflowError>(())
            })
        };
        tokio::task::yield_now().await;
        assert_eq!(pool.reserved(), 0);
        cancel.cancel();
        assert!(
            tokio::time::timeout(std::time::Duration::from_secs(2), waiting)
                .await
                .unwrap()
                .unwrap()
                .is_err()
        );
        assert_eq!(pool.reserved(), 0);
        drop(first);
    }

    #[test]
    fn admitted_inventory_reuses_one_footer_for_random_access_and_keeps_escaped_storage() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Float64,
            false,
        )]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Float64Array::from(vec![1., 2., 3.]))],
        )
        .unwrap();
        let bytes = result_files::encode_file_batches(&[batch.clone(), batch]).unwrap();
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(128 << 10));
        let cancel = CancellationToken::new();
        let mut reader = result_files::open_file_reader(
            std::io::Cursor::new(&bytes),
            &schema,
            Some(2),
            pool.clone(),
            &cancel,
        )
        .unwrap();
        let second = reader.read_batch(1, &cancel).unwrap();
        let first = reader.read_batch(0, &cancel).unwrap();
        assert_eq!(first.batch.batch(), second.batch.batch());
        let escaped = first.column(0).slice(1, 1);
        drop(first);
        drop(second);
        drop(reader);
        drop(bytes);
        assert!(pool.reserved() > 0);
        drop(escaped);
        assert_eq!(pool.reserved(), 0);
    }
}

#[cfg(all(test, feature = "native-study-tests"))]
#[path = "result_pruning_tests.rs"]
mod result_pruning_tests;
