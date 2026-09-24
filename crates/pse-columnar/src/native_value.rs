// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The tagged literal wire codec over actual Arrow storage, including null parents.

/// Encode one actual value with bit-exact floating-point payloads.
/// # Errors
/// Wrong physical type, invalid storage, or an out-of-range row.
pub fn to_json(array: &dyn Array, field: &Field, row: usize) -> Result<String> {
    if array.data_type() != field.data_type() {
        return Err(invalid("literal field and storage differ"));
    }
    let mut text = String::new();
    value(
        array,
        field,
        row,
        &mut Output {
            sink: Sink::Text(&mut text),
            semantic: false,
        },
    )?;
    Ok(text)
}
/// Digest a field-directed value stream with canonical NaNs and distinct signed zeros.
/// This is a payload digest; callers frame the field's projected metadata separately.
/// # Errors
/// Wrong physical type, invalid storage, or an out-of-range row.
pub fn semantic_payload(
    array: &dyn Array,
    field: &Field,
    row: usize,
) -> Result<crate::ContentHash> {
    if array.data_type() != field.data_type() {
        return Err(invalid("value field and storage differ"));
    }
    let mut hash = pse_ids::preimage::PreimageHasher::new_derive_key("pse:native-value-payload:v1");
    value(
        array,
        field,
        row,
        &mut Output {
            sink: Sink::Hash(&mut hash),
            semantic: true,
        },
    )?;
    Ok(crate::ContentHash::from_bytes(*hash.finalize().as_bytes()))
}
enum Sink<'a> {
    Text(&'a mut String),
    Hash(&'a mut pse_ids::preimage::PreimageHasher),
}
struct Output<'a> {
    sink: Sink<'a>,
    semantic: bool,
}
impl Output<'_> {
    fn push_str(&mut self, text: &str) {
        match &mut self.sink {
            Sink::Text(out) => out.push_str(text),
            Sink::Hash(out) => {
                out.update(text.as_bytes());
            }
        }
    }
    fn push(&mut self, character: char) {
        let mut bytes = [0; 4];
        self.push_str(character.encode_utf8(&mut bytes));
    }
}
impl Write for Output<'_> {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.push_str(text);
        Ok(())
    }
}

type Result<T> = std::result::Result<T, crate::CanonError>;
use arrow_array::{
    Array, BinaryArray, BinaryViewArray, BooleanArray, Date32Array, Date64Array, Decimal32Array,
    Decimal64Array, Decimal128Array, Decimal256Array, DictionaryArray, DurationMicrosecondArray,
    DurationMillisecondArray, DurationNanosecondArray, DurationSecondArray, FixedSizeBinaryArray,
    FixedSizeListArray, Float16Array, Float32Array, Float64Array, Int8Array, Int16Array,
    Int32Array, Int64Array, IntervalDayTimeArray, IntervalMonthDayNanoArray,
    IntervalYearMonthArray, LargeBinaryArray, LargeListArray, LargeListViewArray, LargeStringArray,
    ListArray, ListViewArray, MapArray, RunArray, StringArray, StringViewArray, StructArray,
    Time32MillisecondArray, Time32SecondArray, Time64MicrosecondArray, Time64NanosecondArray,
    TimestampMicrosecondArray, TimestampMillisecondArray, TimestampNanosecondArray,
    TimestampSecondArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array, UnionArray,
    types::{
        Decimal32Type, Decimal64Type, Decimal128Type, Decimal256Type, Int8Type, Int16Type,
        Int32Type, Int64Type, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
        validate_decimal_precision_and_scale,
    },
};
use arrow_schema::{DataType, Field, IntervalUnit, TimeUnit};
use std::fmt::Write;

fn invalid(reason: impl Into<String>) -> crate::CanonError {
    crate::CanonError::UnsupportedLayout {
        path: crate::FieldPath::root(),
        what: reason.into(),
    }
}

