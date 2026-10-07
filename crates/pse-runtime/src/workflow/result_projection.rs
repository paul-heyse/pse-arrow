// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native scalar indexes are derived from declared scientific fields. The
//! original admitted Arrow row remains the scientific result authority.

use super::{WorkflowError, contract, relation, result_blocks};
use pse_columnar::{CancellationToken, MemoryConsumer, MemoryPool};
use pse_ids::SemanticId;
use pse_model::generated::runtime::{
    canonical_result_block_outputs::Row as Output, canonical_result_blocks::Row as Block,
    canonical_result_cells::Row as Cell,
};
use pse_operations::{
    canonical::CanonicalStore,
    canonical_execution::{AttemptFence, result_batch_key, result_payload_digest, result_set_key},
};
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::runtime::{
        fit_observations, fit_parameters, simulation_samples, solve_constraints, solve_variables,
    },
};
use std::sync::Arc;

fn scalar_width(id: SemanticId) -> usize {
    if id == solve_variables::RELATION_ID {
        10
    } else if id == solve_constraints::RELATION_ID {
        8
    } else if id == fit_parameters::RELATION_ID {
        2
    } else if id == fit_observations::RELATION_ID {
        4
    } else {
        0
    }
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

#[allow(
    clippy::too_many_arguments,
    reason = "the derived cell binds the immutable batch and each declared scientific output/row coordinate to its original scalar value"
)]
fn cell(
    set: &str,
    batch: &str,
    relation: SemanticId,
    owner: SemanticId,
    field: &str,
    partition: &str,
    row: u64,
    value: Option<f64>,
) -> Result<Cell, WorkflowError> {
    if value.is_some_and(|v| !v.is_finite()) {
        return Err(contract(
            "finite scientific index field contains nonfinite bits",
        ));
    }
    Ok(Cell {
        key: format!("{batch}:{row}:{field}"),
        result_set: set.into(),
        batch: batch.into(),
        output: format!("{relation}:{owner}:{field}"),
        partition: partition.into(),
        row,
        coordinate: row.to_string(),
        cell_kind: if value.is_some() { "finite" } else { "missing" }.into(),
        bits: value.map(|v| v.to_bits().to_be_bytes().to_vec().into()),
        projection: value.map(|v| if v == 0.0 { 0.0 } else { v }),
        interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
    })
}

/// Derive only explicitly declared finite fields, retaining owning declaration,
/// occurrence partition and exact original row coordinates in each index.
pub(super) fn scalar_cells(
    table: &FieldCheckedBatch,
    set: &str,
    batch: &str,
    start: usize,
    count: usize,
) -> Result<Vec<Cell>, WorkflowError> {
    scalar_cells_at(table, set, batch, start, count, 0)
}
pub(super) fn scalar_cells_at(
    table: &FieldCheckedBatch,
    set: &str,
    batch: &str,
    start: usize,
    count: usize,
    origin: u64,
) -> Result<Vec<Cell>, WorkflowError> {
    let id = table.relation_id();
    let mut cells = Vec::with_capacity(count * scalar_width(id));
    if scalar_width(id) == 0 {
        return Ok(cells);
    }
    for index in start..start + count {
        let row = origin
            .checked_add(index as u64)
            .ok_or_else(|| contract("scientific row coordinate overflow"))?;
        if id == solve_variables::RELATION_ID {
            let value = solve_variables::View::from_checked(table)
                .map_err(relation)?
                .row(index)
                .map_err(relation)?;
            let partition = format!("step:{}", value.step);
            for (field, number) in [
                ("value", value.value),
                ("lower", value.lower),
                ("upper", value.upper),
                ("lower_violation", value.lower_violation),
                ("upper_violation", value.upper_violation),
                ("tolerance", value.tolerance),
                ("lower_dual", value.lower_dual),
                ("upper_dual", value.upper_dual),
                ("reduced_cost", value.reduced_cost),
                ("stationarity", value.stationarity),
            ] {
                cells.push(cell(
                    set,
                    batch,
                    id,
                    value.symbol_id,
                    field,
                    &partition,
                    row,
                    number,
                )?);
            }
        } else if id == solve_constraints::RELATION_ID {
            let value = solve_constraints::View::from_checked(table)
                .map_err(relation)?
                .row(index)
                .map_err(relation)?;
            let partition = format!("step:{}", value.step);
            for (field, number) in [
                ("value", value.value),
                ("lower", value.lower),
                ("upper", value.upper),
                ("equality_residual", value.equality_residual),
                ("lower_violation", value.lower_violation),
                ("upper_violation", value.upper_violation),
                ("tolerance", value.tolerance),
                ("dual", value.dual),
            ] {
                cells.push(cell(
                    set,
                    batch,
                    id,
                    value.row_id,
                    field,
                    &partition,
                    row,
                    number,
                )?);
            }
        } else if id == fit_parameters::RELATION_ID {
            let value = fit_parameters::View::from_checked(table)
                .map_err(relation)?
                .row(index)
                .map_err(relation)?;
            for (field, number) in [("value", value.value), ("scale", Some(value.scale))] {
                cells.push(cell(
                    set,
                    batch,
                    id,
                    value.parameter_id,
                    field,
                    "0",
                    row,
                    number,
                )?);
            }
        } else if id == fit_observations::RELATION_ID {
            let value = fit_observations::View::from_checked(table)
                .map_err(relation)?
                .row(index)
                .map_err(relation)?;
            let partition = format!("experiment:{}", value.experiment_id);
            for (field, number) in [
                ("prediction", value.prediction),
                ("residual", value.residual),
                ("standardized_residual", value.standardized_residual),
                ("objective_contribution", value.objective_contribution),
            ] {
                cells.push(cell(
                    set,
                    batch,
                    id,
                    value.observation_id,
                    field,
                    &partition,
                    row,
                    number,
                )?);
            }
        }
    }
    cells.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(cells)
}

