// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Matched science through the actual explicitly selected persistent executable.
#![cfg(all(unix, feature = "property-coolprop"))]
#![recursion_limit = "512"]
use pse_backend_native::coolprop::{WaterFactory, WaterRegime};
use pse_kernels::{
    DerivativeOrder, EvaluationContext, ExecutionScope, ProviderBuffer, ProviderDemand,
    ProviderError, ProviderFactory, ProviderRequest, ProviderRuntime,
};
use pse_runtime::{
    ResourceBudget, SharedRuntime,
    scientific_worker::{LocalWaterFactory, PreparedExecutable, WorkerError},
};
use std::{
    num::NonZeroUsize,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Debug, thiserror::Error)]
enum FixtureError {
    #[error("invalid nonzero scientific fixture budget: {0}")]
    Budget(#[from] std::num::TryFromIntError),
    #[error(transparent)]
    Runtime(#[from] pse_runtime::RuntimeError),
    #[error(transparent)]
    Worker(#[from] WorkerError),
    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
}

fn runtime(directory: &std::path::Path) -> Result<Arc<SharedRuntime>, FixtureError> {
    let count = NonZeroUsize::try_from;
    Ok(SharedRuntime::build(ResourceBudget {
        memory_limit_bytes: count(1usize << 30)?,
        spill_dir: directory.to_path_buf(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: count(8)?,
        threads: pse_engine::ThreadBudget {
            pool_threads: count(16)?,
            target_partitions: count(16)?,
        },
        execution: Default::default(),
        cache: pse_runtime::CacheBudget::disabled(1),
        math: pse_runtime::math::MathPolicy {
            workspace_bytes: 16 << 20,
            ..Default::default()
        },
    })?)
}
fn executable() -> Result<PreparedExecutable, WorkerError> {
    PreparedExecutable::observe(env!("CARGO_BIN_EXE_pse-scientific-worker"), 64 << 20)
}

#[tokio::test(flavor = "multi_thread")]
async fn actual_worker_matches_complete_water_jets_orders_regimes_and_repeated_calls()
-> Result<(), FixtureError> {
    tokio::task::spawn_blocking(|| -> Result<(), FixtureError> {
        let temporary = tempfile::tempdir().unwrap();
        let runtime = runtime(temporary.path())?;
        let registry = pse_quantity::standard::standard_registry().unwrap();
        for (regime, point) in [
            (WaterRegime::Liquid, [300.0, 996.556 / 0.018_015_268]),
            (WaterRegime::Vapor, [500.0, 0.435 / 0.018_015_268]),
            (WaterRegime::Supercritical, [650.0, 500.0 / 0.018_015_268]),
        ] {
            let direct = WaterFactory::prepare(&registry, regime).unwrap();
            let remote = LocalWaterFactory::prepare(
                executable()?,
                &registry,
                regime,
                runtime.math().clone(),
            )
            .unwrap();
            assert_eq!(direct.spec(), remote.spec());
            assert_ne!(direct.configuration_key(), remote.configuration_key());
            let stop = pse_diagnostics::Cancellation::default();
            let flag = stop.flag();
            let deadline = match regime {
                WaterRegime::Liquid => None,
                WaterRegime::Vapor => Some(Instant::now() + Duration::from_secs(60)),
                WaterRegime::Supercritical => Some(Instant::now() + Duration::from_secs(1800)),
            };
            let scope = ExecutionScope::new(stop.clone(), deadline);
            let mut a = direct.create_scoped(scope.clone()).unwrap();
            let mut b = remote.create_scoped(scope).unwrap();
            assert!(!b.supports_linearization());
            for order in [
                DerivativeOrder::Value,
                DerivativeOrder::First,
                DerivativeOrder::Second,
            ] {
                let mut demands = vec![ProviderDemand::CompleteJets];
                if order > DerivativeOrder::Value {
                    demands.extend(
                        [vec![0], vec![1], vec![1, 0]]
                            .map(|columns| ProviderDemand::SelectedColumns { columns }),
                    );
                }
                for demand in demands {
                    let mut raw = ProviderRequest::all(a.spec(), order);
                    if matches!(&demand, ProviderDemand::SelectedColumns { .. }) {
                        raw.outputs = vec![2, 0];
                    }
                    raw.demand = demand;
                    let request_a = a.prepare(raw.clone()).unwrap();
                    let request_b = b.prepare(raw).unwrap();
                    let mut buffer_a = ProviderBuffer::new(&request_a);
                    let mut buffer_b = ProviderBuffer::new(&request_b);
                    for repeat in 0..3 {
                        let point = [point[0] + f64::from(repeat) * 0.01, point[1]];
                        let context = EvaluationContext {
                            cancelled: &flag,
                            max_result_bytes: 4096,
                        };
                        let expected = buffer_a
                            .fill(
                                &mut a,
                                &point,
                                &request_a,
                                ProviderRuntime::default(),
                                &context,
                            )
                            .unwrap();
                        let actual = buffer_b
                            .fill(
                                &mut b,
                                &point,
                                &request_b,
                                ProviderRuntime::default(),
                                &context,
                            )
                            .unwrap();
                        for (left, right) in [
                            (expected.values, actual.values),
                            (expected.jacobian, actual.jacobian),
                            (expected.hessians, actual.hessians),
                        ] {
                            assert_eq!(
                                left.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                                right.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
                            );
                        }
                        assert!(actual.linearization.is_none());
                    }
                }
            }
        }
        Ok(())
    })
    .await??;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn actual_worker_preserves_trial_refusal_and_original_stop_without_stale_success()
-> Result<(), FixtureError> {
    tokio::task::spawn_blocking(|| -> Result<(), FixtureError> {
        let temporary = tempfile::tempdir().unwrap();
        let runtime = runtime(temporary.path())?;
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let factory = LocalWaterFactory::prepare(
            executable()?,
            &registry,
            WaterRegime::Vapor,
            runtime.math().clone(),
        )
        .unwrap();
        let stop = pse_diagnostics::Cancellation::default();
        let flag = stop.flag();
        let mut provider = factory
            .create_scoped(ExecutionScope::new(
                stop.clone(),
                Some(Instant::now() + Duration::from_secs(60)),
            ))
            .unwrap();
        for demand in [
            ProviderDemand::JacobianAction { directions: 1 },
            ProviderDemand::TransposeAction { cotangents: 1 },
            ProviderDemand::WeightedSecondAction { directions: 1 },
        ] {
            assert!(!provider.supports(&demand));
            let order = if matches!(demand, ProviderDemand::WeightedSecondAction { .. }) {
                DerivativeOrder::Second
            } else {
                DerivativeOrder::First
            };
            let mut request = ProviderRequest::all(provider.spec(), order);
            request.demand = demand;
            assert!(matches!(
                provider.prepare(request),
                Err(ProviderError::Contract(_))
            ));
        }
        for columns in [vec![], vec![2], vec![0, 0]] {
            let mut request = ProviderRequest::all(provider.spec(), DerivativeOrder::First);
            request.demand = ProviderDemand::SelectedColumns { columns };
            assert!(matches!(
                provider.prepare(request),
                Err(ProviderError::Contract(_))
            ));
        }
        let request = provider
            .prepare(ProviderRequest::all(
                provider.spec(),
                DerivativeOrder::Second,
            ))
            .unwrap();
        let mut buffer = ProviderBuffer::new(&request);
        let context = EvaluationContext {
            cancelled: &flag,
            max_result_bytes: 4096,
        };
        let good = [500.0, 0.435 / 0.018_015_268];
        assert!(
            buffer
                .fill(
                    &mut provider,
                    &good,
                    &request,
                    ProviderRuntime::default(),
                    &context
                )
                .is_ok()
        );
        assert!(matches!(
            buffer.fill(
                &mut provider,
                &[200.0, good[1]],
                &request,
                ProviderRuntime::default(),
                &context
            ),
            Err(ProviderError::OutsideEnvelope { .. })
        ));
        assert!(
            buffer
                .fill(
                    &mut provider,
                    &good,
                    &request,
                    ProviderRuntime::default(),
                    &context
                )
                .is_ok()
        );
        stop.cancel();
        assert!(matches!(
            buffer.fill(
                &mut provider,
                &good,
                &request,
                ProviderRuntime::default(),
                &context
            ),
            Err(ProviderError::Cancelled)
        ));
        drop(provider);
        Ok(())
    })
    .await??;
    Ok(())
}

#[test]
fn actual_process_custody_retains_admission_until_kill_and_reap() -> Result<(), FixtureError> {
    #[derive(Debug)]
    struct Admission(Arc<AtomicBool>);
    impl Drop for Admission {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Release);
        }
    }
    let released = Arc::new(AtomicBool::new(false));
    let admission = Arc::new(Admission(released.clone()));
    let directory = tempfile::tempdir().unwrap();
    let scope = ExecutionScope::new(
        pse_diagnostics::Cancellation::default(),
        Some(Instant::now() + Duration::from_secs(10)),
    );
    let mut child = executable()?
        .launch(
            &directory.path().join("worker.sock"),
            pse_ids::SemanticId::from_bytes([9; 16]),
            &scope,
            admission.clone(),
            false,
        )
        .unwrap();
    drop(admission);
    assert!(!released.load(Ordering::Acquire));
    let pid = child.pid();
    let end = Instant::now() + Duration::from_secs(5);
    while !directory.path().join("worker.sock").exists() {
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(2));
    }
    child.authenticate_image().unwrap();
    child.drain(Duration::ZERO).unwrap();
    assert!(released.load(Ordering::Acquire));
    assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    Ok(())
}
