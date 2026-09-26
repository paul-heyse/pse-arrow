// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete graph projections and certified structural analysis.
/// Complete immutable dependency projections and library analysis.
pub mod projection;

/// Complete physical process-flow graph and explicit tear-decision groups.
pub mod flowsheet;
/// Library-owned selected-case structural analysis.
pub mod incidence;
/// Conditional BTF initialization plans with explicit boundary inputs.
pub mod initialization;
