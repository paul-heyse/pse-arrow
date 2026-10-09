// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordinary local native reconstruction after releasing every source runtime owner.
use super::*;
use pse_ids::{ContentHash, Frame, FramedHasher};
use pse_operations::canonical::{CanonicalOptions, CanonicalStore};
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{canonical_revisions, modeling_checks, solve_variables},
};
use pse_runtime::{
    math::portable::{ExpectedProducerTarget, ReplayAdmission},
    workflow,
};
use serde_json::{Value, json};
use std::sync::Arc;

fn local_anchor() {}

async fn wait_for_release<T>(runtime: &std::sync::Weak<T>, pool_released: impl Fn() -> bool) {
    // Completion publication can precede the supervisor's final Arc drop. Wait
    // for actual owner release, not one scheduler yield. The timeout is only a
    // deadlock watchdog; the weak owner and pool assertions establish teardown.
    tokio::time::timeout(Duration::from_secs(10), async {
        while runtime.strong_count() != 0 || !pool_released() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("completed native run supervisor released its runtime");
}

#[allow(
    unsafe_code,
    reason = "controlled benchmark composition independently observes its immutable native reconstruction context"
)]
fn local_admission() -> ReplayAdmission {
    // SAFETY: this actual executable supplies its own code anchor. This case has
    // no provider/plugin callbacks, interpreter, JIT, mutable configuration or
    // direct executable-map mutation in mathematical reconstruction. WORKER names
    // the supported native reconstruction role, not a managed worker artifact.
    unsafe {
        ReplayAdmission::observe_local(
            ExpectedProducerTarget::WORKER,
            local_anchor as *const () as usize,
            Arc::new(|| Ok(Vec::new())),
        )
    }
    .expect("supported exact local native reconstruction admission")
}

fn runtime_on(
    owner: &WorkflowRuntime,
    store: CanonicalStore,
    build: ContentHash,
    admission: Option<ReplayAdmission>,
) -> workflow::Runtime {
    workflow::Runtime::from_shared(
        owner.runtime.clone(),
        owner.registry.clone(),
        owner.sessions.clone(),
        workflow::CanonicalDeployment::new(
            store,
            workflow::OuterAttestation {
                source: None,
                build,
            },
            admission,
        ),
    )
}

fn body_counts(phases: &phases::Phases, replay: bool) -> Value {
    let counts = phases.constructions();
    let body = &counts["body.value"];
    let admissions = body["attempts"].as_u64().unwrap_or(0);
    let successes = body["successes"].as_u64().unwrap_or(0);
    let reuses = body["reuses"].as_u64().unwrap_or(0);
    if replay {
        assert!(
            reuses > 0,
            "reopened preparation did not actually reuse any body"
        );
        assert_eq!(
            admissions, 0,
            "reopened preparation fell back to fresh admission"
        );
    } else {
        assert!(
            admissions > 0,
            "fresh control did not actually admit any body"
        );
        assert_eq!(
            admissions, successes,
            "fresh body admission did not complete"
        );
    }
    counts
}

// These owned projections cover the actual generated row fields. Float bits are
// included so JSON's numeric representation cannot weaken historical equality.
fn variable_rows(rows: &[solve_variables::Row]) -> Value {
    json!(rows.iter().map(|row| json!({
        "run_id":row.run_id,"step":row.step,"symbol_id":row.symbol_id,
        "quantity_id":row.quantity_id,"unit_id":row.unit_id,"fixed":row.fixed,
        "parameter":row.parameter,"domain":row.domain,"value":row.value,
        "lower":row.lower,"upper":row.upper,"lower_violation":row.lower_violation,
        "upper_violation":row.upper_violation,"tolerance":row.tolerance,
        "lower_dual":row.lower_dual,"upper_dual":row.upper_dual,
        "reduced_cost":row.reduced_cost,"stationarity":row.stationarity,
        "dual_qualification":row.dual_qualification,
        "float_bits":{
            "value":row.value.map(f64::to_bits),"lower":row.lower.map(f64::to_bits),
            "upper":row.upper.map(f64::to_bits),"lower_violation":row.lower_violation.map(f64::to_bits),
            "upper_violation":row.upper_violation.map(f64::to_bits),"tolerance":row.tolerance.map(f64::to_bits),
            "lower_dual":row.lower_dual.map(f64::to_bits),"upper_dual":row.upper_dual.map(f64::to_bits),
            "reduced_cost":row.reduced_cost.map(f64::to_bits),"stationarity":row.stationarity.map(f64::to_bits)
        }
    })).collect::<Vec<_>>())
}

