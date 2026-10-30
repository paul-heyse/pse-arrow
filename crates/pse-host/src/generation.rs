// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable tool provenance and explicit generation-use evidence.
//!
//! This module does not publish selectors, rebuild tools or prune data. Construction and
//! original workload owners integrate its primitives at those actual effect boundaries.

use crate::{HostError, environment::Environment};
use cargo_metadata::{DependencyKind, Metadata, MetadataCommand, PackageId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{OsStr, OsString},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    },
    path::{Component, Path, PathBuf},
};

const VERSION: u32 = 1;
const MANIFEST: &str = "generation.json";
const REQUIRED_INPUTS: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    ".cargo/config.toml",
];
const BUILD_VARIABLES: &[&str] = &[
    "RUSTC",
    "RUSTFLAGS",
    "RUSTDOCFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_ENCODED_RUSTDOCFLAGS",
    "CARGO_INCREMENTAL",
    "CARGO_PROFILE_DEV_OPT_LEVEL",
    "CARGO_PROFILE_DEV_DEBUG",
    "CARGO_PROFILE_DEV_PANIC",
    "CARGO_PROFILE_DEV_LTO",
    "CARGO_PROFILE_DEV_CODEGEN_UNITS",
    "SOURCE_DATE_EPOCH",
    "CC",
    "CXX",
    "AR",
];

mod bootstrap;
pub use bootstrap::{
    Freshness, HostTool, PreparedGeneration, bootstrap, bootstrap_from, freshness, prepared,
};
mod entry;
pub use entry::{EntryGuard, EntrySettlement, NewWorkDecision, new_work_decision};
mod transfer;
pub use transfer::rebuild_and_reexec;

fn invalid(message: impl Into<String>) -> HostError {
    HostError::Ownership(format!("host generation: {}", message.into()))
}
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn hash_file(path: &Path) -> Result<String, HostError> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    let before = file.metadata()?;
    if !before.is_file() {
        return Err(invalid("fingerprinted input is not a regular file"));
    }
    let mut hash = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    let after = file.metadata()?;
    if (
        before.len(),
        before.mtime(),
        before.mtime_nsec(),
        before.ctime(),
        before.ctime_nsec(),
    ) != (
        after.len(),
        after.mtime(),
        after.mtime_nsec(),
        after.ctime(),
        after.ctime_nsec(),
    ) {
        return Err(invalid("fingerprinted input changed while being read"));
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
fn encoded<T: Serialize>(value: &T) -> Result<Vec<u8>, HostError> {
    serde_json::to_vec(value).map_err(|_| invalid("provenance cannot be encoded"))
}
fn component(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}
fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}
fn checked_input(root: &Path, path: &Path) -> Result<PathBuf, HostError> {
    if !relative(path) {
        return Err(invalid("source input is not checkout-relative"));
    }
    let mut selected = root.to_owned();
    for component in path.components() {
        selected.push(component);
        if fs::symlink_metadata(&selected)?.file_type().is_symlink() {
            return Err(invalid("symlink source input is unsupported"));
        }
    }
    Ok(selected)
}
fn private_directory(path: &Path, create: bool) -> Result<(), HostError> {
    if create && !path.exists() {
        fs::DirBuilder::new().mode(0o700).create(path)?;
    }
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o022 != 0
    {
        return Err(invalid("tool directory is not a private owned directory"));
    }
    Ok(())
}
fn owned_file(path: &Path, create: bool) -> Result<File, HostError> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(create)
        .truncate(false)
        .mode(0o600)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    validate_owned_file(&file)?;
    Ok(file)
}
fn validate_owned_file(file: &File) -> Result<(), HostError> {
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o022 != 0
    {
        return Err(invalid("tool lock is not an owned regular file"));
    }
    Ok(())
}

