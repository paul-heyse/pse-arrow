// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

// The `Send` proofs for the nested async state machines exceed rustc's default depth of
// 128; its `recursion_depth_exceeding_limit` future-incompatibility lint asks for more.
#![recursion_limit = "256"]

//! Declared relational checks and explicit boundary reference admission.
pub mod errmap;
pub mod error;
pub mod invariants;
pub mod references;
pub use crate::error::RuleError;
