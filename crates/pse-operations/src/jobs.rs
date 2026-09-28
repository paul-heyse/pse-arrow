// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The durable job queue: idempotent enqueue, `FOR UPDATE SKIP LOCKED` claims, completion,
//! and requeue under an explicit retry policy. Each try runs as a new attempt
//! (ADR-0112 Outcome 14).
//!
//! Lock order is job row, then attempt row, everywhere a function takes both.

use std::time::Duration;

use chrono::{DateTime, Utc};
use pse_ids::SemanticId;
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgConnection, Row};

use crate::attempts::{self, Lease, NewAttempt, TransitionNote};
use crate::codec;
use crate::error::{Classify, OperationsError, Target};
use crate::lifecycle::AttemptState;
use crate::store::Store;

/// The notification channel for new jobs; the payload is the job identity.
pub const JOBS_CHANNEL: &str = "pse_ops_jobs";

/// How often a job may be tried and how long to wait between tries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetryPolicy {
    /// The maximum number of tries, including the first; at least 1.
    pub max_tries: u32,
    /// The delay before the second try; doubled for each later try.
    pub backoff: Duration,
    /// The longest delay between tries.
    pub backoff_cap: Duration,
}

impl RetryPolicy {
    /// One try, no retry.
    pub const ONCE: Self = Self {
        max_tries: 1,
        backoff: Duration::ZERO,
        backoff_cap: Duration::ZERO,
    };

    /// The delay before try number `next_try` (1-based), or `None` when the policy is
    /// exhausted. The first try has no delay; later tries back off exponentially up to
    /// the cap.
    pub fn delay_before(&self, next_try: u32) -> Option<Duration> {
        if next_try > self.max_tries {
            return None;
        }
        let Some(retries) = next_try.checked_sub(2) else {
            return Some(Duration::ZERO);
        };
        let factor = 2_u32.checked_pow(retries).unwrap_or(u32::MAX);
        Some(
            self.backoff
                .checked_mul(factor)
                .unwrap_or(self.backoff_cap)
                .min(self.backoff_cap),
        )
    }

    fn validate(&self) -> Result<(), OperationsError> {
        if self.max_tries == 0 || self.backoff_cap < self.backoff {
            return Err(OperationsError::InvalidRequest {
                reason: format!(
                    "retry policy needs max_tries >= 1 and backoff_cap >= backoff: {self:?}"
                ),
            });
        }
        Ok(())
    }
}

/// The queue state of a job (registry enumeration `JobState`): queued, running, completed
/// (with a completed or partial attempt), failed (not retried further) or cancelled. The
/// attempt carries the lifecycle of each try.
pub use pse_model::generated::enums::JobState;

/// Work to enqueue together with its first attempt.
#[derive(Clone, Debug, PartialEq)]
pub struct NewJob {
    /// The first attempt, minted by the runtime.
    pub attempt: NewAttempt,
    /// Unique per logical request: a repeated enqueue returns the existing job.
    pub idempotency_key: String,
    /// The payload format version; a worker refuses versions it does not know.
    pub payload_version: i32,
    /// The versioned payload (source bundle, case, profile).
    pub payload: serde_json::Value,
    /// Higher runs first.
    pub priority: i32,
    /// Tries and backoff.
    pub retry: RetryPolicy,
}

/// The result of an enqueue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enqueued {
    /// A new job and attempt were stored.
    Created {
        /// The job.
        job_id: SemanticId,
        /// Its first attempt.
        attempt_id: SemanticId,
    },
    /// The idempotency key already named a job; nothing was stored.
    Existing {
        /// The existing job.
        job_id: SemanticId,
        /// Its current attempt.
        attempt_id: SemanticId,
    },
}

impl Enqueued {
    /// The job, created or existing.
    pub const fn job_id(&self) -> SemanticId {
        match self {
            Self::Created { job_id, .. } | Self::Existing { job_id, .. } => *job_id,
        }
    }

    /// The job's current attempt.
    pub const fn attempt_id(&self) -> SemanticId {
        match self {
            Self::Created { attempt_id, .. } | Self::Existing { attempt_id, .. } => *attempt_id,
        }
    }
}

