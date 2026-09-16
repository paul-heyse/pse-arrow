// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Recursive hashing-copy normalization. A missing selection is a forced-valid zero
//! placeholder below an absent parent, distinct from a visible null in the source.

use crate::{CanonError, CanonicalContract, DictKey, Layout};
use arrow_array::builder::{BinaryBuilder, FixedSizeBinaryBuilder, StringBuilder};
use arrow_array::types::{Int8Type, Int32Type};
use arrow_array::{
    Array, ArrayRef, BinaryArray, BooleanArray, DictionaryArray, FixedSizeBinaryArray,
    FixedSizeListArray, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array, Int64Array,
    ListArray, RecordBatch, StringArray, StructArray, TimestampNanosecondArray, UInt8Array,
    UInt16Array, UInt32Array, UInt64Array,
};
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow_schema::{DataType, Field, Schema};
use std::sync::Arc;

pub(super) fn cast<T: 'static>(array: &dyn Array) -> Result<&T, CanonError> {
    array.as_any().downcast_ref().ok_or_else(|| {
        CanonError::Internal("array implementation differs from admitted storage".to_owned())
    })
}

pub(super) fn field(source: &Field, layout: &Layout) -> Field {
    let data_type = match (source.data_type(), layout) {
        (_, Layout::Enum { .. }) => DataType::Utf8,
        (DataType::List(child), Layout::List(inner)) => {
            DataType::List(Arc::new(field(child, inner)))
        }
        (DataType::FixedSizeList(child, width), Layout::FixedSizeList(inner, _)) => {
            DataType::FixedSizeList(Arc::new(field(child, inner)), *width)
        }
        (DataType::Struct(children), Layout::Struct(layouts)) => DataType::Struct(
            children
                .iter()
                .zip(layouts)
                .map(|(child, (_, layout))| field(child, layout))
                .collect(),
        ),
        (other, _) => other.clone(),
    };
    Field::new(source.name(), data_type, source.is_nullable())
}

pub(super) fn batch(
    contract: &CanonicalContract,
    source: &RecordBatch,
) -> Result<RecordBatch, CanonError> {
    let selection = (0..source.num_rows()).map(Some).collect::<Vec<_>>();
    let fields = contract
        .schema
        .fields()
        .iter()
        .zip(&contract.layouts)
        .map(|(source, layout)| field(source, layout))
        .collect::<Vec<_>>();
    let columns = source
        .columns()
        .iter()
        .zip(contract.schema.fields())
        .zip(&contract.layouts)
        .map(|((array, field), layout)| array_values(field, layout, array.as_ref(), &selection))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RecordBatch::try_new_with_options(
        Arc::new(Schema::new(fields)),
        columns,
        &arrow_array::RecordBatchOptions::new().with_row_count(Some(source.num_rows())),
    )?)
}

fn validity(array: &dyn Array, selection: &[Option<usize>]) -> Option<NullBuffer> {
    let valid = selection
        .iter()
        .map(|index| index.is_none_or(|index| array.is_valid(index)))
        .collect::<Vec<_>>();
    if valid.iter().all(|value| *value) {
        None
    } else {
        Some(NullBuffer::from(valid))
    }
}
fn selected<T: Copy>(
    array: &dyn Array,
    selection: &[Option<usize>],
    zero: T,
    value: impl Fn(usize) -> T,
) -> Vec<Option<T>> {
    selection
        .iter()
        .map(|index| match index {
            None => Some(zero),
            Some(index) if array.is_null(*index) => None,
            Some(index) => Some(value(*index)),
        })
        .collect()
}

