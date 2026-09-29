// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact package resolution over a complete typed dependency inventory.
use crate::AuthoringError;
use petgraph::{algo::toposort, graph::DiGraph};
use pse_model::generated::identities::PackageId;
use pse_model::generated::{authored, normalized};

/// Explicit bounds on the finite package inventory.
#[derive(Clone, Copy, Debug)]
pub struct GraphLimits {
    /// Maximum package count.
    pub nodes: usize,
    /// Maximum dependency count.
    pub edges: usize,
}
use std::collections::{BTreeMap, BTreeSet};

/// Resolve selected package values without planning or executing a relation.
/// # Errors
/// Invalid exact versions, duplicate/missing packages or dependencies, cycles or bounds.
pub fn resolve_rows(
    headers: &[authored::packages::Row],
    limits: GraphLimits,
) -> Result<Vec<normalized::package_graph::Row>, AuthoringError> {
    let edge_count = headers.iter().try_fold(0usize, |sum, row| {
        sum.checked_add(row.dependencies.len())
            .ok_or_else(|| contract("package dependency extent overflow"))
    })?;
    if headers.len() > limits.nodes || edge_count > limits.edges {
        return Err(contract("package graph exceeds declared bounds"));
    }
    let mut packages = BTreeMap::new();
    for header in headers {
        version(&header.version)?;
        if packages.insert(header.package_id, header).is_some() {
            return Err(contract("duplicate package identity"));
        }
    }
    let mut edges = Vec::with_capacity(edge_count);
    for header in headers {
        let mut seen = BTreeSet::new();
        for dependency in &header.dependencies {
            if !seen.insert(dependency.package_id) {
                return Err(contract("duplicate package dependency"));
            }
            let target = packages
                .get(&dependency.package_id)
                .ok_or_else(|| contract("package dependency target absent"))?;
            // ADR-0123 Outcome 7: the typed requirement admits the target's version.
            if !crate::language::requirement_admits(&dependency.version_req, &target.version) {
                return Err(contract("package exact version requirement differs"));
            }
            edges.push((dependency.package_id, header.package_id));
        }
    }
    let mut graph = DiGraph::<PackageId, ()>::new();
    let nodes = packages
        .keys()
        .map(|id| (*id, graph.add_node(*id)))
        .collect::<BTreeMap<_, _>>();
    edges.sort_unstable();
    for (from, to) in edges {
        graph.add_edge(nodes[&from], nodes[&to], ());
    }
    let order = toposort(&graph, None).map_err(|cycle| {
        contract(&format!(
            "package dependency cycle at {}",
            graph[cycle.node_id()]
        ))
    })?;
    let mut depths = BTreeMap::<PackageId, u16>::new();
    for node in order {
        let id = graph[node];
        let depth = packages[&id]
            .dependencies
            .iter()
            .map(|dependency| depths[&dependency.package_id])
            .max()
            .map(|depth| {
                depth
                    .checked_add(1)
                    .ok_or_else(|| contract("package dependency depth exceeds UInt16"))
            })
            .transpose()?
            .unwrap_or(0);
        depths.insert(id, depth);
    }
    Ok(packages
        .into_values()
        .map(|package| {
            let mut dependencies = package
                .dependencies
                .iter()
                .map(|dependency| dependency.package_id)
                .collect::<Vec<_>>();
            dependencies.sort_unstable();
            normalized::package_graph::Row {
                package_id: package.package_id,
                version: package.version.clone(),
                content_hash: package.content_hash,
                depth: i64::from(depths[&package.package_id]),
                dependency_package_ids: dependencies,
                derivation_id: pse_ids::named_id(
                    package.package_id.as_id(),
                    "pass:P0@1:package_graph",
                ),
            }
        })
        .collect())
}

fn version(text: &str) -> Result<semver::Version, AuthoringError> {
    semver::Version::parse(text)
        .map_err(|error| contract(&format!("exact semantic version required: {error}")))
}
fn contract(reason: &str) -> AuthoringError {
    AuthoringError::Contract {
        at: None,
        reason: reason.to_owned(),
    }
}

#[cfg(test)]
mod computation_unit {
    use super::*;
    fn id(n: u8) -> PackageId {
        PackageId::from_bytes([n; 16])
    }
    fn package(n: u8, dependencies: &[u8]) -> authored::packages::Row {
        authored::packages::Row {
            package_id: id(n),
            name: format!("p{n}"),
            version: "1.0.0".into(),
            kind: "library".parse().unwrap(),
            id_policy: "explicit".parse().unwrap(),
            dependencies: dependencies
                .iter()
                .map(
                    |n| authored::packages::AuthoredPackagesFieldDependenciesItem {
                        package_id: id(*n),
                        version_req: crate::language::exact_requirement("1.0.0").unwrap(),
                    },
                )
                .collect(),
            content_hash: pse_ids::ContentHash::from_bytes([n; 32]),
            doc: String::new(),
        }
    }
    #[test]
    fn exact_packages_diamond_isolate_and_invalid_inventory() {
        let limits = GraphLimits {
            nodes: 100,
            edges: 100,
        };
        let rows = vec![
            package(1, &[]),
            package(2, &[1]),
            package(3, &[1]),
            package(4, &[2, 3]),
            package(5, &[]),
        ];
        let expected = resolve_rows(&rows, limits).unwrap();
        assert_eq!(
            expected.iter().map(|row| row.depth).collect::<Vec<_>>(),
            [0, 1, 1, 2, 0]
        );
        let mut reversed = rows.clone();
        reversed.reverse();
        assert_eq!(resolve_rows(&reversed, limits).unwrap(), expected);
        for rows in [
            vec![package(1, &[2])],
            vec![package(1, &[1])],
            vec![package(1, &[]), package(1, &[])],
            vec![package(1, &[]), package(2, &[1, 1])],
            vec![package(1, &[2]), package(2, &[1])],
        ] {
            assert!(resolve_rows(&rows, limits).is_err());
        }
        // A requirement the target's version does not meet is refused.
        let mut invalid = package(2, &[1]);
        invalid.dependencies[0].version_req = crate::language::exact_requirement("1.0.1").unwrap();
        assert!(resolve_rows(&[package(1, &[]), invalid], limits).is_err());
    }
}
