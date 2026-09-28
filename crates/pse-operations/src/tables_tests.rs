// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The query surface's paged statements (Plan 22 O9) against isolated databases.

use std::collections::BTreeMap;
use std::time::Duration;

use pse_model::generated::identities::RunId;

use crate::attempts::{AttemptId, NewAttempt, TransitionNote};
use crate::jobs::{JobFilter, JobState, RetryPolicy};
use crate::lifecycle::AttemptState;
use crate::store_tests::{new_attempt, new_job};
use crate::streams::{ProgressEvent, ProgressValue};
use crate::tables::{
    AttemptScan, JobScan, ProgressValueScan, Scan, TimeRange, TransitionScan,
};
use crate::testing::TestDatabase;
use crate::{Store, mint_id};

/// Every row a scan selects, read `page` at a time; each page holds at most `page` rows.
async fn drain<S: Scan>(scan: &S, store: &Store, page: i64) -> Vec<S::Row> {
    let mut rows = Vec::new();
    let mut after = None;
    loop {
        let read = scan.page(store, after.as_ref(), page).await.unwrap();
        assert!(i64::try_from(read.len()).unwrap() <= page);
        let Some(last) = read.last() else {
            return rows;
        };
        after = Some(S::key(last));
        rows.extend(read);
    }
}

#[tokio::test]
async fn table_scans_page_in_key_order_with_typed_filters() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let run: RunId = mint_id();
    let mut created: Vec<AttemptId> = Vec::new();
    for index in 0..5 {
        let attempt = NewAttempt {
            run_id: if index < 3 { run } else { mint_id() },
            ..new_attempt()
        };
        store.attempts().create(&attempt, Some("test")).await.unwrap();
        created.push(attempt.attempt_id);
    }
    let queued_id = created[4];
    store
        .attempts()
        .transition(queued_id, AttemptState::Queued, &TransitionNote::by("test"))
        .await
        .unwrap();
    created.sort();

    // Pages of two, in primary-key order, each row exactly once.
    let every = drain(&AttemptScan::default(), &store, 2).await;
    assert_eq!(
        every.iter().map(|row| row.attempt_id).collect::<Vec<_>>(),
        created
    );

    // Each typed filter reaches the statement.
    let of_run = drain(
        &AttemptScan {
            runs: vec![run],
            ..AttemptScan::default()
        },
        &store,
        10,
    )
    .await;
    assert_eq!(of_run.len(), 3);
    assert!(of_run.iter().all(|row| row.run_id == run));
    let queued = drain(
        &AttemptScan {
            states: vec![AttemptState::Queued],
            ..AttemptScan::default()
        },
        &store,
        10,
    )
    .await;
    assert_eq!(
        queued.iter().map(|row| row.attempt_id).collect::<Vec<_>>(),
        [queued_id]
    );
    // Filters on several columns are conjoined.
    let planned = drain(
        &AttemptScan {
            attempts: created.clone(),
            states: vec![AttemptState::Planned],
            ..AttemptScan::default()
        },
        &store,
        10,
    )
    .await;
    assert_eq!(
        planned.iter().map(|row| row.attempt_id).collect::<Vec<_>>(),
        created
            .iter()
            .copied()
            .filter(|id| *id != queued_id)
            .collect::<Vec<_>>()
    );
    let later = chrono::Utc::now() + Duration::from_secs(3600);
    assert!(
        drain(
            &AttemptScan {
                created: TimeRange {
                    from: Some(later),
                    to: None
                },
                ..AttemptScan::default()
            },
            &store,
            10
        )
        .await
        .is_empty()
    );
    let until_later = drain(
        &AttemptScan {
            created: TimeRange {
                from: None,
                to: Some(later),
            },
            ..AttemptScan::default()
        },
        &store,
        10,
    )
    .await;
    assert_eq!(until_later.len(), 5);

    // A composite key pages across attempts: every creation and transition row once.
    let transitions = drain(&TransitionScan::default(), &store, 1).await;
    assert_eq!(transitions.len(), 6);
    let of_queued = drain(
        &TransitionScan {
            attempts: vec![queued_id],
            ..TransitionScan::default()
        },
        &store,
        1,
    )
    .await;
    assert_eq!(
        of_queued.iter().map(|row| row.seq).collect::<Vec<_>>(),
        [0, 1]
    );

    // A three-part key with a text component.
    let events: Vec<ProgressEvent> = (0..3)
        .map(|seq| ProgressEvent {
            seq,
            step: 0,
            at: chrono::Utc::now(),
            elapsed_seconds: 0.5,
            phase: "iterate".into(),
            values: BTreeMap::from([
                ("b".to_owned(), ProgressValue::Integer(seq)),
                ("a".to_owned(), ProgressValue::Boolean(true)),
            ]),
        })
        .collect();
    store
        .streams()
        .append_progress(queued_id, &events)
        .await
        .unwrap();
    let values = drain(&ProgressValueScan::default(), &store, 4).await;
    assert_eq!(
        values
            .iter()
            .map(|row| (row.seq, row.name.as_str()))
            .collect::<Vec<_>>(),
        [(0, "a"), (0, "b"), (1, "a"), (1, "b"), (2, "a"), (2, "b")]
    );
    assert!(
        drain(
            &ProgressValueScan {
                attempts: created.iter().copied().filter(|id| *id != queued_id).collect(),
            },
            &store,
            4
        )
        .await
        .is_empty()
    );
    database.remove().await.unwrap();
}

#[tokio::test]
async fn job_listing_is_newest_first_and_scans_by_state() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let jobs = store.jobs();
    let mut enqueued = Vec::new();
    for key in ["first", "second", "third"] {
        let job = new_job(key, RetryPolicy::ONCE);
        let created = jobs.enqueue(&job).await.unwrap();
        enqueued.push(created);
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let listed = jobs.list(&JobFilter::newest(10)).await.unwrap();
    assert_eq!(
        listed
            .iter()
            .map(|job| job.idempotency_key.as_str())
            .collect::<Vec<_>>(),
        ["third", "second", "first"]
    );
    assert_eq!(jobs.list(&JobFilter::newest(1)).await.unwrap().len(), 1);
    assert!(
        jobs.list(&JobFilter {
            states: vec![JobState::Running],
            limit: 10,
        })
        .await
        .unwrap()
        .is_empty()
    );
    assert!(jobs.list(&JobFilter::newest(0)).await.is_err());
    let queued = drain(
        &JobScan {
            states: vec![JobState::Queued],
            ..JobScan::default()
        },
        &store,
        2,
    )
    .await;
    assert_eq!(queued.len(), 3);
    database.remove().await.unwrap();
}
