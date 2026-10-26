// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Advisory progress is bound to exact input; it never replaces physical checks.

use super::{Placement, StableFile, absolute, configuration, no_symlinks, sync_parent};
use crate::canonical::CanonicalError;
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs::OpenOptions, io::{BufRead, Seek, SeekFrom, Write},
    os::unix::fs::{OpenOptionsExt, MetadataExt}, path::{Path, PathBuf}, time::{Duration, Instant}};

/// Exact native request identity for an explicitly resumed frozen operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecoveryBinding {
    /// Native operation, never inferred from a prior result.
    pub operation: String,
    /// Original private source root.
    pub state: PathBuf,
    /// Explicit copy target, when selected.
    pub destination: Option<PathBuf>,
    /// Original physical placement, including an empty catalog.
    pub placement: Option<Placement>,
    /// Fresh restore placement, when selected.
    pub target: Option<Placement>,
    /// SHA-256 of the complete original input stream, including delimiters.
    pub inventory_sha256: String,
}

/// Last bounded native observation. A checkpoint is advisory, never a byte witness.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecoveryCheckpoint {
    /// Interpretation of this standalone document.
    pub version: Version<1>,
    /// Immutable operation and input identity.
    pub binding: RecoveryBinding,
    /// Attempt identity distinguishes explicit restarts from stale progress.
    pub attempt: String,
    /// Monotonically increasing observation within this attempt.
    pub sequence: u64,
    /// Current native physical phase.
    pub phase: RecoveryPhase,
    /// Fully checked and acknowledged instances in this attempt.
    pub completed_entries: u64,
    /// Cumulative physical bytes read or written in this attempt.
    pub processed_bytes: u64,
    /// Physical tree paths checked in this attempt.
    pub processed_paths: u64,
    /// Last native refusal, bounded before publication; absent during byte work.
    pub failure: Option<String>,
}

/// Native phase; a progress report never claims product admission or thaw.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RecoveryPhase {
    /// Exact original inventory has been bound before file effects.
    Bound,
    /// Acquiring and checking the next instance.
    Instance,
    /// Verifying original or already copied bytes.
    Fingerprint,
    /// Checking the independently declared canonical framed payload digest.
    ValidateDescriptor,
    /// Copying original bytes into an unpublished descriptor.
    Copy,
    /// Checking the complete unpublished destination bytes.
    VerifyCopy,
    /// Checking the independently published destination bytes.
    VerifyPublished,
    /// Checking complete trees for unknown paths and synchronization locks.
    ValidateTree,
    /// All original requests and complete tree checks settled.
    Complete,
}

