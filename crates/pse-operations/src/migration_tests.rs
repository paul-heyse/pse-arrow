// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Upgrade controls use immutable SQL from the committed Plan 25d source, only in isolated databases.
use super::*;
use crate::testing::{Fault, FaultPoint, FaultProxy, TestDatabase};
use std::time::Duration;

const SOURCE_DDL: &str = include_str!("../test-fixtures/plan25d/schema.sql");
const SOURCE_PHYSICAL: &str = include_str!("../test-fixtures/plan25d/physical.sql");
const SNAPSHOT: &str = "SELECT jsonb_build_object('attempts',(SELECT jsonb_agg((to_jsonb(x)-'request_identity'-'operational_job_identity'-'operational_job_frame') || jsonb_build_object('recorded_operational_key',COALESCE(to_jsonb(x)->'request_identity',to_jsonb(x)->'operational_job_identity'))) FROM pse_ops.attempts x),'jobs',(SELECT jsonb_agg(to_jsonb(x)) FROM pse_ops.jobs x),'workspaces',(SELECT jsonb_agg(to_jsonb(x)) FROM pse_ops.workspaces x),'publications',(SELECT jsonb_agg(to_jsonb(x)) FROM pse_ops.publications x),'leases',(SELECT jsonb_agg(to_jsonb(x)) FROM pse_ops.reader_leases x),'retention',(SELECT jsonb_agg(to_jsonb(x)) FROM pse_ops.retention_marks x))::text";
const HISTORICAL_ROWS: &str = r#"
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
    "#;
