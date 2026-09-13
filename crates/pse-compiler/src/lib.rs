// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pass registry, `PassSpec` contracts, artifact-hash memo and the pipeline driver (blueprint §3.2, §14.3).
//!
//! Memoization keys are artifact hashes, never plan fingerprints used as memo keys
//! (blueprint §14.3, ADR-0019); `salsa` is deferred with a trigger (ADR-0020).
//!
//! Phase 0: this crate is a declared boundary with no implementation yet.
