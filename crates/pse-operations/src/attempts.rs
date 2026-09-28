// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The attempt registry: creation, transitions with an audit row in the same
//! transaction, heartbeats and the stale sweep (DP-19).

use std::time::Duration;

use chrono::{DateTime, Utc};
use pse_ids::{ContentHash, SemanticId};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgConnection, Row};

use crate::codec;
use crate::error::{Classify, OperationsError, Target};
use crate::lifecycle::{self, AttemptState, Lifecycle};
use crate::store::Store;
pub use pse_model::generated::enums::AttemptKind;

/// An attempt registered before any effect; the runtime mints its identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewAttempt {
    /// The attempt identity (`UUIDv7`, minted by the runtime).
    pub attempt_id: SemanticId,
    /// The run the attempt belongs to.
    pub run_id: SemanticId,
    /// What the attempt computes.
    pub kind: AttemptKind,
    /// The request identity (blueprint §5.1).
    pub request_identity: ContentHash,
    /// The preparation identity, when preparation happened before registration.
    pub preparation_identity: Option<ContentHash>,
    /// The attempt this one supersedes or retries.
    pub parent_attempt: Option<SemanticId>,
}

/// A typed termination: the registry spelling of the outcome plus optional detail.
#[derive(Clone, Debug, PartialEq)]
pub struct Termination {
    /// The termination tag or diagnostic code, as its registry spelling.
    pub code: String,
    /// Structured detail, versioned by its producer.
    pub detail: Option<serde_json::Value>,
}

/// Who asked for a transition and why; recorded in the audit row.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransitionNote {
    /// The worker, service or user that caused the change.
    pub actor: Option<String>,
    /// Why.
    pub reason: Option<String>,
    /// The termination, for transitions that end work.
    pub termination: Option<Termination>,
}

impl TransitionNote {
    /// A note naming its actor.
    pub fn by(actor: impl Into<String>) -> Self {
        Self {
            actor: Some(actor.into()),
            ..Self::default()
        }
    }

    /// Add a reason.
    #[must_use]
    pub fn because(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Add a termination.
    #[must_use]
    pub fn terminated(mut self, termination: Termination) -> Self {
        self.termination = Some(termination);
        self
    }
}

/// An attempt as stored.
#[derive(Clone, Debug, PartialEq)]
pub struct AttemptRecord {
    /// The attempt identity.
    pub attempt_id: SemanticId,
    /// Its run.
    pub run_id: SemanticId,
    /// What the attempt computes.
    pub kind: AttemptKind,
    /// The request identity.
    pub request_identity: ContentHash,
    /// The preparation identity, if known.
    pub preparation_identity: Option<ContentHash>,
    /// The lifecycle state.
    pub state: AttemptState,
    /// The number of transitions so far; the audit sequence of the latest one.
    pub state_version: i32,
    /// The attempt this one supersedes or retries.
    pub parent_attempt: Option<SemanticId>,
    /// The worker that owns or last owned the attempt.
    pub worker: Option<String>,
    /// The lease expiry while running.
    pub lease_expires_at: Option<DateTime<Utc>>,
    /// The durable cancellation request (finding T03): the authority, not the notification.
    pub cancel_requested: bool,
    /// The termination, once work ended.
    pub termination: Option<Termination>,
    /// Registration time.
    pub created_at: DateTime<Utc>,
    /// When work started.
    pub started_at: Option<DateTime<Utc>>,
    /// When work ended.
    pub finished_at: Option<DateTime<Utc>>,
}

impl FromRow<'_, PgRow> for AttemptRecord {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let termination = row
            .try_get::<Option<String>, _>("termination")?
            .map(|code| -> Result<Termination, sqlx::Error> {
                Ok(Termination {
                    code,
                    detail: row.try_get("termination_detail")?,
                })
            })
            .transpose()?;
        Ok(Self {
            attempt_id: codec::id(row, "attempt_id")?,
            run_id: codec::id(row, "run_id")?,
            kind: codec::parsed(row, "kind")?,
            request_identity: codec::hash(row, "request_identity")?,
            preparation_identity: codec::opt_hash(row, "preparation_identity")?,
            state: codec::parsed(row, "state")?,
            state_version: row.try_get("state_version")?,
            parent_attempt: codec::opt_id(row, "parent_attempt")?,
            worker: row.try_get("worker")?,
            lease_expires_at: row.try_get("lease_expires_at")?,
            cancel_requested: row.try_get("cancel_requested")?,
            termination,
            created_at: row.try_get("created_at")?,
            started_at: row.try_get("started_at")?,
            finished_at: row.try_get("finished_at")?,
        })
    }
}

