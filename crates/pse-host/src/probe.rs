// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored scratch workspaces, isolated resolution and disposal with original user leases.
use crate::{catalog::MetadataSnapshot, correctness, environment::Environment};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::{
    ffi::{OsStr, OsString},
    fs::{self, File, OpenOptions},
    io::Write as _,
    os::unix::fs::{DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _},
    path::{Path, PathBuf},
    process::Command,
};

/// Scratch ownership/resolution failure, independent of the probe's own exit status.
#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    /// Original native/ledger/custody refusal, without substituting process-marker authority.
    #[error(transparent)]
    Host(Box<crate::HostError>),
    /// A nonempty original capture has no readable authenticated run binding.
    #[error(
        "probe has an unknown legacy or incomplete run capture at {capture:?}; original custody cannot be established: {source}"
    )]
    RunCustody {
        /// Original capture, preserving native path bytes.
        capture: PathBuf,
        /// Original read/ownership failure; absence never proves completion.
        #[source]
        source: Box<crate::HostError>,
    },
    /// Original file/process failure.
    #[error("probe I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Original retained record decoding failure.
    #[error("probe JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Original authored manifest encoding failure.
    #[error("probe manifest: {0}")]
    Toml(#[from] toml::ser::Error),
    /// Original compiler admission failure.
    #[error(transparent)]
    Correctness(#[from] correctness::CorrectnessError),
    /// Invalid ownership or selection; no guessed source is substituted.
    #[error("probe: {0}")]
    Invalid(String),
}
pse_diagnostics::impl_diagnostic! {
    ProbeError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::HostOwnership) },
    forward(this) { match this { Self::Correctness(error) => Some(error),Self::Host(error)=>Some(error.as_ref()),Self::RunCustody {source,..}=>Some(source.as_ref()), _ => None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}
fn invalid(message: impl Into<String>) -> ProbeError {
    ProbeError::Invalid(message.into())
}
fn host(error: crate::HostError) -> ProbeError {
    ProbeError::Host(Box::new(error))
}
pub(crate) fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Selected source-qualified dependency and explicitly enabled metadata features.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Dependency {
    /// Cargo ID from the originating locked graph.
    pub package_id: String,
    /// Direct scratch dependency name.
    pub name: String,
    /// Exact selected package version.
    pub version: String,
    /// Features explicitly copied from the selected metadata graph.
    pub features: Vec<String>,
}
/// Retained authored input identity, independent of later scratch re-resolution.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Checked record version.
    pub version: u32,
    /// Originating checkout, for source navigation only.
    pub checkout: PathBuf,
    /// Exact seed Cargo.lock digest; it does not freeze a modified manifest.
    pub seed_lock_sha256: String,
    /// Exact copied rust-toolchain.toml digest.
    pub toolchain_sha256: String,
    /// Originating dependency metadata digest.
    pub metadata_sha256: String,
    /// Requested source/version/feature selections.
    pub dependencies: Vec<Dependency>,
}
/// One created or selected authored workspace.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Probe {
    /// Name is a single checked path component.
    pub name: String,
    /// Separate scratch workspace, never the repository build directory.
    pub directory: PathBuf,
    /// Separate Cargo output directory.
    pub target_directory: PathBuf,
    /// Authored input provenance.
    pub manifest: Manifest,
}
fn name_component(name: &str) -> Result<(), ProbeError> {
    if name.is_empty()
        || name.len() > 80
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(invalid(
            "name must contain 1..80 ASCII letters, digits, '-' or '_'",
        ));
    }
    Ok(())
}
fn directory(path: &Path) -> Result<(), ProbeError> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)?;
    let metadata = fs::symlink_metadata(path)?;
    if !path.is_absolute()
        || !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
    {
        return Err(invalid(
            "probe directories must be absolute, private, owned directories",
        ));
    }
    Ok(())
}
fn publish(path: &Path, bytes: &[u8]) -> Result<(), ProbeError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
/// Original live-user lease. Its lock lives outside the removable workspace.
#[derive(Debug)]
pub struct Lease {
    _file: File,
    store: PathBuf,
    name: String,
}
fn lease(store: &Path, name: &str) -> Result<Lease, ProbeError> {
    name_component(name)?;
    directory(&store.join("probes"))?;
    directory(&store.join("probes/.locks"))?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(store.join("probes/.locks").join(name))?;
    rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockExclusive).map_err(
        |error| {
            invalid(format!(
                "probe {name} has a live owner or its lease is unavailable: {error}"
            ))
        },
    )?;
    Ok(Lease {
        _file: file,
        store: store.to_owned(),
        name: name.into(),
    })
}

