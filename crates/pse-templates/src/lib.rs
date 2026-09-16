// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generic declaration/instance substitution over the existing mathematical DAG.
//! Physics remains in admitted package declarations (blueprint §6.15.4, §10).

mod environment;
pub mod identity;
pub mod laws;
pub mod paths;
mod projection;
mod rebuild;

pub use environment::{
    BindingValue, EvaluatedGather, GroupBinding, InstantiationEnvironment, PathBinding,
    PredicateMask,
};
pub use projection::{GroupProjection, project_group};
pub use rebuild::{Instantiation, coordinate_nodes, instantiate};

use pse_ids::SemanticId;

/// Typed template binding failures, distinct from physical inference performed at P10.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum TemplateError {
    /// An actual binding is missing, ambiguous or incompatible.
    #[error("template instance {instance}: {detail}")]
    #[diagnostic(code(validation::invariant))]
    Binding {
        /// Owning instance.
        instance: SemanticId,
        /// Actual unmet declaration contract.
        detail: String,
    },
    /// A demanded compile-time predicate lacks a decided actual outcome.
    #[error("template instance {instance}: predicate {source_id}/{predicate} is undecided")]
    #[diagnostic(code(template::guard_undecidable))]
    GuardUndecidable {
        /// Owning instance.
        instance: SemanticId,
        /// Normalized predicate source.
        source_id: SemanticId,
        /// Source-local predicate ordinal.
        predicate: u64,
    },
    /// Existing structural/lexical mathematical admission.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Math(#[from] pse_mathir::MathIrError),
    /// Cancellation and allocation admission.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Resource(#[from] pse_ids::CanonError),
}
