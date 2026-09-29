// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Feature unification for the pse-arrow workspace, managed by cargo-hakari (ADR-0122).
//!
//! This crate has no code. Its generated `Cargo.toml` depends on every third-party crate
//! the workspace builds with more than one feature set, with the union of those
//! features, and every workspace member depends on it. Whatever packages a command
//! selects, each dependency then resolves to one feature set and one build.
