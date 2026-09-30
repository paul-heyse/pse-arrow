// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use crate::reference_translation::{ReferenceTranslation, ScientificAnchor};
use crate::*;
use pse_ids::SemanticId;
use std::collections::BTreeMap;
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn registry(origin: f64) -> QuantityRegistry {
    let pressure =
        DimensionVector::new([-1, 1, -2, 0, 0, 0, 0, 0].map(|n| Ratio::new(n, 1).unwrap()));
    let temperature = DimensionVector::base(BaseDimension::Temperature);
    let enthalpy =
        DimensionVector::new([2, 1, -2, 0, -1, 0, 0, 0].map(|n| Ratio::new(n, 1).unwrap()));
    let mut b = QuantityRegistryBuilder::new();
    for (n, symbol, dimension, scale) in [
        (1, "Pa", pressure, 1.0),
        (2, "K", temperature, 1.0),
        (3, "J/mol", enthalpy, 1.0),
        (4, "kPa", pressure, 1000.0),
        (5, "kJ/mol", enthalpy, 1000.0),
    ] {
        b.unit(Unit {
            id: id(n).into(),
            symbol: symbol.into(),
            dimension,
            scale_to_canonical: scale,
            offset_to_canonical: 0.0,
            is_affine: false,
            reference_state: None,
            definition: None,
        });
    }
    for (n, dimension) in [(10, pressure), (11, temperature), (12, enthalpy)] {
        b.kind(QuantityKind {
            id: id(n).into(),
            dimension,
            extensive: false,
            addition_kind: QuantityAdditionKind::OriginSensitive,
            category: None,
            definition: None,
        });
    }
    for n in [40, 41] {
        b.reference_state(ReferenceState {
            id: id(n).into(),
            name: format!("datum{n}"),
            kind: ReferenceStateKind::Custom,
            temperature: Some(ReferenceCondition {
                value: 300.0,
                quantity_type: id(21).into(),
                unit: id(2).into(),
            }),
            pressure: Some(ReferenceCondition {
                value: if n == 40 { origin } else { 100_000.0 },
                quantity_type: id(20).into(),
                unit: id(1).into(),
            }),
            include_enthalpy_of_formation: n == 40,
            subject: None,
        });
    }
    for (n, kind, unit, reference, scale) in [
        (20, 10, 1, None, ScaleKind::Point),
        (21, 11, 2, None, ScaleKind::Point),
        (22, 10, 1, Some(40), ScaleKind::Point),
        (23, 10, 4, Some(41), ScaleKind::Point),
        (24, 10, 1, Some(40), ScaleKind::Difference),
        (25, 10, 4, None, ScaleKind::Difference),
        (26, 12, 3, Some(40), ScaleKind::Point),
        (27, 12, 5, Some(41), ScaleKind::Point),
    ] {
        b.quantity_type(QuantityType {
            id: id(n).into(),
            name: None,
            key: QuantityTypeKey {
                kind: id(kind).into(),
                basis: None,
                reference_state: reference.map(|n| id(n).into()),
                scale_kind: scale,
                shape: vec![],
                subject_kind: None,
            },
            canonical_unit: id(unit).into(),
            nominal_magnitude: None,
        });
    }
    b.build().unwrap()
}
fn value(r: &QuantityRegistry, q: u8, value: f64) -> CanonicalMagnitude {
    CanonicalConversionPlan::canonical(r, id(q).into())
        .unwrap()
        .apply(value)
        .unwrap()
}
#[test]
fn datum_origins_are_derived_after_units_and_invert_with_changed_reference() {
    for origin in [101_325.0, 98_000.0] {
        let r = registry(origin);
        let gauge = CanonicalConversionPlan::registered(&r, id(22).into(), id(4).into())
            .unwrap()
            .apply(2.0)
            .unwrap();
        let to_absolute = DatumConversionPlan::admit(&r, id(22).into(), id(20).into()).unwrap();
        let absolute = to_absolute.apply(gauge).unwrap();
        assert_eq!(absolute.value(), origin + 2000.0);
        let inverse = DatumConversionPlan::admit(&r, id(20).into(), id(22).into()).unwrap();
        assert_eq!(inverse.apply(absolute).unwrap(), gauge);
        let stock = DatumConversionPlan::admit(&r, id(22).into(), id(23).into()).unwrap();
        assert_eq!(
            stock.apply(gauge).unwrap().value(),
            (origin + 2000.0 - 100_000.0) / 1000.0
        );
        let difference = DatumConversionPlan::admit(&r, id(24).into(), id(25).into()).unwrap();
        assert_eq!(difference.offset(), 0.0);
        assert_eq!(
            difference.apply(value(&r, 24, 2000.0)).unwrap().value(),
            2.0
        );
        assert!(DatumConversionPlan::admit(&r, id(26).into(), id(27).into()).is_err());
    }
}
fn anchors(r: &QuantityRegistry) -> BTreeMap<SemanticId, (ScientificAnchor, ScientificAnchor)> {
    [(60, 10.0, 0.2), (61, 30.0, 0.4)]
        .into_iter()
        .map(|(member, a, b)| {
            let anchor = |q, v| ScientificAnchor {
                value: value(r, q, v),
                temperature: value(r, 21, 300.0),
                pressure: value(r, 20, 100_000.0),
                provenance: vec![id(90)],
            };
            (id(member), (anchor(26, a), anchor(27, b)))
        })
        .collect()
}
#[test]
fn reference_translation_uses_actual_composition_and_canonical_scale() {
    let r = registry(101_325.0);
    let pairs = anchors(&r);
    let translation =
        ReferenceTranslation::admit(&r, id(26).into(), id(27).into(), pairs.clone()).unwrap();
    let x = BTreeMap::from([(id(60), 0.25), (id(61), 0.75)]);
    let y = translation.apply(value(&r, 26, 125.0), &x).unwrap();
    assert!((y.value() - 0.45).abs() < 1e-15);
    let inverse = ReferenceTranslation::admit(
        &r,
        id(27).into(),
        id(26).into(),
        pairs.into_iter().map(|(id, (a, b))| (id, (b, a))).collect(),
    )
    .unwrap();
    assert!((inverse.apply(y, &x).unwrap().value() - 125.0).abs() < 1e-12);
    assert!(
        translation
            .apply(value(&r, 26, 125.0), &BTreeMap::from([(id(60), 1.0)]))
            .is_err()
    );
    assert!(
        translation
            .apply(
                value(&r, 26, 125.0),
                &BTreeMap::from([(id(60), 0.0), (id(61), 0.0)])
            )
            .is_err()
    );
    assert!(translation.apply(value(&r, 27, 0.45), &x).is_err());
}
#[test]
fn reference_translation_refuses_unpaired_conditions_or_missing_provenance() {
    let r = registry(101_325.0);
    let mut pairs = anchors(&r);
    pairs.get_mut(&id(60)).unwrap().1.pressure = value(&r, 20, 101_325.0);
    assert!(ReferenceTranslation::admit(&r, id(26).into(), id(27).into(), pairs).is_err());
    let mut pairs = anchors(&r);
    pairs.get_mut(&id(60)).unwrap().0.provenance.clear();
    assert!(ReferenceTranslation::admit(&r, id(26).into(), id(27).into(), pairs).is_err());
}
