// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Phase-0 placeholder for the `tests/engine` layer of blueprint §24.1.
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

/// Keeps the layer's harness in every gate from the first commit: a test family
/// that reports no tests is indistinguishable from one that was never wired up.
#[test]
fn phase0_placeholder() {
    assert_eq!(env!("CARGO_PKG_NAME"), "pse-tests-engine");
}
