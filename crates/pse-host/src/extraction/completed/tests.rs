// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use serde_json::json;

fn listing() -> nextest_metadata::TestListSummary {
    let suite = |id: &str| json!({"binary-id":id,"binary-name":"shared-name","package-id":"case-id","kind":"test","binary-path":"/fixture/bin","build-platform":"target","package-name":"case","cwd":"/fixture","status":"listed","testcases":{"same":{"kind":"test","ignored":false,"filter-match":{"status":"matches"}},"ignored":{"kind":"test","ignored":true,"filter-match":{"status":"mismatch","reason":"ignored"}}}});
    serde_json::from_value(json!({"rust-build-meta":{"target-directory":"/fixture/target","base-output-directories":["debug"],"non-test-binaries":{},"linked-paths":[]},"test-count":4,"rust-suites":{"first":suite("first"),"second":suite("second")}})).unwrap()
}
fn reconcile(
    xml: Option<&str>,
    exit: i32,
    capture: bool,
) -> (NextestSummary, Vec<NativeFailure>, Option<ExtractionError>) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("junit.xml");
    if let Some(xml) = xml {
        std::fs::write(&path, xml).unwrap();
    }
    CompletedReport::from_listing(&listing(), 3, path, IgnoredPolicy::Default)
        .unwrap()
        .finish(exit, capture)
}
const PASS: &str = r#"<testsuites tests="2" failures="0" errors="0"><testsuite name="first" tests="1" failures="0" errors="0"><testcase classname="first" name="same"/></testsuite><testsuite name="second" tests="1" failures="0" errors="0"><testcase classname="second" name="same"/></testsuite></testsuites>"#;

