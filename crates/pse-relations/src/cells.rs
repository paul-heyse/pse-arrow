// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Lossless conversion between the registry's `Cell` values and declared Arrow layouts.

use std::sync::Arc;

use arrow::buffer::{NullBuffer, OffsetBuffer};
use arrow_array::builder::{FixedSizeBinaryBuilder, StringDictionaryBuilder};
use arrow_array::types::{Int8Type, Int32Type};
use arrow_array::{
    Array, ArrayRef, BooleanArray, DictionaryArray, FixedSizeBinaryArray, FixedSizeListArray,
    Float64Array, Int16Array, Int32Array, Int64Array, ListArray, RecordBatch, StringArray,
    StructArray, TimestampNanosecondArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use arrow_schema::{DataType, Field, TimeUnit};
use pse_ids::{ContentHash, SemanticId};
use pse_schema::Registry;
use pse_schema::model::{Cell, RelationSpec};

use crate::RelationError;

mod owned;
pub use owned::{batch_from_cells_owned, batch_from_columns_owned};

/// Builds a batch under an exact declared schema and admits its visible logical values.
/// Keys and cross-relation constraints are checked separately at bundle admission.
///
/// # Errors
/// A typed error for row width, cell kind, numeric range, layout or value violations.
pub fn batch_from_cells(
    reg: &Registry,
    spec: &RelationSpec,
    rows: &[Vec<Cell>],
) -> Result<RecordBatch, RelationError> {
    let schema = Arc::new(pse_schema::arrow::relation_schema(reg, spec)?);
    let batch = build_batch(schema, rows)?;
    crate::validate::validate_batch(reg, spec, &batch)
        .map_err(|errors| RelationError::Validation { errors })?;
    Ok(batch)
}

fn build_batch(
    schema: Arc<arrow_schema::Schema>,
    rows: &[Vec<Cell>],
) -> Result<RecordBatch, RelationError> {
    for (row, values) in rows.iter().enumerate() {
        if values.len() != schema.fields().len() {
            return Err(value_error(
                "cell rows",
                row,
                "row width differs from the declared columns",
            ));
        }
    }
    let arrays = schema
        .fields()
        .iter()
        .enumerate()
        .map(|(column, field)| {
            let values = rows
                .iter()
                .map(|row| row[column].clone())
                .collect::<Vec<_>>();
            storage_array_from_cells(field, &values)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let batch = RecordBatch::try_new_with_options(
        schema,
        arrays,
        &arrow_array::RecordBatchOptions::new().with_row_count(Some(rows.len())),
    )?;
    Ok(batch)
}

/// Decodes admitted visible values; dictionary codes never escape this boundary.
///
/// # Errors
/// All schema and logical-value admission failures, or a mismatched array implementation.
pub fn cells_from_batch(
    reg: &Registry,
    spec: &RelationSpec,
    batch: &RecordBatch,
) -> Result<Vec<Vec<Cell>>, RelationError> {
    crate::validate::validate_batch(reg, spec, batch)
        .map_err(|errors| RelationError::Validation { errors })?;
    decode_batch(reg, batch)
}

/// Decode a declared field subset, such as a rule's violation key head. Each column
/// passes the same field/value checks as full relation admission; this does not certify
/// relation membership, key uniqueness or cross-column contracts.
///
/// # Errors
/// Invalid field metadata, array storage or visible recursive values.
pub fn decode_columns(
    reg: &Registry,
    batch: &RecordBatch,
) -> Result<Vec<Vec<Cell>>, RelationError> {
    for (field, array) in batch.schema().fields().iter().zip(batch.columns()) {
        crate::validate::validate_column(reg, field, array.as_ref())
            .map_err(|errors| RelationError::Validation { errors })?;
    }
    decode_batch(reg, batch)
}

pub(crate) fn decode_batch(
    reg: &Registry,
    batch: &RecordBatch,
) -> Result<Vec<Vec<Cell>>, RelationError> {
    (0..batch.num_rows())
        .map(|row| {
            batch
                .schema()
                .fields()
                .iter()
                .zip(batch.columns())
                .map(|(field, array)| cell_at(reg, field, array.as_ref(), row))
                .collect()
        })
        .collect()
}

pub(crate) fn value_error(field: &str, row: usize, reason: &str) -> RelationError {
    RelationError::Value {
        field: field.to_owned(),
        row,
        reason: reason.to_owned(),
    }
}

/// Construct one declared logical column and validate its visible recursive values.
/// Cross-column quantity contracts and relation keys remain batch/bundle obligations.
///
/// # Errors
/// Invalid field metadata, cell/storage mismatch or visible logical-value violation.
pub fn array_from_cells(
    reg: &Registry,
    field: &Field,
    values: &[Cell],
) -> Result<ArrayRef, RelationError> {
    crate::validate::validate_field(reg, field)
        .map_err(|errors| RelationError::Validation { errors })?;
    let array = storage_array_from_cells(field, values)?;
    crate::validate::validate_column(reg, field, array.as_ref())
        .map_err(|errors| RelationError::Validation { errors })?;
    Ok(array)
}

fn scalar<'a, T>(
    field: &Field,
    values: &'a [Cell],
    convert: impl Fn(&'a Cell) -> Option<T>,
) -> Result<Vec<Option<T>>, RelationError> {
    values
        .iter()
        .enumerate()
        .map(|(row, cell)| match cell {
            Cell::Null => Ok(None),
            cell => convert(cell).map(Some).ok_or_else(|| {
                value_error(
                    field.name(),
                    row,
                    "cell kind or numeric range differs from declared storage",
                )
            }),
        })
        .collect()
}

fn storage_array_from_cells(field: &Field, values: &[Cell]) -> Result<ArrayRef, RelationError> {
    macro_rules! signed {
        ($array:ty,$native:ty) => {
            Arc::new(<$array>::from(scalar(field, values, |cell| match cell {
                Cell::I64(value) => <$native>::try_from(*value).ok(),
                _ => None,
            })?))
        };
    }
    macro_rules! unsigned {
        ($array:ty,$native:ty) => {
            Arc::new(<$array>::from(scalar(field, values, |cell| match cell {
                Cell::U64(value) => <$native>::try_from(*value).ok(),
                _ => None,
            })?))
        };
    }
    Ok(match field.data_type() {
        DataType::Boolean => Arc::new(BooleanArray::from(scalar(field, values, |cell| {
            if let Cell::Bool(value) = cell {
                Some(*value)
            } else {
                None
            }
        })?)),
        DataType::Int16 => signed!(Int16Array, i16),
        DataType::Int32 => signed!(Int32Array, i32),
        DataType::Int64 => signed!(Int64Array, i64),
        DataType::UInt8 => unsigned!(UInt8Array, u8),
        DataType::UInt16 => unsigned!(UInt16Array, u16),
        DataType::UInt32 => unsigned!(UInt32Array, u32),
        DataType::UInt64 => unsigned!(UInt64Array, u64),
        DataType::Float64 => Arc::new(Float64Array::from(scalar(field, values, |cell| {
            if let Cell::F64(value) = cell {
                Some(*value)
            } else {
                None
            }
        })?)),
        DataType::Utf8 => Arc::new(StringArray::from(scalar(
            field,
            values,
            |cell| match cell {
                Cell::Text(value) => Some(value.as_str()),
                Cell::Enum(value) if field.metadata().contains_key(pse_schema::arrow::KEY_ENUM) => {
                    Some(*value)
                }
                _ => None,
            },
        )?)),
        DataType::Timestamp(TimeUnit::Nanosecond, zone) if zone.as_deref() == Some("UTC") => {
            Arc::new(
                TimestampNanosecondArray::from(scalar(field, values, |cell| {
                    if let Cell::I64(value) = cell {
                        Some(*value)
                    } else {
                        None
                    }
                })?)
                .with_timezone("UTC"),
            )
        }
        DataType::FixedSizeBinary(width @ (16 | 32)) => binary_array(field, values, *width)?,
        DataType::Dictionary(key, value) if value.as_ref() == &DataType::Utf8 => {
            dictionary_array(field, values, key)?
        }
        DataType::List(child) => list_array(field, child, values, None)?,
        DataType::FixedSizeList(child, width) => list_array(field, child, values, Some(*width))?,
        DataType::Struct(children) => struct_array(field, children, values)?,
        other => {
            return Err(value_error(
                field.name(),
                0,
                &format!("unsupported cell layout {other}"),
            ));
        }
    })
}

fn binary_array(field: &Field, values: &[Cell], width: i32) -> Result<ArrayRef, RelationError> {
    let mut builder = FixedSizeBinaryBuilder::with_capacity(values.len(), width);
    for (row, cell) in values.iter().enumerate() {
        match (width, cell) {
            (_, Cell::Null) => builder.append_null(),
            (16, Cell::Id(value)) => builder.append_value(value.as_bytes())?,
            (32, Cell::Hash(value)) => builder.append_value(value.as_bytes())?,
            _ => {
                return Err(value_error(
                    field.name(),
                    row,
                    "fixed binary requires the declared identity or hash cell",
                ));
            }
        }
    }
    Ok(Arc::new(builder.finish()))
}

fn dictionary_array(
    field: &Field,
    values: &[Cell],
    key: &DataType,
) -> Result<ArrayRef, RelationError> {
    macro_rules! build {
        ($ty:ty) => {{
            let mut builder = StringDictionaryBuilder::<$ty>::new();
            for (row, cell) in values.iter().enumerate() {
                match cell {
                    Cell::Null => builder.append_null(),
                    Cell::Enum(value) => {
                        builder.append(*value)?;
                    }
                    _ => {
                        return Err(value_error(
                            field.name(),
                            row,
                            "dictionary requires an enumeration cell",
                        ));
                    }
                }
            }
            {
                let array: ArrayRef = Arc::new(builder.finish());
                array
            }
        }};
    }
    Ok(match key {
        DataType::Int8 => build!(Int8Type),
        DataType::Int32 => build!(Int32Type),
        _ => {
            return Err(value_error(
                field.name(),
                0,
                "unsupported dictionary key storage",
            ));
        }
    })
}

fn list_array(
    field: &Field,
    child: &Field,
    values: &[Cell],
    width: Option<i32>,
) -> Result<ArrayRef, RelationError> {
    let fixed = width
        .map(usize::try_from)
        .transpose()
        .map_err(|_| value_error(field.name(), 0, "negative fixed list width"))?;
    let mut flattened = Vec::new();
    let mut lengths = Vec::with_capacity(values.len());
    for (row, cell) in values.iter().enumerate() {
        match cell {
            Cell::Null => {
                let count = fixed.unwrap_or(0);
                flattened.extend((0..count).map(|_| default_cell(child.data_type())));
                lengths.push(count);
            }
            Cell::List(items) if fixed.is_none_or(|count| count == items.len()) => {
                lengths.push(items.len());
                flattened.extend_from_slice(items);
            }
            _ => {
                return Err(value_error(
                    field.name(),
                    row,
                    "list cell or fixed length mismatch",
                ));
            }
        }
    }
    let array = storage_array_from_cells(child, &flattened)?;
    let nulls = Some(NullBuffer::from(
        values
            .iter()
            .map(|cell| !matches!(cell, Cell::Null))
            .collect::<Vec<_>>(),
    ));
    Ok(if let Some(width) = width {
        Arc::new(FixedSizeListArray::try_new_with_length(
            Arc::new(child.clone()),
            width,
            array,
            nulls,
            values.len(),
        )?)
    } else {
        let total = lengths
            .iter()
            .try_fold(0usize, |sum, value| sum.checked_add(*value))
            .ok_or_else(|| value_error(field.name(), 0, "list offset overflow"))?;
        i32::try_from(total)
            .map_err(|_| value_error(field.name(), 0, "list offsets exceed Int32 storage"))?;
        Arc::new(ListArray::try_new(
            Arc::new(child.clone()),
            OffsetBuffer::<i32>::from_lengths(lengths),
            array,
            nulls,
        )?)
    })
}

fn struct_array(
    field: &Field,
    children: &arrow_schema::Fields,
    values: &[Cell],
) -> Result<ArrayRef, RelationError> {
    let mut columns = vec![Vec::with_capacity(values.len()); children.len()];
    for (row, cell) in values.iter().enumerate() {
        match cell {
            Cell::Null => {
                for (column, child) in columns.iter_mut().zip(children) {
                    column.push(default_cell(child.data_type()));
                }
            }
            Cell::Struct(items) if items.len() == children.len() => {
                for (column, item) in columns.iter_mut().zip(items) {
                    column.push(item.clone());
                }
            }
            _ => {
                return Err(value_error(
                    field.name(),
                    row,
                    "struct cell width differs from declared children",
                ));
            }
        }
    }
    let arrays = children
        .iter()
        .zip(columns)
        .map(|(child, cells)| storage_array_from_cells(child, &cells))
        .collect::<Result<Vec<_>, _>>()?;
    let nulls = Some(NullBuffer::from(
        values
            .iter()
            .map(|cell| !matches!(cell, Cell::Null))
            .collect::<Vec<_>>(),
    ));
    Ok(Arc::new(StructArray::try_new_with_length(
        children.clone(),
        arrays,
        nulls,
        values.len(),
    )?))
}

fn default_cell(ty: &DataType) -> Cell {
    match ty {
        DataType::Boolean => Cell::Bool(false),
        DataType::Int16 | DataType::Int32 | DataType::Int64 | DataType::Timestamp(..) => {
            Cell::I64(0)
        }
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => Cell::U64(0),
        DataType::Float64 => Cell::F64(0.0),
        DataType::Utf8 => Cell::Text(String::new()),
        DataType::FixedSizeBinary(16) => Cell::Id(SemanticId::from_bytes([0; 16])),
        DataType::FixedSizeBinary(32) => Cell::Hash(ContentHash::from_bytes([0; 32])),
        DataType::Dictionary(..) => Cell::Enum(""),
        DataType::List(_) => Cell::List(Vec::new()),
        DataType::FixedSizeList(child, width) => Cell::List(
            (0..*width)
                .map(|_| default_cell(child.data_type()))
                .collect(),
        ),
        DataType::Struct(children) => Cell::Struct(
            children
                .iter()
                .map(|child| default_cell(child.data_type()))
                .collect(),
        ),
        _ => Cell::Null,
    }
}

fn downcast<'a, T: 'static>(array: &'a dyn Array, field: &Field) -> Result<&'a T, RelationError> {
    array.as_any().downcast_ref::<T>().ok_or_else(|| {
        value_error(
            field.name(),
            0,
            "array implementation differs from declared storage",
        )
    })
}

pub(crate) fn cell_at(
    reg: &Registry,
    field: &Field,
    array: &dyn Array,
    row: usize,
) -> Result<Cell, RelationError> {
    if array.is_null(row) {
        return Ok(Cell::Null);
    }
    macro_rules! get {
        ($ty:ty,$variant:ident,$convert:expr) => {
            Cell::$variant(($convert)(downcast::<$ty>(array, field)?.value(row)))
        };
    }
    Ok(match field.data_type() {
        DataType::Boolean => get!(BooleanArray, Bool, core::convert::identity),
        DataType::Int16 => get!(Int16Array, I64, i64::from),
        DataType::Int32 => get!(Int32Array, I64, i64::from),
        DataType::Int64 => get!(Int64Array, I64, core::convert::identity),
        DataType::UInt8 => get!(UInt8Array, U64, u64::from),
        DataType::UInt16 => get!(UInt16Array, U64, u64::from),
        DataType::UInt32 => get!(UInt32Array, U64, u64::from),
        DataType::UInt64 => get!(UInt64Array, U64, core::convert::identity),
        DataType::Float64 => get!(Float64Array, F64, core::convert::identity),
        DataType::Utf8 => {
            let text = downcast::<StringArray>(array, field)?.value(row);
            decode_text(reg, field, text)?
        }
        DataType::Timestamp(TimeUnit::Nanosecond, zone) if zone.as_deref() == Some("UTC") => {
            get!(TimestampNanosecondArray, I64, core::convert::identity)
        }
        DataType::FixedSizeBinary(16) => Cell::Id(SemanticId::from_bytes(
            downcast::<FixedSizeBinaryArray>(array, field)?
                .value(row)
                .try_into()
                .map_err(|_| value_error(field.name(), row, "invalid identity width"))?,
        )),
        DataType::FixedSizeBinary(32) => Cell::Hash(ContentHash::from_bytes(
            downcast::<FixedSizeBinaryArray>(array, field)?
                .value(row)
                .try_into()
                .map_err(|_| value_error(field.name(), row, "invalid hash width"))?,
        )),
        DataType::Dictionary(key, _) => decode_dictionary(reg, field, array, row, key)?,
        DataType::List(child) => {
            let value = downcast::<ListArray>(array, field)?.value(row);
            Cell::List(
                (0..value.len())
                    .map(|index| cell_at(reg, child, value.as_ref(), index))
                    .collect::<Result<_, _>>()?,
            )
        }
        DataType::FixedSizeList(child, _) => {
            let value = downcast::<FixedSizeListArray>(array, field)?.value(row);
            Cell::List(
                (0..value.len())
                    .map(|index| cell_at(reg, child, value.as_ref(), index))
                    .collect::<Result<_, _>>()?,
            )
        }
        DataType::Struct(children) => {
            let value = downcast::<StructArray>(array, field)?;
            Cell::Struct(
                children
                    .iter()
                    .zip(value.columns())
                    .map(|(child, array)| cell_at(reg, child, array.as_ref(), row))
                    .collect::<Result<_, _>>()?,
            )
        }
        other => {
            return Err(value_error(
                field.name(),
                row,
                &format!("unsupported cell layout {other}"),
            ));
        }
    })
}

fn decode_dictionary(
    reg: &Registry,
    field: &Field,
    array: &dyn Array,
    row: usize,
    key: &DataType,
) -> Result<Cell, RelationError> {
    macro_rules! value {
        ($ty:ty) => {{
            let dictionary = downcast::<DictionaryArray<$ty>>(array, field)?;
            let index = dictionary
                .key(row)
                .ok_or_else(|| value_error(field.name(), row, "null dictionary key"))?;
            let values = downcast::<StringArray>(dictionary.values().as_ref(), field)?;
            if values.is_null(index) {
                return Err(value_error(
                    field.name(),
                    row,
                    "referenced dictionary member is null",
                ));
            }
            values.value(index)
        }};
    }
    let text = match key {
        DataType::Int8 => value!(Int8Type),
        DataType::Int32 => value!(Int32Type),
        _ => return Err(value_error(field.name(), row, "unsupported dictionary key")),
    };
    decode_text(reg, field, text)
}

fn decode_text(reg: &Registry, field: &Field, text: &str) -> Result<Cell, RelationError> {
    if let Some(id) = field.metadata().get(pse_schema::arrow::KEY_ENUM) {
        let enum_id = SemanticId::parse_hex(id).map_err(|_| RelationError::UnknownRegistry {
            relation: format!("enum:{id}"),
        })?;
        let enumeration = reg
            .enums()
            .iter()
            .find(|spec| spec.id == enum_id)
            .ok_or_else(|| RelationError::UnknownRegistry {
                relation: format!("enum:{id}"),
            })?;
        return enumeration
            .members
            .iter()
            .find(|member| member.name == text)
            .map(|member| Cell::Enum(member.name))
            .ok_or_else(|| RelationError::EnumMember {
                field: field.name().to_owned(),
                enumeration: enumeration.name.to_owned(),
                value: text.to_owned(),
            });
    }
    Ok(Cell::Text(text.to_owned()))
}
