// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-derived private root records, available without native solver preparation.
#[cfg(all(unix, feature = "solver-kinsol"))]
use pse_contract::boundary_projection::{AdapterError, AdapterFailure};
#[cfg(all(unix, feature = "solver-kinsol"))]
use pse_contract_proto::support::*;

#[cfg(all(unix, feature = "solver-kinsol"))]
#[path = "../generated/native-root/native-adapters.rs"]
#[rustfmt::skip]
mod native_projection;
#[cfg(all(unix, feature = "solver-kinsol"))]
pub(crate) use native_projection::*;

#[path = "../generated/native-root/mod.rs"]
#[rustfmt::skip]
#[allow(elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, clippy::upper_case_acronyms, reason = "Pinned protobuf generator surface")]
pub mod pb;
#[path = "../generated/native-root/validation/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, missing_docs, reason = "Pinned validator generator packaging")]
mod validation;
/// Complete immutable descriptor closure of the actual native root source owners.
pub const DESCRIPTORS: &[u8] = include_bytes!("../generated/native-root/descriptor.bin");
