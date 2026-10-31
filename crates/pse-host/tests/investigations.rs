// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public source discovery, progressive inspection and truthful native demand.
use assert_cmd::Command;
use serde_json::{Value, json};
use std::path::Path;

#[tokio::test]
async fn interrupt_settles_search_while_an_open_stdout_reader_is_stalled() {
    use tokio::io::AsyncReadExt as _;
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("input.txt"), "small input").unwrap();
    // The initial complete frame alone exceeds pipe capacity. Observing its
    // first byte proves delivery started; retain the reader without draining it.
    let request = json!({"sources":{"paths":["input.txt"]},"predicate":{"kind":"text","patterns":["z".repeat(1<<20)],"literal":true}});
    std::fs::write(
        root.path().join("query.json"),
        serde_json::to_vec(&request).unwrap(),
    )
    .unwrap();
    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_pse"))
        .current_dir(root.path())
        .args(["search", "--request", "query.json", "--format", "ndjson"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut reader = child.stdout.take().unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(10), reader.read_u8())
        .await
        .unwrap()
        .unwrap();
    let pid = rustix::process::Pid::from_raw(child.id().unwrap() as i32).unwrap();
    rustix::process::kill_process(pid, rustix::process::Signal::INT).unwrap();
    let status = tokio::time::timeout(std::time::Duration::from_secs(10), child.wait())
        .await
        .expect("open stalled stdout must not hold search after interruption")
        .unwrap();
    assert_eq!(status.code(), Some(130));
    drop(reader);
}

