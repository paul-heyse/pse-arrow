// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P3: canonicalization (blueprint §14.1).
//!
//! Aliases resolved, defaults materialized, authored constants normalized to package unit
//! sets, expression text parsed into `normalized.*_expr_*`, derived IDs assigned.
//!
//! Packet C-4 fills this module.
