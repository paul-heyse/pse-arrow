// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Store tests against real, isolated PostgreSQL 18 databases: each test creates its own
//! database with [`TestDatabase`] on the server `PSE_DATABASE_URL` names (else the
//! development default), with the registry-generated schema created by `Store::open`, and
//! drops it when it passes. Raw SQL runs on a dedicated [`crate::testing::Session`].

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use pse_ids::{ContentHash, SemanticId};

use crate::attempts::{AttemptId, AttemptKind, NewAttempt, TerminationCode, TransitionNote};
use crate::cancellation::CancelOutcome;
use crate::catalog::{
    Committed, MemberDescriptor, MemberDescriptorSelection, NewIntent, NewWorkspace,
    ProtectedRange, PublicationCommit, PublicationId, PublicationKind, ReadTarget, RetentionReason,
    SettleRequest, Settlement, VersionWindow, WorkspaceId,
};
use crate::jobs::{Enqueued, Finished, JobId, JobOutcome, JobState, NewJob, RetryPolicy};
use crate::lifecycle::AttemptState;
use crate::solutions::{NewSolution, RuntimeOperationalSolutionsRow, SeedVectors};
use crate::streams::{ProgressEvent, ProgressValue, RuntimeOperationalIncumbentsRow};
use crate::testing::TestDatabase;
use crate::{InvariantKind, Opened, OperationsError, SchemaStatus, Store, mint_id};

pub(crate) const LEASE: Duration = Duration::from_secs(30);

/// An identity as a SQL literal, for test-authored statements.
fn lit(id: impl std::fmt::Display) -> String {
    format!("'{id}'::uuid")
}

/// Whether `error` is an invariant violation of `kind`, named `constraint` in `table`.
fn violates(
    error: &OperationsError,
    table: Option<&str>,
    constraint: &str,
    kind: InvariantKind,
) -> bool {
    matches!(
        error,
        OperationsError::InvariantViolation { table: t, constraint: Some(c), kind: k, .. }
            if t.as_deref() == table && c == constraint && *k == kind
    )
}

pub(crate) fn hash(byte: u8) -> ContentHash {
    ContentHash::from_bytes([byte; 32])
}

pub(crate) fn new_attempt() -> NewAttempt {
    NewAttempt {
        attempt_id: mint_id(),
        run_id: mint_id(),
        kind: AttemptKind::Simulation,
        request_identity: hash(1),
        preparation_identity: Some(hash(2)),
        parent_attempt: None,
    }
}

