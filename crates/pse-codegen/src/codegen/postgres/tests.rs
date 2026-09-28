// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    clippy::unwrap_used,
    reason = "pure generation over the actual registry"
)]

use super::*;

fn text(tree: &GeneratedTree, name: &str) -> String {
    String::from_utf8(tree.files[&PathBuf::from(format!("{ROOT}/{name}"))].clone()).unwrap()
}

#[test]
fn ddl_covers_store_relations_enums_identities() {
    let registry = crate::registry().unwrap();
    let tree = crate::codegen::generate(registry, Language::Postgres).unwrap();
    assert_eq!(tree.roots, Language::Postgres.roots());
    let sql = text(&tree, "schema.sql");
    let relations = pse_schema::store::relations(registry);
    assert!(!relations.is_empty());
    for (table, spec) in &relations {
        assert!(
            sql.contains(&format!("CREATE TABLE pse_ops.\"{table}\" (")),
            "{table} is not created"
        );
        assert!(sql.contains(&format!("-- {}\n", spec.qualified_name())));
        for column in &spec.columns {
            assert!(sql.contains(&format!("\n    \"{}\" ", column.name())));
        }
    }
    for name in ddl::enums(registry) {
        let spec = registry.enum_spec(name).unwrap();
        let labels = spec
            .members
            .iter()
            .map(|member| format!("'{}'", member.name))
            .collect::<Vec<_>>()
            .join(", ");
        assert!(sql.contains(&format!(
            "CREATE TYPE pse_ops.{} AS ENUM ({labels});",
            names::enum_type(name)
        )));
    }
    for name in ddl::identities(registry) {
        assert!(sql.contains(&format!(
            "CREATE DOMAIN pse_ops.{} AS ",
            names::identity_domain(name)
        )));
    }
    assert!(sql.contains("CREATE DOMAIN pse_ops.content_hash AS bytea"));
    // Meaning only: no defaults, cascades or identity minting in the generated DDL.
    for absent in ["DEFAULT", "CASCADE", "uuidv7", "CREATE INDEX"] {
        assert!(!sql.contains(absent), "generated DDL contains {absent}");
    }
    let mapping = text(&tree, "cornucopia.toml");
    assert!(mapping.contains("\"pg_catalog.uuid\" = \"pse_ids::SemanticId\""));
    assert!(mapping.contains("\"pse_ops.content_hash\" = \"pse_ids::ContentHash\""));
    for name in ddl::enums(registry) {
        assert!(mapping.contains(&format!("\"pse_ops.{}\"", names::enum_type(name))));
    }
    // Every table's row type maps to the registry row it stores, owned (X7).
    for (table, _) in &relations {
        assert!(
            mapping.contains(&format!("\"pse_ops.{table}\" = {{ rust-type = ")),
            "{table} row type is not mapped"
        );
    }
    assert!(mapping.contains(
        "\"pse_ops.attempts\" = { rust-type = \"pse_model::generated::runtime::operational_attempts::RuntimeOperationalAttemptsRow\", is-copy = false }"
    ));
    assert!(text(&tree, "mod.rs").contains("include_str!(\"schema.sql\")"));
}

#[test]
fn copy_statements_cover_store_tables() {
    let registry = crate::registry().unwrap();
    let tree = crate::codegen::generate(registry, Language::Postgres).unwrap();
    let copy = text(&tree, "copy.rs");
    for (table, spec) in pse_schema::store::relations(registry) {
        let columns = spec
            .columns
            .iter()
            .map(|column| format!("\\\"{}\\\"", column.name()))
            .collect::<Vec<_>>()
            .join(", ");
        assert!(
            copy.contains(&format!(
                "COPY pse_ops.\\\"{table}\\\" ({columns}) FROM STDIN (FORMAT binary)"
            )),
            "{table} has no binary copy"
        );
        assert!(copy.contains(&format!(
            "SELECT {columns} FROM pse_ops.\\\"{table}\\\" WHERE false"
        )));
    }
    assert!(text(&tree, "mod.rs").contains("pub mod copy;"));
}

#[test]
fn schema_fingerprint_covers_both_files() {
    let (path, bytes) = fingerprint_file(b"schema", b"physical").unwrap();
    assert_eq!(path, PathBuf::from(format!("{ROOT}/fingerprint.rs")));
    let source = String::from_utf8(bytes).unwrap();
    let hex = fingerprint(b"schema", b"physical").to_hex();
    assert!(source.contains(&format!("SCHEMA_FINGERPRINT_HEX: &str = \"{hex}\"")));
    assert!(source.contains(&format!(
        "COMMENT ON SCHEMA pse_ops IS 'pse.ops.schema.v1 {hex}'"
    )));
    assert_ne!(
        fingerprint(b"schema", b"physical"),
        fingerprint(b"schema", b"physical2")
    );
    assert_ne!(
        fingerprint(b"schemaphysical", b""),
        fingerprint(b"schema", b"physical")
    );
}
