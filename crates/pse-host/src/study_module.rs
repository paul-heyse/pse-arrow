// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit construction and source-free selection of immutable study modules.
//!
//! The prepared released server CLI owns WASI compilation, optimization, signature
//! extraction and packaging. This owner records provenance and publishes bytes;
//! operations owns admission into an actual database. No Python constructor runs.

use crate::{HostError, HostManager, environment::Environment, server::PreparedServer};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{OsStr, OsString},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

const CRATE: &str = "crates/pse-study-module";
const TARGET: &str = "wasm32-wasip2";
const MAX_ARCHIVE: u64 = 32 * 1024 * 1024;
const MAX_MANIFEST: u64 = 64 * 1024;
const MAX_RECORD: u64 = 8 * 1024 * 1024;
const MEMBERS: [&str; 3] = [
    "surrealism/mod.wasm",
    "surrealism/surrealism.toml",
    "surrealism/exports.toml",
];
const PREAMBLE: &[u8] = b"\0asm\x0d\0\x01\0";
const BUILD_ENV: &[&str] = &[
    "HOME",
    "PATH",
    "USER",
    "LOGNAME",
    "TMPDIR",
    "LANG",
    "LC_ALL",
    "TZ",
    "CARGO_HOME",
    "RUSTUP_HOME",
    "CARGO_BUILD_JOBS",
    "CARGO_INCREMENTAL",
    "CARGO_TERM_COLOR",
    "RUST_MIN_STACK",
    "RUSTC_WRAPPER",
    "RUSTFLAGS",
    "RUSTDOCFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_ENCODED_RUSTDOCFLAGS",
    "CC",
    "CXX",
    "AR",
    "RANLIB",
    "LIBCLANG_PATH",
    "CLANG_PATH",
    "LLVM_CONFIG_PATH",
    "LLVM_DIR",
    "LD_LIBRARY_PATH",
    "LIBRARY_PATH",
    "PKG_CONFIG_PATH",
    "CMAKE_PREFIX_PATH",
    "SCCACHE_DIR",
    "SCCACHE_CACHE_SIZE",
    "SCCACHE_MAX_FRAME_LENGTH",
    "SCCACHE_CONF",
    "SCCACHE_SERVER_UDS",
    "SCCACHE_CLIENT_SIDE",
    "SCCACHE_DIRECT",
    "SCCACHE_NO_DAEMON",
    "SOURCE_DATE_EPOCH",
];

/// Qualification identities are never admissible as production modules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    /// Sole readonly `study::readiness` export.
    Production,
    /// Retained rollback, timeout, memory and trap controls.
    Qualification,
    /// Retained six-second saturation controls; new construction needs CLI support.
    LongHold,
}
impl Variant {
    fn kind(self) -> &'static str {
        match self {
            Self::Production => "pse-study-module-v1",
            Self::Qualification => "pse-study-module-qualification-v1",
            Self::LongHold => "pse-study-module-qualification-long-hold-v1",
        }
    }
    fn features(self) -> Vec<&'static str> {
        match self {
            Self::Production => vec![],
            Self::Qualification => vec!["module-qualification"],
            Self::LongHold => vec!["module-qualification-long-hold"],
        }
    }
    fn exports(self) -> BTreeSet<&'static str> {
        let mut exports = BTreeSet::from(["study::readiness"]);
        if self != Self::Production {
            exports.extend([
                "qualification::busy",
                "qualification::trap",
                "qualification::memory_refused",
                "qualification::hold",
            ]);
        }
        exports
    }
    fn constructible(self) -> Result<(), HostError> {
        if self == Self::LongHold {
            return Err(invalid(
                "long-hold preparation requires an official CLI configuration override; retained long-hold receipts remain selectable; Python repackaging is retired",
            ));
        }
        Ok(())
    }
}

pub use pse_buildinfo::study_module::{ModuleLimits, StudyModuleRecord};
type Limits = ModuleLimits;
/// Module capability minted by explicit preparation or retained-artifact admission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct StudyModuleReceipt(StudyModuleRecord);
impl std::ops::Deref for StudyModuleReceipt {
    type Target = StudyModuleRecord;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl StudyModuleReceipt {
    /// Transfer the original typed record to its canonical admission owner.
    pub fn into_record(self) -> StudyModuleRecord {
        self.0
    }
    /// Exact prepared package; retaining this record does not install it in a database.
    pub fn artifact(&self) -> &Path {
        &self.artifact
    }
    /// Adjacent receipt for the explicit operations/fixture handoff.
    pub fn record(&self) -> PathBuf {
        self.artifact.with_file_name("module.json")
    }
}

fn invalid(message: impl std::fmt::Display) -> HostError {
    HostError::Ownership(format!("study module: {message}"))
}
fn digest_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn digest(path: &Path) -> Result<String, HostError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let length = file.read(&mut buffer)?;
        if length == 0 {
            break;
        }
        hasher.update(&buffer[..length]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
fn hexadecimal(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
fn exact_path(path: &Path) -> Result<(), HostError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::RootDir | Component::Normal(_)))
    {
        return Err(invalid("paths must be absolute without traversal"));
    }
    for ancestor in path.ancestors() {
        if fs::symlink_metadata(ancestor)?.file_type().is_symlink() {
            return Err(invalid("module paths must not traverse symlinks"));
        }
    }
    Ok(())
}
fn private_directory(path: &Path) -> Result<(), HostError> {
    // DirBuilder's mode applies only to newly created directories; never chmod a
    // pre-existing cache or legacy artifact to force it into this contract.
    let mut ancestor = path;
    while !ancestor.exists() {
        ancestor = ancestor
            .parent()
            .ok_or_else(|| invalid("directory has no parent"))?;
    }
    exact_path(ancestor)?;
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)?;
    exact_path(path)?;
    let status = fs::metadata(path)?;
    if !status.is_dir()
        || status.uid() != rustix::process::geteuid().as_raw()
        || status.mode() & 0o7777 != 0o700
    {
        return Err(invalid(
            "module directory must already be private and owned",
        ));
    }
    Ok(())
}

