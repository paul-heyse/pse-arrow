// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact package resolution over a complete typed dependency inventory.
use crate::AuthoringError;
use pse_ids::SemanticId;
use pse_model::generated::{authored, normalized};
use pse_structural::projection::{Dependency, GraphLimits, Projection, Scope};
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
    limits
        .check(headers.len(), edge_count)
        .map_err(|error| graph_error(&error))?;
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
            let required = dependency
                .version_req
                .strip_prefix('=')
                .unwrap_or(&dependency.version_req);
            version(required)?;
            let target = packages
                .get(&dependency.package_id)
                .ok_or_else(|| contract("package dependency target absent"))?;
            if required != target.version {
                return Err(contract("package exact version requirement differs"));
            }
            let mut hash = pse_ids::FramedHasher::new("pse:package-edge:v1");
            hash.id(&dependency.package_id)
                .id(&header.package_id)
                .str("");
            edges.push(Dependency {
                id: hash.finish_id(),
                from: dependency.package_id,
                to: header.package_id,
            });
        }
    }
    let graph = Projection::admit(
        Scope::Whole(SemanticId::NIL),
        packages.keys().copied().collect(),
        edges,
        limits,
    )
    .map_err(|error| graph_error(&error))?;
    let mut depths = BTreeMap::<SemanticId, u16>::new();
    for id in graph.order().map_err(|error| graph_error(&error))? {
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
                derivation_id: pse_ids::named_id(package.package_id, "pass:P0@1:package_graph"),
            }
        })
        .collect())
}

fn graph_error(error: &pse_structural::projection::ProjectionError) -> AuthoringError {
    contract(&error.to_string())
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
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
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
                        version_req: "=1.0.0".into(),
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
        let mut invalid = package(2, &[1]);
        invalid.dependencies[0].version_req = "^1.0".into();
        assert!(resolve_rows(&[package(1, &[]), invalid], limits).is_err());
    }
}
