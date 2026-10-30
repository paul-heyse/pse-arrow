// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public native worktree commands preserve original partial outcomes and byte paths.
#![cfg(test)]
use std::{
    fs,
    os::unix::fs::PermissionsExt as _,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(root)
        .args([
            "-c",
            "user.name=worktree journey",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
}
fn fixture() -> (tempfile::TempDir, PathBuf) {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path().join("original");
    fs::create_dir_all(root.join(".config")).unwrap();
    fs::create_dir(root.join("scripts")).unwrap();
    fs::write(root.join(".config/diagnostic-tools.toml"), "").unwrap();
    fs::write(root.join(".gitignore"), ".envrc.local\n").unwrap();
    fs::write(root.join("written.rs"), "fn written() {}\n").unwrap();
    fs::write(root.join("scripts/bootstrap-host"),"#!/bin/sh\n[ \"$1\" = --from ] || exit 41\nprintf 'original preparation failure\\n' >&2\nexit 42\n").unwrap();
    fs::set_permissions(
        root.join("scripts/bootstrap-host"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    git(&root, &["init", "--quiet", "--initial-branch=main"]);
    git(&root, &["add", "."]);
    git(&root, &["commit", "--quiet", "-m", "original"]);
    (scratch, root)
}
fn pse(root: &Path) -> Command {
    let mut command =
        Command::new(std::env::var_os("CARGO_BIN_EXE_pse").expect("selected native executable"));
    command
        .current_dir(root)
        .arg("-C")
        .arg(root)
        .env("PSE_FORMAT", "json");
    command
}
fn decoded(output: Output, status: i32) -> serde_json::Value {
    assert_eq!(output.status.code(), Some(status), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}
fn schema(root: &Path, name: &str, response: &serde_json::Value) {
    let schema = decoded(pse(root).args(["schema", name]).output().unwrap(), 0);
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(response)
        .unwrap();
}
#[test]
fn native_new_preserves_failed_preparation_and_list_observes_without_preparing() {
    let (scratch, root) = fixture();
    fs::write(root.join(".envrc.local"), "PRIVATE=kept-private\n").unwrap();
    fs::set_permissions(root.join(".envrc.local"), fs::Permissions::from_mode(0o640)).unwrap();
    let created = decoded(
        pse(&root)
            .args(["worktree", "new", "partial"])
            .output()
            .unwrap(),
        1,
    );
    schema(&root, "worktree new", &created);
    assert_eq!(created["payload"]["git_created"], true);
    assert_eq!(created["payload"]["ready"], false);
    assert!(scratch.path().join("partial/written.rs").is_file());
    assert_eq!(
        fs::read(scratch.path().join("partial/.envrc.local")).unwrap(),
        b"PRIVATE=kept-private\n"
    );
    assert!(!created.to_string().contains("kept-private"));
    let stages = created["payload"]["stages"].as_array().unwrap();
    assert_eq!(
        stages
            .iter()
            .find(|stage| stage["stage"] == "host_preparation")
            .unwrap()["command"]["exit_code"],
        42
    );
    assert_eq!(
        stages
            .iter()
            .find(|stage| stage["stage"] == "warm_launch")
            .unwrap()["state"],
        "skipped"
    );
    let listed = decoded(pse(&root).args(["worktree", "list"]).output().unwrap(), 0);
    schema(&root, "worktree list", &listed);
    assert_eq!(listed["payload"]["worktrees"].as_array().unwrap().len(), 2);
    assert!(!scratch.path().join("partial/target").exists());
    let help = pse(&root).args(["worktree", "--help"]).output().unwrap();
    assert!(help.status.success());
    assert!(!String::from_utf8(help.stdout).unwrap().contains("\n  rm "));
}
