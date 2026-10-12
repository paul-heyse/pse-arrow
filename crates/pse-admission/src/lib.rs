// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Owned document intake and physical source admission through actual native bindings.

/// Effectful document parsing, loading, editing and checked relation construction.
pub mod authoring_driver;
/// Physical declarations projected from admitted source rows.
pub mod physical;
#[rustfmt::skip]
mod generated;
