// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `SemanticId`, `ContentHash`, ordinals, derived-ID rules and the canonical IPC
//! serializer (blueprint §3.2, §5.1).
//!
//! The only crate in the workspace allowed to depend on `blake3` (governance
//! `blake3_owner`): every 128-bit identity comes from a `derive_key` context under
//! the `pse.canon.v1` contract (blueprint §5.1, §5.3).
//!
//! Phase 0: this crate is a declared boundary with no implementation yet.
