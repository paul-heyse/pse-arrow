// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The attempt registry: creation, transitions with an audit row in the same
//! transaction, heartbeats and the stale sweep (DP-19). Every read returns the registry
//! row `runtime.operational_attempts` (or `..._attempt_transitions`) as stored.

use std::time::Duration;

use chrono::{DateTime, Utc};
use pse_ids::ContentHash;
use pse_operations_queries::client::Params as _;
use pse_operations_queries::queries::{attempts as statements, store};

use crate::error::{Classify, OperationsError, Target};
use crate::lifecycle::{self, AttemptState, Lifecycle as _};
use crate::store::Store;
pub use pse_model::generated::enums::{
    AttemptKind, DiagnosticCode, NativeRunState, NativeTermination, RuntimeTermination,
    TerminationClass, TrajectoryTermination,
};
pub use pse_model::generated::identities::{AttemptId, RunId};
pub use pse_model::generated::runtime::operational_attempt_transitions::RuntimeOperationalAttemptTransitionsRow;
pub use pse_model::generated::runtime::operational_attempts::RuntimeOperationalAttemptsRow;

/// A store transaction on a pooled connection.
pub(crate) type Tx<'a> = deadpool_postgres::Transaction<'a>;

/// A duration as integral microseconds, saturating: the store's interval unit.
pub(crate) fn micros(duration: Duration) -> i64 {
    i64::try_from(duration.as_micros()).unwrap_or(i64::MAX)
}

/// A registry timestamp (microseconds since the Unix epoch) as a UTC time.
pub(crate) fn utc(column: &'static str, micros: i64) -> Result<DateTime<Utc>, OperationsError> {
    DateTime::from_timestamp_micros(micros).ok_or_else(|| OperationsError::CorruptValue {
        column,
        detail: format!("{micros} µs is outside the representable time range"),
    })
}

/// An attempt registered before any effect; the runtime mints its identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewAttempt {
    /// The attempt identity (`UUIDv7`, minted by the runtime).
    pub attempt_id: AttemptId,
    /// The run the attempt belongs to.
    pub run_id: RunId,
    /// What the attempt computes.
    pub kind: AttemptKind,
    /// The request identity (blueprint §5.1).
    pub request_identity: ContentHash,
    /// The preparation identity, when preparation happened before registration.
    pub preparation_identity: Option<ContentHash>,
    /// The attempt this one supersedes or retries.
    pub parent_attempt: Option<AttemptId>,
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

    /// The termination a stored attempt records: the class selects exactly one typed
    /// column (the store's one-of rule), so a class without its column is corrupt.
    ///
    /// # Errors
    /// [`OperationsError::CorruptValue`] when the class's column is empty.
    pub fn of(row: &RuntimeOperationalAttemptsRow) -> Result<Option<Self>, OperationsError> {
        let missing = |class: TerminationClass| OperationsError::CorruptValue {
            column: "attempts.termination_class",
            detail: format!("class {} without its value", class.as_str()),
        };
        let Some(class) = row.termination_class else {
            return Ok(None);
        };
        Ok(Some(match class {
            TerminationClass::Native => {
                Self::Native(row.termination_native.ok_or_else(|| missing(class))?)
            }
            TerminationClass::RunState => {
                Self::RunState(row.termination_run_state.ok_or_else(|| missing(class))?)
            }
            TerminationClass::Trajectory => {
                Self::Trajectory(row.termination_trajectory.ok_or_else(|| missing(class))?)
            }
            TerminationClass::Runtime => {
                Self::Runtime(row.termination_runtime.ok_or_else(|| missing(class))?)
            }
            TerminationClass::Rule => {
                Self::Rule(row.termination_rule.ok_or_else(|| missing(class))?)
            }
        }))
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

/// Which attempts [`Attempts::list`] returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttemptFilter {
    /// Only the attempts of this run.
    pub run: Option<RunId>,
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

/// A lease to install when an attempt enters `running`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Lease<'a> {
    pub(crate) worker: &'a str,
    pub(crate) duration: Duration,
}

/// Queue a notification on `channel` with an identity as its payload, delivered when the
/// transaction commits.
pub(crate) async fn notify(
    tx: &Tx<'_>,
    target: &Target,
    channel: &str,
    payload: &str,
) -> Result<(), OperationsError> {
    store::notify()
        .params(tx, &store::NotifyParams { channel, payload })
        .one()
        .await
        .classify(target)?;
    Ok(())
}

