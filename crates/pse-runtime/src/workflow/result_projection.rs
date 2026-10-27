// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Admitted scientific batches are sorted and sealed with their actual statistics.
//! Dense trajectory ranges retain their original payload coordinates.

use super::{WorkflowError, contract, relation, result_files};
use pse_columnar::{MemoryConsumer, MemoryPool};
use pse_ids::SemanticId;
use pse_model::generated::runtime::{
    canonical_result_block_outputs::Row as Output, canonical_result_blocks::Row as Block,
};
use pse_operations::{
    canonical::CanonicalStore,
    canonical_execution::{
        AdmittedSealedResultFile, AttemptFence, ResultBlockAppend, ResultPayload, result_batch_key,
        result_set_key,
    },
};
use pse_relations::{columnar::FieldCheckedBatch, generated::runtime::simulation_samples};
use std::sync::Arc;

pub(super) fn check_output_field(id: SemanticId, field: &str) -> Result<(), WorkflowError> {
    if !pse_relations::generated::result_selection::for_relation(id)
        .is_some_and(|selection| selection.values.contains(&field))
    {
        return Err(contract(
            "scientific output field has no declared native selection",
        ));
    }
    Ok(())
}

/// Lower a unit-control predicate through the production owner.
#[cfg(test)]
fn scalar_expression(
    relation: SemanticId,
    field: &str,
    minimum: Option<f64>,
    maximum: Option<f64>,
    missing: Option<bool>,
) -> Result<datafusion::logical_expr::Expr, WorkflowError> {
    check_output_field(relation, field)?;
    super::result_selection::ValuePredicate::new(false, minimum, maximum, missing)?.scalar_expression(field)
}

#[cfg(test)]
fn evaluate_scalar_expression(
    state: &datafusion::execution::session_state::SessionState,
    expression: datafusion::logical_expr::Expr,
    batch: &datafusion::arrow::record_batch::RecordBatch,
) -> Result<datafusion::arrow::array::BooleanArray, WorkflowError> {
    let schema = datafusion::common::DFSchema::try_from(batch.schema().as_ref().clone())
        .map_err(pse_engine::EngineError::from)?;
    let physical = state
        .create_physical_expr(expression, &schema)
        .map_err(pse_engine::EngineError::from)?;
    let mask = physical
        .evaluate(batch)
        .map_err(pse_engine::EngineError::from)?
        .into_array(batch.num_rows())
        .map_err(pse_engine::EngineError::from)?;
    mask.as_any()
        .downcast_ref::<datafusion::arrow::array::BooleanArray>()
        .cloned()
        .ok_or_else(|| contract("native scientific predicate did not return Boolean values"))
}

/// Select original scientific owners and relation-specific partitions through
/// the same native expression as the value predicate. Block positions are not keys.
pub(super) fn scalar_selection_expression(
    relation: SemanticId,
    owner: SemanticId,
    partition: &str,
    field: &str,
    predicate: super::result_selection::ValuePredicate,
) -> Result<datafusion::logical_expr::Expr, WorkflowError> {
    use datafusion::{
        common::ScalarValue,
        logical_expr::{col, lit},
    };
    check_output_field(relation, field)?;
    let values = predicate.scalar_expression(field)?;
    let selection = pse_relations::generated::result_selection::for_relation(relation)
        .filter(|selection| !selection.sort.is_empty())
        .ok_or_else(|| contract("declared scalar result selection"))?;
    let mut keys = col(selection.owner).eq(lit(ScalarValue::FixedSizeBinary(
        16,
        Some(owner.as_bytes().to_vec()),
    )));
    if let Some(step) = selection.step {
        let Some(value) = partition
            .strip_prefix("step:")
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|value| *value >= 0 && format!("step:{value}") == partition)
        else {
            return Ok(lit(false));
        };
        keys = keys.and(col(step).eq(lit(value)));
    } else if let Some(experiment) = selection.experiment {
        let Some(value) = partition
            .strip_prefix("experiment:")
            .and_then(|value| SemanticId::parse_hex(value).ok())
            .filter(|value| format!("experiment:{value}") == partition)
        else {
            return Ok(lit(false));
        };
        keys = keys.and(col(experiment).eq(lit(ScalarValue::FixedSizeBinary(
            16,
            Some(value.as_bytes().to_vec()),
        ))));
    } else if partition != "0" {
        return Ok(lit(false));
    }
    Ok(keys.and(values))
}

/// Ingest the sole admitted scientific IPC payload and its admitted batch statistics
/// through the same fenced transaction. IPC and derived metadata independently
/// choose useful prefixes; scalar count is only a defensive extent ceiling.
#[cfg(all(test, feature = "canonical-tests"))]
pub(super) async fn store_result_table(
    store: &CanonicalStore,
    fence: &AttemptFence,
    id: SemanticId,
    cursor: super::ResultCursor<'_>,
    pool: &Arc<dyn MemoryPool>,
    acknowledgments: &super::durable::PublicationAcks,
) -> Result<(), WorkflowError> {
    let mut queue = ResultAppendQueue::new(store, fence, pool, acknowledgments);
    store_result_table_queued(&mut queue, id, cursor, None).await?;
    queue.flush().await
}

/// Homogeneous file packing and bounded sealed metadata have separate lifetimes.
/// Publication coverage advances only from the returned actual opaque ACKs.
pub(super) struct ResultAppendQueue<'a> {
    store: &'a CanonicalStore,
    fence: &'a AttemptFence,
    pool: &'a Arc<dyn MemoryPool>,
    acknowledgments: &'a super::durable::PublicationAcks,
    items: Vec<PendingBatch>,
    file_extent: usize,
    extent: usize,
    deadline: Option<std::time::Instant>,
    appends: Vec<PendingAppend>,
    append_extent: usize,
    append_deadline: Option<std::time::Instant>,
    complete: Vec<SetCompletion>,
    _owner: Option<Arc<pse_columnar::AllocationLease>>,
    #[cfg(test)]
    requests: usize,
    #[cfg(test)]
    projection_retries: usize,
}
struct PendingAppend {
    append: ResultBlockAppend,
    name: String,
    ordinal: u64,
}
struct SetCompletion {
    name: String,
    ordinal: u64,
}
struct PendingBatch {
    name: String,
    ordinal: u64,
    source: FieldCheckedBatch,
    block: Block,
    outputs: Vec<Output>,
    owner: Arc<pse_columnar::AllocationLease>,
}
/// Empty control storage may remain with one live attempt across synchronous
/// per-sample flushes. No scientific payload or unacknowledged extent is reused.
fn append_queue_control(
    pool: &Arc<dyn MemoryPool>,
    retained: Option<Arc<pse_columnar::AllocationLease>>,
) -> Option<Arc<pse_columnar::AllocationLease>> {
    if retained.is_some() {
        return retained;
    }
    let reservation = MemoryConsumer::new("canonical:append-queue-control").register(pool);
    let owner = reservation
        .try_grow(256 * 1024)
        .ok()
        .map(|()| pse_columnar::AllocationLease::new(reservation));
    if owner.is_some()
        && let Some(context) = pse_diagnostics::observation::current()
    {
        context.count("canonical.result.append_queue_control_grants", "count", 1);
    }
    owner
}

