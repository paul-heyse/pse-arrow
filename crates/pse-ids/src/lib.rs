// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `SemanticId`, `ContentHash`, ordinals, derived-ID rules and the canonical IPC
//! serializer (blueprint §3.2, §5.1).
//!
//! The only crate in the workspace allowed to depend on `blake3` (governance
//! `blake3_owner`): semantic IDs retain their versioned `derive_key` contexts
//! (blueprint §5.1); logical relation hashes use `pse.canon.v2`, separately from
//! physical encoding checksums (blueprint §5.3, ADR-0045).
//!
//! Phase 0: this crate is a declared boundary with no implementation yet.
