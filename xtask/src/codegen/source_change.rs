// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored checkout preview and durable publication of exact generator output.
mod anchor;
mod delta;
#[path = "../../../scripts/source-change-proposal.rs"]
mod handoff;
mod snapshot;
#[cfg(test)]
mod tests;

use super::{normal_relative, publication};
use anyhow::{Context, Result, bail, ensure};
use clap::{Args, Subcommand, ValueEnum};
use handoff::{ProposalFile, ProposalHandoff};
use pse_codegen::codegen::provenance::protocol::Scope;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, VecDeque},
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
};

const MAX_STATUS: u64 = 4 * 1024 * 1024;
const MAX_LOG_STREAM: usize = 32 * 1024;
const SNAPSHOT_CLAIM: &str = "snapshot-claim";
const FAILURE_LOG: &str = "failed-stage.log";

#[derive(Debug, Args)]
pub(crate) struct Selection {
    /// Resolved checkout-private state from the native receiver.
    #[arg(long, hide = true)]
    state_root: PathBuf,
    #[command(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    Preview {
        #[arg(long, value_enum)]
        case: Case,
        #[arg(long = "input")]
        inputs: Vec<PathBuf>,
        #[arg(long)]
        compatibility: Option<String>,
        #[arg(long, hide = true)]
        proposal_stdin: bool,
    },
    Status {
        id: String,
    },
    Apply {
        id: String,
    },
    Recover {
        id: String,
    },
    Discard {
        id: String,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Case {
    OptionalProjectDescription,
    RecordedResultFieldRename,
}

impl Case {
    fn declaration(self) -> &'static str {
        match self {
            Self::OptionalProjectDescription => "crates/pse-schema/src/catalog/projects.rs",
            Self::RecordedResultFieldRename => "crates/pse-model/src/recorded_result.rs",
        }
    }

    fn consumer(self) -> &'static str {
        match self {
            Self::OptionalProjectDescription => "crates/pse-operations/src/canonical_projects.rs",
            Self::RecordedResultFieldRename => {
                "crates/pse-runtime/src/workflow/result_selection.rs"
            }
        }
    }

    fn compatibility(self) -> &'static str {
        match self {
            Self::OptionalProjectDescription => {
                "Historical project rows retain absent description; explicit descriptions remain descriptive metadata."
            }
            Self::RecordedResultFieldRename => {
                "Proto and JSON retain field, tag 7, presence and original field-selection meaning."
            }
        }
    }

    fn test_args(self) -> [&'static str; 5] {
        match self {
            Self::OptionalProjectDescription => [
                "test",
                "pse-operations",
                "--lib",
                "-E",
                "test(=canonical_projects::optional_project_description_preserves_historical_and_explicit_rows)",
            ],
            Self::RecordedResultFieldRename => [
                "test",
                "pse-runtime",
                "--lib",
                "-E",
                "test(workflow::result_selection::tests::)",
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SourceSpan {
    byte_offset: usize,
    byte_length: usize,
    line: usize,
    column: usize,
    column_encoding: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthoredInput {
    path: PathBuf,
    head_sha256: Option<String>,
    working_sha256: String,
    span: Option<SourceSpan>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum State {
    Generating,
    GenerationFailed,
    Ready,
    Applying,
    Applied,
}

#[derive(Debug, Serialize, Deserialize)]
struct Failure {
    code: String,
    stage: String,
    original_exit: Option<i32>,
    original_signal: Option<i32>,
    original_log: Option<String>,
    original_log_sha256: Option<String>,
    recovery: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Status {
    version: u32,
    preview_id: String,
    state: State,
    case: Case,
    checkout_revision: String,
    checkout_sha256: String,
    snapshot_sha256: Option<String>,
    authored: Vec<AuthoredInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proposal: Option<ProposalHandoff>,
    compatibility: String,
    compatibility_note: Option<String>,
    checks: Vec<Vec<String>>,
    completed_stages: Vec<String>,
    affected_packages: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    affected: Option<AffectedEvidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    surrealql_validation: Vec<crate::surrealql_tools::FileValidation>,
    generated: Vec<delta::Change>,
    surviving: Vec<delta::SurvivingFile>,
    failure: Option<Failure>,
}

/// Native affected details, without integration targets or unrelated response content.
#[derive(Debug, Serialize, Deserialize)]
struct AffectedEvidence {
    completeness: String,
    changed_packages: Vec<String>,
    graph: serde_json::Value,
    widening: Vec<serde_json::Value>,
    generators: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Record {
    checkout: PathBuf,
    original_scopes: Vec<Scope>,
    inflight_stage: Option<String>,
    authored_sha256: Option<String>,
    native_sha256: Option<String>,
    delta: Option<delta::Delta>,
    status: Status,
}

struct Operation {
    directory: PathBuf,
    _lease: File,
}

fn is_hex(value: &str, width: usize) -> bool {
    value.len() == width
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut text, "{byte:02x}");
    }
    text
}

fn require_private(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        metadata.is_dir(),
        "source-change state is not an owned directory"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        ensure!(
            metadata.permissions().mode() & 0o077 == 0,
            "source-change state must be owner-private"
        );
    }
    Ok(())
}

fn private_directory(path: &Path) -> Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        builder.mode(0o700);
    }
    builder.create(path)?;
    require_private(path)
}

fn snapshot_claim(state: &Path) -> Result<Option<String>> {
    let path = state.join(SNAPSHOT_CLAIM);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    ensure!(metadata.is_file(), "invalid source-change snapshot claim");
    let owner = String::from_utf8(snapshot::bounded_read(&path, 32)?)?;
    ensure!(
        is_hex(&owner, 32),
        "invalid source-change snapshot claim identity"
    );
    Ok(Some(owner))
}

impl Operation {
    fn create(state: &Path) -> Result<(String, Self)> {
        private_directory(state)?;
        let mut builder = tempfile::Builder::new();
        builder.prefix("operation-").rand_bytes(24);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            builder.permissions(fs::Permissions::from_mode(0o700));
        }
        let temporary = builder.tempdir_in(state)?;
        let name = temporary
            .path()
            .file_name()
            .context("operation directory has no identity")?;
        let id = hex(&Sha256::digest(name.as_encoded_bytes())[..16]);
        let directory = state.join(&id);
        ensure!(
            !directory.exists(),
            "source-change operation identity collision"
        );
        fs::rename(temporary.path(), &directory)?;
        File::open(state)?.sync_all()?;
        Ok((id.clone(), Self::open(state, &id)?))
    }

    fn open(state: &Path, id: &str) -> Result<Self> {
        ensure!(is_hex(id, 32), "invalid source-change preview identity");
        require_private(state)?;
        let directory = state.join(id);
        require_private(&directory)?;
        let lease = publication::lease(&directory)?;
        Ok(Self {
            directory,
            _lease: lease,
        })
    }

    fn read(&self, checkout: &Path, id: &str) -> Result<Record> {
        super::reject_symlinks(&self.directory, Path::new("record.json"))?;
        let record: Record = serde_json::from_slice(&snapshot::bounded_read(
            &self.directory.join("record.json"),
            MAX_STATUS * 2,
        )?)?;
        ensure!(
            record.checkout == checkout
                && record.status.preview_id == id
                && record.status.version == 1,
            "source-change preview belongs to another checkout or version"
        );
        Ok(record)
    }

    fn write(&self, record: &Record) -> Result<()> {
        let status = serde_json::to_vec(&record.status)?;
        let retained = serde_json::to_vec(record)?;
        ensure!(
            status.len() as u64 <= MAX_STATUS && retained.len() as u64 <= MAX_STATUS * 2,
            "source-change record exceeds its retained byte limit"
        );
        snapshot::atomic_write(&self.directory.join("record.json"), &retained)
    }

    fn snapshot(&self) -> PathBuf {
        self.directory.join("checkout")
    }

    fn custody(&self) -> PathBuf {
        self.directory.join("bytes")
    }

    // The checkout publication lease serializes this claim with other operations.
    fn claim_snapshot(&self, id: &str) -> Result<()> {
        let state = self
            .directory
            .parent()
            .context("source-change state root absent")?;
        match snapshot_claim(state)? {
            Some(owner) => ensure!(
                owner == id,
                "preview {owner} owns the private snapshot; recover or discard it"
            ),
            None => snapshot::atomic_write(&state.join(SNAPSHOT_CLAIM), id.as_bytes())?,
        }
        Ok(())
    }

    fn release_claim(&self, id: &str) -> Result<()> {
        let state = self
            .directory
            .parent()
            .context("source-change state root absent")?;
        if snapshot_claim(state)?.as_deref() == Some(id) {
            fs::remove_file(state.join(SNAPSHOT_CLAIM))?;
            File::open(state)?.sync_all()?;
        }
        Ok(())
    }

    fn remove_snapshot(&self) -> Result<()> {
        self.remove_directory(&self.snapshot())
    }

    fn remove_custody(&self) -> Result<()> {
        self.remove_directory(&self.custody())
    }

    fn remove_directory(&self, path: &Path) -> Result<()> {
        match fs::remove_dir_all(path) {
            Ok(()) => File::open(&self.directory)?.sync_all()?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
        Ok(())
    }
}

pub(crate) fn run(checkout: &Path, selection: Selection) -> Result<i32> {
    let checkout = checkout.canonicalize()?;
    let state = selection.state_root;
    ensure!(
        state.is_absolute() && !state.starts_with(&checkout),
        "source-change state must be outside the checkout"
    );
    let recovery = matches!(&selection.action, Action::Recover { .. });
    match selection.action {
        Action::Preview {
            case,
            inputs,
            compatibility,
            proposal_stdin,
        } => {
            let proposal = proposal_stdin
                .then(|| read_proposal(std::io::stdin().lock()))
                .transpose()?;
            preview(&checkout, &state, case, inputs, compatibility, proposal)
        }
        Action::Status { id } => {
            let operation = Operation::open(&state, &id)?;
            emit(&operation.read(&checkout, &id)?.status, 0)
        }
        Action::Apply { id } | Action::Recover { id } => {
            let operation = Operation::open(&state, &id)?;
            let mut record = operation.read(&checkout, &id)?;
            if recovery {
                match record.status.state {
                    State::Generating | State::GenerationFailed => {
                        return generate(&operation, &mut record);
                    }
                    State::Ready => {
                        let _lease = publication::lease(&checkout)?;
                        operation.remove_snapshot()?;
                        operation.release_claim(&id)?;
                        if record
                            .status
                            .failure
                            .as_ref()
                            .is_some_and(|failure| failure.stage == "release-snapshot")
                        {
                            record.status.failure = None;
                            operation.write(&record)?;
                        }
                        return emit(&record.status, i32::from(record.status.failure.is_some()));
                    }
                    State::Applying | State::Applied => (),
                }
            }
            apply(&operation, &mut record, recovery)
        }
        Action::Discard { id } => {
            let operation = Operation::open(&state, &id)?;
            let record = operation.read(&checkout, &id)?;
            ensure!(
                record.status.state != State::Applying,
                "publication has unsettled files; recover before discard"
            );
            let _lease = publication::lease(&checkout)?;
            operation.remove_snapshot()?;
            operation.release_claim(&id)?;
            fs::remove_dir_all(&operation.directory)?;
            File::open(state)?.sync_all()?;
            println!(
                "{}",
                serde_json::json!({"version": 1, "preview_id": id, "state": "discarded"})
            );
            Ok(0)
        }
    }
}

fn emit(status: &Status, code: i32) -> Result<i32> {
    println!("{}", serde_json::to_string(status)?);
    Ok(code)
}

fn read_proposal(input: impl Read) -> Result<ProposalHandoff> {
    let mut bytes = Vec::new();
    input
        .take(handoff::MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= handoff::MAX_BYTES,
        "source-change proposal handoff exceeds its byte limit"
    );
    let proposal: ProposalHandoff = serde_json::from_slice(&bytes)?;
    proposal.validate().map_err(anyhow::Error::msg)?;
    Ok(proposal)
}

fn authored_inputs(
    root: &Path,
    case: Case,
    inputs: Vec<PathBuf>,
    scopes: &[Scope],
    proposal: &[ProposalFile],
) -> Result<Vec<AuthoredInput>> {
    ensure!(
        inputs.len() <= 32,
        "source-change accepts up to 32 supplemental inputs"
    );
    let inputs = inputs
        .into_iter()
        .chain(proposal.iter().map(|file| file.path.clone()))
        .chain([
            PathBuf::from(case.declaration()),
            PathBuf::from(case.consumer()),
        ])
        .map(|path| {
            let path = if path.is_absolute() {
                path.strip_prefix(root)?.to_owned()
            } else {
                path
            };
            ensure!(
                normal_relative(&path) && !scopes.iter().any(|scope| scope.contains(&path)),
                "source-change input must select an authored checkout file"
            );
            Ok(path)
        })
        .collect::<Result<BTreeSet<_>>>()?;
    let selected = snapshot::paths(root)?;
    inputs
        .into_iter()
        .map(|path| {
            ensure!(
                selected.contains(&path),
                "authored input is outside the captured working source: {}",
                path.display()
            );
            let working = publication::observe(root, &path)?.context("authored input is absent")?;
            let working_sha256 = hex(&working.identity);
            if let Some(expected) = proposal.iter().find(|file| file.path == path) {
                ensure!(
                    expected.after_sha256 == working_sha256,
                    "authored file does not match the reviewed proposal postimage: {}",
                    path.display()
                );
            }
            let head_sha256 = snapshot::head_file_identity(root, &path)?;
            let span = if path == Path::new(case.declaration()) {
                ensure!(
                    head_sha256.as_ref() != Some(&working_sha256),
                    "selected declaration has no authored change relative to HEAD"
                );
                let bytes = snapshot::bounded_read(&root.join(&path), 2 * 1024 * 1024)?;
                ensure!(
                    hex(&Sha256::digest(&bytes)) == working_sha256,
                    "authored declaration changed during observation"
                );
                Some(anchor::locate(case, std::str::from_utf8(&bytes)?)?)
            } else {
                None
            };
            Ok(AuthoredInput {
                path,
                head_sha256,
                working_sha256,
                span,
            })
        })
        .collect()
}

fn preview(
    checkout: &Path,
    state: &Path,
    case: Case,
    inputs: Vec<PathBuf>,
    compatibility_note: Option<String>,
    proposal: Option<ProposalHandoff>,
) -> Result<i32> {
    ensure!(
        compatibility_note
            .as_ref()
            .is_none_or(|text| !text.trim().is_empty() && text.len() <= 2048),
        "compatibility note must be 1–2048 bytes"
    );
    let lease = publication::lease(checkout)?;
    private_directory(state)?;
    ensure!(
        !state.canonicalize()?.starts_with(checkout),
        "source-change state must be outside the checkout"
    );
    if let Some(owner) = snapshot_claim(state)? {
        bail!("preview {owner} owns the private snapshot; recover or discard it");
    }
    let original_scopes = delta::scopes(checkout)?;
    let checkout_revision = snapshot::revision(checkout)?;
    if let Some(proposal) = &proposal {
        proposal.validate().map_err(anyhow::Error::msg)?;
        let checkout_id = state
            .parent()
            .and_then(Path::file_name)
            .and_then(std::ffi::OsStr::to_str);
        ensure!(
            checkout_id == Some(proposal.checkout_id.as_str())
                && checkout_revision == proposal.checkout_revision,
            "source-change proposal checkout or revision no longer matches the selected checkout"
        );
    }
    let proposal_files = proposal
        .as_ref()
        .map_or(&[][..], |proposal| proposal.files.as_slice());
    let original_sha256 = snapshot::fingerprint(checkout, &original_scopes, false)?;
    let authored = authored_inputs(checkout, case, inputs, &original_scopes, proposal_files)?;
    let (preview_id, operation) = Operation::create(state)?;
    let mut record = Record {
        checkout: checkout.to_owned(),
        original_scopes,
        inflight_stage: None,
        authored_sha256: None,
        native_sha256: None,
        delta: None,
        status: Status {
            version: 1,
            preview_id,
            state: State::Generating,
            case,
            checkout_revision,
            checkout_sha256: original_sha256,
            snapshot_sha256: None,
            authored,
            proposal,
            compatibility: case.compatibility().into(),
            compatibility_note,
            checks: vec![
                vec!["just".into(), "codegen-all-check".into()],
                vec!["pse".into(), "inspect".into(), "affected".into()],
                std::iter::once("pse")
                    .chain(case.test_args())
                    .map(str::to_owned)
                    .collect(),
            ],
            completed_stages: Vec::new(),
            affected_packages: Vec::new(),
            affected: None,
            surrealql_validation: Vec::new(),
            generated: Vec::new(),
            surviving: Vec::new(),
            failure: None,
        },
    };
    operation.write(&record)?;
    operation.claim_snapshot(&record.status.preview_id)?;
    drop(lease);
    generate(&operation, &mut record)
}

#[derive(Debug)]
struct ChildFailure {
    status: ExitStatus,
    stdout: Captured,
    stderr: Captured,
}

#[derive(Debug)]
struct Captured {
    bytes: Vec<u8>,
    total: u64,
}

fn drain(mut stream: impl Read, limit: usize) -> Result<Captured> {
    let mut tail = VecDeque::new();
    let mut total = 0_u64;
    let mut buffer = [0; 16 * 1024];
    loop {
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total = total.saturating_add(count as u64);
        let chunk = &buffer[count.saturating_sub(limit)..count];
        let excess = (tail.len() + chunk.len()).saturating_sub(limit);
        tail.drain(..excess);
        tail.extend(chunk);
    }
    Ok(Captured {
        bytes: tail.into(),
        total,
    })
}

fn capture(mut command: Command, stdout_limit: usize) -> Result<Captured> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .context("source-change child stdout unavailable")?;
    let stderr = child
        .stderr
        .take()
        .context("source-change child stderr unavailable")?;
    std::thread::scope(|threads| {
        let stdout = threads.spawn(move || drain(stdout, stdout_limit));
        let stderr = threads.spawn(move || drain(stderr, MAX_LOG_STREAM));
        let status = child.wait()?;
        let stdout = stdout
            .join()
            .map_err(|_| anyhow::anyhow!("stdout capture failed"))??;
        let stderr = stderr
            .join()
            .map_err(|_| anyhow::anyhow!("stderr capture failed"))??;
        if !status.success() {
            return Err(ChildFailure {
                status,
                stdout,
                stderr,
            }
            .into());
        }
        Ok(stdout)
    })
}

impl ChildFailure {
    fn signal(&self) -> Option<i32> {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt as _;
            self.status.signal()
        }
        #[cfg(not(unix))]
        None
    }

    fn detail(&self) -> Vec<u8> {
        let mut log = Vec::new();
        for (name, stream) in [("stdout", &self.stdout), ("stderr", &self.stderr)] {
            let tail = &stream.bytes[stream.bytes.len().saturating_sub(MAX_LOG_STREAM)..];
            log.extend(
                format!(
                    "{name}: retained last {} of {} bytes\n",
                    tail.len(),
                    stream.total
                )
                .into_bytes(),
            );
            log.extend(tail);
            log.push(b'\n');
        }
        log
    }
}

