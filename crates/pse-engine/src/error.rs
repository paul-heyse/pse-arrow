// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native provider, admission and execution failures with their diagnostic classes.

/// A native catalog, field-contract or execution failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EngineError {
    /// Original engine error tree and its borrowed classification.
    #[error(transparent)]
    Engine(#[from] pse_columnar::EngineError),
    /// A declared Arrow field or local value contract failed admission.
    #[error(transparent)]
    Relation(std::sync::Arc<pse_relations::RelationError>),
    /// A typed semantic diagnostic from the rule layer, without a dependency cycle.
    /// Shared ownership preserves its code, labels and related findings through engine
    /// `Shared` wrappers instead of stringifying a non-cloneable source.
    #[error(transparent)]
    Semantic(std::sync::Arc<dyn pse_diagnostics::TypedDiagnostic + Send + Sync>),

    /// Every classified engine leaf, each retaining its own diagnostic class.
    #[error("{} platform failures", .errors.len())]
    Multiple {
        /// Complete related findings, in the engine's original traversal order.
        errors: Vec<EngineError>,
    },
    /// An accounted consumer could not obtain the memory it needs.
    ///
    /// Recoverable and configuration-driven: `config_keys` names the settings an
    /// operator can change, because a `runtime::resource_limit` a reader cannot act on
    /// is a stack trace with better manners (blueprint §14.3, ADR-0046).
    #[error("`{consumer}` exhausted its budget: {detail}")]
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
    Cancelled,

    /// An object store, filesystem or Arrow operation failed on well-formed input.
    #[error("{op} failed")]
    Infrastructure {
        /// What was being attempted, in the imperative: `read Delta log`, `write Delta table`.
        op: String,
        /// The underlying failure.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// A plan, batch or import was refused by semantic admission (blueprint §4.4, §5.4).
    #[error("`{path}` was refused at admission: {reason}")]
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
    ConfigInvalid {
        /// The configuration key.
        key: String,
        /// Why the value was refused.
        reason: String,
    },

    /// An authored query or model is wrong (blueprint §23.2, `user.model`).
    #[error("{message}")]
    UserModel {
        /// What the user did that cannot work.
        message: String,
    },

    /// A generated kernel could not evaluate at the point it was given.
    #[error("{message}")]
    EvaluationError {
        /// The engine's message.
        message: String,
    },

    /// A property was unsupported, ambiguous or unresolved in a kernel.
    #[error("{message}")]
    CompileProperty {
        /// The engine's message.
        message: String,
    },

    /// A `pse.canon.v2` failure, keeping the class `pse-ids` gave it.
    #[error(transparent)]
    Canon(#[from] pse_columnar::CanonError),

    /// A reservation was refused, keeping the class `pse-ids` gave it.
    #[error(transparent)]
    Reserve(#[from] pse_columnar::ReserveError),

    /// A postcondition of this crate failed: a platform bug, never a user error.
    #[error("internal invariant: {message}")]
    Internal {
        /// What was expected to hold.
        message: String,
    },
}

impl From<datafusion::common::DataFusionError> for EngineError {
    fn from(error: datafusion::common::DataFusionError) -> Self {
        Self::Engine(pse_columnar::classify(
            error,
            pse_columnar::PlanOrigin::RuleCompiler,
        ))
    }
}

impl From<pse_relations::RelationError> for EngineError {
    fn from(error: pse_relations::RelationError) -> Self {
        Self::Relation(std::sync::Arc::new(error))
    }
}

pse_diagnostics::impl_diagnostic! {
    EngineError,
    code(this) { match this {
            Self::ResourceLimit { .. } => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
            Self::Cancelled => Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),
            Self::Infrastructure { .. } => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
            Self::Admission { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaAdmission),
            Self::ConfigInvalid { .. } => Some(pse_diagnostics::DiagnosticCode::ConfigInvalid),
            Self::UserModel { .. } => Some(pse_diagnostics::DiagnosticCode::UserModel),
            Self::EvaluationError { .. } => Some(pse_diagnostics::DiagnosticCode::SolveEvaluationError),
            Self::CompileProperty { .. } => Some(pse_diagnostics::DiagnosticCode::CompileProperty),
            Self::Internal { .. } => Some(pse_diagnostics::DiagnosticCode::InternalInvariant),
            _ => None,
        } },
    forward(this) { match this {
            Self::Engine(value) => Some(value),
            Self::Relation(value) => Some(value.as_ref()),
            Self::Semantic(value) => Some(value.as_ref()),
            Self::Canon(value) => Some(value),
            Self::Reserve(value) => Some(value),
            _ => None,
        } },
    help(this) { match this {
            Self::ResourceLimit { config_keys, .. } => Some(Box::new(format!("adjust one of: {}", config_keys.join(", ")))),
            _ => None,
        } },
    related(this) { match this {
            Self::Multiple { errors, .. } => Some(Box::new(errors.iter().map(|value| -> &dyn pse_diagnostics::TypedDiagnostic { value }))),
            _ => None,
        } },
    source(_this) { None }
}

#[cfg(test)]
mod tests {
    use pse_diagnostics::TypedDiagnostic;

    use super::*;

    /// The §23.2 class of an error, as a typed diagnostic identity.
    fn code_of(error: &EngineError) -> Option<pse_diagnostics::DiagnosticCode> {
        error.diagnostic_code()
    }

    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "one row per §23.2 class; splitting the table hides that it is exhaustive"
    )]
    fn every_variant_carries_its_blueprint_class() {
        let cases: Vec<(EngineError, Option<pse_diagnostics::DiagnosticCode>)> = vec![
            (
                EngineError::ResourceLimit {
                    consumer: "sort".to_owned(),
                    config_keys: vec!["datafusion.runtime.memory_limit".to_owned()],
                    detail: "out of memory".to_owned(),
                },
                Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
            ),
            (
                EngineError::Cancelled,
                Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),
            ),
            (
                EngineError::Infrastructure {
                    op: "read Delta log".to_owned(),
                    source: Box::new(std::io::Error::other("no such object")),
                },
                Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
            ),
            (
                EngineError::Admission {
                    path: "0".to_owned(),
                    reason: "unknown extension".to_owned(),
                },
                Some(pse_diagnostics::DiagnosticCode::SchemaAdmission),
            ),
            (
                EngineError::ConfigInvalid {
                    key: "datafusion.execution.target_partitions".to_owned(),
                    reason: "oversubscribed".to_owned(),
                },
                Some(pse_diagnostics::DiagnosticCode::ConfigInvalid),
            ),
            (
                EngineError::UserModel {
                    message: "no such column".to_owned(),
                },
                Some(pse_diagnostics::DiagnosticCode::UserModel),
            ),
            (
                EngineError::EvaluationError {
                    message: "log of a negative number".to_owned(),
                },
                Some(pse_diagnostics::DiagnosticCode::SolveEvaluationError),
            ),
            (
                EngineError::CompileProperty {
                    message: "unresolved method".to_owned(),
                },
                Some(pse_diagnostics::DiagnosticCode::CompileProperty),
            ),
            (
                EngineError::Internal {
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
    fn the_from_variants_keep_the_class_pse_ids_assigned() {
        let reserve = EngineError::from(pse_columnar::ReserveError::Exhausted {
            owner: "canonicalize".to_owned(),
            requested: 1,
            reserved: 0,
            limit_hint: "native allocation attachment capacity=0".to_owned(),
        });
        assert_eq!(
            code_of(&reserve),
            Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit)
        );

        let canon = EngineError::from(pse_columnar::CanonError::Cancelled);
        assert_eq!(
            code_of(&canon),
            Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled)
        );
    }
}

pse_columnar::impl_native_error!(EngineError);
