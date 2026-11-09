// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read only the original opt-in Cargo timing report under the build permit.

use crate::extraction::CargoTimingMarker;
use rustix::fs::{Mode, OFlags, openat};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Read},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
    time::SystemTime,
};

const MAX_TIMING_BYTES: usize = 8 * 1024 * 1024;

/// How much of the exact Cargo timing report the workflow retained.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CargoTimingStatus {
    /// The workflow owns a bounded original copy, available by its evidence member.
    Captured,
    /// No checkout writer permit protects the mutable Cargo output domain.
    NoOwnedPermit,
    /// The structured first Cargo build record did not supply a supported run ID.
    MissingRunId,
    /// The exact report predates this build or changed while being read.
    StaleOrChanged,
    /// The report cannot be copied within the cumulative per-invocation bound.
    ExtentExceeded,
    /// The exact report or its private publication was unavailable.
    Unavailable,
}

/// Optional timing observation associated with one original compiler group.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct CargoTimingCapture {
    /// Original compiler context, independent of the producer's run ID.
    pub group: usize,
    /// The timestamp/workspace identity emitted by pinned Cargo.
    pub cargo_run_id: Option<String>,
    /// Workflow-relative member only when the original bytes were copied.
    pub file: Option<String>,
    /// An absent optional report does not change the performing verdict.
    pub observation: CargoTimingStatus,
}

#[derive(Default)]
pub(super) struct Pending {
    captures: Vec<(CargoTimingCapture, Option<Vec<u8>>)>,
    used: usize,
}

impl Pending {
    pub(super) fn observe(
        &mut self,
        group: usize,
        has_permit: bool,
        started: SystemTime,
        adapter: &Path,
        metadata: &cargo_metadata::Metadata,
    ) {
        let mut capture = CargoTimingCapture {
            group,
            cargo_run_id: None,
            file: None,
            observation: CargoTimingStatus::NoOwnedPermit,
        };
        let mut original = None;
        if has_permit {
            match run_id(&adapter.join("cargo-artifacts.jsonl")) {
                Ok(Some(id)) => {
                    // Cargo 3d7cf6e93 src/compiler/timings/mod.rs uses the very same
                    // logger RunId for this filename; the unqualified alias is mutable.
                    let target = metadata.target_directory.as_std_path();
                    let name = format!("cargo-timing-{id}.html");
                    capture.cargo_run_id = Some(id);
                    let observed = if target.is_absolute() {
                        timing_directory(target).and_then(|directory| {
                            read_report(&directory, Path::new(&name), started, MAX_TIMING_BYTES - self.used)
                        })
                    } else {
                        Err(CargoTimingStatus::Unavailable)
                    };
                    match observed {
                        Ok(bytes) => {
                            self.used += bytes.len();
                            capture.observation = CargoTimingStatus::Captured;
                            original = Some(bytes);
                        }
                        Err(reason) => capture.observation = reason,
                    }
                }
                Ok(None) => capture.observation = CargoTimingStatus::MissingRunId,
                Err(_) => capture.observation = CargoTimingStatus::Unavailable,
            }
        }
        self.captures.push((capture, original));
    }

    pub(super) fn publish(self, inventory: &Path) -> Vec<CargoTimingCapture> {
        self.captures.into_iter().map(|(mut capture, bytes)| {
            if let Some(bytes) = bytes {
                let name = format!("group-{}/cargo-timing.html", capture.group);
                let path = inventory.join(&name);
                if crate::invocation::publish_bytes(&path, &bytes, Some(MAX_TIMING_BYTES), 0o400).is_ok() {
                    capture.file = Some(format!("inventory/{name}"));
                } else {
                    capture.observation = CargoTimingStatus::Unavailable;
                }
            }
            capture
        }).collect()
    }
}

fn run_id(stream: &Path) -> Result<Option<String>, crate::HostError> {
    let original = crate::test_inventory::open_private(stream)?;
    let mut line = Vec::new();
    BufReader::new(original).take(513).read_until(b'\n', &mut line)?;
    if line.len() > 512 || line.last() != Some(&b'\n') {
        return Ok(None);
    }
    Ok(match serde_json::from_slice(&line) {
        Ok(CargoTimingMarker::BuildStarted { run_id }) if valid_run_id(run_id.as_bytes()) => Some(run_id),
        _ => None,
    })
}

