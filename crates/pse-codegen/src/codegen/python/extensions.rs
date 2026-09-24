// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Python Arrow storage and versioned codecs from the same extension declarations.

use std::fmt::Write as _;

use arrow_schema::{DataType, Field};

use crate::model::{EXTENSION_TYPES, ExtensionMetadataShape};
use crate::{Registry, SchemaError};

use super::pascal;

pub(super) fn render(reg: &Registry) -> Result<String, SchemaError> {
    let mut source = include_str!("extensions.py.template").to_owned();
    let mut classes = Vec::new();
    let mut names = Vec::new();
    for extension in EXTENSION_TYPES {
        let name = format!("Pse{}", pascal(extension.name.trim_start_matches("pse.")));
        let storage = storage(&extension.storage())?;
        let (binding, prototype) = match extension.metadata {
            ExtensionMetadataShape::VersionOnly => ("None".to_owned(), "None".to_owned()),
            ExtensionMetadataShape::Enum => (
                "\"enum_id\"".to_owned(),
                format!(
                    "{:?}",
                    reg.enums()
                        .first()
                        .ok_or_else(|| super::error("missing enum prototype".to_owned()))?
                        .id
                        .to_hex()
                ),
            ),
            ExtensionMetadataShape::OrdinalRef => (
                "\"target_relation_id\"".to_owned(),
                format!(
                    "{:?}",
                    reg.relations()
                        .first()
                        .ok_or_else(|| super::error("missing ordinal prototype".to_owned()))?
                        .id
                        .to_hex()
                ),
            ),
        };
        let _ = writeln!(
            source,
            "\n\nclass {name}(_PseExtensionType):\n    \"\"\"The declared {} extension.\"\"\"\n\n    _extension_name = {:?}\n    _metadata_version = {}\n    _binding_key = {binding}\n    _prototype_binding = {prototype}\n\n    @classmethod\n    def _declared_storage(cls) -> pa.DataType:\n        return {storage}",
            extension.name, extension.name, extension.metadata_version
        );
        classes.push(name);
        names.push(format!("{:?}", extension.name));
    }
    let _ = writeln!(
        source,
        "\n\n_EXTENSION_TYPES: tuple[type[_PseExtensionType], ...] = (\n    {},\n)\n\nEXTENSION_NAMES: tuple[str, ...] = (\n    {},\n)\n\n_registered = False",
        classes.join(",\n    "),
        names.join(",\n    ")
    );
    source.push_str("\n\ndef register_all() -> None:\n    \"\"\"Register passive decoders once; registration does not admit a batch.\"\"\"\n    global _registered  # noqa: PLW0603\n    if _registered:\n        return\n    for cls in _EXTENSION_TYPES:\n        # The process-global registry may already contain this passive decoder.\n        with contextlib.suppress(pa.ArrowKeyError):\n            pa.register_extension_type(cls(cls._prototype_binding))\n    _registered = True\n");
    Ok(source)
}

fn field(field: &Field) -> Result<String, SchemaError> {
    let mut metadata = field
        .metadata()
        .iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    let data_type =
        if let Some(name) = metadata.remove(&crate::arrow::KEY_EXTENSION_NAME.to_owned()) {
            let encoded = metadata
                .remove(&crate::arrow::KEY_EXTENSION_METADATA.to_owned())
                .ok_or_else(|| super::error("nested extension lacks metadata".to_owned()))?;
            let class = format!("Pse{}", pascal(name.trim_start_matches("pse.")));
            let spec = EXTENSION_TYPES
                .iter()
                .find(|spec| spec.name == name)
                .ok_or_else(|| super::error(format!("unknown nested extension {name}")))?;
            let value: serde_json::Value =
                serde_json::from_str(encoded).map_err(|error| super::error(error.to_string()))?;
            let argument = spec
                .metadata
                .id_key()
                .map(|key| {
                    value
                        .get(key)
                        .and_then(serde_json::Value::as_str)
                        .map(|value| format!("{value:?}"))
                        .ok_or_else(|| super::error(format!("nested extension lacks {key}")))
                })
                .transpose()?
                .unwrap_or_default();
            format!("{class}({argument})")
        } else {
            storage(field.data_type())?
        };
    Ok(format!(
        "pa.field({:?}, {}, nullable={}, metadata={})",
        field.name(),
        data_type,
        if field.is_nullable() { "True" } else { "False" },
        serde_json::to_string(&metadata).map_err(|error| super::error(error.to_string()))?
    ))
}

pub(super) fn storage(ty: &DataType) -> Result<String, SchemaError> {
    Ok(match ty {
        DataType::Boolean => "pa.bool_()".to_owned(),
        DataType::Int8 => "pa.int8()".to_owned(),
        DataType::Int16 => "pa.int16()".to_owned(),
        DataType::Int32 => "pa.int32()".to_owned(),
        DataType::Int64 => "pa.int64()".to_owned(),
        DataType::UInt8 => "pa.uint8()".to_owned(),
        DataType::UInt16 => "pa.uint16()".to_owned(),
        DataType::UInt32 => "pa.uint32()".to_owned(),
        DataType::UInt64 => "pa.uint64()".to_owned(),
        DataType::Float64 => "pa.float64()".to_owned(),
        DataType::Utf8 => "pa.utf8()".to_owned(),
        DataType::FixedSizeBinary(width) => format!("pa.binary({width})"),
        DataType::Timestamp(arrow_schema::TimeUnit::Nanosecond, timezone)
            if timezone.as_deref() == Some("UTC") =>
        {
            "pa.timestamp(\"ns\", tz=\"UTC\")".to_owned()
        }
        DataType::List(child) => format!("pa.list_({})", field(child)?),
        DataType::FixedSizeList(child, width) => format!("pa.list_({}, {width})", field(child)?),
        DataType::Struct(children) => format!(
            "pa.struct([{}])",
            children
                .iter()
                .map(|child| field(child))
                .collect::<Result<Vec<_>, _>>()?
                .join(", ")
        ),
        DataType::Dictionary(key, value) => {
            format!("pa.dictionary({}, {})", storage(key)?, storage(value)?)
        }
        other => {
            return Err(super::error(format!(
                "unsupported Python Arrow storage {other}"
            )));
        }
    })
}
