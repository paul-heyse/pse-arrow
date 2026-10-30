// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use serde_json::json;

fn extractor(capacity: usize) -> CargoExtractor {
    CargoExtractor::new(NonZeroUsize::new(capacity).expect("nonzero fixture"))
}
fn origin(ordinal: u64) -> RecordOrigin {
    RecordOrigin {
        stream: OutputStream::Stdout,
        ordinal,
        byte_offset: 0,
        byte_length: 0,
    }
}
fn diagnostic(code: &str, level: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({"reason":"compiler-message","package_id":"path+file:///fixture#case@0.0.0",
        "target":{"name":"case","kind":["bin"],"crate_types":["bin"],"src_path":"/fixture/src/main.rs","edition":"2024","doc":true,"doctest":false,"test":true},
        "message":{"message":"expected secret-value","code":{"code":code,"explanation":null},"level":level,
            "spans":[{"file_name":"src/α.rs","byte_start":19,"byte_end":21,"line_start":2,"line_end":2,"column_start":3,"column_end":4,"is_primary":true,"text":[],"label":"secret-value","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],
            "children":[],"rendered":"full native context: secret-value"}})).expect("wire fixture")
}
fn terminal(success: bool) -> Vec<u8> {
    serde_json::to_vec(&json!({"reason":"build-finished","success":success}))
        .expect("terminal fixture")
}

#[test]
fn duplicate_presentation_does_not_discard_original_records_or_native_coordinates() {
    let mut extractor = extractor(8);
    let raw = diagnostic("E0308", "error");
    let policy = RedactionPolicy::new([("API_TOKEN".into(), "secret-value".into())]);
    let first = extractor.consume(&raw, origin(0), &policy).expect("first");
    let CargoRecord::Diagnostic(first) = first else {
        panic!("diagnostic")
    };
    assert!(!first.duplicate);
    assert_eq!(first.code.as_deref(), Some("E0308"));
    assert_eq!(first.primary[0].byte_start, 19);
    assert_eq!(first.primary[0].character_start, 3);
    assert_eq!(first.primary[0].line_start, 2);
    assert!(!first.rendered.expect("context").contains("secret-value"));
    let CargoRecord::Diagnostic(second) =
        extractor.consume(&raw, origin(1), &policy).expect("second")
    else {
        panic!("diagnostic")
    };
    assert!(second.duplicate);
    assert_eq!(second.origin.ordinal, 1);
    extractor
        .consume(&terminal(false), origin(2), &policy)
        .expect("terminal");
    let summary = extractor.finish(101);
    assert_eq!(summary.outcome, CompilerOutcome::Failed);
    assert_eq!(summary.unique_primary_failures, Some(1));
    assert_eq!(summary.error_records, 2);
    assert_eq!(summary.duplicates, 1);
    assert!(summary.complete);
}

#[test]
fn zero_exit_and_a_valid_tail_cannot_erase_missing_or_malformed_required_evidence() {
    assert_eq!(extractor(8).finish(0).outcome, CompilerOutcome::Incomplete);
    let mut broken = extractor(8);
    let policy = RedactionPolicy::default();
    assert!(
        broken
            .consume(
                b"{\"reason\":\"compiler-message\",\"message\":false}",
                origin(0),
                &policy
            )
            .is_err()
    );
    broken
        .consume(&terminal(true), origin(1), &policy)
        .expect("valid terminal");
    let summary = broken.finish(0);
    assert_eq!(summary.outcome, CompilerOutcome::Incomplete);
    assert!(!summary.complete);
    let mut lost = extractor(8);
    lost.mark_incomplete();
    lost.consume(&terminal(true), origin(1), &policy)
        .expect("valid terminal");
    assert_eq!(
        lost.finish(7).outcome,
        CompilerOutcome::Failed,
        "known original failure wins over observation loss"
    );
}

#[test]
fn optional_grouping_capacity_cannot_drop_records_or_change_compiler_outcome() {
    let mut extractor = extractor(1);
    let policy = RedactionPolicy::default();
    for (ordinal, code) in ["warning-a", "warning-b"].into_iter().enumerate() {
        assert!(matches!(
            extractor
                .consume(
                    &diagnostic(code, "warning"),
                    origin(ordinal as u64),
                    &policy
                )
                .expect("retained warning"),
            CargoRecord::Diagnostic(_)
        ));
    }
    extractor
        .consume(&terminal(true), origin(2), &policy)
        .expect("terminal");
    let summary = extractor.finish(0);
    assert_eq!(summary.outcome, CompilerOutcome::Succeeded);
    assert_eq!(summary.warning_records, 2);
    assert_eq!(summary.unique_primary_failures, None);
    assert!(
        summary.complete,
        "grouping capacity is distinct from required protocol completeness"
    );
}

#[test]
fn duplicate_terminal_and_post_terminal_records_are_incomplete() {
    let mut extractor = extractor(8);
    let policy = RedactionPolicy::default();
    extractor
        .consume(&terminal(true), origin(0), &policy)
        .expect("terminal");
    extractor
        .consume(&terminal(true), origin(1), &policy)
        .expect("duplicate observed");
    assert_eq!(extractor.finish(0).outcome, CompilerOutcome::Incomplete);
}

#[test]
fn actual_type_error_retains_one_coded_source_failure_and_full_native_context() {
    use std::{fs, path::Path, process::Command};
    let source = tempfile::tempdir().expect("fixture crate");
    fs::write(
        source.path().join("Cargo.toml"),
        "[package]\nname=\"compiler-case\"\nversion=\"0.0.0\"\nedition=\"2024\"\n",
    )
    .expect("manifest");
    fs::create_dir(source.path().join("src")).expect("source directory");
    fs::write(
        source.path().join("src/main.rs"),
        "fn main() { let _: u32 = \"wrong\"; }\n",
    )
    .expect("type-error source");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("checkout");
    fs::copy(
        root.join("rust-toolchain.toml"),
        source.path().join("rust-toolchain.toml"),
    )
    .expect("pinned compiler");
    let cargo = crate::correctness::real_cargo().expect("pinned Cargo");
    let output = Command::new(cargo)
        .current_dir(source.path())
        .args(["check", "--offline", "--message-format=json"])
        .output()
        .expect("actual Cargo check");
    assert!(!output.status.success());
    let mut extractor = extractor(32);
    let policy = RedactionPolicy::default();
    let mut coded = Vec::new();
    let mut byte_offset = 0;
    for (ordinal, line) in output
        .stdout
        .split_inclusive(|byte| *byte == b'\n')
        .enumerate()
    {
        let record_origin = RecordOrigin {
            stream: OutputStream::Stdout,
            ordinal: ordinal as u64,
            byte_offset,
            byte_length: line.len() as u64,
        };
        byte_offset += line.len() as u64;
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let record = extractor
            .consume(line, record_origin, &policy)
            .expect("actual Cargo protocol");
        if let CargoRecord::Diagnostic(record) = record
            && record.code.as_deref() == Some("E0308")
        {
            coded.push(record);
        }
    }
    assert_eq!(coded.len(), 1);
    assert_eq!(coded[0].primary[0].line_start, 1);
    assert!(coded[0].primary[0].file.ends_with("main.rs"));
    assert!(
        coded[0]
            .rendered
            .as_ref()
            .is_some_and(|text| text.contains("mismatched types"))
    );
    let summary = extractor.finish(output.status.code().expect("Cargo exit"));
    assert_eq!(summary.outcome, CompilerOutcome::Failed);
    assert_eq!(summary.unique_primary_failures, Some(1));
    assert!(summary.complete);
}

fn nextest_listing(binaries: &[(&str, &str, &str)]) -> nextest_metadata::TestListSummary {
    let suites: serde_json::Map<_, _> = binaries.iter().map(|(id, name, kind)| {
        ((*id).into(), json!({"binary-id":id,"binary-name":name,"package-id":"path+file:///fixture#case@0.0.0","kind":kind,"binary-path":format!("/captured/{id}"),"build-platform":"target","package-name":"case","cwd":"/fixture","status":"listed","testcases":{"same_name":{"kind":"test","ignored":false,"filter-match":{"status":"matches"}}}}))
    }).collect();
    serde_json::from_value(json!({"rust-build-meta":{"target-directory":"/fixture/target","base-output-directories":["debug"],"non-test-binaries":{},"linked-paths":[]},"test-count":binaries.len(),"rust-suites":suites})).unwrap()
}
fn nextest_extractor(listing: &nextest_metadata::TestListSummary) -> NextestExtractor {
    NextestExtractor::from_listing(3, listing, crate::selection::IgnoredPolicy::Default).unwrap()
}
fn suite_start(name: &str, kind: &str) -> serde_json::Value {
    json!({"type":"suite","event":"started","test_count":1,"nextest":{"crate":"case","test_binary":name,"kind":kind}})
}
fn suite_end(name: &str, kind: &str, failed: bool) -> serde_json::Value {
    json!({"type":"suite","event":if failed {"failed"} else {"ok"},"passed":usize::from(!failed),"failed":usize::from(failed),"ignored":0,"measured":0,"filtered_out":0,"exec_time":0.1,"nextest":{"crate":"case","test_binary":name,"kind":kind}})
}
fn test_event(name: &str, event: &str) -> serde_json::Value {
    let mut value = json!({"type":"test","event":event,"name":name});
    if matches!(event, "ok" | "failed") {
        value["exec_time"] = json!(0.1);
    }
    value
}
fn feed_nextest(
    extractor: &mut NextestExtractor,
    records: &[serde_json::Value],
) -> Vec<NextestRecord> {
    records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            extractor
                .consume(
                    &serde_json::to_vec(record).unwrap(),
                    origin(index as u64),
                    &RedactionPolicy::default(),
                )
                .unwrap()
        })
        .collect()
}

