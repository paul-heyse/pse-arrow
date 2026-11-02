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

#[test]
fn source_detail_defaults_expansion_and_cursor_refusal_follow_actual_schemas() {
    let root = tempfile::tempdir().unwrap();
    let source = format!(
        "//! file docs\n/// {}\nconst BIG: &str = \"{}\";\n/// selected docs\nfn selected() {{ let s = \"a\\nβ\"; fn nested() {{ let n = \"nested\"; }} }}",
        "d".repeat(100_000),
        "x".repeat(100_000)
    );
    std::fs::write(root.path().join("detail.rs"), &source).unwrap();
    let outline = value(
        pse(root.path())
            .args(["inspect", "outline", "detail.rs", "--budget", "1"])
            .output()
            .unwrap(),
    );
    schema(root.path(), "inspect outline", &outline);
    assert!(serde_json::to_vec(&outline).unwrap().len() < 8_000);
    assert_eq!(
        outline["payload"]["outline"]["entries"][0]["signature"],
        "const BIG: &str"
    );
    assert_eq!(
        outline["payload"]["outline"]["facts"],
        serde_json::json!([])
    );
    assert_eq!(
        outline["payload"]["outline"]["detail_coverage"]["detail"],
        "navigation"
    );
    let cursor = outline["payload"]["page"]["next_cursor"].as_str().unwrap();
    let mismatch = pse(root.path())
        .args([
            "inspect",
            "outline",
            "detail.rs",
            "--cursor",
            cursor,
            "--detail",
            "facts",
        ])
        .output()
        .unwrap();
    assert!(!mismatch.status.success());
    let declaration = value(
        pse(root.path())
            .args(["inspect", "declaration", "detail.rs::selected"])
            .output()
            .unwrap(),
    );
    schema(root.path(), "inspect declaration", &declaration);
    assert_eq!(declaration["payload"]["detail_coverage"]["detail"], "facts");
    let facts = declaration["payload"]["facts"].as_array().unwrap();
    assert_eq!(facts.len(), 2);
    assert!(facts.iter().all(|fact| fact["mapping"].is_null()));
    assert_eq!(facts[0]["written"], "/// selected docs");
    assert_eq!(facts[1]["value"]["value"], "a\nβ");
    let mapped = value(
        pse(root.path())
            .args([
                "inspect",
                "declaration",
                "detail.rs::selected",
                "--detail",
                "mapped",
            ])
            .output()
            .unwrap(),
    );
    schema(root.path(), "inspect declaration", &mapped);
    let mappings = mapped["payload"]["facts"][1]["mapping"].as_array().unwrap();
    assert_eq!(mappings.len(), 3);
    let newline = &mappings[1]["source"];
    assert_eq!(
        &source[newline["byte_start"].as_u64().unwrap() as usize
            ..newline["byte_end"].as_u64().unwrap() as usize],
        "\\n"
    );
    let invalid = pse(root.path())
        .args(["inspect", "outline", "detail.rs", "--detail", "unknown"])
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    assert!(!root.path().join("target").exists());
}

#[test]
fn explicit_range_facts_intersect_complete_original_literal_and_plain_range_stays_text() {
    let root = tempfile::tempdir().unwrap();
    let source = "fn selected() { let s = \"abcdef\"; }";
    std::fs::write(root.path().join("range.rs"), source).unwrap();
    let start = (source.find("abcdef").unwrap() + 1).to_string();
    let end = (source.find("abcdef").unwrap() + 2).to_string();
    let args = [
        "inspect",
        "declaration",
        "range.rs",
        "--range-start",
        &start,
        "--range-end",
        &end,
    ];
    let raw = value(pse(root.path()).args(args).output().unwrap());
    assert_eq!(raw["payload"]["selection"]["text"], "b");
    assert_eq!(raw["payload"]["detail_coverage"]["detail"], "navigation");
    assert_eq!(raw["payload"]["facts"], serde_json::json!([]));
    let selected = value(
        pse(root.path())
            .args(args)
            .args(["--detail", "mapped"])
            .output()
            .unwrap(),
    );
    schema(root.path(), "inspect declaration", &selected);
    assert_eq!(selected["payload"]["facts"][0]["written"], "\"abcdef\"");
    assert_eq!(
        selected["payload"]["facts"][0]["mapping"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert_eq!(
        selected["payload"]["detail_coverage"]["scope"],
        "intersecting_range"
    );
}

#[test]
fn explicit_outline_detail_and_projection_remain_independent_in_single_and_batch_responses() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("first.rs"),
        "//! root docs\nfn first() { let s = \"a\\nβ\"; }",
    )
    .unwrap();
    std::fs::write(
        root.path().join("second.rs"),
        "fn second() { let s = \"second\"; }",
    )
    .unwrap();
    let facts = value(
        pse(root.path())
            .args(["inspect", "outline", "first.rs", "--detail", "facts"])
            .output()
            .unwrap(),
    );
    schema(root.path(), "inspect outline", &facts);
    assert_eq!(
        facts["payload"]["outline"]["detail_coverage"]["detail"],
        "facts"
    );
    assert!(
        facts["payload"]["outline"]["detail_coverage"]["includes_file_facts"]
            .as_bool()
            .unwrap()
    );
    let file_facts = facts["payload"]["outline"]["facts"].as_array().unwrap();
    assert_eq!(file_facts.len(), 2);
    assert!(file_facts[0]["owner"].is_null());
    assert!(file_facts.iter().all(|fact| fact["mapping"].is_null()));
    let mapped = value(
        pse(root.path())
            .args(["inspect", "outline", "first.rs", "--detail", "mapped"])
            .output()
            .unwrap(),
    );
    schema(root.path(), "inspect outline", &mapped);
    assert_eq!(
        mapped["payload"]["outline"]["facts"][1]["mapping"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let projected = value(
        pse(root.path())
            .args([
                "inspect",
                "outline",
                "first.rs",
                "--detail",
                "mapped",
                "--fields",
                "outline.facts",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(
        projected["payload"]["outline"]["facts"],
        mapped["payload"]["outline"]["facts"]
    );
    assert!(projected["payload"]["outline"].get("entries").is_none());
    assert_eq!(projected["providers"], mapped["providers"]);
    let navigation = value(
        pse(root.path())
            .args([
                "inspect",
                "outline",
                "first.rs",
                "--fields",
                "outline.facts",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(
        navigation["payload"]["outline"]["facts"],
        serde_json::json!([])
    );
    let batch = value(
        pse(root.path())
            .args([
                "inspect",
                "outline",
                "first.rs",
                "second.rs",
                "--detail",
                "mapped",
            ])
            .output()
            .unwrap(),
    );
    schema(root.path(), "inspect outline batch", &batch);
    let records = batch["payload"]["records"].as_array().unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(
        records[0]["reading"]["outline"]["detail_coverage"]["detail"],
        "mapped"
    );
    assert_eq!(
        records[1]["reading"]["outline"]["facts"][0]["mapping"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert!(!root.path().join("target").exists());
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
