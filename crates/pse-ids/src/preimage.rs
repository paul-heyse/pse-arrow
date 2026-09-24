// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Streaming canonical preimages without exposing the hashing implementation.
use crate::{ContentHash, FrameSink};
/// Hash owner for already framed canonical input, not a second framing contract.
#[derive(Debug)]
pub struct PreimageHasher(blake3::Hasher);
impl Default for PreimageHasher {
    fn default() -> Self {
        Self::new()
    }
}
impl PreimageHasher {
    /// Unkeyed canonical content hash.
    pub fn new() -> Self {
        Self(blake3::Hasher::new())
    }
    /// Domain-separated hash for an existing declared preimage contract.
    pub fn new_derive_key(context: &str) -> Self {
        Self(blake3::Hasher::new_derive_key(context))
    }
    /// Append already framed bytes.
    pub fn update(&mut self, bytes: &[u8]) -> &mut Self {
        self.0.update(bytes);
        self
    }
    /// Current digest, without consuming the stream.
    pub fn finalize(&self) -> ContentHash {
        ContentHash::from_bytes(*self.0.finalize().as_bytes())
    }
}
impl FrameSink for PreimageHasher {
    fn put_fixed(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }
}
/// Unkeyed digest for already encoded/framed bytes.
pub fn hash(bytes: &[u8]) -> ContentHash {
    ContentHash::from_bytes(*blake3::hash(bytes).as_bytes())
}
/// Digest bytes for an existing domain-separated preimage.
pub fn derive_key(context: &str, bytes: &[u8]) -> [u8; 32] {
    blake3::derive_key(context, bytes)
}
