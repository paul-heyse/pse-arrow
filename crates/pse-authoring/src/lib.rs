// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Package and case document loader, expression DSL parser and change sets (blueprint §3.2, §11).
//!
//! Documents are parsed with spans so every authoring failure carries a
//! `pse.source_span` (blueprint §11, §23.2 `authoring.parse`).
//!
//! Phase 0: this crate is a declared boundary with no implementation yet.