impl std::fmt::Display for ChildFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "source-change owner ended with {}", self.status)
    }
}

impl std::error::Error for ChildFailure {}

fn native_identity(root: &Path) -> Result<String> {
    let mut command = snapshot::isolated_command(root, "scripts/pse-env");
    command.args([
        "--native=compiler,solver",
        "--",
        "sh",
        "scripts/source-change-codegen",
        "native-identity",
    ]);
    let output =
        capture(command, 16 * 1024).context("checking prepared compiler and solver identity")?;
    ensure!(
        !output.bytes.is_empty() && output.total == output.bytes.len() as u64,
        "native identity receipt is missing or oversized"
    );
    Ok(hex(&Sha256::digest(output.bytes)))
}

fn isolated_target(root: &Path, activated: bool) -> Result<()> {
    let mut command = if activated {
        let mut command = snapshot::isolated_command(root, "scripts/pse-env");
        command.args(["--", "cargo"]);
        command
    } else {
        snapshot::isolated_command(root, "cargo")
    };
    command.args([
        "metadata",
        "--no-deps",
        "--locked",
        "--offline",
        "--format-version",
        "1",
    ]);
    let output =
        capture(command, MAX_STATUS as usize).context("checking the snapshot Cargo target")?;
    ensure!(
        output.total == output.bytes.len() as u64,
        "snapshot Cargo metadata exceeded its byte limit"
    );
    let metadata: serde_json::Value = serde_json::from_slice(&output.bytes)?;
    let target = root.canonicalize()?.join("target");
    for field in ["target_directory", "build_directory"] {
        let selected = metadata[field]
            .as_str()
            .context("snapshot Cargo directory is missing")?;
        ensure!(
            Path::new(selected) == target,
            "snapshot Cargo selected a foreign {field}"
        );
    }
    super::reject_symlinks(root, Path::new("target"))?;
    Ok(())
}