// Match the retained Python reader's sorted, compact, ensure_ascii JSON identity,
// including non-ASCII source paths. No current source is consulted during selection.
fn canonical(value: &Value) -> Result<String, HostError> {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<_, _> = map.iter().collect();
            let mut fields = Vec::new();
            for (key, value) in sorted {
                fields.push(format!(
                    "{}:{}",
                    canonical(&Value::String(key.clone()))?,
                    canonical(value)?
                ));
            }
            Ok(format!("{{{}}}", fields.join(",")))
        }
        Value::Array(items) => Ok(format!(
            "[{}]",
            items
                .iter()
                .map(canonical)
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        )),
        Value::String(_) => {
            let encoded = serde_json::to_string(value).map_err(invalid)?;
            let mut ascii = String::new();
            for character in encoded.chars() {
                if character.is_ascii() {
                    ascii.push(character);
                } else {
                    for unit in character.encode_utf16(&mut [0; 2]) {
                        use std::fmt::Write as _;
                        write!(ascii, "\\u{unit:04x}").map_err(invalid)?;
                    }
                }
            }
            Ok(ascii)
        }
        _ => serde_json::to_string(value).map_err(invalid),
    }
}
fn input_identity(inputs: &impl Serialize) -> Result<String, HostError> {
    Ok(digest_bytes(
        canonical(&serde_json::to_value(inputs).map_err(invalid)?)?.as_bytes(),
    ))
}
fn declaration(variant: Variant) -> Result<(Value, Limits, String), HostError> {
    // Embedded declarations authenticate retained packages without reading checkout sources.
    let source = if variant == Variant::LongHold {
        include_str!("../../pse-study-module/surrealism-qualification-long-hold.toml")
    } else {
        include_str!("../../pse-study-module/surrealism.toml")
    };
    let config: toml::Value = toml::from_str(source).map_err(invalid)?;
    let config = serde_json::to_value(config).map_err(invalid)?;
    validate_pure(&config)?;
    let mut capabilities = config["capabilities"]
        .as_object()
        .ok_or_else(|| invalid("capabilities absent"))?
        .clone();
    for name in [
        "allow_scripting",
        "allow_arbitrary_queries",
        "allow_functions",
        "allow_net",
    ] {
        capabilities.remove(name);
    }
    let time = capabilities
        .remove("max_execution_time")
        .ok_or_else(|| invalid("timeout absent"))?;
    capabilities.insert("max_execution_time_ms".into(), time);
    let limits: Limits = serde_json::from_value(Value::Object(capabilities)).map_err(invalid)?;
    if limits.max_memory_bytes == 0
        || limits.max_pool_size == 0
        || limits.max_execution_time_ms == 0
        || !limits.strict_timeout
    {
        return Err(invalid("finite strict module limits required"));
    }
    let workspace: toml::Value =
        toml::from_str(include_str!("../../../Cargo.toml")).map_err(invalid)?;
    let sdk = workspace
        .get("workspace")
        .and_then(|value| value.get("dependencies"))
        .and_then(|value| value.get("surrealism"))
        .and_then(toml::Value::as_str)
        .and_then(|value| value.strip_prefix('='))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid("exact SDK pin absent"))?
        .to_owned();
    Ok((config, limits, sdk))
}
fn validate_pure(config: &Value) -> Result<(), HostError> {
    if config["target"] != "rust"
        || config["abi"] != 2
        || config["capabilities"]["allow_scripting"] != false
        || config["capabilities"]["allow_arbitrary_queries"] != false
        || config["capabilities"]["allow_functions"] != json!([])
        || config["capabilities"]["allow_net"] != json!([])
        || config["capabilities"]["max_kv_entries"] != 0
        || config["capabilities"]["max_kv_value_bytes"] != 0
        || config
            .get("attach")
            .is_some_and(|value| value != &json!({}))
    {
        return Err(invalid("module must have no host grants or attachment"));
    }
    Ok(())
}
fn validate_exports(source: &[u8], variant: Variant) -> Result<(), HostError> {
    let manifest: toml::Value =
        toml::from_str(std::str::from_utf8(source).map_err(invalid)?).map_err(invalid)?;
    let functions = manifest
        .get("functions")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| invalid("exports absent"))?;
    let mut names = BTreeSet::new();
    for item in functions {
        let name = item
            .get("name")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| invalid("export name absent"))?;
        if item.get("writeable").and_then(toml::Value::as_bool) != Some(false)
            || !names.insert(name)
        {
            return Err(invalid("exports must be unique and readonly"));
        }
    }
    if names != variant.exports() {
        return Err(invalid("exports differ from selected variant"));
    }
    Ok(())
}

