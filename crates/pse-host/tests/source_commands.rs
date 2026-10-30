// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Same-snapshot structural reading through the actual public parser and binary.
#![cfg(test)]
use std::{ffi::OsString, os::unix::ffi::OsStringExt as _, path::Path, process::Command};
fn pse(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_pse"));
    command.current_dir(root).env("PSE_FORMAT", "json");
    command
}
fn value(output: std::process::Output) -> serde_json::Value {
    assert!(output.status.success(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}
fn validate(root: &Path, verb: &str, response: &serde_json::Value) {
    let schema = value(pse(root).args(["schema", verb]).output().unwrap());
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(response)
        .unwrap();
}
#[test]
fn malformed_rust_keeps_exact_unicode_method_with_context_and_schema() {
    let root = tempfile::tempdir().unwrap();
    let source = "struct HostManager;\nimpl HostManager {\n/// café\n#[inline]\nfn observe(&self) { /* λ */ }\n}\nfn broken(\n";
    std::fs::write(root.path().join("manager.rs"), source).unwrap();
    let response = value(
        pse(root.path())
            .args(["show", "manager.rs::HostManager::observe"])
            .output()
            .unwrap(),
    );
    validate(root.path(), "show", &response);
    assert_eq!(response["observation"]["completeness"], "partial");
    assert!(
        !response["payload"]["parse_issues"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        !response["payload"]
            .as_object()
            .unwrap()
            .contains_key("outline")
    );
    let selected = &response["payload"]["selection"];
    let start = selected["range"]["byte_start"].as_u64().unwrap() as usize;
    let end = selected["range"]["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(selected["text"], &source[start..end]);
    assert!(selected["text"].as_str().unwrap().starts_with("/// café"));
    let output = pse(root.path())
        .args([
            "show",
            "manager.rs::HostManager::observe",
            "--format",
            "human",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("3 | /// café"));
    assert!(text.contains("parse issue"));
    assert!(text.contains("syntactic only"));
    assert!(!root.path().join("target").exists());
}
#[test]
fn duplicate_paths_refuse_until_explicit_original_byte_selection() {
    let root = tempfile::tempdir().unwrap();
    let source = "impl A for S { fn act() {} }\nimpl B for S { fn act() {} }";
    std::fs::write(root.path().join("ambiguous.rs"), source).unwrap();
    let output = pse(root.path())
        .args(["show", "ambiguous.rs::S::act"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(125));
    let failure: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert!(failure["failures"].to_string().contains("2 declarations"));
    let at = source.rfind("fn act").unwrap().to_string();
    let response = value(
        pse(root.path())
            .args(["show", "ambiguous.rs::S::act", "--at-byte", &at])
            .output()
            .unwrap(),
    );
    assert_eq!(response["payload"]["selection"]["range"]["line_start"], 2);
    let missing = pse(root.path())
        .args(["show", "ambiguous.rs::S::missing"])
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(
        String::from_utf8(missing.stderr)
            .unwrap()
            .contains("no declaration")
    );
}
#[test]
fn commonmark_sections_and_explicit_literal_items_replace_line_grep() {
    let root = tempfile::tempdir().unwrap();
    let source = "---\ntitle: metadata\n---\n# Root\n```\n# fake\n```\nChild::heading\n-------------\ntext\n";
    std::fs::write(root.path().join("plan.md"), source).unwrap();
    let outline = value(
        pse(root.path())
            .args(["outline", "plan.md"])
            .output()
            .unwrap(),
    );
    validate(root.path(), "outline", &outline);
    assert_eq!(
        outline["payload"]["outline"]["entries"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        outline["payload"]["outline"]["entries"][1]["range"]["line_start"],
        8
    );
    let response = value(
        pse(root.path())
            .args([
                "show",
                "plan.md",
                "--item",
                "Root",
                "--item",
                "Child::heading",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(
        response["payload"]["selection"]["text"],
        "Child::heading\n-------------\ntext\n"
    );
    let discovery = value(pse(root.path()).args(["help", "show"]).output().unwrap());
    assert_eq!(
        discovery["payload"]["records"][0]["output_schemas"]["availability"],
        "available"
    );
}
#[test]
fn native_filename_bytes_and_explicit_edition_survive_source_reads() {
    let root = tempfile::tempdir().unwrap();
    let file = OsString::from_vec(vec![b'x', 0xff, b'.', b'r', b's']);
    std::fs::write(root.path().join(&file), "fn async() {}\n").unwrap();
    let response = value(
        pse(root.path())
            .arg("outline")
            .arg(&file)
            .args(["--edition", "2015"])
            .output()
            .unwrap(),
    );
    validate(root.path(), "outline", &response);
    assert_eq!(
        response["payload"]["file"],
        serde_json::json!({"Unix":[120,255,46,114,115]})
    );
    assert!(
        response["payload"]["outline"]["parse_issues"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let response = value(pse(root.path()).arg("outline").arg(file).output().unwrap());
    assert!(
        !response["payload"]["outline"]["parse_issues"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    std::fs::write(root.path().join("plain"), "# title").unwrap();
    let output = pse(root.path())
        .args(["outline", "plain"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(125));
    let response = value(
        pse(root.path())
            .args(["outline", "plain", "--language", "markdown"])
            .output()
            .unwrap(),
    );
    assert_eq!(
        response["payload"]["outline"]["entries"][0]["path"][0],
        "title"
    );
}

#[test]
fn multiple_files_use_the_shared_cpu_batch_and_keep_partial_read_results() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("first.rs"), "fn first() {}\n").unwrap();
    std::fs::write(root.path().join("last.md"), "# Last\n").unwrap();
    let output = pse(root.path())
        .args(["outline", "first.rs", "missing.rs", "last.md"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    validate(root.path(), "outline batch", &response);
    assert_eq!(response["command"], "outline batch");
    assert_eq!(response["payload"]["cpu_work_completed"], true);
    let records = response["payload"]["records"].as_array().unwrap();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0]["status"], "read");
    assert_eq!(
        records[0]["reading"]["outline"]["entries"][0]["path"][0],
        "first"
    );
    assert_eq!(records[1]["status"], "failed");
    assert_eq!(
        records[2]["reading"]["outline"]["entries"][0]["path"][0],
        "Last"
    );
    assert_eq!(response["failures"].as_array().unwrap().len(), 1);
    assert!(!root.path().join("target").exists());
}
