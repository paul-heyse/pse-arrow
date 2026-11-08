// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual original assertion failure becomes fresh intent without retaining old executables.
use super::*;
use crate::{
    envelope::{Envelope, Settlement},
    invocation::{InvocationEvidence, InvocationSpec, InvocationStore},
    managed::result::{ManagedSummary, WorkflowSummary},
    test_workflow::execute::ContentIdentity,
};

fn begin(store: &InvocationStore, input: &Input) -> PathBuf {
    let (binding, encoded) = crate::managed::workflow::encode_input(input).unwrap();
    let directory = store
        .begin(InvocationSpec {
            invocation_id: input.invocation.clone(),
            command: input.action.command().into(),
            session: None,
            parent: None,
            operation: serde_json::json!({"workflow": binding}),
            // This ledger fixture has no manager capability. Actual source build/list/test
            // observations below come from the native performer and retain their own identities.
            control: crate::generation::PreparedGeneration {
                generation: "generation.rerun-intent-observation".into(),
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
    directory.path().join("workflow")
}

fn publish(store: &InvocationStore, input: &Input, destination: &Path, report: &mut Report) {
    report.input_cutoff = Some(Ok(ContentIdentity::of(
        &fs::read(destination.parent().unwrap().join("workflow-input.json")).unwrap(),
    )));
    let terminal = report.publish(destination).unwrap();
    let helper_exit = terminal.helper_exit();
    let mut manager = Envelope::response(
        input.action.command(),
        "original native performer observations",
        ManagedSummary {
            bundle: None,
            workflow: Some(WorkflowSummary {
                terminal,
                report: "workflow/report.json".into(),
            }),
            native_lifetime_pending: None,
            child_exit: Some(helper_exit),
            stdout: None,
            stderr: None,
            borrowed_reservation: false,
            service_result: None,
        },
    );
    manager.invocation_id = input.invocation.clone();
    // A completed native test observation is not a claim of manager-level physical settlement.
    manager.settlement = Settlement::Unknown;
    crate::invocation::publish_bytes(
        &store
            .directory(&input.invocation)
            .unwrap()
            .path()
            .join("result.json"),
        &serde_json::to_vec(&manager).unwrap(),
        None,
        0o400,
    )
    .unwrap();
}

#[tokio::test]
async fn actual_native_setup_failure_preserves_original_capture_without_assertion_rerun_grant() {
    let NativeFixture { scratch, root, environment } = NativeFixture::new();
    let root = root.as_path();
    let store =
        InvocationStore::new(crate::paths::CheckoutState::resolve(root, &environment).unwrap());
    let mut input = input(Action::Test);
    input.request.scope = NativeFixture::scope(true);
    input.request.filters.ignore_default_filter = true;
    input.request.filters.words = vec!["rerun_controlled_failure_probe".into()];
    let destination = begin(&store, &input);
    fs::create_dir(&destination).unwrap();
    fs::set_permissions(&destination, fs::Permissions::from_mode(0o700)).unwrap();
    let cargo = crate::correctness::real_cargo_in(&environment).unwrap();
    let metadata = Command::new(&cargo)
        .args([
            "metadata",
            "--format-version=1",
            "--locked",
            "--offline",
            "--no-deps",
        ])
        .current_dir(root)
        .env_clear()
        .envs(&environment)
        .output()
        .unwrap();
    assert!(
        metadata.status.success(),
        "{}",
        String::from_utf8_lossy(&metadata.stderr)
    );
    let metadata: cargo_metadata::Metadata = serde_json::from_slice(&metadata.stdout).unwrap();
    let admitted = test_workflow::Intent::new(
        metadata.clone(),
        input.request.clone(),
        input.profile.clone(),
        environment.clone(),
    )
    .unwrap()
    .admit()
    .unwrap();
    let mut report = Report::new(&input);
    report.selection = Some(admitted.resolved().plan().clone());
    report.admission = admitted.resolved().admitted().to_vec();
    let listed = admitted
        .list(
            &nextest(&environment).unwrap(),
            &destination.join("inventory"),
            crate::discovery::DiscoveryOptions {
                policy: input.policy,
                timings: false,
            },
        )
        .await
        .unwrap();
    report.record_inventory(&destination).unwrap();
    report.selected_tests = Some(listed.inventory().selected_tests_iter().count());
    discovery_phases(&mut report, listed.inventory());
    let mut command = listed.command(0).unwrap();
    assert_eq!(command.program, listed.inventory().nextest.captured);
    assert_eq!(
        report.selection.as_ref().unwrap(),
        &listed.inventory().selection
    );
    // Deliberate runtime-input fault in this caller-owned command only. Discovery's
    // captured correctness adapter and all current image/admission checks stay intact.
    command
        .environment
        .insert("PSE_CORRECTNESS_REAL_CARGO".into(), cargo);
    command
        .environment
        .insert("PSE_CORRECTNESS_EXECUTABLE".into(), "/bin/false".into());
    let observer = listed.observer(0).unwrap();
    execute_groups(
        &input,
        &destination,
        vec![(command, GroupObserver::Test(observer))],
        &mut report,
    )
    .await
    .unwrap();
    if report.outcome != Outcome::Failed {
        let retained = scratch.keep();
        panic!(
            "expected native setup failure, got {:?}; failures {:?}; original evidence retained at {}",
            report.outcome,
            report.failures,
            retained.display()
        );
    }
    assert!(
        report.required_capture_complete,
        "all original raw output was retained"
    );
    assert_eq!(report.selected_tests, Some(1));
    let GroupObservation::Test { observation, .. } = &report.observations[0] else {
        panic!("native test observation absent")
    };
    assert_eq!(observation.summary.outcome(), NextestOutcome::SetupFailed);
    assert_eq!(
        observation.summary.setup(),
        crate::extraction::NextestSetup::FailedByRunner
    );
    assert!(
        !observation.summary.complete(),
        "setup failure does not establish selected-test completion"
    );
    assert_eq!(observation.summary.completed(), 0);
    assert!(observation.summary.failed().is_empty());
    assert!(
        observation.failures.is_empty(),
        "setup is not a fabricated assertion identity"
    );
    assert!(
        fs::metadata(destination.join("run-0.stderr"))
            .unwrap()
            .len()
            > 0
    );
    publish(&store, &input, &destination, &mut report);
    let captured =
        crate::test_inventory::captured(&store, input.invocation.as_str(), None).unwrap();
    assert_eq!(captured.observation.outcome, Outcome::Failed);
    assert!(captured.observation.required_capture_complete);
    assert_eq!(captured.manager_settlement, Some(Settlement::Unknown));
    let mut rerun = input.request.clone();
    rerun.scope = selection::Scope::RerunFailed {
        invocation: input.invocation.to_string(),
    };
    assert!(
        test_workflow::Intent::new(metadata, rerun, input.profile.clone(), environment).is_err()
    );
}

#[tokio::test]
async fn actual_expected_refusal_success_has_original_evidence_and_zero_failed_rerun_is_nonpass() {
    let fixture = NativeFixture::new();
    let root = fixture.root.as_path();
    let environment = fixture.environment;
    let store =
        InvocationStore::new(crate::paths::CheckoutState::resolve(root, &environment).unwrap());
    let mut original = input(Action::Test);
    original.request.scope = NativeFixture::scope(true);
    original.request.filters.ignore_default_filter = true;
    original.request.filters.words = vec!["expected_refusal".into()];
    let destination = begin(&store, &original);
    let mut passed = run(&original, root, &destination, environment.clone())
        .await
        .unwrap();
    assert_eq!(
        passed.outcome,
        Outcome::TestsSucceeded,
        "{:?}",
        passed.failures
    );
    assert_eq!(passed.selected_tests, Some(1));
    assert!(passed.required_capture_complete);
    assert!(
        passed.failures.is_empty(),
        "asserted expected refusals are not outer workflow failures"
    );
    for observed in &passed.observations {
        if let GroupObservation::Test { observation, .. } = observed {
            assert!(observation.summary.complete());
            assert!(observation.summary.failed().is_empty());
        }
    }
    publish(&store, &original, &destination, &mut passed);
    let evidence =
        crate::test_inventory::captured(&store, original.invocation.as_str(), None).unwrap();
    assert_eq!(evidence.observation.outcome, Outcome::TestsSucceeded);
    assert_eq!(evidence.manager_settlement, Some(Settlement::Unknown));

    let mut empty = input(Action::Test);
    empty.request.scope = selection::Scope::RerunFailed {
        invocation: original.invocation.to_string(),
    };
    let empty_destination = begin(&store, &empty);
    let mut report = run(&empty, root, &empty_destination, environment)
        .await
        .unwrap();
    assert_eq!(report.outcome, Outcome::ZeroMatch, "{:?}", report.failures);
    assert_eq!(report.selected_tests, Some(0));
    assert!(report.required_capture_complete);
    assert!(report.fixture_requirements.is_empty());
    assert!(
        report.observations.is_empty(),
        "zero failed identities start no test workload"
    );
    assert!(!empty_destination.join("fixtures").exists());
    publish(&store, &empty, &empty_destination, &mut report);
    let evidence =
        crate::test_inventory::captured(&store, empty.invocation.as_str(), None).unwrap();
    assert_eq!(evidence.observation.outcome, Outcome::ZeroMatch);
    assert_eq!(
        evidence.terminal.helper_exit(),
        1,
        "zero match cannot become an acceptance pass"
    );
    assert_ne!(evidence.id, original.invocation);
    assert_eq!(
        crate::test_inventory::captured(&store, original.invocation.as_str(), None)
            .unwrap()
            .observation
            .outcome,
        Outcome::TestsSucceeded
    );
}

#[tokio::test]
async fn actual_rerun_intent_uses_bound_original_failure_after_old_executable_disposal() {
    let fixture = NativeFixture::new();
    let root = fixture.root.as_path();
    let environment = fixture.environment;
    let store =
        InvocationStore::new(crate::paths::CheckoutState::resolve(root, &environment).unwrap());
    let mut original = input(Action::Test);
    original.request.scope = NativeFixture::scope(true);
    original.request.filters.ignore_default_filter = true;
    original.request.filters.words = vec!["rerun_controlled_failure_probe".into()];
    let destination = begin(&store, &original);
    let mut failing = environment.clone();
    failing.insert("PSE_TEST_RERUN_FAILURE_PROBE".into(), "1".into());
    let mut report = run(&original, root, &destination, failing).await.unwrap();
    assert_eq!(report.outcome, Outcome::Failed, "{:?}", report.failures);
    assert!(report.required_capture_complete);
    assert_eq!(report.selected_tests, Some(1));
    publish(&store, &original, &destination, &mut report);
    let captured =
        crate::test_inventory::captured(&store, original.invocation.as_str(), None).unwrap();
    let failed: Vec<_> = report
        .observations
        .iter()
        .flat_map(|observation| match observation {
            GroupObservation::Test { observation, .. } => observation.summary.failed().to_vec(),
            _ => vec![],
        })
        .collect();
    assert_eq!(failed.len(), 1);
    let old_executable = captured
        .inventory
        .test(&failed[0])
        .unwrap()
        .artifact
        .executable
        .captured
        .clone();
    crate::managed::workflow::release_images(
        destination.parent().unwrap(),
        Action::Test,
        Some(&captured.inventory),
        None,
        &[],
    )
    .unwrap();
    assert!(!old_executable.exists());
    for name in ["cargo-nextest", "cargo-pse-correctness"] {
        assert!(!destination.join("inventory").join(name).exists());
        assert!(
            !destination
                .join("inventory")
                .join(format!("{name}.capture.json"))
                .exists()
        );
    }
    assert!(
        crate::test_inventory::captured(&store, original.invocation.as_str(), None).is_ok(),
        "historical readback needs the committed inventory rather than the old control bytes"
    );
    assert!(
        captured.inventory.test(&failed[0]).is_err(),
        "old executable reuse remains unavailable"
    );

    let metadata: cargo_metadata::Metadata =
        serde_json::from_slice(&fs::read(destination.join("metadata.stdout")).unwrap()).unwrap();
    let mut rerun = input(Action::Test);
    rerun.request.scope = selection::Scope::RerunFailed {
        invocation: original.invocation.to_string(),
    };
    let intent = || {
        test_workflow::Intent::new(
            metadata.clone(),
            rerun.request.clone(),
            rerun.profile.clone(),
            environment.clone(),
        )
    };
    assert_eq!(intent().unwrap().plan.rerun_cases().len(), 1);
    // A self-consistent incomplete historical observation stays readable but cannot
    // silently nominate a complete failed population. This literal negative control
    // claims neither an actually interrupted process nor manager settlement.
    let report_path = destination.join("report.json");
    let manager_path = destination.parent().unwrap().join("result.json");
    let original_report = fs::read(&report_path).unwrap();
    let original_manager = fs::read(&manager_path).unwrap();
    let mut passive: serde_json::Value = serde_json::from_slice(&original_report).unwrap();
    passive["required_capture_complete"] = false.into();
    passive["outcome"] = "incomplete".into();
    let incomplete = serde_json::to_vec(&passive).unwrap();
    let identity = ContentIdentity::of(&incomplete);
    let mut manager: Envelope<ManagedSummary> = serde_json::from_slice(&original_manager).unwrap();
    let terminal = &mut manager.payload.workflow.as_mut().unwrap().terminal;
    terminal.required_capture_complete = false;
    terminal.outcome = Outcome::Incomplete;
    terminal.report.sha256 = identity.sha256;
    terminal.report.byte_length = identity.byte_length;
    let replace = |path: &Path, bytes: &[u8]| {
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(path, bytes).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o400)).unwrap();
    };
    replace(&report_path, &incomplete);
    replace(&manager_path, &serde_json::to_vec(&manager).unwrap());
    assert!(
        !crate::test_inventory::captured(&store, original.invocation.as_str(), None)
            .unwrap()
            .observation
            .required_capture_complete
    );
    assert!(
        intent().is_err(),
        "incomplete original capture is not a zero-match or a fresh rerun grant"
    );
    for complete in [None, Some(false)] {
        let mut passive: serde_json::Value = serde_json::from_slice(&original_report).unwrap();
        passive["outcome"] = "incomplete".into();
        let observed = passive["observations"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|group| group["kind"] == "test")
            .unwrap();
        let summary = observed["observation"]["summary"].as_object_mut().unwrap();
        if let Some(complete) = complete {
            summary.insert("complete".into(), complete.into());
        } else {
            summary.remove("complete");
        }
        let bytes = serde_json::to_vec(&passive).unwrap();
        let identity = ContentIdentity::of(&bytes);
        let terminal = &mut manager.payload.workflow.as_mut().unwrap().terminal;
        terminal.required_capture_complete = true;
        terminal.report.sha256 = identity.sha256;
        terminal.report.byte_length = identity.byte_length;
        replace(&report_path, &bytes);
        replace(&manager_path, &serde_json::to_vec(&manager).unwrap());
        assert!(
            intent().is_err(),
            "a failed name without complete native reconciliation cannot nominate rerun intent"
        );
    }
    replace(&report_path, &original_report);
    replace(&manager_path, &original_manager);
    let inventory_path = destination.join("inventory/inventory.json");
    let inventory_bytes = fs::read(&inventory_path).unwrap();
    fs::set_permissions(&inventory_path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&inventory_path, b"{}").unwrap();
    assert!(
        intent().is_err(),
        "changed original association cannot provide rerun intent"
    );
    fs::write(&inventory_path, &inventory_bytes).unwrap();
    fs::set_permissions(&inventory_path, fs::Permissions::from_mode(0o400)).unwrap();
    let absent = destination.join("inventory/original-inventory.saved");
    fs::rename(&inventory_path, &absent).unwrap();
    assert!(
        intent().is_err(),
        "missing original association cannot provide rerun intent"
    );
    fs::rename(&absent, &inventory_path).unwrap();

    let fresh_destination = begin(&store, &rerun);
    let mut fresh = run(&rerun, root, &fresh_destination, environment)
        .await
        .unwrap();
    assert_eq!(
        fresh.outcome,
        Outcome::TestsSucceeded,
        "{:?}",
        fresh.failures
    );
    assert_eq!(fresh.selected_tests, Some(1));
    assert_ne!(fresh.invocation, original.invocation);
    publish(&store, &rerun, &fresh_destination, &mut fresh);
    let current = crate::test_inventory::captured(&store, rerun.invocation.as_str(), None).unwrap();
    test_workflow::source::verify_rerun(&current.inventory).unwrap();
    assert!(
        current
            .inventory
            .test(&current.inventory.selected_tests()[0])
            .is_ok()
    );
    assert!(!old_executable.exists());
    assert_eq!(
        crate::test_inventory::captured(&store, original.invocation.as_str(), None)
            .unwrap()
            .observation
            .outcome,
        Outcome::Failed
    );
}