async fn migrate(store: &Store) -> Result<MigrationReport, OperationsError> {
    let plan = store.migration_plan().await?;
    store.migrate(&plan).await
}
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
    session.execute(HISTORICAL_ROWS).await.unwrap();
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
    assert_eq!((catalog.len(), operations.len()), (2, 8));
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
    assert!(migrate(db.store()).await.unwrap().final_ready.ready);
    assert_eq!(session.texts(SNAPSHOT).await.unwrap(), before);
    assert_eq!(
        session
            .count("SELECT count(*) FROM pse_ops.catalog_schema_history")
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        session
            .count("SELECT count(*) FROM pse_ops.operations_schema_history")
            .await
            .unwrap(),
        8
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
        reason(migrate(db.store()).await.unwrap_err(), expected);
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
            migrate(db.store()).await.unwrap_err(),
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
        assert!(migrate(&interrupted).await.is_err());
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
        assert!(migrate(db.store()).await.unwrap().final_ready.ready);
        assert_eq!(session.texts(SNAPSHOT).await.unwrap(), before);
        drop(session);
        proxy.shutdown().await.unwrap();
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
        run_history(&mut session.client, &catalog[..1], CATALOG_HISTORY)
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
            migrate(db.store()).await.unwrap_err(),
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
        tokio::time::timeout(Duration::from_secs(2), migrate(&admin))
            .await
            .unwrap()
            .unwrap_err(),
        MigrationRefusal::ActiveGeneration,
    );
    let borrowed = db.store().client().await.unwrap();
    db.store().close();
    tokio::time::sleep(Duration::from_millis(30)).await;
    reason(
        migrate(&admin).await.unwrap_err(),
        MigrationRefusal::ActiveGeneration,
    );
    drop(borrowed);
    let mut attempts = 0;
    loop {
        match migrate(&admin).await {
            Ok(report) if report.final_ready.ready => break,
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
        migrate(&admin).await.unwrap_err(),
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
        tokio::join!(db.store().create(), second.create())
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

#[test]
fn migration_plan25e_frozen_source_identity_and_follow_on_target() {
    let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::OpsSchemaV1);
    hash.part(include_bytes!("../test-fixtures/plan25e/schema.sql"))
        .part(include_bytes!("../test-fixtures/plan25e/physical.sql"));
    assert_eq!(hash.finish_hash().to_hex(), STUDY_MIGRATION_SOURCE);
    let sql = include_str!("../migrations/V5__study_occurrence_policy.sql");
    assert!(sql.contains(INVENTORY_MIGRATION_SOURCE));
    assert!(sql.contains(PLAN25F_OPERATIONS));
    // The replacement has precisely the registry's table declaration, including column order.
    let declared = SCHEMA_SQL
        .split_once("CREATE TABLE pse_ops.\"study_points\" (")
        .unwrap()
        .1
        .split_once("\n);")
        .unwrap()
        .0;
    let transition = sql
        .split_once("CREATE TABLE pse_ops.\"study_points\" (")
        .unwrap()
        .1
        .split_once("\n);")
        .unwrap()
        .0;
    assert_eq!(transition, declared);
    assert!(sql.contains("'kind','legacy_unavailable'"));
    assert!(sql.contains("'predecessor',predecessor"));
    assert!(sql.contains("'binding_hash',encode(binding_hash,'hex')"));
    assert!(!sql.contains("UPDATE pse_ops.jobs"));
    assert!(!sql.contains("UPDATE pse_ops.studies"));
    assert!(!sql.contains("UPDATE pse_ops.study_point_members"));
    let (_, operations) = transitions().unwrap();
    assert_eq!(operations[4].version(), 5);
    assert_eq!(operations[4].name(), "study_occurrence_policy");
}

#[test]
fn migration_source_and_history_prefix_are_admitted_together() {
    for (origin, pending, catalog, operations) in [
        (MIGRATION_SOURCE, false, 0, 0),
        (MIGRATION_SOURCE, true, 0, 0),
        (MIGRATION_SOURCE, true, 1, 0),
        (MIGRATION_SOURCE, true, 1, 5),
        (STUDY_MIGRATION_SOURCE, false, 1, 4),
        (STUDY_MIGRATION_SOURCE, true, 1, 4),
        (STUDY_MIGRATION_SOURCE, true, 1, 5),
        (FRESH_PLAN25E_ORIGIN, false, 0, 0),
        (FRESH_PLAN25E_ORIGIN, true, 0, 0),
        (FRESH_PLAN25E_ORIGIN, true, 0, 1),
        (FRESH_PLAN25E_ORIGIN, true, 1, 4),
    ] {
        assert!(
            admit_prefix(origin, pending, catalog, operations).is_ok(),
            "{origin} {pending} {catalog}/{operations}"
        );
    }
    for (origin, pending, catalog, operations) in [
        (MIGRATION_SOURCE, false, 1, 4),
        (MIGRATION_SOURCE, true, 0, 1),
        (MIGRATION_SOURCE, true, 1, 6),
        (STUDY_MIGRATION_SOURCE, false, 0, 0),
        (STUDY_MIGRATION_SOURCE, false, 1, 3),
        (STUDY_MIGRATION_SOURCE, false, 1, 5),
        (STUDY_MIGRATION_SOURCE, true, 1, 3),
        (STUDY_MIGRATION_SOURCE, true, 0, 4),
        (FRESH_PLAN25E_ORIGIN, false, 0, 1),
        (FRESH_PLAN25E_ORIGIN, true, 1, 0),
        (FRESH_PLAN25E_ORIGIN, true, 1, 5),
        ("unknown", false, 0, 0),
    ] {
        reason(
            admit_prefix(origin, pending, catalog, operations).unwrap_err(),
            MigrationRefusal::ChecksumConflict,
        );
    }
}

#[test]
fn migration_follow_on_preserves_enum_order_and_complete_layout_signatures() {
    let sql = include_str!("../migrations/V5__study_occurrence_policy.sql");
    for (name, target) in plan25f_layout::ENUMS {
        let source = plan25e_layout::ENUMS
            .iter()
            .find(|(old, _)| old == name)
            .unwrap()
            .1;
        assert!(target.starts_with(source));
        for (i, value) in target.iter().enumerate().skip(source.len()) {
            assert!(sql.contains(&format!(
                "ALTER TYPE pse_ops.{name} ADD VALUE '{value}' AFTER '{}';",
                target[i - 1]
            )));
        }
    }
    let signatures: Vec<(String, String, String, String)> = serde_json::from_str(include_str!(
        "../migrations/plan25f-constraint-signatures.json"
    ))
    .unwrap();
    for (table, name, kind) in plan25f_layout::CONSTRAINTS {
        assert!(
            signatures
                .iter()
                .any(|r| r.0 == *table && r.1 == *name && r.2 == *kind),
            "{table}.{name}"
        );
    }
    assert!(!signatures.iter().any(|r| matches!(
        r.1.as_str(),
        "study_points_binding_key"
            | "study_points_predecessor_fkey"
            | "study_points_predecessor_is_earlier_check"
    )));
    let domains: Vec<(String, String, bool)> =
        serde_json::from_str(include_str!("../migrations/plan25f-domain-signatures.json")).unwrap();
    let source_domains: Vec<(String, String, bool)> =
        serde_json::from_str(include_str!("../migrations/plan25e-domain-signatures.json")).unwrap();
    assert_eq!(domains, source_domains);
}

async fn plan25e_source(fresh: bool) -> TestDatabase {
    if !fresh {
        let db = legacy().await;
        let mut session = db.store().schema_session().await.unwrap();
        let (catalog, operations) = transitions().unwrap();
        run_history(&mut session.client, &catalog[..1], CATALOG_HISTORY)
            .await
            .unwrap();
        run_history(&mut session.client, &operations[..4], OPERATIONS_HISTORY)
            .await
            .unwrap();
        drop(session);
        return db;
    }
    let db = TestDatabase::empty().await.unwrap();
    let session = db.session().await.unwrap();
    session
        .execute(include_str!("../test-fixtures/plan25e/schema.sql"))
        .await
        .unwrap();
    session
        .execute(include_str!("../test-fixtures/plan25e/physical.sql"))
        .await
        .unwrap();
    session.execute(&format!("INSERT INTO pse_ops.schema_support_state VALUES ('catalog',1,'fresh','{PLAN25E_CATALOG}',true),('operations',1,'fresh','{PLAN25E_OPERATIONS}',true); COMMENT ON SCHEMA pse_ops IS '{RECORD_PREFIX}{STUDY_MIGRATION_SOURCE}'")).await.unwrap();
    session.execute(HISTORICAL_ROWS).await.unwrap();
    db
}

const STUDY_ROWS: &str = r#"
INSERT INTO pse_ops.studies(study_id,attempt_id,publication_id,finalization_job,definition,state)
VALUES ('00000000-0000-0000-0000-000000000010','00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000005','00000000-0000-0000-0000-000000000003','{"version":2,"scientific_permission":"never_declared","points":[{"case":"historical"}]}','open');
INSERT INTO pse_ops.study_points(study_id,point_index,binding_hash,predecessor,job_id,state,updated_at)
VALUES ('00000000-0000-0000-0000-000000000010',0,decode(repeat('17',32),'hex'),NULL,'00000000-0000-0000-0000-000000000003','completed','2026-01-01');
INSERT INTO pse_ops.study_point_members(study_id,point_index,catalog_name,schema_name,table_name,relation_id,relation_version,contract_fingerprint,table_uri,delta_version,selection_kind)
VALUES ('00000000-0000-0000-0000-000000000010',0,'history','runtime','solution','00000000-0000-0000-0000-000000000011',1,decode(repeat('21',32),'hex'),'file:///historical/solution/',5,'full');
"#;
const STUDY_SNAPSHOT: &str = "SELECT jsonb_build_object('studies',(SELECT jsonb_agg(to_jsonb(x)) FROM pse_ops.studies x),'members',(SELECT jsonb_agg(to_jsonb(x)) FROM pse_ops.study_point_members x))::text";
const POINT_SNAPSHOT: &str = "SELECT jsonb_agg(jsonb_build_object('study_id',study_id,'point_index',point_index,'binding_hash',encode(binding_hash,'hex'),'predecessor',predecessor,'job_id',job_id,'state',state,'updated_at',updated_at) ORDER BY point_index)::text FROM pse_ops.study_points";
const MIGRATED_POINT_SNAPSHOT: &str = "SELECT jsonb_agg(jsonb_build_object('study_id',study_id,'point_index',point_index,'binding_hash',encode(binding_hash,'hex'),'predecessor',(policy->>'predecessor')::integer,'job_id',job_id,'state',state,'updated_at',updated_at) ORDER BY point_index)::text FROM pse_ops.study_points";

// Isolated PostgreSQL journeys are retained for series qualification; focused packet checks do not run them.
#[tokio::test]
async fn migration_plan25e_fresh_and_upgraded_sources_preserve_historical_occurrences() {
    for fresh in [false, true] {
        let db = plan25e_source(fresh).await;
        let session = db.session().await.unwrap();
        session.execute(STUDY_ROWS).await.unwrap();
        let historical = session.texts(SNAPSHOT).await.unwrap();
        let studies = session.texts(STUDY_SNAPSHOT).await.unwrap();
        let points = session.texts(POINT_SNAPSHOT).await.unwrap();
        assert!(migrate(db.store()).await.unwrap().final_ready.ready);
        assert_eq!(session.texts(SNAPSHOT).await.unwrap(), historical);
        assert_eq!(session.texts(STUDY_SNAPSHOT).await.unwrap(), studies);
        assert_eq!(
            session.texts(MIGRATED_POINT_SNAPSHOT).await.unwrap(),
            points
        );
        assert_eq!(session.count("SELECT count(*) FROM pse_ops.study_points WHERE revision=0 AND policy->>'kind'='legacy_unavailable' AND outcome->>'kind'='legacy_unavailable' AND policy->>'binding_hash'=encode(binding_hash,'hex')").await.unwrap(),1);
        assert_eq!(
            session
                .count("SELECT count(*) FROM pse_ops.operations_schema_history")
                .await
                .unwrap(),
            if fresh { 4 } else { 8 }
        );
        if fresh {
            assert_eq!(
                session
                    .count("SELECT count(*) FROM pse_ops.catalog_schema_history")
                    .await
                    .unwrap(),
                1
            );
        } else {
            assert_eq!(
                session
                    .count("SELECT count(*) FROM pse_ops.catalog_schema_history")
                    .await
                    .unwrap(),
                2
            );
        }
        assert!(migrate(db.store()).await.unwrap().final_ready.ready);
        drop(session);
        db.remove().await.unwrap();
    }
}

#[tokio::test]
async fn migration_inspection_is_read_only_and_stale_plan_refuses_before_marker() {
    let db = legacy().await;
    let raw = db.session().await.unwrap();
    let before = comment(&db).await;
    let plan = db.store().migration_plan().await.unwrap();
    assert_eq!(comment(&db).await, before);
    assert_eq!(plan.source, MIGRATION_SOURCE);
    assert_eq!(plan.pending_steps().len(), 10);
    assert_eq!(raw.count("SELECT count(*) FROM pg_tables WHERE schemaname='pse_ops' AND tablename LIKE '%schema_history'").await.unwrap(), 0);
    let mut changed = plan.clone();
    changed.shared_version += 1;
    reason(
        db.store().migrate(&changed).await.unwrap_err(),
        MigrationRefusal::PlanChanged,
    );
    assert_eq!(comment(&db).await, before);
    let report = db.store().migrate(&plan).await.unwrap();
    assert_eq!(report.applied, plan.pending_steps());
    assert!(report.final_ready.ready);
    assert!(report.final_ready.pending_steps().is_empty());
    reason(
        db.store().migrate(&plan).await.unwrap_err(),
        MigrationRefusal::PlanChanged,
    );
    drop(raw);
    db.remove().await.unwrap();
}

#[test]
fn migration_plan25f_frozen_source_and_appended_inventory_target() {
    let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::OpsSchemaV1);
    hash.part(include_bytes!("../test-fixtures/plan25f/schema.sql"))
        .part(include_bytes!("../test-fixtures/plan25f/physical.sql"));
    assert_eq!(hash.finish_hash().to_hex(), INVENTORY_MIGRATION_SOURCE);
    let inventory = include_str!("../migrations/V2__catalog_retirement_inventory.sql");
    let ready = include_str!("../migrations/V6__retirement_ready.sql");
    assert!(ready.contains(JOB_IDENTITY_MIGRATION_SOURCE));
    assert!(inventory.contains(crate::generated::CATALOG_FINGERPRINT_HEX));
    assert!(!inventory.contains("DROP "));
    for table in [
        "orphan_scans",
        "orphan_candidates",
        "reset_records",
        "retired_inventory",
    ] {
        let declaration = SCHEMA_SQL
            .split_once(&format!("CREATE TABLE pse_ops.\"{table}\" ("))
            .unwrap()
            .1
            .split_once("\n);")
            .unwrap()
            .0;
        let transition = inventory
            .split_once(&format!("CREATE TABLE pse_ops.\"{table}\" ("))
            .unwrap()
            .1
            .split_once("\n);")
            .unwrap()
            .0;
        assert_eq!(declaration, transition);
    }
    let signatures: Vec<(String, String, String, String)> = serde_json::from_str(include_str!(
        "../migrations/plan25g-constraint-signatures.json"
    ))
    .unwrap();
    for (table, name, kind) in plan25g_layout::CONSTRAINTS {
        assert!(
            signatures
                .iter()
                .any(|s| s.0 == *table && s.1 == *name && s.2 == *kind),
            "{table}.{name}"
        );
    }
}

