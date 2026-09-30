// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Single and multiple shooting (ADR-0110 Outcome 5) against an analytic optimum, with
//! continuity closing from perturbed nodes and path bounds at the samples.
use super::*;
use crate::workflow::tests::{compiler_profile, id};
use native::solve::{Backend, Controls, SolverSelection};
use pse_backend_native::dynamics::{Method, ScheduledInput};
use pse_modeling::{Bindings, Limits};

/// A tracking problem: `x' = (u − x)/1 s` from `x(0) = 0` with the algebraic `y = x²`
/// (the states are ordered y, x),
/// the control `u` scheduled on [0, 1) and [1, 2], the integral cost `∫ (x − 1)² dt/1 s`
/// and the reported terminal miss `(x − 1.5)²`.
async fn tracking(method: Method) -> crate::workflow::ModelingSimulation {
    let mut physical = crate::workflow::tests::physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    let rows = pse_authoring::language::parse(
        "package p { def Tracking { domain t: Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); param u: Scalar = 0.5; var x[i in t]: Scalar; var y[i in t]: Scalar; eq rate[i in t]: d(x[i])/di == (u - x[i])/1{s}; eq square[i in t]: y[i] == x[i]*x[i]; eq initial: x[0{s}] == 0; annotation start x(0); annotation start y(0); let cost: Scalar = integral(i in t | (x[i]-1)*(x[i]-1)/1{s}); annotation check cost(cost >= 0); let miss[i in t]: Scalar = (x[i]-1.5)*(x[i]-1.5); annotation report miss(\"terminal miss\"); } }",
        id(30),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Tracking")
        .unwrap()
        .declaration_id;
    let package = crate::workflow::tests::runtime_with_workspace(32 << 20)
        .modeling_package(rows, physical)
        .unwrap();
    // IDAS DAE forward sensitivities fail their first error test at t = 0 below 1e-8.
    let rtol = if method == Method::Idas { 1e-8 } else { 1e-10 };
    let profile = native::dynamics::Profile {
        method,
        end: 2.,
        samples: vec![0., 0.5, 1., 1.5, 2.],
        rtol,
        atol: vec![1e-12; 2],
        out_rtol: Some(rtol),
        out_atol: vec![1e-12],
        initial_step: 1e-6,
        parameter_scales: vec![1.],
        schedule: vec![ScheduledInput {
            parameter: 0,
            times: vec![1.],
        }],
        ..Default::default()
    };
    package
        .prepare_simulation(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            Default::default(),
            compiler_profile(),
            profile,
            DerivativeOrder::First,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap()
}
fn methods() -> Vec<Method> {
    let mut methods = vec![Method::Diffsol];
    if cfg!(feature = "solver-idas") {
        methods.push(Method::Idas);
    }
    methods
}
fn solver() -> SolverProfile {
    SolverProfile {
        selection: SolverSelection::Explicit(Backend::Ipopt),
        controls: Controls {
            hessian: HessianMode::LimitedMemory,
            ..Default::default()
        },
        presolve: native::presolve::Policy::Off,
        ..Default::default()
    }
}
/// The shooting request on the tracking problem: the terminal miss and the integral cost
/// with unit weights.
fn request(
    simulation: &crate::workflow::ModelingSimulation,
    method: ShootingMethod,
    nodes: Vec<f64>,
    controlled: bool,
) -> ShootingProfile {
    let c = simulation.contract();
    ShootingProfile {
        method,
        nodes,
        controls: if controlled {
            vec![ShootingControl {
                input: c.parameters[0],
                lower: Some(0.),
                upper: Some(5.),
            }]
        } else {
            vec![]
        },
        path: vec![],
        objective: ShootingObjective {
            // The outputs are the states, then the reported miss.
            terminal: BTreeMap::from([(c.outputs[2], 1.)]),
            integral: BTreeMap::from([(c.quadratures[0], 1.)]),
        },
        solver: solver(),
    }
}
/// The analytic optimum of J(u₁, u₂) = ∫₀² (x − 1)² dt + (x(2) − 1.5)², with x linear in
/// the piecewise-constant control: x = u₁(1 − e^{−t}) on [0, 1] and, with s = t − 1,
/// x = c·e^{−s}·u₁ + (1 − e^{−s})·u₂ on [1, 2], c = 1 − e^{−1}. The normal equations
/// of the quadratic give the minimizer; J* = 4.25 − rᵀA⁻¹r.
fn analytic() -> ([f64; 2], f64) {
    let (e1, e2) = ((-1f64).exp(), (-2f64).exp());
    let c = 1. - e1;
    let square = 1. - 2. * c + (1. - e2) / 2.;
    let (ta, tb) = (c * e1, 1. - e1);
    let a11 = square + c * c * (1. - e2) / 2. + ta * ta;
    let a12 = c * (c - (1. - e2) / 2.) + ta * tb;
    let a22 = square + tb * tb;
    let r1 = e1 + c * c + 1.5 * ta;
    let r2 = e1 + 1.5 * tb;
    let det = a11 * a22 - a12 * a12;
    let u = [(r1 * a22 - r2 * a12) / det, (a11 * r2 - a12 * r1) / det];
    (u, 4.25 - (r1 * u[0] + r2 * u[1]))
}
fn solve(problem: ShootingProblem, initial: Option<&[f64]>) -> ShootingReport {
    Arc::new(problem)
        .solve(
            RunId::from_bytes([7; 16]),
            Arc::new(AtomicBool::new(false)),
            Arc::new(native::solve::Progress::new(256)),
            initial,
        )
        .unwrap()
}

/// ADR-0110 Outcome 5: single shooting and multiple shooting (nodes at 0.5, 1 and 1.5 s)
/// reach the analytic optimum of the tracking problem, whose control grid is its schedule
/// and whose objective has an integral part (adjoint gradient) and a terminal part
/// (forward sensitivities), on Diffsol and IDAS (declared tolerances: 1e-5 on the
/// controls, 1e-6 relative on the objective). Simultaneous collocation of the same
/// definition is Y5a's; the comparison here is the optimum of the piecewise-constant
/// control that both routes discretize.
#[tokio::test]
async fn shooting_matches_simultaneous_optimum() {
    let (optimum, value) = analytic();
    assert!(optimum.iter().all(|u| *u > 0. && *u < 5.), "{optimum:?}");
    for method in methods() {
        let simulation = tracking(method).await;
        for (shooting, nodes) in [
            (ShootingMethod::Single, vec![]),
            (ShootingMethod::Multiple, vec![0.5, 1., 1.5]),
        ] {
            let case = format!("{method:?} {shooting:?}");
            let problem = simulation
                .shooting(request(&simulation, shooting, nodes.clone(), true))
                .unwrap();
            assert_eq!(problem.controls(), 2);
            assert_eq!(problem.contract().variables.len(), 2 + nodes.len());
            let report = solve(problem, None);
            let solve = report.solve.as_ref().unwrap();
            assert_eq!(
                solve.termination.category,
                native::solve::Termination::Success,
                "{case}: {:?}",
                solve.termination
            );
            let controls = &report.controls[&simulation.contract().parameters[0]];
            for (u, expected) in controls.iter().zip(optimum) {
                assert!(
                    (u - expected).abs() < 1e-5,
                    "{case}: {controls:?} vs {optimum:?}"
                );
            }
            let objective = report.objective.unwrap();
            assert!(
                (objective - value).abs() <= 1e-6 * value,
                "{case}: {objective} vs {value}"
            );
            assert!(
                report.continuity.unwrap() < 1e-8,
                "{case}: {:?}",
                report.continuity
            );
            assert_eq!(report.nodes.len(), 1 + nodes.len());
            // The model's own checks hold on the stitched trajectory.
            assert!(
                report.checks_complete
                    && !report.checks.is_empty()
                    && report.checks.iter().all(|c| c.satisfied),
                "{case}: {:?}",
                report.checks
            );
        }
    }
}

/// Multiple shooting's continuity rows close from nodes perturbed away from the chained
/// integration, with and without free controls: the stitched trajectory then equals one
/// integration of the horizon at the candidate's controls at every sample (1e-7), and the
/// nodes equal its states there. Invalid requests are refused before any integration.
#[tokio::test]
async fn multiple_shooting_continuity_closes() {
    for method in methods() {
        let simulation = tracking(method).await;
        for controlled in [false, true] {
            let case = format!("{method:?} controlled={controlled}");
            let problem = Arc::new(
                simulation
                    .shooting(request(
                        &simulation,
                        ShootingMethod::Multiple,
                        vec![0.5, 1., 1.5],
                        controlled,
                    ))
                    .unwrap(),
            );
            let execution = Execution::new(Arc::new(AtomicBool::new(false)), &solver().controls);
            let mut initial = problem.initial_point(&execution).unwrap();
            let controls = problem.controls();
            for node in &mut initial[controls..] {
                *node += 0.3;
            }
            let report = problem
                .solve(
                    RunId::from_bytes([7; 16]),
                    Arc::new(AtomicBool::new(false)),
                    Arc::new(native::solve::Progress::new(256)),
                    Some(&initial),
                )
                .unwrap();
            let solve = report.solve.as_ref().unwrap();
            assert_eq!(
                solve.termination.category,
                native::solve::Termination::Success,
                "{case}: {:?}",
                solve.termination
            );
            assert!(
                report.continuity.unwrap() < 1e-9,
                "{case}: {:?}",
                report.continuity
            );
            // One integration of the horizon at the candidate's controls.
            let mut profile = simulation.profile().clone();
            profile.sensitivity = DynamicSensitivity::None;
            let mut worker = simulation.worker(Arc::new(AtomicBool::new(false))).unwrap();
            let horizon = native::dynamics::integrate(
                &mut worker,
                &profile,
                report.parameters.as_ref().unwrap(),
                Arc::default(),
            )
            .unwrap();
            let stitched = report.trajectory.as_ref().unwrap();
            assert_eq!(stitched.samples.len(), horizon.samples.len());
            for (a, b) in stitched.samples.iter().zip(&horizon.samples) {
                assert_eq!(a.time, b.time);
                for (x, y) in a
                    .outputs
                    .iter()
                    .chain(&a.integrals)
                    .zip(b.outputs.iter().chain(&b.integrals))
                {
                    assert!((x - y).abs() < 1e-7, "{case}: t={} {x} vs {y}", a.time);
                }
            }
            let x = simulation
                .contract()
                .differential
                .iter()
                .position(|d| *d)
                .unwrap();
            for (node, t) in report.nodes.iter().skip(1).zip([0.5, 1., 1.5]) {
                let sample = horizon.samples.iter().find(|s| s.time == t).unwrap();
                assert!(
                    (node[0] - sample.state[x]).abs() < 1e-7,
                    "{case}: node at {t}"
                );
            }
        }
    }
    let simulation = tracking(Method::Diffsol).await;
    let c = simulation.contract().clone();
    let base = request(&simulation, ShootingMethod::Multiple, vec![0.5], true);
    for invalid in [
        ShootingProfile {
            nodes: vec![],
            ..base.clone()
        },
        ShootingProfile {
            nodes: vec![2.],
            ..base.clone()
        },
        ShootingProfile {
            nodes: vec![1., 0.5],
            ..base.clone()
        },
        ShootingProfile {
            method: ShootingMethod::Single,
            ..base.clone()
        },
        ShootingProfile {
            controls: vec![ShootingControl {
                input: c.states[0],
                lower: None,
                upper: None,
            }],
            ..base.clone()
        },
        ShootingProfile {
            objective: ShootingObjective {
                terminal: BTreeMap::from([(c.quadratures[0], 1.)]),
                integral: BTreeMap::new(),
            },
            ..base.clone()
        },
        ShootingProfile {
            solver: SolverProfile {
                controls: Controls {
                    hessian: HessianMode::Exact,
                    ..Default::default()
                },
                ..solver()
            },
            ..base.clone()
        },
    ] {
        assert!(matches!(
            simulation.shooting(invalid),
            Err(WorkflowError::Contract(_))
        ));
    }
}

/// Path bounds hold at every sample of the horizon: with x ≤ 0.9 active, single and
/// multiple shooting agree on the controls (1e-5) and satisfy the bound at the samples.
#[tokio::test]
async fn shooting_path_bounds_hold_at_samples() {
    let simulation = tracking(Method::Diffsol).await;
    let c = simulation.contract().clone();
    // The outputs begin with the states; x is the differential one.
    let x = c.differential.iter().position(|d| *d).unwrap();
    let mut reports = Vec::new();
    for (shooting, nodes) in [
        (ShootingMethod::Single, vec![]),
        (ShootingMethod::Multiple, vec![1.]),
    ] {
        let mut profile = request(&simulation, shooting, nodes, true);
        profile.path = vec![PathBound {
            output: c.outputs[x],
            lower: None,
            upper: Some(0.9),
        }];
        let report = solve(simulation.shooting(profile).unwrap(), None);
        let trajectory = report.trajectory.as_ref().unwrap();
        assert!(
            trajectory
                .samples
                .iter()
                .all(|s| s.outputs[x] <= 0.9 + 1e-7)
        );
        assert!(
            trajectory.samples.iter().any(|s| s.outputs[x] > 0.9 - 1e-6),
            "the bound is active"
        );
        reports.push(report);
    }
    let (single, multiple) = (&reports[0].controls, &reports[1].controls);
    for (a, b) in single.values().flatten().zip(multiple.values().flatten()) {
        assert!((a - b).abs() < 1e-5, "{single:?} vs {multiple:?}");
    }
}

/// The physical context of the authored shooting fixtures: Scalar and Time.
fn physical() -> crate::workflow::PhysicalContext {
    let mut physical = crate::workflow::tests::physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    physical
}
fn parse(
    text: &str,
) -> Result<Vec<pse_authoring::language::Declaration>, pse_authoring::AuthoringError> {
    pse_authoring::language::parse(
        text,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
}
/// A `run shooting` fixture declares its method, its controls (schedules held free) and
/// its integration controls; the package admission refuses each incomplete or misplaced
/// declaration before any integration.
#[tokio::test]
async fn shooting_fixture_needs_authored_controls() {
    let physical = physical();
    let runtime = crate::workflow::tests::runtime();
    let def = "def Root {domain t:Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); param u:Scalar=1; var x[i in t]:Scalar; eq rate[i in t]:d(x[i])/di==(u-x[i])/1{s}; eq initial:x[0{s}]==0;}";
    let integrate =
        "integrate samples(0{s},1{s}) relative(1e-8) normalized_absolute(1e-10) step(1e-4{s});";
    let free = "schedule root.u at(0.5{s}) values(1, 1) free lower(0) upper(2);";
    let fixed = "schedule root.u at(0.5{s}) values(1, 1);";
    for (fixture, refusal) in [
        // No integration controls.
        (
            "run shooting; shoot single;".to_owned(),
            "fixture execution metadata disagrees with its route",
        ),
        // No controls, or no method.
        (
            format!("run shooting; {integrate} shoot single;"),
            "a shooting fixture declares its method",
        ),
        (
            format!("run shooting; {integrate} shoot single; {fixed}"),
            "a shooting fixture declares its method",
        ),
        (
            format!("run shooting; {integrate} {free}"),
            "a shooting fixture declares its method",
        ),
        // Multiple shooting needs its inner nodes; single shooting has none.
        (
            format!("run shooting; {integrate} shoot multiple; {free}"),
            "a shooting fixture declares its method",
        ),
        (
            format!("run shooting; {integrate} shoot single nodes(0.5{{s}}); {free}"),
            "a shooting fixture declares its method",
        ),
        // Only a shooting fixture holds a schedule free.
        (
            format!("run integrated; {integrate} {free}"),
            "only a shooting fixture holds a schedule free",
        ),
    ] {
        let text = format!(
            "package p {{ {def} test shot fixture {{dof 0; {fixture}}} {{child root:Root=Root();}} }}"
        );
        let Err(error) =
            runtime.modeling_package(parse(&text).unwrap(), physical.clone())
        else {
            panic!("admitted: {fixture}");
        };
        assert!(error.to_string().contains(refusal), "{fixture}: {error}");
    }
}

/// ADR-0110 Outcome 5 from authored data (ADR-0119): a `run shooting` fixture declares the
/// method and nodes and holds the control schedule free within its bounds; the model's
/// objective annotations name the integral cost and the terminal miss. The declared
/// shooting problem reaches the tracking problem's analytic optimum by single and multiple
/// shooting (declared tolerances: 1e-5 on the controls, 1e-6 relative on the objective),
/// and conformance solves the fixture and passes its model checks.
#[tokio::test]
async fn authored_shooting_fixture_solves() {
    let physical = physical();
    let def = "def Tracking { domain t: Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); param u: Scalar = 0.5; var x[i in t]: Scalar; var y[i in t]: Scalar; eq rate[i in t]: d(x[i])/di == (u - x[i])/1{s}; eq square[i in t]: y[i] == x[i]*x[i]; eq initial: x[0{s}] == 0; annotation start x(0); annotation start y(0); let cost: Scalar = integral(i in t | (x[i]-1)*(x[i]-1)/1{s}); annotation check cost(cost >= 0); let miss[i in t]: Scalar = (x[i]-1.5)*(x[i]-1.5); annotation report miss(\"terminal miss\"); annotation objective cost(minimize, weight = 1); annotation objective miss(minimize, weight = 1); }";
    let fixture = |name: &str, shoot: &str| {
        format!(
            "test {name} fixture {{ dof 0; run shooting; integrate samples(0{{s}}, 0.5{{s}}, 1{{s}}, 1.5{{s}}, 2{{s}}) relative(1e-10) normalized_absolute(1e-12) step(1e-6{{s}}) quadrature_relative(1e-10) quadrature_absolute(root.cost = 1e-12); schedule root.u at(1{{s}}) values(0.5, 0.5) free lower(0) upper(5); {shoot} }} {{ child root: Tracking = Tracking(); }}"
        )
    };
    let text = format!(
        "package p {{ {def} {} {} }}",
        fixture("single", "shoot single;"),
        fixture("multiple", "shoot multiple nodes(0.5{s}, 1{s}, 1.5{s});")
    );
    let rows = parse(&text).unwrap();
    let rendered = pse_authoring::language::render(&rows).unwrap();
    assert!(
        rendered.contains("schedule root.u at(1{s}) values(0.5, 0.5) free lower(0) upper(5);")
            && rendered.contains("shoot multiple nodes(0.5{s}, 1{s}, 1.5{s});"),
        "{rendered}"
    );
    assert_eq!(
        rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
        parse(&rendered)
            .unwrap()
            .iter()
            .map(|r| &r.value)
            .collect::<Vec<_>>()
    );
    let root = |name| rows.iter().find(|r| r.name == name).unwrap().declaration_id;
    let fixtures = [root("single"), root("multiple")];
    let package = crate::workflow::tests::runtime_with_workspace(32 << 20)
        .modeling_package(rows, physical)
        .unwrap();
    let cancel = crate::CancelSource::new();
    let (optimum, value) = analytic();
    for (fixture, nodes) in fixtures.into_iter().zip([0, 3]) {
        let problem = package
            .declared_shooting(
                fixture,
                compiler_profile(),
                None,
                solver(),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(problem.controls(), 2);
        assert_eq!(problem.contract().variables.len(), 2 + nodes);
        let report = solve(problem, None);
        let solve = report.solve.as_ref().unwrap();
        assert_eq!(
            solve.termination.category,
            native::solve::Termination::Success,
            "{:?}",
            solve.termination
        );
        let controls = report.controls.values().next().unwrap();
        for (u, expected) in controls.iter().zip(optimum) {
            assert!((u - expected).abs() < 1e-5, "{controls:?} vs {optimum:?}");
        }
        let objective = report.objective.unwrap();
        assert!(
            (objective - value).abs() <= 1e-6 * value,
            "{objective} vs {value}"
        );
        assert!(report.checks_complete && report.checks.iter().all(|c| c.satisfied));
    }
    let policy = crate::workflow::ModelingConformancePolicy {
        compiler: compiler_profile(),
        solver: solver(),
        numerical: Default::default(),
        limits: Limits::default(),
        maximum_fixtures: 4,
        maximum_checks: 64,
        fixtures: Default::default(),
        diagnostics: None,
        derivatives: native::derivative_diagnostics::Policy {
            perturbation: 1e-6,
            relative_tolerance: 1e-4,
            maximum_cells: 100,
        },
    };
    let report = package.conform(policy, &cancel).await.unwrap();
    assert!(report.passed(), "{:?}", report.checks);
    assert_eq!(
        report
            .checks
            .iter()
            .filter(|c| c.kind == pse_model::generated::enums::ModelingConformanceKind::StartToSolve)
            .count(),
        2
    );
}
