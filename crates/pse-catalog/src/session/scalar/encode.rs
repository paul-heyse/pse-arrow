// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The tagged literal wire codec over actual Arrow storage, including null parents.

use datafusion::arrow::array::types::{Int8Type, Int32Type};
use datafusion::arrow::array::{
    Array, BooleanArray, DictionaryArray, FixedSizeBinaryArray, FixedSizeListArray, Float64Array,
    Int8Array, Int16Array, Int32Array, Int64Array, LargeListArray, LargeStringArray, ListArray,
    StringArray, StringViewArray, StructArray, TimestampNanosecondArray, UInt8Array, UInt16Array,
    UInt32Array, UInt64Array,
};
use datafusion::arrow::datatypes::{DataType, Field, TimeUnit};
use datafusion::common::{DataFusionError, Result};
use std::fmt::Write;

use super::invalid;

#[cfg(test)]
mod tests;

pub(super) fn admit_type(kind: &DataType) -> Result<()> {
    match kind {
        DataType::Null
        | DataType::Boolean
        | DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64
        | DataType::Float64
        | DataType::Utf8
        | DataType::LargeUtf8
        | DataType::Utf8View
        | DataType::FixedSizeBinary(16 | 32)
        | DataType::Timestamp(TimeUnit::Nanosecond, _) => Ok(()),
        DataType::List(child) | DataType::LargeList(child) | DataType::FixedSizeList(child, _) => {
            admit_type(child.data_type())
        }
        DataType::Struct(children) => children
            .iter()
            .try_for_each(|child| admit_type(child.data_type())),
        DataType::Dictionary(key, value)
            if matches!(key.as_ref(), DataType::Int8 | DataType::Int32)
                && value.as_ref() == &DataType::Utf8 =>
        {
            Ok(())
        }
        _ => Err(invalid(
            "value has no declared tagged literal representation",
        )),
    }
}

fn downcast<T: Array + 'static>(array: &dyn Array) -> Result<&T> {
    array
        .as_any()
        .downcast_ref::<T>()
        .ok_or_else(|| invalid("actual Arrow array differs from its datatype"))
}

fn quoted(value: &str, output: &mut String) -> Result<()> {
    output.push_str(
        &serde_json::to_string(value)
            .map_err(|error| DataFusionError::External(Box::new(error)))?,
    );
    Ok(())
}

fn number(value: impl std::fmt::Display, output: &mut String) -> Result<()> {
    write!(output, "{value}").map_err(|error| DataFusionError::External(Box::new(error)))
}

