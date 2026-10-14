// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact immutable result inventories shared by typed readers and native queries.

use super::{Runtime, WorkflowError, contract, result_files};
use datafusion::{
    arrow::{datatypes::SchemaRef, record_batch::RecordBatch},
    catalog::{Session, TableProvider, streaming::StreamingTable},
    common::{DataFusionError, Result as NativeResult},
    execution::{TaskContext, session_state::SessionState},
    logical_expr::{Expr, TableProviderFilterPushDown, TableType},
    physical_plan::{
        ExecutionPlan, SendableRecordBatchStream,
        stream::RecordBatchStreamAdapter, streaming::PartitionStream,
    },
};
use pse_columnar::{CancellationToken, MemoryConsumer, MemoryReservation, owned_buffer::AllocationScope};
use pse_model::generated::runtime::{canonical_result_batches::Row as FileBatch, canonical_result_blocks::Row as Block};
use pse_operations::canonical_results::ResultRead;
use std::{collections::BTreeMap, sync::Arc};

const READ_BYTES: usize = 3 * result_files::MAX_FILE_BYTES;

#[derive(Debug)]
struct SnapshotInner {
    runtime: Runtime,
    read: Arc<ResultRead>,
    relation: pse_ids::SemanticId,
    schema: SchemaRef,
    batches: BTreeMap<String, FileBatch>,
    blocks: Vec<Block>,
    positions: BTreeMap<String, usize>,
    start: u64,
    end: u64,
    admission: Arc<pse_relations::validate::ValidationContext>,
    cache: tokio::sync::Mutex<Option<CachedFile>>,
    _owner: MemoryReservation,
}

#[derive(Debug)]
struct CachedFile {
    instance: String,
    reader: datafusion::arrow::ipc::reader::FileReader<pse_operations::canonical_files::ProtectedFile>,
    _owner: MemoryReservation,
}

/// One terminal selection and a frozen explicit file/batch inventory. Clones,
/// physical plans and partition streams retain the same nonexpiring read ticket.
#[derive(Clone, Debug)]
pub struct ResultSnapshot(Arc<SnapshotInner>);
impl ResultSnapshot {
    pub(super) async fn capture(
        runtime: &Runtime, read: Arc<ResultRead>, relation: pse_ids::SemanticId,
        set: &str, schema: SchemaRef, start: u64, end: u64,
    ) -> Result<Self, WorkflowError> {
        use pse_model::HeapUsage;
        let owner = MemoryConsumer::new("canonical:result-snapshot-inventory").register(&runtime.shared.pool());
        owner.try_grow(size_of::<SnapshotInner>() + read.retained_bytes()?).map_err(pse_engine::EngineError::from)?;
        let mut batches = BTreeMap::new();
        let mut blocks = Vec::new();
        let mut positions = BTreeMap::new();
        let mut after = None;
        let mut previous = None;
        while start < end {
            // Each bounded RPC page has its own prepaid temporary response scope.
            // Transfer its exact retained strings/rows before growing inventories.
            let response = Arc::new(MemoryConsumer::new("canonical:result-snapshot-page").register(&runtime.shared.pool()));
            response.try_grow(READ_BYTES).map_err(pse_engine::EngineError::from)?;
            let (page, descriptors) = pse_operations::canonical::response_ownership_scope(response.clone(), async {
                let page = runtime.canonical_store()?.result_block_page(
                    &read, set, &relation.to_string(), "0", start, end, after,
                ).await?;
                let ordinals = page.iter().map(|block| block.ordinal).collect::<Vec<_>>();
                let descriptors = runtime.canonical_store()?.result_file_batch_descriptors(&read, set, &ordinals).await?;
                Ok::<_, WorkflowError>((page, descriptors))
            }).await?;
            if page.len() != descriptors.len() { return Err(contract("snapshot descriptor coverage differs")); }
            let finished = page.len() < 64;
            after = page.last().map(|block| block.ordinal);
            let bytes = page.iter().zip(&descriptors).try_fold(0usize, |bytes, (block, batch)| {
                bytes.checked_add(size_of::<Block>() + block.heap_bytes() + block.key.capacity() + 128
                    + size_of::<FileBatch>() + batch.heap_bytes() + batch.key.capacity() + 128)
                    .ok_or_else(|| contract("snapshot inventory extent"))
            })?;
            owner.try_grow(bytes).map_err(pse_engine::EngineError::from)?;
            blocks.try_reserve_exact(page.len()).map_err(|_| contract("snapshot inventory allocation refused"))?;
            for (block, batch) in page.into_iter().zip(descriptors) {
                if batch.key != block.batch || batch.ordinal != block.ordinal || batch.row_count != block.rows
                    || batch.digest != block.payload_digest || batch.payload_bytes != block.payload_bytes
                    || previous.is_some_and(|end| block.start < end) {
                    return Err(contract("snapshot logical coverage differs"));
                }
                previous = Some(block.end);
                positions.insert(block.key.clone(), blocks.len());
                if batches.insert(batch.key.clone(), batch).is_some() { return Err(contract("duplicate frozen file batch")); }
                blocks.push(block);
            }
            if finished { break; }
        }
        Ok(Self(Arc::new(SnapshotInner {runtime: runtime.clone(), read, relation, schema,
            batches, blocks, positions, start, end, admission: runtime.admission_context(), cache: tokio::sync::Mutex::new(None), _owner: owner})))
    }
    /// Exact registry relation schema, including its original metadata.
    pub fn schema(&self) -> SchemaRef { self.0.schema.clone() }
    /// Actual selected attempt and immutable closure, including partiality class.
    pub fn selection(&self) -> &ResultRead { &self.0.read }
    pub(super) fn blocks(&self) -> &[Block] { &self.0.blocks }
    /// Existing configured native state, including its functions and shared pool.
    pub fn session_state(&self) -> SessionState { self.0.runtime.sessions().native_state().clone() }
    /// A native provider over this frozen inventory; no directory discovery.
    pub fn table_provider(&self) -> Arc<dyn TableProvider> { Arc::new(SnapshotProvider(self.clone())) }

