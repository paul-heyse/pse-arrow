// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{
    envelope::CliInvocationId,
    managed::payload::PayloadPolicy,
    test_workflow::execute::{Bounds, TerminalRecord},
};
use nextest_metadata::{FilterMatch, MismatchReason, RustBinaryId, TestCaseName};
use serde_json::{Value, json};
use std::{os::unix::fs::PermissionsExt as _, time::Duration};

#[path = "native_fixture.rs"]
mod native_fixture;
use native_fixture::NativeFixture;

#[path = "rerun_tests.rs"]
mod rerun_tests;

fn input(action: Action) -> Input {
    Input {
        version: 1,
        invocation: CliInvocationId::new(),
        action,
        request: selection::Request {
            operation: match action {
                Action::Check => selection::Operation::Check,
                Action::Rustdoc => selection::Operation::Rustdoc,
                Action::List | Action::Test => selection::Operation::Test,
            },
            scope: selection::Scope::DefaultMembers {
                features: selection::Features::default(),
            },
            platform: None,
            filters: selection::Filters::default(),
            non_incremental: false,
            ignored: selection::IgnoredPolicy::Default,
        },
        profile: "local".into(),
        timings: false,
        policy: PayloadPolicy {
            startup: Duration::from_secs(30),
            write: Duration::from_secs(30),
            drain: Duration::from_secs(30),
        },
        bounds: Bounds {
            record_bytes: 1024 * 1024,
            grouping_capacity: 16,
            failure_records: 10,
        },
    }
}
fn shell(group: usize, root: &Path, script: &str, args: &[&Path]) -> GroupCommand {
    let mut arguments = vec!["-c".into(), script.into(), "workflow-fixture".into()];
    arguments.extend(args.iter().map(|path| path.as_os_str().to_owned()));
    GroupCommand {
        program: "/bin/sh".into(),
        arguments,
        cwd: root.into(),
        environment: std::env::vars_os().collect(),
        group,
        runtime_inputs: crate::discovery::RuntimeInputObservation::NotCapturedByDiscovery,
    }
}
fn observer() -> GroupObserver {
    GroupObserver::Check(CompilerObserver::new(
        NonZeroUsize::new(128).unwrap(),
        NonZeroUsize::new(16).unwrap(),
        10,
        RedactionPolicy::default(),
    ))
}

fn declaration(
    listed: &test_workflow::Listed,
    id: &str,
    ownership: Ownership,
    scope: Value,
    inputs: &[FixtureInput],
) -> Requirement {
    serde_json::from_value(json!({
        "group": 0,
        "unit": listed.inventory.groups[0].build.resolution.units[0],
        "declaration": {
            "id": id, "target": "case", "kind": "lib", "ownership": ownership,
            "scope": scope, "inputs": inputs,
        },
    }))
    .unwrap()
}

fn check_fixture_population(
    listed: &test_workflow::Listed,
    supplemental: Option<&TestListSummary>,
) -> Result<FixtureSelection, WorkflowError> {
    let suites = fixture_suites(&listed.inventory.groups[0], listed.requirements.iter())?;
    let mut selected = FixtureSelection::default();
    selected_fixture_requirements(
        0,
        listed.inventory.selected_tests_iter().count(),
        suites,
        supplemental,
        &mut selected,
    )?;
    Ok(selected)
}

#[tokio::test]
async fn fixture_scope_refuses_renamed_case_and_stale_module_but_accepts_filter_exclusion() {
    let (root, intent) = test_workflow::tests::intent();
    let mut listed = test_workflow::tests::listed(intent);
    let input = input(Action::Test);
    for (scope, missing) in [
        (json!({"tests": ["renamed_case"]}), "case \"renamed_case\""),
        (json!({"modules": ["fixture"]}), "module \"fixture\""),
    ] {
        let group = &mut listed.inventory.groups[0];
        let suite = group
            .listing
            .rust_suites
            .get_mut(&RustBinaryId::new("case"))
            .unwrap();
        let mut excluded = suite.test_cases[&TestCaseName::new("same_name")].clone();
        excluded.filter_match = FilterMatch::Mismatch {
            reason: MismatchReason::Expression,
        };
        suite
            .test_cases
            .insert(TestCaseName::new("fixture_sibling::exists"), excluded);
        group.listing.test_count = suite.test_cases.len();
        listed.requirements = vec![declaration(
            &listed,
            "scope-owner",
            Ownership::PreparationOnly,
            scope,
            &[FixtureInput::Server],
        )];
        let mut report = Report::new(&input);
        let result = fixture_requirements(&input, root.path(), &listed, &mut report).await;
        assert!(matches!(
            result,
            Err(WorkflowError::Fixture(FixtureError::Declaration(message))) if message.contains(missing)
        ));
        assert!(report.phases.is_empty());
    }
    listed.requirements = vec![declaration(
        &listed,
        "known-excluded",
        Ownership::PreparationOnly,
        json!({"tests": ["fixture_sibling::exists"], "modules": ["fixture_sibling"]}),
        &[FixtureInput::Server],
    )];
    let mut report = Report::new(&input);
    let selected = fixture_requirements(&input, root.path(), &listed, &mut report)
        .await
        .unwrap();
    assert!(selected.requirements.is_empty());
    assert!(report.phases.is_empty());
}

