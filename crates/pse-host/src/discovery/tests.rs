// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use serde_json::{Value, json};

fn binary(name: &str) -> Value {
    json!({"binary-id":format!("alpha::{name}"),"binary-name":name,"package-id":"path+file:///workspace/alpha#0.1.0","kind":"test","binary-path":format!("/captured/{name}"),"build-platform":"target"})
}
fn meta() -> Value {
    json!({"target-directory":"/workspace/target","base-output-directories":["debug"],"non-test-binaries":{},"linked-paths":[]})
}
fn binaries() -> BinaryListSummary {
    serde_json::from_value(json!({"rust-build-meta":meta(),"rust-binaries":{"alpha::first":binary("first"),"alpha::second":binary("second")}})).unwrap()
}
fn listing() -> TestListSummary {
    let suite = |name: &str| {
        let mut suite = binary(name);
        suite["package-name"] = json!("alpha");
        suite["cwd"] = json!("/workspace/alpha");
        suite["status"] = json!("listed");
        suite["testcases"] = json!({"same_name":{"kind":"test","ignored":false,"filter-match":{"status":"matches"}},"ignored_test":{"kind":"test","ignored":true,"filter-match":{"status":"mismatch","reason":"ignored"}}});
        suite
    };
    serde_json::from_value(json!({"rust-build-meta":meta(),"test-count":4,"rust-suites":{"alpha::first":suite("first"),"alpha::second":suite("second")}})).unwrap()
}

#[test]
fn official_metadata_preserves_duplicate_names_and_ignored_selection() {
    let listing = listing();
    validate_listing(&binaries(), &listing, IgnoredPolicy::Default).unwrap();
    let selected: Vec<_> = listing
        .rust_suites
        .values()
        .flat_map(|suite| {
            suite
                .test_cases
                .iter()
                .filter(|(_, case)| case.filter_match.is_match())
                .map(move |(name, _)| (suite.binary.binary_id.to_string(), name.to_string()))
        })
        .collect();
    assert_eq!(
        selected,
        [
            ("alpha::first".into(), "same_name".into()),
            ("alpha::second".into(), "same_name".into())
        ]
    );
    assert!(listing.rust_suites.values().all(|suite| {
        suite.test_cases[&nextest_metadata::TestCaseName::new("ignored_test")].ignored
    }));
}
#[test]
fn missing_binary_count_mismatch_and_substituted_binary_refuse() {
    let mut data = listing();
    data.rust_suites
        .remove(&nextest_metadata::RustBinaryId::new("alpha::first"));
    data.test_count = 2;
    assert!(validate_listing(&binaries(), &data, IgnoredPolicy::Default).is_err());
    let mut data = listing();
    data.test_count = 1;
    assert!(validate_listing(&binaries(), &data, IgnoredPolicy::Default).is_err());
    let mut data = listing();
    data.rust_suites
        .values_mut()
        .next()
        .unwrap()
        .binary
        .package_id = "different-source#alpha".into();
    assert!(validate_listing(&binaries(), &data, IgnoredPolicy::Default).is_err());
}
#[test]
fn zero_match_and_unlisted_suites_are_distinct_from_missing_binary_records() {
    let mut data = listing();
    for suite in data.rust_suites.values_mut() {
        for case in suite.test_cases.values_mut() {
            case.filter_match = nextest_metadata::FilterMatch::Mismatch {
                reason: nextest_metadata::MismatchReason::String,
            };
        }
    }
    validate_listing(&binaries(), &data, IgnoredPolicy::Default).unwrap();
    assert!(
        data.rust_suites
            .values()
            .all(|suite| suite.status == RustTestSuiteStatusSummary::LISTED)
    );
    data.rust_suites.values_mut().next().unwrap().status = RustTestSuiteStatusSummary::SKIPPED;
    assert!(validate_listing(&binaries(), &data, IgnoredPolicy::Default).is_err());
    data.rust_suites
        .values_mut()
        .next()
        .unwrap()
        .test_cases
        .clear();
    data.test_count = 2;
    validate_listing(&binaries(), &data, IgnoredPolicy::Default).unwrap();
}

