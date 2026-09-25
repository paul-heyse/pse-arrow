// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::fixtures::*;
use pse_backend_native::solve::Termination;
use pse_ids::named_id;
use pse_runtime::{CancelSource, workflow::RunReport};

#[tokio::test]
async fn declared_directional_valve_approaches_back_pressure_and_remains_closed_against_reverse_flow()
 {
    for back_pressure in [99_000.0, 101_000.0] {
        let owner = WorkflowRuntime::new().unwrap();
        let mut builder = builder(&owner).await;
        vessel(&mut builder, true);
        let f = json("bindings.json");
        let dynamic = sid(&f["vessel_id"]);
        let case = builder
            .declaration_mut()
            .cases
            .iter_mut()
            .find(|c| c.case_id == named_id(dynamic, "case"))
            .unwrap();
        for (role, value) in [
            ("downstream", back_pressure),
            ("valve_width", 100.0),
            ("valve_k", 1e-4),
            ("heat", 0.0),
        ] {
            let symbol = sid(&f["vessel_ports"][role]["symbol_id"]);
            case.values
                .iter_mut()
                .find(|v| v.symbol_id == symbol)
                .unwrap()
                .value = value;
        }
        let revision = builder.freeze().unwrap();
        let mut settings = simulation();
        settings.end = 2000.0;
        settings.samples = vec![0.0, 1.0, 10.0, 100.0, 1000.0, 2000.0];
        let result = revision
            .prepare_simulation(dynamic, settings, compiler(), &CancelSource::new())
            .await
            .unwrap()
            .start()
            .unwrap()
            .wait()
            .await
            .unwrap();
        let RunReport::Simulation(report) = result.report().unwrap() else {
            panic!("missing simulation")
        };
        assert_eq!(
            report.termination,
            pse_backend_native::dynamics::Termination::Completed,
            "{report:?}"
        );
        conservation(&result, report.samples.len() * 2);
        let first = &report.samples[0].outputs;
        let last = &report.samples.last().unwrap().outputs;
        if back_pressure < first[4] {
            assert!(last[0] < first[0]);
            assert!(
                last[4] >= back_pressure - 1e-3 && last[4] - back_pressure < 2.0,
                "final pressure {}",
                last[4]
            );
            assert!(
                report
                    .samples
                    .windows(2)
                    .all(|w| w[1].outputs[0] <= w[0].outputs[0] + 1e-8)
            );
        } else {
            near(last[0], first[0], 1e-7);
            near(last[1], first[1], 1e-5);
        }
    }
}

