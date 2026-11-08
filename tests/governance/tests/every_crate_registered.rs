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
//! metadata and lints.
//!
//! Cargo owns workspace membership. A directory under `crates/` that Cargo does not build
//! is a boundary nobody decided on (blueprint §24.1); adding or removing a crate is an ADR
//! (GOVERNANCE.md), and blueprint §3.2 explains the roles without being parsed.

mod common;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use toml::Value;

/// Package directories Cargo resolves as workspace members.
fn cargo_member_dirs(root: &Path) -> BTreeSet<PathBuf> {
    let metadata = common::membership_metadata_at(root);
    metadata
        .workspace_packages()
        .into_iter()
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
fn crate_directories_and_declared_members_match_cargo() {
    let root = common::workspace_root();
    let members = cargo_member_dirs(&root);
    let unregistered = unregistered(&common::crate_dirs_at(&root), &members);
    assert!(
        unregistered.is_empty(),
        "crates/ contains directories Cargo does not build as workspace members: \
         {unregistered:?}. Add the crate through an ADR (GOVERNANCE.md) or remove the \
         directory."
    );
    let declared: BTreeSet<PathBuf> = common::member_dirs_at(&root)
        .into_iter()
        .map(|dir| {
            dir.canonicalize()
                .unwrap_or_else(|err| panic!("resolving {}: {err}", dir.display()))
        })
        .collect();
    assert_eq!(
        declared, members,
        "[workspace] members in Cargo.toml and Cargo's workspace members differ"
    );
}

#[test]
fn unregistered_crate_directory_is_detected() {
    let stray =
        std::env::temp_dir().join(format!("pse-governance-stray-crate-{}", std::process::id()));
    let member = stray.join("known-member");
    std::fs::create_dir_all(&member).expect("creating the control directory");
    let members = BTreeSet::from([member.canonicalize().unwrap()]);
    let dirs = vec![
        ("known-member".to_owned(), member),
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
fn every_member_inherits_workspace_metadata() {
    let root = common::workspace_root();
    let mut problems: Vec<String> = Vec::new();

    for dir in common::member_dirs_at(&root) {
        let manifest_path = dir.join("Cargo.toml");
        let manifest = common::parse_toml(&manifest_path);
        let rel = common::rel_at(&root, &manifest_path);

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
