// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Inspection consumes the same prepared native provider stream as other reads.

use crate::{
    CatalogError,
    session::{OwnedComputationStream, SnapshotSession},
};
use datafusion::arrow::{array::RecordBatch, datatypes::SchemaRef};
use pse_ids::CancellationToken;
use std::{num::NonZeroUsize, sync::Arc};

/// Owned native inspection stream. Exhaustion/close releases unread work; returned
/// arrays independently retain their original leases and result reservations.
#[derive(Debug)]
pub struct TableReader {
    stream: Option<OwnedComputationStream>,
    schema: SchemaRef,
    cancel: CancellationToken,
}
impl TableReader {
    /// Prepare an exact member through its bound provider and effective policy.
    /// # Errors
    /// Absent/ambiguous member, incompatible policy, planning, cancellation or resources.
    pub async fn new(
        session: &SnapshotSession,
        qualified_name: &str,
        port: Option<&str>,
        batch_size: NonZeroUsize,
        cancel: CancellationToken,
    ) -> Result<Self, CatalogError> {
        cancel.checkpoint()?;
        let plan = session.inspection_plan(qualified_name, port)?;
        let prepared = session.prepare(plan, &cancel)?;
        let schema = Arc::new(prepared.optimized_plan().schema().as_arrow().clone());
        let stream = prepared
            .execute_stream(&cancel)
            .await?
            .with_batch_size(batch_size)?;
        Ok(Self {
            stream: Some(stream),
            schema,
            cancel,
        })
    }
    /// Exact declared schema, including recursive extension metadata.
    pub fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
    /// Shared cancellation signal, usable while a consumer is polling the stream.
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancel.clone()
    }

    /// Poll one native batch, bounded by the selected row count.
    /// # Errors
    /// Cancellation, native execution, policy or resource failure; failures close the stream.
    pub async fn next_batch(&mut self) -> Result<Option<RecordBatch>, CatalogError> {
        if let Err(error) = self.cancel.checkpoint() {
            self.stream = None;
            return Err(error.into());
        }
        let Some(stream) = &mut self.stream else {
            return Ok(None);
        };
        loop {
            match stream.next_batch(&self.cancel).await {
                Ok(Some(batch)) if batch.num_rows() == 0 => {}
                Ok(Some(batch)) => return Ok(Some(batch.into_batch())),
                result => {
                    self.stream = None;
                    return result.map(|batch| {
                        batch.map(pse_ids::owned_buffer::OwnedRecordBatch::into_batch)
                    });
                }
            }
        }
    }
    /// Stop unread work. Retained output arrays remain valid.
    pub fn close(&mut self) {
        self.cancel.cancel();
        self.stream = None;
    }
}