pub(crate) fn new_job(key: &str, retry: RetryPolicy) -> NewJob {
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
async fn finished_attempt(store: &Store) -> AttemptId {
    finished_try(store, None).await
}

/// [`finished_attempt`], as the retry of `parent` when one is given.
async fn finished_try(store: &Store, parent: Option<AttemptId>) -> AttemptId {
    let attempts = store.attempts();
    let attempt = NewAttempt {
        parent_attempt: parent,
        ..new_attempt()
    };
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

/// A registered workspace and its root.
pub(crate) struct Space {
    pub(crate) id: WorkspaceId,
    pub(crate) root: String,
}

pub(crate) async fn space(store: &Store) -> Space {
    let workspace = NewWorkspace {
        workspace_id: mint_id(),
        name: format!("ws-{}", mint_id::<SemanticId>()),
        root_uri: format!("file:///tmp/pse-workspace-{}/", mint_id::<SemanticId>()),
    };
    let row = store
        .catalog()
        .register_workspace(&workspace)
        .await
        .unwrap();
    assert_eq!(row.maintenance_epoch, 0);
    Space {
        id: workspace.workspace_id,
        root: workspace.root_uri,
    }
}

/// A member table under an intent's prefix.
fn table(intent: &NewIntent, name: &str) -> String {
    format!("{}runtime/{name}/", intent.member_prefix)
}

pub(crate) fn member(table_uri: &str, name: &str, version: i64) -> MemberDescriptor {
    MemberDescriptor {
        catalog_name: "artifact".to_owned(),
        schema_name: "runtime".to_owned(),
        table_name: name.to_owned(),
        relation_id: SemanticId::from_bytes([7; 16]),
        relation_version: 1,
        contract_fingerprint: hash(9),
        table_uri: table_uri.to_owned(),
        delta_version: version,
        selection: MemberDescriptorSelection::from_full(),
    }
}

/// Members in the catalog's order.
fn sorted(members: &[MemberDescriptor]) -> Vec<MemberDescriptor> {
    let mut members = members.to_vec();
    members.sort_by(|l, r| {
        (&l.catalog_name, &l.schema_name, &l.table_name).cmp(&(
            &r.catalog_name,
            &r.schema_name,
            &r.table_name,
        ))
    });
    members
}

/// Whether the exact version of a member is protected by some range.
fn protects(ranges: &[ProtectedRange], member: &MemberDescriptor) -> bool {
    ranges.iter().any(|range| {
        range.covers(&member.table_uri)
            && range.from_version <= member.delta_version
            && member.delta_version <= range.through_version
    })
}

/// A registered intent for a new finished attempt.
async fn intent(store: &Store, space: &Space) -> NewIntent {
    let attempt = finished_attempt(store).await;
    let publication_id: PublicationId = mint_id();
    let intent = NewIntent {
        publication_id,
        workspace_id: space.id,
        attempt_id: attempt,
        member_prefix: format!("{}members/{attempt}/{publication_id}/", space.root),
    };
    store.catalog().register_intent(&intent).await.unwrap();
    intent
}

/// The commit of an intent writing two members under its prefix.
fn commit_of(intent: &NewIntent, parent: Option<PublicationId>) -> PublicationCommit {
    PublicationCommit {
        publication_id: intent.publication_id,
        workspace_id: intent.workspace_id,
        attempt_id: intent.attempt_id,
        expected_parent: parent,
        kind: PublicationKind::Run,
        members: vec![
            member(&table(intent, "runs"), "runs", 1),
            member(&table(intent, "metrics"), "metrics", 1),
        ],
        inputs: Vec::new(),
        windows: Vec::new(),
    }
}

/// Register and commit a publication of two members.
async fn publish(
    store: &Store,
    space: &Space,
    parent: Option<PublicationId>,
) -> (NewIntent, PublicationCommit) {
    let intent = intent(store, space).await;
    let commit = commit_of(&intent, parent);
    assert_eq!(
        store.catalog().commit(&commit).await.unwrap(),
        Committed::Advanced {
            publication_id: intent.publication_id
        }
    );
    (intent, commit)
}

// ------------------------------------------------------------------- schema --

#[tokio::test]
async fn generated_schema_creates_empty_store() {
    let database = TestDatabase::empty().await.unwrap();
    let store = database.store().clone();
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
    let session = database.session().await.unwrap();
    let tables: Vec<String> = session
        .texts(
            "SELECT table_name FROM information_schema.tables \
             WHERE table_schema = 'pse_ops' ORDER BY table_name",
        )
        .await
        .unwrap()
        .into_iter()
        .filter_map(|row| row.into_iter().next().flatten())
        .collect();
    assert_eq!(tables, expected);
    for table in &expected {
        let rows = session
            .count(&format!("SELECT count(*) FROM pse_ops.\"{table}\""))
            .await
            .unwrap();
        assert_eq!(rows, 0, "{table} is not empty");
    }
    let recorded = session
        .texts(
            "SELECT obj_description(oid, 'pg_namespace') FROM pg_namespace \
             WHERE nspname = 'pse_ops'",
        )
        .await
        .unwrap();
    assert_eq!(
        recorded,
        [[Some(format!(
            "pse.ops.schema.v1 {}",
            Store::expected_schema()
        ))]]
    );
    let server = store.server().await.unwrap();
    assert!(server.is_supported(), "{server:?}");
    drop(session);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn store_schema_mismatch_refused() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let session = database.session().await.unwrap();
    let other = "0".repeat(64);
    session
        .execute(&format!(
            "COMMENT ON SCHEMA pse_ops IS 'pse.ops.schema.v1 {other}'"
        ))
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
    // A schema without a record (one this build did not create) is refused the same way,
    // and never reset implicitly.
    session
        .execute("COMMENT ON SCHEMA pse_ops IS NULL")
        .await
        .unwrap();
    assert!(matches!(
        store.open().await.unwrap_err(),
        OperationsError::SchemaMismatch { recorded: None, .. }
    ));
    // The explicit reset recreates it from this build.
    assert_eq!(store.reset().await.unwrap(), Opened::Created);
    assert_eq!(store.schema_status().await.unwrap(), SchemaStatus::Current);
    drop(session);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn registry_enums_are_postgres_enums() {
    let database = TestDatabase::create().await.unwrap();
    let session = database.session().await.unwrap();
    let registry = pse_schema::registry().unwrap();
    let mut checked = BTreeSet::new();
    for (table, spec) in pse_schema::store::relations(registry) {
        for column in &spec.columns {
            let Some(name) = column.enum_name() else {
                continue;
            };
            let found = session
                .texts(&format!(
                    "SELECT udt_schema, udt_name FROM information_schema.columns \
                     WHERE table_schema = 'pse_ops' AND table_name = '{table}' \
                       AND column_name = '{}'",
                    column.name()
                ))
                .await
                .unwrap();
            let [row] = found.as_slice() else {
                panic!("{table}.{} has no column type", column.name());
            };
            let (schema, udt) = (row[0].clone().unwrap(), row[1].clone().unwrap());
            assert_eq!(schema, "pse_ops", "{table}.{}", column.name());
            let labels: Vec<String> = session
                .texts(&format!(
                    "SELECT e.enumlabel FROM pg_enum e \
                     JOIN pg_type t ON t.oid = e.enumtypid \
                     JOIN pg_namespace n ON n.oid = t.typnamespace \
                     WHERE n.nspname = 'pse_ops' AND t.typname = '{udt}' \
                     ORDER BY e.enumsortorder"
                ))
                .await
                .unwrap()
                .into_iter()
                .filter_map(|row| row.into_iter().next().flatten())
                .collect();
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
    for name in [
        "AttemptState",
        "JobState",
        "TerminationClass",
        "RetentionPhase",
        "SettlementOutcome",
        "PublicationMemberRole",
        "PublicationKind",
        "MemberSelectionKind",
    ] {
        assert!(checked.contains(name), "{name} is not a store column type");
    }
    drop(session);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn misspelled_enum_literal_fails_prepare() {
    let database = TestDatabase::create().await.unwrap();
    let session = database.session().await.unwrap();
    // A literal compared with an ENUM column is resolved when the statement is prepared.
    let error = session
        .execute(
            "PREPARE misspelled AS SELECT attempt_id FROM pse_ops.attempts WHERE state = 'runing'",
        )
        .await
        .unwrap_err();
    assert!(
        matches!(error, OperationsError::Internal { .. }) && error.to_string().contains("runing"),
        "{error:?}"
    );
    session
        .execute(
            "PREPARE spelled AS SELECT attempt_id FROM pse_ops.attempts WHERE state = 'running'",
        )
        .await
        .unwrap();
    drop(session);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn row_invariants_generated_and_enforced() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let session = database.session().await.unwrap();
    let registry = pse_schema::registry().unwrap();
    // Every declared row check is a named CHECK constraint of its table.
    let constraints: BTreeSet<String> = session
        .texts(
            "SELECT c.conname FROM pg_constraint c \
             JOIN pg_namespace n ON n.oid = c.connamespace WHERE n.nspname = 'pse_ops'",
        )
        .await
        .unwrap()
        .into_iter()
        .filter_map(|row| row.into_iter().next().flatten())
        .collect();
    for (table, spec) in pse_schema::store::relations(registry) {
        for name in spec.checks.keys() {
            let constraint = format!("{table}_{name}_check");
            assert!(
                constraints.contains(&constraint),
                "{constraint} is not generated"
            );
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
    let id = lit(attempt.attempt_id);
    // A running attempt holds a lease (the former multi-column CHECK); the violation
    // arrives as a typed invariant naming the registry rule.
    let error = session
        .execute(&format!(
            "UPDATE pse_ops.attempts SET state = 'running', worker = 'w' WHERE attempt_id = {id}"
        ))
        .await
        .unwrap_err();
    assert!(
        violates(
            &error,
            Some("attempts"),
            "running_holds_lease",
            InvariantKind::Check
        ),
        "{error:?}"
    );
    assert!(!error.is_retryable());
    // Exactly the typed termination column the class selects is present (X4).
    let error = session
        .execute(&format!(
            "UPDATE pse_ops.attempts SET termination_class = 'native' WHERE attempt_id = {id}"
        ))
        .await
        .unwrap_err();
    assert!(
        violates(
            &error,
            Some("attempts"),
            "one_termination",
            InvariantKind::Check
        ),
        "{error:?}"
    );
    // A finite field domain refuses NaN and infinity in PostgreSQL too.
    let error = session
        .execute(&format!(
            "INSERT INTO pse_ops.progress_events (attempt_id, seq, step, at, elapsed_seconds, phase) \
             VALUES ({id}, 0, 0, now(), 'Infinity', 'phase')"
        ))
        .await
        .unwrap_err();
    assert!(
        violates(
            &error,
            Some("progress_events"),
            "elapsed_seconds_finite",
            InvariantKind::Check
        ),
        "{error:?}"
    );
    // A composite reference is enforced, and nothing cascades.
    let error = session
        .execute(&format!(
            "INSERT INTO pse_ops.progress_values (attempt_id, seq, name, kind, \"integer\") \
             VALUES ({id}, 7, 'n', 'integer', 1)"
        ))
        .await
        .unwrap_err();
    assert!(
        violates(
            &error,
            Some("progress_values"),
            "progress_event",
            InvariantKind::ForeignKey
        ),
        "{error:?}"
    );
    // Content hashes are exactly 32 bytes, through their domain.
    let error = session
        .execute(&format!(
            "INSERT INTO pse_ops.source_bundles (bundle_hash, manifest) \
             VALUES ('\\x{}'::bytea, '{{}}')",
            "00".repeat(31)
        ))
        .await
        .unwrap_err();
    assert!(
        matches!(
            &error,
            OperationsError::InvariantViolation { kind: InvariantKind::Check, constraint: Some(c), .. }
                if c == "content_hash_width"
        ),
        "{error:?}"
    );
    // The seed-vector rule is a registry row check.
    let stamp = format!("'\\x{}'::bytea", "01".repeat(32));
    let error = session
        .execute(&format!(
            "INSERT INTO pse_ops.solutions (solution_id, compatibility_stamp, preparation_identity, \
                 kind, backend, profile_stamp, data_stamp) \
             VALUES ({}, {stamp}, {stamp}, 'root', 'kinsol', {stamp}, {stamp})",
            lit(mint_id::<SemanticId>())
        ))
        .await
        .unwrap_err();
    assert!(
        violates(&error, Some("solutions"), "vectors", InvariantKind::Check),
        "{error:?}"
    );
    drop(session);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn transition_history_is_append_only() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let session = database.session().await.unwrap();
    let attempt = new_attempt();
    store.attempts().create(&attempt, None).await.unwrap();
    // SQLSTATE 42501: the owner's own UPDATE and DELETE privileges are revoked.
    for sql in [
        "UPDATE pse_ops.attempt_transitions SET reason = 'rewritten'",
        "DELETE FROM pse_ops.attempt_transitions",
    ] {
        let error = session.execute(sql).await.unwrap_err();
        assert!(
            matches!(error, OperationsError::Internal { .. })
                && error.to_string().contains("permission denied"),
            "{error:?}"
        );
    }
    assert_eq!(
        store
            .attempts()
            .history(attempt.attempt_id)
            .await
            .unwrap()
            .len(),
        1
    );
    drop(session);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn sqlstate_errors_map_to_typed_variants() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();

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
    let holder = database.session().await.unwrap();
    let probe = database.session().await.unwrap();
    let lock = format!(
        "SELECT 1 FROM pse_ops.attempts WHERE attempt_id = {} FOR UPDATE",
        lit(attempt.attempt_id)
    );
    holder.execute(&format!("BEGIN; {lock}")).await.unwrap();
    let nowait = probe.execute(&format!("{lock} NOWAIT")).await.unwrap_err();
    assert!(
        matches!(nowait, OperationsError::LockUnavailable { .. }),
        "{nowait:?}"
    );
    assert!(nowait.is_retryable());
    holder.execute("ROLLBACK").await.unwrap();

    // 57014: a statement timeout cancels the statement.
    probe
        .execute("SET statement_timeout = '20ms'")
        .await
        .unwrap();
    let cancelled = probe.execute("SELECT pg_sleep(2)").await.unwrap_err();
    assert!(
        matches!(cancelled, OperationsError::Cancelled { .. }),
        "{cancelled:?}"
    );
    probe.execute("RESET statement_timeout").await.unwrap();

    // 40001: write skew between two serializable transactions. The server may refuse
    // the second insert or the second commit; either way exactly one step fails.
    let (first, second) = (holder, probe);
    let begin = "BEGIN ISOLATION LEVEL SERIALIZABLE; SELECT count(*) FROM pse_ops.workspaces";
    let write = |name: &str| {
        format!(
            "INSERT INTO pse_ops.workspaces (workspace_id, name, root_uri, maintenance_epoch) \
             VALUES ({}, '{name}', 'file:///x/{name}/', 0)",
            lit(mint_id::<SemanticId>())
        )
    };
    first.execute(begin).await.unwrap();
    second.execute(begin).await.unwrap();
    first.execute(&write("one")).await.unwrap();
    let second_write = second.execute(&write("two")).await;
    first.execute("COMMIT").await.unwrap();
    let conflict = match second_write {
        Err(error) => error,
        Ok(_) => second.execute("COMMIT").await.unwrap_err(),
    };
    assert!(
        matches!(conflict, OperationsError::Retryable { .. }),
        "{conflict:?}"
    );
    assert!(conflict.is_retryable());
    second.execute("ROLLBACK").await.unwrap();

    // 40P01: two transactions lock two rows in opposite order.
    let other = new_attempt();
    store.attempts().create(&other, None).await.unwrap();
    let lock = |id: AttemptId| {
        format!(
            "SELECT 1 FROM pse_ops.attempts WHERE attempt_id = {} FOR UPDATE",
            lit(id)
        )
    };
    let (left, right) = (first, second);
    left.execute(&format!("BEGIN; {}", lock(attempt.attempt_id)))
        .await
        .unwrap();
    right
        .execute(&format!("BEGIN; {}", lock(other.attempt_id)))
        .await
        .unwrap();
    let (lock_other, lock_attempt) = (lock(other.attempt_id), lock(attempt.attempt_id));
    let (l, r) = tokio::join!(left.execute(&lock_other), right.execute(&lock_attempt));
    let failures: Vec<OperationsError> = [l.err(), r.err()].into_iter().flatten().collect();
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(
        matches!(failures[0], OperationsError::Retryable { .. }),
        "{failures:?}"
    );
    left.execute("ROLLBACK").await.unwrap();
    right.execute("ROLLBACK").await.unwrap();
    drop((left, right));
    database.remove().await.unwrap();
}

#[tokio::test]
async fn dropped_statement_does_not_block_the_pool() {
    let database = TestDatabase::create().await.unwrap();
    // One pooled connection: the next operation gets the very connection whose
    // statement was abandoned, unless it is replaced.
    let options = crate::StoreOptions {
        max_connections: 1,
        ..crate::StoreOptions::for_tests()
    };
    let store = Store::connect_with(database.url(), &options).await.unwrap();
    let lock = database.session().await.unwrap();
    lock.execute("BEGIN; LOCK TABLE pse_ops.attempts IN ACCESS EXCLUSIVE MODE")
        .await
        .unwrap();
    // A reader cancelled while it waits on the lock: its statement still runs on the
    // server when the future is dropped.
    let abandoned =
        tokio::time::timeout(Duration::from_millis(200), store.attempts().get(mint_id())).await;
    assert!(abandoned.is_err(), "the read waits on the lock");
    // The pool does not hand that connection out again while it is busy.
    let server = tokio::time::timeout(Duration::from_secs(10), store.server())
        .await
        .expect("a busy connection is replaced, not waited for")
        .unwrap();
    assert!(server.is_supported());
    lock.execute("ROLLBACK").await.unwrap();
    store.close();
    drop(lock);
    database.remove().await.unwrap();
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

#[tokio::test]
async fn illegal_transition_rejected() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
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
                    code: TerminationCode::Rule(
                        crate::attempts::DiagnosticCode::SolveEvaluationError,
                    ),
                    detail: Some(serde_json::json!({ "row": 3 })),
                }),
        )
        .await
        .unwrap();
    assert_eq!(done.state, AttemptState::Failed);
    assert_eq!(done.lease_expires_at, None);
    assert!(done.finished_at.is_some());
    assert_eq!(
        TerminationCode::of(&done).unwrap(),
        Some(TerminationCode::Rule(
            crate::attempts::DiagnosticCode::SolveEvaluationError
        ))
    );
    let row = &done;
    assert_eq!(
        row.termination_class,
        Some(crate::attempts::TerminationClass::Rule)
    );
    assert_eq!(
        row.termination_rule,
        Some(crate::attempts::DiagnosticCode::SolveEvaluationError)
    );
    // The document is stored as jsonb and read back as its text; its value is unchanged.
    let detail: serde_json::Value =
        serde_json::from_str(row.termination_detail.as_deref().unwrap()).unwrap();
    assert_eq!(detail, serde_json::json!({ "row": 3 }));
    let history = attempts.history(attempt.attempt_id).await.unwrap();
    let steps: Vec<(Option<AttemptState>, AttemptState)> =
        history.iter().map(|t| (t.from_state, t.to_state)).collect();
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
    database.remove().await.unwrap();
}

#[tokio::test]
async fn lease_expiry_marks_stale() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
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
    database.remove().await.unwrap();
}

#[tokio::test]
async fn heartbeat_extends_lease_and_returns_cancellation() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
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
    assert!(Some(ack.lease_expires_at.timestamp_micros()) > started.lease_expires_at);
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
    database.remove().await.unwrap();
}

// -------------------------------------------------------------------- jobs --

#[tokio::test]
async fn enqueue_is_idempotent_per_key() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
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
    database.remove().await.unwrap();
}

#[tokio::test]
async fn two_workers_never_claim_same_job() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
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
    let holder = database.session().await.unwrap();
    holder
        .execute(&format!(
            "BEGIN; SELECT 1 FROM pse_ops.jobs WHERE job_id = {} FOR UPDATE",
            lit(first)
        ))
        .await
        .unwrap();
    let skipped = store
        .jobs()
        .claim("worker-b", LEASE)
        .await
        .unwrap()
        .unwrap();
    assert_ne!(skipped.job_id, first);
    holder.execute("ROLLBACK").await.unwrap();
    drop(holder);

    async fn drain(store: &Store, worker: &str) -> Vec<JobId> {
        let mut claimed = Vec::new();
        while let Some(job) = store.jobs().claim(worker, LEASE).await.unwrap() {
            assert_eq!(job.try_number, 1);
            claimed.push(job.job_id);
            tokio::task::yield_now().await;
        }
        claimed
    }
    let (a, b) = tokio::join!(drain(&store, "worker-a"), drain(&store, "worker-b"));
    let mut all: Vec<JobId> = a.iter().chain(&b).copied().collect();
    all.push(skipped.job_id);
    let unique: BTreeSet<JobId> = all.iter().copied().collect();
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
    database.remove().await.unwrap();
}

#[tokio::test]
async fn expired_lease_requeues_as_new_attempt() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
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
    assert_eq!(first.parent_attempt, None);
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
        .map(|t| t.to_state)
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
    // The claim names the try it supersedes: a resumed solve starts from its incumbents.
    assert_eq!(second.parent_attempt, Some(first.attempt_id));
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
    database.remove().await.unwrap();
}

