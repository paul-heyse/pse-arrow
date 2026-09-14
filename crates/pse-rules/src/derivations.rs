// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! `provenance.derivations` rows (blueprint §14.2 rule 4).
//!
//! Every head row carries a derivation at the granularity the registry declares for the
//! head relation: `row` where negative completeness is the deliverable, `rule` where the
//! derivation is the rule plus the identity formula.
//!
//! Packet C-3 fills this module.
