// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Admission for repository-owned writers of Cargo outputs. Cargo retains its own locks.

use crate::{HostError, envelope::CliInvocationId};
use pse_diagnostics::process_facts::{ProcessIdentity, ProcessRead, ProcessReader};
use rustix::fs::{FlockOperation, OFlags, flock};
use serde::{Deserialize, Serialize};
use std::{
    ffi::{OsStr, OsString},
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _},
    path::{Component, Path, PathBuf},
    time::Duration,
};

pub(crate) const CALLBACK: &str = "PSE_CARGO_BUILD_CALLBACK";
const MAX_QUEUE_BYTES: u64 = 1024 * 1024;

fn error(message: impl std::fmt::Display) -> HostError {
    HostError::Ownership(format!("Cargo build admission: {message}"))
}

/// Cargo reports absolute paths even before it creates the directories.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Outputs {
    target: PathBuf,
    build: PathBuf,
}

fn canonical_output(path: &Path) -> io::Result<PathBuf> {
    match path.canonicalize() {
        Ok(path) => Ok(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if fs::symlink_metadata(path).is_ok() {
                return Err(error);
            }
            let name = path.file_name().ok_or(error)?;
            let parent = path.parent().ok_or(io::ErrorKind::NotFound)?;
            Ok(canonical_output(parent)?.join(name))
        }
        Err(error) => Err(error),
    }
}

impl Outputs {
    pub(crate) fn new(target: &Path, build: &Path) -> Result<Self, HostError> {
        if !target.is_absolute() || !build.is_absolute() {
            return Err(error("Cargo did not report absolute output directories"));
        }
        Ok(Self {
            target: canonical_output(target)?,
            build: canonical_output(build)?,
        })
    }

    pub(crate) fn ordinary(root: &Path) -> Result<Self, HostError> {
        let target = root.join("target");
        Self::new(&target, &target)
    }

    fn paths(&self) -> [&Path; 2] {
        [&self.target, &self.build]
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.paths().into_iter().any(|left| {
            other.paths().into_iter().any(|right| {
                left.starts_with(right) || right.starts_with(left) || same_directory(left, right)
            })
        })
    }
}

fn same_directory(left: &Path, right: &Path) -> bool {
    directory_identity(left)
        .zip(directory_identity(right))
        .is_some_and(|(left, right)| left == right)
}

