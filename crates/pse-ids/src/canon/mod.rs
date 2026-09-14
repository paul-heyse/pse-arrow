// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The `pse.canon.v2` canonicalizer (blueprint §5.3, ADR-0045).
//!
//! Only the frozen constants are declared here. `canonicalize`, `logical_hash` and the
//! `stages` module — admission and ordering, recursive normalization, the canonical
//! metadata relation, framing and hashing — are packet B-canon's; this module is the seam
//! they land in, so that the constants the rest of the workspace cites already exist and
//! have exactly one declaration.
//!
//! The three constants below are part of the hashing contract, not tuning parameters.
//! Alignment alone changes the bytes — 872 against 1032 for identical rows — so changing
//! any of them is a new canonicalization version and invalidates every stored logical
//! hash. That is why they are `v2` rather than `v1`: ADR-0045 found that v1's ignored-null
//! payload and its stream/file ambiguity could not be repaired while keeping the name.

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
