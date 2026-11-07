// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "owned native legacy fixtures fail on unexpected setup or settlement"
)]
use super::*;
use crate::canonical::{
    CanonicalError, CanonicalOptions, CanonicalStore, RefusalCode, bounded_query,
};
use surrealdb_types::{Bytes, Decimal, Object, Value};

async fn fixture() -> CanonicalStore {
    let state = std::env::var("PSE_SURREAL_STATE").unwrap();
    let mut options = CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
    options.database = format!(
        "canonical_test_legacy_capture_{}",
        uuid::Uuid::new_v4().simple()
    );
    let store = crate::native_fixture::canonical_fixture_with_options(&options, true).unwrap();
    // Restore only the exact old capture family in this newly owned disposable
    // database. Full predecessor→successor qualification belongs to migration.
    let declaration = crate::canonical::migration::SCHEMA_V3_DECLARATION;
    let mut overlay = String::new();
    for line in declaration.lines() {
        if line.starts_with("DEFINE TABLE canonical_study_capture_gate ")
            || line.starts_with("DEFINE FIELD ")
                && (line.contains(" ON canonical_study_capture_gate ")
                    || line.contains(" ON canonical_study_captures "))
            || line.starts_with("DEFINE EVENT ") && line.contains(" ON canonical_study_captures ")
        {
            let line = line
                .replacen("DEFINE TABLE ", "DEFINE TABLE OVERWRITE ", 1)
                .replacen("DEFINE FIELD ", "DEFINE FIELD OVERWRITE ", 1)
                .replacen("DEFINE EVENT ", "DEFINE EVENT OVERWRITE ", 1);
            overlay.push_str(&line);
            overlay.push('\n');
        }
    }
    for name in ["recover_capture", "commit_capture"] {
        let start = declaration
            .find(&format!("DEFINE FUNCTION fn::pse_workbench_v1::{name}("))
            .unwrap();
        let tail = &declaration[start..];
        let end = tail.find("\n};").unwrap() + 4;
        overlay.push_str(&tail[..end].replacen(
            "DEFINE FUNCTION ",
            "DEFINE FUNCTION OVERWRITE ",
            1,
        ));
        overlay.push('\n');
    }
    bounded_query(store.db.query(overlay)).await.unwrap();
    store
}

fn peer(store: &CanonicalStore) -> CanonicalStore {
    let mut options = CanonicalOptions::from_state(store.deployment_state()).unwrap();
    options.database = store.database().to_owned();
    crate::native_fixture::canonical_fixture_peer(store, &options).unwrap()
}

async fn pending(store: &CanonicalStore, key: &str, owner: &str) {
    bounded_query(store.db.query("LET $expiry=time::micros()+30000000; CREATE ONLY type::record('canonical_study_captures',$key) SET key=$key,owner=$owner,request=$request,incarnation='original-incarnation',issued_expires_at=<decimal>$expiry,state='pending',header_count=0dec,member_count=0dec; UPSERT ONLY type::record('canonical_study_capture_gate','current') SET key='current',capture=$key,owner=$owner,request=$request,incarnation='original-incarnation',issued_expires_at=<decimal>$expiry;")
        .bind(("key",key.to_owned())).bind(("owner",owner.to_owned())).bind(("request",Bytes::from(vec![1])))).await.unwrap();
}

async fn page(store: &CanonicalStore) -> Result<Page, CanonicalError> {
    let mut response =
        bounded_query(store.db.query(format!("{} RETURN {};", body(), RESULT))).await?;
    let index = response.num_statements() - 1;
    response
        .take::<Option<Page>>(index)?
        .ok_or(CanonicalError::IncompleteResponse)
}

