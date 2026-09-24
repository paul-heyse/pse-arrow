// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! No `[patch]`/`[replace]` in the root manifest; Cargo uses mold only for x86_64 Linux.
//!
//! A `[patch]` table makes the committed `Cargo.lock` a lie about what was built, and a
//! `target-cpu` makes a developer machine and CI produce different floating-point
//! results: `-C target-cpu=native` lets LLVM contract `a*b + c` into one
//! FMA, so a parity tolerance can pass in one place and fail in the other (blueprint §3.1,
//! .cargo/config.toml).
//!
//! Both files are parsed, never grepped: their own comments say the words this test bans.

mod common;

use std::collections::BTreeSet;

use toml::Value;

/// Collects every key name and every string value in a TOML document.
fn walk(value: &Value, keys: &mut BTreeSet<String>, strings: &mut Vec<String>) {
    match value {
        Value::Table(table) => {
            for (key, child) in table {
                keys.insert(key.clone());
                walk(child, keys, strings);
            }
        }
        Value::Array(items) => {
            for item in items {
                walk(item, keys, strings);
            }
        }
        Value::String(text) => strings.push(text.clone()),
        _ => {}
    }
}

#[test]
fn root_manifest_has_no_patch_or_replace() {
    let manifest = common::root_manifest();
    for table in ["patch", "replace"] {
        assert!(
            manifest.get(table).is_none(),
            "Cargo.toml declares [{table}]; the committed Cargo.lock would stop describing \
             what CI builds (blueprint §3.1). Pin the dependency instead, by ADR."
        );
    }
}

#[test]
fn cargo_config_uses_mold_without_cpu_overrides() {
    let path = common::workspace_root().join(".cargo/config.toml");
    let config = common::parse_toml(&path);

    let mut keys = BTreeSet::new();
    let mut strings = Vec::new();
    walk(&config, &mut keys, &mut strings);

    for banned in ["paths", "patch", "replace"] {
        assert!(
            !keys.contains(banned),
            ".cargo/config.toml defines forbidden `{banned}`."
        );
    }

    let target = config
        .get("target")
        .and_then(Value::as_table)
        .expect(".cargo/config.toml must define target settings");
    assert_eq!(
        target.len(),
        1,
        "only the x86_64 Linux target may override the linker"
    );
    let linux = target
        .get("x86_64-unknown-linux-gnu")
        .and_then(Value::as_table)
        .expect("x86_64 Linux must configure mold");
    assert_eq!(linux.len(), 2, "only linker and rustflags are allowed");
    assert_eq!(linux.get("linker").and_then(Value::as_str), Some("clang"));
    assert_eq!(
        linux
            .get("rustflags")
            .and_then(Value::as_array)
            .map(|flags| flags.iter().filter_map(Value::as_str).collect::<Vec<_>>()),
        Some(vec!["-C", "link-arg=-fuse-ld=mold"])
    );

    for text in &strings {
        assert!(
            !text.contains("target-cpu"),
            ".cargo/config.toml carries a `target-cpu` setting ({text:?}). FMA contraction \
             would change float results between a developer machine and CI."
        );
    }
}
