// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-owned opaque-byte correspondence, independent of any wire library.
use super::AdapterError;

/// Checked byte access at the actual native scalar owner. This structural conversion
/// does not interpret content identities or admit scientific/current authority.
pub trait ProjectionBytes: Sized {
    /// Consume the actual native byte value without a wire dependency.
    fn into_projection_bytes(self) -> Vec<u8>;
    /// Reconstruct this scalar through its actual checked width/value contract.
    fn from_projection_bytes(bytes: Vec<u8>, path: &str) -> Result<Self, AdapterError>;
    /// Adopt borrowed bytes into this actual native owner. Fixed-width owners can
    /// override this to copy their array directly without a temporary allocation.
    fn from_projection_slice(bytes: &[u8], path: &str) -> Result<Self, AdapterError> {
        Self::from_projection_bytes(bytes.to_vec(), path)
    }
}
impl ProjectionBytes for Vec<u8> {
    fn into_projection_bytes(self) -> Vec<u8> { self }
    fn from_projection_bytes(bytes: Vec<u8>, _: &str) -> Result<Self, AdapterError> { Ok(bytes) }
}
impl<T: ProjectionBytes> ProjectionBytes for Box<T> {
    fn into_projection_bytes(self) -> Vec<u8> { (*self).into_projection_bytes() }
    fn from_projection_bytes(bytes: Vec<u8>, path: &str) -> Result<Self, AdapterError> { T::from_projection_bytes(bytes, path).map(Box::new) }
    fn from_projection_slice(bytes: &[u8], path: &str) -> Result<Self, AdapterError> { T::from_projection_slice(bytes, path).map(Box::new) }
}
