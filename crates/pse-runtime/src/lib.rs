// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Run controller, case overlays, sweeps, result ingestion and lifecycle/cancellation (blueprint §3.2, §20).
//!
//! The runtime owns `probe_host()`: the capability report (solver versions, external
//! function libraries, lockfile hashes) that every run records (blueprint §20, §24.1).
//!
//! Phase 0: this crate is a declared boundary with no implementation yet.
