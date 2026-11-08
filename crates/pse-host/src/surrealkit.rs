// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit source preparation for the standalone gRPC schema test tool.

use crate::{
    HostError,
    diagnostic_tools::{digest, no_symlinks},
    environment::Environment,
    paths::{HostPaths, UserDirectory},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read as _, Write as _},
    os::unix::fs::{MetadataExt as _, OpenOptionsExt as _, PermissionsExt as _},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const PIN: &str = ".config/surrealkit.toml";
const LOCK_PATCH: &str = "scripts/surrealkit-source/grpc-lock.patch";
const BINARY: &str = "surrealkit";
const RECEIPT: &str = "release.json";
const MAX_RECEIPT: u64 = 16 << 10;

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
struct Pin {
    version: String,
    repository: String,
    source_revision: String,
    original_lock_sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Sdk {
    version: String,
    default_features: bool,
    features: Vec<String>,
}

struct Selection {
    pin: Pin,
    sdk: Sdk,
    inputs: String,
    release: PathBuf,
}

#[derive(Deserialize, Serialize)]
struct Receipt {
    #[serde(flatten)]
    pin: Pin,
    source_commit: String,
    source_build: bool,
    release_build_locked: bool,
    input_sha256: String,
    lock_sha256: String,
    binary_sha256: String,
}

fn invalid(message: impl Into<String>) -> HostError {
    HostError::Ownership(message.into())
}

fn hexadecimal(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn release_member(path: &Path, mode: u32) -> Result<File, HostError> {
    no_symlinks(path)?;
    let file = crate::test_inventory::open_private(path)?;
    let metadata = file.metadata()?;
    if metadata.permissions().mode() & 0o7777 != mode || metadata.nlink() != 1 {
        return Err(invalid(
            "SurrealKit release requires protected, singly linked files",
        ));
    }
    Ok(file)
}

fn opened_digest(mut file: File) -> Result<String, HostError> {
    let mut hash = Sha256::new();
    let mut bytes = [0_u8; 65536];
    loop {
        let count = file.read(&mut bytes)?;
        if count == 0 {
            break;
        }
        hash.update(&bytes[..count]);
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

impl Selection {
    fn read(root: &Path, environment: &Environment) -> Result<Self, HostError> {
        let pin: Pin = toml::from_str(&fs::read_to_string(root.join(PIN))?)
            .map_err(|_| invalid("invalid SurrealKit pin"))?;
        if !pin
            .version
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b))
            || pin.version.is_empty()
            || pin.repository != "surrealdb/surrealkit"
            || !hexadecimal(&pin.source_revision, 40)
            || !hexadecimal(&pin.original_lock_sha256, 64)
        {
            return Err(invalid("invalid SurrealKit source identity"));
        }
        let workspace: toml::Value = toml::from_str(&fs::read_to_string(root.join("Cargo.toml"))?)
            .map_err(|_| invalid("invalid workspace manifest"))?;
        let sdk: Sdk = workspace
            .get("workspace")
            .and_then(|value| value.get("metadata"))
            .and_then(|value| value.get("pse"))
            .and_then(|value| value.get("tool-dependencies"))
            .and_then(|value| value.get("surrealdb"))
            .ok_or_else(|| invalid("SurrealKit requires the exact standalone SDK pin"))?
            .clone()
            .try_into()
            .map_err(|_| invalid("invalid standalone SDK pin"))?;
        if sdk.default_features
            || !sdk.version.starts_with('=')
            || sdk.features != ["protocol-grpc", "rustls"]
        {
            return Err(invalid("SurrealKit requires the pinned remote gRPC SDK"));
        }
        let mut identity = Sha256::new();
        for input in [PIN, LOCK_PATCH, "rust-toolchain.toml", ".cargo/config.toml"] {
            identity.update(input.as_bytes());
            identity.update(digest(&root.join(input))?.as_bytes());
        }
        // Include only the SDK selection from the workspace manifest.
        identity.update(sdk.version.as_bytes());
        for feature in &sdk.features {
            identity.update(feature.as_bytes());
            identity.update([0]);
        }
        identity.update(b"surrealkit-grpc-locked-v2");
        let inputs = identity
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let release = HostPaths::new(environment)
            .base(UserDirectory::Data)?
            .join("pse-arrow/tools/surrealkit")
            .join(format!("v{}", pin.version))
            .join(format!("grpc-{}", std::env::consts::ARCH))
            .join("prepared")
            .join(&inputs);
        Ok(Self {
            pin,
            sdk,
            inputs,
            release,
        })
    }

    fn verify_at(&self, binary: &Path) -> Result<(), HostError> {
        let record = binary.with_file_name(RECEIPT);
        let record = release_member(&record, 0o400)?;
        if record.metadata()?.len() > MAX_RECEIPT {
            return Err(invalid(
                "SurrealKit release receipt exceeds its admitted extent",
            ));
        }
        let receipt: Receipt = serde_json::from_reader(record.take(MAX_RECEIPT + 1))
            .map_err(|_| invalid("invalid SurrealKit release receipt"))?;
        if receipt.pin != self.pin
            || receipt.source_commit != self.pin.source_revision
            || !receipt.source_build
            || !receipt.release_build_locked
            || receipt.input_sha256 != self.inputs
            || receipt.lock_sha256
                != opened_digest(release_member(&binary.with_file_name("Cargo.lock"), 0o400)?)?
            || receipt.binary_sha256 != opened_digest(release_member(binary, 0o500)?)?
        {
            return Err(invalid(
                "SurrealKit release does not match prepared source and executable",
            ));
        }
        Ok(())
    }
}

/// Verify the selected source release without preparing or executing anything.
/// # Errors
/// The selected release is absent or its pin, protected receipt, lock, or executable differs.
pub fn verify(root: &Path, environment: &Environment) -> Result<PathBuf, HostError> {
    let selected = Selection::read(root, environment)?;
    let binary = HostPaths::new(environment)
        .selected("PSE_SURREALKIT_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| selected.release.join(BINARY));
    selected.verify_at(&binary).map_err(|error| {
        invalid(format!("SurrealKit release {} is unavailable: {error}; explicitly run scripts/pse-env -- scripts/pse diagnostic-tools install surrealkit", binary.display()))
    })?;
    Ok(binary)
}

fn replace_once(path: &Path, old: &str, new: &str) -> Result<(), HostError> {
    let original = fs::read_to_string(path)?;
    if original.matches(old).count() != 1 {
        return Err(invalid("SurrealKit gRPC source integration changed"));
    }
    fs::write(path, original.replacen(old, new, 1))?;
    Ok(())
}

fn build_sources(source: &Path, sdk: &Sdk) -> Result<(), HostError> {
    let manifest = source.join("Cargo.toml");
    let features =
        serde_json::to_string(&sdk.features).map_err(|_| invalid("invalid SDK features"))?;
    replace_once(
        &manifest,
        "surrealdb = { version = '4.0.0', features = ['protocol-http', 'jwks'] }",
        &format!(
            "surrealdb = {{ version = {:?}, default-features = false, features = {features} }}",
            sdk.version
        ),
    )?;
    replace_once(
        &manifest,
        "surrealdb-types = { version = '4.0.0' }",
        &format!("surrealdb-types = {{ version = {:?} }}", sdk.version),
    )?;
    replace_once(&manifest, "panic = 'abort'", "panic = 'unwind'")?;
    Ok(())
}

fn run(command: &mut Command, stage: &'static str) -> Result<(), HostError> {
    let status = command.stdin(Stdio::null()).status()?;
    if !status.success() {
        return Err(invalid(format!("SurrealKit {stage} failed with {status}")));
    }
    Ok(())
}

/// Build and atomically publish the exact locked gRPC tool in a private generation.
/// # Errors
/// The source, build inputs, locked compilation or immutable release cannot be verified.
pub fn install(root: &Path, environment: &Environment) -> Result<PathBuf, HostError> {
    let selected = Selection::read(root, environment)?;
    let parent = selected
        .release
        .parent()
        .ok_or_else(|| invalid("SurrealKit release has no generation parent"))?;
    no_symlinks(parent)?;
    fs::create_dir_all(parent)?;
    no_symlinks(parent)?;
    let directory = File::open(parent)?;
    rustix::fs::flock(&directory, rustix::fs::FlockOperation::LockExclusive)
        .map_err(std::io::Error::from)?;
    let binary = selected.release.join(BINARY);
    if selected.release.try_exists()? {
        selected.verify_at(&binary)?;
        return Ok(binary);
    }
    let stage = tempfile::Builder::new()
        .prefix(".surrealkit-build-")
        .tempdir_in(parent)?;
    let source = stage.path().join("source");
    fs::create_dir(&source)?;
    let git = || {
        let mut command = Command::new("git");
        command
            .current_dir(&source)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE");
        command
    };
    run(git().args(["init", "--quiet"]), "source initialization")?;
    run(
        git().args([
            "fetch",
            "--quiet",
            "--depth=1",
            &format!("https://github.com/{}.git", selected.pin.repository),
            &selected.pin.source_revision,
        ]),
        "source acquisition",
    )?;
    run(
        git().args([
            "checkout",
            "--quiet",
            "--detach",
            &selected.pin.source_revision,
        ]),
        "source checkout",
    )?;
    let revision = git().args(["rev-parse", "HEAD"]).output()?;
    let source_commit = std::str::from_utf8(&revision.stdout)
        .map_err(|_| invalid("SurrealKit checkout revision is not UTF-8"))?
        .trim();
    let lock = source.join("Cargo.lock");
    if !revision.status.success()
        || source_commit != selected.pin.source_revision
        || digest(&lock)? != selected.pin.original_lock_sha256
    {
        return Err(invalid(
            "SurrealKit checkout or original lock differs from the pinned source",
        ));
    }
    run(
        git().args(["apply", "--check"]).arg(root.join(LOCK_PATCH)),
        "locked SDK integration preflight",
    )?;
    run(
        git().arg("apply").arg(root.join(LOCK_PATCH)),
        "locked SDK integration",
    )?;
    build_sources(&source, &selected.sdk)?;
    let lock_sha256 = digest(&lock)?;
    let mut cargo = Command::new("cargo");
    cargo.current_dir(root).env_clear().envs(environment);
    for variable in [
        "CARGO_TARGET_DIR",
        "CARGO_BUILD_TARGET_DIR",
        "CARGO_BUILD_BUILD_DIR",
        "CARGO_BUILD_TARGET",
    ] {
        cargo.env_remove(variable);
    }
    run(
        cargo
            .args([
                "build",
                "--locked",
                "--release",
                "--no-default-features",
                "--features",
                "cli",
                "-p",
                BINARY,
                "--manifest-path",
            ])
            .arg(source.join("Cargo.toml")),
        "locked release build",
    )?;
    if lock_sha256 != digest(&lock)?
        || selected.inputs != Selection::read(root, environment)?.inputs
    {
        return Err(invalid(
            "SurrealKit preparation inputs changed during the locked build",
        ));
    }
    let pending = stage.path().join("release");
    fs::create_dir(&pending)?;
    fs::set_permissions(&pending, fs::Permissions::from_mode(0o700))?;
    let image = pending.join(BINARY);
    fs::copy(source.join("target/release").join(BINARY), &image)?;
    fs::set_permissions(&image, fs::Permissions::from_mode(0o500))?;
    File::open(&image)?.sync_all()?;
    let published_lock = pending.join("Cargo.lock");
    fs::copy(lock, &published_lock)?;
    fs::set_permissions(&published_lock, fs::Permissions::from_mode(0o400))?;
    File::open(&published_lock)?.sync_all()?;
    let mut record = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(pending.join(RECEIPT))?;
    let receipt = Receipt {
        source_commit: source_commit.to_owned(),
        pin: selected.pin.clone(),
        source_build: true,
        release_build_locked: true,
        input_sha256: selected.inputs.clone(),
        lock_sha256,
        binary_sha256: digest(&image)?,
    };
    serde_json::to_writer(&mut record, &receipt)
        .map_err(|_| invalid("invalid SurrealKit publication receipt"))?;
    record.write_all(b"\n")?;
    record.sync_all()?;
    selected.verify_at(&image)?;
    File::open(&pending)?.sync_all()?;
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        &pending,
        rustix::fs::CWD,
        &selected.release,
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(std::io::Error::from)?;
    directory
        .sync_all()
        .map_err(|source| HostError::PublicationDurabilityUnknown {
            destination: selected.release,
            source,
        })?;
    Ok(binary)
}
