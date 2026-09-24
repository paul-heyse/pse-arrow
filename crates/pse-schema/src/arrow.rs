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

use arrow_schema::{DataType, Field, Schema};

use crate::builder::{Registry, quantity_type_id};
use crate::error::SchemaError;
use crate::ext_metadata;
use crate::model::{ExtensionUse, FieldContract, QuantityContract, RelationSpec};

/// `pse.contract.id`: the relation identity as 32 lowercase hexadecimal digits.
pub const KEY_CONTRACT_ID: &str = "pse.contract.id";
/// `pse.contract.version`: the relation's schema version.
pub const KEY_CONTRACT_VERSION: &str = "pse.contract.version";
/// `pse.contract.fingerprint`: the relation fingerprint as 64 lowercase hexadecimal digits.
pub const KEY_CONTRACT_FINGERPRINT: &str = "pse.contract.fingerprint";
/// `pse.namespace`: the relation's namespace.
pub const KEY_NAMESPACE: &str = "pse.namespace";
/// Named native SQL predicates, encoded as a canonical JSON object in name order.
pub const KEY_CHECKS: &str = "pse.contract.checks";
/// Canonical native Delta policies, reflected in the relation fingerprint.
pub const KEY_DELTA_PROPERTIES: &str = "pse.contract.delta_properties";
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
/// Versioned native row-key encoding, distinct from an ordinary content hash.
pub const KEY_ROW_KEY_ENCODING: &str = "pse.semantic.key_encoding";
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
    Ok(relation_schema_ref(reg, spec)?.as_ref().clone())
}

/// Borrow the immutable registry's exact Arrow schema without serializing metadata.
/// # Errors
/// The declaration is foreign or does not match its complete resolved contract.
pub fn relation_schema_ref(
    reg: &Registry,
    spec: &RelationSpec,
) -> Result<arrow_schema::SchemaRef, SchemaError> {
    if reg.contracts_ready() {
        return Ok(reg.contract(spec)?.schema().clone());
    }
    // Bootstrap assembly precedes the frozen contract arena.
    uncached_relation_schema(reg, spec).map(std::sync::Arc::new)
}

pub(crate) fn uncached_relation_schema(
    reg: &Registry,
    spec: &RelationSpec,
) -> Result<Schema, SchemaError> {
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
        (KEY_CHECKS.to_owned(), checks_json(&spec.checks)?),
        (
            KEY_DELTA_PROPERTIES.to_owned(),
            serde_json::to_string(&spec.delta_properties)
                .map_err(|error| crate::checks::invalid(KEY_DELTA_PROPERTIES, error.to_string()))?,
        ),
    ]);
    Ok(Schema::new_with_metadata(fields, metadata))
}

/// Decode the exact native row-check declarations from an Arrow schema.
/// # Errors
/// Malformed or noncanonical metadata is refused.
pub fn native_checks(
    schema: &Schema,
) -> Result<std::collections::BTreeMap<String, String>, SchemaError> {
    let Some(text) = schema.metadata().get(KEY_CHECKS) else {
        return Ok(std::collections::BTreeMap::new());
    };
    let checks = serde_json::from_str(text)
        .map_err(|error| crate::checks::invalid(KEY_CHECKS, error.to_string()))?;
    if checks_json(&checks)? != *text
        || checks
            .iter()
            .any(|(name, sql)| name.is_empty() || sql.trim().is_empty())
    {
        return Err(crate::checks::invalid(
            KEY_CHECKS,
            "invalid or noncanonical native checks",
        ));
    }
    Ok(checks)
}

fn checks_json(checks: &std::collections::BTreeMap<String, String>) -> Result<String, SchemaError> {
    serde_json::to_string(checks)
        .map_err(|error| crate::checks::invalid(KEY_CHECKS, error.to_string()))
}