fn bounded_read(path: &Path, maximum: u64) -> Result<Vec<u8>, HostError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > maximum {
        return Err(invalid("record or package exceeds admitted extent"));
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(maximum + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 != metadata.len() {
        return Err(invalid("file extent changed while reading"));
    }
    Ok(bytes)
}
/// Authenticate a retained receipt and immutable bytes. Never invokes tools or hashes sources.
pub fn select(record: &Path, variant: Variant) -> Result<StudyModuleReceipt, HostError> {
    exact_path(record)?;
    let receipt = StudyModuleReceipt(
        serde_json::from_slice(&bounded_read(record, MAX_RECORD)?).map_err(invalid)?,
    );
    validate_receipt(&receipt, variant)?;
    if receipt.record() != record {
        return Err(invalid("receipt is not adjacent to its artifact"));
    }
    Ok(receipt)
}
fn validate_receipt(receipt: &StudyModuleReceipt, variant: Variant) -> Result<(), HostError> {
    let (_, limits, sdk) = declaration(variant)?;
    if receipt.kind != variant.kind()
        || receipt.abi != 2
        || receipt.sdk_version != sdk
        || receipt.limits != limits
        || receipt.inputs.get("kind") != Some(&json!(variant.kind()))
        || receipt.inputs.get("features") != Some(&json!(variant.features()))
        || receipt.inputs.get("sdk_version") != Some(&json!(sdk))
        || receipt.inputs.get("target") != Some(&json!(TARGET))
        || receipt.inputs.get("profile") != Some(&json!("release"))
        || !receipt
            .inputs
            .get("builder_sha256")
            .and_then(Value::as_str)
            .is_some_and(hexadecimal)
        || !hexadecimal(&receipt.sha256)
        || receipt.bytes == 0
        || receipt.bytes > MAX_ARCHIVE
    {
        return Err(invalid(
            "receipt identity, variant or resource contract differs",
        ));
    }
    let sources = receipt
        .inputs
        .get("sources")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid("source provenance absent"))?;
    if sources.is_empty()
        || sources
            .values()
            .any(|value| !value.as_str().is_some_and(hexadecimal))
    {
        return Err(invalid("source provenance requires SHA256 identities"));
    }
    if variant == Variant::LongHold {
        let packaging = receipt
            .inputs
            .get("packaging")
            .ok_or_else(|| invalid("long-hold provenance absent"))?;
        if packaging.as_object().is_none_or(|fields| fields.len() != 3)
            || packaging["variant"] != "qualification-long-hold-v1"
            || !packaging["packer_sha256"].as_str().is_some_and(hexadecimal)
            || !packaging["manifest_sha256"]
                .as_str()
                .is_some_and(hexadecimal)
            || packaging["manifest_sha256"]
                != sources
                    .get(&format!("{CRATE}/surrealism-qualification-long-hold.toml"))
                    .cloned()
                    .unwrap_or(Value::Null)
        {
            return Err(invalid("long-hold packaging identity differs"));
        }
    }
    exact_path(&receipt.artifact)?;
    let parent = receipt
        .artifact
        .parent()
        .ok_or_else(|| invalid("artifact parent absent"))?;
    if receipt.artifact.file_name() != Some(OsStr::new("study.surli"))
        || parent.file_name().and_then(OsStr::to_str)
            != Some(input_identity(&receipt.inputs)?.as_str())
        || parent.to_str().is_none_or(|value| value.contains(':'))
    {
        return Err(invalid(
            "artifact must belong to its immutable input directory",
        ));
    }
    let status = fs::symlink_metadata(&receipt.artifact)?;
    let directory = fs::metadata(parent)?;
    let owner = rustix::process::geteuid().as_raw();
    if !status.is_file()
        || status.uid() != owner
        || status.nlink() != 1
        || status.mode() & 0o7777 != 0o400
        || status.len() != receipt.bytes
        || !directory.is_dir()
        || directory.uid() != owner
        || directory.mode() & 0o7777 != 0o700
    {
        return Err(invalid(
            "artifact ownership, privacy, immutability or extent differs",
        ));
    }
    let bytes = bounded_read(&receipt.artifact, MAX_ARCHIVE)?;
    if bytes.len() as u64 != receipt.bytes || digest_bytes(&bytes) != receipt.sha256 {
        return Err(invalid("artifact differs from its admitted SHA256"));
    }
    Ok(())
}

fn environment(caller: &Environment) -> Result<Environment, HostError> {
    for name in caller.keys().filter_map(|name| name.to_str()) {
        if ((name.starts_with("CARGO_") || name.starts_with("RUST"))
            && !BUILD_ENV.contains(&name)
            && !name.starts_with("CARGO_PKG_")
            && !name.starts_with("CARGO_MANIFEST_"))
            && caller
                .get(OsStr::new(name))
                .is_some_and(|value| !value.is_empty())
        {
            return Err(invalid(format!("unsupported module build control {name}")));
        }
    }
    if caller
        .get(OsStr::new("RUSTC_WRAPPER"))
        .is_some_and(|value| {
            !value.is_empty() && Path::new(value).file_name() != Some(OsStr::new("sccache"))
        })
    {
        return Err(invalid(
            "module preparation requires the native sccache wrapper or direct compilation",
        ));
    }
    let mut selected: Environment = caller
        .iter()
        .filter(|(name, _)| name.to_str().is_some_and(|name| BUILD_ENV.contains(&name)))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect();
    selected.insert("LC_ALL".into(), "C".into());
    Ok(selected)
}
fn resolve(name: &str, env: &Environment) -> Result<PathBuf, HostError> {
    let path = env
        .get(OsStr::new("PATH"))
        .ok_or_else(|| invalid("PATH absent"))?;
    for directory in std::env::split_paths(path) {
        let candidate = directory.join(name);
        if candidate.is_file() && fs::metadata(&candidate)?.mode() & 0o111 != 0 {
            // Keep the rustup proxy's argv[0]; canonicalizing cargo to rustup
            // would change the selected program's meaning.
            if candidate.is_absolute() {
                return Ok(candidate);
            }
        }
    }
    Err(invalid(format!("required tool {name} is unavailable")))
}
fn command(
    program: &Path,
    arguments: &[OsString],
    root: &Path,
    env: &Environment,
    maximum: u64,
) -> Result<Vec<u8>, HostError> {
    let mut child = Command::new(program)
        .args(arguments)
        .current_dir(root)
        .env_clear()
        .envs(env)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| invalid("command stderr unavailable"))?;
    let errors = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr
            .take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map(|_| bytes)
    });
    let mut bytes = Vec::new();
    let read = child
        .stdout
        .take()
        .ok_or_else(|| invalid("command stdout unavailable"))?
        .take(maximum + 1)
        .read_to_end(&mut bytes);
    if read.is_err() || bytes.len() as u64 > maximum {
        let _ = child.kill();
    }
    let status = child.wait()?;
    let stderr = errors
        .join()
        .map_err(|_| invalid("command stderr reader failed"))??;
    read?;
    if !status.success() || bytes.len() as u64 > maximum || stderr.len() > 1024 * 1024 {
        return Err(invalid(format!(
            "{} failed ({status}) or exceeded output extent: {}",
            program.display(),
            String::from_utf8_lossy(&stderr)
        )));
    }
    Ok(bytes)
}
fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

fn archive(
    path: &Path,
    variant: Variant,
    tar: &Path,
    root: &Path,
    env: &Environment,
) -> Result<(), HostError> {
    bounded_read(path, MAX_ARCHIVE)?;
    let mut listing = args(&["--zstd", "--list", "--file"]);
    listing.push(path.into());
    let names = command(tar, &listing, root, env, MAX_MANIFEST)?;
    let names = std::str::from_utf8(&names).map_err(invalid)?;
    if names.lines().count() != MEMBERS.len()
        || names.lines().collect::<BTreeSet<_>>() != BTreeSet::from(MEMBERS)
    {
        return Err(invalid("archive has extra, duplicate or missing members"));
    }
    listing.push("--verbose".into());
    let verbose = command(tar, &listing, root, env, MAX_MANIFEST)?;
    let verbose = std::str::from_utf8(&verbose).map_err(invalid)?;
    if verbose.lines().count() != MEMBERS.len()
        || verbose.lines().any(|line| !line.starts_with('-'))
    {
        return Err(invalid(
            "archive must contain only regular unlinked members",
        ));
    }
    let (expected, _, _) = declaration(variant)?;
    for member in MEMBERS {
        let mut arguments = args(&["--zstd", "--extract", "--to-stdout", "--file"]);
        arguments.push(path.into());
        arguments.push("--".into());
        arguments.push(member.into());
        let maximum = if member.ends_with(".wasm") {
            MAX_ARCHIVE
        } else {
            MAX_MANIFEST
        };
        let bytes = command(tar, &arguments, root, env, maximum)?;
        if bytes.is_empty() {
            return Err(invalid("archive member is empty"));
        }
        if member.ends_with(".wasm") {
            if !bytes.starts_with(PREAMBLE) {
                return Err(invalid("module is not a WASI Preview 2 component"));
            }
        } else if member.ends_with("exports.toml") {
            validate_exports(&bytes, variant)?;
        } else {
            let config: toml::Value =
                toml::from_str(std::str::from_utf8(&bytes).map_err(invalid)?).map_err(invalid)?;
            let mut config = serde_json::to_value(config).map_err(invalid)?;
            if config.get("attach") == Some(&json!({})) {
                config
                    .as_object_mut()
                    .ok_or_else(|| invalid("configuration is not an object"))?
                    .remove("attach");
            }
            if config != expected {
                return Err(invalid(
                    "packaged configuration differs from the module-owned declaration",
                ));
            }
        }
    }
    Ok(())
}

