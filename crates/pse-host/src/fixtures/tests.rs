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
    let assessed = ExecutionInputs::assess(
        &directory.path().join("not-a-checkout"),
        &BTreeMap::from([(
            "UNRELATED_CREDENTIAL".into(),
            "private-environment-value".into(),
        )]),
        &[],
    );
    assert!(!format!("{assessed:?}").contains("private-environment-value"));
    let prepared = assessed.prepare(&destination);
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
    let error = ExecutionInputs::assess(directory.path(), &BTreeMap::new(), &[requirement])
        .prepare(&destination)
        .unwrap_err();
    assert!(matches!(
        error,
        FixtureError::Inputs(ref inputs) if inputs.iter().any(|failure| failure.input == Input::ScientificInputs)
    ));
    assert!(!destination.exists());
}

#[test]
fn exact_selection_reports_every_missing_original_input_before_effects() {
    let directory = tempfile::tempdir().unwrap();
    let destination = directory.path().join("must-not-be-created");
    let mut requirement = requirement();
    requirement.declaration.inputs = BTreeSet::from([
        Input::ScientificInputs,
        Input::NativeLane,
        Input::EngineeringEndpoint,
    ]);
    let environment = BTreeMap::from([("PSE_ENGINEERING_ENDPOINT".into(), "".into())]);
    let error = ExecutionInputs::assess(directory.path(), &environment, &[requirement])
        .prepare(&destination)
        .unwrap_err();
    let FixtureError::Inputs(inputs) = error else {
        panic!("expected the complete missing-input population");
    };
    assert_eq!(
        inputs
            .iter()
            .filter(|failure| matches!(failure.cause.as_ref(), FixtureError::Unprepared(_)))
            .map(|failure| failure.input)
            .collect::<Vec<_>>(),
        [
            Input::EngineeringEndpoint,
            Input::ScientificInputs,
            Input::NativeLane
        ]
    );
    assert!(!destination.exists());
}

#[test]
fn exact_readiness_retains_all_refused_missing_and_owner_required_inputs() {
    use pse_diagnostics::TypedDiagnostic as _;
    let directory = tempfile::tempdir().unwrap();
    let destination = directory.path().join("must-not-be-created");
    let mut requirement = requirement();
    requirement.declaration.inputs.extend([
        Input::ScientificInputs,
        Input::NativeLane,
        Input::EngineeringEndpoint,
        Input::DiagnosticEvidence,
    ]);
    let environment = BTreeMap::from([
        (
            "PSE_NATIVE_FIXTURE_MODULE_RECEIPT".into(),
            directory.path().join("absent-module").into_os_string(),
        ),
        (
            "PSE_SURREAL_TOOLS".into(),
            directory.path().join("absent-server").into_os_string(),
        ),
        ("PSE_ENGINEERING_ENDPOINT".into(), "".into()),
        (
            "PSE_DIAGNOSTIC_EVIDENCE".into(),
            "/original/evidence".into(),
        ),
    ]);
    let assessment = ExecutionInputs::assess(directory.path(), &environment, &[requirement]);
    assert!(assessment.blocked());
    let observed = &assessment.observations()[0];
    assert_eq!(observed.inputs.len(), 7);
    for input in [Input::Server, Input::StudyModule] {
        let InputState::Refused { next, .. } = &observed.inputs[&input] else {
            panic!("original owner refusal missing");
        };
        assert_eq!(next, &prepare(input));
    }
    for input in [
        Input::ScientificInputs,
        Input::NativeLane,
        Input::EngineeringEndpoint,
    ] {
        let InputState::Unprepared { code, next } = &observed.inputs[&input] else {
            panic!("original missing-input observation missing");
        };
        assert_eq!(code, "fixture.unprepared");
        assert_eq!(next, &prepare(input));
    }
    assert!(
        matches!(&observed.inputs[&Input::DiagnosticEvidence], InputState::OwnerAdmissionRequired { path: Some(path) } if path == Path::new("/original/evidence"))
    );
    assert!(matches!(
        observed.inputs[&Input::ServiceHelper],
        InputState::OwnerAdmissionRequired { path: None }
    ));
    let error = assessment.prepare(&destination).unwrap_err();
    assert_eq!(error.diagnostic_children().unwrap().count(), 5);
    let FixtureError::Inputs(failures) = error else {
        panic!("aggregate original causes missing");
    };
    assert_eq!(failures.len(), 5);
    assert!(
        failures
            .iter()
            .all(|failure| failure.cause.diagnostic_code().is_some())
    );
    assert!(!destination.exists());
    assert!(!directory.path().join("absent-server").exists());
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
    ExecutionInputs::assess(directory.path(), &environment, &[requirement])
        .prepare(&destination)
        .unwrap();
    assert!(
        !destination.exists(),
        "the test owner, not the host, interprets the selected capsule"
    );
}

#[test]
fn explicit_predecessor_receipt_precedes_state_without_implicit_preparation() {
    let directory = tempfile::tempdir().unwrap();
    let destination = directory.path().join("must-not-be-created");
    let mut requirement = requirement();
    requirement.declaration.ownership = Ownership::PreparationOnly;
    requirement.declaration.inputs = BTreeSet::from([Input::PredecessorModuleState]);
    let state = OsString::from(PREDECESSOR_MODULE_STATE_ENV);
    let receipt = OsString::from(PREDECESSOR_MODULE_RECEIPT_ENV);
    for environment in [
        BTreeMap::from([(state.clone(), "/recorded/state".into())]),
        BTreeMap::from([(receipt.clone(), "/retained/module.json".into())]),
    ] {
        ExecutionInputs::assess(directory.path(), &environment, &[requirement.clone()])
            .prepare(&destination)
            .unwrap();
        assert!(
            !destination.exists(),
            "the native test owner admits the original artifact"
        );
    }
    let empty_explicit_receipt =
        BTreeMap::from([(state, "/recorded/state".into()), (receipt, "".into())]);
    assert!(matches!(
        ExecutionInputs::assess(
            directory.path(),
            &empty_explicit_receipt,
            &[requirement]
        ).prepare(&destination),
        Err(FixtureError::Inputs(ref inputs)) if inputs.len() == 1 && inputs[0].input == Input::PredecessorModuleState
    ));
    assert!(!destination.exists());
}

#[test]
fn target_owned_recorded_module_state_is_not_replaced_by_a_prepared_module() {
    let directory = tempfile::tempdir().unwrap();
    let destination = directory.path().join("must-not-be-created");
    let mut requirement = requirement();
    requirement.declaration.inputs = BTreeSet::from([Input::StudyModule]);
    let environment = BTreeMap::from([(
        "PSE_NATIVE_FIXTURE_MODULE_STATE".into(),
        "/original/owner-admitted-state".into(),
    )]);
    let requirements = vec![requirement];
    let prepared = ExecutionInputs::assess(directory.path(), &environment, &requirements)
        .prepare(&destination)
        .unwrap();
    let command = prepared
        .command(
            crate::test_workflow::GroupCommand {
                program: "/original/test-owner".into(),
                arguments: Vec::new(),
                cwd: directory.path().to_owned(),
                environment: environment.clone(),
                group: 0,
                runtime_inputs: crate::discovery::RuntimeInputObservation::NotCapturedByDiscovery,
            },
            &requirements,
            &directory.path().join("unused-inventory.json"),
            directory.path(),
            &crate::envelope::CliInvocationId::new(),
        )
        .unwrap();
    assert_eq!(command.environment, environment);
    assert_eq!(command.program, PathBuf::from("/original/test-owner"));
    assert!(!destination.exists());
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
