// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete offline file effects. The supervisor owns authenticated application
//! freeze; this owner retains the actual lock and descriptors across byte work.

use super::{FileDescriptor, configuration, validate_file};
use crate::canonical::CanonicalError;
use pse_model::generated::enums::CanonicalFileState;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{File, Metadata, OpenOptions},
    io::{BufRead, Read, Seek, SeekFrom, Write},
    os::{fd::AsRawFd, unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt}},
    path::{Component, Path, PathBuf},
};

/// Maximum encoded entry, excluding any scientific bytes.
pub const ENTRY_BYTES: usize = 64 * 1024;

/// Physical placement, independent of scientific result interpretation.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Placement {
    /// Never-reused private file-tier incarnation.
    pub payload_incarnation: String,
    /// Selected database's physical component.
    pub database: String,
}

impl Placement {
    /// Derive the canonical private root without normalizing untrusted names.
    pub fn root(&self, state: &Path) -> Result<PathBuf, CanonicalError> {
        canonical_uuid(&self.payload_incarnation)?;
        if self.database.is_empty()
            || self.database.len() > 128
            || !self.database.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(configuration("invalid payload database placement"));
        }
        absolute(state)?;
        Ok(state.join("payloads").join(&self.payload_incarnation).join(&self.database))
    }
}

fn canonical_uuid(value: &str) -> Result<(), CanonicalError> {
    if uuid::Uuid::parse_str(value).map_err(configuration)?.to_string() != value {
        return Err(configuration("payload identity is not a canonical UUID"));
    }
    Ok(())
}
fn absolute(path: &Path) -> Result<(), CanonicalError> {
    if !path.is_absolute() || path.components().any(|part| matches!(part, Component::ParentDir)) {
        return Err(configuration("payload path must be an absolute derived path"));
    }
    Ok(())
}
fn no_symlinks(path: &Path) -> Result<(), CanonicalError> {
    absolute(path)?;
    for parent in path.ancestors() {
        match std::fs::symlink_metadata(parent) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(configuration("payload path must not traverse symlinks"));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(configuration(error)),
        }
    }
    Ok(())
}
fn identity(metadata: &Metadata) -> (u64, u64, u32, u32, u64, u64, i64, i64, i64, i64) {
    (metadata.dev(), metadata.ino(), metadata.mode(), metadata.uid(), metadata.nlink(), metadata.len(),
     metadata.mtime(), metadata.mtime_nsec(), metadata.ctime(), metadata.ctime_nsec())
}
fn private(metadata: &Metadata, links: u64) -> Result<(), CanonicalError> {
    if !metadata.is_file() || metadata.uid() != rustix::process::getuid().as_raw()
        || metadata.mode() & 0o7777 != 0o600 || metadata.nlink() != links
    {
        return Err(configuration("payload lost its exact private regular-file identity"));
    }
    Ok(())
}

struct StableFile {
    file: File,
    path: PathBuf,
    before: Metadata,
}
impl StableFile {
    fn open(path: &Path, write: bool, links: u64) -> Result<Self, CanonicalError> {
        no_symlinks(path)?;
        let file = OpenOptions::new().read(true).write(write)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(path).map_err(configuration)?;
        let before = file.metadata().map_err(configuration)?;
        private(&before, links)?;
        let result = Self { file, path: path.into(), before };
        result.unchanged()?;
        Ok(result)
    }
    fn unchanged(&self) -> Result<(), CanonicalError> {
        no_symlinks(&self.path)?;
        let after = self.file.metadata().map_err(configuration)?;
        let visible = std::fs::symlink_metadata(&self.path).map_err(configuration)?;
        if identity(&self.before) != identity(&after) || identity(&after) != identity(&visible) {
            return Err(configuration("payload inode or extent changed during offline operation"));
        }
        Ok(())
    }
    fn fingerprint(&mut self) -> Result<Fingerprint, CanonicalError> {
        let fingerprint = fingerprint_file(&mut self.file)?;
        self.unchanged()?;
        Ok(fingerprint)
    }

}

fn fingerprint_file(file: &mut File) -> Result<Fingerprint, CanonicalError> {
    let before = file.metadata().map_err(configuration)?;
    file.seek(SeekFrom::Start(0)).map_err(configuration)?;
    let mut digest = Sha256::new();
    let mut block = [0u8; 1024 * 1024];
    loop {
        let length = file.read(&mut block).map_err(configuration)?;
        if length == 0 { break; }
        digest.update(&block[..length]);
    }
    if identity(&before) != identity(&file.metadata().map_err(configuration)?) {
        return Err(configuration("payload descriptor changed during fingerprint"));
    }
    file.seek(SeekFrom::Start(0)).map_err(configuration)?;
    Ok(Fingerprint { sha256: hex::encode(digest.finalize()), bytes: before.len() })
}

struct StableDirectory {
    file: File,
    path: PathBuf,
    before: Metadata,
}
impl StableDirectory {
    fn open(path: &Path) -> Result<Self, CanonicalError> {
        no_symlinks(path)?;
        let file = OpenOptions::new().read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path).map_err(configuration)?;
        let before = file.metadata().map_err(configuration)?;
        if !before.is_dir() || before.uid() != rustix::process::getuid().as_raw() || before.mode() & 0o022 != 0 {
            return Err(configuration("offline copy directory lost private ownership"));
        }
        let result = Self { file, path: path.into(), before };
        result.unchanged()?;
        Ok(result)
    }
    fn unchanged(&self) -> Result<(), CanonicalError> {
        no_symlinks(&self.path)?;
        let observed = self.file.metadata().map_err(configuration)?;
        let visible = std::fs::symlink_metadata(&self.path).map_err(configuration)?;
        let identity = |metadata: &Metadata| (metadata.dev(), metadata.ino(), metadata.mode(), metadata.uid());
        if identity(&self.before) != identity(&observed) || identity(&observed) != identity(&visible) {
            return Err(configuration("offline copy directory changed during publication"));
        }
        Ok(())
    }
}
fn anonymous_copy_file(directory: &File) -> Result<File, CanonicalError> {
    use rustix::fs::{openat, Mode, OFlags};
    let descriptor = openat(directory, ".", OFlags::RDWR | OFlags::TMPFILE | OFlags::CLOEXEC | OFlags::NOFOLLOW, Mode::RUSR | Mode::WUSR)
        .map_err(|error| configuration(format!("offline copy requires filesystem support for anonymous O_TMPFILE publication: {error}")))?;
    let file = File::from(descriptor);
    private(&file.metadata().map_err(configuration)?, 0)?;
    Ok(file)
}