fn run(root: &Path, args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_pse"))
        .current_dir(root)
        .args(args)
        .args(["--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).unwrap()
}

#[test]
fn unreadable_source_retains_readable_witness_and_partial_scope() {
    use std::os::unix::fs::PermissionsExt as _;
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("readable.txt"), "needle\n").unwrap();
    let denied = root.path().join("denied.txt");
    std::fs::write(&denied, "needle\n").unwrap();
    std::fs::set_permissions(&denied, std::fs::Permissions::from_mode(0)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_pse"))
        .current_dir(root.path())
        .args(["search", "--all", "text", "needle", "--format", "json"])
        .output()
        .unwrap();
    std::fs::set_permissions(&denied, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(
        !output.status.success(),
        "unreadable scope is not complete success"
    );
    let answer: Value = serde_json::from_slice(&output.stdout).unwrap();
    let report = &answer["payload"];
    assert_eq!(report["results"].as_array().unwrap().len(), 1, "{answer}");
    assert_eq!(report["summary"]["matches"], 1, "{answer}");
    assert_eq!(report["summary"]["failures"], 1, "{answer}");
    assert_eq!(report["summary"]["complete"], false, "{answer}");
    assert_ne!(report["summary"]["count"]["precision"], "exact", "{answer}");
    assert_ne!(answer["subject_state"], "no_match", "{answer}");
    assert!(
        !report["diagnostics"].as_array().unwrap().is_empty(),
        "{answer}"
    );
}

#[test]
fn public_text_discovery_inspection_and_preview_keep_original_bytes() {
    let root = tempfile::tempdir().unwrap();
    let text = "// café must remain\nfn retained() { let answer = 17; }\n";
    std::fs::write(root.path().join("science.rs"), text).unwrap();
    let result = run(root.path(), &["search", "text", "answer"]);
    let report = &result["payload"];
    assert_eq!(report["results"].as_array().unwrap().len(), 1);
    assert_eq!(report["sources"][0]["path"], "science.rs");
    let reference: pse_host::investigation::ResultReference =
        serde_json::from_value(report["results"][0]["reference"].clone()).unwrap();
    let token = reference.encode().unwrap();
    let original = serde_json::to_string(&report["results"][0]["reference"]).unwrap();
    let inspected = run(root.path(), &["inspect", "--result", &original]);
    assert!(
        inspected["payload"]["context"]
            .as_str()
            .unwrap()
            .contains("answer")
    );
    let preview = run(
        root.path(),
        &[
            "propose",
            "--result",
            &token,
            "--replacement",
            "fn retained() { let answer = 18; }\n",
        ],
    );
    assert!(preview["payload"]["selected"].as_array().is_some());
    assert_eq!(
        std::fs::read_to_string(root.path().join("science.rs")).unwrap(),
        text
    );
    std::fs::write(root.path().join("science.rs"), "fn changed() {}\n").unwrap();
    Command::new(env!("CARGO_BIN_EXE_pse"))
        .current_dir(root.path())
        .args(["inspect", "--result", &token, "--format", "json"])
        .assert()
        .failure();
}

#[test]
fn no_match_is_success_and_exact_count_ignores_default_display_limit() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("many.txt"), "witness\n".repeat(43)).unwrap();
    let missing = run(root.path(), &["search", "text", "absent"]);
    assert_eq!(missing["payload"]["summary"]["matches"], 0);
    assert_eq!(
        missing["payload"]["query"]["root"],
        root.path().to_str().unwrap()
    );
    assert_eq!(
        missing["payload"]["query"]["request"]["predicate"]["patterns"],
        json!(["absent"])
    );
    assert_eq!(missing["payload"]["summary"]["complete"], true);
    let limited = run(root.path(), &["search", "text", "witness"]);
    assert_eq!(limited["payload"]["results"].as_array().unwrap().len(), 20);
    assert_eq!(limited["payload"]["summary"]["complete"], false);
    let counted = run(root.path(), &["search", "text", "witness", "--count"]);
    assert_eq!(
        counted["payload"]["summary"]["count"],
        json!({"precision":"exact","value":43})
    );
}

#[test]
fn missing_native_provider_is_unknown_instead_of_a_complete_empty_answer() {
    let root = tempfile::tempdir().unwrap();
    assert!(
        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .arg(root.path())
            .status()
            .unwrap()
            .success()
    );
    let output = Command::new(env!("CARGO_BIN_EXE_pse"))
        .current_dir(root.path())
        .args([
            "search",
            "symbol",
            "missing",
            "--semantic",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let response: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(response["subject_state"], "unknown");
    assert_eq!(response["payload"]["summary"]["complete"], false);
    assert_eq!(response["payload"]["summary"]["unknown"], 1);
    assert_eq!(
        response["payload"]["summary"]["count"],
        json!({"precision":"at_least","value":0})
    );
}

#[test]
fn late_native_protocol_failure_keeps_already_delivered_witness_counts() {
    use std::os::unix::fs::PermissionsExt as _;
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("input.txt"), "authored").unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["add", "input.txt"],
        vec![
            "-c",
            "user.name=Control",
            "-c",
            "user.email=control@example.invalid",
            "commit",
            "-qm",
            "initial",
        ],
    ] {
        assert!(
            std::process::Command::new("git")
                .current_dir(root.path())
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    }
    let tools = tempfile::tempdir().unwrap();
    let script = tools.path().join("git");
    // A complete native record followed by a truncated second record exercises
    // partial transport failure, independently of Git history interpretation.
    std::fs::write(&script,"#!/bin/sh\nprintf 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\\000\\000Control\\0002026-10-31T00:00:00Z\\000first witness\\000truncated'\n").unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
    let path = std::env::join_paths(
        std::iter::once(tools.path().to_owned())
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_pse"))
        .current_dir(root.path())
        .env("PATH", path)
        .args(["search", "history", "--format", "ndjson"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let events: Vec<Value> = output
        .split(|&byte| byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect();
    assert_eq!(
        events
            .iter()
            .filter(|event| event["event"] == "result")
            .count(),
        1
    );
    let terminal = events.last().unwrap();
    assert_eq!(terminal["event"], "terminal");
    assert_eq!(terminal["emitted"], 1);
    assert_eq!(terminal["matches"], 1);
    assert_eq!(terminal["complete"], false);
    assert_eq!(terminal["failures"], 1);
    assert_eq!(terminal["count"], json!({"precision":"at_least","value":1}));
}

#[test]
fn public_stable_pages_continue_original_sources_and_refuse_changed_universe() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("a.rs"), "// witness alpha\n").unwrap();
    std::fs::write(root.path().join("b.toml"), "name = 'witness beta'\n").unwrap();
    std::fs::write(root.path().join("c.md"), "witness gamma\n").unwrap();
    let first = run(
        root.path(),
        &[
            "search",
            "text",
            "witness",
            "--order",
            "deterministic",
            "--limit",
            "1",
        ],
    );
    assert!(
        first["payload"]["results"][0]["context"]
            .as_str()
            .unwrap()
            .contains("alpha")
    );
    let cursor = first["payload"]["summary"]["next_cursor"].as_str().unwrap();
    let second = run(
        root.path(),
        &[
            "search",
            "text",
            "witness",
            "--order",
            "deterministic",
            "--limit",
            "1",
            "--cursor",
            cursor,
        ],
    );
    assert!(
        second["payload"]["results"][0]["context"]
            .as_str()
            .unwrap()
            .contains("beta")
    );
    std::fs::write(root.path().join("c.md"), "witness changed gamma\n").unwrap();
    Command::new(env!("CARGO_BIN_EXE_pse"))
        .current_dir(root.path())
        .args([
            "search",
            "text",
            "witness",
            "--order",
            "deterministic",
            "--limit",
            "1",
            "--cursor",
            cursor,
            "--format",
            "json",
        ])
        .assert()
        .failure();
    let all = run(root.path(), &["search", "text", "witness", "--all"]);
    assert_eq!(
        all["payload"]["summary"]["count"],
        json!({"precision":"exact","value":3})
    );
    assert_eq!(all["payload"]["sources"].as_array().unwrap().len(), 3);
}

#[test]
fn envelope_projection_preserves_shared_source_coverage_and_stream_fields_are_explicit() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("a.txt"), "witness\n").unwrap();
    let response = run(
        root.path(),
        &["search", "text", "witness", "--fields", "results"],
    );
    assert_eq!(response["payload"]["summary"]["matches"], 1);
    assert_eq!(response["payload"]["sources"].as_array().unwrap().len(), 1);
    assert!(response["payload"]["diagnostics"].is_array());
    let schema = run(root.path(), &["schema", "search", "--fields", "results"]);
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(&response)
        .unwrap();
    let rejected = Command::new(env!("CARGO_BIN_EXE_pse"))
        .current_dir(root.path())
        .args([
            "search", "text", "witness", "--format", "ndjson", "--fields", "results",
        ])
        .assert()
        .failure()
        .get_output()
        .clone();
    assert!(
        rejected.stdout.is_empty(),
        "invalid projection must refuse before stream begin"
    );
}

#[test]
fn human_inspection_and_search_render_code_as_lines_and_keep_portable_references() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("source.rs"),
        "// café\nfn clear() {\n    let witness = 1;\n}\n",
    )
    .unwrap();
    for args in [
        vec!["inspect", "--source", "source.rs", "--format", "human"],
        vec!["search", "text", "witness", "--format", "human"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_pse"))
            .current_dir(root.path())
            .args(args)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("\n    let witness = 1;\n"), "{text}");
        assert!(text.contains("reference={\"version\":1"), "{text}");
        assert!(
            !text.contains("\"context\":\""),
            "source code must be readable lines in human output"
        );
    }
}

