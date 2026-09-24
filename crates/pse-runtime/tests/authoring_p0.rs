// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P0 checks actual exact versions and dependency edges, independently of digests.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "fixture assertions"
)]

use pse_columnar::CancellationToken;
use pse_ids::{ContentHash, SemanticId};
#[path = "authoring_support/mod.rs"]
mod support;
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
    let registry_owner = support::registry();
    let session = support::session(
        std::sync::Arc::clone(&registry_owner),
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(512 << 20)),
    );
    let mut first = package(1);
    let second = package(2);
    depends(&mut first, 2, "=1.0.0");
    let graph = resolve(&[first.clone(), second.clone()], &session).unwrap();
    assert_eq!(graph[0].depth, 1);
    assert_eq!(graph[0].dependency_package_ids, vec![id(2)]);
    let mut changed = second.clone();
    changed.version = "2.0.0".to_owned();
    assert!(resolve(&[first.clone(), changed], &session).is_err());
    assert!(resolve(&[first.clone()], &session).is_err());
    let mut cyclic = second.clone();
    depends(&mut cyclic, 1, "1.0.0");
    assert!(resolve(&[first.clone(), cyclic], &session).is_err());
    first.dependencies[0].version_req = "^1.0.0".to_owned();
    assert!(resolve(&[first, second], &session).is_err());
}

#[test]
fn package_dependency_resolution_keeps_the_callers_input_role_distinct() {
    let session = support::session(
        support::registry(),
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(512 << 20)),
    );
    let mut existing = authored::packages::Builder::with_registry(session.registry(), 1).unwrap();
    existing.push(package(99)).unwrap();
    let session = session
        .with_checked_role_inputs(
            std::collections::BTreeMap::from([("packages".to_owned(), existing.finish().unwrap())]),
            &CancellationToken::new(),
        )
        .unwrap();
    let mut first = package(1);
    depends(&mut first, 2, "=1.0.0");
    let graph = resolve(&[first, package(2)], &session).unwrap();
    assert_eq!(graph.len(), 2);
    assert_eq!(graph[0].package_id, id(1));
    assert_eq!(graph[0].dependency_package_ids, vec![id(2)]);
    assert_eq!(
        session
            .input_roles()
            .map(|(role, _)| role)
            .collect::<Vec<_>>(),
        ["packages"]
    );
}

fn resolve(
    rows: &[authored::packages::Row],
    session: &pse_engine::session::EngineSession,
) -> Result<
    Vec<pse_relations::generated::normalized::package_graph::Row>,
    pse_runtime::authoring_driver::DriverError,
> {
    let mut builder = authored::packages::Builder::with_registry(session.registry(), rows.len())?;
    for row in rows {
        builder.push(row.clone())?;
    }
    let graph = pse_runtime::authoring_driver::p0::resolve(
        &builder.finish()?,
        session,
        &CancellationToken::new(),
    )?;
    Ok(pse_relations::generated::normalized::package_graph::View::from_checked(&graph)?.rows()?)
}
