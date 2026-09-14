// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The bridge between `pse-mathir` values and the `compiled.math_*` relations
//! (blueprint §6.9, §7).
//!
//! The math IR is the in-memory form; the relations are the stored form. Keeping the
//! conversion in one module is what stops a pass from inventing a second encoding of an
//! expression graph.
//!
//! Packet C-4 fills this module.
