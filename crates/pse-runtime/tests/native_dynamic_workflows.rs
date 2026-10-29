// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public, store-free dynamics/events, shooting and rolling-horizon callers.
//! Requires solver-diffsol, solver-ipopt, force-validate and captured scientific inputs.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "fixture construction and independent scientific assertions fail directly"
)]

#[path = "../../../tests/fixtures/scientific-inputs.rs"]
mod scientific_inputs;
#[path = "reference_support/context.rs"]
mod support;

use futures_util::{FutureExt, future::BoxFuture};
use pse_admission::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
use pse_backend_native::{
    dynamics::{DynamicSensitivity, Method, Profile, ScheduledInput, Termination},
    solve::{
        Backend, Controls, HessianMode, ReusePolicy, SolveIntent, SolverSelection, StartPolicy,
    },
};
use pse_compiler::workspace::{ModelingCaseBindings, ModelingVariableState};
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use pse_model::{
    generated::{
        authored::fit_cases as authored_fit,
        enums::{ModelingAnalysisRoute, NativeBackend, NativeTermination, NumericalTarget},
        identities::{DeclarationId, FitId, InstanceId},
    },
    scalars::PositiveCount,
};
use pse_modeling::{Bindings, Limits, analysis::Route};
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{
        computation_runs, fit_observations, fit_parameters, fitted_parameter_cells,
        modeling_checks, resolved_numerics, response_sensitivities, simulation_samples,
    },
};
use pse_runtime::{
    CancelSource,
    math::solves::{Outcome, SolverProfile},
    workflow::{
        FitDeclarations, FitDerivatives, FitOptimizer, FitProfile, Horizon, HorizonController,
        HorizonDecision, HorizonInput, HorizonSignal, ModelingAnalysis, ModelingPackage,
        ModelingSimulation, PathBound, RunReport, RuntimeOperationRequest, ShootingControl,
        ShootingEvaluationPolicy, ShootingMethod, ShootingObjective, ShootingProfile,
        StudyOutputDemand,
    },
};
use std::{collections::BTreeMap, panic::AssertUnwindSafe, sync::Arc};
use support::{NativeContext, Result};