    pub(super) async fn read_block(
        &self, block: &Block, cancel: &CancellationToken,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        if self.0.positions.get(&block.key).and_then(|index| self.0.blocks.get(*index)) != Some(block) { return Err(contract("result block differs from frozen snapshot")); }
        let descriptor = self.0.batches.get(&block.batch).ok_or_else(|| contract("block outside frozen snapshot"))?;
        let owner = MemoryConsumer::new("canonical:result-file-decode").register(&self.0.runtime.shared.pool());
        // Prepay physical integrity reads, footer verification, native body and
        // alignment copies before opening the file or invoking Arrow.
        let bytes = usize::try_from(descriptor.payload_bytes).map_err(|_| contract("file byte extent"))?;
        owner.try_grow(bytes.saturating_mul(4).saturating_add(64 * 1024)).map_err(pse_engine::EngineError::from)?;
        let mut cache = self.0.cache.lock().await;
        if cache.as_ref().is_none_or(|cached| cached.instance != descriptor.file_instance) {
            let metadata = MemoryConsumer::new("canonical:result-file-footer-cache").register(&self.0.runtime.shared.pool());
            metadata.try_grow(bytes.saturating_mul(2).saturating_add(64 * 1024)).map_err(pse_engine::EngineError::from)?;
            let file = self.0.runtime.canonical_store()?.open_result_file(&self.0.read, descriptor).await?;
            let count = usize::try_from(file.descriptor().batch_count).map_err(|_| contract("file batch count extent"))?;
            let reader = result_files::open_file_reader(file, &self.schema(), Some(count))?;
            *cache = Some(CachedFile { instance: descriptor.file_instance.clone(), reader, _owner: metadata });
        }
        let cached = cache.as_mut().ok_or_else(|| contract("file cache absent"))?;
        cached.reader.set_index(usize::try_from(descriptor.file_batch_index).map_err(|_| contract("file batch index extent"))?)
            .map_err(super::result_blocks::ResultBlockError::from)?;
        let batch = cached.reader.next().transpose().map_err(super::result_blocks::ResultBlockError::from)?
            .ok_or_else(|| contract("frozen file batch absent"))?;
        drop(cache);
        if batch.num_rows() as u64 != block.rows || batch.schema() != self.schema() {
            return Err(contract("frozen file batch rows/schema differ"));
        }
        let scope = AllocationScope::default();
        let owned = scope.attach_borrowed_reserved(batch, &owner).map_err(pse_engine::EngineError::from)?;
        let spec = self.0.runtime.registry().relation_by_id(self.0.relation)
            .ok_or_else(|| contract("snapshot relation unavailable"))?;
        pse_relations::columnar::FieldCheckedBatch::admit_owned(self.0.runtime.registry(), spec, owned,
            &self.0.admission, cancel)
            .map_err(pse_engine::EngineError::from).map_err(Into::into)
    }
}

