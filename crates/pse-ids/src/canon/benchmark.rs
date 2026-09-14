// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Feature-gated timing observations of the actual reserved production pipeline.
//! Each sample still executes every stage, including admission, cancellation,
//! serialization and ownership release. It cannot enter a stage without preflight.

use std::time::{Duration, Instant};

use super::api::{Stage, canonicalize_observed};
use crate::{CanonError, CanonicalContract, CanonicalOutput, CanonicalizeOptions, MemoryReserver};
use arrow_array::RecordBatch;

/// Actual elapsed times for successive canonical construction stages.
#[derive(Clone, Copy, Debug, Default)]
pub struct StageTimings {
    /// Bounds, value admission and reservation acquisition.
    pub preflight: Duration,
    /// Concatenation, unique key admission, key ordering and row permutation.
    pub admission_and_order: Duration,
    /// Recursive value normalization and the normalized-size postcondition.
    pub normalize: Duration,
    /// Sorted metadata relation construction.
    pub metadata: Duration,
    /// Both finished canonical IPC streams.
    pub ipc_streams: Duration,
    /// Complete frame construction and BLAKE3 hashing.
    pub frame_and_hash: Duration,
}
impl StageTimings {
    /// Stable benchmark labels paired with their measured stage times.
    pub const fn stages(self) -> [(&'static str, Duration); 6] {
        [
            ("preflight", self.preflight),
            ("admission_and_order", self.admission_and_order),
            ("normalize", self.normalize),
            ("metadata", self.metadata),
            ("ipc_streams", self.ipc_streams),
            ("frame_and_hash", self.frame_and_hash),
        ]
    }
}

/// Run ordinary canonicalization, retaining only requested outputs and stage timings.
///
/// # Errors
/// Every production admission, envelope, resource, cancellation and encoding error.
pub fn canonicalize_stages(
    contract: &CanonicalContract,
    batches: &[RecordBatch],
    reserver: &dyn MemoryReserver,
    options: CanonicalizeOptions,
) -> Result<(CanonicalOutput, StageTimings), CanonError> {
    let mut timings = StageTimings::default();
    let mut start = Instant::now();
    let result = canonicalize_observed(contract, batches, reserver, options, |stage, entering| {
        if entering {
            start = Instant::now();
        } else {
            let duration = start.elapsed();
            match stage {
                Stage::Preflight => timings.preflight = duration,
                Stage::AdmissionAndOrder => timings.admission_and_order = duration,
                Stage::Normalize => timings.normalize = duration,
                Stage::Metadata => timings.metadata = duration,
                Stage::IpcStreams => timings.ipc_streams = duration,
                Stage::FrameAndHash => timings.frame_and_hash = duration,
            }
        }
    })?;
    Ok((result, timings))
}
