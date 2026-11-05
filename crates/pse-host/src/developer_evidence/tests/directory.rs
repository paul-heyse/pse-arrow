// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native custody/export/read controls; fixture reports retain incomplete outcomes.
use super::*;
use crate::{
    envelope::Settlement,
    invocation::{InvocationEvidence, InvocationSpec},
};
use pse_contract_proto::support::encode_root;
use std::{fs, num::NonZeroU64, os::unix::fs::PermissionsExt as _, path::PathBuf};

fn revise(wire: &mut pb::CapturedWorkflowReport, report: serde_json::Value) {
    let bytes = serde_json::to_vec(&report).unwrap();
    let observation: ReportObservation = serde_json::from_slice(&bytes).unwrap();
    wire.report_observation =
        encode_root::<_, pb::WorkflowReportObservation>(observation, "report_observation")
            .unwrap()
            .into();
    let mut terminal: TerminalRecord =
        decode_root(wire.terminal.clone().into_option().unwrap(), "terminal").unwrap();
    terminal.report.sha256 = digest(&bytes);
    terminal.report.byte_length = bytes.len() as u64;
    wire.terminal = encode_root::<_, pb::WorkflowTerminalRecord>(terminal, "terminal")
        .unwrap()
        .into();
    wire.report_original = bytes;
}

