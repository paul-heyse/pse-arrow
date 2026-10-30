// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual CLI framing, schema selection and raw-protocol boundaries.
use std::{
    path::Path,
    process::{Command, Output},
};
fn pse() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_pse"));
    command
        .current_dir(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap(),
        )
        .env("PSE_FORMAT", "json");
    command
}
fn json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| panic!("{error}: {output:?}"))
}
#[test]
fn usage_errors_have_structured_context_in_the_requested_format() {
    let output = pse()
        .args(["--format", "json", "--unknown-option"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(value["outcome"], "usage");
    assert_eq!(value["failures"][0]["code"], "cli.usage");
    assert_eq!(
        value["failures"][0]["parser_context"]["error_kind"],
        "UnknownArgument"
    );
    assert!(
        value["invocation_id"]
            .as_str()
            .is_some_and(|value| uuid::Uuid::parse_str(value).is_ok())
    );
    let early = pse()
        .env("PSE_FORMAT", "human")
        .args(["run", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(early.status.code(), Some(2));
    let _: serde_json::Value = serde_json::from_slice(&early.stderr).unwrap();
}
#[test]
fn env_projection_retains_full_evidence_and_matches_its_projected_schema() {
    let state = tempfile::tempdir().unwrap();
    let output = pse()
        .env("XDG_STATE_HOME", state.path())
        .args([
            "env",
            "--explain",
            "--fields",
            "selections",
            "--budget",
            "0",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let value = json(&output);
    assert_eq!(value["command"], "env explain");
    assert_eq!(value["outcome"], "success");
    assert!(value["payload"].get("local_unresolved").is_none());
    assert!(value["payload"]["selections"].is_object());
    assert_eq!(
        value["view"]["omitted_fields"],
        serde_json::json!(["local_unresolved"])
    );
    let artifact = value["artifacts"][0]["location"].as_str().unwrap();
    assert!(Path::new(artifact).starts_with(state.path()));
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(artifact).unwrap()).unwrap();
    assert!(original["payload"]["local_unresolved"].is_boolean());
    assert_eq!(original["invocation_id"], value["invocation_id"]);
    let schema_output = pse()
        .args([
            "schema",
            "env",
            "explain",
            "--fields",
            "selections",
            "--budget",
            "0",
        ])
        .output()
        .unwrap();
    assert!(schema_output.status.success(), "{schema_output:?}");
    let schema = json(&schema_output);
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(schema["$id"], value["schema"]);
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(
        validator.is_valid(&value),
        "{:?}",
        validator
            .iter_errors(&value)
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
    );
}
#[test]
fn incomplete_activity_ndjson_is_one_valid_final_record_with_managed_failure() {
    let directory = tempfile::tempdir().unwrap();
    let output = pse()
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path={}/absent", directory.path().display()),
        )
        .args(["--format", "ndjson", "activity", "selected.service"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let text = std::str::from_utf8(&output.stdout).unwrap();
    assert_eq!(text.lines().count(), 1);
    let value = json(&output);
    assert_eq!(value["record"], "observation_summary");
    assert_eq!(value["subject_state"], "not_applicable");
    assert_eq!(value["observation"]["completeness"], "partial");
    assert!(
        !value["observation"]["limitations"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let schema = json(&pse().args(["schema", "activity"]).output().unwrap());
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(validator.is_valid(&value));
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
}
#[test]
fn protocol_outputs_ignore_format_and_never_gain_an_envelope() {
    let version = pse()
        .env("PSE_FORMAT", "ndjson")
        .arg("--version")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert!(version.stdout.starts_with(b"pse "));
    assert!(version.stdout.ends_with(b"protocol 1\n"));
    let tools = pse()
        .args(["--format", "json", "cargo-tools", "cargo-nextest"])
        .output()
        .unwrap();
    assert!(tools.status.success(), "{tools:?}");
    assert!(!tools.stdout.starts_with(b"{"));
    assert!(!tools.stdout.is_empty());
    let print = pse()
        .args(["--format", "json", "env", "--print"])
        .output()
        .unwrap();
    assert!(print.status.success(), "{print:?}");
    assert!(!print.stdout.starts_with(b"{"));
    assert!(String::from_utf8_lossy(&print.stdout).contains("activation-data"));
}
#[test]
fn foreground_child_options_and_exit_status_stay_literal() {
    let output = pse()
        .args([
            "run",
            "--",
            "/bin/sh",
            "-c",
            "printf '%s\\n' \"$@\"; exit 7",
            "child",
            "--format",
            "json",
            "-h",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(7), "{output:?}");
    assert_eq!(output.stdout, b"--format\njson\n-h\n");
}
#[test]
fn native_explanation_json_selection_uses_the_same_envelope_renderer() {
    let output = pse()
        .args([
            "--format",
            "human",
            "boundary",
            "--explain",
            "--json",
            "--native=compiler",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let value = json(&output);
    assert_eq!(value["payload"]["native_profile"], "compiler");
    assert_eq!(value["payload"]["readiness"], "not_observed");
    assert_eq!(value["subject_state"], "not_applicable");
}
