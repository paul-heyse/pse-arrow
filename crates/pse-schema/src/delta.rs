// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Durable layouts derived from the same declarations as execution schemas.
use crate::{Registry, SchemaError, model::RelationSpec};
use arrow_schema::{DataType, Field, Schema, TimeUnit};

/// Version of the named, lossless execution-to-Delta mapping.
pub const KEY_LAYOUT_VERSION: &str = "pse.layout.version";
/// Complete native execution field descriptor, including nested domain annotations.
/// This is descriptive JSON, never row identity or an execution language.
pub const KEY_EXECUTION_FIELD: &str = "pse.layout.execution_field";
/// Original schema annotations, kept separate from each field's metadata scope.
pub const KEY_SCHEMA_METADATA: &str = "pse.layout.schema_metadata";
/// The current development layout; historical layouts have no compatibility reader.
pub const LAYOUT_VERSION: &str = "1";

/// Generate a lossless Delta-compatible schema from a relation declaration.
/// Semantic annotations and the execution descriptor accompany the physical layout.
/// An extension descriptor is retained only when its storage is unchanged.
/// # Errors
/// A declared type has no implemented lossless durable mapping.
pub fn relation_schema(
    registry: &Registry,
    relation: &RelationSpec,
) -> Result<Schema, SchemaError> {
    storage_schema(&crate::arrow::relation_schema(registry, relation)?)
}

