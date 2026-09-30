// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Additional case shapes use public preparation/execution, with analytic controls.
use super::*;
use pse_backend_native::{self as native, solve::Controls};
use pse_ids::{SemanticId, named_id};
use pse_model::generated::identities::{DeclarationId, FitId};
use pse_runtime::workflow;
use serde_json::{Value, json};

fn id(n: u32) -> SemanticId {
    named_id(SemanticId::NIL, &format!("process-cost.{n}"))
}
fn neutral(physical: &workflow::PhysicalContext) -> (SemanticId, SemanticId) {
    let q = physical.quantities().neutral_dimensionless().unwrap();
    (
        q.as_id(),
        physical
            .quantities()
            .quantity_type(q)
            .unwrap()
            .canonical_unit
            .as_id(),
    )
}
fn port(symbol: SemanticId, physical: &workflow::PhysicalContext) -> Value {
    let (q, u) = neutral(physical);
    json!({"symbol_id":symbol,"quantity_id":q,"unit_id":u})
}
async fn algebraic(
    owner: &WorkflowRuntime,
    quadratic: bool,
    mixed: bool,
) -> (workflow::ModelingPackage, DeclarationId) {
    let physical = physical(owner).await;
    let (q, _) = neutral(&physical);
    let (a, b) = if mixed { (1e-6, 1e6) } else { (2.0, 3.0) };
    let equations = if quadratic {
        "let cost:Scalar=(x-a)*(x-a)+(y-b)*(y-b); annotation objective cost(minimize);"
    } else {
        "eq first:x==a; eq second:y==b;"
    };
    let source = format!(
        "package benchmark {{def Root {{param a:Scalar={a}; param b:Scalar={b}; var x:Scalar; var y:Scalar; annotation start x(a*0.9); annotation start y(b*0.9); annotation nominal x(a); annotation nominal y(b); {equations} annotation check x(abs(x-a)<=a*1e-7); annotation check y(abs(y-b)<=b*1e-7);}} }}"
    );
    let rows = pse_authoring::language::parse(
        &source,
        id(1),
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    (
        runtime(owner).modeling_package(rows, physical).unwrap(),
        root,
    )
}

async fn cone(owner: &WorkflowRuntime) {
    let physical = physical(owner).await;
    let request = workflow::ConicRequest {
        variables: vec![serde_json::from_value(port(id(20), &physical)).unwrap()],
        rows: vec![serde_json::from_value(port(id(50), &physical)).unwrap()],
        objective_port: serde_json::from_value(port(SemanticId::NIL, &physical)).unwrap(),
        quadratic: native::conic::SparseMatrix::zeros(1, 1),
        objective: vec![1.],
        constraints: native::conic::SparseMatrix::new(1, 1, vec![0, 1], vec![0], vec![-1.]),
        rhs: vec![-2.],
        cones: vec![native::conic::Cone::Nonnegative { dimension: 1 }],
        objective_constant: 3.,
    };
    let prepared = runtime(owner)
        .prepare_conic(request, &physical, profile(Backend::Clarabel, true))
        .await
        .unwrap();
    let result = prepared.start().unwrap().finish().await.unwrap();
    let pse_runtime::math::solves::Outcome::Native(report) = &result.outcome else {
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
    let physical = physical(owner).await;
    let (q, _) = neutral(&physical);
    let rows=pse_authoring::language::parse("package benchmark {def Root {param a:Scalar=2; var x:Scalar; let result:Scalar=x/2+a; port inlet:Scalar=x; port outlet:Scalar=result; connect outlet -> inlet; annotation start x(1);}}",id(1),pse_authoring::language::IdentityPolicy::Named,Default::default()).unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime(owner).modeling_package(rows, physical).unwrap();
    let cancel = CancelSource::new();
    let analysis = package
        .declared_analysis(
            root,
            pse_relations::generated::enums::ModelingAnalysisRoute::Steady,
            compiler(),
            profile(Backend::Kinsol, false),
            Default::default(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    let model = package
        .prepare(
            root,
            pse_modeling::specialize::root_instance(root),
            analysis.bindings.clone(),
            analysis.limits,
            &cancel,
        )
        .await
        .unwrap();
    let product = &model.compiled().model;
    let port = |name: &str| {
        product
            .ports
            .values()
            .find(|p| p.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let (input, output) = (port("inlet"), port("outlet"));
    let selection = pse_compiler::workspace::ModelingFlowSelection {
        nodes: std::collections::BTreeSet::from([pse_modeling::specialize::root_instance(root)]),
        connections: product
            .connections
            .keys()
            .map(|id| {
                (
                    *id,
                    pse_runtime::math::flows::Decision {
                        id: *id,
                        cost: 2.0,
                        policy: pse_runtime::math::flows::Policy::Mandatory,
                    },
                )
            })
            .collect(),
    };
    let flow = package
        .prepare_flow(&analysis, selection.clone(), &cancel)
        .await
        .unwrap();
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
    let request = workflow::RecycleRequest {
        tears: selected.decisions,
        units: vec![workflow::CausalUnitRequest {
            node: pse_modeling::specialize::root_instance(root).into(),
            inputs: std::collections::BTreeSet::from([input]),
            outputs: std::collections::BTreeSet::from([output]),
        }],
        anderson: 1,
        damping: pse_model::scalars::Fraction::try_new(1.0).unwrap(),
    };
    let prepared = package
        .prepare_recycle(&analysis, selection, request, &cancel)
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
    let physical = physical(owner).await;
    let source = format!(
        "package benchmark {{ entity kind source provenance {{}} entity source analytic {{}} enum Role {{ measured facets(measured) }} entity kind measurement {{attribute observed:Scalar;}} {} def Identity {{ {} }} }}",
        (0..n).map(|i|format!("@id(\"{}\") entity measurement reading{i} provenance(analytic,Role.measured) {{observed=2.0 ± standard(1.0)}}",id(4000+i as u32))).collect::<String>(),
        (0..n)
            .map(|i| format!("param p{i}:Scalar=1; "))
            .collect::<String>()
    );
    let rows = pse_authoring::language::parse(
        &source,
        id(1),
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Identity")
        .unwrap()
        .declaration_id;
    let mut data = workflow::FitDeclarations::default();
    data.fits.push(serde_json::from_value(json!({"fit_id":id(73),
        "parameters":(0..n).map(|i|json!({"symbol_id":id(1000+i as u32),"fixed":false,"value":1.,"lower":null,"upper":null,"scale":1.})).collect::<Vec<_>>(),
        "experiments":[{"experiment_id":id(74),"case_id":root,"route":"steady","bindings":(0..n).map(|i|json!({"parameter_id":id(1000+i as u32),"path":format!("p{i}")})).collect::<Vec<_>>()}],
        "observations":(0..n).map(|i|json!({"observation_id":id(4000+i as u32),"experiment_id":id(74),"value_attribute":"observed","standard_deviation_attribute":null,"output_path":format!("p{i}"),"time":null,"included":true,"importance":1.})).collect::<Vec<_>>()})).unwrap());
    let package = runtime(owner)
        .modeling_package(rows, physical)
        .unwrap()
        .with_fit_declarations(data)
        .unwrap();
    let mut solver = profile(Backend::Ipopt, true);
    solver.presolve = native::presolve::Policy::Off;
    let prepared = package
        .prepare_fit(
            FitId::from_id(id(73)),
            workflow::FitProfile {
                solver,
                simulations: BTreeMap::new(),
                rank_tolerance: 1e-8,
                max_cells: n * 8,
                derivatives: workflow::FitDerivatives::Responses,
                uncertainty: None,
            },
            compiler(),
            seed_limits(),
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
            let mixed = operation == "mixed-scale";
            let (package, root) = algebraic(owner, quadratic, mixed).await;
            let cancel = CancelSource::new();
            let mut selected = profile(
                if quadratic {
                    Backend::Highs
                } else {
                    Backend::Kinsol
                },
                quadratic,
            );
            selected.presolve = native::presolve::Policy::Off;
            let mut analysis = package
                .declared_analysis(
                    root,
                    pse_relations::generated::enums::ModelingAnalysisRoute::Steady,
                    compiler(),
                    selected,
                    Default::default(),
                    seed_limits(),
                    &cancel,
                )
                .await
                .unwrap();
            let count = if operation == "value-sweep" { 1000 } else { 1 };
            for point in 0..count {
                let expected = [
                    if mixed {
                        1e-6
                    } else {
                        2.0 + point as f64 / 1000.0
                    },
                    if mixed { 1e6 } else { 3.0 },
                ];
                analysis.case.values.insert("a".into(), expected[0]);
                let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
                let symbol = |name: &str| {
                    prepared
                        .model
                        .model
                        .compiled()
                        .model
                        .symbols
                        .values()
                        .find(|s| s.lineage.path.ends_with(&format!(".{name}")))
                        .unwrap()
                        .id
                };
                let coordinates = [symbol("x"), symbol("y")];
                let result = prepared.start().unwrap().wait().await.unwrap();
                authored_success(&result);
                for (id, expected) in coordinates.into_iter().zip(expected) {
                    near(variable(&result, id), expected, expected.abs() * 1e-7);
                }
            }
        }

        "vessel" | "dynamic-rebind" => {
            let package = seed_package(owner).await;
            let root = DeclarationId::from_id(
                SemanticId::parse_hex("29dd6a1a3e444acfbf14992087f9d32c").unwrap(),
            );
            let cancel = CancelSource::new();
            let prepared = package
                .declared_simulation(root, compiler(), None, seed_limits(), &cancel)
                .await
                .unwrap();
            if operation == "vessel" {
                let result = prepared.run(&cancel).await.unwrap();
                assert!(result.accepted, "{result:?}");
                std::hint::black_box(result.tables().unwrap());
            } else {
                let analysis = package
                    .declared_analysis(
                        root,
                        pse_relations::generated::enums::ModelingAnalysisRoute::Integrated,
                        compiler(),
                        profile(Backend::Ipopt, false),
                        Default::default(),
                        seed_limits(),
                        &cancel,
                    )
                    .await
                    .unwrap();
                for step in 1..=4 {
                    let mut case = analysis.case.clone();
                    case.values.insert("root.heat".into(), step as f64);
                    let rebound = package
                        .prepare_simulation(
                            root,
                            pse_modeling::specialize::root_instance(root),
                            analysis.bindings.clone(),
                            seed_limits(),
                            case,
                            compiler(),
                            prepared.profile().clone(),
                            pse_kernels::DerivativeOrder::First,
                            &cancel,
                        )
                        .await
                        .unwrap();
                    let result = rebound.run(&cancel).await.unwrap();
                    assert!(result.accepted, "{result:?}");
                }
            }
        }
        "fit" | "evented-fit" => {
            let package = seed_package(owner).await;
            let (fit, mut selected) = heat_fit(&package, "transient").await;
            if operation == "evented-fit" {
                let root = DeclarationId::from_id(
                    SemanticId::parse_hex("29dd6a1a3e444acfbf14992087f9d32c").unwrap(),
                );
                let simulation = package
                    .declared_simulation(
                        root,
                        compiler(),
                        None,
                        seed_limits(),
                        &CancelSource::new(),
                    )
                    .await
                    .unwrap();
                let mut integration = simulation.profile().clone();
                integration.method = native::dynamics::Method::Diffsol;
                integration.rtol = 1e-6;
                integration.atol.fill(1e-8);
                // Every parameter is a scheduled input changing at 0.5 to its model value:
                // one transition, with the fitted value in effect before it.
                integration.schedule = (0..simulation.contract().parameters.len())
                    .map(|parameter| native::dynamics::ScheduledInput {
                        parameter,
                        times: vec![0.5],
                    })
                    .collect();
                let experiment = SemanticId::parse_hex("b39f24e05b7d5490904f6138b4d7e080").unwrap();
                selected.simulations.insert(experiment.into(), integration);
            }
            let result = package
                .prepare_fit(
                    fit,
                    selected,
                    compiler(),
                    seed_limits(),
                    &CancelSource::new(),
                )
                .await
                .unwrap()
                .start()
                .unwrap()
                .wait()
                .await
                .unwrap();
            let RunReport::Fit(report) = result.report().unwrap() else {
                panic!("missing fit")
            };
            near(report.candidate.as_ref().unwrap()[0], 10., 2e-3);
            assert!(report.estimate_qualified(), "{report:?}");
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
