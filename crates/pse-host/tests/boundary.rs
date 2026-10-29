// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Real published-frontdoor composition without a Python environment.
use std::{
    fs,
    os::unix::{
        fs::{PermissionsExt, symlink},
        process::ExitStatusExt,
    },
    path::Path,
    process::Command,
};

fn fixture() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in [
        "scripts",
        ".config",
        ".cargo",
        "target/host-tools/current",
        "poison",
    ] {
        fs::create_dir_all(root.join(name)).unwrap();
    }
    for name in [
        ".python-version",
        ".config/build.toml",
        ".config/diagnostic-tools.toml",
        ".config/agent-capacity.toml",
        ".cargo/config.toml",
        ".config/sccache.toml",
    ] {
        fs::copy(source.join(name), root.join(name)).unwrap();
    }
    // Use the immutable checked-in adapters, retaining the fixture paths as argv[0].
    // Executing freshly copied scripts can race Linux's delayed writable-file release
    // across concurrent test threads and fail ETXTBSY before the boundary runs.
    for name in ["scripts/pse-env", "scripts/pse-host"] {
        symlink(source.join(name), root.join(name)).unwrap();
    }
    symlink(
        env!("CARGO_BIN_EXE_pse-host"),
        root.join("target/host-tools/current/pse-host"),
    )
    .unwrap();
    for name in ["python", "python3"] {
        let path = root.join("poison").join(name);
        fs::write(
            &path,
            b"#!/bin/sh\nprintf 'Python was invoked\\n' >&2\nexit 99\n",
        )
        .unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }
    fs::write(root.join(".envrc.local"), "export PSE_BUILD_MODE=unbounded\nexport PRIVATE_VALUE=do-not-print-this\nprintf x >> \"$HOME/activation-count\"\n").unwrap();
    directory
}
fn frontdoor(root: &Path) -> Command {
    let mut command = Command::new(root.join("scripts/pse-env"));
    command.current_dir(root).env_clear().env("HOME", root).env(
        "PATH",
        format!("{}:/usr/bin:/bin", root.join("poison").display()),
    );
    for key in ["XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command
}
#[test]
fn boundary_frontdoor_needs_no_python_and_preserves_opaque_child_help() {
    let fixture = fixture();
    let root = fixture.path();
    let result = frontdoor(root)
        .args([
            "/bin/sh",
            "-c",
            "test \"$PRIVATE_VALUE\" = do-not-print-this || exit 90; printf '%s' \"$1\"; exit 7",
            "fixture",
            "--help",
        ])
        .output()
        .unwrap();
    assert_eq!(
        result.status.code(),
        Some(7),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout, b"--help");
    assert_eq!(fs::read(root.join("activation-count")).unwrap(), b"x");
    assert!(!root.join(".venv").exists());
}
#[test]
fn boundary_unsupported_modes_refuse_before_personal_or_command_effects() {
    let fixture = fixture();
    let root = fixture.path();
    for options in [
        vec!["--native=unknown"],
        vec!["--store"],
        vec!["--bind-allocation"],
        vec!["--resource-class", "unknown"],
        vec!["--recipe-default-class", "unknown"],
    ] {
        let result = frontdoor(root)
            .args(&options)
            .args(["--", "/usr/bin/touch", "must-not-exist"])
            .output()
            .unwrap();
        assert_eq!(
            result.status.code(),
            Some(125),
            "{options:?}: {:?}",
            result.status.signal()
        );
        assert!(result.stderr.starts_with(b"pse-env:"));
        assert!(!root.join("activation-count").exists());
        assert!(!root.join("must-not-exist").exists());
    }
}
#[test]
fn boundary_help_after_prefix_is_source_free() {
    let fixture = fixture();
    let root = fixture.path();
    let result = frontdoor(root).args(["--docs", "--help"]).output().unwrap();
    assert!(result.status.success());
    assert!(result.stdout.starts_with(b"pse-env "));
    assert!(!root.join("activation-count").exists());
}
#[test]
fn boundary_missing_selection_never_builds_or_enters_python() {
    let fixture = fixture();
    let root = fixture.path();
    fs::remove_file(root.join("target/host-tools/current/pse-host")).unwrap();
    let result = frontdoor(root).arg("--help").output().unwrap();
    assert_eq!(result.status.code(), Some(125));
    assert!(String::from_utf8_lossy(&result.stderr).contains("scripts/bootstrap-host"));
    assert!(!root.join("activation-count").exists());
}

#[test]
fn native_explanation_is_json_and_never_activates_personal_settings() {
    let fixture = fixture();
    let root = fixture.path();
    let result = frontdoor(root)
        .args(["--explain", "--json", "--native=compiler,klu"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let document: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(document["native_profile"], "compiler,klu");
    assert_eq!(document["readiness"], "not_observed");
    assert!(!root.join("activation-count").exists());
}

#[test]
fn native_launch_without_preparation_never_builds_or_runs_command() {
    let fixture = fixture();
    let root = fixture.path();
    let result = frontdoor(root)
        .args([
            "--native=compiler,klu",
            "--",
            "/usr/bin/touch",
            "must-not-exist",
        ])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(125));
    assert!(!root.join("must-not-exist").exists());
    assert!(!root.join(".cache/pse-arrow/native/rust-v1").exists());
}

#[test]
fn retained_provider_explanations_are_source_free_and_private() {
    let fixture = fixture();
    let root = fixture.path();
    for capabilities in ["", "compiler", "solver", "isolation", "uno", "petsc", "compiler,solver,klu"] {
        let result = frontdoor(root)
            .args(["--explain", "--json", &format!("--native={capabilities}")])
            .output().unwrap();
        assert!(result.status.success(), "{capabilities}: {result:?}");
        let document: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(document["native_profile"], capabilities);
        assert_eq!(document["readiness"], "not_observed");
        assert!(!String::from_utf8_lossy(&result.stdout).contains("do-not-print-this"));
        assert!(!root.join("activation-count").exists());
        assert!(!root.join(".cache").exists());
    }
}