/// Actual compiler/build selection, distinct from the source dependency graph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildContext {
    rustc: String,
    cargo: String,
    profile: String,
    features: BTreeSet<String>,
    environment: BTreeMap<String, Option<String>>,
}
impl BuildContext {
    /// Bind observed tool version output and the selected build profile/features.
    /// Values which may contain private configuration are represented only by digests.
    pub fn new(
        rustc: String,
        cargo: String,
        profile: String,
        features: BTreeSet<String>,
        environment: &Environment,
    ) -> Self {
        let environment = BUILD_VARIABLES
            .iter()
            .map(|name| {
                (
                    (*name).to_owned(),
                    environment
                        .get(OsStr::new(name))
                        .map(|value| digest(value.as_bytes())),
                )
            })
            .collect();
        Self {
            rustc,
            cargo,
            profile,
            features,
            environment,
        }
    }
    /// Record further inputs selected by the builder, such as target-specific compiler
    /// variables or a native provider selection. Their values remain digest-only.
    /// This must cover every additional input the actual construction environment permits.
    pub fn with_additional_inputs(mut self, inputs: &BTreeMap<String, Option<OsString>>) -> Self {
        self.environment.extend(inputs.iter().map(|(name, value)| {
            (
                name.clone(),
                value.as_ref().map(|value| digest(value.as_bytes())),
            )
        }));
        self
    }
}

/// A portable-relative source observation. Paths are relative to the owning checkout.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFingerprint {
    version: u32,
    package_roots: BTreeSet<PathBuf>,
    additional_files: BTreeSet<PathBuf>,
    files: BTreeMap<PathBuf, String>,
    context: BuildContext,
}

fn collect(
    root: &Path,
    directory: &Path,
    files: &mut BTreeMap<PathBuf, String>,
) -> Result<(), HostError> {
    let mut entries =
        fs::read_dir(checked_input(root, directory)?)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        if entry.file_name() == "target" || entry.file_name() == ".git" {
            continue;
        }
        let path = directory.join(entry.file_name());
        let metadata = fs::symlink_metadata(root.join(&path))?;
        if metadata.file_type().is_symlink() {
            return Err(invalid("symlink source input is unsupported"));
        }
        if metadata.is_dir() {
            collect(root, &path, files)?;
        } else if metadata.is_file() {
            files.insert(path.clone(), hash_file(&root.join(path))?);
        } else {
            return Err(invalid("non-regular source input is unsupported"));
        }
    }
    Ok(())
}
impl SourceFingerprint {
    /// Resolve Cargo's actual graph explicitly; no source-only read implicitly calls this.
    pub fn discover(
        root: &Path,
        environment: &Environment,
        context: BuildContext,
    ) -> Result<Self, HostError> {
        let mut metadata = MetadataCommand::new();
        metadata
            .cargo_path("cargo")
            .current_dir(root)
            .manifest_path(root.join("Cargo.toml"))
            .features(cargo_metadata::CargoOpt::SomeFeatures(
                context.features.iter().cloned().collect(),
            ))
            .other_options(vec!["--locked".into()]);
        let output = metadata
            .cargo_command()
            .env_clear()
            .envs(environment)
            .output()?;
        if !output.status.success() {
            return Err(invalid(
                "Cargo metadata failed while selecting host source inputs",
            ));
        }
        let metadata: Metadata = serde_json::from_slice(&output.stdout)
            .map_err(|_| invalid("Cargo metadata did not produce its versioned package graph"))?;
        Self::from_metadata(root, &metadata, context)
    }

