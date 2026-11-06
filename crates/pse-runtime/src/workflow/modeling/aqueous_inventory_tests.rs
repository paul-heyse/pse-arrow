// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual aqueous cell stocks and independently accumulated boundary closure.
use super::*;
use crate::{math::solves::NumericalInputs, workflow::tests as fixture};
#[cfg(feature = "solver-diffsol")]
use pse_backend_native::dynamics::Method;
use pse_backend_native::solve::{Backend, SolverSelection};
#[cfg(feature = "solver-diffsol")]
use pse_modeling::{
    SpecializedModel,
    specialize::{
        Value,
        finite::{FamilyKey, MemberKey},
    },
};
use std::collections::BTreeSet;

async fn conform(name: &str) -> ModelingConformanceReport {
    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package(false)
            .await
            .unwrap();
    let services = runtime.execution_services();
    let selected = package
        .declarations(services.preparation())
        .await
        .unwrap()
        .into_iter()
        .find(|row| row.name == name)
        .unwrap()
        .declaration_id;
    let mut solver = fixture::profile();
    solver.selection = SolverSelection::Explicit(Backend::Ipopt);
    let policy = ModelingConformancePolicy {
        compiler: preparation.compiler,
        solver,
        numerical: NumericalInputs::default(),
        limits: preparation.limits,
        derivatives: pse_backend_native::derivative_diagnostics::Policy {
            perturbation: 1e-6,
            relative_tolerance: 1e-4,
            maximum_cells: 20_000_000,
        },
        maximum_fixtures: 1,
        maximum_checks: 100_000,
        fixtures: ModelingFixtureSelection::Selected(BTreeSet::from([selected])),
        diagnostics: None,
    };
    package
        .conform(&services, policy, &crate::CancelSource::new())
        .await
        .unwrap()
}

fn assert_complete(report: &ModelingConformanceReport) {
    assert!(
        report.complete && report.passed(),
        "{:?}; {:?}",
        report.checks,
        report.failures
    );
    assert_eq!(report.results.len(), 1);
    let result = report.results.values().next().unwrap();
    let closure = result
        .reports
        .iter()
        .filter(|row| row.label == "aqueous cascade global material closure")
        .collect::<Vec<_>>();
    assert_eq!(
        closure.len(),
        2,
        "both actual water and ethanol inventories require closure"
    );
    for row in closure {
        assert!(row.value.abs() <= 0.001, "{row:?}");
    }
    let inventories = result
        .reports
        .iter()
        .filter(|row| row.label == "aqueous cascade component inventory")
        .collect::<Vec<_>>();
    assert!(
        inventories.len() > 2,
        "retain physical inventory across the original timeline"
    );
    assert!(inventories.iter().all(|row| row.value >= -0.001));
    assert!(
        result
            .reports
            .iter()
            .any(|row| row.label == "aqueous cascade imposed temperature" && row.value == 300.)
    );
}

#[tokio::test]
async fn aqueous_inventory_single_cell_matches_exact_mixed_cell_response() {
    assert_complete(&conform("aqueous_single_cell_step").await);
}

#[tokio::test]
async fn aqueous_inventory_four_cells_preserve_nonzero_clock_and_global_stock_balance() {
    assert_complete(&conform("aqueous_four_cell_nonzero_clock").await);
}

#[tokio::test]
async fn aqueous_inventory_zero_flow_retains_original_component_stock() {
    assert_complete(&conform("aqueous_four_cell_zero_flow").await);
}

#[tokio::test]
async fn aqueous_geometry_requires_readmission_instead_of_an_independent_numeric_override() {
    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package(false)
            .await
            .unwrap();
    let services = runtime.execution_services();
    let cancel = crate::CancelSource::new();
    let selected = package
        .declarations(services.preparation())
        .await
        .unwrap()
        .into_iter()
        .find(|row| row.name == "aqueous_single_cell_integrated")
        .unwrap()
        .declaration_id;
    let mut solver = fixture::profile();
    solver.selection = SolverSelection::Explicit(Backend::Ipopt);
    let declared = package
        .declared_execution(
            services.preparation(),
            selected,
            preparation.compiler,
            solver,
            NumericalInputs::default(),
            preparation.limits,
            &cancel,
        )
        .await
        .unwrap();
    // Constructor volume belongs to original admission; the derived width is
    // neither an independently mutable coordinate nor a numeric permission.
    for (path, expected) in [
        ("root.volume", "unknown instantiated member volume"),
        (
            "root.cell_fraction",
            "case value root.cell_fraction requires a finite independent coordinate",
        ),
    ] {
        let mut changed = declared.analysis.clone();
        changed.case.values.insert(path.into(), 0.002);
        let error = package
            .prepare_analysis(services.preparation(), &changed, &cancel)
            .await
            .unwrap_err();
        assert!(error.to_string().contains(expected), "{path}: {error}");
    }
}

