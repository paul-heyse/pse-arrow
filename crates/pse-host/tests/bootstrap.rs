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
fn launcher(root: &Path) -> Command {
    let launcher = root.join("scripts/pse");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/pse"),
        &launcher,
    )
    .unwrap();
    let mut command = Command::new(launcher);
    command
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", root.join("bin").display()),
        )
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

#[test]
fn cold_pse_help_and_invalid_preparation_leave_the_checkout_untouched() {
    let root = checkout();
    let path = root.path();
    executable(
        &path.join("bin/cargo"),
        "#!/bin/sh\ntouch entered-construction\nexit 98\n",
    );
    for arguments in [
        &[][..],
        &["--help"][..],
        &["prepare", "--help"][..],
        &["prepare", "host", "--help"][..],
    ] {
        let result = launcher(path).args(arguments).output().unwrap();
        assert!(result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stdout).contains("pse prepare host [--from CHECKOUT]")
        );
    }
    let unavailable = launcher(path).arg("commands").output().unwrap();
    assert_eq!(unavailable.status.code(), Some(125));
    assert!(String::from_utf8_lossy(&unavailable.stderr).contains("run pse prepare host"));
    for suffix in [&["--unknown"][..], &["--from:unknown", "another-path"][..]] {
        let rejected = launcher(path)
            .args(["prepare", "host"])
            .args(suffix)
            .output()
            .unwrap();
        assert_eq!(rejected.status.code(), Some(2));
    }
    assert!(!path.join("entered-construction").exists());
    assert!(!path.join("target/host-tools/.construction.lock").exists());
    unchanged(path);
}

#[test]
fn cold_pse_hands_explicit_preparation_to_the_fixed_source_owner() {
    let root = checkout();
    let path = root.path();
    let source = source_checkout(
        "#!/bin/sh\n[ \"$#\" = 5 ] && [ \"$1\" = -C ] && [ \"$3\" = __host-generation-bootstrap ] && [ \"$4\" = --from ] || exit 93\n[ \"$PSE_BOOTSTRAP_SOURCE_HELPER\" = \"$5\" ] || exit 96\nprintf 'source helper called\\n'\n",
    );
    executable(&path.join("bin/cargo"), "#!/bin/sh\nexit 98\n");
    let result = launcher(path)
        .args(["prepare", "host", "--from"])
        .arg(source.path())
        .output()
        .unwrap();
    assert!(result.status.success());
    assert_eq!(result.stdout, b"source helper called\n");
    assert!(!path.join("target/host-tools/.construction.lock").exists());
}

#[test]
fn preparation_receipt_contains_only_the_owner_route_and_releases_construction_lock() {
    let root = checkout();
    let path = root.path();
    executable(
        &path.join("bin/cargo"),
        "#!/bin/sh\nprintf 'cargo diagnostic\\n'\n",
    );
    executable(
        &path.join("target/debug/pse-host"),
        "#!/bin/sh\nflock -n target/host-tools/.construction.lock /bin/true || exit 93\nprintf 'native diagnostic\\n'\n",
    );
    let result = seed(path).arg("--report-route").output().unwrap();
    assert!(result.status.success());
    assert_eq!(result.stdout, b"local-seed\n");
    let diagnostics = String::from_utf8_lossy(&result.stderr);
    assert!(diagnostics.contains("cargo diagnostic"));
    assert!(diagnostics.contains("native diagnostic"));

    let source = source_checkout(
        "#!/bin/sh\n[ \"$PSE_BOOTSTRAP_SOURCE_HELPER\" = \"$5\" ] || exit 93\nprintf 'source diagnostic\\n'\n",
    );
    let result = seed(path)
        .args(["--report-route", "--from"])
        .arg(source.path())
        .output()
        .unwrap();
    assert!(result.status.success());
    assert_eq!(result.stdout, b"matching-source\n");
    assert!(String::from_utf8_lossy(&result.stderr).contains("source diagnostic"));
}

