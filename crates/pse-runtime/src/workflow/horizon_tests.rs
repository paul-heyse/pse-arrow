// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Rolling horizons (Plan 22 Y5c1): NMPC on the saturated-actuator (antiwindup) loop,
//! prepared-structure reuse across steps, MHE with an arrival cost, and the durable attempt.
use super::*;
use crate::math::solves::Outcome;
use crate::workflow::{
    ModelingAnalysis, ModelingPackage, ModelingSimulation, RunReport, RunResult, Runtime,
    StartSource, tests::compiler_profile,
};
use pse_backend_native::{
    execution::BackendSettings,
    solve::{Backend, ReusePolicy, SolveIntent, SolverSelection, StartPolicy},
};
use pse_compiler::workspace::{ModelingCaseBindings, ModelingVariableState};
use pse_modeling::{Bindings, DeclarationId, Limits, analysis::Route};
use std::sync::Arc;

/// The antiwindup loop of the `control_fixtures` package (`SaturatedPID`): an actuator
/// saturated to the control library PID's output limits [0, 1] drives a first-order process
/// `x' = (2u − x)/1 s` from `x(0) = x0`. On the simultaneous route (backward-Euler Radau
/// collocation over a 2 s horizon of eight 0.25 s elements) the actuator is free within its
/// limits at every collocation point and the objective is the tracking error `∫(x − r)²`:
/// the controller. On the integrated route it is a fixed input: the plant. The estimator
/// fits the same process, its input `u` constant, over a 1 s window of four elements of
/// order-3 Radau collocation to measurements at the element ends. Its initial state is free
/// (no initial condition), and its objective adds the arrival cost `arrival·(x(0) − prior)²`.
const LOOP: &str = "package p {
    collocation radau alpha(1) beta(0) right(true);
    def Loop(horizon: Time = 2{s}) {
      domain t: Time from 0{s} to horizon;
      when analysis.route==analysis.integrated {
        discretize grid on t using integrated(elements=1,order=1);
      }
      when analysis.route==analysis.simultaneous {
        discretize grid on t using radau(elements=8,order=1);
        let cost: Scalar = integral(i in t | (x[i]-r)*(x[i]-r)/1{s});
        annotation objective cost(minimize);
      }
      param x0: Scalar = 0; param r: Scalar = 1;
      var x[i in t]: Scalar; var u[i in t]: Scalar;
      eq rate[i in t]: d(x[i])/di == (2*u[i] - x[i])/1{s};
      eq initial: x[0{s}] == x0;
      annotation bounds u(0, 1);
      annotation start x(0); annotation start u(0.5);
    }
    def Plant { child process: Loop = Loop(horizon=5{s}); }
    def Estimator {
      domain t: Time from 0{s} to 1{s};
      discretize grid on t using radau(elements=4,order=3);
      param samples: Set<Time> = {0{s}, 0.25{s}, 0.5{s}, 0.75{s}, 1{s}};
      param u: Scalar = 0.5; param prior: Scalar = 0; param arrival: Scalar = 0.001;
      var x[i in t]: Scalar; var y[j in samples]: Scalar;
      eq rate[i in t]: d(x[i])/di == (2*u - x[i])/1{s};
      let misfit: Scalar = sum(j in samples | (x[j]-y[j])*(x[j]-y[j])) + arrival*(x[0{s}]-prior)*(x[0{s}]-prior);
      annotation objective misfit(minimize);
      annotation start x(0);
    } }";
/// The sample period and the window samples of the estimator, in seconds.
const PERIOD: f64 = 0.25;
const WINDOW: [&str; 5] = ["0", "0.25", "0.5", "0.75", "1"];

