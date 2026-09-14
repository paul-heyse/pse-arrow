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
pub mod records;

pub use crate::error::CompilerError;
pub use crate::passes::{
    BoundInput, ExternalInputs, InputBundle, Pass, PassContext, PassOutput, PassRecordDraft,
    PassStatus, PolicySet, StageKey,
};
