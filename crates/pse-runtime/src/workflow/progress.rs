// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact admitted canonical progress history, read one immutable payload at a time.
use super::connected_results::{PrepaidReaderFuture, prepaid_reader_future};
use super::{ProgressCoverage, ProgressEventDocument, Runtime, SourceServices, WorkflowError};
use pse_columnar::{CancellationToken, MemoryConsumer, MemoryReservation};
use pse_operations::{canonical::MetadataResponse, canonical_results::ResultRead};
use std::sync::Arc;

const READ_BYTES: usize = 16 * 1024 * 1024;
type ProgressPageResult = Result<Option<Vec<ProgressEventDocument>>, WorkflowError>;

/// A terminal attempt's recorded event stream. Live process observations remain
/// available on `RunHandle`; this reader never follows provisional stored rows.
#[derive(Debug)]
pub struct ProgressStream {
    services: SourceServices,
    read: Option<Arc<ResultRead>>,
    read_owner: Option<Arc<MemoryReservation>>,
    run: String,
    attempt: String,
    coverage: ProgressCoverage,
    set: Option<String>,
    batches: u64,
    ordinal: u64,
    owner: Option<Arc<MemoryReservation>>,
    cancel: CancellationToken,
}
impl SourceServices {
    /// Reopen bounded progress receipts of one exact admitted run and attempt.
    pub async fn progress(
        &self,
        run: &str,
        attempt: &str,
        cancel: CancellationToken,
    ) -> Result<ProgressStream, WorkflowError> {
        let activity = self.admit_activity()?;
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        activity
            .scope(self.progress_body(run, attempt, cancel)?)
            .await
    }

