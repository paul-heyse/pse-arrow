// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Admitted continuation and independent studies, with explicit adapter and clock scopes.
use super::k4_support as support;
use super::*;
use pse_model::{
    generated::enums::{ModelingAnalysisRoute, StudyPointState, StudyState},
    scalars::FiniteBound,
    study::*,
};
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
    values: &[f64],
    parallel: bool,
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
    let points: Vec<_> = values
        .iter()
        .copied()
        .enumerate()
        .map(|(index, value)| workflow::StudyPoint {
            operation: workflow::OperationRequest::DeclaredCase(workflow::CaseOperation {
                case: root,
                route: ModelingAnalysisRoute::Steady,
                settings: pse_runtime::math::settings::SolveSettings {
                    backend: Some(if parallel {
                        Backend::Ipopt
                    } else {
                        Backend::Kinsol
                    }),
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
                dependencies: if parallel || index == 0 {
                    vec![]
                } else {
                    vec![Dependency::Ordering(OccurrenceKey(index as u32 - 1))]
                },
                start: if parallel || index == 0 {
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
        .admit_study_sources(&sources.physical, &points, &CancelSource::new())
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
fn check(outcomes: &[PointOutcome], occurrences: usize, parallel: bool) {
    assert_eq!(outcomes.len(), occurrences);
    for (index, outcome) in outcomes.iter().enumerate() {
        assert_eq!(outcome.key, OccurrenceKey(index as u32));
        assert_eq!(outcome.lifecycle, StudyPointState::Completed, "{outcome:?}");
        assert!(outcome.scientific.usable, "{outcome:?}");
        assert!(outcome.diagnostic.is_none(), "{outcome:?}");
        assert_eq!(outcome.attempts.len(), 1);
        if parallel {
            assert!(
                matches!(outcome.start, Some(StartProvenance::Fresh)),
                "{outcome:?}"
            );
        } else if index > 0 {
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
async fn stored_rows<R: RelationRow>(
    runtime: &Runtime,
    owner: &WorkflowRuntime,
    run: &str,
    attempt: &str,
    name: &str,
) -> Vec<R> {
    let mut reader = runtime
        .results(run, attempt, name, 0, u64::MAX, owner.cancel.clone())
        .await
        .unwrap();
    let mut rows = Vec::new();
    while let Some(batch) = reader.next_relation_batch().await.unwrap() {
        rows.extend(R::rows(&batch).unwrap());
    }
    rows
}
fn checked_values(variables: &[solve_variables::Row], expected: f64) -> Vec<f64> {
    let values: Vec<_> = variables
        .iter()
        .filter(|row| !row.parameter && !row.fixed)
        .map(|row| row.value.unwrap())
        .collect();
    assert_eq!(values.len(), 2);
    assert!(values.iter().any(|value| (*value - expected).abs() < 1e-7));
    assert!(values.iter().any(|value| (*value - 3.).abs() < 1e-7));
    values
}

fn ephemeral(
    report: &workflow::StudyReport,
    values: &[f64],
    numerical: &mut observations::Observations,
    parallel: bool,
) -> Vec<Value> {
    assert_eq!(report.results.len(), values.len());
    let mut identities = BTreeSet::new();
    report.results.iter().zip(values).enumerate().map(|(index, (result, expected))| {
        let result = result.as_ref().expect("every usable occurrence retains its original result");
        assert!(identities.insert(result.run_id()), "occurrences must retain distinct scientific identities");
        let workflow::StudyOccurrenceResult::Ephemeral(result) = result else {
            panic!("explicit ephemeral adapter returned a retained canonical handle");
        };
        // Count original joined native reports, not reopened metric projections.
        numerical.run(result);
        let RunReport::Modeling(reports) = result.report().unwrap() else { panic!("root study report") };
        assert_eq!(reports.len(), 1);
        if let pse_runtime::math::solves::Outcome::Constant(completed) = &reports[0].outcome {
            for native in completed.component_reports() { numerical.native(native); }
        }
        if parallel {
            match &reports[0].outcome {
                pse_runtime::math::solves::Outcome::Native(native) => assert_eq!(native.backend, Backend::Ipopt, "independent workload uses the nonbatch Ipopt adapter"),
                pse_runtime::math::solves::Outcome::Constant(completed) => {
                    assert!(completed.component_reports().len() > 0, "complete original evaluation must retain actual native component evidence");
                    for native in completed.component_reports() {
                        assert_eq!(native.backend, Backend::Ipopt, "automatic blocks retain actual nonbatch Ipopt component reports");
                    }
                }
                pse_runtime::math::solves::Outcome::Rejected(_) => panic!("usable independent root rejected native execution"),
            }
        }
        let native = metrics(solve_metrics::Row::rows(&result.table("runtime.solve_metrics").unwrap()).unwrap());
        let variables = solve_variables::Row::rows(&result.table("runtime.solve_variables").unwrap()).unwrap();
        json!({"key":index,"run_id":result.run_id,"execution":match result.report().unwrap() { RunReport::Modeling(reports) => match &reports[0].outcome { pse_runtime::math::solves::Outcome::Native(_) => "direct native", pse_runtime::math::solves::Outcome::Constant(_) => "complete original evaluation with automatic native components", pse_runtime::math::solves::Outcome::Rejected(_) => "rejected" }, _ => "other" },"native_metrics":native,"variables":checked_values(&variables, *expected)})
    }).collect()
}

async fn persisted(
    runtime: &Runtime,
    owner: &WorkflowRuntime,
    points: &[(u32, String, Option<String>)],
    numerical: &mut observations::Observations,
) -> Vec<Value> {
    let mut observations = Vec::new();
    assert_eq!(points.len(), VALUES.len());
    assert_eq!(
        points
            .iter()
            .map(|(_, run, _)| run)
            .collect::<BTreeSet<_>>()
            .len(),
        points.len(),
        "durable occurrences must retain distinct run identities"
    );
    for (index, expected) in VALUES.into_iter().enumerate() {
        let (occurrence, run, attempt) = &points[index];
        assert_eq!(*occurrence, index as u32);
        let attempt = attempt
            .as_ref()
            .expect("usable occurrence has exact producing attempt");
        let rows = stored_rows::<solve_metrics::Row>(
            runtime,
            owner,
            run,
            attempt,
            "runtime.solve_metrics",
        )
        .await;
        numerical.persisted_metrics(&rows);
        let native = metrics(rows);
        numerical.rows(
            &stored_rows::<solve_strategy_events::Row>(
                runtime,
                owner,
                run,
                attempt,
                "runtime.solve_strategy_events",
            )
            .await,
        );
        let variables = stored_rows::<solve_variables::Row>(
            runtime,
            owner,
            run,
            attempt,
            "runtime.solve_variables",
        )
        .await;
        let values = checked_values(&variables, expected);
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
    let parallel = spec["adapter"] == "in-process-parallel";
    assert!(durable_mode || parallel || spec["adapter"] == "in-process");
    let occurrences = if parallel { 16 } else { VALUES.len() };
    let threads = if parallel { 16 } else { 1 };
    assert_eq!(spec["blocks"].as_u64(), Some(occurrences as u64));
    assert_eq!(spec["threads"].as_u64(), Some(threads));
    let values: Vec<_> = (0..occurrences)
        .map(|index| VALUES[index % VALUES.len()])
        .collect();
    let name = spec["id"].as_str().unwrap();
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads as usize)
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
                let owner=support::study_owner(parallel);
                let budget=owner.runtime.budget();
                assert_eq!(budget.threads.pool_threads.get(), threads as usize);
                if parallel {
                    assert_eq!(budget.memory_limit_bytes.get(), 128_usize << 30);
                    assert_eq!(budget.math.jobs, 32);
                    assert!(std::thread::available_parallelism().unwrap().get() >= 16,
                        "reference parallel workload requires sixteen effective CPU permits; no reduced profile fallback");
                }
                let runtime_profile=json!({"threads":budget.threads.pool_threads.get(),"target_partitions":budget.threads.target_partitions.get(),"jobs":budget.math.jobs,"pool_limit_bytes":budget.memory_limit_bytes.get(),"math_policy":format!("{:?}",budget.math)});
                let runtime=if durable_mode { runtime(&owner) } else {
                    runtime(&owner).with_durability(workflow::Durability::Ephemeral)
                };
                let cancel=CancelSource::new();
                let database_setup=started.elapsed();
                let sources=support::sources(support::SOURCE);
                phases.reset();
                let before=owner.runtime.math().preparations();
                let started=Instant::now();
                let physical=support::physical(&runtime,&owner,&sources).await;
                let package=runtime.modeling_from_documents(&support::admitted_documents(&owner,&sources.modeling),physical.clone(),&cancel).await.unwrap();
                let definition=definition(&package,&sources,&physical,&values,parallel).await;
                let admission=started.elapsed();
                let admission_counts=support::counts(before,owner.runtime.math().preparations());
                let admission_phases=phases.report(occurrences as u64);
                let admission_constructions=phases.constructions();
                // Each adapter begins with source/definition admission complete;
                // the reference parallel selector has its own declared shared profile.
                let before=owner.runtime.math().preparations();
                phases.reset();
                let started=Instant::now();
                let mut worker_passes=Vec::new();
                let (elapsed,submission,submission_counts,worker_counts,result_read_seconds,outcomes,metrics) = if durable_mode {
                    let start=Instant::now();
                    let handle=runtime.start_defined_study(sources.physical.clone(),definition.clone(),&cancel).await.unwrap();
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
                                if handle.status().await.unwrap().state==StudyState::Concluded { terminal=true; break; }
                                tokio::time::sleep(Duration::from_millis(10)).await;
                            },
                            workflow::Processed::Ran {record,..}=>worker_passes.push(json!({"seconds":seconds,"outcome":record.attempt.as_ref().unwrap().outcome,"solutions":record.solutions.len(),"preparations":support::counts(pass_before,owner.runtime.math().preparations())})),
                            workflow::Processed::Settled{..}|workflow::Processed::Finalized{..}=>worker_passes.push(json!({"seconds":seconds,"preparations":support::counts(pass_before,owner.runtime.math().preparations())})),
                        }
                        assert!(worker_passes.len()<=32,"unexpected study retry");
                        if handle.status().await.unwrap().state==StudyState::Concluded { terminal=true; break; }
                    }
                    let elapsed=started.elapsed();
                    assert!(terminal,"study did not conclude during the bounded serial drain");
                    let worker_counts=support::counts(worker_before,owner.runtime.math().preparations());
                    let read=Instant::now();
                    let status=handle.status().await.unwrap();
                    assert_eq!(status.state,StudyState::Concluded,"{status:?}");
                    let outcomes:Vec<_>=status.points.into_iter().map(|p|p.outcome.unwrap()).collect();
                    check(&outcomes, occurrences, parallel);
                    let retained=handle.result().await.unwrap().unwrap();
                    let metrics=persisted(&runtime,&owner,&retained.points,&mut numerical).await;
                    (elapsed,submission,submission_counts,worker_counts,read.elapsed().as_secs_f64(),outcomes,metrics)
                } else {
                    let report=package.study(&definition,occurrences,&cancel).await.unwrap();
                    let elapsed=started.elapsed();
                    check(&report.outcomes, occurrences, parallel);
                    let read=Instant::now();
                    let metrics=ephemeral(&report,&values,&mut numerical,parallel);
                    (elapsed,Duration::ZERO,json!({"views":0,"observations":0,"rebuilt":0,"shared":0}),serde_json::to_value(report.preparations).unwrap(),read.elapsed().as_secs_f64(),report.outcomes,metrics)
                };
                numerical.preparations(before,owner.runtime.math().preparations());
                let execution_counts=support::counts(before,owner.runtime.math().preparations());
                let execution_phases=phases.report(occurrences as u64);
                let peak=owner.runtime.observation_peak_bytes();
                let rss=owner.runtime.report().unwrap().process_peak_rss_bytes;
                let pool=owner.runtime.pool();
                let retained=pool.reserved();
                let record=json!({"runtime_profile":runtime_profile,"seconds":{"database_setup":database_setup.as_secs_f64(),"source_and_definition_admission":admission.as_secs_f64(),"execution_total":elapsed.as_secs_f64(),"submission":submission.as_secs_f64(),"result_read":result_read_seconds},"admission_preparations":admission_counts,"submission_preparations":submission_counts,"worker_preparations":worker_counts,"execution_preparations":execution_counts,"admission_phases":admission_phases,"execution_phases":execution_phases,"worker_passes":worker_passes,"outcomes":outcomes,"point_metrics":metrics,"numerical_observations":numerical.json(),"definition":definition,"pool_peak_bytes":peak,"process_peak_rss_bytes":rss,"retained_runtime_bytes":retained});
                drop(package); drop(physical); drop(runtime);
                owner.cleanup_fixtures().await.unwrap();
                drop(owner);
                tokio::task::yield_now().await;
                let mut record=record;
                record["admission_constructions"]=admission_constructions;
                record["execution_constructions"]=phases.constructions();
                record["after_runtime_teardown_bytes"]=pool.reserved().into();
                assert_eq!(pool.reserved(), 0, "study allocations survive all runtime owners");
                (elapsed,record)
            });
            timed+=elapsed; records.push(record);
        }
        timed
    }));
    group.finish();
    let maximum = |field: &str| records.iter().filter_map(|r| r[field].as_u64()).max();
    let record = json!({"id":name,"workload":spec,"iterations":records.len(),"occurrences":occurrences,"threads":threads,"native_threads":1,"runtime_profile":records.first().map(|r|&r["runtime_profile"]),"math_policy":records.first().map(|r|&r["runtime_profile"]["math_policy"]),"pool_limit_bytes":records.first().map(|r|&r["runtime_profile"]["pool_limit_bytes"]),"pool_peak_bytes":maximum("pool_peak_bytes"),"process_peak_rss_bytes":maximum("process_peak_rss_bytes"),"retained_runtime_bytes":maximum("retained_runtime_bytes"),"retained_runtime_scope":"shared pool after occurrence reports or connected readers release; excludes external canonical server storage","after_case_teardown_bytes":maximum("after_runtime_teardown_bytes"),"after_retained_runtime_teardown_bytes":maximum("after_runtime_teardown_bytes"),"adapter":if durable_mode {"standalone durable serial work_once"} else if parallel {"explicit ephemeral sixteen independent Ipopt lanes; admitted automatic blocks retain their component-native reports"} else {"explicit ephemeral eight-point KINSOL continuation"},"cache_state":"fresh shared runtime per iteration; source/definition admission precedes execution","timed_scope":if durable_mode {"canonical submission, serial work_once passes including per-point persistence and finalization"} else {"local admitted ephemeral study dispatch including preparation, native attempts and original joined reports; Arrow result projection outside timer"},"persistence_scope":if durable_mode {"canonical scientific retention in standalone worker clocks; exact reopened per-point metrics outside timer"} else {"ephemeral original reports and checked owned result tables; no canonical result retention"},"preparation_counter_scope":"calling process shared math owner; durable work_once runs in that same process", "phase_scope":"execution dispatch and post-dispatch table projection or canonical reopening; result reads are separately clocked", "rss_scope":"whole benchmark process lifetime high-water mark; includes setup, tables and native work, excludes canonical server", "acceptance":{"distinct_occurrence_results":occurrences,"original_root_checks":"x equals assignment and y equals3, absolute tolerance1e-7", "ordering":if parallel {"independent fresh points, exact definition order in outcomes"} else {"original eight-point continuation chain"}},"sampling":"10 flat Criterion samples; smoke runs the same acceptance once without measurement; setup/admission and result projection or reopening outside timer","records":records});
    std::fs::write(
        output.join(format!("{name}-memory.json")),
        serde_json::to_vec_pretty(&record).unwrap(),
    )
    .unwrap();
}
