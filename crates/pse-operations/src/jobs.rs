// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The durable job queue: idempotent enqueue, `FOR UPDATE SKIP LOCKED` claims, completion,
//! and requeue under an explicit retry policy. Each try runs as a new attempt
//! (ADR-0114 Outcome 14).
//!
//! Lock order is job row, then attempt row, everywhere a function takes both.

use std::time::Duration;

use chrono::{DateTime, Utc};
use pse_operations_queries::client::Params as _;
use pse_operations_queries::queries::jobs as statements;

use crate::attempts::{self, AttemptId, Lease, NewAttempt, RunId, TransitionNote, Tx, micros, utc};
use crate::error::{Classify, OperationsError, Target};
use crate::lifecycle::AttemptState;
use crate::store::Store;
/// The queue state of a job (registry enumeration `JobState`): queued, running, completed
/// (with a completed or partial attempt), failed (not retried further) or cancelled. The
/// attempt carries the lifecycle of each try.
pub use pse_model::generated::enums::JobState;
pub use pse_model::generated::identities::JobId;
pub use pse_model::generated::runtime::operational_jobs::RuntimeOperationalJobsRow;

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

    /// The policy a stored job carries.
    ///
    /// # Errors
    /// [`OperationsError::CorruptValue`] for a negative count or backoff.
    pub fn of(job: &RuntimeOperationalJobsRow) -> Result<Self, OperationsError> {
        let duration = |column: &'static str, micros: i64| {
            u64::try_from(micros)
                .map(Duration::from_micros)
                .map_err(|_| OperationsError::CorruptValue {
                    column,
                    detail: format!("negative backoff {micros}"),
                })
        };
        Ok(Self {
            max_tries: u32::try_from(job.max_tries).map_err(|_| OperationsError::CorruptValue {
                column: "jobs.max_tries",
                detail: format!("negative max_tries {}", job.max_tries),
            })?,
            backoff: duration("jobs.backoff_base_us", job.backoff_base_us)?,
            backoff_cap: duration("jobs.backoff_cap_us", job.backoff_cap_us)?,
        })
    }

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

/// Which jobs [`Jobs::list`] returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobFilter {
    /// Only jobs in these states; every state when empty.
    pub states: Vec<JobState>,
    /// At most this many, newest first.
    pub limit: i64,
}

impl JobFilter {
    /// The newest `limit` jobs of every state.
    pub const fn newest(limit: i64) -> Self {
        Self {
            states: Vec::new(),
            limit,
        }
    }
}

/// The result of an enqueue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enqueued {
    /// A new job and attempt were stored.
    Created {
        /// The job.
        job_id: JobId,
        /// Its first attempt.
        attempt_id: AttemptId,
    },
    /// The idempotency key already named a job; nothing was stored.
    Existing {
        /// The existing job.
        job_id: JobId,
        /// Its current attempt.
        attempt_id: AttemptId,
    },
}

impl Enqueued {
    /// The job, created or existing.
    pub const fn job_id(&self) -> JobId {
        match self {
            Self::Created { job_id, .. } | Self::Existing { job_id, .. } => *job_id,
        }
    }

    /// The job's current attempt.
    pub const fn attempt_id(&self) -> AttemptId {
        match self {
            Self::Created { attempt_id, .. } | Self::Existing { attempt_id, .. } => *attempt_id,
        }
    }
}

/// A job a worker now owns.
#[derive(Clone, Debug, PartialEq)]
pub struct ClaimedJob {
    /// The job.
    pub job_id: JobId,
    /// The attempt this try runs as.
    pub attempt_id: AttemptId,
    /// The run the job's attempts try: a retried try runs as the same run.
    pub run_id: RunId,
    /// The try this one supersedes, for a requeued job: its incumbents are where a
    /// resumed solve starts (Plan 22 G8). `None` for the first try.
    pub parent_attempt: Option<AttemptId>,
    /// The payload format version.
    pub payload_version: i32,
    /// The payload.
    pub payload: serde_json::Value,
    /// Which try this is, 1-based.
    pub try_number: i32,
    /// The lease expiry; extend it with [`crate::attempts::Attempts::heartbeat`].
    pub lease_expires_at: DateTime<Utc>,
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
    pub retry_as: Option<AttemptId>,
    /// The result members a completed study point's try wrote under its study's
    /// publication intent; recorded with the point in the same transaction (Plan 22 O7).
    /// Empty for every other job.
    pub members: Vec<crate::catalog::MemberDescriptor>,
}