#[tokio::test]
async fn vessel_fit() {
    let owner = WorkflowRuntime::new().unwrap();
    let mut b = builder(&owner).await;
    vessel(&mut b, false);
    let f = json("bindings.json");
    let dynamic = sid(&f["vessel_id"]);
    let revision = b.freeze().unwrap();
    let mut p = simulation();
    p.sensitivities = true;
    let prepared = revision
        .prepare_simulation(dynamic, p, compiler(), &CancelSource::new())
        .await
        .unwrap();
    let result = prepared.start().unwrap().wait().await.unwrap();
    let RunReport::Simulation(report) = result.report().unwrap() else {
        panic!()
    };
    assert_eq!(
        report.termination,
        pse_backend_native::dynamics::Termination::Completed,
        "{report:?}"
    );
    conservation(&result, report.samples.len() * 2);
    let n = f["expected"]["n0"].as_f64().unwrap();
    let u = f["expected"]["u0"].as_f64().unwrap();
    for sample in &report.samples {
        near(sample.outputs[0], n, 1e-6);
        near(sample.outputs[1], u + 10. * sample.time, 1e-3);
        near(sample.output_sensitivities[4 + 3], sample.time, 2e-4);
        near(sample.outputs[3] * 0.1, sample.outputs[0], 1e-6);
    }
    assert_eq!(
        result
            .table("authored.dynamic_cases")
            .unwrap()
            .batch()
            .num_rows(),
        1
    );
    let mut bad = simulation();
    bad.atol = vec![1e-8; 4];
    assert!(
        revision
            .prepare_simulation(dynamic, bad, compiler(), &CancelSource::new())
            .await
            .is_err()
    );
    // Finite input discontinuities create explicit event records, with right-hand values.
    let mut changed = simulation();
    changed.changes = vec![pse_backend_native::dynamics::InputChange {
        time: 0.5,
        parameters: vec![
            0.,
            0.,
            f["vessel_values"]["inlet_enthalpy"].as_f64().unwrap(),
            0.,
        ],
    }];
    let result = revision
        .prepare_simulation(dynamic, changed, compiler(), &CancelSource::new())
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap();
    let RunReport::Simulation(report) = result.report().unwrap() else {
        panic!()
    };
    conservation(&result, report.samples.len() * 2);
    assert_eq!(report.events.len(), 1);
    near(report.samples.last().unwrap().outputs[1], u + 5., 2e-3);
    // One shared physical parameter in steady, transient and mixed observation sets.
    for kind in ["steady", "transient", "mixed"] {
        let mut b = revision.edit();
        let (fit_id, profile) = heat_fit(&mut b, kind);
        let fit = b
            .freeze()
            .unwrap()
            .prepare_fit(fit_id, profile, compiler(), &CancelSource::new())
            .await
            .unwrap();
        let result = fit.start().unwrap().wait().await.unwrap();
        let RunReport::Fit(report) = result.report().unwrap() else {
            panic!()
        };
        assert!(
            matches!(
                report.solve.as_ref().unwrap().termination.category,
                Termination::Success | Termination::Acceptable
            ),
            "{report:?}"
        );
        assert!(
            report
                .quality
                .as_ref()
                .is_some_and(|quality| quality.feasible()),
            "{report:?}"
        );
        near(report.candidate.as_ref().unwrap()[0], 10., 2e-3);
        assert_eq!(report.rank, Some(1));
        for v in report.responses.as_ref().unwrap().col(0).iter() {
            near(*v, 1., 2e-3);
        }
        assert!(report.objective.unwrap() < 1e-6);
        conservation(&result, if kind == "steady" { 0 } else { 2 });
        assert_eq!(
            result
                .table("authored.fit_cases")
                .unwrap()
                .batch()
                .num_rows(),
            1
        );
    }
}

#[tokio::test]
async fn higher_index_vessel_is_refused_before_native_start() {
    let owner = WorkflowRuntime::new().unwrap();
    let mut b = builder(&owner).await;
    vessel(&mut b, false);
    let dynamic = sid(&json("bindings.json")["vessel_id"]);
    b.declaration_mut()
        .definitions
        .iter_mut()
        .find(|d| d.definition_id == named_id(dynamic, "definition.closure_n"))
        .unwrap()
        .sources[0] = "n - n0".into();
    let error = b
        .freeze()
        .unwrap()
        .prepare_simulation(dynamic, simulation(), compiler(), &CancelSource::new())
        .await
        .unwrap_err();
    assert!(
        error.to_string().contains("structural deficiency")
            && error
                .to_string()
                .contains(&format!("{:?}", named_id(dynamic, "row.closure_n"))),
        "{error}"
    );
}

#[tokio::test]
async fn fitted_vessel_distinguishes_optimum_from_parameter_identifiability() {
    let owner = WorkflowRuntime::new().unwrap();
    let mut b = builder(&owner).await;
    vessel(&mut b, false);
    let f = json("bindings.json");
    let (fit_id, mut settings) = heat_fit(&mut b, "transient");
    let inlet = sid(&f["vessel_ports"]["inlet_enthalpy"]["symbol_id"]);
    let h = f["vessel_values"]["inlet_enthalpy"].as_f64().unwrap();
    b.sources_mut().fits.iter_mut().find(|fit| fit.fit_id == fit_id).unwrap()
        .parameters.push(serde_json::from_value(serde_json::json!({
            "symbol_id":inlet, "fixed":false, "value":h, "lower":h-1000., "upper":h+1000., "scale":1000.
        })).unwrap());
    settings.solver.numerics.requirements.push(requirement(
        b.declaration_mut().model_id,
        None,
        inlet,
        "variable",
        1e-6,
    ));
    let result = b
        .freeze()
        .unwrap()
        .prepare_fit(fit_id, settings, compiler(), &CancelSource::new())
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap();
    let RunReport::Fit(report) = result.report().unwrap() else {
        panic!()
    };
    assert!(
        matches!(
            report.solve.as_ref().unwrap().termination.category,
            Termination::Success | Termination::Acceptable
        ),
        "{report:?}"
    );
    assert!(report.quality.as_ref().unwrap().feasible(), "{report:?}");
    assert!(report.objective.unwrap() < 1e-6, "{report:?}");
    assert_eq!(report.candidate.as_ref().unwrap().len(), 2);
    assert_eq!(
        report.rank,
        Some(1),
        "zero inflow leaves inlet enthalpy unidentifiable: {report:?}"
    );
    conservation(&result, 2);
}

