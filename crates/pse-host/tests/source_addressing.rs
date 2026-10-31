// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent addressed-source journeys through the actual native command boundary.
#![cfg(test)]
use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStringExt as _,
    path::Path,
    process::{Command, Output},
};

fn pse(root: &Path) -> Command {
    let mut command = Command::new(
        std::env::var_os("CARGO_BIN_EXE_pse")
            .expect("Cargo/nextest supplies the selected native executable"),
    );
    command.current_dir(root).env("PSE_FORMAT", "json");
    command
}
fn value(output: Output) -> serde_json::Value {
    assert!(output.status.success(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}
fn git(root: &Path, arguments: &[&OsStr]) {
    let output = Command::new("git")
        .current_dir(root)
        .args([
            "-c",
            "user.name=addressed source fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .args(arguments)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
}
fn words(root: &Path, arguments: &[&str]) {
    git(root, &arguments.iter().map(OsStr::new).collect::<Vec<_>>());
}
fn schema(root: &Path, command: &str, response: &serde_json::Value) {
    let schema = value(pse(root).args(["schema", command]).output().unwrap());
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(response)
        .unwrap();
}

#[test]
fn working_staged_and_committed_unicode_sources_are_selected_without_checkout_preparation() {
    let root = tempfile::tempdir().unwrap();
    words(root.path(), &["init", "--quiet", "--initial-branch=main"]);
    let path = OsString::from_vec(b"source-\xff.rs".to_vec());
    std::fs::write(root.path().join(&path), "fn committed() { let α = 1; }\n").unwrap();
    git(root.path(), &[OsStr::new("add"), OsStr::new("--"), &path]);
    words(root.path(), &["commit", "--quiet", "-m", "original"]);
    std::fs::write(root.path().join(&path), "fn staged() { let β = 2; }\n").unwrap();
    git(root.path(), &[OsStr::new("add"), OsStr::new("--"), &path]);
    std::fs::write(root.path().join(&path), "fn working() { let γ = 3; }\n").unwrap();
    let index_before = std::fs::read(root.path().join(".git/index")).unwrap();
    let working = value(
        pse(root.path())
            .args(["inspect", "declaration"])
            .arg(&path)
            .args(["--at-line", "1", "--at-column", "4"])
            .output()
            .unwrap(),
    );
    let staged = value(
        pse(root.path())
            .args(["inspect", "declaration"])
            .arg(&path)
            .args(["--index", "--at-line", "1", "--at-column", "4"])
            .output()
            .unwrap(),
    );
    let committed = value(
        pse(root.path())
            .args(["inspect", "declaration"])
            .arg(&path)
            .args(["--revision", "HEAD", "--at-line", "1", "--at-column", "4"])
            .output()
            .unwrap(),
    );
    for response in [&working, &staged, &committed] {
        schema(root.path(), "inspect declaration", response);
    }
    assert_eq!(
        working["payload"]["selection"]["text"],
        "fn working() { let γ = 3; }"
    );
    assert_eq!(
        staged["payload"]["selection"]["text"],
        "fn staged() { let β = 2; }"
    );
    assert_eq!(
        committed["payload"]["selection"]["text"],
        "fn committed() { let α = 1; }"
    );
    assert_eq!(
        working["payload"]["locator"]["source"]["origin"]["kind"],
        "working"
    );
    assert_eq!(
        staged["payload"]["locator"]["source"]["origin"]["kind"],
        "index"
    );
    assert_eq!(
        committed["payload"]["locator"]["source"]["origin"]["kind"],
        "blob"
    );
    let projected = value(
        pse(root.path())
            .args(["inspect", "declaration"])
            .arg(&path)
            .args(["--index", "--at-byte", "4", "--fields", "selection"])
            .output()
            .unwrap(),
    );
    assert!(projected["payload"].get("locator").is_none());
    let provider = &projected["providers"][0];
    let identity: serde_json::Value =
        serde_json::from_str(provider["context"].as_str().unwrap()).unwrap();
    assert_eq!(identity, staged["payload"]["locator"]["source"]);
    assert_eq!(provider["analysis_generation"], identity["source_sha256"]);
    assert!(
        provider["provider"]
            .as_str()
            .unwrap()
            .contains("ra_ap_syntax/0.0.360")
    );
    assert_eq!(provider["coordinates"][0]["start"], 0);
    assert_ne!(
        committed["payload"]["locator"]["source"]["origin"]["revision"],
        "HEAD"
    );
    // A committed source remains readable when there is no working file to canonicalize.
    std::fs::remove_file(root.path().join(&path)).unwrap();
    let removed = value(
        pse(root.path())
            .args(["inspect", "outline"])
            .arg(&path)
            .args(["--revision", "HEAD"])
            .output()
            .unwrap(),
    );
    assert_eq!(
        removed["payload"]["outline"]["entries"][0]["path"][0],
        "committed"
    );
    assert_eq!(
        std::fs::read(root.path().join(".git/index")).unwrap(),
        index_before
    );
    assert!(!root.path().join("target").exists());
    assert!(!root.path().join(".config").exists());
}

#[test]
fn nested_declaration_pages_retain_coordinates_and_refuse_changed_working_source() {
    let root = tempfile::tempdir().unwrap();
    let source = "mod nested { struct S; impl S { fn first() {} fn second() {} } }\n";
    std::fs::write(root.path().join("nested.rs"), source).unwrap();
    let first = value(
        pse(root.path())
            .args(["inspect", "outline", "nested.rs", "--budget", "2"])
            .output()
            .unwrap(),
    );
    assert_eq!(
        first["payload"]["outline"]["entries"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(first["payload"]["page"]["total_records"], 5);
    assert_eq!(first["payload"]["page"]["omitted_records"], 3);
    assert_eq!(first["view"]["omitted_records"], 3);
    let cursor = first["payload"]["page"]["next_cursor"].as_str().unwrap();
    let second = value(
        pse(root.path())
            .args([
                "inspect",
                "outline",
                "nested.rs",
                "--budget",
                "2",
                "--cursor",
                cursor,
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(second["payload"]["page"]["offset"], 2);
    assert_eq!(
        second["payload"]["outline"]["entries"][1]["path"],
        serde_json::json!(["nested", "S", "first"])
    );
    let final_cursor = second["payload"]["page"]["next_cursor"].as_str().unwrap();
    let final_page = value(
        pse(root.path())
            .args([
                "inspect",
                "outline",
                "nested.rs",
                "--budget",
                "2",
                "--cursor",
                final_cursor,
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(
        final_page["payload"]["outline"]["entries"][0]["path"],
        serde_json::json!(["nested", "S", "second"])
    );
    assert!(final_page["payload"]["page"]["next_cursor"].is_null());
    std::fs::write(
        root.path().join("nested.rs"),
        format!("// source edit\n{source}"),
    )
    .unwrap();
    let stale = pse(root.path())
        .args(["inspect", "outline", "nested.rs", "--cursor", cursor])
        .output()
        .unwrap();
    assert!(!stale.status.success());
    let failure: serde_json::Value = serde_json::from_slice(&stale.stderr).unwrap();
    assert!(failure["failures"].to_string().contains("source changed"));
}

#[test]
fn unicode_scalar_positions_ranges_and_batch_budget_use_original_bytes() {
    let root = tempfile::tempdir().unwrap();
    let source =
        "impl A for S { fn café() {} }\nimpl B for S { fn café() { let β = 2; } }\nfn broken(\n";
    std::fs::write(root.path().join("duplicate.rs"), source).unwrap();
    let selected = value(
        pse(root.path())
            .args([
                "inspect",
                "declaration",
                "duplicate.rs",
                "--at-line",
                "2",
                "--at-column",
                "31",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(
        selected["payload"]["selection"]["text"],
        "fn café() { let β = 2; }"
    );
    assert_eq!(selected["payload"]["selection"]["range"]["line_start"], 2);
    assert!(
        !selected["payload"]["parse_issues"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let enclosing = value(
        pse(root.path())
            .args([
                "inspect",
                "declaration",
                "duplicate.rs",
                "--at-line",
                "2",
                "--at-column",
                "41",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(
        enclosing["payload"]["selection"]["text"],
        "impl B for S { fn café() { let β = 2; } }"
    );
    let start = source.find('β').unwrap();
    let selected = value(
        pse(root.path())
            .args([
                "inspect",
                "declaration",
                "duplicate.rs",
                "--range-start",
                &start.to_string(),
                "--range-end",
                &(start + 2).to_string(),
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(selected["payload"]["selection"]["text"], "β");
    let invalid = pse(root.path())
        .args([
            "inspect",
            "declaration",
            "duplicate.rs",
            "--range-start",
            &(start + 1).to_string(),
            "--range-end",
            &(start + 2).to_string(),
        ])
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    std::fs::write(root.path().join("last.rs"), "fn last() {}\n").unwrap();
    let output = pse(root.path())
        .args([
            "inspect",
            "outline",
            "duplicate.rs",
            "missing.rs",
            "last.rs",
            "--budget",
            "2",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let batch: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let records = batch["payload"]["records"].as_array().unwrap();
    assert_eq!(records.len(), 3);
    assert_eq!(
        records[0]["reading"]["outline"]["entries"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(records[1]["status"], "failed");
    assert_eq!(
        records[2]["reading"]["outline"]["entries"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(batch["failures"].as_array().unwrap().len(), 1);
}
