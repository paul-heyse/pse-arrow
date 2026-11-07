// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One exact-byte capsule from catalog roots and production package/parser facts.

use miette::{IntoDiagnostic as _, Result, WrapErr as _, bail, ensure, miette};
use pse_admission::authoring_driver::document::{CapturedPackage, capture_package};
use pse_authoring::{
    ParseBudget,
    language::{self, IdentityPolicy},
    p0::{GraphLimits, resolve_rows},
};
use pse_ids::SemanticId;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Component, Path, PathBuf},
};

#[path = "scientific-input-data.rs"]
pub(super) mod data;
use data::{Capsule, CapturedFamily, SCHEMA};

const MAX_PACKAGES: usize = 4096;
pub(super) const MAX_CAPTURE_BYTES: usize = 256 << 20;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Catalog {
    version: u32,
    package_inventory: String,
    pub(super) families: Vec<Family>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Family {
    pub(super) name: String,
    #[serde(default)]
    extends: Option<String>,
    #[serde(default)]
    roots: Vec<String>,
    #[serde(default)]
    documents: Vec<String>,
    pub(super) consumers: Vec<String>,
    #[serde(default)]
    projection: Option<String>,
    #[serde(default)]
    substitutions: Vec<String>,
    #[serde(default)]
    manifest: Option<String>,
}

#[derive(Deserialize)]
struct Manifest {
    version: u32,
    runs: Vec<Run>,
    #[serde(default)]
    settings: ManifestSettings,
}

#[derive(Default, Deserialize)]
struct ManifestSettings {
    diagnostics: Option<String>,
}

#[derive(Deserialize)]
struct Run {
    package: String,
    #[serde(default)]
    dependencies: Vec<String>,
    physical: String,
}

#[derive(Clone)]
struct ParsedDocument {
    scopes: Vec<String>,
    imports: Vec<String>,
}

pub(super) fn capture(root: &Path, catalog_path: &Path) -> Result<Capsule> {
    let catalog_path = if catalog_path.is_absolute() {
        catalog_path
            .strip_prefix(root)
            .into_diagnostic()
            .wrap_err("catalog must be inside the repository")?
    } else {
        catalog_path
    };
    let catalog_name = normalize(catalog_path)?;
    let catalog_bytes = read_file(&checked_path(root, &catalog_name)?)?;
    let catalog_text = std::str::from_utf8(&catalog_bytes).into_diagnostic()?;
    let mut catalog: Catalog = toml::from_str(catalog_text).into_diagnostic()?;
    ensure!(
        catalog.version == 1,
        "unsupported scientific input catalog version"
    );
    ensure!(
        catalog.families.len() <= MAX_PACKAGES,
        "scientific family count exceeds bound"
    );
    catalog.families = resolve_families(catalog.families)?;
    let mut capsule = Capsule {
        schema: SCHEMA.to_owned(),
        files: BTreeMap::from([(catalog_name, catalog_bytes)]),
        families: BTreeMap::new(),
        directories: BTreeMap::new(),
        package_headers: Vec::new(),
    };
    let mut package_roots = BTreeSet::new();
    discover_packages(
        root,
        &catalog.package_inventory,
        &mut package_roots,
        &mut capsule.directories,
    )?;
    let inventory_roots = package_roots.clone();
    let inventory_directories = capsule.directories.clone();
    let mut family_roots = BTreeMap::new();
    for family in &catalog.families {
        ensure!(
            !family.name.is_empty() && !family.consumers.is_empty(),
            "family needs a name and consumers"
        );
        let mut selected = family
            .roots
            .iter()
            .map(|path| normalize(Path::new(path)))
            .collect::<Result<BTreeSet<_>>>()?;
        if let Some(manifest) = &family.manifest {
            let manifest = normalize(Path::new(manifest))?;
            let bytes = read_file(&checked_path(root, &manifest)?)?;
            let manifest_text = std::str::from_utf8(&bytes).into_diagnostic()?;
            let declared: Manifest = toml::from_str(manifest_text).into_diagnostic()?;
            ensure!(
                declared.version == 1 && !declared.runs.is_empty(),
                "unsupported or empty scientific run manifest"
            );
            let directory = Path::new(&manifest)
                .parent()
                .wrap_err("manifest parent absent")?;
            if let Some(diagnostics) = declared.settings.diagnostics {
                let diagnostics = normalize(&directory.join(diagnostics))?;
                capsule.files.insert(
                    diagnostics.clone(),
                    read_file(&checked_path(root, &diagnostics)?)?,
                );
            }
            for run in declared.runs {
                for package in std::iter::once(run.package)
                    .chain(run.dependencies)
                    .chain(std::iter::once(run.physical))
                {
                    selected.insert(normalize(&directory.join(package))?);
                }
            }
            capsule.files.insert(manifest, bytes);
        }
        ensure!(
            !selected.is_empty(),
            "family {} has no package roots",
            family.name
        );
        package_roots.extend(selected.iter().cloned());
        ensure!(
            family_roots.insert(family.name.clone(), selected).is_none(),
            "duplicate scientific family {}",
            family.name
        );
    }
    ensure!(
        package_roots.len() <= MAX_PACKAGES,
        "scientific package inventory exceeds bounds"
    );
    let registry = pse_schema::shared_registry()?;
    let budget = ParseBudget::default();
    let mut packages = BTreeMap::new();
    let mut identities = BTreeMap::new();
    let mut captured_bytes = 0usize;
    for path in package_roots {
        let package = capture_package(&checked_path(root, &path)?, &registry, budget)
            .wrap_err_with(|| format!("capturing package {path}"))?;
        for bytes in package.files.values() {
            captured_bytes = captured_bytes
                .checked_add(bytes.len())
                .wrap_err("capture extent overflow")?;
        }
        ensure!(
            captured_bytes <= MAX_CAPTURE_BYTES,
            "scientific source capture exceeds byte bound"
        );
        ensure!(
            identities
                .insert(package.header.package_id, path.clone())
                .is_none(),
            "ambiguous package identity {}",
            package.header.package_id
        );
        let header_path = format!("{path}/package.toml");
        capsule.package_headers.push(header_path.clone());
        capsule
            .files
            .insert(header_path, package.files["package.toml"].clone());
        packages.insert(path, package);
    }
    let mut parsed = BTreeMap::new();
    for family in catalog.families {
        let mut closure = BTreeSet::new();
        let mut pending = family_roots[&family.name]
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        while let Some(path) = pending.pop() {
            if !closure.insert(path.clone()) {
                continue;
            }
            let package = &packages[&path];
            for dependency in &package.header.dependencies {
                let target = identities.get(&dependency.package_id).wrap_err_with(|| {
                    format!("{path}: missing dependency {}", dependency.package_id)
                })?;
                pending.push(target.clone());
            }
        }
        let headers = closure
            .iter()
            .map(|path| packages[path].header.clone())
            .collect::<Vec<_>>();
        resolve_rows(
            &headers,
            GraphLimits {
                nodes: MAX_PACKAGES,
                edges: MAX_PACKAGES * MAX_PACKAGES,
            },
        )
        .wrap_err_with(|| format!("resolving scientific family {}", family.name))?;
        let mut available = BTreeSet::new();
        for path in &closure {
            let package = &packages[path];
            for (relative, bytes) in &package.files {
                capsule
                    .files
                    .entry(format!("{path}/{relative}"))
                    .or_insert_with(|| bytes.clone());
            }
            for (directory, members) in &package.directories {
                capsule
                    .directories
                    .insert(format!("{path}/{directory}"), members.clone());
            }
            for document in &package.modeling_documents {
                available.insert(format!("{path}/{document}"));
            }
        }
        let mut substitutions = BTreeSet::new();
        for path in &family.substitutions {
            let path = normalize(Path::new(path))?;
            ensure!(
                path.ends_with(".pse")
                    && !packages
                        .keys()
                        .any(|package| path.starts_with(&format!("{package}/"))),
                "substitution must be outside authenticated package transport: {path}"
            );
            if !capsule.files.contains_key(&path) {
                capsule
                    .files
                    .insert(path.clone(), read_file(&checked_path(root, &path)?)?);
            }
            if !parsed.contains_key(&path) {
                let document =
                    pse_ids::named_id(SemanticId::NIL, &format!("scientific-substitution:{path}"));
                parsed.insert(
                    path.clone(),
                    parse_document(
                        &capsule.files[&path],
                        document,
                        IdentityPolicy::Named,
                        budget,
                    )?,
                );
            }
            substitutions.insert(path);
        }
        let selected = if family.documents.is_empty() {
            available.clone()
        } else {
            family
                .documents
                .iter()
                .map(|path| normalize(Path::new(path)))
                .collect::<Result<BTreeSet<_>>>()?
        };
        ensure!(
            selected.is_subset(&available),
            "family {} selects a source outside its package closure",
            family.name
        );
        let documents = if family.projection.as_deref() == Some("parser-fields") {
            // This family observes source attribution, not semantic import admission.
            selected
        } else {
            for path in &closure {
                let package = &packages[path];
                for relative in &package.modeling_documents {
                    let path = format!("{path}/{relative}");
                    if let std::collections::btree_map::Entry::Vacant(entry) = parsed.entry(path) {
                        entry.insert(parse_package_document(package, relative, budget)?);
                    }
                }
            }
            imported_documents(&parsed, &available, &selected, &substitutions)?
        };
        capsule.families.insert(
            family.name,
            CapturedFamily {
                packages: closure.into_iter().collect(),
                documents: documents.into_iter().collect(),
                substitutions: substitutions.into_iter().collect(),
                projection: family.projection,
            },
        );
    }
    let output_bytes = capsule
        .files
        .values()
        .try_fold(0usize, |total, bytes| total.checked_add(bytes.len()))
        .wrap_err("scientific capsule extent overflow")?;
    ensure!(
        output_bytes <= MAX_CAPTURE_BYTES,
        "scientific capsule exceeds byte bound"
    );
    verify_snapshot(
        root,
        &catalog.package_inventory,
        &inventory_roots,
        &inventory_directories,
        &packages,
        &capsule.files,
    )?;
    Ok(capsule)
}

pub(super) fn resolve_families(families: Vec<Family>) -> Result<Vec<Family>> {
    let mut declared = BTreeMap::new();
    for family in families {
        ensure!(
            declared.insert(family.name.clone(), family).is_none(),
            "duplicate scientific family name"
        );
    }
    let mut resolved = BTreeMap::new();
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
        active.len() <= ParseBudget::DEFAULT_MAX_DEPTH as usize,
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

pub(super) fn verify_snapshot(
    root: &Path,
    inventory: &str,
    inventory_roots: &BTreeSet<String>,
    inventory_directories: &BTreeMap<String, Option<Vec<String>>>,
    packages: &BTreeMap<String, CapturedPackage>,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    // Only the capture owner rereads. Consumers all receive the same immutable bytes.
    let mut current_roots = BTreeSet::new();
    let mut current_directories = BTreeMap::new();
    discover_packages(
        root,
        inventory,
        &mut current_roots,
        &mut current_directories,
    )?;
    ensure!(
        &current_roots == inventory_roots && &current_directories == inventory_directories,
        "scientific package-header inventory changed during capture"
    );
    let registry = pse_schema::shared_registry()?;
    for (path, before) in packages {
        let after = capture_package(
            &checked_path(root, path)?,
            &registry,
            ParseBudget::default(),
        )?;
        ensure!(
            before.files == after.files && before.directories == after.directories,
            "scientific package bytes or document membership changed during capture: {path}"
        );
    }
    for (path, before) in files {
        if !packages
            .keys()
            .any(|package| path.starts_with(&format!("{package}/")))
        {
            ensure!(
                *before == read_file(&checked_path(root, path)?)?,
                "scientific auxiliary input changed during capture: {path}"
            );
        }
    }
    Ok(())
}

fn parse_package_document(
    package: &CapturedPackage,
    path: &str,
    budget: ParseBudget,
) -> Result<ParsedDocument> {
    let policy = if package.header.id_policy == pse_relations::generated::enums::IdPolicy::Named {
        IdentityPolicy::Named
    } else {
        IdentityPolicy::Explicit
    };
    parse_document(
        &package.files[path],
        pse_ids::named_id(package.header.package_id.as_id(), path),
        policy,
        budget,
    )
}

fn parse_document(
    bytes: &[u8],
    document: SemanticId,
    policy: IdentityPolicy,
    budget: ParseBudget,
) -> Result<ParsedDocument> {
    let source = std::str::from_utf8(bytes).into_diagnostic()?;
    let rows = language::parse(source, document, policy, budget)?;
    let mut names = BTreeMap::new();
    let mut scopes = Vec::new();
    let mut imports = Vec::new();
    for row in rows {
        let name = match row.parent_id {
            Some(parent) => format!(
                "{}.{}",
                names.get(&parent).wrap_err("modeling parent name absent")?,
                row.name
            ),
            None => row.name.clone(),
        };
        if row.value.kind == pse_relations::generated::enums::ModelingDeclarationKind::Package {
            scopes.push(name.clone());
        }
        if row.value.import.is_some() {
            imports.push(row.name);
        }
        names.insert(row.declaration_id, name);
    }
    Ok(ParsedDocument { scopes, imports })
}

fn imported_documents(
    parsed: &BTreeMap<String, ParsedDocument>,
    available: &BTreeSet<String>,
    selected: &BTreeSet<String>,
    substitutions: &BTreeSet<String>,
) -> Result<BTreeSet<String>> {
    let mut scopes = BTreeMap::new();
    for path in available.iter().chain(substitutions) {
        for name in &parsed[path].scopes {
            ensure!(
                scopes.insert(name.clone(), path.clone()).is_none(),
                "ambiguous modeling package scope {name}"
            );
        }
    }
    let mut visited = BTreeSet::new();
    let mut pending = selected
        .iter()
        .chain(substitutions)
        .cloned()
        .collect::<Vec<_>>();
    while let Some(path) = pending.pop() {
        if !visited.insert(path.clone()) {
            continue;
        }
        for name in &parsed[&path].imports {
            pending.push(
                scopes
                    .get(name)
                    .wrap_err_with(|| format!("{path}: imported package scope {name} is absent"))?
                    .clone(),
            );
        }
    }
    Ok(visited.difference(substitutions).cloned().collect())
}

fn normalize(path: &Path) -> Result<String> {
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

fn checked_path(root: &Path, relative: &str) -> Result<PathBuf> {
    let relative = normalize(Path::new(relative))?;
    let mut path = root.to_owned();
    for component in Path::new(&relative).components() {
        path.push(component);
        ensure!(
            !std::fs::symlink_metadata(&path)
                .into_diagnostic()?
                .file_type()
                .is_symlink(),
            "scientific input traverses a symlink: {}",
            path.display()
        );
    }
    Ok(path)
}

fn read_file(path: &Path) -> Result<Vec<u8>> {
    ensure!(
        std::fs::symlink_metadata(path)
            .into_diagnostic()?
            .file_type()
            .is_file(),
        "scientific input must be a regular file"
    );
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .into_diagnostic()?
        .take(ParseBudget::DEFAULT_MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .into_diagnostic()?;
    ensure!(
        bytes.len() as u64 <= ParseBudget::DEFAULT_MAX_BYTES,
        "scientific input exceeds source byte bound"
    );
    Ok(bytes)
}

pub(super) fn discover_packages(
    root: &Path,
    relative: &str,
    packages: &mut BTreeSet<String>,
    directories: &mut BTreeMap<String, Option<Vec<String>>>,
) -> Result<()> {
    let relative = normalize(Path::new(relative))?;
    ensure!(
        relative.split('/').count() <= ParseBudget::DEFAULT_MAX_DEPTH as usize,
        "package inventory depth exceeds bound"
    );
    let path = checked_path(root, &relative)?;
    if path.join("package.toml").try_exists().into_diagnostic()? {
        packages.insert(relative);
        ensure!(
            packages.len() <= MAX_PACKAGES,
            "package inventory exceeds bound"
        );
        return Ok(());
    }
    let mut entries = std::fs::read_dir(&path)
        .into_diagnostic()?
        .collect::<std::io::Result<Vec<_>>>()
        .into_diagnostic()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    directories.insert(
        relative.clone(),
        Some(
            entries
                .iter()
                .map(|entry| {
                    entry
                        .file_name()
                        .into_string()
                        .map_err(|_| miette!("non-UTF8 inventory member"))
                })
                .collect::<Result<Vec<_>>>()?,
        ),
    );
    ensure!(
        directories.len() <= MAX_PACKAGES * 16,
        "package directory inventory exceeds bound"
    );
    for entry in entries {
        let kind = entry.file_type().into_diagnostic()?;
        ensure!(!kind.is_symlink(), "package inventory contains a symlink");
        if kind.is_dir() {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| miette!("non-UTF8 package directory"))?;
            discover_packages(root, &format!("{relative}/{name}"), packages, directories)?;
        }
    }
    Ok(())
}
