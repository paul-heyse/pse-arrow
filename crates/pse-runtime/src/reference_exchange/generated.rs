// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-derived exchange messages, constraints and full imported descriptors.
#[path = "../generated/reference-exchange/mod.rs"]
#[rustfmt::skip]
#[allow(elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, clippy::upper_case_acronyms, reason = "Pinned protobuf generator surface")]
pub mod pb;
#[path = "../generated/reference-exchange/validation/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, missing_docs, reason = "Pinned validator generator spelling and packaging modules")]
mod validation;
/// Immutable source-derived full descriptor, also usable by independent consumers.
pub const DESCRIPTORS: &[u8] = include_bytes!("../generated/reference-exchange/descriptor.bin");