impl<'a> ResultAppendQueue<'a> {
    pub(super) fn new(
        store: &'a CanonicalStore,
        fence: &'a AttemptFence,
        pool: &'a Arc<dyn MemoryPool>,
        acknowledgments: &'a super::durable::PublicationAcks,
    ) -> Self {
        Self::with_control(store, fence, pool, acknowledgments, None)
    }
    fn with_control(
        store: &'a CanonicalStore,
        fence: &'a AttemptFence,
        pool: &'a Arc<dyn MemoryPool>,
        acknowledgments: &'a super::durable::PublicationAcks,
        retained: Option<Arc<pse_columnar::AllocationLease>>,
    ) -> Self {
        let owner = append_queue_control(pool, retained);
        Self {
            store,
            fence,
            pool,
            acknowledgments,
            items: Vec::new(),
            file_extent: 0,
            extent: 4096,
            deadline: None,
            appends: Vec::new(),
            append_extent: 4096,
            append_deadline: None,
            complete: Vec::new(),
            _owner: owner,
            #[cfg(test)]
            requests: 0,
            #[cfg(test)]
            projection_retries: 0,
        }
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "the logical slice, its checked allocation owner and immutable append metadata have independent lifetimes"
    )]
    async fn push(
        &mut self,
        name: &str,
        ordinal: u64,
        source: &FieldCheckedBatch,
        block: &Block,
        outputs: &[Output],
    ) -> Result<(), WorkflowError> {
        let offered = std::time::Instant::now() + pse_operations::canonical::REQUEST_TIMEOUT;
        let metadata = 4096usize;
        let metadata =
            outputs
                .iter()
                .try_fold(metadata, |bytes, output| -> Result<_, WorkflowError> {
                    bytes
                        .checked_add(
                            pse_operations::canonical_execution::result_output_metadata_extent(
                                output,
                            )?,
                        )
                        .ok_or_else(|| contract("append metadata extent"))
                })?;
        let file_extent = result_files::batch_extent(source.batch())?;
        let encoded_extent = metadata
            .checked_mul(16)
            .and_then(|bytes| bytes.checked_add(4096))
            .ok_or_else(|| contract("append metadata copy extent"))?;
        if !self.items.is_empty()
            && (self.items.last().is_some_and(|item| item.name != name)
                || self.file_extent.saturating_add(file_extent) > result_files::TARGET_FILE_BYTES
                || self.extent.saturating_add(encoded_extent)
                    > pse_operations::canonical::PAYLOAD_BYTES
                || self.items.len() == pse_operations::generated::surreal::RESULT_INDEX_RECORDS)
        {
            self.seal_pending().await?;
        }
        let reservation =
            MemoryConsumer::new("canonical:owned-file-batch-metadata").register(self.pool);
        if reservation.try_grow(encoded_extent).is_err() {
            self.flush().await?;
            reservation
                .try_grow(encoded_extent)
                .map_err(pse_engine::EngineError::from)?;
        }
        let owner = pse_columnar::AllocationLease::new(reservation);
        if self.items.is_empty() {
            self.deadline = Some(offered);
        }
        self.extent = self
            .extent
            .checked_add(encoded_extent)
            .ok_or_else(|| contract("append group extent"))?;
        self.file_extent = self
            .file_extent
            .checked_add(file_extent)
            .ok_or_else(|| contract("file packing extent"))?;
        self.items.push(PendingBatch {
            name: name.into(),
            ordinal,
            source: source.clone(),
            block: block.clone(),
            outputs: outputs.to_vec(),
            owner,
        });
        if self._owner.is_none() {
            self.flush().await?;
        }
        Ok(())
    }
    /// Seal one homogeneous file, then discard its actual byte and batch working
    /// sets before retaining only independently charged append metadata.
    async fn seal_pending(&mut self) -> Result<(), WorkflowError> {
        if self.items.is_empty() {
            return Ok(());
        }
        let deadline = self
            .deadline
            .take()
            .ok_or_else(|| contract("file group original clock absent"))?;
        let items = std::mem::take(&mut self.items);
        self.extent = 4096;
        self.file_extent = 0;
        let batches = items
            .iter()
            .map(|item| item.source.clone())
            .collect::<Vec<_>>();
        result_files::check_file_batches(
            &batches
                .iter()
                .map(|batch| batch.batch().clone())
                .collect::<Vec<_>>(),
        )?;
        let registry = pse_schema::registry()
            .map_err(pse_relations::RelationError::from)
            .map_err(relation)?;
        let cancel = pse_columnar::CancellationToken::new();
        let encode = || {
            pse_relations::result_batch::AdmittedResultFile::encode(
                registry,
                &batches,
                result_files::MAX_FILE_BYTES,
                self.pool,
                &cancel,
            )
        };
        let admitted = match encode() {
            Ok(file) => file,
            Err(error) => {
                let error = relation(error);
                if !projection_memory_refusal(&error) {
                    return Err(error);
                }
                self.flush_metadata().await?;
                pse_relations::result_batch::AdmittedResultFile::encode(
                    registry,
                    &batches,
                    result_files::MAX_FILE_BYTES,
                    self.pool,
                    &cancel,
                )
                .map_err(relation)?
            }
        };
        let digest = pse_operations::canonical_execution::result_payload_digest(admitted.bytes());
        let operation = format!(
            "file:{}:{}:{}",
            self.fence.attempt(),
            items[0].block.key,
            items[items.len() - 1].ordinal
        );
        let schema = items[0].block.output.clone();
        // Do not spend a newer file's clock while an older metadata operation
        // is awaiting its ACK. This is a stricter boundary, never a renewal.
        let seal_deadline = self
            .append_deadline
            .map_or(deadline, |clock| clock.min(deadline));
        let inline = admitted.bytes().len() <= pse_schema::catalog::substrate::RESULT_INLINE_BYTES;
        let (inline_body, sealed_body) = if inline {
            (Some(Arc::new(admitted)), None)
        } else {
            let file = pse_operations::canonical::operation_scope(seal_deadline, async {
                let intent = self
                    .store
                    .reserve_result_file(self.fence, &operation, &digest)
                    .await?;
                self.store
                    .seal_file(
                        &intent,
                        admitted.bytes(),
                        pse_operations::canonical_files::ARROW_IPC_FILE,
                        batches.len() as u64,
                        &schema,
                    )
                    .await
            })
            .await?;
            (None, Some(AdmittedSealedResultFile::bind(file, admitted)?))
        };
        drop(batches);
        let metadata = items
            .into_iter()
            .map(|item| {
                let PendingBatch {
                    name,
                    ordinal,
                    source,
                    block,
                    outputs,
                    owner,
                } = item;
                drop(source); // Checked-storage/export metadata stays charged through sealing.
                (name, ordinal, block, outputs, owner)
            })
            .collect::<Vec<_>>();
        for (index, (name, ordinal, mut block, outputs, owner)) in metadata.into_iter().enumerate()
        {
            // The sealed payload has no resident bytes. Its existing per-item
            // metadata lease covers the descriptor, intent and request copies.
            let owner = if let Some(body) = &inline_body {
                // Encoded body custody and native request/value copies have distinct owners.
                // Prepay every per-member append representation before constructing it.
                let extent = body
                    .bytes()
                    .len()
                    .checked_mul(16)
                    .and_then(|bytes| bytes.checked_add(owner.size()))
                    .and_then(|bytes| bytes.checked_add(4096))
                    .ok_or_else(|| contract("inline append copy extent"))?;
                let reservation =
                    MemoryConsumer::new("canonical:inline-append-transport").register(self.pool);
                reservation
                    .try_grow(extent)
                    .map_err(pse_engine::EngineError::from)?;
                pse_columnar::AllocationLease::new(reservation)
            } else {
                owner
            };
            let payload = if let Some(body) = &inline_body {
                ResultPayload::from_admitted_encoded_owned(
                    body.clone(),
                    index as u64,
                    &operation,
                    owner.clone(),
                )?
            } else {
                ResultPayload::from_admitted_sealed_owned(
                    sealed_body
                        .as_ref()
                        .ok_or_else(|| contract("selected sealed body absent"))?,
                    index as u64,
                    owner.clone(),
                )?
            };
            block.payload_bytes = payload.byte_len();
            block.payload_digest = payload.digest().to_owned();
            let append = ResultBlockAppend::new(
                self.fence,
                &format!("table:{}:{}", self.fence.attempt(), block.key),
                &name,
                ordinal,
                payload,
                block.rows,
                &block,
                &outputs,
                owner.clone(),
            )?;
            // The prepaid envelope includes final physical-binding framing and
            // all driver copies, not just the current pre-binding wire extent.
            let extent = append
                .encoded_extent()
                .checked_add(4096)
                .ok_or_else(|| contract("sealed append metadata extent"))?;
            if extent > owner.size()
                || extent > pse_schema::catalog::substrate::RESULT_APPEND_TARGET_BYTES - 4096
            {
                return Err(contract("sealed append exceeds admitted metadata extent"));
            }
            if !self.appends.is_empty()
                && (self.append_extent.saturating_add(extent)
                    > pse_schema::catalog::substrate::RESULT_APPEND_TARGET_BYTES
                    || self.appends.len()
                        == pse_operations::generated::surreal::RESULT_INDEX_RECORDS)
            {
                self.flush_metadata().await?;
            }
            self.append_extent += extent;
            self.append_deadline = Some(
                self.append_deadline
                    .map_or(deadline, |clock| clock.min(deadline)),
            );
            self.appends.push(PendingAppend {
                append,
                name,
                ordinal,
            });
        }
        Ok(())
    }
    async fn flush_metadata(&mut self) -> Result<(), WorkflowError> {
        if self.appends.is_empty() {
            return Ok(());
        }
        let deadline = self
            .append_deadline
            .take()
            .ok_or_else(|| contract("append group original clock absent"))?;
        let appends = std::mem::take(&mut self.appends)
            .into_iter()
            .map(|item| item.append)
            .collect();
        self.append_extent = 4096;
        // Completion obligations remain in self across fallible I/O. Only the
        // exact final operation ACK can discharge one, including after settlement.
        let acks = pse_operations::canonical::operation_scope(
            deadline,
            self.store.append_result_blocks(self.fence, appends),
        )
        .await?;
        #[cfg(test)]
        {
            self.requests += 1;
        }
        for ack in acks {
            let completed = self.complete.iter().position(|completion| {
                ack.ordinal == completion.ordinal
                    && ack.result_set == result_set_key(self.fence.attempt(), &completion.name)
            });
            self.acknowledgments.append(self.fence, ack)?;
            if let Some(index) = completed {
                self.acknowledgments
                    .complete(self.fence, &self.complete[index].name)?;
                self.complete.remove(index);
            }
        }
        Ok(())
    }
    pub(super) async fn flush(&mut self) -> Result<(), WorkflowError> {
        self.seal_pending().await?;
        self.flush_metadata().await
    }
    pub(super) async fn yield_optional(&mut self) -> Result<(), WorkflowError> {
        // Join and publish authentic earlier ACKs before releasing optional
        // storage. A denied original encoder reservation may then retry cold.
        self.flush().await?;
        if !self.complete.is_empty() {
            return Err(contract("append completion remains unsettled"));
        }
        // Removing acknowledged entries does not release their Vec backing
        // allocation. Drain that allocation before relinquishing its charge.
        self.complete = Vec::new();
        self._owner = None;
        Ok(())
    }
    pub(super) fn has_optional_retention(&self) -> bool {
        self._owner.is_some() || !self.items.is_empty() || !self.appends.is_empty()
    }
    fn try_enable(&mut self) {
        if self._owner.is_none() {
            let reservation =
                MemoryConsumer::new("canonical:append-queue-control").register(self.pool);
            self._owner = reservation
                .try_grow(256 * 1024)
                .ok()
                .map(|()| pse_columnar::AllocationLease::new(reservation));
        }
    }
    /// Callers demanding an ACK keep this explicit barrier. Ordinary complete
    /// run exports use finish_deferred and join once at store_tables' boundary.
    #[cfg(all(test, feature = "canonical-tests"))]
    async fn finish(
        &mut self,
        id: SemanticId,
        position: ResultStreamPosition,
    ) -> Result<(), WorkflowError> {
        self.finish_deferred(id, position).await?;
        self.flush().await
    }
    async fn finish_deferred(
        &mut self,
        id: SemanticId,
        position: ResultStreamPosition,
    ) -> Result<(), WorkflowError> {
        let name = id.to_string();
        if position.ordinal == 0 {
            self.flush().await?;
            self.acknowledgments.expect(self.fence, &name)?;
            let ack = self
                .store
                .declare_empty_result_set(
                    self.fence,
                    &format!("empty:{}:{name}", self.fence.attempt()),
                    &name,
                )
                .await?;
            self.acknowledgments.empty(self.fence, ack)?;
            self.acknowledgments.complete(self.fence, &name)?;
        } else if self._owner.is_none() {
            self.flush().await?;
            self.acknowledgments.complete(self.fence, &name)?;
        } else {
            let ordinal = position.ordinal - 1;
            if self
                .items
                .iter()
                .any(|item| item.name == name && item.ordinal == ordinal)
                || self
                    .appends
                    .iter()
                    .any(|item| item.name == name && item.ordinal == ordinal)
            {
                if self.complete.len() == 256 {
                    return Err(contract("append queue completed set extent"));
                }
                self.complete.push(SetCompletion { name, ordinal });
                self.seal_pending().await?;
            } else {
                // An allocating producer may have required the final append's
                // ACK before its last (possibly empty) cursor poll.
                self.acknowledgments.complete(self.fence, &name)?;
            }
        }
        Ok(())
    }
}

