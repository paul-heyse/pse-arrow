// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Field-directed construction at the scalar JSON interchange boundary.
use super::{Result, invalid};
use arrow_array::{
    Array, ArrayRef, BinaryArray, BooleanArray, Date32Array, Date64Array, Decimal32Array,
    Decimal64Array, Decimal128Array, Decimal256Array, DictionaryArray, DurationMicrosecondArray,
    DurationMillisecondArray, DurationNanosecondArray, DurationSecondArray, FixedSizeBinaryArray,
    FixedSizeListArray, Float16Array, Float32Array, Float64Array, Int8Array, Int16Array,
    Int32Array, Int64Array, IntervalDayTimeArray, IntervalMonthDayNanoArray,
    IntervalYearMonthArray, LargeListArray, LargeListViewArray, ListArray, ListViewArray, MapArray,
    PrimitiveArray, RunArray, StringArray, StructArray, Time32MillisecondArray, Time32SecondArray,
    Time64MicrosecondArray, Time64NanosecondArray, UInt8Array, UInt16Array, UInt32Array,
    UInt64Array, UnionArray, make_array, new_empty_array, new_null_array,
    types::{
        ArrowPrimitiveType, Float16Type, Int8Type, Int16Type, Int32Type, Int64Type,
        IntervalDayTime, IntervalMonthDayNano, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
    },
};
use arrow_schema::{DataType, Field, IntervalUnit, TimeUnit, UnionMode};
use serde_json::Value;
use std::sync::Arc;

