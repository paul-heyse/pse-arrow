// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Same-compiler-invocation artifact capture, before forwarding its completion record.
use crate::correctness::{CorrectnessError, Resolution};
use cargo_metadata::{Artifact, Message};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fmt::Display,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn invalid(message: impl Into<String>) -> CorrectnessError {
    CorrectnessError::Invalid(message.into())
}

pub(super) fn private_directory(path: &Path) -> Result<(), CorrectnessError> {
    let metadata = fs::symlink_metadata(path)?;
    if !path.is_absolute()
        || !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o777 != 0o700
    {
        return Err(invalid(
            "artifact capture requires an owned absolute private directory",
        ));
    }
    Ok(())
}

/// Retained executable bytes, independent of its mutable Cargo output path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Executable {
    /// Original observed path; this path is never reused for execution.
    pub original: PathBuf,
    /// Private captured copy.
    pub captured: PathBuf,
    /// SHA-256 identity of the captured bytes.
    pub sha256: String,
    /// Original byte count.
    pub bytes: u64,
}
impl Executable {
    /// Authenticate retained bytes before lending their path to a child.
    pub fn verify(&self) -> Result<(), CorrectnessError> {
        let mut file = read_file(&self.captured)?;
        self.verify_file(&mut file).map(|_| ())
    }
    fn verify_file(&self, file: &mut File) -> Result<FileIdentity, CorrectnessError> {
        let before = FileIdentity::read(file)?;
        let (bytes, digest) = hash_copy(file, &mut io::sink())?;
        let after = FileIdentity::read(file)?;
        if before != after || bytes != self.bytes || digest != self.sha256 {
            return Err(invalid(
                "captured executable differs from its original identity",
            ));
        }
        Ok(after)
    }
}

/// One metadata traversal of immutable capture records. Successful content authentication
/// is shared only inside this borrowed owner; each reuse still checks the opened inode's
/// change identity. A missing, replaced or modified capture refuses rather than refreshing.
#[derive(Debug, Default)]
pub(super) struct Verification<'a> {
    files: BTreeMap<&'a Path, (&'a Executable, FileIdentity)>,
    #[cfg(test)]
    authentications: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileIdentity {
    device: u64,
    inode: u64,
    bytes: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}
