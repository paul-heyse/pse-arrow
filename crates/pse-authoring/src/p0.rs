// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P0: package and schema resolution (blueprint §14.1).
//!
//! Resolves package headers from an input document bundle into
//! `normalized.package_graph`, with every referenced package pinned by content hash.
//! Phase 0 admits exact version requirements only.
//!
//! Packet C-1 fills this module.