fn compiler() -> pse_compiler::workspace::Profile {
    pse_compiler::workspace::Profile {
        evaluation: pse_math::jets::EvaluationLimits {
            scratch_bytes: 1 << 20,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Admit the retained physical declarations, rather than constructing a private context
/// or assigning generated standard quantities/preconditions to a test-only runtime.
fn package<'a>(
    context: &'a NativeContext,
    source: &'a str,
) -> BoxFuture<'a, Result<ModelingPackage>> {
    async move {
        let physical = scientific_inputs::package_documents("packages/reference/physical");
        let header: toml::Value = toml::from_str(std::str::from_utf8(&physical["package.toml"])?)?;
        let version: Vec<i64> = header["package"]["version"]
            .as_str()
            .unwrap()
            .split('.')
            .map(|part| part.parse().unwrap())
            .collect();
        assert_eq!(version.len(), 3);
        let mut manifest: toml::Value = toml::from_str(&std::fs::read_to_string(
            support::root().join("tests/fixtures/packages/minimal_explicit/package.toml"),
        )?)?;
        manifest["package"]["id_policy"] = "named".into();
        manifest["package"]["name"] = "native.dynamic.callers".into();
        let requirement = toml::Table::from_iter([
            ("operator".into(), "exact".into()),
            ("major".into(), version[0].into()),
            ("minor".into(), version[1].into()),
            ("patch".into(), version[2].into()),
        ]);
        manifest["package"]["dependencies"] = toml::Value::Array(vec![
            toml::Table::from_iter([
                ("package_id".into(), header["package"]["package_id"].clone()),
                ("version_req".into(), requirement.into()),
            ])
            .into(),
        ]);
        let modeling = BTreeMap::from([
            (
                "package.toml".into(),
                toml::to_string(&manifest)?.into_bytes(),
            ),
            ("models/dynamics.pse".into(), source.as_bytes().to_vec()),
        ]);
        let pool = context.shared.pool();
        let token = pse_columnar::CancellationToken::new();
        let validation = context.runtime.validation_context();
        let load = |texts: &BTreeMap<String, Vec<u8>>| -> Result<OwnedDocumentSet> {
            Ok(OwnedDocumentSet::try_from_bundles(
                vec![load_package_documents_owned(
                    texts,
                    context.runtime.registry(),
                    Default::default(),
                    &pool,
                    &token,
                    &validation,
                )?],
                &pool,
                &token,
            )?)
        };
        let physical_documents = load(&physical)?;
        let physical = Box::pin(
            context
                .runtime
                .physical_from_documents(&physical_documents, &token),
        )
        .await?;
        let documents = load(&modeling)?;
        Ok(Box::pin(context.runtime.modeling_from_documents(
            &documents,
            physical,
            &CancelSource::new(),
        ))
        .await?)
    }
    .boxed()
}

async fn declaration(
    context: &NativeContext,
    package: &ModelingPackage,
    name: &str,
) -> Result<DeclarationId> {
    Ok(package
        .declarations(&context.runtime.preparation_services())
        .await?
        .iter()
        .find(|row| row.name == name)
        .unwrap()
        .declaration_id)
}

fn allowance(
    policy: &pse_model::numerics::ResolvedNumericalPolicy,
    kind: NumericalTarget,
    id: SemanticId,
) -> f64 {
    let target = policy
        .targets
        .iter()
        .find(|target| target.kind == kind && target.id == id)
        .expect("production-resolved target for the independent oracle");
    assert!(target.engineering.as_ref().unwrap().budget > 0.);
    assert!(target.budget.is_finite() && target.budget > 0.);
    target.budget
}

async fn settle(context: &NativeContext, body: std::thread::Result<Result<()>>) -> Result<()> {
    let cleanup = context.close().await;
    match body {
        Ok(result) => {
            result?;
            cleanup
        }
        Err(panic) => {
            let _ = cleanup;
            std::panic::resume_unwind(panic)
        }
    }
}

#[tokio::test]
async fn public_events_reset_physical_state_and_preserve_terminal_prefix_and_refusal() -> Result<()>
{
    let context = NativeContext::new()?;
    let body = AssertUnwindSafe(async {
        // x starts at 1 s and rises at 2 s/s: hit 2 s at t=0.5 s, reset to
        // 4 s, then coast. The event and its successor are authored declarations.
        let integrate = "dof 0; route integrated; procedure integrate; integrate samples(0{s},0.25{s},0.75{s},2{s}) relative(1e-8) normalized_absolute(1e-8) step(1e-4{s});";
        let source = format!(r#"package native_events {{
          def Root {{ domain t:Time from 0{{s}} to 2{{s}};
            discretize grid on t using integrated(elements=1,order=1);
            param p:Scalar=2; var x[i in t]:Time;
            eq rate[i in t]:d(x[i])/di==p; eq initial:x[0{{s}}]==1{{s}};
            let hit[i in t]:Time=x[i]-2{{s}}; let jump[i in t]:Time=2*x[i];
            let wrong:Scalar=0;
            stage coast {{ override eq rate[i in t]:d(x[i])/di==0; }}
            annotation check x(x[i]<=4.01{{s}});
          }}
          test evented fixture {{ {integrate} mode rise;
            event root.hit[0{{s}}] direction(either) tolerance(1e-8{{s}})
              reset(root.x[0{{s}}]=root.jump[0{{s}}]) next(coast);
            mode coast facts(stage.coast=true);
          }} {{ child root:Root=Root(); }}
          test mistyped fixture {{ {integrate} mode rise;
            event root.hit[0{{s}}] direction(either) tolerance(1e-8{{s}})
              reset(root.x[0{{s}}]=root.wrong) next(coast);
            mode coast facts(stage.coast=true);
          }} {{ child root:Root=Root(); }}
          test stopped fixture {{ {integrate} mode stop;
            event root.hit[0{{s}}] direction(either) tolerance(1e-8{{s}}) terminal;
          }} {{ child root:Root=Root(); }}
        }}"#);
        let package = package(&context, &source).await?;
        let services = context.runtime.execution_services();
        let cancel = CancelSource::new();
        let profile = Profile {
            method: Method::Diffsol, end: 2., samples: vec![0., 0.25, 0.75, 2.],
            parameter_scales: vec![1.], sensitivity: DynamicSensitivity::Forward,
            ..Default::default()
        };
        let root = declaration(&context, &package, "evented").await?;
        let simulation = Box::pin(package.declared_simulation(
            services.preparation(), root, compiler(), Some(profile.clone()), Limits::default(), &cancel,
        )).await?;
        assert_eq!(simulation.mode_names().collect::<Vec<_>>(), ["rise", "coast"]);
        let trajectory = Box::pin(simulation.run(&services, &cancel)).await?;
        assert!(trajectory.accepted(), "{:?}", trajectory.validation_error());
        assert!(trajectory.checks_complete());
        assert_eq!(trajectory.report().termination, Termination::Completed);
        assert_eq!(trajectory.report().events.len(), 1);
        assert!((trajectory.report().events[0].time - 0.5).abs() < 1e-5);
        assert_eq!(trajectory.report().samples.iter().map(|sample| sample.mode).collect::<Vec<_>>(), [0, 0, 1, 1]);
        for (sample, expected) in trajectory.report().samples.iter().zip([1., 1.5, 4., 4.]) {
            assert!((sample.outputs[0] - expected).abs() < 1e-6);
        }
        // Moving-root sensitivity cancels: changing the rise rate changes event time,
        // while the reset value at the authored threshold remains exactly 4 s.
        assert!(trajectory.report().samples.last().unwrap().output_sensitivities[0].abs() < 1e-5);
        assert_eq!(trajectory.table("runtime.simulation_events")?.batch().num_rows(), 1);

        let mistyped = declaration(&context, &package, "mistyped").await?;
        let error = Box::pin(package.declared_simulation(
            services.preparation(), mistyped, compiler(), Some(profile.clone()), Limits::default(), &cancel,
        )).await.unwrap_err();
        assert!(error.to_string().contains("physical type"), "{error}");
        let stopped = declaration(&context, &package, "stopped").await?;
        let stopped = Box::pin(package.declared_simulation(
            services.preparation(), stopped, compiler(), Some(Profile { sensitivity: DynamicSensitivity::None, ..profile }),
            Limits::default(), &cancel,
        )).await?;
        let stopped = Box::pin(stopped.run(&services, &cancel)).await?;
        assert_eq!(stopped.report().termination, Termination::Event);
        assert_eq!(stopped.report().samples.len(), 2);
        assert!((stopped.report().events[0].time - 0.5).abs() < 1e-5);
        assert!(!stopped.accepted());
        assert!(!stopped.checks_complete());
        Ok(())
    }).catch_unwind().await;
    settle(&context, body).await
}

