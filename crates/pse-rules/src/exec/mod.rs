// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The fixed-point executor (blueprint §14.2 rules 1-3).
//!
//! Strata run in order; within a stratum, rules reach a least fixed point. `unknown` and
//! `conflict` outcomes go to `inferred.undecided`, never to a head relation, so no consumer
//! can mistake an undecided fact for a true one.
//!
//! Packet C-3 fills this module.
