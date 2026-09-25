// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-backed shared engineering fixtures. No expected value is obtained from the product.
#![allow(
    dead_code,
    reason = "shared acceptance and benchmark fixtures have distinct consumers"
)]
#[path = "workflow_runtime.rs"]
mod workflow_runtime;
use pse_backend_native::solve::{Backend, Controls, SolveIntent, SolverSelection};
use pse_ids::SemanticId;
use pse_runtime::math::solves::{BackendSettings, Outcome, SolverProfile};
use pse_runtime::workflow::{ModelBuilder, ModelRevision, RunReport, RunResult, Runtime};
use std::{collections::BTreeMap, sync::Arc};
pub(crate) use workflow_runtime::WorkflowRuntime;
pub(crate) fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("tests/fixtures/plan14").is_dir())
        .unwrap()
        .join("tests/fixtures/plan14")
        .join(name)
}
pub(crate) fn json(name: &str) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(fixture(name)).unwrap()).unwrap()
}
pub(crate) fn sid(v: &serde_json::Value) -> SemanticId {
    serde_json::from_value(v.clone()).unwrap()
}
pub(crate) fn runtime(owner: &WorkflowRuntime) -> Runtime {
    Runtime::from_shared(
        owner.runtime.clone(),
        owner.registry.clone(),
        owner.sessions.clone(),
    )
}
pub(crate) async fn builder(owner: &WorkflowRuntime) -> ModelBuilder {
    let runtime = runtime(owner);
    let texts = BTreeMap::from([
        (
            "package.toml".into(),
            std::fs::read_to_string(fixture("package/package.toml")).unwrap(),
        ),
        (
            "materials/physical.yaml".into(),
            std::fs::read_to_string(fixture("package/materials/physical.yaml")).unwrap(),
        ),
    ]);
    let pool = owner.runtime.pool();
    let bundle = pse_runtime::authoring_driver::document::load_package_texts_owned(
        &texts,
        &owner.registry,
        Default::default(),
        &pool,
        &owner.cancel,
    )
    .unwrap();
    let documents = pse_runtime::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
        vec![bundle],
        &pool,
        &owner.cancel,
    )
    .unwrap();
    let physical = runtime
        .physical_from_documents(&documents, &owner.cancel)
        .await
        .unwrap();
    let mut builder = ModelBuilder::from_declaration(
        runtime,
        serde_json::from_value(json("model.json")).unwrap(),
        physical,
    );
    for row in json("balances.json").as_array().unwrap() {
        builder.balance(serde_json::from_value(row.clone()).unwrap());
    }
    for provider in json("providers.json").as_array().unwrap() {
        builder.native_provider(serde_json::from_value(provider.clone()).unwrap());
    }
    let declaration = builder.declaration_mut().clone();
    let balances = json("balances.json");
    for case in &declaration.cases {
        let mut targets = case
            .variables
            .iter()
            .map(|v| (v.port.symbol_id, "variable", 1e-6))
            .chain(case.rows.iter().map(|r| (r.row_id, "row", 1e-4)))
            .collect::<Vec<_>>();
        targets.extend(
            balances
                .as_array()
                .unwrap()
                .iter()
                .filter(|b| sid(&b["case_id"]) == case.case_id)
                .map(|b| (sid(&b["balance_id"]), "row", 1e-4)),
        );
        for (id, kind, tolerance) in targets {
            builder.numerical_requirement(requirement(
                declaration.model_id,
                Some(case.case_id),
                id,
                kind,
                tolerance,
            ));
        }
    }
    builder
}
pub(crate) fn requirement(
    model: SemanticId,
    case: Option<SemanticId>,
    target: SemanticId,
    kind: &str,
    tolerance: f64,
) -> pse_relations::generated::authored::numerical_requirements::Row {
    serde_json::from_value(serde_json::json!({"requirement_id":pse_ids::named_id(case.unwrap_or(model),&format!("acceptance.{kind}.{target}")),"model_id":model,"case_id":case,"target_id":target,"target_kind":kind,"nominal":null,"scaling_factor":null,"absolute_tolerance":tolerance,"relative_tolerance":null,"unit_id":null,"coordinates":"physical","priority":0,"required":true,"provenance":"Plan 14 acceptance budget migrated by semantic identity"})).unwrap()
}
pub(crate) fn compiler() -> pse_compiler::workspace::Profile {
    Default::default()
}
pub(crate) fn profile(backend: Backend, optimize: bool) -> SolverProfile {
    SolverProfile {
        presolve: Default::default(),
        numerics: Default::default(),
        convexity: Default::default(),
        intent: if optimize {
            SolveIntent::Optimize
        } else if backend == Backend::Kinsol {
            SolveIntent::Root
        } else {
            SolveIntent::FeasiblePoint
        },
        selection: SolverSelection::Explicit(backend),
        controls: Controls::default(),
        backend: BackendSettings::Default,
    }
}
pub(crate) async fn solve(
    revision: &ModelRevision,
    case: SemanticId,
    backend: Backend,
    optimize: bool,
) -> Arc<RunResult> {
    let p = revision
        .prepare(
            case,
            profile(backend, optimize),
            compiler(),
            &pse_runtime::CancelSource::new(),
        )
        .await
        .unwrap();
    p.start().unwrap().wait().await.unwrap()
}
/// Independent declared physical closure is a separate assertion from native status/feasibility.
pub(crate) fn conservation(result: &RunResult, minimum: usize) {
    let batch = result.table("runtime.physical_checks").unwrap();
    let rows = pse_relations::generated::runtime::physical_checks::View::from_checked(&batch)
        .unwrap()
        .rows()
        .unwrap();
    assert!(rows.len() >= minimum);
    for row in rows {
        assert_eq!(row.accepted, Some(true), "{row:?}");
        assert!(row.error.is_none(), "{row:?}");
    }
}
pub(crate) fn success(result: &RunResult) {
    conservation(result, 0);
    let RunReport::Solves(report) = result.report().unwrap() else {
        panic!("wrong report")
    };
    assert_eq!(report.unattempted, 0);
    for r in &report.outcomes {
        match r {
            Outcome::Native(r) => {
                assert!(r.quality.as_ref().is_some_and(|q| q.feasible()), "{r:?}");
                assert!(
                    matches!(
                        r.termination.category,
                        pse_backend_native::solve::Termination::Success
                            | pse_backend_native::solve::Termination::Acceptable
                            | pse_backend_native::solve::Termination::FeasibleOnly
                    ),
                    "{r:?}"
                );
            }
            Outcome::Constant(r) => assert!(r.quality.feasible()),
            Outcome::Rejected(e) => panic!("{e}"),
        }
    }
}
pub(crate) fn variable(result: &RunResult, id: SemanticId) -> f64 {
    use pse_relations::generated::runtime::solve_variables as wire;
    let batch = result.table("runtime.solve_variables").unwrap();
    let view = wire::View::from_checked(&batch).unwrap();
    (0..view.len())
        .find_map(|i| {
            let r = view.row(i).unwrap();
            (r.symbol_id == id).then_some(r.value).flatten()
        })
        .unwrap()
}
pub(crate) fn near(a: f64, b: f64, tol: f64) {
    assert!(
        a.is_finite() && (a - b).abs() <= tol,
        "{a} != {b}, tolerance {tol}"
    )
}
pub(crate) fn vessel(builder: &mut ModelBuilder, valve: bool) {
    let f = json("bindings.json");
    let ports = f["vessel_ports"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(k, _)| valve || !matches!(k.as_str(), "valve_k" | "downstream" | "valve_width"))
        .map(|(role, p)| {
            (
                role.clone(),
                pse_kernels::Port {
                    id: sid(&p["symbol_id"]),
                    quantity: pse_quantity::QuantityTypeId::from_id(sid(&p["quantity_id"])),
                    unit: pse_quantity::UnitId::from_id(sid(&p["unit_id"])),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let values = ports
        .iter()
        .map(|(role, p)| (p.id, f["vessel_values"][role].as_f64().unwrap()))
        .collect();
    builder
        .vessel(pse_runtime::workflow::VesselRecipe {
            id: sid(&f["vessel_id"]),
            ports: pse_runtime::workflow::VesselPorts {
                n: ports["n"].clone(),
                u: ports["u"].clone(),
                temperature: ports["temperature"].clone(),
                density: ports["density"].clone(),
                pressure: ports["pressure"].clone(),
                time: ports["time"].clone(),
                volume: ports["volume"].clone(),
                methane: ports["methane"].clone(),
                ethane: ports["ethane"].clone(),
                inflow: ports["inflow"].clone(),
                outflow: ports["outflow"].clone(),
                inlet_enthalpy: ports["inlet_enthalpy"].clone(),
                heat: ports["heat"].clone(),
                n0: ports["n0"].clone(),
                u0: ports["u0"].clone(),
                temperature0: ports["temperature0"].clone(),
                density0: ports["density0"].clone(),
                pressure0: ports["pressure0"].clone(),
                valve: valve.then(|| {
                    (
                        ports["valve_k"].clone(),
                        ports["downstream"].clone(),
                        ports["valve_width"].clone(),
                    )
                }),
            },
            values,
            functions: pse_runtime::workflow::VesselQuantities {
                amount_rate: pse_quantity::QuantityTypeId::from_id(sid(&f["functions"]["rate_n"])),
                energy_rate: pse_quantity::QuantityTypeId::from_id(sid(&f["functions"]["rate_u"])),
                amount: pse_quantity::QuantityTypeId::from_id(sid(&f["functions"]["closure_n"])),
                energy: pse_quantity::QuantityTypeId::from_id(sid(&f["functions"]["closure_u"])),
                pressure: pse_quantity::QuantityTypeId::from_id(sid(&f["functions"]["closure_p"])),
            },
            balance_tolerances: [1e-6, 1e-3],
            state_scales: serde_json::from_value(f["state_scales"].clone()).unwrap(),
            residual_scales: serde_json::from_value(f["residual_scales"].clone()).unwrap(),
            provider: serde_json::from_value(f["provider"].clone()).unwrap(),
        })
        .unwrap();
}
pub(crate) fn simulation() -> pse_runtime::workflow::SimulationProfile {
    pse_runtime::workflow::SimulationProfile {
        samples: vec![0., 0.25, 0.5, 0.75, 1.],
        atol: vec![1e-8; 5],
        rtol: 1e-7,
        out_rtol: Some(1e-8),
        out_atol: vec![1e-8, 1e-5],
        parameter_scales: vec![1.; 4],
        ..Default::default()
    }
}
/// Replicate complete heat/property/recycle blocks with shared definitions and fresh identities.
/// Small/medium/large shapes exercise 3/24/96 variables without a parallel math evaluator.
pub(crate) fn resize(builder: &mut ModelBuilder, blocks: usize) {
    let f = json("bindings.json");
    let id = sid(&f["root_case"]);
    let balances: Vec<_> = builder
        .sources_mut()
        .balances
        .iter()
        .filter(|b| b.case_id == id)
        .cloned()
        .collect();
    let row = builder.declaration_mut();
    let original = row.cases.iter().find(|c| c.case_id == id).unwrap().clone();
    let case = row.cases.iter_mut().find(|c| c.case_id == id).unwrap();
    for block in 1..blocks {
        let remap =
            |value: SemanticId| pse_ids::named_id(value, &format!("acceptance-block-{block}"));
        let mut copy = original.clone();
        for v in &mut copy.variables {
            v.port.symbol_id = remap(v.port.symbol_id);
        }
        for p in &mut copy.parameters {
            p.symbol_id = remap(p.symbol_id);
        }
        for v in &mut copy.values {
            v.symbol_id = remap(v.symbol_id);
        }
        for r in &mut copy.rows {
            r.row_id = remap(r.row_id);
        }
        for i in &mut copy.instances {
            i.instance_id = remap(i.instance_id);
            for s in &mut i.slots {
                s.source_id = remap(s.source_id);
            }
            for c in &mut i.contributions {
                c.row_id = c.row_id.map(remap);
            }
        }
        case.variables.extend(copy.variables);
        case.parameters.extend(copy.parameters);
        case.values.extend(copy.values);
        case.rows.extend(copy.rows);
        case.instances.extend(copy.instances);
    }
    for block in 1..blocks {
        let remap =
            |value: SemanticId| pse_ids::named_id(value, &format!("acceptance-block-{block}"));
        for balance in &balances {
            let mut copy = balance.clone();
            copy.balance_id = remap(copy.balance_id);
            copy.accumulation = copy.accumulation.map(remap);
            for term in &mut copy.terms {
                term.source_id = remap(term.source_id);
                term.instance_id = remap(term.instance_id);
                term.transfer_id = term.transfer_id.map(remap);
            }
            builder.balance(copy);
        }
    }
}

/// Shared analytic known-heat fit declaration; test and timing paths use one source recipe.
pub(crate) fn heat_fit(
    b: &mut ModelBuilder,
    kind: &str,
) -> (SemanticId, pse_runtime::workflow::FitProfile) {
    let f = json("bindings.json");
    let u = f["expected"]["u0"].as_f64().unwrap();
    let dynamic = sid(&f["vessel_id"]);
    let heat = sid(&f["vessel_ports"]["heat"]["symbol_id"]);
    let fit_id = pse_ids::named_id(dynamic, kind);
    let dataset = pse_ids::named_id(fit_id, "data");
    b.dataset(serde_json::from_value(serde_json::json!({"dataset_id":dataset,"name":kind,"source":"analytic closed-vessel energy conservation","content_hash":format!("blake3:{}","02".repeat(32))})).unwrap());
    let dynamic_case = pse_ids::named_id(dynamic, "case");
    let mut steady = b
        .declaration_mut()
        .cases
        .iter()
        .find(|c| c.case_id == dynamic_case)
        .unwrap()
        .clone();
    steady.case_id = pse_ids::named_id(fit_id, "steady");
    for v in &mut steady.variables {
        v.fixed = true;
    }
    let steady_id = steady.case_id;
    let heat_row = pse_ids::named_id(steady_id, "measured-heat");
    steady.rows.push(pse_relations::generated::authored::computation_models::AuthoredComputationModelsFieldCasesItemRowsItem {
        row_id:heat_row,quantity_id:sid(&f["functions"]["rate_u"]),lower:None,upper:None,
    });
    steady.instances.iter_mut().find(|i|i.instance_id == pse_ids::named_id(dynamic,"instance.heat")).unwrap().contributions.push(
        pse_relations::generated::authored::computation_models::AuthoredComputationModelsFieldCasesItemInstancesItemContributionsItem {output:0,row_id:Some(heat_row),scale:1.0}
    );
    steady
        .instances
        .retain(|instance| !instance.contributions.is_empty());
    b.declaration_mut().cases.push(steady);
    let mut experiments = vec![];
    let mut observations = vec![];
    for transient in [false, true] {
        if (kind == "steady" && transient) || (kind == "transient" && !transient) {
            continue;
        }
        let experiment = pse_ids::named_id(fit_id, if transient { "dynamic" } else { "static" });
        let observation = pse_ids::named_id(experiment, "observation");
        experiments.push(serde_json::json!({"experiment_id":experiment,"case_id":if transient{dynamic_case}else{steady_id},"dynamic_id":if transient{Some(dynamic)}else{None::<SemanticId>}}));
        let unit = sid(&f["ids"]["units"][if transient { "energy" } else { "power" }]);
        b.observation(serde_json::from_value(serde_json::json!({"observation_id":observation,"dataset_id":dataset,"target":"energy balance","value":if transient{u+10.}else{10.},"unit_id":unit,"std_dev":1.,"timestamp":null,"tag":null,"source_span":{"document_id":dataset,"start":0,"end":0}})).unwrap());
        observations.push(serde_json::json!({"observation_id":observation,"experiment_id":experiment,"output_id":if transient{pse_ids::named_id(dynamic,"row.output.u")}else{heat_row},"time":if transient{Some(1.)}else{None::<f64>},"included":true,"importance":1.}));
    }
    let model_id = b.declaration_mut().model_id;
    b.fit(serde_json::from_value(serde_json::json!({"fit_id":fit_id,"model_id":model_id,"parameters":[{"symbol_id":heat,"fixed":false,"value":3.,"lower":0.,"upper":20.,"scale":10.}],"experiments":experiments,"observations":observations})).unwrap());
    let mut solver = profile(Backend::Ipopt, true);
    solver
        .numerics
        .requirements
        .push(requirement(model_id, None, heat, "variable", 1e-6));
    solver.controls.hessian = pse_backend_native::solve::HessianMode::LimitedMemory;
    let mut simulation = simulation();
    simulation.sensitivities = true;
    let profile = pse_runtime::workflow::FitProfile {
        solver,
        simulations: if kind == "steady" {
            BTreeMap::new()
        } else {
            BTreeMap::from([(pse_ids::named_id(fit_id, "dynamic"), simulation)])
        },
        rank_tolerance: 1e-8,
        max_cells: 1 << 20,
    };
    (fit_id, profile)
}
