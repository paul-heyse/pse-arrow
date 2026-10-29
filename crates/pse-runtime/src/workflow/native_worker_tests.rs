// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Focused native scientific worker handoff and ownership controls.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "native worker protocol assertions"
)]
use super::*;

fn inputs(directory: &Path) -> NativeStudyWorkerInputs {
    NativeStudyWorkerInputs { version: 1, state: directory.join("canonical"), endpoint: "grpc://127.0.0.1:12345".into(),
        namespace: "native_fixture".into(), database: "canonical".into(),
        fence: serde_json::from_value(serde_json::json!({
            "schema_version": 3, "schema_epoch": "00000000-0000-0000-0000-000000000001",
            "schema_digest": "test-opening-premise", "issuing_store": "00000000-0000-0000-0000-000000000002"
        })).unwrap(), unit: "pse-native-worker.service".into(),
        allocation: NativeAllocation::materialize(4 << 30, 1 << 30, 1, 2 << 30, None).unwrap(),
        settings: WorkerSettings::default(), nonce: "a".repeat(32), ready: directory.join("ready.json"), terminal: directory.join("terminal.json") }
}
fn observation() -> UnitObservation {
    UnitObservation {
        unit: "pse-native-worker.service".into(),
        invocation: vec![1; 16],
        active_state: "active".into(),
        sub_state: "running".into(),
        main_pid: 123,
        exit_code: 0,
        exit_status: 0,
        control_group: "/original".into(),
    }
}
fn ready(inputs: &NativeStudyWorkerInputs) -> Ready {
    Ready {
        nonce: inputs.nonce.clone(),
        pid: 123,
        invocation: "01".repeat(16),
        endpoint: inputs.endpoint.clone(),
        namespace: inputs.namespace.clone(),
        database: inputs.database.clone(),
        fence: inputs.fence.clone(),
        allocation: inputs.allocation,
        settings: inputs.settings,
        diagnostics: serde_json::Value::Null,
    }
}

#[test]
fn native_worker_requires_application_claim_for_original_invocation_and_policy() {
    let directory = tempfile::tempdir().unwrap();
    let inputs = inputs(directory.path());
    let observation = observation();
    for field in 0..9 {
        let mut ready = ready(&inputs);
        match field {
            0 => ready.nonce = "b".repeat(32),
            1 => ready.pid += 1,
            2 => ready.invocation = "02".repeat(16),
            3 => ready.endpoint.push('x'),
            4 => ready.namespace.push('x'),
            5 => ready.database.push('x'),
            6 => ready.allocation.native_worker_memory_bytes += 1,
            8 => ready.fence = serde_json::from_value(serde_json::json!({"schema_version":3,"schema_epoch":"other","schema_digest":"test-opening-premise","issuing_store":"other"})).unwrap(),
            _ => ready.settings.poll += Duration::from_millis(1),
        }
        assert!(
            require_ready(&inputs, &observation, &ready).is_err(),
            "changed field {field}"
        );
    }
    assert!(require_ready(&inputs, &observation, &ready(&inputs)).is_ok());
    assert!(
        !inputs.ready.exists(),
        "Type=exec facts have not published application readiness"
    );
}

#[test]
fn native_worker_partial_or_untrusted_document_never_becomes_readiness() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let directory = tempfile::tempdir().unwrap();
    let inputs = inputs(directory.path());
    let path = directory.path().join("inputs.json");
    private_write(&path, &inputs).unwrap();
    assert_eq!(
        NativeStudyWorkerInputs::read(&path).unwrap().database(),
        "canonical"
    );
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(NativeStudyWorkerInputs::read(&path).is_err());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let link = directory.path().join("alias.json");
    symlink(&path, &link).unwrap();
    assert!(NativeStudyWorkerInputs::read(&link).is_err());
    fs::write(&path, b"{\"version\":1").unwrap();
    assert!(NativeStudyWorkerInputs::read(&path).is_err());
    fs::write(&path, vec![b' '; DOCUMENT_BYTES as usize + 1]).unwrap();
    assert!(NativeStudyWorkerInputs::read(&path).is_err());
}

