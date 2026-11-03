// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original-case precision inventory; initialization-stage and result checks remain separate.
use super::*;
use crate::math::solves::SolverProfile;
use pse_backend_native::solve::SolveIntent;

#[tokio::test]
async fn ordinary_precision_inventory_retains_instance_sources_and_fallbacks()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package(false).await?;
    let services = runtime.execution_services();
    let declarations = package.declarations(services.preparation()).await?;
    let cancel = crate::CancelSource::new();
    for case_name in ["ordinary_recycle", "ordinary_exchanger", "ordinary_rating"] {
        let case = declarations
            .iter()
            .find(|row| row.name == case_name)
            .ok_or("case missing")?;
        let execution = package
            .declared_execution(
                services.preparation(),
                case.declaration_id,
                preparation.compiler,
                SolverProfile {
                    intent: SolveIntent::Root,
                    numerics: pse_model::numerics::NumericalPolicy {
                        strict_engineering_context: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Default::default(),
                preparation.limits,
                &cancel,
            )
            .await?;
        let prepared = package
            .prepare_analysis(services.preparation(), &execution.analysis, &cancel)
            .await?;
        let model = &prepared.model.model.compiled().model;
        let policy = prepared.solve.numerics();
        eprintln!(
            "case={case_name} revision={} controls={:?} targets={}",
            package.revision_id(),
            prepared.solve.accuracy(),
            policy.targets.len()
        );
        let mut found = BTreeMap::<String, Vec<(SemanticId, _, bool)>>::new();
        for target in &policy.targets {
            let owner = model
                .symbols
                .get(&target.id)
                .map(|symbol| &symbol.lineage)
                .or_else(|| {
                    model
                        .equations
                        .iter()
                        .find(|row| row.id == target.id)
                        .map(|row| &row.lineage)
                });
            let engineering = target.engineering.as_ref();
            assert!(
                !engineering.is_some_and(|context| context.canonical_fallback),
                "{case_name} target {} retains a canonical fallback",
                target.id
            );
            eprintln!(
                "case={case_name} id={} role={:?} instance={:?} path={:?} quantity={} unit={} nominal={} coordinate_scale={} budget={} characteristic={:?} fallback={:?} caloric={:?}",
                target.id,
                target.kind,
                owner.map(|owner| owner.instance),
                owner.map(|owner| &owner.path),
                target.quantity,
                target.unit,
                target.nominal,
                target.coordinate_scale,
                target.budget,
                engineering.and_then(|context| context.characteristic),
                engineering.map(|context| context.canonical_fallback),
                engineering.and_then(|context| context.caloric.as_ref())
            );
            if let Some(context) = engineering.and_then(|context| context.caloric.as_ref()) {
                assert_eq!(context.science_revision, package.revision_id());
                if context.response.is_some() {
                    assert!(!context.science_records.is_empty());
                }
                assert!(!context.consumers.is_empty());
                assert!(
                    context
                        .consumers
                        .iter()
                        .all(|consumer| consumer.power_rule_id != SemanticId::NIL
                            && consumer.actual_flow_id != SemanticId::NIL
                            && !consumer.exposure_expression.is_empty())
                );
                assert!(!engineering.unwrap().canonical_fallback);
                if let Some(owner) = owner {
                    for consumer in &context.consumers {
                        let flow = model
                            .symbols
                            .get(&consumer.actual_flow_id)
                            .expect("actual consuming flow must retain its semantic symbol");
                        assert_eq!(
                            flow.lineage.instance, owner.instance,
                            "exposure must belong to the target's actual state instance"
                        );
                    }
                    found.entry(owner.path.clone()).or_default().push((
                        target.id,
                        owner.instance,
                        context.response.is_some(),
                    ));
                }
            }
            if owner.is_some_and(|owner| {
                owner.path.ends_with(".h")
                    || owner.path.ends_with(".enthalpy")
                    || owner.path.ends_with(".enthalpy_binding")
            }) {
                assert!(
                    engineering
                        .and_then(|context| context.caloric.as_ref())
                        .is_some(),
                    "missing enthalpy context for {}",
                    target.id
                );
            }
        }
        if case_name == "ordinary_recycle" {
            for suffix in [
                "splitter.unit.inlet.h",
                "splitter.unit.inlet.enthalpy",
                "mixer.unit.inlet.h",
                "mixer.unit.inlet.enthalpy",
                "mixer.unit.inlet.enthalpy_binding",
                "feed.state.h",
                "feed.state.enthalpy_binding",
                "mixer.unit.outlet.h",
                "mixer.unit.outlet.enthalpy_binding",
                "heater.unit.inlet.h",
                "heater.unit.inlet.enthalpy_binding",
                "heater.unit.outlet.h",
                "heater.unit.outlet.enthalpy_binding",
                "flash.unit.inlet.h",
                "flash.unit.inlet.enthalpy_binding",
                "flash.unit.outlet.h",
                "flash.unit.outlet.enthalpy_binding",
            ] {
                assert!(
                    found.keys().any(|path| path.ends_with(suffix)),
                    "missing selected caloric target {suffix}; found {found:?}"
                );
            }
            // Indexed inlet paths omit coordinates; instance and target identities distinguish
            // the single-phase recycle inlet from the fresh equilibrium inlet.
            let entries = |suffix: &str| {
                found
                    .iter()
                    .find(|(path, _)| path.ends_with(suffix))
                    .unwrap()
                    .1
            };
            let inlet_h = entries("mixer.unit.inlet.h");
            let recycle = inlet_h
                .iter()
                .find(|(_, _, thermal)| *thermal)
                .expect("recycle inlet thermal context");
            let fresh = inlet_h
                .iter()
                .find(|(_, _, thermal)| !*thermal)
                .expect("fresh inlet energy context");
            assert_ne!(recycle.0, fresh.0);
            assert_ne!(recycle.1, fresh.1);
            assert!(
                entries("mixer.unit.inlet.enthalpy")
                    .iter()
                    .any(|(_, instance, thermal)| *instance == recycle.1 && *thermal)
            );
            assert!(
                entries("mixer.unit.inlet.enthalpy_binding")
                    .iter()
                    .any(|(_, instance, thermal)| *instance == fresh.1 && !*thermal)
            );
        } else {
            for suffix in [
                "exchanger.hot.inlet.h",
                "exchanger.hot.inlet.enthalpy_binding",
                "exchanger.hot.outlet.h",
                "exchanger.hot.outlet.enthalpy_binding",
                "exchanger.cold.inlet.h",
                "exchanger.cold.inlet.enthalpy_binding",
                "exchanger.cold.outlet.h",
                "exchanger.cold.outlet.enthalpy_binding",
            ] {
                assert!(
                    found.keys().any(|path| path.ends_with(suffix)),
                    "missing PR energy target {suffix}; found {found:?}"
                );
            }
        }
    }
    Ok(())
}
