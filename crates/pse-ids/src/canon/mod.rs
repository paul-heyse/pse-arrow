// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The `pse.canon.v2` canonicalizer (blueprint §5.3, ADR-0045).
//!
//! Direct contract/value admission precedes ordering, recursive normalization and two
//! separately finished Arrow IPC streams. Registry-context admission remains the
//! caller's prerequisite for publication and reuse; a hash does not establish it.

mod api;
mod normalize;
mod preflight;
mod stages;

/// Timings of actual canonical construction stages with all production checks enabled.
#[cfg(feature = "bench-instrumentation")]
pub mod benchmark;

pub use api::{CanonicalOutput, CanonicalizeOptions, canonicalize, logical_hash};

/// The frozen canonicalization contract version; the first component of the preimage.
pub const CANON_VERSION: &str = "pse.canon.v2";

/// The IPC alignment of the canonical streams (`IpcWriteOptions::try_new` argument one).
pub const IPC_ALIGNMENT: usize = 64;

/// The IPC metadata version of the canonical streams; never the legacy format.
pub const IPC_METADATA_VERSION: arrow_ipc::MetadataVersion = arrow_ipc::MetadataVersion::V5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_canonical_constants_are_the_frozen_ones() {
        assert_eq!(CANON_VERSION, "pse.canon.v2");
        assert_eq!(IPC_ALIGNMENT, 64);
        assert_eq!(IPC_METADATA_VERSION, arrow_ipc::MetadataVersion::V5);
    }
}
