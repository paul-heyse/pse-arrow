// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete physical type keys (blueprint §6.2, §8.1).
use crate::registry::QuantityRegistry;
use crate::{
    BasisId, DomainKind, QuantityKindId, QuantityTypeId, ReferenceStateId, ScaleKind, SubjectKind,
    UnitId,
};
/// Components that determine one registered physical type.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct QuantityTypeKey {
    /// The measured quantity kind.
    pub kind: QuantityKindId,
    /// Optional material basis.
    pub basis: Option<BasisId>,
    /// Optional datum.
    pub reference_state: Option<ReferenceStateId>,
    /// Point or difference semantics.
    pub scale_kind: ScaleKind,
    /// Ordered domain kinds.
    pub shape: Vec<DomainKind>,
    /// Optional subject obligation; absent and explicit `none` stay distinct.
    pub subject_kind: Option<SubjectKind>,
}
/// A registered physical type and its canonical storage unit.
#[derive(Clone, Debug)]
pub struct QuantityType {
    /// Registry identity.
    pub id: QuantityTypeId,
    /// The complete semantic key.
    pub key: QuantityTypeKey,
    /// Canonical unit of the package.
    pub canonical_unit: UnitId,
    /// Positive finite default scaling magnitude, if declared.
    pub nominal_magnitude: Option<f64>,
}
impl QuantityType {
    /// Whether the package explicitly designated this complete type as neutral.
    pub fn is_neutral_scalar(&self, registry: &QuantityRegistry) -> bool {
        registry.neutral_dimensionless() == Some(self.id)
    }
}