#[test]
fn nextest_reconciles_interleaved_duplicate_names_in_exact_captured_binaries() {
    let listing = nextest_listing(&[
        ("case::first", "first", "test"),
        ("case::second", "second", "test"),
    ]);
    let mut extractor = nextest_extractor(&listing);
    let records = feed_nextest(
        &mut extractor,
        &[
            suite_start("first", "test"),
            suite_start("second", "test"),
            test_event("case::first$same_name", "started"),
            test_event("case::second$same_name", "started"),
            test_event("case::second$same_name", "ok"),
            suite_end("second", "test", false),
            test_event("case::first$same_name", "ok"),
            suite_end("first", "test", false),
        ],
    );
    let NextestRecord::Test { identity, .. } = &records[4] else {
        panic!("test")
    };
    assert_eq!(identity.group, 3);
    assert_eq!(identity.binary_id, "case::second");
    let summary = extractor.finish(0);
    assert_eq!(summary.outcome(), NextestOutcome::Succeeded);
    assert_eq!(summary.setup(), NextestSetup::SucceededByRunner);
    assert_eq!(summary.selected(), 2);
    assert_eq!(summary.completed(), 2);
    assert!(summary.complete());
    let schema = schemars::generate::SchemaSettings::draft2020_12()
        .for_serialize()
        .into_generator()
        .into_root_schema_for::<NextestSummary>();
    assert!(
        jsonschema::validator_for(schema.as_value())
            .unwrap()
            .is_valid(&serde_json::to_value(summary).unwrap())
    );
}