fn track_candidate_outputs(root: &Path, scopes: &[Scope]) -> Result<()> {
    let mut child = snapshot::git(root)
        .args([
            "--literal-pathspecs",
            "add",
            "--force",
            "--pathspec-from-file=-",
            "--pathspec-file-nul",
        ])
        .stdin(Stdio::piped())
        .spawn()?;
    let mut stdin = child
        .stdin
        .take()
        .context("Git pathspec input unavailable")?;
    use std::io::Write as _;
    for path in delta::inventory(root, scopes)? {
        stdin.write_all(path.as_os_str().as_encoded_bytes())?;
        stdin.write_all(&[0])?;
    }
    drop(stdin);
    ensure!(
        child.wait()?.success(),
        "could not bind generated paths in the disposable Git index"
    );
    Ok(())
}

fn observe_affected(record: &mut Record, root: &Path) -> Result<()> {
    let mut command = snapshot::isolated_command(root, "scripts/pse");
    command.args(["inspect", "affected", "--format", "json"]);
    let output = capture(command, MAX_STATUS as usize)
        .context("inspecting source reach in the isolated checkout")?;
    ensure!(
        output.total == output.bytes.len() as u64,
        "affected selection exceeded the retained byte limit"
    );
    let (packages, evidence) = affected_evidence(&output.bytes)?;
    let complete = evidence.completeness == "complete";
    record.status.affected_packages = packages;
    record.status.affected = Some(evidence);
    ensure!(complete, "native affected selection is incomplete");
    Ok(())
}

