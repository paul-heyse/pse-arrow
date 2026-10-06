// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact admitted canonical progress history, read one immutable payload at a time.
use super::{ProgressEventDocument, Runtime, WorkflowError};
use pse_columnar::{CancellationToken, MemoryConsumer, MemoryReservation};
use pse_operations::canonical_results::ResultRead;
use std::time::Duration;

const LIFETIME: Duration = Duration::from_secs(60);
const READ_BYTES: usize = 16 * 1024 * 1024;

/// A terminal attempt's recorded event stream. Live process observations remain
/// available on `RunHandle`; this reader never follows provisional stored rows.
#[derive(Debug)]
pub struct ProgressStream {
    runtime: Runtime,
    read: Option<ResultRead>,
    run: String,
    attempt: String,
    set: Option<String>,
    batches: u64,
    ordinal: u64,
    owner: Option<MemoryReservation>,
    cancel: CancellationToken,
}
impl Runtime {
    /// Reopen bounded progress receipts of one exact admitted run and attempt.
    pub async fn progress(
        &self,
        run: &str,
        attempt: &str,
        cancel: CancellationToken,
    ) -> Result<ProgressStream, WorkflowError> {
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        let read = self
            .canonical_store()
            .read_results(run, attempt, LIFETIME)
            .await?;
        let descriptor = read.sets().iter().find(|set| set.name == "__progress");
        let (set, batches) =
            descriptor.map_or((None, 0), |set| (Some(set.key.clone()), set.batch_count));
        Ok(ProgressStream {
            runtime: self.clone(),
            read: Some(read),
            run: run.into(),
            attempt: attempt.into(),
            set,
            batches,
            ordinal: 0,
            owner: None,
            cancel,
        })
    }
}
impl ProgressStream {
    /// Exact opaque producing run key.
    pub fn run(&self) -> &str {
        &self.run
    }
    /// Exact opaque producing attempt key.
    pub fn attempt(&self) -> &str {
        &self.attempt
    }
    /// Stop further database work and release this stream's read protection.
    pub fn close(&mut self) {
        self.cancel.cancel();
        self.read = None;
        self.owner = None;
    }
    /// One recorded event batch; failures are surfaced and close unread work.
    pub async fn next_page(&mut self) -> Result<Option<Vec<ProgressEventDocument>>, WorkflowError> {
        if self.read.is_none() {
            return Ok(None);
        }
        if let Err(error) = self.cancel.checkpoint() {
            self.close();
            return Err(pse_engine::EngineError::from(error).into());
        }
        if self.ordinal == self.batches {
            self.read = None;
            self.owner = None;
            return Ok(None);
        }
        let outcome = self.next_checked().await;
        if outcome.is_err() {
            self.close();
        }
        outcome
    }
    async fn next_checked(&mut self) -> Result<Option<Vec<ProgressEventDocument>>, WorkflowError> {
        // Retain the previous page's extent until the caller requests another;
        // typed observations may contain explanatory strings and exact evidence.
        self.owner = None;
        let owner = MemoryConsumer::new("canonical:progress").register(&self.runtime.shared.pool());
        owner
            .try_grow(READ_BYTES)
            .map_err(pse_engine::EngineError::from)?;
        let read = self
            .read
            .as_ref()
            .ok_or_else(|| super::contract("progress read closed"))?;
        read.renew(LIFETIME).await?;
        let set = self
            .set
            .as_deref()
            .ok_or_else(|| super::contract("progress descriptor absent"))?;
        let payload = self
            .runtime
            .canonical_store()
            .result_payload(read, set, self.ordinal)
            .await?;
        if payload.batch.row_count > 512 {
            return Err(super::contract("progress receipt row bound exceeded"));
        }
        let events = super::durable::decode_progress(&payload.batch.payload)?;
        if events.len() as u64 != payload.batch.row_count {
            return Err(super::contract("progress receipt count mismatch"));
        }
        self.ordinal += 1;
        self.owner = Some(owner);
        Ok(Some(events))
    }
}
