// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{
    envelope::CliInvocationId,
    managed::payload::PayloadPolicy,
    test_workflow::execute::{Bounds, TerminalRecord},
};
use std::time::Duration;

fn input(action: Action) -> Input {
    Input {
        version: 1,
        invocation: CliInvocationId::new(),
        action,
        request: selection::Request {
            operation: if action == Action::Check {
                selection::Operation::Check
            } else {
                selection::Operation::Test
            },
            scope: selection::Scope::DefaultMembers {
                features: selection::Features::default(),
            },
            platform: None,
            filters: selection::Filters::default(),
            ignored: selection::IgnoredPolicy::Default,
        },
        profile: "local".into(),
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

#[test]
fn terminal_correlation_action_and_complete_capture_are_required() {
    for action in [Action::List, Action::Test, Action::Check] {
        assert_eq!(Action::from_command(action.command()), Some(action));
    }
    assert_eq!(Action::from_command("run"), None);
    let input = input(Action::Check);
    let root = tempfile::tempdir().unwrap();
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
    assert_eq!(terminal.report.bytes, raw.len() as u64);
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
async fn actual_workflow_runs_only_the_original_selected_pure_host_test() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let evidence = tempfile::tempdir().unwrap();
    let mut input = input(Action::Test);
    input.request.scope = selection::Scope::Packages(vec![selection::PackageIntent {
        package: "pse-host".into(),
        target: Some(selection::TargetIntent {
            name: "pse_host".into(),
            kind: Some("lib".into()),
        }),
        features: selection::Features {
            defaults: false,
            ..selection::Features::default()
        },
    }]);
    input.request.filters.words = vec![
        "discovery::tests::official_metadata_preserves_duplicate_names_and_ignored_selection"
            .into(),
    ];
    input.request.filters.ignore_default_filter = true;
    let input: Input = serde_json::from_slice(&serde_json::to_vec(&input).unwrap()).unwrap();
    assert!(input.request.filters.ignore_default_filter);
    let destination = evidence.path().join("workflow");
    let report = run(&input, root, &destination, std::env::vars_os().collect())
        .await
        .unwrap();
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
    let terminal = report.publish(&destination).unwrap();
    terminal.validate(&input.invocation, Action::Test).unwrap();
    assert_eq!(terminal.helper_exit(), 0);
}

#[tokio::test]
async fn actual_package_workflow_reconciles_selected_tests_beside_an_ignored_only_suite() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let evidence = tempfile::tempdir().unwrap();
    let mut input = input(Action::Test);
    input.request.scope = selection::Scope::Packages(vec![selection::PackageIntent {
        package: "pse-host".into(),
        target: None,
        features: selection::Features {
            defaults: false,
            ..selection::Features::default()
        },
    }]);
    input.request.filters.expressions = vec!["test(managed::workflow::tests::)".into()];
    input.request.filters.ignore_default_filter = true;
    let destination = evidence.path().join("workflow");
    let report = run(&input, root, &destination, std::env::vars_os().collect())
        .await
        .unwrap();
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
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let evidence = tempfile::tempdir().unwrap();
    let mut input = input(Action::Check);
    input.request.scope = selection::Scope::Packages(vec![selection::PackageIntent {
        package: "pse-ids".into(),
        target: Some(selection::TargetIntent {
            name: "pse_ids".into(),
            kind: Some("lib".into()),
        }),
        features: selection::Features {
            defaults: false,
            ..selection::Features::default()
        },
    }]);
    let destination = evidence.path().join("check");
    let report = run(&input, root, &destination, std::env::vars_os().collect())
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
    let terminal = report.publish(&destination).unwrap();
    assert_eq!(terminal.helper_exit(), 0);
    terminal.validate(&input.invocation, Action::Check).unwrap();
    input.action = Action::List;
    input.request.operation = selection::Operation::Test;
    input.request.filters.words = vec![format!("no-such-test-{}", input.invocation)];
    let destination = evidence.path().join("zero-list");
    let report = run(&input, root, &destination, std::env::vars_os().collect())
        .await
        .unwrap();
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
