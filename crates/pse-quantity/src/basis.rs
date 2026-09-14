// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Basis declarations (blueprint §6.2, §8.4).
use crate::{BasisId, BasisKind, CompositionBasis, RateBasis, ReferenceStateId};
/// A declared material or specific-quantity basis.
#[derive(Clone, Debug)]
#[expect(
    clippy::struct_field_names,
    reason = "fields retain the reference.bases contract names"
)]
pub struct Basis {
    /// Registry identity.
    pub id: BasisId,
    /// What this quantity is per.
    pub kind: BasisKind,
    /// Optional composition convention.
    pub composition_basis: Option<CompositionBasis>,
    /// Optional rate convention.
    pub rate_basis: Option<RateBasis>,
    /// Conditions required for a standard-volume basis.
    pub reference_conditions: Option<ReferenceStateId>,
}
