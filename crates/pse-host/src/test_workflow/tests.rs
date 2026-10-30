// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use serde_json::json;
use std::os::unix::fs::PermissionsExt;

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
            ignore_default_filter: true,
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

#[test]
fn inventory_ready_requires_each_original_phase_and_refuses_unsupported_or_missing_capture() {
    let (_root, intent) = intent();
    let mut listed = listed(intent);
    assert!(listed.inventory.capture_complete());
    let complete = serde_json::to_value(&listed.inventory).unwrap();
    let mut old = complete.clone();
    old["version"] = 1.into();
    let old: Inventory = serde_json::from_value(old).unwrap();
    assert!(old.verify_executables().is_err());
    let mut missing = complete;
    missing
        .as_object_mut()
        .unwrap()
        .remove("required_capture_complete");
    assert!(serde_json::from_value::<Inventory>(missing).is_err());
    listed.inventory.capture[1].stdout.complete = false;
    assert!(!listed.inventory.capture_complete());
    assert!(listed.observer(0).is_err());
    listed.inventory.capture[1].stdout.complete = true;
    listed.inventory.capture.pop();
    assert!(!listed.inventory.capture_complete());
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
        version: 2,
        required_capture_complete: true,
        capture: [("version", None), ("build", Some(0)), ("list", Some(0))]
            .into_iter()
            .map(|(stage, group)| {
                let stream = crate::managed::result::StreamSummary {
                    bytes_read: 0,
                    bytes_retained: 0,
                    eof: true,
                    complete: true,
                    io_unresolved: false,
                };
                discovery::PhaseCapture {
                    stage: stage.into(),
                    group,
                    child_exit: 0,
                    stdout: stream.clone(),
                    stderr: stream,
                }
            })
            .collect(),
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
    assert!(args.contains(&"--ignore-default-filter"));
    assert!(
        !args.contains(&"--cargo-message-format"),
        "native reused binaries reject compiler build options"
    );
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
    assert!(
        !command
            .environment
            .contains_key(std::ffi::OsStr::new("NEXTEST_EXPERIMENTAL_LIBTEST_JSON"))
    );
    assert!(!args.contains(&"--message-format"));
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
fn run_configuration_is_private_and_preserves_original_profile_policy() {
    let (_root, intent) = intent();
    let mut listed = listed(intent);
    let original = b"[profile.default]\nfail-fast = false\n[profile.selected-profile]\ninherits = 'default'\nretries = 2\n[profile.selected-profile.junit]\npath = 'shared.xml'\nstore-failure-output = true\n";
    listed.inventory.nextest_configuration = original.to_vec();
    // Configuration derivation is a pure semantic projection of already captured bytes;
    // original native profile validation/listing belongs to the discovery owner.
    let first = listed.run_configuration(0).unwrap();
    let mut actual: toml::Value = toml::from_str(&fs::read_to_string(&first).unwrap()).unwrap();
    let expected: toml::Value = toml::from_str(std::str::from_utf8(original).unwrap()).unwrap();
    assert_eq!(
        actual["profile"]["selected-profile"]["junit"]["path"].as_str(),
        listed.report_path(0).to_str()
    );
    actual["profile"]["selected-profile"]["junit"]["path"] =
        expected["profile"]["selected-profile"]["junit"]["path"].clone();
    assert_eq!(actual, expected);
    assert_eq!(listed.inventory.nextest_configuration, original);
    fs::create_dir(listed.capture_root.join("group-1")).unwrap();
    let second = listed.run_configuration(1).unwrap();
    assert_ne!(first, second);
    let second: toml::Value = toml::from_str(&fs::read_to_string(second).unwrap()).unwrap();
    assert_ne!(
        second["profile"]["selected-profile"]["junit"]["path"],
        toml::Value::String(listed.report_path(0).to_str().unwrap().into())
    );
    listed.inventory.runner_profile = "missing".into();
    assert!(
        listed.run_configuration(0).is_err(),
        "projection cannot create an otherwise absent native profile"
    );
}
