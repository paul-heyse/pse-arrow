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
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum RelationError {
    /// The batch's `pse.contract.fingerprint` is not the one this view was generated for.
    #[error("relation `{relation}` expects fingerprint {expected} but the batch carries {actual}")]
    #[diagnostic(
        code(schema::fingerprint_mismatch),
        help("the artifact was written under a different registry; migrate it or recompile")
    )]
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
    #[diagnostic(
        code(schema::unknown_registry),
        help(
            "the artifact predates a relation rename or removal; read it through its own registry"
        )
    )]
    UnknownRegistry {
        /// The relation the batch claims, as its identity or qualified name.
        relation: String,
    },

    /// A field's Arrow storage is not the declared one.
    #[error("field `{field}` is stored as {actual}, and the contract declares {expected}")]
    #[diagnostic(
        code(schema::storage),
        help(
            "an unknown layout is rejected, never decoded opportunistically (blueprint §5.3 step 2)"
        )
    )]
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
    #[diagnostic(
        code(schema::extension_type),
        help("registration is passive; the validator is what admits a value (blueprint §4.4)")
    )]
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
    #[diagnostic(
        code(schema::extension_metadata),
        help(
            "the shapes are exactly `{{\"v\":1}}`, `{{\"v\":1,\"enum_id\":…}}` and `{{\"v\":1,\"target_relation_id\":…}}` (blueprint §4.4)"
        )
    )]
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
    #[diagnostic(
        code(schema::unknown_metadata),
        help(
            "`SERDE_ARROW:*` and other unregistered keys are errors, not values to discard silently (blueprint §4.3)"
        )
    )]
    UnknownMetadata {
        /// The field path, or the empty string for schema-level metadata.
        field: String,
        /// The offending key.
        key: String,
    },

    /// A non-nullable field holds a null, or a nullable one is declared non-null.
    #[error("field `{field}` is declared {declared} and the batch is {actual}")]
    #[diagnostic(
        code(schema::nullability),
        help("a hidden null under a masked parent is still a null (blueprint §5.3 step 2)")
    )]
    Nullability {
        /// The field path.
        field: String,
        /// `non-null` or `nullable`.
        declared: &'static str,
        /// What the batch actually is.
        actual: String,
    },

    /// A dictionary value is not a member of the declared enumeration.
    #[error("field `{field}` holds `{value}`, which is not a member of `{enumeration}`")]
    #[diagnostic(
        code(schema::enum_member),
        help(
            "an enumeration is a closed dictionary; add the member to the registry or correct the value"
        )
    )]
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
    #[diagnostic(
        code(schema::ordinal_range),
        help(
            "an ordinal is artifact-local; only bundle admission can check that it is in range (blueprint §4.4)"
        )
    )]
    OrdinalRange {
        /// The field path.
        field: String,
        /// The ordinal found.
        ordinal: u64,
        /// The target relation.
        target: String,
        /// How many rows the target has.
        rows: u64,
    },

    /// Arrow itself rejected the operation.
    #[error(transparent)]
    #[diagnostic(code(schema::arrow))]
    Arrow(#[from] arrow_schema::ArrowError),
}
