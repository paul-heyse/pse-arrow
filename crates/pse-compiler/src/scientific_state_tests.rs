// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual authored state families through admission, reconstruction and original transport.
#![allow(clippy::unwrap_used, reason = "focused scientific contract assertions")]

use crate::{
    authored_transfer_tests::{context, root, rows},
    workspace::{
        CompilerWorkspace, ModelingCaseBindings, ModelingHint, ModelingOutput, PreparedModeling,
        WorkspaceLimits,
    },
};
use pse_kernels::DerivativeOrder;
use pse_math::{
    assembly::AssemblyLimits, binding::CaseValues, jets::EvaluationLimits, library::Optimization,
};
use pse_modeling::{Bindings, Limits, PhysicalScope, specialize::root_instance};
use std::{collections::BTreeMap, sync::Arc};

fn source() -> String {
    crate::authored_transfer_tests::reference_family_sources("state-coordinates")
}

fn prepare(name: &str) -> PreparedModeling {
    let declarations = rows(&source());
    let id = root(&declarations, "state_coordinate_fixtures", name);
    let mut workspace = CompilerWorkspace::new(context(), WorkspaceLimits::default()).unwrap();
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    let mut bindings = Bindings::default();
    for side in ["source", "receiver"] {
        for member in ["flow", "T", "h", "z[chem.benzene]", "z[chem.toluene]"] {
            bindings.demand.push(format!("root.{side}.{member}"));
        }
        if matches!(name, "phase_temperature" | "disappearing_phase") {
            for phase in ["liquid", "vapor"] {
                bindings.demand.push(format!(
                    "root.{side}.phase_z[chemistry.{phase},chem.toluene]"
                ));
            }
        }
    }
    workspace
        .prepare_modeling_cancellable(
            id,
            root_instance(id),
            bindings,
            Limits::default(),
            pse_diagnostics::Cancellation::default().flag(),
        )
        .unwrap_or_else(|error| panic!("{name}: {error:?}"))
}

// These are independently calculated exact points, not solver solutions. Mirror
// the fixture's supplied coordinates to exercise the actual connection equations;
// receiver reference fractions are independently supplied here and must satisfy
// their authored reconstruction residual. Runtime fixtures separately solve them.
fn values(prepared: &PreparedModeling, enthalpy: f64) -> CaseValues {
    let fixture = prepared.model.fixtures.values().next().unwrap();
    let mut values = prepared
        .case_values(&ModelingCaseBindings::from(fixture))
        .unwrap();
    let supplied = values.scalars.clone();
    for connection in prepared.model.connections.values() {
        for (source, receiver) in &connection.bindings {
            let source = prepared.model.ports[source].symbol;
            let receiver = prepared.model.ports[receiver].symbol;
            values.scalars.insert(receiver, supplied[&source]);
        }
    }
    for id in &prepared.admitted.inputs {
        if values.scalars.contains_key(id) {
            continue;
        }
        let symbol = &prepared.model.symbols[id];
        let path = prepared
            .model
            .paths
            .iter()
            .find_map(|(path, member)| (*member == *id).then_some(path))
            .unwrap();
        let source_path = path.replacen(".receiver.", ".source.", 1);
        let mirrored = prepared
            .model
            .paths
            .get(&source_path)
            .and_then(|source| supplied.get(source))
            .copied();
        let value = mirrored.unwrap_or_else(|| {
            if symbol.lineage.path.ends_with(".T") {
                350.0
            } else if symbol.lineage.path.ends_with(".h") {
                enthalpy
            } else if symbol.lineage.path.ends_with(".flow") {
                if enthalpy == 5000.0 { 3.0 } else { 2.0 }
            } else if symbol.lineage.path.ends_with(".z") {
                let first = path.contains("[chem.benzene]");
                match enthalpy {
                    5000.0 => {
                        if first {
                            1.0 / 3.0
                        } else {
                            2.0 / 3.0
                        }
                    }
                    1750.0 => {
                        if first {
                            0.25
                        } else {
                            0.75
                        }
                    }
                    2000.0 => {
                        if first {
                            0.0
                        } else {
                            1.0
                        }
                    }
                    _ => 0.5,
                }
            } else {
                panic!("unexpected independent state coordinate {path}: {symbol:?}")
            }
        });
        values.scalars.insert(*id, value);
    }
    values
}

