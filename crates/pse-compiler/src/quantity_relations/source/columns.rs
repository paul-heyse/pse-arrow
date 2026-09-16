// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Borrowed access to the exact primitive fields selected by native plans.
use super::invalid;
use crate::CompilerError;
use datafusion::arrow::array::{Array, ArrayRef, FixedSizeBinaryArray, Int64Array, ListArray};
use pse_ids::SemanticId;
use pse_relations::RecordBatch;

pub(super) fn column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a ArrayRef, CompilerError> {
    batch
        .column_by_name(name)
        .ok_or_else(|| invalid(format!("native quantity output omits {name}")))
}
pub(super) fn id(batch: &RecordBatch, name: &str, row: usize) -> Result<SemanticId, CompilerError> {
    identity(column(batch, name)?.as_ref(), row)
}
fn identity(column: &dyn Array, row: usize) -> Result<SemanticId, CompilerError> {
    let values = column
        .as_any()
        .downcast_ref::<FixedSizeBinaryArray>()
        .ok_or_else(|| invalid("native identity storage differs"))?;
    if values.is_null(row) {
        return Err(invalid("native required identity is null"));
    }
    Ok(SemanticId::from_bytes(
        values
            .value(row)
            .try_into()
            .map_err(|_| invalid("native identity width differs"))?,
    ))
}
pub(super) fn ids(
    batch: &RecordBatch,
    name: &str,
    row: usize,
) -> Result<Vec<SemanticId>, CompilerError> {
    let column = column(batch, name)?;
    let list = column
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| invalid("native identity list storage differs"))?;
    if list.is_null(row) {
        return Err(invalid("native required list is null"));
    }
    let values = list.value(row);
    (0..values.len())
        .map(|index| identity(values.as_ref(), index))
        .collect()
}
pub(super) fn integer(batch: &RecordBatch, name: &str, row: usize) -> Result<i64, CompilerError> {
    let values = column(batch, name)?
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| invalid("native ordinal storage differs"))?;
    if values.is_null(row) {
        return Err(invalid("native required ordinal is null"));
    }
    Ok(values.value(row))
}
