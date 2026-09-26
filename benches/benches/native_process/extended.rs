// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Additional case shapes use public preparation/execution, with analytic controls.
use super::*;
use pse_backend_native::{self as native, solve::Controls};
use pse_ids::{SemanticId, named_id};
use pse_runtime::workflow::{self, ModelBuilder};
use serde_json::{Value, json};

fn id(n: u32) -> SemanticId {
    named_id(SemanticId::NIL, &format!("process-cost.{n}"))
}
fn neutral() -> (SemanticId, SemanticId) {
    let f = json("bindings.json");
    (
        sid(&f["ids"]["quantities"]["neutral"]),
        sid(&f["ids"]["units"]["neutral"]),
    )
}
fn port(symbol: SemanticId) -> Value {
    let (q, u) = neutral();
    json!({"symbol_id":symbol,"quantity_id":q,"unit_id":u})
}
async fn algebraic(owner: &WorkflowRuntime, quadratic: bool, mixed: bool) -> ModelBuilder {
    let (q, u) = neutral();
    let targets = if mixed { [1e-6, 1e6] } else { [2., 3.] };
    let definitions = (0..2)
        .map(|i| {
            json!({"definition_id":id(10+i),
        "sources":[if quadratic {"(x-a)*(x-a)"} else {"x-a"}],
        "formals":[{"path":"x","quantity_id":q},{"path":"a","quantity_id":q}],
        "domains":[],"groups":[],"providers":[],"units":[],"literals":[]})
        })
        .collect::<Vec<_>>();
    let row = serde_json::from_value(json!({"model_id":id(1),"name":"analytic two-coordinate case",
        "definitions":definitions,"domains":[],"groups":[],"cases":[{"case_id":id(2),"name":"selected",
        "variables":(0..2).map(|i|json!({"port":port(id(20+i)),"fixed":false,"domain":"continuous","lower":null,"upper":null})).collect::<Vec<_>>(),
        "parameters":[port(id(30)),port(id(31))],
        "instances":(0..2).map(|i|json!({"instance_id":id(40+i),"definition_id":id(10+i),
            "slots":[{"source_id":id(20+i),"formal_quantity_id":q,"formal_unit_id":u},{"source_id":id(30+i),"formal_quantity_id":q,"formal_unit_id":u}],
            "contributions":[{"output":0,"row_id":if quadratic {None} else {Some(id(50+i))},"scale":1.}]})).collect::<Vec<_>>(),
        "rows":if quadratic {vec![]} else {(0..2).map(|i|json!({"row_id":id(50+i),"quantity_id":q,"lower":0.,"upper":0.})).collect::<Vec<_>>()},
        "objective":if quadratic {json!({"quantity_id":q,"sense":"minimize"})} else {Value::Null},
        "values":[{"symbol_id":id(20),"value":targets[0]*0.9},{"symbol_id":id(21),"value":targets[1]*0.9},
            {"symbol_id":id(30),"value":targets[0]},{"symbol_id":id(31),"value":targets[1]}]}]})).unwrap();
    let mut builder = ModelBuilder::from_declaration(runtime(owner), row, physical(owner).await);
    for (i, target) in targets.into_iter().enumerate() {
        for (symbol, kind) in [(id(20 + i as u32), "variable"), (id(50 + i as u32), "row")] {
            if quadratic && kind == "row" {
                continue;
            }
            let mut r = requirement(id(1), Some(id(2)), symbol, kind, target * 1e-8);
            r.nominal = Some(target);
            r.scaling_factor = Some(1. / target);
            builder.numerical_requirement(r);
        }
    }
    builder
}

