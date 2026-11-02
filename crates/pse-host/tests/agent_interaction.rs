// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public discovery and correction refuse invalid demand before provider admission.
use serde_json::Value;
use std::{path::Path, process::Command};
#[cfg(test)]
fn pse(root: &Path) -> Command {
    let mut command = Command::new(
        std::env::var_os("CARGO_BIN_EXE_pse").expect("native test runner supplies pse"),
    );
    command.current_dir(root).env("PSE_FORMAT", "json");
    command
}
#[test]
fn concise_catalog_expands_to_full_parser_without_changing_record_count_semantics() {
    let root = tempfile::tempdir().unwrap();
    let short = pse(root.path())
        .args(["commands", "search", "--limit", "1"])
        .output()
        .unwrap();
    assert!(short.status.success(), "{short:?}");
    let short: Value = serde_json::from_slice(&short.stdout).unwrap();
    let record = &short["payload"]["records"][0];
    assert_eq!(record["path"], serde_json::json!(["search"]));
    assert!(record.get("arguments").is_none());
    let detail: Vec<&str> = record["detail"]
        .as_array()
        .unwrap()
        .iter()
        .skip(1)
        .map(|value| value.as_str().unwrap())
        .collect();
    let full = pse(root.path()).args(detail).output().unwrap();
    assert!(full.status.success(), "{full:?}");
    let full: Value = serde_json::from_slice(&full.stdout).unwrap();
    assert!(
        full["payload"]["records"][0]["arguments"]
            .as_array()
            .is_some_and(|args| args.len() > 3)
    );
}
#[test]
fn invalid_fields_do_not_admit_history_provider_and_report_recoverable_choices() {
    use std::os::unix::fs::PermissionsExt as _;
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("input.rs"), "fn witness() {}\n").unwrap();
    let tools = tempfile::tempdir().unwrap();
    let git = tools.path().join("git");
    std::fs::write(
        &git,
        "#!/bin/sh\n: > \"$PSE_TEST_ADMISSION_WITNESS\"\nexit 19\n",
    )
    .unwrap();
    std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o700)).unwrap();
    let witness = root.path().join("provider-was-admitted");
    let mut paths = vec![tools.path().to_path_buf()];
    paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
    let path = std::env::join_paths(paths).unwrap();
    let base = [
        "search",
        "history",
        "--operation",
        "blame",
        "--path",
        "input.rs",
    ];
    for format in ["json", "human"] {
        let invalid = pse(root.path())
            .env("PATH", &path)
            .env("PSE_TEST_ADMISSION_WITNESS", &witness)
            .args(base)
            .args(["--fields", "records.path", "--format", format])
            .output()
            .unwrap();
        assert_eq!(invalid.status.code(), Some(2), "{invalid:?}");
        assert!(
            !witness.exists(),
            "invalid presentation admitted an actual provider"
        );
        let error = String::from_utf8(invalid.stderr).unwrap();
        assert!(error.contains("authoring.parse.unknown_key"), "{error}");
        assert!(error.contains("Valid choices:"), "{error}");
        assert!(error.contains("results"), "{error}");
    }
    let valid = pse(root.path())
        .env("PATH", path)
        .env("PSE_TEST_ADMISSION_WITNESS", &witness)
        .args(base)
        .output()
        .unwrap();
    assert!(
        witness.exists(),
        "positive control did not reach the selected provider: {valid:?}"
    );
}