/// A job a worker now owns.
#[derive(Clone, Debug, PartialEq)]
pub struct ClaimedJob {
    /// The job.
    pub job_id: SemanticId,
    /// The attempt this try runs as.
    pub attempt_id: SemanticId,
    /// The payload format version.
    pub payload_version: i32,
    /// The payload.
    pub payload: serde_json::Value,
    /// Which try this is, 1-based.
    pub try_number: i32,
    /// The lease expiry; extend it with [`crate::attempts::Attempts::heartbeat`].
    pub lease_expires_at: DateTime<Utc>,
}

/// A job as stored.
#[derive(Clone, Debug, PartialEq)]
pub struct JobRecord {
    /// The job.
    pub job_id: SemanticId,
    /// Its current attempt.
    pub attempt_id: SemanticId,
    /// Its idempotency key.
    pub idempotency_key: String,
    /// The payload format version.
    pub payload_version: i32,
    /// The priority.
    pub priority: i32,
    /// The queue state.
    pub state: JobState,
    /// Tries so far.
    pub tries: i32,
    /// The retry policy.
    pub retry: RetryPolicy,
    /// When the job may next be claimed.
    pub available_at: DateTime<Utc>,
    /// Why the last try ended without success.
    pub last_error: Option<String>,
}

/// A backoff column: integral microseconds.
fn duration(row: &PgRow, column: &str) -> Result<Duration, sqlx::Error> {
    let micros: i64 = row.try_get(column)?;
    let micros = u64::try_from(micros).map_err(|_| sqlx::Error::ColumnDecode {
        index: column.to_owned(),
        source: "negative backoff".into(),
    })?;
    Ok(Duration::from_micros(micros))
}

/// The backoff column value of a duration: integral microseconds, saturating.
fn micros(duration: Duration) -> i64 {
    i64::try_from(duration.as_micros()).unwrap_or(i64::MAX)
}

fn policy(row: &PgRow) -> Result<RetryPolicy, sqlx::Error> {
    let max_tries: i32 = row.try_get("max_tries")?;
    Ok(RetryPolicy {
        max_tries: u32::try_from(max_tries).map_err(|_| sqlx::Error::ColumnDecode {
            index: "max_tries".to_owned(),
            source: "negative max_tries".into(),
        })?,
        backoff: duration(row, "backoff_base_us")?,
        backoff_cap: duration(row, "backoff_cap_us")?,
    })
}

impl FromRow<'_, PgRow> for JobRecord {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            job_id: codec::id(row, "job_id")?,
            attempt_id: codec::id(row, "attempt_id")?,
            idempotency_key: row.try_get("idempotency_key")?,
            payload_version: row.try_get("payload_version")?,
            priority: row.try_get("priority")?,
            state: codec::parsed(row, "state")?,
            tries: row.try_get("tries")?,
            retry: policy(row)?,
            available_at: row.try_get("available_at")?,
            last_error: row.try_get("last_error")?,
        })
    }
}

/// How a worker ends its try.
#[derive(Clone, Debug, PartialEq)]
pub struct JobOutcome {
    /// Completed, partial, failed or cancelled.
    pub state: AttemptState,
    /// The audit note, with the termination.
    pub note: TransitionNote,
    /// For a failed try: retry as this new attempt (minted by the caller) if the policy
    /// still allows another try.
    pub retry_as: Option<SemanticId>,
}

/// What finishing a try did to the job.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finished {
    /// The job ended in this state.
    Ended(JobState),
    /// The job was requeued as a new attempt.
    Requeued {
        /// The new attempt.
        attempt_id: SemanticId,
        /// When it may be claimed.
        available_at: DateTime<Utc>,
    },
}

/// One job handled by [`Jobs::requeue_expired`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requeue {
    /// The job.
    pub job_id: SemanticId,
    /// The attempt whose lease expired; now stale or superseded.
    pub stale_attempt: SemanticId,
    /// What happened to the job.
    pub outcome: Finished,
}

struct LockedJob {
    job_id: SemanticId,
    attempt_id: SemanticId,
    state: JobState,
    tries: i32,
    retry: RetryPolicy,
}

