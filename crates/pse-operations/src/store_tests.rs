// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Store tests against real, isolated PostgreSQL 18 databases: `#[sqlx::test]` creates one
//! database per test from `DATABASE_URL` (`just db-test` maps `PSE_DATABASE_URL` to it), and
//! [`opened`] creates the registry-generated schema in it with `Store::open`.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use chrono::{DateTime, Utc};
use pse_ids::{ContentHash, SemanticId};
use sqlx::PgPool;

use crate::attempts::{
    AttemptKind, DiagnosticCode, NewAttempt, RuntimeTermination, TerminationCode, TransitionNote,
};
use crate::cancellation::CancelOutcome;
use crate::catalog::{
    Committed, Member, ProtectedVersion, PublicationCommit, ReadTarget, Settlement, Workspace,
};
use crate::jobs::{Enqueued, Finished, JobOutcome, JobState, NewJob, RetryPolicy};
use crate::lifecycle::AttemptState;
use crate::solutions::{SeedVectors, Solution};
use crate::streams::{Incumbent, ProgressEvent, ProgressValue};
use crate::{Opened, OperationsError, SchemaStatus, Store, mint_id};

const LEASE: Duration = Duration::from_secs(30);

/// The store of a `#[sqlx::test]` database, with its schema created by `Store::open`.
async fn opened(pool: PgPool) -> Store {
    let store = Store::from_pool(pool);
    assert_eq!(store.open().await.unwrap(), Opened::Created);
    store
}

/// The SQLSTATE and constraint of a refused statement.
fn refusal(error: &sqlx::Error) -> (Option<String>, Option<String>) {
    let database = error.as_database_error();
    (
        database.and_then(|e| e.code()).map(|c| c.into_owned()),
        database.and_then(|e| e.constraint()).map(str::to_owned),
    )
}

fn hash(byte: u8) -> ContentHash {
    ContentHash::from_bytes([byte; 32])
}

fn new_attempt() -> NewAttempt {
    NewAttempt {
        attempt_id: mint_id(),
        run_id: mint_id(),
        kind: AttemptKind::Simulation,
        request_identity: hash(1),
        preparation_identity: Some(hash(2)),
        parent_attempt: None,
    }
}

fn new_job(key: &str, retry: RetryPolicy) -> NewJob {
    NewJob {
        attempt: new_attempt(),
        idempotency_key: key.to_owned(),
        payload_version: 1,
        payload: serde_json::json!({ "case": key }),
        priority: 0,
        retry,
    }
}

fn at(seconds: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(1_790_000_000 + seconds, 0).unwrap()
}

/// Create an attempt and drive it through the lifecycle to `completed`.
async fn finished_attempt(store: &Store) -> SemanticId {
    let attempts = store.attempts();
    let attempt = new_attempt();
    attempts.create(&attempt, Some("test")).await.unwrap();
    attempts
        .transition(
            attempt.attempt_id,
            AttemptState::Queued,
            &TransitionNote::by("test"),
        )
        .await
        .unwrap();
    attempts
        .start(attempt.attempt_id, "worker-a", LEASE)
        .await
        .unwrap();
    attempts
        .transition(
            attempt.attempt_id,
            AttemptState::Completed,
            &TransitionNote::by("worker-a"),
        )
        .await
        .unwrap();
    attempt.attempt_id
}

async fn workspace(store: &Store) -> SemanticId {
    let workspace = Workspace {
        workspace_id: mint_id(),
        name: format!("ws-{}", mint_id()),
        root_uri: "file:///tmp/pse-workspace/".to_owned(),
    };
    store
        .catalog()
        .register_workspace(&workspace)
        .await
        .unwrap();
    workspace.workspace_id
}

fn members(version: i64) -> Vec<Member> {
    vec![
        Member {
            member: "runtime.computation_runs".to_owned(),
            table_uri: "file:///tmp/pse-workspace/runs/".to_owned(),
            delta_version: version,
            contract_fingerprint: hash(9),
        },
        Member {
            member: "runtime.solve_metrics".to_owned(),
            table_uri: "file:///tmp/pse-workspace/metrics/".to_owned(),
            delta_version: version,
            contract_fingerprint: hash(8),
        },
    ]
}

async fn publish(
    store: &Store,
    workspace: SemanticId,
    parent: Option<SemanticId>,
    version: i64,
) -> SemanticId {
    let publication_id = mint_id();
    let commit = PublicationCommit {
        publication_id,
        workspace_id: workspace,
        attempt_id: finished_attempt(store).await,
        expected_parent: parent,
        members: members(version),
    };
    assert_eq!(
        store.catalog().commit(&commit).await.unwrap(),
        Committed::Advanced { publication_id }
    );
    publication_id
}

// ------------------------------------------------------------------- schema --

#[sqlx::test(migrations = false)]
async fn generated_schema_creates_empty_store(pool: PgPool) {
    let store = Store::from_pool(pool);
    assert_eq!(store.schema_status().await.unwrap(), SchemaStatus::Absent);
    assert_eq!(store.open().await.unwrap(), Opened::Created);
    assert_eq!(store.schema_status().await.unwrap(), SchemaStatus::Current);
    // A second open, as every durable runtime does, changes nothing.
    assert_eq!(store.open().await.unwrap(), Opened::Current);

    let registry = pse_schema::registry().unwrap();
    let mut expected: Vec<&str> = pse_schema::store::relations(registry)
        .into_iter()
        .map(|(table, _)| table)
        .collect();
    expected.sort_unstable();
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT table_name::text FROM information_schema.tables \
         WHERE table_schema = 'pse_ops' ORDER BY table_name",
    )
    .fetch_all(store.pool())
    .await
    .unwrap();
    assert_eq!(tables, expected);
    for table in &expected {
        let rows: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT count(*) FROM pse_ops.\"{table}\""
        )))
        .fetch_one(store.pool())
        .await
        .unwrap();
        assert_eq!(rows, 0, "{table} is not empty");
    }
    let recorded: Option<String> = sqlx::query_scalar(
        "SELECT obj_description(oid, 'pg_namespace') FROM pg_namespace WHERE nspname = 'pse_ops'",
    )
    .fetch_one(store.pool())
    .await
    .unwrap();
    assert_eq!(
        recorded,
        Some(format!("pse.ops.schema.v1 {}", Store::expected_schema()))
    );
    let server = store.server().await.unwrap();
    assert!(server.is_supported(), "{server:?}");
}

#[sqlx::test(migrations = false)]
async fn store_schema_mismatch_refused(pool: PgPool) {
    let store = opened(pool).await;
    let other = "0".repeat(64);
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "COMMENT ON SCHEMA pse_ops IS 'pse.ops.schema.v1 {other}'"
    )))
    .execute(store.pool())
    .await
    .unwrap();
    let refused = store.open().await.unwrap_err();
    assert!(
        matches!(&refused, OperationsError::SchemaMismatch { recorded: Some(r), .. } if *r == other),
        "{refused:?}"
    );
    let help = miette::Diagnostic::help(&refused).map(|h| h.to_string());
    assert!(help.is_some_and(|h| h.contains("just db-reset")));
    assert_eq!(
        store.schema_status().await.unwrap(),
        SchemaStatus::Mismatch {
            recorded: Some(other)
        }
    );
    // A schema without a record (for example one the superseded migrations created) is
    // refused the same way, and never reset implicitly.
    sqlx::raw_sql("COMMENT ON SCHEMA pse_ops IS NULL")
        .execute(store.pool())
        .await
        .unwrap();
    assert!(matches!(
        store.open().await.unwrap_err(),
        OperationsError::SchemaMismatch { recorded: None, .. }
    ));
    // The explicit reset recreates it from this build.
    assert_eq!(store.reset().await.unwrap(), Opened::Created);
    assert_eq!(store.schema_status().await.unwrap(), SchemaStatus::Current);
}