#[test]
fn direct_line_inspection_and_json_requests_use_the_same_sources() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("settings.toml"),
        "first = 1\nsecond = 2\nthird = 3\n",
    )
    .unwrap();
    let selected = run(
        root.path(),
        &["inspect", "--source", "settings.toml", "--lines", "2:2"],
    );
    assert_eq!(selected["payload"]["context"], "second = 2\n");
    let request = json!({"sources":{"paths":["settings.toml"]},"predicate":{"kind":"text","patterns":["second"]}});
    std::fs::write(
        root.path().join("request.json"),
        serde_json::to_vec(&request).unwrap(),
    )
    .unwrap();
    let answer = run(root.path(), &["search", "--request", "request.json"]);
    assert_eq!(answer["payload"]["results"].as_array().unwrap().len(), 1);
    let explained = run(
        root.path(),
        &["search", "--request", "request.json", "--explain"],
    );
    assert!(
        explained["payload"]["origins"]
            .as_object()
            .unwrap()
            .contains_key("predicate.kind")
    );
}

#[test]
fn live_ndjson_has_complete_records_and_one_terminal_coverage_summary() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("a.txt"), "value\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_pse"))
        .current_dir(root.path())
        .args(["search", "text", "value", "--format", "ndjson"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let events: Vec<Value> = output
        .split(|&b| b == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect();
    assert_eq!(events.first().unwrap()["event"], "begin");
    assert_eq!(events.last().unwrap()["event"], "terminal");
    assert_eq!(
        events
            .iter()
            .filter(|event| event["event"] == "terminal")
            .count(),
        1
    );
    assert!(events.iter().any(|event| event["event"] == "result"));
}

#[test]
fn exhaustive_budget_limits_presentation_without_losing_exact_work_facts() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("population.txt"), "witness\n".repeat(43)).unwrap();
    let answer = run(
        root.path(),
        &[
            "search",
            "text",
            "witness",
            "--all",
            "--budget",
            "2",
            "--result-fields",
            "context",
            "--group-by",
            "source",
        ],
    );
    let payload = &answer["payload"];
    assert_eq!(payload["results"].as_array().unwrap().len(), 2);
    assert_eq!(payload["summary"]["matches"], 43);
    assert_eq!(payload["summary"]["complete"], true);
    assert_eq!(
        payload["summary"]["count"],
        json!({"precision":"exact","value":43})
    );
    assert_eq!(
        payload["summary"]["groups"]
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap(),
        &json!([0, 1])
    );
    assert!(payload["results"][0].get("name").is_none());
    assert!(payload["results"][0].get("reference").is_some());
    let explain = run(
        root.path(),
        &[
            "search",
            "text",
            "witness",
            "--all",
            "--budget",
            "2",
            "--explain",
        ],
    );
    assert_eq!(explain["payload"]["request"]["presentation"]["budget"], 2);
}