async fn lock_job(
    conn: &mut PgConnection,
    target: &Target,
    job: SemanticId,
) -> Result<LockedJob, OperationsError> {
    let row = sqlx::query(
        "SELECT job_id, attempt_id, state, tries, max_tries, backoff_base_us, backoff_cap_us \
         FROM pse_ops.jobs WHERE job_id = $1 FOR UPDATE",
    )
    .bind(codec::uuid(job))
    .fetch_optional(&mut *conn)
    .await
    .classify(target)?
    .ok_or_else(|| OperationsError::NotFound {
        entity: "job",
        id: job.to_string(),
    })?;
    Ok(LockedJob {
        job_id: codec::id(&row, "job_id").classify(target)?,
        attempt_id: codec::id(&row, "attempt_id").classify(target)?,
        state: codec::parsed(&row, "state").classify(target)?,
        tries: row.try_get("tries").classify(target)?,
        retry: policy(&row).classify(target)?,
    })
}

async fn set_job_state(
    conn: &mut PgConnection,
    target: &Target,
    job: SemanticId,
    state: JobState,
    error: Option<&str>,
) -> Result<(), OperationsError> {
    sqlx::query(
        "UPDATE pse_ops.jobs SET state = $2, last_error = coalesce($3, last_error), \
         updated_at = now() WHERE job_id = $1",
    )
    .bind(codec::uuid(job))
    .bind(state.as_str())
    .bind(error)
    .execute(&mut *conn)
    .await
    .classify(target)?;
    Ok(())
}

/// Requeue `job` as `next`, a new attempt whose parent is `previous`, when the retry
/// policy allows another try; otherwise end the job as failed. A cancellation requested
/// on the previous attempt ends the job as cancelled instead.
async fn requeue(
    conn: &mut PgConnection,
    target: &Target,
    job: &LockedJob,
    previous: SemanticId,
    next: SemanticId,
    reason: &str,
) -> Result<Finished, OperationsError> {
    let prior = attempts::fetch(conn, target, previous).await?;
    if prior.cancel_requested {
        set_job_state(conn, target, job.job_id, JobState::Cancelled, Some(reason)).await?;
        return Ok(Finished::Ended(JobState::Cancelled));
    }
    let next_try = u32::try_from(job.tries)
        .unwrap_or(u32::MAX)
        .saturating_add(1);
    let Some(delay) = job.retry.delay_before(next_try) else {
        let exhausted = format!("{reason}; retries exhausted after {} tries", job.tries);
        set_job_state(conn, target, job.job_id, JobState::Failed, Some(&exhausted)).await?;
        return Ok(Finished::Ended(JobState::Failed));
    };
    let attempt = NewAttempt {
        attempt_id: next,
        run_id: prior.run_id,
        kind: prior.kind,
        request_identity: prior.request_identity,
        preparation_identity: prior.preparation_identity,
        parent_attempt: Some(previous),
    };
    attempts::insert(conn, target, &attempt, Some("retry")).await?;
    let queued = TransitionNote::by("retry").because(format!("try {next_try} after {previous}"));
    attempts::apply(conn, target, next, AttemptState::Queued, &queued, None).await?;
    if prior.state == AttemptState::Stale {
        let superseded = TransitionNote::by("retry").because(format!("superseded by {next}"));
        attempts::apply(
            conn,
            target,
            previous,
            AttemptState::Superseded,
            &superseded,
            None,
        )
        .await?;
    }
    let available_at: DateTime<Utc> = sqlx::query_scalar(
        "UPDATE pse_ops.jobs SET attempt_id = $2, state = 'queued', \
             available_at = now() + $3, last_error = $4, updated_at = now() \
         WHERE job_id = $1 RETURNING available_at",
    )
    .bind(codec::uuid(job.job_id))
    .bind(codec::uuid(next))
    .bind(codec::interval(delay))
    .bind(reason)
    .fetch_one(&mut *conn)
    .await
    .classify(target)?;
    Ok(Finished::Requeued {
        attempt_id: next,
        available_at,
    })
}

/// The job repository.
#[derive(Clone, Copy, Debug)]
pub struct Jobs<'s> {
    store: &'s Store,
}

impl<'s> Jobs<'s> {
    pub(crate) const fn new(store: &'s Store) -> Self {
        Self { store }
    }

    fn target(&self) -> &Target {
        self.store.target()
    }