#[test]
fn nextest_zero_exit_missing_records_setup_failure_and_zero_population_stay_distinct() {
    let listing = nextest_listing(&[("case::first", "first", "test")]);
    assert_eq!(
        nextest_extractor(&listing).finish(0).outcome(),
        NextestOutcome::Incomplete
    );
    let mut extractor = nextest_extractor(&listing);
    feed_nextest(
        &mut extractor,
        &[
            suite_start("first", "test"),
            test_event("case::first$same_name", "started"),
            suite_end("first", "test", false),
        ],
    );
    assert_eq!(extractor.finish(0).outcome(), NextestOutcome::Incomplete);
    let setup =
        nextest_extractor(&listing).finish(nextest_metadata::NextestExitCode::SETUP_SCRIPT_FAILED);
    assert_eq!(setup.outcome(), NextestOutcome::SetupFailed);
    assert_eq!(setup.setup(), NextestSetup::FailedByRunner);
    assert_eq!(setup.child_class(), NextestExitClass::SetupFailed);
    assert!(!setup.complete());
    for exit in [0, nextest_metadata::NextestExitCode::NO_TESTS_RUN] {
        let summary = nextest_extractor(&nextest_listing(&[])).finish(exit);
        assert_eq!(summary.outcome(), NextestOutcome::ZeroMatch);
        assert_eq!(summary.setup(), NextestSetup::Unknown);
    }
}

