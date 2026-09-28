// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Live progress and incumbent streams, inserted in typed `UNNEST` batches
//! (ADR-0114 Outcome 17). Rows are keyed by (attempt, sequence) as the producer numbers
//! them, so a re-sent batch is idempotent. Progress values are typed columns in the
//! `runtime.solve_metrics` value vocabulary, so every number round-trips exactly. Streams
//! are bounded by a retention policy over finished attempts, not by an event cap.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use pse_ids::SemanticId;
use pse_model::generated::enums::{EvidenceUnavailableReason, NativeMetricKind};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};

use crate::codec;
use crate::error::{Classify, OperationsError, Target};
use crate::store::Store;

/// The notification channel for new progress; the payload is the attempt identity.
pub const PROGRESS_CHANNEL: &str = "pse_ops_progress";

/// One typed progress value. A nonfinite real is an unavailable `nonfinite` observation,
/// never a stored number.
#[derive(Clone, Debug, PartialEq)]
pub enum ProgressValue {
    /// A finite real measurement.
    Real(f64),
    /// A count or status identifier.
    Integer(i64),
    /// An explicit Boolean.
    Boolean(bool),
    /// A diagnostic or enumerated text.
    Text(String),
    /// An absent observation and why.
    Unavailable(EvidenceUnavailableReason),
}

impl ProgressValue {
    /// A real value; nonfinite values become [`EvidenceUnavailableReason::Nonfinite`].
    pub const fn real(value: f64) -> Self {
        if value.is_finite() {
            Self::Real(value)
        } else {
            Self::Unavailable(EvidenceUnavailableReason::Nonfinite)
        }
    }

    /// The value kind.
    pub const fn kind(&self) -> NativeMetricKind {
        match self {
            Self::Real(_) => NativeMetricKind::Real,
            Self::Integer(_) => NativeMetricKind::Integer,
            Self::Boolean(_) => NativeMetricKind::Boolean,
            Self::Text(_) => NativeMetricKind::Text,
            Self::Unavailable(_) => NativeMetricKind::Unavailable,
        }
    }
}

/// One progress event of a durable attempt.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgressEvent {
    /// The producer's sequence number within the attempt.
    pub seq: i64,
    /// The step of the run that produced it.
    pub step: i32,
    /// When the producer observed it.
    pub at: DateTime<Utc>,
    /// Seconds since the step's admitted execution began.
    pub elapsed_seconds: f64,
    /// The solver or workflow phase.
    pub phase: String,
    /// The observed values by name.
    pub values: BTreeMap<String, ProgressValue>,
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

/// How long the streams of finished attempts are kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Retention {
    /// Streams of attempts that finished longer ago than this are removed.
    pub finished_for: std::time::Duration,
}

/// Flattened columns of a value batch, one entry per (event, value).
#[derive(Default)]
struct ValueColumns<'a> {
    seq: Vec<i64>,
    name: Vec<&'a str>,
    kind: Vec<&'static str>,
    real: Vec<Option<f64>>,
    integer: Vec<Option<i64>>,
    boolean: Vec<Option<bool>>,
    text: Vec<Option<&'a str>>,
    unavailable: Vec<Option<&'static str>>,
}

impl<'a> ValueColumns<'a> {
    fn push(&mut self, seq: i64, name: &'a str, value: &'a ProgressValue) {
        self.seq.push(seq);
        self.name.push(name);
        self.kind.push(value.kind().as_str());
        self.real.push(match value {
            ProgressValue::Real(v) => Some(*v),
            _ => None,
        });
        self.integer.push(match value {
            ProgressValue::Integer(v) => Some(*v),
            _ => None,
        });
        self.boolean.push(match value {
            ProgressValue::Boolean(v) => Some(*v),
            _ => None,
        });
        self.text.push(match value {
            ProgressValue::Text(v) => Some(v.as_str()),
            _ => None,
        });
        self.unavailable.push(match value {
            ProgressValue::Unavailable(reason) => Some(reason.as_str()),
            _ => None,
        });
    }
}

