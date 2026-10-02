// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Operation identities never certify their actual physical prerequisites.

#![cfg(feature = "fixtures")]

use pse_quantity::{
    IndexSet, Opcode, PhysicalRequirement, QuantityRegistry,
    infer::{InvariantChecker, OpRequest, Operand},
    standard::{StandardInvariantChecker, ids, standard_registry},
};

#[expect(
    clippy::expect_used,
    reason = "test requires the declared physical operation"
)]
fn pressure_operation(registry: &QuantityRegistry) -> &pse_quantity::QuantityOperation {
    registry
        .operations()
        .find(|operation| {
            operation.opcode == Opcode::Div
                && operation.input_kinds == [ids::kind("pressure"), ids::kind("molar_energy")]
        })
        .expect("declared ideal gas density operation")
}

#[test]
fn exact_pressure_origin_is_proved_from_actual_operand_keys() {
    let registry = standard_registry().expect("admitted source projection");
    let operation = pressure_operation(&registry);
    let invariant = operation.precondition_invariants[0];
    let indices = IndexSet::new();
    let mut operands = [
        Operand {
            quantity_type: ids::quantity("pressure.absolute"),
            indices: &indices,
        },
        Operand {
            quantity_type: ids::quantity("molar_energy"),
            indices: &indices,
        },
    ];
    let check = |operands: &[Operand<'_>]| {
        StandardInvariantChecker.check(
            invariant,
            &OpRequest::Div,
            Some(operation),
            operands,
            &registry,
        )
    };
    check(&operands).expect("absolute pressure satisfies actual prerequisite");
    operands[0].quantity_type = ids::quantity("pressure.gauge");
    assert!(
        check(&operands).is_err(),
        "same kind and dimension cannot replace an absolute datum"
    );
    assert!(check(&operands[..1]).is_err());
}

#[test]
fn foreign_operation_clone_and_malformed_predicate_are_refused() {
    let registry = standard_registry().expect("admitted source projection");
    let operation = pressure_operation(&registry);
    let mut declaration = pse_quantity::generated::standard_preconditions()
        .into_iter()
        .find(|value| operation.precondition_invariants.contains(&value.id))
        .expect("actual generated predicate");
    let indices = IndexSet::new();
    let operands = [
        Operand {
            quantity_type: ids::quantity("pressure.absolute"),
            indices: &indices,
        },
        Operand {
            quantity_type: ids::quantity("molar_energy"),
            indices: &indices,
        },
    ];
    declaration
        .check(operation, &operands, &registry)
        .expect("actual admitted operation");
    assert!(
        declaration
            .check(&operation.clone(), &operands, &registry)
            .is_err()
    );
    declaration.operand_positions = vec![0, 0];
    assert!(declaration.validate(&registry).is_err());
    declaration.operand_positions = vec![0];
    declaration.requirement = PhysicalRequirement::EqualOperandBases { required: None };
    assert!(declaration.validate(&registry).is_err());
}

#[test]
fn same_reference_differences_normalize_without_rebasing() {
    let registry = standard_registry().expect("admitted source projection");
    let quantity = |hex| {
        pse_quantity::QuantityTypeId::from_id(
            pse_ids::SemanticId::parse_hex(hex).expect("fixture ID"),
        )
    };
    let stock = quantity("d5bb3d48b9804f2f8d5a6f0a7cadaee8");
    let other = quantity("94b88c5bead5458629c2afc11ffcff20");
    let indices = IndexSet::new();
    let divide = |a, b| {
        pse_quantity::infer::infer_with_evidence(
            &OpRequest::Div,
            &[
                Operand {
                    quantity_type: a,
                    indices: &indices,
                },
                Operand {
                    quantity_type: b,
                    indices: &indices,
                },
            ],
            &registry,
            &StandardInvariantChecker,
        )
    };
    for same in [stock, other] {
        assert_eq!(
            divide(same, same).expect("common actual datum").result,
            registry.neutral_dimensionless().expect("neutral")
        );
    }
    assert!(divide(stock, other).is_err());
    assert!(divide(other, stock).is_err());
    let entropy = quantity("a9c45d0c2b764f4e87d1b95d5bc15da6");
    let entropy_oracle = quantity("01a0e1b4f60f70fdb854e09fec643c3c");
    for same in [entropy, entropy_oracle] {
        assert_eq!(
            divide(same, same).expect("common entropy datum").result,
            registry.neutral_dimensionless().expect("neutral")
        );
    }
    assert!(divide(entropy, entropy_oracle).is_err());
    assert!(divide(entropy_oracle, entropy).is_err());
    assert!(divide(stock, entropy).is_err());
    for wrong in [
        quantity("1831d0d72dc74b299ba8ecb6d4da6f53"), // affine point
        quantity("0f83c0a2bffb4cd09c7f727ae6c42d20"), // component subject
        quantity("571c90b8909e43c0ae859554cc2e3479"), // entropy point
        quantity("e7b2196e91b349f880edbcb227d8ea1c"), // component entropy
    ] {
        assert!(divide(wrong, wrong).is_err());
    }
    let mut declaration = pse_quantity::generated::standard_preconditions()
        .into_iter()
        .find(|v| {
            matches!(
                v.requirement,
                PhysicalRequirement::SameReferenceDifferences { .. }
            )
        })
        .expect("declared family");
    declaration.operand_positions = vec![0];
    assert!(declaration.validate(&registry).is_err());
    declaration.operand_positions = vec![0, 1];
    declaration.requirement = PhysicalRequirement::SameReferenceDifferences {
        required: quantity("1831d0d72dc74b299ba8ecb6d4da6f53"),
        match_shape: false,
    };
    assert!(declaration.validate(&registry).is_err());
}

