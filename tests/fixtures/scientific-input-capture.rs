// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact scientific source closure from catalog roots and production package/parser facts.
#![allow(
    dead_code,
    reason = "explicit exporters and selected-source test binaries use different capture views"
)]

use miette::{IntoDiagnostic as _, Result, WrapErr as _, ensure, miette};
use pse_admission::authoring_driver::document::{
    CapturedPackage, CapturedPackageHeader, capture_package, capture_package_header,
    package_checksum,
};
use pse_authoring::{
    ParseBudget,
    language::{self, IdentityPolicy},
    p0::{GraphLimits, resolve_rows},
};
use pse_ids::{ContentHash, SemanticId};
use pse_model::generated::enums::{IdPolicy, ModelingDeclarationKind};
use pse_schema::Registry;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Path, PathBuf},
};

#[path = "scientific-input-catalog.rs"]
mod catalog;
pub(super) use catalog::{Catalog, resolve_families};
use catalog::{Family, Manifest, normalize};
#[path = "scientific-input-data.rs"]
pub(super) mod data;
use data::{Capsule, CapturedFamily, SCHEMA};

const MAX_PACKAGES: usize = 4096;
pub(super) const MAX_CAPTURE_BYTES: usize = 256 << 20;

#[derive(Clone)]
struct ParsedDocument {
    scopes: Vec<String>,
    imports: Vec<String>,
}

/// A verified family owns its original package transport and auxiliary source bytes.
/// Consumers borrow those bytes for as long as they retain the capture.
#[derive(Debug)]
pub(crate) struct SelectedCapture {
    root: PathBuf,
    snapshot: CaptureSnapshot,
    name: String,
    identity: ContentHash,
}

impl SelectedCapture {
    /// Identity of the selected population, bytes, membership, registry and parse budget.
    pub(crate) fn identity(&self) -> ContentHash {
        self.identity
    }

    /// Selected packages in logical-root order. Each package's paths are ordered too.
    pub(crate) fn packages(&self) -> impl Iterator<Item = (&str, &CapturedPackage)> {
        self.snapshot
            .packages
            .iter()
            .map(|(path, package)| (path.as_str(), package))
    }

    pub(crate) fn package(&self, logical_root: &str) -> Option<&CapturedPackage> {
        self.snapshot.packages.get(logical_root)
    }

    /// Exact selected inputs in repository-relative path order, including catalog/manifest.
    /// Candidate headers used only for discovery are outside this population.
    pub(crate) fn files(&self) -> BTreeMap<String, &[u8]> {
        let mut files = self
            .snapshot
            .auxiliary
            .iter()
            .map(|(path, bytes)| (path.clone(), bytes.as_slice()))
            .collect::<BTreeMap<_, _>>();
        for (path, package) in self.packages() {
            files.extend(
                package
                    .files
                    .iter()
                    .map(|(relative, bytes)| (format!("{path}/{relative}"), bytes.as_slice())),
            );
        }
        files
    }

    /// Verify current checkout stability without changing this capture's admitted bytes.
    pub(crate) fn verify_checkout(&self, root: &Path) -> Result<()> {
        ensure!(
            root.canonicalize().into_diagnostic()? == self.root,
            "scientific capture belongs to a different checkout root"
        );
        let registry = pse_schema::shared_registry()?;
        self.snapshot.verify(&self.root, &registry)
    }

    fn document_sources(&self) -> impl Iterator<Item = (&str, &str)> {
        self.snapshot.families[&self.name].sources(|path| {
            let mut parent = path;
            while let Some((root, _)) = parent.rsplit_once('/') {
                if let Some(package) = self.package(root) {
                    return &package.files[&path[root.len() + 1..]];
                }
                parent = root;
            }
            &self.snapshot.auxiliary[path]
        })
    }

    pub(crate) fn documents(&self) -> Vec<(String, String)> {
        self.document_sources()
            .map(|(path, source)| (path.to_owned(), source.to_owned()))
            .collect()
    }

    pub(crate) fn sources(&self) -> String {
        data::marked_sources(self.document_sources())
    }
}

