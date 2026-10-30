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

fn source_checkout(body: &str) -> tempfile::TempDir {
    let source = tempfile::Builder::new()
        .prefix("copy source ")
        .tempdir()
        .unwrap();
    let tools = source.path().join("target/host-tools");
    fs::create_dir_all(tools.join("generation.fixture")).unwrap();
    executable(&tools.join("generation.fixture/pse-host"), body);
    std::os::unix::fs::symlink("generation.fixture", tools.join("current")).unwrap();
    source
}

#[test]
fn cold_from_uses_fixed_source_entry_and_preserves_literal_checkout_arguments_without_compiling() {
    let root = checkout();
    let source = source_checkout(
        "#!/bin/sh\n[ \"$#\" = 5 ] && [ \"$1\" = -C ] && [ \"$3\" = __host-generation-bootstrap ] && [ \"$4\" = --from ] || exit 93\n[ \"$PSE_BOOTSTRAP_SOURCE_HELPER\" = \"$5\" ] || exit 96\nprintf '%s\\0' \"$@\" > \"$2/source-call\"\n",
    );
    executable(
        &root.path().join("bin/cargo"),
        "#!/bin/sh\ntouch \"$CARGO_PROBE\"\nexit 94\n",
    );
    let result = seed(root.path())
        .args([std::ffi::OsStr::new("--from"), source.path().as_os_str()])
        .env("CARGO_PROBE", root.path().join("cargo-used"))
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!root.path().join("cargo-used").exists());
    let values = fs::read(root.path().join("source-call")).unwrap();
    let parts: Vec<_> = values.split(|byte| *byte == 0).collect();
    use std::os::unix::ffi::OsStrExt;
    assert_eq!(
        parts,
        vec![
            b"-C".as_slice(),
            root.path().as_os_str().as_bytes(),
            b"__host-generation-bootstrap".as_slice(),
            b"--from".as_slice(),
            source.path().as_os_str().as_bytes(),
            b"".as_slice()
        ]
    );
}

#[test]
fn incompatible_source_helper_falls_back_with_reason_and_forwards_from_to_local_seed() {
    let root = checkout();
    let source = source_checkout("#!/bin/sh\nexit 2\n");
    executable(
        &root.path().join("bin/cargo"),
        "#!/bin/sh\ntouch \"$CARGO_PROBE\"\n",
    );
    executable(
        &root.path().join("target/debug/pse-host"),
        "#!/bin/sh\n[ \"$#\" = 3 ] && [ \"$1\" = __host-generation-bootstrap ] && [ \"$2\" = --from ] || exit 95\n",
    );
    let result = seed(root.path())
        .arg(format!("--from={}", source.path().display()))
        .env("CARGO_PROBE", root.path().join("cargo-used"))
        .output()
        .unwrap();
    assert!(result.status.success());
    assert!(root.path().join("cargo-used").exists());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains(
            "source helper unavailable or incompatible (status 2); building a local seed"
        )
    );
}

#[test]
fn source_build_or_validation_failure_is_not_retried_or_accepted_as_a_copy() {
    for status in [101, 125] {
        let root = checkout();
        let source = source_checkout(&format!("#!/bin/sh\nexit {status}\n"));
        executable(
            &root.path().join("bin/cargo"),
            "#!/bin/sh\ntouch \"$CARGO_PROBE\"\n",
        );
        let result = seed(root.path())
            .arg("--from")
            .arg(source.path())
            .env("CARGO_PROBE", root.path().join("cargo-used"))
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(status));
        assert!(!root.path().join("cargo-used").exists());
        unchanged(root.path());
    }
}

#[test]
fn copy_only_mismatch_builds_fresh_local_seed_with_clear_reason() {
    let root = checkout();
    let source = source_checkout("#!/bin/sh\nexit 79\n");
    executable(
        &root.path().join("bin/cargo"),
        "#!/bin/sh\ntouch \"$CARGO_PROBE\"\n",
    );
    executable(
        &root.path().join("target/debug/pse-host"),
        "#!/bin/sh\n[ -z \"${PSE_BOOTSTRAP_SOURCE_HELPER+x}\" ] || exit 97\nexit 0\n",
    );
    let result = seed(root.path())
        .arg("--from")
        .arg(source.path())
        .env("CARGO_PROBE", root.path().join("cargo-used"))
        .output()
        .unwrap();
    assert!(result.status.success());
    assert!(root.path().join("cargo-used").exists());
    assert!(
        String::from_utf8_lossy(&result.stderr)
            .contains("source inputs do not match; building a fresh local seed")
    );
}

#[test]
fn bootstrap_help_never_enters_copy_or_compilation() {
    let root = checkout();
    let source = source_checkout("#!/bin/sh\nexit 99\n");
    executable(&root.path().join("bin/cargo"), "#!/bin/sh\nexit 98\n");
    let result = seed(root.path())
        .arg("--from")
        .arg(source.path())
        .arg("--help")
        .output()
        .unwrap();
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).starts_with("Usage: scripts/bootstrap-host"));
    assert!(!root.path().join("target/host-tools/.update.lock").exists());
    unchanged(root.path());
}