#[tokio::test]
async fn attempt_sweep_then_job_requeue_compose() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
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
    database.remove().await.unwrap();
}

#[tokio::test]
async fn failed_try_retries_under_policy_and_completion_ends_job() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
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
        members: Vec::new(),
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

    database
        .session()
        .await
        .unwrap()
        .execute(&format!(
            "UPDATE pse_ops.jobs SET available_at = now() WHERE job_id = {}",
            lit(job.job_id())
        ))
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
                members: Vec::new(),
            },
        )
        .await
        .unwrap();
    assert_eq!(done, Finished::Ended(JobState::Completed));
    assert_eq!(
        store.attempts().get(attempt_id).await.unwrap().state,
        AttemptState::Completed
    );
    database.remove().await.unwrap();
}

// ------------------------------------------------------------ cancellation --

#[tokio::test]
async fn cancel_notify_stops_running_job() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
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
                    members: Vec::new(),
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
    database.remove().await.unwrap();
}

#[tokio::test]
async fn cancel_observed_after_listener_reconnect() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
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
    let session = database.session().await.unwrap();
    let disrupt = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        session
            .execute(&format!(
                "UPDATE pse_ops.attempts SET cancel_requested = true, cancel_requested_at = now() \
                 WHERE attempt_id = {}",
                lit(claimed.attempt_id)
            ))
            .await
            .unwrap();
        assert_eq!(terminate_listener(&session).await, 1);
    };
    tokio::join!(waiting, disrupt);
    drop(session);
    database.remove().await.unwrap();
}

/// Terminate the store listener's connection; returns how many backends were terminated.
async fn terminate_listener(session: &crate::testing::Session) -> i64 {
    session
        .count(&format!(
            "SELECT count(*) FILTER (WHERE pg_terminate_backend(pid)) FROM pg_stat_activity \
             WHERE datname = current_database() AND application_name = '{}'",
            crate::LISTENER_APPLICATION
        ))
        .await
        .unwrap()
}

/// The next cancellation notification, skipping resynchronizations and other channels.
async fn next_cancel(events: &mut crate::listener::Subscription) -> AttemptId {
    use crate::listener::{Channel, Event};
    loop {
        let event = tokio::time::timeout(Duration::from_secs(10), events.next())
            .await
            .expect("a notification arrives")
            .unwrap();
        if let Event::Notification {
            channel: Channel::Cancel,
            attempt,
        } = event
        {
            return attempt;
        }
    }
}

#[tokio::test]
async fn listener_resyncs_after_connection_loss() {
    use crate::listener::Event;
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let session = database.session().await.unwrap();
    let mut events = store.subscribe().await.unwrap();
    // One LISTEN has taken effect, on one dedicated, named connection.
    assert_eq!(store.listener().unwrap().generation(), 1);
    let first = new_attempt();
    store.attempts().create(&first, None).await.unwrap();
    store
        .request_cancel(first.attempt_id, "user")
        .await
        .unwrap();
    assert_eq!(next_cancel(&mut events).await, first.attempt_id);

    // The connection is lost: the listener reconnects, LISTENs again and tells every
    // subscriber to resynchronize, since notifications sent meanwhile are gone.
    assert_eq!(terminate_listener(&session).await, 1);
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if events.next().await.unwrap() == Event::Resync {
                break;
            }
        }
    })
    .await
    .expect("a resynchronization follows the reconnection");
    assert_eq!(store.listener().unwrap().generation(), 2);
    // Notifications flow again, on the new connection.
    let second = new_attempt();
    store.attempts().create(&second, None).await.unwrap();
    store
        .request_cancel(second.attempt_id, "user")
        .await
        .unwrap();
    assert_eq!(next_cancel(&mut events).await, second.attempt_id);

    // A stopped listener is reported as unavailable, never as silence.
    store.listener().unwrap().stop();
    let stopped = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match events.next().await {
                Ok(_) => {}
                Err(error) => break error,
            }
        }
    })
    .await
    .expect("the subscription ends with the listener");
    assert!(
        matches!(stopped, OperationsError::Unavailable { .. }),
        "{stopped:?}"
    );
    drop(session);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn cancel_before_start_cancels_job() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
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
    database.remove().await.unwrap();
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

#[tokio::test]
async fn progress_batch_insert_roundtrip() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let attempt = new_attempt();
    store.attempts().create(&attempt, None).await.unwrap();
    let events: Vec<ProgressEvent> = (0..500).map(progress_event).collect();
    assert_eq!(
        events[7].values["step_norm"],
        ProgressValue::Unavailable(
            pse_model::generated::enums::EvidenceUnavailableReason::Nonfinite
        )
    );

    let mut events_seen = store.subscribe().await.unwrap();
    assert_eq!(
        store
            .streams()
            .append_progress(attempt.attempt_id, &events)
            .await
            .unwrap(),
        500
    );
    // The batch notified the progress channel, naming its attempt.
    let notified = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let crate::listener::Event::Notification {
                channel: crate::listener::Channel::Progress,
                attempt,
            } = events_seen.next().await.unwrap()
            {
                break attempt;
            }
        }
    })
    .await
    .unwrap();
    assert_eq!(notified, attempt.attempt_id);
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
    database.remove().await.unwrap();
}

