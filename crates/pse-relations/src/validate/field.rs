// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use crate::{RelationError, ext::ExtMetadata};
use arrow_schema::{DataType, Field};
use pse_ids::SemanticId;
use pse_schema::Registry;
use pse_schema::arrow::{
    KEY_ENUM, KEY_EXTENSION_METADATA, KEY_EXTENSION_NAME, KEY_FK, KEY_LOGICAL_TYPE,
    KEY_QUANTITY_TYPE, KEY_ROLE,
};
use pse_schema::model::{ColumnRole, EXTENSION_TYPES, FieldContract};

/// Recursively checks field declarations and invokes the typed extension factories.
/// Fields without semantic declarations may use any native Arrow layout;
/// relation admission separately requires their exact declared metadata.
///
/// # Errors
/// Malformed PSE extensions and unresolved or inconsistent domain declarations.
pub fn validate_field(reg: &Registry, field: &Field) -> Result<(), Vec<RelationError>> {
    let mut errors = Vec::new();
    visit(reg, field, field.name(), &mut errors);
    super::finish(errors)
}

fn visit(reg: &Registry, field: &Field, path: &str, errors: &mut Vec<RelationError>) {
    let declaration = FieldContract::from_field(field.clone());
    if let Err(error) = pse_schema::model::IntegerRange::from_field(field) {
        errors.push(error.into());
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
    for child in declaration.children() {
        visit(
            reg,
            child.field(),
            &format!("{path}.{}", child.name()),
            errors,
        );
    }
}

fn check_semantics(reg: &Registry, field: &Field, path: &str, errors: &mut Vec<RelationError>) {
    let metadata = field.metadata();
    if let Some(name) = metadata.get(KEY_LOGICAL_TYPE) {
        if let Some(logical) = reg.logical_type(name) {
            let expected =
                serde_json::from_str::<DataType>(&logical.arrow_storage)
                    .map_err(|error| error.to_string())
                    .and_then(|storage| {
                        if logical.extension_name.as_ref().is_some_and(|name| {
                            EXTENSION_TYPES.iter().any(|spec| spec.name == name)
                        }) {
                            Ok(storage)
                        } else {
                            pse_schema::arrow::bind_type(reg, &storage, path)
                                .map_err(|error| error.to_string())
                        }
                    });
            if expected.as_ref().ok() != Some(field.data_type()) {
                errors.push(super::mismatch(
                    path,
                    format!(
                        "logical-type declaration differs from storage: expected {expected:?}, actual {:?}",
                        field.data_type()
                    ),
                ));
            }
            if logical.extension_name.as_deref()
                != metadata.get(KEY_EXTENSION_NAME).map(String::as_str)
            {
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
    if !EXTENSION_TYPES.iter().any(|spec| spec.name == name) {
        if name.starts_with("pse.") {
            errors.push(super::mismatch(path, "unknown PSE domain extension"));
        }
        return;
    }
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