impl FileIdentity {
    fn read(file: &File) -> Result<Self, CorrectnessError> {
        let metadata = file.metadata()?;
        Ok(Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            bytes: metadata.len(),
            modified: (metadata.mtime(), metadata.mtime_nsec()),
            changed: (metadata.ctime(), metadata.ctime_nsec()),
        })
    }
}
impl<'a> Verification<'a> {
    pub(super) fn verify(&mut self, executable: &'a Executable) -> Result<(), CorrectnessError> {
        let mut file = read_file(&executable.captured)?;
        if let Some((original, identity)) = self.files.get(executable.captured.as_path()) {
            if *original != executable || *identity != FileIdentity::read(&file)? {
                return Err(invalid(
                    "captured executable changed during inventory traversal",
                ));
            }
            return Ok(());
        }
        #[cfg(test)]
        {
            self.authentications += 1;
        }
        let identity = executable.verify_file(&mut file)?;
        self.files
            .insert(executable.captured.as_path(), (executable, identity));
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn authentications(&self) -> usize {
        self.authentications
    }
}
fn executable_record(path: &Path) -> Result<PathBuf, CorrectnessError> {
    let mut name = path
        .file_name()
        .ok_or_else(|| invalid("captured executable has no basename"))?
        .to_os_string();
    name.push(".capture.json");
    Ok(path.with_file_name(name))
}

/// Recover logical source location only from the capture owner's protected, exact-image
/// association. The original image may already have been replaced or removed.
pub(crate) fn original_executable(
    path: &Path,
    image: &mut impl Read,
) -> Result<Option<PathBuf>, CorrectnessError> {
    let record = match read_file(&executable_record(path)?) {
        Ok(record) => record,
        Err(CorrectnessError::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    private_directory(
        path.parent()
            .ok_or_else(|| invalid("captured executable has no parent"))?,
    )?;
    let metadata = record.metadata()?;
    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o777 != 0o400 {
        return Err(invalid("captured executable association is not protected"));
    }
    let captured: Executable = serde_json::from_reader(record)?;
    if captured.captured != path || !captured.original.is_absolute() {
        return Err(invalid(
            "captured executable association differs from the executing image",
        ));
    }
    captured.verify()?;
    let (bytes, sha256) = hash_copy(image, &mut io::sink())?;
    if bytes != captured.bytes || sha256 != captured.sha256 {
        return Err(invalid(
            "executing image differs from the captured executable association",
        ));
    }
    Ok(Some(captured.original))
}
fn read_file(path: &Path) -> Result<File, CorrectnessError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    if !file.metadata()?.is_file() {
        return Err(invalid("captured executable is not a regular file"));
    }
    Ok(file)
}
fn hash_copy(
    input: &mut impl Read,
    output: &mut impl Write,
) -> Result<(u64, String), CorrectnessError> {
    let mut digest = Sha256::new();
    let mut total = 0;
    let mut buffer = [0_u8; 65536];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        output.write_all(&buffer[..count])?;
        digest.update(&buffer[..count]);
        total += count as u64;
    }
    Ok((
        total,
        digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    ))
}

/// Capture through an open original handle and refuse a concurrently changed original.
pub(crate) fn executable(
    original: &Path,
    destination: &Path,
) -> Result<Executable, CorrectnessError> {
    if !original.is_absolute() {
        return Err(invalid("original executable path must be absolute"));
    }
    let mut input = read_file(original)?;
    let before = input.metadata()?;
    if before.mode() & 0o111 == 0 {
        return Err(invalid("original artifact is not executable"));
    }
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o500)
        .open(destination)?;
    let (bytes, sha256) = hash_copy(&mut input, &mut output)?;
    output.sync_all()?;
    let after = input.metadata()?;
    if before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.len() != after.len()
        || before.mtime() != after.mtime()
        || before.mtime_nsec() != after.mtime_nsec()
        || before.ctime() != after.ctime()
        || before.ctime_nsec() != after.ctime_nsec()
    {
        return Err(invalid("original executable changed during capture"));
    }
    let captured = Executable {
        original: original.into(),
        captured: destination.into(),
        sha256,
        bytes,
    };
    // Publish the existing byte/origin association before returning a path that can be
    // lent to a child, including controls needed before the final inventory exists.
    write_json(&executable_record(destination)?, &captured)?;
    Ok(captured)
}

/// Release one fixed executable of a physically settled workflow. The association
/// and both members must still belong to the original private capture.
pub(crate) fn release_executable(
    directory: &File,
    path: &Path,
) -> Result<(), CorrectnessError> {
    use rustix::fs::{Mode, OFlags, openat, unlinkat};

    let name = path
        .file_name()
        .ok_or_else(|| invalid("captured executable has no basename"))?;
    let record_path = executable_record(path)?;
    let record_name = record_path
        .file_name()
        .ok_or_else(|| invalid("executable association has no basename"))?;
    let open = |name: &std::ffi::OsStr| {
        openat(
            directory,
            name,
            OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK,
            Mode::empty(),
        )
        .map(File::from)
        .map_err(io::Error::from)
    };
    let record = match open(record_name) {
        Ok(record) => record,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return match open(name) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
                _ => Err(invalid("captured executable lacks its original association")),
            };
        }
        Err(error) => return Err(error.into()),
    };
    let protected = |file: &File, mode: u32| -> Result<(), CorrectnessError> {
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o777 != mode
            || metadata.nlink() != 1
        {
            return Err(invalid("executable capture member lacks private custody"));
        }
        Ok(())
    };
    protected(&record, 0o400)?;
    let before = FileIdentity::read(&record)?;
    let mut bytes = Vec::new();
    (&record).take(65_537).read_to_end(&mut bytes)?;
    if bytes.len() > 65_536 || bytes.len() as u64 != before.bytes {
        return Err(invalid("executable association exceeds its expected extent"));
    }
    let executable: Executable = serde_json::from_slice(&bytes)?;
    if executable.captured != path || !executable.original.is_absolute() {
        return Err(invalid("executable association differs from the release member"));
    }
    let mut image = match open(name) {
        Ok(image) => image,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if FileIdentity::read(&open(record_name)?)? != before {
                return Err(invalid("executable association changed before release"));
            }
            unlinkat(directory, record_name, rustix::fs::AtFlags::empty())
                .map_err(io::Error::from)?;
            directory.sync_all()?;
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    protected(&image, 0o500)?;
    let image_identity = executable.verify_file(&mut image)?;
    if FileIdentity::read(&open(name)?)? != image_identity
        || FileIdentity::read(&open(record_name)?)? != before
    {
        return Err(invalid("executable capture changed before release"));
    }
    unlinkat(directory, name, rustix::fs::AtFlags::empty()).map_err(io::Error::from)?;
    unlinkat(directory, record_name, rustix::fs::AtFlags::empty()).map_err(io::Error::from)?;
    directory.sync_all()?;
    Ok(())
}

