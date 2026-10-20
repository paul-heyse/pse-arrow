// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded positive review fixtures through existing public scientific owners.
use super::*;
use pse_backend_native::{self as native, execution, solve::*};
use pse_ids::{SemanticId, named_id};
use pse_math::{
    convexity::{Definiteness, GramCertificate},
    normalization::Normalization,
};
use pse_runtime::workflow;
use serde_json::{Value, json};
use std::sync::atomic::AtomicBool;

fn write(spec: &Value, label: &str, value: Value) {
    let output = PathBuf::from(std::env::var("PSE_PROCESS_COST_OUTPUT").unwrap());
    std::fs::write(
        output.join(format!("{}-{label}.json", spec["id"].as_str().unwrap())),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
    if spec["operation"].as_str() != Some("review-sparse-nlp") {
        audit::retain_math_output(&format!("{}-{label}", spec["id"].as_str().unwrap()), &value);
    }
}

pub(super) fn settings_json(settings: &execution::BackendSettings) -> Value {
    match settings {
        execution::BackendSettings::Default => json!({"selection":"backend_defaults"}),
        settings => serde_json::to_value(settings).unwrap(),
    }
}

async fn package(
    owner: &WorkflowRuntime,
    source: &str,
) -> (workflow::Runtime, ModelingPackage, DeclarationId) {
    let runtime = compact::local_runtime(owner);
    let physical = physical_on(owner, &runtime).await;
    let rows = pse_authoring::language::parse(
        source,
        named_id(SemanticId::NIL, "process-cost.review"),
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime.modeling_package(rows, physical).await.unwrap();
    (runtime, package, root)
}

fn coordinate(prepared: &workflow::ModelingSolvePreparation, name: &str) -> SemanticId {
    let mut symbols = prepared
        .model
        .model
        .compiled()
        .model
        .symbols
        .values()
        .filter(|symbol| {
            symbol.lineage.path == name || symbol.lineage.path.ends_with(&format!(".{name}"))
        });
    let id = symbols.next().unwrap().id;
    assert!(symbols.next().is_none());
    id
}

fn native_json(report: &SolveReport) -> Value {
    let metrics = |values: &BTreeMap<String, Metric>| {
        values
            .iter()
            .map(|(name, value)| {
                let value = match value {
                    Metric::Integer(value) => json!({"kind":"integer","value":value}),
                    Metric::Real(value) => {
                        json!({"kind":"real","value":value,"bits":value.to_bits()})
                    }
                    Metric::Text(value) => json!({"kind":"text","value":value}),
                    Metric::Bool(value) => json!({"kind":"boolean","value":value}),
                    Metric::Unavailable(reason) => json!({"kind":"unavailable","reason":reason}),
                };
                (name.clone(), value)
            })
            .collect::<BTreeMap<_, _>>()
    };
    let violations = |values: &[native::quality::Violation]| {
        values
            .iter()
            .map(|v| json!({"id":v.id,"physical":v.physical,"tolerance":v.tolerance}))
            .collect::<Vec<_>>()
    };
    json!({
        "backend":report.backend,"termination":report.termination,
        "qualification":report.qualification,"variables":report.variables,"rows":report.rows,
        "candidate":report.candidate.as_ref().map(|candidate| json!({
            "kind":candidate.kind,"primal":candidate.primal,"objective":candidate.objective,
            "row_dual":candidate.row_dual,"bound_dual":candidate.bound_dual,
            "slacks":candidate.slacks,"reduced_costs":candidate.reduced_costs})),
        "quality":report.quality.as_ref().map(|q| json!({"rows":violations(&q.rows),
            "bounds":violations(&q.bounds),"integrality":violations(&q.integrality),"normalized_max":q.normalized_max})),
        "metrics":metrics(&report.metrics),"options":report.options,"native_defaults":report.native_defaults,
        "provenance":report.provenance,"events":report.events.iter().map(|event| json!({
            "source":event.source,"phase":event.phase,"elapsed_seconds":event.elapsed.as_secs_f64(),
            "values":metrics(&event.values),"incumbent":event.incumbent.as_ref().map(|i| json!({
                "objective":i.objective,"dual_bound":i.dual_bound,"gap":i.gap,"nodes":i.nodes,
                "seconds":i.seconds,"primal":i.primal}))})).collect::<Vec<_>>(),"dropped_events":report.dropped_events,
        "start_receipt":report.start_receipt.as_ref().map(StartReceipt::snapshot),
        "output_seed":report.warm_start.as_ref().map(WarmStart::snapshot),
        "work":{"evaluations":report.evidence.work.evaluations,
            "iterations":report.evidence.work.iterations,"factorizations":report.evidence.work.factorizations,
            "proof_steps":report.evidence.work.proof_steps},
        "start_submitted":report.evidence.start_submitted,
        "reused_native_state":report.evidence.reused_native_state,
        "callback_failure":report.callback_failure().map(ToString::to_string),
        "validation_failure":report.validation_failure().map(ToString::to_string)
    })
}

/// Sparse nonlinear rows and quartic objective, with unique analytic optimum all ones.
async fn sparse_nlp(
    owner: &WorkflowRuntime,
    spec: &Value,
    observations: &mut observations::Observations,
) {
    let n = spec["blocks"].as_u64().unwrap() as usize;
    assert!(matches!(n, 2 | 16));
    let backend = match spec["backend"].as_str().unwrap() {
        "ipopt" => Backend::Ipopt,
        "pounce" => Backend::Pounce,
        "uno" => Backend::Uno,
        other => panic!("unsupported review NLP backend {other}"),
    };
    let mut selected = profile(backend, true);
    selected.presolve = native::presolve::Policy::Off;
    selected.controls.hessian = match spec["hessian"].as_str().unwrap() {
        "exact" => HessianMode::Exact,
        "limited-memory" => HessianMode::LimitedMemory,
        other => panic!("unsupported review Hessian {other}"),
    };
    selected.controls.time_limit = Duration::from_secs(30);
    if backend == Backend::Uno {
        assert_eq!(selected.controls.hessian, HessianMode::LimitedMemory);
        selected.numerics.native_scaling = false;
    }
    assert_eq!(selected.numerics.engineering_relative_fraction, 0.001);
    let mut source = String::from("package review { def Root {");
    for i in 0..n {
        source.push_str(&format!("var x{i}:Scalar; annotation bounds x{i}(0.1,3); annotation start x{i}({}); annotation check x{i}(abs(x{i}-1)<=0.001);", if i % 2 == 0 { 0.5 } else { 0.7 }));
    }
    let mut objectives = Vec::new();
    for i in (0..n).step_by(2) {
        let j = i + 1;
        source.push_str(&format!("eq circle{i}:x{i}*x{i}+x{j}*x{j}<=2;"));
        objectives.push(format!("(x{i}-2)*(x{i}-2)+(x{j}-2)*(x{j}-2)+0.1*(x{i}-x{j})*(x{i}-x{j})*(x{i}-x{j})*(x{i}-x{j})"));
    }
    source.push_str(&format!("let cost:Scalar={}; annotation objective cost(minimize); annotation report cost(\"cost\"); annotation check cost(abs(cost-{n})<=0.001*{n}); }} }}", objectives.join("+")));
    write(
        spec,
        "request",
        json!({"source":source,"backend":backend,"controls":selected.controls,
        "settings":settings_json(&selected.backend),"numerics":selected.numerics,"presolve":"off",
        "start_role":spec["reuse"],"expected_primal":vec![1.;n],"expected_objective":n,
        "scope":"sparse constrained nonquadratic NLP; paired nonlinear rows; each original coordinate has an authored check"}),
    );
    let (runtime, package, root) = package(owner, &source).await;
    let cancel = audit::cancel_source();
    let analysis = package
        .declared_execution(
            root,
            compiler(),
            selected,
            Default::default(),
            seed_limits(),
            &cancel,
        )
        .await
        .unwrap()
        .analysis;
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    assert_eq!(prepared.model.case.compiled().plan.columns().len(), n);
    assert_eq!(
        prepared.model.case.compiled().plan.structure().rows().len(),
        n / 2
    );
    let mut reports = Vec::new();
    let cold = audit::wait(prepared.start().unwrap()).await.unwrap();
    audit::export_result(&runtime, &cold).await;
    observations.run(&cold, owner.runtime.math());
    let cold_native = validate_nlp(&cold, &prepared, backend, n);
    reports.push(json!({"role":"cold","report":native_json(cold_native)}));
    if spec["reuse"].as_str() == Some("accepted") {
        let mut seed = cold_native
            .warm_start
            .as_ref()
            .expect("qualified backend publishes accepted native seed")
            .clone();
        if backend == Backend::Uno {
            // Uno consumes an accepted primal only, even when original recovery
            // also supplies qualified duals in the published source seed.
            seed.payload = execution::adapter(backend)
                .primal_start(cold_native.candidate.as_ref().unwrap().primal.clone())
                .unwrap();
        }
        let seeded = prepared.clone().with_start(seed).unwrap();
        let accepted = audit::wait(seeded.start().unwrap()).await.unwrap();
        audit::export_result(&runtime, &accepted).await;
        observations.run(&accepted, owner.runtime.math());
        let native = validate_nlp(&accepted, &seeded, backend, n);
        let receipt = native
            .start_receipt
            .as_ref()
            .expect("explicit accepted seed has a receipt");
        assert!(receipt.seed.is_some() && receipt.submitted && native.evidence.start_submitted);
        reports.push(json!({"role":"accepted_explicit_seed","report":native_json(native)}));
    }
    write(
        spec,
        "native-reports",
        json!({"workflow_invocation":true,"reports":reports,
        "interpretation_limits":"accepted case includes an independently assessed cold predecessor; explicit seed submission is observed, native internal consumption and speedup are not inferred"}),
    );
    drop(runtime);
}

fn validate_nlp<'a>(
    result: &'a workflow::RunResult,
    prepared: &workflow::ModelingSolvePreparation,
    backend: Backend,
    n: usize,
) -> &'a SolveReport {
    let report = authored_success(result);
    let pse_runtime::math::solves::Outcome::Native(native) = &report.outcome else {
        panic!("explicit NLP must execute the selected native adapter");
    };
    assert_eq!(native.backend, backend);
    assert!(native.quality.as_ref().unwrap().feasible());
    let mut objective = 0.;
    for i in (0..n).step_by(2) {
        let x = variable(result, coordinate(prepared, &format!("x{i}")));
        let y = variable(result, coordinate(prepared, &format!("x{}", i + 1)));
        near(x, 1., 0.001);
        near(y, 1., 0.001);
        assert!(x * x + y * y <= 2. + 0.002);
        objective += (x - 2.).powi(2) + (y - 2.).powi(2) + 0.1 * (x - y).powi(4);
    }
    near(objective, n as f64, 0.001 * n as f64);
    near(
        native.candidate.as_ref().unwrap().objective.unwrap(),
        objective,
        0.001 * n as f64,
    );
    native
}

