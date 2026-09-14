// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Every error this crate returns, with its blueprint §23.2 class.
//!
//! The classes are spelled as `#[diagnostic(code(...))]` in Rust path form, which is what
//! `tests/governance/tests/error_taxonomy.rs` and the §23.2 table agree on. Nothing here
//! carries a `Debug` rendering into a message a hash could ever see: hashes are computed
//! from framed bytes (§5.3 step 4), errors are read by people.

use crate::contract::FieldPath;

/// A 128-bit identity or 256-bit hash could not be read from bytes or text.
///
/// Identity parsing is a validation step, not an infrastructure failure: a caller handed
/// over something that is not an ID, so the class is `validation::invariant` (§23.2).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, miette::Diagnostic)]
#[non_exhaustive]
pub enum IdError {
    /// A byte slice did not have the exact declared width.
    #[error("expected {expected} bytes, received {actual}")]
    #[diagnostic(code(validation::invariant))]
    Length {
        /// The declared width of the identity type.
        expected: usize,
        /// The width of the slice that was offered.
        actual: usize,
    },

    /// A hexadecimal string did not have exactly two digits per byte.
    #[error("expected {expected} hexadecimal digits, received {actual}")]
    #[diagnostic(code(validation::invariant))]
    HexLength {
        /// Twice the declared width of the identity type.
        expected: usize,
        /// The number of characters that were offered.
        actual: usize,
    },

    /// A character in a hexadecimal string was not a hexadecimal digit.
    #[error("`{found}` at position {position} is not a hexadecimal digit")]
    #[diagnostic(code(validation::invariant))]
    HexDigit {
        /// Zero-based character position of the offending character.
        position: usize,
        /// The offending character.
        found: char,
    },

    /// A prefixed hash was offered without its algorithm prefix.
    #[error("expected the `{expected}` prefix")]
    #[diagnostic(code(validation::invariant))]
    Prefix {
        /// The prefix the format requires, `blake3:` for a content hash.
        expected: &'static str,
    },
}

/// Which `pse.canon.v2` construction bound was exceeded (blueprint §5.3 step 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EnvelopeBound {
    /// Rows in the complete relation.
    Rows,
    /// Bytes of normalized value, offset and validity buffers.
    Bytes,
    /// A `Utf8`/`List` offset that does not fit `i32`.
    Offset,
}

impl EnvelopeBound {
    /// The stable lowercase spelling used in messages and in fixtures.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rows => "rows",
            Self::Bytes => "bytes",
            Self::Offset => "offset",
        }
    }
}

impl std::fmt::Display for EnvelopeBound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A relation could not be admitted, normalized or framed under `pse.canon.v2`.
///
/// The variants split along the §23.2 classes deliberately: a contract problem is a
/// `schema::*` failure the author can fix, a bound or a reservation is
/// `runtime::resource_limit`, and a broken postcondition is `internal::invariant` and
/// never dressed up as either of the first two.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[non_exhaustive]
pub enum CanonError {
    /// An Arrow layout outside the supported matrix (blueprint §5.3 step 2).
    #[error("field `{path}` has layout `{what}`, which `pse.canon.v2` does not support")]
    #[diagnostic(code(schema::unsupported_layout))]
    UnsupportedLayout {
        /// Registry field path of the offending field.
        path: FieldPath,
        /// Terse description of the offending layout.
        what: String,
    },

    /// A batch does not match the declared contract schema.
    #[error("`{path}` is `{actual}` but the contract declares `{expected}`")]
    #[diagnostic(code(schema::contract_mismatch))]
    ContractMismatch {
        /// Registry field path of the offending field, or the empty path for the schema.
        path: FieldPath,
        /// What the contract declares.
        expected: String,
        /// What the batch offered.
        actual: String,
    },

    /// A metadata key outside the registered `pse.` and `ARROW:extension:` namespaces.
    #[error("metadata key `{key}` on `{path}` is not registered")]
    #[diagnostic(code(schema::unregistered_metadata))]
    UnregisteredMetadata {
        /// Registry field path carrying the key, or the empty path for the schema.
        path: FieldPath,
        /// The offending key.
        key: String,
    },

    /// A primary-key column that cannot order a relation.
    #[error("primary-key column `{column}` is invalid: {reason}")]
    #[diagnostic(code(schema::invalid_key))]
    InvalidKey {
        /// The column named by the contract.
        column: String,
        /// Why it cannot be a key.
        reason: String,
    },

