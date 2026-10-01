// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physically typed finite process definitions lowered directly to library mathematics.

pub mod typed_math;

mod physical_identity;
/// Incremental semantic preparation and compiler-owned artifact requests.
pub mod workspace;

#[cfg(test)]
mod authored_transfer_tests;
#[cfg(test)]
mod contextual_contract_tests;
#[cfg(test)]
mod physical_potential_tests;

#[cfg(test)]
mod scientific_witness_tests;
