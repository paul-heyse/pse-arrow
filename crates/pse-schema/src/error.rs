// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The registry's failure taxonomy (blueprint §23.2).
//!
//! Every variant carries a `#[diagnostic(code(...))]` in the §23.2 `schema.*` class, so a
//! caller can branch on the class without parsing a message. The enum is `Clone` because
//! [`crate::registry`] memoizes `Result<Registry, SchemaError>` in a `OnceLock` and has to
//! hand the same failure to every later caller rather than re-running assembly.

/// A declaration the registry refuses to accept.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, miette::Diagnostic)]
pub enum SchemaError {
    /// A declaration is structurally present but violates its semantic contract.
    #[error("invalid {context}: {reason}")]
    #[diagnostic(code(schema::invalid_declaration))]
    InvalidDeclaration {
        /// The declaration or operator being admitted.
        context: String,
        /// The actual violated requirement, independent of content identity.
        reason: String,
    },
    /// Two declarations claim the same name.
    #[error("{kind} `{name}` is declared twice")]
    #[diagnostic(
        code(schema::duplicate_declaration),
        help(
            "one authoritative declaration per meaning: delete one of the two, or version them apart"
        )
    )]
    DuplicateDeclaration {
        /// What was declared twice: `relation`, `enum`, `invariant`, `pass`, `rule`,
        /// `migration`, `document` or `output port`.
        kind: &'static str,
        /// The name that appears twice.
        name: String,
    },

    /// A declaration names something the registry does not declare.
    #[error("{context} refers to `{reference}`, which the registry does not declare")]
    #[diagnostic(
        code(schema::unknown_reference),
        help("declare the target in a catalog module, or correct the reference")
    )]
    UnknownReference {
        /// Where the dangling reference was found, for example
        /// `column authored.entities.package_id`.
        context: String,
        /// The name that does not resolve.
        reference: String,
    },

    /// A relation reached assembly without a snapshot class (blueprint §5.3 step 7).
    #[error("relation `{relation}` declares no snapshot class")]
    #[diagnostic(
        code(schema::missing_snapshot_class),
        help("membership is explicit: pick model, case, derived or sidecar")
    )]
    MissingSnapshotClass {
        /// The relation, as `<namespace>.<name>@<version>`.
        relation: String,
    },

    /// A derived relation declares no derivation granularity (blueprint §14.2 rule 4).
    #[error("derived relation `{relation}` declares no derivation granularity")]
    #[diagnostic(
        code(schema::missing_granularity),
        help(
            "`row` when negative completeness is the deliverable, `rule` when the derivation is the rule plus the identity formula"
        )
    )]
    MissingGranularity {
        /// The relation, as `<namespace>.<name>@<version>`.
        relation: String,
    },

    /// A rule plan keys on a floating-point column (blueprint §14.2 rule 7).
    #[error("rule `{rule}` keys on the `f64` column `{column}`")]
    #[diagnostic(
        code(schema::rule_float_key),
        help(
            "the engine merges -0.0 with +0.0 and treats NaN as self-equal; key on identities, ordinals or enums"
        )
    )]
    RuleFloatKey {
        /// The rule, as `<name>@<version>`.
        rule: String,
        /// The offending column.
        column: String,
    },

    /// A rule negates a relation that is not settled in a lower stratum
    /// (blueprint §14.2 rule 2).
    #[error("rule `{rule}` negates `{relation}`, which is not settled in a lower stratum")]
    #[diagnostic(
        code(schema::rule_stratification),
        help("raise the rule's stratum above every rule that writes the negated relation")
    )]
    RuleStratification {
        /// The rule, as `<name>@<version>`.
        rule: String,
        /// The negated relation.
        relation: String,
    },

    /// A generator could not render the registry.
    #[error("code generation for {language}: {reason}")]
    #[diagnostic(
        code(schema::codegen),
        help("the generator is the fix; a hand edit under a generated path is a red diff")
    )]
    Codegen {
        /// The target language.
        language: &'static str,
        /// What the generator could not render.
        reason: String,
    },
}