#[sqlx::test(migrations = false)]
async fn registry_enums_are_postgres_enums(pool: PgPool) {
    let store = opened(pool).await;
    let registry = pse_schema::registry().unwrap();
    let mut checked = BTreeSet::new();
    for (table, spec) in pse_schema::store::relations(registry) {
        for column in &spec.columns {
            let Some(name) = column.enum_name() else {
                continue;
            };
            let (schema, udt): (String, String) = sqlx::query_as(
                "SELECT udt_schema::text, udt_name::text FROM information_schema.columns \
                 WHERE table_schema = 'pse_ops' AND table_name = $1 AND column_name = $2",
            )
            .bind(table)
            .bind(column.name())
            .fetch_one(store.pool())
            .await
            .unwrap();
            assert_eq!(schema, "pse_ops", "{table}.{}", column.name());
            let labels: Vec<String> = sqlx::query_scalar(
                "SELECT e.enumlabel::text FROM pg_enum e \
                 JOIN pg_type t ON t.oid = e.enumtypid \
                 JOIN pg_namespace n ON n.oid = t.typnamespace \
                 WHERE n.nspname = 'pse_ops' AND t.typname = $1 ORDER BY e.enumsortorder",
            )
            .bind(&udt)
            .fetch_all(store.pool())
            .await
            .unwrap();
            let members: Vec<&str> = registry
                .enum_spec(name)
                .unwrap()
                .members
                .iter()
                .map(|member| member.name)
                .collect();
            assert_eq!(labels, members, "{table}.{} is {udt}", column.name());
            checked.insert(name);
        }
    }
    for name in ["AttemptState", "JobState", "TerminationClass", "RetentionPhase"] {
        assert!(checked.contains(name), "{name} is not a store column type");
    }
}

#[sqlx::test(migrations = false)]
async fn misspelled_enum_literal_fails_prepare(pool: PgPool) {
    let store = opened(pool).await;
    // A literal compared with an ENUM column is resolved when the statement is prepared.
    let error = sqlx::raw_sql(
        "PREPARE misspelled AS SELECT attempt_id FROM pse_ops.attempts WHERE state = 'runing'",
    )
    .execute(store.pool())
    .await
    .unwrap_err();
    assert_eq!(refusal(&error).0.as_deref(), Some("22P02"), "{error}");
    sqlx::raw_sql(
        "PREPARE spelled AS SELECT attempt_id FROM pse_ops.attempts WHERE state = 'running'",
    )
    .execute(store.pool())
    .await
    .unwrap();
}

#[sqlx::test(migrations = false)]
async fn row_invariants_generated_and_enforced(pool: PgPool) {
    let store = opened(pool).await;
    let registry = pse_schema::registry().unwrap();
    // Every declared row check is a named CHECK constraint of its table.
    let constraints: BTreeSet<String> = sqlx::query_scalar(
        "SELECT c.conname::text FROM pg_constraint c \
         JOIN pg_namespace n ON n.oid = c.connamespace WHERE n.nspname = 'pse_ops'",
    )
    .fetch_all(store.pool())
    .await
    .unwrap()
    .into_iter()
    .collect();
    for (table, spec) in pse_schema::store::relations(registry) {
        for name in spec.checks.keys() {
            let constraint = format!("{table}_{name}_check");
            assert!(constraints.contains(&constraint), "{constraint} is not generated");
        }
        for key in &spec.unique_keys {
            assert!(constraints.contains(&format!("{table}_{}_key", key.name)));
        }
        for reference in &spec.foreign_keys {
            assert!(constraints.contains(&format!("{table}_{}_fkey", reference.name)));
        }
    }
    let attempt = new_attempt();
    store.attempts().create(&attempt, None).await.unwrap();
    let id = crate::codec::uuid(attempt.attempt_id);
    let refused = |sql: &'static str| {
        let store = store.clone();
        async move {
            sqlx::query(sql)
                .bind(id)
                .execute(store.pool())
                .await
                .map(|_| ())
                .unwrap_err()
        }
    };
    // A running attempt holds a lease (the former multi-column CHECK).
    let error = refused(
        "UPDATE pse_ops.attempts SET state = 'running', worker = 'w' WHERE attempt_id = $1",
    )
    .await;
    assert_eq!(
        refusal(&error),
        (
            Some("23514".into()),
            Some("attempts_running_holds_lease_check".into())
        )
    );
    // Exactly the typed termination column the class selects is present (X4).
    let error = refused(
        "UPDATE pse_ops.attempts SET termination_class = 'native' WHERE attempt_id = $1",
    )
    .await;
    assert_eq!(
        refusal(&error).1.as_deref(),
        Some("attempts_one_termination_check")
    );
    // A finite field domain refuses NaN and infinity in PostgreSQL too.
    let error = refused(
        "INSERT INTO pse_ops.progress_events (attempt_id, seq, step, at, elapsed_seconds, phase) \
         VALUES ($1, 0, 0, now(), 'Infinity', 'phase')",
    )
    .await;
    assert_eq!(
        refusal(&error).1.as_deref(),
        Some("progress_events_elapsed_seconds_finite")
    );
    // A composite reference is enforced, and nothing cascades.
    let error = refused(
        "INSERT INTO pse_ops.progress_values (attempt_id, seq, name, kind, \"integer\") \
         VALUES ($1, 7, 'n', 'integer', 1)",
    )
    .await;
    assert_eq!(
        refusal(&error),
        (
            Some("23503".into()),
            Some("progress_values_progress_event_fkey".into())
        )
    );
    // Content hashes are exactly 32 bytes, through their domain.
    let error = sqlx::query("INSERT INTO pse_ops.source_bundles (bundle_hash, manifest) VALUES ($1, '{}')")
        .bind(vec![0_u8; 31])
        .execute(store.pool())
        .await
        .unwrap_err();
    assert_eq!(refusal(&error).0.as_deref(), Some("23514"), "{error}");
    // The seed-vector rule is a registry row check.
    let error = sqlx::query(
        "INSERT INTO pse_ops.solutions (solution_id, compatibility_stamp, preparation_identity, \
             kind, backend, profile_stamp, data_stamp) \
         VALUES ($1, $2, $2, 'root', 'kinsol', $2, $2)",
    )
    .bind(crate::codec::uuid(mint_id()))
    .bind(hash(1).as_bytes().to_vec())
    .execute(store.pool())
    .await
    .unwrap_err();
    assert_eq!(refusal(&error).1.as_deref(), Some("solutions_vectors_check"));
}

#[sqlx::test(migrations = false)]
async fn transition_history_is_append_only(pool: PgPool) {
    let store = opened(pool).await;
    let attempt = new_attempt();
    store.attempts().create(&attempt, None).await.unwrap();
    let error = sqlx::query("UPDATE pse_ops.attempt_transitions SET reason = 'rewritten'")
        .execute(store.pool())
        .await
        .unwrap_err();
    let code = error
        .as_database_error()
        .and_then(|e| e.code())
        .map(|c| c.into_owned());
    assert_eq!(code.as_deref(), Some("42501"));
    let error = sqlx::query("DELETE FROM pse_ops.attempt_transitions")
        .execute(store.pool())
        .await
        .unwrap_err();
    let code = error
        .as_database_error()
        .and_then(|e| e.code())
        .map(|c| c.into_owned());
    assert_eq!(code.as_deref(), Some("42501"));
    assert_eq!(
        store
            .attempts()
            .history(attempt.attempt_id)
            .await
            .unwrap()
            .len(),
        1
    );
}

