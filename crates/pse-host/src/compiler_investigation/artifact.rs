// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit selected native compiler artifacts; unstable output remains opaque original bytes.
use super::{digest, invalid};
use crate::{
    HostError,
    correctness::{self, Resolution},
    envelope::{
        ArtifactReference, CliInvocationId, Completeness, Envelope, NextAction, ProviderContext,
        SubjectState,
    },
    environment::Environment,
    generation::{HostTool, PreparedGeneration},
    invocation::InvocationStore,
    managed::result::ManagedSummary,
    paths::CheckoutState,
    source_read::{SourceIdentity, SourceRequest},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::{
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    io::{Read as _, Write as _},
    os::unix::fs::{DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _},
    path::{Path, PathBuf},
    process::Command,
};

const PROVIDER: &str = "rustc/native-artifact-input/v1";
/// Selected native output, never interpreted as a stable compiler schema.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// Body/pass-filtered MIR before/after files.
    Mir,
    /// Selected unit THIR debug text; native rustc has no body filter for this view.
    Thir,
    /// Selected unit encountered concrete layouts; no cross-target ABI conclusion.
    TypeSizes,
    /// Selected unit monomorphization collection, separate from runtime reach.
    MonoItems,
    /// Authored isolated probe evaluation; stdout meaning belongs to its source.
    Evaluate,
}
/// Explicit native artifact selection. One package and one lib/bin root are required.
#[derive(Clone, Debug)]
pub struct Request {
    /// Selected native artifact kind.
    pub kind: Kind,
    /// Actual Cargo package selector, including opaque source-qualified IDs.
    pub package: String,
    /// Exactly one Cargo binary, or None for the library.
    pub binary: Option<String>,
    /// Explicit native target platform; absence uses Cargo's configured target.
    pub target: Option<String>,
    /// Native feature arguments projected from the CLI.
    pub feature_arguments: Vec<OsString>,
    /// Native body substring; required for MIR and never widened to all bodies.
    pub body: Option<String>,
    /// Native pass substrings; multiple passes are combined by rustc's own filter grammar.
    pub passes: Vec<String>,
    /// Original question; this is not an asserted inferred answer.
    pub question: String,
}
fn token(value: &str) -> bool {
    !value.trim().is_empty() && value.trim() != "all" && !value.contains(['&', '|', '\n', '\r'])
}
/// Construct literal native flags, using rustc's body/pass filters before any postprocessing.
pub fn flags(
    kind: Kind,
    body: Option<&str>,
    passes: &[String],
    output: &Path,
) -> Result<Vec<OsString>, HostError> {
    if kind != Kind::Mir && (body.is_some() || !passes.is_empty()) {
        return Err(invalid(
            "body/pass filtering is supported only for MIR; THIR/layout/mono output remains selected-unit scope",
        ));
    }
    Ok(match kind {
        Kind::Mir => {
            let body=body.filter(|body|token(body)).ok_or_else(||invalid("MIR requires a nonempty selected body substring without native filter operators; 'all' is unsupported"))?;
            if passes.iter().any(|pass| !token(pass)) {
                return Err(invalid(
                    "MIR pass names must be nonempty native substrings without filter operators",
                ));
            }
            let filter = if passes.is_empty() {
                body.to_owned()
            } else {
                passes
                    .iter()
                    .map(|pass| format!("{body} & {pass}"))
                    .collect::<Vec<_>>()
                    .join(" | ")
            };
            let directory=output.to_str().ok_or_else(||invalid("this rustc's dump-mir-dir flag requires a UTF-8 output directory; native paths were not converted lossily"))?;
            vec![
                format!("-Zdump-mir={filter}").into(),
                format!("-Zdump-mir-dir={directory}").into(),
                "-Zmir-include-spans=yes".into(),
            ]
        }
        Kind::Thir => vec!["-Zunpretty=thir-tree".into()],
        Kind::TypeSizes => vec!["-Zprint-type-sizes=yes".into()],
        Kind::MonoItems => vec!["-Zprint-mono-items=yes".into()],
        Kind::Evaluate => {
            return Err(invalid(
                "authored probe evaluation uses its existing run owner, not compiler flags",
            ));
        }
    })
}
/// One original input snapshot; only these explicitly retained bytes have a content claim.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CapturedInput {
    /// Actual original path, preserving operating-system bytes.
    pub source: OsString,
    /// Fixed private retained copy path.
    pub snapshot: OsString,
    /// Exact retained bytes.
    pub bytes: u64,
    /// Exact observed/retained content identity.
    pub sha256: String,
}
/// Exact selected compiler/probe context, retained before explicit generation.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Context {
    /// Evidence version, not an unstable debug-output schema.
    pub version: u32,
    /// Original performing invocation.
    pub invocation: CliInvocationId,
    /// Selected native output.
    pub kind: Kind,
    /// User's question, retained without guessing an answer.
    pub question: String,
    /// Native selected body and pass filters, when supported.
    pub body: Option<String>,
    /// Native selected pass substrings.
    pub passes: Vec<String>,
    /// Exact correctness-owned unit/feature/platform admission and performing arguments.
    pub admitted: Resolution,
    /// Independently acquired selected root source; not proof of every compiler-read byte.
    pub source: SourceIdentity,
    /// Exact retained primary source, manifest, lock and toolchain bytes when present.
    pub inputs: Vec<CapturedInput>,
    /// Actual selected rustc version/commit/host before generation.
    pub rustc_identity: String,
    /// Actual selected Cargo version/commit/host before generation.
    pub cargo_identity: String,
    /// Original immutable performing correctness image.
    pub prepared_host: PreparedGeneration,
    /// Authenticated original image checkout, independent of the selected source cwd.
    pub control_checkout: OsString,
    /// Actual child cwd.
    pub cwd: OsString,
    /// Exact selected environment digest, with no environment values published.
    pub environment_sha256: String,
    /// Unique native dump output directory; absence of an index is not an empty successful dump.
    pub output_directory: OsString,
    /// Existing authored probe provenance/resolution when this operation selected that owner.
    pub probe: Option<crate::probe::Probe>,
}
/// Prepared request; it grants no native execution or completed-artifact claim by itself.
#[derive(Debug)]
pub struct Prepared {
    /// Original retained context.
    pub context: Context,
    /// Complete explicit environment after native capability selection.
    pub environment: Environment,
    /// Immutable correctness executable and exact performing arguments.
    pub command: Vec<OsString>,
    /// Fixed unique original evidence directory.
    pub directory: PathBuf,
    context_sha256: String,
}
fn private_directory(path: &Path) -> Result<(), HostError> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)?;
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
    {
        return Err(invalid(
            "compiler artifact directory is not an owned private directory",
        ));
    }
    Ok(())
}
fn publish(path: &Path, bytes: &[u8]) -> Result<(), HostError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
fn identity(program: &OsStr, cwd: &Path, environment: &Environment) -> Result<String, HostError> {
    let output = Command::new(program)
        .arg("-vV")
        .current_dir(cwd)
        .env_clear()
        .envs(environment)
        .output()?;
    if !output.status.success() {
        return Err(invalid(format!(
            "selected tool identity exited {} without generation",
            output.status
        )));
    }
    String::from_utf8(output.stdout).map_err(|_| invalid("selected tool identity is not UTF-8"))
}
fn environment_sha(environment: &Environment) -> String {
    crate::cli::environment_digest(environment)
}
fn generation_environment(environment: &mut Environment) {
    // Native debug dumps are compiler side effects, not cached Cargo products. The explicit
    // generation stage must run the selected compiler passes, including repeated selections.
    environment.insert("CARGO_INCREMENTAL".into(), "0".into());
    environment.insert("SCCACHE_NO_CACHE".into(), "1".into());
    environment.remove(OsStr::new("SCCACHE_RECACHE"));
}
fn snapshot(source: &Path, target: &Path) -> Result<CapturedInput, HostError> {
    let bytes = fs::read(source)?;
    publish(target, &bytes)?;
    Ok(CapturedInput {
        source: source.as_os_str().to_owned(),
        snapshot: target.as_os_str().to_owned(),
        bytes: bytes.len() as u64,
        sha256: digest(&bytes),
    })
}
/// Resolve and retain an explicitly selected native compiler generation without compiling it.
pub fn prepare(
    root: &Path,
    control_root: &Path,
    state: &CheckoutState,
    id: &CliInvocationId,
    request: Request,
    environment: &Environment,
    host: &PreparedGeneration,
) -> Result<Prepared, HostError> {
    if request.question.trim().is_empty() {
        return Err(invalid(
            "native artifact generation requires the selected question",
        ));
    }
    let directory = state.root.join("artifacts/compiler").join(id.as_str());
    let output = directory.join("native");
    let native_flags = flags(
        request.kind,
        request.body.as_deref(),
        &request.passes,
        &output,
    )?;
    let mut arguments = vec![
        "rustc".into(),
        "--locked".into(),
        "-p".into(),
        request.package.into(),
    ];
    if let Some(binary) = request.binary {
        arguments.extend(["--bin".into(), binary.into()]);
    } else {
        arguments.push("--lib".into());
    }
    if let Some(target) = request.target {
        arguments.extend(["--target".into(), target.into()]);
    }
    arguments.extend(request.feature_arguments);
    arguments.push("--".into());
    arguments.extend(native_flags);
    let mut environment = correctness::standalone_adapter_environment(environment)?;
    generation_environment(&mut environment);
    let mut admitted = correctness::resolve_at(root, &environment, &arguments)?;
    if !admitted.native_capabilities.is_empty() {
        environment = crate::native::select_capabilities(
            root,
            &environment,
            &admitted
                .native_capabilities
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(","),
        )?;
        admitted = correctness::resolve_at(root, &environment, &arguments)?;
    }
    let roots = admitted
        .units
        .iter()
        .filter(|unit| unit.root)
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(invalid(
            "compiler artifact generation requires one resolved lib/bin compiler root",
        ));
    }
    let source = SourceRequest::Working {
        path: PathBuf::from(&roots[0].source_path),
    }
    .acquire()
    .map_err(crate::source_command::SourceCommandError::from)?;
    let cargo = correctness::real_cargo_in(&environment)?;
    let rustc_identity = identity(
        environment
            .get(OsStr::new("RUSTC"))
            .map_or(OsStr::new("rustc"), |value| value.as_os_str()),
        root,
        &environment,
    )?;
    let cargo_identity = identity(&cargo, root, &environment)?;
    InvocationStore::new(state.clone()).register()?;
    private_directory(
        directory
            .parent()
            .ok_or_else(|| invalid("compiler artifact parent is absent"))?,
    )?;
    fs::DirBuilder::new().mode(0o700).create(&directory)?;
    private_directory(&output)?;
    let mut inputs = vec![snapshot(
        source.identity().path(),
        &directory.join("source.rs"),
    )?];
    if inputs[0].sha256 != source.identity().source_sha256() {
        return Err(invalid(
            "selected source changed while its original snapshot was retained",
        ));
    }
    for (name, path) in [
        ("manifest.toml", root.join("Cargo.toml")),
        ("Cargo.lock", root.join("Cargo.lock")),
        ("rust-toolchain.toml", root.join("rust-toolchain.toml")),
    ] {
        if path.exists() {
            inputs.push(snapshot(&path, &directory.join(name))?);
        }
    }
    let context = Context {
        version: 1,
        invocation: id.clone(),
        kind: request.kind,
        question: request.question,
        body: request.body,
        passes: request.passes,
        admitted,
        source: source.identity().clone(),
        inputs,
        rustc_identity,
        cargo_identity,
        prepared_host: host.clone(),
        control_checkout: control_root.as_os_str().to_owned(),
        cwd: root.as_os_str().to_owned(),
        environment_sha256: environment_sha(&environment),
        output_directory: output.into_os_string(),
        probe: None,
    };
    let bytes = serde_json::to_vec_pretty(&context).map_err(std::io::Error::other)?;
    publish(&directory.join("context.json"), &bytes)?;
    let command = std::iter::once(
        host.executable(control_root, HostTool::Correctness)?
            .into_os_string(),
    )
    .chain(context.admitted.arguments.iter().cloned())
    .collect();
    Ok(Prepared {
        context,
        environment,
        command,
        directory,
        context_sha256: digest(&bytes),
    })
}
/// Prepare an existing explicitly authored isolated probe through its original graph/lease owner.
/// Evaluation runs the authored binary; other kinds request explicit native compiler output.
#[derive(Clone, Debug)]
pub struct ProbeRequest {
    /// Existing authored probe name.
    pub name: String,
    /// Explicit native output or authored evaluation.
    pub kind: Kind,
    /// Original question, never an inferred answer.
    pub question: String,
    /// Native MIR body substring when applicable.
    pub body: Option<String>,
    /// Native MIR pass substrings when applicable.
    pub passes: Vec<String>,
}
/// Resolve an existing probe and retain its original inputs under the same live-user lease.
pub fn prepare_probe(
    root: &Path,
    control_root: &Path,
    state: &CheckoutState,
    id: &CliInvocationId,
    request: ProbeRequest,
    environment: &Environment,
    host: &PreparedGeneration,
) -> Result<(Prepared, crate::probe::Run), HostError> {
    let ProbeRequest {
        name,
        kind,
        question,
        body,
        passes,
    } = request;
    if question.trim().is_empty() {
        return Err(invalid("probe evidence requires the selected question"));
    }
    let directory = state.root.join("artifacts/compiler").join(id.as_str());
    let output = directory.join("native");
    let native_flags = if kind == Kind::Evaluate {
        if body.is_some() || !passes.is_empty() {
            return Err(invalid(
                "authored probe evaluation does not select MIR body/pass filters",
            ));
        }
        Vec::new()
    } else {
        flags(kind, body.as_deref(), &passes, &output)?
    };
    let result = if kind == Kind::Evaluate {
        crate::probe::prepare_run(root, &state.root, &name, environment, &[])
    } else {
        crate::probe::prepare_compiler(root, &state.root, &name, environment, &native_flags)
    };
    let mut run = result.map_err(|error| HostError::Probe(Box::new(error)))?;
    if kind != Kind::Evaluate {
        generation_environment(&mut run.environment);
    }
    let mut admitted =
        correctness::resolve_at(&run.probe.directory, &run.environment, &run.command[1..])?;
    if !admitted.native_capabilities.is_empty() {
        run.environment = crate::native::select_capabilities(
            root,
            &run.environment,
            &admitted
                .native_capabilities
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(","),
        )?;
        admitted =
            correctness::resolve_at(&run.probe.directory, &run.environment, &run.command[1..])?;
    }
    let source = SourceRequest::Working {
        path: run.probe.directory.join("src/main.rs"),
    }
    .acquire()
    .map_err(crate::source_command::SourceCommandError::from)?;
    InvocationStore::new(state.clone()).register()?;
    private_directory(
        directory
            .parent()
            .ok_or_else(|| invalid("compiler artifact parent is absent"))?,
    )?;
    fs::DirBuilder::new().mode(0o700).create(&directory)?;
    private_directory(&output)?;
    let mut inputs = vec![snapshot(
        source.identity().path(),
        &directory.join("source.rs"),
    )?];
    if inputs[0].sha256 != source.identity().source_sha256() {
        return Err(invalid(
            "authored probe source changed while its original snapshot was retained",
        ));
    }
    for name in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "probe.json",
    ] {
        inputs.push(snapshot(
            &run.probe.directory.join(name),
            &directory.join(name),
        )?);
    }
    inputs.push(snapshot(
        &run.resolution_directory.join("metadata.stdout"),
        &directory.join("metadata.json"),
    )?);
    let context = Context {
        version: 1,
        invocation: id.clone(),
        kind,
        question,
        body,
        passes,
        admitted,
        source: source.identity().clone(),
        inputs,
        rustc_identity: run.rustc_identity.clone(),
        cargo_identity: run.cargo_identity.clone(),
        prepared_host: host.clone(),
        control_checkout: control_root.as_os_str().to_owned(),
        cwd: run.probe.directory.as_os_str().to_owned(),
        environment_sha256: environment_sha(&run.environment),
        output_directory: output.into_os_string(),
        probe: Some(run.probe.clone()),
    };
    let bytes = serde_json::to_vec_pretty(&context).map_err(std::io::Error::other)?;
    publish(&directory.join("context.json"), &bytes)?;
    let command = std::iter::once(
        host.executable(control_root, HostTool::Correctness)?
            .into_os_string(),
    )
    .chain(context.admitted.arguments.iter().cloned())
    .collect();
    Ok((
        Prepared {
            context,
            environment: run.environment.clone(),
            command,
            directory,
            context_sha256: digest(&bytes),
        },
        run,
    ))
}
impl Prepared {
    /// Recheck explicitly retained inputs and native admission immediately before effects.
    /// This does not promise an atomic filesystem snapshot of all transitive source reads.
    pub fn verify(&self) -> Result<(), HostError> {
        for input in &self.context.inputs {
            if digest(&fs::read(Path::new(&input.source))?) != input.sha256 {
                return Err(invalid(
                    "selected compiler input changed after preparation; resolve a new explicit generation",
                ));
            }
        }
        let current = correctness::resolve_at(
            Path::new(&self.context.cwd),
            &self.environment,
            &self.context.admitted.arguments,
        )?;
        if serde_json::to_vec(&current).map_err(std::io::Error::other)?
            != serde_json::to_vec(&self.context.admitted).map_err(std::io::Error::other)?
        {
            return Err(invalid(
                "selected compiler admission changed before generation",
            ));
        }
        Ok(())
    }
    /// Mandatory provenance independent of payload field projection.
    pub fn provider(&self) -> Result<ProviderContext, HostError> {
        let admitted = super::AdmittedContext {
            arguments: &self.context.admitted.arguments,
            packages: &self.context.admitted.packages,
            native_capabilities: &self.context.admitted.native_capabilities,
            matching_units: self
                .context
                .admitted
                .units
                .iter()
                .filter(|unit| unit.root)
                .collect(),
            total_units: self.context.admitted.units.len(),
        };
        Ok(ProviderContext {
            provider: PROVIDER.into(),
            context: Some(serde_json::to_string(&serde_json::json!({"invocation":self.context.invocation,"kind":self.context.kind,"question":self.context.question,"body":self.context.body,"passes":self.context.passes,"admitted":admitted,"source":self.context.source,"inputs":self.context.inputs,"rustc_identity":self.context.rustc_identity,"cargo_identity":self.context.cargo_identity,"prepared_host":self.context.prepared_host,"control_checkout":self.context.control_checkout,"cwd":self.context.cwd,"environment_sha256":self.context.environment_sha256,"full_context":self.directory.join("context.json"),"full_context_sha256":self.context_sha256})).map_err(std::io::Error::other)?),
            analysis_generation: Some(self.context_sha256.clone()),
            availability: SubjectState::Available,
            coordinates: self
                .context
                .inputs
                .first()
                .map(|input| {
                    Ok::<_, HostError>(crate::envelope::SourceCoordinates::Utf8Bytes {
                        source: serde_json::to_string(&input.snapshot)
                            .map_err(std::io::Error::other)?,
                        revision: Some(input.sha256.clone()),
                        start: 0,
                        end: input.bytes as usize,
                    })
                })
                .transpose()?
                .into_iter()
                .collect(),
        })
    }
    /// Original context and unique output directory; these are not successful artifact claims.
    pub fn references(&self) -> Vec<ArtifactReference> {
        self.directory
            .join("context.json")
            .to_str()
            .map(|path| {
                vec![ArtifactReference {
                    kind: "compiler_artifact_context".into(),
                    location: path.into(),
                    digest: Some(self.context_sha256.clone()),
                }]
            })
            .unwrap_or_default()
    }
}
/// One exact original artifact file; its digest is separate from interpretation support.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactFile {
    /// Exact original byte-safe file location.
    pub path: OsString,
    /// Whole retained extent.
    pub bytes: u64,
    /// Exact observed completed bytes.
    pub sha256: String,
}
fn file_digest(path: &Path) -> Result<ArtifactFile, HostError> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    let before = file.metadata()?;
    if !before.is_file() || before.uid() != rustix::process::geteuid().as_raw() {
        return Err(invalid("native artifact is not an owned regular file"));
    }
    let mut hash = Sha256::new();
    let mut count = 0u64;
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
        count += read as u64;
    }
    let after = file.metadata()?;
    if count != before.len()
        || before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.len() != after.len()
        || (
            before.mtime(),
            before.mtime_nsec(),
            before.ctime(),
            before.ctime_nsec(),
        ) != (
            after.mtime(),
            after.mtime_nsec(),
            after.ctime(),
            after.ctime_nsec(),
        )
    {
        return Err(invalid(
            "native artifact changed while its completed bytes were observed",
        ));
    }
    Ok(ArtifactFile {
        path: path.as_os_str().to_owned(),
        bytes: count,
        sha256: hash
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    })
}
fn retain_native(source: &Path, destination: &Path) -> Result<ArtifactFile, HostError> {
    let mut input = OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(source)?;
    let before = input.metadata()?;
    if !before.is_file() || before.uid() != rustix::process::geteuid().as_raw() {
        return Err(invalid(
            "native compiler output is not an owned regular file",
        ));
    }
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(destination)?;
    let mut hash = Sha256::new();
    let mut count = 0u64;
    let mut buffer = [0u8; 8192];
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        output.write_all(&buffer[..read])?;
        hash.update(&buffer[..read]);
        count += read as u64;
    }
    output.sync_all()?;
    let after = input.metadata()?;
    if count != before.len()
        || (
            before.dev(),
            before.ino(),
            before.len(),
            before.mtime(),
            before.mtime_nsec(),
            before.ctime(),
            before.ctime_nsec(),
        ) != (
            after.dev(),
            after.ino(),
            after.len(),
            after.mtime(),
            after.mtime_nsec(),
            after.ctime(),
            after.ctime_nsec(),
        )
    {
        return Err(invalid(
            "native compiler output changed during original retention",
        ));
    }
    Ok(ArtifactFile {
        path: destination.as_os_str().to_owned(),
        bytes: count,
        sha256: hash
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    })
}
/// Completed opaque artifact index; no debug-format parser or semantic pass is invented.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Index {
    /// Checked index version.
    pub version: u32,
    /// Original performing operation.
    pub invocation: CliInvocationId,
    /// Exact retained compiler/source context.
    pub context_sha256: String,
    /// Completed native files and raw stdout/stderr; no body was regenerated by this read.
    pub files: Vec<ArtifactFile>,
}
/// Index successful, drained original outputs before the existing managed owner publishes terminal.
pub fn finish(
    prepared: &Prepared,
    store: &InvocationStore,
) -> Result<Vec<ArtifactReference>, HostError> {
    let mut files = Vec::new();
    let retained = prepared.directory.join("retained");
    private_directory(&retained)?;
    for entry in fs::read_dir(Path::new(&prepared.context.output_directory))? {
        let entry = entry?;
        files.push(retain_native(
            &entry.path(),
            &retained.join(entry.file_name()),
        )?);
    }
    if prepared.context.kind == Kind::Mir && files.is_empty() {
        return Err(invalid(
            "native compiler produced no selected MIR files; body/pass filter may be unsupported or unmatched. Original streams and selected inputs remain retained",
        ));
    }
    let invocation = store.directory(&prepared.context.invocation)?;
    for name in ["stdout.raw", "stderr.raw"] {
        files.push(file_digest(&invocation.path().join(name))?);
    }
    for input in &prepared.context.inputs {
        let file = file_digest(Path::new(&input.snapshot))?;
        if file.sha256 != input.sha256 || file.bytes != input.bytes {
            return Err(invalid(
                "retained authored compiler input changed before original artifact publication",
            ));
        }
        files.push(file);
    }
    let context_file = file_digest(&prepared.directory.join("context.json"))?;
    if context_file.sha256 != prepared.context_sha256 {
        return Err(invalid(
            "original compiler context changed before artifact publication",
        ));
    }
    files.push(context_file);
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let index = Index {
        version: 1,
        invocation: prepared.context.invocation.clone(),
        context_sha256: prepared.context_sha256.clone(),
        files,
    };
    let bytes = serde_json::to_vec_pretty(&index).map_err(std::io::Error::other)?;
    let path = prepared.directory.join("index.json");
    publish(&path, &bytes)?;
    Ok(path
        .to_str()
        .map(|path| {
            vec![ArtifactReference {
                kind: "compiler_artifact_index".into(),
                location: path.into(),
                digest: Some(digest(&bytes)),
            }]
        })
        .unwrap_or_default())
}
/// Bounded original artifact inventory with original managed outcome/capture facts.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Observation {
    /// Original performing identity.
    pub original_invocation: CliInvocationId,
    /// Original exact generation context, not current workspace inference.
    pub context: Context,
    /// Original actually waited compiler/program child status.
    pub child_exit: Option<i32>,
    /// Original managed operation outcome, distinct from this passive read succeeding.
    pub original_outcome: crate::envelope::CommandOutcome,
    /// Original subject projection; no current native user state is inferred from it.
    pub original_subject_state: SubjectState,
    /// Original owner settlement projection, not an external-resource release claim.
    pub original_settlement: crate::envelope::Settlement,
    /// Original native/capture/finalization coverage.
    pub original_observation: crate::envelope::Observation,
    /// Original normalized failures, including unsupported/unmatched dump publication.
    pub original_failures: Vec<crate::envelope::ErrorRecord>,
    /// Original stream/native lifetime observations, separate from semantic output meaning.
    pub capture: ManagedSummary,
    /// Original successful completed-file index, absent after failure or unfinished publication.
    pub index: Option<Index>,
    /// Distinguish original lack of publication from a subsequently missing retained index.
    pub index_availability: IndexAvailability,
}
/// Original publication and current retention are separate observations.
#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IndexAvailability {
    /// Original terminal bound this exact available completed-file index.
    Available,
    /// Original terminal never published a completed-file index.
    NotPublished,
    /// Original terminal bound an index that is now absent; no generation is replayed.
    MissingPublished,
}
/// Read original retained evidence without resolving Cargo, preparing tools or compiling.
pub fn observe(
    root: &Path,
    state: &CheckoutState,
    selector: &str,
) -> Result<Envelope<Observation>, HostError> {
    let store = InvocationStore::new(state.clone());
    let id = store
        .resolve(selector, None)?
        .ok_or_else(|| invalid("original compiler artifact invocation is absent"))?;
    let original = store
        .result::<serde::de::IgnoredAny, ManagedSummary>(&id)?
        .ok_or_else(|| {
            invalid("original compiler artifact operation has no published terminal yet")
        })?;
    if !matches!(
        original.command.as_str(),
        "compiler generate" | "compiler probe"
    ) {
        return Err(invalid(
            "selected original operation is not compiler artifact generation",
        ));
    }
    let directory = state.root.join("artifacts/compiler").join(id.as_str());
    let bytes = crate::test_inventory::read_private(&directory.join("context.json"))?;
    let sha = digest(&bytes);
    if !original.providers.iter().any(|provider| {
        provider.provider == PROVIDER && provider.analysis_generation.as_deref() == Some(&sha)
    }) {
        return Err(invalid(
            "original compiler context identity differs from its managed terminal",
        ));
    }
    let context: Context = serde_json::from_slice(&bytes).map_err(std::io::Error::other)?;
    if context.version != 1 || context.invocation != id {
        return Err(invalid("unsupported/mismatched original compiler context"));
    }
    let index_published = original
        .artifacts
        .iter()
        .any(|artifact| artifact.kind == "compiler_artifact_index");
    let index = if directory.join("index.json").exists() {
        let bytes = crate::test_inventory::read_private(&directory.join("index.json"))?;
        let sha = digest(&bytes);
        if !original.artifacts.iter().any(|artifact| {
            artifact.kind == "compiler_artifact_index" && artifact.digest.as_deref() == Some(&sha)
        }) {
            return Err(invalid(
                "completed compiler artifact index is not bound by the original terminal",
            ));
        }
        let index: Index = serde_json::from_slice(&bytes).map_err(std::io::Error::other)?;
        if index.version != 1
            || index.invocation != id
            || index.context_sha256
                != digest(&serde_json::to_vec_pretty(&context).map_err(std::io::Error::other)?)
        {
            return Err(invalid(
                "completed compiler artifact index has changed context",
            ));
        }
        Some(index)
    } else {
        None
    };
    let index_availability = if index.is_some() {
        IndexAvailability::Available
    } else if index_published {
        IndexAvailability::MissingPublished
    } else {
        IndexAvailability::NotPublished
    };
    let native_provider=ProviderContext {provider:"original-managed-compiler-outcome/v1".into(),context:Some(serde_json::to_string(&serde_json::json!({"invocation":id,"outcome":original.outcome,"subject_state":original.subject_state,"settlement":original.settlement,"child_exit":original.child_exit,"observation":original.observation,"failures":original.failures,"index_availability":index_availability})).map_err(std::io::Error::other)?),analysis_generation:Some(sha),availability:SubjectState::Available,coordinates:vec![]};
    let mut response = Envelope::response(
        "compiler artifacts",
        "original native artifact evidence; no compiler rerun or debug-format interpretation",
        Observation {
            original_invocation: id,
            context,
            child_exit: original.child_exit,
            original_outcome: original.outcome,
            original_subject_state: original.subject_state,
            original_settlement: original.settlement,
            original_observation: original.observation,
            original_failures: original.failures,
            capture: original.payload,
            index,
            index_availability,
        },
    );
    response.providers = original.providers;
    response.providers.push(native_provider);
    response.artifacts = original.artifacts;
    response.observation.scope = "original_selected_native_compiler_artifacts".into();
    response.observation.limitations.push("Native MIR/THIR/layout/mono output remains opaque and pin-specific. Source snapshots cover the explicitly retained inputs, not every transitive byte rustc read. Initialization, storage, liveness, loans, allocation, native resource settlement and runtime reach are distinct questions; no inferred may/must flow facts or ABI guarantees are supplied. If these original artifacts cannot settle the retained question, a pin-specific structured extraction remains an I05 assessment lead; no driver is prepared or installed by this reader.".into());
    if response.payload.index.is_none() {
        response.observation.completeness = Completeness::Partial;
        response.observation.required = false;
        response.observation.limitations.push(if index_published {"The original terminal bound a completed-file index that is now missing. Original source snapshots and native files are not removed or regenerated by this read."}else{"No completed-file index was published. The original failed/interrupted/capture outcome and raw stream coverage remain authoritative; the unique native output directory may contain unindexed partial files."}.into());
    }
    if let Some(root) = root.to_str() {
        if let Some(index) = &response.payload.index {
            for (ordinal, _) in index.files.iter().enumerate() {
                response.next.push(NextAction::Command {argv:vec!["pse".into(),"-C".into(),root.into(),"compiler".into(),"file".into(),response.payload.original_invocation.to_string(),ordinal.to_string()],reason:"read bounded original opaque bytes with the retained full-file digest guard; no compilation".into()});
            }
        }
        for suffix in [None, Some("--stderr")] {
            let mut argv = vec![
                "pse".into(),
                "-C".into(),
                root.into(),
                "log".into(),
                response.payload.original_invocation.to_string(),
            ];
            if let Some(suffix) = suffix {
                argv.push(suffix.into());
            }
            response.next.push(NextAction::Command {argv,reason:"inspect the original captured stream and its native coverage without interpreting a new compiler verdict".into()});
        }
    }
    if root.to_str().is_none() {
        response.observation.limitations.push(
            "Native original paths are retained; Unicode command argv may be unavailable.".into(),
        );
    }
    Ok(response)
}

