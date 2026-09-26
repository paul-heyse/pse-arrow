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
pub(crate) use collection::encode_rows;
pub(crate) use row::allocation_add;
pub use row::{RelationRow, RelationView, RowBuilder};

use std::sync::Arc;

use arrow_array::builder::ArrayBuilder;
use arrow_array::{Array, RecordBatch, RecordBatchOptions};
use arrow_schema::SchemaRef;
use pse_ids::SemanticId;

use crate::RelationError;

pub use codec::{ArrowValue, array_from_values};
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
    contract: pse_schema::resolved_contract::RelationContractHandle,
    batch: RecordBatch,
    // Actual allocation provenance is independent of the local field contract.
    owned: Option<pse_columnar::owned_buffer::OwnedRecordBatch>,
}

#[cfg(test)]
mod foundation_unit {
    #![allow(clippy::unwrap_used, reason = "isolated checked-transform fixture")]
    use super::*;
    use pse_schema::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};
    #[test]
    fn checked_selection_and_strict_cast_reestablish_only_local_evidence() {
        let mut builder = pse_schema::RegistryBuilder::new();
        for (name, ty) in [
            ("text", arrow_schema::DataType::Utf8),
            ("number", arrow_schema::DataType::Int64),
        ] {
            builder.declare_relation(
                RelationDecl::new(
                    Namespace::Authored,
                    name,
                    1,
                    Authority::Authored,
                    SnapshotClass::Model,
                    "test",
                )
                .columns(vec![FieldContract::key(
                    "value",
                    FieldContract::native(ty),
                    "value",
                )])
                .pk(&["value"]),
            );
        }
        let registry = builder.build().unwrap();
        let text = registry.relation("authored.text").unwrap();
        let number = registry.relation("authored.number").unwrap();
        let input = |values| {
            FieldCheckedBatch::admit(
                &registry,
                text,
                RecordBatch::try_new(
                    pse_schema::arrow::relation_schema_ref(&registry, text).unwrap(),
                    vec![Arc::new(arrow_array::StringArray::from(values))],
                )
                .unwrap(),
            )
            .unwrap()
        };
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(1_000_000));
        let cancel = pse_columnar::CancellationToken::new();
        let source = input(vec!["1", "2"]);
        let taken = source
            .take_reserved(
                &arrow_array::UInt32Array::from(vec![1, 0, 1]),
                &pool,
                &cancel,
            )
            .unwrap();
        assert_eq!(
            taken.batch().num_rows(),
            3,
            "duplicates are valid local values, not PK evidence"
        );
        assert!(
            source
                .take_reserved(&arrow_array::UInt32Array::from(vec![2]), &pool, &cancel)
                .is_err()
        );
        assert!(
            source
                .take_reserved(&arrow_array::UInt32Array::from(vec![None]), &pool, &cancel)
                .is_err()
        );
        let cast = taken
            .cast_readmit_reserved(&registry, number, &pool, &cancel)
            .unwrap();
        assert_eq!(
            cast.batch()
                .column(0)
                .as_any()
                .downcast_ref::<arrow_array::Int64Array>()
                .unwrap(),
            &arrow_array::Int64Array::from(vec![2, 1, 2])
        );
        assert!(
            input(vec!["bad"])
                .cast_readmit_reserved(&registry, number, &pool, &cancel)
                .is_err()
        );
        let reader = cast.slice(1, 1).unwrap();
        drop(cast);
        drop(taken);
        assert!(pool.reserved() > 0);
        drop(reader);
        assert_eq!(pool.reserved(), 0);
        cancel.cancel();
        assert!(
            source
                .take_reserved(&arrow_array::UInt32Array::from(vec![0]), &pool, &cancel)
                .is_err()
        );
    }
}

