// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared representation guards used before Arrow's infallible schema conversion
//! and array constructors. Registry meaning and scientific lineage are caller owned.
use arrow::{datatypes::{DataType, Schema}, error::ArrowError, ipc};
const MAX_REPRESENTATION_BYTES: usize = 32 * 1024 * 1024;
const MAX_VALUES: usize = MAX_REPRESENTATION_BYTES;
const MAX_DEPTH: usize = 64;
const MAX_FIELDS: usize = 16 * 1024;
fn invalid(message: &'static str) -> ArrowError { ArrowError::InvalidArgumentError(message.into()) }
/// Check native type support and bounded traversal before writing or decoding.
/// # Errors
/// Unsupported types or excessive depth/field count.
pub fn trusted_shape(ty: &DataType, depth: usize, count: &mut usize) -> Result<(), ArrowError> {
    *count += 1;
    if depth > MAX_DEPTH || *count > MAX_FIELDS {
        return Err(invalid("schema complexity"));
    }
    match ty {
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
        | DataType::Float16
        | DataType::Float32
        | DataType::Float64
        | DataType::Utf8
        | DataType::LargeUtf8
        | DataType::Binary
        | DataType::LargeBinary
        | DataType::Date32
        | DataType::Date64
        | DataType::Time32(
            arrow::datatypes::TimeUnit::Second
            | arrow::datatypes::TimeUnit::Millisecond,
        )
        | DataType::Time64(
            arrow::datatypes::TimeUnit::Microsecond
            | arrow::datatypes::TimeUnit::Nanosecond,
        )
        | DataType::Timestamp(_, _)
        | DataType::Duration(_)
        | DataType::Decimal32(_, _)
        | DataType::Decimal64(_, _)
        | DataType::Decimal128(_, _)
        | DataType::Decimal256(_, _) => {}
        DataType::FixedSizeBinary(n) if *n > 0 && (*n as usize) <= MAX_REPRESENTATION_BYTES => {}
        DataType::List(child) | DataType::LargeList(child) | DataType::Map(child, _) => {
            trusted_shape(child.data_type(), depth + 1, count)?;
        }
        DataType::FixedSizeList(child, n) if *n > 0 && (*n as usize) <= MAX_VALUES => {
            trusted_shape(child.data_type(), depth + 1, count)?;
        }
        DataType::Struct(children) => {
            for child in children {
                trusted_shape(child.data_type(), depth + 1, count)?;
            }
        }
        _ => return Err(invalid("unsupported schema type")),
    }
    Ok(())
}

/// Check every FlatBuffer type discriminator before Arrow's infallible conversion.
/// # Errors
/// Unsupported, incomplete or excessively nested field declarations.
pub fn schema_field(
    field: ipc::Field<'_>,
    depth: usize,
    count: &mut usize,
    expected_fields: usize,
) -> Result<(), ArrowError> {
    *count += 1;
    if depth > MAX_DEPTH
        || *count > expected_fields
        || field.dictionary().is_some()
        || field.type_().is_none()
    {
        return Err(invalid(
            "schema complexity/dictionary/type",
        ));
    }
    let children = field.children();
    let child_count = children.map_or(0, |c| c.len());
    let valid = match field.type_type() {
        ipc::Type::Null
        | ipc::Type::Bool
        | ipc::Type::Utf8
        | ipc::Type::LargeUtf8
        | ipc::Type::Binary
        | ipc::Type::LargeBinary => child_count == 0,
        ipc::Type::Int => {
            field
                .type_as_int()
                .is_some_and(|v| matches!(v.bitWidth(), 8 | 16 | 32 | 64))
                && child_count == 0
        }
        ipc::Type::FloatingPoint => {
            field.type_as_floating_point().is_some_and(|v| {
                matches!(
                    v.precision(),
                    ipc::Precision::HALF | ipc::Precision::SINGLE | ipc::Precision::DOUBLE
                )
            }) && child_count == 0
        }
        ipc::Type::FixedSizeBinary => {
            field.type_as_fixed_size_binary().is_some_and(|v| {
                v.byteWidth() > 0 && (v.byteWidth() as usize) <= MAX_REPRESENTATION_BYTES
            }) && child_count == 0
        }
        ipc::Type::Date => {
            field.type_as_date().is_some_and(|v| {
                matches!(v.unit(), ipc::DateUnit::DAY | ipc::DateUnit::MILLISECOND)
            }) && child_count == 0
        }
        ipc::Type::Time => {
            field.type_as_time().is_some_and(|v| {
                matches!(
                    (v.bitWidth(), v.unit()),
                    (32, ipc::TimeUnit::SECOND | ipc::TimeUnit::MILLISECOND)
                        | (64, ipc::TimeUnit::MICROSECOND | ipc::TimeUnit::NANOSECOND)
                )
            }) && child_count == 0
        }
        ipc::Type::Timestamp => {
            field
                .type_as_timestamp()
                .is_some_and(|v| time_unit(v.unit()))
                && child_count == 0
        }
        ipc::Type::Duration => {
            field
                .type_as_duration()
                .is_some_and(|v| time_unit(v.unit()))
                && child_count == 0
        }
        ipc::Type::Decimal => {
            field.type_as_decimal().is_some_and(|v| {
                matches!(v.bitWidth(), 32 | 64 | 128 | 256)
                    && u8::try_from(v.precision()).is_ok()
                    && i8::try_from(v.scale()).is_ok()
            }) && child_count == 0
        }
        ipc::Type::List | ipc::Type::LargeList | ipc::Type::Map => child_count == 1,
        ipc::Type::FixedSizeList => {
            field
                .type_as_fixed_size_list()
                .is_some_and(|v| v.listSize() > 0 && (v.listSize() as usize) <= MAX_VALUES)
                && child_count == 1
        }
        ipc::Type::Struct_ => true,
        _ => false,
    };
    if !valid {
        return Err(invalid(
            "invalid or unsupported schema type",
        ));
    }
    if let Some(children) = children {
        for child in children {
            schema_field(child, depth + 1, count, expected_fields)?;
        }
    }
    Ok(())
}
fn time_unit(unit: ipc::TimeUnit) -> bool {
    matches!(
        unit,
        ipc::TimeUnit::SECOND
            | ipc::TimeUnit::MILLISECOND
            | ipc::TimeUnit::MICROSECOND
            | ipc::TimeUnit::NANOSECOND
    )
}

/// Validate bitmap and data minima before decoder constructors (some Arrow
/// constructors assert their buffer lengths). The nested node traversal is exactly
/// the declared schema traversal; buffers cannot be borrowed from another field.
/// # Errors
/// Missing/short buffers or inconsistent node lengths.
pub fn layout(
    ty: &DataType,
    nodes: &[&ipc::FieldNode],
    buffers: &[&ipc::Buffer],
    node: &mut usize,
    buffer: &mut usize,
    expected: Option<usize>,
) -> Result<(), ArrowError> {
    let field = nodes
        .get(*node)
        .ok_or(invalid("missing layout node"))?;
    *node += 1;
    let n = usize::try_from(field.length())
        .map_err(|_| invalid("negative layout length"))?;
    if expected.is_some_and(|v| n != v) {
        return Err(invalid("nested field length"));
    }
    if matches!(ty, DataType::Null) {
        if field.null_count() != field.length() {
            return Err(invalid("null field shape"));
        }
        return Ok(());
    }
    let mut take = |minimum: usize| -> Result<(), ArrowError> {
        let value = buffers
            .get(*buffer)
            .ok_or(invalid("missing layout buffer"))?;
        *buffer += 1;
        if usize::try_from(value.length())
            .ok()
            .is_none_or(|len| len < minimum)
        {
            return Err(invalid("short layout buffer"));
        }
        Ok(())
    };
    take(if field.null_count() > 0 {
        n.div_ceil(8)
    } else {
        0
    })?;
    match ty {
        DataType::Struct(children) => {
            for child in children {
                layout(child.data_type(), nodes, buffers, node, buffer, Some(n))?;
            }
        }
        DataType::FixedSizeList(child, width) => {
            let size = n
                .checked_mul(*width as usize)
                .ok_or(invalid("list size overflow"))?;
            if size > MAX_VALUES {
                return Err(invalid("list size bound"));
            }
            layout(child.data_type(), nodes, buffers, node, buffer, Some(size))?;
        }
        DataType::List(child) | DataType::Map(child, _) => {
            take((n + 1) * 4)?;
            layout(child.data_type(), nodes, buffers, node, buffer, None)?;
        }
        DataType::LargeList(child) => {
            take((n + 1) * 8)?;
            layout(child.data_type(), nodes, buffers, node, buffer, None)?;
        }
        DataType::Utf8 | DataType::Binary => {
            take((n + 1) * 4)?;
            take(0)?;
        }
        DataType::LargeUtf8 | DataType::LargeBinary => {
            take((n + 1) * 8)?;
            take(0)?;
        }
        DataType::Boolean => take(n.div_ceil(8))?,
        DataType::FixedSizeBinary(width) => take(
            n.checked_mul(*width as usize)
                .ok_or(invalid("binary size overflow"))?,
        )?,
        other => {
            let width = other
                .primitive_width()
                .ok_or(invalid("unsupported primitive layout"))?;
            take(
                n.checked_mul(width)
                    .ok_or(invalid("primitive size overflow"))?,
            )?;
        }
    }
    Ok(())
}

/// Validate the declared supported schema shape before native conversion.
pub fn expected_shape(expected: &Schema) -> Result<usize, ArrowError> {
    let mut fields = 0;
    for field in expected.fields() {
        trusted_shape(field.data_type(), 0, &mut fields)?;
    }
    Ok(fields)
}