#[tokio::test]
async fn progress_watcher_follows_the_stream_until_the_attempt_ends() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
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
    let incumbent = |seq: i64| RuntimeOperationalIncumbentsRow {
        attempt_id: attempt.attempt_id,
        seq,
        step: 0,
        at: at(seq).timestamp_micros(),
        elapsed_seconds: 0.5,
        phase: "scip.incumbent".into(),
        objective: if seq == 0 { 10.0 } else { 9.0 },
        dual_bound: Some(1.0),
        gap: None,
        nodes: Some(seq),
        seconds: Some(0.5),
        solution_id: None,
    };
    let producer = {
        let store = store.clone();
        let id = attempt.attempt_id;
        let incumbents: Vec<_> = (0..2).map(incumbent).collect();
        tokio::spawn(async move {
            for batch in [0..3, 3..5] {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let events: Vec<ProgressEvent> = batch.map(progress_event).collect();
                store.streams().append_progress(id, &events).await.unwrap();
            }
            // An incumbent alone wakes the watcher too.
            for one in incumbents.chunks(1) {
                tokio::time::sleep(Duration::from_millis(50)).await;
                store.streams().record_incumbents(one, &[]).await.unwrap();
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
            store
                .attempts()
                .transition(id, AttemptState::Completed, &TransitionNote::by("worker-a"))
                .await
                .unwrap();
        })
    };
    let (mut seen, mut incumbents) = (Vec::new(), Vec::new());
    let mut position = crate::streams::StreamPosition::default();
    loop {
        let page = tokio::time::timeout(Duration::from_secs(10), watcher.next(position, 100))
            .await
            .expect("the watcher wakes on appends and on the attempt's end")
            .unwrap();
        if page.is_empty() {
            break;
        }
        position = page.advance(position);
        seen.extend(page.progress);
        incumbents.extend(page.incumbents);
    }
    producer.await.unwrap();
    assert_eq!(seen, (0..5).map(progress_event).collect::<Vec<_>>());
    assert_eq!(incumbents.len(), 2);
    for (read, written) in incumbents.iter().zip((0..2).map(incumbent)) {
        assert!(
            pse_model::SemanticEq::semantic_eq(read, &written),
            "{read:?}"
        );
    }
    assert_eq!(
        position,
        crate::streams::StreamPosition {
            progress: Some(4),
            incumbents: Some(1)
        }
    );
    database.remove().await.unwrap();
}

#[tokio::test]
async fn retention_removes_streams_of_finished_attempts_only() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
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
    database.remove().await.unwrap();
}

#[tokio::test]
async fn incumbents_and_solutions_round_trip() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let attempt = finished_attempt(&store).await;
    use pse_model::generated::enums::NativeBackend;
    let solution = NewSolution {
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
    let put = store.solutions().put(&solution).await.unwrap();
    assert!(stores(&put, &solution), "{put:?}");
    let duplicate = store.solutions().put(&solution).await.unwrap_err();
    assert!(matches!(duplicate, OperationsError::Duplicate { .. }));
    let newer = NewSolution {
        solution_id: mint_id(),
        vectors: SeedVectors::Root {
            primal: vec![9.0, 9.5, f64::MIN_POSITIVE],
        },
        ..solution.clone()
    };
    store.solutions().put(&newer).await.unwrap();
    let highs = NewSolution {
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
    let session = database.session().await.unwrap();
    let stamp = format!("'\\x{}'::bytea", "01".repeat(32));
    let malformed = session
        .execute(&format!(
            "INSERT INTO pse_ops.solutions (solution_id, compatibility_stamp, preparation_identity, \
                 kind, backend, profile_stamp, data_stamp) \
             VALUES ({}, {stamp}, {stamp}, 'root', 'ipopt', {stamp}, {stamp})",
            lit(mint_id::<SemanticId>())
        ))
        .await
        .unwrap_err();
    assert!(
        violates(
            &malformed,
            Some("solutions"),
            "vectors",
            InvariantKind::Check
        ),
        "{malformed:?}"
    );
    // Only an NLP seed carries a barrier, and only a finite positive one.
    for (kind, barrier, rule) in [
        ("root", "1e-9", "vectors"),
        ("nlp", "0", "barrier_positive"),
        ("nlp", "'Infinity'", "barrier_finite"),
    ] {
        let refused = session
            .execute(&format!(
                "INSERT INTO pse_ops.solutions (solution_id, compatibility_stamp, \
                     preparation_identity, kind, backend, profile_stamp, data_stamp, primal, barrier) \
                 VALUES ({}, {stamp}, {stamp}, '{kind}', 'ipopt', {stamp}, {stamp}, \
                     ARRAY[1.0]::double precision[], {barrier})",
                lit(mint_id::<SemanticId>())
            ))
            .await
            .unwrap_err();
        assert!(
            violates(&refused, Some("solutions"), rule, InvariantKind::Check),
            "{kind} {barrier}: {refused:?}"
        );
    }
    drop(session);

    for stored in [&solution, &newer, &highs] {
        let row = store
            .solutions()
            .get(stored.solution_id)
            .await
            .unwrap()
            .unwrap();
        assert!(stores(&row, stored), "{row:?}");
    }
    let seed = store
        .solutions()
        .latest_compatible(&hash(5), &hash(6), NativeBackend::Ipopt)
        .await
        .unwrap()
        .unwrap();
    assert!(stores(&seed, &newer), "{seed:?}");
    let seed = store
        .solutions()
        .latest_compatible(&hash(5), &hash(6), NativeBackend::Highs)
        .await
        .unwrap()
        .unwrap();
    assert!(stores(&seed, &highs), "{seed:?}");
    assert!(
        store
            .solutions()
            .latest_compatible(&hash(7), &hash(6), NativeBackend::Ipopt)
            .await
            .unwrap()
            .is_none()
    );

    let incumbents = [
        RuntimeOperationalIncumbentsRow {
            attempt_id: attempt,
            seq: 0,
            step: 0,
            at: at(0).timestamp_micros(),
            elapsed_seconds: 0.25,
            phase: "scip.incumbent".into(),
            objective: 12.5,
            dual_bound: Some(3.0),
            gap: Some(0.76),
            nodes: Some(1),
            seconds: Some(0.2),
            solution_id: None,
        },
        RuntimeOperationalIncumbentsRow {
            attempt_id: attempt,
            seq: 1,
            step: 0,
            at: at(5).timestamp_micros(),
            elapsed_seconds: 5.0,
            phase: "scip.incumbent".into(),
            objective: 10.0,
            dual_bound: None,
            gap: None,
            nodes: None,
            seconds: None,
            solution_id: Some(newer.solution_id.into()),
        },
    ];
    assert_eq!(
        store
            .streams()
            .record_incumbents(&incumbents, &[])
            .await
            .unwrap(),
        2
    );
    // A re-sent batch stores nothing new.
    assert_eq!(
        store
            .streams()
            .record_incumbents(&incumbents, &[])
            .await
            .unwrap(),
        0
    );
    let latest = store
        .streams()
        .latest_incumbent(attempt)
        .await
        .unwrap()
        .unwrap();
    assert!(
        pse_model::SemanticEq::semantic_eq(&latest, &incumbents[1]),
        "{latest:?}"
    );
    database.remove().await.unwrap();
}

/// An incumbent's captured solution is stored in the incumbent's transaction; a re-sent
/// batch stores neither again. A resumed try finds the newest compatible captured
/// solution of its attempt chain, the nearest attempt first, and retention keeps the
/// incumbents that reference solutions (Plan 22 G8).
#[tokio::test]
async fn incumbent_solutions_follow_the_attempt_chain() {
    use pse_model::generated::enums::NativeBackend;
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let first = finished_attempt(&store).await;
    let second = finished_try(&store, Some(first)).await;
    let third = finished_try(&store, Some(second)).await;
    let solution = |attempt, stamp: u8, value: f64| NewSolution {
        solution_id: mint_id(),
        compatibility_stamp: hash(stamp),
        preparation_identity: hash(6),
        backend: NativeBackend::Scip,
        profile_stamp: hash(3),
        data_stamp: hash(4),
        vectors: SeedVectors::Nlp {
            primal: vec![value, 1.0],
            bounds: None,
            rows: None,
            barrier: None,
        },
        created_by: Some(attempt),
    };
    let incumbent =
        |attempt, seq, objective, solution: Option<&NewSolution>| RuntimeOperationalIncumbentsRow {
            attempt_id: attempt,
            seq,
            step: 0,
            at: at(seq).timestamp_micros(),
            elapsed_seconds: 1.0,
            phase: "scip.incumbent".into(),
            objective,
            dual_bound: Some(1.0),
            gap: None,
            nodes: Some(seq),
            seconds: Some(1.0),
            solution_id: solution.map(|s| s.solution_id.into()),
        };
    let (early, late) = (solution(first, 5, 9.0), solution(first, 5, 7.0));
    let batch = [
        incumbent(first, 0, 9.0, Some(&early)),
        incumbent(first, 1, 8.0, None),
        incumbent(first, 2, 7.0, Some(&late)),
    ];
    let captured = [early.clone(), late.clone()];
    let streams = store.streams();
    assert_eq!(
        streams.record_incumbents(&batch, &captured).await.unwrap(),
        3
    );
    // A re-sent batch stores nothing new, its solutions included.
    assert_eq!(
        streams.record_incumbents(&batch, &captured).await.unwrap(),
        0
    );
    for stored in [&early, &late] {
        let row = store
            .solutions()
            .get(stored.solution_id)
            .await
            .unwrap()
            .unwrap();
        assert!(stores(&row, stored), "{row:?}");
    }
    // A solution no incumbent of the batch references is refused.
    let stray = solution(second, 5, 1.0);
    let refused = streams
        .record_incumbents(
            &[incumbent(second, 0, 1.0, None)],
            std::slice::from_ref(&stray),
        )
        .await
        .unwrap_err();
    assert!(
        matches!(refused, OperationsError::InvalidRequest { .. }),
        "{refused:?}"
    );
    // The second try captured a solution of other coordinates only.
    let other = solution(second, 8, 6.0);
    streams
        .record_incumbents(
            &[incumbent(second, 0, 6.0, Some(&other))],
            std::slice::from_ref(&other),
        )
        .await
        .unwrap();

    let solutions = store.solutions();
    let resume = |attempt, stamp| {
        let solutions = solutions;
        async move {
            solutions
                .latest_in_attempt_chain(attempt, &hash(stamp), &hash(6), NativeBackend::Scip)
                .await
                .unwrap()
                .map(|row| row.solution_id)
        }
    };
    // From the third try: the second's is of other coordinates, so the first's latest.
    assert_eq!(resume(third, 5).await, Some(late.solution_id));
    assert_eq!(resume(third, 8).await, Some(other.solution_id));
    assert_eq!(resume(first, 8).await, None);
    assert!(
        solutions
            .latest_in_attempt_chain(third, &hash(5), &hash(6), NativeBackend::Highs)
            .await
            .unwrap()
            .is_none()
    );

    // Retention removes incumbents without a solution and keeps those that reference one.
    streams
        .apply_retention(crate::streams::Retention {
            finished_for: Duration::ZERO,
        })
        .await
        .unwrap();
    let session = database.session().await.unwrap();
    let kept = session
        .count(&format!(
            "SELECT count(*) FROM pse_ops.incumbents WHERE attempt_id = {}",
            lit(first)
        ))
        .await
        .unwrap();
    assert_eq!(kept, 2);
    assert_eq!(resume(third, 5).await, Some(late.solution_id));
    drop(session);
    database.remove().await.unwrap();
}

/// Whether a stored row holds exactly this seed: identity, evidence and vectors bit for
/// bit.
fn stores(row: &RuntimeOperationalSolutionsRow, solution: &NewSolution) -> bool {
    row.solution_id == solution.solution_id
        && row.compatibility_stamp == solution.compatibility_stamp
        && row.preparation_identity == solution.preparation_identity
        && row.backend == solution.backend
        && row.profile_stamp == solution.profile_stamp
        && row.data_stamp == solution.data_stamp
        && row.created_by == solution.created_by
        && row.kind == solution.vectors.kind()
        && SeedVectors::of(row).is_ok_and(|vectors| vectors == solution.vectors)
}

// ----------------------------------------------------------------- catalog --

#[tokio::test]
async fn concurrent_head_advance_one_winner() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let space = space(&store).await;
    let (base, _) = publish(&store, &space, None).await;

    let mut commits = Vec::new();
    for _ in 0..2 {
        let intent = intent(&store, &space).await;
        commits.push(commit_of(&intent, Some(base.publication_id)));
    }
    let catalog = store.catalog();
    let (left, right) = tokio::join!(catalog.commit(&commits[0]), catalog.commit(&commits[1]));
    let results = [left, right];
    let winners: Vec<PublicationId> = results
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
                if *expected == Some(base.publication_id) && *current == Some(winners[0])
        ),
        "{loser:?}"
    );
    assert_eq!(catalog.head(space.id).await.unwrap(), Some(winners[0]));

    // The loser re-prepares against the new head with the same intent; it never rebases.
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
        catalog.head(space.id).await.unwrap(),
        Some(retried.publication_id)
    );
    let stored = catalog
        .publication(retried.publication_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.publication.parent_publication, Some(winners[0]));
    assert_eq!(stored.members, sorted(&retried.members));
    database.remove().await.unwrap();
}

