// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete physical contracts and anonymous intermediate admission (Plan 25a A2).

use crate::infer::{NoInvariantFacts, OpRequest};
use crate::*;
use pse_ids::SemanticId;

fn raw(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn r(num: i32, den: i32) -> Ratio {
    Ratio::new(num, den).unwrap()
}
// ------------------------------------------------------------------ synthetic fixture --

const A: u8 = 10;
const B: u8 = 11;
const PRODUCT: u8 = 12;
const MOLAR: u8 = 20;
const MASS: u8 = 21;
const NEUTRAL: u8 = 30;

fn dimension(length: i32, mass: i32) -> DimensionVector {
    DimensionVector::base(BaseDimension::Length)
        .pow(r(length, 1))
        .unwrap()
        .mul(
            &DimensionVector::base(BaseDimension::Mass)
                .pow(r(mass, 1))
                .unwrap(),
        )
        .unwrap()
}
fn unit(n: u8, symbol: &str, dimension: DimensionVector) -> Unit {
    Unit {
        id: UnitId::from_id(raw(n)),
        symbol: symbol.into(),
        dimension,
        scale_to_canonical: 1.0,
        offset_to_canonical: 0.0,
        is_affine: false,
        reference_state: None,
        definition: None,
    }
}
fn kind(n: u8, dimension: DimensionVector) -> QuantityKind {
    QuantityKind {
        id: QuantityKindId::from_id(raw(n)),
        dimension,
        extensive: false,
        addition_kind: QuantityAdditionKind::Additive,
        category: None,
        definition: None,
    }
}
fn ty(id: u8, kind: u8, basis: Option<u8>, unit: u8) -> QuantityType {
    QuantityType {
        id: QuantityTypeId::from_id(raw(id)),
        name: None,
        key: QuantityTypeKey {
            kind: QuantityKindId::from_id(raw(kind)),
            basis: basis.map(|b| BasisId::from_id(raw(b))),
            reference_state: None,
            scale_kind: ScaleKind::Point,
            shape: vec![],
            subject_kind: None,
        },
        canonical_unit: UnitId::from_id(raw(unit)),
        nominal_magnitude: None,
    }
}
fn definition(declared_basis: Option<u8>) -> KindDefinition {
    KindDefinition {
        monomial: vec![
            KindFactor {
                kind: QuantityKindId::from_id(raw(A)),
                exponent: Ratio::ONE,
            },
            KindFactor {
                kind: QuantityKindId::from_id(raw(B)),
                exponent: Ratio::ONE,
            },
        ],
        canonical_unit: UnitId::from_id(raw(3)),
        basis: declared_basis.map(|b| BasisId::from_id(raw(b))),
        reference_state: None,
        scale_kind: ScaleKind::Point,
        subject_kind: None,
    }
}
/// Base kinds A (length) and B (mass), types of A in molar and mass basis and of B in
/// molar basis, and the derived kind A·B with its molar result type.
fn builder() -> QuantityRegistryBuilder {
    let mut b = QuantityRegistryBuilder::new();
    b.unit(unit(1, "a", dimension(1, 0)))
        .unit(unit(2, "b", dimension(0, 1)))
        .unit(unit(3, "ab", dimension(1, 1)))
        .unit(unit(4, "one", DimensionVector::DIMENSIONLESS))
        .kind(kind(A, dimension(1, 0)))
        .kind(kind(B, dimension(0, 1)))
        .kind(kind(NEUTRAL, DimensionVector::DIMENSIONLESS))
        .basis(Basis {
            id: BasisId::from_id(raw(MOLAR)),
            kind: BasisKind::Molar,
            composition_basis: None,
            rate_basis: None,
            reference_conditions: None,
        })
        .basis(Basis {
            id: BasisId::from_id(raw(MASS)),
            kind: BasisKind::Mass,
            composition_basis: None,
            rate_basis: None,
            reference_conditions: None,
        })
        .derived_kind(DerivedKind {
            id: QuantityKindId::from_id(raw(PRODUCT)),
            extensive: false,
            addition_kind: QuantityAdditionKind::Additive,
            definition: definition(None),
        })
        .quantity_type(ty(40, A, Some(MOLAR), 1))
        .quantity_type(ty(41, A, Some(MASS), 1))
        .quantity_type(ty(42, B, Some(MOLAR), 2))
        .quantity_type(ty(43, PRODUCT, Some(MOLAR), 3))
        .quantity_type(ty(44, NEUTRAL, None, 4))
        .neutral_dimensionless(QuantityTypeId::from_id(raw(44)));
    b
}
fn q(n: u8) -> QuantityTypeId {
    QuantityTypeId::from_id(raw(n))
}

#[test]
fn anonymous_contracts_retain_factors_until_named_boundary() {
    let registry = builder().build().unwrap();
    let a = ResolvedPhysicalContract::named(q(40), IndexSet::new(), &registry).unwrap();
    let b = ResolvedPhysicalContract::named(q(42), IndexSet::new(), &registry).unwrap();
    let square = resolved::infer_operation(&OpRequest::Mul, &[a.clone(), a.clone()], None, &registry, &NoInvariantFacts).unwrap();
    let admitted = resolved::infer_operation(&OpRequest::Mul, &[square.result, b], None, &registry, &NoInvariantFacts).unwrap();
    assert_eq!(admitted.result.named_id(), None);
    assert_eq!(admitted.result.kind_factors(), &[
        KindFactor {kind: raw(A).into(), exponent:r(2,1)}, KindFactor {kind:raw(B).into(),exponent:Ratio::ONE},
    ]);
    assert_eq!(admitted.result.qualified_factors().len(), 2);
    assert!(admitted.result.at_boundary(q(43), &registry).is_err());
    let completed = resolved::infer_operation(&OpRequest::Div, &[admitted.result, a], None, &registry, &NoInvariantFacts).unwrap();
    assert_eq!(completed.result.require_named().unwrap(), q(43));
}

#[test]
fn qualified_cancellation_keeps_free_binders_and_refuses_wrong_basis() {
    let mut b = builder();
    let kind = EntityKindId::from_id(raw(70));
    b.entity_kind(EntityKind {id:kind,name:"component".into()});
    let mut indexed = ty(48,A,Some(MOLAR),1); indexed.key.shape = vec![kind]; b.quantity_type(indexed);
    let registry = b.build().unwrap();
    let bound = BoundIndexRef::new(raw(71).into(),raw(72).into(),kind);
    let indices = IndexSet::try_from_iter([bound]).unwrap();
    let value = ResolvedPhysicalContract::named(q(48),indices.clone(),&registry).unwrap();
    let cancelled = resolved::infer_operation(&OpRequest::Div,&[value.clone(),value],None,&registry,&NoInvariantFacts).unwrap();
    assert!(cancelled.result.is_pure_number());
    assert_eq!(cancelled.result.indices(),&indices);
    assert_eq!(cancelled.result.axes(),&[kind]);
    assert_eq!(cancelled.result.named_id(),None);
    let values = [q(40), q(41)].map(|id| ResolvedPhysicalContract::named(id, IndexSet::new(), &registry).unwrap());
    assert!(resolved::infer_operation(&OpRequest::Div, &values, None, &registry, &NoInvariantFacts).is_err());
    assert_eq!(resolved::infer_operation(&OpRequest::Div, &[values[0].clone(), values[0].clone()], None, &registry, &NoInvariantFacts).unwrap().result.require_named().unwrap(), q(44));
}

#[test]
fn intermediate_coordinates_have_an_explicit_result_scale() {
    let mut b = builder();
    b.unit(Unit { scale_to_canonical: 1000.0, ..unit(5,"ka",dimension(1,0)) });
    b.quantity_type(ty(49,A,None,5));
    let registry = b.build().unwrap();
    let left=ResolvedPhysicalContract::named(q(49),IndexSet::new(),&registry).unwrap();
    let right=ResolvedPhysicalContract::named(q(42),IndexSet::new(),&registry).unwrap();
    let admitted=resolved::infer_operation(&OpRequest::Mul,&[left,right],Some(q(43)),&registry,&NoInvariantFacts).unwrap();
    assert_eq!(admitted.result.require_named().unwrap(),q(43));
    assert_eq!(admitted.result_scale,1000.0);
}

#[test]
fn scientific_response_sum_requires_scoped_authority_and_retains_both_terms() {
    let mut b = builder();
    b.kind(kind(80, dimension(1,0))).quantity_type(ty(81,80,Some(MOLAR),1));
    let registry=b.build().unwrap();
    let values=[q(40),q(81)].map(|id|ResolvedPhysicalContract::named(id,IndexSet::new(),&registry).unwrap());
    assert!(resolved::infer_operation(&OpRequest::Add,&values,None,&registry,&NoInvariantFacts).is_err());
    let authority=PhysicalFormulaAuthority::response(raw(82));
    let sum=resolved::infer_in_context(&OpRequest::Add,&values,None,&registry,&NoInvariantFacts,Some(&authority)).unwrap();
    assert_eq!(sum.operands,values);
    assert_eq!(sum.result.named_id(),None);
    assert_eq!(sum.result.qualified_factors().len(),2);
    assert!(sum.result.at_boundary(q(40),&registry).is_err());
    let wrong=ResolvedPhysicalContract::named(q(41),IndexSet::new(),&registry).unwrap();
    assert!(resolved::infer_in_context(&OpRequest::Add,&[wrong,values[1].clone()],None,&registry,&NoInvariantFacts,Some(&authority)).is_err());
}

#[test]
fn canonical_coordinates_reject_affine_storage_and_retain_difference_scale() {
    let mut b = builder();
    let mut affine = unit(85, "shifted_a", dimension(1, 0));
    affine.scale_to_canonical = 2.0;
    affine.offset_to_canonical = 10.0;
    affine.is_affine = true;
    let mut invalid = builder();
    invalid.unit(affine.clone()).quantity_type(ty(86, A, None, 85));
    assert!(matches!(invalid.build(), Err(QuantityError::Registry { rule: "quantity_type.canonical_unit", .. })));
    affine.is_affine = false; affine.offset_to_canonical = 0.0;
    let mut origin = kind(88, dimension(1, 0));
    origin.addition_kind = QuantityAdditionKind::OriginSensitive;
    b.kind(origin).unit(affine).quantity_type(ty(86, 88, None, 85));
    let mut difference = ty(87, 88, None, 1);
    difference.key.scale_kind = ScaleKind::Difference;
    b.quantity_type(difference);
    let registry = b.build().unwrap();
    let point = ResolvedPhysicalContract::named(q(86), IndexSet::new(), &registry).unwrap();
    let delta = resolved::infer_operation(&OpRequest::Sub, &[point.clone(), point.clone()], None,
        &registry, &NoInvariantFacts).unwrap();
    assert_eq!(delta.result.require_named().unwrap(), q(87));
    assert_eq!(delta.operand_scales, [2.0, 2.0]);
    let derivative = resolved::infer_partial(&point, &[point.clone()], &registry, &NoInvariantFacts).unwrap();
    assert!(derivative.result.is_pure_number());
    assert_eq!(derivative.result_scale, 1.0);
}

#[test]
fn unlike_subjects_and_affine_points_cannot_cancel_through_anonymous_algebra() {
    let mut b=builder();
    for (n,name) in [(70,"component"),(71,"reaction")] {
        b.entity_kind(EntityKind{id:raw(n).into(),name:name.into()});
        let mut value=ty(n+10,A,Some(MOLAR),1);value.key.subject_kind=Some(raw(n).into());b.quantity_type(value);
    }
    b.reference_state(ReferenceState{id:raw(90).into(),name:"datum".into(),kind:ReferenceStateKind::Custom,
        temperature:None,pressure:None,include_enthalpy_of_formation:false,subject:None});
    let mut point=ty(91,A,Some(MOLAR),1);point.key.reference_state=Some(raw(90).into());b.quantity_type(point);
    let registry=b.build().unwrap();
    for (request, ids) in [(OpRequest::Div, [q(80),q(81)]), (OpRequest::Mul, [q(91),q(42)]), (OpRequest::Div, [q(91),q(40)])] {
        let values = ids.map(|id| ResolvedPhysicalContract::named(id, IndexSet::new(), &registry).unwrap());
        assert!(resolved::infer_operation(&request, &values, None, &registry, &NoInvariantFacts).is_err());
    }
}

#[cfg(feature="fixtures")]
#[test]
fn registered_refusal_cannot_be_rescued_by_a_derived_kind() {
    use crate::standard::{StandardInvariantChecker,standard_registry};
    let registry=standard_registry().unwrap();
    let cp=QuantityTypeId::from_id(SemanticId::parse_hex("cd653ba98fa94d16b5d66b363f21c3d6").unwrap());
    let point=QuantityTypeId::from_id(SemanticId::parse_hex("c64b96975a4a59755f8711d3bf628bc9").unwrap());
    let values = [cp, point].map(|id| ResolvedPhysicalContract::named(id, IndexSet::new(), &registry).unwrap());
    assert!(resolved::infer_operation(&OpRequest::Mul, &values, None, &registry, &StandardInvariantChecker).is_err());
}
#[test]
fn derived_kinds_are_admitted_acyclic_unique_and_derived() {
    let registry = builder().build().unwrap();
    let product = registry
        .kind(QuantityKindId::from_id(raw(PRODUCT)))
        .unwrap();
    assert_eq!(product.dimension, dimension(1, 1));
    assert_eq!(
        registry.kind_by_monomial(&product.definition.as_ref().unwrap().monomial),
        Some(product.id)
    );
    let derived = |id: u8, monomial: Vec<(u8, i32)>| DerivedKind {
        id: QuantityKindId::from_id(raw(id)),
        extensive: false,
        addition_kind: QuantityAdditionKind::Additive,
        definition: KindDefinition {
            monomial: monomial
                .into_iter()
                .map(|(kind, exponent)| KindFactor {
                    kind: QuantityKindId::from_id(raw(kind)),
                    exponent: r(exponent, 1),
                })
                .collect(),
            ..definition(None)
        },
    };
    for (rule, kinds) in [
        (
            "quantity_kind.definition_cycle",
            vec![
                derived(60, vec![(61, 1)]),
                derived(61, vec![(60, 1), (A, 1)]),
            ],
        ),
        (
            "quantity_kind.monomial_unique",
            vec![derived(60, vec![(B, 1), (A, 1)])],
        ),
        (
            "quantity_kind.definition_alias",
            vec![derived(60, vec![(A, 1)])],
        ),
        (
            "quantity_kind.pure_number_factor",
            vec![derived(60, vec![(A, 1), (NEUTRAL, 1)])],
        ),
    ] {
        let mut b = builder();
        for kind in kinds {
            b.derived_kind(kind);
        }
        match b.build() {
            Err(QuantityError::Registry { rule: actual, .. }) => assert_eq!(actual, rule),
            other => panic!("{rule}: {other:?}"),
        }
    }
    // A type of a derived kind stores in its declared canonical unit.
    let mut b = builder();
    b.quantity_type(ty(47, PRODUCT, Some(MASS), 1));
    assert!(b.build().is_err());
}
