// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pure semantics for registry-generated generic modeling declarations.
//! Native math, incremental storage, data ingestion and effects belong to consumers.
pub mod analysis;
pub mod annotation;
pub mod check;
pub mod continuous;
pub mod data;
pub mod expression;
mod extent;
pub mod external;
pub mod specialize;
pub mod types;

pub use check::{CheckedPackage, FiniteReduction, Function, check};
pub use pse_authoring::language::{Declaration, Selected};
use pse_ids::SemanticId;
pub use specialize::{Bindings, Limits, SpecializedModel, specialize};
pub use types::{Type, TypeContext};

/// An attributable checking or specialization failure.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ModelingError {
    /// The declaration owner retains its source witness when admission fails.
    #[error("{cause}")]
    Located {
        /// Original document and complete declaration byte range.
        span: pse_authoring::SourceSpan,
        /// Authored declaration name.
        name: String,
        /// Unchanged classified semantic failure.
        #[source]
        cause: Box<ModelingError>,
    },
    /// An invalid declaration or use.
    #[error("modeling declaration {declaration}: {message}")]
    Contract {
        /// Stable semantic source identity.
        declaration: SemanticId,
        /// Actual violated condition.
        message: String,
    },
    /// A parsed and checked construct needs a later execution capability.
    #[error("modeling declaration {declaration} requires {capability}")]
    Unsupported {
        /// Source identity.
        declaration: SemanticId,
        /// Missing capability, never silently ignored.
        capability: String,
    },
    /// The caller withdrew this computation; never a cached semantic diagnostic.
    #[error("modeling cancelled")]
    Cancelled,
    /// A limit refused expansion before publication.
    #[error("modeling budget: {0}")]
    Budget(String),
}
pse_diagnostics::impl_diagnostic! {
    ModelingError, code(this) { Some(match this {
        ModelingError::Located { .. } => return None,
        ModelingError::Contract { .. } => pse_diagnostics::DiagnosticCode::ValidationInvariant,
        ModelingError::Unsupported { .. } => pse_diagnostics::DiagnosticCode::CapabilityBackend,
        ModelingError::Cancelled => pse_diagnostics::DiagnosticCode::RuntimeCancelled,
        ModelingError::Budget(_) => pse_diagnostics::DiagnosticCode::RuntimeResourceLimit,
    }) },
    forward(this) { match this { ModelingError::Located { cause, .. } => Some(cause.as_ref()), _ => None } }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
impl ModelingError {
    pub(crate) fn located(self, rows: &[Declaration]) -> Self {
        let id = match &self {
            Self::Contract { declaration, .. } | Self::Unsupported { declaration, .. } => {
                *declaration
            }
            _ => return self,
        };
        let mut matches = rows.iter().filter(|row| row.declaration_id == id);
        let Some(row) = matches.next() else {
            return self;
        };
        // Ambiguous IDs and malformed offsets cannot supply a trustworthy witness.
        if matches.next().is_some() || row.source_end < row.source_start {
            return self;
        }
        let (Ok(start), Ok(end)) = (
            u32::try_from(row.source_start),
            u32::try_from(row.source_end),
        ) else {
            return self;
        };
        Self::Located {
            span: pse_authoring::SourceSpan::new(row.document_id, start, end),
            name: row.name.clone(),
            cause: Box::new(self),
        }
    }
    /// Stable failure classification and source identity, independent of display text.
    pub fn boundary_diagnostic(&self) -> pse_model::diagnostic::BoundaryDiagnostic {
        use pse_model::diagnostic::{BoundaryClass as Class, BoundaryDiagnostic, Observation};
        let (class, rule, source, detail) = match self {
            Self::Located { span, name, cause } => {
                let mut diagnostic = cause.boundary_diagnostic();
                for source in &diagnostic.sources {
                    diagnostic
                        .locations
                        .push(pse_model::diagnostic::SourceLocation {
                            source: *source,
                            path: format!("document/{}", span.document_id),
                            name: Some(name.clone()),
                            start: Some(span.start),
                            end: Some(span.end),
                        });
                }
                return diagnostic;
            }
            Self::Contract {
                declaration,
                message,
            } => (
                Class::InvalidModel,
                "modeling.contract",
                Some(*declaration),
                Some(message),
            ),
            Self::Unsupported {
                declaration,
                capability,
            } => (
                Class::Unsupported,
                "modeling.capability",
                Some(*declaration),
                Some(capability),
            ),
            Self::Cancelled => (Class::Cancelled, "modeling.cancelled", None, None),
            Self::Budget(reason) => (Class::ResourceLimit, "modeling.budget", None, Some(reason)),
        };
        let mut diagnostic = BoundaryDiagnostic::new(class, "modeling", source, rule);
        if let Some(detail) = detail {
            diagnostic
                .observations
                .insert("detail".into(), Observation::Text(detail.clone()));
        }
        diagnostic
    }
}
pub(crate) type Result<T> = std::result::Result<T, ModelingError>;
pub(crate) fn invalid(id: SemanticId, message: impl Into<String>) -> ModelingError {
    ModelingError::Contract {
        declaration: id,
        message: message.into(),
    }
}

#[cfg(test)]
mod kernel_types;

#[cfg(test)]
mod kernel_continuous;
#[cfg(test)]
mod kernel_specialization;
