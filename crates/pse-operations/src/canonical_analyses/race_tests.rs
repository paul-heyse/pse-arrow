// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Acknowledged function execution and explicit commit establish backend ordering.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "isolated transaction race fixtures and assertions fail the test on unexpected results"
)]

use super::*;
use crate::canonical::CanonicalOptions;

async fn server_now(store: &CanonicalStore) -> u64 {
    let mut response = bounded_query(store.db.query("RETURN time::micros();"))
        .await
        .unwrap();
    u64::try_from(codec::decode_int(response.take::<surrealdb::types::Value>(0).unwrap()).unwrap())
        .unwrap()
}

// An independent test connection deliberately leaves the production bounded-request
// profile unchanged: its pinned adapter does not expose interactive sessions.
async fn held_transaction(
    options: &CanonicalOptions,
) -> surrealdb::method::Transaction<surrealdb::engine::remote::ws::Client> {
    let address = url::Url::parse(&options.endpoint)
        .unwrap()
        .socket_addrs(|| None)
        .unwrap()[0];
    let db = surrealdb::Surreal::new::<surrealdb::engine::remote::ws::Ws>(address)
        .await
        .unwrap();
    db.signin(surrealdb::opt::auth::Root {
        username: options.username.clone(),
        password: options.password.clone(),
    })
    .await
    .unwrap();
    db.use_ns(options.namespace.clone())
        .use_db(options.database.clone())
        .await
        .unwrap();
    db.begin().await.unwrap()
}

fn assert_conflict(error: surrealdb::Error) {
    assert!(
        matches!(
            error.query_details(),
            Some(surrealdb::types::QueryError::TransactionConflict)
        ),
        "expected backend transaction conflict, received {error}"
    );
}

fn rpc_expiry() -> i64 {
    i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros(),
    )
    .unwrap()
        + 30_000_000
}

fn root(intent: &Analysis, revision: &Revision) -> Object {
    let mut root = Object::new();
    root.insert(
        "key",
        codec::encode_string(analysis_key(
            "pse.analysis.root.v1",
            &[intent.key.as_bytes(), revision.key.as_bytes()],
        ))
        .unwrap(),
    );
    root.insert(
        "problem",
        codec::encode_string(revision.problem.clone()).unwrap(),
    );
    root.insert(
        "revision",
        codec::encode_string(revision.key.clone()).unwrap(),
    );
    root.insert("sequence", codec::encode_uint(revision.sequence).unwrap());
    root
}

