// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original-clock, schedule and event behavior through authored fit experiments.
use super::*;
use crate::workflow::tests::{compiler_profile, id};
use native::{
    NlpOracle,
    solve::{Backend, Execution},
};
use std::sync::atomic::AtomicBool;

/// These analytical estimates and active-bound checks require a more accurate
/// base point than the ordinary 0.1% engineering stopping budgets. Derivatives
/// are still checked at their stated evaluation points, and covariance is still
/// derived at the actual returned candidate; no output-error guarantee is assumed.
fn verification_numerics() -> pse_model::numerics::NumericalPolicy {
    pse_model::numerics::NumericalPolicy {
        kkt: pse_model::numerics::KktTolerances {
            stationarity: 1e-9,
            complementarity: 1e-9,
        },
        ..Default::default()
    }
}

fn assert_verification_accuracy(problem: &FitProblem) {
    assert_eq!(problem.accuracy.stationarity, 1e-9);
    assert_eq!(problem.accuracy.complementarity, 1e-9);
}

fn assert_diffsol_exact_hessian_refused(error: WorkflowError) {
    let WorkflowError::Math(crate::math::MathRuntimeError::Solve(
        ProblemError::DynamicRouteRefused(decision),
    )) = &error
    else {
        panic!("expected the typed dynamic route refusal: {error:?}")
    };
    assert_eq!(decision.requested, native::dynamics::Method::Diffsol);
    assert_eq!(decision.selected, None);
    assert_eq!(decision.candidates.len(), 1);
    let candidate = &decision.candidates[0];
    assert_eq!(candidate.method, native::dynamics::Method::Diffsol);
    assert!(candidate.causes.iter().any(|cause| matches!(
        cause.as_ref(),
        ProblemError::Unsupported(reason) if reason.contains("exact transient Hessians need IDAS")
    )), "{decision:?}");
}
async fn source(mixed: bool, expected: f64) -> (crate::workflow::ModelingPackage, FitProfile) {
    source_case(mixed, expected, "Dynamic").await
}
/// The fit over the transient experiment `case`: `Dynamic`, or `DynamicReset`, whose
/// fixture declares a reset event between two modes (ADR-0119 Outcome 3).
async fn source_case(
    mixed: bool,
    expected: f64,
    case: &str,
) -> (crate::workflow::ModelingPackage, FitProfile) {
    let mut physical = crate::workflow::tests::physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    let body = "{ domain t: Time from 160{s} to 161{s}; discretize grid on t using integrated(elements=1,order=1); param p: Scalar = 2; var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == p; eq initial: x[160{s}] == 10{s}; let y[i in t]: Time = x[i]+i-100{s}; let hit[i in t]: Time = x[i]-11{s}; let jump[i in t]: Time = 20{s}; annotation report y(\"measurement\"); annotation check x(x[i] >= 10{s}); }";
    let integrate =
        "integrate samples(160{s},161{s}) relative(1e-8) normalized_absolute(1e-8) step(1e-5{s});";
    let text = format!(
        "package p {{ test Dynamic fixture {{dof 0; route integrated; procedure integrate; {integrate}}} {body} test DynamicReset fixture {{dof 0; route integrated; procedure integrate; {integrate} mode before; event hit[160{{s}}] direction(either) tolerance(1e-8{{s}}) reset(x[160{{s}}] = jump[160{{s}}]) next(after); mode after;}} {body} def Steady {{ param p: Scalar = 2; let y: Scalar = p; annotation check p(p > 0); }} }}"
    );
    let mut rows = pse_authoring::language::parse(
        &text,
        id(20),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    rows.extend(measured_rows(&[
        (id(71), "Time", Some(expected), Some(1.)),
        (id(72), "Scalar", Some(1.), Some(1.)),
    ]));
    let root = |name| rows.iter().find(|r| r.name == name).unwrap().declaration_id;
    let mut data = FitDeclarations::default();
    let mut fit:FitDeclaration=serde_json::from_value(serde_json::json!({"fit_id":id(73),"parameters":[{"symbol_id":id(3),"fixed":false,"value":2.,"lower":0.1,"upper":10.,"scale":1.}],"experiments":[{"experiment_id":id(74),"case_id":root(case),"route":"integrated","bindings":[{"parameter_id":id(3),"path":"p"}]}],"observations":[{"value_attribute":"value","standard_deviation_attribute":"sigma","observation_id":id(71),"experiment_id":id(74),"output_path":"y[160{s}]","time":161.,"time_basis":"model_clock","included":true,"importance":1.}]})).unwrap();
    if mixed {
        fit.experiments.push(serde_json::from_value(serde_json::json!({"experiment_id":id(75),"case_id":root("Steady"),"route":"steady","bindings":[{"parameter_id":id(3),"path":"p"}]})).unwrap());
        fit.observations.push(serde_json::from_value(serde_json::json!({"value_attribute":"value","standard_deviation_attribute":"sigma","observation_id":id(72),"experiment_id":id(75),"output_path":"y","time":null,"included":true,"importance":1.})).unwrap());
    }
    data.fits.push(fit);
    let profile = FitProfile {
        solver: SolverProfile {
            intent: SolveIntent::Optimize,
            selection: native::solve::SolverSelection::Explicit(Backend::Ipopt),
            controls: native::solve::Controls {
                hessian: HessianMode::LimitedMemory,
                ..Default::default()
            },
            presolve: native::presolve::Policy::Off,
            numerics: verification_numerics(),
            convexity: Default::default(),
            backend: native::execution::BackendSettings::Default,
            sensitivity: None,
            composition: Default::default(),
            reconstruction: None,
        },
        simulations: BTreeMap::from([(
            InstanceId::from(id(74)),
            native::dynamics::Profile {
                start: 160.,
                end: 161.,
                samples: vec![160., 161.],
                parameter_scales: vec![1.],
                ..Default::default()
            },
        )]),
        rank_tolerance: 1e-8,
        max_cells: 100000,
        derivatives: FitDerivatives::Responses,
        uncertainty: None,
    };
    let runtime = crate::workflow::tests::runtime_with_workspace(32 << 20);
    (
        runtime
            .modeling_package(rows, physical)
            .await
            .unwrap()
            .with_fit_declarations(data)
            .await
            .unwrap(),
        profile,
    )
}
#[tokio::test]
async fn mixed_shared_parameter_gradient_uses_inline_forward_sensitivities() {
    let (package, profile) = source(true, 73.).await;
    let cancel = crate::CancelSource::new();
    let mut exact = profile.clone();
    exact.solver.controls.hessian = HessianMode::Exact;
    for simulation in exact.simulations.values_mut() {
        simulation.method = native::dynamics::Method::Diffsol;
    }
    assert_diffsol_exact_hessian_refused(
        package
            .prepare_fit_problem(
                FitId::from(id(73)),
                exact,
                compiler_profile(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap_err(),
    );
    let (problem, _) = package
        .prepare_fit_problem(
            FitId::from(id(73)),
            profile,
            compiler_profile(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(problem.contract.variables.len(), 1);
    let execution = Execution::new(
        Arc::new(AtomicBool::new(false)),
        &problem.profile.solver.controls,
    );
    let mut oracle = FitOracle::new(problem, execution).unwrap();
    assert!((oracle.objective(&[2.]).unwrap() - 0.5).abs() < 1e-6);
    let mut gradient = [0.];
    oracle.gradient(&[2.], &mut gradient).unwrap();
    assert!((gradient[0] - 1.).abs() < 1e-6);
    let step = 1e-4;
    let finite = (oracle.objective(&[2. + step]).unwrap()
        - oracle.objective(&[2. - step]).unwrap())
        / (2. * step);
    assert!((finite - gradient[0]).abs() < 1e-5);
    let RankDiagnostic {
        responses, rank, ..
    } = oracle.response_rank(&[2.]).unwrap();
    assert_eq!(rank, 1);
    assert!((responses[(0, 0)] - 1.).abs() < 1e-6);
    assert!((responses[(1, 0)] - 1.).abs() < 1e-6);
}
/// I8: a Gauss–Newton Hessian needs only forward sensitivities, so a transient fit on
/// Diffsol admits it, where the exact Hessian (the second-order adjoint, IDAS only, Y4b)
/// is refused. The Hessian is the response Gram, the solve converges on it, and the
/// result records its source (PS-07).
#[tokio::test]
async fn gauss_newton_fit_admits_transient() {
    let (package, mut profile) = source(true, 73.).await;
    let cancel = crate::CancelSource::new();
    profile.solver.controls.hessian = HessianMode::GaussNewton;
    let (problem, _) = package
        .prepare_fit_problem(
            FitId::from(id(73)),
            profile.clone(),
            compiler_profile(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(problem.contract.derivatives, DerivativeOrder::Second);
    assert_verification_accuracy(&problem);
    let execution = Execution::new(
        Arc::new(AtomicBool::new(false)),
        &problem.profile.solver.controls,
    );
    let mut oracle = FitOracle::new(problem, execution).unwrap();
    let mut hessian = vec![0.; oracle.hessian_pattern().unwrap().row_idx().len()];
    for sigma in [1., 0.25] {
        oracle.hessian(&[2.], sigma, &[], &mut hessian).unwrap();
        // Both responses are dy/dp = 1 with unit weights: σ·JᵀWJ = 2σ.
        assert_eq!(hessian.len(), 1);
        assert!((hessian[0] - 2. * sigma).abs() < 1e-6, "{hessian:?}");
    }
    #[cfg(feature = "solver-ipopt")]
    {
        let prepared = package
            .prepare_fit(
                FitId::from(id(73)),
                profile,
                compiler_profile(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_verification_accuracy(&prepared.problem);
        let result = prepared.start().unwrap().wait().await.unwrap();
        let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
            panic!("missing fit")
        };
        // Least squares of (p − 2) and (p − 1).
        assert!(
            (report.candidate.as_ref().unwrap()[0] - 1.5).abs() < 1e-5,
            "{report:?}"
        );
        assert_eq!(report.hessian, HessianMode::GaussNewton);
        let table = result.table("runtime.solve_metrics").unwrap();
        let rows = pse_relations::generated::runtime::solve_metrics::RuntimeSolveMetricsView::from_checked(&table)
            .unwrap()
            .rows()
            .unwrap();
        assert!(rows.iter().any(|r| r.namespace == "derivatives"
            && r.name == "hessian"
            && r.text.as_deref() == Some("gauss_newton")));
    }
}
/// ADR-0110 item 3: a gradient-only fit's objective gradient is the adjoint product of its
/// transient experiment, and it equals the forward-sensitivity gradient and central finite
/// differences of the objective, on Diffsol and IDAS, with and without a scheduled input
/// (declared tolerances: 1e-6 against forward, 1e-5 against differences). The gradient fit
/// integrates without sensitivities, needs the limited-memory Hessian, and reruns the
/// forward sensitivities once for rank at its candidate (PS-12).
#[tokio::test]
async fn adjoint_gradient_equals_forward_on_transient_fit() {
    let mut methods = vec![native::dynamics::Method::Diffsol];
    #[cfg(feature = "solver-idas")]
    methods.push(native::dynamics::Method::Idas);
    for (method, scheduled) in methods.into_iter().flat_map(|m| [(m, false), (m, true)]) {
        let (package, mut profile) = source(true, 73.).await;
        profile
            .simulations
            .get_mut(&InstanceId::from(id(74)))
            .unwrap()
            .method = method;
        if scheduled {
            profile
                .simulations
                .get_mut(&InstanceId::from(id(74)))
                .unwrap()
                .schedule
                .push(native::dynamics::ScheduledInput {
                    parameter: 0,
                    times: vec![160.5],
                });
        }
        let cancel = crate::CancelSource::new();
        let oracle = |derivatives| {
            let (package, mut profile) = (package.clone(), profile.clone());
            let cancel = cancel.clone();
            async move {
                profile.derivatives = derivatives;
                let (problem, _) = package
                    .prepare_fit_problem(
                        FitId::from(id(73)),
                        profile,
                        compiler_profile(),
                        Default::default(),
                        &cancel,
                    )
                    .await
                    .unwrap();
                let mut execution = Execution::new(
                    Arc::new(AtomicBool::new(false)),
                    &problem.profile.solver.controls,
                );
                execution.memory = Some(64 << 20);
                FitOracle::new(problem, execution).unwrap()
            }
        };
        let mut forward = oracle(FitDerivatives::Responses).await;
        let mut adjoint = oracle(FitDerivatives::Gradient).await;
        let transient = |o: &FitOracle| {
            let Experiment::Transient(s) = &o.prepared.experiments[0] else {
                panic!("transient experiment")
            };
            s.profile.sensitivity
        };
        assert_eq!(
            transient(&forward),
            native::dynamics::DynamicSensitivity::Forward
        );
        assert_eq!(
            transient(&adjoint),
            native::dynamics::DynamicSensitivity::Adjoint
        );
        for x in [2.5, 0.7] {
            let (mut g, mut a) = ([0.], [0.]);
            forward.gradient(&[x], &mut g).unwrap();
            adjoint.gradient(&[x], &mut a).unwrap();
            assert!(g[0].abs() > 0.1, "a nonzero gradient at {x}: {g:?}");
            assert!(
                (a[0] - g[0]).abs() <= 1e-6 * (1. + g[0].abs()),
                "{method:?} scheduled={scheduled} x={x}: adjoint {a:?} forward {g:?}"
            );
            let step = 1e-4;
            let difference = (adjoint.objective(&[x + step]).unwrap()
                - adjoint.objective(&[x - step]).unwrap())
                / (2. * step);
            assert!(
                (a[0] - difference).abs() <= 1e-5 * (1. + difference.abs()),
                "{method:?} scheduled={scheduled} x={x}: adjoint {a:?} differences {difference}"
            );
            // The gradient fit's own integrations carry no sensitivities.
            let point = adjoint.evaluate(&[x]).unwrap();
            assert!(
                point
                    .trajectories
                    .values()
                    .flat_map(|r| &r.samples)
                    .all(|s| s.output_sensitivities.is_empty())
            );
        }
        // The rank rerun forms the response Jacobian with forward sensitivities.
        let RankDiagnostic {
            responses, rank, ..
        } = adjoint.response_rank(&[2.]).unwrap();
        assert_eq!(rank, 1);
        let expected = if scheduled { 0.5 } else { 1. };
        assert!((responses[(0, 0)] - expected).abs() < 1e-5, "{responses:?}");
        // No response Jacobian means no supplied Hessian.
        let mut gauss_newton = profile.clone();
        gauss_newton.derivatives = FitDerivatives::Gradient;
        gauss_newton.solver.controls.hessian = HessianMode::GaussNewton;
        assert!(
            package
                .prepare_fit_problem(
                    FitId::from(id(73)),
                    gauss_newton,
                    compiler_profile(),
                    Default::default(),
                    &cancel,
                )
                .await
                .is_err()
        );
    }
    #[cfg(feature = "solver-ipopt")]
    {
        // The complete gradient-only fit reaches the forward fit's estimate, records its
        // derivative source and qualifies its rank from the rerun.
        let (package, mut profile) = source(true, 73.).await;
        profile.derivatives = FitDerivatives::Gradient;
        let cancel = crate::CancelSource::new();
        let prepared = package
            .prepare_fit(
                FitId::from(id(73)),
                profile,
                compiler_profile(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_verification_accuracy(&prepared.problem);
        let result = prepared.start().unwrap().wait().await.unwrap();
        let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
            panic!("missing fit")
        };
        // Least squares of (p − 2) and (p − 1).
        assert!(
            (report.candidate.as_ref().unwrap()[0] - 1.5).abs() < 1e-5,
            "{report:?}"
        );
        assert_eq!(report.derivatives, FitDerivatives::Gradient);
        assert_eq!(report.rank, Some(1));
        let table = result.table("runtime.solve_metrics").unwrap();
        let rows = pse_relations::generated::runtime::solve_metrics::RuntimeSolveMetricsView::from_checked(&table)
            .unwrap()
            .rows()
            .unwrap();
        assert!(rows.iter().any(|r| r.namespace == "derivatives"
            && r.name == "gradient"
            && r.text.as_deref() == Some("gradient")));
    }
}
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn nonzero_clock_smooth_scheduled_and_state_reset_fits_share_response_contract() {
    use pse_relations::columnar::RelationRow;

    for mode in 0..3 {
        let expected = match mode {
            0 => 74.,
            // p is a scheduled input from 160.5 s; its second interval keeps the model's
            // value 2, so y = 72 + p/2 (I6).
            1 => 73.5,
            _ => 83.,
        };
        let (package, mut profile) = if mode == 2 {
            source_case(false, expected, "DynamicReset").await
        } else {
            source(false, expected).await
        };
        if mode == 1 {
            profile
                .simulations
                .get_mut(&InstanceId::from(id(74)))
                .unwrap()
                .schedule
                .push(native::dynamics::ScheduledInput {
                    parameter: 0,
                    times: vec![160.5],
                });
        }
        let cancel = crate::CancelSource::new();
        let (problem, _) = package
            .prepare_fit_problem(
                FitId::from(id(73)),
                profile.clone(),
                compiler_profile(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(problem.measurements[0].time, Some(161.));
        assert_eq!(problem.measurements[0].sample_index, Some(1));
        let execution = Execution::new(
            Arc::new(AtomicBool::new(false)),
            &problem.profile.solver.controls,
        );
        let mut oracle = FitOracle::new(problem, execution).unwrap();
        let RankDiagnostic {
            responses, rank, ..
        } = oracle.response_rank(&[2.]).unwrap();
        assert_eq!(rank, 1);
        assert!(
            (responses[(0, 0)] - if mode == 1 { 0.5 } else { 1. }).abs() < 1e-5,
            "mode {mode}"
        );
        drop(oracle);
        let prepared = package
            .prepare_fit(
                FitId::from(id(73)),
                profile,
                compiler_profile(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_verification_accuracy(&prepared.problem);
        let result = prepared.start().unwrap().wait().await.unwrap();
        let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
            panic!("missing fit")
        };
        assert!(
            (report.candidate.as_ref().unwrap()[0] - 3.).abs() < 1e-4,
            "mode {mode}: {report:?}"
        );
        let trajectory = &report.trajectories[&InstanceId::from(id(74))];
        assert_eq!(trajectory.completed_time, 161.);
        assert!((report.predictions[0].unwrap() - expected).abs() < 1e-4);
        if mode == 2 {
            assert!(!trajectory.events.is_empty());
        }
        assert!(
            report.checks_complete && report.checks.iter().all(|c| c.satisfied),
            "{report:?}"
        );
        assert!(report.estimate_qualified(), "mode {mode}: {report:?}");
        let exported = pse_relations::generated::runtime::fitted_parameter_cells::Row::rows(
            &result.export_fit_parameters().unwrap(),
        )
        .unwrap();
        assert_eq!(exported.len(), 1);
        assert_eq!(exported[0].run_id, result.run_id);
        assert_eq!(exported[0].fit_id, FitId::from(id(73)));
        assert_eq!(
            exported[0].source_revision,
            package.revision.identity().as_id()
        );
        assert!((exported[0].value - 3.).abs() < 1e-4);
        assert!(result.usable());
        assert!(
            result
                .table("runtime.modeling_checks")
                .unwrap()
                .batch()
                .num_rows()
                >= 2
        );
    }
}

#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn authored_integration_controls_bind_to_the_experiment_instance() {
    let (package, mut profile) = source(false, 74.).await;
    profile.simulations.clear();
    let prepared = package
        .prepare_fit(
            FitId::from(id(73)),
            profile,
            compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_verification_accuracy(&prepared.problem);
    let result = prepared.start().unwrap().wait().await.unwrap();
    let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
        panic!("missing fit")
    };
    assert!(
        (report.candidate.as_ref().unwrap()[0] - 3.).abs() < 1e-4,
        "{report:?}"
    );
    assert!(result.usable(), "{report:?}");
}

#[tokio::test]
async fn transient_fit_deadline_is_time_limit() {
    let (package, mut profile) = source(false, 74.).await;
    profile
        .simulations
        .get_mut(&InstanceId::from(id(74)))
        .unwrap()
        .time_limit = std::time::Duration::from_nanos(1);
    let cancel = crate::CancelSource::new();
    let (problem, _) = package
        .prepare_fit_problem(
            FitId::from(id(73)),
            profile,
            compiler_profile(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    let problem = Arc::new(problem);
    let execution = Execution::new(
        Arc::new(AtomicBool::new(false)),
        &problem.profile.solver.controls,
    );
    let mut oracle = FitOracle::new(problem.clone(), execution.clone()).unwrap();
    // The Diffsol deadline stays a typed time limit, never an evaluation failure.
    let error = oracle.objective(&[2.]).unwrap_err();
    assert!(
        matches!(
            error,
            ProblemError::Limit {
                kind: native::LimitKind::Time,
                ..
            }
        ),
        "{error:?}"
    );
    assert_eq!(
        native::callback::classify(&error),
        native::callback::Failure::Stopped(native::solve::Termination::TimeLimit)
    );
    let mut state = native::callback::CallbackState::new(execution);
    assert!(
        state
            .evaluate("fit.objective", || oracle.objective(&[2.]))
            .is_none()
    );
    assert_eq!(
        state.terminal.as_ref().map(|t| t.0),
        Some(native::solve::Termination::TimeLimit)
    );
    assert_eq!(
        crate::workflow::diagnostics::observed(&error, pse_diagnostics::DiagnosticStage::Fit).class,
        pse_model::diagnostic::BoundaryClass::ResourceLimit
    );
    // Cancellation of the attempt is a cancellation.
    let cancelled = Execution::new(
        Arc::new(AtomicBool::new(true)),
        &problem.profile.solver.controls,
    );
    let mut oracle = FitOracle::new(problem, cancelled).unwrap();
    let error = oracle.objective(&[2.]).unwrap_err();
    assert!(matches!(error, ProblemError::Cancelled), "{error:?}");
    assert_eq!(
        native::callback::classify(&error),
        native::callback::Failure::Stopped(native::solve::Termination::Cancelled)
    );
}
/// T11 (ADR-0118 items 3 and 8): a transient fit solved with the limited-memory or the
/// Gauss–Newton Hessian takes its covariance by Gauss–Newton from its forward-sensitivity
/// responses (only an exact-Hessian fit reads the exact one from its KKT analysis, I4),
/// and publishes it with that label. `y = 71 + p` observed once with σ = 1 gives `Σ = 1`.
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn gauss_newton_covariance_labelled() {
    use pse_relations::{
        columnar::RelationRow,
        generated::{
            enums::{CovarianceApproximation, DerivedQuantity},
            runtime::{local_validity, parameter_covariances},
        },
    };
    for hessian in [HessianMode::LimitedMemory, HessianMode::GaussNewton] {
        let (package, mut profile) = source(false, 73.).await;
        profile.solver.controls.hessian = hessian;
        let prepared = package
            .prepare_fit(
                FitId::from(id(73)),
                profile,
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_verification_accuracy(&prepared.problem);
        let result = prepared.start().unwrap().wait().await.unwrap();
        let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
            panic!("missing fit")
        };
        assert!(
            (report.candidate.as_ref().unwrap()[0] - 2.).abs() < 1e-5,
            "{report:?}"
        );
        let covariance = report.covariance.as_ref().unwrap();
        assert_eq!(
            covariance.approximation,
            CovarianceApproximation::GaussNewton
        );
        let values = covariance.values.as_ref().unwrap();
        assert!((values[0] - 1.).abs() < 1e-6, "{hessian:?} {values:?}");
        let rows = parameter_covariances::Row::rows(
            &result.table("runtime.parameter_covariances").unwrap(),
        )
        .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].approximation, CovarianceApproximation::GaussNewton);
        let validity =
            local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
        assert_eq!(validity.len(), 1);
        assert_eq!(validity[0].quantity, DerivedQuantity::ParameterCovariance);
        assert!(validity[0].validity.certified);
        // A Gauss–Newton covariance is not read from a KKT point.
        assert_eq!(validity[0].validity.second_order, None);
    }
}

/// Runs the transient fit of [`source`] to its report.
#[cfg(feature = "solver-ipopt")]
async fn transient_fit(
    package: &crate::workflow::ModelingPackage,
    profile: FitProfile,
) -> Arc<crate::workflow::RunResult> {
    let prepared = package
        .prepare_fit(
            FitId::from(id(73)),
            profile,
            compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_bound_dynamic_provider_demands(&prepared.problem);
    assert_verification_accuracy(&prepared.problem);
    prepared.start().unwrap().wait().await.unwrap()
}

/// The programs admitted to native callbacks and the bound workers must agree on
/// their demand, including inner residual minima and explicit provider partials.
#[cfg(any(feature = "solver-ipopt", feature = "solver-idas"))]
fn assert_bound_dynamic_provider_demands(problem: &FitProblem) -> usize {
    let mut checked = 0;
    for experiment in &problem.experiments {
        let Experiment::Transient(simulation) = experiment else {
            continue;
        };
        for function in simulation.program.programs.iter() {
            let providers = &simulation.program.modes[function.mode].providers;
            for body in function.case.assembly.bodies().values() {
                for (key, order) in body
                    .provider_demands(function.case.assembly.order())
                    .unwrap()
                {
                    let admitted = body
                        .providers()
                        .iter()
                        .find(|spec| spec.key() == key)
                        .unwrap();
                    let bound = providers.get(&key).unwrap().spec();
                    bound.check_bound(admitted, order).unwrap_or_else(|error| {
                        panic!(
                            "{:?} provider {} requires {order:?}, bound {:?}: {error}",
                            function.function, admitted.id, bound.derivatives
                        )
                    });
                    checked += 1;
                }
            }
        }
    }
    checked
}
/// S3 through Y4b: an exact-Hessian transient fit, whose Hessian comes from IDAS
/// second-order adjoints, reads its covariance from its own KKT analysis and labels it
/// exact. `y = 71 + p` is linear in `p`, so the exact Hessian carries no residual
/// curvature and `Σ = 1`, as Gauss–Newton gives.
#[cfg(all(feature = "solver-ipopt", feature = "solver-idas"))]
#[tokio::test]
async fn exact_transient_covariance_matches_gauss_newton() {
    use pse_relations::{
        columnar::RelationRow,
        generated::{
            enums::{CovarianceApproximation, DerivedQuantity},
            runtime::local_validity,
        },
    };
    let (package, mut profile) = source(false, 73.).await;
    profile.solver.controls.hessian = HessianMode::Exact;
    for simulation in profile.simulations.values_mut() {
        simulation.method = native::dynamics::Method::Idas;
    }
    let result = transient_fit(&package, profile).await;
    let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
        panic!("missing fit")
    };
    assert!(report.candidate.is_some(), "{report:?}");
    assert!(
        (report.candidate.as_ref().unwrap()[0] - 2.).abs() < 1e-5,
        "{report:?}"
    );
    let covariance = report.covariance.as_ref().unwrap();
    assert_eq!(covariance.approximation, CovarianceApproximation::Exact);
    let values = covariance.values.as_ref().unwrap();
    assert!((values[0] - 1.).abs() < 1e-6, "{values:?}");
    let validity =
        local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
    assert_eq!(validity.len(), 1);
    assert_eq!(validity[0].quantity, DerivedQuantity::ParameterCovariance);
    assert!(validity[0].validity.certified);
    // The exact covariance states the verdicts of the KKT point it was read from.
    assert_eq!(validity[0].validity.second_order, Some(true));
}
/// A parameter held at its bound withholds the covariance: `y = 71 + p` observed at 71
/// puts the unconstrained estimate `p = 0` below the bound 0.1, which holds it.
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn covariance_withheld_at_bound() {
    use pse_relations::{
        columnar::RelationRow,
        generated::{
            enums::{DerivedQuantity, WithheldReason},
            runtime::local_validity,
        },
    };
    let (package, profile) = source(false, 71.).await;
    let result = transient_fit(&package, profile).await;
    let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
        panic!("missing fit")
    };
    assert!(
        (report.candidate.as_ref().unwrap()[0] - 0.1).abs() < 1e-6,
        "{report:?}"
    );
    let covariance = report.covariance.as_ref().unwrap();
    assert!(
        matches!(&covariance.values, Err(FitWithheld::AtBound(held)) if *held == vec![id(3)]),
        "{covariance:?}"
    );
    let validity =
        local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
    assert_eq!(validity.len(), 1);
    assert_eq!(validity[0].quantity, DerivedQuantity::ParameterCovariance);
    assert_eq!(
        validity[0].validity.reason,
        Some(WithheldReason::ParameterAtBound)
    );
}
/// A curved transient fit: `x' = −z/1 s` with the algebraic closure `z = k·x²` from
/// `x(0) = a`, observed through x at four times and z at one, with data from `k = 1.3`,
/// `a = 1.8` (`x = a/(1 + a·k·t)`, t in seconds).
#[cfg(feature = "solver-idas")]
async fn curved_source(
    method: native::dynamics::Method,
) -> (crate::workflow::ModelingPackage, FitProfile) {
    let mut physical = crate::workflow::tests::physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    let mut rows = pse_authoring::language::parse(
        "package p { test Decay fixture { dof 0; route integrated; procedure integrate; integrate samples(0{s},2{s}) relative(1e-8) normalized_absolute(1e-10) step(1e-5{s}); } { domain t: Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); param k: Scalar = 1; param a: Scalar = 2; var x[i in t]: Scalar; var z[i in t]: Scalar; eq rate[i in t]: d(x[i])/di == -z[i]/1{s}; eq closure[i in t]: z[i] == k*x[i]*x[i]; eq initial: x[0{s}] == a; annotation start x(1); annotation start z(1); annotation report x(\"state\"); annotation report z(\"closure\"); } }",
        id(20),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Decay")
        .unwrap()
        .declaration_id;
    let exact = |t: f64| 1.8 / (1. + 1.8 * 1.3 * t);
    let observed = [
        (91, "x", 0.5, exact(0.5)),
        (92, "x", 1.0, exact(1.0)),
        (93, "x", 1.5, exact(1.5)),
        (94, "x", 2.0, exact(2.0)),
        (95, "z", 1.0, 1.3 * exact(1.0).powi(2)),
    ];
    rows.extend(measured_rows(
        &observed
            .iter()
            .map(|(obs, _, _, value)| (id(*obs), "Scalar", Some(*value), Some(0.1)))
            .collect::<Vec<_>>(),
    ));
    let mut data = FitDeclarations::default();
    let mut observations = Vec::new();
    for (obs, member, t, _) in observed {
        observations.push(serde_json::json!({"value_attribute":"value","standard_deviation_attribute":"sigma","observation_id":id(obs),"experiment_id":id(84),"output_path":format!("{member}[0{{s}}]"),"time":t,"time_basis":"model_clock","included":true,"importance":1.}));
    }
    data.fits.push(serde_json::from_value(serde_json::json!({"fit_id":id(80),"parameters":[{"symbol_id":id(81),"fixed":false,"value":1.,"lower":0.1,"upper":10.,"scale":1.},{"symbol_id":id(82),"fixed":false,"value":2.,"lower":0.1,"upper":10.,"scale":1.}],"experiments":[{"experiment_id":id(84),"case_id":root,"route":"integrated","bindings":[{"parameter_id":id(81),"path":"k"},{"parameter_id":id(82),"path":"a"}]}],"observations":observations})).unwrap());
    let profile = FitProfile {
        solver: SolverProfile {
            intent: SolveIntent::Optimize,
            selection: native::solve::SolverSelection::Explicit(Backend::Ipopt),
            controls: native::solve::Controls {
                hessian: HessianMode::Exact,
                ..Default::default()
            },
            presolve: native::presolve::Policy::Off,
            numerics: verification_numerics(),
            convexity: Default::default(),
            backend: native::execution::BackendSettings::Default,
            sensitivity: None,
            composition: Default::default(),
            reconstruction: None,
        },
        simulations: BTreeMap::from([(
            InstanceId::from(id(84)),
            native::dynamics::Profile {
                method,
                end: 2.,
                samples: vec![0., 2.],
                rtol: 1e-8,
                atol: vec![1e-10; 2],
                initial_step: 1e-5,
                parameter_scales: vec![1.; 2],
                ..Default::default()
            },
        )]),
        rank_tolerance: 1e-8,
        max_cells: 100000,
        derivatives: FitDerivatives::Responses,
        uncertainty: None,
    };
    let runtime = crate::workflow::tests::runtime_with_workspace(32 << 20);
    (
        runtime
            .modeling_package(rows, physical)
            .await
            .unwrap()
            .with_fit_declarations(data)
            .await
            .unwrap(),
        profile,
    )
}

/// ADR-0110 item 4 through the fit: an exact Hessian is admitted for an IDAS transient
/// experiment, whose curvature block comes from second-order adjoint sensitivities, and
/// refused with a typed reason for a Diffsol one. The Lagrangian Hessian equals central
/// differences of the forward-sensitivity objective gradient (declared relative tolerance
/// 1e-4, steps of 1e-5) at two points and two objective weights, and the solve records the
/// transient Hessian's source (PS-07).
#[cfg(feature = "solver-idas")]
#[tokio::test]
async fn exact_transient_fit_hessian_matches_finite_difference() {
    let cancel = crate::CancelSource::new();
    let (package, profile) = curved_source(native::dynamics::Method::Diffsol).await;
    let refused = package
        .prepare_fit_problem(
            FitId::from(id(80)),
            profile,
            compiler_profile(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap_err();
    assert_diffsol_exact_hessian_refused(refused);
    let (package, profile) = curved_source(native::dynamics::Method::Idas).await;
    let (problem, _) = package
        .prepare_fit_problem(
            FitId::from(id(80)),
            profile.clone(),
            compiler_profile(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    let Experiment::Transient(s) = &problem.experiments[0] else {
        panic!("transient experiment")
    };
    assert_eq!(s.program.contract.derivatives, DerivativeOrder::Second);
    assert!(assert_bound_dynamic_provider_demands(&problem) > 0);
    let mut execution = Execution::new(
        Arc::new(AtomicBool::new(false)),
        &problem.profile.solver.controls,
    );
    execution.memory = Some(256 << 20);
    let mut oracle = FitOracle::new(problem, execution).unwrap();
    let pattern = oracle.hessian_pattern().unwrap().to_owned().unwrap();
    for x in [[1., 2.], [1.6, 1.5]] {
        for sigma in [1., 0.5] {
            let mut values = vec![0.; pattern.row_idx().len()];
            oracle.hessian(&x, sigma, &[], &mut values).unwrap();
            let h = SparseColMat::new(pattern.clone(), values).to_dense();
            let step = 1e-5;
            for j in 0..2 {
                let mut gradient = |delta: f64| {
                    let mut y = x;
                    y[j] += delta;
                    let mut g = [0.; 2];
                    oracle.gradient(&y, &mut g).unwrap();
                    g
                };
                let plus = gradient(step);
                let minus = gradient(-step);
                for i in j..2 {
                    let difference = sigma * (plus[i] - minus[i]) / (2. * step);
                    assert!(
                        (h[(i, j)] - difference).abs() <= 1e-4 * (1. + difference.abs()),
                        "x={x:?} sigma={sigma}: H[{i},{j}] {} vs differences {difference}",
                        h[(i, j)]
                    );
                }
            }
        }
    }
    #[cfg(feature = "solver-ipopt")]
    {
        let prepared = package
            .prepare_fit(
                FitId::from(id(80)),
                profile,
                compiler_profile(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_verification_accuracy(&prepared.problem);
        let result = prepared.start().unwrap().wait().await.unwrap();
        let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
            panic!("missing fit")
        };
        let candidate = report.candidate.as_ref().unwrap();
        assert!(
            (candidate[0] - 1.3).abs() < 1e-5 && (candidate[1] - 1.8).abs() < 1e-5,
            "{report:?}"
        );
        assert_eq!(report.hessian, HessianMode::Exact);
        let table = result.table("runtime.solve_metrics").unwrap();
        let rows = pse_relations::generated::runtime::solve_metrics::RuntimeSolveMetricsView::from_checked(&table)
            .unwrap()
            .rows()
            .unwrap();
        assert!(rows.iter().any(|r| r.namespace == "derivatives"
            && r.name == "transient_hessian"
            && r.text.as_deref() == Some("idas_forward_over_adjoint")));
    }
}
