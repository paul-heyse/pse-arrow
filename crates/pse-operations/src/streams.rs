// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Live progress and incumbent streams, inserted by binary `COPY` (ADR-0114 Outcomes 17
//! and 26). Rows are keyed by (attempt, sequence) as the producer numbers them: the
//! sequence numbers already stored are filtered out in the same transaction, so a re-sent
//! batch is idempotent. Progress values are typed columns in the `runtime.solve_metrics`
//! value vocabulary, so every number round-trips exactly. Streams are bounded by a
//! retention policy over finished attempts, not by an event cap.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use pse_model::generated::enums::{EvidenceUnavailableReason, NativeMetricKind};
use pse_operations_queries::client::Params as _;
use pse_operations_queries::queries::streams as statements;
use tokio_postgres::types::ToSql;

use crate::attempts::{self, AttemptId, micros, utc};
use crate::bulk::{Cells, copy_in};
use crate::error::{Classify, OperationsError, Target};
use crate::generated::copy;
use crate::listener::{Channel, Event, Subscription};
use crate::solutions::{self, NewSolution};
use crate::store::Store;
pub use pse_model::generated::runtime::operational_incumbents::RuntimeOperationalIncumbentsRow;
use pse_model::generated::runtime::operational_progress_events::RuntimeOperationalProgressEventsRow;
use pse_model::generated::runtime::operational_progress_values::RuntimeOperationalProgressValuesRow;

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

    /// The value a stored row holds: exactly the column its kind selects.
    fn of(row: &RuntimeOperationalProgressValuesRow) -> Result<Self, OperationsError> {
        let missing = || OperationsError::CorruptValue {
            column: "progress_values.kind",
            detail: format!(
                "progress value `{}` lacks its {} field",
                row.name,
                row.kind.as_str()
            ),
        };
        Ok(match row.kind {
            NativeMetricKind::Real => Self::Real(row.real.ok_or_else(missing)?),
            NativeMetricKind::Integer => Self::Integer(row.integer.ok_or_else(missing)?),
            NativeMetricKind::Boolean => Self::Boolean(row.boolean.ok_or_else(missing)?),
            NativeMetricKind::Text => Self::Text(row.text.clone().ok_or_else(missing)?),
            NativeMetricKind::Unavailable => {
                Self::Unavailable(row.unavailable.ok_or_else(missing)?)
            }
        })
    }
}

/// One progress event of a durable attempt: a `runtime.operational_progress_events` row
/// with its values.
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

/// How long the streams of finished attempts are kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Retention {
    /// Streams of attempts that finished longer ago than this are removed.
    pub finished_for: std::time::Duration,
}

/// One progress value's cells, in the order of the table's binary copy.
struct ValueCells<'a> {
    seq: i64,
    name: &'a str,
    kind: NativeMetricKind,
    real: Option<f64>,
    integer: Option<i64>,
    boolean: Option<bool>,
    text: Option<&'a str>,
    unavailable: Option<EvidenceUnavailableReason>,
}

