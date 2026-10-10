// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable occurrence contracts and canonical dependency-scoped execution.
#[cfg(feature = "canonical-tests")]
use super::durable_tests::durable_runtime;
use super::worker_tests::sources;
use super::*;
use pse_model::study::*;
#[cfg(feature = "canonical-tests")]
use pse_operations::canonical_studies::StudySummary;
#[cfg(any(feature = "canonical-tests", feature = "solver-ipopt"))]
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
            // Independent KKT budgets are explicit; root correspondence uses target requirements.
            settings: crate::math::settings::SolveSettings {
                numerics: pse_model::numerics::NumericalPolicy {
                    kkt: pse_model::numerics::KktTolerances {
                        stationarity: 1e-10,
                        complementarity: 1e-10,
                    },
                    ..Default::default()
                },
                backend: Some(pse_backend_native::solve::Backend::Ipopt),
                intent: pse_backend_native::solve::SolveIntent::Root,
                ..Default::default()
            },
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
#[cfg(feature = "canonical-tests")]
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
            &cancel,
        )
        .await
        .unwrap();
    let declarations = package.declarations().await.unwrap();
    let find = |name: &str| {
        declarations
            .iter()
            .find(|declaration| declaration.name == name)
            .unwrap()
            .declaration_id
    };
    let points = points(find("Root"), find("Failed"), find("Fixed"));
    let definition = package
        .admit_study_sources(&physical, &points, &cancel)
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
        physical: PhysicalSource {
            revision: "fixture-physical".into(),
            identity: hash,
        },
        modeling_revision: "fixture-revision".into(),
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
    assert_eq!(serde_json::to_value(&definition).unwrap()["version"], 8);
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
    assert!(
        error
            .to_string()
            .contains("study definition version 3 is unsupported")
    );
    assert!(
        error
            .to_string()
            .contains("explicit readmission is required")
    );
    assert_eq!(historical, retained);
    for malformed in [r#"{}"#, r#"{"version":"3"}"#, r#"{"version":8}"#] {
        assert!(matches!(
            StudyDefinition::readmission(malformed),
            Err(WorkflowError::Input(_))
        ));
    }
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn canonical_study_cancel_preserves_distinct_unattempted_outcomes() {
    let runtime = durable_runtime();
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
    assert_eq!(
        definition.points[0].binding_hash,
        definition.points[1].binding_hash
    );
    let handle = runtime
        .start_defined_study(sources.physical, definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let receipt = handle.cancel().await.unwrap();
    assert!(!receipt.already_concluded);
    for _ in 0..4 {
        if matches!(runtime.work_once().await.unwrap(), Processed::Idle) {
            break;
        }
    }
    let status = handle.status().await.unwrap();
    assert_eq!(status.state, StudyState::Concluded);
    assert!(status.cancelled);
    assert_eq!(status.points.len(), 2);
    assert_ne!(status.points[0].run, status.points[1].run);
    assert!(status.points.iter().all(|point| {
        point.settled
            && point.state == StudyPointState::Cancelled
            && point.attempt.is_none()
            && point
                .outcome
                .as_ref()
                .is_some_and(|outcome| outcome.attempts.is_empty() && !outcome.scientific.usable)
    }));
    let retained = handle.result().await.unwrap().unwrap();
    let run = runtime
        .canonical_store()
        .canonical_run(&retained.run)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(run.terminal_class.as_deref(), Some("cancelled"));
    let restarted = runtime.study(handle.study_id());
    assert_eq!(
        restarted.result().await.unwrap().unwrap().attempt,
        retained.attempt
    );
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn canonical_study_plan_admits_physical_once_before_ready_occurrences() {
    let runtime = durable_runtime();
    let mut authored = None;
    let (sources, _) = admitted(&runtime, |_, _, fixed| {
        let points = vec![point(fixed, 3, vec![], StartPolicy::Fresh)];
        authored = Some(points.clone());
        points
    })
    .await;
    let before = pse_engine::cache_service::CacheComponent::report(runtime.physical_cache.as_ref());
    let handle = runtime
        .start_study(
            StudyPlan {
                sources,
                points: authored.unwrap(),
            },
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let after = pse_engine::cache_service::CacheComponent::report(runtime.physical_cache.as_ref());
    assert_eq!(
        after[0].misses,
        before[0].misses + 1,
        "original document ingress admits one exact canonical physical receipt"
    );
    assert_eq!(
        after[0].hits,
        before[0].hits + 1,
        "definition admission reuses the ingress owner"
    );
    assert!(matches!(
        runtime.work_once().await.unwrap(),
        Processed::Ran { .. }
    ));
    let status = handle.status().await.unwrap();
    assert!(status.points[0].outcome.as_ref().unwrap().scientific.usable);
    let after = pse_engine::cache_service::CacheComponent::report(runtime.physical_cache.as_ref());
    assert_eq!(
        after[0].misses,
        before[0].misses + 1,
        "ready preparation does not reparse or re-admit physical documents"
    );
    assert_eq!(after[0].hits, before[0].hits + 2);
}

#[cfg(all(feature = "canonical-tests", feature = "native-solvers"))]
#[tokio::test]
async fn canonical_study_equal_bindings_and_failed_usable_dependency() {
    let runtime = durable_runtime();
    let (mut sources, definition) = admitted(&runtime, |root, failed, _| {
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
    sources.modeling.clear();
    let handle = runtime
        .start_defined_study(sources.physical, definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let before =
        pse_engine::cache_service::CacheComponent::report(runtime.physical_cache.as_ref())[0].hits;
    for _ in 0..8 {
        if matches!(runtime.work_once().await.unwrap(), Processed::Idle) {
            break;
        }
    }
    let status = handle.status().await.unwrap();
    assert_eq!(status.state, StudyState::Concluded);
    assert_eq!(status.points[3].state, StudyPointState::Failed);
    assert!(status.points[3].attempt.is_none());
    assert!(status.points[0].outcome.as_ref().unwrap().scientific.usable);
    assert!(status.points[1].outcome.as_ref().unwrap().scientific.usable);
    let retained = handle.result().await.unwrap().unwrap();
    let run = runtime
        .canonical_store()
        .canonical_run(&retained.run)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(run.terminal_class.as_deref(), Some("partial"));
    assert_ne!(status.points[0].attempt, status.points[1].attempt);
    let cache = pse_engine::cache_service::CacheComponent::report(runtime.physical_cache.as_ref());
    assert!(
        cache[0].hits >= before + 3,
        "each ready scientific occurrence reuses the original canonical physical admission"
    );
    assert!(
        cache[0].entries <= 2,
        "physical admissions and IPC receipts each retain at most one exact owner"
    );
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn canonical_study_expired_claim_recovers_without_scientific_rerun() {
    let runtime = durable_runtime();
    let (sources, mut definition) = admitted(&runtime, |_, _, fixed| {
        vec![point(fixed, 5, vec![], StartPolicy::Fresh)]
    })
    .await;
    definition.points[0].policy.attempt_limit = 1;
    let handle = runtime
        .start_defined_study(sources.physical, definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let key = pse_operations::canonical_studies::point_key(
        &handle.study_id().to_string(),
        OccurrenceKey(5),
    );
    let scope = runtime.canonical_store().study_scope(&key).await.unwrap();
    let claim = runtime
        .canonical_store()
        .claim_study_point(
            &scope,
            None,
            "expired-fixture",
            "departed-worker",
            Duration::from_secs(60),
        )
        .await
        .unwrap();
    pse_operations::testing::expire_acknowledged_attempt(runtime.canonical_store(), &claim.fence)
        .await
        .unwrap();
    let point = runtime
        .canonical_store()
        .canonical_study_point(&key)
        .await
        .unwrap()
        .unwrap();
    assert!(runtime.recover_study_point(&point.key).await.unwrap());
    let actual = runtime
        .canonical_store()
        .canonical_attempt(claim.fence.attempt())
        .await
        .unwrap()
        .unwrap();
    assert!(actual.terminal);
    assert_eq!(actual.outcome.as_deref(), Some("failed"));
    let after = runtime
        .canonical_store()
        .canonical_study_point(&key)
        .await
        .unwrap()
        .unwrap();
    assert!(after.settled);
    let facts = pse_operations::canonical_studies::point_facts(&after).unwrap();
    assert!(!facts.native_started && !facts.scientific.usable);
    assert_eq!(facts.attempt_count, 1);
}

#[tokio::test]
async fn related_case_study_uses_secant_then_original_correction() {
    let runtime = tests::runtime_with_workspace(64 << 20);
    let (package, definition) = related_definition(&runtime).await;
    let study = package
        .study(&definition, 3, &crate::CancelSource::new())
        .await
        .unwrap();
    assert!(
        study
            .outcomes
            .iter()
            .all(|outcome| outcome.scientific.usable),
        "{:?}",
        study.outcomes
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

async fn related_definition(runtime: &Runtime) -> (ModelingPackage, StudyDefinition) {
    let (physical, modeling) = sources(CASES);
    let cancel = crate::CancelSource::new();
    let package = runtime
        .package_from_sources(
            std::slice::from_ref(&modeling),
            runtime
                .physical_from_sources(&physical, &cancel)
                .await
                .unwrap(),
            &cancel,
        )
        .await
        .unwrap();
    let root = package
        .declarations()
        .await
        .unwrap()
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
        .admit_study_sources(&physical, &points, &cancel)
        .await
        .unwrap();
    (package, definition)
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn managed_primary_reopened_continuation_uses_shared_secant_and_original_correction() {
    use pse_relations::generated::runtime::{solve_strategy_events, solve_variables};
    let fixture = ManagedStudyFixture::start(false).await;
    let runtime = fixture.runtime.clone();
    let operations = runtime.operations().unwrap().clone();
    let (package, definition) = related_definition(&runtime).await;
    let cancel = crate::CancelSource::new();
    let study = fixture
        .dispatch(&cancel, package.study(&definition, 3, &cancel))
        .await
        .unwrap();
    assert!(
        study
            .outcomes
            .iter()
            .all(|outcome| outcome.scientific.usable),
        "{:?}",
        study.outcomes
    );
    let third = study.results[2].as_ref().unwrap();
    let (run, attempt) = third.stored_keys().unwrap();
    let reopened = operations.record(run, attempt).await.unwrap();
    assert_eq!(reopened.solutions.len(), 1);
    let mut events = runtime
        .results(
            run,
            attempt,
            "runtime.solve_strategy_events",
            0,
            u64::MAX,
            pse_columnar::CancellationToken::new(),
        )
        .await
        .unwrap();
    let mut predicted = false;
    while let Some(batch) = events.next_batch().await.unwrap() {
        let checked = solve_strategy_events::View::try_from_batch_with_registry(
            &runtime.registry,
            &batch,
            &runtime.validation_context().unwrap(),
        )
        .unwrap();
        for row in checked.rows().unwrap() {
            predicted |= row.start_origin
                == Some(pse_model::generated::enums::NumericalStartOrigin::Predicted);
        }
    }
    assert!(
        predicted,
        "retained third occurrence must consume the shared secant producer"
    );
    let mut variables = runtime
        .results(
            run,
            attempt,
            "runtime.solve_variables",
            0,
            u64::MAX,
            pse_columnar::CancellationToken::new(),
        )
        .await
        .unwrap();
    let mut corrected = false;
    while let Some(batch) = variables.next_batch().await.unwrap() {
        let checked = solve_variables::View::try_from_batch_with_registry(
            &runtime.registry,
            &batch,
            &runtime.validation_context().unwrap(),
        )
        .unwrap();
        for row in checked.rows().unwrap() {
            corrected |= row
                .value
                .is_some_and(|value| (value - 4.08_f64.sqrt()).abs() < 1e-8);
        }
    }
    assert!(
        corrected,
        "retained original correction must solve actual target equation"
    );
    drop(events);
    drop(variables);
    fixture.finish().await;
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn canonical_study_explicit_missing_seed_refuses_without_native_attempt() {
    let runtime = durable_runtime();
    let (package, mut definition) = related_definition(&runtime).await;
    definition.points.truncate(1);
    definition.points[0].policy.start = StartPolicy::Explicit {
        role: SeedRole::PrimalSolution,
        seed: pse_operations::mint_id(),
    };
    let (physical, _) = sources(CASES);
    let handle = runtime
        .start_defined_study(physical, definition, &crate::CancelSource::new())
        .await
        .unwrap();
    loop {
        if matches!(runtime.work_once().await.unwrap(), Processed::Idle) {
            break;
        }
    }
    let status = handle.status().await.unwrap();
    let point = &status.points[0];
    assert!(point.settled);
    assert!(point.attempt.is_none());
    let outcome = point.outcome.as_ref().unwrap();
    assert!(!outcome.scientific.usable);
    assert_eq!(
        outcome.diagnostic.as_ref().unwrap().rule,
        pse_diagnostics::DiagnosticRule::StudySeedUnavailable
    );
    assert!(
        runtime
            .canonical_store()
            .canonical_run(&point.run)
            .await
            .unwrap()
            .unwrap()
            .current_attempt
            .is_none()
    );
    drop(package);
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn canonical_study_summary_live_owner_and_expired_writer_rebuild_without_science() {
    let runtime = durable_runtime();
    let (package, mut definition) = related_definition(&runtime).await;
    definition.points.truncate(1);
    let (physical, _) = sources(CASES);
    let handle = runtime
        .start_defined_study(physical, definition, &crate::CancelSource::new())
        .await
        .unwrap();
    assert!(
        handle.result().await.unwrap().is_none(),
        "open study has no sealed result"
    );
    assert!(matches!(
        runtime.work_once().await.unwrap(),
        Processed::Ran { .. }
    ));
    let key = handle.study_id().to_string();
    let store = runtime.canonical_store();
    let header = store.canonical_study(&key).await.unwrap().unwrap();
    assert!(!header.metadata.is_empty());
    let page = store.study_page(None).await.unwrap();
    let summary = page
        .iter()
        .find(|summary| summary.key == header.key)
        .unwrap();
    assert_eq!(*summary, StudySummary::from(&header));
    assert!(
        serde_json::to_value(summary)
            .unwrap()
            .get("metadata")
            .is_none()
    );
    let fence = store
        .begin_study_finalization(
            &StudySummary::from(&header),
            "crashed-summary",
            Duration::from_millis(250),
        )
        .await
        .unwrap()
        .unwrap();
    assert!(
        store
            .begin_study_finalization(
                &StudySummary::from(&header),
                "competing-summary",
                Duration::from_secs(1)
            )
            .await
            .unwrap()
            .is_none()
    );
    store
        .append_result_batch(
            &fence,
            "summary-partial",
            "__effect_free_summary",
            0,
            &[1],
            1,
        )
        .await
        .unwrap();
    let before = store
        .canonical_study_point(&pse_operations::canonical_studies::point_key(
            &key,
            OccurrenceKey(1),
        ))
        .await
        .unwrap()
        .unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(runtime.finalize_canonical_study(&key).await.unwrap());
    let after = store
        .canonical_study_point(&before.key)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        before, after,
        "summary rebuild must not execute or revise any scientific occurrence"
    );
    let status = handle.status().await.unwrap();
    assert_ne!(status.result_attempt.as_deref(), Some(fence.attempt()));
    let previous = store
        .canonical_attempt(fence.attempt())
        .await
        .unwrap()
        .unwrap();
    assert!(previous.terminal);
    assert_eq!(previous.outcome.as_deref(), Some("failed"));
    let final_attempt = store
        .canonical_attempt(status.result_attempt.as_ref().unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(final_attempt.outcome.as_deref(), Some("succeeded"));
    let results = handle.result().await.unwrap().unwrap();
    assert_eq!(results.attempt, status.result_attempt.unwrap());
    assert_eq!(results.points.len(), 1);
    drop(package);
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[allow(
    unsafe_code,
    reason = "fixture asserts rejected stale success only; actual summary rebuilt by owning runtime"
)]
#[tokio::test]
async fn canonical_study_cancellation_after_summary_close_fences_success() {
    let runtime = durable_runtime();
    let (_, mut definition) = related_definition(&runtime).await;
    definition.points.truncate(1);
    let (physical, _) = sources(CASES);
    let handle = runtime
        .start_defined_study(physical, definition, &crate::CancelSource::new())
        .await
        .unwrap();
    assert!(matches!(
        runtime.work_once().await.unwrap(),
        Processed::Ran { .. }
    ));
    let key = handle.study_id().to_string();
    let store = runtime.canonical_store();
    let study = store.canonical_study(&key).await.unwrap().unwrap();
    let fence = store
        .begin_study_finalization(
            &StudySummary::from(&study),
            "summary-before-cancel",
            Duration::from_millis(300),
        )
        .await
        .unwrap()
        .unwrap();
    let closed = store
        .close_result_ingestion(&fence, "summary-before-cancel-close")
        .await
        .unwrap();
    let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
    handle.cancel().await.unwrap();
    assert!(
        // SAFETY: this isolated negative fixture deliberately presents a stale success;
        // cancellation must fence it before any scientific publication can be admitted.
        unsafe {
            store
                .seal_attempt(
                    &manifest,
                    "stale-parent-success",
                    pse_operations::canonical_execution::TerminalClass::Succeeded,
                    &[1],
                )
                .await
        }
        .is_err()
    );
    assert!(
        // SAFETY: the same cancelled summary fixture must refuse the stale owner's
        // success assertion before admitting a study conclusion or exposing results.
        unsafe {
            store
                .seal_study_summary(
                    &key,
                    &manifest,
                    "stale-owned-summary",
                    pse_operations::canonical_execution::TerminalClass::Succeeded,
                    &[1],
                )
                .await
        }
        .is_err()
    );
    tokio::time::sleep(Duration::from_millis(350)).await;
    assert!(runtime.finalize_canonical_study(&key).await.unwrap());
    let status = handle.status().await.unwrap();
    assert_eq!(status.state, StudyState::Concluded);
    assert!(status.cancelled);
    let parent = store
        .canonical_attempt(status.result_attempt.as_ref().unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(parent.outcome.as_deref(), Some("cancelled"));
    assert!(handle.cancel().await.unwrap().already_concluded);
}

#[tokio::test]
async fn study_creation_acquisition_observes_caller_cancellation() {
    let cancel = crate::CancelSource::new();
    let polled = std::cell::Cell::new(false);
    let result = study::creation_input(&cancel, async {
        polled.set(true);
        cancel.cancel();
        std::future::pending::<Result<(), WorkflowError>>().await
    })
    .await;
    assert!(polled.get(), "cancellation was raised during acquisition");
    assert_eq!(
        result.unwrap_err().boundary_diagnostic().code,
        pse_diagnostics::DiagnosticCode::RuntimeCancelled
    );
    let polled = std::cell::Cell::new(false);
    let result = study::creation_input(&cancel, async {
        polled.set(true);
        Ok::<_, WorkflowError>(())
    })
    .await;
    assert!(!polled.get(), "pre-cancelled acquisition never starts");
    assert_eq!(
        result.unwrap_err().boundary_diagnostic().code,
        pse_diagnostics::DiagnosticCode::RuntimeCancelled
    );
}

#[cfg(feature = "canonical-tests")]
async fn stored_value_definition(runtime: &Runtime, count: u32) -> StudyDefinition {
    let (_, admitted) = related_definition(runtime).await;
    let first = &admitted.points[0];
    StudyDefinition {
        points: (0..count)
            .map(|index| {
                let mut next = first.clone();
                next.policy.key = OccurrenceKey(index);
                next.policy.start = StartPolicy::Fresh;
                next.policy.dependencies.clear();
                next.binding.entries.values_mut().next().unwrap().canonical =
                    pse_model::scalars::FiniteBound::try_new(4.0 + f64::from(index % 20) * 0.01)
                        .unwrap();
                next.binding_hash = next.binding.identity();
                next
            })
            .collect(),
        ..admitted
    }
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn stored_study_creation_reuses_one_basis_for_thousand_value_occurrences() {
    let runtime = durable_runtime();
    let definition = stored_value_definition(&runtime, 1000).await;
    assert_eq!(
        definition.points[0].binding_hash,
        definition.points[20].binding_hash
    );
    let (physical, _) = sources(CASES);
    let (handle, constructions) = study_operations::counted_declared_admissions(
        runtime.start_defined_study(physical, definition, &crate::CancelSource::new()),
    )
    .await;
    let handle = handle.unwrap();
    assert_eq!(
        constructions, 1,
        "actual stored readmission retains one declared basis"
    );
    let key = handle.study_id().to_string();
    let header = runtime
        .canonical_store()
        .canonical_study(&key)
        .await
        .unwrap()
        .unwrap();
    assert!(header.active);
    assert_eq!(header.point_count, 1000);
    assert_eq!(header.next_ordinal, 1000);
    let first = runtime
        .canonical_store()
        .canonical_study_point(&pse_operations::canonical_studies::point_key(
            &key,
            OccurrenceKey(0),
        ))
        .await
        .unwrap()
        .unwrap();
    let repeated = runtime
        .canonical_store()
        .canonical_study_point(&pse_operations::canonical_studies::point_key(
            &key,
            OccurrenceKey(20),
        ))
        .await
        .unwrap()
        .unwrap();
    let last = runtime
        .canonical_store()
        .canonical_study_point(&pse_operations::canonical_studies::point_key(
            &key,
            OccurrenceKey(999),
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first.descriptor, repeated.descriptor);
    assert_ne!(first.key, repeated.key);
    assert_ne!(first.run, repeated.run);
    assert_eq!(last.ordinal, 999);
    assert!(!first.assigned && !repeated.assigned && !last.assigned);
    handle.cancel().await.unwrap();
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn stored_study_creation_rechecks_later_binding_source_and_seed_policy() {
    let runtime = durable_runtime();
    let original = stored_value_definition(&runtime, 2).await;
    for fault in 0..5 {
        let mut definition = original.clone();
        let next = &mut definition.points[1];
        match fault {
            0 => next.binding.entries.values_mut().next().unwrap().parameter = false,
            1 => next.binding.context = pse_ids::ContentHash::from_bytes([7; 32]),
            2 => next.operation.source.physical_context = pse_ids::ContentHash::from_bytes([7; 32]),
            3 => next.policy.seed_need = SeedNeed::NotNeeded,
            _ => {
                next.binding.entries.values_mut().next().unwrap().quantity =
                    pse_ids::SemanticId::NIL
            }
        }
        next.binding_hash = next.binding.identity();
        let (physical, _) = sources(CASES);
        let (result, constructions) = study_operations::counted_declared_admissions(
            runtime.start_defined_study(physical, definition, &crate::CancelSource::new()),
        )
        .await;
        assert!(
            result.is_err(),
            "later fault {fault} must be independently refused"
        );
        assert_eq!(
            constructions, 1,
            "fault {fault} cannot inherit admission from point zero"
        );
    }
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn stored_study_creation_replaces_basis_for_changed_descriptor_and_target_layout() {
    let runtime = durable_runtime();
    let mut definition = stored_value_definition(&runtime, 4).await;
    definition.points[1]
        .operation
        .preparation
        .compiler
        .class_proof_work += 1;
    definition.points[2].binding.entries.clear();
    definition.points[2].binding_hash = definition.points[2].binding.identity();
    let (physical, _) = sources(CASES);
    let (result, constructions) = study_operations::counted_declared_admissions(
        runtime.start_defined_study(physical, definition, &crate::CancelSource::new()),
    )
    .await;
    let handle = result.unwrap();
    assert_eq!(
        constructions, 4,
        "each changed structural premise replaces the sole owner"
    );
    handle.cancel().await.unwrap();

    let mut definition = stored_value_definition(&runtime, 2).await;
    let OperationRequest::DeclaredCase(case) = &mut definition.points[1].operation.operation else {
        panic!("declared case expected");
    };
    case.settings.controls.threads = 0;
    let (physical, _) = sources(CASES);
    let (result, constructions) = study_operations::counted_declared_admissions(
        runtime.start_defined_study(physical, definition, &crate::CancelSource::new()),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(
        constructions, 2,
        "changed invalid settings replace then fail admission"
    );
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn default_durable_study_refuses_precancelled_creation() {
    let runtime = durable_runtime();
    let (package, definition) = related_definition(&runtime).await;
    let cancel = crate::CancelSource::new();
    cancel.cancel();
    let (result, constructions) =
        study_operations::counted_declared_admissions(package.study(&definition, 3, &cancel)).await;
    assert_eq!(
        result.unwrap_err().boundary_diagnostic().code,
        pse_diagnostics::DiagnosticCode::RuntimeCancelled
    );
    assert_eq!(
        constructions, 0,
        "default durable creation sees the caller's original scope"
    );
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn stored_study_creation_observes_cancellation_during_declared_admission() {
    let runtime = durable_runtime();
    let definition = stored_value_definition(&runtime, 1000).await;
    let cancel = crate::CancelSource::new();
    let (physical, _) = sources(CASES);
    let (result, constructions) = study_operations::counted_declared_admissions(
        study_operations::cancel_after_declared_admission(
            runtime.start_defined_study(physical, definition, &cancel),
            cancel.clone(),
        ),
    )
    .await;
    assert_eq!(
        result.unwrap_err().boundary_diagnostic().code,
        pse_diagnostics::DiagnosticCode::RuntimeCancelled
    );
    assert_eq!(
        constructions, 1,
        "cancellation stops after the first structural owner"
    );
}

#[tokio::test]
async fn study_creation_publication_awaits_issued_effect_before_cancellation() {
    let cancel = crate::CancelSource::new();
    let completed = std::cell::Cell::new(false);
    let result = study::creation_effect(&cancel, async {
        cancel.cancel();
        tokio::task::yield_now().await;
        completed.set(true);
        Ok::<_, WorkflowError>(())
    })
    .await;
    assert!(
        completed.get(),
        "issued publication future was completed through its owner"
    );
    assert_eq!(
        result.unwrap_err().boundary_diagnostic().code,
        pse_diagnostics::DiagnosticCode::RuntimeCancelled
    );
}

#[cfg(feature = "solver-ipopt")]
fn parallel_study_runtime() -> Runtime {
    study_runtime_with_population(16, 32)
}

#[cfg(feature = "solver-ipopt")]
fn study_runtime_with_population(threads: usize, jobs: usize) -> Runtime {
    let n = |value| std::num::NonZeroUsize::new(value).unwrap();
    let shared = SharedRuntime::build(crate::ResourceBudget {
        memory_limit_bytes: n(2 << 30),
        spill_dir: std::env::temp_dir(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: n(5),
        threads: pse_engine::ThreadBudget {
            pool_threads: n(threads),
            target_partitions: n(1),
        },
        execution: Default::default(),
        cache: crate::CacheBudget::disabled(1024),
        math: crate::math::MathPolicy {
            worker_bytes: 8 << 20,
            workspace_bytes: 64 << 20,
            foreign_bytes: 1 << 20,
            jobs,
            ..Default::default()
        },
        hashing_may_use_pool: false,
    })
    .unwrap();
    let registry = pse_schema::shared_registry().unwrap();
    let sessions = Arc::new(
        shared
            .session_factory(pse_engine::session::native_engine_profile())
            .unwrap(),
    );
    Runtime::from_shared(shared, registry, sessions, tests::canonical_deployment())
        .unwrap()
        .with_durability(DurabilitySelection::Ephemeral)
        .unwrap()
}

#[cfg(feature = "solver-ipopt")]
async fn parallel_study_definition(
    runtime: &Runtime,
    count: usize,
    failing: Option<usize>,
) -> (ModelingPackage, StudyDefinition) {
    let (physical, modeling) = sources(CASES);
    let cancel = crate::CancelSource::new();
    let package = runtime
        .package_from_sources(
            std::slice::from_ref(&modeling),
            runtime
                .physical_from_sources(&physical, &cancel)
                .await
                .unwrap(),
            &cancel,
        )
        .await
        .unwrap();
    let rows = package.declarations().await.unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let failed = rows
        .iter()
        .find(|row| row.name == "Failed")
        .unwrap()
        .declaration_id;
    let points = (0..count)
        .map(|index| {
            let mut point = point(
                if failing == Some(index) { failed } else { root },
                ((count - index) * 3) as u32,
                vec![],
                StartPolicy::Fresh,
            );
            let OperationRequest::DeclaredCase(case) = &mut point.operation else {
                panic!("case expected");
            };
            assert_eq!(
                case.settings.backend,
                Some(pse_backend_native::solve::Backend::Ipopt)
            );
            point
        })
        .collect::<Vec<_>>();
    let definition = package
        .admit_study_sources(&physical, &points, &cancel)
        .await
        .unwrap();
    (package, definition)
}

#[cfg(feature = "solver-ipopt")]
#[derive(Debug, Default)]
struct StudyNativeProbe {
    state: std::sync::Mutex<StudyNativeState>,
    released: std::sync::Condvar,
    changed: tokio::sync::Notify,
}
#[cfg(feature = "solver-ipopt")]
#[derive(Debug, Default)]
struct StudyNativeState {
    entered: Vec<std::thread::ThreadId>,
    released: std::collections::HashSet<std::thread::ThreadId>,
    release_all: bool,
    active: usize,
    maximum: usize,
}
#[cfg(feature = "solver-ipopt")]
impl StudyNativeProbe {
    fn enter(&self, cancel: &std::sync::atomic::AtomicBool) {
        let thread = std::thread::current().id();
        let mut state = self.state.lock().unwrap();
        if state.entered.contains(&thread) {
            return;
        }
        state.entered.push(thread);
        state.active += 1;
        state.maximum = state.maximum.max(state.active);
        self.changed.notify_waiters();
        let deadline = std::time::Instant::now() + Duration::from_secs(60);
        while !state.release_all
            && !state.released.contains(&thread)
            && !cancel.load(std::sync::atomic::Ordering::Acquire)
        {
            assert!(
                std::time::Instant::now() < deadline,
                "native study entry did not drain"
            );
            state = self
                .released
                .wait_timeout(state, Duration::from_millis(10))
                .unwrap()
                .0;
        }
        state.active -= 1;
        self.changed.notify_waiters();
    }
    async fn wait(&self, predicate: impl Fn(&StudyNativeState) -> bool) {
        tokio::time::timeout(Duration::from_secs(60), async {
            loop {
                let changed = self.changed.notified();
                if predicate(&self.state.lock().unwrap()) {
                    return;
                }
                changed.await;
            }
        })
        .await
        .unwrap();
    }
    fn release_following(&self) {
        let mut state = self.state.lock().unwrap();
        let following = state.entered.iter().skip(1).copied().collect::<Vec<_>>();
        state.released.extend(following);
        self.released.notify_all();
    }
    fn release_all(&self) {
        self.state.lock().unwrap().release_all = true;
        self.released.notify_all();
    }
}
#[cfg(feature = "solver-ipopt")]
struct StudyProbeDrain(Arc<StudyNativeProbe>);
#[cfg(feature = "solver-ipopt")]
impl Drop for StudyProbeDrain {
    fn drop(&mut self) {
        self.0.release_all();
    }
}

#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn ephemeral_study_sixteen_native_lanes_reuse_capacity_and_keep_equal_occurrences() {
    let runtime = parallel_study_runtime();
    let (package, definition) = parallel_study_definition(&runtime, 20, None).await;
    assert!(
        definition
            .points
            .windows(2)
            .all(|points| points[0].binding_hash == points[1].binding_hash)
    );
    let probe = Arc::new(StudyNativeProbe::default());
    let _drain = StudyProbeDrain(probe.clone());
    let observer = probe.clone();
    let cancel = crate::CancelSource::new();
    let run = crate::math::observed_native_entry(
        Arc::new(move |flag| observer.enter(flag)),
        package.study(&definition, 20, &cancel),
    );
    let controls = async {
        probe
            .wait(|state| state.entered.len() == 16 && state.active == 16)
            .await;
        probe.release_following();
        probe.wait(|state| state.entered.len() == 20).await;
        let state = probe.state.lock().unwrap();
        assert_eq!(state.maximum, 16);
        assert!(
            !state.released.contains(&state.entered[0]),
            "later occurrences enter while the first native owner is still held"
        );
        drop(state);
        probe.release_all();
    };
    let (report, ()) = tokio::join!(run, controls);
    let report = report.unwrap();
    assert_eq!(report.outcomes.len(), 20);
    assert!(
        report
            .outcomes
            .iter()
            .all(|outcome| outcome.scientific.usable),
        "{:?}",
        report.outcomes
    );
    assert_eq!(
        report
            .outcomes
            .iter()
            .map(|outcome| outcome.key)
            .collect::<Vec<_>>(),
        definition
            .points
            .iter()
            .map(|point| point.policy.key)
            .collect::<Vec<_>>()
    );
    let identities = report
        .results
        .iter()
        .map(|result| result.as_ref().unwrap().run_id())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        identities.len(),
        20,
        "equal bindings remain twenty distinct executions"
    );
    assert_eq!(probe.state.lock().unwrap().active, 0);
}

#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn ephemeral_study_cancellation_drains_sixteen_issued_lanes_without_starting_tail() {
    let runtime = parallel_study_runtime();
    let (package, definition) = parallel_study_definition(&runtime, 20, None).await;
    let probe = Arc::new(StudyNativeProbe::default());
    let _drain = StudyProbeDrain(probe.clone());
    let observer = probe.clone();
    let cancel = crate::CancelSource::new();
    let run = crate::math::observed_native_entry(
        Arc::new(move |flag| observer.enter(flag)),
        package.study(&definition, 20, &cancel),
    );
    let controls = async {
        probe
            .wait(|state| state.entered.len() == 16 && state.active == 16)
            .await;
        cancel.cancel();
    };
    let (report, ()) = tokio::join!(run, controls);
    let report = report.unwrap();
    assert!(
        report
            .outcomes
            .iter()
            .all(|outcome| outcome.lifecycle == StudyPointState::Cancelled)
    );
    assert!(
        report.outcomes[16..]
            .iter()
            .all(|outcome| outcome.attempts.is_empty())
    );
    assert_eq!(probe.state.lock().unwrap().entered.len(), 16);
    assert_eq!(probe.state.lock().unwrap().active, 0);
    assert!(
        report
            .outcomes
            .iter()
            .all(|outcome| outcome.effect == EffectState::Absent)
    );
}

#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn ephemeral_study_numerical_failpoint_preserves_independent_scientific_results() {
    let runtime = parallel_study_runtime();
    let (package, definition) = parallel_study_definition(&runtime, 17, Some(5)).await;
    let report = package
        .study(&definition, 17, &crate::CancelSource::new())
        .await
        .unwrap();
    assert!(!report.outcomes[5].scientific.usable);
    assert!(
        report
            .outcomes
            .iter()
            .enumerate()
            .all(|(index, outcome)| index == 5 || outcome.scientific.usable),
        "{:?}",
        report.outcomes
    );
    assert_eq!(
        report
            .outcomes
            .iter()
            .map(|outcome| outcome.key)
            .collect::<Vec<_>>(),
        definition
            .points
            .iter()
            .map(|point| point.policy.key)
            .collect::<Vec<_>>()
    );
}

#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn ephemeral_study_continuation_reuses_one_private_native_session() {
    let runtime = tests::runtime_with_workspace(64 << 20);
    let (package, definition) = related_definition(&runtime).await;
    let threads = Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));
    let observed = threads.clone();
    let report = crate::math::observed_native_entry(
        Arc::new(move |_| {
            observed.lock().unwrap().insert(std::thread::current().id());
        }),
        package.study(&definition, 3, &crate::CancelSource::new()),
    )
    .await
    .unwrap();
    assert!(
        report
            .outcomes
            .iter()
            .all(|outcome| outcome.scientific.usable)
    );
    assert_eq!(
        threads.lock().unwrap().len(),
        1,
        "a genuine chain keeps one owner thread"
    );
    for index in 1..3 {
        assert!(matches!(report.outcomes[index].start,
            Some(StartProvenance::Continuation { predecessor, .. }) if predecessor == definition.points[index - 1].policy.key));
    }

    let StudyOccurrenceResult::Ephemeral(first) = report.results[0].as_ref().unwrap() else {
        panic!("ephemeral source result expected");
    };
    let StudyOccurrenceResult::Ephemeral(third) = report.results[2].as_ref().unwrap() else {
        panic!("ephemeral target result expected");
    };
    let RunReport::Modeling(first_results) = first.report().unwrap() else {
        panic!("modeling source result expected");
    };
    let RunReport::Modeling(third_results) = third.report().unwrap() else {
        panic!("modeling target result expected");
    };
    let portable = first_results[0].portable_prediction().unwrap().unwrap();
    let lease = runtime
        .shared
        .math()
        .reserve("test:rebound-continuation-source", 8192)
        .unwrap();
    let portable = Arc::new(pse_columnar::Leased::new(Arc::new(portable), lease));
    let binding_members = definition.points[2]
        .binding
        .entries
        .keys()
        .copied()
        .collect::<Vec<_>>();
    let target = &third_results[0].prepared;
    assert_eq!(
        definition.points[0]
            .binding
            .entries
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        binding_members,
        "source and target bind the same members"
    );
    assert!(
        definition.points[0]
            .binding
            .entries
            .values()
            .all(|entry| entry.parameter)
    );
    assert!(target.profile.reconstruction.is_none());
    assert!(target.profile.numerics.goals.is_empty());
    assert!(target.providers.is_empty());
    assert!(
        target
            .model
            .model
            .compiled()
            .admitted
            .implicit_systems()
            .next()
            .is_none()
    );
    assert!(target.profile.sensitivity.is_none());
    assert_eq!(
        portable.key.numerical_policy,
        Some(target.solve.numerics().key),
        "the point must carry the same resolved numeric policy"
    );
    let rebound = study::rebind_prediction_source(
        &package,
        target,
        &definition.points[0].binding,
        &binding_members,
        portable.clone(),
        &crate::CancelSource::new(),
    )
    .await
    .unwrap()
    .expect("the exact parameter-only source is eligible for a bounded view rebind");
    let source_parameter = definition.points[0]
        .binding
        .entries
        .iter()
        .find(|(_, entry)| entry.parameter)
        .map(|(member, entry)| (*member, entry.canonical.into_inner()))
        .unwrap();
    assert_eq!(
        rebound.prepared.model.values.scalars[&source_parameter.0],
        source_parameter.1
    );
    assert_eq!(
        rebound.prepared.solve.seed_preparation_identity(),
        first_results[0].prepared.solve.seed_preparation_identity(),
        "rebound source keeps its original seed compatibility identity"
    );
    let primal = portable
        .primal
        .iter()
        .copied()
        .map(f64::from_bits)
        .collect::<Vec<_>>();
    assert_eq!(
        rebound.prepared.solve.semantic_point_key(&primal).unwrap(),
        portable.key,
        "the original portable point remains bound to the rebuilt source solve"
    );
    assert!(
        third_results[0]
            .prepared
            .solve
            .related_target_parameters(&rebound.prepared.solve, &[source_parameter])
            .is_ok()
    );
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[tokio::test]
async fn durable_worker_sixteen_cancelled_occurrences_obey_action_bound_and_drain() {
    let runtime = parallel_study_runtime();
    let runtime = runtime
        .with_durability(DurabilitySelection::Durable {
            worker: "parallel-effect-free-study".into(),
            policy: durable_tests::quick(),
        })
        .unwrap();
    let (sources, definition) = admitted(&runtime, |_, _, fixed| {
        (0..16)
            .map(|index| point(fixed, (index + 1) * 7, vec![], StartPolicy::Fresh))
            .collect()
    })
    .await;
    let handle = runtime
        .start_defined_study(sources.physical, definition, &crate::CancelSource::new())
        .await
        .unwrap();
    handle.cancel().await.unwrap();
    let settings = WorkerSettings {
        poll: Duration::from_millis(1),
        recovery: Duration::from_secs(30),
        maximum_actions: Some(7),
        maximum_in_flight: Some(16),
        until_idle: true,
    };
    assert_eq!(
        runtime
            .serve(settings, &crate::CancelSource::new())
            .await
            .unwrap(),
        7
    );
    let partial = handle.status().await.unwrap();
    assert_eq!(
        partial.points.iter().filter(|point| point.settled).count(),
        7
    );
    assert_ne!(partial.state, StudyState::Concluded);
    let remaining = runtime
        .serve(
            WorkerSettings {
                maximum_actions: None,
                ..settings
            },
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(
        remaining, 10,
        "nine occurrences and one exact parent finalization remain"
    );
    let complete = handle.status().await.unwrap();
    assert_eq!(complete.state, StudyState::Concluded);
    assert!(complete.points.iter().all(|point| point.settled
        && point.attempt.is_none()
        && point.state == StudyPointState::Cancelled));
    assert_eq!(
        runtime
            .serve(
                WorkerSettings {
                    maximum_actions: None,
                    ..settings
                },
                &crate::CancelSource::new()
            )
            .await
            .unwrap(),
        0,
        "complete drained discovery observes idle"
    );
}

#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn ephemeral_study_single_population_slot_refuses_before_preparation_or_native_entry() {
    let runtime = study_runtime_with_population(1, 1);
    let (package, definition) = related_definition(&runtime).await;
    let entered = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observer = entered.clone();
    let (result, preparations) = crate::math::counted(crate::math::observed_native_entry(
        Arc::new(move |_| {
            observer.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }),
        package.study(&definition, 3, &crate::CancelSource::new()),
    ))
    .await;
    assert!(matches!(
        result,
        Err(WorkflowError::Math(MathRuntimeError::Limit(
            "study preparation and native sequence population",
        )))
    ));
    assert_eq!(entered.load(std::sync::atomic::Ordering::Relaxed), 0);
    assert_eq!(
        (
            preparations.views,
            preparations.observations,
            preparations.rebuilt,
            preparations.shared
        ),
        (0, 0, 0, 0),
        "population refusal precedes operation preparation"
    );
}

#[cfg(feature = "canonical-tests")]
struct ManagedStudyFixture {
    runtime: Runtime,
    directory: std::path::PathBuf,
    controls_resource: String,
    nonce: String,
    worker_pid: u32,
    finished: bool,
    drain_deadline: std::sync::Mutex<Option<std::time::Instant>>,
}
#[cfg(feature = "canonical-tests")]
fn managed_private_json(path: &std::path::Path, value: &impl serde::Serialize) {
    use std::{io::Write, os::unix::fs::OpenOptionsExt};
    let bytes = serde_json::to_vec(value).unwrap();
    assert!(bytes.len() <= 8192);
    let temporary = path.with_extension(format!("{}-temporary", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .unwrap();
    file.write_all(&bytes).unwrap();
    file.sync_all().unwrap();
    std::fs::rename(temporary, path).unwrap();
}
#[cfg(feature = "canonical-tests")]
fn managed_read_json(path: &std::path::Path) -> serde_json::Value {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::symlink_metadata(path).unwrap();
    assert!(metadata.is_file() && !metadata.file_type().is_symlink());
    assert!(metadata.len() <= 8192 && metadata.mode() & 0o077 == 0);
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
#[cfg(feature = "canonical-tests")]
impl ManagedStudyFixture {
    async fn start(hold_entries: bool) -> Self {
        use std::os::unix::fs::DirBuilderExt;
        // This is the observer's separately placed pool. The managed primary retains
        // the exact 128 GiB/16 CPU/32 population profile recorded in the state.
        let runtime = tests::runtime_with_workspace(64 << 20);
        let store = runtime.canonical_store();
        let context = managed_read_json(
            &store
                .deployment_state()
                .join(".contexts")
                .join(format!("{}.json", store.database())),
        );
        let state = std::path::PathBuf::from(context["receiver_state"].as_str().unwrap());
        let allocation = store.native_allocation().unwrap();
        allocation.validate().unwrap();
        assert_eq!(allocation.native_workers, 1);
        let profile = allocation.execution.unwrap();
        assert_eq!(
            (
                profile.pool_memory_bytes,
                profile.cpu_threads,
                profile.case_lanes,
                profile.math_jobs
            ),
            (128 << 30, 16, 16, 32)
        );
        assert!(
            runtime.shared.budget().memory_limit_bytes.get() <= profile.observer_memory_bytes
                && profile.observer_memory_bytes <= 4 << 30
        );
        let receiver = store.managed_primary_receiver().unwrap().unwrap();
        let nonce: pse_ids::SemanticId = pse_operations::mint_id();
        let nonce = nonce.to_string();
        let directory = state.join(format!("native-entry-qualification-{nonce}"));
        let controls_resource = pse_operations::testing::register_fixture_controls(
            &state,
            store.database(),
            &directory,
        )
        .unwrap();
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .unwrap();
        managed_private_json(
            &directory.join("request.json"),
            &serde_json::json!({
                "nonce": nonce, "canonical_database": store.database(), "entries": 16, "entry_timeout_ms": 90_000,
            }),
        );
        let mut fixture = Self {
            runtime: runtime
                .with_durability(DurabilitySelection::Durable {
                    worker: "managed-study-observer".into(),
                    policy: durable_tests::quick(),
                })
                .unwrap(),
            directory,
            controls_resource,
            nonce,
            worker_pid: 0,
            finished: false,
            drain_deadline: std::sync::Mutex::new(None),
        };
        if !hold_entries {
            fixture.signal("release.json");
        }
        let output = tokio::time::timeout(
            Duration::from_secs(45),
            tokio::process::Command::new(&receiver.supervisor_executable)
                .arg(&receiver.supervisor_script)
                .arg("ensure-primary")
                .arg("--state")
                .arg(&state)
                .arg("--canonical-database")
                .arg(fixture.runtime.canonical_store().database())
                .arg("--observer-pid")
                .arg(std::process::id().to_string())
                .arg("--qualification-native-entry")
                .arg(&fixture.directory)
                .kill_on_drop(true)
                .output(),
        )
        .await
        .unwrap()
        .unwrap();
        assert!(output.stdout.len() <= 64 << 10 && output.stderr.len() <= 64 << 10);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let ready: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(ready["ready"], true);
        assert_eq!(
            ready["canonical_database"],
            fixture.runtime.canonical_store().database()
        );
        let marker = managed_read_json(&state.join("primary-receiver.json"));
        assert_eq!(
            marker["canonical_database"],
            fixture.runtime.canonical_store().database()
        );
        assert_eq!(marker["pool_memory_bytes"], 128_u64 << 30);
        assert_eq!(marker["cpu_threads"], 16);
        assert_eq!(marker["case_lanes"], 16);
        assert_eq!(marker["math_jobs"], 32);
        assert_eq!(
            marker["qualification_native_entry"],
            fixture.directory.to_str().unwrap()
        );
        fixture.worker_pid = u32::try_from(marker["pid"].as_u64().unwrap()).unwrap();
        assert_ne!(fixture.worker_pid, std::process::id());
        fixture
    }
    fn signal(&self, name: &str) {
        managed_private_json(&self.directory.join(name), &self.nonce);
    }
    fn drain_deadline(&self) -> std::time::Instant {
        let mut deadline = self.drain_deadline.lock().unwrap();
        *deadline.get_or_insert_with(|| std::time::Instant::now() + Duration::from_secs(45))
    }
    async fn dispatch<T>(
        &self,
        cancel: &crate::CancelSource,
        operation: impl Future<Output = T>,
    ) -> T {
        tokio::pin!(operation);
        let failure = tokio::select! {
            result = &mut operation => return result,
            failure = async {
                let deadline = std::time::Instant::now() + Duration::from_secs(90);
                loop {
                    let failure = self.directory.join("failure.json");
                    if failure.exists() {
                        return format!("native qualification controller failed: {:?}", managed_read_json(&failure));
                    }
                    if !std::path::Path::new(&format!("/proc/{}", self.worker_pid)).exists() {
                        return String::from("qualified primary exited during public study dispatch");
                    }
                    if std::time::Instant::now() >= deadline {
                        return String::from("public managed study dispatch exceeded its 90 second operation bound");
                    }
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            } => failure,
        };
        cancel.cancel();
        for name in ["release.json", "stop.json"] {
            if !self.directory.join(name).exists() {
                self.signal(name);
            }
        }
        // Keep the public future and its cancellation owner alive while the real
        // primary drains. A timeout drops neither the fixture nor its drain owner;
        // Drop retains the actual worker drain ownership during the explicit failure.
        let drained = tokio::time::timeout_at(
            tokio::time::Instant::from_std(self.drain_deadline()),
            async {
                tokio::join!(
                    async {
                        let _ = (&mut operation).await;
                    },
                    async {
                        while std::path::Path::new(&format!("/proc/{}", self.worker_pid)).exists() {
                            tokio::time::sleep(Duration::from_millis(20)).await;
                        }
                    }
                );
            },
        )
        .await
        .is_ok();
        panic!("{failure}; public cancellation and primary drain completed: {drained}");
    }
    #[cfg(feature = "solver-ipopt")]
    async fn observation(
        &self,
        predicate: impl Fn(&serde_json::Value) -> bool,
    ) -> serde_json::Value {
        tokio::time::timeout(Duration::from_secs(90), async {
            loop {
                let failure = self.directory.join("failure.json");
                assert!(
                    !failure.exists(),
                    "native qualification failed: {:?}",
                    failure.exists().then(|| managed_read_json(&failure))
                );
                assert!(
                    std::path::Path::new(&format!("/proc/{}", self.worker_pid)).exists(),
                    "qualified primary exited before its native observation"
                );
                let entries = self.directory.join("entries.json");
                if !entries.exists() {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    continue;
                }
                let value = managed_read_json(&entries);
                assert_eq!(value["nonce"], self.nonce);
                assert_eq!(
                    value["canonical_database"],
                    self.runtime.canonical_store().database()
                );
                assert_eq!(value["pid"], self.worker_pid);
                if predicate(&value) {
                    return value;
                }
                assert!(
                    std::path::Path::new(&format!("/proc/{}", self.worker_pid)).exists(),
                    "qualified primary exited before its native observation"
                );
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap_or_else(|_| {
            let entries = self.directory.join("entries.json");
            let last = entries.exists().then(|| managed_read_json(&entries));
            panic!(
                "native entry observation exceeded 90 seconds: active {:?}, maximum {:?}, observed entries {:?}, worker {}",
                last.as_ref().and_then(|value| value["active"].as_u64()),
                last.as_ref().and_then(|value| value["maximum"].as_u64()),
                last.as_ref().and_then(|value| value["entries"].as_array().map(Vec::len)),
                self.worker_pid,
            )
        })
    }
    async fn finish(mut self) {
        self.signal("release.json");
        self.signal("stop.json");
        tokio::time::timeout_at(
            tokio::time::Instant::from_std(self.drain_deadline()),
            async {
                while std::path::Path::new(&format!("/proc/{}", self.worker_pid)).exists() {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            },
        )
        .await
        .unwrap();
        let final_entries = managed_read_json(&self.directory.join("entries.json"));
        assert_eq!(final_entries["active"], 0);
        assert!(!self.directory.join("failure.json").exists());
        self.finished = true;
    }
}
#[cfg(feature = "canonical-tests")]
impl Drop for ManagedStudyFixture {
    fn drop(&mut self) {
        if self.worker_pid == 0 {
            // Startup may publish the bounded controller observation before readiness
            // fails. Associate only this fixture's nonce/database with its actual PID.
            let observation = self.directory.join("entries.json");
            if let Ok(metadata) = std::fs::symlink_metadata(&observation)
                && metadata.is_file()
                && !metadata.file_type().is_symlink()
                && metadata.len() <= 8192
                && let Ok(bytes) = std::fs::read(&observation)
                && let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes)
                && value["nonce"] == self.nonce
                && value["canonical_database"] == self.runtime.canonical_store().database()
                && let Some(pid) = value["pid"]
                    .as_u64()
                    .and_then(|pid| u32::try_from(pid).ok())
            {
                self.worker_pid = pid;
            }
        }
        if !self.finished {
            // Qualification panic also releases native entry owners and asks the same
            // primary driver to cancel/drain; it never tears a running owner away.
            if !self.directory.join("release.json").exists() {
                self.signal("release.json");
            }
            if !self.directory.join("stop.json").exists() {
                self.signal("stop.json");
            }
            let deadline = self.drain_deadline();
            while self.worker_pid != 0
                && std::path::Path::new(&format!("/proc/{}", self.worker_pid)).exists()
                && std::time::Instant::now() < deadline
            {
                std::thread::sleep(Duration::from_millis(20));
            }
        }
        if self.worker_pid == 0
            || !std::path::Path::new(&format!("/proc/{}", self.worker_pid)).exists()
        {
            let result = pse_operations::testing::record_fixture_controls_drain(
                &self.controls_resource,
                self.directory.parent().unwrap(),
                self.runtime.canonical_store().database(),
            );
            if let Err(error) = result {
                if std::thread::panicking() {
                    eprintln!("managed fixture retained after drain failure: {error}");
                } else {
                    panic!("managed fixture drain failed: {error}");
                }
            }
        }
    }
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[tokio::test]
async fn managed_primary_study_enters_sixteen_native_owners_and_retains_original_results() {
    use pse_relations::generated::runtime::solve_variables;
    use std::collections::BTreeSet;
    let fixture = ManagedStudyFixture::start(true).await;
    let runtime = &fixture.runtime;
    let (package, definition) = parallel_study_definition(runtime, 16, None).await;
    assert!(
        definition
            .points
            .windows(2)
            .all(|points| points[0].binding_hash == points[1].binding_hash)
    );
    let cancel = crate::CancelSource::new();
    let control = async {
        let value = fixture.observation(|value| value["active"] == 16).await;
        let entries = value["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 16);
        assert_eq!(value["maximum"], 16);
        assert!(entries.iter().all(|entry| entry["released"] == false));
        let threads = entries
            .iter()
            .map(|entry| entry["thread"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            threads.len(),
            16,
            "sixteen actual owner threads on one primary PID hold CPU admission together"
        );
        fixture.signal("release.json");
    };
    let (report, ()) = fixture
        .dispatch(&cancel, async {
            tokio::join!(package.study(&definition, 16, &cancel), control)
        })
        .await;
    let report = report.unwrap();
    assert!(
        report
            .outcomes
            .iter()
            .all(|outcome| outcome.scientific.usable),
        "{:?}",
        report.outcomes
    );
    assert_eq!(
        report
            .outcomes
            .iter()
            .map(|outcome| outcome.key)
            .collect::<Vec<_>>(),
        definition
            .points
            .iter()
            .map(|point| point.policy.key)
            .collect::<Vec<_>>()
    );
    let identities = report
        .results
        .iter()
        .map(|result| result.as_ref().unwrap().run_id())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        identities.len(),
        16,
        "equal bindings have sixteen exact retained executions"
    );
    for result in &report.results {
        let result = result.as_ref().unwrap();
        assert!(matches!(result, StudyOccurrenceResult::Retained { .. }));
        let (run, attempt) = result.stored_keys().unwrap();
        let mut variables = runtime
            .results(
                run,
                attempt,
                "runtime.solve_variables",
                0,
                u64::MAX,
                pse_columnar::CancellationToken::new(),
            )
            .await
            .unwrap();
        let mut roots = 0;
        while let Some(batch) = variables.next_batch().await.unwrap() {
            let view = solve_variables::View::try_from_batch_with_registry(
                &runtime.registry,
                &batch,
                &runtime.validation_context().unwrap(),
            )
            .unwrap();
            for row in view
                .rows()
                .unwrap()
                .into_iter()
                .filter(|row| !row.parameter)
            {
                let value = row.value.unwrap();
                let allowance = row.tolerance.unwrap();
                assert!(value.is_finite() && allowance.is_finite() && allowance > 0.0);
                assert!(
                    (value - 2.0).abs() <= allowance,
                    "original native root must satisfy its published production allowance"
                );
                roots += 1;
            }
        }
        assert_eq!(roots, 1);
    }
    fixture.observation(|value| value["active"] == 0).await;
    fixture.finish().await;
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[tokio::test]
async fn managed_primary_study_cancellation_drains_admitted_sixteen_and_refuses_tail() {
    let fixture = ManagedStudyFixture::start(true).await;
    let (package, definition) = parallel_study_definition(&fixture.runtime, 20, None).await;
    let cancel = crate::CancelSource::new();
    let control = async {
        fixture.observation(|value| value["active"] == 16).await;
        cancel.cancel();
        let stopped = fixture.observation(|value| value["active"] == 0).await;
        assert_eq!(stopped["maximum"], 16);
        let entries = stopped["entries"].as_array().unwrap();
        assert_eq!(
            entries.len(),
            16,
            "cancellation cannot issue a seventeenth native owner"
        );
        assert!(
            entries
                .iter()
                .all(|entry| entry["released"] == true && entry["stop_observed"] == true),
            "actual admitted owners exit their entry barrier through the native stop flag"
        );
    };
    let (report, ()) = fixture
        .dispatch(&cancel, async {
            tokio::join!(package.study(&definition, 20, &cancel), control)
        })
        .await;
    let report = report.unwrap();
    assert!(
        report
            .outcomes
            .iter()
            .all(|outcome| outcome.lifecycle == StudyPointState::Cancelled),
        "{:?}",
        report.outcomes
    );
    assert!(
        report.outcomes[16..]
            .iter()
            .all(|outcome| outcome.attempts.is_empty()),
        "the unissued tail settles through the existing effect-free cancellation policy"
    );
    assert!(report.results[16..].iter().all(Option::is_none));
    fixture.finish().await;
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[allow(
    unsafe_code,
    reason = "separate finite Arrow transport fixture asserts Partial only; sixteen ordinary native roots retain their actual scientific qualification"
)]
#[tokio::test]
async fn managed_primary_publication_serves_exact_partial_history_and_escaped_arrow_retirement() {
    use datafusion::arrow::array::{Array, Float64Array};
    use pse_columnar::CancellationToken;
    use pse_operations::canonical_analyses::AnalysisNode;
    use pse_operations::canonical_execution::{RunRequest, TerminalClass};
    use pse_relations::generated::{
        enums::NativeMetricKind,
        runtime::{solve_metrics, solve_variables},
    };
    use std::sync::atomic::{AtomicBool, Ordering};

    let fixture = ManagedStudyFixture::start(true).await;
    let runtime = &fixture.runtime;
    let store = runtime.canonical_store();
    let pool = runtime.shared.pool();
    let validation = runtime.validation_context().unwrap();
    let transport_run = format!("serving-transport-{}", fixture.nonce);
    let revision = store
        .edit(&transport_run, None, "partial Arrow transport fixture", &[])
        .await
        .unwrap();
    store
        .begin_run(&RunRequest {
            key: transport_run.clone(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        })
        .await
        .unwrap();
    let mut attempts = Vec::new();
    // These two transport-only attempts exercise actual typed publication and
    // exact current/history selection; neither fabricates scientific success.
    for (ordinal, temperature) in [273.15, 300.0].into_iter().enumerate() {
        let fence = store
            .claim_run(
                &transport_run,
                &format!("serving-claim-{ordinal}"),
                "serving-transport-fixture",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        let run_id = pse_operations::mint_id();
        let mut builder =
            solve_metrics::Builder::with_registry(&runtime.registry, 3, &validation).unwrap();
        for (step, real) in [Some(-0.0), None, Some(temperature)]
            .into_iter()
            .enumerate()
        {
            builder
                .push(solve_metrics::Row {
                    run_id,
                    step: step as i64,
                    namespace: "partial-serving-transport".into(),
                    name: "temperature".into(),
                    kind: if real.is_some() {
                        NativeMetricKind::Real
                    } else {
                        NativeMetricKind::Unavailable
                    },
                    real,
                    integer: None,
                    boolean: None,
                    text: None,
                    unavailable: real.is_none().then_some(
                        pse_model::generated::enums::EvidenceUnavailableReason::Nonfinite,
                    ),
                })
                .unwrap();
        }
        let table = builder.finish().unwrap();
        result_projection::store_result_table(
            store,
            &fence,
            solve_metrics::RELATION_ID,
            ResultCursor::from_checked(table.clone()),
            &pool,
        )
        .await
        .unwrap();
        let closed = store
            .close_result_ingestion(&fence, &format!("serving-close-{ordinal}"))
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        let seal = format!("serving-seal-{ordinal}");
        // SAFETY: these finite generated transport rows establish Partial only.
        unsafe { store.seal_attempt(&manifest, &seal, TerminalClass::Partial, &[1]) }
            .await
            .unwrap();
        attempts.push(fence.attempt().to_owned());
    }

    let (package, definition) = parallel_study_definition(runtime, 16, None).await;
    let cancel = crate::CancelSource::new();
    let published = AtomicBool::new(false);
    let science = async {
        let report = package.study(&definition, 16, &cancel).await.unwrap();
        published.store(true, Ordering::Release);
        report
    };
    let serving = async {
        let held = fixture.observation(|value| value["active"] == 16).await;
        let owners = held["entries"].as_array().unwrap();
        assert_eq!(owners.len(), 16);
        assert_eq!(held["maximum"], 16);
        assert_eq!(
            owners
                .iter()
                .map(|entry| entry["thread"].as_str().unwrap())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            16
        );
        assert!(owners.iter().all(|entry| entry["released"] == false));
        assert!(!published.load(Ordering::Acquire));
        assert!(
            runtime
                .latest_results(
                    &transport_run,
                    &[TerminalClass::Succeeded],
                    "runtime.solve_metrics",
                    0,
                    3,
                    CancellationToken::new(),
                )
                .await
                .is_err(),
            "a partial transport observation cannot be substituted for scientific success"
        );
        let mut current = runtime
            .latest_results(
                &transport_run,
                &[TerminalClass::Partial],
                "runtime.solve_metrics",
                1,
                3,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let mut historical = runtime
            .results(
                &transport_run,
                &attempts[0],
                "runtime.solve_metrics",
                0,
                3,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(current.selection().attempt().key, attempts[1]);
        assert_eq!(historical.selection().attempt().key, attempts[0]);
        assert_eq!(
            current.selection().attempt().outcome.as_deref(),
            Some("partial")
        );
        assert_eq!(
            historical.selection().attempt().outcome.as_deref(),
            Some("partial")
        );
        let current_batch = current.next_batch().await.unwrap().unwrap();
        let historical_batch = historical.next_batch().await.unwrap().unwrap();
        assert_eq!(current_batch.num_rows(), 2);
        assert_eq!(historical_batch.num_rows(), 3);
        let current_values = current_batch.column_by_name("real").unwrap().clone();
        let historical_values = historical_batch.column_by_name("real").unwrap().clone();
        let values = current_values
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert!(values.is_null(0));
        assert_eq!(values.value(1), 300.0);
        let values = historical_values
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(values.value(0).to_bits(), (-0.0f64).to_bits());
        assert!(values.is_null(1));
        assert_eq!(values.value(2), 273.15);
        assert!(current.next_batch().await.unwrap().is_none());
        assert!(historical.next_batch().await.unwrap().is_none());
        assert!(store.forget_run_results(&transport_run).await.is_err());

        let mut analysis = store
            .new_analysis(
                &revision,
                "partial-transport-reader-handoff-fixture:v2",
                vec![1],
                current.selection().manifest().digest.clone(),
                1,
                0,
            )
            .await
            .unwrap();
        let node = AnalysisNode {
            key: format!("serving-transport-node-{}", fixture.nonce),
            analysis: analysis.key.clone(),
            semantic: "partial-transport-temperature".into(),
            kind: "result".into(),
        };
        let inputs = [current.selection().clone(), historical.selection().clone()];
        pse_operations::canonical_analyses::seal_analysis_request(
            &mut analysis,
            std::slice::from_ref(&revision),
            &inputs,
            std::slice::from_ref(&node),
            &[],
        );
        assert!(
            store
                .persist_analysis(
                    &analysis,
                    std::slice::from_ref(&revision),
                    &inputs,
                    &[node],
                    &[]
                )
                .await
                .unwrap()
                .active
        );
        drop(inputs);
        // Both exact protected selections are open before any native owner is
        // released, and remain open throughout all sixteen real publications.
        fixture.signal("release.json");
        let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
        let mut serving_reads = 0;
        loop {
            let mut reopened = runtime
                .results(
                    &transport_run,
                    &attempts[0],
                    "runtime.solve_metrics",
                    2,
                    3,
                    CancellationToken::new(),
                )
                .await
                .unwrap();
            assert_eq!(reopened.selection().attempt().key, attempts[0]);
            let batch = reopened.next_batch().await.unwrap().unwrap();
            assert_eq!(batch.num_rows(), 1);
            assert_eq!(
                batch
                    .column_by_name("real")
                    .unwrap()
                    .as_any()
                    .downcast_ref::<Float64Array>()
                    .unwrap()
                    .value(0),
                273.15
            );
            assert!(reopened.next_batch().await.unwrap().is_none());
            serving_reads += 1;
            if published.load(Ordering::Acquire) {
                break;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "scientific publication did not finish"
            );
        }
        assert!(serving_reads > 0);
        fixture.observation(|value| value["active"] == 0).await;
        drop(current_batch);
        drop(historical_batch);
        drop(current);
        drop(historical);
        assert!(
            store.forget_run_results(&transport_run).await.is_err(),
            "analysis retention takes over before the readers release their exact inputs"
        );
        for _ in 0..5 {
            assert!(
                !runtime
                    .forget_analysis_results(&analysis.key)
                    .await
                    .unwrap()
            );
        }
        let mut retired = false;
        for _ in 0..32 {
            if store.forget_run_results(&transport_run).await.is_ok() {
                retired = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(retired, "reader release did not permit bounded retirement");
        let mut reclaimed = 0;
        let mut complete = false;
        for _ in 0..8 {
            let page = store.reclaim_result_page(&transport_run).await.unwrap();
            reclaimed += page.batches;
            if page.complete {
                complete = true;
                break;
            }
        }
        assert!(complete);
        assert_eq!(reclaimed, 2);
        assert!(
            runtime
                .results(
                    &transport_run,
                    &attempts[0],
                    "runtime.solve_metrics",
                    0,
                    3,
                    CancellationToken::new(),
                )
                .await
                .is_err()
        );
        // Escaped decoded Arrow buffers need no storage access after reader,
        // publication, analysis and result cleanup have all finished.
        let values = current_values
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert!(values.is_null(0));
        assert_eq!(values.value(1), 300.0);
        let values = historical_values
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert_eq!(values.value(0).to_bits(), (-0.0f64).to_bits());
        assert!(values.is_null(1));
        assert_eq!(values.value(2), 273.15);
        let retained_bytes = pool.reserved();
        drop(current_values);
        drop(historical_values);
        assert!(
            pool.reserved() < retained_bytes,
            "escaping arrays retain real accounted allocation owners"
        );
    };
    let (report, ()) = fixture
        .dispatch(&cancel, async { tokio::join!(science, serving) })
        .await;
    assert_eq!(report.outcomes.len(), 16);
    assert!(
        report
            .outcomes
            .iter()
            .all(|outcome| outcome.scientific.usable)
    );
    assert_eq!(
        report
            .outcomes
            .iter()
            .map(|outcome| outcome.key)
            .collect::<Vec<_>>(),
        definition
            .points
            .iter()
            .map(|point| point.policy.key)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        report
            .results
            .iter()
            .map(|result| result.as_ref().unwrap().run_id())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        16
    );
    for result in &report.results {
        let (run, attempt) = result.as_ref().unwrap().stored_keys().unwrap();
        let mut reader = runtime
            .results(
                run,
                attempt,
                "runtime.solve_variables",
                0,
                u64::MAX,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(
            reader.selection().attempt().outcome.as_deref(),
            Some("succeeded")
        );
        let mut roots = 0;
        while let Some(batch) = reader.next_batch().await.unwrap() {
            let view = solve_variables::View::try_from_batch_with_registry(
                &runtime.registry,
                &batch,
                &validation,
            )
            .unwrap();
            for row in view.rows().unwrap() {
                let value = row.value.unwrap();
                if row.parameter {
                    assert_eq!(value, 4.0);
                } else {
                    let allowance = row.tolerance.unwrap();
                    assert!(value.is_finite() && allowance.is_finite() && allowance > 0.0);
                    assert!((value - 2.0).abs() <= allowance);
                    assert!(value > 1.0, "original authored root check");
                    roots += 1;
                }
            }
        }
        assert_eq!(roots, 1);
    }
    fixture.finish().await;
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn ephemeral_study_complete_kinsol_blocks_seed_original_eight_point_continuation() {
    // The original benchmark science: two independent root equations, the same
    // repeated parameter values, authored start/checks and exact continuation policy.
    const BLOCKS: &str = "package k4 { def Root { param a:Scalar=2; var x:Scalar; var y:Scalar; eq first:x==a; eq second:y==3; annotation start x(1); annotation start y(1); annotation report x(\"x\"); annotation check x(abs(x-a)<1e-7); annotation check y(abs(y-3)<1e-7); } def Fixed { param x:Scalar=2; eq first:x==2; } }";
    const VALUES: [f64; 8] = [2., 2., 3., 2., 4., 3., 2., 2.];
    let runtime = tests::runtime_with_workspace(64 << 20);
    let cancel = crate::CancelSource::new();
    let (physical, modeling) = sources(BLOCKS);
    let package = runtime
        .package_from_sources(
            std::slice::from_ref(&modeling),
            runtime
                .physical_from_sources(&physical, &cancel)
                .await
                .unwrap(),
            &cancel,
        )
        .await
        .unwrap();
    let declarations = package.declarations().await.unwrap();
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let fixed = declarations
        .iter()
        .find(|row| row.name == "Fixed")
        .unwrap()
        .declaration_id;
    let scalar = package.quantities.neutral_dimensionless().unwrap();
    let unit = package
        .quantities
        .quantity_type(scalar)
        .unwrap()
        .canonical_unit;
    let mut points = Vec::new();
    for (index, value) in VALUES.into_iter().enumerate() {
        let index = index as u32;
        let mut next = point(
            root,
            index,
            if index == 0 {
                vec![]
            } else {
                vec![Dependency::Ordering(OccurrenceKey(index - 1))]
            },
            if index == 0 {
                StartPolicy::Fresh
            } else {
                StartPolicy::Continuation(SeedEdge {
                    predecessor: OccurrenceKey(index - 1),
                    role: SeedRole::PrimalSolution,
                    permission: ContinuationPermission::RequireUsable,
                    unavailable: UnavailableSeedPolicy::Refuse,
                })
            },
        );
        let OperationRequest::DeclaredCase(operation) = &mut next.operation else {
            panic!("declared root")
        };
        operation.settings = crate::math::settings::SolveSettings {
            backend: Some(pse_backend_native::solve::Backend::Kinsol),
            intent: pse_backend_native::solve::SolveIntent::Root,
            presolve: pse_backend_native::presolve::PolicyKind::Off,
            controls: pse_backend_native::solve::Controls {
                threads: 1,
                ..Default::default()
            },
            ..Default::default()
        };
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
        .admit_study_sources(&physical, &points, &cancel)
        .await
        .unwrap();
    let study = package.study(&definition, 8, &cancel).await.unwrap();
    let mut identities = std::collections::BTreeSet::new();
    for (index, (outcome, expected)) in study.outcomes.iter().zip(VALUES).enumerate() {
        assert_eq!(outcome.key, OccurrenceKey(index as u32));
        assert_eq!(outcome.lifecycle, StudyPointState::Completed, "{outcome:?}");
        assert!(outcome.scientific.usable, "{outcome:?}");
        assert!(outcome.diagnostic.is_none(), "{outcome:?}");
        assert_eq!(outcome.attempts.len(), 1);
        if index > 0 {
            assert!(
                matches!(outcome.start, Some(StartProvenance::Continuation { predecessor, .. })
                if predecessor == OccurrenceKey(index as u32 - 1)),
                "{outcome:?}"
            );
        }
        let occurrence = study.results[index].as_ref().unwrap();
        assert!(identities.insert(occurrence.run_id()));
        let RunReport::Modeling(reports) = occurrence.report().unwrap() else {
            panic!("modeling")
        };
        assert_eq!(reports.len(), 1);
        let source = study_execution::completed_modeling_seed(&reports[0]).unwrap();
        let lease = runtime
            .native()
            .reserve("test:original-seed", source.owned_extent().unwrap())
            .unwrap();
        let leased = source.materialize(lease).unwrap();
        let seed = leased.warm();
        assert_eq!(seed.origin.as_ref().unwrap().run, Some(occurrence.run_id()));
        assert_eq!(seed.origin.as_ref().unwrap().attempt, 0);
        let pse_backend_native::solve::WarmPayload::Root(primal) = &seed.payload else {
            panic!("root primal")
        };
        assert_eq!(primal.len(), 2);
        assert!(primal.iter().any(|value| (*value - expected).abs() < 1e-7));
        assert!(primal.iter().any(|value| (*value - 3.).abs() < 1e-7));
        if index == 0 {
            let crate::math::solves::Outcome::Constant(completed) = &reports[0].outcome else {
                panic!(
                    "automatic block completion must exercise original-coordinate seed extraction"
                )
            };
            assert_eq!(completed.component_reports().len(), 2);
            assert!(
                completed
                    .component_reports()
                    .all(|native| native.backend == pse_backend_native::solve::Backend::Kinsol)
            );
            assert_eq!(
                primal,
                &completed
                    .coordinates
                    .iter()
                    .map(|(_, value)| *value)
                    .collect::<Vec<_>>()
            );
        }
    }
    // A genuinely all-fixed original evaluation has no numerical compatibility
    // or free coordinates and must remain seed-free.
    let mut fixed_point = point(fixed, 9, vec![], StartPolicy::Fresh);
    let OperationRequest::DeclaredCase(operation) = &mut fixed_point.operation else {
        panic!("declared fixed")
    };
    operation.settings.backend = Some(pse_backend_native::solve::Backend::Kinsol);
    let fixed_definition = package
        .admit_study_sources(&physical, &[fixed_point], &cancel)
        .await
        .unwrap();
    let fixed = package.study(&fixed_definition, 1, &cancel).await.unwrap();
    assert!(fixed.outcomes[0].scientific.usable);
    let RunReport::Modeling(reports) = fixed.results[0].as_ref().unwrap().report().unwrap() else {
        panic!("modeling")
    };
    assert!(reports[0].prepared.solve.compatibility().is_none());
    assert!(study_execution::completed_modeling_seed(&reports[0]).is_none());
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn complete_original_seed_reserves_before_allocation_and_preserves_retry_attempt() {
    use datafusion::execution::memory_pool::MemoryConsumer;
    const COORDINATES: usize = 64;
    let mut source = String::from("package roomy { def Root { param a:Scalar=2;");
    for index in 0..COORDINATES {
        source.push_str(&format!("var x{index}:Scalar; eq e{index}:x{index}==a; annotation start x{index}(1); annotation check x{index}(abs(x{index}-a)<1e-7);"));
    }
    source.push_str("} }");
    let runtime = tests::runtime_with_workspace(64 << 20);
    let cancel = crate::CancelSource::new();
    let (physical, modeling) = sources(&source);
    let package = runtime
        .package_from_sources(
            std::slice::from_ref(&modeling),
            runtime
                .physical_from_sources(&physical, &cancel)
                .await
                .unwrap(),
            &cancel,
        )
        .await
        .unwrap();
    let root = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let mut requested = point(root, 0, vec![], StartPolicy::Fresh);
    let OperationRequest::DeclaredCase(operation) = &mut requested.operation else {
        panic!("declared root")
    };
    operation.settings.backend = Some(pse_backend_native::solve::Backend::Kinsol);
    operation.settings.presolve = pse_backend_native::presolve::PolicyKind::Off;
    let definition = package
        .admit_study_sources(&physical, &[requested], &cancel)
        .await
        .unwrap();
    let PreparedStudyOperation::DeclaredCase(prepared) = definition.points[0]
        .operation
        .prepare(&package, &cancel)
        .await
        .unwrap()
    else {
        panic!("declared root")
    };
    let mut staged = staged::Staged::open(&runtime, None).unwrap();
    let run_id = pse_operations::mint_id();
    // Actual completion-owner attempt3, with64 native components. Neither the
    // study occurrence ordinal0 nor any component position defines provenance.
    let completed = staged
        .run(
            *prepared.clone(),
            modeling::assessment::Obligations::Final,
            run_id,
            3,
            None,
            &cancel,
        )
        .await
        .unwrap();
    staged.close().await;
    assert!(completed.accepted);
    assert_eq!(completed.original_attempt, 3);
    let crate::math::solves::Outcome::Constant(original) = &completed.outcome else {
        panic!("complete block evaluation")
    };
    assert_eq!(original.coordinates.len(), COORDINATES);
    assert_eq!(original.component_reports().len(), COORDINATES);
    let pool = runtime.shared.pool();
    let reserved = pool.reserved();
    let descriptor = study_execution::completed_modeling_seed(&completed).unwrap();
    assert_eq!(
        pool.reserved(),
        reserved,
        "compatibility inspection retains no numeric allocation"
    );
    let extent = descriptor.owned_extent().unwrap();
    assert!(extent >= COORDINATES * size_of::<f64>());
    let lease = runtime
        .native()
        .reserve("test:complete-original-seed", extent)
        .unwrap();
    let leased = descriptor.materialize(lease).unwrap();
    assert_eq!(leased.warm().origin.as_ref().unwrap().run, Some(run_id));
    assert_eq!(leased.warm().origin.as_ref().unwrap().attempt, 3);
    let pse_backend_native::solve::WarmPayload::Root(primal) = &leased.warm().payload else {
        panic!("root seed")
    };
    assert_eq!(primal.len(), COORDINATES);
    assert!(primal.iter().all(|value| (*value - 2.).abs() < 1e-7));
    assert_eq!(
        pool.reserved(),
        reserved + extent,
        "seed wrapper retains the full pre-allocation grant"
    );
    drop(leased);
    assert_eq!(pool.reserved(), reserved);
    let insufficient = runtime
        .native()
        .reserve("test:unadmitted-seed", extent - 1)
        .unwrap();
    assert!(
        descriptor.materialize(insufficient).is_err(),
        "insufficient grant refuses before collecting the primal vector"
    );
    assert_eq!(pool.reserved(), reserved);

    let previous = RunResult::joined(
        run_id,
        runtime.clone(),
        RunRequest::Modeling(vec![*prepared.clone()]),
        None,
        Ok(RunReport::Modeling(vec![completed.clone()])),
    )
    .finished(None, false)
    .await;
    let mut target = prepared;
    let allowance = extent - 1;
    let filler = MemoryConsumer::new("test:seed-pressure").register(&pool);
    filler
        .try_grow(runtime.shared.budget().memory_limit_bytes.get() - pool.reserved() - allowance)
        .unwrap();
    let before = pool.reserved();
    let error =
        study_execution::continuation_start(&package, &mut target, Some(&previous), None, &cancel)
            .await
            .unwrap_err();
    assert_eq!(
        error.boundary_diagnostic().rule,
        pse_diagnostics::DiagnosticRule::MathLimit
    );
    assert_eq!(
        pool.reserved(),
        before,
        "failed reservation allocates no escaping numeric seed"
    );
    assert!(target.stored_seed_owner.is_none());
    drop(filler);
    let before = pool.reserved();
    study_execution::continuation_start(&package, &mut target, Some(&previous), None, &cancel)
        .await
        .unwrap();
    assert!(
        target.stored_seed_owner.is_some(),
        "prepared target owns the converted primal grant"
    );
    assert_eq!(pool.reserved(), before + extent);
    let seed_identity = Arc::as_ptr(target.solve.explicit_seed().unwrap());
    let filler = MemoryConsumer::new("test:seed-clone-pressure").register(&pool);
    filler
        .try_grow(runtime.shared.budget().memory_limit_bytes.get() - pool.reserved())
        .unwrap();
    let saturated = pool.reserved();
    // Actual execution callers clone preparations for the RunRequest and for
    // assessment. They share one immutable numeric seed and its original grant.
    let bare = target.solve.clone();
    assert_eq!(Arc::as_ptr(bare.explicit_seed().unwrap()), seed_identity);
    let request = RunRequest::Modeling(vec![target.as_ref().clone()]);
    let mut clones: Vec<_> = (0..16).map(|_| target.as_ref().clone()).collect();
    let RunRequest::Modeling(requests) = &request else {
        panic!("modeling request")
    };
    for clone in clones.iter().chain(requests) {
        assert_eq!(
            Arc::as_ptr(clone.solve.explicit_seed().unwrap()),
            seed_identity,
            "preparation cloning must share the actual WarmStart payload"
        );
        assert!(Arc::ptr_eq(
            clone.stored_seed_owner.as_ref().unwrap(),
            target.stored_seed_owner.as_ref().unwrap()
        ));
    }
    assert_eq!(
        pool.reserved(),
        saturated,
        "cloned preparations require no duplicate seed allocation"
    );
    drop(target);
    assert_eq!(
        pool.reserved(),
        saturated,
        "other preparations still own the seed grant"
    );
    drop(request);
    while let Some(clone) = clones.pop() {
        drop(clone);
        assert_eq!(
            pool.reserved(),
            saturated,
            "the bare solve retains its seed grant after every modeling owner drops"
        );
    }
    assert_eq!(Arc::as_ptr(bare.explicit_seed().unwrap()), seed_identity);
    drop(bare);
    assert_eq!(
        pool.reserved(),
        saturated - extent,
        "the final bare solve releases the shared seed payload and grant together"
    );
    drop(filler);
    assert_eq!(pool.reserved(), before);
}