fn array_values(
    source: &Field,
    layout: &Layout,
    array: &dyn Array,
    selection: &[Option<usize>],
) -> Result<ArrayRef, CanonError> {
    macro_rules! primitive {
        ($array:ty,$zero:expr,$map:expr) => {{
            let values = cast::<$array>(array)?;
            {
                let result: ArrayRef =
                    Arc::new(<$array>::from(selected(array, selection, $zero, |index| {
                        ($map)(values.value(index))
                    })));
                result
            }
        }};
    }
    Ok(match layout {
        Layout::Boolean => primitive!(BooleanArray, false, core::convert::identity),
        Layout::Int8 => primitive!(Int8Array, 0, core::convert::identity),
        Layout::Int16 => primitive!(Int16Array, 0, core::convert::identity),
        Layout::Int32 => primitive!(Int32Array, 0, core::convert::identity),
        Layout::Int64 => primitive!(Int64Array, 0, core::convert::identity),
        Layout::UInt8 => primitive!(UInt8Array, 0, core::convert::identity),
        Layout::UInt16 => primitive!(UInt16Array, 0, core::convert::identity),
        Layout::UInt32 => primitive!(UInt32Array, 0, core::convert::identity),
        Layout::UInt64 => primitive!(UInt64Array, 0, core::convert::identity),
        Layout::Float32 => primitive!(Float32Array, 0.0, |value| f32::from_bits(
            crate::canonical_f32_bits(value)
        )),
        Layout::Float64 => primitive!(Float64Array, 0.0, |value| f64::from_bits(
            crate::canonical_f64_bits(value)
        )),
        Layout::TimestampNsUtc => {
            let values = cast::<TimestampNanosecondArray>(array)?;
            Arc::new(
                TimestampNanosecondArray::from(selected(array, selection, 0, |index| {
                    values.value(index)
                }))
                .with_timezone("UTC"),
            )
        }
        Layout::Utf8 | Layout::Enum { .. } => text(layout, array, selection)?,
        Layout::Binary => binary(array, selection)?,
        Layout::FixedSizeBinary(width) => fixed_binary(array, selection, *width)?,
        Layout::List(inner) | Layout::FixedSizeList(inner, _) => {
            list(source, layout, inner, array, selection)?
        }
        Layout::Struct(children) => structure(source, children, array, selection)?,
    })
}

