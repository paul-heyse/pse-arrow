// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{
    correctness,
    selection::{
        Features, Filters, IgnoredPolicy, Operation, PackageIntent, Request, Scope, TargetIntent,
    },
};
use serde_json::json;
use std::ffi::OsString;

fn metadata(declarations: serde_json::Value) -> Metadata {
    serde_json::from_value(json!({
        "packages":[{
            "name":"example", "version":"0.1.0", "id":"path+file:///workspace/example#0.1.0", "source":null,
            "manifest_path":"/workspace/example/Cargo.toml", "dependencies":[],
            "targets":[{"name":"owned", "kind":["test"], "crate_types":["bin"], "src_path":"/workspace/example/tests/owned.rs", "edition":"2024"}],
            "features":{"native":[]}, "metadata":{"pse":{"test-fixtures":declarations}}
        }],
        "workspace_members":["path+file:///workspace/example#0.1.0"],
        "workspace_default_members":["path+file:///workspace/example#0.1.0"],
        "workspace_root":"/workspace", "target_directory":"/workspace/target", "version":1
    })).unwrap()
}
fn owned() -> serde_json::Value {
    json!({"id":"owned-service", "target":"owned", "kind":"test", "ownership":"target-owned", "inputs":["server","study-module","service-helper"]})
}
fn resolved(data: &Metadata, features: &[&str], operation: Operation) -> selection::Resolved {
    let request = Request {
        operation,
        scope: Scope::Packages(vec![PackageIntent {
            package: "example".into(),
            target: Some(TargetIntent {
                name: "owned".into(),
                kind: Some("test".into()),
            }),
            features: Features::default(),
        }]),
        platform: None,
        filters: Filters::default(),
        ignored: IgnoredPolicy::Default,
    };
    selection::plan(data, request)
        .unwrap()
        .resolve_with(|arguments: &[OsString]| {
            Ok(correctness::Resolution {
                arguments: arguments.to_vec(),
                packages: vec!["example".into()],
                native_capabilities: BTreeSet::new(),
                units: vec![CompilerUnit {
                    package_id: "path+file:///workspace/example#0.1.0".into(),
                    package_name: "example".into(),
                    target_name: "owned".into(),
                    target_kinds: vec!["test".into()],
                    source_path: "/workspace/example/tests/owned.rs".into(),
                    features: features
                        .iter()
                        .map(|feature| String::from(*feature))
                        .collect(),
                    platform: None,
                    mode: if operation == Operation::Test {
                        "test"
                    } else {
                        "check"
                    }
                    .into(),
                    root: true,
                }],
            })
        })
        .unwrap()
}
fn requirement() -> Requirement {
    let data = metadata(json!([owned()]));
    requirements(&data, &resolved(&data, &[], Operation::Test))
        .unwrap()
        .remove(0)
}

#[test]
fn pure_selection_neither_prepares_tools_nor_requires_a_database() {
    let directory = tempfile::tempdir().unwrap();
    let destination = directory.path().join("must-not-be-created");
    let prepared = PreparedExecution::prepare(
        &directory.path().join("not-a-checkout"),
        &BTreeMap::new(),
        &destination,
        &[],
    );
    assert!(prepared.is_ok());
    assert!(!destination.exists());
}

#[test]
fn unsupported_evidence_input_refuses_before_preparation_or_service_effects() {
    let directory = tempfile::tempdir().unwrap();
    let destination = directory.path().join("must-not-be-created");
    let mut requirement = requirement();
    requirement
        .declaration
        .inputs
        .insert(Input::ScientificInputs);
    let error = PreparedExecution::prepare(
        directory.path(),
        &BTreeMap::new(),
        &destination,
        &[requirement],
    )
    .unwrap_err();
    assert!(matches!(
        error,
        FixtureError::Unprepared(Input::ScientificInputs)
    ));
    assert!(!destination.exists());
}

#[test]
fn explicit_scientific_input_remains_with_its_original_owner_without_helper_builds() {
    let directory = tempfile::tempdir().unwrap();
    let destination = directory.path().join("must-not-be-created");
    let mut requirement = requirement();
    requirement.declaration.ownership = Ownership::PreparationOnly;
    requirement.declaration.inputs = BTreeSet::from([Input::ScientificInputs]);
    let environment = BTreeMap::from([(
        "PSE_SCIENTIFIC_INPUTS".into(),
        "/original/capsule.json".into(),
    )]);
    PreparedExecution::prepare(directory.path(), &environment, &destination, &[requirement])
        .unwrap();
    assert!(
        !destination.exists(),
        "the test owner, not the host, interprets the selected capsule"
    );
}

