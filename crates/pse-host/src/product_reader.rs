// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Captured product reader selection and opaque passive transport, without domain admission.
use crate::{
    correctness,
    discovery::{BuildCapture, Executable},
    environment::Environment,
    paths::CheckoutState,
};
use pse_diagnostics::{DrainFailurePolicy, StderrCapture, SupervisedChild, SupervisionError};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
/// Bounded observer polling; no native scientific or canonical control is retained.
pub mod follow;
pub mod native;

const PROTOCOL: &str = "pse.product-reader";
const REQUEST_SCHEMA: &str = "urn:pse:product-reader:v1:request";
const RESPONSE_SCHEMA: &str = "urn:pse:product-reader:v1:response";
const REQUEST_BYTES: usize = 1024 * 1024;
const RESPONSE_BYTES: usize = 40 * 1024 * 1024;
const STDERR_BYTES: usize = 65536;
const READ_TIMEOUT: Duration = Duration::from_secs(30);
pub(crate) const PROJECT_PERFORM_PROTOCOL: &str = "pse.project-perform";
pub(crate) const PROJECT_PERFORM_REQUEST_BYTES: usize = REQUEST_BYTES;
pub(crate) const PROJECT_PERFORM_RESPONSE_BYTES: usize = 1024 * 1024;

/// Preparation, original native child ownership, or transport validation failed.
#[derive(Debug, thiserror::Error)]
pub enum ProductReaderError {
    /// Original local file or pipe failure.
    #[error("product reader I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Original external JSON decoding failure.
    #[error("product reader JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Original correctness-owned admission or captured executable refusal.
    #[error(transparent)]
    Correctness(#[from] correctness::CorrectnessError),
    /// Original host generation custody failure.
    #[error(transparent)]
    Host(#[from] crate::HostError),
    /// Original checkout resolver failure.
    #[error(transparent)]
    Environment(#[from] crate::environment::EnvironmentError),
    /// Required original compiler capture was absent or invalid after a successful build.
    #[error("required source-build capture at {path} is unavailable: {source}")]
    BuildEvidence {
        /// Exact retained capture path, independent of a mutable Cargo target.
        path: PathBuf,
        /// Original file/decoding cause.
        #[source]
        source: Box<ProductReaderError>,
    },
    /// Original synchronous native child supervision failure.
    #[error(transparent)]
    Supervision(#[from] SupervisionError),
    /// Independent helper, pipe and settlement failures after all owned tasks were joined.
    #[error("product reader helper failed in {} independent responsibilities", failures.len())]
    Execution {
        /// Original causes, without replacing one failure with another.
        failures: Vec<ProductReaderError>,
        /// Original stdout prefix, unmodified.
        stdout: Vec<u8>,
        /// Original stderr prefix, unmodified.
        stderr: Vec<u8>,
    },
    /// Checked association or bounded protocol refused; original bytes remain available.
    #[error("product reader transport: {message}")]
    Transport {
        /// Host transport explanation, without interpreting product state.
        message: String,
        /// Original retained stdout prefix, unmodified.
        stdout: Vec<u8>,
        /// Original retained stderr prefix, unmodified.
        stderr: Vec<u8>,
    },
    /// Invalid immutable selection or unsupported caller transport context.
    #[error("product reader selection: {0}")]
    Invalid(String),
}
pse_diagnostics::impl_diagnostic! {
    ProductReaderError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(this) { match this { Self::BuildEvidence{source,..}=>Some(source.as_ref()), Self::Correctness(error)=>Some(error), Self::Host(error)=>Some(error), Self::Environment(error)=>Some(error), Self::Supervision(error)=>Some(error), _=>None } },
    help(_this) { None }, related(this) { match this { Self::Execution {failures,..}=>Some(Box::new(failures.iter().map(|error| {let error:&dyn pse_diagnostics::TypedDiagnostic=error;error}))), _=>None } }, source(_this) { None }
}
fn invalid(message: impl Into<String>) -> ProductReaderError {
    ProductReaderError::Invalid(message.into())
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn private_directory(path: &Path) -> Result<(), ProductReaderError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o777 != 0o700
    {
        return Err(invalid(
            "product selection directory must be private and owned",
        ));
    }
    Ok(())
}
fn read_file(path: &Path, maximum: usize) -> Result<Vec<u8>, ProductReaderError> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.uid() != rustix::process::geteuid().as_raw() {
        return Err(invalid("product selection file must be regular and owned"));
    }
    let mut bytes = Vec::new();
    file.take(maximum as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(invalid("product selection file exceeds its decoding bound"));
    }
    Ok(bytes)
}
fn publish(path: &Path, bytes: &[u8]) -> Result<(), ProductReaderError> {
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
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u32,
    checkout_id: String,
    selection_id: String,
    executable: Executable,
    source: crate::generation::SourceFingerprint,
    host_generation: crate::generation::PreparedGeneration,
    correctness: Executable,
    build: Evidence,
    discovery: Evidence,
    request_schema: Evidence,
    response_schema: Evidence,
}
/// Transport category describes this read command, never the observed scientific outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// Product accepted the request, including unavailable or failed observed work.
    Success,
    /// Product refused decoding or its own selection/domain contract.
    Refused,
}
/// Directional schema route, interpreted only as product transport metadata.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Contract {
    /// Accepted product input representation.
    Request,
    /// Emitted product response representation.
    Response,
}
/// Exact product payload and original transport bytes. No host domain decoder is involved.
pub struct ProductReply {
    category: Category,
    payload: Box<RawValue>,
    frame: Vec<u8>,
    stderr: Vec<u8>,
    next_request: Option<Box<RawValue>>,
    managed_invocation: Option<pse_diagnostics::process_facts::ManagedInvocationCandidate>,
}
impl std::fmt::Debug for ProductReply {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProductReply")
            .field("category", &self.category)
            .field("payload_bytes", &self.payload.get().len())
            .field("frame_bytes", &self.frame.len())
            .field("stderr_bytes", &self.stderr.len())
            .finish_non_exhaustive()
    }
}
impl ProductReply {
    /// Original product command category.
    pub fn category(&self) -> Category {
        self.category
    }
    /// Original lexical product payload, including duplicate keys and exact numbers.
    pub fn payload(&self) -> &RawValue {
        &self.payload
    }
    /// Complete original product response frame.
    pub fn frame(&self) -> &[u8] {
        &self.frame
    }
    /// Bounded original stderr prefix, independent of product payload.
    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }
    /// Opaque product-issued continuation; the host never decodes scientific coordinates.
    pub fn next_request(&self) -> Option<&RawValue> {
        self.next_request.as_deref()
    }
    /// Producer-published routing candidate; native custody and membership require readback.
    pub fn managed_invocation(
        &self,
    ) -> Option<&pse_diagnostics::process_facts::ManagedInvocationCandidate> {
        self.managed_invocation.as_ref()
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Response {
    protocol: String,
    version: u32,
    correlation: String,
    schema: String,
    category: Category,
    payload: Box<RawValue>,
    #[serde(default)]
    next_request: Option<Box<RawValue>>,
    #[serde(default)]
    managed_invocation: Option<pse_diagnostics::process_facts::ManagedInvocationCandidate>,
}
#[derive(Serialize)]
struct Request<'a> {
    protocol: &'static str,
    version: u32,
    correlation: &'a str,
    request: &'a RawValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    access: Option<&'static str>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectEffectReply {
    protocol: String,
    version: u32,
    operation: Option<String>,
    pub(crate) target: Option<ProjectEffectTarget>,
    pub(crate) outcome: ProjectEffectOutcome,
    pub(crate) cleanup: Option<crate::envelope::ErrorRecord>,
}
#[derive(Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectEffectTarget {
    pub(crate) issuing_store: String,
    pub(crate) namespace: String,
    pub(crate) database: String,
}
#[derive(Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectEffectAcknowledgement {
    pub(crate) key: String,
    pub(crate) kind: String,
    pub(crate) project: String,
    pub(crate) branch: Option<String>,
    pub(crate) revision: Option<String>,
    generation: u64,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ProjectEffectOutcome {
    Acknowledged {
        acknowledgement: ProjectEffectAcknowledgement,
    },
    Absent,
    ImportIssued {
        intent: Box<RawValue>,
    },
    Imported {
        report: Box<RawValue>,
    },
    ImportRecovered {
        report: Box<RawValue>,
    },
    ImportOpened {
        source: Box<RawValue>,
    },
    Export {
        status: Box<RawValue>,
    },
    Refused {
        failure: Box<crate::envelope::ErrorRecord>,
    },
    Unresolved {
        failure: Box<crate::envelope::ErrorRecord>,
        import_status: Option<Box<RawValue>>,
        publication_status: Option<Box<RawValue>>,
        readback_failure: Option<Box<crate::envelope::ErrorRecord>>,
    },
}
#[derive(Debug)]
pub(crate) struct Output {
    pub(crate) status: i32,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stdout_bytes: u64,
    pub(crate) stderr: Vec<u8>,
}
fn prefix(mut input: impl Read, maximum: usize) -> (Vec<u8>, u64, Option<std::io::Error>) {
    let mut bytes = Vec::new();
    let mut total = 0;
    let mut buffer = [0_u8; 8192];
    loop {
        let count = match input.read(&mut buffer) {
            Ok(count) => count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return (bytes, total, Some(error)),
        };
        if count == 0 {
            return (bytes, total, None);
        }
        total += count as u64;
        let keep = count.min(maximum - bytes.len());
        bytes.extend_from_slice(&buffer[..keep]);
    }
}
// SupervisedChild retains the actual pidfd/group owner. These readers only drain bounded
// prefixes; neither a pipe EOF nor a successful product response establishes child settlement.
pub(crate) fn execute(
    command: Command,
    input: Vec<u8>,
    maximum: usize,
    timeout: Option<Duration>,
) -> Result<Output, ProductReaderError> {
    execute_control(command, input, maximum, timeout, None)
}
fn execute_control(
    mut command: Command,
    input: Vec<u8>,
    maximum: usize,
    timeout: Option<Duration>,
    control: Option<&pse_diagnostics::Cancellation>,
) -> Result<Output, ProductReaderError> {
    command.stdin(Stdio::piped()).stdout(Stdio::piped());
    let deadline = timeout.map(|timeout| Instant::now() + timeout);
    let mut child = SupervisedChild::spawn(
        &mut command,
        DrainFailurePolicy::Park,
        Some(StderrCapture {
            bytes: STDERR_BYTES,
            stack_bytes: 256 * 1024,
        }),
    )?;
    let mut failures = Vec::new();
    let writer = match child.take_stdin() {
        Some(mut stdin) => match std::thread::Builder::new()
            .name("pse-product-input".into())
            .spawn(move || stdin.write_all(&input))
        {
            Ok(task) => Some(task),
            Err(error) => {
                failures.push(error.into());
                None
            }
        },
        None => {
            failures.push(invalid("child input pipe absent"));
            None
        }
    };
    let reader = match child.take_stdout() {
        Some(stdout) => match std::thread::Builder::new()
            .name("pse-product-output".into())
            .spawn(move || prefix(stdout, maximum))
        {
            Ok(task) => Some(task),
            Err(error) => {
                failures.push(error.into());
                None
            }
        },
        None => {
            failures.push(invalid("child output pipe absent"));
            None
        }
    };
    if failures.is_empty()
        && let Err(error) = child.wait_ready(control, deadline)
    {
        failures.push(error.into());
    }
    let status = match child.cancel() {
        Ok(status) => Some(status),
        Err(error) => {
            failures.push(error.into());
            child.retained_status()
        }
    };
    // Park ownership returns a cancellation error only after leader/group completion (for
    // example a stderr reader failure); unknown drain retains custody inside that owner.
    let mut stdout = Vec::new();
    let mut stdout_bytes = 0;
    if let Some(writer) = writer {
        match writer.join() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => failures.push(error.into()),
            Err(_) => failures.push(invalid("product input worker panicked")),
        }
    }
    if let Some(reader) = reader {
        match reader.join() {
            Ok((bytes, total, error)) => {
                stdout = bytes;
                stdout_bytes = total;
                if let Some(error) = error {
                    failures.push(error.into());
                }
            }
            Err(_) => failures.push(invalid("product output worker panicked")),
        }
    }
    let stderr = child.take_stderr_prefix().unwrap_or_default();
    if !failures.is_empty() {
        return Err(ProductReaderError::Execution {
            failures,
            stdout,
            stderr,
        });
    }
    let status = status.ok_or_else(|| invalid("completed helper has no retained child status"))?;
    use std::os::unix::process::ExitStatusExt as _;
    Ok(Output {
        status: status
            .code()
            .unwrap_or_else(|| 128 + status.signal().unwrap_or(1)),
        stdout,
        stdout_bytes,
        stderr,
    })
}
fn transport(message: impl Into<String>, output: Output) -> ProductReaderError {
    ProductReaderError::Transport {
        message: message.into(),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}
fn decode(output: Output, correlation: &str) -> Result<ProductReply, ProductReaderError> {
    if output.stdout_bytes > RESPONSE_BYTES as u64 {
        return Err(transport("response exceeds complete-frame extent", output));
    }
    let decoded: Response = match serde_json::from_slice(&output.stdout) {
        Ok(decoded) => decoded,
        Err(error) => {
            return Err(transport(
                format!("invalid response frame: {error}"),
                output,
            ));
        }
    };
    if decoded.protocol != PROTOCOL
        || decoded.version != 1
        || decoded.correlation != correlation
        || decoded.schema != RESPONSE_SCHEMA
    {
        return Err(transport(
            "response transport identity/correlation mismatch",
            output,
        ));
    }
    if output.status
        != match decoded.category {
            Category::Success => 0,
            Category::Refused => 2,
        }
    {
        return Err(transport(
            "response category and original child status disagree",
            output,
        ));
    }
    Ok(ProductReply {
        category: decoded.category,
        payload: decoded.payload,
        frame: output.stdout,
        stderr: output.stderr,
        next_request: decoded.next_request,
        managed_invocation: decoded.managed_invocation,
    })
}

pub(crate) fn project_import_intent_matches(
    intent: &RawValue,
    operation: &str,
    issuing_store: &str,
) -> bool {
    #[derive(Deserialize)]
    struct Intent<'a> {
        #[serde(borrow)]
        publication: Publication<'a>,
    }
    #[derive(Deserialize)]
    struct Publication<'a> {
        operation: &'a str,
        issuing_store: &'a str,
    }
    serde_json::from_str::<Intent<'_>>(intent.get()).is_ok_and(|intent| {
        intent.publication.operation == operation
            && intent.publication.issuing_store == issuing_store
    })
}

