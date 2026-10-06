// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Content identities whose semantic roles cannot be substituted implicitly (ADR-0150).
crate::semantic_id_newtype! {
    /// Exact source inventory and revision attribution.
    SourceRevisionHash: ContentHash,
    /// Complete admitted documents, imports, physical and capability closure.
    AdmittedClosureHash: ContentHash,
    /// Normalized mathematics and its complete semantic dependencies.
    SemanticBodyHash: ContentHash,
    /// Compatible structure, coordinate order and compiler settings.
    PreparedViewHash: ContentHash,
    /// Canonical member values and physical binding context.
    BindingHash: ContentHash,
    /// Effective numerical execution settings.
    ProfileHash: ContentHash,
    /// Scientific request attribution, distinct from a submitted occurrence.
    LineageRequestHash: ContentHash,
    /// Submitted occurrence and idempotency scope.
    OperationalJobHash: ContentHash,
}
/// Role boundaries reject implicit substitutions.
/// ```compile_fail
/// use pse_ids::roles::{BindingHash, ProfileHash};
/// fn profile(_: ProfileHash) {}
/// profile(BindingHash::from_bytes([0; 32]));
/// ```
#[derive(Debug)]
pub struct RoleBoundary;
impl SourceRevisionHash {
    /// Canonical display form at publication boundaries.
    pub fn to_prefixed(self) -> String {
        self.as_id().to_prefixed()
    }
}
impl ProfileHash {
    /// Canonical display form at publication boundaries.
    pub fn to_prefixed(self) -> String {
        self.as_id().to_prefixed()
    }
}
/// Recorded operational evidence retains its original frame, including unknown provenance.
/// Historical evidence cannot implicitly satisfy a current operational cache key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedOperationalJobIdentity {
    /// The original stored bytes.
    pub digest: crate::ContentHash,
    /// Independently recorded frame; absence means unknown historical provenance.
    pub frame: Option<String>,
}
impl RecordedOperationalJobIdentity {
    /// A newly framed current operational request.
    pub fn current(value: OperationalJobHash) -> Self {
        Self {
            digest: value.as_id(),
            frame: Some(crate::Frame::DurableJobRequestV6.as_str().to_owned()),
        }
    }
    /// Read evidence without upgrading its original frame.
    pub const fn recorded(digest: crate::ContentHash, frame: Option<String>) -> Self {
        Self { digest, frame }
    }
    /// A current key only when recorded provenance establishes the required frame.
    pub fn current_key(&self) -> Option<OperationalJobHash> {
        (self.frame.as_deref() == Some(crate::Frame::DurableJobRequestV6.as_str()))
            .then(|| OperationalJobHash::from(self.digest))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recorded_frames_do_not_grant_current_role_proof() {
        let digest = crate::ContentHash::from_bytes([3; 32]);
        let unknown = RecordedOperationalJobIdentity::recorded(digest, None);
        let historical = RecordedOperationalJobIdentity::recorded(
            digest,
            Some(crate::Frame::DurableJobRequestV3.as_str().into()),
        );
        assert_eq!(unknown.digest, historical.digest);
        assert!(unknown.current_key().is_none());
        assert!(historical.current_key().is_none());
        let current = RecordedOperationalJobIdentity::current(OperationalJobHash::from(digest));
        assert_eq!(
            current.current_key(),
            Some(OperationalJobHash::from(digest))
        );
    }
}
