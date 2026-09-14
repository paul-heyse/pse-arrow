// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Backend conformance tests: native, NL and Pyomo agree (blueprint §24.1).
//!
//! Native residuals and Jacobians equal Pyomo residuals and Jacobians (`PyNumero`)
//! at random valid points to 1e-10 relative; the NL round trip solves with Ipopt
//! to the same solution as the native driver; SOL parsing round-trips.
//!
//! The Python half of this layer is `python/pse/parity`; this crate holds the
//! Rust-side comparisons that need no interpreter.
//!
//! Phase 0: the layer is declared and its harness runs; the tests arrive with the
//! crates they exercise.

#[cfg(test)]
pub mod canon_fixtures;
#[cfg(test)]
pub mod fixture;
