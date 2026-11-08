// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Lightweight authored scientific declarations for capture and passive source reach.

use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path},
};

const MAX_FAMILIES: usize = 4096;
const MAX_FAMILY_DEPTH: usize = 64;

#[derive(Debug, thiserror::Error)]
pub(crate) enum CatalogError {
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    Toml(#[from] Box<toml::de::Error>),
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    Limit(&'static str),
}

pse_diagnostics::impl_diagnostic! {
    CatalogError,
    code(this) {
        use pse_diagnostics::DiagnosticCode as Code;
        Some(match this {
            Self::Utf8(_) | Self::Toml(_) => Code::AuthoringParse,
            Self::Invalid(_) => Code::AuthoringReferenceContract,
            Self::Limit(_) => Code::RuntimeResourceLimit,
        })
    },
    forward(_this) { None },
    help(_this) { None },
    related(_this) { None },
    source(_this) { None }
}

type Result<T> = std::result::Result<T, CatalogError>;

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
        let text = std::str::from_utf8(bytes)?;
        let catalog: Self = toml::from_str(text).map_err(Box::new)?;
        if catalog.version != 1 {
            return Err(CatalogError::Invalid(
                "unsupported scientific input catalog version".into(),
            ));
        }
        if catalog.families.len() > MAX_FAMILIES {
            return Err(CatalogError::Limit("scientific family count exceeds bound"));
        }
        Ok(catalog)
    }
}

impl Manifest {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(bytes)?;
        let manifest: Self = toml::from_str(text).map_err(Box::new)?;
        if manifest.version != 1 || manifest.runs.is_empty() {
            return Err(CatalogError::Invalid(
                "unsupported or empty scientific run manifest".into(),
            ));
        }
        Ok(manifest)
    }
}

pub(crate) fn resolve_families(
    families: Vec<Family>,
    selection: Option<&str>,
) -> Result<Vec<Family>> {
    let mut declared = BTreeMap::new();
    for family in families {
        if declared.insert(family.name.clone(), family).is_some() {
            return Err(CatalogError::Invalid(
                "duplicate scientific family name".into(),
            ));
        }
    }
    let mut resolved = BTreeMap::new();
    if let Some(name) = selection {
        if !declared.contains_key(name) {
            return Err(CatalogError::Invalid(format!(
                "unknown scientific family {name}"
            )));
        }
        resolve_family(name, &declared, &mut resolved, &mut BTreeSet::new())?;
        return Ok(vec![resolved.remove(name).ok_or_else(|| {
            CatalogError::Invalid("selected family was not resolved".into())
        })?]);
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
    if !active.insert(name.to_owned()) {
        return Err(CatalogError::Invalid(format!(
            "scientific family inheritance cycle at {name}"
        )));
    }
    if active.len() > MAX_FAMILY_DEPTH {
        return Err(CatalogError::Limit(
            "scientific family inheritance exceeds depth bound",
        ));
    }
    let mut family = declared
        .get(name)
        .ok_or_else(|| CatalogError::Invalid(format!("missing scientific base family {name}")))?
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
                parts.push(part.to_str().ok_or_else(|| {
                    CatalogError::Invalid("non-UTF8 scientific input path".into())
                })?)
            }
            Component::CurDir => (),
            Component::ParentDir => {
                if parts.pop().is_none() {
                    return Err(CatalogError::Invalid(
                        "scientific path escapes repository".into(),
                    ));
                }
            }
            _ => {
                return Err(CatalogError::Invalid(
                    "scientific input path must be repository-relative".into(),
                ));
            }
        }
    }
    if parts.is_empty() {
        return Err(CatalogError::Invalid("empty scientific input path".into()));
    }
    Ok(parts.join("/"))
}
