// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native Arrow casts visit physical children, including bytes hidden by parents.
//! Propagate struct masks and compact null-container ranges through Arrow kernels
//! before converting the declared Delta storage shape back to semantic values.
use datafusion::{
    arrow::{
        array::{Array, ArrayRef, StructArray, UInt64Array, make_array},
        compute::take,
        datatypes::DataType,
    },
    common::{DataFusionError, Result},
};
use std::sync::Arc;

pub(super) fn storage(array: ArrayRef) -> Result<ArrayRef> {
    match array.data_type() {
        DataType::Struct(_) => {
            let source = array
                .as_any()
                .downcast_ref::<StructArray>()
                .ok_or_else(|| {
                    DataFusionError::Internal("declared struct has a different Arrow array".into())
                })?;
            // flatten owns parent-mask propagation, including already-null children.
            // Keep the declared fields: child nulls remain hidden by the same parent.
            let (_, children) = source.flatten();
            let children = children
                .into_iter()
                .map(storage)
                .collect::<Result<Vec<_>>>()?;
            Ok(Arc::new(StructArray::try_new_with_length(
                source.fields().clone(),
                children,
                source.nulls().cloned(),
                source.len(),
            )?))
        }
        DataType::List(_) | DataType::Map(_, _) => {
            // Delta's declared storage uses ordinary List/Map offsets. Native take
            // drops child ranges belonging to null rows while retaining row order,
            // duplicates and parent validity. It also removes inaccessible slices.
            let array = if array.null_count() == 0 {
                array
            } else {
                let length = u64::try_from(array.len())
                    .map_err(|error| DataFusionError::External(Box::new(error)))?;
                take(
                    array.as_ref(),
                    &UInt64Array::from_iter_values(0..length),
                    None,
                )?
            };
            let data = array.to_data();
            let children = data
                .child_data()
                .iter()
                .map(|child| storage(make_array(child.clone())).map(|value| value.to_data()))
                .collect::<Result<Vec<_>>>()?;
            Ok(make_array(
                data.into_builder().child_data(children).build()?,
            ))
        }
        _ => Ok(array),
    }
}

/// An internal equality view, never an admitted/output declaration. Native Arrow
/// equality ignores parent-masked slots, but its array construction must first
/// permit those slots (notably fixed-size-list -> list null-row placeholders).
pub(super) fn comparison_type(data_type: &DataType) -> DataType {
    let field = |field: &datafusion::arrow::datatypes::FieldRef, nullable| {
        Arc::new(
            field
                .as_ref()
                .clone()
                .with_data_type(comparison_type(field.data_type()))
                .with_nullable(nullable),
        )
    };
    match data_type {
        DataType::Struct(fields) => {
            DataType::Struct(fields.iter().map(|value| field(value, true)).collect())
        }
        DataType::List(value) => DataType::List(field(value, true)),
        DataType::Map(entries, sorted) => {
            // Arrow maps require non-null entries and keys. These are positional
            // storage fields; no invisible entries survive native take above.
            let DataType::Struct(fields) = entries.data_type() else {
                return data_type.clone();
            };
            let fields = fields
                .iter()
                .enumerate()
                .map(|(index, value)| field(value, index != 0))
                .collect();
            DataType::Map(
                Arc::new(
                    entries
                        .as_ref()
                        .clone()
                        .with_data_type(DataType::Struct(fields)),
                ),
                *sorted,
            )
        }
        _ => data_type.clone(),
    }
}
