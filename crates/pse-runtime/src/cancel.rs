// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The cancellation source behind `pse_ids::CancellationToken`: cooperative
//! cancellation that leaves no partial artifact behind (blueprint §14.3, §24.1).
//!
//! Packet E fills this.
