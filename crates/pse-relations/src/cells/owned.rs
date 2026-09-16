// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reserve-first construction using actual recursive cells, including masked defaults.

use std::sync::Arc;

use arrow_array::RecordBatch;
use arrow_schema::{DataType, Field, Schema};
use pse_ids::{
    CancellationToken, CanonError, Envelope, EnvelopeBound, MemoryReserver, ReservationLease,
};
use pse_schema::Registry;
use pse_schema::model::{Cell, RelationSpec};

use crate::RelationError;

/// Builds a complete declared relation after reserving recursive construction space.
/// Every exposed Arrow buffer retains the same reservation, including detached children.
/// Input `Cell` storage remains the caller's responsibility; the reservation also covers
/// conservative temporary copies while the input and result coexist.
///
/// # Errors
/// Typed envelope, reservation, cancellation, schema or visible-value failures.
pub fn batch_from_cells_owned(
    reg: &Registry,
    spec: &RelationSpec,
    rows: &[Vec<Cell>],
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<RecordBatch, RelationError> {
    let schema = Arc::new(pse_schema::arrow::relation_schema(reg, spec)?);
    let batch = batch_from_columns_owned(reg, schema, rows, reserver, cancel)?;
    crate::validate::validate_batch(reg, spec, &batch)
        .map_err(|errors| RelationError::Validation { errors })?;
    Ok(batch)
}

/// Builds a declared field subset after reserving its recursive construction space.
/// Each field and visible value is checked, without certifying relation membership,
/// keys or cross-column rules. Suitable for a rule's declared violation head.
///
/// # Errors
/// Typed envelope, reservation, cancellation, field or visible-value failures.
pub fn batch_from_columns_owned(
    reg: &Registry,
    schema: Arc<Schema>,
    rows: &[Vec<Cell>],
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<RecordBatch, RelationError> {
    cancel.checkpoint()?;
    bound(rows.len(), Envelope::DEFAULT.max_rows, EnvelopeBound::Rows)?;
    let mut count = Count::default();
    for field in schema.fields() {
        crate::validate::validate_field(reg, field)
            .map_err(|errors| RelationError::Validation { errors })?;
        count.field(field, 0)?;
    }
    for (key, value) in schema.metadata() {
        count.temporary = add(count.temporary, mul(add(key.len(), value.len())?, 8)?)?;
    }
    for (row_number, row) in rows.iter().enumerate() {
        cancel.checkpoint()?;
        if row.len() != schema.fields().len() {
            return Err(super::value_error(
                "cell rows",
                row_number,
                "row width mismatch",
            ));
        }
        for (field, cell) in schema.fields().iter().zip(row) {
            count.cell(field, Some(cell), 0)?;
        }
    }
    bound(
        count.storage,
        Envelope::DEFAULT.max_normalized_bytes,
        EnvelopeBound::Bytes,
    )?;
    let mut reservation = reserver.open("relations:cell-construction");
    reservation
        .try_grow(add(16_384, add(count.temporary, mul(count.storage, 8)?)?)?)
        .map_err(CanonError::from)?;
    cancel.checkpoint()?;
    let batch = super::build_batch(schema, rows)?;
    for (field, array) in batch.schema().fields().iter().zip(batch.columns()) {
        crate::validate::validate_column(reg, field, array.as_ref())
            .map_err(|errors| RelationError::Validation { errors })?;
    }
    cancel.checkpoint()?;
    let retained = pse_ids::owned_buffer::retained_buffer_bytes(&batch)?;
    reservation.shrink(reservation.size().saturating_sub(retained));
    Ok(pse_ids::owned_buffer::attach_reservation(
        batch,
        ReservationLease::new(reservation),
    )?)
}

#[derive(Default)]
struct Count {
    storage: usize,
    temporary: usize,
}
impl Count {
    fn field(&mut self, field: &Field, depth: usize) -> Result<(), RelationError> {
        depth_bound(depth)?;
        self.temporary = add(self.temporary, mul(add(1024, field.name().len())?, 8)?)?;
        self.storage = add(self.storage, 256)?; // Buffer padding for empty arrays too.
        for (key, value) in field.metadata() {
            self.temporary = add(self.temporary, mul(add(key.len(), value.len())?, 8)?)?;
        }
        match field.data_type() {
            DataType::List(child) | DataType::FixedSizeList(child, _) => {
                self.field(child, depth + 1)?;
            }
            DataType::Struct(children) => {
                for child in children {
                    self.field(child, depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn cell(
        &mut self,
        field: &Field,
        cell: Option<&Cell>,
        depth: usize,
    ) -> Result<(), RelationError> {
        depth_bound(depth)?;
        // Charge actual declared slots plus a conservatively rounded validity byte.
        // Parent containers clone their child Cells while the root column copy lives;
        // two slots per ancestor cover Vec growth and the temporary decoded value.
        let slots = match field.data_type() {
            DataType::Boolean => 1,
            DataType::Utf8 | DataType::List(_) => 4,
            DataType::Dictionary(_, _) => 9, // key + value offset + value validity
            DataType::FixedSizeBinary(width) => {
                usize::try_from(*width).map_err(|_| invalid(field))?
            }
            DataType::Struct(_) | DataType::FixedSizeList(_, _) => 0,
            kind => kind.primitive_width().ok_or_else(|| invalid(field))?,
        };
        self.storage = add(self.storage, add(slots, 1)?)?;
        self.temporary = add(self.temporary, mul(size_of::<Cell>(), mul(2, depth + 2)?)?)?;
        let visible = cell.filter(|cell| !matches!(cell, Cell::Null));
        match (field.data_type(), visible) {
            (DataType::List(child), Some(Cell::List(items))) => self.items(child, items, depth)?,
            (DataType::FixedSizeList(child, width), value) => {
                let width = usize::try_from(*width).map_err(|_| invalid(field))?;
                if let Some(Cell::List(items)) = value {
                    if items.len() != width {
                        return Err(invalid(field));
                    }
                    self.items(child, items, depth)?;
                } else if value.is_none() {
                    // Masked fixed children are materialized as valid defaults by the
                    // sole storage builder, so their space must still be reserved.
                    for _ in 0..width {
                        self.cell(child, None, depth + 1)?;
                    }
                } else {
                    return Err(invalid(field));
                }
            }
            (DataType::Struct(fields), value) => {
                if let Some(Cell::Struct(items)) = value {
                    if fields.len() != items.len() {
                        return Err(invalid(field));
                    }
                    for (child, item) in fields.iter().zip(items) {
                        self.cell(child, Some(item), depth + 1)?;
                    }
                } else if value.is_none() {
                    for child in fields {
                        self.cell(child, None, depth + 1)?;
                    }
                } else {
                    return Err(invalid(field));
                }
            }
            (DataType::Utf8, Some(Cell::Text(value))) => self.text(value.len(), depth)?,
            (DataType::Utf8 | DataType::Dictionary(_, _), Some(Cell::Enum(value))) => {
                self.text(value.len(), depth)?;
            }
            (_, Some(Cell::List(_) | Cell::Struct(_) | Cell::Text(_) | Cell::Enum(_)))
            | (DataType::List(_) | DataType::Utf8 | DataType::Dictionary(_, _), Some(_)) => {
                return Err(invalid(field));
            }
            _ => {} // Scalar kinds/ranges are checked by the existing storage builder.
        }
        Ok(())
    }

    fn items(&mut self, field: &Field, items: &[Cell], depth: usize) -> Result<(), RelationError> {
        bound(items.len(), i32::MAX as u64, EnvelopeBound::Rows)?;
        for item in items {
            self.cell(field, Some(item), depth + 1)?;
        }
        Ok(())
    }
    fn text(&mut self, bytes: usize, depth: usize) -> Result<(), RelationError> {
        self.storage = add(self.storage, bytes)?;
        self.temporary = add(self.temporary, mul(bytes, mul(depth + 4, 4)?)?)?;
        Ok(())
    }
}
fn invalid(field: &Field) -> RelationError {
    super::value_error(field.name(), 0, "cell shape differs from declared storage")
}
fn overflow() -> CanonError {
    CanonError::Envelope {
        what: EnvelopeBound::Bytes,
        limit: Envelope::DEFAULT.max_normalized_bytes,
        actual: u64::MAX,
    }
}
fn add(one: usize, two: usize) -> Result<usize, CanonError> {
    one.checked_add(two).ok_or_else(overflow)
}
fn mul(one: usize, two: usize) -> Result<usize, CanonError> {
    one.checked_mul(two).ok_or_else(overflow)
}
fn depth_bound(depth: usize) -> Result<(), RelationError> {
    if depth > 64 {
        Err(super::value_error(
            "cell rows",
            0,
            "nested construction exceeds depth 64",
        ))
    } else {
        Ok(())
    }
}
fn bound(actual: usize, limit: u64, what: EnvelopeBound) -> Result<(), CanonError> {
    let actual = u64::try_from(actual).map_err(|_| overflow())?;
    if actual > limit {
        Err(CanonError::Envelope {
            what,
            limit,
            actual,
        })
    } else {
        Ok(())
    }
}
