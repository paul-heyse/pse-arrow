// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit project effects and exact acknowledgement lookup through the performing owner.
use super::{ProjectSource, context::ResolvedSource, product_error, request_file};
use crate::{
    HostError,
    cli::GenerationIntent,
    envelope::{
        CommandOutcome, Completeness, Envelope, ErrorRecord, NextAction, OutputOptions,
        RedactionPolicy, Settlement, SubjectState,
    },
    environment::Environment,
    investigation::NativePath,
    product_reader::{
        self, ProductReaderSelection, ProjectEffectAcknowledgement, ProjectEffectOutcome,
        ProjectEffectTarget,
    },
};
use clap::{Args, Subcommand};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use sha2::{Digest as _, Sha256};
use std::{
    fs::{self, DirBuilder, File, OpenOptions},
    io::{self, Read as _},
    num::NonZeroU64,
    os::unix::{
        ffi::OsStrExt as _,
        fs::{DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _},
    },
    path::{Path, PathBuf},
};

#[derive(Debug, Subcommand)]
pub(in crate::cli) enum Action {
    /// Create a project under an explicit original operation identity.
    #[command(add(crate::catalog::description::CommandEffect::Apply))]
    Create {
        #[command(flatten)]
        target: Selection,
        /// Canonical project key.
        project: String,
        /// Human-readable project title.
        title: String,
    },
    /// Apply an authored navigation intent, read from stdin by default.
    #[command(add(crate::catalog::description::CommandEffect::Apply))]
    Navigate {
        #[command(flatten)]
        target: Selection,
        /// Authored JSON containing the original project descriptor and typed request.
        #[arg(long)]
        intent: Option<PathBuf>,
    },
    /// Reconcile an original operation identity without submitting another effect.
    #[command(add(crate::catalog::description::CommandEffect::Recover))]
    Acknowledgement {
        #[command(flatten)]
        target: Selection,
    },
    /// Issue the original receiving intent for a verified project pack.
    #[command(add(crate::catalog::description::CommandEffect::Prepare))]
    ImportIssue {
        #[command(flatten)]
        target: Selection,
        /// Existing logical project pack.
        input: PathBuf,
        /// Title for the receiving project.
        title: String,
        /// Override the receiving branch name.
        #[arg(long)]
        branch_name: Option<String>,
    },
    /// Apply the exact issued import intent to the original pack.
    #[command(add(crate::catalog::description::CommandEffect::Apply))]
    Import {
        #[command(flatten)]
        target: Selection,
        /// Original logical project pack.
        input: PathBuf,
        /// Exact issued intent JSON from another checkout. Uses retained custody by default.
        #[arg(long)]
        intent: Option<PathBuf>,
    },
    /// Recover only the original unsettled import artifact.
    #[command(add(crate::catalog::description::CommandEffect::Recover))]
    ImportRecover {
        #[command(flatten)]
        target: Selection,
        /// Authored receipt for an exact earlier artifact. Defaults to the current original artifact.
        #[arg(long)]
        receipt: Option<PathBuf>,
    },
    /// Observe the original receiving import and artifact checkpoint.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    ImportStatus {
        #[command(flatten)]
        target: Selection,
        /// Exact issued intent JSON, if custody was acquired from another checkout.
        #[arg(long)]
        intent: Option<PathBuf>,
    },
    /// Reopen the receiving branch from protected copied source after completed import.
    #[command(add(crate::catalog::description::CommandEffect::Read))]
    ImportOpen {
        #[command(flatten)]
        target: Selection,
        /// Exact issued intent JSON from another checkout. Uses retained custody by default.
        #[arg(long)]
        intent: Option<PathBuf>,
    },
    /// Release local custody once the original project import is complete.
    #[command(add(crate::catalog::description::CommandEffect::Apply))]
    ImportForget {
        #[command(flatten)]
        target: Selection,
    },
    /// Publish a project pack and exact receipt to a fresh directory.
    #[command(add(crate::catalog::description::CommandEffect::Apply))]
    Export {
        #[command(flatten)]
        target: Selection,
        /// Fresh destination directory.
        destination: PathBuf,
        /// Authored JSON with `request` and optional `physical`. Defaults to stdin.
        #[arg(long)]
        intent: Option<PathBuf>,
    },
    /// Publish bounded NDJSON pages with the original protected cohort receipt.
    #[command(add(crate::catalog::description::CommandEffect::Apply))]
    ExportCohort {
        #[command(flatten)]
        target: Selection,
        /// Fresh directory containing `cohort.ndjson` and its receipt.
        destination: PathBuf,
        /// Authored JSON with `request` and optional `maximum_bytes`. Defaults to stdin.
        #[arg(long)]
        intent: Option<PathBuf>,
    },
    /// Publish one exact cohort member's Arrow result with its original receipt.
    #[command(add(crate::catalog::description::CommandEffect::Apply))]
    ExportResult {
        #[command(flatten)]
        target: Selection,
        /// Fresh directory containing `result.arrow` and its receipt.
        destination: PathBuf,
        /// Authored JSON with `request`, `member`, `relation`, `start`, `end` and optional `maximum_bytes`.
        #[arg(long)]
        intent: Option<PathBuf>,
    },
    /// Publish one registered output aggregate with the original cohort and coverage receipt.
    #[command(add(crate::catalog::description::CommandEffect::Apply))]
    ExportAnalysis {
        #[command(flatten)]
        target: Selection,
        /// Fresh directory containing the receipt and `analysis.arrow` when produced.
        destination: PathBuf,
        /// Authored JSON with `request` and `kpi`. Defaults to stdin.
        #[arg(long)]
        intent: Option<PathBuf>,
    },
    /// Finish publishing an exact complete staged deliverable without recapturing data.
    #[command(add(crate::catalog::description::CommandEffect::Recover))]
    ExportRecover {
        #[command(flatten)]
        target: Selection,
        /// Exact original destination directory.
        destination: PathBuf,
    },
}
#[derive(Debug, Args)]
pub(in crate::cli) struct Selection {
    #[command(flatten)]
    source: ProjectSource,
    /// Expected issuing store reported by the original project read.
    #[arg(long)]
    issuing_store: Option<String>,
    /// Original operation identity, retained across acknowledgement queries.
    #[arg(long)]
    operation: String,
}
impl Action {
    pub(super) fn intent(&self) -> GenerationIntent {
        match self {
            Self::Acknowledgement { .. } | Self::ImportStatus { .. } | Self::ImportOpen { .. } => GenerationIntent::Passive,
            Self::ImportForget { .. } | Self::ExportRecover { .. } => GenerationIntent::ExistingControl,
            _ => GenerationIntent::NewWork,
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RequestAction {
    Create {
        project: String,
        title: String,
    },
    Navigate {
        project: Box<RawValue>,
        request: Box<RawValue>,
    },
    Acknowledgement,
    ImportIssue {
        input: NativePath,
        title: String,
        branch_name: Option<String>,
    },
    Import {
        input: NativePath,
        intent: Box<RawValue>,
    },
    ImportRecover {
        intent: Box<RawValue>,
        original: Box<RawValue>,
    },
    ImportOpen { intent: Box<RawValue> },
    Export { destination: NativePath, request: ExportRequest },
    ExportRecover { destination: NativePath },
}

#[derive(Clone, Copy)]
enum ExportKind { Pack, Cohort, Result, Analysis }

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum ExportRequest {
    Pack(Box<RawValue>), Cohort(Box<RawValue>), Result(Box<RawValue>), Analysis(Box<RawValue>),
}

impl ExportKind {
    fn request(self, body: Box<RawValue>) -> ExportRequest {
        match self {
            Self::Pack => ExportRequest::Pack(body),
            Self::Cohort => ExportRequest::Cohort(body),
            Self::Result => ExportRequest::Result(body),
            Self::Analysis => ExportRequest::Analysis(body),
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NavigationIntent {
    project: Box<RawValue>,
    request: Box<RawValue>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImportRecovery {
    intent: Box<RawValue>,
    original: Box<RawValue>,
}
#[derive(Serialize)]
struct Request<'a> {
    protocol: &'static str,
    version: u32,
    operation: &'a str,
    target: Target<'a>,
    timeout_ms: Option<NonZeroU64>,
    action: RequestAction,
}
#[derive(Serialize)]
struct Target<'a> {
    state: NativePath,
    database: Option<&'a str>,
    issuing_store: &'a str,
}

#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Disposition {
    Unprepared,
    Acknowledged,
    ImportIssued,
    Imported,
    ImportRecovered,
    ImportOpened,
    PublicationUnknown,
    PublicationPending,
    PublicationReady,
    Published,
    ImportForgotten,
    Absent,
    Refused,
    Unresolved,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ExportStatus { Unknown, Pending, Ready, Published }

impl ExportStatus {
    fn presentation(self) -> (Disposition, SubjectState, Settlement, String) {
        let (disposition, subject, settlement, summary) = match self {
            Self::Unknown => (Disposition::PublicationUnknown, SubjectState::Unknown, Settlement::Unknown, "no exact original publication was observed"),
            Self::Pending => (Disposition::PublicationPending, SubjectState::Unknown, Settlement::Incomplete, "the original publication has no complete staged deliverable"),
            Self::Ready => (Disposition::PublicationReady, SubjectState::Unknown, Settlement::Incomplete, "the original complete deliverable is staged"),
            Self::Published => (Disposition::Published, SubjectState::Succeeded, Settlement::Complete, "the original deliverable and receipt are published"),
        };
        (disposition, subject, settlement, summary.into())
    }
}
#[derive(Serialize, JsonSchema)]
pub(in crate::cli) struct ProjectEffectView {
    reader: String,
    operation: String,
    target: Option<ProjectEffectTarget>,
    disposition: Disposition,
    acknowledgement: Option<ProjectEffectAcknowledgement>,
    import: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    publication: Option<serde_json::Value>,
}

fn navigation_intent(path: Option<&Path>) -> Result<NavigationIntent, HostError> {
    let decode = |text: &str| {
        serde_json::from_str(text).map_err(|error| {
            HostError::Ownership(format!(
                "invalid project navigation intent ({:?}, line {}, column {})",
                error.classify(),
                error.line(),
                error.column()
            ))
        })
    };
    if let Some(path) = path {
        return decode(request_file(path)?.get());
    }
    let mut bytes = Vec::new();
    io::stdin()
        .lock()
        .take(product_reader::PROJECT_PERFORM_REQUEST_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > product_reader::PROJECT_PERFORM_REQUEST_BYTES {
        return Err(HostError::Ownership(
            "project navigation intent exceeds its byte bound".into(),
        ));
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| HostError::Ownership("project navigation intent is not UTF-8 JSON".into()))?;
    decode(text)
}

pub(super) fn original_input(
    selected: Option<&Path>,
    input: Option<PathBuf>,
) -> Result<Box<RawValue>, HostError> {
    match input {
        Some(path) => request_file(&super::super::source_path(selected, path)?),
        None => super::read_request(io::stdin().lock()),
    }
}

fn safe_original(value: &RawValue, policy: &RedactionPolicy) -> Option<serde_json::Value> {
    fn contains_secret(value: &serde_json::Value, policy: &RedactionPolicy) -> bool {
        match value {
            serde_json::Value::String(value) => policy.redact(value) != *value,
            serde_json::Value::Array(values) => {
                values.iter().any(|value| contains_secret(value, policy))
            }
            serde_json::Value::Object(entries) => entries
                .iter()
                .any(|(key, value)| policy.redact(key) != *key || contains_secret(value, policy)),
            _ => false,
        }
    }
    let parsed = serde_json::from_str(value.get()).ok()?;
    (!contains_secret(&parsed, policy)).then_some(parsed)
}

const INTENT_BYTES: usize = product_reader::PROJECT_PERFORM_RESPONSE_BYTES;
const MAXIMUM_ORIGINALS: usize = 64;

fn original_path(
    root: &Path,
    environment: &Environment,
    source: &ResolvedSource,
    issuer: &str,
    operation: &str,
) -> Result<PathBuf, HostError> {
    let state = crate::paths::CheckoutState::resolve(root, environment)?;
    let mut digest = Sha256::new();
    for value in [
        source.state.as_os_str().as_bytes(),
        issuer.as_bytes(),
        operation.as_bytes(),
    ] {
        digest.update((value.len() as u64).to_le_bytes());
        digest.update(value);
    }
    let digest: String = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let name = format!("{digest}.json");
    Ok(state.root.join("project-imports").join(name))
}

fn read_original(path: &Path) -> Result<Box<RawValue>, HostError> {
    let parent = intent_parent(path)?;
    super::super::private_state::directory(parent)?;
    let bytes = super::super::private_state::read(path, INTENT_BYTES)?;
    serde_json::from_slice(&bytes).map_err(|error| io::Error::other(error).into())
}

fn publish_original(path: &Path, intent: &RawValue) -> Result<(), HostError> {
    let parent = intent_parent(path)?;
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(parent)?;
    let _lease = lock_originals(parent)?;
    let mut originals = 0;
    for entry in parent.read_dir()? {
        if entry?
            .path()
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            originals += 1;
            if originals >= MAXIMUM_ORIGINALS {
                return Err(HostError::Ownership("checkout import intent capacity reached; release completed originals with import-forget".into()));
            }
        }
    }
    crate::invocation::publish_bytes(path, intent.get().as_bytes(), Some(INTENT_BYTES), 0o600)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

fn lock_originals(parent: &Path) -> Result<File, HostError> {
    super::super::private_state::directory(parent)?;
    let flags = rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o600)
        .custom_flags(flags.bits() as i32)
        .open(parent.join(".lock"))?;
    let metadata = lock.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
    {
        return Err(HostError::Ownership(
            "import intent lease is not an owned private file".into(),
        ));
    }
    lock.lock()?;
    Ok(lock)
}

fn release_original(path: &Path, original: &RawValue) -> Result<(), HostError> {
    let parent = intent_parent(path)?;
    let _lease = lock_originals(parent)?;
    if read_original(path)?.get() != original.get() {
        return Err(HostError::Ownership(
            "local import intent changed during settlement observation".into(),
        ));
    }
    fs::remove_file(path)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

fn intent_parent(path: &Path) -> Result<&Path, HostError> {
    path.parent()
        .ok_or_else(|| HostError::Ownership("import intent has no owner directory".into()))
}

#[derive(Deserialize)]
struct ImportCheckpoint {
    artifact_intent: Option<Box<RawValue>>,
    project_complete: bool,
}

fn observe_import_checkpoint(
    reader: &ProductReaderSelection,
    source: &ResolvedSource,
    intent: &RawValue,
    output: &OutputOptions,
    environment: &Environment,
) -> Result<ImportCheckpoint, HostError> {
    #[derive(Deserialize)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum ResultKind {
        ImportStatus { report: Option<ImportCheckpoint> },
    }
    #[derive(Deserialize)]
    struct Readback {
        target: ProjectEffectTarget,
        result: ResultKind,
    }
    let request = super::online_project_request(
        source,
        super::ProjectQuery::ImportStatus {
            intent: intent.to_owned(),
        },
    )?;
    let reply = reader
        .read(&request, output.invocation_id.as_str(), environment)
        .map_err(product_error)?;
    let Readback { target, result } =
        serde_json::from_str(reply.payload().get()).map_err(|_| {
            HostError::Ownership(
                "original import checkpoint could not be observed; use import-status".into(),
            )
        })?;
    if source.issuing_store.as_deref() != Some(&target.issuing_store)
        || source
            .database
            .as_ref()
            .is_some_and(|database| database != &target.database)
    {
        return Err(HostError::Ownership(
            "original import checkpoint target differs".into(),
        ));
    }
    let ResultKind::ImportStatus { report } = result;
    report.ok_or_else(|| {
        HostError::Ownership("the original import has no acknowledged checkpoint".into())
    })
}

fn recovery(
    root: &Path,
    source: &ResolvedSource,
    issuer: &str,
    operation: &str,
    policy: &RedactionPolicy,
) -> Option<NextAction> {
    original_lookup(root, source, issuer, operation, policy, false)
}

fn original_lookup(
    root: &Path,
    source: &ResolvedSource,
    issuer: &str,
    operation: &str,
    policy: &RedactionPolicy,
    import: bool,
) -> Option<NextAction> {
    let mut argv = vec![
        root.join("scripts/pse").to_str()?.to_owned(),
        "-C".into(),
        root.to_str()?.into(),
        "project".into(),
        if import {
            "import-status".into()
        } else {
            "acknowledgement".into()
        },
        format!("--reader={}", source.reader),
        format!("--state={}", source.state.to_str()?),
        format!("--issuing-store={issuer}"),
        format!("--operation={operation}"),
    ];
    if let Some(database) = &source.database {
        argv.push(format!("--database={database}"));
    }
    if let Some(timeout) = source.timeout_ms {
        argv.push(format!("--timeout-ms={timeout}"));
    }
    argv.iter()
        .all(|word| policy.redact(word) == *word)
        .then_some(NextAction::Command {
        argv,
        reason: if import {
            "read the exact original import checkpoint before considering recovery".into()
        } else {
            "look up the exact original project acknowledgement before considering another effect".into()
        },
    })
}

fn export_lookup(
    root: &Path, source: &ResolvedSource, issuer: &str, operation: &str, destination: &Path,
    policy: &RedactionPolicy, ready: bool,
) -> Option<NextAction> {
    let mut argv = vec![
        root.join("scripts/pse").to_str()?.into(), "-C".into(), root.to_str()?.into(), "project".into(),
        if ready { "export-recover".into() } else { "export-status".into() },
        format!("--reader={}", source.reader), format!("--issuing-store={issuer}"),
        format!("--operation={operation}"),
    ];
    if ready {
        argv.push(format!("--state={}", source.state.to_str()?));
        if let Some(database) = &source.database { argv.push(format!("--database={database}")); }
        if let Some(timeout) = source.timeout_ms { argv.push(format!("--timeout-ms={timeout}")); }
    }
    argv.push(destination.to_str()?.into());
    argv.iter().all(|word| policy.redact(word) == *word).then_some(NextAction::Command {
        argv, reason: if ready { "publish only the already complete original staged deliverable".into() }
        else { "read the exact original publication before considering recovery".into() },
    })
}

pub(super) fn dispatch(
    action: Action,
    root: &Path,
    selected: Option<&Path>,
    output: &OutputOptions,
    environment: &Environment,
) -> Result<i32, HostError> {
    let root = root.canonicalize()?;
    enum Input {
        Ready(RequestAction),
        Navigation(Option<PathBuf>),
        ImportIssue(PathBuf, String, Option<String>),
        Import(PathBuf, Option<PathBuf>),
        ImportRecover(Option<PathBuf>),
        ImportStatus(Option<PathBuf>),
        ImportOpen(Option<PathBuf>),
        ImportForget,
        Export(ExportKind, PathBuf, Option<PathBuf>),
        ExportRecover(PathBuf),
    }
    let (command, target, request) = match action {
        Action::Create {
            target,
            project,
            title,
        } => (
            "project create",
            target,
            Input::Ready(RequestAction::Create { project, title }),
        ),
        Action::Navigate { target, intent } => {
            ("project navigate", target, Input::Navigation(intent))
        }
        Action::Acknowledgement { target } => (
            "project acknowledgement",
            target,
            Input::Ready(RequestAction::Acknowledgement),
        ),
        Action::ImportIssue {
            target,
            input,
            title,
            branch_name,
        } => (
            "project import-issue",
            target,
            Input::ImportIssue(input, title, branch_name),
        ),
        Action::Import {
            target,
            input,
            intent,
        } => ("project import", target, Input::Import(input, intent)),
        Action::ImportRecover { target, receipt } => (
            "project import-recover",
            target,
            Input::ImportRecover(receipt),
        ),
        Action::ImportStatus { target, intent } => {
            ("project import-status", target, Input::ImportStatus(intent))
        }
        Action::ImportOpen { target, intent } => ("project import-open", target, Input::ImportOpen(intent)),
        Action::ImportForget { target } => ("project import-forget", target, Input::ImportForget),
        Action::Export { target, destination, intent } => ("project export", target, Input::Export(ExportKind::Pack, destination, intent)),
        Action::ExportCohort { target, destination, intent } => ("project export-cohort", target, Input::Export(ExportKind::Cohort, destination, intent)),
        Action::ExportResult { target, destination, intent } => ("project export-result", target, Input::Export(ExportKind::Result, destination, intent)),
        Action::ExportAnalysis { target, destination, intent } => ("project export-analysis", target, Input::Export(ExportKind::Analysis, destination, intent)),
        Action::ExportRecover { target, destination } => ("project export-recover", target, Input::ExportRecover(destination)),
    };
    if command == "project import-status" {
        output.validate_for::<super::ProductView<'static>>(command)?;
    } else {
        output.validate_for::<ProjectEffectView>(command)?;
    }
    let mut source = match target
        .source
        .resolve(&root, selected, environment, output)?
    {
        Ok(source) => source,
        Err(code) => return Ok(code),
    };
    if let (Some(expected), Some(explicit)) = (&source.issuing_store, &target.issuing_store)
        && expected != explicit
    {
        return Err(HostError::Ownership(
            "explicit issuing store differs from the configured target".into(),
        ));
    }
    let issuer = source
        .issuing_store
        .clone()
        .or(target.issuing_store)
        .ok_or_else(|| {
            HostError::Ownership(
                "direct project effects require the expected --issuing-store".into(),
            )
        })?;
    source.issuing_store = Some(issuer.clone());
    let reader =
        ProductReaderSelection::open(&root, environment, &source.reader).map_err(product_error)?;
    let resolve_path = |path| -> Result<PathBuf, HostError> {
        Ok(std::path::absolute(super::super::source_path(selected, path)?)?)
    };
    let export_destination = match &request {
        Input::Export(_, destination, _) | Input::ExportRecover(destination) => Some(resolve_path(destination.clone())?),
        _ => None,
    };
    let is_import = matches!(
        request,
        Input::ImportIssue(..)
            | Input::Import(..)
            | Input::ImportRecover(..)
            | Input::ImportStatus(..)
            | Input::ImportOpen(..)
            | Input::ImportForget
    );
    let original = if is_import {
        Some(original_path(
            &root,
            environment,
            &source,
            &issuer,
            &target.operation,
        )?)
    } else {
        None
    };
    let retained_path = || {
        original.as_deref().ok_or_else(|| {
            HostError::Ownership("project action has no original import intent".into())
        })
    };
    let policy = RedactionPolicy::from_inputs(environment);
    let mut response = Envelope::response(
        command,
        "original project operation observed",
        ProjectEffectView {
            reader: reader.id().to_owned(),
            operation: policy.redact(&target.operation),
            target: None,
            disposition: Disposition::Unprepared,
            acknowledgement: None,
            import: None,
            publication: None,
        },
    );
    response.observation.scope = "project_operation".into();
    let supported = if export_destination.is_some() {
        reader.supports_project_export()
    } else if command == "project import-open" {
        reader.supports_project_import_open()
    } else if is_import {
        reader.supports_project_import()
    } else {
        reader.supports_project_perform()
    };
    if !supported.map_err(product_error)? {
        if command == "project import-status" {
            return Err(HostError::Ownership("selected capture does not support import status; prepare the current product reader".into()));
        }
        response.summary = "selected capture does not support project effects".into();
        response.outcome = CommandOutcome::BoundaryFailure;
        response.subject_state = SubjectState::Unprepared;
        if let (Some(program), Some(checkout)) = (root.join("scripts/pse").to_str(), root.to_str())
            && [program, checkout]
                .iter()
                .all(|word| policy.redact(word) == *word)
        {
            response.next.push(NextAction::Command {
                argv: vec![
                    program.into(),
                    "-C".into(),
                    checkout.into(),
                    "observe".into(),
                    "prepare".into(),
                ],
                reason: "prepare a captured product image with the original effect protocol".into(),
            });
        }
        return Ok(output.emit(response)?);
    }
    let load_intent = |input| {
        let intent = match input {
            Some(input) => original_input(selected, Some(input)),
            None => read_original(retained_path()?),
        }?;
        if !product_reader::project_import_intent_matches(&intent, &target.operation, &issuer) {
            return Err(HostError::Ownership(
                "original import intent differs from the selected operation or store".into(),
            ));
        }
        Ok(intent)
    };
    if let Input::ImportStatus(intent) = request {
        let intent = load_intent(intent)?;
        let request =
            super::online_project_request(&source, super::ProjectQuery::ImportStatus { intent })?;
        // A typed passive query must use the read schema and owner output.
        return super::emit_project_read(command, &reader, &request, output, environment);
    }
    if matches!(request, Input::ImportForget) {
        let intent = load_intent(None)?;
        let checkpoint = observe_import_checkpoint(&reader, &source, &intent, output, environment)?;
        if !checkpoint.project_complete {
            return Err(HostError::Ownership(
                "original import is incomplete; keep its intent for recovery".into(),
            ));
        }
        release_original(retained_path()?, &intent)?;
        response.summary = "settled original import intent released from this checkout".into();
        response.payload.disposition = Disposition::ImportForgotten;
        response.subject_state = SubjectState::Succeeded;
        response.settlement = Settlement::Complete;
        return Ok(output.emit(response)?);
    }
    let resolve_pack = |path| -> Result<NativePath, HostError> {
        Ok(resolve_path(path)?.into())
    };
    let request = match request {
        Input::Ready(request) => request,
        Input::Navigation(intent) => {
            let path = intent
                .map(|path| super::super::source_path(selected, path))
                .transpose()?;
            let NavigationIntent { project, request } = navigation_intent(path.as_deref())?;
            RequestAction::Navigate { project, request }
        }
        Input::ImportIssue(input, title, branch_name) => {
            let path = retained_path()?;
            match fs::symlink_metadata(path) {
                Ok(_) => return Err(HostError::Ownership("this operation already has an original import intent; use import-status or import".into())),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
            RequestAction::ImportIssue {
                input: resolve_pack(input)?,
                title,
                branch_name,
            }
        }
        Input::Import(input, intent) => RequestAction::Import {
            input: resolve_pack(input)?,
            intent: load_intent(intent)?,
        },
        Input::ImportOpen(intent) => RequestAction::ImportOpen { intent: load_intent(intent)? },
        Input::Export(kind, destination, intent) => RequestAction::Export {
            destination: resolve_pack(destination)?, request: kind.request(original_input(selected, intent)?),
        },
        Input::ExportRecover(destination) => RequestAction::ExportRecover { destination: resolve_pack(destination)? },
        Input::ImportRecover(Some(receipt)) => {
            let ImportRecovery { intent, original } = serde_json::from_str(
                original_input(selected, Some(receipt))?.get(),
            )
            .map_err(|_| {
                HostError::Ownership("invalid original project import recovery receipt".into())
            })?;
            RequestAction::ImportRecover { intent, original }
        }
        Input::ImportRecover(None) => {
            let intent = load_intent(None)?;
            let checkpoint =
                observe_import_checkpoint(&reader, &source, &intent, output, environment)?;
            let original = checkpoint.artifact_intent.ok_or_else(|| {
                HostError::Ownership(
                    "the original import has no active artifact to recover; use import-status"
                        .into(),
                )
            })?;
            RequestAction::ImportRecover { intent, original }
        }
        Input::ImportStatus(_) | Input::ImportForget => {
            return Err(HostError::Ownership(
                "passive import action reached the performing owner".into(),
            ));
        }
    };
    let request = serde_json::value::to_raw_value(&Request {
        protocol: product_reader::PROJECT_PERFORM_PROTOCOL,
        version: 1,
        operation: &target.operation,
        target: Target {
            state: source.state.clone().into(),
            database: source.database.as_deref(),
            issuing_store: &issuer,
        },
        timeout_ms: source.timeout_ms,
        action: request,
    })
    .map_err(io::Error::other)?;
    if request.get().len() > product_reader::PROJECT_PERFORM_REQUEST_BYTES {
        return Err(HostError::Ownership(
            "project operation exceeds its request byte bound".into(),
        ));
    }
    let reply = reader.perform_project(
        &request,
        &target.operation,
        &issuer,
        source.database.as_deref(),
        environment,
    );
    let mut export_status = None;
    match reply {
        Ok(reply) => {
            let expected_outcome = match &reply.outcome {
                ProjectEffectOutcome::ImportIssued { .. } => command == "project import-issue",
                ProjectEffectOutcome::Imported { .. } => command == "project import",
                ProjectEffectOutcome::ImportRecovered { .. } => command == "project import-recover",
                ProjectEffectOutcome::ImportOpened { .. } => command == "project import-open",
                ProjectEffectOutcome::Export { .. } => export_destination.is_some(),
                ProjectEffectOutcome::Acknowledged { .. } | ProjectEffectOutcome::Absent => {
                    !is_import && export_destination.is_none()
                }
                ProjectEffectOutcome::Refused { .. } | ProjectEffectOutcome::Unresolved { .. } => {
                    true
                }
            };
            if !expected_outcome {
                return Err(HostError::Ownership(
                    "captured project outcome differs from the requested action".into(),
                ));
            }
            response.payload.target = reply.target.map(|mut observed| {
                if source.database.is_none() {
                    source.database = Some(observed.database.clone());
                }
                observed.issuing_store = policy.redact(&observed.issuing_store);
                observed.namespace = policy.redact(&observed.namespace);
                observed.database = policy.redact(&observed.database);
                observed
            });
            response.settlement = Settlement::Complete;
            match reply.outcome {
                ProjectEffectOutcome::Acknowledged {
                    mut acknowledgement,
                } => {
                    response.subject_state = SubjectState::Succeeded;
                    response.payload.disposition = Disposition::Acknowledged;
                    acknowledgement.key = policy.redact(&acknowledgement.key);
                    acknowledgement.kind = policy.redact(&acknowledgement.kind);
                    acknowledgement.project = policy.redact(&acknowledgement.project);
                    acknowledgement.branch =
                        acknowledgement.branch.map(|value| policy.redact(&value));
                    acknowledgement.revision =
                        acknowledgement.revision.map(|value| policy.redact(&value));
                    response.payload.acknowledgement = Some(acknowledgement);
                }
                ProjectEffectOutcome::Absent => {
                    response.summary =
                        "original acknowledgement absent; effect remains unproved".into();
                    response.payload.disposition = Disposition::Absent;
                    response.subject_state = SubjectState::Unknown;
                    response.settlement = Settlement::Unknown;
                }
                ProjectEffectOutcome::Refused { failure } => {
                    response.summary = "original project operation refused".into();
                    response.payload.disposition = Disposition::Refused;
                    response.subject_state = SubjectState::Failed;
                    response.outcome = CommandOutcome::Failure;
                    response.failures.push(*failure);
                }
                ProjectEffectOutcome::ImportIssued { intent } => {
                    publish_original(retained_path()?, &intent)?;
                    response.payload.disposition = Disposition::ImportIssued;
                    response.summary =
                        "original import intent retained by checkout and operation identity".into();
                    response.subject_state = SubjectState::Succeeded;
                }
                ProjectEffectOutcome::Imported { report } => {
                    response.payload.disposition = Disposition::Imported;
                    response.payload.import = safe_original(&report, &policy);
                    response.subject_state = SubjectState::Succeeded;
                }
                ProjectEffectOutcome::ImportRecovered { report } => {
                    response.payload.disposition = Disposition::ImportRecovered;
                    response.payload.import = safe_original(&report, &policy);
                    #[derive(Deserialize)]
                    struct RecoveryReport {
                        state: String,
                        import: ImportCheckpoint,
                    }
                    let observed: RecoveryReport =
                        serde_json::from_str(report.get()).map_err(io::Error::other)?;
                    if observed.import.project_complete {
                        response.subject_state = SubjectState::Succeeded;
                    } else {
                        response.subject_state = SubjectState::Unknown;
                        if observed.state == "original_cleanup_pending" {
                            response.settlement = Settlement::Incomplete;
                        }
                    }
                }
                ProjectEffectOutcome::ImportOpened { source } => {
                    response.payload.disposition = Disposition::ImportOpened;
                    response.payload.import = safe_original(&source, &policy);
                    response.subject_state = SubjectState::Succeeded;
                }
                ProjectEffectOutcome::Export { status } => {
                    let observed: ExportStatus = serde_json::from_str(status.get()).map_err(io::Error::other)?;
                    export_status = Some(observed);
                    (response.payload.disposition, response.subject_state, response.settlement, response.summary) = observed.presentation();
                    response.payload.publication = safe_original(&status, &policy);
                }
                ProjectEffectOutcome::Unresolved {
                    failure,
                    import_status,
                    publication_status,
                    readback_failure,
                } => {
                    response.payload.disposition = Disposition::Unresolved;
                    response.payload.import = import_status
                        .as_deref()
                        .and_then(|value| safe_original(value, &policy));
                    response.payload.publication = publication_status.as_deref().and_then(|value| safe_original(value, &policy));
                    export_status = publication_status.as_deref().map(|value| serde_json::from_str(value.get())).transpose().map_err(io::Error::other)?;
                    response.failures.push(*failure);
                    response
                        .failures
                        .extend(readback_failure.map(|failure| *failure));
                }
            }
            response.child_exit = Some(
                if reply.cleanup.is_some()
                    || matches!(
                        response.payload.disposition,
                        Disposition::Refused | Disposition::Unresolved
                    )
                {
                    2
                } else {
                    0
                },
            );
            if let Some(cleanup) = reply.cleanup {
                response.settlement = Settlement::Unknown;
                response.outcome = CommandOutcome::BoundaryFailure;
                response.failures.push(cleanup);
            }
        }
        Err(error) => {
            response.payload.disposition = Disposition::Unresolved;
            response
                .failures
                .push(ErrorRecord::from_host(&product_error(error), &policy));
        }
    }
    if matches!(response.payload.disposition, Disposition::Unresolved) {
        response.summary = "original project operation settlement is unresolved".into();
        response.outcome = CommandOutcome::BoundaryFailure;
        response.subject_state = SubjectState::Unknown;
        response.settlement = if matches!(export_status, Some(ExportStatus::Pending | ExportStatus::Ready)) {
            Settlement::Incomplete
        } else { Settlement::Unknown };
        response.observation.completeness = Completeness::Partial;
    }
    if let Some(destination) = &export_destination {
        if matches!(response.payload.disposition, Disposition::Unresolved | Disposition::PublicationUnknown | Disposition::PublicationPending | Disposition::PublicationReady)
            || response.settlement == Settlement::Unknown {
            response.next.extend(export_lookup(&root, &source, &issuer, &target.operation, destination, &policy, matches!(export_status, Some(ExportStatus::Ready))));
        }
    } else if matches!(
        response.payload.disposition,
        Disposition::Absent | Disposition::Unresolved
    ) {
        let next = if is_import {
            original_lookup(&root, &source, &issuer, &target.operation, &policy, true)
        } else {
            recovery(&root, &source, &issuer, &target.operation, &policy)
        };
        response.next.extend(next);
    }
    Ok(output.emit(response)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;

    #[test]
    fn original_import_intent_is_exclusive_and_release_rechecks_exact_bytes() {
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary.path().join("imports/original.json");
        let intent = RawValue::from_string(
            r#"{"publication":{"operation":"op","issuing_store":"store"}}"#.into(),
        )
        .unwrap();
        let changed = RawValue::from_string(
            r#"{"publication":{"operation":"op","issuing_store":"other"}}"#.into(),
        )
        .unwrap();
        assert!(product_reader::project_import_intent_matches(
            &intent, "op", "store"
        ));
        assert!(!product_reader::project_import_intent_matches(
            &changed, "op", "store"
        ));
        publish_original(&path, &intent).unwrap();
        assert!(publish_original(&path, &changed).is_err());
        assert!(release_original(&path, &changed).is_err());
        assert_eq!(read_original(&path).unwrap().get(), intent.get());
        release_original(&path, &intent).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn project_effect_selects_original_recovery_without_replaying_the_mutation() {
        let action = crate::cli::Cli::try_parse_from([
            "pse",
            "project",
            "create",
            "--reader=reader",
            "--state=/tmp/canonical state",
            "--issuing-store=issuer",
            "--operation=original",
            "--database=selected",
            "p",
            "title",
        ])
        .unwrap();
        let Some(crate::cli::Command::Project {
            action: super::super::ProjectAction::Effect(Action::Create { target, .. }),
        }) = action.command
        else {
            panic!("expected an explicit project effect");
        };
        let root = Path::new("/tmp/project root");
        let source = ResolvedSource {
            state: target.source.state.unwrap(),
            database: target.source.database,
            issuing_store: None,
            reader: target.source.reader.unwrap(),
            timeout_ms: None,
        };
        let issuer = target.issuing_store.unwrap();
        let Some(NextAction::Command { argv, .. }) = recovery(
            root,
            &source,
            &issuer,
            &target.operation,
            &Default::default(),
        ) else {
            panic!("expected an exact recovery action");
        };
        assert!(argv.contains(&"acknowledgement".into()));
        assert!(argv.contains(&"--operation=original".into()));
        assert!(argv.contains(&"--state=/tmp/canonical state".into()));
        assert!(!argv.contains(&"create".into()));
        assert!(crate::cli::Cli::try_parse_from(argv).is_ok());
        let secret = RedactionPolicy::new([("TOKEN".into(), "original".into())]);
        assert!(recovery(root, &source, &issuer, &target.operation, &secret).is_none());
    }
}
