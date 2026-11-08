// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive, conservative scientific input reach from authored catalog and package headers.

#[path = "../../../../tests/fixtures/scientific-input-catalog.rs"]
mod catalog;

use catalog::{Catalog, Manifest, normalize, resolve_families};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    os::unix::fs::MetadataExt as _,
    path::{Path, PathBuf},
};

pub(super) const CATALOG: &str = "tests/fixtures/scientific-inputs.toml";
const MAX_HEADER_BYTES: u64 = 1 << 20;
const MAX_PATHS: usize = 65_536;
const MAX_PACKAGES: usize = 4096;

pub(super) struct FamilyReach {
    pub(super) name: String,
    pub(super) consumers: Vec<String>,
    trees: BTreeSet<PathBuf>,
    files: BTreeSet<PathBuf>,
}

pub(super) struct Projection {
    families: Vec<FamilyReach>,
    context: BTreeSet<PathBuf>,
}

impl Projection {
    pub(super) fn matches<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl Iterator<Item = &'a FamilyReach> + 'a {
        self.families.iter().filter(move |family| {
            family.files.contains(path) || family.trees.iter().any(|root| path.starts_with(root))
        })
    }

    pub(super) fn changes_context(&self, path: &Path) -> bool {
        self.context.contains(path)
    }
}

pub(super) fn context_path(path: &Path) -> bool {
    [
        CATALOG,
        "tests/fixtures/scientific-input-catalog.rs",
        "tests/fixtures/scientific-input-capture.rs",
        "tests/fixtures/scientific-input-data.rs",
        "crates/pse-host/src/affected.rs",
        "crates/pse-host/src/affected/scientific.rs",
    ]
    .iter()
    .any(|owner| path == Path::new(owner))
        || (path.starts_with("packages/reference")
            && path.file_name().is_some_and(|name| name == "package.toml"))
}

pub(super) fn scientific_path(path: &Path) -> bool {
    path.starts_with("packages/reference")
        || path.starts_with("tests/fixtures/scientific")
        || context_path(path)
}

pub(super) fn verify_changed_path(root: &Path, path: &Path) -> Result<(), String> {
    if path.to_str().is_none() {
        return Err("scientific changed path is not UTF-8".into());
    }
    // Deletions have a valid old association. For any existing prefix, refuse
    // a symlink or a path that no longer denotes its declared location.
    for relative in path.ancestors() {
        let absolute = root.join(relative);
        let metadata = match fs::symlink_metadata(&absolute) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("{}: {error}", relative.display())),
        };
        let resolved = absolute
            .canonicalize()
            .map_err(|error| format!("{}: {error}", relative.display()))?;
        if metadata.file_type().is_symlink() || resolved != absolute {
            return Err(format!("scientific changed path alias: {}", relative.display()));
        }
        return Ok(());
    }
    Err(format!("scientific changed path cannot be observed: {}", path.display()))
}

#[derive(Deserialize)]
struct PackageHeader {
    package: Package,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    package_id: String,
    version: String,
    dependencies: Vec<Dependency>,
}

#[derive(Deserialize)]
struct Dependency {
    package_id: String,
    version_req: VersionRequirement,
}

#[derive(Deserialize)]
struct VersionRequirement {
    operator: String,
    major: u64,
    minor: u64,
    patch: u64,
}

type Stamp = (u64, u64, u64, i64, i64, i64, i64);

fn stamp(metadata: &fs::Metadata) -> Stamp {
    (
        metadata.dev(),
        metadata.ino(),
        metadata.len(),
        metadata.mtime(),
        metadata.mtime_nsec(),
        metadata.ctime(),
        metadata.ctime_nsec(),
    )
}

struct Sources<'a> {
    root: &'a Path,
    stamps: BTreeMap<PathBuf, Stamp>,
}

