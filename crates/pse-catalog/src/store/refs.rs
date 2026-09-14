// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The ref compare-and-swap: `PutMode::Update` against the prior version, `Create` for
//! an absent ref, and the declared retry policy on a precondition failure
//! (blueprint §20.1).
//!
//! Packet B-store fills this.
