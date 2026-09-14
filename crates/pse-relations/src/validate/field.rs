// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use crate::{RelationError, ext::ExtMetadata};
use arrow_schema::{DataType, Field, TimeUnit};
use pse_ids::SemanticId;
use pse_schema::Registry;
use pse_schema::arrow::{
    KEY_ENUM, KEY_EXTENSION_METADATA, KEY_EXTENSION_NAME, KEY_FK, KEY_LOGICAL_TYPE,
    KEY_QUANTITY_TYPE, KEY_ROLE,
};
use pse_schema::model::{ColumnRole, EXTENSION_TYPES, render_data_type};

/// Recursively checks field declarations and invokes the typed extension factories.
/// Fields without semantic declarations may use ordinary supported Arrow scalar layouts;
/// relation admission separately requires their exact declared metadata.
///
/// # Errors
/// Unknown metadata, unsupported storage, malformed extensions and unresolved declarations.
pub fn validate_field(reg: &Registry, field: &Field) -> Result<(), Vec<RelationError>> {
    let mut errors = Vec::new();
    visit(reg, field, field.name(), false, &mut errors);
    super::finish(errors)
}

fn visit(
    reg: &Registry,
    field: &Field,
    path: &str,
    inside_extension: bool,
    errors: &mut Vec<RelationError>,
) {
    for key in field.metadata().keys() {
        if !matches!(
            key.as_str(),
            KEY_LOGICAL_TYPE
                | KEY_QUANTITY_TYPE
                | KEY_ROLE
                | KEY_FK
                | KEY_ENUM
                | KEY_EXTENSION_NAME
                | KEY_EXTENSION_METADATA
        ) {
            errors.push(RelationError::UnknownMetadata {
                field: path.to_owned(),
                key: key.clone(),
            });
        }
    }
    check_semantics(reg, field, path, errors);
    let extension = field.metadata().get(KEY_EXTENSION_NAME);
    if let Some(name) = extension {
        check_extension(reg, field, path, name, errors);
    } else if field.metadata().contains_key(KEY_EXTENSION_METADATA)
        || field.metadata().contains_key(KEY_ENUM)
    {
        errors.push(super::mismatch(
            path,
            "extension metadata or enum identity without an extension name",
        ));
    }
    match field.data_type() {
        DataType::Struct(children) => {
            for child in children {
                visit(
                    reg,
                    child,
                    &format!("{path}.{}", child.name()),
                    inside_extension || extension.is_some(),
                    errors,
                );
            }
        }
        DataType::List(child) => visit(
            reg,
            child,
            &format!("{path}.item"),
            inside_extension || extension.is_some(),
            errors,
        ),
        DataType::FixedSizeList(child, width) if *width > 0 => visit(
            reg,
            child,
            &format!("{path}.item"),
            inside_extension || extension.is_some(),
            errors,
        ),
        DataType::Dictionary(key, value)
            if value.as_ref() == &DataType::Utf8
                && ((key.as_ref() == &DataType::Int32 && extension.is_some())
                    || (key.as_ref() == &DataType::Int8 && inside_extension)) => {}
        DataType::Boolean
        | DataType::Float64
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64
        | DataType::Utf8 => {}
        DataType::Int16 if inside_extension => {}
        DataType::FixedSizeBinary(16 | 32) if inside_extension || extension.is_some() => {}
        DataType::Timestamp(TimeUnit::Nanosecond, zone) if zone.as_deref() == Some("UTC") => {}
        other => errors.push(super::mismatch(
            path,
            format!("unsupported declared layout {other}"),
        )),
    }
}