async fn absent_creation_settlement_order(creation_commits_first: bool) {
    let state = std::env::var("PSE_SURREAL_STATE").expect("explicit canonical fixture required");
    let mut options = CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
    options.database = format!(
        "canonical_test_analysis_race_{}",
        uuid::Uuid::new_v4().simple()
    );
    let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
    let revision = store
        .edit("race-source", None, "race-initial", &[])
        .await
        .unwrap();
    let mut intent = store
        .new_analysis(&revision, "held-creation:v2", vec![], "empty".into(), 0, 0)
        .await
        .unwrap();
    intent.creation_expires_at = server_now(&store).await + 2_000_000;
    intent.key = occurrence_key(
        &intent.primary_authority,
        &intent.creation_nonce,
        intent.creation_expires_at,
    );
    seal_analysis_request(&mut intent, std::slice::from_ref(&revision), &[], &[], &[]);

    let creation = held_transaction(&options).await;
    let response = creation
        .query("RETURN fn::pse_analysis_v1::begin($expiry,$row,$sources,$inputs);")
        .bind(("expiry", rpc_expiry()))
        .bind(("row", wire::encode_canonical_analyses(&intent).unwrap()))
        .bind(("sources", vec![root(&intent, &revision)]))
        .bind(("inputs", Vec::<Object>::new()))
        .await
        .unwrap();
    response.check().unwrap(); // Server has executed the creation; commit is withheld.
    loop {
        let observed = server_now(&store).await;
        if observed >= intent.creation_expires_at {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await; // Clock polling backoff only.
    }
    let settlement = held_transaction(&options).await;
    let mut response = settlement
        .query("RETURN fn::pse_retention_v1::forget_analysis($expiry,$key);")
        .bind(("expiry", rpc_expiry()))
        .bind(("key", intent.key.clone()))
        .await
        .unwrap();
    response = response.check().unwrap();
    let mut settled = response.take::<Option<Object>>(0).unwrap().unwrap();
    assert!(codec::decode_boolean(codec::required(&mut settled, "complete").unwrap()).unwrap());
    // Both transactions have acknowledged their exact function bodies. Only one
    // may commit: expiry alone does not settle the already-dispatched creation.
    if creation_commits_first {
        creation.commit().await.unwrap();
        assert_conflict(settlement.commit().await.unwrap_err());
        assert!(
            store.analysis(&intent.key).await.is_err(),
            "committed incomplete creation stays unavailable"
        );
        for _ in 0..4 {
            if store.forget_analysis_results(&intent.key).await.unwrap() {
                break;
            }
        }
    } else {
        settlement.commit().await.unwrap();
        assert_conflict(creation.commit().await.unwrap_err());
    }
    assert!(
        store.forget_analysis_results(&intent.key).await.unwrap(),
        "encoded original key resolves a lost final acknowledgement"
    );
    assert!(store.settle_analysis(&intent).await.unwrap());
    assert!(
        store
            .persist_analysis(&intent, std::slice::from_ref(&revision), &[], &[], &[])
            .await
            .is_err()
    );
    let mut residue = bounded_query(store.db.query("RETURN array::concat((SELECT key FROM canonical_analyses WHERE key=$key),(SELECT key FROM canonical_analysis_nodes WHERE analysis=$key),(SELECT key FROM canonical_analysis_edges WHERE analysis=$key),(SELECT key FROM canonical_analysis_inputs WHERE analysis=$key),(SELECT key FROM canonical_roots WHERE owner_kind='analysis' AND owner=$key),(SELECT key FROM canonical_guards WHERE key='analysis:'+$key));").bind(("key",intent.key.clone()))).await.unwrap();
    assert!(residue.take::<Vec<Object>>(0).unwrap().is_empty());
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn canonical_analysis_absent_settlement_commits_before_dispatched_creation() {
    absent_creation_settlement_order(false).await;
}
#[tokio::test]
async fn canonical_analysis_dispatched_creation_commits_before_absent_settlement() {
    absent_creation_settlement_order(true).await;
}

#[tokio::test]
async fn canonical_analysis_held_page_append_and_activation_conflict_with_retirement() {
    let state = std::env::var("PSE_SURREAL_STATE").expect("explicit canonical fixture required");
    let mut options = CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
    options.database = format!(
        "canonical_test_analysis_mutation_{}",
        uuid::Uuid::new_v4().simple()
    );
    let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
    let revision = store
        .edit("mutation-source", None, "mutation-initial", &[])
        .await
        .unwrap();
    for (operation, query) in [
        (
            "read",
            "RETURN fn::pse_analysis_v1::read($expiry,$key,'','nodes');",
        ),
        (
            "append",
            "RETURN fn::pse_analysis_v1::append($expiry,$key,$nodes,[]);",
        ),
        (
            "activate",
            "RETURN fn::pse_analysis_v1::activate($expiry,$key);",
        ),
    ] {
        let mut intent = store
            .new_analysis(
                &revision,
                "held-mutation:v2",
                vec![],
                operation.into(),
                1,
                0,
            )
            .await
            .unwrap();
        let node = AnalysisNode {
            key: format!("{}:node", intent.key),
            analysis: intent.key.clone(),
            semantic: "held-variable".into(),
            kind: "variable".into(),
        };
        seal_analysis_request(
            &mut intent,
            std::slice::from_ref(&revision),
            &[],
            std::slice::from_ref(&node),
            &[],
        );
        if operation == "read" {
            store
                .persist_analysis(
                    &intent,
                    std::slice::from_ref(&revision),
                    &[],
                    std::slice::from_ref(&node),
                    &[],
                )
                .await
                .unwrap();
        } else {
            bounded_query(store.db.query("BEGIN; RETURN fn::pse_analysis_v1::begin($pse_rpc_expires_at,$row,$sources,[]); COMMIT;")
                .bind(("row",wire::encode_canonical_analyses(&intent).unwrap()))
                .bind(("sources",vec![root(&intent,&revision)]))).await.unwrap().check().unwrap();
            if operation == "activate" {
                store
                    .append_analysis(
                        &intent.key,
                        vec![wire::encode_canonical_analysis_nodes(&node).unwrap()],
                        vec![],
                    )
                    .await
                    .unwrap();
            }
        }
        let held = held_transaction(&options).await;
        held.query(query)
            .bind(("expiry", rpc_expiry()))
            .bind(("key", intent.key.clone()))
            .bind((
                "nodes",
                vec![wire::encode_canonical_analysis_nodes(&node).unwrap()],
            ))
            .await
            .unwrap()
            .check()
            .unwrap();
        // Function response proves registration of the exact header before the
        // competing production transaction marks it unavailable and removes roots.
        assert!(!store.forget_analysis_results(&intent.key).await.unwrap());
        assert_conflict(held.commit().await.unwrap_err());
        assert!(store.analysis_node_page(&intent.key, None).await.is_err());
        assert!(
            store
                .append_analysis(&intent.key, vec![], vec![])
                .await
                .is_err()
        );
        assert!(bounded_query(store.db.query("BEGIN; RETURN fn::pse_analysis_v1::activate($pse_rpc_expires_at,$key); COMMIT;").bind(("key",intent.key.clone()))).await.is_err());
    }
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn canonical_analysis_original_request_rejects_changed_payload_and_lineage() {
    let state = std::env::var("PSE_SURREAL_STATE").expect("explicit canonical fixture required");
    let mut options = CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
    options.database = format!(
        "canonical_test_analysis_seal_{}",
        uuid::Uuid::new_v4().simple()
    );
    let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
    let revision = store
        .edit("seal-source", None, "seal-initial", &[])
        .await
        .unwrap();
    let mut intent = store
        .new_analysis(
            &revision,
            "sealed-request:v2",
            vec![1],
            "empty".into(),
            1,
            0,
        )
        .await
        .unwrap();
    let node = AnalysisNode {
        key: format!("{}:node", intent.key),
        analysis: intent.key.clone(),
        semantic: "variable".into(),
        kind: "variable".into(),
    };
    seal_analysis_request(
        &mut intent,
        std::slice::from_ref(&revision),
        &[],
        std::slice::from_ref(&node),
        &[],
    );
    store
        .persist_analysis(
            &intent,
            std::slice::from_ref(&revision),
            &[],
            std::slice::from_ref(&node),
            &[],
        )
        .await
        .unwrap();
    let mut changed = node.clone();
    changed.semantic = "different-variable".into();
    assert!(matches!(
        store
            .persist_analysis(
                &intent,
                std::slice::from_ref(&revision),
                &[],
                std::slice::from_ref(&changed),
                &[]
            )
            .await,
        Err(CanonicalError::OperationReused)
    ));
    let other = store
        .edit("other-seal-source", None, "other-seal-initial", &[])
        .await
        .unwrap();
    assert!(matches!(
        store
            .persist_analysis(
                &intent,
                &[revision.clone(), other.clone()],
                &[],
                std::slice::from_ref(&node),
                &[]
            )
            .await,
        Err(CanonicalError::OperationReused)
    ));
    // Even explicitly resealing different bytes cannot reuse a server occurrence.
    let mut resealed = intent.clone();
    resealed.configuration = vec![2].into();
    seal_analysis_request(
        &mut resealed,
        std::slice::from_ref(&revision),
        &[],
        std::slice::from_ref(&node),
        &[],
    );
    assert!(
        store
            .persist_analysis(
                &resealed,
                std::slice::from_ref(&revision),
                &[],
                std::slice::from_ref(&node),
                &[]
            )
            .await
            .is_err()
    );
    let mut other_lineage = intent.clone();
    seal_analysis_request(
        &mut other_lineage,
        &[revision.clone(), other.clone()],
        &[],
        std::slice::from_ref(&node),
        &[],
    );
    assert!(
        store
            .persist_analysis(
                &other_lineage,
                &[revision.clone(), other],
                &[],
                std::slice::from_ref(&node),
                &[]
            )
            .await
            .is_err()
    );
    assert_eq!(
        store.analysis_node_page(&intent.key, None).await.unwrap(),
        vec![node]
    );
    store.remove_isolated_fixture().await.unwrap();
}
