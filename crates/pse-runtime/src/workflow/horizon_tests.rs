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

/// Horizon fixtures exercise retained native artifacts and value rebinds at a realistic
/// worker scale while remaining within a bounded shared test pool.
fn horizon_runtime() -> Runtime {
    crate::workflow::tests::runtime_on(
        2 << 30,
        crate::math::MathPolicy {
            worker_bytes: 1 << 30,
            workspace_bytes: 1 << 30,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
    )
}

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

async fn package(runtime: Runtime) -> (ModelingPackage, [DeclarationId; 3]) {
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
    let package = runtime.modeling_package(rows, physical).await.unwrap();
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
    let allowance = pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
    let profile = pse_backend_native::dynamics::Profile {
        method: pse_backend_native::dynamics::Method::Diffsol,
        end,
        samples: vec![0., end],
        rtol: allowance,
        atol: vec![allowance],
        initial_step: 1e-4,
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
    let mut specification = analysis(
        root,
        fixed(
            &[("x0", 0.), ("r", setpoints[0]), ("u[0{s}]", 0.)],
            &["u[0{s}]"],
        ),
    );
    // Resolve every coordinate used by the independent tracking oracle through
    // the compiler's authored path and index binding.
    specification.bindings.demand = ["x0".into(), "r".into(), "x[0{s}]".into(), "u[0{s}]".into()]
        .into_iter()
        .chain((1..=8).flat_map(|index| {
            ["x", "u"].map(move |name| format!("{name}[{}{{s}}]", index as f64 * PERIOD))
        }))
        .collect();
    HorizonController {
        package: package.clone(),
        analysis: specification,
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

/// Independent optimum of the backward-Euler tracking model. With independent
/// bounded moves and a constant setpoint, reaching it as quickly as possible and
/// then holding it minimizes every remaining squared tracking error.
fn tracking_cost(mut x: f64, r: f64, first: Option<f64>) -> f64 {
    let mut cost = 0.;
    for index in 0..8 {
        let u = if index == 0 {
            first.unwrap_or_else(|| ((r - 0.8 * x) / 0.4).clamp(0., 1.))
        } else {
            ((r - 0.8 * x) / 0.4).clamp(0., 1.)
        };
        x = 0.8 * x + 0.4 * u;
        cost += PERIOD * (x - r).powi(2);
    }
    cost
}
fn controller_objective(step: &crate::workflow::ModelingResult) {
    use pse_model::generated::enums::NumericalTarget;
    let paths = &step.prepared.model.model.compiled().model.paths;
    let at = |name: &str| step.values.scalars[&paths[name]];
    let r = at("r");
    let original = (1..=8)
        .map(|index| PERIOD * (at(&format!("x[{}{{s}}]", index as f64 * PERIOD)) - r).powi(2))
        .sum::<f64>();
    let numerics = step.prepared.solve.numerics();
    let target = crate::workflow::tests::engineering_target(
        numerics,
        NumericalTarget::Objective,
        SemanticId::NIL,
    );
    let Outcome::Native(native) = &step.outcome else {
        panic!("{:?}", step.outcome)
    };
    let candidate = native.candidate.as_ref().unwrap();
    assert!((candidate.objective.unwrap() - original).abs() <= target.budget);
    assert!(native.quality.as_ref().unwrap().feasible());
    assert_eq!(
        numerics.policy.kkt,
        pse_model::numerics::NumericalPolicy::default().kkt
    );
    assert_eq!(
        native.qualification,
        pse_backend_native::solve::Qualification::Stationary
    );
    assert_eq!(
        native.termination.assurance,
        pse_backend_native::solve::Assurance::LocalStationary
    );

    // Independently construct an exact feasible optimum of the authored convex QP.
    let mut optimum = std::collections::BTreeMap::from([
        (paths["x[0{s}]"], at("x0")),
        (paths["u[0{s}]"], at("u[0{s}]")),
    ]);
    let mut x = at("x0");
    for index in 1..=8 {
        let u = ((r - 0.8 * x) / 0.4).clamp(0., 1.);
        x = 0.8 * x + 0.4 * u;
        optimum.insert(paths[&format!("x[{}{{s}}]", index as f64 * PERIOD)], x);
        optimum.insert(paths[&format!("u[{}{{s}}]", index as f64 * PERIOD)], u);
    }
    let plan = &step.prepared.model.case.compiled().plan;
    let columns = plan.columns();
    let rows = plan.structure().rows();
    let row_ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let normalization =
        pse_math::normalization::Normalization::from_policy(numerics, columns, &row_ids).unwrap();
    let observation = native.observation.as_ref().unwrap();
    let stationarity = observation.stationarity.as_ref().unwrap();
    let multipliers = candidate.row_dual.as_ref().unwrap();
    let (lower_dual, upper_dual) = candidate.bound_dual.as_ref().unwrap();
    assert_eq!(candidate.primal.len(), columns.len());
    assert_eq!(stationarity.len(), columns.len());
    assert_eq!(lower_dual.len(), columns.len());
    assert_eq!(upper_dual.len(), columns.len());
    assert_eq!(multipliers.len(), rows.len());
    assert_eq!(observation.equality_residuals.len(), rows.len());
    let accuracy = step.prepared.solve.accuracy();
    let mut gap_bound = 0.;
    let mut bound_sides = 0;
    for (index, id) in columns.iter().enumerate() {
        let primal = candidate.primal[index];
        assert_eq!(primal, step.values.scalars[id]);
        let reference = optimum[id];
        let variable = plan
            .structure()
            .variables()
            .iter()
            .find(|v| v.port.id == *id)
            .unwrap();
        assert!(!variable.fixed);
        assert!(
            variable
                .lower
                .zip(variable.upper)
                .is_none_or(|(lower, upper)| lower != upper)
        );
        let budget =
            crate::workflow::tests::engineering_target(numerics, NumericalTarget::Variable, *id)
                .budget;
        assert!(stationarity[index].is_finite());
        assert!(
            stationarity[index].abs() * normalization.variables[index] / normalization.objective
                <= accuracy.stationarity
        );
        gap_bound += accuracy.stationarity * normalization.objective
            / normalization.variables[index]
            * (primal - reference).abs();
        for (bound, dual, lower) in [
            (variable.lower, lower_dual[index], true),
            (variable.upper, upper_dual[index], false),
        ] {
            assert!(dual.is_finite() && dual >= 0.);
            if let Some(bound) = bound {
                assert!(if lower {
                    reference >= bound && primal >= bound - budget
                } else {
                    reference <= bound && primal <= bound + budget
                });
                assert!(
                    (dual * (primal - bound)).abs() / normalization.objective
                        <= accuracy.complementarity
                );
                gap_bound += accuracy.complementarity * normalization.objective;
                bound_sides += 1;
            } else {
                assert_eq!(dual, 0.);
            }
        }
    }
    for (index, row) in rows.iter().enumerate() {
        assert!(row.lower.is_finite());
        assert_eq!(row.lower, row.upper);
        assert_eq!(observation.bounds[index], (row.lower, row.upper));
        let budget =
            crate::workflow::tests::engineering_target(numerics, NumericalTarget::Row, row.id)
                .budget;
        assert!(observation.equality_residuals[index].unwrap().abs() <= budget);
        assert!(multipliers[index].is_finite());
        gap_bound += multipliers[index].abs() * budget;
    }
    assert_eq!(
        observation.complementarity.as_ref().unwrap().len(),
        bound_sides
    );
    // Convex first-order inequality: the gap is bounded by stationarity times
    // displacement, equality multipliers times admitted row errors, and bound
    // complementarity. The Objective target alone promises no optimum-gap bound.
    let reference = tracking_cost(at("x0"), r, None);
    assert!(
        original - reference <= gap_bound,
        "cost={original}, reference={reference}, admitted KKT/row consequence={gap_bound}"
    );
}

/// NMPC closes the antiwindup loop under the production numerical policy. Each
/// controller satisfies the original physical and KKT conditions and their independently
/// derived convex tracking consequence, applies its actual first move, and respects
/// the physical input limits.
#[tokio::test]
async fn nmpc_closed_loop_on_antiwindup() {
    const STEPS: usize = 20;
    let runtime = horizon_runtime();
    let (package, [root, whole, _]) = package(runtime.clone()).await;
    let (plant, actuator, x) = plant(&package, whole, 0., STEPS).await;
    let setpoints = (0..STEPS)
        .map(|k| if k < 12 { 1. } else { 0.6 })
        .collect::<Vec<_>>();
    let result = run(
        &runtime,
        Horizon {
            output: crate::workflow::StudyOutputDemand::Full {},
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
        let solved = &modeling(&result)[k];
        controller_objective(solved);
        let paths = &solved.prepared.model.model.compiled().model.paths;
        let target = crate::workflow::tests::engineering_target(
            solved.prepared.solve.numerics(),
            pse_model::generated::enums::NumericalTarget::Variable,
            paths["u[0.25{s}]"],
        );
        assert!(
            (-target.budget..=1. + target.budget).contains(&u),
            "{k}: {u}"
        );
        assert_eq!(u, solved.values.scalars[&paths["u[0.25{s}]"]]);
        if k > 0 {
            assert_eq!(step.measured, report.steps[k - 1].reached);
        }
    }
    let applied = |k: usize| report.steps[k].applied[0];
    let reached = |k: usize| report.steps[k].reached[0];
    // Settled on each setpoint.
    let state_budget = |k: usize| {
        let step = &modeling(&result)[k];
        crate::workflow::tests::engineering_target(
            step.prepared.solve.numerics(),
            pse_model::generated::enums::NumericalTarget::Variable,
            step.prepared.model.model.compiled().model.paths["x[0.25{s}]"],
        )
        .budget
    };
    assert!(
        (reached(11) - 1.).abs() <= state_budget(11),
        "{}",
        reached(11)
    );
    assert!(
        (reached(STEPS - 1) - 0.6).abs() <= state_budget(STEPS - 1),
        "{}",
        reached(STEPS - 1)
    );
    assert!(
        (applied(STEPS - 1) - 0.3).abs()
            <= crate::workflow::tests::engineering_target(
                modeling(&result)[STEPS - 1].prepared.solve.numerics(),
                pse_model::generated::enums::NumericalTarget::Variable,
                modeling(&result)[STEPS - 1]
                    .prepared
                    .model
                    .model
                    .compiled()
                    .model
                    .paths["u[0.25{s}]"]
            )
            .budget,
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
        let runtime = horizon_runtime();
        let (package, [root, whole, _]) = package(runtime.clone()).await;
        let (plant, actuator, x) = plant(&package, whole, 0.2, STEPS).await;
        let mut controller = controller(&package, root, x, vec![1.; STEPS]);
        controller.analysis.solver.selection = SolverSelection::Explicit(backend);
        controller.analysis.solver.backend = settings;
        let sqp = backend == Backend::Pounce;
        let before = runtime.native().preparations();
        let handle = runtime
            .start_horizon(
                Horizon {
                    output: crate::workflow::StudyOutputDemand::Full {},
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
    specification.bindings.demand = vec!["x[0{s}]".into(), "x[1{s}]".into(), "arrival".into()];
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
/// the arrival cost's bias of the truth under the production policy; from then on
/// each prior is the previous estimate one period on. Physical states and the
/// independent least-squares decision quantity use the frozen engineering budgets.
#[tokio::test]
async fn mhe_recovers_initial_state() {
    const STEPS: usize = 8;
    let runtime = horizon_runtime();
    let (package, [_, whole, estimator]) = package(runtime.clone()).await;
    let (plant, actuator, x) = plant(&package, whole, 0.8, STEPS).await;
    let plant_budget = crate::workflow::tests::engineering_target(
        plant.numerics(),
        pse_model::generated::enums::NumericalTarget::Variable,
        plant.contract().states[0],
    )
    .budget;
    let result = run(
        &runtime,
        Horizon {
            output: crate::workflow::StudyOutputDemand::Full {},
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
            (sample.measured[0] - truth(sample.time)).abs() <= plant_budget,
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
        let budget = |path: &str| {
            crate::workflow::tests::engineering_target(
                step.prepared.solve.numerics(),
                pse_model::generated::enums::NumericalTarget::Variable,
                paths[path],
            )
            .budget
        };
        assert!(
            (at("x[0{s}]") - start).abs() <= budget("x[0{s}]"),
            "{k}: {} vs {start}",
            at("x[0{s}]")
        );
        assert!(
            (at("x[1{s}]") - now).abs() <= budget("x[1{s}]"),
            "{k}: {} vs {now}",
            at("x[1{s}]")
        );
    }
    // Independently minimize the first window's continuous-model least squares
    // at its actual measurements and arrival prior. No tiny bias must be resolved.
    let first = &steps[0];
    let paths = &first.prepared.model.model.compiled().model.paths;
    let start = first.values.scalars[&paths["x[0{s}]"]];
    let prior = first.values.scalars[&paths["prior"]];
    let arrival = first.values.scalars[&paths["arrival"]];
    assert_eq!(prior, 0.);
    let measurements = WINDOW
        .iter()
        .map(|time| {
            let t = time.parse::<f64>().unwrap();
            (
                (-t).exp(),
                first.values.scalars[&paths[&format!("y[{time}{{s}}]")]],
            )
        })
        .collect::<Vec<_>>();
    let optimum = (measurements
        .iter()
        .map(|(e, y)| e * (y - 1. + e))
        .sum::<f64>()
        + arrival * prior)
        / (measurements.iter().map(|(e, _)| e * e).sum::<f64>() + arrival);
    let cost = |initial: f64| {
        measurements
            .iter()
            .map(|(e, y)| (1. + (initial - 1.) * e - y).powi(2))
            .sum::<f64>()
            + arrival * (initial - prior).powi(2)
    };
    let allowance = crate::workflow::tests::engineering_target(
        first.prepared.solve.numerics(),
        pse_model::generated::enums::NumericalTarget::Objective,
        SemanticId::NIL,
    )
    .budget;
    assert!((cost(start) - cost(optimum)).abs() <= allowance);
}

/// One native horizon retains every completed step and reopens recorded progress.
#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn horizon_records_one_durable_attempt() {
    use crate::workflow::RunDurability;
    const STEPS: usize = 4;
    let runtime = crate::workflow::durable_tests::durable_runtime_on(horizon_runtime());
    let (package, [root, whole, _]) = package(runtime.clone()).await;
    let (plant, actuator, x) = plant(&package, whole, 0.2, STEPS).await;
    let handle = runtime
        .start_horizon(
            Horizon {
                output: crate::workflow::StudyOutputDemand::Full {},
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
    let run = handle.canonical_run_key().unwrap().to_owned();
    let attempt = handle.canonical_attempt_key().unwrap().to_owned();
    let result = handle.wait().await.unwrap();
    assert!(result.usable(), "{:?}", result.report());
    let RunDurability::Durable(record) = result.durability() else {
        panic!("ephemeral horizon")
    };
    assert_eq!(
        record.attempt.as_ref().unwrap().outcome.as_deref(),
        Some("succeeded")
    );
    assert_eq!(
        record
            .solutions
            .iter()
            .map(|(step, _)| *step)
            .collect::<Vec<_>>(),
        (0..STEPS).collect::<Vec<_>>()
    );
    let mut progress = runtime
        .progress(&run, &attempt, pse_columnar::CancellationToken::new())
        .await
        .unwrap();
    let mut samples = Vec::new();
    while let Some(page) = progress.next_page().await.unwrap() {
        for event in page {
            if event.phase == "horizon.step" {
                let crate::workflow::ProgressMetricDocument::Integer(step) = event.values["step"]
                else {
                    panic!("non-integral native step")
                };
                samples.push(step);
            }
        }
    }
    assert_eq!(
        samples,
        (0..STEPS).map(|step| step as i64).collect::<Vec<_>>()
    );
}

/// A horizon is refused before any effect when its loop is inconsistent: a stage path that
/// is not a case value of its specification (steps would respecialize), a controller that
/// binds an estimate without an estimator, a measurement that is not a plant output, an
/// input that is not a plant parameter, and a plant profile that ends before the loop.
#[tokio::test]
async fn horizon_refuses_inconsistent_loops() {
    let runtime = horizon_runtime();
    let (package, [root, whole, _]) = package(runtime.clone()).await;
    let (plant, actuator, x) = plant(&package, whole, 0., 4).await;
    let horizon = |controller: HorizonController, steps: usize, input: SemanticId| Horizon {
        output: crate::workflow::StudyOutputDemand::Full {},
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
/// Both prediction and full solves use the production KKT and physical budgets.
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
    let runtime = horizon_runtime();
    let (package, [root, whole, _]) = package(runtime.clone()).await;
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
    let result = run(
        &runtime,
        Horizon {
            output: crate::workflow::StudyOutputDemand::Full {},
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
/// Independently assess the first move's tracking regret at each loop's actual
/// measured state, using the production objective allowance as an empirical
/// decision criterion. KKT acceptance does not certify this regret bound. A prediction
/// references its background solve, whose primal is at a different state, so it
/// cannot be compared as if that background primal were the applied prediction.
fn matches_full_resolve(advanced: &RunResult, full: &RunResult) -> Vec<HorizonDecision> {
    let (advanced_report, full_report) = (advanced.horizon().unwrap(), full.horizon().unwrap());
    assert_eq!(advanced_report.steps.len(), full_report.steps.len());
    for (k, f) in full_report.steps.iter().enumerate() {
        assert_eq!(f.decision, HorizonDecision::Solved, "{k}");
    }
    for run in [advanced, full] {
        for result in modeling(run) {
            controller_objective(result);
        }
        for (k, sample) in run.horizon().unwrap().steps.iter().enumerate() {
            let source = &modeling(run)[sample.controller.unwrap()];
            let paths = &source.prepared.model.model.compiled().model.paths;
            let objective_budget = crate::workflow::tests::engineering_target(
                source.prepared.solve.numerics(),
                pse_model::generated::enums::NumericalTarget::Objective,
                SemanticId::NIL,
            )
            .budget;
            let r = if k < 12 { 1. } else { 0.6 };
            let regret = tracking_cost(sample.measured[0], r, Some(sample.applied[0]))
                - tracking_cost(sample.measured[0], r, None);
            assert!(
                regret.abs() <= objective_budget,
                "{k}: empirical first-move regret {regret}, allowance={objective_budget} ({:?})",
                sample.decision
            );
            let input_budget = crate::workflow::tests::engineering_target(
                source.prepared.solve.numerics(),
                pse_model::generated::enums::NumericalTarget::Variable,
                paths["u[0.25{s}]"],
            )
            .budget;
            assert!((-input_budget..=1. + input_budget).contains(&sample.applied[0]));
            let state_budget = crate::workflow::tests::engineering_target(
                source.prepared.solve.numerics(),
                pse_model::generated::enums::NumericalTarget::Variable,
                paths["x[0.25{s}]"],
            )
            .budget;
            let expected = 2. * sample.applied[0]
                + (sample.measured[0] - 2. * sample.applied[0]) * (-PERIOD).exp();
            assert!(
                (sample.reached[0] - expected).abs() <= state_budget,
                "{k}: plant {} vs {expected}",
                sample.reached[0]
            );
        }
    }
    advanced_report.steps.iter().map(|s| s.decision).collect()
}

/// Advanced-step NMPC (Plan 22 Y5c2): after applying its moves, the controller solves at
/// the state its own solution predicts one period ahead and keeps that solve's parametric
/// factor; at the next sample one backsolve corrects the prediction to the measured state.
/// The loop's KKT conditions are linear in the state while the active set holds, so every
/// predicted sample is assessed against the same empirical first-move decision
/// criterion as a full solve, at its actual measured state.
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
    let decisions = matches_full_resolve(&advanced, &full);
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
    let decisions = matches_full_resolve(&advanced, &full);
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
    matches_full_resolve(&advanced, &full);
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
    let runtime = horizon_runtime();
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
    let cause = match &failed.report {
        Err(cause) => cause.clone(),
        Ok(_) => panic!("zero evaluation cap must refuse the actual target"),
    };
    assert!(cause.strategy_trace().is_some());
    // The compact accumulator copies the same already assessed products,
    // including the original absolute failed ordinal and all mandatory tables.
    let demand = crate::workflow::StudyOutputDemand::Compact {
        detail: Default::default(),
        failed_detail: Default::default(),
    };
    let mut history = prefix::OriginalHistory::new(&runtime, &demand).unwrap();
    history.admit_step(&runtime, &good).unwrap();
    let mut first_original = RunResult::joined(
        completed.run_id,
        runtime.clone(),
        crate::workflow::RunRequest::Modeling(vec![good.clone()]),
        None,
        Ok(RunReport::Modeling(vec![first.clone()])),
    )
    .completed();
    history.validate_original(&first_original).unwrap();
    history.export(&first_original).await.unwrap();
    history.take_original(&mut first_original);
    let mut original_failure = RunResult::joined(
        completed.run_id,
        runtime.clone(),
        crate::workflow::RunRequest::Modeling(vec![bad.clone()]),
        None,
        Err(cause.clone()),
    );
    original_failure.modeling_origin = 1;
    let mut original_failure = original_failure.completed();
    let failed_assessments = serde_json::to_value(&original_failure.assessments).unwrap();
    history.admit_step(&runtime, &bad).unwrap();
    history.validate_original(&original_failure).unwrap();
    history.export(&original_failure).await.unwrap();
    history.take_original(&mut original_failure);
    let (
        compact_completion,
        detail,
        summaries,
        _owners,
        _findings,
        _summary_error,
        _detail_membership,
    ) = history.finish(false, None);
    assert!(detail.unwrap().is_empty());
    assert_eq!(
        compact_completion
            .solves
            .iter()
            .map(|row| row.step)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(
        serde_json::to_value(&compact_completion.assessments[..completed.assessments.len()])
            .unwrap(),
        serde_json::to_value(&completed.assessments).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&compact_completion.assessments[completed.assessments.len()..])
            .unwrap(),
        failed_assessments
    );
    assert!(
        summaries.contains_key(&pse_relations::generated::runtime::resolved_numerics::RELATION_ID)
    );
    let summary_runs = summaries[&solve_runs::RELATION_ID]
        .iter()
        .flat_map(|batch| solve_runs::Row::rows(batch).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(summary_runs, compact_completion.solves);
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

/// Compact execution uses the same actual native session and advanced-step
/// factor, while retaining only current role state and the rolling input window.
#[tokio::test]
async fn compact_horizon_bounds_roles_and_keeps_actual_background_permission() {
    const STEPS: usize = 20;
    let runtime = horizon_runtime();
    let (package, [root, whole, _]) = package(runtime.clone()).await;
    let (plant, actuator, x) = plant(&package, whole, 0., STEPS).await;
    let mut controller = controller(&package, root, x, vec![1.; STEPS]);
    controller.advanced = Some(AdvancedStep {
        predictions: vec![("x0".into(), "x[0.25{s}]".into())],
    });
    let output = crate::workflow::StudyOutputDemand::Compact {
        detail: Default::default(),
        failed_detail: Default::default(),
    };
    let admitted = Arc::new(
        driver::Admitted::new(
            &runtime,
            Horizon {
                output: output.clone(),
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
        .unwrap(),
    );
    let original = prefix::OriginalHistory::new(&runtime, &output).unwrap();
    let staged = crate::workflow::staged::Staged::open(&runtime, None).unwrap();
    let mut execution = driver::Loop::new(
        admitted,
        staged,
        pse_operations::mint_id(),
        crate::CancelSource::new(),
        Arc::new(pse_backend_native::solve::Progress::new(0)),
        None,
        original,
    );
    let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
    let observed = hub.start("horizon role control");
    pse_diagnostics::observation::scope(Some(observed.context()), execution.run())
        .await
        .unwrap();
    let snapshot = observed
        .reader()
        .snapshot(std::time::Duration::from_secs(60));
    for (name, expected) in [
        ("horizon.prediction.requests", STEPS),
        ("horizon.background.requests", STEPS - 1),
    ] {
        assert_eq!(
            snapshot
                .metrics
                .iter()
                .find(|metric| metric.name == name)
                .unwrap()
                .cumulative,
            pse_diagnostics::observation::Value::Count(expected as u64)
        );
    }
    assert!(
        hub.list(0, usize::MAX)
            .operations
            .iter()
            .any(|operation| operation.kind == "horizon.background")
    );
    let (roles, requests, reports, measurements, inputs) = execution.retained_counts();
    assert!(roles <= 4, "{roles}");
    assert_eq!((requests, reports, measurements, inputs), (0, 0, 1, 0));
    assert_eq!(execution.retained_sample_count(), 1);
    let sample = execution.last_sample().unwrap();
    assert_eq!(sample.time, PERIOD * (STEPS - 1) as f64);
    let latest_prediction = hub
        .list(0, usize::MAX)
        .operations
        .into_iter()
        .filter(|operation| operation.kind == "horizon.prediction")
        .max_by_key(|operation| operation.id.sequence)
        .and_then(|operation| hub.reader(operation.id))
        .map(|reader| reader.snapshot(std::time::Duration::from_secs(60)));
    assert_eq!(
        sample.decision,
        HorizonDecision::Predicted,
        "final actual sample: {sample:?}; prediction: {latest_prediction:?}"
    );
    let background = execution
        .retained_result(sample.controller.unwrap())
        .unwrap();
    assert!(background.completion.decision.permits_use());
    let Outcome::Native(native) = &background.outcome else {
        panic!("actual native background required")
    };
    assert!(
        native
            .evidence
            .sensitivity
            .as_ref()
            .unwrap()
            .retained
            .is_some_and(|bytes| bytes > 0)
    );
    assert_eq!(
        execution.accepted_role(driver::Role::Controller),
        sample.controller
    );
    assert!(
        execution
            .completion()
            .assessments
            .iter()
            .all(|row| row.permits_result)
    );
    // A later actually dispatched refusal cannot displace the last accepted
    // role owner. The next native request must use that older original seed.
    let accepted = execution.accepted_role(driver::Role::Controller).unwrap();
    let values = std::collections::BTreeMap::from([
        ("x0".into(), 0.4),
        ("r".into(), 1.),
        ("u[0{s}]".into(), 0.2),
    ]);
    let mut bad = execution
        .prepare_solve(driver::Role::Controller, values.clone(), false)
        .await
        .unwrap();
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
    assert!(
        execution
            .execute_solve(driver::Role::Controller, bad)
            .await
            .is_err()
    );
    assert_eq!(
        execution.accepted_role(driver::Role::Controller),
        Some(accepted)
    );
    let prepared = execution
        .prepare_solve(driver::Role::Controller, values, false)
        .await
        .unwrap();
    let next = execution
        .execute_solve(driver::Role::Controller, prepared)
        .await
        .unwrap();
    let Outcome::Native(native) = &execution.retained_result(next).unwrap().outcome else {
        panic!("actual native successor required")
    };
    assert_eq!(
        native.start_receipt.as_ref().unwrap().previous_attempt,
        Some(accepted)
    );
    assert!(native.start_receipt.as_ref().unwrap().submitted);
    execution.close().await.unwrap();
}

#[tokio::test]
async fn full_horizon_preparation_materialization_requires_actual_pool_admission() {
    let result = antiwindup_with_start_policy(None, false, StartPolicy::PreviousAccepted, 1).await;
    let prepared = &modeling(&result)[0].prepared;
    let runtime = &result.runtime;
    let pool = runtime.shared.pool();
    let filler =
        pse_columnar::MemoryConsumer::new("test:full-horizon-copy-pressure").register(&pool);
    filler
        .try_grow(runtime.shared.budget().memory_limit_bytes.get() - pool.reserved())
        .unwrap();
    assert!(prefix::preparation_copy_owner(runtime, prepared).is_err());
    drop(filler);
    let original_reserved = pool.reserved();
    let owners = prefix::preparation_copy_owner(runtime, prepared).unwrap();
    assert!(pool.reserved() > original_reserved);
    drop(owners);
    assert_eq!(pool.reserved(), original_reserved);
}

#[tokio::test]
async fn compact_horizon_keeps_original_summaries_and_absolute_member_ordinals() {
    use pse_relations::{
        columnar::RelationRow,
        generated::runtime::{resolved_numerics, solve_variables},
    };
    const STEPS: usize = 3;
    let runtime = horizon_runtime();
    let (package, [root, whole, _]) = package(runtime.clone()).await;
    let (plant, actuator, x) = plant(&package, whole, 0., STEPS).await;
    let controller = controller(&package, root, x, vec![1.; STEPS]);
    let prepared = package
        .prepare_analysis(&controller.analysis, &crate::CancelSource::new())
        .await
        .unwrap();
    let symbol = prepared.model.model.compiled().model.paths["u[0.25{s}]"];
    drop(prepared);
    let detail = crate::workflow::StudyDetailDemand {
        relations: Vec::new(),
        members: vec![crate::workflow::StudyMemberDemand::Variable {
            step: 2,
            symbol_id: symbol,
        }],
        maximum_rows: 1,
        maximum_bytes: 1 << 20,
    };
    let result = run(
        &runtime,
        Horizon {
            output: crate::workflow::StudyOutputDemand::Compact {
                detail,
                failed_detail: Default::default(),
            },
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
    )
    .await;
    assert!(result.usable(), "{:?}", result.report());
    let RunReport::Horizon(report) = result.report().unwrap() else {
        panic!("compact horizon must be explicit")
    };
    assert_eq!(report.solves, STEPS);
    assert!(report.detail_error().is_none());
    assert_eq!(result.horizon().unwrap().steps.len(), 1);
    assert_eq!(result.horizon().unwrap().completed_steps, STEPS);
    assert_eq!(
        result
            .completion()
            .unwrap()
            .solves
            .iter()
            .map(|row| row.step)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    let tables = result.tables().unwrap();
    let selected = solve_variables::Row::rows(&tables[&solve_variables::RELATION_ID]).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!((selected[0].step, selected[0].symbol_id), (2, symbol));
    let policies = resolved_numerics::Row::rows(&tables[&resolved_numerics::RELATION_ID]).unwrap();
    assert_eq!(
        policies
            .iter()
            .map(|row| row.step)
            .collect::<std::collections::BTreeSet<_>>(),
        [0, 1, 2].into_iter().collect()
    );
    let mut range = result
        .table_range("runtime.solve_variables", 1..2, 1)
        .unwrap();
    assert!(range.next_batch().await.unwrap().is_none());
    assert!(range.complete());
}

/// Controls use the production loop/finalizer and authentic source and attempt
/// owners; no fixture invents permission or a scientific native report.
async fn owned_loop(
    runtime: &Runtime,
    horizon: Horizon,
    durable: bool,
) -> (
    driver::Loop,
    Arc<driver::Admitted>,
    Option<(String, String)>,
) {
    let maximum_solves = horizon.steps * 3;
    let output = horizon.output.clone();
    let admitted = Arc::new(
        driver::Admitted::new(runtime, horizon, &crate::CancelSource::new())
            .await
            .unwrap(),
    );
    let run_id = pse_operations::mint_id();
    let mut attempt = if durable {
        Some(crate::workflow::durable::DurableAttempt::new(runtime.operations().unwrap()).unwrap())
    } else {
        None
    };
    if let Some(attempt) = &mut attempt {
        let request = admitted.request();
        attempt
            .register_horizon(
                run_id,
                runtime,
                &request.sources,
                request.identity,
                &request.provenance,
            )
            .await
            .unwrap();
        attempt.start(Arc::new(|| {})).await.unwrap();
        attempt
            .begin_horizon_prefix(output.clone(), maximum_solves)
            .unwrap();
    }
    let keys = attempt
        .as_ref()
        .map(|attempt| attempt.canonical_keys(run_id));
    let original = prefix::OriginalHistory::new(runtime, &output).unwrap();
    let staged = crate::workflow::staged::Staged::open(runtime, None).unwrap();
    let execution = driver::Loop::new(
        admitted.clone(),
        staged,
        run_id,
        crate::CancelSource::new(),
        Arc::new(pse_backend_native::solve::Progress::new(0)),
        attempt,
        original,
    );
    (execution, admitted, keys)
}

#[tokio::test]
async fn compact_horizon_post_science_pressure_moves_original_completion_on_refusal() {
    #[cfg(feature = "canonical-tests")]
    let runtime = crate::workflow::durable_tests::durable_runtime_on(horizon_runtime());
    #[cfg(not(feature = "canonical-tests"))]
    let runtime = horizon_runtime();
    let (package, [root, whole, _]) = package(runtime.clone()).await;
    let (plant, actuator, x) = plant(&package, whole, 0., 2).await;
    let output = crate::workflow::StudyOutputDemand::Compact {
        detail: Default::default(),
        failed_detail: Default::default(),
    };
    let (mut execution, admitted, keys) = owned_loop(
        &runtime,
        Horizon {
            output,
            plant,
            period: PERIOD,
            steps: 2,
            inputs: vec![HorizonInput {
                parameter: actuator,
                initial: 0.,
            }],
            estimator: None,
            controller: Some(controller(&package, root, x, vec![1.; 2])),
        },
        cfg!(feature = "canonical-tests"),
    )
    .await;
    execution.refuse_next_export();
    let outcome = execution.run().await;
    assert!(outcome.is_err());
    let expected = execution.before_export_refusal().unwrap().clone();
    assert_eq!(
        serde_json::to_value(execution.completion()).unwrap(),
        expected
    );
    assert_eq!(execution.completion().solves.len(), 1);
    assert_eq!(execution.completion().lineage.len(), 1);
    assert_eq!(execution.completion().assessments.len(), 1);
    let result = execution.finish(runtime.clone(), outcome, false).await;
    assert!(result.report().is_err());
    assert_eq!(
        serde_json::to_value(result.completion().unwrap()).unwrap(),
        expected
    );
    let summary_error = result
        .horizon_output
        .as_ref()
        .unwrap()
        .summary_error()
        .unwrap();
    for id in prefix::summary_ids() {
        assert!(result.table_ids().contains(&id));
        let mut cursor = result
            .cursor_by_id(id, 1, crate::workflow::ResultOrder::Canonical)
            .unwrap();
        let spec = runtime.registry().relation_by_id(id).unwrap();
        assert_eq!(
            cursor.schema(),
            pse_schema::arrow::relation_schema_ref(runtime.registry(), spec).unwrap()
        );
        let error = cursor.next_batch().await.unwrap_err();
        assert_eq!(error.to_string(), summary_error.to_string());
        assert!(
            !cursor.complete(),
            "failed export cannot become complete empty membership for {id}"
        );
    }
    #[cfg(feature = "canonical-tests")]
    if let Some((run, attempt)) = keys {
        let crate::workflow::RunDurability::Durable(record) = result.durability() else {
            panic!("actual durable attempt required")
        };
        assert!(record.attempt.is_err() && record.manifest.is_none());
        assert!(
            runtime
                .operations()
                .unwrap()
                .store()
                .read_results(&run, &attempt)
                .await
                .is_err()
        );
    }
    #[cfg(not(feature = "canonical-tests"))]
    let _ = keys;
    admitted.close_sources().await.unwrap();
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn compact_horizon_cancel_before_first_solve_closes_exact_empty_summary_membership() {
    let runtime = crate::workflow::durable_tests::durable_runtime_on(horizon_runtime());
    let (package, [root, whole, _]) = package(runtime.clone()).await;
    let (plant, actuator, x) = plant(&package, whole, 0., 2).await;
    let output = crate::workflow::StudyOutputDemand::Compact {
        detail: Default::default(),
        failed_detail: Default::default(),
    };
    let (mut execution, admitted, keys) = owned_loop(
        &runtime,
        Horizon {
            output,
            plant,
            period: PERIOD,
            steps: 2,
            inputs: vec![HorizonInput {
                parameter: actuator,
                initial: 0.,
            }],
            estimator: None,
            controller: Some(controller(&package, root, x, vec![1.; 2])),
        },
        true,
    )
    .await;
    execution.cancel_before_first_step();
    let outcome = execution.run().await;
    assert!(outcome.is_err() && execution.completion().solves.is_empty());
    let result = execution.finish(runtime.clone(), outcome, true).await;
    admitted.close_sources().await.unwrap();
    let tables = result.tables().unwrap();
    let findings = pse_relations::generated::runtime::modeling_findings::RELATION_ID;
    for id in prefix::summary_ids() {
        assert!(tables.contains_key(&id), "missing {id}");
        if id != findings {
            assert_eq!(tables[&id].batch().num_rows(), 0);
            let mut cursor = result
                .cursor_by_id(id, 1, crate::workflow::ResultOrder::Canonical)
                .unwrap();
            while let Some(batch) = cursor.next_batch().await.unwrap() {
                assert_eq!(batch.batch().num_rows(), 0);
            }
            assert!(
                cursor.complete(),
                "zero-solve declared empty remains complete"
            );
        }
        let spec = runtime.registry().relation_by_id(id).unwrap();
        assert_eq!(
            tables[&id].batch().schema(),
            pse_schema::arrow::relation_schema_ref(runtime.registry(), spec).unwrap()
        );
    }
    let crate::workflow::RunDurability::Durable(record) = result.durability() else {
        panic!("actual durable attempt required")
    };
    assert!(record.attempt.is_ok(), "{:?}", record.attempt);
    let descriptors = pse_operations::canonical_execution::decode_result_descriptors(
        record.manifest.as_ref().unwrap(),
    )
    .unwrap();
    for id in prefix::summary_ids()
        .into_iter()
        .filter(|id| *id != findings)
    {
        assert_eq!(
            descriptors
                .iter()
                .find(|descriptor| descriptor.name == id.to_string())
                .unwrap()
                .row_count,
            0
        );
    }
    let (run, attempt) = keys.unwrap();
    let admitted_read = runtime
        .operations()
        .unwrap()
        .store()
        .read_results(&run, &attempt)
        .await
        .unwrap();
    drop(admitted_read);
}

#[tokio::test]
async fn compact_mhe_keeps_only_rolling_measurements_inputs_and_arrival_role() {
    const STEPS: usize = 12;
    let runtime = horizon_runtime();
    let (package, [_, whole, estimator]) = package(runtime.clone()).await;
    let (plant, actuator, x) = plant(&package, whole, 0.8, STEPS).await;
    let output = crate::workflow::StudyOutputDemand::Compact {
        detail: Default::default(),
        failed_detail: Default::default(),
    };
    let (mut execution, admitted, _) = owned_loop(
        &runtime,
        Horizon {
            output,
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
        false,
    )
    .await;
    let outcome = execution.run().await;
    assert!(outcome.is_ok(), "{outcome:?}");
    let (roles, requests, reports, measurements, inputs) = execution.retained_counts();
    assert_eq!(
        (roles, requests, reports, measurements, inputs),
        (1, 0, 0, 5, 4)
    );
    let latest = execution
        .retained_result(execution.accepted_role(driver::Role::Estimator).unwrap())
        .unwrap();
    assert!(latest.accepted && latest.completion.decision.permits_use());
    let paths = &latest.prepared.model.model.compiled().model.paths;
    let now = latest.values.scalars[&paths["x[1{s}]"]];
    let expected = 1. - 0.2 * (-((STEPS - 1) as f64) * PERIOD).exp();
    let budget = crate::workflow::tests::engineering_target(
        latest.prepared.solve.numerics(),
        pse_model::generated::enums::NumericalTarget::Variable,
        paths["x[1{s}]"],
    )
    .budget;
    assert!((now - expected).abs() <= budget);
    let result = execution.finish(runtime, outcome, false).await;
    assert!(result.usable(), "{:?}", result.report());
    assert_eq!(result.completion().unwrap().solves.len(), STEPS - 4);
    admitted.close_sources().await.unwrap();
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn compact_multi_step_horizon_seals_one_original_completion_and_seed_inventory() {
    const STEPS: usize = 3;
    let runtime = crate::workflow::durable_tests::durable_runtime_on(horizon_runtime());
    let (package, [root, whole, _]) = package(runtime.clone()).await;
    let (plant, actuator, x) = plant(&package, whole, 0., STEPS).await;
    let result = run(
        &runtime,
        Horizon {
            output: crate::workflow::StudyOutputDemand::Compact {
                detail: Default::default(),
                failed_detail: Default::default(),
            },
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
    )
    .await;
    assert!(result.usable(), "{:?}", result.report());
    let crate::workflow::RunDurability::Durable(record) = result.durability() else {
        panic!("durable horizon required")
    };
    assert_eq!(
        record.attempt.as_ref().unwrap().outcome.as_deref(),
        Some("succeeded")
    );
    assert_eq!(
        record
            .solutions
            .iter()
            .map(|(step, _)| *step)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(
        serde_json::to_value(
            record
                .completion
                .as_ref()
                .unwrap()
                .completion
                .as_ref()
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(result.completion().unwrap()).unwrap()
    );
    let descriptors = pse_operations::canonical_execution::decode_result_descriptors(
        record.manifest.as_ref().unwrap(),
    )
    .unwrap();
    for id in [
        pse_relations::generated::runtime::solve_runs::RELATION_ID,
        pse_relations::generated::runtime::candidate_assessments::RELATION_ID,
        pse_relations::generated::runtime::run_lineage::RELATION_ID,
    ] {
        assert_eq!(
            descriptors
                .iter()
                .find(|descriptor| descriptor.name == id.to_string())
                .unwrap()
                .row_count,
            STEPS as u64
        );
    }
    for (_, solution) in &record.solutions {
        assert!(runtime.operations().unwrap().seed(*solution).await.is_ok());
    }
}

#[tokio::test]
async fn compact_failed_detail_uses_one_whole_horizon_cap_and_original_outcome() {
    let runtime = horizon_runtime();
    let (package, [root, whole, _]) = package(runtime.clone()).await;
    let (plant, actuator, x) = plant(&package, whole, 0., 3).await;
    let controller = controller(&package, root, x, vec![1.; 3]);
    let prepared = package
        .prepare_analysis(&controller.analysis, &crate::CancelSource::new())
        .await
        .unwrap();
    let symbol = prepared.model.model.compiled().model.paths["u[0.25{s}]"];
    drop(prepared);
    let member = |step| crate::workflow::StudyMemberDemand::Variable {
        step,
        symbol_id: symbol,
    };
    let detail = crate::workflow::StudyDetailDemand {
        relations: Vec::new(),
        members: vec![member(0)],
        maximum_rows: 1,
        maximum_bytes: 1 << 20,
    };
    let failed_detail = crate::workflow::StudyDetailDemand {
        relations: Vec::new(),
        members: vec![member(0), member(1)],
        maximum_rows: 1,
        maximum_bytes: 1 << 20,
    };
    let output = crate::workflow::StudyOutputDemand::Compact {
        detail,
        failed_detail,
    };
    let (mut execution, admitted, _) = owned_loop(
        &runtime,
        Horizon {
            output,
            plant,
            period: PERIOD,
            steps: 3,
            inputs: vec![HorizonInput {
                parameter: actuator,
                initial: 0.,
            }],
            estimator: None,
            controller: Some(controller),
        },
        false,
    )
    .await;
    let values = std::collections::BTreeMap::from([
        ("x0".into(), 0.4),
        ("r".into(), 1.),
        ("u[0{s}]".into(), 0.2),
    ]);
    for _ in 0..2 {
        let prepared = execution
            .prepare_solve(driver::Role::Controller, values.clone(), false)
            .await
            .unwrap();
        execution
            .execute_solve(driver::Role::Controller, prepared)
            .await
            .unwrap();
    }
    let prefix = execution.completion().clone();
    let mut bad = execution
        .prepare_solve(driver::Role::Controller, values, false)
        .await
        .unwrap();
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
    let failure = execution
        .execute_solve(driver::Role::Controller, bad)
        .await
        .unwrap_err();
    assert!(failure.strategy_trace().is_some());
    let result = execution.finish(runtime, Err(failure), false).await;
    let completion = result.completion().unwrap();
    assert_eq!(completion.solves.len(), 3);
    assert_eq!(
        serde_json::to_value(&completion.solves[..2]).unwrap(),
        serde_json::to_value(&prefix.solves).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&completion.assessments[..2]).unwrap(),
        serde_json::to_value(&prefix.assessments).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&completion.lineage[..2]).unwrap(),
        serde_json::to_value(&prefix.lineage).unwrap()
    );
    assert!(completion.solves[2].error.is_some());
    let detail_error = result
        .horizon_output
        .as_ref()
        .unwrap()
        .detail_error()
        .unwrap();
    let variables = pse_relations::generated::runtime::solve_variables::RELATION_ID;
    assert!(
        result.table_ids().contains(&variables),
        "requested failed detail retains declared membership"
    );
    let mut cursor = result
        .cursor_by_id(variables, 1, crate::workflow::ResultOrder::Canonical)
        .unwrap();
    let spec = result.runtime.registry().relation_by_id(variables).unwrap();
    assert_eq!(
        cursor.schema(),
        pse_schema::arrow::relation_schema_ref(result.runtime.registry(), spec).unwrap()
    );
    let export_error = cursor.next_batch().await.unwrap_err();
    assert_eq!(export_error.to_string(), detail_error.to_string());
    assert!(
        !cursor.complete(),
        "requested detail cap refusal is incomplete, never unrequested or complete empty"
    );
    drop(cursor);
    assert!(result.report().unwrap_err().strategy_trace().is_some());
    admitted.close_sources().await.unwrap();
}

/// A real producer's original rows fit repeatedly beside one native admission;
/// old report allowances cannot accumulate behind a Compact history owner.
#[tokio::test]
async fn compact_original_history_releases_report_allowance_in_a_finite_pool() {
    let source = antiwindup_with_start_policy(None, false, StartPolicy::PreviousAccepted, 1).await;
    let runtime = &source.runtime;
    let native = modeling(&source)[0].clone();
    assert!(
        native
            .completion
            .accuracy
            .iter()
            .all(|goal| goal.failure.is_none())
    );
    let prepared = &native.prepared;
    let output = crate::workflow::StudyOutputDemand::Compact {
        detail: Default::default(),
        failed_detail: Default::default(),
    };
    let mut history = prefix::OriginalHistory::new(runtime, &output).unwrap();
    let baseline = history.retained_owner_bytes();
    history.admit_step(runtime, prepared).unwrap();
    let allowance = history.retained_owner_bytes() - baseline;
    let pool = runtime.shared.pool();
    let limit = runtime.shared.budget().memory_limit_bytes.get();
    let cushion = allowance + 256 * 1024;
    let filler =
        pse_columnar::MemoryConsumer::new("test:horizon-one-producer-allowance").register(&pool);
    filler.try_grow(limit - pool.reserved() - cushion).unwrap();
    for step in 0..32 {
        if step != 0 {
            history.admit_step(runtime, prepared).unwrap();
        }
        let mut original = RunResult::joined(
            source.run_id,
            runtime.clone(),
            crate::workflow::RunRequest::Modeling(vec![prepared.clone()]),
            None,
            Ok(RunReport::Modeling(vec![native.clone()])),
        );
        original.modeling_origin = step;
        original = original.completed();
        let expected = serde_json::to_value(original.completion().unwrap()).unwrap();
        history.validate_original(&original).unwrap();
        let issued = history.retained_owner_bytes();
        history.take_original(&mut original);
        assert!(
            history.retained_owner_bytes() < issued,
            "report-only native capacity must leave each committed step"
        );
        assert_eq!(
            serde_json::to_value(&history.completion.solves[step..]).unwrap(),
            expected["solves"]
        );
        assert_eq!(
            serde_json::to_value(&history.completion.lineage[step..]).unwrap(),
            expected["lineage"]
        );
        assert_eq!(
            serde_json::to_value(&history.completion.assessments[step..]).unwrap(),
            expected["assessments"]
        );
    }
    assert_eq!(history.completion.solves.len(), 32);
    assert!(
        history.retained_owner_bytes() < baseline + 2 * allowance,
        "retained originals must follow row/capacity extent, not 32 report envelopes"
    );
    drop(filler);
    drop(history);
}

#[tokio::test]
async fn compact_summary_refusal_invalidates_prior_complete_prefixes() {
    let runtime = horizon_runtime();
    let (package, [root, whole, _]) = package(runtime.clone()).await;
    let (plant, actuator, x) = plant(&package, whole, 0., 2).await;
    let output = crate::workflow::StudyOutputDemand::Compact {
        detail: Default::default(),
        failed_detail: Default::default(),
    };
    let (mut execution, admitted, _) = owned_loop(
        &runtime,
        Horizon {
            output,
            plant,
            period: PERIOD,
            steps: 2,
            inputs: vec![HorizonInput {
                parameter: actuator,
                initial: 0.,
            }],
            estimator: None,
            controller: Some(controller(&package, root, x, vec![1.; 2])),
        },
        false,
    )
    .await;
    let values = std::collections::BTreeMap::from([
        ("x0".into(), 0.4),
        ("r".into(), 1.),
        ("u[0{s}]".into(), 0.2),
    ]);
    let first = execution
        .prepare_solve(driver::Role::Controller, values.clone(), false)
        .await
        .unwrap();
    execution
        .execute_solve(driver::Role::Controller, first)
        .await
        .unwrap();
    let first = serde_json::to_value(execution.completion()).unwrap();
    execution.refuse_next_export();
    let second = execution
        .prepare_solve(driver::Role::Controller, values, false)
        .await
        .unwrap();
    let refusal = execution
        .execute_solve(driver::Role::Controller, second)
        .await
        .unwrap_err();
    let second = execution.before_export_refusal().unwrap().clone();
    let aggregate = serde_json::to_value(execution.completion()).unwrap();
    for name in ["assessments", "solves", "lineage", "accuracy_goals"] {
        let mut expected = first[name].as_array().unwrap().clone();
        expected.extend(second[name].as_array().unwrap().iter().cloned());
        assert_eq!(aggregate[name], serde_json::Value::Array(expected));
    }
    let result = execution.finish(runtime, Err(refusal), false).await;
    assert_eq!(
        serde_json::to_value(result.completion().unwrap()).unwrap(),
        aggregate
    );
    for id in prefix::summary_ids() {
        let mut cursor = result
            .cursor_by_id(id, 1, crate::workflow::ResultOrder::Canonical)
            .unwrap();
        assert!(
            cursor.next_batch().await.is_err(),
            "earlier complete prefix is incomplete after a later issued step is missing for {id}"
        );
        assert!(!cursor.complete());
    }
    admitted.close_sources().await.unwrap();
}

#[tokio::test]
async fn plan38b_replay_horizon_reconstructs_stages_and_reruns_after_cancelled_prefix() {
    let runtime = horizon_runtime();
    let (package, [root, whole, _]) = package(runtime.clone()).await;
    let (plant, actuator, output) = plant(&package, whole, 0.0, 3).await;
    let original = Horizon {
        output: crate::workflow::StudyOutputDemand::Full {},
        plant,
        period: PERIOD,
        steps: 3,
        inputs: vec![HorizonInput {
            parameter: actuator,
            initial: 0.5,
        }],
        estimator: None,
        controller: Some(controller(&package, root, output, vec![0.8, 1.0, 0.6])),
    };
    // Use the production horizon constructor owner before any worker; a retained
    // interrupted original input still carries every control/stage mapping.
    let constructor = crate::workflow::ReplayHorizonConstructor::new(&original).unwrap();
    let sources = vec![package.clone(), package.clone()];
    let restored = constructor
        .reconstruct(&sources, &crate::CancelSource::new())
        .await
        .unwrap();
    assert_eq!(restored.steps, original.steps);
    assert_eq!(restored.period.to_bits(), original.period.to_bits());
    assert_eq!(
        restored.controller.as_ref().unwrap().moves,
        original.controller.as_ref().unwrap().moves
    );
    assert_eq!(
        restored.controller.as_ref().unwrap().bindings,
        original.controller.as_ref().unwrap().bindings
    );
    assert_eq!(
        restored.controller.as_ref().unwrap().analysis.bindings,
        original.controller.as_ref().unwrap().analysis.bindings
    );

    // Keep the actual closed-loop supervisor control small: two periods.
    let mut open = restored;
    open.steps = 2;
    let handle = runtime
        .start_horizon(open, &crate::CancelSource::new())
        .await
        .unwrap();
    handle.cancel();
    let stopped = handle.wait().await.unwrap();
    let replay = stopped.replay();
    let document = replay.document_with_limit(128 << 20).unwrap();
    assert_eq!(document.coverage.requested, 2);
    assert!(document.coverage.observed <= 2);
    assert!(!document.native_continuation);
    assert!(replay.native_continuation().is_err());
    let next = replay.rerun().await.unwrap();
    let rerun = next.wait().await.unwrap();
    assert_ne!(rerun.run_id, stopped.run_id);
    assert!(rerun.report().is_ok(), "{:?}", rerun.report());
    assert_eq!(rerun.horizon().unwrap().completed_steps, 2);
}
