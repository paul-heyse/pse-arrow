// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Same-compiler-invocation artifact capture, before forwarding its completion record.
use crate::correctness::{CorrectnessError, Resolution};
use cargo_metadata::{Artifact, Message};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufRead, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
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
pub(super) fn executable(
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
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| invalid("Cargo stdout was not captured"))?;
    let mut stdout = io::BufReader::new(stdout);
    let mut failures = Vec::new();
    let mut artifacts = Vec::new();
    let mut line = Vec::new();
    loop {
        line.clear();
        match stdout.read_until(b'\n', &mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(error) => {
                failures.push(error.to_string());
                let _ = child.kill();
                break;
            }
        }
        if let Err(error) = stream.write_all(&line) {
            failures.push(error.to_string());
        }
        for message in Message::parse_stream(io::Cursor::new(&line)) {
            if let Ok(Message::CompilerArtifact(artifact)) = message
                && let Some(path) = &artifact.executable
            {
                match executable(
                    path.as_std_path(),
                    &directory.join(format!("artifact-{}", artifacts.len())),
                ) {
                    Ok(executable) => artifacts.push(BuiltArtifact {
                        cargo: artifact,
                        executable,
                    }),
                    Err(error) => failures.push(error.to_string()),
                }
            }
        }
        if let Err(error) = forwarded.write_all(&line).and_then(|()| forwarded.flush()) {
            failures.push(error.to_string());
        }
    }
    let status = child.wait()?;
    use std::os::unix::process::ExitStatusExt as _;
    let cargo_exit = status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(1));
    let captured = BuildCapture {
        version: 1,
        resolution: resolution.clone(),
        artifacts,
        cargo_exit,
    };
    if let Err(error) = stream
        .sync_all()
        .map_err(CorrectnessError::from)
        .and_then(|()| write_json(&directory.join("build.json"), &captured))
    {
        failures.push(error.to_string());
    }
    if !failures.is_empty() {
        return Err(invalid(format!(
            "Cargo exited {cargo_exit}; artifact capture/forwarding failed: {}",
            failures.join("; ")
        )));
    }
    Ok(cargo_exit)
}

#[cfg(test)]
mod tests;
