// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Cross-process cancellation (ADR-0114 Outcome 15, finding T03).
//!
//! `pse_ops.attempts.cancel_requested` is the authority; heartbeats return it. `NOTIFY` only
//! shortens latency: notifications are lost while a listener is disconnected, so the
//! watcher re-reads the column after `LISTEN` commits, after every notification for its
//! attempt, and after every reported connection loss.

use pse_operations_queries::queries::{cancellation as statements, jobs as job_statements};
use sqlx::postgres::PgListener;

use crate::attempts::{self, AttemptId, TransitionNote};
use crate::error::{Classify, OperationsError, Target};
use crate::jobs::JobState;
use crate::lifecycle::{AttemptState, Lifecycle};
use crate::store::Store;

/// The notification channel; the payload is the attempt identity.
pub const CANCEL_CHANNEL: &str = "pse_ops_cancel";

/// What a cancellation request did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancelOutcome {
    /// The attempt had not started; it is now cancelled, and so is its job.
    CancelledBeforeStart,
    /// The request is recorded; the owning worker stops cooperatively.
    Requested,
    /// The attempt had already finished in this state; nothing changed.
    AlreadyFinished(AttemptState),
}

impl Store {
    /// Request cancellation of an attempt: set the durable flag and notify watchers in one
    /// transaction. Work that has not started is cancelled at once.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`]; classified driver failures.
    pub async fn request_cancel(
        &self,
        attempt: AttemptId,
        actor: &str,
    ) -> Result<CancelOutcome, OperationsError> {
        let target = self.target();
        let mut client = self.client().await?;
        let tx = client.transaction().await.classify(target)?;
        // Job before attempt: the lock order of the job queue.
        let job = job_statements::lock_job_of_attempt()
            .bind(&tx, &attempt)
            .opt()
            .await
            .classify(target)?;
        let (state, _) = attempts::lock_state(&tx, target, attempt).await?;
        if state.is_final() {
            tx.rollback().await.classify(target)?;
            return Ok(CancelOutcome::AlreadyFinished(state));
        }
        statements::request_cancel()
            .bind(&tx, &attempt)
            .await
            .classify(target)?;
        let outcome = if matches!(state, AttemptState::Planned | AttemptState::Queued) {
            let note = TransitionNote::by(actor).because("cancelled before start");
            attempts::apply(&tx, target, attempt, AttemptState::Cancelled, &note, None).await?;
            if let Some(job) = job {
                crate::jobs::set_job_state(&tx, target, job.job_id, JobState::Cancelled, None)
                    .await?;
            }
            CancelOutcome::CancelledBeforeStart
        } else {
            CancelOutcome::Requested
        };
        // Delivered to listeners when this transaction commits.
        attempts::notify(&tx, target, CANCEL_CHANNEL, &attempt.to_string()).await?;
        tx.commit().await.classify(target)?;
        Ok(outcome)
    }

    /// Start watching one attempt for cancellation. The watcher holds one pooled
    /// connection for `LISTEN`.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn watch_cancellation(
        &self,
        attempt: AttemptId,
    ) -> Result<CancellationWatcher, OperationsError> {
        let target = self.target().clone();
        let mut listener = PgListener::connect_with(self.pool())
            .await
            .classify(&target)?;
        // Autocommit: LISTEN is in effect once this returns, before the first re-read.
        listener.listen(CANCEL_CHANNEL).await.classify(&target)?;
        Ok(CancellationWatcher {
            listener,
            store: self.clone(),
            attempt,
            payload: attempt.to_string(),
            target,
        })
    }
}

/// Waits for cancellation of one attempt, reading the durable flag as the authority.
#[derive(Debug)]
pub struct CancellationWatcher {
    listener: PgListener,
    store: Store,
    attempt: AttemptId,
    payload: String,
    target: Target,
}

impl CancellationWatcher {
    /// The attempt being watched.
    pub const fn attempt(&self) -> AttemptId {
        self.attempt
    }

    /// Read the durable cancellation flag.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`]; classified driver failures.
    pub async fn is_requested(&self) -> Result<bool, OperationsError> {
        let client = self.store.client().await?;
        statements::cancel_requested()
            .bind(&client, &self.attempt)
            .opt()
            .await
            .classify(&self.target)?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "attempt",
                id: self.attempt.to_string(),
            })
    }

    /// Resolve once cancellation has been requested. Re-reads the flag first (the
    /// listen-then-inspect rule), after each notification naming this attempt, and after
    /// each connection loss, when notifications sent meanwhile are gone.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn requested(&mut self) -> Result<(), OperationsError> {
        loop {
            if self.is_requested().await? {
                return Ok(());
            }
            loop {
                match self.listener.try_recv().await.classify(&self.target)? {
                    Some(notification) if notification.payload() == self.payload => break,
                    Some(_) => {}
                    // The connection was lost and re-established: re-read the authority.
                    None => break,
                }
            }
        }
    }
}