fn visit(
    directory: &Path,
    root: &Path,
    sources: &mut BTreeMap<String, String>,
) -> Result<(), HostError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if ["target", ".git", "__pycache__"]
            .iter()
            .any(|name| entry.file_name() == *name)
        {
            continue;
        }
        let path = entry.path();
        let status = fs::symlink_metadata(&path)?;
        if status.file_type().is_symlink() {
            return Err(invalid("module inputs must not traverse symlinks"));
        }
        if status.is_dir() {
            visit(&path, root, sources)?;
        } else if status.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(invalid)?
                .to_str()
                .ok_or_else(|| invalid("source path must be UTF-8"))?
                .to_owned();
            sources.insert(relative, digest(&path)?);
        } else {
            return Err(invalid("module inputs must be regular files"));
        }
    }
    Ok(())
}
fn local_sources(metadata: &Value, root: &Path) -> Result<BTreeMap<String, String>, HostError> {
    let packages = metadata["packages"]
        .as_array()
        .ok_or_else(|| invalid("dependency packages unavailable"))?;
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .ok_or_else(|| invalid("dependency graph unavailable"))?;
    let selected = packages
        .iter()
        .find(|package| {
            package["manifest_path"].as_str() == root.join(CRATE).join("Cargo.toml").to_str()
        })
        .and_then(|package| package["id"].as_str())
        .ok_or_else(|| invalid("module absent from workspace"))?;
    let mut pending = vec![selected];
    let mut visited = BTreeSet::new();
    let mut sources = BTreeMap::new();
    while let Some(id) = pending.pop() {
        if !visited.insert(id) {
            continue;
        }
        let package = packages
            .iter()
            .find(|package| package["id"].as_str() == Some(id))
            .ok_or_else(|| invalid("dependency package absent"))?;
        let node = nodes
            .iter()
            .find(|node| node["id"].as_str() == Some(id))
            .ok_or_else(|| invalid("dependency node absent"))?;
        for dependency in node["deps"]
            .as_array()
            .ok_or_else(|| invalid("dependency edges absent"))?
        {
            let kinds = dependency["dep_kinds"]
                .as_array()
                .ok_or_else(|| invalid("dependency kinds absent"))?;
            if kinds.iter().any(|kind| kind["kind"] != "dev") {
                pending.push(
                    dependency["pkg"]
                        .as_str()
                        .ok_or_else(|| invalid("dependency identity absent"))?,
                );
            }
        }
        if package["source"].is_null() {
            let manifest = Path::new(
                package["manifest_path"]
                    .as_str()
                    .ok_or_else(|| invalid("manifest absent"))?,
            );
            exact_path(manifest)?;
            if !manifest.starts_with(root) {
                return Err(invalid("local module dependency escaped workspace"));
            }
            visit(
                manifest
                    .parent()
                    .ok_or_else(|| invalid("manifest parent absent"))?,
                root,
                &mut sources,
            )?;
        }
    }
    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".cargo/config.toml",
    ] {
        let path = root.join(relative);
        exact_path(&path)?;
        sources.insert(relative.into(), digest(&path)?);
    }
    Ok(sources)
}
fn inputs(
    root: &Path,
    server: &PreparedServer,
    variant: Variant,
    env: &Environment,
) -> Result<BTreeMap<String, Value>, HostError> {
    let (declared, _, sdk) = declaration(variant)?;
    let current: toml::Value = toml::from_str(&fs::read_to_string(
        root.join(CRATE).join("surrealism.toml"),
    )?)
    .map_err(invalid)?;
    if serde_json::to_value(current).map_err(invalid)? != declared {
        return Err(invalid(
            "host declaration is stale; explicitly rebuild host tools",
        ));
    }
    let cargo = resolve("cargo", env)?;
    let mut arguments = args(&[
        "metadata",
        "--locked",
        "--format-version",
        "1",
        "--filter-platform",
        TARGET,
        "--manifest-path",
    ]);
    arguments.push(root.join(CRATE).join("Cargo.toml").into_os_string());
    if !variant.features().is_empty() {
        arguments.push("--features".into());
        arguments.push(
            variant
                .features()
                .iter()
                .map(|feature| format!("pse-study-module/{feature}"))
                .collect::<Vec<_>>()
                .join(",")
                .into(),
        );
    }
    let metadata: Value =
        serde_json::from_slice(&command(&cargo, &arguments, root, env, 64 * 1024 * 1024)?)
            .map_err(invalid)?;
    if !metadata["packages"].as_array().is_some_and(|packages| {
        packages
            .iter()
            .any(|package| package["name"] == "surrealism" && package["version"] == sdk)
    }) {
        return Err(invalid(
            "resolved module SDK differs from the exact workspace pin",
        ));
    }
    let sources = local_sources(&metadata, root)?;
    let mut tools = BTreeMap::new();
    for name in ["cargo", "rustc", "tar", "zstd"] {
        let path = resolve(name, env)?;
        let version = command(&path, &args(&["--version"]), root, env, MAX_MANIFEST)?;
        tools.insert(name, json!({"path":path,"sha256":digest(&path)?,"version":String::from_utf8(version).map_err(invalid)?}));
    }
    let rustc = resolve("rustc", env)?;
    let sysroot = command(
        &rustc,
        &args(&["--print", "sysroot"]),
        root,
        env,
        MAX_MANIFEST,
    )?;
    let sysroot = PathBuf::from(String::from_utf8(sysroot).map_err(invalid)?.trim());
    for name in ["cargo", "rustc"] {
        let path = sysroot.join("bin").join(name);
        tools.insert(
            if name == "cargo" {
                "toolchain_cargo"
            } else {
                "toolchain_rustc"
            },
            json!({"path":path,"sha256":digest(&path)?}),
        );
    }
    if let Some(wrapper) = env
        .get(OsStr::new("RUSTC_WRAPPER"))
        .filter(|value| !value.is_empty())
    {
        let wrapper = PathBuf::from(wrapper);
        let wrapper = if wrapper.is_absolute() {
            wrapper
        } else {
            resolve(
                wrapper
                    .to_str()
                    .ok_or_else(|| invalid("wrapper must be UTF-8"))?,
                env,
            )?
        };
        tools.insert(
            "compiler_wrapper",
            json!({"path":wrapper,"sha256":digest(&wrapper)?}),
        );
    }
    let mut configuration = BTreeMap::new();
    let cargo_home = env
        .get(OsStr::new("CARGO_HOME"))
        .map(PathBuf::from)
        .or_else(|| {
            env.get(OsStr::new("HOME"))
                .map(|home| PathBuf::from(home).join(".cargo"))
        });
    let mut config_directories: BTreeSet<PathBuf> = root
        .join(CRATE)
        .ancestors()
        .map(|directory| directory.join(".cargo"))
        .collect();
    if let Some(home) = cargo_home {
        config_directories.insert(home);
    }
    for directory in config_directories {
        for name in ["config", "config.toml"] {
            let path = directory.join(name);
            let identity = if path.exists() {
                Some(digest(&path)?)
            } else {
                None
            };
            configuration.insert(path, identity);
        }
    }
    let host = std::env::current_exe()?;
    let controls: BTreeMap<_, _> = env
        .iter()
        .map(|(key, value)| {
            Ok((
                key.to_str()
                    .ok_or_else(|| invalid("environment name is not UTF-8"))?,
                digest_bytes(value.as_encoded_bytes()),
            ))
        })
        .collect::<Result<_, HostError>>()?;
    Ok(BTreeMap::from([
        ("kind".into(), json!(variant.kind())),
        ("features".into(), json!(variant.features())),
        ("sdk_version".into(), json!(sdk)),
        ("target".into(), json!(TARGET)),
        ("profile".into(), json!("release")),
        (
            "builder_sha256".into(),
            json!(digest(&server.executable())?),
        ),
        ("sources".into(), json!(sources)),
        (
            "preparation".into(),
            json!({"kind":"rust-cli-v1","host_sha256":digest(&host)?,"server_recipe":server.recipe(),"tools":tools,"configuration":configuration,"environment":controls,"command":["module","build","--out","study.surli",CRATE],"archive_validation":"tar-zstd-exact-three-members-v1"}),
        ),
    ]))
}
fn publish(
    modules: &Path,
    pending: &Path,
    receipt: &StudyModuleReceipt,
    variant: Variant,
) -> Result<StudyModuleReceipt, HostError> {
    // TempDir guarantees exclusive naming, while the issued artifact contract
    // additionally requires an explicitly private final directory.
    fs::set_permissions(pending, fs::Permissions::from_mode(0o700))?;
    let artifact = pending.join("study.surli");
    fs::set_permissions(&artifact, fs::Permissions::from_mode(0o400))?;
    File::open(&artifact)?.sync_all()?;
    let mut record = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(pending.join("module.json"))?;
    serde_json::to_writer(&mut record, receipt).map_err(invalid)?;
    record.write_all(b"\n")?;
    record.sync_all()?;
    File::open(pending)?.sync_all()?;
    let destination = receipt
        .artifact
        .parent()
        .ok_or_else(|| invalid("destination absent"))?;
    match rustix::fs::renameat_with(
        rustix::fs::CWD,
        pending,
        rustix::fs::CWD,
        destination,
        rustix::fs::RenameFlags::NOREPLACE,
    ) {
        Ok(()) => {
            File::open(modules)?.sync_all()?;
        }
        Err(rustix::io::Errno::EXIST) => {
            let existing = select(&receipt.record(), variant)?;
            if existing != *receipt {
                return Err(invalid(
                    "concurrent completed generation differs; retained unchanged",
                ));
            }
            return Ok(existing);
        }
        Err(error) => return Err(std::io::Error::from(error).into()),
    }
    select(&receipt.record(), variant)
}
fn prepare_inner(
    root: &Path,
    caller: &Environment,
    tool_root: &Path,
    variant: Variant,
) -> Result<StudyModuleReceipt, HostError> {
    variant.constructible()?;
    exact_path(root)?;
    if !tool_root.is_absolute()
        || tool_root.to_str().is_none_or(|value| value.contains(':'))
        || tool_root
            .components()
            .any(|part| !matches!(part, Component::RootDir | Component::Normal(_)))
    {
        return Err(invalid(
            "module tools root must be absolute, without traversal or path separators",
        ));
    }
    let env = environment(caller)?;
    let server = crate::server::select(caller)?;
    let before = inputs(root, &server, variant, &env)?;
    let modules = tool_root.join("rust-v1/modules");
    private_directory(&modules)?;
    let directory = modules.join(input_identity(&before)?);
    if directory.exists() {
        let receipt = select(&directory.join("module.json"), variant)?;
        if receipt.inputs != serde_json::Map::from_iter(before.clone()) {
            return Err(invalid("cached module inputs differ"));
        }
        return Ok(receipt);
    }
    let pending = tempfile::Builder::new()
        .prefix("pending-")
        .tempdir_in(&modules)?;
    let artifact = pending.path().join("study.surli");
    let mut arguments = args(&["module", "build"]);
    if !variant.features().is_empty() {
        arguments.extend(args(&["--features", &variant.features().join(",")]));
    }
    arguments.push("--out".into());
    arguments.push(artifact.clone().into_os_string());
    arguments.push(root.join(CRATE).into_os_string());
    command(
        &server.executable(),
        &arguments,
        root,
        &env,
        2 * 1024 * 1024,
    )?;
    if inputs(root, &server, variant, &env)? != before {
        return Err(invalid(
            "module sources, locked dependencies or tools changed during build",
        ));
    }
    archive(&artifact, variant, &resolve("tar", &env)?, root, &env)?;
    let (_, limits, sdk) = declaration(variant)?;
    let receipt = StudyModuleReceipt(StudyModuleRecord {
        kind: variant.kind().into(),
        artifact: directory.join("study.surli"),
        sha256: digest(&artifact)?,
        bytes: fs::metadata(&artifact)?.len(),
        sdk_version: sdk,
        abi: 2,
        inputs: serde_json::Map::from_iter(before),
        limits,
    });
    publish(&modules, pending.path(), &receipt, variant)
}
/// Explicit preparation inside authenticated native containment. No service is started.
pub async fn prepare(
    root: &Path,
    environment: &Environment,
    tool_root: &Path,
    variant: Variant,
) -> Result<StudyModuleReceipt, HostError> {
    variant.constructible()?;
    let manager = HostManager::connect_user(None).await?;
    let current = manager.current_unit().await?;
    if !manager.native_workload(&current).await? {
        return Err(invalid(
            "explicit preparation requires scripts/pse-env containment",
        ));
    }
    let (root, environment, tool_root) =
        (root.to_owned(), environment.clone(), tool_root.to_owned());
    tokio::task::spawn_blocking(move || prepare_inner(&root, &environment, &tool_root, variant))
        .await
        .map_err(|error| invalid(format!("module preparation worker failed: {error}")))?
}
fn tool_root(env: &Environment) -> Result<PathBuf, HostError> {
    if let Some(root) = env.get(OsStr::new("PSE_SURREAL_TOOLS")) {
        return Ok(root.into());
    }
    let data = match env.get(OsStr::new("XDG_DATA_HOME")) {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(
            env.get(OsStr::new("HOME"))
                .ok_or_else(|| invalid("HOME unavailable"))?,
        )
        .join(".local/share"),
    };
    Ok(data.join("pse-arrow/tools/surreal"))
}
/// Narrow explicit setup surface. Selection requires an exact receipt, never current sources.
pub fn run_cli(arguments: &[OsString]) -> Result<i32, HostError> {
    if arguments == [OsString::from("--help")]
        || arguments == [OsString::from("-h")]
        || (arguments.len() == 2
            && matches!(arguments[0].to_str(), Some("prepare" | "select"))
            && matches!(arguments[1].to_str(), Some("--help" | "-h")))
    {
        writeln!(
            std::io::stdout(),
            "study-module prepare [--tool-root PATH] [--qualification] | select --receipt PATH [--qualification [--long-hold]]\nPreparation uses the already selected native server CLI; selection never builds or hashes source.\nNew long-hold preparation requires official CLI configuration support. No database is started or changed."
        )?;
        return Ok(0);
    }
    let operation = arguments
        .first()
        .and_then(|value| value.to_str())
        .ok_or_else(|| invalid("expected prepare or select"))?;
    let mut qualification = false;
    let mut long_hold = false;
    let mut selected_record = None;
    let mut selected_root = None;
    let mut index = 1;
    while index < arguments.len() {
        match arguments[index].to_str() {
            Some("--qualification") if !qualification => qualification = true,
            Some("--long-hold") if !long_hold => long_hold = true,
            Some("--receipt") if selected_record.is_none() => {
                index += 1;
                selected_record = Some(PathBuf::from(
                    arguments
                        .get(index)
                        .ok_or_else(|| invalid("--receipt needs a path"))?,
                ));
            }
            Some("--tool-root") if selected_root.is_none() => {
                index += 1;
                selected_root = Some(PathBuf::from(
                    arguments
                        .get(index)
                        .ok_or_else(|| invalid("--tool-root needs a path"))?,
                ));
            }
            _ => {
                return Err(invalid(
                    "unknown or duplicate study-module argument; use --help",
                ));
            }
        }
        index += 1;
    }
    if long_hold && !qualification {
        return Err(invalid("--long-hold requires --qualification"));
    }
    let variant = if long_hold {
        Variant::LongHold
    } else if qualification {
        Variant::Qualification
    } else {
        Variant::Production
    };
    let receipt = match operation {
        "select" if selected_root.is_none() => select(
            &selected_record.ok_or_else(|| invalid("select requires --receipt"))?,
            variant,
        )?,
        "prepare" if selected_record.is_none() => {
            variant.constructible()?;
            let mut root = std::env::current_dir()?;
            while !root.join(".config/diagnostic-tools.toml").is_file() {
                if !root.pop() {
                    return Err(invalid(
                        "run explicit preparation inside the selected checkout",
                    ));
                }
            }
            let env = std::env::vars_os().collect();
            let tools = selected_root.map(Ok).unwrap_or_else(|| tool_root(&env))?;
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?
                .block_on(prepare(&root, &env, &tools, variant))?
        }
        _ => {
            return Err(invalid(
                "expected prepare [--tool-root PATH] or select --receipt PATH",
            ));
        }
    };
    serde_json::to_writer(std::io::stdout(), &receipt).map_err(invalid)?;
    writeln!(std::io::stdout())?;
    Ok(0)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "isolated mechanism fixtures fail immediately"
)]
mod tests {
    use super::*;