#[tokio::test]
async fn legacy_pending_capture_refuses_active_owner_and_fences_stale_commit_after_authenticated_drain()
 {
    let store = fixture().await;
    let original = peer(&store);
    let owner = original.registered_file_owner().await.unwrap();
    pending(&store, "legacy-pending", &owner).await;
    assert!(matches!(
        page(&store).await,
        Err(CanonicalError::Refused {
            code: RefusalCode::StudyCaptureLegacyPendingNotDrained,
            ..
        })
    ));
    original
        .drain_owned_connection_until(
            tokio::time::Instant::now() + std::time::Duration::from_secs(10),
        )
        .await
        .unwrap();
    let settled = page(&store).await.unwrap();
    assert_eq!(settled.capture.as_deref(), Some("legacy-pending"));
    assert!(settled.done);
    let complete = page(&store).await.unwrap();
    assert!(complete.capture.is_none() && complete.done);
    let mut saved=bounded_query(store.db.query("SELECT state,owner,request FROM ONLY type::record('canonical_study_captures','legacy-pending'); SELECT incarnation,capture FROM ONLY type::record('canonical_study_capture_gate','current');")).await.unwrap();
    let receipt = saved.take::<Option<Object>>(0).unwrap().unwrap();
    assert_eq!(
        receipt.get("state"),
        Some(&Value::String("released".into()))
    );
    assert_eq!(receipt.get("owner"), Some(&Value::String(owner.clone())));
    let gate = saved.take::<Option<Object>>(1).unwrap().unwrap();
    assert_ne!(
        gate.get("incarnation"),
        Some(&Value::String("original-incarnation".into()))
    );
    assert!(matches!(gate.get("capture"), None | Some(Value::None)));
    drop(saved);
    let late=bounded_query(store.db.query("RETURN fn::pse_workbench_v1::commit_capture($pse_rpc_expires_at,'legacy-pending',$owner,$request,[],1,1);").bind(("owner",owner)).bind(("request",Bytes::from(vec![1])))).await.unwrap_err();
    assert!(matches!(
        late,
        CanonicalError::Refused {
            code: RefusalCode::StudyCaptureOriginalGrantClosed,
            ..
        }
    ));
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn legacy_abandoned_frozen_capture_cleanup_is_paged_and_preserves_other_capture_facts() {
    let store = fixture().await;
    let original = peer(&store);
    let owner = original.registered_file_owner().await.unwrap();
    pending(&store, "legacy-frozen", &owner).await;
    bounded_query(store.db.query("UPDATE ONLY type::record('canonical_study_captures','legacy-frozen') SET header_count=1dec,member_count=257dec; UPDATE ONLY type::record('canonical_study_captures','legacy-frozen') SET state='frozen'; UPDATE ONLY type::record('canonical_study_capture_gate','current') SET capture=NONE,owner=NONE,request=NONE,issued_expires_at=0dec;")).await.unwrap();
    let rows = (0..257)
        .map(|ordinal| {
            let mut row = Object::new();
            row.insert("key", format!("legacy-frozen:{ordinal}"));
            row.insert("capture", "legacy-frozen");
            row.insert("ordinal", Decimal::from(ordinal));
            row.insert("body", Bytes::from(vec![1]));
            row.insert("study", "study");
            row.insert("run", "run");
            row.insert("attempt", Value::None);
            row.insert("manifest", Value::None);
            row
        })
        .collect::<Vec<_>>();
    bounded_query(store.db.query("INSERT INTO canonical_study_capture_members $rows.map(|$row|object::extend($row,{id:type::record('canonical_study_capture_members',$row.key)})); CREATE ONLY type::record('canonical_study_capture_headers','legacy-frozen:0') SET key='legacy-frozen:0',capture='legacy-frozen',ordinal=0dec,body=$body; CREATE ONLY type::record('canonical_study_capture_headers','other-capture:0') SET key='other-capture:0',capture='other-capture',ordinal=0dec,body=$body;")
        .bind(("rows",rows)).bind(("body",Bytes::from(vec![1])))).await.unwrap();
    original
        .drain_owned_connection_until(
            tokio::time::Instant::now() + std::time::Duration::from_secs(10),
        )
        .await
        .unwrap();
    let first = page(&store).await.unwrap();
    assert!(!first.done);
    let second = page(&store).await.unwrap();
    assert!(second.done);
    let mut remaining=bounded_query(store.db.query("SELECT key FROM canonical_study_capture_members WHERE capture='legacy-frozen'; SELECT key FROM canonical_study_capture_headers WHERE capture='legacy-frozen'; SELECT key FROM canonical_study_capture_headers WHERE capture='other-capture';")).await.unwrap();
    assert!(remaining.take::<Vec<Object>>(0).unwrap().is_empty());
    assert!(remaining.take::<Vec<Object>>(1).unwrap().is_empty());
    assert_eq!(remaining.take::<Vec<Object>>(2).unwrap().len(), 1);
    drop(remaining);
    store.remove_isolated_fixture().await.unwrap();
}
