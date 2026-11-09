// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One bounded last observation by the original workflow helper; never a verdict.
use super::Action;
use crate::{HostError, envelope::CliInvocationId, invocation::InvocationDirectory};
use rustix::fs::{AtFlags, Mode, OFlags, openat, renameat, unlinkat};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

const NAME: &str = "live-progress.json";
const STAGING: &str = "live-progress.pending";
const MAX_BYTES: usize = 16 * 1024;
const MAX_GROUPS: usize = 64;
const MAX_GROUP_NAME_BYTES: usize = 128;

/// The last workflow phase entered by the original helper.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LivePhase {
    /// The helper entered original Cargo workspace metadata capture.
    ReadingMetadata,
    /// The helper is resolving the captured workspace's requested selection.
    ResolvingSelection,
    /// Native discovery has been entered; build admission provides finer facts.
    DiscoveringTests,
    /// The helper is resolving the original fixture input requirements.
    ResolvingFixtureInputs,
    /// The helper has entered fixture input assessment and preparation.
    PreparingFixtureInputs,
    /// The helper is resolving original native group policy.
    ResolvingGroupPolicy,
    /// The helper entered compiler or native runner groups.
    ExecutingGroups,
    /// The helper handed original observations to the report publisher.
    PublishingReport,
}

/// The last observation for a compiler context. A returned runner is not settlement.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub(crate) enum LiveGroup {
    /// The helper actually awaited the named host predecessor.
    WaitingForPredecessor {
        /// Exact named group shared with the predecessor.
        native_group: String,
        /// Original compiler context holding the predecessor.
        predecessor: usize,
    },
    /// The helper entered its captured command phase. No runner slot is inferred.
    EnteredCommand,
    /// The captured command phase returned to the helper.
    CommandReturned,
}

/// Exact invocation-bound last observation, independent of the original terminal.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LiveProgress {
    invocation: CliInvocationId,
    action: Action,
    updated_unix_ms: u64,
    phase: LivePhase,
    groups: BTreeMap<usize, LiveGroup>,
    omitted_groups: bool,
}

impl LiveProgress {
    pub(crate) fn entered_groups(&self) -> impl Iterator<Item = usize> + '_ {
        self.groups.iter().filter_map(|(&group, progress)| {
            matches!(progress, LiveGroup::EnteredCommand | LiveGroup::CommandReturned).then_some(group)
        })
    }
}

fn invalid(message: &'static str) -> HostError {
    HostError::Ownership(message.into())
}

fn open_workflow(path: &Path) -> Result<File, HostError> {
    let directory = OpenOptions::new()
        .read(true)
        .custom_flags((OFlags::DIRECTORY | OFlags::NOFOLLOW).bits() as i32)
        .open(path)?;
    let metadata = directory.metadata()?;
    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o777 != 0o700 {
        return Err(invalid("live progress directory is not private original custody"));
    }
    Ok(directory)
}

fn read_snapshot(
    directory: &File,
    invocation: &CliInvocationId,
    action: Action,
) -> Result<Option<LiveProgress>, HostError> {
    let fd = match openat(
        directory,
        NAME,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    ) {
        Ok(fd) => fd,
        Err(rustix::io::Errno::NOENT) => return Ok(None),
        Err(error) => return Err(io::Error::from(error).into()),
    };
    let file = File::from(fd);
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
        || metadata.nlink() != 1
        || metadata.len() > MAX_BYTES as u64
    {
        return Err(invalid("live progress is not a bounded private original file"));
    }
    let mut bytes = Vec::new();
    file.take((MAX_BYTES + 1) as u64).read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BYTES {
        return Err(invalid("live progress exceeded its read allowance"));
    }
    let snapshot: LiveProgress = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    if &snapshot.invocation != invocation || snapshot.action != action || snapshot.groups.len() > MAX_GROUPS {
        return Err(invalid("live progress does not match the original workflow"));
    }
    Ok(Some(snapshot))
}

