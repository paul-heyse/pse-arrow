// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Closed host-header projection over responsibility-owned operation/document catalogues.
use anyhow::{Context, Result, bail, ensure};
use pse_codegen::codegen::{GeneratedTree, documents::CatalogueDocument, provenance};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    path::PathBuf,
};

pub(super) fn append(tree: &mut GeneratedTree, private: &[CatalogueDocument]) -> Result<()> {
    let native = pse_runtime::host_operations::catalogue();
    let mut operations = serde_json::to_value(&native.operations)?;
    let rows = operations
        .as_array_mut()
        .context("native operations must be an array")?;
    let mut evidence = xtask::development::host_operations::declaration();
    let slugs = evidence
        .get("document_slugs")
        .and_then(Value::as_object)
        .context("tooling operation requires original document references")?;
    let fields = slugs.iter().map(|(field, slug)| {
        Ok(serde_json::json!({"field": field, "document": reference(private, slug.as_str().context("document slug must be text")?, false)?}))
    }).collect::<Result<Vec<_>>>()?;
    evidence["documents"] = fields.into();
    let output = reference(
        private,
        evidence["output_slug"]
            .as_str()
            .context("tooling result slug must be text")?,
        true,
    )?;
    evidence["output_document"] = output;
    evidence
        .as_object_mut()
        .context("tooling operation must be an object")?
        .remove("document_slugs");
    evidence
        .as_object_mut()
        .context("tooling operation must be an object")?
        .remove("output_slug");
    rows.push(evidence);
    let mut identities = BTreeSet::new();
    for operation in rows.iter() {
        let identity = operation["operation"]
            .as_str()
            .context("operation requires an identity")?;
        ensure!(
            identities.insert(identity),
            "duplicate host operation {identity}"
        );
    }
    let schemas = pse_runtime::host_operations::header_schemas();
    let source = python(&schemas, &operations, &serde_json::to_value(native)?)?;
    let path = PathBuf::from("scripts/generated/native_operations.py");
    let root = crate::workspace_root()?;
    let formatted =
        super::python_stubs::format_projection(&root, source.as_bytes(), &path.to_string_lossy())?;
    tree.files.insert(path.clone(), formatted);
    tree.files.insert(
        "scripts/generated/host_operation_headers.json".into(),
        serde_json::to_vec_pretty(&schemas)?,
    );
    for (owner, symbol) in [
        ("crates/pse-runtime/src/host_operations.rs", "operations"),
        ("xtask/src/development/host_operations.rs", "declaration"),
    ] {
        tree.relationships.push(provenance::SourceRelationship {
            declaration_key: format!("{owner}::{symbol}"),
            declaration_kind: "host_operation_projection".into(),
            source_path: Some(owner.into()),
            source_symbol: Some(symbol.into()),
            source_role: "definition".into(),
            source_identity: None,
            generator_path: "xtask/src/codegen/host_operations.rs".into(),
            generator_symbol: "append".into(),
            output_path: path.to_string_lossy().into_owned(),
            direction: Some("input_output".into()),
            contract_sections: Vec::new(),
            basis: "declared".into(),
        });
    }
    Ok(())
}
fn reference(catalogue: &[CatalogueDocument], slug: &str, output: bool) -> Result<Value> {
    let document = catalogue
        .iter()
        .find(|entry| entry.slug == slug && entry.output == output)
        .with_context(|| format!("host operation has no original directional document: {slug}"))?;
    Ok(serde_json::json!({"slug": slug, "name": document.name,
        "input": catalogue.iter().any(|entry| entry.slug == slug && !entry.output),
        "output": catalogue.iter().any(|entry| entry.slug == slug && entry.output)}))
}
fn python(schemas: &[(&str, Value)], operations: &Value, native: &Value) -> Result<String> {
    let mut source = String::from(
        "# @generated from Rust host operation owners; do not edit\n\"\"\"Host contracts projected from the existing Rust operation owners.\"\"\"\nfrom __future__ import annotations\n\nfrom typing import Annotated, Literal\nfrom pydantic import Field, JsonValue, TypeAdapter\nfrom scripts import execution_profiles\nfrom scripts.host_contracts import Contract\n\n",
    );
    let known = schemas
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    let mut inherited = BTreeMap::<String, BTreeSet<String>>::new();
    for (name, schema) in schemas {
        let properties = schema["properties"]
            .as_object()
            .with_context(|| format!("host header {name} is not an object"))?;
        ensure!(
            schema["additionalProperties"] == false,
            "host header {name} must refuse unknown fields"
        );
        let base = schema
            .get("x-pse-host-base")
            .and_then(Value::as_str)
            .unwrap_or("Contract");
        let base_fields = if base == "Contract" {
            BTreeSet::new()
        } else {
            inherited
                .get(base)
                .with_context(|| format!("host base {base} must precede {name}"))?
                .clone()
        };
        writeln!(source, "class {name}({base}):")?;
        let required = schema["required"]
            .as_array()
            .map(|fields| {
                fields
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let definitions = schema
            .get("$defs")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let mut count = 0;
        for (field, shape) in properties {
            if base_fields.contains(field) {
                continue;
            }
            ensure!(
                identifier(field),
                "unsupported host field identifier {field}"
            );
            let annotation = python_type(shape, &definitions, &known)
                .with_context(|| format!("host header {name}.{field}"))?;
            let default = if let Some(reference) =
                shape.get("x-pse-host-default").and_then(Value::as_str)
            {
                if reference == "Resources" {
                    " = Field(default_factory=Resources)".into()
                } else if let Some(attribute) = reference.strip_prefix("production.capabilities.") {
                    ensure!(
                        matches!(
                            attribute,
                            "memory_limit_bytes" | "threads" | "max_spill_bytes"
                        ) && attribute == field,
                        "unsupported production capability reference {reference}"
                    );
                    format!(
                        " = Field(default_factory=lambda: execution_profiles.declarations().capabilities.{attribute})"
                    )
                } else {
                    bail!("unsupported host default reference {reference}")
                }
            } else if let Some(default) = shape.get("default") {
                if default.is_array() {
                    " = Field(default_factory=list)".into()
                } else {
                    format!(" = {}", python_literal(default)?)
                }
            } else if !required.contains(field.as_str()) {
                " = None".into()
            } else {
                String::new()
            };
            writeln!(source, "    {field}: {annotation}{default}")?;
            count += 1;
        }
        if count == 0 {
            writeln!(source, "    pass")?;
        }
        writeln!(source)?;
        inherited.insert((*name).to_owned(), properties.keys().cloned().collect());
    }
    writeln!(
        source,
        "DEFAULT_BATCH_SIZE: int = {}\n",
        pse_runtime::host_operations::project::default_batch_size()
    )?;
    writeln!(source, "ENVELOPES: dict[str, type[Contract]] = {{")?;
    for (name, _) in schemas {
        writeln!(source, "    {name:?}: {name},")?;
    }
    writeln!(source, "}}\n")?;
    writeln!(
        source,
        "OPERATION_DECLARATIONS: list[OperationDeclaration] = TypeAdapter(list[OperationDeclaration]).validate_json({:?}, strict=True)",
        serde_json::to_string(operations)?
    )?;
    writeln!(
        source,
        "NATIVE_CATALOGUE: OperationCatalogue = TypeAdapter(OperationCatalogue).validate_json({:?}, strict=True)\n",
        serde_json::to_string(native)?
    )?;
    writeln!(
        source,
        "__all__ = ({})",
        known
            .iter()
            .map(|name| format!("{name:?},"))
            .collect::<Vec<_>>()
            .join(" ")
    )?;
    Ok(source)
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        && value.as_bytes()[0].is_ascii_alphabetic()
}
fn python_type(schema: &Value, definitions: &Value, known: &BTreeSet<String>) -> Result<String> {
    if schema == &Value::Bool(true) || schema.as_object().is_some_and(serde_json::Map::is_empty) {
        return Ok("JsonValue".into());
    }
    let object = schema
        .as_object()
        .context("unsupported host field schema")?;
    for key in object.keys() {
        ensure!(
            matches!(
                key.as_str(),
                "$ref"
                    | "type"
                    | "const"
                    | "enum"
                    | "anyOf"
                    | "oneOf"
                    | "items"
                    | "format"
                    | "minimum"
                    | "maximum"
                    | "exclusiveMinimum"
                    | "exclusiveMaximum"
                    | "minLength"
                    | "maxLength"
                    | "minItems"
                    | "maxItems"
                    | "pattern"
                    | "title"
                    | "description"
                    | "default"
                    | "x-pse-host-default"
            ),
            "unsupported host field schema construct {key}"
        );
    }
    let unconstrained = object.keys().all(|key| {
        matches!(
            key.as_str(),
            "title" | "description" | "default" | "x-pse-host-default"
        )
    });
    let mut annotation = if unconstrained {
        "JsonValue".into()
    } else if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        let name = reference
            .strip_prefix("#/$defs/")
            .context("external host references are unsupported")?;
        if known.contains(name) {
            name.to_owned()
        } else {
            python_type(
                definitions
                    .get(name)
                    .with_context(|| format!("missing header definition {name}"))?,
                definitions,
                known,
            )?
        }
    } else if let Some(value) = schema.get("const") {
        format!("Literal[{}]", python_literal(value)?)
    } else if let Some(variants) = schema.get("enum").and_then(Value::as_array) {
        ensure!(!variants.is_empty(), "empty host enumeration");
        format!(
            "Literal[{}]",
            variants
                .iter()
                .map(python_literal)
                .collect::<Result<Vec<_>>>()?
                .join(", ")
        )
    } else if let Some(variants) = schema
        .get("anyOf")
        .or_else(|| schema.get("oneOf"))
        .and_then(Value::as_array)
    {
        python_union(variants, definitions, known)?
    } else {
        match schema.get("type") {
            Some(Value::String(kind)) => match kind.as_str() {
                "string" => "str".into(),
                "integer" => "int".into(),
                "number" => "float".into(),
                "boolean" => "bool".into(),
                "null" => "None".into(),
                "array" => format!(
                    "list[{}]",
                    python_type(
                        schema.get("items").context("host array requires items")?,
                        definitions,
                        known
                    )?
                ),
                other => bail!("unsupported host type {other}"),
            },
            Some(Value::Array(kinds)) => kinds
                .iter()
                .map(|kind| {
                    let mut branch = schema.clone();
                    branch["type"] = kind.clone();
                    python_type(&branch, definitions, known)
                })
                .collect::<Result<Vec<_>>>()?
                .join(" | "),
            _ => bail!("host field has no supported type"),
        }
    };
    let mut constraints = Vec::new();
    for (keyword, target) in [
        ("minimum", "ge"),
        ("maximum", "le"),
        ("exclusiveMinimum", "gt"),
        ("exclusiveMaximum", "lt"),
        ("minLength", "min_length"),
        ("maxLength", "max_length"),
        ("minItems", "min_length"),
        ("maxItems", "max_length"),
        ("pattern", "pattern"),
    ] {
        if let Some(value) = schema.get(keyword) {
            constraints.push(format!("{target}={}", python_literal(value)?));
        }
    }
    if !constraints.is_empty() {
        annotation = format!("Annotated[{annotation}, Field({})]", constraints.join(", "));
    }
    Ok(annotation)
}
fn python_union(
    variants: &[Value],
    definitions: &Value,
    known: &BTreeSet<String>,
) -> Result<String> {
    let mut literals = Vec::new();
    let mut other = Vec::new();
    for variant in variants {
        let annotation = python_type(variant, definitions, known)?;
        if let Some(members) = annotation
            .strip_prefix("Literal[")
            .and_then(|value| value.strip_suffix(']'))
        {
            literals.push(members.to_owned());
        } else if !other.contains(&annotation) {
            other.push(annotation);
        }
    }
    if !literals.is_empty() {
        other.insert(0, format!("Literal[{}]", literals.join(", ")));
    }
    Ok(other.join(" | "))
}
fn python_literal(value: &Value) -> Result<String> {
    Ok(match value {
        Value::Null => "None".into(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        Value::String(text) => format!("{text:?}"),
        Value::Number(number) => number.to_string(),
        other => bail!("unsupported host literal {other}"),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn host_operation_projection_refuses_unknown_schema_constructs() {
        assert!(
            python_type(
                &serde_json::json!({"type":"integer", "not":{"const":4}}),
                &serde_json::json!({}),
                &BTreeSet::new()
            )
            .is_err()
        );
        assert_eq!(
            python_type(
                &serde_json::json!({"type":"integer", "minimum":1,"maximum":16}),
                &serde_json::json!({}),
                &BTreeSet::new()
            )
            .unwrap(),
            "Annotated[int, Field(ge=1, le=16)]"
        );
        assert_eq!(python_type(&serde_json::json!({"anyOf":[{"const":"observation"},{"const":"capture"},{"type":"null"}]}), &serde_json::json!({}), &BTreeSet::new()).unwrap(), "Literal[\"observation\", \"capture\"] | None");
    }
}
