// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Evaluation program, automatic differentiation and faer sparse linear algebra (blueprint §3.2, §16).
//!
//! Per-equation reverse mode for residuals and Jacobians, `num-dual` inside kernels;
//! one thread budget shared with tokio and rayon (blueprint §16, §18.8).
//!
//! Phase 0: this crate is a declared boundary with no implementation yet.
