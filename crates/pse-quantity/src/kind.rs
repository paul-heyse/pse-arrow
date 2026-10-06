// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Quantity kind declarations (blueprint §6.2), including kinds derived from a monomial of
//! other kinds (ADR-0124).
use crate::{
    BasisId, DimensionVector, EntityKindId, QuantityAdditionKind, QuantityKindCategory,
    QuantityKindId, Ratio, ReferenceStateId, ScaleKind, UnitId,
};
/// What is measured, independent of basis and datum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuantityKind {
    /// Registry identity.
    pub id: QuantityKindId,
    /// SI exponents; this alone never resolves a physical type. A derived kind's
    /// dimension is derived from its monomial at admission.
    pub dimension: DimensionVector,
    /// Whether values scale with system size.
    pub extensive: bool,
    /// Additive or origin-sensitive arithmetic.
    pub addition_kind: QuantityAdditionKind,
    /// Discrete category of a dimensionless pure-number kind; `None` for a measured kind.
    pub category: Option<QuantityKindCategory>,
    /// `None` for a base kind; a derived kind's declaration, its monomial expanded to
    /// canonical base-kind factors at admission.
    pub definition: Option<KindDefinition>,
}

/// One factor of a kind monomial: a kind and its nonzero rational exponent.
#[derive(
    serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct KindFactor {
    /// The factor kind.
    pub kind: QuantityKindId,
    /// Its nonzero rational exponent.
    pub exponent: Ratio,
}

/// What a derived kind is (ADR-0124): a monomial of other kinds, its declared canonical
/// unit, and the complete result a multiplicative chain resolving to it takes. Kinds are
/// declared, never synthesized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KindDefinition {
    /// Factors over declared kinds; admission expands derived factors, so the admitted
    /// monomial is over base kinds in identity order.
    pub monomial: Vec<KindFactor>,
    /// The canonical unit every quantity type of this kind stores in.
    pub canonical_unit: UnitId,
    /// Declared result basis. Without one, every basis-carrying chain factor must agree.
    pub basis: Option<BasisId>,
    /// Declared result datum.
    pub reference_state: Option<ReferenceStateId>,
    /// Declared result scale.
    pub scale_kind: ScaleKind,
    /// Declared result subject.
    pub subject_kind: Option<EntityKindId>,
}

/// A derived kind as authored: no dimension, which admission derives from the monomial.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DerivedKind {
    /// Registry identity.
    pub id: QuantityKindId,
    /// Whether values scale with system size.
    pub extensive: bool,
    /// Additive or origin-sensitive arithmetic.
    pub addition_kind: QuantityAdditionKind,
    /// The monomial, canonical unit and result policy.
    pub definition: KindDefinition,
}