#[sqlx::test(migrations = false)]
async fn sqlstate_errors_map_to_typed_variants(pool: PgPool) {
    let store = opened(pool).await;
    let target = store.target().clone();

    // 23505: a second attempt with the same identity.
    let attempt = new_attempt();
    store.attempts().create(&attempt, None).await.unwrap();
    let duplicate = store.attempts().create(&attempt, None).await.unwrap_err();
    assert!(
        matches!(&duplicate, OperationsError::Duplicate { constraint: Some(c), .. } if c == "attempts_pkey"),
        "{duplicate:?}"
    );
    assert!(!duplicate.is_retryable());

    // 55P03: NOWAIT on a row another transaction holds.
    let mut holder = store.pool().begin().await.unwrap();
    sqlx::query("SELECT 1 FROM pse_ops.attempts WHERE attempt_id = $1 FOR UPDATE")
        .bind(crate::codec::uuid(attempt.attempt_id))
        .execute(&mut *holder)
        .await
        .unwrap();
    let nowait =
        sqlx::query("SELECT 1 FROM pse_ops.attempts WHERE attempt_id = $1 FOR UPDATE NOWAIT")
            .bind(crate::codec::uuid(attempt.attempt_id))
            .execute(store.pool())
            .await
            .unwrap_err();
    let nowait = OperationsError::from_sqlx(nowait, &target);
    assert!(
        matches!(nowait, OperationsError::LockUnavailable { .. }),
        "{nowait:?}"
    );
    assert!(nowait.is_retryable());
    holder.rollback().await.unwrap();

    // 57014: a statement timeout cancels the statement.
    let mut conn = store.pool().acquire().await.unwrap();
    sqlx::query("SET statement_timeout = '20ms'")
        .execute(&mut *conn)
        .await
        .unwrap();
    let cancelled = sqlx::query("SELECT pg_sleep(2)")
        .execute(&mut *conn)
        .await
        .unwrap_err();
    let cancelled = OperationsError::from_sqlx(cancelled, &target);
    assert!(
        matches!(cancelled, OperationsError::Cancelled { .. }),
        "{cancelled:?}"
    );
    sqlx::query("RESET statement_timeout")
        .execute(&mut *conn)
        .await
        .unwrap();
    drop(conn);

    // 40001: write skew between two serializable transactions. The server may refuse
    // the second insert or the second commit; either way exactly one step fails.
    let mut first = store.pool().begin().await.unwrap();
    let mut second = store.pool().begin().await.unwrap();
    let serializable = "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE";
    let read = "SELECT count(*) FROM pse_ops.workspaces";
    let write = "INSERT INTO pse_ops.workspaces (workspace_id, name, root_uri) \
                 VALUES ($1, $2, 'file:///x/')";
    sqlx::query(serializable)
        .execute(&mut *first)
        .await
        .unwrap();
    sqlx::query(serializable)
        .execute(&mut *second)
        .await
        .unwrap();
    let _: i64 = sqlx::query_scalar(read)
        .fetch_one(&mut *first)
        .await
        .unwrap();
    let _: i64 = sqlx::query_scalar(read)
        .fetch_one(&mut *second)
        .await
        .unwrap();
    sqlx::query(write)
        .bind(crate::codec::uuid(mint_id()))
        .bind("one")
        .execute(&mut *first)
        .await
        .unwrap();
    let second_write = sqlx::query(write)
        .bind(crate::codec::uuid(mint_id()))
        .bind("two")
        .execute(&mut *second)
        .await;
    first.commit().await.unwrap();
    let failure = match second_write {
        Err(error) => error,
        Ok(_) => second.commit().await.unwrap_err(),
    };
    let conflict = OperationsError::from_sqlx(failure, &target);
    assert!(
        matches!(conflict, OperationsError::Conflict { .. }),
        "{conflict:?}"
    );
    assert!(conflict.is_retryable());

    // 40P01: two transactions lock two rows in opposite order.
    let other = new_attempt();
    store.attempts().create(&other, None).await.unwrap();
    let (a, b) = (
        crate::codec::uuid(attempt.attempt_id),
        crate::codec::uuid(other.attempt_id),
    );
    let mut left = store.pool().begin().await.unwrap();
    let mut right = store.pool().begin().await.unwrap();
    let lock = "SELECT 1 FROM pse_ops.attempts WHERE attempt_id = $1 FOR UPDATE";
    sqlx::query(lock).bind(a).execute(&mut *left).await.unwrap();
    sqlx::query(lock)
        .bind(b)
        .execute(&mut *right)
        .await
        .unwrap();
    let (l, r) = tokio::join!(
        sqlx::query(lock).bind(b).execute(&mut *left),
        sqlx::query(lock).bind(a).execute(&mut *right),
    );
    let failures: Vec<OperationsError> = [l.err(), r.err()]
        .into_iter()
        .flatten()
        .map(|e| OperationsError::from_sqlx(e, &target))
        .collect();
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(
        matches!(failures[0], OperationsError::Deadlock { .. }),
        "{failures:?}"
    );
    left.rollback().await.unwrap();
    right.rollback().await.unwrap();
}

#[tokio::test]
async fn unreachable_server_is_unavailable_naming_the_target() {
    let options = crate::StoreOptions {
        acquire_timeout: Duration::from_secs(2),
        ..crate::StoreOptions::default()
    };
    let error = Store::connect_with("postgres:///pse?host=/nonexistent-pse-socket", &options)
        .await
        .unwrap_err();
    assert!(
        matches!(error, OperationsError::Unavailable { .. }),
        "{error:?}"
    );
    assert!(
        error.to_string().contains("/nonexistent-pse-socket"),
        "{error}"
    );
    assert!(error.is_retryable());
}

// ---------------------------------------------------------------- attempts --