async fn plan25f_source(fresh: bool) -> TestDatabase {
    if !fresh {
        let db = plan25e_source(false).await;
        let mut session = db.store().schema_session().await.unwrap();
        let (_, operations) = transitions().unwrap();
        run_history(&mut session.client, &operations[..5], OPERATIONS_HISTORY)
            .await
            .unwrap();
        drop(session);
        return db;
    }
    let db = TestDatabase::empty().await.unwrap();
    let raw = db.session().await.unwrap();
    raw.execute(include_str!("../test-fixtures/plan25f/schema.sql"))
        .await
        .unwrap();
    raw.execute(include_str!("../test-fixtures/plan25f/physical.sql"))
        .await
        .unwrap();
    raw.execute(&format!("INSERT INTO pse_ops.schema_support_state VALUES('catalog',1,'fresh','{PLAN25E_CATALOG}',true),('operations',1,'fresh','{PLAN25F_OPERATIONS}',true); COMMENT ON SCHEMA pse_ops IS '{RECORD_PREFIX}{INVENTORY_MIGRATION_SOURCE}'")).await.unwrap();
    raw.execute(HISTORICAL_ROWS).await.unwrap();
    db
}
#[tokio::test]
async fn migration_plan25f_fresh_and_upgraded_inventory_preserves_every_catalog_row() {
    for fresh in [true, false] {
        let db = plan25f_source(fresh).await;
        let raw = db.session().await.unwrap();
        let before = raw.texts(SNAPSHOT).await.unwrap();
        let marker = comment(&db).await;
        let plan = db.store().migration_plan().await.unwrap();
        assert_eq!(comment(&db).await, marker);
        assert_eq!(
            plan.pending_steps()
                .iter()
                .map(|s| s.version)
                .collect::<Vec<_>>(),
            [2, 6, 7, 8]
        );
        let report = db.store().migrate(&plan).await.unwrap();
        assert!(report.final_ready.ready);
        assert_eq!(report.applied, plan.pending_steps());
        assert_eq!(raw.texts(SNAPSHOT).await.unwrap(), before);
        let reopened = Store::open_with(db.url(), &crate::store::StoreOptions::for_tests())
            .await
            .unwrap();
        reopened.close();
        drop(raw);
        db.remove().await.unwrap();
    }
}
#[tokio::test]
async fn migration_inventory_interruption_and_malformed_timestamp_never_destroy_catalog() {
    for point in [FaultPoint::BeforeCommit, FaultPoint::AfterCommit] {
        let db = plan25f_source(false).await;
        let raw = db.session().await.unwrap();
        let before = raw.texts(SNAPSHOT).await.unwrap();
        let proxy = FaultProxy::start(db.url()).await.unwrap();
        proxy.arm(Fault {
            marker: "CREATE TABLE pse_ops.\"orphan_scans\"",
            point,
        });
        let interrupted =
            Store::connect_with(proxy.url(), &crate::store::StoreOptions::for_tests())
                .await
                .unwrap();
        let plan = interrupted.migration_plan().await.unwrap();
        assert!(interrupted.migrate(&plan).await.is_err());
        assert!(proxy.fired());
        interrupted.close();
        reason(
            db.store().open().await.unwrap_err(),
            MigrationRefusal::NotReady,
        );
        let resumed = db.store().migration_plan().await.unwrap();
        let report = db.store().migrate(&resumed).await.unwrap();
        assert!(report.final_ready.ready);
        assert_eq!(raw.texts(SNAPSHOT).await.unwrap(), before);
        proxy.shutdown().await.unwrap();
        drop(raw);
        db.remove().await.unwrap();
    }
    let db = plan25f_source(false).await;
    let raw = db.session().await.unwrap();
    let before = comment(&db).await;
    raw.execute(
        "UPDATE pse_ops.operations_schema_history SET applied_on='malformed' WHERE version=5",
    )
    .await
    .unwrap();
    reason(
        db.store().migration_plan().await.unwrap_err(),
        MigrationRefusal::ChecksumConflict,
    );
    assert_eq!(comment(&db).await, before);
    drop(raw);
    db.remove().await.unwrap();
}

