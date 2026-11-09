// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Feed-flash specifications consume the original BT phase and balance owners.
use super::*;
use crate::{
    math::solves::NumericalInputs,
    reference_source_fixture::{reference_family_sources, rows},
    workflow::{PreparationSettings, tests as fixture},
};
use pse_backend_native::solve::{Backend, SolverSelection};
use pse_kernels::{DerivativeOrder, ProviderSelectionRegularity};
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
    conform_admitted(&runtime, &package, selected, &PreparationSettings::default()).await
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
    conform_admitted(&runtime, &package, selected, &PreparationSettings::default()).await
}

#[tokio::test]
async fn nested_bt_result_observes_actual_liquid_two_phase_and_vapor_invocations() {
    use pse_modeling::specialize::{
        Value,
        finite::{FamilyKey, MemberKey},
    };
    use pse_relations::{columnar::RelationRow, generated::runtime::provider_selections};

    let (runtime, package, _spill, preparation) = pr_jacobian_tests::ordinary_delivery_package(false)
        .await
        .unwrap();
    let declarations = package
        .declarations(&runtime.preparation_services())
        .await
        .unwrap();
    let ideal = declarations
        .iter()
        .find(|row| row.name == "IdealFlash")
        .unwrap();
    let split = declarations
        .iter()
        .find(|row| row.name == "split" && row.parent_id == Some(ideal.declaration_id))
        .unwrap();
    let components = [
        "01a0e1461e93764398a669ea78d82d27", // benzene
        "01a0e1461e93764398a669eb046f4eaf", // toluene
    ]
    .map(|id| DeclarationId::from_id(SemanticId::parse_hex(id).unwrap()));
    for (fixture, branch) in [
        ("nested_liquid", "liquid_only"),
        ("nested_two_phase", "two_phase"),
        ("nested_vapor", "vapor_only"),
    ] {
        let selected = declarations
            .iter()
            .find(|row| row.name == fixture)
            .unwrap()
            .declaration_id;
        let authored_regime = declarations
            .iter()
            .find(|row| row.name == branch && row.parent_id == Some(split.declaration_id))
            .unwrap()
            .declaration_id;
        let report = conform_admitted(&runtime, &package, selected, &preparation).await;
        assert!(
            report.complete && report.passed(),
            "{fixture}: {:?}; {:?}", report.checks, report.failures
        );
        let result = &report.results[&selected];
        let table = result.table("runtime.provider_selections").unwrap();
        let rows = provider_selections::Row::rows(&table).unwrap();
        assert_eq!(rows.len(), result.provider_selections().len(), "{fixture}");
        for (ordinal, (row, visit)) in rows.iter().zip(result.provider_selections()).enumerate() {
            let selection = visit.observation.selection();
            assert_eq!(row.run_id, result.run_id);
            assert_eq!(row.step, 0);
            assert_eq!(row.ordinal, ordinal as i64);
            assert_eq!(row.attempt_id, None);
            assert_eq!(row.native_attempt, result.original_attempt as i64);
            assert_eq!(row.instance_id.as_id(), visit.instance);
            assert_eq!(row.source_id, visit.source);
            assert_eq!(row.provider, visit.observation.provider().0);
            assert_eq!(row.invocation, visit.observation.invocation());
            assert_eq!(row.selector_id, selection.selector);
            assert_eq!(row.regime_id, selection.regime);
            assert_eq!(row.policy, selection.policy);
            assert_eq!(row.product, selection.product);
            assert_eq!(row.requested_order, 0);
            assert_eq!(row.selection_order, 0);
            assert_eq!(row.regularity, ProviderSelectionRegularity::Point);
        }
        let compiled = result.prepared.model.model.compiled();
        let (selector, expected) = compiled
            .model
            .regimes
            .iter()
            .find_map(|(selector, selection)| {
                selection.alternatives.iter()
                    .find(|alternative| alternative.declaration == authored_regime)
                    .map(|alternative| (*selector, alternative.id))
            })
            .unwrap();
        let mut visits = result
            .provider_selections()
            .iter()
            .filter(|visit| visit.observation.selection().selector == selector.as_id())
            .peekable();
        let provider = visits
            .peek()
            .unwrap_or_else(|| panic!("{fixture} original invocation absent"))
            .observation
            .provider();
        let registration = result
            .prepared
            .providers
            .values()
            .map(pse_kernels::Registration::spec)
            .find(|spec| spec.key() == provider)
            .unwrap();
        let system = compiled
            .admitted
            .implicit_systems()
            .find(|system| system.descriptor.spec().id == selector.as_id())
            .unwrap();
        let original = system.descriptor.spec();
        assert_eq!(registration.inputs, original.inputs);
        assert_eq!(registration.outputs, original.outputs);

        let model = &compiled.model;
        let outer = compiled
            .numerical_annotations(pse_compiler::workspace::NumericalAnnotationMode::Steady)
            .unwrap();
        for alternative in &model.regimes[&selector].alternatives {
            let residual = system.residuals.iter().find(|r| r.id == alternative.id).unwrap();
            assert_eq!(residual.rows.len(), alternative.equations.len());
            for row in &alternative.equations {
                let outside = outer.target(row.id).unwrap();
                let inside = residual.numerical_annotations.target(row.id).unwrap();
                assert!(outside.nested);
                assert_eq!(outside.declaration, row.lineage.declaration);
                assert_eq!(inside.declaration, row.lineage.declaration);
            }
        }
        let instance = model.instances[&selector].parent.unwrap();
        let declaration = model.instances[&instance].members["z"];
        let tuples = model
            .finite_facts
            .family(&FamilyKey {
                instance,
                declaration,
                role: "binding.indices.domain".into(),
                parents: Vec::new(),
            })
            .unwrap();
        assert_eq!(tuples.len(), components.len());
        for component in components {
            let tuple = tuples
                .iter()
                .find(|tuple| {
                    matches!(tuple.as_slice(), [Value::Entity { id, .. }] if *id == component)
                })
                .unwrap();
            let member = model
                .finite_facts
                .require_member(&MemberKey {
                    instance,
                    declaration,
                    tuple: tuple.clone(),
                })
                .unwrap();
            assert_eq!(
                registration.inputs.iter().filter(|port| port.id == member).count(),
                1,
            );
        }
        for visit in visits {
            let observed = visit.observation.selection();
            assert_ne!(visit.source, SemanticId::NIL);
            assert_eq!(visit.observation.provider(), provider);
            assert_eq!(observed.regime, expected, "{fixture}");
            assert_eq!(visit.observation.requested_order(), DerivativeOrder::Value);
            assert_eq!(observed.selection_order, DerivativeOrder::Value);
            assert_eq!(observed.regularity, ProviderSelectionRegularity::Point);
        }
    }
}

