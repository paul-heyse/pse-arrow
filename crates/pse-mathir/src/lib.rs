// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Expression graph, operator catalog and contracts, canonicalization and unit inference (blueprint §3.2, §12).
//!
//! The math IR is relational: expression graphs are rows, never an in-memory AST that
//! escapes the registry (blueprint §12).
//!
//! Phase 0: this crate is a declared boundary with no implementation yet.