/// What finishing a try did to the job.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finished {
    /// The job ended in this state.
    Ended(JobState),
    /// The job was requeued as a new attempt.
    Requeued {
        /// The new attempt.
        attempt_id: AttemptId,
        /// When it may be claimed.
        available_at: DateTime<Utc>,
    },
}

/// One job handled by [`Jobs::requeue_expired`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requeue {
    /// The job.
    pub job_id: JobId,
    /// The attempt whose lease expired; now stale or superseded.
    pub stale_attempt: AttemptId,
    /// What happened to the job.
    pub outcome: Finished,
}

pub(crate) async fn lock_job(
    tx: &Tx<'_>,
    target: &Target,
    job: JobId,
) -> Result<RuntimeOperationalJobsRow, OperationsError> {
    statements::lock_job()
        .bind(tx, &job)
        .opt()
        .await
        .classify(target)?
        .ok_or_else(|| OperationsError::NotFound {
            entity: "job",
            id: job.to_string(),
        })
}

pub(crate) async fn set_job_state(
    tx: &Tx<'_>,
    target: &Target,
    job: JobId,
    state: JobState,
    error: Option<&str>,
) -> Result<(), OperationsError> {
    statements::set_job_state()
        .params(
            tx,
            &statements::SetJobStateParams {
                state,
                last_error: error,
                job_id: job,
            },
        )
        .await
        .classify(target)?;
    Ok(())
}

/// Insert a job for its already inserted attempt, in `state` (queued, or waiting until
/// released). Returns `false`, storing nothing, when the idempotency key names another job.
pub(crate) async fn insert_job(
    tx: &Tx<'_>,
    target: &Target,
    job_id: JobId,
    job: &NewJob,
    state: JobState,
) -> Result<bool, OperationsError> {
    job.retry.validate()?;
    if job.payload_version <= 0 {
        return Err(OperationsError::InvalidRequest {
            reason: format!("payload version {} is not positive", job.payload_version),
        });
    }
    let created = statements::insert_job()
        .params(
            tx,
            &statements::InsertJobParams {
                job_id,
                attempt_id: job.attempt.attempt_id,
                idempotency_key: job.idempotency_key.as_str(),
                payload_version: job.payload_version,
                payload: &job.payload,
                priority: job.priority,
                state,
                max_tries: i32::try_from(job.retry.max_tries).unwrap_or(i32::MAX),
                backoff_base_us: micros(job.retry.backoff),
                backoff_cap_us: micros(job.retry.backoff_cap),
            },
        )
        .opt()
        .await
        .classify(target)?;
    Ok(created.is_some())
}

/// Release a waiting job: it becomes claimable, and its planned attempt is queued. Workers
/// are woken when the transaction commits.
pub(crate) async fn release(
    tx: &Tx<'_>,
    target: &Target,
    job: JobId,
    note: &TransitionNote,
) -> Result<(), OperationsError> {
    let locked = lock_job(tx, target, job).await?;
    let released = statements::release_job()
        .params(
            tx,
            &statements::ReleaseJobParams {
                queued: JobState::Queued,
                job_id: job,
                waiting: JobState::Waiting,
            },
        )
        .opt()
        .await
        .classify(target)?;
    if released.is_none() {
        return Err(OperationsError::InvalidRequest {
            reason: format!("job {job} is {}, not waiting", locked.state.as_str()),
        });
    }
    // Effect reconciliation may suspend a retry whose new try was already queued.
    if attempts::fetch(tx, target, locked.attempt_id).await?.state != AttemptState::Queued {
        attempts::apply(
            tx,
            target,
            locked.attempt_id,
            AttemptState::Queued,
            note,
            None,
        )
        .await?;
    }
    attempts::notify(tx, target, JOBS_CHANNEL, &job.to_string()).await
}

