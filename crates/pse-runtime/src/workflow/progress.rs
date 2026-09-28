// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! A durable attempt's streams as they are stored (Plan 22 O9): its progress events and
//! the incumbents of its branch-and-bound searches, read in bounded pages and, when
//! followed, over the store's listener until the attempt ends.
use super::{Durability, Runtime, WorkflowError};
use pse_backend_native::solve::Metric;
use pse_operations::{
    Store,
    attempts::AttemptId,
    streams::{
        ProgressEvent, ProgressValue, ProgressWatcher, RuntimeOperationalIncumbentsRow,
        StreamPage, StreamPosition,
    },
};
use std::collections::BTreeMap;

/// One stored record of a durable attempt's streams.
#[derive(Clone, Debug, PartialEq)]
pub enum StreamRecord {
    /// A progress event with its values.
    Progress(ProgressEvent),
    /// An incumbent of a branch-and-bound search, with the progress context of the event
    /// that reported it.
    Incumbent(RuntimeOperationalIncumbentsRow),
}

impl StreamRecord {
    /// When the producer observed it, in microseconds since the Unix epoch.
    pub fn at(&self) -> i64 {
        match self {
            Self::Progress(event) => event.at.timestamp_micros(),
            Self::Incumbent(incumbent) => incumbent.at,
        }
    }

    /// Its sequence number in its own stream (progress events and incumbents are
    /// numbered separately).
    pub const fn sequence(&self) -> i64 {
        match self {
            Self::Progress(event) => event.seq,
            Self::Incumbent(incumbent) => incumbent.seq,
        }
    }

    /// The step of the run that produced it.
    pub const fn step(&self) -> i32 {
        match self {
            Self::Progress(event) => event.step,
            Self::Incumbent(incumbent) => incumbent.step,
        }
    }

    /// The solver or workflow phase that reported it.
    pub fn phase(&self) -> &str {
        match self {
            Self::Progress(event) => &event.phase,
            Self::Incumbent(incumbent) => &incumbent.phase,
        }
    }

    /// Seconds since the step's admitted execution began.
    pub const fn elapsed_seconds(&self) -> f64 {
        match self {
            Self::Progress(event) => event.elapsed_seconds,
            Self::Incumbent(incumbent) => incumbent.elapsed_seconds,
        }
    }

    /// A progress event's values in the native metric vocabulary; an incumbent's typed
    /// values are its row's columns.
    pub fn values(&self) -> BTreeMap<String, Metric> {
        match self {
            Self::Progress(event) => event
                .values
                .iter()
                .map(|(name, value)| (name.clone(), metric(value)))
                .collect(),
            Self::Incumbent(_) => BTreeMap::new(),
        }
    }
}

/// A stored value as the native metric it was recorded from.
fn metric(value: &ProgressValue) -> Metric {
    match value {
        ProgressValue::Real(v) => Metric::Real(*v),
        ProgressValue::Integer(v) => Metric::Integer(*v),
        ProgressValue::Boolean(v) => Metric::Bool(*v),
        ProgressValue::Text(v) => Metric::Text(v.clone()),
        ProgressValue::Unavailable(reason) => Metric::Unavailable(*reason),
    }
}

/// How long a followed stream waits for its attempt to be registered.
const REGISTRATION: std::time::Duration = std::time::Duration::from_secs(5);

/// How a [`ProgressStream`] reads.
#[derive(Debug)]
enum Reader {
    /// What is stored now, then the end.
    Stored(Store),
    /// Everything, waiting for more until the attempt ends.
    Following(Box<ProgressWatcher>),
}

/// A durable attempt's progress events and incumbents in bounded pages. A followed stream
/// waits for new rows until the attempt stops working; otherwise it ends after the rows
/// stored when they were read. Closing it (or cancelling `cancel`) ends it at once.
#[derive(Debug)]
pub struct ProgressStream {
    reader: Option<Reader>,
    attempt: AttemptId,
    position: StreamPosition,
    page: i64,
    cancel: pse_columnar::CancellationToken,
}

