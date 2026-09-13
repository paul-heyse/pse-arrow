// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Criterion benchmarks for the blueprint §24.3 performance groups.
//!
//! The library target exists so the package has something to own the `[[bench]]` targets;
//! shared fixture builders will live here once there is something to measure. Baselines
//! are saved per snapshot and compared by hand: CI runs the benchmarks for compilation and
//! a single iteration only, never as a timing gate (plan §6).
