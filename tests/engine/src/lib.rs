// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Engine reproducibility and pushdown truthfulness tests (blueprint §24.1).
//!
//! Engine reproducibility: the same rule plan encoded in two fresh processes
//! yields byte-identical `datafusion-proto` bytes; adding a rule to the engine
//! profile or changing an allow-listed setting misses every rule-pass memo; a
//! batch with a `pse.*` extension over the wrong storage type is rejected at
//! planning; a `distinct` or join on a `Float64` column is rejected by the rule
//! compiler.
//!
//! Pushdown truthfulness: the wrapper provider re-applies every `Exact` filter
//! over the golden snapshots and finds no survivor; an `IN` over a key column
//! reaches the scan as `Exact`.
//!
//! Phase 0: the layer is declared and its harness runs; the tests arrive with the
//! crates they exercise.

pub mod oracle;