    fn fixture_inputs(variant: Variant) -> BTreeMap<String, Value> {
        let (_, _, sdk) = declaration(variant).unwrap();
        let mut inputs = BTreeMap::from([
            ("kind".into(), json!(variant.kind())),
            ("features".into(), json!(variant.features())),
            ("sdk_version".into(), json!(sdk)),
            ("target".into(), json!(TARGET)),
            ("profile".into(), json!("release")),
            ("builder_sha256".into(), json!("1".repeat(64))),
            (
                "sources".into(),
                json!({"deleted/source.rs":"2".repeat(64)}),
            ),
        ]);
        if variant == Variant::LongHold {
            let name = format!("{CRATE}/surrealism-qualification-long-hold.toml");
            inputs.insert("sources".into(), json!({(name):"2".repeat(64)}));
            inputs.insert("packaging".into(), json!({"variant":"qualification-long-hold-v1","packer_sha256":"3".repeat(64),"manifest_sha256":"2".repeat(64)}));
        }
        inputs
    }
    fn fixture_receipt(modules: &Path, pending: &Path, variant: Variant) -> StudyModuleReceipt {
        let payload = b"controlled module publication bytes";
        fs::write(pending.join("study.surli"), payload).unwrap();
        let inputs = fixture_inputs(variant);
        let (_, limits, sdk) = declaration(variant).unwrap();
        StudyModuleReceipt(StudyModuleRecord {
            kind: variant.kind().into(),
            artifact: modules
                .join(input_identity(&inputs).unwrap())
                .join("study.surli"),
            sha256: digest_bytes(payload),
            bytes: payload.len() as u64,
            sdk_version: sdk,
            abi: 2,
            inputs: serde_json::Map::from_iter(inputs),
            limits,
        })
    }
    fn exports(variant: Variant) -> String {
        variant
            .exports()
            .iter()
            .map(|name| format!("[[functions]]\nname = {name:?}\nwriteable = false\n"))
            .collect()
    }

