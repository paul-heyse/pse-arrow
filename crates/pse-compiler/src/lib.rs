// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physically typed finite process definitions lowered directly to library mathematics.

pub mod typed_math;
pub mod source_binding;

mod physical_identity;
/// Incremental semantic preparation and compiler-owned artifact requests.
pub mod workspace;
