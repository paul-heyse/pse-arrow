// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One-way selected-result facts, independent of service activation and read permission.

use pse_contract::boundary_projection::{AdapterError, AdapterFailure};
use pse_contract_proto::support::*;

#[path = "generated/engineering/recorded-result-adapters.rs"]
#[rustfmt::skip]
mod generated;
pub(crate) use generated::*;