fn affected_evidence(bytes: &[u8]) -> Result<(Vec<String>, AffectedEvidence)> {
    #[derive(Deserialize)]
    struct SelectedPackage {
        name: String,
    }
    #[derive(Deserialize)]
    struct Reach {
        changed_packages: Vec<String>,
        selected_packages: Vec<SelectedPackage>,
        graph: serde_json::Value,
        widening: Vec<serde_json::Value>,
        generators: Vec<serde_json::Value>,
    }
    #[derive(Deserialize)]
    struct Observation {
        completeness: String,
    }
    #[derive(Deserialize)]
    struct Report {
        observation: Observation,
        payload: Reach,
    }
    let Report {
        observation,
        payload,
    } = serde_json::from_slice(bytes)?;
    let packages: Vec<_> = payload
        .selected_packages
        .into_iter()
        .map(|package| package.name)
        .collect();
    let evidence = AffectedEvidence {
        completeness: observation.completeness,
        changed_packages: payload.changed_packages,
        graph: payload.graph,
        widening: payload.widening,
        generators: payload.generators,
    };
    ensure!(
        serde_json::to_vec(&(&packages, &evidence))?.len() <= 512 * 1024,
        "affected projection exceeds its retained byte limit"
    );
    Ok((packages, evidence))
}

