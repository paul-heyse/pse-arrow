// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native Git observations. Object/ref/index/worktree facts retain separate meanings.

use gix::bstr::ByteSlice;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    os::unix::ffi::OsStrExt,
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

/// Failure to observe the requested repository, never an empty or clean result.
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    /// Preserve Gitoxide's structured source chain.
    #[error("Git repository observation: {0}")]
    Git(#[from] gix::Error),
    /// A source path could not be resolved.
    #[error("Git repository input/output: {0}")]
    Io(#[from] std::io::Error),
    /// The selected source cannot establish the requested observation.
    #[error("Git repository premise: {0}")]
    Premise(String),
}
pse_diagnostics::impl_diagnostic! {
    RepositoryError,
    code(this) { Some(match this {
        Self::Git(_) => pse_diagnostics::DiagnosticCode::RuntimeInfrastructure,
        Self::Io(_) => pse_diagnostics::DiagnosticCode::HostIo,
        Self::Premise(_) => pse_diagnostics::DiagnosticCode::HostOwnership,
    }) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}

/// Git names are bytes; a display conversion never replaces a non-Unicode identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "encoding", content = "value", rename_all = "snake_case")]
pub enum GitName {
    /// Exact UTF-8 representation.
    Utf8(String),
    /// Original Unix/Git bytes where UTF-8 cannot represent the identity.
    Bytes(Vec<u8>),
}
impl GitName {
    fn new(bytes: &[u8]) -> Self {
        match std::str::from_utf8(bytes) {
            Ok(text) => Self::Utf8(text.to_owned()),
            Err(_) => Self::Bytes(bytes.to_vec()),
        }
    }
    /// Exact source bytes for joins and deterministic ordering.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Utf8(value) => value.as_bytes(),
            Self::Bytes(value) => value,
        }
    }
}

/// Attached, detached and unborn HEAD cannot be confused by optional sentinel fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum HeadState {
    /// A symbolic branch exists without a commit.
    Unborn {
        /// Full symbolic reference name.
        reference: GitName,
    },
    /// HEAD names a branch and its observed commit.
    Attached {
        /// Full symbolic reference name.
        reference: GitName,
        /// Object ID with the repository's own hash algorithm.
        commit: String,
    },
    /// HEAD directly names its observed commit.
    Detached {
        /// Object ID with the repository's own hash algorithm.
        commit: String,
    },
}

/// The independently compared Git representations.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ChangeLayer {
    /// HEAD tree versus the staging index.
    Staged,
    /// Index versus tracked working-tree content.
    Worktree,
    /// Untracked non-ignored working-tree entries.
    Untracked,
}
/// The upstream algorithm's actual change classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    /// Added entry.
    Added,
    /// Removed entry.
    Removed,
    /// Content or mode changed.
    Modified,
    /// File type changed.
    TypeChanged,
    /// Unmerged index entry.
    Conflict,
    /// Index entry marked intent-to-add.
    IntentToAdd,
    /// Upstream classified a rename.
    Renamed,
    /// Upstream classified a copy.
    Copied,
}
/// One observed layer change. A path may correctly occur in multiple layers.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PathChange {
    /// Checkout-relative original bytes.
    pub path: GitName,
    /// Separate staged, tracked and untracked meanings.
    pub layer: ChangeLayer,
    /// Native classification, without parsing porcelain or debug strings.
    pub kind: ChangeKind,
    /// Original source for an actual upstream rewrite result.
    pub previous_path: Option<GitName>,
}
/// One live observation, not an atomic multi-file snapshot or a mutation capability.
#[derive(Debug, Serialize, JsonSchema)]
pub struct RepositoryObservation {
    /// Exact working tree opened by the reader.
    pub worktree: GitName,
    /// Per-worktree Git administration directory.
    pub git_dir: GitName,
    /// Shared repository administration directory.
    pub common_dir: GitName,
    /// Actual index inspected, including any explicit Git index selection.
    pub index_path: GitName,
    /// HEAD before status traversal.
    pub head_before: HeadState,
    /// HEAD observed after traversal; equality does not prove an atomic filesystem snapshot.
    pub head_after: HeadState,
    /// Stable byte-path/layer ordering, independent of upstream parallel event order.
    pub changes: Vec<PathChange>,
    /// Distinct changed paths, including both endpoints of any reported rewrite.
    pub dirty_path_count: usize,
}