#[test]
fn changed_ignored_policy_cannot_reinterpret_original_selected_tests() {
    let data = listing();
    assert!(validate_listing(&binaries(), &data, IgnoredPolicy::Only).is_err());
    let mut data = listing();
    for suite in data.rust_suites.values_mut() {
        for case in suite.test_cases.values_mut() {
            case.filter_match = nextest_metadata::FilterMatch::Matches;
        }
    }
    assert!(validate_listing(&binaries(), &data, IgnoredPolicy::Default).is_err());
    validate_listing(&binaries(), &data, IgnoredPolicy::All).unwrap();
}

#[test]
fn metadata_cannot_invent_a_built_artifact_or_change_the_embedded_binary_id() {
    let mut declared = binaries();
    let captured = BuildCapture {
        version: 1,
        resolution: crate::correctness::Resolution {
            arguments: vec![],
            packages: vec![],
            units: vec![],
            native_capabilities: BTreeSet::new(),
        },
        artifacts: vec![],
        cargo_exit: 0,
        incomplete: Default::default(),
    };
    assert!(
        pin_binaries(&mut declared, &captured)
            .unwrap_err()
            .to_string()
            .contains("no same-build captured Cargo artifact")
    );
    let mut declared = binaries();
    declared
        .rust_binaries
        .values_mut()
        .next()
        .unwrap()
        .binary_id = nextest_metadata::RustBinaryId::new("wrong-identity");
    assert!(
        pin_binaries(&mut declared, &captured)
            .unwrap_err()
            .to_string()
            .contains("map key differs")
    );
}

fn non_test_capture() -> (tempfile::TempDir, BinaryListSummary, BuildCapture) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("target");
    let original = target.join("debug/pse-worker");
    fs::create_dir_all(original.parent().unwrap()).unwrap();
    fs::write(&original, b"original same-build worker").unwrap();
    fs::set_permissions(&original, fs::Permissions::from_mode(0o700)).unwrap();
    let executable =
        capture::executable(&original, &directory.path().join("captured-worker")).unwrap();
    let package = "path+file:///workspace/alpha#0.1.0";
    let cargo = serde_json::from_value(json!({
        "package_id":package,
        "manifest_path":"/workspace/alpha/Cargo.toml",
        "target":{"kind":["bin"],"crate_types":["bin"],"name":"pse-worker",
            "src_path":"/workspace/alpha/src/bin/pse-worker.rs","edition":"2024",
            "doctest":false,"test":false,"doc":false},
        "profile":{"opt_level":"0","debuginfo":0,"debug_assertions":false,
            "overflow_checks":false,"test":false},
        "features":[],"filenames":[original],"executable":original,"fresh":false
    }))
    .unwrap();
    let mut metadata = meta();
    metadata["target-directory"] = json!(target);
    metadata["non-test-binaries"] = json!({package:[
        {"name":"pse-worker","kind":"bin-exe","path":"debug/pse-worker"},
        {"name":"linked","kind":"dylib","path":"debug/liblinked.so"}
    ]});
    let binaries = serde_json::from_value(json!({
        "rust-build-meta":metadata,"rust-binaries":{}
    }))
    .unwrap();
    let captured = BuildCapture {
        version: 1,
        resolution: crate::correctness::Resolution {
            arguments: vec![],
            packages: vec![],
            units: vec![],
            native_capabilities: BTreeSet::new(),
        },
        artifacts: vec![BuiltArtifact { cargo, executable }],
        cargo_exit: 0,
        incomplete: Default::default(),
    };
    (directory, binaries, captured)
}

#[test]
fn native_non_test_binary_mapping_survives_original_cargo_output_replacement() {
    let (_directory, mut declared, captured) = non_test_capture();
    pin_binaries(&mut declared, &captured).unwrap();
    let metadata = &declared.rust_build_meta;
    let files = metadata.non_test_binaries.values().next().unwrap();
    let binary = files.iter().find(|file| file.name == "pse-worker").unwrap();
    assert!(
        binary.path.is_relative(),
        "retain nextest's relative-path contract"
    );
    let mapped = metadata.target_directory.join(&binary.path);
    assert_eq!(
        mapped.canonicalize().unwrap(),
        captured.artifacts[0]
            .executable
            .captured
            .canonicalize()
            .unwrap()
    );
    fs::write(
        &captured.artifacts[0].executable.original,
        b"later different-feature worker",
    )
    .unwrap();
    assert_eq!(fs::read(mapped).unwrap(), b"original same-build worker");
    captured.artifacts[0].executable.verify().unwrap();
    assert_eq!(
        files
            .iter()
            .find(|file| file.name == "linked")
            .unwrap()
            .path,
        "debug/liblinked.so"
    );
}

