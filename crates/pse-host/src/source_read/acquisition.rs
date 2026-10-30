// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared exact-source addressing for source readers and proposal producers.
use super::{SourceReadError, source_digest};
use crate::repository::GitName;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

/// Explicit acquisition operand; working files need no repository or Cargo preparation.
#[derive(Clone, Debug)]
pub enum SourceRequest {
    /// Read exactly one working file.
    Working {
        /// Exact native working-file operand.
        path: PathBuf,
    },
    /// Read the raw stage-zero index blob.
    Index {
        /// Explicit repository root.
        repository: PathBuf,
        /// Repository-relative native path.
        path: PathBuf,
    },
    /// Resolve a revision, then read its raw tree path without checking it out.
    Blob {
        /// Explicit repository root.
        repository: PathBuf,
        /// Revision selector resolved to an object before constructing source identity.
        revision: String,
        /// Repository-relative native path.
        path: PathBuf,
    },
}
impl SourceRequest {
    /// Exact native operand used for format inference and human navigation.
    pub fn path(&self) -> &Path {
        match self {
            Self::Working { path } | Self::Index { path, .. } | Self::Blob { path, .. } => path,
        }
    }
    /// Acquire once; original text remains available to all downstream consumers.
    pub fn acquire(&self) -> Result<SourceText, SourceReadError> {
        match self {
            Self::Working { path } => {
                let absolute = if path.is_absolute() {
                    path.clone()
                } else {
                    std::env::current_dir()?.join(path)
                };
                let text = std::fs::read_to_string(&absolute)?;
                SourceText::new(absolute, text)
            }
            Self::Index { repository, path }
            | Self::Blob {
                repository, path, ..
            } => {
                let revision = match self {
                    Self::Blob { revision, .. } => Some(revision.as_str()),
                    _ => None,
                };
                let blob = crate::repository::source::read(repository, path, revision)?;
                let text = String::from_utf8(blob.bytes).map_err(|error| {
                    SourceReadError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, error))
                })?;
                let origin = if let Some(revision) = blob.revision {
                    SourceOrigin::Blob {
                        git_dir: blob.git_dir,
                        revision,
                        blob: blob.blob,
                    }
                } else {
                    SourceOrigin::Index {
                        git_dir: blob.git_dir,
                        index_path: blob.index_path.ok_or(SourceReadError::InvalidIdentity)?,
                        blob: blob.blob,
                    }
                };
                Ok(SourceText {
                    identity: SourceIdentity {
                        path: path.as_os_str().to_owned(),
                        source_sha256: source_digest(&text),
                        origin,
                    },
                    text,
                })
            }
        }
    }
}

/// Resolved source kind and repository context; no mutable revision selector survives.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceOrigin {
    /// Independently acquired working bytes, not an atomic multi-file snapshot.
    Working,
    /// Raw index object, retaining the actual selected index.
    Index {
        /// Per-worktree repository administration directory.
        git_dir: GitName,
        /// Actual selected index path.
        index_path: GitName,
        /// Resolved raw blob object ID.
        blob: String,
    },
    /// Raw committed object; the original selector has been resolved.
    Blob {
        /// Per-worktree repository administration directory.
        git_dir: GitName,
        /// Resolved revision object ID, never a mutable selector.
        revision: String,
        /// Resolved raw blob object ID.
        blob: String,
    },
}
/// Exact native path, source kind/context and digest of the original UTF-8 bytes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SourceIdentity {
    path: OsString,
    source_sha256: String,
    origin: SourceOrigin,
}
impl SourceIdentity {
    /// Native operand: absolute for working files, Git-relative for index/tree paths.
    pub fn path(&self) -> &Path {
        Path::new(&self.path)
    }
    /// Digest includes every original byte, trivia and malformed text.
    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }
    /// Resolved source kind and repository context.
    pub fn origin(&self) -> &SourceOrigin {
        &self.origin
    }
    /// Same addressed input independent of its observed content generation.
    pub fn same_operand(&self, other: &Self) -> bool {
        if self.path != other.path {
            return false;
        }
        match (&self.origin, &other.origin) {
            (SourceOrigin::Working, SourceOrigin::Working) => true,
            (
                SourceOrigin::Index {
                    git_dir: a,
                    index_path: x,
                    ..
                },
                SourceOrigin::Index {
                    git_dir: b,
                    index_path: y,
                    ..
                },
            ) => a == b && x == y,
            (SourceOrigin::Blob { git_dir: a, .. }, SourceOrigin::Blob { git_dir: b, .. }) => {
                a == b
            }
            _ => false,
        }
    }
}
/// Immutable original retained by acquisition, reading and proposal consumers.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct SourceText {
    pub(super) identity: SourceIdentity,
    pub(super) text: String,
}
impl SourceText {
    /// Bind explicitly supplied absolute working bytes without filesystem access.
    pub fn new(path: PathBuf, text: String) -> Result<Self, SourceReadError> {
        if !path.is_absolute() {
            return Err(SourceReadError::RelativePath(path));
        }
        Ok(Self {
            identity: SourceIdentity {
                path: path.into_os_string(),
                source_sha256: source_digest(&text),
                origin: SourceOrigin::Working,
            },
            text,
        })
    }
    /// Exact source address.
    pub fn identity(&self) -> &SourceIdentity {
        &self.identity
    }
    /// All original text; no parser round trip or normalization.
    pub fn text(&self) -> &str {
        &self.text
    }
}