struct ExistingLock(StableFile);
impl ExistingLock {
    fn acquire(root: &Path, key: &str) -> Result<Option<Self>, CanonicalError> {
        let path = root.join(".locks").join(format!("{key}.lock"));
        no_symlinks(&path)?;
        if !path.try_exists().map_err(configuration)? { return Ok(None); }
        let locked = StableFile::open(&path, true, 1)?;
        if locked.before.len() != 0 { return Err(configuration("instance lock is not empty")); }
        locked.file.try_lock().map_err(configuration)?;
        locked.unchanged()?;
        Ok(Some(Self(locked)))
    }
    fn unchanged(&self) -> Result<(), CanonicalError> { self.0.unchanged() }
}

fn restore_destination_lock(root: &Path, key: &str) -> Result<ExistingLock, CanonicalError> {
    let directory = root.join(".locks");
    no_symlinks(&directory)?;
    std::fs::create_dir_all(&directory).map_err(configuration)?;
    no_symlinks(&directory)?;
    let metadata = std::fs::symlink_metadata(&directory).map_err(configuration)?;
    if !metadata.is_dir() || metadata.uid() != rustix::process::getuid().as_raw() {
        return Err(configuration("restore lock directory lost private ownership"));
    }
    let path = directory.join(format!("{key}.lock"));
    match OpenOptions::new().read(true).write(true).create_new(true).mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(&path) {
        Ok(file) => {
            let before = file.metadata().map_err(configuration)?;
            private(&before, 1)?;
            file.try_lock().map_err(configuration)?;
            file.sync_all().map_err(configuration)?;
            sync_parent(&path)?;
            let locked = StableFile { file, path, before };
            locked.unchanged()?;
            Ok(ExistingLock(locked))
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            ExistingLock::acquire(root, key)?.ok_or_else(|| configuration("fresh restore destination lock disappeared"))
        }
        Err(error) => Err(configuration(error)),
    }
}

/// Manifest's exact transport-independent byte binding.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct Fingerprint {
    /// SHA-256 over all copied bytes.
    pub sha256: String,
    /// Complete physical extent.
    pub bytes: u64,
}

/// One bounded catalog-owned instance; inline bodies never create these entries.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryEntry {
    /// Supervisor context coordinate, retained without physical authority.
    pub context: String,
    /// Original placement database.
    pub database: String,
    /// Original file-tier incarnation.
    pub payload_incarnation: String,
    /// Original file catalog row, including its explicit lifecycle state.
    pub instance: Map<String, Value>,
    /// Exact derived files, with physical byte bindings.
    pub files: BTreeMap<String, Fingerprint>,
}
impl InventoryEntry {
    fn text(&self, name: &str) -> Result<&str, CanonicalError> {
        self.instance.get(name).and_then(Value::as_str).ok_or_else(|| configuration(format!("missing file catalog {name}")))
    }
    fn uint(&self, name: &str) -> Result<u64, CanonicalError> {
        let value = self.instance.get(name).ok_or_else(|| configuration("missing file extent"))?;
        catalog_uint(value)
    }
    fn placement(&self) -> Placement {
        Placement { database: self.database.clone(), payload_incarnation: self.payload_incarnation.clone() }
    }
    fn paths(&self, state: &Path) -> Result<[PathBuf; 2], CanonicalError> {
        let key = self.text("key")?;
        canonical_uuid(key)?;
        if self.text("storage_incarnation")? != self.payload_incarnation {
            return Err(configuration("file instance belongs to another storage incarnation"));
        }
        let root = self.placement().root(state)?;
        Ok([root.join("instances").join(format!("{key}.sealed")), root.join(".pending").join(format!("{key}.pending"))])
    }
    fn state(&self) -> Result<CanonicalFileState, CanonicalError> {
        self.text("state")?.parse().map_err(configuration)
    }
    fn descriptor(&self) -> Result<Option<FileDescriptor>, CanonicalError> {
        let state = self.state()?;
        let unsealed = self.text("digest")? == "" && self.uint("payload_bytes")? == 0
            && self.text("encoding")? == "" && self.uint("batch_count")? == 0 && self.text("schema")? == "";
        if matches!(state, CanonicalFileState::Sealed | CanonicalFileState::Attached) || (state == CanonicalFileState::Collecting && !unsealed) {
            let descriptor = FileDescriptor { instance: self.text("key")?.into(), digest: self.text("digest")?.into(),
                bytes: self.uint("payload_bytes")?, encoding: self.text("encoding")?.into(), batch_count: self.uint("batch_count")?,
                schema: self.text("schema")?.into(), interpretation: self.text("interpretation")?.into() };
            descriptor.validate()?;
            Ok(Some(descriptor))
        } else { Ok(None) }
    }
}

fn catalog_uint(value: &Value) -> Result<u64, CanonicalError> {
    match value {
        Value::Number(number) => number.as_u64().ok_or_else(|| configuration("file extent is not an unsigned integer")),
        Value::String(text) if !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()) => text.parse().map_err(configuration),
        _ => Err(configuration("file extent is not an unsigned decimal")),
    }
}

/// Whole physical operation; no guard is returned for a later Python copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// Reconcile frozen crash residue and record complete physical inventory.
    Inventory,
    /// Copy exact inventoried live bytes while retaining each existing lock.
    Backup,
    /// Verify live bytes under their original locks without copying.
    VerifyLive,
    /// Check immutable archive bytes without requiring live source locks.
    Verify,
    /// Copy immutable archive bytes to fresh or exactly matching resumed placement.
    Restore,
}

