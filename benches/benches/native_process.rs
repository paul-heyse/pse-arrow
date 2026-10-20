// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Application-level case rebuilds; Cargo compilation is untimed cached setup.
// Match the runtime's finite depth for nested workflow auto-trait proofs.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "qualification workloads fail on invalid or incomplete execution"
)]
#[path = "audit/mod.rs"]
mod audit;
#[path = "native_process/compact.rs"]
mod compact;
#[path = "native_process/curved_fit.rs"]
mod curved_fit;
#[path = "native_process/extended.rs"]
mod extended;
#[path = "../../tests/support/plan14.rs"]
mod fixture;
#[path = "k4/accuracy.rs"]
mod k4_accuracy;
#[path = "k4/studies.rs"]
mod k4_studies;
#[path = "k4/support.rs"]
mod k4_support;
#[path = "native_process/observations.rs"]
mod observations;
#[path = "native_process/phases.rs"]
mod phases;
#[path = "native_process/restart.rs"]
mod restart;
#[path = "native_process/results_analysis.rs"]
mod results_analysis;
#[path = "native_process/review_cases.rs"]
mod review_cases;
#[path = "native_process/shooting.rs"]
mod shooting;
#[path = "native_process/source_reuse.rs"]
mod source_reuse;
use criterion::{Criterion, criterion_group, criterion_main};
use fixture::*;
use pse_backend_native::solve::{Backend, Metric, Termination};
use pse_model::generated::identities::DeclarationId;
use pse_runtime::{
    CancelSource,
    workflow::{ModelingPackage, RunReport},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroUsize,
    path::PathBuf,
    time::{Duration, Instant},
};