fn run_stage(operation: &Operation, record: &mut Record, stage: &str) -> Result<()> {
    let root = operation.snapshot();
    let command = match stage {
        "check-isolated-target" => return isolated_target(&root, false),
        "prepare-host" => {
            let mut command = snapshot::isolated_command(&root, "scripts/bootstrap-host");
            command.arg("--from").arg(&record.checkout);
            command
        }
        "native-preflight" => {
            isolated_target(&root, true)?;
            let live = native_identity(&record.checkout)?;
            ensure!(
                native_identity(&root)? == live,
                "isolated checkout selected different native providers"
            );
            record.native_sha256 = Some(live);
            return Ok(());
        }
        "rust-contracts" | "recorded-result" | "surreal" | "full-codegen" => {
            let mut command = snapshot::isolated_command(&root, "scripts/pse-env");
            command.args([
                "--recipe-default-class",
                "functional",
                "--",
                "sh",
                "scripts/source-change-codegen",
                stage,
            ]);
            command
        }
        "codegen-check" => {
            track_candidate_outputs(&root, &delta::scopes(&root)?)?;
            let mut command = snapshot::isolated_command(&root, "just");
            command.arg("codegen-all-check");
            command
        }
        "affected-selection" => return observe_affected(record, &root),
        "receiving-tests" => {
            let mut command = snapshot::isolated_command(&root, "scripts/pse");
            command.args(record.status.case.test_args());
            command
        }
        _ => bail!("unrecognized source-change stage"),
    };
    capture(command, MAX_LOG_STREAM).context("running the selected source-change stage")?;
    Ok(())
}

