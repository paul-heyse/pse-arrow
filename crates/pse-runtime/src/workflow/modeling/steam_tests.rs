// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit prepared production registrations, actual authored steam equipment,
//! native root solves, and retained original-model conformance observations.
use super::*;
use crate::{math::solves::NumericalInputs, workflow::tests as fixture};
use pse_backend_native::coolprop::{self, WaterRegime};
use pse_model::generated::enums::ModelingDeclarationKind as DeclarationKind;
use std::collections::BTreeSet;

async fn conform(name: &str) {
    let runtime = fixture::runtime_with_workspace(128 << 20);
    let services = runtime.execution_services();
    let physical = fixture::physical();
    let registrations: BTreeMap<_, _> = [WaterRegime::Liquid, WaterRegime::Vapor]
        .into_iter()
        .map(|regime| {
            (
                regime.implementation().to_owned(),
                coolprop::registration(&physical.quantities, regime).unwrap(),
            )
        })
        .collect();
    let source = crate::scientific_inputs::family_sources("steam");
    let rows = crate::scientific_inputs::parse_sources(
        &source,
        pse_authoring::language::IdentityPolicy::Named,
    );
    let selected = rows
        .iter()
        .find(|row| row.name == name && row.value.kind == DeclarationKind::Test)
        .unwrap()
        .declaration_id;
    let policy = ModelingConformancePolicy {
        compiler: fixture::compiler_profile(),
        solver: fixture::profile(),
        numerical: NumericalInputs::default(),
        limits: Limits::default(),
        derivatives: pse_backend_native::derivative_diagnostics::Policy {
            perturbation: 1e-6,
            relative_tolerance: 1e-4,
            maximum_cells: 100,
        },
        maximum_fixtures: 1,
        maximum_checks: 64,
        fixtures: ModelingFixtureSelection::Selected(BTreeSet::from([selected])),
        diagnostics: None,
    };
    if name == "liquid_pump" {
        // Change the actual constructor selection to an existing, identified
        // non-water species. Admission must refuse its CAS before any solve.
        let wrong = source.replace(
            "water_component=chem.water",
            "water_component=chem.nitrogen",
        );
        assert_ne!(wrong, source);
        let rejected = runtime
            .modeling_package_registered(
                crate::scientific_inputs::parse_sources(
                    &wrong,
                    pse_authoring::language::IdentityPolicy::Named,
                ),
                physical.clone(),
                registrations.clone(),
            )
            .await
            .unwrap();
        let refusal = rejected
            .conform(&services, policy.clone(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(!refusal.passed());
        assert!(
            refusal.results.is_empty(),
            "wrong CAS must not reach a native solve"
        );
        assert!(
            refusal.checks.iter().any(|check| check
                .message
                .contains("HEOS Water state requires an explicitly identified water component")),
            "{:?}; failures={:?}",
            refusal.checks,
            refusal.failures
        );
    }
    // This is the public prepared registration route used by downstream callers;
    // no source-name dispatch, test provider, or separate thermodynamic solver.
    let package = runtime
        .modeling_package_registered(rows, physical, registrations)
        .await
        .unwrap();
    let report = package
        .conform(&services, policy, &crate::CancelSource::new())
        .await
        .unwrap();
    assert!(report.complete, "{:?}", report.checks);
    assert!(
        report.passed(),
        "{:?}; failures={:?}",
        report.checks,
        report.failures
    );
    assert!(
        report.results.contains_key(&selected),
        "an actual native result must be retained"
    );
    let crate::math::solves::Outcome::Native(native) = &report.results[&selected].outcome else {
        panic!("actual authored steam fixture did not produce a native solve");
    };
    assert_eq!(native.backend, pse_backend_native::solve::Backend::Kinsol);
    runtime.close();
    runtime.clear_program_cache();
    runtime.shared.math().shutdown_native_lanes().await.unwrap();
}

#[tokio::test]
async fn authored_liquid_pump_matches_independent_hydraulic_power_and_iapws_pressure() {
    conform("liquid_pump").await;
}

#[tokio::test]
async fn authored_steam_turbine_matches_released_single_phase_reference() {
    conform("vapor_turbine").await;
}

#[tokio::test]
async fn authored_hot_steam_turbine_matches_second_released_single_phase_reference() {
    conform("hot_vapor_turbine").await;
}

#[tokio::test]
async fn document_ingress_uses_explicit_prepared_water_without_a_second_source_path() {
    use pse_admission::authoring_driver::document::{
        OwnedDocumentSet, load_package_documents_owned,
    };
    let runtime = fixture::runtime();
    let physical = fixture::physical();
    let provider = coolprop::registration(physical.quantities(), WaterRegime::Vapor).unwrap();
    let source = format!(
        "package water_point {{ fn pressure(T:Temperature,rho:Density)->Pressure external \"coolprop8-water-vapor\" revision \"{}\" data \"{}\" output 0 derivatives 2 source analytic smoothness 2; def Point {{ var result:Pressure; eq binding:result==pressure(500{{K}},{}{{mol/m^3}}); annotation start result(100000{{Pa}}); annotation nominal result(100000{{Pa}}); }} test point fixture {{ dof 0; route steady; procedure solve; }} {{ child root:Point=Point(); expect root.result==99967.9423{{Pa}} tolerance 0.02{{Pa}}; }} }}",
        provider.spec().revision,
        provider.spec().data,
        0.435 / 0.018_015_268,
    );
    let manifest =
        include_str!("../../../../../tests/fixtures/packages/minimal_explicit/package.toml")
            .replace("id_policy = \"explicit\"", "id_policy = \"named\"");
    let pool = runtime.shared.pool();
    let token = pse_columnar::CancellationToken::new();
    let bundle = load_package_documents_owned(
        &BTreeMap::from([
            ("package.toml".into(), manifest.into_bytes()),
            ("models/water-point.pse".into(), source.into_bytes()),
        ]),
        runtime.registry(),
        Default::default(),
        &pool,
        &token,
        &runtime.validation_context(),
    )
    .unwrap();
    let documents = OwnedDocumentSet::try_from_bundles(vec![bundle], &pool, &token).unwrap();
    let package = runtime
        .modeling_from_documents_registered(
            &documents,
            physical,
            BTreeMap::from([(WaterRegime::Vapor.implementation().to_owned(), provider)]),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let services = runtime.execution_services();
    let selected = package
        .declarations(services.preparation())
        .await
        .unwrap()
        .iter()
        .find(|row| row.name == "point")
        .unwrap()
        .declaration_id;
    let report = package
        .conform(
            &services,
            ModelingConformancePolicy {
                compiler: fixture::compiler_profile(),
                solver: fixture::profile(),
                numerical: Default::default(),
                limits: Default::default(),
                derivatives: pse_backend_native::derivative_diagnostics::Policy {
                    perturbation: 1e-6,
                    relative_tolerance: 1e-4,
                    maximum_cells: 100,
                },
                maximum_fixtures: 1,
                maximum_checks: 16,
                fixtures: ModelingFixtureSelection::Selected(BTreeSet::from([selected])),
                diagnostics: None,
            },
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert!(
        report.passed(),
        "{:?}; failures={:?}",
        report.checks,
        report.failures
    );
    runtime.close();
    runtime.clear_program_cache();
    runtime.shared.math().shutdown_native_lanes().await.unwrap();
}