impl FieldCheckedBatch {
    /// Admit through the actual immutable native function/configuration owner.
    /// This carries the same local certificate as `admit`, without substituting
    /// a registry-default SQL environment for the selected session.
    /// # Errors
    /// Foreign context, schema/value failure or cancellation.
    pub fn admit_in(
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        batch: RecordBatch,
        context: &crate::validate::ValidationContext,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, RelationError> {
        let contract = registry.contract(spec)?;
        crate::validate::validate_schema(registry, spec, &batch.schema())
            .map_err(|errors| RelationError::Validation { errors })?;
        context
            .relation(registry, spec)?
            .evaluate(&batch, 256, cancel)?
            .require_valid()?;
        Ok(Self {
            relation_id: spec.id,
            contract,
            batch,
            owned: None,
        })
    }
    /// Transfer native allocation ownership while admitting in the exact selected context.
    /// # Errors
    /// Foreign context, schema/value failure or cancellation.
    pub fn admit_owned_in(
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        batch: pse_columnar::owned_buffer::OwnedRecordBatch,
        context: &crate::validate::ValidationContext,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, RelationError> {
        let mut checked = Self::admit_in(registry, spec, batch.batch().clone(), context, cancel)?;
        checked.owned = Some(batch);
        Ok(checked)
    }
    /// Select ordered, possibly repeated rows with Arrow's bounds-checked take.
    /// Local field evidence survives; uniqueness and completeness do not.
    /// # Errors
    /// Null/out-of-range indices, allocation refusal or cancellation.
    pub fn take_reserved(
        &self,
        indices: &arrow_array::UInt32Array,
        pool: &Arc<dyn pse_columnar::MemoryPool>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        if indices.null_count() != 0 {
            return Err(mismatch("nonnull take indices"));
        }
        let extent = pse_columnar::allocation_extent::algorithm_decode_extent(&self.batch)?
            .checked_mul(indices.len().max(1))
            .ok_or_else(|| mismatch("bounded take extent"))?;
        let reservation =
            pse_columnar::MemoryConsumer::new("relations:checked-take").register(pool);
        reservation
            .try_grow(extent)
            .map_err(pse_columnar::CanonError::from)?;
        let columns = self
            .batch
            .columns()
            .iter()
            .map(|column| {
                arrow::compute::take(
                    column.as_ref(),
                    indices,
                    Some(arrow::compute::TakeOptions { check_bounds: true }),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let batch = RecordBatch::try_new_with_options(
            self.batch.schema(),
            columns,
            &RecordBatchOptions::new().with_row_count(Some(indices.len())),
        )?;
        cancel.checkpoint()?;
        let scope = pse_columnar::owned_buffer::AllocationScope::default();
        if let Some(owned) = &self.owned {
            scope.import(owned)?;
        }
        let owned = scope.attach_reserved(batch, reservation)?;
        Ok(Self {
            batch: owned.batch().clone(),
            owned: Some(owned),
            ..self.clone()
        })
    }
    /// Apply an explicitly requested Arrow type conversion, then fully re-admit
    /// the target's fields and local values. Cast failure is an error, never a new null.
    /// Arrow's numerical conversion semantics apply; this is not a lossless-cast claim.
    /// # Errors
    /// Target shape/name mismatch, conversion or value failure, resource refusal or cancellation.
    pub fn cast_readmit_reserved(
        &self,
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        pool: &Arc<dyn pse_columnar::MemoryPool>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        let schema = pse_schema::arrow::relation_schema_ref(registry, spec)?;
        if !schema.fields().iter().map(|field| field.name()).eq(self
            .batch
            .schema()
            .fields()
            .iter()
            .map(|field| field.name()))
        {
            return Err(mismatch("declared cast column names and arity"));
        }
        let reservation =
            pse_columnar::MemoryConsumer::new("relations:cast-readmission").register(pool);
        reservation
            .try_grow(pse_columnar::allocation_extent::algorithm_decode_extent(
                &self.batch,
            )?)
            .map_err(pse_columnar::CanonError::from)?;
        let options = arrow::compute::CastOptions {
            safe: false,
            ..Default::default()
        };
        let columns = self
            .batch
            .columns()
            .iter()
            .zip(schema.fields())
            .map(|(column, field)| {
                arrow::compute::cast_with_options(column.as_ref(), field.data_type(), &options)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let batch = RecordBatch::try_new_with_options(
            schema,
            columns,
            &RecordBatchOptions::new().with_row_count(Some(self.batch.num_rows())),
        )?;
        cancel.checkpoint()?;
        let scope = pse_columnar::owned_buffer::AllocationScope::default();
        if let Some(owned) = &self.owned {
            scope.import(owned)?;
        }
        Self::admit_owned(registry, spec, scope.attach_reserved(batch, reservation)?)
    }
    /// Compare the exact checked row domain and retained semantic authority.
    /// Equal values in different arrays are not the same source owner.
    pub fn same_source(&self, other: &Self) -> bool {
        self.contract.require_equivalent(&other.contract).is_ok()
            && self.batch.schema() == other.batch.schema()
            && self.batch.num_rows() == other.batch.num_rows()
            && self.batch.num_columns() == other.batch.num_columns()
            && self
                .batch
                .columns()
                .iter()
                .zip(other.batch.columns())
                .all(|(left, right)| Arc::ptr_eq(left, right))
    }

    /// Actual owned storage, when allocation admission has completed.
    pub const fn owned(&self) -> Option<&pse_columnar::owned_buffer::OwnedRecordBatch> {
        self.owned.as_ref()
    }

    /// Filter checked rows with native Arrow selection, preserving local evidence.
    /// False and null mask entries are excluded. Keys and completeness remain open.
    /// # Errors
    /// Mask length, allocation admission, cancellation or Arrow representation failure.
    pub fn filter_reserved(
        &self,
        mask: &arrow_array::BooleanArray,
        pool: &Arc<dyn pse_columnar::MemoryPool>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        if mask.len() != self.batch.num_rows() {
            return Err(mismatch("a filter mask in the checked row domain"));
        }
        let extent = self
            .batch
            .get_array_memory_size()
            .checked_mul(8)
            .and_then(|n| n.checked_add(mask.len().checked_mul(16)?))
            .and_then(|n| n.checked_add(self.batch.schema().fields().len().checked_mul(256)?))
            .ok_or_else(|| mismatch("a representable filter allocation extent"))?;
        let reservation =
            pse_columnar::MemoryConsumer::new("relations:checked-filter").register(pool);
        reservation
            .try_grow(extent)
            .map_err(pse_columnar::CanonError::from)?;
        let predicate = arrow::compute::FilterBuilder::new(mask).build();
        let batch = predicate.filter_record_batch(&self.batch)?;
        cancel.checkpoint()?;
        let scope = pse_columnar::owned_buffer::AllocationScope::default();
        if let Some(owned) = &self.owned {
            scope.import(owned)?;
        }
        let owned = scope.attach_reserved(batch, reservation)?;
        Ok(Self {
            batch: owned.batch().clone(),
            owned: Some(owned),
            ..self.clone()
        })
    }

    /// Project unchanged fields into an exact target declaration. Only missing
    /// target row checks execute; field evidence and allocation owners survive.
    /// # Errors
    /// Foreign semantic definitions, incompatible fields, missing values or failed checks.
    pub fn project_exact(
        &self,
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        positions: &[usize],
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        registry.admit_contract(&self.contract)?;
        let contract = registry.contract(spec)?;
        let projected = self.batch.project(positions)?;
        exact_fields(projected.schema().fields(), contract.schema().fields())?;
        let batch = RecordBatch::try_new_with_options(
            contract.schema().clone(),
            projected.columns().to_vec(),
            &RecordBatchOptions::new().with_row_count(Some(projected.num_rows())),
        )?;
        let missing = spec
            .checks
            .iter()
            .filter(|(_, sql)| {
                !self
                    .contract
                    .resolved()
                    .declaration
                    .checks
                    .values()
                    .any(|known| known == *sql)
            })
            .map(|(name, _)| format!("check:{name}"))
            .collect::<std::collections::BTreeSet<_>>();
        if !missing.is_empty() {
            crate::validate::ValidationContext::for_registry(registry)?
                .relation(registry, spec)?
                .evaluate_missing_checks(&batch, &missing, cancel)?
                .require_valid()?;
        }
        Ok(Self {
            relation_id: spec.id,
            contract: contract.clone(),
            batch,
            owned: self
                .owned
                .as_ref()
                .map(|owned| {
                    owned.project(positions).and_then(|owned| {
                        owned.with_schema_metadata(contract.schema().metadata().clone())
                    })
                })
                .transpose()?,
        })
    }

    /// Isolate caller-owned raw buffers and admit their values once under the exact
    /// declaration. Both the copy and validation scratch are reserved before use;
    /// the returned immutable buffers retain only their own allocation claim.
    /// # Errors
    /// Declaration/value failure, cancellation, resource exhaustion or invalid storage.
    pub fn admit_external(
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        batch: &RecordBatch,
        pool: &Arc<dyn pse_columnar::MemoryPool>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, RelationError> {
        Self::admit_external_in(
            registry,
            spec,
            batch,
            crate::validate::ValidationContext::for_registry(registry)?.as_ref(),
            pool,
            cancel,
        )
    }

    /// Isolate raw buffers and admit through the selected immutable engine context.
    /// The copy and validation scratch retain the same bounds as registry admission.
    /// # Errors
    /// Foreign context, invalid fields/values, cancellation or allocation refusal.
    pub fn admit_external_in(
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        batch: &RecordBatch,
        context: &crate::validate::ValidationContext,
        pool: &Arc<dyn pse_columnar::MemoryPool>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        let scratch = pse_columnar::MemoryConsumer::new("relations:raw-admission").register(pool);
        scratch
            .try_grow(pse_columnar::columnar_validation_extent(batch)?)
            .map_err(pse_columnar::CanonError::from)?;
        cancel.checkpoint()?;
        let isolated = pse_columnar::owned_buffer::OwnedRecordBatch::copy(batch, pool, cancel)?;
        let input = Self::admit_owned_in(registry, spec, isolated, context, cancel)?;
        cancel.checkpoint()?;
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
        batch: &pse_columnar::owned_buffer::OwnedRecordBatch,
        positions: &[usize],
    ) -> Result<Self, RelationError> {
        let projected = batch.project(positions)?;
        let schema = pse_schema::arrow::relation_schema_ref(registry, spec)?;
        exact_fields(projected.schema().fields(), schema.fields())?;
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
        batch: pse_columnar::owned_buffer::OwnedRecordBatch,
    ) -> Result<Self, RelationError> {
        let mut checked = Self::admit(registry, spec, batch.batch().clone())?;
        checked.owned = Some(batch);
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
        let contract = registry.contract(spec)?;
        crate::validate::validate_batch(registry, spec, &batch)
            .map_err(|errors| RelationError::Validation { errors })?;
        Ok(Self {
            relation_id: spec.id,
            contract,
            batch,
            owned: None,
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
        let contract = registry.contract(spec)?;
        let actual = registry
            .relation_by_id(contract.relation_id())
            .ok_or_else(|| mismatch("the admitted registry relation"))?;
        let schema = pse_schema::arrow::relation_schema_ref(registry, actual)?;
        for input in inputs {
            input.contract.require_equivalent(&contract)?;
            exact_fields(input.batch.schema().fields(), schema.fields())?;
        }
        let batch = arrow::compute::concat_batches(&schema, inputs.iter().map(Self::batch))?;
        Ok(Self {
            relation_id: spec.id,
            contract,
            batch,
            owned: None,
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
        pool: &Arc<dyn pse_columnar::MemoryPool>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        if let [input] = inputs {
            input.check_declaration(registry, spec)?;
            return input.rebind(registry)?.retained(pool, cancel);
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
        let schema = spec
            .columns
            .iter()
            .try_fold(0usize, |bytes, column| {
                bytes.checked_add(column.field().size())
            })
            .and_then(|bytes| bytes.checked_mul(4))
            .and_then(|bytes| bytes.checked_add(size_of::<Self>()))
            .ok_or_else(|| mismatch("a representable concatenation schema extent"))?;
        let reservation =
            pse_columnar::MemoryConsumer::new("relations:checked-concatenation").register(pool);
        reservation
            .try_grow(
                arrays
                    .checked_add(schema)
                    .ok_or_else(|| mismatch("a representable concatenation extent"))?,
            )
            .map_err(pse_columnar::CanonError::from)?;
        let mut result = Self::concat(registry, spec, inputs)?;
        cancel.checkpoint()?;
        let scope = pse_columnar::owned_buffer::AllocationScope::default();
        for input in inputs {
            if let Some(owned) = &input.owned {
                scope.import(owned)?;
            }
        }
        let owned = scope.attach_reserved(result.batch, reservation)?;
        result.batch = owned.batch().clone();
        result.owned = Some(owned);
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
            contract: self.contract.clone(),
            batch: self.batch.slice(offset, length),
            owned: self
                .owned
                .as_ref()
                .map(|owned| owned.slice(offset, length))
                .transpose()?,
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
        self.contract
            .require_equivalent(&registry.contract(spec)?)?;
        Ok(())
    }

    /// Retains the same immutable values with the destination's shared buffer leases.
    /// The allocator adapter preserves existing ownership and does not revalidate values.
    ///
    /// # Errors
    /// Cancellation, resource exhaustion or invalid buffer ownership.
    pub fn retained(
        &self,
        pool: &Arc<dyn pse_columnar::MemoryPool>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        if self.owned.is_some() {
            return Ok(self.clone());
        }
        let owned =
            pse_columnar::owned_buffer::OwnedRecordBatch::export(self.batch.clone(), pool, cancel)?;
        Ok(Self {
            relation_id: self.relation_id,
            contract: self.contract.clone(),
            batch: owned.batch().clone(),
            owned: Some(owned),
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
        pool: &Arc<dyn pse_columnar::MemoryPool>,
        mut options: pse_columnar::CanonicalizeOptions,
    ) -> Result<(Self, pse_columnar::CanonicalOutput), RelationError> {
        self.check_declaration(registry, spec)?;
        let contract = crate::canonical::contract(registry, spec)?;
        options.keep_sorted = true;
        let mut output = pse_columnar::canonicalize(
            &contract,
            std::slice::from_ref(&self.batch),
            pool,
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
                contract: self.contract.clone(),
                batch: batch.batch().clone(),
                owned: Some(batch),
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

    /// Bind an already checked batch to an equivalent independent registry.
    /// Physical metadata and allocation owners are retained without adaptation.
    /// # Errors
    /// Any reachable semantic definition differs from the destination.
    pub fn rebind(&self, registry: &pse_schema::Registry) -> Result<Self, RelationError> {
        Ok(Self {
            contract: registry.admit_contract(&self.contract)?,
            ..self.clone()
        })
    }

    pub(crate) fn for_generated(
        &self,
        relation_id: SemanticId,
        expected: &'static [pse_schema::resolved_contract::ExpectedContract],
    ) -> Result<&RecordBatch, RelationError> {
        if self.relation_id != relation_id {
            return Err(mismatch("the requested generated relation identity"));
        }
        self.contract.require_generated(expected)?;
        Ok(&self.batch)
    }
}

fn exact_fields(
    actual: &arrow_schema::Fields,
    expected: &arrow_schema::Fields,
) -> Result<(), RelationError> {
    if actual.len() != expected.len() {
        return Err(mismatch("the complete native field inventory"));
    }
    for (actual, expected) in actual.iter().zip(expected) {
        pse_schema::field_contract::execution_field(actual, expected, expected.name())?;
    }
    Ok(())
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

/// Only generated code can append typed values and mint construction evidence.
pub(crate) struct BatchBuilder {
    prepared: Arc<crate::validate::PreparedLocalContract>,
    relation_id: SemanticId,
    contract: pse_schema::resolved_contract::RelationContractHandle,
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
        registry: &pse_schema::Registry,
        contract: pse_schema::resolved_contract::RelationContractHandle,
        schema: SchemaRef,
        capacity: usize,
    ) -> Result<Self, RelationError> {
        let spec = registry
            .relation_by_id(contract.relation_id())
            .ok_or_else(|| mismatch("declared builder relation"))?;
        let columns = schema
            .fields()
            .iter()
            .map(|field| storage::make(field.data_type(), capacity))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            prepared: crate::validate::ValidationContext::for_registry(registry)?
                .relation(registry, spec)?,
            relation_id: contract.relation_id(),
            contract,
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
        // Native domain checks run on finish. Any storage failure invalidates the builder,
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
        exact_fields(batch.schema().fields(), self.schema.fields())?;
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
                RecordBatch::try_new_with_options(
                    Arc::clone(&self.schema),
                    batch.columns().to_vec(),
                    &RecordBatchOptions::new().with_row_count(Some(batch.num_rows())),
                )?
            }
            _ => arrow::compute::concat_batches(&self.schema, &self.chunks)?,
        };
        self.prepared
            .evaluate(&batch, 256, &pse_columnar::CancellationToken::new())?
            .require_valid()?;
        Ok(FieldCheckedBatch {
            relation_id: self.relation_id,
            contract: self.contract,
            batch,
            owned: None,
        })
    }
}

#[cfg(test)]
mod consolidation_unit {
    use super::*;
    use crate::generated::reference::dimensions;

    #[test]
    fn generated_builders_views_and_foreign_rebinding_retain_native_columns()
    -> Result<(), RelationError> {
        let registry = pse_schema::catalog::assemble()?;
        let mut builder = dimensions::Builder::with_registry(&registry, 1)?;
        builder.push(dimensions::Row {
            ordinal: 0,
            name: "length".into(),
        })?;
        let checked = builder.finish()?;
        for _ in 0..10 {
            assert_eq!(
                dimensions::View::from_checked(&checked)?.rows()?[0].name,
                "length"
            );
        }
        let destination = pse_schema::catalog::assemble()?;
        let rebound = checked.rebind(&destination)?;
        assert!(Arc::ptr_eq(
            checked.batch().column(0),
            rebound.batch().column(0)
        ));
        assert_eq!(checked.batch().schema(), rebound.batch().schema());
        let sliced = rebound.slice(0, 1)?;
        let combined = FieldCheckedBatch::concat(
            &destination,
            dimensions::spec(&destination)?,
            &[checked, sliced],
        )?;
        assert_eq!(dimensions::View::from_checked(&combined)?.rows()?.len(), 2);
        drop(registry);
        drop(destination);
        assert_eq!(dimensions::View::from_checked(&rebound)?.rows()?.len(), 1);
        Ok(())
    }

    #[test]
    fn empty_batches_and_incompatible_generated_views_are_distinct() -> Result<(), RelationError> {
        let checked = dimensions::Builder::new()?.finish()?;
        assert_eq!(dimensions::View::from_checked(&checked)?.rows()?.len(), 0);
        assert!(
            crate::generated::reference::schema_enum_types::View::from_checked(&checked).is_err()
        );
        Ok(())
    }

    #[test]
    fn changed_projection_cannot_reuse_parent_admission() -> Result<(), RelationError> {
        let registry = pse_schema::registry()?;
        let mut builder = dimensions::Builder::new()?;
        builder.push(dimensions::Row {
            ordinal: 0,
            name: "length".into(),
        })?;
        let checked = builder.finish()?;
        let projected = checked.batch().project(&[1])?;
        assert!(
            FieldCheckedBatch::admit(registry, dimensions::spec(registry)?, projected).is_err()
        );
        Ok(())
    }
}

#[cfg(test)]
mod integrated_performance_unit {
    use super::*;
    use arrow_array::{BooleanArray, Int64Array};
    use pse_schema::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};

    fn registry() -> pse_schema::Registry {
        let mut registry = pse_schema::RegistryBuilder::new();
        let field = FieldContract::payload(
            "n",
            FieldContract::native(arrow_schema::DataType::Int64),
            "test value",
        );
        for (name, checks) in [
            ("values", std::collections::BTreeMap::new()),
            (
                "positive",
                std::collections::BTreeMap::from([("positive".into(), "n > 0".into())]),
            ),
        ] {
            registry.declare_relation(
                RelationDecl::new(
                    Namespace::Authored,
                    name,
                    1,
                    Authority::Authored,
                    SnapshotClass::Model,
                    "isolated preservation",
                )
                .pk(&["n"])
                .columns(vec![field.clone()])
                .checks(checks),
            );
        }
        let registry = registry.build().unwrap();
        crate::validate::ValidationContext::install_default(
            &registry,
            datafusion::prelude::SessionContext::new().state(),
        )
        .unwrap();
        registry
    }

    #[test]
    fn exact_source_uses_contract_and_array_owners_not_equal_values() {
        let registry = registry();
        let spec = registry.relation("authored.values").unwrap();
        let batch = || {
            RecordBatch::try_new(
                Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap()),
                vec![Arc::new(Int64Array::from(vec![1, 2]))],
            )
            .unwrap()
        };
        let source = FieldCheckedBatch::admit(&registry, spec, batch()).unwrap();
        assert!(source.same_source(&source.clone()));
        assert!(!source.same_source(&FieldCheckedBatch::admit(&registry, spec, batch()).unwrap()));
        assert!(!source.same_source(&source.slice(0, 1).unwrap()));
    }
    #[test]
    fn checked_handoffs_preserve_evidence_but_not_cross_chunk_keys() {
        let registry = registry();
        let spec = registry.relation("authored.values").unwrap();
        let validation = crate::validate::ValidationContext::for_registry(&registry).unwrap();
        let prepared = validation.relation(&registry, spec).unwrap();
        let batch = RecordBatch::try_new(
            prepared.schema().clone(),
            vec![Arc::new(Int64Array::from(vec![0, 1, 2]))],
        )
        .unwrap();
        let checked = FieldCheckedBatch::admit(&registry, spec, batch).unwrap();
        let calls = prepared.evaluation_count();
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        let cancel = pse_columnar::CancellationToken::new();
        let selected = checked
            .filter_reserved(
                &BooleanArray::from(vec![Some(true), None, Some(false)]),
                &pool,
                &cancel,
            )
            .unwrap();
        assert_eq!(selected.batch().num_rows(), 1);
        let sliced = checked.slice(0, 1).unwrap();
        let projected = sliced
            .project_exact(&registry, spec, &[0], &cancel)
            .unwrap();
        let duplicate = FieldCheckedBatch::concat(&registry, spec, &[selected, projected]).unwrap();
        assert_eq!(duplicate.batch().num_rows(), 2);
        assert_eq!(
            duplicate
                .batch()
                .column(0)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap()
                .values()
                .as_ref(),
            &[0, 0]
        );
        assert_eq!(
            prepared.evaluation_count(),
            calls,
            "local evidence is preserved; duplicate keys remain open"
        );
        assert!(
            checked
                .filter_reserved(&BooleanArray::from(vec![true]), &pool, &cancel)
                .is_err()
        );
        assert!(
            checked
                .project_exact(&registry, spec, &[1], &cancel)
                .is_err()
        );
        let strict = registry.relation("authored.positive").unwrap();
        assert!(
            checked
                .project_exact(&registry, strict, &[0], &cancel)
                .is_err()
        );
        assert!(
            checked
                .slice(1, 2)
                .unwrap()
                .project_exact(&registry, strict, &[0], &cancel)
                .is_ok()
        );
        assert!(Arc::ptr_eq(
            &prepared,
            &validation.relation(&registry, spec).unwrap()
        ));
        assert_eq!(validation.prepared_count().unwrap(), 2);
    }
    #[tokio::test]
    async fn concatenating_checked_chunks_does_not_certify_duplicate_keys() {
        use crate::native::execution::context::SessionContext;
        use crate::validate::obligations::{ObligationTemplates, RelationInputs};
        let registry = registry();
        let spec = registry.relation("authored.values").unwrap();
        let schema = pse_schema::arrow::relation_schema_ref(&registry, spec).unwrap();
        let checked = FieldCheckedBatch::admit(
            &registry,
            spec,
            RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![7]))]).unwrap(),
        )
        .unwrap();
        let context = SessionContext::new();
        for copies in [1, 2] {
            let batch =
                FieldCheckedBatch::concat(&registry, spec, &vec![checked.clone(); copies]).unwrap();
            let input = context
                .read_batch(batch.into_batch())
                .unwrap()
                .into_unoptimized_plan();
            let plans = ObligationTemplates::new(&registry)
                .bind(&RelationInputs::from([(spec.id, input)]), &context.state())
                .unwrap();
            let mut violations = 0;
            for plan in plans {
                violations += context
                    .execute_logical_plan(plan)
                    .await
                    .unwrap()
                    .collect()
                    .await
                    .unwrap()
                    .iter()
                    .map(RecordBatch::num_rows)
                    .sum::<usize>();
            }
            assert_eq!(violations, copies - 1);
        }
    }
}
