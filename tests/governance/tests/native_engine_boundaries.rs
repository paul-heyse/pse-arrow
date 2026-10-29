// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "governance tests report invalid metadata and forbidden ownership by panicking"
)]

//! N06 dependency and deletion boundaries, observed from Cargo's resolved graph.
//! Normal/build reach excludes dev-only edges; testkit declarations remain dev-only.

mod common;

use cargo_metadata::{DependencyKind, Metadata, PackageId};
use std::collections::{BTreeMap, BTreeSet};

fn runtime_dependencies(metadata: &Metadata, start: &str) -> BTreeSet<String> {
    let package = metadata.workspace_packages().into_iter().find(|package| package.name.as_str() == start).unwrap_or_else(|| panic!("required workspace owner {start} is absent from Cargo metadata"));
    let packages: BTreeMap<_, _> = metadata.packages.iter().map(|package| (&package.id, package)).collect();
    let graph = metadata.resolve.as_ref().expect("N06 requires resolved Cargo metadata, not --no-deps");
    let nodes: BTreeMap<_, _> = graph.nodes.iter().map(|node| (&node.id, node)).collect();
    let mut reached = BTreeSet::<PackageId>::new();
    let mut pending = vec![package.id.clone()];
    while let Some(id) = pending.pop() {
        if !reached.insert(id.clone()) { continue; }
        let node = nodes.get(&id).unwrap_or_else(|| panic!("Cargo resolve has no node for {id}"));
        for edge in &node.deps {
            if edge.dep_kinds.iter().any(|kind| kind.kind != DependencyKind::Development) { pending.push(edge.pkg.clone()); }
        }
    }
    reached.iter().map(|id| packages.get(id).unwrap_or_else(|| panic!("Cargo resolve references an absent package {id}")).name.to_string()).collect()
}

#[test]
fn native_engine_owners_keep_resolved_dependency_and_retirement_boundaries() {
    let metadata = common::metadata();
    let mut failures = Vec::new();
    for owner in ["pse-engine", "pse-testkit", "pse-admission"] {
        let forbidden: Vec<_> = runtime_dependencies(&metadata, owner).into_iter().filter(|name| {
            ["pse-catalog", "pse-runtime", "pse-compiler", "pse-operations", "pse-math"].contains(&name.as_str())
                || ["deltalake", "pse-backend", "pse-ipopt"].iter().any(|prefix| name.starts_with(prefix))
        }).collect();
        if !forbidden.is_empty() { failures.push(format!("{owner} reaches forbidden product/storage dependencies: {forbidden:?}")); }
    }
    let members = metadata.workspace_packages();
    assert!(!members.is_empty(), "Cargo metadata returned no workspace members");
    for package in &members {
        for dependency in &package.dependencies {
            if dependency.name == "pse-testkit" && dependency.kind != DependencyKind::Development {
                failures.push(format!("{} has a production/build testkit edge", package.name));
            }
        }
    }
    for name in ["pse-engine", "pse-ids", "pse-diagnostics", "pse-schema", "pse-relations"] {
        let package = members.iter().find(|package| package.name.as_str() == name).unwrap_or_else(|| panic!("required workspace owner {name} is absent"));
        if package.dependencies.iter().any(|dependency| dependency.name == "pse-testkit") {
            failures.push(format!("{name} creates a testkit dependency cycle"));
        }
    }
    let root = common::workspace_root();
    for relative in ["crates/pse-catalog/Cargo.toml", "crates/pse-operations-queries/Cargo.toml", "crates/pse-runtime/src/reserve.rs", "tests/support/session_factory.rs"] {
        if root.join(relative).exists() { failures.push(format!("superseded owner remains: {relative}")); }
    }
    assert!(failures.is_empty(), "N06 dependency/deletion checks: {} failures; baseline 0\n{}", failures.len(), failures.join("\n"));
}
