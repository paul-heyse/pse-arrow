// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public passive comparison of frozen workflow admissions; no planning or compilation.
use pse_host::{
    compiler_investigation,
    correctness::{CompilerUnit, Resolution},
    envelope::{CliInvocationId, CommandOutcome, Envelope, Settlement, SubjectState},
    generation::PreparedGeneration,
    invocation::{InvocationSpec, InvocationStore},
    managed::result::{ManagedSummary, WorkflowSummary},
    paths::CheckoutState,
    test_workflow::execute::{Action, Outcome, ReportReference, TerminalRecord},
};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    os::unix::fs::{DirBuilderExt as _, PermissionsExt as _},
    path::{Path, PathBuf},
    process::{Command, Output},
};

struct Fixture {
    scratch: tempfile::TempDir,
    root: PathBuf,
    state: CheckoutState,
    environment: BTreeMap<OsString, OsString>,
    id: CliInvocationId,
    report: PathBuf,
    metadata: PathBuf,
}
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
#[cfg(test)]
impl Fixture {
    fn new(command: &str, admitted: bool) -> Self {
        let scratch = tempfile::tempdir().unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let mut environment = std::env::vars_os().collect::<BTreeMap<_, _>>();
        environment.insert(
            "XDG_STATE_HOME".into(),
            scratch.path().join("state").into_os_string(),
        );
        environment.insert("PSE_FORMAT".into(), "json".into());
        let state = CheckoutState::resolve(&root, &environment).unwrap();
        let store = InvocationStore::new(state.clone());
        let id = CliInvocationId::new();
        let directory = store
            .begin(InvocationSpec {
                invocation_id: id.clone(),
                command: command.into(),
                session: None,
                parent: None,
                operation: (),
                control: PreparedGeneration {
                    generation: "retained-data-fixture".into(),
                    protocol: 1,
                    manifest_digest: "a".repeat(64),
                },
                environment_digest: "b".repeat(64),
                scientific: Vec::new(),
            })
            .unwrap();
        let source = scratch.path().join("original/src/lib.rs");
        let package = "path+file:///original#case@0.0.0";
        let unit = CompilerUnit {
            package_id: package.into(),
            package_name: "case".into(),
            target_name: "case".into(),
            target_kinds: vec!["lib".into()],
            source_path: source.to_str().unwrap().into(),
            features: BTreeSet::new(),
            platform: None,
            mode: "check".into(),
            root: true,
        };
        let group = Resolution {
            arguments: vec!["check".into(), "--lib".into()],
            packages: vec!["case".into()],
            units: vec![unit],
            native_capabilities: BTreeSet::new(),
        };
        let groups = if admitted {
            vec![group.clone(), group]
        } else {
            Vec::new()
        };
        let bytes=serde_json::to_vec(&json!({"version":1,"invocation":id,"action":"check","outcome":"check_succeeded","selection":if admitted {json!({"recorded":true})}else{Value::Null},"admission":groups,"phases":[],"observations":[],"failures":[],"required_capture_complete":true})).unwrap();
        let parent = directory.path().join("workflow");
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&parent)
            .unwrap();
        let report = parent.join("report.json");
        std::fs::write(&report, &bytes).unwrap();
        std::fs::set_permissions(&report, std::fs::Permissions::from_mode(0o400)).unwrap();
        let terminal = TerminalRecord {
            protocol: "pse.test-workflow".into(),
            version: 1,
            invocation: id.clone(),
            action: Action::Check,
            outcome: Outcome::CheckSucceeded,
            report: ReportReference {
                path: "report.json".into(),
                sha256: digest(&bytes),
                byte_length: bytes.len() as u64,
            },
            required_capture_complete: true,
        };
        // Frozen fixture data exercises retained-byte admission; it constructs no native verdict capability.
        let mut result = Envelope::response(
            command,
            "original frozen fixture",
            ManagedSummary {
                bundle: None,
                native_lifetime_pending: Some(false),
                workflow: Some(WorkflowSummary {
                    terminal,
                    report: report.clone().into_os_string(),
                }),
                child_exit: Some(0),
                stdout: None,
                stderr: None,
                borrowed_reservation: false,
                service_result: None,
            },
        );
        result.invocation_id = id.clone();
        result.outcome = CommandOutcome::Success;
        result.subject_state = SubjectState::Succeeded;
        result.settlement = Settlement::Complete;
        let result_path = directory.path().join("result.json");
        std::fs::write(&result_path, serde_json::to_vec(&result).unwrap()).unwrap();
        std::fs::set_permissions(result_path, std::fs::Permissions::from_mode(0o400)).unwrap();
        let metadata = scratch.path().join("metadata.json");
        std::fs::write(&metadata,serde_json::to_vec(&json!({"packages":[{"name":"case","version":"0.0.0","id":package,"source":null,"dependencies":[],"targets":[{"name":"case","kind":["lib"],"crate_types":["lib"],"src_path":source,"edition":"2024","test":true,"doc":true,"doctest":true}],"features":{},"manifest_path":scratch.path().join("original/Cargo.toml")}],"workspace_members":[package],"workspace_default_members":[package],"resolve":{"nodes":[{"id":package,"dependencies":[],"deps":[],"features":[]}],"root":package},"target_directory":scratch.path().join("original/target"),"workspace_root":scratch.path().join("original"),"metadata":null,"version":1})).unwrap()).unwrap();
        Self {
            scratch,
            root,
            state,
            environment,
            id,
            report,
            metadata,
        }
    }
    fn cli(&self, args: &[&str]) -> Output {
        Command::new(
            std::env::var_os("CARGO_BIN_EXE_pse")
                .expect("native test runner supplies current public executable"),
        )
        .arg("-C")
        .arg(&self.root)
        .args(args)
        .env_clear()
        .envs(&self.environment)
        .current_dir(&self.root)
        .output()
        .unwrap()
    }
    fn publication(&self) -> Value {
        let output = self.cli(&[
            "inspect",
            "compiler-artifacts",
            self.id.as_str(),
            "--metadata",
            self.metadata.to_str().unwrap(),
        ]);
        assert!(output.status.success(), "{output:?}");
        serde_json::from_slice(&output.stdout).unwrap()
    }
}

