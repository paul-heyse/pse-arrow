// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public native server selection is read-only and does not depend on Python or source.
#![cfg(test)]
use pse_host::{environment::Environment, server};
use std::{fs, process::Command};

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
    let fixture = tempfile::tempdir().unwrap();
    let refused = |arguments: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_pse"))
            .arg("server")
            .args(arguments)
            .current_dir(fixture.path())
            .env("PSE_SURREAL_TOOLS", fixture.path().join("tools"))
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(125), "{arguments:?}: {output:?}");
        assert!(output.stderr.starts_with(b"pse-host: "), "{output:?}");
    };
    for command in [
        "start",
        "setup",
        "restart",
        "build-server",
        "prepare --force",
    ] {
        refused(&[command]);
    }
    for arguments in [
        &["prepare", "--from-release"][..],
        &["select", "--from-release", "/unused"],
        &["prepare", "--from-release", "/unused", "--force"],
    ] {
        refused(arguments);
    }
    assert_eq!(fs::read_dir(fixture.path()).unwrap().count(), 0);
}