/// Insert a planned attempt and its creation audit row.
pub(crate) async fn insert(
    tx: &Tx<'_>,
    target: &Target,
    attempt: &NewAttempt,
    actor: Option<&str>,
) -> Result<(), OperationsError> {
    statements::insert_attempt()
        .params(
            tx,
            &statements::InsertAttemptParams {
                attempt_id: attempt.attempt_id,
                run_id: attempt.run_id,
                kind: attempt.kind,
                request_identity: attempt.request_identity,
                preparation_identity: attempt.preparation_identity,
                state: lifecycle::INITIAL,
                parent_attempt: attempt.parent_attempt,
            },
        )
        .await
        .classify(target)?;
    statements::insert_transition()
        .params(
            tx,
            &statements::InsertTransitionParams {
                attempt_id: attempt.attempt_id,
                seq: 0,
                from_state: None,
                to_state: lifecycle::INITIAL,
                actor,
                reason: None::<&str>,
            },
        )
        .await
        .classify(target)?;
    Ok(())
}

/// Lock the attempt row and return its state, version and kind.
pub(crate) async fn lock(
    tx: &Tx<'_>,
    target: &Target,
    attempt: AttemptId,
) -> Result<(AttemptState, i32, AttemptKind), OperationsError> {
    let locked = statements::lock_attempt()
        .bind(tx, &attempt)
        .opt()
        .await
        .classify(target)?
        .ok_or_else(|| OperationsError::NotFound {
            entity: "attempt",
            id: attempt.to_string(),
        })?;
    Ok((locked.state, locked.state_version, locked.kind))
}

/// Lock the attempt row and return its state and version.
pub(crate) async fn lock_state(
    tx: &Tx<'_>,
    target: &Target,
    attempt: AttemptId,
) -> Result<(AttemptState, i32), OperationsError> {
    let (state, version, _) = lock(tx, target, attempt).await?;
    Ok((state, version))
}

/// Apply one transition under the attempt's row lock: check it against the transition
/// table of the attempt's kind, update the row and append the audit row. Returns the
/// previous state.
pub(crate) async fn apply(
    tx: &Tx<'_>,
    target: &Target,
    attempt: AttemptId,
    to: AttemptState,
    note: &TransitionNote,
    lease: Option<Lease<'_>>,
) -> Result<AttemptState, OperationsError> {
    let (from, version, kind) = lock(tx, target, attempt).await?;
    if !lifecycle::legal(kind, from, to) {
        return Err(OperationsError::IllegalTransition { attempt, from, to });
    }
    if (to == AttemptState::Running) != lease.is_some() {
        return Err(OperationsError::InvalidRequest {
            reason: format!("attempt {attempt}: a lease is required exactly when entering running"),
        });
    }
    let next = version + 1;
    statements::update_state()
        .params(
            tx,
            &statements::UpdateStateParams {
                state: to,
                state_version: next,
                worker: lease.map(|lease| lease.worker),
                lease_us: lease.map(|lease| micros(lease.duration)),
                ends_work: to.ends_work(),
                attempt_id: attempt,
            },
        )
        .await
        .classify(target)?;
    if let Some(termination) = &note.termination {
        // A termination replaces every termination column at once, keeping the one-of rule.
        let code = &termination.code;
        statements::set_termination()
            .params(
                tx,
                &statements::SetTerminationParams {
                    termination_class: code.class(),
                    termination_native: code.native(),
                    termination_run_state: code.run_state(),
                    termination_trajectory: code.trajectory(),
                    termination_runtime: code.runtime(),
                    termination_rule: code.rule(),
                    termination_detail: termination.detail.as_ref(),
                    attempt_id: attempt,
                },
            )
            .await
            .classify(target)?;
    }
    statements::insert_transition()
        .params(
            tx,
            &statements::InsertTransitionParams {
                attempt_id: attempt,
                seq: next,
                from_state: Some(from),
                to_state: to,
                actor: note.actor.as_deref(),
                reason: note.reason.as_deref(),
            },
        )
        .await
        .classify(target)?;
    if to.ends_work() {
        // A stream watcher learns that no further progress will come.
        notify(
            tx,
            target,
            crate::streams::PROGRESS_CHANNEL,
            &attempt.to_string(),
        )
        .await?;
    }
    Ok(from)
}