#[sqlx::test(migrations = false)]
async fn illegal_transition_rejected(pool: PgPool) {
    let store = opened(pool).await;
    let attempts = store.attempts();
    let attempt = new_attempt();
    let created = attempts.create(&attempt, Some("runtime")).await.unwrap();
    assert_eq!(created.state, AttemptState::Planned);

    let refused = attempts
        .transition(
            attempt.attempt_id,
            AttemptState::Completed,
            &TransitionNote::by("x"),
        )
        .await
        .unwrap_err();
    assert!(
        matches!(
            refused,
            OperationsError::IllegalTransition {
                from: AttemptState::Planned,
                to: AttemptState::Completed,
                ..
            }
        ),
        "{refused:?}"
    );
    // Nothing changed, and the refusal left no audit row.
    let unchanged = attempts.get(attempt.attempt_id).await.unwrap();
    assert_eq!(
        (unchanged.state, unchanged.state_version),
        (AttemptState::Planned, 0)
    );
    assert_eq!(attempts.history(attempt.attempt_id).await.unwrap().len(), 1);

    // Entering running needs a lease; the generic transition refuses it.
    attempts
        .transition(
            attempt.attempt_id,
            AttemptState::Queued,
            &TransitionNote::by("q"),
        )
        .await
        .unwrap();
    let no_lease = attempts
        .transition(
            attempt.attempt_id,
            AttemptState::Running,
            &TransitionNote::by("w"),
        )
        .await
        .unwrap_err();
    assert!(
        matches!(no_lease, OperationsError::InvalidRequest { .. }),
        "{no_lease:?}"
    );

    // The legal path records one audit row per transition, in order.
    attempts
        .start(attempt.attempt_id, "worker-a", LEASE)
        .await
        .unwrap();
    let done = attempts
        .transition(
            attempt.attempt_id,
            AttemptState::Failed,
            &TransitionNote::by("worker-a")
                .because("evaluation failed")
                .terminated(crate::attempts::Termination {
                    code: TerminationCode::Rule(DiagnosticCode::SolveEvaluationError),
                    detail: Some(serde_json::json!({ "row": 3 })),
                }),
        )
        .await
        .unwrap();
    assert_eq!(done.state, AttemptState::Failed);
    assert_eq!(done.lease_expires_at, None);
    assert!(done.finished_at.is_some());
    assert_eq!(
        done.termination.as_ref().map(|t| &t.code),
        Some(&TerminationCode::Rule(DiagnosticCode::SolveEvaluationError))
    );
    let row = done.row();
    assert_eq!(
        row.termination_class,
        Some(crate::attempts::TerminationClass::Rule)
    );
    assert_eq!(row.termination_rule, Some(DiagnosticCode::SolveEvaluationError));
    assert_eq!(row.termination_detail.as_deref(), Some("{\"row\":3}"));
    // A later termination replaces every termination column at once.
    let cancelled = crate::attempts::Termination {
        code: TerminationCode::Runtime(RuntimeTermination::Cancelled),
        detail: None,
    };
    let row = crate::attempts::AttemptRecord {
        termination: Some(cancelled),
        ..done.clone()
    }
    .row();
    assert_eq!(row.termination_rule, None);
    assert_eq!(row.termination_runtime, Some(RuntimeTermination::Cancelled));
    let history = attempts.history(attempt.attempt_id).await.unwrap();
    let steps: Vec<(Option<AttemptState>, AttemptState)> =
        history.iter().map(|t| (t.from, t.to)).collect();
    assert_eq!(
        steps,
        [
            (None, AttemptState::Planned),
            (Some(AttemptState::Planned), AttemptState::Queued),
            (Some(AttemptState::Queued), AttemptState::Running),
            (Some(AttemptState::Running), AttemptState::Failed),
        ]
    );
    assert_eq!(
        history.last().and_then(|t| t.reason.as_deref()),
        Some("evaluation failed")
    );
    let refused_again = attempts
        .transition(
            attempt.attempt_id,
            AttemptState::Queued,
            &TransitionNote::by("x"),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        refused_again,
        OperationsError::IllegalTransition { .. }
    ));
}

#[sqlx::test(migrations = false)]
async fn lease_expiry_marks_stale(pool: PgPool) {
    let store = opened(pool).await;
    let attempts = store.attempts();
    let attempt = new_attempt();
    attempts.create(&attempt, None).await.unwrap();
    attempts
        .transition(
            attempt.attempt_id,
            AttemptState::Queued,
            &TransitionNote::by("q"),
        )
        .await
        .unwrap();
    attempts
        .start(attempt.attempt_id, "worker-a", Duration::from_millis(1))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(20)).await;

    // An expired lease cannot be renewed.
    let lost = attempts
        .heartbeat(attempt.attempt_id, "worker-a", LEASE)
        .await
        .unwrap_err();
    assert!(
        matches!(lost, OperationsError::LeaseLost { .. }),
        "{lost:?}"
    );

    assert_eq!(
        attempts.sweep_stale(100).await.unwrap(),
        [attempt.attempt_id]
    );
    assert_eq!(attempts.sweep_stale(100).await.unwrap(), []);
    let stale = attempts.get(attempt.attempt_id).await.unwrap();
    assert_eq!(stale.state, AttemptState::Stale);
    assert_eq!(stale.lease_expires_at, None);
    assert_eq!(stale.worker.as_deref(), Some("worker-a"));
}

#[sqlx::test(migrations = false)]
async fn heartbeat_extends_lease_and_returns_cancellation(pool: PgPool) {
    let store = opened(pool).await;
    let attempts = store.attempts();
    let attempt = new_attempt();
    attempts.create(&attempt, None).await.unwrap();
    attempts
        .transition(
            attempt.attempt_id,
            AttemptState::Queued,
            &TransitionNote::by("q"),
        )
        .await
        .unwrap();
    let started = attempts
        .start(attempt.attempt_id, "worker-a", LEASE)
        .await
        .unwrap();
    let ack = attempts
        .heartbeat(attempt.attempt_id, "worker-a", Duration::from_secs(120))
        .await
        .unwrap();
    assert!(!ack.cancel_requested);
    assert!(Some(ack.lease_expires_at) > started.lease_expires_at);
    // Another worker cannot heartbeat someone else's lease.
    let foreign = attempts
        .heartbeat(attempt.attempt_id, "worker-b", LEASE)
        .await
        .unwrap_err();
    assert!(matches!(foreign, OperationsError::LeaseLost { .. }));

    assert_eq!(
        store
            .request_cancel(attempt.attempt_id, "user")
            .await
            .unwrap(),
        CancelOutcome::Requested
    );
    let ack = attempts
        .heartbeat(attempt.attempt_id, "worker-a", LEASE)
        .await
        .unwrap();
    assert!(ack.cancel_requested);
}

// -------------------------------------------------------------------- jobs --

#[sqlx::test(migrations = false)]
async fn enqueue_is_idempotent_per_key(pool: PgPool) {
    let store = opened(pool).await;
    let job = new_job("request-1", RetryPolicy::ONCE);
    let created = store.jobs().enqueue(&job).await.unwrap();
    assert!(matches!(created, Enqueued::Created { .. }));
    let attempt = store.attempts().get(created.attempt_id()).await.unwrap();
    assert_eq!(attempt.state, AttemptState::Queued);

    // Same key, freshly minted attempt: the existing job is returned and nothing is stored.
    let again = new_job("request-1", RetryPolicy::ONCE);
    let existing = store.jobs().enqueue(&again).await.unwrap();
    assert_eq!(
        existing,
        Enqueued::Existing {
            job_id: created.job_id(),
            attempt_id: created.attempt_id()
        }
    );
    let missing = store
        .attempts()
        .get(again.attempt.attempt_id)
        .await
        .unwrap_err();
    assert!(matches!(missing, OperationsError::NotFound { .. }));

    let refused = store
        .jobs()
        .enqueue(&NewJob {
            payload_version: 0,
            ..new_job("request-2", RetryPolicy::ONCE)
        })
        .await
        .unwrap_err();
    assert!(matches!(refused, OperationsError::InvalidRequest { .. }));
}

