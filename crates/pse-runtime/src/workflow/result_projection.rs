// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native scalar indexes are derived from declared scientific fields. The
//! original admitted Arrow row remains the scientific result authority.

use super::{WorkflowError, contract, relation, result_blocks};
use pse_columnar::{MemoryConsumer, MemoryPool};
use pse_ids::SemanticId;
use pse_model::generated::runtime::{
    canonical_result_block_outputs::Row as Output, canonical_result_blocks::Row as Block,
    canonical_result_rows::Row as RowLocator,
};
use pse_operations::{
    canonical::CanonicalStore,
    canonical_execution::{
        AttemptFence, ResultBlockAppend, ResultPayload, result_batch_key, result_set_key,
    },
};
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::runtime::{
        fit_observations, fit_parameters, simulation_samples, solve_constraints, solve_variables,
    },
};
use std::sync::Arc;

fn indexed_scalar(id: SemanticId) -> bool {
    [
        solve_variables::RELATION_ID,
        solve_constraints::RELATION_ID,
        fit_parameters::RELATION_ID,
        fit_observations::RELATION_ID,
    ]
    .contains(&id)
}
pub(super) fn check_output_field(id: SemanticId, field: &str) -> Result<(), WorkflowError> {
    let fields: &[&str] = if id == solve_variables::RELATION_ID {
        &[
            "value",
            "lower",
            "upper",
            "lower_violation",
            "upper_violation",
            "tolerance",
            "lower_dual",
            "upper_dual",
            "reduced_cost",
            "stationarity",
        ]
    } else if id == solve_constraints::RELATION_ID {
        &[
            "value",
            "lower",
            "upper",
            "equality_residual",
            "lower_violation",
            "upper_violation",
            "tolerance",
            "dual",
        ]
    } else if id == fit_parameters::RELATION_ID {
        &["value", "scale"]
    } else if id == fit_observations::RELATION_ID {
        &[
            "prediction",
            "residual",
            "standardized_residual",
            "objective_contribution",
        ]
    } else if id == simulation_samples::RELATION_ID {
        &["value"]
    } else {
        &[]
    };
    if !fields.contains(&field) {
        return Err(contract(
            "scientific output field has no declared native index",
        ));
    }
    Ok(())
}

/// Reconstruct the full typed key from the original columns, independently of
/// any selected value field or numeric access predicate.
pub(super) fn selected_scalar_row(
    table: &FieldCheckedBatch,
    attempt: &str,
    set: &str,
    batch: &str,
    index: usize,
    origin: u64,
) -> Result<RowLocator, WorkflowError> {
    use pse_relations::columnar::ArrowValue;
    let relation_id = table.relation_id();
    if index >= table.batch().num_rows() {
        return Err(contract("selected scalar row outside original block"));
    }
    let (run_id, owner, step, experiment_id) = if relation_id == solve_variables::RELATION_ID {
        let view = solve_variables::View::from_checked(table).map_err(relation)?;
        (
            ArrowValue::read(view.run_id_column(), index).map_err(relation)?,
            ArrowValue::read(view.symbol_id_column(), index).map_err(relation)?,
            Some(view.step_column().value(index)),
            None,
        )
    } else if relation_id == solve_constraints::RELATION_ID {
        let view = solve_constraints::View::from_checked(table).map_err(relation)?;
        (
            ArrowValue::read(view.run_id_column(), index).map_err(relation)?,
            ArrowValue::read(view.row_id_column(), index).map_err(relation)?,
            Some(view.step_column().value(index)),
            None,
        )
    } else if relation_id == fit_parameters::RELATION_ID {
        let view = fit_parameters::View::from_checked(table).map_err(relation)?;
        (
            ArrowValue::read(view.run_id_column(), index).map_err(relation)?,
            ArrowValue::read(view.parameter_id_column(), index).map_err(relation)?,
            None,
            None,
        )
    } else if relation_id == fit_observations::RELATION_ID {
        let view = fit_observations::View::from_checked(table).map_err(relation)?;
        (
            ArrowValue::read(view.run_id_column(), index).map_err(relation)?,
            ArrowValue::read(view.observation_id_column(), index).map_err(relation)?,
            None,
            Some(ArrowValue::read(view.experiment_id_column(), index).map_err(relation)?),
        )
    } else {
        return Err(contract("relation has no scalar row locator"));
    };
    let partition = if let Some(step) = step {
        format!("step:{step}")
    } else if let Some(experiment) = experiment_id {
        format!("experiment:{experiment}")
    } else {
        "0".into()
    };
    let row = origin
        .checked_add(index as u64)
        .ok_or_else(|| contract("scientific row coordinate overflow"))?;
    let locator = RowLocator {
        key: format!("{batch}:{row}"),
        result_set: set.into(),
        attempt: attempt.into(),
        batch: batch.into(),
        relation_id,
        run_id,
        owner,
        step,
        experiment_id,
        output: format!("{relation_id}:{owner}"),
        partition,
        row,
        offset: index as u64,
        interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
    };
    pse_operations::canonical_results::validate_result_row(&locator)?;
    Ok(locator)
}
/// Validate every locator premise against its exact original decoded row.
pub(super) fn verify_scalar_row(
    table: &FieldCheckedBatch,
    locator: &RowLocator,
    attempt: &str,
    set: &str,
    batch: &str,
    origin: u64,
) -> Result<usize, WorkflowError> {
    let index = locator
        .row
        .checked_sub(origin)
        .and_then(|row| usize::try_from(row).ok())
        .ok_or_else(|| contract("selected scalar row outside original block"))?;
    if selected_scalar_row(table, attempt, set, batch, index, origin)? != *locator {
        return Err(contract(
            "native scalar locator differs from original scientific row",
        ));
    }
    Ok(index)
}
pub(super) fn scalar_rows_at(
    table: &FieldCheckedBatch,
    attempt: &str,
    set: &str,
    batch: &str,
    start: usize,
    count: usize,
    origin: u64,
) -> Result<Vec<RowLocator>, WorkflowError> {
    if !indexed_scalar(table.relation_id()) {
        return Ok(Vec::new());
    }
    let end = start
        .checked_add(count)
        .filter(|end| *end <= table.batch().num_rows())
        .ok_or_else(|| contract("scalar row window outside original batch"))?;
    let mut rows = (start..end)
        .map(|index| selected_scalar_row(table, attempt, set, batch, index, origin))
        .collect::<Result<Vec<_>, _>>()?;
    rows.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(rows)
}

