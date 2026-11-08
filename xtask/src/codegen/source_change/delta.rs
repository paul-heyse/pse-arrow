// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact generator-owned inventory, persisted postimages and guarded recovery.
use super::{Scope, hex, snapshot};
use crate::codegen::{collect_files, normal_relative, publication, reject_symlinks};
use anyhow::{Context, Result, ensure};
use pse_codegen::codegen::provenance::protocol::{MAP_PATH, OutputMap, ScopeKind, VERSION};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const MAX_GENERATED_FILE: u64 = 64 * 1024 * 1024;
const MAX_STAGED_BYTES: u64 = 256 * 1024 * 1024;

pub(super) fn scopes(root: &Path) -> Result<Vec<Scope>> {
    reject_symlinks(root, Path::new(MAP_PATH))?;
    let map: OutputMap = serde_json::from_slice(&snapshot::bounded_read(
        &root.join(MAP_PATH),
        4 * 1024 * 1024,
    )?)?;
    ensure!(
        map.schema_version == VERSION,
        "unsupported generated ownership map version"
    );
    let scopes: BTreeSet<_> = map
        .owners
        .into_iter()
        .flat_map(|owner| owner.outputs)
        .collect();
    ensure!(
        scopes
            .iter()
            .any(|scope| scope.path == MAP_PATH && scope.kind == ScopeKind::File),
        "generated ownership map has no exact self owner"
    );
    ensure!(
        scopes
            .iter()
            .all(|scope| normal_relative(Path::new(&scope.path))
                && ![".git", "target", "build", ".local"]
                    .iter()
                    .any(|root| Path::new(&scope.path).starts_with(root))),
        "generated ownership map contains an unsafe output scope"
    );
    Ok(scopes.into_iter().collect())
}