/// Canonical private compiler-artifact association, preserving Cargo's original basename.
pub(crate) fn artifact_path(
    directory: &Path,
    index: usize,
    original: &Path,
) -> Result<PathBuf, CorrectnessError> {
    let name = original
        .file_name()
        .ok_or_else(|| invalid("Cargo executable has no basename"))?;
    Ok(directory.join(format!("artifact-{index}")).join(name))
}

/// An actual Cargo compiler-artifact record and its captured executable.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuiltArtifact {
    /// Cargo owns enabled features, target identity and profile interpretation.
    pub cargo: Artifact,
    /// Captured while consuming this compiler-artifact record.
    pub executable: Executable,
}

/// Same-invocation compiler selection and artifact evidence, including failed builds.
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildCapture {
    /// Version of this host-owned capture envelope, not an upstream format version.
    pub version: u32,
    /// The exact admitted Cargo invocation used for this build.
    pub resolution: Resolution,
    /// Actual executable artifacts, including non-test helpers.
    pub artifacts: Vec<BuiltArtifact>,
    /// Cargo child exit, independent of capture/forwarding success.
    pub cargo_exit: i32,
    /// At most one bounded first cause per domain; a zero Cargo exit does not clear these.
    pub incomplete: BTreeMap<CaptureFailureDomain, String>,
}