fn generator_stages(root: &Path) -> Result<String> {
    let mut command = snapshot::isolated_command(root, "sh");
    command.args(["scripts/source-change-codegen", "--stages"]);
    let output = capture(command, 1024)?;
    ensure!(
        !output.bytes.is_empty() && output.total == output.bytes.len() as u64,
        "ordered generator stage declaration is absent or invalid"
    );
    let stages = String::from_utf8(output.bytes)?;
    let mut seen = BTreeSet::new();
    ensure!(
        stages.lines().all(|stage| matches!(
            stage,
            "rust-contracts" | "recorded-result" | "surreal" | "full-codegen"
        ) && seen.insert(stage)),
        "ordered generator stage declaration contains an unsupported or repeated stage"
    );
    Ok(stages)
}

fn verify_original(record: &Record) -> Result<()> {
    ensure!(
        snapshot::revision(&record.checkout)? == record.status.checkout_revision
            && snapshot::fingerprint(&record.checkout, &record.original_scopes, false)?
                == record.status.checkout_sha256,
        "checkout or inputs changed since source-change preview"
    );
    Ok(())
}

fn generate(operation: &Operation, record: &mut Record) -> Result<i32> {
    record.status.state = State::Generating;
    record.status.failure = None;
    operation.write(record)?;
    let mut active = "snapshot".to_owned();
    let result = (|| {
        {
            let _lease = publication::lease(&record.checkout)?;
            active = "source-preflight".into();
            verify_original(record)?;
            operation.claim_snapshot(&record.status.preview_id)?;
        }
        let root = operation.snapshot();
        if let Some(expected) = &record.status.snapshot_sha256 {
            active = "snapshot-readback".into();
            let observed = delta::scopes(&root)
                .and_then(|scopes| snapshot::fingerprint(&root, &scopes, false));
            let matches = snapshot::revision(&root)
                .is_ok_and(|revision| revision == record.status.checkout_revision)
                && observed.is_ok_and(|sha256| sha256 == *expected);
            if !matches {
                ensure!(
                    record.inflight_stage.is_some(),
                    "retained private snapshot changed; discard this preview and reacquire the source"
                );
                // A killed generator can leave an unreceipted private tree. Replay from
                // the bound checkout rather than accepting any unverified intermediate.
                record.status.snapshot_sha256 = None;
                record.native_sha256 = None;
                record.status.completed_stages.clear();
                record.status.affected_packages.clear();
                record.status.affected = None;
                record.status.surrealql_validation.clear();
            }
        }
        if record.status.snapshot_sha256.is_none() {
            active = "snapshot".into();
            operation.remove_snapshot()?;
            snapshot::create(
                &record.checkout,
                &root,
                &record.status.checkout_revision,
                &record.original_scopes,
            )?;
            let copied = snapshot::fingerprint(&root, &record.original_scopes, false)?;
            ensure!(
                copied == record.status.checkout_sha256,
                "checkout inputs changed during snapshot acquisition"
            );
            record.status.snapshot_sha256 = Some(copied);
            record.status.completed_stages.push("snapshot".into());
            operation.write(record)?;
        }
        if let Some(native) = &record.native_sha256 {
            active = "native-readback".into();
            ensure!(
                native_identity(&record.checkout)? == *native && native_identity(&root)? == *native,
                "prepared compiler or solver changed since source-change preview"
            );
        }
        active = "ordered-generator-stages".into();
        let generators = generator_stages(&root)?;
        let stages = ["check-isolated-target", "prepare-host", "native-preflight"]
            .into_iter()
            .chain(generators.lines())
            .chain(["codegen-check", "affected-selection", "receiving-tests"]);
        for stage in stages {
            if record
                .status
                .completed_stages
                .iter()
                .any(|completed| completed == stage)
            {
                continue;
            }
            active = stage.into();
            record.inflight_stage = Some(stage.into());
            operation.write(record)?;
            let attempt = run_stage(operation, record, stage);
            let scopes = delta::scopes(&root)?;
            record.status.snapshot_sha256 = Some(snapshot::fingerprint(&root, &scopes, false)?);
            record.inflight_stage = None;
            attempt?;
            record.status.completed_stages.push(stage.into());
            operation.write(record)?;
        }
        active = "retain-generated-delta".into();
        let _lease = publication::lease(&record.checkout)?;
        verify_original(record)?;
        let scopes = delta::scopes(&root)?;
        let authored = snapshot::fingerprint(&record.checkout, &scopes, true)?;
        ensure!(
            snapshot::fingerprint(&root, &scopes, true)? == authored,
            "generation or receiving checks changed files outside declared generator outputs"
        );
        operation.remove_custody()?;
        let delta = delta::Delta::stage(&record.checkout, &root, &operation.custody(), scopes)?;
        active = "surrealql-validation".into();
        record.status.surrealql_validation =
            validate_surrealql(&record.status.authored, &delta, &root, &operation.custody())?;
        ensure!(
            record
                .status
                .surrealql_validation
                .iter()
                .all(|file| file.valid),
            "changed SurrealQL postimages failed parse-only validation"
        );
        record.status.completed_stages.push(active.clone());
        active = "retain-generated-delta".into();
        record.status.generated = delta.changes.clone();
        record.authored_sha256 = Some(authored);
        record.delta = Some(delta);
        record.status.completed_stages.push(active.clone());
        record.status.state = State::Ready;
        record.status.failure = None;
        operation.write(record)?;
        active = "release-snapshot".into();
        operation.remove_snapshot()?;
        operation.release_claim(&record.status.preview_id)
    })();
    match result {
        Ok(()) => emit(&record.status, 0),
        Err(error) => {
            if record.status.state != State::Ready {
                record.status.state = State::GenerationFailed;
            }
            failure(operation, record, &active, &error)
        }
    }
}

