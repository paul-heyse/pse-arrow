// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! Every crate directory is an actual Cargo workspace member; the declared members are
//! exactly the packages Cargo resolves; every member inherits the workspace package
//! metadata and lints; the feature that `[workspace.metadata.pse].validate-features` names
//! actually exists.
//!
//! Cargo owns workspace membership. A directory under `crates/` that Cargo does not build
//! is a boundary nobody decided on (blueprint §24.1); adding or removing a crate is an ADR
//! (GOVERNANCE.md), and blueprint §3.2 explains the roles without being parsed.

mod common;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use toml::Value;

/// Package directories Cargo resolves as workspace members.
fn cargo_member_dirs() -> BTreeSet<PathBuf> {
    let metadata = common::metadata();
    metadata
        .packages
        .iter()
        .filter(|pkg| metadata.workspace_members.contains(&pkg.id))
        .map(|pkg| {
            let manifest: &Path = pkg.manifest_path.as_std_path();
            let dir = manifest
                .parent()
                .expect("a manifest has a parent directory");
            dir.canonicalize()
                .unwrap_or_else(|err| panic!("resolving {}: {err}", dir.display()))
        })
        .collect()
}

/// Crate directories that are not Cargo workspace members, by directory name.
fn unregistered(dirs: &[(String, PathBuf)], members: &BTreeSet<PathBuf>) -> Vec<String> {
    dirs.iter()
        .filter(|(_, path)| {
            !path
                .canonicalize()
                .is_ok_and(|resolved| members.contains(&resolved))
        })
        .map(|(name, _)| name.clone())
        .collect()
}

#[test]
fn every_crate_directory_is_a_workspace_member() {
    let unregistered = unregistered(&common::crate_dirs(), &cargo_member_dirs());
    assert!(
        unregistered.is_empty(),
        "crates/ contains directories Cargo does not build as workspace members: \
         {unregistered:?}. Add the crate through an ADR (GOVERNANCE.md) or remove the \
         directory."
    );
}

#[test]
fn unregistered_crate_directory_is_detected() {
    let members: BTreeSet<PathBuf> = cargo_member_dirs();
    let stray =
        std::env::temp_dir().join(format!("pse-governance-stray-crate-{}", std::process::id()));
    std::fs::create_dir_all(&stray).expect("creating the control directory");
    let dirs = vec![
        (
            "pse-ids".to_owned(),
            common::workspace_root().join("crates/pse-ids"),
        ),
        ("pse-stray".to_owned(), stray.clone()),
    ];
    let found = unregistered(&dirs, &members);
    std::fs::remove_dir_all(&stray).expect("removing the control directory");
    assert_eq!(
        found,
        ["pse-stray"],
        "control: a non-member directory must be reported"
    );
}

#[test]
fn declared_members_are_cargo_members() {
    let declared: BTreeSet<PathBuf> = common::member_dirs()
        .into_iter()
        .map(|dir| {
            dir.canonicalize()
                .unwrap_or_else(|err| panic!("resolving {}: {err}", dir.display()))
        })
        .collect();
    let resolved = cargo_member_dirs();
    assert_eq!(
        declared, resolved,
        "[workspace] members in Cargo.toml and Cargo's resolved workspace members differ"
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
