// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit native xtask capture; passive selection never compiles or reconstructs its parser.
mod process;
use crate::{
    catalog::description::{Availability, Description, Page},
    correctness,
    discovery::{BuildCapture, Executable, capture},
    environment::Environment,
    generation::{BuildContext, GenerationUse, SourceFingerprint},
    managed::payload::PayloadCommand,
    ownership::document::DocumentOwnership,
    paths::CheckoutState,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::{
    collections::BTreeSet,
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    io::{Read as _, Write as _},
    os::unix::fs::{DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

const MAX_DOCUMENT: usize = 8 * 1024 * 1024;
const MAX_MANIFEST: usize = 1024 * 1024;
const READ_TIMEOUT: Duration = Duration::from_secs(30);
const SHARED_INPUTS: &[&str] = &[
    "scripts/workspace.rs",
    "scripts/cli-catalog.rs",
    "scripts/document-ownership.rs",
    "scripts/generated-outputs.rs",
];

/// Original preparation/read failure, never an empty successful owner response.
#[derive(Debug, thiserror::Error)]
pub enum PreparedXtaskError {
    /// Original file/pipe failure.
    #[error("prepared xtask I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Original native external record decoding failure.
    #[error("prepared xtask JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Original compiler admission/capture refusal.
    #[error(transparent)]
    Correctness(#[from] correctness::CorrectnessError),
    /// Original generation/selection owner failure.
    #[error(transparent)]
    Host(#[from] crate::HostError),
    /// Original checkout-state selection failure.
    #[error(transparent)]
    Environment(#[from] crate::environment::EnvironmentError),
    /// Original native process ownership/settlement failure.
    #[error(transparent)]
    Supervision(#[from] pse_diagnostics::SupervisionError),
    /// Preserved independent failures after all owned drains and child settlement were attempted.
    #[error("prepared xtask child failed in {} independent responsibilities", failures.len())]
    Execution {
        /// Original independent failures.
        failures: Vec<PreparedXtaskError>,
        /// Original bounded stdout prefix.
        stdout: Vec<u8>,
        /// Original bounded stderr prefix.
        stderr: Vec<u8>,
    },
    /// Native command refusal retains original status and streams.
    #[error("prepared xtask {phase} exited {status}")]
    Command {
        /// Exact preparation or typed read phase.
        phase: &'static str,
        /// Original exit or signal encoding.
        status: i32,
        /// Original bounded stdout prefix.
        stdout: Vec<u8>,
        /// Original bounded stderr prefix.
        stderr: Vec<u8>,
    },
    /// A complete native read cannot be established from a truncated prefix.
    #[error("prepared xtask {phase} emitted {bytes} bytes beyond its complete-read extent")]
    Extent {
        /// Typed native read phase.
        phase: &'static str,
        /// Fully drained original stdout byte count.
        bytes: u64,
        /// Original retained stdout prefix.
        stdout: Vec<u8>,
        /// Original retained stderr prefix.
        stderr: Vec<u8>,
    },
    /// Build zero is independent of publishing the required native compiler capture.
    #[error("prepared xtask compiler evidence at {path}: {source}")]
    BuildEvidence {
        /// Fixed original callback destination.
        path: PathBuf,
        /// Original read/decoding cause.
        source: Box<PreparedXtaskError>,
    },
    /// Original capture operation and independent generation-use settlement failure.
    #[error(
        "prepared xtask control capture failed: {operation}; generation settlement failed: {settlement}"
    )]
    Settlement {
        /// Original operation.
        operation: Box<PreparedXtaskError>,
        /// Independent custody failure.
        settlement: crate::HostError,
    },
    /// Missing, stale or inconsistent evidence refuses use without repair.
    #[error("prepared xtask: {0}")]
    Invalid(String),
}
pse_diagnostics::impl_diagnostic! {
    PreparedXtaskError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::HostOwnership) },
    forward(this) { match this { Self::Host(error) => Some(error), Self::Environment(error) => Some(error), Self::Correctness(error) => Some(error), Self::Supervision(error) => Some(error), Self::BuildEvidence { source, .. } => Some(source.as_ref()), _ => None } },
    help(_this) { None }, related(this) { match this { Self::Execution { failures, .. } => Some(Box::new(failures.iter().map(|error| { let error: &dyn pse_diagnostics::TypedDiagnostic = error; error }))), Self::Settlement { operation, settlement } => { let errors: [&dyn pse_diagnostics::TypedDiagnostic;2] = [operation.as_ref(), settlement]; Some(Box::new(errors.into_iter())) }, _ => None } }, source(_this) { None }
}
fn invalid(message: impl Into<String>) -> PreparedXtaskError {
    PreparedXtaskError::Invalid(message.into())
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn private_directory(path: &Path, create: bool) -> Result<(), PreparedXtaskError> {
    if create {
        match fs::DirBuilder::new().mode(0o700).create(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }
    let metadata = fs::symlink_metadata(path)?;
    if !path.is_absolute()
        || !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o777 != 0o700
    {
        return Err(invalid(
            "selection directories must be absolute, private and owned",
        ));
    }
    Ok(())
}
fn read(path: &Path, maximum: usize) -> Result<Vec<u8>, PreparedXtaskError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.uid() != rustix::process::geteuid().as_raw() {
        return Err(invalid("selection evidence must be a regular owned file"));
    }
    let mut bytes = Vec::new();
    file.take(maximum as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(invalid("selection evidence exceeds its decoding extent"));
    }
    Ok(bytes)
}
fn publish(path: &Path, bytes: &[u8]) -> Result<(), PreparedXtaskError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Evidence {
    name: String,
    sha256: String,
    bytes: usize,
}
impl Evidence {
    fn new(name: &str, bytes: &[u8]) -> Self {
        Self {
            name: name.into(),
            sha256: hash(bytes),
            bytes: bytes.len(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u32,
    checkout_id: String,
    selection_id: String,
    executable: Executable,
    control: Executable,
    source: SourceFingerprint,
    context: BuildContext,
    features: BTreeSet<String>,
    build: Evidence,
    catalog: Evidence,
    owner_schema: Evidence,
}

/// Authenticated retained executable plus its original compiler/parser/schema evidence.
#[derive(Debug)]
pub struct PreparedXtask {
    checkout: PathBuf,
    directory: PathBuf,
    manifest: Manifest,
}
impl PreparedXtask {
    /// Immutable checkout-bound selection key, never an arbitrary executable path.
    pub fn id(&self) -> &str {
        &self.manifest.selection_id
    }
    /// Exact captured program identity.
    pub fn executable(&self) -> &Executable {
        &self.manifest.executable
    }
    /// Actual root features from the same compiler-artifact and admission evidence.
    pub fn features(&self) -> &BTreeSet<String> {
        &self.manifest.features
    }
    /// Original compiled source/context identity; no currentness claim follows from the digest.
    pub fn source_identity(&self) -> Result<String, PreparedXtaskError> {
        Ok(self.manifest.source.digest()?)
    }
    /// Open an existing selection using only original retained evidence and source reads.
    pub fn open(
        checkout: &Path,
        environment: &Environment,
        id: &str,
    ) -> Result<Self, PreparedXtaskError> {
        if id.len() != 36 || uuid::Uuid::parse_str(id).is_err() {
            return Err(invalid("invalid xtask selection key"));
        }
        let checkout = checkout.canonicalize()?;
        let state = CheckoutState::resolve(&checkout, environment)?;
        private_directory(&state.root, false)?;
        private_directory(&state.root.join("tools"), false)?;
        private_directory(&state.root.join("tools/xtask"), false)?;
        let directory = state.root.join("tools/xtask").join(id);
        private_directory(&directory, false)?;
        let manifest: Manifest =
            serde_json::from_slice(&read(&directory.join("xtask.json"), MAX_MANIFEST)?)?;
        if manifest.version != 1
            || manifest.checkout_id != state.checkout_id.as_str()
            || manifest.selection_id != id
        {
            return Err(invalid("xtask checkout/selection manifest mismatch"));
        }
        let selected = Self {
            checkout,
            directory,
            manifest,
        };
        selected.verify()?;
        Ok(selected)
    }
    fn evidence(
        &self,
        record: &Evidence,
        name: &str,
        maximum: usize,
    ) -> Result<Vec<u8>, PreparedXtaskError> {
        if record.name != name {
            return Err(invalid("evidence path differs from its fixed association"));
        }
        let bytes = read(&self.directory.join(name), maximum)?;
        if bytes.len() != record.bytes || hash(&bytes) != record.sha256 {
            return Err(invalid(
                "captured evidence differs from its original identity",
            ));
        }
        Ok(bytes)
    }
    fn verify(&self) -> Result<(), PreparedXtaskError> {
        private_directory(&self.directory, false)?;
        private_directory(&self.directory.join("build"), false)?;
        if self.manifest.control.captured != self.directory.join("cargo-pse-correctness") {
            return Err(invalid(
                "captured correctness control lies outside selection",
            ));
        }
        self.manifest.control.verify()?;
        let build: BuildCapture = serde_json::from_slice(&self.evidence(
            &self.manifest.build,
            "build/build.json",
            MAX_DOCUMENT,
        )?)?;
        let executable = select_executable(&build, &self.directory, &self.checkout)?;
        if executable != &self.manifest.executable {
            return Err(invalid(
                "selected executable differs from original compiler capture",
            ));
        }
        if self.manifest.features != BTreeSet::from(["force-validate".into()]) {
            return Err(invalid("unsupported captured xtask feature profile"));
        }
        executable.verify()?;
        catalog_document(&self.evidence(&self.manifest.catalog, "commands.json", MAX_DOCUMENT)?)?;
        schema_document(&self.evidence(
            &self.manifest.owner_schema,
            "owner-schema.json",
            MAX_DOCUMENT,
        )?)?;
        Ok(())
    }
    fn require_current_sources(&self) -> Result<(), PreparedXtaskError> {
        if !self
            .manifest
            .source
            .matches_sources(&self.checkout, &self.manifest.context)?
        {
            return Err(invalid(
                "xtask source association is stale; prepare a new selection explicitly",
            ));
        }
        Ok(())
    }
    /// Original native parser export. Missing per-verb metadata remains unavailable in it.
    pub fn catalog(&self) -> Result<Page<Description>, PreparedXtaskError> {
        self.verify()?;
        let mut page = catalog_document(&self.evidence(
            &self.manifest.catalog,
            "commands.json",
            MAX_DOCUMENT,
        )?)?;
        page.source.generation = Some(self.id().into());
        page.source.freshness = crate::catalog::description::Freshness::RetainedSnapshot;
        page.limitations.push("Captured native parser metadata; current source association and execution readiness are independently checked only before dispatch.".into());
        Ok(page)
    }
    /// Exact response schema emitted by the captured native document owner.
    pub fn owner_schema(&self) -> Result<serde_json::Value, PreparedXtaskError> {
        self.verify()?;
        schema_document(&self.evidence(
            &self.manifest.owner_schema,
            "owner-schema.json",
            MAX_DOCUMENT,
        )?)
    }
    /// Return exact process inputs for the shared managed owner; this starts no child.
    /// The caller retains admission, invocation and physical lifecycle responsibility.
    pub fn launch(
        &self,
        arguments: Vec<OsString>,
        environment: Environment,
    ) -> Result<PayloadCommand, PreparedXtaskError> {
        self.verify()?;
        self.require_current_sources()?;
        Ok(PayloadCommand {
            program: self.manifest.executable.captured.clone(),
            arguments,
            cwd: self.checkout.clone(),
            environment,
        })
    }
    /// Bounded typed read of the existing native document-owner protocol only.
    pub fn document_owner(
        &self,
        path: &Path,
        environment: &Environment,
    ) -> Result<DocumentOwnership, PreparedXtaskError> {
        self.verify()?;
        self.require_current_sources()?;
        let mut command = self.command(environment);
        command.arg("owner").arg("--").arg(path);
        let output = process::run(command, MAX_DOCUMENT, Some(READ_TIMEOUT))?;
        let bytes = successful(output, "document-owner", true)?;
        let document: DocumentOwnership = serde_json::from_slice(&bytes)?;
        let expected =
            crate::ownership::resolve(&self.checkout.join(path)).map_err(crate::HostError::from)?;
        if Path::new(&document.path) != expected {
            return Err(invalid(
                "native document response names another selected source",
            ));
        }
        Ok(document)
    }
    fn command(&self, environment: &Environment) -> Command {
        let mut command = Command::new(&self.manifest.executable.captured);
        command
            .current_dir(&self.checkout)
            .env_clear()
            .envs(environment);
        command
    }
}

fn catalog_document(bytes: &[u8]) -> Result<Page<Description>, PreparedXtaskError> {
    let page: Page<Description> = serde_json::from_slice(bytes)?;
    if page.availability != Availability::Available
        || page.source.owner != "compiled xtask command parser"
        || !page.records.iter().any(|record| record.path.is_empty())
        || !page.records.iter().any(|record| record.path == ["owner"])
    {
        return Err(invalid(
            "native parser discovery is incomplete or belongs to another owner",
        ));
    }
    Ok(page)
}
fn schema_document(bytes: &[u8]) -> Result<serde_json::Value, PreparedXtaskError> {
    let value: serde_json::Value = serde_json::from_slice(bytes)?;
    if !value.is_object()
        || !value
            .get("$schema")
            .is_some_and(serde_json::Value::is_string)
    {
        return Err(invalid(
            "native document-owner export is not a declared JSON Schema document",
        ));
    }
    Ok(value)
}
fn select_executable<'a>(
    build: &'a BuildCapture,
    directory: &Path,
    checkout: &Path,
) -> Result<&'a Executable, PreparedXtaskError> {
    if build.version != 1 || build.cargo_exit != 0 || !build.incomplete.is_empty() {
        return Err(invalid("compiler capture is failed or incomplete"));
    }
    let artifacts: Vec<_> = build
        .artifacts
        .iter()
        .enumerate()
        .filter(|(_, artifact)| {
            artifact.cargo.target.name == "xtask"
                && artifact
                    .cargo
                    .target
                    .kind
                    .iter()
                    .any(|kind| kind.to_string() == "bin")
        })
        .collect();
    let [(index, artifact)] = artifacts.as_slice() else {
        return Err(invalid(
            "compiler capture does not select exactly one xtask binary",
        ));
    };
    let features: BTreeSet<_> = artifact.cargo.features.iter().cloned().collect();
    if features != BTreeSet::from(["force-validate".into()])
        || artifact.executable.captured != directory.join("build").join(format!("artifact-{index}"))
        || artifact.cargo.target.src_path.as_std_path() != checkout.join("xtask/src/main.rs")
    {
        return Err(invalid(
            "compiler artifact differs from the bounded xtask source/feature/capture profile",
        ));
    }
    if !build.resolution.units.iter().any(|unit| {
        unit.root
            && unit.package_name == "xtask"
            && unit.package_id == artifact.cargo.package_id.repr
            && unit.target_name == "xtask"
            && unit.mode == "build"
            && unit.source_path == artifact.cargo.target.src_path.as_str()
            && unit.features == features
    }) {
        return Err(invalid(
            "admitted compiler units do not bind the same xtask artifact",
        ));
    }
    Ok(&artifact.executable)
}
fn successful(
    output: process::Output,
    phase: &'static str,
    complete: bool,
) -> Result<Vec<u8>, PreparedXtaskError> {
    if output.status != 0 {
        return Err(PreparedXtaskError::Command {
            phase,
            status: output.status,
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }
    if complete && output.stdout_bytes > output.stdout.len() as u64 {
        return Err(PreparedXtaskError::Extent {
            phase,
            bytes: output.stdout_bytes,
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }
    Ok(output.stdout)
}
fn source(
    checkout: &Path,
    environment: &Environment,
) -> Result<(SourceFingerprint, BuildContext), PreparedXtaskError> {
    let cargo = correctness::real_cargo_in(environment)?;
    let observed = |program: &OsStr| -> Result<String, PreparedXtaskError> {
        let mut command = Command::new(program);
        command
            .arg("-vV")
            .current_dir(checkout)
            .env_clear()
            .envs(environment);
        String::from_utf8(successful(
            process::run(command, 65536, Some(READ_TIMEOUT))?,
            "compiler-version",
            true,
        )?)
        .map_err(|_| invalid("compiler version is not UTF-8"))
    };
    let rustc = environment
        .get(OsStr::new("RUSTC"))
        .map_or(OsStr::new("rustc"), |value| value.as_os_str());
    let context = BuildContext::new(
        observed(rustc)?,
        observed(&cargo)?,
        "dev".into(),
        BTreeSet::from(["xtask/force-validate".into()]),
        environment,
    );
    let arguments: Vec<OsString> = [
        "metadata",
        "--format-version=1",
        "--locked",
        "--offline",
        "--no-default-features",
        "--features",
        "xtask/force-validate",
    ]
    .into_iter()
    .map(Into::into)
    .collect();
    let metadata: cargo_metadata::Metadata = serde_json::from_slice(&correctness::capture_at(
        &cargo,
        &arguments,
        checkout,
        environment,
    )?)?;
    let source =
        SourceFingerprint::from_metadata_for(checkout, &metadata, context.clone(), "xtask")?
            .with_additional_files(checkout, SHARED_INPUTS.iter().map(PathBuf::from))?;
    Ok((source, context))
}
fn source_build_command(
    control: &Executable,
    checkout: &Path,
    environment: &Environment,
) -> Command {
    let mut command = Command::new(&control.captured);
    command
        .current_dir(checkout)
        .env_clear()
        .envs(environment)
        .env("PSE_CORRECTNESS_ADAPTER", "1")
        .env("PSE_CORRECTNESS_EXECUTABLE", &control.captured)
        .args([
            "build",
            "--locked",
            "-p",
            "xtask",
            "--bin",
            "xtask",
            "--no-default-features",
            "--features",
            "force-validate",
            "--message-format=json",
        ]);
    command
}
fn build_evidence(directory: &Path) -> Result<(Vec<u8>, BuildCapture), PreparedXtaskError> {
    let path = directory.join("build.json");
    let result = (|| {
        let bytes = read(&path, MAX_DOCUMENT)?;
        let build = serde_json::from_slice(&bytes)?;
        Ok((bytes, build))
    })();
    result.map_err(|source| PreparedXtaskError::BuildEvidence {
        path,
        source: Box::new(source),
    })
}

/// Explicit correctness-admitted preparation. Failed evidence remains at its minted selection.
/// The caller owns new-work admission; passive selection never calls this route.
pub fn prepare(
    checkout: &Path,
    environment: &Environment,
) -> Result<PreparedXtask, PreparedXtaskError> {
    let checkout = checkout.canonicalize()?;
    let state = CheckoutState::resolve(&checkout, environment)?;
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&state.root)?;
    private_directory(&state.root, false)?;
    private_directory(&state.root.join("tools"), true)?;
    private_directory(&state.root.join("tools/xtask"), true)?;
    let id = uuid::Uuid::new_v4().to_string();
    let selected_dir = state.root.join("tools/xtask").join(&id);
    private_directory(&selected_dir, true)?;
    let build_dir = selected_dir.join("build");
    private_directory(&build_dir, true)?;
    let prepared = crate::generation::prepared(&checkout)?.ok_or_else(|| {
        invalid("correctness control is unprepared; run scripts/bootstrap-host explicitly")
    })?;
    let tools = checkout.join("target/host-tools");
    let usage = GenerationUse::acquire(&tools, &prepared.generation)?;
    let captured = capture::executable(
        &tools
            .join(&prepared.generation)
            .join("cargo-pse-correctness"),
        &selected_dir.join("cargo-pse-correctness"),
    )
    .map_err(PreparedXtaskError::from);
    let control = match (captured, usage.settle()) {
        (value, Ok(())) => value?,
        (Ok(_), Err(error)) => return Err(error.into()),
        (Err(operation), Err(settlement)) => {
            return Err(PreparedXtaskError::Settlement {
                operation: Box::new(operation),
                settlement,
            });
        }
    };
    let mut environment = correctness::standalone_adapter_environment(environment)?;
    let (before, context) = source(&checkout, &environment)?;
    environment.insert(
        "PSE_CORRECTNESS_BUILD_CAPTURE".into(),
        build_dir.clone().into_os_string(),
    );
    control.verify()?;
    let command = source_build_command(&control, &checkout, &environment);
    successful(process::run(command, 65536, None)?, "source-build", false)?;
    let (build_bytes, build) = build_evidence(&build_dir)?;
    let executable = select_executable(&build, &selected_dir, &checkout)?.clone();
    executable.verify()?;
    environment.remove(OsStr::new("PSE_CORRECTNESS_BUILD_CAPTURE"));
    let query = |args: &[&str], name: &str, phase| -> Result<Vec<u8>, PreparedXtaskError> {
        let mut command = Command::new(&executable.captured);
        command
            .current_dir(&checkout)
            .env_clear()
            .envs(&environment)
            .args(args);
        let output = process::run(command, MAX_DOCUMENT, Some(READ_TIMEOUT))?;
        publish(&selected_dir.join(name), &output.stdout)?;
        publish(&selected_dir.join(format!("{name}.stderr")), &output.stderr)?;
        successful(output, phase, true)
    };
    let catalog = query(
        &["commands", "--json", "--recursive"],
        "commands.json",
        "parser-discovery",
    )?;
    catalog_document(&catalog)?;
    let schema = query(&["owner", "--schema"], "owner-schema.json", "owner-schema")?;
    schema_document(&schema)?;
    let (after, _) = source(&checkout, &environment)?;
    before.require_unchanged(&after)?;
    let selected = PreparedXtask {
        checkout,
        directory: selected_dir,
        manifest: Manifest {
            version: 1,
            checkout_id: state.checkout_id.as_str().into(),
            selection_id: id,
            executable,
            control,
            source: before,
            context,
            features: BTreeSet::from(["force-validate".into()]),
            build: Evidence::new("build/build.json", &build_bytes),
            catalog: Evidence::new("commands.json", &catalog),
            owner_schema: Evidence::new("owner-schema.json", &schema),
        },
    };
    selected.verify()?;
    let bytes = serde_json::to_vec(&selected.manifest)?;
    if bytes.len() > MAX_MANIFEST {
        return Err(invalid(
            "selection manifest exceeds supported decoding extent",
        ));
    }
    publish(&selected.directory.join("xtask.json"), &bytes)?;
    Ok(selected)
}

#[cfg(test)]
mod tests;