    /// Enqueue a job and its first attempt (planned, then queued) in one transaction. An
    /// idempotency key that already names a job returns that job and stores nothing.
    ///
    /// # Errors
    ///
    /// [`OperationsError::InvalidRequest`] for an invalid retry policy or payload version;
    /// [`OperationsError::Duplicate`] when the attempt identity already exists; classified
    /// driver failures.
    pub async fn enqueue(&self, job: &NewJob) -> Result<Enqueued, OperationsError> {
        job.retry.validate()?;
        if job.payload_version <= 0 {
            return Err(OperationsError::InvalidRequest {
                reason: format!("payload version {} is not positive", job.payload_version),
            });
        }
        if let Some(existing) = self.find_by_key(&job.idempotency_key).await? {
            return Ok(existing);
        }
        let target = self.target();
        let mut tx = self.store.pool().begin().await.classify(target)?;
        attempts::insert(&mut tx, target, &job.attempt, Some("enqueue")).await?;
        let note = TransitionNote::by("enqueue");
        attempts::apply(
            &mut tx,
            target,
            job.attempt.attempt_id,
            AttemptState::Queued,
            &note,
            None,
        )
        .await?;
        let max_tries = i32::try_from(job.retry.max_tries).unwrap_or(i32::MAX);
        let created: Option<uuid::Uuid> = sqlx::query_scalar(
            "INSERT INTO pse_ops.jobs (attempt_id, idempotency_key, payload_version, payload, \
                 priority, state, max_tries, backoff_base_us, backoff_cap_us) \
             VALUES ($1, $2, $3, $4, $5, 'queued', $6, $7, $8) \
             ON CONFLICT (idempotency_key) DO NOTHING RETURNING job_id",
        )
        .bind(codec::uuid(job.attempt.attempt_id))
        .bind(&job.idempotency_key)
        .bind(job.payload_version)
        .bind(&job.payload)
        .bind(job.priority)
        .bind(max_tries)
        .bind(micros(job.retry.backoff))
        .bind(micros(job.retry.backoff_cap))
        .fetch_optional(&mut *tx)
        .await
        .classify(target)?;
        let Some(job_id) = created else {
            // A concurrent enqueue with the same key committed first: undo our attempt.
            tx.rollback().await.classify(target)?;
            return self
                .find_by_key(&job.idempotency_key)
                .await?
                .ok_or_else(|| OperationsError::NotFound {
                    entity: "job with idempotency key",
                    id: job.idempotency_key.clone(),
                });
        };
        // Wakes idle workers when the transaction commits; workers also poll, so a lost
        // notification only delays a claim.
        sqlx::query("SELECT pg_notify($1, $2)")
            .bind(JOBS_CHANNEL)
            .bind(job_id.to_string())
            .execute(&mut *tx)
            .await
            .classify(target)?;
        tx.commit().await.classify(target)?;
        Ok(Enqueued::Created {
            job_id: SemanticId::from_bytes(job_id.into_bytes()),
            attempt_id: job.attempt.attempt_id,
        })
    }

    async fn find_by_key(&self, key: &str) -> Result<Option<Enqueued>, OperationsError> {
        let row =
            sqlx::query("SELECT job_id, attempt_id FROM pse_ops.jobs WHERE idempotency_key = $1")
                .bind(key)
                .fetch_optional(self.store.pool())
                .await
                .classify(self.target())?;
        row.map(|row| {
            Ok(Enqueued::Existing {
                job_id: codec::id(&row, "job_id")?,
                attempt_id: codec::id(&row, "attempt_id")?,
            })
        })
        .transpose()
        .classify(self.target())
    }

    /// Read one job.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`]; classified driver failures.
    pub async fn get(&self, job: SemanticId) -> Result<JobRecord, OperationsError> {
        sqlx::query_as(
            "SELECT job_id, attempt_id, idempotency_key, payload_version, priority, state, \
                 tries, max_tries, backoff_base_us, backoff_cap_us, available_at, last_error \
             FROM pse_ops.jobs WHERE job_id = $1",
        )
        .bind(codec::uuid(job))
        .fetch_optional(self.store.pool())
        .await
        .classify(self.target())?
        .ok_or_else(|| OperationsError::NotFound {
            entity: "job",
            id: job.to_string(),
        })
    }