/// Arrow kernels evaluate predicates on the decoded requested field, while take
/// gathers original columns without normalizing their scientific bits.
pub(super) fn scalar_predicate(
    table: &FieldCheckedBatch,
    field: &str,
    minimum: Option<f64>,
    maximum: Option<f64>,
    missing: Option<bool>,
) -> Result<datafusion::arrow::array::BooleanArray, WorkflowError> {
    use datafusion::arrow::{
        array::{Array, BooleanArray, Float64Array, UInt8Array},
        compute::{
            and_kleene, is_not_null, is_null,
            kernels::cmp::{eq, gt_eq, lt_eq},
        },
    };
    check_output_field(table.relation_id(), field)?;
    if minimum.is_some_and(|value| !value.is_finite())
        || maximum.is_some_and(|value| !value.is_finite())
        || minimum
            .zip(maximum)
            .is_some_and(|(lower, upper)| lower > upper)
        || (missing == Some(true) && (minimum.is_some() || maximum.is_some()))
    {
        return Err(contract("invalid scalar access predicate"));
    }
    let values = table
        .batch()
        .column_by_name(field)
        .and_then(|column| column.as_any().downcast_ref::<Float64Array>())
        .ok_or_else(|| contract("declared scientific scalar column absent"))?;
    let kernel = |error| WorkflowError::from(result_blocks::ResultBlockError::from(error));
    if missing == Some(true) {
        return is_null(values).map_err(kernel);
    }
    let mut mask = if missing == Some(false) || minimum.is_some() || maximum.is_some() {
        let finite: UInt8Array = values.unary(|value| u8::from(value.is_finite()));
        let finite = eq(&finite, &UInt8Array::new_scalar(1)).map_err(kernel)?;
        and_kleene(&is_not_null(values).map_err(kernel)?, &finite).map_err(kernel)?
    } else {
        BooleanArray::from(vec![true; values.len()])
    };
    if minimum.is_some() || maximum.is_some() {
        // Arrow totalOrder distinguishes signed zeros. Numeric access treats them
        // equally, using a temporary comparison array while retaining original bits.
        let comparison: Float64Array = values.unary(|value| if value == 0.0 { 0.0 } else { value });
        if let Some(lower) = minimum {
            mask = and_kleene(
                &mask,
                &gt_eq(
                    &comparison,
                    &Float64Array::new_scalar(if lower == 0.0 { 0.0 } else { lower }),
                )
                .map_err(kernel)?,
            )
            .map_err(kernel)?;
        }
        if let Some(upper) = maximum {
            mask = and_kleene(
                &mask,
                &lt_eq(
                    &comparison,
                    &Float64Array::new_scalar(if upper == 0.0 { 0.0 } else { upper }),
                )
                .map_err(kernel)?,
            )
            .map_err(kernel)?;
        }
    }
    Ok(mask)
}

/// Ingest the sole admitted scientific IPC payload and its derived scalar index
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

