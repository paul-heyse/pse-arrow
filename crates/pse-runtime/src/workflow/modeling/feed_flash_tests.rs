// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Feed-flash specifications consume the original BT phase and balance owners.
use super::*;
use crate::{
    math::solves::NumericalInputs,
    reference_source_fixture::{reference_family_sources, rows},
    workflow::tests as fixture,
};
use pse_backend_native::solve::{Backend, SolverSelection};
use std::collections::BTreeSet;

fn replace_once(source: &str, original: &str, replacement: &str) -> String {
    assert_eq!(source.matches(original).count(), 1);
    source.replacen(original, replacement, 1)
}

fn source(isothermal: bool, temperature: f64, dynamic: bool) -> String {
    let mut source = reference_family_sources("recycle-reference");
    source.push_str(&format!(
        r#"package feed_flash_control {{
        use flashing @"1.0.0"; use bt_ideal @"1.0.0";
        use chem @"1.0.0"; use numerical_policy @"1.0.0";
        test selected fixture {{ dof 0; route {}; procedure {}; {}
          policy {{ limits body_occurrences(1000000); }}
          fix root.flash.inlet.flow=1{{mol/s}};
          fix root.flash.inlet.T={temperature}{{K}};
          fix root.flash.inlet.pressure=101325{{Pa}};
          fix root.flash.inlet.z[chem.benzene]=0.5{{1}};
          fix root.flash.inlet.z[chem.toluene]=0.5{{1}};
        }} {{
          child root:flashing.FeedFlash=flashing.FeedFlash(
            inlet_pkg=bt_ideal.BTIdeal(defined_state=true),
            outlet_pkg=bt_ideal.BTIdeal(defined_state=false),isothermal={isothermal});
          expect root.flash.outlet.T=={temperature}{{K}} tolerance numerical_policy.temperature;
          expect root.flash.outlet.flow==1{{mol/s}} tolerance numerical_policy.molar_flow;
          expect root.flash.outlet.pressure==101325{{Pa}} tolerance numerical_policy.pressure;
          expect root.flash.liquid_component_flow[chem.benzene]+root.flash.vapor_component_flow[chem.benzene]==0.5{{mol/s}} tolerance numerical_policy.component_flow;
          expect root.flash.liquid_component_flow[chem.toluene]+root.flash.vapor_component_flow[chem.toluene]==0.5{{mol/s}} tolerance numerical_policy.component_flow;
          expect root.flash.outlet.beta==0.3961 tolerance numerical_policy.dimensionless;
        }}
        }}"#,
        if dynamic { "integrated" } else { "steady" },
        if dynamic { "integrate" } else { "solve" },
        if dynamic { "integrate samples(0{s},1{s}) relative(global) normalized_absolute(global) step(1e-5{s});" } else { "" },
    ));
    source
}

async fn conform(isothermal: bool, dynamic: bool) -> ModelingConformanceReport {
    conform_source(&source(isothermal, 368.0, dynamic)).await
}

async fn conform_source(source: &str) -> ModelingConformanceReport {
    let runtime = fixture::runtime_with_workspace(128 << 20);
    let rows = rows(source);
    let selected = rows
        .iter()
        .find(|row| row.name == "selected")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(rows, fixture::physical())
        .await
        .unwrap();
    conform_admitted(&runtime, &package, selected).await
}

async fn conform_reference(name: &str) -> ModelingConformanceReport {
    let (runtime, package, _spill, _) = pr_jacobian_tests::ordinary_delivery_package(false)
        .await
        .unwrap();
    let selected = package
        .declarations(&runtime.preparation_services())
        .await
        .unwrap()
        .into_iter()
        .find(|row| row.name == name)
        .unwrap()
        .declaration_id;
    conform_admitted(&runtime, &package, selected).await
}

