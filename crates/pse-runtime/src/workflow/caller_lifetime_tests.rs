// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Workflow caller ownership through actual native drain and canonical aliases.

use super::*;
use pse_operations::canonical::CanonicalError;
use std::time::Duration;

#[tokio::test]
async fn escaped_store_operations_and_runtime_aliases_close_but_independent_runtime_remains_open() {
    let runtime = durable_tests::durable_runtime();
    let escaped = runtime.canonical_store().unwrap().clone();
    let operations = runtime.operations().unwrap().clone();
    let alias = runtime.clone();
    let independent = Runtime::from_shared(
        runtime.shared.clone(),
        runtime.model.clone(),
        runtime.canonical().unwrap().clone(),
    )
    .unwrap();
    runtime.close();
    runtime.close();
    assert!(matches!(
        escaped.revision("absent").await,
        Err(CanonicalError::CallerClosed)
    ));
    assert!(matches!(
        operations.runs("absent", None, 1).await,
        Err(WorkflowError::Canonical(CanonicalError::CallerClosed))
    ));
    assert!(matches!(
        alias.canonical_store().unwrap().admit_activity(),
        Err(CanonicalError::CallerClosed)
    ));
    assert!(
        independent
            .canonical_store()
            .unwrap()
            .revision("absent")
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn abandoned_native_waiter_keeps_activity_until_worker_thread_is_joined() {
    let runtime = durable_tests::durable_runtime();
    let activity = runtime.canonical_store().unwrap().admit_activity().unwrap();
    let (started, start) = tokio::sync::oneshot::channel();
    let (release, gate) = std::sync::mpsc::channel();
    let handle = activity
        .sync_scope(|| {
            runtime.native().submit(1, 1024, move |_, _| {
                started.send(()).unwrap();
                gate.recv().unwrap();
                Ok(((), 0))
            })
        })
        .unwrap();
    drop(activity);
    start.await.unwrap();
    runtime.close();
    drop(handle);
    assert!(matches!(
        runtime.canonical_store().unwrap().disconnect().await,
        Err(CanonicalError::ConnectionBusy { .. })
    ));
    release.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match runtime.canonical_store().unwrap().disconnect().await {
                Err(CanonicalError::ConnectionBusy { .. }) => tokio::task::yield_now().await,
                result => {
                    result.unwrap();
                    break;
                }
            }
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn durable_attempt_and_progress_writer_hold_activity_through_sender_drain() {
    let runtime = durable_tests::durable_runtime();
    let attempt = durable::DurableAttempt::new(runtime.operations().unwrap()).unwrap();
    runtime.close();
    assert!(matches!(
        runtime.canonical_store().unwrap().disconnect().await,
        Err(CanonicalError::ConnectionBusy { .. })
    ));
    drop(attempt);
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match runtime.canonical_store().unwrap().disconnect().await {
                Err(CanonicalError::ConnectionBusy { .. }) => tokio::task::yield_now().await,
                result => {
                    result.unwrap();
                    break;
                }
            }
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn prepared_native_request_is_idle_and_cannot_start_after_caller_close() {
    use crate::math::solves::SolverProfile;
    use pse_backend_native::{
        conic::{Cone, SparseMatrix},
        execution::BackendSettings,
        solve::{Controls, SolveIntent, SolverSelection},
    };
    let runtime = durable_tests::durable_runtime();
    let physical = tests::physical();
    let quantity = physical.quantities.neutral_dimensionless().unwrap();
    let unit = physical
        .quantities
        .quantity_type(quantity)
        .unwrap()
        .canonical_unit;
    let port = |symbol_id| AnalysisPort {
        symbol_id,
        quantity_id: quantity.as_id(),
        unit_id: unit.as_id(),
    };
    let request = ConicRequest {
        variables: vec![port(tests::id(1))],
        rows: vec![port(tests::id(2))],
        objective_port: port(pse_ids::SemanticId::NIL),
        quadratic: SparseMatrix::zeros(1, 1),
        objective: vec![1.0],
        constraints: SparseMatrix::new(1, 1, vec![0, 1], vec![0], vec![-1.0]),
        rhs: vec![-2.0],
        cones: vec![Cone::Nonnegative { dimension: 1 }],
        objective_constant: 3.0,
    };
    let profile = SolverProfile {
        block_overrides: Default::default(),
        block_workers: 1,
        intent: SolveIntent::Optimize,
        selection: SolverSelection::Auto,
        controls: Controls::default(),
        backend: BackendSettings::Default,
        presolve: Default::default(),
        numerics: Default::default(),
        convexity: Default::default(),
        sensitivity: None,
        composition: Default::default(),
        reconstruction: None,
    };
    let prepared = runtime
        .prepare_conic(request, &physical, profile)
        .await
        .unwrap();
    runtime.close();
    assert!(matches!(prepared.start(), Err(WorkflowError::CallerClosed)));
    runtime
        .canonical_store()
        .unwrap()
        .disconnect()
        .await
        .unwrap();
}

#[cfg(any(feature = "solver-diffsol", feature = "solver-ipopt"))]
#[derive(Default)]
struct FamilyEntryDrain {
    state: std::sync::Mutex<(bool, bool, bool)>,
    changed: tokio::sync::Notify,
    released: std::sync::Condvar,
}
#[cfg(any(feature = "solver-diffsol", feature = "solver-ipopt"))]
#[expect(
    clippy::unwrap_used,
    reason = "test synchronization poisoning and watchdog expiry must fail the native drain control"
)]
impl FamilyEntryDrain {
    fn enter(&self, flag: &std::sync::atomic::AtomicBool) {
        use std::sync::atomic::Ordering;
        let mut state = self.state.lock().unwrap();
        state.0 = true;
        self.changed.notify_waiters();
        let watchdog = std::time::Instant::now() + Duration::from_secs(10);
        while !state.2 {
            if flag.load(Ordering::Acquire) {
                state.1 = true;
                self.changed.notify_waiters();
            }
            assert!(
                std::time::Instant::now() < watchdog,
                "family worker entry was not drained"
            );
            state = self
                .released
                .wait_timeout(state, Duration::from_millis(5))
                .unwrap()
                .0;
        }
    }
    async fn wait(&self, stopped: bool) {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let changed = self.changed.notified();
                let ready = {
                    let state = self.state.lock().unwrap();
                    if stopped { state.1 } else { state.0 }
                };
                if ready {
                    break;
                }
                changed.await;
            }
        })
        .await
        .unwrap();
    }
    fn release(&self) {
        self.state.lock().unwrap().2 = true;
        self.released.notify_all();
    }
}
#[cfg(any(feature = "solver-diffsol", feature = "solver-ipopt"))]
struct FamilyEntryRelease(Arc<FamilyEntryDrain>);
#[cfg(any(feature = "solver-diffsol", feature = "solver-ipopt"))]
impl Drop for FamilyEntryRelease {
    fn drop(&mut self) {
        self.0.release();
    }
}

