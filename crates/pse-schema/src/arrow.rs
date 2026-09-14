// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The Arrow schema of a declared relation, metadata and all (blueprint §4.3).
//!
//! Metadata is attached **once, at construction**, and never patched afterwards. A field
//! that acquires its `ARROW:extension:name` later has, in between, been a field that
//! claimed nothing — and the plan that read it in between had no way to know. Nested
//! children carry their own metadata for the same reason: a `list<pse.semantic_id>` whose
//! child field is bare is a list of 16-byte blobs to every consumer that looks at the
//! child.
//!
//! Sorted insertion into a `HashMap` does not stabilize its iteration order, so canonical
//! hashing serializes the ordered metadata relation of §5.3 step 3 rather than an Arrow
//! map; these maps are ordinary Arrow field metadata and carry no ordering claim.

use std::collections::HashMap;

use arrow_schema::{Field, Fields, Schema};

use crate::builder::{Registry, quantity_type_id};
use crate::error::SchemaError;
use crate::ext_metadata;
use crate::model::{ColumnSpec, ExtensionUse, LogicalType, QuantityContract, RelationSpec};

/// `pse.contract.id`: the relation identity as 32 lowercase hexadecimal digits.
pub const KEY_CONTRACT_ID: &str = "pse.contract.id";
/// `pse.contract.version`: the relation's schema version.
pub const KEY_CONTRACT_VERSION: &str = "pse.contract.version";
/// `pse.contract.fingerprint`: the relation fingerprint as 64 lowercase hexadecimal digits.
pub const KEY_CONTRACT_FINGERPRINT: &str = "pse.contract.fingerprint";
/// `pse.namespace`: the relation's namespace.
pub const KEY_NAMESPACE: &str = "pse.namespace";
/// `pse.semantic.logical_type`: the registry logical-type name.
pub const KEY_LOGICAL_TYPE: &str = "pse.semantic.logical_type";
/// `pse.semantic.quantity_type`: the column's single quantity contract.
pub const KEY_QUANTITY_TYPE: &str = "pse.semantic.quantity_type";
/// `pse.semantic.role`: the column role.
pub const KEY_ROLE: &str = "pse.semantic.role";
/// `pse.semantic.fk`: `<relation>.<column>`.
pub const KEY_FK: &str = "pse.semantic.fk";
/// `pse.semantic.enum`: the enumeration identity as 32 lowercase hexadecimal digits.
pub const KEY_ENUM: &str = "pse.semantic.enum";
/// `ARROW:extension:name`, Arrow's canonical extension key.
pub const KEY_EXTENSION_NAME: &str = "ARROW:extension:name";
/// `ARROW:extension:metadata`, mandatory beside the name when the type has metadata.
pub const KEY_EXTENSION_METADATA: &str = "ARROW:extension:metadata";

/// The Arrow schema of `spec`, with its schema and field metadata (blueprint §4.3).
///
/// # Errors
///
/// [`SchemaError::UnknownReference`] when a column references an enumeration or an
/// ordinal-ref target the registry does not declare. Assembly rejects those already, so
/// reaching this error means the spec did not come from this registry.
pub fn relation_schema(reg: &Registry, spec: &RelationSpec) -> Result<Schema, SchemaError> {
    let mut fields = Vec::with_capacity(spec.columns.len());
    for column in &spec.columns {
        fields.push(field_for(reg, column)?);
    }
    let metadata = HashMap::from([
        (KEY_CONTRACT_ID.to_owned(), spec.id.to_hex()),
        (
            KEY_CONTRACT_VERSION.to_owned(),
            spec.key.version.to_string(),
        ),
        (
            KEY_CONTRACT_FINGERPRINT.to_owned(),
            spec.fingerprint.to_hex(),
        ),
        (
            KEY_NAMESPACE.to_owned(),
            spec.key.namespace.as_str().to_owned(),
        ),
    ]);
    Ok(Schema::new_with_metadata(fields, metadata))
}

/// The Arrow field of one declared column (blueprint §4.3).
///
/// # Errors
///
/// [`SchemaError::UnknownReference`] when the column references an enumeration or an
/// ordinal-ref target the registry does not declare.
pub fn field_for(reg: &Registry, col: &ColumnSpec) -> Result<Field, SchemaError> {
    let mut metadata = semantic_metadata(reg, &col.logical_type, col.name)?;
    metadata.insert(KEY_ROLE.to_owned(), col.role.as_str().to_owned());
    match col.quantity {
        QuantityContract::None | QuantityContract::PerRow => {}
        QuantityContract::Column(name) => {
            metadata.insert(
                KEY_QUANTITY_TYPE.to_owned(),
                quantity_type_id(name).to_hex(),
            );
        }
    }
    if let Some(fk) = col.fk {
        metadata.insert(KEY_FK.to_owned(), fk.to_string());
    }
    Ok(Field::new(
        col.name,
        data_type_for(reg, &col.logical_type, col.name)?,
        col.nullable,
    )
    .with_metadata(metadata))
}

/// The Arrow storage of a logical type, with its children's metadata attached.
///
/// # Errors
///
/// The errors of [`semantic_metadata`] for any nested extension use.
fn data_type_for(
    reg: &Registry,
    ty: &LogicalType,
    path: &str,
) -> Result<arrow_schema::DataType, SchemaError> {
    let data_type = match ty {
        LogicalType::List(element) => {
            arrow_schema::DataType::List(child_field(reg, element, &format!("{path}.item"))?.into())
        }
        LogicalType::FixedList(element, width) => arrow_schema::DataType::FixedSizeList(
            child_field(reg, element, &format!("{path}.item"))?.into(),
            *width,
        ),
        LogicalType::Struct(children) => {
            let mut fields = Vec::with_capacity(children.len());
            for (name, child_type, nullable) in children {
                let mut field = child_field(reg, child_type, &format!("{path}.{name}"))?;
                field = field.with_name(*name).with_nullable(*nullable);
                fields.push(field);
            }
            arrow_schema::DataType::Struct(Fields::from(fields))
        }
        other => other.data_type(),
    };
    Ok(data_type)
}

