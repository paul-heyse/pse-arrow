// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Git-owned worktrees with explicit preparation stages and independently observed users.
use crate::{
    HostError,
    artifacts::Content,
    envelope::{
        CommandOutcome, Completeness, Envelope, ErrorRecord, RedactionPolicy, Settlement,
        SubjectState,
    },
    environment::Environment,
    generation::{self, HostTool, PreparedGeneration},
    invocation::InvocationStore,
    paths::CheckoutState,
    repository::{self, WorktreeObservation},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use gix::bstr::ByteSlice as _;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    ffi::{OsStr, OsString},
    fs::{self, File, OpenOptions},
    io::{Read as _, Write as _},
    os::unix::{
        ffi::{OsStrExt as _, OsStringExt as _},
        fs::{MetadataExt as _, OpenOptionsExt as _, PermissionsExt as _},
        process::ExitStatusExt as _,
    },
    path::{Component, Path, PathBuf},
    process::Command,
    sync::{Arc, atomic::AtomicBool},
    time::UNIX_EPOCH,
};

/// One explicitly selected sibling checkout. Names never become shell source.
#[derive(Clone, Debug)]
pub struct NewRequest {
    /// Single native path component beneath the source checkout's parent.
    pub name: OsString,
    /// Explicit Git base selector; HEAD by default at the CLI boundary.
    pub base: OsString,
    /// New branch; absence selects the same native name as the checkout.
    pub branch: Option<OsString>,
    /// Start the existing detached workspace check after successful preparation/registration.
    pub warm: bool,
}
/// Original child output and status, separate from the overall multi-stage operation.
#[derive(Debug, Serialize, JsonSchema)]
pub struct CommandReceipt {
    /// Exact native executable operand.
    pub program: OsString,
    /// Literal argument boundaries and native bytes.
    pub arguments: Vec<OsString>,
    /// Original normal exit status, absent on a signal.
    pub exit_code: Option<i32>,
    /// Original terminating signal, absent on a normal exit.
    pub signal: Option<i32>,
    /// Original stdout, encoded without Unicode loss.
    pub stdout: Content,
    /// Original stderr, encoded without Unicode loss.
    pub stderr: Content,
}
/// Independent state of one original stage.
#[derive(Clone, Copy, Debug, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StageState {
    /// The original performing owner completed this stage.
    Succeeded,
    /// The original operation refused or failed.
    Failed,
    /// The stage was explicitly unselected or its input was absent.
    Skipped,
    /// A failed prerequisite prevented this stage from starting.
    Pending,
}
/// One original creation/preparation result; no rollback is inferred.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Stage {
    /// Stable stage name.
    pub stage: String,
    /// Independent completion state.
    pub state: StageState,
    /// Scope/interpretation of the observation.
    pub detail: String,
    /// Original child evidence when a child actually ran.
    pub command: Option<CommandReceipt>,
    /// Original typed source failure when the operation refused before a child result.
    pub failure: Option<ErrorRecord>,
    /// Concrete recovery route after failure; it does not authorize deletion.
    pub remedy: Option<String>,
}
/// Local configuration copy facts; configuration values are never rendered.
#[derive(Debug, Serialize, JsonSchema)]
pub struct LocalEnvironment {
    /// Original source file mode, including permission/special bits.
    pub mode: u32,
    /// Exact copied byte count.
    pub bytes: u64,
    /// True when identical destination bytes/mode already existed.
    pub already_present: bool,
}
/// Existing checkout/store identity, projected from the original A04 owner.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Registration {
    /// Authoritative inspected checkout identity.
    pub checkout_id: String,
    /// Shared repository relation.
    pub repository_id: String,
    /// Existing private state namespace, outside the worktree.
    pub state: OsString,
    /// Original canonical checkout directory, including its filesystem identity.
    pub checkout: ObservedDirectory,
    /// Original Git administration directory for this checkout incarnation.
    pub administration: ObservedDirectory,
    /// Ordinary checkout-local Cargo target, with identity when present.
    pub target: Option<ObservedDirectory>,
    /// External checkout state directory identity when present.
    pub state_directory: Option<ObservedDirectory>,
}
/// Filesystem evidence for association. It confers no release authority.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ObservedDirectory {
    /// Exact path bytes selected by the original owner.
    pub path: OsString,
    /// Filesystem device.
    pub device: u64,
    /// Directory inode on that filesystem.
    pub inode: u64,
    /// Creation time when the filesystem supplies it, as seconds and nanoseconds since the epoch.
    pub created: Option<(u64, u32)>,
}
impl ObservedDirectory {
    fn observe(path: &Path) -> Result<Option<Self>, HostError> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        if !metadata.is_dir() {
            return Err(HostError::Ownership(format!(
                "worktree association expected a directory at {}",
                path.display(),
            )));
        }
        Ok(Some(Self {
            path: path.as_os_str().to_owned(),
            device: metadata.dev(),
            inode: metadata.ino(),
            created: metadata
                .created()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|elapsed| (elapsed.as_secs(), elapsed.subsec_nanos())),
        }))
    }

    fn required(path: &Path) -> Result<Self, HostError> {
        Self::observe(path)?.ok_or_else(|| {
            HostError::Ownership(format!(
                "worktree association directory is absent at {}",
                path.display(),
            ))
        })
    }
}