const TRACKING: &str = r#"package native_shooting {
  def Tracking { domain t:Time from 0{s} to 2{s};
    discretize grid on t using integrated(elements=1,order=1);
    param u:Scalar=0.5; var x[i in t]:Scalar; var y[i in t]:Scalar;
    eq rate[i in t]:d(x[i])/di==(u-x[i])/1{s};
    eq square[i in t]:y[i]==x[i]*x[i]; eq initial:x[0{s}]==0;
    annotation start x(0); annotation start y(0);
    let cost:Scalar=integral(i in t | (x[i]-1)*(x[i]-1)/1{s});
    annotation check cost(cost>=0);
    let miss[i in t]:Scalar=(x[i]-1.5)*(x[i]-1.5);
    annotation report miss("terminal miss");
  }
}"#;

fn shooting_solver() -> SolverProfile {
    SolverProfile {
        selection: SolverSelection::Explicit(Backend::Ipopt),
        controls: Controls {
            hessian: HessianMode::LimitedMemory,
            ..Default::default()
        },
        presolve: pse_backend_native::presolve::Policy::Off,
        ..Default::default()
    }
}

/// Exact normal equations for the two held controls: J = 4.25 - 2r'u + u'Au.
/// Obtained by integrating x=u1(1-exp(-t)), then x=x(1)exp(-s)+u2(1-exp(-s)).
fn tracking_quadratic() -> ([f64; 3], [f64; 2]) {
    let (e1, e2) = ((-1f64).exp(), (-2f64).exp());
    let c = 1. - e1;
    let square = 1. - 2. * c + (1. - e2) / 2.;
    let (ta, tb) = (c * e1, c);
    (
        [
            square + c * c * (1. - e2) / 2. + ta * ta,
            c * (c - (1. - e2) / 2.) + ta * tb,
            square + tb * tb,
        ],
        [e1 + c * c + 1.5 * ta, e1 + 1.5 * tb],
    )
}

fn tracking_objective(u: &[f64]) -> f64 {
    assert_eq!(u.len(), 2);
    let ([a, b, d], [r, s]) = tracking_quadratic();
    4.25 - 2. * (r * u[0] + s * u[1]) + a * u[0] * u[0] + 2. * b * u[0] * u[1] + d * u[1] * u[1]
}

