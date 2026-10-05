// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable study definition and durable publication through the existing worker owners.
use super::durable_tests::quick;
use super::worker_tests::{ipopt, job_durable, sources};
use super::*;
use pse_model::study::*;
use pse_operations::{lifecycle::AttemptState, testing::TestDatabase};
use std::time::Duration;

const CASES: &str = r#"package algebraic {
def Root { param a:Scalar=4; var x:Scalar; eq square:x*x==a; annotation start x(1); annotation bounds x(0,10); annotation report x("root"); annotation check x(x>1); }
def Failed { var x:Scalar; eq square:x*x == -1; annotation start x(1); annotation report x("root"); }
def Fixed { param a:Scalar=4; param x:Scalar=2; eq square:x*x==a; annotation report x("root"); annotation check x(x>1); }
def Storage { domain t:Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]:Time; eq rate[i in t]:d(x[i])/di==1; eq initial:x[0{s}]==1{s}; }
test dynamic fixture { dof 0; route integrated; procedure integrate; integrate samples(0{s},1{s}) relative(1e-8) normalized_absolute(1e-10) step(1e-4{s}); } {child root:Storage=Storage();}
}"#;
fn point(
    case: pse_model::generated::identities::DeclarationId,
    key: u32,
    dependencies: Vec<Dependency>,
    start: StartPolicy,
) -> StudyPoint {
    StudyPoint {
        operation: OperationRequest::DeclaredCase(CaseOperation {
            case,
            route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
            settings: ipopt(),
        }),
        preparation: PreparationSettings {
            compiler: tests::compiler_profile(),
            ..Default::default()
        },
        overlay: PointOverlay::default(),
        policy: StudyPointPolicy {
            key: OccurrenceKey(key),
            dependencies,
            start,
            attempt_limit: 1,
        },
    }
}
async fn admitted(
    runtime: &Runtime,
    points: impl FnOnce(
        pse_model::generated::identities::DeclarationId,
        pse_model::generated::identities::DeclarationId,
        pse_model::generated::identities::DeclarationId,
    ) -> Vec<StudyPoint>,
) -> (PackageSources, StudyDefinition) {
    let (physical, modeling) = sources(CASES);
    let cancel = crate::CancelSource::new();
    let package = runtime
        .package_from_sources(
            std::slice::from_ref(&modeling),
            runtime
                .physical_from_sources(&physical, &cancel)
                .await
                .unwrap(),
        )
        .unwrap();
    let find = |name: &str| {
        package
            .declarations()
            .iter()
            .find(|declaration| declaration.name == name)
            .unwrap()
            .declaration_id
    };
    let points = points(find("Root"), find("Failed"), find("Fixed"));
    let definition = package
        .admit_study_points(
            crate::authoring_driver::document::package_checksum(&physical),
            vec![crate::authoring_driver::document::package_checksum(
                &modeling,
            )],
            &points,
            &cancel,
        )
        .await
        .unwrap();
    (
        PackageSources {
            physical,
            modeling: vec![modeling],
        },
        definition,
    )
}
#[test]
fn study_request_codec_unit_excludes_owner_seed_need_and_bare_overlays() {
    let case = pse_model::generated::identities::DeclarationId::from_bytes([4; 16]);
    let request = StudyRequest {
        version: pse_model::document::Version,
        points: vec![point(case, 7, vec![], StartPolicy::Fresh)],
    };
    let encoded = serde_json::to_value(request).unwrap();
    assert_eq!(encoded["version"], 4);
    assert!(encoded["points"][0]["policy"].get("seed_need").is_none());
    let decoded: StudyRequest = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), encoded);
    for version in [1, 3] {
        let mut historical = encoded.clone();
        historical["version"] = serde_json::json!(version);
        assert!(serde_json::from_value::<StudyRequest>(historical).is_err());
    }
    let mut bare = encoded;
    bare["points"][0]["overlay"] = serde_json::json!({"values":{"x":1}});
    assert!(serde_json::from_value::<StudyRequest>(bare).is_err());
}

