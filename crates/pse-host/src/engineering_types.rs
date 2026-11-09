// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Common engineering message ownership, independent of RPCs and runtime.

/// Shared message types, views and reflection.
#[path = "generated/engineering-shared/mod.rs"]
#[rustfmt::skip]
#[allow(elided_lifetimes_in_paths, trivial_numeric_casts, missing_debug_implementations, missing_docs, clippy::upper_case_acronyms, reason = "Pinned protobuf generator surface")]
pub mod pb;
#[path = "generated/engineering-shared/validation/mod.rs"]
#[rustfmt::skip]
#[allow(unused_qualifications, elided_lifetimes_in_paths, trivial_numeric_casts, unreachable_pub, missing_docs, reason = "Pinned validator generator spelling and packaging modules")]
mod validation;
