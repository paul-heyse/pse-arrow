// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public discovery/status routes remain useful with absent richer preparation.
#![cfg(test)]
use std::{path::Path, process::Command};
fn pse(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_pse"));
    command.current_dir(root).env("PSE_FORMAT", "json");
    command
}
fn value(output: &std::process::Output) -> serde_json::Value {
    assert!(output.status.success(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}
#[test]
fn catalog_and_scoped_help_work_outside_checkouts_and_publish_matching_schemas() {
    let root = tempfile::tempdir().unwrap();
    let output = pse(root.path())
        .args(["commands", "--json", "--prefix", "native", "--limit", "1"])
        .output()
        .unwrap();
    let document = value(&output);
    assert_eq!(document["payload"]["records"].as_array().unwrap().len(), 1);
    assert_eq!(document["payload"]["continuation"], 1);
    assert_eq!(
        document["payload"]["records"][0]["path"],
        serde_json::json!(["native"])
    );
    assert_eq!(document["providers"][0]["availability"], "available");
    let schema = value(
        &pse(root.path())
            .args(["schema", "commands"])
            .output()
            .unwrap(),
    );
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(validator.is_valid(&document));
    let help = value(
        &pse(root.path())
            .args(["help", "--json", "native", "prepare"])
            .output()
            .unwrap(),
    );
    let record = &help["payload"]["records"][0];
    assert_eq!(record["path"], serde_json::json!(["native", "prepare"]));
    assert!(
        record["long_help"]
            .as_str()
            .unwrap()
            .contains("--capabilities")
    );
    assert_eq!(record["output_schemas"]["availability"], "available");
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}
#[test]
fn finite_static_and_open_provider_candidates_keep_their_actual_domains() {
    let root = tempfile::tempdir().unwrap();
    let static_values = value(
        &pse(root.path())
            .args([
                "commands",
                "--argument",
                "format",
                "--metadata",
                "/definitely/absent/unrelated.json",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(static_values["payload"]["availability"], "available");
    assert_eq!(static_values["payload"]["exhaustiveness"], "exhaustive");
    assert_eq!(
        static_values["payload"]["records"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    let dynamic = value(
        &pse(root.path())
            .args([
                "help",
                "native",
                "prepare",
                "--argument",
                "capabilities",
                "--prefix",
                "co",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(dynamic["payload"]["records"][0]["value"], "compiler");
    assert_eq!(dynamic["payload"]["exhaustiveness"], "open");
    assert_eq!(dynamic["observation"]["completeness"], "partial");
    assert_eq!(dynamic["observation"]["required"], false);
    let unknown = value(
        &pse(root.path())
            .args(["help", "run", "--argument", "command"])
            .output()
            .unwrap(),
    );
    assert_eq!(unknown["payload"]["availability"], "unknown");
    assert!(unknown["payload"]["known_matches"].is_null());
    assert!(
        !unknown["observation"]["limitations"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}
#[test]
fn actual_owned_help_command_is_discoverable_and_page_zero_is_usage_error() {
    let root = tempfile::tempdir().unwrap();
    let output = value(
        &pse(root.path())
            .args(["commands", "--prefix", "help"])
            .output()
            .unwrap(),
    );
    assert_eq!(
        output["payload"]["records"][0]["path"],
        serde_json::json!(["help"])
    );
    let output = pse(root.path())
        .args(["commands", "--limit", "0"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["outcome"], "usage");
}
#[test]
fn status_summary_and_line_are_passive_and_do_not_create_checkout_state() {
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
    std::fs::create_dir_all(root.path().join(".config")).unwrap();
    std::fs::write(root.path().join(".config/diagnostic-tools.toml"), "").unwrap();
    std::fs::create_dir_all(root.path().join("docs/plans")).unwrap();
    std::fs::write(
        root.path().join("docs/plans/README.md"),
        "[59: current](59-agent-cli.md#current)\n",
    )
    .unwrap();
    let state = root.path().join("state");
    let output = value(
        &pse(root.path())
            .env("XDG_STATE_HOME", &state)
            .args(["status", "--brief"])
            .output()
            .unwrap(),
    );
    assert_eq!(output["command"], "status brief");
    assert_eq!(output["payload"]["head"]["value"]["state"], "unborn");
    assert_eq!(output["payload"]["invocation_records"]["value"], 0);
    assert_eq!(
        output["payload"]["live_workloads"]["availability"],
        "unavailable"
    );
    let schema = value(
        &pse(root.path())
            .args(["schema", "status", "brief"])
            .output()
            .unwrap(),
    );
    assert!(
        jsonschema::validator_for(&schema)
            .unwrap()
            .is_valid(&output)
    );
    let line = pse(root.path())
        .env("XDG_STATE_HOME", &state)
        .args(["--format", "human", "status", "--line"])
        .output()
        .unwrap();
    assert!(line.status.success(), "{line:?}");
    assert_eq!(
        std::str::from_utf8(&line.stdout).unwrap().lines().count(),
        1
    );
    assert!(String::from_utf8_lossy(&line.stdout).contains("coverage=Complete"));
    assert!(!state.exists());
    assert!(!root.path().join("target").exists());
}
