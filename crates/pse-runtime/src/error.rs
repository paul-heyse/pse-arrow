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

use pse_catalog::CatalogError;

/// A run-controller, session or budget failure (blueprint §23.2).
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[non_exhaustive]
pub enum RuntimeError {
    /// A configured value cannot be honoured.
    ///
    /// §18.8 requires oversubscription to be "a configuration error, not a runtime
    /// surprise"; this is the shape that sentence takes.
    #[error("`{key}` is invalid: {reason}")]
    #[diagnostic(code(config::invalid))]
    ConfigInvalid {
        /// The configuration key.
        key: String,
        /// Why the value was refused.
        reason: String,
    },

    /// A filesystem, process or store operation failed on well-formed input.
    #[error("{op} failed")]
    #[diagnostic(code(runtime::infrastructure))]
    Infrastructure {
        /// What was being attempted.
        op: String,
        /// The underlying failure.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// A cancellation checkpoint observed a cancelled token.
    #[error("cancelled")]
    #[diagnostic(code(runtime::cancelled))]
    Cancelled,

    /// A catalog failure, keeping the class `pse-catalog` assigned it.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Catalog(#[from] CatalogError),

    /// A reservation was refused, keeping the class `pse-ids` assigned it.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Reserve(#[from] pse_ids::ReserveError),

    /// A postcondition of this crate failed: a platform bug, never a user error.
    #[error("internal invariant: {message}")]
    #[diagnostic(code(internal::invariant))]
    Internal {
        /// What was expected to hold.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use miette::Diagnostic;

    use super::*;

    /// The §23.2 class of an error, as the diagnostic code renders it.
    fn code_of(error: &RuntimeError) -> String {
        error
            .code()
            .map_or_else(|| "<none>".to_owned(), |code| code.to_string())
    }

    #[test]
    fn every_variant_carries_its_blueprint_class() {
        let cases: Vec<(RuntimeError, &str)> = vec![
            (
                RuntimeError::ConfigInvalid {
                    key: "datafusion.execution.target_partitions".to_owned(),
                    reason: "oversubscribed".to_owned(),
                },
                "config::invalid",
            ),
            (
                RuntimeError::Infrastructure {
                    op: "create spill directory".to_owned(),
                    source: Box::new(std::io::Error::other("read-only filesystem")),
                },
                "runtime::infrastructure",
            ),
            (RuntimeError::Cancelled, "runtime::cancelled"),
            (
                RuntimeError::Internal {
                    message: "postcondition".to_owned(),
                },
                "internal::invariant",
            ),
        ];
        for (error, expected) in cases {
            assert_eq!(code_of(&error), expected, "{error}");
        }
    }

    #[test]
    fn the_from_variants_keep_the_class_the_lower_crate_assigned() {
        let catalog = RuntimeError::from(CatalogError::Sealed);
        assert_eq!(code_of(&catalog), "schema::sealed");

        let reserve = RuntimeError::from(pse_ids::ReserveError::Exhausted {
            owner: "canonicalize".to_owned(),
            requested: 1,
            reserved: 0,
            limit_hint: "FixedBudget limit_bytes=0".to_owned(),
        });
        assert_eq!(code_of(&reserve), "runtime::resource_limit");
    }
}
