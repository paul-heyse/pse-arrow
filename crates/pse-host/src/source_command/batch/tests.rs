// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
#[tokio::test]
async fn indexed_batch_retains_file_order_and_all_independent_failures() {
    let root = tempfile::tempdir().unwrap();
    let a = root.path().join("a.rs");
    let missing = root.path().join("missing.rs");
    let b = root.path().join("b.md");
    std::fs::write(&a, "fn a() {}\n").unwrap();
    std::fs::write(&b, "# b\n").unwrap();
    let mut task = start(
        vec![a.clone(), missing.clone(), b.clone()],
        None,
        RustEdition::E2024,
        CancellationToken::new(),
    );
    let response = task.settle().await.unwrap();
    assert_eq!(response.exit_code(), 1);
    assert!(response.payload.cpu_work_completed);
    assert_eq!(response.failures.len(), 1);
    assert!(
        matches!(&response.payload.records[0],Record::Read{reading} if reading.file==a.as_os_str())
    );
    assert!(
        matches!(&response.payload.records[1],Record::Failed{file,..} if file==missing.as_os_str())
    );
    assert!(
        matches!(&response.payload.records[2],Record::Read{reading} if reading.file==b.as_os_str())
    );
    let schema = crate::envelope::schema::<Report>(
        "outline batch",
        &crate::envelope::Presentation::default(),
    )
    .unwrap();
    jsonschema::validator_for(&schema.to_value())
        .unwrap()
        .validate(&serde_json::to_value(response).unwrap())
        .unwrap();
    assert!(matches!(
        task.settle().await,
        Err(SourceCommandError::AlreadySettled)
    ));
}
#[tokio::test]
async fn cancelled_inputs_are_explicit_but_completion_requires_actual_return() {
    let token = CancellationToken::new();
    token.cancel();
    let mut task = start(
        vec!["one.rs".into(), "two.rs".into()],
        None,
        RustEdition::E2024,
        token,
    );
    let response = task.settle().await.unwrap();
    assert_eq!(response.exit_code(), 130);
    assert!(response.payload.cpu_work_completed);
    assert!(response.payload.cancellation_requested);
    assert!(
        response
            .payload
            .records
            .iter()
            .all(|record| matches!(record, Record::Cancelled { .. }))
    );
}
#[tokio::test]
async fn cancelling_a_settle_awaiter_keeps_running_cpu_handle_until_owner_settles() {
    use std::sync::{Arc, Condvar, Mutex};
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let started_tx = Mutex::new(Some(started_tx));
    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let worker_release = release.clone();
    let mut task = submit(
        vec!["controlled.rs".into()],
        CancellationToken::new(),
        move |file| {
            assert!(rayon::current_thread_index().is_some());
            started_tx.lock().unwrap().take().unwrap().send(()).unwrap();
            let (flag, ready) = &*worker_release;
            let mut released = flag.lock().unwrap();
            while !*released {
                released = ready.wait(released).unwrap();
            }
            Ok(FileOutline {
                file: file.as_os_str().into(),
                outline: crate::source_read::rust("fn complete() {}", RustEdition::E2024),
            })
        },
    );
    started_rx.await.unwrap();
    assert!(task.settle().now_or_never().is_none());
    task.cancel();
    {
        let (flag, ready) = &*release;
        *flag.lock().unwrap() = true;
        ready.notify_all();
    }
    let response = task.settle().await.unwrap();
    assert!(response.payload.cpu_work_completed);
    assert!(response.payload.cancellation_requested);
    assert!(matches!(&response.payload.records[0], Record::Read { .. }));
}

#[tokio::test]
async fn independent_files_execute_on_distinct_shared_workers_without_blocking_tokio() {
    use std::sync::{Arc, Condvar, Mutex};
    // Honor an explicitly selected one-worker caller pool without inventing a new pool.
    if rayon::current_num_threads() < 2 {
        return;
    }
    let (started_tx, mut started_rx) = tokio::sync::mpsc::unbounded_channel();
    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let worker_release = release.clone();
    let mut task = submit(
        vec!["one.rs".into(), "two.rs".into()],
        CancellationToken::new(),
        move |file| {
            started_tx
                .send(rayon::current_thread_index().unwrap())
                .unwrap();
            let (flag, ready) = &*worker_release;
            let mut released = flag.lock().unwrap();
            while !*released {
                released = ready.wait(released).unwrap();
            }
            Ok(FileOutline {
                file: file.as_os_str().into(),
                outline: crate::source_read::rust("fn file() {}", RustEdition::E2024),
            })
        },
    );
    let observed = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let first = started_rx.recv().await.unwrap();
        let second = started_rx.recv().await.unwrap();
        (first, second)
    })
    .await;
    // Always release and settle, even if a broken serial implementation timed out.
    {
        let (flag, ready) = &*release;
        *flag.lock().unwrap() = true;
        ready.notify_all();
    }
    let response = task.settle().await.unwrap();
    assert_eq!(response.payload.records.len(), 2);
    let (first, second) =
        observed.expect("both independent CPU jobs should start before either is released");
    assert_ne!(first, second);
}