fn sync_parent(path: &Path) -> Result<(), CanonicalError> {
    let parent = path.parent().ok_or_else(|| configuration("payload parent absent"))?;
    no_symlinks(parent)?;
    #[cfg(test)]
    PARENT_SYNC_OBSERVATION.with(|observation| {
        if let Some(observation) = observation.borrow_mut().take() { observation(parent); }
    });
    OpenOptions::new().read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(parent).and_then(|directory| directory.sync_all()).map_err(configuration)
}

fn reconcile(paths: &[PathBuf; 2]) -> Result<(), CanonicalError> {
    if !paths[0].try_exists().map_err(configuration)? || !paths[1].try_exists().map_err(configuration)? { return Ok(()); }
    let mut final_file = StableFile::open(&paths[0], false, 2)?;
    let pending = StableFile::open(&paths[1], false, 2)?;
    if identity(&final_file.before) != identity(&pending.before) {
        return Err(configuration("published staging alias has a different inode"));
    }
    final_file.unchanged()?;
    pending.unchanged()?;
    std::fs::remove_file(&paths[1]).map_err(configuration)?;
    sync_parent(&paths[1])?;
    final_file.before = final_file.file.metadata().map_err(configuration)?;
    private(&final_file.before, 1)?;
    final_file.unchanged()
}

fn copy(source: &mut StableFile, target: &Path, expected: &Fingerprint, descriptor: Option<&FileDescriptor>, resume: bool) -> Result<(), CanonicalError> {
    no_symlinks(target)?;
    let parent = target.parent().ok_or_else(|| configuration("copy destination parent absent"))?;
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(parent).map_err(configuration)?;
    no_symlinks(target)?;
    let directory = StableDirectory::open(parent)?;
    if target.try_exists().map_err(configuration)? {
        if !resume { return Err(configuration("payload backup destination already exists")); }
        let mut existing = StableFile::open(target, false, 1)?;
        if existing.fingerprint()? != *expected { return Err(configuration("resumed payload differs from its archive")); }
        if existing.before.dev() == source.before.dev() && existing.before.ino() == source.before.ino() {
            return Err(configuration("restore source aliases destination"));
        }
        if let Some(descriptor) = descriptor { validate_file(&mut existing.file, descriptor)?; }
        existing.file.sync_all().map_err(configuration)?;
        directory.file.sync_all().map_err(configuration)?;
        directory.unchanged()?;
        existing.unchanged()?;
        source.unchanged()?;
        return Ok(());
    }
    // No named partial payload survives process death. Unsupported filesystems
    // refuse here, before a target name or weaker temporary path is introduced.
    let mut output = anonymous_copy_file(&directory.file)?;
    #[cfg(test)]
    COPY_OBSERVATION.with(|observation| {
        if let Some(observation) = observation.borrow_mut().take() { observation(&source.path, target); }
    });
    source.file.seek(SeekFrom::Start(0)).map_err(configuration)?;
    std::io::copy(&mut source.file, &mut output).map_err(configuration)?;
    output.sync_all().map_err(configuration)?;
    if fingerprint_file(&mut output)? != *expected { return Err(configuration("copied payload differs from frozen inventory")); }
    if source.fingerprint()? != *expected { return Err(configuration("source changed during offline payload copy")); }
    if let Some(descriptor) = descriptor {
        validate_file(&mut output, descriptor)?;
        validate_file(&mut source.file, descriptor)?;
    }
    source.unchanged()?;
    directory.unchanged()?;
    #[cfg(test)]
    COPY_BEFORE_PUBLICATION.with(|observation| {
        observation.borrow_mut().take().map_or(Ok(()), |observation| observation(&source.path, target))
    })?;
    let name = target.file_name().ok_or_else(|| configuration("copy destination name absent"))?;
    let descriptor_path = PathBuf::from(format!("/proc/self/fd/{}", output.as_raw_fd()));
    // This fixed self-owned kernel descriptor link is deliberate. linkat's
    // destination is the held no-follow directory and never replaces a name.
    rustix::fs::linkat(rustix::fs::CWD, &descriptor_path, &directory.file, name, rustix::fs::AtFlags::SYMLINK_FOLLOW)
        .map_err(configuration)?;
    output.sync_all().map_err(configuration)?;
    directory.file.sync_all().map_err(configuration)?;
    directory.unchanged()?;
    #[cfg(test)]
    COPY_AFTER_PUBLICATION.with(|observation| {
        observation.borrow_mut().take().map_or(Ok(()), |observation| observation(&source.path, target))
    })?;
    let before = output.metadata().map_err(configuration)?;
    private(&before, 1)?;
    let mut copied = StableFile { file: output, path: target.into(), before };
    if copied.fingerprint()? != *expected { return Err(configuration("published copy differs from frozen inventory")); }
    if let Some(descriptor) = descriptor { validate_file(&mut copied.file, descriptor)?; }
    source.unchanged()?;
    copied.unchanged()
}

/// Read one entry with a hard allocation bound before decoding.
pub fn read_entry(reader: &mut impl BufRead) -> Result<Option<Vec<u8>>, CanonicalError> {
    let mut bytes = Vec::new();
    let length = reader.take((ENTRY_BYTES + 1) as u64).read_until(b'\n', &mut bytes).map_err(configuration)?;
    if length == 0 { return Ok(None); }
    if length > ENTRY_BYTES { return Err(configuration("payload inventory entry exceeds bounded envelope")); }
    Ok(Some(bytes))
}

