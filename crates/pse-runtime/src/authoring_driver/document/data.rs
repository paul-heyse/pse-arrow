// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Package data documents decoded by Arrow type (ADR-0125).
//!
//! A `data/*.parquet` document is decoded here, and only here, into the Arrow-free
//! [`RowSet`] the modeling kernel admits through a dataset's declared relation. Each column
//! is read by its Arrow type: `Float64` magnitudes, `FixedSizeBinary(16)` identities, `Utf8`
//! values, `Int64` integers, `Boolean`s and dictionaries of `Utf8` member names. Any other
//! type is refused with the column named. A column's `unit` metadata is carried as stated;
//! admission refuses it unless it is the declared storage unit, so it is never a second
//! authority.
use super::allocation::{Allocation, add, mul};
use super::load::contract;
use crate::authoring_driver::DriverError;
use datafusion::arrow::array::{Array, AsArray};
use datafusion::arrow::datatypes::{DataType, Float64Type, Int64Type};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use pse_ids::SemanticId;
use pse_modeling::document::{DocumentColumn, RowSet, Values};
use std::collections::HashMap;

/// The field metadata key under which a document may state a column's unit.
pub const UNIT_METADATA: &str = "unit";

/// Decode the Parquet bytes of the data document at `path`.
///
/// # Errors
/// Bytes that are not Parquet, a nested or unsupported Arrow type, or a dictionary whose
/// values are not `Utf8` member names.
pub(super) fn decode(
    path: &str,
    bytes: &bytes::Bytes,
    budget: pse_authoring::ParseBudget,
    mut allocation: Option<&mut Allocation<'_>>,
) -> Result<RowSet, DriverError> {
    let refuse = |reason: String| contract(None, &format!("data document {path}: {reason}"));
    let start = allocation.as_ref().map_or(0, |funds| funds.size());
    // Metadata and compressed-page storage are bounded before the reader is constructed.
    if let Some(funds) = allocation.as_deref_mut() {
        funds.grow(mul(bytes.len(), 8)?)?;
    }
    let builder = ParquetRecordBatchReaderBuilder::try_new(bytes.clone())
        .map_err(|e| refuse(format!("not a Parquet document: {e}")))?;
    let schema = builder.schema().clone();
    let rows = u64::try_from(builder.metadata().file_metadata().num_rows())
        .map_err(|_| refuse("negative row count".into()))?;
    if rows > budget.max_data_rows {
        return Err(DriverError::Authoring(
            pse_authoring::AuthoringError::Budget {
                limit: "data document rows",
                allowed: budget.max_data_rows,
                needed: rows,
            },
        ));
    }
    const BATCH_ROWS: usize = 1024;
    let batch_rows = BATCH_ROWS
        .min(usize::try_from(rows).map_err(|_| refuse("row count exceeds address space".into()))?);
    // A text value cannot exceed its entire uncompressed column chunk. This deliberately
    // overestimates dictionary expansion while charging only one bounded output batch.
    let mut transient = 0;
    for group in builder.metadata().row_groups() {
        let mut group_bytes = 0;
        for (column, field) in group.columns().iter().zip(schema.fields()) {
            let decoded = usize::try_from(column.uncompressed_size())
                .map_err(|_| refuse("invalid uncompressed column size".into()))?;
            let text = matches!(
                field.data_type(),
                DataType::Utf8 | DataType::LargeUtf8 | DataType::Dictionary(_, _)
            );
            group_bytes = add(
                group_bytes,
                add(
                    mul(decoded, 4)?,
                    mul(batch_rows, if text { add(decoded, 128)? } else { 128 })?,
                )?,
            )?;
        }
        transient = transient.max(group_bytes);
    }
    if let Some(funds) = allocation.as_deref_mut() {
        funds.grow(add(mul(transient, 4)?, mul(schema.fields().len(), 1024)?)?)?;
    }
    let mut columns = schema
        .fields()
        .iter()
        .map(|field| {
            let values = match field.data_type() {
                DataType::Float64 => Values::Magnitude(Vec::new()),
                DataType::FixedSizeBinary(16) => Values::Identity(Vec::new()),
                DataType::Utf8 | DataType::LargeUtf8 => Values::Text(Vec::new()),
                DataType::Int64 => Values::Integer(Vec::new()),
                DataType::Boolean => Values::Boolean(Vec::new()),
                DataType::Dictionary(_, value) if matches!(value.as_ref(), DataType::Utf8 | DataType::LargeUtf8) => {
                    Values::Member {
                        members: Vec::new(),
                        keys: Vec::new(),
                    }
                }
                other => {
                    return Err(refuse(format!(
                        "column {} has Arrow type {other}; a data document stores Float64, FixedSizeBinary(16), Utf8, Int64, Boolean or a Utf8 dictionary",
                        field.name()
                    )));
                }
            };
            Ok((
                DocumentColumn {
                    name: field.name().clone(),
                    unit: field.metadata().get(UNIT_METADATA).cloned(),
                    values,
                },
                HashMap::<String, u32>::new(),
            ))
        })
        .collect::<Result<Vec<_>, DriverError>>()?;
    let reader = builder
        .with_batch_size(BATCH_ROWS)
        .build()
        .map_err(|e| refuse(format!("unreadable Parquet: {e}")))?;
    for batch in reader {
        if let Some(funds) = allocation.as_deref_mut() {
            funds.cancel.checkpoint()?;
        }
        let batch = batch.map_err(|e| refuse(format!("unreadable Parquet: {e}")))?;
        for ((column, members), array) in columns.iter_mut().zip(batch.columns()) {
            // Charge every owned vector growth and string/dictionary copy before append.
            let growth = append_extent(array.as_ref())?;
            if let Some(funds) = allocation.as_deref_mut() {
                funds.grow(growth)?;
            }
            append(column, members, array.as_ref())
                .map_err(|reason| refuse(format!("column {}: {reason}", column.name)))?;
        }
    }
    let rows =
        RowSet::new(columns.into_iter().map(|(column, _)| column).collect()).map_err(refuse)?;
    if let Some(funds) = allocation {
        funds.retain(start, rows.retained_bytes())?;
    }
    Ok(rows)
}