#[sqlx::test(migrations = false)]
async fn two_workers_never_claim_same_job(pool: PgPool) {
    let store = opened(pool).await;
    let mut enqueued = BTreeSet::new();
    for index in 0..24 {
        let job = store
            .jobs()
            .enqueue(&new_job(&format!("job-{index}"), RetryPolicy::ONCE))
            .await
            .unwrap();
        enqueued.insert(job.job_id());
    }

    // A row locked by one claimer is skipped by the other, not waited for.
    let first = enqueued.iter().next().copied().unwrap();
    let mut holder = store.pool().begin().await.unwrap();
    sqlx::query("SELECT 1 FROM pse_ops.jobs WHERE job_id = $1 FOR UPDATE")
        .bind(crate::codec::uuid(first))
        .execute(&mut *holder)
        .await
        .unwrap();
    let skipped = store
        .jobs()
        .claim("worker-b", LEASE)
        .await
        .unwrap()
        .unwrap();
    assert_ne!(skipped.job_id, first);
    holder.rollback().await.unwrap();

    async fn drain(store: &Store, worker: &str) -> Vec<SemanticId> {
        let mut claimed = Vec::new();
        while let Some(job) = store.jobs().claim(worker, LEASE).await.unwrap() {
            assert_eq!(job.try_number, 1);
            claimed.push(job.job_id);
            tokio::task::yield_now().await;
        }
        claimed
    }
    let (a, b) = tokio::join!(drain(&store, "worker-a"), drain(&store, "worker-b"));
    let mut all: Vec<SemanticId> = a.iter().chain(&b).copied().collect();
    all.push(skipped.job_id);
    let unique: BTreeSet<SemanticId> = all.iter().copied().collect();
    assert_eq!(unique.len(), all.len(), "a job was claimed twice");
    assert_eq!(unique, enqueued);
    assert!(
        !a.is_empty() && !b.is_empty(),
        "both workers should have claimed work"
    );

    for job in &all {
        let record = store.jobs().get(*job).await.unwrap();
        assert_eq!(record.state, JobState::Running);
        let attempt = store.attempts().get(record.attempt_id).await.unwrap();
        assert_eq!(attempt.state, AttemptState::Running);
    }
}

#[sqlx::test(migrations = false)]
async fn expired_lease_requeues_as_new_attempt(pool: PgPool) {
    let store = opened(pool).await;
    let retry = RetryPolicy {
        max_tries: 2,
        backoff: Duration::ZERO,
        backoff_cap: Duration::ZERO,
    };
    let job = store
        .jobs()
        .enqueue(&new_job("long-solve", retry))
        .await
        .unwrap();
    let first = store
        .jobs()
        .claim("worker-a", Duration::from_millis(1))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first.attempt_id, job.attempt_id());
    tokio::time::sleep(Duration::from_millis(20)).await;

    // The worker vanished: the sweep marks its attempt stale and requeues a new attempt.
    let next = mint_id();
    let mut minted = Some(next);
    let swept = store
        .jobs()
        .requeue_expired(10, || minted.take().unwrap_or_else(mint_id))
        .await
        .unwrap();
    assert_eq!(swept.len(), 1);
    assert_eq!(swept[0].stale_attempt, first.attempt_id);
    assert!(
        matches!(swept[0].outcome, Finished::Requeued { attempt_id, .. } if attempt_id == next)
    );

    let old = store.attempts().get(first.attempt_id).await.unwrap();
    assert_eq!(old.state, AttemptState::Superseded);
    let old_steps: Vec<AttemptState> = store
        .attempts()
        .history(first.attempt_id)
        .await
        .unwrap()
        .iter()
        .map(|t| t.to)
        .collect();
    assert_eq!(
        old_steps,
        [
            AttemptState::Planned,
            AttemptState::Queued,
            AttemptState::Running,
            AttemptState::Stale,
            AttemptState::Superseded,
        ]
    );
    let new = store.attempts().get(next).await.unwrap();
    assert_eq!(new.state, AttemptState::Queued);
    assert_eq!(new.parent_attempt, Some(first.attempt_id));
    assert_eq!(new.run_id, old.run_id);

    // The retry runs as the new attempt; a second expiry exhausts the policy.
    let second = store
        .jobs()
        .claim("worker-b", Duration::from_millis(1))
        .await
        .unwrap()
        .unwrap();
    assert_eq!((second.attempt_id, second.try_number), (next, 2));
    tokio::time::sleep(Duration::from_millis(20)).await;
    let swept = store.jobs().requeue_expired(10, mint_id).await.unwrap();
    assert_eq!(swept.len(), 1);
    assert_eq!(swept[0].outcome, Finished::Ended(JobState::Failed));
    let record = store.jobs().get(job.job_id()).await.unwrap();
    assert_eq!((record.state, record.tries), (JobState::Failed, 2));
    assert_eq!(
        store.attempts().get(next).await.unwrap().state,
        AttemptState::Stale
    );
    assert!(
        store
            .jobs()
            .requeue_expired(10, mint_id)
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrations = false)]
async fn attempt_sweep_then_job_requeue_compose(pool: PgPool) {
    let store = opened(pool).await;
    let retry = RetryPolicy {
        max_tries: 3,
        backoff: Duration::ZERO,
        backoff_cap: Duration::ZERO,
    };
    let job = store
        .jobs()
        .enqueue(&new_job("swept", retry))
        .await
        .unwrap();
    let claimed = store
        .jobs()
        .claim("worker-a", Duration::from_millis(1))
        .await
        .unwrap()
        .unwrap();
    tokio::time::sleep(Duration::from_millis(20)).await;
    // The attempt sweep runs first and marks the job's attempt stale without requeueing.
    assert_eq!(
        store.attempts().sweep_stale(10).await.unwrap(),
        [claimed.attempt_id]
    );
    assert_eq!(
        store.jobs().get(job.job_id()).await.unwrap().state,
        JobState::Running
    );
    // The job sweep still recovers the job from its stale attempt.
    let swept = store.jobs().requeue_expired(10, mint_id).await.unwrap();
    assert_eq!(swept.len(), 1);
    let Finished::Requeued { attempt_id, .. } = swept[0].outcome else {
        panic!("expected a requeue, got {:?}", swept[0].outcome);
    };
    assert_eq!(
        store
            .attempts()
            .get(claimed.attempt_id)
            .await
            .unwrap()
            .state,
        AttemptState::Superseded
    );
    let record = store.jobs().get(job.job_id()).await.unwrap();
    assert_eq!(
        (record.state, record.attempt_id),
        (JobState::Queued, attempt_id)
    );
}