#[test]
fn native_binary_exclusions_reconcile_only_the_selected_population() {
    use nextest_metadata::RustTestSuiteStatusSummary;
    for status in [
        RustTestSuiteStatusSummary::SKIPPED,
        RustTestSuiteStatusSummary::SKIPPED_DEFAULT_FILTER,
    ] {
        let mut data = listing();
        let excluded = data
            .rust_suites
            .get_mut(&nextest_metadata::RustBinaryId::new("second"))
            .unwrap();
        excluded.status = status;
        excluded.test_cases.clear();
        data.test_count = 2;
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("junit.xml");
        std::fs::write(&path, r#"<testsuites tests="1" failures="0" errors="0"><testsuite name="first" tests="1" failures="0" errors="0"><testcase classname="first" name="same"/></testsuite></testsuites>"#).unwrap();
        let report =
            CompletedReport::from_listing(&data, 3, path.clone(), IgnoredPolicy::Default).unwrap();
        let (summary, failures, error) = report.finish(0, true);
        assert_eq!(summary.selected(), 1);
        assert_eq!(summary.completed(), 1);
        assert_eq!(summary.outcome(), NextestOutcome::Succeeded);
        assert!(failures.is_empty());
        assert!(error.is_none());
        std::fs::write(&path, PASS).unwrap();
        let report = CompletedReport::from_listing(&data, 3, path, IgnoredPolicy::Default).unwrap();
        assert_eq!(
            report.finish(0, true).0.outcome(),
            NextestOutcome::Incomplete
        );
    }
    for status in [
        RustTestSuiteStatusSummary::new_const("unknown"),
        RustTestSuiteStatusSummary::SKIPPED,
        RustTestSuiteStatusSummary::SKIPPED_DEFAULT_FILTER,
    ] {
        let mut data = listing();
        data.rust_suites.values_mut().next().unwrap().status = status;
        assert!(
            CompletedReport::from_listing(&data, 3, "unused.xml".into(), IgnoredPolicy::Default)
                .is_err()
        );
    }
}

#[test]
fn native_completed_report_preserves_binary_identity_and_actual_capture() {
    let (summary, failures, error) = reconcile(Some(PASS), 0, true);
    assert!(error.is_none());
    assert_eq!(summary.outcome(), NextestOutcome::Succeeded);
    assert!(summary.complete());
    assert_eq!(summary.selected(), 2);
    assert_eq!(summary.completed(), 2);
    assert!(failures.is_empty());
    let (summary, _, _) = reconcile(Some(PASS), 0, false);
    assert_eq!(summary.outcome(), NextestOutcome::Incomplete);
    let (summary, _, _) = reconcile(
        Some(PASS),
        nextest_metadata::NextestExitCode::WRITE_OUTPUT_ERROR,
        true,
    );
    assert_eq!(summary.outcome(), NextestOutcome::RunnerFailed);
}

#[test]
fn absent_partial_foreign_and_duplicate_reports_cannot_pass() {
    for xml in [
        None,
        Some("<testsuites>"),
        Some("<testsuites tests=\"0\" failures=\"0\" errors=\"0\"/>"),
    ] {
        assert_eq!(
            reconcile(xml, 0, true).0.outcome(),
            NextestOutcome::Incomplete
        );
    }
    let foreign = PASS.replace("classname=\"second\"", "classname=\"first\"");
    let (summary, _, error) = reconcile(Some(&foreign), 0, true);
    assert_eq!(summary.outcome(), NextestOutcome::Incomplete);
    assert!(error.is_some());
    let duplicate=PASS.replace("<testcase classname=\"first\" name=\"same\"/>","<testcase classname=\"first\" name=\"same\"/><testcase classname=\"first\" name=\"same\"/>");
    assert_eq!(
        reconcile(Some(&duplicate), 0, true).0.outcome(),
        NextestOutcome::Incomplete
    );
}

#[test]
fn native_setup_failure_and_known_test_failure_survive_incomplete_coverage() {
    let xml = r#"<testsuites tests="1" failures="1" errors="0"><testsuite name="@setup-script:fixture" tests="1" failures="1" errors="0"><testcase name="fixture"><failure>setup failed</failure></testcase></testsuite></testsuites>"#;
    let (summary, _, _) = reconcile(
        Some(xml),
        nextest_metadata::NextestExitCode::SETUP_SCRIPT_FAILED,
        true,
    );
    assert_eq!(summary.outcome(), NextestOutcome::SetupFailed);
    assert!(!summary.complete());
    let xml = r#"<testsuites tests="1" failures="1" errors="0"><testsuite name="first" tests="1" failures="1" errors="0"><testcase classname="first" name="same"><failure>original assertion</failure></testcase></testsuite></testsuites>"#;
    let (summary, failures, _) = reconcile(Some(xml), 100, true);
    assert_eq!(summary.outcome(), NextestOutcome::TestFailed);
    assert!(!summary.complete());
    assert_eq!(failures[0].identity.binary_id, "first");
    assert_eq!(failures[0].identity.group, 3);
    assert_eq!(
        failures[0].description.as_deref(),
        Some("original assertion")
    );
}

#[test]
fn target_setup_refusal_is_attributed_through_managed_observation() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("junit.xml");
    let identity = TestIdentity {
        group: 3,
        package_id: "case-id".into(),
        binary_id: "first".into(),
        test_name: "same".into(),
    };
    let marker = TargetSetupRefusal::NativeServiceInputs.marker();
    let xml = |case: &str| PASS.replace("<testcase classname=\"first\" name=\"same\"/>", case);
    let observe = |case: &str, owned: bool| {
        std::fs::write(&path, xml(case)).unwrap();
        let report =
            CompletedReport::from_listing(&listing(), 3, path.clone(), IgnoredPolicy::Default)
                .unwrap();
        let observer = crate::test_workflow::CompletedObserver::new(report);
        let observer = if owned {
            observer.with_target_owned([identity.clone()].into())
        } else {
            observer
        };
        let (observation, error) =
            observer.finish(nextest_metadata::NextestExitCode::TEST_RUN_FAILED, true);
        assert!(error.is_none());
        assert!(observation.summary.complete());
        assert_eq!(
            observation.summary.failed(),
            std::slice::from_ref(&identity)
        );
        assert_eq!(
            observation.summary.child_class(),
            NextestExitClass::TestsFailed
        );
        assert_eq!(observation.failures[0].identity, identity);
        assert_eq!(observation.failures[0].report, path);
        observation
    };
    for stream in ["system-err", "system-out"] {
        let case = format!(
            "<testcase classname=\"first\" name=\"same\"><failure>original setup error</failure><{stream}>{marker}\noriginal stderr</{stream}></testcase>"
        );
        let observation = observe(&case, true);
        assert_eq!(observation.summary.outcome(), NextestOutcome::SetupFailed);
        assert_eq!(
            observation.summary.setup(),
            NextestSetup::FailedByTargetOwner
        );
        assert_eq!(
            serde_json::to_value(&observation).unwrap()["failures"][0]["target_setup_refusals"],
            json!(["native_service_inputs"])
        );
        let unowned = observe(&case, false);
        assert_eq!(unowned.summary.outcome(), NextestOutcome::TestFailed);
        assert!(unowned.failures[0].target_setup_refusals.is_empty());
    }
    let case = format!(
        "<testcase classname=\"first\" name=\"same\"><failure>{marker}</failure><system-err>assertion text mentioning {marker}</system-err></testcase>"
    );
    let observation = observe(&case, true);
    assert_eq!(observation.summary.outcome(), NextestOutcome::TestFailed);
    assert_eq!(observation.summary.setup(), NextestSetup::Unknown);
    assert!(observation.failures[0].target_setup_refusals.is_empty());

    let case = format!(
        "<testcase classname=\"first\" name=\"same\"><failure>original setup error</failure><system-err>{marker}</system-err><rerunFailure>assertion in retry</rerunFailure></testcase>"
    );
    let observation = observe(&case, true);
    assert_eq!(observation.summary.outcome(), NextestOutcome::TestFailed);
    assert_eq!(
        observation.summary.setup(),
        NextestSetup::FailedByTargetOwner
    );
    assert_eq!(
        observation.failures[0].target_setup_refusals,
        [TargetSetupRefusal::NativeServiceInputs]
    );
}

