// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact immutable result inventories shared by typed readers and native queries.

use super::{Runtime, WorkflowError, contract, result_files};
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
    pub(super) runtime: Runtime,
    pub(super) read: Arc<ResultRead>,
    relation: pse_ids::SemanticId,
    pub(super) schema: SchemaRef,
    admission: Arc<pse_relations::validate::ValidationContext>,
    cache: tokio::sync::Mutex<Option<CachedFile>>,
    pub(super) read_owner: Arc<MemoryReservation>,
    _owner: MemoryReservation,
}
impl ProtectedResultDecode {
    pub(super) fn new(
        runtime: &Runtime,
        read: Arc<ResultRead>,
        read_owner: Arc<MemoryReservation>,
        relation: pse_ids::SemanticId,
        schema: SchemaRef,
    ) -> Result<Arc<Self>, WorkflowError> {
        let required = read.retained_bytes()?
            .checked_add(MetadataResponse::Protection.allocation_bytes())
            .ok_or_else(|| contract("result protected metadata extent"))?;
        if read_owner.size() < required {
            return Err(contract("result read metadata is not prepaid"));
        }
        read.retain_metadata_owner(read_owner.clone())?;
        let owner = MemoryConsumer::new("canonical:protected-result-decoder").register(&runtime.shared.pool());
        owner.try_grow(size_of::<Self>() + 2 * size_of::<usize>()).map_err(pse_engine::EngineError::from)?;
        Ok(Arc::new(Self { runtime: runtime.clone(), read, read_owner, relation, schema,
            admission: runtime.admission_context(), cache: tokio::sync::Mutex::new(None), _owner: owner }))
    }
}