fn decode_project_effect(
    output: Output,
    operation: &str,
    issuing_store: &str,
    database: Option<&str>,
) -> Result<ProjectEffectReply, ProductReaderError> {
    if output.stdout_bytes > PROJECT_PERFORM_RESPONSE_BYTES as u64 {
        return Err(transport(
            "project response exceeds complete-frame extent",
            output,
        ));
    }
    let reply: ProjectEffectReply = match serde_json::from_slice(&output.stdout) {
        Ok(reply) => reply,
        Err(error) => {
            return Err(transport(
                format!("invalid project response frame: {error}"),
                output,
            ));
        }
    };
    let success = matches!(
        reply.outcome,
        ProjectEffectOutcome::Acknowledged { .. }
            | ProjectEffectOutcome::Absent
            | ProjectEffectOutcome::ImportIssued { .. }
            | ProjectEffectOutcome::Imported { .. }
            | ProjectEffectOutcome::ImportRecovered { .. }
            | ProjectEffectOutcome::ImportOpened { .. }
            | ProjectEffectOutcome::Export { .. }
    );
    let target_matches = match &reply.target {
        Some(target) => {
            target.issuing_store == issuing_store
                && database.is_none_or(|database| database == target.database)
        }
        None => !success,
    };
    // This is a single-request authenticated subprocess. Only the exact
    // no-effect refusal can predate decoding the operation identity.
    let operation_matches = match reply.operation.as_deref() {
        Some(actual) => actual == operation,
        None => {
            matches!(reply.outcome, ProjectEffectOutcome::Refused { .. })
                && reply.target.is_none()
                && reply.cleanup.is_none()
        }
    };
    let acknowledgement_matches = match &reply.outcome {
        ProjectEffectOutcome::Acknowledged { acknowledgement } => acknowledgement.key == operation,
        ProjectEffectOutcome::ImportIssued { intent } => {
            project_import_intent_matches(intent, operation, issuing_store)
        }
        _ => true,
    };
    if reply.protocol != PROJECT_PERFORM_PROTOCOL
        || reply.version != 1
        || !operation_matches
        || !acknowledgement_matches
        || !target_matches
        || output.status
            != if success && reply.cleanup.is_none() {
                0
            } else {
                2
            }
    {
        return Err(transport(
            "project response identity, target or status mismatch",
            output,
        ));
    }
    Ok(reply)
}