fn mark(phases: &mut BTreeMap<String, f64>, name: &str, started: Instant) {
    *phases.entry(name.into()).or_default() += started.elapsed().as_secs_f64();
}
fn observe_joined_cancellation(
    result: &pse_runtime::workflow::RunResult,
    observations: &mut observations::Observations,
    service: &pse_runtime::math::MathService,
) {
    use pse_backend_native::ProblemError;
    use pse_model::generated::enums::{NumericalAttemptObservation, NumericalEventKind};
    use pse_model::strategy::{Phase, Scope};
    use pse_runtime::{math::MathRuntimeError, workflow::WorkflowError};
    assert!(
        !result.usable(),
        "cancelled work cannot establish usable science"
    );
    match result.report() {
        Ok(RunReport::Modeling(reports)) => {
            assert!(
                matches!(&reports[0].outcome, pse_runtime::math::solves::Outcome::Native(report)
                if report.termination.category == Termination::Cancelled)
            );
            for (step, report) in reports.iter().enumerate() {
                observations.modeling(report, step, service);
            }
        }
        Err(WorkflowError::Math(MathRuntimeError::Strategy { cause, trace })) => {
            let MathRuntimeError::Solve(ProblemError::Math(pse_math::MathError::Typed {
                cause: provider_cause,
                ..
            })) = cause.as_ref()
            else {
                panic!("unexpected strategy cancellation cause: {cause}");
            };
            assert!(matches!(
                provider_cause.as_error().downcast_ref::<ProblemError>(),
                Some(ProblemError::Provider(
                    pse_kernels::ProviderError::Cancelled
                ))
            ));
            // Automatic blocks retain native facts when cancellation prevents original assessment.
            let summary = result.attempt_summary().unwrap();
            assert!(!summary.usable);
            assert_eq!(summary.omitted_steps, 0);
            assert_eq!(summary.omitted_native_attempts, 0);
            let attempt = summary
                .native_attempts
                .iter()
                .find(|attempt| {
                    attempt.record.backend == Backend::Ipopt
                        && attempt.record.termination.category == Termination::Cancelled
                        && attempt
                            .record
                            .work
                            .evaluations
                            .is_some_and(|count| count > 0)
                })
                .expect("entered Ipopt cancellation must retain its native report and work");
            let step = summary
                .steps
                .iter()
                .find(|step| step.step == attempt.step)
                .unwrap();
            assert_eq!(step.omitted_decisions, 0);
            let (started_index, started) = step
                .decisions
                .iter()
                .enumerate()
                .find(|(_, event)| {
                    event.mechanism == attempt.mechanism
                        && event.phase == Phase::Native
                        && event.kind == NumericalEventKind::Started
                })
                .expect("actual native entry must precede cancellation");
            assert!(started.decision.is_some());
            let terminal = |kind| {
                step.decisions.iter().enumerate().find(|(_, event)| {
                    event.mechanism == started.mechanism
                        && event.decision == started.decision
                        && event.phase == Phase::Native
                        && event.kind == kind
                        && event.observation == Some(NumericalAttemptObservation::Cancelled)
                })
            };
            let (finished_index, finished) = terminal(NumericalEventKind::Finished)
                .expect("entered cancellation must retain its native finish event");
            let (abandoned_index, abandoned) = terminal(NumericalEventKind::Abandoned)
                .expect("entered cancellation must retain its original abandonment");
            assert!(started_index < finished_index && finished_index < abandoned_index);
            assert!(finished.permission.is_none() && abandoned.permission.is_none());
            let work = finished
                .work
                .expect("native cancellation retains its actual charge");
            assert_eq!(work.phase, Phase::Native);
            assert_eq!(work.scope, Scope::Task);
            assert!(work.observed.attempts > 0);
            assert!(
                work.observed
                    .evaluations
                    .is_some_and(|count| count >= attempt.record.work.evaluations.unwrap())
            );
            assert_eq!(
                abandoned.work,
                Some(work),
                "abandonment preserves the original charging owner and work"
            );
            observations.rows(
                trace
                    .rows(result.run_id, attempt.step, service, 0..trace.event_count())
                    .unwrap()
                    .map(Result::unwrap),
            );
        }
        Ok(_) => panic!("wrong cancellation report family"),
        Err(error) => panic!("unexpected cancellation failure: {error}"),
    }
}
/// Vary instance count by composing the authored unit, without restating its equations.
async fn heater_blocks(
    package: &ModelingPackage,
    blocks: usize,
) -> (ModelingPackage, DeclarationId) {
    let engineering = pse_model::numerics::DEFAULT_ENGINEERING_ACCURACY;
    let temperature_tolerance = engineering * 350.0;
    let flow_tolerance = engineering * 5.0;
    let mut fixture = String::new();
    let mut children = String::from(
        "permission selected_unknown_fit families(pcsaft_parameters.nonassociating,properties.predictive_rule) allow_unknown true allow_extrapolation false;",
    );
    for i in 0..blocks {
        fixture.push_str(&format!("fix block{i}.duty=16204.445642740735{{W}};"));
        children.push_str(&format!("child block{i}:homogeneous_units.HeaterRecycle=homogeneous_units.HeaterRecycle(selected=vessel_fixtures.alkanes,law=pcsaft_data.potential,ideal_h=vessel_fixtures.ideal_enthalpy,composition=vessel_fixtures.fraction); expect block{i}.phase.T==350{{K}} tolerance {temperature_tolerance}{{K}}; expect block{i}.recycle==5{{mol/s}} tolerance {flow_tolerance}{{mol/s}};"));
    }
    let source = format!(
        "@id(\"b70ab2554b57594e8d2b75288e80da8e\") package homogeneous_fixtures {{test workload fixture {{dof 0; route steady; procedure solve; {fixture}}} {{{children}}} }}"
    );
    let extra = pse_authoring::language::parse(
        &source,
        pse_ids::named_id(pse_ids::SemanticId::NIL, "process-cost-heaters"),
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let case = extra
        .iter()
        .find(|r| r.name == "workload")
        .unwrap()
        .declaration_id;
    let mut rows = package.declarations().await.unwrap().to_vec();
    rows.extend(
        extra
            .into_iter()
            .filter(|r| r.value.kind.as_str() != "package"),
    );
    (package.with_declarations(rows).await.unwrap(), case)
}
fn process(c: &mut Criterion) {
    use tracing_subscriber::prelude::*;
    let compiler_phases = phases::Phases::default();
    let spec: serde_json::Value =
        serde_json::from_str(&std::env::var("PSE_PROCESS_COST_SPEC").unwrap()).unwrap();
    let id = spec["id"].as_str().unwrap();
    let mut audit = audit::Audit::start(id, &spec);
    tracing_subscriber::registry()
        .with(compiler_phases.clone())
        .with(audit.capture_layer())
        .init();
    let _audit_context = audit.enter();
    let operation = spec["operation"].as_str().unwrap();
    let reuse = spec["reuse"].as_str().unwrap();
    let blocks = spec["blocks"].as_u64().unwrap() as usize;
    let threads = spec["threads"].as_u64().unwrap() as usize;
    let output = PathBuf::from(std::env::var("PSE_PROCESS_COST_OUTPUT").unwrap());
    if operation == "source-edit-copy" {
        source_reuse::measure(c, &spec, &output, &compiler_phases);
        audit.complete();
        return;
    }
    if operation == "result_analysis" {
        results_analysis::measure(c, &spec, &output, &compiler_phases);
        audit.complete();
        return;
    }
    if operation == "k4-study" {
        k4_studies::measure(c, &spec, &output, &compiler_phases);
        audit.complete();
        return;
    }
    if operation == "k4-restart" {
        restart::measure(c, &spec, &output, &compiler_phases);
        audit.complete();
        return;
    }
    if operation == "k4-accuracy" {
        k4_accuracy::measure(c, &spec, &output, &compiler_phases);
        audit.complete();
        return;
    }
    if spec["extended"].as_bool() == Some(true) || matches!(operation, "vessel" | "fit") {
        extended::measure(c, &spec, &output, &compiler_phases);
        audit.complete();
        return;
    }
    assert!(
        matches!(operation, "heater" | "flash" | "cancellation" | "results"),
        "unsupported process benchmark operation: {operation}"
    );
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()
        .unwrap();
    let flash = DeclarationId::from_id(
        pse_ids::SemanticId::parse_hex("040af20814bc57abb565c3c7f680be05").unwrap(),
    );
    let mut demand = Vec::new();
    if reuse == "warm" {
        demand.push(if operation == "flash" {
            "root.liquid.T".into()
        } else {
            "block0.phase.T".into()
        });
    }
    if spec["difficult"].as_bool() == Some(true) {
        demand.extend(["root.liquid.rho".into(), "root.vapor.rho".into()]);
    }
    let seed = |owner: &WorkflowRuntime| {
        let target = runtime(owner);
        (
            executor.block_on(seed_package_on(owner, target.clone())),
            target,
        )
    };
    let retained = if reuse == "cold" {
        None
    } else {
        let owner = WorkflowRuntime::with_threads(NonZeroUsize::new(threads).unwrap()).unwrap();
        let retained = audit::with_owner_sync(&owner, &executor, || {
            let _stage = audit::stage("retained_cold_setup");
            let (source, target) = seed(&owner);
            let (package, case) = if operation == "flash" {
                (source.clone(), flash)
            } else {
                executor.block_on(heater_blocks(&source, blocks))
            };
            let execution = executor
                .block_on(package.declared_execution_with_demand(
                    case,
                    compiler(),
                    profile(Backend::Ipopt, false),
                    Default::default(),
                    seed_limits(),
                    demand.clone(),
                    &audit::cancel_source(),
                ))
                .unwrap();
            let prepared = executor
                .block_on(package.prepare_declared(&execution, &audit::cancel_source()))
                .unwrap();
            let result = executor
                .block_on(audit::scope(async {
                    audit::wait(prepared.start().unwrap()).await
                }))
                .unwrap();
            executor.block_on(audit::export_result(&target, &result));
            authored_success(&result);
            (source, package, case, target)
        });
        Some((owner, retained.0, retained.1, retained.2, retained.3))
    };
    compiler_phases.reset();
    let mut variables = BTreeSet::new();
    let mut peak = 0;
    let mut rss = 0;
    let mut retained_bytes = 0;
    let mut after_teardown_bytes = 0;
    let mut iterations = 0_u64;
    let mut records = observations::InvocationRecords::new(&output, id);
    let mut group = c.benchmark_group("process");
    group
        .sample_size(10)
        .sampling_mode(criterion::SamplingMode::Flat)
        .warm_up_time(Duration::from_millis(250))
        .measurement_time(Duration::from_secs(1));
    group.bench_function(id, |b| {
        b.iter(|| {
            let invocation_begin = Instant::now();
            let mut observations = observations::Observations::default();
            let mut phases = BTreeMap::new();
            let mut native_seconds = BTreeMap::<String, f64>::new();
            compiler_phases.reset();
            let executed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| pse_runtime::invocation::counted_sync(|| {
            let _entered = executor.enter();
            let begin = Instant::now();
            let local = if retained.is_none() {
                Some(WorkflowRuntime::with_threads(NonZeroUsize::new(threads).unwrap()).unwrap())
            } else {
                None
            };
            let owner = match &retained {
                Some((owner, _, _, _, _)) => owner,
                None => local.as_ref().unwrap(),
            };
            let teardown_begin = audit::with_owner_sync(owner, &executor, || {
                let _iteration_stage = audit::stage("case_execution");
                owner.runtime.reset_observation_peak();
                mark(&mut phases, "runtime_admission", begin);
                let begin = Instant::now();
                let source_stage = audit::stage("source_admission");
                let (target, mut package, case) = if let Some((_, source, package, case, target)) =
                    &retained
                {
                    let (package, case) = if reuse == "structure" {
                        executor.block_on(heater_blocks(source, blocks + (iterations as usize % 2)))
                    } else {
                        (package.clone(), *case)
                    };
                    (target.clone(), package, case)
                } else {
                    let (source, target) = seed(owner);
                    let (package, case) = if operation == "flash" {
                        (source, flash)
                    } else {
                        executor.block_on(heater_blocks(&source, blocks))
                    };
                    (target, package, case)
                };
                if reuse == "specialization" {
                    let mut rows = executor.block_on(package.declarations()).unwrap().to_vec();
                    let row = rows
                        .iter_mut()
                        .find(|r| {
                            r.declaration_id
                                == DeclarationId::from_id(
                                    pse_ids::SemanticId::parse_hex(
                                        "0ed2b62ea07d570bb1db5f48ec53354e",
                                    )
                                    .unwrap(),
                                )
                        })
                        .unwrap();
                    row.value.contribution.as_mut().unwrap().expression =
                        format!("{}*fraction*recycle", 1.0 + (iterations + 1) as f64 * 1e-10);
                    package = executor.block_on(package.with_declarations(rows)).unwrap();
                }
                mark(&mut phases, "source_admission", begin);
                drop(source_stage);
                let prepare_stage = audit::stage("case_preparation");
                let begin = Instant::now();
                let cancel = audit::cancel_source();
                let mut execution = executor
                    .block_on(package.declared_execution_with_demand(
                        case,
                        compiler(),
                        profile(Backend::Ipopt, false),
                        Default::default(),
                        seed_limits(),
                        demand.clone(),
                        &cancel,
                    ))
                    .unwrap();
                if reuse == "warm" {
                    let path = if operation == "flash" {
                        "root.liquid.T"
                    } else {
                        "block0.phase.T"
                    };
                    execution.analysis.case.values.insert(
                        path.into(),
                        if operation == "flash" { 280.0 } else { 313.15 }
                            + (iterations + 1) as f64 * 1e-9,
                    );
                }
                if spec["difficult"].as_bool() == Some(true) {
                    execution
                        .analysis
                        .case
                        .values
                        .insert("root.liquid.rho".into(), 12400.0 * 1.02);
                    execution
                        .analysis
                        .case
                        .values
                        .insert("root.vapor.rho".into(), 1000.0 * 1.02);
                }
                let prepared = executor
                    .block_on(package.prepare_declared(&execution, &cancel))
                    .unwrap();
                variables.insert(
                    prepared
                        .model
                        .case
                        .compiled()
                        .plan
                        .structure()
                        .free_variables()
                        .count(),
                );
                let handle = prepared.start().unwrap();
                mark(&mut phases, "case_preparation_and_start", begin);
                drop(prepare_stage);
                let native_stage = audit::stage("numerical_execution");
                let begin = Instant::now();
                if operation == "cancellation" {
                    executor.block_on(async {
                        while handle.progress().0.is_empty()
                            && handle.result().is_none()
                            && !audit::stop_requested()
                        {
                            tokio::task::yield_now().await;
                        }
                        if !audit::stop_requested() {
                            assert!(
                                handle.result().is_none(),
                                "case completed before observable cancellation checkpoint"
                            );
                        }
                    });
                    mark(&mut phases, "native_to_first_event", begin);
                    let stop = Instant::now();
                    handle.cancel();
                    let result = executor
                        .block_on(audit::scope(audit::wait_ref(&handle)))
                        .unwrap();
                    mark(&mut phases, "cancellation_to_join", stop);
                    executor.block_on(audit::export_result(&target, &result));
                    observations.invocation(&result);
                    observe_joined_cancellation(&result, &mut observations, owner.runtime.math());
                    drop(result);
                } else {
                    let result = executor
                        .block_on(audit::scope(audit::wait_ref(&handle)))
                        .unwrap();
                    executor.block_on(audit::export_result(&target, &result));
                    mark(&mut phases, "native_join", begin);
                    drop(native_stage);
                    let _assessment_stage = audit::stage("original_assessment_and_results");
                    let begin = Instant::now();
                    let report = authored_success(&result);
                    let RunReport::Modeling(_) = result.report().unwrap() else {
                        panic!("wrong report")
                    };
                    observations.run(&result, owner.runtime.math());
                    if let pse_runtime::math::solves::Outcome::Native(report) = &report.outcome {
                        for (name, value) in &report.metrics {
                            if (name.ends_with(".seconds") || name.starts_with("timing."))
                                && let Metric::Real(value) = value
                            {
                                *native_seconds.entry(name.clone()).or_insert(0.0) += value;
                            }
                        }
                    }
                    std::hint::black_box(result.report().unwrap());
                    if !matches!(operation, "vessel" | "fit") {
                        std::hint::black_box(result.table("runtime.solve_variables").unwrap());
                    }
                    mark(&mut phases, "physical_validation_and_results", begin);
                    if operation == "results" {
                        let _read_stage = audit::stage("connected_result_reads");
                        let begin = Instant::now();
                        let run = result.canonical_run_key().unwrap();
                        let attempt = result.canonical_attempt_key().unwrap();
                        for _ in 0..spec["result_reads"].as_u64().unwrap_or(1) {
                            let mut reader = executor
                                .block_on(target.results(
                                    run,
                                    attempt,
                                    "runtime.solve_variables",
                                    0,
                                    u64::MAX,
                                    pse_columnar::CancellationToken::new(),
                                ))
                                .unwrap();
                            let mut count = 0;
                            while let Some(batch) = executor.block_on(reader.next_batch()).unwrap()
                            {
                                count += batch.num_rows();
                            }
                            assert_eq!(
                                count,
                                result
                                    .table("runtime.solve_variables")
                                    .unwrap()
                                    .batch()
                                    .num_rows()
                            );
                        }
                        mark(&mut phases, "connected_result_reopen", begin);
                    }
                    drop(result);
                }
                let teardown_begin = Instant::now();
                drop(handle);
                drop(prepared);
                drop(package);
                drop(target);
                teardown_begin
            });
            let _teardown_stage = audit::stage("case_teardown");
            let begin = teardown_begin;
            if retained.is_none() {
                let cleanup = executor.block_on(owner.cleanup_fixtures());
                audit::cleanup_result(cleanup.is_ok());
                cleanup.unwrap();
            }
            peak = peak.max(owner.runtime.observation_peak_bytes());
            rss = rss.max(
                owner
                    .runtime
                    .report()
                    .unwrap()
                    .process_peak_rss_bytes
                    .unwrap(),
            );
            let pool = owner.runtime.pool();
            retained_bytes = retained_bytes.max(pool.reserved());
            drop(local);
            after_teardown_bytes = after_teardown_bytes.max(pool.reserved());
            mark(&mut phases, "case_teardown", begin);
            })));
            let preparation_counts = match &executed { Ok((_, counts)) => Some(*counts), Err(_) => None };
            if let Some(counts) = preparation_counts { observations.counted_preparations(counts); }
            records.push(serde_json::json!({"seconds":invocation_begin.elapsed().as_secs_f64(),
                "phase_seconds":phases,"native_seconds":native_seconds,"compiler_phases":compiler_phases.report(1),
                "compiler_constructions":compiler_phases.constructions(),"numerical_observations":observations.json(),
                "preparations":preparation_counts,"start_role":reuse,
                "outcome":if executed.is_ok() { "completed" } else { "panicked_incomplete" },
                "clock_scope":"whole executed harness closure including setup, scientific execution, requested access and teardown; phase intervals inclusive"}));
            if let Err(panic) = executed { records.flush(); std::panic::resume_unwind(panic); }
            iterations += 1;
        })
    });
    group.finish();
    // Warm samples deliberately retain their original revision and runtime.
    // Observe that owner's release separately from each sample's case teardown.
    let retained_pool = retained
        .as_ref()
        .map(|(owner, _, _, _, _)| owner.runtime.pool());
    if let Some((owner, source, package, _, target)) = retained {
        // These source handles own canonical protection. End their retained
        // sample lifetime before asking the fixture owner to close its store.
        drop(source);
        drop(package);
        drop(target);
        let cleanup = executor.block_on(owner.cleanup_fixtures());
        audit::cleanup_result(cleanup.is_ok());
        cleanup.unwrap();
    }
    drop(executor);
    let final_retained_runtime_bytes = retained_pool.map(|pool| pool.reserved());
    records.flush();
    std::fs::write(output.join(format!("{id}-memory.json")),serde_json::to_vec_pretty(&serde_json::json!({
        "id":id,"iterations":iterations,"pool_peak_bytes":peak,"process_peak_rss_bytes":rss,
        "retained_runtime_bytes":retained_bytes,"after_case_teardown_bytes":after_teardown_bytes,
        "after_retained_runtime_teardown_bytes":final_retained_runtime_bytes,
        "effective_process_parallelism":std::thread::available_parallelism().unwrap().get(),
        "workload":spec,"variables_observed":variables,"threads":threads,"native_threads":1,
        "observations_file":format!("{id}-observations.jsonl"),
        "observation_population":"one JSONL record per executed closure, including warmup; Criterion measured samples separate; membership unavailable",
        "phase_scope":"inclusive synchronous compiler spans; cache hits do not execute spans; absent phases performed no work",
        "unavailable_submetrics":["JIT is not enabled", "native conversion and property-state construction are included in preparation/native execution, without separate clocks"],
        "scope":"case source admission, preparation/rebuild, joined native execution, validation/results, connected result reopening and teardown; application compilation excluded",
        "sampling":"Actual operation count is iterations. Statistical mode configures 10 flat Criterion samples, 250 ms warmup and a 1 s target; smoke execution is not a statistical timing estimate.",
        "memory_scope":"pool observation per operation; case teardown retains the warm runtime until sampling ends; final retained-runtime teardown is null for cold cases; process lifetime VmHWM from the dedicated benchmark process"
    })).unwrap()).unwrap();
    audit.complete();
}
fn configuration() -> Criterion {
    Criterion::default().output_directory(
        &PathBuf::from(std::env::var("PSE_PROCESS_COST_OUTPUT").unwrap()).join("criterion"),
    )
}
criterion_group! {name=benches;config=configuration();targets=process}
criterion_main!(benches);