#[test]
fn fixture_scope_fallback_evidence_does_not_select_an_excluded_binary() {
    let (_root, intent) = test_workflow::tests::intent();
    let mut listed = test_workflow::tests::listed(intent);
    listed.requirements = vec![declaration(
        &listed,
        "conditionally-listed",
        Ownership::TargetOwned,
        json!({"tests": ["same_name"]}),
        &[FixtureInput::Server, FixtureInput::StudyModule],
    )];
    let original = &mut listed.inventory.groups[0].listing;
    let mut supplemental: TestListSummary =
        serde_json::from_value(serde_json::to_value(&*original).unwrap()).unwrap();
    let id = RustBinaryId::new("case");
    let suite = original.rust_suites.get_mut(&id).unwrap();
    suite.status = RustTestSuiteStatusSummary::SKIPPED_DEFAULT_FILTER;
    suite.test_cases.clear();
    original.test_count = 0;
    let missing = check_fixture_population(&listed, None);
    assert!(
        matches!(missing, Err(WorkflowError::Invalid(message)) if message.contains("omitted captured suite"))
    );
    let selected = check_fixture_population(&listed, Some(&supplemental)).unwrap();
    assert!(selected.requirements.is_empty());
    assert!(selected.target_owned.is_empty());

    supplemental
        .rust_suites
        .get_mut(&id)
        .unwrap()
        .test_cases
        .clear();
    supplemental.test_count = 0;
    assert!(matches!(
        check_fixture_population(&listed, Some(&supplemental)),
        Err(WorkflowError::Fixture(FixtureError::Declaration(message))) if message.contains("absent test case")
    ));
}

#[test]
fn fixture_scope_rejects_conflicting_owners_and_mixed_companion_selection() {
    let (_root, intent) = test_workflow::tests::intent();
    let mut listed = test_workflow::tests::listed(intent);
    let target = declaration(
        &listed,
        "target-owner",
        Ownership::TargetOwned,
        json!({"tests": ["same_name"]}),
        &[FixtureInput::Server, FixtureInput::StudyModule],
    );
    let borrowed = declaration(
        &listed,
        "borrowed-owner",
        Ownership::RunnerBorrowed,
        json!({}),
        &[FixtureInput::Server, FixtureInput::StudyModule],
    );
    listed.requirements = vec![target.clone()];
    let selected = check_fixture_population(&listed, None).unwrap();
    assert_eq!(selected.target_owned[&0].len(), 1);
    assert_eq!(
        selected.target_owned[&0].first().unwrap().test_name,
        "same_name"
    );
    listed.requirements.push(borrowed.clone());
    assert!(matches!(
        check_fixture_population(&listed, None),
        Err(WorkflowError::Fixture(FixtureError::Declaration(message))) if message.contains("disagree on performing ownership")
    ));

    let companion = declaration(
        &listed,
        "companion",
        Ownership::PreparationOnly,
        json!({"tests": ["same_name"]}),
        &[FixtureInput::QualificationStudyModule],
    );
    listed.requirements = vec![borrowed, companion];
    let selected = check_fixture_population(&listed, None).unwrap();
    assert_eq!(selected.requirements.len(), 2);
    assert!(selected.target_owned.is_empty());

    let group = &mut listed.inventory.groups[0];
    let suite = group
        .listing
        .rust_suites
        .get_mut(&RustBinaryId::new("case"))
        .unwrap();
    let original = suite.test_cases[&TestCaseName::new("same_name")].clone();
    suite
        .test_cases
        .insert(TestCaseName::new("ordinary_case"), original);
    group.listing.test_count = 2;
    assert!(matches!(
        check_fixture_population(&listed, None),
        Err(WorkflowError::Fixture(FixtureError::Declaration(message))) if message.contains("among 2 selected cases")
    ));
}

