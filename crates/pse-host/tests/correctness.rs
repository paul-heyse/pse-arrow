// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native process boundary checks isolate the adapter from developer environment state.
#![cfg(test)]
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};
fn executable(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}
fn adapter(root: &Path) -> Command {
    let mut command = Command::new(
        std::env::var_os("CARGO_BIN_EXE_cargo-pse-correctness")
            .expect("Cargo/nextest supplies the selected runtime correctness executable"),
    );
    command
        .current_dir(root)
        .env(
            "PSE_TEST_CHECKOUT",
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap(),
        )
        .env("CARGO", root.join("cargo"))
        .env_remove("PSE_CORRECTNESS_REAL_CARGO")
        .env_remove("PSE_CORRECTNESS_ADAPTER")
        .env_remove("NEXTEST_TEST_PHASE");
    command
}
fn pure_cargo(root: &Path, version: u32) {
    fs::write(
        root.join("metadata.json"),
        json!({"packages":[{"id":"pure","name":"pure","features":{}}]}).to_string(),
    )
    .unwrap();
    fs::write(root.join("graph.json"),json!({"version":version,"roots":[0],"units":[{"pkg_id":"pure","target":{"name":"pure","kind":["lib"],"src_path":"/pure/lib.rs"},"features":[],"dependencies":[],"platform":null,"mode":"test"}]}).to_string()).unwrap();
    executable(
        &root.join("cargo"),
        r#"#!/bin/sh
[ "$CARGO" = "$PWD/cargo" ] || exit 96
[ -z "${PSE_CORRECTNESS_ADAPTER+x}" ] || exit 97
printf '%s\n' "$@" >> calls
case " $* " in
  *' locate-project '*) printf '%s/Cargo.toml\n' "$PSE_TEST_CHECKOUT" ;;
  *' --unit-graph '*)
    case " $* " in *' --no-deps '*) printf 'Cargo check rejects Clippy-only --no-deps\n' >&2; exit 2 ;; esac
    cat graph.json ;;
  *' metadata '*) cat metadata.json ;;
  *) printf '{"reason":"compiler-artifact","marker":"original"}\n'; printf 'compiler stderr\n' >&2; exit 17 ;;