fn evaluate(
    prepared: &PreparedModeling,
    values: &CaseValues,
) -> Result<Vec<f64>, pse_math::MathError> {
    let cancel = pse_diagnostics::Cancellation::default();
    let registry = context().quantities;
    let plan = prepared
        .admitted
        .plan(
            &registry,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            &cancel.flag(),
        )
        .unwrap();
    let assembly = Arc::new(
        plan.compile(
            Optimization::default(),
            EvaluationLimits::default(),
            &pse_kernels::ExecutionScope::new(cancel.clone(), None),
        )
        .unwrap(),
    );
    let mut worker = assembly.worker(pse_kernels::ProviderBindings::new(), cancel);
    let computed = worker.constraints(values)?;
    Ok(prepared.admitted.ordered_values(computed).unwrap())
}

fn assert_point(name: &str, enthalpy: f64) {
    let prepared = prepare(name);
    let outputs = evaluate(&prepared, &values(&prepared, enthalpy))
        .unwrap_or_else(|error| panic!("{name}: {error:?}"));
    let observed: BTreeMap<_, _> = prepared
        .admitted
        .outputs
        .iter()
        .zip(&outputs)
        .map(|(output, value)| (output.row_id(), *value))
        .collect();
    let expectations = prepared.assess_expectations(&observed).unwrap();
    assert!(!expectations.is_empty());
    assert!(
        expectations.iter().all(|row| row.passed),
        "{name}: {expectations:?}"
    );
    assert!(
        prepared
            .assess_validity(&observed)
            .unwrap()
            .iter()
            .all(|row| row.within())
    );
    let mut equations = 0;
    let mut checks = 0;
    for (output, value) in prepared.admitted.outputs.iter().zip(outputs) {
        match output {
            ModelingOutput::Equation { .. } => {
                equations += 1;
                assert!(value.abs() < 1e-7, "{name}: {output:?}: residual {value}");
            }
            ModelingOutput::Hint {
                kind: ModelingHint::Check,
                ..
            } => {
                checks += 1;
                assert_eq!(value, 1.0, "{name}: {output:?}");
            }
            _ => {}
        }
    }
    assert!(
        equations >= 2,
        "caloric and connection equations are exercised"
    );
    assert!(checks > 0, "exact composition/flow checks are exercised");
    assert_eq!(prepared.model.connections.len(), 1);
}

#[test]
fn scientific_state_six_authored_coordinate_families_preserve_caloric_and_transport_correspondence()
{
    for (name, enthalpy) in [
        ("total_temperature", 1500.0),
        ("total_enthalpy", 1500.0),
        ("component_temperature", 1500.0),
        ("component_enthalpy", 1500.0),
        ("phase_component_temperature", 5000.0),
        ("phase_temperature", 5000.0),
    ] {
        assert_point(name, enthalpy);
    }
}

#[test]
fn scientific_state_zero_flow_and_disappearing_phase_keep_supplied_intensives() {
    assert_point("total_zero_flow", 1500.0);
    assert_point("disappearing_phase", 1750.0);
    assert_point("zero_component", 2000.0);
}

#[test]
fn scientific_state_component_coordinates_refuse_zero_total_before_division() {
    let prepared = prepare("component_temperature");
    let mut values = values(&prepared, 1500.0);
    for (id, value) in &mut values.scalars {
        if prepared.model.symbols[id]
            .lineage
            .path
            .ends_with(".component_rate")
            || prepared.model.symbols[id].lineage.path.ends_with(".flow")
        {
            *value = 0.0;
        }
    }
    assert!(
        evaluate(&prepared, &values).is_err(),
        "zero total has no reconstructed bulk composition"
    );
}