#[test]
fn exact_fixture_refusal_records_complete_pending_account_before_helpers() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("must-not-be-created");
    let requirement: Requirement = serde_json::from_value(json!({
        "group": 2,
        "unit": {
            "package_id": "original-package", "package_name": "example",
            "target_name": "original", "target_kinds": ["lib"],
            "source_path": "/original/lib.rs", "features": [], "platform": null,
            "mode": "test", "root": true
        },
        "declaration": {
            "id": "selected-original", "target": "original", "kind": "lib",
            "ownership": "target-owned", "scope": { "tests": ["selected_case"] },
            "inputs": ["trace-processor", "native-lane", "scientific-worker", "engineering-endpoint"]
        }
    }))
    .unwrap();
    let mut report = Report::new(&input(Action::Test));
    report.selected_tests = Some(1);
    let result = prepare_fixtures(
        &mut report,
        root.path(),
        &Environment::new(),
        &destination,
        &[requirement],
        None,
    )
    .unwrap();
    assert!(result.is_none());
    assert_eq!(report.outcome, Outcome::Pending);
    assert_eq!(report.selected_tests, Some(1));
    assert_eq!(report.pending.len(), 1);
    let Pending::Fixture(pending) = &report.pending[0] else {
        panic!("original fixture account missing");
    };
    assert_eq!(pending.requirement.group(), 2);
    assert_eq!(pending.readiness.inputs.len(), 4);
    assert!(matches!(
        pending.readiness.inputs[&crate::fixtures::Input::ScientificWorker],
        crate::fixtures::InputState::OwnerAdmissionRequired { .. }
    ));
    assert_eq!(report.failures.len(), 1);
    let WorkflowError::Fixture(crate::fixtures::FixtureError::Inputs(failures)) = &report.causes[0]
    else {
        panic!("typed original failures missing");
    };
    assert_eq!(failures.len(), 3);
    assert!(report.phases.is_empty());
    assert!(!destination.exists());
}

#[test]
fn failed_discovery_attempt_preserves_source_phases_without_inventory_or_rerun_grant() {
    let root = tempfile::tempdir().unwrap();
    let mut report = Report::new(&input(Action::Test));
    let phase = Phase {
        native_stage: Some("build".into()),
        elapsed_seconds: Some(0.25),
        kind: PhaseKind::Discovery,
        group: Some(2),
        stdout: "group-2/build.stdout".into(),
        stderr: "group-2/build.stderr".into(),
        child_exit: Some(17),
        capture_complete: true,
        stdout_capture: Some(crate::managed::result::StreamSummary {
            bytes_read: 12,
            bytes_retained: 12,
            eof: true,
            complete: true,
            io_unresolved: false,
        }),
        stderr_capture: Some(crate::managed::result::StreamSummary {
            bytes_read: 19,
            bytes_retained: 19,
            eof: true,
            complete: true,
            io_unresolved: false,
        }),
    };
    let error = crate::discovery::DiscoveryError::Attempt {
        evidence: root.path().join("inventory"),
        phases: vec![phase],
        error: Box::new(crate::discovery::DiscoveryError::Command {
            stage: "build",
            exit: 17,
            evidence: root.path().join("inventory/group-2"),
        }),
    };
    failed(&mut report, error.into(), root.path());
    assert_eq!(report.phases.len(), 1);
    assert_eq!(report.phases[0].stdout, "inventory/group-2/build.stdout");
    assert_eq!(report.phases[0].child_exit, Some(17));
    assert_eq!(
        report.phases[0]
            .stdout_capture
            .as_ref()
            .unwrap()
            .bytes_retained,
        12
    );
    assert!(report.required_capture_complete);
    assert_eq!(report.outcome, Outcome::Failed);
    assert!(report.inventory.is_none());
    assert!(report.inventory_content.is_none());
    assert!(report.selected_tests.is_none());
    assert_eq!(report.causes.len(), 1);
    let WorkflowError::Discovery(crate::discovery::DiscoveryError::Attempt { phases, .. }) =
        &report.causes[0]
    else {
        panic!("original native cause")
    };
    assert_eq!(
        phases[0].stdout, "group-2/build.stdout",
        "original account remains relative to its own source root"
    );
}

