// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable occurrence contracts and canonical dependency-scoped execution.
#[cfg(feature = "canonical-tests")]
use super::durable_tests::durable_runtime;
#[cfg(any(
    feature = "canonical-tests",
    feature = "solver-ipopt",
    feature = "solver-kinsol"
))]
use super::tests::sources;
use super::*;
use pse_model::study::*;
#[cfg(feature = "canonical-tests")]
use pse_operations::canonical_studies::StudySummary;
#[cfg(any(feature = "canonical-tests", feature = "solver-ipopt"))]
use std::time::Duration;

#[cfg(any(feature = "canonical-tests", feature = "solver-ipopt"))]
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
        .admit_study(
            &StudyInputs::Explicit(points.to_vec()),
            &StudyOutputDemand::Full {},
            &cancel,
        )
        .await
        .unwrap();
    let definition = runtime.record_study(&definition, &cancel).await.unwrap();
    (
        PackageSources {
            physical,
            modeling: vec![modeling],
        },
        definition,
    )
}
#[cfg(all(feature = "canonical-tests", feature = "solver-kinsol"))]
#[tokio::test]
#[allow(
    clippy::print_stderr,
    reason = "bounded I29 inquiry reports actual owner phase durations without treating overlapping phases as exclusive time"
)]
async fn canonical_worker_claim_and_idle_phase_probe() {
    use pse_diagnostics::observation::{Outcome, Value};
    let runtime = durable_runtime();
    let mut samples = Vec::new();
    async fn observed(runtime: &Runtime) -> (Processed, u128, Vec<(String, u64)>) {
        let operation = runtime.observations().hub().start("inquiry.worker-action");
        let reader = operation.reader();
        let started = std::time::Instant::now();
        let action =
            pse_diagnostics::observation::scope(Some(operation.context()), runtime.work_once())
                .await
                .unwrap();
        let elapsed = started.elapsed().as_micros();
        operation.finish(Outcome::Completed);
        let mut pending = vec![reader];
        let mut visited = std::collections::HashSet::new();
        let mut durations = Vec::new();
        while let Some(reader) = pending.pop() {
            let snapshot = reader.snapshot(Duration::from_secs(60));
            if !visited.insert(snapshot.id) {
                continue;
            }
            for metric in snapshot.metrics {
                if metric.name.starts_with("phase.")
                    && let Value::Count(nanos) = metric.cumulative
                {
                    durations.push((metric.name, nanos));
                }
            }
            pending.extend(
                snapshot
                    .children
                    .iter()
                    .filter_map(|child| runtime.observations().reader(child.id)),
            );
        }
        (action, elapsed, durations)
    }
    for _ in 0..5 {
        let (action, elapsed, durations) = observed(&runtime).await;
        assert!(matches!(action, Processed::Idle));
        assert!(
            !durations
                .iter()
                .any(|(name, _)| name.contains("claim_acquire"))
        );
        samples.push(("idle", elapsed, durations));
    }
    let (_, definition) = admitted(&runtime, |_, _, fixed| {
        (0..5)
            .map(|index| {
                let mut requested = point(fixed, index, vec![], StartPolicy::Fresh);
                assert!(
                    matches!(&requested.operation, OperationRequest::DeclaredCase(_)),
                    "fixed fixture requires a declared case"
                );
                if let OperationRequest::DeclaredCase(operation) = &mut requested.operation {
                    operation.settings.backend = Some(pse_backend_native::solve::Backend::Kinsol);
                }
                requested
            })
            .collect()
    })
    .await;
    let handle = runtime
        .start_recorded_study(definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let mut completed = 0;
    let mut attempts = std::collections::BTreeSet::new();
    for _ in 0..20 {
        let (action, elapsed, durations) = observed(&runtime).await;
        let kind = match action {
            Processed::Ran { record, .. } => {
                assert!(record.attempt.is_ok() && record.manifest.is_some());
                assert!(attempts.insert(record.attempt_id));
                assert!(
                    durations
                        .iter()
                        .any(|(name, _)| name == "phase.managed.worker.claim_acquire.nanoseconds")
                );
                completed += 1;
                "productive"
            }
            Processed::Maintenance { .. } => "maintenance",
            other => panic!("unexpected action before five acknowledged points: {other:?}"),
        };
        samples.push((kind, elapsed, durations));
        if completed == 5 {
            break;
        }
    }
    assert_eq!(completed, 5);
    let status = handle.status().await.unwrap();
    assert!(
        status.points.iter().all(|point| point
            .outcome
            .as_ref()
            .is_some_and(|outcome| outcome.scientific.usable)),
        "original fixed-case outcomes: {:?}",
        status.points
    );
    eprintln!(
        "I29 ordinary local canonical store, five idle passes then five fixed scientific occurrences; action_us and independently owned phase_ns (overlap retained): {samples:?}"
    );
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[tokio::test]
async fn canonical_same_runtime_concurrent_claims_preserve_distinct_native_publication() {
    let runtime = durable_runtime();
    let cancel = crate::CancelSource::new();
    let (physical, modeling) = sources(
        "package algebraic { def Root { var x:Scalar; annotation bounds x(0,2); annotation start x(1); let cost:Scalar=x; annotation objective cost(minimize); eq floor:x>=0.5; annotation report x(\"minimum\"); } }",
    );
    let package = runtime
        .package_from_sources(
            std::slice::from_ref(&modeling),
            runtime.physical_from_sources(&physical, &cancel).await.unwrap(),
            &cancel,
        )
        .await
        .unwrap();
    let root = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|declaration| declaration.name == "Root")
        .unwrap()
        .declaration_id;
    let mut handles = Vec::new();
    for key in [7, 11] {
        let mut requested = point(root, key, vec![], StartPolicy::Fresh);
        let OperationRequest::DeclaredCase(operation) = &mut requested.operation else {
            unreachable!("authored algebraic operation")
        };
        operation.settings.intent = pse_backend_native::solve::SolveIntent::Optimize;
        let local = package
            .admit_study(
                &StudyInputs::Explicit(vec![requested]),
                &StudyOutputDemand::Full {},
                &cancel,
            )
            .await
            .unwrap();
        let recorded = runtime.record_study(&local, &cancel).await.unwrap();
        handles.push(runtime.start_recorded_study(recorded, &cancel).await.unwrap());
    }
    let settings = WorkerSettings {
        maximum_actions: Some(1),
        maximum_in_flight: Some(1),
        until_idle: true,
        ..Default::default()
    };
    // Both callers consume the same first candidate premise. Hold native dispatch
    // until both claim responses return, exposing any borrowed idempotent receipt.
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let left_stop = crate::CancelSource::new();
    let right_stop = crate::CancelSource::new();
    let joined = async {
        tokio::join!(
            worker::synchronized_first_study_claim(
                barrier.clone(),
                runtime.serve(settings, &left_stop),
            ),
            worker::synchronized_first_study_claim(
                barrier,
                runtime.serve(settings, &right_stop),
            ),
        )
    };
    tokio::pin!(joined);
    let (left, right) = match tokio::time::timeout(Duration::from_secs(60), joined.as_mut()).await {
        Ok(result) => result,
        Err(error) => {
            left_stop.cancel();
            right_stop.cancel();
            let drained = joined.await;
            panic!("concurrent claims timed out after owned worker drain: {error}; {drained:?}");
        }
    };
    assert_eq!(left.unwrap(), 1);
    assert_eq!(right.unwrap(), 1);
    // A caller may legitimately spend its one action finalizing its peer's
    // already-settled study. Finish only any remaining ordinary work; native
    // timing must not determine which study supplied that caller's action.
    for _ in 0..4 {
        let mut pending = false;
        for handle in &handles {
            pending |= handle.status().await.unwrap().points.iter().any(|point| !point.settled);
        }
        if !pending {
            break;
        }
        runtime.work_once().await.unwrap();
    }
    let mut attempts = std::collections::BTreeSet::new();
    for handle in handles {
        let status = handle.status().await.unwrap();
        assert_eq!(status.points.len(), 1);
        let point = &status.points[0];
        assert!(point.settled && point.outcome.as_ref().unwrap().scientific.usable);
        assert_eq!(point.outcome.as_ref().unwrap().attempts.len(), 1);
        let attempt = point.attempt.as_deref().unwrap();
        assert!(attempts.insert(attempt.to_owned()));
        let record = runtime.operations().unwrap().record(&point.run, attempt).await.unwrap();
        assert!(record.attempt.is_ok() && record.manifest.is_some());
        let original = record.completion.as_ref().unwrap().completion.as_ref().unwrap();
        assert_eq!(original.solves.len(), 1);
        assert!((original.solves[0].objective.unwrap() - 0.5).abs() < 1e-6);
        runtime.finalize_canonical_study(&handle.study_id().to_string()).await.unwrap();
        assert!(handle.result().await.unwrap().is_some());
    }
    assert_eq!(attempts.len(), 2);
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[tokio::test]
async fn same_worker_publication_handoff_refills_and_bounds_actual_unacknowledged_attempts() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    for mode in 0..3 {
        let runtime = durable_tests::durable_runtime_on(tests::runtime_with_workspace(128 << 20));
        // Root retains its authored two-sided box. Its existing Ipopt route
        // is eligible; KINSOL's sign constraints cannot encode that box.
        let (_, definition) = admitted(&runtime, |root, _, _| {
            (0..3)
                .map(|index| point(root, index, vec![], StartPolicy::Fresh))
                .collect()
        })
        .await;
        let handle = runtime
            .start_recorded_study(definition, &crate::CancelSource::new())
            .await
            .unwrap();
        let stop = crate::CancelSource::new();
        let count = Arc::new(AtomicUsize::new(0));
        let changed = Arc::new(tokio::sync::Notify::new());
        let observer = {
            let count = count.clone();
            let changed = changed.clone();
            Arc::new(move || {
                count.fetch_add(1, Ordering::AcqRel);
                changed.notify_one();
            })
        };
        let (reached, reached_rx) = tokio::sync::oneshot::channel();
        let (release, release_rx) = tokio::sync::oneshot::channel();
        let settings = WorkerSettings {
            maximum_in_flight: Some(1),
            maximum_actions: Some(3),
            until_idle: true,
            ..Default::default()
        };
        let serving = durable::gated_worker_export(
            reached,
            release_rx,
            mode == 1,
            worker::observed_scientific_publication(observer, runtime.serve(settings, &stop)),
        );
        let control = async {
            let reached = matches!(
                tokio::time::timeout(Duration::from_secs(20), reached_rx).await,
                Ok(Ok(()))
            );
            let second = if reached {
                tokio::time::timeout(Duration::from_secs(20), async {
                    loop {
                        let notified = changed.notified();
                        if count.load(Ordering::Acquire) >= 2 {
                            break;
                        }
                        notified.await;
                    }
                })
                .await
                .is_ok()
            } else {
                false
            };
            let pending = handle.status().await;
            let observed = count.load(Ordering::Acquire);
            if mode == 2 || !second {
                stop.cancel();
            }
            let cancelled = if mode == 2 && second {
                // Keep the admitted publisher blocked until the second owner
                // has actually settled cancellation without a payload slot.
                tokio::time::timeout(Duration::from_secs(20), async {
                    loop {
                        let status = handle.status().await?;
                        if let Some(point) = status.points.iter().find(|point| point.settled)
                            && let Some(attempt) = point.attempt.as_deref()
                        {
                            let record = runtime.operations()?.record(&point.run, attempt).await?;
                            break Ok::<_, WorkflowError>((status, record));
                        }
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                })
                .await
                .ok()
                .and_then(Result::ok)
            } else {
                None
            };
            // Every issued native/publication owner drains before assertions.
            let _ = release.send(());
            (reached, second, observed, pending, cancelled)
        };
        let (served, (reached, second, observed, pending, cancelled)) =
            tokio::join!(serving, control);
        runtime.shared.math().shutdown_native_lanes().await;
        assert!(
            reached && second,
            "one actual width-one worker must complete its second scientific/native owner while first export is held: mode={mode}, first_export={reached}, second_ready={second}, ready_count={observed}, served={:?}, pending={pending:?}",
            served.as_ref().map_err(ToString::to_string)
        );
        assert_eq!(
            observed, 2,
            "W Running + W Publishing forbids a third unacknowledged claim"
        );
        let pending = pending.unwrap();
        assert_eq!(
            pending
                .points
                .iter()
                .filter(|point| point.attempt.is_some())
                .count(),
            2
        );
        assert!(
            pending
                .points
                .iter()
                .filter(|point| point.attempt.is_some())
                .all(|point| !point.settled),
            "neither publication nor slot acknowledgement is a terminal ACK"
        );
        if mode == 0 {
            assert_eq!(served.unwrap(), 3);
            let status = handle.status().await.unwrap();
            assert!(status.points.iter().all(|point| {
                point.settled
                    && point
                        .outcome
                        .as_ref()
                        .is_some_and(|outcome| outcome.scientific.usable)
            }));
        } else if mode == 1 {
            assert!(
                served.is_err(),
                "late required publication refusal stays visible"
            );
        } else {
            assert!(served.is_ok(), "cancellation joins both issued attempts");
            let (before_release, record) = cancelled
                .expect("unadmitted cancellation settles while first publisher remains held");
            assert_eq!(
                before_release
                    .points
                    .iter()
                    .filter(|point| point.settled)
                    .count(),
                1
            );
            assert_eq!(
                before_release
                    .points
                    .iter()
                    .filter(|point| point.attempt.is_some())
                    .count(),
                2
            );
            assert!(record.attempt.as_ref().unwrap().terminal());
            assert_eq!(
                record.completion.as_ref().unwrap().state,
                pse_model::generated::enums::AttemptState::Cancelled
            );
            assert!(
                record.completion.as_ref().unwrap().completion.is_none(),
                "cancelled waiting owner never exports original scientific payload"
            );
            assert!(record.solutions.is_empty());
            let descriptors: Vec<pse_operations::canonical_execution::ResultSetDescriptor> =
                serde_json::from_slice(record.manifest.as_ref().unwrap().descriptors.as_slice())
                    .unwrap();
            assert!(
                descriptors.iter().all(|set| set.name == "__progress"),
                "only already-owned progress may exist without a publication slot: {descriptors:?}"
            );
            let status = handle.status().await.unwrap();
            assert!(
                status
                    .points
                    .iter()
                    .filter(|point| point.attempt.is_some())
                    .count()
                    <= 2
            );
        }
    }
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[tokio::test]
async fn same_worker_publication_refills_a_study_created_after_actual_idle_discovery() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let runtime = durable_tests::durable_runtime_on(tests::runtime_with_workspace(128 << 20));
    let requested = |root, key| vec![point(root, key, vec![], StartPolicy::Fresh)];
    let (_, first_definition) = admitted(&runtime, |root, _, _| requested(root, 0)).await;
    // Recording this definition retains its exact source; occurrence creation
    // happens only in start_recorded_study after the real idle event below.
    let (_, late_definition) = admitted(&runtime, |root, _, _| requested(root, 1)).await;
    let first = runtime
        .start_recorded_study(first_definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let stop = crate::CancelSource::new();
    let ready = Arc::new(AtomicUsize::new(0));
    let ready_changed = Arc::new(tokio::sync::Notify::new());
    let idle = Arc::new(AtomicUsize::new(0));
    let idle_changed = Arc::new(tokio::sync::Notify::new());
    let ready_observer = {
        let ready = ready.clone();
        let changed = ready_changed.clone();
        Arc::new(move || {
            ready.fetch_add(1, Ordering::AcqRel);
            changed.notify_one();
        })
    };
    let idle_observer = {
        let idle = idle.clone();
        let changed = idle_changed.clone();
        Arc::new(move || {
            idle.fetch_add(1, Ordering::AcqRel);
            changed.notify_one();
        })
    };
    let (reached, reached_rx) = tokio::sync::oneshot::channel();
    let (release, release_rx) = tokio::sync::oneshot::channel();
    let settings = WorkerSettings {
        maximum_in_flight: Some(1),
        maximum_actions: Some(2),
        poll: Duration::from_millis(10),
        ..Default::default()
    };
    let serving = durable::gated_worker_export(
        reached,
        release_rx,
        false,
        worker::observed_scientific_publication(
            ready_observer,
            worker::observed_idle(idle_observer, runtime.serve(settings, &stop)),
        ),
    );
    let control = async {
        let reached = matches!(
            tokio::time::timeout(Duration::from_secs(20), reached_rx).await,
            Ok(Ok(()))
        );
        let idle_before = idle.load(Ordering::Acquire);
        let observed_idle = if reached {
            tokio::time::timeout(Duration::from_secs(20), async {
                loop {
                    let changed = idle_changed.notified();
                    if idle.load(Ordering::Acquire) > idle_before {
                        break;
                    }
                    changed.await;
                }
            })
            .await
            .is_ok()
        } else {
            false
        };
        let late = if observed_idle {
            Some(
                tokio::time::timeout(Duration::from_secs(20), async {
                    let handle = runtime
                        .start_recorded_study(late_definition, &crate::CancelSource::new())
                        .await?;
                    loop {
                        let changed = ready_changed.notified();
                        if ready.load(Ordering::Acquire) >= 2 {
                            break;
                        }
                        changed.await;
                    }
                    let pending = handle.status().await?;
                    Ok::<_, WorkflowError>((handle, pending))
                })
                .await,
            )
        } else {
            None
        };
        let first_pending = first.status().await;
        if !matches!(&late, Some(Ok(Ok(_)))) {
            stop.cancel();
        }
        let _ = release.send(());
        (reached, observed_idle, late, first_pending)
    };
    let (served, (reached, observed_idle, late, first_pending)) = tokio::join!(serving, control);
    runtime.shared.math().shutdown_native_lanes().await;
    assert!(
        reached && observed_idle,
        "actual idle discovery occurs while the first publisher remains held: first_export={reached}, idle={observed_idle}, ready_count={}, served={:?}, first_pending={first_pending:?}",
        ready.load(Ordering::Acquire),
        served.as_ref().map_err(ToString::to_string)
    );
    let (late, late_pending) = late
        .expect("late occurrence was created")
        .expect("late native owner joined before release")
        .unwrap();
    assert!(!first_pending.unwrap().points[0].settled);
    assert!(late_pending.points[0].attempt.is_some() && !late_pending.points[0].settled);
    assert_eq!(served.unwrap(), 2);
    assert_eq!(ready.load(Ordering::Acquire), 2);
    for handle in [first, late] {
        let status = handle.status().await.unwrap();
        assert!(status.points.iter().all(|point| {
            point.settled
                && point
                    .outcome
                    .as_ref()
                    .is_some_and(|outcome| outcome.scientific.usable)
        }));
    }
}

#[test]
fn study_request_codec_unit_excludes_owner_seed_need_and_bare_overlays() {
    let case = pse_model::generated::identities::DeclarationId::from_bytes([4; 16]);
    let request = StudyRequest {
        output: StudyOutputDemand::Full {},
        version: pse_model::document::Version,
        inputs: StudyInputs::Explicit(vec![point(case, 7, vec![], StartPolicy::Fresh)]),
    };
    let encoded = serde_json::to_value(request).unwrap();
    assert_eq!(encoded["version"], 8);
    assert!(
        encoded["inputs"]["value"][0]["policy"]
            .get("seed_need")
            .is_none()
    );
    let decoded: StudyRequest = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), encoded);
    let mut omitted = encoded.clone();
    omitted["inputs"]["value"][0]["preparation"]["compiler"]["optimization"]
        .as_object_mut()
        .unwrap()
        .remove("evaluator");
    assert!(serde_json::from_value::<StudyRequest>(omitted).is_err());
    for version in [1, 3, 6] {
        let mut historical = encoded.clone();
        historical["version"] = serde_json::json!(version);
        assert!(serde_json::from_value::<StudyRequest>(historical).is_err());
    }
    let mut bare = encoded;
    bare["inputs"]["value"][0]["overlay"] = serde_json::json!({"values":{"x":1}});
    assert!(serde_json::from_value::<StudyRequest>(bare).is_err());
}

#[test]
fn study_variants_codec_unit_retains_explicit_preparation_and_refuses_old_version() {
    let hash = pse_ids::ContentHash::from_bytes([4; 32]);
    let variants = StudyVariants {
        version: pse_model::document::Version,
        source: OperationSource {
            research: None,
            context: None,
            revision: hash.into(),
            physical_context: hash,
        },
        choices: vec![],
        selected: vec![SelectedStudyVariant {
            ordinal: 0,
            point: point(
                pse_model::generated::identities::DeclarationId::from_bytes([4; 16]),
                0,
                vec![],
                StartPolicy::Fresh,
            ),
        }],
    };
    let mut wire = serde_json::to_value(&variants).unwrap();
    assert_eq!(wire["version"], 2);
    let decoded: StudyVariants = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), wire);
    wire["version"] = serde_json::json!(1);
    assert!(serde_json::from_value::<StudyVariants>(wire).is_err());
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
        research: None,
        context: None,
        variants: None,
        version: pse_model::document::Version,
        physical: PhysicalSource {
            revision: "fixture-physical".into(),
            identity: hash,
        },
        modeling_revision: "fixture-revision".into(),
        points: vec![StudyPointDefinition {
            operation: StudyOperation {
                output: StudyOutputDemand::Full {},
                version: pse_model::document::Version,
                source: OperationSource {
                    research: None,
                    context: None,
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
    let WorkflowError::DocumentDecode(decode) = &error else {
        panic!("readmission must retain its native decoder refusal");
    };
    assert_eq!(
        decode.location().stage,
        pse_model::document::DecodeStage::Header
    );
    assert!(decode.inner().to_string().contains("unsupported version 3"));
    assert_eq!(
        decode.location().category,
        pse_model::document::DecodeCategory::Data
    );
    assert!(!error.to_string().contains("class_proof_work"));
    assert_eq!(historical, retained);
    for malformed in [r#"{}"#, r#"{"version":"3"}"#, r#"{"version":8}"#] {
        assert!(matches!(
            StudyDefinition::readmission(malformed),
            Err(WorkflowError::DocumentDecode(_))
        ));
    }
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn canonical_study_cancel_preserves_distinct_unattempted_outcomes() {
    let runtime = durable_runtime();
    let (_, definition) = admitted(&runtime, |_, _, fixed| {
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
        .start_recorded_study(definition, &crate::CancelSource::new())
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
        .unwrap()
        .canonical_run(&retained.run)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        run.terminal_class.as_ref().map(|value| value.as_str()),
        Some("cancelled")
    );
    let restarted = runtime.study(handle.study_id());
    assert_eq!(
        restarted.result().await.unwrap().unwrap().attempt,
        retained.attempt
    );
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn canonical_terminal_inventory_pages_once_accounts_owners_and_refuses_late_corruption() {
    use datafusion::execution::memory_pool::MemoryConsumer;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let runtime = durable_runtime();
    let (_, definition) = admitted(&runtime, |_, _, fixed| {
        (0..65)
            .map(|index| point(fixed, index * 3 + 7, vec![], StartPolicy::Fresh))
            .collect()
    })
    .await;
    let expected = definition
        .points
        .iter()
        .map(|point| point.policy.key)
        .collect::<Vec<_>>();
    let handle = runtime
        .start_recorded_study(definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let pages = Arc::new(AtomicUsize::new(0));
    let count = pages.clone();
    study::TERMINAL_PAGE_OBSERVER
        .scope(
            Box::new(move || {
                count.fetch_add(1, Ordering::Relaxed);
            }),
            async {
                assert!(handle.terminal_inventory().await.unwrap().is_none());
            },
        )
        .await;
    assert_eq!(
        pages.load(Ordering::Relaxed),
        0,
        "open observation never hydrates occurrences"
    );
    handle.cancel().await.unwrap();
    for _ in 0..70 {
        if matches!(runtime.work_once().await.unwrap(), Processed::Idle) {
            break;
        }
    }
    let pool = runtime.shared.pool();
    let before = pool.reserved();
    let count = pages.clone();
    let inventory = study::TERMINAL_PAGE_OBSERVER
        .scope(
            Box::new(move || {
                count.fetch_add(1, Ordering::Relaxed);
            }),
            handle.terminal_inventory(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        pages.load(Ordering::Relaxed),
        3,
        "two populated pages and one end page realize every internal view"
    );
    assert_eq!(
        inventory
            .facts
            .iter()
            .map(|fact| fact.key)
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        inventory
            .status
            .points
            .iter()
            .map(|point| OccurrenceKey(point.point_index))
            .collect::<Vec<_>>(),
        expected
    );
    assert!(inventory.status.points.iter().all(|point| point.settled
        && point.state == StudyPointState::Cancelled
        && point.attempt.is_none()));
    let retained = pool.reserved();
    assert!(retained > before);
    let owner = inventory.owner.clone();
    drop(inventory);
    assert_eq!(
        pool.reserved(),
        retained,
        "retained views share one allocation owner"
    );
    drop(owner);
    assert_eq!(pool.reserved(), before);
    // Evict idle reusable storage before saturating the pool; otherwise pressure
    // recovery can legitimately free it and admit the terminal inventory.
    runtime.shared.math().clear_program_cache();
    let filler = MemoryConsumer::new("test:terminal-inventory-pressure").register(&pool);
    filler
        .try_grow(runtime.shared.budget().memory_limit_bytes.get() - pool.reserved())
        .unwrap();
    pages.store(0, Ordering::Relaxed);
    let count = pages.clone();
    let refused = study::TERMINAL_PAGE_OBSERVER
        .scope(
            Box::new(move || {
                count.fetch_add(1, Ordering::Relaxed);
            }),
            handle.terminal_inventory(),
        )
        .await;
    assert!(refused.is_err());
    assert_eq!(
        pages.load(Ordering::Relaxed),
        0,
        "admission precedes occurrence page allocation"
    );
    drop(filler);
    pse_operations::testing::invalidate_terminal_study_outcome(
        runtime.canonical_store().unwrap(),
        &handle.study_id().to_string(),
        *expected.last().unwrap(),
    )
    .await
    .unwrap();
    pages.store(0, Ordering::Relaxed);
    // The deliberate full-pool pressure may evict independent preparation caches.
    // The late-page failure must return exactly its own starting reservation.
    let before_corrupt = pool.reserved();
    let count = pages.clone();
    let refused = study::TERMINAL_PAGE_OBSERVER
        .scope(
            Box::new(move || {
                count.fetch_add(1, Ordering::Relaxed);
            }),
            handle.terminal_inventory(),
        )
        .await;
    assert!(
        refused.is_err(),
        "no successfully decoded prefix escapes a corrupt later page"
    );
    assert_eq!(pages.load(Ordering::Relaxed), 2);
    assert_eq!(
        pool.reserved(),
        before_corrupt,
        "failed realization releases every reservation"
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
                output: StudyOutputDemand::Full {},
                sources,
                inputs: StudyInputs::Explicit(authored.unwrap()),
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
        after[0].hits, before[0].hits,
        "local definition admission and recording carry the ingress owner without a canonical reload"
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
    assert_eq!(
        after[0].hits,
        before[0].hits + 1,
        "ready occurrence preparation reuses the one canonical physical admission"
    );
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
        .start_recorded_study(definition, &crate::CancelSource::new())
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
        .unwrap()
        .canonical_run(&retained.run)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        run.terminal_class.as_ref().map(|value| value.as_str()),
        Some("partial")
    );
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
    for (attempt_limit, native_started) in [(1, false), (2, false), (2, true)] {
        let runtime = durable_runtime();
        let (_, mut definition) = admitted(&runtime, |_, _, fixed| {
            vec![point(fixed, 5, vec![], StartPolicy::Fresh)]
        })
        .await;
        definition.points[0].policy.attempt_limit = attempt_limit;
        let handle = runtime
            .start_recorded_study(definition, &crate::CancelSource::new())
            .await
            .unwrap();
        let key = pse_operations::canonical_studies::point_key(
            &handle.study_id().to_string(),
            OccurrenceKey(5),
        );
        let scope = runtime
            .canonical_store()
            .unwrap()
            .study_scope(&key)
            .await
            .unwrap();
        let claim = runtime
            .canonical_store()
            .unwrap()
            .claim_study_point(
                &scope,
                None,
                "expired-fixture",
                "departed-worker",
                Duration::from_secs(60),
            )
            .await
            .unwrap();
        if native_started {
            runtime
                .canonical_store()
                .unwrap()
                .mark_study_started(&claim)
                .await
                .unwrap();
        }
        pse_operations::testing::expire_acknowledged_attempt(
            runtime.canonical_store().unwrap(),
            &claim.fence,
        )
        .await
        .unwrap();
        let point = runtime
            .canonical_store()
            .unwrap()
            .canonical_study_point(&key)
            .await
            .unwrap()
            .unwrap();
        let assigned_hint = runtime
            .canonical_store()
            .unwrap()
            .study_assigned_page(&point.study, None)
            .await
            .unwrap();
        assert!(assigned_hint.iter().any(|hint| hint.key == point.key));
        assert!(runtime.recover_study_point(&point.key).await.unwrap());
        let actual = runtime
            .canonical_store()
            .unwrap()
            .canonical_attempt(claim.fence.attempt())
            .await
            .unwrap()
            .unwrap();
        assert!(actual.terminal());
        assert_eq!(
            actual.outcome.as_ref().map(|value| value.as_str()),
            Some("failed")
        );
        let after = runtime
            .canonical_store()
            .unwrap()
            .canonical_study_point(&key)
            .await
            .unwrap()
            .unwrap();
        assert!(!after.assigned);
        assert_eq!(after.settled, native_started || attempt_limit == 1);
        let facts = pse_operations::canonical_studies::point_facts(&after).unwrap();
        assert_eq!(facts.native_started, native_started);
        assert!(!facts.scientific.usable);
        let effect = if native_started {
            EffectState::Unknown
        } else {
            EffectState::Absent
        };
        assert_eq!(
            facts.effect, effect,
            "only an exact fenced pre-dispatch occurrence proves absence"
        );
        let record = runtime
            .operations()
            .unwrap()
            .record(&point.run, claim.fence.attempt())
            .await
            .unwrap();
        let stored = record.completion.as_ref().unwrap();
        assert!(stored.completion.is_none());
        assert_eq!(stored.termination.effect, effect);
        assert_eq!(facts.attempt_count, 1);
        // A delayed assigned-page hint must not project this attempt twice, whether
        // normal policy has concluded it or may claim a genuinely new retry next.
        assert!(!runtime.recover_study_point(&point.key).await.unwrap());
        let unchanged = runtime
            .canonical_store()
            .unwrap()
            .canonical_study_point(&key)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(unchanged.revision, after.revision);
        assert_eq!(unchanged.attempt, after.attempt);
        assert_eq!(unchanged.facts, after.facts);
        assert_eq!(unchanged.outcome, after.outcome);
        assert_eq!(unchanged.assigned, after.assigned);
        assert_eq!(unchanged.settled, after.settled);
    }
}

#[cfg(feature = "solver-ipopt")]
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
        pse_diagnostics::Cancellation::default(),
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
    let StudyOccurrenceResult::Ephemeral(third_run) = study.results[2].as_ref().unwrap() else {
        panic!("original ephemeral occurrence owner required");
    };
    review_qualification_tests::assert_review_transport(
        third_run,
        &[
            ("/steps/0/constructor", "replay-case-constructor.v2"),
            ("/steps/0/decisions", "replay-decisions.v2"),
        ],
    )
    .await;
}

#[cfg(any(feature = "canonical-tests", feature = "solver-ipopt"))]
async fn related_definition(runtime: &Runtime) -> (ModelingPackage, LocalStudy) {
    related_definition_with_primal_accuracy(runtime, None).await
}

#[cfg(any(feature = "canonical-tests", feature = "solver-ipopt"))]
async fn related_definition_with_primal_accuracy(
    runtime: &Runtime,
    primal_absolute_tolerance: Option<f64>,
) -> (ModelingPackage, LocalStudy) {
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
    let requirements = if let Some(absolute) = primal_absolute_tolerance {
        let before_views = runtime.shared.math().preparations().views;
        let model = package
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(
            runtime.shared.math().preparations().views,
            before_views,
            "identity inspection must not prewarm the study's structural solver view"
        );
        [
            ("x", pse_model::generated::enums::NumericalTarget::Variable),
            ("square", pse_model::generated::enums::NumericalTarget::Row),
        ]
        .into_iter()
        .map(|(path, target_kind)| {
            let targets = if target_kind == pse_model::generated::enums::NumericalTarget::Variable {
                model
                    .compiled()
                    .model
                    .symbols
                    .values()
                    .filter(|symbol| {
                        symbol.role
                            == pse_model::generated::enums::ModelingDeclarationKind::Variable
                            && symbol.lineage.path.rsplit('.').next() == Some(path)
                    })
                    .map(|symbol| symbol.id)
                    .collect::<Vec<_>>()
            } else {
                assert_eq!(
                    target_kind,
                    pse_model::generated::enums::NumericalTarget::Row,
                    "fixture requests only original variable and equation accuracy"
                );
                model
                    .compiled()
                    .model
                    .equations
                    .iter()
                    .filter(|equation| equation.lineage.path.rsplit('.').next() == Some(path))
                    .map(|equation| equation.id)
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                targets.len(),
                1,
                "unique actual {target_kind:?} lineage required for {path}"
            );
            let target = targets[0];
            pse_model::numerics::NumericalRequirement {
                requirement_id: pse_ids::named_id(
                    root.as_id(),
                    &format!("one-slot-accuracy:{path}"),
                ),
                model_id: Some(root),
                case_id: None,
                instance_id: None,
                fit_id: None,
                target_id: target,
                target_kind,
                nominal: None,
                scaling_factor: None,
                absolute_tolerance: Some(absolute),
                relative_tolerance: Some(0.),
                shared_engineering_allowance: None,
                engineering_rule_id: None,
                unit_id: Some(unit.as_id()),
                coordinates: pse_model::generated::enums::NumericalCoordinates::Physical,
                priority: 0,
                required: true,
                provenance: "one-slot original square/root accuracy".into(),
            }
        })
        .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
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
        // Freeze explicit target-unit primal accuracy before study admission.
        // A relative engineering fraction alone retains the canonical fallback
        // when this fixture supplies no characteristic engineering scale.
        operation
            .settings
            .numerics
            .requirements
            .extend(requirements.iter().cloned());
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
        .admit_study(
            &StudyInputs::Explicit(points.to_vec()),
            &StudyOutputDemand::Full {},
            &cancel,
        )
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
    let (_, definition) = related_definition(&runtime).await;
    let cancel = crate::CancelSource::new();
    let definition = runtime
        .record_study(&definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let observed = runtime
        .observations()
        .hub()
        .start("qualification.managed-study");
    let observation = observed.reader();
    let study = pse_diagnostics::observation::scope(
        Some(observed.context()),
        fixture.dispatch(&cancel, runtime.execute_study(&definition, 3, &cancel)),
    )
    .await
    .unwrap();
    observed.finish(pse_diagnostics::observation::Outcome::Completed);
    assert!(
        study
            .outcomes
            .iter()
            .all(|outcome| outcome.scientific.usable),
        "{:?}",
        study.outcomes
    );
    let third = study.results[2].as_ref().unwrap();
    let selected = study.diagnostic_selection().await.unwrap().unwrap();
    let inventory = runtime
        .study(selected.study_id)
        .review_inventory(&ReviewInventoryRequest {
            attempts: ReviewAttemptSelection::Retained,
            maximum_members: 64,
        })
        .await
        .unwrap();
    assert!(!inventory.truncated);
    for original in study.results.iter().flatten() {
        let (run, attempt) = original.stored_keys().unwrap();
        assert!(
            inventory
                .members
                .iter()
                .any(|member| member.run.as_deref() == Some(run)
                    && member.attempt.as_deref() == Some(attempt)
                    && member.terminal)
        );
    }
    review_qualification_tests::assert_inventory_transport(
        inventory,
        observation.snapshot(Duration::from_secs(60)),
    );
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
            runtime.registry(),
            &batch,
            &runtime.validation_context(),
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
            runtime.registry(),
            &batch,
            &runtime.validation_context(),
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
async fn canonical_study_explicit_missing_seed_refuses_before_activation() {
    let runtime = durable_runtime();
    let (package, mut definition) = related_definition(&runtime).await;
    definition.points.truncate(1);
    definition.points[0].policy.start = StartPolicy::Explicit {
        role: SeedRole::PrimalSolution,
        seed: pse_operations::mint_id(),
    };
    let before = runtime
        .canonical_store()
        .unwrap()
        .study_page(None)
        .await
        .unwrap();
    let definition = runtime
        .record_study(&definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let failure = runtime
        .start_recorded_study(definition, &crate::CancelSource::new())
        .await
        .unwrap_err();
    assert!(matches!(failure, WorkflowError::SeedRead(_)), "{failure:?}");
    assert_eq!(
        runtime
            .canonical_store()
            .unwrap()
            .study_page(None)
            .await
            .unwrap(),
        before
    );
    assert!(matches!(
        runtime.work_once().await.unwrap(),
        Processed::Idle
    ));
    drop(package);
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[tokio::test]
async fn canonical_study_explicit_retained_input_survives_producer_retirement_and_worker_replacement()
 {
    retained_input_study_survives_producer_retirement_and_worker_replacement(false).await;
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[tokio::test]
async fn canonical_lazy_study_explicit_retained_input_survives_producer_retirement_and_worker_replacement()
 {
    retained_input_study_survives_producer_retirement_and_worker_replacement(true).await;
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
async fn retained_input_study_survives_producer_retirement_and_worker_replacement(lazy: bool) {
    let runtime = durable_tests::durable_runtime_on(tests::runtime_with_workspace(64 << 20));
    let (package, mut definition) = related_definition(&runtime).await;
    definition.points.truncate(2);
    let second_key = definition.points[1].policy.key;
    definition.points[1] = definition.points[0].clone();
    definition.points[1].policy.key = second_key;
    let cancel = crate::CancelSource::new();
    let mut lane = study_operations::StudyPreparationLane::new();
    let prepared = lane
        .prepare(
            &package,
            &definition.points[0].operation,
            &definition.points[0].binding,
            &cancel,
        )
        .await
        .unwrap();
    let original = prepared
        .start(&runtime, &cancel)
        .await
        .unwrap()
        .wait()
        .await
        .unwrap();
    assert!(original.usable(), "{:?}", original.report());
    let producer = original.canonical_run_key().unwrap().to_owned();
    let attempt = original.canonical_attempt_key().unwrap().to_owned();
    let record = runtime
        .operations()
        .unwrap()
        .record(&producer, &attempt)
        .await
        .unwrap();
    assert_eq!(record.solutions.len(), 1);
    let solution = record.solutions[0].1;
    drop(record);
    drop(original);
    drop(prepared);
    lane.close().await.unwrap();
    for point in &mut definition.points {
        point.policy.start = StartPolicy::Explicit {
            role: SeedRole::PrimalSolution,
            seed: solution,
        };
    }
    let before_selection = runtime.shared.math().preparations().views;
    if lazy {
        let scalar = package.quantities.neutral_dimensionless().unwrap();
        let unit = package
            .quantities
            .quantity_type(scalar)
            .unwrap()
            .canonical_unit;
        let selected = definition
            .points
            .iter()
            .enumerate()
            .map(|(ordinal, admitted)| {
                let overlay = PointOverlay {
                    assignments: admitted
                        .binding
                        .entries
                        .values()
                        .map(|entry| {
                            assert_eq!(
                                entry.quantity,
                                scalar.as_id(),
                                "fixture overlay is canonical Scalar"
                            );
                            BindingAssignment {
                                target: BindingTarget::Member(entry.member),
                                value: BindingQuantity {
                                    magnitude: entry.canonical,
                                    quantity: entry.quantity,
                                    unit: unit.as_id(),
                                },
                            }
                        })
                        .collect(),
                };
                assert!(
                    !overlay.assignments.is_empty(),
                    "the lazy seed control also admits a physical overlay"
                );
                SelectedStudyVariant {
                    ordinal: ordinal as u64,
                    point: StudyPoint {
                        operation: admitted.operation.operation.clone(),
                        preparation: admitted.operation.preparation.clone(),
                        overlay,
                        policy: StudyPointPolicy {
                            key: admitted.policy.key,
                            dependencies: admitted.policy.dependencies.clone(),
                            start: admitted.policy.start.clone(),
                            attempt_limit: admitted.policy.attempt_limit,
                        },
                    },
                }
            })
            .collect();
        let alternative = SourceDelta {
            parent: package.revision_id().into(),
            ..Default::default()
        };
        definition = package
            .admit_study(
                &StudyInputs::Lazy(Box::new(StudyVariants {
                    version: pse_model::document::Version,
                    source: OperationSource::of(&package),
                    choices: vec![SourceChoice {
                        alternatives: vec![alternative.clone(), alternative],
                    }],
                    selected,
                })),
                &StudyOutputDemand::Full {},
                &cancel,
            )
            .await
            .unwrap();
        assert!(definition.points.is_empty());
        assert!(
            definition
                .graph()
                .points
                .iter()
                .all(|policy| policy.seed_need == SeedNeed::Pending)
        );
        assert_eq!(
            runtime.shared.math().preparations().views,
            before_selection,
            "lazy descriptor admission must not construct target science to check an explicit seed"
        );
    }
    let policies = definition.graph().points;
    let definition = runtime
        .record_study(&definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let handle = runtime
        .start_recorded_study(definition.clone(), &cancel)
        .await
        .unwrap();
    if lazy {
        assert_eq!(
            runtime.shared.math().preparations().views,
            before_selection,
            "activation owns the seed capsule without constructing selected source/overlay science"
        );
    }
    let study = handle.study_id().to_string();
    let store = runtime.canonical_store().unwrap();
    let capsule = store
        .study_input(&study, &solution.to_string())
        .await
        .unwrap();
    let retained = runtime
        .operations()
        .unwrap()
        .artifact_seed(&capsule)
        .await
        .unwrap();
    assert_eq!(retained.solution(), solution);
    assert_eq!(retained.header().run, producer);
    assert_eq!(retained.header().attempt, attempt);
    let retained_header = retained.header().clone();
    let retained_assessment = retained.assessment().unwrap().clone();
    let retained_primal = match &retained.start().payload {
        pse_backend_native::solve::WarmPayload::Root(primal)
        | pse_backend_native::solve::WarmPayload::Nlp { primal, .. } => primal
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        other => panic!("original root fixture has an unexpected warm payload: {other:?}"),
    };
    drop(retained);
    for point in &policies {
        let native = store
            .canonical_study_point(&pse_operations::canonical_studies::point_key(
                &study, point.key,
            ))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            native.seed_solution.as_deref(),
            Some(solution.to_string().as_str())
        );
        assert_eq!(
            store
                .study_input(&study, native.seed_solution.as_ref().unwrap())
                .await
                .unwrap(),
            capsule,
            "repeated exact external seed shares one copied study input"
        );
        assert!(
            !native.assigned && native.attempt.is_none(),
            "producer retires before any dispatch"
        );
        if lazy {
            let facts = pse_operations::canonical_studies::point_facts(&native).unwrap();
            assert_eq!(
                pse_operations::canonical_studies::point_policy(&native)
                    .unwrap()
                    .seed_need,
                SeedNeed::Pending
            );
            assert!(facts.admitted_seed_need.is_none());
            assert!(native.admission.is_none());
            assert_eq!(facts.attempt_count, 0);
            assert!(
                store.canonical_run(&native.run).await.unwrap().is_none(),
                "unselected science has no published native run"
            );
        }
    }
    drop(package);
    runtime.clear_program_cache();
    let mut retired = false;
    for _ in 0..32 {
        if store.forget_run_results(&producer).await.is_ok() {
            retired = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(
        retired,
        "study owns copied capsule, not a coarse producer obligation"
    );
    assert!(
        runtime.operations().unwrap().seed(solution).await.is_err(),
        "fresh original lookup is unavailable"
    );
    let replacement = runtime
        .clone()
        .with_durability(DurabilitySelection::Durable {
            worker: "replacement-retained-study-input".into(),
            policy: LeasePolicy::default(),
        })
        .unwrap();
    if lazy {
        let owned = replacement
            .operations()
            .unwrap()
            .artifact_seed(&capsule)
            .await
            .unwrap();
        assert_eq!(
            owned.header(),
            &retained_header,
            "copied original header survives producer retirement"
        );
        assert_eq!(
            owned.assessment(),
            Some(&retained_assessment),
            "copied scientific seed permission survives retirement"
        );
        drop(owned);
        let mut selection_lane = study::StudyCandidateLane::new();
        for policy in &policies {
            let key = pse_operations::canonical_studies::point_key(&study, policy.key);
            let scope = store.study_scope(&key).await.unwrap();
            assert!(matches!(
                scope.action(None).unwrap().kind,
                ActionKind::Wait(WaitReason::ScientificAdmission)
            ));
            assert!(
                store
                    .claim_study_point(
                        &scope,
                        None,
                        "unselected-explicit-seed",
                        "replacement",
                        Duration::from_secs(60)
                    )
                    .await
                    .is_err()
            );
            assert!(
                replacement
                    .admit_study_candidate(&scope, &mut selection_lane, &cancel)
                    .await
                    .unwrap()
            );
            let selected = store.canonical_study_point(&key).await.unwrap().unwrap();
            let facts = pse_operations::canonical_studies::point_facts(&selected).unwrap();
            assert_eq!(facts.admitted_seed_need, Some(SeedNeed::Required));
            assert_eq!(
                facts.attempt_count, 0,
                "source/overlay admission consumes no native attempt"
            );
            assert!(!selected.assigned && selected.attempt.is_none());
            let scope = store.study_scope(&key).await.unwrap();
            let unresolved = Some(SeedFact {
                role: SeedRole::PrimalSolution,
                availability: SeedAvailability::Unresolved,
            });
            assert!(matches!(
                scope.action(unresolved.clone()).unwrap().kind,
                ActionKind::Wait(WaitReason::SeedResolution {
                    role: SeedRole::PrimalSolution
                })
            ));
            assert!(
                store
                    .claim_study_point(
                        &scope,
                        unresolved,
                        "unresolved-explicit-seed",
                        "replacement",
                        Duration::from_secs(60)
                    )
                    .await
                    .is_err()
            );
            let prepared = replacement
                .prepare_study_candidate_in_lane(&scope, &mut selection_lane, &cancel)
                .await
                .unwrap();
            assert!(
                matches!(prepared.seed.as_ref(), Some(SeedFact { role: SeedRole::PrimalSolution,
                availability: SeedAvailability::Compatible { seed } }) if *seed == solution)
            );
            assert_eq!(
                scope.action(prepared.seed.clone()).unwrap().kind,
                ActionKind::Start(StartProvenance::Explicit {
                    role: SeedRole::PrimalSolution,
                    seed: solution
                })
            );
            let PreparedStudyOperation::DeclaredCase(case) = &prepared.operation else {
                panic!("original declared case");
            };
            assert!(
                case.stored_seed_owner.is_some(),
                "selected target carries its admitted seed allocation"
            );
            assert_eq!(
                case.solve
                    .source_start(None)
                    .unwrap()
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                retained_primal,
                "authorized explicit start carries the exact retained original warm coordinates"
            );
            drop(prepared);
            let selected = store.canonical_study_point(&key).await.unwrap().unwrap();
            assert!(
                !selected.assigned && selected.attempt.is_none(),
                "compatibility is established before any native claim"
            );
        }
        selection_lane.close().await.unwrap();
        assert!(
            replacement
                .operations()
                .unwrap()
                .seed(solution)
                .await
                .is_err(),
            "preparation cannot resurrect fresh producer lookup"
        );
    }
    for _ in 0..8 {
        if handle
            .status()
            .await
            .unwrap()
            .points
            .iter()
            .all(|point| point.settled)
        {
            break;
        }
        assert!(
            !matches!(replacement.work_once().await.unwrap(), Processed::Idle),
            "replacement must consume retained study input"
        );
    }
    let status = handle.status().await.unwrap();
    assert_eq!(status.points.len(), 2);
    for point in &status.points {
        assert!(point.settled);
        let outcome = point.outcome.as_ref().unwrap();
        assert!(outcome.scientific.usable, "{outcome:?}");
        assert_eq!(
            outcome.start,
            Some(StartProvenance::Explicit {
                role: SeedRole::PrimalSolution,
                seed: solution
            })
        );
        assert_eq!(
            outcome.key.0, point.point_index,
            "actual provenance belongs to each occurrence"
        );
        let result = replacement
            .operations()
            .unwrap()
            .record(&point.run, point.attempt.as_ref().unwrap())
            .await
            .unwrap();
        assert_eq!(
            result.solutions.len(),
            1,
            "replacement executes a native corrected solve"
        );
    }
    assert_eq!(
        store
            .study_input(&study, &solution.to_string())
            .await
            .unwrap(),
        capsule
    );
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn canonical_study_summary_live_owner_and_expired_writer_rebuild_without_science() {
    let runtime = durable_runtime();
    let (package, mut definition) = related_definition(&runtime).await;
    definition.points.truncate(1);
    let definition = runtime
        .record_study(&definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let handle = runtime
        .start_recorded_study(definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let key = handle.study_id().to_string();
    let store = runtime.canonical_store().unwrap();
    assert!(store.study_has_pending(&key).await.unwrap());
    let parent_key = store.canonical_study(&key).await.unwrap().unwrap().run;
    let before_parent = store.canonical_run(&parent_key).await.unwrap().unwrap();
    assert!(!runtime.finalize_canonical_study(&key).await.unwrap());
    assert!(
        pse_model::SemanticEq::semantic_eq(
            &store.canonical_run(&parent_key).await.unwrap().unwrap(),
            &before_parent
        ),
        "pending discovery does not claim or alter summary authority"
    );
    assert!(
        handle.result().await.unwrap().is_none(),
        "open study has no sealed result"
    );
    assert!(matches!(
        runtime.work_once().await.unwrap(),
        Processed::Ran { .. }
    ));
    assert!(!store.study_has_pending(&key).await.unwrap());
    let projected = store.study_outcome_page(&key, None).await.unwrap();
    assert_eq!(projected.len(), 1);
    let projected = &projected[0];
    let original = store
        .canonical_study_point(&pse_operations::canonical_studies::point_key(
            &key,
            OccurrenceKey(projected.occurrence.try_into().unwrap()),
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(projected.descriptor, original.descriptor);
    assert_eq!(
        projected.policy().unwrap(),
        pse_operations::canonical_studies::point_policy(&original).unwrap()
    );
    assert_eq!(
        serde_json::to_value(projected.outcome().unwrap()).unwrap(),
        serde_json::to_value(
            pse_operations::canonical_studies::point_outcome(&original)
                .unwrap()
                .unwrap()
        )
        .unwrap()
    );
    assert!(
        store
            .study_outcome_page(&key, Some(projected.ordinal))
            .await
            .unwrap()
            .is_empty()
    );
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
            Duration::from_secs(60),
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
            pse_operations::canonical_execution::ResultPayload::new([1].to_vec()).unwrap(),
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
    pse_operations::testing::expire_acknowledged_attempt(store, &fence)
        .await
        .unwrap();
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
    assert!(previous.terminal());
    assert_eq!(
        previous.outcome.as_ref().map(|value| value.as_str()),
        Some("failed")
    );
    let recovered = runtime
        .operations()
        .unwrap()
        .record(&header.run, fence.attempt())
        .await
        .unwrap();
    let stored = recovered.completion.as_ref().unwrap();
    assert!(stored.completion.is_none());
    assert_eq!(
        stored.termination.effect,
        EffectState::Absent,
        "immutable summary-only authority grants no native dispatch capability"
    );
    let final_attempt = store
        .canonical_attempt(status.result_attempt.as_ref().unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        final_attempt.outcome.as_ref().map(|value| value.as_str()),
        Some("succeeded")
    );
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
    let definition = runtime
        .record_study(&definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let handle = runtime
        .start_recorded_study(definition, &crate::CancelSource::new())
        .await
        .unwrap();
    assert!(matches!(
        runtime.work_once().await.unwrap(),
        Processed::Ran { .. }
    ));
    let key = handle.study_id().to_string();
    let store = runtime.canonical_store().unwrap();
    let study = store.canonical_study(&key).await.unwrap().unwrap();
    let fence = store
        .begin_study_finalization(
            &StudySummary::from(&study),
            "summary-before-cancel",
            Duration::from_secs(60),
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
    // SAFETY: this isolated negative fixture deliberately presents a stale success;
    // cancellation must fence it before any scientific publication can be admitted.
    let rejected_parent = unsafe {
        store
            .seal_attempt(
                &manifest,
                "stale-parent-success",
                pse_operations::canonical_execution::TerminalClass::Succeeded,
                &[1],
                pse_operations::canonical_execution::SeedEligibility::Unavailable,
            )
            .await
    }
    .unwrap_err();
    assert!(
        matches!(rejected_parent,pse_operations::canonical::CanonicalError::Refused {
        code:pse_operations::canonical::RefusalCode::ExecutionSummaryCancellationWinsTerminalSeal,..
    }),
        "the acknowledged cancellation, not a transport error, must refuse success"
    );
    // SAFETY: the same cancelled summary fixture must refuse the stale owner's
    // success assertion before admitting a study conclusion or exposing results.
    let rejected_summary = unsafe {
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
    .unwrap_err();
    assert!(
        matches!(
            rejected_summary,
            pse_operations::canonical::CanonicalError::Refused {
                code: pse_operations::canonical::RefusalCode::StudySummaryCancellationWinsAdmission,
                ..
            }
        ),
        "summary admission must retain the exact cancellation refusal"
    );
    pse_operations::testing::expire_acknowledged_attempt(store, &fence)
        .await
        .unwrap();
    assert!(runtime.finalize_canonical_study(&key).await.unwrap());
    let status = handle.status().await.unwrap();
    assert_eq!(status.state, StudyState::Concluded);
    assert!(status.cancelled);
    let parent = store
        .canonical_attempt(status.result_attempt.as_ref().unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        parent.outcome.as_ref().map(|value| value.as_str()),
        Some("cancelled")
    );
    let recovered = runtime
        .operations()
        .unwrap()
        .record(&study.run, fence.attempt())
        .await
        .unwrap();
    let stored = recovered.completion.as_ref().unwrap();
    assert_eq!(
        stored.state,
        pse_model::generated::enums::AttemptState::Cancelled
    );
    assert!(stored.completion.is_none());
    assert_eq!(
        stored.termination.effect,
        EffectState::Absent,
        "acknowledged cancellation retains the exact summary-only absence proof"
    );
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
#[tokio::test]
async fn explicit_recording_preserves_exact_retained_sources_and_lazy_selection_without_construction()
 {
    let receiver = durable_runtime();
    let local_runtime = tests::runtime_with_workspace(64 << 20);
    let (package, explicit) = related_definition(&local_runtime).await;
    assert!(package.canonical_revision().is_none());
    let original = &explicit.points[0];
    let requested = StudyPoint {
        operation: original.operation.operation.clone(),
        preparation: original.operation.preparation.clone(),
        overlay: PointOverlay::default(),
        policy: StudyPointPolicy {
            key: OccurrenceKey(17),
            dependencies: vec![],
            start: StartPolicy::Fresh,
            attempt_limit: 1,
        },
    };
    let local = package
        .admit_study(
            &StudyInputs::Lazy(Box::new(StudyVariants {
                version: pse_model::document::Version,
                source: OperationSource::of(&package),
                choices: vec![SourceChoice {
                    alternatives: vec![SourceDelta {
                        parent: package.revision_id().into(),
                        removals: vec![
                            pse_model::generated::identities::DeclarationId::from_bytes([0; 16]),
                        ],
                        ..Default::default()
                    }],
                }],
                selected: vec![SelectedStudyVariant {
                    ordinal: 0,
                    point: requested,
                }],
            })),
            &StudyOutputDemand::Full {},
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let before = local_runtime.shared.math().preparations().views;
    assert!(matches!(
        local_runtime
            .record_study(&local, &crate::CancelSource::new())
            .await,
        Err(WorkflowError::RecordingUnavailable)
    ));
    let recorded = receiver
        .record_study(&local, &crate::CancelSource::new())
        .await
        .unwrap();
    assert_eq!(local_runtime.shared.math().preparations().views, before);
    assert!(recorded.points.is_empty());
    assert_eq!(recorded.point_count(), 1);
    let retained = recorded.variants.as_ref().unwrap();
    assert_eq!(
        retained.variants.selected[0].point.policy.key,
        OccurrenceKey(17)
    );
    assert_eq!(
        retained.variants.choices[0].alternatives[0].parent,
        recorded.modeling_revision
    );
    assert_ne!(recorded.modeling_revision, local.modeling_revision);
    let (physical, _) = sources(CASES);
    let stored = receiver
        .operations()
        .unwrap()
        .sources(&recorded.physical)
        .await
        .unwrap();
    assert_eq!(
        &**stored, &physical,
        "recording retains exact original bytes, including comments and formatting"
    );
    let context = receiver
        .physical_source(&recorded.physical, &crate::CancelSource::new())
        .await
        .unwrap();
    let revision = receiver
        .canonical_store()
        .unwrap()
        .revision(&recorded.modeling_revision)
        .await
        .unwrap()
        .unwrap();
    let reopened = receiver
        .modeling_revision(revision, context, Default::default())
        .await
        .unwrap();
    assert_eq!(
        OperationSource::of(&reopened),
        OperationSource::of(&package)
    );
    assert!(
        package.canonical_revision().is_none(),
        "recording never mutates the local owner"
    );
    assert_eq!(
        local.variants.as_ref().unwrap().variants.choices[0].alternatives[0].parent,
        package.revision_id()
    );
}

#[cfg(feature = "canonical-tests")]
async fn stored_value_definition(runtime: &Runtime, count: u32) -> StudyDefinition {
    let (_, mut admitted) = related_definition(runtime).await;
    let first = admitted.points[0].clone();
    admitted.points = (0..count)
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
        .collect();
    runtime
        .record_study(&admitted, &crate::CancelSource::new())
        .await
        .unwrap()
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
    let (handle, constructions) = study_operations::counted_declared_admissions(
        runtime.start_recorded_study(definition, &crate::CancelSource::new()),
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
        .unwrap()
        .canonical_study(&key)
        .await
        .unwrap()
        .unwrap();
    assert!(header.active);
    assert_eq!(header.point_count, 1000);
    assert_eq!(header.next_ordinal, 1000);
    let first = runtime
        .canonical_store()
        .unwrap()
        .canonical_study_point(&pse_operations::canonical_studies::point_key(
            &key,
            OccurrenceKey(0),
        ))
        .await
        .unwrap()
        .unwrap();
    let repeated = runtime
        .canonical_store()
        .unwrap()
        .canonical_study_point(&pse_operations::canonical_studies::point_key(
            &key,
            OccurrenceKey(20),
        ))
        .await
        .unwrap()
        .unwrap();
    let last = runtime
        .canonical_store()
        .unwrap()
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
        let (result, constructions) = study_operations::counted_declared_admissions(
            runtime.start_recorded_study(definition, &crate::CancelSource::new()),
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
    let (result, constructions) = study_operations::counted_declared_admissions(
        runtime.start_recorded_study(definition, &crate::CancelSource::new()),
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
    let (result, constructions) = study_operations::counted_declared_admissions(
        runtime.start_recorded_study(definition, &crate::CancelSource::new()),
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
async fn recorded_durable_study_refuses_precancelled_creation() {
    let runtime = durable_runtime();
    let (_, definition) = related_definition(&runtime).await;
    let cancel = crate::CancelSource::new();
    let definition = runtime
        .record_study(&definition, &crate::CancelSource::new())
        .await
        .unwrap();
    cancel.cancel();
    let (result, constructions) = study_operations::counted_declared_admissions(
        runtime.execute_study(&definition, 3, &cancel),
    )
    .await;
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
    let (result, constructions) = study_operations::counted_declared_admissions(
        study_operations::cancel_after_declared_admission(
            runtime.start_recorded_study(definition, &cancel),
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
    })
    .unwrap();
    let registry = pse_schema::shared_registry().unwrap();
    let sessions = Arc::new(
        shared
            .session_factory(pse_engine::session::native_engine_profile())
            .unwrap(),
    );
    Runtime::from_shared(
        shared,
        sessions.bind_model(registry.clone()).unwrap(),
        tests::canonical_deployment(),
    )
    .unwrap()
    .with_durability(DurabilitySelection::Ephemeral)
    .unwrap()
}

#[cfg(feature = "solver-ipopt")]
async fn parallel_study_definition(
    runtime: &Runtime,
    count: usize,
    failing: Option<usize>,
) -> (ModelingPackage, LocalStudy) {
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
        .admit_study(
            &StudyInputs::Explicit(points.to_vec()),
            &StudyOutputDemand::Full {},
            &cancel,
        )
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
    tasks: Vec<Arc<std::sync::atomic::AtomicBool>>,
    entered: Vec<std::thread::ThreadId>,
    released: std::collections::HashSet<std::thread::ThreadId>,
    release_all: bool,
    active: usize,
    maximum: usize,
}
#[cfg(feature = "solver-ipopt")]
impl StudyNativeProbe {
    fn enter(&self, cancel: &Arc<std::sync::atomic::AtomicBool>) {
        let thread = std::thread::current().id();
        let mut state = self.state.lock().unwrap();
        // Neutral parking/reset keeps the completed task's original cancellation
        // owner. A fresh occurrence gets a new owner even on the same native thread.
        if state.tasks.iter().any(|task| Arc::ptr_eq(task, cancel)) {
            return;
        }
        state.tasks.push(cancel.clone());
        if !state.entered.contains(&thread) {
            state.entered.push(thread);
        }
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
        // The watchdog can panic while holding this test gate. Cleanup must still
        // release the other native owners without a second panic during unwinding.
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .release_all = true;
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
    let run = crate::math::observed_native_task_entry(
        Arc::new(move |flag| observer.enter(flag)),
        package.study(&definition, 20, &cancel),
    );
    let controls = async {
        probe
            .wait(|state| state.entered.len() == 16 && state.active == 16)
            .await;
        probe.release_following();
        probe.wait(|state| state.tasks.len() == 20).await;
        let state = probe.state.lock().unwrap();
        assert_eq!(
            state.entered.len(),
            16,
            "tail occurrences reuse admitted native threads"
        );
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
    let run = crate::math::observed_native_task_entry(
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
    assert_eq!(probe.state.lock().unwrap().tasks.len(), 16);
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
    let (_, definition) = admitted(&runtime, |_, _, fixed| {
        (0..16)
            .map(|index| point(fixed, (index + 1) * 7, vec![], StartPolicy::Fresh))
            .collect()
    })
    .await;
    let handle = runtime
        .start_recorded_study(definition, &crate::CancelSource::new())
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
async fn ephemeral_study_single_population_slot_lends_preparation_and_completes() {
    let runtime = study_runtime_with_population(1, 1);
    let (package, definition) =
        related_definition_with_primal_accuracy(&runtime, Some(1e-12)).await;
    let entered = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observer = entered.clone();
    let (result, preparations) = tokio::time::timeout(
        Duration::from_secs(60),
        crate::math::counted(crate::math::observed_native_entry(
            Arc::new(move |_| {
                observer.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }),
            package.study(&definition, 3, &crate::CancelSource::new()),
        )),
    )
    .await
    .expect("one population lane must lend preparation without deadlock");
    let report = result.unwrap();
    assert_eq!(report.outcomes.len(), definition.points.len());
    assert!(
        report
            .outcomes
            .iter()
            .all(|outcome| outcome.scientific.usable),
        "{:?}",
        report.outcomes
    );
    assert!(entered.load(std::sync::atomic::Ordering::Relaxed) > 0);
    assert!(
        preparations.views > 0,
        "the single lane performs actual preparation before native execution"
    );
    for (index, parameter) in [4_f64, 4.04, 4.08].into_iter().enumerate() {
        let RunReport::Modeling(results) =
            report.results[index].as_ref().unwrap().report().unwrap()
        else {
            panic!("the continuation must return its actual modeling result");
        };
        assert_eq!(results.len(), 1);
        let result = &results[0];
        let x = result
            .prepared
            .model
            .model
            .compiled()
            .model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path == "x" || symbol.lineage.path.ends_with(".x"))
            .unwrap()
            .id;
        let value = result.values.scalars[&x];
        let targets = result
            .prepared
            .solve
            .numerics()
            .targets
            .iter()
            .filter(|target| {
                target.id == x || target.kind == pse_model::generated::enums::NumericalTarget::Row
            })
            .map(|target| {
                (
                    target.id,
                    target.kind,
                    target.absolute,
                    target.relative,
                    target.budget,
                )
            })
            .collect::<Vec<_>>();
        eprintln!(
            "one-slot-continuation index={index} parameter={parameter:.17} value={value:.17} targets={targets:?}"
        );
        assert_eq!(targets.len(), 2);
        assert!(
            targets.iter().all(|target| target.4 <= 1e-10),
            "explicit original-space budgets must support the independent oracle: index={index} targets={targets:?}"
        );
        assert!(
            (value - parameter.sqrt()).abs() < 1e-8,
            "one-slot positive root: index={index} parameter={parameter:.17} value={value:.17}"
        );
        assert!(
            (value * value - parameter).abs() < 1e-8,
            "original square residual: index={index} parameter={parameter:.17} value={value:.17}"
        );
        if index > 0 {
            assert!(matches!(report.outcomes[index].start,
                Some(StartProvenance::Continuation { predecessor, .. })
                    if predecessor == definition.points[index - 1].policy.key));
        }
        if index == 2 {
            assert_eq!(
                result.prepared.solve.entry_origin(false),
                pse_model::strategy::StartOrigin::Predicted
            );
            assert_eq!(
                result.strategy.as_ref().unwrap().starts[0],
                pse_model::strategy::StartOrigin::Predicted,
                "the single retained population must lend the actual secant screening step"
            );
        }
    }
}

#[cfg(feature = "canonical-tests")]
struct ManagedStudyFixture {
    runtime: Runtime,
    directory: std::path::PathBuf,
    controls_resource: String,
    nonce: String,
    worker_pid: u32,
    primary_owner: serde_json::Value,
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
        Self::start_on(hold_entries, tests::durable_runtime(), 16).await
    }

    async fn start_on(hold_entries: bool, runtime: Runtime, entries: usize) -> Self {
        use std::os::unix::fs::DirBuilderExt;
        // This is the observer's separately placed pool. The managed primary retains
        // the exact 128 GiB/16 CPU/32 population profile recorded in the state.
        let store = runtime.canonical_store().unwrap();
        let context = managed_read_json(
            &store
                .deployment_state()
                .join(".contexts")
                .join(format!("{}.json", store.database())),
        );
        let state = std::path::PathBuf::from(context["receiver_state"].as_str().unwrap());
        let selected = NativeExecutionSelection::from_store(store).unwrap();
        let allocation = selected.allocation();
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
        let receiver = selected.receiver().unwrap();
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
                "nonce": nonce, "canonical_database": store.database(), "entries": entries, "entry_timeout_ms": 90_000,
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
            primary_owner: serde_json::Value::Null,
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
                .arg(fixture.runtime.canonical_store().unwrap().database())
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
        fixture.primary_owner = ready["primary_owner"].clone();
        assert_eq!(
            ready["canonical_database"],
            fixture.runtime.canonical_store().unwrap().database()
        );
        let marker = managed_read_json(&state.join("primary-receiver.json"));
        assert_eq!(
            ready["diagnostics"]["identity"],
            marker["diagnostics"]["identity"]
        );
        assert_eq!(
            ready["diagnostics"]["checkpoint"],
            marker["diagnostics"]["checkpoint"]
        );
        assert_eq!(
            ready["diagnostics"]["canonical_database"],
            marker["canonical_database"]
        );
        assert_eq!(ready["diagnostics"]["nonce"], marker["nonce"]);
        assert_eq!(
            marker["canonical_database"],
            fixture.runtime.canonical_store().unwrap().database()
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
                    self.runtime.canonical_store().unwrap().database()
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

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[allow(
    unsafe_code,
    reason = "finite transport-only rows assert Partial; separate managed roots retain independent original numerical assertions"
)]
#[tokio::test]
async fn managed_primary_multipage_read_overlaps_compiler_leases_and_original_science() {
    use datafusion::arrow::array::{Array, Float64Array};
    use pse_columnar::{CancellationToken, MemoryConsumer};
    use pse_compiler::workspace::{CompilerContext, WorkspaceLimits};
    use pse_operations::canonical_execution::{RunRequest, TerminalClass};
    use pse_relations::generated::{enums::NativeMetricKind, runtime::solve_metrics};

    let local = tests::runtime_on(
        256 << 20,
        crate::math::MathPolicy {
            workspace_bytes: 64 << 20,
            worker_bytes: 64 << 20,
            artifact_bytes: 16 << 20,
            foreign_bytes: 8 << 20,
            inner_session_bytes: 0,
            ..Default::default()
        },
    );
    let fixture =
        ManagedStudyFixture::start_on(true, durable_tests::durable_runtime_on(local), 3).await;
    let runtime = &fixture.runtime;
    let store = runtime.canonical_store().unwrap();
    let pool = runtime.shared.pool();
    let key = format!("multipage-managed-{}", fixture.nonce);
    let revision = store.edit(&key, None, "transport-only", &[]).await.unwrap();
    store
        .begin_run(&RunRequest {
            key: key.clone(),
            revision,
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        })
        .await
        .unwrap();
    let fence = store
        .claim_run(&key, "claim", "transport-only", Duration::from_secs(60))
        .await
        .unwrap();
    let run_id = pse_operations::mint_id();
    let mut batches = Vec::new();
    for step in 0..65 {
        let mut builder = solve_metrics::Builder::with_registry(
            runtime.registry(),
            1,
            &runtime.validation_context(),
        )
        .unwrap();
        builder
            .push(solve_metrics::Row {
                run_id,
                step,
                namespace: "transport-only".into(),
                name: "temperature".into(),
                kind: NativeMetricKind::Real,
                real: Some(273.15 + step as f64),
                integer: None,
                boolean: None,
                text: None,
                unavailable: None,
            })
            .unwrap();
        batches.push(builder.finish().unwrap());
    }
    result_projection::store_result_table(
        store,
        &fence,
        solve_metrics::RELATION_ID,
        ResultCursor::from_retained(
            batches[0].batch().schema(),
            solve_metrics::RELATION_ID,
            &batches,
        )
        .unwrap(),
        &pool,
        &durable::PublicationAcks::new(&pool),
    )
    .await
    .unwrap();
    drop(batches);
    let closed = store.close_result_ingestion(&fence, "close").await.unwrap();
    let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
    // SAFETY: these observations establish transport availability only.
    unsafe {
        store.seal_attempt(
            &manifest,
            "seal",
            TerminalClass::Partial,
            &[1],
            pse_operations::canonical_execution::SeedEligibility::Unavailable,
        )
    }
    .await
    .unwrap();
    let (_, local) = parallel_study_definition(runtime, 3, None).await;
    let cancel = crate::CancelSource::new();
    let definition = runtime.record_study(&local, &cancel).await.unwrap();
    drop(local);
    let science = runtime.execute_study(&definition, 3, &cancel);
    let serving = async {
        let held = fixture.observation(|value| value["active"] == 3).await;
        assert_eq!(held["entries"].as_array().unwrap().len(), 3);
        let physical = tests::physical();
        let workspace = || {
            runtime
                .native()
                .workspace(
                    CompilerContext {
                        quantities: physical.quantities.clone(),
                        preconditions: physical.preconditions.clone(),
                        providers: Default::default(),
                    },
                    WorkspaceLimits::default(),
                )
                .unwrap()
        };
        let before = pool.reserved();
        let first = workspace();
        let second = workspace();
        assert_eq!(pool.reserved() - before, 128 << 20);
        let baseline = pool.reserved();
        let settle_read = || async {
            // The issued release retains this read's metadata grant through its
            // SDK suffix. Other live study readers are intentionally not drained.
            tokio::time::timeout(Duration::from_secs(10), async {
                while pool.reserved() != baseline {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            })
            .await
            .expect("owned result-read suffix must release its grant");
        };
        let stop = CancellationToken::new();
        let mut reader = runtime
            .results(
                fence.run(),
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                65,
                stop.clone(),
            )
            .await
            .unwrap();
        let snapshot = reader.snapshot();
        assert_eq!(snapshot.blocks().len(), 65);
        let alias = snapshot.clone();
        let mut rows = 0;
        while let Some(batch) = reader.next_batch().await.unwrap() {
            let values = batch
                .column_by_name("real")
                .unwrap()
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap();
            assert_eq!(batch.num_rows(), 1);
            assert_eq!(values.value(0), 273.15 + rows as f64);
            rows += 1;
        }
        assert_eq!(rows, 65);
        drop(reader);
        drop(snapshot);
        assert!(pool.reserved() > baseline);
        drop(alias);
        settle_read().await;
        assert_eq!(pool.reserved(), baseline);
        let pressure = MemoryConsumer::new("test:managed-result-pressure").register(&pool);
        let remaining = (256 << 20) - pool.reserved();
        let pressure_bytes = remaining
            .checked_sub(pse_operations::canonical::MetadataResponse::Selection.allocation_bytes())
            .expect(
                "original observer pool must fit a complete selection with both compiler leases",
            )
            + 1;
        pressure.try_grow(pressure_bytes).unwrap();
        assert!(
            runtime
                .results(
                    fence.run(),
                    fence.attempt(),
                    "runtime.solve_metrics",
                    0,
                    65,
                    CancellationToken::new()
                )
                .await
                .is_err()
        );
        drop(pressure);
        let mut reader = runtime
            .results(
                fence.run(),
                fence.attempt(),
                "runtime.solve_metrics",
                0,
                65,
                stop.clone(),
            )
            .await
            .unwrap();
        stop.cancel();
        assert!(reader.next_batch().await.is_err());
        drop(reader);
        settle_read().await;
        assert_eq!(pool.reserved(), baseline);
        drop(first);
        drop(second);
        assert_eq!(pool.reserved(), before);
        fixture.signal("release.json");
    };
    let (report, ()) = fixture
        .dispatch(&cancel, async { tokio::join!(science, serving) })
        .await;
    let report = report.unwrap();
    assert_eq!(report.outcomes.len(), 3);
    assert!(
        report
            .outcomes
            .iter()
            .all(|outcome| outcome.scientific.usable)
    );
    assert_eq!(report.results.len(), 3);
    assert_eq!(
        report
            .results
            .iter()
            .map(|result| result.as_ref().unwrap().stored_keys().unwrap())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3,
        "each managed occurrence retains its own exact result identity"
    );
    for result in &report.results {
        let (run, attempt) = result.as_ref().unwrap().stored_keys().unwrap();
        let reopened = runtime
            .operations()
            .unwrap()
            .record(run, attempt)
            .await
            .unwrap();
        assert_eq!(reopened.solutions.len(), 1);
        let mut variables = runtime
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
        let mut original_checked = false;
        while let Some(batch) = variables.next_batch().await.unwrap() {
            let view = pse_relations::generated::runtime::solve_variables::View::try_from_batch_with_registry(runtime.registry(), &batch, &runtime.validation_context()).unwrap();
            for row in view.rows().unwrap() {
                if row.parameter {
                    continue;
                }
                if let Some(value) = row.value {
                    let allowance = row.tolerance.unwrap();
                    assert!(value.is_finite() && allowance.is_finite() && allowance > 0.0);
                    assert!((value - 2.0).abs() <= allowance);
                    assert!((value * value - 4.0).abs() <= allowance * (4.0 + allowance));
                    original_checked = true;
                }
            }
        }
        assert!(original_checked, "actual managed original root retained");
    }
    drop(report);
    fixture.finish().await;
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[tokio::test]
async fn managed_primary_death_returns_incomplete_study_without_fabricated_terminal() {
    let mut fixture = ManagedStudyFixture::start_on(true, tests::durable_runtime(), 1).await;
    let runtime = &fixture.runtime;
    let (_, local) = parallel_study_definition(runtime, 1, None).await;
    let cancel = crate::CancelSource::new();
    let definition = runtime.record_study(&local, &cancel).await.unwrap();
    let science = runtime.execute_study(&definition, 1, &cancel);
    let death = async {
        fixture.observation(|value| value["active"] == 1).await;
        let unit = fixture.primary_owner["scope"]["unit"].as_str().unwrap();
        let observed = tokio::process::Command::new("systemctl")
            .args(["--user", "show", "--property=InvocationID", "--value", unit])
            .output()
            .await
            .unwrap();
        assert!(observed.status.success());
        assert_eq!(
            std::str::from_utf8(&observed.stdout).unwrap().trim(),
            fixture.primary_owner["scope"]["invocation"]
                .as_str()
                .unwrap()
        );
        // This fixture owns the isolated database's exact primary invocation.
        // Deliberate death leaves durable work incomplete rather than inventing an ACK.
        assert!(
            tokio::process::Command::new("systemctl")
                .args([
                    "--user",
                    "kill",
                    "--kill-whom=all",
                    "--signal=SIGKILL",
                    unit
                ])
                .status()
                .await
                .unwrap()
                .success()
        );
    };
    let (result, ()) = tokio::time::timeout(Duration::from_secs(45), async {
        tokio::join!(science, death)
    })
    .await
    .unwrap();
    let study = match result {
        Err(WorkflowError::StudyExecutionUnavailable { study }) => study,
        result => panic!("dead primary must preserve an incomplete study identity: {result:?}"),
    };
    assert!(runtime.study(study).result().await.unwrap().is_none());
    assert!(!std::path::Path::new(&format!("/proc/{}", fixture.worker_pid)).exists());
    fixture.finished = true;
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
                && value["canonical_database"] == self.runtime.canonical_store().unwrap().database()
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
                self.runtime.canonical_store().unwrap().database(),
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
    let (_, definition) = parallel_study_definition(runtime, 16, None).await;
    assert!(
        definition
            .points
            .windows(2)
            .all(|points| points[0].binding_hash == points[1].binding_hash)
    );
    let cancel = crate::CancelSource::new();
    let definition = runtime
        .record_study(&definition, &crate::CancelSource::new())
        .await
        .unwrap();
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
            tokio::join!(runtime.execute_study(&definition, 16, &cancel), control)
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
                runtime.registry(),
                &batch,
                &runtime.validation_context(),
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
async fn managed_primary_sixteen_survive_caller_disconnect_until_explicit_cancel() {
    let fixture = ManagedStudyFixture::start(true).await;
    let runtime = &fixture.runtime;
    let (_, definition) = parallel_study_definition(runtime, 16, None).await;
    let definition = runtime
        .record_study(&definition, &crate::CancelSource::new())
        .await
        .unwrap();
    // Use an independent physical client as well as an independent caller token;
    // cloning a deployment would keep the same driver's transport-close authority.
    let store = runtime.canonical_store().unwrap();
    let options = pse_operations::canonical::CanonicalOptions::from_state_for_database(
        store.deployment_state(),
        Some(store.database()),
    )
    .unwrap();
    let independent_store = pse_operations::canonical::CanonicalStore::connect(&options)
        .await
        .unwrap();
    independent_store.open().await.unwrap();
    let independent = Runtime::from_shared(
        runtime.shared.clone(),
        runtime.model.clone(),
        CanonicalDeployment::new(
            independent_store,
            runtime.canonical().unwrap().attestation(),
            runtime.canonical().unwrap().producer().cloned(),
        ),
    )
    .unwrap();
    let handle = runtime
        .start_recorded_study(definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let study = handle.study_id();
    fixture.observation(|value| value["active"] == 16).await;
    drop(handle);
    // Preparation retention owns source protections on this physical client.
    // Evict its idle cache and join the registered releases before disconnecting.
    runtime.clear_program_cache();
    pse_operations::testing::drain_result_releases(runtime.canonical_store().unwrap())
        .await
        .unwrap();
    runtime.close();
    runtime
        .canonical_store()
        .unwrap()
        .disconnect()
        .await
        .unwrap();

    let handle = independent.study(study);
    let status = handle.status().await.unwrap();
    assert!(!status.cancelled);
    assert_eq!(status.state, StudyState::Open);
    assert_eq!(status.points.len(), 16);
    assert!(status.points.iter().all(|point| !point.settled));
    let held = fixture.observation(|value| value["active"] == 16).await;
    assert!(
        held["entries"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| { entry["released"] == false && entry["stop_observed"] == false })
    );

    let cancelled = handle.cancel().await.unwrap();
    assert_eq!(cancelled.study_id, study);
    assert!(!cancelled.already_concluded);
    let drained = fixture.observation(|value| value["active"] == 0).await;
    assert_eq!(drained["maximum"], 16);
    let entries = drained["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 16);
    assert!(
        entries
            .iter()
            .all(|entry| { entry["released"] == true && entry["stop_observed"] == true })
    );
    tokio::time::timeout(Duration::from_secs(45), async {
        loop {
            let status = handle.status().await.unwrap();
            if status.state == StudyState::Concluded {
                assert!(status.cancelled);
                assert_eq!(status.points.len(), 16);
                assert!(
                    status.points.iter().all(|point| {
                        point.settled && point.state == StudyPointState::Cancelled
                    })
                );
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    independent.close();
    independent
        .canonical_store()
        .unwrap()
        .disconnect()
        .await
        .unwrap();
    fixture.finish().await;
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[tokio::test]
async fn managed_primary_worker_stop_joins_sixteen_native_owners_without_cancelling_study() {
    let fixture = ManagedStudyFixture::start(true).await;
    let runtime = &fixture.runtime;
    let (_, definition) = parallel_study_definition(runtime, 16, None).await;
    let definition = runtime
        .record_study(&definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let handle = runtime
        .start_recorded_study(definition, &crate::CancelSource::new())
        .await
        .unwrap();
    fixture.observation(|value| value["active"] == 16).await;
    // Stop the actual primary's worker lifetime without authorizing the release
    // barrier or changing the independent durable study cancellation authority.
    fixture.signal("stop.json");
    let stopped = fixture.observation(|value| value["active"] == 0).await;
    assert_eq!(stopped["maximum"], 16);
    let entries = stopped["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 16);
    assert!(
        entries
            .iter()
            .all(|entry| { entry["released"] == true && entry["stop_observed"] == true })
    );
    tokio::time::timeout_at(
        tokio::time::Instant::from_std(fixture.drain_deadline()),
        async {
            while std::path::Path::new(&format!("/proc/{}", fixture.worker_pid)).exists() {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        },
    )
    .await
    .unwrap();
    let status = handle.status().await.unwrap();
    assert!(!status.cancelled);
    assert_eq!(status.points.len(), 16);
    assert!(status.points.iter().all(|point| {
        point
            .outcome
            .as_ref()
            .is_none_or(|outcome| !outcome.scientific.usable)
    }));
    fixture.finish().await;
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[tokio::test]
async fn managed_primary_study_cancellation_drains_admitted_sixteen_and_refuses_tail() {
    let fixture = ManagedStudyFixture::start(true).await;
    let runtime = &fixture.runtime;
    let (_, definition) = parallel_study_definition(&fixture.runtime, 20, None).await;
    let cancel = crate::CancelSource::new();
    let definition = runtime
        .record_study(&definition, &crate::CancelSource::new())
        .await
        .unwrap();
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
            tokio::join!(runtime.execute_study(&definition, 20, &cancel), control)
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
    use pse_operations::canonical_analyses::{AnalysisEdge, AnalysisNode};
    use pse_operations::canonical_execution::{RunRequest, TerminalClass};
    use pse_relations::generated::{
        enums::NativeMetricKind,
        runtime::{solve_metrics, solve_variables},
    };
    use std::sync::atomic::{AtomicBool, Ordering};

    let fixture = ManagedStudyFixture::start(true).await;
    let runtime = &fixture.runtime;
    let store = runtime.canonical_store().unwrap();
    let pool = runtime.shared.pool();
    let validation = runtime.validation_context();
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
            solve_metrics::Builder::with_registry(runtime.registry(), 3, &validation).unwrap();
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
            &durable::PublicationAcks::new(&pool),
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
        unsafe {
            store.seal_attempt(
                &manifest,
                &seal,
                TerminalClass::Partial,
                &[1],
                pse_operations::canonical_execution::SeedEligibility::Unavailable,
            )
        }
        .await
        .unwrap();
        attempts.push(fence.attempt().to_owned());
    }

    let (_, definition) = parallel_study_definition(runtime, 16, None).await;
    let cancel = crate::CancelSource::new();
    let definition = runtime
        .record_study(&definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let published = AtomicBool::new(false);
    let science = async {
        let report = runtime
            .execute_study(&definition, 16, &cancel)
            .await
            .unwrap();
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
            current
                .selection()
                .attempt()
                .outcome
                .as_ref()
                .map(|value| value.as_str()),
            Some("partial")
        );
        assert_eq!(
            historical
                .selection()
                .attempt()
                .outcome
                .as_ref()
                .map(|value| value.as_str()),
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

        // This fixture proves exact partial transport ownership only. Copy its
        // original IPC members under both actual protected result selections;
        // it cannot claim a final state or a qualified scientific local analysis.
        let producer_inputs = [current.selection().clone(), historical.selection().clone()];
        let mut original_members = Vec::new();
        for input in &producer_inputs {
            let set = input
                .sets()
                .iter()
                .find(|set| set.name == solve_metrics::RELATION_ID.to_string())
                .unwrap();
            assert_eq!(set.batch_count, 1);
            let member = store.result_payload(input, &set.key, 0).await.unwrap();
            assert_eq!(
                member.batch.row_count, 3,
                "artifact evidence preserves the full original member rather than the projected reader slice"
            );
            original_members.push(member);
        }
        let provenance = serde_json::to_vec(
            &producer_inputs
                .iter()
                .map(|input| {
                    (
                        &input.run().key,
                        &input.run().revision,
                        input.run().sequence,
                        &input.attempt().key,
                        input.attempt().generation,
                        &input.attempt().outcome,
                        input.manifest(),
                        input.sets(),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let mut artifact = store
            .new_artifact(
                &revision,
                "transport-fixture",
                provenance,
                2,
                original_members
                    .iter()
                    .map(|member| member.batch.payload.len() as u64)
                    .sum(),
            )
            .await
            .unwrap();
        let blocks = original_members
            .iter()
            .enumerate()
            .map(|(ordinal, member)| {
                pse_operations::canonical_artifacts::artifact_block(
                    &artifact.key,
                    ordinal as u64,
                    "runtime.solve_metrics",
                    member.batch.payload.to_vec(),
                    &member.batch.digest,
                    member.batch.row_count,
                )
                .native_file(&solve_metrics::RELATION_ID.to_string())
                .unwrap()
            })
            .collect::<Vec<_>>();
        drop(original_members);
        let staging_owner = format!("partial-transport-staging:{}", fixture.nonce);
        pse_operations::canonical_artifacts::seal_artifact_request(
            &mut artifact,
            std::slice::from_ref(&revision),
            &producer_inputs,
            &blocks,
            "audit",
            &staging_owner,
        );
        store
            .persist_artifact(
                &artifact,
                std::slice::from_ref(&revision),
                &producer_inputs,
                &blocks,
                "audit",
                &staging_owner,
            )
            .await
            .unwrap();
        let artifact_read = store.read_artifact(&artifact.key).await.unwrap();
        drop(producer_inputs);
        let mut analysis = store
            .new_analysis(
                &revision,
                "partial-transport-reader-handoff-fixture:v2",
                vec![1],
                artifact.creation_request_digest.clone(),
                1,
                2,
            )
            .await
            .unwrap();
        let node = AnalysisNode {
            key: format!("serving-transport-node-{}", fixture.nonce),
            analysis: analysis.key.clone(),
            semantic: "partial-transport-temperature".into(),
            kind: "result".into(),
        };
        let edges = (0..2)
            .map(|ordinal| AnalysisEdge {
                key: format!("serving-transport-edge-{}-{ordinal}", fixture.nonce),
                analysis: analysis.key.clone(),
                source: node.key.clone(),
                target: node.key.clone(),
                kind: format!("retained_partial_transport_{ordinal}"),
                evidence_artifact: Some(artifact.key.clone()),
                evidence_block: Some(ordinal),
                evidence_row: Some(2),
                evidence_field: Some("real".into()),
            })
            .collect::<Vec<_>>();
        let inputs = [artifact_read];
        pse_operations::canonical_analyses::seal_analysis_request(
            &mut analysis,
            std::slice::from_ref(&revision),
            &inputs,
            std::slice::from_ref(&node),
            &edges,
        );
        assert!(
            store
                .persist_analysis(
                    &analysis,
                    std::slice::from_ref(&revision),
                    &inputs,
                    &[node],
                    &edges
                )
                .await
                .unwrap()
                .active
        );
        store
            .release_artifact_intent(&artifact, "audit", &staging_owner)
            .await
            .unwrap();
        for read in inputs {
            read.release().await.unwrap();
        }
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
        let mut retirement = None;
        for _ in 0..32 {
            if let Ok(page) = store.forget_run_results(&transport_run).await {
                retirement = Some(page);
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let retirement = retirement.expect("reader release did not permit bounded retirement");
        // Retirement returns its first bounded cleanup page. Include it when
        // accounting for the exact two batches across both attempt generations.
        let mut reclaimed = retirement.batches;
        let mut complete = retirement.complete;
        // Each transport attempt needs separate bounded pages for its batch,
        // result set, append receipt and generation advance. Completion is a
        // final page after both immutable attempt generations are traversed.
        let maximum_pages = attempts.len() * 4 + 1;
        for _ in 0..maximum_pages {
            if complete {
                break;
            }
            let page = store.reclaim_result_page(&transport_run).await.unwrap();
            reclaimed += page.batches;
            if page.complete {
                complete = true;
                break;
            }
        }
        assert!(
            complete,
            "transport reclamation exceeded {maximum_pages} pages"
        );
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
        assert!(
            store.analysis(&analysis.key).await.unwrap().active,
            "the independent graph survives while producer results drain"
        );
        let retained_edges = store.analysis_edge_page(&analysis.key, None).await.unwrap();
        assert_eq!(retained_edges.len(), 2);
        assert!(
            retained_edges
                .iter()
                .all(
                    |edge| edge.evidence_artifact.as_deref() == Some(artifact.key.as_str())
                        && edge.evidence_row == Some(2)
                        && edge.evidence_field.as_deref() == Some("real")
                )
        );
        let independent = store.read_artifact(&artifact.key).await.unwrap();
        for ordinal in 0..2 {
            let copied = independent.block(ordinal).await.unwrap();
            assert_eq!(
                copied.payload, blocks[ordinal as usize].payload,
                "both original IPC members remain bit-exact after producer retirement"
            );
        }
        independent.release().await.unwrap();
        // Analysis retirement may still await its original finite creation window;
        // it never needs to keep the reclaimed producer's result inventory.
        let _complete = runtime
            .forget_analysis_results(&analysis.key)
            .await
            .unwrap();
        // Escaped decoded Arrow buffers need no storage access after reader close,
        // native publication and original producer result cleanup have finished.
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
            reader
                .selection()
                .attempt()
                .outcome
                .as_ref()
                .map(|value| value.as_str()),
            Some("succeeded")
        );
        let mut roots = 0;
        while let Some(batch) = reader.next_batch().await.unwrap() {
            let view = solve_variables::View::try_from_batch_with_registry(
                runtime.registry(),
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
        .admit_study(
            &StudyInputs::Explicit(points.to_vec()),
            &StudyOutputDemand::Full {},
            &cancel,
        )
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
        .admit_study(
            &StudyInputs::Explicit([fixed_point].to_vec()),
            &StudyOutputDemand::Full {},
            &cancel,
        )
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
        .admit_study(
            &StudyInputs::Explicit([requested].to_vec()),
            &StudyOutputDemand::Full {},
            &cancel,
        )
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
    // Saturation below must constrain the seed grant itself; optional retained
    // preparation products may otherwise be reclaimed to admit that grant.
    runtime.clear_program_cache();
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
        RunOutcome::Modeling {
            request: vec![*prepared.clone()],
            result: Ok(vec![completed.clone()]),
        },
        None,
    )
    .finished(None, false)
    .await;
    let mut target = prepared;
    // Production wrappers borrow the precharged study occurrence metadata owner.
    let continuation_owner = runtime
        .native()
        .reserve("test:continuation-wrapper", 4096)
        .unwrap();
    let allowance = extent - 1;
    let filler = MemoryConsumer::new("test:seed-pressure").register(&pool);
    filler
        .try_grow(runtime.shared.budget().memory_limit_bytes.get() - pool.reserved() - allowance)
        .unwrap();
    let before = pool.reserved();
    let refused =
        study_execution::minimal_continuation(&previous, continuation_owner.clone()).unwrap();
    let error =
        study_execution::continuation_start(&package, &mut target, Some(&refused), None, &cancel)
            .await
            .unwrap_err();
    drop(refused);
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
    let admitted =
        study_execution::minimal_continuation(&previous, continuation_owner.clone()).unwrap();
    study_execution::continuation_start(&package, &mut target, Some(&admitted), None, &cancel)
        .await
        .unwrap();
    drop(admitted);
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
    let request = vec![target.as_ref().clone()];
    let mut clones: Vec<_> = (0..16).map(|_| target.as_ref().clone()).collect();
    let requests = &request;
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

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn lazy_study_admits_only_ready_selected_science_and_preserves_base_head() {
    let runtime = durable_runtime();
    let (package, _) = related_definition(&runtime).await;
    let fixed = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|row| row.name == "Fixed")
        .unwrap()
        .declaration_id;
    let base = package.canonical_revision().unwrap().clone();
    let mut requested = point(fixed, 9, vec![], StartPolicy::Fresh);
    if let OperationRequest::DeclaredCase(case) = &mut requested.operation {
        case.settings.backend = None;
    }
    let mut invalid = requested.clone();
    invalid.policy.key = OccurrenceKey(3);
    let alternatives = vec![
        SourceDelta {
            parent: package.revision_id().into(),
            ..Default::default()
        },
        SourceDelta {
            parent: package.revision_id().into(),
            removals: vec![pse_model::generated::identities::DeclarationId::from_bytes(
                [0; 16],
            )],
            ..Default::default()
        },
    ];
    let variants = StudyVariants {
        version: pse_model::document::Version,
        source: OperationSource::of(&package),
        choices: vec![SourceChoice { alternatives }],
        selected: vec![
            SelectedStudyVariant {
                ordinal: 0,
                point: requested,
            },
            SelectedStudyVariant {
                ordinal: 1,
                point: invalid,
            },
        ],
    };
    let cancel = crate::CancelSource::new();
    let before = runtime.shared.math().preparations().views;
    let definition = package
        .admit_study(
            &StudyInputs::Lazy(Box::new(variants)),
            &StudyOutputDemand::Full {},
            &cancel,
        )
        .await
        .unwrap();
    assert!(definition.points.is_empty());
    assert_eq!(definition.point_count(), 2);
    assert!(
        definition
            .graph()
            .points
            .iter()
            .all(|point| point.seed_need == SeedNeed::Pending)
    );
    let definition = runtime
        .record_study(&definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let handle = runtime
        .start_recorded_study(definition, &cancel)
        .await
        .unwrap();
    assert_eq!(
        runtime.shared.math().preparations().views,
        before,
        "descriptor admission and activation construct no selected science"
    );
    let study = handle.study_id().to_string();
    let valid_key = pse_operations::canonical_studies::point_key(&study, OccurrenceKey(9));
    let invalid_key = pse_operations::canonical_studies::point_key(&study, OccurrenceKey(3));
    let scope = runtime
        .canonical_store()
        .unwrap()
        .study_scope(&valid_key)
        .await
        .unwrap();
    assert!(
        runtime
            .canonical_store()
            .unwrap()
            .claim_study_point(
                &scope,
                None,
                "pending-native",
                "worker",
                Duration::from_secs(60)
            )
            .await
            .is_err()
    );
    assert!(
        runtime
            .admit_study_candidate(&scope, &mut study::StudyCandidateLane::new(), &cancel)
            .await
            .unwrap()
    );
    let selected = runtime
        .canonical_store()
        .unwrap()
        .canonical_study_point(&valid_key)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        selected.ordinal, 0,
        "visible order is selection order, not Cartesian ordinal or key order"
    );
    assert_eq!(
        pse_operations::canonical_studies::point_facts(&selected)
            .unwrap()
            .attempt_count,
        0
    );
    let science = runtime
        .canonical_store()
        .unwrap()
        .canonical_run(&selected.run)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        science.revision,
        format!("study-variant-source:{valid_key}")
    );
    assert_eq!(package.canonical_revision().unwrap(), &base);
    let scope = runtime
        .canonical_store()
        .unwrap()
        .study_scope(&invalid_key)
        .await
        .unwrap();
    assert!(
        runtime
            .admit_study_candidate(&scope, &mut study::StudyCandidateLane::new(), &cancel)
            .await
            .unwrap()
    );
    let invalid = runtime
        .canonical_store()
        .unwrap()
        .canonical_study_point(&invalid_key)
        .await
        .unwrap()
        .unwrap();
    assert!(invalid.settled);
    assert!(invalid.attempt.is_none());
    assert_eq!(
        pse_operations::canonical_studies::point_facts(&invalid)
            .unwrap()
            .attempt_count,
        0
    );
    assert!(
        runtime
            .canonical_store()
            .unwrap()
            .canonical_run(&invalid.run)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(package.canonical_revision().unwrap(), &base);
    runtime
        .canonical_store()
        .unwrap()
        .edit(
            &base.problem,
            Some(&base.key),
            "base-head-control-after-variants",
            &[],
        )
        .await
        .unwrap();
    handle.cancel().await.unwrap();
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn lazy_study_cancellation_and_summary_do_not_construct_unattempted_science() {
    let runtime = durable_runtime();
    let (package, explicit) = related_definition(&runtime).await;
    let requested = StudyPoint {
        operation: explicit.points[0].operation.operation.clone(),
        preparation: explicit.points[0].operation.preparation.clone(),
        overlay: PointOverlay::default(),
        policy: StudyPointPolicy {
            key: OccurrenceKey(77),
            dependencies: vec![],
            start: StartPolicy::Fresh,
            attempt_limit: 1,
        },
    };
    let variants = StudyVariants {
        version: pse_model::document::Version,
        source: OperationSource::of(&package),
        choices: vec![SourceChoice {
            alternatives: vec![SourceDelta {
                parent: package.revision_id().into(),
                removals: vec![pse_model::generated::identities::DeclarationId::from_bytes(
                    [0; 16],
                )],
                ..Default::default()
            }],
        }],
        selected: vec![SelectedStudyVariant {
            ordinal: 0,
            point: requested,
        }],
    };
    let cancel = crate::CancelSource::new();
    let definition = package
        .admit_study(
            &StudyInputs::Lazy(Box::new(variants)),
            &StudyOutputDemand::Full {},
            &cancel,
        )
        .await
        .unwrap();
    let definition = runtime
        .record_study(&definition, &crate::CancelSource::new())
        .await
        .unwrap();
    let handle = runtime
        .start_recorded_study(definition, &cancel)
        .await
        .unwrap();
    let before = runtime.shared.math().preparations().views;
    handle.cancel().await.unwrap();
    let key = pse_operations::canonical_studies::point_key(
        &handle.study_id().to_string(),
        OccurrenceKey(77),
    );
    let scope = runtime
        .canonical_store()
        .unwrap()
        .study_scope(&key)
        .await
        .unwrap();
    let action = scope.action(None).unwrap();
    assert!(matches!(action.kind, ActionKind::Cancel));
    runtime
        .settle_study_candidate(&scope, &action, None)
        .await
        .unwrap();
    assert!(
        runtime
            .finalize_canonical_study(&handle.study_id().to_string())
            .await
            .unwrap()
    );
    assert!(
        runtime
            .retained_study_admissions(&handle.study_id().to_string())
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(runtime.shared.math().preparations().views, before);
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn project_context_recorded_study_keeps_original_source_after_backdated_correction() {
    let runtime = durable_runtime();
    let cancel = crate::CancelSource::new();
    let (physical_documents, modeling) = sources(
        "package algebraic { def Fixed { param x:Scalar=2; eq fixed:x==2; annotation report x(\"value\"); } }",
    );
    let physical = runtime
        .physical_from_sources(&physical_documents, &cancel)
        .await
        .unwrap();
    let package = runtime
        .package_from_sources(std::slice::from_ref(&modeling), physical.clone(), &cancel)
        .await
        .unwrap();
    let store = runtime.canonical_store().unwrap();
    store
        .create_project(
            "context-project-create",
            "context-project",
            "History consumer",
        )
        .await
        .unwrap();
    let project = store.project("context-project").await.unwrap().unwrap();
    runtime
        .project_record_context(
            "engineering-original",
            &project,
            "calibration",
            &package,
            0,
            None,
            None,
            false,
        )
        .await
        .unwrap();
    let original_cut = store.project_context_cut(&project.key).await.unwrap();
    let original = store
        .select_project_context(&original_cut, "calibration", 20)
        .await
        .unwrap()
        .unwrap();
    let selected = runtime
        .project_context_reopen(&original, &cancel)
        .await
        .unwrap();
    let fixed = selected
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|row| row.name == "Fixed")
        .unwrap()
        .declaration_id;
    let mut requested = point(fixed, 0, vec![], StartPolicy::Fresh);
    if let OperationRequest::DeclaredCase(case) = &mut requested.operation {
        case.settings.backend = None;
    }
    let local = selected
        .admit_study(
            &StudyInputs::Explicit(vec![requested.clone()]),
            &StudyOutputDemand::Full {},
            &cancel,
        )
        .await
        .unwrap();
    let recorded = runtime.record_study(&local, &cancel).await.unwrap();
    assert_eq!(recorded.context.as_ref(), Some(&original));
    assert_eq!(recorded.modeling_revision, original.context.revision);
    assert_eq!(
        recorded.points[0].operation.source.context.as_ref(),
        Some(&original)
    );
    let encoded = serde_json::to_string(&recorded).unwrap();
    let handle = runtime
        .start_recorded_study(recorded, &cancel)
        .await
        .unwrap();
    let before = store
        .canonical_study(&handle.study_id().to_string())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        before.physical_revision.as_deref(),
        Some(original.context.physical_revision.as_str())
    );
    assert_eq!(
        before.physical_identity.as_deref(),
        Some(original.context.physical_source_identity.as_str()),
        "canonical study attribution uses the original physical checksum, not its JSON prefix"
    );
    // One worker lane must refresh historical attribution even when science and
    // physical source are byte-for-byte identical in two independent studies.
    runtime
        .project_record_context(
            "engineering-other",
            &project,
            "other-scope",
            &package,
            0,
            None,
            None,
            false,
        )
        .await
        .unwrap();
    let other_cut = store.project_context_cut(&project.key).await.unwrap();
    let other = store
        .select_project_context(&other_cut, "other-scope", 20)
        .await
        .unwrap()
        .unwrap();
    let other_package = runtime
        .project_context_reopen(&other, &cancel)
        .await
        .unwrap();
    let other_local = other_package
        .admit_study(
            &StudyInputs::Explicit(vec![requested]),
            &StudyOutputDemand::Full {},
            &cancel,
        )
        .await
        .unwrap();
    let other_recorded = runtime.record_study(&other_local, &cancel).await.unwrap();
    assert_eq!(other_recorded.modeling_revision, original.context.revision);
    let other_handle = runtime
        .start_recorded_study(other_recorded, &cancel)
        .await
        .unwrap();
    let mut lane = study::StudyCandidateLane::new();
    for (selected_handle, expected) in [(&handle, &original), (&other_handle, &other)] {
        let point = pse_operations::canonical_studies::point_key(
            &selected_handle.study_id().to_string(),
            OccurrenceKey(0),
        );
        let scope = store.study_scope(&point).await.unwrap();
        let prepared = runtime
            .prepare_study_candidate_in_lane(&scope, &mut lane, &cancel)
            .await
            .unwrap();
        let PreparedStudyOperation::DeclaredCase(prepared) = prepared.operation else {
            panic!("original constant algebraic operation");
        };
        assert_eq!(prepared.source.project_context.as_ref(), Some(expected));
    }
    lane.close().await.unwrap();
    let (replacement_physical, replacement_modeling) = sources(
        "package algebraic { def Fixed { param x:Scalar=3; eq fixed:x==3; annotation report x(\"value\"); } }",
    );
    let replacement = runtime
        .package_from_sources(
            std::slice::from_ref(&replacement_modeling),
            runtime
                .physical_from_sources(&replacement_physical, &cancel)
                .await
                .unwrap(),
            &cancel,
        )
        .await
        .unwrap();
    runtime
        .project_record_context(
            "engineering-correction",
            &project,
            "calibration",
            &replacement,
            10,
            None,
            Some("engineering-original"),
            false,
        )
        .await
        .unwrap();
    let latest_cut = store.project_context_cut(&project.key).await.unwrap();
    let corrected = store
        .select_project_context(&latest_cut, "calibration", 20)
        .await
        .unwrap()
        .unwrap();
    let new_package = runtime
        .project_context_package(&corrected, physical.clone(), &cancel)
        .await
        .unwrap();
    assert_ne!(
        OperationSource::of(&selected).revision,
        OperationSource::of(&new_package).revision
    );
    runtime
        .project_record_context(
            "engineering-deletion",
            &project,
            "calibration",
            &replacement,
            15,
            None,
            Some("engineering-correction"),
            true,
        )
        .await
        .unwrap();
    let final_cut = store.project_context_cut(&project.key).await.unwrap();
    assert!(
        store
            .select_project_context(&final_cut, "calibration", 20)
            .await
            .unwrap()
            .is_none()
    );
    let old_definition = StudyDefinition::readmission(&encoded).unwrap();
    let old_package = runtime
        .project_context_package(old_definition.context.as_ref().unwrap(), physical, &cancel)
        .await
        .unwrap();
    old_definition.points[0]
        .operation
        .source
        .check(&old_package)
        .unwrap();
    assert_eq!(old_definition.context.as_ref(), Some(&original));
    let after = store
        .canonical_study(&handle.study_id().to_string())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(before.metadata, after.metadata);
    assert_eq!(before.revision, after.revision);
    let portable = selected.replay_source().unwrap();
    assert_eq!(portable.context.as_ref(), Some(&original));
    handle.cancel().await.unwrap();
    other_handle.cancel().await.unwrap();
    for _ in 0..4 {
        if matches!(runtime.work_once().await.unwrap(), Processed::Idle) {
            break;
        }
    }
}