/// Whole-relation fixture export; production pure outcomes use the typed route.
#[cfg(all(test, feature = "canonical-tests"))]
pub(super) async fn store_result_table_queued(
    queue: &mut ResultAppendQueue<'_>,
    id: SemanticId,
    cursor: super::ResultCursor<'_>,
    allowance: Option<&mut super::study_demand::DetailAllowance>,
) -> Result<(), WorkflowError> {
    let mut position = ResultStreamPosition::default();
    if let Err(error) =
        append_result_table_queued(queue, id, cursor, &mut position, allowance).await
    {
        queue.flush().await?;
        return Err(error);
    }
    queue.finish(id, position).await
}

/// Recorded complete relations share production's deferred append queue; the
/// fixture caller must flush once before closing its ingestion fence.
#[cfg(all(test, feature = "canonical-tests"))]
pub(super) async fn store_recorded_table_deferred(
    queue: &mut ResultAppendQueue<'_>,
    id: SemanticId,
    cursor: super::ResultCursor<'_>,
) -> Result<(), WorkflowError> {
    let mut position = ResultStreamPosition::default();
    if let Err(error) = append_result_table_queued(queue, id, cursor, &mut position, None).await {
        queue.flush().await?;
        return Err(error);
    }
    queue.finish_deferred(id, position).await
}

/// Only concrete memory admission failures permit rebuilding a pure projection.
/// Cancellation, scientific/field refusal and mixed error collections never do.
pub(super) fn projection_memory_refusal(error: &WorkflowError) -> bool {
    fn native(error: &datafusion::common::DataFusionError) -> bool {
        use datafusion::common::DataFusionError;
        match error {
            DataFusionError::ResourcesExhausted(_) => true,
            DataFusionError::Context(_, inner) | DataFusionError::Diagnostic(_, inner) => {
                native(inner)
            }
            DataFusionError::Shared(inner) => native(inner),
            DataFusionError::Collection(errors) => !errors.is_empty() && errors.iter().all(native),
            _ => false,
        }
    }
    fn relation(error: &pse_relations::RelationError) -> bool {
        match error {
            pse_relations::RelationError::Preparation(inner) => relation(inner),
            pse_relations::RelationError::Engine(error) => native(error.native()),
            pse_relations::RelationError::Canon(pse_columnar::CanonError::NativeResource(
                error,
            )) => native(error),
            pse_relations::RelationError::Canon(pse_columnar::CanonError::Reservation(
                pse_columnar::ReserveError::Exhausted { .. },
            )) => true,
            _ => false,
        }
    }
    fn engine(error: &pse_engine::EngineError) -> bool {
        match error {
            pse_engine::EngineError::Engine(error) => native(error.native()),
            pse_engine::EngineError::Relation(error) => relation(error),
            pse_engine::EngineError::Multiple { errors } => {
                !errors.is_empty() && errors.iter().all(engine)
            }
            _ => false,
        }
    }
    match error {
        WorkflowError::Shared(inner) => projection_memory_refusal(inner),
        WorkflowError::Engine(error) => engine(error),
        _ => false,
    }
}

struct ProjectionCancellation(pse_columnar::CancellationToken);
impl Drop for ProjectionCancellation {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

/// A typed original outcome supplies repeatable *encoding*, never repeated
/// science. Optional append storage may coexist with this producer; an actual
/// memory refusal gets one cold retry at the last delivered original row.
pub(super) async fn store_run_result_table_queued(
    queue: &mut ResultAppendQueue<'_>,
    result: &super::RunResult,
    id: SemanticId,
    selector: Option<&super::study_demand::StudyMemberSelection>,
    allowance: Option<&mut super::study_demand::DetailAllowance>,
) -> Result<(), WorkflowError> {
    let result = append_run_result_table_queued(queue, result, id, selector, allowance).await;
    if result.is_err() {
        queue.flush().await?;
    }
    result
}
async fn append_run_result_table_queued(
    queue: &mut ResultAppendQueue<'_>,
    result: &super::RunResult,
    id: SemanticId,
    selector: Option<&super::study_demand::StudyMemberSelection>,
    mut allowance: Option<&mut super::study_demand::DetailAllowance>,
) -> Result<(), WorkflowError> {
    queue.acknowledgments.expect(queue.fence, &id.to_string())?;
    let scope = ProjectionCancellation(pse_columnar::CancellationToken::new());
    let mut delivered = 0usize;
    let mut cold_retry = false;
    let mut position = ResultStreamPosition::default();
    let mut cursor = result
        .cursor_by_id_range_controlled(
            id,
            1024,
            super::ResultOrder::Canonical,
            delivered..usize::MAX,
            scope.0.clone(),
        )
        .map_err(WorkflowError::Shared)?;
    loop {
        let table = match cursor.next_batch().await {
            Ok(Some(table)) => table,
            Ok(None) => break,
            Err(error)
                if !cold_retry
                    && queue.has_optional_retention()
                    && projection_memory_refusal(&error) =>
            {
                cursor.release_failed_projection()?;
                drop(cursor); // Actual pure producer drain precedes append drain/reclaim.
                queue.yield_optional().await?;
                if scope.0.is_cancelled() {
                    return Err(pse_engine::EngineError::Cancelled.into());
                }
                cold_retry = true;
                #[cfg(test)]
                {
                    queue.projection_retries += 1;
                }
                cursor = result
                    .cursor_by_id_range_controlled(
                        id,
                        1024,
                        super::ResultOrder::Canonical,
                        delivered..usize::MAX,
                        scope.0.clone(),
                    )
                    .map_err(WorkflowError::Shared)?;
                continue;
            }
            Err(error) => {
                queue.flush().await?;
                return Err(WorkflowError::Shared(error));
            }
        };
        delivered = delivered
            .checked_add(table.batch().num_rows())
            .ok_or_else(|| contract("projection original row extent"))?;
        let table = if let Some(selector) = selector {
            match selector.select(table.clone(), &scope.0) {
                Ok(selected) => selected,
                Err(error)
                    if queue.has_optional_retention() && projection_memory_refusal(&error) =>
                {
                    queue.yield_optional().await?;
                    selector.select(table, &scope.0)?
                }
                Err(error) => return Err(error),
            }
        } else {
            table
        };
        if let Some(allowance) = allowance.as_mut() {
            allowance.accept(&table)?;
        }
        let end = position
            .origin
            .checked_add(table.batch().num_rows() as u64)
            .ok_or_else(|| contract("result global row overflow"))?;
        queue.try_enable();
        store_result_chunk(queue, id, &table, position.origin, &mut position.ordinal).await?;
        position.origin = end;
    }
    if !cursor.complete() {
        return Err(contract("original result relation did not complete"));
    }
    queue.finish_deferred(id, position).await
}

/// Issued append positions belong to the sole serial producer. Authoritative
/// coverage belongs independently to PublicationAcks and advances only on ACK.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ResultStreamPosition {
    pub(super) origin: u64,
    pub(super) ordinal: u64,
}

/// Append an exhausted original producer without closing the relation. Horizon
/// prefixes remain private until their one final coherent attempt close.
#[allow(
    clippy::too_many_arguments,
    reason = "prefix publication keeps store fencing, source cursor, allocation, ACK state, stream position and detail allowance explicit"
)]
pub(super) async fn append_result_table_prefix(
    store: &CanonicalStore,
    fence: &AttemptFence,
    id: SemanticId,
    cursor: super::ResultCursor<'_>,
    pool: &Arc<dyn MemoryPool>,
    acknowledgments: &super::durable::PublicationAcks,
    position: &mut ResultStreamPosition,
    allowance: Option<&mut super::study_demand::DetailAllowance>,
) -> Result<(), WorkflowError> {
    let mut queue = ResultAppendQueue::new(store, fence, pool, acknowledgments);
    if let Err(error) =
        append_result_table_queued(&mut queue, id, cursor, position, allowance).await
    {
        queue.flush().await?;
        return Err(error);
    }
    queue.flush().await?;
    Ok(())
}

async fn append_result_table_queued(
    queue: &mut ResultAppendQueue<'_>,
    id: SemanticId,
    mut cursor: super::ResultCursor<'_>,
    position: &mut ResultStreamPosition,
    mut allowance: Option<&mut super::study_demand::DetailAllowance>,
) -> Result<(), WorkflowError> {
    // Advertised demand precedes producer polling and every append effect. An
    // early producer refusal stays incomplete rather than disappearing at close.
    queue.acknowledgments.expect(queue.fence, &id.to_string())?;
    loop {
        // Unknown producers may need all formerly available cold capacity for
        // their next row-copy chunk. Do not retain optional append storage over
        // that poll or replay a producer which has already failed admission.
        if !cursor.owns_checked_buffers() {
            queue.yield_optional().await?;
        }
        let Some(table) = cursor.next_batch().await.map_err(WorkflowError::Shared)? else {
            break;
        };
        if let Some(allowance) = allowance.as_mut() {
            allowance.accept(&table)?;
        }
        let end = position
            .origin
            .checked_add(table.batch().num_rows() as u64)
            .ok_or_else(|| contract("result global row overflow"))?;
        queue.try_enable();
        store_result_chunk(queue, id, &table, position.origin, &mut position.ordinal).await?;
        position.origin = end;
    }
    if !cursor.complete() {
        return Err(contract("result relation producer did not complete"));
    }
    Ok(())
}

