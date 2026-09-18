// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native provider, admission and execution failures with their diagnostic classes.

/// A native catalog, field-contract or execution failure.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[non_exhaustive]
pub enum CatalogError {
    /// A declared Arrow field or local value contract failed admission.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Relation(std::sync::Arc<pse_relations::RelationError>),
    /// A typed semantic diagnostic from the rule layer, without a dependency cycle.
    /// Shared ownership preserves its code, labels and related findings through engine
    /// `Shared` wrappers instead of stringifying a non-cloneable source.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Semantic(std::sync::Arc<dyn miette::Diagnostic + Send + Sync>),

    /// Every classified engine leaf, each retaining its own diagnostic class.
    #[error("{} platform failures", .errors.len())]
    Multiple {
        /// Complete related findings, in the engine's original traversal order.
        #[related]
        errors: Vec<CatalogError>,
    },
    /// An accounted consumer could not obtain the memory it needs.
    ///
    /// Recoverable and configuration-driven: `config_keys` names the settings an
    /// operator can change, because a `runtime::resource_limit` a reader cannot act on
    /// is a stack trace with better manners (blueprint §14.3, ADR-0046).
    #[error("`{consumer}` exhausted its budget: {detail}")]
    #[diagnostic(
        code(runtime::resource_limit),
        help("adjust one of: {}", config_keys.join(", "))
    )]
    ResourceLimit {
        /// The consumer that asked, as the engine or the platform named it.
        consumer: String,
        /// Configuration keys named by the engine or established by the bound runtime,
        /// in order of first appearance. The original engine detail remains unchanged.
        config_keys: Vec<String>,
        /// The engine's own message, kept verbatim.
        detail: String,
    },

    /// A cancellation checkpoint observed a cancelled token.
    #[error("cancelled")]
    #[diagnostic(code(runtime::cancelled))]
    Cancelled,

    /// An object store, filesystem or Arrow operation failed on well-formed input.
    #[error("{op} failed")]
    #[diagnostic(code(runtime::infrastructure))]
    Infrastructure {
        /// What was being attempted, in the imperative: `read Delta log`, `write Delta table`.
        op: String,
        /// The underlying failure.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// A plan, batch or import was refused by semantic admission (blueprint §4.4, §5.4).
    #[error("`{path}` was refused at admission: {reason}")]
    #[diagnostic(code(schema::admission))]
    Admission {
        /// Where in the plan or schema the refusal happened.
        path: String,
        /// Why.
        reason: String,
    },

    /// A configuration key was rejected by the typed configuration path.
    ///
    /// Typed, because `SessionConfig::set_str` panics on an invalid value instead of
    /// returning one of these; it is banned in `clippy.toml` for that reason.
    #[error("`{key}` is invalid: {reason}")]
    #[diagnostic(code(config::invalid))]
    ConfigInvalid {
        /// The configuration key.
        key: String,
        /// Why the value was refused.
        reason: String,
    },

    /// An authored query or model is wrong (blueprint §23.2, `user.model`).
    #[error("{message}")]
    #[diagnostic(code(user::model))]
    UserModel {
        /// What the user did that cannot work.
        message: String,
    },

    /// A generated kernel could not evaluate at the point it was given.
    #[error("{message}")]
    #[diagnostic(code(solve::evaluation_error))]
    EvaluationError {
        /// The engine's message.
        message: String,
    },

    /// A property was unsupported, ambiguous or unresolved in a kernel.
    #[error("{message}")]
    #[diagnostic(code(compile::property))]
    CompileProperty {
        /// The engine's message.
        message: String,
    },

    /// A `pse.canon.v2` failure, keeping the class `pse-ids` gave it.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Canon(#[from] pse_ids::CanonError),

    /// A reservation was refused, keeping the class `pse-ids` gave it.
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

impl From<pse_relations::RelationError> for CatalogError {
    fn from(error: pse_relations::RelationError) -> Self {
        Self::Relation(std::sync::Arc::new(error))
    }
}

#[cfg(test)]
mod tests {
    use miette::Diagnostic;

    use super::*;

    /// The §23.2 class of an error, as the diagnostic code renders it.
    fn code_of(error: &CatalogError) -> String {
        error
            .code()
            .map_or_else(|| "<none>".to_owned(), |code| code.to_string())
    }

    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "one row per §23.2 class; splitting the table hides that it is exhaustive"
    )]
    fn every_variant_carries_its_blueprint_class() {
        let cases: Vec<(CatalogError, &str)> = vec![
            (
                CatalogError::ResourceLimit {
                    consumer: "sort".to_owned(),
                    config_keys: vec!["datafusion.runtime.memory_limit".to_owned()],
                    detail: "out of memory".to_owned(),
                },
                "runtime::resource_limit",
            ),
            (CatalogError::Cancelled, "runtime::cancelled"),
            (
                CatalogError::Infrastructure {
                    op: "read Delta log".to_owned(),
                    source: Box::new(std::io::Error::other("no such object")),
                },
                "runtime::infrastructure",
            ),
            (
                CatalogError::Admission {
                    path: "0".to_owned(),
                    reason: "unknown extension".to_owned(),
                },
                "schema::admission",
            ),
            (
                CatalogError::ConfigInvalid {
                    key: "datafusion.execution.target_partitions".to_owned(),
                    reason: "oversubscribed".to_owned(),
                },
                "config::invalid",
            ),
            (
                CatalogError::UserModel {
                    message: "no such column".to_owned(),
                },
                "user::model",
            ),
            (
                CatalogError::EvaluationError {
                    message: "log of a negative number".to_owned(),
                },
                "solve::evaluation_error",
            ),
            (
                CatalogError::CompileProperty {
                    message: "unresolved method".to_owned(),
                },
                "compile::property",
            ),
            (
                CatalogError::Internal {
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
    fn the_from_variants_keep_the_class_pse_ids_assigned() {
        let reserve = CatalogError::from(pse_ids::ReserveError::Exhausted {
            owner: "canonicalize".to_owned(),
            requested: 1,
            reserved: 0,
            limit_hint: "FixedBudget limit_bytes=0".to_owned(),
        });
        assert_eq!(code_of(&reserve), "runtime::resource_limit");

        let canon = CatalogError::from(pse_ids::CanonError::Cancelled);
        assert_eq!(code_of(&canon), "runtime::cancelled");
    }
}