#[sqlx::test(migrations = false)]
async fn failed_try_retries_under_policy_and_completion_ends_job(pool: PgPool) {
    let store = opened(pool).await;
    let retry = RetryPolicy {
        max_tries: 2,
        backoff: Duration::from_secs(60),
        backoff_cap: Duration::from_secs(60),
    };
    let job = store
        .jobs()
        .enqueue(&new_job("flaky", retry))
        .await
        .unwrap();
    let claimed = store
        .jobs()
        .claim("worker-a", LEASE)
        .await
        .unwrap()
        .unwrap();

    // Only the owner may finish the try.
    let outcome = JobOutcome {
        state: AttemptState::Failed,
        note: TransitionNote::by("worker-a").because("license server unreachable"),
        retry_as: Some(mint_id()),
    };
    let foreign = store
        .jobs()
        .finish(job.job_id(), "worker-b", &outcome)
        .await
        .unwrap_err();
    assert!(matches!(foreign, OperationsError::LeaseLost { .. }));

    let finished = store
        .jobs()
        .finish(job.job_id(), "worker-a", &outcome)
        .await
        .unwrap();
    let Finished::Requeued {
        attempt_id,
        available_at,
    } = finished
    else {
        panic!("expected a requeue, got {finished:?}");
    };
    assert_eq!(Some(attempt_id), outcome.retry_as);
    assert!(available_at > claimed.lease_expires_at - chrono::TimeDelta::seconds(31));
    // Backoff: not claimable yet.
    assert!(
        store
            .jobs()
            .claim("worker-b", LEASE)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        store
            .attempts()
            .get(claimed.attempt_id)
            .await
            .unwrap()
            .state,
        AttemptState::Failed
    );

    sqlx::query("UPDATE pse_ops.jobs SET available_at = now() WHERE job_id = $1")
        .bind(crate::codec::uuid(job.job_id()))
        .execute(store.pool())
        .await
        .unwrap();
    let retried = store
        .jobs()
        .claim("worker-b", LEASE)
        .await
        .unwrap()
        .unwrap();
    assert_eq!((retried.attempt_id, retried.try_number), (attempt_id, 2));
    let done = store
        .jobs()
        .finish(
            job.job_id(),
            "worker-b",
            &JobOutcome {
                state: AttemptState::Completed,
                note: TransitionNote::by("worker-b"),
                retry_as: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(done, Finished::Ended(JobState::Completed));
    assert_eq!(
        store.attempts().get(attempt_id).await.unwrap().state,
        AttemptState::Completed
    );
}

// ------------------------------------------------------------ cancellation --

#[sqlx::test(migrations = false)]
async fn cancel_notify_stops_running_job(pool: PgPool) {
    let store = opened(pool).await;
    let job = store
        .jobs()
        .enqueue(&new_job("cancel-me", RetryPolicy::ONCE))
        .await
        .unwrap();
    let claimed = store
        .jobs()
        .claim("worker-a", LEASE)
        .await
        .unwrap()
        .unwrap();
    let mut watcher = store.watch_cancellation(claimed.attempt_id).await.unwrap();
    assert!(!watcher.is_requested().await.unwrap());

    // The worker: wait for cancellation, then end the try as cancelled.
    let worker = async {
        tokio::time::timeout(Duration::from_secs(10), watcher.requested())
            .await
            .expect("the watcher must observe the cancellation")
            .unwrap();
        store
            .jobs()
            .finish(
                job.job_id(),
                "worker-a",
                &JobOutcome {
                    state: AttemptState::Cancelled,
                    note: TransitionNote::by("worker-a").because("cancel requested"),
                    retry_as: None,
                },
            )
            .await
            .unwrap()
    };
    // Another process: request cancellation after the worker started waiting.
    let requester = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        store
            .request_cancel(claimed.attempt_id, "user")
            .await
            .unwrap()
    };
    let (finished, requested) = tokio::join!(worker, requester);
    assert_eq!(requested, CancelOutcome::Requested);
    assert_eq!(finished, Finished::Ended(JobState::Cancelled));
    let attempt = store.attempts().get(claimed.attempt_id).await.unwrap();
    assert_eq!(attempt.state, AttemptState::Cancelled);
    assert!(attempt.cancel_requested);
    assert_eq!(
        store
            .request_cancel(claimed.attempt_id, "user")
            .await
            .unwrap(),
        CancelOutcome::AlreadyFinished(AttemptState::Cancelled)
    );
}

#[sqlx::test(migrations = false)]
async fn cancel_observed_after_listener_reconnect(pool: PgPool) {
    let store = opened(pool).await;
    store
        .jobs()
        .enqueue(&new_job("reconnect", RetryPolicy::ONCE))
        .await
        .unwrap();
    let claimed = store
        .jobs()
        .claim("worker-a", LEASE)
        .await
        .unwrap()
        .unwrap();
    let mut watcher = store.watch_cancellation(claimed.attempt_id).await.unwrap();

    let waiting = async {
        tokio::time::timeout(Duration::from_secs(10), watcher.requested())
            .await
            .expect("a lost connection must lead to a re-read")
            .unwrap();
    };
    // The request is written without a notification (as if it was lost), then the
    // listener's connection is terminated: only the re-read after reconnect can see it.
    let disrupt = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        sqlx::query(
            "UPDATE pse_ops.attempts SET cancel_requested = true, cancel_requested_at = now() \
             WHERE attempt_id = $1",
        )
        .bind(crate::codec::uuid(claimed.attempt_id))
        .execute(store.pool())
        .await
        .unwrap();
        let terminated: Vec<bool> = sqlx::query_scalar(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity \
             WHERE datname = current_database() AND pid <> pg_backend_pid() \
               AND query LIKE 'LISTEN%'",
        )
        .fetch_all(store.pool())
        .await
        .unwrap();
        assert_eq!(terminated, [true]);
    };
    tokio::join!(waiting, disrupt);
}

#[sqlx::test(migrations = false)]
async fn cancel_before_start_cancels_job(pool: PgPool) {
    let store = opened(pool).await;
    let job = store
        .jobs()
        .enqueue(&new_job("never-run", RetryPolicy::ONCE))
        .await
        .unwrap();
    assert_eq!(
        store
            .request_cancel(job.attempt_id(), "user")
            .await
            .unwrap(),
        CancelOutcome::CancelledBeforeStart
    );
    assert_eq!(
        store.jobs().get(job.job_id()).await.unwrap().state,
        JobState::Cancelled
    );
    assert_eq!(
        store.attempts().get(job.attempt_id()).await.unwrap().state,
        AttemptState::Cancelled
    );
    assert!(
        store
            .jobs()
            .claim("worker-a", LEASE)
            .await
            .unwrap()
            .is_none()
    );
}

// ----------------------------------------------------------------- streams --

/// A progress event whose values cover every kind, including a real whose shortest
/// decimal form a jsonb payload would round (0.1 + 0.2).
fn progress_event(seq: i64) -> ProgressEvent {
    let mut values = BTreeMap::from([
        ("iteration".to_owned(), ProgressValue::Integer(seq)),
        (
            "inf_pr".to_owned(),
            ProgressValue::real(seq as f64 * 0.25 + (0.1 + 0.2)),
        ),
        ("restored".to_owned(), ProgressValue::Boolean(seq % 2 == 0)),
        ("mode".to_owned(), ProgressValue::Text(format!("m{seq}"))),
    ]);
    if seq % 7 == 0 {
        values.insert("step_norm".to_owned(), ProgressValue::real(f64::NAN));
    }
    ProgressEvent {
        seq,
        step: i32::try_from(seq / 100).unwrap(),
        at: at(seq),
        elapsed_seconds: seq as f64 * 1e-3,
        phase: if seq < 250 {
            "restoration"
        } else {
            "optimality"
        }
        .to_owned(),
        values,
    }
}

#[sqlx::test(migrations = false)]
async fn progress_batch_insert_roundtrip(pool: PgPool) {
    let store = opened(pool).await;
    let attempt = new_attempt();
    store.attempts().create(&attempt, None).await.unwrap();
    let events: Vec<ProgressEvent> = (0..500).map(progress_event).collect();
    assert_eq!(
        events[7].values["step_norm"],
        ProgressValue::Unavailable(
            pse_model::generated::enums::EvidenceUnavailableReason::Nonfinite
        )
    );

    let mut listener = sqlx::postgres::PgListener::connect_with(store.pool())
        .await
        .unwrap();
    listener
        .listen(crate::streams::PROGRESS_CHANNEL)
        .await
        .unwrap();
    assert_eq!(
        store
            .streams()
            .append_progress(attempt.attempt_id, &events)
            .await
            .unwrap(),
        500
    );
    let notification = tokio::time::timeout(Duration::from_secs(5), listener.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        notification.payload(),
        crate::codec::uuid(attempt.attempt_id).to_string()
    );
    // A re-sent overlapping batch inserts only the new tail.
    let resent: Vec<ProgressEvent> = (495..505).map(progress_event).collect();
    assert_eq!(
        store
            .streams()
            .append_progress(attempt.attempt_id, &resent)
            .await
            .unwrap(),
        5
    );

    let mut read = Vec::new();
    let mut after = None;
    loop {
        let page = store
            .streams()
            .progress(attempt.attempt_id, after, 128)
            .await
            .unwrap();
        let Some(last) = page.last() else { break };
        after = Some(last.seq);
        read.extend(page);
    }
    let mut expected = events;
    expected.extend(resent.into_iter().skip(5));
    // Every value round-trips exactly, reals bit for bit.
    assert_eq!(read, expected);
    assert_eq!(
        store.streams().snapshot(attempt.attempt_id).await.unwrap(),
        expected
    );
}

