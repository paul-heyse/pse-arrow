// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `Cell` rows to and from `RecordBatch`es (blueprint §4.1).
//!
//! The registry describes itself as rows of [`pse_schema::model::Cell`]; this is where
//! those rows become Arrow arrays under the declared contract, and back. There is no
//! second row model: a struct mirroring a relation is a governance failure.
//!
//! Packet A-7 fills this module.
