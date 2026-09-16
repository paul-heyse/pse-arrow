// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One generated relation association for typed algorithm outputs.

use super::FieldCheckedBatch;
use crate::RelationError;

/// Associates a generated top-level row with its declared Arrow builder.
/// This is a mechanical codec association, never a relational validity claim.
pub trait RelationRow: Sized + 'static {
    /// The generated column builder for this exact row declaration.
    type Builder: Send + 'static;
    /// Resolve the complete matching declaration in the receiving registry.
    /// # Errors
    /// Missing or mismatched declaration.
    fn relation(
        registry: &pse_schema::Registry,
    ) -> Result<&pse_schema::model::RelationSpec, RelationError>;
    /// Start generated native columns for this declaration.
    /// # Errors
    /// Declaration or Arrow construction failure.
    fn builder(
        registry: &pse_schema::Registry,
        capacity: usize,
    ) -> Result<Self::Builder, RelationError>;
    /// Check the row's local values and append directly to its native columns.
    /// # Errors
    /// Local values or Arrow offsets are invalid.
    fn push(builder: &mut Self::Builder, row: Self) -> Result<(), RelationError>;
    /// Complete the actual generated columns, with local field checks retained.
    /// # Errors
    /// Arrow construction failure.
    fn finish(builder: Self::Builder) -> Result<FieldCheckedBatch, RelationError>;
    /// Decode an explicit algorithm input after exact native selection and ordering.
    /// The caller retains the completed columns and accounts for the returned DTOs.
    /// # Errors
    /// The checked owner has a different generated declaration.
    fn rows(batch: &FieldCheckedBatch) -> Result<Vec<Self>, RelationError>;
    /// Conservative initial schema and nested Arrow buffer allocation forecast.
    fn builder_allocation_size() -> usize;
    /// Conservative masked-child storage for one row, including null fixed-size values.
    fn minimum_row_allocation_size() -> usize;
    /// Checked logical payload extent for reserve-before-append accounting.
    /// Collection owners add capacity growth, bitmap and per-column buffer overhead.
    /// # Errors
    /// The actual nested payload extent overflows addressable storage.
    fn allocation_size(&self) -> Result<usize, RelationError>;
}

pub(crate) fn allocation_add(left: usize, right: usize) -> Result<usize, RelationError> {
    left.checked_add(right)
        .ok_or_else(|| super::mismatch("representable generated payload extent"))
}