#[test]
fn native_worker_refuses_unknown_version_and_non_sustained_or_unbounded_case_inputs() {
    let directory = tempfile::tempdir().unwrap();
    let original = inputs(directory.path());
    for change in 0..5 {
        let mut inputs = original.clone();
        match change {
            0 => inputs.version = 2,
            1 => inputs.settings.until_idle = true,
            2 => inputs.settings.maximum_actions = Some(1),
            3 => inputs.settings.maximum_in_flight = Some(0),
            _ => inputs.settings.poll = Duration::ZERO,
        }
        assert!(inputs.validate().is_err());
    }
    let profiles = super::super::ProductionProfiles::read(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.config/agent-capacity.toml"),
    )
    .unwrap();
    let mut selected = original;
    selected.allocation = profiles
        .select(super::super::ProductionProfileName::Reference)
        .unwrap();
    assert!(
        selected.validate().is_err(),
        "case lanes must be materialized before launch"
    );
    selected.settings.maximum_in_flight = Some(16);
    selected.validate().unwrap();
    let execution = selected.allocation.execution.unwrap();
    assert_eq!(
        (
            execution.cpu_threads,
            execution.case_lanes,
            execution.math_jobs
        ),
        (16, 16, 32)
    );
    selected.settings.maximum_in_flight = Some(17);
    assert!(selected.validate().is_err());
}

#[test]
fn native_worker_unknown_process_facts_never_prove_original_leader_exit() {
    let ProcessRead::Present(facts) = ProcessReader::default().process(std::process::id()) else {
        panic!("own process facts unavailable")
    };
    assert!(!original_leader_ended(
        &facts.identity,
        ProcessRead::Unavailable("denied".into())
    ));
    assert!(!original_leader_ended(
        &facts.identity,
        ProcessRead::Changed
    ));
    assert!(!original_leader_ended(
        &facts.identity,
        ProcessRead::Present(facts.clone())
    ));
    let mut replacement = facts.clone();
    replacement.identity.start_ticks += 1;
    assert!(original_leader_ended(
        &facts.identity,
        ProcessRead::Present(replacement)
    ));
    assert!(original_leader_ended(&facts.identity, ProcessRead::Absent));
}

#[test]
fn native_worker_real_original_leader_exit_is_distinct_from_native_join_receipt() {
    let mut child = std::process::Command::new("/bin/sleep")
        .arg("60")
        .spawn()
        .unwrap();
    let ProcessRead::Present(facts) = ProcessReader::default().process(child.id()) else {
        let _ = child.kill();
        let _ = child.wait();
        panic!("child process identity unavailable")
    };
    assert!(!original_leader_ended(
        &facts.identity,
        ProcessReader::default().process(child.id())
    ));
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(original_leader_ended(
        &facts.identity,
        ProcessReader::default().process(child.id())
    ));
    let directory = tempfile::tempdir().unwrap();
    let inputs = inputs(directory.path());
    assert!(
        !inputs.terminal.exists(),
        "host process death has not fabricated an application join receipt"
    );
}

#[test]
fn native_worker_multiple_processes_keep_per_process_and_aggregate_envelopes() {
    let profile = super::super::NativeExecutionAllocation {
        pool_memory_bytes: 6 << 30,
        worker_bytes: 1 << 30,
        cpu_threads: 32,
        case_lanes: 16,
        math_jobs: 32,
        compiler_cores: 32,
        admission_wait_ms: 30_000,
        process_headroom_bytes: 2 << 30,
        observer_memory_bytes: 8 << 30,
    };
    let allocation =
        NativeAllocation::materialize(28 << 30, 2 << 30, 2, 8 << 30, Some(profile)).unwrap();
    assert_eq!(allocation.native_workers, 2);
    assert_eq!(allocation.execution, Some(profile));
    assert!(NativeAllocation::materialize(25 << 30, 2 << 30, 2, 8 << 30, Some(profile)).is_err());
    assert!(NativeAllocation::materialize(28 << 30, 2 << 30, 3, 8 << 30, Some(profile)).is_err());
}

#[test]
fn native_worker_refuses_unrepresentable_service_environment_before_dispatch() {
    for name in ["", "1START", "NEXTEST_BIN_EXE_pse-worker", "BAD=NAME", "λ"] {
        assert!(service_environment_entry(name, "value").is_err());
    }
    let value = "$HOME %n = literal\t\nλ";
    assert_eq!(
        service_environment_entry("_NATIVE_58", value).unwrap(),
        format!("_NATIVE_58={value}")
    );
    assert!(service_environment_entry("NATIVE", "bad\0value").is_err());
}
