// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual phase stocks, elemental projection and original closed-transfer checks.
#![allow(
    clippy::unwrap_used,
    reason = "focused scientific inventory assertions"
)]

use crate::{
    authored_transfer_tests::{context, reference_family_sources, root, rows},
    workspace::{
        CompileError, CompilerWorkspace, ModelingOutput, PreparedModeling, WorkspaceLimits,
    },
};
use pse_kernels::DerivativeOrder;
use pse_math::{
    assembly::AssemblyLimits, binding::CaseValues, jets::EvaluationLimits, library::Optimization,
};
use pse_modeling::{Bindings, Limits, PhysicalScope, analysis::Route, specialize::root_instance};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

fn source() -> String {
    reference_family_sources("inventory")
}

fn prepare(text: &str) -> Result<PreparedModeling, CompileError> {
    prepare_point(
        text,
        "inventory_fixtures",
        "closed_transfer",
        Route::Simultaneous,
    )
}

fn prepare_point(
    text: &str,
    package: &str,
    name: &str,
    route: Route,
) -> Result<PreparedModeling, CompileError> {
    let declarations = rows(text);
    let id = root(&declarations, package, name);
    let mut workspace = CompilerWorkspace::new(context(), WorkspaceLimits::default()).unwrap();
    workspace.publish_modeling(declarations, PhysicalScope::default())?;
    let mut bindings = Bindings::default().with_analysis(route);
    if package == "inventory_fixtures" && name == "closed_transfer" {
        for time in ["0{s}", "1{s}"] {
            for phase in ["liquid", "vapor"] {
                for species in ["benzene", "toluene"] {
                    bindings.demand.push(format!(
                        "root.cell[{time}].phase_amount[chemistry.{phase},chem.{species}]"
                    ));
                }
            }
        }
    }
    workspace.prepare_modeling_cancellable(
        id,
        root_instance(id),
        bindings,
        Limits::default(),
        pse_diagnostics::Cancellation::default().flag(),
    )
}

// An independent exact trajectory, supplied to the actual compiled source rather
// than obtained by solving its equations or reusing its projection functions.
fn exact_values(prepared: &PreparedModeling) -> CaseValues {
    let mut values = CaseValues {
        scalars: BTreeMap::new(),
    };
    for (time, liquid_benzene, liquid_toluene, vapor_benzene, vapor_toluene) in
        [("0{s}", 2.0, 3.0, 1.0, 4.0), ("1{s}", 1.75, 2.5, 1.25, 4.5)]
    {
        for (phase, species, amount) in [
            ("liquid", "benzene", liquid_benzene),
            ("liquid", "toluene", liquid_toluene),
            ("vapor", "benzene", vapor_benzene),
            ("vapor", "toluene", vapor_toluene),
        ] {
            let path = format!("root.cell[{time}].phase_amount[chemistry.{phase},chem.{species}]");
            let id = prepared.model.paths[&path];
            assert!(prepared.admitted.inputs.contains(&id), "{path}");
            values.scalars.insert(id, amount);
        }
    }
    assert_eq!(
        values.scalars.keys().copied().collect::<BTreeSet<_>>(),
        prepared.admitted.inputs.iter().copied().collect(),
        "the independent trajectory supplies every actual input without numerical start defaults"
    );
    values
}

fn evaluate(prepared: &PreparedModeling, values: &CaseValues) -> Vec<f64> {
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
    let computed = worker.constraints(values).unwrap();
    prepared.admitted.ordered_values(computed).unwrap()
}