#[test]
fn operational_identity_v7_preserves_v6_digest_and_unknown_frame_provenance() {
    let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::OpsSchemaV1);
    hash.part(include_bytes!("../test-fixtures/plan25g/schema.sql"))
        .part(include_bytes!("../test-fixtures/plan25g/physical.sql"));
    assert_eq!(hash.finish_hash().to_hex(), JOB_IDENTITY_MIGRATION_SOURCE);
    let sql = include_str!("../migrations/V7__operational_job_identity.sql");
    assert!(sql.contains("RENAME COLUMN request_identity TO operational_job_identity"));
    assert!(sql.contains("ADD COLUMN operational_job_frame text"));
    assert!(!sql.contains("UPDATE pse_ops.attempts"));
    assert!(sql.contains(NATIVE_BACKENDS_MIGRATION_SOURCE));
    let (_, operations) = transitions().unwrap();
    assert_eq!(
        operations
            .iter()
            .find(|step| step.version() == 7)
            .unwrap()
            .version(),
        7
    );
    assert_eq!(operations.last().unwrap().version(), 8);
}

#[tokio::test]
async fn operational_identity_v7_migrates_fresh_v6_and_preserves_unknown_provenance() {
    let db = TestDatabase::empty().await.unwrap();
    let session = db.session().await.unwrap();
    session
        .execute(include_str!("../test-fixtures/plan25g/schema.sql"))
        .await
        .unwrap();
    session
        .execute(include_str!("../test-fixtures/plan25g/physical.sql"))
        .await
        .unwrap();
    session.execute(&format!("INSERT INTO pse_ops.schema_support_state VALUES ('catalog',1,'fresh','{}',true),('operations',1,'fresh','{PLAN25F_OPERATIONS}',true); COMMENT ON SCHEMA pse_ops IS '{RECORD_PREFIX}{JOB_IDENTITY_MIGRATION_SOURCE}'", crate::generated::CATALOG_FINGERPRINT_HEX)).await.unwrap();
    session.execute("INSERT INTO pse_ops.attempts(attempt_id,run_id,kind,request_identity,state) VALUES('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000002','simulation',decode(repeat('17',32),'hex'),'completed')").await.unwrap();
    let plan = db.store().migration_plan().await.unwrap();
    assert_eq!(
        plan.pending_steps()
            .iter()
            .map(|s| s.version)
            .collect::<Vec<_>>(),
        vec![7, 8]
    );
    let report = db.store().migrate(&plan).await.unwrap();
    assert!(report.final_ready.ready);
    assert_eq!(session.texts("SELECT encode(operational_job_identity,'hex'), operational_job_frame FROM pse_ops.attempts").await.unwrap(), vec![vec![Some("17".repeat(32)),None]]);
    db.store().open().await.unwrap();
    let attempt = db
        .store()
        .attempts()
        .get(
            pse_ids::SemanticId::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1])
                .into(),
        )
        .await
        .unwrap();
    let recorded = pse_ids::roles::RecordedOperationalJobIdentity::recorded(
        attempt.operational_job_identity,
        attempt.operational_job_frame,
    );
    assert!(recorded.current_key().is_none());
    drop(session);
    db.remove().await.unwrap();
}

