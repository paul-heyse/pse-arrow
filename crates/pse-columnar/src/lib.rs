// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native columnar admission, buffer ownership and engine error boundaries.
use pse_ids::{
    ContentHash, FramedHasher, LogicalHash, SemanticId, canonical_f32_bits, canonical_f64_bits,
    frame, id,
};
pub mod allocation_extent;
pub mod canon;
pub mod contract;
pub mod native_field;
pub mod native_value;
pub mod owned_buffer;
pub mod resource;
pub mod row_token;

mod engine;
pub mod error;
pub use allocation_extent::{
    algorithm_decode_extent, columnar_validation_extent, schema_working_extent,
};
pub use canon::{
    CANON_VERSION, CanonicalOutput, CanonicalizeOptions, IPC_ALIGNMENT, IPC_METADATA_VERSION,
    canonicalize, logical_hash,
};
pub use contract::{CanonicalContract, CanonicalField, Envelope, FieldPath};
pub use datafusion_common::DataFusionError as NativeError;
pub use datafusion_execution::memory_pool::{
    GreedyMemoryPool, MemoryConsumer, MemoryPool, MemoryReservation,
};
pub use engine::{
    Attachment, EngineError, Failure, Observation, PlanOrigin, classify, external, observe,
};
pub use error::{CanonError, EnvelopeBound, ReserveError};
pub use pse_diagnostics::{DiagnosticCode, FailureClass, TypedDiagnostic};
pub use resource::{AllocationLease, CancellationToken, Leased, PayloadOwner};
/// Explicit native conversion for a domain error owned by the invoking native crate.
#[macro_export]
macro_rules! impl_native_error {
    ($ty:ty) => {
        impl From<$ty> for $crate::NativeError {
            fn from(cause: $ty) -> Self {
                $crate::external(cause)
            }
        }
    };
}
#[cfg(test)]
mod diagnostic_unit;