impl<'a> ValueCells<'a> {
    fn new(seq: i64, name: &'a str, value: &'a ProgressValue) -> Self {
        Self {
            seq,
            name,
            kind: value.kind(),
            real: match value {
                ProgressValue::Real(v) => Some(*v),
                _ => None,
            },
            integer: match value {
                ProgressValue::Integer(v) => Some(*v),
                _ => None,
            },
            boolean: match value {
                ProgressValue::Boolean(v) => Some(*v),
                _ => None,
            },
            text: match value {
                ProgressValue::Text(v) => Some(v.as_str()),
                _ => None,
            },
            unavailable: match value {
                ProgressValue::Unavailable(reason) => Some(*reason),
                _ => None,
            },
        }
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

    /// Insert a batch of progress events and their values in one transaction and notify
    /// watchers. Events whose sequence number is already stored are skipped with their
    /// values. Returns the events inserted.
    ///
    /// # Errors
    ///
    /// Classified driver failures (a missing attempt is an invariant violation of the
    /// `attempt_id` reference).
    pub async fn append_progress(
        &self,
        attempt: AttemptId,
        events: &[ProgressEvent],
    ) -> Result<u64, OperationsError> {
        if events.is_empty() {
            return Ok(0);
        }
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let seqs: Vec<i64> = events.iter().map(|event| event.seq).collect();
        let stored: BTreeSet<i64> = statements::existing_progress_seqs()
            .params(
                &tx,
                &statements::ExistingProgressSeqsParams {
                    attempt_id: attempt,
                    seqs: seqs.as_slice(),
                },
            )
            .all()
            .await
            .classify(target)?
            .into_iter()
            .collect();
        let fresh: Vec<&ProgressEvent> = events
            .iter()
            .filter(|event| !stored.contains(&event.seq))
            .collect();
        let id: &(dyn ToSql + Sync) = &attempt;
        let mut rows: Vec<Cells<'_>> = Vec::with_capacity(fresh.len());
        for event in &fresh {
            rows.push(vec![
                id,
                &event.seq,
                &event.step,
                &event.at,
                &event.elapsed_seconds,
                &event.phase,
            ]);
        }
        let inserted = copy_in(&tx, target, &copy::PROGRESS_EVENTS, &rows).await?;
        let mut cells: Vec<ValueCells<'_>> = Vec::new();
        for event in &fresh {
            for (name, value) in &event.values {
                cells.push(ValueCells::new(event.seq, name, value));
            }
        }
        let mut rows: Vec<Cells<'_>> = Vec::with_capacity(cells.len());
        for cell in &cells {
            rows.push(vec![
                id,
                &cell.seq,
                &cell.name,
                &cell.kind,
                &cell.real,
                &cell.integer,
                &cell.boolean,
                &cell.text,
                &cell.unavailable,
            ]);
        }
        copy_in(&tx, target, &copy::PROGRESS_VALUES, &rows).await?;
        attempts::notify(&tx, target, PROGRESS_CHANNEL, &attempt.to_string()).await?;
        tx.commit().await.classify(target)?;
        Ok(inserted)
    }

    /// Progress events after `after` (exclusive), in sequence order, at most `limit`, with
    /// their values.
    ///
    /// # Errors
    ///
    /// Classified driver failures, and a value that violates its kind.
    pub async fn progress(
        &self,
        attempt: AttemptId,
        after: Option<i64>,
        limit: i64,
    ) -> Result<Vec<ProgressEvent>, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let rows: Vec<RuntimeOperationalProgressEventsRow> = statements::progress_page()
            .params(
                &tx,
                &statements::ProgressPageParams {
                    attempt_id: attempt,
                    after,
                    limit,
                },
            )
            .all()
            .await
            .classify(target)?;
        let mut events = rows
            .into_iter()
            .map(|row| {
                Ok(ProgressEvent {
                    seq: row.seq,
                    step: row.step,
                    at: utc("progress_events.at", row.at)?,
                    elapsed_seconds: row.elapsed_seconds,
                    phase: row.phase,
                    values: BTreeMap::new(),
                })
            })
            .collect::<Result<Vec<_>, OperationsError>>()?;
        if let (Some(first), Some(last)) = (events.first(), events.last()) {
            let values = statements::progress_values()
                .params(
                    &tx,
                    &statements::ProgressValuesParams {
                        attempt_id: attempt,
                        first: first.seq,
                        last: last.seq,
                    },
                )
                .all()
                .await
                .classify(target)?;
            let index: BTreeMap<i64, usize> =
                events.iter().enumerate().map(|(i, e)| (e.seq, i)).collect();
            for row in &values {
                if let Some(&i) = index.get(&row.seq) {
                    events[i]
                        .values
                        .insert(row.name.clone(), ProgressValue::of(row)?);
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
    pub async fn snapshot(&self, attempt: AttemptId) -> Result<Vec<ProgressEvent>, OperationsError> {
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
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let age = micros(policy.finished_for);
        statements::delete_finished_values()
            .bind(&tx, &age)
            .await
            .classify(target)?;
        let removed = statements::delete_finished_events()
            .bind(&tx, &age)
            .await
            .classify(target)?;
        statements::delete_finished_incumbents()
            .bind(&tx, &age)
            .await
            .classify(target)?;
        tx.commit().await.classify(target)?;
        Ok(removed)
    }

    /// Insert a batch of incumbents of one attempt together with the solutions they
    /// capture, in one transaction: each of `solutions` is referenced by exactly one
    /// incumbent of the batch and is stored with it (generated statements), then the
    /// incumbents are copied. Incumbents whose sequence number is already stored are
    /// skipped with their solutions, so a re-sent batch is idempotent. An incumbent may
    /// also reference a solution stored earlier. Returns the incumbents inserted.
    ///
    /// # Errors
    ///
    /// [`OperationsError::InvalidRequest`] for a batch naming several attempts or a
    /// solution no incumbent (or more than one) references; classified driver failures.
    pub async fn record_incumbents(
        &self,
        incumbents: &[RuntimeOperationalIncumbentsRow],
        solutions: &[NewSolution],
    ) -> Result<u64, OperationsError> {
        let Some(attempt) = incumbents.first().map(|incumbent| incumbent.attempt_id) else {
            return if solutions.is_empty() {
                Ok(0)
            } else {
                Err(OperationsError::InvalidRequest {
                    reason: "a captured solution without its incumbent".to_owned(),
                })
            };
        };
        if incumbents.iter().any(|incumbent| incumbent.attempt_id != attempt) {
            return Err(OperationsError::InvalidRequest {
                reason: "an incumbent batch belongs to one attempt".to_owned(),
            });
        }
        for solution in solutions {
            let references = incumbents
                .iter()
                .filter(|incumbent| incumbent.solution_id == Some(solution.solution_id))
                .count();
            if references != 1 {
                return Err(OperationsError::InvalidRequest {
                    reason: format!(
                        "captured solution {} is referenced by {references} incumbents of the batch",
                        solution.solution_id
                    ),
                });
            }
        }
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let seqs: Vec<i64> = incumbents.iter().map(|incumbent| incumbent.seq).collect();
        let stored: BTreeSet<i64> = statements::existing_incumbent_seqs()
            .params(
                &tx,
                &statements::ExistingIncumbentSeqsParams {
                    attempt_id: attempt,
                    seqs: seqs.as_slice(),
                },
            )
            .all()
            .await
            .classify(target)?
            .into_iter()
            .collect();
        let fresh = incumbents
            .iter()
            .filter(|incumbent| !stored.contains(&incumbent.seq))
            .map(|incumbent| Ok((incumbent, utc("incumbents.at", incumbent.at)?)))
            .collect::<Result<Vec<_>, OperationsError>>()?;
        // The solutions first: the incumbents reference them.
        for (incumbent, _) in &fresh {
            if let Some(solution) = solutions
                .iter()
                .find(|solution| incumbent.solution_id == Some(solution.solution_id))
            {
                solutions::insert(&tx, target, solution).await?;
            }
        }
        let mut rows: Vec<Cells<'_>> = Vec::with_capacity(fresh.len());
        for (incumbent, at) in &fresh {
            let id: &(dyn ToSql + Sync) = &incumbent.attempt_id;
            rows.push(vec![
                id,
                &incumbent.seq,
                at,
                &incumbent.objective,
                &incumbent.dual_bound,
                &incumbent.gap,
                &incumbent.solution_id,
            ]);
        }
        let inserted = copy_in(&tx, target, &copy::INCUMBENTS, &rows).await?;
        tx.commit().await.classify(target)?;
        Ok(inserted)
    }

    /// The latest incumbent of an attempt, if any.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn latest_incumbent(
        &self,
        attempt: AttemptId,
    ) -> Result<Option<RuntimeOperationalIncumbentsRow>, OperationsError> {
        let client = self.store.client().await?;
        statements::latest_incumbent()
            .bind(&client, &attempt)
            .opt()
            .await
            .classify(self.target())
    }
}

impl Store {
    /// Watch one attempt's progress stream. The watcher subscribes to the store's listener
    /// and returns once `LISTEN` is in effect; notifications only wake it, and it always
    /// reads the stored events.
    ///
    /// # Errors
    ///
    /// [`OperationsError::Unavailable`] when `LISTEN` does not take effect.
    pub async fn watch_progress(
        &self,
        attempt: AttemptId,
    ) -> Result<ProgressWatcher, OperationsError> {
        Ok(ProgressWatcher {
            events: self.subscribe().await?,
            store: self.clone(),
            attempt,
        })
    }
}

/// Follows one attempt's progress stream as it is written (ADR-0114 Outcome 17).
#[derive(Debug)]
pub struct ProgressWatcher {
    events: Subscription,
    store: Store,
    attempt: AttemptId,
}

impl ProgressWatcher {
    /// The next stored events after `after`, at most `limit`, waiting until some exist.
    /// Returns an empty page once the attempt has stopped working and every stored event
    /// was returned. A notification lost while the listener reconnects only delays the
    /// read: the watcher reads again on every resynchronization.
    ///
    /// # Errors
    ///
    /// Classified driver failures; [`OperationsError::NotFound`] for an unknown attempt;
    /// [`OperationsError::Unavailable`] once the store's listener stopped.
    pub async fn next(
        &mut self,
        after: Option<i64>,
        limit: i64,
    ) -> Result<Vec<ProgressEvent>, OperationsError> {
        use crate::lifecycle::Lifecycle;
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
                match self.events.next().await? {
                    Event::Notification {
                        channel: Channel::Progress,
                        attempt,
                    } if attempt == self.attempt => break,
                    Event::Resync => break,
                    Event::Notification { .. } => {}
                }
            }
        }
    }
}
