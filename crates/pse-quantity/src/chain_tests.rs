// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Multiplicative chains resolved against declared kinds (ADR-0124, Plan 23 KR2).

use crate::infer::{Chain, NoInvariantFacts, OpRequest, Operand, OperationSelection, infer_chain};
use crate::*;
use pse_ids::SemanticId;

fn raw(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn r(num: i32, den: i32) -> Ratio {
    Ratio::new(num, den).unwrap()
}
fn leaf(quantity_type: QuantityTypeId, indices: &IndexSet) -> Box<Chain<'_>> {
    Box::new(Chain::Leaf(Operand {
        quantity_type,
        indices,
    }))
}
fn mul<'a>(a: Box<Chain<'a>>, b: Box<Chain<'a>>) -> Box<Chain<'a>> {
    Box::new(Chain::Mul(a, b))
}
fn div<'a>(a: Box<Chain<'a>>, b: Box<Chain<'a>>) -> Box<Chain<'a>> {
    Box::new(Chain::Div(a, b))
}
fn pow<'a>(base: Box<Chain<'a>>, exponent: i32, scalar: Operand<'a>) -> Box<Chain<'a>> {
    Box::new(Chain::Pow {
        base,
        exponent: r(exponent, 1),
        power: scalar,
    })
}

// ------------------------------------------------------------------ synthetic fixture --

const A: u8 = 10;
const B: u8 = 11;
const PRODUCT: u8 = 12;
const DECLARED_BASIS: u8 = 13;
const RULE_RESULT: u8 = 14;
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
fn basis_must_agree_or_be_declared() {
    let registry = builder().build().unwrap();
    let scalar = IndexSet::new();
    let product = |a, b| {
        infer_chain(
            &mul(leaf(a, &scalar), leaf(b, &scalar)),
            &registry,
            &NoInvariantFacts,
        )
    };
    // Equal bases: the derived kind's result takes the factors' basis.
    let inferred = product(q(40), q(42)).unwrap();
    assert_eq!(inferred.result, q(43));
    assert_eq!(
        inferred.selected,
        OperationSelection::BuiltIn(infer::BuiltInRule::Chain)
    );
    // A mass-basis and a molar-basis factor disagree, and nothing is declared.
    assert!(matches!(
        product(q(41), q(42)),
        Err(QuantityError::Incompatible {
            reason: IncompatibilityReason::BasisMismatch,
            ..
        })
    ));
    // A derived kind that declares its basis takes it whatever the factors carry. Its
    // monomial A·B·B², written over another derived kind, expands to A·B³.
    let mut declared = builder();
    declared
        .unit(unit(6, "ab3", dimension(1, 3)))
        .derived_kind(DerivedKind {
            id: QuantityKindId::from_id(raw(DECLARED_BASIS)),
            extensive: false,
            addition_kind: QuantityAdditionKind::Additive,
            definition: KindDefinition {
                monomial: vec![
                    KindFactor {
                        kind: QuantityKindId::from_id(raw(PRODUCT)),
                        exponent: Ratio::ONE,
                    },
                    KindFactor {
                        kind: QuantityKindId::from_id(raw(B)),
                        exponent: r(2, 1),
                    },
                ],
                canonical_unit: UnitId::from_id(raw(6)),
                ..definition(Some(MOLAR))
            },
        })
        .quantity_type(ty(45, DECLARED_BASIS, Some(MOLAR), 6));
    let registry = declared.build().unwrap();
    let chain = mul(
        mul(leaf(q(41), &scalar), leaf(q(42), &scalar)),
        mul(leaf(q(42), &scalar), leaf(q(42), &scalar)),
    );
    assert_eq!(
        infer_chain(&chain, &registry, &NoInvariantFacts)
            .unwrap()
            .result,
        q(45)
    );
}

