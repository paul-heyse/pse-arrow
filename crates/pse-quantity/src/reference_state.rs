// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reference state declarations (blueprint §6.2, §8.4).
use crate::{ReferenceStateId, ReferenceStateKind};
use pse_ids::SemanticId;
/// The datum carried by origin-sensitive quantities.
#[derive(Clone, Debug)]
pub struct ReferenceState {
    /// Registry identity.
    pub id: ReferenceStateId,
    /// The reference convention.
    pub kind: ReferenceStateKind,
    /// Reference temperature in K, when specified.
    pub temperature: Option<f64>,
    /// Reference pressure in Pa, when specified.
    pub pressure: Option<f64>,
    /// Whether formation enthalpy is included.
    pub include_enthalpy_of_formation: bool,
    /// Optional phase identity; material admission checks this foreign key.
    pub phase: Option<SemanticId>,
}
