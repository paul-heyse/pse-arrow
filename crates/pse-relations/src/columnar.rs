// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Arrow construction and borrowed access for generated relation interfaces.
//!
//! Construction establishes physical layout and the generated local value contracts.
//! It does not establish primary keys, foreign keys, ordinal ranges or domain rules.

mod codec;
mod collection;
mod row;
mod storage;
pub use collection::Collection;
pub use row::RelationRow;
pub(crate) use row::allocation_add;

use std::sync::Arc;

use arrow_array::builder::ArrayBuilder;
use arrow_array::{Array, RecordBatch, RecordBatchOptions};
use arrow_schema::SchemaRef;
use pse_ids::SemanticId;

use crate::RelationError;

pub use codec::ArrowValue;
pub(crate) use codec::{append_string, read_string};

/// A stable reference to a column in a generated relation declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColumnReference {
    /// The relation whose declaration contains this field.
    pub relation_id: SemanticId,
    /// The declared field name.
    pub name: &'static str,
    /// The field's position in that exact schema, never an artifact row ordinal.
    pub position: usize,
}

/// A batch whose physical layout, fields and local values have been checked.
///
/// The capability is minted by generated construction or by raw admission under the
/// actual registry declaration. This is local field evidence only: a candidate may still
/// contain duplicate keys, out-of-range ordinals or unresolved references.
#[derive(Clone, Debug)]
pub struct FieldCheckedBatch {
    relation_id: SemanticId,
    declaration: Arc<str>,
    batch: RecordBatch,
    // Set only after this module attaches an allocation lease, or after an operation
    // that returns leased storage. This records ownership, not semantic validity.
    leased: bool,
}

impl FieldCheckedBatch {
    /// Isolate caller-owned raw buffers and admit their values once under the exact
    /// declaration. Both the copy and validation scratch are reserved before use;
    /// the returned immutable buffers retain only their own allocation claim.
    /// # Errors
    /// Declaration/value failure, cancellation, resource exhaustion or invalid storage.
    pub fn admit_external(
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        batch: &RecordBatch,
        reserver: &dyn pse_ids::MemoryReserver,
        cancel: &pse_ids::CancellationToken,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        let mut scratch = reserver.open("relations:raw-admission");
        scratch
            .try_grow(pse_ids::validation_extent(batch)?)
            .map_err(pse_ids::CanonError::from)?;
        cancel.checkpoint()?;
        let isolated = pse_ids::owned_buffer::copy_batch(batch, reserver, cancel)?;
        let mut input = Self::admit(registry, spec, isolated)?;
        cancel.checkpoint()?;
        input.leased = true;
        Ok(input)
    }

    /// Admit exact relation fields projected from a wider owned native result.
    /// Native intermediate schemas do not retain relation-level headers. This
    /// operation checks each complete field, attaches only the declared relation
    /// header and runs local value admission. It never rewrites field meaning.
    /// # Errors
    /// Missing or incompatible fields, invalid local values or a different declaration.
    pub fn admit_owned_projection(
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        batch: &pse_ids::owned_buffer::OwnedRecordBatch,
        positions: &[usize],
    ) -> Result<Self, RelationError> {
        let projected = batch.project(positions)?;
        let schema = pse_schema::arrow::relation_schema(registry, spec)?;
        if projected.schema().fields() != schema.fields() {
            return Err(mismatch("exact declared native projection fields"));
        }
        Self::admit_owned(
            registry,
            spec,
            projected.with_schema_metadata(schema.metadata().clone())?,
        )
    }

    /// Check external native values once while transferring their existing result
    /// allocation claim. Ownership alone never establishes local field validity.
    /// # Errors
    /// A different declaration, physical layout or invalid local value.
    pub fn admit_owned(
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        batch: pse_ids::owned_buffer::OwnedRecordBatch,
    ) -> Result<Self, RelationError> {
        let mut checked = Self::admit(registry, spec, batch.into_batch())?;
        checked.leased = true;
        Ok(checked)
    }

    /// Admits one raw candidate under its exact authoritative relation declaration.
    /// Local field and extension predicates execute once before this capability exists.
    ///
    /// # Errors
    /// A different registry declaration, physical layout, field or local value violation.
    pub fn admit(
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        batch: RecordBatch,
    ) -> Result<Self, RelationError> {
        let declaration = declaration(registry, spec)?;
        crate::validate::validate_batch(registry, spec, &batch)
            .map_err(|errors| RelationError::Validation { errors })?;
        Ok(Self {
            relation_id: spec.id,
            declaration: declaration.into(),
            batch,
            leased: false,
        })
    }

