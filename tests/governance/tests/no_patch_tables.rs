// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! No `[patch]`/`[replace]` in the root manifest, and `.cargo/config.toml` stays minimal.
//!
//! A `[patch]` table makes the committed `Cargo.lock` a lie about what was built, and a
//! `target-cpu` or `rustflags` key makes a developer machine and CI produce different
//! floating-point results: `-C target-cpu=native` lets LLVM contract `a*b + c` into one
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
fn cargo_config_stays_minimal() {
    let path = common::workspace_root().join(".cargo/config.toml");
    let config = common::parse_toml(&path);

    let mut keys = BTreeSet::new();
    let mut strings = Vec::new();
    walk(&config, &mut keys, &mut strings);

    for banned in ["linker", "rustflags", "paths", "patch", "replace"] {
        assert!(
            !keys.contains(banned),
            ".cargo/config.toml defines `{banned}`. It is deliberately minimal: a linker or \
             rustflags override belongs in ~/.cargo/config.toml, never in the repository."
        );
    }

    for text in &strings {
        assert!(
            !text.contains("target-cpu"),
            ".cargo/config.toml carries a `target-cpu` setting ({text:?}). FMA contraction \
             would change float results between a developer machine and CI."
        );
    }
}