    /// Claim the next available job for `worker`: the highest priority, then the oldest
    /// availability. Rows locked by other claimers are skipped, so two workers never
    /// claim the same job. The job's attempt moves to `running` under a lease.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn claim(
        &self,
        worker: &str,
        lease: Duration,
    ) -> Result<Option<ClaimedJob>, OperationsError> {
        let target = self.target();
        let mut tx = self.store.pool().begin().await.classify(target)?;
        let row = sqlx::query(
            "WITH next AS ( \
                 SELECT job_id FROM pse_ops.jobs \
                 WHERE state = 'queued' AND available_at <= now() \
                 ORDER BY priority DESC, available_at, job_id \
                 LIMIT 1 FOR UPDATE SKIP LOCKED \
             ) \
             UPDATE pse_ops.jobs AS j SET state = 'running', tries = j.tries + 1, updated_at = now() \
             FROM next WHERE j.job_id = next.job_id \
             RETURNING j.job_id, j.attempt_id, j.payload_version, j.payload, j.tries",
        )
        .fetch_optional(&mut *tx)
        .await
        .classify(target)?;
        let Some(row) = row else {
            tx.rollback().await.classify(target)?;
            return Ok(None);
        };
        let job_id = codec::id(&row, "job_id").classify(target)?;
        let attempt_id = codec::id(&row, "attempt_id").classify(target)?;
        let try_number: i32 = row.try_get("tries").classify(target)?;
        let note =
            TransitionNote::by(worker).because(format!("claimed job {job_id}, try {try_number}"));
        attempts::apply(
            &mut tx,
            target,
            attempt_id,
            AttemptState::Running,
            &note,
            Some(Lease {
                worker,
                duration: lease,
            }),
        )
        .await?;
        let lease_expires_at: DateTime<Utc> = sqlx::query_scalar(
            "SELECT lease_expires_at FROM pse_ops.attempts WHERE attempt_id = $1",
        )
        .bind(codec::uuid(attempt_id))
        .fetch_one(&mut *tx)
        .await
        .classify(target)?;
        tx.commit().await.classify(target)?;
        Ok(Some(ClaimedJob {
            job_id,
            attempt_id,
            payload_version: row.try_get("payload_version").classify(target)?,
            payload: row.try_get("payload").classify(target)?,
            try_number,
            lease_expires_at,
        }))
    }

    /// End the worker's try. A failed try with [`JobOutcome::retry_as`] is requeued as that
    /// new attempt while the retry policy allows; otherwise the job ends.
    ///
    /// # Errors
    ///
    /// [`OperationsError::LeaseLost`] when `worker` does not own the running attempt;
    /// [`OperationsError::IllegalTransition`] or [`OperationsError::InvalidRequest`] for an
    /// outcome that does not end work; classified driver failures.
    pub async fn finish(
        &self,
        job: SemanticId,
        worker: &str,
        outcome: &JobOutcome,
    ) -> Result<Finished, OperationsError> {
        if !matches!(
            outcome.state,
            AttemptState::Completed
                | AttemptState::Partial
                | AttemptState::Failed
                | AttemptState::Cancelled
        ) {
            return Err(OperationsError::InvalidRequest {
                reason: format!("a worker cannot finish a try as {}", outcome.state.as_str()),
            });
        }
        let target = self.target();
        let mut tx = self.store.pool().begin().await.classify(target)?;
        let locked = lock_job(&mut tx, target, job).await?;
        let owner: Option<String> = sqlx::query_scalar(
            "SELECT worker FROM pse_ops.attempts \
             WHERE attempt_id = $1 AND state = 'running' FOR UPDATE",
        )
        .bind(codec::uuid(locked.attempt_id))
        .fetch_optional(&mut *tx)
        .await
        .classify(target)?
        .flatten();
        if locked.state != JobState::Running || owner.as_deref() != Some(worker) {
            return Err(OperationsError::LeaseLost {
                attempt: locked.attempt_id,
                worker: worker.to_owned(),
            });
        }
        attempts::apply(
            &mut tx,
            target,
            locked.attempt_id,
            outcome.state,
            &outcome.note,
            None,
        )
        .await?;
        let finished = match (outcome.state, outcome.retry_as) {
            (AttemptState::Failed, Some(next)) => {
                let reason = outcome
                    .note
                    .reason
                    .clone()
                    .unwrap_or_else(|| "try failed".to_owned());
                requeue(&mut tx, target, &locked, locked.attempt_id, next, &reason).await?
            }
            (state, _) => {
                let ended = match state {
                    AttemptState::Completed | AttemptState::Partial => JobState::Completed,
                    AttemptState::Cancelled => JobState::Cancelled,
                    _ => JobState::Failed,
                };
                set_job_state(&mut tx, target, job, ended, outcome.note.reason.as_deref()).await?;
                Finished::Ended(ended)
            }
        };
        tx.commit().await.classify(target)?;
        Ok(finished)
    }

    /// Recover jobs whose worker vanished: each running job whose attempt's lease expired
    /// (or whose attempt is already stale) has that attempt marked stale and is requeued
    /// as a new attempt minted by `mint`, while the retry policy allows; otherwise the job
    /// fails. At most `limit` jobs per call; jobs locked elsewhere are skipped.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn requeue_expired(
        &self,
        limit: i64,
        mut mint: impl FnMut() -> SemanticId,
    ) -> Result<Vec<Requeue>, OperationsError> {
        let target = self.target();
        let mut tx = self.store.pool().begin().await.classify(target)?;
        let rows = sqlx::query(
            "SELECT j.job_id FROM pse_ops.jobs AS j \
             JOIN pse_ops.attempts AS a ON a.attempt_id = j.attempt_id \
             WHERE j.state = 'running' \
               AND (a.state = 'stale' OR (a.state = 'running' AND a.lease_expires_at <= now())) \
             ORDER BY j.job_id LIMIT $1 FOR UPDATE OF j SKIP LOCKED",
        )
        .bind(limit)
        .fetch_all(&mut *tx)
        .await
        .classify(target)?;
        let mut handled = Vec::with_capacity(rows.len());
        for row in rows {
            let job_id = codec::id(&row, "job_id").classify(target)?;
            let locked = lock_job(&mut tx, target, job_id).await?;
            let (state, _) = attempts::lock_state(&mut tx, target, locked.attempt_id).await?;
            if state == AttemptState::Running {
                let note = TransitionNote::by("stale-sweep").because("lease expired");
                attempts::apply(
                    &mut tx,
                    target,
                    locked.attempt_id,
                    AttemptState::Stale,
                    &note,
                    None,
                )
                .await?;
            }
            let outcome = requeue(
                &mut tx,
                target,
                &locked,
                locked.attempt_id,
                mint(),
                "lease expired",
            )
            .await?;
            handled.push(Requeue {
                job_id,
                stale_attempt: locked.attempt_id,
                outcome,
            });
        }
        tx.commit().await.classify(target)?;
        Ok(handled)
    }
}