async fn conform_admitted(
    runtime: &Runtime,
    package: &ModelingPackage,
    selected: DeclarationId,
) -> ModelingConformanceReport {
    let services = runtime.execution_services();
    let mut solver = fixture::profile();
    solver.selection = SolverSelection::Explicit(Backend::Ipopt);
    let policy = ModelingConformancePolicy {
        // Full flash applicability evidence exceeds the toy fixture's 1 MiB
        // scratch ceiling. Use the existing production compiler defaults.
        compiler: pse_compiler::workspace::Profile::default(),
        solver,
        numerical: NumericalInputs::default(),
        limits: Limits::default(),
        derivatives: pse_backend_native::derivative_diagnostics::Policy {
            perturbation: 1e-6,
            relative_tolerance: 1e-4,
            maximum_cells: 10000,
        },
        maximum_fixtures: 1,
        maximum_checks: 4096,
        fixtures: ModelingFixtureSelection::Selected(BTreeSet::from([selected])),
        diagnostics: None,
    };
    package
        .conform(&services, policy, &crate::CancelSource::new())
        .await
        .unwrap()
}

#[tokio::test]
async fn isothermal_feed_flash_supplies_latent_heat_for_a_frozen_liquid_feed() {
    // BTLiquid deliberately keeps the supplied liquid phase rather than
    // equilibrating it. Its caloric datum and correlations are the BT outlet's.
    let report = conform_reference("feed_flash_liquid_isothermal").await;
    assert!(
        report.complete && report.passed(),
        "{:?}; {:?}",
        report.checks,
        report.failures
    );
    let result = report.results.values().next().unwrap();
    let heat = result
        .reports
        .iter()
        .find(|row| row.label == "feed flash heat duty")
        .unwrap();
    assert!(
        heat.value > 10000.0,
        "vaporization requires a positive heat supply: {heat:?}"
    );
}

#[tokio::test]
async fn isenthalpic_feed_flash_cools_a_frozen_liquid_feed_without_heat_supply() {
    let report = conform_reference("feed_flash_liquid_isenthalpic").await;
    assert!(
        report.complete && report.passed(),
        "{:?}; {:?}",
        report.checks,
        report.failures
    );
    let result = report.results.values().next().unwrap();
    let heat = result
        .reports
        .iter()
        .find(|row| row.label == "feed flash heat duty")
        .unwrap();
    assert!(heat.value.abs() < 1.0, "{heat:?}");
}

#[tokio::test]
async fn isothermal_feed_flash_preserves_both_components_and_thermal_specification() {
    let report = conform_reference("feed_flash_equilibrium_isothermal").await;
    assert!(
        report.complete && report.passed(),
        "{:?}; {:?}",
        report.checks,
        report.failures
    );
    assert_eq!(report.results.len(), 1);
    assert_phase_stream_readback(&report, false);
}

#[tokio::test]
async fn isenthalpic_feed_flash_preserves_both_components_and_thermal_specification() {
    let report = conform_reference("feed_flash_equilibrium_isenthalpic").await;
    assert!(
        report.complete && report.passed(),
        "{:?}; {:?}",
        report.checks,
        report.failures
    );
    assert_eq!(report.results.len(), 1);
    assert_phase_stream_readback(&report, true);
}

