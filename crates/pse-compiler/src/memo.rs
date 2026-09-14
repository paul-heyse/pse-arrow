// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The artifact-hash memo (blueprint §14.3, ADR-0042).
//!
//! `salsa` is deferred behind this memo with a stated trigger: a conservative complete
//! cache key reuses a stage only when every declared input matches, which is coarse but
//! never wrong. Fine-grained memoization is register row R-13.
//!
//! Packet C-4 fills this module.