fn directory_identity(path: &Path) -> Option<(u64, u64)> {
    let metadata = fs::metadata(path).ok()?;
    metadata.is_dir().then(|| (metadata.dev(), metadata.ino()))
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Stage {
    Waiting,
    Building,
    Capturing,
    Listing,
    Running,
    Settling,
}

#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
pub(crate) struct Activity {
    pub(crate) invocation: Option<CliInvocationId>,
    stage: Stage,
    position: Option<usize>,
    blocker: Option<CliInvocationId>,
    cancellation_requested: bool,
}

#[derive(Deserialize, Serialize)]
struct Scope {
    path: PathBuf,
    inode: u64,
    device: u64,
}

impl Scope {
    fn capture() -> Result<Self, HostError> {
        let text = fs::read_to_string("/proc/self/cgroup")?;
        let mut entries = text.lines().filter_map(|line| line.strip_prefix("0::/"));
        let group = entries
            .next()
            .ok_or_else(|| error("native writer group is unavailable"))?;
        let relative = Path::new(group);
        if group.is_empty()
            || entries.next().is_some()
            || relative
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err(error("native writer group is ambiguous"));
        }
        let path = Path::new("/sys/fs/cgroup").join(relative);
        let metadata = fs::metadata(&path)?;
        Ok(Self {
            path,
            inode: metadata.ino(),
            device: metadata.dev(),
        })
    }

    fn drained(&self) -> bool {
        match fs::metadata(&self.path) {
            Ok(metadata) if metadata.ino() == self.inode && metadata.dev() == self.device => {
                fs::read_to_string(self.path.join("cgroup.events"))
                    .is_ok_and(|contents| contents.lines().any(|line| line == "populated 0"))
            }
            Ok(_) => true,
            Err(error) => error.kind() == io::ErrorKind::NotFound,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Request {
    token: uuid::Uuid,
    invocation: Option<CliInvocationId>,
    process: ProcessIdentity,
    scope: Option<Scope>,
    outputs: Outputs,
    stage: Stage,
    cancelled: bool,
    #[serde(default)]
    released: bool,
}

impl Request {
    fn abandoned(&self, observer: &ProcessIdentity) -> bool {
        if self.process.boot_id != observer.boot_id {
            return true;
        }
        if self.process.pid_namespace != observer.pid_namespace {
            return false;
        }
        let gone = match ProcessReader::default().process(self.process.pid) {
            ProcessRead::Present(facts) => {
                facts.identity != self.process || facts.stat.state == 'Z'
            }
            ProcessRead::Absent => true,
            ProcessRead::Changed | ProcessRead::Unavailable(_) => false,
        };
        gone && (self.released
            || self.stage == Stage::Waiting
            || self.scope.as_ref().is_none_or(Scope::drained))
    }

    fn conflicts(&self, outputs: &Outputs) -> bool {
        !self.cancelled && !self.released && self.outputs.overlaps(outputs)
    }
}

fn current_process() -> Result<ProcessIdentity, HostError> {
    match ProcessReader::default().process(std::process::id()) {
        ProcessRead::Present(facts) => Ok(facts.identity),
        _ => Err(error("current process identity is unavailable")),
    }
}

fn owned_directory(path: &Path, create: bool, write_mask: u32) -> io::Result<()> {
    if create {
        match fs::DirBuilder::new().mode(0o700).create(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & write_mask != 0
    {
        return Err(io::Error::other(
            "Cargo coordination directory is not owned and private",
        ));
    }
    Ok(())
}

fn open_lock(path: &Path, create: bool) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(create)
        .mode(0o600)
        .custom_flags((OFlags::NOFOLLOW | OFlags::NONBLOCK).bits() as i32)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o022 != 0
    {
        return Err(io::Error::other(
            "Cargo coordination lock is not an owned regular file",
        ));
    }
    Ok(file)
}

struct Queue {
    directory: PathBuf,
    _lock: File,
    requests: Vec<Request>,
}

impl Queue {
    fn open(root: &Path, create: bool) -> Result<Self, HostError> {
        let parent = root.join("build");
        owned_directory(&parent, create, 0o002)?;
        let directory = parent.join(".pse-cargo");
        owned_directory(&directory, create, 0o077)?;
        let lock = open_lock(&directory.join("queue.lock"), create)?;
        flock(
            &lock,
            if create {
                FlockOperation::LockExclusive
            } else {
                FlockOperation::LockShared
            },
        )
        .map_err(io::Error::from)?;
        let requests = match open_lock(&directory.join("queue.json"), false) {
            Ok(file) => {
                let mut bytes = Vec::new();
                file.take(MAX_QUEUE_BYTES + 1).read_to_end(&mut bytes)?;
                if bytes.len() as u64 > MAX_QUEUE_BYTES {
                    return Err(error("live queue exceeds its observation bound"));
                }
                serde_json::from_slice(&bytes).map_err(error)?
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(error.into()),
        };
        Ok(Self {
            directory,
            _lock: lock,
            requests,
        })
    }

    fn publish(&self) -> Result<(), HostError> {
        let bytes = serde_json::to_vec(&self.requests).map_err(error)?;
        if bytes.len() as u64 > MAX_QUEUE_BYTES {
            return Err(error("live queue exceeds its storage bound"));
        }
        let mut file = tempfile::NamedTempFile::new_in(&self.directory)?;
        file.write_all(&bytes)?;
        file.persist(self.directory.join("queue.json"))
            .map_err(|failure| failure.error)?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct Permit {
    root: PathBuf,
    token: uuid::Uuid,
    default_writer: Option<File>,
    retain_for_drain: bool,
    pub(crate) waited: bool,
}

impl Permit {
    /// `None` means the original waiting request was cancelled before any writer started.
    pub(crate) fn acquire(
        root: &Path,
        outputs: Outputs,
        invocation: Option<CliInvocationId>,
    ) -> Result<Option<Self>, HostError> {
        let coordination = root.join("build/.pse-cargo");
        if outputs
            .paths()
            .iter()
            .any(|path| coordination.starts_with(path) || path.starts_with(&coordination))
        {
            return Err(error(
                "Cargo outputs overlap the build coordination directory",
            ));
        }
        let process = current_process()?;
        let scope = invocation.as_ref().map(|_| Scope::capture()).transpose()?;
        let ordinary = outputs.overlaps(&Outputs::ordinary(root)?);
        let identities = outputs.paths().map(directory_identity);
        let mut permit = Self {
            root: root.to_owned(),
            token: uuid::Uuid::now_v7(),
            default_writer: None,
            retain_for_drain: false,
            waited: false,
        };
        {
            let mut queue = Queue::open(root, true)?;
            queue
                .requests
                .retain(|request| !request.abandoned(&process));
            queue.requests.push(Request {
                token: permit.token,
                invocation: invocation.clone(),
                process: process.clone(),
                scope,
                outputs: outputs.clone(),
                stage: Stage::Waiting,
                cancelled: false,
                released: false,
            });
            queue.publish()?;
        }
        let mut announced = false;
        loop {
            {
                let mut queue = Queue::open(root, true)?;
                let mut index = queue
                    .requests
                    .iter()
                    .position(|request| request.token == permit.token)
                    .ok_or_else(|| error("original waiting request is unavailable"))?;
                if queue.requests[index].cancelled {
                    return Ok(None);
                }
                let mut changed = false;
                let blocked = loop {
                    let first = queue.requests[..index]
                        .iter()
                        .position(|request| request.conflicts(&outputs));
                    let Some(first) = first else { break false };
                    if !queue.requests[first].abandoned(&process) {
                        break true;
                    }
                    queue.requests.remove(first);
                    index -= 1;
                    changed = true;
                };
                if !blocked && (!ordinary || permit.lock_default(&queue.directory)?) {
                    if Outputs::new(&outputs.target, &outputs.build)? != outputs
                        || identities
                            .into_iter()
                            .zip(outputs.paths())
                            .any(|(expected, path)| {
                                expected.is_some() && expected != directory_identity(path)
                            })
                    {
                        return Err(error("Cargo output directories changed while queued"));
                    }
                    queue.requests[index].stage = Stage::Building;
                    queue.publish()?;
                    return Ok(Some(permit));
                }
                if changed {
                    queue.publish()?;
                }
            }
            if !announced {
                let id = invocation
                    .as_ref()
                    .map_or("standalone command", CliInvocationId::as_str);
                writeln!(
                    io::stderr(),
                    "pse: waiting for Cargo build admission ({id})"
                )?;
                announced = true;
                permit.waited = true;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    fn lock_default(&mut self, directory: &Path) -> Result<bool, HostError> {
        let file = open_lock(&directory.join("default-writer.lock"), true)?;
        match flock(&file, FlockOperation::NonBlockingLockExclusive) {
            Ok(()) => {
                self.default_writer = Some(file);
                Ok(true)
            }
            Err(rustix::io::Errno::WOULDBLOCK) => Ok(false),
            Err(failure) => Err(io::Error::from(failure).into()),
        }
    }

    pub(crate) fn callback(&self) -> OsString {
        self.token.to_string().into()
    }

    pub(crate) fn retain_for_drain(&mut self) {
        self.retain_for_drain = true;
    }

    pub(crate) fn release_to_runner(&mut self) -> Result<(), HostError> {
        let mut queue = Queue::open(&self.root, true)?;
        let request = queue
            .requests
            .iter_mut()
            .find(|request| request.token == self.token)
            .ok_or_else(|| error("original build permit is unavailable"))?;
        request.stage = Stage::Running;
        request.released = true;
        queue.publish()?;
        drop(self.default_writer.take());
        Ok(())
    }

    pub(crate) fn stage(&self, stage: Stage) -> Result<(), HostError> {
        let mut queue = Queue::open(&self.root, true)?;
        let request = queue
            .requests
            .iter_mut()
            .find(|request| request.token == self.token)
            .ok_or_else(|| error("original build permit is unavailable"))?;
        request.stage = stage;
        queue.publish()
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        if self.retain_for_drain {
            if let Err(failure) = self.stage(Stage::Settling) {
                let _ = writeln!(io::stderr(), "pse: {failure}");
            }
            return;
        }
        let released = (|| -> Result<(), HostError> {
            let mut queue = Queue::open(&self.root, true)?;
            queue.requests.retain(|request| request.token != self.token);
            queue.publish()
        })();
        if let Err(failure) = released {
            let _ = writeln!(io::stderr(), "pse: {failure}");
        }
    }
}

pub(crate) fn authorize_callback(
    root: &Path,
    outputs: &Outputs,
    token: &OsStr,
) -> Result<(), HostError> {
    let token = token
        .to_str()
        .and_then(|value| uuid::Uuid::parse_str(value).ok())
        .ok_or_else(|| error("invalid build callback identity"))?;
    let queue = Queue::open(root, false)?;
    let request = queue
        .requests
        .iter()
        .find(|request| request.token == token && request.stage == Stage::Building)
        .filter(|request| &request.outputs == outputs && request.invocation.is_some())
        .ok_or_else(|| error("build callback differs from its admitted writer"))?;
    let reader = ProcessReader::default();
    let mut pid = std::process::id();
    for _ in 0..64 {
        let ProcessRead::Present(facts) = reader.process(pid) else {
            break;
        };
        if facts.identity == request.process {
            return Ok(());
        }
        if facts.stat.parent_pid == 0 || facts.stat.parent_pid == pid {
            break;
        }
        pid = facts.stat.parent_pid;
    }
    Err(error(
        "build callback is outside the original writer's process ancestry",
    ))
}

pub(crate) fn activity(root: &Path) -> Result<Vec<Activity>, HostError> {
    let queue = match Queue::open(root, false) {
        Ok(queue) => queue,
        Err(HostError::Io(failure)) if failure.kind() == io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(failure) => return Err(failure),
    };
    let observer = current_process()?;
    let requests: Vec<_> = queue
        .requests
        .iter()
        .filter(|request| !request.abandoned(&observer))
        .collect();
    Ok(requests
        .iter()
        .enumerate()
        .map(|(index, request)| {
            let preceding = || {
                requests[..index]
                    .iter()
                    .filter(|earlier| earlier.conflicts(&request.outputs))
            };
            Activity {
                invocation: request.invocation.clone(),
                stage: request.stage,
                position: (request.stage == Stage::Waiting).then(|| preceding().count() + 1),
                blocker: preceding()
                    .next()
                    .and_then(|earlier| earlier.invocation.clone()),
                cancellation_requested: request.cancelled,
            }
        })
        .collect())
}

pub(crate) fn cancel_waiter(root: &Path, invocation: &CliInvocationId) -> Result<bool, HostError> {
    if !root.join("build/.pse-cargo/queue.lock").is_file() {
        return Ok(false);
    }
    let mut queue = Queue::open(root, true)?;
    let Some(request) = queue.requests.iter_mut().find(|request| {
        request.invocation.as_ref() == Some(invocation) && request.stage == Stage::Waiting
    }) else {
        return Ok(false);
    };
    request.cancelled = true;
    queue.publish()?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{os::unix::fs::symlink, sync::mpsc, time::Instant};

    fn await_waiter(root: &Path, id: &CliInvocationId) -> Activity {
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(row) = activity(root)
                .unwrap()
                .into_iter()
                .find(|row| row.invocation.as_ref() == Some(id) && row.stage == Stage::Waiting)
            {
                return row;
            }
            assert!(Instant::now() < end, "waiting request was never observed");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn overlapping_intermediates_queue_and_cancellation_leaves_owner_intact() {
        let root = tempfile::tempdir().unwrap();
        let shared = root.path().join("intermediates");
        let first = Outputs::new(&root.path().join("artifact-a"), &shared).unwrap();
        let second = Outputs::new(&root.path().join("artifact-b"), &shared).unwrap();
        let separate = Outputs::new(
            &root.path().join("artifact-c"),
            &root.path().join("intermediates-c"),
        )
        .unwrap();
        let owner = CliInvocationId::new();
        let waiter = CliInvocationId::new();
        let (sender, receiver) = mpsc::channel();
        let owner_permit = Permit::acquire(root.path(), first, Some(owner.clone()))
            .unwrap()
            .unwrap();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                sender
                    .send(Permit::acquire(root.path(), second, Some(waiter.clone())).unwrap())
                    .unwrap();
            });
            let queued = await_waiter(root.path(), &waiter);
            assert_eq!(queued.position, Some(2));
            assert_eq!(queued.blocker, Some(owner.clone()));
            let independent = Permit::acquire(root.path(), separate, None)
                .unwrap()
                .unwrap();
            assert!(cancel_waiter(root.path(), &waiter).unwrap());
            assert!(
                receiver
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap()
                    .is_none()
            );
            assert!(!cancel_waiter(root.path(), &owner).unwrap());
            assert!(
                activity(root.path())
                    .unwrap()
                    .iter()
                    .any(|row| row.invocation.as_ref() == Some(&owner))
            );
            drop(independent);
        });
        drop(owner_permit);
        assert!(activity(root.path()).unwrap().is_empty());
    }

    #[test]
    fn same_physical_directory_conflicts_across_lexical_aliases() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target");
        let alias = root.path().join("alias");
        fs::create_dir(&target).unwrap();
        symlink(&target, &alias).unwrap();
        let first = Outputs::new(&target, &target).unwrap();
        let second = Outputs::new(&alias.join("missing"), &alias).unwrap();
        assert!(first.overlaps(&second));
        assert_eq!(second.build, target.canonicalize().unwrap());
    }

    #[test]
    fn runner_remains_visible_without_blocking_a_later_writer() {
        let root = tempfile::tempdir().unwrap();
        let outputs = Outputs::ordinary(root.path()).unwrap();
        let id = CliInvocationId::new();
        let mut permit = Permit::acquire(root.path(), outputs.clone(), Some(id.clone()))
            .unwrap()
            .unwrap();
        permit.release_to_runner().unwrap();
        let writer = Permit::acquire(root.path(), outputs, None)
            .unwrap()
            .unwrap();
        assert!(!writer.waited);
        let live = activity(root.path()).unwrap();
        assert_eq!(live[0].invocation, Some(id));
        assert_eq!(live[0].stage, Stage::Running);
    }
}
