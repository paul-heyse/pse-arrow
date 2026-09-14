// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The `SchemaProvider` for one namespace, serving `table()` from the snapshot loaded
//! at session creation and performing no I/O (blueprint §5.4).
//!
//! Packet B-providers fills this.
