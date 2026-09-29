// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! `rust-toolchain.toml` pins a dated nightly at or above `[workspace.package].rust-version`.
//!
//! ADR-0122 pins one dated nightly for its Cargo build-sharing features; the source uses no
//! nightly language or library feature. `rust-version` is therefore the stable language
//! floor the source is written against: Cargo's MSRV-aware resolver and clippy's
//! `incompatible_msrv` read it, and `dependency_floors` holds every resolved package to it.
//! The pin must be dated, so it cannot move silently, and the toolchain that runs this
//! test -- the pinned one -- must be at or above the floor.

mod common;

use std::process::Command;

/// `major.minor.patch` of a version, ignoring any pre-release such as `-nightly`.
fn release(text: &str) -> semver::Version {
    let core = text.trim().split('-').next().unwrap_or_default();
    let mut parts: Vec<&str> = core.split('.').collect();
    while parts.len() < 3 {
        parts.push("0");
    }
    semver::Version::parse(&parts[..3].join("."))
        .unwrap_or_else(|err| panic!("`{text}` is not a Rust release: {err}"))
}

/// True for a channel of the form `nightly-YYYY-MM-DD`.
fn dated_nightly(channel: &str) -> bool {
    channel.strip_prefix("nightly-").is_some_and(|date| {
        let fields: Vec<&str> = date.split('-').collect();
        fields.len() == 3
            && [4, 2, 2]
                .iter()
                .zip(&fields)
                .all(|(len, field)| field.len() == *len && field.bytes().all(|b| b.is_ascii_digit()))
    })
}

#[test]
fn toolchain_is_a_dated_nightly_at_or_above_the_language_floor() {
    let root = common::workspace_root();

    let toolchain = common::parse_toml(&root.join("rust-toolchain.toml"));
    let channel = common::dig(&toolchain, &["toolchain", "channel"])
        .and_then(toml::Value::as_str)
        .expect("rust-toolchain.toml [toolchain] channel");
    assert!(
        dated_nightly(channel),
        "rust-toolchain.toml channel `{channel}` is not a dated nightly (nightly-YYYY-MM-DD); \
         an undated channel moves silently (ADR-0122)."
    );

    let manifest = common::root_manifest();
    let floor = common::dig(&manifest, &["workspace", "package", "rust-version"])
        .and_then(toml::Value::as_str)
        .expect("Cargo.toml [workspace.package] rust-version");

    // Run from the workspace root, rustup resolves the pinned channel: the toolchain that
    // built and runs this test.
    let output = Command::new("rustc")
        .arg("-vV")
        .current_dir(&root)
        .output()
        .expect("running rustc -vV");
    assert!(output.status.success(), "rustc -vV failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual = stdout
        .lines()
        .find_map(|line| line.strip_prefix("release: "))
        .expect("rustc -vV reports its release");

    assert!(
        release(actual) >= release(floor),
        "the pinned toolchain `{channel}` (rustc {actual}) is below \
         [workspace.package].rust-version {floor}; move the pin forward or the floor back \
         (ADR-0122)."
    );
}

#[test]
fn channel_and_release_parsing_controls() {
    assert!(dated_nightly("nightly-2026-09-29"));
    for undated in ["nightly", "stable", "1.98.1", "nightly-2026-9-29", "beta-2026-09-29"] {
        assert!(!dated_nightly(undated), "{undated}");
    }
    assert_eq!(release("1.101.0-nightly"), semver::Version::new(1, 101, 0));
    assert_eq!(release("1.98"), semver::Version::new(1, 98, 0));
    assert!(release("1.101.0-nightly") >= release("1.98.1"));
    assert!(release("1.98.0") < release("1.98.1"));
}
