// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Build the pinned managed sccache source in private staging.
use super::*;
use std::io::Write as _;
use std::os::unix::fs::MetadataExt as _;

const PATCH: &str = "scripts/sccache-source/managed-disk.patch";
const RECEIPT: &str = ".pse-source-build.json";
const TARGET: &str = "x86_64-unknown-linux-gnu";

#[derive(Clone, Debug, Deserialize)]
pub(super) struct Spec {
    pub(super) revision: String,
    pub(super) patch_sha256: String,
}

impl Spec {
    pub(super) fn validate(&self) -> Result<(), HostError> {
        source_generation(&self.patch_sha256)?;
        if self.revision.len() != 40
            || !self
                .revision
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(invalid("invalid diagnostic source revision"));
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
struct Receipt {
    schema_version: u32,
    version: String,
    source: String,
    source_sha256: String,
    revision: String,
    patch_sha256: String,
    lock_sha256: String,
    binary_sha256: String,
    toolchain: String,
    target: String,
}

pub(super) fn patch(root: &Path, build: &Spec) -> Result<Vec<u8>, HostError> {
    let path = root.join(PATCH);
    no_symlinks(&path)?;
    let mut bytes = Vec::new();
    File::open(path)?.take(262145).read_to_end(&mut bytes)?;
    let observed: String = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if bytes.len() > 262144 || observed != build.patch_sha256 {
        return Err(invalid(
            "managed sccache patch differs from the declaration",
        ));
    }
    Ok(bytes)
}

pub(super) fn verify(target: &Path, spec: &ToolSpec, build: &Spec) -> Result<(), HostError> {
    let record = target.with_file_name(RECEIPT);
    no_symlinks(&record)?;
    let directory = target
        .parent()
        .ok_or_else(|| invalid("source generation missing"))?;
    let uid = rustix::process::geteuid().as_raw();
    for (path, mode) in [
        (directory, 0o700),
        (target, 0o500),
        (record.as_path(), 0o400),
    ] {
        let metadata = fs::metadata(path)?;
        if metadata.uid() != uid || metadata.permissions().mode() & 0o777 != mode {
            return Err(invalid(
                "managed sccache generation has unexpected ownership or permissions",
            ));
        }
    }
    let mut bytes = Vec::new();
    File::open(&record)?.take(16385).read_to_end(&mut bytes)?;
    if bytes.len() > 16384 {
        return Err(invalid("managed sccache build receipt exceeds allowance"));
    }
    let record: Receipt = serde_json::from_slice(&bytes)
        .map_err(|_| invalid("invalid managed sccache source build receipt"))?;
    if record.schema_version != 1
        || record.version != spec.version
        || record.source != spec.url
        || record.source_sha256 != spec.sha256
        || record.revision != build.revision
        || record.patch_sha256 != build.patch_sha256
        || !hash_valid(&record.lock_sha256)
        || record.toolchain.is_empty()
        || record.target != TARGET
        || record.binary_sha256 != digest(target)?
    {
        return Err(invalid(
            "managed sccache executable differs from its source build receipt",
        ));
    }
    Ok(())
}

fn extract(archive: &Path, parent: &Path, spec: &ToolSpec) -> Result<PathBuf, HostError> {
    if fs::metadata(archive)?.len() > spec.archive_limit()? || digest(archive)? != spec.sha256 {
        return Err(invalid(
            "managed sccache source archive differs from its pin",
        ));
    }
    let source = parent.join(format!("sccache-{}", spec.version));
    fs::create_dir(&source)?;
    let mut tar = tar::Archive::new(GzDecoder::new(File::open(archive)?).take(16 << 20));
    let mut seen = BTreeSet::new();
    let mut bytes = 0_u64;
    for entry in tar.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        let target = parent.join(&path);
        bytes = bytes.saturating_add(entry.size());
        if !relative(&path)
            || target == source
            || !target.starts_with(&source)
            || !entry.header().entry_type().is_file()
            || !seen.insert(path)
            || seen.len() > 1024
            || bytes > 12 << 20
        {
            return Err(invalid(
                "managed sccache source archive has an invalid entry",
            ));
        }
        let size = entry.size();
        fs::create_dir_all(
            target
                .parent()
                .ok_or_else(|| invalid("source parent missing"))?,
        )?;
        copy_bounded(&mut entry, &target, size, 12 << 20, false)?;
    }
    if tar.into_inner().limit() == 0 {
        return Err(invalid(
            "managed sccache source archive exceeds decoded allowance",
        ));
    }
    Ok(source)
}

pub(super) fn publish(
    root: &Path,
    archive: &Path,
    target: &Path,
    spec: &ToolSpec,
    patch: &[u8],
    scratch: &Path,
    environment: &Environment,
) -> Result<(), HostError> {
    let build = spec
        .source_build
        .as_ref()
        .ok_or_else(|| invalid("source build declaration missing"))?;
    let source = extract(archive, scratch, spec)?;
    let vcs: serde_json::Value =
        serde_json::from_reader(File::open(source.join(".cargo_vcs_info.json"))?)
            .map_err(|_| invalid("managed sccache source archive lacks commit identity"))?;
    if vcs.pointer("/git/sha1").and_then(serde_json::Value::as_str) != Some(&build.revision) {
        return Err(invalid(
            "managed sccache source commit differs from its pin",
        ));
    }
    let lock = source.join("Cargo.lock");
    let lock_sha256 = digest(&lock)?;
    let patch_file = scratch.join("selected.patch");
    fs::write(&patch_file, patch)?;
    let status = Command::new("git")
        .env_clear()
        .envs(environment)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .current_dir(&source)
        .args(["apply", "--whitespace=error"])
        .arg(&patch_file)
        .stdin(Stdio::null())
        .status()?;
    if !status.success() {
        return Err(invalid("managed sccache source patch did not apply"));
    }
    let toolchain_declaration: toml::Value =
        toml::from_str(&fs::read_to_string(root.join("rust-toolchain.toml"))?)
            .map_err(|_| invalid("invalid managed build toolchain declaration"))?;
    let declared = toolchain_declaration
        .get("toolchain")
        .and_then(|value| value.get("channel"))
        .and_then(toml::Value::as_str)
        .ok_or_else(|| invalid("missing managed build toolchain channel"))?;
    let command = |name| {
        let mut command = Command::new(name);
        command
            .current_dir(root)
            .env_clear()
            .envs(environment)
            .env_remove("RUSTUP_TOOLCHAIN");
        command
    };
    let active = command("rustup")
        .args(["show", "active-toolchain"])
        .output()?;
    let active = String::from_utf8_lossy(&active.stdout);
    let toolchain = active
        .split_whitespace()
        .next()
        .filter(|toolchain| {
            *toolchain == declared
                || toolchain
                    .strip_prefix(declared)
                    .is_some_and(|suffix| suffix.starts_with('-'))
        })
        .ok_or_else(|| invalid("managed sccache build requires the checkout's pinned toolchain"))?;
    let output = scratch.join("build");
    let mut cargo = command("cargo");
    for key in [
        "CARGO_TARGET_DIR",
        "CARGO_BUILD_TARGET_DIR",
        "CARGO_BUILD_BUILD_DIR",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTFLAGS",
    ] {
        cargo.env_remove(key);
    }
    let status = cargo
        .env("RUSTC_WRAPPER", "")
        .env("RUSTC_WORKSPACE_WRAPPER", "")
        .args([
            "build",
            "--locked",
            "--release",
            "--bin",
            "sccache",
            "--target",
            TARGET,
            "--target-dir",
        ])
        .arg(&output)
        .arg("--manifest-path")
        .arg(source.join("Cargo.toml"))
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() || digest(&lock)? != lock_sha256 {
        return Err(invalid(
            "managed sccache locked source build failed or changed its lock",
        ));
    }
    let pending = scratch.join("release");
    fs::create_dir(&pending)?;
    fs::set_permissions(&pending, fs::Permissions::from_mode(0o700))?;
    let binary = pending.join(&spec.member);
    let built = output.join(TARGET).join("release/sccache");
    copy_bounded(
        &mut File::open(&built)?,
        &binary,
        fs::metadata(&built)?.len(),
        spec.binary_limit()?,
        true,
    )?;
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o500))?;
    let mut record = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(pending.join(RECEIPT))?;
    serde_json::to_writer(
        &mut record,
        &Receipt {
            schema_version: 1,
            version: spec.version.clone(),
            source: spec.url.clone(),
            source_sha256: spec.sha256.clone(),
            revision: build.revision.clone(),
            patch_sha256: build.patch_sha256.clone(),
            lock_sha256,
            binary_sha256: digest(&binary)?,
            toolchain: toolchain.to_owned(),
            target: TARGET.to_owned(),
        },
    )
    .map_err(|_| invalid("cannot encode managed sccache source receipt"))?;
    record.write_all(b"\n")?;
    record.sync_all()?;
    File::open(&pending)?.sync_all()?;
    verify_at(&binary, spec)?;
    let destination = target
        .parent()
        .ok_or_else(|| invalid("tool generation missing"))?;
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        &pending,
        rustix::fs::CWD,
        destination,
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(std::io::Error::from)?;
    File::open(
        destination
            .parent()
            .ok_or_else(|| invalid("tool parent missing"))?,
    )?
    .sync_all()?;
    Ok(())
}
