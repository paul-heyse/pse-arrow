// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a governance test reports by panicking with the offending path; the workspace panic policy governs library code"
)]

//! `pse-ids` is the only workspace crate with a direct dependency on `blake3`.
//!
//! Identity is a contract, not a convenience: every 128-bit ID comes from a `derive_key`
//! context (blueprint §5.1). The same crate owns `pse.canon.v2` logical hashing and
//! encoded integrity checksums (§5.3). A second crate hashing on its own
//! would invent another identity contract, so the dependency itself is fenced.

mod common;

/// The crate that owns hashing (blueprint §3.1 "`blake3` ... **`pse-ids`** (sole hasher)").
const OWNER: &str = "pse-ids";

#[test]
fn only_pse_ids_depends_on_blake3() {
    let metadata = common::metadata();
    let members: Vec<&cargo_metadata::Package> = metadata
        .packages
        .iter()
        .filter(|pkg| metadata.workspace_members.contains(&pkg.id))
        .collect();
    assert!(
        !members.is_empty(),
        "cargo metadata returned no workspace members"
    );

    let offenders: Vec<String> = members
        .iter()
        .filter(|pkg| pkg.name.as_str() != OWNER)
        .filter(|pkg| pkg.dependencies.iter().any(|dep| dep.name == "blake3"))
        .map(|pkg| pkg.name.to_string())
        .collect();

    assert!(
        offenders.is_empty(),
        "crates other than `{OWNER}` depend on blake3 directly: {offenders:?}. Ask pse-ids \
         for the hash instead (blueprint §5.1)."
    );
}

#[test]
fn pse_ids_still_owns_blake3() {
    let metadata = common::metadata();
    let owner = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name.as_str() == OWNER)
        .expect("pse-ids is a workspace member");
    assert!(
        owner.dependencies.iter().any(|dep| dep.name == "blake3"),
        "`{OWNER}` no longer depends on blake3; either the hasher moved (an ADR) or the \
         dependency was dropped by accident"
    );
}
