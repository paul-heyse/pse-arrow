// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public offline schema investigations retain exact acquired evidence.
use serde_json::Value;
use std::{
    path::PathBuf,
    process::{Command, Output},
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

fn cli(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pse"))
        .current_dir(root())
        .args(["--format", "json"])
        .args(arguments)
        .output()
        .unwrap()
}

fn value(output: Output) -> Value {
    assert!(output.status.success(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

const DESCRIPTOR: &str = "crates/pse-host/src/generated/plan61/descriptor.bin";

#[test]
fn offline_schema_search_expansion_and_stale_reference() {
    let temporary = tempfile::tempdir().unwrap();
    let descriptor = temporary.path().join("schema.bin");
    std::fs::copy(root().join(DESCRIPTOR), &descriptor).unwrap();
    let path = descriptor.to_str().unwrap();
    let found = value(cli(&["search", "schema", "Payload", "--descriptor", path]));
    let rows = found["payload"]["results"].as_array().unwrap();
    assert!(!rows.is_empty());
    let reference = serde_json::to_string(&rows[0]["reference"]).unwrap();
    let expanded = value(cli(&["inspect", "--result", &reference]));
    assert_eq!(expanded["command"], "inspect");
    let mut bytes = std::fs::read(&descriptor).unwrap();
    // A valid unknown descriptor-set field changes artifact identity without
    // making the new descriptor malformed.
    bytes.extend([0xa0, 0x06, 0x01]);
    std::fs::write(&descriptor, bytes).unwrap();
    assert!(!cli(&["inspect", "--result", &reference]).status.success());
}

#[test]
fn schema_and_payload_output_schemas_are_public() {
    for command in ["inspect schema", "inspect payload"] {
        let output = cli(&["schema", command]);
        assert!(output.status.success(), "{command}: {output:?}");
        let _: Value = serde_json::from_slice(&output.stdout).unwrap();
    }
}

#[test]
fn missing_buf_is_partial_failure_with_structural_results() {
    let temporary = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_pse"))
        .current_dir(root())
        .env("PATH", temporary.path())
        .args([
            "--format",
            "json",
            "inspect",
            "schema",
            "--descriptor",
            DESCRIPTOR,
            "--compare-with",
            DESCRIPTOR,
        ])
        .output()
        .unwrap();
    assert!(!output.status.success(), "{output:?}");
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["outcome"], "failure");
    assert_eq!(report["observation"]["completeness"], "partial");
    assert_eq!(report["failures"].as_array().unwrap().len(), 3);
    assert_eq!(report["payload"]["differences"], serde_json::json!([]));
}

#[test]
fn payload_ingress_refuses_malformed_wire_duplicate_json_and_unknown_text() {
    let temporary = tempfile::tempdir().unwrap();
    for (encoding, input) in [
        ("binary", b"\x0a\x05x".as_slice()),
        ("json", br#"{"count":"1","count":"2"}"#.as_slice()),
        ("textproto", b"not_an_authored_field: 3".as_slice()),
    ] {
        let file = temporary.path().join(encoding);
        std::fs::write(&file, input).unwrap();
        let output = cli(&[
            "inspect",
            "payload",
            "--descriptor",
            DESCRIPTOR,
            "--message",
            "plan61.qualification.v1.Payload",
            "--input",
            file.to_str().unwrap(),
            "--encoding",
            encoding,
        ]);
        assert!(
            !output.status.success(),
            "{encoding} unexpectedly accepted: {output:?}"
        );
    }
}

#[cfg(unix)]
#[test]
fn native_artifact_paths_preserve_non_utf8_bytes() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let temporary = tempfile::tempdir().unwrap();
    let descriptor = temporary
        .path()
        .join(OsString::from_vec(b"schema-\xff.bin".to_vec()));
    let input = temporary
        .path()
        .join(OsString::from_vec(b"payload-\xfe.bin".to_vec()));
    std::fs::copy(root().join(DESCRIPTOR), &descriptor).unwrap();
    std::fs::write(&input, [0x08, 0]).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_pse"))
        .current_dir(root())
        .args(["--format", "json", "inspect", "payload", "--descriptor"])
        .arg(&descriptor)
        .args(["--message", "plan61.qualification.v1.Payload", "--input"])
        .arg(&input)
        .args(["--encoding", "binary"])
        .output()
        .unwrap();
    let report = value(output);
    assert!(report["payload"]["descriptor"].is_object());
    assert!(report["payload"]["input"].is_object());
}
