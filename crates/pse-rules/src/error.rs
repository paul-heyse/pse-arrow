// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! What rule compilation and execution can be wrong about (blueprint §23.2).
//!
//! `DataFusionError` is mapped into this taxonomy at one place ([`crate::errmap`]), never
//! per call site: the same variant means `internal.invariant` from the rule compiler and
//! `user.model` from an analytics query, and a per-site `match` gets that wrong exactly
//! once before nobody can tell which is which.

/// A rule that will not compile, or an execution that failed.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum RuleError {
    /// A rule plan keys on a floating-point column (blueprint §14.2 rule 7).
    #[error("rule `{rule}` keys on the `Float64` column `{column}`")]
    #[diagnostic(
        code(rule::float_key),
        help(
            "the engine merges -0.0 with +0.0 and treats NaN as self-equal; key on identities, ordinals or enums"
        )
    )]
    FloatKey {
        /// The rule, as `<name>@<version>`.
        rule: String,
        /// The offending column.
        column: String,
    },

    /// The rule's output does not meet its head relation's contract
    /// (blueprint §14.2 rule 6).
    #[error(
        "rule `{rule}` produced `{found}` for head column `{column}`, which declares `{expected}`"
    )]
    #[diagnostic(
        code(rule::head_schema_mismatch),
        help(
            "`can_cast_types` and `CastOptions {{ safe: false }}` do not prove losslessness; attach destination metadata only after the representability check"
        )
    )]
    HeadSchemaMismatch {
        /// The rule, as `<name>@<version>`.
        rule: String,
        /// The head column.
        column: String,
        /// The declared contract.
        expected: String,
        /// The contract inferred from the plan.
        found: String,
    },

    /// A platform postcondition failed.
    #[error("internal invariant: {what}")]
    #[diagnostic(
        code(internal::invariant),
        help(
            "a rule plan that does not type is a platform bug, not a user error (blueprint §23.2)"
        )
    )]
    Internal {
        /// What did not hold.
        what: String,
    },

    /// The engine ran out of a budgeted resource.
    #[error("{consumer} exceeded its resource limit")]
    #[diagnostic(
        code(runtime::resource_limit),
        help(
            "names the consumer and the configuration keys that bound it, so the limit can be raised deliberately"
        )
    )]
    ResourceLimit {
        /// Which consumer exhausted its budget.
        consumer: String,
        /// The configuration keys that bound it.
        config_keys: Vec<String>,
    },

    /// Storage, Arrow or input/output failed underneath the rule.
    #[error("infrastructure failure during {op}: {detail}")]
    #[diagnostic(
        code(runtime::infrastructure),
        help("the operation is retryable in principle; the rule itself is not at fault")
    )]
    Infrastructure {
        /// What was being done.
        op: String,
        /// What the layer below reported.
        detail: String,
    },

    /// An engine or platform configuration key is invalid.
    #[error("configuration key `{key}` is invalid: {reason}")]
    #[diagnostic(
        code(config::invalid),
        help(
            "settings go through typed `ConfigOptions`; `SessionConfig::set_str` panics instead and is banned"
        )
    )]
    ConfigInvalid {
        /// The key.
        key: String,
        /// Why it was rejected.
        reason: String,
    },
}