#[tokio::test]
async fn commit_is_idempotent_per_attempt_and_settles() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let catalog = store.catalog();
    let space = space(&store).await;
    let intent = intent(&store, &space).await;
    let commit = commit_of(&intent, None);
    let settle = |commit: &PublicationCommit| SettleRequest {
        settlement_id: mint_id(),
        attempt_id: commit.attempt_id,
        publication_id: commit.publication_id,
        workspace_id: commit.workspace_id,
        expected_parent: commit.expected_parent,
    };
    // Registered, not committed, head still the expected parent: provably not committed.
    assert_eq!(
        catalog.settle(&settle(&commit)).await.unwrap(),
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
        catalog.settle(&settle(&commit)).await.unwrap(),
        Settlement::Committed {
            publication_id: commit.publication_id
        }
    );
    // The complete request is compared: another member vector reuses the identity.
    let mut changed = commit.clone();
    changed.members[0].delta_version += 1;
    let reused = catalog.commit(&changed).await.unwrap_err();
    assert!(
        matches!(reused, OperationsError::PublicationIdentityReused { .. }),
        "{reused:?}"
    );
    let mut reparented = commit.clone();
    reparented.expected_parent = Some(commit.publication_id);
    assert!(matches!(
        catalog.commit(&reparented).await.unwrap_err(),
        OperationsError::PublicationIdentityReused { .. }
    ));
    // Registering the same intent again is idempotent; another prefix reuses its identity.
    assert_eq!(
        catalog
            .register_intent(&intent)
            .await
            .unwrap()
            .member_prefix,
        intent.member_prefix
    );
    let other = NewIntent {
        member_prefix: format!("{}elsewhere/", space.root),
        ..intent.clone()
    };
    assert!(matches!(
        catalog.register_intent(&other).await.unwrap_err(),
        OperationsError::PublicationIdentityReused { .. }
    ));

    // Only finished attempts are published.
    let running = new_attempt();
    store.attempts().create(&running, None).await.unwrap();
    let unfinished = NewIntent {
        publication_id: mint_id(),
        workspace_id: space.id,
        attempt_id: running.attempt_id,
        member_prefix: format!("{}members/{}/", space.root, running.attempt_id),
    };
    catalog.register_intent(&unfinished).await.unwrap();
    let refused = catalog
        .commit(&commit_of(&unfinished, Some(commit.publication_id)))
        .await
        .unwrap_err();
    assert!(
        matches!(refused, OperationsError::InvalidRequest { .. }),
        "{refused:?}"
    );
    let session = database.session().await.unwrap();
    assert_eq!(
        session
            .count("SELECT count(*) FROM pse_ops.settlements")
            .await
            .unwrap(),
        2
    );
    drop(session);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn maintenance_waits_for_reader_leases() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let catalog = store.catalog();
    let space = space(&store).await;
    let (old, old_commit) = publish(&store, &space, None).await;
    let (head, head_commit) = publish(&store, &space, Some(old.publication_id)).await;

    // A reader resolves the head, then another reads the old publication explicitly.
    let head_lease = catalog
        .acquire_reader_lease(mint_id(), ReadTarget::Head(space.id), "reader-1", LEASE)
        .await
        .unwrap();
    assert_eq!(head_lease.lease.publication_id, head.publication_id);
    let lease = catalog
        .acquire_reader_lease(
            mint_id(),
            ReadTarget::Publication(old.publication_id),
            "reader-2",
            LEASE,
        )
        .await
        .unwrap();
    assert_eq!(lease.record.members, sorted(&old_commit.members));

    // The head is protected from retention.
    let protected = catalog
        .mark_expiring(space.id, head.publication_id)
        .await
        .unwrap_err();
    assert!(matches!(
        protected,
        OperationsError::ProtectedPublication { .. }
    ));

    catalog
        .mark_expiring(space.id, old.publication_id)
        .await
        .unwrap();
    // No new lease once expiring.
    let refused = catalog
        .acquire_reader_lease(
            mint_id(),
            ReadTarget::Publication(old.publication_id),
            "reader-3",
            LEASE,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(refused, OperationsError::PublicationRetiring { .. }),
        "{refused:?}"
    );
    // Maintenance waits while the existing lease is live.
    let busy = catalog
        .wait_for_readers(
            old.publication_id,
            Duration::from_millis(10),
            Duration::from_millis(50),
        )
        .await
        .unwrap_err();
    assert!(
        matches!(busy, OperationsError::ReadersActive { active: 1, .. }),
        "{busy:?}"
    );
    for busy in [
        catalog
            .deletion_plan(space.id, old.publication_id)
            .await
            .map(|_| ())
            .unwrap_err(),
        catalog
            .mark_deleted(space.id, old.publication_id)
            .await
            .unwrap_err(),
    ] {
        assert!(
            matches!(busy, OperationsError::ReadersActive { .. }),
            "{busy:?}"
        );
    }
    // Versions a live lease reads stay protected.
    let protected = catalog.protected_versions(space.id).await.unwrap();
    for member in &old_commit.members {
        assert!(protects(&protected, member), "{protected:?}");
    }

    // The reader finishes; maintenance proceeds.
    let release = async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(
            catalog
                .release_reader_lease(lease.lease.lease_id)
                .await
                .unwrap()
        );
    };
    let wait = catalog.wait_for_readers(
        old.publication_id,
        Duration::from_millis(10),
        Duration::from_secs(10),
    );
    let ((), waited) = tokio::join!(release, wait);
    waited.unwrap();
    assert!(
        !catalog
            .release_reader_lease(lease.lease.lease_id)
            .await
            .unwrap()
    );
    let mut planned = catalog
        .deletion_plan(space.id, old.publication_id)
        .await
        .unwrap();
    planned.sort();
    let mut expected: Vec<String> = old_commit
        .members
        .iter()
        .map(|m| m.table_uri.clone())
        .collect();
    expected.sort();
    assert_eq!(planned, expected);
    catalog
        .mark_deleted(space.id, old.publication_id)
        .await
        .unwrap();
    catalog
        .mark_deleted(space.id, old.publication_id)
        .await
        .unwrap();
    assert!(
        catalog
            .deletion_plan(space.id, old.publication_id)
            .await
            .unwrap()
            .is_empty()
    );

    let protected = catalog.protected_versions(space.id).await.unwrap();
    for member in &old_commit.members {
        assert!(!protects(&protected, member), "{protected:?}");
    }
    for member in &head_commit.members {
        assert!(protects(&protected, member), "{protected:?}");
    }
    assert!(
        catalog
            .release_reader_lease(head_lease.lease.lease_id)
            .await
            .unwrap()
    );
    database.remove().await.unwrap();
}

