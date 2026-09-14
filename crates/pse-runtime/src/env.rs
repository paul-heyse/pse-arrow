// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The shared `RuntimeEnv`: one configured accounted budget, disk manager and object
//! store registry for every snapshot session in the process (blueprint §14.3,
//! ADR-0046).
//!
//! Packet E fills this.