pub(super) fn capture_family(root: &Path, catalog: &Path, name: &str) -> Result<SelectedCapture> {
    let registry = pse_schema::shared_registry()?;
    let budget = ParseBudget::default();
    let catalog = catalog.strip_prefix(root).unwrap_or(catalog);
    let root = root.canonicalize().into_diagnostic()?;
    let snapshot = CaptureSnapshot::read(&root, catalog, Some(name), &registry, budget)?;
    snapshot.verify(&root, &registry)?;
    let identity = snapshot.identity(name, &registry, budget)?;
    Ok(SelectedCapture {
        root,
        snapshot,
        name: name.to_owned(),
        identity,
    })
}

pub(super) fn capture(root: &Path, catalog: &Path) -> Result<Capsule> {
    let registry = pse_schema::shared_registry()?;
    let budget = ParseBudget::default();
    let snapshot = CaptureSnapshot::read(root, catalog, None, &registry, budget)?;
    snapshot.verify(root, &registry)?;
    Ok(snapshot.into_capsule())
}

#[derive(Debug)]
pub(super) struct CaptureSnapshot {
    inventory: String,
    inventory_roots: BTreeSet<String>,
    inventory_directories: BTreeMap<String, Option<Vec<String>>>,
    headers: BTreeMap<String, CapturedPackageHeader>,
    packages: BTreeMap<String, CapturedPackage>,
    auxiliary: BTreeMap<String, Vec<u8>>,
    families: BTreeMap<String, CapturedFamily>,
}

impl CaptureSnapshot {
    pub(super) fn read(
        root: &Path,
        catalog_path: &Path,
        family: Option<&str>,
        registry: &Registry,
        budget: ParseBudget,
    ) -> Result<Self> {
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
        let catalog = Catalog::from_bytes(&catalog_bytes)?;
        let families = resolve_families(catalog.families, family)?;
        let mut snapshot = Self {
            inventory: catalog.package_inventory,
            inventory_roots: BTreeSet::new(),
            inventory_directories: BTreeMap::new(),
            headers: BTreeMap::new(),
            packages: BTreeMap::new(),
            auxiliary: BTreeMap::from([(catalog_name, catalog_bytes)]),
            families: BTreeMap::new(),
        };
        discover_packages(
            root,
            &snapshot.inventory,
            &mut snapshot.inventory_roots,
            &mut snapshot.inventory_directories,
        )?;
        let mut candidates = snapshot.inventory_roots.clone();
        let mut family_roots = BTreeMap::new();
        for family in &families {
            ensure!(
                !family.name.is_empty() && !family.consumers.is_empty(),
                "family needs a name and consumers"
            );
            let roots = family_roots_from_sources(root, family, &mut snapshot.auxiliary)?;
            candidates.extend(roots.iter().cloned());
            family_roots.insert(family.name.as_str(), roots);
        }
        ensure!(
            candidates.len() <= MAX_PACKAGES,
            "scientific package inventory exceeds bounds"
        );
        let mut identities = BTreeMap::new();
        let mut remaining = MAX_CAPTURE_BYTES;
        for path in candidates {
            let header = capture_package_header(&checked_path(root, &path)?, budget)
                .wrap_err_with(|| format!("capturing candidate header {path}"))?;
            charge(&mut remaining, header.bytes.len())?;
            ensure!(
                identities
                    .insert(header.header.package_id, path.clone())
                    .is_none(),
                "ambiguous package identity {}",
                header.header.package_id
            );
            snapshot.headers.insert(path, header);
        }
        let mut closures = BTreeMap::new();
        let mut selected_packages = BTreeSet::new();
        for family in &families {
            let mut closure = BTreeSet::new();
            let mut pending = family_roots[family.name.as_str()]
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            while let Some(path) = pending.pop() {
                if !closure.insert(path) {
                    continue;
                }
                for dependency in &snapshot.headers[path].header.dependencies {
                    let target = identities.get(&dependency.package_id).wrap_err_with(|| {
                        format!("{path}: missing dependency {}", dependency.package_id)
                    })?;
                    pending.push(target);
                }
            }
            let headers = closure
                .iter()
                .map(|path| snapshot.headers[*path].header.clone())
                .collect::<Vec<_>>();
            resolve_rows(
                &headers,
                GraphLimits {
                    nodes: MAX_PACKAGES,
                    edges: MAX_PACKAGES * MAX_PACKAGES,
                },
            )
            .wrap_err_with(|| format!("resolving scientific family {}", family.name))?;
            selected_packages.extend(closure.iter().copied());
            closures.insert(family.name.as_str(), closure);
        }
        for path in selected_packages {
            let package = capture_package(
                &checked_path(root, path)?,
                registry,
                ParseBudget {
                    max_bytes: budget.max_bytes.min(remaining as u64),
                    ..budget
                },
            )
            .wrap_err_with(|| format!("capturing package {path}"))?;
            ensure!(
                package.files["package.toml"] == snapshot.headers[path].bytes,
                "scientific package header changed during capture: {path}"
            );
            for bytes in package.files.values() {
                charge(&mut remaining, bytes.len())?;
            }
            snapshot.packages.insert(path.to_owned(), package);
        }
        let mut parsed = BTreeMap::new();
        for family in &families {
            let captured = materialize_family(
                root,
                family,
                &closures[family.name.as_str()],
                &mut snapshot,
                &mut parsed,
                budget,
            )?;
            snapshot.families.insert(family.name.clone(), captured);
        }
        for bytes in snapshot.auxiliary.values() {
            charge(&mut remaining, bytes.len())?;
        }
        Ok(snapshot)
    }