#[sqlx::test(migrations = false)]
async fn progress_watcher_follows_the_stream_until_the_attempt_ends(pool: PgPool) {
    let store = opened(pool).await;
    let attempt = new_attempt();
    let attempts = store.attempts();
    attempts.create(&attempt, None).await.unwrap();
    attempts
        .transition(
            attempt.attempt_id,
            AttemptState::Queued,
            &TransitionNote::by("q"),
        )
        .await
        .unwrap();
    attempts
        .start(attempt.attempt_id, "worker-a", LEASE)
        .await
        .unwrap();
    let mut watcher = store.watch_progress(attempt.attempt_id).await.unwrap();
    let producer = {
        let store = store.clone();
        let id = attempt.attempt_id;
        tokio::spawn(async move {
            for batch in [0..3, 3..5] {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let events: Vec<ProgressEvent> = batch.map(progress_event).collect();
                store.streams().append_progress(id, &events).await.unwrap();
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
            store
                .attempts()
                .transition(id, AttemptState::Completed, &TransitionNote::by("worker-a"))
                .await
                .unwrap();
        })
    };
    let mut seen = Vec::new();
    loop {
        let page = tokio::time::timeout(
            Duration::from_secs(10),
            watcher.next(seen.last().map(|e: &ProgressEvent| e.seq), 100),
        )
        .await
        .expect("the watcher wakes on appends and on the attempt's end")
        .unwrap();
        if page.is_empty() {
            break;
        }
        seen.extend(page);
    }
    producer.await.unwrap();
    assert_eq!(seen, (0..5).map(progress_event).collect::<Vec<_>>());
}

#[sqlx::test(migrations = false)]
async fn retention_removes_streams_of_finished_attempts_only(pool: PgPool) {
    let store = opened(pool).await;
    let finished = finished_attempt(&store).await;
    let running = new_attempt();
    let attempts = store.attempts();
    attempts.create(&running, None).await.unwrap();
    attempts
        .transition(
            running.attempt_id,
            AttemptState::Queued,
            &TransitionNote::by("q"),
        )
        .await
        .unwrap();
    attempts
        .start(running.attempt_id, "worker-a", LEASE)
        .await
        .unwrap();
    let events: Vec<ProgressEvent> = (0..10).map(progress_event).collect();
    for attempt in [finished, running.attempt_id] {
        store
            .streams()
            .append_progress(attempt, &events)
            .await
            .unwrap();
    }
    let keep_long = crate::streams::Retention {
        finished_for: Duration::from_secs(3600),
    };
    assert_eq!(store.streams().apply_retention(keep_long).await.unwrap(), 0);
    let expire_now = crate::streams::Retention {
        finished_for: Duration::ZERO,
    };
    assert_eq!(
        store.streams().apply_retention(expire_now).await.unwrap(),
        10
    );
    assert!(store.streams().snapshot(finished).await.unwrap().is_empty());
    assert_eq!(
        store
            .streams()
            .snapshot(running.attempt_id)
            .await
            .unwrap()
            .len(),
        10
    );
}

#[sqlx::test(migrations = false)]
async fn incumbents_and_solutions_round_trip(pool: PgPool) {
    let store = opened(pool).await;
    let attempt = finished_attempt(&store).await;
    use pse_model::generated::enums::NativeBackend;
    let solution = Solution {
        solution_id: mint_id(),
        compatibility_stamp: hash(5),
        preparation_identity: hash(6),
        backend: NativeBackend::Ipopt,
        profile_stamp: hash(3),
        data_stamp: hash(4),
        vectors: SeedVectors::Nlp {
            primal: vec![0.1 + 0.2, -1.5e-300, 7.0],
            bounds: Some((vec![0.0; 3], vec![1.0, 2.0, 3.0])),
            rows: None,
            barrier: Some(0.1 * 2.5e-9),
        },
        created_by: Some(attempt),
    };
    store.solutions().put(&solution).await.unwrap();
    let duplicate = store.solutions().put(&solution).await.unwrap_err();
    assert!(matches!(duplicate, OperationsError::Duplicate { .. }));
    let newer = Solution {
        solution_id: mint_id(),
        vectors: SeedVectors::Root {
            primal: vec![9.0, 9.5, f64::MIN_POSITIVE],
        },
        ..solution.clone()
    };
    store.solutions().put(&newer).await.unwrap();
    let highs = Solution {
        solution_id: mint_id(),
        backend: NativeBackend::Highs,
        vectors: SeedVectors::Highs {
            primal: None,
            dual: Some((vec![1.0], vec![2.0, 3.0])),
            basis: Some((vec![1], vec![0, 4])),
        },
        ..solution.clone()
    };
    store.solutions().put(&highs).await.unwrap();
    // The database enforces which vectors a kind carries, as the registry states.
    let malformed = sqlx::query(
        "INSERT INTO pse_ops.solutions (solution_id, compatibility_stamp, preparation_identity, \
             kind, backend, profile_stamp, data_stamp) VALUES ($1, $2, $2, 'root', 'ipopt', $2, $2)",
    )
    .bind(crate::codec::uuid(mint_id()))
    .bind(hash(1).as_bytes().to_vec())
    .execute(store.pool())
    .await
    .unwrap_err();
    assert_eq!(
        malformed
            .as_database_error()
            .and_then(|e| e.code())
            .as_deref(),
        Some("23514")
    );
    // Only an NLP seed carries a barrier, and only a finite positive one.
    for (kind, barrier) in [("root", 1e-9), ("nlp", 0.0), ("nlp", f64::INFINITY)] {
        let refused = sqlx::query(
            "INSERT INTO pse_ops.solutions (solution_id, compatibility_stamp, \
                 preparation_identity, kind, backend, profile_stamp, data_stamp, primal, barrier) \
             VALUES ($1, $2, $2, $3::pse_ops.stored_seed_kind, 'ipopt', $2, $2, \
                 ARRAY[1.0]::double precision[], $4)",
        )
        .bind(crate::codec::uuid(mint_id()))
        .bind(hash(1).as_bytes().to_vec())
        .bind(kind)
        .bind(barrier)
        .execute(store.pool())
        .await
        .unwrap_err();
        assert_eq!(
            refused
                .as_database_error()
                .and_then(|e| e.code())
                .as_deref(),
            Some("23514"),
            "{kind} {barrier}"
        );
    }

    for stored in [&solution, &newer, &highs] {
        assert_eq!(
            store
                .solutions()
                .get(stored.solution_id)
                .await
                .unwrap()
                .map(|s| s.solution)
                .as_ref(),
            Some(stored)
        );
    }
    let seed = store
        .solutions()
        .latest_compatible(&hash(5), &hash(6), NativeBackend::Ipopt)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(seed.solution, newer);
    assert_eq!(
        store
            .solutions()
            .latest_compatible(&hash(5), &hash(6), NativeBackend::Highs)
            .await
            .unwrap()
            .map(|s| s.solution),
        Some(highs)
    );
    assert!(
        store
            .solutions()
            .latest_compatible(&hash(7), &hash(6), NativeBackend::Ipopt)
            .await
            .unwrap()
            .is_none()
    );

    let incumbents = [
        Incumbent {
            seq: 0,
            at: at(0),
            objective: 12.5,
            dual_bound: Some(3.0),
            gap: Some(0.76),
            solution_id: None,
        },
        Incumbent {
            seq: 1,
            at: at(5),
            objective: 10.0,
            dual_bound: None,
            gap: None,
            solution_id: Some(newer.solution_id),
        },
    ];
    assert_eq!(
        store
            .streams()
            .record_incumbents(attempt, &incumbents)
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        store.streams().latest_incumbent(attempt).await.unwrap(),
        Some(incumbents[1].clone())
    );
}

// ----------------------------------------------------------------- catalog --

#[sqlx::test(migrations = false)]
async fn concurrent_head_advance_one_winner(pool: PgPool) {
    let store = opened(pool).await;
    let workspace = workspace(&store).await;
    let base = publish(&store, workspace, None, 0).await;

    let commits: Vec<PublicationCommit> = {
        let mut commits = Vec::new();
        for version in [1, 2] {
            commits.push(PublicationCommit {
                publication_id: mint_id(),
                workspace_id: workspace,
                attempt_id: finished_attempt(&store).await,
                expected_parent: Some(base),
                members: members(version),
            });
        }
        commits
    };
    let catalog = store.catalog();
    let (left, right) = tokio::join!(catalog.commit(&commits[0]), catalog.commit(&commits[1]));
    let results = [left, right];
    let winners: Vec<SemanticId> = results
        .iter()
        .filter_map(|r| match r {
            Ok(Committed::Advanced { publication_id }) => Some(*publication_id),
            _ => None,
        })
        .collect();
    assert_eq!(winners.len(), 1, "{results:?}");
    let loser = results.iter().find_map(|r| r.as_ref().err()).unwrap();
    assert!(
        matches!(
            loser,
            OperationsError::PublicationConflict { expected, current, .. }
                if *expected == Some(base) && *current == Some(winners[0])
        ),
        "{loser:?}"
    );
    assert_eq!(catalog.head(workspace).await.unwrap(), Some(winners[0]));

    // The loser re-prepares against the new head; it never rebases.
    let loser_commit = commits
        .iter()
        .find(|c| c.publication_id != winners[0])
        .unwrap();
    let retried = PublicationCommit {
        expected_parent: Some(winners[0]),
        ..loser_commit.clone()
    };
    assert_eq!(
        catalog.commit(&retried).await.unwrap(),
        Committed::Advanced {
            publication_id: retried.publication_id
        }
    );
    assert_eq!(
        catalog.head(workspace).await.unwrap(),
        Some(retried.publication_id)
    );
    assert_eq!(
        catalog.members(retried.publication_id).await.unwrap(),
        retried.members
    );
}

#[sqlx::test(migrations = false)]
async fn commit_is_idempotent_per_attempt_and_settles(pool: PgPool) {
    let store = opened(pool).await;
    let catalog = store.catalog();
    let workspace = workspace(&store).await;
    let attempt = finished_attempt(&store).await;
    let commit = PublicationCommit {
        publication_id: mint_id(),
        workspace_id: workspace,
        attempt_id: attempt,
        expected_parent: None,
        members: members(0),
    };
    assert_eq!(
        catalog.settle(attempt).await.unwrap(),
        Settlement::ProvedNoncommit
    );
    catalog.commit(&commit).await.unwrap();
    // A retry after a lost acknowledgement is idempotent, even with a stale parent.
    assert_eq!(
        catalog.commit(&commit).await.unwrap(),
        Committed::AlreadyCommitted {
            publication_id: commit.publication_id
        }
    );
    assert_eq!(
        catalog.settle(attempt).await.unwrap(),
        Settlement::Committed {
            publication_id: commit.publication_id
        }
    );

    // Only finished attempts are published.
    let running = new_attempt();
    store.attempts().create(&running, None).await.unwrap();
    let refused = catalog
        .commit(&PublicationCommit {
            publication_id: mint_id(),
            attempt_id: running.attempt_id,
            expected_parent: Some(commit.publication_id),
            ..commit.clone()
        })
        .await
        .unwrap_err();
    assert!(
        matches!(refused, OperationsError::InvalidRequest { .. }),
        "{refused:?}"
    );
}

#[sqlx::test(migrations = false)]
async fn maintenance_waits_for_reader_leases(pool: PgPool) {
    let store = opened(pool).await;
    let catalog = store.catalog();
    let workspace = workspace(&store).await;
    let old = publish(&store, workspace, None, 0).await;
    let head = publish(&store, workspace, Some(old), 1).await;

    // A reader resolves the head, then another reads the old publication explicitly.
    let head_lease = catalog
        .acquire_reader_lease(ReadTarget::Head(workspace), "reader-1", LEASE)
        .await
        .unwrap();
    assert_eq!(head_lease.publication_id, head);
    let lease = catalog
        .acquire_reader_lease(ReadTarget::Publication(old), "reader-2", LEASE)
        .await
        .unwrap();
    assert_eq!(lease.members, members(0));

    // The head is protected from retention.
    let protected = catalog.mark_expiring(workspace, head).await.unwrap_err();
    assert!(matches!(
        protected,
        OperationsError::ProtectedPublication { .. }
    ));

    catalog.mark_expiring(workspace, old).await.unwrap();
    // No new lease once expiring.
    let refused = catalog
        .acquire_reader_lease(ReadTarget::Publication(old), "reader-3", LEASE)
        .await
        .unwrap_err();
    assert!(
        matches!(refused, OperationsError::PublicationRetiring { .. }),
        "{refused:?}"
    );
    // Maintenance waits while the existing lease is live.
    let busy = catalog
        .wait_for_readers(old, Duration::from_millis(10), Duration::from_millis(50))
        .await
        .unwrap_err();
    assert!(
        matches!(busy, OperationsError::ReadersActive { active: 1, .. }),
        "{busy:?}"
    );
    let busy = catalog.mark_deleted(workspace, old).await.unwrap_err();
    assert!(matches!(busy, OperationsError::ReadersActive { .. }));
    // Versions a live lease reads stay protected.
    let protected = catalog.protected_versions(workspace).await.unwrap();
    assert!(protected.contains(&ProtectedVersion {
        table_uri: "file:///tmp/pse-workspace/runs/".to_owned(),
        delta_version: 0,
    }));

    // The reader finishes; maintenance proceeds.
    let release = async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(catalog.release_reader_lease(lease.lease_id).await.unwrap());
    };
    let wait = catalog.wait_for_readers(old, Duration::from_millis(10), Duration::from_secs(10));
    let ((), waited) = tokio::join!(release, wait);
    waited.unwrap();
    assert!(!catalog.release_reader_lease(lease.lease_id).await.unwrap());
    catalog.mark_deleted(workspace, old).await.unwrap();
    catalog.mark_deleted(workspace, old).await.unwrap();

    let protected = catalog.protected_versions(workspace).await.unwrap();
    assert_eq!(
        protected,
        [
            ProtectedVersion {
                table_uri: "file:///tmp/pse-workspace/metrics/".to_owned(),
                delta_version: 1,
            },
            ProtectedVersion {
                table_uri: "file:///tmp/pse-workspace/runs/".to_owned(),
                delta_version: 1,
            },
        ]
    );
    assert!(
        catalog
            .release_reader_lease(head_lease.lease_id)
            .await
            .unwrap()
    );
}
