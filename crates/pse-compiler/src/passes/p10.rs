// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P10: math canonicalization and unit inference (blueprint §14.1, §7.4).
//!
//! Never folds parameter symbols: static attributes are computed on a case-bound view
//! whose substitutions are recorded (blueprint revision 2, finding F7).
//!
//! Packet C-4 fills this module.