pub(super) async fn finish_result_table_prefix(
    store: &CanonicalStore,
    fence: &AttemptFence,
    id: SemanticId,
    acknowledgments: &super::durable::PublicationAcks,
    position: ResultStreamPosition,
) -> Result<(), WorkflowError> {
    let name = id.to_string();
    if position.ordinal == 0 {
        acknowledgments.expect(fence, &name)?;
        let ack = store
            .declare_empty_result_set(fence, &format!("empty:{}:{name}", fence.attempt()), &name)
            .await?;
        acknowledgments.empty(fence, ack)?;
    }
    acknowledgments.complete(fence, &name)?;
    Ok(())
}

/// One sealed live batch advances coverage only after its actual append ACK.
pub(super) async fn append_live_sample_batch(
    store: &CanonicalStore,
    fence: &AttemptFence,
    pool: &Arc<dyn MemoryPool>,
    acknowledgments: &super::durable::PublicationAcks,
    position: &mut ResultStreamPosition,
    retained_control: &mut Option<Arc<pse_columnar::AllocationLease>>,
    table: &FieldCheckedBatch,
) -> Result<(), WorkflowError> {
    let id = pse_relations::generated::runtime::live_simulation_samples::RELATION_ID;
    if table.relation_id() != id {
        return Err(contract(
            "live sample relation differs from declared producer",
        ));
    }
    acknowledgments.expect(fence, &id.to_string())?;
    let end = position
        .origin
        .checked_add(table.batch().num_rows() as u64)
        .ok_or_else(|| contract("live sample row extent"))?;
    let mut queue = ResultAppendQueue::with_control(
        store,
        fence,
        pool,
        acknowledgments,
        retained_control.take(),
    );
    store_result_chunk(
        &mut queue,
        id,
        table,
        position.origin,
        &mut position.ordinal,
    )
    .await?;
    queue.flush().await?;
    *retained_control = queue._owner.take();
    position.origin = end;
    Ok(())
}

async fn store_result_chunk(
    queue: &mut ResultAppendQueue<'_>,
    id: SemanticId,
    table: &FieldCheckedBatch,
    origin: u64,
    ordinal: &mut u64,
) -> Result<(), WorkflowError> {
    let fence = queue.fence;
    let pool = queue.pool;
    if id != table.relation_id() {
        return Err(contract("result table relation identity mismatch"));
    }
    if table.batch().num_rows() == 0 {
        return Ok(());
    }
    if id == simulation_samples::RELATION_ID {
        return store_trajectory(queue, table, origin, ordinal).await;
    }
    let sorted = pse_relations::result_batch::sort_new_scalar(
        table,
        pool,
        &pse_columnar::CancellationToken::new(),
    )
    .map_err(relation)?;
    let table = &sorted;
    let reservation = MemoryConsumer::new("canonical:result-batch-encoding").register(pool);
    // Native encoding and descriptor copies retain their own prepaid allowance.
    let name = id.to_string();
    let set = result_set_key(fence.attempt(), &name);
    let window = 32768;
    let scratch = 16 * pse_operations::canonical_execution::EXECUTION_METADATA_BYTES;
    if reservation.try_grow(scratch).is_err() {
        queue.yield_optional().await?;
        reservation
            .try_grow(scratch)
            .map_err(pse_engine::EngineError::from)?;
    }
    let _encoder_owner = pse_columnar::AllocationLease::new(reservation);
    for base in (0..table.batch().num_rows()).step_by(window) {
        let count = window.min(table.batch().num_rows() - base);
        let mut offset = 0;
        while offset < count {
            let current = *ordinal;
            let key = result_batch_key(fence.attempt(), &set, current);
            let rows =
                result_files::result_prefix_rows(table.batch(), base + offset, count - offset)?;
            let payload = table.slice(base + offset, rows).map_err(relation)?;
            let start = origin
                .checked_add((base + offset) as u64)
                .ok_or_else(|| contract("result global row overflow"))?;
            let block = Block {
                key: key.clone(),
                batch: key,
                result_set: set.clone(),
                output: name.clone(),
                partition: "0".into(),
                ordinal: current,
                start,
                end: start + rows as u64,
                rows: rows as u64,
                columns: table.batch().num_columns() as u64,
                coordinate_min: None,
                coordinate_max: None,
                payload_bytes: 0,
                payload_digest: String::new(),
                statistics: None,
                interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
            };
            queue.push(&name, current, &payload, &block, &[]).await?;
            *ordinal += 1;
            offset += rows;
        }
    }
    Ok(())
}

// Partition one already admitted window. Offsets remain original batch offsets;
// the caller prepays partition, range, tree and descriptor storage together.
fn trajectory_ranges(view: &simulation_samples::View<'_>, base: usize, rows: usize) -> Result<Vec<std::ops::Range<usize>>, WorkflowError> {
    let end = base.checked_add(rows).filter(|end| *end <= view.len())
        .ok_or_else(|| contract("trajectory output range outside original rows"))?;
    let symbols: datafusion::arrow::array::ArrayRef = Arc::new(view.symbol_id_column().slice(base, rows));
    let mut ranges = datafusion::arrow::compute::partition(&[symbols]).map_err(pse_columnar::CanonError::from).map_err(pse_engine::EngineError::from)?.ranges();
    let mut seen = std::collections::BTreeSet::new();
    for range in &mut ranges {
        range.start += base; range.end += base;
        let bytes: [u8; 16] = view.symbol_id_column().value(range.start).try_into()
            .map_err(|_| contract("trajectory symbol identity width"))?;
        if !seen.insert(SemanticId::from_bytes(bytes)) { return Err(contract("canonical trajectory symbol is not contiguous")); }
    }
    if ranges.last().is_some_and(|range| range.end != end) { return Err(contract("trajectory partition extent")); }
    Ok(ranges)
}
fn outputs_from_ranges(table: &FieldCheckedBatch, view: &simulation_samples::View<'_>, ranges: &[std::ops::Range<usize>], key: &str, set: &str, origin: u64, accepted_end: usize) -> Result<Vec<Output>, WorkflowError> {
    let mut outputs = Vec::with_capacity(ranges.len());
    for range in ranges.iter().take_while(|range| range.start < accepted_end) {
        let end = range.end.min(accepted_end);
        let symbol = SemanticId::from_bytes(view.symbol_id_column().value(range.start).try_into()
            .map_err(|_| contract("trajectory symbol identity width"))?);
        // Preserve Rust f64 folding, including both zero signs and NaN handling.
        let (min, max) = view.time_column().values()[range.start..end].iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), time| (min.min(*time), max.max(*time)));
        outputs.push(Output {
            key: format!("{key}:value:{:04}", outputs.len()), batch: key.into(), result_set: set.into(),
            output: format!("{}:{symbol}:value", table.relation_id()), partition: "0".into(),
            start: origin.checked_add(range.start as u64).ok_or_else(|| contract("trajectory row extent"))?,
            end: origin.checked_add(end as u64).ok_or_else(|| contract("trajectory row extent"))?,
            coordinate_min: Some(min), coordinate_max: Some(max), interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        });
    }
    Ok(outputs)
}
#[cfg(test)]
fn trajectory_outputs(table: &FieldCheckedBatch, key: &str, set: &str, origin: u64, base: usize, rows: usize) -> Result<pse_columnar::Leased<Vec<Output>>, WorkflowError> {
    let bytes = super::result_selection::trajectory_index_extent(rows).ok_or_else(|| contract("trajectory index extent"))?;
    let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(bytes));
    let owner = MemoryConsumer::new("fixture:trajectory-index").register(&pool);
    owner.try_grow(bytes).map_err(pse_engine::EngineError::from)?;
    let view = simulation_samples::View::from_checked(table).map_err(relation)?;
    let ranges = trajectory_ranges(&view, base, rows)?;
    let output = outputs_from_ranges(table, &view, &ranges, key, set, origin, base + rows)?;
    Ok(pse_columnar::Leased::new(Arc::new(output), pse_columnar::AllocationLease::new(owner)))
}

async fn store_trajectory(
    queue: &mut ResultAppendQueue<'_>,
    table: &FieldCheckedBatch,
    origin: u64,
    ordinal: &mut u64,
) -> Result<(), WorkflowError> {
    let fence = queue.fence;
    let pool = queue.pool;
    let id = table.relation_id();
    let name = id.to_string();
    let set = result_set_key(fence.attempt(), &name);
    let owner = MemoryConsumer::new("canonical:trajectory-output-index").register(pool);
    // Bounded output descriptors and their encoded metadata coexist with the
    // original checked chunk, writer scratch and one self-contained IPC payload.
    let scratch = super::result_selection::trajectory_index_extent(1024).ok_or_else(|| contract("trajectory index extent"))?;
    if owner.try_grow(scratch).is_err() {
        queue.yield_optional().await?;
        owner
            .try_grow(scratch)
            .map_err(pse_engine::EngineError::from)?;
    }
    let _encoder_owner = pse_columnar::AllocationLease::new(owner);
    let view = simulation_samples::View::from_checked(table).map_err(relation)?;
    let mut base = 0;
    while base < table.batch().num_rows() {
        let key = result_batch_key(fence.attempt(), &set, *ordinal);
        let count = (table.batch().num_rows() - base).min(1024);
        let ranges = trajectory_ranges(&view, base, count)?;
        let candidates = outputs_from_ranges(table, &view, &ranges, &key, &set, origin, base + count)?;
        let mut metadata = 4096_usize;
        let mut count = 0;
        for output in &candidates {
            let extent =
                pse_operations::canonical_execution::result_output_metadata_extent(output)?;
            if metadata
                .checked_add(extent)
                .is_none_or(|n| n > pse_operations::canonical_execution::EXECUTION_METADATA_BYTES)
            {
                break;
            }
            metadata += extent;
            count = usize::try_from(output.end - origin - base as u64)
                .map_err(|_| contract("trajectory prefix extent"))?;
        }
        drop(candidates);
        if count == 0 {
            return Err(contract(
                "single trajectory output exceeds metadata admission",
            ));
        }
        let rows = result_files::result_prefix_rows(table.batch(), base, count)?;
        let payload = table.slice(base, rows).map_err(relation)?;
        let outputs = outputs_from_ranges(table, &view, &ranges, &key, &set, origin, base + rows)?;
        let start = origin
            .checked_add(base as u64)
            .ok_or_else(|| contract("trajectory row extent"))?;
        let end = start
            .checked_add(rows as u64)
            .ok_or_else(|| contract("trajectory row extent"))?;
        let block = Block {
            key: key.clone(),
            batch: key.clone(),
            result_set: set.clone(),
            output: name.clone(),
            partition: "0".into(),
            ordinal: *ordinal,
            start,
            end,
            rows: rows as u64,
            columns: table.batch().num_columns() as u64,
            coordinate_min: None,
            coordinate_max: None,
            payload_bytes: 0,
            payload_digest: String::new(),
            statistics: None,
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        };
        queue
            .push(&name, *ordinal, &payload, &block, &outputs)
            .await?;
        *ordinal += 1;
        base += rows;
    }
    Ok(())
}

