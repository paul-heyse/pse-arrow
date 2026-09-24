// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::fixtures::*;
use pse_backend_native::solve::{Backend, HessianMode};
use pse_ids::{SemanticId, named_id};
use pse_runtime::{
    CancelSource,
    workflow::{FitProfile, RunReport},
};
use std::collections::BTreeMap;
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
        let heat = sid(&f["vessel_ports"]["heat"]["symbol_id"]);
        let fit_id = named_id(dynamic, kind);
        let dataset = named_id(fit_id, "data");
        b.dataset(serde_json::from_value(serde_json::json!({"dataset_id":dataset,"name":kind,"source":"analytic closed-vessel energy conservation","content_hash":format!("blake3:{}","02".repeat(32))})).unwrap());
        let dynamic_case = named_id(dynamic, "case");
        let mut steady = b
            .declaration_mut()
            .cases
            .iter()
            .find(|c| c.case_id == dynamic_case)
            .unwrap()
            .clone();
        steady.case_id = named_id(fit_id, "steady");
        for v in &mut steady.variables {
            v.fixed = true;
        }
        let steady_id = steady.case_id;
        let heat_row = named_id(steady_id, "measured-heat");
        steady.rows.push(pse_relations::generated::authored::computation_models::AuthoredComputationModelsFieldCasesItemRowsItem {
            row_id:heat_row,quantity_id:sid(&f["functions"]["rate_u"]),lower:None,upper:None,
        });
        steady.instances.iter_mut().find(|i|i.instance_id == named_id(dynamic,"instance.heat")).unwrap().contributions.push(
            pse_relations::generated::authored::computation_models::AuthoredComputationModelsFieldCasesItemInstancesItemContributionsItem {output:0,row_id:Some(heat_row),scale:1.0}
        );
        b.declaration_mut().cases.push(steady);
        let mut experiments = vec![];
        let mut observations = vec![];
        for transient in [false, true] {
            if (kind == "steady" && transient) || (kind == "transient" && !transient) {
                continue;
            }
            let experiment = named_id(fit_id, if transient { "dynamic" } else { "static" });
            let observation = named_id(experiment, "observation");
            experiments.push(serde_json::json!({"experiment_id":experiment,"case_id":if transient{dynamic_case}else{steady_id},"dynamic_id":if transient{Some(dynamic)}else{None::<SemanticId>}}));
            let unit = sid(&f["ids"]["units"][if transient { "energy" } else { "power" }]);
            b.observation(serde_json::from_value(serde_json::json!({"observation_id":observation,"dataset_id":dataset,"target":"energy balance","value":if transient{u+10.}else{10.},"unit_id":unit,"std_dev":1.,"timestamp":null,"tag":null,"source_span":{"document_id":dataset,"start":0,"end":0}})).unwrap());
            observations.push(serde_json::json!({"observation_id":observation,"experiment_id":experiment,"output_id":if transient{named_id(dynamic,"row.output.u")}else{heat_row},"time":if transient{Some(1.)}else{None::<f64>},"included":true,"importance":1.}));
        }
        let model_id = b.declaration_mut().model_id;
        b.fit(serde_json::from_value(serde_json::json!({"fit_id":fit_id,"model_id":model_id,"parameters":[{"symbol_id":heat,"fixed":false,"value":3.,"lower":0.,"upper":20.,"scale":10.}],"experiments":experiments,"observations":observations})).unwrap());
        let mut solver = profile(Backend::Ipopt, 1, 0, true);
        solver.controls.hessian = HessianMode::LimitedMemory;
        let mut simulation = simulation();
        simulation.sensitivities = true;
        let profile = FitProfile {
            solver,
            simulations: if kind == "steady" {
                BTreeMap::new()
            } else {
                BTreeMap::from([(named_id(fit_id, "dynamic"), simulation)])
            },
            rank_tolerance: 1e-8,
            max_cells: 1 << 20,
        };
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
