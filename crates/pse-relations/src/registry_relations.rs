// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The registry materialized as batches (blueprint §4.1).
//!
//! `reference.schema_*`, `reference.pass_*` and `reference.rule_*` as `RecordBatch`es, so
//! that the platform can query its own schema through the same catalog as everything else
//! rather than through a side channel that could disagree with it.
//!
//! Packet A-7 fills this module.
