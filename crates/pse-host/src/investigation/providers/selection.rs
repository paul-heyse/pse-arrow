// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordinary selection is passive; explicit dependency widening reads locked offline metadata.

use super::super::*;
use crate::source_read::SourceRequest;
use std::{
    collections::BTreeSet,
    fs,
    os::unix::ffi::{OsStrExt as _, OsStringExt as _},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};
use tokio_util::sync::CancellationToken;

/// One acquired input. Raw bytes are retained only for providers requiring full input.
#[derive(Debug)]
pub struct ProviderInput {
    /// Original native path, relative to the selected repository where possible.
    pub path: PathBuf,
    /// Actual acquired origin, with revision selectors already resolved.
    pub source: SourceKind,
    /// Original bytes, not a lossy text projection.
    pub bytes: Vec<u8>,
    /// Observed snapshot/freshness premises.
    pub guard: Option<SourceGuard>,
}

/// Acquire the selected bytes without interpreting them as UTF-8 or preparing a service.
pub fn acquire(root: &Path, source: &SourceRequest) -> Result<ProviderInput, Error> {
    acquire_cancellable(root, source, &CancellationToken::new())
}

pub(super) fn acquire_cancellable(
    root: &Path,
    source: &SourceRequest,
    token: &CancellationToken,
) -> Result<ProviderInput, Error> {
    if token.is_cancelled() {
        return Err(Error::Cancelled);
    }
    let path = source
        .path()
        .strip_prefix(root)
        .unwrap_or(source.path())
        .to_path_buf();
    let (bytes, kind, mut guard) = match source {
        SourceRequest::Working { path } => {
            let path = root.join(path);
            let before = guard(&path)?;
            let bytes = read_cancellable(fs::File::open(&path)?, token)?;
            if guard(&path)? != before {
                return Err(Error::Stale(format!(
                    "source changed during read: {path:?}"
                )));
            }
            (bytes, SourceKind::Working, Some(before))
        }
        SourceRequest::Index { repository, path } => {
            let blob = crate::repository::source::read(repository, path, None)
                .map_err(|e| Error::Io(std::io::Error::other(e)))?;
            (
                blob.bytes,
                SourceKind::Index,
                Some(SourceGuard {
                    git_blob: Some(blob.blob),
                    ..Default::default()
                }),
            )
        }
        SourceRequest::Blob {
            repository,
            path,
            revision,
        } => {
            let blob = crate::repository::source::read(repository, path, Some(revision))
                .map_err(|e| Error::Io(std::io::Error::other(e)))?;
            (
                blob.bytes,
                SourceKind::Revision {
                    revision: blob
                        .revision
                        .ok_or_else(|| Error::Invalid("resolved revision absent".into()))?,
                },
                Some(SourceGuard {
                    git_blob: Some(blob.blob),
                    ..Default::default()
                }),
            )
        }
    };
    use sha2::{Digest as _, Sha256};
    let mut hash = Sha256::new();
    for chunk in bytes.chunks(64 * 1024) {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        hash.update(chunk);
    }
    let exact = super::hex(&hash.finalize());
    guard.get_or_insert_with(SourceGuard::default).sha256 = Some(exact);
    guard.get_or_insert_with(SourceGuard::default).root =
        Some(NativePath::from(root.canonicalize()?));
    Ok(ProviderInput {
        path,
        source: kind,
        bytes,
        guard,
    })
}

fn read_cancellable(
    mut reader: impl std::io::Read,
    token: &CancellationToken,
) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 64 * 1024];
    loop {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let size = reader.read(&mut chunk)?;
        if size == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..size]);
    }
    Ok(bytes)
}