#[tokio::test]
async fn valve_event_reset_preserves_conservation_and_changes_mode() {
    let owner = WorkflowRuntime::new().unwrap();
    let mut b = builder(&owner).await;
    vessel(&mut b, true);
    let f = json("bindings.json");
    let dynamic = sid(&f["vessel_id"]);
    let source = b.declaration_mut();
    let model = source.model_id;
    let case_id = named_id(dynamic, "case");
    let template = source
        .definitions
        .iter()
        .find(|d| d.definition_id == named_id(dynamic, "definition.amount_in"))
        .unwrap()
        .clone();
    let instance = source
        .cases
        .iter()
        .find(|c| c.case_id == case_id)
        .unwrap()
        .instances[0]
        .clone();
    for (role, expression, quantity) in [("valve_guard", "n - 0.9998 * n0", "amount")] {
        let mut d = template.clone();
        d.definition_id = named_id(dynamic, role);
        d.sources = vec![expression.into()];
        d.providers.clear();
        d.units = vec![
            serde_json::from_value(
                serde_json::json!({"spelling":"1", "unit_id":sid(&f["ids"]["units"]["neutral"])}),
            )
            .unwrap(),
        ];
        source.definitions.push(d.clone());
        let mut i = instance.clone();
        i.instance_id = named_id(d.definition_id, "instance");
        i.definition_id = d.definition_id;
        let row = named_id(dynamic, &format!("row.{role}"));
        i.contributions = vec![pse_relations::generated::authored::computation_models::AuthoredComputationModelsFieldCasesItemInstancesItemContributionsItem {output:0,row_id:Some(row),scale:1.0}];
        let c = source
            .cases
            .iter_mut()
            .find(|c| c.case_id == case_id)
            .unwrap();
        c.instances.push(i);
        c.rows.push(serde_json::from_value(serde_json::json!({"row_id":row,"quantity_id":sid(&f["ids"]["quantities"][quantity]),"lower":null,"upper":null})).unwrap());
    }
    let d = b
        .sources_mut()
        .dynamics
        .iter_mut()
        .find(|d| d.dynamic_id == dynamic)
        .unwrap();
    assert_eq!(d.model_id, model);
    d.modes.push(d.modes[0].clone());
    d.modes[0].events.push(serde_json::from_value(serde_json::json!({"event_id":named_id(dynamic,"close-valve"),"guard_row":named_id(dynamic,"row.valve_guard"),"terminal":false,"next_mode":1,"tolerance":1e-9,"reset_rows":d.outputs})).unwrap());
    for balance in b
        .sources_mut()
        .balances
        .iter_mut()
        .filter(|b| b.case_id == case_id)
    {
        for term in &mut balance.terms {
            if term.role == pse_relations::generated::enums::BalanceRole::Outlet {
                term.mode = Some(0);
            }
        }
    }
    let revision = b.freeze().unwrap();
    let result = revision
        .prepare_simulation(dynamic, simulation(), compiler(), &CancelSource::new())
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap();
    let RunReport::Simulation(r) = result.report().unwrap() else {
        panic!()
    };
    assert_eq!(
        r.termination,
        pse_backend_native::dynamics::Termination::Completed,
        "{r:?}"
    );
    conservation(&result, r.samples.len() * 2);
    assert_eq!(r.events.len(), 1);
    let event = &r.events[0];
    assert!(event.time > 0. && event.time < 1.);
    for (a, b) in event.before.iter().zip(event.after.as_ref().unwrap()) {
        near(*a, *b, 1e-8);
    }
    near(
        r.samples.last().unwrap().outputs[0],
        f["expected"]["n0"].as_f64().unwrap() * 0.9998,
        1e-5,
    );
}
