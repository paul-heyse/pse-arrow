// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Application-level case rebuilds; Cargo compilation is untimed cached setup.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "qualification workloads fail on invalid or incomplete execution"
)]
#[path = "native_process/extended.rs"]
mod extended;
#[path = "../../tests/support/plan14.rs"]
mod fixture;
#[path = "native_process/phases.rs"]
mod phases;
use criterion::{Criterion, criterion_group, criterion_main};
use fixture::*;
use pse_backend_native::solve::{Backend, Metric, Termination};
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
/// Vary instance count by composing the authored unit, without restating its equations.
fn heater_blocks(package: &ModelingPackage, blocks: usize) -> (ModelingPackage,pse_ids::SemanticId) {
    let mut fixture=String::new();
    let mut children=String::new();
    for i in 0..blocks {
        fixture.push_str(&format!("fix block{i}.duty=16204.445642740735{{W}};"));
        children.push_str(&format!("child block{i}:homogeneous_units.HeaterRecycle=homogeneous_units.HeaterRecycle(selected=chem.alkanes,law=pcsaft.potential,ideal_h=vessel_fixtures.ideal_enthalpy,composition=vessel_fixtures.fraction); expect block{i}.phase.T==350{{K}} tolerance 0.00001{{K}}; expect block{i}.recycle==5{{mol/s}} tolerance 0.000001{{mol/s}};"));
    }
    let source=format!("@id(\"b70ab2554b57594e8d2b75288e80da8e\") package homogeneous_fixtures {{test workload fixture {{dof 0; run steady; {fixture}}} {{{children}}} }}");
    let extra=pse_authoring::language::parse(&source,pse_ids::named_id(pse_ids::SemanticId::NIL,"process-cost-heaters"),pse_authoring::language::IdentityPolicy::Named,Default::default()).unwrap();
    let case=extra.iter().find(|r|r.name=="workload").unwrap().declaration_id;
    let mut rows=package.declarations().to_vec();
    rows.extend(extra.into_iter().filter(|r|r.value.kind.as_str()!="package"));
    (package.with_declarations(rows).unwrap(),case)
}
fn process(c: &mut Criterion) {
    use tracing_subscriber::prelude::*;
    let compiler_phases = phases::Phases::default();
    tracing_subscriber::registry()
        .with(compiler_phases.clone())
        .init();
    let spec: serde_json::Value =
        serde_json::from_str(&std::env::var("PSE_PROCESS_COST_SPEC").unwrap()).unwrap();
    let id = spec["id"].as_str().unwrap();
    let operation = spec["operation"].as_str().unwrap();
    let reuse = spec["reuse"].as_str().unwrap();
    let blocks = spec["blocks"].as_u64().unwrap() as usize;
    let threads = spec["threads"].as_u64().unwrap() as usize;
    let output = PathBuf::from(std::env::var("PSE_PROCESS_COST_OUTPUT").unwrap());
    if spec["extended"].as_bool() == Some(true) || matches!(operation, "vessel" | "fit") {
        extended::measure(c, &spec, &output, &compiler_phases);
        return;
    }
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()
        .unwrap();
    let flash=pse_ids::SemanticId::parse_hex("040af20814bc57abb565c3c7f680be05").unwrap();
    let retained = if reuse == "cold" { None } else {
        let owner=WorkflowRuntime::with_threads(NonZeroUsize::new(threads).unwrap()).unwrap();
        let source=executor.block_on(seed_package(&owner));
        let (package,case)=if operation=="flash" {(source.clone(),flash)} else {heater_blocks(&source,blocks)};
        let prepared=executor.block_on(seed_prepare(&package,case,profile(Backend::Ipopt,false),&CancelSource::new())).unwrap();
        authored_success(&executor.block_on(prepared.start().unwrap().wait()).unwrap());
        Some((owner,source,package,case))
    };
    let mut phases = BTreeMap::new();
    compiler_phases.reset();
    let mut native_seconds = BTreeMap::new();
    let mut variables = BTreeSet::new();
    let mut peak = 0;
    let mut rss = 0;
    let mut retained_bytes = 0;
    let mut after_teardown_bytes = 0;
    let mut iterations = 0_u64;
    let mut group = c.benchmark_group("process");
    group
        .sample_size(10)
        .sampling_mode(criterion::SamplingMode::Flat)
        .warm_up_time(Duration::from_millis(250))
        .measurement_time(Duration::from_secs(1));
    group.bench_function(id, |b| b.iter(|| {
        let _entered = executor.enter();
        let begin=Instant::now();
        let local=if retained.is_none() {Some(WorkflowRuntime::with_threads(NonZeroUsize::new(threads).unwrap()).unwrap())} else {None};
        let owner=match &retained {Some((owner,_,_,_))=>owner,None=>local.as_ref().unwrap()};
        owner.runtime.reset_observation_peak();
        mark(&mut phases,"runtime_admission",begin);
        let begin=Instant::now();
        let (mut package,case)=if let Some((_,source,package,case))=&retained {
            if reuse=="structure" {heater_blocks(source,blocks+(iterations as usize%2))} else {(package.clone(),*case)}
        } else {
            let source=executor.block_on(seed_package(owner));
            if operation=="flash" {(source,flash)} else {heater_blocks(&source,blocks)}
        };
        if reuse=="specialization" {
            let mut rows=package.declarations().to_vec();
            let row=rows.iter_mut().find(|r|r.declaration_id==pse_ids::SemanticId::parse_hex("0ed2b62ea07d570bb1db5f48ec53354e").unwrap()).unwrap();
            row.value.contribution.as_mut().unwrap().expression=format!("{}*fraction*recycle",1.0+(iterations+1) as f64*1e-10);
            package=package.with_declarations(rows).unwrap();
        }
        mark(&mut phases,"source_admission",begin);
        let begin=Instant::now();
        let cancel=CancelSource::new();
        let mut analysis=executor.block_on(package.declared_analysis(case,pse_relations::generated::enums::ModelingAnalysisRoute::Steady,compiler(),profile(Backend::Ipopt,false),Default::default(),seed_limits(),&cancel)).unwrap();
        if reuse=="warm" {
            let path=if operation=="flash" {"root.liquid.T"} else {"block0.phase.T"};
            analysis.case.values.insert(path.into(),if operation=="flash" {280.0} else {313.15}+(iterations+1) as f64*1e-9);
        }
        if spec["difficult"].as_bool()==Some(true) {
            analysis.case.values.insert("root.liquid.rho".into(),12400.0*1.02);
            analysis.case.values.insert("root.vapor.rho".into(),1000.0*1.02);
        }
        let prepared=executor.block_on(package.prepare_analysis(&analysis,&cancel)).unwrap();
        variables.insert(prepared.model.case.compiled().plan.structure().free_variables().count());
        let handle=prepared.start().unwrap();
        mark(&mut phases,"case_preparation_and_start",begin);
        let begin=Instant::now();
        if operation=="cancellation" {
            executor.block_on(async {
                while handle.progress().0.is_empty() && handle.result().is_none() {tokio::task::yield_now().await;}
                assert!(handle.result().is_none(),"case completed before observable cancellation checkpoint");
            });
            mark(&mut phases,"native_to_first_event",begin);
            let stop=Instant::now();
            handle.cancel();
            let result=executor.block_on(handle.wait()).unwrap();
            mark(&mut phases,"cancellation_to_join",stop);
            let RunReport::Modeling(report)=result.report().unwrap() else {panic!("wrong report")};
            assert!(matches!(&report[0].outcome,pse_runtime::math::solves::Outcome::Native(r) if r.termination.category==Termination::Cancelled),"{report:?}");
            drop(result);
        } else {
            let result=executor.block_on(handle.wait()).unwrap();
            mark(&mut phases,"native_join",begin);
            let begin=Instant::now();
            let report=authored_success(&result);
            if let pse_runtime::math::solves::Outcome::Native(report)=&report.outcome {
                for (name,value) in &report.metrics {
                    if (name.ends_with(".seconds") || name.starts_with("timing.")) && let Metric::Real(value)=value {
                        *native_seconds.entry(name.clone()).or_insert(0.0)+=value;
                    }
                }
            }
            std::hint::black_box(result.report().unwrap());
            if !matches!(operation,"vessel"|"fit") {std::hint::black_box(result.table("runtime.solve_variables").unwrap());}
            mark(&mut phases,"physical_validation_and_results",begin);
            if operation=="publication" {
                let begin=Instant::now();
                let directory=tempfile::tempdir().unwrap();
                let base=url::Url::from_directory_path(directory.path()).unwrap();
                let mut parent = None;
                for _ in 0..spec["publications"].as_u64().unwrap_or(1) {
                    let command=result.prepare_publication(base.clone(),case,parent,&owner.cancel).unwrap();
                    let ticket=command.ticket.clone();
                    parent=Some(command.publication_id);
                    let committed=executor.block_on(command.commit(&owner.cancel)).unwrap();
                    let reopened=executor.block_on(runtime(owner).open(committed.clone(),&owner.cancel)).unwrap();
                    assert_eq!(reopened.root().version,committed.version);
                    if spec["publications"].is_number() {
                        for _ in 0..2 {
                            assert_eq!(executor.block_on(runtime(owner).settle_publication(&ticket,&owner.cancel)),
                                pse_runtime::workflow::PublicationSettlement::Committed{root:committed.clone()});
                        }
                    }
                }
                drop(directory);
                mark(&mut phases,"publication_reopen",begin);
            }
            drop(result);
        }
        let begin=Instant::now();
        drop(handle);
        drop(prepared);
        drop(package);
        peak=peak.max(owner.runtime.observation_peak_bytes());
        rss=rss.max(owner.runtime.report().unwrap().process_peak_rss_bytes.unwrap());
        let pool=owner.runtime.pool();
        retained_bytes=retained_bytes.max(pool.reserved());
        drop(local);
        after_teardown_bytes=after_teardown_bytes.max(pool.reserved());
        mark(&mut phases,"case_teardown",begin);
        iterations+=1;
    }));
    group.finish();
    // Warm samples deliberately retain their original revision and runtime.
    // Observe that owner's release separately from each sample's case teardown.
    let retained_pool = retained.as_ref().map(|(owner, _, _, _)| owner.runtime.pool());
    drop(retained);
    drop(executor);
    let final_retained_runtime_bytes = retained_pool.map(|pool| pool.reserved());
    for value in phases.values_mut() {
        *value /= iterations as f64;
    }
    for value in native_seconds.values_mut() {
        *value /= iterations as f64;
    }
    std::fs::write(output.join(format!("{id}-memory.json")),serde_json::to_vec_pretty(&serde_json::json!({
        "id":id,"iterations":iterations,"pool_peak_bytes":peak,"process_peak_rss_bytes":rss,
        "retained_runtime_bytes":retained_bytes,"after_case_teardown_bytes":after_teardown_bytes,
        "after_retained_runtime_teardown_bytes":final_retained_runtime_bytes,
        "effective_process_parallelism":std::thread::available_parallelism().unwrap().get(),
        "workload":spec,"variables_observed":variables,"threads":threads,"native_threads":1,
        "phase_seconds":phases,"native_seconds":native_seconds,
        "compiler_phases":compiler_phases.report(iterations),
        "phase_scope":"inclusive synchronous compiler spans; cache hits do not execute spans; absent phases performed no work",
        "unavailable_submetrics":["JIT is not enabled", "native conversion and property-state construction are included in preparation/native execution, without separate clocks"],
        "scope":"case source admission, preparation/rebuild, joined native execution, validation/results, optional publication and teardown; application compilation excluded",
        "sampling":"10 flat Criterion samples, 250 ms warmup, 1 s target measurement time (extended for slow operations)",
        "memory_scope":"pool observation per operation; case teardown retains the warm runtime until sampling ends; final retained-runtime teardown is null for cold cases; process lifetime VmHWM from the dedicated benchmark process"
    })).unwrap()).unwrap();
}
fn configuration() -> Criterion {
    Criterion::default().output_directory(
        &PathBuf::from(std::env::var("PSE_PROCESS_COST_OUTPUT").unwrap()).join("criterion"),
    )
}
criterion_group! {name=benches;config=configuration();targets=process}
criterion_main!(benches);