/// Observe a source name and native identity without opening its contents.
pub(super) fn observe(root: &Path, source: &SourceRequest) -> Result<ProviderInput, Error> {
    use gix::bstr::ByteSlice as _;
    let path = source
        .path()
        .strip_prefix(root)
        .unwrap_or(source.path())
        .to_path_buf();
    let (kind, mut guard) = match source {
        SourceRequest::Working { path } => (SourceKind::Working, guard(&root.join(path))?),
        SourceRequest::Index { repository, path } => {
            let repository =
                gix::open(repository).map_err(|error| Error::Io(std::io::Error::other(error)))?;
            let index = repository
                .index()
                .map_err(|error| Error::Io(std::io::Error::other(error)))?;
            let entry = index
                .entry_by_path_and_stage(
                    path.as_os_str().as_bytes().as_bstr(),
                    gix::index::entry::Stage::Unconflicted,
                )
                .ok_or_else(|| Error::Stale("selected index entry disappeared".into()))?;
            (
                SourceKind::Index,
                SourceGuard {
                    git_blob: Some(entry.id.to_string()),
                    ..Default::default()
                },
            )
        }
        SourceRequest::Blob {
            repository,
            path,
            revision,
        } => {
            let repository =
                gix::open(repository).map_err(|error| Error::Io(std::io::Error::other(error)))?;
            let id = repository
                .rev_parse_single(revision.as_str())
                .map_err(|error| Error::Io(std::io::Error::other(error)))?;
            let tree = id
                .object()
                .and_then(|object| object.peel_to_tree())
                .map_err(|error| Error::Io(std::io::Error::other(error)))?;
            let entry = tree
                .lookup_entry_by_path(path)
                .map_err(|error| Error::Io(std::io::Error::other(error)))?
                .ok_or_else(|| Error::Stale("selected revision entry disappeared".into()))?;
            (
                SourceKind::Revision {
                    revision: id.to_string(),
                },
                SourceGuard {
                    git_blob: Some(entry.id().to_string()),
                    ..Default::default()
                },
            )
        }
    };
    guard.root = Some(NativePath::from(root.canonicalize()?));
    Ok(ProviderInput {
        path,
        source: kind,
        bytes: Vec::new(),
        guard: Some(guard),
    })
}

pub(super) fn guard(path: &Path) -> Result<SourceGuard, Error> {
    let metadata = fs::metadata(path)?;
    let modified_nanos = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|d| d.as_nanos().try_into().ok());
    Ok(SourceGuard {
        sha256: None,
        length: Some(metadata.len()),
        modified_nanos,
        observed: None,
        root: None,
        git_blob: None,
    })
}

fn git_paths(root: &Path, kind: &SourceKind) -> Result<Vec<PathBuf>, Error> {
    let repository = gix::open(root).map_err(|e| Error::Io(std::io::Error::other(e)))?;
    match kind {
        SourceKind::Revision { revision } => {
            let id = repository
                .rev_parse_single(revision.as_str())
                .map_err(|e| Error::Io(std::io::Error::other(e)))?;
            let tree = id
                .object()
                .and_then(|o| o.peel_to_tree())
                .map_err(|e| Error::Io(std::io::Error::other(e)))?;
            tree.traverse()
                .breadthfirst
                .files()
                .map_err(|e| Error::Io(std::io::Error::other(e)))
                .map(|entries| {
                    entries
                        .into_iter()
                        .filter(|e| e.mode.is_blob())
                        .map(|e| PathBuf::from(std::ffi::OsString::from_vec(e.filepath.to_vec())))
                        .collect()
                })
        }
        _ => {
            let index = repository
                .index()
                .map_err(|e| Error::Io(std::io::Error::other(e)))?;
            Ok(index
                .entries()
                .iter()
                .filter(|e| e.stage() == gix::index::entry::Stage::Unconflicted)
                .map(|e| PathBuf::from(std::ffi::OsString::from_vec(e.path(&index).to_vec())))
                .collect())
        }
    }
}