    pub(super) fn verify(&self, root: &Path, registry: &Registry) -> Result<()> {
        let mut current_roots = BTreeSet::new();
        let mut current_directories = BTreeMap::new();
        discover_packages(
            root,
            &self.inventory,
            &mut current_roots,
            &mut current_directories,
        )?;
        ensure!(
            current_roots == self.inventory_roots
                && current_directories == self.inventory_directories,
            "scientific package-header inventory changed during capture"
        );
        for (path, before) in &self.packages {
            before.verify(&checked_path(root, path)?, registry).wrap_err_with(|| {
                format!("scientific package bytes or document membership changed during capture: {path}")
            })?;
        }
        for (path, before) in &self.headers {
            if !self.packages.contains_key(path) {
                let path = format!("{path}/package.toml");
                ensure!(
                    before.bytes == read_file(&checked_path(root, &path)?)?,
                    "scientific package header changed during capture: {path}"
                );
            }
        }
        for (path, before) in &self.auxiliary {
            ensure!(
                *before == read_file(&checked_path(root, path)?)?,
                "scientific auxiliary input changed during capture: {path}"
            );
        }
        Ok(())
    }

    fn identity(
        &self,
        name: &str,
        registry: &Registry,
        budget: ParseBudget,
    ) -> Result<ContentHash> {
        let packages = self
            .packages
            .iter()
            .map(|(path, package)| {
                (
                    path,
                    package.header.package_id,
                    &package.header.version,
                    package.header.content_hash,
                    &package.directories,
                    &package.modeling_documents,
                )
            })
            .collect::<Vec<_>>();
        let metadata = serde_json::to_vec(&(
            "pse-selected-scientific-input-v1",
            name,
            &self.families[name],
            packages,
            package_checksum(&self.auxiliary),
            registry.fingerprint(),
            (
                budget.max_depth,
                budget.max_aliases,
                budget.max_bytes,
                budget.max_data_rows,
            ),
        ))
        .into_diagnostic()?;
        Ok(pse_ids::preimage::hash(&metadata))
    }

    fn into_capsule(self) -> Capsule {
        let mut files = self.auxiliary;
        let mut directories = self.inventory_directories;
        let package_headers = self
            .headers
            .into_iter()
            .map(|(path, header)| {
                let path = format!("{path}/package.toml");
                files.insert(path.clone(), header.bytes);
                path
            })
            .collect();
        for (path, package) in self.packages {
            files.extend(
                package
                    .files
                    .into_iter()
                    .map(|(relative, bytes)| (format!("{path}/{relative}"), bytes)),
            );
            directories.extend(
                package
                    .directories
                    .into_iter()
                    .map(|(relative, members)| (format!("{path}/{relative}"), members)),
            );
        }
        Capsule {
            schema: SCHEMA.to_owned(),
            files,
            families: self.families,
            directories,
            package_headers,
        }
    }
}