    // Construct and erase the large terminal-selection and completion-read state
    // before activity and boundary callers poll it. Its grant follows the child
    // through cancellation and drop, independently of response ownership.
    #[inline(never)]
    fn progress_body<'a>(
        &'a self,
        run: &'a str,
        attempt: &'a str,
        cancel: CancellationToken,
    ) -> Result<PrepaidReaderFuture<'a, Result<ProgressStream, WorkflowError>>, WorkflowError> {
        prepaid_reader_future(&self.preparation().pool(), self.progress_checked(run, attempt, cancel))
    }

    async fn progress_checked(
        &self,
        run: &str,
        attempt: &str,
        cancel: CancellationToken,
    ) -> Result<ProgressStream, WorkflowError> {
        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
        let response = super::connected_results::pooled_response_owner(
            &self.preparation().pool(),
            "canonical:progress-selection-response",
            MetadataResponse::Selection,
        )?;
        let read = Arc::new(
            pse_operations::canonical::response_ownership_scope(
                response.clone(),
                self.canonical_store()?.read_results(run, attempt),
            )
            .await?,
        );
        let read_owner = Arc::new(super::connected_results::retain_protected_metadata(
            &self.preparation().pool(),
            response,
            read.retained_bytes()?,
            |owner| read.retain_metadata_owner(owner),
        )?);
        read.retain_metadata_owner(read_owner.clone())?;
        let response = super::connected_results::pooled_response_owner(
            &self.preparation().pool(),
            "canonical:progress-coverage-response",
            MetadataResponse::Header,
        )?;
        let operations = super::Operations::from_store(
            self.canonical_store()?.clone(),
            "progress-reader",
            super::LeasePolicy::default(),
            self.preparation().pool(),
        );
        let coverage = pse_operations::canonical::response_ownership_scope(
            Arc::new((read.clone(), read_owner.clone(), response)),
            operations.progress_coverage(&read),
        )
        .await?;
        let descriptor = read.sets().iter().find(|set| set.name == "__progress");
        let (set, batches) =
            descriptor.map_or((None, 0), |set| (Some(set.key.clone()), set.batch_count));
        Ok(ProgressStream {
            services: self.clone(),
            read: Some(read),
            read_owner: Some(read_owner),
            run: run.into(),
            attempt: attempt.into(),
            coverage,
            set,
            batches,
            ordinal: 0,
            owner: None,
            cancel,
        })
    }
}
impl ProgressStream {
    /// Exact physical progress batch position, scoped to this immutable terminal attempt.
    pub(super) fn batch_position(&self) -> u64 {
        self.ordinal
    }
    /// Resume an observer at a previously captured immutable batch position.
    pub(super) fn resume_batch(&mut self, ordinal: u64) -> Result<(), WorkflowError> {
        if ordinal > self.batches {
            return Err(super::contract(
                "progress observer batch cursor exceeds original history",
            ));
        }
        self.ordinal = ordinal;
        Ok(())
    }
    /// Original terminal telemetry coverage copied under this exact read permission.
    /// This copied fact remains inspectable after close and grants no further reads.
    pub fn coverage(&self) -> &ProgressCoverage {
        &self.coverage
    }
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
        self.read_owner = None;
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
            self.read_owner = None;
            self.owner = None;
            return Ok(None);
        }
        let cancel = self.cancel.clone();
        let outcome = match self.next_checked() {
            Ok(next) => cancel
                .until_cancelled(next)
                .await
                .map_err(pse_engine::EngineError::from)
                .map_err(WorkflowError::from)
                .and_then(std::convert::identity),
            Err(error) => Err(error),
        };
        if outcome.is_err() {
            self.close();
        }
        outcome
    }
    #[inline(never)]
    fn next_checked(
        &mut self,
    ) -> Result<PrepaidReaderFuture<'_, ProgressPageResult>, WorkflowError> {
        let pool = self.services.preparation().pool();
        prepaid_reader_future(&pool, self.next_checked_body())
    }

    async fn next_checked_body(&mut self) -> Result<Option<Vec<ProgressEventDocument>>, WorkflowError> {
        // Retain the previous page's extent until the caller requests another;
        // typed observations may contain explanatory strings and exact evidence.
        self.owner = None;
        let owner = Arc::new(
            MemoryConsumer::new("canonical:progress").register(&self.services.preparation().pool()),
        );
        owner
            .try_grow(READ_BYTES)
            .map_err(pse_engine::EngineError::from)?;
        let read = self
            .read
            .as_ref()
            .ok_or_else(|| super::contract("progress read closed"))?;

        let set = self
            .set
            .as_deref()
            .ok_or_else(|| super::contract("progress descriptor absent"))?;
        let response = super::connected_results::pooled_response_owner(
            &self.services.preparation().pool(),
            "canonical:progress-file-response",
            MetadataResponse::Page,
        )?;
        let read_owner = self
            .read_owner
            .as_ref()
            .ok_or_else(|| super::contract("progress metadata owner closed"))?;
        let response = Arc::new((read.clone(), read_owner.clone(), response, owner.clone()));
        let descriptors = pse_operations::canonical::response_ownership_scope(
            response.clone(),
            self.services
                .canonical_store()?
                .result_payload_batch_descriptors(read, set, &[self.ordinal]),
        )
        .await?;
        let descriptor = descriptors
            .first()
            .ok_or_else(|| super::contract("progress file descriptor absent"))?;
        let bytes = usize::try_from(descriptor.payload_bytes)
            .map_err(|_| super::contract("progress file extent"))?;
        owner
            .try_grow(bytes.saturating_mul(8))
            .map_err(pse_engine::EngineError::from)?;
        let payload = pse_operations::canonical::response_ownership_scope(
            response,
            self.services
                .canonical_store()?
                .result_payload(read, set, self.ordinal),
        )
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

impl Runtime {
    /// Read exact original progress through the selected source/read capability.
    pub async fn progress(
        &self,
        run: &str,
        attempt: &str,
        cancel: CancellationToken,
    ) -> Result<ProgressStream, WorkflowError> {
        self.source_services().progress(run, attempt, cancel).await
    }
}


#[cfg(test)]
mod progress_future_unit {
    use super::*;
    use std::future::Future;

    // Type layout can be inspected without constructing a source or touching a
    // canonical store. These closures are never invoked.
    fn future_bytes<A, F: Future>(_: impl FnOnce(A) -> F) -> usize {
        size_of::<F>()
    }

    #[test]
    fn progress_forwarding_futures_do_not_embed_the_native_read_state() {
        let constructor = future_bytes(|source: &'static SourceServices| {
            source.progress("run", "attempt", CancellationToken::new())
        });
        let constructor_body = future_bytes(|source: &'static SourceServices| {
            source.progress_checked("run", "attempt", CancellationToken::new())
        });
        let page = future_bytes(|stream: &'static mut ProgressStream| stream.next_page());
        let page_body = future_bytes(|stream: &'static mut ProgressStream| stream.next_checked_body());
        assert!(constructor < constructor_body);
        assert!(page < page_body);
        eprintln!(
            "progress future bytes: constructor={constructor}, constructor_body={constructor_body}, page={page}, page_body={page_body}"
        );
    }
}