impl Sources<'_> {
    fn observe(&mut self, path: &Path) -> Result<fs::Metadata, String> {
        if self.stamps.len() >= MAX_PATHS {
            return Err("scientific projection path limit exceeded".into());
        }
        let absolute = self.root.join(path);
        let metadata = fs::symlink_metadata(&absolute)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if absolute
            .canonicalize()
            .map_err(|error| format!("{}: {error}", path.display()))?
            != absolute
            || metadata.file_type().is_symlink()
        {
            return Err(format!("scientific path alias: {}", path.display()));
        }
        let observed = stamp(&metadata);
        if self
            .stamps
            .insert(path.to_owned(), observed)
            .is_some_and(|previous| previous != observed)
        {
            return Err(format!("scientific source changed: {}", path.display()));
        }
        Ok(metadata)
    }

    fn file(&mut self, path: &Path) -> Result<(), String> {
        if !self.observe(path)?.is_file() {
            return Err(format!("scientific file absent: {}", path.display()));
        }
        Ok(())
    }

    fn read(&mut self, path: &Path) -> Result<Vec<u8>, String> {
        let observed = self.observe(path)?;
        if !observed.is_file() {
            return Err(format!("scientific file absent: {}", path.display()));
        }
        let expected = stamp(&observed);
        let error = |error| format!("{}: {error}", path.display());
        let mut source = fs::File::open(self.root.join(path)).map_err(error)?;
        if stamp(&source.metadata().map_err(error)?) != expected {
            return Err(format!("scientific source changed before read: {}", path.display()));
        }
        let mut content = Vec::new();
        (&mut source)
            .take(MAX_HEADER_BYTES + 1)
            .read_to_end(&mut content)
            .map_err(error)?;
        if stamp(&source.metadata().map_err(error)?) != expected {
            return Err(format!("scientific source changed during read: {}", path.display()));
        }
        if content.len() as u64 > MAX_HEADER_BYTES {
            return Err(format!("scientific header exceeds limit: {}", path.display()));
        }
        Ok(content)
    }

    fn verify(&self) -> Result<(), String> {
        for (path, expected) in &self.stamps {
            let metadata = fs::symlink_metadata(self.root.join(path))
                .map_err(|error| format!("{}: {error}", path.display()))?;
            if stamp(&metadata) != *expected {
                return Err(format!("scientific projection input changed: {}", path.display()));
            }
        }
        Ok(())
    }

    fn discover(&mut self, inventory: PathBuf) -> Result<BTreeSet<PathBuf>, String> {
        let mut packages = BTreeSet::new();
        let mut pending = vec![inventory];
        while let Some(path) = pending.pop() {
            if !self.observe(&path)?.is_dir() {
                return Err(format!("scientific inventory directory absent: {}", path.display()));
            }
            let header = path.join("package.toml");
            match fs::symlink_metadata(self.root.join(&header)) {
                Ok(_) => {
                    packages.insert(path);
                    if packages.len() > MAX_PACKAGES {
                        return Err("scientific package inventory exceeds limit".into());
                    }
                    continue;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(format!("{}: {error}", header.display())),
            }
            let entries = fs::read_dir(self.root.join(&path))
                .map_err(|error| format!("{}: {error}", path.display()))?;
            for entry in entries {
                let entry = entry.map_err(|error| error.to_string())?;
                let kind = entry.file_type().map_err(|error| error.to_string())?;
                if kind.is_symlink() {
                    return Err(format!("scientific inventory alias: {}", entry.path().display()));
                }
                if kind.is_dir() {
                    let name = entry.file_name();
                    if name.to_str().is_none() {
                        return Err(format!("non-UTF8 scientific inventory entry: {}", entry.path().display()));
                    }
                    pending.push(path.join(name));
                    if pending.len() > MAX_PATHS {
                        return Err("scientific inventory directory limit exceeded".into());
                    }
                }
            }
        }
        Ok(packages)
    }
}

fn relative(path: &Path) -> Result<PathBuf, String> {
    normalize(path)
        .map(PathBuf::from)
        .map_err(|error| error.to_string())
}

struct PreparedFamily {
    name: String,
    consumers: Vec<String>,
    roots: BTreeSet<PathBuf>,
    documents: BTreeSet<PathBuf>,
    files: BTreeSet<PathBuf>,
    parser_fields: bool,
}