/// One audit row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransitionRecord {
    /// The attempt.
    pub attempt_id: SemanticId,
    /// Sequence within the attempt; 0 is creation.
    pub seq: i32,
    /// The previous state; `None` at creation.
    pub from: Option<AttemptState>,
    /// The new state.
    pub to: AttemptState,
    /// Who caused it.
    pub actor: Option<String>,
    /// Why.
    pub reason: Option<String>,
    /// When.
    pub at: DateTime<Utc>,
}

impl FromRow<'_, PgRow> for TransitionRecord {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            attempt_id: codec::id(row, "attempt_id")?,
            seq: row.try_get("seq")?,
            from: codec::opt_parsed(row, "from_state")?,
            to: codec::parsed(row, "to_state")?,
            actor: row.try_get("actor")?,
            reason: row.try_get("reason")?,
            at: row.try_get("at")?,
        })
    }
}

/// Which attempts [`Attempts::list`] returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttemptFilter {
    /// Only the attempts of this run.
    pub run: Option<SemanticId>,
    /// Only attempts in these states; every state when empty.
    pub states: Vec<AttemptState>,
    /// At most this many, newest first.
    pub limit: i64,
}

impl AttemptFilter {
    /// The newest `limit` attempts of every run and state.
    pub const fn newest(limit: i64) -> Self {
        Self {
            run: None,
            states: Vec::new(),
            limit,
        }
    }
}

/// What a heartbeat returns: the durable cancellation flag and the new lease expiry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeartbeatAck {
    /// Whether cancellation was requested; the worker stops cooperatively when true.
    pub cancel_requested: bool,
    /// The extended lease.
    pub lease_expires_at: DateTime<Utc>,
}

/// The column list every attempt read selects, as a literal for `concat!`.
macro_rules! attempt_columns {
    () => {
        "attempt_id, run_id, kind, request_identity, preparation_identity, state, \
         state_version, parent_attempt, worker, lease_expires_at, cancel_requested, \
         termination, termination_detail, created_at, started_at, finished_at"
    };
}

/// A lease to install when an attempt enters `running`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Lease<'a> {
    pub(crate) worker: &'a str,
    pub(crate) duration: Duration,
}

