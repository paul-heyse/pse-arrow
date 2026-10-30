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
fn native_retry_and_skipped_population_do_not_require_event_reconstruction() {
    let xml=PASS.replace("<testcase classname=\"first\" name=\"same\"/>","<testcase classname=\"first\" name=\"same\"><flakyFailure>earlier attempt</flakyFailure></testcase><testcase classname=\"first\" name=\"ignored\"><skipped/></testcase>");
    let (summary, _, error) = reconcile(Some(&xml), 0, true);
    assert!(error.is_none());
    assert_eq!(summary.outcome(), NextestOutcome::Succeeded);
    assert_eq!(summary.retries.len(), 1);
    assert_eq!(summary.retries[0].attempts, 2);
    assert!(summary.retries[0].flaky);
}
