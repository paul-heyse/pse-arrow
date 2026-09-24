// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native field traversal and explicit projections for three distinct equivalence questions.
use arrow_schema::{ArrowError, DataType, Field, FieldRef, UnionFields};
use std::sync::Arc;

/// Human-readable field prose, deliberately excluded from execution/value identity.
pub const DOCUMENTATION: &str = "pse.domain.doc";
/// What an observation or comparison is intended to establish.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataPurpose {
    /// Exact physical metadata, names, nullability and storage remain observable.
    PhysicalObservation,
    /// Exclude only declared prose; unknown metadata remains semantic.
    ExecutionIdentity,
    /// Value domains matter; root aliases and reference/role usage do not.
    ValueIdentity,
}
/// Borrow native child fields in their declared order, including dictionary value children.
pub fn children(kind: &DataType) -> Vec<&Field> {
    match kind {
        DataType::List(child)
        | DataType::LargeList(child)
        | DataType::ListView(child)
        | DataType::LargeListView(child)
        | DataType::FixedSizeList(child, _)
        | DataType::Map(child, _) => vec![child],
        DataType::Struct(fields) => fields.iter().map(AsRef::as_ref).collect(),
        DataType::Union(fields, _) => fields.iter().map(|(_, field)| field.as_ref()).collect(),
        DataType::Dictionary(_, value) => children(value),
        DataType::RunEndEncoded(runs, values) => vec![runs, values],
        _ => vec![],
    }
}
/// Transform each real native field, preserving native containers and their properties.
/// # Errors
/// An invalid union field mapping. No partially transformed field is returned.
pub fn map(
    field: &Field,
    transform: &mut impl FnMut(&Field) -> Field,
) -> Result<Field, ArrowError> {
    let transformed = transform(field);
    Ok(transformed
        .clone()
        .with_data_type(map_type(transformed.data_type(), transform)?))
}
fn map_type(
    kind: &DataType,
    transform: &mut impl FnMut(&Field) -> Field,
) -> Result<DataType, ArrowError> {
    let mut child = |field: &FieldRef| map(field, transform).map(Arc::new);
    Ok(match kind {
        DataType::List(field) => DataType::List(child(field)?),
        DataType::LargeList(field) => DataType::LargeList(child(field)?),
        DataType::ListView(field) => DataType::ListView(child(field)?),
        DataType::LargeListView(field) => DataType::LargeListView(child(field)?),
        DataType::FixedSizeList(field, width) => DataType::FixedSizeList(child(field)?, *width),
        DataType::Map(field, sorted) => DataType::Map(child(field)?, *sorted),
        DataType::Struct(fields) => DataType::Struct(
            fields
                .iter()
                .map(child)
                .collect::<Result<Vec<_>, _>>()?
                .into(),
        ),
        DataType::Union(fields, mode) => DataType::Union(
            UnionFields::try_new(
                fields.iter().map(|(id, _)| id),
                fields
                    .iter()
                    .map(|(_, field)| child(field))
                    .collect::<Result<Vec<_>, _>>()?,
            )?,
            *mode,
        ),
        DataType::Dictionary(key, value) => {
            DataType::Dictionary(key.clone(), Box::new(map_type(value, transform)?))
        }
        DataType::RunEndEncoded(runs, values) => {
            DataType::RunEndEncoded(child(runs)?, child(values)?)
        }
        leaf => leaf.clone(),
    })
}
/// Project metadata under a named equivalence policy. Unrecognized keys are always retained.
/// # Errors
/// An invalid native field tree.
pub fn project(field: &Field, purpose: MetadataPurpose) -> Result<Field, ArrowError> {
    let mut field = map(field, &mut |field| {
        let mut field = field.clone();
        field.metadata_mut().retain(|key, _| match purpose {
            MetadataPurpose::PhysicalObservation => true,
            MetadataPurpose::ExecutionIdentity => key != DOCUMENTATION,
            MetadataPurpose::ValueIdentity => !matches!(
                key.as_str(),
                DOCUMENTATION
                    | "pse.domain.role"
                    | "pse.semantic.role"
                    | "pse.domain.fk.relation"
                    | "pse.domain.fk.column"
                    | "pse.semantic.fk"
                    | "pse.semantic.reference"
            ),
        });
        field
    })?;
    if purpose == MetadataPurpose::ValueIdentity {
        field = field.with_name("item").with_nullable(false);
    }
    Ok(field)
}

/// Deterministically encode a native declaration, including unknown metadata.
/// # Errors
/// Native serialization fails.
pub fn canonical_json(value: &impl serde::Serialize) -> Result<String, serde_json::Error> {
    let mut value = serde_json::to_value(value)?;
    value.sort_all_objects();
    serde_json::to_string(&value)
}
