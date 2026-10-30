// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{
    envelope::RedactionPolicy,
    extraction::{NextestOutcome, OutputStream},
    managed::streams::ChunkObserver,
};
use serde_json::json;
use std::{num::NonZeroUsize, os::unix::fs::PermissionsExt};

fn intent() -> (tempfile::TempDir, Intent) {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("src")).unwrap();
    fs::write(
        root.path().join("Cargo.toml"),
        "[package]\nname=\"case\"\nversion=\"0.0.0\"\nedition=\"2024\"\n",
    )
    .unwrap();
    fs::write(
        root.path().join("src/lib.rs"),
        "#[test] fn same_name() {}\n",
    )
    .unwrap();
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    fs::copy(
        checkout.join("rust-toolchain.toml"),
        root.path().join("rust-toolchain.toml"),
    )
    .unwrap();
    let path = root.path().to_str().unwrap();
    let metadata: cargo_metadata::Metadata = serde_json::from_value(json!({"packages":[{"name":"case","version":"0.0.0","id":"case-id","manifest_path":format!("{path}/Cargo.toml"),"dependencies":[],"targets":[{"name":"case","kind":["lib"],"crate_types":["lib"],"src_path":format!("{path}/src/lib.rs"),"edition":"2024","required-features":[]}],"features":{"default":[]}}],"workspace_members":["case-id"],"workspace_default_members":["case-id"],"workspace_root":path,"target_directory":format!("{path}/target"),"version":1})).unwrap();
    let request = Request {
        operation: selection::Operation::Test,
        scope: selection::Scope::Packages(vec![selection::PackageIntent {
            package: "case".into(),
            target: Some(selection::TargetIntent {
                name: "case".into(),
                kind: Some("lib".into()),
            }),
            features: selection::Features::default(),
        }]),
        platform: None,
        filters: selection::Filters {
            words: vec!["same_name".into()],
            expressions: vec!["test(same_name)".into()],
        },
        ignored: selection::IgnoredPolicy::Default,
    };
    let intent = Intent::new(
        metadata,
        request,
        "selected-profile".into(),
        std::env::vars_os().collect(),
    )
    .unwrap();
    (root, intent)
}
fn listed(intent: Intent) -> Listed {
    let resolved = intent
        .plan
        .clone()
        .resolve_with(|arguments| {
            Ok(correctness::Resolution {
                arguments: arguments.to_vec(),
                packages: vec!["case".into()],
                native_capabilities: BTreeSet::new(),
                units: vec![correctness::CompilerUnit {
                    package_id: "case-id".into(),
                    package_name: "case".into(),
                    target_name: "case".into(),
                    target_kinds: vec!["lib".into()],
                    source_path: intent
                        .context
                        .checkout
                        .join("src/lib.rs")
                        .to_str()
                        .unwrap()
                        .into(),
                    features: BTreeSet::new(),
                    platform: None,
                    mode: "test".into(),
                    root: true,
                }],
            })
        })
        .unwrap();
    let capture_root = intent.context.checkout.join("captured");
    fs::create_dir(&capture_root).unwrap();
    fs::create_dir(capture_root.join("group-0")).unwrap();
    let captured = capture_root.join("captured-test");
    fs::write(&captured, b"#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&captured, fs::Permissions::from_mode(0o500)).unwrap();
    let executable = discovery::Executable {
        original: intent.context.checkout.join("target/debug/case"),
        captured: captured.clone(),
        bytes: fs::metadata(&captured).unwrap().len(),
        sha256: Sha256::digest(fs::read(&captured).unwrap())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    };
    let metadata = json!({"target-directory":intent.context.checkout.join("target"),"base-output-directories":["debug"],"non-test-binaries":{},"linked-paths":[]});
    let binary = json!({"binary-id":"case","binary-name":"case","package-id":"case-id","kind":"lib","binary-path":captured,"build-platform":"target"});
    let binaries: nextest_metadata::BinaryListSummary =
        serde_json::from_value(json!({"rust-build-meta":metadata,"rust-binaries":{"case":binary}}))
            .unwrap();
    let mut suite = binary;
    suite["package-name"] = json!("case");
    suite["cwd"] = json!(intent.context.checkout);
    suite["status"] = json!("listed");
    suite["testcases"] =
        json!({"same_name":{"kind":"test","ignored":false,"filter-match":{"status":"matches"}}});
    let listing: nextest_metadata::TestListSummary = serde_json::from_value(
        json!({"rust-build-meta":metadata,"test-count":1,"rust-suites":{"case":suite}}),
    )
    .unwrap();
    let artifact: cargo_metadata::Artifact = serde_json::from_value(json!({"package_id":"case-id","manifest_path":intent.context.checkout.join("Cargo.toml"),"target":{"name":"case","kind":["lib"],"crate_types":["lib"],"src_path":intent.context.checkout.join("src/lib.rs"),"edition":"2024","doc":true,"doctest":false,"test":true},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":true},"features":[],"filenames":[captured],"executable":captured,"fresh":false})).unwrap();
    let configuration = b"[profile.selected-profile]\n".to_vec();
    fs::write(capture_root.join("nextest.toml"), &configuration).unwrap();
    fs::write(
        capture_root.join("group-0/binaries.json"),
        serde_json::to_vec(&binaries).unwrap(),
    )
    .unwrap();
    fs::write(
        capture_root.join("group-0/cargo-metadata.json"),
        serde_json::to_vec(&intent.context.metadata).unwrap(),
    )
    .unwrap();
    let controls = [
        "nextest.toml",
        "group-0/binaries.json",
        "group-0/cargo-metadata.json",
    ]
    .iter()
    .map(|name| {
        let path = capture_root.join(name);
        Input {
            digest: digest(&path).unwrap(),
            path,
        }
    })
    .collect();
    let inventory = Inventory {
        version: 1,
        selection: resolved.plan().clone(),
        host_generation: crate::generation::PreparedGeneration {
            generation: "fixture".into(),
            protocol: 1,
            manifest_digest: "a".repeat(64),
        },
        nextest: executable.clone(),
        correctness: executable.clone(),
        nextest_version: "cargo-nextest 0.9.146 (fixture)".into(),
        nextest_configuration: configuration,
        runner_profile: "selected-profile".into(),
        groups: vec![discovery::ListedGroup {
            build: discovery::BuildCapture {
                version: 1,
                resolution: resolved.admitted()[0].clone(),
                artifacts: vec![discovery::BuiltArtifact {
                    cargo: artifact,
                    executable,
                }],
                cargo_exit: 0,
                incomplete: Default::default(),
            },
            binaries,
            listing,
        }],
        setup: discovery::SetupObservation::NotRunByListing,
        runtime_inputs: discovery::RuntimeInputObservation::NotCapturedByDiscovery,
        physical_scope: discovery::PhysicalScopeObservation::NotEstablishedByDiscovery,
        generation_use: None,
    };
    Listed {
        context: intent.context,
        resolved,
        requirements: vec![],
        inventory,
        capture_root,
        controls,
    }
}
fn wire() -> Vec<u8> {
    let records = [
        json!({"type":"suite","event":"started","test_count":1,"nextest":{"crate":"case","test_binary":"case","kind":"lib"}}),
        json!({"type":"test","event":"started","name":"case::case$same_name"}),
        json!({"type":"test","event":"ok","name":"case::case$same_name","exec_time":0.1}),
        json!({"type":"suite","event":"ok","passed":1,"failed":0,"ignored":0,"measured":0,"filtered_out":0,"exec_time":0.1,"nextest":{"crate":"case","test_binary":"case","kind":"lib"}}),
    ];
    records
        .iter()
        .flat_map(|record| {
            let mut bytes = serde_json::to_vec(record).unwrap();
            bytes.push(b'\n');
            bytes
        })
        .collect()
}
#[test]
fn explain_is_stage_specific_and_changed_declarations_refuse_before_effects() {
    let (root, intent) = intent();
    assert!(matches!(
        intent.explain().admission,
        Information::Unavailable { .. }
    ));
    assert!(!root.path().join("target").exists());
    fs::write(root.path().join("Cargo.toml"), "changed declaration").unwrap();
    assert!(matches!(intent.admit(), Err(WorkflowError::Changed(_))));
    assert!(!root.path().join("target").exists());
}
#[test]
fn actual_admission_uses_selected_checkout_without_changing_process_context() {
    let (root, _) = intent();
    let before = std::env::current_dir().unwrap();
    let prepared = std::process::Command::new(correctness::real_cargo().unwrap())
        .current_dir(root.path())
        .args(["generate-lockfile", "--offline"])
        .output()
        .unwrap();
    assert!(
        prepared.status.success(),
        "{}",
        String::from_utf8_lossy(&prepared.stderr)
    );
    let metadata = cargo_metadata::MetadataCommand::new()
        .current_dir(root.path())
        .no_deps()
        .exec()
        .unwrap();
    let request = Request {
        operation: selection::Operation::Test,
        scope: selection::Scope::Packages(vec![selection::PackageIntent {
            package: "case".into(),
            target: Some(selection::TargetIntent {
                name: "case".into(),
                kind: Some("lib".into()),
            }),
            features: selection::Features::default(),
        }]),
        platform: None,
        filters: selection::Filters::default(),
        ignored: selection::IgnoredPolicy::Default,
    };
    let mut unavailable_environment: Environment = std::env::vars_os().collect();
    unavailable_environment.insert(
        "PSE_CORRECTNESS_REAL_CARGO".into(),
        root.path()
            .join("unavailable-selected-cargo")
            .into_os_string(),
    );
    let unavailable = Intent::new(
        metadata.clone(),
        request.clone(),
        "default".into(),
        unavailable_environment,
    )
    .unwrap()
    .admit();
    assert!(
        matches!(
            unavailable,
            Err(WorkflowError::Correctness(CorrectnessError::Io(_)))
        ),
        "selected Cargo cannot silently fall back to the process environment"
    );
    let admitted = Intent::new(
        metadata,
        request,
        "default".into(),
        std::env::vars_os().collect(),
    )
    .unwrap()
    .admit()
    .unwrap();
    assert_eq!(std::env::current_dir().unwrap(), before);
    assert_eq!(admitted.resolved().plan().groups().len(), 1);
    assert!(admitted.resolved().native_capabilities().is_empty());
    assert!(
        admitted.resolved().admitted()[0]
            .units
            .iter()
            .all(|unit| unit.package_name != "arrow" && !unit.package_name.starts_with("arrow-"))
    );
    assert!(
        admitted.resolved().admitted()[0]
            .arguments
            .iter()
            .all(|argument| !argument.to_string_lossy().contains("force_validate"))
    );
    assert!(
        admitted.resolved().admitted()[0]
            .units
            .iter()
            .filter(|unit| unit.root)
            .all(|unit| Path::new(&unit.source_path).starts_with(root.path()))
    );
    assert!(matches!(
        admitted.explain().selected_tests,
        Information::Unavailable { .. }
    ));
    assert!(
        !root.path().join("target/debug/deps").exists(),
        "admission never compiles the selected target"
    );
}
#[test]
fn listed_commands_preserve_context_profile_policy_and_refuse_changed_capture() {
    let (_root, intent) = intent();
    let listed = listed(intent);
    let command = listed.command(0).unwrap();
    let args: Vec<_> = command
        .arguments
        .iter()
        .map(|arg| arg.to_str().unwrap())
        .collect();
    assert!(
        args.windows(2)
            .any(|pair| pair == ["--profile", "selected-profile"])
    );
    assert!(
        args.windows(2)
            .any(|pair| pair == ["--run-ignored", "default"])
    );
    assert!(
        args.windows(2)
            .any(|pair| pair == ["-E", "test(same_name)"])
    );
    assert_eq!(args.last(), Some(&"same_name"));
    assert!(
        !args.contains(&"-p"),
        "reused native metadata owns exact build scope"
    );
    assert_eq!(command.cwd, listed.context.checkout);
    assert_eq!(
        command.environment[std::ffi::OsStr::new("NEXTEST_EXPERIMENTAL_LIBTEST_JSON")],
        "1"
    );
    let debug = format!("{command:?}");
    assert!(!debug.contains("same_name"));
    assert!(!debug.contains("selected-profile"));
    assert!(matches!(
        command.runtime_inputs,
        discovery::RuntimeInputObservation::NotCapturedByDiscovery
    ));
    let page = listed.page(0, 1).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records[0].identity.binary_id, "case");
    assert_eq!(page.records[0].target, "case");
    assert_eq!(page.continuation, None);
    assert_eq!(page.source_basis, "immutable_captured_binary_inventory");
    fs::write(
        listed.context.checkout.join("src/lib.rs"),
        "new source is not the captured binary",
    )
    .unwrap();
    assert!(
        listed.command(0).is_ok(),
        "explicit immutable reuse does not claim fresh source compilation"
    );
    fs::write(
        listed.capture_root.join("group-0/cargo-metadata.json"),
        b"{}",
    )
    .unwrap();
    assert!(matches!(listed.command(0), Err(WorkflowError::Changed(_))));
}
#[test]
fn required_observer_handles_chunk_boundaries_and_never_promotes_lost_or_truncated_evidence() {
    let (_root, intent) = intent();
    let listed = listed(intent);
    let bytes = wire();
    let mut observer = listed
        .observer(
            0,
            NonZeroUsize::new(4096).unwrap(),
            RedactionPolicy::default(),
        )
        .unwrap();
    let mut offset = 0;
    for chunk in bytes.chunks(7) {
        observer
            .observe(OutputStream::Stdout, offset, chunk)
            .unwrap();
        offset += chunk.len() as u64;
    }
    observer
        .observe(OutputStream::Stderr, 0, b"\xff arbitrary diagnostics")
        .unwrap();
    let (observed, error) = observer.finish(0, true);
    assert!(error.is_none());
    assert_eq!(observed.summary.outcome(), NextestOutcome::Succeeded);
    for (data, retained) in [(&bytes[..bytes.len() - 1], true), (bytes.as_slice(), false)] {
        let mut observer = listed
            .observer(
                0,
                NonZeroUsize::new(4096).unwrap(),
                RedactionPolicy::default(),
            )
            .unwrap();
        observer.observe(OutputStream::Stdout, 0, data).unwrap();
        let (observed, _) = observer.finish(0, retained);
        assert_eq!(observed.summary.outcome(), NextestOutcome::Incomplete);
    }
}
#[test]
fn oversized_or_malformed_records_do_not_stop_later_required_observation() {
    let (_root, intent) = intent();
    let listed = listed(intent);
    let mut observer = listed
        .observer(
            0,
            NonZeroUsize::new(300).unwrap(),
            RedactionPolicy::default(),
        )
        .unwrap();
    let mut bytes = vec![b'x'; 500];
    bytes.push(b'\n');
    bytes.extend(wire());
    assert!(observer.observe(OutputStream::Stdout, 0, &bytes).is_err());
    let (observed, _) = observer.finish(0, true);
    assert_eq!(observed.summary.completed(), 1);
    assert_eq!(observed.summary.outcome(), NextestOutcome::Incomplete);
}
#[test]
fn failure_addresses_retain_exact_raw_offsets_after_an_independent_protocol_failure() {
    let (_root, intent) = intent();
    let listed = listed(intent);
    let mut bytes = b"not JSON\n".to_vec();
    for line in wire().split_inclusive(|byte| *byte == b'\n') {
        let mut record: serde_json::Value = serde_json::from_slice(line).unwrap();
        if record["event"] == "ok" {
            record["event"] = json!("failed");
            if record["type"] == "test" {
                record["stdout"] = json!("panic at src/lib.rs:7:9\noriginal detail");
            } else {
                record["passed"] = json!(0);
                record["failed"] = json!(1);
            }
        }
        bytes.extend(serde_json::to_vec(&record).unwrap());
        bytes.push(b'\n');
    }
    let mut observer = listed
        .observer(
            0,
            NonZeroUsize::new(4096).unwrap(),
            RedactionPolicy::default(),
        )
        .unwrap();
    let failure = observer
        .observe(OutputStream::Stdout, 0, &bytes)
        .unwrap_err();
    assert_eq!(failure.origin.byte_offset, 0);
    assert!(matches!(failure.error, ExtractionError::Json(_)));
    let (observed, _) = observer.finish(100, true);
    assert_eq!(observed.summary.outcome(), NextestOutcome::TestFailed);
    assert!(!observed.summary.complete());
    assert_eq!(observed.failures.len(), 1);
    let original = &observed.failures[0];
    assert_eq!(original.identity.test_name, "same_name");
    assert_eq!(original.origin.ordinal, 3);
    let start = original.origin.byte_offset as usize;
    let end = start + original.origin.byte_length as usize;
    let raw: serde_json::Value = serde_json::from_slice(&bytes[start..end]).unwrap();
    assert_eq!(raw["event"], "failed");
    assert!(raw["stdout"].as_str().unwrap().contains("src/lib.rs:7:9"));
}