fn validate_surrealql(
    authored: &[AuthoredInput],
    delta: &delta::Delta,
    root: &Path,
    custody: &Path,
) -> Result<Vec<crate::surrealql_tools::FileValidation>> {
    let changed_authored = authored
        .iter()
        .filter(|input| input.head_sha256.as_ref() != Some(&input.working_sha256))
        .map(|input| {
            (
                input.path.as_path(),
                root.join(&input.path),
                input.working_sha256.as_str(),
            )
        });
    let changed_generated = delta.changes.iter().filter_map(|change| {
        change
            .postimage()
            .map(|(path, sha256)| (path, custody.join(sha256), sha256))
    });
    let inputs: Vec<_> = changed_authored
        .chain(changed_generated)
        .filter(|(path, _, _)| {
            path.extension()
                .is_some_and(|extension| extension == "surql")
        })
        .collect();
    ensure!(
        inputs.len() <= 32
            && inputs
                .iter()
                .all(|(path, _, _)| path.as_os_str().len() <= 1024),
        "changed SurrealQL postimages exceed the retained path limit"
    );
    Ok(inputs
        .into_iter()
        .map(|(path, staged, sha256)| {
            crate::surrealql_tools::validate_file(path, &staged, Some(sha256))
        })
        .collect())
}

fn apply(operation: &Operation, record: &mut Record, recovery: bool) -> Result<i32> {
    if record.status.state == State::Applied {
        if let Err(error) = operation.remove_custody() {
            return failure(operation, record, "release-postimages", &error);
        }
        if record
            .status
            .failure
            .as_ref()
            .is_some_and(|failure| failure.stage == "release-postimages")
        {
            record.status.failure = None;
            operation.write(record)?;
        }
        return emit(&record.status, i32::from(record.status.failure.is_some()));
    }
    ensure!(
        record.status.state == State::Ready || (recovery && record.status.state == State::Applying),
        "apply requires a ready preview and interrupted publication requires recover"
    );
    let mut active = "publication-preflight";
    let result = (|| {
        let _lease = publication::lease(&record.checkout)?;
        let delta = record
            .delta
            .as_ref()
            .context("preview has no generated delta")?;
        ensure!(
            native_identity(&record.checkout)?
                == *record
                    .native_sha256
                    .as_ref()
                    .context("preview has no prepared native binding")?,
            "prepared compiler or solver changed since source-change preview"
        );
        delta.preflight(
            &record.checkout,
            &operation.custody(),
            record.status.state == State::Applying,
        )?;
        ensure!(
            snapshot::revision(&record.checkout)? == record.status.checkout_revision
                && snapshot::fingerprint(&record.checkout, &delta.scopes, true)?
                    == *record
                        .authored_sha256
                        .as_ref()
                        .context("preview has no authored binding")?,
            "checkout revision or authored inputs changed since source-change preview"
        );
        operation.remove_snapshot()?;
        operation.release_claim(&record.status.preview_id)?;
        active = "publish-generated-files";
        record.status.state = State::Applying;
        record.status.failure = None;
        operation.write(record)?;
        delta.publish(&record.checkout, &operation.custody())?;
        record.status.surviving = delta.surviving(&record.checkout);
        ensure!(
            delta.all_postimages(&record.checkout)?,
            "publication readback found an unsettled generated file"
        );
        record.status.completed_stages.push(active.into());
        record.status.state = State::Applied;
        operation.write(record)?;
        active = "release-postimages";
        operation.remove_custody()
    })();
    match result {
        Ok(()) => emit(&record.status, 0),
        Err(error) => {
            if let Some(delta) = &record.delta {
                record.status.surviving = delta.surviving(&record.checkout);
            }
            failure(operation, record, active, &error)
        }
    }
}