fn check_rows(rows: &[modeling_checks::Row]) -> Value {
    json!(rows.iter().map(|row| json!({
        "run_id":row.run_id,"step":row.step,"sample_index":row.sample_index,
        "time":row.time,"target_id":row.target_id,"source_id":row.source_id,
        "kind":row.kind,"value":row.value,"tolerance":row.tolerance,
        "satisfied":row.satisfied,"within_validity":row.within_validity,
        "extrapolation_allowed":row.extrapolation_allowed,"basis":row.basis,
        "layer":row.layer,"claim_id":row.claim_id,"claim_owner":row.claim_owner,
        "claim_owner_lineage":row.claim_owner_lineage,"coverage_id":row.coverage_id,
        "evidence_id":row.evidence_id,"form_id":row.form_id,"call_id":row.call_id,
        "selected_records":row.selected_records,"dependencies":row.dependencies,
        "input_values":row.input_values.iter().map(|input| json!({
            "name":input.name,"value":input.value,"quantity_type":input.quantity_type,
            "value_bits":input.value.to_bits()
        })).collect::<Vec<_>>(),
        "applicability_outcome":row.applicability_outcome,
        "applicability_basis":row.applicability_basis,"permission_ids":row.permission_ids,
        "unknown_allowed":row.unknown_allowed,"observation_instance":row.observation_instance,
        "applicability_required":row.applicability_required,"applicability_reason":row.applicability_reason,
        "applicability_permissions":row.applicability_permissions.iter().map(|permission| json!({
            "permission_id":permission.permission_id,"scope":permission.scope,
            "target_kind":permission.target_kind,"targets":permission.targets,
            "allow_unknown":permission.allow_unknown,"allow_extrapolation":permission.allow_extrapolation
        })).collect::<Vec<_>>(),
        "float_bits":{"time":row.time.map(f64::to_bits),"value":row.value.to_bits(),
            "tolerance":row.tolerance.map(f64::to_bits)}
    })).collect::<Vec<_>>())
}

fn revision_row(row: &canonical_revisions::Row) -> Value {
    json!({"key":row.key,"problem":row.problem,"sequence":row.sequence,"parent":row.parent,
        "operation":row.operation,"request":row.request,"interpretation":row.interpretation})
}

fn original_rows(result: &workflow::RunResult) -> Value {
    let report = authored_success(result);
    let mut variables =
        solve_variables::Row::rows(&result.table("runtime.solve_variables").unwrap()).unwrap();
    variables.sort_by_key(|row| row.symbol_id);
    let solved: Vec<_> = variables
        .iter()
        .filter(|row| !row.fixed && !row.parameter)
        .collect();
    assert_eq!(solved.len(), 2);
    for (name, expected) in [("x", 2.), ("y", 3.)] {
        let mut symbols = report
            .prepared
            .model
            .model
            .compiled()
            .model
            .symbols
            .values()
            .filter(|symbol| {
                symbol.lineage.path == name || symbol.lineage.path.ends_with(&format!(".{name}"))
            });
        let symbol = symbols.next().expect("original authored coordinate");
        assert!(symbols.next().is_none(), "original coordinate is ambiguous");
        let row = solved
            .iter()
            .find(|row| row.symbol_id == symbol.id)
            .expect("original coordinate result row");
        let allowance = row
            .tolerance
            .expect("original production physical allowance");
        assert!(allowance.is_finite() && allowance > 0.);
        assert!(
            (row.value.unwrap() - expected).abs() <= allowance,
            "original independently specified {name} violates its physical allowance"
        );
    }
    let checks =
        modeling_checks::Row::rows(&result.table("runtime.modeling_checks").unwrap()).unwrap();
    assert!(!checks.is_empty());
    assert!(checks.iter().all(|row| row.satisfied));
    // Owned rows retain no source runtime, compiler, report or Arrow buffer.
    json!({"variables":variable_rows(&variables),"checks":check_rows(&checks),"accepted":report.accepted,
        "value_bits":variables.iter().map(|row| (row.symbol_id, row.value.map(f64::to_bits), row.tolerance.map(f64::to_bits))).collect::<Vec<_>>()})
}