#[test]
fn scientific_state_phase_component_caloric_coordinates_refuse_disappearing_phase_intensives() {
    let prepared = prepare("phase_component_temperature");
    let mut values = values(&prepared, 5000.0);
    // The actual semantic phase key is retained in the phase-component state slot.
    let vanished = prepared
        .model
        .state_specifications
        .values()
        .flat_map(|state| &state.coordinates)
        .filter(|(key, _)| key.name == "phase_component")
        .map(|(key, _)| key.indices[0].clone())
        .next()
        .unwrap();
    for state in prepared.model.state_specifications.values() {
        for (key, id) in &state.coordinates {
            if key.name == "phase_component" && key.indices[0] == vanished {
                *values.scalars.get_mut(id).unwrap() = 0.0;
            }
        }
    }
    assert!(
        evaluate(&prepared, &values).is_err(),
        "a zero phase flow has no reconstructed intensive phase enthalpy"
    );
}

#[test]
fn scientific_state_negative_component_flow_is_an_exact_domain_refusal() {
    let prepared = prepare("component_temperature");
    let mut values = values(&prepared, 1500.0);
    let id = prepared
        .admitted
        .inputs
        .iter()
        .find(|id| {
            prepared.model.symbols[*id]
                .lineage
                .path
                .ends_with(".component_rate")
        })
        .unwrap();
    *values.scalars.get_mut(id).unwrap() = -1e-12;
    assert!(
        evaluate(&prepared, &values).is_err(),
        "normalization allowance never admits negative component flow"
    );
}

#[test]
fn scientific_state_fraction_domains_and_normalization_remain_separate_checks() {
    let prepared = prepare("total_temperature");
    for fraction in [0.4, -1e-12] {
        let mut values = values(&prepared, 1500.0);
        for (id, value) in &mut values.scalars {
            if prepared.model.symbols[id].lineage.path.ends_with(".z") {
                let first = prepared
                    .model
                    .paths
                    .iter()
                    .any(|(path, member)| *member == *id && path.contains("[chem.benzene]"));
                *value = if fraction < 0.0 && !first {
                    1.0 - fraction
                } else {
                    fraction
                };
            }
        }
        let output_values = evaluate(&prepared, &values).unwrap();
        let checks: Vec<_> = prepared
            .admitted
            .outputs
            .iter()
            .zip(output_values)
            .filter_map(|(output, value)| match output {
                ModelingOutput::Hint {
                    declaration,
                    kind: ModelingHint::Check,
                    ..
                } => Some((declaration.to_string(), value)),
                _ => None,
            })
            .collect();
        if fraction < 0.0 {
            assert!(
                checks
                    .iter()
                    .filter(|(source, _)| source == "01a0e18b78db742e8fd36e1e9159c2a3")
                    .all(|(_, value)| *value == 1.0),
                "the negative trace control stays normalized"
            );
            assert!(
                checks.iter().any(
                    |(source, value)| source == "64d0200000004000800000000000000a" && *value == 0.0
                ),
                "a normalized negative trace is still outside the exact mole-fraction domain"
            );
        } else {
            assert!(
                checks.iter().any(
                    |(source, value)| source == "01a0e18b78db742e8fd36e1e9159c2a3" && *value == 0.0
                ),
                "normalization must be checked even for individually admissible fractions"
            );
        }
    }
}

#[test]
fn scientific_state_coordinate_family_mismatch_still_requires_explicit_translation() {
    let text = format!(
        "{}\npackage state_mismatch {{use state_coordinate_fixtures @\"1.0.0\"; test Root {{child pair:state_coordinate_fixtures.Correspondence=state_coordinate_fixtures.Correspondence(inlet=state_coordinate_fixtures.TotalTemperature(),outlet=state_coordinate_fixtures.ComponentTemperature(defined_state=false));}}}}",
        source()
    );
    let declarations = rows(&text);
    let id = root(&declarations, "state_mismatch", "Root");
    let mut workspace = CompilerWorkspace::new(context(), WorkspaceLimits::default()).unwrap();
    workspace
        .publish_modeling(declarations, PhysicalScope::default())
        .unwrap();
    let error = workspace
        .specialize_modeling(
            id,
            root_instance(id),
            Bindings::default(),
            Limits::default(),
        )
        .unwrap_err();
    assert!(
        format!("{error:?}").contains(
            "independent state/species correspondence differs; explicit translator required"
        ),
        "{error:?}"
    );
}