#[test]
fn actual_contracts_select_disjoint_rules_and_reject_overlap_or_cross_normalization() {
    let registry = standard_registry().expect("admitted source projection");
    let quantity = |hex| {
        pse_quantity::QuantityTypeId::from_id(
            pse_ids::SemanticId::parse_hex(hex).expect("fixture ID"),
        )
    };
    let pressure = quantity("a0fa145fd8b2ecec448c39666960debc");
    let difference = quantity("d3e8173d5c454d8b94388323bb7f05c2");
    let component = quantity("39bebc51897c494a9f7f84bf3962cb0e");
    let mixture = quantity("b6edcb08fc072d81a9f99b45f7f9c90a");
    let indices = IndexSet::new();
    let divide = |registry: &QuantityRegistry, a, b| {
        pse_quantity::infer::infer_with_evidence(
            &OpRequest::Div,
            &[
                Operand {
                    quantity_type: a,
                    indices: &indices,
                },
                Operand {
                    quantity_type: b,
                    indices: &indices,
                },
            ],
            registry,
            &StandardInvariantChecker,
        )
    };
    for (left, right) in [(pressure, difference), (component, mixture)] {
        assert_eq!(
            divide(&registry, left, left)
                .expect("declared left contract")
                .result,
            registry.neutral_dimensionless().expect("neutral")
        );
        assert_eq!(
            divide(&registry, right, right)
                .expect("declared right contract")
                .result,
            registry.neutral_dimensionless().expect("neutral")
        );
        if left == component {
            let result = divide(&registry, left, right).expect("component fraction of mixture");
            assert_eq!(result.result, ids::quantity("mole_fraction"));
            assert!(matches!(result.selected,
                pse_quantity::infer::OperationSelection::Registered { operation, .. }
                if operation == pse_quantity::OperationId::from_id(
                    pse_ids::SemanticId::parse_hex("0c532d7b69bb436b85ff4dcdecd49ad2").unwrap()
                )
            ));
        } else {
            assert!(divide(&registry, left, right).is_err());
        }
        assert!(divide(&registry, right, left).is_err());
    }
    assert!(
        divide(
            &registry,
            ids::quantity("pressure.gauge"),
            ids::quantity("pressure.gauge")
        )
        .is_err()
    );
    let selected = divide(&registry, difference, difference).expect("proved declaration");
    let pse_quantity::infer::OperationSelection::Registered { operation, .. } = selected.selected
    else {
        panic!("normalization must remain declared");
    };
    let mut duplicate = registry
        .operation(operation)
        .expect("selected operation")
        .clone();
    duplicate.id = pse_quantity::OperationId::from_id(pse_ids::SemanticId::from_bytes([239; 16]));
    let mut builder = registry.to_builder();
    builder.operation(duplicate);
    let ambiguous = builder.build().expect("individually valid declarations");
    assert!(matches!(
        divide(&ambiguous, difference, difference),
        Err(pse_quantity::QuantityError::OperationUnsupported {
            ordered_matches: 2,
            ..
        })
    ));
}