async fn historical_rows<R: RelationRow>(
    runtime: &workflow::Runtime,
    run: &str,
    attempt: &str,
    relation: &str,
) -> Vec<R> {
    let mut reader = runtime
        .results(
            run,
            attempt,
            relation,
            0,
            u64::MAX,
            CancelSource::new().token(),
        )
        .await
        .unwrap();
    let mut rows = Vec::new();
    while let Some(batch) = reader.next_relation_batch().await.unwrap() {
        rows.extend(R::rows(&batch).unwrap());
    }
    rows
}

async fn native_warmup(phases: &phases::Phases, build: ContentHash) -> Value {
    let started = Instant::now();
    let state = std::env::var_os("PSE_SURREAL_STATE").unwrap();
    let mut options = CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
    options.database = format!(
        "canonical_test_restart_warmup_{}",
        pse_operations::mint_id::<pse_ids::SemanticId>().to_hex()
    );
    let store = CanonicalStore::connect(&options).await.unwrap();
    store.create().await.unwrap();
    let sources = k4_support::sources(k4_support::SOURCE);
    let owner = WorkflowRuntime::with_threads(NonZeroUsize::new(1).unwrap()).unwrap();
    let warmup = Arc::downgrade(&owner.runtime);
    let pool = owner.runtime.pool();
    // Exercise the actual original native route before observing replay eligibility.
    // This database and runtime cannot seed eligible products in the measured one.
    let runtime = runtime_on(&owner, store.clone(), build, None);
    let physical = k4_support::physical(&runtime, &owner, &sources).await;
    let package = runtime
        .modeling_from_documents(
            &k4_support::admitted_documents(&owner, &sources.modeling),
            physical,
            &owner.cancellation,
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
    phases.reset();
    let prepared = seed_prepare(
        &package,
        root.into(),
        profile(Backend::Ipopt, false),
        &owner.cancellation,
    )
    .await
    .unwrap();
    let constructions = body_counts(phases, false);
    let result = prepared.start().unwrap().wait().await.unwrap();
    let rows = original_rows(&result);
    drop(result);
    drop(prepared);
    drop(package);
    drop(runtime);
    drop(owner);
    wait_for_release(&warmup, || pool.reserved() == 0).await;
    assert!(
        warmup.upgrade().is_none(),
        "native warmup runtime survived release"
    );
    assert_eq!(
        pool.reserved(),
        0,
        "native warmup retained pool allocations"
    );
    store.remove_isolated_fixture().await.unwrap();
    json!({"database":options.database,"seconds":started.elapsed().as_secs_f64(),
        "replay_admission":false,"constructions":constructions,"original_rows":rows,
        "after_runtime_teardown_bytes":pool.reserved(),"runtime_released":warmup.upgrade().is_none()})
}

async fn iteration(phases: &phases::Phases, build: ContentHash) -> (Duration, Value) {
    // Initial native execution may lazily change the executable/loader context.
    // Finish it before capturing the quiet-context seed identity; do not retry a
    // mismatch between that seed and the independently observed receiving context.
    let warmup = native_warmup(phases, build).await;
    let setup = Instant::now();
    let state = std::env::var_os("PSE_SURREAL_STATE").unwrap();
    let mut options = CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
    options.database = format!(
        "canonical_test_restart_{}",
        pse_operations::mint_id::<pse_ids::SemanticId>().to_hex()
    );
    let store = CanonicalStore::connect(&options).await.unwrap();
    store.create().await.unwrap();
    let sources = k4_support::sources(k4_support::SOURCE);
    let owner = WorkflowRuntime::with_threads(NonZeroUsize::new(1).unwrap()).unwrap();
    let origin = Arc::downgrade(&owner.runtime);
    let origin_pool = owner.runtime.pool();
    let admission = local_admission();
    let local_identity = admission.identity();
    let runtime = runtime_on(&owner, store, build, Some(admission));
    let physical = k4_support::physical(&runtime, &owner, &sources).await;
    let package = runtime
        .modeling_from_documents(
            &k4_support::admitted_documents(&owner, &sources.modeling),
            physical,
            &owner.cancellation,
        )
        .await
        .unwrap();
    let revision = package.canonical_revision().clone();
    let root = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    phases.reset();
    let prepared = seed_prepare(
        &package,
        root.into(),
        profile(Backend::Ipopt, false),
        &owner.cancellation,
    )
    .await
    .unwrap();
    let seed_constructions = body_counts(phases, false);
    let result = prepared.start().unwrap().wait().await.unwrap();
    let original = original_rows(&result);
    let run = result.canonical_run_key().unwrap().to_owned();
    let attempt = result.canonical_attempt_key().unwrap().to_owned();
    let seed_peak = owner.runtime.observation_peak_bytes();
    let setup_seconds = setup.elapsed().as_secs_f64();

    // The measured operation begins with ordinary release, not interruption.
    let started = Instant::now();
    drop(result);
    drop(prepared);
    drop(package);
    drop(runtime);
    drop(owner);
    wait_for_release(&origin, || origin_pool.reserved() == 0).await;
    assert!(
        origin.upgrade().is_none(),
        "source runtime survived restart boundary"
    );
    assert_eq!(
        origin_pool.reserved(),
        0,
        "source runtime retained pool allocations"
    );
    let release_seconds = started.elapsed().as_secs_f64();
    let phase = Instant::now();
    let store = CanonicalStore::connect(&options).await.unwrap();
    store.open().await.unwrap();
    let owner = WorkflowRuntime::with_threads(NonZeroUsize::new(1).unwrap()).unwrap();
    let receiving = Arc::downgrade(&owner.runtime);
    let pool = owner.runtime.pool();
    let admission = local_admission();
    assert_eq!(
        admission.identity(),
        local_identity,
        "receiving executable/loader/config context changed; no retry or forced hit"
    );
    let runtime = runtime_on(&owner, store, build, Some(admission));
    let open_seconds = phase.elapsed().as_secs_f64();
    let budget = owner.runtime.budget();
    let runtime_profile = json!({"pool_limit_bytes":budget.memory_limit_bytes.get(),"threads":budget.threads.pool_threads.get(),"target_partitions":budget.threads.target_partitions.get(),"math_policy":format!("{:?}",budget.math),"cache":format!("{:?}",budget.cache)});
    let phase = Instant::now();
    let physical = k4_support::physical(&runtime, &owner, &sources).await;
    let package = runtime
        .modeling_revision(revision.clone(), physical, Default::default())
        .await
        .unwrap();
    assert_eq!(package.canonical_revision(), &revision);
    let reopen_seconds = phase.elapsed().as_secs_f64();
    phases.reset();
    let phase = Instant::now();
    let prepared = seed_prepare(
        &package,
        root.into(),
        profile(Backend::Ipopt, false),
        &owner.cancellation,
    )
    .await
    .unwrap();
    let replay_constructions = body_counts(phases, true);
    let replay_phases = phases.report(1);
    let preparation_seconds = phase.elapsed().as_secs_f64();
    let phase = Instant::now();
    let result = prepared.start().unwrap().wait().await.unwrap();
    let reopened = original_rows(&result);
    assert_ne!(result.canonical_run_key().unwrap(), run);
    let solve_seconds = phase.elapsed().as_secs_f64();
    let phase = Instant::now();
    let mut variables = historical_rows::<solve_variables::Row>(
        &runtime,
        &run,
        &attempt,
        "runtime.solve_variables",
    )
    .await;
    variables.sort_by_key(|row| row.symbol_id);
    assert_eq!(variable_rows(&variables), original["variables"]);
    assert_eq!(
        json!(
            variables
                .iter()
                .map(|row| (
                    row.symbol_id,
                    row.value.map(f64::to_bits),
                    row.tolerance.map(f64::to_bits)
                ))
                .collect::<Vec<_>>()
        ),
        original["value_bits"]
    );
    let checks = historical_rows::<modeling_checks::Row>(
        &runtime,
        &run,
        &attempt,
        "runtime.modeling_checks",
    )
    .await;
    assert_eq!(check_rows(&checks), original["checks"]);
    let history_seconds = phase.elapsed().as_secs_f64();
    let peak = seed_peak.max(owner.runtime.observation_peak_bytes());
    let retained = pool.reserved();
    let store = runtime.canonical_store().clone();
    let phase = Instant::now();
    drop(result);
    drop(prepared);
    drop(package);
    drop(runtime);
    drop(owner);
    wait_for_release(&receiving, || pool.reserved() == 0).await;
    assert!(
        receiving.upgrade().is_none(),
        "reopened runtime escaped teardown"
    );
    assert_eq!(
        pool.reserved(),
        0,
        "reopened runtime retained pool allocations"
    );
    let teardown_seconds = phase.elapsed().as_secs_f64();
    let elapsed = started.elapsed();

    // Same canonical source, distinct runtime, intentionally absent eligibility:
    // actual fresh admission is the control, never a fabricated replay label.
    let fresh_started = Instant::now();
    let owner = WorkflowRuntime::with_threads(NonZeroUsize::new(1).unwrap()).unwrap();
    let fresh = Arc::downgrade(&owner.runtime);
    let fresh_pool = owner.runtime.pool();
    let runtime = runtime_on(&owner, store.clone(), build, None);
    let physical = k4_support::physical(&runtime, &owner, &sources).await;
    let package = runtime
        .modeling_revision(revision.clone(), physical, Default::default())
        .await
        .unwrap();
    assert_eq!(package.canonical_revision(), &revision);
    phases.reset();
    let prepared = seed_prepare(
        &package,
        root.into(),
        profile(Backend::Ipopt, false),
        &owner.cancellation,
    )
    .await
    .unwrap();
    let fresh_constructions = body_counts(phases, false);
    let result = prepared.start().unwrap().wait().await.unwrap();
    let fresh_rows = original_rows(&result);
    let fresh_run = result.canonical_run_key().unwrap().to_owned();
    assert_ne!(fresh_run, run);
    let process_peak_rss = owner
        .runtime
        .report()
        .unwrap()
        .process_peak_rss_bytes
        .unwrap();
    drop(result);
    drop(prepared);
    drop(package);
    drop(runtime);
    drop(owner);
    wait_for_release(&fresh, || fresh_pool.reserved() == 0).await;
    assert!(fresh.upgrade().is_none());
    assert_eq!(fresh_pool.reserved(), 0);
    let fresh_control_seconds = fresh_started.elapsed().as_secs_f64();
    let cleanup_started = Instant::now();
    store.remove_isolated_fixture().await.unwrap();
    (
        elapsed,
        json!({"database":options.database,"source_revision":revision_row(&revision),"root":root,
        "seed_run":run,"seed_attempt":attempt,"fresh_run":fresh_run,
        "local_identity":local_identity,"runtime_profile":runtime_profile,"untimed_native_warmup":warmup,
        "seed_constructions":seed_constructions,"replay_constructions":replay_constructions,"fresh_constructions":fresh_constructions,
        "replay_compiler_phases":replay_phases,"original_rows":original,"reopened_rows":reopened,"fresh_rows":fresh_rows,
        "phase_seconds":{"untimed_native_warmup":warmup["seconds"],"untimed_seed":setup_seconds,"origin_release":release_seconds,"connect_runtime_local_observation":open_seconds,"physical_admission_exact_source_reopen":reopen_seconds,"selected_preparation":preparation_seconds,"native_solve_assessment_append":solve_seconds,"original_exact_history_read":history_seconds,"receiving_release":teardown_seconds,"untimed_fresh_control":fresh_control_seconds,"untimed_database_removal":cleanup_started.elapsed().as_secs_f64()},
        "pool_peak_bytes":peak,"process_peak_rss_bytes":process_peak_rss,"retained_runtime_bytes":retained,"after_runtime_teardown_bytes":pool.reserved()}),
    )
}

pub(super) fn measure(
    c: &mut Criterion,
    spec: &Value,
    output: &std::path::Path,
    phases: &phases::Phases,
) {
    assert_eq!(spec["threads"].as_u64(), Some(1));
    assert_eq!(spec["blocks"].as_u64(), Some(1));
    let name = spec["id"].as_str().unwrap();
    let actual = std::env::current_exe().unwrap();
    let mut hash = FramedHasher::new(Frame::BuildInputsV1);
    hash.str("pse.benchmark.receiving-executable.v1")
        .part(&std::fs::read(&actual).unwrap());
    let build = hash.finish_hash();
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
    group.bench_function(name, |b| {
        b.iter_custom(|iterations| {
            let mut timed = Duration::ZERO;
            for _ in 0..iterations {
                let (elapsed, record) = executor.block_on(iteration(phases, build));
                timed += elapsed;
                records.push(record);
            }
            timed
        })
    });
    group.finish();
    let maximum = |field: &str| {
        records
            .iter()
            .map(|record| record[field].as_u64().unwrap())
            .max()
            .unwrap()
    };
    std::fs::write(output.join(format!("{name}-memory.json")), serde_json::to_vec_pretty(&json!({
        "id":name,"workload":spec,"iterations":records.len(),"threads":1,"native_threads":1,
        "pool_peak_bytes":maximum("pool_peak_bytes"),"process_peak_rss_bytes":maximum("process_peak_rss_bytes"),"retained_runtime_bytes":maximum("retained_runtime_bytes"),"after_case_teardown_bytes":maximum("after_runtime_teardown_bytes"),"after_retained_runtime_teardown_bytes":maximum("after_runtime_teardown_bytes"),
        "runtime_profile":records.first().map(|r| &r["runtime_profile"]),"receiving_executable":actual,"outer_build":build,"outer_source":null,
        "local_role":"controlled native benchmark immutable reconstruction; WORKER admission role is not a managed-primary artifact", "effective_configuration":"empty; no providers, plugins, interpreter callbacks or mutable reconstruction configuration", "strict_producer_receipt":false,
        "cache_state":"untimed original native warmup in a separate isolated database and runtime without replay admission, fully released before seed local observation; isolated seeded canonical database per iteration; every original runtime/compiler owner dropped before exact source reopen; fresh control has a separate runtime and no replay admission",
        "timed_scope":"ordinary origin release, authenticated reconnect/open, fresh runtime/local observation, physical admission and exact source reopen, replay preparation, one original native solve/assessment/append, exact historical variable/check reads, receiving runtime release",
        "untimed_scope":"Cargo/native installation, original Root native warmup in a separate isolated database/runtime without replay eligibility, initial schema/source/solve/product seeding, separate fresh-admission control and isolated database removal",
        "phase_scope":"complete calling-process operations; inclusive production spans are separate observations", "memory_scope":"calling-process pools for seed/replay; native warmup and fresh control excluded from pool peak; process VmHWM includes native warmup and all controls; external canonical server unobserved",
        "limitations":"same executable process and quiet warmed loader context after an untimed original Root native solve; no first native initialization, managed worker/process/Python startup, crash/interruption, server restart, strict producer qualification, deployment portability or replay-only numerical execution claim; weak-runtime release does not establish database-client/task quiescence and asynchronous protection release can continue into untimed cleanup; fresh-control timing has a different operation boundary and is not a speedup comparison",
        "acceptance":"actual semantic body reuse with zero fresh admissions after weak-owner and zero-pool release proof; cold/fresh successful admissions; exact source/run/attempt historical rows and floating-point bits; original root x=2,y=3 within production physical allowances and authored checks",
        "sampling":"10 flat Criterion samples; setup/control/removal excluded via iter_custom; smoke executes the same acceptance without timing receipt","records":records
    })).unwrap()).unwrap();
}
