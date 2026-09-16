// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact recursive admission of native Arrow fields, before physical adaptation.
//!
//! Arrow's `equals_datatype` intentionally omits names and metadata; DataFusion's
//! struct compatibility adapter admits missing and extra fields. Neither establishes
//! the identity of a declared value. This boundary uses native fields themselves.

use std::collections::HashSet;

use arrow_schema::{DataType, Field, Schema};

use crate::SchemaError;

/// Check an execution declaration and every nested field without changing its meaning.
/// Arbitrary native Arrow types are eligible; this is not a type allowlist.
/// # Errors
/// Duplicate names or any difference in fields, metadata, or nullability.
pub fn execution_schema(actual: &Schema, expected: &Schema) -> Result<(), SchemaError> {
    declaration(expected)?;
    compare_fields(actual.fields(), expected.fields(), "execution", false)?;
    if actual.metadata() != expected.metadata() {
        return Err(mismatch("execution", "schema metadata differs"));
    }
    Ok(())
}

/// Admit one field at a named boundary using the same recursive execution contract.
/// # Errors
/// Names, types, nullability or metadata differ, or nested paths are ambiguous.
pub fn execution_field(actual: &Field, expected: &Field, path: &str) -> Result<(), SchemaError> {
    declaration(&Schema::new(vec![expected.clone()]))?;
    unique_type(actual.data_type(), path, false)?;
    compare_field(actual, expected, path, false)
}

/// Admit the named Delta scan adaptation: BinaryView/Utf8View become Binary/Utf8.
/// All field names, metadata and nullability must still match. Delta list element
/// names are already part of the generated durable declaration.
/// Schema-level metadata is outside this field check: Delta has no schema-level
/// Arrow metadata slot. Durable relation identity needs its table declaration too.
/// # Errors
/// A scan differs beyond the named view-array adaptation.
pub fn delta_scan_schema(actual: &Schema, expected: &Schema) -> Result<(), SchemaError> {
    declaration(expected)?;
    compare_fields(actual.fields(), expected.fields(), "delta", true)
}

/// Refuse ambiguous field paths in a native declaration.
/// # Errors
/// A struct or schema declares the same field name twice.
pub fn declaration(schema: &Schema) -> Result<(), SchemaError> {
    crate::arrow::native_checks(schema)?;
    unique_fields(schema.fields(), "schema", true)
}

fn unique_fields(
    fields: &[arrow_schema::FieldRef],
    path: &str,
    validate_extensions: bool,
) -> Result<(), SchemaError> {
    let mut names = HashSet::new();
    for field in fields {
        if validate_extensions {
            crate::model::IntegerRange::from_field(field)?;
            crate::model::TaggedAlternative::from_field(field)?;
            crate::model::CollectionContract::from_field(field)?;
            crate::model::ReferenceContract::from_field(field)?;
        }
        if !names.insert(field.name()) {
            return Err(mismatch(path, &format!("duplicate field {}", field.name())));
        }
        if validate_extensions
            && let Some(extension) = field.metadata().get(crate::arrow::KEY_EXTENSION_NAME)
            && let Some(spec) = crate::model::EXTENSION_TYPES
                .iter()
                .find(|spec| spec.name == extension)
            && field.data_type() != &spec.storage()
        {
            return Err(mismatch(
                path,
                "PSE extension descriptor has incompatible physical storage",
            ));
        }
        unique_type(
            field.data_type(),
            &format!("{path}.{}", field.name()),
            validate_extensions,
        )?;
    }
    Ok(())
}

fn unique_type(
    data_type: &DataType,
    path: &str,
    validate_extensions: bool,
) -> Result<(), SchemaError> {
    match data_type {
        DataType::Struct(fields) => unique_fields(fields, path, validate_extensions),
        DataType::List(field)
        | DataType::LargeList(field)
        | DataType::ListView(field)
        | DataType::LargeListView(field)
        | DataType::FixedSizeList(field, _)
        | DataType::Map(field, _) => {
            unique_fields(std::slice::from_ref(field), path, validate_extensions)
        }
        DataType::Dictionary(_, value) => unique_type(value, path, validate_extensions),
        DataType::Union(fields, _) => unique_fields(
            &fields
                .iter()
                .map(|(_, field)| field.clone())
                .collect::<Vec<_>>(),
            path,
            validate_extensions,
        ),
        DataType::RunEndEncoded(run_ends, values) => unique_fields(
            &[run_ends.clone(), values.clone()],
            path,
            validate_extensions,
        ),
        _ => Ok(()),
    }
}

