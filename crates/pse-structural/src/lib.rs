// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Incidence, bipartite matching, Dulmage-Mendelsohn, SCC/BTD, degrees of freedom and tear selection (blueprint §3.2, §15).
//!
//! `petgraph` leaves intra-component order arbitrary; every result is sorted by
//! semantic ID before it becomes a relation (blueprint §15.3).
//!
//! Phase 0: this crate is a declared boundary with no implementation yet.
