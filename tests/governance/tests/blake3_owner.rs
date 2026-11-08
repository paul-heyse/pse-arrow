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
//! context (blueprint §5.1). The same crate owns encoded integrity checksums
//! (§5.3). A second crate hashing on its own
//! would invent another identity contract, so the dependency itself is fenced.

mod common;

/// The crate that owns hashing (blueprint §3.1 "`blake3` ... **`pse-ids`** (sole hasher)").
const OWNER: &str = "pse-ids";

fn ownership<'a>(
    packages: impl IntoIterator<Item = (bool, &'a str, bool)>,
) -> (bool, Vec<&'a str>) {
    let mut owns = false;
    let mut offenders = Vec::new();
    for (member, name, depends_on_blake3) in packages {
        if member && depends_on_blake3 {
            if name == OWNER {
                owns = true;
            } else {
                offenders.push(name);
            }
        }
    }
    (owns, offenders)
}

#[test]
fn only_pse_ids_depends_on_blake3() {
    let metadata = common::membership_metadata_at(&common::workspace_root());
    let (owns, offenders) = ownership(metadata.packages.iter().map(|pkg| {
        (
            metadata.workspace_members.contains(&pkg.id),
            pkg.name.as_str(),
            pkg.dependencies.iter().any(|dep| dep.name == "blake3"),
        )
    }));
    assert!(
        owns && offenders.is_empty(),
        "sole hasher policy (blueprint §5.1): `{OWNER}` declares blake3: {owns}; \
         competing workspace declarations: {offenders:?}"
    );
}

#[test]
fn only_workspace_declarations_can_satisfy_or_violate_ownership() {
    assert_eq!(ownership([]), (false, vec![]));
    assert_eq!(ownership([(false, OWNER, true)]), (false, vec![]));
    assert_eq!(ownership([(true, OWNER, false)]), (false, vec![]));
    assert_eq!(
        ownership([(true, OWNER, true), (true, "other", true)]),
        (true, vec!["other"])
    );
}
