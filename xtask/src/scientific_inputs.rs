// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One exact-byte capsule from catalog roots and production package/parser facts.

use anyhow::{Context, Result, bail, ensure};
use pse_admission::authoring_driver::document::{CapturedPackage, capture_package};
use pse_authoring::{
    ParseBudget,
    language::{self, IdentityPolicy},
    p0::{GraphLimits, resolve_rows},
};
use pse_ids::SemanticId;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Component, Path, PathBuf},
};

const MAX_PACKAGES: usize = 4096;
const MAX_CAPTURE_BYTES: usize = 256 << 20;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    version: u32,
    package_inventory: String,
    families: Vec<Family>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Family {
    name: String,
    #[serde(default)]
    extends: Option<String>,
    #[serde(default)]
    roots: Vec<String>,
    #[serde(default)]
    documents: Vec<String>,
    consumers: Vec<String>,
    #[serde(default)]
    projection: Option<String>,
    #[serde(default)]
    substitutions: Vec<String>,
    #[serde(default)]
    manifest: Option<String>,
}

#[derive(Debug, Serialize)]
struct Capsule {
    schema: &'static str,
    files: BTreeMap<String, Vec<u8>>,
    families: BTreeMap<String, CapturedFamily>,
    directories: BTreeMap<String, Option<Vec<String>>>,
    package_headers: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CapturedFamily {
    packages: Vec<String>,
    documents: Vec<String>,
    substitutions: Vec<String>,
    projection: Option<String>,
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

pub(super) fn run(root: &Path, catalog: &Path, output: &Path) -> Result<()> {
    let capsule = capture(root, catalog)?;
    std::fs::write(output, serde_json::to_vec(&capsule)?)
        .with_context(|| format!("writing scientific inputs {}", output.display()))
}

fn capture(root: &Path, catalog_path: &Path) -> Result<Capsule> {
    let catalog_path = if catalog_path.is_absolute() {
        catalog_path
            .strip_prefix(root)
            .context("catalog must be inside the repository")?
    } else {
        catalog_path
    };
    let catalog_name = normalize(catalog_path)?;
    let catalog_bytes = read_file(&checked_path(root, &catalog_name)?)?;
    let mut catalog: Catalog = toml::from_str(std::str::from_utf8(&catalog_bytes)?)?;
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
        schema: "scientific-inputs-v1",
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
            let declared: Manifest = toml::from_str(std::str::from_utf8(&bytes)?)?;
            ensure!(
                declared.version == 1 && !declared.runs.is_empty(),
                "unsupported or empty scientific run manifest"
            );
            let directory = Path::new(&manifest)
                .parent()
                .context("manifest parent absent")?;
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
            .with_context(|| format!("capturing package {path}"))?;
        for bytes in package.files.values() {
            captured_bytes = captured_bytes
                .checked_add(bytes.len())
                .context("capture extent overflow")?;
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
                let target = identities.get(&dependency.package_id).with_context(|| {
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
        .with_context(|| format!("resolving scientific family {}", family.name))?;
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
        .context("scientific capsule extent overflow")?;
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

fn resolve_families(families: Vec<Family>) -> Result<Vec<Family>> {
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
        .with_context(|| format!("missing scientific base family {name}"))?
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

fn verify_snapshot(
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
    let rows = language::parse(std::str::from_utf8(bytes)?, document, policy, budget)?;
    let mut names = BTreeMap::new();
    let mut scopes = Vec::new();
    let mut imports = Vec::new();
    for row in rows {
        let name = match row.parent_id {
            Some(parent) => format!(
                "{}.{}",
                names.get(&parent).context("modeling parent name absent")?,
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
                    .with_context(|| format!("{path}: imported package scope {name} is absent"))?
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
                parts.push(part.to_str().context("non-UTF8 scientific input path")?)
            }
            Component::CurDir => (),
            Component::ParentDir => {
                ensure!(parts.pop().is_some(), "scientific path escapes repository");
            }
            _ => bail!("scientific input path must be repository-relative"),
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
            !std::fs::symlink_metadata(&path)?.file_type().is_symlink(),
            "scientific input traverses a symlink: {}",
            path.display()
        );
    }
    Ok(path)
}

fn read_file(path: &Path) -> Result<Vec<u8>> {
    ensure!(
        std::fs::symlink_metadata(path)?.file_type().is_file(),
        "scientific input must be a regular file"
    );
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(ParseBudget::DEFAULT_MAX_BYTES + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= ParseBudget::DEFAULT_MAX_BYTES,
        "scientific input exceeds source byte bound"
    );
    Ok(bytes)
}

fn discover_packages(
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
    if path.join("package.toml").try_exists()? {
        packages.insert(relative);
        ensure!(
            packages.len() <= MAX_PACKAGES,
            "package inventory exceeds bound"
        );
        return Ok(());
    }
    let mut entries = std::fs::read_dir(&path)?.collect::<std::io::Result<Vec<_>>>()?;
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
                        .map_err(|_| anyhow::anyhow!("non-UTF8 inventory member"))
                })
                .collect::<Result<Vec<_>>>()?,
        ),
    );
    ensure!(
        directories.len() <= MAX_PACKAGES * 16,
        "package directory inventory exceeds bound"
    );
    for entry in entries {
        let kind = entry.file_type()?;
        ensure!(!kind.is_symlink(), "package inventory contains a symlink");
        if kind.is_dir() {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("non-UTF8 package directory"))?;
            discover_packages(root, &format!("{relative}/{name}"), packages, directories)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(root: &Path, path: &str, identity: u8, dependencies: &[u8], source: &str) {
        let directory = root.join(path);
        std::fs::create_dir_all(directory.join("models")).unwrap();
        let dependencies = dependencies.iter().map(|id| format!(
            "{{package_id=\"{}\",version_req={{operator=\"exact\",major=1,minor=0,patch=0}}}}",
            format!("{id:02x}").repeat(16),
        )).collect::<Vec<_>>().join(",");
        let header = format!(
            r#"[package]
package_id = "{}"
name = "p{identity}"
version = "1.0.0"
kind = "model"
id_policy = "named"
dependencies = [{dependencies}]
doc = "scientific capture control"
"#,
            format!("{identity:02x}").repeat(16)
        );
        std::fs::write(directory.join("package.toml"), header).unwrap();
        std::fs::write(directory.join("models/source.pse"), source).unwrap();
    }

    fn catalog(root: &Path, selection: &str) {
        std::fs::write(
            root.join("inputs.toml"),
            format!(
                r#"version = 1
package_inventory = "inventory"
[[families]]
name = "control"
consumers = ["unit-control"]
{selection}
"#
            ),
        )
        .unwrap();
    }

    fn fixture() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        package(
            root.path(),
            "inventory/a",
            1,
            &[2],
            "package alpha { use beta @\"1.0.0\"; }\n",
        );
        package(root.path(), "inventory/b", 2, &[], "package beta {}\n");
        package(
            root.path(),
            "inventory/unused",
            3,
            &[],
            "package unused {}\n",
        );
        catalog(
            root.path(),
            r#"roots = ["inventory/a"]
documents = ["inventory/a/models/source.pse"]
projection = "declarations""#,
        );
        root
    }

    #[test]
    fn scientific_inputs_derive_p0_imports_and_source_only_reload() {
        let root = fixture();
        std::fs::create_dir(root.path().join("inventory/b/data")).unwrap();
        let data = "inventory/b/data/bytes.parquet";
        std::fs::write(root.path().join(data), [0, 255, 13, 10]).unwrap();
        let first = capture(root.path(), Path::new("inputs.toml")).unwrap();
        assert_eq!(first.schema, "scientific-inputs-v1");
        assert_eq!(
            first.families["control"].packages,
            ["inventory/a", "inventory/b"]
        );
        assert_eq!(first.families["control"].documents.len(), 2);
        assert_eq!(first.files[data], [0, 255, 13, 10]);
        assert_eq!(first.directories["inventory/a/data"], None);
        assert!(first.files.contains_key("inventory/unused/package.toml"));
        assert!(
            !first
                .files
                .contains_key("inventory/unused/models/source.pse")
        );
        // The same compiled capture code observes a data-only edit and a new
        // imported document, retaining exact CRLF source bytes and full transport.
        std::fs::write(root.path().join(data), [0, 254, 13, 10]).unwrap();
        std::fs::write(
            root.path().join("inventory/b/models/source.pse"),
            "package beta { use gamma @\"1.0.0\"; }\n",
        )
        .unwrap();
        assert!(capture(root.path(), Path::new("inputs.toml")).is_err());
        let added = "inventory/b/models/added.pse";
        let bytes = b"package gamma {}\r\n";
        std::fs::write(root.path().join(added), bytes).unwrap();
        let changed = capture(root.path(), Path::new("inputs.toml")).unwrap();
        assert_eq!(changed.files[added], bytes);
        assert_eq!(changed.files[data], [0, 254, 13, 10]);
        assert!(
            changed.families["control"]
                .documents
                .iter()
                .any(|path| path == added)
        );
        assert!(
            changed.directories["inventory/b/models"]
                .as_ref()
                .unwrap()
                .contains(&"added.pse".into())
        );
        std::fs::remove_file(root.path().join(added)).unwrap();
        assert!(capture(root.path(), Path::new("inputs.toml")).is_err());
    }

    #[test]
    fn scientific_inputs_refuse_ambiguous_scopes_and_missing_package_dependencies() {
        let root = fixture();
        std::fs::write(
            root.path().join("inventory/b/models/duplicate.pse"),
            "package beta {}\n",
        )
        .unwrap();
        let error = capture(root.path(), Path::new("inputs.toml")).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("ambiguous modeling package scope beta")
        );
        std::fs::remove_file(root.path().join("inventory/b/models/duplicate.pse")).unwrap();
        package(root.path(), "inventory/a", 1, &[4], "package alpha {}\n");
        let error = capture(root.path(), Path::new("inputs.toml")).unwrap_err();
        assert!(error.to_string().contains("missing dependency"));
    }

    #[test]
    fn scientific_inputs_keep_parser_projection_and_synthetic_sources_distinct() {
        let root = fixture();
        std::fs::write(
            root.path().join("inventory/a/models/source.pse"),
            "package alpha { use deliberately_absent @\"1.0.0\"; }\n",
        )
        .unwrap();
        catalog(
            root.path(),
            r#"roots = ["inventory/a"]
documents = ["inventory/a/models/source.pse"]
projection = "parser-fields""#,
        );
        let parser_only = capture(root.path(), Path::new("inputs.toml")).unwrap();
        assert_eq!(
            parser_only.families["control"].documents,
            ["inventory/a/models/source.pse"]
        );
        std::fs::write(
            root.path().join("inventory/a/models/source.pse"),
            "package alpha {}\n",
        )
        .unwrap();
        std::fs::write(
            root.path().join("inventory/b/models/extra.pse"),
            "package extra {}\n",
        )
        .unwrap();
        let extra = b"package synthetic { use extra @\"1.0.0\"; }\r\n";
        std::fs::write(root.path().join("synthetic.pse"), extra).unwrap();
        catalog(
            root.path(),
            r#"roots = ["inventory/a"]
documents = ["inventory/a/models/source.pse"]
projection = "synthetic-declarations"
substitutions = ["synthetic.pse"]"#,
        );
        let synthetic = capture(root.path(), Path::new("inputs.toml")).unwrap();
        let family = &synthetic.families["control"];
        assert_eq!(family.substitutions, ["synthetic.pse"]);
        assert!(
            family
                .documents
                .iter()
                .any(|path| path == "inventory/b/models/extra.pse")
        );
        assert!(!family.documents.iter().any(|path| path == "synthetic.pse"));
        assert_eq!(synthetic.files["synthetic.pse"], extra);
        assert_eq!(family.projection.as_deref(), Some("synthetic-declarations"));
    }

    #[test]
    fn scientific_inputs_manifest_owns_run_roots_and_real_version_resolution() {
        let root = fixture();
        let manifest = r#"version = 1
[settings]
intent = "root"
diagnostics = "../diagnostics.json"
[[runs]]
package = "../inventory/a"
dependencies = ["../inventory/b"]
physical = "../inventory/b"
"#;
        let diagnostics = b"{\"limit\":1}\r\n";
        std::fs::write(root.path().join("diagnostics.json"), diagnostics).unwrap();
        std::fs::create_dir(root.path().join("config")).unwrap();
        std::fs::write(root.path().join("config/runs.toml"), manifest).unwrap();
        catalog(
            root.path(),
            r#"manifest = "config/runs.toml"
projection = "authentic-transport""#,
        );
        let captured = capture(root.path(), Path::new("inputs.toml")).unwrap();
        assert_eq!(captured.files["config/runs.toml"], manifest.as_bytes());
        assert_eq!(captured.files["diagnostics.json"], diagnostics);
        assert_eq!(
            captured.families["control"].packages,
            ["inventory/a", "inventory/b"]
        );
        let header = root.path().join("inventory/b/package.toml");
        std::fs::write(
            &header,
            std::fs::read_to_string(&header)
                .unwrap()
                .replace("version = \"1.0.0\"", "version = \"1.0.1\""),
        )
        .unwrap();
        assert!(capture(root.path(), Path::new("inputs.toml")).is_err());
    }

    #[test]
    fn scientific_inputs_inherit_one_projection_and_refuse_missing_or_cyclic_bases() {
        let root = fixture();
        std::fs::write(
            root.path().join("inventory/b/models/extra.pse"),
            "package extra {}\n",
        )
        .unwrap();
        std::fs::write(
            root.path().join("synthetic.pse"),
            "package synthetic { use beta @\"1.0.0\"; }\n",
        )
        .unwrap();
        let base = format!(
            "{}\nsubstitutions = [\"synthetic.pse\"]\n",
            std::fs::read_to_string(root.path().join("inputs.toml")).unwrap()
        );
        let inherited = format!(
            r#"{base}
[[families]]
name = "derived"
extends = "control"
roots = ["inventory/b"]
documents = ["inventory/b/models/extra.pse"]
consumers = ["derived-consumer"]
"#
        );
        std::fs::write(root.path().join("inputs.toml"), &inherited).unwrap();
        let capsule = capture(root.path(), Path::new("inputs.toml")).unwrap();
        assert_eq!(
            capsule.families["derived"].projection,
            capsule.families["control"].projection
        );
        assert_eq!(capsule.families["control"].documents.len(), 2);
        assert_eq!(capsule.families["derived"].documents.len(), 3);
        assert_eq!(capsule.families["derived"].substitutions, ["synthetic.pse"]);
        let declared: Catalog = toml::from_str(&inherited).unwrap();
        let resolved = resolve_families(declared.families).unwrap();
        assert_eq!(
            resolved
                .iter()
                .find(|family| family.name == "derived")
                .unwrap()
                .consumers,
            ["derived-consumer"]
        );
        std::fs::write(
            root.path().join("inputs.toml"),
            inherited.replace("extends = \"control\"", "extends = \"missing\""),
        )
        .unwrap();
        assert!(
            capture(root.path(), Path::new("inputs.toml"))
                .unwrap_err()
                .to_string()
                .contains("missing scientific base family missing")
        );
        std::fs::write(
            root.path().join("inputs.toml"),
            inherited.replace(
                "name = \"control\"",
                "name = \"control\"\nextends = \"derived\"",
            ),
        )
        .unwrap();
        assert!(
            capture(root.path(), Path::new("inputs.toml"))
                .unwrap_err()
                .to_string()
                .contains("inheritance cycle")
        );
    }

    #[test]
    fn scientific_inputs_reject_package_auxiliary_and_inventory_drift_before_publication() {
        let root = fixture();
        let capsule = capture(root.path(), Path::new("inputs.toml")).unwrap();
        let registry = pse_schema::shared_registry().unwrap();
        let packages = capsule
            .package_headers
            .iter()
            .map(|header| {
                let path = header.strip_suffix("/package.toml").unwrap();
                (
                    path.to_owned(),
                    capture_package(&root.path().join(path), &registry, ParseBudget::default())
                        .unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut inventory = BTreeSet::new();
        let mut directories = BTreeMap::new();
        discover_packages(root.path(), "inventory", &mut inventory, &mut directories).unwrap();
        let verify = || {
            verify_snapshot(
                root.path(),
                "inventory",
                &inventory,
                &directories,
                &packages,
                &capsule.files,
            )
        };
        verify().unwrap();
        let source = root.path().join("inventory/b/models/source.pse");
        let original = std::fs::read(&source).unwrap();
        std::fs::write(&source, b"package beta {} // source drift\n").unwrap();
        assert!(
            verify()
                .unwrap_err()
                .to_string()
                .contains("changed during capture")
        );
        std::fs::write(&source, original).unwrap();
        let absent = root.path().join("inventory/b/data");
        std::fs::create_dir(&absent).unwrap();
        assert!(
            verify()
                .unwrap_err()
                .to_string()
                .contains("changed during capture")
        );
        std::fs::remove_dir(absent).unwrap();
        package(root.path(), "inventory/added", 4, &[], "package added {}\n");
        assert!(
            verify()
                .unwrap_err()
                .to_string()
                .contains("inventory changed during capture")
        );
        std::fs::remove_dir_all(root.path().join("inventory/added")).unwrap();
        verify().unwrap();
        let catalog = root.path().join("inputs.toml");
        std::fs::write(
            &catalog,
            format!(
                "{}\n# auxiliary drift\n",
                std::fs::read_to_string(&catalog).unwrap()
            ),
        )
        .unwrap();
        assert!(
            verify()
                .unwrap_err()
                .to_string()
                .contains("auxiliary input changed during capture")
        );
    }
}