pub(super) fn inventory(root: &Path, scopes: &[Scope]) -> Result<BTreeSet<PathBuf>> {
    let mut files = BTreeSet::new();
    for scope in scopes {
        let path = Path::new(&scope.path);
        ensure!(normal_relative(path), "unsafe output scope");
        reject_symlinks(root, path)?;
        match scope.kind {
            ScopeKind::File => {
                if publication::observe(root, path)?.is_some() {
                    files.insert(path.to_owned());
                }
            }
            ScopeKind::Tree => collect_files(root, path, &mut files)?,
        }
    }
    Ok(files)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Change {
    path: PathBuf,
    before_sha256: Option<String>,
    after_sha256: Option<String>,
}

impl Change {
    pub(super) fn postimage(&self) -> Option<(&Path, &str)> {
        Some((&self.path, self.after_sha256.as_deref()?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Delta {
    pub(super) scopes: Vec<Scope>,
    unchanged_sha256: String,
    pub(super) changes: Vec<Change>,
}

fn observe(root: &Path, path: &Path) -> Result<Option<String>> {
    Ok(publication::observe(root, path)?.map(|file| hex(&file.identity)))
}

fn add_identity(digest: &mut Sha256, path: &Path, sha256: &str) {
    digest.update(path.as_os_str().as_encoded_bytes());
    digest.update([0]);
    digest.update(sha256.as_bytes());
}

fn decode_identity(text: &str) -> Result<publication::Identity> {
    ensure!(super::is_hex(text, 64), "invalid retained file digest");
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)?;
    }
    Ok(bytes)
}

impl Delta {
    pub(super) fn stage(
        root: &Path,
        candidate: &Path,
        custody: &Path,
        scopes: Vec<Scope>,
    ) -> Result<Self> {
        let original = inventory(root, &scopes)?;
        let produced = inventory(candidate, &scopes)?;
        let mut unchanged = Sha256::new();
        let mut changes = Vec::new();
        let mut retained_bytes = 0;
        fs::create_dir_all(custody)?;
        for path in original.union(&produced) {
            let before_sha256 = observe(root, path)?;
            let after_sha256 = observe(candidate, path)?;
            if before_sha256 == after_sha256 {
                if let Some(identity) = &before_sha256 {
                    add_identity(&mut unchanged, path, identity);
                }
                continue;
            }
            if let Some(identity) = &after_sha256 {
                let bytes = snapshot::bounded_read(&candidate.join(path), MAX_GENERATED_FILE)?;
                ensure!(
                    hex(&Sha256::digest(&bytes)) == *identity,
                    "candidate changed during staging"
                );
                retained_bytes += bytes.len() as u64;
                ensure!(
                    retained_bytes <= MAX_STAGED_BYTES,
                    "generated postimages exceed the preview custody limit"
                );
                snapshot::atomic_write(&custody.join(identity), &bytes)?;
            }
            changes.push(Change {
                path: path.clone(),
                before_sha256,
                after_sha256,
            });
        }
        Ok(Self {
            scopes,
            unchanged_sha256: hex(&unchanged.finalize()),
            changes,
        })
    }

    pub(super) fn preflight(&self, root: &Path, custody: &Path, recovery: bool) -> Result<()> {
        let mut remaining = inventory(root, &self.scopes)?;
        let mut seen = BTreeSet::new();
        for change in &self.changes {
            ensure!(
                normal_relative(&change.path)
                    && self.scopes.iter().any(|scope| scope.contains(&change.path))
                    && seen.insert(&change.path),
                "invalid retained publication destination"
            );
            remaining.remove(&change.path);
            let current = observe(root, &change.path)?;
            ensure!(
                current == change.before_sha256 || (recovery && current == change.after_sha256),
                "generated destination changed since preview: {}",
                change.path.display()
            );
            change.bytes(custody)?;
        }
        let mut unchanged = Sha256::new();
        for path in remaining {
            let identity =
                observe(root, &path)?.context("generated inventory changed during preflight")?;
            add_identity(&mut unchanged, &path, &identity);
        }
        ensure!(
            hex(&unchanged.finalize()) == self.unchanged_sha256,
            "unchanged generated inventory changed since preview"
        );
        Ok(())
    }

    pub(super) fn publish(&self, root: &Path, custody: &Path) -> Result<()> {
        for change in &self.changes {
            change
                .publish(root, custody)
                .with_context(|| format!("publishing {}", change.path.display()))?;
        }
        Ok(())
    }

    pub(super) fn all_postimages(&self, root: &Path) -> Result<bool> {
        for change in &self.changes {
            if observe(root, &change.path)? != change.after_sha256 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn surviving(&self, root: &Path) -> Vec<SurvivingFile> {
        self.changes
            .iter()
            .map(|change| {
                let current = observe(root, &change.path);
                let state = match &current {
                    Ok(identity) if *identity == change.after_sha256 => SurvivingState::Postimage,
                    Ok(identity) if *identity == change.before_sha256 => SurvivingState::Preimage,
                    Ok(_) => SurvivingState::Conflict,
                    Err(_) => SurvivingState::Unavailable,
                };
                SurvivingFile {
                    path: change.path.clone(),
                    state,
                    sha256: current.ok().flatten(),
                }
            })
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct SurvivingFile {
    path: PathBuf,
    state: SurvivingState,
    sha256: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SurvivingState {
    Postimage,
    Preimage,
    Conflict,
    Unavailable,
}

impl Change {
    fn bytes(&self, custody: &Path) -> Result<Option<Vec<u8>>> {
        self.after_sha256
            .as_ref()
            .map(|identity| {
                ensure!(super::is_hex(identity, 64), "invalid staged file digest");
                reject_symlinks(custody, Path::new(identity))?;
                let bytes = snapshot::bounded_read(&custody.join(identity), MAX_GENERATED_FILE)?;
                ensure!(
                    hex(&Sha256::digest(&bytes)) == *identity,
                    "retained generated bytes changed"
                );
                Ok(bytes)
            })
            .transpose()
    }

    fn publish(&self, root: &Path, custody: &Path) -> Result<()> {
        let bytes = self.bytes(custody)?;
        let before = self
            .before_sha256
            .as_deref()
            .map(decode_identity)
            .transpose()?;
        let after = bytes.as_deref().map(|bytes| publication::StagedFile {
            identity: Sha256::digest(bytes).into(),
            bytes,
        });
        publication::Entry {
            relative: self.path.clone(),
            before,
            after,
        }
        .publish(root)
    }
}