#[tokio::test]
async fn operational_identity_v7_migrates_upgraded_v6_and_normalizes_only_known_names() {
    let db = legacy().await;
    let mut session = db.store().schema_session().await.unwrap();
    let (catalog, operations) = transitions().unwrap();
    run_history(&mut session.client, &catalog[..1], CATALOG_HISTORY)
        .await
        .unwrap();
    run_history(&mut session.client, &operations[..5], OPERATIONS_HISTORY)
        .await
        .unwrap();
    run_history(&mut session.client, &catalog, CATALOG_HISTORY)
        .await
        .unwrap();
    run_history(&mut session.client, &operations[..6], OPERATIONS_HISTORY)
        .await
        .unwrap();
    drop(session);
    let raw = db.session().await.unwrap();
    let before = raw.texts(SNAPSHOT).await.unwrap();
    assert_eq!(raw.count("SELECT count(*) FROM pg_constraint WHERE conrelid='pse_ops.study_points'::regclass AND contype='n' AND conname LIKE '%_not_null1'").await.unwrap(), 6);
    let plan = db.store().migration_plan().await.unwrap();
    assert_eq!(
        plan.pending_steps()
            .iter()
            .map(|s| s.version)
            .collect::<Vec<_>>(),
        vec![7, 8]
    );
    assert!(db.store().migrate(&plan).await.unwrap().final_ready.ready);
    assert_eq!(raw.texts(SNAPSHOT).await.unwrap(), before);
    assert_eq!(raw.count("SELECT count(*) FROM pg_constraint WHERE conrelid='pse_ops.study_points'::regclass AND contype='n' AND conname LIKE '%_not_null1'").await.unwrap(), 0);
    assert_eq!(raw.texts("SELECT encode(operational_job_identity,'hex'), operational_job_frame FROM pse_ops.attempts").await.unwrap(), vec![vec![Some("17".repeat(32)), None]]);
    db.store().open().await.unwrap();
    drop(raw);
    db.remove().await.unwrap();
}

