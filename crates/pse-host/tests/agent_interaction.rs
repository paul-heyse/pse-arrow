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