/// Insert a planned attempt and its creation audit row.
pub(crate) async fn insert(
    conn: &mut PgConnection,
    target: &Target,
    attempt: &NewAttempt,
    actor: Option<&str>,
) -> Result<(), OperationsError> {
    sqlx::query(
        "INSERT INTO pse_ops.attempts \
             (attempt_id, run_id, kind, request_identity, preparation_identity, state, parent_attempt) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(codec::uuid(attempt.attempt_id))
    .bind(codec::uuid(attempt.run_id))
    .bind(attempt.kind.as_str())
    .bind(codec::hash_bytes(&attempt.request_identity))
    .bind(attempt.preparation_identity.as_ref().map(codec::hash_bytes))
    .bind(lifecycle::INITIAL.as_str())
    .bind(attempt.parent_attempt.map(codec::uuid))
    .execute(&mut *conn)
    .await
    .classify(target)?;
    sqlx::query(
        "INSERT INTO pse_ops.attempt_transitions (attempt_id, seq, from_state, to_state, actor) \
         VALUES ($1, 0, NULL, $2, $3)",
    )
    .bind(codec::uuid(attempt.attempt_id))
    .bind(lifecycle::INITIAL.as_str())
    .bind(actor)
    .execute(&mut *conn)
    .await
    .classify(target)?;
    Ok(())
}

/// Lock the attempt row and return its state.
pub(crate) async fn lock_state(
    conn: &mut PgConnection,
    target: &Target,
    attempt: SemanticId,
) -> Result<(AttemptState, i32), OperationsError> {
    let row = sqlx::query(
        "SELECT state, state_version FROM pse_ops.attempts WHERE attempt_id = $1 FOR UPDATE",
    )
    .bind(codec::uuid(attempt))
    .fetch_optional(&mut *conn)
    .await
    .classify(target)?
    .ok_or_else(|| OperationsError::NotFound {
        entity: "attempt",
        id: attempt.to_string(),
    })?;
    let state = codec::parsed(&row, "state").classify(target)?;
    let version = row.try_get("state_version").classify(target)?;
    Ok((state, version))
}

/// Apply one transition under the attempt's row lock: check it against the one transition
/// table, update the row and append the audit row. Returns the previous state.
pub(crate) async fn apply(
    conn: &mut PgConnection,
    target: &Target,
    attempt: SemanticId,
    to: AttemptState,
    note: &TransitionNote,
    lease: Option<Lease<'_>>,
) -> Result<AttemptState, OperationsError> {
    let (from, version) = lock_state(conn, target, attempt).await?;
    if !from.may_become(to) {
        return Err(OperationsError::IllegalTransition { attempt, from, to });
    }
    if (to == AttemptState::Running) != lease.is_some() {
        return Err(OperationsError::InvalidRequest {
            reason: format!("attempt {attempt}: a lease is required exactly when entering running"),
        });
    }
    let next = version + 1;
    sqlx::query(
        "UPDATE pse_ops.attempts SET \
             state = $2, state_version = $3, updated_at = now(), \
             worker = coalesce($4, worker), \
             lease_expires_at = CASE WHEN $4 IS NULL THEN NULL ELSE now() + $5 END, \
             heartbeat_at = CASE WHEN $4 IS NULL THEN heartbeat_at ELSE now() END, \
             started_at = CASE WHEN $4 IS NULL THEN started_at ELSE now() END, \
             finished_at = CASE WHEN $6 THEN coalesce(finished_at, now()) ELSE finished_at END, \
             termination = coalesce($7, termination), \
             termination_detail = coalesce($8, termination_detail) \
         WHERE attempt_id = $1",
    )
    .bind(codec::uuid(attempt))
    .bind(to.as_str())
    .bind(next)
    .bind(lease.map(|lease| lease.worker))
    .bind(lease.map(|lease| codec::interval(lease.duration)))
    .bind(to.ends_work())
    .bind(note.termination.as_ref().map(|t| t.code.as_str()))
    .bind(note.termination.as_ref().and_then(|t| t.detail.as_ref()))
    .execute(&mut *conn)
    .await
    .classify(target)?;
    sqlx::query(
        "INSERT INTO pse_ops.attempt_transitions \
             (attempt_id, seq, from_state, to_state, actor, reason) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(codec::uuid(attempt))
    .bind(next)
    .bind(from.as_str())
    .bind(to.as_str())
    .bind(note.actor.as_deref())
    .bind(note.reason.as_deref())
    .execute(&mut *conn)
    .await
    .classify(target)?;
    if to.ends_work() {
        // A stream watcher learns that no further progress will come.
        sqlx::query("SELECT pg_notify($1, $2)")
            .bind(crate::streams::PROGRESS_CHANNEL)
            .bind(codec::uuid(attempt).to_string())
            .execute(&mut *conn)
            .await
            .classify(target)?;
    }
    Ok(from)
}

/// Read one attempt.
pub(crate) async fn fetch(
    conn: &mut PgConnection,
    target: &Target,
    attempt: SemanticId,
) -> Result<AttemptRecord, OperationsError> {
    sqlx::query_as::<_, AttemptRecord>(concat!(
        "SELECT ",
        attempt_columns!(),
        " FROM pse_ops.attempts WHERE attempt_id = $1"
    ))
    .bind(codec::uuid(attempt))
    .fetch_optional(&mut *conn)
    .await
    .classify(target)?
    .ok_or_else(|| OperationsError::NotFound {
        entity: "attempt",
        id: attempt.to_string(),
    })
}

/// The attempt repository.
#[derive(Clone, Copy, Debug)]
pub struct Attempts<'s> {
    store: &'s Store,
}

impl<'s> Attempts<'s> {
    pub(crate) const fn new(store: &'s Store) -> Self {
        Self { store }
    }

    fn target(&self) -> &Target {
        self.store.target()
    }

    /// Register a planned attempt with its creation audit row.
    ///
    /// # Errors
    ///
    /// [`OperationsError::Duplicate`] when the identity already exists; classified driver
    /// failures.
    pub async fn create(
        &self,
        attempt: &NewAttempt,
        actor: Option<&str>,
    ) -> Result<AttemptRecord, OperationsError> {
        let mut tx = self.store.pool().begin().await.classify(self.target())?;
        insert(&mut tx, self.target(), attempt, actor).await?;
        let record = fetch(&mut tx, self.target(), attempt.attempt_id).await?;
        tx.commit().await.classify(self.target())?;
        Ok(record)
    }

    /// Read one attempt.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`]; classified driver failures.
    pub async fn get(&self, attempt: SemanticId) -> Result<AttemptRecord, OperationsError> {
        let mut conn = self.store.pool().acquire().await.classify(self.target())?;
        fetch(&mut conn, self.target(), attempt).await
    }

    /// Change an attempt's state, if the transition table allows it. Entering `running`
    /// goes through [`Attempts::start`] or a job claim, which install the lease.
    ///
    /// # Errors
    ///
    /// [`OperationsError::IllegalTransition`], [`OperationsError::NotFound`],
    /// [`OperationsError::InvalidRequest`] for `running`; classified driver failures.
    pub async fn transition(
        &self,
        attempt: SemanticId,
        to: AttemptState,
        note: &TransitionNote,
    ) -> Result<AttemptRecord, OperationsError> {
        let mut tx = self.store.pool().begin().await.classify(self.target())?;
        apply(&mut tx, self.target(), attempt, to, note, None).await?;
        let record = fetch(&mut tx, self.target(), attempt).await?;
        tx.commit().await.classify(self.target())?;
        Ok(record)
    }

    /// Move a queued attempt to `running` under a lease owned by `worker`.
    ///
    /// # Errors
    ///
    /// As for [`Attempts::transition`].
    pub async fn start(
        &self,
        attempt: SemanticId,
        worker: &str,
        lease: Duration,
    ) -> Result<AttemptRecord, OperationsError> {
        let mut tx = self.store.pool().begin().await.classify(self.target())?;
        let lease = Lease {
            worker,
            duration: lease,
        };
        let note = TransitionNote::by(worker);
        apply(
            &mut tx,
            self.target(),
            attempt,
            AttemptState::Running,
            &note,
            Some(lease),
        )
        .await?;
        let record = fetch(&mut tx, self.target(), attempt).await?;
        tx.commit().await.classify(self.target())?;
        Ok(record)
    }

    /// Extend the running attempt's lease. Returns the durable cancellation flag, which is
    /// the authority for cancellation (finding T03).
    ///
    /// # Errors
    ///
    /// [`OperationsError::LeaseLost`] when the attempt is not running, is owned by another
    /// worker, or its lease already expired; classified driver failures.
    pub async fn heartbeat(
        &self,
        attempt: SemanticId,
        worker: &str,
        lease: Duration,
    ) -> Result<HeartbeatAck, OperationsError> {
        let row = sqlx::query(
            "UPDATE pse_ops.attempts \
             SET lease_expires_at = now() + $3, heartbeat_at = now() \
             WHERE attempt_id = $1 AND worker = $2 AND state = 'running' \
               AND lease_expires_at > now() \
             RETURNING cancel_requested, lease_expires_at",
        )
        .bind(codec::uuid(attempt))
        .bind(worker)
        .bind(codec::interval(lease))
        .fetch_optional(self.store.pool())
        .await
        .classify(self.target())?
        .ok_or_else(|| OperationsError::LeaseLost {
            attempt,
            worker: worker.to_owned(),
        })?;
        Ok(HeartbeatAck {
            cancel_requested: row.try_get("cancel_requested").classify(self.target())?,
            lease_expires_at: row.try_get("lease_expires_at").classify(self.target())?,
        })
    }

    /// The audit trail of one attempt, in order.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn history(
        &self,
        attempt: SemanticId,
    ) -> Result<Vec<TransitionRecord>, OperationsError> {
        sqlx::query_as(
            "SELECT attempt_id, seq, from_state, to_state, actor, reason, at \
             FROM pse_ops.attempt_transitions WHERE attempt_id = $1 ORDER BY seq",
        )
        .bind(codec::uuid(attempt))
        .fetch_all(self.store.pool())
        .await
        .classify(self.target())
    }

    /// Attempts newest first: all of them, or those of one run or in the given states, at
    /// most `filter.limit`. This is what survives a restart of the process that ran them.
    ///
    /// # Errors
    ///
    /// [`OperationsError::InvalidRequest`] for a non-positive limit; classified driver
    /// failures.
    pub async fn list(
        &self,
        filter: &AttemptFilter,
    ) -> Result<Vec<AttemptRecord>, OperationsError> {
        if filter.limit <= 0 {
            return Err(OperationsError::InvalidRequest {
                reason: format!("attempt listing limit {} is not positive", filter.limit),
            });
        }
        sqlx::query_as::<_, AttemptRecord>(concat!(
            "SELECT ",
            attempt_columns!(),
            " FROM pse_ops.attempts \
             WHERE ($1::uuid IS NULL OR run_id = $1) \
               AND (cardinality($2::text[]) = 0 OR state = ANY($2::text[])) \
             ORDER BY created_at DESC, attempt_id DESC LIMIT $3"
        ))
        .bind(filter.run.map(codec::uuid))
        .bind(
            filter
                .states
                .iter()
                .map(|state| state.as_str())
                .collect::<Vec<_>>(),
        )
        .bind(filter.limit)
        .fetch_all(self.store.pool())
        .await
        .classify(self.target())
    }

    /// Mark every running attempt whose lease expired as `stale`, at most `limit` per
    /// call. Attempts locked by another sweeper are skipped. Job-backed attempts are
    /// also requeued by [`crate::jobs::Jobs::requeue_expired`]; this sweep covers the rest.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn sweep_stale(&self, limit: i64) -> Result<Vec<SemanticId>, OperationsError> {
        let mut tx = self.store.pool().begin().await.classify(self.target())?;
        let rows = sqlx::query(
            "SELECT attempt_id FROM pse_ops.attempts \
             WHERE state = 'running' AND lease_expires_at <= now() \
             ORDER BY lease_expires_at LIMIT $1 FOR UPDATE SKIP LOCKED",
        )
        .bind(limit)
        .fetch_all(&mut *tx)
        .await
        .classify(self.target())?;
        let note = TransitionNote::by("stale-sweep").because("lease expired");
        let mut swept = Vec::with_capacity(rows.len());
        for row in rows {
            let attempt = codec::id(&row, "attempt_id").classify(self.target())?;
            apply(
                &mut tx,
                self.target(),
                attempt,
                AttemptState::Stale,
                &note,
                None,
            )
            .await?;
            swept.push(attempt);
        }
        tx.commit().await.classify(self.target())?;
        Ok(swept)
    }
}
