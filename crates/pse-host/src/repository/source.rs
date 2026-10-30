// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read-only index/tree addressing through the repository owner.

use super::{GitName, RepositoryError};
use gix::bstr::ByteSlice as _;
use std::{
    os::unix::ffi::OsStrExt as _,
    path::{Component, Path, PathBuf},
};

/// Discover the actual Git root for source-only commands, without product checkout policy.
pub fn root(directory: &Path) -> Result<PathBuf, RepositoryError> {
    let mut repository = gix::discover(directory)?;
    repository.make_paths_absolute()?;
    Ok(repository
        .workdir()
        .unwrap_or_else(|| repository.git_dir())
        .to_owned())
}

/// One resolved raw blob; selectors such as HEAD no longer remain mutable identities.
#[derive(Debug)]
pub struct BlobSource {
    /// Per-worktree repository administration directory.
    pub git_dir: GitName,
    /// The exact index path when selected.
    pub index_path: Option<GitName>,
    /// Selected commit/tree object before path traversal.
    pub revision: Option<String>,
    /// Raw blob object identity.
    pub blob: String,
    /// Original blob bytes, without checkout filters.
    pub bytes: Vec<u8>,
}

/// Select the stage-zero index blob, or a path in an explicitly selected revision tree.
/// Paths are Git-relative byte paths. Conflicts, missing entries and non-blobs refuse.
pub fn read(
    root: &Path,
    path: &Path,
    revision: Option<&str>,
) -> Result<BlobSource, RepositoryError> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(RepositoryError::Premise(
            "source path must be repository-relative without . or .. components".into(),
        ));
    }
    let mut repository = gix::open(root.canonicalize()?)?;
    repository.make_paths_absolute()?;
    let (id, resolved_revision, index_path) = if let Some(selector) = revision {
        let revision = repository.rev_parse_single(selector)?;
        let resolved = revision.to_string();
        let tree = revision.object()?.peel_to_tree()?;
        let entry = tree.lookup_entry_by_path(path)?.ok_or_else(|| {
            RepositoryError::Premise(format!(
                "source path {path:?} is absent from revision {resolved}"
            ))
        })?;
        (entry.id().detach(), Some(resolved), None)
    } else {
        let index = repository.index()?;
        let entry = index
            .entry_by_path_and_stage(
                path.as_os_str().as_bytes().as_bstr(),
                gix::index::entry::Stage::Unconflicted,
            )
            .ok_or_else(|| {
                RepositoryError::Premise(format!(
                    "source path {path:?} has no stage-zero index entry (missing or conflicted)"
                ))
            })?;
        (
            entry.id,
            None,
            Some(GitName::new(repository.index_path().as_os_str().as_bytes())),
        )
    };
    let blob = repository.find_object(id)?.try_into_blob()?;
    Ok(BlobSource {
        git_dir: GitName::new(repository.git_dir().as_os_str().as_bytes()),
        index_path,
        revision: resolved_revision,
        blob: id.to_string(),
        bytes: blob.data.clone(),
    })
}
