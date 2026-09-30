// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native field traversal and schema-declared projections for distinct contract questions.
use arrow_schema::{ArrowError, DataType, Field, FieldRef, UnionFields};
use crate::generated::field_facets::{Admission, FACETS};
use std::sync::Arc;

pub use crate::generated::field_facets::{DOC as DOCUMENTATION, STRUCTURE as STRUCTURE_NAME};
/// What an observation or comparison is intended to establish.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataPurpose {
    /// Exact physical metadata, names, nullability and storage remain observable.
    PhysicalObservation,
    /// Exclude only declared prose; unknown metadata remains semantic.
    ExecutionIdentity,
    /// Value domains matter; root aliases and reference/role usage do not.
    ValueIdentity,
    /// Root storage contract, excluding declared root usage/quantity/identity facets.
    /// Nested fields remain exact. This does not establish physical compatibility.
    LogicalStorageType,
    /// Logical storage identity additionally omits structure presentation at every depth.
    LogicalTypeIdentity,
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
    let mut root = true;
    map(field, &mut |field| {
        let projected = project_at(field, purpose, root);
        root = false;
        projected
    })
}

/// Project only the root storage contract, preserving the existing nested declaration.
pub fn logical_storage_type(field: &Field) -> Field {
    project_at(field, MetadataPurpose::LogicalStorageType, true)
}

fn project_at(field: &Field, purpose: MetadataPurpose, root: bool) -> Field {
    let mut field = field.clone();
    field.metadata_mut().retain(|key, _| {
        let Some(facet) = FACETS.iter().find(|facet| facet.key == key) else {
            return true;
        };
        match purpose {
            MetadataPurpose::PhysicalObservation => true,
            MetadataPurpose::ExecutionIdentity => facet.execution,
            MetadataPurpose::ValueIdentity => facet.value,
            MetadataPurpose::LogicalStorageType => !root || facet.storage_root,
            MetadataPurpose::LogicalTypeIdentity => facet.type_identity && (!root || facet.storage_root),
        }
    });
    if root && matches!(purpose, MetadataPurpose::ValueIdentity | MetadataPurpose::LogicalStorageType | MetadataPurpose::LogicalTypeIdentity) {
        field = field.with_name("item").with_nullable(false);
    }
    field
}

/// Directional metadata predicates, separate from equality and storage projections.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataAdmission {
    /// A declared target checks values; established or unclassified meaning cannot change.
    CheckedTarget,
    /// The expected field was already established by the input expression. Only missing
    /// annotations may be restored; even presentation conflicts must be refused.
    RestoreEstablished,
}

/// Check one field's metadata under the selected directional boundary predicate.
/// This establishes no storage, nullability, value-domain or physical conversion proof.
pub fn admits_metadata(
    source: &std::collections::HashMap<String, String>,
    target: &std::collections::HashMap<String, String>,
    purpose: MetadataAdmission,
) -> bool {
    if purpose == MetadataAdmission::RestoreEstablished {
        return source.iter().all(|(key, value)| target.get(key) == Some(value));
    }
    source.keys().chain(target.keys()).all(|key| {
        let admission = FACETS.iter().find(|facet| facet.key == key)
            .map_or(Admission::Exact, |facet| facet.admission);
        match admission {
            Admission::Ignore => true,
            Admission::Exact => source.get(key) == target.get(key),
            Admission::Preserve => source.get(key).is_none_or(|value| target.get(key) == Some(value)),
        }
    })
}

/// Deterministically encode a native declaration, including unknown metadata.
/// # Errors
/// Native serialization fails.
pub fn canonical_json(value: &impl serde::Serialize) -> Result<String, serde_json::Error> {
    let mut value = serde_json::to_value(value)?;
    value.sort_all_objects();
    serde_json::to_string(&value)
}

#[cfg(test)]
mod tests;
