// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual adapter controls for caller authority and protected release handoff.

use crate::canonical::{CanonicalError, ProtectedSelection, ProtectionOwner, QueryEffect, Revision};
use std::{sync::{Arc, atomic::AtomicBool}, time::Duration};
use surrealdb::types::Value;

#[tokio::test]
async fn caller_close_rejects_escaped_aliases_but_admitted_work_and_other_caller_use_rpc() {
    let (store, mut received, server) = super::tests::canonical_read_barrier_peer().await;
    let escaped = store.clone();
    let independent = store.independent_caller().unwrap();
    let activity = store.admit_activity().unwrap();
    store.close();
    assert!(matches!(escaped.revision("closed").await, Err(CanonicalError::CallerClosed)));
    assert!(matches!(store.disconnect().await, Err(CanonicalError::ConnectionBusy { .. })));
    let mut response = activity.scope(escaped.protected_query("problem", QueryEffect::Read,
        "lifetime::continuation", || Ok(escaped.db.query("RETURN 'CONTINUATION';")))).await.unwrap();
    assert_eq!(response.take::<Value>(0).unwrap(), Value::Bool(true));
    assert_eq!(received.recv().await.unwrap(), "RETURN 'CONTINUATION'");
    drop(activity);
    let mut response = independent.protected_query("problem", QueryEffect::Read,
        "lifetime::independent", || Ok(independent.db.query("RETURN 'INDEPENDENT';"))).await.unwrap();
    assert_eq!(response.take::<Value>(0).unwrap(), Value::Bool(true));
    assert_eq!(received.recv().await.unwrap(), "RETURN 'INDEPENDENT'");
    independent.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server).await.unwrap().unwrap();
}

#[tokio::test]
async fn last_protected_owner_transfers_activity_before_reader_count_disappears() {
    let (store, mut received, server) = super::tests::canonical_read_barrier_peer().await;
    let key = "release-handoff";
    let held = store.problem_turns.gate(&format!("pin:{key}")).unwrap().write_owned().await;
    let activity = store.admit_activity().unwrap();
    store.result_read_drain.retain().unwrap();
    let selection = ProtectedSelection { owner: Arc::new(ProtectionOwner {
        key: key.into(),
        revision: Revision { key: "revision".into(), problem: "problem".into(), sequence: 1,
            parent: None, operation: "original".into(), request: vec![].into(), interpretation: "test".into() },
        store: Some(store.clone()), activity: std::sync::Mutex::new(Some(activity)), executor: Some(tokio::runtime::Handle::current()), released: AtomicBool::new(false),
    }) };
    store.close();
    drop(selection);
    // Drop removes the reader count synchronously while the original release is
    // blocked at the actual ProblemTurns admission queue. Its activity remains.
    assert_eq!(store.result_read_drain.live(), 0);
    assert!(matches!(store.disconnect().await, Err(CanonicalError::ConnectionBusy { .. })));
    drop(held);
    store.result_read_drain.drain().await.unwrap();
    assert_eq!(received.recv().await.unwrap(), crate::canonical::RELEASE_PROTECTION.trim().trim_end_matches(';'));
    store.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server).await.unwrap().unwrap();
}

#[tokio::test]
async fn failed_release_retains_exact_authority_until_explicit_original_grant_recovery() {
    let (store, mut received, server) = super::tests::canonical_read_barrier_peer().await;
    let issuing = store.activity_store().unwrap();
    store.result_read_drain.retain().unwrap();
    let revision = Revision { key: "original-revision".into(), problem: "original-problem".into(), sequence: 7,
        parent: None, operation: "original-operation".into(), request: vec![].into(), interpretation: "test".into() };
    store.result_read_drain.release_protection(issuing, revision, "original-grant".into())
        .complete(Err(CanonicalError::Timeout));
    store.close();
    assert_eq!(store.result_read_drain.live(), 0);
    assert!(matches!(store.result_read_drain.drain().await, Err(CanonicalError::ProtectionRelease { .. })));
    assert!(matches!(store.disconnect().await, Err(CanonicalError::ConnectionBusy { .. })));
    store.recover_protection_releases().await.unwrap();
    assert_eq!(received.recv().await.unwrap(), crate::canonical::RELEASE_PROTECTION.trim().trim_end_matches(';'));
    store.result_read_drain.drain().await.unwrap();
    store.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server).await.unwrap().unwrap();
}

#[tokio::test]
async fn acknowledged_protection_alias_is_idle_and_does_not_hold_physical_admission() {
    let (store, mut received, server) = super::tests::canonical_read_barrier_peer().await;
    store.result_read_drain.retain().unwrap();
    let selection = ProtectedSelection { owner: Arc::new(ProtectionOwner {
        key: "idle-released-grant".into(),
        revision: Revision { key: "revision".into(), problem: "problem".into(), sequence: 1,
            parent: None, operation: "original".into(), request: vec![].into(), interpretation: "test".into() },
        store: Some(store.clone()), activity: std::sync::Mutex::new(Some(store.admit_activity().unwrap())),
        executor: Some(tokio::runtime::Handle::current()), released: AtomicBool::new(false),
    }) };
    let alias = selection.clone();
    store.close();
    store.release(&selection).await.unwrap();
    assert_eq!(received.recv().await.unwrap(), crate::canonical::RELEASE_PROTECTION.trim().trim_end_matches(';'));
    assert_eq!(store.result_read_drain.live(), 0);
    store.disconnect().await.unwrap();
    alias.finish().await.unwrap();
    selection.finish().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server).await.unwrap().unwrap();
}

#[tokio::test]
async fn abandoned_rpc_waiter_retains_activity_until_actual_late_response_drains() {
    let (store, mut received, server) = super::tests::canonical_read_barrier_peer().await;
    let independent = store.independent_caller().unwrap();
    let active = store.clone();
    let caller = tokio::spawn(async move {
        active.protected_query("problem", QueryEffect::Read, "lifetime::late_response",
            || Ok(active.db.query("RETURN 'FIRST_READ';"))).await
    });
    assert_eq!(received.recv().await.unwrap(), "RETURN 'FIRST_READ'");
    caller.abort();
    assert!(caller.await.unwrap_err().is_cancelled());
    store.close();
    assert!(matches!(store.disconnect().await, Err(CanonicalError::ConnectionBusy { .. })));
    let mut response = independent.protected_query("problem", QueryEffect::Read,
        "lifetime::late_response_sentinel", || Ok(independent.db.query("RETURN 'SECOND_READ';"))).await.unwrap();
    assert_eq!(response.take::<Value>(0).unwrap(), Value::Bool(true));
    assert_eq!(received.recv().await.unwrap(), "RETURN 'SECOND_READ'");
    // The sentinel follows the abandoned request's response in the same native
    // driver loop; retired context metadata must no longer retain its activity.
    independent.disconnect().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), server).await.unwrap().unwrap();
}