#[test]
fn required_original_fixture_result_cannot_be_inferred_from_wrapper_success() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("junit.xml");
    let owner_path = root.path().join("fixture-outcome.json");
    std::fs::write(&path, PASS).unwrap();
    let report = || {
        CompletedReport::from_listing(&listing(), 3, path.clone(), IgnoredPolicy::Default)
            .unwrap()
            .with_fixture_outcome(owner_path.clone())
    };
    for contents in [
        None,
        Some("{}"),
        Some(r#"{"version":2,"runner_exit":0,"fixture_settled":true}"#),
    ] {
        if let Some(contents) = contents {
            std::fs::write(&owner_path, contents).unwrap();
        }
        let (summary, _, error) = report().finish(0, true);
        assert!(error.is_some());
        assert_eq!(summary.outcome(), NextestOutcome::Incomplete);
        assert_eq!(summary.child_exit(), 0);
        assert_eq!(summary.native_child_exit(), None);
        assert_eq!(summary.fixture_settled, Some(false));
        assert!(!summary.complete());
    }
    std::fs::write(
        &owner_path,
        r#"{"version":1,"runner_exit":0,"fixture_settled":true}"#,
    )
    .unwrap();
    let (summary, _, error) = report().finish(0, true);
    assert!(error.is_none());
    assert_eq!(summary.outcome(), NextestOutcome::Succeeded);
    assert_eq!(summary.native_child_exit(), Some(0));
    assert_eq!(summary.fixture_settled, Some(true));
    assert!(summary.complete());
}

#[test]
fn native_retry_and_skipped_population_do_not_require_event_reconstruction() {
    let xml=PASS.replace("<testcase classname=\"first\" name=\"same\"/>","<testcase classname=\"first\" name=\"same\"><flakyFailure>earlier attempt</flakyFailure></testcase><testcase classname=\"first\" name=\"ignored\"><skipped/></testcase>");
    let (summary, _, error) = reconcile(Some(&xml), 0, true);
    assert!(error.is_none());
    assert_eq!(summary.outcome(), NextestOutcome::Succeeded);
    assert_eq!(summary.retries.len(), 1);
    assert_eq!(summary.retries[0].attempts, 2);
    assert!(summary.retries[0].flaky);
}