/// Create an authored scratch workspace from exact locked packages without changing the checkout.
/// Dependency queries accept source-qualified IDs or an unambiguous `name[@version]`.
pub fn new(
    root: &Path,
    store: &Path,
    name: &str,
    queries: &[String],
    snapshot: &MetadataSnapshot,
) -> Result<Probe, ProbeError> {
    let _lease = lease(store, name)?;
    let root = root.canonicalize()?;
    let toolchain = fs::read(root.join("rust-toolchain.toml"))?;
    let lock = fs::read(root.join("Cargo.lock"))?;
    let mut dependencies = toml::map::Map::new();
    let mut selected = Vec::new();
    for query in queries {
        let package = select(&snapshot.metadata, query)?;
        if dependencies.contains_key(package.name.as_str()) {
            return Err(invalid("duplicate scratch dependency name"));
        }
        let node = snapshot
            .metadata
            .resolve
            .as_ref()
            .and_then(|resolve| resolve.nodes.iter().find(|node| node.id == package.id))
            .ok_or_else(|| {
                invalid("dependency features unavailable without a resolved metadata graph")
            })?;
        let features: Vec<String> = node.features.iter().map(ToString::to_string).collect();
        let mut declaration = toml::map::Map::new();
        declaration.insert(
            "version".into(),
            toml::Value::String(format!("={}", package.version)),
        );
        declaration.insert("default-features".into(), toml::Value::Boolean(false));
        declaration.insert(
            "features".into(),
            toml::Value::Array(features.iter().cloned().map(toml::Value::String).collect()),
        );
        match &package.source {
            Some(source) if source.is_crates_io() => {}
            None => {
                let path = package
                    .manifest_path
                    .parent()
                    .ok_or_else(|| invalid("package manifest has no parent"))?;
                declaration.insert("path".into(), toml::Value::String(path.to_string()));
            }
            Some(_) => {
                return Err(invalid(
                    "non-crates.io dependency needs an explicit authored source declaration; create with supported dependencies then edit the retained Cargo.toml",
                ));
            }
        }
        dependencies.insert(package.name.to_string(), toml::Value::Table(declaration));
        selected.push(Dependency {
            package_id: package.id.to_string(),
            name: package.name.to_string(),
            version: package.version.to_string(),
            features,
        });
    }
    let package = toml::toml! { name = "pse-scratch-probe" version = "0.0.0" edition = "2024" };
    let document = toml::Value::Table(toml::map::Map::from_iter([
        ("package".into(), toml::Value::Table(package)),
        (
            "workspace".into(),
            toml::Value::Table(toml::map::Map::new()),
        ),
        ("dependencies".into(), toml::Value::Table(dependencies)),
    ]));
    let manifest = Manifest {
        version: 1,
        checkout: root,
        seed_lock_sha256: digest(&lock),
        toolchain_sha256: digest(&toolchain),
        metadata_sha256: snapshot.digest.clone(),
        dependencies: selected,
    };
    let path = store.join("probes").join(name);
    fs::DirBuilder::new().mode(0o700).create(&path)?;
    directory(&path.join("src"))?;
    publish(
        &path.join("Cargo.toml"),
        toml::to_string_pretty(&document)?.as_bytes(),
    )?;
    publish(&path.join("Cargo.lock"), &lock)?;
    publish(&path.join("rust-toolchain.toml"), &toolchain)?;
    publish(&path.join("src/main.rs"), b"fn main() {}\n")?;
    publish(
        &path.join("probe.json"),
        &serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(Probe {
        name: name.into(),
        target_directory: path.join("target"),
        directory: path,
        manifest,
    })
}
pub(crate) fn select<'a>(
    metadata: &'a cargo_metadata::Metadata,
    query: &str,
) -> Result<&'a cargo_metadata::Package, ProbeError> {
    if let Some(package) = metadata
        .packages
        .iter()
        .find(|package| package.id.repr == query)
    {
        return Ok(package);
    }
    let (name, version) = query
        .rsplit_once('@')
        .map_or((query, None), |(name, version)| (name, Some(version)));
    let candidates: Vec<_> = metadata
        .packages
        .iter()
        .filter(|package| {
            package.name == name
                && version.is_none_or(|version| package.version.to_string() == version)
        })
        .collect();
    match candidates.as_slice() {
        [package] => Ok(package),
        [] => Err(invalid(format!("no locked package matches {query}"))),
        _ => Err(invalid(format!(
            "ambiguous package {query}; select a Cargo package ID"
        ))),
    }
}
/// Select a retained authored workspace and hold its exclusive original lease.
pub fn open(store: &Path, name: &str) -> Result<(Probe, Lease), ProbeError> {
    let lease = lease(store, name)?;
    let path = store.join("probes").join(name);
    if !path.is_dir() {
        return Err(invalid("named authored probe does not exist"));
    }
    directory(&path)?;
    let manifest: Manifest = serde_json::from_slice(&fs::read(path.join("probe.json"))?)?;
    if manifest.version != 1 {
        return Err(invalid(
            "unsupported probe record version; retain authored inputs and migrate explicitly",
        ));
    }
    Ok((
        Probe {
            name: name.into(),
            target_directory: path.join("target"),
            directory: path,
            manifest,
        },
        lease,
    ))
}
/// Planned managed operation, with original authored inputs leased until native drain completes.
#[derive(Debug)]
pub struct Run {
    /// Selected scratch workspace.
    pub probe: Probe,
    /// Actual resolved metadata, retained before any compiler invocation.
    pub resolution: MetadataSnapshot,
    /// Original resolution stdout/stderr directory, retained even on compiler failure.
    pub resolution_directory: PathBuf,
    /// Actual selected compiler version/commit/host, observed in the scratch workspace.
    pub rustc_identity: String,
    /// Actual selected Cargo version/commit/host, observed in the scratch workspace.
    pub cargo_identity: String,
    /// Native command to run through the managed invocation owner.
    pub command: Vec<OsString>,
    /// Complete environment with any ambient shared target override removed.
    pub environment: Environment,
    /// Keep alive through original native settlement.
    pub lease: Lease,
}
/// Original performing selection, published before launch while the existing probe lease is held.
#[derive(Clone, Debug)]
pub struct ManagedRun {
    /// Original B03 identity, never a PID/marker ownership claim.
    pub invocation: crate::envelope::CliInvocationId,
    /// Actual original verb, either probe run or compiler probe.
    pub command_name: String,
    /// Exact full native performing argv.
    pub command: Vec<OsString>,
    /// Actual immutable performing CLI control identity.
    pub control: crate::generation::PreparedGeneration,
    /// Original image checkout, independent of selected source cwd.
    pub control_checkout: PathBuf,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunReference {
    version: u32,
    checkout: OsString,
    checkout_id: String,
    repository_id: String,
    ledger_root: OsString,
    invocation: crate::envelope::CliInvocationId,
    command_name: String,
    command: Vec<OsString>,
    cwd: OsString,
    control: crate::generation::PreparedGeneration,
    control_checkout: OsString,
    probe_manifest_sha256: String,
}
impl Run {
    /// Publish one durable original-run binding before any native payload launch. Missing or
    /// incomplete original evidence after a crash subsequently protects this probe from rm.
    pub fn record_managed(
        &self,
        root: &Path,
        environment: &Environment,
        selection: ManagedRun,
    ) -> Result<(), ProbeError> {
        let root = root.canonicalize()?;
        if self.lease.name != self.probe.name
            || self.probe.directory != self.lease.store.join("probes").join(&self.lease.name)
            || self.resolution_directory.parent()
                != Some(self.probe.directory.join("runs").as_path())
        {
            return Err(invalid(
                "managed run does not hold this probe's original lease/capture directory",
            ));
        }
        if !matches!(
            selection.command_name.as_str(),
            "probe run" | "compiler probe"
        ) || selection.command.is_empty()
        {
            return Err(invalid(
                "unsupported managed probe verb or empty native argv",
            ));
        }
        let state = crate::paths::CheckoutState::resolve(&root, environment)
            .map_err(crate::HostError::from)
            .map_err(host)?;
        if state.root != self.lease.store {
            return Err(invalid(
                "managed probe lease differs from original checkout ledger",
            ));
        }
        let reference = RunReference {
            version: 1,
            checkout: root.into_os_string(),
            checkout_id: state.checkout_id.as_str().into(),
            repository_id: state.repository_id.as_str().into(),
            ledger_root: state.root.into_os_string(),
            invocation: selection.invocation,
            command_name: selection.command_name,
            command: selection.command,
            cwd: self.probe.directory.as_os_str().to_owned(),
            control: selection.control,
            control_checkout: selection.control_checkout.into_os_string(),
            probe_manifest_sha256: digest(&fs::read(self.probe.directory.join("probe.json"))?),
        };
        publish(
            &self.resolution_directory.join("managed-run.json"),
            &serde_json::to_vec_pretty(&reference)?,
        )?;
        for path in [
            &self.resolution_directory,
            &self.probe.directory.join("runs"),
            &self.probe.directory,
        ] {
            File::open(path)?.sync_all()?;
        }
        Ok(())
    }
}
/// Resolve the scratch graph explicitly, retain its original bytes, then prepare a correctness run.
/// The caller runs `command` through the existing managed native boundary with scratch cwd.
pub fn prepare_run(
    root: &Path,
    store: &Path,
    name: &str,
    environment: &Environment,
    arguments: &[OsString],
) -> Result<Run, ProbeError> {
    prepare_operation(root, store, name, environment, "run", arguments)
}
/// Resolve the same authored scratch graph/lease for a selected native compiler artifact stage.
/// Arguments after `--` are compiler arguments here, rather than arguments to the probe binary.
pub fn prepare_compiler(
    root: &Path,
    store: &Path,
    name: &str,
    environment: &Environment,
    arguments: &[OsString],
) -> Result<Run, ProbeError> {
    prepare_operation(root, store, name, environment, "rustc", arguments)
}
fn prepare_operation(
    root: &Path,
    store: &Path,
    name: &str,
    environment: &Environment,
    operation: &str,
    arguments: &[OsString],
) -> Result<Run, ProbeError> {
    let (probe, lease) = open(store, name)?;
    let toolchain = fs::read(probe.directory.join("rust-toolchain.toml"))?;
    if digest(&toolchain) != probe.manifest.toolchain_sha256 {
        return Err(invalid(
            "copied toolchain pin changed; retain authored inputs and create a new explicitly pinned probe",
        ));
    }
    if environment
        .get(OsStr::new("RUSTUP_TOOLCHAIN"))
        .is_some_and(|value| !value.is_empty())
    {
        return Err(invalid(
            "RUSTUP_TOOLCHAIN would override the copied pin; remove that override before running this probe",
        ));
    }
    let mut environment = correctness::standalone_adapter_environment(environment)?;
    environment.remove(OsStr::new("CARGO_TARGET_DIR"));
    environment.remove(OsStr::new("CARGO_BUILD_TARGET_DIR"));
    environment.remove(OsStr::new("CARGO_BUILD_BUILD_DIR"));
    let cargo = correctness::real_cargo_in(&environment)?;
    let observe = |program: &OsStr| -> Result<String, ProbeError> {
        let output = Command::new(program)
            .arg("-vV")
            .current_dir(&probe.directory)
            .env_clear()
            .envs(&environment)
            .output()?;
        if !output.status.success() {
            return Err(invalid(format!(
                "scratch tool identity exited {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        String::from_utf8(output.stdout).map_err(|_| invalid("scratch tool identity is not UTF-8"))
    };
    let rustc_identity = observe(
        environment
            .get(OsStr::new("RUSTC"))
            .map_or(OsStr::new("rustc"), |value| value.as_os_str()),
    )?;
    let cargo_identity = observe(&cargo)?;
    let output = Command::new(&cargo)
        .current_dir(&probe.directory)
        .env_clear()
        .envs(&environment)
        .args(["metadata", "--format-version", "1", "--offline"])
        .arg("--manifest-path")
        .arg(probe.directory.join("Cargo.toml"))
        .arg("--config")
        .arg(format!("build.target-dir={:?}", probe.target_directory))
        .output()?;
    let capture = probe
        .directory
        .join("runs")
        .join(uuid::Uuid::now_v7().to_string());
    directory(&capture)?;
    publish(&capture.join("metadata.stdout"), &output.stdout)?;
    publish(&capture.join("metadata.stderr"), &output.stderr)?;
    publish(&capture.join("rustc.identity"), rustc_identity.as_bytes())?;
    publish(&capture.join("cargo.identity"), cargo_identity.as_bytes())?;
    publish(&capture.join("rust-toolchain.toml"), &toolchain)?;
    if !output.status.success() {
        return Err(invalid(format!(
            "scratch resolution exited {}; original diagnostics retained at {}",
            output.status,
            capture.display()
        )));
    }
    let resolution = MetadataSnapshot {
        metadata: serde_json::from_slice(&output.stdout)?,
        digest: digest(&output.stdout),
    };
    let prepared = crate::generation::prepared(root)
        .map_err(|error| invalid(error.to_string()))?
        .ok_or_else(|| invalid("correctness tool unprepared; run scripts/bootstrap-host"))?;
    let executable = prepared
        .executable(root, crate::generation::HostTool::Correctness)
        .map_err(|error| invalid(error.to_string()))?;
    let mut command = vec![
        executable.into_os_string(),
        operation.into(),
        "--config".into(),
        format!("build.build-dir={:?}", probe.target_directory).into(),
        "--bin".into(),
        "pse-scratch-probe".into(),
        "--locked".into(),
        "--offline".into(),
        "--target-dir".into(),
        probe.target_directory.clone().into_os_string(),
    ];
    if !arguments.is_empty() {
        command.push("--".into());
        command.extend_from_slice(arguments);
    }
    Ok(Run {
        probe,
        resolution,
        resolution_directory: capture,
        rustc_identity,
        cargo_identity,
        command,
        environment,
        lease,
    })
}
#[derive(Deserialize)]
struct OriginalOperation {
    command: Vec<OsString>,
    cwd: OsString,
}
fn references(
    root: &Path,
    store: &Path,
    probe: &Probe,
    environment: &Environment,
) -> Result<Vec<RunReference>, ProbeError> {
    let runs = probe.directory.join("runs");
    let entries = match fs::read_dir(&runs) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(error) => return Err(error.into()),
    };
    let mut result = Vec::new();
    let root = root.canonicalize()?;
    for entry in entries {
        let path = entry?.path();
        let metadata = fs::symlink_metadata(&path)?;
        if !metadata.is_dir()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o077 != 0
        {
            return Err(invalid(
                "probe run capture is not an owned private directory",
            ));
        }
        if fs::read_dir(&path)?.next().is_none() {
            continue;
        }
        let bytes = crate::test_inventory::read_private(&path.join("managed-run.json")).map_err(
            |error| ProbeError::RunCustody {
                capture: path.clone(),
                source: Box::new(error),
            },
        )?;
        let reference: RunReference = serde_json::from_slice(&bytes)?;
        if reference.version != 1
            || Path::new(&reference.checkout) != root
            || Path::new(&reference.ledger_root) != store
            || Path::new(&reference.cwd) != probe.directory
            || reference.probe_manifest_sha256
                != digest(&fs::read(probe.directory.join("probe.json"))?)
            || !matches!(
                reference.command_name.as_str(),
                "probe run" | "compiler probe"
            )
        {
            return Err(invalid(
                "probe run reference differs from original checkout/probe selection",
            ));
        }
        let state = crate::paths::CheckoutState::resolve(&root, environment)
            .map_err(crate::HostError::from)
            .map_err(host)?;
        if state.root != store
            || state.checkout_id.as_str() != reference.checkout_id
            || state.repository_id.as_str() != reference.repository_id
        {
            return Err(invalid(
                "probe run reference differs from inspected checkout/ledger identity",
            ));
        }
        let original: crate::invocation::InvocationMetadata<OriginalOperation> =
            crate::invocation::InvocationStore::new(state)
                .metadata(&reference.invocation)
                .map_err(host)?;
        if original.selection.command != reference.command_name
            || original.selection.operation.cwd != reference.cwd
            || original.selection.operation.command != reference.command
            || original.selection.control != reference.control
        {
            return Err(invalid(
                "original invocation metadata does not bind this probe's exact cwd/argv/verb/control",
            ));
        }
        result.push(reference);
    }
    Ok(result)
}
async fn remove_with<F, Fut>(
    root: &Path,
    store: &Path,
    name: &str,
    environment: &Environment,
    observe: F,
) -> Result<PathBuf, ProbeError>
where
    F: FnMut(RunReference) -> Fut,
    Fut: Future<Output = Result<bool, ProbeError>>,
{
    let (probe, _lease) = open(store, name)?;
    let recorded = references(root, store, &probe, environment)?;
    let observed = futures_util::future::join_all(recorded.into_iter().map(observe)).await;
    for native in observed {
        if !native? {
            return Err(invalid(
                "an original probe native root remains live or unsettled, including an authenticated borrowed enclosing owner; explicit rm remains protected until that original root drains",
            ));
        }
    }
    refuse_live_processes(&probe.directory)?;
    fs::remove_dir_all(&probe.directory)?;
    Ok(probe.directory)
}
/// Remove a named probe only after the same lease excludes new supported runs and every
/// durable original run matches its ledger selection and authenticated native root drain.
/// Failed calculations can be drained; missing/legacy/unknown custody never grants deletion.
pub async fn rm(
    root: &Path,
    store: &Path,
    name: &str,
    environment: &Environment,
) -> Result<PathBuf, ProbeError> {
    remove_with(root, store, name, environment, |reference| async move {
        crate::foreground::observe_recorded_drain(
            Path::new(&reference.checkout),
            &reference.invocation,
            environment,
        )
        .await
        .map_err(host)
    })
    .await
}
fn refuse_live_processes(path: &Path) -> Result<(), ProbeError> {
    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        if !entry
            .file_name()
            .as_encoded_bytes()
            .iter()
            .all(u8::is_ascii_digit)
        {
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        if metadata.uid() != rustix::process::geteuid().as_raw() {
            continue;
        }
        for resource in ["cwd", "exe"] {
            match fs::read_link(entry.path().join(resource)) {
                Ok(target) if target.starts_with(path) => {
                    return Err(invalid(format!(
                        "probe has a live process {} using {resource}",
                        entry.file_name().to_string_lossy()
                    )));
                }
                Ok(_) => {}
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied
                    ) => {}
                Err(error) => return Err(error.into()),
            }
        }
        let descriptors = match fs::read_dir(entry.path().join("fd")) {
            Ok(descriptors) => descriptors,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied
                ) =>
            {
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        for descriptor in descriptors {
            match descriptor.and_then(|descriptor| fs::read_link(descriptor.path())) {
                Ok(target) if target.starts_with(path) => {
                    return Err(invalid(format!(
                        "probe has a live process {} holding an open file",
                        entry.file_name().to_string_lossy()
                    )));
                }
                Ok(_) => {}
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied
                    ) => {}
                Err(error) => return Err(error.into()),
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