/// A nested child field, named `item` unless the caller renames it.
///
/// Children carry `pse.semantic.logical_type` and, for an extension use, the Arrow
/// extension keys — but never a role, a foreign key or a quantity contract, which are
/// statements about a *column* and would be a second, weaker claim on a child.
///
/// # Errors
///
/// The errors of [`semantic_metadata`].
fn child_field(reg: &Registry, ty: &LogicalType, path: &str) -> Result<Field, SchemaError> {
    let metadata = semantic_metadata(reg, ty, path)?;
    Ok(Field::new("item", data_type_for(reg, ty, path)?, false).with_metadata(metadata))
}

/// The `pse.semantic.*` and `ARROW:extension:*` metadata of one logical type.
///
/// # Errors
///
/// [`SchemaError::UnknownReference`] when an enumeration or ordinal-ref target does not
/// resolve.
fn semantic_metadata(
    reg: &Registry,
    ty: &LogicalType,
    path: &str,
) -> Result<HashMap<String, String>, SchemaError> {
    let mut metadata = HashMap::from([(KEY_LOGICAL_TYPE.to_owned(), ty.name())]);
    let Some(use_) = ty.extension() else {
        return Ok(metadata);
    };
    let spec = use_.spec();
    let id = match use_ {
        ExtensionUse::Enum(name) => {
            let enum_spec = reg
                .enum_spec(name)
                .ok_or_else(|| SchemaError::UnknownReference {
                    context: format!("field {path}"),
                    reference: format!("enum:{name}"),
                })?;
            metadata.insert(KEY_ENUM.to_owned(), enum_spec.id.to_hex());
            Some(enum_spec.id)
        }
        ExtensionUse::OrdinalRef { target } => {
            let relation = reg
                .relation(target)
                .ok_or_else(|| SchemaError::UnknownReference {
                    context: format!("field {path}"),
                    reference: (*target).to_owned(),
                })?;
            Some(relation.id)
        }
        _ => None,
    };
    metadata.insert(KEY_EXTENSION_NAME.to_owned(), spec.name.to_owned());
    metadata.insert(
        KEY_EXTENSION_METADATA.to_owned(),
        ext_metadata::canonical(spec.metadata, spec.metadata_version, id),
    );
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;

    #[test]
    fn the_schema_carries_the_four_contract_keys() {
        let reg = registry().expect("the registry assembles");
        let spec = reg
            .relation("reference.schema_relations")
            .expect("declared in §4.1");
        let schema = relation_schema(reg, spec).expect("the schema builds");
        let metadata = schema.metadata();
        assert_eq!(metadata.get(KEY_CONTRACT_ID), Some(&spec.id.to_hex()));
        assert_eq!(metadata.get(KEY_CONTRACT_VERSION), Some(&"1".to_owned()));
        assert_eq!(
            metadata.get(KEY_CONTRACT_FINGERPRINT),
            Some(&spec.fingerprint.to_hex())
        );
        assert_eq!(metadata.get(KEY_NAMESPACE), Some(&"reference".to_owned()));
        assert_eq!(metadata.len(), 4, "no undeclared schema metadata key");
    }

    #[test]
    fn an_enum_field_names_its_enumeration_in_both_places() {
        let reg = registry().expect("the registry assembles");
        let spec = reg
            .relation("reference.schema_relations")
            .expect("declared in §4.1");
        let schema = relation_schema(reg, spec).expect("the schema builds");
        let field = schema.field_with_name("namespace").expect("declared");
        let enum_id = reg.enum_spec("Namespace").expect("declared").id;
        assert_eq!(
            field.metadata().get(KEY_LOGICAL_TYPE),
            Some(&"enum:Namespace".to_owned())
        );
        assert_eq!(field.metadata().get(KEY_ENUM), Some(&enum_id.to_hex()));
        assert_eq!(
            field.metadata().get(KEY_EXTENSION_NAME),
            Some(&"pse.enum".to_owned())
        );
        assert_eq!(
            field.metadata().get(KEY_EXTENSION_METADATA),
            Some(&format!("{{\"v\":1,\"enum_id\":\"{}\"}}", enum_id.to_hex()))
        );
    }

    #[test]
    fn a_foreign_key_column_names_its_target() {
        let reg = registry().expect("the registry assembles");
        let spec = reg
            .relation("reference.schema_columns")
            .expect("declared in §4.1");
        let schema = relation_schema(reg, spec).expect("the schema builds");
        let field = schema.field_with_name("relation_id").expect("declared");
        assert_eq!(
            field.metadata().get(KEY_FK),
            Some(&"reference.schema_relations.relation_id".to_owned())
        );
        assert_eq!(
            field.metadata().get(KEY_EXTENSION_NAME),
            Some(&"pse.semantic_id".to_owned())
        );
    }

    #[test]
    fn a_list_child_carries_its_own_logical_type() {
        let reg = registry().expect("the registry assembles");
        let spec = reg
            .relation("reference.schema_relations")
            .expect("declared in §4.1");
        let schema = relation_schema(reg, spec).expect("the schema builds");
        let field = schema.field_with_name("primary_key").expect("declared");
        let arrow_schema::DataType::List(child) = field.data_type() else {
            panic!("primary_key is declared as a list");
        };
        assert_eq!(
            child.metadata().get(KEY_LOGICAL_TYPE),
            Some(&"text".to_owned())
        );
    }
}
