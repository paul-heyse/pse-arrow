// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use crate::{
    AllocationLease, CancellationToken, CanonError, CanonicalContract, Envelope, LogicalHash,
    MemoryPool,
};
use arrow_array::RecordBatch;
use bytes::Bytes;

/// Deployment-selected construction limits and optional retained outputs.
#[derive(Clone, Debug, Default)]
pub struct CanonicalizeOptions {
    /// The selected row and normalized-buffer resource envelope.
    pub envelope: Envelope,
    /// Retain the complete framed canonical preimage with its reservation owner.
    pub keep_preimage: bool,
    /// Retain original values in primary-key order, with reservation-owning buffers.
    pub keep_sorted: bool,
    /// Driver cancellation checked before and between allocation stages.
    pub cancel: Option<CancellationToken>,
}

/// Logical identity and requested outputs. Every retained byte or Arrow buffer carries
/// its allocation lease; cloning or slicing an output cannot detach its reservation.
#[derive(Debug)]
pub struct CanonicalOutput {
    /// BLAKE3 of the exact `pse.canon.v2` preimage.
    pub logical_hash: LogicalHash,
    /// Rows in the complete relation.
    pub row_count: u64,
    /// The complete framed preimage, when requested.
    pub preimage: Option<Bytes>,
    /// Original values in unique primary-key order, when requested.
    pub sorted: Option<crate::owned_buffer::OwnedRecordBatch>,
}

/// Admits the exact declared storage, visible nullability, enum domain and unique key,
/// then builds a recursive normalized hashing copy under a single finite reservation.
/// Registry-specific quantities, foreign keys and domain invariants are admitted by the
/// relation/catalog boundary before publication or reuse; this hash is never their proof.
///
/// Input allocations remain their caller's responsibility. A conservative temporary
/// input claim accounts for their coexistence with all canonical construction stages.
///
/// # Errors
/// Contract/value violations, supported-envelope limits, reservation exhaustion,
/// cancellation, Arrow failures or an internal postcondition violation.
pub fn canonicalize(
    contract: &CanonicalContract,
    batches: &[RecordBatch],
    pool: &std::sync::Arc<dyn MemoryPool>,
    options: CanonicalizeOptions,
) -> Result<CanonicalOutput, CanonError> {
    canonicalize_observed(contract, batches, pool, options, |_, _| {})
}

/// Internal boundaries shared by production and optional timing instrumentation.
#[derive(Clone, Copy)]
pub(super) enum Stage {
    Preflight,
    AdmissionAndOrder,
    Normalize,
    Metadata,
    IpcStreams,
    FrameAndHash,
}

pub(super) fn canonicalize_observed(
    contract: &CanonicalContract,
    batches: &[RecordBatch],
    pool: &std::sync::Arc<dyn MemoryPool>,
    options: CanonicalizeOptions,
    mut observe: impl FnMut(Stage, bool),
) -> Result<CanonicalOutput, CanonError> {
    observe(Stage::Preflight, true);
    let cancel = options.cancel.unwrap_or_default();
    cancel.checkpoint()?;
    let reservation = crate::MemoryConsumer::new(format!(
        "canonicalize:{}@{}",
        contract.relation_id, contract.schema_version.0
    ))
    .register(pool);
    let control = super::preflight::control_bytes(contract)?;
    reservation.try_grow(control)?;
    let estimate = super::preflight::estimate(contract, batches, options.envelope, &cancel)?;
    reservation.try_grow(estimate.reservation.saturating_sub(reservation.size()))?;
    cancel.checkpoint()?;
    // The mutable reservation outlives every construction stage. It is frozen only
    // after temporary stages have dropped and the final retained extent is known.
    observe(Stage::Preflight, false);
    observe(Stage::AdmissionAndOrder, true);
    let sorted = super::stages::admit_and_order(contract, batches)?;
    observe(Stage::AdmissionAndOrder, false);
    cancel.checkpoint()?;
    observe(Stage::Normalize, true);
    let normalized = super::normalize::batch(contract, &sorted)?;
    cancel.checkpoint()?;
    let actual = normalized
        .columns()
        .iter()
        .try_fold(0usize, |total, array| {
            super::preflight::add(total, logical_buffer_bytes(&array.to_data())?)
        })?;
    if actual != estimate.normalized {
        return Err(CanonError::Internal(format!(
            "normalized buffer extent {actual} differs from preflight {}",
            estimate.normalized
        )));
    }
    observe(Stage::Normalize, false);
    observe(Stage::Metadata, true);
    let metadata = super::stages::metadata_relation(contract)?;
    observe(Stage::Metadata, false);
    observe(Stage::IpcStreams, true);
    let metadata_stream = super::stages::stream(&metadata, estimate.stream_limit)?;
    let data_stream = super::stages::stream(&normalized, estimate.stream_limit)?;
    observe(Stage::IpcStreams, false);
    observe(Stage::FrameAndHash, true);
    let (logical_hash, mut preimage) = super::stages::frame_and_hash(
        contract,
        &metadata_stream,
        &data_stream,
        options.keep_preimage,
    )?;
    observe(Stage::FrameAndHash, false);
    drop(metadata_stream);
    drop(data_stream);
    cancel.checkpoint()?;
    drop(normalized);
    drop(metadata);
    if let Some(bytes) = &mut preimage {
        bytes.shrink_to_fit();
    }
    let sorted = options.keep_sorted.then_some(sorted);
    let retained = super::preflight::add(
        sorted
            .as_ref()
            .map(crate::owned_buffer::retained_buffer_bytes)
            .transpose()?
            .unwrap_or(0),
        preimage.as_ref().map_or(0, Vec::capacity),
    )?;
    reservation.shrink(reservation.size().saturating_sub(retained));
    let sorted = sorted
        .map(|batch| {
            let bytes = crate::owned_buffer::retained_buffer_bytes(&batch)?;
            crate::owned_buffer::OwnedRecordBatch::from_reserved(batch, reservation.split(bytes))
        })
        .transpose()?;
    let preimage = preimage
        .map(|bytes| {
            let extent = bytes.capacity();
            crate::owned_buffer::attach_bytes(
                bytes,
                AllocationLease::new(reservation.split(extent)),
            )
        })
        .transpose()?;
    Ok(CanonicalOutput {
        logical_hash,
        row_count: u64::try_from(estimate.rows)
            .map_err(|_| CanonError::Internal("row count does not fit u64".to_owned()))?,
        preimage,
        sorted,
    })
}

/// Computes only logical identity, releasing all temporary reservations on return.
///
/// # Errors
/// The same admission, resource and serialization failures as [`canonicalize`].
pub fn logical_hash(
    contract: &CanonicalContract,
    batches: &[RecordBatch],
    pool: &std::sync::Arc<dyn MemoryPool>,
) -> Result<LogicalHash, CanonError> {
    Ok(canonicalize(contract, batches, pool, CanonicalizeOptions::default())?.logical_hash)
}
fn logical_buffer_bytes(data: &arrow::array::ArrayData) -> Result<usize, CanonError> {
    let mut total = data.buffers().iter().try_fold(0usize, |total, buffer| {
        super::preflight::add(total, buffer.len())
    })?;
    if let Some(nulls) = data.nulls() {
        total = super::preflight::add(total, nulls.buffer().len())?;
    }
    for child in data.child_data() {
        total = super::preflight::add(total, logical_buffer_bytes(child)?)?;
    }
    Ok(total)
}
