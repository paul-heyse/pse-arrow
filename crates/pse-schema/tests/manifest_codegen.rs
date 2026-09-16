// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Manifest Rust projection follows declarations and refuses incompatible native codecs.

use std::path::Path;

use pse_schema::builder::RegistryBuilder;
use pse_schema::codegen::{Language, generate};
use pse_schema::model::{ManifestField, ManifestRustBinding, ManifestSpec, ManifestType};
use pse_schema::{Registry, SchemaError};
use quote::ToTokens;

fn registry(fields: Vec<ManifestField>) -> Result<Registry, SchemaError> {
    let mut builder = RegistryBuilder::new();
    builder.declare_manifest(ManifestSpec::new("fixture.v3", "fixture.v2", fields));
    builder.build()
}

#[test]
fn native_codecs_reject_changed_wire_shapes_before_generation() {
    for binding in [
        ManifestRustBinding::SnapshotKind,
        ManifestRustBinding::SnapshotId,
        ManifestRustBinding::LogicalHash,
        ManifestRustBinding::EncodingChecksum,
        ManifestRustBinding::SchemaVersion,
        ManifestRustBinding::EncodingFormat,
        ManifestRustBinding::SnapshotParents,
    ] {
        let result = registry(vec![
            ManifestField::new("value", ManifestType::Bool, "value").with_rust(binding),
        ]);
        assert!(matches!(
            result,
            Err(SchemaError::InvalidDeclaration { .. })
        ));
    }
    let result = registry(vec![
        ManifestField::new(
            "parents",
            ManifestType::list(ManifestType::Struct(vec![
                ManifestField::new("role", ManifestType::Text, "role"),
                ManifestField::new("snapshot_id", ManifestType::Hash, "id"),
            ])),
            "parents",
        )
        .with_rust(ManifestRustBinding::SnapshotParents),
    ]);
    assert!(matches!(
        result,
        Err(SchemaError::InvalidDeclaration { .. })
    ));
}

#[test]
fn new_nested_manifest_fields_are_projected_without_generator_inventory_changes() {
    let registry = registry(vec![
        ManifestField::new("z_first", ManifestType::U64, "first"),
        ManifestField::new(
            "future",
            ManifestType::optional(ManifestType::list(ManifestType::Struct(vec![
                ManifestField::new("type", ManifestType::Text, "a keyword"),
                ManifestField::new("amount", ManifestType::U32, "amount"),
            ]))),
            "future field",
        ),
    ])
    .expect("fixture declaration");
    let tree = generate(&registry, Language::Rust).expect("generation");
    assert!(
        tree.files
            .keys()
            .all(|path| !path.starts_with("crates/pse-compiler")),
        "a standalone manifest does not declare a mathematical graph bridge"
    );
    let bytes = &tree.files[Path::new("crates/pse-catalog/src/generated/manifest.rs")];
    let source = std::str::from_utf8(bytes).expect("UTF-8");
    let syntax = syn::parse_file(source).expect("Rust syntax");
    let objects = syntax
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Struct(object) => Some(object),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(objects.len(), 2);
    let root = objects
        .iter()
        .find(|item| item.ident == "Manifest")
        .expect("root");
    let fields = root.fields.iter().collect::<Vec<_>>();
    assert_eq!(
        fields[0].ident.as_ref().expect("name").to_string(),
        "r#z_first"
    );
    assert_eq!(
        fields[1].ident.as_ref().expect("name").to_string(),
        "r#future"
    );
    assert_eq!(
        fields[1].ty.to_token_stream().to_string(),
        "Option < Vec < ManifestFutureItem > >"
    );
    assert!(source.contains("pub const MANIFEST_VERSION: &str = \"fixture.v3\""));
    assert_eq!(source.matches("deny_unknown_fields").count(), 2);
    assert!(source.contains("rename = \"type\""));
    assert!(!source.contains("skip_serializing_if"));
}

#[test]
fn unsupported_rust_field_names_return_typed_codegen_error() {
    let registry = registry(vec![ManifestField::new(
        "two words",
        ManifestType::Text,
        "wire",
    )])
    .expect("valid wire declaration");
    assert!(matches!(
        generate(&registry, Language::Rust),
        Err(SchemaError::Codegen {
            language: "rust",
            ..
        })
    ));
}