async fn cone(owner: &WorkflowRuntime) {
    let request = workflow::ConicRequest {
        variables: vec![serde_json::from_value(port(id(20))).unwrap()],
        rows: vec![serde_json::from_value(port(id(50))).unwrap()],
        objective_port: serde_json::from_value(port(SemanticId::NIL)).unwrap(),
        quadratic: native::conic::Matrix::zeros((1, 1)),
        objective: vec![1.],
        constraints: native::conic::Matrix::new(1, 1, vec![0, 1], vec![0], vec![-1.]),
        rhs: vec![-2.],
        cones: vec![native::conic::Cone::NonnegativeConeT(1)],
        objective_constant: 3.,
        gram_factors: vec![],
        gram_weights: vec![],
    };
    let prepared = runtime(owner)
        .prepare_conic(
            request,
            &physical(owner).await,
            profile(Backend::Clarabel, true),
        )
        .await
        .unwrap();
    let result = prepared.start().unwrap().finish().await.unwrap();
    let pse_runtime::math::solves::Outcome::Native(report) = &result.outcomes[0] else {
        panic!("missing cone result")
    };
    near(report.candidate.as_ref().unwrap().primal[0], 2., 1e-6);
    near(
        report.candidate.as_ref().unwrap().objective.unwrap(),
        5.,
        1e-6,
    );
    assert!(report.quality.as_ref().unwrap().feasible());
}