/// Selected immutable product reader; construction authenticates resolver-owned evidence.
#[derive(Debug)]
pub struct ProductReaderSelection {
    checkout: PathBuf,
    directory: PathBuf,
    manifest: Manifest,
}
impl ProductReaderSelection {
    /// Host-minted resolver key, independent of mutable source and current host selection.
    pub fn id(&self) -> &str {
        &self.manifest.selection_id
    }
    /// Actual captured product executable identity.
    pub fn executable(&self) -> &Executable {
        &self.manifest.executable
    }
    /// Original source/build context observed unchanged around explicit preparation.
    /// Later reads intentionally use the captured image, without claiming current sources.
    pub fn source_identity(&self) -> Result<String, ProductReaderError> {
        Ok(self.manifest.source.digest()?)
    }
    /// Open an explicit resolver key; passive selection never builds or checks host freshness.
    pub fn open(
        checkout: &Path,
        environment: &Environment,
        selection_id: &str,
    ) -> Result<Self, ProductReaderError> {
        if uuid::Uuid::parse_str(selection_id).is_err() || selection_id.len() != 36 {
            return Err(invalid("invalid product reader selection key"));
        }
        let checkout = checkout.canonicalize()?;
        let state = CheckoutState::resolve(&checkout, environment)?;
        let products = state.root.join("products");
        private_directory(&products)?;
        private_directory(&products.join("readers"))?;
        let directory = products.join("readers").join(selection_id);
        private_directory(&directory)?;
        let manifest: Manifest =
            serde_json::from_slice(&read_file(&directory.join("reader.json"), REQUEST_BYTES)?)?;
        if manifest.version != 1
            || manifest.checkout_id != state.checkout_id.as_str()
            || manifest.selection_id != selection_id
        {
            return Err(invalid(
                "product reader checkout/manifest identity mismatch",
            ));
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
        evidence: &Evidence,
        name: &str,
        maximum: usize,
    ) -> Result<Vec<u8>, ProductReaderError> {
        if evidence.name != name {
            return Err(invalid(
                "product evidence path differs from its fixed association",
            ));
        }
        let bytes = read_file(&self.directory.join(name), maximum)?;
        if bytes.len() != evidence.bytes || hash(&bytes) != evidence.sha256 {
            return Err(invalid("captured product evidence identity mismatch"));
        }
        Ok(bytes)
    }
    fn verify(&self) -> Result<(), ProductReaderError> {
        private_directory(&self.directory)?;
        private_directory(&self.directory.join("build"))?;
        if self.manifest.correctness.captured != self.directory.join("cargo-pse-correctness") {
            return Err(invalid(
                "recorded correctness control is outside its fixed capture association",
            ));
        }
        self.manifest.correctness.verify()?;
        let bytes = self.evidence(&self.manifest.build, "build/build.json", RESPONSE_BYTES)?;
        let build: BuildCapture = serde_json::from_slice(&bytes)?;
        let executable = select_executable(&build, &self.directory)?;
        if executable != &self.manifest.executable {
            return Err(invalid(
                "product executable does not match its original compiler artifact",
            ));
        }
        executable.verify()?;
        discovery_document(&self.evidence(
            &self.manifest.discovery,
            "discovery.json",
            RESPONSE_BYTES,
        )?)?;
        schema_document(
            &self.evidence(
                &self.manifest.request_schema,
                "request-schema.json",
                RESPONSE_BYTES,
            )?,
            REQUEST_SCHEMA,
        )?;
        schema_document(
            &self.evidence(
                &self.manifest.response_schema,
                "response-schema.json",
                RESPONSE_BYTES,
            )?,
            RESPONSE_SCHEMA,
        )?;
        Ok(())
    }
    /// Captured complete discovery response from this selected executable.
    pub fn discovery(&self) -> Result<Vec<u8>, ProductReaderError> {
        self.evidence(&self.manifest.discovery, "discovery.json", RESPONSE_BYTES)
    }
    pub(crate) fn supports_project_perform(&self) -> Result<bool, ProductReaderError> {
        self.project_capability("project_perform", 1)
    }

    pub(crate) fn supports_project_import(&self) -> Result<bool, ProductReaderError> {
        self.project_capability("project_import", 1)
    }

    pub(crate) fn supports_project_import_open(&self) -> Result<bool, ProductReaderError> {
        self.project_capability("project_import_open", 1)
    }

    pub(crate) fn supports_project_export(&self) -> Result<bool, ProductReaderError> {
        self.project_capability("project_export", 2)
    }

    fn project_capability(&self, selected: &str, version: u64) -> Result<bool, ProductReaderError> {
        let response: Response = serde_json::from_slice(&self.discovery()?)?;
        let capability: serde_json::Value = serde_json::from_str(response.payload.get())?;
        Ok(capability["project_perform"].as_u64() == Some(1)
            && capability[selected].as_u64() == Some(version))
    }

    pub(crate) fn perform_project(
        &self,
        request: &RawValue,
        operation: &str,
        issuing_store: &str,
        database: Option<&str>,
        environment: &Environment,
    ) -> Result<ProjectEffectReply, ProductReaderError> {
        self.verify()?;
        let mut command = Command::new(&self.manifest.executable.captured);
        command
            .arg("perform")
            .current_dir(&self.checkout)
            .env_clear()
            .envs(environment);
        let output = execute(
            command,
            request.get().as_bytes().to_vec(),
            PROJECT_PERFORM_RESPONSE_BYTES,
            Some(READ_TIMEOUT),
        )?;
        decode_project_effect(output, operation, issuing_store, database)
    }
    /// Captured exact schema document; the product owns all domain properties.
    pub fn schema(&self, contract: Contract) -> Result<Box<RawValue>, ProductReaderError> {
        let (evidence, name, identity) = match contract {
            Contract::Response => (
                &self.manifest.response_schema,
                "response-schema.json",
                RESPONSE_SCHEMA,
            ),
            Contract::Request => (
                &self.manifest.request_schema,
                "request-schema.json",
                REQUEST_SCHEMA,
            ),
        };
        schema_document(&self.evidence(evidence, name, RESPONSE_BYTES)?, identity)
    }
    /// Delegate one opaque product operation, preserving its lexical payload and native refusal.
    pub fn read(
        &self,
        request: &RawValue,
        correlation: &str,
        environment: &Environment,
    ) -> Result<ProductReply, ProductReaderError> {
        self.read_control(request, correlation, environment, None, false)
    }
    /// Explicitly admitted diagnostic effects; passive callers cannot enter this route implicitly.
    pub fn investigate(
        &self,
        request: &RawValue,
        correlation: &str,
        environment: &Environment,
    ) -> Result<ProductReply, ProductReaderError> {
        self.read_control(request, correlation, environment, None, true)
    }
    /// Passive read with an observer-only stop token; cancellation settles only this helper.
    pub fn read_cancellable(
        &self,
        request: &RawValue,
        correlation: &str,
        environment: &Environment,
        control: &pse_diagnostics::Cancellation,
    ) -> Result<ProductReply, ProductReaderError> {
        self.read_control(request, correlation, environment, Some(control), false)
    }
    fn read_control(
        &self,
        request: &RawValue,
        correlation: &str,
        environment: &Environment,
        control: Option<&pse_diagnostics::Cancellation>,
        investigation: bool,
    ) -> Result<ProductReply, ProductReaderError> {
        if correlation.is_empty() || correlation.len() > 128 {
            return Err(invalid("correlation requires one through 128 UTF-8 bytes"));
        }
        let bytes = serde_json::to_vec(&Request {
            protocol: PROTOCOL,
            version: 1,
            correlation,
            request,
            access: investigation.then_some("investigation"),
        })?;
        if bytes.len() > REQUEST_BYTES {
            return Err(invalid("request exceeds product transport extent"));
        }
        self.verify()?;
        let mut command = Command::new(&self.manifest.executable.captured);
        command
            .arg("observe")
            .current_dir(&self.checkout)
            .env_clear()
            .envs(environment);
        decode(
            execute_control(command, bytes, RESPONSE_BYTES, Some(READ_TIMEOUT), control)?,
            correlation,
        )
    }
}
fn select_executable<'a>(
    build: &'a BuildCapture,
    directory: &Path,
) -> Result<&'a Executable, ProductReaderError> {
    if build.version != 1 || build.cargo_exit != 0 || !build.incomplete.is_empty() {
        return Err(invalid("source compiler capture is incomplete or failed"));
    }
    let matches: Vec<_> = build
        .artifacts
        .iter()
        .enumerate()
        .filter(|(_, artifact)| {
            artifact.cargo.target.name == "pse-project"
                && artifact
                    .cargo
                    .target
                    .kind
                    .iter()
                    .any(|kind| kind.to_string() == "bin")
                && artifact
                    .cargo
                    .features
                    .iter()
                    .any(|feature| feature == "product-reader")
        })
        .collect();
    if matches.len() != 1 {
        return Err(invalid(
            "source build did not capture exactly one selected product reader",
        ));
    }
    let (index, artifact) = matches[0];
    let original =
        artifact.cargo.executable.as_ref().ok_or_else(|| {
            invalid("selected product compiler artifact has no original executable")
        })?;
    if artifact.executable.original != original.as_std_path()
        || artifact.executable.captured
            != crate::discovery::capture::artifact_path(
                &directory.join("build"),
                index,
                original.as_std_path(),
            )?
    {
        return Err(invalid(
            "recorded product executable is outside its fixed capture association",
        ));
    }
    if !build.resolution.units.iter().any(|unit| {
        unit.root
            && unit.package_name == "pse-runtime"
            && unit.package_id == artifact.cargo.package_id.repr
            && unit.source_path == artifact.cargo.target.src_path.as_str()
            && unit.target_name == "pse-project"
            && unit.mode == "build"
            && unit.features.contains("product-reader")
    }) {
        return Err(invalid(
            "actual compiler units do not establish the same product package/target/source selection",
        ));
    }
    Ok(&artifact.executable)
}
fn schema_document(frame: &[u8], identity: &str) -> Result<Box<RawValue>, ProductReaderError> {
    #[derive(Deserialize)]
    struct SchemaPayload {
        kind: String,
        identity: String,
        document: Box<RawValue>,
    }
    let response: Response = serde_json::from_slice(frame)?;
    let payload: SchemaPayload = serde_json::from_str(response.payload.get())?;
    let document: serde_json::Value = serde_json::from_str(payload.document.get())?;
    if response.protocol != PROTOCOL
        || response.version != 1
        || response.schema != RESPONSE_SCHEMA
        || response.category != Category::Success
        || payload.kind != "schema"
        || payload.identity != identity
        || document.get("$id").and_then(serde_json::Value::as_str) != Some(identity)
    {
        return Err(invalid("product schema identity/route mismatch"));
    }
    Ok(payload.document)
}
fn discovery_document(frame: &[u8]) -> Result<(), ProductReaderError> {
    #[derive(Deserialize)]
    struct Discovery {
        kind: String,
        request_schema: String,
        response_schema: String,
        request_bytes: usize,
        response_bytes: usize,
    }
    let response: Response = serde_json::from_slice(frame)?;
    let payload: Discovery = serde_json::from_str(response.payload.get())?;
    if response.protocol != PROTOCOL
        || response.version != 1
        || response.schema != RESPONSE_SCHEMA
        || response.category != Category::Success
        || payload.kind != "discovery"
        || payload.request_schema != REQUEST_SCHEMA
        || payload.response_schema != RESPONSE_SCHEMA
        || payload.request_bytes != REQUEST_BYTES
        || payload.response_bytes != RESPONSE_BYTES
    {
        return Err(invalid(
            "product discovery transport/schema/extent mismatch",
        ));
    }
    Ok(())
}
fn source(
    checkout: &Path,
    environment: &Environment,
    installed_schema: bool,
) -> Result<crate::generation::SourceFingerprint, ProductReaderError> {
    let cargo = correctness::real_cargo_in(environment)?;
    let observed = |program: &std::ffi::OsStr| -> Result<String, ProductReaderError> {
        let mut command = Command::new(program);
        command
            .arg("-vV")
            .current_dir(checkout)
            .env_clear()
            .envs(environment);
        let output = execute(command, Vec::new(), STDERR_BYTES, Some(READ_TIMEOUT))?;
        if output.status != 0 || output.stdout_bytes > STDERR_BYTES as u64 {
            return Err(transport("tool version observation failed", output));
        }
        String::from_utf8(output.stdout).map_err(|_| invalid("tool version output is not UTF-8"))
    };
    let rustc = environment
        .get(std::ffi::OsStr::new("RUSTC"))
        .map_or(std::ffi::OsStr::new("rustc"), |value| value.as_os_str());
    let features = std::collections::BTreeSet::from([
        format!("pse-runtime/{}", reader_feature(installed_schema)),
        "pse-runtime/force-validate".into(),
    ]);
    let inputs = environment
        .iter()
        .map(|(name, value)| (name.to_string_lossy().into_owned(), Some(value.clone())))
        .collect();
    let context = crate::generation::BuildContext::new(
        observed(rustc)?,
        observed(&cargo)?,
        "dev".into(),
        features,
        environment,
    )
    .with_additional_inputs(&inputs);
    let metadata_features = format!(
        "pse-runtime/{},pse-runtime/force-validate",
        reader_feature(installed_schema)
    );
    let arguments: Vec<std::ffi::OsString> = [
        "metadata",
        "--format-version=1",
        "--locked",
        "--offline",
        "--features",
        &metadata_features,
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
    let extras = crate::generation::EXTERNAL_INPUTS
        .iter()
        .copied()
        .chain([
            "scripts/diagnostic_sql/timeline.sql",
            "scripts/diagnostic_sql/coverage.sql",
            "scripts/diagnostic_sql/numerical.sql",
        ])
        .map(PathBuf::from);
    Ok(crate::generation::SourceFingerprint::from_metadata_for(
        checkout,
        &metadata,
        context,
        "pse-runtime",
    )?
    .with_additional_files(checkout, extras)?)
}

fn reader_feature(installed_schema: bool) -> &'static str {
    if installed_schema {
        "product-operators"
    } else {
        "product-reader"
    }
}

