// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bundle assembly for the Python Pyomo adapter: schemas, ordering and source maps (blueprint §3.2, §21.2).
//!
//! This crate assembles the bundle; it never imports Python. The adapter that
//! consumes it lives in `python/pse/adapters/pyomo` (blueprint §21.2, §21.3).
//!
//! Phase 0: this crate is a declared boundary with no implementation yet.
