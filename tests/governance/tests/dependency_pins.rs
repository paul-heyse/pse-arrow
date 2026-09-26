// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Cargo owns pins; the blueprint describes policy rather than a duplicate lockfile.
#![allow(
    clippy::expect_used,
    reason = "test reports malformed manifest contracts"
)]
mod common;
use toml::Value;

fn pinned(value: &Value) -> bool {
    if let Some(table) = value.as_table() {
        if table.contains_key("path") {
            return true;
        }
        if table.contains_key("git") {
            return table
                .get("rev")
                .and_then(Value::as_str)
                .is_some_and(|rev| rev.len() == 40 && rev.bytes().all(|c| c.is_ascii_hexdigit()));
        }
    }
    let version = value
        .as_str()
        .or_else(|| value.get("version").and_then(Value::as_str));
    version
        .and_then(|v| v.strip_prefix('='))
        .is_some_and(|v| semver::Version::parse(v).is_ok())
}

#[test]
fn external_workspace_dependencies_have_exact_manifest_pins() {
    let manifest = common::root_manifest();
    let dependencies = common::dig(&manifest, &["workspace", "dependencies"])
        .and_then(Value::as_table)
        .expect("workspace dependency declarations");
    let unpinned = dependencies
        .iter()
        .filter(|(_, v)| !pinned(v))
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    assert!(
        unpinned.is_empty(),
        "dependencies without an exact version or commit: {unpinned:?}"
    );
}

#[test]
fn moving_ranges_and_git_branches_are_not_pins() {
    for source in [
        "version = '=1.2.3'",
        "path = 'crates/local'",
        "git = 'https://example.test/repo'\nrev = '0123456789abcdef0123456789abcdef01234567'",
    ] {
        assert!(
            pinned(&toml::from_str::<Value>(source).expect("fixture")),
            "{source}"
        );
    }
    for source in [
        "version = '1.2.3'",
        "version = '=1.2'",
        "version = '=1.2.3, <2'",
        "git = 'https://example.test/repo'\nbranch = 'main'",
    ] {
        assert!(
            !pinned(&toml::from_str::<Value>(source).expect("fixture")),
            "{source}"
        );
    }
}
