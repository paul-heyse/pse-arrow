// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared direct-native study projections and readonly WASI policy.
/// Owner-derived direct adapters, shared by native and WASI callers.
#[path = "generated/native.rs"]
#[rustfmt::skip]
pub mod generated;
/// Source-generated native value boundary support.
pub mod native;
mod policy;
/// Evaluate the same bounded pure readiness operation used by the WASI export.
pub use policy::evaluate_readiness;