#[test]
fn actual_admitted_root_selects_authored_ownership_and_inputs_without_preparation() {
    let data = metadata(json!([owned()]));
    let selected = requirements(&data, &resolved(&data, &[], Operation::Test)).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].ownership(), Ownership::TargetOwned);
    assert_eq!(
        selected[0].inputs(),
        &BTreeSet::from([Input::Server, Input::StudyModule, Input::ServiceHelper])
    );
    assert_eq!(
        (selected[0].group(), selected[0].target_name()),
        (0, "owned")
    );
    assert!(selected[0].applies_to_test("any_case"));
    assert!(
        requirements(&data, &resolved(&data, &[], Operation::Check))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn qualified_modules_have_exact_boundaries_and_feature_scope_is_actual() {
    let mut declaration = owned();
    declaration["scope"] = json!({"tests":["exact_case"], "modules":["outer::owned"]});
    declaration["required-features"] = json!(["native"]);
    let data = metadata(json!([declaration]));
    assert!(
        requirements(&data, &resolved(&data, &[], Operation::Test))
            .unwrap()
            .is_empty()
    );
    let selected = requirements(&data, &resolved(&data, &["native"], Operation::Test)).unwrap();
    let selected = &selected[0];
    assert!(selected.applies_to_test("exact_case"));
    assert!(selected.applies_to_test("outer::owned::nested::case"));
    assert!(!selected.applies_to_test("outer::owned_extra::case"));
    assert!(!selected.applies_to_test("prefix::outer::owned::case"));
    assert!(!selected.declaration().scope.is_whole_target());
}

#[test]
fn malformed_or_contradictory_declarations_refuse() {
    for change in [
        json!({"target":"missing"}),
        json!({"required-features":["unknown"]}),
        json!({"inputs":[]}),
        json!({"inputs":["server"]}),
        json!({"scope":{"modules":["test(foo)"]}}),
        json!({"scope":{"modules":["outer::"]}}),
        json!({"typo":true}),
    ] {
        let mut declaration = owned();
        for (key, value) in change.as_object().unwrap() {
            declaration[key] = value.clone();
        }
        let data = metadata(json!([declaration]));
        assert!(requirements(&data, &resolved(&data, &[], Operation::Test)).is_err());
    }
    let data = metadata(json!([owned(), owned()]));
    assert!(requirements(&data, &resolved(&data, &[], Operation::Test)).is_err());
}

#[test]
fn missing_sources_report_exact_prepare_routes_and_do_not_adopt_arbitrary_paths() {
    let requirement = requirement();
    let observed = readiness(&requirement, &Sources::default()).unwrap();
    let InputState::Unprepared { code, next } = &observed.inputs[&Input::StudyModule] else {
        panic!("missing prepared module")
    };
    assert_eq!(code, "fixture.unprepared");
    assert_eq!(
        next.as_slice(),
        ["scripts/pse-host", "study-module", "prepare"]
    );
    let InputState::Unprepared { next, .. } = &observed.inputs[&Input::Server] else {
        panic!("missing prepared server")
    };
    assert_eq!(next.as_slice(), ["scripts/pse-host", "server", "prepare"]);
    assert!(matches!(
        observed.inputs[&Input::ServiceHelper],
        InputState::OwnerAdmissionRequired { path: None }
    ));
    let mut sources = Sources::default();
    sources
        .locations
        .insert(Input::ServiceHelper, Path::new("/nonexistent/helper"));
    sources
        .locations
        .insert(Input::StudyModule, Path::new("/original/module-state"));
    let observed = readiness(&requirement, &sources).unwrap();
    assert!(
        matches!(&observed.inputs[&Input::ServiceHelper],InputState::OwnerAdmissionRequired {path:Some(path)} if path == Path::new("/nonexistent/helper"))
    );
    assert!(
        matches!(&observed.inputs[&Input::StudyModule],InputState::OwnerAdmissionRequired {path:Some(path)} if path == Path::new("/original/module-state"))
    );
}

#[test]
fn existing_capture_owner_verifies_bytes_without_conferring_source_build_or_service_authority() {
    use sha2::{Digest, Sha256};
    let directory = tempfile::tempdir().unwrap();
    let retained = directory.path().join("retained-helper");
    std::fs::write(&retained, b"original immutable bytes").unwrap();
    let executable = Executable {
        original: "/mutable/cargo/helper".into(),
        captured: retained.clone(),
        bytes: 24,
        sha256: Sha256::digest(b"original immutable bytes")
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    };
    let mut sources = Sources::default();
    sources
        .executables
        .insert(Input::ServiceHelper, &executable);
    let observed = readiness(&requirement(), &sources).unwrap();
    assert_eq!(observed.ownership, Ownership::TargetOwned);
    assert!(
        matches!(&observed.inputs[&Input::ServiceHelper],InputState::CapturedBytes {path,..} if path == &retained)
    );
    std::fs::write(&retained, b"changed bytes").unwrap();
    assert!(matches!(
        readiness(&requirement(), &sources),
        Err(FixtureError::Capture(_))
    ));
}

#[test]
fn runner_borrowing_is_distinct_and_requires_its_actual_helper() {
    let mut declaration = owned();
    declaration["ownership"] = json!("runner-borrowed");
    let data = metadata(json!([declaration.clone()]));
    let requirements = requirements(&data, &resolved(&data, &[], Operation::Test)).unwrap();
    assert_eq!(requirements[0].ownership(), Ownership::RunnerBorrowed);
    declaration["inputs"] = json!(["server", "study-module"]);
    let data = metadata(json!([declaration]));
    assert!(super::requirements(&data, &resolved(&data, &[], Operation::Test)).is_err());
}

#[test]
fn real_manifest_declarations_cover_both_ownership_modes_without_a_parallel_registry() {
    let metadata = cargo_metadata::MetadataCommand::new()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .no_deps()
        .exec()
        .unwrap();
    let operations = metadata
        .packages
        .iter()
        .find(|package| package.name.as_str() == "pse-operations")
        .unwrap();
    let runtime = metadata
        .packages
        .iter()
        .find(|package| package.name.as_str() == "pse-runtime")
        .unwrap();
    let operations = declarations(operations).unwrap();
    let runtime = declarations(runtime).unwrap();
    assert!(
        operations
            .iter()
            .any(|declaration| declaration.target == "native_service"
                && declaration.ownership == Ownership::TargetOwned
                && !declaration.inputs.contains(&Input::ServiceHelper))
    );
    assert!(
        operations
            .iter()
            .any(|declaration| declaration.ownership == Ownership::RunnerBorrowed)
    );
    assert!(!operations.iter().any(|declaration| declaration.target == "pse_operations" && declaration.scope.contains("canonical_staging::canonical_server_unit::declaration_count_and_metadata_extent_remain_bounded")));
    for target in [
        "native_study",
        "native_repeated_analysis",
        "native_adoption",
        "native_research",
        "native_boundary_acceptance",
        "native_worker_lifecycle",
        "worker",
    ] {
        assert!(
            runtime
                .iter()
                .any(|declaration| declaration.target == target
                    && declaration.inputs.contains(&Input::StudyModule)),
            "{target}"
        );
    }
    assert!(
        runtime
            .iter()
            .any(|declaration| declaration.target == "pse_runtime"
                && declaration.inputs.contains(&Input::Worker))
    );
}

#[test]
fn real_scopes_distinguish_local_covariance_and_memory_controls_from_owned_service_and_worker_cases()
 {
    let metadata = cargo_metadata::MetadataCommand::new()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .no_deps()
        .exec()
        .unwrap();
    let runtime = metadata
        .packages
        .iter()
        .find(|package| package.name.as_str() == "pse-runtime")
        .unwrap();
    let declarations = declarations(runtime).unwrap();
    let inputs = |target: &str, test: &str| {
        declarations
            .iter()
            .filter(|declaration| declaration.target == target && declaration.scope.contains(test))
            .flat_map(|declaration| declaration.inputs.iter().copied())
            .collect::<BTreeSet<_>>()
    };
    assert!(
        inputs(
            "native_adoption",
            "native_nonunit_importance_withholds_fit_covariance"
        )
        .is_empty()
    );
    assert!(
        inputs(
            "native_adoption",
            "native_fit_covariance_retains_physical_custody_and_refuses_payload_tampering"
        )
        .is_empty()
    );
    assert!(inputs("native_adoption","canonical::native_canonical_covariance_reopens_exact_fit_and_refuses_tampered_or_missing_reference").contains(&Input::StudyModule));
    let memory = inputs(
        "native_boundary_acceptance",
        "public_durable_scip_memory_stop_retains_unavailable_feasibility_and_exact_rows",
    );
    assert!(memory.contains(&Input::StudyModule));
    assert!(!memory.contains(&Input::Worker));
    assert!(
        !inputs(
            "native_boundary_acceptance",
            "public_managed_detail_keeps_external_owners_exact_attempts_and_complete_prefixes"
        )
        .contains(&Input::Worker),
        "same-package integration targets receive the worker from Cargo"
    );
    assert!(
        !inputs(
            "pse_runtime",
            "workflow::tests::workflow_composition_binds_same_owner_factory_without_a_store"
        )
        .contains(&Input::Server)
    );
    assert!(
        inputs(
            "pse_runtime",
            "workflow::tests::workflow_durability_selection_uses_this_deployment"
        )
        .contains(&Input::Server)
    );
}