#[test]
fn undeclared_monomial_is_refused_with_factors() {
    let registry = builder().build().unwrap();
    let scalar = IndexSet::new();
    // A²·B names no declared kind, and none is synthesized.
    let chain = mul(
        mul(leaf(q(40), &scalar), leaf(q(40), &scalar)),
        leaf(q(42), &scalar),
    );
    let error = infer_chain(&chain, &registry, &NoInvariantFacts).unwrap_err();
    let QuantityError::UndeclaredMonomial { factors } = &error else {
        panic!("{error}");
    };
    assert_eq!(
        factors,
        &vec![
            KindFactor {
                kind: QuantityKindId::from_id(raw(A)),
                exponent: r(2, 1),
            },
            KindFactor {
                kind: QuantityKindId::from_id(raw(B)),
                exponent: Ratio::ONE,
            },
        ]
    );
    let message = error.to_string();
    assert!(
        message.contains(&format!("{}^2", QuantityKindId::from_id(raw(A)))),
        "{message}"
    );
    assert!(
        message.contains(&format!("{}^1", QuantityKindId::from_id(raw(B)))),
        "{message}"
    );
    // A cancelled monomial names no declared kind either.
    let cancelled = div(leaf(q(40), &scalar), leaf(q(40), &scalar));
    assert!(matches!(
        infer_chain(&cancelled, &registry, &NoInvariantFacts),
        Err(QuantityError::UndeclaredMonomial { factors }) if factors.is_empty()
    ));
}

