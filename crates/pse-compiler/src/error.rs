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

/// Native model composition or a finite domain algorithm failed.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum CompilerError {
    /// Exact material composition or element declaration failure.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Material(#[from] pse_material::MaterialError),
    /// A selected kernel method lacks its actual executable or parameter binding.
    #[error("selected method {method_id} has an unbound kernel {kernel}: {detail}",
        kernel = kernel_id.map_or_else(|| "undeclared".to_owned(), |id| id.to_string()))]
    #[diagnostic(code(kernel::unbound_parameter))]
    KernelUnbound {
        /// Exact selected method declaration.
        method_id: pse_ids::SemanticId,
        /// Actual kernel declaration, if the method supplied one.
        kernel_id: Option<pse_ids::SemanticId>,
        /// Missing or incompatible actual contract.
        detail: String,
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

    /// A pass failed its declared contract, retaining the actual diagnostic rows.
    #[error("{pass} failed its postconditions")]
    #[diagnostic(code(internal::invariant))]
    Postcondition {
        /// The pass whose declared output contract failed.
        pass: String,
        /// All findings produced by the failed execution.
        findings: Vec<pse_relations::RecordBatch>,
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

    /// A declaration cannot be instantiated under its actual bindings.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Template(#[from] pse_templates::TemplateError),

    /// A catalog admission or publication failure, retaining its own code.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Catalog(#[from] pse_catalog::CatalogError),

    /// A typed relation admission failure.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Relation(#[from] pse_relations::RelationError),

    /// Cancellation or a bounded canonical operation failed.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Canon(#[from] pse_ids::CanonError),

    /// Physical quantity admission failed.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Quantity(#[from] pse_quantity::QuantityError),

    /// Mathematical graph admission failed.
    #[error(transparent)]
    #[diagnostic(transparent)]
    MathIr(#[from] pse_mathir::MathIrError),

    /// A registry failure, with its own code.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Schema(#[from] SchemaError),
}