async fn conform_admitted(
    runtime: &Runtime,
    package: &ModelingPackage,
    selected: DeclarationId,
    preparation: &PreparationSettings,
) -> ModelingConformanceReport {
    let services = runtime.execution_services();
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

fn assert_original_feed_exposure(result: &ModelingResult, expected_flow: f64, permitted: bool) {
    use pse_model::generated::enums::ModelingCheckKind;
    use pse_modeling::annotation::AnnotationValue;
    let model = &result.prepared.model.model.compiled().model;
    let registry = &result.prepared.model.case.compiled().quantities;
    let mut selected = BTreeSet::new();
    let mut declarations = BTreeSet::new();
    let mut flows = BTreeSet::new();
    for requirement in &result.prepared.solve.numerics().policy.caloric_requirements {
        for consumer in &requirement.consumers {
            let key = (consumer.declaration_id, consumer.actual_flow_id);
            selected.insert(key);
            declarations.insert(consumer.declaration_id);
            flows.insert(consumer.actual_flow_id);
            let source = model
                .annotations
                .iter()
                .find(|annotation| {
                    annotation.lineage.declaration.as_id() == consumer.declaration_id
                })
                .unwrap();
            assert!(matches!(&source.value, AnnotationValue::CaloricContext(_)));
            assert_eq!(Some(source.lineage.instance), requirement.instance_id);
            let flow = model.symbols.get(&consumer.actual_flow_id).unwrap();
            let quantity = flow
                .ty
                .quantity_scheme()
                .unwrap()
                .resolve(registry, &BTreeMap::new())
                .unwrap();
            assert_eq!(quantity.as_id(), consumer.exposure_quantity_id);
            assert_eq!(
                registry
                    .quantity_type(quantity)
                    .unwrap()
                    .canonical_unit
                    .as_id(),
                consumer.exposure_unit_id
            );
            // Original declaration: capacity1mol/s plus the shared0.001mol/s
            // physical margin. The candidate cannot redefine this frozen premise.
            assert!((consumer.aggregate_exposure - 1.001).abs() <= 1e-12);
            let checks = result
                .checks
                .iter()
                .filter(|check| {
                    check.kind == ModelingCheckKind::Check
                        && check.source_id.as_id() == key.0
                        && check.target_id == key.1
                })
                .collect::<Vec<_>>();
            assert_eq!(
                checks.len(),
                1,
                "one original exposure observation per source/actual-flow member"
            );
            let check = checks[0];
            assert_eq!(check.run_id, result.run_id);
            assert_eq!(
                check.observation_instance,
                requirement.instance_id.map(|id| id.as_id())
            );
            assert!(check.dependencies.contains(&consumer.target_id));
            assert!(check.dependencies.contains(&consumer.power_rule_id));
            assert_eq!(check.tolerance, Some(consumer.aggregate_exposure));
            assert_eq!(check.satisfied, permitted);
            assert!((check.value - expected_flow).abs() <= 0.001);
            assert_eq!(check.value, result.values.scalars[&consumer.actual_flow_id]);
            let bound = check
                .input_values
                .iter()
                .filter(|value| value.name == "frozen_exposure_bound")
                .collect::<Vec<_>>();
            assert_eq!(bound.len(), 1);
            assert_eq!(bound[0].quantity_type, consumer.exposure_quantity_id);
            assert_eq!(bound[0].value, consumer.aggregate_exposure);
        }
    }
    let authored = model
        .annotations
        .iter()
        .filter(|annotation| matches!(&annotation.value, AnnotationValue::CaloricContext(_)))
        .map(|annotation| annotation.lineage.declaration.as_id())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        declarations, authored,
        "complete selected caloric/energy context population"
    );
    assert_eq!(declarations.len(), 4);
    assert_eq!(
        flows.len(),
        2,
        "actual inlet and outlet flow members stay distinct"
    );
    let observed = result
        .checks
        .iter()
        .filter(|check| {
            check
                .input_values
                .iter()
                .any(|value| value.name == "frozen_exposure_bound")
        })
        .map(|check| (check.source_id.as_id(), check.target_id))
        .collect::<BTreeSet<_>>();
    assert_eq!(observed, selected);
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
    use pse_model::generated::enums::{ModelingApplicabilityOutcome, ModelingPermissionTarget};
    use pse_relations::{
        columnar::RelationRow as _,
        generated::runtime::{modeling_checks, modeling_reports},
    };

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
        let claim = |name: &str| {
            declarations
                .iter()
                .find(|row| row.name == name)
                .unwrap()
                .declaration_id
                .as_id()
        };
        let permission = |name: &str| {
            declarations
                .iter()
                .find(|row| row.name == name && row.parent_id == Some(selected.declaration_id))
                .unwrap()
        };
        let unknown = permission("unknown_cp");
        let perry = permission("benzene_liquid_cp_extrapolation");
        ordinary_delivery_tests::assert_unknown_caloric_interval(
            &result,
            claim("feos_interval_unknown"),
        )?;
        ordinary_delivery_tests::assert_perry_extrapolation(
            &result,
            claim("correlation_range"),
            selected.declaration_id.as_id(),
            perry.declaration_id.as_id(),
        )?;
        let original_unknown = unknown.value.permission.as_ref().unwrap();
        assert_eq!(
            original_unknown.target_kind,
            ModelingPermissionTarget::Families
        );
        assert!(original_unknown.allow_unknown && !original_unknown.allow_extrapolation);
        assert_eq!(original_unknown.targets.len(), 1);
        for check in result
            .checks
            .iter()
            .filter(|check| check.claim_id == Some(claim("feos_interval_unknown")))
        {
            assert_eq!(
                check.applicability_outcome,
                Some(ModelingApplicabilityOutcome::UnknownEvidence)
            );
            let authorization = check
                .applicability_permissions
                .iter()
                .find(|authorization| authorization.permission_id == unknown.declaration_id.as_id())
                .unwrap();
            assert_eq!(authorization.scope, selected.declaration_id.as_id());
            assert_eq!(authorization.target_kind, original_unknown.target_kind);
            assert_eq!(authorization.targets, vec![claim("feos_poling_cp")]);
            assert!(authorization.allow_unknown && !authorization.allow_extrapolation);
            assert!(
                !check.permission_ids.contains(&perry.declaration_id.as_id()),
                "a Perry record allowance cannot authorize unknown vapor-Cp evidence"
            );
        }
        assert_original_feed_exposure(&result, 1., true);
        let checked = modeling_checks::Row::rows(&result.table("runtime.modeling_checks")?)?;
        assert_eq!(
            checked, result.checks,
            "complete original qualifier, authorization and exposure facts survive typed readback"
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

#[tokio::test]
async fn ordinary_feed_flash_excess_actual_throughput_is_rejected_by_frozen_exposure()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use pse_relations::{columnar::RelationRow as _, generated::runtime::modeling_checks};
    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package(false).await?;
    let services = runtime.execution_services();
    let declarations = package.declarations(services.preparation()).await?;
    for name in [
        "ordinary_feed_flash_isothermal",
        "ordinary_feed_flash_isenthalpic",
    ] {
        // Strictly beyond capacity+0.001mol/s, while the independent equimolar
        // component expectations remain within0.001mol/s:0.0015/2=0.00075.
        // This isolates the exposure premise without discarding authored checks.
        let excess_flow = 1.0015;
        let selected = declarations.iter().find(|row| row.name == name).unwrap();
        let cancel = crate::CancelSource::new();
        let mut changed = package
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
        // An explicitly re-admitted caller numeric overlay, not a mutation of the
        // immutable authored fixture or a candidate-dependent capacity estimate.
        assert_eq!(
            changed
                .analysis
                .case
                .values
                .insert("root.unit.flash.inlet.flow".into(), excess_flow),
            Some(1.)
        );
        let DeclaredAlgebraicOutcome::Solve(result) = package
            .execute_declared(&services, &changed, &cancel)
            .await?
        else {
            return Err(
                "ordinary excess-flow control must retain its original native solve".into(),
            );
        };
        assert!(
            !result.accepted,
            "over-capacity flow cannot retain the original caloric premise"
        );
        let crate::math::solves::Outcome::Native(native) = &result.outcome else {
            return Err("exposure refusal requires an actual native candidate".into());
        };
        assert!(native.candidate.is_some());
        assert_original_feed_exposure(&result, excess_flow, false);
        let failures = result
            .checks
            .iter()
            .filter(|check| !check.satisfied)
            .collect::<Vec<_>>();
        assert!(!failures.is_empty());
        assert!(
            failures.iter().all(|check| check
                .input_values
                .iter()
                .any(|value| value.name == "frozen_exposure_bound")),
            "{failures:?}"
        );
        let report = |label: &str| {
            result
                .reports
                .iter()
                .find(|row| row.label == label)
                .unwrap()
                .value
        };
        assert!(
            (report("feed flash liquid flow") + report("feed flash vapor flow") - excess_flow)
                .abs()
                <= 0.001
        );
        assert!(
            (report("ordinary feed flash outlet energy")
                - report("ordinary feed flash inlet energy")
                - report("feed flash heat duty"))
            .abs()
                <= 1.
        );
        let readback = modeling_checks::Row::rows(&result.table("runtime.modeling_checks")?)?;
        assert_eq!(
            readback, result.checks,
            "the failed original exposure remains readable with exact attribution"
        );
        assert_eq!(
            package
                .declarations(services.preparation())
                .await?
                .iter()
                .find(|row| row.declaration_id == selected.declaration_id)
                .unwrap()
                .value,
            selected.value
        );
    }
    Ok(())
}