#[tokio::test]
async fn catalog_protects_published_versions() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let catalog = store.catalog();
    let space = space(&store).await;
    // P1 writes a, b and d.
    let first = intent(&store, &space).await;
    let (a, b, d) = (
        member(&table(&first, "a"), "a", 1),
        member(&table(&first, "b"), "b", 1),
        member(&table(&first, "d"), "d", 1),
    );
    let mut p1 = commit_of(&first, None);
    p1.members = vec![a.clone(), b.clone(), d.clone()];
    catalog.commit(&p1).await.unwrap();
    // P2 writes c, retains a, reads b and a's change window [0, 1].
    let second = intent(&store, &space).await;
    let c = member(&table(&second, "c"), "c", 1);
    let mut p2 = commit_of(&second, Some(first.publication_id));
    p2.members = vec![c.clone(), a.clone()];
    p2.inputs = vec![b.clone()];
    p2.windows = vec![VersionWindow {
        table_uri: a.table_uri.clone(),
        from_version: 0,
        through_version: 1,
    }];
    catalog.commit(&p2).await.unwrap();
    // A live, unpublished intent.
    let live = intent(&store, &space).await;

    let protected = catalog.protected_versions(space.id).await.unwrap();
    for member in [&a, &b, &c, &d] {
        assert!(protects(&protected, member), "{member:?} in {protected:?}");
    }
    assert!(protected.contains(&ProtectedRange {
        table_uri: a.table_uri.clone(),
        from_version: 0,
        through_version: 1,
        reason: RetentionReason::Changes,
    }));
    assert!(protected.contains(&ProtectedRange {
        table_uri: live.member_prefix.clone(),
        from_version: 0,
        through_version: i64::MAX,
        reason: RetentionReason::Attempt,
    }));
    // Committed intents protect no prefix: their members are protected by publication.
    assert!(
        !protected
            .iter()
            .any(|range| range.table_uri == first.member_prefix)
    );

    // P1 expiring with a live lease still protects what only it selects (d).
    let reader = catalog
        .acquire_reader_lease(
            mint_id(),
            ReadTarget::Publication(first.publication_id),
            "reader",
            LEASE,
        )
        .await
        .unwrap();
    catalog
        .mark_expiring(space.id, first.publication_id)
        .await
        .unwrap();
    assert!(protects(
        &catalog.protected_versions(space.id).await.unwrap(),
        &d
    ));
    catalog
        .release_reader_lease(reader.lease.lease_id)
        .await
        .unwrap();
    let protected = catalog.protected_versions(space.id).await.unwrap();
    assert!(!protects(&protected, &d));
    // a stays protected by P2 (member and window), b by P2's input.
    assert!(protects(&protected, &a) && protects(&protected, &b));
    catalog
        .mark_deleted(space.id, first.publication_id)
        .await
        .unwrap();

    // Collection maintains the tables undeleted publications select, as members or
    // inputs (b, written by the deleted P1, is P2's input), each with the ranges it must
    // keep.
    let plan = catalog.begin_collect(space.id).await.unwrap();
    assert_eq!(plan.tables.keys().cloned().collect::<Vec<_>>(), {
        let mut tables = vec![
            a.table_uri.clone(),
            b.table_uri.clone(),
            c.table_uri.clone(),
        ];
        tables.sort();
        tables
    });
    assert_eq!(
        plan.tables[&a.table_uri]
            .iter()
            .map(|range| range.reason)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([RetentionReason::Publication, RetentionReason::Changes])
    );
    // A table under a live intent's prefix keeps its whole history.
    assert!(
        ProtectedRange {
            table_uri: live.member_prefix.clone(),
            from_version: 0,
            through_version: i64::MAX,
            reason: RetentionReason::Attempt,
        }
        .covers(&table(&live, "x"))
    );
    database.remove().await.unwrap();
}

#[tokio::test]
async fn commit_refuses_retiring_inputs_and_retained_members() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let catalog = store.catalog();
    let space = space(&store).await;
    let (first, p1) = publish(&store, &space, None).await;
    let (second, _) = publish(&store, &space, Some(first.publication_id)).await;
    // A retained member keeps its table and version under its own name here.
    let rename = |member: &MemberDescriptor, name: &str| MemberDescriptor {
        table_name: name.to_owned(),
        ..member.clone()
    };
    let retained = rename(&p1.members[0], "retained");

    // While P1 is live, a new publication may retain its member or read it.
    let reader = intent(&store, &space).await;
    let mut reads = commit_of(&reader, Some(second.publication_id));
    reads.members.push(retained.clone());
    reads.inputs.push(p1.members[0].clone());
    catalog.commit(&reads).await.unwrap();

    catalog
        .mark_expiring(space.id, first.publication_id)
        .await
        .unwrap();
    // Once only retiring publications select them, they are refused.
    let late = intent(&store, &space).await;
    let mut retains = commit_of(&late, Some(reader.publication_id));
    retains.members.push(rename(&p1.members[1], "retained"));
    let refused = catalog.commit(&retains).await.unwrap_err();
    assert!(
        matches!(&refused, OperationsError::InputRetired { table_uri, .. } if *table_uri == p1.members[1].table_uri),
        "{refused:?}"
    );
    let mut reads_retired = commit_of(&late, Some(reader.publication_id));
    reads_retired.inputs.push(p1.members[1].clone());
    assert!(matches!(
        catalog.commit(&reads_retired).await.unwrap_err(),
        OperationsError::InputRetired { .. }
    ));
    // p1.members[0] is still selected by the live `reads` publication.
    let mut reads_live = commit_of(&late, Some(reader.publication_id));
    reads_live.inputs.push(p1.members[0].clone());
    catalog.commit(&reads_live).await.unwrap();

    // A retirement waits for a commit that locked the publication it retains.
    let (third, p3) = publish(&store, &space, Some(late.publication_id)).await;
    let (fourth, _) = publish(&store, &space, Some(third.publication_id)).await;
    let holder = database.session().await.unwrap();
    holder
        .execute(&format!(
            "BEGIN; SELECT 1 FROM pse_ops.publications WHERE publication_id = {} FOR SHARE",
            lit(third.publication_id)
        ))
        .await
        .unwrap();
    let marking = catalog.mark_expiring(space.id, third.publication_id);
    let released = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        holder.execute("ROLLBACK").await.unwrap();
    };
    let (marked, ()) = tokio::join!(marking, released);
    marked.unwrap();
    let after = intent(&store, &space).await;
    let mut retains = commit_of(&after, Some(fourth.publication_id));
    retains.members.push(rename(&p3.members[0], "retained"));
    assert!(matches!(
        catalog.commit(&retains).await.unwrap_err(),
        OperationsError::InputRetired { .. }
    ));
    drop(holder);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn commit_requires_member_under_intent_prefix() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let catalog = store.catalog();
    let space = space(&store).await;
    let own = intent(&store, &space).await;
    let other = intent(&store, &space).await;

    // A member outside the intent's prefix that no publication selects.
    let mut foreign = commit_of(&own, None);
    foreign
        .members
        .push(member("file:///elsewhere/table/", "t", 0));
    let refused = catalog.commit(&foreign).await.unwrap_err();
    assert!(
        matches!(refused, OperationsError::InvalidRequest { .. }),
        "{refused:?}"
    );
    // A member under another intent's prefix.
    let mut borrowed = commit_of(&own, None);
    borrowed.members.push(member(&table(&other, "x"), "x", 1));
    assert!(matches!(
        catalog.commit(&borrowed).await.unwrap_err(),
        OperationsError::InvalidRequest { .. }
    ));
    // An input the publication writes itself.
    let mut own_input = commit_of(&own, None);
    own_input.inputs.push(own_input.members[0].clone());
    assert!(matches!(
        catalog.commit(&own_input).await.unwrap_err(),
        OperationsError::InvalidRequest { .. }
    ));
    // Two members under one qualified name.
    let mut twice = commit_of(&own, None);
    twice.members.push(twice.members[0].clone());
    assert!(matches!(
        catalog.commit(&twice).await.unwrap_err(),
        OperationsError::InvalidRequest { .. }
    ));
    // The intent itself must be registered, for its workspace and attempt.
    let mut unregistered = commit_of(&own, None);
    unregistered.publication_id = mint_id();
    assert!(matches!(
        catalog.commit(&unregistered).await.unwrap_err(),
        OperationsError::NotFound { .. }
    ));
    let mut mismatched = commit_of(&own, None);
    mismatched.attempt_id = other.attempt_id;
    assert!(matches!(
        catalog.commit(&mismatched).await.unwrap_err(),
        OperationsError::PublicationIdentityReused { .. }
    ));
    // Nothing became visible.
    assert_eq!(catalog.head(space.id).await.unwrap(), None);
    catalog.commit(&commit_of(&own, None)).await.unwrap();
    database.remove().await.unwrap();
}

