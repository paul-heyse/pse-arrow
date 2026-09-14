// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Verification of an existing stored object against its intended manifest member:
//! length, format, encoding checksum and, for an untrusted import, the recomputed
//! logical hash (blueprint §20.1).
//!
//! Packet B-store fills this.
