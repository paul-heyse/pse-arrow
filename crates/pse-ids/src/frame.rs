// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The unkeyed framing used by the `pse.canon.v2` and `pse.snapshot.v2` preimages
//! (blueprint §5.3 steps 4 and 7).
//!
//! One trait with four operations, so the same code can build a preimage in memory (for a
//! fixture to inspect) and stream it into a hasher (for production), and neither can drift
//! from the other.
//!
//! **This is not the framing of [`crate::derive`].** A derived identity is keyed
//! (`blake3::Hasher::new_derive_key`) and *every* part carries a `u64` little-endian
//! length, integers included. A canonical preimage is unkeyed and mixes variable-length
//! components, which carry a length, with fixed-width components — IDs, hashes, counts and
//! schema versions — which carry their declared width and no length. Both are frozen; the
//! distinction is the contract, not an inconsistency.

/// A sink that accepts the framed components of a canonical preimage.
///
/// Implemented for `Vec<u8>` (the preimage as bytes) and for `blake3::Hasher` (the preimage
/// as a digest, without ever materializing it).
pub trait FrameSink {
    /// A variable-length component: `u64` little-endian byte length, then the bytes.
    fn put_len_prefixed(&mut self, bytes: &[u8]) {
        self.put_u64_le(framed_len(bytes));
        self.put_fixed(bytes);
    }

    /// A fixed-width 32-bit component: four little-endian bytes, no length.
    fn put_u32_le(&mut self, v: u32) {
        self.put_fixed(&v.to_le_bytes());
    }

    /// A fixed-width 64-bit component — a count, an ordinal: eight little-endian bytes,
    /// no length.
    fn put_u64_le(&mut self, v: u64) {
        self.put_fixed(&v.to_le_bytes());
    }

    /// A component whose width the contract fixes — an ID, a hash: the bytes verbatim.
    fn put_fixed(&mut self, bytes: &[u8]);
}

/// The `u64` length of a variable-length component.
///
/// `usize` is at most 64 bits on every supported target, so the saturating fallback is
/// unreachable; it exists because the crate's panic policy has no room for an `expect`
/// that only a 128-bit address space could reach.
fn framed_len(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

impl FrameSink for Vec<u8> {
    fn put_fixed(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}
impl FrameSink for blake3::Hasher {
    fn put_fixed(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }
}

#[cfg(test)]
mod consolidation_unit {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn a_length_prefix_is_eight_little_endian_bytes() {
        let mut buffer = Vec::new();
        buffer.put_len_prefixed(b"pse");
        assert_eq!(buffer, vec![3, 0, 0, 0, 0, 0, 0, 0, b'p', b's', b'e']);
    }

    #[test]
    fn fixed_width_components_carry_no_length() {
        let mut buffer = Vec::new();
        buffer.put_u32_le(1);
        buffer.put_u64_le(2);
        buffer.put_fixed(&[0xaa, 0xbb]);
        assert_eq!(buffer, vec![1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0xaa, 0xbb]);
    }

    #[test]
    fn length_prefixing_separates_a_split_that_concatenation_would_not() {
        let mut left = Vec::new();
        left.put_len_prefixed(b"ab");
        left.put_len_prefixed(b"c");

        let mut right = Vec::new();
        right.put_len_prefixed(b"a");
        right.put_len_prefixed(b"bc");

        assert_ne!(left, right);
    }

    proptest! {
        /// The hasher sink and the byte sink agree: hashing the built preimage equals
        /// streaming the same components into a hasher.
        #[test]
        fn the_two_sinks_frame_identically(
            parts in proptest::collection::vec(proptest::collection::vec(any::<u8>(), 0..24), 0..8),
            counts in proptest::collection::vec(any::<u64>(), 0..4),
            versions in proptest::collection::vec(any::<u32>(), 0..4),
        ) {
            let mut bytes: Vec<u8> = Vec::new();
            let mut hasher = blake3::Hasher::new();
            for part in &parts {
                bytes.put_len_prefixed(part);
                hasher.put_len_prefixed(part);
                bytes.put_fixed(part);
                hasher.put_fixed(part);
            }
            for count in &counts {
                bytes.put_u64_le(*count);
                hasher.put_u64_le(*count);
            }
            for version in &versions {
                bytes.put_u32_le(*version);
                hasher.put_u32_le(*version);
            }
            let whole = blake3::hash(&bytes);
            let streamed = hasher.finalize();
            prop_assert_eq!(whole.as_bytes(), streamed.as_bytes());
        }
    }
}
