// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-generated engineering crossings shared by actual native workflows.

use pse_contract::boundary_projection::{AdapterError, AdapterFailure};
use pse_contract_proto::support::*;

#[path = "generated/engineering/native-adapters.rs"]
#[rustfmt::skip]
mod generated;
pub use crate::host_operations::engineering::inspection::{
    decode_modeling_inspection, encode_modeling_inspection,
};
pub use generated::*;
