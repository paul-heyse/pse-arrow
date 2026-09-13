// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Phase-0 placeholder for the `tests/conformance` layer of blueprint §24.1.
//!
//! Native residuals and Jacobians equal Pyomo residuals and Jacobians (`PyNumero`)
//! at random valid points to 1e-10 relative; the NL round trip solves with Ipopt
//! to the same solution as the native driver; SOL parsing round-trips.
//!
//! The Python half of this layer is `python/pse/parity`; this crate holds the
//! Rust-side comparisons that need no interpreter.

/// Keeps the layer's harness in every gate from the first commit: a test family
/// that reports no tests is indistinguishable from one that was never wired up.
#[test]
fn phase0_placeholder() {
    assert_eq!(env!("CARGO_PKG_NAME"), "pse-tests-conformance");
}