    /// Concatenates actual checked columns under one exact relation declaration.
    /// This preserves local field validity without rescanning values. Keys and other
    /// relational obligations remain open, including duplicates introduced by concatenation.
    ///
    /// # Errors
    /// A different declaration, incompatible Arrow fields or concatenation failure.
    pub fn concat(
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        inputs: &[Self],
    ) -> Result<Self, RelationError> {
        let declaration = declaration(registry, spec)?;
        let schema = Arc::new(pse_schema::arrow::relation_schema(registry, spec)?);
        for input in inputs {
            input.for_declaration(spec.id, &declaration)?;
        }
        let batch = arrow::compute::concat_batches(&schema, inputs.iter().map(Self::batch))?;
        Ok(Self {
            relation_id: spec.id,
            declaration: declaration.into(),
            batch,
            leased: false,
        })
    }

    /// Concatenate checked Arrow inputs while retaining the shared allocation claim.
    /// Reservation precedes Arrow allocation. A single input reuses its immutable
    /// columns; empty input still constructs the exact declared empty relation.
    /// This transfers local field validity only, leaving relational obligations open.
    ///
    /// # Errors
    /// Declaration mismatch, checked allocation overflow, cancellation, resource
    /// exhaustion or Arrow concatenation failure.
    pub fn concat_reserved(
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        inputs: &[Self],
        reserver: &dyn pse_ids::MemoryReserver,
        cancel: &pse_ids::CancellationToken,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        if let [input] = inputs {
            input.check_declaration(registry, spec)?;
            return input.retained(reserver, cancel);
        }
        // Includes dictionary unification and offset/null-buffer construction scratch.
        // It is an allocation forecast, never a row or semantic admission claim.
        let arrays = inputs
            .iter()
            .try_fold(0usize, |bytes, input| {
                bytes.checked_add(input.batch.get_array_memory_size())
            })
            .and_then(|bytes| bytes.checked_mul(8))
            .ok_or_else(|| mismatch("a representable concatenation extent"))?;
        let declaration_bytes = inputs
            .iter()
            .map(|input| input.declaration.len())
            .max()
            .unwrap_or(0);
        let schema = spec
            .columns
            .len()
            .checked_mul(4096)
            .and_then(|bytes| bytes.checked_add(declaration_bytes.checked_mul(4)?))
            .ok_or_else(|| mismatch("a representable concatenation schema extent"))?;
        let mut reservation = reserver.open("relations:checked-concatenation");
        reservation
            .try_grow(
                arrays
                    .checked_add(schema)
                    .ok_or_else(|| mismatch("a representable concatenation extent"))?,
            )
            .map_err(pse_ids::CanonError::from)?;
        let mut result = Self::concat(registry, spec, inputs)?;
        cancel.checkpoint()?;
        result.batch = pse_ids::owned_buffer::attach_reservation(
            result.batch,
            pse_ids::ReservationLease::new(reservation),
        )?;
        result.leased = true;
        Ok(result)
    }

    /// Retains a checked subset of rows without copying or rescanning their values.
    ///
    /// # Errors
    /// The requested row range lies outside this batch.
    pub fn slice(&self, offset: usize, length: usize) -> Result<Self, RelationError> {
        if offset
            .checked_add(length)
            .is_none_or(|end| end > self.batch.num_rows())
        {
            return Err(mismatch("a slice within the checked batch"));
        }
        Ok(Self {
            relation_id: self.relation_id,
            declaration: Arc::clone(&self.declaration),
            batch: self.batch.slice(offset, length),
            leased: self.leased,
        })
    }

    /// Checks the retained complete declaration against the receiving registry.
    /// No array values are inspected again.
    ///
    /// # Errors
    /// The supplied declaration is not authoritative or differs from this owner.
    pub fn check_declaration(
        &self,
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
    ) -> Result<(), RelationError> {
        self.for_declaration(spec.id, &declaration(registry, spec)?)?;
        Ok(())
    }

