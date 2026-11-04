// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded complete-call measurements, separate from independent scientific oracles.
use super::*;
use pse_backend_native::coolprop::{WaterFactory, WaterRegime};
use pse_kernels::{
    DerivativeOrder, EvaluationContext, ExecutionScope, ProviderBuffer, ProviderFactory,
    ProviderRequest, ProviderRuntime,
};
use std::{
    io::Write as _,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(serde::Serialize)]
struct Sample {
    placement: &'static str,
    regime: &'static str,
    order: &'static str,
    condition: &'static str,
    canonical_input_bits: [u64; 2],
    science_specification: pse_ids::ContentHash,
    factory_configuration: pse_ids::ContentHash,
    warm_calls: usize,
    factory_prepare_ns: u128,
    binding_prepare_ns: u128,
    cold_complete_call_ns: u128,
    warm_complete_calls_total_ns: u128,
    drain_ns: u128,
    total_including_prepare_and_drain_ns: u128,
    accounted_before: usize,
    accounted_live: usize,
    accounted_retained_after_drain: usize,
    accounted_phase_high_water: usize,
    configured_opaque_allowance_bytes: Option<usize>,
    parent_cumulative_vm_hwm_bytes: Option<u64>,
    child_snapshots_after_native_close_before_transport_shutdown: Vec<process::ChildMeasurement>,
}
fn cells(completed: &pse_kernels::ProviderCompletion<'_>) -> Vec<u64> {
    completed
        .values
        .iter()
        .chain(completed.jacobian)
        .chain(completed.hessians)
        .map(|value| value.to_bits())
        .collect()
}

