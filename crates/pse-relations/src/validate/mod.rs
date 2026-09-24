// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Active admission of declared storage and visible logical values. Batch admission
//! does not certify keys or foreign keys; publication additionally binds [`obligations::ObligationTemplates`].

pub mod obligations;
mod occurrences;
pub mod planner;
pub mod predicates;
mod prepared;
pub mod row_checks;
pub use prepared::{
    PreparedLocalContract, ValidationContext, ValidationReport, prepare_expression,
};
mod field;

use arrow_array::{Array, RecordBatch};
use arrow_schema::{Field, Schema};
use pse_ids::ContentHash;
use pse_schema::Registry;
use pse_schema::model::RelationSpec;

use crate::RelationError;

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
    if let Err(error) = reg.contract(spec) {
        errors.push(error.into());
        return Err(errors);
    }
    let Some(spec) = reg.relation_by_id(spec.id) else {
        errors.push(mismatch(&spec.key.to_string(), "relation is not declared"));
        return Err(errors);
    };
    let expected = match pse_schema::arrow::relation_schema_ref(reg, spec) {
        Ok(schema) => schema,
        Err(error) => {
            errors.push(error.into());
            return Err(errors);
        }
    };
    if std::ptr::eq(schema, expected.as_ref()) {
        return Ok(());
    }
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
    for key in schema.metadata().keys() {
        if expected.metadata().contains_key(key) {
            continue;
        }
        errors.push(RelationError::UnknownMetadata {
            field: String::new(),
            key: key.clone(),
        });
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
        if let Some(expected) = expected.fields().get(index)
            && let Err(error) = pse_schema::field_contract::execution_field(
                actual,
                expected,
                &format!("{}.{index}", spec.key),
            )
        {
            errors.push(error.into());
        }
    }
    finish(errors)
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
    let context = ValidationContext::for_registry(reg).map_err(|error| vec![error])?;
    context
        .relation(reg, spec)
        .and_then(|prepared| prepared.evaluate(batch, 256, &pse_columnar::CancellationToken::new()))
        .and_then(ValidationReport::require_valid)
        .map_err(|error| vec![error])
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
    let context = ValidationContext::for_registry(reg).map_err(|error| vec![error])?;
    let prepared = context.column(reg, field).map_err(|error| vec![error])?;
    array
        .to_data()
        .validate_full()
        .map_err(|error| vec![error.into()])?;
    let batch = RecordBatch::try_new(
        std::sync::Arc::clone(prepared.schema()),
        vec![arrow_array::make_array(array.to_data())],
    )
    .map_err(|error| vec![error.into()])?;
    prepared
        .evaluate(&batch, 256, &pse_columnar::CancellationToken::new())
        .and_then(ValidationReport::require_valid)
        .map_err(|error| vec![error])
}
