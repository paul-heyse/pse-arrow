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
pub mod logic;
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
    /// A declared variable domain the requested analysis cannot admit (ADR-0103).
    #[error(
        "variable {path} in {}: {} for {}",
        .domain.as_str(),
        .reason.as_str(),
        .analysis.as_str()
    )]
    Domain {
        /// Instantiated variable identity.
        variable: SemanticId,
        /// Declaring source identity.
        declaration: SemanticId,
        /// Authored instance path of the variable.
        path: String,
        /// Declared domain.
        domain: pse_model::generated::enums::ModelingVariableDomain,
        /// Analysis whose admission refused the variable.
        analysis: DomainAnalysis,
        /// Violated admission rule.
        reason: DomainRefusal,
    },
    /// A constraint-form or disjunction lowering the case cannot admit (ADR-0104).
    #[error(
        "{} realization of {form}: {} ({subject})",
        .realization.as_str(),
        .reason.as_str()
    )]
    Realization {
        /// Authored form or disjunction declaration.
        declaration: SemanticId,
        /// Authored path of the form.
        form: String,
        /// The row or variable the refusal names.
        subject: String,
        /// Declared realization.
        realization: pse_model::generated::enums::ModelingRealizationPolicy,
        /// Violated precondition.
        reason: RealizationRefusal,
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
        ModelingError::Domain { reason, .. } => if reason.is_unsupported() {
            pse_diagnostics::DiagnosticCode::CapabilityBackend
        } else {
            pse_diagnostics::DiagnosticCode::ValidationInvariant
        },
        ModelingError::Realization { .. } => pse_diagnostics::DiagnosticCode::CapabilityBackend,
        ModelingError::Cancelled => pse_diagnostics::DiagnosticCode::RuntimeCancelled,
        ModelingError::Budget(_) => pse_diagnostics::DiagnosticCode::RuntimeResourceLimit,
    }) },
    forward(this) { match this { ModelingError::Located { cause, .. } => Some(cause.as_ref()), _ => None } }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
impl ModelingError {
    pub(crate) fn located(self, rows: &[Declaration]) -> Self {
        let id = match &self {
            Self::Contract { declaration, .. }
            | Self::Unsupported { declaration, .. }
            | Self::Domain { declaration, .. }
            | Self::Realization { declaration, .. } => *declaration,
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
            Self::Domain {
                variable,
                declaration,
                path,
                domain,
                analysis,
                reason,
            } => {
                let class = if reason.is_unsupported() {
                    Class::Unsupported
                } else {
                    Class::InvalidModel
                };
                let mut diagnostic = BoundaryDiagnostic::new(
                    class,
                    "modeling",
                    [*declaration, *variable],
                    "modeling.domain",
                );
                for (name, value) in [
                    ("variable", path.as_str()),
                    ("domain", domain.as_str()),
                    ("analysis", analysis.as_str()),
                    ("reason", reason.as_str()),
                ] {
                    diagnostic
                        .observations
                        .insert(name.into(), Observation::Text(value.into()));
                }
                return diagnostic;
            }
            Self::Realization {
                declaration,
                form,
                subject,
                realization,
                reason,
            } => {
                let mut diagnostic = BoundaryDiagnostic::new(
                    Class::Unsupported,
                    "modeling",
                    [*declaration],
                    "modeling.realization",
                );
                for (name, value) in [
                    ("form", form.as_str()),
                    ("subject", subject.as_str()),
                    ("realization", realization.as_str()),
                    ("reason", reason.as_str()),
                ] {
                    diagnostic
                        .observations
                        .insert(name.into(), Observation::Text(value.into()));
                }
                return diagnostic;
            }
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
/// The analysis whose admission refused a declared discrete domain (ADR-0103 item 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomainAnalysis {
    /// Case preparation, which every analysis passes: typing and finite bounds.
    Preparation,
    /// A square root solve; discrete variables are admitted only when fixed.
    Root,
    /// Initialization; discrete variables are admitted only when fixed.
    Initialization,
    /// Parameter estimation; refused unless the case fixes every discrete variable.
    Fitting,
    /// Integrated dynamics; discrete variables are fixed per segment.
    IntegratedDynamics,
}
impl DomainAnalysis {
    /// Stable diagnostic spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Preparation => "preparation",
            Self::Root => "root",
            Self::Initialization => "initialization",
            Self::Fitting => "fitting",
            Self::IntegratedDynamics => "integrated_dynamics",
        }
    }
}
/// The admission rule a declared discrete domain violated (ADR-0103 items 3, 4 and 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomainRefusal {
    /// Integer and binary require a dimensionless count or indicator quantity kind.
    QuantityKind,
    /// Integer and semi domains require finite case bounds.
    InfiniteBound,
    /// The bounds leave no admissible value: binary outside [0, 1], an empty integer
    /// range, or a semi interval that is not positive.
    EmptyDomain,
    /// The analysis admits a discrete variable only when the case fixes it.
    Free,
}
impl DomainRefusal {
    /// Stable diagnostic spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::QuantityKind => "requires a dimensionless count or indicator quantity",
            Self::InfiniteBound => "requires finite case bounds",
            Self::EmptyDomain => "bounds admit no value of the domain",
            Self::Free => "must be fixed by the case",
        }
    }
    /// A missing capability of the analysis, as opposed to an invalid model.
    pub const fn is_unsupported(self) -> bool {
        matches!(self, Self::InfiniteBound | Self::Free)
    }
}
/// The precondition a realization found unmet at preparation (ADR-0104 §3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealizationRefusal {
    /// Hull and linear lowerings need a finite case bound on the named variable.
    InfiniteBound,
    /// A derived big-M needs the named row's interval to be FBBT-complete.
    IncompleteInterval,
    /// A derived big-M needs the named row's interval to be finite over the case box.
    UnboundedInterval,
    /// A nonlinear disjunct row needs the declared epsilon of `hull(epsilon)`.
    Nonlinear,
}
impl RealizationRefusal {
    /// Stable diagnostic spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InfiniteBound => "requires finite case bounds",
            Self::IncompleteInterval => "row interval is not FBBT-complete",
            Self::UnboundedInterval => "row interval is not finite over the case box",
            Self::Nonlinear => "a nonlinear disjunct row requires hull(epsilon)",
        }
    }
}
pub(crate) type Result<T> = std::result::Result<T, ModelingError>;
/// The unique registry quantity typing binary decisions and convex weights (ADR-0103/0104).
pub(crate) fn indicator_type(
    registry: &pse_quantity::QuantityRegistry,
    at: SemanticId,
) -> Result<Type> {
    let mut found = registry.quantity_types().filter(|t| {
        t.key.basis.is_none()
            && t.key.reference_state.is_none()
            && t.key.shape.is_empty()
            && t.key.subject_kind.is_none()
            && registry.kind(t.key.kind).is_ok_and(|k| {
                k.category == Some(pse_quantity::QuantityKindCategory::Indicator)
            })
    });
    match (found.next(), found.next()) {
        (Some(ty), None) => Ok(Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
            ty.id,
        ))),
        _ => Err(invalid(
            at,
            "constraint forms need exactly one declared indicator quantity",
        )),
    }
}
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