/// Read native table policy without requiring a live registry.
/// # Errors
/// Missing, malformed, noncanonical or reserved property declarations.
pub fn delta_properties(
    schema: &Schema,
) -> Result<std::collections::BTreeMap<String, String>, SchemaError> {
    let invalid = |reason: String| crate::checks::invalid(KEY_DELTA_PROPERTIES, reason);
    let text = schema
        .metadata()
        .get(KEY_DELTA_PROPERTIES)
        .ok_or_else(|| invalid("missing native table policy".into()))?;
    let properties: std::collections::BTreeMap<String, String> =
        serde_json::from_str(text).map_err(|error| invalid(error.to_string()))?;
    if serde_json::to_string(&properties).map_err(|error| invalid(error.to_string()))? != *text {
        return Err(invalid("noncanonical native table policy".into()));
    }
    validate_delta_properties(&properties)?;
    Ok(properties)
}

pub(crate) fn validate_delta_properties(
    properties: &std::collections::BTreeMap<String, String>,
) -> Result<(), SchemaError> {
    if let Some(interval) = properties.get("delta.checkpointInterval")
        && interval.parse::<std::num::NonZeroU64>().is_err()
    {
        return Err(crate::checks::invalid(
            KEY_DELTA_PROPERTIES,
            "checkpoint interval must be a positive native commit count",
        ));
    }
    if properties.iter().any(|(key, value)| {
        !key.starts_with("delta.")
            || key.starts_with("delta.constraints.")
            || value.trim().is_empty()
    }) {
        return Err(crate::checks::invalid(
            KEY_DELTA_PROPERTIES,
            "table policies must be nonempty native Delta properties; CHECK constraints are declared separately",
        ));
    }
    Ok(())
}

/// The Arrow field of one declared column (blueprint §4.3).
///
/// # Errors
///
/// [`SchemaError::UnknownReference`] when the column references an enumeration or an
/// ordinal-ref target the registry does not declare.
pub fn field_for(reg: &Registry, col: &FieldContract) -> Result<Field, SchemaError> {
    bind_field(reg, col.field(), col.name())
}

fn bind_field(reg: &Registry, field: &Field, path: &str) -> Result<Field, SchemaError> {
    let contract = FieldContract::from_field(field.clone());
    let mut metadata = field.metadata().clone();
    if contract.extension().is_none()
        && metadata
            .get(KEY_EXTENSION_NAME)
            .is_some_and(|name| name.starts_with("pse."))
    {
        return Err(crate::checks::invalid(
            path,
            "PSE extension requires a domain declaration",
        ));
    }
    metadata.retain(|key, _| !key.starts_with("pse.domain."));
    let mut semantic = semantic_metadata(reg, &contract, path)?;
    semantic.insert(KEY_ROLE.to_owned(), contract.role().as_str().to_owned());
    if let QuantityContract::Column(name) = contract.quantity() {
        semantic.insert(
            KEY_QUANTITY_TYPE.to_owned(),
            quantity_type_id(name).to_hex(),
        );
    }
    if let Some(fk) = contract.fk() {
        semantic.insert(KEY_FK.to_owned(), fk.to_string());
    }
    for (key, value) in &semantic {
        if metadata.get(key).is_some_and(|existing| existing != value) {
            return Err(crate::checks::invalid(
                path,
                format!("native metadata disagrees with derived {key}"),
            ));
        }
    }
    for key in [KEY_ENUM, KEY_QUANTITY_TYPE, KEY_FK] {
        if metadata.contains_key(key) && !semantic.contains_key(key) {
            return Err(crate::checks::invalid(
                path,
                format!("derived {key} requires a domain declaration"),
            ));
        }
    }
    metadata.extend(semantic);
    Ok(field
        .clone()
        .with_data_type(if contract.extension().is_some() {
            field.data_type().clone()
        } else {
            bind_type(reg, field.data_type(), path)?
        })
        .with_metadata(metadata))
}

