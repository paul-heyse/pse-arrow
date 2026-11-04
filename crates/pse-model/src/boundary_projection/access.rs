// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed access at the actual owner, including fields private to its Rust module.

/// A record's actual field tuple. Stable tags/names detect stale adapter layouts.
pub trait ProjectionRecord: Sized {
    /// Interpretation version co-declared by the owner.
    const VERSION: u32;
    /// Tuple layout, with explicit stable tags rather than inferred wire numbers.
    const FIELD_LAYOUT: &'static [(u32, &'static str)];
    /// Actual declared Rust field types.
    type Fields;
    /// Consume the original record without a generic intermediate copy.
    fn into_projection_fields(self) -> Self::Fields;
    /// Construct the actual native record at its defining module.
    /// # Errors
    /// The original owner refuses malformed invariant-bearing field components.
    fn from_projection_fields(fields: Self::Fields) -> Result<Self, AdapterError>;
}

/// An actual alternative selected by its explicitly declared stable tag.
pub trait ProjectionAlternative<const TAG: u32>: Sized {
    /// Interpretation version co-declared by the owner.
    const VERSION: u32;
    /// Actual payload tuple layout.
    const FIELD_LAYOUT: &'static [(u32, &'static str)];
    /// Actual native payload field types; unit alternatives use `()`.
    type Fields;
    /// Extract this alternative, returning other alternatives unchanged.
    fn into_projection_alternative(self) -> Result<Self::Fields, Self>;
    /// Construct this actual alternative at its defining module.
    fn from_projection_alternative(fields: Self::Fields) -> Self;
}

/// Native adoption failure category, separate from contextual scientific admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterFailure {
    /// A generated adapter no longer matches its actual source owner.
    LayoutMismatch,
    /// A binary field cannot be represented by this native interpretation.
    UnknownField,
    /// An absent or unsupported alternative cannot become a native default.
    UnknownAlternative,
    /// Required exact data is absent or malformed.
    InvalidValue,
    /// A repeated entry would silently overwrite another native key.
    DuplicateKey,
}

/// Original structured failure while crossing a selected typed projection.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AdapterError {
    /// Mechanical native adoption cannot preserve the supplied wire meaning.
    #[error("boundary adoption {kind:?} at `{path}`: {reason}")]
    Refused {
        /// Complete field/alternative path.
        path: String,
        /// Typed mechanical failure category.
        kind: AdapterFailure,
        /// Original refusal detail.
        reason: String,
    },
    /// The actual owner's shared local validation rejected its value.
    #[error(transparent)]
    Validation(#[from] super::ValidationError),
}

impl AdapterError {
    /// Refuse adoption without discarding an unsupported value.
    pub fn refused(
        path: impl Into<String>,
        kind: AdapterFailure,
        reason: impl Into<String>,
    ) -> Self {
        Self::Refused {
            path: path.into(),
            kind,
            reason: reason.into(),
        }
    }
    /// Attach the containing field path to an owner-local construction refusal.
    pub fn at(self, parent: &str) -> Self {
        match self {
            Self::Refused { path, kind, reason } => Self::Refused {
                path: super::field_path(parent, &path),
                kind,
                reason,
            },
            Self::Validation(super::ValidationError::Rule { path, rule, reason }) => {
                Self::Validation(super::ValidationError::Rule {
                    path: super::field_path(parent, &path),
                    rule,
                    reason,
                })
            }
        }
    }
}

pse_diagnostics::impl_diagnostic! {
    AdapterError,
    code(this) { match this {
        Self::Refused { kind: AdapterFailure::LayoutMismatch, .. } => Some(pse_diagnostics::DiagnosticCode::WorkflowInternal),
        Self::Refused { .. } => Some(pse_diagnostics::DiagnosticCode::WorkflowInput),
        Self::Validation(_) => None,
    } },
    forward(this) { match this { Self::Validation(error) => Some(error), _ => None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None },
    facts(this) {
        let mut facts = pse_diagnostics::DiagnosticFacts::default();
        if let Self::Refused { path, kind, reason } = this {
            facts.observe("field_path", pse_diagnostics::DiagnosticObservation::Text(path.clone()));
            facts.observe("adoption_category", pse_diagnostics::DiagnosticObservation::Text(format!("{kind:?}")));
            facts.observe("detail", pse_diagnostics::DiagnosticObservation::Text(reason.clone()));
        }
        facts
    }
}
impl crate::diagnostic::DiagnosticProjection for AdapterError {}