#[test]
fn rule_and_chain_disagreement_is_refused() {
    // A registered rule types A·B as another kind of the same dimension; the chain types
    // it as the declared derived kind. Neither route takes precedence.
    let mut b = builder();
    b.kind(kind(RULE_RESULT, dimension(1, 1)))
        .quantity_type(ty(46, RULE_RESULT, Some(MOLAR), 3))
        .operation(QuantityOperation {
            id: OperationId::from_id(raw(50)),
            opcode: Opcode::Mul,
            input_kinds: vec![
                QuantityKindId::from_id(raw(A)),
                QuantityKindId::from_id(raw(B)),
            ],
            result_kind: QuantityKindId::from_id(raw(RULE_RESULT)),
            basis_rule: BasisRule::Preserve,
            reference_rule: ReferenceRule::RequireEqual,
            scale_rule: QuantityScaleRule::Point,
            shape_rule: QuantityShapeRule::SameIndices,
            basis_source: Some(0),
            reference_source: None,
            scale_source: None,
            shape_source: None,
            subject_rule: SubjectRule::RequireEqual,
            subject_source: None,
            result_basis: None,
            result_reference_state: None,
            result_subject_kind: None,
            input_conversions: vec![],
            precondition_invariants: vec![],
        });
    let registry = b.build().unwrap();
    let scalar = IndexSet::new();
    let chain = mul(leaf(q(40), &scalar), leaf(q(42), &scalar));
    match infer_chain(&chain, &registry, &NoInvariantFacts) {
        Err(QuantityError::RouteDisagreement { registered, chain }) => {
            assert_eq!((registered, chain), (q(46), q(43)));
        }
        other => panic!("{other:?}"),
    }
    // The single-operation request alone is typed by the registered rule, unchanged.
    let operands = [
        Operand {
            quantity_type: q(40),
            indices: &scalar,
        },
        Operand {
            quantity_type: q(42),
            indices: &scalar,
        },
    ];
    assert_eq!(
        infer::infer(&OpRequest::Mul, &operands, &registry)
            .unwrap()
            .result,
        q(46)
    );
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

// ----------------------------------------------------------- the physical document --

#[cfg(feature = "fixtures")]
mod standard {
    use super::*;
    use crate::standard::{StandardInvariantChecker, standard_registry};

    fn id(hex: &str) -> QuantityTypeId {
        QuantityTypeId::from_id(SemanticId::parse_hex(hex).unwrap())
    }
    const MOLAR_CP: &str = "cd653ba98fa94d16b5d66b363f21c3d6";
    const TEMPERATURE: &str = "c64b96975a4a59755f8711d3bf628bc9";
    const DELTA_T: &str = "459a933fd00837bbc50372e31ac9801c";
    const DELTA_H: &str = "d5bb3d48b9804f2f8d5a6f0a7cadaee8";
    const MOLAR_ENTHALPY: &str = "1831d0d72dc74b299ba8ecb6d4da6f53";
    const GAUGE_PRESSURE: &str = "89248e0d0a55e73fe378d8af87a26900";
    const PRESSURE: &str = "a0fa145fd8b2ecec448c39666960debc";
    const MOLE_FRACTION: &str = "4f842bb63da53a798dca2764c2c64ece";
    const SCALAR: &str = "dc255c612cf27e30cb835377c8dafcf4";

    /// The type of the declared coefficient kind MolarCp/Temperature^power.
    fn coefficient(registry: &QuantityRegistry, power: i32) -> QuantityTypeId {
        use crate::scheme::{Scheme, Substitution};
        Scheme::Quotient(
            Box::new(Scheme::Concrete(id(MOLAR_CP))),
            Box::new(Scheme::Power(
                Box::new(Scheme::Concrete(id(TEMPERATURE))),
                r(power, 1),
            )),
        )
        .resolve_with_evidence(registry, &Substitution::new(), &StandardInvariantChecker)
        .unwrap()
    }

    #[test]
    fn eligible_leaves_are_exactly_true_zero_ratio_points_and_differences() {
        let registry = standard_registry().unwrap();
        let scalar = IndexSet::new();
        let c2 = coefficient(&registry, 2);
        // Each factor multiplies a coefficient; no rule types the product, so only its
        // eligibility decides whether the chain is refused before its monomial is looked up.
        for (factor, eligible) in [
            (MOLAR_CP, true),        // point of an additive kind, no datum
            (TEMPERATURE, true),     // absolute temperature: true-zero ratio point
            (PRESSURE, true),        // absolute pressure: true-zero ratio point
            (DELTA_T, true),         // difference
            (DELTA_H, true),         // difference with a datum
            (MOLE_FRACTION, true),   // dimensionless
            (MOLAR_ENTHALPY, false), // point with a nonzero datum
            (GAUGE_PRESSURE, false), // point with the gauge datum
        ] {
            let chain = mul(leaf(c2, &scalar), leaf(id(factor), &scalar));
            let result = infer_chain(&chain, &registry, &StandardInvariantChecker);
            let refused = matches!(result, Err(QuantityError::ChainLeafIneligible { .. }));
            assert_eq!(!refused, eligible, "{factor}: {result:?}");
        }
    }

    #[test]
    fn nonzero_datum_leaf_is_refused_with_its_factor() {
        let registry = standard_registry().unwrap();
        let scalar = IndexSet::new();
        let c3 = coefficient(&registry, 3);
        let temperature = id(TEMPERATURE);
        let neutral = Operand {
            quantity_type: id(SCALAR),
            indices: &scalar,
        };
        // c3·T³ alone resolves; with a datum-bearing enthalpy point the whole chain is
        // refused, never partly resolved, and the refusal names that factor.
        let increment = mul(
            leaf(c3, &scalar),
            pow(leaf(temperature, &scalar), 3, neutral),
        );
        assert!(infer_chain(&increment, &registry, &StandardInvariantChecker).is_ok());
        for datum in [MOLAR_ENTHALPY, GAUGE_PRESSURE] {
            let chain = mul(
                mul(
                    leaf(c3, &scalar),
                    pow(leaf(temperature, &scalar), 3, neutral),
                ),
                leaf(id(datum), &scalar),
            );
            let error = infer_chain(&chain, &registry, &StandardInvariantChecker).unwrap_err();
            assert!(
                matches!(error, QuantityError::ChainLeafIneligible { factor } if factor == id(datum)),
                "{error}"
            );
            assert!(error.to_string().contains(datum), "{error}");
        }
    }

    #[test]
    fn registered_rules_and_chains_agree_on_the_caloric_product() {
        // The registered rule types MolarCp·ΔT as DeltaH; the declared molar enthalpy kind
        // types the same monomial as DeltaH. They agree, so the rule's evidence is kept.
        let registry = standard_registry().unwrap();
        let scalar = IndexSet::new();
        let chain = mul(leaf(id(MOLAR_CP), &scalar), leaf(id(DELTA_T), &scalar));
        let inferred = infer_chain(&chain, &registry, &StandardInvariantChecker).unwrap();
        assert_eq!(inferred.result, id(DELTA_H));
        assert!(matches!(
            inferred.selected,
            OperationSelection::Registered { .. }
        ));
        // With a temperature point the rule's precondition refuses; the declared kind
        // still types the increment.
        let point = mul(leaf(id(MOLAR_CP), &scalar), leaf(id(TEMPERATURE), &scalar));
        assert_eq!(
            infer_chain(&point, &registry, &StandardInvariantChecker)
                .unwrap()
                .result,
            id(DELTA_H)
        );
    }
}
