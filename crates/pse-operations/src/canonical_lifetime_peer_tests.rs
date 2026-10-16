// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual adapter controls for caller authority and protected release handoff.

use crate::canonical::{
    CanonicalError, ProtectedSelection, ProtectionOwner, QueryEffect, Revision,
};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};
use surrealdb::types::Value;

#[tokio::test]
async fn caller_close_rejects_escaped_aliases_but_admitted_work_and_other_caller_use_rpc() {
    let (store, mut received, server) = super::tests::canonical_read_barrier_peer().await;
    let escaped = store.clone();
    let independent = store.independent_caller().unwrap();
    let activity = store.admit_activity().unwrap();
    store.close();
    assert!(matches!(
        escaped.revision("closed").await,
        Err(CanonicalError::CallerClosed)
    ));
    assert!(matches!(
        store.disconnect().await,
        Err(CanonicalError::ConnectionBusy { .. })
    ));
    let mut response = activity
        .scope(escaped.protected_query(
            "problem",
            QueryEffect::Read,
            "lifetime::continuation",
            || Ok(escaped.db.query("RETURN 'CONTINUATION';")),
        ))
        .await
        .unwrap();
    assert_eq!(response.take::<Value>(0).unwrap(), Value::Bool(true));
    assert_eq!(received.recv().await.unwrap(), "RETURN 'CONTINUATION'");
    drop(activity);
    let mut response = independent
        .protected_query(
            "problem",
            QueryEffect::Read,
            "lifetime::independent",
            || Ok(independent.db.query("RETURN 'INDEPENDENT';")),
        )
        .await
        .unwrap();
    assert_eq!(response.take::<Value>(0).unwrap(), Value::Bool(true));
    assert_eq!(received.recv().await.unwrap(), "RETURN 'INDEPENDENT'");
    independent.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn last_protected_owner_transfers_activity_before_reader_count_disappears() {
    let (store, mut received, server) = super::tests::canonical_read_barrier_peer().await;
    let key = "release-handoff";
    let held = store
        .problem_turns
        .gate(&format!("pin:{key}"))
        .unwrap()
        .write_owned()
        .await;
    let activity = store.admit_activity().unwrap();
    store.result_read_drain.retain().unwrap();
    let selection = ProtectedSelection {
        owner: Arc::new(ProtectionOwner {
            key: key.into(),
            revision: Revision {
                key: "revision".into(),
                problem: "problem".into(),
                sequence: 1,
                parent: None,
                operation: "original".into(),
                request: vec![].into(),
                interpretation: "test".into(),
            },
            store: Some(store.clone()),
            activity: std::sync::Mutex::new(Some(activity)),
            executor: Some(tokio::runtime::Handle::current()),
            released: AtomicBool::new(false),
            metadata_owner: std::sync::OnceLock::new(),
        }),
    };
    use pse_columnar::{GreedyMemoryPool, MemoryConsumer, MemoryPool};
    // Refused overlapping admission attaches the original whole response grant
    // instead of an independently allocated retained-metadata grant.
    let retained_bytes = crate::canonical::MetadataResponse::Selection.allocation_bytes();
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(retained_bytes));
    let allocation = MemoryConsumer::new("test:queued-protection-metadata").register(&pool);
    allocation.try_grow(retained_bytes).unwrap();
    let allocation: Arc<dyn std::fmt::Debug + Send + Sync> = Arc::new(allocation);
    let late_sdk = allocation.clone();
    selection.retain_metadata_owner(allocation.clone()).unwrap();
    selection.retain_metadata_owner(allocation.clone()).unwrap();
    assert!(selection.retain_metadata_owner(Arc::new(())).is_err());
    drop(allocation);
    store.close();
    drop(selection);
    // Drop removes the reader count synchronously while the original release is
    // blocked at the actual ProblemTurns admission queue. Its activity remains.
    assert_eq!(store.result_read_drain.live(), 0);
    assert_eq!(
        pool.reserved(),
        retained_bytes,
        "queued cleanup retains actual metadata after the facade drops"
    );
    drop(late_sdk);
    assert_eq!(
        pool.reserved(),
        retained_bytes,
        "late original SDK release cannot retire queued cleanup's fallback grant"
    );
    assert!(matches!(
        store.disconnect().await,
        Err(CanonicalError::ConnectionBusy { .. })
    ));
    drop(held);
    store.result_read_drain.drain().await.unwrap();
    assert_eq!(
        pool.reserved(),
        0,
        "actual tracked cleanup and SDK drain release metadata"
    );
    assert_eq!(
        received.recv().await.unwrap(),
        crate::canonical::RELEASE_PROTECTION
            .trim()
            .trim_end_matches(';')
    );
    store.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn failed_release_retains_exact_authority_until_explicit_original_grant_recovery() {
    let (store, mut received, server) = super::tests::canonical_read_barrier_peer().await;
    let issuing = store.activity_store().unwrap();
    store.result_read_drain.retain().unwrap();
    let revision = Revision {
        key: "original-revision".into(),
        problem: "original-problem".into(),
        sequence: 7,
        parent: None,
        operation: "original-operation".into(),
        request: vec![].into(),
        interpretation: "test".into(),
    };
    use pse_columnar::{GreedyMemoryPool, MemoryConsumer, MemoryPool};
    let retained_bytes =
        64 * 1024 + crate::canonical::MetadataResponse::Protection.allocation_bytes();
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(retained_bytes));
    let allocation = MemoryConsumer::new("test:failed-protection-metadata").register(&pool);
    allocation.try_grow(retained_bytes).unwrap();
    store
        .result_read_drain
        .release_protection(
            issuing,
            revision,
            "original-grant".into(),
            Some(Arc::new(allocation)),
        )
        .complete(Err(CanonicalError::Timeout));
    assert_eq!(
        pool.reserved(),
        retained_bytes,
        "failure recovery owns original metadata"
    );
    store.close();
    assert_eq!(store.result_read_drain.live(), 0);
    assert!(matches!(
        store.result_read_drain.drain().await,
        Err(CanonicalError::ProtectionRelease { .. })
    ));
    assert!(matches!(
        store.disconnect().await,
        Err(CanonicalError::ConnectionBusy { .. })
    ));
    store.recover_protection_releases().await.unwrap();
    assert_eq!(
        pool.reserved(),
        0,
        "acknowledged original cleanup releases its actual allocation"
    );
    assert_eq!(
        received.recv().await.unwrap(),
        crate::canonical::RELEASE_PROTECTION
            .trim()
            .trim_end_matches(';')
    );
    store.result_read_drain.drain().await.unwrap();
    store.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn acknowledged_protection_alias_is_idle_and_does_not_hold_physical_admission() {
    let (store, mut received, server) = super::tests::canonical_read_barrier_peer().await;
    store.result_read_drain.retain().unwrap();
    let selection = ProtectedSelection {
        owner: Arc::new(ProtectionOwner {
            key: "idle-released-grant".into(),
            revision: Revision {
                key: "revision".into(),
                problem: "problem".into(),
                sequence: 1,
                parent: None,
                operation: "original".into(),
                request: vec![].into(),
                interpretation: "test".into(),
            },
            store: Some(store.clone()),
            activity: std::sync::Mutex::new(Some(store.admit_activity().unwrap())),
            executor: Some(tokio::runtime::Handle::current()),
            released: AtomicBool::new(false),
            metadata_owner: std::sync::OnceLock::new(),
        }),
    };
    let alias = selection.clone();
    store.close();
    store.release(&selection).await.unwrap();
    assert_eq!(
        received.recv().await.unwrap(),
        crate::canonical::RELEASE_PROTECTION
            .trim()
            .trim_end_matches(';')
    );
    assert_eq!(store.result_read_drain.live(), 0);
    store.disconnect().await.unwrap();
    alias.finish().await.unwrap();
    selection.finish().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn abandoned_rpc_waiter_retains_activity_until_actual_late_response_drains() {
    let (store, mut received, server) = super::tests::canonical_read_barrier_peer().await;
    let independent = store.independent_caller().unwrap();
    let active = store.clone();
    let caller = tokio::spawn(async move {
        active
            .protected_query(
                "problem",
                QueryEffect::Read,
                "lifetime::late_response",
                || Ok(active.db.query("RETURN 'FIRST_READ';")),
            )
            .await
    });
    assert_eq!(received.recv().await.unwrap(), "RETURN 'FIRST_READ'");
    caller.abort();
    assert!(caller.await.unwrap_err().is_cancelled());
    store.close();
    assert!(matches!(
        store.disconnect().await,
        Err(CanonicalError::ConnectionBusy { .. })
    ));
    let mut response = independent
        .protected_query(
            "problem",
            QueryEffect::Read,
            "lifetime::late_response_sentinel",
            || Ok(independent.db.query("RETURN 'SECOND_READ';")),
        )
        .await
        .unwrap();
    assert_eq!(response.take::<Value>(0).unwrap(), Value::Bool(true));
    assert_eq!(received.recv().await.unwrap(), "RETURN 'SECOND_READ'");
    // The sentinel follows the abandoned request's response in the same native
    // driver loop; retired context metadata must no longer retain its activity.
    independent.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn final_fixture_teardown_seals_admission_and_joins_actual_response_work() {
    use std::{future::Future, task::Poll};
    let (mut store, mut received, server) = super::tests::canonical_read_barrier_peer().await;
    // This wire fixture owns its clients directly and needs no service disposal.
    store.database = "canonical_test_wire_terminal_drain".into();
    let activity = store.admit_activity().unwrap();
    let first_store = store.clone();
    let first_activity = activity.clone();
    let first = tokio::spawn(async move {
        first_activity
            .scope(first_store.protected_query(
                "problem",
                QueryEffect::Read,
                "lifetime::first_terminal",
                || Ok(first_store.db.query("RETURN 'FIRST_READ';")),
            ))
            .await
    });
    assert_eq!(received.recv().await.unwrap(), "RETURN 'FIRST_READ'");
    let mut cleanup = Box::pin(store.remove_isolated_fixture());
    std::future::poll_fn(|cx| {
        assert!(cleanup.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    assert!(matches!(
        store.admit_activity(),
        Err(CanonicalError::ConnectionUnavailable)
    ));
    assert!(matches!(
        store.disconnect().await,
        Err(CanonicalError::ConnectionBusy { .. })
    ));
    let mut second = activity
        .scope(store.protected_query(
            "problem",
            QueryEffect::Read,
            "lifetime::second_terminal",
            || Ok(store.db.query("RETURN 'SECOND_READ';")),
        ))
        .await
        .unwrap();
    assert_eq!(second.take::<Value>(0).unwrap(), Value::Bool(true));
    assert_eq!(received.recv().await.unwrap(), "RETURN 'SECOND_READ'");
    let mut first = first.await.unwrap().unwrap();
    assert_eq!(first.take::<Value>(0).unwrap(), Value::Bool(true));
    std::future::poll_fn(|cx| {
        assert!(cleanup.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    drop(activity);
    tokio::time::timeout(Duration::from_secs(2), cleanup)
        .await
        .unwrap()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
    assert!(received.try_recv().is_err());
}

#[allow(
    clippy::unwrap_used,
    reason = "test-only wire fixture requires its current-thread runtime to initialize successfully"
)]
async fn decision_wire_fixture(
    mut store: crate::canonical::CanonicalStore,
) -> crate::canonical::CanonicalStore {
    static EXECUTOR: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    let executor = EXECUTOR.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
    });
    let lifetime = Arc::new(crate::testing::FixtureLifetime::new(
        store.clone(),
        executor,
        "wire-decision-only".into(),
    ));
    // No service resource was allocated. Exercise the real fixture membership
    // and final decision while keeping fallback registry publication disabled.
    *lifetime.removed.lock().await = true;
    store.fixture_lifetime = Some(lifetime);
    store
}

#[tokio::test]
async fn independent_fixture_caller_mint_finishes_before_final_borrower_decision() {
    let (store, mut received, server) = super::tests::canonical_read_barrier_peer().await;
    let store = Arc::new(decision_wire_fixture(store).await);
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let issuer = store.clone();
    let paused = barrier.clone();
    let mint = std::thread::spawn(move || {
        crate::canonical::lifetime::pause_next_caller_mint(paused);
        issuer.independent_caller().unwrap()
    });
    barrier.wait();
    // Pause precisely after Open was validated but before fixture Arc cloning.
    // The final decision must be excluded throughout this old race window.
    let admission_held = store.db.access.physical.caller_mint_holds_admission();
    barrier.wait();
    let borrower = mint.join().unwrap();
    assert!(admission_held);
    assert!(
        !store.seal_final_fixture_owner(),
        "the completely constructed caller is a registered fixture borrower"
    );
    drop(store.admit_activity().unwrap());
    drop(borrower);
    assert!(store.seal_final_fixture_owner());
    store.drain_fixture_connection().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
    assert!(received.try_recv().is_err());
}

#[tokio::test]
async fn final_fixture_decision_fences_independent_mint_and_clones_remain_closed_aliases() {
    let (store, mut received, server) = super::tests::canonical_read_barrier_peer().await;
    let store = decision_wire_fixture(store).await;
    assert!(store.seal_final_fixture_owner());
    assert!(matches!(
        store.independent_caller(),
        Err(CanonicalError::ConnectionUnavailable)
    ));
    let closed_alias = store.clone();
    assert!(matches!(
        closed_alias.admit_activity(),
        Err(CanonicalError::ConnectionUnavailable)
    ));
    assert!(
        store.seal_final_fixture_owner(),
        "a closed alias cannot undo the original final decision"
    );
    store.drain_fixture_connection().await.unwrap();
    drop(closed_alias);
    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .unwrap()
        .unwrap();
    assert!(received.try_recv().is_err());
}