fn value_of(row: &PgRow) -> Result<(i64, String, ProgressValue), sqlx::Error> {
    let seq: i64 = row.try_get("seq")?;
    let name: String = row.try_get("name")?;
    let kind: NativeMetricKind = codec::parsed(row, "kind")?;
    let missing = || sqlx::Error::ColumnDecode {
        index: "kind".to_owned(),
        source: format!("progress value `{name}` lacks its {} field", kind.as_str()).into(),
    };
    let value = match kind {
        NativeMetricKind::Real => {
            ProgressValue::Real(row.try_get::<Option<f64>, _>("real")?.ok_or_else(missing)?)
        }
        NativeMetricKind::Integer => ProgressValue::Integer(
            row.try_get::<Option<i64>, _>("integer")?
                .ok_or_else(missing)?,
        ),
        NativeMetricKind::Boolean => ProgressValue::Boolean(
            row.try_get::<Option<bool>, _>("boolean")?
                .ok_or_else(missing)?,
        ),
        NativeMetricKind::Text => ProgressValue::Text(
            row.try_get::<Option<String>, _>("text")?
                .ok_or_else(missing)?,
        ),
        NativeMetricKind::Unavailable => {
            ProgressValue::Unavailable(codec::opt_parsed(row, "unavailable")?.ok_or_else(missing)?)
        }
    };
    Ok((seq, name, value))
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

    /// Insert a batch of progress events and their values in one transaction and notify
    /// watchers. Events whose sequence number is already stored are skipped with their
    /// values. Returns the events inserted.
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
        let inserted: Vec<i64> = sqlx::query_scalar(
            "INSERT INTO pse_ops.progress_events (attempt_id, seq, step, at, elapsed_seconds, phase) \
             SELECT $1, e.seq, e.step, e.at, e.elapsed, e.phase \
             FROM UNNEST($2::bigint[], $3::integer[], $4::timestamptz[], \
                         $5::float8[], $6::text[]) AS e (seq, step, at, elapsed, phase) \
             ON CONFLICT (attempt_id, seq) DO NOTHING RETURNING seq",
        )
        .bind(codec::uuid(attempt))
        .bind(events.iter().map(|e| e.seq).collect::<Vec<_>>())
        .bind(events.iter().map(|e| e.step).collect::<Vec<_>>())
        .bind(events.iter().map(|e| e.at).collect::<Vec<_>>())
        .bind(events.iter().map(|e| e.elapsed_seconds).collect::<Vec<_>>())
        .bind(events.iter().map(|e| e.phase.as_str()).collect::<Vec<_>>())
        .fetch_all(&mut *tx)
        .await
        .classify(target)?;
        let fresh: std::collections::BTreeSet<i64> = inserted.iter().copied().collect();
        let mut columns = ValueColumns::default();
        for event in events.iter().filter(|e| fresh.contains(&e.seq)) {
            for (name, value) in &event.values {
                columns.push(event.seq, name, value);
            }
        }
        if !columns.seq.is_empty() {
            sqlx::query(
                "INSERT INTO pse_ops.progress_values \
                     (attempt_id, seq, name, kind, \"real\", \"integer\", \"boolean\", \"text\", unavailable) \
                 SELECT $1, v.seq, v.name, v.kind::pse_ops.native_metric_kind, v.r, v.i, v.b, \
                     v.t, v.u::pse_ops.evidence_unavailable_reason \
                 FROM UNNEST($2::bigint[], $3::text[], $4::text[], $5::float8[], $6::bigint[], \
                             $7::boolean[], $8::text[], $9::text[]) \
                     AS v (seq, name, kind, r, i, b, t, u)",
            )
            .bind(codec::uuid(attempt))
            .bind(&columns.seq)
            .bind(&columns.name)
            .bind(&columns.kind)
            .bind(&columns.real)
            .bind(&columns.integer)
            .bind(&columns.boolean)
            .bind(&columns.text)
            .bind(&columns.unavailable)
            .execute(&mut *tx)
            .await
            .classify(target)?;
        }
        notify(&mut tx, target, PROGRESS_CHANNEL, attempt).await?;
        tx.commit().await.classify(target)?;
        Ok(u64::try_from(inserted.len()).unwrap_or(u64::MAX))
    }

    /// Progress events after `after` (exclusive), in sequence order, at most `limit`, with
    /// their values.
    ///
    /// # Errors
    ///
    /// Classified driver failures, and a decode failure for a value that violates its kind.
    pub async fn progress(
        &self,
        attempt: SemanticId,
        after: Option<i64>,
        limit: i64,
    ) -> Result<Vec<ProgressEvent>, OperationsError> {
        let target = self.target();
        let mut tx = self.store.pool().begin().await.classify(target)?;
        let rows = sqlx::query(
            "SELECT seq, step, at, elapsed_seconds, phase FROM pse_ops.progress_events \
             WHERE attempt_id = $1 AND seq > coalesce($2, -1) ORDER BY seq LIMIT $3",
        )
        .bind(codec::uuid(attempt))
        .bind(after)
        .bind(limit)
        .fetch_all(&mut *tx)
        .await
        .classify(target)?;
        let mut events = rows
            .iter()
            .map(|row| {
                Ok(ProgressEvent {
                    seq: row.try_get("seq")?,
                    step: row.try_get("step")?,
                    at: row.try_get("at")?,
                    elapsed_seconds: row.try_get("elapsed_seconds")?,
                    phase: row.try_get("phase")?,
                    values: BTreeMap::new(),
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()
            .classify(target)?;
        if let (Some(first), Some(last)) = (events.first(), events.last()) {
            let values = sqlx::query(
                "SELECT seq, name, kind::text AS kind, \"real\", \"integer\", \"boolean\", \"text\", \
                     unavailable::text AS unavailable \
                 FROM pse_ops.progress_values \
                 WHERE attempt_id = $1 AND seq BETWEEN $2 AND $3 ORDER BY seq, name",
            )
            .bind(codec::uuid(attempt))
            .bind(first.seq)
            .bind(last.seq)
            .fetch_all(&mut *tx)
            .await
            .classify(target)?;
            let index: BTreeMap<i64, usize> =
                events.iter().enumerate().map(|(i, e)| (e.seq, i)).collect();
            for row in &values {
                let (seq, name, value) = value_of(row).classify(target)?;
                if let Some(&i) = index.get(&seq) {
                    events[i].values.insert(name, value);
                }
            }
        }
        tx.commit().await.classify(target)?;
        Ok(events)
    }

    /// Every stored progress event of an attempt, in sequence order: the snapshot a
    /// publication derives its rows from.
    ///
    /// # Errors
    ///
    /// As for [`Streams::progress`].
    pub async fn snapshot(
        &self,
        attempt: SemanticId,
    ) -> Result<Vec<ProgressEvent>, OperationsError> {
        const PAGE: i64 = 4096;
        let mut all = Vec::new();
        let mut after = None;
        loop {
            let page = self.progress(attempt, after, PAGE).await?;
            let full = i64::try_from(page.len()).unwrap_or(i64::MAX) == PAGE;
            after = page.last().map(|e| e.seq).or(after);
            all.extend(page);
            if !full {
                return Ok(all);
            }
        }
    }

    /// Remove the streams (progress events, their values and incumbents without a kept
    /// solution) of attempts that finished longer ago than the policy allows. Running,
    /// queued and planned attempts are never touched. Returns the events removed.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn apply_retention(&self, policy: Retention) -> Result<u64, OperationsError> {
        let target = self.target();
        let mut tx = self.store.pool().begin().await.classify(target)?;
        let age = codec::interval(policy.finished_for);
        // Deletes are explicit (no cascade): an event's values go before the event.
        sqlx::query(
            "DELETE FROM pse_ops.progress_values AS v USING pse_ops.attempts AS a \
             WHERE v.attempt_id = a.attempt_id AND a.finished_at IS NOT NULL \
               AND a.finished_at < now() - $1",
        )
        .bind(age)
        .execute(&mut *tx)
        .await
        .classify(target)?;
        let removed = sqlx::query(
            "DELETE FROM pse_ops.progress_events AS e USING pse_ops.attempts AS a \
             WHERE e.attempt_id = a.attempt_id AND a.finished_at IS NOT NULL \
               AND a.finished_at < now() - $1",
        )
        .bind(age)
        .execute(&mut *tx)
        .await
        .classify(target)?
        .rows_affected();
        sqlx::query(
            "DELETE FROM pse_ops.incumbents AS i USING pse_ops.attempts AS a \
             WHERE i.attempt_id = a.attempt_id AND i.solution_id IS NULL \
               AND a.finished_at IS NOT NULL AND a.finished_at < now() - $1",
        )
        .bind(age)
        .execute(&mut *tx)
        .await
        .classify(target)?;
        tx.commit().await.classify(target)?;
        Ok(removed)
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

impl Store {
    /// Watch one attempt's progress stream. The watcher holds one pooled connection for
    /// `LISTEN`; notifications only wake it, and it always reads the stored events.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn watch_progress(
        &self,
        attempt: SemanticId,
    ) -> Result<ProgressWatcher, OperationsError> {
        let target = self.target().clone();
        let mut listener = sqlx::postgres::PgListener::connect_with(self.pool())
            .await
            .classify(&target)?;
        // Autocommit: LISTEN is in effect once this returns, before the first read.
        listener.listen(PROGRESS_CHANNEL).await.classify(&target)?;
        Ok(ProgressWatcher {
            listener,
            store: self.clone(),
            attempt,
            payload: codec::uuid(attempt).to_string(),
        })
    }
}

/// Follows one attempt's progress stream as it is written (ADR-0114 Outcome 17).
#[derive(Debug)]
pub struct ProgressWatcher {
    listener: sqlx::postgres::PgListener,
    store: Store,
    attempt: SemanticId,
    payload: String,
}

impl ProgressWatcher {
    /// The next stored events after `after`, at most `limit`, waiting until some exist.
    /// Returns an empty page once the attempt has stopped working and every stored event
    /// was returned. A notification lost while the listener reconnects only delays the
    /// read: the watcher reads again after every reconnect.
    ///
    /// # Errors
    ///
    /// Classified driver failures; [`OperationsError::NotFound`] for an unknown attempt.
    pub async fn next(
        &mut self,
        after: Option<i64>,
        limit: i64,
    ) -> Result<Vec<ProgressEvent>, OperationsError> {
        use crate::lifecycle::Lifecycle;
        let target = self.store.target().clone();
        loop {
            let page = self
                .store
                .streams()
                .progress(self.attempt, after, limit)
                .await?;
            if !page.is_empty() {
                return Ok(page);
            }
            let state = self.store.attempts().get(self.attempt).await?.state;
            if state.ends_work() || state.is_final() {
                // Events committed before the terminal transition are already visible.
                return self
                    .store
                    .streams()
                    .progress(self.attempt, after, limit)
                    .await;
            }
            loop {
                match self.listener.try_recv().await.classify(&target)? {
                    Some(notification) if notification.payload() == self.payload => break,
                    Some(_) => {}
                    None => break,
                }
            }
        }
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