#[tokio::test]
async fn abandoned_intent_cannot_commit() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let catalog = store.catalog();
    let space = space(&store).await;
    let abandoned = intent(&store, &space).await;
    catalog
        .abandon_intent(abandoned.publication_id)
        .await
        .unwrap();
    catalog
        .abandon_intent(abandoned.publication_id)
        .await
        .unwrap();
    let refused = catalog
        .commit(&commit_of(&abandoned, None))
        .await
        .unwrap_err();
    assert!(
        matches!(refused, OperationsError::IntentAbandoned { .. }),
        "{refused:?}"
    );
    assert_eq!(
        catalog
            .settle(&SettleRequest {
                settlement_id: mint_id(),
                attempt_id: abandoned.attempt_id,
                publication_id: abandoned.publication_id,
                workspace_id: space.id,
                expected_parent: None,
            })
            .await
            .unwrap(),
        Settlement::ProvedNoncommit
    );
    // A committed intent is never abandoned.
    let (committed, _) = publish(&store, &space, None).await;
    assert!(matches!(
        catalog
            .abandon_intent(committed.publication_id)
            .await
            .unwrap_err(),
        OperationsError::InvalidRequest { .. }
    ));
    database.remove().await.unwrap();
}

#[tokio::test]
async fn settle_distinguishes_committed_noncommit_conflict() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let catalog = store.catalog();
    let space = space(&store).await;
    let request = |intent: &NewIntent, parent: Option<PublicationId>| SettleRequest {
        settlement_id: mint_id(),
        attempt_id: intent.attempt_id,
        publication_id: intent.publication_id,
        workspace_id: intent.workspace_id,
        expected_parent: parent,
    };

    // An intent never registered: nothing can have been written or committed (D9).
    let attempt = finished_attempt(&store).await;
    let unregistered = NewIntent {
        publication_id: mint_id(),
        workspace_id: space.id,
        attempt_id: attempt,
        member_prefix: format!("{}members/{attempt}/", space.root),
    };
    assert_eq!(
        catalog.settle(&request(&unregistered, None)).await.unwrap(),
        Settlement::ProvedNoncommit
    );
    // Registered and prepared against the current head: provably not committed.
    let pending = intent(&store, &space).await;
    assert_eq!(
        catalog.settle(&request(&pending, None)).await.unwrap(),
        Settlement::ProvedNoncommit
    );
    // Another publication advanced the head: the request conflicts.
    let (base, _) = publish(&store, &space, None).await;
    assert_eq!(
        catalog.settle(&request(&pending, None)).await.unwrap(),
        Settlement::Conflict {
            reason: "the workspace head moved from the expected parent".to_owned(),
            head: Some(base.publication_id),
        }
    );
    // Committed.
    catalog
        .commit(&commit_of(&pending, Some(base.publication_id)))
        .await
        .unwrap();
    assert_eq!(
        catalog
            .settle(&request(&pending, Some(base.publication_id)))
            .await
            .unwrap(),
        Settlement::Committed {
            publication_id: pending.publication_id
        }
    );
    // The attempt is published as another publication: a request for a second intent
    // of the same attempt conflicts.
    let second = NewIntent {
        publication_id: mint_id(),
        member_prefix: format!("{}second/", pending.member_prefix),
        ..pending.clone()
    };
    catalog.register_intent(&second).await.unwrap();
    assert!(matches!(
        catalog.settle(&request(&second, Some(pending.publication_id))).await.unwrap(),
        Settlement::Conflict { head: Some(head), .. } if head == pending.publication_id
    ));

    // Settlement waits for a commit still in flight on the attempt.
    let inflight = intent(&store, &space).await;
    let holder = database.session().await.unwrap();
    holder
        .execute(&format!(
            "BEGIN; SELECT 1 FROM pse_ops.attempts WHERE attempt_id = {} FOR SHARE",
            lit(inflight.attempt_id)
        ))
        .await
        .unwrap();
    let inflight_request = request(&inflight, Some(pending.publication_id));
    let settling = async {
        let settled = catalog.settle(&inflight_request).await;
        (settled, Instant::now())
    };
    let released = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let at = Instant::now();
        holder.execute("ROLLBACK").await.unwrap();
        at
    };
    let ((settled, settled_at), released_at) = tokio::join!(settling, released);
    assert_eq!(settled.unwrap(), Settlement::ProvedNoncommit);
    assert!(settled_at >= released_at);

    let session = database.session().await.unwrap();
    let outcomes = session
        .texts("SELECT outcome::text FROM pse_ops.settlements ORDER BY settled_at")
        .await
        .unwrap()
        .into_iter()
        .filter_map(|row| row.into_iter().next().flatten())
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes,
        [
            "proved_noncommit",
            "proved_noncommit",
            "conflict",
            "committed",
            "conflict",
            "proved_noncommit"
        ]
    );
    drop((holder, session));
    database.remove().await.unwrap();
}

#[tokio::test]
async fn reader_lease_renews_and_lapses() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let catalog = store.catalog();
    let space = space(&store).await;
    let (published, _) = publish(&store, &space, None).await;
    let target = ReadTarget::Publication(published.publication_id);

    let lease = catalog
        .acquire_reader_lease(mint_id(), target, "reader", Duration::from_millis(500))
        .await
        .unwrap();
    let renewed = catalog
        .renew_reader_lease(lease.lease.lease_id, Duration::from_secs(60))
        .await
        .unwrap();
    assert!(renewed.expires_at > lease.lease.expires_at);
    assert_eq!(
        catalog
            .active_reader_leases(published.publication_id)
            .await
            .unwrap(),
        1
    );
    // A released lease cannot be renewed.
    assert!(
        catalog
            .release_reader_lease(lease.lease.lease_id)
            .await
            .unwrap()
    );
    assert!(matches!(
        catalog
            .renew_reader_lease(lease.lease.lease_id, LEASE)
            .await
            .unwrap_err(),
        OperationsError::ReaderLeaseLapsed { .. }
    ));
    // Nor can one whose expiry passed: it protects nothing any more.
    let short = catalog
        .acquire_reader_lease(mint_id(), target, "slow", Duration::from_millis(50))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(matches!(
        catalog
            .renew_reader_lease(short.lease.lease_id, LEASE)
            .await
            .unwrap_err(),
        OperationsError::ReaderLeaseLapsed { lease } if lease == short.lease.lease_id
    ));
    assert_eq!(
        catalog
            .active_reader_leases(published.publication_id)
            .await
            .unwrap(),
        0
    );
    assert!(
        catalog
            .reader_lease(short.lease.lease_id)
            .await
            .unwrap()
            .is_some_and(|row| row.released_at.is_none())
    );
    database.remove().await.unwrap();
}

#[tokio::test]
async fn head_lease_records_resolved_publication() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let catalog = store.catalog();
    let space = space(&store).await;
    // An empty head cannot be read.
    assert!(matches!(
        catalog
            .acquire_reader_lease(mint_id(), ReadTarget::Head(space.id), "r", LEASE)
            .await
            .unwrap_err(),
        OperationsError::NotFound { .. }
    ));
    let (first, _) = publish(&store, &space, None).await;
    let (second, second_commit) = publish(&store, &space, Some(first.publication_id)).await;
    let head = catalog
        .acquire_reader_lease(mint_id(), ReadTarget::Head(space.id), "head-reader", LEASE)
        .await
        .unwrap();
    assert_eq!(head.lease.publication_id, second.publication_id);
    assert_eq!(head.lease.head_of, Some(space.id));
    assert_eq!(head.lease.holder, "head-reader");
    assert_eq!(
        head.record.publication.publication_id,
        second.publication_id
    );
    assert_eq!(
        head.record.publication.parent_publication,
        Some(first.publication_id)
    );
    assert_eq!(head.record.members, sorted(&second_commit.members));
    assert_eq!(
        head.maintenance_epoch,
        catalog
            .workspace(space.id)
            .await
            .unwrap()
            .unwrap()
            .maintenance_epoch
    );
    let exact = catalog
        .acquire_reader_lease(
            mint_id(),
            ReadTarget::Publication(first.publication_id),
            "exact",
            LEASE,
        )
        .await
        .unwrap();
    assert_eq!(exact.lease.head_of, None);
    assert_eq!(
        exact.record.publication.publication_id,
        first.publication_id
    );
    database.remove().await.unwrap();
}

#[tokio::test]
async fn deletion_plan_excludes_shared_tables() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let catalog = store.catalog();
    let space = space(&store).await;
    // Another workspace publishes e.
    let elsewhere = self::space(&store).await;
    let (foreign, foreign_commit) = publish(&store, &elsewhere, None).await;
    let e = foreign_commit.members[0].clone();

    // P1 writes a, b, d and g, and retains e from the other workspace.
    let first = intent(&store, &space).await;
    let (a, b, d, g) = (
        member(&table(&first, "a"), "a", 1),
        member(&table(&first, "b"), "b", 1),
        member(&table(&first, "d"), "d", 1),
        member(&table(&first, "g"), "g", 1),
    );
    let mut p1 = commit_of(&first, None);
    p1.members = vec![a.clone(), b.clone(), d.clone(), e.clone(), g.clone()];
    catalog.commit(&p1).await.unwrap();
    // P2 writes c, retains a and reads g.
    let second = intent(&store, &space).await;
    let c = member(&table(&second, "c"), "c", 1);
    let mut p2 = commit_of(&second, Some(first.publication_id));
    p2.members = vec![c.clone(), a.clone()];
    p2.inputs = vec![g.clone()];
    catalog.commit(&p2).await.unwrap();
    // P3, the head, writes f and read b's change window.
    let third = intent(&store, &space).await;
    let mut p3 = commit_of(&third, Some(second.publication_id));
    p3.windows = vec![VersionWindow {
        table_uri: b.table_uri.clone(),
        from_version: 0,
        through_version: 1,
    }];
    catalog.commit(&p3).await.unwrap();

    // Retiring P1 removes only d: a and g are selected by P2 (as a member and an input),
    // b is covered by P3's window, and e belongs to the other workspace.
    catalog
        .mark_expiring(space.id, first.publication_id)
        .await
        .unwrap();
    assert_eq!(
        catalog
            .deletion_plan(space.id, first.publication_id)
            .await
            .unwrap(),
        [d.table_uri.clone()]
    );
    catalog
        .mark_deleted(space.id, first.publication_id)
        .await
        .unwrap();
    // Once P1 is deleted, retiring P2 removes c, and a and g (which only the deleted P1
    // selected besides P2).
    catalog
        .mark_expiring(space.id, second.publication_id)
        .await
        .unwrap();
    let mut planned = catalog
        .deletion_plan(space.id, second.publication_id)
        .await
        .unwrap();
    planned.sort();
    let mut expected = vec![
        a.table_uri.clone(),
        c.table_uri.clone(),
        g.table_uri.clone(),
    ];
    expected.sort();
    assert_eq!(planned, expected);
    // The other workspace's publication is untouched and still protects e.
    assert!(protects(
        &catalog.protected_versions(elsewhere.id).await.unwrap(),
        &e
    ));
    assert_eq!(
        catalog.head(elsewhere.id).await.unwrap(),
        Some(foreign.publication_id)
    );
    database.remove().await.unwrap();
}

