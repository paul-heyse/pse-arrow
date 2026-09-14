// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The package and case document loader (blueprint §22.1).
//!
//! YAML through `serde-saphyr` and TOML through `toml` with `Spanned<T>`, so every row's
//! `pse.source_span` comes from the parser rather than from a guess. Hostile input is
//! refused without a panic, under the [`crate::span::ParseBudget`].
//!
//! Packet C-1 fills this module.