#[cfg(test)]
mod tests;
/// Bounded original opaque file read; only its explicitly verified digest is a whole-file claim.
#[derive(Debug, Serialize, JsonSchema)]
pub struct FileRead {
    /// Original performing operation.
    pub original_invocation: CliInvocationId,
    /// Exact historical file index entry.
    pub file: ArtifactFile,
    /// Selected bytes from the same checked descriptor after full original digest validation.
    pub read: crate::artifacts::DescriptorRead,
}
/// Verify the original index/file digest and read one bounded extent; never invoke a compiler.
pub fn read_file(
    root: &Path,
    state: &CheckoutState,
    selector: &str,
    ordinal: usize,
    offset: u64,
    length: std::num::NonZeroU64,
) -> Result<Envelope<FileRead>, HostError> {
    let original = observe(root, state, selector)?;
    let index=original.payload.index.as_ref().ok_or_else(||invalid("original generation has no completed artifact index; inspect its native outcome/stream coverage"))?;
    let file = index
        .files
        .get(ordinal)
        .cloned()
        .ok_or_else(|| invalid("file ordinal is outside original artifact index"))?;
    let descriptor = crate::test_inventory::open_private(Path::new(&file.path))?;
    let read =
        crate::artifacts::read_descriptor(descriptor, offset, Some(length), Some(&file.sha256))?;
    let next = read.next_offset;
    let mut response = Envelope::response(
        "compiler file",
        "bounded original opaque bytes after whole-file digest verification; no compilation",
        FileRead {
            original_invocation: original.payload.original_invocation,
            file,
            read,
        },
    );
    response.providers = original.providers;
    response.artifacts = original.artifacts;
    response.observation.scope = "original_compiler_artifact_descriptor_range".into();
    response.observation.limitations.push("The output is byte bounded, but full original-file SHA verification scans the whole file. Native debug output is opaque; metadata/digest checks around the same descriptor do not establish atomic snapshot isolation or interpreted flow/layout/ABI/runtime facts.".into());
    if let (Some(next), Some(root)) = (next, root.to_str()) {
        response.next.push(NextAction::Command {
            argv: vec![
                "pse".into(),
                "-C".into(),
                root.into(),
                "compiler".into(),
                "file".into(),
                response.payload.original_invocation.to_string(),
                ordinal.to_string(),
                "--offset".into(),
                next.to_string(),
                "--length".into(),
                length.to_string(),
            ],
            reason:
                "read the next bounded original extent with the same complete-file digest guard"
                    .into(),
        });
    }
    Ok(response)
}