#[test]
fn nextest_failure_output_retry_suffix_and_repeated_native_suite_blocks_are_preserved() {
    let listing = nextest_listing(&[("case::first", "first", "test")]);
    let mut extractor = nextest_extractor(&listing);
    let mut failure = test_event("case::first$same_name#2", "failed");
    failure["stdout"] = json!("thread panicked at src/α.rs:7:9: secret-value");
    failure["reason"] = json!("native failure");
    let policy = RedactionPolicy::new([("API_TOKEN".into(), "secret-value".into())]);
    let records = [
        suite_start("first", "test"),
        test_event("case::first$same_name", "started"),
        suite_end("first", "test", false),
        suite_start("first", "test"),
        failure,
        suite_end("first", "test", true),
    ];
    let mut observed = None;
    for (index, record) in records.iter().enumerate() {
        let decoded = extractor
            .consume(
                &serde_json::to_vec(record).unwrap(),
                origin(index as u64),
                &policy,
            )
            .unwrap();
        if index == 4 {
            observed = Some(decoded);
        }
    }
    let NextestRecord::Test {
        attempts,
        output,
        identity,
        origin,
        ..
    } = observed.unwrap()
    else {
        panic!("failure")
    };
    assert_eq!(attempts, Some(2));
    assert_eq!(identity.test_name, "same_name");
    assert_eq!(origin.ordinal, 4);
    let output = output.unwrap();
    assert!(output.contains("src/α.rs:7:9"));
    assert!(!output.contains("secret-value"));
    let summary = extractor.finish(100);
    assert_eq!(summary.outcome(), NextestOutcome::TestFailed);
    assert!(summary.complete());
    assert_eq!(summary.failed(), &[identity]);
    let outstanding = nextest_extractor(&listing)
        .finish(nextest_metadata::NextestExitCode::RERUN_TESTS_OUTSTANDING);
    assert_eq!(
        outstanding.child_class(),
        NextestExitClass::OutstandingRerun
    );
    assert_ne!(outstanding.outcome(), NextestOutcome::Succeeded);
}

#[test]
fn nextest_ambiguous_native_test_names_never_invent_a_target_kind() {
    let listing = nextest_listing(&[
        ("case::test/shared", "shared", "test"),
        ("case::bin/shared", "shared", "bin"),
    ]);
    let mut extractor = nextest_extractor(&listing);
    feed_nextest(
        &mut extractor,
        &[suite_start("shared", "test"), suite_start("shared", "bin")],
    );
    let error = extractor
        .consume(
            &serde_json::to_vec(&test_event("case::shared$same_name", "failed")).unwrap(),
            origin(2),
            &RedactionPolicy::default(),
        )
        .unwrap_err();
    assert!(error.to_string().contains("ambiguous"));
    let summary = extractor.finish(0);
    assert_eq!(summary.outcome(), NextestOutcome::TestFailed);
    assert!(!summary.complete());
    assert!(
        summary.failed().is_empty(),
        "unknown kind cannot be promoted to a rerun identity"
    );
}