    /// Retains the same immutable values with the destination's shared buffer leases.
    /// The allocator adapter preserves existing ownership and does not revalidate values.
    ///
    /// # Errors
    /// Cancellation, resource exhaustion or invalid buffer ownership.
    pub fn retained(
        &self,
        reserver: &dyn pse_ids::MemoryReserver,
        cancel: &pse_ids::CancellationToken,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        if self.leased {
            return Ok(self.clone());
        }
        Ok(Self {
            relation_id: self.relation_id,
            declaration: Arc::clone(&self.declaration),
            batch: pse_ids::owned_buffer::export_query_batch(self.batch.clone(), reserver, cancel)?,
            leased: true,
        })
    }

    /// Run the declared canonicalizer and retain its actual sorted values as checked fields.
    /// Local field validity transfers through this known row permutation; no hash or raw
    /// caller-supplied result mints the capability. Other relational obligations remain open.
    /// Sorted values are always requested and transferred out of `CanonicalOutput.sorted`;
    /// its returned identity/count/preimage describe the same completed transformation.
    ///
    /// # Errors
    /// A different declaration, canonical key/value refusal, cancellation or resource failure.
    pub fn canonicalize(
        &self,
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        reserver: &dyn pse_ids::MemoryReserver,
        mut options: pse_ids::CanonicalizeOptions,
    ) -> Result<(Self, pse_ids::CanonicalOutput), RelationError> {
        self.check_declaration(registry, spec)?;
        let contract = crate::canonical::contract(registry, spec)?;
        options.keep_sorted = true;
        let mut output = pse_ids::canonicalize(
            &contract,
            std::slice::from_ref(&self.batch),
            reserver,
            options,
        )?;
        let batch = output
            .sorted
            .take()
            .ok_or_else(|| RelationError::Contract {
                relation: spec.qualified_name(),
                reason: "canonicalizer did not return the requested sorted values".into(),
            })?;
        Ok((
            Self {
                relation_id: self.relation_id,
                declaration: Arc::clone(&self.declaration),
                batch,
                leased: true,
            },
            output,
        ))
    }

    /// The declaration used to construct the batch.
    pub const fn relation_id(&self) -> SemanticId {
        self.relation_id
    }

    /// Borrows the immutable Arrow columns, retaining their owners.
    pub const fn batch(&self) -> &RecordBatch {
        &self.batch
    }

    /// Extracts a raw candidate batch; relational admission remains an explicit boundary.
    pub fn into_batch(self) -> RecordBatch {
        self.batch
    }

    pub(crate) fn for_declaration(
        &self,
        relation_id: SemanticId,
        declaration: &str,
    ) -> Result<&RecordBatch, RelationError> {
        if self.relation_id != relation_id || self.declaration.as_ref() != declaration {
            return Err(RelationError::Contract {
                relation: relation_id.to_string(),
                reason: "checked batch belongs to a different complete relation declaration"
                    .to_owned(),
            });
        }
        Ok(&self.batch)
    }
}

fn declaration<'a>(
    registry: &'a pse_schema::Registry,
    spec: &pse_schema::model::RelationSpec,
) -> Result<std::borrow::Cow<'a, str>, RelationError> {
    let actual =
        registry
            .relation_by_id(spec.id)
            .ok_or_else(|| RelationError::UnknownRegistry {
                relation: spec.qualified_name(),
            })?;
    // The immutable registry already owns this exact lossless projection. Borrow
    // it for actual declarations; the registry reconstructs external candidates
    // independently, so matching IDs or fingerprints still confer no authority.
    let declared = registry.compiled_declaration(actual)?;
    if registry.compiled_declaration(spec)? != declared {
        return Err(RelationError::Contract {
            relation: spec.qualified_name(),
            reason: "candidate declaration differs from the authoritative registry".to_owned(),
        });
    }
    Ok(declared)
}

/// Downcasts a column after its owning view checked the declared Arrow layout.
pub(crate) fn array<T: Array + 'static>(array: &dyn Array) -> Result<&T, RelationError> {
    array
        .as_any()
        .downcast_ref::<T>()
        .ok_or_else(|| mismatch(std::any::type_name::<T>()))
}

pub(crate) fn builder<T: ArrayBuilder + 'static>(
    builder: &mut dyn ArrayBuilder,
) -> Result<&mut T, RelationError> {
    builder
        .as_any_mut()
        .downcast_mut::<T>()
        .ok_or_else(|| mismatch(std::any::type_name::<T>()))
}

pub(crate) fn mismatch(expected: &str) -> RelationError {
    RelationError::Contract {
        relation: "generated Arrow codec".to_owned(),
        reason: format!("actual layout does not match {expected}"),
    }
}

