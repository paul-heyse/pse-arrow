// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive artifact continuation through the authoritative CLI and native backend.
#![cfg(test)]
use base64::Engine as _;
use std::{path::Path, process::Command};
fn pse(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_pse"));
    command
        .current_dir(root)
        .env("PSE_FORMAT", "json")
        .env("XDG_STATE_HOME", root.join("uncreated-state"));
    command
}
fn value(output: std::process::Output) -> serde_json::Value {
    assert!(output.status.success(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}
#[test]
fn default_prefix_continues_under_the_native_validator_without_writing_state() {
    let root = tempfile::tempdir().unwrap();
    let source = "abcde".repeat(1200);
    let file = root.path().join("opaque.json");
    std::fs::write(&file, &source).unwrap();
    let first = value(
        pse(root.path())
            .args(["artifact", "read", "opaque.json"])
            .output()
            .unwrap(),
    );
    assert_eq!(
        first["payload"]["byte_range"],
        serde_json::json!({"start":0,"end":4096})
    );
    assert_eq!(first["payload"]["whole_object"], false);
    assert_eq!(first["payload"]["content"]["encoding"], "utf8");
    assert_eq!(first["payload"]["content"]["data"], &source[..4096]);
    assert_eq!(first["payload"]["next_offset"], 4096);
    assert_eq!(first["payload"]["atomic_snapshot"], false);
    let schema = value(
        pse(root.path())
            .args(["schema", "artifact", "read"])
            .output()
            .unwrap(),
    );
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(&first)
        .unwrap();
    let argv = first["next"][0]["argv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|arg| arg.as_str().unwrap())
        .collect::<Vec<_>>();
    let continuation = value(pse(root.path()).args(argv).output().unwrap());
    assert_eq!(continuation["payload"]["content"]["data"], &source[4096..]);
    assert!(continuation["payload"]["next_offset"].is_null());
    assert_eq!(continuation["payload"]["whole_object"], false);
    let full = value(
        pse(root.path())
            .args(["artifact", "read", "opaque.json", "--all"])
            .output()
            .unwrap(),
    );
    assert_eq!(full["payload"]["whole_object"], true);
    assert_eq!(full["payload"]["content"]["data"], source);
    assert!(!root.path().join("uncreated-state").exists());
    assert!(!root.path().join("target").exists());
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
}
#[test]
fn condition_mismatch_and_invalid_extent_preserve_structured_native_failures() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("evidence"), b"source").unwrap();
    let mismatch = pse(root.path())
        .args([
            "artifact",
            "read",
            "evidence",
            "--if-match",
            "deliberately-wrong",
        ])
        .output()
        .unwrap();
    assert_eq!(mismatch.status.code(), Some(125));
    let error: serde_json::Value = serde_json::from_slice(&mismatch.stderr).unwrap();
    assert!(
        !error["failures"][0]["causes"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(error.to_string().to_lowercase().contains("precondition"));
    for args in [
        vec!["artifact", "read", "missing"],
        vec!["artifact", "read", "evidence", "--offset", "999"],
        vec![
            "artifact",
            "read",
            "evidence",
            "--offset",
            "18446744073709551615",
            "--length",
            "1",
        ],
    ] {
        let output = pse(root.path()).args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(125));
        let failure: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(failure["outcome"], "boundary_failure");
    }
    for args in [
        vec!["artifact", "read", "evidence", "--length", "0"],
        vec!["artifact", "read", "evidence", "--length", "2", "--all"],
    ] {
        assert_eq!(
            pse(root.path()).args(args).output().unwrap().status.code(),
            Some(2)
        );
    }
    assert!(!root.path().join("uncreated-state").exists());
}
#[test]
fn binary_and_json_looking_text_remain_exact_opaque_content() {
    let root = tempfile::tempdir().unwrap();
    let bytes = [0, 255, b'\n', 1];
    std::fs::write(root.path().join("binary"), bytes).unwrap();
    let response = value(
        pse(root.path())
            .args(["artifact", "read", "binary", "--all"])
            .output()
            .unwrap(),
    );
    assert_eq!(response["payload"]["content"]["encoding"], "base64");
    assert_eq!(
        base64::engine::general_purpose::STANDARD
            .decode(response["payload"]["content"]["data"].as_str().unwrap())
            .unwrap(),
        bytes
    );
    let source = "{ \"duplicate\": 1, \"duplicate\": 1e+0002 }\n";
    std::fs::write(root.path().join("raw.json"), source).unwrap();
    let response = value(
        pse(root.path())
            .args(["artifact", "read", "raw.json", "--all"])
            .output()
            .unwrap(),
    );
    assert_eq!(response["payload"]["content"]["data"], source);
    let human = pse(root.path())
        .args(["artifact", "read", "binary", "--format", "human"])
        .output()
        .unwrap();
    assert!(human.status.success());
    let text = String::from_utf8(human.stdout).unwrap();
    assert!(text.contains("base64 selected bytes"));
    assert!(text.contains("selected_sha256="));
}
#[test]
fn unsupported_native_filename_bytes_are_refused_instead_of_substituted() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt as _};
    let root = tempfile::tempdir().unwrap();
    let name = OsString::from_vec(b"file-\xff".to_vec());
    std::fs::write(root.path().join(&name), b"data").unwrap();
    let output = pse(root.path())
        .args(["artifact", "read"])
        .arg(name)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(125));
    let failure: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(failure["outcome"], "boundary_failure");
    assert!(
        !failure["failures"][0]["causes"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(!root.path().join("uncreated-state").exists());
}