fn failure(
    operation: &Operation,
    record: &mut Record,
    stage: &str,
    error: &anyhow::Error,
) -> Result<i32> {
    eprintln!("source-change {stage}: {error:#}");
    let recoverable = !matches!(
        stage,
        "source-preflight" | "snapshot-readback" | "native-readback" | "publication-preflight"
    );
    let code = match stage {
        "source-preflight" | "snapshot-readback" | "native-readback" | "publication-preflight" => {
            "source_change.stale_preview"
        }
        "publish-generated-files" => "source_change.publication_interrupted",
        _ => "source_change.stage_failed",
    };
    let original = error.downcast_ref::<ChildFailure>();
    let original_log_sha256 = original
        .map(|failure| {
            let detail = failure.detail();
            snapshot::atomic_write(&operation.directory.join(FAILURE_LOG), &detail)?;
            Ok::<_, anyhow::Error>(hex(&Sha256::digest(detail)))
        })
        .transpose()?;
    record.status.failure = Some(Failure {
        code: code.into(),
        stage: stage.into(),
        original_exit: original.and_then(|failure| failure.status.code()),
        original_signal: original.and_then(ChildFailure::signal),
        original_log: original_log_sha256.as_ref().map(|_| FAILURE_LOG.to_owned()),
        original_log_sha256,
        recovery: recoverable.then(|| {
            vec![
                "pse".into(),
                "source-change".into(),
                "recover".into(),
                record.status.preview_id.clone(),
            ]
        }),
    });
    operation.write(record)?;
    emit(&record.status, 1)
}
