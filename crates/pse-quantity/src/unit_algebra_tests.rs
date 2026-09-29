// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Unit algebra: composed literals, atomic and defined units (ADR-0124, Plan 23 KR1).

use crate::literal::{LiteralContext, resolve_literal};
use crate::*;
use pse_ids::SemanticId;

fn raw(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn uid(n: u8) -> UnitId {
    UnitId::from_id(raw(n))
}
fn r(num: i32, den: i32) -> Ratio {
    Ratio::new(num, den).unwrap()
}
fn factor(n: u8, num: i32) -> UnitFactor {
    UnitFactor {
        unit: uid(n),
        exponent: r(num, 1),
    }
}
fn product(factors: &[(&str, i32, i32)]) -> UnitProduct {
    UnitProduct::from_factors(
        factors
            .iter()
            .map(|(symbol, num, den)| ((*symbol).to_owned(), r(*num, *den))),
    )
    .unwrap()
}
fn energy() -> DimensionVector {
    let base = DimensionVector::base;
    base(BaseDimension::Mass)
        .mul(&base(BaseDimension::Length).pow(r(2, 1)).unwrap())
        .unwrap()
        .div(&base(BaseDimension::Time).pow(r(2, 1)).unwrap())
        .unwrap()
}
fn pressure() -> DimensionVector {
    let base = DimensionVector::base;
    base(BaseDimension::Mass)
        .div(&base(BaseDimension::Length))
        .unwrap()
        .div(&base(BaseDimension::Time).pow(r(2, 1)).unwrap())
        .unwrap()
}
fn molar_heat_capacity() -> DimensionVector {
    energy()
        .div(&DimensionVector::base(BaseDimension::Amount))
        .unwrap()
        .div(&DimensionVector::base(BaseDimension::Temperature))
        .unwrap()
}
fn atomic(n: u8, symbol: &str, dimension: DimensionVector, scale: f64) -> Unit {
    Unit {
        id: uid(n),
        symbol: symbol.into(),
        dimension,
        scale_to_canonical: scale,
        offset_to_canonical: 0.0,
        is_affine: false,
        reference_state: None,
        definition: None,
    }
}
/// The defined `J/(K*mol)`: `J` 1, `K` 2, `mol` 3.
fn molar_cp_unit() -> DefinedUnit {
    let composition = vec![factor(1, 1), factor(2, -1), factor(3, -1)];
    DefinedUnit {
        id: unit_product_id(&canonical(composition.clone())),
        symbol: "J/(K*mol)".into(),
        composition,
    }
}
fn canonical(factors: Vec<UnitFactor>) -> Vec<UnitFactor> {
    unit::canonical_factors(factors).unwrap()
}
const GAUGE: u8 = 40;
const MOLAR_CP: u8 = 50;

/// Atomic J, K, mol, kJ, kmol, m, s, Pa, degC (affine) and psig (datum-restricted), the
/// defined `J/(K*mol)` as the canonical unit of a molar heat capacity type, and nothing
/// else composite.
fn builder() -> QuantityRegistryBuilder {
    let base = DimensionVector::base;
    let mut b = QuantityRegistryBuilder::new();
    b.unit(atomic(1, "J", energy(), 1.0))
        .unit(atomic(2, "K", base(BaseDimension::Temperature), 1.0))
        .unit(atomic(3, "mol", base(BaseDimension::Amount), 1.0))
        .unit(atomic(4, "kJ", energy(), 1000.0))
        .unit(atomic(5, "kmol", base(BaseDimension::Amount), 1000.0))
        .unit(atomic(6, "m", base(BaseDimension::Length), 1.0))
        .unit(atomic(7, "s", base(BaseDimension::Time), 1.0))
        .unit(atomic(8, "Pa", pressure(), 1.0))
        .unit(Unit {
            offset_to_canonical: 273.15,
            is_affine: true,
            ..atomic(9, "degC", base(BaseDimension::Temperature), 1.0)
        })
        .unit(Unit {
            reference_state: Some(ReferenceStateId::from_id(raw(GAUGE))),
            ..atomic(10, "psig", pressure(), 6894.757_293_168)
        })
        .reference_state(ReferenceState {
            id: ReferenceStateId::from_id(raw(GAUGE)),
            name: "gauge".into(),
            kind: ReferenceStateKind::Custom,
            temperature: None,
            pressure: None,
            include_enthalpy_of_formation: false,
            subject: None,
        })
        .defined_unit(molar_cp_unit())
        .kind(QuantityKind {
            id: QuantityKindId::from_id(raw(MOLAR_CP)),
            dimension: molar_heat_capacity(),
            extensive: false,
            addition_kind: QuantityAdditionKind::Additive,
            category: None,
            definition: None,
        })
        .quantity_type(QuantityType {
            id: QuantityTypeId::from_id(raw(MOLAR_CP)),
            name: None,
            key: QuantityTypeKey {
                kind: QuantityKindId::from_id(raw(MOLAR_CP)),
                basis: None,
                reference_state: None,
                scale_kind: ScaleKind::Point,
                shape: vec![],
                subject_kind: None,
            },
            canonical_unit: molar_cp_unit().id,
            nominal_magnitude: None,
        });
    b
}

#[test]
fn composite_literal_needs_no_registered_whole_unit() {
    let registry = builder().build().unwrap();
    let molar_cp = QuantityTypeId::from_id(raw(MOLAR_CP));
    // kJ/(kmol*K) is registered nowhere; it composes from atomic factors.
    let literal = registry
        .compose(&product(&[("kJ", 1, 1), ("kmol", -1, 1), ("K", -1, 1)]))
        .unwrap();
    assert!(registry.unit(literal.id).is_err());
    assert_eq!(literal.dimension, molar_heat_capacity());
    assert_eq!(literal.scale_to_canonical.to_bits(), 1.0_f64.to_bits());
    assert_eq!(
        resolve_literal(
            &literal,
            LiteralContext::Explicit {
                quantity_type: molar_cp
            },
            &registry
        )
        .unwrap(),
        molar_cp
    );
    assert_eq!(
        resolve_literal(&literal, LiteralContext::Free, &registry).unwrap(),
        molar_cp
    );
    let canonical = registry
        .unit(registry.quantity_type(molar_cp).unwrap().canonical_unit)
        .unwrap();
    let conversion =
        convert_spec_for_type(&literal, canonical, &registry.quantity_type(molar_cp).unwrap().key)
            .unwrap();
    assert_eq!(convert_value(&conversion, 75.3).to_bits(), 75.3_f64.to_bits());
    // J/(mol*K) composes to the defined canonical unit itself.
    let spelled = registry
        .compose(&product(&[("J", 1, 1), ("mol", -1, 1), ("K", -1, 1)]))
        .unwrap();
    assert_eq!(&spelled, canonical);
    // A defined unit is spelled by its composition, never by its row name.
    assert!(matches!(
        registry.compose(&UnitProduct::symbol("J/(K*mol)")),
        Err(QuantityError::UnknownUnitSymbol { .. })
    ));
}

#[test]
fn affine_unit_only_as_sole_factor() {
    let registry = builder().build().unwrap();
    let celsius = registry.compose(&UnitProduct::symbol("degC")).unwrap();
    assert_eq!(celsius.id, uid(9));
    assert_eq!(celsius.offset_to_canonical.to_bits(), 273.15_f64.to_bits());
    let gauge = registry.compose(&UnitProduct::symbol("psig")).unwrap();
    assert_eq!(gauge.reference_state, Some(ReferenceStateId::from_id(raw(GAUGE))));
    for refused in [
        product(&[("degC", 1, 1), ("s", -1, 1)]),
        product(&[("degC", 2, 1)]),
        product(&[("degC", -1, 1)]),
        product(&[("psig", 1, 1), ("m", 2, 1)]),
        product(&[("psig", 1, 2)]),
    ] {
        assert!(
            matches!(
                registry.compose(&refused),
                Err(QuantityError::AffineUnitFactor { .. })
            ),
            "{refused}"
        );
    }
    assert!(registry.compose(&product(&[("K", 1, 1), ("s", -1, 1)])).is_ok());
    // A defined unit cannot take an affine or datum-restricted factor either.
    for bad in [9, 10] {
        let composition = vec![factor(bad, 1), factor(7, -1)];
        let mut b = builder();
        b.defined_unit(DefinedUnit {
            id: unit_product_id(&canonical(composition.clone())),
            symbol: "bad".into(),
            composition,
        });
        assert!(
            matches!(
                b.build(),
                Err(QuantityError::Registry {
                    rule: "unit.affine_factor",
                    ..
                })
            ),
            "{bad}"
        );
    }
}

#[test]
fn defined_unit_dimension_and_scale_are_derived() {
    // Defined kJ/kmol and kJ/mol author their composition only.
    let per_kmol = vec![factor(4, 1), factor(5, -1)];
    let per_mol = vec![factor(4, 1), factor(3, -1)];
    // A defined unit over a defined unit: m2, then J/m2.
    let square = vec![factor(6, 2)];
    let square_id = unit_product_id(&canonical(square.clone()));
    let per_area = vec![
        factor(1, 1),
        UnitFactor {
            unit: square_id,
            exponent: r(-1, 1),
        },
    ];
    let per_area_atomic = canonical(vec![factor(1, 1), factor(6, -2)]);
    let mut b = builder();
    for (symbol, composition) in [
        ("kJ/kmol", per_kmol.clone()),
        ("kJ/mol", per_mol.clone()),
        ("m^2", square),
    ] {
        b.defined_unit(DefinedUnit {
            id: unit_product_id(&canonical(composition.clone())),
            symbol: symbol.into(),
            composition,
        });
    }
    b.defined_unit(DefinedUnit {
        id: unit_product_id(&per_area_atomic),
        symbol: "J/m^2".into(),
        composition: per_area,
    });
    let registry = b.build().unwrap();
    let molar_energy = energy()
        .div(&DimensionVector::base(BaseDimension::Amount))
        .unwrap();
    let kj_per_kmol = registry
        .unit(unit_product_id(&canonical(per_kmol.clone())))
        .unwrap();
    assert_eq!(kj_per_kmol.dimension, molar_energy);
    assert_eq!(kj_per_kmol.scale_to_canonical.to_bits(), 1.0_f64.to_bits());
    assert_eq!(kj_per_kmol.definition, Some(canonical(per_kmol)));
    let kj_per_mol = registry
        .unit(unit_product_id(&canonical(per_mol)))
        .unwrap();
    assert_eq!(kj_per_mol.scale_to_canonical.to_bits(), 1000.0_f64.to_bits());
    let per_area = registry.unit(unit_product_id(&per_area_atomic)).unwrap();
    assert_eq!(per_area.definition.as_deref(), Some(per_area_atomic.as_slice()));
    assert_eq!(
        per_area.dimension,
        energy()
            .div(&DimensionVector::base(BaseDimension::Length).pow(r(2, 1)).unwrap())
            .unwrap()
    );
    // The spelling of a defined unit composes back to the same unit.
    for unit in registry.units() {
        let spelled = registry.unit_product(unit.id).unwrap();
        assert_eq!(&registry.compose(&spelled).unwrap(), unit, "{spelled}");
    }
    assert_eq!(
        registry.unit_product(per_area.id).unwrap().to_string(),
        "J/m^2"
    );
    // The declared identity must be the product identity; aliases, cycles and a
    // registered spelling of the empty product as an atomic `1` are refused.
    let refusals: [(&str, Box<dyn Fn(&mut QuantityRegistryBuilder)>); 4] = [
        (
            "unit.defined_identity",
            Box::new(|b| {
                b.defined_unit(DefinedUnit {
                    id: uid(90),
                    symbol: "kJ/kmol".into(),
                    composition: vec![factor(4, 1), factor(5, -1)],
                });
            }),
        ),
        (
            "unit.defined_alias",
            Box::new(|b| {
                b.defined_unit(DefinedUnit {
                    id: uid(94),
                    symbol: "kelvin".into(),
                    composition: vec![factor(2, 1)],
                });
            }),
        ),
        (
            "unit.defined_cycle",
            Box::new(|b| {
                b.defined_unit(DefinedUnit {
                    id: uid(91),
                    symbol: "a".into(),
                    composition: vec![factor(92, 1)],
                })
                .defined_unit(DefinedUnit {
                    id: uid(92),
                    symbol: "b".into(),
                    composition: vec![factor(91, 2)],
                });
            }),
        ),
        (
            "unit.symbol",
            Box::new(|b| {
                b.unit(atomic(93, "1", DimensionVector::DIMENSIONLESS, 1.0));
            }),
        ),
    ];
    for (rule, refuse) in refusals {
        let mut b = builder();
        refuse(&mut b);
        match b.build() {
            Err(QuantityError::Registry { rule: actual, .. }) => assert_eq!(actual, rule),
            other => panic!("{rule}: {other:?}"),
        }
    }
    // The empty product is a defined unit with no factors.
    let mut b = builder();
    b.defined_unit(DefinedUnit {
        id: unit_product_id(&[]),
        symbol: "1".into(),
        composition: vec![],
    });
    let registry = b.build().unwrap();
    let one = registry.compose(&UnitProduct::one()).unwrap();
    assert_eq!(one.id, unit_product_id(&[]));
    assert!(one.dimension.is_dimensionless());
    assert!(registry.unit(one.id).is_ok());
}

#[test]
fn unit_product_identity_is_the_frozen_vector() {
    // `pse-ids` freezes the same parts under `pse.quantity.unit-product.v1`.
    assert_eq!(
        unit_product_id(&[factor(1, 1), factor(2, -1)])
            .as_id()
            .to_hex(),
        "b65b26cc780fb634de036e1195b0af54"
    );
    assert_eq!(unit_product_id(&[factor(1, 1)]), uid(1));
}