async fn plan25h_source(fresh: bool) -> TestDatabase {
    let db = legacy().await;
    let mut session = db.store().schema_session().await.unwrap();
    let (catalog, operations) = transitions().unwrap();
    run_history(&mut session.client, &catalog[..1], CATALOG_HISTORY)
        .await
        .unwrap();
    run_history(&mut session.client, &operations[..5], OPERATIONS_HISTORY)
        .await
        .unwrap();
    run_history(&mut session.client, &catalog, CATALOG_HISTORY)
        .await
        .unwrap();
    run_history(&mut session.client, &operations[..7], OPERATIONS_HISTORY)
        .await
        .unwrap();
    if fresh {
        // A schema created at V7 has the same physical layout with no upgrade history.
        session.client.batch_execute("DROP TABLE pse_ops.catalog_schema_history; DROP TABLE pse_ops.operations_schema_history; UPDATE pse_ops.schema_support_state SET source='fresh'").await.unwrap();
    }
    drop(session);
    db
}

#[tokio::test]
async fn native_backends_v8_preserves_v7_rows_histories_and_rejects_enum_drift() {
    for fresh in [false, true] {
        let db = plan25h_source(fresh).await;
        let raw = db.session().await.unwrap();
        let before = raw.texts(SNAPSHOT).await.unwrap();
        let marker = comment(&db).await;
        let plan = db.store().migration_plan().await.unwrap();
        assert_eq!(comment(&db).await, marker);
        assert_eq!(
            plan.pending_steps()
                .iter()
                .map(|step| step.version)
                .collect::<Vec<_>>(),
            [8]
        );
        assert!(db.store().migrate(&plan).await.unwrap().final_ready.ready);
        assert_eq!(raw.texts(SNAPSHOT).await.unwrap(), before);
        assert_eq!(raw.texts("SELECT enumlabel::text FROM pg_enum WHERE enumtypid='pse_ops.native_backend'::regtype ORDER BY enumsortorder").await.unwrap().into_iter().flatten().flatten().collect::<Vec<_>>(), crate::generated::layout::ENUMS.iter().find(|(name,_)| *name=="native_backend").unwrap().1);
        db.store().open().await.unwrap();
        drop(raw);
        db.remove().await.unwrap();
    }
    let db = legacy().await;
    let raw = db.session().await.unwrap();
    raw.execute("ALTER TYPE pse_ops.native_backend ADD VALUE 'foreign_profile'")
        .await
        .unwrap();
    let marker = comment(&db).await;
    reason(
        db.store().migration_plan().await.unwrap_err(),
        MigrationRefusal::Drift,
    );
    assert_eq!(comment(&db).await, marker);
    drop(raw);
    db.remove().await.unwrap();
}

