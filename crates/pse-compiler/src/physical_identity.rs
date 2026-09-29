// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Canonical framing of actual admitted declarations, never a caller's revision assertion.
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_quantity::{PhysicalPreconditions, QuantityRegistry};
fn id(h: &mut FramedHasher, v: Option<SemanticId>) {
    h.u64(u64::from(v.is_some()));
    if let Some(v) = v {
        h.id(&v);
    }
}
fn number(h: &mut FramedHasher, v: Option<f64>) {
    h.u64(u64::from(v.is_some()));
    if let Some(v) = v {
        h.u64(v.to_bits());
    }
}
fn ratio(h: &mut FramedHasher, v: pse_quantity::Ratio) {
    h.part(&v.num().to_le_bytes()).part(&v.den().to_le_bytes());
}
fn word(h: &mut FramedHasher, v: Option<&str>) {
    h.u64(u64::from(v.is_some()));
    if let Some(v) = v {
        h.str(v);
    }
}
pub(crate) fn identity(r: &QuantityRegistry, p: &PhysicalPreconditions) -> ContentHash {
    let mut h = FramedHasher::new(pse_ids::Frame::MathPhysicalInventoryV4);
    h.str("entity-kinds").u64(r.entity_kinds().count() as u64);
    for kind in r.entity_kinds() {
        h.id(&kind.id.as_id());
    }
    h.str("units").u64(r.units().len() as u64);
    for v in r.units() {
        h.id(&v.id.as_id())
            .str(&v.symbol)
            .part(&v.dimension.canonical_bytes())
            .u64(v.scale_to_canonical.to_bits())
            .u64(v.offset_to_canonical.to_bits())
            .u64(u64::from(v.is_affine));
        id(
            &mut h,
            v.reference_state.map(pse_quantity::ReferenceStateId::as_id),
        );
        // A defined unit's canonical atomic factors (ADR-0124).
        h.u64(u64::from(v.definition.is_some()));
        for f in v.definition.iter().flatten() {
            h.id(&f.unit.as_id());
            ratio(&mut h, f.exponent);
        }
    }
    h.str("kinds").u64(r.kinds().len() as u64);
    for v in r.kinds() {
        h.id(&v.id.as_id())
            .part(&v.dimension.canonical_bytes())
            .u64(u64::from(v.extensive))
            .str(v.addition_kind.as_str());
        word(
            &mut h,
            v.category.map(pse_quantity::QuantityKindCategory::as_str),
        );
        // A derived kind's canonical monomial, canonical unit and result (ADR-0124).
        h.u64(u64::from(v.definition.is_some()));
        if let Some(d) = &v.definition {
            h.u64(d.monomial.len() as u64);
            for f in &d.monomial {
                h.id(&f.kind.as_id());
                ratio(&mut h, f.exponent);
            }
            h.id(&d.canonical_unit.as_id()).str(d.scale_kind.as_str());
            id(&mut h, d.basis.map(pse_quantity::BasisId::as_id));
            id(
                &mut h,
                d.reference_state.map(pse_quantity::ReferenceStateId::as_id),
            );
            id(&mut h, d.subject_kind.map(pse_quantity::EntityKindId::as_id));
        }
    }
    h.str("bases").u64(r.bases().len() as u64);
    for v in r.bases() {
        h.id(&v.id.as_id()).str(v.kind.as_str());
        word(
            &mut h,
            v.composition_basis
                .map(pse_quantity::CompositionBasis::as_str),
        );
        word(&mut h, v.rate_basis.map(pse_quantity::RateBasis::as_str));
        id(
            &mut h,
            v.reference_conditions
                .map(pse_quantity::ReferenceStateId::as_id),
        );
    }
    h.str("references").u64(r.reference_states().len() as u64);
    for v in r.reference_states() {
        h.id(&v.id.as_id())
            .str(v.kind.as_str())
            .u64(u64::from(v.include_enthalpy_of_formation));
        number(&mut h, v.temperature);
        number(&mut h, v.pressure);
        id(&mut h, v.subject);
    }
    h.str("quantities").u64(r.quantity_types().count() as u64);
    for v in r.quantity_types() {
        let k = &v.key;
        h.id(&v.id.as_id())
            .id(&k.kind.as_id())
            .id(&v.canonical_unit.as_id())
            .str(k.scale_kind.as_str());
        id(&mut h, k.basis.map(pse_quantity::BasisId::as_id));
        id(
            &mut h,
            k.reference_state.map(pse_quantity::ReferenceStateId::as_id),
        );
        id(
            &mut h,
            k.subject_kind.map(pse_quantity::EntityKindId::as_id),
        );
        number(&mut h, v.nominal_magnitude);
        h.u64(k.shape.len() as u64);
        for d in &k.shape {
            h.id(&d.as_id());
        }
    }
    h.str("conversions").u64(r.conversions().len() as u64);
    for v in r.conversions() {
        h.id(&v.id.as_id())
            .id(&v.from.as_id())
            .id(&v.to.as_id())
            .str(v.kind.as_str());
        id(&mut h, v.kernel);
        number(&mut h, v.scale);
        number(&mut h, v.offset);
        h.u64(v.required_parameters.len() as u64);
        for n in &v.required_parameters {
            h.str(n);
        }
    }
    h.str("operations").u64(r.operations().len() as u64);
    for v in r.operations() {
        h.id(&v.id.as_id())
            .str(v.opcode.as_str())
            .id(&v.result_kind.as_id())
            .str(v.basis_rule.as_str())
            .str(v.reference_rule.as_str())
            .str(v.scale_rule.as_str())
            .str(v.shape_rule.as_str())
            .str(v.subject_rule.as_str());
        for n in [
            v.basis_source,
            v.reference_source,
            v.scale_source,
            v.shape_source,
            v.subject_source,
        ] {
            h.u64(n.map_or(u64::MAX, u64::from));
        }
        id(&mut h, v.result_basis.map(pse_quantity::BasisId::as_id));
        id(
            &mut h,
            v.result_reference_state
                .map(pse_quantity::ReferenceStateId::as_id),
        );
        id(
            &mut h,
            v.result_subject_kind.map(pse_quantity::EntityKindId::as_id),
        );
        h.u64(v.input_kinds.len() as u64);
        for n in &v.input_kinds {
            h.id(&n.as_id());
        }
        h.u64(v.input_conversions.len() as u64);
        for n in &v.input_conversions {
            h.u64(n.operand.into()).id(&n.conversion.as_id());
        }
        h.u64(v.precondition_invariants.len() as u64);
        for n in &v.precondition_invariants {
            h.id(&n.as_id());
        }
    }
    h.str("reductions");
    for (op, d) in r.reduction_domains() {
        h.id(&op.as_id()).id(&d.as_id());
    }
    h.str("unit-sets").u64(r.unit_sets().len() as u64);
    for v in r.unit_sets() {
        h.id(&v.id.as_id());
        for n in v.base {
            id(&mut h, n.map(pse_quantity::UnitId::as_id));
        }
    }
    id(
        &mut h,
        r.neutral_dimensionless()
            .map(pse_quantity::QuantityTypeId::as_id),
    );
    h.str("preconditions").u64(p.declarations().len() as u64);
    for v in p.declarations() {
        h.id(&v.id.as_id()).u64(v.operand_positions.len() as u64);
        for n in &v.operand_positions {
            h.u64((*n).into());
        }
        match v.requirement {
            pse_quantity::PhysicalRequirement::EqualOperandBases { required } => {
                h.str("equal-bases");
                id(&mut h, required.map(pse_quantity::BasisId::as_id));
            }
            pse_quantity::PhysicalRequirement::OperandQuantityContract {
                required,
                match_shape,
            } => {
                h.str("quantity")
                    .id(&required.as_id())
                    .u64(u64::from(match_shape));
            }
            pse_quantity::PhysicalRequirement::SameReferenceDifferences {
                required,
                match_shape,
            } => {
                h.str("same-reference-differences")
                    .id(&required.as_id())
                    .u64(u64::from(match_shape));
            }
        }
    }
    h.finish_hash()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_quantity::*;

    fn registry(declared_basis: bool) -> QuantityRegistry {
        let raw = |n: u8| SemanticId::from_bytes([n; 16]);
        let length = DimensionVector::base(BaseDimension::Length);
        let mass = DimensionVector::base(BaseDimension::Mass);
        let unit = |n: u8, dimension| Unit {
            id: UnitId::from_id(raw(n)),
            symbol: format!("u{n}"),
            dimension,
            scale_to_canonical: 1.0,
            offset_to_canonical: 0.0,
            is_affine: false,
            reference_state: None,
            definition: None,
        };
        let kind = |n: u8, dimension| QuantityKind {
            id: QuantityKindId::from_id(raw(n)),
            dimension,
            extensive: false,
            addition_kind: QuantityAdditionKind::Additive,
            category: None,
            definition: None,
        };
        let molar = BasisId::from_id(raw(20));
        let mut b = QuantityRegistryBuilder::new();
        b.unit(unit(1, length.mul(&mass).unwrap()))
            .kind(kind(10, length))
            .kind(kind(11, mass))
            .basis(Basis {
                id: molar,
                kind: BasisKind::Molar,
                composition_basis: None,
                rate_basis: None,
                reference_conditions: None,
            })
            .derived_kind(DerivedKind {
                id: QuantityKindId::from_id(raw(12)),
                extensive: false,
                addition_kind: QuantityAdditionKind::Additive,
                definition: KindDefinition {
                    monomial: [10, 11]
                        .map(|n| KindFactor {
                            kind: QuantityKindId::from_id(raw(n)),
                            exponent: Ratio::ONE,
                        })
                        .to_vec(),
                    canonical_unit: UnitId::from_id(raw(1)),
                    basis: declared_basis.then_some(molar),
                    reference_state: None,
                    scale_kind: ScaleKind::Point,
                    subject_kind: None,
                },
            })
            .quantity_type(QuantityType {
                id: QuantityTypeId::from_id(raw(30)),
                key: QuantityTypeKey {
                    kind: QuantityKindId::from_id(raw(12)),
                    basis: Some(molar),
                    reference_state: None,
                    scale_kind: ScaleKind::Point,
                    shape: vec![],
                    subject_kind: None,
                },
                canonical_unit: UnitId::from_id(raw(1)),
                nominal_magnitude: None,
            });
        b.build().unwrap()
    }

    /// The frozen preimage layout of `pse.math.physical-inventory.v4` over an empty
    /// inventory, and its sensitivity to a derived kind's definition (ADR-0124).
    #[test]
    fn physical_inventory_identity_frames_derived_definitions() {
        let none = PhysicalPreconditions::new(vec![]).unwrap();
        let empty = QuantityRegistryBuilder::new().build().unwrap();
        assert_eq!(
            identity(&empty, &none).to_hex(),
            "506fbeee3aae5b9c684b8aecd134f4be5b44eebe795e82ac2f569334e712251f"
        );
        // Two inventories that differ only in a derived kind's declared result basis.
        assert_ne!(identity(&registry(false), &none), identity(&registry(true), &none));
        assert_eq!(identity(&registry(true), &none), identity(&registry(true), &none));
    }
}
