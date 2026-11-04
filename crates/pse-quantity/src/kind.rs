// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Quantity kind declarations (blueprint §6.2), including kinds derived from a monomial of
//! other kinds (ADR-0124).
use crate::{
    BasisId, DimensionVector, EntityKindId, QuantityAdditionKind, QuantityKindCategory,
    QuantityKindId, Ratio, ReferenceStateId, ScaleKind, UnitId,
};
crate::quantity_boundary_record! {
callback: quantity_kind_boundary; name: "PhysicalQuantityKind"; version: 1;
direction: Both; reserved_tags: []; reserved_names: [];
construct: [];
/// What is measured, independent of basis and datum.
#[derive(
    schemars::JsonSchema, serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq,
)]
#[schemars(deny_unknown_fields)]
pub struct QuantityKind {
    /// Registry identity.
    pub id: QuantityKindId => 1;
    /// SI exponents; this alone never resolves a physical type. A derived kind's
    /// dimension is derived from its monomial at admission.
    pub dimension: DimensionVector => 2;
    /// Whether values scale with system size.
    pub extensive: bool => 3;
    /// Additive or origin-sensitive arithmetic.
    pub addition_kind: QuantityAdditionKind => 4;
    /// Discrete category of a dimensionless pure-number kind; `None` for a measured kind.
    pub category: Option<QuantityKindCategory> => 5;
    /// `None` for a base kind; a derived kind's declaration, its monomial expanded to
    /// canonical base-kind factors at admission.
    pub definition: Option<KindDefinition> => 6;
}
}

crate::quantity_boundary_record! {
callback: kind_factor_boundary; name: "KindFactor"; version: 1;
direction: Both; reserved_tags: []; reserved_names: [];
construct: [];
/// One factor of a kind monomial: a kind and its nonzero rational exponent.
#[derive(
    schemars::JsonSchema,
    serde::Serialize,
    serde::Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
#[schemars(deny_unknown_fields)]
pub struct KindFactor {
    /// The factor kind.
    pub kind: QuantityKindId => 1;
    /// Its nonzero rational exponent.
    pub exponent: Ratio => 2;
}
}

crate::quantity_boundary_record! {
callback: kind_definition_boundary; name: "KindDefinition"; version: 1;
direction: Both; reserved_tags: []; reserved_names: [];
construct: [];
/// What a derived kind is (ADR-0124): a monomial of other kinds, its declared canonical
/// unit, and the complete result a multiplicative chain resolving to it takes. Kinds are
/// declared, never synthesized.
#[derive(
    schemars::JsonSchema, serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq,
)]
#[schemars(deny_unknown_fields)]
pub struct KindDefinition {
    /// Factors over declared kinds; admission expands derived factors, so the admitted
    /// monomial is over base kinds in identity order.
    pub monomial: Vec<KindFactor> => 1;
    /// The canonical unit every quantity type of this kind stores in.
    pub canonical_unit: UnitId => 2;
    /// Declared result basis. Without one, every basis-carrying chain factor must agree.
    pub basis: Option<BasisId> => 3;
    /// Declared result datum.
    pub reference_state: Option<ReferenceStateId> => 4;
    /// Declared result scale.
    pub scale_kind: ScaleKind => 5;
    /// Declared result subject.
    pub subject_kind: Option<EntityKindId> => 6;
}
}

pse_contract::projection_struct! {
/// A derived kind as authored: no dimension, which admission derives from the monomial.
#[derive(
    schemars::JsonSchema, serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq,
)]
#[schemars(deny_unknown_fields)]
pub struct DerivedKind {
    projection: { name: "DeclaredDerivedKind", rust_path: crate::DerivedKind, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Registry identity.
    pub id: QuantityKindId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Whether values scale with system size.
    pub extensive: bool => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Additive or origin-sensitive arithmetic.
    pub addition_kind: QuantityAdditionKind => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// The monomial, canonical unit and result policy.
    pub definition: KindDefinition => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
