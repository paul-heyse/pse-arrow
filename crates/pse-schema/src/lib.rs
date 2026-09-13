// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Relation, logical-type and extension-type registry plus the syn/quote/prettyplease code generator (blueprint §3.2).
//!
//! The registry is the single authority for every relation shape; `pse-relations` is
//! its generated output, committed to the tree and diffed in CI (blueprint §3.2, §4).
//!
//! Phase 0: this crate is a declared boundary with no implementation yet.