#[cfg(feature = "solver-diffsol")]
#[derive(Clone)]
struct ComponentRead {
    component: DeclarationId,
    original_stock: f64,
    stocks: Vec<(SemanticId, usize)>,
    incoming: usize,
    outgoing: usize,
}

#[cfg(feature = "solver-diffsol")]
fn finite_family(
    model: &SpecializedModel,
    instance: InstanceId,
    declaration: DeclarationId,
) -> &[Vec<Value>] {
    model
        .finite_facts
        .family(&FamilyKey {
            instance,
            declaration,
            role: "binding.indices.domain".into(),
            parents: Vec::new(),
        })
        .unwrap()
}

#[cfg(feature = "solver-diffsol")]
fn component_reads(trajectory: &ModelingTrajectory, initial_tracer: f64) -> Vec<ComponentRead> {
    let declaration = |text| SemanticId::parse_hex(text).unwrap().into();
    let model = &trajectory.prepared().model().compiled().model;
    let contract = trajectory.prepared().contract();
    let cascade = model
        .instances
        .values()
        .find(|instance| instance.definition == declaration("66e03000000000000000000000000120"))
        .unwrap()
        .id;
    let supplied = declaration("66e03000000000000000000000000130");
    let discharged = declaration("66e03000000000000000000000000131");
    let phase_amount = declaration("64e02000000000000000000000000207");
    let cell = declaration("66e03000000000000000000000000110");
    let components = finite_family(model, cascade, supplied);
    assert_eq!(components.len(), 2);
    // Existing species catalogue identities, not quantity or row-order guesses.
    [
        (declaration("01a0e1461e93764398a669ec398dd8a0"), 55.),
        (
            declaration("01a0e1461e93764398a669f485d31290"),
            initial_tracer,
        ),
    ]
    .into_iter()
    .map(|(component, original_stock)| {
        let tuple = components
            .iter()
            .find(
                |tuple| matches!(tuple.as_slice(), [Value::Entity { id, .. }] if *id == component),
            )
            .unwrap();
        let quadrature = |declaration| {
            let member = model
                .finite_facts
                .require_member(&MemberKey {
                    instance: cascade,
                    declaration,
                    tuple: tuple.clone(),
                })
                .unwrap();
            let integral = model.integral_of(member).unwrap();
            contract
                .quadratures
                .iter()
                .position(|id| *id == integral)
                .unwrap()
        };
        let stocks = model
            .instances
            .values()
            .filter(|instance| instance.definition == cell)
            .map(|instance| {
                let candidates = finite_family(model, instance.id, phase_amount)
                    .iter()
                    .filter(|phase_tuple| phase_tuple.last() == tuple.first())
                    .collect::<Vec<_>>();
                assert_eq!(
                    candidates.len(),
                    1,
                    "one actual liquid-phase stock per component/cell"
                );
                let member = model
                    .finite_facts
                    .require_member(&MemberKey {
                        instance: instance.id,
                        declaration: phase_amount,
                        tuple: candidates[0].clone(),
                    })
                    .unwrap();
                let index = contract.states.iter().position(|id| *id == member).unwrap();
                assert!(contract.differential[index]);
                (member, index)
            })
            .collect();
        ComponentRead {
            component,
            original_stock,
            stocks,
            incoming: quadrature(supplied),
            outgoing: quadrature(discharged),
        }
    })
    .collect()
}

#[cfg(feature = "solver-diffsol")]
fn prefix_balances(
    trajectory: &ModelingTrajectory,
    samples: &[&pse_backend_native::dynamics::Sample],
    times: &[f64],
    reads: &[ComponentRead],
) -> Result<(), String> {
    if samples.iter().map(|sample| sample.time).collect::<Vec<_>>() != times {
        return Err("original requested physical point is missing or reordered".into());
    }
    let contract = trajectory.prepared().contract();
    for sample in samples {
        if sample.integrals.len() != contract.quadratures.len() {
            return Err("native cumulative quadrature extent differs".into());
        }
        // This owner only projects produced native coordinates to canonical
        // physical values; it does not reevaluate a model or a definite integral.
        let (_, physical, _) = trajectory
            .sample_context(sample)
            .map_err(|error| error.to_string())?;
        for read in reads {
            let amount = read.stocks.iter().try_fold(0., |total, (member, index)| {
                if contract.states.get(*index) != Some(member) || sample.state.get(*index).is_none()
                {
                    return Err(
                        "actual phase stock/native coordinate correspondence differs".to_owned(),
                    );
                }
                physical
                    .scalars
                    .get(member)
                    .map(|amount| total + amount)
                    .ok_or_else(|| "actual phase stock is unavailable".to_owned())
            })?;
            let error = (amount - read.original_stock)
                - (sample.integrals[read.incoming] - sample.integrals[read.outgoing]);
            if !error.is_finite() || error.abs() > 0.001 {
                return Err(format!(
                    "component {:?} at {}s: independent global stock/flux error {error}mol",
                    read.component, sample.time
                ));
            }
        }
    }
    Ok(())
}