fn fixture(
    scratch: &Path,
    extent: u64,
    omitted: bool,
) -> (InvocationStore, CliInvocationId, PathBuf) {
    let mut wire = full_original(omitted);
    let input: Input = serde_json::from_slice(&wire.files[0].original).unwrap();
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
            command: "test".into(),
            session: None,
            parent: None,
            operation: serde_json::json!({"workflow":binding}),
            control: crate::generation::PreparedGeneration {
                generation: "generation.directory-control".into(),
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
    fs::create_dir(&parent).unwrap();
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
    let raw = parent.join("run-0.stdout");
    crate::invocation::publish_file(&raw, 0o400, |file| {
        std::io::copy(&mut std::io::repeat(0).take(extent), file)?;
        Ok(())
    })
    .unwrap();
    let mut report: serde_json::Value = serde_json::from_slice(&wire.report_original).unwrap();
    report["evidence"][0]["content"] = serde_json::to_value(ContentIdentity::of(
        &fs::read(directory.path().join("workflow-input.json")).unwrap(),
    ))
    .unwrap();
    if !omitted {
        let mut hash = Sha256::new();
        let mut file = fs::File::open(raw).unwrap();
        let mut buffer = [0; 65536];
        loop {
            let count = file.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
        }
        let identity = ContentIdentity {
            byte_length: extent,
            sha256: hash
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        };
        report["evidence"][1]["content"] = serde_json::to_value(identity).unwrap();
    }
    revise(&mut wire, report);
    crate::invocation::publish_bytes(
        &parent.join("report.json"),
        &wire.report_original,
        None,
        0o400,
    )
    .unwrap();
    let terminal = decode_root(wire.terminal.into_option().unwrap(), "terminal").unwrap();
    let mut manager = Envelope::response(
        "test",
        "passive original incomplete directory fixture",
        ManagedSummary {
            bundle: None,
            workflow: Some(WorkflowSummary {
                terminal,
                report: "workflow/report.json".into(),
            }),
            native_lifetime_pending: Some(true),
            child_exit: Some(125),
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

pub(crate) fn exported(scratch: &Path, omitted: bool) -> (PathBuf, PathBuf) {
    let (store, id, parent) = fixture(scratch, 16, omitted);
    let output = scratch.join("bundle");
    export_directory(&store, id.as_str(), None, &output).unwrap();
    (output, parent)
}

fn manifest(path: &Path) -> pb::CapturedWorkflowDirectoryManifest {
    pb::CapturedWorkflowDirectoryManifest::decode_from_slice(
        &fs::read(path.join("manifest.pb")).unwrap(),
    )
    .unwrap()
}
fn replace(path: &Path, bytes: &[u8]) {
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o400)).unwrap();
}

#[test]
fn directory_export_and_offline_read_exceed_compact_without_promoting_original_outcome() {
    let scratch = tempfile::tempdir().unwrap();
    let (store, id, parent) = fixture(scratch.path(), 65 * 1024 * 1024, false);
    let compact = scratch.path().join("compact.pb");
    assert!(export(&store, id.as_str(), None, &compact, EvidenceScope::Full).is_err());
    assert!(!compact.exists());
    let output = scratch.path().join("bundle");
    let receipt = export_directory(&store, id.as_str(), None, &output).unwrap();
    assert!(receipt.member_bytes > 64 * 1024 * 1024);
    assert_eq!(receipt.readback.full_original_bytes_available, Some(true));
    assert_eq!(
        receipt.readback.recorded_manager_settlement,
        Some(Settlement::Unknown)
    );
    let wire = manifest(&output);
    let report = wire.report.as_option().unwrap();
    assert!(report.files.is_empty());
    let passive: ReportObservation = serde_json::from_slice(&report.report_original).unwrap();
    assert_eq!(passive.outcome, Outcome::Incomplete);
    assert!(!passive.required_capture_complete);
    assert_eq!(passive.version, 3);
    let metadata = fs::read(output.join("manifest.pb")).unwrap();
    assert!(export_directory(&store, id.as_str(), None, &output).is_err());
    assert_eq!(fs::read(output.join("manifest.pb")).unwrap(), metadata);
    // A moved recipient directory remains self-contained after all original native files go.
    let moved = scratch.path().join("recipient");
    fs::rename(output, &moved).unwrap();
    fs::remove_dir_all(parent.parent().unwrap()).unwrap();
    let mut reader = open_directory(&moved, &binary_options()).unwrap();
    assert_eq!(reader.member_bytes(), receipt.member_bytes);
    let range = reader
        .read_member(1, 65 * 1024 * 1024 - 8, NonZeroU64::new(8).unwrap())
        .unwrap();
    assert_eq!(range.byte_range, 65 * 1024 * 1024 - 8..65 * 1024 * 1024);
    assert!(!range.atomic_snapshot);
    assert!(!range.whole_file);
    assert!(range.verified_sha256.is_some());
}

#[test]
fn directory_retained_descriptors_survive_replacement_unlink_and_refuse_inplace_mutation() {
    let scratch = tempfile::tempdir().unwrap();
    let (bundle, _) = exported(scratch.path(), false);
    let mut reader = open_directory(&bundle, &binary_options()).unwrap();
    let member = bundle.join("members/1");
    fs::rename(&member, bundle.join("saved")).unwrap();
    fs::write(&member, b"replacement").unwrap();
    fs::remove_file(bundle.join("saved")).unwrap();
    let range = reader
        .read_member(1, 0, NonZeroU64::new(4).unwrap())
        .unwrap();
    assert_eq!(range.byte_range, 0..4);
    assert_eq!(range.identity.bytes, 16);
    assert!(open_directory(&bundle, &binary_options()).is_err());
    // Acquired fd identity is independent from the directory lease.
    fs::remove_dir_all(&bundle).unwrap();
    assert!(
        reader
            .read_member(0, 0, NonZeroU64::new(4).unwrap())
            .is_ok()
    );

    let other = tempfile::tempdir().unwrap();
    let (bundle, _) = exported(other.path(), false);
    let mut reader = open_directory(&bundle, &binary_options()).unwrap();
    replace(&bundle.join("members/1"), &[1; 16]);
    assert!(
        reader
            .read_member(1, 0, NonZeroU64::new(4).unwrap())
            .is_err()
    );
}

#[test]
fn directory_missing_corrupt_extra_and_original_omission_never_grant_full_bytes() {
    for mode in 0..5 {
        let scratch = tempfile::tempdir().unwrap();
        let (bundle, _) = exported(scratch.path(), false);
        match mode {
            0 => fs::remove_file(bundle.join("members/1")).unwrap(),
            1 => replace(&bundle.join("members/1"), &[1; 16]),
            2 => fs::write(bundle.join("members/01"), [0; 16]).unwrap(),
            3 => {
                fs::remove_file(bundle.join("members/1")).unwrap();
                std::os::unix::fs::symlink("../../outside", bundle.join("members/1")).unwrap();
            }
            _ => {
                fs::remove_file(bundle.join("members/1")).unwrap();
                rustix::fs::mknodat(
                    rustix::fs::CWD,
                    bundle.join("members/1"),
                    rustix::fs::FileType::Fifo,
                    rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
                    0,
                )
                .unwrap();
            }
        }
        assert!(
            open_directory(&bundle, &binary_options()).is_err(),
            "mode {mode}"
        );
        let leaked = fs::read_dir("/proc/self/fd")
            .unwrap()
            .filter_map(Result::ok)
            .filter_map(|entry| fs::read_link(entry.path()).ok())
            .any(|path| path.starts_with(&bundle));
        assert!(!leaked, "mode {mode}");
    }
    let scratch = tempfile::tempdir().unwrap();
    let (bundle, parent) = exported(scratch.path(), true);
    assert!(parent.join("run-0.stdout").exists());
    let reader = open_directory(&bundle, &binary_options()).unwrap();
    assert_eq!(reader.readback().full_original_bytes_available, Some(false));
    assert_eq!(reader.readback().original_dependency_omissions, Some(1));
    assert!(!bundle.join("members/1").exists());
    fs::write(bundle.join("members/1"), [0; 16]).unwrap();
    assert!(open_directory(&bundle, &binary_options()).is_err());
}

#[test]
fn directory_manifest_requires_report_scope_current_commitment_and_exact_descriptor_world() {
    for mode in 0..5 {
        let scratch = tempfile::tempdir().unwrap();
        let (bundle, _) = exported(scratch.path(), false);
        let mut wire = manifest(&bundle);
        match mode {
            0 => {
                wire.report.as_option_mut().unwrap().scope =
                    encode_root::<_, pb::WorkflowEvidenceScope>(EvidenceScope::Full, "scope")
                        .unwrap()
                        .into()
            }
            1 => wire.report = original().into(),
            2 => wire.profile = Default::default(),
            3 => replace(
                &bundle.join("descriptor.bin"),
                crate::code_intelligence_contracts::DESCRIPTORS,
            ),
            _ => {
                let mut set = descriptors();
                set.file[0].package = Some("foreign.world".into());
                replace(&bundle.join("descriptor.bin"), &set.encode_to_vec());
            }
        }
        replace(&bundle.join("manifest.pb"), &wire.encode_to_vec());
        assert!(
            open_directory(&bundle, &binary_options()).is_err(),
            "mode {mode}"
        );
    }
    let scratch = tempfile::tempdir().unwrap();
    let (bundle, _) = exported(scratch.path(), false);
    let size = fs::metadata(bundle.join("manifest.pb")).unwrap().len() as usize;
    assert!(open_directory(&bundle, &binary_options().with_max_message_size(size - 1)).is_err());
    fs::set_permissions(
        bundle.join("manifest.pb"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    let file = fs::OpenOptions::new()
        .write(true)
        .open(bundle.join("manifest.pb"))
        .unwrap();
    file.set_len(MAX_INPUT_BYTES as u64 + 1).unwrap();
    assert!(
        matches!(open_directory(&bundle,&binary_options()),Err(HostError::DeveloperEvidence(error)) if matches!(*error,EvidenceError::InputLimit { .. }))
    );
}

#[test]
fn directory_digest_consistency_cannot_replace_original_input_role_validation() {
    let scratch = tempfile::tempdir().unwrap();
    let (bundle, _) = exported(scratch.path(), false);
    let input = fs::read(bundle.join("members/0")).unwrap();
    let mut input: serde_json::Value = serde_json::from_slice(&input).unwrap();
    input["invocation"] = serde_json::to_value(CliInvocationId::new()).unwrap();
    let input = serde_json::to_vec(&input).unwrap();
    replace(&bundle.join("members/0"), &input);
    let mut wire = manifest(&bundle);
    let report = wire.report.as_option_mut().unwrap();
    let mut json: serde_json::Value = serde_json::from_slice(&report.report_original).unwrap();
    json["evidence"][0]["content"] = serde_json::to_value(ContentIdentity::of(&input)).unwrap();
    revise(report, json);
    replace(&bundle.join("manifest.pb"), &wire.encode_to_vec());
    assert!(
        matches!(open_directory(&bundle,&binary_options()),Err(HostError::DeveloperEvidence(error)) if matches!(*error,EvidenceError::Workflow(_)))
    );
}

#[test]
fn directory_inventory_uses_original_commitment_and_population_checks() {
    let scratch = tempfile::tempdir().unwrap();
    let (bundle, _) = exported(scratch.path(), false);
    // Passive incomplete native inventory: no executable or population readiness is granted.
    let executable = serde_json::json!({"original":"unavailable","captured":"unavailable","sha256":"a".repeat(64),"bytes":0});
    let inventory = serde_json::json!({
        "version":2,"required_capture_complete":false,"capture":[],
        "selection":{"operation":"test","groups":[],"filters":{"words":[],"expressions":[]},"ignored":"default"},
        "host_generation":{"generation":"original","protocol":1,"manifest_digest":"b".repeat(64)},
        "nextest":executable,"correctness":executable,"nextest_version":"original incomplete capture",
        "nextest_configuration":[],"runner_profile":"original","groups":[],
        "setup":"NotRunByListing","runtime_inputs":"NotCapturedByDiscovery","physical_scope":"NotEstablishedByDiscovery"
    });
    let inventory = serde_json::to_vec(&inventory).unwrap();
    let content = ContentIdentity::of(&inventory);
    fs::write(bundle.join("members/2"), &inventory).unwrap();
    fs::set_permissions(bundle.join("members/2"), fs::Permissions::from_mode(0o400)).unwrap();
    let mut wire = manifest(&bundle);
    let report = wire.report.as_option_mut().unwrap();
    let mut json: serde_json::Value = serde_json::from_slice(&report.report_original).unwrap();
    json["inventory"] = serde_json::json!("inventory/inventory.json");
    json["inventory_content"] = serde_json::to_value(&content).unwrap();
    json["selected_tests"] = serde_json::json!(0);
    json["evidence"].as_array_mut().unwrap().push(serde_json::json!({"root":"workflow","path":"inventory/inventory.json","role":"inventory","group":null,"content":content,"omission":null,"failure":null}));
    revise(report, json.clone());
    replace(&bundle.join("manifest.pb"), &wire.encode_to_vec());
    let reader = open_directory(&bundle, &binary_options()).unwrap();
    assert_eq!(reader.readback().inventory_sha256, Some(content.sha256));
    assert_eq!(reader.readback().selected_tests, Some(0));
    json["selected_tests"] = serde_json::json!(1);
    revise(wire.report.as_option_mut().unwrap(), json);
    replace(&bundle.join("manifest.pb"), &wire.encode_to_vec());
    assert!(
        matches!(open_directory(&bundle,&binary_options()),Err(HostError::DeveloperEvidence(error)) if matches!(*error,EvidenceError::Workflow(_)))
    );
}