    #[test]
    fn study_module_identity_matches_retained_compact_ascii_json() {
        assert_eq!(
            canonical(&json!({"z":[true,0],"é":"🦀"})).unwrap(),
            r#"{"z":[true,0],"\u00e9":"\ud83e\udd80"}"#
        );
        let mut original = fixture_inputs(Variant::Production);
        let first = input_identity(&original).unwrap();
        original.insert("features".into(), json!(["module-qualification"]));
        assert_ne!(first, input_identity(&original).unwrap());
        original.insert("preparation".into(), json!({"host_sha256":"4".repeat(64)}));
        let second = input_identity(&original).unwrap();
        original.get_mut("preparation").unwrap()["host_sha256"] = json!("5".repeat(64));
        assert_ne!(second, input_identity(&original).unwrap());
    }

    #[test]
    fn study_module_exports_and_grants_preserve_pure_variant_contracts() {
        for variant in [
            Variant::Production,
            Variant::Qualification,
            Variant::LongHold,
        ] {
            let (config, _, _) = declaration(variant).unwrap();
            validate_pure(&config).unwrap();
            validate_exports(exports(variant).as_bytes(), variant).unwrap();
            assert!(
                validate_exports(
                    exports(variant)
                        .replace("writeable = false", "writeable = true")
                        .as_bytes(),
                    variant
                )
                .is_err()
            );
            assert!(
                validate_exports(
                    format!("{}{}", exports(variant), exports(variant)).as_bytes(),
                    variant
                )
                .is_err()
            );
            let mut granted = config.clone();
            granted["capabilities"]["allow_scripting"] = json!(true);
            assert!(validate_pure(&granted).is_err());
            let mut attached = config;
            attached["attach"] = json!({"fs":"secret"});
            assert!(validate_pure(&attached).is_err());
        }
        assert!(
            validate_exports(
                exports(Variant::Qualification).as_bytes(),
                Variant::Production
            )
            .is_err()
        );
        assert!(
            validate_exports(
                exports(Variant::Production).as_bytes(),
                Variant::Qualification
            )
            .is_err()
        );
    }

