// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Semantic admission of plans and batches: the first and final analyzer rules,
//! recursive field validation, and the refusal of any scan that is not a snapshot
//! relation (blueprint §4.4, §5.4).
//!
//! Packet B-session fills this.