#[test]
fn study_definition_historical_readmission_codec_unit() {
    let hash = pse_ids::ContentHash::from_bytes([4; 32]);
    let binding = AdmittedBinding {
        revision: hash.into(),
        context: hash,
        entries: std::collections::BTreeMap::new(),
    };
    let requested = point(
        pse_model::generated::identities::DeclarationId::from_bytes([5; 16]),
        7,
        vec![],
        StartPolicy::Fresh,
    );
    let definition = StudyDefinition {
        version: pse_model::document::Version,
        physical: hash,
        modeling: vec![pse_ids::ContentHash::from_bytes([5; 32])],
        points: vec![StudyPointDefinition {
            operation: StudyOperation {
                version: pse_model::document::Version,
                source: OperationSource {
                    revision: hash.into(),
                    physical_context: hash,
                },
                preparation: requested.preparation,
                operation: requested.operation,
                admitted_horizon: None,
            },
            binding_hash: binding.identity(),
            binding,
            policy: PointPolicy {
                key: OccurrenceKey(7),
                dependencies: vec![],
                seed_need: SeedNeed::NotNeeded,
                start: StartPolicy::Fresh,
                attempt_limit: 1,
            },
        }],
    };
    let current = serde_json::to_string(&definition).unwrap();
    assert_eq!(serde_json::to_value(&definition).unwrap()["version"], 6);
    let decoded = StudyDefinition::readmission(&current).unwrap();
    assert_eq!(serde_json::to_string(&decoded).unwrap(), current);

    // A historical envelope is refused even when its scientific layout cannot be read
    // by this build, and even when the version follows the nested document in the bytes.
    let mut former = serde_json::to_value(&definition).unwrap();
    former["version"] = serde_json::json!(3);
    former["points"][0]["operation"]["version"] = serde_json::json!(1);
    let compiler = former["points"][0]["operation"]["preparation"]["compiler"]
        .as_object_mut()
        .unwrap();
    compiler.remove("class_proof_work");
    compiler.remove("assembly");
    let historical = serde_json::to_string(&former).unwrap();
    let retained = historical.clone();
    let error = StudyDefinition::readmission(&historical).unwrap_err();
    let WorkflowError::Operations(pse_operations::OperationsError::InvalidRequest { reason }) =
        &error
    else {
        panic!("{error:?}")
    };
    assert!(reason.contains("study definition version 3 is unsupported"));
    assert!(reason.contains("explicit readmission is required"));
    assert_eq!(
        error.boundary_diagnostic().rule,
        pse_diagnostics::DiagnosticRule::WorkflowOperations
    );
    assert_eq!(historical, retained);
    for malformed in [r#"{}"#, r#"{"version":"3"}"#, r#"{"version":6}"#] {
        assert!(matches!(
            StudyDefinition::readmission(malformed),
            Err(WorkflowError::Input(_))
        ));
    }
}