#[cfg(feature = "solver-diffsol")]
async fn assert_integrated(
    name: &str,
    cells: usize,
    times: &[f64],
    response: &str,
    stock: &str,
    initial_tracer: f64,
    counterexamples: bool,
) {
    let report = conform(name).await;
    assert!(
        report.complete && report.passed(),
        "{:?}; {:?}",
        report.checks,
        report.failures
    );
    assert!(
        report.results.is_empty(),
        "an integrated trajectory is its own native result"
    );
    assert_eq!(report.trajectories.len(), 1);
    let trajectory = report.trajectories.values().next().unwrap();
    assert!(trajectory.accepted() && trajectory.checks_complete());
    assert_eq!(trajectory.prepared().profile().method, Method::Diffsol);
    assert_eq!(
        trajectory
            .report()
            .samples
            .iter()
            .map(|sample| sample.time)
            .collect::<Vec<_>>(),
        times
    );
    let cell: DeclarationId = SemanticId::parse_hex("66e03000000000000000000000000110")
        .unwrap()
        .into();
    assert_eq!(
        trajectory
            .prepared()
            .model()
            .compiled()
            .model
            .instances
            .values()
            .filter(|instance| instance.definition == cell)
            .count(),
        cells,
        "requested output times must not create additional physical cells"
    );
    assert_eq!(
        trajectory
            .prepared()
            .contract()
            .differential
            .iter()
            .filter(|state| **state)
            .count(),
        cells * 2,
        "water and ethanol actual amount are the differential states per cell"
    );
    for source in [response, stock] {
        let source: DeclarationId = SemanticId::parse_hex(source).unwrap().into();
        for &time in times {
            let checks = trajectory
                .checks()
                .iter()
                .filter(|check| check.source_id == source && check.time == Some(time))
                .collect::<Vec<_>>();
            assert_eq!(
                checks
                    .iter()
                    .map(|check| check.target_id)
                    .collect::<BTreeSet<_>>()
                    .len(),
                2,
                "original source {source:?} must check both component targets at physical time {time}"
            );
            assert!(checks.iter().all(|check| check.satisfied));
        }
    }
    let reads = component_reads(trajectory, initial_tracer);
    assert!(reads.iter().all(|read| read.stocks.len() == cells));
    let samples = trajectory.report().samples.iter().collect::<Vec<_>>();
    prefix_balances(trajectory, &samples, times, &reads).unwrap();
    if counterexamples {
        let mut wrong_component = reads.clone();
        wrong_component[1].incoming = reads[0].incoming;
        wrong_component[1].outgoing = reads[0].outgoing;
        assert!(
            prefix_balances(trajectory, &samples, times, &wrong_component).is_err(),
            "water boundary quadratures cannot certify ethanol inventory"
        );
        let missing = samples
            .iter()
            .copied()
            .filter(|sample| sample.time != 5.25)
            .collect::<Vec<_>>();
        assert!(
            prefix_balances(trajectory, &missing, times, &reads).is_err(),
            "missing the original5.25s point cannot be treated as full coverage"
        );
    }
    let closure = trajectory
        .reports()
        .iter()
        .filter(|row| row.label == "aqueous cascade global material closure")
        .collect::<Vec<_>>();
    // Sample checks above retain physical time. Reports are terminal whole-domain
    // observations, not a second time-indexed trajectory table.
    assert_eq!(
        closure.len(),
        2,
        "both component closures retain their terminal original report"
    );
    assert!(closure.iter().all(|row| row.value.abs() <= 0.001));
    let terminal_source: DeclarationId = SemanticId::parse_hex("66e03000000000000000000000000157")
        .unwrap()
        .into();
    let terminal = trajectory
        .checks()
        .iter()
        .filter(|check| check.source_id == terminal_source)
        .collect::<Vec<_>>();
    assert_eq!(
        terminal
            .iter()
            .map(|check| check.target_id)
            .collect::<BTreeSet<_>>()
            .len(),
        2
    );
    assert!(
        terminal
            .iter()
            .all(|check| check.time == times.last().copied() && check.satisfied)
    );
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn aqueous_integrated_single_cell_preserves_original_response_and_stock() {
    assert_integrated(
        "aqueous_single_cell_integrated",
        1,
        &[0., 1., 5.],
        "66e03000000000000000000000000242",
        "66e03000000000000000000000000243",
        0.,
        false,
    )
    .await;
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn aqueous_integrated_four_cells_use_one_nonzero_physical_clock() {
    assert_integrated(
        "aqueous_four_cell_integrated_nonzero_clock",
        4,
        &[5., 5.25, 6., 7., 10.],
        "66e03000000000000000000000000252",
        "66e03000000000000000000000000253",
        0.,
        true,
    )
    .await;
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn aqueous_integrated_zero_flow_preserves_original_nonzero_inventory() {
    assert_integrated(
        "aqueous_four_cell_integrated_zero_flow",
        4,
        &[0., 1., 5.],
        "66e03000000000000000000000000262",
        "66e03000000000000000000000000263",
        0.01,
        false,
    )
    .await;
}