pub(super) fn value(
    array: &dyn Array,
    field: &Field,
    row: usize,
    output: &mut String,
) -> Result<()> {
    if row >= array.len() {
        return Err(invalid("encoded row lies outside the actual array"));
    }
    if array.is_null(row) {
        output.push_str("[\"null\",null]");
        return Ok(());
    }
    let tag = value_tag(array, field)?;
    output.push_str("[\"");
    output.push_str(tag);
    output.push_str("\",");
    match array.data_type() {
        DataType::Boolean => number(downcast::<BooleanArray>(array)?.value(row), output)?,
        DataType::Int8 => number(downcast::<Int8Array>(array)?.value(row), output)?,
        DataType::Int16 => number(downcast::<Int16Array>(array)?.value(row), output)?,
        DataType::Int32 => number(downcast::<Int32Array>(array)?.value(row), output)?,
        DataType::Int64 => number(downcast::<Int64Array>(array)?.value(row), output)?,
        DataType::UInt8 => number(downcast::<UInt8Array>(array)?.value(row), output)?,
        DataType::UInt16 => number(downcast::<UInt16Array>(array)?.value(row), output)?,
        DataType::UInt32 => number(downcast::<UInt32Array>(array)?.value(row), output)?,
        DataType::UInt64 => number(downcast::<UInt64Array>(array)?.value(row), output)?,
        DataType::Timestamp(TimeUnit::Nanosecond, _) => number(
            downcast::<TimestampNanosecondArray>(array)?.value(row),
            output,
        )?,
        DataType::Float64 => {
            write!(
                output,
                "\"{:016x}\"",
                downcast::<Float64Array>(array)?.value(row).to_bits()
            )
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        }
        DataType::Utf8 => quoted(downcast::<StringArray>(array)?.value(row), output)?,
        DataType::LargeUtf8 => quoted(downcast::<LargeStringArray>(array)?.value(row), output)?,
        DataType::Utf8View => quoted(downcast::<StringViewArray>(array)?.value(row), output)?,
        DataType::FixedSizeBinary(16 | 32) => {
            output.push('"');
            for byte in downcast::<FixedSizeBinaryArray>(array)?.value(row) {
                write!(output, "{byte:02x}")
                    .map_err(|error| DataFusionError::External(Box::new(error)))?;
            }
            output.push('"');
        }
        DataType::Dictionary(key, _) => dictionary(array, key, row, output)?,
        DataType::List(child) => {
            let array = downcast::<ListArray>(array)?;
            let offsets = array.value_offsets();
            let start =
                usize::try_from(offsets[row]).map_err(|_| invalid("negative list offset"))?;
            let end =
                usize::try_from(offsets[row + 1]).map_err(|_| invalid("negative list offset"))?;
            list(array.values().as_ref(), child, start..end, output)?;
        }
        DataType::LargeList(child) => {
            let array = downcast::<LargeListArray>(array)?;
            let offsets = array.value_offsets();
            let start =
                usize::try_from(offsets[row]).map_err(|_| invalid("negative list offset"))?;
            let end =
                usize::try_from(offsets[row + 1]).map_err(|_| invalid("negative list offset"))?;
            list(array.values().as_ref(), child, start..end, output)?;
        }
        DataType::FixedSizeList(child, width) => {
            let array = downcast::<FixedSizeListArray>(array)?;
            let start = usize::try_from(array.value_offset(row))
                .map_err(|_| invalid("negative list offset"))?;
            let count = usize::try_from(*width).map_err(|_| invalid("negative list width"))?;
            list(array.values().as_ref(), child, start..start + count, output)?;
        }
        DataType::Struct(fields) => {
            let array = downcast::<StructArray>(array)?;
            output.push('[');
            for (index, field) in fields.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                value(array.column(index).as_ref(), field, row, output)?;
            }
            output.push(']');
        }
        _ => return Err(invalid("unsupported tagged literal layout")),
    }
    output.push(']');
    Ok(())
}

fn list(
    array: &dyn Array,
    field: &Field,
    range: std::ops::Range<usize>,
    output: &mut String,
) -> Result<()> {
    output.push('[');
    for (ordinal, row) in range.enumerate() {
        if ordinal != 0 {
            output.push(',');
        }
        value(array, field, row, output)?;
    }
    output.push(']');
    Ok(())
}

fn value_tag(array: &dyn Array, field: &Field) -> Result<&'static str> {
    Ok(match array.data_type() {
        DataType::Boolean => "bool",
        DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::Timestamp(..) => "i64",
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => "u64",
        DataType::Float64 => "f64",
        DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View
            if field.metadata().contains_key(pse_schema::arrow::KEY_ENUM) =>
        {
            "enum"
        }
        DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View => "text",
        DataType::FixedSizeBinary(16) => "id",
        DataType::FixedSizeBinary(32) => "hash",
        DataType::Dictionary(..)
            if field
                .metadata()
                .get(pse_schema::arrow::KEY_LOGICAL_TYPE)
                .is_some_and(|value| value == "text") =>
        {
            "text"
        }
        DataType::Dictionary(..) => "enum",
        DataType::List(_) | DataType::LargeList(_) | DataType::FixedSizeList(..) => "list",
        DataType::Struct(_) => "struct",
        _ => return Err(invalid("value has no tagged literal representation")),
    })
}

fn dictionary(array: &dyn Array, key: &DataType, row: usize, output: &mut String) -> Result<()> {
    let (values, index) = if key == &DataType::Int8 {
        let array = downcast::<DictionaryArray<Int8Type>>(array)?;
        (
            array.values(),
            usize::try_from(array.keys().value(row))
                .map_err(|_| invalid("negative dictionary code"))?,
        )
    } else {
        let array = downcast::<DictionaryArray<Int32Type>>(array)?;
        (
            array.values(),
            usize::try_from(array.keys().value(row))
                .map_err(|_| invalid("negative dictionary code"))?,
        )
    };
    let values = downcast::<StringArray>(values.as_ref())?;
    if index >= values.len() || values.is_null(index) {
        return Err(invalid("dictionary does not resolve to a visible member"));
    }
    quoted(values.value(index), output)?;
    Ok(())
}