#[derive(Debug)]
struct SnapshotProvider(ResultSnapshot);
#[async_trait::async_trait]
impl TableProvider for SnapshotProvider {
    fn schema(&self) -> SchemaRef { self.0.schema() }
    fn table_type(&self) -> TableType { TableType::Temporary }
    fn supports_filters_pushdown(&self, filters: &[&Expr]) -> NativeResult<Vec<TableProviderFilterPushDown>> {
        Ok(vec![TableProviderFilterPushDown::Unsupported; filters.len()])
    }
    async fn scan(&self, state: &dyn Session, projection: Option<&Vec<usize>>, filters: &[Expr], limit: Option<usize>) -> NativeResult<Arc<dyn ExecutionPlan>> {
        if !filters.is_empty() { return Err(DataFusionError::Plan("snapshot predicates require native residual evaluation".into())); }
        let state = state.as_any().downcast_ref::<SessionState>()
            .ok_or_else(|| DataFusionError::Plan("snapshot requires configured native session state".into()))?;
        if !Arc::ptr_eq(&state.runtime_env().memory_pool, &self.0.0.runtime.shared.pool()) {
            return Err(DataFusionError::Plan("snapshot query uses another deployment pool".into()));
        }
        StreamingTable::try_new(self.schema(), vec![Arc::new(SnapshotPartition(self.0.clone()))])?
            .scan(state, projection, filters, limit).await
    }
}

#[derive(Debug)]
struct SnapshotPartition(ResultSnapshot);
impl PartitionStream for SnapshotPartition {
    fn schema(&self) -> &SchemaRef { &self.0.0.schema }
    fn execute(&self, context: Arc<TaskContext>) -> SendableRecordBatchStream {
        let pool_matches = Arc::ptr_eq(&context.runtime_env().memory_pool, &self.0.0.runtime.shared.pool());
        let snapshot = self.0.clone();
        let cancel = CancellationToken::new();
        let stream = futures_util::stream::try_unfold((snapshot, 0usize, cancel, pool_matches),
            |(snapshot, mut index, cancel, pool_matches)| async move {
                if !pool_matches { return Err(DataFusionError::Execution("snapshot execution uses another deployment pool".into())); }
                while let Some(block) = snapshot.0.blocks.get(index) {
                    index += 1;
                    let lower = snapshot.0.start.max(block.start);
                    let upper = snapshot.0.end.min(block.end);
                    if lower >= upper { continue; }
                    let batch = snapshot.read_block(block, &cancel).await.map_err(pse_columnar::external)?;
                    let batch = batch.checked_export(&cancel).map_err(pse_columnar::external)?;
                    let count = usize::try_from(upper-lower).map_err(|_| pse_columnar::external(contract("snapshot selected row extent")))?;
                    let offset = usize::try_from(lower-block.start).map_err(|_| pse_columnar::external(contract("snapshot selected row offset")))?;
                    let batch = batch.slice(offset, count);
                    return Ok(Some((batch, (snapshot, index, cancel, pool_matches))));
                }
                Ok::<Option<(RecordBatch, _)>, DataFusionError>(None)
            });
        Box::pin(RecordBatchStreamAdapter::new(self.0.schema(), stream))
    }
}