fn valid_run_id(id: &[u8]) -> bool {
    // Cargo 3d7cf6e93 src/util/logger.rs RunId::FORMAT + workspace hash.
    id.len() == 36
        && id[..8].iter().all(u8::is_ascii_digit)
        && id[8] == b'T'
        && id[9..18].iter().all(u8::is_ascii_digit)
        && id[18..20] == *b"Z-"
        && id[20..].iter().all(|byte| matches!(*byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn timing_directory(target: &Path) -> Result<File, CargoTimingStatus> {
    crate::test_inventory::checked_parent(target).map_err(|_| CargoTimingStatus::Unavailable)?;
    let root = OpenOptions::new()
        .read(true)
        .custom_flags((OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC).bits() as i32)
        .open(target).map_err(|_| CargoTimingStatus::Unavailable)?;
    let opened = root.metadata().map_err(|_| CargoTimingStatus::Unavailable)?;
    let named = fs::symlink_metadata(target).map_err(|_| CargoTimingStatus::Unavailable)?;
    let user = rustix::process::geteuid().as_raw();
    if opened.uid() != user || opened.mode() & 0o777 != 0o700
        || (opened.dev(), opened.ino()) != (named.dev(), named.ino())
    {
        return Err(CargoTimingStatus::StaleOrChanged);
    }
    let child = File::from(openat(
        &root,
        "cargo-timings",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    ).map_err(|_| CargoTimingStatus::Unavailable)?);
    if child.metadata().map_err(|_| CargoTimingStatus::Unavailable)?.uid() != user {
        return Err(CargoTimingStatus::Unavailable);
    }
    Ok(child)
}

fn read_report(
    directory: &File, name: &Path, started: SystemTime, budget: usize,
) -> Result<Vec<u8>, CargoTimingStatus> {
    // Cargo uses the process umask for this mutable original. The private,
    // descriptor-bound target root above is its write boundary; the retained copy is 0400.
    let open = || {
        let file = File::from(openat(
            directory, name, OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        ).map_err(|_| CargoTimingStatus::Unavailable)?);
        let metadata = file.metadata().map_err(|_| CargoTimingStatus::Unavailable)?;
        if !metadata.is_file() || metadata.uid() != rustix::process::geteuid().as_raw() {
            return Err(CargoTimingStatus::Unavailable);
        }
        Ok(file)
    };
    let mut file = open()?;
    let before = crate::artifacts::descriptor_identity(&file).map_err(|_| CargoTimingStatus::Unavailable)?;
    let modified = file.metadata().and_then(|metadata| metadata.modified())
        .map_err(|_: io::Error| CargoTimingStatus::Unavailable)?;
    if modified < started {
        return Err(CargoTimingStatus::StaleOrChanged);
    }
    if before.bytes > budget as u64 {
        return Err(CargoTimingStatus::ExtentExceeded);
    }
    let mut bytes = Vec::new();
    (&mut file).take(budget as u64 + 1).read_to_end(&mut bytes)
        .map_err(|_| CargoTimingStatus::Unavailable)?;
    if bytes.len() > budget {
        return Err(CargoTimingStatus::ExtentExceeded);
    }
    let fresh = open().map_err(|_| CargoTimingStatus::StaleOrChanged)?;
    if bytes.len() as u64 != before.bytes
        || crate::artifacts::descriptor_identity(&file).ok().as_ref() != Some(&before)
        || crate::artifacts::descriptor_identity(&fresh).ok().as_ref() != Some(&before)
    {
        return Err(CargoTimingStatus::StaleOrChanged);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt as _;

    #[test]
    fn original_timing_marker_cannot_address_alias_or_another_path() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("cargo-artifacts.jsonl");
        fs::write(&path, b"{\"reason\":\"build-started\",\"run_id\":\"20060724T012128000Z-b0fd440798ab3cfb\"}\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(run_id(&path).unwrap().as_deref(), Some("20060724T012128000Z-b0fd440798ab3cfb"));
        fs::write(&path, b"{\"reason\":\"build-started\",\"run_id\":\"../cargo-timing.html\"}\n").unwrap();
        assert!(run_id(&path).unwrap().is_none());
    }

    #[test]
    fn stale_or_oversized_timing_report_is_not_adopted() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("cargo-timing-stamped.html");
        fs::write(&path, b"original report").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let parent = File::open(directory.path()).unwrap();
        let name = Path::new("cargo-timing-stamped.html");
        let after = SystemTime::now() + std::time::Duration::from_secs(1);
        assert!(matches!(read_report(&parent, name, after, 64), Err(CargoTimingStatus::StaleOrChanged)));
        assert!(matches!(read_report(&parent, name, std::time::UNIX_EPOCH, 1), Err(CargoTimingStatus::ExtentExceeded)));
        assert_eq!(read_report(&parent, name, std::time::UNIX_EPOCH, 64).unwrap(), b"original report");
    }
}
