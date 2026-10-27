// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Controlled native result-file construction and exact facts from its admitted batches.
//! Deserializing recorded metadata never constructs an admission or append capability.

use crate::{
    RelationError,
    columnar::{FieldCheckedBatch, mismatch},
};
use arrow::{
    array::{Array, FixedSizeBinaryArray, Float64Array, Int64Array},
    compute,
    row::{RowConverter, SortField},
};
use datafusion_common::{ScalarValue, Statistics, stats::Precision};
use pse_columnar::{AllocationLease, CancellationToken, MemoryConsumer, MemoryPool};
use pse_ids::{ContentHash, SemanticId};
use pse_schema::{Registry, resolved_contract::ExpectedContract};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Expectation {
    relation: SemanticId,
    version: u32,
    semantic_version: u32,
    semantics: ContentHash,
    encoding: ContentHash,
}
impl From<ExpectedContract> for Expectation {
    fn from(value: ExpectedContract) -> Self {
        Self {
            relation: value.relation,
            version: value.version,
            semantic_version: value.semantic_version,
            semantics: value.semantics,
            encoding: value.encoding,
        }
    }
}

/// Exact immutable payload and interpretation coordinates supplied by the publication owner.
/// These coordinates alone carry no statistics or admission authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatisticsBinding {
    /// Original canonical inline body key or never-reused sealed file instance.
    pub payload: String,
    /// Integrity digest of the complete encoded file.
    pub digest: String,
    /// Original native record-batch index inside that file.
    pub index: u64,
    /// Current canonical execution interpretation.
    pub interpretation: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Extremum {
    Id([u8; 16]),
    Integer(i64),
    FloatBits(u64),
}
impl Extremum {
    fn scalar(&self, kind: &arrow_schema::DataType) -> Option<ScalarValue> {
        match (self, kind) {
            (Self::Id(value), arrow_schema::DataType::FixedSizeBinary(16)) => {
                Some(ScalarValue::FixedSizeBinary(16, Some(value.to_vec())))
            }
            (Self::Integer(value), arrow_schema::DataType::Int64) => {
                Some(ScalarValue::Int64(Some(*value)))
            }
            (Self::FloatBits(bits), arrow_schema::DataType::Float64)
                if f64::from_bits(*bits).is_finite() =>
            {
                Some(ScalarValue::Float64(Some(f64::from_bits(*bits))))
            }
            _ => None,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ColumnFact {
    nulls: u64,
    minimum: Option<Extremum>,
    maximum: Option<Extremum>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct FactRecord {
    version: u32,
    contract: Expectation,
    rows: u64,
    columns: Vec<ColumnFact>,
}
#[derive(Debug)]
struct FactsInner {
    record: FactRecord,
    _owner: Arc<AllocationLease>,
}
/// Non-deserializable facts derived from the actual batch supplied to the encoder.
/// Clones retain the same immutable facts and their allocation owner.
#[derive(Clone, Debug)]
pub struct BatchFacts(Arc<FactsInner>);

#[derive(Serialize, Deserialize)]
struct RecordedFacts {
    binding: StatisticsBinding,
    facts: FactRecord,
}
impl BatchFacts {
    /// Actual admitted scientific relation.
    pub fn relation(&self) -> SemanticId {
        self.0.record.contract.relation
    }
    /// Actual record-batch row count.
    pub fn rows(&self) -> u64 {
        self.0.record.rows
    }
    /// Encode a bounded closed metadata envelope for the verified sealed-file owner.
    /// # Errors
    /// Encoding failure or metadata beyond the independently declared bound.
    pub fn encode_bound(&self, binding: &StatisticsBinding) -> Result<Vec<u8>, RelationError> {
        if binding.payload.is_empty()
            || binding.payload.len() > 64
            || binding.digest.len() != 64
            || !binding.digest.bytes().all(|byte| byte.is_ascii_hexdigit())
            || binding.interpretation.is_empty()
            || binding.interpretation.len() > 128
        {
            return Err(mismatch("bounded sealed result statistics binding"));
        }
        #[derive(Serialize)]
        struct Borrowed<'a> {
            binding: &'a StatisticsBinding,
            facts: &'a FactRecord,
        }
        let bytes = serde_json::to_vec(&Borrowed {
            binding,
            facts: &self.0.record,
        })
        .map_err(|_| mismatch("result statistics encoding"))?;
        if bytes.len() > pse_schema::catalog::substrate::RESULT_STATISTICS_BYTES {
            return Err(mismatch("bounded result statistics envelope"));
        }
        Ok(bytes)
    }
}

/// Complete native file and facts produced together from immutable admitted values.
/// Neither raw IPC nor serialized facts can construct this capability.
#[derive(Debug)]
pub struct AdmittedResultFile {
    bytes: Vec<u8>,
    relation: SemanticId,
    facts: Vec<BatchFacts>,
    _encoded: Arc<AllocationLease>,
}
impl AdmittedResultFile {
    /// Encode the actual admitted batches once and retain their exact facts.
    /// The selected registry proves the complete declaration; no copied field list
    /// establishes selection meaning. Callers retain source allocation ownership.
    /// # Errors
    /// Foreign/incompatible declarations, duplicate scalar keys, cancellation,
    /// allocation refusal or an oversized complete native file.
    pub fn encode(
        registry: &Registry,
        batches: &[FieldCheckedBatch],
        byte_limit: usize,
        pool: &Arc<dyn MemoryPool>,
        cancel: &CancellationToken,
    ) -> Result<Self, RelationError> {
        cancel.checkpoint()?;
        let first = batches
            .first()
            .ok_or_else(|| mismatch("nonempty admitted file inventory"))?;
        if batches.len() > pse_columnar::ipc_file::MAX_BATCHES {
            return Err(mismatch("bounded file batch inventory"));
        }
        let encoder = MemoryConsumer::new("relations:admitted-result-file").register(pool);
        encoder.try_grow(pse_columnar::ipc_file::writer_preparation_extent(first.batch().schema().as_ref(), batches.len())?)
            .map_err(pse_columnar::CanonError::from)?;
        let native = batches
            .iter()
            .map(|batch| batch.batch().clone())
            .collect::<Vec<_>>();
        pse_columnar::ipc_file::preflight(&native, byte_limit)?;
        let spec = registry
            .relation_by_id(first.relation_id())
            .ok_or_else(|| mismatch("registered result relation"))?;
        let expected = ExpectedContract::capture(registry, spec)?;
        let metadata = batches.iter().try_fold(1024usize, |bytes, batch| {
            if batch.batch().num_columns() > 256 {
                return Err(mismatch("bounded result column inventory"));
            }
            batch
                .batch()
                .num_columns()
                .checked_mul(512)
                .and_then(|extent| extent.checked_add(1024))
                .and_then(|extent| bytes.checked_add(extent))
                .ok_or_else(|| mismatch("result facts allocation extent"))
        })?;
        let owner = MemoryConsumer::new("relations:admitted-result-facts").register(pool);
        owner
            .try_grow(metadata)
            .map_err(pse_columnar::CanonError::from)?;
        let owner = AllocationLease::new(owner);
        let extent = pse_columnar::ipc_file::writer_extent(&native, byte_limit)?;
        encoder.try_grow(extent.checked_sub(encoder.size()).ok_or_else(|| mismatch("complete writer preparation extent"))?)
            .map_err(pse_columnar::CanonError::from)?;
        let mut facts = Vec::with_capacity(batches.len());
        for batch in batches {
            cancel.checkpoint()?;
            batch.check_declaration(registry, spec)?;
            if batch.relation_id() != first.relation_id()
                || batch.batch().schema() != first.batch().schema()
            {
                return Err(mismatch("one exact admitted file schema"));
            }
            validate_keys(batch, spec, pool, cancel)?;
            let nulls = logical_null_counts(batch.batch(), pool, cancel)?;
            let selection = crate::generated::result_selection::for_relation(batch.relation_id());
            let schema = batch.batch().schema();
            let columns = batch
                .batch()
                .columns()
                .iter()
                .enumerate()
                .map(|(position, array)| {
                    let field = &schema.fields()[position];
                    let selected = selection.is_some_and(|selection| {
                        selection.sort.contains(&field.name().as_str())
                            || selection.values.contains(&field.name().as_str())
                    });
                    let (minimum, maximum) = if selected {
                        extrema(array.as_ref(), cancel)?
                    } else {
                        (None, None)
                    };
                    Ok(ColumnFact {
                        nulls: nulls[position],
                        minimum,
                        maximum,
                    })
                })
                .collect::<Result<Vec<_>, RelationError>>()?;
            facts.push(BatchFacts(Arc::new(FactsInner {
                record: FactRecord {
                    version: pse_schema::catalog::substrate::RESULT_STATISTICS_VERSION,
                    contract: expected.into(),
                    rows: batch.batch().num_rows() as u64,
                    columns,
                },
                _owner: owner.clone(),
            })));
        }
        let bytes = pse_columnar::ipc_file::encode(&native, byte_limit)?;
        cancel.checkpoint()?;
        drop(native);
        // The complete writer has dropped its scratch. Transfer the original
        // grant to the actual retained allocation without releasing/reacquiring
        // byte custody or keeping the transient writer reallocation/scratch/control allowance.
        let surplus = encoder
            .size()
            .checked_sub(bytes.capacity())
            .ok_or_else(|| mismatch("encoded result allocation within encoder grant"))?;
        encoder.shrink(surplus);
        Ok(Self {
            bytes,
            relation: first.relation_id(),
            facts,
            _encoded: AllocationLease::new(encoder),
        })
    }
    /// Exact bytes that were encoded with these facts.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Exact relation shared by every encoded member.
    pub fn relation(&self) -> SemanticId {
        self.relation
    }
    /// Non-forgeable facts in native file batch order.
    pub fn facts(&self) -> &[BatchFacts] {
        &self.facts
    }
}

// Dictionary, union and run-end arrays can allocate logical validity bitmaps.
// Native primitive/Null counts do not allocate; encoded counts prepay the existing
// columnar predicate extent before asking Arrow to inspect logical validity.
fn logical_null_counts(
    batch: &arrow::record_batch::RecordBatch,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<Vec<u64>, RelationError> {
    cancel.checkpoint()?;
    let scratch = MemoryConsumer::new("relations:result-logical-nulls").register(pool);
    if batch.columns().iter().any(|array| {
        matches!(
            array.data_type(),
            arrow_schema::DataType::Dictionary(_, _)
                | arrow_schema::DataType::Union(_, _)
                | arrow_schema::DataType::RunEndEncoded(_, _)
        )
    }) {
        scratch
            .try_grow(pse_columnar::columnar_validation_extent(batch)?)
            .map_err(pse_columnar::CanonError::from)?;
    }
    batch
        .columns()
        .iter()
        .map(|array| {
            cancel.checkpoint()?;
            Ok(array.logical_null_count() as u64)
        })
        .collect()
}

/// Check original scalar result primary keys after complete field admission.
/// Dense relations do not acquire scalar ordering or key scratch through this route.
/// # Errors
/// Missing/foreign declaration, duplicate keys, cancellation or refused scratch.
pub fn validate_scalar_keys(
    registry: &Registry,
    batch: &FieldCheckedBatch,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<(), RelationError> {
    cancel.checkpoint()?;
    let spec = registry
        .relation_by_id(batch.relation_id())
        .ok_or_else(|| mismatch("registered result relation"))?;
    batch.check_declaration(registry, spec)?;
    validate_keys(batch, spec, pool, cancel)
}

fn extrema(
    array: &dyn Array,
    cancel: &CancellationToken,
) -> Result<(Option<Extremum>, Option<Extremum>), RelationError> {
    cancel.checkpoint()?;
    if let Some(values) = array.as_any().downcast_ref::<FixedSizeBinaryArray>() {
        if values.value_length() != 16 {
            return Ok((None, None));
        }
        let convert = |value: &[u8]| value.try_into().ok().map(Extremum::Id);
        return Ok((
            compute::min_fixed_size_binary(values).and_then(convert),
            compute::max_fixed_size_binary(values).and_then(convert),
        ));
    }
    if let Some(values) = array.as_any().downcast_ref::<Int64Array>() {
        return Ok((
            compute::min(values).map(Extremum::Integer),
            compute::max(values).map(Extremum::Integer),
        ));
    }
    if let Some(values) = array.as_any().downcast_ref::<Float64Array>() {
        for (index, value) in values.iter().enumerate() {
            if index % 1024 == 0 {
                cancel.checkpoint()?;
            }
            if value.is_some_and(|value| !value.is_finite()) {
                return Ok((None, None));
            }
        }
        return Ok((
            compute::min(values).map(|value| Extremum::FloatBits(value.to_bits())),
            compute::max(values).map(|value| Extremum::FloatBits(value.to_bits())),
        ));
    }
    Ok((None, None))
}

fn validate_keys(
    batch: &FieldCheckedBatch,
    spec: &pse_schema::model::RelationSpec,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<(), RelationError> {
    if !crate::generated::result_selection::for_relation(batch.relation_id())
        .is_some_and(|selection| !selection.sort.is_empty())
    {
        return Ok(());
    }
    let owner = MemoryConsumer::new("relations:result-key-uniqueness").register(pool);
    owner
        .try_grow(
            batch
                .batch()
                .num_rows()
                .checked_mul(256)
                .ok_or_else(|| mismatch("result key scratch extent"))?,
        )
        .map_err(pse_columnar::CanonError::from)?;
    let columns = spec
        .primary_key
        .iter()
        .map(|name| {
            batch
                .batch()
                .column_by_name(name)
                .cloned()
                .ok_or_else(|| mismatch("declared result primary key"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let converter = RowConverter::new(
        columns
            .iter()
            .map(|array| SortField::new(array.data_type().clone()))
            .collect(),
    )?;
    let rows = converter.convert_columns(&columns)?;
    let mut keys = HashSet::with_capacity(batch.batch().num_rows());
    for (index, row) in rows.iter().enumerate() {
        if index % 1024 == 0 {
            cancel.checkpoint()?;
        }
        if !keys.insert(row) {
            return Err(mismatch("unique original scientific result primary keys"));
        }
    }
    Ok(())
}

/// Sort a new scalar batch before assigning durable row positions. Dense and
/// undeclared result orderings retain their existing route and allocations.
/// # Errors
/// Missing declared keys, cancellation, Arrow failure or refused scratch/take allowance.
pub fn sort_new_scalar(
    batch: &FieldCheckedBatch,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<FieldCheckedBatch, RelationError> {
    let Some(selection) = crate::generated::result_selection::for_relation(batch.relation_id())
        .filter(|selection| !selection.sort.is_empty())
    else {
        return Ok(batch.clone());
    };
    cancel.checkpoint()?;
    let owner = MemoryConsumer::new("relations:result-key-sort").register(pool);
    owner
        .try_grow(
            batch
                .batch()
                .num_rows()
                .checked_mul(64 + selection.sort.len() * 32)
                .ok_or_else(|| mismatch("result sort extent"))?,
        )
        .map_err(pse_columnar::CanonError::from)?;
    let columns = selection
        .sort
        .iter()
        .map(|name| {
            Ok(compute::SortColumn {
                values: batch
                    .batch()
                    .column_by_name(name)
                    .cloned()
                    .ok_or_else(|| mismatch("declared result sort column"))?,
                options: None,
            })
        })
        .collect::<Result<Vec<_>, RelationError>>()?;
    let indices = compute::lexsort_to_indices(&columns, None)?;
    cancel.checkpoint()?;
    batch.take_reserved(&indices, pool, cancel)
}

/// Interpret only bound canonical metadata. Missing, malformed, changed or
/// unsupported facts remain unknown; this operation cannot issue an append proof.
pub fn recorded_statistics(
    bytes: &[u8],
    registry: &Registry,
    relation: SemanticId,
    schema: &arrow_schema::Schema,
    binding: &StatisticsBinding,
) -> Option<Statistics> {
    if bytes.len() > pse_schema::catalog::substrate::RESULT_STATISTICS_BYTES {
        return None;
    }
    let record: RecordedFacts = serde_json::from_slice(bytes).ok()?;
    let spec = registry.relation_by_id(relation)?;
    let expected = ExpectedContract::capture(registry, spec).ok()?;
    if record.binding != *binding
        || record.facts.version != pse_schema::catalog::substrate::RESULT_STATISTICS_VERSION
        || record.facts.contract != expected.into()
        || record.facts.columns.len() != schema.fields().len()
        || pse_schema::fingerprint::encoding_fields(schema.fields().iter().map(AsRef::as_ref))
            .ok()?
            != expected.encoding
    {
        return None;
    }
    let rows = usize::try_from(record.facts.rows).ok()?;
    let mut statistics = Statistics::new_unknown(schema);
    statistics.num_rows = Precision::Exact(rows);
    for ((fact, statistics), field) in record
        .facts
        .columns
        .iter()
        .zip(&mut statistics.column_statistics)
        .zip(schema.fields())
    {
        let nulls = usize::try_from(fact.nulls).ok()?;
        if nulls > rows {
            return None;
        }
        statistics.null_count = Precision::Exact(nulls);
        match (&fact.minimum, &fact.maximum) {
            (None, None) => {}
            (Some(minimum), Some(maximum)) => {
                let minimum = minimum.scalar(field.data_type())?;
                let maximum = maximum.scalar(field.data_type())?;
                if minimum
                    .partial_cmp(&maximum)
                    .is_none_or(|order| order.is_gt())
                {
                    return None;
                }
                statistics.min_value = Precision::Exact(minimum);
                statistics.max_value = Precision::Exact(maximum);
            }
            _ => return None,
        }
    }
    Some(statistics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::runtime::{
        fit_observations, fit_parameters, solve_constraints, solve_variables,
    };
    use datafusion::prelude::SessionContext;
    use pse_model::generated::identities::{InstanceId, RunId};

    fn pool() -> Arc<dyn MemoryPool> {
        Arc::new(pse_columnar::GreedyMemoryPool::new(16 * 1024 * 1024))
    }
    fn validation() -> crate::validate::ValidationContext {
        crate::validate::ValidationContext::new(
            pse_schema::registry().unwrap(),
            SessionContext::new().state(),
        )
    }
    fn binding(index: u64) -> StatisticsBinding {
        StatisticsBinding {
            payload: "00000000-0000-0000-0000-000000000001".into(),
            digest: "a".repeat(64),
            index,
            interpretation: "pse.substrate.test".into(),
        }
    }
    fn parameters() -> FieldCheckedBatch {
        let context = validation();
        let mut builder = fit_parameters::Builder::new(&context).unwrap();
        for (owner, value) in [
            (3, Some(2.0)),
            (1, None),
            (2, Some(-0.0)),
            (4, Some(f64::from_bits(0x4008_0000_0000_0042))),
        ] {
            builder
                .push(fit_parameters::Row {
                    run_id: RunId::from_bytes([7; 16]),
                    parameter_id: SemanticId::from_bytes([owner; 16]),
                    fixed: false,
                    value,
                    unit_id: SemanticId::NIL,
                    scale: 2.0,
                    at_bound: None,
                })
                .unwrap();
        }
        builder.finish().unwrap()
    }

    #[test]
    fn logical_null_statistics_count_encoded_nulls_and_refuse_before_native_scratch() {
        use arrow::array::{
            DictionaryArray, Int8Array, Int32Array, NullArray, RunArray, StringArray,
        };
        use arrow::datatypes::{Int8Type, Int32Type};
        let dictionary = DictionaryArray::<Int8Type>::try_new(
            Int8Array::from(vec![Some(0), Some(1), Some(1), None]),
            Arc::new(StringArray::from(vec![Some("x"), None])),
        )
        .unwrap();
        let runs = RunArray::<Int32Type>::try_new(
            &Int32Array::from(vec![1, 4]),
            &StringArray::from(vec![Some("x"), None]),
        )
        .unwrap();
        assert_eq!(dictionary.null_count(), 1);
        assert_eq!(runs.null_count(), 0);
        let dictionary: Arc<dyn Array> = Arc::new(dictionary);
        let runs: Arc<dyn Array> = Arc::new(runs);
        let nulls: Arc<dyn Array> = Arc::new(NullArray::new(4));
        let batch = arrow::record_batch::RecordBatch::try_from_iter(vec![
            ("dictionary", dictionary),
            ("runs", runs),
            ("null", nulls),
        ])
        .unwrap();
        let pool = pool();
        let cancel = CancellationToken::new();
        assert_eq!(
            logical_null_counts(&batch, &pool, &cancel).unwrap(),
            [3, 3, 4]
        );
        assert_eq!(pool.reserved(), 0);
        let refused: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(0));
        assert!(logical_null_counts(&batch, &refused, &cancel).is_err());
        assert_eq!(refused.reserved(), 0);
        cancel.cancel();
        assert!(logical_null_counts(&batch, &pool, &cancel).is_err());
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn admitted_file_releases_encoder_scratch_but_keeps_bytes_and_facts_in_custody() {
        let registry = pse_schema::registry().unwrap();
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(512 * 1024));
        let cancel = CancellationToken::new();
        let source = parameters().retained(&pool, &cancel).unwrap();
        let source_extent = pool.reserved();
        let byte_limit = 65536;
        let encoded = Arc::new(
            AdmittedResultFile::encode(
                registry,
                std::slice::from_ref(&source),
                byte_limit,
                &pool,
                &cancel,
            )
            .unwrap(),
        );
        let capacity = encoded.bytes.capacity();
        let facts = encoded.facts()[0].clone();
        let facts_extent = facts.0._owner.size();
        assert_eq!(encoded._encoded.size(), capacity);
        assert_eq!(pool.reserved(), source_extent + capacity + facts_extent);
        let reuse = MemoryConsumer::new("test:released-encoder-scratch").register(&pool);
        reuse.try_grow(pse_columnar::ipc_file::writer_extent(&[source.batch().clone()], byte_limit).unwrap() - capacity).unwrap();
        drop(reuse);

        let reader_owner = encoded.clone();
        drop(encoded);
        drop(source);
        assert_eq!(pool.reserved(), capacity + facts_extent);
        let decoded = arrow::ipc::reader::FileReader::try_new(
            std::io::Cursor::new(reader_owner.bytes()),
            None,
        )
        .unwrap()
        .next()
        .unwrap()
        .unwrap();
        let value = decoded.schema().index_of("value").unwrap();
        let values = decoded
            .column(value)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(values.value(2).to_bits(), (-0.0f64).to_bits());
        assert_eq!(facts.rows(), 4);
        assert!(facts.encode_bound(&binding(0)).is_ok());
        drop(reader_owner);
        assert_eq!(pool.reserved(), facts_extent);
        drop(facts);
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn admitted_file_facts_come_from_actual_sorted_values_and_preserve_bits() {
        let registry = pse_schema::registry().unwrap();
        let pool = pool();
        let cancel = CancellationToken::new();
        let source = parameters();
        let sorted = sort_new_scalar(&source, &pool, &cancel).unwrap();
        let view = fit_parameters::View::from_checked(&sorted).unwrap();
        assert_eq!(
            view.rows()
                .unwrap()
                .iter()
                .map(|row| row.parameter_id)
                .collect::<Vec<_>>(),
            [1, 2, 3, 4].map(|owner| SemanticId::from_bytes([owner; 16]))
        );
        let encoded =
            AdmittedResultFile::encode(registry, &[sorted.clone(), source], 65536, &pool, &cancel)
                .unwrap();
        let bytes = encoded.facts()[0].encode_bound(&binding(0)).unwrap();
        let statistics = recorded_statistics(
            &bytes,
            registry,
            encoded.relation(),
            sorted.batch().schema().as_ref(),
            &binding(0),
        )
        .unwrap();
        assert_eq!(statistics.num_rows, Precision::Exact(4));
        let value = sorted.batch().schema().index_of("value").unwrap();
        assert_eq!(
            statistics.column_statistics[value].null_count,
            Precision::Exact(1)
        );
        assert_eq!(
            statistics.column_statistics[value].min_value,
            Precision::Exact(ScalarValue::Float64(Some(-0.0)))
        );
        assert_eq!(
            statistics.column_statistics[value].max_value,
            Precision::Exact(ScalarValue::Float64(Some(f64::from_bits(
                0x4008_0000_0000_0042
            ))))
        );
        let mut reader =
            arrow::ipc::reader::FileReader::try_new(std::io::Cursor::new(encoded.bytes()), None)
                .unwrap();
        reader.set_index(0).unwrap();
        let decoded = reader.next().unwrap().unwrap();
        let values = decoded
            .column(value)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert!(values.is_null(0));
        assert_eq!(values.value(1).to_bits(), (-0.0f64).to_bits());
        assert_eq!(values.value(2).to_bits(), 2.0f64.to_bits());
        assert_eq!(values.value(3).to_bits(), 0x4008_0000_0000_0042);
        let mut changed = binding(0);
        for field in 0..4 {
            match field {
                0 => changed.digest = "b".repeat(64),
                1 => changed.payload.push('x'),
                2 => changed.index += 1,
                _ => changed.interpretation.push('x'),
            }
            assert!(
                recorded_statistics(
                    &bytes,
                    registry,
                    encoded.relation(),
                    sorted.batch().schema().as_ref(),
                    &changed
                )
                .is_none()
            );
            changed = binding(0);
        }
        assert!(
            recorded_statistics(
                &bytes,
                registry,
                solve_variables::RELATION_ID,
                sorted.batch().schema().as_ref(),
                &binding(0)
            )
            .is_none()
        );
        let mut record: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        record["facts"]["contract"]["encoding"] = serde_json::to_value(ContentHash::NIL).unwrap();
        assert!(
            recorded_statistics(
                &serde_json::to_vec(&record).unwrap(),
                registry,
                encoded.relation(),
                sorted.batch().schema().as_ref(),
                &binding(0)
            )
            .is_none()
        );
        assert_eq!(bytes, encoded.facts()[0].encode_bound(&binding(0)).unwrap());
        drop(encoded);
        drop(sorted);
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn admitted_facts_use_each_declared_key_shape_and_empty_batches() {
        let context = validation();
        let registry = pse_schema::registry().unwrap();
        let pool = pool();
        let cancel = CancellationToken::new();
        let tables = [
            solve_variables::Builder::new(&context)
                .unwrap()
                .finish()
                .unwrap(),
            solve_constraints::Builder::new(&context)
                .unwrap()
                .finish()
                .unwrap(),
            fit_parameters::Builder::new(&context)
                .unwrap()
                .finish()
                .unwrap(),
            fit_observations::Builder::new(&context)
                .unwrap()
                .finish()
                .unwrap(),
        ];
        for (table, keys) in tables.iter().zip([
            &["run_id", "step", "symbol_id"][..],
            &["run_id", "step", "row_id"][..],
            &["run_id", "parameter_id"][..],
            &["run_id", "experiment_id", "observation_id"][..],
        ]) {
            assert_eq!(
                crate::generated::result_selection::for_relation(table.relation_id())
                    .unwrap()
                    .sort,
                keys
            );
            let encoded = AdmittedResultFile::encode(
                registry,
                std::slice::from_ref(table),
                65536,
                &pool,
                &cancel,
            )
            .unwrap();
            let facts = encoded.facts()[0].encode_bound(&binding(0)).unwrap();
            let statistics = recorded_statistics(
                &facts,
                registry,
                table.relation_id(),
                table.batch().schema().as_ref(),
                &binding(0),
            )
            .unwrap();
            assert_eq!(statistics.num_rows, Precision::Exact(0));
            assert!(
                statistics
                    .column_statistics
                    .iter()
                    .all(|column| column.null_count == Precision::Exact(0)
                        && column.min_value == Precision::Absent)
            );
        }
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn original_primary_key_uniqueness_includes_fit_observation_across_experiments() {
        let context = validation();
        let registry = pse_schema::registry().unwrap();
        let mut builder = fit_observations::Builder::new(&context).unwrap();
        for experiment in [1, 2] {
            builder
                .push(fit_observations::Row {
                    run_id: RunId::from_bytes([7; 16]),
                    observation_id: SemanticId::from_bytes([3; 16]),
                    experiment_id: InstanceId::from_bytes([experiment; 16]),
                    included: true,
                    prediction: Some(1.0),
                    residual: Some(0.0),
                    standardized_residual: None,
                    objective_contribution: None,
                    unit_id: SemanticId::NIL,
                })
                .unwrap();
        }
        let table = builder.finish().unwrap();
        let pool = pool();
        assert!(
            AdmittedResultFile::encode(registry, &[table], 65536, &pool, &CancellationToken::new())
                .is_err()
        );
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn nonfinite_values_have_unknown_generic_extrema_and_actual_null_counts() {
        for exceptional in [
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::from_bits(0x7ff8_0000_0000_0042),
        ] {
            let values = Float64Array::from(vec![Some(1.0), None, Some(exceptional)]);
            let (minimum, maximum) = extrema(&values, &CancellationToken::new()).unwrap();
            assert!(minimum.is_none() && maximum.is_none());
            assert_eq!(values.null_count(), 1);
        }
    }

    #[test]
    fn admitted_file_refusal_and_cancellation_release_actual_allowances() {
        let registry = pse_schema::registry().unwrap();
        let table = parameters();
        let small: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(16));
        assert!(
            AdmittedResultFile::encode(
                registry,
                std::slice::from_ref(&table),
                65536,
                &small,
                &CancellationToken::new()
            )
            .is_err()
        );
        assert_eq!(small.reserved(), 0);
        let pool = pool();
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert!(
            AdmittedResultFile::encode(
                registry,
                std::slice::from_ref(&table),
                65536,
                &pool,
                &cancel
            )
            .is_err()
        );
        assert!(sort_new_scalar(&table, &pool, &cancel).is_err());
        assert_eq!(pool.reserved(), 0);
    }
}