#[test]
fn retained_workflow_unit_context_roundtrip_preserves_groups_and_exact_digest_guards() {
    let fixture = Fixture::new("check", true);
    let original =
        compiler_investigation::unit_context(&fixture.root, &fixture.state, fixture.id.as_str())
            .unwrap();
    assert!(original.payload.admission_available);
    assert_eq!(original.payload.admission.len(), 2);
    assert!(matches!(
        original.payload.origin,
        compiler_investigation::UnitContextOrigin::WorkflowReport {
            action: Action::Check,
            ..
        }
    ));
    assert!(
        compiler_investigation::artifact::observe(
            &fixture.root,
            &fixture.state,
            fixture.id.as_str()
        )
        .is_err(),
        "the artifact-purpose reader must stay strict"
    );
    let publication = fixture.publication();
    let address = publication["payload"]["reference"]["address"]
        .as_str()
        .unwrap();
    let limited = fixture.cli(&[
        "inspect",
        "compiler-artifacts",
        fixture.id.as_str(),
        "--metadata",
        fixture.metadata.to_str().unwrap(),
        "--fields",
        "reference,metadata",
    ]);
    assert!(limited.status.success(), "{limited:?}");
    let limited: Value = serde_json::from_slice(&limited.stdout).unwrap();
    assert_eq!(limited["payload"]["reference"]["address"], address);
    assert_eq!(limited["observation"]["reference"], address);
    let expanded = fixture.cli(&["inspect", "--result", address]);
    assert!(expanded.status.success(), "{expanded:?}");
    let expanded: Value = serde_json::from_slice(&expanded.stdout).unwrap();
    let rows = expanded["payload"]["unit_records"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["group"], 0);
    assert_eq!(rows[1]["group"], 1);
    assert_eq!(rows[1]["group_unit"], 0);
    assert_eq!(expanded["payload"]["admission_groups"][1]["unit_start"], 1);
    let bounded = fixture.cli(&[
        "inspect",
        "--result",
        address,
        "--fields",
        "unit_records,admission_groups",
        "--budget",
        "1",
    ]);
    assert_eq!(bounded.status.code(), Some(1), "{bounded:?}");
    let bounded: Value = serde_json::from_slice(&bounded.stdout).unwrap();
    assert_eq!(bounded["observation"]["completeness"], "partial");
    assert_eq!(bounded["observation"]["reference"], address);
    assert_eq!(
        bounded["payload"]["unit_records"].as_array().unwrap().len(),
        1
    );
    assert_eq!(bounded["view"]["omitted_records"], 1);
    assert_eq!(bounded["view"]["truncated"], true);
    let compared = fixture.cli(&[
        "inspect",
        "comparison",
        "--view",
        "context",
        "--before-result",
        address,
        "--after-result",
        address,
        "--before-checkout",
        fixture.root.to_str().unwrap(),
        "--after-checkout",
        fixture.root.to_str().unwrap(),
    ]);
    assert_eq!(
        compared.status.code(),
        Some(1),
        "duplicate admitted units must retain ambiguous alignment: {compared:?}"
    );
    let compared: Value = serde_json::from_slice(&compared.stdout).unwrap();
    assert_eq!(compared["observation"]["completeness"], "partial");
    assert!(compared["failures"].as_array().unwrap().is_empty());
    assert!(
        compared["payload"]["comparison"]["units"]
            .as_array()
            .unwrap()
            .iter()
            .any(|pair| pair["correspondence"] == "ambiguous")
    );
    assert_eq!(
        compared["payload"]["comparison"]["before"]["value"]["original_context"]["origin"]["kind"],
        "workflow_report"
    );
    let mut forged = publication["payload"]["reference"].clone();
    forged.as_object_mut().unwrap().remove("address");
    forged["locator"]["compiler_context_sha256"] = json!("c".repeat(64));
    let forged = serde_json::to_string(&forged).unwrap();
    let refused = fixture.cli(&["inspect", "--result", &forged]);
    assert!(!refused.status.success());
    assert!(
        String::from_utf8_lossy(&refused.stderr)
            .contains("original compiler context no longer matches"),
        "{refused:?}"
    );
    std::fs::set_permissions(&fixture.report, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::write(&fixture.report, b"changed frozen report").unwrap();
    std::fs::set_permissions(&fixture.report, std::fs::Permissions::from_mode(0o400)).unwrap();
    assert!(
        compiler_investigation::unit_context(&fixture.root, &fixture.state, fixture.id.as_str())
            .is_err()
    );
    assert!(
        !fixture
            .cli(&["inspect", "--result", address])
            .status
            .success()
    );
    assert!(fixture.scratch.path().is_dir());
}

#[test]
fn retained_context_refuses_unsupported_operations_and_preserves_unavailable_admission() {
    let unsupported = Fixture::new("run", true);
    assert!(
        compiler_investigation::unit_context(
            &unsupported.root,
            &unsupported.state,
            unsupported.id.as_str()
        )
        .is_err()
    );
    assert!(
        !unsupported
            .cli(&[
                "inspect",
                "compiler-artifacts",
                unsupported.id.as_str(),
                "--metadata",
                unsupported.metadata.to_str().unwrap()
            ])
            .status
            .success()
    );
    let missing = Fixture::new("check", false);
    let original =
        compiler_investigation::unit_context(&missing.root, &missing.state, missing.id.as_str())
            .unwrap();
    assert!(!original.payload.admission_available);
    assert!(
        !missing
            .cli(&[
                "inspect",
                "compiler-artifacts",
                missing.id.as_str(),
                "--metadata",
                missing.metadata.to_str().unwrap()
            ])
            .status
            .success()
    );
}
