// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use serde_json::json;
use std::os::unix::fs::PermissionsExt;

pub(super) fn intent() -> (tempfile::TempDir, Intent) {
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
    fs::create_dir(root.path().join(".config")).unwrap();
    fs::write(
        root.path().join(".config/nextest.toml"),
        "[profile.default]\ndefault-filter = 'none()'\n[profile.selected-profile]\ndefault-filter = 'all()'\n",
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
        non_incremental: false,
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

#[test]
fn fixture_retirement_uses_the_original_selected_population() {
    let (_root, intent) = intent();
    let listed = listed(intent);
    let group = &listed.inventory.groups[0];
    let mut candidate = json!({
        "group": 0,
        "unit": group.build.resolution.units[0],
        "declaration": {
            "id": "physical-owner",
            "target": "case",
            "kind": "lib",
            "scope": {"tests": ["different_test"]},
            "ownership": "target-owned",
            "inputs": ["server"]
        }
    });
    fs::remove_file(&group.build.artifacts[0].executable.captured).unwrap();
    let unselected = serde_json::from_value(candidate.clone()).unwrap();
    assert!(
        selected_target_owned_groups(&listed.inventory, &[unselected])
            .unwrap()
            .is_empty()
    );

    candidate["declaration"]["scope"]["tests"] = json!(["same_name"]);
    let selected = serde_json::from_value(candidate.clone()).unwrap();
    assert_eq!(
        selected_target_owned_groups(&listed.inventory, &[selected]).unwrap(),
        [0].into()
    );

    candidate["declaration"]["ownership"] = json!("runner-borrowed");
    let borrowed = serde_json::from_value(candidate).unwrap();
    assert!(
        selected_target_owned_groups(&listed.inventory, &[borrowed])
            .unwrap()
            .is_empty()
    );
}
#[test]
fn metadata_traversal_authenticates_unique_executables_once_and_refuses_changed_captures() {
    let (_root, intent) = intent();
    let mut listed = listed(intent);
    let group = &mut listed.inventory.groups[0];
    let suite = group.listing.rust_suites.values_mut().next().unwrap();
    let case = suite.test_cases.values().next().unwrap().clone();
    suite.test_cases = (0..44)
        .map(|index| {
            (
                nextest_metadata::TestCaseName::new(&format!("case_{index}")),
                case.clone(),
            )
        })
        .collect();
    let mut second = suite.clone();
    let second_path = listed.capture_root.join("captured-second");
    fs::copy(&group.build.artifacts[0].executable.captured, &second_path).unwrap();
    second.binary.binary_id = nextest_metadata::RustBinaryId::new("second");
    second.binary.binary_path = second_path.to_str().unwrap().into();
    let mut second_artifact = group.build.artifacts[0].clone();
    second_artifact.executable.captured = second_path.clone();
    group.build.artifacts.push(second_artifact);
    group
        .binaries
        .rust_binaries
        .insert(second.binary.binary_id.clone(), second.binary.clone());
    group
        .listing
        .rust_suites
        .insert(second.binary.binary_id.clone(), second);
    group.listing.test_count = 88;
    let identities = listed.inventory.selected_tests();
    assert_eq!(identities.len(), 88);
    let mut verified = listed.inventory.verified_tests();
    for _ in 0..3 {
        for identity in &identities {
            verified.test(identity).unwrap();
        }
    }
    assert_eq!(
        verified.authentications(),
        2,
        "264 metadata lookups authenticate two captured images"
    );
    let first = &listed.inventory.groups[0].build.artifacts[0]
        .executable
        .captured;
    let mut tampered = fs::read(first).unwrap();
    tampered[0] ^= 1;
    fs::set_permissions(first, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(first, tampered).unwrap();
    assert!(verified.test(&identities[0]).is_err());
    fs::remove_file(second_path).unwrap();
    assert!(verified.test(identities.last().unwrap()).is_err());
    assert_eq!(verified.authentications(), 2);
    assert!(listed.inventory.test(&identities[0]).is_err());
}
#[test]
fn ordinary_selection_skips_rerun_authentication_but_launch_still_refuses_missing_capture() {
    let (_root, intent) = intent();
    let listed = listed(intent);
    let identity = listed.inventory.selected_tests()[0].clone();
    fs::remove_file(
        &listed.inventory.groups[0].build.artifacts[0]
            .executable
            .captured,
    )
    .unwrap();
    assert!(listed.inventory.test(&identity).is_err());
    source::verify_rerun(&listed.inventory).unwrap();
    assert!(listed.command(0).is_err());
}

#[test]
fn launch_verifies_its_group_helpers_and_pre_filter_images_without_opening_another_group() {
    let (_root, intent) = intent();
    let mut listed = listed(intent);
    let mut unrelated: discovery::ListedGroup =
        serde_json::from_value(serde_json::to_value(&listed.inventory.groups[0]).unwrap()).unwrap();
    unrelated.build.artifacts[0].executable.captured =
        listed.capture_root.join("missing-other-image");
    listed.inventory.groups.push(unrelated);
    let capture = &mut listed.inventory.capture;
    let first_group_len = capture.len();
    capture.extend_from_within(1..);
    for phase in &mut capture[first_group_len..] {
        phase.group = Some(1);
    }
    let mut artifacts = Vec::new();
    for name in ["helper", "before-name-filter"] {
        let mut artifact = listed.inventory.groups[0].build.artifacts[0].clone();
        artifact.executable.captured = listed.capture_root.join(name);
        fs::copy(
            &listed.inventory.nextest.captured,
            &artifact.executable.captured,
        )
        .unwrap();
        artifacts.push(artifact);
    }
    let group = &mut listed.inventory.groups[0];
    let helper = artifacts[0].executable.captured.clone();
    let before_filter = artifacts[1].executable.captured.clone();
    let target_directory = &group.binaries.rust_build_meta.target_directory;
    let relative = pathdiff::diff_paths(&helper, target_directory).unwrap();
    let mut binaries = serde_json::to_value(&group.binaries).unwrap();
    binaries["rust-build-meta"]["non-test-binaries"] =
        json!({"case-id":[{"name":"worker","kind":"bin-exe","path":relative}]});
    let mut earlier = binaries["rust-binaries"]["case"].clone();
    earlier["binary-id"] = "before-name-filter".into();
    earlier["binary-path"] = json!(before_filter);
    binaries["rust-binaries"]["before-name-filter"] = earlier;
    group.binaries = serde_json::from_value(binaries).unwrap();
    group.build.artifacts.extend(artifacts);
    let binary_metadata = listed.capture_root.join("group-0/binaries.json");
    fs::write(
        &binary_metadata,
        serde_json::to_vec(&group.binaries).unwrap(),
    )
    .unwrap();
    listed
        .controls
        .iter_mut()
        .find(|control| control.path == binary_metadata)
        .unwrap()
        .digest = digest(&binary_metadata).unwrap();

    assert!(listed.inventory.verify_executables().is_err());
    assert_eq!(listed.page(0, 1).unwrap().total, 2);
    assert!(listed.command(0).is_ok());
    fs::remove_file(&helper).unwrap();
    assert!(listed.command(0).is_err());
    fs::copy(&listed.inventory.nextest.captured, helper).unwrap();
    fs::remove_file(before_filter).unwrap();
    assert!(listed.command(0).is_err());
}

#[test]
fn rerun_verification_preserves_exact_source_feature_identity_and_capture_guards() {
    let (_root, intent) = intent();
    let mut listed = listed(intent);
    let case = selection::RerunCase {
        package: "case".into(),
        target: selection::TargetIntent {
            name: "case".into(),
            kind: Some("lib".into()),
        },
        features: selection::Features {
            defaults: false,
            ..Default::default()
        },
        binary_id: "case".into(),
        test_name: "same_name".into(),
    };
    let selection = listed.inventory.selection.clone();
    let basis = selection::SourceBasis::RerunFailed {
        invocation: crate::envelope::CliInvocationId::new(),
    };
    listed.inventory.selection = selection.clone().source(basis.clone(), vec![case.clone()]);
    source::verify_rerun(&listed.inventory).unwrap();
    let mut ambiguous = serde_json::to_value(&listed.inventory).unwrap();
    let group = ambiguous["groups"][0].clone();
    ambiguous["groups"].as_array_mut().unwrap().push(group);
    let group = ambiguous["selection"]["groups"][0].clone();
    ambiguous["selection"]["groups"]
        .as_array_mut()
        .unwrap()
        .push(group);
    let ambiguous = serde_json::from_value(ambiguous).unwrap();
    let Err(WorkflowError::Invalid(message)) = source::verify_rerun(&ambiguous) else {
        panic!("two current matches must refuse the original identity");
    };
    assert!(message.contains("ambiguous"));
    let suite = listed.inventory.groups[0]
        .listing
        .rust_suites
        .values_mut()
        .next()
        .unwrap();
    let current = suite.test_cases.values().next().unwrap().clone();
    let extra = nextest_metadata::TestCaseName::new("another_current_test");
    suite.test_cases.insert(extra, current);
    let Err(WorkflowError::Invalid(message)) = source::verify_rerun(&listed.inventory) else {
        panic!("a rerun must refuse extra current matches");
    };
    assert!(message.contains("outside the original failed identities"));
    listed.inventory.groups[0]
        .listing
        .rust_suites
        .values_mut()
        .next()
        .unwrap()
        .test_cases
        .retain(|name, _| name.as_str() == "same_name");
    let mut wrong_features = case.clone();
    wrong_features
        .features
        .enabled
        .insert("different-feature".into());
    listed.inventory.selection = selection
        .clone()
        .source(basis.clone(), vec![wrong_features]);
    assert!(source::verify_rerun(&listed.inventory).is_err());
    let mut wrong_test = case.clone();
    wrong_test.test_name = "different_test".into();
    listed.inventory.selection = selection.clone().source(basis.clone(), vec![wrong_test]);
    assert!(source::verify_rerun(&listed.inventory).is_err());
    listed.inventory.selection = selection.source(basis, vec![case]);
    fs::remove_file(
        &listed.inventory.groups[0].build.artifacts[0]
            .executable
            .captured,
    )
    .unwrap();
    assert!(source::verify_rerun(&listed.inventory).is_err());
}
pub(super) fn listed(intent: Intent) -> Listed {
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
                    elapsed_seconds: None,
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
        nextest_version: "cargo-nextest 0.9.151 (fixture)".into(),
        nextest_configuration: configuration,
        runner_profile: "selected-profile".into(),
        cargo_timings: Vec::new(),
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
            cargo_metadata_digest: None,
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
    let runner = intent.explain().brief().runner.unwrap();
    assert_eq!(runner.profile, "selected-profile");
    assert_eq!(runner.declared_default_filter, Some("all()"));
    assert!(!root.path().join("target").exists());
    fs::write(root.path().join("Cargo.toml"), "changed declaration").unwrap();
    assert!(matches!(intent.admit(), Err(WorkflowError::Changed(_))));
    assert!(!root.path().join("target").exists());
}

#[test]
fn doctest_only_target_remains_a_cargo_root_with_separate_inventory_and_verdict() {
    let (_root, mut intent) = intent();
    let target = &mut intent.context.metadata.packages[0].targets[0];
    target.test = false;
    target.doctest = true;
    let request = Request {
        operation: selection::Operation::Test,
        scope: selection::Scope::Workspace {
            exclude: Vec::new(),
            features: selection::Features::default(),
        },
        platform: None,
        non_incremental: false,
        filters: selection::Filters::default(),
        ignored: selection::IgnoredPolicy::Default,
    };
    intent.plan = selection::plan(&intent.context.metadata, request).unwrap();
    assert_eq!(intent.plan.groups()[0].roots()[0].package_name(), "case");
    let explanation = intent.explain().brief();
    let [target] = explanation.evidence.cargo_targets.as_slice() else {
        panic!("expected the library declaration");
    };
    assert_eq!(target.kinds, ["lib"]);
    assert!(!target.test);
    assert!(target.doctest);
    assert!(matches!(
        explanation.evidence.doctest_route,
        Information::Available("just doctest")
    ));
    assert!(matches!(
        explanation.admission,
        Information::Unavailable { .. }
    ));
    assert!(matches!(
        explanation.evidence.nextest_inventory,
        Information::Unavailable { .. }
    ));
    assert!(matches!(
        explanation.evidence.original_verdict,
        Information::Unavailable { .. }
    ));
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
        non_incremental: false,
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
        Information::Unavailable { reason } if reason == "Nextest inventory requires explicit build/list"
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
    let explanation = listed.explain();
    assert!(matches!(
        explanation.evidence.nextest_inventory,
        Information::Available(_)
    ));
    assert!(matches!(
        explanation.evidence.original_verdict,
        Information::Unavailable { .. }
    ));
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
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
    let initial = fs::metadata(&first).unwrap();
    assert_eq!(initial.mode() & 0o777, 0o400);
    assert_eq!(listed.run_configuration(0).unwrap(), first);
    assert_eq!(fs::metadata(&first).unwrap().ino(), initial.ino());
    let mut actual: toml::Value = toml::from_str(&fs::read_to_string(&first).unwrap()).unwrap();
    let expected: toml::Value = toml::from_str(std::str::from_utf8(original).unwrap()).unwrap();
    assert_eq!(
        actual["profile"]["selected-profile"]["junit"]["path"].as_str(),
        execute::junit::transport_path(&listed.report_path(0)).to_str()
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
        toml::Value::String(execute::junit::transport_path(&listed.report_path(0)).to_str().unwrap().into())
    );
    fs::set_permissions(&first, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(listed.run_configuration(0).is_err());
    assert_eq!(fs::metadata(&first).unwrap().mode() & 0o777, 0o600);
    fs::write(&first, b"changed original configuration").unwrap();
    fs::set_permissions(&first, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(listed.run_configuration(0).is_err());
    assert_eq!(fs::read(&first).unwrap(), b"changed original configuration");
    listed.inventory.runner_profile = "missing".into();
    assert!(
        listed.run_configuration(0).is_err(),
        "projection cannot create an otherwise absent native profile"
    );
}
