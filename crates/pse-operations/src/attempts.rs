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
pub use pse_model::generated::enums::{
    AttemptKind, DiagnosticCode, NativeRunState, NativeTermination, RuntimeTermination,
    TerminationClass, TrajectoryTermination,
};
pub use pse_model::generated::runtime::operational_attempts::RuntimeOperationalAttemptsRow;

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

/// Why work ended: exactly one typed termination, selected by its [`TerminationClass`]
/// (Plan 22 X4). Each class is its own stored column; no outcome is a free string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminationCode {
    /// The last native solver termination.
    Native(NativeTermination),
    /// The last run state, when no native termination was reported.
    RunState(NativeRunState),
    /// A trajectory's termination.
    Trajectory(TrajectoryTermination),
    /// An outcome the durable runtime owns (cancellation, infrastructure, ...).
    Runtime(RuntimeTermination),
    /// The diagnostic code of a failure no other class owns (X4).
    Rule(DiagnosticCode),
}

impl TerminationCode {
    /// The class selecting this termination's column.
    pub const fn class(&self) -> TerminationClass {
        match self {
            Self::Native(_) => TerminationClass::Native,
            Self::RunState(_) => TerminationClass::RunState,
            Self::Trajectory(_) => TerminationClass::Trajectory,
            Self::Runtime(_) => TerminationClass::Runtime,
            Self::Rule(_) => TerminationClass::Rule,
        }
    }

