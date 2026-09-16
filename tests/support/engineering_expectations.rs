// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Independently stated outcomes for the shipped single-phase nitrogen examples.

use pse_catalog::Snapshot;
use pse_ids::SemanticId;
use pse_relations::generated::{compiled, enums::SymbolRole, normalized};

fn id(value: &str) -> SemanticId {
    SemanticId::parse_hex(value).unwrap()
}

pub(crate) fn check(
    canonical: &Snapshot,
    instances: &[normalized::instance_bindings::Row],
    state_template: SemanticId,
    component_flow_state: bool,
    heater: bool,
) {
    check_equations(
        canonical,
        instances,
        state_template,
        component_flow_state,
        heater,
    );
    check_symbols(
        canonical,
        instances,
        state_template,
        component_flow_state,
        heater,
    );
}

fn check_equations(
    canonical: &Snapshot,
    instances: &[normalized::instance_bindings::Row],
    state_template: SemanticId,
    component_flow_state: bool,
    heater: bool,
) {
    let equations = compiled::math_indexed_equations::View::from_checked(
        canonical
            .relation("compiled", "math_indexed_equations")
            .unwrap()
            .checked(),
    )
    .unwrap()
    .rows()
    .unwrap();
    let by_declaration = |declaration: &str| {
        equations
            .iter()
            .filter(|row| row.equation_decl_id == Some(id(declaration)))
            .collect::<Vec<_>>()
    };
    let closure = by_declaration(if component_flow_state {
        "c2c19be5d5204d80ab99802de54fa4cc"
    } else {
        "809bcfe6144e4283a80b4affe686fa10"
    });
    assert_eq!(
        closure.len(),
        1,
        "only the outlet state needs composition closure"
    );
    let outlet = instances
        .iter()
        .find(|row| {
            row.template_id == state_template
                && row.submodel_name.as_deref()
                    == Some(if heater {
                        "properties_out"
                    } else {
                        "mixed_state"
                    })
        })
        .unwrap();
    assert_eq!(closure[0].owner_instance_id, outlet.instance_id);
    let states = instances
        .iter()
        .filter(|row| row.template_id == state_template)
        .collect::<Vec<_>>();
    assert_eq!(
        by_declaration("f6479c28e09e4ec1bc5d83f2d9e6a38e").len(),
        if component_flow_state {
            states.len()
        } else {
            0
        },
        "FcTP defines each component flow through its state composition"
    );
    assert_eq!(
        by_declaration(if heater {
            "7075234cdb2c4a86868e9e93b4974fc3"
        } else {
            "805b27eb8d654b70bc5e5f7bd8f1f9c3"
        })
        .len(),
        1,
        "one scalar or inlet-indexed pressure equation"
    );
    for disabled in [
        "85eedc866a5a406183e8c5d5c9e2445f",
        "d21198887b734bdaa6d975d784eb6001",
    ] {
        assert!(
            by_declaration(disabled).is_empty(),
            "default balances have neither pressure drop nor temperature equality"
        );
    }
    assert_eq!(
        equations
            .iter()
            .filter(|row| row.law_instance_id.is_some())
            .count(),
        2,
        "one material and one enthalpy conservation equation for single-species nitrogen"
    );
    assert!(
        equations
            .iter()
            .all(|row| row.residual_quantity_type_id.is_some())
    );
}

fn check_symbols(
    canonical: &Snapshot,
    instances: &[normalized::instance_bindings::Row],
    state_template: SemanticId,
    component_flow_state: bool,
    heater: bool,
) {
    let states = instances
        .iter()
        .filter(|row| row.template_id == state_template);
    let symbols = compiled::symbols::View::from_checked(
        canonical.relation("compiled", "symbols").unwrap().checked(),
    )
    .unwrap()
    .rows()
    .unwrap();
    for state in states {
        for (declaration, role, indexed) in if component_flow_state {
            [
                (
                    "eb889bdb424c416d9e889da639a36879",
                    SymbolRole::Variable,
                    false,
                ),
                (
                    "81ad47e3c3fd46ed89459602827b74a7",
                    SymbolRole::Variable,
                    false,
                ),
                (
                    "49011f4388df42cea7dfb55b4abe1db9",
                    SymbolRole::Variable,
                    true,
                ),
                (
                    "dd186e057b1541bb9a61423ae239c006",
                    SymbolRole::Expression,
                    false,
                ),
            ]
        } else {
            [
                (
                    "75d55d1c66da4e97bd17de4480d959d5",
                    SymbolRole::Variable,
                    false,
                ),
                (
                    "989f9115f04f4d8d8c30f2cd464df2c7",
                    SymbolRole::Variable,
                    false,
                ),
                (
                    "6405ce47ec2a4cbd8cbe1fe5da6eb3ee",
                    SymbolRole::Variable,
                    false,
                ),
                (
                    "197c95242fc44d9382babf49170dbe7e",
                    SymbolRole::Expression,
                    true,
                ),
            ]
        } {
            let actual = symbols
                .iter()
                .filter(|row| {
                    row.owner_instance_id == state.instance_id
                        && row.symbol_decl_id == id(declaration)
                })
                .collect::<Vec<_>>();
            assert_eq!(
                actual.len(),
                1,
                "one nitrogen member per state declaration {declaration}"
            );
            assert_eq!(actual[0].role, role);
            assert_eq!(actual[0].index.len(), usize::from(indexed));
        }
    }
    let count = |declaration: &str| {
        symbols
            .iter()
            .filter(|row| row.symbol_decl_id == id(declaration))
            .count()
    };
    assert_eq!(
        count("e0d5feb4f68748d6850f57f74b4b2b7f"),
        usize::from(heater),
        "heat is enabled only for the heater"
    );
    for disabled in [
        "cb57f6f755e04164ad6a9f5a7200f596",
        "ca583148b35e410ba0a7c6172c9ac328",
        "0b77b73c169547eda8ebcb2ca28b840f",
    ] {
        assert_eq!(
            count(disabled),
            0,
            "work, pressure change and holdup are disabled"
        );
    }
}
