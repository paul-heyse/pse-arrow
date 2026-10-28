// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Black-box seed publication checks use private checkouts, never installed host selection.
use std::{
    fs,
    os::unix::{fs::PermissionsExt, process::ExitStatusExt},
    path::Path,
    process::Command,
};

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
        root.path().join("target/host-tools/pse-host"),
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
        .env_remove("CARGO_TARGET_DIR");
    command
}
#[test]
fn failed_construction_preserves_completed_selection() {
    let root = checkout();
    executable(
        &root.path().join("bin/cargo"),
        "#!/bin/sh\n[ -z \"${RUSTC_WRAPPER+x}\" ] || exit 92\n[ -z \"${RUSTC_WORKSPACE_WRAPPER+x}\" ] || exit 93\nexit 47\n",
    );
    let result = seed(root.path()).output().unwrap();
    assert_eq!(result.status.code(), Some(47));
    assert_eq!(
        fs::read_to_string(root.path().join("target/host-tools/pse-host")).unwrap(),
        "previous completed selection"
    );
}
#[test]
fn successful_construction_publishes_only_completed_executable() {
    let root = checkout();
    executable(
        &root.path().join("bin/cargo"),
        "#!/bin/sh\n[ -z \"${RUSTC_WRAPPER+x}\" ] || exit 92\n[ -z \"${RUSTC_WORKSPACE_WRAPPER+x}\" ] || exit 93\nexit 0\n",
    );
    let artifact = "#!/bin/sh\nprintf 'completed native selection\\n'\n";
    executable(&root.path().join("target/debug/pse-host"), artifact);
    let result = seed(root.path()).output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        fs::read_to_string(root.path().join("target/host-tools/pse-host")).unwrap(),
        artifact
    );
    assert_eq!(
        fs::read_dir(root.path().join("target/host-tools"))
            .unwrap()
            .count(),
        1
    );
}
#[test]
fn foreign_target_cannot_publish_stale_local_artifact() {
    let root = checkout();
    executable(&root.path().join("bin/cargo"), "#!/bin/sh\nexit 0\n");
    executable(
        &root.path().join("target/debug/pse-host"),
        "#!/bin/sh\nexit 0\n",
    );
    let result = seed(root.path())
        .env("CARGO_TARGET_DIR", root.path().join("foreign-output"))
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert_eq!(
        fs::read_to_string(root.path().join("target/host-tools/pse-host")).unwrap(),
        "previous completed selection"
    );
}

#[test]
fn invalid_completed_artifact_preserves_previous_selection() {
    let root = checkout();
    executable(&root.path().join("bin/cargo"), "#!/bin/sh\nexit 0\n");
    executable(
        &root.path().join("target/debug/pse-host"),
        "#!/bin/sh\nexit 69\n",
    );
    let result = seed(root.path()).output().unwrap();
    assert_eq!(result.status.code(), Some(69));
    assert_eq!(
        fs::read_to_string(root.path().join("target/host-tools/pse-host")).unwrap(),
        "previous completed selection"
    );
    assert_eq!(
        fs::read_dir(root.path().join("target/host-tools"))
            .unwrap()
            .count(),
        1
    );
}
#[test]
fn interrupted_construction_cannot_publish_a_stale_artifact() {
    let root = checkout();
    executable(
        &root.path().join("bin/cargo"),
        "#!/bin/sh\nkill -TERM \"$PPID\"\nexit 0\n",
    );
    executable(
        &root.path().join("target/debug/pse-host"),
        "#!/bin/sh\nexit 0\n",
    );
    let result = seed(root.path()).output().unwrap();
    assert!(result.status.code() == Some(143) || result.status.signal() == Some(15));
    assert_eq!(
        fs::read_to_string(root.path().join("target/host-tools/pse-host")).unwrap(),
        "previous completed selection"
    );
}