/// Sole serial append owner shared by whole relation exports. Encoded items
/// retain their pre-allocation grants until the one transaction/settlement joins.
/// Publication coverage advances only from the returned actual opaque ACKs.
pub(super) struct ResultAppendQueue<'a> {
    store: &'a CanonicalStore,
    fence: &'a AttemptFence,
    pool: &'a Arc<dyn MemoryPool>,
    acknowledgments: &'a super::durable::PublicationAcks,
    items: Vec<ResultBlockAppend>,
    extent: usize,
    deadline: Option<std::time::Instant>,
    complete: Vec<String>,
    _owner: Option<Arc<pse_columnar::AllocationLease>>,
    #[cfg(test)]
    requests: usize,
    #[cfg(test)]
    projection_retries: usize,
}
impl<'a> ResultAppendQueue<'a> {
    pub(super) fn new(
        store: &'a CanonicalStore,
        fence: &'a AttemptFence,
        pool: &'a Arc<dyn MemoryPool>,
        acknowledgments: &'a super::durable::PublicationAcks,
    ) -> Self {
        let reservation = MemoryConsumer::new("canonical:append-queue-control").register(pool);
        // Maximum advertised set population, copied names and queue/control Arc
        // envelopes. Per-item encoded bytes have separate actual-sized grants.
        // Batching is optional: a denied queue envelope preserves the original
        // synchronous append path, with no retained item or completion vectors.
        let owner = reservation
            .try_grow(256 * 1024)
            .ok()
            .map(|()| pse_columnar::AllocationLease::new(reservation));
        Self {
            store,
            fence,
            pool,
            acknowledgments,
            items: Vec::new(),
            extent: 4096,
            deadline: None,
            complete: Vec::new(),
            _owner: owner,
            #[cfg(test)]
            requests: 0,
            #[cfg(test)]
            projection_retries: 0,
        }
    }
    async fn push(
        &mut self,
        name: &str,
        ordinal: u64,
        payload: ResultPayload,
        block: &Block,
        rows: &[RowLocator],
        outputs: &[Output],
    ) -> Result<(), WorkflowError> {
        let offered = std::time::Instant::now() + pse_operations::canonical::REQUEST_TIMEOUT;
        if self._owner.is_none() {
            return self
                .append_individual(name, ordinal, payload, block, rows, outputs, offered)
                .await;
        }
        let metadata =
            rows.iter()
                .try_fold(4096usize, |bytes, row| -> Result<_, WorkflowError> {
                    bytes
                        .checked_add(
                            pse_operations::canonical_execution::result_row_metadata_extent(row)?,
                        )
                        .ok_or_else(|| contract("append queue metadata extent"))
                })?;
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
                        .ok_or_else(|| contract("append queue metadata extent"))
                })?;
        let extent = payload
            .as_slice()
            .len()
            .checked_add(metadata)
            .and_then(|bytes| bytes.checked_mul(16))
            .and_then(|bytes| bytes.checked_add(4096))
            .ok_or_else(|| contract("append queue copy extent"))?;
        let reservation = MemoryConsumer::new("canonical:owned-append-item").register(self.pool);
        if reservation.try_grow(extent).is_err() {
            self.flush().await?;
            if reservation.try_grow(extent).is_err() {
                // Batching is optional. The producer's existing encoder/metadata
                // grant still owns this synchronous individual append and drain.
                return self
                    .append_individual(name, ordinal, payload, block, rows, outputs, offered)
                    .await;
            }
        }
        let owner = pse_columnar::AllocationLease::new(reservation);
        let item = ResultBlockAppend::new(
            self.fence,
            &format!("table:{}:{}", self.fence.attempt(), block.key),
            name,
            ordinal,
            payload,
            block.rows,
            block,
            rows,
            outputs,
            owner,
        )?;
        if self
            .extent
            .checked_add(item.encoded_extent())
            .is_none_or(|bytes| bytes > pse_operations::canonical::PAYLOAD_BYTES)
            || self.items.len() == pse_operations::generated::surreal::RESULT_INDEX_RECORDS
        {
            self.flush().await?;
        }
        self.extent = self
            .extent
            .checked_add(item.encoded_extent())
            .ok_or_else(|| contract("append group extent"))?;
        if self.extent > pse_operations::canonical::PAYLOAD_BYTES {
            return Err(contract("one append exceeds group envelope"));
        }
        if self.items.is_empty() {
            self.deadline = Some(offered);
        }
        self.items.push(item);
        Ok(())
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "one exact append carries its independent table position, payload, block indexes, outputs and original deadline"
    )]
    async fn append_individual(
        &mut self,
        name: &str,
        ordinal: u64,
        payload: ResultPayload,
        block: &Block,
        rows: &[RowLocator],
        outputs: &[Output],
        deadline: std::time::Instant,
    ) -> Result<(), WorkflowError> {
        let ack = pse_operations::canonical::operation_scope(
            deadline,
            self.store.append_result_block_indexes(
                self.fence,
                &format!("table:{}:{}", self.fence.attempt(), block.key),
                name,
                ordinal,
                payload,
                block.rows,
                block,
                rows,
                outputs,
            ),
        )
        .await?;
        #[cfg(test)]
        {
            self.requests += 1;
        }
        self.acknowledgments.append(self.fence, ack)
    }
    pub(super) async fn flush(&mut self) -> Result<(), WorkflowError> {
        // Producer exhaustion belongs to this drained group. If append or ACK
        // validation fails, a later cleanup flush must not complete these sets
        // without their receipts and replace the original refusal.
        let complete = std::mem::take(&mut self.complete);
        if !self.items.is_empty() {
            let items = std::mem::take(&mut self.items);
            let deadline = self
                .deadline
                .take()
                .ok_or_else(|| contract("append group original clock absent"))?;
            let acks = pse_operations::canonical::operation_scope(
                deadline,
                self.store.append_result_blocks(self.fence, items),
            )
            .await?;
            #[cfg(test)]
            {
                self.requests += 1;
            }
            for ack in acks {
                self.acknowledgments.append(self.fence, ack)?;
            }
            self.extent = 4096;
        }
        for name in complete {
            self.acknowledgments.complete(self.fence, &name)?;
        }
        Ok(())
    }
    pub(super) async fn yield_optional(&mut self) -> Result<(), WorkflowError> {
        // Join and publish authentic earlier ACKs before releasing optional
        // storage. A denied original encoder reservation may then retry cold.
        self.flush().await?;
        self._owner = None;
        Ok(())
    }
    pub(super) fn has_optional_retention(&self) -> bool {
        self._owner.is_some() || !self.items.is_empty()
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
    async fn finish(
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
            self.acknowledgments.complete(self.fence, &name)?;
        } else {
            if self.complete.len() == 256 {
                return Err(contract("append queue completed set extent"));
            }
            self.complete.push(name);
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
    queue.finish(id, position).await
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
    table: &FieldCheckedBatch,
) -> Result<(), WorkflowError> {
    let id = pse_relations::generated::runtime::live_simulation_samples::RELATION_ID;
    if table.relation_id() != id {
        return Err(contract("live sample relation differs from declared producer"));
    }
    acknowledgments.expect(fence, &id.to_string())?;
    let end = position.origin.checked_add(table.batch().num_rows() as u64)
        .ok_or_else(|| contract("live sample row extent"))?;
    let mut queue = ResultAppendQueue::new(store, fence, pool, acknowledgments);
    store_result_chunk(&mut queue, id, table, position.origin, &mut position.ordinal).await?;
    queue.flush().await?;
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
    let reservation = MemoryConsumer::new("canonical:result-scalar-index").register(pool);
    // Live prepared locators, encoded metadata, Arrow writer/protobuf scratch and
    // the outgoing payload are independent of either encoded admission ceiling.
    let name = id.to_string();
    let set = result_set_key(fence.attempt(), &name);
    let indexed = usize::from(indexed_scalar(id));
    let window = if indexed != 0 {
        pse_operations::generated::surreal::RESULT_INDEX_RECORDS.min(1024)
    } else {
        32768
    };
    let scratch = window * indexed * (size_of::<RowLocator>() + 1024)
        + window * size_of::<usize>()
        + 16 * pse_operations::canonical_execution::EXECUTION_METADATA_BYTES
        + 4 * result_blocks::RESULT_BLOCK_BYTES;
    if reservation.try_grow(scratch).is_err() {
        queue.yield_optional().await?;
        reservation
            .try_grow(scratch)
            .map_err(pse_engine::EngineError::from)?;
    }
    let encoder_owner = pse_columnar::AllocationLease::new(reservation);
    for base in (0..table.batch().num_rows()).step_by(window) {
        let count = window.min(table.batch().num_rows() - base);
        let initial_key = result_batch_key(fence.attempt(), &set, *ordinal);
        let prepared = scalar_rows_at(
            table,
            fence.attempt(),
            &set,
            &initial_key,
            base,
            count,
            origin,
        )?;
        let mut extents = vec![0_usize; count];
        for locator in &prepared {
            let extent = pse_operations::canonical_execution::result_row_metadata_extent(locator)?;
            extents[(locator.row - origin) as usize - base] += extent;
        }
        let mut offset = 0;
        while offset < count {
            let current = *ordinal;
            let key = result_batch_key(fence.attempt(), &set, current);
            // Descriptor envelope allowance includes maximum encoded actual
            // identity lengths below. Row extents are prepared once per window.
            let rows = metadata_prefix(&extents[offset..])?;
            let (payload, rows) =
                result_blocks::encode_result_prefix(table.batch(), base + offset, rows)?;
            let payload = ResultPayload::new_owned(payload, encoder_owner.clone())?;
            let start = origin
                .checked_add((base + offset) as u64)
                .ok_or_else(|| contract("result global row overflow"))?;
            let locators = prepared
                .iter()
                .filter(|locator| locator.row >= start && locator.row < start + rows as u64)
                .map(|locator| {
                    let mut locator = locator.clone();
                    locator.key = format!("{key}{}", &locator.key[initial_key.len()..]);
                    locator.batch = key.clone();
                    locator.offset = locator.row - start;
                    locator
                })
                .collect::<Vec<_>>();
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
                payload_bytes: payload.as_slice().len() as u64,
                payload_digest: payload.digest().to_owned(),
                interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
            };
            queue
                .push(&name, current, payload, &block, &locators, &[])
                .await?;
            *ordinal += 1;
            offset += rows;
        }
    }
    Ok(())
}