#[test]
fn nextest_ignored_only_retains_native_counter_sentinel_without_misclassifying_population() {
    let mut listing = nextest_listing(&[("case::first", "first", "test")]);
    let suite = listing.rust_suites.values_mut().next().unwrap();
    let case = suite.test_cases.values_mut().next().unwrap();
    case.ignored = true;
    assert!(
        NextestExtractor::from_listing(0, &listing, crate::selection::IgnoredPolicy::Default)
            .is_err()
    );
    let mut extractor =
        NextestExtractor::from_listing(0, &listing, crate::selection::IgnoredPolicy::Only).unwrap();
    let mut terminal = suite_end("first", "test", false);
    terminal["ignored"] = json!(1);
    terminal["filtered_out"] = json!(u64::MAX);
    let records = feed_nextest(
        &mut extractor,
        &[
            suite_start("first", "test"),
            test_event("case::first$same_name", "started"),
            test_event("case::first$same_name", "ok"),
            terminal,
        ],
    );
    let NextestRecord::Suite { filtered_out, .. } = &records[3] else {
        panic!("suite")
    };
    assert_eq!(*filtered_out, Some(u64::MAX));
    assert_eq!(extractor.finish(0).outcome(), NextestOutcome::Succeeded);
}

#[test]
fn nextest_malformed_lost_duplicate_or_unsupported_protocol_cannot_pass() {
    let listing = nextest_listing(&[("case::first", "first", "test")]);
    let records = [
        suite_start("first", "test"),
        test_event("case::first$same_name", "started"),
        test_event("case::first$same_name", "ok"),
        suite_end("first", "test", false),
    ];
    let mut lost = nextest_extractor(&listing);
    feed_nextest(&mut lost, &records);
    lost.mark_incomplete();
    assert_eq!(lost.finish(0).outcome(), NextestOutcome::Incomplete);
    for malformed in [
        json!({"type":"test","event":"future","name":"case::first$same_name"}),
        json!({"type":"suite","event":"started","test_count":1,"nextest":{"crate":"case","test_binary":"first","kind":"test","stress_index":0}}),
    ] {
        let mut extractor = nextest_extractor(&listing);
        feed_nextest(&mut extractor, &records);
        assert!(
            extractor
                .consume(
                    &serde_json::to_vec(&malformed).unwrap(),
                    origin(4),
                    &RedactionPolicy::default()
                )
                .is_err()
        );
        assert_eq!(extractor.finish(0).outcome(), NextestOutcome::Incomplete);
    }
    let mut duplicate = nextest_extractor(&listing);
    feed_nextest(&mut duplicate, &records);
    assert!(
        duplicate
            .consume(
                &serde_json::to_vec(&records[3]).unwrap(),
                origin(4),
                &RedactionPolicy::default()
            )
            .is_err()
    );
    assert_eq!(duplicate.finish(0).outcome(), NextestOutcome::Incomplete);
}

