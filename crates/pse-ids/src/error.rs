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

pse_diagnostics::impl_diagnostic! {
    IdError,
    code(this) { match this {
            Self::Length { .. } | Self::HexLength { .. } | Self::HexDigit { .. } | Self::Prefix { .. } => Some(pse_diagnostics::DiagnosticCode::ValidationInvariant),



            _ => None,
        } },
    forward(_this) { None },
    help(_this) { None },
    related(_this) { None },
    source(_this) { None }
}
