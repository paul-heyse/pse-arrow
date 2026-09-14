// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Which species are valid in which phase (blueprint §6.4, §9.1).
//!
//! Filled by packet Q-material: `species_valid_in_phase` as a four-valued
//! predicate minus conflict, feeding `inferred.phase_species`.
