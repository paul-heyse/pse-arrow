// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact binary content: the value of a registry `Binary` column (ADR-0125).

/// Exact bytes, compared, framed and stored byte for byte. A package data document's
/// content is one; text is never implied.
#[derive(
    Clone,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(transparent)]
pub struct Bytes(Vec<u8>);

impl Bytes {
    /// Own `bytes` without copying.
    pub const fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
    /// The exact bytes.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
    /// Release the owned bytes.
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl std::ops::Deref for Bytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bytes({} bytes)", self.0.len())
    }
}

impl crate::SemanticEq for Bytes {
    fn semantic_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl crate::HeapUsage for Bytes {
    fn heap_bytes(&self) -> usize {
        self.0.capacity()
    }
}

impl crate::SemanticFrame for Bytes {
    fn frame(&self, hash: &mut pse_ids::FramedHasher) {
        hash.part(&self.0);
    }
}