/// Derive a durable layout for an explicitly supplied execution schema.
/// # Errors
/// Unsupported types require an explicit new mapping, never a lossy implicit cast.
pub fn storage_schema(schema: &Schema) -> Result<Schema, SchemaError> {
    crate::field_contract::declaration(schema)?;
    reject_layout_metadata(schema.metadata())?;
    let schema_metadata = descriptor(schema.metadata())?;
    let fields = schema
        .fields()
        .iter()
        .map(|f| {
            let mut field = storage_field(f)?;
            field
                .metadata_mut()
                .insert(KEY_SCHEMA_METADATA.into(), schema_metadata.clone());
            Ok(field)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Schema::new_with_metadata(fields, schema.metadata().clone()))
}

/// Reconstruct a native execution declaration from a self-describing durable schema.
/// No registry or domain UDF is needed to read the declaration. Its physical shape
/// and descriptors are verified together; this does not establish value invariants.
/// # Errors
/// Missing/unknown layout, inconsistent annotations, or a false physical declaration.
pub fn execution_schema(storage: &Schema) -> Result<Schema, SchemaError> {
    let mut metadata = None;
    let mut fields = Vec::with_capacity(storage.fields().len());
    for field in storage.fields() {
        let get = |key: &str| {
            field
                .metadata()
                .get(key)
                .ok_or_else(|| SchemaError::InvalidDeclaration {
                    context: field.name().to_owned(),
                    reason: format!("missing durable descriptor {key}"),
                })
        };
        if get(KEY_LAYOUT_VERSION)? != LAYOUT_VERSION {
            return Err(SchemaError::InvalidDeclaration {
                context: field.name().to_owned(),
                reason: "unknown durable layout version".into(),
            });
        }
        let current: std::collections::HashMap<String, String> =
            serde_json::from_str(get(KEY_SCHEMA_METADATA)?)
                .map_err(|error| descriptor_error(&error))?;
        if metadata
            .as_ref()
            .is_some_and(|expected| expected != &current)
        {
            return Err(SchemaError::InvalidDeclaration {
                context: field.name().to_owned(),
                reason: "inconsistent durable schema annotations".into(),
            });
        }
        metadata = Some(current);
        fields.push(
            serde_json::from_str::<Field>(get(KEY_EXECUTION_FIELD)?)
                .map_err(|error| descriptor_error(&error))?,
        );
    }
    let metadata = metadata.ok_or_else(|| SchemaError::InvalidDeclaration {
        context: "Delta schema".into(),
        reason: "no fields carrying a durable declaration".into(),
    })?;
    let execution = Schema::new_with_metadata(fields, metadata);
    crate::field_contract::delta_scan_schema(storage, &storage_schema(&execution)?)?;
    Ok(execution)
}

fn reject_layout_metadata(
    metadata: &std::collections::HashMap<String, String>,
) -> Result<(), SchemaError> {
    if metadata.keys().any(|key| key.starts_with("pse.layout.")) {
        return Err(SchemaError::InvalidDeclaration {
            context: "execution declaration".into(),
            reason: "durable descriptors cannot be supplied as execution declarations".into(),
        });
    }
    Ok(())
}

fn storage_field(field: &Field) -> Result<Field, SchemaError> {
    reject_layout_metadata(field.metadata())?;
    let data_type = storage_type(field.data_type())?;
    let mut metadata = field.metadata().clone();
    if &data_type != field.data_type() {
        metadata.remove(crate::arrow::KEY_EXTENSION_NAME);
        metadata.remove(crate::arrow::KEY_EXTENSION_METADATA);
        metadata.retain(|key, _| !key.starts_with("pse.semantic."));
    }
    metadata.insert(KEY_LAYOUT_VERSION.into(), LAYOUT_VERSION.into());
    // Sort all metadata maps, including those on nested native fields. A HashMap's
    // insertion order must not change the emitted declaration.
    metadata.insert(KEY_EXECUTION_FIELD.into(), descriptor(field)?);
    Ok(Field::new(field.name(), data_type, field.is_nullable()).with_metadata(metadata))
}

fn descriptor(value: &impl serde::Serialize) -> Result<String, SchemaError> {
    let mut descriptor = serde_json::to_value(value).map_err(|error| descriptor_error(&error))?;
    descriptor.sort_all_objects();
    serde_json::to_string(&descriptor).map_err(|error| descriptor_error(&error))
}

fn descriptor_error(error: &serde_json::Error) -> SchemaError {
    SchemaError::InvalidDeclaration {
        context: "Delta field descriptor".into(),
        reason: error.to_string(),
    }
}
fn storage_type(value: &DataType) -> Result<DataType, SchemaError> {
    use DataType as T;
    Ok(match value {
        T::UInt8 => T::Int16,
        T::UInt16 => T::Int32,
        T::UInt64 => T::Decimal128(20, 0),
        T::FixedSizeBinary(_) | T::LargeBinary | T::BinaryView => T::Binary,
        T::LargeUtf8 | T::Utf8View => T::Utf8,
        T::Float16 => T::Float32,
        T::Dictionary(_, values) => storage_type(values)?,
        T::List(field)
        | T::LargeList(field)
        | T::ListView(field)
        | T::LargeListView(field)
        | T::FixedSizeList(field, _) => {
            // Delta's Arrow conversion names every array element `element`.
            // The execution projection restores the declaration's original name.
            // Delta's ArrayType has a type and containsNull but no element-field
            // metadata. The enclosing execution descriptor preserves those facts.
            T::List(
                Field::new(
                    "element",
                    storage_type(field.data_type())?,
                    field.is_nullable(),
                )
                .into(),
            )
        }
        T::Struct(fields) => T::Struct(
            fields
                .iter()
                .map(|f| storage_field(f))
                .collect::<Result<Vec<_>, _>>()?
                .into(),
        ),
        T::Map(field, _) => {
            let T::Struct(fields) = field.data_type() else {
                return Err(SchemaError::InvalidDeclaration {
                    context: field.name().to_owned(),
                    reason: "map entries must be a key/value struct".into(),
                });
            };
            if fields.len() != 2 || fields[0].is_nullable() || field.is_nullable() {
                return Err(SchemaError::InvalidDeclaration {
                    context: field.name().to_owned(),
                    reason: "map requires non-null entries and keys".into(),
                });
            }
            // Delta has no sorted-map guarantee. Refuse to invent one on decode.
            if matches!(value, T::Map(_, true)) {
                return Err(SchemaError::InvalidDeclaration {
                    context: field.name().to_owned(),
                    reason: "sorted map durability requires an explicit order proof".into(),
                });
            }
            T::Map(
                Field::new(
                    "key_value",
                    T::Struct(
                        vec![
                            Field::new("key", storage_type(fields[0].data_type())?, false),
                            Field::new(
                                "value",
                                storage_type(fields[1].data_type())?,
                                fields[1].is_nullable(),
                            ),
                        ]
                        .into(),
                    ),
                    false,
                )
                .into(),
                false,
            )
        }
        T::Boolean
        | T::Int8
        | T::Int16
        | T::Int32
        | T::Int64
        | T::Float32
        | T::Float64
        | T::Utf8
        | T::Binary
        | T::Date32 => value.clone(),
        T::Timestamp(TimeUnit::Nanosecond | TimeUnit::Microsecond, zone)
            if zone.as_deref().is_none_or(|zone| zone == "UTC") =>
        {
            value.clone()
        }
        // Preserve raw ticks for units/zones outside Delta's native timestamp
        // contract. Scaling to microseconds would overflow valid Arrow values;
        // the execution descriptor restores the original unit and timezone.
        T::UInt32 | T::Timestamp(_, _) => T::Int64,
        T::Decimal128(precision, scale)
            if *precision <= 38 && u8::try_from(*scale).is_ok_and(|scale| scale <= *precision) =>
        {
            value.clone()
        }
        _ => {
            return Err(SchemaError::InvalidDeclaration {
                context: "Delta durable layout".into(),
                reason: format!("no lossless declared mapping for {value:?}"),
            });
        }
    })
}
