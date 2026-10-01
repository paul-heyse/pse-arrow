// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Upgrade controls use immutable SQL from the committed Plan 25d source, only in isolated databases.
use super::*;
use crate::testing::{Fault, FaultPoint, FaultProxy, TestDatabase};
use std::time::Duration;

const SOURCE_DDL: &str = include_str!("../test-fixtures/plan25d/schema.sql");
const SOURCE_PHYSICAL: &str = include_str!("../test-fixtures/plan25d/physical.sql");
const SNAPSHOT: &str = "SELECT jsonb_build_object('attempts',(SELECT jsonb_agg(to_jsonb(x)) FROM pse_ops.attempts x),'jobs',(SELECT jsonb_agg(to_jsonb(x)) FROM pse_ops.jobs x),'workspaces',(SELECT jsonb_agg(to_jsonb(x)) FROM pse_ops.workspaces x),'publications',(SELECT jsonb_agg(to_jsonb(x)) FROM pse_ops.publications x),'leases',(SELECT jsonb_agg(to_jsonb(x)) FROM pse_ops.reader_leases x),'retention',(SELECT jsonb_agg(to_jsonb(x)) FROM pse_ops.retention_marks x))::text";
async fn legacy() -> TestDatabase {
    let db = TestDatabase::empty().await.unwrap();
    let session = db.session().await.unwrap();
    session.execute(SOURCE_DDL).await.unwrap();
    session.execute(SOURCE_PHYSICAL).await.unwrap();
    session
        .execute(&format!(
            "COMMENT ON SCHEMA pse_ops IS '{RECORD_PREFIX}{MIGRATION_SOURCE}'"
        ))
        .await
        .unwrap();
    session.execute(r#"
      INSERT INTO pse_ops.attempts(attempt_id,run_id,kind,request_identity,state,worker,lease_expires_at,heartbeat_at)
      VALUES ('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000002','simulation',decode(repeat('17',32),'hex'),'running','historical-worker','2030-01-01','2026-01-01');
      INSERT INTO pse_ops.jobs(job_id,attempt_id,idempotency_key,payload_version,payload,state,max_tries,backoff_base_us,backoff_cap_us)
      VALUES ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000001','historical-job',1,'{"historical":{"no_new_endpoint":true},"values":[1,2,3]}','running',4,1000,10000);
      INSERT INTO pse_ops.workspaces(workspace_id,name,root_uri,maintenance_epoch) VALUES ('00000000-0000-0000-0000-000000000004','historical-workspace','file:///historical/',9);
      INSERT INTO pse_ops.publication_intents(publication_id,workspace_id,attempt_id,member_prefix)
      VALUES ('00000000-0000-0000-0000-000000000005','00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000001','file:///historical/members/');
      INSERT INTO pse_ops.publications(publication_id,workspace_id,attempt_id,kind)
      VALUES ('00000000-0000-0000-0000-000000000005','00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000001','relations');
      INSERT INTO pse_ops.reader_leases(lease_id,publication_id,head_of,holder,expires_at)
      VALUES ('00000000-0000-0000-0000-000000000006','00000000-0000-0000-0000-000000000005','00000000-0000-0000-0000-000000000004','historical-reader','2030-01-01');
      INSERT INTO pse_ops.retention_marks(publication_id,phase) VALUES ('00000000-0000-0000-0000-000000000005','expiring');
    "#).await.unwrap();
    db
}
fn reason(error: OperationsError, expected: MigrationRefusal) {
    assert!(
        matches!(error,OperationsError::MigrationRefused{reason,..} if reason==expected),
        "{error:?}"
    );
}
async fn comment(db: &TestDatabase) -> Vec<Vec<Option<String>>> {
    db.session()
        .await
        .unwrap()
        .texts(
            "SELECT obj_description(oid,'pg_namespace') FROM pg_namespace WHERE nspname='pse_ops'",
        )
        .await
        .unwrap()
}
#[test]
fn migration_frozen_source_identity_and_history_ownership() {
    let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::OpsSchemaV1);
    hash.part(SOURCE_DDL.as_bytes())
        .part(SOURCE_PHYSICAL.as_bytes());
    assert_eq!(hash.finish_hash().to_hex(), MIGRATION_SOURCE);
    let (catalog, operations) = transitions().unwrap();
    assert_eq!((catalog.len(), operations.len()), (1, 4));
    assert_ne!(CATALOG_HISTORY, OPERATIONS_HISTORY);
}
#[tokio::test]
async fn migration_supported_source_preserves_payload_and_catalog() {
    let db = legacy().await;
    let session = db.session().await.unwrap();
    let before = session.texts(SNAPSHOT).await.unwrap();
    assert!(matches!(
        db.store().open().await,
        Err(OperationsError::SchemaMismatch { .. })
    ));
    assert_eq!(db.store().migrate().await.unwrap(), Opened::Current);
    assert_eq!(session.texts(SNAPSHOT).await.unwrap(), before);
    assert_eq!(
        session
            .count("SELECT count(*) FROM pse_ops.catalog_schema_history")
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        session
            .count("SELECT count(*) FROM pse_ops.operations_schema_history")
            .await
            .unwrap(),
        4
    );
    assert_eq!(session.count("SELECT count(*) FROM pg_tables WHERE schemaname='public' AND tablename='refinery_schema_history'").await.unwrap(),0);
    let reopened = Store::open_with(db.url(), &crate::store::StoreOptions::for_tests())
        .await
        .unwrap();
    assert_eq!(
        reopened.schema_status().await.unwrap(),
        SchemaStatus::Current
    );
    reopened.close();
    drop(session);
    db.remove().await.unwrap();
}
#[tokio::test]
async fn migration_unknown_source_and_drift_refuse_without_effects() {
    for defect in [
        "COMMENT ON SCHEMA pse_ops IS 'pse.ops.schema.v1 unknown'",
        "ALTER TABLE pse_ops.jobs ADD COLUMN foreign_column text",
        "ALTER TABLE pse_ops.jobs DROP CONSTRAINT jobs_payload_version_positive_check",
        "ALTER TABLE pse_ops.jobs DROP CONSTRAINT jobs_payload_version_positive_check; ALTER TABLE pse_ops.jobs ADD CONSTRAINT jobs_payload_version_positive_check CHECK(payload_version >= 0)",
        "ALTER TABLE pse_ops.jobs ALTER COLUMN tries DROP DEFAULT",
        "DROP INDEX pse_ops.jobs_claim_idx",
        "DROP INDEX pse_ops.jobs_claim_idx; CREATE INDEX jobs_claim_idx ON pse_ops.jobs(job_id)",
        "ALTER TYPE pse_ops.native_termination ADD VALUE 'foreign_status'",
        "CREATE DOMAIN pse_ops.foreign_domain AS uuid",
        "CREATE TYPE pse_ops.foreign_enum AS ENUM ('unknown')",
        "ALTER DOMAIN pse_ops.content_hash DROP CONSTRAINT content_hash_width; ALTER DOMAIN pse_ops.content_hash ADD CONSTRAINT content_hash_width CHECK (octet_length(VALUE) >= 16)",
    ] {
        let db = legacy().await;
        let session = db.session().await.unwrap();
        session.execute(defect).await.unwrap();
        let before_comment = comment(&db).await;
        let before = session.texts(SNAPSHOT).await.unwrap();
        let expected = if defect.starts_with("COMMENT") {
            MigrationRefusal::UnknownSource
        } else {
            MigrationRefusal::Drift
        };
        reason(db.store().migrate().await.unwrap_err(), expected);
        assert_eq!(comment(&db).await, before_comment);
        assert_eq!(session.texts(SNAPSHOT).await.unwrap(), before);
        assert_eq!(session.count("SELECT count(*) FROM pg_tables WHERE schemaname='pse_ops' AND tablename IN ('catalog_schema_history','operations_schema_history','schema_support_state')").await.unwrap(),0);
        drop(session);
        db.remove().await.unwrap();
    }
}
#[tokio::test]
async fn migration_both_history_conflicts_refuse_before_marker_or_other_history() {
    for table in [CATALOG_HISTORY, OPERATIONS_HISTORY] {
        let db = legacy().await;
        let session = db.session().await.unwrap();
        session.execute(&format!("CREATE TABLE {table}(version integer PRIMARY KEY,name text NOT NULL,applied_on text NOT NULL,checksum text NOT NULL); INSERT INTO {table} VALUES(1,'incorrect','2026-01-01','1')")).await.unwrap();
        let before = comment(&db).await;
        reason(
            db.store().migrate().await.unwrap_err(),
            MigrationRefusal::ChecksumConflict,
        );
        assert_eq!(comment(&db).await, before);
        assert_eq!(session.count("SELECT count(*) FROM pg_tables WHERE schemaname='pse_ops' AND tablename='schema_support_state'").await.unwrap(),0);
        drop(session);
        db.remove().await.unwrap();
    }
}
#[tokio::test]
async fn migration_interrupted_committed_prefix_resumes_and_open_refuses_not_ready() {
    for point in [FaultPoint::BeforeCommit, FaultPoint::AfterCommit] {
        let db = legacy().await;
        let session = db.session().await.unwrap();
        let before = session.texts(SNAPSHOT).await.unwrap();
        let proxy = FaultProxy::start(db.url()).await.unwrap();
        proxy.arm(Fault {
            marker: "CREATE TABLE pse_ops.\"schema_support_state\"",
            point,
        });
        let interrupted =
            Store::connect_with(proxy.url(), &crate::store::StoreOptions::for_tests())
                .await
                .unwrap();
        assert!(interrupted.migrate().await.is_err());
        assert!(proxy.fired());
        interrupted.close();
        reason(
            db.store().open().await.unwrap_err(),
            MigrationRefusal::NotReady,
        );
        reason(
            db.store().client().await.unwrap_err(),
            MigrationRefusal::NotReady,
        );
        assert_eq!(db.store().migrate().await.unwrap(), Opened::Current);
        assert_eq!(session.texts(SNAPSHOT).await.unwrap(), before);
        drop(session);
        drop(proxy);
        db.remove().await.unwrap();
    }
}
#[tokio::test]
async fn migration_committed_partial_drift_and_checksum_conflict_preserve_prefix() {
    for defect in [
        "UPDATE pse_ops.operations_schema_history SET checksum='1' WHERE version=1",
        "ALTER TABLE pse_ops.jobs ADD COLUMN foreign_column text",
    ] {
        let db = legacy().await;
        let mut session = db.store().schema_session().await.unwrap();
        let pending = format!("{PENDING_PREFIX}{MIGRATION_SOURCE} {SCHEMA_FINGERPRINT_HEX}");
        session
            .client
            .batch_execute(&format!("COMMENT ON SCHEMA pse_ops IS '{pending}'"))
            .await
            .unwrap();
        let (catalog, operations) = transitions().unwrap();
        run_history(&mut session.client, &catalog, CATALOG_HISTORY)
            .await
            .unwrap();
        run_history(&mut session.client, &operations[..1], OPERATIONS_HISTORY)
            .await
            .unwrap();
        drop(session);
        let raw = db.session().await.unwrap();
        raw.execute(defect).await.unwrap();
        let before_comment = comment(&db).await;
        reason(
            db.store().migrate().await.unwrap_err(),
            if defect.starts_with("UPDATE") {
                MigrationRefusal::ChecksumConflict
            } else {
                MigrationRefusal::Drift
            },
        );
        assert_eq!(comment(&db).await, before_comment);
        assert_eq!(
            raw.count("SELECT count(*) FROM pse_ops.operations_schema_history")
                .await
                .unwrap(),
            1
        );
        drop(raw);
        db.remove().await.unwrap();
    }
}
#[tokio::test]
async fn migration_open_generation_and_competing_owner_refuse_until_quiescent() {
    let db = TestDatabase::create().await.unwrap();
    let admin = Store::connect_with(db.url(), &crate::store::StoreOptions::for_tests())
        .await
        .unwrap();
    reason(
        tokio::time::timeout(Duration::from_secs(2), admin.migrate())
            .await
            .unwrap()
            .unwrap_err(),
        MigrationRefusal::ActiveGeneration,
    );
    let borrowed = db.store().client().await.unwrap();
    db.store().close();
    tokio::time::sleep(Duration::from_millis(30)).await;
    reason(
        admin.migrate().await.unwrap_err(),
        MigrationRefusal::ActiveGeneration,
    );
    drop(borrowed);
    let mut attempts = 0;
    loop {
        match admin.migrate().await {
            Ok(Opened::Current) => break,
            Err(OperationsError::MigrationRefused {
                reason: MigrationRefusal::ActiveGeneration,
                ..
            }) if attempts < 100 => {
                attempts += 1;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            other => panic!("{other:?}"),
        }
    }
    let session = admin.schema_session().await.unwrap();
    session
        .client
        .query_one("SELECT pg_advisory_lock($1)", &[&SCHEMA_LOCK])
        .await
        .unwrap();
    reason(
        admin.migrate().await.unwrap_err(),
        MigrationRefusal::ActiveGeneration,
    );
    drop(session);
    admin.close();
    db.remove().await.unwrap();
}

#[tokio::test]
async fn migration_read_only_repeated_and_concurrent_open_never_wait_for_existing_generation() {
    let db = TestDatabase::empty().await.unwrap();
    let second = Store::connect_with(db.url(), &crate::store::StoreOptions::for_tests())
        .await
        .unwrap();
    let (first, other) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(db.store().open(), second.open())
    })
    .await
    .unwrap();
    let states = [first.unwrap(), other.unwrap()];
    assert!(states.contains(&Opened::Created));
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(2), db.store().open())
            .await
            .unwrap()
            .unwrap(),
        Opened::Current
    );
    let third = Store::connect_with(db.url(), &crate::store::StoreOptions::for_tests())
        .await
        .unwrap();
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(2), third.open())
            .await
            .unwrap()
            .unwrap(),
        Opened::Current
    );
    second.close();
    third.close();
    db.remove().await.unwrap();
}
