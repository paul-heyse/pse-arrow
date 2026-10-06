// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Paired admitted studies: identical source/order/profile/policy, fresh runtime per iteration.
use super::k4_support as support;
use super::*;
use pse_model::{
    generated::enums::{ModelingAnalysisRoute, StudyPointState},
    scalars::FiniteBound,
    study::*,
};
use pse_operations::{jobs::RetryPolicy, studies::StudyState, testing::TestDatabase};
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{solve_metrics, solve_strategy_events, solve_variables},
};
use pse_runtime::workflow::{self, Runtime};
use serde_json::{Value, json};
const VALUES: [f64; 8] = [2., 2., 3., 2., 4., 3., 2., 2.];

async fn definition(
    package: &ModelingPackage,
    sources: &workflow::PackageSources,
    physical: &workflow::PhysicalContext,
) -> workflow::StudyDefinition {
    let root = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    // Assignments use the admitted registry and its declared canonical unit.
    let context = workflow::OperationSource::of(package);
    let scalar = physical.quantities().neutral_dimensionless().unwrap();
    let quantity = scalar.as_id();
    let unit = physical
        .quantities()
        .quantity_type(scalar)
        .unwrap()
        .canonical_unit
        .as_id();
    let points: Vec<_> = VALUES
        .into_iter()
        .enumerate()
        .map(|(index, value)| workflow::StudyPoint {
            operation: workflow::OperationRequest::DeclaredCase(workflow::CaseOperation {
                case: root,
                route: ModelingAnalysisRoute::Steady,
                settings: pse_runtime::math::settings::SolveSettings {
                    backend: Some(Backend::Kinsol),
                    intent: pse_backend_native::solve::SolveIntent::Root,
                    presolve: pse_backend_native::presolve::PolicyKind::Off,
                    controls: pse_backend_native::solve::Controls {
                        threads: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }),
            preparation: workflow::PreparationSettings::default(),
            overlay: workflow::PointOverlay {
                assignments: vec![workflow::BindingAssignment {
                    target: workflow::BindingTarget::Path("a".into()),
                    value: workflow::BindingQuantity {
                        magnitude: FiniteBound::try_new(value).unwrap(),
                        quantity,
                        unit,
                    },
                }],
            },
            policy: workflow::StudyPointPolicy {
                key: OccurrenceKey(index as u32),
                dependencies: if index == 0 {
                    vec![]
                } else {
                    vec![Dependency::Ordering(OccurrenceKey(index as u32 - 1))]
                },
                start: if index == 0 {
                    StartPolicy::Fresh
                } else {
                    StartPolicy::Continuation(SeedEdge {
                        predecessor: OccurrenceKey(index as u32 - 1),
                        role: SeedRole::PrimalSolution,
                        permission: ContinuationPermission::RequireUsable,
                        unavailable: UnavailableSeedPolicy::Refuse,
                    })
                },
                attempt_limit: 1,
            },
        })
        .collect();
    let admitted = package
        .admit_study_points(
            pse_runtime::authoring_driver::document::package_checksum(&sources.physical),
            &points,
            &CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(
        admitted.points[0].binding_hash,
        admitted.points[1].binding_hash
    );
    assert_eq!(
        admitted.points[0].binding_hash,
        admitted.points[3].binding_hash
    );
    assert!(
        admitted
            .points
            .iter()
            .all(|p| p.operation.source.physical_context == context.physical_context)
    );
    admitted
}
fn check(outcomes: &[PointOutcome]) {
    assert_eq!(outcomes.len(), 8);
    for (index, outcome) in outcomes.iter().enumerate() {
        assert_eq!(outcome.key, OccurrenceKey(index as u32));
        assert_eq!(outcome.lifecycle, StudyPointState::Completed, "{outcome:?}");
        assert!(outcome.scientific.usable, "{outcome:?}");
        assert!(outcome.diagnostic.is_none(), "{outcome:?}");
        assert_eq!(outcome.attempts.len(), 1);
        if index > 0 {
            assert!(
                matches!(outcome.start,Some(StartProvenance::Continuation { predecessor, .. }) if predecessor == OccurrenceKey(index as u32-1)),
                "{outcome:?}"
            );
        }
    }
}
fn metrics(rows: Vec<solve_metrics::Row>) -> Value {
    Value::Array(rows.into_iter().map(|r| json!({"namespace":r.namespace,"name":r.name,"kind":r.kind,"real":r.real,"integer":r.integer,"boolean":r.boolean,"text":r.text,"unavailable":r.unavailable})).collect())
}
async fn persisted(
    runtime: &Runtime,
    owner: &WorkflowRuntime,
    publication: pse_operations::catalog::PublicationId,
    numerical: &mut observations::Observations,
) -> Vec<Value> {
    let opened = runtime.open(publication, &owner.cancel).await.unwrap();
    let original = opened.publication();
    let mut observations = Vec::new();
    for (index, expected) in VALUES.into_iter().enumerate() {
        let member = |name: &str| datafusion::common::ResolvedTableReference {
            catalog: format!("point_{index}").into(),
            schema: "runtime".into(),
            table: name.into(),
        };
        let relation = |name: &str| owner.registry.relation(&format!("runtime.{name}")).unwrap();
        let completed = original
            .relation_stream(&member("solve_metrics"), &owner.cancel)
            .await
            .unwrap()
            .collect(&owner.cancel)
            .await
            .unwrap();
        let checked = completed
            .checked_relation(&owner.registry, relation("solve_metrics"), &owner.cancel)
            .unwrap();
        let rows = solve_metrics::Row::rows(&checked).unwrap();
        numerical.persisted_metrics(&rows);
        let native = metrics(rows);
        let completed = original
            .relation_stream(&member("solve_strategy_events"), &owner.cancel)
            .await
            .unwrap()
            .collect(&owner.cancel)
            .await
            .unwrap();
        let checked = completed
            .checked_relation(
                &owner.registry,
                relation("solve_strategy_events"),
                &owner.cancel,
            )
            .unwrap();
        numerical.rows(&solve_strategy_events::Row::rows(&checked).unwrap());
        let completed = original
            .relation_stream(&member("solve_variables"), &owner.cancel)
            .await
            .unwrap()
            .collect(&owner.cancel)
            .await
            .unwrap();
        let checked = completed
            .checked_relation(&owner.registry, relation("solve_variables"), &owner.cancel)
            .unwrap();
        let variables = solve_variables::Row::rows(&checked).unwrap();
        let values: Vec<_> = variables
            .iter()
            .filter(|r| !r.parameter && !r.fixed)
            .map(|r| r.value.unwrap())
            .collect();
        assert_eq!(values.len(), 2);
        assert!(values.iter().any(|value| (*value - expected).abs() < 1e-7));
        assert!(values.iter().any(|value| (*value - 3.).abs() < 1e-7));
        observations.push(json!({"key":index,"native_metrics":native,"variables":values}));
    }
    observations
}

pub(super) fn measure(
    c: &mut Criterion,
    spec: &Value,
    output: &std::path::Path,
    phases: &phases::Phases,
) {
    let durable_mode = spec["adapter"] == "durable";
    let name = spec["id"].as_str().unwrap();
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let mut records = Vec::new();
    let mut group = c.benchmark_group("process");
    group
        .sample_size(10)
        .sampling_mode(criterion::SamplingMode::Flat)
        .warm_up_time(Duration::from_millis(1))
        .measurement_time(Duration::from_secs(1));
    group.bench_function(name, |b| b.iter_custom(|iterations| {
        let mut timed = Duration::ZERO;
        for _ in 0..iterations {
            let (elapsed, record) = executor.block_on(async {
                let mut numerical=observations::Observations::default();
                let started=Instant::now();
                let database=if durable_mode { Some(TestDatabase::create().await.unwrap()) } else { None };
                let database_setup=started.elapsed();
                let owner=WorkflowRuntime::with_threads(NonZeroUsize::new(1).unwrap()).unwrap();
                let runtime=match &database { Some(database)=>durable(&owner,database.url()).await, None=>runtime(&owner) };
                let sources=support::sources(support::SOURCE);
                phases.reset();
                let before=owner.runtime.math().preparations();
                let started=Instant::now();
                let physical=support::physical(&runtime,&owner,&sources).await;
                let package=runtime.modeling_from_documents(&support::admitted_documents(&owner,&sources.modeling),physical.clone()).await.unwrap();
                let definition=definition(&package,&sources,&physical).await;
                let admission=started.elapsed();
                let admission_counts=support::counts(before,owner.runtime.math().preparations());
                let admission_phases=phases.report(8);
                let admission_constructions=phases.constructions();
                // Both adapters begin execution with the same admission-warmed cache;
                // each iteration has a fresh independent one-thread runtime.
                let before=owner.runtime.math().preparations();
                phases.reset();
                let started=Instant::now();
                let mut worker_passes=Vec::new();
                let directory=tempfile::tempdir().unwrap();
                let (elapsed,submission,submission_counts,worker_counts,result_read_seconds,outcomes,metrics) = if durable_mode {
                    let workspace=runtime.register_workspace("k4",url::Url::from_directory_path(directory.path()).unwrap()).await.unwrap();
                    let start=Instant::now();
                    let handle=runtime.start_defined_study(&workspace,sources.physical,definition.clone(),RetryPolicy::ONCE,0).await.unwrap();
                    let submission=start.elapsed();
                    let submission_counts=support::counts(before,owner.runtime.math().preparations());
                    let worker_before=owner.runtime.math().preparations();
                    let mut terminal=false;
                    for _ in 0..256 {
                        let pass_before=owner.runtime.math().preparations();
                        let start=Instant::now();
                        let processed=runtime.work_once().await.unwrap();
                        let seconds=start.elapsed().as_secs_f64();
                        match processed {
                            workflow::Processed::Idle=> {
                                if handle.status().await.unwrap().state==StudyState::Published { terminal=true; break; }
                                tokio::time::sleep(Duration::from_millis(10)).await;
                            },
                            workflow::Processed::Ran {record,..}=>worker_passes.push(json!({"seconds":seconds,"attempt":record.attempt.as_ref().unwrap().state,"attempt_kind":record.attempt.as_ref().unwrap().kind,"solutions":record.solutions.len(),"preparations":support::counts(pass_before,owner.runtime.math().preparations())})),
                        }
                        assert!(worker_passes.len()<=32,"unexpected study retry");
                        if handle.status().await.unwrap().state==StudyState::Published { terminal=true; break; }
                    }
                    let elapsed=started.elapsed();
                    assert!(terminal,"study did not publish during the bounded serial drain");
                    let worker_counts=support::counts(worker_before,owner.runtime.math().preparations());
                    let read=Instant::now();
                    let status=handle.status().await.unwrap();
                    assert_eq!(status.state,StudyState::Published,"{status:?}");
                    let outcomes:Vec<_>=status.points.into_iter().map(|p|p.outcome.unwrap()).collect();
                    check(&outcomes);
                    let published=handle.result().await.unwrap().unwrap();
                    let metrics=persisted(&runtime,&owner,published.publication_id,&mut numerical).await;
                    (elapsed,submission,submission_counts,worker_counts,read.elapsed().as_secs_f64(),outcomes,metrics)
                } else {
                    let report=package.study(&definition,8,&CancelSource::new()).await.unwrap();
                    let elapsed=started.elapsed();
                    check(&report.outcomes);
                    let read=Instant::now();
                    let mut metrics=Vec::new();
                    for (index,result) in report.results.iter().enumerate() {
                        let result=result.as_ref().unwrap();
                        let report=authored_success(result);
                        numerical.run(result);
                        let symbol=report.prepared.model.model.compiled().model.symbols.values().find(|s|s.lineage.path.ends_with(".x")).unwrap().id;
                        near(variable(result,symbol),VALUES[index],1e-7);
                        let rows=solve_metrics::Row::rows(&result.table("runtime.solve_metrics").unwrap()).unwrap();
                        metrics.push(json!({"key":index,"native_metrics":self::metrics(rows)}));
                    }
                    (elapsed,Duration::ZERO,json!({"views":0,"observations":0,"rebuilt":0,"shared":0}),serde_json::to_value(report.preparations).unwrap(),read.elapsed().as_secs_f64(),report.outcomes,metrics)
                };
                numerical.preparations(before,owner.runtime.math().preparations());
                let execution_counts=support::counts(before,owner.runtime.math().preparations());
                let execution_phases=phases.report(8);
                let peak=owner.runtime.observation_peak_bytes();
                let rss=owner.runtime.report().unwrap().process_peak_rss_bytes;
                let pool=owner.runtime.pool();
                let retained=pool.reserved();
                let record=json!({"seconds":{"database_setup":database_setup.as_secs_f64(),"source_and_definition_admission":admission.as_secs_f64(),"execution_total":elapsed.as_secs_f64(),"submission":submission.as_secs_f64(),"result_read":result_read_seconds},"admission_preparations":admission_counts,"submission_preparations":submission_counts,"worker_preparations":worker_counts,"execution_preparations":execution_counts,"admission_phases":admission_phases,"execution_phases":execution_phases,"worker_passes":worker_passes,"outcomes":outcomes,"point_metrics":metrics,"numerical_observations":numerical.json(),"definition":definition,"pool_peak_bytes":peak,"process_peak_rss_bytes":rss,"retained_runtime_bytes":retained});
                drop(package); drop(physical); drop(runtime); drop(owner);
                tokio::task::yield_now().await;
                let mut record=record;
                record["admission_constructions"]=admission_constructions;
                record["execution_constructions"]=phases.constructions();
                record["after_runtime_teardown_bytes"]=pool.reserved().into();
                assert_eq!(pool.reserved(), 0, "study allocations survive all runtime owners");
                if let Some(database)=database { database.remove().await.unwrap(); }
                (elapsed,record)
            });
            timed+=elapsed; records.push(record);
        }
        timed
    }));
    group.finish();
    let maximum = |field: &str| records.iter().filter_map(|r| r[field].as_u64()).max();
    let record = json!({"id":name,"workload":spec,"iterations":records.len(),"occurrences":8,"threads":1,"native_threads":1,"math_policy":format!("{:?}",pse_runtime::math::MathPolicy::default()),"pool_limit_bytes":64_u64<<30,"pool_peak_bytes":maximum("pool_peak_bytes"),"process_peak_rss_bytes":maximum("process_peak_rss_bytes"),"retained_runtime_bytes":maximum("retained_runtime_bytes"),"after_case_teardown_bytes":maximum("after_runtime_teardown_bytes"),"after_retained_runtime_teardown_bytes":maximum("after_runtime_teardown_bytes"),"cache_state":"fresh independent runtime per iteration; identical source/definition admission precedes execution in each adapter","timed_scope":if durable_mode {"workspace registration, durable submission, serial work_once passes including per-point persistence and finalization"} else {"in-process admitted study execution including preparation and native attempts"},"persistence_scope":"durable worker clocks include persistence; no estimated or subtracted phase; actual reopened native metrics retained","sampling":"10 flat Criterion samples; database/setup/admission and result reopening outside Criterion timer","records":records});
    std::fs::write(
        output.join(format!("{name}-memory.json")),
        serde_json::to_vec_pretty(&record).unwrap(),
    )
    .unwrap();
}
