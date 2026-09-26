// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Executable registry coverage and authority checks against actual declarations.
#![allow(
    clippy::expect_used,
    reason = "governance reports exact missing declarations"
)]
mod common;
use pse_schema::model::{Authority, Namespace, SnapshotClass};

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
                if let Some(fk) = &column.fk() {
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
                        column.name(),
                        target.key
                    );
                }
            }
        }
    }
    for pass in registry
        .algorithms()
        .iter()
        .filter(|pass| pass.name != "source")
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