#[tokio::test]
async fn public_single_and_multiple_shooting_match_analytic_objective_and_refuse_study_projection()
-> Result<()> {
    let context = NativeContext::new()?;
    let body = AssertUnwindSafe(async {
        let package = package(&context, TRACKING).await?;
        let root = declaration(&context, &package, "Tracking").await?;
        let services = context.runtime.execution_services();
        let accuracy =
            pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
        let simulation = Box::pin(package.prepare_simulation(
            services.preparation(),
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            Default::default(),
            compiler(),
            Profile {
                method: Method::Diffsol,
                end: 2.,
                samples: vec![0., 0.5, 1., 1.5, 2.],
                rtol: accuracy,
                atol: vec![accuracy; 2],
                out_rtol: Some(accuracy),
                out_atol: vec![accuracy],
                initial_step: 1e-4,
                parameter_scales: vec![1.],
                schedule: vec![ScheduledInput {
                    parameter: 0,
                    times: vec![1.],
                }],
                ..Default::default()
            },
            DerivativeOrder::First,
            &CancelSource::new(),
        ))
        .await?;
        let contract = simulation.contract();
        assert_eq!(contract.outputs.len(), 3);
        let state = contract
            .differential
            .iter()
            .position(|differential| *differential)
            .unwrap();
        let ([a, b, d], [r, s]) = tracking_quadratic();
        let determinant = a * d - b * b;
        let optimum = [(r * d - s * b) / determinant, (a * s - b * r) / determinant];
        assert!(optimum.iter().all(|u| *u > 0. && *u < 5.));
        let minimum = tracking_objective(&optimum);
        for (method, nodes) in [
            (ShootingMethod::Single, vec![]),
            (ShootingMethod::Multiple, vec![0.5, 1., 1.5]),
        ] {
            let problem = Arc::new(simulation.shooting(
                services.preparation(),
                ShootingProfile {
                    method,
                    nodes: nodes.clone(),
                    controls: vec![ShootingControl {
                        input: contract.parameters[0],
                        lower: Some(0.),
                        upper: Some(5.),
                    }],
                    path: vec![PathBound {
                        output: contract.outputs[state],
                        lower: None,
                        upper: Some(2.),
                    }],
                    objective: ShootingObjective {
                        terminal: BTreeMap::from([(contract.outputs[2], 1.)]),
                        integral: BTreeMap::from([(contract.quadratures[0], 1.)]),
                    },
                    evaluation: ShootingEvaluationPolicy::Demanded,
                    solver: shooting_solver(),
                },
            )?);
            let budget = allowance(
                problem.numerics(),
                NumericalTarget::Objective,
                SemanticId::NIL,
            );
            // Host-only preparation must not manufacture a durable scientific projection.
            let error = RuntimeOperationRequest::Shooting {
                problem: problem.clone(),
                initial: None,
            }
            .into_study()
            .unwrap_err();
            let diagnostic = error.boundary_diagnostic();
            assert_eq!(
                diagnostic.class,
                pse_model::diagnostic::BoundaryClass::Unsupported
            );
            assert_eq!(
                diagnostic.rule,
                pse_diagnostics::DiagnosticRule::StudyOperationUnsupported
            );
            // Explicit initial coordinates are checked before a native worker is issued.
            assert!(
                problem
                    .start_with_initial(&services, vec![f64::NAN])
                    .is_err()
            );
            // Supply independent controls and deliberately displaced inner nodes;
            // multiple shooting must close continuity instead of inheriting a rollout.
            let mut initial = vec![0.4, 1.2];
            initial.extend(nodes.iter().map(|_| 0.3));
            assert_eq!(initial.len(), problem.contract().variables.len());
            let result = problem
                .start_with_initial(&services, initial)?
                .wait()
                .await?;
            assert!(
                result.usable(),
                "{:?}; {:?}",
                result.assessments(),
                result.report()
            );
            let RunReport::Shooting(report) = result.report().unwrap() else {
                panic!("shooting report required")
            };
            assert_eq!(report.method, method);
            assert_eq!(
                report.solve.as_ref().unwrap().termination.category,
                pse_backend_native::solve::Termination::Success
            );
            assert!(report.quality.as_ref().unwrap().feasible());
            let rows = report.constraint_values.as_ref().unwrap();
            assert_eq!(rows.len(), problem.contract().rows.len());
            assert_eq!(rows.len(), nodes.len() + simulation.profile().samples.len());
            for (index, (value, id)) in rows.iter().zip(&problem.contract().rows).enumerate() {
                let row_budget = allowance(problem.numerics(), NumericalTarget::Row, *id);
                assert!(value.is_finite());
                if index < nodes.len() {
                    assert!(
                        value.abs() <= row_budget,
                        "unclosed continuity row {id}: {value}"
                    );
                } else {
                    assert!(*value <= 2. + row_budget, "violated path row {id}: {value}");
                }
            }
            let controls = &report.controls[&contract.parameters[0]];
            assert!(
                controls
                    .iter()
                    .all(|u| u.is_finite() && (0. ..=5.).contains(u))
            );
            // This is an empirical objective check, not a forward-control accuracy
            // guarantee inferred from stationarity or local integration tolerances.
            assert!((tracking_objective(controls) - minimum).abs() <= budget);
            assert_eq!(report.nodes.len(), nodes.len() + 1);
            assert!(report.checks_complete && !report.checks.is_empty());
            assert!(report.checks.iter().all(|check| check.satisfied));
            let trajectory = report.trajectory.as_ref().unwrap();
            assert_eq!(
                trajectory
                    .samples
                    .iter()
                    .map(|sample| sample.time)
                    .collect::<Vec<_>>(),
                simulation.profile().samples
            );
            let path_budget = allowance(
                simulation.numerics(),
                NumericalTarget::Variable,
                contract.states[state],
            );
            for sample in &trajectory.samples {
                assert!(sample.outputs[state] <= 2. + path_budget);
            }
            let last = trajectory.samples.last().unwrap();
            let terminal = (last.outputs[state] - 1.5).powi(2);
            assert!(
                (last.outputs[2] - terminal).abs() <= 16. * f64::EPSILON * terminal.abs().max(1.)
            );
            let published = last.integrals[0] + last.outputs[2];
            assert!(
                (report.objective.unwrap() - published).abs()
                    <= 16.
                        * f64::EPSILON
                        * (last.integrals[0].abs() + last.outputs[2].abs()).max(1.)
            );
            assert_eq!(
                result.table("runtime.solve_variables")?.batch().num_rows(),
                report.candidate.as_ref().unwrap().len()
            );
        }
        Ok(())
    })
    .catch_unwind()
    .await;
    settle(&context, body).await
}