fn append_extent(array: &dyn Array) -> Result<usize, DriverError> {
    let strings = if let Some(values) = array.as_string_opt::<i32>() {
        (0..values.len()).try_fold(0, |sum, row| add(sum, values.value(row).len()))?
    } else if let Some(values) = array.as_string_opt::<i64>() {
        (0..values.len()).try_fold(0, |sum, row| add(sum, values.value(row).len()))?
    } else if let Some(dictionary) = array.as_any_dictionary_opt() {
        append_extent(dictionary.values().as_ref())?
    } else {
        0
    };
    // Doubling covers Vec capacity growth; 128 bytes per row covers Option payloads,
    // dictionary names, keys and hash-map buckets in addition to copied string bytes.
    mul(add(mul(array.len(), 128)?, strings)?, 2)
}

/// Append one batch's values of a column, by its Arrow type.
fn append(
    column: &mut DocumentColumn,
    index: &mut HashMap<String, u32>,
    array: &dyn Array,
) -> Result<(), String> {
    let present = |row: usize| array.is_valid(row);
    match &mut column.values {
        Values::Magnitude(values) => {
            let array = array.as_primitive::<Float64Type>();
            values.extend((0..array.len()).map(|row| present(row).then(|| array.value(row))));
        }
        Values::Integer(values) => {
            let array = array.as_primitive::<Int64Type>();
            values.extend((0..array.len()).map(|row| present(row).then(|| array.value(row))));
        }
        Values::Boolean(values) => {
            let array = array.as_boolean();
            values.extend((0..array.len()).map(|row| present(row).then(|| array.value(row))));
        }
        Values::Identity(values) => {
            let array = array.as_fixed_size_binary();
            for row in 0..array.len() {
                values.push(if present(row) {
                    let bytes: [u8; 16] = array
                        .value(row)
                        .try_into()
                        .map_err(|_| "an identity is 16 bytes".to_owned())?;
                    Some(SemanticId::from_bytes(bytes))
                } else {
                    None
                });
            }
        }
        Values::Text(values) => {
            if let Some(array) = array.as_string_opt::<i32>() {
                values.extend(
                    (0..array.len()).map(|row| present(row).then(|| array.value(row).to_owned())),
                );
            } else {
                let array = array.as_string::<i64>();
                values.extend(
                    (0..array.len()).map(|row| present(row).then(|| array.value(row).to_owned())),
                );
            }
        }
        Values::Member { members, keys } => {
            let array = array
                .as_any_dictionary_opt()
                .ok_or_else(|| "a dictionary column is not a dictionary".to_owned())?;
            let dictionary = array.values();
            let names = if let Some(names) = dictionary.as_string_opt::<i32>() {
                (0..names.len())
                    .map(|i| names.value(i).to_owned())
                    .collect::<Vec<_>>()
            } else {
                let names = dictionary.as_string::<i64>();
                (0..names.len())
                    .map(|i| names.value(i).to_owned())
                    .collect()
            };
            // Each batch may carry its own dictionary; members are one list by name.
            let global = names
                .into_iter()
                .map(|name| {
                    let next = u32::try_from(members.len())
                        .map_err(|_| "too many dictionary members".to_owned())?;
                    Ok(*index.entry(name.clone()).or_insert_with(|| {
                        members.push(name);
                        next
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let nulls = array.logical_nulls();
            for (row, key) in array.normalized_keys().into_iter().enumerate() {
                keys.push(if nulls.as_ref().is_some_and(|n| n.is_null(row)) {
                    None
                } else {
                    Some(
                        *global
                            .get(key)
                            .ok_or_else(|| format!("row {row} indexes outside its dictionary"))?,
                    )
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::arrow::array::{RecordBatch, StringArray};
    use datafusion::arrow::datatypes::{Field, Schema};
    use pse_columnar::{CancellationToken, GreedyMemoryPool, MemoryPool};
    use std::sync::Arc;

    #[test]
    fn compressed_data_is_charged_for_decoded_growth_and_releases_failed_reservations() {
        let schema = Arc::new(Schema::new(vec![Field::new("text", DataType::Utf8, false)]));
        let text = "x".repeat(1024);
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(StringArray::from(vec![text.as_str(); 3000]))],
        )
        .unwrap();
        let mut bytes = Vec::new();
        let mut writer = parquet::arrow::ArrowWriter::try_new(&mut bytes, schema, None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();
        let bytes = bytes::Bytes::from(bytes);
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(512 << 20));
        let cancel = CancellationToken::new();
        let mut funds = Allocation::new(&pool, &cancel);
        let rows = decode(
            "bank.parquet",
            &bytes,
            pse_authoring::ParseBudget::default(),
            Some(&mut funds),
        )
        .unwrap();
        assert_eq!(rows.rows(), 3000);
        assert!(rows.retained_bytes() > bytes.len() * 10);
        assert_eq!(pool.reserved(), rows.retained_bytes());
        let lease = funds.finish();
        drop(rows);
        drop(lease);
        assert_eq!(pool.reserved(), 0);
        let small: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(bytes.len() * 16 + 65536));
        let mut funds = Allocation::new(&small, &cancel);
        assert!(
            decode(
                "bank.parquet",
                &bytes,
                pse_authoring::ParseBudget::default(),
                Some(&mut funds)
            )
            .is_err()
        );
        drop(funds);
        assert_eq!(small.reserved(), 0);
        let mut funds = Allocation::new(&pool, &cancel);
        let budget = pse_authoring::ParseBudget {
            max_data_rows: 2999,
            ..pse_authoring::ParseBudget::default()
        };
        assert!(
            decode("bank.parquet", &bytes, budget, Some(&mut funds))
                .unwrap_err()
                .to_string()
                .contains("data document rows")
        );
        drop(funds);
        assert_eq!(pool.reserved(), 0);
        cancel.cancel();
        let mut funds = Allocation::new(&pool, &cancel);
        assert!(decode("bank.parquet", &bytes, budget, Some(&mut funds)).is_err());
        drop(funds);
        assert_eq!(pool.reserved(), 0);
    }
}
