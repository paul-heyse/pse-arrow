// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Columnar admission failures.
use crate::contract::FieldPath;
/// Which native columnar allocation bound was exceeded (blueprint §14.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EnvelopeBound {
    /// Rows in the complete relation.
    Rows,
    /// Bytes of normalized value, offset and validity buffers.
    Bytes,
}

impl EnvelopeBound {
    /// The stable lowercase spelling used in messages and in fixtures.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rows => "rows",
            Self::Bytes => "bytes",
        }
    }
}

impl std::fmt::Display for EnvelopeBound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Native columnar data could not be admitted, allocated or represented.
///
/// The variants split along the §23.2 classes deliberately: a contract problem is a
/// `schema::*` failure the author can fix, a bound or a reservation is
/// `runtime::resource_limit`, and a broken postcondition is `internal::invariant` and
/// never dressed up as either of the first two.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CanonError {
    /// An Arrow layout outside the native operation's supported matrix.
    #[error("field `{path}` has unsupported native columnar layout `{what}`")]
    UnsupportedLayout {
        /// Registry field path of the offending field.
        path: FieldPath,
        /// Terse description of the offending layout.
        what: String,
    },

    /// A supported-size bound was exceeded (blueprint §14.3).
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
