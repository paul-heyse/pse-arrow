// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! What a relation can be wrong about (blueprint §23.2, §4.4).
//!
//! Every variant names a *field or relation*, not just a condition, because a validator
//! returns violating keys plural: the first error is never the only one reported (§23.2).
//! The `schema.*` class is the same one the registry and the engine use, so a caller can
//! branch on the class without knowing which layer produced it.

use pse_ids::ContentHash;

/// A batch, field or metadata value that does not meet its declared contract.
#[derive(Debug, thiserror::Error)]
pub enum RelationError {
    /// Failed local obligations, with typed findings and explicit truncation state.
    #[error("{} local validation violations", report.violations)]
    LocalFindings {
        /// Actual native findings from the prepared validator.
        report: Box<crate::validate::ValidationReport>,
    },
    /// Native preparation or evaluation failure retaining the original causes.
    #[error(transparent)]
    Engine(#[from] pse_columnar::EngineError),
    /// Accounted construction exceeded an envelope, was cancelled or failed ownership.
    #[error(transparent)]
    Canon(#[from] pse_columnar::CanonError),

    /// The batch's `pse.contract.fingerprint` is not the one this view was generated for.
    #[error("relation `{relation}` expects fingerprint {expected} but the batch carries {actual}")]
    FingerprintMismatch {
        /// The relation, as `<namespace>.<name>@<version>`.
        relation: String,
        /// The fingerprint the generated view was built against.
        expected: ContentHash,
        /// The fingerprint the batch carries.
        actual: ContentHash,
    },

    /// The batch claims a relation the runtime registry does not declare.
    #[error("relation `{relation}` is not declared by this registry")]
    UnknownRegistry {
        /// The relation the batch claims, as its identity or qualified name.
        relation: String,
    },

    /// A field's Arrow storage is not the declared one.
    #[error("field `{field}` is stored as {actual}, and the contract declares {expected}")]
    Storage {
        /// The field path, for example `columns.0.item`.
        field: String,
        /// The declared storage.
        expected: String,
        /// The storage found.
        actual: String,
    },

    /// A field's `ARROW:extension:name` is missing, unknown, or not the declared one.
    #[error("field `{field}` claims extension `{name}`: {reason}")]
    ExtensionType {
        /// The field path.
        field: String,
        /// The extension name found, or the declared one when none was found.
        name: String,
        /// Why it was rejected.
        reason: String,
    },

    /// A field's `ARROW:extension:metadata` is missing, malformed, or of an unknown
    /// generation.
    #[error("field `{field}` carries extension metadata `{found}` for `{name}`: {reason}")]
    ExtensionMetadata {
        /// The field path.
        field: String,
        /// The extension name.
        name: String,
        /// The metadata string found.
        found: String,
        /// Why it was rejected.
        reason: String,
    },

    /// A field carries a metadata key nothing declares.
    #[error("field `{field}` carries the undeclared metadata key `{key}`")]
    UnknownMetadata {
        /// The field path, or the empty string for schema-level metadata.
        field: String,
        /// The offending key.
        key: String,
    },

    /// A non-nullable field holds a null, or a nullable one is declared non-null.
    #[error("field `{field}` is declared {declared} and the batch is {actual}")]
    Nullability {
        /// The field path.
        field: String,
        /// `non-null` or `nullable`.
        declared: &'static str,
        /// What the batch actually is.
        actual: String,
    },

    /// A string value is not a member of the declared enumeration.
    #[error("field `{field}` holds `{value}`, which is not a member of `{enumeration}`")]
    EnumMember {
        /// The field path.
        field: String,
        /// The declared enumeration.
        enumeration: String,
        /// The value found.
        value: String,
    },

    /// An ordinal reference points outside its target relation.
    #[error("field `{field}` references ordinal {ordinal} of `{target}`, which has {rows} rows")]
    OrdinalRange {
        /// The field path.
        field: String,
        /// The ordinal found.
        ordinal: i64,
        /// The target relation.
        target: String,
        /// How many rows the target has.
        rows: i64,
    },

    /// A declared relation contract or metadata value does not match the offered data.
    #[error("relation `{relation}` violates its contract: {reason}")]
    Contract {
        /// The relation or field being admitted.
        relation: String,
        /// The specific mismatch.
        reason: String,
    },

    /// A visible row value violates its declared logical meaning.
    #[error("field `{field}` in row {row}: {reason}")]
    Value {
        /// The complete field path.
        field: String,
        /// The containing row.
        row: usize,
        /// The violated invariant.
        reason: String,
    },

    /// All violations found at an admission boundary.
    #[error("relation admission found {} violations", .errors.len())]
    Validation {
        /// Independently actionable findings.
        errors: Vec<RelationError>,
    },

    /// The bound registry declaration could not produce its contract.
    #[error(transparent)]
    Schema(#[from] pse_schema::SchemaError),

    /// Arrow itself rejected the operation.
    #[error(transparent)]
    Arrow(#[from] arrow_schema::ArrowError),
}

pse_diagnostics::impl_diagnostic! {
    RelationError,
    code(this) { match this {
            Self::LocalFindings { .. } | Self::Value { .. } => Some(pse_diagnostics::DiagnosticCode::ValidationInvariant),
            Self::FingerprintMismatch { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaFingerprintMismatch),
            Self::UnknownRegistry { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaUnknownRegistry),
            Self::Storage { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaStorage),
            Self::ExtensionType { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaExtensionType),
            Self::ExtensionMetadata { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaExtensionMetadata),
            Self::UnknownMetadata { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaUnknownMetadata),
            Self::Nullability { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaNullability),
            Self::EnumMember { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaEnumMember),
            Self::OrdinalRange { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaOrdinalRange),
            Self::Contract { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaContractMismatch),

            Self::Validation { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaAdmission),
            Self::Arrow(..) => Some(pse_diagnostics::DiagnosticCode::SchemaArrow),
            _ => None,
        } },
    forward(this) { match this {
            Self::Engine(error) => Some(error),
            Self::Canon(value) => Some(value),
            Self::Schema(value) => Some(value),
            _ => None,
        } },
    help(this) { match this {
            Self::FingerprintMismatch { .. } => Some(Box::new("the artifact was written under a different registry; migrate it or recompile")),
            Self::UnknownRegistry { .. } => Some(Box::new("the artifact predates a relation rename or removal; read it through its own registry")),
            Self::Storage { .. } => Some(Box::new("an unknown layout is rejected, never decoded opportunistically (blueprint §5.3 step 2)")),
            Self::ExtensionType { .. } => Some(Box::new("registration is passive; the validator is what admits a value (blueprint §4.4)")),
            Self::ExtensionMetadata { .. } => Some(Box::new("the shapes are exactly `{\"v\":1}`, `{\"v\":1,\"enum_id\":…}` and `{\"v\":1,\"target_relation_id\":…}` (blueprint §4.4)")),
            Self::UnknownMetadata { .. } => Some(Box::new("`SERDE_ARROW:*` and other unregistered keys are errors, not values to discard silently (blueprint §4.3)")),
            Self::Nullability { .. } => Some(Box::new("required visible values must be non-null; null parents mask their payload (blueprint §5.3 step 2)")),
            Self::EnumMember { .. } => Some(Box::new("an enumeration has a declared member domain; add the member to the registry or correct the value")),
            Self::OrdinalRange { .. } => Some(Box::new("an ordinal is artifact-local; only bundle admission can check that it is in range (blueprint §4.4)")),
            _ => None,
        } },
    related(this) { match this {
            Self::Validation { errors, .. } => Some(Box::new(errors.iter().map(|value| -> &dyn pse_diagnostics::TypedDiagnostic { value }))),
            _ => None,
        } },
    source(_this) { None }
}

pse_columnar::impl_native_error!(RelationError);

impl From<pse_model::ModelError> for RelationError {
    fn from(error: pse_model::ModelError) -> Self {
        match error {
            pse_model::ModelError::Malformed(message) => crate::columnar::mismatch(&message),
            pse_model::ModelError::EnumMember {
                field,
                enumeration,
                value,
            } => Self::EnumMember {
                field,
                enumeration,
                value,
            },
        }
    }
}