fn check_semantics(reg: &Registry, field: &Field, path: &str, errors: &mut Vec<RelationError>) {
    let metadata = field.metadata();
    if let Some(name) = metadata.get(KEY_LOGICAL_TYPE) {
        if let Some(logical) = reg.logical_type(name) {
            match render_data_type(field.data_type()) {
                Ok(actual) if actual == logical.arrow_storage => {}
                _ => errors.push(super::mismatch(
                    path,
                    "logical-type declaration differs from storage",
                )),
            }
            if logical.extension_name != metadata.get(KEY_EXTENSION_NAME).map(String::as_str) {
                errors.push(super::mismatch(
                    path,
                    "logical type and extension name disagree",
                ));
            }
        } else {
            errors.push(super::mismatch(
                path,
                format!("logical type {name} is not registered"),
            ));
        }
    }
    if let Some(name) = metadata.get(KEY_LOGICAL_TYPE) {
        if let Some(enumeration) = name
            .strip_prefix("enum:")
            .and_then(|name| reg.enum_spec(name))
            && metadata.get(KEY_ENUM) != Some(&enumeration.id.to_hex())
        {
            errors.push(super::mismatch(
                path,
                "logical enum name and declared identity disagree",
            ));
        }
        if let Some(target) = name
            .strip_prefix("ordinal_ref:")
            .and_then(|name| reg.relation(name))
        {
            let matches = metadata
                .get(KEY_EXTENSION_METADATA)
                .and_then(|text| ExtMetadata::parse(&EXTENSION_TYPES[6], text).ok())
                .is_some_and(|value| value.target_relation_id == Some(target.id));
            if !matches {
                errors.push(super::mismatch(
                    path,
                    "logical ordinal target and declared identity disagree",
                ));
            }
        }
    }
    if let Some(role) = metadata.get(KEY_ROLE)
        && !ColumnRole::ALL
            .iter()
            .any(|candidate| candidate.as_str() == role)
    {
        errors.push(super::mismatch(path, format!("unknown column role {role}")));
    }
    if let Some(id) = metadata.get(KEY_QUANTITY_TYPE)
        && SemanticId::parse_hex(id).map_or(true, |parsed| parsed.to_hex() != *id)
    {
        errors.push(super::mismatch(path, "quantity identity is malformed"));
    }
    if let Some(fk) = metadata.get(KEY_FK) {
        let resolved = fk.rsplit_once('.').and_then(|(relation, column)| {
            reg.relation(relation).and_then(|spec| spec.column(column))
        });
        if resolved.is_none() {
            errors.push(super::mismatch(
                path,
                format!("foreign key declaration {fk} does not resolve"),
            ));
        }
    }
}

fn check_extension(
    reg: &Registry,
    field: &Field,
    path: &str,
    name: &str,
    errors: &mut Vec<RelationError>,
) {
    if let Err(error) = crate::ext::validate_extension(field) {
        errors.push(RelationError::ExtensionType {
            field: path.to_owned(),
            name: name.to_owned(),
            reason: error.to_string(),
        });
        return;
    }
    let Some(spec) = EXTENSION_TYPES.iter().find(|spec| spec.name == name) else {
        return;
    };
    let Some(text) = field.metadata().get(KEY_EXTENSION_METADATA) else {
        return;
    };
    let Ok(metadata) = ExtMetadata::parse(spec, text) else {
        return;
    };
    if metadata.canonical_json(spec) != *text {
        errors.push(super::mismatch(
            path,
            "extension metadata must use its canonical declared spelling",
        ));
    }
    if let Some(id) = metadata.enum_id {
        if !reg.enums().iter().any(|enumeration| enumeration.id == id) {
            errors.push(RelationError::UnknownRegistry {
                relation: format!("enum:{id}"),
            });
        }
        if field.metadata().get(KEY_ENUM) != Some(&id.to_hex()) {
            errors.push(super::mismatch(
                path,
                "semantic enum identity differs from extension identity",
            ));
        }
    } else if field.metadata().contains_key(KEY_ENUM) {
        errors.push(super::mismatch(
            path,
            "enum identity is present on a non-enum extension",
        ));
    }
    if let Some(id) = metadata.target_relation_id
        && reg.relation_by_id(id).is_none()
    {
        errors.push(RelationError::UnknownRegistry {
            relation: format!("ordinal target:{id}"),
        });
    }
}