fn registration(
    path: &Path,
    state: &CheckoutState,
    observed_git_dir: Option<&Path>,
) -> Result<Registration, HostError> {
    let path = path.canonicalize()?;
    let mut repository = gix::open_opts(&path, gix::open::Options::isolated())
        .map_err(repository::RepositoryError::from)?;
    repository
        .make_paths_absolute()
        .map_err(repository::RepositoryError::from)?;
    let git_dir = repository.git_dir().canonicalize()?;
    if let Some(expected) = observed_git_dir
        && expected.canonicalize()? != git_dir
    {
        return Err(HostError::Ownership(
            "Git worktree administration changed during observation".into(),
        ));
    }
    Ok(Registration {
        checkout_id: state.checkout_id.as_str().into(),
        repository_id: state.repository_id.as_str().into(),
        state: state.root.as_os_str().to_owned(),
        checkout: ObservedDirectory::required(&path)?,
        administration: ObservedDirectory::required(&git_dir)?,
        target: ObservedDirectory::observe(&path.join("target"))?,
        state_directory: ObservedDirectory::observe(&state.root)?,
    })
}
/// Truthful multi-stage result; a failed preparation leaves the actual checkout intact.
#[derive(Debug, Serialize, JsonSchema)]
pub struct NewReport {
    /// Original explicit source checkout.
    pub source: OsString,
    /// Exact sibling destination, including non-UTF8 name bytes.
    pub destination: OsString,
    /// Selected new branch.
    pub branch: OsString,
    /// Original explicit base selector.
    pub base: OsString,
    /// Git itself returned successful creation.
    pub git_created: bool,
    /// Filesystem presence observed at return; absence is not cleanup authority.
    pub destination_exists: bool,
    /// Required preparation and existing state registration completed.
    pub ready: bool,
    /// Whether the caller selected the optional detached workspace check.
    pub warm_requested: bool,
    /// Copied local-environment facts, absent when no source file existed or copying failed.
    pub local_environment: Option<LocalEnvironment>,
    /// Authenticated complete destination host generation.
    pub host: Option<PreparedGeneration>,
    /// Registered existing operational namespace.
    pub registration: Option<Registration>,
    /// Existing warm-launch acknowledgement; launch is not build completion.
    pub warm: Option<Envelope<serde_json::Value>>,
    /// Original independent stage outcomes, including pending stages.
    pub stages: Vec<Stage>,
}
fn content(bytes: Vec<u8>) -> Content {
    match String::from_utf8(bytes) {
        Ok(text) => Content::Utf8(text),
        Err(error) => Content::Base64(STANDARD.encode(error.into_bytes())),
    }
}
fn checkout_environment(environment: &Environment) -> Result<Environment, HostError> {
    // Git itself owns this list, including local repository variables added in future releases.
    let output = Command::new("git")
        .args(["rev-parse", "--local-env-vars"])
        .env_clear()
        .envs(environment)
        .output()?;
    if !output.status.success() {
        return Err(HostError::Ownership(
            "Git could not identify its repository-local environment variables".into(),
        ));
    }
    let names = std::str::from_utf8(&output.stdout).map_err(std::io::Error::other)?;
    let mut selected = environment.clone();
    for name in names.lines() {
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err(HostError::Ownership(
                "Git returned an unsupported repository-local environment name".into(),
            ));
        }
        selected.remove(OsStr::new(name));
    }
    Ok(selected)
}
fn run(
    program: &Path,
    arguments: &[OsString],
    cwd: &Path,
    environment: &Environment,
) -> Result<CommandReceipt, HostError> {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(cwd)
        .env_clear()
        .envs(environment)
        .output()?;
    Ok(CommandReceipt {
        program: program.as_os_str().to_owned(),
        arguments: arguments.to_vec(),
        exit_code: output.status.code(),
        signal: output.status.signal(),
        stdout: content(output.stdout),
        stderr: content(output.stderr),
    })
}
fn success(command: &CommandReceipt) -> bool {
    command.exit_code == Some(0) && command.signal.is_none()
}
fn stage(name: &str, state: StageState, detail: impl Into<String>) -> Stage {
    Stage {
        stage: name.into(),
        state,
        detail: detail.into(),
        command: None,
        failure: None,
        remedy: None,
    }
}
fn failed(name: &str, error: HostError, policy: &RedactionPolicy, remedy: &str) -> Stage {
    Stage {
        failure: Some(ErrorRecord::from_typed(&error, policy)),
        remedy: Some(remedy.into()),
        ..stage(
            name,
            StageState::Failed,
            "original owner refused this stage",
        )
    }
}
fn command_stage(
    name: &str,
    result: Result<CommandReceipt, HostError>,
    policy: &RedactionPolicy,
    remedy: &str,
) -> Stage {
    match result {
        Ok(command) => {
            let state = if success(&command) {
                StageState::Succeeded
            } else {
                StageState::Failed
            };
            Stage {
                state,
                command: Some(command),
                remedy: (state == StageState::Failed).then(|| remedy.into()),
                ..stage(
                    name,
                    state,
                    "original child output/status retained; later stages have independent outcomes",
                )
            }
        }
        Err(error) => failed(name, error, policy, remedy),
    }
}
fn pending(report: &mut NewReport, names: &[&str]) {
    report.stages.extend(names.iter().map(|name| {
        if *name == "warm_launch" && !report.warm_requested {
            stage(
                name,
                StageState::Skipped,
                "--warm was not selected; no workspace build was started",
            )
        } else {
            stage(
                name,
                StageState::Pending,
                "a required earlier stage did not complete; checkout was not deleted",
            )
        }
    }));
}
fn finish(mut report: NewReport) -> Envelope<NewReport> {
    report.destination_exists = Path::new(&report.destination).exists();
    let failed = report
        .stages
        .iter()
        .any(|stage| stage.state == StageState::Failed);
    let mut response = Envelope::response(
        "worktree new",
        if report.ready {
            "worktree prepared; warm-launch outcome remains independently recorded"
        } else {
            "worktree creation/preparation observations retained; partial checkout was not deleted"
        },
        report,
    );
    response.observation.scope =
        "explicit_git_worktree_creation_and_original_preparation_stages".into();
    response.observation.limitations.push("Git creation, local configuration, immutable host preparation, registration and warm launch have separate outcomes; no automatic rollback, direnv change or build-completion inference".into());
    if failed {
        response.outcome = CommandOutcome::Failure;
        response.subject_state = SubjectState::Failed;
        response.observation.completeness = Completeness::Partial;
    }
    response
}
fn copied_environment(
    source: &Path,
    destination: &Path,
) -> Result<Option<LocalEnvironment>, HostError> {
    let mut input = match File::open(source.join(".envrc.local")) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let metadata = input.metadata()?;
    if !metadata.is_file() {
        return Err(HostError::Ownership(
            "selected .envrc.local is not a regular file".into(),
        ));
    }
    let mode = metadata.mode() & 0o7777;
    let target = destination.join(".envrc.local");
    let mut output = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&target)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let target_metadata = fs::symlink_metadata(&target)?;
            if !target_metadata.is_file() || target_metadata.file_type().is_symlink() {
                return Err(HostError::Ownership("destination .envrc.local already exists with an unsupported file type; it was not replaced".into()));
            }
            let mut bytes = Vec::new();
            input.read_to_end(&mut bytes)?;
            if fs::read(&target)? != bytes || target_metadata.mode() & 0o7777 != mode {
                return Err(HostError::Ownership("destination .envrc.local differs from selected source; concurrent configuration was not replaced".into()));
            }
            return Ok(Some(LocalEnvironment {
                mode,
                bytes: bytes.len() as u64,
                already_present: true,
            }));
        }
        Err(error) => return Err(error.into()),
    };
    let bytes = std::io::copy(&mut input, &mut output)?;
    output.set_permissions(fs::Permissions::from_mode(mode))?;
    output.flush()?;
    output.sync_all()?;
    Ok(Some(LocalEnvironment {
        mode,
        bytes,
        already_present: false,
    }))
}
/// Create one sibling worktree and explicitly prepare it. No failure path removes files,
/// branches or worktree administration, even if another participant starts using the checkout.
pub fn create(
    root: &Path,
    request: NewRequest,
    environment: &Environment,
) -> Result<Envelope<NewReport>, HostError> {
    let root = root.canonicalize()?;
    let name = Path::new(&request.name);
    if name.components().count() != 1
        || !matches!(name.components().next(), Some(Component::Normal(_)))
        || request.name.as_bytes().starts_with(b"-")
    {
        return Err(HostError::Ownership(
            "worktree name must be one non-option native path component".into(),
        ));
    }
    let destination = root
        .parent()
        .ok_or_else(|| HostError::Ownership("source checkout has no sibling directory".into()))?
        .join(name);
    let selected_environment = checkout_environment(environment)?;
    let environment = &selected_environment;
    let branch = request.branch.unwrap_or_else(|| request.name.clone());
    let policy = RedactionPolicy::from_inputs(environment);
    let mut report = NewReport {
        source: root.as_os_str().to_owned(),
        destination: destination.as_os_str().to_owned(),
        branch: branch.clone(),
        base: request.base.clone(),
        git_created: false,
        destination_exists: false,
        ready: false,
        warm_requested: request.warm,
        local_environment: None,
        host: None,
        registration: None,
        warm: None,
        stages: vec![],
    };
    let arguments = vec![
        "-C".into(),
        root.as_os_str().to_owned(),
        "worktree".into(),
        "add".into(),
        "-b".into(),
        branch,
        "--".into(),
        destination.as_os_str().to_owned(),
        request.base,
    ];
    let git = command_stage(
        "git_add",
        run(Path::new("git"), &arguments, &root, environment),
        &policy,
        "inspect the original Git error and existing destination; no automatic cleanup occurred",
    );
    report.git_created = git.state == StageState::Succeeded;
    report.stages.push(git);
    if !report.git_created {
        pending(
            &mut report,
            &[
                "local_environment",
                "host_preparation",
                "registration",
                "warm_launch",
            ],
        );
        return Ok(finish(report));
    }
    match copied_environment(&root, &destination) {
        Ok(Some(copy)) => {
            report.local_environment = Some(copy);
            report.stages.push(stage(
                "local_environment",
                StageState::Succeeded,
                "original .envrc.local bytes/mode copied without rendering configuration values",
            ));
        }
        Ok(None) => report.stages.push(stage(
            "local_environment",
            StageState::Skipped,
            "source checkout has no .envrc.local; repository defaults remain selected",
        )),
        Err(error) => {
            report.stages.push(failed("local_environment",error,&policy,"supply the source .envrc.local with its original mode, then explicitly bootstrap this existing checkout"));
            pending(
                &mut report,
                &["host_preparation", "registration", "warm_launch"],
            );
            return Ok(finish(report));
        }
    }
    let mut preparation = command_stage(
        "host_preparation",
        run(
            &destination.join("scripts/bootstrap-host"),
            &["--from".into(), root.as_os_str().to_owned()],
            &destination,
            environment,
        ),
        &policy,
        "run this checkout's scripts/bootstrap-host --from SOURCE after resolving the original preparation failure; preserve the existing checkout",
    );
    if preparation.state == StageState::Succeeded {
        match generation::prepared(&destination) {
            Ok(Some(host)) => report.host = Some(host),
            Ok(None) => preparation = failed(
                "host_preparation",
                HostError::Ownership(
                    "bootstrap returned success without an authenticated completed host generation"
                        .into(),
                ),
                &policy,
                "run explicit scripts/bootstrap-host for this existing checkout and inspect its completed generation",
            ),
            Err(error) => {
                preparation = failed(
                    "host_preparation",
                    error,
                    &policy,
                    "repair preparation through this checkout's explicit bootstrap owner; preserve original failure evidence",
                )
            }
        }
    }
    let prepared = preparation.state == StageState::Succeeded;
    report.stages.push(preparation);
    if !prepared {
        pending(&mut report, &["registration", "warm_launch"]);
        return Ok(finish(report));
    }
    match CheckoutState::resolve(&destination, environment)
        .map_err(HostError::from)
        .and_then(|state| {
            InvocationStore::new(state.clone()).register()?;
            registration(&destination, &state, None)
        }) {
        Ok(registration) => {
            report.registration = Some(registration);
            report.stages.push(stage(
                "registration",
                StageState::Succeeded,
                "existing original checkout-bound private invocation namespace initialized",
            ));
            report.ready = true;
        }
        Err(error) => {
            report.stages.push(failed("registration",error,&policy,"repair the original checkout state namespace; the prepared checkout remains present"));
            pending(&mut report, &["warm_launch"]);
            return Ok(finish(report));
        }
    }
    if request.warm {
        let result = report
            .host
            .as_ref()
            .ok_or_else(|| HostError::Ownership("prepared generation is absent".into()))
            .and_then(|host| host.executable(&destination, HostTool::Pse))
            .and_then(|program| {
                run(
                    &program,
                    &[
                        "-C".into(),
                        destination.as_os_str().to_owned(),
                        "check".into(),
                        "--workspace".into(),
                        "--detach".into(),
                        "--format".into(),
                        "json".into(),
                    ],
                    &destination,
                    environment,
                )
            });
        let mut warm = command_stage(
            "warm_launch",
            result,
            &policy,
            "inspect the original warm-launch evidence; the checkout is prepared and no duplicate warm run was started",
        );
        if warm.state == StageState::Succeeded {
            let decoded = warm
                .command
                .as_ref()
                .and_then(|receipt| match &receipt.stdout {
                    Content::Utf8(text) => Some(text),
                    Content::Base64(_) => None,
                })
                .ok_or_else(|| {
                    HostError::Ownership(
                        "warm launcher did not return a UTF-8 acknowledgement".into(),
                    )
                })
                .and_then(|text| {
                    serde_json::from_str(text).map_err(|error| {
                        HostError::Ownership(format!("warm acknowledgement is invalid: {error}"))
                    })
                });
            match decoded {
                Ok(acknowledgement) => report.warm = Some(acknowledgement),
                Err(error) => {
                    warm.state = StageState::Failed;
                    warm.failure = Some(ErrorRecord::from_typed(&error, &policy));
                    warm.remedy=Some("warm submission may exist; inspect original destination invocation history before retrying".into());
                }
            }
        }
        report.stages.push(warm);
    } else {
        report.stages.push(stage(
            "warm_launch",
            StageState::Skipped,
            "--warm was not selected; no workspace build was started",
        ));
    }
    Ok(finish(report))
}

