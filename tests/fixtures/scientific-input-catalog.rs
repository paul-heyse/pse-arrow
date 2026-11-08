// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Lightweight authored scientific declarations for capture and passive source reach.

use miette::{IntoDiagnostic as _, Result, WrapErr as _, bail, ensure};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path},
};

const MAX_FAMILIES: usize = 4096;
const MAX_FAMILY_DEPTH: usize = 64;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Catalog {
    version: u32,
    pub(crate) package_inventory: String,
    pub(crate) families: Vec<Family>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Family {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) extends: Option<String>,
    #[serde(default)]
    pub(crate) roots: Vec<String>,
    #[serde(default)]
    pub(crate) documents: Vec<String>,
    pub(crate) consumers: Vec<String>,
    #[serde(default)]
    pub(crate) projection: Option<String>,
    #[serde(default)]
    pub(crate) substitutions: Vec<String>,
    #[serde(default)]
    pub(crate) manifest: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct Manifest {
    version: u32,
    pub(crate) runs: Vec<Run>,
    #[serde(default)]
    pub(crate) settings: ManifestSettings,
}

#[derive(Default, Deserialize)]
pub(crate) struct ManifestSettings {
    pub(crate) diagnostics: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct Run {
    pub(crate) package: String,
    #[serde(default)]
    pub(crate) dependencies: Vec<String>,
    pub(crate) physical: String,
}

impl Run {
    pub(crate) fn roots(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.package.as_str())
            .chain(self.dependencies.iter().map(String::as_str))
            .chain(std::iter::once(self.physical.as_str()))
    }
}

impl Catalog {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(bytes).into_diagnostic()?;
        let catalog: Self = toml::from_str(text).into_diagnostic()?;
        ensure!(
            catalog.version == 1,
            "unsupported scientific input catalog version"
        );
        ensure!(
            catalog.families.len() <= MAX_FAMILIES,
            "scientific family count exceeds bound"
        );
        Ok(catalog)
    }
}

impl Manifest {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(bytes).into_diagnostic()?;
        let manifest: Self = toml::from_str(text).into_diagnostic()?;
        ensure!(
            manifest.version == 1 && !manifest.runs.is_empty(),
            "unsupported or empty scientific run manifest"
        );
        Ok(manifest)
    }
}

pub(crate) fn resolve_families(families: Vec<Family>, selection: Option<&str>) -> Result<Vec<Family>> {
    let mut declared = BTreeMap::new();
    for family in families {
        ensure!(
            declared.insert(family.name.clone(), family).is_none(),
            "duplicate scientific family name"
        );
    }
    let mut resolved = BTreeMap::new();
    if let Some(name) = selection {
        ensure!(declared.contains_key(name), "unknown scientific family {name}");
        resolve_family(name, &declared, &mut resolved, &mut BTreeSet::new())?;
        return Ok(vec![resolved.remove(name).wrap_err("selected family was not resolved")?]);
    }
    for name in declared.keys() {
        resolve_family(name, &declared, &mut resolved, &mut BTreeSet::new())?;
    }
    Ok(resolved.into_values().collect())
}

fn resolve_family(
    name: &str,
    declared: &BTreeMap<String, Family>,
    resolved: &mut BTreeMap<String, Family>,
    active: &mut BTreeSet<String>,
) -> Result<()> {
    if resolved.contains_key(name) {
        return Ok(());
    }
    ensure!(
        active.insert(name.to_owned()),
        "scientific family inheritance cycle at {name}"
    );
    ensure!(
        active.len() <= MAX_FAMILY_DEPTH,
        "scientific family inheritance exceeds depth bound"
    );
    let mut family = declared
        .get(name)
        .wrap_err_with(|| format!("missing scientific base family {name}"))?
        .clone();
    if let Some(base) = &family.extends {
        resolve_family(base, declared, resolved, active)?;
        let parent = &resolved[base];
        family.roots.extend(parent.roots.iter().cloned());
        family.documents.extend(parent.documents.iter().cloned());
        family
            .substitutions
            .extend(parent.substitutions.iter().cloned());
        if family.projection.is_none() {
            family.projection = parent.projection.clone();
        }
        if family.manifest.is_none() {
            family.manifest = parent.manifest.clone();
        }
    }
    family.roots = family
        .roots
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    family.documents = family
        .documents
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    family.substitutions = family
        .substitutions
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    family.extends = None;
    resolved.insert(name.to_owned(), family);
    active.remove(name);
    Ok(())
}

pub(crate) fn normalize(path: &Path) -> Result<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                parts.push(part.to_str().wrap_err("non-UTF8 scientific input path")?)
            }
            Component::CurDir => (),
            Component::ParentDir => {
                ensure!(parts.pop().is_some(), "scientific path escapes repository");
            }
            _ => {
                bail!("scientific input path must be repository-relative");
            }
        }
    }
    ensure!(!parts.is_empty(), "empty scientific input path");
    Ok(parts.join("/"))
}