#[test]
fn native_non_test_mapping_refuses_missing_foreign_or_ambiguous_cargo_artifacts() {
    for mode in ["missing", "foreign", "ambiguous"] {
        let (_directory, mut declared, mut captured) = non_test_capture();
        match mode {
            "missing" => captured.artifacts.clear(),
            "foreign" => captured.artifacts[0].cargo.package_id.repr = "other-package".into(),
            "ambiguous" => captured.artifacts.push(captured.artifacts[0].clone()),
            _ => unreachable!(),
        }
        assert!(pin_binaries(&mut declared, &captured).is_err(), "{mode}");
    }
}

#[tokio::test]
async fn explicit_real_host_listing_captures_actual_inventory_without_setup_or_providers() {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let data = cargo_metadata::MetadataCommand::new()
        .current_dir(checkout)
        .no_deps()
        .exec()
        .unwrap();
    let request = selection::Request {
        operation: Operation::Test,
        scope: selection::Scope::Packages(vec![selection::PackageIntent {
            package: "pse-host".into(),
            target: Some(selection::TargetIntent {
                name: "pse_host".into(),
                kind: Some("lib".into()),
            }),
            features: selection::Features::default(),
        }]),
        platform: None,
        filters: selection::Filters {
            ignore_default_filter: false,
            words: vec!["source_symlinks_and_existing_capture_destinations_refuse".into()],
            expressions: vec![],
        },
        ignored: IgnoredPolicy::Default,
    };
    let resolved = selection::plan(&data, request).unwrap().resolve().unwrap();
    let root = tempfile::tempdir().unwrap();
    let nextest = std::env::var_os("PATH")
        .into_iter()
        .flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .map(|path| path.join("cargo-nextest"))
        .find(|path| path.is_file())
        .unwrap()
        .canonicalize()
        .unwrap();
    let inventory = list(
        &resolved,
        checkout,
        &nextest,
        &root.path().join("inventory"),
        "local",
        &std::env::vars_os().collect(),
        DiscoveryOptions {
            timings: false,
            policy: crate::managed::payload::PayloadPolicy {
                startup: std::time::Duration::from_secs(30),
                drain: std::time::Duration::from_secs(30),
                write: std::time::Duration::from_secs(30),
            },
        },
    )
    .await
    .unwrap();
    assert_eq!(inventory.setup, SetupObservation::NotRunByListing);
    assert_eq!(inventory.runner_profile, "local");
    assert_eq!(
        inventory.physical_scope,
        PhysicalScopeObservation::NotEstablishedByDiscovery
    );
    #[derive(serde::Deserialize)]
    struct Claim {
        owner: pse_diagnostics::process_facts::ProcessIdentity,
    }
    let claims = checkout
        .join("target/host-tools/.uses")
        .join(&inventory.host_generation.generation);
    for claim in fs::read_dir(claims).unwrap() {
        let path = claim.unwrap().path();
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => panic!("{error}"),
        };
        let claim: Claim = serde_json::from_slice(&bytes).unwrap();
        assert_ne!(
            claim.owner.pid,
            std::process::id(),
            "this listing released its original generation before children; other agents retain their independent claims"
        );
    }
    assert!(inventory.all_suites_listed());
    assert_eq!(inventory.selected_tests().len(), 1);
    assert!(
        inventory.selected_tests()[0]
            .test_name
            .ends_with("source_symlinks_and_existing_capture_destinations_refuse")
    );
    let exact = inventory.test(&inventory.selected_tests()[0]).unwrap();
    assert!(!exact.case.ignored);
    assert_eq!(exact.ignored_policy, IgnoredPolicy::Default);
    let mut missing = exact.identity.clone();
    missing.test_name = "absent_test".into();
    assert!(inventory.test(&missing).is_err());
    assert!(
        inventory.groups[0]
            .build
            .resolution
            .native_capabilities
            .is_empty()
    );
    inventory.verify_executables().unwrap();
    for binary in inventory.groups[0].binaries.rust_binaries.values() {
        assert!(binary.binary_path.as_std_path().starts_with(root.path()));
    }
}
