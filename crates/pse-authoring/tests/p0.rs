// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P0 checks actual exact versions and dependency edges, independently of digests.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "fixture assertions"
)]

use pse_ids::{ContentHash, SemanticId};
use pse_relations::generated::{
    authored,
    enums::{IdPolicy, PackageKind},
};
fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn package(value: u8) -> authored::packages::Row {
    authored::packages::Row {
        package_id: id(value),
        name: format!("package_{value}"),
        version: "1.0.0".to_owned(),
        kind: PackageKind::Model,
        id_policy: IdPolicy::Explicit,
        dependencies: vec![],
        content_hash: ContentHash::from_bytes([0; 32]),
        doc: String::new(),
    }
}
fn depends(package: &mut authored::packages::Row, target: u8, version: &str) {
    package
        .dependencies
        .push(authored::packages::AuthoredPackagesFieldDependenciesItem {
            package_id: id(target),
            version_req: version.to_owned(),
        });
}
#[test]
fn identical_hashes_do_not_hide_changed_versions_missing_packages_or_cycles() {
    let registry = pse_schema::registry().unwrap();
    let mut first = package(1);
    let second = package(2);
    depends(&mut first, 2, "=1.0.0");
    let graph =
        pse_authoring::p0::resolve_headers(&[first.clone(), second.clone()], registry).unwrap();
    assert_eq!(graph.rows[0].depth, 1);
    assert_eq!(graph.rows[0].dependency_package_ids, vec![id(2)]);
    let mut changed = second.clone();
    changed.version = "2.0.0".to_owned();
    assert!(pse_authoring::p0::resolve_headers(&[first.clone(), changed], registry).is_err());
    assert!(pse_authoring::p0::resolve_headers(&[first.clone()], registry).is_err());
    let mut cyclic = second.clone();
    depends(&mut cyclic, 1, "1.0.0");
    assert!(pse_authoring::p0::resolve_headers(&[first.clone(), cyclic], registry).is_err());
    first.dependencies[0].version_req = "^1.0.0".to_owned();
    assert!(pse_authoring::p0::resolve_headers(&[first, second], registry).is_err());
}
