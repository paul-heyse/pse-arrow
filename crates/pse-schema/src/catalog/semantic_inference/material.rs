// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual material membership is a declared join over finite member inventories.
use super::{RegistryBuilder, assertion};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    assertion(
        builder,
        "inferred.phase_species",
        "phase_species_assertions",
    );
}
