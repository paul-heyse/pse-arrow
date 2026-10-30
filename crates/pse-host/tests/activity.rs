// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Diagnosis must survive unavailable manager and environment setup.
#![cfg(test)]
use std::process::Command;

#[test]
fn unavailable_bus_preserves_partial_diagnosis_without_activation() {
    let directory = tempfile::tempdir().unwrap();
    let output = Command::new(
        std::env::var_os("CARGO_BIN_EXE_pse-host")
            .expect("Cargo/nextest supplies the selected runtime host executable"),
    )
    .args(["activity", "selected.service"])
    .env(
        "DBUS_SESSION_BUS_ADDRESS",
        format!("unix:path={}/absent-bus", directory.path().display()),
    )
    .env("VIRTUAL_ENV", directory.path().join("absent-python"))
    .env("RUSTC_WRAPPER", directory.path().join("absent-compiler"))
    .env("PSE_SURREAL_STATE", directory.path().join("absent-store"))
    .env("PSE_RESOURCE_CLASS", "invalid-admission")
    .output()
    .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["observation"]["completeness"], "partial");
    let report = &report["payload"];
    assert_eq!(report["requested_unit"], "selected.service");
    assert!(report["manager"].is_null());
    assert!(
        report["manager_unavailable"]
            .as_str()
            .is_some_and(|detail| !detail.is_empty())
    );
    assert!(report["observer"]["identity"]["pid"].is_number());
    assert!(
        report["observer"]["control_group"]
            .as_str()
            .unwrap()
            .starts_with('/')
    );
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
}
