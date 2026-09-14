// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The Rust generator: typed views, builders, validators, enums and migrations
//! (blueprint §4.2, ADR-0031, ADR-0051).
//!
//! A non-macro `syn`/`quote`/`prettyplease` pipeline whose output is committed under
//! `crates/pse-relations/src/generated/` and `crates/pse-authoring/src/generated/` and
//! diffed in CI. Not a proc macro: generated source that nobody can read is generated
//! source that nobody reviews, and the `no_shadow_structs` governance grep needs symbol
//! names it can find in a file.
//!
//! Packet A-6 fills this module. The `@generated` header is prepended *after*
//! `prettyplease::unparse`, because the formatter would otherwise reflow it.
