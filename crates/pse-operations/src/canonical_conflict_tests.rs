// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Held responses establish execution order; explicit commits establish races.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "isolated exact conflict fixtures fail on unexpected responses"
)]
#![allow(
    unsafe_code,
    reason = "isolated owner asserts only its synthetic terminal payloads"
)]
use super::*;
use crate::canonical::{ACQUIRE_PROTECTION, CanonicalOptions};
use crate::canonical_execution::{RunRequest, SeedEligibility, TerminalClass};
use std::time::Duration;
use surrealdb::types::Value;

async fn fixture() -> (CanonicalStore, CanonicalOptions, Revision) {
    let state = std::env::var("PSE_SURREAL_STATE").expect("explicit managed fixture required");
    let mut options = CanonicalOptions::from_state(std::path::Path::new(&state)).unwrap();
    options.database = format!("canonical_test_exact_{}", uuid::Uuid::new_v4().simple());
    let store = crate::testing::canonical_fixture_with_options(&options, true).unwrap();
    let revision = store.edit("source", None, "first", &[]).await.unwrap();
    (store, options, revision)
}
async fn held(
    options: &CanonicalOptions,
) -> surrealdb::method::Transaction<surrealdb::engine::remote::grpc::Client> {
    let address = url::Url::parse(&options.endpoint)
        .unwrap()
        .socket_addrs(|| None)
        .unwrap()[0];
    let db = surrealdb::Surreal::new::<surrealdb::engine::remote::grpc::Grpc>(address)
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
fn conflict(error: surrealdb::Error) {
    assert!(
        matches!(error.query_details(), Some(QueryError::TransactionConflict)),
        "{error}"
    );
}
fn thrown(error: CanonicalError, message: &str) -> Result<(), CanonicalError> {
    let CanonicalError::Refused { code, .. } = error else {
        return Err(error);
    };
    assert_eq!(code.message(), message);
    Ok(())
}
fn expiry() -> i64 {
    i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros(),
    )
    .unwrap()
        + 30_000_000
}
fn body(query: &str) -> String {
    query
        .trim()
        .strip_prefix("BEGIN;")
        .unwrap()
        .trim()
        .strip_suffix("COMMIT;")
        .unwrap()
        .replace("TIMEOUT $pse_rpc_timeout", "TIMEOUT 30s")
}
async fn grant(store: &CanonicalStore, revision: &Revision, window: i64) -> (String, i64) {
    let mut response=bounded_query(store.db.query("RETURN {authority:type::record('canonical_guards','retention:'+$problem).incarnation,now:time::micros()};").bind(("problem",revision.problem.clone()))).await.unwrap();
    let mut row = response.take::<Option<Object>>(0).unwrap().unwrap();
    let authority =
        canonical_codec::decode_string(canonical_codec::required(&mut row, "authority").unwrap())
            .unwrap();
    let deadline = canonical_codec::decode_int(canonical_codec::required(&mut row, "now").unwrap())
        .unwrap()
        + window;
    (
        format!(
            "pse.protection.v2:{authority}:{}:{deadline}",
            uuid::Uuid::new_v4()
        ),
        deadline,
    )
}
async fn floor(store: &CanonicalStore, problem: &str) -> u64 {
    let mut row=bounded_query(store.db.query("RETURN type::record('canonical_guards','retention:'+$problem).protection_closed_through ?? 0dec;").bind(("problem",problem.to_owned()))).await.unwrap();
    canonical_codec::decode_uint(row.take::<Value>(0).unwrap()).unwrap()
}
async fn finish(store: &CanonicalStore, revision: &Revision, key: &str) -> String {
    store
        .begin_run(&RunRequest {
            purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
            admission_study: None,
            admission_point: None,
            key: key.into(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        })
        .await
        .unwrap();
    let fence = store
        .claim_run(
            key,
            &format!("claim-{key}"),
            "worker",
            Duration::from_secs(60),
        )
        .await
        .unwrap();
    store
        .append_result_batch(
            &fence,
            &format!("append-{key}"),
            "values",
            0,
            crate::canonical_execution::ResultPayload::new([42].to_vec()).unwrap(),
            1,
        )
        .await
        .unwrap();
    let closed = store
        .close_result_ingestion(&fence, &format!("close-{key}"))
        .await
        .unwrap();
    let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
    // SAFETY: this fixture owns the exact synthetic payload and partial outcome.
    unsafe {
        store
            .seal_attempt(
                &manifest,
                &format!("seal-{key}"),
                TerminalClass::Partial,
                &[42],
                SeedEligibility::Unavailable,
            )
            .await
    }
    .unwrap();
    fence.attempt().into()
}
fn study(revision: &Revision, key: &str, run: &str) -> crate::canonical_studies::Study {
    crate::canonical_studies::Study {
        physical_revision: None,
        physical_identity: None,
        context: None,
        key: key.into(),
        run: run.into(),
        problem: revision.problem.clone(),
        revision: revision.key.clone(),
        metadata: vec![1].into(),
        interpretation: wire::INTERPRETATION.into(),
        point_count: 1,
        next_ordinal: 0,
        dependency_count: 0,
        installed_dependencies: 0,
        active: false,
        cancelled: false,
        generation: 0,
        terminal: false,
    }
}

#[tokio::test]
async fn exact_terminal_seal_and_unrelated_source_creator_commute_both_orders() {
    for creator_first in [false, true] {
        let (store, options, revision) = fixture().await;
        store
            .begin_run(&RunRequest {
                purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
                admission_study: None,
                admission_point: None,
                key: "sealing".into(),
                revision: revision.clone(),
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        let fence = store
            .claim_run("sealing", "claim-sealing", "owner", Duration::from_secs(60))
            .await
            .unwrap();
        let closed = store
            .close_result_ingestion(&fence, "close-sealing")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        let request = serde_json::to_vec(&(
            fence.run(),
            fence.attempt(),
            fence.generation(),
            &manifest.row().key,
            &manifest.row().digest,
            TerminalClass::Partial,
            crate::canonical_execution::result_payload_digest(&[42]),
            None::<&str>,
            SeedEligibility::Unavailable,
        ))
        .unwrap();
        let sealing = held(&options).await;
        sealing.query("RETURN fn::pse_execution_v1::seal_ack($expiry,$run,$attempt,$authority,'seal-sealing',$request,$manifest,$digest,'partial',$completion,false,NONE);")
            .bind(("expiry", expiry()))
            .bind(("run", fence.run().to_owned()))
            .bind(("attempt", fence.attempt().to_owned()))
            .bind(("authority", canonical_codec::encode_uint(fence.generation()).unwrap()))
            .bind(("request", surrealdb::types::Bytes::from(request)))
            .bind(("manifest", manifest.row().key.clone()))
            .bind(("digest", manifest.row().digest.clone()))
            .bind(("completion", surrealdb::types::Bytes::from(vec![42])))
            .await.unwrap().check().unwrap();
        let creator = held(&options).await;
        creator
            .query(body(RETAIN))
            .bind(("pse_rpc_expires_at", expiry()))
            .bind(("problem", revision.problem.clone()))
            .bind(("revision", revision.key.clone()))
            .bind(("root", "history:unrelated"))
            .bind(("owner_kind", "history"))
            .bind(("owner", "unrelated"))
            .bind(("interpretation", wire::INTERPRETATION))
            .await
            .unwrap()
            .check()
            .unwrap();
        // Removing this attempt's root adds no new retention requirement. A
        // genuine creator elsewhere in the same problem must remain independent.
        if creator_first {
            creator.commit().await.unwrap();
            sealing.commit().await.unwrap();
        } else {
            sealing.commit().await.unwrap();
            creator.commit().await.unwrap();
        }
        let terminal = store
            .canonical_attempt(fence.attempt())
            .await
            .unwrap()
            .unwrap();
        assert!(terminal.terminal());
        assert_eq!(
            terminal.outcome.as_ref().map(|value| value.as_str()),
            Some("partial")
        );
        assert_eq!(terminal.completion.as_ref().unwrap().as_slice(), &[42]);
        let run = store.canonical_run(fence.run()).await.unwrap().unwrap();
        assert_eq!(run.terminal_attempt.as_deref(), Some(fence.attempt()));
        let mut roots = bounded_query(store.db.query("SELECT * FROM ONLY type::record('canonical_roots',$active); SELECT * FROM ONLY type::record('canonical_roots','history:unrelated');")
            .bind(("active", format!("active_attempt:{}",fence.attempt())))).await.unwrap();
        assert!(roots.take::<Option<Object>>(0).unwrap().is_none());
        assert!(roots.take::<Option<Object>>(1).unwrap().is_some());
        // SAFETY: this fixture owns the frozen manifest and synthetic partial
        // completion; replay uses the same terminal class and exposes no seeds.
        unsafe {
            store
                .seal_attempt(
                    &manifest,
                    "seal-sealing",
                    TerminalClass::Partial,
                    &[42],
                    SeedEligibility::Unavailable,
                )
                .await
        }
        .unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
}

#[tokio::test]
async fn exact_terminal_root_release_and_source_collection_preserve_run_both_orders() {
    for collector_first in [false, true] {
        let (store, options, first) = fixture().await;
        let edit = |version: &str| crate::canonical::ObjectEdit {
            logical: "x".into(),
            scope: "root".into(),
            name: "x".into(),
            version: Some(crate::canonical::ObjectVersion {
                key: version.into(),
                logical: "x".into(),
                kind: "test".into(),
                payload: vec![1].into(),
                interpretation: wire::INTERPRETATION.into(),
            }),
            references: vec![],
        };
        let old = store
            .edit("source", Some(&first.key), "second", &[edit("v1")])
            .await
            .unwrap();
        store
            .edit("source", Some(&old.key), "third", &[edit("v2")])
            .await
            .unwrap();
        store
            .begin_run(&RunRequest {
                purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
                admission_study: None,
                admission_point: None,
                key: "sealing".into(),
                revision: old.clone(),
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        store.forget_history(&old).await.unwrap();
        let fence = store
            .claim_run("sealing", "claim-sealing", "owner", Duration::from_secs(60))
            .await
            .unwrap();
        let closed = store
            .close_result_ingestion(&fence, "close-sealing")
            .await
            .unwrap();
        let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
        let request = serde_json::to_vec(&(
            fence.run(),
            fence.attempt(),
            fence.generation(),
            &manifest.row().key,
            &manifest.row().digest,
            TerminalClass::Partial,
            crate::canonical_execution::result_payload_digest(&[42]),
            None::<&str>,
            SeedEligibility::Unavailable,
        ))
        .unwrap();
        let sealing = held(&options).await;
        sealing.query("RETURN fn::pse_execution_v1::seal_ack($expiry,$run,$attempt,$authority,'seal-sealing',$request,$manifest,$digest,'partial',$completion,false,NONE);")
            .bind(("expiry",expiry())).bind(("run",fence.run().to_owned()))
            .bind(("attempt",fence.attempt().to_owned()))
            .bind(("authority",canonical_codec::encode_uint(fence.generation()).unwrap()))
            .bind(("request",surrealdb::types::Bytes::from(request)))
            .bind(("manifest",manifest.row().key.clone())).bind(("digest",manifest.row().digest.clone()))
            .bind(("completion",surrealdb::types::Bytes::from(vec![42])))
            .await.unwrap().check().unwrap();
        let collector = held(&options).await;
        let mut response = collector
            .query("RETURN fn::pse_retention_v1::source_step($pse_rpc_expires_at,$problem);")
            .bind(("pse_rpc_expires_at", expiry()))
            .bind(("problem", old.problem.clone()))
            .await
            .unwrap()
            .check()
            .unwrap();
        let result = response
            .take::<Option<Object>>(response.num_statements() - 1)
            .unwrap()
            .unwrap();
        assert_eq!(result.get("membership"), Some(&Value::Bool(false)));
        assert_eq!(result.get("version"), Some(&Value::Bool(false)));
        if collector_first {
            collector.commit().await.unwrap();
            conflict(sealing.commit().await.unwrap_err());
        } else {
            sealing.commit().await.unwrap();
            if let Err(error) = collector.commit().await {
                // SSI also tracks the root read by this collector. Removing the
                // active-attempt root may abort its old decision even though the
                // independent run root still preserves the selected source.
                conflict(error);
            }
        }
        // A definite conflict rebuilds the same seal, never another scientific
        // attempt. A completed seal's same-identity replay is also unchanged.
        // SAFETY: the fixture owns the exact frozen observations and synthetic
        // partial completion in either commit order, with no seed eligibility.
        unsafe {
            store
                .seal_attempt(
                    &manifest,
                    "seal-sealing",
                    TerminalClass::Partial,
                    &[42],
                    SeedEligibility::Unavailable,
                )
                .await
        }
        .unwrap();
        // Rebuild every collector premise through the production path after a
        // definite SSI abort; a committed collector receives the same recheck.
        let page = store.reclaim_source_page(&old.problem).await.unwrap();
        assert!(page.examined <= PAGE);
        assert_eq!((page.memberships, page.versions), (0, 0));
        // A committed held collector leaves the cursor after the sole interval.
        // Its empty tail resets the pass before a later call revisits that hold.
        let next_pass = store.reclaim_source_page(&old.problem).await.unwrap();
        assert!(next_pass.examined <= PAGE);
        assert!(page.examined + next_pass.examined > 0);
        assert_eq!((next_pass.memberships, next_pass.versions), (0, 0));
        let pin = store.protect(old, Duration::from_secs(60)).await.unwrap();
        let members = store.membership_page(&pin, None, "").await.unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].version, "v1");
        store.release(&pin).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
}

#[tokio::test]
async fn exact_scoped_head_page_and_source_activation_both_commit_orders() {
    for reader_first in [false, true] {
        let (store, options, first) = fixture().await;
        let edit = |name: &str, scope: &str, version: &str| crate::canonical::ObjectEdit {
            logical: name.into(),
            scope: scope.into(),
            name: name.into(),
            version: Some(crate::canonical::ObjectVersion {
                key: version.into(),
                logical: name.into(),
                kind: "test".into(),
                payload: vec![1].into(),
                interpretation: wire::INTERPRETATION.into(),
            }),
            references: vec![],
        };
        let revision = store
            .edit(
                "source",
                Some(&first.key),
                "second",
                &[edit("x", "root", "v1")],
            )
            .await
            .unwrap();
        let pin = store
            .protect(revision.clone(), Duration::from_secs(60))
            .await
            .unwrap();
        // The writer changes only another scope. This page's membership rows
        // remain untouched: the registered head is the shared changing premise.
        let stage = match store
            .stage_edits(
                "source",
                Some(&revision.key),
                "third",
                &[edit("y", "other", "v2")],
            )
            .await
            .unwrap()
        {
            crate::canonical_staging::StageOutcome::Closed(stage) => Some(stage),
            crate::canonical_staging::StageOutcome::Acknowledged(_) => None,
        }
        .expect("fresh operation must close a stage");
        let reader = held(&options).await;
        let mut response = reader
            .query(body(&crate::canonical_selection::test_membership_page_sql()))
            .bind(("pse_rpc_expires_at", expiry()))
            .bind(("problem", revision.problem.clone()))
            .bind(("revision", revision.key.clone()))
            .bind((
                "sequence",
                canonical_codec::encode_uint(revision.sequence).unwrap(),
            ))
            .bind(("protection", pin.key().to_owned()))
            .bind(("scope", "root"))
            .bind(("after", ""))
            .await
            .unwrap()
            .check()
            .unwrap();
        let page: Vec<Object> = response
            .take(response.num_statements().saturating_sub(1))
            .unwrap();
        assert_eq!(page.len(), 1);
        assert_eq!(
            wire::decode_canonical_memberships(page.into_iter().next().unwrap())
                .unwrap()
                .version,
            "v1"
        );
        let writer = held(&options).await;
        writer
            .query(body(&crate::canonical::test_source_edit_sql()))
            .bind(("pse_rpc_expires_at", expiry()))
            .bind(("problem", stage.problem.clone()))
            .bind(("expected", stage.expected.clone()))
            .bind(("operation", stage.operation.clone()))
            .bind(("interpretation", wire::INTERPRETATION))
            .bind((
                "request",
                surrealdb::types::Bytes::from(stage.request.clone()),
            ))
            .bind(("request_digest", stage.request_digest.clone()))
            .bind((
                "generation",
                canonical_codec::encode_uint(stage.generation).unwrap(),
            ))
            .await
            .unwrap()
            .check()
            .unwrap();
        if reader_first {
            reader.commit().await.unwrap();
            writer.commit().await.unwrap();
        } else {
            writer.commit().await.unwrap();
            conflict(reader.commit().await.unwrap_err());
        }
        // Rebuild the actual page under a fresh transaction after the conflict:
        // the original selection uses historical intervals, never the new head.
        let historical = store.membership_page(&pin, Some("root"), "").await.unwrap();
        assert_eq!(historical.len(), 1);
        assert_eq!(historical[0].version, "v1");
        assert!(
            store
                .membership_page(&pin, Some("other"), "")
                .await
                .unwrap()
                .is_empty()
        );
        let current = store.revision("third").await.unwrap().unwrap();
        let fresh = store
            .protect(current, Duration::from_secs(60))
            .await
            .unwrap();
        let new_scope = store
            .membership_page(&fresh, Some("other"), "")
            .await
            .unwrap();
        assert_eq!(new_scope.len(), 1);
        assert_eq!(new_scope[0].version, "v2");
        store.release(&pin).await.unwrap();
        store.release(&fresh).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
}

#[tokio::test]
async fn exact_source_root_and_pin_creation_reclaim_both_orders() {
    for pin in [false, true] {
        for creator_first in [false, true] {
            let (store, options, _) = fixture().await;
            let edit = |key: &str| crate::canonical::ObjectEdit {
                logical: "x".into(),
                scope: "root".into(),
                name: "x".into(),
                version: Some(crate::canonical::ObjectVersion {
                    key: key.into(),
                    logical: "x".into(),
                    kind: "test".into(),
                    payload: vec![1].into(),
                    interpretation: wire::INTERPRETATION.into(),
                }),
                references: vec![],
            };
            let old = store
                .edit("source", Some("first"), "second", &[edit("v1")])
                .await
                .unwrap();
            store
                .edit("source", Some("second"), "third", &[edit("v2")])
                .await
                .unwrap();
            // Withdraw only the root so the held native transactions below,
            // rather than foreground cleanup, determine the collector's commit.
            bounded_query(
                store
                    .db
                    .query(DROP_ROOT)
                    .bind(("problem", old.problem.clone()))
                    .bind(("revision", old.key.clone()))
                    .bind(("root", format!("history:{}", old.key)))
                    .bind(("owner_kind", "history"))
                    .bind(("owner", old.key.clone())),
            )
            .await
            .unwrap();
            let creator = held(&options).await;
            let (key, _) = grant(&store, &old, 30_000_000).await;
            if pin {
                creator
                    .query(body(ACQUIRE_PROTECTION))
                    .bind(("pse_rpc_expires_at", expiry()))
                    .bind(("problem", old.problem.clone()))
                    .bind(("revision", old.key.clone()))
                    .bind(("key", key.clone()))
                    .bind(("lifetime", 60_000_000i64))
                    .await
                    .unwrap()
                    .check()
                    .unwrap();
            } else {
                creator
                    .query(body(RETAIN))
                    .bind(("pse_rpc_expires_at", expiry()))
                    .bind(("problem", old.problem.clone()))
                    .bind(("revision", old.key.clone()))
                    .bind(("root", "history:late"))
                    .bind(("owner_kind", "history"))
                    .bind(("owner", "late"))
                    .bind(("interpretation", wire::INTERPRETATION))
                    .await
                    .unwrap()
                    .check()
                    .unwrap();
            }
            let collector = held(&options).await;
            collector
                .query("RETURN fn::pse_retention_v1::source_step($pse_rpc_expires_at,$problem);")
                .bind(("pse_rpc_expires_at", expiry()))
                .bind(("problem", old.problem.clone()))
                .await
                .unwrap()
                .check()
                .unwrap();
            if creator_first {
                creator.commit().await.unwrap();
                conflict(collector.commit().await.unwrap_err());
            } else {
                collector.commit().await.unwrap();
                conflict(creator.commit().await.unwrap_err());
                assert!(
                    store
                        .retain_revision(&old, &RetentionOwner::History("late".into()))
                        .await
                        .is_err()
                );
            }
            store.remove_isolated_fixture().await.unwrap();
        }
    }
}

#[tokio::test]
async fn exact_protection_absent_settlement_both_orders_and_replay() {
    for creator_first in [false, true] {
        let (store, options, revision) = fixture().await;
        assert_eq!(floor(&store, &revision.problem).await, 0);
        let creator = held(&options).await;
        // Issue and acquire the short-lived grant on the already-open session:
        // client setup and response latency are not its acquisition subject.
        let acquisition = format!(
            "LET $fixture_deadline=time::micros()+500000; \
             LET $key='pse.protection.v2:'+type::record('canonical_guards','retention:'+$problem).incarnation+':'+$nonce+':'+<string>$fixture_deadline; \
             {}",
            body(ACQUIRE_PROTECTION).replace(
                "RETURN $rpc_result;",
                "RETURN {key:$key,deadline:$fixture_deadline};",
            )
        );
        let mut response = creator
            .query(acquisition)
            .bind(("pse_rpc_expires_at", expiry()))
            .bind(("problem", revision.problem.clone()))
            .bind(("revision", revision.key.clone()))
            .bind(("nonce", uuid::Uuid::new_v4().to_string()))
            .bind(("lifetime", 100_000i64))
            .await
            .unwrap()
            .check()
            .unwrap();
        let mut issued = response
            .take::<Option<Object>>(response.num_statements() - 1)
            .unwrap()
            .unwrap();
        let key =
            canonical_codec::decode_string(canonical_codec::required(&mut issued, "key").unwrap())
                .unwrap();
        let deadline = canonical_codec::decode_int(
            canonical_codec::required(&mut issued, "deadline").unwrap(),
        )
        .unwrap();
        while expiry() - 30_000_000 < deadline {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let settlement = held(&options).await;
        settlement
            .query("RETURN fn::pse_retention_v1::settle_protection($expiry,$problem,$key);")
            .bind(("expiry", expiry()))
            .bind(("problem", revision.problem.clone()))
            .bind(("key", key.clone()))
            .await
            .unwrap()
            .check()
            .unwrap();
        if creator_first {
            creator.commit().await.unwrap();
            conflict(settlement.commit().await.unwrap_err());
        } else {
            settlement.commit().await.unwrap();
            conflict(creator.commit().await.unwrap_err());
        }
        assert!(
            store
                .settle_protection_grant(&revision.problem, &key)
                .await
                .unwrap()
        );
        assert!(
            store
                .settle_protection_grant(&revision.problem, &key)
                .await
                .unwrap()
        ); // lost final receipt settles exactly
        assert_eq!(
            floor(&store, &revision.problem).await,
            u64::try_from(deadline).unwrap()
        );
        let parts = key.split(':').collect::<Vec<_>>();
        let earlier = format!(
            "pse.protection.v2:{}:{}:{}",
            parts[1],
            uuid::Uuid::new_v4(),
            deadline - 1
        );
        assert!(
            store
                .settle_protection_grant(&revision.problem, &earlier)
                .await
                .unwrap()
        );
        assert_eq!(
            floor(&store, &revision.problem).await,
            u64::try_from(deadline).unwrap(),
            "settling an older window cannot clear or reduce the floor"
        );
        assert!(
            bounded_query(
                store
                    .db
                    .query(ACQUIRE_PROTECTION)
                    .bind(("problem", revision.problem.clone()))
                    .bind(("revision", revision.key.clone()))
                    .bind(("key", key.clone()))
                    .bind(("lifetime", 60_000_000i64))
            )
            .await
            .is_err()
        );
        let wrong = key.replacen("pse.protection.v2:", "pse.protection.v2:restored-", 1);
        assert!(
            store
                .settle_protection_grant(&revision.problem, &wrong)
                .await
                .is_err()
        );
        let mut residue = bounded_query(
            store
                .db
                .query("SELECT key FROM ONLY type::record('canonical_protections',$key);")
                .bind(("key", key)),
        )
        .await
        .unwrap();
        assert!(residue.take::<Option<Object>>(0).unwrap().is_none());
        store.remove_isolated_fixture().await.unwrap();
    }
}

#[tokio::test]
async fn exact_terminal_summary_and_point_creators_retirement_both_orders() {
    for point in [false, true] {
        for creator_first in [false, true] {
            let (store, options, revision) = fixture().await;
            finish(&store, &revision, "target").await;
            if point {
                finish(&store, &revision, "parent").await;
                bounded_query(
                    store
                        .db
                        .query("RETURN fn::pse_study_v1::create($pse_rpc_expires_at,$row);")
                        .bind((
                            "row",
                            wire::encode_canonical_studies(&study(&revision, "study", "parent"))
                                .unwrap(),
                        )),
                )
                .await
                .unwrap();
            }
            let creator = held(&options).await;
            if point {
                let row = crate::canonical_studies::StudyPoint {
                    key: "point".into(),
                    study: "study".into(),
                    admission_run: None,
                    variant_problem: None,
                    variant_operation: None,
                    ordinal: 0,
                    occurrence: 0,
                    seed_solution: None,
                    policy: vec![1].into(),
                    descriptor: vec![2].into(),
                    // This fixture already owns its exact run/source; no lazy input admission has occurred.
                    admission: None,
                    facts: vec![3].into(),
                    outcome: None,
                    run: "target".into(),
                    revision: 0,
                    assigned: false,
                    settled: false,
                    attempt: None,
                    start: None,
                };
                creator
                    .query("RETURN fn::pse_study_v1::append($expiry,'study',$points);")
                    .bind(("expiry", expiry()))
                    .bind((
                        "points",
                        vec![wire::encode_canonical_study_points(&row).unwrap()],
                    ))
                    .await
                    .unwrap()
                    .check()
                    .unwrap();
            } else {
                creator
                    .query("RETURN fn::pse_study_v1::create($expiry,$row);")
                    .bind(("expiry", expiry()))
                    .bind((
                        "row",
                        wire::encode_canonical_studies(&study(&revision, "study", "target"))
                            .unwrap(),
                    ))
                    .await
                    .unwrap()
                    .check()
                    .unwrap();
            }
            let retirement = held(&options).await;
            retirement
                .query("RETURN fn::pse_retention_v1::forget_run($expiry,'target');")
                .bind(("expiry", expiry()))
                .await
                .unwrap()
                .check()
                .unwrap();
            if creator_first {
                creator.commit().await.unwrap();
                conflict(retirement.commit().await.unwrap_err());
                assert!(store.forget_run_results("target").await.is_err());
            } else {
                retirement.commit().await.unwrap();
                conflict(creator.commit().await.unwrap_err());
                assert!(
                    bounded_query(store.db.query(
                        "RETURN fn::pse_execution_v1::result_creator($pse_rpc_expires_at,'target');"
                    ))
                    .await
                    .is_err()
                );
            }
            store.remove_isolated_fixture().await.unwrap();
        }
    }
}

#[tokio::test]
async fn exact_cancellation_registers_dispatch_both_orders() {
    for cancellation_first in [false, true] {
        let (store, options, revision) = fixture().await;
        store
            .begin_run(&RunRequest {
                purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
                admission_study: None,
                admission_point: None,
                key: "target".into(),
                revision: revision.clone(),
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
        bounded_query(
            store
                .db
                .query("RETURN fn::pse_study_v1::create($pse_rpc_expires_at,$row);")
                .bind((
                    "row",
                    wire::encode_canonical_studies(&study(&revision, "study", "target")).unwrap(),
                )),
        )
        .await
        .unwrap();
        let fence = store
            .claim_run("target", "claim", "worker", Duration::from_secs(60))
            .await
            .unwrap();
        let writer = held(&options).await;
        writer.query("RETURN fn::pse_execution_v1::writer($expiry,'target',$attempt,$generation,'dispatch','dispatch');").bind(("expiry",expiry())).bind(("attempt",fence.attempt().to_owned())).bind(("generation",canonical_codec::encode_uint(fence.generation()).unwrap())).await.unwrap().check().unwrap();
        let cancel = held(&options).await;
        cancel
            .query("RETURN fn::pse_study_v1::cancel($expiry,'study');")
            .bind(("expiry", expiry()))
            .await
            .unwrap()
            .check()
            .unwrap();
        if cancellation_first {
            cancel.commit().await.unwrap();
            conflict(writer.commit().await.unwrap_err());
        } else {
            writer.commit().await.unwrap();
            cancel.commit().await.unwrap();
        }
        let mut result=bounded_query(store.db.query("RETURN fn::pse_execution_v1::writer($pse_rpc_expires_at,'target',$attempt,$generation,'dispatch','dispatch');").bind(("attempt",fence.attempt().to_owned())).bind(("generation",canonical_codec::encode_uint(fence.generation()).unwrap()))).await.unwrap();
        let row = result.take::<Option<Object>>(0).unwrap().unwrap();
        assert_eq!(
            row.get("pse_writer"),
            Some(&Value::String("study-cancelled".into()))
        );
        store.remove_isolated_fixture().await.unwrap();
    }
}

fn point(
    key: &str,
    run: &str,
    ordinal: u64,
    settled: bool,
) -> crate::canonical_studies::StudyPoint {
    crate::canonical_studies::StudyPoint {
        key: key.into(),
        study: "study".into(),
        admission_run: None,
        variant_problem: None,
        variant_operation: None,
        ordinal,
        occurrence: ordinal,
        seed_solution: None,
        policy: vec![1].into(),
        descriptor: vec![2].into(),
        admission: None,
        facts: vec![3].into(),
        outcome: None,
        run: run.into(),
        revision: 0,
        assigned: false,
        settled,
        attempt: None,
        start: None,
    }
}
async fn begin(store: &CanonicalStore, revision: &Revision, key: &str) {
    store
        .begin_run(&RunRequest {
            purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
            admission_study: None,
            admission_point: None,
            key: key.into(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        })
        .await
        .unwrap();
}
async fn install_study(
    store: &CanonicalStore,
    revision: &Revision,
    points: &[crate::canonical_studies::StudyPoint],
) {
    begin(store, revision, "summary").await;
    let mut row = study(revision, "study", "summary");
    row.point_count = points.len() as u64;
    bounded_query(
        store
            .db
            .query("RETURN fn::pse_study_v1::create($pse_rpc_expires_at,$row);")
            .bind(("row", wire::encode_canonical_studies(&row).unwrap())),
    )
    .await
    .unwrap();
    // Install every endpoint page before any dependency phase. These independent
    // point fixtures declare no edges, so zero installed dependencies completes
    // that phase without issuing an invalid empty append_dependencies request.
    assert_eq!(row.dependency_count, 0);
    for page in points.chunks(64) {
        bounded_query(
            store
                .db
                .query("RETURN fn::pse_study_v1::append($pse_rpc_expires_at,'study',$points);")
                .bind((
                    "points",
                    page.iter()
                        .map(wire::encode_canonical_study_points)
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap(),
                )),
        )
        .await
        .unwrap();
    }
    bounded_query(
        store
            .db
            .query("RETURN fn::pse_study_v1::activate($pse_rpc_expires_at,'study');"),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn exact_cancelled_study_finalizer_has_only_summary_publication_authority() {
    let (store, _, revision) = fixture().await;
    finish(&store, &revision, "point-run").await;
    install_study(&store, &revision, &[point("point", "point-run", 0, true)]).await;
    let cancelled = store.cancel_study("study").await.unwrap();
    assert!(
        store
            .claim_run("summary", "ordinary", "worker", Duration::from_secs(60))
            .await
            .is_err()
    );
    let fence = store
        .begin_study_finalization(
            &crate::canonical_studies::StudySummary::from(&cancelled),
            "summary-worker",
            Duration::from_secs(60),
        )
        .await
        .unwrap()
        .unwrap();
    store
        .append_result_batch(
            &fence,
            "summary-append",
            "values",
            0,
            crate::canonical_execution::ResultPayload::new([42].to_vec()).unwrap(),
            1,
        )
        .await
        .unwrap();
    store
        .renew_attempt(&fence, Duration::from_secs(60))
        .await
        .unwrap();
    for kind in ["dispatch", "seed"] {
        assert!(bounded_query(store.db.query("RETURN fn::pse_execution_v1::writer($pse_rpc_expires_at,'summary',$attempt,$generation,$kind,$kind);").bind(("attempt",fence.attempt().to_owned())).bind(("generation",canonical_codec::encode_uint(fence.generation()).unwrap())).bind(("kind",kind))).await.is_err());
    }
    let closed = store
        .close_result_ingestion(&fence, "summary-close")
        .await
        .unwrap();
    let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
    // SAFETY: the fixture owns the settled synthetic occurrence and this exact summary.
    let receipt = unsafe {
        store
            .seal_study_summary(
                "study",
                &manifest,
                "summary-seal",
                TerminalClass::Cancelled,
                b"settled original occurrence",
            )
            .await
    }
    .unwrap();
    assert_eq!(
        receipt.outcome.as_ref().map(|value| value.as_str()),
        Some("cancelled")
    );
    assert!(receipt.seed_manifest.is_none());
    assert!(
        store
            .canonical_study("study")
            .await
            .unwrap()
            .unwrap()
            .terminal
    );
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn exact_disjoint_point_writers_share_unchanged_cancellation_authority() {
    let (store, options, revision) = fixture().await;
    begin(&store, &revision, "a").await;
    begin(&store, &revision, "b").await;
    install_study(
        &store,
        &revision,
        &[point("a", "a", 0, false), point("b", "b", 1, false)],
    )
    .await;
    let a = store
        .claim_run("a", "claim-a", "worker", Duration::from_secs(60))
        .await
        .unwrap();
    let b = store
        .claim_run("b", "claim-b", "worker", Duration::from_secs(60))
        .await
        .unwrap();
    let first = held(&options).await;
    let second = held(&options).await;
    for (transaction, fence) in [(&first, &a), (&second, &b)] {
        transaction.query("RETURN fn::pse_execution_v1::writer($expiry,$run,$attempt,$generation,'dispatch','dispatch');").bind(("expiry",expiry())).bind(("run",fence.run().to_owned())).bind(("attempt",fence.attempt().to_owned())).bind(("generation",canonical_codec::encode_uint(fence.generation()).unwrap())).await.unwrap().check().unwrap();
    }
    first.commit().await.unwrap();
    second.commit().await.unwrap();
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn exact_result_pin_creation_retirement_both_orders() {
    for creator_first in [false, true] {
        let (store, options, revision) = fixture().await;
        let attempt = finish(&store, &revision, "target").await;
        let manifest = store
            .canonical_attempt(&attempt)
            .await
            .unwrap()
            .unwrap()
            .closed_manifest
            .unwrap();
        let pin = store
            .protect(revision.clone(), Duration::from_secs(60))
            .await
            .unwrap();
        let creator = held(&options).await;
        creator
            .query(format!(
                "{}\n{}",
                crate::canonical::PROTECTED_BEGIN
                    .trim()
                    .strip_prefix("BEGIN;")
                    .unwrap(),
                crate::canonical_results::RESULT_ADMISSION
            ))
            .bind(("pse_rpc_expires_at", expiry()))
            .bind(("problem", revision.problem.clone()))
            .bind(("revision", revision.key.clone()))
            .bind((
                "sequence",
                canonical_codec::encode_uint(revision.sequence).unwrap(),
            ))
            .bind(("protection", pin.key().to_owned()))
            .bind(("run", "target"))
            .bind(("attempt", attempt))
            .bind(("manifest", manifest))
            .await
            .unwrap()
            .check()
            .unwrap();
        let retirement = held(&options).await;
        retirement
            .query("RETURN fn::pse_retention_v1::forget_run($expiry,'target');")
            .bind(("expiry", expiry()))
            .await
            .unwrap()
            .check()
            .unwrap();
        if creator_first {
            creator.commit().await.unwrap();
            conflict(retirement.commit().await.unwrap_err());
            assert!(store.forget_run_results("target").await.is_err());
        } else {
            retirement.commit().await.unwrap();
            conflict(creator.commit().await.unwrap_err());
        }
        store.release(&pin).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
}

#[tokio::test]
async fn exact_dispatched_pre_ticket_renewal_and_expired_admission_retirement_both_orders() {
    for renewal_first in [false, true] {
        let (store, options, revision) = fixture().await;
        let attempt = finish(&store, &revision, "target").await;
        let manifest = store
            .canonical_attempt(&attempt)
            .await
            .unwrap()
            .unwrap()
            .closed_manifest
            .unwrap();
        let control_deadline = original_deadline(crate::canonical::REQUEST_TIMEOUT);
        // read_results and read_completion commit this source-only admission
        // before admitting their strong manifest ticket. No public reader exists yet.
        let pin = store
            .protect(revision.clone(), Duration::from_secs(5))
            .await
            .unwrap();
        bounded_query(
            store
                .db
                .query(format!(
                    "{}\n{}\nCOMMIT;",
                    crate::canonical::PROTECTED_BEGIN,
                    crate::canonical_results::RESULT_ADMISSION
                ))
                .bind(("problem", revision.problem.clone()))
                .bind(("revision", revision.key.clone()))
                .bind((
                    "sequence",
                    canonical_codec::encode_uint(revision.sequence).unwrap(),
                ))
                .bind(("protection", pin.key().to_owned()))
                .bind(("run", "target"))
                .bind(("attempt", attempt.clone()))
                .bind(("manifest", manifest)),
        )
        .await
        .unwrap();
        let renewal = held(&options).await;
        renewal
            .query(format!(
                "{}\n{}",
                crate::canonical::PROTECTED_BEGIN
                    .trim()
                    .strip_prefix("BEGIN;")
                    .unwrap(),
                crate::canonical_results::RENEW_PROTECTION
            ))
            .bind(("pse_rpc_expires_at", expiry()))
            .bind(("problem", revision.problem.clone()))
            .bind(("revision", revision.key.clone()))
            .bind((
                "sequence",
                canonical_codec::encode_uint(revision.sequence).unwrap(),
            ))
            .bind(("protection", pin.key().to_owned()))
            .bind(("lifetime", 60_000_000i64))
            .await
            .unwrap()
            .check()
            .unwrap();
        // Observe the exact source deadline instead of assuming the public
        // reader's lifetime or treating an open manifest ticket as expired.
        within_clock(control_deadline, async {
        loop {
            let mut clock = bounded_query(store.db.query("RETURN {now:time::micros(),expiry:type::record('canonical_protections',$key).expires_at};")
                .bind(("key", pin.key().to_owned()))).await.unwrap();
            let mut clock = clock.take::<Option<Object>>(0).unwrap().unwrap();
            let now = canonical_codec::decode_int(canonical_codec::required(&mut clock, "now").unwrap()).unwrap();
            let expiry = canonical_codec::decode_int(canonical_codec::required(&mut clock, "expiry").unwrap()).unwrap();
            if now >= expiry { break; }
            tokio::time::sleep(Duration::from_micros((expiry - now).unsigned_abs().min(250_000))).await;
        }
        Ok(())
        }).await.unwrap();
        let retirement = held(&options).await;
        retirement
            .query("RETURN fn::pse_retention_v1::forget_run($expiry,'target');")
            .bind(("expiry", expiry()))
            .await
            .unwrap()
            .check()
            .unwrap();
        if renewal_first {
            renewal.commit().await.unwrap();
            conflict(retirement.commit().await.unwrap_err());
            thrown(
                store.forget_run_results("target").await.unwrap_err(),
                "run results have protected readers",
            )
            .unwrap();
        } else {
            retirement.commit().await.unwrap();
            conflict(renewal.commit().await.unwrap_err());
            // The same protected descriptor query used by the reader must now
            // reject this retired pre-ticket admission before exposing any row.
            let error = bounded_query(store.db.metadata_query(crate::canonical::MetadataResponse::Page,
                format!("{}\nRETURN SELECT * FROM canonical_result_batches WHERE result_set=$set AND ordinal IN $selected ORDER BY ordinal LIMIT 64; COMMIT;", crate::canonical::PROTECTED_BEGIN))
                .bind(("problem", revision.problem.clone()))
                .bind(("revision", revision.key.clone()))
                .bind(("sequence", canonical_codec::encode_uint(revision.sequence).unwrap()))
                .bind(("protection", pin.key().to_owned()))
                .bind(("set", crate::canonical_execution::result_set_key(&attempt, "values")))
                .bind(("selected", vec![canonical_codec::encode_uint(0).unwrap()]))).await.unwrap_err();
            thrown(error, "immutable selection protection expired").unwrap();
        }
        store.release(&pin).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
}

#[tokio::test]
async fn exact_dispatched_source_renewal_and_expired_pin_reclaim_both_orders() {
    for renewal_first in [false, true] {
        let (store, options, _) = fixture().await;
        let edit = |key: &str| crate::canonical::ObjectEdit {
            logical: "x".into(),
            scope: "root".into(),
            name: "x".into(),
            version: Some(crate::canonical::ObjectVersion {
                key: key.into(),
                logical: "x".into(),
                kind: "test".into(),
                payload: vec![1].into(),
                interpretation: wire::INTERPRETATION.into(),
            }),
            references: vec![],
        };
        let revision = store
            .edit("source", Some("first"), "second", &[edit("v1")])
            .await
            .unwrap();
        store
            .edit("source", Some("second"), "third", &[edit("v2")])
            .await
            .unwrap();
        let pin = store
            .protect(revision.clone(), Duration::from_secs(2))
            .await
            .unwrap();
        store.forget_history(&revision).await.unwrap();
        let renewal = held(&options).await;
        renewal
            .query(format!(
                "{}\n{}",
                crate::canonical::PROTECTED_BEGIN
                    .trim()
                    .strip_prefix("BEGIN;")
                    .unwrap(),
                crate::canonical_results::RENEW_PROTECTION
            ))
            .bind(("pse_rpc_expires_at", expiry()))
            .bind(("problem", revision.problem.clone()))
            .bind(("revision", revision.key.clone()))
            .bind((
                "sequence",
                canonical_codec::encode_uint(revision.sequence).unwrap(),
            ))
            .bind(("protection", pin.key().to_owned()))
            .bind(("lifetime", 60_000_000i64))
            .await
            .unwrap()
            .check()
            .unwrap();
        tokio::time::sleep(Duration::from_millis(2100)).await;
        let collector = held(&options).await;
        collector
            .query("RETURN fn::pse_retention_v1::source_step($pse_rpc_expires_at,$problem);")
            .bind(("pse_rpc_expires_at", expiry()))
            .bind(("problem", revision.problem.clone()))
            .await
            .unwrap()
            .check()
            .unwrap();
        if renewal_first {
            renewal.commit().await.unwrap();
            conflict(collector.commit().await.unwrap_err());
            assert!(store.object("v1").await.unwrap().is_some());
        } else {
            collector.commit().await.unwrap();
            conflict(renewal.commit().await.unwrap_err());
            assert!(
                store
                    .select_names(&pin, "root", &["x".into()])
                    .await
                    .is_err()
            );
            assert!(store.object("v1").await.unwrap().is_none());
        }
        store.release(&pin).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
}

#[tokio::test]
async fn exact_released_tombstone_survives_original_window_then_collects_without_recreation() {
    let (store, _, revision) = fixture().await;
    let (key, deadline) = grant(&store, &revision, 500_000).await;
    bounded_query(
        store
            .db
            .query(crate::canonical::RELEASE_PROTECTION)
            .bind(("problem", revision.problem.clone()))
            .bind(("revision", revision.key.clone()))
            .bind((
                "sequence",
                canonical_codec::encode_uint(revision.sequence).unwrap(),
            ))
            .bind(("protection", key.clone())),
    )
    .await
    .unwrap();
    assert!(
        !store
            .settle_protection_grant(&revision.problem, &key)
            .await
            .unwrap()
    );
    let mut before = bounded_query(
        store
            .db
            .query("SELECT * FROM ONLY type::record('canonical_protections',$key);")
            .bind(("key", key.clone())),
    )
    .await
    .unwrap();
    let before = before.take::<Option<Object>>(0).unwrap().unwrap();
    assert_eq!(before.get("released"), Some(&Value::Bool(true)));
    assert_eq!(floor(&store, &revision.problem).await, 0);
    while expiry() - 30_000_000 < deadline {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(
        store
            .settle_protection_grant(&revision.problem, &key)
            .await
            .unwrap()
    );
    // A delayed release acknowledgment uses the original key; the closed floor
    // prevents it from recreating physical grant residue after settlement.
    bounded_query(
        store
            .db
            .query(crate::canonical::RELEASE_PROTECTION)
            .bind(("problem", revision.problem.clone()))
            .bind(("revision", revision.key.clone()))
            .bind((
                "sequence",
                canonical_codec::encode_uint(revision.sequence).unwrap(),
            ))
            .bind(("protection", key.clone())),
    )
    .await
    .unwrap();
    let mut after = bounded_query(
        store
            .db
            .query("SELECT * FROM ONLY type::record('canonical_protections',$key);")
            .bind(("key", key.clone())),
    )
    .await
    .unwrap();
    assert!(after.take::<Option<Object>>(0).unwrap().is_none());
    assert_eq!(
        floor(&store, &revision.problem).await,
        u64::try_from(deadline).unwrap()
    );
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn historical_source_grant_settlement_keeps_exact_receipt_and_fences_unknown_grants() {
    let (store, _, revision) = fixture().await;
    let selection = store
        .protect(revision.clone(), Duration::from_secs(60))
        .await
        .unwrap();
    let key = selection.grant_key().to_owned();
    selection.finish().await.unwrap();
    let next_authority = uuid::Uuid::new_v4().to_string();
    bounded_query(store.db.query("UPDATE ONLY type::record('canonical_guards','retention:'+$problem) SET incarnation=$authority,protection_closed_through=29dec;").bind(("problem",revision.problem.clone())).bind(("authority",next_authority))).await.unwrap();
    assert!(
        store
            .settle_protection_grant(&revision.problem, &key)
            .await
            .unwrap()
    );
    assert!(
        store
            .settle_protection_grant(&revision.problem, &key)
            .await
            .unwrap()
    );
    assert_eq!(floor(&store, &revision.problem).await, 29);
    let mut receipt = bounded_query(
        store
            .db
            .query("RETURN SELECT * FROM ONLY type::record('canonical_protections',$key);")
            .bind(("key", key.clone())),
    )
    .await
    .unwrap();
    assert_eq!(
        receipt
            .take::<Option<Object>>(0)
            .unwrap()
            .unwrap()
            .get("released"),
        Some(&Value::Bool(true))
    );
    let parts = key.split(':').collect::<Vec<_>>();
    let unknown = format!(
        "pse.protection.v2:{}:{}:{}",
        parts[1],
        uuid::Uuid::new_v4(),
        parts[3]
    );
    assert!(
        store
            .settle_protection_grant(&revision.problem, &unknown)
            .await
            .is_err()
    );
    assert!(
        store
            .settle_protection_grant("other-problem", &key)
            .await
            .is_err()
    );
    assert!(
        bounded_query(
            store
                .db
                .query(ACQUIRE_PROTECTION)
                .bind(("problem", revision.problem.clone()))
                .bind(("revision", revision.key.clone()))
                .bind(("key", key.clone()))
                .bind(("lifetime", 60_000_000i64))
        )
        .await
        .is_err()
    );
    assert_eq!(floor(&store, &revision.problem).await, 29);
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn commit_order_publication_never_appears_behind_an_advanced_page_cursor() {
    let (store, options, revision) = fixture().await;
    let initial = store
        .begin_run(&RunRequest {
            purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
            admission_study: None,
            admission_point: None,
            key: "publication-base".into(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        })
        .await
        .unwrap();
    let early = held(&options).await;
    let late = held(&options).await;
    let mut early_row = initial.clone();
    early_row.key = "publication-early-draw".into();
    early_row.sequence = 0;
    let mut late_row = early_row.clone();
    late_row.key = "publication-late-draw".into();
    // Both transactions draw from the same conflict-key version. Commit the later
    // draw first; the earlier draw must abort rather than publish behind its cursor.
    early
        .query("RETURN fn::pse_execution_v1::begin_run($expiry,$row,[],NONE);")
        .bind(("expiry", expiry()))
        .bind(("row", wire::encode_canonical_runs(&early_row).unwrap()))
        .await
        .unwrap()
        .check()
        .unwrap();
    late.query("RETURN fn::pse_execution_v1::begin_run($expiry,$row,[],NONE);")
        .bind(("expiry", expiry()))
        .bind(("row", wire::encode_canonical_runs(&late_row).unwrap()))
        .await
        .unwrap()
        .check()
        .unwrap();
    late.commit().await.unwrap();
    conflict(early.commit().await.unwrap_err());
    let first = store
        .execution_run_page(&revision.problem, Some(initial.sequence), 1)
        .await
        .unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].key, late_row.key);
    let advanced = first[0].sequence;
    let retried = store
        .begin_run(&RunRequest {
            purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
            admission_study: None,
            admission_point: None,
            key: early_row.key.clone().into(),
            revision: revision.clone(),
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        })
        .await
        .unwrap();
    assert!(retried.sequence > advanced);
    for index in 0..3 {
        store
            .begin_run(&RunRequest {
                purpose: pse_model::generated::enums::CanonicalRunPurpose::Execution,
                admission_study: None,
                admission_point: None,
                key: format!("publication-tail-{index}").into(),
                revision: revision.clone(),
                sources: vec![],
                request: vec![1],
                source_selection: vec![2],
                attestation: vec![3],
            })
            .await
            .unwrap();
    }
    let mut cursor = advanced;
    let mut seen = Vec::new();
    loop {
        let page = store
            .execution_run_page(&revision.problem, Some(cursor), 2)
            .await
            .unwrap();
        if page.is_empty() {
            break;
        }
        assert!(page.iter().all(|row| row.sequence > cursor));
        cursor = page.last().unwrap().sequence;
        seen.extend(page.into_iter().map(|row| row.key));
    }
    assert_eq!(seen.len(), 4);
    assert_eq!(seen[0], early_row.key);
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn actual_native_conflict_rebuilds_the_complete_decision_under_original_clock() {
    use crate::canonical::{QueryEffect, pacing_observation};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let (store, options, _) = fixture().await;
    bounded_query(store.db.query(
        "CREATE canonical_guards:qualified_retry SET key='qualified_retry',generation=0dec;",
    ))
    .await
    .unwrap();
    let rival = held(&options).await;
    rival
        .query("UPDATE canonical_guards:qualified_retry SET generation+=1dec;")
        .await
        .unwrap()
        .check()
        .unwrap();
    let reached = Arc::new(tokio::sync::Notify::new());
    let committed = Arc::new(tokio::sync::Notify::new());
    let attempts = Arc::new(AtomicUsize::new(0));
    let probe = pacing_observation::Probe::default();
    let started = std::time::Instant::now();
    let decision=probe.scope(crate::canonical::operation_scope(started+Duration::from_secs(20),store.protected_query("qualified-native-retry",QueryEffect::Mutation,"qualification::native_conflict_retry",|| {
        let options=options.clone();
        let reached=reached.clone();
        let committed=committed.clone();
        let ordinal=attempts.fetch_add(1,Ordering::SeqCst);
        Ok(async move {
            let transaction=held(&options).await;
            let mut selected=transaction.query("SELECT VALUE generation FROM ONLY canonical_guards:qualified_retry FOR UPDATE;").await?.check()?;
            let generation=selected.take::<Value>(0)?;
            if ordinal==0 {
                reached.notify_one();
                committed.notified().await;
            }
            let response=transaction.query("UPDATE canonical_guards:qualified_retry SET generation=$generation+1dec;").bind(("generation",generation)).await?.check()?;
            transaction.commit().await?;
            Ok(response)
        })
    })));
    let commit_first = async {
        reached.notified().await;
        rival.commit().await.unwrap();
        committed.notify_one();
    };
    let (response, ()) = tokio::time::timeout(Duration::from_secs(25), async {
        tokio::join!(decision, commit_first)
    })
    .await
    .unwrap();
    response.unwrap();
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        2,
        "exactly the known conflict permits one complete rebuild"
    );
    assert_eq!(probe.snapshot()["qualified-native-retry"].conflicts, 1);
    let mut result = bounded_query(
        store
            .db
            .query("SELECT VALUE generation FROM ONLY canonical_guards:qualified_retry;"),
    )
    .await
    .unwrap();
    assert_eq!(
        canonical_codec::decode_uint(result.take(0).unwrap()).unwrap(),
        2
    );
    eprintln!(
        "actual native conflict retry: elapsed={:?}, observations={:#?}",
        started.elapsed(),
        probe.snapshot()
    );
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn native_sql_serializable_isolation_rejects_existing_row_write_skew_in_both_commit_orders() {
    let (store, options, _) = fixture().await;
    for reverse in [false, true] {
        let left_key = format!("ssi-existing-{reverse}-left");
        let right_key = format!("ssi-existing-{reverse}-right");
        for key in [&left_key, &right_key] {
            bounded_query(store.db.query("CREATE ONLY type::record('canonical_guards',$key) SET key=$key,generation=0dec;").bind(("key",key.clone()))).await.unwrap();
        }
        let left = held(&options).await;
        let right = held(&options).await;
        for transaction in [&left, &right] {
            let mut selected=transaction.query("RETURN [type::record('canonical_guards',$left).generation,type::record('canonical_guards',$right).generation];")
                .bind(("left",left_key.clone())).bind(("right",right_key.clone())).await.unwrap().check().unwrap();
            let values = selected.take::<Vec<Value>>(0).unwrap();
            assert_eq!(
                values
                    .into_iter()
                    .map(|value| canonical_codec::decode_uint(value).unwrap())
                    .collect::<Vec<_>>(),
                vec![0, 0]
            );
        }
        left.query("UPDATE ONLY type::record('canonical_guards',$key) SET generation=1dec;")
            .bind(("key", left_key.clone()))
            .await
            .unwrap()
            .check()
            .unwrap();
        right
            .query("UPDATE ONLY type::record('canonical_guards',$key) SET generation=1dec;")
            .bind(("key", right_key.clone()))
            .await
            .unwrap()
            .check()
            .unwrap();
        if reverse {
            right.commit().await.unwrap();
            conflict(left.commit().await.unwrap_err());
        } else {
            left.commit().await.unwrap();
            conflict(right.commit().await.unwrap_err());
        }
        let mut selected=bounded_query(store.db.query("RETURN [type::record('canonical_guards',$left).generation,type::record('canonical_guards',$right).generation];")
            .bind(("left",left_key)).bind(("right",right_key))).await.unwrap();
        let values = selected.take::<Vec<Value>>(0).unwrap();
        assert_eq!(
            values
                .into_iter()
                .map(|value| canonical_codec::decode_uint(value).unwrap())
                .sum::<u64>(),
            1
        );
    }
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn native_sql_serializable_isolation_rejects_absent_key_write_skew_in_both_commit_orders() {
    let (store, options, _) = fixture().await;
    for reverse in [false, true] {
        let left_key = format!("ssi-absent-{reverse}-left");
        let right_key = format!("ssi-absent-{reverse}-right");
        let left = held(&options).await;
        let right = held(&options).await;
        for (transaction, key) in [(&left, &right_key), (&right, &left_key)] {
            let mut selected = transaction
                .query("SELECT * FROM ONLY type::record('canonical_guards',$key);")
                .bind(("key", key.clone()))
                .await
                .unwrap()
                .check()
                .unwrap();
            assert!(selected.take::<Option<Object>>(0).unwrap().is_none());
        }
        for (transaction, key) in [(&left, &left_key), (&right, &right_key)] {
            transaction.query("CREATE ONLY type::record('canonical_guards',$key) SET key=$key,generation=1dec;").bind(("key",key.clone())).await.unwrap().check().unwrap();
        }
        if reverse {
            right.commit().await.unwrap();
            conflict(left.commit().await.unwrap_err());
        } else {
            left.commit().await.unwrap();
            conflict(right.commit().await.unwrap_err());
        }
        let mut selected=bounded_query(store.db.query("RETURN [type::record('canonical_guards',$left).generation!=NONE,type::record('canonical_guards',$right).generation!=NONE];")
            .bind(("left",left_key)).bind(("right",right_key))).await.unwrap();
        let values = selected.take::<Vec<bool>>(0).unwrap();
        assert_eq!(values.into_iter().filter(|exists| *exists).count(), 1);
    }
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn native_sql_serializable_isolation_detects_empty_record_range_interference() {
    let (store, options, _) = fixture().await;
    let reader = held(&options).await;
    let mut selected = reader
        .query("SELECT * FROM $page ORDER BY id LIMIT 17;")
        .bind((
            "page",
            canonical_codec::record_page("canonical_guards", "zz_ssi_range_"),
        ))
        .await
        .unwrap()
        .check()
        .unwrap();
    assert!(selected.take::<Vec<Object>>(0).unwrap().is_empty());
    let writer = held(&options).await;
    writer.query("CREATE canonical_guards:zz_ssi_range_inserted SET key='zz_ssi_range_inserted',generation=1dec;").await.unwrap().check().unwrap();
    writer.commit().await.unwrap();
    reader.query("CREATE canonical_guards:ssi_range_decision SET key='ssi_range_decision',generation=1dec;").await.unwrap().check().unwrap();
    conflict(reader.commit().await.unwrap_err());
    let mut selected = bounded_query(
        store
            .db
            .query("SELECT * FROM ONLY canonical_guards:ssi_range_decision;"),
    )
    .await
    .unwrap();
    assert!(selected.take::<Option<Object>>(0).unwrap().is_none());
    store.remove_isolated_fixture().await.unwrap();
}

#[tokio::test]
async fn native_sql_snapshot_reader_remains_stable_until_cancel_then_new_reader_observes_commit() {
    let (store, options, _) = fixture().await;
    bounded_query(
        store
            .db
            .query("CREATE canonical_guards:ssi_snapshot SET key='ssi_snapshot',generation=0dec;"),
    )
    .await
    .unwrap();
    let reader = held(&options).await;
    let mut selected = reader
        .query("SELECT VALUE generation FROM ONLY canonical_guards:ssi_snapshot;")
        .await
        .unwrap()
        .check()
        .unwrap();
    assert_eq!(
        canonical_codec::decode_uint(selected.take::<Value>(0).unwrap()).unwrap(),
        0
    );
    bounded_query(
        store
            .db
            .query("UPDATE canonical_guards:ssi_snapshot SET generation=1dec;"),
    )
    .await
    .unwrap();
    let mut selected = reader
        .query("SELECT VALUE generation FROM ONLY canonical_guards:ssi_snapshot;")
        .await
        .unwrap()
        .check()
        .unwrap();
    assert_eq!(
        canonical_codec::decode_uint(selected.take::<Value>(0).unwrap()).unwrap(),
        0
    );
    reader.cancel().await.unwrap();
    let reader = held(&options).await;
    let mut selected = reader
        .query("SELECT VALUE generation FROM ONLY canonical_guards:ssi_snapshot;")
        .await
        .unwrap()
        .check()
        .unwrap();
    assert_eq!(
        canonical_codec::decode_uint(selected.take::<Value>(0).unwrap()).unwrap(),
        1
    );
    reader.cancel().await.unwrap();
    store.remove_isolated_fixture().await.unwrap();
}