    #[test]
    fn study_module_archive_controls_bind_exact_members_component_configuration_and_exports() {
        let scratch = tempfile::tempdir().unwrap();
        let root = scratch.path();
        let stage = root.join("package");
        fs::create_dir_all(stage.join("surrealism")).unwrap();
        let wasm = stage.join(MEMBERS[0]);
        let config = stage.join(MEMBERS[1]);
        let functions = stage.join(MEMBERS[2]);
        let declared = include_str!("../../pse-study-module/surrealism.toml");
        fs::write(&wasm, PREAMBLE).unwrap();
        fs::write(&config, declared).unwrap();
        fs::write(&functions, exports(Variant::Production)).unwrap();
        let env = Environment::from_iter([("PATH".into(), std::env::var_os("PATH").unwrap())]);
        let tar = resolve("tar", &env).unwrap();
        let artifact = root.join("fixture.surli");
        let package = |members: &[&str]| {
            let mut arguments = args(&["--zstd", "--create", "--file"]);
            arguments.push(artifact.clone().into_os_string());
            arguments.push("--directory".into());
            arguments.push(stage.clone().into_os_string());
            arguments.push("--".into());
            arguments.extend(args(members));
            command(&tar, &arguments, root, &env, MAX_MANIFEST).unwrap();
        };
        package(&MEMBERS);
        archive(&artifact, Variant::Production, &tar, root, &env).unwrap();
        package(&[MEMBERS[0], MEMBERS[1], MEMBERS[2], MEMBERS[2]]);
        assert!(archive(&artifact, Variant::Production, &tar, root, &env).is_err());
        fs::write(stage.join("extra"), "attachment").unwrap();
        package(&[MEMBERS[0], MEMBERS[1], MEMBERS[2], "extra"]);
        assert!(archive(&artifact, Variant::Production, &tar, root, &env).is_err());
        fs::write(&wasm, b"not-a-component").unwrap();
        package(&MEMBERS);
        assert!(archive(&artifact, Variant::Production, &tar, root, &env).is_err());
        fs::write(&wasm, PREAMBLE).unwrap();
        fs::write(
            &config,
            declared.replace("allow_net = []", "allow_net = [\"example.invalid\"]"),
        )
        .unwrap();
        package(&MEMBERS);
        assert!(archive(&artifact, Variant::Production, &tar, root, &env).is_err());
        fs::write(&config, declared).unwrap();
        fs::write(&functions, exports(Variant::Qualification)).unwrap();
        package(&MEMBERS);
        assert!(archive(&artifact, Variant::Production, &tar, root, &env).is_err());
        archive(&artifact, Variant::Qualification, &tar, root, &env).unwrap();
        fs::remove_file(&wasm).unwrap();
        std::os::unix::fs::symlink(&functions, &wasm).unwrap();
        package(&MEMBERS);
        assert!(archive(&artifact, Variant::Qualification, &tar, root, &env).is_err());
    }