impl Runtime {
    /// The stored streams of a durable attempt of this runtime's store, `page` rows of each
    /// stream at a time; with `follow`, the stream waits for new rows until the attempt
    /// ends (listen, then read: nothing stored after the stream starts is missed), and for
    /// the attempt itself to be registered for up to five seconds.
    ///
    /// # Errors
    /// An ephemeral runtime; a page of no rows; an unknown attempt; the store's listener
    /// not in effect (when following); store failures.
    pub async fn progress(
        &self,
        attempt: AttemptId,
        follow: bool,
        page: usize,
        cancel: pse_columnar::CancellationToken,
    ) -> Result<ProgressStream, WorkflowError> {
        let Durability::Durable(operations) = &self.durability else {
            return Err(super::contract(
                "a stored progress stream needs a durable runtime (ADR-0114 Outcome 16)",
            ));
        };
        let page = i64::try_from(page)
            .ok()
            .filter(|page| *page > 0)
            .ok_or_else(|| super::contract("a progress page holds at least one row"))?;
        let store = operations.store();
        let reader = if follow {
            Reader::Following(Box::new(store.watch_progress(attempt).await?))
        } else {
            Reader::Stored(store.clone())
        };
        // An unknown attempt is refused before the first read. A followed attempt may be
        // one this process has just started, whose run registers it a moment later.
        let registered = tokio::time::Instant::now() + REGISTRATION;
        loop {
            match store.attempts().get(attempt).await {
                Ok(_) => break,
                Err(pse_operations::OperationsError::NotFound { .. })
                    if follow && tokio::time::Instant::now() < registered =>
                {
                    tokio::select! {
                        () = cancel.cancelled() => break,
                        () = tokio::time::sleep(std::time::Duration::from_millis(20)) => {}
                    }
                }
                Err(error) => return Err(error.into()),
            }
        }
        Ok(ProgressStream {
            reader: Some(reader),
            attempt,
            position: StreamPosition::default(),
            page,
            cancel,
        })
    }
}

impl ProgressStream {
    /// The attempt whose streams these are.
    pub const fn attempt(&self) -> AttemptId {
        self.attempt
    }

    /// Stop reading: the next page is `None`, and a waiting read returns `None`.
    pub fn close(&mut self) {
        self.cancel.cancel();
        self.reader = None;
    }

    /// The next records in observation order, at most the page size of each stream;
    /// `None` once the stream ended or was closed.
    ///
    /// # Errors
    /// Store failures; the store's listener stopped. The stream ends after an error.
    pub async fn next_page(&mut self) -> Result<Option<Vec<StreamRecord>>, WorkflowError> {
        let Some(reader) = self.reader.as_mut() else {
            return Ok(None);
        };
        if self.cancel.is_cancelled() {
            self.reader = None;
            return Ok(None);
        }
        let (position, page) = (self.position, self.page);
        let attempt = self.attempt;
        let read = async {
            match reader {
                Reader::Stored(store) => store.streams().page(attempt, position, page).await,
                Reader::Following(watcher) => watcher.next(position, page).await,
            }
        };
        let cancel = self.cancel.clone();
        let result = tokio::select! {
            () = cancel.cancelled() => None,
            read = read => Some(read),
        };
        let page = match result {
            None => {
                self.reader = None;
                return Ok(None);
            }
            Some(Err(error)) => {
                self.reader = None;
                return Err(error.into());
            }
            Some(Ok(page)) => page,
        };
        if page.is_empty() {
            self.reader = None;
            return Ok(None);
        }
        self.position = page.advance(self.position);
        Ok(Some(records(page)))
    }
}

/// A page's records merged in observation order; progress first among equal instants.
fn records(page: StreamPage) -> Vec<StreamRecord> {
    let mut records: Vec<StreamRecord> = page
        .progress
        .into_iter()
        .map(StreamRecord::Progress)
        .chain(page.incumbents.into_iter().map(StreamRecord::Incumbent))
        .collect();
    // Stable: each stream is already in sequence order.
    records.sort_by_key(StreamRecord::at);
    records
}
