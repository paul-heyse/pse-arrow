// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native preparation entry-point refusal and kernel lifetime fixtures.
#![cfg(test)]
use pse_diagnostics::{DrainFailurePolicy, SupervisedChild};
use pse_host::{environment::Environment, native};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    os::unix::fs::PermissionsExt as _,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[test]
#[ignore = "runs actual compiler/KLU preparation twice in a private cache"]
fn native_preparation_cancellation_preserves_original_pointer_and_retries() {
    let directory = tempfile::tempdir().unwrap();
    let cache = directory.path().join("cache");
    let profile = cache.join("rust-v1/kinsol");
    let marker = directory.path().join("entered-build");
    let selected = std::env::var_os("CMAKE").unwrap_or_else(|| "cmake".into());
    let cmake = if PathBuf::from(&selected).is_absolute() {
        PathBuf::from(selected)
    } else {
        std::env::split_paths(&std::env::var_os("PATH").unwrap())
            .map(|path| path.join(&selected))
            .find(|path| path.is_file())
            .expect("prepared CMake executable")
    };
    let wrapper = directory.path().join("cmake");
    std::fs::write(
        &wrapper,
        r#"#!/bin/sh
set -eu
if [ "${1-}" = --build ] && [ "$PSE_TEST_BLOCK_CMAKE" = 1 ]; then
    printf 'build\n' > "$PSE_TEST_CMAKE_MARKER"
    exec /bin/sleep 600
fi
exec "$PSE_TEST_REAL_CMAKE" "$@"
"#,
    )
    .unwrap();
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o700)).unwrap();

    let prepare = |controlled: bool, blocked: bool| {
        let mut command = Command::new(std::env::var_os("CARGO_BIN_EXE_pse-host").unwrap());
        command
            .args(["native", "prepare", "--capabilities", "compiler,klu"])
            .env("PSE_NATIVE_CACHE", &cache)
            .env("CMAKE", if controlled { &wrapper } else { &cmake })
            .env("PSE_TEST_REAL_CMAKE", &cmake)
            .env("PSE_TEST_CMAKE_MARKER", &marker)
            .env("PSE_TEST_BLOCK_CMAKE", if blocked { "1" } else { "0" })
            .env_remove("SUITESPARSE_INCLUDE_DIR")
            .env_remove("SUITESPARSE_LIBRARY_DIR")
            .env_remove("PSE_KLU_PROVIDER_RECEIPT")
            .stdout(Stdio::null());
        SupervisedChild::spawn(
            &mut command,
            DrainFailurePolicy::Refuse(Duration::from_secs(10)),
            None,
        )
        .unwrap()
    };
    let complete = |mut owner: SupervisedChild| {
        owner
            .wait_ready(None, Some(Instant::now() + Duration::from_secs(120)))
            .unwrap();
        assert!(owner.drain().unwrap().success());
    };
    complete(prepare(false, false));
    let pointer = profile.join("current.json");
    let original = std::fs::read(&pointer).unwrap();
    let mut blocked = prepare(true, true);
    let deadline = Instant::now() + Duration::from_secs(90);
    while !marker.is_file() {
        assert!(
            Instant::now() < deadline,
            "original preparation did not enter build"
        );
        assert!(
            blocked.try_wait().unwrap().is_none(),
            "preparation exited before build"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(std::fs::read_to_string(&marker).unwrap(), "build\n");
    let locks = std::fs::read_dir(profile.join("locks")).unwrap();
    assert_eq!(
        locks
            .filter(|entry| {
                let lock = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(entry.as_ref().unwrap().path())
                    .unwrap();
                lock.try_lock().is_err()
            })
            .count(),
        1,
        "the performing build must still own exactly one recipe lock"
    );
    assert!(!blocked.cancel().unwrap().success());
    assert_eq!(std::fs::read(&pointer).unwrap(), original);
    complete(prepare(true, false));
    assert_ne!(std::fs::read(pointer).unwrap(), original);
}

#[test]
fn native_cli_rejects_unsupported_profiles_without_effects() {
    let directory = tempfile::tempdir().unwrap();
    let cache = directory.path().join("uncreated-native-cache");
    for profile in [",", "ipopt", "compiler,ipopt", "compiler,klu,ipopt"] {
        let output = Command::new(
            std::env::var_os("CARGO_BIN_EXE_pse-host")
                .expect("Cargo/nextest supplies the selected runtime host executable"),
        )
        .args([
            "--format",
            "json",
            "native",
            "prepare",
            "--capabilities",
            profile,
        ])
        .env("PSE_NATIVE_CACHE", &cache)
        .output()
        .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["outcome"], "usage");
        assert_eq!(error["failures"][0]["code"], "cli.usage");
        let parser = &error["failures"][0]["parser_context"];
        assert_eq!(parser["error_kind"], "ValueValidation");
        assert_eq!(parser["InvalidValue"], profile);
        assert_eq!(parser["InvalidArg"], "--capabilities <CAPABILITIES>");
        assert!(!cache.exists());
    }
}
#[test]
fn native_missing_selection_reports_explicit_preparation_without_creation() {
    let directory = tempfile::tempdir().unwrap();
    let missing = directory.path().join("missing");
    let mut environment = Environment::new();
    environment.insert("PSE_NATIVE_CACHE".into(), missing.clone().into_os_string());
    let error = native::select(directory.path(), &environment)
        .unwrap_err()
        .to_string();
    assert!(error.contains("native prepare --capabilities compiler,klu"));
    assert!(!missing.exists());
}
#[test]
#[ignore = "subprocess fixture invoked only by the cancellation test"]
fn native_lock_holder() {
    let path = std::env::var_os("PSE_NATIVE_LOCK_FIXTURE").unwrap();
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .read(true)
        .open(&path)
        .unwrap();
    file.lock().unwrap();
    let mut ready = File::create(std::env::var_os("PSE_NATIVE_READY_FIXTURE").unwrap()).unwrap();
    ready.write_all(b"locked").unwrap();
    loop {
        std::thread::park();
    }
}
#[test]
fn native_cancellation_releases_kernel_lock_without_publishing() {
    let directory = tempfile::tempdir().unwrap();
    let lock = directory.path().join("recipe.lock");
    let ready = directory.path().join("ready");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "native_lock_holder", "--ignored"])
        .env("PSE_NATIVE_LOCK_FIXTURE", &lock)
        .env("PSE_NATIVE_READY_FIXTURE", &ready)
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ready.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    if !ready.exists() {
        let _ = child.kill();
        let _ = child.wait();
        panic!("native lock fixture did not become ready");
    }
    let file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(&lock)
        .unwrap();
    assert!(file.try_lock().is_err());
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(file.try_lock().is_ok());
    assert!(!directory.path().join("current.json").exists());
}
