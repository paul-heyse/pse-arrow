// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Every failure the runtime returns, with its blueprint §23.2 class.
//!
//! The runtime sits above the catalog, so most of what can go wrong here has already been
//! classified there. [`RuntimeError::Catalog`] and [`RuntimeError::Reserve`] are
//! `#[diagnostic(transparent)]` for that reason: a reservation refused inside a
//! canonicalization is `runtime::resource_limit` whether the caller is a pass, a session
//! or a run, and re-deciding its class at each layer is how one event acquires three
//! names.

use pse_engine::EngineError;

/// A run-controller, session or budget failure (blueprint §23.2).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RuntimeError {
    /// A configured value cannot be honoured.
    ///
    /// §18.8 requires oversubscription to be "a configuration error, not a runtime
    /// surprise"; this is the shape that sentence takes.
    #[error("`{key}` is invalid: {reason}")]
    ConfigInvalid {
        /// The configuration key.
        key: String,
        /// Why the value was refused.
        reason: String,
    },

    /// A filesystem, process or store operation failed on well-formed input.
    #[error("{op} failed")]
    Infrastructure {
        /// What was being attempted.
        op: String,
        /// The underlying failure.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// A cancellation checkpoint observed a cancelled token.
    #[error("cancelled")]
    Cancelled,

    /// A catalog failure, keeping the class `pse-catalog` assigned it.
    #[error(transparent)]
    Catalog(#[from] EngineError),

    /// A reservation was refused, keeping the class `pse-ids` assigned it.
    #[error(transparent)]
    Reserve(#[from] pse_columnar::ReserveError),

    /// A postcondition of this crate failed: a platform bug, never a user error.
    #[error("internal invariant: {message}")]
    Internal {
        /// What was expected to hold.
        message: String,
    },
}

pse_diagnostics::impl_diagnostic! {
    RuntimeError,
    code(this) { match this {
            Self::ConfigInvalid { .. } => Some(pse_diagnostics::DiagnosticCode::ConfigInvalid),
            Self::Infrastructure { .. } => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
            Self::Cancelled => Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),
            Self::Internal { .. } => Some(pse_diagnostics::DiagnosticCode::InternalInvariant),
            _ => None,
        } },
    forward(this) { match this {
            Self::Catalog(value) => Some(value),
            Self::Reserve(value) => Some(value),
            _ => None,
        } },
    help(_this) { None },
    related(_this) { None },
    source(_this) { None }
}

#[cfg(test)]
mod tests {
    use pse_diagnostics::TypedDiagnostic;

    use super::*;

    /// The §23.2 class of an error, as a typed diagnostic identity.
    fn code_of(error: &RuntimeError) -> Option<pse_diagnostics::DiagnosticCode> {
        error.diagnostic_code()
    }

    #[test]
    fn every_variant_carries_its_blueprint_class() {
        let cases: Vec<(RuntimeError, Option<pse_diagnostics::DiagnosticCode>)> = vec![
            (
                RuntimeError::ConfigInvalid {
                    key: "datafusion.execution.target_partitions".to_owned(),
                    reason: "oversubscribed".to_owned(),
                },
                Some(pse_diagnostics::DiagnosticCode::ConfigInvalid),
            ),
            (
                RuntimeError::Infrastructure {
                    op: "create spill directory".to_owned(),
                    source: Box::new(std::io::Error::other("read-only filesystem")),
                },
                Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
            ),
            (
                RuntimeError::Cancelled,
                Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),
            ),
            (
                RuntimeError::Internal {
                    message: "postcondition".to_owned(),
                },
                Some(pse_diagnostics::DiagnosticCode::InternalInvariant),
            ),
        ];
        for (error, expected) in cases {
            assert_eq!(code_of(&error), expected, "{error}");
        }
    }

    #[test]
    fn the_from_variants_keep_the_class_the_lower_crate_assigned() {
        let catalog = RuntimeError::from(EngineError::Admission {
            path: "native.input".into(),
            reason: "incompatible declaration".into(),
        });
        assert_eq!(
            code_of(&catalog),
            Some(pse_diagnostics::DiagnosticCode::SchemaAdmission)
        );

        let reserve = RuntimeError::from(pse_columnar::ReserveError::Exhausted {
            owner: "canonicalize".to_owned(),
            requested: 1,
            reserved: 0,
            limit_hint: "native allocation attachment capacity=0".to_owned(),
        });
        assert_eq!(
            code_of(&reserve),
            Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit)
        );
    }
}

pse_columnar::impl_native_error!(RuntimeError);
