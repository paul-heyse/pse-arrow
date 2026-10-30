// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::process::Command;
fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(root.path())
            .args(["init", "--quiet", "--initial-branch", "main"])
            .status()
            .unwrap()
            .success()
    );
    root
}
#[test]
fn markdown_plan_navigation_uses_real_links_without_following_documents() {
    let source = "| Plan | Owner |\n|---|---|\n| [59: **Agent** `CLI`](59-agent-cli.md#current) | active index |\n\n[reference][p]\n\n[p]: 58-rust-owned-system.md#remaining\n\n[manual](../dev/guide.md) [external](https://example.test/90.md)\n\n`[not a link](99-do-not-inspect.md)`\n";
    let value = plan_links(source);
    assert_eq!(value.listed.len(), 2);
    assert_eq!(value.primary.as_ref().unwrap().label, "59: Agent CLI");
    assert_eq!(value.listed[0].fragment.as_deref(), Some("current"));
    assert_eq!(value.listed[1].document, "58-rust-owned-system.md");
}
#[test]
fn status_reads_unborn_checkout_and_known_empty_store_without_creating_state() {
    let root = fixture();
    std::fs::create_dir_all(root.path().join("docs/plans")).unwrap();
    std::fs::write(
        root.path().join("docs/plans/README.md"),
        "[59: current](59-agent-cli.md#current)\n",
    )
    .unwrap();
    let state = root.path().join("private-state");
    let env = [("XDG_STATE_HOME".into(), state.as_os_str().to_owned())].into();
    let response = inspect(
        root.path(),
        &env,
        true,
        InspectionScope::Full,
        Arc::new(AtomicBool::new(false)),
    );
    assert_eq!(response.exit_code(), 0, "{:?}", response.failures);
    assert_eq!(response.observation.completeness, Completeness::Complete);
    assert!(
        matches!(response.payload.invocations,Information::Available(ref values) if values.is_empty())
    );
    assert!(matches!(
        response.payload.host_tools,
        Information::Available(generation::Freshness::Unprepared)
    ));
    assert!(
        matches!(response.payload.repository,Information::Available(ref facts) if matches!(facts.head_after,repository::HeadState::Unborn{..}))
    );
    assert!(!state.exists());
    assert!(!root.path().join("target").exists());
    assert!(!root.path().join(".git/index").exists());
    let mut bytes = Vec::new();
    envelope::write(
        &mut bytes,
        response,
        envelope::OutputFormat::Json,
        &envelope::Presentation::default(),
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let schema = envelope::schema::<Report>("status", &envelope::Presentation::default()).unwrap();
    let validator = jsonschema::validator_for(schema.as_value()).unwrap();
    assert!(validator.is_valid(&value));
}
#[test]
fn one_unavailable_source_preserves_other_observations_and_cannot_report_complete_success() {
    let root = fixture();
    let state = root.path().join("state");
    let env = [("XDG_STATE_HOME".into(), state.as_os_str().to_owned())].into();
    let response = inspect(
        root.path(),
        &env,
        false,
        InspectionScope::Full,
        Arc::new(AtomicBool::new(false)),
    );
    assert_eq!(response.exit_code(), 1);
    assert_eq!(response.observation.completeness, Completeness::Partial);
    assert!(matches!(
        response.payload.plans,
        Information::Unavailable { .. }
    ));
    assert!(matches!(
        response.payload.repository,
        Information::Available(_)
    ));
    assert!(matches!(
        response.payload.worktrees,
        Information::Unavailable { .. }
    ));
    assert!(!response.failures.is_empty());
    assert!(!state.exists());
}
#[test]
fn unreadable_repository_is_unavailable_and_never_a_fabricated_clean_checkout() {
    let root = tempfile::tempdir().unwrap();
    let env = [(
        "XDG_STATE_HOME".into(),
        root.path().join("state").into_os_string(),
    )]
    .into();
    let response = inspect(
        root.path(),
        &env,
        true,
        InspectionScope::Full,
        Arc::new(AtomicBool::new(false)),
    );
    assert_eq!(response.exit_code(), 1);
    assert!(matches!(
        response.payload.repository,
        Information::Unavailable { .. }
    ));
    assert!(matches!(
        response.payload.invocations,
        Information::Unavailable { .. }
    ));
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn summary_does_not_open_retained_results_and_partial_history_keeps_healthy_rows() {
    use crate::invocation::InvocationSpec;
    let root = fixture();
    std::fs::create_dir_all(root.path().join("docs/plans")).unwrap();
    std::fs::write(
        root.path().join("docs/plans/README.md"),
        "[59](59-agent-cli.md)",
    )
    .unwrap();
    let env = [(
        "XDG_STATE_HOME".into(),
        root.path().join("state").into_os_string(),
    )]
    .into();
    let store = InvocationStore::new(CheckoutState::resolve(root.path(), &env).unwrap());
    let begin = || {
        store
            .begin(InvocationSpec {
                invocation_id: envelope::CliInvocationId::new(),
                command: "run".into(),
                session: None,
                parent: None,
                operation: serde_json::value::RawValue::from_string(
                    "{\"huge\":1234567890123456789012345678901234567890,\"same\":1,\"same\":2}"
                        .into(),
                )
                .unwrap(),
                control: generation::PreparedGeneration {
                    generation: "fixture".into(),
                    protocol: 1,
                    manifest_digest: "a".repeat(64),
                },
                environment_digest: "b".repeat(64),
                scientific: Vec::new(),
            })
            .unwrap()
    };
    let healthy = begin();
    std::fs::write(
        healthy.path().join("result.json"),
        b"broken retained response",
    )
    .unwrap();
    let inspect_scope = |scope| {
        inspect(
            root.path(),
            &env,
            false,
            scope,
            Arc::new(AtomicBool::new(false)),
        )
    };
    let summary = inspect_scope(InspectionScope::Summary);
    assert_eq!(summary.observation.completeness, Completeness::Complete);
    assert!(summary.failures.is_empty());
    assert!(matches!(
        summary.payload.summary().invocation_records,
        Information::Available(1)
    ));
    let Information::Available(rows) = summary.payload.invocations else {
        panic!("metadata available")
    };
    assert!(matches!(
        rows[0].recorded_response,
        Information::Unavailable { .. }
    ));
    let full = inspect_scope(InspectionScope::Full);
    assert_eq!(full.observation.completeness, Completeness::Partial);
    assert!(!full.failures.is_empty());
    let Information::Available(rows) = full.payload.invocations else {
        panic!("metadata preserved")
    };
    assert_eq!(rows.len(), 1);
    let corrupt = begin();
    std::fs::write(corrupt.path().join("meta.json"), b"broken metadata").unwrap();
    let partial = inspect_scope(InspectionScope::Summary);
    assert_eq!(partial.observation.completeness, Completeness::Partial);
    assert!(
        partial
            .failures
            .iter()
            .any(|failure| failure.facts.observations.contains_key("invocation_id"))
    );
    let Information::Available(rows) = partial.payload.invocations else {
        panic!("healthy row preserved")
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].invocation_id, *healthy.id());
}
