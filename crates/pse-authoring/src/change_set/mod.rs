// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Change sets: the only write path into `authored` (blueprint §22.2, decision D2).
//!
//! A change set is validated by P2 on the resulting snapshot, applied atomically, and
//! yields a new model or case revision. Staged rows are typed relation rows, never JSON
//! text, so D1 holds inside the change log too.
//!
//! Packet C-2 fills this module.