#[derive(Debug)]
pub(crate) struct Selection {
    include: globset::GlobSet,
    exclude: globset::GlobSet,
    types: ignore::types::Types,
    paths: Vec<PathBuf>,
    package_paths: Vec<PathBuf>,
    package_boundaries: Vec<PathBuf>,
    tracked: Option<BTreeSet<PathBuf>>,
    changed: Option<BTreeSet<PathBuf>>,
    generated: Option<crate::ownership::OwnershipMap>,
    dependencies: Vec<PathBuf>,
}
impl Selection {
    pub(crate) fn new(root: &Path, selector: &SourceSelector) -> Result<Self, Error> {
        Self::build(root, selector, true)
    }
    /// Prepared semantic owners already bound their dependency universe; reads never run Cargo.
    pub(crate) fn for_prepared_context(
        root: &Path,
        selector: &SourceSelector,
    ) -> Result<Self, Error> {
        Self::build(root, selector, false)
    }
    fn build(
        root: &Path,
        selector: &SourceSelector,
        capture_dependencies: bool,
    ) -> Result<Self, Error> {
        let mut dependencies = Vec::new();
        if selector.include_dependencies && selector.kind != SourceKind::Working {
            return Err(Error::Invalid("external dependency sources require the working universe; historical Cargo metadata is not inferred from current resolution".into()));
        }
        if selector.include_dependencies && capture_dependencies {
            let environment: crate::environment::Environment = std::env::vars_os().collect();
            let snapshot = crate::dependency::capture(root, &environment)
                .map_err(|error| Error::Unavailable(error.to_string()))?;
            let selected_root = root.canonicalize()?;
            for package in &snapshot.metadata.packages {
                if snapshot.metadata.workspace_members.contains(&package.id) {
                    continue;
                }
                let directory = package
                    .manifest_path
                    .parent()
                    .ok_or_else(|| Error::Invalid("dependency manifest has no parent".into()))?
                    .as_std_path()
                    .canonicalize()?;
                if !directory.starts_with(&selected_root) {
                    dependencies.push(directory);
                }
            }
            dependencies.sort();
            dependencies.dedup();
        }
        let build = |values: &[String]| -> Result<globset::GlobSet, Error> {
            let mut builder = globset::GlobSetBuilder::new();
            for value in values {
                builder.add(globset::Glob::new(value).map_err(|e| Error::Invalid(e.to_string()))?);
            }
            builder.build().map_err(|e| Error::Invalid(e.to_string()))
        };
        let mut types = ignore::types::TypesBuilder::new();
        types.add_defaults();
        for name in &selector.types {
            types.select(name);
        }
        let types = types.build().map_err(|e| Error::Invalid(e.to_string()))?;
        let tracked = if selector.tracked.is_some() {
            Some(git_paths(root, &SourceKind::Index)?.into_iter().collect())
        } else {
            None
        };
        let changed = if selector.changed {
            let report = crate::repository::observe(
                root,
                std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            )
            .map_err(|e| Error::Io(std::io::Error::other(e)))?;
            Some(
                report
                    .changes
                    .into_iter()
                    .map(|c| {
                        PathBuf::from(std::ffi::OsString::from_vec(c.path.as_bytes().to_vec()))
                    })
                    .collect(),
            )
        } else {
            None
        };
        let paths = selector
            .paths
            .iter()
            .map(|p| {
                let path = root.join(p).components().collect::<PathBuf>();
                path.strip_prefix(root).unwrap_or(&path).to_path_buf()
            })
            .collect::<Vec<_>>();
        let mut package_paths = Vec::new();
        let mut package_boundaries = Vec::new();
        if !selector.packages.is_empty() {
            let mut found = BTreeSet::new();
            let mut manifests = match &selector.kind {
                SourceKind::Working => ignore::WalkBuilder::new(root)
                    .hidden(!selector.include_hidden)
                    .ignore(!selector.include_ignored)
                    .git_ignore(!selector.include_ignored)
                    .git_global(!selector.include_ignored)
                    .git_exclude(!selector.include_ignored)
                    .build()
                    .filter_map(|entry| match entry {
                        Ok(entry) if entry.file_name() == "Cargo.toml" => Some(Ok(entry
                            .path()
                            .strip_prefix(root)
                            .unwrap_or(entry.path())
                            .to_path_buf())),
                        Ok(_) => None,
                        Err(error) => Some(Err(Error::Io(std::io::Error::other(error)))),
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                _ => git_paths(root, &selector.kind)?
                    .into_iter()
                    .filter(|path| path.file_name().is_some_and(|name| name == "Cargo.toml"))
                    .collect(),
            };
            manifests.extend(
                dependencies
                    .iter()
                    .map(|directory| directory.join("Cargo.toml")),
            );
            for path in manifests {
                let bytes = match &selector.kind {
                    SourceKind::Working => fs::read(root.join(&path))?,
                    SourceKind::Index => {
                        crate::repository::source::read(root, &path, None)
                            .map_err(|e| Error::Io(std::io::Error::other(e)))?
                            .bytes
                    }
                    SourceKind::Revision { revision } => {
                        crate::repository::source::read(root, &path, Some(revision))
                            .map_err(|e| Error::Io(std::io::Error::other(e)))?
                            .bytes
                    }
                };
                let manifest: toml::Value = toml::from_str(
                    std::str::from_utf8(&bytes).map_err(|e| Error::Invalid(e.to_string()))?,
                )
                .map_err(|e| Error::Invalid(e.to_string()))?;
                if let Some(name) = manifest
                    .get("package")
                    .and_then(|p| p.get("name"))
                    .and_then(toml::Value::as_str)
                {
                    if let Some(parent) = path.parent() {
                        package_boundaries.push(parent.to_path_buf());
                    }
                    if selector.packages.iter().any(|n| n == name) {
                        found.insert(name.to_owned());
                        if let Some(parent) = path.parent() {
                            package_paths.push(parent.to_path_buf());
                        }
                    }
                }
            }
            if selector.packages.iter().any(|n| !found.contains(n)) {
                return Err(Error::Invalid(
                    "selected package has no authored manifest in this source universe".into(),
                ));
            }
        }
        Ok(Self {
            include: build(&selector.globs)?,
            exclude: build(&selector.exclude)?,
            types,
            paths,
            package_paths,
            package_boundaries,
            tracked,
            changed,
            generated: crate::ownership::OwnershipMap::read(root).ok(),
            dependencies,
        })
    }
    pub(crate) fn accepts(&self, path: &Path, selector: &SourceSelector) -> bool {
        if path.components().any(|c| c.as_os_str() == ".git") {
            return false;
        }
        if !selector.include_dependencies
            && path.components().any(|c| {
                matches!(
                    c.as_os_str().to_str(),
                    Some(".venv" | "node_modules" | "vendor")
                )
            })
        {
            return false;
        }
        if !selector.include_generated
            && (path
                .components()
                .any(|c| matches!(c.as_os_str().to_str(), Some("target" | "build")))
                || self
                    .generated
                    .as_ref()
                    .is_some_and(|m| m.owner(path).is_ok_and(|o| o.is_some())))
        {
            return false;
        }
        if !self.paths.is_empty() && !self.paths.iter().any(|p| path == p || path.starts_with(p)) {
            return false;
        }
        if !self.package_paths.is_empty()
            && !self
                .package_boundaries
                .iter()
                .filter(|root| path.starts_with(root))
                .max_by_key(|root| root.components().count())
                .is_some_and(|owner| self.package_paths.contains(owner))
        {
            return false;
        }
        if !self.include.is_empty() && !self.include.is_match(path) {
            return false;
        }
        if self.exclude.is_match(path) || self.types.matched(path, false).is_ignore() {
            return false;
        }
        if let Some(expected) = selector.tracked
            && self
                .tracked
                .as_ref()
                .is_none_or(|set| set.contains(path) != expected)
        {
            return false;
        }
        !self.changed.as_ref().is_some_and(|set| !set.contains(path))
    }
    pub(crate) fn descends(&self, path: &Path) -> bool {
        let intersects = |roots: &[PathBuf]| {
            roots.is_empty()
                || roots
                    .iter()
                    .any(|root| path.starts_with(root) || root.starts_with(path))
        };
        intersects(&self.paths) && intersects(&self.package_paths)
    }
    pub(crate) fn dependency_roots(&self) -> impl Iterator<Item = &Path> {
        self.dependencies
            .iter()
            .filter(|candidate| {
                !self
                    .dependencies
                    .iter()
                    .any(|parent| parent != *candidate && candidate.starts_with(parent))
            })
            .map(PathBuf::as_path)
    }
}

/// Visit selected operands without acquiring their content or preparing any provider.
pub(super) fn visit(
    root: &Path,
    selector: &SourceSelector,
    token: &CancellationToken,
    emit: &mut dyn FnMut(Result<SourceRequest, Error>) -> Result<bool, Error>,
) -> Result<(), Error> {
    if token.is_cancelled() {
        return Ok(());
    }
    if selector.include_dependencies && selector.kind != SourceKind::Working {
        return Err(Error::Invalid("external dependency sources require the working universe; historical metadata is not inferred from current resolution".into()));
    }
    let resolved = if let SourceKind::Revision { revision } = &selector.kind {
        let repository = gix::open(root).map_err(|e| Error::Io(std::io::Error::other(e)))?;
        let revision = repository
            .rev_parse_single(revision.as_str())
            .map_err(|e| Error::Io(std::io::Error::other(e)))?
            .to_string();
        let mut resolved = selector.clone();
        resolved.kind = SourceKind::Revision { revision };
        Some(resolved)
    } else {
        None
    };
    let selector = resolved.as_ref().unwrap_or(selector);
    let selected = std::sync::Arc::new(Selection::new(root, selector)?);
    visit_selected(root, selector, token, selected, emit)
}

fn visit_selected(
    root: &Path,
    selector: &SourceSelector,
    token: &CancellationToken,
    selected: std::sync::Arc<Selection>,
    emit: &mut dyn FnMut(Result<SourceRequest, Error>) -> Result<bool, Error>,
) -> Result<(), Error> {
    match &selector.kind {
        SourceKind::Working => {
            for path in &selector.paths {
                if let Err(error) = fs::symlink_metadata(root.join(path))
                    && !emit(Err(Error::Io(error)))?
                {
                    return Ok(());
                }
            }
            let mut walker = ignore::WalkBuilder::new(root);
            for dependency in selected.dependency_roots() {
                walker.add(dependency);
            }
            walker
                .hidden(!selector.include_hidden)
                .ignore(!selector.include_ignored)
                .git_ignore(!selector.include_ignored)
                .git_global(!selector.include_ignored)
                .git_exclude(!selector.include_ignored)
                .follow_links(false);
            let filter = selected.clone();
            let filter_root = root.to_path_buf();
            walker.filter_entry(move |entry| {
                !entry.file_type().is_some_and(|kind| kind.is_dir())
                    || filter.descends(
                        entry
                            .path()
                            .strip_prefix(&filter_root)
                            .unwrap_or(entry.path()),
                    )
            });
            let mut seen = BTreeSet::new();
            // Explicit files participate even when recursive ignore rules omit them.
            for path in &selector.paths {
                let absolute = root.join(path);
                let relative = absolute.strip_prefix(root).unwrap_or(&absolute);
                if absolute.is_file() && selected.accepts(relative, selector) {
                    seen.insert(relative.to_path_buf());
                    if !emit(Ok(SourceRequest::Working { path: absolute }))? {
                        return Ok(());
                    }
                }
            }
            for entry in walker.build() {
                if token.is_cancelled() {
                    break;
                }
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        if !emit(Err(Error::Io(std::io::Error::other(e))))? {
                            break;
                        }
                        continue;
                    }
                };
                if !entry.file_type().is_some_and(|t| t.is_file()) {
                    continue;
                }
                let path = entry.path().strip_prefix(root).unwrap_or(entry.path());
                if selected.accepts(path, selector)
                    && seen.insert(path.to_path_buf())
                    && !emit(Ok(SourceRequest::Working {
                        path: entry.path().to_path_buf(),
                    }))?
                {
                    break;
                }
            }
        }
        kind => {
            for path in git_paths(root, kind)? {
                if token.is_cancelled() {
                    break;
                }
                if !selected.accepts(&path, selector) {
                    continue;
                }
                let source = match kind {
                    SourceKind::Index => SourceRequest::Index {
                        repository: root.to_path_buf(),
                        path,
                    },
                    SourceKind::Revision { revision } => SourceRequest::Blob {
                        repository: root.to_path_buf(),
                        path,
                        revision: revision.clone(),
                    },
                    SourceKind::Working => continue,
                };
                if !emit(Ok(source))? {
                    break;
                }
            }
        }
    }
    Ok(())
}

