// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Acquisition controls use the native report publisher and the actual retained reader.
use super::*;
use crate::{
    envelope::Settlement,
    invocation::{InvocationEvidence, InvocationSpec},
    test_workflow::execute::{ContentIdentity, Input},
};
use std::{fs, os::unix::fs::PermissionsExt as _};

async fn retained_failure(
    scratch: &Path,
) -> (InvocationStore, CliInvocationId, std::path::PathBuf) {
    let input: Input = serde_json::from_slice(&full_original(false).files[0].original).unwrap();
    let environment = crate::environment::Environment::from([(
        "XDG_STATE_HOME".into(),
        scratch.join("state").into_os_string(),
    )]);
    let state = crate::paths::CheckoutState::from_paths(
        &scratch.join("checkout"),
        &scratch.join("common.git"),
        &environment,
    )
    .unwrap();
    let store = InvocationStore::new(state);
    let (binding, encoded) = crate::managed::workflow::encode_input(&input).unwrap();
    let directory = store
        .begin(InvocationSpec {
            invocation_id: input.invocation.clone(),
            command: input.action.command().into(),
            session: None,
            parent: None,
            operation: serde_json::json!({"workflow": binding}),
            control: crate::generation::PreparedGeneration {
                generation: "generation.acquisition-control".into(),
                protocol: 1,
                manifest_digest: "a".repeat(64),
            },
            environment_digest: "b".repeat(64),
            scientific: vec![],
        })
        .unwrap();
    directory
        .publish_evidence(InvocationEvidence::WorkflowInput, &encoded)
        .unwrap();
    let parent = directory.path().join("workflow");
    let mut report = crate::test_workflow::execute::run(
        &input,
        &scratch.join("checkout"),
        &parent,
        crate::environment::Environment::new(),
    )
    .await
    .unwrap();
    assert_eq!(report.outcome, Outcome::Failed);
    assert_eq!(report.failures.len(), 1);
    report.input_cutoff = Some(Ok(ContentIdentity::of(
        &fs::read(directory.path().join("workflow-input.json")).unwrap(),
    )));
    let terminal = report.publish(&parent).unwrap();
    // This passive manager fixture deliberately records unresolved settlement. It does
    // not create a native manager capability or claim an actual interrupted process.
    let mut manager = Envelope::response(
        input.action.command(),
        "recorded unresolved manager",
        ManagedSummary {
            bundle: None,
            workflow: Some(WorkflowSummary {
                terminal,
                report: "workflow/report.json".into(),
            }),
            native_lifetime_pending: Some(true),
            child_exit: Some(1),
            stdout: None,
            stderr: None,
            borrowed_reservation: false,
            service_result: None,
        },
    );
    manager.invocation_id = input.invocation.clone();
    manager.settlement = Settlement::Unknown;
    crate::invocation::publish_bytes(
        &directory.path().join("result.json"),
        &serde_json::to_vec(&manager).unwrap(),
        None,
        0o400,
    )
    .unwrap();
    (store, input.invocation, parent)
}

#[tokio::test]
async fn retained_report_without_inventory_exports_without_promoting_native_settlement() {
    let scratch = tempfile::tempdir().unwrap();
    let (store, id, parent) = retained_failure(scratch.path()).await;
    let report_before = fs::read(parent.join("report.json")).unwrap();
    for scope in [EvidenceScope::Report, EvidenceScope::Full] {
        let output = scratch.path().join(format!("{scope:?}.bin"));
        let receipt = export(&store, id.as_str(), None, &output, scope).unwrap();
        let bytes = fs::read(&output).unwrap();
        let readback =
            inspect(&descriptors(), &receipt.message, &bytes, &binary_options()).unwrap();
        assert_eq!(
            readback.recorded_manager_settlement,
            Some(Settlement::Unknown)
        );
        assert_eq!(readback.recorded_native_lifetime_pending, Some(true));
        assert!(readback.selected_tests.is_none());
        assert!(readback.selected_population_complete.is_none());
        assert_eq!(readback.scope, scope);
        if scope == EvidenceScope::Full {
            assert_eq!(readback.included_files, 1);
            assert_eq!(readback.full_original_bytes_available, Some(true));
        }
        assert!(export(&store, id.as_str(), None, &output, scope).is_err());
        assert_eq!(
            fs::read(output).unwrap(),
            bytes,
            "exclusive publication preserves original output"
        );
    }
    let output = scratch.path().join("selection.bin");
    assert!(
        matches!(
            export(&store, id.as_str(), None, &output, EvidenceScope::Selection),
            Err(HostError::Ownership(_)),
        ),
        "absent original inventory is an unavailable association, not a test failure"
    );
    assert!(!output.exists());
    assert_eq!(fs::read(parent.join("report.json")).unwrap(), report_before);
    assert!(
        !parent.join("inventory").exists(),
        "a read cannot prepare missing population"
    );
}

#[tokio::test]
async fn failed_full_acquisition_never_publishes_or_repairs_committed_dependencies() {
    let scratch = tempfile::tempdir().unwrap();
    let (store, id, parent) = retained_failure(scratch.path()).await;
    let report_before = fs::read(parent.join("report.json")).unwrap();
    let input = parent.parent().unwrap().join("workflow-input.json");
    let before = fs::read(&input).unwrap();
    assert!(
        crate::test_inventory::captured_full_report(
            &store,
            id.as_str(),
            None,
            report_before.len() + before.len() - 1,
        )
        .is_err(),
        "the entire original closure must fit before publication"
    );
    let mut changed = before.clone();
    changed[0] = b'[';
    fs::set_permissions(&input, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&input, &changed).unwrap();
    fs::set_permissions(&input, fs::Permissions::from_mode(0o400)).unwrap();
    let output = scratch.path().join("corrupt.bin");
    assert!(
        matches!(
            export(&store, id.as_str(), None, &output, EvidenceScope::Full),
            Err(HostError::Ownership(_)),
        ),
        "changed committed bytes retain the native integrity refusal"
    );
    assert!(!output.exists());
    assert_eq!(fs::read(&input).unwrap(), changed);
    assert_eq!(
        fs::metadata(&input).unwrap().permissions().mode() & 0o777,
        0o400
    );
    fs::remove_file(&input).unwrap();
    let output = scratch.path().join("missing.bin");
    assert!(
        matches!(
            export(&store, id.as_str(), None, &output, EvidenceScope::Full),
            Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound,
        ),
        "missing original bytes retain the actual acquisition error"
    );
    assert!(!output.exists());
    assert!(
        !input.exists(),
        "failed acquisition cannot recreate the missing original"
    );
    let report_output = scratch.path().join("report-still-readable.bin");
    export(
        &store,
        id.as_str(),
        None,
        &report_output,
        EvidenceScope::Report,
    )
    .unwrap();
    assert_eq!(fs::read(parent.join("report.json")).unwrap(), report_before);
}