#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive native Arrow literal dispatch"
)]
pub(super) fn value(field: &Field, tagged: &Value) -> Result<ArrayRef> {
    let parts = tagged
        .as_array()
        .filter(|parts| parts.len() == 2)
        .ok_or_else(|| invalid("literal requires exactly a tag and value"))?;
    let tag = parts[0]
        .as_str()
        .ok_or_else(|| invalid("literal tag must be text"))?;
    let value = &parts[1];
    if tag == "null" {
        if !value.is_null() {
            return Err(invalid("null tag requires null payload"));
        }
        return Ok(new_null_array(field.data_type(), 1));
    }
    let expected = pse_columnar::native_value::value_tag(field.data_type(), field)
        .map_err(|error| invalid(error.to_string()))?;
    if tag != expected {
        return Err(invalid(format!("expected {expected} literal, found {tag}")));
    }
    let text = || {
        value
            .as_str()
            .ok_or_else(|| invalid("literal payload must be text"))
    };
    macro_rules! signed {
        ($ty:ty, $array:ty) => {{
            let source = value
                .as_i64()
                .ok_or_else(|| invalid("exact signed integer required"))?;
            let native = <$ty>::try_from(source).map_err(|error| invalid(error.to_string()))?;
            {
                let array: ArrayRef = Arc::new(<$array>::from(vec![native]));
                array
            }
        }};
    }
    macro_rules! unsigned {
        ($ty:ty, $array:ty) => {{
            let source = value
                .as_u64()
                .ok_or_else(|| invalid("exact unsigned integer required"))?;
            let native = <$ty>::try_from(source).map_err(|error| invalid(error.to_string()))?;
            {
                let array: ArrayRef = Arc::new(<$array>::from(vec![native]));
                array
            }
        }};
    }
    macro_rules! temporal {
        ($array:ty, $ty:ty) => {{
            let source = value
                .as_i64()
                .ok_or_else(|| invalid("exact temporal integer required"))?;
            let native = <$ty>::try_from(source)
                .map_err(|_| invalid("temporal integer exceeds its native range"))?;
            {
                let array: ArrayRef = Arc::new(
                    <$array>::from(vec![native]).with_data_type(field.data_type().clone()),
                );
                array
            }
        }};
    }
    macro_rules! decimal {
        ($array:ty, $native:ty) => {{
            let native: $native = text()?
                .parse()
                .map_err(|error| invalid(format!("invalid decimal: {error}")))?;
            {
                let array: ArrayRef = Arc::new(
                    <$array>::from(vec![native]).with_data_type(field.data_type().clone()),
                );
                array
            }
        }};
    }
    let array: ArrayRef = match field.data_type() {
        DataType::Boolean => Arc::new(BooleanArray::from(vec![
            value.as_bool().ok_or_else(|| invalid("Boolean required"))?,
        ])),
        DataType::Int8 => signed!(i8, Int8Array),
        DataType::Int16 => signed!(i16, Int16Array),
        DataType::Int32 => signed!(i32, Int32Array),
        DataType::Int64 => Arc::new(Int64Array::from(vec![
            value
                .as_i64()
                .ok_or_else(|| invalid("exact signed integer required"))?,
        ])),
        DataType::UInt8 => unsigned!(u8, UInt8Array),
        DataType::UInt16 => unsigned!(u16, UInt16Array),
        DataType::UInt32 => unsigned!(u32, UInt32Array),
        DataType::UInt64 => Arc::new(UInt64Array::from(vec![
            value
                .as_u64()
                .ok_or_else(|| invalid("exact unsigned integer required"))?,
        ])),
        DataType::Float16 => Arc::new(Float16Array::from(vec![
            <Float16Type as ArrowPrimitiveType>::Native::from_bits(
                u16::try_from(bits(text()?, 4)?).map_err(|error| invalid(error.to_string()))?,
            ),
        ])),
        DataType::Float32 => Arc::new(Float32Array::from(vec![f32::from_bits(
            u32::try_from(bits(text()?, 8)?).map_err(|error| invalid(error.to_string()))?,
        )])),
        DataType::Float64 => Arc::new(Float64Array::from(vec![f64::from_bits(bits(text()?, 16)?)])),
        DataType::Decimal32(..) => decimal!(Decimal32Array, i32),
        DataType::Decimal64(..) => decimal!(Decimal64Array, i64),
        DataType::Decimal128(..) => decimal!(Decimal128Array, i128),
        DataType::Decimal256(..) => decimal!(Decimal256Array, arrow_buffer::i256),
        DataType::Date32 => temporal!(Date32Array, i32),
        DataType::Date64 => temporal!(Date64Array, i64),
        DataType::Time32(TimeUnit::Second) => temporal!(Time32SecondArray, i32),
        DataType::Time32(TimeUnit::Millisecond) => temporal!(Time32MillisecondArray, i32),
        DataType::Time64(TimeUnit::Microsecond) => temporal!(Time64MicrosecondArray, i64),
        DataType::Time64(TimeUnit::Nanosecond) => temporal!(Time64NanosecondArray, i64),
        DataType::Duration(TimeUnit::Second) => temporal!(DurationSecondArray, i64),
        DataType::Duration(TimeUnit::Millisecond) => temporal!(DurationMillisecondArray, i64),
        DataType::Duration(TimeUnit::Microsecond) => temporal!(DurationMicrosecondArray, i64),
        DataType::Duration(TimeUnit::Nanosecond) => temporal!(DurationNanosecondArray, i64),
        DataType::Interval(IntervalUnit::YearMonth) => temporal!(IntervalYearMonthArray, i32),
        DataType::Interval(IntervalUnit::DayTime) => {
            let parts = integers(value, 2)?;
            Arc::new(IntervalDayTimeArray::from(vec![IntervalDayTime::new(
                i32::try_from(parts[0]).map_err(|error| invalid(error.to_string()))?,
                i32::try_from(parts[1]).map_err(|error| invalid(error.to_string()))?,
            )]))
        }
        DataType::Interval(IntervalUnit::MonthDayNano) => {
            let parts = integers(value, 3)?;
            Arc::new(IntervalMonthDayNanoArray::from(vec![
                IntervalMonthDayNano::new(
                    i32::try_from(parts[0]).map_err(|error| invalid(error.to_string()))?,
                    i32::try_from(parts[1]).map_err(|error| invalid(error.to_string()))?,
                    parts[2],
                ),
            ]))
        }
        DataType::Utf8 => Arc::new(StringArray::from(vec![text()?])),
        DataType::LargeUtf8 | DataType::Utf8View => {
            let source = StringArray::from(vec![text()?]);
            arrow_cast::cast_with_options(
                &source,
                field.data_type(),
                &arrow_cast::CastOptions {
                    safe: false,
                    ..Default::default()
                },
            )
            .map_err(|error| invalid(error.to_string()))?
        }
        DataType::Dictionary(key, kind) => {
            let child = self::value(&field.clone().with_data_type(kind.as_ref().clone()), tagged)?;
            macro_rules! dictionary {
                ($ty:ty) => {{
                    {
                        let array: ArrayRef = Arc::new(
                            DictionaryArray::<$ty>::try_new(
                                PrimitiveArray::<$ty>::from(vec![0]),
                                child,
                            )
                            .map_err(|error| invalid(error.to_string()))?,
                        );
                        array
                    }
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
                _ => return Err(invalid("invalid dictionary key")),
            }
        }
        DataType::RunEndEncoded(runs, child) => {
            let array = self::value(child, tagged)?;
            macro_rules! run {
                ($ty:ty) => {{
                    let run = RunArray::<$ty>::try_new(
                        &PrimitiveArray::<$ty>::from(vec![1]),
                        array.as_ref(),
                    )
                    .map_err(|error| invalid(error.to_string()))?;
                    // Rebind the exact caller fields; RunArray constructors use default names.
                    make_array(
                        run.to_data()
                            .into_builder()
                            .data_type(field.data_type().clone())
                            .build()
                            .map_err(|error| invalid(error.to_string()))?,
                    )
                }};
            }
            match runs.data_type() {
                DataType::Int16 => run!(Int16Type),
                DataType::Int32 => run!(Int32Type),
                DataType::Int64 => run!(Int64Type),
                _ => return Err(invalid("invalid run-end type")),
            }
        }
        DataType::Binary | DataType::LargeBinary | DataType::BinaryView => {
            let bytes = hex(text()?)?;
            let array = BinaryArray::from(vec![bytes.as_slice()]);
            arrow_cast::cast(&array, field.data_type())
                .map_err(|error| invalid(error.to_string()))?
        }
        DataType::Timestamp(..) => {
            let source = Int64Array::from(vec![
                value
                    .as_i64()
                    .ok_or_else(|| invalid("timestamp integer required"))?,
            ]);
            arrow_cast::cast_with_options(
                &source,
                field.data_type(),
                &arrow_cast::CastOptions {
                    safe: false,
                    ..Default::default()
                },
            )
            .map_err(|error| invalid(error.to_string()))?
        }
        DataType::FixedSizeBinary(width) => {
            let bytes = hex(text()?)?;
            if i32::try_from(bytes.len()).ok() != Some(*width) {
                return Err(invalid("binary literal width differs from field"));
            }
            Arc::new(
                FixedSizeBinaryArray::try_from_iter([bytes.as_slice()].into_iter())
                    .map_err(|error| invalid(error.to_string()))?,
            )
        }
        DataType::Struct(fields) => {
            let values = value
                .as_array()
                .ok_or_else(|| invalid("struct payload must be an array"))?;
            if values.len() != fields.len() {
                return Err(invalid("struct literal arity differs from its field"));
            }
            let children = fields
                .iter()
                .zip(values)
                .map(|(field, item)| self::value(field, item))
                .collect::<Result<Vec<_>>>()?;
            Arc::new(
                StructArray::try_new(fields.clone(), children, None)
                    .map_err(|error| invalid(error.to_string()))?,
            )
        }
        DataType::List(child)
        | DataType::LargeList(child)
        | DataType::FixedSizeList(child, _)
        | DataType::ListView(child)
        | DataType::LargeListView(child)
        | DataType::Map(child, _) => {
            let values = value
                .as_array()
                .ok_or_else(|| invalid("list payload must be an array"))?;
            let items = values
                .iter()
                .map(|item| self::value(child, item))
                .collect::<Result<Vec<_>>>()?;
            let array = if items.is_empty() {
                new_empty_array(child.data_type())
            } else {
                arrow_select::concat::concat(&items.iter().map(AsRef::as_ref).collect::<Vec<_>>())
                    .map_err(|error| invalid(error.to_string()))?
            };
            let length = i32::try_from(values.len()).map_err(|error| invalid(error.to_string()))?;
            match field.data_type() {
                DataType::List(_) => Arc::new(
                    ListArray::try_new(
                        Arc::clone(child),
                        arrow_buffer::OffsetBuffer::new(vec![0, length].into()),
                        array,
                        None,
                    )
                    .map_err(|error| invalid(error.to_string()))?,
                ),
                DataType::LargeList(_) => Arc::new(
                    LargeListArray::try_new(
                        Arc::clone(child),
                        arrow_buffer::OffsetBuffer::new(vec![0, i64::from(length)].into()),
                        array,
                        None,
                    )
                    .map_err(|error| invalid(error.to_string()))?,
                ),
                DataType::ListView(_) => Arc::new(
                    ListViewArray::try_new(
                        Arc::clone(child),
                        vec![0_i32].into(),
                        vec![length].into(),
                        array,
                        None,
                    )
                    .map_err(|error| invalid(error.to_string()))?,
                ),
                DataType::LargeListView(_) => Arc::new(
                    LargeListViewArray::try_new(
                        Arc::clone(child),
                        vec![0_i64].into(),
                        vec![i64::from(length)].into(),
                        array,
                        None,
                    )
                    .map_err(|error| invalid(error.to_string()))?,
                ),
                DataType::Map(_, sorted) => {
                    let entries = array
                        .as_any()
                        .downcast_ref::<StructArray>()
                        .ok_or_else(|| invalid("map entries must be struct"))?
                        .clone();
                    Arc::new(
                        MapArray::try_new(
                            Arc::clone(child),
                            arrow_buffer::OffsetBuffer::new(vec![0, length].into()),
                            entries,
                            None,
                            *sorted,
                        )
                        .map_err(|error| invalid(error.to_string()))?,
                    )
                }
                DataType::FixedSizeList(_, width) => {
                    if length != *width {
                        return Err(invalid("fixed-size list literal has wrong length"));
                    }
                    Arc::new(
                        FixedSizeListArray::try_new(Arc::clone(child), *width, array, None)
                            .map_err(|error| invalid(error.to_string()))?,
                    )
                }
                _ => return Err(invalid("unsupported list layout")),
            }
        }
        DataType::Union(fields, mode) => {
            let payload = value
                .as_array()
                .filter(|parts| parts.len() == 2)
                .ok_or_else(|| invalid("union payload requires type ID and value"))?;
            let id = i8::try_from(
                payload[0]
                    .as_i64()
                    .ok_or_else(|| invalid("union type ID must be an integer"))?,
            )
            .map_err(|error| invalid(error.to_string()))?;
            if !fields.iter().any(|(candidate, _)| candidate == id) {
                return Err(invalid("union type ID not declared"));
            }
            let children = fields
                .iter()
                .map(|(candidate, child)| {
                    if candidate == id {
                        self::value(child, &payload[1])
                    } else if *mode == UnionMode::Sparse {
                        Ok(new_null_array(child.data_type(), 1))
                    } else {
                        Ok(new_empty_array(child.data_type()))
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            let offsets = (*mode == UnionMode::Dense).then(|| vec![0_i32].into());
            Arc::new(
                UnionArray::try_new(fields.clone(), vec![id].into(), offsets, children)
                    .map_err(|error| invalid(error.to_string()))?,
            )
        }
        _ => return Err(invalid("unsupported native literal datatype")),
    };
    Ok(array)
}
fn hex(text: &str) -> Result<Vec<u8>> {
    if !text.len().is_multiple_of(2)
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid(
            "binary payload requires lowercase hexadecimal byte pairs",
        ));
    }
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let value = std::str::from_utf8(pair).map_err(|error| invalid(error.to_string()))?;
            u8::from_str_radix(value, 16).map_err(|error| invalid(error.to_string()))
        })
        .collect()
}

fn bits(text: &str, length: usize) -> Result<u64> {
    if text.len() != length
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid(
            "floating-point literal requires exact lowercase hexadecimal bits",
        ));
    }
    u64::from_str_radix(text, 16).map_err(|error| invalid(error.to_string()))
}
fn integers(value: &Value, length: usize) -> Result<Vec<i64>> {
    let parts = value
        .as_array()
        .filter(|parts| parts.len() == length)
        .ok_or_else(|| invalid("interval literal shape differs"))?;
    parts
        .iter()
        .map(|part| {
            part.as_i64()
                .ok_or_else(|| invalid("interval components must be exact integers"))
        })
        .collect()
}