fn source_build_command(
    control: &Executable,
    checkout: &Path,
    environment: &Environment,
    installed_schema: bool,
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
            "pse-runtime",
            "--bin",
            "pse-project",
            "--features",
            reader_feature(installed_schema),
            "--message-format=json",
        ]);
    command
}
fn build_evidence(directory: &Path) -> Result<(Vec<u8>, BuildCapture), ProductReaderError> {
    let path = directory.join("build.json");
    let result = (|| {
        let bytes = read_file(&path, RESPONSE_BYTES)?;
        let build = serde_json::from_slice(&bytes)?;
        Ok((bytes, build))
    })();
    result.map_err(|source| ProductReaderError::BuildEvidence {
        path,
        source: Box::new(source),
    })
}

/// Explicit source-build/capture preparation. The caller owns new-work admission; passive
/// open/read never enters this route. Failed preparation evidence is retained at its minted ID.
pub fn prepare(
    checkout: &Path,
    environment: &Environment,
) -> Result<ProductReaderSelection, ProductReaderError> {
    prepare_selected(checkout, environment, false)
}

pub(crate) fn prepare_selected(
    checkout: &Path,
    environment: &Environment,
    installed_schema: bool,
) -> Result<ProductReaderSelection, ProductReaderError> {
    prepare_with_permit(checkout, environment, installed_schema, None)
}

