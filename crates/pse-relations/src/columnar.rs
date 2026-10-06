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
    storage: Arc<CheckedStorage>,
    export_owner: Option<Arc<pse_columnar::AllocationLease>>,
    admission: Option<LocalFieldAdmission>,
}

/// Successful local admission tied to the actual immutable native predicate owner.
/// It carries no arrays, keys, foreign-key closure or persisted validation authority.
#[derive(Clone, Debug)]
pub struct LocalFieldAdmission {
    contract: pse_schema::resolved_contract::RelationContractHandle,
    prepared: Arc<crate::validate::PreparedLocalContract>,
}
impl LocalFieldAdmission {
    /// Compare the complete declaration and actual native implementation owner.
    /// # Errors
    /// A changed declaration or foreign context is refused.
    pub fn matches(
        &self,
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        context: &crate::validate::ValidationContext,
    ) -> Result<bool, RelationError> {
        self.contract
            .require_equivalent(&registry.contract(spec)?)?;
        Ok(self
            .prepared
            .same_native_owner(context.relation(registry, spec)?.as_ref()))
    }
}

/// Clones share the actual column vectors as well as their native buffers.
#[derive(Debug)]
struct CheckedStorage {
    batch: RecordBatch,
    // Actual allocation provenance is independent of the local field contract.
    owned: Option<pse_columnar::owned_buffer::OwnedRecordBatch>,
    metadata: Option<Arc<pse_columnar::AllocationLease>>,
    pool: Option<std::sync::Weak<dyn pse_columnar::MemoryPool>>,
}
type StorageMetadata = (
    Arc<pse_columnar::AllocationLease>,
    std::sync::Weak<dyn pse_columnar::MemoryPool>,
);

#[cfg(test)]
mod foundation_unit {
    #![allow(clippy::unwrap_used, reason = "isolated checked-transform fixture")]
    use super::*;
    use crate::native::execution::context::SessionContext;
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
                &crate::validate::ValidationContext::new(&registry, SessionContext::new().state()),
                &pse_columnar::CancellationToken::new(),
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
            .cast_readmit_reserved(
                &registry,
                number,
                &pool,
                &cancel,
                &crate::validate::ValidationContext::new(&registry, SessionContext::new().state()),
            )
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
                .cast_readmit_reserved(
                    &registry,
                    number,
                    &pool,
                    &cancel,
                    &crate::validate::ValidationContext::new(
                        &registry,
                        SessionContext::new().state()
                    )
                )
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
    fn from_parts(
        relation_id: SemanticId,
        contract: pse_schema::resolved_contract::RelationContractHandle,
        batch: RecordBatch,
        owned: Option<pse_columnar::owned_buffer::OwnedRecordBatch>,
    ) -> Self {
        Self::from_parts_with_metadata(relation_id, contract, batch, owned, None)
    }

    fn from_parts_with_metadata(
        relation_id: SemanticId,
        contract: pse_schema::resolved_contract::RelationContractHandle,
        batch: RecordBatch,
        owned: Option<pse_columnar::owned_buffer::OwnedRecordBatch>,
        metadata: Option<StorageMetadata>,
    ) -> Self {
        let (metadata, pool) =
            metadata.map_or((None, None), |(lease, pool)| (Some(lease), Some(pool)));
        Self {
            relation_id,
            contract,
            storage: Arc::new(CheckedStorage {
                batch,
                owned,
                metadata,
                pool,
            }),
            export_owner: None,
            admission: None,
        }
    }