    /// Capture the local normal/build dependency closure from an already captured Cargo graph.
    pub fn from_metadata(
        root: &Path,
        metadata: &Metadata,
        context: BuildContext,
    ) -> Result<Self, HostError> {
        let package = metadata
            .packages
            .iter()
            .find(|package| package.name == "pse-host")
            .ok_or_else(|| invalid("pse-host package is missing from metadata"))?;
        let graph = metadata
            .resolve
            .as_ref()
            .ok_or_else(|| invalid("resolved package graph is required"))?;
        let mut pending = vec![package.id.clone()];
        let mut seen = BTreeSet::<PackageId>::new();
        let mut package_roots = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !seen.insert(id.clone()) {
                continue;
            }
            let package = metadata
                .packages
                .iter()
                .find(|package| package.id == id)
                .ok_or_else(|| invalid("resolved package has no declaration"))?;
            if package.source.is_none() {
                let directory = package
                    .manifest_path
                    .parent()
                    .ok_or_else(|| invalid("package manifest has no parent"))?
                    .as_std_path();
                let selected = directory
                    .strip_prefix(root)
                    .map_err(|_| invalid("local host dependency lies outside the checkout"))?;
                if !relative(selected) {
                    return Err(invalid("local package root is not checkout-relative"));
                }
                package_roots.insert(selected.to_owned());
            }
            let node = graph
                .nodes
                .iter()
                .find(|node| node.id == id)
                .ok_or_else(|| invalid("resolved package has no graph node"))?;
            pending.extend(
                node.deps
                    .iter()
                    .filter(|dep| {
                        dep.dep_kinds
                            .iter()
                            .any(|kind| kind.kind != DependencyKind::Development)
                    })
                    .map(|dep| dep.pkg.clone()),
            );
        }
        let mut result = Self {
            version: VERSION,
            package_roots,
            additional_files: BTreeSet::new(),
            files: BTreeMap::new(),
            context,
        };
        result.files = result.observe_files(root)?;
        Ok(result)
    }

    /// Bind declared compile-time inputs outside package roots, such as `include_str!`
    /// files. The builder must supply the same set to both construction observations.
    pub fn with_additional_files(
        mut self,
        root: &Path,
        inputs: impl IntoIterator<Item = PathBuf>,
    ) -> Result<Self, HostError> {
        for input in inputs {
            if !relative(&input) {
                return Err(invalid("additional build input is not checkout-relative"));
            }
            self.additional_files.insert(input);
        }
        self.files = self.observe_files(root)?;
        Ok(self)
    }

    fn observe_files(&self, root: &Path) -> Result<BTreeMap<PathBuf, String>, HostError> {
        if self.version != VERSION
            || self.package_roots.iter().any(|path| !relative(path))
            || self.additional_files.iter().any(|path| !relative(path))
        {
            return Err(invalid(
                "unsupported fingerprint or invalid package-root path",
            ));
        }
        let mut files = BTreeMap::new();
        for input in REQUIRED_INPUTS {
            files.insert(
                PathBuf::from(input),
                hash_file(&checked_input(root, Path::new(input))?)?,
            );
        }
        for input in &self.additional_files {
            files.insert(input.clone(), hash_file(&checked_input(root, input)?)?);
        }
        for directory in &self.package_roots {
            collect(root, directory, &mut files)?;
        }
        Ok(files)
    }
    /// Stable digest of the complete source/context observation, without checkout-specific IDs.
    pub fn digest(&self) -> Result<String, HostError> {
        Ok(digest(&encoded(self)?))
    }
    /// Rehash only recorded package roots and required inputs; no Cargo invocation.
    /// Directory membership is re-read so added/deleted files invalidate reuse too.
    pub fn matches_sources(&self, root: &Path, context: &BuildContext) -> Result<bool, HostError> {
        if &self.context != context {
            return Ok(false);
        }
        match self.observe_files(root) {
            Ok(files) => Ok(self.files == files),
            Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
    /// Compare fresh pre/post construction observations, including a newly resolved closure.
    pub fn require_unchanged(&self, after: &Self) -> Result<(), HostError> {
        if self != after {
            return Err(invalid(
                "source/build context changed during construction; generation must not publish",
            ));
        }
        Ok(())
    }
}

/// Private completed generation manifest. Bootstrap owns the subsequent selector publication.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerationManifest {
    version: u32,
    control_protocol: u32,
    source: SourceFingerprint,
    binaries: BTreeMap<String, String>,
}
impl GenerationManifest {
    /// Capture completed binary bytes only when pre/post source observations agree.
    pub fn capture(
        directory: &Path,
        before: SourceFingerprint,
        after: &SourceFingerprint,
        binaries: &[&str],
    ) -> Result<Self, HostError> {
        before.require_unchanged(after)?;
        let mut result = BTreeMap::new();
        for name in binaries {
            if !component(name) || *name == MANIFEST || result.contains_key(*name) {
                return Err(invalid("invalid or duplicate binary name"));
            }
            let path = directory.join(name);
            if !fs::symlink_metadata(&path)?.is_file() {
                return Err(invalid("generation binary is not a regular file"));
            }
            result.insert((*name).to_owned(), hash_file(&path)?);
        }
        if result.is_empty() {
            return Err(invalid("generation has no binaries"));
        }
        Ok(Self {
            version: VERSION,
            control_protocol: 1,
            source: before,
            binaries: result,
        })
    }
    /// Persist exactly once in a private stage; this does not publish a current selector.
    pub fn write(&self, directory: &Path) -> Result<(), HostError> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(directory.join(MANIFEST))?;
        file.write_all(&encoded(self)?)?;
        file.sync_all()?;
        Ok(())
    }
    /// Authenticate prepared bytes under an already owned generation directory.
    pub fn load(directory: &Path) -> Result<Self, HostError> {
        private_directory(directory, false)?;
        let path = directory.join(MANIFEST);
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
            .open(path)?;
        validate_owned_file(&file)?;
        if file.metadata()?.len() > 16 * 1024 * 1024 {
            return Err(invalid("generation manifest exceeds its bound"));
        }
        let mut bytes = Vec::new();
        file.take(16 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
        let result: Self = serde_json::from_slice(&bytes)
            .map_err(|_| invalid("generation manifest is invalid"))?;
        if result.version != VERSION
            || result.control_protocol != 1
            || result.source.version != VERSION
            || result.binaries.is_empty()
        {
            return Err(invalid("unsupported or empty generation manifest"));
        }
        for (name, expected) in &result.binaries {
            if !component(name) || name == MANIFEST {
                return Err(invalid("generation binary path is invalid"));
            }
            if hash_file(&directory.join(name))? != *expected {
                return Err(invalid(
                    "prepared generation binary differs from its manifest",
                ));
            }
        }
        Ok(result)
    }
    /// Source observation for explicit freshness checks; control need not read source.
    pub fn source(&self) -> &SourceFingerprint {
        &self.source
    }
    /// Semantic identity binds source, build context, control protocol and binary bytes.
    pub fn digest(&self) -> Result<String, HostError> {
        Ok(digest(&encoded(self)?))
    }
    /// Existing-work control protocol supported by the authenticated tool set.
    pub fn control_protocol(&self) -> u32 {
        self.control_protocol
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UseRecord {
    version: u32,
    owner: pse_diagnostics::process_facts::ProcessIdentity,
    #[serde(default)]
    manifest_digest: String,
    #[serde(default)]
    handoff: Option<transfer::UseHandoff>,
}

/// An unsettled durable claim remains protective even after a controller crash.
#[derive(Debug)]
pub struct GenerationUse {
    lock: File,
    record: PathBuf,
    manifest: GenerationManifest,
}

/// Exact scoped inspection, never an instruction to remove a generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenerationUseState {
    /// At least one kernel lock currently protects a user.
    Active,
    /// Durable claims remain; process death/ledger absence does not prove child settlement.
    Unsettled,
    /// No scoped lock or durable claim was observed under the update exclusion.
    Available,
}

fn use_paths(root: &Path, generation: &str) -> Result<(PathBuf, PathBuf), HostError> {
    if !component(generation) {
        return Err(invalid("generation name is not a path component"));
    }
    private_directory(root, false)?;
    let uses = root.join(".uses");
    private_directory(&uses, true)?;
    let records = uses.join(generation);
    private_directory(&records, true)?;
    Ok((uses.join(format!("{generation}.lock")), records))
}
fn update_lock(root: &Path) -> Result<File, HostError> {
    private_directory(root, false)?;
    let file = owned_file(&root.join(".update.lock"), true)?;
    rustix::fs::flock(&file, rustix::fs::FlockOperation::LockShared)
        .map_err(std::io::Error::from)?;
    Ok(file)
}
impl GenerationUse {
    /// Acquire before handing a prepared generation to a child or another owner.
    /// The caller retains this owner until its actual child/fixture/finalizer obligations settle.
    pub fn acquire(root: &Path, generation: &str) -> Result<Self, HostError> {
        if !component(generation) {
            return Err(invalid("generation name is not a path component"));
        }
        let _update = update_lock(root)?;
        let manifest = GenerationManifest::load(&root.join(generation))?;
        let (lock, records) = use_paths(root, generation)?;
        let lock = owned_file(&lock, true)?;
        rustix::fs::flock(&lock, rustix::fs::FlockOperation::LockShared)
            .map_err(std::io::Error::from)?;
        let owner = match pse_diagnostics::process_facts::ProcessReader::default()
            .process(std::process::id())
        {
            pse_diagnostics::process_facts::ProcessRead::Present(facts) => facts.identity,
            _ => {
                return Err(invalid(
                    "current process identity cannot establish generation ownership",
                ));
            }
        };
        let record = records.join(format!("{}.json", uuid::Uuid::new_v4()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&record)?;
        file.write_all(&encoded(&UseRecord {
            version: VERSION,
            owner,
            manifest_digest: manifest.digest()?,
            handoff: None,
        })?)?;
        file.sync_all()?;
        File::open(&records)?.sync_all()?;
        Ok(Self {
            lock,
            record,
            manifest,
        })
    }
    /// Authenticated bytes protected by this actual-use owner.
    pub fn manifest(&self) -> &GenerationManifest {
        &self.manifest
    }
    /// Release only after the actual owner has established settlement, never on observation timeout.
    /// Dropping without this call intentionally leaves protective unsettled evidence.
    pub fn settle(self) -> Result<(), HostError> {
        fs::remove_file(&self.record)?;
        File::open(
            self.record
                .parent()
                .ok_or_else(|| invalid("claim has no parent"))?,
        )?
        .sync_all()?;
        drop(self.lock);
        Ok(())
    }
    /// Inspect only this generation's kernel lock and claim directory while excluding publication.
    pub fn inspect(root: &Path, generation: &str) -> Result<GenerationUseState, HostError> {
        if !component(generation) {
            return Err(invalid("generation name is not a path component"));
        }
        private_directory(root, false)?;
        private_directory(&root.join(generation), false)?;
        let update = root.join(".update.lock");
        let update = owned_file(&update, false)?;
        rustix::fs::flock(&update, rustix::fs::FlockOperation::LockShared)
            .map_err(std::io::Error::from)?;
        let uses = root.join(".uses");
        match fs::symlink_metadata(&uses) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(GenerationUseState::Available);
            }
            Err(error) => return Err(error.into()),
            Ok(_) => {}
        }
        private_directory(&uses, false)?;
        let records = uses.join(generation);
        let lock = match owned_file(&uses.join(format!("{generation}.lock")), false) {
            Ok(file) => file,
            Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(if fs::symlink_metadata(&records).is_ok() {
                    GenerationUseState::Unsettled
                } else {
                    GenerationUseState::Available
                });
            }
            Err(error) => return Err(error),
        };
        match rustix::fs::flock(&lock, rustix::fs::FlockOperation::NonBlockingLockExclusive) {
            Err(rustix::io::Errno::WOULDBLOCK) => return Ok(GenerationUseState::Active),
            Err(error) => return Err(std::io::Error::from(error).into()),
            Ok(()) => {}
        }
        private_directory(&records, false)?;
        if fs::read_dir(records)?.next().is_some() {
            Ok(GenerationUseState::Unsettled)
        } else {
            Ok(GenerationUseState::Available)
        }
    }
}

#[cfg(test)]
mod tests;
