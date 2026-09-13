// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Phase-0 placeholder for the `tests/lifecycle` layer of blueprint §24.1.
//!
//! A run reaches a terminal state exactly once; cancellation is observed at every
//! await point and leaves the artifact store consistent; a wall-clock limit yields
//! `runtime.timeout` rather than a partial snapshot; case overlays and sweeps
//! produce the recorded set of runs.
//!
//! `.config/nextest.toml` puts this package in the `solver` test group: these
//! tests spawn Ipopt and MUMPS and get a 300 s slow timeout and two threads.

/// Keeps the layer's harness in every gate from the first commit: a test family
/// that reports no tests is indistinguishable from one that was never wired up.
#[test]
fn phase0_placeholder() {
    assert_eq!(env!("CARGO_PKG_NAME"), "pse-tests-lifecycle");
}