fn assert_phase_stream_readback(report: &ModelingConformanceReport, isenthalpic: bool) {
    use pse_relations::{columnar::RelationRow as _, generated::runtime::modeling_reports};
    let result = report.results.values().next().unwrap();
    let table = result.table("runtime.modeling_reports").unwrap();
    let rows = modeling_reports::Row::rows(&table).unwrap();
    let read = |label: &str| rows.iter().find(|row| row.label == label).unwrap();
    let liquid = read("feed flash liquid flow");
    let vapor = read("feed flash vapor flow");
    assert_ne!(liquid.target_id, vapor.target_id);
    assert_ne!(liquid.source_id, vapor.source_id);
    assert_eq!(liquid.quantity_id, vapor.quantity_id);
    assert_eq!(liquid.unit_id, vapor.unit_id);
    assert_eq!(liquid.run_id, result.run_id);
    assert_eq!(vapor.run_id, result.run_id);
    for read in [liquid, vapor] {
        let original = result
            .reports
            .iter()
            .find(|row| row.target_id == read.target_id)
            .unwrap();
        assert_eq!(read.value, original.value);
    }
    let heat = read("feed flash heat duty");
    assert!(heat.transfer_context.is_some());
    let input = read("inlet.energy_flow").value;
    let output = read("outlet.energy_flow").value;
    assert!(
        (output - input - heat.value).abs() <= 1.,
        "original energy closure: {input}, {output}, {heat:?}"
    );
    if isenthalpic {
        assert!(heat.value.abs() <= 1.0, "{heat:?}");
    }
}

#[tokio::test]
async fn feed_flash_refuses_temporal_use_before_numerical_execution() {
    let report = conform(true, true).await;
    assert!(!report.passed());
    assert!(report.results.is_empty() && report.trajectories.is_empty());
    assert!(
        report.checks.iter().any(|check| check
            .message
            .contains("feed flash is a steady boundary unit")),
        "{:?}",
        report.checks
    );
}

#[tokio::test]
async fn feed_flash_refuses_holdup_before_numerical_execution() {
    let source = replace_once(
        &source(true, 368.0, false),
        ",isothermal=true);",
        ",isothermal=true,holdup=true);",
    );
    let report = conform_source(&source).await;
    assert!(!report.passed());
    assert!(report.results.is_empty() && report.trajectories.is_empty());
    assert!(
        report.checks.iter().any(|check| check
            .message
            .contains("feed flash has no material or energy storage")),
        "{:?}",
        report.checks
    );
}

