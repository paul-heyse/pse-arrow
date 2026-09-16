// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "assertions in generator regression tests"
)]

//! Deterministic complete generation and parseable emitted Rust.

use pse_schema::codegen::{Language, generate};
use pse_schema::model::SourceColumn;

#[test]
fn source_schema_sorts_nested_objects_and_keeps_required_column_order() {
    let registry = pse_schema::registry().expect("registry");
    let tree = generate(registry, Language::Markdown).expect("document schema generation");
    let bytes = &tree.files[std::path::Path::new("docs/generated/schema/authoring.schema.json")];
    let text = std::str::from_utf8(bytes).expect("UTF-8 schema");
    let schema: serde_json::Value = serde_json::from_slice(bytes).expect("valid schema JSON");
    for document in registry.documents() {
        for section in &document.sections {
            let key = format!("source:{}:{}", document.name, section.key);
            let mut definition = schema["$defs"][&key].clone();
            definition.sort_all_objects();
            let sorted = serde_json::to_string(&definition).expect("schema serialization");
            assert!(text.contains(&format!("\"{key}\":{sorted}")), "{key}");

            let relation = registry
                .relation(section.relation)
                .expect("source relation");
            let required: Vec<_> = relation
                .columns
                .iter()
                .filter(|column| {
                    !column.nullable()
                        && section.identity_column != Some(column.name())
                        && section.source_column(column) == SourceColumn::Authored
                })
                .map(pse_schema::model::FieldContract::name)
                .collect();
            assert_eq!(definition["required"], serde_json::json!(required), "{key}");
        }
    }
}

#[test]
fn all_targets_regenerate_identical_bytes_and_complete_inventory() {
    let reg = pse_schema::registry().expect("registry");
    for language in Language::ALL {
        let first = generate(reg, language).expect("first generation");
        let second = generate(reg, language).expect("second generation");
        assert_eq!(first, second);
        assert!(!first.files.is_empty());
        assert!(
            first
                .files
                .keys()
                .all(|path| first.roots.iter().any(|root| path.starts_with(root)))
        );
        if language == Language::Rust {
            for (path, bytes) in &first.files {
                syn::parse_file(std::str::from_utf8(bytes).expect("utf8"))
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            }
            for spec in reg.relations() {
                let path = format!(
                    "crates/pse-relations/src/generated/{}/{}.rs",
                    spec.key.namespace.as_str(),
                    spec.key.name
                );
                assert!(
                    first.files.contains_key(std::path::Path::new(&path)),
                    "{path}"
                );
            }
        }
    }
}

#[test]
fn source_schema_retains_dsl_grammar_beside_identity_alternatives() {
    let registry = pse_schema::registry().expect("registry");
    let tree = generate(registry, Language::Markdown).expect("document schema generation");
    let bytes = &tree.files[std::path::Path::new("docs/generated/schema/authoring.schema.json")];
    let schema: serde_json::Value = serde_json::from_slice(bytes).expect("valid schema JSON");
    for document in registry.documents() {
        for section in &document.sections {
            let definition = &schema["$defs"][format!("source:{}:{}", document.name, section.key)];
            for (path, syntax) in section.expression_fields {
                let mut field = definition;
                for segment in path.split('.') {
                    if field["anyOf"][1]["type"] == "null" {
                        field = &field["anyOf"][0];
                    }
                    let name = segment.trim_end_matches("[]");
                    field = &field["properties"][name];
                    for _ in 0..(segment.len() - name.len()) / 2 {
                        if field["anyOf"][1]["type"] == "null" {
                            field = &field["anyOf"][0];
                        }
                        field = &field["items"];
                    }
                }
                if field["anyOf"][1]["type"] == "null" {
                    field = &field["anyOf"][0];
                }
                assert_eq!(
                    field["x-pse-dsl-syntax"],
                    syntax.as_str(),
                    "{}.{path}",
                    section.relation
                );
            }
        }
    }
}