/// Admit a native representation recursively, without performing value validation.
/// # Errors
/// A native child has no representation.
pub fn admit_type(kind: &DataType) -> Result<()> {
    // Admission follows native Arrow layout. The codec never admits a second type universe.
    match kind {
        DataType::Time32(TimeUnit::Microsecond | TimeUnit::Nanosecond)
        | DataType::Time64(TimeUnit::Second | TimeUnit::Millisecond) => {
            Err(invalid("time unit does not fit the native Arrow width"))
        }
        DataType::FixedSizeBinary(width) | DataType::FixedSizeList(_, width) if *width < 0 => {
            Err(invalid("negative native fixed width"))
        }
        DataType::Decimal32(precision, scale) => {
            validate_decimal_precision_and_scale::<Decimal32Type>(*precision, *scale)
                .map_err(|error| invalid(error.to_string()))
        }
        DataType::Decimal64(precision, scale) => {
            validate_decimal_precision_and_scale::<Decimal64Type>(*precision, *scale)
                .map_err(|error| invalid(error.to_string()))
        }
        DataType::Decimal128(precision, scale) => {
            validate_decimal_precision_and_scale::<Decimal128Type>(*precision, *scale)
                .map_err(|error| invalid(error.to_string()))
        }
        DataType::Decimal256(precision, scale) => {
            validate_decimal_precision_and_scale::<Decimal256Type>(*precision, *scale)
                .map_err(|error| invalid(error.to_string()))
        }
        DataType::Dictionary(key, _) if !key.is_dictionary_key_type() => {
            Err(invalid("invalid native dictionary key type"))
        }
        DataType::RunEndEncoded(runs, _)
            if runs.is_nullable()
                || !matches!(
                    runs.data_type(),
                    DataType::Int16 | DataType::Int32 | DataType::Int64
                ) =>
        {
            Err(invalid("invalid native run-end field"))
        }
        DataType::Map(entries, _)
            if entries.is_nullable()
                || !matches!(entries.data_type(), DataType::Struct(fields) if fields.len() == 2 && !fields[0].is_nullable()) =>
        {
            Err(invalid("invalid native map entries"))
        }
        DataType::Union(children, _) if children.is_empty() => {
            Err(invalid("literal union needs an active child"))
        }
        DataType::List(child)
        | DataType::LargeList(child)
        | DataType::ListView(child)
        | DataType::LargeListView(child)
        | DataType::FixedSizeList(child, _)
        | DataType::Map(child, _)
        | DataType::RunEndEncoded(_, child) => admit_type(child.data_type()),
        DataType::Struct(children) => children
            .iter()
            .try_for_each(|child| admit_type(child.data_type())),
        DataType::Union(children, _) => children
            .iter()
            .try_for_each(|(_, child)| admit_type(child.data_type())),
        DataType::Dictionary(_, value) => admit_type(value),
        _ => Ok(()),
    }
}

fn downcast<T: Array + 'static>(array: &dyn Array) -> Result<&T> {
    array
        .as_any()
        .downcast_ref::<T>()
        .ok_or_else(|| invalid("actual Arrow array differs from its datatype"))
}

fn quoted(value: &str, output: &mut Output<'_>) -> Result<()> {
    output.push_str(&serde_json::to_string(value).map_err(|error| invalid(error.to_string()))?);
    Ok(())
}

fn number(value: impl std::fmt::Display, output: &mut Output<'_>) -> Result<()> {
    write!(output, "{value}").map_err(|error| invalid(error.to_string()))
}

