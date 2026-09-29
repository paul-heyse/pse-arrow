// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

// The `Send` proofs for the nested async state machines exceed rustc's default depth of
// 128; its `recursion_depth_exceeding_limit` future-incompatibility lint asks for more.
#![recursion_limit = "256"]

//! Native provider contracts, owned Arrow execution and coherent Delta publications.

pub mod artifact;
pub mod assembly;
pub mod cache_service;
pub mod contract;
pub mod delta;
pub mod inspection;
pub mod selection;

use pse_engine::EngineError;