#[derive(Debug)]
struct CachedFile {
    instance: String,
    reader:
        datafusion::arrow::ipc::reader::FileReader<pse_operations::canonical_files::ProtectedFile>,
    _owner: MemoryReservation,
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
        runtime: &Runtime,
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
            .register(&runtime.shared.pool());
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
                let response = super::connected_results::response_owner(
                    runtime,
                    "canonical:result-snapshot-page",
                    MetadataResponse::BlockPage,
                )?;
                if let Some(context) = pse_diagnostics::observation::current() { context.count("canonical.result.block_page_requests", "count", 1); }
                let page = pse_operations::canonical::response_ownership_scope(
                    Arc::new((read.clone(), read_owner.clone(), response.clone())),
                    runtime.canonical_store()?.result_block_page(
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
                // The first page is retained while the descriptor RPC runs. Its
                // own temporary allowance retires independently of the next RPC.
                let page_bytes = page
                    .capacity()
                    .checked_mul(size_of::<Block>())
                    .and_then(|bytes| {
                        page.iter()
                            .try_fold(bytes, |n, row| n.checked_add(row.heap_bytes()))
                    })
                    .ok_or_else(|| contract("snapshot page extent"))?;
                let page_owner = super::connected_results::retain_metadata(
                    &runtime.shared.pool(),
                    response,
                    page_bytes,
                )?;
                let ordinals = page.iter().map(|block| block.ordinal).collect::<Vec<_>>();
                let response = super::connected_results::response_owner(
                    runtime,
                    "canonical:result-snapshot-descriptors",
                    MetadataResponse::Page,
                )?;
                if let Some(context) = pse_diagnostics::observation::current() { context.count("canonical.result.descriptor_page_requests", "count", 1); }
                let descriptors = pse_operations::canonical::response_ownership_scope(
                    Arc::new((read.clone(), read_owner.clone(), response.clone())),
                    runtime
                        .canonical_store()?
                        .result_file_batch_descriptors(&read, set, &ordinals),
                )
                .await?;
                if page.len() != descriptors.len() {
                    return Err(contract("snapshot descriptor coverage differs"));
                }
                let finished = page.len() < 64;
                after = page.last().map(|block| block.ordinal);
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
                drop(page_owner);
                drop(response);
                if finished {
                    break;
                }
            }
        }
        Ok(Self(Arc::new(SnapshotInner {
            decode: ProtectedResultDecode::new(runtime, read, read_owner, relation, schema)?,
            batches, blocks, positions, start, end, _owner: owner,
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
        self.0.decode.runtime.bound_model().native_state()
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

impl ProtectedResultDecode {
    pub(super) async fn read_block(
        &self,
        block: &Block,
        descriptor: &FileBatch,
        cancel: &CancellationToken,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        if descriptor.key != block.batch || descriptor.ordinal != block.ordinal
            || descriptor.row_count != block.rows || descriptor.digest != block.payload_digest
            || descriptor.payload_bytes != block.payload_bytes {
            return Err(contract("protected result descriptor coverage differs"));
        }
        let owner = MemoryConsumer::new("canonical:result-file-decode")
            .register(&self.runtime.shared.pool());
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
        let mut cache = self.cache.lock().await;
        if cache
            .as_ref()
            .is_none_or(|cached| cached.instance != descriptor.file_instance)
        {
            let metadata = MemoryConsumer::new("canonical:result-file-footer-cache")
                .register(&self.runtime.shared.pool());
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
            let response = super::connected_results::response_owner(
                &self.runtime,
                "canonical:result-file-open-response",
                MetadataResponse::Header,
            )?;
            let file = pse_operations::canonical::response_ownership_scope(
                Arc::new((self.read.clone(), self.read_owner.clone(), response)),
                self
                    .runtime
                    .canonical_store()?
                    .open_result_file(&self.read, descriptor),
            )
            .await?;
            let count = usize::try_from(file.descriptor().batch_count)
                .map_err(|_| contract("file batch count extent"))?;
            let reader = result_files::open_file_reader(file, &self.schema, Some(count))?;
            *cache = Some(CachedFile {
                instance: descriptor.file_instance.clone(),
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
                usize::try_from(descriptor.file_batch_index)
                    .map_err(|_| contract("file batch index extent"))?,
            )
            .map_err(super::result_blocks::ResultBlockError::from)?;
        let batch = cached
            .reader
            .next()
            .transpose()
            .map_err(super::result_blocks::ResultBlockError::from)?
            .ok_or_else(|| contract("frozen file batch absent"))?;
        drop(cache);
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("canonical.result.native_decodes", "count", 1);
            context.count("canonical.result.decoded_array_bytes", "bytes", batch.get_array_memory_size() as u64);
        }
        if batch.num_rows() as u64 != block.rows || batch.schema() != self.schema.clone() {
            return Err(contract("frozen file batch rows/schema differ"));
        }
        let scope = AllocationScope::default();
        let owned = scope
            .attach_borrowed_reserved(batch, &owner)
            .map_err(pse_engine::EngineError::from)?;
        let spec = self
            .runtime
            .registry()
            .relation_by_id(self.relation)
            .ok_or_else(|| contract("snapshot relation unavailable"))?;
        pse_relations::columnar::FieldCheckedBatch::admit_owned(
            self.runtime.registry(),
            spec,
            owned,
            &self.admission,
            cancel,
        )
        .map_err(pse_engine::EngineError::from)
        .map_err(Into::into)
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
    Range { offset: usize, length: usize },
    Sparse { offsets: Vec<u32>, _owner: MemoryReservation },
}
impl SelectedRows {
    pub(super) fn from_offsets(offsets: Vec<u32>, owner: MemoryReservation) -> Result<Self, WorkflowError> {
        let bytes = offsets.capacity().checked_mul(size_of::<u32>())
            .ok_or_else(|| contract("selected offset extent"))?;
        if bytes > owner.size() { return Err(contract("selected offsets are not prepaid")); }
        if offsets.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(contract("selected offsets repeat or reverse source rows"));
        }
        if offsets.windows(2).all(|pair| pair[0].checked_add(1) == Some(pair[1])) {
            let offset = offsets.first().copied().unwrap_or(0) as usize;
            let length = offsets.len();
            drop(offsets);
            drop(owner);
            Ok(Self::Range { offset, length })
        } else {
            owner.shrink(owner.size() - bytes);
            Ok(Self::Sparse { offsets, _owner: owner })
        }
    }
    fn len(&self) -> usize {
        match self { Self::Range {length, ..} => *length, Self::Sparse {offsets, ..} => offsets.len() }
    }
    pub(super) fn apply(&self, batch: &pse_relations::columnar::FieldCheckedBatch,
        pool: &Arc<dyn pse_columnar::MemoryPool>, cancel: &CancellationToken)
        -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        match self {
            Self::Range {offset, length} => batch.slice(*offset, *length).map_err(super::relation),
            Self::Sparse {offsets, ..} => {
                let scratch = MemoryConsumer::new("canonical:selected-take-offsets").register(pool);
                scratch.try_grow(offsets.len().saturating_mul(size_of::<u32>()))
                    .map_err(pse_engine::EngineError::from)?;
                let indices = datafusion::arrow::array::UInt32Array::from(offsets.clone());
                let batch = batch.take_reserved(&indices, pool, cancel).map_err(super::relation);
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
        let owner = MemoryConsumer::new("canonical:selected-snapshot-inventory").register(&decode.runtime.shared.pool());
        owner.try_grow(size_of::<SelectedSnapshotInner>() + 2 * size_of::<usize>())
            .map_err(pse_engine::EngineError::from)?;
        Ok(Self {decode, entries: Vec::new(), rows: 0, owner})
    }
    pub(super) fn push(&mut self, block: &Block, descriptor: &FileBatch, rows: SelectedRows) -> Result<(), WorkflowError> {
        use pse_model::HeapUsage;
        let extent = 2 * size_of::<SelectedBlock>() + block.heap_bytes() + descriptor.heap_bytes();
        self.owner.try_grow(extent).map_err(pse_engine::EngineError::from)?;
        self.entries.try_reserve_exact(1).map_err(|_| contract("selected snapshot inventory allocation refused"))?;
        self.rows = self.rows.checked_add(rows.len() as u64).ok_or_else(|| contract("selected snapshot row count extent"))?;
        self.entries.push(SelectedBlock {block: block.clone(), descriptor: descriptor.clone(), rows});
        Ok(())
    }
    pub(super) fn finish(self) -> SelectedResultSnapshot {
        SelectedResultSnapshot(Arc::new(SelectedSnapshotInner {decode: self.decode, entries: self.entries,
            rows: self.rows, _owner: self.owner}))
    }
}
impl SelectedResultSnapshot {
    /// Full original registry schema; row restrictions are owned by this snapshot.
    pub fn schema(&self) -> SchemaRef { self.0.decode.schema.clone() }
    /// Exact original terminal run/attempt/manifest, including partiality.
    pub fn selection(&self) -> &ResultRead { &self.0.decode.read }
    /// Number of rows in the complete frozen selected universe.
    pub fn row_count(&self) -> u64 { self.0.rows }
    /// Original configured native functions, registry and pool.
    pub fn session_state(&self) -> SessionState { self.0.decode.runtime.bound_model().native_state() }
    /// Native provider restricting rows before any residual, limit or projection.
    pub fn table_provider(&self) -> Arc<dyn TableProvider> {
        Arc::new(SnapshotProvider(FrozenSnapshot::Selected(self.clone())))
    }
}

#[derive(Clone, Debug)]
enum FrozenSnapshot { Full(ResultSnapshot), Selected(SelectedResultSnapshot) }
impl FrozenSnapshot {
    fn decode(&self) -> &Arc<ProtectedResultDecode> {
        match self {Self::Full(s) => &s.0.decode, Self::Selected(s) => &s.0.decode}
    }
    fn schema(&self) -> SchemaRef { self.decode().schema.clone() }
    async fn next(&self, index: &mut usize, cache: &mut Option<(usize, pse_relations::columnar::FieldCheckedBatch)>,
        cancel: &CancellationToken) -> Result<Option<pse_relations::columnar::FieldCheckedBatch>, WorkflowError> {
        match self {
            Self::Full(snapshot) => {
                while let Some(block) = snapshot.0.blocks.get(*index) {
                    *index += 1;
                    let lower = snapshot.0.start.max(block.start);
                    let upper = snapshot.0.end.min(block.end);
                    if lower >= upper {continue;}
                    let batch = snapshot.read_block(block, cancel).await?;
                    let offset = usize::try_from(lower - block.start).map_err(|_| contract("snapshot row offset extent"))?;
                    let count = usize::try_from(upper - lower).map_err(|_| contract("snapshot row count extent"))?;
                    return batch.slice(offset, count).map(Some).map_err(super::relation);
                }
                Ok(None)
            }
            Self::Selected(snapshot) => {
                let Some(entry) = snapshot.0.entries.get(*index) else { *cache = None; return Ok(None); };
                let cached = cache.as_ref().and_then(|(position, batch)| snapshot.0.entries.get(*position)
                    .filter(|prior| prior.block == entry.block && prior.descriptor == entry.descriptor)
                    .map(|_| batch.clone()));
                let checked = match cached { Some(batch) => batch,
                    None => snapshot.0.decode.read_block(&entry.block, &entry.descriptor, cancel).await? };
                *cache = Some((*index, checked.clone()));
                *index += 1;
                entry.rows.apply(&checked, &snapshot.0.decode.runtime.shared.pool(), cancel).map(Some)
            }
        }
    }
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
        Ok(vec![
            TableProviderFilterPushDown::Unsupported;
            filters.len()
        ])
    }
    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> NativeResult<Arc<dyn ExecutionPlan>> {
        if !filters.is_empty() {
            return Err(DataFusionError::Plan(
                "snapshot predicates require native residual evaluation".into(),
            ));
        }
        let state = state
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| {
                DataFusionError::Plan("snapshot requires configured native session state".into())
            })?;
        if !Arc::ptr_eq(
            &state.runtime_env().memory_pool,
            &self.0.decode().runtime.shared.pool(),
        ) {
            return Err(DataFusionError::Plan(
                "snapshot query uses another deployment pool".into(),
            ));
        }
        StreamingTable::try_new(
            self.schema(),
            vec![Arc::new(SnapshotPartition(self.0.clone()))],
        )?
        .scan(state, projection, filters, limit)
        .await
    }
}

#[derive(Debug)]
struct SnapshotPartition(FrozenSnapshot);
impl PartitionStream for SnapshotPartition {
    fn schema(&self) -> &SchemaRef {
        &self.0.decode().schema
    }
    fn execute(&self, context: Arc<TaskContext>) -> SendableRecordBatchStream {
        let pool_matches = Arc::ptr_eq(
            &context.runtime_env().memory_pool,
            &self.0.decode().runtime.shared.pool(),
        );
        let snapshot = self.0.clone();
        let cancel = CancellationToken::new();
        let stream = futures_util::stream::try_unfold(
            (snapshot, 0usize, None, cancel, pool_matches),
            |(snapshot, mut index, mut cache, cancel, pool_matches)| async move {
                if !pool_matches {
                    return Err(DataFusionError::Execution("snapshot execution uses another deployment pool".into()));
                }
                let batch = snapshot.next(&mut index, &mut cache, &cancel).await.map_err(pse_columnar::external)?;
                let Some(batch) = batch else { return Ok(None); };
                let batch = batch.checked_export(&cancel).map_err(pse_columnar::external)?;
                Ok::<_, DataFusionError>(Some((batch, (snapshot, index, cache, cancel, pool_matches))))
            },
        );
        Box::pin(RecordBatchStreamAdapter::new(self.0.schema(), stream))
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
        let owner = grant(); owner.try_grow(12).unwrap();
        let dense = SelectedRows::from_offsets(vec![2,3,4], owner).unwrap();
        assert!(matches!(dense, SelectedRows::Range {offset: 2, length: 3}));
        assert_eq!(pool.reserved(), 0);
        let owner = grant(); owner.try_grow(12).unwrap();
        let sparse = SelectedRows::from_offsets(vec![0,2,u32::MAX], owner).unwrap();
        assert_eq!(sparse.len(), 3);
        assert_eq!(pool.reserved(), 12);
        drop(sparse);
        assert_eq!(pool.reserved(), 0);
        for offsets in [vec![2,2], vec![2,1]] {
            let owner = grant(); owner.try_grow(8).unwrap();
            assert!(SelectedRows::from_offsets(offsets, owner).is_err());
            assert_eq!(pool.reserved(), 0);
        }
        assert!(SelectedRows::from_offsets(vec![1,2], grant()).is_err());
        assert_eq!(pool.reserved(), 0);
    }
}