    fn with_admission(mut self, admission: Option<LocalFieldAdmission>) -> Self {
        self.admission = admission;
        self
    }
    /// Borrow the native owner of this batch's successful local admission.
    pub fn local_admission(&self) -> Option<&LocalFieldAdmission> {
        self.admission.as_ref()
    }
    /// Recheck local values only when their actual native owner changed.
    /// Raw admission always evaluates; this operation consumes retained immutable evidence.
    /// # Errors
    /// Declaration/context mismatch, native refusal or cancellation.
    pub fn validate_context(
        &self,
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        context: &crate::validate::ValidationContext,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<(), RelationError> {
        self.readmit_context(registry, spec, context, cancel)
            .map(|_| ())
    }
    /// Retain these immutable values with successful admission in the receiving owner.
    /// # Errors
    /// Declaration/context mismatch, native refusal or cancellation.
    pub fn readmit_context(
        &self,
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        context: &crate::validate::ValidationContext,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        self.check_declaration(registry, spec)?;
        if self
            .admission
            .as_ref()
            .map(|admission| admission.matches(registry, spec, context))
            .transpose()?
            .unwrap_or(false)
        {
            return Ok(self.clone());
        }
        let prepared = context.relation(registry, spec)?;
        prepared
            .evaluate(self.batch(), 256, cancel)?
            .require_valid()?;
        cancel.checkpoint()?;
        Ok(self.clone().with_admission(Some(LocalFieldAdmission {
            contract: self.contract.clone(),
            prepared,
        })))
    }

    /// Retain a pre-admitted container's metadata with its checked exports.
    /// Cloning the checked batch shares its storage and this owner without allocating
    /// another column vector. The caller reserves the container before constructing it.
    pub fn with_export_owner(mut self, owner: Arc<pse_columnar::AllocationLease>) -> Self {
        self.export_owner = Some(owner);
        self
    }

    /// Export Arrow arrays retaining the original checked storage, container owner
    /// and pre-admitted wrapper metadata until the last escaped buffer drops.
    /// Values and their existing backing allocation leases are shared unchanged.
    /// # Errors
    /// Cancellation, metadata reservation refusal, wholly bufferless storage
    /// (which cannot retain an escaped owner), or Arrow representation failure.
    pub fn checked_export(
        &self,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<RecordBatch, RelationError> {
        #[derive(Debug)]
        struct ExportOwner {
            _checked: FieldCheckedBatch,
            _metadata: Option<StorageMetadata>,
        }
        cancel.checkpoint()?;
        let units = pse_columnar::owned_buffer::payload_wrapper_units(self.batch())?;
        // Express conservative wrapper bytes in the existing transform metadata
        // reservation's units of four ArrayRef slots, independent of pointer size.
        let columns = units
            .checked_mul(256)
            .map(|bytes| bytes.div_ceil(4 * size_of::<arrow_array::ArrayRef>()))
            .and_then(|n| n.checked_add(self.batch().num_columns()))
            .ok_or_else(|| mismatch("checked export metadata extent"))?;
        let metadata = self.transform_metadata(columns, cancel)?;
        let owner = Arc::new(ExportOwner {
            _checked: self.clone(),
            _metadata: metadata,
        });
        Ok(pse_columnar::owned_buffer::retain_payload(
            self.batch(),
            owner,
            cancel,
        )?)
    }

    fn transform_metadata(
        &self,
        columns: usize,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Option<StorageMetadata>, RelationError> {
        self.storage
            .pool
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
            .map(|pool| Self::reserve_metadata(&pool, columns, 4, cancel))
            .transpose()
    }

    fn reserve_metadata(
        pool: &Arc<dyn pse_columnar::MemoryPool>,
        columns: usize,
        vectors: usize,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<StorageMetadata, RelationError> {
        let bytes = columns
            .checked_mul(vectors * size_of::<arrow_array::ArrayRef>())
            .and_then(|n| n.checked_add(size_of::<CheckedStorage>() + 256))
            .ok_or_else(|| mismatch("checked storage metadata extent"))?;
        let metadata =
            pse_columnar::MemoryConsumer::new("relations:checked-storage").register(pool);
        metadata
            .try_grow(bytes)
            .map_err(pse_columnar::CanonError::from)?;
        cancel.checkpoint()?;
        Ok((
            pse_columnar::AllocationLease::new(metadata),
            Arc::downgrade(pool),
        ))
    }

    /// Admit through the actual immutable native function/configuration owner.
    /// This carries the same local certificate as `admit`, without substituting
    /// a registry-default SQL environment for the selected session.
    /// # Errors
    /// Foreign context, schema/value failure or cancellation.
    pub fn admit(
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        batch: RecordBatch,
        context: &crate::validate::ValidationContext,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        let contract = registry.contract(spec)?;
        crate::validate::validate_schema(registry, spec, &batch.schema())
            .map_err(|errors| RelationError::Validation { errors })?;
        let prepared = context.relation(registry, spec)?;
        prepared.evaluate(&batch, 256, cancel)?.require_valid()?;
        cancel.checkpoint()?;
        let admission = LocalFieldAdmission {
            contract: contract.clone(),
            prepared,
        };
        Ok(Self::from_parts(spec.id, contract, batch, None).with_admission(Some(admission)))
    }
    /// Transfer native allocation ownership while admitting in the exact selected context.
    /// # Errors
    /// Foreign context, schema/value failure or cancellation.
    pub fn admit_owned(
        registry: &pse_schema::Registry,
        spec: &pse_schema::model::RelationSpec,
        batch: pse_columnar::owned_buffer::OwnedRecordBatch,
        context: &crate::validate::ValidationContext,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, RelationError> {
        let checked = Self::admit(registry, spec, batch.batch().clone(), context, cancel)?;
        Ok(Self::from_parts(
            spec.id,
            checked.contract,
            batch.batch().clone(),
            Some(batch),
        )
        .with_admission(checked.admission))
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
        let extent = pse_columnar::allocation_extent::algorithm_decode_extent(&self.storage.batch)?
            .checked_mul(indices.len().max(1))
            .ok_or_else(|| mismatch("bounded take extent"))?;
        let reservation =
            pse_columnar::MemoryConsumer::new("relations:checked-take").register(pool);
        reservation
            .try_grow(extent)
            .map_err(pse_columnar::CanonError::from)?;
        let columns = self
            .storage
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
            self.storage.batch.schema(),
            columns,
            &RecordBatchOptions::new().with_row_count(Some(indices.len())),
        )?;
        cancel.checkpoint()?;
        let scope = pse_columnar::owned_buffer::AllocationScope::default();
        if let Some(owned) = &self.storage.owned {
            scope.import(owned)?;
        }
        let owned = scope.attach_reserved(batch, reservation)?;
        Self::from_parts(
            self.relation_id,
            self.contract.clone(),
            owned.batch().clone(),
            Some(owned),
        )
        .with_admission(self.admission.clone())
        .retained(pool, cancel)
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
        context: &crate::validate::ValidationContext,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        let schema = pse_schema::arrow::relation_schema_ref(registry, spec)?;
        if !schema.fields().iter().map(|field| field.name()).eq(self
            .storage
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
                &self.storage.batch,
            )?)
            .map_err(pse_columnar::CanonError::from)?;
        let options = arrow::compute::CastOptions {
            safe: false,
            ..Default::default()
        };
        let columns = self
            .storage
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
            &RecordBatchOptions::new().with_row_count(Some(self.storage.batch.num_rows())),
        )?;
        cancel.checkpoint()?;
        let scope = pse_columnar::owned_buffer::AllocationScope::default();
        if let Some(owned) = &self.storage.owned {
            scope.import(owned)?;
        }
        Self::admit_owned(
            registry,
            spec,
            scope.attach_reserved(batch, reservation)?,
            context,
            cancel,
        )?
        .retained(pool, cancel)
    }
    /// Compare the exact checked row domain and retained semantic authority.
    /// Equal values in different arrays are not the same source owner.
    pub fn same_source(&self, other: &Self) -> bool {
        self.contract.require_equivalent(&other.contract).is_ok()
            && self.storage.batch.schema() == other.storage.batch.schema()
            && self.storage.batch.num_rows() == other.storage.batch.num_rows()
            && self.storage.batch.num_columns() == other.storage.batch.num_columns()
            && self
                .storage
                .batch
                .columns()
                .iter()
                .zip(other.storage.batch.columns())
                .all(|(left, right)| Arc::ptr_eq(left, right))
    }

    /// Actual owned storage, when allocation admission has completed.
    pub fn owned(&self) -> Option<&pse_columnar::owned_buffer::OwnedRecordBatch> {
        self.storage.owned.as_ref()
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
        if mask.len() != self.storage.batch.num_rows() {
            return Err(mismatch("a filter mask in the checked row domain"));
        }
        let extent = self
            .storage
            .batch
            .get_array_memory_size()
            .checked_mul(8)
            .and_then(|n| n.checked_add(mask.len().checked_mul(16)?))
            .and_then(|n| {
                n.checked_add(
                    self.storage
                        .batch
                        .schema()
                        .fields()
                        .len()
                        .checked_mul(256)?,
                )
            })
            .ok_or_else(|| mismatch("a representable filter allocation extent"))?;
        let reservation =
            pse_columnar::MemoryConsumer::new("relations:checked-filter").register(pool);
        reservation
            .try_grow(extent)
            .map_err(pse_columnar::CanonError::from)?;
        let predicate = arrow::compute::FilterBuilder::new(mask).build();
        let batch = predicate.filter_record_batch(&self.storage.batch)?;
        cancel.checkpoint()?;
        let scope = pse_columnar::owned_buffer::AllocationScope::default();
        if let Some(owned) = &self.storage.owned {
            scope.import(owned)?;
        }
        let owned = scope.attach_reserved(batch, reservation)?;
        Self::from_parts(
            self.relation_id,
            self.contract.clone(),
            owned.batch().clone(),
            Some(owned),
        )
        .with_admission(self.admission.clone())
        .retained(pool, cancel)
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
        context: &crate::validate::ValidationContext,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        registry.admit_contract(&self.contract)?;
        let contract = registry.contract(spec)?;
        // Admission overlaps the native projection scratch and precedes every new
        // column vector, including the owned projection's metadata reconstruction.
        let metadata = self.transform_metadata(positions.len(), cancel)?;
        let projected = self.storage.batch.project(positions)?;
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
        let prepared = context.relation(registry, spec)?;
        let same_owner = self
            .admission
            .as_ref()
            .is_some_and(|admission| admission.prepared.same_native_owner(&prepared));
        if !same_owner {
            prepared.evaluate(&batch, 256, cancel)?.require_valid()?;
        } else if !missing.is_empty() {
            prepared
                .evaluate_missing_checks(&batch, &missing, cancel)?
                .require_valid()?;
        }
        let admission = LocalFieldAdmission {
            contract: contract.clone(),
            prepared,
        };
        let owned = self
            .storage
            .owned
            .as_ref()
            .map(|owned| {
                owned.project(positions).and_then(|owned| {
                    owned.with_schema_metadata(contract.schema().metadata().clone())
                })
            })
            .transpose()?;
        Ok(
            Self::from_parts_with_metadata(spec.id, contract.clone(), batch, owned, metadata)
                .with_admission(Some(admission)),
        )
    }

    /// Isolate raw buffers and admit through the selected immutable engine context.
    /// The copy and validation scratch retain the same bounds as registry admission.
    /// # Errors
    /// Foreign context, invalid fields/values, cancellation or allocation refusal.
    pub fn admit_external(
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
        let input =
            Self::admit_owned(registry, spec, isolated, context, cancel)?.retained(pool, cancel)?;
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
        context: &crate::validate::ValidationContext,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Self, RelationError> {
        let projected = batch.project(positions)?;
        let schema = pse_schema::arrow::relation_schema_ref(registry, spec)?;
        exact_fields(projected.schema().fields(), schema.fields())?;
        Self::admit_owned(
            registry,
            spec,
            projected.with_schema_metadata(schema.metadata().clone())?,
            context,
            cancel,
        )
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
            exact_fields(input.storage.batch.schema().fields(), schema.fields())?;
        }
        let batch = arrow::compute::concat_batches(&schema, inputs.iter().map(Self::batch))?;
        let admission = inputs
            .first()
            .and_then(|input| input.admission.clone())
            .filter(|first| {
                inputs.iter().all(|input| {
                    input
                        .admission
                        .as_ref()
                        .is_some_and(|other| first.prepared.same_native_owner(&other.prepared))
                })
            });
        Ok(Self::from_parts(spec.id, contract, batch, None).with_admission(admission))
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
                bytes.checked_add(input.storage.batch.get_array_memory_size())
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
        let result = Self::concat(registry, spec, inputs)?;
        cancel.checkpoint()?;
        let scope = pse_columnar::owned_buffer::AllocationScope::default();
        for input in inputs {
            if let Some(owned) = &input.storage.owned {
                scope.import(owned)?;
            }
        }
        let admission = result.admission.clone();
        let owned = scope.attach_reserved(result.into_batch(), reservation)?;
        Self::from_parts(
            spec.id,
            registry.contract(spec)?,
            owned.batch().clone(),
            Some(owned),
        )
        .with_admission(admission)
        .retained(pool, cancel)
    }

    /// Retains a checked subset of rows without copying or rescanning their values.
    ///
    /// # Errors
    /// The requested row range lies outside this batch or retained metadata is refused.
    pub fn slice(&self, offset: usize, length: usize) -> Result<Self, RelationError> {
        if offset
            .checked_add(length)
            .is_none_or(|end| end > self.storage.batch.num_rows())
        {
            return Err(mismatch("a slice within the checked batch"));
        }
        let metadata = self.transform_metadata(
            self.storage.batch.num_columns(),
            &pse_columnar::CancellationToken::new(),
        )?;
        let owned = self
            .storage
            .owned
            .as_ref()
            .map(|owned| owned.slice(offset, length))
            .transpose()?;
        Ok(Self::from_parts_with_metadata(
            self.relation_id,
            self.contract.clone(),
            self.storage.batch.slice(offset, length),
            owned,
            metadata,
        )
        .with_admission(self.admission.clone()))
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
        if self.storage.owned.is_some() && self.storage.metadata.is_some() {
            return Ok(self.clone());
        }
        let (metadata, pool_owner) =
            Self::reserve_metadata(pool, self.storage.batch.num_columns(), 2, cancel)?;
        let owned = match &self.storage.owned {
            Some(owned) => owned.clone(),
            None => pse_columnar::owned_buffer::OwnedRecordBatch::export(
                self.storage.batch.clone(),
                pool,
                cancel,
            )?,
        };
        Ok(Self {
            relation_id: self.relation_id,
            contract: self.contract.clone(),
            storage: Arc::new(CheckedStorage {
                batch: owned.batch().clone(),
                owned: Some(owned),
                metadata: Some(metadata),
                pool: Some(pool_owner),
            }),
            export_owner: self.export_owner.clone(),
            admission: self.admission.clone(),
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
            std::slice::from_ref(&self.storage.batch),
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
            Self::from_parts(
                self.relation_id,
                self.contract.clone(),
                batch.batch().clone(),
                Some(batch),
            )
            .with_admission(self.admission.clone())
            .retained(pool, &pse_columnar::CancellationToken::new())?,
            output,
        ))
    }

    /// The declaration used to construct the batch.
    pub const fn relation_id(&self) -> SemanticId {
        self.relation_id
    }

    /// Borrows the immutable Arrow columns, retaining their owners.
    pub fn batch(&self) -> &RecordBatch {
        &self.storage.batch
    }

    /// Extracts a raw, unadmitted interoperability candidate. Checked storage and
    /// container metadata leases are relinquished; the receiver owns admission and
    /// accounting for the raw column vectors. Existing backing-buffer leases remain.
    /// This is not the checked-export boundary; use checked clones to retain it.
    pub fn into_batch(self) -> RecordBatch {
        match Arc::try_unwrap(self.storage) {
            Ok(storage) => storage.batch,
            Err(storage) => storage.batch.clone(),
        }
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
        Ok(&self.storage.batch)
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
        context: &crate::validate::ValidationContext,
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
            prepared: context.relation(registry, spec)?,
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
        let admission = LocalFieldAdmission {
            contract: self.contract.clone(),
            prepared: self.prepared,
        };
        Ok(
            FieldCheckedBatch::from_parts(self.relation_id, self.contract, batch, None)
                .with_admission(Some(admission)),
        )
    }
}

#[cfg(test)]
mod consolidation_unit {
    use super::*;
    use crate::generated::reference::dimensions;
    use crate::native::execution::context::SessionContext;

    #[test]
    fn generated_builders_views_and_foreign_rebinding_retain_native_columns()
    -> Result<(), RelationError> {
        let registry = pse_schema::catalog::assemble()?;
        let mut builder = dimensions::Builder::with_registry(
            &registry,
            1,
            &crate::validate::ValidationContext::new(&registry, SessionContext::new().state()),
        )?;
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
        let checked = dimensions::Builder::new(&crate::validate::ValidationContext::new(
            pse_schema::registry().unwrap(),
            SessionContext::new().state(),
        ))?
        .finish()?;
        assert_eq!(dimensions::View::from_checked(&checked)?.rows()?.len(), 0);
        assert!(
            crate::generated::reference::schema_enum_types::View::from_checked(&checked).is_err()
        );
        Ok(())
    }

    #[test]
    fn changed_projection_cannot_reuse_parent_admission() -> Result<(), RelationError> {
        let registry = pse_schema::registry()?;
        let mut builder = dimensions::Builder::new(&crate::validate::ValidationContext::new(
            pse_schema::registry().unwrap(),
            SessionContext::new().state(),
        ))?;
        builder.push(dimensions::Row {
            ordinal: 0,
            name: "length".into(),
        })?;
        let checked = builder.finish()?;
        let projected = checked.batch().project(&[1])?;
        assert!(
            FieldCheckedBatch::admit(
                registry,
                dimensions::spec(registry)?,
                projected,
                &crate::validate::ValidationContext::new(registry, SessionContext::new().state()),
                &pse_columnar::CancellationToken::new()
            )
            .is_err()
        );
        Ok(())
    }
}

#[cfg(test)]
mod integrated_performance_unit {
    use super::*;
    use crate::native::execution::context::SessionContext;
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
        registry.build().unwrap()
    }

    #[test]
    fn shared_checked_storage_and_export_metadata_follow_last_reader() {
        let registry = registry();
        let spec = registry.relation("authored.values").unwrap();
        let context =
            crate::validate::ValidationContext::new(&registry, SessionContext::new().state());
        let cancel = pse_columnar::CancellationToken::new();
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        let raw = RecordBatch::try_new(
            pse_schema::arrow::relation_schema_ref(&registry, spec).unwrap(),
            vec![Arc::new(Int64Array::from(vec![1, 2]))],
        )
        .unwrap();
        let checked = FieldCheckedBatch::admit(&registry, spec, raw, &context, &cancel)
            .unwrap()
            .retained(&pool, &cancel)
            .unwrap();
        let storage_bytes = checked.storage.metadata.as_ref().unwrap().size();
        let container =
            pse_columnar::MemoryConsumer::new("checked-export:test-container").register(&pool);
        container.try_grow(4096).unwrap();
        let checked = checked.with_export_owner(pse_columnar::AllocationLease::new(container));
        let retained = pool.reserved();
        let reader = checked.clone();
        assert!(Arc::ptr_eq(&checked.storage, &reader.storage));
        assert_eq!(pool.reserved(), retained);
        let selected = checked.slice(0, 1).unwrap();
        assert!(
            selected.export_owner.is_none(),
            "new storage does not retain the obsolete container grant"
        );
        let selected_storage = selected.storage.metadata.as_ref().unwrap().size();
        drop(checked);
        assert!(pool.reserved() >= 4096 + storage_bytes + selected_storage);
        drop(reader);
        assert_eq!(
            pool.reserved(),
            retained - 4096 - storage_bytes + selected_storage
        );
        assert_eq!(selected.batch().num_rows(), 1);
        drop(selected);
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn checked_arrow_export_retains_storage_container_and_wrapper_lease_without_copying() {
        let registry = registry();
        let spec = registry.relation("authored.values").unwrap();
        let context =
            crate::validate::ValidationContext::new(&registry, SessionContext::new().state());
        let cancel = pse_columnar::CancellationToken::new();
        for values in [vec![], vec![1, 2, 3]] {
            let pool: Arc<dyn pse_columnar::MemoryPool> =
                Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
            let raw = RecordBatch::try_new(
                pse_schema::arrow::relation_schema_ref(&registry, spec).unwrap(),
                vec![Arc::new(Int64Array::from(values.clone()))],
            )
            .unwrap();
            let checked = FieldCheckedBatch::admit(&registry, spec, raw, &context, &cancel)
                .unwrap()
                .retained(&pool, &cancel)
                .unwrap();
            let container =
                pse_columnar::MemoryConsumer::new("checked-export:test-container").register(&pool);
            container.try_grow(4096).unwrap();
            let checked = checked.with_export_owner(pse_columnar::AllocationLease::new(container));
            let before = pool.reserved();
            let pressure =
                pse_columnar::MemoryConsumer::new("checked-export:test-pressure").register(&pool);
            pressure.try_grow((1 << 20) - before).unwrap();
            assert!(checked.checked_export(&cancel).is_err());
            assert_eq!(pool.reserved(), 1 << 20);
            drop(pressure);
            assert_eq!(pool.reserved(), before);
            let exported = checked.checked_export(&cancel).unwrap();
            let retained = pool.reserved();
            assert!(
                retained > before,
                "wrapper metadata has its own admitted grant"
            );
            assert_eq!(exported.schema(), checked.batch().schema());
            assert_eq!(
                exported.column(0).to_data().buffers()[0].as_ptr(),
                checked.batch().column(0).to_data().buffers()[0].as_ptr(),
                "no value allocation was copied"
            );
            let escaped = exported.column(0).clone();
            let escaped_clone = escaped.clone();
            assert_eq!(
                pool.reserved(),
                retained,
                "export clones do not recharge backing allocations"
            );
            drop(checked);
            drop(exported);
            assert_eq!(
                pool.reserved(),
                retained,
                "an escaped array holds every original and wrapper lease"
            );
            assert_eq!(
                escaped.as_any().downcast_ref::<Int64Array>().unwrap(),
                &Int64Array::from(values)
            );
            drop(escaped);
            assert_eq!(pool.reserved(), retained);
            drop(escaped_clone);
            assert_eq!(pool.reserved(), 0);
        }
    }

    #[test]
    fn exhausted_pool_refuses_checked_slice_and_projection_before_new_storage() {
        let registry = registry();
        let spec = registry.relation("authored.values").unwrap();
        let context =
            crate::validate::ValidationContext::new(&registry, SessionContext::new().state());
        let cancel = pse_columnar::CancellationToken::new();
        let limit = 1 << 20;
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(limit));
        let raw = RecordBatch::try_new(
            pse_schema::arrow::relation_schema_ref(&registry, spec).unwrap(),
            vec![Arc::new(Int64Array::from(vec![1, 2]))],
        )
        .unwrap();
        let checked = FieldCheckedBatch::admit(&registry, spec, raw, &context, &cancel)
            .unwrap()
            .retained(&pool, &cancel)
            .unwrap();
        let occupied =
            pse_columnar::MemoryConsumer::new("checked-transform:test-occupied").register(&pool);
        occupied.try_grow(limit - pool.reserved()).unwrap();
        let calls = context
            .relation(&registry, spec)
            .unwrap()
            .evaluation_count();
        assert!(checked.slice(0, 1).is_err());
        assert!(
            checked
                .project_exact(&registry, spec, &[0], &cancel, &context)
                .is_err()
        );
        assert_eq!(
            context
                .relation(&registry, spec)
                .unwrap()
                .evaluation_count(),
            calls
        );
        assert_eq!(pool.reserved(), limit);
        assert_eq!(checked.batch().num_rows(), 2);
        drop(occupied);
        let selected = checked.slice(0, 1).unwrap();
        let projected = checked
            .project_exact(&registry, spec, &[0], &cancel, &context)
            .unwrap();
        drop((checked, selected, projected));
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn raw_candidates_relinquish_checked_metadata_but_keep_buffer_owners() {
        let registry = registry();
        let spec = registry.relation("authored.values").unwrap();
        let context =
            crate::validate::ValidationContext::new(&registry, SessionContext::new().state());
        let cancel = pse_columnar::CancellationToken::new();
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        for shared in [false, true] {
            let raw = RecordBatch::try_new(
                pse_schema::arrow::relation_schema_ref(&registry, spec).unwrap(),
                vec![Arc::new(Int64Array::from(vec![1, 2]))],
            )
            .unwrap();
            let checked = FieldCheckedBatch::admit(&registry, spec, raw, &context, &cancel)
                .unwrap()
                .retained(&pool, &cancel)
                .unwrap();
            let buffers = checked.owned().unwrap().retained_bytes().unwrap();
            let container =
                pse_columnar::MemoryConsumer::new("raw-candidate:test-container").register(&pool);
            container.try_grow(4096).unwrap();
            let checked = checked.with_export_owner(pse_columnar::AllocationLease::new(container));
            let raw = if shared {
                let raw = checked.clone().into_batch();
                drop(checked);
                raw
            } else {
                checked.into_batch()
            };
            // The raw Vec belongs to destination admission, not checked export accounting.
            assert_eq!(pool.reserved(), buffers);
            assert_eq!(
                raw.column(0).as_any().downcast_ref::<Int64Array>().unwrap(),
                &Int64Array::from(vec![1, 2])
            );
            drop(raw);
            assert_eq!(pool.reserved(), 0);
        }
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
        let source = FieldCheckedBatch::admit(
            &registry,
            spec,
            batch(),
            &crate::validate::ValidationContext::new(&registry, SessionContext::new().state()),
            &pse_columnar::CancellationToken::new(),
        )
        .unwrap();
        assert!(source.same_source(&source.clone()));
        assert!(
            !source.same_source(
                &FieldCheckedBatch::admit(
                    &registry,
                    spec,
                    batch(),
                    &crate::validate::ValidationContext::new(
                        &registry,
                        SessionContext::new().state()
                    ),
                    &pse_columnar::CancellationToken::new()
                )
                .unwrap()
            )
        );
        assert!(!source.same_source(&source.slice(0, 1).unwrap()));
    }
    #[test]
    fn checked_handoffs_preserve_evidence_but_not_cross_chunk_keys() {
        let registry = registry();
        let spec = registry.relation("authored.values").unwrap();
        let validation =
            crate::validate::ValidationContext::new(&registry, SessionContext::new().state());
        let prepared = validation.relation(&registry, spec).unwrap();
        let batch = RecordBatch::try_new(
            prepared.schema().clone(),
            vec![Arc::new(Int64Array::from(vec![0, 1, 2]))],
        )
        .unwrap();
        let checked = FieldCheckedBatch::admit(
            &registry,
            spec,
            batch,
            &validation,
            &pse_columnar::CancellationToken::new(),
        )
        .unwrap();
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
            .project_exact(&registry, spec, &[0], &cancel, &validation)
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
                .project_exact(&registry, spec, &[1], &cancel, &validation)
                .is_err()
        );
        let strict = registry.relation("authored.positive").unwrap();
        assert!(
            checked
                .project_exact(&registry, strict, &[0], &cancel, &validation)
                .is_err()
        );
        assert!(
            checked
                .slice(1, 2)
                .unwrap()
                .project_exact(&registry, strict, &[0], &cancel, &validation)
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
            &crate::validate::ValidationContext::new(&registry, SessionContext::new().state()),
            &pse_columnar::CancellationToken::new(),
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
                .bind(
                    &RelationInputs::from([(spec.id, input)]),
                    &context.state(),
                    None,
                )
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

#[cfg(test)]
mod native_owner_unit {
    use super::*;
    use crate::native::execution::context::SessionContext;
    use crate::native::logical_expr::{ColumnarValue, Volatility, create_udf};
    use arrow_array::{BooleanArray, Int64Array};
    use arrow_schema::DataType;
    use pse_schema::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};

    #[test]
    fn native_owner_reuse_changed_bindings_and_values_are_distinct() {
        let mut builder = pse_schema::RegistryBuilder::new();
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "policy",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "native owner control",
            )
            .pk(&["n"])
            .columns(vec![FieldContract::payload(
                "n",
                FieldContract::native(DataType::Int64),
                "value",
            )])
            .checks(std::collections::BTreeMap::from([(
                "actual".into(),
                "actual_policy(n)".into(),
            )])),
        );
        let registry = builder.build().unwrap();
        let spec = registry.relation("authored.policy").unwrap();
        let context = |accepted| {
            let session = SessionContext::new();
            session.register_udf(create_udf(
                "actual_policy",
                vec![DataType::Int64],
                DataType::Boolean,
                Volatility::Immutable,
                Arc::new(move |_| {
                    Ok(ColumnarValue::Scalar(
                        crate::native::common::ScalarValue::Boolean(Some(accepted)),
                    ))
                }),
            ));
            crate::validate::ValidationContext::new(&registry, session.state())
        };
        let accepted = context(true);
        let refused = context(false);
        let raw = |value| {
            RecordBatch::try_new(
                pse_schema::arrow::relation_schema_ref(&registry, spec).unwrap(),
                vec![Arc::new(Int64Array::from(vec![value]))],
            )
            .unwrap()
        };
        let cancel = pse_columnar::CancellationToken::new();
        let prepared = accepted.relation(&registry, spec).unwrap();
        let source = FieldCheckedBatch::admit(&registry, spec, raw(1), &accepted, &cancel).unwrap();
        let count = prepared.evaluation_count();
        source
            .validate_context(&registry, spec, &accepted, &cancel)
            .unwrap();
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        let filtered = source
            .filter_reserved(&BooleanArray::from(vec![true]), &pool, &cancel)
            .unwrap();
        let taken = source
            .take_reserved(&arrow_array::UInt32Array::from(vec![0, 0]), &pool, &cancel)
            .unwrap();
        let projected = source
            .project_exact(&registry, spec, &[0], &cancel, &accepted)
            .unwrap();
        let combined = FieldCheckedBatch::concat_reserved(
            &registry,
            spec,
            &[filtered, taken, projected, source.slice(0, 1).unwrap()],
            &pool,
            &cancel,
        )
        .unwrap();
        combined
            .validate_context(&registry, spec, &accepted, &cancel)
            .unwrap();
        assert_eq!(
            prepared.evaluation_count(),
            count,
            "same native owner reuses successful immutable admission"
        );
        assert!(
            source
                .validate_context(&registry, spec, &refused, &cancel)
                .is_err()
        );
        assert!(
            source
                .project_exact(&registry, spec, &[0], &cancel, &refused)
                .is_err(),
            "equal schema and SQL cannot reuse another actual UDF implementation"
        );
        assert_eq!(
            refused
                .relation(&registry, spec)
                .unwrap()
                .evaluation_count(),
            2
        );
        FieldCheckedBatch::admit(&registry, spec, raw(2), &accepted, &cancel).unwrap();
        assert_eq!(
            prepared.evaluation_count(),
            count + 1,
            "raw changed values always evaluate"
        );
        let another = context(true);
        let other = FieldCheckedBatch::admit(&registry, spec, raw(3), &another, &cancel).unwrap();
        let mixed = FieldCheckedBatch::concat(&registry, spec, &[source.clone(), other]).unwrap();
        assert!(
            mixed.local_admission().is_none(),
            "different owners cannot mint common assurance"
        );
        mixed
            .validate_context(&registry, spec, &accepted, &cancel)
            .unwrap();
        assert_eq!(prepared.evaluation_count(), count + 2);
        cancel.cancel();
        assert!(
            source
                .validate_context(&registry, spec, &accepted, &cancel)
                .is_err()
        );
    }
}
