// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The fixed helper's native operations; placement and descendant settlement remain outside.
use super::{
    Action, Input, Outcome, Pending, Phase, PhaseKind, Report, phase, report::GroupObservation,
};
use crate::{
    envelope::RedactionPolicy,
    environment::Environment,
    extraction::{CompilerOutcome, NextestOutcome, OutputStream},
    managed::{result::GenericObserver, streams::ChunkObserver},
    selection,
    test_workflow::{self, CompilerObserver, GroupCommand, NextestObserver, WorkflowError},
};
use std::{
    collections::BTreeSet,
    ffi::OsStr,
    fs,
    num::NonZeroUsize,
    os::unix::fs::{DirBuilderExt as _, PermissionsExt as _},
    path::{Path, PathBuf},
    process::Command,
};

fn directory(path: &Path) -> Result<(), WorkflowError> {
    fs::DirBuilder::new().mode(0o700).create(path)?;
    Ok(())
}
fn command(request: GroupCommand) -> Command {
    let mut command = Command::new(request.program);
    command
        .args(request.arguments)
        .current_dir(request.cwd)
        .env_clear()
        .envs(request.environment);
    command
}
fn nextest(environment: &Environment) -> Result<PathBuf, WorkflowError> {
    let paths = environment
        .get(OsStr::new("PATH"))
        .ok_or_else(|| WorkflowError::Invalid("nextest selection requires PATH".into()))?;
    for directory in std::env::split_paths(paths) {
        let path = directory.join("cargo-nextest");
        match fs::metadata(&path) {
            Ok(metadata) if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 => {
                return Ok(path.canonicalize()?);
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Err(WorkflowError::Invalid(
        "pinned cargo-nextest is unprepared on the selected PATH".into(),
    ))
}
fn record_phase<O: ChunkObserver>(
    report: &mut Report,
    kind: PhaseKind,
    group: Option<usize>,
    prefix: &str,
    capture: &mut phase::Captured<O>,
) {
    report.required_capture_complete &= capture.capture_complete;
    report.phases.push(Phase {
        kind,
        group,
        stdout: format!("{prefix}.stdout"),
        stderr: format!("{prefix}.stderr"),
        child_exit: capture.child_exit,
        capture_complete: capture.capture_complete,
        stdout_capture: capture.stdout.clone(),
        stderr_capture: capture.stderr.clone(),
    });
    for failure in capture.failures.drain(..) {
        report.failure(failure);
    }
}
fn failed(report: &mut Report, error: WorkflowError) {
    let incomplete = matches!(
        &error,
        WorkflowError::Discovery(crate::discovery::DiscoveryError::Capture { .. })
            | WorkflowError::ChildWait { .. }
    );
    report.failure(error);
    report.outcome = if incomplete {
        report.required_capture_complete = false;
        Outcome::Incomplete
    } else {
        Outcome::Failed
    };
}
fn native_pending(report: &mut Report, capabilities: BTreeSet<String>, error: WorkflowError) {
    report.pending.push(Pending::Native { capabilities });
    report.failure(error);
    report.outcome = Outcome::Pending;
}

/// Run only inside the authenticated fixed helper's existing managed physical scope.
/// `destination` must be a new absolute private directory. After initialization, original
/// operation failures are retained in the returned report, including successful-child capture
/// failures; no report or child exit establishes physical settlement. This never prepares a
/// native provider or starts a fixture. The caller publishes the report and final frame.
pub async fn run(
    input: &Input,
    checkout: &Path,
    destination: &Path,
    environment: Environment,
) -> Result<Report, WorkflowError> {
    if !destination.is_absolute() {
        return Err(WorkflowError::Invalid(
            "workflow evidence destination must be absolute".into(),
        ));
    }
    directory(destination)?;
    let mut report = Report::new(input);
    let result = perform(input, checkout, destination, environment, &mut report).await;
    if let Err(error) = result {
        failed(&mut report, error);
    }
    Ok(report)
}
async fn perform(
    input: &Input,
    checkout: &Path,
    destination: &Path,
    environment: Environment,
    report: &mut Report,
) -> Result<(), WorkflowError> {
    if input.version != 1 || input.profile.is_empty() {
        return Err(WorkflowError::Invalid(
            "unsupported workflow input or empty profile".into(),
        ));
    }
    let operation = if input.action == Action::Check {
        selection::Operation::Check
    } else {
        selection::Operation::Test
    };
    if input.request.operation != operation {
        return Err(WorkflowError::Invalid(
            "workflow action and selected compiler operation differ".into(),
        ));
    }
    let record = NonZeroUsize::new(input.bounds.record_bytes)
        .ok_or_else(|| WorkflowError::Invalid("workflow record bound must be positive".into()))?;
    let grouping = NonZeroUsize::new(input.bounds.grouping_capacity)
        .ok_or_else(|| WorkflowError::Invalid("workflow grouping bound must be positive".into()))?;
    let checkout = checkout.canonicalize()?;
    let cargo = crate::correctness::real_cargo_in(&environment)?;
    let mut metadata = Command::new(cargo);
    metadata
        .args([
            "metadata",
            "--format-version=1",
            "--locked",
            "--offline",
            "--no-deps",
        ])
        .current_dir(&checkout)
        .env_clear()
        .envs(&environment);
    let mut captured = phase::run(
        metadata,
        &destination.join("metadata.stdout"),
        &destination.join("metadata.stderr"),
        GenericObserver,
        input.policy,
    )
    .await?;
    record_phase(report, PhaseKind::Metadata, None, "metadata", &mut captured);
    if let Some(exit) = captured.child_exit.filter(|exit| *exit != 0) {
        report.failure(WorkflowError::PhaseExit {
            phase: "metadata",
            exit,
        });
    }
    if !captured.capture_complete || !report.causes.is_empty() || captured.child_exit != Some(0) {
        report.outcome = if captured.capture_complete && captured.child_exit.is_some() {
            Outcome::Failed
        } else {
            Outcome::Incomplete
        };
        return Ok(());
    }
    let metadata: cargo_metadata::Metadata =
        serde_json::from_slice(&fs::read(destination.join("metadata.stdout"))?)
            .map_err(crate::discovery::DiscoveryError::from)?;
    if metadata.workspace_root.as_std_path().canonicalize()? != checkout {
        return Err(WorkflowError::Invalid(
            "captured Cargo workspace differs from selected checkout".into(),
        ));
    }
    if input.action == Action::Check {
        let mut admitted =
            test_workflow::CheckIntent::new(metadata, input.request.clone(), environment)?
                .admit()?;
        report.selection = Some(admitted.resolved().plan().clone());
        report.admission = admitted.resolved().admitted().to_vec();
        let capabilities = admitted.resolved().native_capabilities();
        if let Err(error) = admitted.select_native() {
            if matches!(error, WorkflowError::NativeReadiness(_)) {
                native_pending(report, capabilities, error);
                return Ok(());
            }
            return Err(error);
        }
        let prepared = admitted.prepare(&destination.join("check-control"))?;
        let mut groups = Vec::new();
        for group in 0..report.admission.len() {
            let capture = destination.join(format!("check-build-{group}"));
            directory(&capture)?;
            let mut request = prepared.command(group)?;
            request.environment.insert(
                "PSE_CORRECTNESS_BUILD_CAPTURE".into(),
                capture.into_os_string(),
            );
            groups.push((
                request,
                GroupObserver::Check(CompilerObserver::new(
                    record,
                    grouping,
                    input.bounds.failure_records,
                    RedactionPolicy::from_environment(),
                )),
            ));
        }
        execute_groups(input, destination, groups, report).await?;
        // Check's original callback publishes its exact compiler-unit context too. Admission
        // alone cannot claim that later compilation used the same capability/feature closure.
        for group in 0..report.admission.len() {
            if let Err(error) = check_capture(destination, group, report) {
                report.failure(error);
                report.required_capture_complete = false;
                if report.outcome != Outcome::Failed {
                    report.outcome = Outcome::Incomplete;
                }
            }
        }
    } else {
        let mut admitted = test_workflow::Intent::new(
            metadata,
            input.request.clone(),
            input.profile.clone(),
            environment,
        )?
        .admit()?;
        report.selection = Some(admitted.resolved.plan().clone());
        report.admission = admitted.resolved.admitted().to_vec();
        let capabilities = admitted.resolved.native_capabilities();
        if let Err(error) = admitted.context.select_native(&admitted.resolved) {
            if matches!(error, WorkflowError::NativeReadiness(_)) {
                native_pending(report, capabilities, error);
                return Ok(());
            }
            return Err(error);
        }
        let nextest = nextest(&admitted.context.environment)?;
        let listed = admitted
            .list(&nextest, &destination.join("inventory"), input.policy)
            .await?;
        report.inventory = Some("inventory/inventory.json".into());
        report.selected_tests = Some(listed.inventory.selected_tests_iter().count());
        report.required_capture_complete &= listed.inventory.required_capture_complete;
        discovery_phases(report, &listed.inventory);
        if !listed.inventory.all_suites_listed() {
            report.outcome = Outcome::Incomplete;
            return Ok(());
        }
        if listed.inventory.is_zero_match() {
            report.outcome = Outcome::ZeroMatch;
            return Ok(());
        }
        if input.action == Action::List {
            report.outcome = Outcome::Listed;
            return Ok(());
        }
        for requirement in &listed.requirements {
            let mut applies = false;
            for identity in listed.inventory.selected_tests_iter() {
                let captured = listed.inventory.test(&identity)?;
                if test_workflow::matches_requirement(
                    requirement,
                    &identity,
                    &captured.artifact.cargo,
                ) {
                    applies = true;
                    break;
                }
            }
            if applies {
                report.pending.push(Pending::Fixture {
                    requirement: requirement.clone(),
                    readiness: crate::fixtures::readiness(
                        requirement,
                        &crate::fixtures::Sources::default(),
                    )?,
                });
            }
        }
        if !report.pending.is_empty() {
            report.outcome = Outcome::Pending;
            return Ok(());
        }
        if !group_policy(input, destination, &listed, report).await? {
            return Ok(());
        }
        let mut groups = Vec::new();
        for group in 0..listed.inventory.groups.len() {
            // Empty individual contexts have no runnable work; the aggregate population above
            // remains nonempty and every selected context is accounted independently.
            if !listed
                .inventory
                .selected_tests_iter()
                .any(|identity| identity.group == group)
            {
                continue;
            }
            groups.push((
                listed.command(group)?,
                GroupObserver::Test(listed.observer(
                    group,
                    record,
                    RedactionPolicy::from_environment(),
                )?),
            ));
        }
        execute_groups(input, destination, groups, report).await?;
    }
    Ok(())
}
fn check_capture(destination: &Path, group: usize, report: &Report) -> Result<(), WorkflowError> {
    let path = destination.join(format!("check-build-{group}/build.json"));
    let captured: crate::discovery::BuildCapture = serde_json::from_slice(&fs::read(&path)?)
        .map_err(crate::discovery::DiscoveryError::from)?;
    let expected = &report.admission[group];
    if captured.version != 1
        || !captured.incomplete.is_empty()
        || report
            .phases
            .iter()
            .find(|phase| phase.group == Some(group) && matches!(phase.kind, PhaseKind::Check))
            .and_then(|phase| phase.child_exit)
            != Some(captured.cargo_exit)
        || captured.resolution.native_capabilities != expected.native_capabilities
        || captured.resolution.units.len() != expected.units.len()
        || !captured
            .resolution
            .units
            .iter()
            .all(|unit| expected.units.contains(unit))
    {
        return Err(WorkflowError::Invalid(format!(
            "actual check compiler capture differs from admitted context at {}",
            path.display()
        )));
    }
    report
        .selection
        .as_ref()
        .ok_or_else(|| WorkflowError::Invalid("original check selection is absent".into()))?
        .groups()[group]
        .validate_resolution(&captured.resolution)?;
    Ok(())
}
fn discovery_phases(report: &mut Report, inventory: &crate::discovery::Inventory) {
    for captured in &inventory.capture {
        let prefix = match captured.group {
            Some(group) => format!("inventory/group-{group}/{}", captured.stage),
            None => format!("inventory/{}", captured.stage),
        };
        report.phases.push(Phase {
            kind: PhaseKind::Discovery,
            group: captured.group,
            stdout: format!("{prefix}.stdout"),
            stderr: format!("{prefix}.stderr"),
            child_exit: Some(captured.child_exit),
            capture_complete: captured.stdout.complete && captured.stderr.complete,
            stdout_capture: Some(captured.stdout.clone()),
            stderr_capture: Some(captured.stderr.clone()),
        });
    }
}
/// Ask the native policy evaluator for membership, never decode config/filter grammar.
async fn group_policy(
    input: &Input,
    destination: &Path,
    listed: &test_workflow::Listed,
    report: &mut Report,
) -> Result<bool, WorkflowError> {
    if listed.inventory.groups.len() < 2 {
        return Ok(true);
    }
    let mut named = Vec::new();
    for group in 0..listed.inventory.groups.len() {
        if !listed
            .inventory
            .selected_tests_iter()
            .any(|identity| identity.group == group)
        {
            continue;
        }
        let directory = listed.capture_root.join(format!("group-{group}"));
        let mut command = Command::new(&listed.inventory.nextest.captured);
        command
            .args([
                "nextest",
                "list",
                "--message-format=json",
                "--profile",
                &listed.inventory.runner_profile,
            ])
            .arg("--config-file")
            .arg(listed.capture_root.join("nextest.toml"))
            .arg("--binaries-metadata")
            .arg(directory.join("binaries.json"))
            .arg("--cargo-metadata")
            .arg(directory.join("cargo-metadata.json"))
            .args(["--run-ignored", "all", "-E", "not group(@global)"])
            .current_dir(&listed.context.checkout)
            .env_clear()
            .envs(&listed.context.environment);
        if listed.inventory.selection.filters().ignore_default_filter {
            command.arg("--ignore-default-filter");
        }
        let prefix = format!("group-policy-{group}");
        let mut capture = phase::run(
            command,
            &destination.join(format!("{prefix}.stdout")),
            &destination.join(format!("{prefix}.stderr")),
            GenericObserver,
            input.policy,
        )
        .await?;
        record_phase(
            report,
            PhaseKind::GroupPolicy,
            Some(group),
            &prefix,
            &mut capture,
        );
        if let Some(exit) = capture.child_exit.filter(|exit| *exit != 0) {
            report.failure(WorkflowError::PhaseExit {
                phase: "nextest-group-policy",
                exit,
            });
        }
        if !capture.capture_complete || !report.causes.is_empty() || capture.child_exit != Some(0) {
            report.outcome = if capture.capture_complete && capture.child_exit.is_some() {
                Outcome::Failed
            } else {
                Outcome::Incomplete
            };
            return Ok(false);
        }
        let membership: nextest_metadata::TestListSummary =
            serde_json::from_slice(&fs::read(destination.join(format!("{prefix}.stdout")))?)
                .map_err(crate::discovery::DiscoveryError::from)?;
        // Multiple -E arguments are a union in native nextest. Query membership alone,
        // then intersect exact native binary/test identities with the original population.
        if listed
            .inventory
            .selected_tests_iter()
            .filter(|identity| identity.group == group)
            .any(|identity| {
                membership
                    .rust_suites
                    .get(&nextest_metadata::RustBinaryId::new(&identity.binary_id))
                    .filter(|suite| suite.binary.package_id == identity.package_id)
                    .and_then(|suite| {
                        suite
                            .test_cases
                            .get(&nextest_metadata::TestCaseName::new(&identity.test_name))
                    })
                    .is_some_and(|case| case.filter_match.is_match())
            })
        {
            named.push(group);
        }
    }
    if named.len() > 1 {
        report
            .pending
            .push(Pending::CrossRunnerGroups { groups: named });
        report.outcome = Outcome::Pending;
        return Ok(false);
    }
    Ok(true)
}
enum GroupObserver {
    Check(CompilerObserver),
    Test(NextestObserver),
}
impl ChunkObserver for GroupObserver {
    type Error = WorkflowError;
    fn observe(
        &mut self,
        stream: OutputStream,
        offset: u64,
        bytes: &[u8],
    ) -> Result<(), Self::Error> {
        match self {
            Self::Check(observer) => observer.observe(stream, offset, bytes).map_err(Into::into),
            Self::Test(observer) => observer.observe(stream, offset, bytes).map_err(Into::into),
        }
    }
}
impl GroupObserver {
    fn finish(
        self,
        group: usize,
        exit: i32,
        complete: bool,
        failure_budget: usize,
    ) -> (GroupObservation, bool, bool, bool, Option<WorkflowError>) {
        match self {
            Self::Check(observer) => {
                let (observation, error) = observer.finish(exit, complete);
                let success = observation.summary.outcome() == CompilerOutcome::Succeeded;
                let failed = observation.summary.outcome() == CompilerOutcome::Failed;
                let complete = observation.summary.complete();
                (
                    GroupObservation::Check { group, observation },
                    success,
                    failed,
                    complete,
                    error.map(Into::into),
                )
            }
            Self::Test(observer) => {
                let (mut observation, error) = observer.finish(exit, complete);
                let success = observation.summary.outcome() == NextestOutcome::Succeeded;
                let failed = matches!(
                    observation.summary.outcome(),
                    NextestOutcome::TestFailed
                        | NextestOutcome::SetupFailed
                        | NextestOutcome::RunnerFailed
                );
                let complete = observation.summary.complete();
                let omitted_failure_records =
                    observation.failures.len().saturating_sub(failure_budget);
                observation.failures.truncate(failure_budget);
                (
                    GroupObservation::Test {
                        group,
                        observation,
                        failure_budget,
                        omitted_failure_records,
                    },
                    success,
                    failed,
                    complete,
                    error.map(Into::into),
                )
            }
        }
    }
}
async fn execute_groups(
    input: &Input,
    destination: &Path,
    groups: Vec<(GroupCommand, GroupObserver)>,
    report: &mut Report,
) -> Result<(), WorkflowError> {
    if groups.is_empty() {
        report.outcome = Outcome::Incomplete;
        return Ok(());
    }
    let kind = if input.action == Action::Check {
        PhaseKind::Check
    } else {
        PhaseKind::Test
    };
    let mut tasks = tokio::task::JoinSet::new();
    for (request, observer) in groups {
        let group = request.group;
        let prefix = format!("run-{group}");
        let stdout = destination.join(format!("{prefix}.stdout"));
        let stderr = destination.join(format!("{prefix}.stderr"));
        let policy = input.policy;
        tasks.spawn(async move {
            (
                group,
                prefix,
                phase::run(command(request), &stdout, &stderr, observer, policy).await,
            )
        });
    }
    let mut success = true;
    let mut known_failure = false;
    let mut complete = true;
    while let Some(result) = tasks.join_next().await {
        match result {
            Err(error) => {
                report.failure(crate::HostError::Task(error).into());
                success = false;
                complete = false;
                report.required_capture_complete = false;
            }
            Ok((group, prefix, Err(error))) => {
                report.phases.push(Phase {
                    kind,
                    group: Some(group),
                    stdout: format!("{prefix}.stdout"),
                    stderr: format!("{prefix}.stderr"),
                    child_exit: None,
                    capture_complete: false,
                    stdout_capture: None,
                    stderr_capture: None,
                });
                report.failure(error);
                success = false;
                complete = false;
                report.required_capture_complete = false;
            }
            Ok((group, prefix, Ok(mut captured))) => {
                record_phase(report, kind, Some(group), &prefix, &mut captured);
                if let Some(exit) = captured.child_exit.filter(|exit| *exit != 0) {
                    report.failure(WorkflowError::PhaseExit {
                        phase: if input.action == Action::Check {
                            "check"
                        } else {
                            "test"
                        },
                        exit,
                    });
                }
                let Some(mut tee) = captured.capture.take() else {
                    success = false;
                    complete = false;
                    continue;
                };
                for error in tee.observer_errors.iter_mut().filter_map(Option::take) {
                    report.failure(error);
                    success = false;
                }
                match (tee.observer.take(), captured.child_exit) {
                    (Some(observer), Some(exit)) => {
                        let (observation, passed, failed, known, error) = observer.finish(
                            group,
                            exit,
                            captured.capture_complete,
                            input.bounds.failure_records,
                        );
                        report.observations.push(observation);
                        success &= passed;
                        known_failure |= failed;
                        complete &= known;
                        if let Some(error) = error {
                            report.failure(error);
                            success = false;
                        }
                    }
                    _ => {
                        report.failure(WorkflowError::Invalid(
                            "required native group observer or waited exit is absent".into(),
                        ));
                        success = false;
                        complete = false;
                    }
                }
            }
        }
    }
    report.phases.sort_by_key(|phase| phase.group);
    report.outcome = if known_failure {
        Outcome::Failed
    } else if !report.required_capture_complete || !complete {
        Outcome::Incomplete
    } else if !success || !report.causes.is_empty() {
        Outcome::Failed
    } else if input.action == Action::Check {
        Outcome::CheckSucceeded
    } else {
        Outcome::TestsSucceeded
    };
    Ok(())
}

#[cfg(test)]
mod tests;
