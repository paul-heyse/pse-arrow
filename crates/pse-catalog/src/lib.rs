// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native provider contracts, owned Arrow execution and coherent Delta publications.

pub mod artifact;
pub mod assembly;
pub mod cache_service;
pub mod contract;
pub mod delta;
pub mod inspection;
pub mod selection;

use pse_engine::EngineError;
