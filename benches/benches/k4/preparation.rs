// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Small production-route controls complement the independently qualified thermodynamic M1 cases.
use super::*;
#[path = "demand.rs"]
mod demand;
#[path = "support.rs"]
mod support;
#[path = "worker_lifetime.rs"]
mod worker_lifetime;
use pse_engine::cache_service::CacheComponent;
use pse_runtime::workflow::ModelingDiagnosticPreparation;
use std::{collections::BTreeSet, num::NonZeroUsize};

async fn stage(
    owner: &WorkflowRuntime,
    package: &ModelingPackage,
    name: &str,
    phases: &phases::Phases,
) -> (Duration, Value, ModelingDiagnosticPreparation) {
    let root = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let before = owner.runtime.math().preparations();
    owner.runtime.reset_observation_peak();
    phases.reset();
    let started = Instant::now();
    let mut analysis = package
        .declared_execution(
            root,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            &CancelSource::new(),
        )
        .await
        .unwrap()
        .analysis;
    let analysis_seconds = started.elapsed().as_secs_f64();
    if name == "value" {
        analysis.case.values.insert("a".into(), 2.5);
    }
    if name == "binding-a" || name == "binding-a-return" {
        analysis.bindings.demand = vec!["x".into()];
    }
    let phase = Instant::now();
    let prepared = package
        .prepare_diagnostics(&analysis, &CancelSource::new())
        .await
        .unwrap();
    let resolution_seconds = phase.elapsed().as_secs_f64();
    let phase = Instant::now();
    owner
        .runtime
        .math()
        .assemble(prepared.model.case.clone())
        .await
        .unwrap();
    let derivative_seconds = phase.elapsed().as_secs_f64();
    let elapsed = started.elapsed();
    let product = prepared.model.model.compiled();
    let case = prepared.model.case.compiled();
    let body_ids: BTreeSet<_> = product
        .admitted
        .bodies
        .values()
        .map(|b| b.math().allocation_identity())
        .collect();
    let observations = phases.report(1);
    let body_admissions = observations["pse.case.semantic_body_admission"]["calls"]
        .as_u64()
        .unwrap_or(0);
    if matches!(name, "warm" | "span-only") {
        assert_eq!(
            body_admissions, 0,
            "retained unchanged arithmetic must not be readmitted"
        );
    }
    let record = json!({"body_admission_attempts":body_admissions,"stage":name,"outcome":"prepared","seconds":{"analysis":analysis_seconds,"case_resolution":resolution_seconds,"derivative_programs":derivative_seconds,"total":elapsed.as_secs_f64()},"preparations":support::counts(before,owner.runtime.math().preparations()),"compiler_phases":observations,"bodies":product.admitted.bodies.len(),"allocations":{"model":product.model.allocation_identity(),"admitted":product.admitted.allocation_identity(),"math_bodies":body_ids,"plan":case.plan.allocation_identity(),"bindings":case.coefficient_values.allocation_identity(),"attribution":case.occurrences.allocation_identity()},"pool_reserved_bytes":owner.runtime.pool().reserved(),"pool_peak_bytes":owner.runtime.observation_peak_bytes(),"process_peak_rss_bytes":owner.runtime.report().unwrap().process_peak_rss_bytes});
    let mut record = record;
    record["constructions"] = phases.constructions();
    (elapsed, record, prepared)
}
async fn preparation(owner: &WorkflowRuntime, phases: &phases::Phases) -> (Duration, Value) {
    let store = pse_operations::testing::canonical_fixture_store().unwrap();
    owner.register_fixture(store.clone()).unwrap();
    let runtime = Runtime::from_shared(
        owner.runtime.clone(),
        owner.registry.clone(),
        owner.sessions.clone(),
        pse_runtime::workflow::CanonicalDeployment::new(
            store,
            pse_runtime::workflow::OuterAttestation {
                source: Some(pse_ids::ContentHash::from_bytes([0; 32])),
                build: pse_ids::ContentHash::from_bytes([1; 32]),
            },
            None,
        ),
    );
    let sources = support::sources(support::SOURCE);
    let setup = Instant::now();
    let original = support::package(&runtime, owner, &sources).await;
    let setup_seconds = setup.elapsed().as_secs_f64();
    let mut timed = Duration::ZERO;
    let mut records = Vec::new();
    let mut retained = Vec::new();
    for name in [
        "cold",
        "warm",
        "value",
        "binding-a",
        "binding-b",
        "binding-a-return",
    ] {
        let (elapsed, record, prepared) = stage(owner, &original, name, phases).await;
        timed += elapsed;
        records.push(record);
        retained.push(prepared);
    }
    let mut span_sources = sources.clone();
    span_sources.modeling[0]
        .get_mut("models/root.pse")
        .unwrap()
        .splice(0..0, b"\n\n".iter().copied());
    let setup = Instant::now();
    let shifted = support::package(&runtime, owner, &span_sources).await;
    let source_admission = setup.elapsed();
    let (elapsed, mut record, prepared) = stage(owner, &shifted, "span-only", phases).await;
    record["source_admission_seconds"] = source_admission.as_secs_f64().into();
    // Timers directly cover their own synchronous setup, never estimate internal phases.
    let original_product = retained[0].model.model.compiled().clone();
    let shifted_product = prepared.model.model.compiled();
    assert_ne!(
        original_product.occurrences(),
        shifted_product.occurrences()
    );
    for (key, body) in &original_product.admitted.bodies {
        assert!(std::sync::Arc::ptr_eq(
            body.math(),
            shifted_product.admitted.bodies[key].math()
        ));
    }
    timed += elapsed;
    records.push(record);
    retained.push(prepared);
    let mut edited_sources = sources.clone();
    edited_sources.modeling[0].insert(
        "models/root.pse".into(),
        support::SOURCE.replace("x==a", "x==a+1").into_bytes(),
    );
    let setup = Instant::now();
    let edited = support::package(&runtime, owner, &edited_sources).await;
    let source_admission = setup.elapsed();
    let (elapsed, mut record, prepared) = stage(owner, &edited, "one-equation", phases).await;
    record["source_admission_seconds"] = source_admission.as_secs_f64().into();
    assert!(original_product.admitted.bodies.keys().any(|key| {
        !prepared
            .model
            .model
            .compiled()
            .admitted
            .bodies
            .contains_key(key)
    }));
    assert!(original_product.admitted.bodies.keys().any(|key| {
        prepared
            .model
            .model
            .compiled()
            .admitted
            .bodies
            .contains_key(key)
    }));
    timed += elapsed;
    records.push(record);
    retained.push(prepared);
    let mut context_sources = sources.clone();
    let mut physical: Value = serde_saphyr::from_str(
        std::str::from_utf8(&context_sources.physical["materials/physical.yaml"]).unwrap(),
    )
    .unwrap();
    // A referenced Scalar quantity's nominal belongs to the complete physical identity.
    physical["quantity_types"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|q| q["name"] == "Scalar")
        .unwrap()["nominal_magnitude"] = 2.0.into();
    context_sources.physical.insert(
        "materials/physical.yaml".into(),
        serde_json::to_vec(&physical).unwrap(),
    );
    let setup = Instant::now();
    let changed = support::package(&runtime, owner, &context_sources).await;
    let source_admission = setup.elapsed();
    assert_ne!(
        pse_runtime::workflow::OperationSource::of(&original).physical_context,
        pse_runtime::workflow::OperationSource::of(&changed).physical_context
    );
    let (elapsed, mut record, prepared) =
        stage(owner, &changed, "referenced-context", phases).await;
    record["source_admission_seconds"] = source_admission.as_secs_f64().into();
    assert!(original_product.admitted.bodies.keys().all(|key| {
        !prepared
            .model
            .model
            .compiled()
            .admitted
            .bodies
            .contains_key(key)
    }));
    timed += elapsed;
    records.push(record);
    retained.push(prepared);
    let first = &records[3]["allocations"]["model"];
    assert_eq!(
        first, &records[5]["allocations"]["model"],
        "retained A/B/A returns to the admitted specialization"
    );
    (
        timed,
        json!({"source_admission_seconds":setup_seconds,"stages":records,"cache_reports":owner.runtime.math().report()}),
    )
}
async fn pressure(
    owner: &WorkflowRuntime,
    spec: &Value,
    phases: &phases::Phases,
) -> (Duration, Value) {
    let store = pse_operations::testing::canonical_fixture_store().unwrap();
    owner.register_fixture(store.clone()).unwrap();
    let runtime = Runtime::from_shared(
        owner.runtime.clone(),
        owner.registry.clone(),
        owner.sessions.clone(),
        pse_runtime::workflow::CanonicalDeployment::new(
            store,
            pse_runtime::workflow::OuterAttestation {
                source: Some(pse_ids::ContentHash::from_bytes([0; 32])),
                build: pse_ids::ContentHash::from_bytes([1; 32]),
            },
            None,
        ),
    );
    let service = owner.runtime.math();
    let pool = owner.runtime.pool();
    let baseline = pool.reserved();
    let sources = support::sources(support::SOURCE);
    let started = Instant::now();
    let package = support::package(&runtime, owner, &sources).await;
    let root = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    phases.reset();
    let first = package
        .prepare(
            root,
            pse_modeling::specialize::root_instance(root),
            Default::default(),
            Default::default(),
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let escaped_math: Vec<_> = first
        .compiled()
        .admitted
        .bodies
        .values()
        .map(|b| b.math().clone())
        .collect();
    let escaped_model = first.compiled().model.clone();
    let escaped_admitted = first.compiled().admitted.clone();
    let first_ids: BTreeSet<_> = escaped_math
        .iter()
        .map(|b| b.allocation_identity())
        .collect();
    let mut unique_ids = first_ids.clone();
    let mut escaped_ids = BTreeSet::from([
        escaped_model.allocation_identity(),
        escaped_admitted.allocation_identity(),
    ]);
    escaped_ids.extend(first_ids.iter().copied());
    let before_pressure = pool.reserved();
    let initial_cache = service.report();
    let mut cold_admissions = Vec::new();
    for index in 0..spec["variants"].as_u64().unwrap() {
        let texts =
            support::sources(&support::SOURCE.replace("x==a", &format!("x==a+{}", index + 1)));
        let other = support::package(&runtime, owner, &texts).await;
        let model = other
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                &CancelSource::new(),
            )
            .await
            .unwrap();
        let ids: BTreeSet<_> = model
            .compiled()
            .admitted
            .bodies
            .values()
            .map(|b| b.math().allocation_identity())
            .collect();
        unique_ids.extend(ids.iter().copied());
        cold_admissions.push(json!({"variant":index,"math_allocation_ids":ids,"pool_reserved_bytes":pool.reserved()}));
    }
    let pressured_cache = service.report();
    let pressure_pool = pool.reserved();
    // A fresh workspace makes lawful post-pressure recomputation observable; the original
    // compiler's Salsa memo is not mistaken for service-resident retention.
    phases.reset();
    let fresh = support::package(&runtime, owner, &sources).await;
    let fresh_model = fresh
        .prepare(
            root,
            pse_modeling::specialize::root_instance(root),
            Default::default(),
            Default::default(),
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let recomputed_ids: BTreeSet<_> = fresh_model
        .compiled()
        .admitted
        .bodies
        .values()
        .map(|b| b.math().allocation_identity())
        .collect();
    let recomputation_phases = phases.report(1);
    let recomputation_constructions = phases.constructions();
    assert_ne!(
        first_ids, recomputed_ids,
        "fresh workspace recomputes arithmetic after pressure or denied retention"
    );
    assert!(
        recomputation_phases["pse.case.semantic_body_admission"]["calls"]
            .as_u64()
            .unwrap_or(0)
            > 0
    );
    let escaped_before_clear = pool.reserved();
    service.clear_program_cache();
    let after_retention_clear = pool.reserved();
    drop(first);
    drop(package);
    drop(fresh_model);
    drop(fresh);
    let escaped_only = pool.reserved();
    assert!(!escaped_math.is_empty());
    assert!(
        escaped_only > baseline,
        "escaped aliases retain allocation charges"
    );
    for body in &escaped_math {
        assert!(body.output_count() > 0);
    }
    drop(escaped_math);
    let model_only = pool.reserved();
    drop(escaped_model);
    let admitted_alias_only = pool.reserved();
    drop(escaped_admitted);
    let after_final_alias = pool.reserved();
    assert_eq!(
        after_final_alias, baseline,
        "final escaped owner releases its accounted allocation graph"
    );
    let elapsed = started.elapsed();
    (
        elapsed,
        json!({"stages":[{"stage":"pressure-and-release","outcome":"prepared","seconds":{"total":elapsed.as_secs_f64()}}],"baseline_pool_bytes":baseline,"before_pressure_pool_bytes":before_pressure,"after_pressure_pool_bytes":pressure_pool,"before_retention_clear_bytes":escaped_before_clear,"after_retention_clear_bytes":after_retention_clear,"escaped_only_bytes":escaped_only,"model_alias_only_bytes":model_only,"admitted_alias_only_bytes":admitted_alias_only,"after_final_alias_bytes":after_final_alias,"initial_cache":initial_cache,"pressured_cache":pressured_cache,"after_clear_cache":service.report(),"escaped_allocation_ids":escaped_ids,"unique_live_escaped_allocation_count":escaped_ids.len(),"live_allocation_scope":"escaped math payloads and SharedAllocation model/admitted descriptors; pool separately accounts for their attached ownership graph and wrappers","first_math_ids":first_ids,"recomputed_math_ids":recomputed_ids,"recomputation_phases":recomputation_phases,"recomputation_constructions":recomputation_constructions,"unique_observed_math_addresses":unique_ids,"address_scope":"local payload addresses; released allocations can reuse addresses, so historical union is not a simultaneous-live count","variants":cold_admissions}),
    )
}

pub(super) fn measure(
    c: &mut Criterion,
    spec: &Value,
    output: Option<&Path>,
    phases: &phases::Phases,
) {
    let name = spec["id"].as_str().unwrap();
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let mut records = Vec::new();
    let mut group = c.benchmark_group("modeling_preparation");
    group
        .sample_size(10)
        .sampling_mode(criterion::SamplingMode::Flat)
        .warm_up_time(Duration::from_millis(1))
        .measurement_time(Duration::from_secs(1));
    group.bench_function(name, |b| {
        b.iter_custom(|iterations| {
            let mut timed = Duration::ZERO;
            for _ in 0..iterations {
                let mut policy = pse_runtime::math::MathPolicy::default();
                if let Some(bytes) = spec["retention_bytes"].as_u64() {
                    policy.artifact_bytes = bytes as usize;
                }
                if spec["worker_permit_lifetime"] == true {
                    policy.jobs = 1;
                }
                let math_policy = format!("{policy:?}");
                let owner =
                    WorkflowRuntime::with_math(NonZeroUsize::new(1).unwrap(), policy).unwrap();
                let pool = owner.runtime.pool();
                let (elapsed, mut record) = if spec["demand_orders"] == true {
                    executor.block_on(demand::run(&owner, phases))
                } else if spec["worker_permit_lifetime"] == true {
                    executor.block_on(worker_lifetime::run(&owner, phases))
                } else if spec["pressure"] == true {
                    executor.block_on(pressure(&owner, spec, phases))
                } else {
                    executor.block_on(preparation(&owner, phases))
                };
                record["pool_peak_bytes"] = owner.runtime.observation_peak_bytes().into();
                record["math_policy"] = math_policy.into();
                record["process_peak_rss_bytes"] = owner
                    .runtime
                    .report()
                    .unwrap()
                    .process_peak_rss_bytes
                    .into();
                // The operation has returned and its runtime/products have dropped.
                // Drain isolated fixtures outside the accumulated measured interval.
                executor.block_on(owner.cleanup_fixtures()).unwrap();
                drop(owner);
                executor.block_on(async {
                    tokio::task::yield_now().await;
                });
                record["after_runtime_teardown_bytes"] = pool.reserved().into();
                assert_eq!(
                    pool.reserved(),
                    0,
                    "runtime teardown releases every admitted product reservation"
                );
                timed += elapsed;
                records.push(record);
            }
            timed
        })
    });
    group.finish();
    let record = json!({"experiment":"modeling_preparation","id":name,"workload":spec,"threads":1,"native_threads":1,"pool_limit_bytes":64_u64<<30,"force_validation":"collector enables pse-relations/force-validate","iterations":records.len(),"scope":"actual source admission, specialization, solver-view resolution and derivative assembly; pressure uses bare semantic preparation without native solves; every iteration retains its own runtime only","sampling":"10 flat Criterion samples; ordinary preparation sums directly observed stage durations; pressure measures complete pressure/release path","records":records});
    println!("{record}");
    if let Some(output) = output {
        std::fs::create_dir_all(output).unwrap();
        std::fs::write(
            output.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&record).unwrap(),
        )
        .unwrap();
    }
}
