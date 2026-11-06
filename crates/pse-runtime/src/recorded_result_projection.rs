// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One-way selected-result facts, independent of service activation and read permission.

use pse_contract::boundary_projection::{AdapterError, AdapterFailure};
use pse_contract_proto::support::*;

/// The passive file owns its complete reflection world and message implementations.
#[path = "generated/recorded-result/mod.rs"]
#[rustfmt::skip]
#[allow(unreachable_pub, elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, clippy::upper_case_acronyms, reason = "Pinned protobuf supplier emits public scaffolding inside this intentionally crate-private passive boundary")]
pub(crate) mod pb;
#[path = "generated/recorded-result/validation/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, missing_docs, reason = "Pinned validator generator spelling and packaging modules")]
mod validation;

#[path = "generated/recorded-result/native-adapters.rs"]
#[rustfmt::skip]
mod generated;
pub(crate) use generated::*;

#[cfg(test)]
pub(crate) const DESCRIPTORS: &[u8] = include_bytes!("generated/recorded-result/descriptor.bin");
