// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Executable registry coverage and authority checks against actual declarations.
#![allow(
    clippy::expect_used,
    reason = "governance reports exact missing declarations"
)]
mod common;
use pse_schema::model::{Authority, Namespace, SnapshotClass};
use std::collections::BTreeSet;

fn relation_names(cell: &str) -> Vec<&str> {
    let mut depth = 0_usize;
    let mut names = Vec::new();
    for (index, text) in cell.split('`').enumerate() {
        if index % 2 == 1 {
            if depth == 0 {
                names.push(text);
            }
        } else {
            for character in text.chars() {
                match character {
                    '(' => depth += 1,
                    ')' => depth = depth.saturating_sub(1),
                    _ => {}
                }
            }
        }
    }
    names
}

#[test]
fn relation_index_excludes_explanatory_inline_types() {
    assert_eq!(
        relation_names("`template_expr_*` (one family per `pse.expr_dsl` column), `units`"),
        ["template_expr_*", "units"]
    );
}

#[test]
fn appendix_b_has_declared_relations_or_explicit_deferred_contracts() {
    let root = common::workspace_root();
    let blueprint = common::read(&root.join("docs/authoritative_design/blueprint.md"));
    let appendix = blueprint
        .split("## Appendix B. Relation index")
        .nth(1)
        .expect("Appendix B")
        .split("## Appendix C.")
        .next()
        .expect("Appendix B body");
    let deferred = common::parse_toml(&root.join("tests/governance/appendix-b-deferred.toml"));
    let deferred = deferred["deferred"]
        .as_table()
        .expect("explicit deferred rows");
    let registry = pse_schema::registry().expect("registry");
    let mut seen = BTreeSet::new();
    for row in appendix.lines().filter(|row| row.starts_with("| `")) {
        let columns = row.split('|').collect::<Vec<_>>();
        let namespace = columns[1].trim().trim_matches('`');
        for name in relation_names(columns[2]) {
            let qualified = format!("{namespace}.{name}");
            if let Some(prefix) = qualified.strip_suffix('*') {
                assert!(
                    registry
                        .relations()
                        .iter()
                        .any(|spec| spec.qualified_name().starts_with(prefix)),
                    "empty family {qualified}"
                );
            } else if registry.relation(&qualified).is_none() {
                let reason = deferred
                    .get(&qualified)
                    .and_then(toml::Value::as_str)
                    .expect(&qualified);
                assert!(!reason.trim().is_empty(), "{qualified}");
                seen.insert(qualified);
            }
        }
    }
    assert_eq!(
        seen,
        deferred.keys().cloned().collect(),
        "deferred rows cannot hide registered or removed declarations"
    );
}

#[test]
fn primitive_authority_has_no_compiled_fk_or_later_pass_writer() {
    let registry = pse_schema::registry().expect("registry");
    for spec in registry.relations() {
        assert!(!spec.primary_key.is_empty(), "{}", spec.key);
        match spec.snapshot_class {
            SnapshotClass::Derived => assert_eq!(spec.authority, Authority::Derived),
            SnapshotClass::Model | SnapshotClass::Case => {
                assert_ne!(spec.authority, Authority::Derived);
            }
            SnapshotClass::Sidecar => {}
        }
        if matches!(spec.authority, Authority::Authored | Authority::Reference) {
            for column in &spec.columns {
                if let Some(fk) = &column.fk {
                    let target = registry
                        .relation(fk.relation)
                        .expect("admitted foreign key");
                    assert!(
                        !matches!(
                            target.key.namespace,
                            Namespace::Compiled | Namespace::Runtime
                        ),
                        "{}.{} -> {}",
                        spec.key,
                        column.name,
                        target.key
                    );
                }
            }
        }
    }
    for pass in registry
        .passes()
        .iter()
        .filter(|pass| !matches!(pass.name, "P0" | "P1" | "P2"))
    {
        for port in &pass.outputs {
            let spec = registry.relation(&port.relation).expect("declared output");
            assert_eq!(
                spec.authority,
                Authority::Derived,
                "{}.{}",
                pass.name,
                port.port
            );
        }
    }
}
