// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `RuleSpec` to DataFusion `LogicalPlan` compiler, fixed-point executor and provenance
//! rows (blueprint §3.2, §14.2).
//!
//! One of exactly two crates allowed to depend on DataFusion planning types
//! (`pse-catalog` is the other); rule plans must encode to byte-identical
//! `datafusion-proto` bytes across processes (blueprint §3.2, §14.2, §24.1).
//!
//! Phase 0: this crate is a declared boundary with no implementation yet.
