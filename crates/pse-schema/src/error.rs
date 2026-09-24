// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The registry's failure taxonomy (blueprint §23.2).
//!
//! Every variant projects a typed leaf diagnostic code in the §23.2 `schema.*` class, so a
//! caller can branch on the class without parsing a message. The enum is `Clone` because
//! [`crate::registry`] memoizes `Result<Registry, SchemaError>` in a `OnceLock` and has to
//! hand the same failure to every later caller rather than re-running assembly.

/// A declaration the registry refuses to accept.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SchemaError {
    /// A declaration is structurally present but violates its semantic contract.
    #[error("invalid {context}: {reason}")]
    InvalidDeclaration {
        /// The declaration or operator being admitted.
        context: String,
        /// The actual violated requirement, independent of content identity.
        reason: String,
    },
    /// Two declarations claim the same name.
    #[error("{kind} `{name}` is declared twice")]
    DuplicateDeclaration {
        /// What was declared twice: `relation`, `enum`, `invariant`, `pass`, `rule`,
        /// `migration`, `document` or `output port`.
        kind: &'static str,
        /// The name that appears twice.
        name: String,
    },

    /// A declaration names something the registry does not declare.
    #[error("{context} refers to `{reference}`, which the registry does not declare")]
    UnknownReference {
        /// Where the dangling reference was found, for example
        /// `column authored.entities.package_id`.
        context: String,
        /// The name that does not resolve.
        reference: String,
    },

    /// A relation reached assembly without a snapshot class (blueprint §5.3 step 7).
    #[error("relation `{relation}` declares no snapshot class")]
    MissingSnapshotClass {
        /// The relation, as `<namespace>.<name>@<version>`.
        relation: String,
    },

    /// A derived relation declares no derivation granularity (blueprint §14.2 rule 4).
    #[error("derived relation `{relation}` declares no derivation granularity")]
    MissingGranularity {
        /// The relation, as `<namespace>.<name>@<version>`.
        relation: String,
    },

    /// A generator could not render the registry.
    #[error("code generation for {language}: {reason}")]
    Codegen {
        /// The target language.
        language: &'static str,
        /// What the generator could not render.
        reason: String,
    },
}

pse_diagnostics::impl_diagnostic! {
    SchemaError,
    code(this) { match this {
            Self::InvalidDeclaration { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaInvalidDeclaration),
            Self::DuplicateDeclaration { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaDuplicateDeclaration),
            Self::UnknownReference { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaUnknownReference),
            Self::MissingSnapshotClass { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaMissingSnapshotClass),
            Self::MissingGranularity { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaMissingGranularity),
            Self::Codegen { .. } => Some(pse_diagnostics::DiagnosticCode::SchemaCodegen),
            _ => None,
        } },
    forward(_this) { None },
    help(this) { match this {
            Self::DuplicateDeclaration { .. } => Some(Box::new("one authoritative declaration per meaning: delete one of the two, or version them apart")),
            Self::UnknownReference { .. } => Some(Box::new("declare the target in a catalog module, or correct the reference")),
            Self::MissingSnapshotClass { .. } => Some(Box::new("membership is explicit: pick model, case, derived or sidecar")),
            Self::MissingGranularity { .. } => Some(Box::new("`row` when negative completeness is the deliverable, `rule` when the derivation is the rule plus the identity formula")),
            Self::Codegen { .. } => Some(Box::new("the generator is the fix; a hand edit under a generated path is a red diff")),
            Self::InvalidDeclaration { .. } => None,
        } },
    related(_this) { None },
    source(_this) { None }
}

pse_columnar::impl_native_error!(SchemaError);
