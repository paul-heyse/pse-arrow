// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Run lifecycle tests: controller, overlays, sweeps, cancellation (blueprint §24.1, §20).
//!
//! A run reaches a terminal state exactly once; cancellation is observed at every
//! await point and leaves the artifact store consistent; a wall-clock limit yields
//! `runtime.timeout` rather than a partial snapshot; case overlays and sweeps
//! produce the recorded set of runs.
//!
//! `.config/nextest.toml` puts this package in the `solver` test group: these
//! tests spawn Ipopt and MUMPS and get a 300 s slow timeout and two threads.
//!
//! Phase 0: the layer is declared and its harness runs; the tests arrive with the
//! crates they exercise.