#[tokio::test]
async fn maintenance_bumps_epoch_before_effects() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let catalog = store.catalog();
    let space = space(&store).await;
    let epoch = || async {
        catalog
            .workspace(space.id)
            .await
            .unwrap()
            .unwrap()
            .maintenance_epoch
    };
    assert_eq!(epoch().await, 0);
    let (old, _) = publish(&store, &space, None).await;
    let (head, _) = publish(&store, &space, Some(old.publication_id)).await;
    let before = catalog
        .acquire_reader_lease(mint_id(), ReadTarget::Head(space.id), "before", LEASE)
        .await
        .unwrap();
    assert_eq!(before.maintenance_epoch, 0);

    // Retirement advances the epoch in the transaction that marks, before any file
    // effect; a refused retirement (the head) changes nothing.
    assert!(
        catalog
            .mark_expiring(space.id, head.publication_id)
            .await
            .is_err()
    );
    assert_eq!(epoch().await, 0);
    assert_eq!(
        catalog
            .mark_expiring(space.id, old.publication_id)
            .await
            .unwrap(),
        1
    );
    // Collection advances it again before it plans.
    let plan = catalog.begin_collect(space.id).await.unwrap();
    assert_eq!(plan.maintenance_epoch, 2);
    assert_eq!(epoch().await, 2);
    // A reader after maintenance keys its caches on the new epoch; the earlier reader
    // keeps its own.
    let after = catalog
        .acquire_reader_lease(mint_id(), ReadTarget::Head(space.id), "after", LEASE)
        .await
        .unwrap();
    assert_eq!(after.maintenance_epoch, 2);
    assert_eq!(before.maintenance_epoch, 0);
    // Maintainers of a workspace are serialized: a held advisory lock delays collection.
    let holder = database.session().await.unwrap();
    holder
        .execute(&format!(
            "BEGIN; SELECT pg_advisory_xact_lock(hashtextextended('pse_ops.maintenance:{}', 0))",
            space.id
        ))
        .await
        .unwrap();
    let collecting = catalog.begin_collect(space.id);
    let released = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(epoch().await, 2);
        holder.execute("ROLLBACK").await.unwrap();
    };
    let (plan, ()) = tokio::join!(collecting, released);
    assert_eq!(plan.unwrap().maintenance_epoch, 3);
    drop(holder);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn reclaimable_intents_are_fenced() {
    let database = TestDatabase::create().await.unwrap();
    let store = database.store().clone();
    let catalog = store.catalog();
    let space = space(&store).await;
    // Abandoned.
    let abandoned = intent(&store, &space).await;
    catalog
        .abandon_intent(abandoned.publication_id)
        .await
        .unwrap();
    // Its attempt was published as another publication.
    let (published, _) = publish(&store, &space, None).await;
    let duplicate = NewIntent {
        publication_id: mint_id(),
        member_prefix: format!("{}retry/", published.member_prefix),
        ..published.clone()
    };
    catalog.register_intent(&duplicate).await.unwrap();
    // Its attempt went stale.
    let attempts = store.attempts();
    let stale_attempt = new_attempt();
    attempts.create(&stale_attempt, None).await.unwrap();
    attempts
        .transition(
            stale_attempt.attempt_id,
            AttemptState::Queued,
            &TransitionNote::by("test"),
        )
        .await
        .unwrap();
    attempts
        .start(stale_attempt.attempt_id, "worker", LEASE)
        .await
        .unwrap();
    attempts
        .transition(
            stale_attempt.attempt_id,
            AttemptState::Stale,
            &TransitionNote::by("sweep"),
        )
        .await
        .unwrap();
    let stale = NewIntent {
        publication_id: mint_id(),
        workspace_id: space.id,
        attempt_id: stale_attempt.attempt_id,
        member_prefix: format!("{}members/{}/", space.root, stale_attempt.attempt_id),
    };
    catalog.register_intent(&stale).await.unwrap();
    // Live: finished, unpublished, not abandoned.
    let live = intent(&store, &space).await;

    let claimed = catalog.claim_reclaimable(space.id).await.unwrap();
    let ids: BTreeSet<PublicationId> = claimed.iter().map(|i| i.publication_id).collect();
    assert_eq!(
        ids,
        BTreeSet::from([
            abandoned.publication_id,
            duplicate.publication_id,
            stale.publication_id
        ])
    );
    // Claimed intents are fenced: abandoned, so no commit can publish them.
    assert!(claimed.iter().all(|i| i.abandoned_at.is_some()));
    assert!(matches!(
        catalog
            .commit(&commit_of(&duplicate, Some(published.publication_id)))
            .await
            .unwrap_err(),
        OperationsError::IntentAbandoned { .. } | OperationsError::PublicationIdentityReused { .. }
    ));
    // Only the live intent's prefix is protected.
    let prefixes: Vec<String> = catalog
        .protected_versions(space.id)
        .await
        .unwrap()
        .into_iter()
        .filter(|range| range.reason == RetentionReason::Attempt)
        .map(|range| range.table_uri)
        .collect();
    assert_eq!(prefixes, [live.member_prefix.clone()]);

    // Recording the removal is idempotent; a reclaimed intent is not claimed again.
    catalog
        .mark_reclaimed(abandoned.publication_id)
        .await
        .unwrap();
    catalog
        .mark_reclaimed(abandoned.publication_id)
        .await
        .unwrap();
    let again: BTreeSet<PublicationId> = catalog
        .claim_reclaimable(space.id)
        .await
        .unwrap()
        .iter()
        .map(|i| i.publication_id)
        .collect();
    assert_eq!(
        again,
        BTreeSet::from([duplicate.publication_id, stale.publication_id])
    );
    for intent in [&duplicate, &stale] {
        catalog.mark_reclaimed(intent.publication_id).await.unwrap();
    }
    assert!(
        catalog
            .claim_reclaimable(space.id)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        catalog
            .intent(abandoned.publication_id)
            .await
            .unwrap()
            .is_some_and(|i| i.reclaimed_at.is_some())
    );
    // A committed intent is never reclaimed.
    assert!(matches!(
        catalog
            .mark_reclaimed(published.publication_id)
            .await
            .unwrap_err(),
        OperationsError::InvalidRequest { .. }
    ));
    database.remove().await.unwrap();
}

#[tokio::test]
async fn lost_commit_acknowledgement_settles() {
    use crate::testing::{Fault, FaultPoint, FaultProxy};
    let database = TestDatabase::create().await.unwrap();
    let proxy = FaultProxy::start(database.url()).await.unwrap();
    let store = Store::open_with(proxy.url(), &crate::StoreOptions::for_tests())
        .await
        .unwrap();
    let catalog = store.catalog();
    let space = space(&store).await;
    let fault = |point| Fault {
        marker: "INSERT INTO pse_ops.publications ",
        point,
    };
    let settle = |commit: &PublicationCommit| SettleRequest {
        settlement_id: mint_id(),
        attempt_id: commit.attempt_id,
        publication_id: commit.publication_id,
        workspace_id: commit.workspace_id,
        expected_parent: commit.expected_parent,
    };

    // The connection drops before COMMIT reaches the server: nothing was committed.
    let first = commit_of(&intent(&store, &space).await, None);
    proxy.arm(fault(FaultPoint::BeforeCommit));
    let lost = catalog.commit(&first).await.unwrap_err();
    assert!(proxy.fired());
    assert!(
        matches!(lost, OperationsError::Unavailable { .. }),
        "{lost:?}"
    );
    assert_eq!(
        catalog.settle(&settle(&first)).await.unwrap(),
        Settlement::ProvedNoncommit
    );
    // The same request commits on a retry.
    catalog.commit(&first).await.unwrap();

    // The server commits but the acknowledgement is lost: settlement finds it.
    let second = commit_of(&intent(&store, &space).await, Some(first.publication_id));
    proxy.arm(fault(FaultPoint::AfterCommit));
    let lost = catalog.commit(&second).await.unwrap_err();
    assert!(proxy.fired());
    assert!(
        matches!(lost, OperationsError::Unavailable { .. }),
        "{lost:?}"
    );
    assert_eq!(
        catalog.settle(&settle(&second)).await.unwrap(),
        Settlement::Committed {
            publication_id: second.publication_id
        }
    );
    assert_eq!(
        catalog.head(space.id).await.unwrap(),
        Some(second.publication_id)
    );
    store.close();
    drop(proxy);
    database.remove().await.unwrap();
}