/// Perform a bounded stream of complete instance operations, retaining every
/// instance's guard through reconciliation, byte work and final validation.
pub fn run(operation: Operation, state: &Path, placement: Option<&Placement>, destination: Option<&Path>, target: Option<&Placement>,
    input: &mut impl BufRead, output: &mut impl Write) -> Result<(), CanonicalError>
{
    absolute(state)?;
    if let Some(destination) = destination {
        absolute(destination)?;
        if destination == state || destination.starts_with(state) || state.starts_with(destination) {
            return Err(configuration("offline copy roots must be disjoint"));
        }
    }
    let live = matches!(operation, Operation::Inventory | Operation::Backup | Operation::VerifyLive);
    let copying = matches!(operation, Operation::Backup | Operation::Restore);
    if copying && destination.is_none() { return Err(configuration("copy operation requires destination")); }
    let mut expected = BTreeSet::new();
    let mut target_expected = BTreeSet::new();
    let mut target_roots = BTreeSet::new();
    let mut roots = BTreeSet::new();
    if let Some(placement) = placement { roots.insert(placement.root(state)?); }
    if operation == Operation::Restore && let (Some(destination), Some(placement)) = (destination, target.or(placement)) {
        target_roots.insert(placement.root(destination)?);
    }
    let mut instances = BTreeSet::new();
    while let Some(line) = read_entry(input)? {
        let mut entry: InventoryEntry = serde_json::from_slice(&line).map_err(configuration)?;
        let paths = entry.paths(state)?;
        let root = entry.placement().root(state)?;
        roots.insert(root.clone());
        if !instances.insert((root.clone(), entry.text("key")?.to_owned())) {
            return Err(configuration("duplicate file instance inventory"));
        }
        let descriptor = entry.descriptor()?;
        let lock = if live { ExistingLock::acquire(&root, entry.text("key")?)? } else { None };
        if live {
            if (paths[0].try_exists().map_err(configuration)? || paths[1].try_exists().map_err(configuration)?) && lock.is_none() {
                return Err(configuration("existing payload lacks its original instance lock"));
            }
            reconcile(&paths)?;
        }
        let original = std::mem::take(&mut entry.files);
        let destination_lock = if operation == Operation::Restore && !original.is_empty() {
            let destination = destination.ok_or_else(|| configuration("restore destination absent"))?;
            let placement = target.cloned().unwrap_or_else(|| entry.placement());
            let directory = placement.root(destination)?;
            target_roots.insert(directory.clone());
            Some(restore_destination_lock(&directory, entry.text("key")?)?)
        } else { None };
        let mut found = BTreeMap::new();
        for (index, path) in paths.iter().enumerate() {
            no_symlinks(path)?;
            let name = path.strip_prefix(state).map_err(configuration)?.to_string_lossy().into_owned();
            let exists = path.try_exists().map_err(configuration)?;
            if !exists {
                if index == 0 && matches!(entry.state()?, CanonicalFileState::Sealed | CanonicalFileState::Attached) {
                    return Err(configuration("required sealed file is missing"));
                }
                if original.contains_key(&name) { return Err(configuration("inventoried payload is missing")); }
                continue;
            }
            let mut file = StableFile::open(path, false, 1)?;
            if index == 0 && let Some(descriptor) = &descriptor { validate_file(&mut file.file, descriptor)?; }
            let fingerprint = file.fingerprint()?;
            if operation != Operation::Inventory && original.get(&name) != Some(&fingerprint) {
                return Err(configuration("payload differs from its exact frozen inventory"));
            }
            if !expected.insert(path.clone()) { return Err(configuration("ambiguous physical inventory")); }
            if copying {
                let destination = destination.ok_or_else(|| configuration("missing copy destination"))?;
                let target_path = if let Some(placement) = target {
                    let directory = placement.root(destination)?;
                    directory.join(if index == 0 { "instances" } else { ".pending" }).join(path.file_name().ok_or_else(|| configuration("payload name absent"))?)
                } else { destination.join(&name) };
                if operation == Operation::Restore { target_expected.insert(target_path.clone()); }
                copy(&mut file, &target_path, &fingerprint, if index == 0 { descriptor.as_ref() } else { None }, operation == Operation::Restore)?;
            }
            if index == 0 && let Some(descriptor) = &descriptor { validate_file(&mut file.file, descriptor)?; }
            file.unchanged()?;
            found.insert(name, fingerprint);
        }
        if operation != Operation::Inventory && original != found {
            return Err(configuration("payload inventory contains unknown file coordinates"));
        }
        if let Some(lock) = lock {
            lock.unchanged()?;
            expected.insert(lock.0.path.clone());
        }
        if let Some(lock) = destination_lock {
            lock.unchanged()?;
            target_expected.insert(lock.0.path.clone());
        }
        entry.files = found;
        let encoded = serde_json::to_vec(&entry).map_err(configuration)?;
        if encoded.len() + 1 > ENTRY_BYTES { return Err(configuration("output inventory entry exceeds bounded envelope")); }
        output.write_all(&encoded).and_then(|()| output.write_all(b"\n")).map_err(configuration)?;
    }
    // Inspect complete placement trees, including unknown locks and symlinks. No
    // tree entry is trusted merely because a prior catalog record was accepted.
    if !live { roots.insert(state.join("payloads")); }
    for root in roots { check_tree(&root, &expected)?; }
    for root in target_roots { check_tree(&root, &target_expected)?; }
    output.flush().map_err(configuration)
}

fn check_tree(root: &Path, expected: &BTreeSet<PathBuf>) -> Result<(), CanonicalError> {
    no_symlinks(root)?;
    if !root.try_exists().map_err(configuration)? { return Ok(()); }
    let metadata = std::fs::symlink_metadata(root).map_err(configuration)?;
    if !metadata.is_dir() || metadata.uid() != rustix::process::getuid().as_raw() {
        return Err(configuration("payload directory lost its private ownership"));
    }
    for entry in std::fs::read_dir(root).map_err(configuration)? {
        let entry = entry.map_err(configuration)?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path).map_err(configuration)?;
        if metadata.is_dir() { check_tree(&path, expected)?; }
        else if !metadata.is_file() || !expected.contains(&path) {
            return Err(configuration("unknown payload bytes or synchronization lock refuse complete inventory"));
        }
    }
    Ok(())
}