fn original_closures(
    prepared: &PreparedModeling,
    outputs: &[f64],
) -> Vec<pse_modeling::specialize::ClosureAssessment> {
    let selected: BTreeSet<_> = prepared
        .model
        .closures
        .values()
        .filter(|closure| {
            ["component_closure", "element_closure", "total_closure"]
                .iter()
                .any(|name| closure.lineage.path.ends_with(name))
        })
        .map(|closure| closure.id)
        .collect();
    let contributions = prepared
        .admitted
        .outputs
        .iter()
        .zip(outputs)
        .filter_map(|(output, value)| match output {
            ModelingOutput::Contribution {
                accumulator,
                contribution,
            } if selected.contains(accumulator) => Some((*contribution, *value)),
            _ => None,
        })
        .collect();
    let budgets = prepared
        .model
        .closures
        .keys()
        .filter(|id| selected.contains(id))
        .map(|id| (*id, 1e-9))
        .collect();
    prepared
        .model
        .assess_closures(&contributions, &selected, &budgets)
        .unwrap()
}

#[test]
fn scientific_inventory_phase_allocation_preserves_original_component_element_and_total_stocks() {
    let prepared = prepare(&source()).unwrap();
    let outputs = evaluate(&prepared, &exact_values(&prepared));
    let observed: BTreeMap<_, _> = prepared
        .admitted
        .outputs
        .iter()
        .zip(&outputs)
        .map(|(output, value)| (output.row_id(), *value))
        .collect();
    let expectations = prepared.assess_expectations(&observed).unwrap();
    assert_eq!(
        expectations.len(),
        10,
        "all authored stock/flux expectations are assessed"
    );
    assert!(
        expectations.iter().all(|row| row.passed),
        "{expectations:?}"
    );
    for (output, value) in prepared.admitted.outputs.iter().zip(&outputs) {
        if matches!(output, ModelingOutput::Equation { .. }) {
            assert!(value.abs() < 1e-9, "{output:?}: {value}");
        }
    }
    let closures = original_closures(&prepared, &outputs);
    assert_eq!(closures.len(), 5, "two species, two elements, one total");
    assert!(
        closures
            .iter()
            .all(|closure| closure.satisfied == Some(true)),
        "{closures:?}"
    );
    assert_eq!(
        prepared.model.inventory_balances.len(),
        4,
        "only allocated phase stocks evolve"
    );
    let registry = context().quantities;
    let element = registry
        .quantity_types()
        .find(|q| q.name.as_deref() == Some("ElementAmount"))
        .unwrap();
    let component = registry
        .quantity_types()
        .find(|q| q.name.as_deref() == Some("Amount"))
        .unwrap();
    assert_ne!(element.key.subject_kind, component.key.subject_kind);
    for closure in prepared
        .model
        .closures
        .values()
        .filter(|c| c.lineage.path.ends_with("element_closure"))
    {
        assert_eq!(
            closure
                .ty
                .quantity_scheme()
                .unwrap()
                .resolve(&registry, &Default::default())
                .unwrap(),
            element.id
        );
    }
}

#[test]
fn scientific_inventory_wrong_sign_and_double_counted_transfer_fail_original_projection_closure() {
    let text = source();
    let original = "then -transfer_rate[j] else transfer_rate[j]";
    assert_eq!(text.matches(original).count(), 1);
    for wrong in [
        "then -transfer_rate[j] else -transfer_rate[j]",
        "then -transfer_rate[j] else 2*transfer_rate[j]",
    ] {
        let prepared = prepare(&text.replace(original, wrong)).unwrap();
        let outputs = evaluate(&prepared, &exact_values(&prepared));
        let closures = original_closures(&prepared, &outputs);
        assert_eq!(closures.len(), 5);
        assert!(
            closures
                .iter()
                .any(|closure| closure.satisfied == Some(false)),
            "{wrong}: {closures:?}"
        );
    }
}

