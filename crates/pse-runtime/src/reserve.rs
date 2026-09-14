// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The `MemoryReserver` implementation over a DataFusion `MemoryPool`: a
//! `MemoryConsumer` per owner, registered without spill and grown before the
//! allocation it accounts for (blueprint §14.3, ADR-0046).
//!
//! Packet E fills this.
