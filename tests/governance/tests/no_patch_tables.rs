// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! Exact root-local vendor source pins; Cargo uses mold only for `x86_64` Linux.
//!
//! Root source overrides must agree with the unique selected lock entry (ADR-0189). A
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
fn root_manifest_has_exact_selected_local_sources_and_no_replace() {
    let manifest = common::root_manifest();
    assert!(manifest.get("replace").is_none(), "replacement tables are unsupported");
    let Some(patches) = manifest.get("patch").and_then(Value::as_table) else { return; };
    assert_eq!(patches.len(), 1, "only explicit crates-io source pins are supported");
    let sources = patches.get("crates-io").and_then(Value::as_table).expect("root patch.crates-io table");
    let root = common::workspace_root().canonicalize().unwrap();
    let lock = common::parse_toml(&root.join("Cargo.lock"));
    let packages = lock["package"].as_array().unwrap();
    for (name, source) in sources {
        let source = source.as_table().expect("source pin is a table");
        assert_eq!(source.keys().map(String::as_str).collect::<BTreeSet<_>>(), BTreeSet::from(["path", "version"]), "aliases, git and implicit version selection are unsupported");
        let path = source["path"].as_str().unwrap();
        assert_eq!(path, format!("vendor/{name}"), "source pins have one repository-local owner");
        let owner = root.join(path).canonicalize().unwrap();
        assert!(owner.starts_with(root.join("vendor")), "vendor source escapes the checkout's vendor owner");
        let vendor = common::parse_toml(&owner.join("Cargo.toml"));
        assert_eq!(vendor["package"]["name"].as_str(), Some(name.as_str()));
        let version = vendor["package"]["version"].as_str().unwrap();
        assert_eq!(source["version"].as_str(), Some(format!("={version}").as_str()));
        let selected: Vec<_> = packages.iter().filter(|package| package["name"].as_str() == Some(name.as_str())).collect();
        assert_eq!(selected.len(), 1, "source pin must select one actual package/version");
        assert_eq!(selected[0]["version"].as_str(), Some(version));
        assert!(selected[0].get("source").is_none() && selected[0].get("checksum").is_none(), "lock must record the actual local source, not a registry/git substitute");
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
