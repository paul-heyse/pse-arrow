// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Delta's development revision and its separately pinned kernel form one contract.
#![allow(
    clippy::unwrap_used,
    reason = "governance assertions on required manifest keys"
)]
mod common;

#[test]
fn delta_and_kernel_resolve_to_the_declared_full_revisions() {
    let manifest = common::root_manifest();
    let delta = manifest["workspace"]["dependencies"]["deltalake"]["rev"]
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
        let source = packages[0].source.as_ref().unwrap().to_string();
        assert!(
            source.ends_with(&format!("#{revision}")),
            "{name}: {source} differs from declared revision"
        );
    }
}
