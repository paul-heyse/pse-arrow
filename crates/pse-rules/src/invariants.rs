// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P2's body: running every `reference.schema_invariants` rule over a snapshot
//! (blueprint §14.1, §22.2).
//!
//! A validator returns violating keys plural. The first error is never the only one
//! reported (blueprint §23.2).
//!
//! Packet C-3 fills this module.