fn head(repository: &gix::Repository) -> Result<HeadState, RepositoryError> {
    let head = repository.head()?;
    let reference = head
        .referent_name()
        .map(|name| GitName::new(name.as_bstr().as_bytes()));
    let commit = head.id().map(|id| id.to_string());
    match (reference, commit) {
        (Some(reference), None) if head.is_unborn() => Ok(HeadState::Unborn { reference }),
        (Some(reference), Some(commit)) => Ok(HeadState::Attached { reference, commit }),
        (None, Some(commit)) if head.is_detached() => Ok(HeadState::Detached { commit }),
        _ => Err(RepositoryError::Premise(
            "HEAD does not establish an attached, detached or unborn identity".into(),
        )),
    }
}

/// Observe full status with upstream configuration/attributes/ignore semantics. Never writes
/// index refreshes, references or working-tree files. Cancellation cannot become clean status.
/// Rewrite inference is deliberately disabled: changed-path selection needs both source and
/// destination, without fuzzy similarity or rename-candidate limits changing its population.
pub fn observe(
    root: &Path,
    interrupt: Arc<AtomicBool>,
) -> Result<RepositoryObservation, RepositoryError> {
    let root = root.canonicalize()?;
    let mut repository = gix::open(&root)?;
    repository.make_paths_absolute()?;
    let worktree = repository
        .workdir()
        .ok_or_else(|| RepositoryError::Premise("status requires a working tree".into()))?
        .to_owned();
    if worktree.canonicalize()? != root {
        return Err(RepositoryError::Premise(
            "status opened a different working tree than requested".into(),
        ));
    }
    let head_before = head(&repository)?;
    let status = repository
        .status(gix::progress::Discard)?
        .untracked_files(gix::status::UntrackedFiles::Files)
        .tree_index_track_renames(gix::status::tree_index::TrackRenames::Disabled)
        .should_interrupt_owned(interrupt.clone());
    let mut changes = Vec::new();
    for item in status.into_iter(std::iter::empty::<gix::bstr::BString>())? {
        let item = item?;
        let path = GitName::new(item.location().as_bytes());
        let (layer, kind, previous_path) = match item {
            gix::status::Item::TreeIndex(change) => {
                use gix::diff::index::ChangeRef;
                let (kind, previous_path) = match change {
                    ChangeRef::Addition { .. } => (ChangeKind::Added, None),
                    ChangeRef::Deletion { .. } => (ChangeKind::Removed, None),
                    ChangeRef::Modification { .. } => (ChangeKind::Modified, None),
                    ChangeRef::Rewrite {
                        source_location,
                        copy,
                        ..
                    } => (
                        if copy {
                            ChangeKind::Copied
                        } else {
                            ChangeKind::Renamed
                        },
                        Some(GitName::new(source_location.as_bytes())),
                    ),
                };
                (ChangeLayer::Staged, kind, previous_path)
            }
            gix::status::Item::IndexWorktree(change) => {
                use gix::status::index_worktree::iter::Summary;
                let Some(summary) = change.summary() else {
                    continue;
                };
                let kind = match summary {
                    Summary::Added => ChangeKind::Added,
                    Summary::Removed => ChangeKind::Removed,
                    Summary::Modified => ChangeKind::Modified,
                    Summary::TypeChange => ChangeKind::TypeChanged,
                    Summary::Conflict => ChangeKind::Conflict,
                    Summary::IntentToAdd => ChangeKind::IntentToAdd,
                    Summary::Renamed => ChangeKind::Renamed,
                    Summary::Copied => ChangeKind::Copied,
                };
                let layer = if matches!(
                    change,
                    gix::status::index_worktree::Item::DirectoryContents { .. }
                ) {
                    ChangeLayer::Untracked
                } else {
                    ChangeLayer::Worktree
                };
                let previous = match change {
                    gix::status::index_worktree::Item::Rewrite { source, .. } => {
                        Some(GitName::new(source.rela_path().as_bytes()))
                    }
                    _ => None,
                };
                (layer, kind, previous)
            }
        };
        changes.push(PathChange {
            path,
            layer,
            kind,
            previous_path,
        });
    }
    if interrupt.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(RepositoryError::Premise(
            "status was cancelled before complete observation".into(),
        ));
    }
    changes.sort_by(|left, right| {
        (left.path.as_bytes(), left.layer).cmp(&(right.path.as_bytes(), right.layer))
    });
    let dirty_path_count = changes
        .iter()
        .flat_map(|change| {
            std::iter::once(change.path.as_bytes())
                .chain(change.previous_path.as_ref().map(GitName::as_bytes))
        })
        .collect::<BTreeSet<_>>()
        .len();
    let common_dir = repository.common_dir().canonicalize()?;
    Ok(RepositoryObservation {
        worktree: GitName::new(worktree.as_os_str().as_bytes()),
        git_dir: GitName::new(repository.git_dir().as_os_str().as_bytes()),
        common_dir: GitName::new(common_dir.as_os_str().as_bytes()),
        index_path: GitName::new(repository.index_path().as_os_str().as_bytes()),
        head_before,
        head_after: head(&repository)?,
        changes,
        dirty_path_count,
    })
}

