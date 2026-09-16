// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `RuleSpec` to DataFusion `LogicalPlan`, the fixed-point executor and provenance rows
//! (blueprint §3.2, §14.2).
//!
//! One of exactly two crates allowed to depend on DataFusion planning types (`pse-catalog`
//! is the other). Set-oriented inference is written as `reference.rule_specs`, not as Rust
//! loops over rows, so that a new diagnostic check is data rather than a runtime branch
//! (§22.3).
//!
//! # Layout
//!
//! - [`error`] — [`RuleError`] with its §23.2 codes.
//! - [`plan`] — the rule-plan compiler.
//! - [`exec`] — the fixed-point executor.
//! - [`invariants`] — P2's body.
//! - [`derivations`] — `provenance.derivations` rows.
//! - [`errmap`] — `DataFusionError` into the taxonomy, at one place.

pub mod errmap;
pub mod error;
pub mod exec;
pub mod invariants;
pub mod plan;
pub mod strata;
pub mod validator;

pub use crate::error::RuleError;
