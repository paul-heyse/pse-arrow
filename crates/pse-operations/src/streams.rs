// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Live progress and incumbent streams, inserted in typed `UNNEST` batches
//! (ADR-0112 Outcome 17). Rows are keyed by (attempt, sequence) as the producer numbers
//! them, so a re-sent batch is idempotent.

use chrono::{DateTime, Utc};
use pse_ids::SemanticId;
use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};

use crate::codec;
use crate::error::{Classify, OperationsError, Target};
use crate::store::Store;

/// The notification channel for new progress; the payload is the attempt identity.
pub const PROGRESS_CHANNEL: &str = "pse_ops_progress";

/// One progress event.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgressEvent {
    /// The producer's sequence number within the attempt.
    pub seq: i64,
    /// When the producer observed it.
    pub at: DateTime<Utc>,
    /// The solver or workflow phase.
    pub phase: String,
    /// The observed values, versioned by the producer.
    pub payload: serde_json::Value,
}

impl FromRow<'_, PgRow> for ProgressEvent {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            seq: row.try_get("seq")?,
            at: row.try_get("at")?,
            phase: row.try_get("phase")?,
            payload: row.try_get("payload")?,
        })
    }
}

/// One incumbent: an improving feasible point and the bound at that time.
#[derive(Clone, Debug, PartialEq)]
pub struct Incumbent {
    /// The producer's sequence number within the attempt.
    pub seq: i64,
    /// When it was found.
    pub at: DateTime<Utc>,
    /// The incumbent objective.
    pub objective: f64,
    /// The dual bound, when the solver reports one.
    pub dual_bound: Option<f64>,
    /// The relative gap, when defined.
    pub gap: Option<f64>,
    /// The stored solution, when kept for warm starts or resumption.
    pub solution_id: Option<SemanticId>,
}

impl FromRow<'_, PgRow> for Incumbent {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            seq: row.try_get("seq")?,
            at: row.try_get("at")?,
            objective: row.try_get("objective")?,
            dual_bound: row.try_get("dual_bound")?,
            gap: row.try_get("gap")?,
            solution_id: codec::opt_id(row, "solution_id")?,
        })
    }
}

/// The stream repository.
#[derive(Clone, Copy, Debug)]
pub struct Streams<'s> {
    store: &'s Store,
}

impl<'s> Streams<'s> {
    pub(crate) const fn new(store: &'s Store) -> Self {
        Self { store }
    }

    fn target(&self) -> &Target {
        self.store.target()
    }

    /// Insert a batch of progress events in one statement and notify watchers. Events
    /// whose sequence number is already stored are skipped. Returns the rows inserted.
    ///
    /// # Errors
    ///
    /// Classified driver failures (a missing attempt is a foreign-key violation).
    pub async fn append_progress(
        &self,
        attempt: SemanticId,
        events: &[ProgressEvent],
    ) -> Result<u64, OperationsError> {
        if events.is_empty() {
            return Ok(0);
        }
        let target = self.target();
        let mut tx = self.store.pool().begin().await.classify(target)?;
        let inserted = sqlx::query(
            "INSERT INTO pse_ops.progress_events (attempt_id, seq, at, phase, payload) \
             SELECT $1, e.seq, e.at, e.phase, e.payload \
             FROM UNNEST($2::bigint[], $3::timestamptz[], $4::text[], $5::jsonb[]) \
                 AS e (seq, at, phase, payload) \
             ON CONFLICT (attempt_id, seq) DO NOTHING",
        )
        .bind(codec::uuid(attempt))
        .bind(events.iter().map(|e| e.seq).collect::<Vec<_>>())
        .bind(events.iter().map(|e| e.at).collect::<Vec<_>>())
        .bind(events.iter().map(|e| e.phase.as_str()).collect::<Vec<_>>())
        .bind(events.iter().map(|e| e.payload.clone()).collect::<Vec<_>>())
        .execute(&mut *tx)
        .await
        .classify(target)?
        .rows_affected();
        notify(&mut tx, target, PROGRESS_CHANNEL, attempt).await?;
        tx.commit().await.classify(target)?;
        Ok(inserted)
    }

    /// Progress events after `after` (exclusive), in sequence order, at most `limit`.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn progress(
        &self,
        attempt: SemanticId,
        after: Option<i64>,
        limit: i64,
    ) -> Result<Vec<ProgressEvent>, OperationsError> {
        sqlx::query_as(
            "SELECT seq, at, phase, payload FROM pse_ops.progress_events \
             WHERE attempt_id = $1 AND seq > coalesce($2, -1) ORDER BY seq LIMIT $3",
        )
        .bind(codec::uuid(attempt))
        .bind(after)
        .bind(limit)
        .fetch_all(self.store.pool())
        .await
        .classify(self.target())
    }

    /// Insert a batch of incumbents in one statement. Returns the rows inserted.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn record_incumbents(
        &self,
        attempt: SemanticId,
        incumbents: &[Incumbent],
    ) -> Result<u64, OperationsError> {
        if incumbents.is_empty() {
            return Ok(0);
        }
        sqlx::query(
            "INSERT INTO pse_ops.incumbents \
                 (attempt_id, seq, at, objective, dual_bound, gap, solution_id) \
             SELECT $1, i.seq, i.at, i.objective, i.dual_bound, i.gap, i.solution_id \
             FROM UNNEST($2::bigint[], $3::timestamptz[], $4::float8[], $5::float8[], \
                         $6::float8[], $7::uuid[]) \
                 AS i (seq, at, objective, dual_bound, gap, solution_id) \
             ON CONFLICT (attempt_id, seq) DO NOTHING",
        )
        .bind(codec::uuid(attempt))
        .bind(incumbents.iter().map(|i| i.seq).collect::<Vec<_>>())
        .bind(incumbents.iter().map(|i| i.at).collect::<Vec<_>>())
        .bind(incumbents.iter().map(|i| i.objective).collect::<Vec<_>>())
        .bind(incumbents.iter().map(|i| i.dual_bound).collect::<Vec<_>>())
        .bind(incumbents.iter().map(|i| i.gap).collect::<Vec<_>>())
        .bind(
            incumbents
                .iter()
                .map(|i| i.solution_id.map(codec::uuid))
                .collect::<Vec<_>>(),
        )
        .execute(self.store.pool())
        .await
        .classify(self.target())
        .map(|done| done.rows_affected())
    }

    /// The latest incumbent of an attempt, if any.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn latest_incumbent(
        &self,
        attempt: SemanticId,
    ) -> Result<Option<Incumbent>, OperationsError> {
        sqlx::query_as(
            "SELECT seq, at, objective, dual_bound, gap, solution_id FROM pse_ops.incumbents \
             WHERE attempt_id = $1 ORDER BY seq DESC LIMIT 1",
        )
        .bind(codec::uuid(attempt))
        .fetch_optional(self.store.pool())
        .await
        .classify(self.target())
    }
}

async fn notify(
    conn: &mut sqlx::PgConnection,
    target: &Target,
    channel: &str,
    attempt: SemanticId,
) -> Result<(), OperationsError> {
    sqlx::query("SELECT pg_notify($1, $2)")
        .bind(channel)
        .bind(codec::uuid(attempt).to_string())
        .execute(&mut *conn)
        .await
        .classify(target)?;
    Ok(())
}
