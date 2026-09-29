// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Enforce the resolved normal closure, including transitive native back-edges.
//!
//! The cargo-hakari workspace-hack (ADR-0122) is a feature-only edge: a member cannot name
//! its dependencies, which exist only to unify features. The closure does not follow it.
use anyhow::{Context, Result};
use cargo_metadata::{DependencyKind, Metadata, PackageId};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The workspace-hack package named by `.config/hakari.toml`, its one declaration.
fn feature_only_package(root: &Path) -> Result<String> {
    let path = root.join(".config/hakari.toml");
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let config: toml::Table =
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
    config
        .get("hakari-package")
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .with_context(|| format!("{} names no hakari-package", path.display()))
}

pub(crate) fn check(root: &Path, metadata: &Metadata) -> Result<Vec<String>> {
    let feature_only = feature_only_package(root)?;
    let profiles = metadata.workspace_metadata["pse"]["dependency-ceilings"]
        .as_object()
        .context("missing dependency ceiling declarations")?;
    let packages: BTreeMap<_, _> = metadata.packages.iter().map(|p| (&p.id, p)).collect();
    let nodes: BTreeMap<_, _> = metadata
        .resolve
        .as_ref()
        .context("missing resolved dependency graph")?
        .nodes
        .iter()
        .map(|n| (&n.id, n))
        .collect();
    let mut failures = Vec::new();
    for (profile, declaration) in profiles {
        let roots: Vec<String> = serde_json::from_value(declaration["roots"].clone())?;
        let forbidden: Vec<String> = serde_json::from_value(declaration["forbidden"].clone())?;
        for root in roots {
            let package = metadata
                .workspace_members
                .iter()
                .find(|id| packages[id].name.as_str() == root)
                .with_context(|| format!("missing ceiling root {root}"))?;
            let mut pending = vec![package];
            let mut visited = BTreeSet::<&PackageId>::new();
            while let Some(id) = pending.pop() {
                if !visited.insert(id) {
                    continue;
                }
                let package = packages[id];
                if forbidden
                    .iter()
                    .any(|pattern| matches(pattern, package.name.as_str()))
                {
                    failures.push(format!(
                        "{profile}: {root} normal closure contains {}",
                        package.name
                    ));
                }
                for edge in &nodes[id].deps {
                    if edge
                        .dep_kinds
                        .iter()
                        .any(|kind| kind.kind == DependencyKind::Normal)
                        && packages[&edge.pkg].name.as_str() != feature_only
                        && !packages[&edge.pkg]
                            .targets
                            .iter()
                            .any(cargo_metadata::Target::is_proc_macro)
                    {
                        pending.push(&edge.pkg);
                    }
                }
            }
            println!(
                "ceiling {profile}: {root} ({} normal packages)",
                visited.len()
            );
        }
    }
    Ok(failures)
}
fn matches(pattern: &str, name: &str) -> bool {
    pattern
        .strip_suffix('*')
        .map_or(name == pattern, |prefix| name.starts_with(prefix))
}