/// Requeue `job` as `next`, a new attempt whose parent is its current attempt, when the
/// retry policy allows another try; otherwise end the job as failed. A cancellation
/// requested on the previous attempt ends the job as cancelled instead.
pub(crate) async fn requeue(
    tx: &Tx<'_>,
    target: &Target,
    job: &RuntimeOperationalJobsRow,
    next: AttemptId,
    reason: &str,
) -> Result<Finished, OperationsError> {
    let previous = job.attempt_id;
    let prior = attempts::fetch(tx, target, previous).await?;
    if prior.cancel_requested {
        set_job_state(tx, target, job.job_id, JobState::Cancelled, Some(reason)).await?;
        return Ok(Finished::Ended(JobState::Cancelled));
    }
    let next_try = u32::try_from(job.tries)
        .unwrap_or(u32::MAX)
        .saturating_add(1);
    let Some(delay) = RetryPolicy::of(job)?.delay_before(next_try) else {
        let exhausted = format!("{reason}; retries exhausted after {} tries", job.tries);
        set_job_state(tx, target, job.job_id, JobState::Failed, Some(&exhausted)).await?;
        return Ok(Finished::Ended(JobState::Failed));
    };
    let attempt = NewAttempt {
        attempt_id: next,
        run_id: prior.run_id,
        kind: prior.kind,
        operational_job_identity: pse_ids::roles::RecordedOperationalJobIdentity::recorded(
            prior.operational_job_identity,
            prior.operational_job_frame,
        ),
        preparation_identity: prior.preparation_identity,
        parent_attempt: Some(previous),
    };
    attempts::insert(tx, target, &attempt, Some("retry")).await?;
    let queued = TransitionNote::by("retry").because(format!("try {next_try} after {previous}"));
    attempts::apply(tx, target, next, AttemptState::Queued, &queued, None).await?;
    if prior.state == AttemptState::Stale {
        let superseded = TransitionNote::by("retry").because(format!("superseded by {next}"));
        attempts::apply(
            tx,
            target,
            previous,
            AttemptState::Superseded,
            &superseded,
            None,
        )
        .await?;
    }
    let available_at = statements::requeue_job()
        .params(
            tx,
            &statements::RequeueJobParams {
                attempt_id: next,
                state: JobState::Queued,
                delay_us: micros(delay),
                last_error: reason,
                job_id: job.job_id,
            },
        )
        .one()
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
        if let Some(existing) = self.find_by_key(&job.idempotency_key).await? {
            return Ok(existing);
        }
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        attempts::insert(&tx, target, &job.attempt, Some("enqueue")).await?;
        let note = TransitionNote::by("enqueue");
        attempts::apply(
            &tx,
            target,
            job.attempt.attempt_id,
            AttemptState::Queued,
            &note,
            None,
        )
        .await?;
        // The job identity is minted here, on the runtime's UUIDv7 path; the store mints
        // no domain identity (ADR-0114 Outcome 13).
        let job_id: JobId = crate::mint_id();
        if !insert_job(&tx, target, job_id, job, JobState::Queued).await? {
            // A concurrent enqueue with the same key committed first: undo our attempt.
            tx.rollback().await.classify(target)?;
            return self
                .find_by_key(&job.idempotency_key)
                .await?
                .ok_or_else(|| OperationsError::NotFound {
                    entity: "job with idempotency key",
                    id: job.idempotency_key.clone(),
                });
        }
        // Wakes idle workers when the transaction commits; workers also poll, so a lost
        // notification only delays a claim.
        attempts::notify(&tx, target, JOBS_CHANNEL, &job_id.to_string()).await?;
        tx.commit().await.classify(target)?;
        Ok(Enqueued::Created {
            job_id,
            attempt_id: job.attempt.attempt_id,
        })
    }

    async fn find_by_key(&self, key: &str) -> Result<Option<Enqueued>, OperationsError> {
        let client = self.store.client().await?;
        let job = statements::job_by_key()
            .bind(&client, &key)
            .opt()
            .await
            .classify(self.target())?;
        Ok(job.map(|job| Enqueued::Existing {
            job_id: job.job_id,
            attempt_id: job.attempt_id,
        }))
    }

    /// Read one job.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`]; classified driver failures.
    pub async fn get(&self, job: JobId) -> Result<RuntimeOperationalJobsRow, OperationsError> {
        let client = self.store.client().await?;
        statements::job()
            .bind(&client, &job)
            .opt()
            .await
            .classify(self.target())?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "job",
                id: job.to_string(),
            })
    }

    /// Jobs newest first: all of them or those in the given states, at most
    /// `filter.limit`.
    ///
    /// # Errors
    ///
    /// [`OperationsError::InvalidRequest`] for a non-positive limit; classified driver
    /// failures.
    pub async fn list(
        &self,
        filter: &JobFilter,
    ) -> Result<Vec<RuntimeOperationalJobsRow>, OperationsError> {
        if filter.limit <= 0 {
            return Err(OperationsError::InvalidRequest {
                reason: format!("job listing limit {} is not positive", filter.limit),
            });
        }
        let client = self.store.client().await?;
        statements::list_jobs()
            .params(
                &client,
                &statements::ListJobsParams {
                    states: filter.states.as_slice(),
                    limit: filter.limit,
                },
            )
            .all()
            .await
            .classify(self.target())
    }

    /// Claim the next available job for `worker`: the highest priority, then the oldest
    /// availability. Rows locked by other claimers are skipped, so two workers never
    /// claim the same job. The job's attempt moves to `running` under a lease.
    ///
    /// # Errors
    ///
    /// [`OperationsError::CorruptValue`] for a stored payload that is not JSON; classified
    /// driver failures.
    pub async fn claim(
        &self,
        worker: &str,
        lease: Duration,
    ) -> Result<Option<ClaimedJob>, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let Some(job) = statements::claim_job()
            .bind(&tx, &JobState::Running)
            .opt()
            .await
            .classify(target)?
        else {
            tx.rollback().await.classify(target)?;
            return Ok(None);
        };
        let note = TransitionNote::by(worker)
            .because(format!("claimed job {}, try {}", job.job_id, job.tries));
        attempts::apply(
            &tx,
            target,
            job.attempt_id,
            AttemptState::Running,
            &note,
            Some(Lease {
                worker,
                duration: lease,
            }),
        )
        .await?;
        let attempt = attempts::fetch(&tx, target, job.attempt_id).await?;
        // A study point's try is now assigned (Plan 22 O7).
        crate::studies::job_changed(&tx, target, job.job_id, &[]).await?;
        tx.commit().await.classify(target)?;
        let lease_expires_at =
            attempt
                .lease_expires_at
                .ok_or_else(|| OperationsError::CorruptValue {
                    column: "attempts.lease_expires_at",
                    detail: format!("running attempt {} holds no lease", job.attempt_id),
                })?;
        Ok(Some(ClaimedJob {
            job_id: job.job_id,
            attempt_id: job.attempt_id,
            run_id: attempt.run_id,
            parent_attempt: attempt.parent_attempt,
            payload_version: job.payload_version,
            payload: serde_json::from_str(&job.payload).map_err(|error| {
                OperationsError::CorruptValue {
                    column: "jobs.payload",
                    detail: error.to_string(),
                }
            })?,
            try_number: job.tries,
            lease_expires_at: utc("attempts.lease_expires_at", lease_expires_at)?,
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
        job: JobId,
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
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let locked = lock_job(&tx, target, job).await?;
        let owner = statements::running_owner()
            .params(
                &tx,
                &statements::RunningOwnerParams {
                    attempt_id: locked.attempt_id,
                    running: AttemptState::Running,
                },
            )
            .opt()
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
            &tx,
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
                requeue(&tx, target, &locked, next, &reason).await?
            }
            (state, _) => {
                let ended = match state {
                    AttemptState::Completed | AttemptState::Partial => JobState::Completed,
                    AttemptState::Cancelled => JobState::Cancelled,
                    _ => JobState::Failed,
                };
                set_job_state(&tx, target, job, ended, outcome.note.reason.as_deref()).await?;
                Finished::Ended(ended)
            }
        };
        // A study point follows its job, with the members a completed try wrote.
        crate::studies::job_changed(&tx, target, job, &outcome.members).await?;
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
        mut mint: impl FnMut() -> AttemptId,
    ) -> Result<Vec<Requeue>, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let expired = statements::expired_jobs()
            .bind(&tx, &limit)
            .all()
            .await
            .classify(target)?;
        let mut handled = Vec::with_capacity(expired.len());
        for job_id in expired.into_iter().map(JobId::from_id) {
            let locked = lock_job(&tx, target, job_id).await?;
            let (state, _) = attempts::lock_state(&tx, target, locked.attempt_id).await?;
            if state == AttemptState::Running {
                let note = TransitionNote::by("stale-sweep").because("lease expired");
                attempts::apply(
                    &tx,
                    target,
                    locked.attempt_id,
                    AttemptState::Stale,
                    &note,
                    None,
                )
                .await?;
            }
            let outcome = requeue(&tx, target, &locked, mint(), "lease expired").await?;
            crate::studies::job_changed(&tx, target, job_id, &[]).await?;
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