/// Bind the domain annotations on the child fields of a native physical type.
///
/// # Errors
/// An enum or ordinal domain does not resolve in the registry.
pub fn bind_type(reg: &Registry, ty: &DataType, path: &str) -> Result<DataType, SchemaError> {
    let child = |f: &std::sync::Arc<Field>| {
        bind_field(reg, f, &format!("{path}.{}", f.name())).map(std::sync::Arc::new)
    };
    Ok(match ty {
        DataType::List(f) => DataType::List(child(f)?),
        DataType::LargeList(f) => DataType::LargeList(child(f)?),
        DataType::ListView(f) => DataType::ListView(child(f)?),
        DataType::LargeListView(f) => DataType::LargeListView(child(f)?),
        DataType::FixedSizeList(f, n) => DataType::FixedSizeList(child(f)?, *n),
        DataType::Map(f, sorted) => DataType::Map(child(f)?, *sorted),
        DataType::Struct(fields) => DataType::Struct(
            fields
                .iter()
                .map(child)
                .collect::<Result<Vec<_>, _>>()?
                .into(),
        ),
        DataType::Union(fields, mode) => DataType::Union(
            arrow_schema::UnionFields::try_new(
                fields.iter().map(|(id, _)| id),
                fields
                    .iter()
                    .map(|(_, f)| child(f))
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .map_err(|e| crate::checks::invalid(path, e.to_string()))?,
            *mode,
        ),
        DataType::Dictionary(key, value) => {
            DataType::Dictionary(key.clone(), Box::new(bind_type(reg, value, path)?))
        }
        DataType::RunEndEncoded(run, values) => {
            DataType::RunEndEncoded(child(run)?, child(values)?)
        }
        other => other.clone(),
    })
}

/// The `pse.semantic.*` and `ARROW:extension:*` metadata of one logical type.
///
/// # Errors
///
/// [`SchemaError::UnknownReference`] when an enumeration or ordinal-ref target does not
/// resolve.
fn semantic_metadata(
    reg: &Registry,
    ty: &FieldContract,
    path: &str,
) -> Result<HashMap<String, String>, SchemaError> {
    let name = ty.type_name()?;
    // Native expressions can construct arbitrary Arrow shapes. A registry alias
    // is useful only when that alias is actually declared; the native field and
    // its recursively bound child descriptors otherwise carry the full type.
    let mut metadata = if ty.extension().is_some() || reg.logical_type(&name).is_some() {
        HashMap::from([(KEY_LOGICAL_TYPE.to_owned(), name)])
    } else {
        HashMap::new()
    };
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
            return Ok(enum_metadata(name, enum_spec.id));
        }
        ExtensionUse::OrdinalRef { target } => {
            let relation = reg
                .relation(target)
                .ok_or_else(|| SchemaError::UnknownReference {
                    context: format!("field {path}"),
                    reference: target.to_owned(),
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

/// The common metadata projection for a declared string enumeration, including
/// enum children owned by composite extension storage.
pub(crate) fn enum_metadata(name: &str, id: pse_ids::SemanticId) -> HashMap<String, String> {
    let spec = ExtensionUse::Enum(name).spec();
    HashMap::from([
        (KEY_LOGICAL_TYPE.to_owned(), format!("enum:{name}")),
        (KEY_ENUM.to_owned(), id.to_hex()),
        (KEY_EXTENSION_NAME.to_owned(), spec.name.to_owned()),
        (
            KEY_EXTENSION_METADATA.to_owned(),
            ext_metadata::canonical(spec.metadata, spec.metadata_version, Some(id)),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;

    #[test]
    fn the_schema_carries_its_declared_contract_keys() {
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
        assert_eq!(metadata.get(KEY_CHECKS), Some(&"{}".to_owned()));
        assert_eq!(
            metadata.get(KEY_DELTA_PROPERTIES),
            Some(&serde_json::to_string(&spec.delta_properties).unwrap())
        );
        assert_eq!(metadata.len(), 6, "no undeclared schema metadata key");
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
        let DataType::List(child) = field.data_type() else {
            panic!("primary_key is declared as a list");
        };
        assert_eq!(
            child.metadata().get(KEY_LOGICAL_TYPE),
            Some(&"text".to_owned())
        );
    }
}