#[test]
fn scientific_inventory_unknown_composition_cannot_create_elemental_stock() {
    let text = source();
    let original = "entity chemistry.species benzene ";
    assert_eq!(text.matches(original).count(), 1);
    let mut changes = 0;
    let unknown = text
        .lines()
        .map(|line| {
            if line.contains(original) {
                let replacement = line.replace(
                    "composition = chemistry.CompositionKnowledge.Complete, ",
                    "",
                );
                assert_ne!(line, replacement);
                changes += 1;
                replacement
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(changes, 1);
    let error = prepare(&unknown).unwrap_err();
    let CompileError::Modeling(error) = error else {
        panic!("expected authored inventory refusal: {error:?}")
    };
    let mut cause = &error;
    while let pse_modeling::ModelingError::Located { cause: inner, .. } = cause {
        cause = inner;
    }
    let pse_modeling::ModelingError::Contract {
        reason: pse_modeling::ModelingContractReason::AuthoredRequirement { requirement },
        ..
    } = cause
    else {
        panic!("expected complete-composition requirement: {cause:?}")
    };
    assert_eq!(
        *requirement,
        pse_ids::SemanticId::parse_hex("64e02000000000000000000000000206").unwrap()
    );
}

#[test]
fn scientific_inventory_element_stock_cannot_be_declared_as_component_amount() {
    let text = source();
    let original = "element_closure[e in elements]:ElementAmount observation";
    assert_eq!(text.matches(original).count(), 1);
    assert!(
        prepare(&text.replace(
            original,
            "element_closure[e in elements]:Amount observation"
        ))
        .is_err()
    );
}

#[test]
fn scientific_inventory_homogeneous_allocation_uses_actual_stock_not_normalization_amount() {
    let text = source()
        + r#"
package homogeneous_inventory_control {
use vessels @"1.0.0";use helmholtz @"1.0.0";use chemistry @"1.0.0";use chem @"1.0.0";
fn composition(j:chemistry.species)->Scalar=0.5;
fn ideal_h(T:Temperature,j:chemistry.species)->DeltaH=2000{J/mol};
test point {
child stock:vessels.HomogeneousInventory=vessels.HomogeneousInventory(selected={chem.benzene,chem.toluene},law=helmholtz.ideal,ideal_h=ideal_h,composition=composition,V=0.2{m^3});
expect stock.component_amount[chem.benzene]==1{mol} tolerance 1e-9{mol};
expect stock.component_amount[chem.toluene]==1{mol} tolerance 1e-9{mol};
}
}"#;
    let prepared = prepare_point(
        &text,
        "homogeneous_inventory_control",
        "point",
        Route::Steady,
    )
    .unwrap();
    // rho=10 mol/m3, V=0.2 m3: actual total is 2 mol, while the
    // Helmholtz normalization contains only 1 mol split equally.
    let energy = 4000.0 - 10.0 * 8.314_462_618_153_24 * 300.0 * 0.2;
    let values = CaseValues {
        scalars: prepared
            .admitted
            .inputs
            .iter()
            .map(|id| {
                let path = &prepared.model.symbols[id].lineage.path;
                let value = if path.ends_with(".phase.T") {
                    300.0
                } else if path.ends_with(".phase.rho") {
                    10.0
                } else if path.contains(".phase.amount") {
                    0.5
                } else if path.ends_with(".amount") {
                    2.0
                } else if path.ends_with(".volume") {
                    0.2
                } else if path.ends_with(".energy") {
                    energy
                } else {
                    panic!("unexpected homogeneous inventory input: {path}")
                };
                (*id, value)
            })
            .collect(),
    };
    let outputs = evaluate(&prepared, &values);
    let observed = prepared
        .admitted
        .outputs
        .iter()
        .zip(&outputs)
        .map(|(output, value)| (output.row_id(), *value))
        .collect();
    let expectations = prepared.assess_expectations(&observed).unwrap();
    assert_eq!(expectations.len(), 2);
    assert!(
        expectations.iter().all(|row| row.passed),
        "{expectations:?}"
    );
    for (output, value) in prepared.admitted.outputs.iter().zip(outputs) {
        if matches!(output, ModelingOutput::Equation { .. }) {
            assert!(value.abs() < 1e-8, "{output:?}: {value}");
        }
    }
}