const LOOP: &str = r#"package native_horizon {
  collocation radau alpha(1) beta(0) right(true);
  def Loop(horizon:Time=2{s}) {
    domain t:Time from 0{s} to horizon;
    when analysis.route==analysis.integrated {
      discretize grid on t using integrated(elements=1,order=1);
    }
    when analysis.route==analysis.simultaneous {
      discretize grid on t using radau(elements=8,order=1);
      let cost:Scalar=integral(i in t | (x[i]-r)*(x[i]-r)/1{s});
      annotation objective cost(minimize);
    }
    param x0:Scalar=0; param r:Scalar=1;
    var x[i in t]:Scalar; var u[i in t]:Scalar;
    eq rate[i in t]:d(x[i])/di==(2*u[i]-x[i])/1{s};
    eq initial:x[0{s}]==x0;
    annotation bounds u(0,1); annotation start x(0); annotation start u(0.5);
  }
  def Plant { child process:Loop=Loop(horizon=5{s}); }
}"#;

fn fixed(values: &[(&str, f64)], paths: &[&str]) -> ModelingCaseBindings {
    ModelingCaseBindings {
        values: values
            .iter()
            .map(|(path, value)| ((*path).into(), *value))
            .collect(),
        variables: paths
            .iter()
            .map(|path| {
                (
                    (*path).into(),
                    ModelingVariableState {
                        fixed: Some(true),
                        ..Default::default()
                    },
                )
            })
            .collect(),
        ..Default::default()
    }
}

async fn horizon_plant(
    context: &NativeContext,
    package: &ModelingPackage,
    steps: usize,
) -> Result<ModelingSimulation> {
    let root = declaration(context, package, "Plant").await?;
    let end = steps as f64 * 0.25;
    let accuracy = pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
    Ok(Box::pin(package.prepare_simulation(
        &context.runtime.preparation_services(),
        root,
        pse_modeling::specialize::root_instance(root),
        Bindings::default(),
        Limits::default(),
        fixed(
            &[("process.u[0{s}]", 0.5), ("process.x0", 0.)],
            &["process.u[0{s}]"],
        ),
        compiler(),
        Profile {
            method: Method::Diffsol,
            end,
            samples: vec![0., end],
            rtol: accuracy,
            atol: vec![accuracy],
            initial_step: 1e-4,
            parameter_scales: vec![1.; 3],
            ..Default::default()
        },
        DerivativeOrder::First,
        &CancelSource::new(),
    ))
    .await?)
}

