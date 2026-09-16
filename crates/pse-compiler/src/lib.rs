// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The pass registry, the `PassSpec` contracts, the artifact-hash memo and the pipeline
//! driver (blueprint §3.2, §14).
//!
//! Memoization keys are artifact hashes, never plan fingerprints: a plan is a pure function
//! of the rule plan, the catalog snapshot and the engine profile, all of which are already
//! in the key, so hashing the plan adds no information about reuse validity (§14.2 rule 5,
//! ADR-0019). `salsa` is deferred behind this memo with a stated trigger (ADR-0042).
//!
//! # Layout
//!
//! - [`error`] — [`CompilerError`], which forwards the authoring, rule and registry codes
//!   transparently.
//! - [`passes`] — the [`passes::Pass`] trait, the bundle types and the pass
//!   implementations.
//! - [`driver`] — the pipeline driver.
//! - [`memo`] — the artifact-hash memo.
//! - [`records`] — pass records and findings as relations.
//! - [`mathir_relations`] — the math IR to `compiled.math_*` bridge.

pub mod driver;
pub mod error;
pub mod mathir_relations;
pub mod memo;
pub mod passes;
pub mod quantity_relations;
pub mod records;
pub mod validator;

pub use crate::error::CompilerError;
pub use crate::passes::{
    BoundInput, InputBundle, Pass, PassContext, PassStatus, PolicySet, StageKey,
};

/// Compose native process inference with Delta operations for the actual caller.
/// Further domain planners can be composed through `UnifiedPlanner::new`.
#[must_use]
pub fn query_planner() -> std::sync::Arc<pse_catalog::session::planner::UnifiedPlanner> {
    std::sync::Arc::new(pse_catalog::session::planner::UnifiedPlanner::new(vec![
        std::sync::Arc::new(pse_rules::strata::native::RuleExtensionPlanner),
    ]))
}

// Generated formatting is owned by the registry's pinned prettyplease emitter.
#[rustfmt::skip]
mod generated;
