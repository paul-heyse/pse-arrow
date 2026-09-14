// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Active admission of declared storage and visible logical values. Batch admission
//! does not certify keys or foreign keys; publication additionally calls [`validate_bundle`].

mod bundle;
mod field;
mod values;

use arrow_array::{Array, RecordBatch};
use arrow_schema::{Field, Schema};
use pse_ids::{ContentHash, SemanticId};
use pse_schema::Registry;
use pse_schema::model::RelationSpec;

use crate::RelationError;

pub use bundle::validate_bundle;
pub use field::validate_field;

pub(super) fn finish(errors: Vec<RelationError>) -> Result<(), Vec<RelationError>> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
pub(super) fn mismatch(relation: &str, reason: impl Into<String>) -> RelationError {
    RelationError::Contract {
        relation: relation.to_owned(),
        reason: reason.into(),
    }
}

/// Admits a schema by direct comparison with the exact declaration in `reg`, including
/// every field, order, nullability and metadata entry. A matching digest cannot bypass it.
///
/// # Errors
/// Every independent schema or field mismatch found.
pub fn validate_schema(
    reg: &Registry,
    spec: &RelationSpec,
    schema: &Schema,
) -> Result<(), Vec<RelationError>> {
    use pse_schema::arrow::KEY_CONTRACT_FINGERPRINT;
    let mut errors = Vec::new();
    if reg.relation_by_id(spec.id) != Some(spec) {
        errors.push(mismatch(
            &spec.key.to_string(),
            "descriptor does not equal its registry declaration",
        ));
    }
    let expected = match pse_schema::arrow::relation_schema(reg, spec) {
        Ok(schema) => schema,
        Err(error) => {
            errors.push(error.into());
            return Err(errors);
        }
    };
    if let Some(actual) = schema
        .metadata()
        .get(KEY_CONTRACT_FINGERPRINT)
        .and_then(|text| ContentHash::parse_hex(text).ok())
        && actual != spec.fingerprint
    {
        errors.push(RelationError::FingerprintMismatch {
            relation: spec.key.to_string(),
            expected: spec.fingerprint,
            actual,
        });
    }
    for (key, value) in expected.metadata() {
        if schema.metadata().get(key) != Some(value) {
            errors.push(mismatch(
                &spec.key.to_string(),
                format!("metadata {key} differs from its declaration"),
            ));
        }
    }
    for (key, value) in schema.metadata() {
        if expected.metadata().contains_key(key) {
            continue;
        }
        if matches!(key.as_str(), "pse.snapshot_id" | "pse.producer_pass_id") {
            let invalid = if key == "pse.snapshot_id" {
                ContentHash::parse_prefixed(value).map_or(true, |hash| hash.to_prefixed() != *value)
            } else {
                SemanticId::parse_hex(value).map_or(true, |id| id.to_hex() != *value)
            };
            if invalid {
                errors.push(mismatch(
                    &spec.key.to_string(),
                    format!("contextual identity {key} is malformed"),
                ));
            }
        } else {
            errors.push(RelationError::UnknownMetadata {
                field: String::new(),
                key: key.clone(),
            });
        }
    }
    if schema.fields().len() != expected.fields().len() {
        errors.push(mismatch(
            &spec.key.to_string(),
            "field count differs from declared schema",
        ));
    }
    for (index, actual) in schema.fields().iter().enumerate() {
        if let Err(found) = validate_field(reg, actual) {
            errors.extend(found);
        }
        if let Some(expected) = expected.fields().get(index) {
            compare_field(
                &format!("{}.{index}", spec.key),
                expected,
                actual,
                &mut errors,
            );
        }
    }
    finish(errors)
}

fn compare_field(path: &str, expected: &Field, actual: &Field, errors: &mut Vec<RelationError>) {
    if expected.name() != actual.name() {
        errors.push(mismatch(path, "field name or column order differs"));
    }
    if expected.data_type() != actual.data_type() {
        errors.push(RelationError::Storage {
            field: path.to_owned(),
            expected: expected.data_type().to_string(),
            actual: actual.data_type().to_string(),
        });
    }
    if expected.is_nullable() != actual.is_nullable() {
        errors.push(RelationError::Nullability {
            field: path.to_owned(),
            declared: if expected.is_nullable() {
                "nullable"
            } else {
                "non-null"
            },
            actual: format!("nullable={}", actual.is_nullable()),
        });
    }
    if expected.metadata() != actual.metadata() {
        errors.push(mismatch(
            path,
            "field metadata differs from declared metadata",
        ));
    }
}

/// Admits storage, schema declarations and visible values, respecting null parent masks.
/// This accepts candidate rows before P2; keys and references require bundle admission.
///
/// # Errors
/// All schema violations, or all independently decodable row-value violations.
pub fn validate_batch(
    reg: &Registry,
    spec: &RelationSpec,
    batch: &RecordBatch,
) -> Result<(), Vec<RelationError>> {
    validate_schema(reg, spec, &batch.schema())?;
    let mut errors = Vec::new();
    for array in batch.columns() {
        if let Err(error) = array.to_data().validate_full() {
            errors.push(error.into());
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let schema = batch.schema();
    for row in 0..batch.num_rows() {
        let mut cells = Vec::with_capacity(batch.num_columns());
        for (field, array) in schema.fields().iter().zip(batch.columns()) {
            match crate::cells::cell_at(reg, field, array.as_ref(), row) {
                Ok(cell) => {
                    values::validate_cell(field, &cell, row, field.name(), &mut errors);
                    cells.push(cell);
                }
                Err(error) => {
                    errors.push(error);
                    cells.push(pse_schema::model::Cell::Null);
                }
            }
        }
        values::validate_quantities(spec, &cells, row, &mut errors);
    }
    finish(errors)
}

/// Validate a single declared field's storage and visible recursive logical values.
/// This is the checked rule-head/literal column boundary; it does not manufacture a
/// relation declaration or certify cross-column quantities, keys or foreign keys.
///
/// # Errors
/// Field metadata/storage violations or visible logical-value errors.
pub fn validate_column(
    reg: &Registry,
    field: &Field,
    array: &dyn Array,
) -> Result<(), Vec<RelationError>> {
    validate_field(reg, field)?;
    if field.data_type() != array.data_type() {
        return Err(vec![RelationError::Storage {
            field: field.name().to_owned(),
            expected: field.data_type().to_string(),
            actual: array.data_type().to_string(),
        }]);
    }
    array
        .to_data()
        .validate_full()
        .map_err(|error| vec![error.into()])?;
    let mut errors = Vec::new();
    for row in 0..array.len() {
        match crate::cells::cell_at(reg, field, array, row) {
            Ok(cell) => values::validate_cell(field, &cell, row, field.name(), &mut errors),
            Err(error) => errors.push(error),
        }
    }
    finish(errors)
}
