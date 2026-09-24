// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Separate working allowances for decoded algorithm values and native columnar admission.
use crate::{CanonError, Envelope, EnvelopeBound};
use arrow::array::ArrayData;
use arrow_array::RecordBatch;

/// Bound visible values, recursive value slots and dictionary expansion before decoding.
/// Existing backing buffers retain their own owners and are not charged again here.
/// The forecast is allocation accounting, not registry or semantic admission.
/// # Errors
/// Checked extent overflow returns a resource-limit envelope error.
pub fn algorithm_decode_extent(batch: &RecordBatch) -> Result<usize, CanonError> {
    fn slots(data: &ArrayData) -> Result<usize, CanonError> {
        data.child_data()
            .iter()
            .try_fold(data.len(), |total, child| {
                total.checked_add(slots(child)?).ok_or_else(overflow)
            })
    }
    let slots = batch.columns().iter().try_fold(0usize, |total, array| {
        total
            .checked_add(slots(&array.to_data())?)
            .ok_or_else(overflow)
    })?;
    // IPC slices can all report the capacity of the same complete file. Row decoding
    // copies visible values, not each slice's retained allocation. That allocation
    // already holds its own lease; multiplying it per field can reject a small batch.
    let bytes = batch.columns().iter().try_fold(0usize, |total, array| {
        let data = array.to_data();
        let expanded = dictionary_expansion(&data)?;
        total
            .checked_add(visible_buffers(&data)?)
            .and_then(|bytes| bytes.checked_add(expanded))
            .ok_or_else(overflow)
    })?;
    bytes
        .checked_mul(32)
        .and_then(|bytes| {
            slots
                .checked_mul(256)
                .and_then(|slots| bytes.checked_add(slots))
        })
        .and_then(|bytes| bytes.checked_add(4096))
        .ok_or_else(overflow)
}

fn visible_buffers(data: &ArrayData) -> Result<usize, CanonError> {
    let mut bytes = data.nulls().map_or(0, |nulls| nulls.buffer().len());
    for buffer in data.buffers() {
        bytes = bytes.checked_add(buffer.len()).ok_or_else(overflow)?;
    }
    data.child_data().iter().try_fold(bytes, |total, child| {
        total
            .checked_add(visible_buffers(child)?)
            .ok_or_else(overflow)
    })
}

/// Bound decoded UTF-8 dictionary copies, including nested dictionary children.
/// # Errors
/// Missing dictionary storage or checked extent overflow returns an envelope error.
fn dictionary_expansion(data: &ArrayData) -> Result<usize, CanonError> {
    use arrow_schema::DataType;
    let mut bytes = 0usize;
    if matches!(data.data_type(), DataType::Dictionary(_, value) if value.as_ref() == &DataType::Utf8)
    {
        let values = data.child_data().first().ok_or_else(overflow)?;
        let offsets = values
            .buffers()
            .first()
            .ok_or_else(overflow)?
            .typed_data::<i32>();
        let max = offsets.windows(2).try_fold(0usize, |max, pair| {
            usize::try_from(pair[1].checked_sub(pair[0]).ok_or_else(overflow)?)
                .map(|length| max.max(length))
                .map_err(|_| overflow())
        })?;
        bytes = data.len().checked_mul(max).ok_or_else(overflow)?;
    }
    for child in data.child_data() {
        bytes = bytes
            .checked_add(dictionary_expansion(child)?)
            .ok_or_else(overflow)?;
    }
    Ok(bytes)
}

fn overflow() -> CanonError {
    CanonError::Envelope {
        what: EnvelopeBound::Bytes,
        limit: Envelope::DEFAULT.max_normalized_bytes,
        actual: u64::MAX,
    }
}

/// Reserve schema/control working storage without charging the represented values.
/// The immutable Arrow buffers retain their own native allocation owners.
/// # Errors
/// Checked schema size overflow.
pub fn schema_working_extent(schema: &arrow_schema::Schema) -> Result<usize, CanonError> {
    let metadata = schema
        .metadata()
        .iter()
        .try_fold(0usize, |bytes, (key, value)| {
            bytes
                .checked_add(key.capacity())
                .and_then(|size| size.checked_add(value.capacity()))
                .ok_or_else(overflow)
        })?;
    schema
        .fields()
        .size()
        .checked_add(metadata)
        .and_then(|size| size.checked_mul(2))
        .and_then(|size| size.checked_add(4096))
        .ok_or_else(overflow)
}
/// Working allowance for columnar predicate arrays and nested occurrence indexes.
/// This excludes Rust row decoding and the independently owned input buffers.
/// # Errors
/// Checked extent overflow.
pub fn columnar_validation_extent(batch: &RecordBatch) -> Result<usize, CanonError> {
    fn slots(data: &ArrayData) -> Result<usize, CanonError> {
        data.child_data()
            .iter()
            .try_fold(data.len(), |total, child| {
                total.checked_add(slots(child)?).ok_or_else(overflow)
            })
    }
    let mut bytes = schema_working_extent(batch.schema_ref())?;
    for array in batch.columns() {
        bytes = bytes
            .checked_add(
                slots(&array.to_data())?
                    .checked_mul(64)
                    .ok_or_else(overflow)?,
            )
            .ok_or_else(overflow)?;
    }
    Ok(bytes)
}