#[test]
fn preparation_receipt_preserves_original_failure_status() {
    use pse_host::generation::{BuildContext, GenerationManifest, SourceFingerprint};

    let root = checkout();
    let path = root.path();
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(path)
            .status()
            .unwrap()
            .success()
    );
    fs::create_dir(path.join(".config")).unwrap();
    fs::write(path.join(".config/diagnostic-tools.toml"), "").unwrap();
    fs::create_dir(path.join(".cargo")).unwrap();
    fs::write(path.join(".cargo/config.toml"), "").unwrap();
    fs::write(
        path.join("rust-toolchain.toml"),
        include_str!("../../../rust-toolchain.toml"),
    )
    .unwrap();
    fs::write(
        path.join("Cargo.toml"),
        "[package]\nname = 'pse-host'\nversion = '0.0.1'\nedition = '2024'\n",
    )
    .unwrap();
    fs::create_dir(path.join("src")).unwrap();
    fs::write(path.join("src/lib.rs"), "").unwrap();
    let metadata = cargo_metadata::MetadataCommand::new()
        .current_dir(path)
        .other_options(vec!["--offline".into()])
        .exec()
        .unwrap();
    let fingerprint = SourceFingerprint::from_metadata(
        path,
        &metadata,
        BuildContext::new(
            "fixture rustc".into(),
            "fixture Cargo".into(),
            "dev".into(),
            Default::default(),
            &Default::default(),
        ),
    )
    .unwrap();
    let selected = path.join("target/host-tools/current");
    let generation = path.join("target/host-tools/generation.fixture");
    fs::create_dir(&generation).unwrap();
    executable(&generation.join("pse"), "#!/bin/sh\nexit 0\n");
    let manifest =
        GenerationManifest::capture(&generation, fingerprint.clone(), &fingerprint, &["pse"])
            .unwrap();
    manifest.write(&generation).unwrap();
    fs::remove_file(&selected).unwrap();
    std::os::unix::fs::symlink("generation.fixture", &selected).unwrap();

    let invoke = || {
        let mut command = Command::new(
            std::env::var_os("CARGO_BIN_EXE_pse")
                .expect("Cargo/nextest supplies the selected CLI executable"),
        );
        command
            .arg("-C")
            .arg(path)
            .args(["prepare", "host", "--format", "json"]);
        command
    };
    executable(
        &path.join("scripts/bootstrap-host"),
        "#!/bin/sh\n[ \"$1\" = --report-route ] || exit 92\nprintf x >> \"$(dirname \"$0\")/invocations\"\nprintf 'verified-current\\n'\n",
    );
    let prepared = invoke().output().unwrap();
    assert!(prepared.status.success());
    let response: serde_json::Value = serde_json::from_slice(&prepared.stdout).unwrap();
    assert_eq!(response["payload"]["route"], "verified_current");
    assert_eq!(
        response["payload"]["selected_generation"]["generation"],
        "generation.fixture"
    );
    assert_eq!(
        response["payload"]["selected_generation"]["manifest_digest"],
        manifest.digest().unwrap()
    );
    let repeated = invoke().output().unwrap();
    assert!(repeated.status.success());
    let repeated: serde_json::Value = serde_json::from_slice(&repeated.stdout).unwrap();
    assert_eq!(repeated["payload"], response["payload"]);
    let unsupported = invoke().args(["--fields", "unsupported"]).output().unwrap();
    assert!(!unsupported.status.success());
    assert!(
        String::from_utf8_lossy(&unsupported.stderr).contains("unknown payload field unsupported")
    );
    assert_eq!(fs::read(path.join("scripts/invocations")).unwrap(), b"xx");

    fs::remove_file(&selected).unwrap();
    let missing = invoke().output().unwrap();
    assert!(!missing.status.success());
    let diagnostic = String::from_utf8_lossy(&missing.stderr);
    assert!(
        diagnostic.contains("preparation did not leave an authenticated generation"),
        "{diagnostic}"
    );

    executable(&path.join("scripts/bootstrap-host"), "#!/bin/sh\nexit 79\n");
    let failed = invoke().output().unwrap();
    assert_eq!(failed.status.code(), Some(79));
    let response: serde_json::Value = serde_json::from_slice(&failed.stdout).unwrap();
    assert_eq!(response["outcome"], "boundary_failure");
    assert_eq!(response["child_exit"], 79);
    assert_eq!(response["next"][0]["argv"][1], "prepare");
}