#[test]
fn explicitly_selected_native_filename_bytes_round_trip_through_public_reference() {
    use std::os::unix::ffi::OsStringExt as _;
    let root = tempfile::tempdir().unwrap();
    let name = std::ffi::OsString::from_vec(b"raw-\xff.rs".to_vec());
    std::fs::write(root.path().join(&name), "fn needle() {}\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_pse"))
        .current_dir(root.path())
        .args(["search", "text", "needle", "--path"])
        .arg(&name)
        .args(["--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let answer: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(
        answer["payload"]["sources"][0]["path"]["unix"],
        json!(b"raw-\xff.rs".to_vec())
    );
    let reference: pse_host::investigation::ResultReference =
        serde_json::from_value(answer["payload"]["results"][0]["reference"].clone()).unwrap();
    let inspected = run(
        root.path(),
        &["inspect", "--result", &reference.encode().unwrap()],
    );
    assert!(
        inspected["payload"]["context"]
            .as_str()
            .unwrap()
            .contains("needle")
    );
}

#[test]
fn an_explicit_cli_pattern_does_not_override_saved_case_with_an_implicit_default() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join(".config")).unwrap();
    std::fs::write(
        root.path().join(".config/pse-search.toml"),
        "[defaults.predicate]\nkind = 'text'\ncase = 'insensitive'\n",
    )
    .unwrap();
    std::fs::write(root.path().join("data.txt"), "NEEDLE\n").unwrap();
    let answer = run(
        root.path(),
        &["search", "text", "needle", "--path", "data.txt"],
    );
    assert_eq!(answer["payload"]["results"].as_array().unwrap().len(), 1);
}
