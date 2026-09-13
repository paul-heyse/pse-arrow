// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Structural algorithm tests against reference fixtures (blueprint §24.1, §15).
//!
//! Matching, the Dulmage-Mendelsohn partition, SCC order and connected components
//! agree with reference fixtures generated once from Pyomo incidence analysis on
//! the IDAES tutorials. `petgraph` leaves intra-component order arbitrary, so the
//! fixtures compare semantic-ID-sorted output, never petgraph's own ordering
//! (blueprint §15.3).
//!
//! Phase 0: the layer is declared and its harness runs; the tests arrive with the
//! crates they exercise.