#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn immutable_definition_equal_bindings_distinct_occurrences_and_usable_dependency_refusal() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = job_durable(&database, "worker", quick()).await;
    let directory = tempfile::tempdir().unwrap();
    let workspace = runtime
        .register_workspace(
            "studies",
            url::Url::from_directory_path(directory.path()).unwrap(),
        )
        .await
        .unwrap();
    let (sources, definition) = admitted(&runtime, |root, failed, _| {
        vec![
            point(root, 2, vec![], StartPolicy::Fresh),
            point(root, 4, vec![], StartPolicy::Fresh),
            point(failed, 8, vec![], StartPolicy::Fresh),
            point(
                root,
                12,
                vec![Dependency::UsableResult(OccurrenceKey(8))],
                StartPolicy::Fresh,
            ),
        ]
    })
    .await;
    assert_eq!(
        definition.points[0].binding_hash,
        definition.points[1].binding_hash
    );
    let handle = runtime
        .start_defined_study(&workspace, sources, definition, RetryPolicy::ONCE, 0)
        .await
        .unwrap();
    while !matches!(runtime.work_once().await.unwrap(), Processed::Idle) {}
    let status = handle.status().await.unwrap();
    assert_eq!(status.state, StudyState::Published);
    assert_eq!(status.attempt_state, AttemptState::Partial);
    assert_eq!(status.points[3].state, StudyPointState::Failed);
    assert!(
        status.points[3]
            .outcome
            .as_ref()
            .unwrap()
            .diagnostic
            .is_some()
    );
    assert_ne!(status.points[3].state, StudyPointState::Cancelled);
    let published = handle.wait(Duration::from_millis(10)).await.unwrap();
    assert_eq!(published.attempt_id, status.attempt_id);
    drop(runtime);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn cancellation_preserves_one_outcome_for_every_unattempted_occurrence() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = job_durable(&database, "worker", quick()).await;
    let directory = tempfile::tempdir().unwrap();
    let workspace = runtime
        .register_workspace(
            "cancelled",
            url::Url::from_directory_path(directory.path()).unwrap(),
        )
        .await
        .unwrap();
    let (sources, definition) = admitted(&runtime, |_, _, fixed| {
        vec![
            point(fixed, 3, vec![], StartPolicy::Fresh),
            point(
                fixed,
                9,
                vec![Dependency::Ordering(OccurrenceKey(3))],
                StartPolicy::Fresh,
            ),
        ]
    })
    .await;
    let handle = runtime
        .start_defined_study(&workspace, sources, definition, RetryPolicy::ONCE, 0)
        .await
        .unwrap();
    assert!(handle.cancel().await.unwrap().concluded);
    let status = handle.status().await.unwrap();
    assert_eq!(status.points.len(), 2);
    assert_eq!(status.attempt_state, AttemptState::Cancelled);
    assert!(status.points.iter().all(|point| {
        point.state == StudyPointState::Cancelled
            && point
                .outcome
                .as_ref()
                .is_some_and(|outcome| outcome.attempts.is_empty() && !outcome.scientific.usable)
    }));
    drop(runtime);
    database.remove().await.unwrap();
}