esac
"#,
    );
}
#[test]
fn native_correctness_graph_stdout_is_private_and_compiler_exit_survives() {
    let root = tempfile::tempdir().unwrap();
    pure_cargo(root.path(), 1);
    let result = adapter(root.path())
        .args(["test", "--no-run", "--message-format=json", "--", "tail"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(17));
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "{\"reason\":\"compiler-artifact\",\"marker\":\"original\"}\n"
    );
    assert_eq!(
        String::from_utf8(result.stderr).unwrap(),
        "compiler stderr\n"
    );
    let calls = fs::read_to_string(root.path().join("calls")).unwrap();
    assert_eq!(calls.matches("--unit-graph").count(), 1);
    assert_eq!(calls.matches("tail").count(), 1);
}
#[test]
fn native_correctness_clippy_no_deps_reaches_only_actual_linter() {
    let root = tempfile::tempdir().unwrap();
    pure_cargo(root.path(), 1);
    let result = adapter(root.path())
        .args([
            "clippy",
            "--keep-going",
            "--no-deps",
            "--all-targets",
            "--",
            "-Dwarnings",
        ])
        .output()
        .unwrap();
    assert_eq!(
        result.status.code(),
        Some(17),
        "original linter exit: {result:?}"
    );
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "{\"reason\":\"compiler-artifact\",\"marker\":\"original\"}\n"
    );
    assert_eq!(
        String::from_utf8(result.stderr).unwrap(),
        "compiler stderr\n"
    );
    let calls = fs::read_to_string(root.path().join("calls")).unwrap();
    assert_eq!(calls.matches("--unit-graph").count(), 1);
    assert_eq!(
        calls.matches("--no-deps").count(),
        1,
        "only actual Clippy consumes its flag"
    );
    assert_eq!(
        calls.matches("--keep-going").count(),
        2,
        "both graph and linter retain continuation"
    );
    assert_eq!(calls.matches("--all-targets").count(), 2);
    assert_eq!(calls.matches("-Dwarnings").count(), 1);
}
#[test]
fn native_correctness_unknown_graph_never_executes_compiler() {
    let root = tempfile::tempdir().unwrap();
    pure_cargo(root.path(), 2);
    let result = adapter(root.path()).arg("check").output().unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("unsupported Cargo unit-graph")
    );
    assert_eq!(
        fs::read_to_string(root.path().join("calls"))
            .unwrap()
            .matches("check")
            .count(),
        1
    );
}
#[test]
fn native_correctness_adapter_listing_transparently_uses_real_cargo() {
    let root = tempfile::tempdir().unwrap();
    pure_cargo(root.path(), 2);
    let result = adapter(root.path())
        .env("PSE_CORRECTNESS_REAL_CARGO", root.path().join("cargo"))
        .env("PSE_CORRECTNESS_ADAPTER", "1")
        .env("NEXTEST_TEST_PHASE", "list")
        .args(["metadata", "--format-version=1"])
        .output()
        .unwrap();
    assert!(result.status.success());
    assert_eq!(
        fs::read_to_string(root.path().join("calls")).unwrap(),
        "metadata\n--format-version=1\n"
    );
}
#[test]
fn native_correctness_reuse_refuses_before_runner_launch() {
    let root = tempfile::tempdir().unwrap();
    executable(
        &root.path().join("cargo-nextest"),
        "#!/bin/sh\ntouch launched\nexit 0\n",
    );
    let result = adapter(root.path())
        .env("PATH", root.path())
        .args(["nextest", "run", "--binaries-metadata=fake"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(!root.path().join("launched").exists());
}
#[test]
fn native_correctness_setup_restores_real_cargo_once() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("env");
    fs::write(&destination, "").unwrap();
    let result = adapter(root.path())
        .env("PSE_CORRECTNESS_REAL_CARGO", "/real/cargo")
        .env("NEXTEST_ENV", &destination)
        .arg("--nextest-test-env")
        .output()
        .unwrap();
    assert!(result.status.success(), "{:?}", result);
    assert_eq!(
        fs::read_to_string(destination).unwrap(),
        "CARGO=/real/cargo\nPSE_CORRECTNESS_ADAPTER=0\n"
    );
}
#[test]
fn native_correctness_missing_real_cargo_refuses_recursion() {
    let root = tempfile::tempdir().unwrap();
    let result = adapter(root.path())
        .env(
            "CARGO",
            std::env::var_os("CARGO_BIN_EXE_cargo-pse-correctness")
                .expect("Cargo/nextest supplies the selected runtime correctness executable"),
        )
        .arg("check")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("refusing recursion")
    );
}
#[test]
fn native_correctness_nextest_attempt_observes_restored_cargo() {
    // Cargo's own libtest route has no nextest setup marker. The native nextest journey
    // selects this same test with the marker present and must prove actual restoration.
    if let Some(real) = std::env::var_os("PSE_CORRECTNESS_REAL_CARGO") {
        assert_eq!(std::env::var_os("CARGO"), Some(real));
        assert_ne!(
            std::env::var_os("PSE_CORRECTNESS_ADAPTER"),
            Some("1".into())
        );
    }
}
#[test]
fn native_correctness_nextest_build_adapter_receives_translated_cargo_argv() {
    let root = tempfile::tempdir().unwrap();
    pure_cargo(root.path(), 1);
    executable(
        &root.path().join("cargo-nextest"),
        r#"#!/bin/sh
[ "$1" = nextest ] && [ "$2" = list ] || exit 98
[ "$CARGO" = "$PSE_CORRECTNESS_EXECUTABLE" ] || exit 99
[ -z "${NEXTEST_TEST_PHASE+x}" ] || exit 95
exec "$CARGO" --color=always test --no-run --message-format=json --lib
"#,
    );
    let result = adapter(root.path())
        .env("PATH", format!("{}:/usr/bin:/bin", root.path().display()))
        .env("NEXTEST_TEST_PHASE", "list")
        .args(["nextest", "list", "--lib"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(17));
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "{\"reason\":\"compiler-artifact\",\"marker\":\"original\"}\n"
    );
    let calls = fs::read_to_string(root.path().join("calls")).unwrap();
    assert_eq!(calls.matches("--no-run").count(), 2);
    assert_eq!(calls.matches("--lib").count(), 2);
    assert_eq!(calls.matches("--unit-graph").count(), 1);
}
#[test]
fn native_correctness_version_requires_no_cargo_or_source_graph() {
    let root = tempfile::tempdir().unwrap();
    let result = adapter(root.path())
        .env("CARGO", "/missing/cargo")
        .arg("--version")
        .output()
        .unwrap();
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!(
            "cargo-pse-correctness {} protocol 1\n",
            env!("CARGO_PKG_VERSION")
        )
    );
    assert!(result.stderr.is_empty());
}

#[test]
fn native_correctness_discovery_marker_cannot_bypass_compiler_guard() {
    let root = tempfile::tempdir().unwrap();
    pure_cargo(root.path(), 2);
    let result = adapter(root.path())
        .env("PSE_CORRECTNESS_ADAPTER", "1")
        .env("PSE_CORRECTNESS_REAL_CARGO", root.path().join("cargo"))
        .env("NEXTEST_TEST_PHASE", "list")
        .arg("check")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("unsupported Cargo unit-graph")
    );
}

#[test]
fn native_correctness_workspace_and_config_overrides_refuse_before_runner() {
    for option in [
        "--manifest-path=other/Cargo.toml",
        "--config-file=other.toml",
        "--tool-config-file=tool:/other.toml",
    ] {
        let root = tempfile::tempdir().unwrap();
        executable(
            &root.path().join("cargo-nextest"),
            "#!/bin/sh\ntouch launched\n",
        );
        let result = adapter(root.path())
            .env("PATH", root.path())
            .args(["nextest", "run", option])
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2), "{option}");
        assert!(!root.path().join("launched").exists());
    }
}
#[test]
fn native_correctness_foreign_workspace_refuses_before_runner() {
    let root = tempfile::tempdir().unwrap();
    pure_cargo(root.path(), 1);
    fs::write(root.path().join("Cargo.toml"), "[workspace]\n").unwrap();
    executable(
        &root.path().join("cargo-nextest"),
        "#!/bin/sh\ntouch launched\n",
    );
    let result = adapter(root.path())
        .env("PSE_TEST_CHECKOUT", root.path())
        .env("PATH", format!("{}:/usr/bin:/bin", root.path().display()))
        .args(["nextest", "run"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("foreign workspace")
    );
    assert!(!root.path().join("launched").exists());
}