#[test]
fn request_relative_root_refusal_keeps_origin_and_executable_passive_recovery() {
    let root = tempfile::tempdir().unwrap();
    let input = root.path().join("inputs");
    std::fs::create_dir(&input).unwrap();
    let request = input.join("query.json");
    std::fs::write(&request, r#"{"sources":{"paths":["missing root"]},"predicate":{"kind":"symbol","provider":"semantic","name":"needle"}}"#).unwrap();
    let state = tempfile::tempdir().unwrap();
    let run = |arguments: &[&str]| {
        pse(root.path())
            .env("XDG_STATE_HOME", state.path())
            .args(arguments)
            .output()
            .unwrap()
    };
    let explained = run(&["search", "--request", "inputs/query.json", "--explain"]);
    assert!(explained.status.success(), "{explained:?}");
    let explained: Value = serde_json::from_slice(&explained.stdout).unwrap();
    validate_schema(root.path(), "search explain", &explained);
    assert_eq!(
        explained["payload"]["request"]["sources"]["paths"][0],
        input.join("missing root").to_str().unwrap()
    );
    assert_eq!(
        explained["payload"]["origins"]["sources.paths"]["provider"],
        "request file"
    );
    for format in ["json", "ndjson", "human"] {
        let refused = run(&[
            "search",
            "--request",
            "inputs/query.json",
            "--format",
            format,
        ]);
        assert!(!refused.status.success(), "{refused:?}");
        assert!(
            refused.stdout.is_empty(),
            "selection refusal must precede Begin: {refused:?}"
        );
        if format == "human" {
            assert!(
                String::from_utf8(refused.stderr)
                    .unwrap()
                    .contains("missing_root")
            );
            continue;
        }
        let failure: Value = serde_json::from_slice(&refused.stderr).unwrap();
        let recovery = &failure["failures"][0]["recovery"];
        assert_eq!(recovery["operation"], "select_root");
        assert_eq!(recovery["cause"], "missing_root");
        assert_eq!(
            recovery["operand"],
            input.join("missing root").to_str().unwrap()
        );
        assert_eq!(recovery["origin"]["provider"], "request file");
        assert_eq!(recovery["origin"]["source"], request.to_str().unwrap());
        let argv = recovery["next"]["argv"]
            .as_array()
            .unwrap()
            .iter()
            .skip(1)
            .map(|argument| argument.as_str().unwrap())
            .collect::<Vec<_>>();
        let replayed = run(&argv);
        assert!(replayed.status.success(), "{replayed:?}");
        let replayed: Value = serde_json::from_slice(&replayed.stdout).unwrap();
        assert_eq!(
            replayed["payload"]["request"],
            explained["payload"]["request"]
        );
    }
    let override_explained = run(&[
        "search",
        "--request",
        "inputs/query.json",
        "--path",
        "override",
        "--explain",
    ]);
    assert!(
        override_explained.status.success(),
        "{override_explained:?}"
    );
    let override_explained: Value = serde_json::from_slice(&override_explained.stdout).unwrap();
    assert_eq!(
        override_explained["payload"]["request"]["sources"]["paths"][0],
        root.path().join("override").to_str().unwrap()
    );
    assert_eq!(
        override_explained["payload"]["origins"]["sources.paths"]["provider"],
        "explicit CLI values"
    );
    assert!(
        !state.path().join("pse").exists(),
        "passive resolution/refusal prepared native state"
    );
}

fn validate_schema(root: &Path, command: &str, response: &Value) {
    let output = pse(root).args(["schema", command]).output().unwrap();
    assert!(output.status.success(), "{output:?}");
    let schema: Value = serde_json::from_slice(&output.stdout).unwrap();
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(response)
        .unwrap();
}

#[test]
fn query_failure_is_consumed_from_stdout_diagnostic_and_terminal() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("input.rs"), "fn needle() {}\n").unwrap();
    let base = [
        "search",
        "--path",
        "input.rs",
        "syntax",
        "--tree-sitter",
        "(definitely_not_a_rust_node)",
    ];
    let output = pse(root.path()).args(base).output().unwrap();
    assert!(!output.status.success(), "{output:?}");
    assert!(
        output.stderr.is_empty(),
        "in-query failure belongs to stdout: {output:?}"
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    validate_schema(root.path(), "search", &response);
    assert!(response["failures"].as_array().unwrap().is_empty());
    assert!(response["payload"]["summary"]["failures"].as_u64().unwrap() > 0);
    assert_eq!(response["payload"]["summary"]["complete"], false);
    assert!(
        !response["payload"]["diagnostics"][0]["detail"]["parser_context"]
            .as_object()
            .unwrap()
            .is_empty()
    );
    let streamed = pse(root.path())
        .args(base)
        .args(["--format", "ndjson"])
        .output()
        .unwrap();
    assert!(!streamed.status.success(), "{streamed:?}");
    assert!(streamed.stderr.is_empty());
    let events = String::from_utf8(streamed.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    let schema_output = pse(root.path())
        .args(["schema", "search stream"])
        .output()
        .unwrap();
    assert!(schema_output.status.success());
    let schema: Value = serde_json::from_slice(&schema_output.stdout).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    for event in &events {
        validator.validate(event).unwrap();
    }
    assert!(events.iter().any(|event| event["event"] == "diagnostic"));
    let terminal = events.last().unwrap();
    assert_eq!(terminal["event"], "terminal");
    assert_eq!(terminal["complete"], false);
    assert!(terminal["failures"].as_u64().unwrap() > 0);
}

#[test]
fn source_ids_and_guarded_expansion_keep_source_identity_and_freshness() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("input.rs");
    std::fs::write(&source, "fn needle() {}\n").unwrap();
    let output = pse(root.path())
        .args([
            "search",
            "--path",
            "input.rs",
            "text",
            "needle",
            "--result-fields",
            "name",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    validate_schema(root.path(), "search", &response);
    let row = &response["payload"]["results"][0];
    let sources = response["payload"]["sources"].as_array().unwrap();
    let selected = sources
        .iter()
        .find(|source| source["id"] == row["source"])
        .unwrap();
    assert_eq!(selected["path"], "input.rs");
    assert_eq!(row["context"], "");
    let address = row["reference"]["address"].as_str().unwrap();
    let guarded = pse(root.path())
        .args(["inspect", "--result", address])
        .output()
        .unwrap();
    assert!(guarded.status.success(), "{guarded:?}");
    let guarded: Value = serde_json::from_slice(&guarded.stdout).unwrap();
    validate_schema(root.path(), "inspect", &guarded);
    assert!(
        guarded["payload"]["context"]
            .as_str()
            .unwrap()
            .contains("needle")
    );
    let start = row["range"]["byte_start"].as_u64().unwrap().to_string();
    let end = row["range"]["byte_end"].as_u64().unwrap().to_string();
    let direct = pse(root.path())
        .args([
            "inspect",
            "--source",
            "input.rs",
            "--range-start",
            &start,
            "--range-end",
            &end,
        ])
        .output()
        .unwrap();
    assert!(direct.status.success(), "{direct:?}");
    let direct: Value = serde_json::from_slice(&direct.stdout).unwrap();
    validate_schema(root.path(), "inspect", &direct);
    assert_eq!(direct["payload"]["context"], guarded["payload"]["context"]);
    std::fs::write(&source, "fn changed() {}\n").unwrap();
    let stale = pse(root.path())
        .args(["inspect", "--result", address])
        .output()
        .unwrap();
    assert!(!stale.status.success(), "{stale:?}");
    assert!(stale.stdout.is_empty());
    let malformed = pse(root.path())
        .args(["inspect", "--result", "not-a-source-address"])
        .output()
        .unwrap();
    assert!(!malformed.status.success(), "{malformed:?}");
    assert!(malformed.stdout.is_empty());
    let current = pse(root.path())
        .args(["inspect", "--source", "input.rs"])
        .output()
        .unwrap();
    assert!(current.status.success(), "{current:?}");
    let current: Value = serde_json::from_slice(&current.stdout).unwrap();
    assert!(
        current["payload"]["context"]
            .as_str()
            .unwrap()
            .contains("changed")
    );
}
