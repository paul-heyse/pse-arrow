// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact immutable result inventories shared by typed readers and native queries.

use super::{SourceServices, WorkflowError, contract, result_files};
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
use pse_columnar::{
    CancellationToken, MemoryConsumer, MemoryReservation, owned_buffer::AllocationScope,
};
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
    pub(super) services: SourceServices,
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
impl ProtectedResultDecode {
    pub(super) fn new(
        services: &SourceServices,
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
        services: &SourceServices,
        relation: pse_ids::SemanticId,
        schema: SchemaRef,
    ) -> Result<Arc<Self>, WorkflowError> {
        let owner = MemoryConsumer::new("canonical:protected-result-decoder")
            .register(&services.preparation().pool());
        owner
            .try_grow(size_of::<Self>() + 2 * size_of::<usize>())
            .map_err(pse_engine::EngineError::from)?;
        Ok(Arc::new(Self {
            services: services.clone(),
            relation,
            schema,
            admission: services.preparation().admission_context(),
            cache: tokio::sync::Mutex::new(None),
            _owner: owner,
        }))
    }
}

#[derive(Debug)]
struct CachedFile {
    instance: String,
    reader: datafusion::arrow::ipc::reader::FileReader<result_files::ResultFileSource>,
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
        services: &SourceServices,
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
            .register(&services.preparation().pool());
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
                    &services.preparation().pool(),
                    "canonical:result-snapshot-joined-page",
                    MetadataResponse::JoinedBlockPage,
                )?;
                if let Some(context) = pse_diagnostics::observation::current() {
                    context.count("canonical.result.joined_page_requests", "count", 1);
                }
                let joined = pse_operations::canonical::response_ownership_scope(
                    Arc::new((read.clone(), read_owner.clone(), response.clone())),
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
            .preparation()
            .bound_model()
            .native_state()
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
        let owner = MemoryConsumer::new("canonical:result-file-decode")
            .register(&self.services.preparation().pool());
        // Prepay physical integrity reads, footer verification, native body and
        // alignment copies before opening the file or invoking Arrow.
        let bytes =
            usize::try_from(descriptor.payload_bytes).map_err(|_| contract("file byte extent"))?;
        let schema_bytes = result_files::schema_extent(&self.schema)?;
        owner
            .try_grow(
                bytes
                    .saturating_mul(4)
                    .saturating_add(schema_bytes.saturating_mul(4))
                    .saturating_add(64 * 1024),
            )
            .map_err(pse_engine::EngineError::from)?;
        if cache
            .as_ref()
            .is_none_or(|cached| cached.instance != locator.payload_identity())
        {
            let metadata = MemoryConsumer::new("canonical:result-file-footer-cache")
                .register(&self.services.preparation().pool());
            metadata
                .try_grow(
                    bytes
                        .saturating_mul(2)
                        .saturating_add(schema_bytes.saturating_mul(4))
                        .saturating_add(64 * 1024),
                )
                .map_err(pse_engine::EngineError::from)?;
            // Both sequential catalog RPCs inside open_result_file must retain
            // the exact read ticket and a separately prepaid metadata response.
            // The file-shaped decode/footer grants cannot bound SDK metadata,
            // and connection activity alone does not retain this ResultRead.
            let response = super::connected_results::pooled_response_owner(
                &self.services.preparation().pool(),
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
            let (source, count) = match &locator {
                pse_operations::canonical_execution::PayloadLocator::Sealed { .. } => {
                    let file = pse_operations::canonical::response_ownership_scope(
                        Arc::new((authority.clone(), response)),
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
                    (
                        result_files::ResultFileSource::Sealed(Box::new(file)),
                        count,
                    )
                }
                pse_operations::canonical_execution::PayloadLocator::Inline { .. } => {
                    let body = pse_operations::canonical::response_ownership_scope(
                        Arc::new((authority.clone(), response)),
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
                    (
                        result_files::ResultFileSource::Inline(std::io::Cursor::new(
                            body.body.into_vec(),
                        )),
                        count,
                    )
                }
            };
            let reader = result_files::open_file_reader(source, &self.schema, Some(count))?;
            *cache = Some(CachedFile {
                instance: locator.payload_identity().to_owned(),
                reader,
                _owner: metadata,
            });
        }
        let cached = cache
            .as_mut()
            .ok_or_else(|| contract("file cache absent"))?;
        cached
            .reader
            .set_index(
                usize::try_from(descriptor.payload_index)
                    .map_err(|_| contract("file batch index extent"))?,
            )
            .map_err(super::result_blocks::ResultBlockError::from)?;
        if !decode {
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("canonical.result.pruned_batches", "count", 1);
            }
            return Ok(None);
        }
        let batch = cached
            .reader
            .next()
            .transpose()
            .map_err(super::result_blocks::ResultBlockError::from)?
            .ok_or_else(|| contract("frozen file batch absent"))?;
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
        let scope = AllocationScope::default();
        let owned = scope
            .attach_borrowed_reserved(batch, &owner)
            .map_err(pse_engine::EngineError::from)?;
        let spec = self
            .services
            .preparation()
            .registry()
            .relation_by_id(self.relation)
            .ok_or_else(|| contract("snapshot relation unavailable"))?;
        let checked = pse_relations::columnar::FieldCheckedBatch::admit_owned(
            self.services.preparation().registry(),
            spec,
            owned,
            &self.admission,
            cancel,
        )
        .map_err(pse_engine::EngineError::from)?;
        pse_relations::result_batch::validate_scalar_keys(
            self.services.preparation().registry(),
            &checked,
            &self.services.preparation().pool(),
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
            .register(&decode.services.preparation().pool());
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
            .preparation()
            .bound_model()
            .native_state()
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
                        &snapshot.0.decode.services.preparation().pool(),
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
        services: &SourceServices,
        relation: pse_ids::SemanticId,
        schema: SchemaRef,
        state: &SessionState,
        expression: Expr,
    ) -> NativeResult<Arc<Self>> {
        use datafusion::common::DFSchema;
        let owner = MemoryConsumer::new("canonical:result-selection-plan")
            .register(&services.preparation().pool());
        // The registry predicate has bounded shape; retain the native expression,
        // pruning rewrite and schema handles for the full selection lifetime.
        owner.try_grow(64 * 1024 + schema.fields().len() * 512)?;
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
        services: &SourceServices,
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
        let extent = self
            .schema
            .fields()
            .len()
            .checked_mul(512)
            .and_then(|bytes| bytes.checked_add(2048))
            .and_then(|bytes| count.checked_mul(bytes))
            .and_then(|base| {
                (0..count).try_fold(base, |bytes, index| {
                    let encoded = member(index)
                        .and_then(|(block, _)| block.statistics.as_ref())
                        .map_or(0, |bytes| bytes.len());
                    bytes.checked_add(encoded.checked_mul(16)?)
                })
            })
            .ok_or_else(|| {
                DataFusionError::ResourcesExhausted("result pruning metadata extent".into())
            })?;
        let owner = MemoryConsumer::new("canonical:result-pruning-page")
            .register(&services.preparation().pool());
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
                    services.preparation().registry(),
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
        services: &SourceServices,
        block: &Block,
        descriptor: &FileBatch,
    ) -> NativeResult<bool> {
        Ok(self
            .prune_members(services, 1, |_| Some((block, descriptor)))?
            .is_none_or(|selection| selection.keeps(0)))
    }
}

fn prepare_pruning_members<'a>(
    services: &SourceServices,
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
            &self.0.decode().services.preparation().pool(),
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
            &self.snapshot.decode().services.preparation().pool(),
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
    use std::{
        fs::File,
        io::Read,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
        time::Instant,
    };

    struct Fixture {
        path: PathBuf,
        schema: SchemaRef,
        digest: pse_ids::ContentHash,
        bytes: usize,
        rows: usize,
    }
    #[derive(Default)]
    struct Counts {
        opens: AtomicUsize,
        footers: AtomicUsize,
        decodes: AtomicUsize,
        active: AtomicUsize,
        simultaneous: AtomicUsize,
        charge: AtomicUsize,
    }
    fn read_file(
        file: &Fixture,
        batches: &[usize],
        selected_rows: bool,
        pool: &Arc<dyn MemoryPool>,
        counts: &Counts,
        rendezvous: Option<&std::sync::Barrier>,
    ) -> (usize, f64, pse_columnar::owned_buffer::OwnedRecordBatch) {
        // Same conservative body/footer/decoded overlap envelope as production;
        // this primitive control has no canonical permission or storage effects.
        let owner = MemoryConsumer::new("test:decoder-lane-envelope").register(pool);
        owner.try_grow(file.bytes * 6 + 256 * 1024).unwrap();
        counts.charge.fetch_max(pool.reserved(), Ordering::SeqCst);
        let mut reader = File::open(&file.path).unwrap();
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).unwrap();
        assert_eq!(
            pse_ids::encoding_checksum(&bytes).content_hash(),
            file.digest
        );
        drop(bytes);
        counts.opens.fetch_add(1, Ordering::SeqCst);
        let mut reader = result_files::open_file_reader(reader, &file.schema, Some(4)).unwrap();
        counts.footers.fetch_add(1, Ordering::SeqCst);
        let active = counts.active.fetch_add(1, Ordering::SeqCst) + 1;
        counts.simultaneous.fetch_max(active, Ordering::SeqCst);
        if let Some(barrier) = rendezvous {
            barrier.wait();
        }
        let mut rows = 0;
        let mut sum = 0.;
        let mut escaped = None;
        for &index in batches {
            reader.set_index(index).unwrap();
            let batch = reader.next().unwrap().unwrap();
            counts.decodes.fetch_add(1, Ordering::SeqCst);
            assert_eq!(batch.num_rows(), file.rows);
            assert_eq!(batch.schema(), file.schema);
            let owned = AllocationScope::default()
                .attach_borrowed_reserved(batch, &owner)
                .unwrap();
            let selected = if selected_rows {
                owned.slice(0, 1).unwrap()
            } else {
                owned
            };
            let values = selected
                .batch()
                .column(0)
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap();
            rows += values.len();
            sum += values.values().iter().sum::<f64>();
            escaped = Some(selected);
        }
        counts.active.fetch_sub(1, Ordering::SeqCst);
        (rows, sum, escaped.unwrap())
    }

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
    fn decoder_file_affinity_and_independent_lane_inquiry() {
        // 1/4/16 files, sparse/full batch and row selection, and a 4x final-file
        // skew. Timings include integrity/open/footer/decode/selection/aliases.
        for file_count in [1, 4, 16] {
            for base_rows in [64, 8192] {
                let directory = tempfile::tempdir().unwrap();
                let schema = Arc::new(Schema::new(vec![Field::new(
                    "value",
                    DataType::Float64,
                    false,
                )]));
                let files = (0..file_count)
                    .map(|file| {
                        let rows = base_rows * if file + 1 == file_count { 4 } else { 1 };
                        let batch = RecordBatch::try_new(
                            schema.clone(),
                            vec![Arc::new(Float64Array::from_iter_values(
                                (0..rows).map(|row| (file * 100000 + row) as f64),
                            ))],
                        )
                        .unwrap();
                        let bytes = result_files::encode_file_batches(&vec![batch; 4]).unwrap();
                        let path = directory.path().join(format!("{file}.arrow"));
                        std::fs::write(&path, &bytes).unwrap();
                        Fixture {
                            path,
                            schema: schema.clone(),
                            digest: pse_ids::encoding_checksum(&bytes).content_hash(),
                            bytes: bytes.len(),
                            rows,
                        }
                    })
                    .collect::<Vec<_>>();
                for sparse in [true, false] {
                    let batches: &[usize] = if sparse { &[2] } else { &[0, 1, 2, 3] };
                    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(128 << 20));
                    let serial = Counts::default();
                    let started = Instant::now();
                    let mut expected = (0, 0.);
                    // A single reader cache reuses a lone file, but switches
                    // on every interleaved original batch ordinal across files.
                    let mut serial_aliases = (0..file_count).map(|_| None).collect::<Vec<_>>();
                    if file_count == 1 {
                        let (rows, sum, alias) =
                            read_file(&files[0], batches, sparse, &pool, &serial, None);
                        expected = (rows, sum);
                        serial_aliases[0] = Some(alias);
                    } else {
                        for &batch in batches {
                            for (position, file) in files.iter().enumerate() {
                                let (rows, sum, alias) =
                                    read_file(file, &[batch], sparse, &pool, &serial, None);
                                expected.0 += rows;
                                expected.1 += sum;
                                assert!(alias.retained_bytes().unwrap() > 0);
                                serial_aliases[position] = Some(alias);
                            }
                        }
                    }
                    let serial_ns = started.elapsed().as_nanos();
                    assert!(pool.reserved() > 0);
                    drop(serial_aliases);
                    assert_eq!(pool.reserved(), 0);
                    let affinity = Counts::default();
                    let started = Instant::now();
                    let mut actual = (0, 0.);
                    let mut aliases = Vec::new();
                    for file in &files {
                        let (rows, sum, alias) =
                            read_file(file, batches, sparse, &pool, &affinity, None);
                        actual.0 += rows;
                        actual.1 += sum;
                        aliases.push(alias);
                    }
                    let affinity_ns = started.elapsed().as_nanos();
                    assert_eq!(actual, expected);
                    assert_eq!(affinity.opens.load(Ordering::SeqCst), file_count);
                    assert_eq!(
                        serial.opens.load(Ordering::SeqCst),
                        if file_count == 1 {
                            1
                        } else {
                            file_count * batches.len()
                        }
                    );
                    assert!(
                        pool.reserved() > 0,
                        "escaped Arrow aliases remain charged after readers return"
                    );
                    drop(aliases);
                    assert_eq!(pool.reserved(), 0);
                    let parallel = Counts::default();
                    let lanes = file_count.min(2);
                    let barrier = std::sync::Barrier::new(lanes);
                    let started = Instant::now();
                    let outputs = std::thread::scope(|scope| {
                        (0..lanes)
                            .map(|lane| {
                                let files = &files;
                                let pool = &pool;
                                let parallel = &parallel;
                                let barrier = &barrier;
                                scope.spawn(move || {
                                    let mut answer = (0, 0.);
                                    let mut aliases = Vec::new();
                                    for (position, file) in files
                                        .iter()
                                        .enumerate()
                                        .filter(|(position, _)| position % lanes == lane)
                                    {
                                        let (rows, sum, alias) = read_file(
                                            file,
                                            batches,
                                            sparse,
                                            pool,
                                            parallel,
                                            (position == lane).then_some(barrier),
                                        );
                                        answer.0 += rows;
                                        answer.1 += sum;
                                        aliases.push(alias);
                                    }
                                    (answer, aliases)
                                })
                            })
                            .collect::<Vec<_>>()
                            .into_iter()
                            .map(|task| task.join().unwrap())
                            .collect::<Vec<_>>()
                    });
                    let parallel_ns = started.elapsed().as_nanos();
                    let actual = outputs
                        .iter()
                        .fold((0, 0.), |a, b| (a.0 + b.0.0, a.1 + b.0.1));
                    assert_eq!(actual, expected);
                    assert_eq!(parallel.simultaneous.load(Ordering::SeqCst), lanes);
                    assert_eq!(parallel.opens.load(Ordering::SeqCst), file_count);
                    assert_eq!(
                        parallel.decodes.load(Ordering::SeqCst),
                        file_count * batches.len()
                    );
                    assert!(pool.reserved() > 0);
                    drop(outputs);
                    assert_eq!(pool.reserved(), 0);
                    eprintln!(
                        "decoder-inquiry files={file_count} base_rows={base_rows} sparse={sparse} serial_ns={serial_ns} affinity_ns={affinity_ns} lanes_ns={parallel_ns} serial_opens={} affinity_opens={} lane_opens={} decodes={} simultaneous={} lane_charge={} serial_charge={} affinity_charge={} footer_reads={}",
                        serial.opens.load(Ordering::SeqCst),
                        affinity.opens.load(Ordering::SeqCst),
                        parallel.opens.load(Ordering::SeqCst),
                        parallel.decodes.load(Ordering::SeqCst),
                        parallel.simultaneous.load(Ordering::SeqCst),
                        parallel.charge.load(Ordering::SeqCst),
                        serial.charge.load(Ordering::SeqCst),
                        affinity.charge.load(Ordering::SeqCst),
                        parallel.footers.load(Ordering::SeqCst)
                    );
                }
                let mut corrupt = std::fs::read(&files[0].path).unwrap();
                corrupt[0] ^= 1;
                assert!(
                    result_files::open_file_reader(std::io::Cursor::new(corrupt), &schema, Some(4))
                        .is_err()
                );
                let other = Arc::new(Schema::new(vec![Field::new(
                    "other",
                    DataType::Float64,
                    false,
                )]));
                assert!(
                    result_files::open_file_reader(
                        File::open(&files[0].path).unwrap(),
                        &other,
                        Some(4)
                    )
                    .is_err()
                );
            }
        }
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
#[path = "result_pruning_tests.rs"]
mod result_pruning_tests;
