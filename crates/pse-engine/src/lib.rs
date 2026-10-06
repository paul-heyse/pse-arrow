// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

// The `Send` proofs for the nested async state machines exceed rustc's default depth of
// 128; its `recursion_depth_exceeding_limit` future-incompatibility lint asks for more.
#![recursion_limit = "256"]

//! Native DataFusion contracts and owned execution, independent of durable storage.

pub mod arrow_stream;
pub mod cache_service;
pub mod error;
pub mod operation;
pub mod provider;
pub mod session;
pub use error::EngineError;
pub use provider::BoxFut;
pub use session::{EngineFactory, EngineSession, ExecutionSettings, ThreadBudget};

pub mod resources;
pub mod stores;

/// Engine-owned expression binding for columnar contracts.
pub mod validation;