/// Exact factual Linux owner observation. A foreign namespace or inaccessible
/// process is an error; manager/cgroup drain remains the supervisor's concern.
pub fn owner_dead(owner: &Map<String, Value>) -> Result<bool, CanonicalError> {
    let text = |name: &str| owner.get(name).and_then(Value::as_str).ok_or_else(|| configuration("incomplete file owner identity"));
    canonical_uuid(text("key")?)?;
    let boot = text("boot")?;
    let namespace = text("pid_namespace")?;
    let start = text("start")?;
    let raw = owner.get("pid").ok_or_else(|| configuration("missing owner PID"))?;
    let pid = catalog_uint(raw)?;
    if pid == 0 || pid > u32::MAX as u64 || raw.as_str().is_some_and(|text| text != pid.to_string()) {
        return Err(configuration("invalid file owner PID"));
    }
    if std::fs::read_to_string("/proc/sys/kernel/random/boot_id").map_err(configuration)?.trim() != boot { return Ok(true); }
    let current = std::fs::read_link("/proc/self/ns/pid").map_err(configuration)?;
    if current.to_string_lossy() != namespace { return Err(configuration("file owner PID namespace cannot be authenticated")); }
    let process = PathBuf::from(format!("/proc/{pid}"));
    let observed = (|| {
        let namespace = std::fs::read_link(process.join("ns/pid"))?;
        let stat = std::fs::read_to_string(process.join("stat"))?;
        Ok::<_, std::io::Error>((namespace, stat))
    })();
    match observed {
        Ok((actual, stat)) => {
            if actual != current { return Err(configuration("file owner namespace changed during observation")); }
            let (_, tail) = stat.rsplit_once(')').ok_or_else(|| configuration("owner process stat malformed"))?;
            let actual_start = tail.split_whitespace().nth(19).ok_or_else(|| configuration("owner process start absent"))?;
            Ok(actual_start != start)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            match std::fs::metadata(process) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
                _ => Err(configuration("file owner identity observation is uncertain")),
            }
        }
        Err(error) => Err(configuration(error)),
    }
}

/// Validate or remove a retired, already drained physical placement. Validation
/// precedes every unlink; the exact existing lock is retained through deletion.
pub fn retired(state: &Path, placement: &Placement, input: &mut impl BufRead, remove: bool) -> Result<(), CanonicalError> {
    let root = placement.root(state)?;
    let mut keys = BTreeSet::new();
    let mut expected = BTreeSet::new();
    let mut bindings = BTreeMap::new();
    while let Some(line) = read_entry(input)? {
        let entry: InventoryEntry = serde_json::from_slice(&line).map_err(configuration)?;
        let original = entry.paths(state)?;
        entry.state()?;
        let key = entry.text("key")?.to_owned();
        for path in &original {
            let name = path.strip_prefix(state).map_err(configuration)?.to_string_lossy();
            if entry.files.contains_key(name.as_ref()) {
                let directory = if path.extension().is_some_and(|ext| ext == "sealed") { "instances" } else { ".pending" };
                let target = root.join(directory).join(path.file_name().ok_or_else(|| configuration("retired name absent"))?);
                expected.insert(target.clone());
                bindings.insert(target, entry.files.get(name.as_ref()).ok_or_else(|| configuration("retired byte binding absent"))?.clone());
            }
        }
        if entry.files.keys().any(|name| !original.iter().any(|path| path.strip_prefix(state).is_ok_and(|path| path.to_string_lossy() == *name))) {
            return Err(configuration("retired payload has invalid original coordinates"));
        }
        if !keys.insert(key) { return Err(configuration("duplicate retired instance inventory")); }
    }
    for key in &keys {
        let lock = ExistingLock::acquire(&root, key)?;
        for path in [root.join("instances").join(format!("{key}.sealed")), root.join(".pending").join(format!("{key}.pending"))] {
            no_symlinks(&path)?;
            if !path.try_exists().map_err(configuration)? { continue; }
            if lock.is_none() { return Err(configuration("retired payload lacks original lock")); }
            let expected_binding = bindings.get(&path).ok_or_else(|| configuration("retired payload lacks original byte inventory"))?;
            let mut file = StableFile::open(&path, false, 1)?;
            if file.fingerprint()? != *expected_binding { return Err(configuration("retired payload differs from exact inventoried bytes")); }
        }
        if let Some(lock) = lock { lock.unchanged()?; expected.insert(lock.0.path.clone()); }
    }
    check_tree(&root, &expected)?;
    if !remove || !root.try_exists().map_err(configuration)? { return Ok(()); }
    for key in keys {
        let lock = ExistingLock::acquire(&root, &key)?;
        for path in [root.join("instances").join(format!("{key}.sealed")), root.join(".pending").join(format!("{key}.pending"))] {
            no_symlinks(&path)?;
            if !path.try_exists().map_err(configuration)? { continue; }
            if !expected.contains(&path) || lock.is_none() { return Err(configuration("retired file lost exact inventory or lock")); }
            let mut file = StableFile::open(&path, false, 1)?;
            if Some(&file.fingerprint()?) != bindings.get(&path) { return Err(configuration("retired payload changed before removal")); }
            file.unchanged()?;
            std::fs::remove_file(&path).map_err(configuration)?;
            sync_parent(&path)?;
        }
        if let Some(lock) = lock {
            lock.unchanged()?;
            std::fs::remove_file(&lock.0.path).map_err(configuration)?;
            sync_parent(&lock.0.path)?;
        }
    }
    remove_empty_directories(&root)
}
fn remove_empty_directories(root: &Path) -> Result<(), CanonicalError> {
    for entry in std::fs::read_dir(root).map_err(configuration)? {
        let path = entry.map_err(configuration)?.path();
        no_symlinks(&path)?;
        if !std::fs::symlink_metadata(&path).map_err(configuration)?.is_dir() {
            return Err(configuration("unknown retired payload retained"));
        }
        remove_empty_directories(&path)?;
    }
    std::fs::remove_dir(root).map_err(configuration)?;
    sync_parent(root)
}