fn metadata_prefix(extents: &[usize]) -> Result<usize, WorkflowError> {
    // Runtime result identities are fixed-width hashes, relation names are at
    // most128 bytes, partition is "0", and descriptor scalars are fixed-width.
    // This conservative envelope covers those fields and array/map headers.
    let mut metadata = 4096_usize;
    let mut rows = 0;
    for extent in extents {
        if metadata.saturating_add(*extent)
            > pse_operations::canonical_execution::EXECUTION_METADATA_BYTES
        {
            break;
        }
        metadata += extent;
        rows += 1;
    }
    if rows == 0 {
        return Err(contract(
            "single result row index exceeds metadata admission",
        ));
    }
    Ok(rows)
}

/// Original contiguous symbol ranges within one payload-shaped block. Canonical
/// trajectory projection is symbol-major; each symbol has one exact range here.
pub(super) fn trajectory_outputs(
    table: &FieldCheckedBatch,
    key: &str,
    set: &str,
    origin: u64,
    base: usize,
    rows: usize,
) -> Result<Vec<Output>, WorkflowError> {
    let view = simulation_samples::View::from_checked(table).map_err(relation)?;
    let end = base
        .checked_add(rows)
        .filter(|end| *end <= view.len())
        .ok_or_else(|| contract("trajectory output range outside original rows"))?;
    let mut outputs = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut position = base;
    while position < end {
        let symbol = view.row(position).map_err(relation)?.symbol_id;
        if !seen.insert(symbol) {
            return Err(contract("canonical trajectory symbol is not contiguous"));
        }
        let start = position;
        let (mut min, mut max) = (f64::INFINITY, f64::NEG_INFINITY);
        while position < end && view.row(position).map_err(relation)?.symbol_id == symbol {
            let time = view.row(position).map_err(relation)?.time;
            min = min.min(time);
            max = max.max(time);
            position += 1;
        }
        outputs.push(Output {
            key: format!("{key}:value:{:04}", outputs.len()),
            batch: key.into(),
            result_set: set.into(),
            output: format!("{}:{symbol}:value", table.relation_id()),
            partition: "0".into(),
            start: origin
                .checked_add(start as u64)
                .ok_or_else(|| contract("trajectory row extent"))?,
            end: origin
                .checked_add(position as u64)
                .ok_or_else(|| contract("trajectory row extent"))?,
            coordinate_min: Some(min),
            coordinate_max: Some(max),
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        });
    }
    Ok(outputs)
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
    let scratch = 4 * result_blocks::RESULT_BLOCK_BYTES + 1024 * (size_of::<Output>() + 1024);
    if owner.try_grow(scratch).is_err() {
        queue.yield_optional().await?;
        owner
            .try_grow(scratch)
            .map_err(pse_engine::EngineError::from)?;
    }
    let encoder_owner = pse_columnar::AllocationLease::new(owner);
    let mut base = 0;
    while base < table.batch().num_rows() {
        let key = result_batch_key(fence.attempt(), &set, *ordinal);
        let count = (table.batch().num_rows() - base).min(1024);
        let candidates = trajectory_outputs(table, &key, &set, origin, base, count)?;
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
        let (payload, rows) = result_blocks::encode_result_prefix(table.batch(), base, count)?;
        let payload = ResultPayload::new_owned(payload, encoder_owner.clone())?;
        let outputs = trajectory_outputs(table, &key, &set, origin, base, rows)?;
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
            payload_bytes: payload.as_slice().len() as u64,
            payload_digest: payload.digest().to_owned(),
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        };
        queue
            .push(&name, *ordinal, payload, &block, &[], &outputs)
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
    async fn canonical_append_queue_groups_actual_run_projections_and_retries_cold_memory() {
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
            .prepare_analysis(&analysis, &crate::CancelSource::new())
            .await
            .unwrap();
        let result = prepared.start().unwrap().wait().await.unwrap();
        assert!(result.usable(), "{:?}", result.report());
        let super::super::Durability::Durable(operations) = runtime.durability() else {
            unreachable!()
        };
        let store = operations.store();
        let revision = store
            .edit("actual-projection", None, "immutable original result", &[])
            .await
            .unwrap();
        store
            .begin_run(&pse_operations::canonical_execution::RunRequest {
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
            2,
            "real allocating RunResult projections group across relations"
        );
        assert_eq!(queue.requests, 0);
        queue.flush().await.unwrap();
        assert_eq!(queue.requests, 1);
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
        assert!(!queue.items.is_empty());
        let datafusion::execution::memory_pool::MemoryLimit::Finite(limit) =
            operations.pool.memory_limit()
        else {
            unreachable!()
        };
        let pressure =
            MemoryConsumer::new("test:typed-projection-cold-pressure").register(&operations.pool);
        pressure
            .try_grow(limit - operations.pool.reserved() - 16)
            .unwrap();
        store_run_result_table_queued(&mut queue, &result, run_lineage::RELATION_ID, None, None)
            .await
            .unwrap();
        assert_eq!(
            queue.projection_retries, 1,
            "actual original projection memory failure gets exactly one drained cold retry"
        );
        drop(pressure);
        queue.flush().await.unwrap();
        assert_eq!(
            queue.requests, 2,
            "large optional batch joins before cold original projection"
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
            Some(1)
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
            unreachable!()
        };
        let store = operations.store();
        let revision = store
            .edit("cancelled-append-queue-fixture", None, "source", &[])
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
        let study = "cancelled-append-queue-study";
        let occurrence = OccurrenceKey(0);
        store
            .create_study(
                study,
                &request("cancelled-append-queue-summary"),
                &[9],
                &[NewOccurrence {
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
        for (id, table) in [
            (solve_metrics::RELATION_ID, metrics),
            (fit_parameters::RELATION_ID, parameters),
        ] {
            let original = [table];
            let cursor = super::super::ResultCursor::from_retained(
                original[0].batch().schema(),
                id,
                &original,
            )
            .unwrap();
            assert!(cursor.owns_checked_buffers());
            store_result_table_queued(&mut queue, id, cursor, None)
                .await
                .unwrap();
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
        assert_eq!(queue.items.len(), 2);
        assert!(
            operations.pool.reserved() > idle,
            "owned encoded items stay charged while queued"
        );
        store.cancel_study(study).await.unwrap();
        // An empty relation first drains the completed nonempty relations, just
        // as the production typed export does when it reaches empty membership.
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
        assert!(queue.items.is_empty() && queue.complete.is_empty());
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
    async fn canonical_append_queue_packs_relations_and_flushes_before_empty_membership() {
        use pse_relations::generated::{enums::NativeMetricKind, runtime::solve_metrics};
        let runtime = super::super::durable_tests::durable_runtime();
        let super::super::Durability::Durable(operations) = runtime.durability() else {
            unreachable!()
        };
        let store = operations.store();
        let revision = store
            .edit("append-queue-fixture", None, "source", &[])
            .await
            .unwrap();
        store
            .begin_run(&pse_operations::canonical_execution::RunRequest {
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
        for (id, table) in [
            (solve_metrics::RELATION_ID, metrics),
            (fit_parameters::RELATION_ID, parameters),
        ] {
            let original = [table];
            let cursor = super::super::ResultCursor::from_retained(
                original[0].batch().schema(),
                id,
                &original,
            )
            .unwrap();
            assert!(cursor.owns_checked_buffers());
            store_result_table_queued(&mut queue, id, cursor, None)
                .await
                .unwrap();
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
        assert_eq!(queue.items.len(), 2);
        assert!(
            operations.pool.reserved() > idle,
            "owned encoded items stay charged while queued"
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
            "two small relations share one append submission"
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
            unreachable!()
        };
        let store = operations.store();
        let revision = store
            .edit("cold-append-fixture", None, "source", &[])
            .await
            .unwrap();
        store
            .begin_run(&pse_operations::canonical_execution::RunRequest {
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
            datafusion::execution::memory_pool::GreedyMemoryPool::new(128 * 1024),
        );
        // This real producer-owned payload/encoder allowance fits. An optional
        // 256KiB queue cannot fit and must not add a new admission requirement.
        let producer = MemoryConsumer::new("cold-append-producer").register(&pool);
        producer.try_grow(64 * 1024).unwrap();
        let mut queue = ResultAppendQueue::new(store, &fence, &pool, &acks);
        assert!(queue._owner.is_none());
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
        let (payload, rows) = result_blocks::encode_result_prefix(table.batch(), 0, 1).unwrap();
        let payload = ResultPayload::new(payload).unwrap();
        let locators = scalar_rows_at(&table, fence.attempt(), &set, &key, 0, rows, 0).unwrap();
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
            payload_bytes: payload.as_slice().len() as u64,
            payload_digest: payload.digest().into(),
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        };
        acks.expect(&fence, &name).unwrap();
        queue
            .push(&name, 0, payload, &block, &locators, &[])
            .await
            .unwrap();
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
            unreachable!()
        };
        let store = operations.store();
        let revision = store
            .edit("allocating-append-fixture", None, "source", &[])
            .await
            .unwrap();
        store
            .begin_run(&pse_operations::canonical_execution::RunRequest {
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
            datafusion::execution::memory_pool::GreedyMemoryPool::new(8 << 20),
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
        store_result_table_queued(&mut queue, solve_metrics::RELATION_ID, checked, None)
            .await
            .unwrap();
        assert_eq!(queue.requests, 0);
        assert!(!queue.items.is_empty());
        assert!(pool.reserved() > 256 * 1024);
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
                    .try_grow((8 << 20) - (64 << 10))
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
        let (payload, count) = result_blocks::encode_result_prefix(table.batch(), 0, 6).unwrap();
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

    #[test]
    fn scalar_row_locators_are_field_independent_and_predicates_preserve_original_values() {
        let registry = pse_schema::registry().unwrap();
        let validation = validation();
        let mut builder = fit_parameters::Builder::with_registry(registry, 6, &validation).unwrap();
        let parameter = SemanticId::from_bytes([1; 16]);
        // Scientific relations admit finite values only. Preserve an arbitrary
        // admitted IEEE payload as well as both zero signs through projection.
        let precise = f64::from_bits(0x4008_0000_0000_0042);
        for value in [
            Some(-0.0),
            Some(0.0),
            None,
            Some(2.0),
            Some(precise),
            Some(f64::MAX),
        ] {
            builder
                .push(fit_parameters::Row {
                    run_id: pse_operations::mint_id(),
                    parameter_id: parameter,
                    fixed: false,
                    value,
                    unit_id: SemanticId::NIL,
                    scale: 2.0,
                    at_bound: None,
                })
                .unwrap();
        }
        let table = builder.finish().unwrap();
        let locators = scalar_rows_at(&table, "attempt", "set", "block", 0, 6, 10000).unwrap();
        assert_eq!(
            locators.len(),
            6,
            "value and scale share one scientific row locator"
        );
        let zero = selected_scalar_row(&table, "attempt", "set", "block", 0, 10000).unwrap();
        assert_eq!(zero.key, "block:10000");
        assert_eq!(
            zero.output,
            format!("{}:{parameter}", fit_parameters::RELATION_ID)
        );
        assert_eq!(zero.partition, "0");
        assert_eq!(zero.offset, 0);
        assert_eq!(
            selected(&scalar_predicate(&table, "value", None, None, None).unwrap()),
            vec![0, 1, 2, 3, 4, 5]
        );
        assert_eq!(
            selected(&scalar_predicate(&table, "value", None, None, Some(true)).unwrap()),
            vec![2]
        );
        assert_eq!(
            selected(&scalar_predicate(&table, "value", None, None, Some(false)).unwrap()),
            vec![0, 1, 3, 4, 5]
        );
        assert_eq!(
            selected(&scalar_predicate(&table, "scale", Some(2.0), Some(2.0), None).unwrap()),
            vec![0, 1, 2, 3, 4, 5]
        );
        let mask = scalar_predicate(&table, "value", Some(0.0), Some(-0.0), None).unwrap();
        assert_eq!(selected(&mask), vec![0, 1]);
        let gathered = filter_record_batch(table.batch(), &mask).unwrap();
        let values = gathered
            .column_by_name("value")
            .unwrap()
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(values.value(0).to_bits(), (-0.0f64).to_bits());
        assert_eq!(values.value(1).to_bits(), 0.0f64.to_bits());
        let original = table
            .batch()
            .column_by_name("value")
            .unwrap()
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(original.value(4).to_bits(), precise.to_bits());
        assert_eq!(original.value(5).to_bits(), f64::MAX.to_bits());
        assert!(original.is_null(2));
        assert!(scalar_predicate(&table, "fixed", None, None, None).is_err());
        assert!(scalar_predicate(&table, "value", Some(f64::NAN), None, None).is_err());
        assert!(scalar_predicate(&table, "value", Some(3.0), Some(2.0), None).is_err());
        assert!(scalar_predicate(&table, "value", Some(0.0), None, Some(true)).is_err());
        assert!(selected_scalar_row(&table, "attempt", "set", "block", 6, 0).is_err());
        assert!(selected_scalar_row(&table, "attempt", "set", "block", 1, u64::MAX).is_err());
        let mut forged = zero.clone();
        forged.run_id = pse_operations::mint_id();
        assert!(pse_operations::canonical_results::validate_result_row(&forged).is_ok());
        assert!(
            verify_scalar_row(&table, &forged, "attempt", "set", "block", 10000).is_err(),
            "shape-valid foreign keys are refused against original decoded columns"
        );
        assert_eq!(
            verify_scalar_row(&table, &zero, "attempt", "set", "block", 10000).unwrap(),
            0
        );
        for forged in [
            RowLocator {
                attempt: "foreign".into(),
                ..zero.clone()
            },
            RowLocator {
                result_set: "foreign".into(),
                ..zero.clone()
            },
            RowLocator {
                offset: 1,
                ..zero.clone()
            },
            RowLocator {
                owner: SemanticId::NIL,
                ..zero.clone()
            },
        ] {
            assert!(verify_scalar_row(&table, &forged, "attempt", "set", "block", 10000).is_err());
        }
    }

    #[test]
    fn scalar_row_locators_reconstruct_solve_keys_without_numeric_roundtrip_and_fit_experiment() {
        use pse_model::generated::enums::DualQualification;
        let registry = pse_schema::registry().unwrap();
        let validation = validation();
        let run_id = pse_operations::mint_id();
        let owner = SemanticId::from_bytes([7; 16]);
        // Actual scientific steps have the registry's bounded ordinal domain.
        let step = i64::from(u32::MAX);
        let mut variables =
            solve_variables::Builder::with_registry(registry, 1, &validation).unwrap();
        variables
            .push(solve_variables::Row {
                run_id,
                step,
                symbol_id: owner,
                quantity_id: None,
                unit_id: None,
                fixed: false,
                parameter: false,
                domain: None,
                value: Some(-0.0),
                lower: None,
                upper: None,
                lower_violation: None,
                upper_violation: None,
                tolerance: None,
                lower_dual: None,
                upper_dual: None,
                reduced_cost: None,
                stationarity: None,
                dual_qualification: DualQualification::Unavailable,
            })
            .unwrap();
        let variables = variables.finish().unwrap();
        let locator = selected_scalar_row(&variables, "attempt", "set", "block", 0, 0).unwrap();
        assert_eq!(locator.run_id, run_id);
        assert_eq!(locator.owner, owner);
        assert_eq!(locator.step, Some(step));
        assert_eq!(locator.partition, format!("step:{step}"));
        // The locator transport itself retains exact native integers beyond
        // f64's integral range, independently of the scientific step domain.
        let large_step = 9_007_199_254_740_993;
        let wide_locator = RowLocator {
            step: Some(large_step),
            partition: format!("step:{large_step}"),
            ..locator.clone()
        };
        pse_operations::canonical_results::validate_result_row(&wide_locator).unwrap();
        let encoded =
            pse_operations::generated::surreal::encode_canonical_result_rows(&wide_locator)
                .unwrap();
        let decoded =
            pse_operations::generated::surreal::decode_canonical_result_rows(encoded).unwrap();
        assert_eq!(decoded, wide_locator);
        assert_eq!(decoded.step, Some(large_step));
        assert_eq!(
            scalar_rows_at(&variables, "attempt", "set", "block", 0, 1, 0)
                .unwrap()
                .len(),
            1,
            "ten scalar fields still produce one locator"
        );
        let mut constraints =
            solve_constraints::Builder::with_registry(registry, 1, &validation).unwrap();
        constraints
            .push(solve_constraints::Row {
                run_id,
                step,
                row_id: owner,
                quantity_id: None,
                unit_id: None,
                value: None,
                lower: None,
                upper: None,
                equality_residual: None,
                lower_violation: None,
                upper_violation: None,
                tolerance: None,
                dual: None,
                dual_qualification: DualQualification::Unavailable,
            })
            .unwrap();
        let constraints = constraints.finish().unwrap();
        let constraint =
            selected_scalar_row(&constraints, "attempt", "set", "block", 0, 0).unwrap();
        assert_eq!(
            (constraint.run_id, constraint.owner, constraint.step),
            (run_id, owner, Some(step))
        );
        assert_ne!(constraint.relation_id, locator.relation_id);
        let experiment = SemanticId::from_bytes([9; 16]);
        let mut observations =
            fit_observations::Builder::with_registry(registry, 1, &validation).unwrap();
        observations
            .push(fit_observations::Row {
                run_id,
                observation_id: owner,
                experiment_id: experiment.into(),
                included: true,
                prediction: None,
                residual: None,
                standardized_residual: None,
                objective_contribution: None,
                unit_id: SemanticId::NIL,
            })
            .unwrap();
        let observations = observations.finish().unwrap();
        let observed = selected_scalar_row(&observations, "attempt", "set", "block", 0, 0).unwrap();
        assert_eq!(
            (observed.run_id, observed.owner, observed.experiment_id),
            (run_id, owner, Some(experiment))
        );
        assert_eq!(observed.step, None);
        assert_eq!(observed.partition, format!("experiment:{experiment}"));
    }

    #[test]
    fn result_metadata_prefix_packs_one_locator_per_row_by_encoded_extent() {
        let relation = solve_variables::RELATION_ID;
        let mut extents = Vec::new();
        for row in 0..1000 {
            let locator = RowLocator {
                key: format!("{}:{row}", "b".repeat(64)),
                result_set: "s".repeat(64),
                attempt: "a".repeat(64),
                batch: "b".repeat(64),
                relation_id: relation,
                run_id: SemanticId::NIL.into(),
                owner: relation,
                step: Some(0),
                experiment_id: None,
                output: format!("{relation}:{relation}"),
                partition: "step:0".into(),
                row,
                offset: row,
                interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
            };
            extents.push(
                pse_operations::canonical_execution::result_row_metadata_extent(&locator).unwrap(),
            );
        }
        let rows = metadata_prefix(&extents).unwrap();
        assert!(
            rows > 6,
            "packing does not multiply metadata by ten requested fields"
        );
        assert!(rows < 1000);
        assert!(
            4096 + extents[..rows].iter().sum::<usize>()
                <= pse_operations::canonical_execution::EXECUTION_METADATA_BYTES
        );
        assert!(
            4096 + extents[..=rows].iter().sum::<usize>()
                > pse_operations::canonical_execution::EXECUTION_METADATA_BYTES
        );
        assert!(
            metadata_prefix(&[pse_operations::canonical_execution::EXECUTION_METADATA_BYTES])
                .is_err()
        );
    }
}
