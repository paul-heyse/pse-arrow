// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Test-only adapters for fixtures expressed in the public native literal format.
//! Production bulk construction uses generated Arrow builders, never scalar literals.
use crate::RelationError;
use arrow_array::{ArrayRef, RecordBatch};
use arrow_schema::Field;
use pse_schema::{NativeLiteral, Registry, model::RelationSpec};
use serde_json::Value;
use std::sync::Arc;

/// Build and validate an array from field-directed literal fixtures.
/// # Errors
/// Representation or declared semantic admission fails.
pub fn array_from_literals(
    registry: &Registry,
    field: &Field,
    values: &[Value],
) -> Result<ArrayRef, RelationError> {
    let array = literal_array(field, values)?;
    crate::validate::validate_column(registry, field, array.as_ref())
        .map_err(|errors| RelationError::Validation { errors })?;
    Ok(array)
}
fn literal_array(field: &Field, values: &[Value]) -> Result<ArrayRef, RelationError> {
    if values.is_empty() {
        return Ok(arrow_array::new_empty_array(field.data_type()));
    }
    let arrays = values
        .iter()
        .map(|value| {
            NativeLiteral::from_json(Arc::new(field.clone()), &value.to_string())
                .map(|v| Arc::clone(v.array()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(arrow::compute::concat(
        &arrays.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
    )?)
}
/// Build a fixture under its explicit registry declaration.
/// # Errors
/// Any width, representation or semantic violation.
pub fn batch_from_literals(
    registry: &Registry,
    spec: &RelationSpec,
    rows: &[Vec<Value>],
) -> Result<RecordBatch, RelationError> {
    let batch = untrusted_batch_from_literals(registry, spec, rows)?;
    crate::validate::validate_batch(registry, spec, &batch)
        .map_err(|errors| RelationError::Validation { errors })?;
    Ok(batch)
}
/// Build structurally valid Arrow input without certifying its declared domains.
/// Use for negative admission fixtures and primitive codec/canonicalization oracles.
/// # Errors
/// A width, literal representation or Arrow layout is invalid.
pub fn untrusted_batch_from_literals(
    registry: &Registry,
    spec: &RelationSpec,
    rows: &[Vec<Value>],
) -> Result<RecordBatch, RelationError> {
    let schema = Arc::new(pse_schema::arrow::relation_schema(registry, spec)?);
    if rows.iter().any(|row| row.len() != schema.fields().len()) {
        return Err(crate::columnar::mismatch("native fixture width"));
    }
    let columns = schema
        .fields()
        .iter()
        .enumerate()
        .map(|(i, f)| literal_array(f, &rows.iter().map(|r| r[i].clone()).collect::<Vec<_>>()))
        .collect::<Result<_, _>>()?;
    let batch = RecordBatch::try_new(schema, columns)?;
    Ok(batch)
}
/// Encode generated rows as untrusted Arrow input for negative boundary fixtures.
/// This uses the generated structural codec without claiming local value admission.
/// # Errors
/// A generated declaration, value encoding or Arrow layout is invalid.
pub fn untrusted_batch_from_rows<T: crate::columnar::RelationRow + crate::columnar::ArrowValue>(
    registry: &Registry,
    rows: &[T],
) -> Result<RecordBatch, RelationError> {
    let schema = Arc::new(pse_schema::arrow::relation_schema(
        registry,
        T::relation(registry)?,
    )?);
    let kind = arrow_schema::DataType::Struct(schema.fields().clone());
    let arrays = rows
        .iter()
        .map(|row| row.to_array(&kind))
        .collect::<Result<Vec<_>, _>>()?;
    let array = if arrays.is_empty() {
        arrow_array::new_empty_array(&kind)
    } else {
        arrow::compute::concat(&arrays.iter().map(AsRef::as_ref).collect::<Vec<_>>())?
    };
    let values = array
        .as_any()
        .downcast_ref::<arrow_array::StructArray>()
        .ok_or_else(|| crate::columnar::mismatch("generated row struct"))?;
    Ok(RecordBatch::try_new(schema, values.columns().to_vec())?)
}
/// Inspect admitted output through the lossless literal interchange boundary.
/// # Errors
/// Invalid batch or unsupported literal representation.
pub fn literals_from_batch(
    registry: &Registry,
    spec: &RelationSpec,
    batch: &RecordBatch,
) -> Result<Vec<Vec<Value>>, RelationError> {
    crate::validate::validate_batch(registry, spec, batch)
        .map_err(|errors| RelationError::Validation { errors })?;
    literal_rows(batch)
}

/// Observe an arbitrary native projection through its own exact fields.
/// # Errors
/// A malformed native array or literal representation.
pub fn literal_rows(batch: &RecordBatch) -> Result<Vec<Vec<Value>>, RelationError> {
    for array in batch.columns() {
        array.to_data().validate_full()?;
    }
    (0..batch.num_rows())
        .map(|row| {
            batch
                .schema()
                .fields()
                .iter()
                .zip(batch.columns())
                .map(|(f, a)| {
                    let literal = pse_schema::literal::to_json(a.as_ref(), f, row)?;
                    serde_json::from_str(&literal)
                        .map_err(|_| crate::columnar::mismatch("native literal JSON"))
                })
                .collect()
        })
        .collect()
}

/// Encode a generated fixture value through its declared native field.
/// # Errors
/// A generated value cannot be represented by the supplied field.
pub fn literal_value<T: crate::columnar::ArrowValue>(
    field: &Field,
    value: &T,
) -> Result<Value, RelationError> {
    let array = value.to_array(field.data_type())?;
    let text = pse_schema::literal::to_json(array.as_ref(), field, 0)?;
    serde_json::from_str(&text).map_err(|_| crate::columnar::mismatch("native fixture literal"))
}