#[cfg(test)]
mod retry_unit {
    use super::*;

    #[test]
    fn backoff_doubles_up_to_the_cap_and_stops_at_max_tries() {
        let policy = RetryPolicy {
            max_tries: 5,
            backoff: Duration::from_secs(1),
            backoff_cap: Duration::from_secs(5),
        };
        let delays: Vec<Option<Duration>> = (1..=6).map(|n| policy.delay_before(n)).collect();
        assert_eq!(
            delays,
            [
                Some(Duration::ZERO),
                Some(Duration::from_secs(1)),
                Some(Duration::from_secs(2)),
                Some(Duration::from_secs(4)),
                Some(Duration::from_secs(5)),
                None,
            ]
        );
        assert_eq!(RetryPolicy::ONCE.delay_before(2), None);
    }

    #[test]
    fn invalid_policies_are_refused() {
        assert!(
            RetryPolicy {
                max_tries: 0,
                ..RetryPolicy::ONCE
            }
            .validate()
            .is_err()
        );
        assert!(
            RetryPolicy {
                max_tries: 2,
                backoff: Duration::from_secs(2),
                backoff_cap: Duration::from_secs(1),
            }
            .validate()
            .is_err()
        );
        assert!(RetryPolicy::ONCE.validate().is_ok());
    }
}
