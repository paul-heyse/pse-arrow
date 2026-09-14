// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The registered exact-predicate subset and its pure pushdown verdict, evaluated with
//! DataFusion physical expressions and Arrow `filter_record_batch` (blueprint §5.4,
//! ADR-0048).
//!
//! Packet B-providers fills this.
