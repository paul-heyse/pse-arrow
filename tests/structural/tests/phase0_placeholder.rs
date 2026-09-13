// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Phase-0 placeholder for the `tests/structural` layer of blueprint §24.1.
//!
//! Matching, the Dulmage-Mendelsohn partition, SCC order and connected components
//! agree with reference fixtures generated once from Pyomo incidence analysis on
//! the IDAES tutorials. `petgraph` leaves intra-component order arbitrary, so the
//! fixtures compare semantic-ID-sorted output, never petgraph's own ordering
//! (blueprint §15.3).

/// Keeps the layer's harness in every gate from the first commit: a test family
/// that reports no tests is indistinguishable from one that was never wired up.
#[test]
fn phase0_placeholder() {
    assert_eq!(env!("CARGO_PKG_NAME"), "pse-tests-structural");
}