#[tokio::test]
async fn public_horizon_applies_controller_moves_to_analytic_plant_and_refuses_duplicate_input()
-> Result<()> {
    let context = NativeContext::new()?;
    let body = AssertUnwindSafe(async {
        let package = package(&context, LOOP).await?;
        let root = declaration(&context, &package, "Loop").await?;
        let plant = horizon_plant(&context, &package, 3).await?;
        let actuator = plant.model().compiled().model.paths["process.u[0{s}]"];
        let output = plant.contract().outputs[0];
        let plant_budget = allowance(
            plant.numerics(),
            NumericalTarget::Variable,
            plant.contract().states[0],
        );
        let mut bindings = Bindings::default().with_analysis(Route::Simultaneous);
        bindings.demand = ["x0".into(), "r".into(), "x[0{s}]".into(), "u[0{s}]".into()]
            .into_iter()
            .chain((1..=8).flat_map(|index| {
                ["x", "u"].map(move |name| format!("{name}[{}{{s}}]", index as f64 * 0.25))
            }))
            .collect();
        let controller = HorizonController {
            package: package.clone(),
            analysis: ModelingAnalysis {
                root,
                instance: pse_modeling::specialize::root_instance(root),
                bindings,
                limits: Limits::default(),
                case: fixed(&[("x0", 0.), ("r", 1.), ("u[0{s}]", 0.)], &["u[0{s}]"]),
                order: DerivativeOrder::Second,
                compiler: compiler(),
                solver: SolverProfile {
                    intent: SolveIntent::Optimize,
                    selection: SolverSelection::Explicit(Backend::Ipopt),
                    controls: Controls {
                        reuse: ReusePolicy::AllowRebuild,
                        start: StartPolicy::PreviousAccepted,
                        ..Default::default()
                    },
                    presolve: pse_backend_native::presolve::Policy::Off,
                    ..Default::default()
                },
                numerical: Default::default(),
            },
            bindings: vec![
                ("x0".into(), HorizonSignal::Measured(output)),
                ("r".into(), HorizonSignal::Trajectory(vec![1., 1., 0.6])),
                ("u[0{s}]".into(), HorizonSignal::Applied(0)),
            ],
            moves: vec![("u[0.25{s}]".into(), 0)],
            advanced: None,
        };
        let horizon = Horizon {
            output: StudyOutputDemand::Full {},
            plant,
            period: 0.25,
            steps: 3,
            inputs: vec![HorizonInput {
                parameter: actuator,
                initial: 0.,
            }],
            estimator: None,
            controller: Some(controller),
        };
        let mut invalid = horizon.clone();
        invalid.inputs.push(invalid.inputs[0].clone());
        let error = Box::pin(context.runtime.start_horizon(invalid, &CancelSource::new()))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("driven once"), "{error}");
        let result = Box::pin(context.runtime.start_horizon(horizon, &CancelSource::new()))
            .await?
            .wait()
            .await?;
        assert!(
            result.usable(),
            "{:?}; {:?}",
            result.assessments(),
            result.report()
        );
        let report = result.horizon().unwrap();
        assert_eq!(report.completed_steps, 3);
        assert_eq!(report.steps.len(), 3);
        assert_eq!(report.outputs, [output]);
        assert_eq!(report.inputs, [actuator]);
        let RunReport::Modeling(solves) = result.report().unwrap() else {
            panic!("controller reports required")
        };
        assert_eq!(solves.len(), 3);
        let mut expected = 0.;
        for (index, (step, solve)) in report.steps.iter().zip(solves).enumerate() {
            assert_eq!(step.time, index as f64 * 0.25);
            assert_eq!(step.decision, HorizonDecision::Solved);
            assert_eq!(step.controller, Some(index));
            assert!((step.measured[0] - expected).abs() <= plant_budget);
            if index != 0 {
                assert_eq!(step.measured, report.steps[index - 1].reached);
            }
            let paths = &solve.prepared.model.model.compiled().model.paths;
            let budget = allowance(
                solve.prepared.solve.numerics(),
                NumericalTarget::Variable,
                paths["u[0.25{s}]"],
            );
            let u = step.applied[0];
            assert!((-budget..=1. + budget).contains(&u));
            assert_eq!(u, solve.values.scalars[&paths["u[0.25{s}]"]]);
            let Outcome::Native(native) = &solve.outcome else {
                panic!("native controller required")
            };
            assert!(native.quality.as_ref().unwrap().feasible());
            assert_eq!(
                native.qualification,
                pse_backend_native::solve::Qualification::Stationary
            );
            // Independent exact plant response under this period's actually applied hold.
            expected = 2. * u + (step.measured[0] - 2. * u) * (-0.25f64).exp();
            assert!((step.reached[0] - expected).abs() <= plant_budget);
        }
        Ok(())
    })
    .catch_unwind()
    .await;
    settle(&context, body).await
}

const ACCUMULATION: &str = r#"package accumulation {
  def Experiment {
    domain t:Time from 0{s} to 1{s};
    discretize grid on t using integrated(elements=1,order=1);
    param rate:Scalar=3; var total[i in t]:Time;
    eq balance[i in t]:d(total[i])/di==rate;
    eq initial:total[0{s}]==2{s};
    let observed[i in t]:Time=total[i];
    annotation report observed("measurement");
    annotation check total(total[i]>=2{s});
  }
  test experiment_integrated fixture {
    dof 0; route integrated; procedure integrate;
    integrate samples(0{s},0.5{s},1{s}) relative(global)
      normalized_absolute(global) step(1e-4{s});
  } { child root:Experiment=Experiment(); }
  entity kind origin provenance {attribute title:Text;}
  entity origin experiment {title="analytic total(t)=2 s+3*t"}
  enum role {measured facets(measured)}
  entity kind reading {attribute value:Time?;attribute sigma:Time?;}
  @id("68686868686868686868686868686868")
  entity reading measured provenance(experiment,role.measured) {value=5{s},sigma=1{s}}
}"#;

