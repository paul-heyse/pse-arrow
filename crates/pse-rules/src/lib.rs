// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `RuleSpec` to DataFusion `LogicalPlan`, the fixed-point executor and provenance rows
//! (blueprint §3.2, §14.2).
//!
//! # Layout
//!
//! - [`error`] — [`RuleError`] with its §23.2 codes.
//! - [`plan`] — the rule-plan compiler.
//! - [`exec`] — the fixed-point executor.
//! - [`invariants`] — P2's body.
//! - [`strata`] — native finite inference and typed provenance outputs.
//! - [`errmap`] — `DataFusionError` into the taxonomy, at one place.

pub mod errmap;
pub mod error;
pub mod exec;
pub mod invariants;
pub mod plan;
pub mod strata;

pub use crate::error::RuleError;