/// Independent authored-coordinate evaluator, never the lowered coefficient projection.
struct Projection {
    a: f64,
    x: usize,
    y: usize,
}
impl execution::OriginalModel for Projection {
    fn evaluate(&mut self, primal: &[f64]) -> Result<execution::Evaluation, native::ProblemError> {
        let (x, y) = (primal[self.x], primal[self.y]);
        Ok(execution::Evaluation {
            constraints: vec![x + y],
            objective: Some((x - self.a).powi(2) + (y - 0.4).powi(2)),
            sources: vec![],
        })
    }
}

/// Public native adapter batch; no synthetic workflow run or invocation is constructed.
async fn pounce_convex(
    owner: &WorkflowRuntime,
    spec: &Value,
    observations: &mut observations::Observations,
) {
    let count = spec["blocks"].as_u64().unwrap() as usize;
    let workers = spec["workers"].as_u64().unwrap() as usize;
    assert!(matches!(count, 1 | 8) && matches!(workers, 1 | 2 | 4));
    let mut programs = Vec::new();
    let mut proofs = Vec::new();
    let mut normalizations = Vec::new();
    let mut budgets = Vec::new();
    let mut originals = Vec::new();
    let mut compatibilities = Vec::new();
    let values = [0.25, 0.5, 0.75, 1., 1.25, 1.5, 1.75, 2.];
    let cancel = audit::cancel_source();
    for a in values.into_iter().take(count) {
        let source = format!(
            "package review {{ def Root {{ var x:Scalar; var y:Scalar; eq budget:x+y<=1; let cost:Scalar=(x-{a})*(x-{a})+(y-0.4)*(y-0.4); annotation objective cost(minimize); annotation bounds x(0,5); annotation bounds y(0,5); annotation start x(0); annotation start y(0); }} }}"
        );
        let (_runtime, package, root) = package(owner, &source).await;
        let mut selected = profile(Backend::PounceConvex, true);
        selected.controls.threads = workers;
        selected.presolve = native::presolve::Policy::Off;
        let analysis = package
            .declared_execution(
                root,
                compiler(),
                selected,
                Default::default(),
                seed_limits(),
                &cancel,
            )
            .await
            .unwrap()
            .analysis;
        let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
        // Solver admission classifies its own overlay; the original compiler case
        // deliberately leaves coefficient evidence optional until explicitly demanded.
        let class_cancel = pse_diagnostics::Cancellation::new(cancel.token().is_cancelled());
        let compiled = prepared
            .model
            .case
            .compiled()
            .prepare_class(&prepared.model.values, &class_cancel.flag())
            .unwrap();
        cancel.checkpoint().unwrap();
        let coefficients = compiled
            .coefficients
            .as_ref()
            .expect("explicit polynomial coefficient demand")
            .as_ref()
            .clone();
        assert!(coefficients.matches_values(&prepared.model.values));
        let problem = native::CoefficientProblem::from_plan(&compiled.plan, coefficients).unwrap();
        assert_eq!(problem.bounds.len(), 1);
        let proof = match GramCertificate::certify(
            problem.hessian.as_ref().unwrap(),
            1.,
            1 << 20,
            &AtomicBool::new(false),
        )
        .unwrap()
        {
            Definiteness::Psd(proof) => proof,
            other => panic!("convex review objective needs exact PSD proof: {other:?}"),
        };
        let columns = compiled.plan.columns();
        originals.push(Projection {
            a,
            x: columns
                .iter()
                .position(|id| *id == coordinate(&prepared, "x"))
                .unwrap(),
            y: columns
                .iter()
                .position(|id| *id == coordinate(&prepared, "y"))
                .unwrap(),
        });
        normalizations.push(
            Normalization::from_policy(prepared.solve.numerics(), columns, &problem.contract.rows)
                .unwrap(),
        );
        budgets.push(
            native::quality::Tolerances::from_policy(
                prepared.solve.numerics(),
                columns,
                &problem.contract.rows,
            )
            .unwrap(),
        );
        compatibilities.push(prepared.solve.compatibility().unwrap().clone());
        programs.push(problem);
        proofs.push(proof);
    }
    let controls = Controls {
        threads: workers,
        time_limit: Duration::from_secs(10),
        ..Default::default()
    };
    let settings = execution::BackendSettings::PounceConvex(
        native::settings::pounce_convex::Settings::default(),
    );
    write(
        spec,
        "request",
        json!({"workflow_invocation":false,"representation":"direct public coefficient adapter with authored model extraction",
        "a":&values[..count],"b":0.4,"members":count,"workers":workers,"controls":controls,"settings":settings,
        "physical_accuracy":0.001,"original":"min (x-a)^2+(y-0.4)^2 subject to x+y<=1, x/y in [0,5]",
        "cancellation":"native profile has no running interrupt; stop before entry, otherwise bounded native deadline then joined teardown"}),
    );
    let cancellation = pse_diagnostics::Cancellation::new(cancel.token().is_cancelled());
    let signal = cancellation.clone();
    let observation = pse_diagnostics::observation::current();
    let task = tokio::task::spawn_blocking(move || {
        let _observation = observation.as_ref().map(|context| context.enter());
        native::pounce::with_threads(workers, 8 << 20, move || {
            let snapshot = execution::Snapshot::observe(&execution::LINKED);
            let constants = [0.];
            let accuracy = programs
                .iter()
                .zip(&budgets)
                .zip(&normalizations)
                .map(|((_problem, t), n)| {
                    ResolvedAccuracy::resolve(
                        &pse_model::numerics::NumericalPolicy::default(),
                        t,
                        n,
                    )
                    .unwrap()
                })
                .collect::<Vec<_>>();
            let steps = programs
                .iter()
                .zip(&proofs)
                .zip(&budgets)
                .zip(&normalizations)
                .zip(&accuracy)
                .zip(originals.iter_mut())
                .zip(compatibilities)
                .map(
                    |(
                        (((((problem, proof), tolerances), normalization), accuracy), original),
                        compatibility,
                    )| {
                        (
                            execution::Step {
                                snapshot: &snapshot,
                                structure: None,
                                adapter: execution::adapter(Backend::PounceConvex),
                                settings: &settings,
                                controls: &controls,
                                accuracy,
                                execution: Execution::new(cancellation.clone(), &controls),
                                tolerances,
                                normalization,
                                compatibility,
                                warm: None,
                            },
                            execution::Coefficients {
                                lowered: None,
                                problem,
                                certificate: Some(proof),
                                row_constants: &constants,
                                row_bounds: problem.bounds.clone(),
                                original,
                            },
                        )
                    },
                )
                .collect();
            let mut retained = execution::Retained::default();
            let reports = execution::coefficients_batch(&mut retained, steps);
            drop(retained);
            Ok::<_, native::ProblemError>((reports, originals))
        })
        .unwrap()
    });
    tokio::pin!(task);
    let (reports, originals) = tokio::select! {
        joined = &mut task => joined.unwrap(),
        () = cancel.cancelled() => { signal.cancel(); task.await.unwrap() }
    };
    audit::direct_joined();
    write(
        spec,
        "native-reports",
        json!({"workflow_invocation":false,"members":reports.iter().map(|report| match report {
        Ok(report) => native_json(report), Err(error) => json!({"error":error.to_string()})
    }).collect::<Vec<_>>(),"joined_native_scope":true}),
    );
    audit::checkpoint();
    assert_eq!(reports.len(), count);
    for (result, original) in reports.iter().zip(&originals) {
        let report = result.as_ref().unwrap();
        observations.native(report);
        assert_eq!(report.backend, Backend::PounceConvex);
        assert_eq!(report.metrics["batch"], Metric::Integer(count as i64));
        assert!(report.quality.as_ref().unwrap().feasible());
        assert!(report.validation_failure().is_none());
        assert_eq!(report.qualification, Qualification::OptimalWithinTolerance);
        let candidate = report.candidate.as_ref().unwrap();
        let (x, y) = (candidate.primal[original.x], candidate.primal[original.y]);
        let (opt_x, opt_y) = if original.a <= 0.6 {
            (original.a, 0.4)
        } else if original.a < 1.4 {
            ((original.a + 0.6) / 2., (1.4 - original.a) / 2.)
        } else {
            (1., 0.)
        };
        near(x, opt_x, 0.001);
        near(y, opt_y, 0.001);
        assert!(x >= -0.001 && y >= -0.001 && x + y <= 1.001 && x <= 5.001 && y <= 5.001);
        let actual = (x - original.a).powi(2) + (y - 0.4).powi(2);
        let optimum = (opt_x - original.a).powi(2) + (opt_y - 0.4).powi(2);
        near(actual, optimum, 0.001 * optimum.max(1.));
        near(candidate.objective.unwrap(), actual, 0.001 * actual.max(1.));
        assert!(!report.evidence.reused_native_state && !report.evidence.start_submitted);
    }
}

