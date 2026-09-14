// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Composed physical typing checks with scoped invariant verification.
use pse_ids::SemanticId;
use pse_quantity::infer::{
    InvariantChecker, OpRequest, Operand, OperationSelection, infer, infer_with_evidence,
};
use pse_quantity::literal::{LiteralContext, resolve_literal};
use pse_quantity::*;
fn raw(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn qty(n: u8) -> QuantityTypeId {
    QuantityTypeId::from_id(raw(n))
}
#[expect(
    clippy::expect_used,
    reason = "test fixture must fail immediately when its declared contracts cannot be built"
)]
#[expect(
    clippy::too_many_lines,
    reason = "the fixture declares all coupled unit-kind-type-operation rows together"
)]
fn fixture(extra_rules: usize) -> QuantityRegistry {
    let temperature = DimensionVector::base(BaseDimension::Temperature);
    let squared = temperature
        .pow(Ratio::new(2, 1).expect("two"))
        .expect("squared");
    let mut b = QuantityRegistryBuilder::new();
    b.unit(Unit {
        id: UnitId::from_id(raw(1)),
        symbol: "K".into(),
        dimension: temperature,
        scale_to_canonical: 1.0,
        offset_to_canonical: 0.0,
        is_affine: false,
        reference_state: None,
    });
    b.unit(Unit {
        id: UnitId::from_id(raw(2)),
        symbol: "1".into(),
        dimension: DimensionVector::DIMENSIONLESS,
        scale_to_canonical: 1.0,
        offset_to_canonical: 0.0,
        is_affine: false,
        reference_state: None,
    });
    b.kind(QuantityKind {
        id: QuantityKindId::from_id(raw(1)),
        dimension: temperature,
        extensive: false,
        addition_kind: QuantityAdditionKind::OriginSensitive,
    });
    b.kind(QuantityKind {
        id: QuantityKindId::from_id(raw(2)),
        dimension: DimensionVector::DIMENSIONLESS,
        extensive: false,
        addition_kind: QuantityAdditionKind::Additive,
    });
    b.kind(QuantityKind {
        id: QuantityKindId::from_id(raw(3)),
        dimension: squared,
        extensive: false,
        addition_kind: QuantityAdditionKind::Additive,
    });
    b.unit(Unit {
        id: UnitId::from_id(raw(3)),
        symbol: "K2".into(),
        dimension: squared,
        scale_to_canonical: 1.0,
        offset_to_canonical: 0.0,
        is_affine: false,
        reference_state: None,
    });
    let key = QuantityTypeKey {
        kind: QuantityKindId::from_id(raw(1)),
        basis: None,
        reference_state: None,
        scale_kind: ScaleKind::Point,
        shape: vec![],
        subject_kind: None,
    };
    for (id, scale_kind) in [(1, ScaleKind::Point), (2, ScaleKind::Difference)] {
        b.quantity_type(QuantityType {
            id: qty(id),
            key: QuantityTypeKey {
                scale_kind,
                ..key.clone()
            },
            canonical_unit: UnitId::from_id(raw(1)),
            nominal_magnitude: None,
        });
    }
    b.quantity_type(QuantityType {
        id: qty(3),
        key: QuantityTypeKey {
            kind: QuantityKindId::from_id(raw(2)),
            ..key.clone()
        },
        canonical_unit: UnitId::from_id(raw(2)),
        nominal_magnitude: None,
    })
    .neutral_dimensionless(qty(3));
    b.quantity_type(QuantityType {
        id: qty(4),
        key: QuantityTypeKey {
            kind: QuantityKindId::from_id(raw(3)),
            ..key
        },
        canonical_unit: UnitId::from_id(raw(3)),
        nominal_magnitude: None,
    });
    for n in 0..extra_rules {
        b.operation(QuantityOperation {
            id: OperationId::from_id(raw(u8::try_from(10 + n).expect("few rules"))),
            opcode: Opcode::Mul,
            input_kinds: vec![QuantityKindId::from_id(raw(1)); 2],
            result_kind: QuantityKindId::from_id(raw(3)),
            basis_rule: BasisRule::RequireEqual,
            reference_rule: ReferenceRule::RequireEqual,
            scale_rule: QuantityScaleRule::Point,
            shape_rule: QuantityShapeRule::SameIndices,
            basis_source: None,
            reference_source: None,
            scale_source: None,
            shape_source: None,
            subject_rule: SubjectRule::RequireEqual,
            subject_source: None,
            result_subject_kind: None,
            result_basis: None,
            result_reference_state: None,
            input_conversions: vec![],
            precondition_invariants: vec![],
        });
    }
    b.build().expect("registry")
}
fn operand(ty: u8, indices: &IndexSet) -> Operand<'_> {
    Operand {
        quantity_type: qty(ty),
        indices,
    }
}
#[test]
fn point_difference_algebra_and_neutral_scaling_keep_full_type() {
    let registry = fixture(0);
    let scalar = IndexSet::new();
    assert_eq!(
        infer(
            &OpRequest::Add,
            &[operand(1, &scalar), operand(2, &scalar)],
            &registry
        )
        .expect("T+deltaT")
        .result,
        qty(1)
    );
    assert_eq!(
        infer(
            &OpRequest::Add,
            &[operand(2, &scalar), operand(2, &scalar)],
            &registry
        )
        .expect("deltaT+deltaT")
        .result,
        qty(2)
    );
    assert_eq!(
        infer(
            &OpRequest::Sub,
            &[operand(1, &scalar), operand(1, &scalar)],
            &registry
        )
        .expect("T-T")
        .result,
        qty(2)
    );
    assert!(
        infer(
            &OpRequest::Add,
            &[operand(1, &scalar), operand(1, &scalar)],
            &registry
        )
        .is_err()
    );
    assert!(
        infer(
            &OpRequest::Sub,
            &[operand(2, &scalar), operand(1, &scalar)],
            &registry
        )
        .is_err()
    );
    let scaled = infer(
        &OpRequest::Mul,
        &[operand(3, &scalar), operand(2, &scalar)],
        &registry,
    )
    .expect("alpha deltaT");
    assert_eq!(scaled.result, qty(2));
    assert_eq!(
        infer(
            &OpRequest::Add,
            &[
                operand(1, &scalar),
                Operand {
                    quantity_type: scaled.result,
                    indices: &scaled.indices
                }
            ],
            &registry
        )
        .expect("T+alpha deltaT")
        .result,
        qty(1)
    );
}
#[test]
fn missing_and_ambiguous_compositions_never_use_dimension_only_fallback() {
    let scalar = IndexSet::new();
    let operands = [operand(2, &scalar), operand(2, &scalar)];
    assert!(matches!(
        infer(&OpRequest::Mul, &operands, &fixture(0)),
        Err(QuantityError::OperationUnsupported {
            ordered_matches: 0,
            ..
        })
    ));
    let result = infer(&OpRequest::Mul, &operands, &fixture(1)).expect("one declared rule");
    assert_eq!(result.result, qty(4));
    assert!(matches!(
        result.selected,
        OperationSelection::Registered { .. }
    ));
    assert!(matches!(
        infer(&OpRequest::Mul, &operands, &fixture(2)),
        Err(QuantityError::OperationUnsupported {
            ordered_matches: 2,
            ..
        })
    ));
}
#[test]
fn literal_occurrences_resolve_point_and_difference_independently() {
    let registry = fixture(0);
    let kelvin = UnitId::from_id(raw(1));
    assert!(matches!(
        resolve_literal(kelvin, LiteralContext::Free, &registry),
        Err(QuantityError::AmbiguousLiteral { .. })
    ));
    assert_eq!(
        resolve_literal(
            kelvin,
            LiteralContext::Explicit {
                quantity_type: qty(1)
            },
            &registry
        )
        .expect("point occurrence"),
        qty(1)
    );
    assert_eq!(
        resolve_literal(
            kelvin,
            LiteralContext::Explicit {
                quantity_type: qty(2)
            },
            &registry
        )
        .expect("difference occurrence"),
        qty(2)
    );
}
#[test]
fn certified_weighted_mean_requires_scoped_actual_weight_check() {
    struct Weights {
        values: Vec<f64>,
    }
    impl InvariantChecker for Weights {
        fn check(
            &self,
            id: InvariantId,
            request: &OpRequest<'_>,
            operation: Option<&QuantityOperation>,
            operands: &[Operand<'_>],
            registry: &QuantityRegistry,
        ) -> Result<(), QuantityError> {
            let valid = id == InvariantId::from_id(raw(9))
                && matches!(
                    request,
                    OpRequest::WeightedMean {
                        normalization: WeightNormalization::CertifiedUnitSum,
                        ..
                    }
                )
                && operation.is_none()
                && operands.len() == self.values.len() * 2
                && self.values.iter().all(|x| x.is_finite())
                && self.values.iter().sum::<f64>().to_bits() == 1.0_f64.to_bits()
                && operands
                    .iter()
                    .step_by(2)
                    .all(|x| registry.neutral_dimensionless() == Some(x.quantity_type));
            if valid {
                Ok(())
            } else {
                Err(QuantityError::InferencePrecondition {
                    rule: "test.actual_unit_sum",
                    detail: "actual weight sum/context is invalid".into(),
                })
            }
        }
    }
    let registry = fixture(0);
    let scalar = IndexSet::new();
    let operands = [
        operand(3, &scalar),
        operand(1, &scalar),
        operand(3, &scalar),
        operand(1, &scalar),
    ];
    let request = OpRequest::WeightedMean {
        normalization: WeightNormalization::CertifiedUnitSum,
        certified_invariant: Some(InvariantId::from_id(raw(9))),
    };
    assert!(infer(&request, &operands, &registry).is_err());
    assert!(
        infer_with_evidence(
            &request,
            &operands,
            &registry,
            &Weights {
                values: vec![0.5, 0.75]
            }
        )
        .is_err()
    );
    assert_eq!(
        infer_with_evidence(
            &request,
            &operands,
            &registry,
            &Weights {
                values: vec![0.25, 0.75]
            }
        )
        .expect("actual sum one")
        .result,
        qty(1)
    );
}

#[test]
fn registered_integral_removes_only_its_actual_lexical_binder() {
    let original = fixture(1);
    let mut builder = original.to_builder();
    let mut body = original.quantity_type(qty(1)).unwrap().clone();
    body.id = qty(5);
    body.key.shape = vec![DomainKind::Custom];
    builder.quantity_type(body);
    let mut operation = original.operations_for(Opcode::Mul).next().unwrap().clone();
    operation.id = OperationId::from_id(raw(40));
    operation.opcode = Opcode::Integral;
    operation.input_kinds.truncate(1);
    operation.shape_rule = QuantityShapeRule::ReduceBoundIndex;
    builder.operation(operation);
    let registry = builder.build().unwrap();
    let bound = BoundIndexRef::new(
        BoundIndexId::from_id(raw(50)),
        DomainId::from_id(raw(51)),
        DomainKind::Custom,
    );
    let indices = IndexSet::try_from_iter([bound]).unwrap();
    let request = OpRequest::Integral {
        domain_unit: UnitId::from_id(raw(1)),
        bound,
    };
    let result = infer(&request, &[operand(5, &indices)], &registry).unwrap();
    assert_eq!(result.result, qty(4));
    assert!(result.indices.is_empty());
    assert!(matches!(
        result.selected,
        OperationSelection::Registered { .. }
    ));
    for wrong in [
        BoundIndexRef {
            bound_index: BoundIndexId::from_id(raw(52)),
            ..bound
        },
        BoundIndexRef {
            domain: DomainId::from_id(raw(52)),
            ..bound
        },
    ] {
        assert!(
            infer(
                &OpRequest::Integral {
                    domain_unit: UnitId::from_id(raw(1)),
                    bound: wrong
                },
                &[operand(5, &indices)],
                &registry
            )
            .is_err()
        );
    }
    assert!(
        infer(
            &OpRequest::Integral {
                domain_unit: UnitId::from_id(raw(2)),
                bound
            },
            &[operand(5, &indices)],
            &registry
        )
        .is_err()
    );
}
