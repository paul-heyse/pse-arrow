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

const PROTOCOL: &str = "pse.product-reader";
const REQUEST_SCHEMA: &str = "urn:pse:product-reader:v1:request";
const RESPONSE_SCHEMA: &str = "urn:pse:product-reader:v1:response";
const REQUEST_BYTES: usize = 1024 * 1024;
const RESPONSE_BYTES: usize = 40 * 1024 * 1024;
const STDERR_BYTES: usize = 65536;
const READ_TIMEOUT: Duration = Duration::from_secs(30);

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
    /// Preserve original operation and independent generation-custody settlement failure.
    #[error(
        "product reader preparation failed: {operation}; custody settlement failed: {settlement}"
    )]
    Settlement {
        /// Original operation error.
        operation: Box<ProductReaderError>,
        /// Independent original custody error.
        settlement: crate::HostError,
    },
}
pse_diagnostics::impl_diagnostic! {
    ProductReaderError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(this) { match this { Self::Correctness(error)=>Some(error), Self::Host(error)=>Some(error), Self::Environment(error)=>Some(error), Self::Supervision(error)=>Some(error), _=>None } },
    help(_this) { None }, related(this) { match this { Self::Execution {failures,..}=>Some(Box::new(failures.iter().map(|error| {let error:&dyn pse_diagnostics::TypedDiagnostic=error;error}))), Self::Settlement {operation,settlement}=>{ let errors:[&dyn pse_diagnostics::TypedDiagnostic;2]=[operation.as_ref(),settlement];Some(Box::new(errors.into_iter())) }, _=>None } }, source(_this) { None }
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
}
#[derive(Serialize)]
struct Request<'a> {
    protocol: &'static str,
    version: u32,
    correlation: &'a str,
    request: &'a RawValue,
}
#[derive(Debug)]
struct Output {
    status: i32,
    stdout: Vec<u8>,
    stdout_bytes: u64,
    stderr: Vec<u8>,
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
fn execute(
    mut command: Command,
    input: Vec<u8>,
    maximum: usize,
    timeout: Option<Duration>,
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
        && let Err(error) = child.wait_ready(None, deadline)
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
    })
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
        if correlation.is_empty() || correlation.len() > 128 {
            return Err(invalid("correlation requires one through 128 UTF-8 bytes"));
        }
        let bytes = serde_json::to_vec(&Request {
            protocol: PROTOCOL,
            version: 1,
            correlation,
            request,
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
            execute(command, bytes, RESPONSE_BYTES, Some(READ_TIMEOUT))?,
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
    if artifact.executable.captured != directory.join("build").join(format!("artifact-{index}")) {
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
        "pse-runtime/product-reader".into(),
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
    let arguments: Vec<std::ffi::OsString> = [
        "metadata",
        "--format-version=1",
        "--locked",
        "--offline",
        "--features",
        "pse-runtime/product-reader,pse-runtime/force-validate",
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

/// Explicit source-build/capture preparation. The caller owns new-work admission; passive
/// open/read never enters this route. Failed preparation evidence is retained at its minted ID.
pub fn prepare(
    checkout: &Path,
    environment: &Environment,
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
    let tools = checkout.join("target/host-tools");
    let usage = crate::generation::GenerationUse::acquire(&tools, &prepared.generation)?;
    let captured = crate::discovery::capture::executable(
        &tools
            .join(&prepared.generation)
            .join("cargo-pse-correctness"),
        &directory.join("cargo-pse-correctness"),
    )
    .map_err(ProductReaderError::from);
    let correctness = match (captured, usage.settle()) {
        (result, Ok(())) => result?,
        (Ok(_), Err(error)) => return Err(error.into()),
        (Err(operation), Err(settlement)) => {
            return Err(ProductReaderError::Settlement {
                operation: Box::new(operation),
                settlement,
            });
        }
    };
    // No original-generation path is lent to a child after this explicit release.
    // The captured control and product do not establish whole source-build settlement.
    (|| {
        let mut environment = correctness::standalone_adapter_environment(environment)?;
        let before = source(&checkout, &environment)?;
        environment.insert(
            "PSE_CORRECTNESS_BUILD_CAPTURE".into(),
            build_directory.clone().into_os_string(),
        );
        correctness.verify()?;
        let mut command = Command::new(&correctness.captured);
        command
            .current_dir(&checkout)
            .env_clear()
            .envs(&environment)
            .args([
                "build",
                "--locked",
                "-p",
                "pse-runtime",
                "--bin",
                "pse-project",
                "--features",
                "product-reader",
                "--message-format=json",
            ]);
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
        let build_bytes = read_file(&build_directory.join("build.json"), RESPONSE_BYTES)?;
        let build: BuildCapture = serde_json::from_slice(&build_bytes)?;
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
        selected
            .manifest
            .source
            .require_unchanged(&source(&checkout, &environment)?)?;
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