#[tokio::test]
async fn simultaneous_native_producers_publish_distinct_private_completed_reports() {
    let fixture = NativeFixture::new();
    let checkout = &fixture.root;
    let mut first = input(Action::Test);
    first.profile = "default".into();
    first.request.scope = NativeFixture::scope(true);
    first.request.filters.expressions = vec!["test(=ordinary_case)".into()];
    first.request.filters.ignore_default_filter = true;
    let mut second = first.clone();
    second.invocation = CliInvocationId::new();
    let a = fixture.scratch.path().join("first");
    let b = fixture.scratch.path().join("second");
    let (left, right) = tokio::join!(
        run(&first, checkout, &a, fixture.environment.clone()),
        run(&second, checkout, &b, fixture.environment),
    );
    let left = left.unwrap();
    let right = right.unwrap();
    for report in [&left, &right] {
        assert_eq!(
            report.outcome,
            Outcome::TestsSucceeded,
            "{:?}",
            report.failures
        );
        assert_eq!(report.selected_tests, Some(1));
        assert!(report.required_capture_complete);
        assert!(report.phases.iter().all(|phase| {
            phase
                .elapsed_seconds
                .is_some_and(|elapsed| elapsed.is_finite() && elapsed >= 0.0)
        }));
        let stages: Vec<_> = report
            .phases
            .iter()
            .filter_map(|phase| phase.native_stage.as_deref())
            .collect();
        assert_eq!(stages, ["version", "build", "list"]);
    }
    let native = |report: &Report| {
        let GroupObservation::Test { observation, .. } = &report.observations[0] else {
            panic!("native observation")
        };
        assert_eq!(observation.summary.completed(), 1);
        assert!(observation.summary.complete());
        (
            observation.report.clone(),
            quick_junit::Report::deserialize(std::io::BufReader::new(
                fs::File::open(&observation.report).unwrap(),
            ))
            .unwrap(),
        )
    };
    let (left_path, left_native) = native(&left);
    let (right_path, right_native) = native(&right);
    assert_ne!(left_path, right_path);
    assert!(left_path.starts_with(&a));
    assert!(right_path.starts_with(&b));
    assert!(left_native.uuid.is_some());
    assert_ne!(left_native.uuid, right_native.uuid);
}