fn compare_fields(
    actual: &arrow_schema::Fields,
    expected: &arrow_schema::Fields,
    path: &str,
    scan: bool,
) -> Result<(), SchemaError> {
    unique_fields(actual, path, false)?;
    if actual.len() != expected.len() {
        return Err(mismatch(path, "field count differs"));
    }
    for (actual, expected) in actual.iter().zip(expected) {
        compare_field(
            actual,
            expected,
            &format!("{path}.{}", expected.name()),
            scan,
        )?;
    }
    Ok(())
}

fn compare_field(
    actual: &Field,
    expected: &Field,
    path: &str,
    scan: bool,
) -> Result<(), SchemaError> {
    if actual.name() != expected.name() {
        return Err(mismatch(path, "field name or order differs"));
    }
    if actual.is_nullable() != expected.is_nullable() {
        return Err(mismatch(path, "nullability differs"));
    }
    if actual.metadata() != expected.metadata() {
        return Err(mismatch(path, "field metadata differs"));
    }
    if actual.dict_is_ordered() != expected.dict_is_ordered() {
        return Err(mismatch(path, "dictionary ordering declaration differs"));
    }
    compare_type(actual.data_type(), expected.data_type(), path, scan)
}

fn compare_type(
    actual: &DataType,
    expected: &DataType,
    path: &str,
    scan: bool,
) -> Result<(), SchemaError> {
    match (actual, expected) {
        (DataType::BinaryView, DataType::Binary) | (DataType::Utf8View, DataType::Utf8) if scan => {
            Ok(())
        }
        (DataType::Struct(actual), DataType::Struct(expected)) => {
            compare_fields(actual, expected, path, scan)
        }
        (DataType::List(actual), DataType::List(expected))
        | (DataType::LargeList(actual), DataType::LargeList(expected))
        | (DataType::ListView(actual), DataType::ListView(expected))
        | (DataType::LargeListView(actual), DataType::LargeListView(expected)) => {
            compare_field(actual, expected, path, scan)
        }
        (DataType::FixedSizeList(actual, a), DataType::FixedSizeList(expected, b)) if a == b => {
            compare_field(actual, expected, path, scan)
        }
        (DataType::Map(actual, a), DataType::Map(expected, b)) if a == b => {
            compare_field(actual, expected, path, scan)
        }
        (DataType::Dictionary(a_key, actual), DataType::Dictionary(b_key, expected))
            if a_key == b_key =>
        {
            compare_type(actual, expected, path, scan)
        }
        (DataType::Union(actual, a), DataType::Union(expected, b)) if a == b => {
            if actual.len() != expected.len() {
                return Err(mismatch(path, "union arm count differs"));
            }
            for ((a_id, actual), (b_id, expected)) in actual.iter().zip(expected.iter()) {
                if a_id != b_id {
                    return Err(mismatch(path, "union arm identity differs"));
                }
                compare_field(actual, expected, path, scan)?;
            }
            Ok(())
        }
        (DataType::RunEndEncoded(a_ends, actual), DataType::RunEndEncoded(b_ends, expected)) => {
            compare_field(a_ends, b_ends, path, scan)?;
            compare_field(actual, expected, path, scan)
        }
        _ if actual == expected => Ok(()),
        _ => Err(mismatch(
            path,
            &format!("type {actual} differs from {expected}"),
        )),
    }
}

fn mismatch(path: &str, reason: &str) -> SchemaError {
    SchemaError::InvalidDeclaration {
        context: path.into(),
        reason: reason.into(),
    }
}