/// Read-only identity and HEAD for one main/linked worktree; no dirty or live-process claim.
#[derive(Debug, Serialize, JsonSchema)]
pub struct WorktreeObservation {
    /// Original per-worktree administration path bytes.
    pub git_dir: GitName,
    /// Native configured worktree path, absent for bare repositories.
    pub worktree: Option<GitName>,
    /// Whether this is the main repository rather than a linked worktree.
    pub main: bool,
    /// Native bare-repository configuration.
    pub bare: bool,
    /// Original HEAD observation; absence is accompanied by an independent failure.
    pub head: Option<HeadState>,
}
/// One independent enumeration/HEAD failure with location when upstream supplies it.
#[derive(Debug)]
pub struct WorktreeFailure {
    /// Administration directory known before the failed HEAD read, if available.
    pub git_dir: Option<GitName>,
    /// Original typed source chain.
    pub error: RepositoryError,
}
/// Every independently readable entry survives another worktree's failure.
#[derive(Debug)]
pub struct WorktreeInventory {
    /// Actual observed entries, sorted by original administration-path bytes.
    pub entries: Vec<WorktreeObservation>,
    /// Original failures in observation order, without guessed replacement entries.
    pub failures: Vec<WorktreeFailure>,
}
/// Enumerate main and linked worktrees through Gitoxide without shelling out or mutations.
/// Missing/inaccessible linked directories remain representable by the native iterator.
pub fn observe_worktrees(root: &Path) -> Result<WorktreeInventory, RepositoryError> {
    let root = root.canonicalize()?;
    let mut repository = gix::open(&root)?;
    repository.make_paths_absolute()?;
    let mut entries = Vec::new();
    let mut failures = Vec::new();
    for entry in repository.worktrees_including_main()? {
        match entry {
            Err(error) => failures.push(WorktreeFailure {
                git_dir: None,
                error: error.into(),
            }),
            Ok(repository) => {
                let git_dir = GitName::new(repository.git_dir().as_os_str().as_bytes());
                let head = match head(&repository) {
                    Ok(head) => Some(head),
                    Err(error) => {
                        failures.push(WorktreeFailure {
                            git_dir: Some(git_dir.clone()),
                            error,
                        });
                        None
                    }
                };
                entries.push(WorktreeObservation {
                    git_dir,
                    worktree: repository
                        .workdir()
                        .map(|path| GitName::new(path.as_os_str().as_bytes())),
                    main: !matches!(repository.kind(), gix::repository::Kind::LinkedWorkTree),
                    bare: repository.is_bare(),
                    head,
                });
            }
        }
    }
    entries.sort_by(|left, right| left.git_dir.as_bytes().cmp(right.git_dir.as_bytes()));
    Ok(WorktreeInventory { entries, failures })
}

#[cfg(test)]
mod tests;
