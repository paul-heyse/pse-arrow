// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Cold seed checks. Native publication is exercised by generation::bootstrap tests.
#![cfg(test)]
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

fn executable(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}
fn checkout() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    for directory in ["scripts", "bin", "target/host-tools", "target/debug"] {
        fs::create_dir_all(root.path().join(directory)).unwrap();
    }
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/bootstrap-host"),
        root.path().join("scripts/bootstrap-host"),
    )
    .unwrap();
    fs::write(
        root.path().join("target/host-tools/current"),
        "previous completed selection",
    )
    .unwrap();
    root
}
fn seed(root: &Path) -> Command {
    let mut command = Command::new("/bin/sh");
    command
        .arg(root.join("scripts/bootstrap-host"))
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", root.join("bin").display()),
        )
        .env("RUSTC_WRAPPER", "/missing/retired-python-wrapper")
        .env("RUSTC_WORKSPACE_WRAPPER", "/missing/retired-python-wrapper")
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET");
    command
}
fn unchanged(root: &Path) {
    assert_eq!(
        fs::read_to_string(root.join("target/host-tools/current")).unwrap(),
        "previous completed selection"
    );
}
#[test]
fn failed_seed_construction_preserves_completed_selection() {
    let root = checkout();
    executable(
        &root.path().join("bin/cargo"),
        "#!/bin/sh\n[ -z \"${RUSTC_WRAPPER+x}\" ] || exit 92\n[ -z \"${RUSTC_WORKSPACE_WRAPPER+x}\" ] || exit 93\nexit 47\n",
    );
    let result = seed(root.path()).output().unwrap();
    assert_eq!(result.status.code(), Some(47));
    unchanged(root.path());
}
#[test]
fn seed_executes_only_fixed_native_construction_without_lock_inheritance() {
    let root = checkout();
    executable(&root.path().join("bin/cargo"), "#!/bin/sh\nexit 0\n");
    executable(
        &root.path().join("target/debug/pse-host"),
        "#!/bin/sh\n[ \"$#\" = 1 ] && [ \"$1\" = __host-generation-bootstrap ] || exit 90\n[ ! -e /proc/self/fd/9 ] || exit 91\n[ -z \"${RUSTC_WRAPPER+x}\" ] || exit 92\n[ -z \"${RUSTC_WORKSPACE_WRAPPER+x}\" ] || exit 93\nprintf 'native helper called\\n'\n",
    );
    let result = seed(root.path()).output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout, b"native helper called\n");
    unchanged(root.path());
}
#[test]
fn native_construction_failure_is_preserved_without_shell_publication() {
    let root = checkout();
    executable(&root.path().join("bin/cargo"), "#!/bin/sh\nexit 0\n");
    executable(
        &root.path().join("target/debug/pse-host"),
        "#!/bin/sh\nexit 69\n",
    );
    assert_eq!(seed(root.path()).output().unwrap().status.code(), Some(69));
    unchanged(root.path());
}
#[test]
fn foreign_target_cannot_publish_or_enter_construction() {
    let root = checkout();
    executable(
        &root.path().join("bin/cargo"),
        "#!/bin/sh\ntouch entered-construction\n",
    );
    let result = seed(root.path())
        .env("CARGO_BUILD_TARGET", "foreign-target")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(125));
    assert!(!root.path().join("entered-construction").exists());
    unchanged(root.path());
}
