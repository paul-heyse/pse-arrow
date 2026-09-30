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
pub(super) fn decode(path: &str, bytes: &bytes::Bytes) -> Result<RowSet, DriverError> {
    let refuse = |reason: String| contract(None, &format!("data document {path}: {reason}"));
    let builder = ParquetRecordBatchReaderBuilder::try_new(bytes.clone())
        .map_err(|e| refuse(format!("not a Parquet document: {e}")))?;
    let schema = builder.schema().clone();
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
        .build()
        .map_err(|e| refuse(format!("unreadable Parquet: {e}")))?;
    for batch in reader {
        let batch = batch.map_err(|e| refuse(format!("unreadable Parquet: {e}")))?;
        for ((column, members), array) in columns.iter_mut().zip(batch.columns()) {
            append(column, members, array.as_ref()).map_err(|reason| {
                refuse(format!("column {}: {reason}", column.name))
            })?;
        }
    }
    RowSet::new(columns.into_iter().map(|(column, _)| column).collect()).map_err(refuse)
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
                (0..names.len()).map(|i| names.value(i).to_owned()).collect::<Vec<_>>()
            } else {
                let names = dictionary.as_string::<i64>();
                (0..names.len()).map(|i| names.value(i).to_owned()).collect()
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
                    Some(*global.get(key).ok_or_else(|| {
                        format!("row {row} indexes outside its dictionary")
                    })?)
                });
            }
        }
    }
    Ok(())
}
