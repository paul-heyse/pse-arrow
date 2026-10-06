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
#[cfg(feature = "canonical-tests")]
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
        .start_defined_study(sources.physical, definition)
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
        .start_study(StudyPlan {
            sources,
            points: authored.unwrap(),
        })
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
        .start_defined_study(sources.physical, definition)
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
        .start_defined_study(sources.physical, definition)
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
            Duration::from_micros(1),
        )
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
async fn canonical_study_reopened_continuation_uses_shared_secant_and_original_correction() {
    use pse_relations::generated::runtime::{solve_strategy_events, solve_variables};
    let mut runtime = tests::runtime_with_workspace(64 << 20);
    let operations = Operations::from_store(
        runtime.canonical_store().clone(),
        "retained-prediction",
        durable_tests::quick(),
        runtime.shared.pool().clone(),
    );
    runtime = runtime.with_durability(Durability::Durable(operations.clone()));
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
        .start_defined_study(physical, definition)
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
        .start_defined_study(physical, definition)
        .await
        .unwrap();
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
    drop(package);
}

#[cfg(feature = "canonical-tests")]
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
        .start_defined_study(physical, definition)
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
