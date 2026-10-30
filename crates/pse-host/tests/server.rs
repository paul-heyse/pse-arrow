// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public native server selection is read-only and does not depend on Python or source.
#![cfg(test)]
use pse_host::{environment::Environment, server};
use std::fs;

#[test]
fn server_missing_selection_refuses_without_creating_tools_or_reading_checkout() {
    let fixture = tempfile::tempdir().unwrap();
    let missing = fixture.path().join("unprepared");
    let mut environment = Environment::new();
    environment.insert("PSE_SURREAL_TOOLS".into(), missing.clone().into_os_string());
    let error = server::select(&environment).unwrap_err().to_string();
    assert!(error.contains("server prepare"), "{error}");
    assert!(!missing.exists());
}

#[test]
fn server_corrupt_selection_never_searches_or_builds_a_fallback() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("rust-v1");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("current.json"), b"not a selection").unwrap();
    let mut environment = Environment::new();
    environment.insert("PSE_SURREAL_TOOLS".into(), fixture.path().into());
    assert!(server::select(&environment).is_err());
    assert_eq!(
        fs::read(root.join("current.json")).unwrap(),
        b"not a selection"
    );
    assert!(!root.join("builds").exists());
}

#[test]
fn server_cli_rejects_implicit_build_and_service_arguments_before_effects() {
    for command in [
        "start",
        "setup",
        "restart",
        "build-server",
        "prepare --force",
    ] {
        assert!(
            !std::process::Command::new(env!("CARGO_BIN_EXE_pse-host"))
                .args(["server", command])
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    for arguments in [
        vec!["prepare", "--from-release"],
        vec!["select", "--from-release", "/unused"],
        vec!["prepare", "--from-release", "/unused", "--force"],
    ] {
        assert!(
            !std::process::Command::new(env!("CARGO_BIN_EXE_pse-host"))
                .arg("server")
                .args(arguments)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
}
