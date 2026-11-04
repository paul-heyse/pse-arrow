// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Every error this crate returns, with its blueprint §23.2 class.
//!
//! The classes are spelled as `#[diagnostic(code(...))]` in Rust path form, which is what
//! `tests/governance/tests/error_taxonomy.rs` and the §23.2 table agree on. Nothing here
//! carries a `Debug` rendering into a message a hash could ever see: hashes are computed
//! from framed bytes (§5.3 step 4), errors are read by people.

/// A 128-bit identity or 256-bit hash could not be read from bytes or text.
///
/// Identity parsing is a validation step, not an infrastructure failure: a caller handed
/// over something that is not an ID, so the class is `validation::invariant` (§23.2).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum IdError {
    /// A native structural membership traversal exceeded its admitted resource.
    #[error("native scientific membership allowance exhausted: {resource}")]
    ScientificMembershipLimit {
        /// Actual exhausted traversal resource.
        resource: &'static str,
    },
    /// A native source uses unsupported JSON normalization.
    #[error("unsupported native scientific membership profile: {profile}")]
    ScientificMembershipProfile {
        /// Unsupported source/normalization behavior.
        profile: &'static str,
    },
    /// Native serialization emitted a different shape after its count pass.
    #[error("native scientific membership serialization changed after counting")]
    ScientificMembershipChangedShape,
    /// Native serialization refused with its original bounded explanation.
    #[error("native scientific membership serialization: {detail}")]
    ScientificMembershipSerialization {
        /// Original bounded serialization explanation.
        detail: String,
    },
    /// A preadmitted native traversal allocation failed.
    #[error("native scientific membership allocation failed: {detail}")]
    ScientificMembershipAllocation {
        /// Original allocator refusal explanation.
        detail: String,
    },
    /// A scientific replay descriptor exceeds its bounded wire extent.
    #[error("scientific replay descriptor exceeds the bounded wire extent")]
    ScientificRecipeLimit,
    /// A scientific replay descriptor is malformed JSON.
    #[error("malformed scientific replay descriptor")]
    ScientificRecipeEncoding,
    /// The qualified immutable descriptor digest differs from its actual bytes.
    #[error("scientific replay descriptor differs from qualified payload digest")]
    ScientificRecipeDigest,
    /// A byte slice did not have the exact declared width.
    #[error("expected {expected} bytes, received {actual}")]
    Length {
        /// The declared width of the identity type.
        expected: usize,
        /// The width of the slice that was offered.
        actual: usize,
    },

    /// A hexadecimal string did not have exactly two digits per byte.
    #[error("expected {expected} hexadecimal digits, received {actual}")]
    HexLength {
        /// Twice the declared width of the identity type.
        expected: usize,
        /// The number of characters that were offered.
        actual: usize,
    },

    /// A character in a hexadecimal string was not a hexadecimal digit.
    #[error("`{found}` at position {position} is not a hexadecimal digit")]
    HexDigit {
        /// Zero-based character position of the offending character.
        position: usize,
        /// The offending character.
        found: char,
    },

    /// A prefixed hash was offered without its algorithm prefix.
    #[error("expected the `{expected}` prefix")]
    Prefix {
        /// The prefix the format requires, `blake3:` for a content hash.
        expected: &'static str,
    },
}

impl IdError {
    /// Owned explanation storage beyond the inline error value.
    pub fn retained_heap_bytes(&self) -> usize {
        match self {
            Self::ScientificMembershipSerialization { detail }
            | Self::ScientificMembershipAllocation { detail } => detail.capacity(),
            _ => 0,
        }
    }
}

pse_diagnostics::impl_diagnostic! {
    IdError,
    code(this) { match this {
            Self::ScientificMembershipLimit { .. } | Self::ScientificMembershipAllocation { .. } => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
            Self::ScientificMembershipProfile { .. } | Self::ScientificMembershipSerialization { .. } => Some(pse_diagnostics::DiagnosticCode::ValidationInvariant),
            Self::ScientificMembershipChangedShape => Some(pse_diagnostics::DiagnosticCode::InternalInvariant),
            Self::ScientificRecipeLimit | Self::ScientificRecipeEncoding | Self::ScientificRecipeDigest | Self::Length { .. } | Self::HexLength { .. } | Self::HexDigit { .. } | Self::Prefix { .. } => Some(pse_diagnostics::DiagnosticCode::ValidationInvariant),



            _ => None,
        } },
    forward(_this) { None },
    help(_this) { None },
    related(_this) { None },
    source(_this) { None }
}