async fn recycle(owner: &WorkflowRuntime) {
    let mut builder = algebraic(owner, false, false).await;
    *builder.declaration_mut() = serde_json::from_value(json!({"model_id":id(1),"name":"declared recycle",
        "definitions":[],"domains":[],"groups":[],"cases":[{"case_id":id(2),"name":"selected","variables":[],"parameters":[],"instances":[],"rows":[],"objective":null,"values":[]}]})).unwrap();
    *builder.sources_mut() = Default::default();
    let (q, _) = neutral();
    let input = workflow::port_id(id(3), "in");
    let output = workflow::port_id(id(3), "out");
    builder.sources_mut().composition = serde_json::from_value(json!({
        "roots":[{"model_id":id(1),"root_instance_id":id(3)}],
        "templates":[{"template_id":id(4),"package_id":id(5),"name":"unit","version":"1","kind":"unit","default_initializer_template_id":null,"default_scaler_template_id":null,"idaes_class":null,"doc":""}],
        "instances":[{"instance_id":id(3),"parent_instance_id":null,"template_id":id(4),"name":"unit","param_values":[],"feature_values":[],"property_package_id":null,"reaction_package_id":null,"doc":""}],
        "symbols":[{"template_id":id(4),"symbol_decl_id":id(6),"name":"x","role":"variable","quantity_type_id":q,"indexed_by":[],"default_lower":null,"default_upper":null,"default_initial":1.,"reference_to":null,"wrt_domain":null,"guard_id":null,"idaes_name":null,"doc":""},
            {"template_id":id(4),"symbol_decl_id":id(7),"name":"a","role":"parameter","quantity_type_id":q,"indexed_by":[],"default_lower":null,"default_upper":null,"default_initial":2.,"reference_to":null,"wrt_domain":null,"guard_id":null,"idaes_name":null,"doc":""}],
        "equations":[{"template_id":id(4),"equation_decl_id":id(8),"name":"map","indexed_by":[],"filter":null,"expression":"x/2+a == 0","sense":"eq","guard_id":null,"idaes_name":null,"doc":""}],
        "ports":[{"template_id":id(4),"name":"out","kind":"material","direction":"outlet","bound_to":"x","guard_id":null,"doc":""},{"template_id":id(4),"name":"in","kind":"material","direction":"inlet","bound_to":"x","guard_id":null,"doc":""}],
        "connection_rules":[{"rule_template_id":id(60),"expansion":"equality"}],
        "connections":[{"connection_id":id(61),"from_port_id":output,"to_port_id":input,"rule_template_id":id(60),"tear_cost":2.,"tear_policy":"mandatory","tear_group":null,"doc":""}]})).unwrap();
    let revision = builder.freeze().unwrap();
    let flow = revision.prepare_flow(id(2), id(3)).await.unwrap();
    let selected = owner
        .runtime
        .math()
        .select_tears(
            flow,
            pse_runtime::math::flows::TearMethod::UnweightedHeuristic,
            Controls::default(),
        )
        .unwrap()
        .finish()
        .await
        .unwrap()
        .selected
        .clone()
        .unwrap();
    let x = workflow::symbol_id(id(3), id(6), &[]);
    let request = workflow::RecycleRequest {
        case: id(2),
        flow: id(3),
        tears: selected.decisions,
        units: vec![workflow::CausalUnitRequest {
            node: id(3),
            case: id(2),
            inputs: BTreeMap::from([(workflow::symbol_id(input, x, &[]), x)]),
            outputs: BTreeMap::from([(
                workflow::symbol_id(output, x, &[]),
                workflow::symbol_id(id(3), id(8), &[]),
            )]),
        }],
        anderson: 1,
        damping: 1.,
    };
    let prepared = revision
        .prepare_recycle(
            request,
            profile(Backend::Kinsol, false),
            compiler(),
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let result = prepared.start().unwrap().finish().await.unwrap();
    near(
        result.report.candidate.as_ref().unwrap().primal[0],
        4.,
        1e-6,
    );
    assert!(result.report.quality.as_ref().unwrap().feasible());
}

async fn sparse_fit(owner: &WorkflowRuntime, n: usize) {
    let (q, u) = neutral();
    let row=serde_json::from_value(json!({"model_id":id(1),"name":"independent observations","domains":[],"groups":[],
        "definitions":[{"definition_id":id(10),"sources":["a"],"formals":[{"path":"a","quantity_id":q}],"domains":[],"groups":[],"providers":[],"units":[],"literals":[]}],
        "cases":[{"case_id":id(2),"name":"responses","variables":[],
            "parameters":(0..n).map(|i|port(id(1000+i as u32))).collect::<Vec<_>>(),
            "instances":(0..n).map(|i|json!({"instance_id":id(2000+i as u32),"definition_id":id(10),"slots":[{"source_id":id(1000+i as u32),"formal_quantity_id":q,"formal_unit_id":u}],"contributions":[{"output":0,"row_id":id(3000+i as u32),"scale":1.}]})).collect::<Vec<_>>(),
            "rows":(0..n).map(|i|json!({"row_id":id(3000+i as u32),"quantity_id":q,"lower":null,"upper":null})).collect::<Vec<_>>(),
            "objective":null,"values":(0..n).map(|i|json!({"symbol_id":id(1000+i as u32),"value":1.})).collect::<Vec<_>>()}]})).unwrap();
    let mut b = ModelBuilder::from_declaration(runtime(owner), row, physical(owner).await);
    b.dataset(serde_json::from_value(json!({"dataset_id":id(70),"name":"independent","source":"analytic identity responses","content_hash":pse_ids::ContentHash::from_bytes([1;32])})).unwrap());
    for i in 0..n {
        b.observation(serde_json::from_value(json!({"observation_id":id(4000+i as u32),"dataset_id":id(70),"target":"identity","value":2.,"unit_id":u,"std_dev":1.,"timestamp":null,"tag":null,"source_span":{"document_id":id(70),"start":0,"end":0}})).unwrap());
    }
    b.fit(serde_json::from_value(json!({"fit_id":id(73),"model_id":id(1),
        "parameters":(0..n).map(|i|json!({"symbol_id":id(1000+i as u32),"fixed":false,"value":1.,"lower":null,"upper":null,"scale":1.})).collect::<Vec<_>>(),
        "experiments":[{"experiment_id":id(74),"case_id":id(2),"dynamic_id":null}],
        "observations":(0..n).map(|i|json!({"observation_id":id(4000+i as u32),"experiment_id":id(74),"output_id":id(3000+i as u32),"time":null,"included":true,"importance":1.})).collect::<Vec<_>>()})).unwrap());
    let mut solver = profile(Backend::Ipopt, true);
    solver.presolve = native::presolve::Policy::Off;
    let prepared = b
        .freeze()
        .unwrap()
        .prepare_fit(
            id(73),
            workflow::FitProfile {
                solver,
                simulations: BTreeMap::new(),
                rank_tolerance: 1e-8,
                max_cells: n * 8,
            },
            compiler(),
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let result = prepared.start().unwrap().wait().await.unwrap();
    let RunReport::Fit(report) = result.report().unwrap() else {
        panic!("missing sparse fit")
    };
    assert!(report.quality.as_ref().unwrap().feasible());
    for value in report.candidate.as_ref().unwrap() {
        near(*value, 2., 1e-6);
    }
    assert!(
        report.responses.is_none(),
        "dense rank exceeds its separately declared cell cap"
    );
}

async fn run(owner: &WorkflowRuntime, operation: &str, size: usize) {
    match operation {
        "conic" => cone(owner).await,
        "recycle" => recycle(owner).await,
        "sparse-fit" => sparse_fit(owner, size).await,
        "mixed-scale" | "qp" | "value-sweep" => {
            let quadratic = operation == "qp";
            let mut revision = algebraic(owner, quadratic, operation == "mixed-scale")
                .await
                .freeze()
                .unwrap();
            let count = if operation == "value-sweep" { 1000 } else { 1 };
            for point in 0..count {
                if point > 0 {
                    let mut edit = revision.edit();
                    edit.declaration_mut().cases[0]
                        .values
                        .iter_mut()
                        .find(|v| v.symbol_id == id(30))
                        .unwrap()
                        .value = 2. + point as f64 / 1000.;
                    revision = edit.freeze().unwrap();
                }
                let mut selected = profile(
                    if quadratic {
                        Backend::Highs
                    } else {
                        Backend::Kinsol
                    },
                    quadratic,
                );
                selected.presolve = native::presolve::Policy::Off;
                let prepared = revision
                    .prepare(id(2), selected, compiler(), &CancelSource::new())
                    .await
                    .unwrap();
                let result = prepared.start().unwrap().wait().await.unwrap();
                let RunReport::Solves(reports) = result.report().unwrap() else {
                    panic!("missing algebraic result")
                };
                let pse_runtime::math::solves::Outcome::Native(report) = &reports.outcomes[0]
                else {
                    panic!("native solve required")
                };
                assert!(report.quality.as_ref().unwrap().feasible());
                let case = &revision.declaration().cases[0];
                for i in 0..2 {
                    let expected = case
                        .values
                        .iter()
                        .find(|v| v.symbol_id == id(30 + i))
                        .unwrap()
                        .value;
                    near(
                        variable(&result, id(20 + i)),
                        expected,
                        expected.abs() * 1e-7,
                    );
                }
            }
        }
        "dynamic-rebind" | "evented-fit" => {
            let mut b = builder(owner).await;
            vessel(&mut b, false);
            let f = json("bindings.json");
            let dynamic = sid(&f["vessel_id"]);
            if operation == "dynamic-rebind" {
                let revision = b.freeze().unwrap();
                let prepared = revision
                    .prepare_simulation(dynamic, simulation(), compiler(), &CancelSource::new())
                    .await
                    .unwrap();
                for step in 1..=4 {
                    let mut selected = simulation();
                    selected.end = step as f64 * 0.25;
                    selected.samples = vec![0., selected.end];
                    let rebound = prepared
                        .rebind(
                            &BTreeMap::from([(
                                sid(&f["vessel_ports"]["heat"]["symbol_id"]),
                                step as f64,
                            )]),
                            selected,
                            compiler(),
                            &CancelSource::new(),
                        )
                        .await
                        .unwrap();
                    let result = rebound.start().unwrap().wait().await.unwrap();
                    let RunReport::Simulation(report) = result.report().unwrap() else {
                        panic!("missing trajectory")
                    };
                    assert_eq!(report.termination, native::dynamics::Termination::Completed);
                    conservation(&result, report.samples.len() * 2);
                }
            } else {
                let (fit, mut selected) = heat_fit(&mut b, "transient");
                // A scheduled change leaves the first half of the heat parameter
                // active. U(1)=U(0)+0.5*q+0.5*10 has its unique fit at q=10.
                let dynamic_row = b
                    .sources_mut()
                    .dynamics
                    .iter()
                    .find(|d| d.dynamic_id == dynamic)
                    .unwrap()
                    .clone();
                let heat = sid(&f["vessel_ports"]["heat"]["symbol_id"]);
                let case = b
                    .declaration_mut()
                    .cases
                    .iter()
                    .find(|c| c.case_id == dynamic_row.case_id)
                    .unwrap();
                let parameters = dynamic_row
                    .parameters
                    .iter()
                    .map(|s| {
                        if *s == heat {
                            10.
                        } else {
                            case.values
                                .iter()
                                .find(|v| v.symbol_id == *s)
                                .unwrap()
                                .value
                        }
                    })
                    .collect();
                selected
                    .simulations
                    .values_mut()
                    .next()
                    .unwrap()
                    .changes
                    .push(native::dynamics::InputChange {
                        time: 0.5,
                        parameters,
                    });
                let prepared = b
                    .freeze()
                    .unwrap()
                    .prepare_fit(fit, selected, compiler(), &CancelSource::new())
                    .await
                    .unwrap();
                let result = prepared.start().unwrap().wait().await.unwrap();
                let RunReport::Fit(report) = result.report().unwrap() else {
                    panic!("missing event fit")
                };
                near(report.candidate.as_ref().unwrap()[0], 10., 2e-3);
                assert!(report.estimate_qualified(), "{report:?}");
            }
        }
        _ => panic!("unknown extended workload {operation}"),
    }
}

pub(super) fn measure(
    c: &mut Criterion,
    spec: &Value,
    output: &std::path::Path,
    compiler_phases: &phases::Phases,
) {
    let name = spec["id"].as_str().unwrap();
    let operation = spec["operation"].as_str().unwrap();
    let size = spec["blocks"].as_u64().unwrap() as usize;
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let mut peak = 0;
    let mut rss = 0;
    let mut retained = 0;
    let mut after_teardown = 0;
    let mut iterations = 0;
    compiler_phases.reset();
    let mut group = c.benchmark_group("process");
    group
        .sample_size(10)
        .sampling_mode(criterion::SamplingMode::Flat)
        .warm_up_time(Duration::from_millis(250))
        .measurement_time(Duration::from_secs(1));
    group.bench_function(name, |b| {
        b.iter(|| {
            let owner = WorkflowRuntime::with_threads(NonZeroUsize::new(1).unwrap()).unwrap();
            executor.block_on(run(&owner, operation, size));
            peak = peak.max(owner.runtime.observation_peak_bytes());
            rss = rss.max(
                owner
                    .runtime
                    .report()
                    .unwrap()
                    .process_peak_rss_bytes
                    .unwrap(),
            );
            let pool = owner.runtime.pool();
            retained = retained.max(pool.reserved());
            drop(owner);
            executor.block_on(async {
                tokio::task::yield_now().await;
            });
            after_teardown = after_teardown.max(pool.reserved());
            iterations += 1;
        })
    });
    group.finish();
    std::fs::write(output.join(format!("{name}-memory.json")),serde_json::to_vec_pretty(&json!({
        "id":name,"iterations":iterations,"pool_peak_bytes":peak,"process_peak_rss_bytes":rss,
        "retained_runtime_bytes":retained,"after_case_teardown_bytes":after_teardown,
        "after_retained_runtime_teardown_bytes":null,"workload":spec,
        "threads":1,"native_threads":1,"compiler_phases":compiler_phases.report(iterations),
        "effective_process_parallelism":std::thread::available_parallelism().unwrap().get(),
        "scope":"fresh runtime per iteration; public source/preparation/execution/analytic validation and teardown; value sweep retains one compiler for 1000 revisions; dynamic rebind retains initial preparation",
        "start_policy":"NoPriorStart","sampling":"10 flat Criterion samples; warmup 250ms; target 1s extended for slow cases",
        "memory_scope":"finite-pool reservations and independent process VmHWM; retained runtime before owner teardown reported separately"
    })).unwrap()).unwrap();
}