#[cfg(all(feature = "solver-kinsol", feature = "solver-root-isolation"))]
#[tokio::test]
async fn ordinary_feed_flash_keeps_published_inputs_and_original_phase_stream_readback()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use pse_relations::{columnar::RelationRow as _, generated::runtime::modeling_reports};

    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package(false).await?;
    let services = runtime.execution_services();
    let declarations = package.declarations(services.preparation()).await?;
    for (name, isothermal) in [
        ("ordinary_feed_flash_isothermal", true),
        ("ordinary_feed_flash_isenthalpic", false),
    ] {
        let selected = declarations
            .iter()
            .find(|row| row.name == name)
            .ok_or("ordinary feed-flash case absent")?;
        assert_eq!(
            selected.value.kind,
            pse_model::generated::enums::ModelingDeclarationKind::Case
        );
        let original = selected.value.clone();
        let cancel = crate::CancelSource::new();
        let declared = package
            .declared_execution(
                services.preparation(),
                selected.declaration_id,
                preparation.compiler,
                pr_jacobian_tests::numerical_solver(crate::math::solves::SolverProfile {
                    intent: pse_backend_native::solve::SolveIntent::Root,
                    numerics: pse_model::numerics::NumericalPolicy {
                        strict_engineering_context: true,
                        ..Default::default()
                    },
                    ..Default::default()
                })?,
                Default::default(),
                preparation.limits,
                &cancel,
            )
            .await?;
        let execution = package
            .execute_declared(&services, &declared, &cancel)
            .await
            .map_err(|error| {
                let message = error.to_string();
                let model = &declared.model.compiled().model;
                let targets = model
                    .symbols
                    .values()
                    .filter(|symbol| message.contains(&symbol.id.to_string()))
                    .map(|symbol| symbol.lineage.path.as_str())
                    .collect::<Vec<_>>();
                format!("{name}: {message}; original targets: {targets:?}")
            })?;
        let DeclaredAlgebraicOutcome::Solve(result) = execution else {
            return Err("ordinary feed flash must execute its authored solve".into());
        };
        assert!(result.accepted, "{name}: {:?}", result.diagnostic());
        assert!(!result.checks.is_empty());
        assert!(
            result.checks.iter().all(|check| check.satisfied),
            "{:?}",
            result.checks
        );
        let scalar = |label: &str| -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
            Ok(result
                .reports
                .iter()
                .find(|row| row.label == label)
                .ok_or_else(|| format!("missing original report: {label}"))?
                .value)
        };
        let inlet = scalar("ordinary feed flash inlet temperature")?;
        let outlet = scalar("ordinary feed flash outlet temperature")?;
        let beta = scalar("ordinary feed flash vapor fraction")?;
        let heat = scalar("feed flash heat duty")?;
        let energy_in = scalar("ordinary feed flash inlet energy")?;
        let energy_out = scalar("ordinary feed flash outlet energy")?;
        assert!((inlet - 368.).abs() <= 0.1);
        assert!(
            (energy_out - energy_in - heat).abs() <= 1.,
            "original energy closure: {energy_in}, {energy_out}, {heat}"
        );
        if isothermal {
            assert!((outlet - inlet).abs() <= 0.1);
            assert!(
                heat > 10000.,
                "the frozen liquid feed requires latent heat: {heat}"
            );
            // Independent binary Raoult/Rachford-Rice closed form, using the
            // original Poling Wagner2.5,5 CAS rows rather than native psat calls.
            let ratio = |a: f64, b: f64, c: f64, d: f64, tc: f64, pc: f64| {
                let tau = 1. - 368. / tc;
                pc / 101325.
                    * ((a * tau + b * tau.powf(1.5) + c * tau.powf(2.5) + d * tau.powi(5))
                        / (1. - tau))
                        .exp()
            };
            let benzene = ratio(-7.01433, 1.55256, -1.8479, -3.713, 562.16, 4898000.) - 1.;
            let toluene = ratio(-7.316, 1.59425, -1.93165, -3.7222, 591.8, 4106000.) - 1.;
            let expected = -(benzene + toluene) / (2. * benzene * toluene);
            assert!(
                (beta - expected).abs() <= 0.001,
                "ordinary vapor fraction: native={beta}, independent={expected}"
            );
        } else {
            assert!(
                outlet < inlet - 1.,
                "adiabatic equilibration cools the frozen liquid feed: {outlet}"
            );
            assert!((0. ..1.).contains(&beta));
            assert!(heat.abs() <= 1.);
        }
        let table = result.table("runtime.modeling_reports")?;
        let readback = modeling_reports::Row::rows(&table)?;
        assert_eq!(readback.len(), result.reports.len());
        for row in &readback {
            let report = result
                .reports
                .iter()
                .find(|report| {
                    report.target_id == row.target_id && report.source_id == row.source_id
                })
                .ok_or("foreign readback target/source")?;
            assert_eq!(
                (row.source_id, row.quantity_id, row.unit_id, row.value),
                (
                    report.source_id,
                    report.quantity_id,
                    report.unit_id,
                    report.value
                )
            );
            assert_eq!(row.run_id, result.run_id);
        }
        let liquid = readback
            .iter()
            .find(|row| row.label == "feed flash liquid flow")
            .ok_or("liquid stream readback absent")?;
        let vapor = readback
            .iter()
            .find(|row| row.label == "feed flash vapor flow")
            .ok_or("vapor stream readback absent")?;
        assert_ne!(
            (liquid.target_id, liquid.source_id),
            (vapor.target_id, vapor.source_id)
        );
        assert_eq!(
            (liquid.quantity_id, liquid.unit_id),
            (vapor.quantity_id, vapor.unit_id)
        );
        assert!((liquid.value + vapor.value - 1.).abs() <= 0.001);
        let after = package.declarations(services.preparation()).await?;
        assert_eq!(
            after
                .iter()
                .find(|row| row.declaration_id == selected.declaration_id)
                .ok_or("original case disappeared")?
                .value,
            original
        );
    }
    Ok(())
}
