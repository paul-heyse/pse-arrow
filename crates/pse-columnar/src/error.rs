// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Columnar admission failures.
use crate::contract::FieldPath;
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
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CanonError {
    /// An Arrow layout outside the supported matrix (blueprint §5.3 step 2).
    #[error("field `{path}` has layout `{what}`, which `pse.canon.v2` does not support")]
    UnsupportedLayout {
        /// Registry field path of the offending field.
        path: FieldPath,
        /// Terse description of the offending layout.
        what: String,
    },

    /// A batch does not match the declared contract schema.
    #[error("`{path}` is `{actual}` but the contract declares `{expected}`")]
    ContractMismatch {
        /// Registry field path of the offending field, or the empty path for the schema.
        path: FieldPath,
        /// What the contract declares.
        expected: String,
        /// What the batch offered.
        actual: String,
    },

    /// A primary-key column that cannot order a relation.
    #[error("primary-key column `{column}` is invalid: {reason}")]
    InvalidKey {
        /// The column named by the contract.
        column: String,
        /// Why it cannot be a key.
        reason: String,
    },

    /// A required key column held a null (blueprint §5.3 step 1).
    #[error("primary-key column `{column}` is null in row {row}")]
    NullKey {
        /// The key column.
        column: String,
        /// Zero-based row index in the concatenated relation.
        row: usize,
    },

    /// Two rows shared a primary key (blueprint §5.3 step 1).
    #[error("rows {first} and {second} share a primary key")]
    DuplicateKey {
        /// Zero-based index of the first row of the pair.
        first: usize,
        /// Zero-based index of the second row of the pair.
        second: usize,
    },

    /// A dictionary code addressed a value outside its dictionary.
    #[error("dictionary code {code} on `{path}` addresses a dictionary of {len} values")]
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
    EnumMember {
        /// Registry field path of the enum column.
        path: FieldPath,
        /// The offending value.
        value: String,
    },

    /// A supported-size bound was exceeded (blueprint §5.3 step 8).
    #[error("{what} bound exceeded: {actual} against a limit of {limit}")]
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
    Reservation(#[from] ReserveError),

    /// Original native allocation failure, retaining its concrete source.
    #[error(transparent)]
    NativeResource(#[from] datafusion_common::DataFusionError),

    /// A cancellation checkpoint observed a cancelled token.
    #[error("cancelled")]
    Cancelled,

    /// Arrow refused an operation on well-formed input.
    #[error(transparent)]
    Arrow(#[from] arrow_schema::ArrowError),

    /// A postcondition of this crate failed; a platform bug, never a user error.
    #[error("internal invariant: {0}")]
    Internal(String),
}

/// An accounted allocation was refused (blueprint §14.3, ADR-0046).
///
/// The variant names the consumer and the configuration that bounds it, because a
/// `runtime::resource_limit` a reader cannot act on is a stack trace with better manners.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ReserveError {
    /// The budget cannot cover the request on top of what is already reserved.
    #[error(
        "`{owner}` requested {requested} bytes on top of {reserved} already reserved, \
         which exceeds {limit_hint}"
    )]
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

pse_diagnostics::impl_diagnostic! {
    CanonError,
    code(this) { match this {
            Self::UnsupportedLayout { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaUnsupportedLayout),
            Self::ContractMismatch { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaContractMismatch),
            Self::InvalidKey { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaInvalidKey),
            Self::NullKey { .. } | Self::DuplicateKey { .. } | Self::DictionaryOutOfBounds { .. } | Self::EnumMember { .. } => Some(pse_diagnostics::DiagnosticCode::ValidationInvariant),



            Self::Envelope { .. } | Self::Reservation(..) | Self::NativeResource(..) => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),

            Self::Cancelled => Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),
            Self::Arrow(..) => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
            Self::Internal(..) => Some(pse_diagnostics::DiagnosticCode::InternalInvariant),
            _ => None,
        } },
    forward(_this) { None },
    help(_this) { None },
    related(_this) { None },
    source(_this) { None }
}

pse_diagnostics::impl_diagnostic! {
    ReserveError,
    code(this) { match this {
            Self::Exhausted { .. } => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
            _ => None,
        } },
    forward(_this) { None },
    help(_this) { None },
    related(_this) { None },
    source(_this) { None }
}

crate::impl_native_error!(CanonError);
crate::impl_native_error!(ReserveError);
