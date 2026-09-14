// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! What the compiler can be wrong about (blueprint §23.2).
//!
//! The authoring and rule errors are re-exported through `#[from]` with
//! `#[diagnostic(transparent)]`: a caller sees the original code — `authoring.parse.syntax`,
//! `rule.float_key` — rather than a compiler-shaped wrapper that has lost which layer knew
//! what was wrong. Failures carry the relation rows and source spans involved; they are
//! never flattened into "run failed".

use pse_authoring::AuthoringError;
use pse_rules::RuleError;
use pse_schema::SchemaError;

/// A pipeline that will not run, or a stage that failed.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum CompilerError {
    /// The declared ports do not describe a runnable pipeline (blueprint §14.1).
    #[error("stage graph: {reason}")]
    #[diagnostic(
        code(compile::stage_graph),
        help("every read names its source stage and port; one port has one producer")
    )]
    StageGraph {
        /// What is wrong with the graph.
        reason: String,
    },

    /// A pass postcondition failed.
    #[error("internal invariant: {what}")]
    #[diagnostic(
        code(internal::invariant),
        help(
            "a failed postcondition is a platform bug; the pass declared it and did not establish it"
        )
    )]
    Internal {
        /// What did not hold.
        what: String,
    },

    /// A budgeted resource ran out.
    #[error("{consumer} exceeded its resource limit")]
    #[diagnostic(
        code(runtime::resource_limit),
        help(
            "preflight reserves before allocating; exceeding a bound is never a partial publication (blueprint §5.3 step 8)"
        )
    )]
    ResourceLimit {
        /// Which consumer exhausted its budget.
        consumer: String,
        /// The configuration keys that bound it.
        config_keys: Vec<String>,
    },

    /// Storage or input/output failed underneath a pass.
    #[error("infrastructure failure during {op}: {detail}")]
    #[diagnostic(
        code(runtime::infrastructure),
        help("an interrupted operation leaves the old ref valid and publishes nothing mixed")
    )]
    Infrastructure {
        /// What was being done.
        op: String,
        /// What the layer below reported.
        detail: String,
    },

    /// An authoring failure, with its own code.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Authoring(#[from] AuthoringError),

    /// A rule failure, with its own code.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Rule(#[from] RuleError),

    /// A registry failure, with its own code.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Schema(#[from] SchemaError),
}