fn package(runtime: Runtime) -> (ModelingPackage, [DeclarationId; 3]) {
    let mut physical = crate::workflow::tests::physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    let rows = pse_authoring::language::parse(
        LOOP,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = |name| rows.iter().find(|r| r.name == name).unwrap().declaration_id;
    let roots = [root("Loop"), root("Plant"), root("Estimator")];
    let package = runtime.modeling_package(rows, physical).unwrap();
    (package, roots)
}
fn fixed(values: &[(&str, f64)], fixed: &[&str]) -> ModelingCaseBindings {
    ModelingCaseBindings {
        members: std::collections::BTreeMap::new(),
        values: values.iter().map(|(p, v)| ((*p).to_owned(), *v)).collect(),
        variables: fixed
            .iter()
            .map(|p| {
                (
                    (*p).to_owned(),
                    ModelingVariableState {
                        fixed: Some(true),
                        ..Default::default()
                    },
                )
            })
            .collect(),
    }
}
/// The plant: the loop's integrated route over 5 s, with the actuator a fixed input, from
/// `x0`, integrated over `steps` periods. Returns it with the actuator's parameter and the
/// process output.
async fn plant(
    package: &ModelingPackage,
    root: DeclarationId,
    x0: f64,
    steps: usize,
) -> (ModelingSimulation, SemanticId, SemanticId) {
    let end = steps as f64 * PERIOD;
    let profile = pse_backend_native::dynamics::Profile {
        method: pse_backend_native::dynamics::Method::Diffsol,
        end,
        samples: vec![0., end],
        rtol: 1e-10,
        atol: vec![1e-12],
        initial_step: 1e-6,
        // The contract parameters: r, u and x0.
        parameter_scales: vec![1.; 3],
        ..Default::default()
    };
    let simulation = package
        .prepare_simulation(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            fixed(
                &[("process.u[0{s}]", 0.5), ("process.x0", x0)],
                &["process.u[0{s}]"],
            ),
            compiler_profile(),
            profile,
            pse_kernels::DerivativeOrder::First,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let actuator = simulation.model().compiled().model.paths["process.u[0{s}]"];
    let c = simulation.contract();
    assert_eq!(c.states.len(), 1);
    assert!(c.parameters.contains(&actuator));
    // The outputs lead with the states.
    let x = c.outputs[0];
    (simulation, actuator, x)
}
fn solver() -> crate::math::solves::SolverProfile {
    let mut solver = crate::workflow::tests::profile();
    solver.intent = SolveIntent::Optimize;
    solver.selection = SolverSelection::Explicit(Backend::Ipopt);
    solver.controls.reuse = ReusePolicy::AllowRebuild;
    solver.controls.start = StartPolicy::PreviousAccepted;
    // Presolve would fold the rebound initial state into the native bounds, and a retained
    // Ipopt problem serves only unchanged bounds.
    solver.presolve = pse_backend_native::presolve::Policy::Off;
    solver
}
fn analysis(root: DeclarationId, case: ModelingCaseBindings) -> ModelingAnalysis {
    ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default().with_analysis(Route::Simultaneous),
        limits: Limits::default(),
        case,
        order: pse_kernels::DerivativeOrder::Second,
        compiler: compiler_profile(),
        solver: solver(),
        numerical: Default::default(),
    }
}
/// The NMPC controller of the loop: the measured process output is its initial state, the
/// setpoint follows `setpoints`, and the actuator at the horizon's start holds the move
/// applied over the previous period, so the first element's move is the only one applied.
fn controller(
    package: &ModelingPackage,
    root: DeclarationId,
    x: SemanticId,
    setpoints: Vec<f64>,
) -> HorizonController {
    HorizonController {
        package: package.clone(),
        analysis: analysis(
            root,
            fixed(
                &[("x0", 0.), ("r", setpoints[0]), ("u[0{s}]", 0.)],
                &["u[0{s}]"],
            ),
        ),
        bindings: vec![
            ("x0".into(), HorizonSignal::Measured(x)),
            ("r".into(), HorizonSignal::Trajectory(setpoints)),
            ("u[0{s}]".into(), HorizonSignal::Applied(0)),
        ],
        moves: vec![("u[0.25{s}]".into(), 0)],
        advanced: None,
    }
}
async fn run(runtime: &Runtime, horizon: Horizon) -> Arc<RunResult> {
    runtime
        .start_horizon(horizon, &crate::CancelSource::new())
        .await
        .unwrap()
        .wait()
        .await
        .unwrap()
}
fn modeling(result: &RunResult) -> &[crate::workflow::ModelingResult] {
    match result.report().unwrap() {
        RunReport::Modeling(steps) => steps,
        other => panic!("{other:?}"),
    }
}

/// NMPC closes the antiwindup loop: the actuator is saturated at its upper limit while the
/// output rises and at its lower limit after the setpoint drops, every applied move
/// respects the limits (declared tolerance 1e-7), and the plant output settles on each
/// setpoint within the declared tolerance of 1e-3, the controller's backward-Euler model
/// against the plant's exact integration notwithstanding.
#[tokio::test]
async fn nmpc_closed_loop_on_antiwindup() {
    const STEPS: usize = 20;
    let runtime = crate::workflow::tests::runtime_with_workspace(32 << 20);
    let (package, [root, whole, _]) = package(runtime.clone());
    let (plant, actuator, x) = plant(&package, whole, 0., STEPS).await;
    let setpoints = (0..STEPS)
        .map(|k| if k < 12 { 1. } else { 0.6 })
        .collect::<Vec<_>>();
    let result = run(
        &runtime,
        Horizon {
            plant,
            period: PERIOD,
            steps: STEPS,
            inputs: vec![HorizonInput {
                parameter: actuator,
                initial: 0.,
            }],
            estimator: None,
            controller: Some(controller(&package, root, x, setpoints.clone())),
        },
    )
    .await;
    assert!(
        result.usable(),
        "{:?}; report={:?}",
        result.assessments(),
        result.report()
    );
    let report = result.horizon().unwrap();
    assert_eq!(
        (report.outputs[0], report.inputs.as_slice()),
        (x, &[actuator][..])
    );
    assert_eq!(report.steps.len(), STEPS);
    assert_eq!(modeling(&result).len(), STEPS);
    for (k, step) in report.steps.iter().enumerate() {
        assert_eq!(step.decision, HorizonDecision::Solved, "{k}");
        assert_eq!(step.controller, Some(k));
        assert!((step.time - k as f64 * PERIOD).abs() < 1e-12);
        let u = step.applied[0];
        assert!((-1e-7..=1. + 1e-7).contains(&u), "{k}: {u}");
        if k > 0 {
            assert_eq!(step.measured, report.steps[k - 1].reached);
        }
    }
    let applied = |k: usize| report.steps[k].applied[0];
    let reached = |k: usize| report.steps[k].reached[0];
    // Saturated while rising from rest and after the setpoint drop.
    assert!(
        applied(0) > 1. - 1e-6 && applied(1) > 1. - 1e-6,
        "{report:?}"
    );
    assert!(applied(12) < 1e-6, "{report:?}");
    // Settled on each setpoint.
    assert!((reached(11) - 1.).abs() < 1e-3, "{}", reached(11));
    assert!(
        (reached(STEPS - 1) - 0.6).abs() < 1e-3,
        "{}",
        reached(STEPS - 1)
    );
    assert!(
        (applied(STEPS - 1) - 0.3).abs() < 1e-3,
        "{}",
        applied(STEPS - 1)
    );
}

/// A horizon prepares each stage's structure once: every step rebinds values on the one
/// prepared view (A6). Each controller step after the first reuses the session's retained
/// native problem, starts from the previous step's solved values and consumes its native
/// seed (N2): Ipopt's primal-dual restart, and POUNCE's SQP working set, which passes the
/// presolve boundary because a value-only rebind keeps the native transformation. The
/// plant integrates on the same session between the steps.
#[tokio::test]
async fn horizon_reuses_prepared_view() {
    const STEPS: usize = 6;
    let mut backends = vec![(Backend::Ipopt, BackendSettings::Default)];
    if cfg!(feature = "solver-pounce") {
        backends.push((
            Backend::Pounce,
            BackendSettings::Pounce(pse_backend_native::settings::pounce::Settings {
                method: pse_backend_native::settings::pounce::Method::ActiveSetSqp,
                ..Default::default()
            }),
        ));
    }
    for (backend, settings) in backends {
        let runtime = crate::workflow::tests::runtime_with_workspace(32 << 20);
        let (package, [root, whole, _]) = package(runtime.clone());
        let (plant, actuator, x) = plant(&package, whole, 0.2, STEPS).await;
        let mut controller = controller(&package, root, x, vec![1.; STEPS]);
        controller.analysis.solver.selection = SolverSelection::Explicit(backend);
        controller.analysis.solver.backend = settings;
        let sqp = backend == Backend::Pounce;
        let before = runtime.native().preparations();
        let handle = runtime
            .start_horizon(
                Horizon {
                    plant,
                    period: PERIOD,
                    steps: STEPS,
                    inputs: vec![HorizonInput {
                        parameter: actuator,
                        initial: 0.,
                    }],
                    estimator: None,
                    controller: Some(controller),
                },
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let result = handle.wait().await.unwrap();
        let after = runtime.native().preparations();
        assert!(result.usable(), "{backend:?}: {:?}", result.assessments());
        assert_eq!(
            after.views - before.views,
            1,
            "{backend:?}: one prepared view"
        );
        assert!(after.rebuilt - before.rebuilt >= STEPS - 1, "{backend:?}");
        let steps = modeling(&result);
        assert_eq!(steps.len(), STEPS);
        let u = steps[0].prepared.model.model.compiled().model.paths["u[0.25{s}]"];
        for (k, step) in steps.iter().enumerate() {
            assert!(step.accepted, "{backend:?} {k}: {:?}", step.diagnostic());
            let Outcome::Native(native) = &step.outcome else {
                panic!("{:?}", step.outcome)
            };
            let receipt = native.start_receipt.as_ref().unwrap();
            if k == 0 {
                assert!(!native.evidence.reused_native_state);
                assert_eq!(receipt.previous_attempt, None);
            } else {
                assert!(native.evidence.reused_native_state, "{backend:?} {k}");
                assert_eq!(receipt.previous_attempt, Some(k - 1), "{backend:?} {k}");
                assert!(receipt.submitted, "{backend:?} {k}");
                assert_eq!(
                    native.evidence.working_set_submitted, sqp,
                    "{backend:?} {k}"
                );
                assert_eq!(step.prepared.starts[&u], StartSource::Predecessor, "{k}");
            }
        }
        // One horizon.step event per sample on the run's stream.
        let (events, _) = handle.progress();
        let samples = events
            .iter()
            .filter(|e| e.phase == "horizon.step")
            .map(|e| e.values["step"].clone())
            .collect::<Vec<_>>();
        assert_eq!(
            samples,
            (0..STEPS)
                .map(|k| pse_backend_native::solve::Metric::Integer(k as i64))
                .collect::<Vec<_>>(),
            "{backend:?}"
        );
    }
}

/// The estimator of a horizon over an open-loop plant from `x0 = 0.8`, with the prior of
/// its first window at 0.
fn estimation(package: &ModelingPackage, root: DeclarationId, x: SemanticId) -> HorizonEstimator {
    let paths = |name: &str| {
        WINDOW
            .iter()
            .map(|t| format!("{name}[{t}{{s}}]"))
            .collect::<Vec<_>>()
    };
    let measured = paths("y");
    let mut values = measured
        .iter()
        .map(|p| (p.as_str(), 0.))
        .collect::<Vec<_>>();
    values.extend([("prior", 0.), ("u", 0.5)]);
    let mut specification = analysis(
        root,
        fixed(
            &values,
            &measured.iter().map(String::as_str).collect::<Vec<_>>(),
        ),
    );
    specification.bindings.demand = vec!["x[0{s}]".into(), "x[1{s}]".into()];
    HorizonEstimator {
        package: package.clone(),
        analysis: specification,
        window: 4,
        measurements: vec![(x, measured)],
        inputs: vec![(0, WindowInput::Constant("u".into()))],
        arrival: vec![Arrival {
            prior: "prior".into(),
            next: "x[0.25{s}]".into(),
            initial: 0.,
        }],
    }
}

/// MHE recovers the initial state: with measurements of the open-loop plant from
/// `x(0) = 0.8` and a first prior of 0, the first window's free initial state lands within
/// the arrival cost's bias of the truth (declared tolerance 1e-3); from then on each prior
/// is the previous estimate one period on, and every window's initial and current states
/// match the plant within 1e-5.
#[tokio::test]
async fn mhe_recovers_initial_state() {
    const STEPS: usize = 8;
    let runtime = crate::workflow::tests::runtime_with_workspace(32 << 20);
    let (package, [_, whole, estimator]) = package(runtime.clone());
    let (plant, actuator, x) = plant(&package, whole, 0.8, STEPS).await;
    let result = run(
        &runtime,
        Horizon {
            plant,
            period: PERIOD,
            steps: STEPS,
            inputs: vec![HorizonInput {
                parameter: actuator,
                initial: 0.5,
            }],
            estimator: Some(estimation(&package, estimator, x)),
            controller: None,
        },
    )
    .await;
    assert!(
        result.usable(),
        "{:?}; report={:?}",
        result.assessments(),
        result.report()
    );
    let report = result.horizon().unwrap();
    let steps = modeling(&result);
    let truth = |t: f64| 1. - 0.2 * (-t).exp();
    for (k, sample) in report.steps.iter().enumerate() {
        assert_eq!(sample.decision, HorizonDecision::OpenLoop);
        assert_eq!(sample.applied, vec![0.5]);
        assert!(
            (sample.measured[0] - truth(sample.time)).abs() < 1e-8,
            "{k}"
        );
        let Some(index) = sample.estimator else {
            assert!(k < 4, "{k}");
            continue;
        };
        assert_eq!(index, k - 4);
        let step = &steps[index];
        assert!(step.accepted, "{:?}", step.diagnostic());
        let paths = &step.prepared.model.model.compiled().model.paths;
        let at = |path: &str| step.values.scalars[&paths[path]];
        let (start, now) = (truth(sample.time - 1.), truth(sample.time));
        let tolerance = if index == 0 { 1e-3 } else { 1e-5 };
        assert!(
            (at("x[0{s}]") - start).abs() < tolerance,
            "{k}: {} vs {start}",
            at("x[0{s}]")
        );
        assert!(
            (at("x[1{s}]") - now).abs() < tolerance,
            "{k}: {} vs {now}",
            at("x[1{s}]")
        );
    }
    // The first window's estimate is biased towards its prior of 0, never onto it.
    let first = &steps[0];
    let paths = &first.prepared.model.model.compiled().model.paths;
    let start = first.values.scalars[&paths["x[0{s}]"]];
    assert!(start < 0.8 && start > 0.799, "{start}");
}

/// A horizon under a durable runtime is one attempt: registered before any effect, with its
/// modeling steps' accepted seeds stored per step (O6) and one `horizon.step` event per
/// sample in its stored progress stream (O5).
#[tokio::test]
async fn horizon_records_one_durable_attempt() {
    use crate::workflow::{Durability, Operations, RunDurability};
    use pse_operations::{attempts::AttemptKind, lifecycle::AttemptState, testing::TestDatabase};
    const STEPS: usize = 4;
    let database = TestDatabase::create().await.unwrap();
    let operations = Operations::connect(
        database.url(),
        "horizon",
        crate::workflow::durable_tests::quick(),
    )
    .await
    .unwrap();
    let runtime = crate::workflow::tests::runtime_with_workspace(32 << 20)
        .with_durability(Durability::Durable(operations));
    let (package, [root, whole, _]) = package(runtime.clone());
    let (plant, actuator, x) = plant(&package, whole, 0.2, STEPS).await;
    let handle = runtime
        .start_horizon(
            Horizon {
                plant,
                period: PERIOD,
                steps: STEPS,
                inputs: vec![HorizonInput {
                    parameter: actuator,
                    initial: 0.,
                }],
                estimator: None,
                controller: Some(controller(&package, root, x, vec![1.; STEPS])),
            },
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let attempt = handle.attempt_id().unwrap();
    let result = handle.wait().await.unwrap();
    assert!(
        result.usable(),
        "{:?}; report={:?}",
        result.assessments(),
        result.report()
    );
    let RunDurability::Durable(record) = result.durability() else {
        panic!("ephemeral horizon")
    };
    assert_eq!(record.attempt_id, attempt);
    let stored = record.attempt.as_ref().unwrap();
    assert_eq!(stored.state, AttemptState::Completed);
    assert_eq!(stored.kind, AttemptKind::Modeling);
    assert_eq!(
        record
            .solutions
            .iter()
            .map(|(step, _)| *step)
            .collect::<Vec<_>>(),
        (0..STEPS).collect::<Vec<_>>()
    );
    let samples = record
        .progress
        .as_ref()
        .unwrap()
        .iter()
        .filter(|e| e.phase == "horizon.step")
        .map(|e| e.values["step"].clone())
        .collect::<Vec<_>>();
    assert_eq!(
        samples,
        (0..STEPS)
            .map(|k| pse_operations::streams::ProgressValue::Integer(k as i64))
            .collect::<Vec<_>>()
    );
    drop(runtime);
    database.remove().await.unwrap();
}

/// A horizon is refused before any effect when its loop is inconsistent: a stage path that
/// is not a case value of its specification (steps would respecialize), a controller that
/// binds an estimate without an estimator, a measurement that is not a plant output, an
/// input that is not a plant parameter, and a plant profile that ends before the loop.
#[tokio::test]
async fn horizon_refuses_inconsistent_loops() {
    let runtime = crate::workflow::tests::runtime_with_workspace(32 << 20);
    let (package, [root, whole, _]) = package(runtime.clone());
    let (plant, actuator, x) = plant(&package, whole, 0., 4).await;
    let horizon = |controller: HorizonController, steps: usize, input: SemanticId| Horizon {
        plant: plant.clone(),
        period: PERIOD,
        steps,
        inputs: vec![HorizonInput {
            parameter: input,
            initial: 0.,
        }],
        estimator: None,
        controller: Some(controller),
    };
    let refusal = |horizon: Horizon| {
        let runtime = runtime.clone();
        async move {
            let error = runtime
                .start_horizon(horizon, &crate::CancelSource::new())
                .await
                .unwrap_err();
            error.to_string()
        }
    };
    let mut unbound = controller(&package, root, x, vec![1.; 4]);
    unbound.analysis.case.values.remove("r");
    let mut estimated = controller(&package, root, x, vec![1.; 4]);
    estimated.bindings[0].1 = HorizonSignal::Estimated("x[1{s}]".into());
    let mut unmeasured = controller(&package, root, x, vec![1.; 4]);
    unmeasured.bindings[0].1 = HorizonSignal::Measured(actuator);
    for (horizon, message) in [
        (horizon(unbound, 4, actuator), "binds only case values"),
        (
            horizon(estimated, 4, actuator),
            "estimate without an estimator",
        ),
        (
            horizon(unmeasured, 4, actuator),
            "is an output of the plant's contract",
        ),
        (
            horizon(controller(&package, root, x, vec![1.; 4]), 4, x),
            "is a parameter of the plant's contract",
        ),
        (
            horizon(controller(&package, root, x, vec![1.; 5]), 5, actuator),
            "covers every period",
        ),
    ] {
        let error = refusal(horizon).await;
        assert!(error.contains(message), "{message}: {error}");
    }
}

/// The antiwindup loop over `STEPS` samples, from rest, whose setpoint drops from 1 to 0.6
/// at sample 12, under a controller that decides samples by advanced step when `advanced`.
/// Its KKT budgets are tightened to 1e-12, so a prediction and a full solve at one state
/// agree to far better than the comparison's tolerance of 1e-8.
async fn antiwindup(advanced: Option<AdvancedStep>) -> Arc<RunResult> {
    antiwindup_with_activity(advanced, false).await
}
async fn antiwindup_with_activity(
    advanced: Option<AdvancedStep>,
    activity: bool,
) -> Arc<RunResult> {
    antiwindup_with_start_policy(advanced, activity, StartPolicy::PreviousAccepted, 20).await
}
async fn antiwindup_with_start_policy(
    advanced: Option<AdvancedStep>,
    activity: bool,
    start: StartPolicy,
    steps: usize,
) -> Arc<RunResult> {
    let runtime = crate::workflow::tests::runtime_with_workspace(32 << 20);
    let (package, [root, whole, _]) = package(runtime.clone());
    let (plant, actuator, x) = plant(&package, whole, 0., steps).await;
    let setpoints = (0..steps)
        .map(|k| if k < 12 { 1. } else { 0.6 })
        .collect::<Vec<_>>();
    let mut controller = controller(&package, root, x, setpoints);
    controller.advanced = advanced;
    controller.analysis.solver.controls.start = start;
    if activity {
        controller
            .analysis
            .solver
            .composition
            .recovery
            .push(pse_model::strategy::StartOrigin::Predicted);
    }
    controller.analysis.solver.numerics.kkt = pse_model::numerics::KktTolerances {
        stationarity: 1e-12,
        complementarity: 1e-12,
    };
    let result = run(
        &runtime,
        Horizon {
            plant,
            period: PERIOD,
            steps,
            inputs: vec![HorizonInput {
                parameter: actuator,
                initial: 0.,
            }],
            estimator: None,
            controller: Some(controller),
        },
    )
    .await;
    assert!(
        result.usable(),
        "{:?}; report={:?}",
        result.assessments(),
        result.report()
    );
    result
}
/// The advanced loop's applied moves and plant outputs against the fully re-solving loop's,
/// within `tolerance`; returns the advanced loop's decisions.
fn matches_full_resolve(
    advanced: &RunResult,
    full: &RunResult,
    tolerance: f64,
) -> Vec<HorizonDecision> {
    let (advanced, full) = (advanced.horizon().unwrap(), full.horizon().unwrap());
    assert_eq!(advanced.steps.len(), full.steps.len());
    for (k, (a, f)) in advanced.steps.iter().zip(&full.steps).enumerate() {
        assert_eq!(f.decision, HorizonDecision::Solved, "{k}");
        let (u, v) = (a.applied[0], f.applied[0]);
        assert!(
            (u - v).abs() < tolerance,
            "{k}: applied {u} vs {v} ({:?})",
            a.decision
        );
        let (x, y) = (a.reached[0], f.reached[0]);
        assert!((x - y).abs() < tolerance, "{k}: reached {x} vs {y}");
    }
    advanced.steps.iter().map(|s| s.decision).collect()
}

/// Advanced-step NMPC (Plan 22 Y5c2): after applying its moves, the controller solves at
/// the state its own solution predicts one period ahead and keeps that solve's parametric
/// factor; at the next sample one backsolve corrects the prediction to the measured state.
/// The loop's KKT conditions are linear in the state while the active set holds, so every
/// predicted sample applies the moves of the full re-solve at the measured state (declared
/// tolerance 1e-8), the backward-Euler model's mismatch with the plant notwithstanding.
/// Where that mismatch moves an actuator off its limit (sample 1) the controller falls
/// back to a full solve.
/// Each background solve keeps a factor of positive size, charged to the job's allowance.
#[tokio::test]
async fn advanced_step_matches_full_resolve() {
    let full = antiwindup(None).await;
    let advanced = antiwindup(Some(AdvancedStep {
        predictions: vec![("x0".into(), "x[0.25{s}]".into())],
    }))
    .await;
    let decisions = matches_full_resolve(&advanced, &full, 1e-8);
    assert_eq!(decisions[0], HorizonDecision::Solved);
    let predicted = decisions
        .iter()
        .filter(|d| **d == HorizonDecision::Predicted)
        .count();
    assert!(predicted >= 15, "{decisions:?}");
    let report = advanced.horizon().unwrap();
    let steps = modeling(&advanced);
    for (k, sample) in report.steps.iter().enumerate() {
        assert_eq!(sample.advanced.is_some(), k + 1 < report.steps.len(), "{k}");
        if sample.decision == HorizonDecision::Predicted {
            assert_eq!(sample.controller, report.steps[k - 1].advanced, "{k}");
            assert_eq!(sample.fallback, None);
        }
        let Some(background) = sample.advanced else {
            continue;
        };
        let Outcome::Native(native) = &steps[background].outcome else {
            panic!("{:?}", steps[background].outcome)
        };
        let parametric = native.evidence.sensitivity.as_ref().unwrap();
        assert!(
            parametric.retained.is_some_and(|b| b > 0),
            "{k}: {parametric:?}"
        );
    }
}

/// Without a predicting path the background solve holds the measured state, so the next
/// sample's prediction steps over a whole period's change. Where that changes the active
/// set (the actuator leaving its limit as the output rises, or reaching it after the
/// setpoint drop) the controller falls back to a full solve and records why; the loop still
/// applies the full re-solve's moves at every sample.
#[tokio::test]
async fn advanced_step_falls_back_on_active_set_change() {
    let full = antiwindup(None).await;
    let advanced = antiwindup(Some(AdvancedStep::default())).await;
    let decisions = matches_full_resolve(&advanced, &full, 1e-8);
    let report = advanced.horizon().unwrap();
    let fallbacks = report
        .steps
        .iter()
        .filter(|s| s.decision == HorizonDecision::Fallback)
        .collect::<Vec<_>>();
    assert!(!fallbacks.is_empty(), "{decisions:?}");
    assert!(
        fallbacks.iter().any(|s| matches!(
            s.fallback,
            Some(pse_backend_native::kkt::Fallback::ActiveSet { .. })
        )),
        "{fallbacks:?}"
    );
    assert!(
        decisions.contains(&HorizonDecision::Predicted),
        "{decisions:?}"
    );
}

#[tokio::test]
async fn advanced_step_activity_start_is_corrected_before_move_authorization() {
    let full = antiwindup(None).await;
    let advanced = antiwindup_with_activity(Some(AdvancedStep::default()), true).await;
    matches_full_resolve(&advanced, &full, 1e-8);
    let report = advanced.horizon().unwrap();
    let corrected = report
        .steps
        .iter()
        .filter(|step| step.activity.is_some())
        .collect::<Vec<_>>();
    assert!(!corrected.is_empty());
    let results = modeling(&advanced);
    for step in corrected {
        assert_eq!(step.decision, HorizonDecision::Fallback);
        assert!(matches!(
            step.fallback,
            Some(pse_backend_native::kkt::Fallback::ActiveSet { .. })
        ));
        assert!(
            results[step.controller.unwrap()]
                .completion
                .decision
                .permits_use()
        );
        let activity = step.activity.as_ref().unwrap();
        assert!(activity.work.backsolves > 0);
        // A start-only tracked/partial endpoint never authorizes a move itself.
        assert_ne!(step.decision, HorizonDecision::Predicted);
    }
}

#[tokio::test]
async fn advanced_no_prior_start_consumes_fresh_screened_activity_with_one_task_owner() {
    let advanced = antiwindup_with_start_policy(
        Some(AdvancedStep::default()),
        true,
        StartPolicy::NoPriorStart,
        4,
    )
    .await;
    let horizon = advanced.horizon().unwrap();
    let results = modeling(&advanced);
    for result in results {
        assert!(
            result
                .prepared
                .starts
                .values()
                .all(|source| *source != StartSource::Predecessor)
        );
    }
    let corrected = horizon
        .steps
        .iter()
        .filter(|step| step.activity.is_some())
        .collect::<Vec<_>>();
    assert!(!corrected.is_empty(), "{:?}", horizon.steps);
    for step in corrected {
        assert_eq!(step.decision, HorizonDecision::Fallback);
        let result = &results[step.controller.unwrap()];
        assert!(result.completion.decision.permits_use());
        let target = &result.prepared.solve;
        assert_eq!(target.controls().start, StartPolicy::NoPriorStart);
        let scope = target.task_scope().unwrap();
        assert!(scope.deadline().is_some());
        let owner = target.task_admission().unwrap();
        assert!(owner.matches_scope(&scope));
        assert!(owner.entry_dispatched());
        assert!(step.activity.as_ref().unwrap().work.backsolves > 0);
    }
}
#[tokio::test]
async fn horizon_proposal_queue_preserves_the_original_clock_and_cancellation_owner() {
    use std::{
        sync::atomic::{AtomicBool, Ordering},
        time::{Duration, Instant},
    };
    let runtime = crate::workflow::tests::runtime_with_workspace(32 << 20);
    let staged = crate::workflow::staged::Staged::open(&runtime, None).unwrap();
    let cancel = crate::CancelSource::new();
    let flag = Arc::new(AtomicBool::new(false));
    let deadline = Instant::now() + Duration::from_secs(5);
    let scope = pse_kernels::ExecutionScope::new(flag.clone(), Some(deadline));
    let original = scope.clone();
    let same_owner = flag.clone();
    let observed = staged
        .native_in_task(1, scope, &cancel, move |_, flag, _| {
            assert!(Arc::ptr_eq(flag, &same_owner));
            let execution = pse_backend_native::solve::Execution::within(
                flag.clone(),
                &pse_backend_native::solve::Controls::default(),
                original,
            )?;
            Ok(execution.scope()?.deadline())
        })
        .await
        .unwrap();
    assert_eq!(observed, Some(deadline));
    assert!(!flag.load(Ordering::Acquire));
    let dispatched = Arc::new(AtomicBool::new(false));
    let effect = dispatched.clone();
    let expired = pse_kernels::ExecutionScope::new(flag.clone(), Some(Instant::now()));
    let refused = staged
        .native_in_task(1, expired, &cancel, move |_, _, _| {
            effect.store(true, Ordering::Release);
            Ok(())
        })
        .await;
    assert!(refused.is_err());
    assert!(!dispatched.load(Ordering::Acquire));
    assert!(!flag.load(Ordering::Acquire));
    assert!(!cancel.token().is_cancelled());
    staged.close().await;
}

#[tokio::test]
async fn later_horizon_failure_retains_native_prefix_and_attributes_only_actual_failed_target() {
    use pse_relations::{
        columnar::RelationRow,
        generated::runtime::{
            solve_runs, solve_strategy_events, solve_strategy_products, solve_variables,
        },
    };
    let completed =
        antiwindup_with_start_policy(None, false, StartPolicy::PreviousAccepted, 1).await;
    let first = modeling(&completed)[0].clone();
    assert!(matches!(&first.outcome, Outcome::Native(_)) && first.accepted);
    let original_tables = completed.tables().unwrap();
    let original_events =
        solve_strategy_events::Row::rows(&original_tables[&solve_strategy_events::RELATION_ID])
            .unwrap();
    let original_products =
        solve_strategy_products::Row::rows(&original_tables[&solve_strategy_products::RELATION_ID])
            .unwrap();
    let original_variables =
        solve_variables::Row::rows(&original_tables[&solve_variables::RELATION_ID]).unwrap();
    assert!(!original_events.is_empty());
    assert!(
        original_variables
            .iter()
            .any(|row| !row.fixed && !row.parameter && row.value.is_some())
    );
    let runtime = completed.runtime.clone();
    let good = first.prepared.clone();
    let mut bad = good.clone();
    let scope = pse_kernels::ExecutionScope::new(
        Arc::default(),
        Some(std::time::Instant::now() + std::time::Duration::from_secs(5)),
    );
    let admission = crate::math::strategy::admission::TaskAdmission::new(
        pse_model::strategy::WorkLimits {
            attempts: 4,
            evaluations: Some(0),
            iterations: None,
            factorizations: None,
            proof_steps: None,
        },
        scope.clone(),
        None,
        false,
    );
    bad.solve = bad.solve.within_admitted_task(scope, admission).unwrap();
    let failed = runtime
        .start_modeling(vec![bad.clone()], false, &crate::CancelSource::new())
        .await
        .unwrap()
        .wait()
        .await
        .unwrap();
    let cause = failed
        .report
        .as_ref()
        .err()
        .expect("zero evaluation cap must refuse the actual target")
        .clone();
    assert!(cause.strategy_trace().is_some());
    let (report, failure) = driver::joined_modeling(
        Err(crate::workflow::WorkflowError::Shared(cause)),
        vec![first.clone()],
        Some(1),
    );
    let mut joined = RunResult::joined(
        completed.run_id,
        runtime.clone(),
        crate::workflow::RunRequest::Modeling(vec![good.clone(), bad]),
        None,
        report,
    );
    joined.modeling_failure = failure;
    let joined = joined.finished(None, false).await;
    assert!(!joined.usable());
    assert!(joined.modeling_result(0).unwrap().accepted);
    assert!(joined.modeling_error(0).is_none());
    assert!(joined.modeling_error(1).unwrap().strategy_trace().is_some());
    let tables = joined.tables().unwrap();
    let events =
        solve_strategy_events::Row::rows(&tables[&solve_strategy_events::RELATION_ID]).unwrap();
    assert!(events.iter().any(|row| row.step == 0));
    assert!(events.iter().any(|row| row.step == 1));
    assert_eq!(
        events
            .iter()
            .filter(|row| row.step == 0)
            .collect::<Vec<_>>(),
        original_events.iter().collect::<Vec<_>>()
    );
    let products =
        solve_strategy_products::Row::rows(&tables[&solve_strategy_products::RELATION_ID]).unwrap();
    // An ordinary native rung has no composed product receipt. Preserve its actual
    // receipt inventory rather than inventing one because a later target failed.
    assert_eq!(
        products
            .iter()
            .filter(|row| row.step == 0)
            .collect::<Vec<_>>(),
        original_products.iter().collect::<Vec<_>>()
    );
    let variables = solve_variables::Row::rows(&tables[&solve_variables::RELATION_ID]).unwrap();
    assert!(variables.iter().any(|row| row.step == 0));
    assert_eq!(
        variables
            .iter()
            .filter(|row| row.step == 0)
            .collect::<Vec<_>>(),
        original_variables.iter().collect::<Vec<_>>()
    );
    let runs = solve_runs::Row::rows(&tables[&solve_runs::RELATION_ID]).unwrap();
    assert!(runs[0].error.is_none());
    assert!(runs[1].error.is_some());

    let (report, failure) = driver::joined_modeling(
        Err(crate::workflow::contract("later plant task failure")),
        vec![first],
        None,
    );
    let mut plant = RunResult::joined(
        completed.run_id,
        runtime,
        crate::workflow::RunRequest::Modeling(vec![good]),
        None,
        report,
    );
    plant.modeling_failure = failure;
    let plant = plant.finished(None, false).await;
    assert!(!plant.usable());
    assert!(plant.modeling_result(0).unwrap().accepted);
    assert!(plant.modeling_error(0).is_none());
    let tables = plant.tables().unwrap();
    let events =
        solve_strategy_events::Row::rows(&tables[&solve_strategy_events::RELATION_ID]).unwrap();
    assert_eq!(events, original_events);
    let products =
        solve_strategy_products::Row::rows(&tables[&solve_strategy_products::RELATION_ID]).unwrap();
    assert_eq!(products, original_products);
    let variables = solve_variables::Row::rows(&tables[&solve_variables::RELATION_ID]).unwrap();
    assert_eq!(variables, original_variables);
    let runs = solve_runs::Row::rows(&tables[&solve_runs::RELATION_ID]).unwrap();
    assert_eq!(runs.len(), 1);
    assert!(runs[0].error.is_none());
}