pub(crate) fn visible(array: &dyn Array, index: usize) -> Result<(), RelationError> {
    if index >= array.len() || array.is_null(index) {
        return Err(mismatch("a visible row within the array"));
    }
    Ok(())
}

pub(crate) fn value_error(field: &str, row: usize, reason: &str) -> RelationError {
    RelationError::Value {
        field: field.to_owned(),
        row,
        reason: reason.to_owned(),
    }
}

/// Only generated code can append typed values and mint construction evidence.
pub(crate) struct BatchBuilder {
    relation_id: SemanticId,
    declaration: &'static str,
    schema: SchemaRef,
    columns: Vec<Box<dyn ArrayBuilder>>,
    chunks: Vec<RecordBatch>,
    rows: usize,
    pending: usize,
    poisoned: bool,
}

impl std::fmt::Debug for BatchBuilder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BatchBuilder")
            .field("relation_id", &self.relation_id)
            .field("rows", &self.rows)
            .field("pending", &self.pending)
            .field("chunks", &self.chunks.len())
            .field("poisoned", &self.poisoned)
            .finish_non_exhaustive()
    }
}

impl BatchBuilder {
    pub(crate) fn new(
        relation_id: SemanticId,
        declaration: &'static str,
        schema: SchemaRef,
        capacity: usize,
    ) -> Result<Self, RelationError> {
        let columns = schema
            .fields()
            .iter()
            .map(|field| storage::make(field.data_type(), capacity))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            relation_id,
            declaration,
            schema,
            columns,
            chunks: Vec::new(),
            rows: 0,
            pending: 0,
            poisoned: false,
        })
    }

    pub(crate) const fn len(&self) -> usize {
        self.rows
    }

    pub(crate) fn append(
        &mut self,
        append: impl FnOnce(&mut [Box<dyn ArrayBuilder>]) -> Result<(), RelationError>,
    ) -> Result<(), RelationError> {
        self.check()?;
        let rows = self
            .rows
            .checked_add(1)
            .ok_or_else(|| mismatch("representable row count"))?;
        // Scalar checks precede this call. Any storage failure invalidates the builder,
        // so a partially appended struct/list can never be finished as a valid batch.
        if let Err(error) = append(&mut self.columns) {
            self.poisoned = true;
            return Err(error);
        }
        self.rows = rows;
        self.pending += 1;
        Ok(())
    }

    pub(crate) fn append_batch(&mut self, batch: &RecordBatch) -> Result<(), RelationError> {
        self.check()?;
        if batch.schema().fields() != self.schema.fields() {
            return Err(mismatch("the generated bulk-append field contract"));
        }
        let rows = self
            .rows
            .checked_add(batch.num_rows())
            .ok_or_else(|| mismatch("representable row count"))?;
        if batch.num_rows() != 0 {
            self.flush()?;
            // Retain Arrow arrays directly. A later finish uses Arrow concatenation,
            // including dictionary unification, rather than materializing rows.
            self.chunks.push(batch.clone());
        }
        self.rows = rows;
        Ok(())
    }

    fn check(&self) -> Result<(), RelationError> {
        if self.poisoned {
            return Err(mismatch("an unfailed Arrow builder"));
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), RelationError> {
        if self.pending != 0 {
            let columns = self.columns.iter_mut().map(ArrayBuilder::finish).collect();
            let options = RecordBatchOptions::new().with_row_count(Some(self.pending));
            self.chunks.push(RecordBatch::try_new_with_options(
                Arc::clone(&self.schema),
                columns,
                &options,
            )?);
            self.pending = 0;
        }
        Ok(())
    }

    pub(crate) fn finish(mut self) -> Result<FieldCheckedBatch, RelationError> {
        self.check()?;
        self.flush()?;
        let batch = match self.chunks.len() {
            0 => RecordBatch::new_empty(Arc::clone(&self.schema)),
            1 => {
                let batch = self
                    .chunks
                    .pop()
                    .ok_or_else(|| mismatch("one retained Arrow chunk"))?;
                // Contextual metadata belongs to the destination construction.
                RecordBatch::try_new(Arc::clone(&self.schema), batch.columns().to_vec())?
            }
            _ => arrow::compute::concat_batches(&self.schema, &self.chunks)?,
        };
        Ok(FieldCheckedBatch {
            relation_id: self.relation_id,
            declaration: self.declaration.into(),
            batch,
            leased: false,
        })
    }
}
