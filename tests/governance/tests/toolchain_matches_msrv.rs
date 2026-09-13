// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! `rust-toolchain.toml`'s channel equals `[workspace.package].rust-version`.
//!
//! ADR-0018 makes the MSRV the toolchain we actually compile with, rather than
//! DataFusion's own 1.94.0 floor: every crate is `publish = false` through phase 1, so no
//! consumer needs a lower one, and a declared floor nothing exercises is a claim, not a
//! fact. The two therefore move together, and this is what makes "together" mechanical.

mod common;

#[test]
fn toolchain_channel_equals_workspace_rust_version() {
    let root = common::workspace_root();

    let toolchain = common::parse_toml(&root.join("rust-toolchain.toml"));
    let channel = common::dig(&toolchain, &["toolchain", "channel"])
        .and_then(toml::Value::as_str)
        .expect("rust-toolchain.toml [toolchain] channel");

    let manifest = common::root_manifest();
    let rust_version = common::dig(&manifest, &["workspace", "package", "rust-version"])
        .and_then(toml::Value::as_str)
        .expect("Cargo.toml [workspace.package] rust-version");

    assert_eq!(
        channel, rust_version,
        "rust-toolchain.toml channel and [workspace.package].rust-version must be equal \
         (ADR-0018). Bump both in the same commit."
    );
}