/// Proper SOC and PSD geometry, rather than a one-dimensional nonnegative cone.
async fn conic(
    owner: &WorkflowRuntime,
    spec: &Value,
    observations: &mut observations::Observations,
) {
    let runtime = compact::local_runtime(owner);
    let physical = physical_on(owner, &runtime).await;
    let quantity = physical.quantities().neutral_dimensionless().unwrap();
    let unit = physical
        .quantities()
        .quantity_type(quantity)
        .unwrap()
        .canonical_unit
        .as_id();
    let port = |name: &str| workflow::AnalysisPort {
        symbol_id: named_id(SemanticId::NIL, &format!("review-conic.{name}")),
        quantity_id: quantity.as_id(),
        unit_id: unit,
    };
    let shape = spec["shape"].as_str().unwrap();
    let (matrix, rhs, cone, optimum) = match shape {
        // s=(t,1,2) in SOC_3, so t>=sqrt(5).
        "soc" => (
            native::conic::SparseMatrix::new(3, 1, vec![0, 1], vec![0], vec![-1.]),
            vec![0., 1., 2.],
            native::conic::Cone::SecondOrder { dimension: 3 },
            5_f64.sqrt(),
        ),
        // Upper triangle svec([[t,1],[1,t]])=(t,sqrt(2),t), PSD iff t>=1.
        "psd" => (
            native::conic::SparseMatrix::new(3, 1, vec![0, 2], vec![0, 2], vec![-1., -1.]),
            vec![0., 2_f64.sqrt(), 0.],
            native::conic::Cone::PsdTriangle { order: 2 },
            1.,
        ),
        other => panic!("unsupported proper cone {other}"),
    };
    let request = workflow::ConicRequest {
        variables: vec![port("t")],
        rows: vec![port("r0"), port("r1"), port("r2")],
        objective_port: workflow::AnalysisPort {
            symbol_id: SemanticId::NIL,
            ..port("objective")
        },
        quadratic: native::conic::SparseMatrix::zeros(1, 1),
        objective: vec![1.],
        constraints: matrix,
        rhs,
        cones: vec![cone],
        objective_constant: 0.,
    };
    let mut selected = profile(Backend::Clarabel, true);
    selected.controls.time_limit = Duration::from_secs(10);
    write(
        spec,
        "request",
        json!({"workflow_invocation":false,"request":request,
        "controls":selected.controls,"settings":settings_json(&selected.backend),"numerics":selected.numerics,
        "analytic_optimum":optimum,"coordinate_convention":"Ax+s=b; PSD uses upper triangle svec with sqrt(2) off-diagonal"}),
    );
    let prepared = runtime
        .prepare_conic(request, &physical, selected)
        .await
        .unwrap();
    let result = audit::finish(prepared.start().unwrap()).await.unwrap();
    let pse_runtime::math::solves::Outcome::Native(report) = &result.outcome else {
        panic!("proper cone request must execute a native cone adapter");
    };
    write(
        spec,
        "native-reports",
        json!({"workflow_invocation":false,"reports":[native_json(report)]}),
    );
    observations.native(report);
    assert_eq!(report.backend, Backend::Clarabel);
    assert!(report.quality.as_ref().unwrap().feasible());
    assert_eq!(report.qualification, Qualification::OptimalWithinTolerance);
    let t = report.candidate.as_ref().unwrap().primal[0];
    near(t, optimum, 0.001 * optimum);
    near(
        report.candidate.as_ref().unwrap().objective.unwrap(),
        t,
        0.001 * optimum,
    );
    if shape == "soc" {
        assert!(t >= 5_f64.sqrt() - 0.001 * optimum);
    } else {
        assert!(t - 1. >= -0.001 && t + 1. >= -0.001);
    }
}

pub(super) async fn run(
    owner: &WorkflowRuntime,
    spec: &Value,
    observations: &mut observations::Observations,
) {
    match spec["operation"].as_str().unwrap() {
        "review-sparse-nlp" => Box::pin(sparse_nlp(owner, spec, observations)).await,
        "review-pounce-convex" => Box::pin(pounce_convex(owner, spec, observations)).await,
        "review-conic" => Box::pin(conic(owner, spec, observations)).await,
        operation => panic!("unsupported review operation {operation}"),
    }
}