#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn mixed_durable_study_returns_to_binding_recovers_lease_and_records_seed_fallback_without_fabricated_results()
 {
    let database = TestDatabase::create().await.unwrap();
    let runtime = job_durable(&database, "restarted", quick()).await;
    let directory = tempfile::tempdir().unwrap();
    let workspace = runtime
        .register_workspace(
            "mixed",
            url::Url::from_directory_path(directory.path()).unwrap(),
        )
        .await
        .unwrap();
    let (physical, modeling) = sources(CASES);
    let cancel = crate::CancelSource::new();
    let package = runtime
        .package_from_sources(
            std::slice::from_ref(&modeling),
            runtime
                .physical_from_sources(&physical, &cancel)
                .await
                .unwrap(),
        )
        .unwrap();
    let find = |name: &str| {
        package
            .declarations()
            .iter()
            .find(|r| r.name == name)
            .unwrap()
            .declaration_id
    };
    let root = find("Root");
    let scalar = package.quantities.neutral_dimensionless().unwrap();
    let unit = package
        .quantities
        .quantity_type(scalar)
        .unwrap()
        .canonical_unit;
    let continuation = |predecessor, unavailable| {
        StartPolicy::Continuation(SeedEdge {
            predecessor: OccurrenceKey(predecessor),
            role: SeedRole::PrimalSolution,
            permission: ContinuationPermission::RequireUsable,
            unavailable,
        })
    };
    let binding = |mut point: StudyPoint, value| {
        point.overlay.assignments.push(BindingAssignment {
            target: BindingTarget::Path("a".into()),
            value: BindingQuantity {
                magnitude: pse_model::scalars::FiniteBound::try_new(value).unwrap(),
                quantity: scalar.as_id(),
                unit: unit.as_id(),
            },
        });
        point
    };
    let mut first = binding(point(root, 2, vec![], StartPolicy::Fresh), 4.);
    first.policy.attempt_limit = 2;
    let simulation = StudyPoint {
        operation: OperationRequest::Simulation(SimulationOperation {
            case: find("dynamic"),
            profile: None,
        }),
        ..point(
            root,
            12,
            vec![Dependency::Ordering(OccurrenceKey(10))],
            StartPolicy::Fresh,
        )
    };
    let points = vec![
        first,
        binding(
            point(
                root,
                4,
                vec![],
                continuation(2, UnavailableSeedPolicy::Refuse),
            ),
            9.,
        ),
        binding(
            point(
                root,
                6,
                vec![],
                continuation(4, UnavailableSeedPolicy::Refuse),
            ),
            4.,
        ),
        point(
            root,
            8,
            vec![Dependency::Ordering(OccurrenceKey(6))],
            StartPolicy::Explicit {
                role: SeedRole::PrimalSolution,
                seed: pse_operations::mint_id(),
            },
        ),
        point(
            root,
            10,
            vec![],
            continuation(8, UnavailableSeedPolicy::FreshOnUnavailable),
        ),
        simulation,
        point(
            root,
            14,
            vec![Dependency::UsableResult(OccurrenceKey(8))],
            StartPolicy::Fresh,
        ),
        point(
            root,
            16,
            vec![Dependency::Ordering(OccurrenceKey(12))],
            StartPolicy::Fresh,
        ),
    ];
    let definition = package
        .admit_study_points(
            crate::authoring_driver::document::package_checksum(&physical),
            vec![crate::authoring_driver::document::package_checksum(
                &modeling,
            )],
            &points,
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(
        definition.points[0].binding_hash,
        definition.points[2].binding_hash
    );
    assert_ne!(
        definition.points[0].binding_hash,
        definition.points[1].binding_hash
    );
    assert_eq!(definition.points[5].policy.seed_need, SeedNeed::NotNeeded);
    let handle = runtime
        .start_defined_study(
            &workspace,
            PackageSources {
                physical,
                modeling: vec![modeling],
            },
            definition,
            RetryPolicy::ONCE,
            0,
        )
        .await
        .unwrap();
    let Durability::Durable(operations) = runtime.durability() else {
        panic!("durable study")
    };
    // Simulate a process dying after a claim, before dispatch. Recovery must keep
    // occurrence/run identity and add an attempt, without inventing a native result.
    let crashed = operations
        .store()
        .jobs()
        .claim("crashed", Duration::from_millis(1))
        .await
        .unwrap()
        .unwrap();
    let before = operations
        .store()
        .studies()
        .get(handle.study_id())
        .await
        .unwrap();
    assert_eq!(
        before
            .points
            .iter()
            .find(|p| p.job_id == crashed.job_id)
            .unwrap()
            .point_index,
        2
    );
    let cancelled = before
        .points
        .iter()
        .find(|p| p.point_index == 16)
        .unwrap()
        .attempt_id;
    assert_eq!(
        operations
            .store()
            .request_cancel(cancelled, "mixed-study-control")
            .await
            .unwrap(),
        pse_operations::cancellation::CancelOutcome::CancelledBeforeStart
    );
    tokio::time::sleep(Duration::from_millis(20)).await;
    let recovered = operations.recover().await.unwrap();
    assert!(
        recovered
            .requeued
            .iter()
            .any(|r| r.stale_attempt == crashed.attempt_id),
        "expired occurrence was not recovered: {recovered:?}"
    );
    let deadline = tokio::time::Instant::now() + Duration::from_secs(120);
    let mut native_roots = std::collections::BTreeMap::new();
    let mut native_runs = std::collections::BTreeMap::new();
    let mut native_simulation = false;
    loop {
        let (processed, result) = runtime.work_once_with_result().await.unwrap();
        if let Processed::Ran { record, .. } = &processed {
            assert!(
                record.attempt.is_ok(),
                "durable occurrence did not record termination: {:?}",
                record.attempt
            );
        }
        if let (Processed::Ran { job, .. }, Some(result)) = (&processed, result) {
            let occurrence = operations
                .store()
                .studies()
                .point_of_job(*job)
                .await
                .unwrap()
                .unwrap()
                .point_index;
            native_runs.insert(occurrence, result.run_id);
            match result.report().unwrap() {
                RunReport::Modeling(results) => {
                    let x = results[0]
                        .reports
                        .iter()
                        .find(|r| r.label == "root")
                        .unwrap()
                        .value;
                    native_roots.insert(occurrence, x);
                }
                RunReport::Simulation(trajectory) => {
                    assert!(trajectory.accepted());
                    assert!(
                        (trajectory.report().samples.last().unwrap().state[0] - 2.).abs() < 1e-6
                    );
                    native_simulation = true;
                }
                other => panic!("unexpected mixed-study operation: {other:?}"),
            }
        }
        let status = handle.status().await.unwrap();
        if status.state == StudyState::Published {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "mixed study did not settle: {status:#?}"
        );
        if matches!(processed, Processed::Idle) {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
    let status = handle.status().await.unwrap();
    for (key, x) in [(2, 2.), (4, 3.), (6, 2.), (10, 2.)] {
        let actual = native_roots
            .get(&key)
            .unwrap_or_else(|| panic!("occurrence {key} produced no native root: {status:#?}"));
        assert!(
            (*actual - x).abs() < 1e-6,
            "return path solved the wrong binding"
        );
    }
    assert!(native_simulation);
    assert_ne!(
        native_runs[&2], native_runs[&6],
        "equal bindings retain separate experiment runs"
    );
    assert_eq!(
        native_runs[&2], crashed.run_id,
        "recovery retains the interrupted occurrence run"
    );
    assert_eq!(status.state, StudyState::Published);
    assert_eq!(status.attempt_state, AttemptState::Partial);
    for index in [0, 1, 2, 4, 5] {
        assert_eq!(
            status.points[index].state,
            StudyPointState::Completed,
            "{status:?}"
        );
        assert!(
            status.points[index]
                .outcome
                .as_ref()
                .unwrap()
                .scientific
                .usable
        );
    }
    for (index, predecessor) in [(1, 2), (2, 4)] {
        assert!(
            matches!(status.points[index].outcome.as_ref().unwrap().start,
            Some(StartProvenance::Continuation {predecessor: key, ..}) if key == OccurrenceKey(predecessor))
        );
    }
    assert!(matches!(
        status.points[4].outcome.as_ref().unwrap().start,
        Some(StartProvenance::FreshFallback {
            predecessor: OccurrenceKey(8),
            reason: SeedUnavailable::Absent,
            ..
        })
    ));
    assert_eq!(
        status.points[5].outcome.as_ref().unwrap().start,
        Some(StartProvenance::NotNeeded)
    );
    assert_eq!(status.points[7].state, StudyPointState::Cancelled);
    assert!(!status.points[7].outcome.as_ref().unwrap().scientific.usable);
    for index in [3, 6] {
        let outcome = status.points[index].outcome.as_ref().unwrap();
        assert_eq!(status.points[index].state, StudyPointState::Failed);
        assert!(outcome.diagnostic.is_some());
        assert!(!outcome.scientific.usable);
    }
    let settled = operations
        .store()
        .studies()
        .get(handle.study_id())
        .await
        .unwrap();
    let retried = &settled.points[0];
    assert_ne!(retried.attempt_id, crashed.attempt_id);
    let attempt = operations
        .store()
        .attempts()
        .get(retried.attempt_id)
        .await
        .unwrap();
    assert_eq!(attempt.parent_attempt, Some(crashed.attempt_id));
    assert_eq!(attempt.run_id, crashed.run_id);
    assert_ne!(settled.points[0].job_id, settled.points[2].job_id);
    let members = operations
        .store()
        .studies()
        .available_members(handle.study_id())
        .await
        .unwrap();
    assert!(
        members.iter().all(|(key, _)| ![8, 14, 16].contains(key)),
        "pre-result refusal or cancellation fabricated members"
    );
    let published = handle.wait(Duration::from_millis(10)).await.unwrap();
    assert_eq!(published.attempt_id, status.attempt_id);
    assert!(matches!(
        runtime.work_once().await.unwrap(),
        Processed::Idle
    ));
    let after = operations
        .store()
        .studies()
        .available_members(handle.study_id())
        .await
        .unwrap();
    assert_eq!(
        after.len(),
        members.len(),
        "settled occurrences must not publish twice"
    );
    assert_eq!(handle.status().await.unwrap().points.len(), points.len());
    drop((handle, package, runtime));
    database.remove().await.unwrap();
}

#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn related_case_study_uses_secant_then_original_correction() {
    let runtime = tests::runtime_with_workspace(64 << 20);
    let (physical, modeling) = sources(CASES);
    let cancel = crate::CancelSource::new();
    let package = runtime
        .package_from_sources(
            std::slice::from_ref(&modeling),
            runtime
                .physical_from_sources(&physical, &cancel)
                .await
                .unwrap(),
        )
        .unwrap();
    let root = package
        .declarations()
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let scalar = package.quantities.neutral_dimensionless().unwrap();
    let unit = package
        .quantities
        .quantity_type(scalar)
        .unwrap()
        .canonical_unit;
    let mut points = Vec::new();
    for (index, value) in [4., 4.04, 4.08].into_iter().enumerate() {
        let start = if index == 0 {
            StartPolicy::Fresh
        } else {
            StartPolicy::Continuation(SeedEdge {
                predecessor: OccurrenceKey(index as u32),
                role: SeedRole::PrimalSolution,
                permission: ContinuationPermission::RequireUsable,
                unavailable: UnavailableSeedPolicy::Refuse,
            })
        };
        let mut next = point(root, index as u32 + 1, vec![], start);
        let OperationRequest::DeclaredCase(operation) = &mut next.operation else {
            panic!("case expected");
        };
        operation.settings.intent = pse_backend_native::solve::SolveIntent::Root;
        operation.settings.controls.start =
            pse_backend_native::solve::StartPolicy::PreviousAccepted;
        operation
            .settings
            .composition
            .recovery
            .push(pse_model::strategy::StartOrigin::Predicted);
        next.overlay.assignments.push(BindingAssignment {
            target: BindingTarget::Path("a".into()),
            value: BindingQuantity {
                magnitude: pse_model::scalars::FiniteBound::try_new(value).unwrap(),
                quantity: scalar.as_id(),
                unit: unit.as_id(),
            },
        });
        points.push(next);
    }
    let definition = package
        .admit_study_points(
            crate::authoring_driver::document::package_checksum(&physical),
            vec![crate::authoring_driver::document::package_checksum(
                &modeling,
            )],
            &points,
            &cancel,
        )
        .await
        .unwrap();
    let study = package.study(&definition, 3, &cancel).await.unwrap();
    assert!(
        study
            .outcomes
            .iter()
            .all(|outcome| outcome.scientific.usable)
    );
    let result = |index: usize| {
        let RunReport::Modeling(results) = study.results[index].as_ref().unwrap().report().unwrap()
        else {
            panic!("modeling expected");
        };
        results[0].clone()
    };
    let first = result(0);
    let second = result(1);
    let third = result(2);
    assert!(
        second.root_predictor().is_err(),
        "no sensitivity/factor was requested"
    );
    let execution = pse_backend_native::solve::Execution::new(
        Arc::default(),
        &pse_backend_native::solve::Controls::default(),
    );
    let proposal = second
        .secant_prediction(
            &first,
            &third.prepared,
            pse_model::strategy::BranchPolicy::any_qualified(),
            &execution,
        )
        .unwrap();
    assert_eq!(
        third.prepared.solve.source_start(None).unwrap(),
        proposal
            .values()
            .map(|(_, value)| value)
            .collect::<Vec<_>>(),
        "actual third occurrence consumed the shared secant endpoint"
    );
    assert_eq!(
        third.prepared.solve.numerical_strategy().start.policy,
        pse_backend_native::solve::StartPolicy::PreviousAccepted
    );
    assert_eq!(
        third.prepared.solve.entry_origin(false),
        pse_model::strategy::StartOrigin::Predicted
    );
    assert_eq!(
        third.strategy.as_ref().unwrap().starts[0],
        pse_model::strategy::StartOrigin::Predicted
    );
    let crate::math::solves::Outcome::Native(report) = &third.outcome else {
        panic!("original native correction expected");
    };
    assert!((report.candidate.as_ref().unwrap().primal[0] - 4.08_f64.sqrt()).abs() < 1e-8);
    assert_eq!(study.outcomes[2].key, OccurrenceKey(3));
}