#[cfg(test)]
type CopyObservation = Box<dyn FnOnce(&Path, &Path)>;
#[cfg(test)]
thread_local! { static COPY_OBSERVATION: std::cell::RefCell<Option<CopyObservation>> = const { std::cell::RefCell::new(None) }; }
#[cfg(test)]
type ParentSyncObservation = Box<dyn FnOnce(&Path)>;
#[cfg(test)]
thread_local! { static PARENT_SYNC_OBSERVATION: std::cell::RefCell<Option<ParentSyncObservation>> = const { std::cell::RefCell::new(None) }; }
#[cfg(test)]
type PublicationObservation = Box<dyn FnOnce(&Path, &Path) -> Result<(), CanonicalError>>;
#[cfg(test)]
thread_local! {
    static COPY_BEFORE_PUBLICATION: std::cell::RefCell<Option<PublicationObservation>> = const { std::cell::RefCell::new(None) };
    static COPY_AFTER_PUBLICATION: std::cell::RefCell<Option<PublicationObservation>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn fixture() -> Result<(tempfile::TempDir, Placement, InventoryEntry), CanonicalError> {
        let directory = tempfile::tempdir().map_err(configuration)?;
        let placement = Placement { payload_incarnation: uuid::Uuid::new_v4().to_string(), database: "scientific".into() };
        let key = uuid::Uuid::new_v4().to_string();
        let bytes = b"scientific payload";
        let descriptor = FileDescriptor { instance: key.clone(), digest: crate::canonical_execution::result_payload_digest(bytes), bytes: bytes.len() as u64,
            encoding: super::super::OPAQUE_BYTES.into(), batch_count: 0, schema: String::new(), interpretation: crate::generated::surreal::INTERPRETATION.into() };
        super::super::publish(&placement.root(directory.path())?, &descriptor, bytes)?;
        let row = serde_json::json!({"key":key,"storage_incarnation":placement.payload_incarnation,"digest":descriptor.digest,"payload_bytes":descriptor.bytes.to_string(),
            "encoding":descriptor.encoding,"batch_count":"0","schema":"","interpretation":descriptor.interpretation,"state":"sealed"});
        let instance = row.as_object().cloned().ok_or_else(|| configuration("fixture row absent"))?;
        let entry = InventoryEntry { context: ".".into(), database: placement.database.clone(), payload_incarnation: placement.payload_incarnation.clone(), instance, files: BTreeMap::new() };
        Ok((directory, placement, entry))
    }
    fn execute(operation: Operation, state: &Path, placement: &Placement, entry: &InventoryEntry, destination: Option<&Path>, target: Option<&Placement>) -> Result<InventoryEntry, CanonicalError> {
        let mut input = Cursor::new(serde_json::to_vec(entry).map_err(configuration)?);
        let mut output = Vec::new();
        run(operation, state, Some(placement), destination, target, &mut input, &mut output)?;
        serde_json::from_slice(&output).map_err(configuration)
    }

    #[test]
    fn offline_complete_inventory_backup_and_lockless_archive_restore() -> Result<(), CanonicalError> {
        let (source, placement, entry) = fixture()?;
        let backup = tempfile::tempdir().map_err(configuration)?;
        let restored = tempfile::tempdir().map_err(configuration)?;
        let inventory = execute(Operation::Inventory, source.path(), &placement, &entry, None, None)?;
        assert_eq!(inventory.files.len(), 1);
        execute(Operation::Backup, source.path(), &placement, &inventory, Some(backup.path()), None)?;
        assert!(!placement.root(backup.path())?.join(".locks").exists());
        execute(Operation::Verify, backup.path(), &placement, &inventory, None, None)?;
        let fresh = Placement { payload_incarnation: uuid::Uuid::new_v4().to_string(), database: "restored".into() };
        execute(Operation::Restore, backup.path(), &placement, &inventory, Some(restored.path()), Some(&fresh))?;
        // Exact interrupted restore is resumable without copying or adopting a new inode.
        execute(Operation::Restore, backup.path(), &placement, &inventory, Some(restored.path()), Some(&fresh))?;
        assert!(fresh.root(restored.path())?.join(".locks").exists());
        Ok(())
    }
    #[test]
    fn offline_existing_lock_is_required_and_live_borrower_refuses() -> Result<(), CanonicalError> {
        let (source, placement, entry) = fixture()?;
        let key = entry.text("key")?;
        let root = placement.root(source.path())?;
        let lock = ExistingLock::acquire(&root, key)?.ok_or_else(|| configuration("fixture lock absent"))?;
        assert!(execute(Operation::Inventory, source.path(), &placement, &entry, None, None).is_err());
        drop(lock);
        std::fs::remove_file(root.join(".locks").join(format!("{key}.lock"))).map_err(configuration)?;
        assert!(execute(Operation::Inventory, source.path(), &placement, &entry, None, None).is_err());
        Ok(())
    }
    #[test]
    fn offline_only_authenticated_final_pending_two_link_alias_reconciles() -> Result<(), CanonicalError> {
        let (source, placement, entry) = fixture()?;
        let paths = entry.paths(source.path())?;
        std::fs::create_dir_all(paths[1].parent().ok_or_else(|| configuration("fixture parent"))?).map_err(configuration)?;
        std::fs::hard_link(&paths[0], &paths[1]).map_err(configuration)?;
        execute(Operation::Inventory, source.path(), &placement, &entry, None, None)?;
        assert!(!paths[1].exists());
        let unknown = source.path().join("unknown-alias");
        std::fs::hard_link(&paths[0], unknown).map_err(configuration)?;
        assert!(execute(Operation::Inventory, source.path(), &placement, &entry, None, None).is_err());
        Ok(())
    }
    #[test]
    fn offline_unknown_content_symlinks_corruption_and_private_mode_refuse() -> Result<(), CanonicalError> {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let (source, placement, entry) = fixture()?;
        let inventory = execute(Operation::Inventory, source.path(), &placement, &entry, None, None)?;
        let path = entry.paths(source.path())?[0].clone();
        let unknown = placement.root(source.path())?.join("unknown");
        std::fs::write(&unknown, b"unknown").map_err(configuration)?;
        assert!(execute(Operation::Inventory, source.path(), &placement, &entry, None, None).is_err());
        std::fs::remove_file(&unknown).map_err(configuration)?;
        symlink(&path, &unknown).map_err(configuration)?;
        assert!(execute(Operation::Inventory, source.path(), &placement, &entry, None, None).is_err());
        std::fs::remove_file(unknown).map_err(configuration)?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).map_err(configuration)?;
        assert!(execute(Operation::Inventory, source.path(), &placement, &entry, None, None).is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o4600)).map_err(configuration)?;
        assert!(execute(Operation::Inventory, source.path(), &placement, &entry, None, None).is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).map_err(configuration)?;
        std::fs::write(&path, vec![b'x'; entry.uint("payload_bytes")? as usize]).map_err(configuration)?;
        assert!(execute(Operation::VerifyLive, source.path(), &placement, &inventory, None, None).is_err());
        Ok(())
    }
    #[test]
    fn offline_pending_absence_and_partial_bytes_preserve_unknown_effect() -> Result<(), CanonicalError> {
        let (source, placement, mut entry) = fixture()?;
        let paths = entry.paths(source.path())?;
        std::fs::remove_file(&paths[0]).map_err(configuration)?;
        entry.instance.insert("state".into(), Value::String("pending".into()));
        entry.instance.insert("digest".into(), Value::String(String::new()));
        entry.instance.insert("payload_bytes".into(), Value::String("0".into()));
        entry.instance.insert("encoding".into(), Value::String(String::new()));
        let empty = execute(Operation::Inventory, source.path(), &placement, &entry, None, None)?;
        assert!(empty.files.is_empty());
        std::fs::create_dir_all(paths[1].parent().ok_or_else(|| configuration("fixture parent"))?).map_err(configuration)?;
        let mut file = OpenOptions::new().write(true).create_new(true).mode(0o600).open(&paths[1]).map_err(configuration)?;
        file.write_all(b"partial").map_err(configuration)?;
        let partial = execute(Operation::Inventory, source.path(), &placement, &entry, None, None)?;
        assert_eq!(partial.files.len(), 1);
        assert_eq!(partial.text("state")?, "pending");
        Ok(())
    }
    #[test]
    fn offline_extent_entry_bound_and_owner_observation_are_exact() -> Result<(), CanonicalError> {
        let mut large = Cursor::new(vec![b'x'; ENTRY_BYTES + 10]);
        assert!(read_entry(&mut large).is_err());
        assert_eq!(large.position(), ENTRY_BYTES as u64 + 1);
        assert_eq!(catalog_uint(&Value::String(u64::MAX.to_string()))?, u64::MAX);
        assert!(catalog_uint(&Value::String("18446744073709551616".into())).is_err());
        let current = super::super::ProcessIdentity::current()?;
        let owner = serde_json::json!({"key":uuid::Uuid::new_v4().to_string(),"boot":current.boot,"pid_namespace":current.pid_namespace,"start":current.start,"pid":current.pid.to_string()});
        let mut owner = owner.as_object().cloned().ok_or_else(|| configuration("owner fixture"))?;
        assert!(!owner_dead(&owner)?);
        owner.insert("start".into(), Value::String("0".into()));
        assert!(owner_dead(&owner)?);
        owner.insert("pid_namespace".into(), Value::String("pid:[foreign]".into()));
        assert!(owner_dead(&owner).is_err());
        Ok(())
    }
    #[test]
    fn offline_guard_conflicts_with_online_flock_and_survives_copy_boundary() -> Result<(), CanonicalError> {
        use rustix::fs::{flock, FlockOperation};
        let (source, placement, entry) = fixture()?;
        let root = placement.root(source.path())?;
        let key = entry.text("key")?.to_owned();
        let online = super::super::instance_lock_file(&root, &key)?;
        flock(&online, FlockOperation::NonBlockingLockExclusive).map_err(configuration)?;
        assert!(ExistingLock::acquire(&root, &key).is_err());
        flock(&online, FlockOperation::Unlock).map_err(configuration)?;
        let lock = ExistingLock::acquire(&root, &key)?.ok_or_else(|| configuration("lock absent"))?;
        assert!(flock(&online, FlockOperation::NonBlockingLockExclusive).is_err());
        drop(lock);
        let inventory = execute(Operation::Inventory, source.path(), &placement, &entry, None, None)?;
        let backup = tempfile::tempdir().map_err(configuration)?;
        COPY_OBSERVATION.with(|observation| *observation.borrow_mut() = Some(Box::new(move |_, _| {
            assert!(flock(&online, FlockOperation::NonBlockingLockExclusive).is_err());
        })));
        execute(Operation::Backup, source.path(), &placement, &inventory, Some(backup.path()), None)?;
        Ok(())
    }
    #[test]
    fn offline_copy_refuses_source_inode_replacement_and_destination_symlink() -> Result<(), CanonicalError> {
        use std::os::unix::fs::symlink;
        for replacement in 0..4 {
            let (source, placement, entry) = fixture()?;
            let inventory = execute(Operation::Inventory, source.path(), &placement, &entry, None, None)?;
            let backup = tempfile::tempdir().map_err(configuration)?;
            COPY_OBSERVATION.with(|observation| *observation.borrow_mut() = Some(Box::new(move |source, target| {
                if replacement == 0 {
                    std::fs::rename(source, source.with_extension("original")).unwrap();
                    let mut replacement = OpenOptions::new().write(true).create_new(true).mode(0o600).open(source).unwrap();
                    replacement.write_all(b"scientific payload").unwrap();
                } else {
                    if target.try_exists().unwrap() { std::fs::remove_file(target).unwrap(); }
                    match replacement {
                        1 => { symlink(source, target).unwrap(); }
                        2 => { std::fs::write(target, b"different inode").unwrap(); }
                        _ => { std::fs::hard_link(source, target).unwrap(); }
                    }
                }
            })));
            assert!(execute(Operation::Backup, source.path(), &placement, &inventory, Some(backup.path()), None).is_err());
        }
        Ok(())
    }
    #[test]
    fn offline_retired_cleanup_refuses_unknown_or_held_lock_before_unlink() -> Result<(), CanonicalError> {
        let (source, placement, entry) = fixture()?;
        let inventory = execute(Operation::Inventory, source.path(), &placement, &entry, None, None)?;
        let root = placement.root(source.path())?;
        let unknown = root.join("unknown");
        std::fs::write(&unknown, b"preserved").map_err(configuration)?;
        let encoded = serde_json::to_vec(&inventory).map_err(configuration)?;
        assert!(retired(source.path(), &placement, &mut Cursor::new(&encoded), true).is_err());
        assert!(entry.paths(source.path())?[0].exists());
        std::fs::remove_file(unknown).map_err(configuration)?;
        let lock = ExistingLock::acquire(&root, entry.text("key")?)?.ok_or_else(|| configuration("lock absent"))?;
        assert!(retired(source.path(), &placement, &mut Cursor::new(&encoded), true).is_err());
        assert!(entry.paths(source.path())?[0].exists());
        drop(lock);
        retired(source.path(), &placement, &mut Cursor::new(encoded), true)?;
        assert!(!root.exists());
        Ok(())
    }

    #[test]
    fn offline_canonical_digest_and_archive_fingerprint_are_independent() -> Result<(), CanonicalError> {
        let (source, placement, entry) = fixture()?;
        let mut inventory = execute(Operation::Inventory, source.path(), &placement, &entry, None, None)?;
        let correct = inventory.instance.get("digest").cloned().ok_or_else(|| configuration("fixture digest absent"))?;
        inventory.instance.insert("digest".into(), Value::String("0".repeat(64)));
        assert!(execute(Operation::VerifyLive, source.path(), &placement, &inventory, None, None).is_err());
        inventory.instance.insert("digest".into(), correct);
        for binding in inventory.files.values_mut() { binding.sha256 = "0".repeat(64); }
        assert!(execute(Operation::VerifyLive, source.path(), &placement, &inventory, None, None).is_err());
        Ok(())
    }

    #[test]
    fn offline_owner_observation_refuses_noncanonical_catalog_key() -> Result<(), CanonicalError> {
        let current = super::super::ProcessIdentity::current()?;
        let key = "abcdef01-2345-4678-9abc-def012345678".to_owned();
        let owner = serde_json::json!({"key":key,"boot":current.boot,"pid_namespace":current.pid_namespace,"start":current.start,"pid":current.pid.to_string()});
        let mut owner = owner.as_object().cloned().ok_or_else(|| configuration("owner fixture"))?;
        assert!(!owner_dead(&owner)?);
        for invalid in ["not-a-uuid".to_owned(), key.to_uppercase(), key.replace('-', "")] {
            owner.insert("key".into(), Value::String(invalid));
            assert!(owner_dead(&owner).is_err());
        }
        owner.remove("key");
        assert!(owner_dead(&owner).is_err());
        Ok(())
    }

    #[test]
    fn offline_unnamed_copy_interruption_and_exact_published_restore_resume() -> Result<(), CanonicalError> {
        let (source, placement, entry) = fixture()?;
        let inventory = execute(Operation::Inventory, source.path(), &placement, &entry, None, None)?;
        let archive = tempfile::tempdir().map_err(configuration)?;
        execute(Operation::Backup, source.path(), &placement, &inventory, Some(archive.path()), None)?;
        for after_link in [false, true] {
            let restored = tempfile::tempdir().map_err(configuration)?;
            let target = Placement { payload_incarnation: uuid::Uuid::new_v4().to_string(), database: "restored".into() };
            let path = target.root(restored.path())?.join("instances").join(format!("{}.sealed", entry.text("key")?));
            let hook: PublicationObservation = Box::new(move |_, target| {
                assert_eq!(target.exists(), after_link);
                Err(configuration("injected process interruption at publication boundary"))
            });
            if after_link { COPY_AFTER_PUBLICATION.with(|observation| *observation.borrow_mut() = Some(hook)); }
            else { COPY_BEFORE_PUBLICATION.with(|observation| *observation.borrow_mut() = Some(hook)); }
            assert!(execute(Operation::Restore, archive.path(), &placement, &inventory, Some(restored.path()), Some(&target)).is_err());
            assert_eq!(path.exists(), after_link);
            let inode = if after_link { Some(std::fs::metadata(&path).map_err(configuration)?.ino()) } else { None };
            execute(Operation::Restore, archive.path(), &placement, &inventory, Some(restored.path()), Some(&target))?;
            if let Some(inode) = inode { assert_eq!(std::fs::metadata(&path).map_err(configuration)?.ino(), inode); }
        }
        Ok(())
    }
    #[test]
    fn offline_anonymous_publication_never_overwrites_a_racing_destination() -> Result<(), CanonicalError> {
        let (source, placement, entry) = fixture()?;
        let inventory = execute(Operation::Inventory, source.path(), &placement, &entry, None, None)?;
        let backup = tempfile::tempdir().map_err(configuration)?;
        COPY_BEFORE_PUBLICATION.with(|observation| *observation.borrow_mut() = Some(Box::new(|_, target| {
            std::fs::write(target, b"preserve independently created destination").map_err(configuration)
        })));
        assert!(execute(Operation::Backup, source.path(), &placement, &inventory, Some(backup.path()), None).is_err());
        let path = entry.paths(backup.path())?[0].clone();
        assert_eq!(std::fs::read(path).map_err(configuration)?, b"preserve independently created destination");
        let file = File::open(entry.paths(source.path())?[0].clone()).map_err(configuration)?;
        assert!(anonymous_copy_file(&file).is_err());
        Ok(())
    }

    #[test]
    fn offline_parent_sync_refuses_symlink_substitution_after_precheck() -> Result<(), CanonicalError> {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().map_err(configuration)?;
        let parent = root.path().join("intended");
        let replacement = root.path().join("unrelated");
        std::fs::create_dir(&parent).map_err(configuration)?;
        std::fs::create_dir(&replacement).map_err(configuration)?;
        PARENT_SYNC_OBSERVATION.with(|observation| *observation.borrow_mut() = Some(Box::new(move |parent| {
            std::fs::rename(parent, parent.with_extension("original")).unwrap();
            symlink(replacement, parent).unwrap();
        })));
        assert!(sync_parent(&parent.join("payload")).is_err());
        Ok(())
    }

}