#[cfg(any(feature = "solver-diffsol", feature = "solver-ipopt"))]
#[expect(
    clippy::unwrap_used,
    reason = "shared test assertion requires successful worker entry and bounded native completion"
)]
async fn cancel_prepared_family_at_worker_entry(
    runtime: &Runtime,
    prepared: &PreparedStudyOperation,
) {
    let baseline = runtime.shared.math().test_admission_snapshot();
    let gate = Arc::new(FamilyEntryDrain::default());
    let _drain = FamilyEntryRelease(gate.clone());
    let observed = gate.clone();
    let cancel = crate::CancelSource::new();
    let handle = run::observed_family_worker_entry(
        Arc::new(move |flag| observed.enter(flag)),
        prepared.start(runtime, &cancel),
    )
    .await
    .unwrap();
    gate.wait(false).await;
    let entered = runtime.shared.math().test_admission_snapshot();
    assert!(
        entered.0 > baseline.0,
        "actual family worker retains its resource allocation"
    );
    assert_eq!(
        entered.1 + 1,
        baseline.1,
        "worker entered after active CPU admission"
    );
    assert_eq!(
        entered.2 + 1,
        baseline.2,
        "worker entered after native job admission"
    );
    let mut waiting = Box::pin(handle.wait());
    assert!(futures_util::poll!(waiting.as_mut()).is_pending());
    cancel.cancel();
    gate.wait(true).await;
    assert!(
        futures_util::poll!(waiting.as_mut()).is_pending(),
        "caller stop must await issued worker return and join"
    );
    let stopping = runtime.shared.math().test_admission_snapshot();
    assert_eq!(
        (stopping.1, stopping.2),
        (entered.1, entered.2),
        "active resources remain held while worker teardown is gated"
    );
    assert!(
        stopping.0 > baseline.0,
        "the issued family worker still owns its charged allocation"
    );
    gate.release();
    let result = tokio::time::timeout(Duration::from_secs(10), waiting)
        .await
        .unwrap()
        .unwrap();
    assert!(
        !result.usable(),
        "an execution cancelled before its family producer starts cannot produce usable science"
    );
    assert!(result.canonical_run_key().is_none());
    assert!(result.canonical_attempt_key().is_none());
    let drained = runtime.shared.math().test_admission_snapshot();
    assert_eq!(
        (drained.1, drained.2),
        (baseline.1, baseline.2),
        "completion follows actual native join and resource release"
    );
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn prepared_study_simulation_forwards_caller_cancel_after_worker_dispatch_and_joins_drain() {
    let runtime = tests::runtime();
    let rows = pse_authoring::language::parse(
        "package p { def Root { domain t:Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]:Time; eq rate[i in t]:d(x[i])/di==1; eq initial:x[0{s}]==1{s}; } test plant fixture { dof 0; route integrated; procedure integrate; integrate samples(0{s},1{s}) relative(1e-8) normalized_absolute(1e-10) step(1e-4{s}); } { child root:Root=Root(); } }",
        pse_ids::SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named, Default::default(),
    ).unwrap();
    let case = rows
        .iter()
        .find(|row| row.name == "plant")
        .unwrap()
        .declaration_id;
    let mut physical = tests::physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    let package = runtime.modeling_package(rows, physical).await.unwrap();
    let simulation = package
        .declared_simulation(
            case,
            tests::compiler_profile(),
            None,
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let prepared = PreparedStudyOperation::Simulation(Box::new(simulation));
    cancel_prepared_family_at_worker_entry(&runtime, &prepared).await;
    let rerun = prepared
        .start(&runtime, &crate::CancelSource::new())
        .await
        .unwrap()
        .wait()
        .await
        .unwrap();
    assert!(rerun.usable(), "{:?}", rerun.report());
    let RunReport::Simulation(trajectory) = rerun.report().unwrap() else {
        panic!("original simulation");
    };
    assert_eq!(trajectory.report().samples.last().unwrap().time, 1.0);
    assert!((trajectory.report().samples.last().unwrap().state[0] - 2.0).abs() < 1e-8);
}

#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn prepared_study_fit_forwards_caller_cancel_after_worker_dispatch_and_joins_drain() {
    use super::fitting::regression;
    use pse_backend_native::solve::HessianMode;
    let runtime = tests::runtime();
    let package =
        regression::package_on(runtime.clone(), false, regression::declared(), [1.; 4]).await;
    let fit = package
        .prepare_fit(
            tests::id(32).into(),
            regression::profile(HessianMode::Exact, None),
            tests::compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let prepared = PreparedStudyOperation::Fit(Box::new(fit));
    cancel_prepared_family_at_worker_entry(&runtime, &prepared).await;
    let rerun = prepared
        .start(&runtime, &crate::CancelSource::new())
        .await
        .unwrap()
        .wait()
        .await
        .unwrap();
    assert!(rerun.usable(), "{:?}", rerun.report());
    let RunReport::Fit(fit) = rerun.report().unwrap() else {
        panic!("original fit");
    };
    let (_, expected) = regression::analytic();
    let candidate = fit.candidate.as_ref().unwrap();
    assert_eq!(candidate.len(), expected.len());
    for (actual, expected) in candidate.iter().zip(expected) {
        assert!((actual - expected).abs() < 1e-6);
    }
}