/// Ingest the sole admitted scientific IPC payload and its derived scalar index
/// through the same fenced transaction. IPC and derived metadata independently
/// choose useful prefixes; scalar count is only a defensive extent ceiling.
pub(super) async fn store_result_table(
    store: &CanonicalStore,
    fence: &AttemptFence,
    id: SemanticId,
    table: &FieldCheckedBatch,
    pool: &Arc<dyn MemoryPool>,
) -> Result<(), WorkflowError> {
    if id != table.relation_id() {
        return Err(contract("result table relation identity mismatch"));
    }
    if table.batch().num_rows() == 0 {
        let owner = MemoryConsumer::new("canonical:empty-result-block").register(pool);
        owner
            .try_grow(2 * 1024 * 1024)
            .map_err(pse_engine::EngineError::from)?;
        let name = id.to_string();
        let set = result_set_key(fence.attempt(), &name);
        let key = result_batch_key(fence.attempt(), &set, 0);
        let payload = result_blocks::encode_result_block(table.batch())?;
        let block = Block {
            key: key.clone(),
            batch: key,
            result_set: set,
            output: name.clone(),
            partition: "0".into(),
            ordinal: 0,
            start: 0,
            end: 0,
            rows: 0,
            columns: table.batch().num_columns() as u64,
            coordinate_min: None,
            coordinate_max: None,
            payload_bytes: payload.len() as u64,
            payload_digest: result_payload_digest(&payload),
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        };
        store
            .append_result_block(
                fence,
                &format!("table:{}:{}", fence.attempt(), block.key),
                &name,
                0,
                &payload,
                0,
                &block,
            )
            .await?;
        return Ok(());
    }
    if id == simulation_samples::RELATION_ID {
        return store_trajectory(store, fence, table, pool).await;
    }
    let reservation = MemoryConsumer::new("canonical:result-scalar-index").register(pool);
    // Live prepared cells, encoded metadata, Arrow writer/protobuf scratch and
    // the outgoing payload are independent of either encoded admission ceiling.
    let name = id.to_string();
    let set = result_set_key(fence.attempt(), &name);
    let width = scalar_width(id);
    let window = pse_operations::generated::surreal::RESULT_INDEX_RECORDS
        .checked_div(width)
        .map_or(32768, |rows| rows.min(1024));
    reservation
        .try_grow(
            window * width * (size_of::<Cell>() + 1024)
                + window * size_of::<usize>()
                + 16 * pse_operations::canonical_execution::EXECUTION_METADATA_BYTES
                + 4 * result_blocks::RESULT_BLOCK_BYTES,
        )
        .map_err(pse_engine::EngineError::from)?;
    let mut ordinal = 0_u64;
    for base in (0..table.batch().num_rows()).step_by(window) {
        let count = window.min(table.batch().num_rows() - base);
        let initial_key = result_batch_key(fence.attempt(), &set, ordinal);
        let prepared = scalar_cells(table, &set, &initial_key, base, count)?;
        let mut extents = vec![0_usize; count];
        for cell in &prepared {
            let extent = pse_operations::canonical_execution::result_cell_metadata_extent(cell)?;
            extents[cell.row as usize - base] += extent;
        }
        let mut offset = 0;
        while offset < count {
            let current = ordinal;
            let key = result_batch_key(fence.attempt(), &set, current);
            // Descriptor envelope allowance includes maximum encoded actual
            // identity lengths below. Cell extents are prepared once per window.
            let rows = metadata_prefix(&extents[offset..])?;
            let (payload, rows) =
                result_blocks::encode_result_prefix(table.batch(), base + offset, rows)?;
            let start = base + offset;
            let cells = prepared
                .iter()
                .filter(|cell| cell.row >= start as u64 && cell.row < (start + rows) as u64)
                .map(|cell| {
                    let mut cell = cell.clone();
                    cell.key = format!("{key}{}", &cell.key[initial_key.len()..]);
                    cell.batch = key.clone();
                    cell
                })
                .collect::<Vec<_>>();
            let block = Block {
                key: key.clone(),
                batch: key,
                result_set: set.clone(),
                output: name.clone(),
                partition: "0".into(),
                ordinal: current,
                start: start as u64,
                end: (start + rows) as u64,
                rows: rows as u64,
                columns: table.batch().num_columns() as u64,
                coordinate_min: None,
                coordinate_max: None,
                payload_bytes: payload.len() as u64,
                payload_digest: result_payload_digest(&payload),
                interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
            };
            store
                .append_result_block_cells(
                    fence,
                    &format!("table:{}:{}", fence.attempt(), block.key),
                    &name,
                    current,
                    &payload,
                    rows as u64,
                    &block,
                    &cells,
                )
                .await?;
            ordinal += 1;
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

/// Sort storage by the declared output and sample keys. Original scientific
/// sample/time coordinates and all exact value/metadata columns remain intact.
fn grouped_trajectory(
    table: &FieldCheckedBatch,
    pool: &Arc<dyn MemoryPool>,
) -> Result<FieldCheckedBatch, WorkflowError> {
    let extent = table
        .batch()
        .num_rows()
        .checked_mul(64)
        .ok_or_else(|| contract("trajectory order extent overflow"))?;
    let owner = MemoryConsumer::new("canonical:trajectory-order").register(pool);
    owner
        .try_grow(extent)
        .map_err(pse_engine::EngineError::from)?;
    let view = simulation_samples::View::from_checked(table).map_err(relation)?;
    let mut order = Vec::with_capacity(view.len());
    for index in 0..view.len() {
        let row = view.row(index).map_err(relation)?;
        order.push((
            row.symbol_id,
            row.sample,
            u32::try_from(index)
                .map_err(|_| contract("trajectory sort row exceeds Arrow take extent"))?,
        ));
    }
    order.sort_unstable_by_key(|&(symbol, sample, _)| (symbol, sample));
    let indices = datafusion::arrow::array::UInt32Array::from(
        order.into_iter().map(|(_, _, row)| row).collect::<Vec<_>>(),
    );
    table
        .take_reserved(&indices, pool, &CancellationToken::new())
        .map_err(relation)
}

async fn store_trajectory(
    store: &CanonicalStore,
    fence: &AttemptFence,
    table: &FieldCheckedBatch,
    pool: &Arc<dyn MemoryPool>,
) -> Result<(), WorkflowError> {
    let table = grouped_trajectory(table, pool)?;
    let view = simulation_samples::View::from_checked(&table).map_err(relation)?;
    let id = table.relation_id();
    let name = id.to_string();
    let set = result_set_key(fence.attempt(), &name);
    let mut ordinal = 0_u64;
    let mut base = 0;
    let owner = MemoryConsumer::new("canonical:trajectory-output-index").register(pool);
    owner
        .try_grow(2 * 1024 * 1024)
        .map_err(pse_engine::EngineError::from)?;
    while base < view.len() {
        let symbol = view.row(base).map_err(relation)?.symbol_id;
        let mut end = base + 1;
        while end < view.len() && view.row(end).map_err(relation)?.symbol_id == symbol {
            end += 1;
        }
        let batch = table.batch().slice(base, end - base);
        result_blocks::visit_result_blocks_async(&batch, |offset, rows, payload| {
            let start = base + offset;
            let current = ordinal;
            ordinal += 1;
            let key = result_batch_key(fence.attempt(), &set, current);
            let extrema = (start..start + rows).try_fold(
                (f64::INFINITY, f64::NEG_INFINITY),
                |(min, max), row| {
                    let time = view.row(row).map_err(relation)?.time;
                    Ok::<_, WorkflowError>((min.min(time), max.max(time)))
                },
            );
            let block = Block {
                key: key.clone(),
                batch: key.clone(),
                result_set: set.clone(),
                output: name.clone(),
                partition: "0".into(),
                ordinal: current,
                start: start as u64,
                end: (start + rows) as u64,
                rows: rows as u64,
                columns: table.batch().num_columns() as u64,
                coordinate_min: None,
                coordinate_max: None,
                payload_bytes: payload.len() as u64,
                payload_digest: result_payload_digest(&payload),
                interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
            };
            let output = Output {
                key: format!("{key}:value"),
                batch: key,
                result_set: set.clone(),
                output: format!("{id}:{symbol}:value"),
                partition: "0".into(),
                start: start as u64,
                end: (start + rows) as u64,
                coordinate_min: None,
                coordinate_max: None,
                interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
            };
            let operation = format!("table:{}:{}", fence.attempt(), block.key);
            let name = name.clone();
            async move {
                let (min, max) = extrema?;
                let mut output = output;
                output.coordinate_min = Some(min);
                output.coordinate_max = Some(max);
                store
                    .append_result_block_indexes(
                        fence,
                        &operation,
                        &name,
                        current,
                        &payload,
                        rows as u64,
                        &block,
                        &[],
                        &[output],
                    )
                    .await?;
                Ok::<(), WorkflowError>(())
            }
        })
        .await?;
        base = end;
    }
    Ok(())
}

#[cfg(test)]
mod result_projection_unit {
    use super::*;
    #[test]
    fn result_metadata_prefix_uses_encoded_extent_and_keeps_exact_cells() {
        let relation = solve_variables::RELATION_ID;
        let mut extents = Vec::new();
        let mut cells = Vec::new();
        for row in 0..100 {
            let mut extent = 0;
            for field in [
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
            ] {
                let value = if row % 2 == 0 { Some(-0.0) } else { None };
                let cell = cell(
                    &"s".repeat(64),
                    &"b".repeat(64),
                    relation,
                    relation,
                    field,
                    "step:0",
                    row,
                    value,
                )
                .unwrap();
                extent += pse_operations::canonical_execution::result_cell_metadata_extent(&cell)
                    .unwrap();
                cells.push(cell);
            }
            extents.push(extent);
        }
        let rows = metadata_prefix(&extents).unwrap();
        assert!(rows > 6, "ten-field rows are no longer capped by64 cells");
        assert!(rows < 100);
        assert!(
            4096 + extents[..rows].iter().sum::<usize>()
                <= pse_operations::canonical_execution::EXECUTION_METADATA_BYTES
        );
        assert!(
            4096 + extents[..=rows].iter().sum::<usize>()
                > pse_operations::canonical_execution::EXECUTION_METADATA_BYTES
        );
        assert_eq!(
            cells[0].bits.as_ref().unwrap().as_slice(),
            (-0.0f64).to_bits().to_be_bytes()
        );
        assert!(cells[10].bits.is_none());
        assert!(
            metadata_prefix(&[pse_operations::canonical_execution::EXECUTION_METADATA_BYTES])
                .is_err()
        );
    }
}
