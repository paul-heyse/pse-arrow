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
