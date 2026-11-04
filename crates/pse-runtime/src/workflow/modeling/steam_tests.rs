// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit prepared production registrations, actual authored steam equipment,
//! native root solves, and retained original-model conformance observations.
use super::*;
use crate::{math::solves::NumericalInputs, workflow::tests as fixture};
use pse_backend_native::coolprop::{self, WaterRegime};
use pse_model::generated::enums::ModelingDeclarationKind as DeclarationKind;
use std::collections::BTreeSet;

async fn steam_package(
    runtime: &Runtime,
    physical: PhysicalContext,
    providers: BTreeMap<String, pse_kernels::Registration>,
    wrong_component: bool,
) -> ModelingPackage {
    use pse_admission::authoring_driver::document::{
        OwnedDocumentSet, load_package_documents_owned,
    };
    let token = pse_columnar::CancellationToken::new();
    let pool = runtime.shared.pool();
    let bundles = crate::scientific_inputs::family_packages("steam")
        .into_iter()
        .map(|(_, mut files)| {
            if wrong_component {
                for (path, bytes) in &mut files {
                    if path.ends_with(".pse") {
                        let source = std::str::from_utf8(bytes).unwrap();
                        *bytes = source
                            .replace(
                                "water_component=chem.water",
                                "water_component=chem.nitrogen",
                            )
                            .into_bytes();
                    }
                }
            }
            load_package_documents_owned(
                &files,
                runtime.registry(),
                Default::default(),
                &pool,
                &token,
                &runtime.validation_context(),
            )
            .unwrap()
        })
        .collect();
    let documents = OwnedDocumentSet::try_from_bundles(bundles, &pool, &token).unwrap();
    runtime
        .modeling_from_documents_registered(
            &documents,
            physical,
            providers,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap()
}

async fn conform(name: &str, worker: Option<&std::path::Path>) {
    use std::io::Write as _;
    let started = std::time::Instant::now();
    let child_observations = Arc::new(Mutex::new(Vec::new()));
    let runtime = fixture::runtime_with_workspace(128 << 20);
    let services = runtime.execution_services();
    let physical = fixture::physical();
    let registrations: BTreeMap<_, _> = [WaterRegime::Liquid, WaterRegime::Vapor]
        .into_iter()
        .map(|regime| {
            (
                regime.implementation().to_owned(),
                match worker {
                    None => coolprop::registration(&physical.quantities, regime).unwrap(),
                    Some(path) => crate::scientific_worker::LocalWaterFactory::registration(
                        crate::scientific_worker::PreparedExecutable::observe(path, 64 << 20)
                            .unwrap()
                            .measured(child_observations.clone()),
                        &physical.quantities,
                        regime,
                        runtime.shared.math().clone(),
                    )
                    .unwrap(),
                },
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
        let rejected = steam_package(&runtime, physical.clone(), registrations.clone(), true).await;
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
    let package = steam_package(&runtime, physical, registrations, false).await;
    let prepare_ns = started.elapsed().as_nanos();
    let mut conform_ns = Vec::new();
    let mut phase_pool_high_water = Vec::new();
    let mut retained_after_conform = Vec::new();
    for _ in 0..2 {
        runtime.shared.reset_observation_peak();
        let solve_started = std::time::Instant::now();
        let report = package
            .conform(&services, policy.clone(), &crate::CancelSource::new())
            .await
            .unwrap();
        conform_ns.push(solve_started.elapsed().as_nanos());
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
        let crate::math::solves::Outcome::Native(native) = &report.results[&selected].outcome
        else {
            panic!("actual authored steam fixture did not produce a native solve");
        };
        assert_eq!(native.backend, pse_backend_native::solve::Backend::Kinsol);
        drop(report);
        phase_pool_high_water.push(runtime.shared.observation_peak_bytes());
        retained_after_conform.push(runtime.shared.report().unwrap().pool_reserved_now);
    }
    let drain_started = std::time::Instant::now();
    runtime.close();
    runtime.clear_program_cache();
    runtime.shared.math().shutdown_native_lanes().await.unwrap();
    let drain_ns = drain_started.elapsed().as_nanos();
    let total_ns = started.elapsed().as_nanos();
    let resources = runtime.shared.report().unwrap();
    let children = child_observations.lock().unwrap().clone();
    for child in &children {
        assert!(
            child.after_close_ack && child.error.is_none() && child.identity.is_some(),
            "{child:?}"
        );
        assert!(
            child
                .peak_resident_bytes_at_close
                .is_some_and(|bytes| bytes > 0)
        );
        assert_eq!(child.exit_code, Some(0));
        assert!(child.signal.is_none());
        assert!(!std::path::Path::new(&format!("/proc/{}", child.pid)).exists());
    }
    if worker.is_some() {
        assert!(
            !children.is_empty(),
            "equipment must execute through actual worker processes"
        );
    }
    writeln!(std::io::stderr().lock(), "PSE_SCIENTIFIC_WORKER_EQUIPMENT_MEASUREMENT={}", serde_json::json!({
        "case": name, "placement": if worker.is_some() { "local-worker" } else { "in-process" },
        "condition": "fresh runtime and admitted positive package; repeat the same package with reused preparation; pump also retains its prior wrong-CAS intake control",
        "prepare_ns": prepare_ns, "fresh_conform_ns": conform_ns[0], "repeated_conform_ns": conform_ns[1],
        "drain_ns": drain_ns, "total_including_prepare_repeated_case_and_drain_ns": total_ns,
        "accounted_conform_phase_high_water_bytes": phase_pool_high_water,
        "accounted_retained_after_conform_bytes": retained_after_conform,
        "accounted_retained_after_drain_bytes": resources.pool_reserved_now,
        "accounted_all_phases_high_water_bytes": resources.pool_peak_bytes,
        "parent_cumulative_vm_hwm_bytes": resources.process_peak_rss_bytes,
        "configured_opaque_allowance_bytes_per_child": if worker.is_some() { Some(64usize << 20) } else { None },
        "child_snapshots_after_native_close_before_transport_shutdown": children,
    })).unwrap();
}

#[tokio::test]
async fn authored_liquid_pump_matches_independent_hydraulic_power_and_iapws_pressure() {
    conform("liquid_pump", None).await;
}

#[tokio::test]
async fn authored_steam_turbine_matches_released_single_phase_reference() {
    conform("vapor_turbine", None).await;
}

#[tokio::test]
async fn authored_hot_steam_turbine_matches_second_released_single_phase_reference() {
    conform("hot_vapor_turbine", None).await;
}

fn scientific_worker_executable() -> std::path::PathBuf {
    std::env::var_os("PSE_SCIENTIFIC_WORKER_BINARY")
        .map(std::path::PathBuf::from)
        .expect("source-built scientific worker executable fixture must be explicitly supplied by the native correctness owner")
}

#[tokio::test]
async fn authored_liquid_pump_through_prepared_scientific_worker_keeps_original_controls() {
    conform("liquid_pump", Some(&scientific_worker_executable())).await;
}

#[tokio::test]
async fn authored_steam_turbine_through_prepared_scientific_worker_keeps_original_controls() {
    conform("vapor_turbine", Some(&scientific_worker_executable())).await;
}

#[tokio::test]
async fn authored_hot_steam_turbine_through_prepared_scientific_worker_keeps_original_controls() {
    conform("hot_vapor_turbine", Some(&scientific_worker_executable())).await;
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
