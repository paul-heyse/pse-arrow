// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Small actual-executor controls, independent of the scientific unit-test harness.
#![cfg(all(feature = "diagnostic-taskdump", tokio_unstable))]
#![allow(
    clippy::unwrap_used,
    reason = "controlled diagnostic fixture setup fails directly"
)]
use pse_model::document::Version;
use pse_runtime::tooling::taskdump::{TaskDumpRequest, request};
use pse_runtime::{CancelSource, observation_detail::CaptureError};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn actual_executor_traces_pending_channel_and_retains_issued_request() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let (_send, receive) = tokio::sync::oneshot::channel::<()>();
    let pending = runtime.spawn(async move {
        let _ = receive.await;
    });
    let selected_request = TaskDumpRequest {
        version: Version,
        wait: Duration::from_secs(5),
        maximum_tasks: 32,
        maximum_bytes: 65536,
    };
    let ticket = request(
        runtime.handle().clone(),
        selected_request,
        &CancelSource::new(),
    )
    .unwrap();
    let result = ticket.wait(Duration::from_secs(5)).unwrap();
    assert_eq!(result.target_state, "completed");
    assert!(
        result
            .tasks
            .iter()
            .any(|task| task.task == format!("{:?}", pending.id()))
    );
    assert!(!pending.is_finished());
    pending.abort();
}

#[test]
fn blocked_worker_outlives_observer_and_later_settles_same_request() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let (entered, waiting) = std::sync::mpsc::channel();
    let (release, blocked) = std::sync::mpsc::channel();
    let work = runtime.spawn(async move {
        entered.send(()).unwrap();
        blocked.recv().unwrap();
    });
    waiting.recv_timeout(Duration::from_secs(5)).unwrap();
    let ticket = request(
        runtime.handle().clone(),
        TaskDumpRequest {
            version: Version,
            wait: Duration::from_millis(20),
            maximum_tasks: 32,
            maximum_bytes: 65536,
        },
        &CancelSource::new(),
    )
    .unwrap();
    let owner = Arc::new(ticket);
    let observer = Arc::clone(&owner);
    let result = observer.wait(Duration::from_millis(20)).unwrap();
    assert_eq!(result.target_state, "pending");
    assert!(result.observer_wait_expired);
    let identity = result.request;
    drop(observer);
    assert!(!work.is_finished());
    release.send(()).unwrap();
    let settled = owner.wait(Duration::from_secs(5)).unwrap();
    assert_eq!(settled.request, identity);
    assert_eq!(settled.target_state, "completed");
}

#[test]
fn cancelled_before_issuance_does_not_require_cooperating_executor() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let cancel = CancelSource::new();
    cancel.cancel();
    let result = request(
        runtime.handle().clone(),
        TaskDumpRequest {
            version: Version,
            wait: Duration::from_secs(1),
            maximum_tasks: 1,
            maximum_bytes: 64,
        },
        &cancel,
    );
    assert!(
        matches!(result, Err(CaptureError::Invalid(message)) if message.contains("cancelled before issuance"))
    );
}

#[test]
fn unsupported_local_executor_refuses_before_issuance() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    assert!(
        request(
            runtime.handle().clone(),
            TaskDumpRequest {
                version: Version,
                wait: Duration::from_secs(1),
                maximum_tasks: 1,
                maximum_bytes: 64,
            },
            &CancelSource::new()
        )
        .is_err()
    );
}
