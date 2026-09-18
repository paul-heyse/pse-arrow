// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Delta's development revision and its separately pinned kernel form one contract.
#![allow(
    clippy::unwrap_used,
    reason = "governance assertions on required manifest keys"
)]
mod common;

#[test]
fn delta_override_hashes_match_the_selected_source_and_patch() {
    let root = common::workspace_root();
    let vendor = root.join("vendor/delta-rs");
    let provenance: serde_json::Value =
        serde_json::from_slice(&std::fs::read(vendor.join("PROVENANCE.json")).unwrap()).unwrap();
    let patch = std::fs::read(root.join("tooling/delta-native-seams.patch")).unwrap();
    assert_eq!(sha256(&patch), provenance["patch_sha256"].as_str().unwrap());
    for (path, expected) in provenance["vendored_sha256"].as_object().unwrap() {
        assert!(!path.contains("..") && !path.starts_with('/'));
        let bytes = std::fs::read(vendor.join(path)).unwrap();
        assert_eq!(
            sha256(&bytes),
            expected.as_str().unwrap(),
            "source drift: {path}"
        );
    }
}

#[test]
fn delta_and_kernel_resolve_to_the_declared_full_revisions() {
    let manifest = common::root_manifest();
    let delta = manifest["workspace"]["metadata"]["pse"]["delta-source-revision"]
        .as_str()
        .unwrap();
    let kernel = manifest["workspace"]["metadata"]["pse"]["delta-kernel-revision"]
        .as_str()
        .unwrap();
    for revision in [delta, kernel] {
        assert_eq!(
            revision.len(),
            40,
            "development dependencies require full revisions"
        );
        assert!(revision.bytes().all(|b| b.is_ascii_hexdigit()));
    }
    let metadata = common::metadata();
    for (name, revision) in [
        ("deltalake-core", delta),
        ("deltalake-derive", delta),
        ("buoyant_kernel", kernel),
        ("buoyant_kernel_derive", kernel),
        ("buoyant_kernel_engine", kernel),
    ] {
        let packages: Vec<_> = metadata
            .packages
            .iter()
            .filter(|p| p.name.as_str() == name)
            .collect();
        assert_eq!(
            packages.len(),
            1,
            "{name} must have exactly one resolved package"
        );
        if name.starts_with("deltalake-") {
            assert!(
                packages[0].source.is_none(),
                "Delta source override must be repository-owned"
            );
            let path = packages[0].manifest_path.as_std_path();
            assert!(path.starts_with(common::workspace_root().join("vendor/delta-rs")));
            let provenance: serde_json::Value = serde_json::from_slice(
                &std::fs::read(common::workspace_root().join("vendor/delta-rs/PROVENANCE.json"))
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(provenance["revision"].as_str(), Some(revision));
            continue;
        }
        let source = packages[0].source.as_ref().unwrap().to_string();
        assert!(
            source.ends_with(&format!("#{revision}")),
            "{name}: {source} differs from declared revision"
        );
    }
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write;
    let mut digest = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(digest, "{byte:02x}").unwrap();
    }
    digest
}
