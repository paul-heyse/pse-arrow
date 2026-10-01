// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Normalized admission through authored declarations and the ordinary compiler.
use crate::workspace::{CompilerWorkspace, Inputs, Profile, WorkspaceLimits};
use pse_authoring::{
    ParseBudget,
    language::{IdentityPolicy, parse},
};
use pse_ids::SemanticId;
use pse_math::binding::CaseValues;
use pse_modeling::{Bindings, Limits, PhysicalScope, specialize::root_instance};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};

fn control(potential: &str, response: &str, derivative: bool) -> Result<(), String> {
    let result = if derivative { "Scalar" } else { "Length" };
    let zero = if derivative { "0" } else { "0{m}" };
    let tolerance = if derivative { "1e-12" } else { "1e-12{m}" };
    let text = format!(
        r#"package p {{
        entity kind convention {{}} entity convention ideal {{}}
        coordinate map coords(x:Length) {{ slot x=x/1{{m}}; }}
        reconstruction family for coords(x:Length)->Length reference ideal=1{{m}};
        fn law(x:Coordinate<coords.x>)->Reduced<family>=0*x;
        fn potential(x:Length)->Length={potential};
        response response_value from potential(x:Length,y:Length,potential:Fn(x:Length)->Length)->{result}={response};
        test witness_control {{ expect response_value(2{{m}},2{{m}},potential)=={zero} tolerance {tolerance}; }}
    }}"#
    );
    let rows = parse(
        &text,
        SemanticId::from_bytes([75; 16]),
        IdentityPolicy::Named,
        ParseBudget::default(),
    )
    .map_err(|error| error.to_string())?;
    let inputs = Inputs {
        quantities: Arc::new(pse_quantity::standard::standard_registry().unwrap()),
        preconditions: Arc::new(
            pse_quantity::PhysicalPreconditions::new(
                pse_quantity::generated::standard_preconditions(),
            )
            .unwrap(),
        ),
        flows: BTreeMap::new(),
        definitions: BTreeMap::new(),
        domains: BTreeMap::new(),
        groups: BTreeMap::new(),
        providers: BTreeMap::new(),
        cases: BTreeMap::new(),
        values: BTreeMap::new(),
    };
    let mut workspace = CompilerWorkspace::new(inputs, WorkspaceLimits::default())
        .map_err(|error| error.to_string())?;
    workspace
        .publish_modeling(rows.clone(), PhysicalScope::default())
        .map_err(|error| error.to_string())?;
    let root = rows
        .iter()
        .find(|row| row.name == "witness_control")
        .unwrap()
        .declaration_id;
    let outcome = workspace
        .check_modeling_point(
            root,
            root_instance(root),
            Bindings::default(),
            Limits::default(),
            &CaseValues {
                scalars: BTreeMap::new(),
            },
            Profile::default(),
            Arc::new(AtomicBool::new(false)),
        )
        .map_err(|error| error.to_string())?;
    assert!(!outcome.expectations.is_empty());
    assert!(
        outcome
            .expectations
            .iter()
            .all(|expectation| expectation.passed),
        "{outcome:?}"
    );
    Ok(())
}

#[test]
fn scientific_witness_accepts_zero_reduced_law_value_and_partials() {
    for (response, derivative) in [
        ("potential(x)", false),
        ("partial(potential,x)(x)", true),
        ("potential(x)-potential(y)", false),
    ] {
        control("reconstruct(family,law,x)", response, derivative).unwrap();
    }
}

#[test]
fn scientific_witness_refuses_response_zero_coefficients_and_cancellation() {
    for (response, derivative) in [
        ("0*potential(x)", false),
        ("potential(x)-potential(x)", false),
        (
            "potential(x)*(x/1{m}+1)-potential(x)*x/1{m}-potential(x)",
            false,
        ),
        ("0*partial(potential,x)(x)", true),
        ("partial(potential,x)(x)-partial(potential,x)(x)", true),
    ] {
        let error = control("reconstruct(family,law,x)", response, derivative).unwrap_err();
        assert!(
            error.contains("must retain its selected potential or partials after normalization"),
            "{response}: {error}"
        );
    }
}

#[test]
fn scientific_witness_refuses_erased_selected_reconstruction() {
    for potential in [
        "0*reconstruct(family,law,x)",
        "reconstruct(family,law,x)-reconstruct(family,law,x)",
    ] {
        let error = control(potential, "potential(x)", false).unwrap_err();
        assert!(
            error.contains("must retain its physical reconstruction after normalization"),
            "{potential}: {error}"
        );
    }
}