#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive native Arrow value dispatch"
)]
fn value(array: &dyn Array, field: &Field, row: usize, output: &mut Output<'_>) -> Result<()> {
    if row >= array.len() {
        return Err(invalid("encoded row lies outside the actual array"));
    }
    if array.is_null(row) {
        output.push_str("[\"null\",null]");
        return Ok(());
    }
    if let DataType::Dictionary(key, kind) = field.data_type() {
        macro_rules! dictionary {
            ($ty:ty) => {{
                let array = downcast::<DictionaryArray<$ty>>(array)?;
                let index = array
                    .key(row)
                    .ok_or_else(|| invalid("visible dictionary key absent"))?;
                return value(
                    array.values().as_ref(),
                    &field.clone().with_data_type(kind.as_ref().clone()),
                    index,
                    output,
                );
            }};
        }
        match key.as_ref() {
            DataType::Int8 => dictionary!(Int8Type),
            DataType::Int16 => dictionary!(Int16Type),
            DataType::Int32 => dictionary!(Int32Type),
            DataType::Int64 => dictionary!(Int64Type),
            DataType::UInt8 => dictionary!(UInt8Type),
            DataType::UInt16 => dictionary!(UInt16Type),
            DataType::UInt32 => dictionary!(UInt32Type),
            DataType::UInt64 => dictionary!(UInt64Type),
            _ => return Err(invalid("invalid dictionary key type")),
        }
    }
    if let DataType::RunEndEncoded(runs, child) = field.data_type() {
        macro_rules! run {
            ($ty:ty) => {{
                let run = downcast::<RunArray<$ty>>(array)?;
                return value(
                    run.values().as_ref(),
                    child,
                    run.get_physical_index(row),
                    output,
                );
            }};
        }
        match runs.data_type() {
            DataType::Int16 => run!(Int16Type),
            DataType::Int32 => run!(Int32Type),
            DataType::Int64 => run!(Int64Type),
            _ => return Err(invalid("invalid run-end type")),
        }
    }
    let tag = value_tag(array.data_type(), field)?;
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
        DataType::Timestamp(unit, _) => match unit {
            TimeUnit::Second => {
                number(downcast::<TimestampSecondArray>(array)?.value(row), output)?;
            }
            TimeUnit::Millisecond => number(
                downcast::<TimestampMillisecondArray>(array)?.value(row),
                output,
            )?,
            TimeUnit::Microsecond => number(
                downcast::<TimestampMicrosecondArray>(array)?.value(row),
                output,
            )?,
            TimeUnit::Nanosecond => number(
                downcast::<TimestampNanosecondArray>(array)?.value(row),
                output,
            )?,
        },
        DataType::Date32 => number(downcast::<Date32Array>(array)?.value(row), output)?,
        DataType::Date64 => number(downcast::<Date64Array>(array)?.value(row), output)?,
        DataType::Time32(TimeUnit::Second) => {
            number(downcast::<Time32SecondArray>(array)?.value(row), output)?;
        }
        DataType::Time32(TimeUnit::Millisecond) => number(
            downcast::<Time32MillisecondArray>(array)?.value(row),
            output,
        )?,
        DataType::Time64(TimeUnit::Microsecond) => number(
            downcast::<Time64MicrosecondArray>(array)?.value(row),
            output,
        )?,
        DataType::Time64(TimeUnit::Nanosecond) => {
            number(downcast::<Time64NanosecondArray>(array)?.value(row), output)?;
        }
        DataType::Duration(unit) => match unit {
            TimeUnit::Second => number(downcast::<DurationSecondArray>(array)?.value(row), output)?,
            TimeUnit::Millisecond => number(
                downcast::<DurationMillisecondArray>(array)?.value(row),
                output,
            )?,
            TimeUnit::Microsecond => number(
                downcast::<DurationMicrosecondArray>(array)?.value(row),
                output,
            )?,
            TimeUnit::Nanosecond => number(
                downcast::<DurationNanosecondArray>(array)?.value(row),
                output,
            )?,
        },
        DataType::Decimal32(..) => quoted(
            &downcast::<Decimal32Array>(array)?.value(row).to_string(),
            output,
        )?,
        DataType::Decimal64(..) => quoted(
            &downcast::<Decimal64Array>(array)?.value(row).to_string(),
            output,
        )?,
        DataType::Decimal128(..) => quoted(
            &downcast::<Decimal128Array>(array)?.value(row).to_string(),
            output,
        )?,
        DataType::Decimal256(..) => quoted(
            &downcast::<Decimal256Array>(array)?.value(row).to_string(),
            output,
        )?,
        DataType::Interval(IntervalUnit::YearMonth) => number(
            downcast::<IntervalYearMonthArray>(array)?.value(row),
            output,
        )?,
        DataType::Interval(IntervalUnit::DayTime) => {
            let item = downcast::<IntervalDayTimeArray>(array)?.value(row);
            number(
                format_args!("[{},{}]", item.days, item.milliseconds),
                output,
            )?;
        }
        DataType::Interval(IntervalUnit::MonthDayNano) => {
            let item = downcast::<IntervalMonthDayNanoArray>(array)?.value(row);
            number(
                format_args!("[{},{},{}]", item.months, item.days, item.nanoseconds),
                output,
            )?;
        }
        DataType::Float16 => quoted(
            &format!(
                "{:04x}",
                if output.semantic && downcast::<Float16Array>(array)?.value(row).is_nan() {
                    0x7e00
                } else {
                    downcast::<Float16Array>(array)?.value(row).to_bits()
                }
            ),
            output,
        )?,
        DataType::Float32 => quoted(
            &format!(
                "{:08x}",
                if output.semantic {
                    crate::canonical_f32_bits(downcast::<Float32Array>(array)?.value(row))
                } else {
                    downcast::<Float32Array>(array)?.value(row).to_bits()
                }
            ),
            output,
        )?,
        DataType::Float64 => {
            write!(
                output,
                "\"{:016x}\"",
                if output.semantic {
                    crate::canonical_f64_bits(downcast::<Float64Array>(array)?.value(row))
                } else {
                    downcast::<Float64Array>(array)?.value(row).to_bits()
                }
            )
            .map_err(|error| invalid(error.to_string()))?;
        }
        DataType::Utf8 => quoted(downcast::<StringArray>(array)?.value(row), output)?,
        DataType::LargeUtf8 => quoted(downcast::<LargeStringArray>(array)?.value(row), output)?,
        DataType::Utf8View => quoted(downcast::<StringViewArray>(array)?.value(row), output)?,
        DataType::Binary => bytes(downcast::<BinaryArray>(array)?.value(row), output)?,
        DataType::LargeBinary => bytes(downcast::<LargeBinaryArray>(array)?.value(row), output)?,
        DataType::BinaryView => bytes(downcast::<BinaryViewArray>(array)?.value(row), output)?,
        DataType::FixedSizeBinary(_) => {
            bytes(downcast::<FixedSizeBinaryArray>(array)?.value(row), output)?;
        }
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
        DataType::ListView(child) => {
            let values = downcast::<ListViewArray>(array)?.value(row);
            list(values.as_ref(), child, 0..values.len(), output)?;
        }
        DataType::LargeListView(child) => {
            let values = downcast::<LargeListViewArray>(array)?.value(row);
            list(values.as_ref(), child, 0..values.len(), output)?;
        }
        DataType::Map(entries, _) => {
            let values = downcast::<MapArray>(array)?.value(row);
            list(&values, entries, 0..values.len(), output)?;
        }
        DataType::Union(fields, _) => {
            let values = downcast::<UnionArray>(array)?;
            let id = values.type_id(row);
            let child = fields
                .iter()
                .find_map(|(candidate, field)| (candidate == id).then_some(field))
                .ok_or_else(|| invalid("union field absent"))?;
            number(format_args!("[{id},"), output)?;
            value(
                values.child(id).as_ref(),
                child,
                values.value_offset(row),
                output,
            )?;
            output.push(']');
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
    output: &mut Output<'_>,
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

/// The field-directed literal tag; physical widths remain declared by the field.
/// # Errors
/// A native representation has no tag.
pub fn value_tag(kind: &DataType, field: &Field) -> Result<&'static str> {
    Ok(match kind {
        DataType::Boolean => "bool",
        DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::Timestamp(..)
        | DataType::Date32
        | DataType::Date64
        | DataType::Time32(_)
        | DataType::Time64(_)
        | DataType::Duration(_)
        | DataType::Interval(IntervalUnit::YearMonth) => "i64",
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => "u64",
        DataType::Float16 => "f16",
        DataType::Float32 => "f32",
        DataType::Float64 => "f64",
        DataType::Decimal32(..)
        | DataType::Decimal64(..)
        | DataType::Decimal128(..)
        | DataType::Decimal256(..) => "decimal",
        DataType::Interval(_) => "interval",
        DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View
            if field.metadata().contains_key("pse.semantic.enum") =>
        {
            "enum"
        }
        DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View => "text",
        DataType::FixedSizeBinary(16) => "id",
        DataType::FixedSizeBinary(32) => "hash",
        DataType::FixedSizeBinary(_)
        | DataType::Binary
        | DataType::LargeBinary
        | DataType::BinaryView => "bytes",
        DataType::Dictionary(_, child) => return value_tag(child, field),
        DataType::RunEndEncoded(_, child) => return value_tag(child.data_type(), child),
        DataType::List(_)
        | DataType::LargeList(_)
        | DataType::FixedSizeList(..)
        | DataType::ListView(_)
        | DataType::LargeListView(_) => "list",
        DataType::Struct(_) => "struct",
        DataType::Map(..) => "map",
        DataType::Union(..) => "union",
        DataType::Null => "null",
    })
}
fn bytes(values: &[u8], output: &mut Output<'_>) -> Result<()> {
    output.push('"');
    for byte in values {
        write!(output, "{byte:02x}").map_err(|error| invalid(error.to_string()))?;
    }
    output.push('"');
    Ok(())
}

#[cfg(test)]
mod consolidation_unit;