#[tokio::test]
async fn native_backends_v8_interruption_resumes_without_rewriting_history_or_rows() {
    for point in [FaultPoint::BeforeCommit, FaultPoint::AfterCommit] {
        let db = plan25h_source(true).await;
        let raw = db.session().await.unwrap();
        let before = raw.texts(SNAPSHOT).await.unwrap();
        let proxy = FaultProxy::start(db.url()).await.unwrap();
        proxy.arm(Fault {
            marker: "ALTER TYPE pse_ops.native_backend ADD VALUE 'uno'",
            point,
        });
        let interrupted =
            Store::connect_with(proxy.url(), &crate::store::StoreOptions::for_tests())
                .await
                .unwrap();
        let plan = interrupted.migration_plan().await.unwrap();
        assert!(interrupted.migrate(&plan).await.is_err());
        assert!(proxy.fired());
        interrupted.close();
        let resumed = db.store().migration_plan().await.unwrap();
        assert!(
            db.store()
                .migrate(&resumed)
                .await
                .unwrap()
                .final_ready
                .ready
        );
        assert_eq!(raw.texts(SNAPSHOT).await.unwrap(), before);
        assert_eq!(
            raw.count("SELECT count(*) FROM pse_ops.operations_schema_history WHERE version=8")
                .await
                .unwrap(),
            1
        );
        proxy.shutdown().await.unwrap();
        drop(raw);
        db.remove().await.unwrap();
    }
}