#[tokio::test]
async fn public_integrated_fit_preserves_analytic_rate_response_and_observation_provenance()
-> Result<()> {
    let context = NativeContext::new()?;
    let body = AssertUnwindSafe(async {
        let package = package(&context, ACCUMULATION).await?;
        let services = context.runtime.execution_services();
        let case = declaration(&context, &package, "experiment_integrated").await?;
        let fraction =
            pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
        let integration = Profile {
            method: Method::Diffsol,
            start: 0.,
            end: 1.,
            samples: vec![0., 0.5, 1.],
            rtol: fraction,
            atol: vec![fraction],
            initial_step: 1e-4,
            parameter_scales: vec![1.],
            sensitivity: DynamicSensitivity::Forward,
            ..Default::default()
        };
        let simulation = Box::pin(package.declared_simulation(
            services.preparation(),
            case,
            compiler(),
            Some(integration.clone()),
            Limits::default(),
            &CancelSource::new(),
        ))
        .await?;
        assert_eq!(simulation.contract().states.len(), 1);
        let state_budget = allowance(
            simulation.numerics(),
            NumericalTarget::Variable,
            simulation.contract().states[0],
        );
        let handle = simulation.start(&services)?;
        let joined = Box::pin(handle.wait()).await?;
        assert_eq!(Box::pin(handle.wait()).await?.run_id, joined.run_id);
        assert!(joined.usable(), "{:?}", joined.report());
        let RunReport::Simulation(trajectory) = joined.report().unwrap() else {
            panic!("original standalone trajectory required")
        };
        assert_eq!(trajectory.report().termination, Termination::Completed);
        assert!(trajectory.accepted() && trajectory.checks_complete());
        let standalone =
            simulation_samples::Row::rows(&joined.table("runtime.simulation_samples")?)?;
        assert_eq!(
            standalone.len(),
            6,
            "state and authored observation at all three samples"
        );
        for row in &standalone {
            assert!([0., 0.5, 1.].contains(&row.time));
            assert!((row.value - (2. + 3. * row.time)).abs() <= state_budget);
        }

        let identity = |byte| SemanticId::from_bytes([byte; 16]);
        let fit_id = FitId::from_id(identity(101));
        let parameter_id = identity(102);
        let experiment_id = InstanceId::from_id(identity(103));
        let observation_id = DeclarationId::from_id(identity(104));
        let mut declarations = FitDeclarations::default();
        declarations.fits.push(authored_fit::Row {
            fit_id,
            parameters: vec![authored_fit::AuthoredFitCasesFieldParametersItem {
                symbol_id: parameter_id,
                fixed: false,
                value: 1.,
                lower: Some(0.),
                upper: Some(10.),
                scale: 1.,
            }],
            experiments: vec![authored_fit::AuthoredFitCasesFieldExperimentsItem {
                experiment_id,
                case_id: case,
                route: ModelingAnalysisRoute::Integrated,
                bindings: vec![
                    authored_fit::AuthoredFitCasesFieldExperimentsItemBindingsItem {
                        parameter_id,
                        path: "root.rate".into(),
                    },
                ],
            }],
            observations: vec![authored_fit::AuthoredFitCasesFieldObservationsItem {
                observation_id,
                value_attribute: "value".into(),
                standard_deviation_attribute: Some("sigma".into()),
                experiment_id,
                output_path: "root.observed[0{s}]".into(),
                time: Some(1.),
                time_basis: None,
                time_unit_id: None,
                included: true,
                importance: 1.,
            }],
        });
        let package = Box::pin(
            package.with_fit_declarations(&context.runtime.source_services(), declarations),
        )
        .await?;
        let prepared = Box::pin(package.prepare_fit(
            services.preparation(),
            fit_id,
            FitProfile {
                optimizer: FitOptimizer::NativeNlp,
                solver: SolverProfile {
                    intent: SolveIntent::Optimize,
                    selection: SolverSelection::Explicit(Backend::Ipopt),
                    controls: Controls {
                        hessian: HessianMode::LimitedMemory,
                        ..Default::default()
                    },
                    presolve: pse_backend_native::presolve::Policy::Off,
                    ..Default::default()
                },
                // The experiment's instance owns the same explicit controls; standalone
                // coordinate identities are not copied into this native profile.
                simulations: BTreeMap::from([(experiment_id, integration)]),
                rank_tolerance: 1e-8,
                max_cells: 10_000,
                derivatives: FitDerivatives::Responses,
                experiment_workers: PositiveCount::try_new(1)?,
                uncertainty: None,
            },
            compiler(),
            Limits::default(),
            &CancelSource::new(),
        ))
        .await?;
        let parameter_budget =
            allowance(prepared.numerics(), NumericalTarget::Variable, parameter_id);
        let target = prepared
            .numerics()
            .targets
            .iter()
            .find(|target| target.kind == NumericalTarget::Variable && target.id == parameter_id)
            .unwrap();
        let parameter_scale = target.coordinate_scale;
        assert_eq!(parameter_scale, 1.);
        let result = prepared.start(&services)?.wait().await?;
        assert!(
            result.usable(),
            "{:?}; {:?}",
            result.assessments(),
            result.report()
        );
        let RunReport::Fit(report) = result.report().unwrap() else {
            panic!("original transient fit required")
        };
        assert_eq!(report.derivatives, FitDerivatives::Responses);
        assert_eq!(report.hessian, HessianMode::LimitedMemory);
        assert!(report.estimate_qualified());
        assert!(report.checks_complete && !report.checks.is_empty());
        assert!(report.checks.iter().all(|check| check.satisfied));
        assert!(report.validation_error.is_none());
        let parameters = fit_parameters::Row::rows(&result.table("runtime.fit_parameters")?)?;
        let [parameter] = parameters.as_slice() else {
            panic!("one fitted rate")
        };
        assert_eq!(parameter.parameter_id, parameter_id);
        assert_eq!(parameter.scale, parameter_scale);
        assert!(!parameter.fixed);
        let fitted = parameter.value.unwrap();
        // The analytic observation is 5 s at 1 s from 2 s: its unique fitted rate is 3.
        assert!(
            (fitted - 3.).abs() <= parameter_budget,
            "rate={fitted}; admitted budget={parameter_budget}"
        );
        let policies = resolved_numerics::Row::rows(&result.table("runtime.resolved_numerics")?)?;
        let policy = policies
            .iter()
            .find(|row| {
                row.target_kind == NumericalTarget::Variable && row.target_id == parameter_id
            })
            .unwrap();
        assert_eq!(policy.budget, parameter_budget);
        assert_eq!(policy.coordinate_scale, parameter_scale);

        let responses =
            response_sensitivities::Row::rows(&result.table("runtime.response_sensitivities")?)?;
        let [response] = responses.as_slice() else {
            panic!("one original response")
        };
        let observations = fit_observations::Row::rows(&result.table("runtime.fit_observations")?)?;
        let [observation] = observations.as_slice() else {
            panic!("one original observation")
        };
        assert_eq!(response.run_id, result.run_id);
        assert_eq!(response.experiment_id, experiment_id);
        assert_eq!(response.parameter_id, parameter_id);
        assert_eq!(response.parameter_column, 0);
        assert_eq!(response.sample, 0);
        assert_eq!(response.time, Some(1.));
        assert_eq!(response.parameter_unit_id, parameter.unit_id);
        assert_eq!(observation.observation_id, observation_id.as_id());
        assert_eq!(observation.run_id, response.run_id);
        assert_eq!(observation.experiment_id, response.experiment_id);
        assert!(observation.included);
        assert_eq!(response.output_unit_id, observation.unit_id);
        assert!((observation.prediction.unwrap() - (2. + fitted)).abs() <= state_budget);

        // The published response is weighted and parameter-scaled. Here sigma=1 s,
        // importance=1 and the scale=1, so d(total)/d(rate)=t. A material half-scale
        // displacement checks its physical effect against the actual state allowance;
        // local integration controls alone do not certify derivative error.
        let delta = 0.5 * parameter_scale;
        assert!(delta > parameter_budget && delta * response.time.unwrap() > state_budget);
        assert!(fitted + delta <= 10.);
        assert!((response.value * delta - delta * response.time.unwrap()).abs() <= state_budget);
        let runs = computation_runs::Row::rows(&result.table("runtime.computation_runs")?)?;
        let [run] = runs.as_slice() else {
            panic!("one original computation")
        };
        assert!(matches!(
            run.termination,
            Some(NativeTermination::Success | NativeTermination::Acceptable)
        ));
        assert_eq!(run.backend, Some(NativeBackend::Ipopt));
        assert!(run.error.is_none());
        assert_eq!(run.estimate_qualified, Some(true));
        assert_eq!(result.completion().unwrap().computation.as_ref(), Some(run));
        let exported = fitted_parameter_cells::Row::rows(&result.export_fit_parameters()?)?;
        let [exported] = exported.as_slice() else {
            panic!("one qualified parameter export")
        };
        assert_eq!(exported.run_id, result.run_id);
        assert_eq!(exported.fit_id, fit_id);
        assert_eq!(exported.parameter_id, parameter_id);
        assert_eq!(exported.value.to_bits(), fitted.to_bits());
        assert_eq!(exported.unit_id, parameter.unit_id);
        assert_eq!(
            exported.source_revision,
            pse_runtime::workflow::OperationSource::of(&package)
                .revision
                .as_id()
        );
        let checks = modeling_checks::Row::rows(&result.table("runtime.modeling_checks")?)?;
        assert!(!checks.is_empty() && checks.iter().all(|check| check.satisfied));
        Ok(())
    })
    .catch_unwind()
    .await;
    settle(&context, body).await
}
