// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete package and rule dependency graphs using library ordering and SCC analysis.
//!
//! Complete graph projections and certified structural analysis.
/// Complete immutable dependency projections and library analysis.
pub mod projection;

/// Typed graph projection contracts.
pub mod domains;

#[cfg(test)]
#[path = "rule_schedule_tests.rs"]
mod computation_unit;

/// Complete physical process-flow graph and explicit tear-decision groups.
pub mod flowsheet;
/// Library-owned selected-case structural analysis.
pub mod incidence;
/// Conditional BTF initialization plans with explicit boundary inputs.
pub mod initialization;
