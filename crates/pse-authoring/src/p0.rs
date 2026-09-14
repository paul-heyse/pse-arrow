// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P0 resolves the actual exact-version package dependency graph before identity reuse.

use crate::{AuthoringError, document::DocumentBundle};
use pse_ids::SemanticId;
use pse_relations::generated::{authored, normalized};
use pse_schema::Registry;
use std::collections::{BTreeMap, BTreeSet};

/// Admitted graph rows, in stable package identity order.
#[derive(Clone, Debug, PartialEq)]
pub struct PackageGraph {
    /// Complete dependencies and their dependency-first depth.
    pub rows: Vec<normalized::package_graph::Row>,
}

/// Resolve all supplied packages, rejecting missing, cyclic or incompatible dependencies.
///
/// # Errors
/// Versions must be exact semantic versions. Duplicate packages and dependencies,
/// missing pins, mismatched versions and dependency cycles are typed failures.
pub fn resolve(
    bundles: &[DocumentBundle],
    registry: &Registry,
) -> Result<PackageGraph, AuthoringError> {
    resolve_headers(
        &bundles
            .iter()
            .map(|bundle| bundle.package.clone())
            .collect::<Vec<_>>(),
        registry,
    )
}

/// Resolve a complete explicit header inventory, as used by the compiler's P0 adapter.
///
/// # Errors
/// The same graph contract failures as [`resolve`].
pub fn resolve_headers(
    headers: &[authored::packages::Row],
    registry: &Registry,
) -> Result<PackageGraph, AuthoringError> {
    let mut packages = BTreeMap::new();
    for header in headers {
        version(&header.version)?;
        if packages.insert(header.package_id, header).is_some() {
            return Err(contract("duplicate package identity"));
        }
    }
    validate_dependencies(&packages)?;
    let depth = depths(&packages)?;
    let rows = packages
        .values()
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
                depth: depth[&package.package_id],
                dependency_package_ids: dependencies,
                derivation_id: pse_ids::named_id(package.package_id, "pass:P0@1:package_graph"),
            }
        })
        .collect::<Vec<_>>();
    let spec = registry
        .relation("normalized.package_graph")
        .ok_or_else(|| contract("missing normalized.package_graph"))?;
    pse_relations::cells::batch_from_cells(
        registry,
        spec,
        &rows
            .iter()
            .cloned()
            .map(normalized::package_graph::Row::into_cells)
            .collect::<Vec<_>>(),
    )
    .map_err(|error| contract(&error.to_string()))?;
    Ok(PackageGraph { rows })
}

fn validate_dependencies(
    packages: &BTreeMap<SemanticId, &authored::packages::Row>,
) -> Result<(), AuthoringError> {
    let mut requirements = BTreeMap::<SemanticId, semver::Version>::new();
    for package in packages.values() {
        let mut seen = BTreeSet::new();
        for dependency in &package.dependencies {
            if !seen.insert(dependency.package_id) {
                return Err(contract("duplicate dependency package identity"));
            }
            let required = version(
                dependency
                    .version_req
                    .strip_prefix('=')
                    .unwrap_or(&dependency.version_req),
            )?;
            if let Some(previous) = requirements.insert(dependency.package_id, required.clone())
                && previous != required
            {
                return Err(AuthoringError::PackageVersionConflict {
                    dependency: dependency.package_id.to_string(),
                    first: previous.to_string(),
                    second: required.to_string(),
                });
            }
            let target = packages.get(&dependency.package_id).ok_or_else(|| {
                AuthoringError::PackageUnresolved {
                    name: package.name.clone(),
                    dependency: dependency.package_id.to_string(),
                }
            })?;
            if version(&target.version)? != required {
                return Err(AuthoringError::PackageVersionConflict {
                    dependency: target.name.clone(),
                    first: required.to_string(),
                    second: target.version.clone(),
                });
            }
        }
    }
    Ok(())
}

fn depths(
    packages: &BTreeMap<SemanticId, &authored::packages::Row>,
) -> Result<BTreeMap<SemanticId, u16>, AuthoringError> {
    let mut depths = BTreeMap::new();
    while depths.len() < packages.len() {
        let before = depths.len();
        for (id, package) in packages {
            if depths.contains_key(id) {
                continue;
            }
            let parents = package
                .dependencies
                .iter()
                .map(|dependency| depths.get(&dependency.package_id).copied())
                .collect::<Option<Vec<u16>>>();
            if let Some(parents) = parents {
                let depth = parents
                    .into_iter()
                    .max()
                    .map(|depth| {
                        depth
                            .checked_add(1)
                            .ok_or_else(|| contract("package dependency depth exceeds UInt16"))
                    })
                    .transpose()?
                    .unwrap_or(0);
                depths.insert(*id, depth);
            }
        }
        if depths.len() == before {
            return Err(contract("cyclic package dependency"));
        }
    }
    Ok(depths)
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
