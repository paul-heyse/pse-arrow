// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Settlement of the original target-owned effects, separate from Nextest verdicts.

use crate::HostError;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read},
    os::unix::fs::{MetadataExt as _, OpenOptionsExt as _},
    path::{Path, PathBuf},
};
use uuid::Uuid;

/// An invocation-private directory allocated by the managed group owner.
pub const DIRECTORY_ENV: &str = "PSE_TEST_TARGET_SETTLEMENT_DIRECTORY";
const RECORD_BYTES: usize = 4096;
const MAX_RECORDS: usize = 4096;

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Effect {
    version: u8,
    run: Uuid,
    binary: String,
    test: String,
    attempt: usize,
    owner: Uuid,
}

/// A single physical service effect within the original Nextest attempt.
#[derive(Debug)]
pub struct TargetEffect {
    directory: PathBuf,
    effect: Effect,
}

impl TargetEffect {
    /// Publish intent before the owner dispatches the physical effect. Standalone
    /// or runner-borrowed owners without a Nextest attempt have no target record.
    pub fn begin() -> Result<Option<Self>, HostError> {
        let Some(directory) = std::env::var_os(DIRECTORY_ENV).map(PathBuf::from) else {
            return Ok(None);
        };
        let Some(run) = std::env::var_os("NEXTEST_RUN_ID") else {
            return Ok(None);
        };
        let Some(attempt) = std::env::var_os("NEXTEST_ATTEMPT") else {
            return Ok(None);
        };
        let binary = std::env::var("NEXTEST_BINARY_ID").map_err(|_| invalid_identity())?;
        let test = std::env::var("NEXTEST_TEST_NAME").map_err(|_| invalid_identity())?;
        let run = run
            .to_str()
            .and_then(|run| run.parse().ok())
            .ok_or_else(invalid_identity)?;
        let attempt = attempt
            .to_str()
            .and_then(|attempt| attempt.parse().ok())
            .filter(|&attempt| attempt != 0)
            .ok_or_else(invalid_identity)?;
        Self::begin_effect(
            directory,
            Effect {
                version: 1,
                run,
                binary,
                test,
                attempt,
                owner: Uuid::new_v4(),
            },
        )
        .map(Some)
    }

    fn begin_effect(directory: PathBuf, effect: Effect) -> Result<Self, HostError> {
        private_directory(&directory)?;
        let owner = Self { directory, effect };
        owner.publish("started")?;
        Ok(owner)
    }

    /// Only the original physical owner may call this after observing its drain.
    pub fn settle(&self) -> Result<(), HostError> {
        self.publish("settled")
    }

    fn publish(&self, stage: &str) -> Result<(), HostError> {
        let path = self
            .directory
            .join(format!("{}.{stage}", self.effect.owner));
        let bytes = serde_json::to_vec(&self.effect).map_err(|error| {
            HostError::Ownership(format!("encode target-owned settlement: {error}"))
        })?;
        crate::invocation::publish_bytes(&path, &bytes, Some(RECORD_BYTES), 0o400)
    }
}

fn invalid_identity() -> HostError {
    HostError::Ownership("invalid Nextest target-owned settlement identity".into())
}

fn private_directory(path: &Path) -> io::Result<()> {
    let metadata = path.symlink_metadata()?;
    if !path.is_absolute()
        || !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
    {
        return Err(io::Error::other(
            "target-owned settlement directory is not private",
        ));
    }
    Ok(())
}

fn read_effect(path: &Path) -> io::Result<Effect> {
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.nlink() != 1
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o7777 != 0o400
        || metadata.len() > RECORD_BYTES as u64
    {
        return Err(io::Error::other(
            "target-owned settlement record is not bounded and private",
        ));
    }
    let mut bytes = Vec::new();
    file.take(RECORD_BYTES as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > RECORD_BYTES {
        return Err(io::Error::other(
            "target-owned settlement record exceeds its bound",
        ));
    }
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}

pub(crate) fn settled(
    directory: &Path,
    run: &str,
    attempts: &BTreeSet<(String, String, usize)>,
) -> io::Result<bool> {
    private_directory(directory)?;
    let run: Uuid = run.parse().map_err(io::Error::other)?;
    let mut covered = BTreeSet::new();
    let mut starts = 0;
    let mut finishes = 0;
    for (index, entry) in fs::read_dir(directory)?.enumerate() {
        if index >= MAX_RECORDS {
            return Ok(false);
        }
        let path = entry?.path();
        let Some(stage) = path.extension().and_then(|stage| stage.to_str()) else {
            return Ok(false);
        };
        if stage == "settled" {
            finishes += 1;
            continue;
        }
        if stage != "started" {
            return Ok(false);
        }
        starts += 1;
        let started = read_effect(&path)?;
        if started.version != 1
            || started.run != run
            || path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|stem| stem.parse::<Uuid>().ok())
                != Some(started.owner)
            || read_effect(&path.with_extension("settled"))? != started
        {
            return Ok(false);
        }
        let attempt = (started.binary, started.test, started.attempt);
        if !attempts.contains(&attempt) {
            return Ok(false);
        }
        covered.insert(attempt);
    }
    Ok(starts == finishes && !attempts.is_empty() && &covered == attempts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt as _;

    #[test]
    fn owner_records_require_the_original_run_and_every_effect_of_every_attempt() {
        let root = tempfile::Builder::new()
            .permissions(fs::Permissions::from_mode(0o700))
            .tempdir()
            .unwrap();
        let run = Uuid::now_v7();
        let effect = |attempt| {
            TargetEffect::begin_effect(
                root.path().to_owned(),
                Effect {
                    version: 1,
                    run,
                    binary: "pkg::target".into(),
                    test: "selected::case".into(),
                    attempt,
                    owner: Uuid::new_v4(),
                },
            )
            .unwrap()
        };
        let selected = |attempt| ("pkg::target".into(), "selected::case".into(), attempt);
        let attempts = BTreeSet::from([selected(1), selected(2)]);
        let run_id = run.to_string();
        let check = || settled(root.path(), &run_id, &attempts).unwrap_or(false);

        assert!(!check(), "missing owner");
        effect(1).settle().unwrap();
        let second = effect(2);
        let sibling = effect(2);
        second.settle().unwrap();
        assert!(
            !check(),
            "one unsettled effect keeps its attempt in custody"
        );
        sibling.settle().unwrap();
        assert!(check());
        assert!(!settled(root.path(), &Uuid::now_v7().to_string(), &attempts).unwrap());
        assert!(!settled(root.path(), &run_id, &BTreeSet::from([selected(1)])).unwrap());
        effect(3).settle().unwrap();
        assert!(!check(), "foreign attempts never establish settlement");
    }
}