/// Per-operation observer and durable binding; no global registry or implicit resume.
#[derive(Debug)]
pub struct RecoveryControl {
    path: Option<PathBuf>,
    checkpoint: Option<RecoveryCheckpoint>,
    resumed: bool,
    updated: Instant,
    lock: Option<StableFile>,
}
impl RecoveryControl {
    /// Select an explicit private checkpoint, or observe an in-process operation only.
    pub fn new(path: Option<PathBuf>) -> Self {
        Self { path, checkpoint: None, resumed: false, updated: Instant::now(), lock: None }
    }
    /// Authenticate the whole input before effects and retain exact resume authority.
    pub fn bind(&mut self, operation: &str, state: &Path, placement: Option<&Placement>,
        destination: Option<&Path>, target: Option<&Placement>, input: &mut (impl BufRead + Seek))
        -> Result<(), CanonicalError>
    {
        input.seek(SeekFrom::Start(0)).map_err(configuration)?;
        let mut digest = Sha256::new();
        let mut bytes = [0u8; 64 * 1024];
        loop {
            let length = input.read(&mut bytes).map_err(configuration)?;
            if length == 0 { break; }
            digest.update(&bytes[..length]);
        }
        input.seek(SeekFrom::Start(0)).map_err(configuration)?;
        let binding = RecoveryBinding { operation: operation.into(), state: state.into(),
            destination: destination.map(Path::to_path_buf), placement: placement.cloned(),
            target: target.cloned(), inventory_sha256: hex::encode(digest.finalize()) };
        if let Some(path) = &self.path {
            absolute(path)?;
            no_symlinks(path)?;
            let parent = path.parent().ok_or_else(|| configuration("checkpoint parent absent"))?;
            let directory = super::StableDirectory::open(parent)?;
            if directory.before.mode() & 0o077 != 0 {
                return Err(configuration("recovery checkpoint directory must be private"));
            }
            let lock_path = path.with_extension("lock");
            let file = OpenOptions::new().read(true).write(true).create(true).truncate(false)
                .mode(0o600).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
                .open(&lock_path).map_err(configuration)?;
            let before = file.metadata().map_err(configuration)?;
            super::private(&before, 1)?;
            file.try_lock().map_err(configuration)?;
            let lock = StableFile { file, path: lock_path, before };
            lock.unchanged()?;
            self.lock = Some(lock);
            if path.try_exists().map_err(configuration)? {
                let mut file = StableFile::open(path, false, 1)?;
                if file.before.len() > super::ENTRY_BYTES as u64 {
                    return Err(configuration("recovery checkpoint exceeds its bounded envelope"));
                }
                let previous: RecoveryCheckpoint = serde_json::from_reader(&mut file.file).map_err(configuration)?;
                file.unchanged()?;
                if previous.binding != binding {
                    return Err(configuration("recovery checkpoint differs from the exact frozen request"));
                }
                self.resumed = true;
            }
        }
        self.checkpoint = Some(RecoveryCheckpoint { version: Version, binding,
            attempt: uuid::Uuid::new_v4().to_string(), sequence: 0, phase: RecoveryPhase::Bound,
            completed_entries: 0, processed_bytes: 0, processed_paths: 0, failure: None });
        self.publish()
    }
    /// Exact matching request was retained, but every physical byte still needs checking.
    pub fn resumed(&self) -> bool { self.resumed }
    /// Retain an actionable native refusal without replacing its actual last phase.
    pub fn refused(&mut self, error: &CanonicalError) -> Result<(), CanonicalError> {
        if let Some(checkpoint) = &mut self.checkpoint {
            checkpoint.failure = Some(error.to_string().chars().take(4096).collect());
        }
        self.publish()
    }
    /// Publish a durable phase boundary.
    pub fn phase(&mut self, phase: RecoveryPhase) -> Result<(), CanonicalError> {
        if let Some(checkpoint) = &mut self.checkpoint { checkpoint.phase = phase; }
        self.publish()
    }
    /// Advance a completed instance without skipping it on any later attempt.
    pub fn acknowledged(&mut self) -> Result<(), CanonicalError> {
        if let Some(checkpoint) = &mut self.checkpoint {
            checkpoint.completed_entries = checkpoint.completed_entries.checked_add(1)
                .ok_or_else(|| configuration("recovery entry counter overflow"))?;
        }
        self.publish()
    }
    /// Report byte work at bounded intervals, including within very large files.
    pub fn bytes(&mut self, length: usize) -> Result<(), CanonicalError> {
        if let Some(checkpoint) = &mut self.checkpoint {
            checkpoint.processed_bytes = checkpoint.processed_bytes.checked_add(length as u64)
                .ok_or_else(|| configuration("recovery byte counter overflow"))?;
        }
        if self.updated.elapsed() >= Duration::from_millis(250) { self.publish()?; }
        Ok(())
    }
    /// Record an actually inspected path during complete-tree validation.
    pub fn path(&mut self) -> Result<(), CanonicalError> {
        if let Some(checkpoint) = &mut self.checkpoint {
            checkpoint.processed_paths = checkpoint.processed_paths.checked_add(1)
                .ok_or_else(|| configuration("recovery path counter overflow"))?;
        }
        self.bytes(0)
    }
    fn publish(&mut self) -> Result<(), CanonicalError> {
        if let Some(lock) = &self.lock { lock.unchanged()?; }
        let Some(checkpoint) = &mut self.checkpoint else { return Ok(()); };
        checkpoint.sequence = checkpoint.sequence.checked_add(1)
            .ok_or_else(|| configuration("recovery progress counter overflow"))?;
        let Some(path) = &self.path else { return Ok(()); };
        let parent = path.parent().ok_or_else(|| configuration("checkpoint parent absent"))?;
        let directory = super::StableDirectory::open(parent)?;
        if directory.before.mode() & 0o077 != 0 {
            return Err(configuration("recovery checkpoint directory must be private"));
        }
        let temporary = parent.join(format!(".checkpoint-{}", uuid::Uuid::new_v4()));
        let mut file = OpenOptions::new().write(true).create_new(true).mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(&temporary).map_err(configuration)?;
        let bytes = serde_json::to_vec(checkpoint).map_err(configuration)?;
        if bytes.len() > super::ENTRY_BYTES { return Err(configuration("checkpoint exceeds bounded envelope")); }
        file.write_all(&bytes).and_then(|()| file.sync_all()).map_err(configuration)?;
        directory.unchanged()?;
        std::fs::rename(&temporary, path).map_err(configuration)?;
        sync_parent(path)?;
        directory.unchanged()?;
        self.updated = Instant::now();
        Ok(())
    }
}
