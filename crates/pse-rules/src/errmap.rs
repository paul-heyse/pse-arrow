// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `DataFusionError` into the platform taxonomy, at one place (blueprint §23.2).
//!
//! Per-call-site classification is how a `ResourcesExhausted` becomes an
//! `internal.invariant` in one path and a `user.model` in another. The mapping table is
//! in §23.2 and the origin - rule compiler, analytics or kernel UDF - is what selects the
//! row.
//!
//! Packet C-3 fills this module.
