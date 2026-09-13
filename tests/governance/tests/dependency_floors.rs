// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! Every package in the resolved graph declares a `rust-version` no higher than ours.
//!
//! Blueprint §3.1 asks for the pre-1.0 crates' declared floors to be "re-checked at each
//! upgrade". This is that check, and it runs on every PR rather than at upgrade time: a
//! transitive bump that raises its MSRV past ours is a build failure for the next person
//! on a clean machine, and cargo's own resolver only reports it when it cannot resolve.

mod common;

use toml::Value;

/// Normalises `1.85`, `^1.85`, `1.85.0` to a comparable `semver::Version`.
fn floor(text: &str) -> Option<semver::Version> {
    let cleaned = text.trim().trim_start_matches(['^', '=', '~', '>']).trim();
    let mut parts: Vec<&str> = cleaned.split('.').collect();
    while parts.len() < 3 {
        parts.push("0");
    }
    semver::Version::parse(&parts[..3].join(".")).ok()
}

#[test]
fn no_dependency_declares_a_higher_msrv_than_the_workspace() {
    let manifest = common::root_manifest();
    let ours_text = common::dig(&manifest, &["workspace", "package", "rust-version"])
        .and_then(Value::as_str)
        .expect("[workspace.package] rust-version");
    let ours = floor(ours_text).expect("workspace rust-version parses");

    let metadata = common::metadata();
    let mut offenders: Vec<String> = metadata
        .packages
        .iter()
        .filter_map(|pkg| {
            let declared = pkg.rust_version.as_ref()?;
            let parsed = floor(&declared.to_string())?;
            (parsed > ours).then(|| format!("{} {} needs rustc {parsed}", pkg.name, pkg.version))
        })
        .collect();
    offenders.sort();
    offenders.dedup();

    assert!(
        offenders.is_empty(),
        "packages in the resolved graph declare a rust-version above ours ({ours}):\n  {}\n\
         Either pin the dependency back, or raise rust-toolchain.toml and \
         [workspace.package].rust-version together (ADR-0018).",
        offenders.join("\n  ")
    );
}
