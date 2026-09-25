// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native construction helpers, private to the declaration projection.
use super::invalid;
use crate::SchemaError;
use arrow_array::{
    Array, ArrayRef, BooleanArray, FixedSizeBinaryArray, Int64Array, ListArray, StringArray,
    StructArray,
};
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow_schema::{DataType, Field};
use std::sync::Arc;

pub(super) fn text<S: AsRef<str>>(values: impl IntoIterator<Item = Option<S>>) -> ArrayRef {
    Arc::new(StringArray::from_iter(values))
}
pub(super) fn ids(
    values: impl IntoIterator<Item = Option<pse_ids::SemanticId>>,
) -> Result<ArrayRef, SchemaError> {
    let values = values
        .into_iter()
        .map(|v| v.map(|v| *v.as_bytes()))
        .collect::<Vec<_>>();
    Ok(Arc::new(
        FixedSizeBinaryArray::try_from_sparse_iter_with_size(values.into_iter(), 16)
            .map_err(invalid)?,
    ))
}
pub(super) fn int(values: impl IntoIterator<Item = Option<i64>>) -> ArrayRef {
    Arc::new(Int64Array::from_iter(values))
}
pub(super) fn boolean(values: impl IntoIterator<Item = bool>) -> ArrayRef {
    Arc::new(BooleanArray::from_iter(values.into_iter().map(Some)))
}
pub(super) fn list(
    lengths: impl IntoIterator<Item = usize>,
    values: ArrayRef,
) -> Result<ArrayRef, SchemaError> {
    let mut offsets = vec![0_i32];
    let mut count = 0_i32;
    for length in lengths {
        count = count
            .checked_add(i32::try_from(length).map_err(invalid)?)
            .ok_or_else(|| invalid("list offset overflow"))?;
        offsets.push(count);
    }
    if usize::try_from(count).map_err(invalid)? != values.len() {
        return Err(invalid("list lengths differ from values"));
    }
    Ok(Arc::new(
        ListArray::try_new(
            Arc::new(Field::new("item", values.data_type().clone(), true)),
            OffsetBuffer::new(ScalarBuffer::from(offsets)),
            values,
            None,
        )
        .map_err(invalid)?,
    ))
}
pub(super) fn structure(columns: Vec<(&str, ArrayRef)>) -> Result<ArrayRef, SchemaError> {
    structure_nullable(columns, None)
}
pub(super) fn structure_nullable(
    columns: Vec<(&str, ArrayRef)>,
    validity: Option<Vec<bool>>,
) -> Result<ArrayRef, SchemaError> {
    let fields = columns
        .iter()
        .map(|(name, a)| Field::new(*name, a.data_type().clone(), true))
        .collect::<Vec<_>>();
    Ok(Arc::new(
        StructArray::try_new(
            fields.into(),
            columns.into_iter().map(|(_, a)| a).collect(),
            validity.map(NullBuffer::from),
        )
        .map_err(invalid)?,
    ))
}
// Native cast handles scalar and dictionary storage. Containers retain the exact
// declared child fields and validity instead of inferring metadata from values.
pub(super) fn align(array: ArrayRef, ty: &DataType) -> Result<ArrayRef, SchemaError> {
    match ty {
        DataType::Struct(fields) => {
            let input = array
                .as_any()
                .downcast_ref::<StructArray>()
                .ok_or_else(|| invalid("expected native struct"))?;
            if fields.len() != input.num_columns() {
                return Err(invalid("struct projection width"));
            }
            let children = input
                .columns()
                .iter()
                .zip(fields)
                .map(|(a, f)| align(Arc::clone(a), f.data_type()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Arc::new(
                StructArray::try_new(fields.clone(), children, input.nulls().cloned())
                    .map_err(invalid)?,
            ))
        }
        DataType::List(field) => {
            let input = array
                .as_any()
                .downcast_ref::<ListArray>()
                .ok_or_else(|| invalid("expected native list"))?;
            Ok(Arc::new(
                ListArray::try_new(
                    Arc::clone(field),
                    input.offsets().clone(),
                    align(Arc::clone(input.values()), field.data_type())?,
                    input.nulls().cloned(),
                )
                .map_err(invalid)?,
            ))
        }
        _ if array.data_type() == ty => Ok(array),
        _ => arrow_cast::cast_with_options(
            array.as_ref(),
            ty,
            &arrow_cast::CastOptions {
                safe: false,
                ..Default::default()
            },
        )
        .map_err(invalid),
    }
}