/// Direct ancestry comparison, with the selected base resolved before per-worktree traversal.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Divergence {
    /// Commit selected by the list operation's original base selector.
    pub base_commit: String,
    /// Commits reachable only from this worktree's HEAD.
    pub ahead: usize,
    /// Commits reachable only from the selected base.
    pub behind: usize,
}
/// Original native service state, distinct from a terminal payload row or a missing PID.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum NativeUse {
    /// Original matching service/cgroup still has an actual native user.
    Live,
    /// Original performing owner authenticated native drain.
    Drained,
    /// Unknown evidence is retained rather than interpreted as no user.
    Unavailable {
        /// Original observation/refusal detail.
        reason: String,
    },
}
/// Original invocation history and independently checked native use.
#[derive(Debug, Serialize, JsonSchema)]
pub struct InvocationUse {
    /// Existing original invocation ID.
    pub invocation: crate::envelope::CliInvocationId,
    /// Original explicit command.
    pub command: String,
    /// Existing session association, never a process-ownership claim.
    pub session: Option<String>,
    /// Original selection timestamp; not current process activity.
    pub created_unix_ms: u64,
    /// An original terminal projection exists, independently of native resources.
    pub terminal_present: bool,
    /// Original performing owner's settlement projection, when readable.
    pub settlement: Option<Settlement>,
    /// Actual native owner observation, never inferred from the fields above.
    pub native: NativeUse,
}
/// One independently observed main/linked worktree; unavailable facts keep their failures.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ListedWorktree {
    /// Native Git administration/path/branch identity.
    pub git: WorktreeObservation,
    /// Existing checkout/state association, absent when the original path cannot be verified.
    pub registration: Option<Registration>,
    /// Staged/worktree/untracked changes counted as distinct dirty paths.
    pub dirty_paths: Option<usize>,
    /// Direct ancestry comparison; absence has an original failure or unborn/bare context.
    pub divergence: Option<Divergence>,
    /// Original checkout-bound invocations with independent native observations.
    pub invocations: Vec<InvocationUse>,
    /// Latest original metadata creation time; no editor/filesystem activity is fabricated.
    pub last_recorded_creation_ms: Option<u64>,
    /// Every independent per-worktree source/observation failure.
    pub failures: Vec<ErrorRecord>,
}
/// Read-only full inventory, not a removal permit or a stop/settlement operation.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ListReport {
    /// Explicit selected Git base selector.
    pub base: OsString,
    /// Resolved immutable base commit when available.
    pub base_commit: Option<String>,
    /// Every independently readable native worktree in original byte-path order.
    pub worktrees: Vec<ListedWorktree>,
}
fn reachable(
    repository: &gix::Repository,
    id: gix::ObjectId,
) -> Result<BTreeSet<gix::ObjectId>, HostError> {
    repository
        .rev_walk([id])
        .all()
        .map_err(repository::RepositoryError::from)?
        .map(|info| {
            info.map(|info| info.id)
                .map_err(|error| HostError::Repository(error.into()))
        })
        .collect()
}
async fn activity(
    root: &Path,
    state: CheckoutState,
    environment: &Environment,
    policy: &RedactionPolicy,
) -> Result<(Vec<InvocationUse>, Vec<ErrorRecord>), HostError> {
    let store = InvocationStore::new(state);
    let history = store.history::<serde::de::IgnoredAny>()?;
    let mut failures = history
        .failures
        .iter()
        .map(|error| ErrorRecord::from_typed(error, policy))
        .collect::<Vec<_>>();
    let store = &store;
    let observations =
        futures_util::future::join_all(history.records.into_iter().map(|metadata| async move {
            let id = metadata.selection.invocation_id;
            let result = store.result::<serde::de::IgnoredAny, serde_json::Value>(&id);
            let mut errors = Vec::new();
            let (terminal_present, settlement) = match result {
                Ok(Some(result)) => (true, Some(result.settlement)),
                Ok(None) => (false, None),
                Err(error) => {
                    errors.push(ErrorRecord::from_typed(&error, policy));
                    (false, None)
                }
            };
            let native = match original_native_drained(root, store, &id, environment).await {
                Ok(true) => NativeUse::Drained,
                Ok(false) => NativeUse::Live,
                Err(error) => {
                    let reason = error.to_string();
                    errors.push(ErrorRecord::from_typed(&error, policy));
                    NativeUse::Unavailable { reason }
                }
            };
            (
                InvocationUse {
                    invocation: id,
                    command: metadata.selection.command,
                    session: metadata.selection.session,
                    created_unix_ms: metadata.created_unix_ms,
                    terminal_present,
                    settlement,
                    native,
                },
                errors,
            )
        }))
        .await;
    let mut records = Vec::new();
    for (record, error) in observations {
        records.push(record);
        failures.extend(error);
    }
    Ok((records, failures))
}
#[derive(Deserialize)]
struct CustodyOwner {
    version: u32,
}
async fn original_native_drained(
    root: &Path,
    store: &InvocationStore,
    id: &crate::envelope::CliInvocationId,
    environment: &Environment,
) -> Result<bool, HostError> {
    let directory = store.directory(id)?;
    let owner: CustodyOwner = directory
        .evidence(crate::invocation::InvocationEvidence::Custody)?
        .ok_or_else(|| {
            HostError::Ownership(
                "original native custody is absent; terminal metadata does not establish drain"
                    .into(),
            )
        })?;
    // A routing hint only: each original owner validates its full custody/metadata and
    // establishes drain from actual manager/kernel facts, never this version or a terminal row.
    match owner.version {
        4 => crate::foreground::observe_recorded_drain(root, id, environment).await,
        5 => crate::managed::supervisor::observe_recorded(root, id, environment)
            .await
            .map(|observed| observed.native_drained),
        version => Err(HostError::Ownership(format!(
            "unsupported original native custody version {version}; no drain inferred"
        ))),
    }
}
/// List actual Git/metadata/native facts without host preparation, worktree mutation,
/// capacity settlement or a declaration that unknown readers/resources do not exist.
pub async fn list(
    root: &Path,
    base: &OsStr,
    environment: &Environment,
) -> Result<Envelope<ListReport>, HostError> {
    let policy = RedactionPolicy::from_inputs(environment);
    let inventory = repository::observe_worktrees(root)?;
    let mut failures = inventory
        .failures
        .iter()
        .map(|failure| ErrorRecord::from_typed(&failure.error, &policy))
        .collect::<Vec<_>>();
    let repository = gix::open(root).map_err(repository::RepositoryError::from)?;
    let base_id = repository
        .rev_parse_single(base.as_bytes().as_bstr())
        .map(|id| id.detach())
        .map_err(repository::RepositoryError::from);
    let base_id = match base_id {
        Ok(id) => Some(id),
        Err(error) => {
            failures.push(ErrorRecord::from_typed(&error, &policy));
            None
        }
    };
    let base_reachable = base_id.map(|id| reachable(&repository, id)).transpose();
    let base_reachable = match base_reachable {
        Ok(value) => value,
        Err(error) => {
            failures.push(ErrorRecord::from_typed(&error, &policy));
            None
        }
    };
    let base_reachable = &base_reachable;
    let policy = &policy;
    let rows =
        futures_util::future::join_all(inventory.entries.into_iter().map(|git| async move {
            let mut row = ListedWorktree {
                git,
                registration: None,
                dirty_paths: None,
                divergence: None,
                invocations: vec![],
                last_recorded_creation_ms: None,
                failures: vec![],
            };
            if let Some(path) = &row.git.worktree {
                let path = PathBuf::from(OsString::from_vec(path.as_bytes().to_vec()));
                match repository::observe(&path, Arc::new(AtomicBool::new(false))) {
                    Ok(observation) => row.dirty_paths = Some(observation.dirty_path_count),
                    Err(error) => row.failures.push(ErrorRecord::from_typed(&error, policy)),
                }
                if let (Some(base), Some(base_reachable)) = (base_id, base_reachable.as_ref()) {
                    let comparison = (|| {
                        let repository =
                            gix::open(&path).map_err(repository::RepositoryError::from)?;
                        let head = repository
                            .head_id()
                            .map_err(repository::RepositoryError::from)?
                            .detach();
                        let current = reachable(&repository, head)?;
                        Ok::<_, HostError>(Divergence {
                            base_commit: base.to_string(),
                            ahead: current.difference(base_reachable).count(),
                            behind: base_reachable.difference(&current).count(),
                        })
                    })();
                    match comparison {
                        Ok(comparison) => row.divergence = Some(comparison),
                        Err(error) => row.failures.push(ErrorRecord::from_typed(&error, policy)),
                    }
                }
                let use_observation = async {
                    let state = CheckoutState::resolve(&path, environment)?;
                    let admin = OsStr::from_bytes(row.git.git_dir.as_bytes());
                    row.registration = Some(registration(&path, &state, Some(Path::new(admin)))?);
                    activity(&path, state, environment, policy).await
                }
                .await;
                match use_observation {
                    Ok((records, errors)) => {
                        row.last_recorded_creation_ms =
                            records.iter().map(|record| record.created_unix_ms).max();
                        row.invocations = records;
                        row.failures.extend(errors);
                    }
                    Err(error) => row.failures.push(ErrorRecord::from_typed(&error, policy)),
                }
            }
            row
        }))
        .await;
    for row in &rows {
        failures.extend(row.failures.iter().cloned());
    }
    let mut response = Envelope::response(
        "worktree list",
        "native Git inventory and independently observed checkout users; no removal authority",
        ListReport {
            base: base.to_owned(),
            base_commit: base_id.map(|id| id.to_string()),
            worktrees: rows,
        },
    );
    response.failures = failures;
    response.observation.scope =
        "git_worktrees_direct_commit_ancestry_and_original_invocation_users".into();
    response.observation.limitations.push("last activity is original invocation metadata creation time only; terminal rows do not clear native/fixture/artifact protection; this is not an atomic snapshot or a removal permit".into());
    if !response.failures.is_empty() {
        response.observation.completeness = Completeness::Partial;
    }
    Ok(response)
}

#[cfg(test)]
mod tests;