pub(crate) fn prepare_with_permit(
    checkout: &Path,
    environment: &Environment,
    installed_schema: bool,
    permit: Option<&crate::build_admission::Permit>,
) -> Result<ProductReaderSelection, ProductReaderError> {
    let checkout = checkout.canonicalize()?;
    let state = CheckoutState::resolve(&checkout, environment)?;
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&state.root)?;
    private_directory(&state.root)?;
    for path in [
        state.root.join("products"),
        state.root.join("products/readers"),
    ] {
        match fs::DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        };
        private_directory(&path)?;
    }
    let parent = state.root.join("products/readers");
    let id = uuid::Uuid::new_v4().to_string();
    let directory = parent.join(&id);
    fs::DirBuilder::new().mode(0o700).create(&directory)?;
    let build_directory = directory.join("build");
    fs::DirBuilder::new().mode(0o700).create(&build_directory)?;
    let prepared = crate::generation::prepared(&checkout)?.ok_or_else(|| {
        invalid("native correctness control is unprepared; run scripts/bootstrap-host")
    })?;
    let source_control =
        prepared.executable(&checkout, crate::generation::HostTool::Correctness)?;
    let correctness = crate::discovery::capture::executable(
        &source_control,
        &directory.join("cargo-pse-correctness"),
    )?;
    // Children use independently authenticated copies; source-build process settlement remains separate.
    (|| {
        let mut environment = correctness::standalone_adapter_environment(environment)?;
        let before = source(&checkout, &environment, installed_schema)?;
        environment.insert(
            "PSE_CORRECTNESS_BUILD_CAPTURE".into(),
            build_directory.clone().into_os_string(),
        );
        correctness.verify()?;
        let mut command =
            source_build_command(&correctness, &checkout, &environment, installed_schema);
        if let Some(permit) = permit {
            command.env(crate::build_admission::CALLBACK, permit.callback());
        }
        let output = execute(command, Vec::new(), STDERR_BYTES, None)?;
        if output.status != 0 {
            return Err(transport(
                format!(
                    "original source build exited {}; captured compiler output at {}",
                    output.status,
                    build_directory.display()
                ),
                output,
            ));
        }
        let (build_bytes, build) = build_evidence(&build_directory)?;
        let executable = select_executable(&build, &directory)?.clone();
        executable.verify()?;
        // The manifest is published only after all three native responses were obtained from
        // this exact retained executable. No product self-hash participates in this binding.
        let evidence = |name: &str, bytes: &[u8]| Evidence {
            name: name.into(),
            sha256: hash(bytes),
            bytes: bytes.len(),
        };
        let mut selected = ProductReaderSelection {
            checkout: checkout.clone(),
            directory: directory.clone(),
            manifest: Manifest {
                version: 1,
                checkout_id: state.checkout_id.as_str().into(),
                selection_id: id.clone(),
                executable,
                source: before,
                host_generation: prepared,
                correctness,
                build: Evidence {
                    name: "build/build.json".into(),
                    sha256: hash(&build_bytes),
                    bytes: build_bytes.len(),
                },
                discovery: Evidence {
                    name: String::new(),
                    sha256: String::new(),
                    bytes: 0,
                },
                request_schema: Evidence {
                    name: String::new(),
                    sha256: String::new(),
                    bytes: 0,
                },
                response_schema: Evidence {
                    name: String::new(),
                    sha256: String::new(),
                    bytes: 0,
                },
            },
        };
        let query = |body: &str,
                     correlation: &str,
                     name: &str|
         -> Result<ProductReply, ProductReaderError> {
            let request = RawValue::from_string(body.into())?;
            let bytes = serde_json::to_vec(&Request {
                protocol: PROTOCOL,
                version: 1,
                correlation,
                request: &request,
                access: None,
            })?;
            let mut command = Command::new(&selected.manifest.executable.captured);
            command
                .arg("observe")
                .current_dir(&checkout)
                .env_clear()
                .envs(&environment);
            let output = execute(command, bytes, RESPONSE_BYTES, Some(READ_TIMEOUT))?;
            publish(&directory.join(name), &output.stdout)?;
            publish(&directory.join(format!("{name}.stderr")), &output.stderr)?;
            let reply = decode(output, correlation)?;
            if reply.category != Category::Success {
                return Err(ProductReaderError::Transport {
                    message: "captured product refused discovery/schema".into(),
                    stdout: reply.frame,
                    stderr: reply.stderr,
                });
            }
            Ok(reply)
        };
        let discovery = query(
            r#"{"operation":"discover"}"#,
            "prepare-discovery",
            "discovery.json",
        )?;
        discovery_document(&discovery.frame)?;
        let request = query(
            r#"{"operation":"schema","contract":"request"}"#,
            "prepare-request-schema",
            "request-schema.json",
        )?;
        schema_document(&request.frame, REQUEST_SCHEMA)?;
        let response = query(
            r#"{"operation":"schema","contract":"response"}"#,
            "prepare-response-schema",
            "response-schema.json",
        )?;
        schema_document(&response.frame, RESPONSE_SCHEMA)?;
        selected.manifest.discovery = evidence("discovery.json", &discovery.frame);
        selected.manifest.request_schema = evidence("request-schema.json", &request.frame);
        selected.manifest.response_schema = evidence("response-schema.json", &response.frame);
        environment.remove(std::ffi::OsStr::new("PSE_CORRECTNESS_BUILD_CAPTURE"));
        selected.manifest.source.require_unchanged(&source(
            &checkout,
            &environment,
            installed_schema,
        )?)?;
        selected.verify()?;
        let manifest = serde_json::to_vec(&selected.manifest)?;
        if manifest.len() > REQUEST_BYTES {
            return Err(invalid(
                "complete product selection manifest exceeds its supported decoding extent",
            ));
        }
        publish(&directory.join("reader.json"), &manifest)?;
        Ok(selected)
    })()
}

#[cfg(test)]
mod tests;
