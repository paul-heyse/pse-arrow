// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Account decoded diagnostic keys and conservative full-binding support before allocation.
use crate::RuleError;
use datafusion::arrow::array::{ArrayData, RecordBatch};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, Reservation};

pub(crate) fn reserve_evidence<'a>(
    outputs: impl Iterator<Item = &'a RecordBatch>,
    mut dependencies: impl Iterator<Item = &'a RecordBatch>,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<Box<dyn Reservation>, RuleError> {
    cancel
        .checkpoint()
        .map_err(pse_catalog::CatalogError::from)?;
    let mut output_bytes = 0usize;
    let mut count = 0usize;
    for batch in outputs {
        output_bytes = add(output_bytes, extent(batch)?)?;
        count = add(count, batch.num_rows())?;
    }
    if count == 0 {
        // No diagnostic/supporting cells are decoded for an empty rule result.
        return Ok(session.reserver().open("rules:diagnostic-cells"));
    }
    let supporting = dependencies.try_fold(0, |total, batch| add(total, extent(batch)?))?;
    // Each result carries supporting rows in provenance and possibly undecided output;
    // the factor also covers temporary strings and the original decoded dependency rows.
    let bytes = add(
        mul(output_bytes, 4)?,
        mul(supporting, add(mul(count, 4)?, 1)?)?,
    )?;
    let mut reservation = session.reserver().open("rules:diagnostic-cells");
    reservation
        .try_grow(bytes)
        .map_err(pse_catalog::CatalogError::from)?;
    Ok(reservation)
}
fn extent(batch: &RecordBatch) -> Result<usize, RuleError> {
    let slots = batch
        .columns()
        .iter()
        .try_fold(0, |total, array| add(total, slot_count(&array.to_data())?))?;
    let bytes = pse_ids::owned_buffer::retained_buffer_bytes(batch)
        .map_err(pse_catalog::CatalogError::from)?;
    // Text JSON escaping is at most six bytes per input byte; IDs/hash literals add
    // at most a 2x hexadecimal expansion. The fixed per-cell charge covers nested tags.
    add(mul(bytes, 32)?, mul(slots, 256)?)
}
fn slot_count(data: &ArrayData) -> Result<usize, RuleError> {
    data.child_data()
        .iter()
        .try_fold(data.len(), |total, child| add(total, slot_count(child)?))
}
fn add(a: usize, b: usize) -> Result<usize, RuleError> {
    a.checked_add(b).ok_or_else(|| RuleError::ResourceLimit {
        consumer: "rules:diagnostic-cells extent overflow".into(),
        config_keys: vec!["datafusion.runtime.memory_limit".into()],
    })
}
fn mul(a: usize, b: usize) -> Result<usize, RuleError> {
    a.checked_mul(b).ok_or_else(|| RuleError::ResourceLimit {
        consumer: "rules:diagnostic-cells extent overflow".into(),
        config_keys: vec!["datafusion.runtime.memory_limit".into()],
    })
}