#[test]
fn terminal_correlation_action_and_complete_capture_are_required() {
    for action in [Action::List, Action::Test, Action::Check] {
        assert_eq!(Action::from_command(action.command()), Some(action));
    }
    assert_eq!(Action::from_command("run"), None);
    let input = input(Action::Check);
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let mut report = Report::new(&input);
    report.outcome = Outcome::CheckSucceeded;
    let terminal = report.publish(root.path()).unwrap();
    terminal.validate(&input.invocation, Action::Check).unwrap();
    assert_eq!(terminal.helper_exit(), 0);
    let mut wrong = terminal.clone();
    wrong.outcome = Outcome::TestsSucceeded;
    assert!(wrong.validate(&input.invocation, Action::Check).is_err());
    assert_eq!(wrong.helper_exit(), 125);
    let mut lost = terminal.clone();
    lost.required_capture_complete = false;
    assert!(lost.validate(&input.invocation, Action::Check).is_err());
    assert_eq!(lost.helper_exit(), 125);
    for (outcome, exit) in [
        (Outcome::Failed, 1),
        (Outcome::ZeroMatch, 1),
        (Outcome::Pending, 125),
        (Outcome::Incomplete, 125),
    ] {
        let mut terminal = terminal.clone();
        terminal.outcome = outcome;
        terminal.validate(&input.invocation, Action::Check).unwrap();
        assert_eq!(terminal.helper_exit(), exit);
    }
    assert!(
        terminal
            .validate(&CliInvocationId::new(), Action::Check)
            .is_err()
    );
    let mut foreign = serde_json::to_value(&terminal).unwrap();
    foreign["extra"] = true.into();
    assert!(serde_json::from_value::<TerminalRecord>(foreign).is_err());
    assert!(
        report.publish(root.path()).is_err(),
        "durable original report cannot be replaced"
    );
    let mut frame = Vec::new();
    terminal.write(&mut frame).unwrap();
    assert_eq!(frame.iter().filter(|byte| **byte == b'\n').count(), 1);
    let raw = fs::read(root.path().join("report.json")).unwrap();
    assert_eq!(terminal.report.byte_length, raw.len() as u64);
    use sha2::Digest as _;
    assert_eq!(
        terminal.report.sha256,
        sha2::Sha256::digest(&raw)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
}
#[tokio::test]
async fn independent_compiler_groups_run_concurrently_and_reconcile_native_records() {
    let root = tempfile::tempdir().unwrap();
    let input = input(Action::Check);
    let mut report = Report::new(&input);
    let a = root.path().join("a");
    let b = root.path().join("b");
    // A serial implementation cannot pass either group's bounded peer handshake.
    let script = "touch \"$1\"; n=0; while [ ! -e \"$2\" ]; do n=$((n+1)); [ \"$n\" -lt 50 ] || exit 75; sleep 0.02; done; printf '%s\\n' '{\"reason\":\"build-finished\",\"success\":true}'";
    execute_groups(
        &input,
        root.path(),
        vec![
            (shell(0, root.path(), script, &[&a, &b]), observer()),
            (shell(1, root.path(), script, &[&b, &a]), observer()),
        ],
        &mut report,
    )
    .await
    .unwrap();
    assert_eq!(report.outcome, Outcome::CheckSucceeded);
    assert!(report.required_capture_complete);
    assert!(report.failures.is_empty());
    assert_eq!(report.observations.len(), 2);
    for phase in &report.phases {
        assert!(phase.capture_complete);
        assert_eq!(phase.child_exit, Some(0));
        assert!(
            fs::read(root.path().join(&phase.stdout))
                .unwrap()
                .ends_with(b"\n")
        );
    }
}
#[tokio::test]
async fn native_named_exclusions_sequence_shared_contexts_without_serializing_other_names() {
    let root = tempfile::tempdir().unwrap();
    let input = input(Action::Check);
    let mut report = Report::new(&input);
    let a = root.path().join("a");
    let b = root.path().join("b");
    let settled = root.path().join("settled");
    let first = "touch \"$1\"; n=0; while [ ! -e \"$2\" ]; do n=$((n+1)); [ \"$n\" -lt 100 ] || exit 75; sleep 0.01; done; touch \"$3\"; printf '%s\\n' '{\"reason\":\"build-finished\",\"success\":true}'";
    let second = "[ -e \"$1\" ] || exit 76; printf '%s\\n' '{\"reason\":\"build-finished\",\"success\":true}'";
    execute_groups_with_policy(
        &input,
        root.path(),
        vec![
            (
                shell(0, root.path(), first, &[&a, &b, &settled]),
                observer(),
            ),
            (shell(1, root.path(), second, &[&settled]), observer()),
            (
                shell(2, root.path(), first, &[&b, &a, &settled]),
                observer(),
            ),
        ],
        &mut report,
        &BTreeMap::from([
            (0, BTreeSet::from(["scientific".into()])),
            (1, BTreeSet::from(["scientific".into()])),
            (2, BTreeSet::from(["store".into()])),
        ]),
    )
    .await
    .unwrap();
    assert_eq!(
        report.outcome,
        Outcome::CheckSucceeded,
        "{:?}",
        report.failures
    );
    assert_eq!(report.observations.len(), 3);
}

#[tokio::test]
async fn malformed_required_records_never_promote_child_zero_and_raw_tail_is_retained() {
    let root = tempfile::tempdir().unwrap();
    let input = input(Action::Check);
    let mut report = Report::new(&input);
    let script = "printf 'invalid-required-json\\n'; n=0; while [ \"$n\" -lt 1000 ]; do printf 'raw-tail-stdout\\n'; printf 'raw-tail-stderr\\n' >&2; n=$((n+1)); done; printf '%s\\n' '{\"reason\":\"build-finished\",\"success\":true}'";
    execute_groups(
        &input,
        root.path(),
        vec![(shell(0, root.path(), script, &[]), observer())],
        &mut report,
    )
    .await
    .unwrap();
    assert_eq!(report.outcome, Outcome::Incomplete);
    assert!(report.required_capture_complete);
    assert_eq!(report.phases[0].child_exit, Some(0));
    assert!(!report.failures.is_empty());
    let stdout = fs::read(root.path().join("run-0.stdout")).unwrap();
    assert!(stdout.ends_with(b"{\"reason\":\"build-finished\",\"success\":true}\n"));
    assert_eq!(
        fs::read(root.path().join("run-0.stderr")).unwrap().len(),
        "raw-tail-stderr\n".len() * 1000
    );
}
#[tokio::test]
async fn all_groups_settle_and_keep_original_exit_when_an_independent_group_fails() {
    let root = tempfile::tempdir().unwrap();
    let input = input(Action::Check);
    let mut report = Report::new(&input);
    let failure = "printf '%s\\n' '{\"reason\":\"build-finished\",\"success\":false}'; exit 17";
    let success = "sleep 0.02; printf '%s\\n' '{\"reason\":\"build-finished\",\"success\":true}'";
    execute_groups(
        &input,
        root.path(),
        vec![
            (shell(0, root.path(), failure, &[]), observer()),
            (shell(1, root.path(), success, &[]), observer()),
        ],
        &mut report,
    )
    .await
    .unwrap();
    assert_eq!(report.outcome, Outcome::Failed);
    assert_eq!(report.observations.len(), 2);
    assert_eq!(report.phases[0].child_exit, Some(17));
    assert_eq!(report.phases[1].child_exit, Some(0));
    assert!(report.required_capture_complete);
}
#[tokio::test]
async fn unsupported_input_retains_a_failed_report_without_metadata_or_build() {
    let root = tempfile::tempdir().unwrap();
    let mut input = input(Action::Test);
    input.version = 99;
    let report = run(
        &input,
        root.path(),
        &root.path().join("evidence"),
        Environment::new(),
    )
    .await
    .unwrap();
    assert_eq!(report.outcome, Outcome::Failed);
    assert_eq!(report.failures.len(), 1);
    assert!(report.phases.is_empty());
    assert!(!root.path().join("evidence/metadata.stdout").exists());
}
#[tokio::test]
async fn actual_workflow_runs_only_the_original_selected_pure_test() {
    let NativeFixture {
        scratch: evidence,
        root,
        environment,
    } = NativeFixture::new();
    let mut input = input(Action::Test);
    input.request.scope = NativeFixture::scope(true);
    input.request.filters.words = vec!["ordinary_case".into()];
    input.request.filters.ignore_default_filter = true;
    let input: Input = serde_json::from_slice(&serde_json::to_vec(&input).unwrap()).unwrap();
    assert!(input.request.filters.ignore_default_filter);
    let destination = evidence.path().join("workflow");
    let mut report = run(&input, &root, &destination, environment).await.unwrap();
    if report.outcome != Outcome::TestsSucceeded {
        let terminal = report.publish(&destination).unwrap();
        let evidence = evidence.keep();
        panic!(
            "actual workflow {:?}, selected {:?}, capture {}, failures {:?}; original report {} ({}) retained at {}",
            report.outcome,
            report.selected_tests,
            report.required_capture_complete,
            report.failures,
            terminal.report.path,
            terminal.report.sha256,
            evidence.display()
        );
    }
    assert_eq!(report.selected_tests, Some(1));
    assert!(report.required_capture_complete);
    assert_eq!(report.observations.len(), 1);
    let inventory: crate::discovery::Inventory =
        serde_json::from_slice(&fs::read(destination.join("inventory/inventory.json")).unwrap())
            .unwrap();
    assert!(inventory.selection.filters().ignore_default_filter);
    verify_current_rerun_context(&inventory, evidence.path());
    let terminal = report.publish(&destination).unwrap();
    terminal.validate(&input.invocation, Action::Test).unwrap();
    assert_eq!(terminal.helper_exit(), 0);
    // Exercise the real source-built population; no second runner, inventory or verdict.
    let report_original = fs::read(destination.join("report.json")).unwrap();
    let inventory_original = fs::read(destination.join("inventory/inventory.json")).unwrap();
    let captured = crate::test_inventory::CapturedEvidence {
        id: input.invocation.clone(),
        report: report_original,
        observation: super::super::ReportObservation::of(&report),
        terminal,
        inventory_sha256: crate::source_read::source_digest(
            std::str::from_utf8(&inventory_original).unwrap(),
        ),
        inventory_bytes: inventory_original,
        inventory,
        manager_settlement: None,
        native_lifetime_pending: None,
    };
    let wire =
        crate::developer_evidence_contracts::encode_captured_workflow_evidence(&captured).unwrap();
    let descriptors = <buffa_descriptor::generated::descriptor::FileDescriptorSet as buffa::Message>::decode_from_slice(crate::developer_evidence_contracts::DESCRIPTORS).unwrap();
    let read = |wire: &crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::CapturedWorkflowEvidence| {
        crate::developer_evidence::inspect(&descriptors, "pse.developer.evidence.v1.CapturedWorkflowEvidence", &buffa::Message::encode_to_vec(wire), &crate::developer_evidence::binary_options())
    };
    let readback = read(&wire).unwrap();
    assert_eq!(readback.selected_tests, Some(1));
    assert_eq!(readback.selected_population_complete, Some(true));
    assert_eq!(readback.report_version, 3);
    assert_eq!(readback.scope, super::super::EvidenceScope::Selection);
    assert!(
        readback.recorded_manager_settlement.is_none(),
        "this direct native test has no recorded manager owner"
    );
    let mut changed = wire.clone();
    changed.selected_tests.clear();
    assert!(read(&changed).is_err());
    let mut changed = wire.clone();
    changed.selected_population_complete = Some(false);
    assert!(read(&changed).is_err());
    let mut changed = wire.clone();
    let marker = b"\"nextest_version\":\"";
    let offset = changed
        .inventory_original
        .windows(marker.len())
        .position(|part| part == marker)
        .unwrap()
        + marker.len();
    changed.inventory_original[offset] = if changed.inventory_original[offset] == b'x' {
        b'y'
    } else {
        b'x'
    };
    assert_eq!(
        changed.inventory_original.len(),
        wire.inventory_original.len()
    );
    changed.inventory_sha256 = crate::source_read::source_digest(
        std::str::from_utf8(&changed.inventory_original).unwrap(),
    );
    assert!(
        read(&changed).is_err(),
        "a new observed digest cannot replace the original report commitment"
    );
    let mut changed = wire;
    changed.inventory_original.clear();
    assert!(read(&changed).is_err());
}

/// Challenge current rerun eligibility using the actual native build/list population.
/// These altered observations are never written over the committed original inventory.
fn verify_current_rerun_context(original: &crate::discovery::Inventory, scratch: &Path) {
    let identity = original.selected_tests().pop().unwrap();
    let selected = original.test(&identity).unwrap();
    let root = original.selection.groups()[identity.group]
        .roots()
        .iter()
        .find(|root| root.package_id() == identity.package_id)
        .unwrap();
    let case = selection::RerunCase {
        package: root.package_name().into(),
        target: selection::TargetIntent {
            name: selected.artifact.cargo.target.name.to_string(),
            kind: Some(selected.artifact.cargo.target.kind[0].to_string()),
        },
        features: selection::Features {
            defaults: false,
            all: false,
            enabled: selected.artifact.cargo.features.iter().cloned().collect(),
            absent: root.features().absent.clone(),
        },
        binary_id: identity.binary_id.clone(),
        test_name: identity.test_name.clone(),
    };
    let fresh = || {
        let mut current: crate::discovery::Inventory =
            serde_json::from_slice(&serde_json::to_vec(original).unwrap()).unwrap();
        current.selection = current.selection.source(
            selection::SourceBasis::RerunFailed {
                invocation: CliInvocationId::new(),
            },
            vec![case.clone()],
        );
        current
    };
    test_workflow::source::verify_rerun(&fresh()).unwrap();
    let mut renamed = serde_json::to_value(fresh()).unwrap();
    let roots = renamed["selection"]["groups"][identity.group]["roots"]
        .as_array_mut()
        .unwrap();
    let root = roots
        .iter_mut()
        .find(|root| root["package_id"] == identity.package_id)
        .unwrap();
    root["package_name"] = "absent-current-package".into();
    let renamed = serde_json::from_value(renamed).unwrap();
    assert!(test_workflow::source::verify_rerun(&renamed).is_err());
    let mut changed_features = fresh();
    changed_features.groups[identity.group]
        .build
        .artifacts
        .iter_mut()
        .find(|artifact| artifact.executable.captured == selected.artifact.executable.captured)
        .unwrap()
        .cargo
        .features
        .push("unselected-current-feature".into());
    assert!(test_workflow::source::verify_rerun(&changed_features).is_err());
    let mut missing_test = fresh();
    missing_test.groups[identity.group]
        .listing
        .rust_suites
        .get_mut(&RustBinaryId::new(&identity.binary_id))
        .unwrap()
        .test_cases
        .remove(&TestCaseName::new(&identity.test_name));
    assert!(test_workflow::source::verify_rerun(&missing_test).is_err());
    let mut stale_binary = fresh();
    let artifact = stale_binary.groups[identity.group]
        .build
        .artifacts
        .iter_mut()
        .find(|artifact| artifact.executable.captured == selected.artifact.executable.captured)
        .unwrap();
    let isolated = scratch.join("stale-current-binary");
    fs::copy(&artifact.executable.captured, &isolated).unwrap();
    fs::set_permissions(&isolated, fs::Permissions::from_mode(0o700)).unwrap();
    use std::io::Write as _;
    fs::OpenOptions::new()
        .append(true)
        .open(&isolated)
        .unwrap()
        .write_all(b"changed current executable")
        .unwrap();
    artifact.executable.captured = isolated.clone();
    stale_binary.groups[identity.group]
        .listing
        .rust_suites
        .get_mut(&RustBinaryId::new(&identity.binary_id))
        .unwrap()
        .binary
        .binary_path = isolated.to_str().unwrap().into();
    assert!(test_workflow::source::verify_rerun(&stale_binary).is_err());
    assert_eq!(original.selected_tests(), vec![identity]);
    original.verify_executables().unwrap();
}

#[tokio::test]
async fn actual_package_workflow_reconciles_selected_tests_beside_an_ignored_only_suite() {
    let NativeFixture {
        scratch: evidence,
        root,
        environment,
    } = NativeFixture::new();
    let mut input = input(Action::Test);
    input.request.scope = NativeFixture::scope(false);
    input.request.filters.expressions =
        vec!["test(=ordinary_case) | test(=expected_refusal)".into()];
    input.request.filters.ignore_default_filter = true;
    let destination = evidence.path().join("workflow");
    let mut report = run(&input, &root, &destination, environment).await.unwrap();
    if report.outcome != Outcome::TestsSucceeded {
        report.publish(&destination).unwrap();
        let evidence = evidence.keep();
        panic!(
            "package-wide workflow {:?}, selected {:?}, failures {:?}; original evidence {}",
            report.outcome,
            report.selected_tests,
            report.failures,
            evidence.display()
        );
    }
    assert_eq!(report.selected_tests, Some(2));
    assert!(report.required_capture_complete);
    let GroupObservation::Test { observation, .. } = &report.observations[0] else {
        panic!("test observation")
    };
    assert_eq!(observation.summary.completed(), 2);
    assert!(observation.summary.complete());
    assert_eq!(
        observation.summary.setup(),
        crate::extraction::NextestSetup::SucceededByRunner
    );
    let inventory: crate::discovery::Inventory =
        serde_json::from_slice(&fs::read(destination.join("inventory/inventory.json")).unwrap())
            .unwrap();
    assert!(
        inventory.groups[0]
            .listing
            .rust_suites
            .values()
            .any(|suite| suite.binary.binary_name == "native"
                && suite
                    .test_cases
                    .values()
                    .any(|case| case.ignored && !case.filter_match.is_match())),
        "actual package-wide capture includes the original ignored-only integration suite"
    );
    assert_eq!(report.publish(&destination).unwrap().helper_exit(), 0);
}

#[tokio::test]
async fn actual_check_and_zero_match_listing_keep_distinct_native_outcomes() {
    let NativeFixture {
        scratch: evidence,
        root,
        environment,
    } = NativeFixture::new();
    let mut input = input(Action::Check);
    input.request.scope = NativeFixture::scope(true);
    let destination = evidence.path().join("check");
    let mut report = run(&input, &root, &destination, environment.clone())
        .await
        .unwrap();
    if report.outcome != Outcome::CheckSucceeded {
        report.publish(&destination).unwrap();
        let evidence = evidence.keep();
        panic!(
            "actual check {:?}, failures {:?}; original evidence {}",
            report.outcome,
            report.failures,
            evidence.display()
        );
    }
    assert!(report.selected_tests.is_none());
    assert!(report.inventory.is_none());
    assert_eq!(report.observations.len(), 1);
    assert!(report.required_capture_complete);
    let metadata: cargo_metadata::Metadata =
        serde_json::from_slice(&fs::read(destination.join("metadata.stdout")).unwrap()).unwrap();
    assert_eq!(
        metadata.target_directory.as_std_path(),
        root.join("artifact")
    );
    assert_eq!(
        metadata.build_directory.unwrap().as_std_path(),
        root.join("intermediate")
    );
    let terminal = report.publish(&destination).unwrap();
    assert_eq!(terminal.helper_exit(), 0);
    terminal.validate(&input.invocation, Action::Check).unwrap();
    input.action = Action::List;
    input.request.operation = selection::Operation::Test;
    input.request.filters.words = vec![format!("no-such-test-{}", input.invocation)];
    let destination = evidence.path().join("zero-list");
    let mut report = run(&input, &root, &destination, environment).await.unwrap();
    if report.outcome != Outcome::ZeroMatch {
        report.publish(&destination).unwrap();
        let evidence = evidence.keep();
        panic!(
            "actual zero listing {:?}, failures {:?}; original evidence {}",
            report.outcome,
            report.failures,
            evidence.display()
        );
    }
    assert_eq!(report.selected_tests, Some(0));
    assert!(report.observations.is_empty());
    assert!(report.required_capture_complete);
    assert_eq!(report.publish(&destination).unwrap().helper_exit(), 1);
}

#[tokio::test]
async fn actual_list_and_rustdoc_preserve_population_and_compiler_meanings() {
    let NativeFixture {
        scratch: evidence,
        root: checkout,
        environment,
    } = NativeFixture::new();
    let mut input = input(Action::List);
    input.request.scope = NativeFixture::scope(true);
    let destination = evidence.path().join("list");
    let mut listed = run(&input, &checkout, &destination, environment.clone())
        .await
        .unwrap();
    assert_eq!(listed.outcome, Outcome::Listed, "{:?}", listed.failures);
    assert!(listed.selected_tests.is_some_and(|count| count > 0));
    assert!(listed.inventory.is_some());
    assert!(
        listed.observations.is_empty(),
        "listing executes no assertions"
    );
    assert!(listed.required_capture_complete);
    let terminal = listed.publish(&destination).unwrap();
    terminal.validate(&input.invocation, Action::List).unwrap();
    assert_eq!(terminal.helper_exit(), 0);

    input.invocation = CliInvocationId::new();
    input.action = Action::Rustdoc;
    input.request.operation = selection::Operation::Rustdoc;
    let destination = evidence.path().join("rustdoc");
    let mut documented = run(&input, &checkout, &destination, environment)
        .await
        .unwrap();
    assert_eq!(
        documented.outcome,
        Outcome::CheckSucceeded,
        "{:?}",
        documented.failures
    );
    assert!(documented.selected_tests.is_none());
    assert!(documented.inventory.is_none());
    assert!(documented.required_capture_complete);
    assert_eq!(documented.observations.len(), 1);
    assert!(matches!(
        documented.observations[0],
        GroupObservation::Check { .. }
    ));
    let terminal = documented.publish(&destination).unwrap();
    terminal
        .validate(&input.invocation, Action::Rustdoc)
        .unwrap();
    assert_eq!(terminal.helper_exit(), 0);
    assert!(terminal.validate(&input.invocation, Action::Check).is_err());
}
