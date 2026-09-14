// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P1: authoring parse (blueprint §14.1).
//!
//! Stages authored rows and change operations, including the identity-based target rows.
//! P1 does not mutate an existing committed snapshot; it produces a candidate that P2
//! validates.
//!
//! Packet C-2 fills this module.
