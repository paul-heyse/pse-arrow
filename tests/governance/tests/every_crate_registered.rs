// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! Every crate directory is in the blueprint or in `layout_additions.toml`; every member
//! inherits the workspace package metadata and lints; the feature that
//! `[workspace.metadata.pse].validate-features` names actually exists.
//!
//! Blueprint §3.2's code block is the layout authority. A crate that appears on disk
//! without appearing there (or in the amendment file beside this test) is a boundary
//! nobody decided on (blueprint §24.1 "Schema governance").

mod common;

use std::collections::BTreeSet;

use regex::Regex;
use toml::Value;

/// Crate names from the §3.2 code block: indented `pse-<name>/` entries.
fn blueprint_crates() -> BTreeSet<String> {
    let blueprint = common::workspace_root().join("docs/authoritative_design/blueprint.md");
    let text = common::read(&blueprint);
    let re = Regex::new(r"(?m)^\s+(pse-[a-z0-9-]+)/").expect("static regex");
    re.captures_iter(&text)
        .map(|caps| caps[1].to_owned())
        .collect()
}

/// The amendment file: crates §3.2 does not list, each covered by an ADR.
fn layout_additions() -> BTreeSet<String> {
    let path = common::workspace_root().join("tests/governance/layout_additions.toml");
    let doc = common::parse_toml(&path);
    let reason = doc
        .get("reason")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim();
    assert!(
        !reason.is_empty(),
        "layout_additions.toml must carry a `reason` naming the ADR that amends §3.2"
    );
    doc.get("additions")
        .and_then(Value::as_array)
        .expect("layout_additions.toml `additions`")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("additions entries are strings")
                .to_owned()
        })
        .collect()
}

#[test]
fn every_crate_directory_is_registered() {
    let registered: BTreeSet<String> = blueprint_crates()
        .union(&layout_additions())
        .cloned()
        .collect();

    let unregistered: Vec<String> = common::crate_dirs()
        .into_iter()
        .map(|(name, _)| name)
        .filter(|name| !registered.contains(name))
        .collect();

    assert!(
        unregistered.is_empty(),
        "crates/ contains directories that blueprint §3.2 does not list and \
         tests/governance/layout_additions.toml does not amend in: {unregistered:?}. \
         Adding a crate is an ADR (GOVERNANCE.md)."
    );
}

#[test]
fn blueprint_crates_all_exist() {
    let on_disk: BTreeSet<String> = common::crate_dirs().into_iter().map(|(n, _)| n).collect();
    let amendment = common::parse_toml(&common::workspace_root().join("tests/governance/layout_additions.toml"));
    let removals: BTreeSet<_> = amendment["removals"].as_array().expect("explicit removal decisions").iter().map(|v| v.as_str().expect("crate name").to_owned()).collect();
    assert!(removals.is_disjoint(&on_disk), "retired crate directories must be removed");
    let missing: Vec<String> = blueprint_crates()
        .into_iter()
        .filter(|name| !on_disk.contains(name) && !removals.contains(name))
        .collect();
    assert!(
        missing.is_empty(),
        "blueprint §3.2 lists crates that do not exist under crates/: {missing:?}"
    );
}

#[test]
fn every_member_inherits_workspace_metadata() {
    let mut problems: Vec<String> = Vec::new();

    for dir in common::member_dirs() {
        let manifest_path = dir.join("Cargo.toml");
        let manifest = common::parse_toml(&manifest_path);
        let rel = common::rel(&manifest_path);

        for field in ["version", "edition", "rust-version", "license"] {
            let inherited = common::dig(&manifest, &["package", field])
                .and_then(|v| v.get("workspace"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if !inherited {
                problems.push(format!(
                    "{rel}: [package].{field} is not `{field}.workspace = true`"
                ));
            }
        }

        // `publish` is either inherited or literally false: xtask, benches and the test
        // families are never published even after the library crates flip at phase-1 exit.
        let publish = common::dig(&manifest, &["package", "publish"]);
        let publish_ok = match publish {
            Some(Value::Boolean(false)) => true,
            Some(value) => value
                .get("workspace")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            None => false,
        };
        if !publish_ok {
            problems.push(format!(
                "{rel}: [package].publish must be `publish.workspace = true` or `publish = false`"
            ));
        }

        let lints_ok = common::dig(&manifest, &["lints", "workspace"])
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !lints_ok {
            problems.push(format!(
                "{rel}: missing `[lints] workspace = true`; the crate would opt out of the \
                 whole lint policy silently"
            ));
        }
    }

    assert!(
        problems.is_empty(),
        "manifest inheritance problems:\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn validate_features_exist() {
    let manifest = common::root_manifest();
    let features = common::dig(
        &manifest,
        &["workspace", "metadata", "pse", "validate-features"],
    )
    .and_then(Value::as_array)
    .expect("[workspace.metadata.pse] validate-features");
    assert!(
        !features.is_empty(),
        "validate-features is empty: nothing would switch on Arrow's force_validate in CI \
         (blueprint §24.1 \"Structural validity\")"
    );

    for entry in features {
        let spec = entry
            .as_str()
            .expect("validate-features entries are strings");
        let (package, feature) = spec
            .split_once('/')
            .unwrap_or_else(|| panic!("validate-features entry `{spec}` is not `package/feature`"));
        let manifest_path = common::workspace_root()
            .join("crates")
            .join(package)
            .join("Cargo.toml");
        assert!(
            manifest_path.is_file(),
            "validate-features names `{package}`, which has no manifest at {}",
            common::rel(&manifest_path)
        );
        let crate_manifest = common::parse_toml(&manifest_path);
        let declared = common::dig(&crate_manifest, &["features", feature]).is_some();
        assert!(
            declared,
            "{package}/Cargo.toml has no `[features] {feature}`, but \
             [workspace.metadata.pse].validate-features names it"
        );
        let default = common::dig(&crate_manifest, &["features", "default"])
            .and_then(Value::as_array)
            .is_some_and(|d| d.iter().any(|v| v.as_str() == Some(feature)));
        assert!(
            !default,
            "{package}/{feature} is a default feature. force_validate must be opt-in: every \
             test invocation passes it explicitly so a release build never pays for it."
        );
    }
}
