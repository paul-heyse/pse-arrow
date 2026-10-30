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
    assert_eq!(summary.outcome(), CompilerOutcome::Failed);
    assert_eq!(summary.unique_primary_failures(), Some(1));
    assert_eq!(summary.error_records(), 2);
    assert_eq!(summary.duplicates(), 1);
    assert!(summary.complete());
}

#[test]
fn zero_exit_and_a_valid_tail_cannot_erase_missing_or_malformed_required_evidence() {
    assert_eq!(
        extractor(8).finish(0).outcome(),
        CompilerOutcome::Incomplete
    );
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
    assert_eq!(summary.outcome(), CompilerOutcome::Incomplete);
    assert!(!summary.complete());
    let mut lost = extractor(8);
    lost.mark_incomplete();
    lost.consume(&terminal(true), origin(1), &policy)
        .expect("valid terminal");
    assert_eq!(
        lost.finish(7).outcome(),
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
    assert_eq!(summary.outcome(), CompilerOutcome::Succeeded);
    assert_eq!(summary.warning_records(), 2);
    assert_eq!(summary.unique_primary_failures(), None);
    assert!(
        summary.complete(),
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
    assert_eq!(extractor.finish(0).outcome(), CompilerOutcome::Incomplete);
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
    assert_eq!(summary.outcome(), CompilerOutcome::Failed);
    assert_eq!(summary.unique_primary_failures(), Some(1));
    assert!(summary.complete());
}
