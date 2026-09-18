// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native process-model composition over exact Arrow and Delta inputs.
//! Parsing and finite domain algorithms consume owned facts; output graphs and
//! publication execute through the common DataFusion provider boundary.

pub mod documents;
pub mod error;
pub mod mathir_relations;
pub mod native;
pub mod passes;
pub mod quantity_relations;

pub use crate::error::CompilerError;
pub use crate::passes::{
    Algorithm, AlgorithmContext, AlgorithmInputs, AlgorithmOutput, BoundInput,
};
pub use pse_catalog::artifact::{ArtifactPlan, RelationOutput};

/// Compose native process inference with Delta operations for the actual caller.
/// Further domain planners can be composed through `UnifiedPlanner::new`.
#[must_use]
pub fn query_planner() -> std::sync::Arc<pse_catalog::session::planner::UnifiedPlanner> {
    std::sync::Arc::new(pse_catalog::session::planner::UnifiedPlanner::new(vec![
        std::sync::Arc::new(pse_rules::strata::native::RuleExtensionPlanner),
        std::sync::Arc::new(native::CompilerExtensionPlanner),
    ]))
}

// Generated formatting is owned by the registry's pinned prettyplease emitter.
#[rustfmt::skip]
mod generated;
