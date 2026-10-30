// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native preparation entry-point refusal and kernel lifetime fixtures.
#![cfg(test)]
use pse_host::{environment::Environment, native};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[test]
fn native_cli_rejects_unsupported_profiles_without_effects() {
    let directory = tempfile::tempdir().unwrap();
    for profile in [",", "ipopt", "compiler,ipopt", "compiler,klu,ipopt"] {
        let output = Command::new(env!("CARGO_BIN_EXE_pse"))
            .args(["native", "prepare", "--capabilities", profile])
            .current_dir(directory.path())
            .env("PSE_NATIVE_CACHE", directory.path().join("cache"))
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(125), "{profile}: {output:?}");
    }
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
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