impl BuildCapture {
    /// Read a completed capture manifest from its original private directory.
    /// This checks file custody and extent, not correspondence to current working-source bytes.
    /// # Errors
    /// Unprotected, replaced, oversized or malformed input refuses without preparing anything.
    pub fn read(path: &Path) -> Result<Self, crate::HostError> {
        private_directory(
            path.parent()
                .ok_or_else(|| invalid("capture directory absent"))?,
        )?;
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(
                (rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32,
            )
            .open(path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o777 != 0o400
            || metadata.nlink() != 1
            || metadata.len() > 64 << 20
        {
            return Err(
                invalid("capture manifest requires bounded protected original bytes").into(),
            );
        }
        let before = crate::artifacts::descriptor_identity(&file)?;
        let mut bytes = Vec::new();
        (&file).take((64 << 20) + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 != metadata.len()
            || crate::artifacts::descriptor_identity(&file)? != before
        {
            return Err(invalid("capture manifest changed during acquisition").into());
        }
        Ok(serde_json::from_slice(&bytes).map_err(CorrectnessError::from)?)
    }

    /// Select one original non-test binary in its captured Cargo root/source context.
    /// The captured bytes are verified; a supplied manifest does not prove current source freshness.
    /// # Errors
    /// Incomplete capture, foreign root, missing features, ambiguous binaries or changed image.
    pub fn require_binary(
        &self,
        root: &Path,
        package: &str,
        name: &str,
        source: &str,
        required_features: &[&str],
    ) -> Result<Executable, CorrectnessError> {
        if self.version != 1 || self.cargo_exit != 0 || !self.incomplete.is_empty() {
            return Err(invalid("source build or original capture is incomplete"));
        }
        let mut selected = self.artifacts.iter().filter(|artifact| {
            artifact.cargo.target.name == name
                && !artifact.cargo.profile.test
                && artifact
                    .cargo
                    .target
                    .kind
                    .iter()
                    .any(|kind| kind.to_string() == "bin")
        });
        let artifact = selected
            .next()
            .ok_or_else(|| invalid("requested captured binary absent"))?;
        if selected.next().is_some() {
            return Err(invalid("capture must select exactly one requested binary"));
        }
        if artifact.cargo.target.src_path.as_std_path() != root.join(source)
            || !required_features.iter().all(|required| {
                artifact
                    .cargo
                    .features
                    .iter()
                    .any(|feature| feature == required)
            })
            || !self.resolution.units.iter().any(|unit| {
                unit.root
                    && unit.package_name == package
                    && unit.package_id == artifact.cargo.package_id.repr
                    && unit.target_name == name
                    && unit.mode == "build"
                    && unit.source_path == artifact.cargo.target.src_path.as_str()
            })
            || artifact
                .cargo
                .executable
                .as_ref()
                .is_none_or(|path| path.as_std_path() != artifact.executable.original)
        {
            return Err(invalid(
                "captured image differs from its actual native source/build context",
            ));
        }
        artifact.executable.verify()?;
        Ok(artifact.executable.clone())
    }
}

/// Independent capture domains; none establishes physical descendant settlement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureFailureDomain {
    /// Original pipe could not be drained completely.
    Read,
    /// Raw storage stopped accepting bytes.
    Storage,
    /// A required record could not be decoded within the supported record policy.
    Protocol,
    /// A required executable could not be captured.
    Artifact,
    /// Original byte forwarding stopped accepting bytes.
    Forwarding,
    /// Retained raw storage could not be synchronized.
    Synchronization,
    /// Capture manifest publication failed.
    Manifest,
    /// Requested child termination failed after a pipe read failure.
    Termination,
}

// This is a decoding contract for one Cargo JSON record, not a workload memory/CPU cap.
// Oversized records remain fully streamed to storage/forwarding and make capture incomplete.
const MAX_RECORD_BYTES: usize = 4 * 1024 * 1024;
const MAX_CAUSE_CHARS: usize = 2048;
type Failures = BTreeMap<CaptureFailureDomain, String>;

fn first_failure(failures: &mut Failures, domain: CaptureFailureDomain, cause: impl Display) {
    failures
        .entry(domain)
        .or_insert_with(|| cause.to_string().chars().take(MAX_CAUSE_CHARS).collect());
}

fn sink(
    bytes: &[u8],
    output: &mut impl Write,
    domain: CaptureFailureDomain,
    failures: &mut Failures,
) {
    if !failures.contains_key(&domain)
        && let Err(error) = output.write_all(bytes).and_then(|()| output.flush())
    {
        first_failure(failures, domain, error);
    }
}

fn record(
    bytes: &[u8],
    directory: &Path,
    artifacts: &mut Vec<BuiltArtifact>,
    failures: &mut Failures,
) {
    if failures.contains_key(&CaptureFailureDomain::Protocol)
        || failures.contains_key(&CaptureFailureDomain::Artifact)
    {
        return;
    }
    if bytes
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
        != Some(b'{')
    {
        return;
    }
    let value: serde_json::Value = match serde_json::from_slice(bytes) {
        Ok(value) => value,
        Err(error) => {
            first_failure(failures, CaptureFailureDomain::Protocol, error);
            return;
        }
    };
    if value.get("reason").and_then(serde_json::Value::as_str) != Some("compiler-artifact") {
        return;
    }
    match serde_json::from_value::<Message>(value) {
        Ok(Message::CompilerArtifact(artifact)) => {
            if let Some(path) = &artifact.executable {
                // Preserve Cargo's basename/argv[0] contract inside a distinct private
                // directory; native owners may legitimately require their program name.
                let captured = (|| {
                    let captured = artifact_path(directory, artifacts.len(), path.as_std_path())?;
                    let parent = captured
                        .parent()
                        .ok_or_else(|| invalid("Cargo artifact capture parent is absent"))?;
                    fs::DirBuilder::new().mode(0o700).create(parent)?;
                    executable(path.as_std_path(), &captured)
                })();
                match captured {
                    Ok(executable) => artifacts.push(BuiltArtifact {
                        cargo: artifact,
                        executable,
                    }),
                    Err(error) => first_failure(failures, CaptureFailureDomain::Artifact, error),
                }
            }
        }
        Ok(_) => first_failure(
            failures,
            CaptureFailureDomain::Protocol,
            "required compiler artifact decoded as another record",
        ),
        Err(error) => first_failure(failures, CaptureFailureDomain::Protocol, error),
    }
}

fn consume_frame(
    frame: crate::stream_records::Frame<'_>,
    forwarded: &mut impl Write,
    directory: &Path,
    artifacts: &mut Vec<BuiltArtifact>,
    failures: &mut Failures,
) {
    use crate::stream_records::Frame;
    match frame {
        Frame::Complete(bytes) => {
            record(bytes, directory, artifacts, failures);
            sink(bytes, forwarded, CaptureFailureDomain::Forwarding, failures);
        }
        Frame::Oversized { bytes, .. } => {
            first_failure(
                failures,
                CaptureFailureDomain::Protocol,
                "Cargo record exceeds the supported 4MiB decoding policy; raw stream retained independently",
            );
            sink(bytes, forwarded, CaptureFailureDomain::Forwarding, failures);
        }
        Frame::Truncated(bytes) => {
            first_failure(
                failures,
                CaptureFailureDomain::Protocol,
                "Cargo stream ended without a record delimiter; original bytes retained independently",
            );
            sink(bytes, forwarded, CaptureFailureDomain::Forwarding, failures);
        }
    }
}

fn drain(
    input: &mut impl Read,
    raw: &mut impl Write,
    forwarded: &mut impl Write,
    directory: &Path,
    artifacts: &mut Vec<BuiltArtifact>,
    failures: &mut Failures,
) -> io::Result<()> {
    use crate::stream_records::RecordFramer;
    let mut chunk = [0_u8; 65536];
    let limit = std::num::NonZeroUsize::new(MAX_RECORD_BYTES)
        .ok_or_else(|| io::Error::other("record decoding policy is zero"))?;
    let mut framer = RecordFramer::new(limit);
    loop {
        let count = match input.read(&mut chunk) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if count == 0 {
            break;
        }
        // Raw retention remains independent of record decoding and failed presentation sinks.
        sink(
            &chunk[..count],
            raw,
            CaptureFailureDomain::Storage,
            failures,
        );
        framer.push(&chunk[..count], |frame| {
            consume_frame(frame, forwarded, directory, artifacts, failures)
        });
    }
    framer.finish(|frame| consume_frame(frame, forwarded, directory, artifacts, failures));
    Ok(())
}

pub(super) fn write_json(path: &Path, value: &impl Serialize) -> Result<(), CorrectnessError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(path)?;
    serde_json::to_writer(&mut file, value)?;
    file.sync_all()?;
    Ok(())
}

/// Internal correctness callback: capture each executable before forwarding its Cargo record.
/// A capture or forwarding failure still drains stdout and waits for the original child.
pub(crate) fn execute_build(
    mut command: Command,
    resolution: &Resolution,
    directory: &Path,
    forwarded: &mut impl Write,
) -> Result<i32, CorrectnessError> {
    private_directory(directory)?;
    let mut stream = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(directory.join("cargo-artifacts.jsonl"))?;
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| invalid("Cargo stdout was not captured"))?;
    let mut failures = Failures::new();
    let mut artifacts = Vec::new();
    if let Err(error) = drain(
        &mut stdout,
        &mut stream,
        forwarded,
        directory,
        &mut artifacts,
        &mut failures,
    ) {
        first_failure(&mut failures, CaptureFailureDomain::Read, error);
        if let Err(error) = child.kill() {
            first_failure(&mut failures, CaptureFailureDomain::Termination, error);
        }
    }
    let status = child.wait()?;
    use std::os::unix::process::ExitStatusExt as _;
    let cargo_exit = status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(1));
    if let Err(error) = stream.sync_all() {
        first_failure(&mut failures, CaptureFailureDomain::Synchronization, error);
    }
    let captured = BuildCapture {
        version: 1,
        resolution: resolution.clone(),
        artifacts,
        cargo_exit,
        incomplete: failures.clone(),
    };
    if let Err(error) = write_json(&directory.join("build.json"), &captured) {
        first_failure(&mut failures, CaptureFailureDomain::Manifest, error);
    }
    if !failures.is_empty() {
        return Err(invalid(format!(
            "Cargo exited {cargo_exit}; artifact capture/forwarding failed: {}",
            failures
                .iter()
                .map(|(domain, cause)| format!("{domain:?}: {cause}"))
                .collect::<Vec<_>>()
                .join("; ")
        )));
    }
    Ok(cargo_exit)
}

#[cfg(test)]
mod tests;