    /// The registry spelling of the carried value.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Native(value) => value.as_str(),
            Self::RunState(value) => value.as_str(),
            Self::Trajectory(value) => value.as_str(),
            Self::Runtime(value) => value.as_str(),
            Self::Rule(value) => value.as_str(),
        }
    }

    const fn native(&self) -> Option<NativeTermination> {
        if let Self::Native(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    const fn run_state(&self) -> Option<NativeRunState> {
        if let Self::RunState(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    const fn trajectory(&self) -> Option<TrajectoryTermination> {
        if let Self::Trajectory(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    const fn runtime(&self) -> Option<RuntimeTermination> {
        if let Self::Runtime(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    const fn rule(&self) -> Option<DiagnosticCode> {
        if let Self::Rule(value) = self {
            Some(*value)
        } else {
            None
        }
    }
}

/// A typed termination with its optional structured detail.
#[derive(Clone, Debug, PartialEq)]
pub struct Termination {
    /// The typed termination.
    pub code: TerminationCode,
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
    /// The latest heartbeat of the lease owner.
    pub heartbeat_at: Option<DateTime<Utc>>,
    /// The durable cancellation request (finding T03): the authority, not the notification.
    pub cancel_requested: bool,
    /// When cancellation was requested.
    pub cancel_requested_at: Option<DateTime<Utc>>,
    /// The termination, once work ended.
    pub termination: Option<Termination>,
    /// Registration time.
    pub created_at: DateTime<Utc>,
    /// The latest change of the stored row.
    pub updated_at: DateTime<Utc>,
    /// When work started.
    pub started_at: Option<DateTime<Utc>>,
    /// When work ended.
    pub finished_at: Option<DateTime<Utc>>,
}

impl FromRow<'_, PgRow> for AttemptRecord {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let termination = codec::opt_parsed::<TerminationClass>(row, "termination_class")?
            .map(|class| -> Result<Termination, sqlx::Error> {
                let code = match class {
                    TerminationClass::Native => {
                        TerminationCode::Native(codec::parsed(row, "termination_native")?)
                    }
                    TerminationClass::RunState => {
                        TerminationCode::RunState(codec::parsed(row, "termination_run_state")?)
                    }
                    TerminationClass::Trajectory => TerminationCode::Trajectory(codec::parsed(
                        row,
                        "termination_trajectory",
                    )?),
                    TerminationClass::Runtime => {
                        TerminationCode::Runtime(codec::parsed(row, "termination_runtime")?)
                    }
                    TerminationClass::Rule => {
                        TerminationCode::Rule(codec::parsed(row, "termination_rule")?)
                    }
                };
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
            heartbeat_at: row.try_get("heartbeat_at")?,
            cancel_requested: row.try_get("cancel_requested")?,
            cancel_requested_at: row.try_get("cancel_requested_at")?,
            termination,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            started_at: row.try_get("started_at")?,
            finished_at: row.try_get("finished_at")?,
        })
    }
}

impl AttemptRecord {
    /// This attempt as a `runtime.operational_attempts` row, the registry's reading of the
    /// stored columns: typed ids, timestamps in UTC microseconds (the store's precision),
    /// the typed termination columns and the termination detail as JSON text.
    pub fn row(&self) -> RuntimeOperationalAttemptsRow {
        let micros = |value: DateTime<Utc>| value.timestamp_micros();
        let opt = |value: Option<DateTime<Utc>>| value.map(micros);
        let code = self.termination.as_ref().map(|t| &t.code);
        RuntimeOperationalAttemptsRow {
            attempt_id: self.attempt_id.into(),
            run_id: self.run_id.into(),
            kind: self.kind,
            request_identity: self.request_identity,
            preparation_identity: self.preparation_identity,
            state: self.state,
            state_version: self.state_version,
            parent_attempt: self.parent_attempt.map(Into::into),
            worker: self.worker.clone(),
            lease_expires_at: opt(self.lease_expires_at),
            heartbeat_at: opt(self.heartbeat_at),
            cancel_requested: self.cancel_requested,
            cancel_requested_at: opt(self.cancel_requested_at),
            termination_class: code.map(TerminationCode::class),
            termination_native: code.and_then(TerminationCode::native),
            termination_run_state: code.and_then(TerminationCode::run_state),
            termination_trajectory: code.and_then(TerminationCode::trajectory),
            termination_runtime: code.and_then(TerminationCode::runtime),
            termination_rule: code.and_then(TerminationCode::rule),
            termination_detail: self
                .termination
                .as_ref()
                .and_then(|t| t.detail.as_ref())
                .map(ToString::to_string),
            created_at: micros(self.created_at),
            updated_at: micros(self.updated_at),
            started_at: opt(self.started_at),
            finished_at: opt(self.finished_at),
        }
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
// Enumerations are read as text until the statements take typed values (Plan 22 B2).
macro_rules! attempt_columns {
    () => {
        "attempt_id, run_id, kind::text AS kind, request_identity, preparation_identity, \
         state::text AS state, state_version, parent_attempt, worker, lease_expires_at, \
         heartbeat_at, cancel_requested, cancel_requested_at, \
         termination_class::text AS termination_class, \
         termination_native::text AS termination_native, \
         termination_run_state::text AS termination_run_state, \
         termination_trajectory::text AS termination_trajectory, \
         termination_runtime::text AS termination_runtime, \
         termination_rule::text AS termination_rule, \
         termination_detail, created_at, updated_at, started_at, finished_at"
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
         VALUES ($1, $2, $3::pse_ops.attempt_kind, $4, $5, $6::pse_ops.attempt_state, $7)",
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
         VALUES ($1, 0, NULL, $2::pse_ops.attempt_state, $3)",
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
        "SELECT state::text AS state, state_version FROM pse_ops.attempts \
         WHERE attempt_id = $1 FOR UPDATE",
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
    // A termination replaces every termination column at once, keeping the one-of rule.
    let code = note.termination.as_ref().map(|t| &t.code);
    sqlx::query(
        "UPDATE pse_ops.attempts SET \
             state = $2::pse_ops.attempt_state, state_version = $3, updated_at = now(), \
             worker = coalesce($4, worker), \
             lease_expires_at = CASE WHEN $4 IS NULL THEN NULL ELSE now() + $5 END, \
             heartbeat_at = CASE WHEN $4 IS NULL THEN heartbeat_at ELSE now() END, \
             started_at = CASE WHEN $4 IS NULL THEN started_at ELSE now() END, \
             finished_at = CASE WHEN $6 THEN coalesce(finished_at, now()) ELSE finished_at END, \
             termination_class = CASE WHEN $7 THEN $8::pse_ops.termination_class \
                 ELSE termination_class END, \
             termination_native = CASE WHEN $7 THEN $9::pse_ops.native_termination \
                 ELSE termination_native END, \
             termination_run_state = CASE WHEN $7 THEN $10::pse_ops.native_run_state \
                 ELSE termination_run_state END, \
             termination_trajectory = CASE WHEN $7 THEN $11::pse_ops.trajectory_termination \
                 ELSE termination_trajectory END, \
             termination_runtime = CASE WHEN $7 THEN $12::pse_ops.runtime_termination \
                 ELSE termination_runtime END, \
             termination_rule = CASE WHEN $7 THEN $13::pse_ops.diagnostic_code \
                 ELSE termination_rule END, \
             termination_detail = CASE WHEN $7 THEN $14 ELSE termination_detail END \
         WHERE attempt_id = $1",
    )
    .bind(codec::uuid(attempt))
    .bind(to.as_str())
    .bind(next)
    .bind(lease.map(|lease| lease.worker))
    .bind(lease.map(|lease| codec::interval(lease.duration)))
    .bind(to.ends_work())
    .bind(code.is_some())
    .bind(code.map(|code| code.class().as_str()))
    .bind(code.and_then(TerminationCode::native).map(NativeTermination::as_str))
    .bind(code.and_then(TerminationCode::run_state).map(NativeRunState::as_str))
    .bind(
        code.and_then(TerminationCode::trajectory)
            .map(TrajectoryTermination::as_str),
    )
    .bind(code.and_then(TerminationCode::runtime).map(RuntimeTermination::as_str))
    .bind(code.and_then(TerminationCode::rule).map(DiagnosticCode::as_str))
    .bind(note.termination.as_ref().and_then(|t| t.detail.as_ref()))
    .execute(&mut *conn)
    .await
    .classify(target)?;
    sqlx::query(
        "INSERT INTO pse_ops.attempt_transitions \
             (attempt_id, seq, from_state, to_state, actor, reason) \
         VALUES ($1, $2, $3::pse_ops.attempt_state, $4::pse_ops.attempt_state, $5, $6)",
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
            "SELECT attempt_id, seq, from_state::text AS from_state, to_state::text AS to_state, \
                 actor, reason, at \
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
               AND (cardinality($2::text[]) = 0 \
                    OR state = ANY($2::text[]::pse_ops.attempt_state[])) \
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