#[cfg(test)]
mod result_projection_unit {
    use super::*;
    use datafusion::arrow::{
        array::{Array, BooleanArray, Float64Array},
        compute::filter_record_batch,
    };
    use pse_relations::generated::runtime::fit_parameters;
    #[cfg(feature = "canonical-tests")]
    use pse_relations::generated::runtime::solve_constraints;

    fn validation() -> pse_relations::validate::ValidationContext {
        pse_relations::validate::ValidationContext::new(
            pse_schema::registry().unwrap(),
            pse_engine::validation::NativeValidation(
                datafusion::prelude::SessionContext::new().state(),
            ),
        )
    }
    fn selected(mask: &BooleanArray) -> Vec<usize> {
        (0..mask.len())
            .filter(|index| mask.is_valid(*index) && mask.value(*index))
            .collect()
    }

    #[test]
    fn native_scalar_predicates_keep_ieee_payloads_and_conditional_finite_modes() {
        use datafusion::arrow::{
            datatypes::{DataType, Field, Schema},
            record_batch::RecordBatch,
        };
        // This is an expression-control batch, not an admitted scientific relation.
        let nan = f64::from_bits(0x7ff8_0000_0000_0042);
        let tiny = f64::from_bits(1);
        let values = Arc::new(Float64Array::from(vec![
            None,
            Some(f64::NEG_INFINITY),
            Some(f64::INFINITY),
            Some(nan),
            Some(-0.0),
            Some(0.0),
            Some(tiny),
            Some(f64::MAX),
            Some(-3.0),
        ]));
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "value",
                DataType::Float64,
                true,
            )])),
            vec![values.clone()],
        )
        .unwrap();
        let state = datafusion::prelude::SessionContext::new().state();
        let evaluate = |minimum, maximum, missing| {
            let expression = scalar_expression(
                fit_parameters::RELATION_ID,
                "value",
                minimum,
                maximum,
                missing,
            )
            .unwrap();
            evaluate_scalar_expression(&state, expression, &batch).unwrap()
        };
        assert_eq!(
            selected(&evaluate(None, None, None)),
            (0..9).collect::<Vec<_>>()
        );
        assert_eq!(selected(&evaluate(None, None, Some(true))), [0]);
        assert_eq!(
            selected(&evaluate(None, None, Some(false))),
            [4, 5, 6, 7, 8]
        );
        let zeros = evaluate(Some(-0.0), Some(0.0), None);
        assert_eq!(selected(&zeros), [4, 5]);
        assert_eq!(selected(&evaluate(Some(tiny), Some(tiny), None)), [6]);
        let selected = filter_record_batch(&batch, &zeros).unwrap();
        let selected = selected
            .column(0)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(selected.value(0).to_bits(), (-0.0f64).to_bits());
        assert_eq!(selected.value(1).to_bits(), 0.0f64.to_bits());
        assert_eq!(values.value(3).to_bits(), nan.to_bits());
        assert_eq!(values.value(6).to_bits(), tiny.to_bits());
    }
    #[test]
    fn projection_retry_requires_concrete_memory_cause() {
        use datafusion::common::DataFusionError;
        let memory = || {
            pse_engine::EngineError::from(DataFusionError::ResourcesExhausted("actual pool".into()))
        };
        assert!(projection_memory_refusal(&WorkflowError::Shared(Arc::new(
            memory().into()
        ))));
        assert!(!projection_memory_refusal(
            &pse_engine::EngineError::Cancelled.into()
        ));
        assert!(!projection_memory_refusal(
            &pse_engine::EngineError::ResourceLimit {
                consumer: "not a concrete allocation cause".into(),
                config_keys: vec![],
                detail: "resource limit".into(),
            }
            .into()
        ));
        assert!(!projection_memory_refusal(
            &pse_engine::EngineError::from(DataFusionError::Collection(vec![
                DataFusionError::ResourcesExhausted("pool".into()),
                DataFusionError::Execution("scientific refusal".into())
            ]))
            .into()
        ));
        assert!(!projection_memory_refusal(&contract("producer refused")));
    }

    #[cfg(all(feature = "canonical-tests", feature = "solver-kinsol"))]
    #[tokio::test]
    async fn canonical_append_queue_real_projections_preserve_ack_boundaries_and_memory_refusal() {
        use pse_relations::generated::{
            enums::NativeMetricKind,
            runtime::{run_lineage, solve_metrics, solve_runs},
        };
        let runtime = super::super::durable_tests::durable_runtime();
        let ephemeral = runtime
            .clone()
            .with_durability(super::super::DurabilitySelection::Ephemeral)
            .unwrap();
        let (package, analysis) = super::super::durable_tests::package_on(
            &ephemeral,
            super::super::durable_tests::LINEAR,
        )
        .await;
        let prepared = package
            .prepare_analysis(
                &ephemeral.preparation_services(),
                &analysis,
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let result = prepared
            .start(&ephemeral.execution_services())
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert!(result.usable(), "{:?}", result.report());
        let super::super::Durability::Durable(operations) = runtime.durability() else {
            panic!("canonical projection fixture requires durable operations")
        };
        let store = operations.store();
        let revision = store
            .edit("actual-projection", None, "immutable original result", &[])
            .await
            .unwrap();
        store
            .begin_run(&pse_operations::canonical_execution::RunRequest {
                purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
                admission_study: None,
                admission_point: None,
                key: "actual-projection-run".into(),
                revision,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run(
                "actual-projection-run",
                "actual-projection-claim",
                "worker",
                std::time::Duration::from_secs(60),
            )
            .await
            .unwrap();
        let acks = super::super::durable::PublicationAcks::new(&operations.pool);
        let mut queue = ResultAppendQueue::new(store, &fence, &operations.pool, &acks);
        for id in [run_lineage::RELATION_ID, solve_runs::RELATION_ID] {
            assert!(
                !result
                    .cursor_by_id(id, 1024, super::super::ResultOrder::Canonical)
                    .unwrap()
                    .owns_checked_buffers()
            );
            store_run_result_table_queued(&mut queue, &result, id, None, None)
                .await
                .unwrap();
        }
        assert_eq!(
            queue.items.len(),
            0,
            "each complete projection closes its admitted homogeneous body"
        );
        assert_eq!(queue.appends.len(), 2);
        assert_eq!(queue.complete.len(), 2);
        assert_eq!(queue.requests, 0);
        queue.flush().await.unwrap();
        assert_eq!(queue.requests, 1);
        assert!(queue.complete.is_empty());
        drop(queue);

        // A valid large queued immutable metric consumes enough optional bytes
        // for cold encoder capacity to fit after the authentic append joins.
        let revision = store
            .edit(
                "cold-projection",
                None,
                "same original scientific result",
                &[],
            )
            .await
            .unwrap();
        store
            .begin_run(&pse_operations::canonical_execution::RunRequest {
                purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
                admission_study: None,
                admission_point: None,
                key: "cold-projection-run".into(),
                revision,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run(
                "cold-projection-run",
                "cold-projection-claim",
                "worker",
                std::time::Duration::from_secs(60),
            )
            .await
            .unwrap();
        let acks = super::super::durable::PublicationAcks::new(&operations.pool);
        let mut queue = ResultAppendQueue::new(store, &fence, &operations.pool, &acks);
        let validation = runtime.validation_context();
        let mut metrics =
            solve_metrics::Builder::with_registry(runtime.registry(), 1, &validation).unwrap();
        metrics
            .push(solve_metrics::Row {
                run_id: pse_operations::mint_id(),
                step: 0,
                namespace: "original".into(),
                name: "large".into(),
                kind: NativeMetricKind::Text,
                real: None,
                integer: None,
                boolean: None,
                text: Some("x".repeat(450 * 1024)),
                unavailable: None,
            })
            .unwrap();
        store_result_table_queued(
            &mut queue,
            solve_metrics::RELATION_ID,
            super::super::ResultCursor::from_checked(metrics.finish().unwrap()),
            None,
        )
        .await
        .unwrap();
        assert!(queue.items.is_empty());
        let datafusion::execution::memory_pool::MemoryLimit::Finite(limit) =
            operations.pool.memory_limit()
        else {
            panic!("canonical projection fixture requires a finite shared pool")
        };
        let pressure =
            MemoryConsumer::new("test:typed-projection-cold-pressure").register(&operations.pool);
        pressure
            .try_grow(limit - operations.pool.reserved() - 16)
            .unwrap();
        let refused = store_run_result_table_queued(
            &mut queue,
            &result,
            run_lineage::RELATION_ID,
            None,
            None,
        )
        .await
        .unwrap_err();
        assert!(
            projection_memory_refusal(&refused),
            "a live external owner prevents file encoding: {refused:?}"
        );
        assert!(
            queue.projection_retries <= 1,
            "a cold retry never renews or multiplies the operation"
        );
        assert!(
            acks.complete(&fence, &run_lineage::RELATION_ID.to_string())
                .is_err()
        );
        drop(pressure);
        queue.flush().await.unwrap();
        assert_eq!(
            queue.requests, 1,
            "only the prior completed metric was acknowledged"
        );
        assert_eq!(
            pse_operations::testing::pending_result_batch_rows(
                store,
                &fence,
                &run_lineage::RELATION_ID.to_string(),
                0
            )
            .await
            .unwrap(),
            None
        );
        drop(queue);
        store.remove_isolated_fixture().await.unwrap();
    }

    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn canonical_append_queue_cancelled_flush_preserves_refusal_and_incomplete_coverage() {
        use pse_model::study::{OccurrenceKey, PointPolicy, SeedNeed, StartPolicy};
        use pse_operations::canonical_studies::{NewOccurrence, point_key};
        use pse_relations::generated::{enums::NativeMetricKind, runtime::solve_metrics};
        let runtime = super::super::durable_tests::durable_runtime();
        let super::super::Durability::Durable(operations) = runtime.durability() else {
            panic!("canonical projection fixture requires durable operations")
        };
        let store = operations.store();
        let revision = store
            .edit("cancelled-append-queue-fixture", None, "source", &[])
            .await
            .unwrap();
        let request = |key: &str| pse_operations::canonical_execution::RunRequest {
            purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
            admission_study: None,
            admission_point: None,
            key: key.into(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        };
        let study = "cancelled-append-queue-study";
        let occurrence = OccurrenceKey(0);
        store
            .create_study(
                study,
                &request("cancelled-append-queue-summary"),
                &[9],
                &[NewOccurrence {
                    projection: None,
                    policy: PointPolicy {
                        key: occurrence,
                        dependencies: vec![],
                        start: StartPolicy::Fresh,
                        seed_need: SeedNeed::NotNeeded,
                        attempt_limit: 1,
                    },
                    descriptor: vec![42],
                    run: request("cancelled-append-queue-run"),
                }],
                &|| false,
            )
            .await
            .unwrap();
        let scope = store
            .study_scope(&point_key(study, occurrence))
            .await
            .unwrap();
        let fence = store
            .claim_study_point(
                &scope,
                None,
                "cancelled-append-queue-claim",
                "worker",
                std::time::Duration::from_secs(60),
            )
            .await
            .unwrap()
            .fence;
        let validation = runtime.validation_context();
        let mut metrics =
            solve_metrics::Builder::with_registry(runtime.registry(), 1, &validation).unwrap();
        metrics
            .push(solve_metrics::Row {
                run_id: pse_operations::mint_id(),
                step: 0,
                namespace: "independent".into(),
                name: "small".into(),
                kind: NativeMetricKind::Real,
                real: Some(-0.0),
                integer: None,
                boolean: None,
                text: None,
                unavailable: None,
            })
            .unwrap();
        let metrics = metrics.finish().unwrap();
        let empty = solve_constraints::Builder::with_registry(runtime.registry(), 0, &validation)
            .unwrap()
            .finish()
            .unwrap();
        let acks = super::super::durable::PublicationAcks::new(&operations.pool);
        let before = operations.pool.reserved();
        let mut queue = ResultAppendQueue::new(store, &fence, &operations.pool, &acks);
        let idle = operations.pool.reserved();
        assert!(idle > before, "queue control is admitted before allocation");
        for (id, table) in [(solve_metrics::RELATION_ID, metrics)] {
            let original = [table];
            let cursor = super::super::ResultCursor::from_retained(
                original[0].batch().schema(),
                id,
                &original,
            )
            .unwrap();
            assert!(cursor.owns_checked_buffers());
            let mut position = ResultStreamPosition::default();
            append_result_table_queued(&mut queue, id, cursor, &mut position, None)
                .await
                .unwrap();
            queue.finish_deferred(id, position).await.unwrap();
            assert!(
                acks.complete(&fence, &id.to_string()).is_err(),
                "issuing a queue item grants no ACK coverage"
            );
            assert_eq!(
                pse_operations::testing::pending_result_batch_rows(
                    store,
                    &fence,
                    &id.to_string(),
                    0
                )
                .await
                .unwrap(),
                None,
                "queueing produces no server append effects"
            );
        }
        assert_eq!(queue.requests, 0);
        assert!(queue.items.is_empty());
        assert_eq!(queue.appends.len(), 1);
        assert_eq!(queue.complete.len(), 1);
        assert!(
            operations.pool.reserved() > idle,
            "owned encoded items stay charged while queued"
        );
        store.cancel_study(study).await.unwrap();
        // Empty membership must first drain the incomplete pending relation;
        // cancellation cannot manufacture an ACK for that pending publication.
        let error = store_result_table_queued(
            &mut queue,
            solve_constraints::RELATION_ID,
            super::super::ResultCursor::from_checked(empty),
            None,
        )
        .await
        .unwrap_err();
        assert!(
            matches!(
                &error,
                WorkflowError::Canonical(
                    pse_operations::canonical::CanonicalError::StudyCancellation {
                        run,
                        attempt,
                        generation,
                        ..
                    }
                ) if run == fence.run()
                    && attempt == fence.attempt()
                    && *generation == fence.generation()
            ),
            "the actual append refusal must name the original fence: {error:?}"
        );
        assert!(queue.items.is_empty() && queue.appends.is_empty());
        assert_eq!(
            queue.complete.len(),
            1,
            "failed I/O retains the final-ACK obligation"
        );
        assert_eq!(
            operations.pool.reserved(),
            idle,
            "the failed actual append joins and releases every owned payload"
        );
        // The production wrapper drains once more before returning its original
        // error. This used to manufacture IncompleteResponse from missing ACKs.
        queue.flush().await.unwrap();
        for id in [
            solve_metrics::RELATION_ID,
            fit_parameters::RELATION_ID,
            solve_constraints::RELATION_ID,
        ] {
            assert!(
                acks.complete(&fence, &id.to_string()).is_err(),
                "neither failed flush nor cleanup grants completion coverage"
            );
            assert_eq!(
                pse_operations::testing::pending_result_batch_rows(
                    store,
                    &fence,
                    &id.to_string(),
                    0
                )
                .await
                .unwrap(),
                None,
                "cancelled group appends no partial table payloads"
            );
        }
        drop(queue);
        assert_eq!(operations.pool.reserved(), before);
        let frozen = acks.freeze(&fence).unwrap();
        assert!(
            matches!(
                store
                    .close_acknowledged(&fence, "cancelled-queue-incomplete-close", frozen)
                    .await,
                Err(pse_operations::canonical::CanonicalError::IncompleteResponse)
            ),
            "authentic incomplete coverage still refuses manifest close"
        );
        store.remove_isolated_fixture().await.unwrap();
    }

    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn canonical_append_queue_groups_encoded_schemas_and_acknowledges_final_operations() {
        use pse_relations::generated::{enums::NativeMetricKind, runtime::solve_metrics};
        let runtime = super::super::durable_tests::durable_runtime();
        let super::super::Durability::Durable(operations) = runtime.durability() else {
            panic!("canonical projection fixture requires durable operations")
        };
        let store = operations.store();
        let revision = store
            .edit("append-queue-fixture", None, "source", &[])
            .await
            .unwrap();
        store
            .begin_run(&pse_operations::canonical_execution::RunRequest {
                purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
                admission_study: None,
                admission_point: None,
                key: "append-queue-run".into(),
                revision,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run(
                "append-queue-run",
                "append-queue-claim",
                "worker",
                std::time::Duration::from_secs(60),
            )
            .await
            .unwrap();
        let validation = runtime.validation_context();
        let mut metrics =
            solve_metrics::Builder::with_registry(runtime.registry(), 1, &validation).unwrap();
        metrics
            .push(solve_metrics::Row {
                run_id: pse_operations::mint_id(),
                step: 0,
                namespace: "independent".into(),
                name: "small".into(),
                kind: NativeMetricKind::Real,
                real: Some(-0.0),
                integer: None,
                boolean: None,
                text: None,
                unavailable: None,
            })
            .unwrap();
        let metrics = metrics.finish().unwrap();
        let mut parameters =
            fit_parameters::Builder::with_registry(runtime.registry(), 1, &validation).unwrap();
        parameters
            .push(fit_parameters::Row {
                run_id: pse_operations::mint_id(),
                parameter_id: SemanticId::from_bytes([5; 16]),
                fixed: false,
                value: Some(2.0),
                unit_id: SemanticId::NIL,
                scale: 1.0,
                at_bound: None,
            })
            .unwrap();
        let parameters = parameters.finish().unwrap();
        let empty = solve_constraints::Builder::with_registry(runtime.registry(), 0, &validation)
            .unwrap()
            .finish()
            .unwrap();
        let acks = super::super::durable::PublicationAcks::new(&operations.pool);
        let before = operations.pool.reserved();
        let mut queue = ResultAppendQueue::new(store, &fence, &operations.pool, &acks);
        let idle = operations.pool.reserved();
        assert!(idle > before, "queue control is admitted before allocation");
        let mut original_deadline = None;
        for (id, table) in [
            (solve_metrics::RELATION_ID, metrics),
            (fit_parameters::RELATION_ID, parameters),
        ] {
            let source_reservation =
                MemoryConsumer::new("test:publication-source-owner").register(&operations.pool);
            source_reservation.try_grow(64 * 1024).unwrap();
            let source_owner = pse_columnar::AllocationLease::new(source_reservation);
            let observed_source = Arc::downgrade(&source_owner);
            let table = table
                .retained(&operations.pool, &pse_columnar::CancellationToken::new())
                .unwrap();
            let original = [table.with_export_owner(source_owner)];
            let original_buffer = original[0].batch().column(0).to_data().buffers()[0].as_ptr();
            let cursor = super::super::ResultCursor::from_retained(
                original[0].batch().schema(),
                id,
                &original,
            )
            .unwrap();
            assert!(cursor.owns_checked_buffers());
            let mut position = ResultStreamPosition::default();
            append_result_table_queued(&mut queue, id, cursor, &mut position, None)
                .await
                .unwrap();
            drop(original);
            assert!(
                observed_source.upgrade().is_none(),
                "packing replaces obsolete source-container metadata"
            );
            let packed = &queue.items.last().unwrap().source;
            let bytes = packed
                .owned()
                .expect("packing retains admitted buffer ownership")
                .retained_bytes()
                .unwrap();
            assert!(
                bytes > 0,
                "actual scientific buffers retain their allocation grants"
            );
            if id == fit_parameters::RELATION_ID {
                assert_eq!(
                    fit_parameters::View::from_checked(packed)
                        .unwrap()
                        .row(0)
                        .unwrap()
                        .value,
                    Some(2.0)
                );
            } else {
                assert_eq!(
                    packed.batch().column(0).to_data().buffers()[0].as_ptr(),
                    original_buffer,
                    "unsorted packing shares the original scientific backing"
                );
                assert_eq!(
                    solve_metrics::View::from_checked(packed)
                        .unwrap()
                        .row(0)
                        .unwrap()
                        .real
                        .unwrap()
                        .to_bits(),
                    (-0.0f64).to_bits()
                );
            }
            queue.finish_deferred(id, position).await.unwrap();
            assert!(
                observed_source.upgrade().is_none(),
                "completed encoding releases source ownership before retaining append payloads"
            );
            assert!(acks.complete(&fence, &id.to_string()).is_err());
            if let Some(clock) = original_deadline {
                assert_eq!(
                    queue.append_deadline,
                    Some(clock),
                    "new files do not renew the oldest operation clock"
                );
            } else {
                original_deadline = queue.append_deadline;
            }
            assert!(operations.pool.reserved() > idle);
            assert!(
                operations.pool.reserved() - idle < result_files::MAX_FILE_BYTES,
                "queued payload custody must not retain the three-file encoder allowance"
            );
            assert_eq!(
                pse_operations::testing::pending_result_batch_rows(
                    store,
                    &fence,
                    &id.to_string(),
                    0
                )
                .await
                .unwrap(),
                None,
                "sealing and producer completion do not grant an append ACK"
            );
        }
        assert_eq!(queue.requests, 0);
        assert!(queue.items.is_empty());
        assert_eq!(queue.appends.len(), 2);
        assert_eq!(queue.complete.len(), 2);
        queue.flush().await.unwrap();
        assert_eq!(
            queue.requests, 1,
            "separate homogeneous files share one metadata submission"
        );
        assert!(queue.appends.is_empty() && queue.complete.is_empty());
        assert!(queue.complete.capacity() > 0);
        let metrics_payload = pse_operations::testing::pending_result_payload_coordinates(
            store,
            &fence,
            &solve_metrics::RELATION_ID.to_string(),
            0,
        )
        .await
        .unwrap()
        .unwrap();
        let parameters_payload = pse_operations::testing::pending_result_payload_coordinates(
            store,
            &fence,
            &fit_parameters::RELATION_ID.to_string(),
            0,
        )
        .await
        .unwrap()
        .unwrap();
        assert!(matches!(
            metrics_payload.0,
            pse_operations::canonical_execution::PayloadLocator::Inline { index: 0, .. }
        ));
        assert!(matches!(
            parameters_payload.0,
            pse_operations::canonical_execution::PayloadLocator::Inline { index: 0, .. }
        ));
        assert_ne!(
            metrics_payload.0.payload_identity(),
            parameters_payload.0.payload_identity(),
            "unlike schemas use independent admitted bodies"
        );
        assert_eq!(metrics_payload.1, solve_metrics::RELATION_ID.to_string());
        assert_eq!(
            parameters_payload.1,
            fit_parameters::RELATION_ID.to_string()
        );
        assert_eq!(
            pse_operations::testing::result_payload_counts(store, &fence)
                .await
                .unwrap(),
            (2, 0)
        );
        store_result_table_queued(
            &mut queue,
            solve_constraints::RELATION_ID,
            super::super::ResultCursor::from_checked(empty),
            None,
        )
        .await
        .unwrap();
        assert_eq!(
            queue.requests, 1,
            "empty membership introduces no IPC append"
        );
        assert!(queue.items.is_empty());
        assert_eq!(
            operations.pool.reserved(),
            idle,
            "joined append releases item charges"
        );
        for id in [solve_metrics::RELATION_ID, fit_parameters::RELATION_ID] {
            acks.complete(&fence, &id.to_string()).unwrap();
            assert_eq!(
                pse_operations::testing::pending_result_batch_rows(
                    store,
                    &fence,
                    &id.to_string(),
                    0
                )
                .await
                .unwrap(),
                Some(1)
            );
        }
        acks.complete(&fence, &solve_constraints::RELATION_ID.to_string())
            .unwrap();
        assert_eq!(
            pse_operations::testing::pending_result_batch_rows(
                store,
                &fence,
                &solve_constraints::RELATION_ID.to_string(),
                0
            )
            .await
            .unwrap(),
            None,
            "demanded empty membership has no IPC append"
        );
        queue.yield_optional().await.unwrap();
        assert_eq!(queue.complete.capacity(), 0);
        assert!(queue._owner.is_none());
        drop(queue);
        assert_eq!(operations.pool.reserved(), before);
        let closed = store
            .close_result_ingestion(&fence, "append-queue-close")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        let descriptors =
            pse_operations::canonical_execution::decode_result_descriptors(manifest.row()).unwrap();
        assert_eq!(descriptors.len(), 3);
        for descriptor in descriptors {
            let rows = u64::from(descriptor.name != solve_constraints::RELATION_ID.to_string());
            assert_eq!((descriptor.batch_count, descriptor.row_count), (rows, rows));
        }
        store.remove_isolated_fixture().await.unwrap();
    }
    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn canonical_append_queue_denied_control_preserves_cold_append_and_demand() {
        let runtime = super::super::durable_tests::durable_runtime();
        let super::super::Durability::Durable(operations) = runtime.durability() else {
            panic!("canonical projection fixture requires durable operations")
        };
        let store = operations.store();
        let revision = store
            .edit("cold-append-fixture", None, "source", &[])
            .await
            .unwrap();
        store
            .begin_run(&pse_operations::canonical_execution::RunRequest {
                purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
                admission_study: None,
                admission_point: None,
                key: "cold-append-run".into(),
                revision,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run(
                "cold-append-run",
                "cold-append-claim",
                "worker",
                std::time::Duration::from_secs(60),
            )
            .await
            .unwrap();
        let acks = super::super::durable::PublicationAcks::new(&operations.pool);
        let pool: Arc<dyn MemoryPool> = Arc::new(
            datafusion::execution::memory_pool::GreedyMemoryPool::new(128 << 20),
        );
        // Pressure denies only the optional queue at construction. Once that
        // other borrower leaves, the mandatory cold file encoder has its actual
        // allowance; disabling batching must not fabricate a smaller encoder.
        let producer = MemoryConsumer::new("cold-append-producer").register(&pool);
        producer.try_grow(64 * 1024).unwrap();
        let pressure = MemoryConsumer::new("cold-append-other-borrower").register(&pool);
        pressure
            .try_grow((128 << 20) - 64 * 1024 - 128 * 1024)
            .unwrap();
        let mut queue = ResultAppendQueue::new(store, &fence, &pool, &acks);
        assert!(queue._owner.is_none());
        drop(pressure);
        let id = fit_parameters::RELATION_ID;
        let mut builder = fit_parameters::Builder::with_registry(
            runtime.registry(),
            1,
            &runtime.validation_context(),
        )
        .unwrap();
        builder
            .push(fit_parameters::Row {
                run_id: pse_operations::mint_id(),
                parameter_id: SemanticId::from_bytes([9; 16]),
                fixed: false,
                value: Some(-0.0),
                unit_id: SemanticId::NIL,
                scale: 1.0,
                at_bound: None,
            })
            .unwrap();
        let table = builder.finish().unwrap();
        let name = id.to_string();
        let set = result_set_key(fence.attempt(), &name);
        let key = result_batch_key(fence.attempt(), &set, 0);
        let rows = 1;
        let payload = result_files::encode_result_file(&table.batch().slice(0, rows)).unwrap();
        let _encoded = payload;
        let block = Block {
            key: key.clone(),
            batch: key,
            result_set: set,
            output: name.clone(),
            partition: "0".into(),
            ordinal: 0,
            start: 0,
            end: rows as u64,
            rows: rows as u64,
            columns: table.batch().num_columns() as u64,
            coordinate_min: None,
            coordinate_max: None,
            payload_bytes: 0,
            payload_digest: String::new(),
            statistics: None,
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        };
        acks.expect(&fence, &name).unwrap();
        queue.push(&name, 0, &table, &block, &[]).await.unwrap();
        queue
            .finish(
                id,
                ResultStreamPosition {
                    origin: rows as u64,
                    ordinal: 1,
                },
            )
            .await
            .unwrap();
        assert_eq!(queue.requests, 1);
        assert!(
            queue.items.is_empty() && queue.complete.is_empty(),
            "disabled queue retains no item or copied completion names"
        );
        assert_eq!(pool.reserved(), 64 * 1024);
        acks.complete(&fence, &name).unwrap();
        assert_eq!(
            pse_operations::testing::pending_result_batch_rows(store, &fence, &name, 0)
                .await
                .unwrap(),
            Some(1)
        );
        let failed_id = solve_constraints::RELATION_ID;
        let schema = pse_schema::arrow::relation_schema_ref(
            runtime.registry(),
            runtime.registry().relation_by_id(failed_id).unwrap(),
        )
        .unwrap();
        let refused = super::super::ResultCursor::new(
            schema,
            failed_id,
            1,
            super::super::ResultOrder::Canonical,
            |_| async { Err(Arc::new(contract("producer refused before any row"))) },
        )
        .unwrap();
        assert!(
            store_result_table_queued(&mut queue, failed_id, refused, None)
                .await
                .is_err()
        );
        assert!(acks.complete(&fence, &failed_id.to_string()).is_err());
        let frozen = acks.freeze(&fence).unwrap();
        assert!(
            store
                .close_acknowledged(&fence, "cold-incomplete-close", frozen)
                .await
                .is_err(),
            "refused advertised demand stays incomplete rather than absent"
        );
        store.remove_isolated_fixture().await.unwrap();
    }
    #[cfg(feature = "canonical-tests")]
    #[tokio::test]
    async fn canonical_append_queue_drains_optional_storage_before_allocating_cursor_poll() {
        use pse_relations::generated::{enums::NativeMetricKind, runtime::solve_metrics};
        let runtime = super::super::durable_tests::durable_runtime();
        let super::super::Durability::Durable(operations) = runtime.durability() else {
            panic!("canonical projection fixture requires durable operations")
        };
        let store = operations.store();
        let revision = store
            .edit("allocating-append-fixture", None, "source", &[])
            .await
            .unwrap();
        store
            .begin_run(&pse_operations::canonical_execution::RunRequest {
                purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
                admission_study: None,
                admission_point: None,
                key: "allocating-append-run".into(),
                revision,
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run(
                "allocating-append-run",
                "allocating-append-claim",
                "worker",
                std::time::Duration::from_secs(60),
            )
            .await
            .unwrap();
        let acks = super::super::durable::PublicationAcks::new(&operations.pool);
        let pool: Arc<dyn MemoryPool> = Arc::new(
            datafusion::execution::memory_pool::GreedyMemoryPool::new(128 << 20),
        );
        let validation = runtime.validation_context();
        let mut builder =
            solve_metrics::Builder::with_registry(runtime.registry(), 1, &validation).unwrap();
        builder
            .push(solve_metrics::Row {
                run_id: pse_operations::mint_id(),
                step: 0,
                namespace: "independent".into(),
                name: "queued".into(),
                kind: NativeMetricKind::Real,
                real: Some(-0.0),
                integer: None,
                boolean: None,
                text: None,
                unavailable: None,
            })
            .unwrap();
        let mut queue = ResultAppendQueue::new(store, &fence, &pool, &acks);
        let checked = super::super::ResultCursor::from_checked(builder.finish().unwrap());
        assert!(checked.owns_checked_buffers());
        let mut position = ResultStreamPosition::default();
        append_result_table_queued(
            &mut queue,
            solve_metrics::RELATION_ID,
            checked,
            &mut position,
            None,
        )
        .await
        .unwrap();
        assert_eq!(queue.requests, 0);
        assert!(!queue.items.is_empty());
        assert!(pool.reserved() > 256 * 1024);
        queue
            .finish_deferred(solve_metrics::RELATION_ID, position)
            .await
            .unwrap();
        assert_eq!(queue.complete.len(), 1);
        assert!(queue.complete.capacity() > 0);
        let id = fit_parameters::RELATION_ID;
        let schema = pse_schema::arrow::relation_schema_ref(
            runtime.registry(),
            runtime.registry().relation_by_id(id).unwrap(),
        )
        .unwrap();
        let registry = runtime.registry();
        let producer_pool = pool.clone();
        let polls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed = polls.clone();
        let allocating = super::super::ResultCursor::new(
            schema,
            id,
            1,
            super::super::ResultOrder::Canonical,
            move |request| async move {
                observed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                // This valid cold producer cannot coexist even with the queue
                // control envelope. Its admission and checked row construction
                // occur only on the first poll, and are never replayed.
                let reservation =
                    MemoryConsumer::new("allocating-append-cursor").register(&producer_pool);
                reservation
                    .try_grow((128 << 20) - (64 << 10))
                    .map_err(|error| {
                        Arc::new(WorkflowError::from(pse_engine::EngineError::from(error)))
                    })?;
                let scratch = vec![0_u8; 64 << 10];
                let mut builder = fit_parameters::Builder::with_registry(registry, 1, &validation)
                    .map_err(|error| Arc::new(relation(error)))?;
                builder
                    .push(fit_parameters::Row {
                        run_id: pse_operations::mint_id(),
                        parameter_id: SemanticId::from_bytes([11; 16]),
                        fixed: false,
                        value: Some(3.0),
                        unit_id: SemanticId::NIL,
                        scale: 1.0,
                        at_bound: None,
                    })
                    .map_err(|error| Arc::new(relation(error)))?;
                let table = builder
                    .finish()
                    .map_err(|error| Arc::new(relation(error)))?;
                request.send_checked(table).await?;
                drop(scratch);
                drop(reservation);
                Ok(())
            },
        )
        .unwrap();
        assert!(!allocating.owns_checked_buffers());
        store_result_table_queued(&mut queue, id, allocating, None)
            .await
            .unwrap();
        assert_eq!(polls.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(
            queue.requests, 2,
            "each boundary drains before the unknown next producer poll"
        );
        assert!(queue.items.is_empty() && queue._owner.is_none());
        assert!(queue.appends.is_empty() && queue.complete.is_empty());
        assert_eq!(queue.complete.capacity(), 0);
        assert_eq!(pool.reserved(), 0);
        for id in [solve_metrics::RELATION_ID, fit_parameters::RELATION_ID] {
            acks.complete(&fence, &id.to_string()).unwrap();
            assert_eq!(
                pse_operations::testing::pending_result_batch_rows(
                    store,
                    &fence,
                    &id.to_string(),
                    0
                )
                .await
                .unwrap(),
                Some(1)
            );
        }
        store.remove_isolated_fixture().await.unwrap();
    }
    #[test]
    fn trajectory_partitions_keep_1024_boundaries_prefixes_and_refuse_discontiguous_symbols() {
        let registry = pse_schema::registry().unwrap();
        let validation = validation();
        let run_id = pse_operations::mint_id();
        let mut builder = simulation_samples::Builder::with_registry(registry, 1027, &validation).unwrap();
        for index in 0..1027 {
            builder.push(simulation_samples::Row { run_id, sample: index as i64, time: index as f64,
                symbol_id: SemanticId::from_bytes([if index < 1023 { 1 } else { 2 }; 16]), quantity_id: SemanticId::NIL,
                unit_id: SemanticId::NIL, value: -0. }).unwrap();
        }
        let table = builder.finish().unwrap();
        let full = trajectory_outputs(&table, "block", "set", 50, 0, 1024).unwrap();
        assert_eq!((full[0].start, full[0].end, full[1].start, full[1].end), (50, 1073, 1073, 1074));
        let next = trajectory_outputs(&table, "next", "set", 50, 1024, 3).unwrap();
        assert_eq!((next[0].start, next[0].end), (1074, 1077));
        let view = simulation_samples::View::from_checked(&table).unwrap();
        let ranges = trajectory_ranges(&view, 0, 1024).unwrap();
        let prefix = outputs_from_ranges(&table, &view, &ranges, "block", "set", 50, 1000).unwrap();
        assert_eq!((prefix.len(), prefix[0].start, prefix[0].end), (1, 50, 1050));
        assert_eq!((prefix[0].coordinate_min, prefix[0].coordinate_max), (Some(0.), Some(999.)));
        let mut builder = simulation_samples::Builder::with_registry(registry, 3, &validation).unwrap();
        for (index, symbol) in [1, 2, 1].into_iter().enumerate() {
            builder.push(simulation_samples::Row { run_id, sample: index as i64, time: index as f64,
                symbol_id: SemanticId::from_bytes([symbol; 16]), quantity_id: SemanticId::NIL, unit_id: SemanticId::NIL, value: 0. }).unwrap();
        }
        assert!(trajectory_outputs(&builder.finish().unwrap(), "bad", "set", 0, 0, 3).is_err());
    }
    #[test]
    fn dense_trajectory_block_keeps_multiple_exact_symbol_ranges() {
        let registry = pse_schema::registry().unwrap();
        let validation = validation();
        let run_id = pse_operations::mint_id();
        let symbols = [
            SemanticId::from_bytes([1; 16]),
            SemanticId::from_bytes([2; 16]),
        ];
        let mut builder =
            simulation_samples::Builder::with_registry(registry, 6, &validation).unwrap();
        for symbol_id in symbols {
            for sample in 0..3 {
                builder
                    .push(simulation_samples::Row {
                        run_id,
                        sample,
                        time: sample as f64,
                        symbol_id,
                        quantity_id: SemanticId::NIL,
                        unit_id: SemanticId::NIL,
                        value: if sample == 0 { -0.0 } else { sample as f64 },
                    })
                    .unwrap();
            }
        }
        let table = builder.finish().unwrap();
        let outputs = trajectory_outputs(&table, "block", "set", 100, 0, 6).unwrap();
        assert_eq!(outputs.len(), 2);
        assert_eq!((outputs[0].start, outputs[0].end), (100, 103));
        assert_eq!((outputs[1].start, outputs[1].end), (103, 106));
        assert!(outputs.windows(2).all(|pair| pair[0].key < pair[1].key));
        assert!(outputs.iter().all(|output| output.batch == "block"
            && output.coordinate_min == Some(0.0)
            && output.coordinate_max == Some(2.0)));
        let count = 6;
        let payload = result_files::encode_result_file(&table.batch().slice(0, count)).unwrap();
        assert_eq!(count, 6);
        assert!(!payload.is_empty());
        let partial = trajectory_outputs(&table, "next", "set", 100, 2, 3).unwrap();
        assert_eq!(
            (partial[0].start, partial[0].end, partial[0].coordinate_min),
            (102, 103, Some(2.0))
        );
        assert_eq!(
            (partial[1].start, partial[1].end, partial[1].coordinate_max),
            (103, 105, Some(1.0))
        );
        assert!(trajectory_outputs(&table, "block", "set", u64::MAX, 0, 6).is_err());
    }
}

#[cfg(test)]
mod live_append_control_unit {
    use super::*;
    #[test]
    fn live_append_control_reuses_one_bounded_grant_and_releases_final_charge() {
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(512 * 1024));
        let mut control = append_queue_control(&pool, None);
        let original = Arc::downgrade(control.as_ref().unwrap());
        for _ in 0..32 {
            let prior = control.as_ref().unwrap().clone();
            control = append_queue_control(&pool, control.take());
            assert!(Arc::ptr_eq(&prior, control.as_ref().unwrap()));
            assert_eq!(pool.reserved(), 256 * 1024);
        }
        drop(control);
        assert!(original.upgrade().is_none());
        assert_eq!(pool.reserved(), 0);
        let tiny: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1024));
        assert!(
            append_queue_control(&tiny, None).is_none(),
            "optional queue retention refusal preserves synchronous fallback"
        );
        assert_eq!(tiny.reserved(), 0);
    }
}
