// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit native tool capture; passive selection never compiles or reconstructs a target.
mod process;
use crate::{
    catalog::description::{Availability, Description, Page},
    correctness,
    discovery::{BuildCapture, Executable, capture},
    environment::Environment,
    generation::{BuildContext, SourceFingerprint},
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

/// A source-built immutable helper, with one authoritative artifact/feature profile.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    schemars::JsonSchema,
    clap::ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
pub enum NativeToolTarget {
    /// Existing development composition and document owner.
    #[default]
    Xtask,
    /// Explicitly prepared native HIR/IDE semantic service.
    NativeAnalysis,
}
struct ToolProfile {
    name: &'static str,
    package: &'static str,
    binary: &'static str,
    source: &'static str,
    features: &'static [&'static str],
}
impl NativeToolTarget {
    fn profile(self) -> ToolProfile {
        match self {
            Self::Xtask => ToolProfile {
                name: "xtask",
                package: "xtask",
                binary: "xtask",
                source: "xtask/src/main.rs",
                features: &["force-validate"],
            },
            Self::NativeAnalysis => ToolProfile {
                name: "native-analysis",
                package: "pse-host",
                binary: "pse-native-analysis",
                source: "crates/pse-host/src/bin/pse-native-analysis.rs",
                features: &["native-hir"],
            },
        }
    }
    /// Fixed preparation namespace, also used by the command grammar.
    pub fn name(self) -> &'static str {
        self.profile().name
    }
    fn features(self) -> BTreeSet<String> {
        self.profile()
            .features
            .iter()
            .map(|f| (*f).into())
            .collect()
    }
    fn manifest(self) -> String {
        format!("{}.json", self.name())
    }
}
impl std::fmt::Display for NativeToolTarget {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Original preparation/read failure, never an empty successful owner response.
#[derive(Debug, thiserror::Error)]
pub enum PreparedNativeToolError {
    /// Original file/pipe failure.
    #[error("prepared native tool I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Original native external record decoding failure.
    #[error("prepared native tool JSON: {0}")]
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
    #[error("prepared native tool child failed in {} independent responsibilities", failures.len())]
    Execution {
        /// Original independent failures.
        failures: Vec<PreparedNativeToolError>,
        /// Original bounded stdout prefix.
        stdout: Vec<u8>,
        /// Original bounded stderr prefix.
        stderr: Vec<u8>,
    },
    /// Native command refusal retains original status and streams.
    #[error("prepared native tool {phase} exited {status}")]
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
    #[error("prepared native tool {phase} emitted {bytes} bytes beyond its complete-read extent")]
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
    #[error("prepared native tool compiler evidence at {path}: {source}")]
    BuildEvidence {
        /// Fixed original callback destination.
        path: PathBuf,
        /// Original read/decoding cause.
        source: Box<PreparedNativeToolError>,
    },
    /// Passive current-selection failure retains the original pointer/open cause.
    #[error(
        "current {target} selection at {path} is unavailable; explicitly run pse tools prepare {target}: {source}"
    )]
    CurrentSelection {
        /// Actual immutable tool target which must be explicitly prepared.
        target: NativeToolTarget,
        /// Fixed checkout-private current selector, never an arbitrary executable path.
        path: PathBuf,
        /// Original source/manifest/association failure.
        #[source]
        source: Box<PreparedNativeToolError>,
    },
    /// Missing, stale or inconsistent evidence refuses use without repair.
    #[error("prepared native tool: {0}")]
    Invalid(String),
}
pse_diagnostics::impl_diagnostic! {
    PreparedNativeToolError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::HostOwnership) },
    forward(this) { match this { Self::Host(error) => Some(error), Self::Environment(error) => Some(error), Self::Correctness(error) => Some(error), Self::Supervision(error) => Some(error), Self::BuildEvidence { source, .. } | Self::CurrentSelection {source,..} => Some(source.as_ref()), _ => None } },
    help(_this) { None }, related(this) { match this { Self::Execution { failures, .. } => Some(Box::new(failures.iter().map(|error| { let error: &dyn pse_diagnostics::TypedDiagnostic = error; error }))), _ => None } }, source(_this) { None },
    facts(this) { this.command_facts() }
}
impl PreparedNativeToolError {
    fn command_facts(&self) -> pse_diagnostics::DiagnosticFacts {
        use pse_diagnostics::DiagnosticObservation as Observation;
        let mut facts = pse_diagnostics::DiagnosticFacts::default();
        let (stdout, stderr) = match self {
            Self::Command {
                phase,
                status,
                stdout,
                stderr,
            } => {
                facts.observe("phase", Observation::Text((*phase).into()));
                facts.observe("status", Observation::Integer(i64::from(*status)));
                (stdout, stderr)
            }
            Self::Extent {
                phase,
                bytes,
                stdout,
                stderr,
            } => {
                facts.observe("phase", Observation::Text((*phase).into()));
                facts.observe(
                    "observed_stdout_bytes",
                    Observation::Text(bytes.to_string()),
                );
                (stdout, stderr)
            }
            Self::Execution { stdout, stderr, .. } => (stdout, stderr),
            _ => return facts,
        };
        // These are explicitly lossy text projections of already bounded original
        // prefixes. The envelope's normal redaction applies to both strings.
        facts.observe(
            "stdout_prefix_utf8_lossy",
            Observation::Text(String::from_utf8_lossy(stdout).into_owned()),
        );
        facts.observe(
            "stderr_prefix_utf8_lossy",
            Observation::Text(String::from_utf8_lossy(stderr).into_owned()),
        );
        facts
    }
}
fn invalid(message: impl Into<String>) -> PreparedNativeToolError {
    PreparedNativeToolError::Invalid(message.into())
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn private_directory(path: &Path, create: bool) -> Result<(), PreparedNativeToolError> {
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
fn read(path: &Path, maximum: usize) -> Result<Vec<u8>, PreparedNativeToolError> {
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
fn publish(path: &Path, bytes: &[u8]) -> Result<(), PreparedNativeToolError> {
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
    #[serde(default)]
    target: NativeToolTarget,
    checkout_id: String,
    selection_id: String,
    executable: Executable,
    control: Executable,
    source: SourceFingerprint,
    context: BuildContext,
    features: BTreeSet<String>,
    build: Evidence,
    catalog: Option<Evidence>,
    owner_schema: Option<Evidence>,
    #[serde(default)]
    protocol: Option<Evidence>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CurrentSelection {
    version: u32,
    checkout_id: String,
    selection_id: String,
}

/// Authenticated retained executable plus its original compiler/parser/schema evidence.
#[derive(Debug)]
pub struct PreparedNativeTool {
    checkout: PathBuf,
    directory: PathBuf,
    manifest: Manifest,
}
/// Complete original native read streams after the exact helper and all readers settled.
#[derive(Debug)]
pub(crate) struct NativeRead {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub status: i32,
}
/// Recognize explicit standard owner read flags only before a literal argument tail.
pub(crate) fn passive_read(arguments: &[OsString]) -> bool {
    arguments
        .iter()
        .take_while(|argument| argument.as_os_str() != "--")
        .any(|argument| argument == "--schema" || argument == "--help" || argument == "-h")
}
impl PreparedNativeTool {
    /// Run a known passive native docs read without source preparation or a managed wrapper.
    pub(crate) fn read_passive_docs(
        &self,
        arguments: Vec<OsString>,
        environment: &Environment,
    ) -> Result<NativeRead, PreparedNativeToolError> {
        if self.manifest.target != NativeToolTarget::Xtask
            || arguments.first().is_none_or(|argument| argument != "docs")
            || !passive_read(&arguments)
        {
            return Err(invalid(
                "native passive docs reads require explicit help/schema before a literal argument tail",
            ));
        }
        self.verify()?;
        // Help/schema describe this captured executable. Source freshness remains a separate
        // dispatch requirement for work, as it already is for retained parser discovery.
        let mut command = self.command(environment);
        command.args(arguments);
        let output = process::run(command, MAX_DOCUMENT, Some(READ_TIMEOUT))?;
        if output.stdout_bytes > MAX_DOCUMENT as u64 {
            return Err(PreparedNativeToolError::Extent {
                phase: "passive-docs",
                bytes: output.stdout_bytes,
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }
        Ok(NativeRead {
            stdout: output.stdout,
            stderr: output.stderr,
            status: output.status,
        })
    }
    /// Read the explicitly prepared current selection without compiling, creating or repairing it.
    pub fn current(
        checkout: &Path,
        environment: &Environment,
    ) -> Result<Self, PreparedNativeToolError> {
        Self::current_for(checkout, environment, NativeToolTarget::Xtask)
    }
    /// Select an existing target-specific helper without compiling or repairing it.
    pub fn current_for(
        checkout: &Path,
        environment: &Environment,
        target: NativeToolTarget,
    ) -> Result<Self, PreparedNativeToolError> {
        let checkout = checkout.canonicalize()?;
        let state = CheckoutState::resolve(&checkout, environment)?;
        let path = state
            .root
            .join("tools")
            .join(target.name())
            .join("current.json");
        let operation = (|| {
            private_directory(&state.root, false)?;
            private_directory(&state.root.join("tools"), false)?;
            private_directory(&state.root.join("tools").join(target.name()), false)?;
            let current: CurrentSelection = serde_json::from_slice(&read(&path, 4096)?)?;
            if current.version != 1 || current.checkout_id != state.checkout_id.as_str() {
                return Err(invalid(
                    "current native tool pointer belongs to another checkout or unsupported version",
                ));
            }
            Self::open_for(&checkout, environment, &current.selection_id, target)
        })();
        operation.map_err(|source| PreparedNativeToolError::CurrentSelection {
            target,
            path,
            source: Box::new(source),
        })
    }
    fn select_current(&self) -> Result<(), PreparedNativeToolError> {
        self.verify()?;
        let parent = self
            .directory
            .parent()
            .ok_or_else(|| invalid("native tool selection parent absent"))?;
        private_directory(parent, false)?;
        let path = parent.join("current.json");
        match fs::symlink_metadata(&path) {
            Ok(metadata)
                if !metadata.is_file() || metadata.uid() != rustix::process::geteuid().as_raw() =>
            {
                return Err(invalid(
                    "current native tool selector must be a regular owned file",
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let value = CurrentSelection {
            version: 1,
            checkout_id: self.manifest.checkout_id.clone(),
            selection_id: self.id().into(),
        };
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(&serde_json::to_vec(&value)?)?;
        let mut permissions = temporary.as_file().metadata()?.permissions();
        permissions.set_readonly(true);
        temporary.as_file().set_permissions(permissions)?;
        temporary.as_file().sync_all()?;
        temporary.persist(&path).map_err(|error| error.error)?;
        fs::File::open(parent)?.sync_all()?;
        Ok(())
    }
    /// Immutable checkout-bound selection key, never an arbitrary executable path.
    pub fn id(&self) -> &str {
        &self.manifest.selection_id
    }
    /// Actual authenticated target, never inferred from an executable filename.
    pub fn target(&self) -> NativeToolTarget {
        self.manifest.target
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
    pub fn source_identity(&self) -> Result<String, PreparedNativeToolError> {
        Ok(self.manifest.source.digest()?)
    }
    /// Open an existing selection using only original retained evidence and source reads.
    pub fn open(
        checkout: &Path,
        environment: &Environment,
        id: &str,
    ) -> Result<Self, PreparedNativeToolError> {
        Self::open_for(checkout, environment, id, NativeToolTarget::Xtask)
    }
    /// Authenticate a selected helper against its target-specific source/artifact profile.
    pub fn open_for(
        checkout: &Path,
        environment: &Environment,
        id: &str,
        target: NativeToolTarget,
    ) -> Result<Self, PreparedNativeToolError> {
        if id.len() != 36 || uuid::Uuid::parse_str(id).is_err() {
            return Err(invalid("invalid native tool selection key"));
        }
        let checkout = checkout.canonicalize()?;
        let state = CheckoutState::resolve(&checkout, environment)?;
        private_directory(&state.root, false)?;
        private_directory(&state.root.join("tools"), false)?;
        private_directory(&state.root.join("tools").join(target.name()), false)?;
        let directory = state.root.join("tools").join(target.name()).join(id);
        private_directory(&directory, false)?;
        let manifest: Manifest =
            serde_json::from_slice(&read(&directory.join(target.manifest()), MAX_MANIFEST)?)?;
        if manifest.version != 1
            || manifest.checkout_id != state.checkout_id.as_str()
            || manifest.selection_id != id
            || manifest.target != target
        {
            return Err(invalid("native tool checkout/selection manifest mismatch"));
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
    ) -> Result<Vec<u8>, PreparedNativeToolError> {
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
    fn verify(&self) -> Result<(), PreparedNativeToolError> {
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
        let executable = select_executable_for(
            &build,
            &self.directory,
            &self.checkout,
            self.manifest.target,
        )?;
        if executable != &self.manifest.executable {
            return Err(invalid(
                "selected executable differs from original compiler capture",
            ));
        }
        if self.manifest.features != self.manifest.target.features() {
            return Err(invalid("unsupported captured native tool feature profile"));
        }
        executable.verify()?;
        match self.manifest.target {
            NativeToolTarget::Xtask => {
                if self.manifest.protocol.is_some() {
                    return Err(invalid(
                        "xtask cannot carry native-analysis protocol evidence",
                    ));
                }
                catalog_document(
                    &self.evidence(
                        self.manifest
                            .catalog
                            .as_ref()
                            .ok_or_else(|| invalid("xtask parser evidence absent"))?,
                        "commands.json",
                        MAX_DOCUMENT,
                    )?,
                )?;
                schema_document(
                    &self.evidence(
                        self.manifest
                            .owner_schema
                            .as_ref()
                            .ok_or_else(|| invalid("xtask owner schema absent"))?,
                        "owner-schema.json",
                        MAX_DOCUMENT,
                    )?,
                )?;
            }
            NativeToolTarget::NativeAnalysis => {
                if self.manifest.catalog.is_some() || self.manifest.owner_schema.is_some() {
                    return Err(invalid(
                        "native analysis is not an xtask parser/document owner",
                    ));
                }
                protocol_document(
                    &self.evidence(
                        self.manifest
                            .protocol
                            .as_ref()
                            .ok_or_else(|| invalid("native semantic protocol evidence absent"))?,
                        "protocol.json",
                        MAX_DOCUMENT,
                    )?,
                )?;
            }
        }
        Ok(())
    }
    fn require_current_sources(&self) -> Result<(), PreparedNativeToolError> {
        if !self
            .manifest
            .source
            .matches_sources(&self.checkout, &self.manifest.context)?
        {
            return Err(invalid(
                "native tool source association is stale; prepare a new selection explicitly",
            ));
        }
        Ok(())
    }
    /// Original native parser export. Missing per-verb metadata remains unavailable in it.
    pub fn catalog(&self) -> Result<Page<Description>, PreparedNativeToolError> {
        self.verify()?;
        let mut page = catalog_document(&self.evidence(
            self.manifest.catalog.as_ref().ok_or_else(|| {
                invalid("this target exposes its fixed semantic protocol, not an xtask catalog")
            })?,
            "commands.json",
            MAX_DOCUMENT,
        )?)?;
        page.source.generation = Some(self.id().into());
        page.source.freshness = crate::catalog::description::Freshness::RetainedSnapshot;
        page.limitations.push("Captured native parser metadata; current source association and execution readiness are independently checked only before dispatch.".into());
        Ok(page)
    }
    /// Exact response schema emitted by the captured native document owner.
    pub fn owner_schema(&self) -> Result<serde_json::Value, PreparedNativeToolError> {
        self.verify()?;
        schema_document(
            &self.evidence(
                self.manifest
                    .owner_schema
                    .as_ref()
                    .ok_or_else(|| invalid("this target has no document-owner response schema"))?,
                "owner-schema.json",
                MAX_DOCUMENT,
            )?,
        )
    }
    /// Original fixed semantic request/configuration/reply schemas from the captured helper.
    pub fn protocol(&self) -> Result<serde_json::Value, PreparedNativeToolError> {
        self.verify()?;
        protocol_document(
            &self.evidence(
                self.manifest
                    .protocol
                    .as_ref()
                    .ok_or_else(|| invalid("this target has no native semantic protocol"))?,
                "protocol.json",
                MAX_DOCUMENT,
            )?,
        )
    }
    /// Return exact process inputs for the shared managed owner; this starts no child.
    /// The caller retains admission, invocation and physical lifecycle responsibility.
    pub fn launch(
        &self,
        arguments: Vec<OsString>,
        environment: Environment,
    ) -> Result<PayloadCommand, PreparedNativeToolError> {
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
    ) -> Result<DocumentOwnership, PreparedNativeToolError> {
        if self.manifest.target != NativeToolTarget::Xtask {
            return Err(invalid("document ownership requires the xtask target"));
        }
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

fn catalog_document(bytes: &[u8]) -> Result<Page<Description>, PreparedNativeToolError> {
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
fn schema_document(bytes: &[u8]) -> Result<serde_json::Value, PreparedNativeToolError> {
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
fn protocol_document(bytes: &[u8]) -> Result<serde_json::Value, PreparedNativeToolError> {
    let value: serde_json::Value = serde_json::from_slice(bytes)?;
    if value.get("version").and_then(serde_json::Value::as_u64) != Some(1)
        || value.get("tool").and_then(serde_json::Value::as_str) != Some("native-analysis")
    {
        return Err(invalid(
            "native semantic protocol belongs to another target or version",
        ));
    }
    for name in ["request_schema", "configuration_schema", "response_schema"] {
        let schema = value
            .get(name)
            .ok_or_else(|| invalid(format!("native semantic protocol omits {name}")))?;
        if !schema.is_object()
            || !schema
                .get("$schema")
                .is_some_and(serde_json::Value::is_string)
        {
            return Err(invalid(
                "native semantic protocol requires actual directional JSON Schemas",
            ));
        }
    }
    Ok(value)
}
#[cfg(test)]
fn select_executable<'a>(
    build: &'a BuildCapture,
    directory: &Path,
    checkout: &Path,
) -> Result<&'a Executable, PreparedNativeToolError> {
    select_executable_for(build, directory, checkout, NativeToolTarget::Xtask)
}
fn select_executable_for<'a>(
    build: &'a BuildCapture,
    directory: &Path,
    checkout: &Path,
    target: NativeToolTarget,
) -> Result<&'a Executable, PreparedNativeToolError> {
    let profile = target.profile();
    if build.version != 1 || build.cargo_exit != 0 || !build.incomplete.is_empty() {
        return Err(invalid("compiler capture is failed or incomplete"));
    }
    let artifacts: Vec<_> = build
        .artifacts
        .iter()
        .enumerate()
        .filter(|(_, artifact)| {
            artifact.cargo.target.name == profile.binary
                && artifact
                    .cargo
                    .target
                    .kind
                    .iter()
                    .any(|kind| kind.to_string() == "bin")
        })
        .collect();
    let [(index, artifact)] = artifacts.as_slice() else {
        return Err(invalid(format!(
            "compiler capture does not select exactly one {} binary",
            profile.binary
        )));
    };
    let features: BTreeSet<_> = artifact.cargo.features.iter().cloned().collect();
    let original = artifact.cargo.executable.as_ref().ok_or_else(|| {
        invalid("selected native tool compiler artifact has no original executable")
    })?;
    if features != target.features()
        || artifact.executable.original != original.as_std_path()
        || artifact.executable.captured
            != capture::artifact_path(&directory.join("build"), *index, original.as_std_path())?
        || artifact.cargo.target.src_path.as_std_path() != checkout.join(profile.source)
    {
        return Err(invalid(
            "compiler artifact differs from the selected native tool source/feature/capture profile",
        ));
    }
    if !build.resolution.units.iter().any(|unit| {
        unit.root
            && unit.package_name == profile.package
            && unit.package_id == artifact.cargo.package_id.repr
            && unit.target_name == profile.binary
            && unit.mode == "build"
            && unit.source_path == artifact.cargo.target.src_path.as_str()
            && unit.features == features
    }) {
        return Err(invalid(
            "admitted compiler units do not bind the same native tool artifact",
        ));
    }
    Ok(&artifact.executable)
}
fn successful(
    output: process::Output,
    phase: &'static str,
    complete: bool,
) -> Result<Vec<u8>, PreparedNativeToolError> {
    if output.status != 0 {
        return Err(PreparedNativeToolError::Command {
            phase,
            status: output.status,
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }
    if complete && output.stdout_bytes > output.stdout.len() as u64 {
        return Err(PreparedNativeToolError::Extent {
            phase,
            bytes: output.stdout_bytes,
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }
    Ok(output.stdout)
}
fn source_for(
    checkout: &Path,
    environment: &Environment,
    target: NativeToolTarget,
) -> Result<(SourceFingerprint, BuildContext, correctness::Resolution), PreparedNativeToolError> {
    let selected_environment = correctness::compiler_environment(checkout, environment)?;
    let environment = &selected_environment;
    let cargo = correctness::real_cargo_in(environment)?;
    let observed = |program: &OsStr| -> Result<String, PreparedNativeToolError> {
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
        target
            .profile()
            .features
            .iter()
            .map(|feature| format!("{}/{feature}", target.profile().package))
            .collect(),
        environment,
    );
    let scoped_features = target
        .profile()
        .features
        .iter()
        .map(|feature| format!("{}/{feature}", target.profile().package))
        .collect::<Vec<_>>()
        .join(",");
    let arguments: Vec<OsString> = [
        "metadata",
        "--format-version=1",
        "--locked",
        "--offline",
        "--no-default-features",
        "--features",
        &scoped_features,
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
    let arguments: Vec<OsString> = source_build_arguments_for(target)
        .iter()
        .map(OsString::from)
        .collect();
    let resolution = correctness::resolve_with(&arguments, |arguments| {
        correctness::capture_at(&cargo, arguments, checkout, environment)
    })?;
    let source =
        SourceFingerprint::from_resolution(checkout, &metadata, context.clone(), &resolution)?
            .with_additional_files(checkout, SHARED_INPUTS.iter().map(PathBuf::from))?;
    Ok((source, context, resolution))
}
fn source_build_arguments_for(target: NativeToolTarget) -> Vec<String> {
    let profile = target.profile();
    [
        "build",
        "--locked",
        "-p",
        profile.package,
        "--bin",
        profile.binary,
        "--no-default-features",
        "--features",
        &profile.features.join(","),
        "--message-format=json-render-diagnostics",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}
#[cfg(test)]
fn source_build_command(
    control: &Executable,
    checkout: &Path,
    environment: &Environment,
) -> Command {
    source_build_command_for(control, checkout, environment, NativeToolTarget::Xtask)
}
fn source_build_command_for(
    control: &Executable,
    checkout: &Path,
    environment: &Environment,
    target: NativeToolTarget,
) -> Command {
    let mut command = Command::new(&control.captured);
    command
        .current_dir(checkout)
        .env_clear()
        .envs(environment)
        .env("PSE_CORRECTNESS_ADAPTER", "1")
        .env("PSE_CORRECTNESS_EXECUTABLE", &control.captured)
        .args(source_build_arguments_for(target));
    command
}
fn require_same_compiler_context(
    before: &correctness::Resolution,
    after: &correctness::Resolution,
) -> Result<(), PreparedNativeToolError> {
    if before.arguments != after.arguments
        || before.packages != after.packages
        || before.native_capabilities != after.native_capabilities
        || before.units.len() != after.units.len()
        || before.units.iter().any(|unit| !after.units.contains(unit))
    {
        return Err(invalid(
            "native tool compiler-unit selection changed during construction; explicitly re-resolve",
        ));
    }
    Ok(())
}
fn build_evidence(directory: &Path) -> Result<(Vec<u8>, BuildCapture), PreparedNativeToolError> {
    let path = directory.join("build.json");
    let result = (|| {
        let bytes = read(&path, MAX_DOCUMENT)?;
        let build = serde_json::from_slice(&bytes)?;
        Ok((bytes, build))
    })();
    result.map_err(|source| PreparedNativeToolError::BuildEvidence {
        path,
        source: Box::new(source),
    })
}

/// Explicit correctness-admitted preparation. Failed evidence remains at its minted selection.
/// The caller owns new-work admission; passive selection never calls this route.
pub fn prepare(
    checkout: &Path,
    environment: &Environment,
) -> Result<PreparedNativeTool, PreparedNativeToolError> {
    prepare_for(checkout, environment, NativeToolTarget::Xtask)
}
/// Explicitly capture exactly one declared target with its actual compiler and protocol evidence.
pub fn prepare_for(
    checkout: &Path,
    environment: &Environment,
    target: NativeToolTarget,
) -> Result<PreparedNativeTool, PreparedNativeToolError> {
    let checkout = checkout.canonicalize()?;
    let state = CheckoutState::resolve(&checkout, environment)?;
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&state.root)?;
    private_directory(&state.root, false)?;
    private_directory(&state.root.join("tools"), true)?;
    private_directory(&state.root.join("tools").join(target.name()), true)?;
    let id = uuid::Uuid::new_v4().to_string();
    let selected_dir = state.root.join("tools").join(target.name()).join(&id);
    private_directory(&selected_dir, true)?;
    let build_dir = selected_dir.join("build");
    private_directory(&build_dir, true)?;
    let prepared = crate::generation::prepared(&checkout)?.ok_or_else(|| {
        invalid("correctness control is unprepared; run scripts/bootstrap-host explicitly")
    })?;
    let source_control =
        prepared.executable(&checkout, crate::generation::HostTool::Correctness)?;
    let control =
        capture::executable(&source_control, &selected_dir.join("cargo-pse-correctness"))?;
    let mut environment = correctness::compiler_environment(
        &checkout,
        &correctness::standalone_adapter_environment(environment)?,
    )?;
    let (before, context, admission) = source_for(&checkout, &environment, target)?;
    environment.insert(
        "PSE_CORRECTNESS_BUILD_CAPTURE".into(),
        build_dir.clone().into_os_string(),
    );
    control.verify()?;
    let command = source_build_command_for(&control, &checkout, &environment, target);
    successful(process::run(command, 65536, None)?, "source-build", false)?;
    let (build_bytes, build) = build_evidence(&build_dir)?;
    require_same_compiler_context(&admission, &build.resolution)?;
    let executable = select_executable_for(&build, &selected_dir, &checkout, target)?.clone();
    executable.verify()?;
    environment.remove(OsStr::new("PSE_CORRECTNESS_BUILD_CAPTURE"));
    let query = |args: &[&str], name: &str, phase| -> Result<Vec<u8>, PreparedNativeToolError> {
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
    let (catalog, schema, protocol) = if target == NativeToolTarget::Xtask {
        let catalog = query(
            &["commands", "--json", "--recursive"],
            "commands.json",
            "parser-discovery",
        )?;
        catalog_document(&catalog)?;
        let schema = query(&["owner", "--schema"], "owner-schema.json", "owner-schema")?;
        schema_document(&schema)?;
        (Some(catalog), Some(schema), None)
    } else {
        let protocol = query(&["--protocol"], "protocol.json", "native-semantic-protocol")?;
        protocol_document(&protocol)?;
        (None, None, Some(protocol))
    };
    let (after, _, admission_after) = source_for(&checkout, &environment, target)?;
    require_same_compiler_context(&build.resolution, &admission_after)?;
    before.require_unchanged(&after)?;
    let selected = PreparedNativeTool {
        checkout,
        directory: selected_dir,
        manifest: Manifest {
            version: 1,
            target,
            checkout_id: state.checkout_id.as_str().into(),
            selection_id: id,
            executable,
            control,
            source: before,
            context,
            features: target.features(),
            build: Evidence::new("build/build.json", &build_bytes),
            catalog: catalog
                .as_ref()
                .map(|bytes| Evidence::new("commands.json", bytes)),
            owner_schema: schema
                .as_ref()
                .map(|bytes| Evidence::new("owner-schema.json", bytes)),
            protocol: protocol
                .as_ref()
                .map(|bytes| Evidence::new("protocol.json", bytes)),
        },
    };
    selected.verify()?;
    let bytes = serde_json::to_vec(&selected.manifest)?;
    if bytes.len() > MAX_MANIFEST {
        return Err(invalid(
            "selection manifest exceeds supported decoding extent",
        ));
    }
    publish(&selected.directory.join(target.manifest()), &bytes)?;
    selected.select_current()?;
    Ok(selected)
}

#[cfg(test)]
mod tests;
