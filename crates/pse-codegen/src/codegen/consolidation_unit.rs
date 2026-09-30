// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    clippy::unwrap_used,
    reason = "pure generation over the actual registry"
)]
use super::*;

#[test]
fn all_renderers_are_deterministic_and_json_escaping_roundtrips() {
    let registry = crate::registry().unwrap();
    for language in Language::ALL {
        let first = generate(registry, language).unwrap();
        assert!(!first.files.is_empty());
        assert_eq!(first, generate(registry, language).unwrap());
    }
    let schema = jsonschema::generate(registry).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&schema).unwrap();
    assert!(parsed["$defs"]["runtime.validation_findings"].is_object());
    for text in ["\"quoted\"\\path", "line\n\t\r\u{0001}", "東京🦀"] {
        let quoted = jsonschema::quote(text);
        assert_eq!(serde_json::from_str::<String>(&quoted).unwrap(), text);
    }
}

#[test]
fn foundation_unit_semantic_values_and_serde_follow_declared_consumers() {
    let registry = crate::registry().unwrap();
    let tree = generate(registry, Language::Rust).unwrap();
    let read = |path: &str| std::str::from_utf8(&tree.files[&PathBuf::from(path)]).unwrap();
    let semantic = read("crates/pse-model/src/generated/normalized/package_graph.rs");
    assert!(!semantic.contains("RowBuilder"));
    assert!(
        !tree
            .files
            .keys()
            .any(|p| p.to_string_lossy().contains("expr_nodes"))
    );
    assert!(!semantic.contains("serde::Serialize"));
    assert!(
        read("crates/pse-model/src/generated/authored/packages.rs").contains("serde::Serialize")
    );
    assert!(
        read("crates/pse-model/src/generated/authored/numerical_requirements.rs")
            .contains("serde::Serialize")
    );
    assert!(
        read("crates/pse-model/src/generated/runtime/native_dependencies.rs")
            .contains("serde::Serialize")
    );
    assert!(
        read("crates/pse-relations/src/generated/normalized/package_graph.rs")
            .contains("RowBuilder")
    );
}

#[test]
fn field_facet_policy_is_generated_into_the_lower_columnar_layer() {
    let tree = generate(crate::registry().unwrap(), Language::Rust).unwrap();
    let path = PathBuf::from("crates/pse-columnar/src/generated/field_facets.rs");
    let policy = std::str::from_utf8(&tree.files[&path]).unwrap();
    assert!(tree.roots.contains(&PathBuf::from("crates/pse-columnar/src/generated")));
    assert!(policy.contains("pub const FACETS"));
    assert!(!policy.contains("pse_schema::"));
    for facet in crate::model::field_facets::FACETS {
        assert!(policy.contains(&format!("pub const {}:", facet.constant)));
        assert!(policy.contains(facet.key));
    }
}
