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
        let (bytes, digest) = hash_copy(&mut file, &mut io::sink())?;
        if bytes != self.bytes || digest != self.sha256 {
            return Err(invalid(
                "captured executable differs from its original identity",
            ));
        }
        Ok(())
    }
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
    Ok(Executable {
        original: original.into(),
        captured: destination.into(),
        sha256,
        bytes,
    })
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
                    let name = path
                        .file_name()
                        .ok_or_else(|| invalid("Cargo executable has no basename"))?;
                    let directory = directory.join(format!("artifact-{}", artifacts.len()));
                    fs::DirBuilder::new().mode(0o700).create(&directory)?;
                    executable(path.as_std_path(), &directory.join(name))
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