fn charge(remaining: &mut usize, size: usize) -> Result<()> {
    *remaining = remaining
        .checked_sub(size)
        .wrap_err("scientific source capture exceeds byte bound")?;
    Ok(())
}

fn family_roots_from_sources(
    root: &Path,
    family: &Family,
    auxiliary: &mut BTreeMap<String, Vec<u8>>,
) -> Result<BTreeSet<String>> {
    let mut selected = family
        .roots
        .iter()
        .map(|path| normalize(Path::new(path)))
        .collect::<Result<BTreeSet<_>>>()?;
    if let Some(manifest) = &family.manifest {
        let manifest = normalize(Path::new(manifest))?;
        let bytes = auxiliary_input(root, &manifest, auxiliary)?;
        let declared = Manifest::from_bytes(bytes)?;
        let directory = Path::new(&manifest)
            .parent()
            .wrap_err("manifest parent absent")?;
        if let Some(diagnostics) = declared.settings.diagnostics {
            auxiliary_input(root, &normalize(&directory.join(diagnostics))?, auxiliary)?;
        }
        for run in declared.runs {
            for package in run.roots() {
                selected.insert(normalize(&directory.join(package))?);
            }
        }
    }
    ensure!(
        !selected.is_empty(),
        "family {} has no package roots",
        family.name
    );
    Ok(selected)
}

fn auxiliary_input<'a>(
    root: &Path,
    path: &str,
    files: &'a mut BTreeMap<String, Vec<u8>>,
) -> Result<&'a [u8]> {
    let entry = files.entry(path.to_owned());
    let bytes = match entry {
        std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(read_file(&checked_path(root, path)?)?)
        }
    };
    Ok(bytes)
}

fn materialize_family(
    root: &Path,
    family: &Family,
    closure: &BTreeSet<&str>,
    snapshot: &mut CaptureSnapshot,
    parsed: &mut BTreeMap<String, ParsedDocument>,
    budget: ParseBudget,
) -> Result<CapturedFamily> {
    let available = closure
        .iter()
        .flat_map(|path| {
            snapshot.packages[*path]
                .modeling_documents
                .iter()
                .map(move |document| format!("{path}/{document}"))
        })
        .collect::<BTreeSet<_>>();
    let mut substitutions = BTreeSet::new();
    for path in &family.substitutions {
        let path = normalize(Path::new(path))?;
        ensure!(
            path.ends_with(".pse")
                && !snapshot
                    .headers
                    .keys()
                    .any(|package| path.starts_with(&format!("{package}/"))),
            "substitution must be outside authenticated package transport: {path}"
        );
        let bytes = auxiliary_input(root, &path, &mut snapshot.auxiliary)?;
        if !parsed.contains_key(&path) {
            let document =
                pse_ids::named_id(SemanticId::NIL, &format!("scientific-substitution:{path}"));
            parsed.insert(
                path.clone(),
                parse_document(bytes, document, IdentityPolicy::Named, budget)?,
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
        "family {} selects a missing source or a source outside its package closure",
        family.name
    );
    let documents = if family.projection.as_deref() == Some("parser-fields") {
        // This family observes source attribution without semantic import admission.
        selected
    } else {
        for path in closure {
            let package = &snapshot.packages[*path];
            for relative in &package.modeling_documents {
                let path = format!("{path}/{relative}");
                if let std::collections::btree_map::Entry::Vacant(entry) = parsed.entry(path) {
                    entry.insert(parse_package_document(package, relative, budget)?);
                }
            }
        }
        imported_documents(parsed, &available, &selected, &substitutions)?
    };
    Ok(CapturedFamily {
        packages: closure.iter().map(|path| (*path).to_owned()).collect(),
        documents: documents.into_iter().collect(),
        substitutions: substitutions.into_iter().collect(),
        projection: family.projection.clone(),
    })
}

fn parse_package_document(
    package: &CapturedPackage,
    path: &str,
    budget: ParseBudget,
) -> Result<ParsedDocument> {
    let policy = if package.header.id_policy == IdPolicy::Named {
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
        if row.value.kind == ModelingDeclarationKind::Package {
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
