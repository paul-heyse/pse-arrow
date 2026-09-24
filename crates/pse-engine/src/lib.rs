// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native DataFusion contracts and owned execution, independent of durable storage.

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
