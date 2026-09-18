// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native provider contracts, owned Arrow execution and coherent Delta publications.

pub mod artifact;
pub mod cache_service;
pub mod contract;
pub mod delta;
pub mod error;
pub mod failure;
pub mod inspection;
pub mod provider;
pub mod session;

pub use crate::error::CatalogError;
pub use crate::failure::{PlanOrigin, classify};
pub use crate::provider::BoxFut;
pub use crate::session::{ExecutionSettings, ThreadBudget};
