// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed native workflow input and required producer protocol, separate from physical drain.
use super::{payload::PayloadCommand, supervisor};
use crate::{
    HostError, HostManager,
    envelope::{
        CliInvocationId, CommandOutcome, Completeness, Envelope, RedactionPolicy, Settlement,
    },
    generation::{HostTool, PreparedGeneration},
    invocation::{DeferredTestDrain, InvocationDirectory, InvocationEvidence, InvocationStore},
    stream_records::{Frame, RecordFramer},
    test_inventory::CapturedReport,
    test_workflow::{
        WorkflowError,
        execute::{Action, Input, ReportObservation, TerminalRecord},
    },
};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use sha2::{Digest as _, Sha256};
use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{self, Read},
    num::NonZeroUsize,
    os::unix::fs::{MetadataExt as _, OpenOptionsExt as _},
    path::{Path, PathBuf},
    sync::Arc,
};

pub(crate) mod borrowed;

/// The original fixed helper or its required transport failed.
#[derive(Debug, thiserror::Error)]
pub enum WorkflowOwnerError {
    /// Original local evidence or pipe failure.
    #[error("managed workflow I/O: {0}")]
    Io(#[from] io::Error),
    /// Original strict input/terminal/report decode failure.
    #[error("managed workflow JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Original native ownership/identity failure.
    #[error(transparent)]
    Host(#[from] HostError),
    /// Original operation-owned construction or execution failure.
    #[error(transparent)]
    Operation(#[from] WorkflowError),
    /// A checked fixed input, frame or original artifact association failed.
    #[error("managed workflow: {0}")]
    Invalid(String),
}
pse_diagnostics::impl_diagnostic! {
    WorkflowOwnerError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(this) { match this { Self::Host(error)=>Some(error), Self::Operation(error)=>Some(error), _=>None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}
fn invalid(message: impl Into<String>) -> WorkflowOwnerError {
    WorkflowOwnerError::Invalid(message.into())
}
fn host(error: impl Into<WorkflowOwnerError>) -> HostError {
    HostError::WorkflowOwner(Box::new(error.into()))
}
pub(crate) fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(super) struct ReleasedWorkflow {
    captured: CapturedReport,
    paired_result_sha256: String,
}

impl ReleasedWorkflow {
    pub(super) fn publish_compact(
        self,
        store: &InvocationStore,
        directory: &InvocationDirectory,
        drain: DeferredTestDrain,
        policy: &RedactionPolicy,
    ) -> Result<(), HostError> {
        store.publish_released_compact_test_result(
            directory,
            &self.captured,
            drain,
            &self.paired_result_sha256,
            policy,
        )
    }
}

/// Retire only private workflow controls after the original foreground owner has
/// published the reconciled result and proved the entire workload drained.
pub(crate) fn release_foreground_images(
    store: &InvocationStore,
    directory: &InvocationDirectory,
    terminal: &super::result::TerminalResult<super::result::ManagedSummary>,
) -> Result<(), HostError> {
    let result = terminal.response();
    if !result.payload.borrowed_reservation && release_interrupted_listing(directory, result)? {
        return Ok(());
    }
    let Some(workflow) = &result.payload.workflow else {
        return Ok(());
    };
    if result.settlement != Settlement::Complete
        || result.payload.native_lifetime_pending != Some(false)
        || result.payload.borrowed_reservation
        || !workflow.terminal.required_capture_complete
    {
        return Ok(());
    }
    release_original_images(store, directory, workflow.terminal.action).map(drop)
}

/// After the finalizer or recorded control proves native worker drain, authenticate
/// original completion independently. The paired records remain immutable.
pub(super) fn release_detached_images(
    store: &InvocationStore,
    directory: &InvocationDirectory,
    result: &Envelope<super::result::ManagedSummary>,
) -> Result<Option<ReleasedWorkflow>, HostError> {
    let Some(workflow) = &result.payload.workflow else {
        return Ok(None);
    };
    if &result.invocation_id != directory.id()
        || result.settlement != Settlement::Unknown
        || result.payload.native_lifetime_pending != Some(true)
        || result.payload.borrowed_reservation
        || !complete_capture(result)
        || !workflow.terminal.required_capture_complete
    {
        return Ok(None);
    }
    release_original_images(store, directory, workflow.terminal.action).map(Some)
}

pub(crate) fn release_reconciled_images(
    store: &InvocationStore,
    id: &CliInvocationId,
) -> Result<(), HostError> {
    let directory = store.directory(id)?;
    if let Some(result) =
        store.result::<serde::de::IgnoredAny, super::result::ManagedSummary>(id)?
        && let Some(released) = release_detached_images(store, &directory, &result)?
    {
        released.publish_compact(
            store,
            &directory,
            DeferredTestDrain::Detached,
            &RedactionPolicy::from_environment(),
        )?;
    }
    Ok(())
}

pub(crate) fn complete_capture(result: &Envelope<super::result::ManagedSummary>) -> bool {
    result.payload.child_exit.is_some()
        && result.observation.completeness == Completeness::Complete
        && [&result.payload.stdout, &result.payload.stderr]
            .into_iter()
            .all(|stream| {
                stream
                    .as_ref()
                    .is_some_and(|stream| stream.complete && !stream.io_unresolved)
            })
}

fn release_original_images(
    store: &InvocationStore,
    directory: &InvocationDirectory,
    action: Action,
) -> Result<ReleasedWorkflow, HostError> {
    let paired_result_sha256 = digest(&directory.paired_settlement_bytes()?);
    let maximum = Some(crate::developer_evidence::MAX_INPUT_BYTES);
    let original = crate::test_inventory::captured_report_at_limit(
        store,
        directory.id().as_str(),
        None,
        maximum,
    )?;
    let inventory = if action.is_compiler() {
        None
    } else {
        require_fixture_settlement(&original)?;
        original
            .observation
            .inventory
            .as_ref()
            .map(|_| crate::test_inventory::captured_inventory(&original, maximum))
            .transpose()?
            .map(|(inventory, _)| inventory)
    };
    let fixture_settled = match (action, inventory.as_ref()) {
        (Action::Test, Some(inventory)) => {
            #[derive(Deserialize)]
            struct FixtureRequirements {
                fixture_requirements: Vec<crate::fixtures::Requirement>,
            }
            let requirements: FixtureRequirements =
                serde_json::from_slice(&original.report).map_err(host)?;
            let groups = crate::test_workflow::selected_target_owned_groups(
                inventory,
                &requirements.fixture_requirements,
            )
            .map_err(host)?;
            Some(target_fixture_settled(&original.report, &groups)?)
        }
        _ => None,
    };
    let unlisted = if inventory.is_none() && !action.is_compiler() {
        unlisted_builds(&original)?
    } else {
        Vec::new()
    };
    release_images(
        directory.path(),
        action,
        inventory.as_ref(),
        fixture_settled,
        &unlisted,
    )?;
    Ok(ReleasedWorkflow {
        captured: original,
        paired_result_sha256,
    })
}

fn target_fixture_settled(report: &[u8], required: &BTreeSet<usize>) -> Result<bool, HostError> {
    if required.is_empty() {
        return Ok(true);
    }
    #[derive(Deserialize)]
    struct Summary {
        target_fixture_settled: Option<bool>,
    }
    #[derive(Deserialize)]
    struct Observation {
        summary: Summary,
    }
    #[derive(Deserialize)]
    struct Group {
        kind: String,
        group: usize,
        observation: Observation,
    }
    #[derive(Deserialize)]
    struct TargetReport {
        observations: Vec<Group>,
    }
    let original: TargetReport = serde_json::from_slice(report).map_err(host)?;
    let mut settled = BTreeSet::new();
    for group in original.observations {
        if required.contains(&group.group)
            && (group.kind != "test"
                || group.observation.summary.target_fixture_settled != Some(true)
                || !settled.insert(group.group))
        {
            return Ok(false);
        }
    }
    Ok(&settled == required)
}

#[cfg(test)]
#[test]
fn target_fixture_release_depends_on_all_original_owner_observations() {
    let required = BTreeSet::from([1, 3]);
    let report = |groups: &[(usize, Option<bool>)]| {
        serde_json::to_vec(&serde_json::json!({
            "outcome": "tests_failed",
            "observations": groups.iter().map(|(group, settled)| serde_json::json!({
                "kind": "test",
                "group": group,
                "observation": {"summary": {"target_fixture_settled": settled}}
            })).collect::<Vec<_>>()
        }))
        .unwrap()
    };
    assert!(
        target_fixture_settled(&report(&[(1, Some(true)), (3, Some(true))]), &required).unwrap()
    );
    for groups in [
        &[(1, Some(true))][..],
        &[(1, Some(true)), (3, None)][..],
        &[(1, Some(false)), (3, Some(true))][..],
        &[(1, Some(true)), (1, Some(true)), (3, Some(true))][..],
    ] {
        assert!(!target_fixture_settled(&report(groups), &required).unwrap());
    }
}

fn unlisted_builds(
    original: &CapturedReport,
) -> Result<Vec<(usize, crate::discovery::BuildCapture)>, HostError> {
    use crate::test_workflow::execute::{EvidenceRole, EvidenceRoot, Outcome, evidence};

    if original.observation.outcome != Outcome::Failed {
        return Ok(Vec::new());
    }
    let files = original.observation.evidence.as_deref().ok_or_else(|| {
        host(invalid(
            "failed discovery lacks an original compiler capture account",
        ))
    })?;
    let builds: Vec<_> = files
        .iter()
        .filter(|file| file.role == EvidenceRole::CompilerContext)
        .cloned()
        .collect();
    if builds.is_empty() {
        return Ok(Vec::new());
    }
    let captured = evidence::capture(
        &original.parent,
        &builds,
        crate::developer_evidence::MAX_INPUT_BYTES,
    )?;
    if captured.len() != builds.len() {
        return Err(host(invalid(
            "original failed discovery has an unsettled compiler capture",
        )));
    }
    #[derive(Deserialize)]
    struct OriginalSelection {
        selection: crate::selection::Plan,
        admission: Vec<crate::correctness::Resolution>,
    }
    let selected: OriginalSelection = serde_json::from_slice(&original.report).map_err(host)?;
    captured
        .into_iter()
        .map(|capture| {
            let commitment = &builds[capture.dependency];
            let group = commitment
                .group
                .ok_or_else(|| host(invalid("original compiler capture group absent")))?;
            let admitted = selected
                .admission
                .get(group)
                .ok_or_else(|| host(invalid("original compiler capture group was not admitted")))?;
            let plan =
                selected.selection.groups().get(group).ok_or_else(|| {
                    host(invalid("original compiler capture group was not selected"))
                })?;
            let build: crate::discovery::BuildCapture =
                serde_json::from_slice(&capture.original).map_err(host)?;
            if commitment.root != EvidenceRoot::Workflow
                || commitment.path != format!("inventory/group-{group}/build/build.json")
                || build.version != 1
                || !build.incomplete.is_empty()
                || build.resolution.packages != admitted.packages
                || build.resolution.native_capabilities != admitted.native_capabilities
                || !build.resolution.same_units(admitted)
            {
                return Err(host(invalid(
                    "failed discovery compiler capture differs from its original admitted group",
                )));
            }
            plan.validate_resolution(&build.resolution)?;
            Ok((group, build))
        })
        .collect()
}

fn require_fixture_settlement(original: &CapturedReport) -> Result<(), HostError> {
    use crate::test_workflow::execute::{EvidenceRole, evidence};

    if original.observation.action != Action::Test {
        return Ok(());
    }
    let files = original.observation.evidence.as_deref().ok_or_else(|| {
        host(invalid(
            "original test has no fixture settlement evidence account",
        ))
    })?;
    let fixtures: Vec<_> = files
        .iter()
        .filter(|file| file.role == EvidenceRole::FixtureOutcome)
        .cloned()
        .collect();
    let captured = evidence::capture(&original.parent, &fixtures, 1024 * 1024)?;
    if captured.len() != fixtures.len() {
        return Err(host(invalid(
            "original borrowed fixture settlement is unobserved",
        )));
    }
    for captured in captured {
        let outcome: crate::fixtures::CanonicalRunReport =
            serde_json::from_slice(&captured.original).map_err(host)?;
        if outcome.version != 1 || outcome.runner_exit.is_none() || !outcome.fixture_settled {
            return Err(host(invalid("original borrowed fixture did not settle")));
        }
    }
    Ok(())
}

const RELEASE_DIRECTORY_FLAGS: rustix::fs::OFlags = rustix::fs::OFlags::RDONLY
    .union(rustix::fs::OFlags::DIRECTORY)
    .union(rustix::fs::OFlags::NOFOLLOW)
    .union(rustix::fs::OFlags::CLOEXEC);

fn release_private_child(parent: &File, name: &str) -> Result<Option<File>, HostError> {
    let file: File = match rustix::fs::openat(
        parent,
        name,
        RELEASE_DIRECTORY_FLAGS,
        rustix::fs::Mode::empty(),
    ) {
        Ok(fd) => fd.into(),
        Err(rustix::io::Errno::NOENT) => return Ok(None),
        Err(error) => return Err(io::Error::from(error).into()),
    };
    let metadata = file.metadata()?;
    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o777 != 0o700 {
        return Err(host(invalid(
            "workflow control directory lacks private custody",
        )));
    }
    Ok(Some(file))
}

fn lock_release_directory(invocation: &Path) -> Result<File, HostError> {
    let invocation_root: File = rustix::fs::open(
        invocation,
        RELEASE_DIRECTORY_FLAGS,
        rustix::fs::Mode::empty(),
    )
    .map_err(io::Error::from)?
    .into();
    // All release routes serialize against the same stable directory inode. No
    // additional lock file survives an otherwise complete invocation release.
    let metadata = invocation_root.metadata()?;
    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o777 != 0o700 {
        return Err(host(invalid(
            "original invocation directory lacks private custody",
        )));
    }
    rustix::fs::flock(
        &invocation_root,
        rustix::fs::FlockOperation::NonBlockingLockExclusive,
    )
    .map_err(io::Error::from)?;
    Ok(invocation_root)
}

/// Only a physically settled list with no published inventory has no image consumers.
pub(super) fn release_interrupted_listing(
    directory: &InvocationDirectory,
    result: &Envelope<super::result::ManagedSummary>,
) -> Result<bool, HostError> {
    if Action::from_command(&result.command) != Some(Action::List)
        || !matches!(result.outcome, CommandOutcome::Interrupted)
        || result.settlement != Settlement::Complete
        || result.payload.native_lifetime_pending != Some(false)
        || result.payload.workflow.is_some()
        || &result.invocation_id != directory.id()
    {
        return Ok(false);
    }
    directory.paired_settlement_bytes()?;
    discard_unpublished_inventory(directory.path())?;
    Ok(true)
}

fn discard_unpublished_inventory(invocation: &Path) -> Result<(), HostError> {
    let root = lock_release_directory(invocation)?;
    let Some(workflow) = release_private_child(&root, "workflow")? else {
        return Ok(());
    };
    let Some(inventory) = release_private_child(&workflow, "inventory")? else {
        crate::discovery::capture::collect_unused(&invocation.join("workflow/inventory"))?;
        return Ok(());
    };
    for (owner, name) in [(&workflow, "report.json"), (&inventory, "inventory.json")] {
        match rustix::fs::statat(owner, name, rustix::fs::AtFlags::SYMLINK_NOFOLLOW) {
            Err(rustix::io::Errno::NOENT) => {}
            Err(error) => return Err(io::Error::from(error).into()),
            Ok(_) => {
                return Err(host(invalid(
                    "interrupted listing already has published evidence",
                )));
            }
        }
    }
    let path = invocation.join("workflow/inventory");
    let selected = fs::symlink_metadata(&path)?;
    let opened = inventory.metadata()?;
    if !selected.is_dir() || (selected.dev(), selected.ino()) != (opened.dev(), opened.ino()) {
        return Err(host(invalid("interrupted inventory directory changed")));
    }
    fs::remove_dir_all(path)?;
    workflow.sync_all()?;
    crate::discovery::capture::collect_unused(&invocation.join("workflow/inventory"))?;
    Ok(())
}

pub(crate) fn release_images(
    invocation: &Path,
    action: Action,
    inventory: Option<&crate::discovery::Inventory>,
    fixture_settled: Option<bool>,
    unlisted: &[(usize, crate::discovery::BuildCapture)],
) -> Result<(), HostError> {
    if fixture_settled == Some(false) {
        return Err(host(invalid(
            "target-owned fixture capture release awaits verified settlement",
        )));
    }
    let open = release_private_child;
    let (location, controls): (_, &[_]) = if action.is_compiler() {
        ("check-control", &["cargo-pse-correctness"])
    } else {
        ("inventory", &["cargo-nextest", "cargo-pse-correctness"])
    };
    let invocation_root = lock_release_directory(invocation)?;
    let Some(workflow) = open(&invocation_root, "workflow")? else {
        return Ok(());
    };
    let Some(directory) = open(&workflow, location)? else {
        return Ok(());
    };
    let path = invocation.join("workflow").join(location);
    for control in controls {
        let expected = inventory.map(|inventory| match *control {
            "cargo-nextest" => &inventory.nextest,
            _ => &inventory.correctness,
        });
        crate::discovery::capture::release_executable(&directory, &path.join(control), expected)?;
    }
    let release_group =
        |group: usize, captured: &crate::discovery::BuildCapture| -> Result<(), HostError> {
            let group_name = format!("group-{group}");
            let group_root = open(&directory, &group_name)?
                .ok_or_else(|| host(invalid("original compiler group directory is absent")))?;
            let build = open(&group_root, "build")?
                .ok_or_else(|| host(invalid("original compiler capture directory is absent")))?;
            let build_path = path.join(&group_name).join("build");
            release_artifacts(&build, &build_path, &captured.artifacts, &open)
        };
    for (group, capture) in unlisted {
        release_group(*group, capture)?;
    }
    if let Some(inventory) = inventory {
        for (group, listed) in inventory.groups.iter().enumerate() {
            release_group(group, &listed.build)?;
        }
        if fixture_settled.is_some()
            && let Some(fixtures) = open(&workflow, "fixtures")?
        {
            let fixtures_path = invocation.join("workflow/fixtures");
            let release = |name: &str, targets: &[&str]| -> Result<(), HostError> {
                let Some(build) = open(&fixtures, name)? else {
                    return Ok(());
                };
                let path = fixtures_path.join(name);
                let receipt = crate::discovery::BuildCapture::read(&path.join("build.json"))?;
                let opened = build.metadata()?;
                let selected = path.symlink_metadata()?;
                if (opened.dev(), opened.ino()) != (selected.dev(), selected.ino())
                    || receipt.version != 1
                    || receipt.cargo_exit != 0
                    || !receipt.incomplete.is_empty()
                    || receipt.artifacts.is_empty()
                    || receipt.artifacts.iter().any(|artifact| {
                        artifact.cargo.profile.test
                            || !targets.contains(&artifact.cargo.target.name.as_str())
                    })
                {
                    return Err(host(invalid(
                        "fixture helper capture does not match its original build position",
                    )));
                }
                release_artifacts(&build, &path, &receipt.artifacts, &open)
            };
            release(
                "canonical-tools",
                &["pse-canonical-service", "pse-canonical-test"],
            )?;
            release("observation-helper", &["pse-observation-fixture"])?;
            for group in 0..inventory.groups.len() {
                release(&format!("worker-{group}"), &["pse-worker"])?;
                release(
                    &format!("scientific-worker-{group}"),
                    &["pse-scientific-worker"],
                )?;
            }
        }
    }
    Ok(())
}

fn release_artifacts(
    directory: &File,
    path: &Path,
    artifacts: &[crate::discovery::capture::BuiltArtifact],
    open: &impl Fn(&File, &str) -> Result<Option<File>, HostError>,
) -> Result<(), HostError> {
    for (ordinal, artifact) in artifacts.iter().enumerate() {
        let executable = &artifact.executable;
        let expected =
            crate::discovery::capture::artifact_path(path, ordinal, &executable.original)?;
        if executable.captured != expected {
            return Err(host(invalid(
                "captured test image differs from its compiler position",
            )));
        }
        let member_root = open(directory, &format!("artifact-{ordinal}"))?
            .ok_or_else(|| host(invalid("original executable directory is absent")))?;
        crate::discovery::capture::release_executable(&member_root, &expected, Some(executable))?;
    }
    Ok(())
}

/// Metadata points at one authoritative encoded private input; it does not repeat selection.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InputBinding {
    /// Original exact encoded input extent.
    bytes: u64,
    /// Original exact encoded input identity.
    sha256: String,
}
#[derive(Deserialize)]
struct Metadata {
    workflow: Option<InputBinding>,
}
/// Serialize the input once; RawValue publication preserves exactly these bytes.
pub(crate) fn encode_input(input: &Input) -> Result<(InputBinding, Box<RawValue>), HostError> {
    let value = serde_json::value::to_raw_value(input).map_err(host)?;
    if value.get().len() > 1024 * 1024 {
        return Err(host(invalid(
            "private input exceeds its 1 MiB protocol extent",
        )));
    }
    Ok((
        InputBinding {
            bytes: value.get().len() as u64,
            sha256: digest(value.get().as_bytes()),
        },
        value,
    ))
}
/// Only this fixed prepared executable/argument vector can produce a workflow verdict.
pub(crate) fn command(
    root: &Path,
    id: &CliInvocationId,
    control: &PreparedGeneration,
    environment: crate::environment::Environment,
) -> Result<PayloadCommand, HostError> {
    Ok(PayloadCommand {
        program: control.executable(root, HostTool::Host)?,
        arguments: arguments(root, id),
        cwd: root.to_owned(),
        environment,
    })
}
fn arguments(root: &Path, id: &CliInvocationId) -> Vec<OsString> {
    vec![
        "__workflow".into(),
        "--checkout-bytes".into(),
        super::request::encode_path(root).into(),
        "--invocation".into(),
        id.as_str().into(),
    ]
}
fn input(
    directory: &InvocationDirectory,
    binding: &InputBinding,
    id: &CliInvocationId,
    command: &str,
) -> Result<Input, HostError> {
    bound_input(directory, binding, id, command).map(|(input, _)| input)
}
fn bound_input(
    directory: &InvocationDirectory,
    binding: &InputBinding,
    id: &CliInvocationId,
    command: &str,
) -> Result<(Input, Vec<u8>), HostError> {
    let bytes = directory
        .evidence_bytes(InvocationEvidence::WorkflowInput)?
        .ok_or_else(|| host(invalid("original workflow input is absent")))?;
    let value: Box<RawValue> = serde_json::from_slice(&bytes).map_err(host)?;
    if binding.bytes != value.get().len() as u64 || binding.sha256 != digest(value.get().as_bytes())
    {
        return Err(host(invalid(
            "original workflow input byte identity differs from metadata",
        )));
    }
    let input: Input = serde_json::from_str(value.get()).map_err(host)?;
    input.validate_original(id, command).map_err(host)?;
    Ok((input, bytes))
}
/// Read original retained input through its original metadata commitment. This is
/// observation/selection data, not a fresh admission or a process-control capability.
pub(crate) fn retained_input(
    store: &InvocationStore,
    id: &CliInvocationId,
    action: Action,
    report: &ReportObservation,
) -> Result<Input, HostError> {
    let metadata = store.metadata::<Metadata>(id)?;
    if metadata.selection.command != action.command() {
        return Err(host(invalid(
            "retained workflow command differs from requested original action",
        )));
    }
    let binding = metadata
        .selection
        .operation
        .workflow
        .ok_or_else(|| host(invalid("original workflow input binding is absent")))?;
    let (input, bytes) = bound_input(
        &store.directory(id)?,
        &binding,
        id,
        &metadata.selection.command,
    )?;
    report.validate_input_bytes(&bytes).map_err(host)?;
    Ok(input)
}
/// Internal entry: authenticate the actual original payload leader before any compilation.
pub(crate) async fn run(root: PathBuf, id: CliInvocationId) -> Result<i32, HostError> {
    let environment = std::env::vars_os().collect();
    let store = InvocationStore::new(crate::paths::CheckoutState::resolve(&root, &environment)?);
    let directory = Arc::new(store.directory(&id)?);
    let metadata = if directory
        .evidence::<serde_json::Value>(InvocationEvidence::Launch)?
        .is_none()
    {
        let custody: crate::foreground::ForegroundCustody = directory
            .evidence(InvocationEvidence::Custody)?
            .ok_or_else(|| host(invalid("foreground native custody is absent")))?;
        let manager = HostManager::connect_user(None).await?;
        let membership = manager.current_membership().await?;
        let actual = match pse_diagnostics::process_facts::ProcessReader::default()
            .process(std::process::id())
        {
            pse_diagnostics::process_facts::ProcessRead::Present(facts) => facts.identity,
            _ => {
                return Err(host(invalid(
                    "foreground helper process identity unavailable",
                )));
            }
        };
        if custody.version != 4
            || custody.child != actual
            || custody.manager != manager.identity_key()
            || custody.unit != membership.root().unit
            || !membership.matches_root(&custody.root)
            || !manager.native_workload(membership.root()).await?
        {
            return Err(host(invalid(
                "workflow is not the original native foreground child",
            )));
        }
        let metadata = store.metadata::<Metadata>(&id)?;
        if std::env::current_exe()?
            != metadata
                .selection
                .control
                .executable(&root, HostTool::Host)?
        {
            return Err(host(invalid(
                "foreground helper executable differs from captured prepared image",
            )));
        }
        metadata
    } else {
        let leader: supervisor::PayloadLeader = supervisor::current_payload(&environment).await?;
        if leader.root() != root.canonicalize()? || leader.invocation_id() != &id {
            return Err(host(invalid(
                "workflow entry differs from authenticated native payload",
            )));
        }
        leader.metadata::<Metadata>()?
    };
    let binding = metadata
        .selection
        .operation
        .workflow
        .ok_or_else(|| host(invalid("original workflow input binding is absent")))?;
    let (input, accepted_bytes) =
        bound_input(&directory, &binding, &id, &metadata.selection.command)?;
    let accepted_content = crate::test_workflow::execute::ContentIdentity::of(&accepted_bytes);
    let destination = directory.path().join("workflow");
    let mut report =
        crate::test_workflow::execute::run_managed(&input, &root, &destination, environment)
            .await
            .map_err(host)?;
    // Keep the exact full-file identity accepted before execution, including the
    // native publisher's newline. A later read cannot redefine original input.
    report.input_cutoff = Some(Ok(accepted_content));
    let terminal = report.publish(&destination).map_err(host)?;
    terminal.validate(&id, input.action).map_err(host)?;
    let status = terminal.helper_exit();
    terminal.write(&mut io::stdout().lock()).map_err(host)?;
    Ok(status)
}

pub(crate) enum Observer {
    Raw,
    Workflow(WorkflowObserver),
}
pub(crate) struct WorkflowObserver {
    invocation: CliInvocationId,
    action: Action,
    framer: RecordFramer,
    next_offset: u64,
    records: u64,
    terminal: Option<TerminalRecord>,
    invalid: bool,
}
impl Observer {
    pub(super) fn for_payload(
        context: &supervisor::Context,
        root: &Path,
        command: &PayloadCommand,
    ) -> Result<Self, HostError> {
        Self::for_operation(
            &context.store,
            &context.directory,
            root,
            &command.program,
            &command.arguments,
        )
    }
    pub(crate) fn for_operation(
        store: &InvocationStore,
        directory: &Arc<InvocationDirectory>,
        root: &Path,
        program: &Path,
        arguments: &[OsString],
    ) -> Result<Self, HostError> {
        let metadata = store.metadata::<Box<RawValue>>(directory.id())?;
        let Some(action) = Action::from_command(&metadata.selection.command) else {
            return Ok(Self::Raw);
        };
        if program
            != metadata
                .selection
                .control
                .executable(root, HostTool::Host)?
                .as_path()
            || arguments != self::arguments(root, directory.id())
        {
            return Err(host(invalid(
                "workflow payload is not the original fixed helper",
            )));
        }
        let operation: Metadata =
            serde_json::from_str(metadata.selection.operation.get()).map_err(host)?;
        let binding = operation
            .workflow
            .ok_or_else(|| host(invalid("workflow operation has no original input binding")))?;
        let input = input(
            directory,
            &binding,
            directory.id(),
            &metadata.selection.command,
        )?;
        Ok(Self::Workflow(WorkflowObserver {
            invocation: input.invocation,
            action,
            framer: RecordFramer::new(
                NonZeroUsize::new(input.bounds.record_bytes)
                    .ok_or_else(|| host(invalid("zero terminal frame extent")))?,
            ),
            next_offset: 0,
            records: 0,
            terminal: None,
            invalid: false,
        }))
    }
    pub(crate) fn finish(self) -> Result<Option<ObservedTerminal>, HostError> {
        let Self::Workflow(mut observer) = self else {
            return Ok(None);
        };
        observer.framer.finish(|_| observer.invalid = true);
        if observer.invalid || observer.records != 1 {
            return Err(host(invalid(
                "required workflow terminal frame is missing, malformed, duplicated or truncated",
            )));
        }
        let terminal = observer
            .terminal
            .ok_or_else(|| host(invalid("required workflow terminal is absent")))?;
        terminal
            .validate(&observer.invocation, observer.action)
            .map_err(host)?;
        Ok(Some(ObservedTerminal { terminal }))
    }
}
impl super::streams::ChunkObserver for Observer {
    type Error = HostError;
    fn observe(
        &mut self,
        stream: crate::extraction::OutputStream,
        offset: u64,
        bytes: &[u8],
    ) -> Result<(), HostError> {
        let Self::Workflow(observer) = self else {
            return Ok(());
        };
        if stream == crate::extraction::OutputStream::Stderr {
            return Ok(());
        }
        if offset != observer.next_offset {
            observer.invalid = true;
            return Err(host(invalid(
                "workflow terminal byte delivery is discontinuous",
            )));
        }
        observer.next_offset = offset.saturating_add(bytes.len() as u64);
        let mut first = None;
        observer.framer.push(bytes, |frame| {
            let result = match frame {
                Frame::Complete(bytes) => {
                    observer.records = observer.records.saturating_add(1);
                    if observer.records != 1 {
                        Err(host(invalid(
                            "workflow emitted more than one required terminal record",
                        )))
                    } else {
                        match serde_json::from_slice::<TerminalRecord>(bytes) {
                            Ok(terminal) => {
                                match terminal.validate(&observer.invocation, observer.action) {
                                    Ok(()) => {
                                        observer.terminal = Some(terminal);
                                        Ok(())
                                    }
                                    Err(error) => Err(host(error)),
                                }
                            }
                            Err(error) => Err(host(error)),
                        }
                    }
                }
                Frame::Oversized { .. } | Frame::Truncated(_) => Err(host(invalid(
                    "workflow terminal record exceeds framing policy or is incomplete",
                ))),
            };
            if let Err(error) = result {
                observer.invalid = true;
                if first.is_none() {
                    first = Some(error);
                }
            }
        });
        first.map_or(Ok(()), Err)
    }
}

/// Constructed only after original helper transport and durable report bytes are checked.
pub(crate) struct Validated {
    terminal: TerminalRecord,
    path: PathBuf,
    failures: Vec<crate::envelope::ErrorRecord>,
    pub(super) cancelled_before_build: bool,
}
impl Validated {
    pub(super) fn into_parts(self) -> (TerminalRecord, PathBuf, Vec<crate::envelope::ErrorRecord>) {
        (self.terminal, self.path, self.failures)
    }
}
/// Only the fixed-helper observer can construct this; decoding transport data grants no proof.
pub(crate) struct ObservedTerminal {
    terminal: TerminalRecord,
}
pub(crate) async fn finish_capture(
    directory: Arc<InvocationDirectory>,
    capture: &mut Result<
        super::streams::TeeReport<tokio::fs::File, tokio::fs::File, Observer>,
        HostError,
    >,
    child_exit: Option<i32>,
    required: bool,
) -> Result<Option<Validated>, HostError> {
    match capture {
        Ok(capture) => match capture
            .observer
            .take()
            .ok_or_else(|| host(invalid("original workflow observer unavailable")))?
            .finish()?
        {
            Some(terminal) => validate_report(directory, terminal, child_exit)
                .await
                .map(Some),
            None => Ok(None),
        },
        Err(_) if required => Err(host(invalid("original foreground capture unavailable"))),
        Err(_) => Ok(None),
    }
}
pub(crate) async fn validate_report(
    directory: Arc<InvocationDirectory>,
    observed: ObservedTerminal,
    child_exit: Option<i32>,
) -> Result<Validated, HostError> {
    let terminal = observed.terminal;
    if child_exit != Some(terminal.helper_exit()) {
        return Err(host(invalid(
            "workflow terminal and original waited helper status disagree",
        )));
    }
    tokio::task::spawn_blocking(move || {
        let parent = directory.path().join("workflow");
        let metadata = fs::symlink_metadata(&parent)?;
        if !metadata.is_dir()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o777 != 0o700
        {
            return Err(host(invalid(
                "workflow report directory is not original private evidence",
            )));
        }
        let path = parent.join(&terminal.report.path);
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
            .open(&path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o022 != 0
            || metadata.len() != terminal.report.byte_length
        {
            return Err(host(invalid(
                "workflow report file identity/extent differs from original terminal",
            )));
        }
        let mut input = HashingReader {
            input: file,
            sha: Sha256::new(),
            bytes: 0,
        };
        let facts = {
            let mut json = serde_json::Deserializer::from_reader(io::BufReader::new(&mut input));
            let facts = ReportObservation::deserialize(&mut json).map_err(host)?;
            json.end().map_err(host)?;
            facts
        };
        let sha = input
            .sha
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        terminal
            .report
            .validate_content(input.bytes, &sha)
            .map_err(host)?;
        facts.validate_against(&terminal).map_err(host)?;
        facts.require_current().map_err(host)?;
        if facts.inventory.is_some() {
            let bytes = crate::test_inventory::read_original_inventory(&parent)?;
            facts.validate_inventory(&bytes).map_err(host)?;
        }
        Ok(Validated {
            cancelled_before_build: facts.outcome == crate::test_workflow::execute::Outcome::Failed
                && facts.inventory.is_none()
                && facts.failures.iter().any(|failure| {
                    failure.code.as_deref()
                        == Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled.as_str())
                }),
            terminal,
            path,
            failures: facts.failures,
        })
    })
    .await?
}
struct HashingReader<R> {
    input: R,
    sha: Sha256,
    bytes: u64,
}
impl<R: Read> Read for HashingReader<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let count = self.input.read(bytes)?;
        self.sha.update(&bytes[..count]);
        self.bytes = self.bytes.saturating_add(count as u64);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancelled_listing_releases_unpublished_images_and_keeps_committed_inventory() {
        use std::os::unix::fs::{DirBuilderExt as _, PermissionsExt as _, symlink};

        let scratch = tempfile::tempdir().unwrap();
        fs::set_permissions(scratch.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let workflow = scratch.path().join("workflow");
        let inventory = workflow.join("inventory");
        let images = inventory.join("group-0/build/artifact-0");
        fs::DirBuilder::new()
            .mode(0o700)
            .recursive(true)
            .create(&images)
            .unwrap();
        let source = scratch.path().join("outside-inventory");
        fs::write(&source, b"unrelated original").unwrap();
        symlink(&source, images.join("external-link")).unwrap();
        let captured = images.join("captured-image");
        fs::write(&captured, b"private scratch").unwrap();

        for receipt in [
            workflow.join("report.json"),
            inventory.join("inventory.json"),
        ] {
            fs::write(&receipt, b"original publication").unwrap();
            assert!(discard_unpublished_inventory(scratch.path()).is_err());
            assert!(captured.exists());
            fs::remove_file(receipt).unwrap();
        }

        discard_unpublished_inventory(scratch.path()).unwrap();
        discard_unpublished_inventory(scratch.path()).unwrap();
        assert!(!inventory.exists());
        assert_eq!(fs::read(source).unwrap(), b"unrelated original");
    }

    #[test]
    fn workflow_controls_release_refuses_changed_capture_and_can_resume() {
        use std::os::unix::fs::{DirBuilderExt as _, PermissionsExt as _};

        let scratch = tempfile::tempdir().unwrap();
        fs::set_permissions(scratch.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let original = scratch.path().join("original");
        fs::write(&original, b"original").unwrap();
        fs::set_permissions(&original, fs::Permissions::from_mode(0o700)).unwrap();
        let inventory = scratch.path().join("workflow/inventory");
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&inventory)
            .unwrap();
        let nextest = inventory.join("cargo-nextest");
        let association = inventory.join("cargo-nextest.capture.json");
        let correctness = inventory.join("cargo-pse-correctness");
        let report = inventory.join("inventory.json");
        fs::write(&report, b"retained inventory").unwrap();
        crate::discovery::capture::executable(&original, &nextest).unwrap();
        crate::discovery::capture::executable(&original, &correctness).unwrap();
        fs::set_permissions(&nextest, fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(&nextest, b"mutation").unwrap();
        fs::set_permissions(&nextest, fs::Permissions::from_mode(0o500)).unwrap();

        let competing = File::open(scratch.path()).unwrap();
        rustix::fs::flock(&competing, rustix::fs::FlockOperation::LockExclusive).unwrap();
        assert!(
            matches!(release_images(scratch.path(), Action::Test, None, None, &[]),
            Err(HostError::Io(error)) if error.kind() == io::ErrorKind::WouldBlock)
        );
        drop(competing);
        assert!(release_images(scratch.path(), Action::Test, None, None, &[]).is_err());
        assert!(nextest.is_file() && association.is_file() && correctness.is_file());
        fs::remove_file(&nextest).unwrap(); // Interruption between image and sidecar unlink.
        release_images(scratch.path(), Action::Test, None, None, &[]).unwrap();
        release_images(scratch.path(), Action::Test, None, None, &[]).unwrap();
        assert!(!association.exists() && !correctness.exists());
        assert_eq!(fs::read(report).unwrap(), b"retained inventory");
    }
    use crate::{
        managed::streams::ChunkObserver as _,
        test_workflow::execute::{Outcome, ReportReference},
    };
    use std::os::unix::fs::{DirBuilderExt as _, PermissionsExt as _};
    fn observer(id: &CliInvocationId) -> Observer {
        Observer::Workflow(WorkflowObserver {
            invocation: id.clone(),
            action: Action::Test,
            framer: RecordFramer::new(NonZeroUsize::new(4096).unwrap()),
            next_offset: 0,
            records: 0,
            terminal: None,
            invalid: false,
        })
    }
    #[test]
    fn retained_input_requires_original_binding_before_rerun_selection() {
        use crate::{
            invocation::{InvocationSpec, InvocationStore},
            managed::payload::PayloadPolicy,
            test_workflow::execute::Bounds,
        };
        let scratch = tempfile::tempdir().unwrap();
        let environment = crate::environment::Environment::from([(
            "XDG_STATE_HOME".into(),
            scratch.path().join("state").into_os_string(),
        )]);
        let state = crate::paths::CheckoutState::from_paths(
            &scratch.path().join("checkout"),
            &scratch.path().join("common.git"),
            &environment,
        )
        .unwrap();
        let store = InvocationStore::new(state);
        let id = CliInvocationId::new();
        let original = Input {
            version: 1,
            invocation: id.clone(),
            action: Action::Test,
            request: crate::selection::Request {
                operation: crate::selection::Operation::Test,
                scope: crate::selection::Scope::DefaultMembers {
                    features: crate::selection::Features::default(),
                },
                platform: None,
                filters: crate::selection::Filters::default(),
                non_incremental: false,
                ignored: crate::selection::IgnoredPolicy::Default,
            },
            profile: "alpha".into(),
            timings: false,
            policy: PayloadPolicy {
                startup: std::time::Duration::from_secs(1),
                write: std::time::Duration::from_secs(1),
                drain: std::time::Duration::from_secs(1),
            },
            bounds: Bounds {
                record_bytes: 1024,
                grouping_capacity: 4,
                failure_records: 4,
            },
        };
        let (binding, encoded) = encode_input(&original).unwrap();
        let directory = store
            .begin(InvocationSpec {
                invocation_id: id.clone(),
                command: "test".into(),
                session: None,
                parent: None,
                operation: serde_json::json!({"workflow":binding}),
                control: PreparedGeneration {
                    generation: "unit.original".into(),
                    protocol: 1,
                    manifest_digest: "a".repeat(64),
                },
                environment_digest: "b".repeat(64),
                scientific: vec![],
            })
            .unwrap();
        directory
            .publish_evidence(InvocationEvidence::WorkflowInput, &encoded)
            .unwrap();
        let report: ReportObservation = serde_json::from_value(serde_json::json!({"version":2,"invocation":id,"action":"test","outcome":"failed","required_capture_complete":false,"failures":[]})).unwrap();
        assert_eq!(
            retained_input(&store, &id, Action::Test, &report)
                .unwrap()
                .profile,
            "alpha"
        );
        assert!(retained_input(&store, &id, Action::Check, &report).is_err());
        let path = directory.path().join("workflow-input.json");
        let before = fs::read_to_string(&path).unwrap();
        let (_, accepted_bytes) = bound_input(&directory, &binding, &id, "test").unwrap();
        assert_eq!(accepted_bytes, before.as_bytes());
        assert_eq!(
            accepted_bytes.last(),
            Some(&b'\n'),
            "native publication includes its original newline"
        );
        let accepted_content = crate::test_workflow::execute::ContentIdentity::of(&accepted_bytes);
        let mut current = report.clone();
        current.version = 3;
        current.evidence = Some(vec![crate::test_workflow::execute::EvidenceFile {
            root: crate::test_workflow::execute::EvidenceRoot::Invocation,
            path: "workflow-input.json".into(),
            role: crate::test_workflow::execute::EvidenceRole::Input,
            group: None,
            content: Some(accepted_content.clone()),
            omission: None,
            failure: None,
        }]);
        assert!(retained_input(&store, &id, Action::Test, &current).is_ok());
        let whitespace_replacement = format!("{before} \n\t");
        fs::write(&path, &whitespace_replacement).unwrap();
        let (_, rebound_bytes) = bound_input(&directory, &binding, &id, "test").unwrap();
        assert_ne!(
            crate::test_workflow::execute::ContentIdentity::of(&rebound_bytes),
            accepted_content,
            "metadata authenticates JSON value bytes, so it cannot redefine the accepted full-file identity"
        );
        assert!(
            retained_input(&store, &id, Action::Test, &current).is_err(),
            "appended whitespace must refuse against the original full-file commitment"
        );
        fs::write(&path, &before).unwrap();
        assert!(retained_input(&store, &id, Action::Test, &current).is_ok());
        assert_eq!(before.matches("alpha").count(), 1);
        let after = before.replace("alpha", "omega");
        assert_eq!(before.len(), after.len());
        fs::write(&path, after).unwrap();
        let merely_parsed: Input = directory
            .evidence(InvocationEvidence::WorkflowInput)
            .unwrap()
            .unwrap();
        assert_eq!(
            merely_parsed.profile, "omega",
            "syntactic correlation alone is weaker than the original commitment"
        );
        assert!(
            retained_input(&store, &id, Action::Test, &report)
                .unwrap_err()
                .to_string()
                .contains("byte identity differs")
        );
    }
    #[test]
    fn generic_observer_accepts_verb_owned_metadata_without_workflow_decoding() {
        let scratch = tempfile::tempdir().unwrap();
        let environment = crate::environment::Environment::from([(
            "XDG_STATE_HOME".into(),
            scratch.path().join("state").into_os_string(),
        )]);
        let state = crate::paths::CheckoutState::from_paths(
            &scratch.path().join("checkout"),
            &scratch.path().join("common.git"),
            &environment,
        )
        .unwrap();
        let store = InvocationStore::new(state);
        for operation in [
            serde_json::json!("generic operation"),
            serde_json::json!([1, 2, 3]),
        ] {
            let directory = Arc::new(
                store
                    .begin(crate::invocation::InvocationSpec {
                        invocation_id: CliInvocationId::new(),
                        command: "run".into(),
                        session: None,
                        parent: None,
                        operation,
                        control: PreparedGeneration {
                            generation: "generation.fixture".into(),
                            protocol: 1,
                            manifest_digest: "a".repeat(64),
                        },
                        environment_digest: "b".repeat(64),
                        scientific: Vec::new(),
                    })
                    .unwrap(),
            );
            assert!(matches!(
                Observer::for_operation(
                    &store,
                    &directory,
                    scratch.path(),
                    Path::new("/bin/true"),
                    &[]
                )
                .unwrap(),
                Observer::Raw
            ));
        }
    }
    fn terminal(id: &CliInvocationId, bytes: &[u8]) -> TerminalRecord {
        TerminalRecord {
            protocol: "pse.test-workflow".into(),
            version: 1,
            invocation: id.clone(),
            action: Action::Test,
            outcome: Outcome::TestsSucceeded,
            report: ReportReference {
                path: "report.json".into(),
                sha256: digest(bytes),
                byte_length: bytes.len() as u64,
            },
            required_capture_complete: true,
        }
    }
    fn observed(id: &CliInvocationId, record: &TerminalRecord) -> ObservedTerminal {
        let mut observer = observer(id);
        let mut bytes = Vec::new();
        record.write(&mut bytes).unwrap();
        observer
            .observe(crate::extraction::OutputStream::Stdout, 0, &bytes)
            .unwrap();
        observer.finish().unwrap().unwrap()
    }
    #[test]
    fn required_terminal_is_single_complete_correlated_and_never_generic_authority() {
        let id = CliInvocationId::new();
        let record = terminal(&id, b"{}");
        let mut bytes = Vec::new();
        record.write(&mut bytes).unwrap();
        let mut valid = observer(&id);
        let split = bytes.len() / 2;
        valid
            .observe(crate::extraction::OutputStream::Stdout, 0, &bytes[..split])
            .unwrap();
        valid
            .observe(
                crate::extraction::OutputStream::Stdout,
                split as u64,
                &bytes[split..],
            )
            .unwrap();
        assert!(valid.finish().unwrap().is_some());
        let mut duplicate = observer(&id);
        duplicate
            .observe(crate::extraction::OutputStream::Stdout, 0, &bytes)
            .unwrap();
        assert!(
            duplicate
                .observe(
                    crate::extraction::OutputStream::Stdout,
                    bytes.len() as u64,
                    &bytes
                )
                .is_err()
        );
        assert!(duplicate.finish().is_err());
        let mut truncated = observer(&id);
        truncated
            .observe(
                crate::extraction::OutputStream::Stdout,
                0,
                &bytes[..bytes.len() - 1],
            )
            .unwrap();
        assert!(truncated.finish().is_err());
        let mut gap = observer(&id);
        assert!(
            gap.observe(crate::extraction::OutputStream::Stdout, 1, &bytes)
                .is_err()
        );
        assert!(gap.finish().is_err());
        let mut foreign = observer(&CliInvocationId::new());
        assert!(
            foreign
                .observe(crate::extraction::OutputStream::Stdout, 0, &bytes)
                .is_err()
        );
        assert!(foreign.finish().is_err());
        let mut raw = Observer::Raw;
        raw.observe(crate::extraction::OutputStream::Stdout, 0, &bytes)
            .unwrap();
        assert!(raw.finish().unwrap().is_none());
        let mut traversal = record;
        traversal.report.path = "../outside.json".into();
        let mut bytes = Vec::new();
        traversal.write(&mut bytes).unwrap();
        let mut reader = observer(&id);
        assert!(
            reader
                .observe(crate::extraction::OutputStream::Stdout, 0, &bytes)
                .is_err()
        );
    }
    #[tokio::test]
    async fn original_report_requires_private_complete_unchanged_bytes_and_waited_status() {
        use crate::{
            invocation::{InvocationSpec, InvocationStore},
            paths::CheckoutState,
            selection::{Features, Filters, IgnoredPolicy, Operation, Request, Scope},
            test_workflow::execute::{
                Bounds, ContentIdentity, EvidenceFile, EvidenceRole, EvidenceRoot,
            },
        };
        let scratch = tempfile::tempdir().unwrap();
        let env = crate::environment::Environment::from([(
            "XDG_STATE_HOME".into(),
            scratch.path().join("state").into_os_string(),
        )]);
        let state = CheckoutState::from_paths(
            &scratch.path().join("checkout"),
            &scratch.path().join("common.git"),
            &env,
        )
        .unwrap();
        let id = CliInvocationId::new();
        let input = Input {
            version: 1,
            invocation: id.clone(),
            action: Action::Test,
            request: Request {
                operation: Operation::Test,
                scope: Scope::DefaultMembers {
                    features: Features::default(),
                },
                platform: None,
                filters: Filters::default(),
                non_incremental: false,
                ignored: IgnoredPolicy::Default,
            },
            profile: "local".into(),
            timings: false,
            policy: super::super::payload::PayloadPolicy {
                startup: std::time::Duration::from_secs(30),
                write: std::time::Duration::from_secs(30),
                drain: std::time::Duration::from_secs(30),
            },
            bounds: Bounds {
                record_bytes: 4096,
                grouping_capacity: 16,
                failure_records: 10,
            },
        };
        let (binding, encoded) = encode_input(&input).unwrap();
        let store = InvocationStore::new(state);
        let directory = Arc::new(
            store
                .begin(InvocationSpec {
                    invocation_id: id.clone(),
                    command: "test".into(),
                    session: None,
                    parent: None,
                    operation: serde_json::json!({"workflow": binding.clone()}),
                    control: PreparedGeneration {
                        generation: "generation.fixture".into(),
                        protocol: 1,
                        manifest_digest: "a".repeat(64),
                    },
                    environment_digest: "b".repeat(64),
                    scientific: vec![],
                })
                .unwrap(),
        );
        directory
            .publish_evidence(InvocationEvidence::WorkflowInput, &encoded)
            .unwrap();
        let (_, original_input) = bound_input(&directory, &binding, &id, "test").unwrap();
        let parent = directory.path().join("workflow");
        fs::DirBuilder::new().mode(0o700).create(&parent).unwrap();
        let path = parent.join("report.json");
        // Author passive report fixtures for byte-custody checks, not native test
        // verdicts. The package-workflow controls acquire actual selected passes.
        let report_bytes = |outcome, failures, inventory: Option<ContentIdentity>| {
            let mut evidence = vec![EvidenceFile {
                root: EvidenceRoot::Invocation,
                path: "workflow-input.json".into(),
                role: EvidenceRole::Input,
                group: None,
                content: Some(ContentIdentity::of(&original_input)),
                omission: None,
                failure: None,
            }];
            if let Some(content) = &inventory {
                evidence.push(EvidenceFile {
                    root: EvidenceRoot::Workflow,
                    path: "inventory/inventory.json".into(),
                    role: EvidenceRole::Inventory,
                    group: None,
                    content: Some(content.clone()),
                    omission: None,
                    failure: None,
                });
            }
            serde_json::to_vec(&ReportObservation {
                version: 3,
                invocation: id.clone(),
                action: Action::Test,
                outcome,
                inventory: inventory
                    .as_ref()
                    .map(|_| "inventory/inventory.json".into()),
                selected_tests: None,
                required_capture_complete: true,
                failures,
                inventory_content: inventory,
                evidence: Some(evidence),
            })
            .unwrap()
        };
        let original = report_bytes(Outcome::TestsSucceeded, vec![], None);
        fs::write(&path, &original).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let record = terminal(&id, &original);
        assert!(
            validate_report(directory.clone(), observed(&id, &record), Some(0))
                .await
                .is_ok()
        );
        assert!(
            validate_report(directory.clone(), observed(&id, &record), Some(1))
                .await
                .is_err()
        );
        let mut legacy: serde_json::Value = serde_json::from_slice(&original).unwrap();
        legacy["version"] = 2.into();
        legacy.as_object_mut().unwrap().remove("evidence");
        let legacy = serde_json::to_vec(&legacy).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, &legacy).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        assert!(
            validate_report(
                directory.clone(),
                observed(&id, &terminal(&id, &legacy)),
                Some(0)
            )
            .await
            .is_err(),
            "retained interpretation two cannot be newly published by a current helper"
        );
        let failure = crate::envelope::ErrorRecord::from_typed(
            &HostError::Ownership("original fixture prerequisite unavailable".into()),
            &RedactionPolicy::from_environment(),
        );
        let failed = report_bytes(Outcome::Failed, vec![failure], None);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, &failed).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let mut failed_record = terminal(&id, &failed);
        failed_record.outcome = Outcome::Failed;
        let validated = validate_report(
            directory.clone(),
            observed(&id, &failed_record),
            Some(failed_record.helper_exit()),
        )
        .await
        .unwrap();
        let (_, _, failures) = validated.into_parts();
        assert_eq!(failures.len(), 1);
        assert!(
            failures[0]
                .message
                .contains("original fixture prerequisite unavailable")
        );
        let inventory_parent = parent.join("inventory");
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&inventory_parent)
            .unwrap();
        let inventory = inventory_parent.join("inventory.json");
        let original_inventory = br#"{"member":"first"}"#;
        let replacement_inventory = br#"{"member":"other"}"#;
        assert_eq!(original_inventory.len(), replacement_inventory.len());
        fs::write(&inventory, original_inventory).unwrap();
        fs::set_permissions(&inventory, fs::Permissions::from_mode(0o400)).unwrap();
        let bound = report_bytes(
            Outcome::TestsSucceeded,
            vec![],
            Some(ContentIdentity::of(original_inventory)),
        );
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, &bound).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let bound_terminal = terminal(&id, &bound);
        assert!(
            validate_report(directory.clone(), observed(&id, &bound_terminal), Some(0))
                .await
                .is_ok()
        );
        fs::set_permissions(&inventory, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&inventory, replacement_inventory).unwrap();
        fs::set_permissions(&inventory, fs::Permissions::from_mode(0o400)).unwrap();
        assert!(
            validate_report(directory.clone(), observed(&id, &bound_terminal), Some(0))
                .await
                .is_err()
        );
        fs::remove_file(&inventory).unwrap();
        assert!(
            validate_report(directory.clone(), observed(&id, &bound_terminal), Some(0))
                .await
                .is_err()
        );
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, b"{\"status\":\"tampered\"}").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        assert!(
            validate_report(directory.clone(), observed(&id, &record), Some(0))
                .await
                .is_err()
        );
        fs::remove_file(&path).unwrap();
        std::os::unix::fs::symlink(scratch.path().join("outside.json"), &path).unwrap();
        assert!(
            validate_report(directory, observed(&id, &record), Some(0))
                .await
                .is_err()
        );
    }
}