/// Enumerate operands for explicitly stable or composed consumers; contents remain unacquired.
pub fn select(
    root: &Path,
    selector: &SourceSelector,
    token: &CancellationToken,
) -> Result<Vec<SourceRequest>, Error> {
    let mut sources = Vec::new();
    visit(root, selector, token, &mut |source| {
        sources.push(source?);
        Ok(true)
    })?;
    Ok(sources)
}

pub(crate) fn select_prepared_context(
    root: &Path,
    selector: &SourceSelector,
    token: &CancellationToken,
) -> Result<Vec<SourceRequest>, Error> {
    let selected = std::sync::Arc::new(Selection::for_prepared_context(root, selector)?);
    let mut sources = Vec::new();
    visit_selected(root, selector, token, selected, &mut |source| {
        sources.push(source?);
        Ok(true)
    })?;
    Ok(sources)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whole_source_acquisition_checks_cancellation_between_native_reads() {
        struct CancelOnRead {
            token: CancellationToken,
            reads: usize,
        }
        impl std::io::Read for CancelOnRead {
            fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
                self.reads += 1;
                bytes.fill(b'x');
                self.token.cancel();
                Ok(bytes.len())
            }
        }
        let token = CancellationToken::new();
        let mut reader = CancelOnRead {
            token: token.clone(),
            reads: 0,
        };
        assert!(matches!(
            read_cancellable(&mut reader, &token),
            Err(Error::Cancelled)
        ));
        assert_eq!(reader.reads, 1);
    }
}
