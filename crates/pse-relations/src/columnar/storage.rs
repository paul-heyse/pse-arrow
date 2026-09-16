// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Checked construction of the Arrow storage shapes admitted by the registry.

use arrow_array::builder::{
    ArrayBuilder, FixedSizeListBuilder, ListBuilder, StructBuilder, make_builder,
};
use arrow_schema::{DataType, TimeUnit};

use crate::RelationError;

pub(super) fn make(
    kind: &DataType,
    capacity: usize,
) -> Result<Box<dyn ArrayBuilder>, RelationError> {
    Ok(match kind {
        DataType::List(field) => Box::new(
            ListBuilder::with_capacity(make(field.data_type(), capacity)?, capacity)
                .with_field(field.clone()),
        ),
        DataType::FixedSizeList(field, width) => {
            let count = usize::try_from(*width)
                .map_err(|_| super::mismatch("nonnegative fixed-list width"))?;
            let values = capacity
                .checked_mul(count)
                .ok_or_else(|| super::mismatch("representable fixed-list capacity"))?;
            Box::new(
                FixedSizeListBuilder::with_capacity(
                    make(field.data_type(), values)?,
                    *width,
                    capacity,
                )
                .with_field(field.clone()),
            )
        }
        DataType::Struct(fields) => {
            let builders = fields
                .iter()
                .map(|field| make(field.data_type(), capacity))
                .collect::<Result<_, _>>()?;
            Box::new(StructBuilder::new(fields.clone(), builders))
        }
        DataType::Boolean
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64
        | DataType::Float64
        | DataType::Utf8
        | DataType::FixedSizeBinary(16 | 32)
        | DataType::Timestamp(TimeUnit::Nanosecond, _) => make_builder(kind, capacity),
        _ => return Err(super::mismatch("a registered generated Arrow storage type")),
    })
}
