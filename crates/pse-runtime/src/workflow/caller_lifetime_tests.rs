// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Workflow caller ownership through actual native drain and canonical aliases.

use super::*;
use pse_operations::canonical::CanonicalError;
use std::time::Duration;

#[tokio::test]
async fn escaped_store_operations_and_runtime_aliases_close_but_independent_runtime_remains_open() {
    let runtime = durable_tests::durable_runtime();
    let escaped = runtime.canonical_store()?.clone();
    let operations = runtime.operations().unwrap().clone();
    let alias = runtime.clone();
    let independent = Runtime::from_shared(
        runtime.shared.clone(),
        runtime.model.clone(),
        runtime.canonical.clone(),
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
        alias.canonical_store()?.admit_activity(),
        Err(CanonicalError::CallerClosed)
    ));
    assert!(
        independent
            .canonical_store()?
            .revision("absent")
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn abandoned_native_waiter_keeps_activity_until_worker_thread_is_joined() {
    let runtime = tests::runtime();
    let activity = runtime.canonical_store()?.admit_activity().unwrap();
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
        runtime.canonical_store()?.disconnect().await,
        Err(CanonicalError::ConnectionBusy { .. })
    ));
    release.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match runtime.canonical_store()?.disconnect().await {
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
        runtime.canonical_store()?.disconnect().await,
        Err(CanonicalError::ConnectionBusy { .. })
    ));
    drop(attempt);
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match runtime.canonical_store()?.disconnect().await {
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
    let runtime = tests::runtime();
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
    assert!(matches!(
        prepared.start(),
        Err(WorkflowError::Canonical(CanonicalError::CallerClosed))
    ));
    runtime.canonical_store()?.disconnect().await.unwrap();
}