/// Read one attempt.
pub(crate) async fn fetch(
    tx: &Tx<'_>,
    target: &Target,
    attempt: AttemptId,
) -> Result<RuntimeOperationalAttemptsRow, OperationsError> {
    statements::attempt()
        .bind(tx, &attempt)
        .opt()
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
    ) -> Result<RuntimeOperationalAttemptsRow, OperationsError> {
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(self.target())?;
        insert(&tx, self.target(), attempt, actor).await?;
        let row = fetch(&tx, self.target(), attempt.attempt_id).await?;
        tx.commit().await.classify(self.target())?;
        Ok(row)
    }

    /// Read one attempt.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`]; classified driver failures.
    pub async fn get(
        &self,
        attempt: AttemptId,
    ) -> Result<RuntimeOperationalAttemptsRow, OperationsError> {
        let client = self.store.client().await?;
        statements::attempt()
            .bind(&client, &attempt)
            .opt()
            .await
            .classify(self.target())?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "attempt",
                id: attempt.to_string(),
            })
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
        attempt: AttemptId,
        to: AttemptState,
        note: &TransitionNote,
    ) -> Result<RuntimeOperationalAttemptsRow, OperationsError> {
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(self.target())?;
        apply(&tx, self.target(), attempt, to, note, None).await?;
        let row = fetch(&tx, self.target(), attempt).await?;
        tx.commit().await.classify(self.target())?;
        Ok(row)
    }

    /// Move a queued attempt to `running` under a lease owned by `worker`.
    ///
    /// # Errors
    ///
    /// As for [`Attempts::transition`].
    pub async fn start(
        &self,
        attempt: AttemptId,
        worker: &str,
        lease: Duration,
    ) -> Result<RuntimeOperationalAttemptsRow, OperationsError> {
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(self.target())?;
        let lease = Lease {
            worker,
            duration: lease,
        };
        let note = TransitionNote::by(worker);
        apply(
            &tx,
            self.target(),
            attempt,
            AttemptState::Running,
            &note,
            Some(lease),
        )
        .await?;
        let row = fetch(&tx, self.target(), attempt).await?;
        tx.commit().await.classify(self.target())?;
        Ok(row)
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
        attempt: AttemptId,
        worker: &str,
        lease: Duration,
    ) -> Result<HeartbeatAck, OperationsError> {
        let client = self.store.client().await?;
        let ack = statements::heartbeat()
            .params(
                &client,
                &statements::HeartbeatParams {
                    lease_us: micros(lease),
                    attempt_id: attempt,
                    worker,
                    running: AttemptState::Running,
                },
            )
            .opt()
            .await
            .classify(self.target())?
            .ok_or_else(|| OperationsError::LeaseLost {
                attempt,
                worker: worker.to_owned(),
            })?;
        Ok(HeartbeatAck {
            cancel_requested: ack.cancel_requested,
            lease_expires_at: ack.lease_expires_at,
        })
    }

    /// The audit trail of one attempt, in order.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn history(
        &self,
        attempt: AttemptId,
    ) -> Result<Vec<RuntimeOperationalAttemptTransitionsRow>, OperationsError> {
        let client = self.store.client().await?;
        statements::history()
            .bind(&client, &attempt)
            .all()
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
    ) -> Result<Vec<RuntimeOperationalAttemptsRow>, OperationsError> {
        if filter.limit <= 0 {
            return Err(OperationsError::InvalidRequest {
                reason: format!("attempt listing limit {} is not positive", filter.limit),
            });
        }
        let client = self.store.client().await?;
        statements::list_attempts()
            .params(
                &client,
                &statements::ListAttemptsParams {
                    run: filter.run,
                    states: filter.states.as_slice(),
                    limit: filter.limit,
                },
            )
            .all()
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
    pub async fn sweep_stale(&self, limit: i64) -> Result<Vec<AttemptId>, OperationsError> {
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(self.target())?;
        let expired = statements::expired_attempts()
            .bind(&tx, &limit)
            .all()
            .await
            .classify(self.target())?;
        let note = TransitionNote::by("stale-sweep").because("lease expired");
        let mut swept = Vec::with_capacity(expired.len());
        for attempt in expired.into_iter().map(AttemptId::from_id) {
            apply(
                &tx,
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