#[test]
fn complete_water_calls_measure_fresh_and_reused_bindings_with_actual_reap() {
    let temporary = tempfile::tempdir().unwrap();
    let count = |n| NonZeroUsize::new(n).unwrap();
    let runtime = crate::SharedRuntime::build(crate::ResourceBudget {
        memory_limit_bytes: count(1 << 30),
        spill_dir: temporary.path().to_path_buf(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: count(16),
        threads: pse_engine::ThreadBudget {
            pool_threads: count(16),
            target_partitions: count(16),
        },
        execution: Default::default(),
        cache: crate::CacheBudget::disabled(1),
        math: crate::math::MathPolicy {
            workspace_bytes: 16 << 20,
            ..Default::default()
        },
    })
    .unwrap();
    let executable = std::env::var_os("PSE_SCIENTIFIC_WORKER_BINARY")
        .expect("explicit source-built scientific worker measurement fixture");
    let registry = pse_quantity::standard::standard_registry().unwrap();
    let mut samples = Vec::new();
    for (regime, label, point) in [
        (
            WaterRegime::Liquid,
            "liquid",
            [300.0, 996.556 / 0.018_015_268],
        ),
        (WaterRegime::Vapor, "vapor", [500.0, 0.435 / 0.018_015_268]),
        (
            WaterRegime::Supercritical,
            "supercritical",
            [650.0, 500.0 / 0.018_015_268],
        ),
    ] {
        let mut expected = None;
        for placement in ["in-process", "local-worker"] {
            let observations = Arc::new(Mutex::new(Vec::new()));
            let before = runtime.report().unwrap().pool_reserved_now;
            runtime.reset_observation_peak();
            let total = Instant::now();
            let mut configured_opaque_allowance_bytes = None;
            let factory: Box<dyn ProviderFactory> = if placement == "in-process" {
                Box::new(WaterFactory::prepare(&registry, regime).unwrap())
            } else {
                let executable = PreparedExecutable::observe(&executable, 64 << 20)
                    .unwrap()
                    .measured(observations.clone());
                configured_opaque_allowance_bytes = Some(executable.opaque_bytes());
                Box::new(
                    LocalWaterFactory::prepare(
                        executable,
                        &registry,
                        regime,
                        runtime.math().clone(),
                    )
                    .unwrap(),
                )
            };
            let factory_prepare_ns = total.elapsed().as_nanos();
            let science_specification = factory.spec().identity();
            let factory_configuration = factory.configuration_key();
            let control = pse_diagnostics::Cancellation::default();
            let flag = control.flag();
            let scope =
                ExecutionScope::new(control, Some(Instant::now() + Duration::from_secs(60)));
            let preparation = Instant::now();
            let mut binding = factory.create_scoped(scope).unwrap();
            let request = binding
                .prepare(ProviderRequest::all(
                    binding.spec(),
                    DerivativeOrder::Second,
                ))
                .unwrap();
            let mut buffer = ProviderBuffer::new(&request);
            let binding_prepare_ns = preparation.elapsed().as_nanos();
            let context = EvaluationContext {
                cancelled: &flag,
                max_result_bytes: 4096,
            };
            let cold = Instant::now();
            let completed = buffer
                .fill(
                    &mut binding,
                    &point,
                    &request,
                    ProviderRuntime::default(),
                    &context,
                )
                .unwrap();
            let cold_complete_call_ns = cold.elapsed().as_nanos();
            let actual = cells(&completed);
            if let Some(expected) = &expected {
                assert_eq!(&actual, expected);
            } else {
                expected = Some(actual);
            }
            let mut warm_complete_calls_total_ns = 0;
            let warm_calls = 8;
            for _ in 0..warm_calls {
                let started = Instant::now();
                let completed = buffer
                    .fill(
                        &mut binding,
                        &point,
                        &request,
                        ProviderRuntime::default(),
                        &context,
                    )
                    .unwrap();
                warm_complete_calls_total_ns += started.elapsed().as_nanos();
                assert_eq!(cells(&completed), *expected.as_ref().unwrap());
            }
            let live = runtime.report().unwrap().pool_reserved_now;
            let drain = Instant::now();
            drop(buffer);
            drop(binding);
            drop(factory);
            let drain_ns = drain.elapsed().as_nanos();
            let total_ns = total.elapsed().as_nanos();
            let after = runtime.report().unwrap();
            let children = observations.lock().unwrap().clone();
            if placement == "local-worker" {
                assert_eq!(children.len(), 1);
                for child in &children {
                    assert!(
                        child.after_close_ack && child.error.is_none() && child.identity.is_some(),
                        "{child:?}"
                    );
                    assert!(
                        child
                            .peak_resident_bytes_at_close
                            .is_some_and(|bytes| bytes > 0)
                    );
                    assert_eq!(child.exit_code, Some(0));
                    assert!(child.signal.is_none());
                    assert!(!std::path::Path::new(&format!("/proc/{}", child.pid)).exists());
                }
            } else {
                assert!(children.is_empty());
            }
            assert_eq!(
                after.pool_reserved_now, before,
                "worker custody must release after actual reap"
            );
            samples.push(Sample { placement, regime: label, order: "CompleteJets/Second", warm_calls,
                condition: "fresh explicit factory/binding; first coherent fill then eight identical-state fills; includes actual close/reap; no OS cache flush",
                canonical_input_bits: point.map(f64::to_bits), science_specification, factory_configuration,
                factory_prepare_ns, binding_prepare_ns, cold_complete_call_ns, warm_complete_calls_total_ns,
                drain_ns, total_including_prepare_and_drain_ns: total_ns, accounted_before: before,
                accounted_live: live, accounted_retained_after_drain: after.pool_reserved_now,
                accounted_phase_high_water: runtime.observation_peak_bytes(), parent_cumulative_vm_hwm_bytes: after.process_peak_rss_bytes,
                configured_opaque_allowance_bytes,
                child_snapshots_after_native_close_before_transport_shutdown: children });
        }
    }
    writeln!(
        std::io::stderr().lock(),
        "PSE_SCIENTIFIC_WORKER_COMPLETE_CALL_MEASUREMENTS={}",
        serde_json::to_string(&samples).unwrap()
    )
    .unwrap();
}
