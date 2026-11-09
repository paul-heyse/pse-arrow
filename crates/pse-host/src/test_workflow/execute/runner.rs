// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The fixed helper's native operations; placement and descendant settlement remain outside.
use super::{
    Action, EvidenceRole, Input, Outcome, Pending, Phase, PhaseKind, Report, phase,
    progress::{LiveGroup, LivePhase, Writer as ProgressWriter},
    report::GroupObservation,
};
use crate::{
    discovery::{self, BuiltArtifact, ListedGroup, TestIdentity},
    envelope::RedactionPolicy,
    environment::Environment,
    extraction::{CompilerOutcome, NextestOutcome, OutputStream},
    fixtures::{FixtureError, Input as FixtureInput, Ownership, Requirement},
    managed::{result::GenericObserver, streams::ChunkObserver},
    selection,
    test_workflow::{self, CompilerObserver, CompletedObserver, GroupCommand, WorkflowError},
};
use nextest_metadata::{RustTestSuiteStatusSummary, RustTestSuiteSummary, TestListSummary};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fs,
    num::NonZeroUsize,
    os::unix::fs::DirBuilderExt as _,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
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
    Ok(crate::correctness::resolve_nextest(
        environment
            .get(OsStr::new("PATH"))
            .map(|path| path.as_os_str()),
    )?)
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
        predecessor_wait: None,
        native_stage: None,
        elapsed_seconds: Some(capture.elapsed_seconds),
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
fn failed(report: &mut Report, error: WorkflowError, destination: &Path) {
    let mut incomplete = match &error {
        WorkflowError::Discovery(error) => error.capture_incomplete(),
        WorkflowError::ChildWait { .. } => true,
        _ => false,
    };
    if let WorkflowError::Discovery(discovery::DiscoveryError::Attempt {
        evidence, phases, ..
    }) = &error
    {
        let prefix = evidence
            .strip_prefix(destination)
            .ok()
            .filter(|prefix| {
                !prefix.as_os_str().is_empty()
                    && prefix
                        .components()
                        .all(|part| matches!(part, std::path::Component::Normal(_)))
            })
            .and_then(Path::to_str);
        match prefix {
            Some(prefix) => {
                for phase in phases {
                    let mut original = phase.clone();
                    original.stdout = format!("{prefix}/{}", phase.stdout);
                    original.stderr = format!("{prefix}/{}", phase.stderr);
                    report.required_capture_complete &= original.capture_complete;
                    report.phases.push(original);
                }
            }
            _ => {
                report.failure(WorkflowError::Invalid(
                    "original discovery account is outside the performing evidence root".into(),
                ));
                incomplete = true;
            }
        }
    }
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

fn prepare_fixtures(
    report: &mut Report,
    root: &Path,
    environment: &Environment,
    destination: &Path,
    requirements: &[Requirement],
    permit: Option<&crate::build_admission::Permit>,
) -> Result<Option<crate::fixtures::PreparedExecution>, WorkflowError> {
    let assessed = crate::fixtures::ExecutionInputs::assess(root, environment, requirements);
    if assessed.blocked() {
        for (requirement, readiness) in requirements.iter().zip(assessed.observations()) {
            report
                .pending
                .push(Pending::Fixture(Box::new(super::FixturePending {
                    requirement: requirement.clone(),
                    readiness: readiness.clone(),
                })));
        }
        if let Err(error) = assessed.prepare(destination) {
            report.failure(error.into());
        }
        report.outcome = Outcome::Pending;
        return Ok(None);
    }
    if let Some(permit) = permit {
        permit.stage(crate::build_admission::Stage::Building)?;
    }
    Ok(Some(assessed.prepare_with_permit(destination, permit)?))
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
    run_inner(input, checkout, destination, environment, false).await
}

pub(crate) async fn run_managed(
    input: &Input,
    checkout: &Path,
    destination: &Path,
    environment: Environment,
) -> Result<Report, WorkflowError> {
    run_inner(input, checkout, destination, environment, true).await
}

async fn run_inner(
    input: &Input,
    checkout: &Path,
    destination: &Path,
    environment: Environment,
    coordinated: bool,
) -> Result<Report, WorkflowError> {
    if !destination.is_absolute() {
        return Err(WorkflowError::Invalid(
            "workflow evidence destination must be absolute".into(),
        ));
    }
    directory(destination)?;
    let mut report = Report::new(input);
    let progress = coordinated
        .then(|| ProgressWriter::new(destination, input.invocation.clone(), input.action).ok())
        .flatten()
        .map(Arc::new);
    let result = perform(
        input,
        checkout,
        destination,
        environment,
        &mut report,
        coordinated,
        progress.as_ref(),
    )
    .await;
    if let Err(error) = result {
        failed(&mut report, error, destination);
    }
    if let Some(progress) = progress {
        progress.phase(LivePhase::PublishingReport);
    }
    Ok(report)
}

async fn acquire_build(
    checkout: &Path,
    outputs: Option<crate::build_admission::Outputs>,
    input: &Input,
) -> Result<Option<crate::build_admission::Permit>, WorkflowError> {
    let Some(outputs) = outputs else {
        return Ok(None);
    };
    let root = checkout.to_owned();
    let invocation = input.invocation.clone();
    let permit = tokio::task::spawn_blocking(move || {
        crate::build_admission::Permit::acquire(&root, outputs, Some(invocation))
    })
    .await
    .map_err(crate::HostError::from)??
    .ok_or(WorkflowError::BuildCancelled)?;
    Ok(Some(permit))
}

async fn perform(
    input: &Input,
    checkout: &Path,
    destination: &Path,
    environment: Environment,
    report: &mut Report,
    coordinated: bool,
    progress: Option<&Arc<ProgressWriter>>,
) -> Result<(), WorkflowError> {
    use crate::build_admission::{CALLBACK, Outputs, Stage};
    if input.version != 1 || input.profile.is_empty() {
        return Err(WorkflowError::Invalid(
            "unsupported workflow input or empty profile".into(),
        ));
    }
    let operation = match input.action {
        Action::Check => selection::Operation::Check,
        Action::Rustdoc => selection::Operation::Rustdoc,
        Action::List | Action::Test => selection::Operation::Test,
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
    if let Some(progress) = progress {
        progress.phase(LivePhase::ReadingMetadata);
    }
    report.expect_file("metadata.stdout".into(), EvidenceRole::Stdout, None, false);
    report.expect_file("metadata.stderr".into(), EvidenceRole::Stderr, None, false);
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
            .map_err(discovery::DiscoveryError::from)?;
    if metadata.workspace_root.as_std_path().canonicalize()? != checkout {
        return Err(WorkflowError::Invalid(
            "captured Cargo workspace differs from selected checkout".into(),
        ));
    }
    if let Some(progress) = progress {
        progress.phase(LivePhase::ResolvingSelection);
    }
    let outputs = if coordinated {
        let build_directory = metadata.build_directory.as_ref().ok_or_else(|| {
            WorkflowError::Invalid("Cargo intermediate directory is missing".into())
        })?;
        Some(Outputs::new(
            metadata.target_directory.as_std_path(),
            build_directory.as_std_path(),
        )?)
    } else {
        None
    };
    if input.action.is_compiler() {
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
        let mut permit = acquire_build(&checkout, outputs, input).await?;
        let prepared = admitted.prepare(&destination.join("check-control"))?;
        let mut groups = Vec::new();
        for group in 0..report.admission.len() {
            let capture = destination.join(format!("check-build-{group}"));
            report.expect_path(
                destination,
                &capture.join("build.json"),
                EvidenceRole::CompilerContext,
                Some(group),
                false,
            )?;
            directory(&capture)?;
            let mut request = prepared.command(group)?;
            if let Some(permit) = &permit {
                request
                    .environment
                    .insert(CALLBACK.into(), permit.callback());
            }
            if input.timings {
                let end = request
                    .arguments
                    .iter()
                    .position(|argument| argument == "--")
                    .unwrap_or(request.arguments.len());
                request.arguments.insert(end, "--timings".into());
            }
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
        if permit.is_some() {
            // Each group uses the same observed Cargo output domain.
            let serialized = (0..report.admission.len())
                .map(|group| (group, BTreeSet::from(["cargo output".to_owned()])))
                .collect();
            execute_groups_observed(input, destination, groups, report, &serialized, progress).await?;
        } else {
            execute_groups_observed(input, destination, groups, report, &BTreeMap::new(), progress).await?;
        }
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
        if let Some(permit) = &mut permit {
            if !report.required_capture_complete {
                permit.retain_for_drain();
            }
            permit.stage(Stage::Capturing)?;
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
        let mut permit = acquire_build(&checkout, outputs, input).await?;
        if let Some(progress) = progress {
            progress.phase(LivePhase::DiscoveringTests);
        }
        let listed = match admitted
            .list_with_permit(
                &nextest,
                &destination.join("inventory"),
                discovery::DiscoveryOptions {
                    policy: input.policy,
                    timings: input.timings,
                },
                permit.as_ref(),
            )
            .await
        {
            Ok(listed) => listed,
            Err(error) => {
                if let Some(permit) = &mut permit
                    && matches!(&error, WorkflowError::Discovery(error) if error.capture_incomplete())
                {
                    permit.retain_for_drain();
                }
                return Err(error);
            }
        };
        report.record_inventory(destination)?;
        report.fixture_requirements = listed.requirements.clone();
        report.selected_tests = Some(listed.inventory.selected_tests_iter().count());
        report.required_capture_complete &= listed.inventory.required_capture_complete;
        discovery_phases(report, &listed.inventory);
        if !listed.inventory.selected_population_complete() {
            report.outcome = Outcome::Incomplete;
            return Ok(());
        }
        test_workflow::source::verify_rerun(&listed.inventory)?;
        if listed.inventory.is_zero_match() {
            report.outcome = Outcome::ZeroMatch;
            return Ok(());
        }
        if input.action == Action::List {
            report.outcome = Outcome::Listed;
            return Ok(());
        }
        if let Some(progress) = progress {
            progress.phase(LivePhase::ResolvingFixtureInputs);
        }
        let FixtureSelection {
            requirements,
            mut target_owned,
        } = fixture_requirements(input, destination, &listed, report).await?;
        if let Some(progress) = progress.filter(|_| !requirements.is_empty()) {
            progress.phase(LivePhase::PreparingFixtureInputs);
        }
        let Some(fixtures) = prepare_fixtures(
            report,
            &checkout,
            &listed.context.environment,
            &destination.join("fixtures"),
            &requirements,
            permit.as_ref(),
        )?
        else {
            return Ok(());
        };
        if let Some(permit) = &permit {
            permit.stage(Stage::Listing)?;
        }
        if let Some(progress) = progress {
            progress.phase(LivePhase::ResolvingGroupPolicy);
        }
        let serialized = group_policy(input, destination, &listed, report).await?;
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
            let mut observer = listed.observer(group)?;
            let mut request = listed.command(group)?;
            let settlement_env = crate::fixtures::target_settlement::DIRECTORY_ENV;
            request.environment.remove(OsStr::new(settlement_env));
            if let Some(cases) = target_owned.remove(&group) {
                let settlement = destination.join(format!("target-settlement-{group}"));
                directory(&settlement)?;
                request
                    .environment
                    .insert(settlement_env.into(), settlement.clone().into_os_string());
                observer = observer
                    .with_target_owned(cases)
                    .with_target_settlement(settlement);
            }
            report.expect_path(
                destination,
                observer.report_path(),
                EvidenceRole::RunnerReport,
                Some(group),
                false,
            )?;
            if requirements.iter().any(|requirement| {
                requirement.group() == group && requirement.ownership() == Ownership::RunnerBorrowed
            }) {
                let outcome = destination.join(format!("fixture-outcome-{group}.json"));
                report.expect_path(
                    destination,
                    &outcome,
                    EvidenceRole::FixtureOutcome,
                    Some(group),
                    false,
                )?;
                observer = observer.with_fixture_outcome(outcome);
            }
            report.expect_path(
                destination,
                &listed.run_configuration_path(group),
                EvidenceRole::RunnerConfiguration,
                Some(group),
                true,
            )?;
            groups.push((
                fixtures.command(
                    request,
                    &requirements,
                    &destination.join("inventory/inventory.json"),
                    destination,
                    &input.invocation,
                )?,
                GroupObserver::Test(observer),
            ));
        }
        if let Some(permit) = &mut permit {
            // Ordinary runtime can issue its own supervised build. Protect the owned
            // compiler, capture and listing stages; preserve the original runtime boundary.
            permit.release_to_runner()?;
        }
        execute_groups_observed(input, destination, groups, report, &serialized, progress).await?;
    }
    Ok(())
}
fn check_capture(destination: &Path, group: usize, report: &Report) -> Result<(), WorkflowError> {
    let path = destination.join(format!("check-build-{group}/build.json"));
    let captured: discovery::BuildCapture =
        serde_json::from_slice(&fs::read(&path)?).map_err(discovery::DiscoveryError::from)?;
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
        || !captured.resolution.same_units(expected)
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
fn discovery_phases(report: &mut Report, inventory: &discovery::Inventory) {
    for timing in &inventory.cargo_timings {
        if let Some(path) = &timing.file {
            report.expect_file(
                path.clone(),
                EvidenceRole::DiscoveryContext,
                Some(timing.group),
                true,
            );
        }
    }
    report.cargo_timings.clone_from(&inventory.cargo_timings);
    for captured in &inventory.capture {
        let prefix = match captured.group {
            Some(group) => format!("inventory/group-{group}/{}", captured.stage),
            None => format!("inventory/{}", captured.stage),
        };
        report.phases.push(Phase {
            predecessor_wait: None,
            native_stage: Some(captured.stage.clone()),
            elapsed_seconds: captured.elapsed_seconds,
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

struct FixtureSuite<'a> {
    original: &'a RustTestSuiteSummary,
    artifact: &'a BuiltArtifact,
    requirements: Vec<&'a Requirement>,
}

#[derive(Default)]
struct FixtureSelection {
    requirements: Vec<Requirement>,
    target_owned: BTreeMap<usize, BTreeSet<TestIdentity>>,
}

fn fixture_suites<'a>(
    group: &'a ListedGroup,
    requirements: impl Iterator<Item = &'a Requirement>,
) -> Result<Vec<FixtureSuite<'a>>, WorkflowError> {
    let mut suites = BTreeMap::new();
    for requirement in requirements {
        let mut matching = group.build.artifacts.iter().filter_map(|artifact| {
            if !artifact.cargo.profile.test
                || !test_workflow::matches_requirement_target(requirement, &artifact.cargo)
            {
                return None;
            }
            group
                .listing
                .rust_suites
                .values()
                .find(|suite| {
                    suite.binary.binary_path.as_std_path() == artifact.executable.captured
                })
                .map(|suite| (suite, artifact))
        });
        let (suite, artifact) = matching
            .next()
            .filter(|_| matching.next().is_none())
            .ok_or_else(|| {
                FixtureError::Declaration(format!(
                    "fixture {} for {}:{} in compiler group {} has no unique captured test suite",
                    requirement.declaration().id,
                    requirement.package_id(),
                    requirement.target_name(),
                    requirement.group(),
                ))
            })?;
        suites
            .entry(&suite.binary.binary_id)
            .or_insert_with(|| FixtureSuite {
                original: suite,
                artifact,
                requirements: Vec::new(),
            })
            .requirements
            .push(requirement);
    }
    Ok(suites.into_values().collect())
}

fn scoped_suite(suite: &FixtureSuite<'_>) -> bool {
    suite.original.status != RustTestSuiteStatusSummary::LISTED
        && suite
            .requirements
            .iter()
            .any(|requirement| !requirement.declaration().scope.is_whole_target())
}

fn selected_fixture_requirements(
    group: usize,
    selected_count: usize,
    suites: Vec<FixtureSuite<'_>>,
    supplemental: Option<&TestListSummary>,
    selected: &mut FixtureSelection,
) -> Result<(), WorkflowError> {
    let mut companion = None;
    let mut companion_cases = 0;
    for suite in suites {
        let population = if scoped_suite(&suite) {
            supplemental
                .and_then(|listing| listing.rust_suites.get(&suite.original.binary.binary_id))
                .filter(|suite| suite.status == RustTestSuiteStatusSummary::LISTED)
                .ok_or_else(|| {
                    WorkflowError::Invalid(format!(
                        "fixture scope enumeration omitted captured suite {}",
                        suite.original.binary.binary_id,
                    ))
                })?
        } else {
            suite.original
        };
        for requirement in &suite.requirements {
            requirement
                .validate_test_scope(population.test_cases.keys().map(|case| case.as_str()))?;
        }

        let mut applies = BTreeSet::new();
        for (case, _) in suite
            .original
            .test_cases
            .iter()
            .filter(|(_, case)| case.filter_match.is_match())
        {
            let mut performing: Option<&Requirement> = None;
            let mut needs_companion = false;
            for (ordinal, requirement) in suite.requirements.iter().enumerate() {
                if !requirement.applies_to_test(case.as_str()) {
                    continue;
                }
                applies.insert(ordinal);
                for variant in [
                    FixtureInput::QualificationStudyModule,
                    FixtureInput::LongHoldStudyModule,
                ] {
                    if requirement.inputs().contains(&variant) {
                        if companion.is_some_and(|previous| previous != variant) {
                            return Err(FixtureError::Declaration(format!(
                                "compiler group {group} selects different study-module companion variants"
                            )).into());
                        }
                        companion = Some(variant);
                        needs_companion = true;
                    }
                }
                if requirement.ownership() == Ownership::PreparationOnly {
                    continue;
                }
                if let Some(previous) = performing
                    && previous.ownership() != requirement.ownership()
                {
                    return Err(FixtureError::Declaration(format!(
                        "fixture declarations {} and {} for compiler group {} suite {} case {} disagree on performing ownership",
                        previous.declaration().id,
                        requirement.declaration().id,
                        requirement.group(),
                        suite.original.binary.binary_id,
                        case,
                    ))
                    .into());
                }
                performing = Some(requirement);
            }
            if performing.is_some_and(|owner| owner.ownership() == Ownership::TargetOwned) {
                selected
                    .target_owned
                    .entry(group)
                    .or_default()
                    .insert(TestIdentity {
                        group,
                        package_id: suite.original.binary.package_id.clone(),
                        binary_id: suite.original.binary.binary_id.to_string(),
                        test_name: case.to_string(),
                    });
            }
            companion_cases += usize::from(needs_companion);
        }
        if !applies.is_empty() {
            suite
                .artifact
                .executable
                .verify()
                .map_err(discovery::DiscoveryError::from)?;
            selected.requirements.extend(
                applies
                    .into_iter()
                    .map(|ordinal| suite.requirements[ordinal].clone()),
            );
        }
    }
    if let Some(variant) = companion
        && companion_cases != selected_count
    {
        return Err(FixtureError::Declaration(format!(
            "compiler group {group} selects {companion_cases} cases requiring {variant:?} among {selected_count} selected cases; select the companion cases separately"
        )).into());
    }
    Ok(())
}

async fn fixture_requirements(
    input: &Input,
    destination: &Path,
    listed: &test_workflow::Listed,
    report: &mut Report,
) -> Result<FixtureSelection, WorkflowError> {
    let mut selected = FixtureSelection::default();
    if listed.requirements.is_empty() {
        return Ok(selected);
    }
    for (group, inventory) in listed.inventory.groups.iter().enumerate() {
        let declarations = listed
            .requirements
            .iter()
            .filter(|requirement| requirement.group() == group);
        if declarations.clone().next().is_none() {
            continue;
        }
        let selected_count = inventory
            .listing
            .rust_suites
            .values()
            .flat_map(|suite| suite.test_cases.values())
            .filter(|case| case.filter_match.is_match())
            .count();
        if selected_count == 0 {
            continue;
        }
        let suites = fixture_suites(inventory, declarations)?;
        let expressions = suites
            .iter()
            .filter(|suite| scoped_suite(suite))
            .map(|suite| {
                format!(
                    "binary_id(={})",
                    selection::equality(suite.original.binary.binary_id.as_str())
                )
            })
            .collect::<Vec<_>>();
        // Native Nextest enumerates every case of a listed suite and records its filter result.
        // Re-list only skipped suites whose declared scope needs correspondence evidence.
        let supplemental = if expressions.is_empty() {
            None
        } else {
            Some(
                fixture_scope_listing(input, destination, listed, report, group, &expressions)
                    .await?,
            )
        };
        selected_fixture_requirements(
            group,
            selected_count,
            suites,
            supplemental.as_ref(),
            &mut selected,
        )?;
    }
    Ok(selected)
}

async fn fixture_scope_listing(
    input: &Input,
    destination: &Path,
    listed: &test_workflow::Listed,
    report: &mut Report,
    group: usize,
    expressions: &[String],
) -> Result<TestListSummary, WorkflowError> {
    let mut command = reuse_listing(listed, group)?;
    command.arg("--ignore-default-filter");
    for expression in expressions {
        command.args(["-E", expression]);
    }
    let prefix = format!("fixture-scope-{group}");
    let stdout = format!("{prefix}.stdout");
    let stderr = format!("{prefix}.stderr");
    report.expect_file(stdout.clone(), EvidenceRole::Stdout, Some(group), false);
    report.expect_file(stderr.clone(), EvidenceRole::Stderr, Some(group), false);
    let mut capture = phase::run(
        command,
        &destination.join(&stdout),
        &destination.join(stderr),
        GenericObserver,
        input.policy,
    )
    .await?;
    record_phase(
        report,
        PhaseKind::Discovery,
        Some(group),
        &prefix,
        &mut capture,
    );
    if let Some(phase) = report.phases.last_mut() {
        phase.native_stage = Some("fixture-scope".into());
    }
    if !capture.capture_complete || capture.child_exit.is_none() {
        return Err(discovery::DiscoveryError::Capture {
            stage: "fixture-scope",
            evidence: destination.into(),
            failures: Vec::new(),
        }
        .into());
    }
    if let Some(exit) = capture.child_exit.filter(|exit| *exit != 0) {
        return Err(WorkflowError::PhaseExit {
            phase: "nextest-fixture-scope",
            exit,
        });
    }
    let listing = serde_json::from_slice(&fs::read(destination.join(stdout))?)
        .map_err(discovery::DiscoveryError::from)?;
    discovery::validate_listing(
        &listed.inventory.groups[group].binaries,
        &listing,
        selection::IgnoredPolicy::All,
    )?;
    Ok(listing)
}

fn reuse_listing(listed: &test_workflow::Listed, group: usize) -> Result<Command, WorkflowError> {
    listed.verify_reuse_inputs(group)?;
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
        .args(["--run-ignored", "all"])
        .current_dir(&listed.context.checkout)
        .env_clear()
        .envs(&listed.context.environment)
        .env_remove("NEXTEST_TEST_PHASE");
    Ok(command)
}

/// Ask the native policy evaluator for membership, never decode config/filter grammar.
async fn group_policy(
    input: &Input,
    destination: &Path,
    listed: &test_workflow::Listed,
    report: &mut Report,
) -> Result<BTreeMap<usize, BTreeSet<String>>, WorkflowError> {
    if listed.inventory.groups.len() < 2 {
        return Ok(BTreeMap::new());
    }
    // Read declaration names only. Nextest remains the policy/filter membership authority.
    let configuration = std::str::from_utf8(&listed.inventory.nextest_configuration)
        .map_err(|error| WorkflowError::Invalid(error.to_string()))?;
    let configuration: toml::Value =
        toml::from_str(configuration).map_err(|error| WorkflowError::Invalid(error.to_string()))?;
    let names: Vec<_> = configuration
        .get("test-groups")
        .and_then(toml::Value::as_table)
        .map(|groups| groups.keys().cloned().collect())
        .unwrap_or_default();
    let mut named: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
    for group in 0..listed.inventory.groups.len() {
        if !listed
            .inventory
            .selected_tests_iter()
            .any(|identity| identity.group == group)
        {
            continue;
        }
        for (ordinal, name) in names.iter().enumerate() {
            let mut command = reuse_listing(listed, group)?;
            command.args(["-E", &format!("group(={})", selection::equality(name))]);
            if listed.inventory.selection.filters().ignore_default_filter {
                command.arg("--ignore-default-filter");
            }
            let prefix = format!("group-policy-{group}-{ordinal}");
            report.expect_file(
                format!("{prefix}.stdout"),
                EvidenceRole::Stdout,
                Some(group),
                false,
            );
            report.expect_file(
                format!("{prefix}.stderr"),
                EvidenceRole::Stderr,
                Some(group),
                false,
            );
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
            if !capture.capture_complete
                || !report.causes.is_empty()
                || capture.child_exit != Some(0)
            {
                report.outcome = if capture.capture_complete && capture.child_exit.is_some() {
                    Outcome::Failed
                } else {
                    Outcome::Incomplete
                };
                return Err(WorkflowError::Invalid(
                    "native group-policy listing failed".into(),
                ));
            }
            let membership: TestListSummary =
                serde_json::from_slice(&fs::read(destination.join(format!("{prefix}.stdout")))?)
                    .map_err(discovery::DiscoveryError::from)?;
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
                named.entry(group).or_default().insert(name.clone());
            }
        }
    }
    Ok(named)
}

enum GroupObserver {
    Check(CompilerObserver),
    Test(CompletedObserver),
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
#[cfg(test)]
async fn execute_groups(
    input: &Input,
    destination: &Path,
    groups: Vec<(GroupCommand, GroupObserver)>,
    report: &mut Report,
) -> Result<(), WorkflowError> {
    execute_groups_with_policy(input, destination, groups, report, &BTreeMap::new()).await
}
#[cfg(test)]
async fn execute_groups_with_policy(
    input: &Input,
    destination: &Path,
    groups: Vec<(GroupCommand, GroupObserver)>,
    report: &mut Report,
    serialized: &BTreeMap<usize, BTreeSet<String>>,
) -> Result<(), WorkflowError> {
    execute_groups_observed(input, destination, groups, report, serialized, None).await
}
async fn execute_groups_observed(
    input: &Input,
    destination: &Path,
    groups: Vec<(GroupCommand, GroupObserver)>,
    report: &mut Report,
    serialized: &BTreeMap<usize, BTreeSet<String>>,
    progress: Option<&Arc<ProgressWriter>>,
) -> Result<(), WorkflowError> {
    if groups.is_empty() {
        report.outcome = Outcome::Incomplete;
        return Ok(());
    }
    let kind = if input.action.is_compiler() {
        PhaseKind::Check
    } else {
        PhaseKind::Test
    };
    if let Some(progress) = progress {
        progress.phase(LivePhase::ExecutingGroups);
    }
    let mut tasks = tokio::task::JoinSet::new();
    let mut predecessors = BTreeMap::new();
    for (request, observer) in groups {
        let group = request.group;
        let prefix = format!("run-{group}");
        let stdout = destination.join(format!("{prefix}.stdout"));
        let stderr = destination.join(format!("{prefix}.stderr"));
        report.expect_file(
            format!("{prefix}.stdout"),
            EvidenceRole::Stdout,
            Some(group),
            false,
        );
        report.expect_file(
            format!("{prefix}.stderr"),
            EvidenceRole::Stderr,
            Some(group),
            false,
        );
        let policy = input.policy;
        // Only contexts sharing a native named exclusion sequence. Each native runner still
        // schedules its own tests; unrelated names and global contexts retain concurrency.
        let mut wait = Vec::new();
        let mut done = Vec::new();
        for name in serialized.get(&group).into_iter().flatten() {
            let (sender, next) = tokio::sync::oneshot::channel::<()>();
            if let Some((previous_group, previous)) = predecessors.insert(name.clone(), (group, next)) {
                wait.push((name.clone(), previous_group, previous));
            }
            done.push(sender);
        }
        let progress = progress.cloned();
        tasks.spawn(async move {
            let started = std::time::Instant::now();
            let mut groups = BTreeMap::new();
            for (name, previous_group, mut previous) in wait {
                if matches!(previous.try_recv(), Err(tokio::sync::oneshot::error::TryRecvError::Empty)) {
                    if let Some(progress) = &progress {
                        progress.group(group, LiveGroup::WaitingForPredecessor {
                            native_group: name.clone(), predecessor: previous_group,
                        });
                    }
                    let _ = previous.await;
                }
                groups.insert(name, previous_group);
            }
            let predecessor_wait = (!groups.is_empty()).then(|| super::report::PredecessorWait {
                elapsed_seconds: started.elapsed().as_secs_f64(), groups,
            });
            if let Some(progress) = &progress {
                progress.group(group, LiveGroup::EnteredCommand);
            }
            let junit_report = request.junit_report.clone();
            let result = phase::run_capturing_junit(
                command(request), &stdout, &stderr, observer, policy, junit_report.as_deref(),
            ).await;
            drop(done);
            if let Some(progress) = &progress {
                progress.group(group, LiveGroup::CommandReturned);
            }
            (group, prefix, predecessor_wait, result)
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
            Ok((group, prefix, predecessor_wait, Err(error))) => {
                report.phases.push(Phase {
                    predecessor_wait,
                    native_stage: None,
                    elapsed_seconds: None,
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
            Ok((group, prefix, predecessor_wait, Ok(mut captured))) => {
                record_phase(report, kind, Some(group), &prefix, &mut captured);
                if let Some(phase) = report.phases.last_mut() {
                    phase.predecessor_wait = predecessor_wait;
                }
                if let Some(exit) = captured.child_exit.filter(|exit| *exit != 0) {
                    report.failure(WorkflowError::PhaseExit {
                        phase: input.action.command(),
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
    } else if input.action.is_compiler() {
        Outcome::CheckSucceeded
    } else {
        Outcome::TestsSucceeded
    };
    Ok(())
}

#[cfg(test)]
mod tests;
