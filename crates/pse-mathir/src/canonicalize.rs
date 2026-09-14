// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pass P10, in order (blueprint §7.4).
//!
//! Filled by packet M-4: `canonicalize(input, Policy::Strict)` running acyclicity,
//! indices, bottom-up inference, guarded folding, typed hash-consing and numbering.
