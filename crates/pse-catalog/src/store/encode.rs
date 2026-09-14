// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Arrow IPC file and Parquet encoders with an explicit successful finish, and the
//! encoding checksum taken from the finished bytes (blueprint §5.3 step 6, §20.1).
//!
//! Packet B-store fills this.