    #[test]
    fn study_module_selection_is_source_free_and_binds_private_immutable_bytes() {
        let scratch = tempfile::tempdir().unwrap();
        let pending = tempfile::tempdir_in(scratch.path()).unwrap();
        let receipt = fixture_receipt(scratch.path(), pending.path(), Variant::Production);
        let selected = publish(
            scratch.path(),
            pending.path(),
            &receipt,
            Variant::Production,
        )
        .unwrap();
        assert_eq!(
            select(&selected.record(), Variant::Production).unwrap(),
            receipt
        );
        assert!(select(&selected.record(), Variant::Qualification).is_err());
        let mut wrong = receipt.clone();
        wrong.0.limits.max_pool_size += 1;
        assert!(validate_receipt(&wrong, Variant::Production).is_err());
        let mut wrong = receipt.clone();
        wrong
            .0
            .inputs
            .insert("builder_sha256".into(), json!("0".repeat(64)));
        assert!(validate_receipt(&wrong, Variant::Production).is_err());
        let alias = scratch.path().join("hardlink");
        fs::hard_link(receipt.artifact(), &alias).unwrap();
        assert!(select(&receipt.record(), Variant::Production).is_err());
        fs::remove_file(alias).unwrap();
        let alias = scratch.path().join("alias");
        std::os::unix::fs::symlink(receipt.artifact().parent().unwrap(), &alias).unwrap();
        assert!(select(&alias.join("module.json"), Variant::Production).is_err());
        fs::set_permissions(receipt.artifact(), fs::Permissions::from_mode(0o600)).unwrap();
        assert!(select(&receipt.record(), Variant::Production).is_err());
        fs::write(receipt.artifact(), b"changed").unwrap();
        fs::set_permissions(receipt.artifact(), fs::Permissions::from_mode(0o400)).unwrap();
        assert!(select(&receipt.record(), Variant::Production).is_err());
    }

    #[test]
    fn study_module_publication_never_replaces_existing_completed_or_empty_directory() {
        let scratch = tempfile::tempdir().unwrap();
        let one = tempfile::tempdir_in(scratch.path()).unwrap();
        let receipt = fixture_receipt(scratch.path(), one.path(), Variant::Production);
        let destination = receipt.artifact().parent().unwrap();
        fs::create_dir(destination).unwrap();
        fs::set_permissions(destination, fs::Permissions::from_mode(0o700)).unwrap();
        assert!(publish(scratch.path(), one.path(), &receipt, Variant::Production).is_err());
        assert_eq!(fs::read_dir(destination).unwrap().count(), 0);
        fs::remove_dir(destination).unwrap();
        fs::remove_file(one.path().join("module.json")).unwrap();
        publish(scratch.path(), one.path(), &receipt, Variant::Production).unwrap();
        let two = tempfile::tempdir_in(scratch.path()).unwrap();
        let mut other = fixture_receipt(scratch.path(), two.path(), Variant::Production);
        assert_eq!(
            publish(scratch.path(), two.path(), &other, Variant::Production).unwrap(),
            receipt
        );
        fs::remove_file(two.path().join("module.json")).unwrap();
        other.0.sha256 = "0".repeat(64);
        assert!(publish(scratch.path(), two.path(), &other, Variant::Production).is_err());
        assert_eq!(
            select(&receipt.record(), Variant::Production).unwrap(),
            receipt
        );
    }

    #[test]
    fn study_module_retained_long_hold_identity_is_distinct_and_new_build_refuses() {
        let scratch = tempfile::tempdir().unwrap();
        let pending = tempfile::tempdir_in(scratch.path()).unwrap();
        let receipt = fixture_receipt(scratch.path(), pending.path(), Variant::LongHold);
        publish(scratch.path(), pending.path(), &receipt, Variant::LongHold).unwrap();
        select(&receipt.record(), Variant::LongHold).unwrap();
        assert!(select(&receipt.record(), Variant::Qualification).is_err());
        // The argument guard runs before environment, manager or filesystem preparation.
        let error = run_cli(&args(&[
            "prepare",
            "--qualification",
            "--long-hold",
            "--tool-root",
            "/must-not-be-created",
        ]))
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("official CLI configuration override")
        );
    }

    #[test]
    fn study_module_sources_follow_non_dev_local_dependency_closure_and_refuse_symlinks() {
        let scratch = tempfile::tempdir().unwrap();
        let root = scratch.path();
        for directory in [CRATE, "crates/dependency", "crates/dev-only", ".cargo"] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }
        for name in [
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            ".cargo/config.toml",
            "crates/pse-study-module/Cargo.toml",
            "crates/pse-study-module/lib.rs",
            "crates/dependency/Cargo.toml",
            "crates/dependency/lib.rs",
            "crates/dev-only/Cargo.toml",
        ] {
            fs::write(root.join(name), name).unwrap();
        }
        let metadata = json!({"packages":[
            {"id":"module","source":null,"manifest_path":root.join(CRATE).join("Cargo.toml")},
            {"id":"local","source":null,"manifest_path":root.join("crates/dependency/Cargo.toml")},
            {"id":"dev","source":null,"manifest_path":root.join("crates/dev-only/Cargo.toml")},
            {"id":"registry","source":"registry+fixture","manifest_path":"/unused/registry/Cargo.toml"}],
            "resolve":{"nodes":[{"id":"module","deps":[{"pkg":"registry","dep_kinds":[{"kind":null}]},{"pkg":"dev","dep_kinds":[{"kind":"dev"}]}]},
            {"id":"registry","deps":[{"pkg":"local","dep_kinds":[{"kind":"build"}]}]},
            {"id":"local","deps":[]},{"id":"dev","deps":[]}]}});
        let sources = local_sources(&metadata, root).unwrap();
        assert!(sources.contains_key("crates/dependency/lib.rs"));
        assert!(!sources.contains_key("crates/dev-only/Cargo.toml"));
        fs::write(root.join("crates/dependency/lib.rs"), "changed").unwrap();
        assert_ne!(sources, local_sources(&metadata, root).unwrap());
        std::os::unix::fs::symlink(root.join("Cargo.lock"), root.join(CRATE).join("linked"))
            .unwrap();
        assert!(local_sources(&metadata, root).is_err());
    }

    #[test]
    fn study_module_build_environment_rejects_hidden_controls_and_keeps_host_width() {
        let env = Environment::from_iter([
            ("PATH".into(), "/usr/bin".into()),
            ("CARGO_BUILD_JOBS".into(), "32".into()),
            ("RUST_MIN_STACK".into(), "16777216".into()),
            ("BASH_ENV".into(), "/not-evaluated".into()),
        ]);
        let selected = environment(&env).unwrap();
        assert_eq!(
            selected.get(OsStr::new("CARGO_BUILD_JOBS")),
            Some(&OsString::from("32"))
        );
        assert!(!selected.contains_key(OsStr::new("BASH_ENV")));
        for name in [
            "CARGO_TARGET_DIR",
            "RUSTUP_TOOLCHAIN",
            "RUSTC_WORKSPACE_WRAPPER",
        ] {
            let mut changed = env.clone();
            changed.insert(name.into(), "unsupported".into());
            assert!(environment(&changed).is_err());
        }
    }
}