/// Observe the original owner's current snapshot without preparing or starting anything.
pub(crate) fn read(directory: &InvocationDirectory, action: Action) -> Result<Option<LiveProgress>, HostError> {
    let workflow = match open_workflow(&directory.path().join("workflow")) {
        Ok(workflow) => workflow,
        Err(HostError::Io(error)) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    read_snapshot(&workflow, directory.id(), action)
}

/// After committed report publication, release only this original optional snapshot.
pub(crate) fn clear(path: &Path, invocation: &CliInvocationId, action: Action) -> Result<(), HostError> {
    let directory = open_workflow(path)?;
    if read_snapshot(&directory, invocation, action)?.is_some() {
        match unlinkat(&directory, NAME, AtFlags::empty()) {
            Ok(()) | Err(rustix::io::Errno::NOENT) => {}
            Err(error) => return Err(io::Error::from(error).into()),
        }
    }
    Ok(())
}

pub(super) struct Writer {
    directory: File,
    snapshot: Mutex<LiveProgress>,
}

impl Writer {
    pub(super) fn new(path: &Path, invocation: CliInvocationId, action: Action) -> Result<Self, HostError> {
        Ok(Self {
            directory: open_workflow(path)?,
            snapshot: Mutex::new(LiveProgress {
                invocation,
                action,
                updated_unix_ms: 0,
                phase: LivePhase::ReadingMetadata,
                groups: BTreeMap::new(),
                omitted_groups: false,
            }),
        })
    }

    pub(super) fn phase(&self, phase: LivePhase) {
        self.update(|snapshot| snapshot.phase = phase);
    }

    pub(super) fn group(&self, id: usize, group: LiveGroup) {
        self.update(|snapshot| {
            if matches!(&group, LiveGroup::WaitingForPredecessor { native_group, .. }
                if native_group.len() > MAX_GROUP_NAME_BYTES
                    || native_group.chars().any(|character| character.is_control() || matches!(character, '"' | '\\')))
                || (!snapshot.groups.contains_key(&id) && snapshot.groups.len() == MAX_GROUPS)
            {
                snapshot.omitted_groups = true;
                return;
            }
            snapshot.groups.insert(id, group);
        });
    }

    fn update(&self, update: impl FnOnce(&mut LiveProgress)) {
        let Ok(mut snapshot) = self.snapshot.lock() else { return; };
        let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else { return; };
        let Ok(now) = u64::try_from(now.as_millis()) else { return; };
        snapshot.updated_unix_ms = now;
        update(&mut snapshot);
        let Ok(bytes) = serde_json::to_vec(&*snapshot) else { return; };
        if bytes.len() > MAX_BYTES { return; }
        let Ok(fd) = openat(
            &self.directory,
            STAGING,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::RUSR | Mode::WUSR,
        ) else { return; };
        if File::from(fd).write_all(&bytes).is_err()
            || renameat(&self.directory, STAGING, &self.directory, NAME).is_err()
        {
            let _ = unlinkat(&self.directory, STAGING, AtFlags::empty());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt};

    #[test]
    fn live_progress_keeps_last_state_and_refuses_foreign_cleanup() {
        let directory = tempfile::tempdir().unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let invocation = CliInvocationId::new();
        let writer = Writer::new(directory.path(), invocation.clone(), Action::Test).unwrap();
        writer.phase(LivePhase::ExecutingGroups);
        writer.group(1, LiveGroup::WaitingForPredecessor { native_group: "native".into(), predecessor: 0 });
        let before = read_snapshot(&writer.directory, &invocation, Action::Test).unwrap().unwrap();
        assert!(matches!(before.groups[&1], LiveGroup::WaitingForPredecessor { predecessor: 0, .. }));
        writer.group(1, LiveGroup::EnteredCommand);
        for group in 2..=MAX_GROUPS + 1 {
            writer.group(group, LiveGroup::CommandReturned);
        }
        let after = read_snapshot(&writer.directory, &invocation, Action::Test).unwrap().unwrap();
        assert!(matches!(after.groups[&1], LiveGroup::EnteredCommand));
        assert_eq!(after.groups.len(), MAX_GROUPS);
        assert!(after.omitted_groups);
        assert!(clear(directory.path(), &CliInvocationId::new(), Action::Test).is_err());
        clear(directory.path(), &invocation, Action::Test).unwrap();
        assert!(read_snapshot(&writer.directory, &invocation, Action::Test).unwrap().is_none());
        assert!(!directory.path().join(STAGING).exists());
    }
}
