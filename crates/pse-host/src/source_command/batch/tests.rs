// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

#[tokio::test]
async fn requested_detail_reaches_each_batch_task_and_exhausted_pages_construct_none() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first.rs");
    let last = root.path().join("last.rs");
    let docs = root.path().join("docs.rs");
    std::fs::write(&first, "fn first() { let s = \"a\\nβ\"; }").unwrap();
    std::fs::write(&last, "fn last() { let s = \"unselected\"; }").unwrap();
    std::fs::write(&docs, "//! file-owned docs").unwrap();
    let mut task = start_page(
        vec![first, last, docs],
        None,
        RustEdition::E2024,
        CancellationToken::new(),
        Some(1),
        crate::source_read::SourceDetail::Mapped,
    );
    let response = task.settle().await.unwrap();
    let Record::Read { reading: first } = &response.payload.records[0] else {
        panic!("first source was not read")
    };
    let Record::Read { reading: last } = &response.payload.records[1] else {
        panic!("last source was not read")
    };
    assert_eq!(first.outline.facts.len(), 1);
    assert_eq!(first.outline.facts[0].mapping.as_ref().unwrap().len(), 3);
    assert!(last.outline.facts.is_empty());
    assert!(last.outline.detail_coverage.ranges.is_empty());
    assert!(!last.outline.detail_coverage.includes_file_facts);
    let Record::Read { reading: docs } = &response.payload.records[2] else {
        panic!("doc-only source was not read")
    };
    assert!(docs.outline.entries.is_empty());
    assert!(docs.outline.detail_coverage.includes_file_facts);
    assert_eq!(docs.outline.facts[0].written, "//! file-owned docs");
}

#[tokio::test]
async fn native_cancel_hook_releases_actual_work_without_cancelling_sibling() {
    use std::sync::{Arc, Condvar, Mutex};
    let parent = CancellationToken::new();
    let sibling = parent.child_token();
    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let worker_release = release.clone();
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let mut task = spawn_cpu(parent.clone(), move |token| {
        started_tx.send(()).unwrap();
        let (flag, ready) = &*worker_release;
        let mut released = flag.lock().unwrap();
        while !*released {
            released = ready.wait(released).unwrap();
        }
        token.is_cancelled()
    })
    .with_cancel_hook(move || {
        let (flag, ready) = &*release;
        *flag.lock().unwrap() = true;
        ready.notify_all();
    });
    started_rx.await.unwrap();
    assert!(task.settle().now_or_never().is_none());
    task.cancel();
    assert!(task.settle().await.unwrap());
    assert!(!parent.is_cancelled());
    assert!(!sibling.is_cancelled());
}

#[tokio::test]
async fn declaration_budget_applies_across_nested_entries_and_all_operand_outcomes_remain() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first.rs");
    let last = root.path().join("last.rs");
    std::fs::write(&first, "mod m { fn one() {} fn two() {} }").unwrap();
    std::fs::write(&last, "fn last() {}").unwrap();
    let mut task = start_page(
        vec![first, root.path().join("missing.rs"), last],
        None,
        RustEdition::E2024,
        CancellationToken::new(),
        Some(2),
        crate::source_read::SourceDetail::Navigation,
    );
    let response = task.settle().await.unwrap();
    assert_eq!(response.payload.records.len(), 3);
    let lengths: Vec<_> = response
        .payload
        .records
        .iter()
        .filter_map(|record| match record {
            Record::Read { reading } => Some(reading.outline.entries.len()),
            _ => None,
        })
        .collect();
    assert_eq!(lengths, [2, 0]);
    let Record::Read { reading } = &response.payload.records[0] else {
        panic!("first read")
    };
    assert_eq!(reading.page.total_records, 3);
    assert_eq!(reading.page.omitted_records, 1);
    assert!(matches!(response.payload.records[1], Record::Failed { .. }));
    assert_eq!(response.failures.len(), 1);
    assert!(response.payload.cpu_work_completed);
}
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
        "inspect outline batch",
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
                source: crate::source_read::SourceText::new(
                    std::env::current_dir().unwrap().join(file),
                    "fixture".into(),
                )
                .unwrap()
                .identity()
                .clone(),
                page: crate::source_read::PageState {
                    total_records: 1,
                    offset: 0,
                    returned_records: 1,
                    omitted_records: 0,
                    next_cursor: None,
                },
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
                source: crate::source_read::SourceText::new(
                    std::env::current_dir().unwrap().join(file),
                    "fixture".into(),
                )
                .unwrap()
                .identity()
                .clone(),
                page: crate::source_read::PageState {
                    total_records: 1,
                    offset: 0,
                    returned_records: 1,
                    omitted_records: 0,
                    next_cursor: None,
                },
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

#[tokio::test]
async fn worker_panics_keep_redacted_string_messages_and_explicit_opaque_payloads() {
    for kind in 0..3 {
        let mut task = submit(
            vec!["panic.rs".into()],
            CancellationToken::new(),
            move |_| match kind {
                0 => std::panic::panic_any(String::from("hidden-string")),
                1 => std::panic::panic_any("hidden-static"),
                _ => std::panic::panic_any(7u8),
            },
        );
        task.redaction = RedactionPolicy::new([("TOKEN".into(), "hidden".into())]);
        let failure = task.settle().await.unwrap_err();
        match (kind, failure) {
            (0, SourceCommandError::WorkerPanic(message)) => {
                assert_eq!(message, "<redacted:TOKEN>-string")
            }
            (1, SourceCommandError::WorkerPanic(message)) => {
                assert_eq!(message, "<redacted:TOKEN>-static")
            }
            (2, SourceCommandError::OpaqueWorkerPanic) => {}
            (_, other) => panic!("unexpected payload interpretation {other:?}"),
        }
    }
}