pub(super) fn read(root: &Path) -> Result<Projection, String> {
    let mut sources = Sources {
        root,
        stamps: BTreeMap::new(),
    };
    let catalog = Catalog::from_bytes(&sources.read(Path::new(CATALOG))?)
        .map_err(|error| error.to_string())?;
    let mut package_roots =
        sources.discover(relative(Path::new(&catalog.package_inventory))?)?;
    let families = resolve_families(catalog.families, None).map_err(|error| error.to_string())?;
    let mut prepared = Vec::new();
    let mut context = BTreeSet::new();
    for family in families {
        let mut roots = family
            .roots
            .iter()
            .map(|path| relative(Path::new(path)))
            .collect::<Result<BTreeSet<_>, _>>()?;
        let documents = family
            .documents
            .iter()
            .map(|path| relative(Path::new(path)))
            .collect::<Result<BTreeSet<_>, _>>()?;
        let mut files = family
            .substitutions
            .iter()
            .map(|path| relative(Path::new(path)))
            .collect::<Result<BTreeSet<_>, _>>()?;
        if let Some(manifest) = family.manifest {
            let manifest = relative(Path::new(&manifest))?;
            let input = Manifest::from_bytes(&sources.read(&manifest)?)
                .map_err(|error| error.to_string())?;
            let directory = manifest.parent().ok_or("scientific manifest parent absent")?;
            if let Some(diagnostics) = input.settings.diagnostics {
                let path = relative(&directory.join(diagnostics))?;
                context.insert(path.clone());
                files.insert(path);
            }
            for run in input.runs {
                for package in run.roots() {
                    roots.insert(relative(&directory.join(package))?);
                }
            }
            context.insert(manifest.clone());
            files.insert(manifest);
        }
        if roots.is_empty() || family.consumers.is_empty() || family.name.is_empty() {
            return Err(format!("scientific family {} lacks roots or consumers", family.name));
        }
        for path in documents.iter().chain(&files) {
            sources.file(path)?;
        }
        package_roots.extend(roots.iter().cloned());
        let parser_fields = match family.projection.as_deref() {
            Some("parser-fields") => true,
            Some("declarations" | "authentic-transport" | "synthetic-declarations") | None => false,
            Some(other) => return Err(format!("unknown scientific projection {other}")),
        };
        prepared.push(PreparedFamily {
            name: family.name,
            consumers: family.consumers,
            roots,
            documents,
            files,
            parser_fields,
        });
    }
    if prepared.is_empty() || package_roots.len() > MAX_PACKAGES {
        return Err("scientific family or package population is invalid".into());
    }
    let mut packages = BTreeMap::new();
    let mut identities = BTreeMap::new();
    let mut names = BTreeSet::new();
    for root in package_roots {
        let path = root.join("package.toml");
        let bytes = sources.read(&path)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let header: PackageHeader =
            toml::from_str(text).map_err(|error| format!("{}: {error}", path.display()))?;
        let package = header.package;
        let components = package.version.split('.').collect::<Vec<_>>();
        if package.name.is_empty()
            || package.package_id.len() != 32
            || !package.package_id.bytes().all(|part| {
                part.is_ascii_digit() || (b'a'..=b'f').contains(&part)
            })
            || components.len() != 3
            || components.iter().any(|part| part.parse::<u64>().is_err())
        {
            return Err(format!("invalid scientific package identity at {}", path.display()));
        }
        if !names.insert(package.name.clone())
            || identities.insert(package.package_id.clone(), root.clone()).is_some()
        {
            return Err(format!("ambiguous scientific package identity at {}", path.display()));
        }
        context.insert(path);
        packages.insert(root, package);
    }
    let mut families = Vec::new();
    for family in prepared {
        let mut trees = BTreeSet::new();
        let mut active = BTreeSet::new();
        let mut pending = family.roots.into_iter().map(|path| (path, false)).collect::<Vec<_>>();
        while let Some((path, expanded)) = pending.pop() {
            if expanded {
                active.remove(&path);
                trees.insert(path);
                continue;
            }
            if trees.contains(&path) {
                continue;
            }
            if !active.insert(path.clone()) {
                return Err(format!("scientific package dependency cycle at {}", path.display()));
            }
            let package = packages
                .get(&path)
                .ok_or_else(|| format!("missing scientific package {}", path.display()))?;
            pending.push((path.clone(), true));
            for dependency in &package.dependencies {
                let target = identities.get(&dependency.package_id).ok_or_else(|| {
                    format!("{}: missing scientific dependency {}", path.display(), dependency.package_id)
                })?;
                let req = &dependency.version_req;
                if req.operator != "exact"
                    || packages[target].version != format!("{}.{}.{}", req.major, req.minor, req.patch)
                {
                    return Err(format!("{}: unresolved scientific dependency version for {}", path.display(), target.display()));
                }
                pending.push((target.clone(), false));
            }
        }
        if family.documents.iter().any(|document| !trees.iter().any(|tree| document.starts_with(tree))) {
            return Err(format!("{}: scientific document outside its package closure", family.name));
        }
        let mut files = family.files;
        if family.parser_fields && !family.documents.is_empty() {
            // Parser-field projections have no semantic imports or typed data reads.
            // Every semantic projection conservatively retains all package trees.
            trees.clear();
        }
        files.extend(family.documents);
        families.push(FamilyReach { name: family.name, consumers: family.consumers, trees, files });
    }
    sources.verify()?;
    Ok(Projection { families, context })
}
