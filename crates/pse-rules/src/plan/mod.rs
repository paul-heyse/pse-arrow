// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `RuleSpec` to DataFusion `LogicalPlan` (blueprint §14.2).
//!
//! A rule body is a bounded algebra compiled through `LogicalPlanBuilder`, never a Rust
//! loop over rows. Key columns are key-role and never `Float64`: the engine merges `-0.0`
//! with `+0.0` and treats NaN as self-equal (§14.2 rule 7).
//!
//! Packet C-3 fills this module.
