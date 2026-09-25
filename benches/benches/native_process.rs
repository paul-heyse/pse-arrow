// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Application-level case rebuilds; Cargo compilation is untimed cached setup.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "qualification workloads fail on invalid or incomplete execution"
)]
#[path = "../../tests/support/plan14.rs"]
mod fixture;
#[path = "native_process/phases.rs"]
mod phases;
use criterion::{Criterion, criterion_group, criterion_main};
use fixture::*;
use pse_backend_native::solve::{Backend, Metric, Termination};
use pse_runtime::{
    CancelSource,
    workflow::{ModelRevision, RunReport},
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
fn process(c: &mut Criterion) {
    use tracing_subscriber::prelude::*;
    let compiler_phases = phases::Phases::default();
    tracing_subscriber::registry()
        .with(compiler_phases.clone())
        .init();
    let spec: serde_json::Value =
        serde_json::from_str(&std::env::var("PSE_PLAN14_COST_SPEC").unwrap()).unwrap();
    let id = spec["id"].as_str().unwrap();
    let operation = spec["operation"].as_str().unwrap();
    let reuse = spec["reuse"].as_str().unwrap();
    let blocks = spec["blocks"].as_u64().unwrap() as usize;
    let threads = spec["threads"].as_u64().unwrap() as usize;
    let output = PathBuf::from(std::env::var("PSE_PLAN14_COST_OUTPUT").unwrap());
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()
        .unwrap();
    let bindings = json("bindings.json");
    let dynamic = sid(&bindings["vessel_id"]);
    let root = sid(&bindings["root_case"]);
    let retained = if reuse == "cold" {
        None
    } else {
        let owner = WorkflowRuntime::with_threads(NonZeroUsize::new(threads).unwrap()).unwrap();
        let mut draft = executor.block_on(builder(&owner));
        if operation == "heater" && reuse != "structure" {
            resize(&mut draft, blocks);
        }
        let revision = draft.freeze().unwrap();
        let case = if operation == "flash" {
            revision
                .declaration()
                .cases
                .iter()
                .find(|case| case.name == "flash")
                .unwrap()
                .case_id
        } else {
            root
        };
        success(&executor.block_on(solve(&revision, case, Backend::Ipopt, false)));
        Some((owner, revision))
    };
    let mut phases = BTreeMap::new();
    compiler_phases.reset();
    let mut native_seconds = BTreeMap::new();
    let mut variables = BTreeSet::new();
    let mut peak = 0;
    let mut rss = 0;
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
        let owner=match &retained {Some((owner,_))=>owner,None=>local.as_ref().unwrap()};
        owner.runtime.reset_observation_peak();
        mark(&mut phases,"runtime_admission",begin);
        let begin=Instant::now();
        let mut draft=match &retained {Some((_,revision))=>revision.edit(),None=>executor.block_on(builder(owner))};
        if (operation=="heater" && reuse=="cold") || operation=="cancellation" {resize(&mut draft,blocks);}
        if reuse=="structure" {resize(&mut draft,blocks+(iterations as usize%2));}
        if reuse=="specialization" {
            let definition=draft.declaration_mut().definitions.iter_mut().find(|definition|definition.sources.iter().any(|source| source=="fraction * recycle")).unwrap();
            definition.sources[0]=format!("{} * fraction * recycle",1.+(iterations+1) as f64*1e-10);
        }
        if reuse=="warm" {
            let case=draft.declaration_mut().cases.iter_mut().find(|case|if operation=="flash" {case.name=="flash"} else {case.case_id==root}).unwrap();
            case.values[0].value+=(iterations+1) as f64*1e-9;
        }
        if spec["difficult"].as_bool()==Some(true) {
            let flash=draft.declaration_mut().cases.iter_mut().find(|case|case.name=="flash").unwrap();
            // Off-reference density guesses; physical acceptance tolerances stay unchanged.
            for value in flash.values.iter_mut().take(2) {value.value*=1.02;}
        }
        if matches!(operation,"vessel"|"fit") {vessel(&mut draft,false);}
        let fit=if operation=="fit" {Some(heat_fit(&mut draft,"transient"))} else {None};
        let revision:ModelRevision=draft.freeze().unwrap();
        mark(&mut phases,"source_admission",begin);
        let begin=Instant::now();
        let case=if operation=="flash" {revision.declaration().cases.iter().find(|case|case.name=="flash").unwrap().case_id} else {root};
        let handle=if operation=="vessel" {
            variables.insert(5);
            revision_prepare_simulation(&executor,&revision,dynamic).start().unwrap()
        } else if let Some((fit_id,profile))=fit {
            variables.insert(1);
            executor.block_on(revision.prepare_fit(fit_id,profile,compiler(),&CancelSource::new())).unwrap().start().unwrap()
        } else {
            let declaration=revision.declaration().cases.iter().find(|source|source.case_id==case).unwrap();
            let n=declaration.variables.iter().filter(|v|!v.fixed).count();
            variables.insert(n);
            executor.block_on(revision.prepare(case,profile(Backend::Ipopt,false),compiler(),&CancelSource::new())).unwrap().start().unwrap()
        };
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
            let RunReport::Solves(report)=result.report().unwrap() else {panic!("wrong report")};
            assert!(report.outcomes.iter().any(|outcome| matches!(outcome,pse_runtime::math::solves::Outcome::Native(r) if r.termination.category==Termination::Cancelled)),"{report:?}");
            drop(result);
        } else {
            let result=executor.block_on(handle.wait()).unwrap();
            mark(&mut phases,"native_join",begin);
            let begin=Instant::now();
            match result.report().unwrap() {
                RunReport::Solves(report)=>{
                    success(&result);
                    for outcome in &report.outcomes {
                        if let pse_runtime::math::solves::Outcome::Native(report)=outcome {
                            for (name,value) in &report.metrics {
                                if (name.ends_with(".seconds") || name.starts_with("timing.")) && let Metric::Real(value)=value {
                                    *native_seconds.entry(name.clone()).or_insert(0.)+=value;
                                }
                            }
                        }
                    }
                },
                RunReport::Simulation(report)=>{assert_eq!(report.termination,pse_backend_native::dynamics::Termination::Completed);conservation(&result,report.samples.len()*2);},
                RunReport::Fit(report)=>{assert!(matches!(report.solve.as_ref().unwrap().termination.category,Termination::Success|Termination::Acceptable));assert!(report.quality.as_ref().unwrap().feasible());near(report.candidate.as_ref().unwrap()[0],10.,2e-3);},
            }
            std::hint::black_box(result.report().unwrap());
            if !matches!(operation,"vessel"|"fit") {std::hint::black_box(result.table("runtime.solve_variables").unwrap());}
            mark(&mut phases,"physical_validation_and_results",begin);
            if operation=="publication" {
                let begin=Instant::now();
                let directory=tempfile::tempdir().unwrap();
                let base=url::Url::from_directory_path(directory.path()).unwrap();
                let command=result.prepare_publication(base,case,None,&owner.cancel).unwrap();
                let committed=executor.block_on(command.commit(&owner.cancel)).unwrap();
                let reopened=executor.block_on(runtime(owner).open(committed.clone(),&owner.cancel)).unwrap();
                assert_eq!(reopened.root().version,committed.version);
                drop(reopened);
                drop(directory);
                mark(&mut phases,"publication_reopen",begin);
            }
            drop(result);
        }
        let begin=Instant::now();
        drop(handle);
        drop(revision);
        peak=peak.max(owner.runtime.observation_peak_bytes());
        rss=rss.max(owner.runtime.report().unwrap().process_peak_rss_bytes.unwrap());
        drop(local);
        mark(&mut phases,"case_teardown",begin);
        iterations+=1;
    }));
    group.finish();
    drop(retained);
    drop(executor);
    for value in phases.values_mut() {
        *value /= iterations as f64;
    }
    for value in native_seconds.values_mut() {
        *value /= iterations as f64;
    }
    std::fs::write(output.join(format!("{id}-memory.json")),serde_json::to_vec_pretty(&serde_json::json!({
        "id":id,"iterations":iterations,"pool_peak_bytes":peak,"process_peak_rss_bytes":rss,
        "workload":spec,"variables_observed":variables,"threads":threads,"native_threads":1,
        "phase_seconds":phases,"native_seconds":native_seconds,
        "compiler_phases":compiler_phases.report(iterations),
        "phase_scope":"inclusive synchronous compiler spans; cache hits do not execute spans; absent phases performed no work",
        "unavailable_submetrics":["JIT is not enabled", "native conversion and property-state construction are included in preparation/native execution, without separate clocks"],
        "scope":"case source admission, preparation/rebuild, joined native execution, validation/results, optional publication and teardown; application compilation excluded",
        "sampling":"10 flat Criterion samples, 250 ms warmup, 1 s target measurement time (extended for slow operations)",
        "memory_scope":"pool observation per operation; process lifetime VmHWM from the dedicated benchmark process"
    })).unwrap()).unwrap();
}
fn revision_prepare_simulation(
    executor: &tokio::runtime::Runtime,
    revision: &ModelRevision,
    id: pse_ids::SemanticId,
) -> pse_runtime::workflow::PreparedSimulation {
    executor
        .block_on(revision.prepare_simulation(id, simulation(), compiler(), &CancelSource::new()))
        .unwrap()
}
fn configuration() -> Criterion {
    Criterion::default().output_directory(
        &PathBuf::from(std::env::var("PSE_PLAN14_COST_OUTPUT").unwrap()).join("criterion"),
    )
}
criterion_group! {name=benches;config=configuration();targets=process}
criterion_main!(benches);