#[test]
fn actual_pinned_nextest_stream_preserves_population_failure_retries_and_setup_exit() {
    use std::{fs, path::Path, process::Command};
    let source = tempfile::tempdir().unwrap();
    fs::create_dir(source.path().join("src")).unwrap();
    fs::create_dir(source.path().join("tests")).unwrap();
    fs::write(
        source.path().join("Cargo.toml"),
        "[package]\nname=\"native-case\"\nversion=\"0.0.0\"\nedition=\"2024\"\n",
    )
    .unwrap();
    fs::write(
        source.path().join("src/lib.rs"),
        "#[test] fn same_name() {}\n",
    )
    .unwrap();
    fs::write(
        source.path().join("tests/first.rs"),
        "#[test] fn same_name() { panic!(\"retained native failure\"); }\n",
    )
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    fs::copy(
        root.join("rust-toolchain.toml"),
        source.path().join("rust-toolchain.toml"),
    )
    .unwrap();
    let cargo = crate::correctness::real_cargo().unwrap();
    let command = || {
        let mut command = Command::new(&cargo);
        command
            .current_dir(source.path())
            .arg("nextest")
            .env("NEXTEST_EXPERIMENTAL_LIBTEST_JSON", "1");
        command
    };
    let version = command().arg("--version").output().unwrap();
    assert!(
        String::from_utf8(version.stdout)
            .unwrap()
            .starts_with("cargo-nextest 0.9.146 ")
    );
    let listed = command()
        .args(["list", "--message-format", "json"])
        .output()
        .unwrap();
    assert!(
        listed.status.success(),
        "{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let listing: nextest_metadata::TestListSummary =
        serde_json::from_slice(&listed.stdout).unwrap();
    let mut extractor = nextest_extractor(&listing);
    let run = command()
        .args([
            "run",
            "--message-format",
            "libtest-json-plus",
            "--message-format-version",
            "0.1",
            "--cargo-message-format",
            "json",
            "--retries",
            "1",
            "--no-fail-fast",
        ])
        .output()
        .unwrap();
    assert_eq!(
        run.status.code(),
        Some(nextest_metadata::NextestExitCode::TEST_RUN_FAILED)
    );
    let mut failed_identity = None;
    let mut byte_offset = 0;
    for (ordinal, line) in run
        .stdout
        .split_inclusive(|byte| *byte == b'\n')
        .enumerate()
    {
        let record = extractor
            .consume(
                line,
                RecordOrigin {
                    stream: OutputStream::Stdout,
                    ordinal: ordinal as u64,
                    byte_offset,
                    byte_length: line.len() as u64,
                },
                &RedactionPolicy::default(),
            )
            .unwrap();
        byte_offset += line.len() as u64;
        if let NextestRecord::Test {
            identity,
            event: NextestTestEvent::Failed,
            output,
            attempts,
            ..
        } = record
        {
            assert_eq!(attempts, Some(2));
            assert!(output.unwrap().contains("tests/first.rs:1:"));
            failed_identity = Some(identity);
        }
    }
    let summary = extractor.finish(run.status.code().unwrap());
    assert_eq!(summary.outcome(), NextestOutcome::TestFailed);
    assert_eq!(summary.selected(), 2);
    assert_eq!(summary.completed(), 2);
    assert!(summary.complete(), "{summary:?}");
    assert_eq!(summary.failed(), &[failed_identity.unwrap()]);
    fs::create_dir(source.path().join(".config")).unwrap();
    fs::write(source.path().join(".config/nextest.toml"), "experimental=[\"setup-scripts\"]\n[scripts.setup.failure]\ncommand=\"sh -c 'exit 7'\"\n[[profile.default.scripts]]\nfilter=\"all()\"\nsetup=\"failure\"\n").unwrap();
    let setup_run = command()
        .args([
            "run",
            "--message-format",
            "libtest-json-plus",
            "--message-format-version",
            "0.1",
        ])
        .output()
        .unwrap();
    assert_eq!(
        setup_run.status.code(),
        Some(nextest_metadata::NextestExitCode::SETUP_SCRIPT_FAILED),
        "{}",
        String::from_utf8_lossy(&setup_run.stderr)
    );
    assert!(
        setup_run.stdout.is_empty(),
        "pinned libtest reporter emits no setup events"
    );
    let summary = nextest_extractor(&listing).finish(setup_run.status.code().unwrap());
    assert_eq!(summary.outcome(), NextestOutcome::SetupFailed);
    assert_eq!(summary.setup(), NextestSetup::FailedByRunner);
    assert_eq!(summary.completed(), 0);
}
