// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One generated relation association for typed algorithm outputs.

use super::FieldCheckedBatch;
use crate::RelationError;

/// Associates a generated top-level row with its declared Arrow builder.
/// This is a mechanical codec association, never a relational validity claim.
pub trait RelationRow: Sized + 'static {
    /// Encode the declared fields in registry order into already admitted native builders.
    /// # Errors
    /// A value cannot be represented by its declared Arrow storage.
    fn append_columns(
        &self,
        columns: &mut [Box<dyn arrow_array::builder::ArrayBuilder>],
    ) -> Result<(), RelationError>;
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
    /// Encode the row directly into native columns; finish performs domain admission.
    /// # Errors
    /// Values cannot be encoded or Arrow offsets are invalid.
    fn push(builder: &mut Self::Builder, row: Self) -> Result<(), RelationError>;
    /// Complete and validate the actual generated columns against their local contracts.
    /// # Errors
    /// Arrow construction or declared local value validation fails.
    fn finish(builder: Self::Builder) -> Result<FieldCheckedBatch, RelationError>;
    /// Decode an explicit algorithm input after exact native selection and ordering.
    /// The caller retains the completed columns and accounts for the returned DTOs.
    /// # Errors
    /// The checked owner has a different generated declaration.
    fn rows(batch: &FieldCheckedBatch) -> Result<Vec<Self>, RelationError>;
    /// Decode selected positions after one checked declaration admission.
    /// Input order and duplicate positions are preserved.
    /// # Errors
    /// A foreign declaration or an out-of-bounds position.
    fn rows_at(batch: &FieldCheckedBatch, positions: &[usize]) -> Result<Vec<Self>, RelationError>;
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

/// A generated borrowed view with an exact row association.
pub trait RelationView {
    /// Declared row, independent of the physical array container.
    type Row: RelationRow;
    /// The checked native columns.
    fn batch(&self) -> &crate::RecordBatch;
}

/// Shared construction mechanics for all registry-generated relation declarations.
#[derive(Debug)]
pub struct RowBuilder<R: RelationRow> {
    columns: super::BatchBuilder,
    marker: std::marker::PhantomData<fn() -> R>,
}
impl<R: RelationRow> RowBuilder<R> {
    /// Open empty columns using the built-in declaration.
    /// # Errors
    /// Registry or Arrow layout failure.
    pub fn new() -> Result<Self, RelationError> {
        Self::with_capacity(0)
    }
    /// Open native columns with initial row capacity.
    /// # Errors
    /// Registry or Arrow layout failure.
    pub fn with_capacity(capacity: usize) -> Result<Self, RelationError> {
        Self::with_registry(pse_schema::registry()?, capacity)
    }
    /// Bind once to an exact immutable declaration owner.
    /// # Errors
    /// Incompatible generated declaration, schema or capacity.
    pub fn with_registry(
        registry: &pse_schema::Registry,
        capacity: usize,
    ) -> Result<Self, RelationError> {
        let spec = R::relation(registry)?;
        let schema = pse_schema::arrow::relation_schema_ref(registry, spec)?;
        Ok(Self {
            columns: super::BatchBuilder::new(
                registry,
                registry.contract(spec)?,
                schema,
                capacity,
            )?,
            marker: std::marker::PhantomData,
        })
    }
    /// Number of appended rows.
    pub const fn len(&self) -> usize {
        self.columns.len()
    }
    /// Whether no rows have been appended.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Append typed values without a JSON intermediate.
    /// # Errors
    /// Encoding or buffer growth failure.
    pub fn push(&mut self, row: R) -> Result<(), RelationError> {
        let result = self.columns.append(|columns| row.append_columns(columns));
        drop(row);
        result
    }
    /// Append checked arrays directly with their exact relation association.
    /// # Errors
    /// A foreign field contract or invalid Arrow extent.
    pub fn append_view(&mut self, view: &impl RelationView<Row = R>) -> Result<(), RelationError> {
        self.columns.append_batch(view.batch())
    }
    /// Establish complete local field evidence over the completed columns.
    /// # Errors
    /// Layout or visible value validation fails; prior append failure remains fatal.
    pub fn finish(self) -> Result<FieldCheckedBatch, RelationError> {
        self.columns.finish()
    }
}

pub(crate) fn allocation_add(left: usize, right: usize) -> Result<usize, RelationError> {
    left.checked_add(right)
        .ok_or_else(|| super::mismatch("representable generated payload extent"))
}
