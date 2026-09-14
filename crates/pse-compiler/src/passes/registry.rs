// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The pass registry: the implementations behind the declared `PassSpec`s
//! (blueprint §14.1).
//!
//! A pass is a `reference.pass_specs` row first and a Rust type second; this module is
//! where the two are bound together and where a spec without an implementation is caught.
//!
//! Packet C-4 fills this module.