    /// A required key column held a null (blueprint §5.3 step 1).
    #[error("primary-key column `{column}` is null in row {row}")]
    #[diagnostic(code(validation::invariant))]
    NullKey {
        /// The key column.
        column: String,
        /// Zero-based row index in the concatenated relation.
        row: usize,
    },

    /// Two rows shared a primary key (blueprint §5.3 step 1).
    #[error("rows {first} and {second} share a primary key")]
    #[diagnostic(code(validation::invariant))]
    DuplicateKey {
        /// Zero-based index of the first row of the pair.
        first: usize,
        /// Zero-based index of the second row of the pair.
        second: usize,
    },

    /// A dictionary code addressed a value outside its dictionary.
    #[error("dictionary code {code} on `{path}` addresses a dictionary of {len} values")]
    #[diagnostic(code(validation::invariant))]
    DictionaryOutOfBounds {
        /// Registry field path of the dictionary column.
        path: FieldPath,
        /// The offending code.
        code: i64,
        /// Number of values the dictionary holds.
        len: usize,
    },

    /// A dictionary value was not a member of the declared enum domain.
    #[error("`{value}` on `{path}` is not a member of the declared enum")]
    #[diagnostic(code(validation::invariant))]
    EnumMember {
        /// Registry field path of the enum column.
        path: FieldPath,
        /// The offending value.
        value: String,
    },

    /// A supported-size bound was exceeded (blueprint §5.3 step 8).
    #[error("{what} bound exceeded: {actual} against a limit of {limit}")]
    #[diagnostic(code(runtime::resource_limit))]
    Envelope {
        /// Which bound.
        what: EnvelopeBound,
        /// The limit in force.
        limit: u64,
        /// What was requested or measured.
        actual: u64,
    },

    /// A reservation could not be grown before allocating (blueprint §14.3).
    #[error(transparent)]
    #[diagnostic(code(runtime::resource_limit))]
    Reservation(#[from] ReserveError),

    /// A cancellation checkpoint observed a cancelled token.
    #[error("cancelled")]
    #[diagnostic(code(runtime::cancelled))]
    Cancelled,

    /// Arrow refused an operation on well-formed input.
    #[error(transparent)]
    #[diagnostic(code(runtime::infrastructure))]
    Arrow(#[from] arrow_schema::ArrowError),

    /// A postcondition of this crate failed; a platform bug, never a user error.
    #[error("internal invariant: {0}")]
    #[diagnostic(code(internal::invariant))]
    Internal(String),
}

/// A snapshot frame could not be named (blueprint §5.3 step 7).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, miette::Diagnostic)]
#[non_exhaustive]
pub enum SnapshotError {
    /// Two semantic parents claimed the same role.
    #[error("parent role `{role}` appears more than once")]
    #[diagnostic(code(validation::invariant))]
    DuplicateParentRole {
        /// The repeated role.
        role: String,
    },

    /// Two members claimed the same port.
    #[error("member port `{port}` appears more than once")]
    #[diagnostic(code(validation::invariant))]
    DuplicateMemberPort {
        /// The repeated port.
        port: String,
    },

    /// A case snapshot did not name the model it overlays.
    #[error("a case snapshot requires exactly one parent with role `model`")]
    #[diagnostic(code(validation::invariant))]
    MissingModelParent,
}

/// An accounted allocation was refused (blueprint §14.3, ADR-0046).
///
/// The variant names the consumer and the configuration that bounds it, because a
/// `runtime::resource_limit` a reader cannot act on is a stack trace with better manners.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, miette::Diagnostic)]
#[non_exhaustive]
pub enum ReserveError {
    /// The budget cannot cover the request on top of what is already reserved.
    #[error(
        "`{owner}` requested {requested} bytes on top of {reserved} already reserved, \
         which exceeds {limit_hint}"
    )]
    #[diagnostic(code(runtime::resource_limit))]
    Exhausted {
        /// The consumer that asked, as it was named at `open`.
        owner: String,
        /// Bytes requested by this call.
        requested: usize,
        /// Bytes already reserved across the budget when the request was refused.
        reserved: usize,
        /// The configuration that bounds the budget, for the operator to change.
        limit_hint: String,
    },
}
