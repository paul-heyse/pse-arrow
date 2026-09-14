// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Encoded-artifact integrity: a plain BLAKE3 hash over the bytes an object actually
//! holds (blueprint §5.3 step 6, ADR-0045).
//!
//! This is the *other* hash. [`crate::id::LogicalHash`] says two relations mean the same
//! thing; an [`EncodingChecksum`] says one object on one store is intact. ADR-0045 keeps
//! them apart because v1 conflated them and could then neither survive an IPC-file/Parquet
//! round trip nor detect a truncated object under a plausible name. The manifest records
//! both, and neither is ever substituted for the other.
//!
//! Unkeyed, unframed and over the *finished* bytes: an IPC file is not complete until its
//! footer is written, so a checksum taken before `finish()` checks a file that does not
//! exist.

use crate::id::{ContentHash, EncodingChecksum};

/// The checksum of a complete stored object.
///
/// Plain, unkeyed BLAKE3 over `bytes`, so `b3sum` reproduces it without this crate.
///
/// ```
/// use pse_ids::encoding_checksum;
///
/// let checksum = encoding_checksum(b"pse");
/// assert_eq!(checksum.0.to_hex().len(), 64);
/// assert_ne!(checksum, encoding_checksum(b"ps"));
/// ```
pub fn encoding_checksum(bytes: &[u8]) -> EncodingChecksum {
    EncodingChecksum(ContentHash::from_bytes(*blake3::hash(bytes).as_bytes()))
}

/// An incremental [`encoding_checksum`], and a `std::io::Write` tee for a writer that
/// produces the bytes it is checking.
///
/// An Arrow IPC or Parquet writer owns its own framing and padding; wrapping it means the
/// checksum covers exactly the bytes that reach the store, with no second pass over a
/// buffer that may no longer exist.
///
/// ```
/// use std::io::Write;
///
/// use pse_ids::{encoding_checksum, EncodingHasher};
///
/// let mut hasher = EncodingHasher::new();
/// hasher.write_all(b"ps").and_then(|()| hasher.write_all(b"e")).expect("in-memory write");
/// assert_eq!(hasher.finish(), encoding_checksum(b"pse"));
/// ```
#[derive(Clone, Debug, Default)]
pub struct EncodingHasher {
    hasher: blake3::Hasher,
}

impl EncodingHasher {
    /// Opens an empty hasher.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the next stored bytes.
    pub fn update(&mut self, bytes: &[u8]) {
        self.hasher.update(bytes);
    }

    /// The checksum of everything added so far.
    pub fn finish(self) -> EncodingChecksum {
        EncodingChecksum(ContentHash::from_bytes(*self.hasher.finalize().as_bytes()))
    }
}

impl std::io::Write for EncodingHasher {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.hasher.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;

    use proptest::prelude::*;

    use super::*;

    #[test]
    fn the_checksum_is_plain_blake3_over_the_bytes() {
        assert_eq!(
            encoding_checksum(b"pse").0.as_bytes(),
            blake3::hash(b"pse").as_bytes()
        );
        assert_eq!(
            encoding_checksum(b"").0.as_bytes(),
            blake3::hash(b"").as_bytes()
        );
    }

    #[test]
    fn a_truncated_object_does_not_check_out() {
        assert_ne!(encoding_checksum(b"pse"), encoding_checksum(b"ps"));
    }

    #[test]
    fn the_tee_writes_everything_it_is_given() {
        let mut hasher = EncodingHasher::new();
        assert_eq!(hasher.write(b"pse").ok(), Some(3));
        assert!(hasher.flush().is_ok());
        assert_eq!(hasher.finish(), encoding_checksum(b"pse"));
    }

    proptest! {
        /// Chunking is a property of the writer, not of the object: any split of the same
        /// bytes checks out the same.
        #[test]
        fn chunking_does_not_change_the_checksum(
            chunks in proptest::collection::vec(
                proptest::collection::vec(any::<u8>(), 0..32),
                0..8,
            ),
        ) {
            let mut streamed = EncodingHasher::new();
            let mut whole: Vec<u8> = Vec::new();
            for chunk in &chunks {
                streamed.update(chunk);
                whole.extend_from_slice(chunk);
            }
            prop_assert_eq!(streamed.finish(), encoding_checksum(&whole));
        }
    }
}
