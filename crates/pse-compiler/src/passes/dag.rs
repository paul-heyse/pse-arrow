// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The stage DAG, derived from the declared ports (blueprint §14.1).
//!
//! The driver derives its graph from named stage and port edges; reading a relation schema
//! without an input binding is invalid. That is what stops a pass from consuming whatever
//! artifact happened to be latest.
//!
//! Packet C-4 fills this module.