pub(super) fn enum_value(
    array: &dyn Array,
    key: Option<DictKey>,
    index: usize,
) -> Result<Option<&str>, CanonError> {
    macro_rules! decode {
        ($key:ty) => {{
            let dictionary = cast::<DictionaryArray<$key>>(array)?;
            let Some(index) = dictionary.key(index) else {
                return Ok(None);
            };
            let values = cast::<StringArray>(dictionary.values().as_ref())?;
            if index >= values.len() {
                return Err(CanonError::Internal(
                    "invalid dictionary escaped Arrow admission".to_owned(),
                ));
            }
            if values.is_null(index) {
                None
            } else {
                Some(values.value(index))
            }
        }};
    }
    Ok(match key {
        Some(DictKey::Int8) => decode!(Int8Type),
        Some(DictKey::Int32) => decode!(Int32Type),
        None => {
            let values = cast::<StringArray>(array)?;
            (!values.is_null(index)).then(|| values.value(index))
        }
    })
}
fn text(
    layout: &Layout,
    array: &dyn Array,
    selection: &[Option<usize>],
) -> Result<ArrayRef, CanonError> {
    let mut builder = StringBuilder::new();
    for index in selection {
        match index {
            None => builder.append_value(""),
            Some(index) if array.is_null(*index) => builder.append_null(),
            Some(index) => match layout {
                Layout::Enum { key, .. } => match enum_value(array, *key, *index)? {
                    Some(value) => builder.append_value(value),
                    None => builder.append_null(),
                },
                _ => builder.append_value(cast::<StringArray>(array)?.value(*index)),
            },
        }
    }
    Ok(Arc::new(builder.finish()))
}
fn binary(array: &dyn Array, selection: &[Option<usize>]) -> Result<ArrayRef, CanonError> {
    let source = cast::<BinaryArray>(array)?;
    let mut builder = BinaryBuilder::new();
    for index in selection {
        match index {
            None => builder.append_value([]),
            Some(index) if array.is_null(*index) => builder.append_null(),
            Some(index) => builder.append_value(source.value(*index)),
        }
    }
    Ok(Arc::new(builder.finish()))
}
fn fixed_binary(
    array: &dyn Array,
    selection: &[Option<usize>],
    width: i32,
) -> Result<ArrayRef, CanonError> {
    let source = cast::<FixedSizeBinaryArray>(array)?;
    let mut builder = FixedSizeBinaryBuilder::with_capacity(selection.len(), width);
    let zero = if selection.iter().any(Option::is_none) {
        vec![
            0;
            usize::try_from(width)
                .map_err(|_| CanonError::Internal("negative admitted binary width".to_owned()))?
        ]
    } else {
        Vec::new()
    };
    for index in selection {
        match index {
            None => builder.append_value(&zero)?,
            Some(index) if array.is_null(*index) => builder.append_null(),
            Some(index) => builder.append_value(source.value(*index))?,
        }
    }
    Ok(Arc::new(builder.finish()))
}
fn structure(
    source: &Field,
    layouts: &[(String, Layout)],
    array: &dyn Array,
    selection: &[Option<usize>],
) -> Result<ArrayRef, CanonError> {
    let source_array = cast::<StructArray>(array)?;
    let DataType::Struct(source_fields) = source.data_type() else {
        return Err(CanonError::Internal("struct field mismatch".to_owned()));
    };
    let child_selection = selection
        .iter()
        .map(|index| index.filter(|index| array.is_valid(*index)))
        .collect::<Vec<_>>();
    let children = source_fields
        .iter()
        .zip(layouts)
        .zip(source_array.columns())
        .map(|((field, (_, layout)), array)| {
            array_values(field, layout, array.as_ref(), &child_selection)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let fields = source_fields
        .iter()
        .zip(layouts)
        .map(|(source, (_, layout))| field(source, layout))
        .collect();
    Ok(Arc::new(StructArray::try_new_with_length(
        fields,
        children,
        validity(array, selection),
        selection.len(),
    )?))
}
fn list(
    source: &Field,
    layout: &Layout,
    inner: &Layout,
    array: &dyn Array,
    selection: &[Option<usize>],
) -> Result<ArrayRef, CanonError> {
    let mut child_selection = Vec::new();
    let mut offsets = vec![0i32];
    let (child, values) = match source.data_type() {
        DataType::List(child) => {
            let list = cast::<ListArray>(array)?;
            (child, list.values())
        }
        DataType::FixedSizeList(child, _) => {
            let list = cast::<FixedSizeListArray>(array)?;
            (child, list.values())
        }
        _ => return Err(CanonError::Internal("list field mismatch".to_owned())),
    };
    for index in selection {
        match layout {
            Layout::List(_) => {
                if let Some(index) = index.filter(|index| array.is_valid(*index)) {
                    let list = cast::<ListArray>(array)?;
                    let start = usize::try_from(list.value_offsets()[index])
                        .map_err(|_| CanonError::Internal("negative admitted offset".to_owned()))?;
                    let end = usize::try_from(list.value_offsets()[index + 1])
                        .map_err(|_| CanonError::Internal("negative admitted offset".to_owned()))?;
                    child_selection.extend((start..end).map(Some));
                }
            }
            Layout::FixedSizeList(_, width) => {
                let width = usize::try_from(*width)
                    .map_err(|_| CanonError::Internal("negative admitted list width".to_owned()))?;
                if let Some(index) = index.filter(|index| array.is_valid(*index)) {
                    let start = index.checked_mul(width).ok_or_else(|| {
                        CanonError::Internal("admitted offset overflow".to_owned())
                    })?;
                    child_selection.extend((start..start + width).map(Some));
                } else {
                    child_selection.extend((0..width).map(|_| None));
                }
            }
            _ => return Err(CanonError::Internal("list layout mismatch".to_owned())),
        }
        offsets.push(
            i32::try_from(child_selection.len()).map_err(|_| CanonError::Envelope {
                what: crate::EnvelopeBound::Offset,
                limit: i32::MAX as u64,
                actual: u64::try_from(child_selection.len()).unwrap_or(u64::MAX),
            })?,
        );
    }
    let normalized = array_values(child, inner, values.as_ref(), &child_selection)?;
    let child = Arc::new(field(child, inner));
    Ok(match layout {
        Layout::List(_) => Arc::new(ListArray::try_new(
            child,
            OffsetBuffer::new(ScalarBuffer::from(offsets)),
            normalized,
            validity(array, selection),
        )?),
        Layout::FixedSizeList(_, width) => Arc::new(FixedSizeListArray::try_new_with_length(
            child,
            *width,
            normalized,
            validity(array, selection),
            selection.len(),
        )?),
        _ => return Err(CanonError::Internal("list layout mismatch".to_owned())),
    })
}
