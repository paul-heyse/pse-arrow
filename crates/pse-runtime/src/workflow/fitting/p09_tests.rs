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

/// Parameter estimates use the frozen physical target. Dynamic derivative actions
/// are empirical physical-output comparisons, not certified endpoint-error bounds.
fn parameter_allowance(problem: &FitProblem, symbol: SemanticId) -> f64 {
    crate::workflow::tests::engineering_target(&problem.numerics, NumericalTarget::Variable, symbol)
        .budget
}

fn assert_action(actual: f64, expected: f64, allowance: f64) {
    assert!(actual.is_finite() && expected.is_finite() && allowance.is_finite() && allowance >= 0.);
    assert!(
        (actual - expected).abs() <= allowance,
        "empirical action {actual} versus {expected}, composed physical allowance {allowance}"
    );
}

fn objective_scale(problem: &FitProblem) -> f64 {
    crate::workflow::tests::engineering_target(
        &problem.numerics,
        NumericalTarget::Objective,
        SemanticId::NIL,
    )
    .coordinate_scale
}

/// Derivative-only observations use the fit's production job owner too: actual
/// threads, prepared extent, task clock and foreign allowance remain admitted
/// until the evaluator and native thread have been destroyed and joined.
async fn with_admitted_fit_oracle(
    problem: Arc<FitProblem>,
    check: impl FnOnce(&mut FitOracle) + Send + 'static,
) {
    let service = problem.runtime.native().clone();
    let controls = problem.profile.solver.controls.clone();
    let deadline = std::time::Instant::now()
        .checked_add(controls.time_limit)
        .unwrap();
    service
        .job_scoped(
            controls.threads,
            problem.bytes,
            Default::default(),
            Some(deadline),
            move |flag| {
                let scope = pse_kernels::ExecutionScope::new(flag.clone(), Some(deadline));
                let mut execution = Execution::within(flag, &controls, scope)?;
                execution.memory = Some(
                    problem
                        .runtime
                        .shared
                        .budget()
                        .math
                        .foreign_allowance(&controls),
                );
                let mut oracle = FitOracle::new(problem, execution)?;
                check(&mut oracle);
                Ok(())
            },
        )
        .await
        .unwrap();
}

/// The fixture's affine y has unit physical gain from x. The retained production
/// assessment supplies x's quantity allowance; the explicit steady y=p is exact.
fn affine_output_allowances(
    problem: &FitProblem,
    assessments: &[modeling::Assessment],
) -> Vec<f64> {
    problem
        .measurements
        .iter()
        .map(|observation| match &assessments[observation.experiment] {
            modeling::Assessment::Transient(simulation) => {
                assert_eq!(simulation.contract().states.len(), 1);
                let target = crate::workflow::tests::engineering_target(
                    simulation.numerics(),
                    NumericalTarget::Variable,
                    simulation.contract().states[0],
                );
                assert_eq!(target.quantity, observation.port.quantity.as_id());
                target.budget
            }
            modeling::Assessment::Steady { .. } => 0.,
        })
        .collect()
}

/// Compose the independent physical prediction and material response comparisons
/// through the actual weighted loss r*J. This is an empirical acceptance allowance;
/// native local integration tolerances do not certify a global error enclosure.
fn affine_gradient_allowance(
    problem: &FitProblem,
    allowances: &[f64],
    x: f64,
    slopes: &[f64],
    intercepts: &[f64],
) -> f64 {
    let delta = 0.5 * problem.declaration.parameters[0].scale;
    problem
        .measurements
        .iter()
        .enumerate()
        .map(|(row, observation)| {
            let e = allowances[row];
            let residual = intercepts[row] + slopes[row] * x - observation.value.unwrap();
            let action = slopes[row] * delta;
            observation.importance / observation.sigma.unwrap_or(1.).powi(2)
                * (residual.abs() * e + action.abs() * e + e * e)
        })
        .sum::<f64>()
        / objective_scale(problem)
}

/// Independent analytic response over a material half-coordinate parameter action,
/// plus one observed parameter response under the same retained integration profile.
/// These are empirical action checks, not certified derivative-error enclosures.
fn assert_affine_responses(
    oracle: &mut FitOracle,
    x: f64,
    slopes: &[f64],
    intercepts: &[f64],
    assessments: &[modeling::Assessment],
) {
    let problem = oracle.prepared.clone();
    let scale = problem.declaration.parameters[0].scale;
    let delta = 0.5 * scale;
    assert!(delta > parameter_allowance(&problem, id(3)));
    let RankDiagnostic {
        responses, rank, ..
    } = oracle.response_rank(&[x]).unwrap();
    assert_eq!(rank, 1);
    assert_eq!(responses.nrows(), slopes.len());
    assert_eq!(intercepts.len(), slopes.len());
    let base = oracle.evaluate(&[x]).unwrap().predictions.clone();
    let changed = oracle.evaluate(&[x + delta]).unwrap().predictions.clone();
    let allowances = affine_output_allowances(&problem, assessments);
    for (row, &slope) in slopes.iter().enumerate() {
        let measurement = &problem.measurements[row];
        assert_eq!(measurement.id, [id(71), id(72)][row]);
        assert_eq!(measurement.time, (row == 0).then_some(161.));
        let weight = measurement.importance.sqrt() / measurement.sigma.unwrap_or(1.);
        let expected = slope * delta * weight;
        assert_action(
            responses[(row, 0)] * delta * weight,
            expected,
            allowances[row] * weight,
        );
        match &assessments[measurement.experiment] {
            modeling::Assessment::Transient(simulation) => {
                assert_eq!(simulation.contract().states.len(), 1);
                // The authored affine output has unit gain from physical x.
                // Both primal integrations have their own frozen x allowance;
                // their difference cannot use a derivative-error authority.
                let allowance = crate::workflow::tests::engineering_target(
                    simulation.numerics(),
                    NumericalTarget::Variable,
                    simulation.contract().states[0],
                )
                .budget;
                assert!(delta * slope.abs() > 2. * allowance);
                assert!((base[row] - (intercepts[row] + slope * x)).abs() <= allowance);
                assert!(
                    (changed[row] - (intercepts[row] + slope * (x + delta))).abs() <= allowance
                );
                assert!(
                    ((changed[row] - base[row]) * weight - expected).abs()
                        <= 2. * allowance * weight
                );
            }
            modeling::Assessment::Steady { .. } => {
                // This experiment is the explicit primitive y=p, with no local
                // numerical solve or integration error in its value transport.
                assert_eq!(base[row], intercepts[row] + slope * x);
                assert_eq!(changed[row], intercepts[row] + slope * (x + delta));
                assert_eq!((changed[row] - base[row]) * weight, expected);
            }
        }
    }
}

#[cfg(feature = "solver-ipopt")]
fn assert_report_responses(
    problem: &FitProblem,
    report: &FitReport,
    slopes: &[f64],
    allowances: &[f64],
) {
    let candidate = report.candidate.as_ref().unwrap();
    assert_eq!(candidate.len(), 1);
    let responses = report.responses.as_ref().unwrap();
    assert_eq!((responses.nrows(), responses.ncols()), (slopes.len(), 1));
    let delta = 0.5 * problem.declaration.parameters[0].scale;
    assert!(delta > parameter_allowance(problem, id(3)));
    for (row, slope) in slopes.iter().enumerate() {
        let observation = &problem.measurements[row];
        assert_eq!(observation.id, [id(71), id(72)][row]);
        assert_eq!(observation.time, (row == 0).then_some(161.));
        let weight = observation.importance.sqrt() / observation.sigma.unwrap_or(1.);
        assert_action(
            responses[(row, 0)] * delta * weight,
            slope * delta * weight,
            allowances[row] * weight,
        );
    }
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
    let integrate = "integrate samples(160{s},161{s}) relative(global) normalized_absolute(global) step(1e-5{s});";
    let event_tolerance = pse_model::numerics::DEFAULT_ENGINEERING_ACCURACY;
    let text = format!(
        "package p {{ test Dynamic fixture {{dof 0; route integrated; procedure integrate; {integrate}}} {body} test DynamicReset fixture {{dof 0; route integrated; procedure integrate; {integrate} mode before; event hit[160{{s}}] direction(either) tolerance({event_tolerance}{{s}}) reset(x[160{{s}}] = jump[160{{s}}]) next(after); mode after;}} {body} def Steady {{ param p: Scalar = 2; let y: Scalar = p; annotation check p(p > 0); }} }}"
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
    let mut profile = FitProfile {
        solver: SolverProfile {
            intent: SolveIntent::Optimize,
            selection: native::solve::SolverSelection::Explicit(Backend::Ipopt),
            controls: native::solve::Controls {
                hessian: HessianMode::LimitedMemory,
                ..Default::default()
            },
            presolve: native::presolve::Policy::Off,
            numerics: Default::default(),
            convexity: Default::default(),
            backend: native::execution::BackendSettings::Default,
            sensitivity: None,
            composition: Default::default(),
            reconstruction: None,
        },
        simulations: BTreeMap::new(),
        rank_tolerance: 1e-8,
        max_cells: 100000,
        derivatives: FitDerivatives::Responses,
        uncertainty: None,
    };
    let case_id = root(case);
    let runtime = crate::workflow::tests::runtime_with_workspace(32 << 20);
    let package = runtime
        .modeling_package(rows, physical)
        .await
        .unwrap()
        .with_fit_declarations(data)
        .await
        .unwrap();
    let simulation = package
        .declared_simulation(
            case_id,
            compiler_profile(),
            None,
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    profile
        .simulations
        .insert(InstanceId::from(id(74)), simulation.profile().clone());
    (package, profile)
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
    let (problem, assessments) = package
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
    let problem = oracle.prepared.clone();
    let allowance = crate::workflow::tests::engineering_target(
        &problem.numerics,
        NumericalTarget::Objective,
        SemanticId::NIL,
    )
    .budget;
    assert!((oracle.objective(&[2.]).unwrap() - 0.5).abs() <= allowance);
    let mut gradient = [0.];
    oracle.gradient(&[2.], &mut gradient).unwrap();
    let output_allowances = affine_output_allowances(&problem, &assessments);
    assert_action(
        gradient[0] * 0.5 / objective_scale(&problem),
        0.5 / objective_scale(&problem),
        affine_gradient_allowance(&problem, &output_allowances, 2., &[1., 1.], &[71., 0.]),
    );
    assert_affine_responses(&mut oracle, 2., &[1., 1.], &[71., 0.], &assessments);
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
    for integration in profile.simulations.values_mut() {
        integration.method = native::dynamics::Method::Diffsol;
    }
    let (problem, assessments) = package
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
    let execution = Execution::new(
        Arc::new(AtomicBool::new(false)),
        &problem.profile.solver.controls,
    );
    let mut oracle = FitOracle::new(problem, execution).unwrap();
    let mut hessian = vec![0.; oracle.hessian_pattern().unwrap().row_idx().len()];
    let allowances = affine_output_allowances(&oracle.prepared, &assessments);
    let hessian_allowance =
        allowances.iter().map(|e| e + e * e).sum::<f64>() / objective_scale(&oracle.prepared);
    for sigma in [1., 0.25] {
        oracle.hessian(&[2.], sigma, &[], &mut hessian).unwrap();
        // Both responses are dy/dp = 1 with unit weights: σ·JᵀWJ = 2σ.
        assert_eq!(hessian.len(), 1);
        assert_action(
            hessian[0] * 0.25 / objective_scale(&oracle.prepared),
            2. * sigma * 0.25 / objective_scale(&oracle.prepared),
            sigma * hessian_allowance,
        );
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
        let frozen = prepared.problem.clone();
        let output_allowances = affine_output_allowances(&frozen, &prepared.assessments);
        let result = prepared.start().unwrap().wait().await.unwrap();
        let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
            panic!("missing fit")
        };
        // Least squares of (p − 2) and (p − 1).
        assert!(
            (report.candidate.as_ref().unwrap()[0] - 1.5).abs()
                <= parameter_allowance(&frozen, id(3)),
            "{report:?}"
        );
        assert_report_responses(&frozen, report, &[1., 1.], &output_allowances);
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
/// transient experiment. Forward and adjoint material actions are independently checked
/// against the analytic affine experiment on their actual production profiles, with
/// and without a scheduled input. The gradient fit
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
        for derivatives in [FitDerivatives::Responses, FitDerivatives::Gradient] {
            let mut selected = profile.clone();
            selected.derivatives = derivatives;
            let (problem, assessments) = package
                .prepare_fit_problem(
                    FitId::from(id(73)),
                    selected,
                    compiler_profile(),
                    Default::default(),
                    &cancel,
                )
                .await
                .unwrap();
            with_admitted_fit_oracle(problem.into(), move |oracle| {
                let Experiment::Transient(s) = &oracle.prepared.experiments[0] else {
                    panic!("transient experiment")
                };
                assert_eq!(
                    s.profile.sensitivity,
                    if derivatives == FitDerivatives::Responses {
                        native::dynamics::DynamicSensitivity::Forward
                    } else {
                        native::dynamics::DynamicSensitivity::Adjoint
                    }
                );
                for x in [2.5, 0.7] {
                    let mut g = [0.];
                    oracle.gradient(&[x], &mut g).unwrap();
                    assert!(g[0].abs() > 0.1, "a nonzero gradient at {x}: {g:?}");
                    let expected = if scheduled {
                        1.25 * x - 1.5
                    } else {
                        2. * x - 3.
                    };
                    let action = 0.5 / objective_scale(&oracle.prepared);
                    let slopes = [if scheduled { 0.5 } else { 1. }, 1.];
                    let intercepts = [if scheduled { 72. } else { 71. }, 0.];
                    let allowances = affine_output_allowances(&oracle.prepared, &assessments);
                    assert_action(
                        g[0] * action,
                        expected * action,
                        affine_gradient_allowance(
                            &oracle.prepared,
                            &allowances,
                            x,
                            &slopes,
                            &intercepts,
                        ),
                    );
                    // The gradient fit's own integrations carry no sensitivities.
                    if derivatives == FitDerivatives::Gradient {
                        let point = oracle.evaluate(&[x]).unwrap();
                        assert!(
                            point
                                .trajectories
                                .values()
                                .flat_map(|r| &r.samples)
                                .all(|s| s.output_sensitivities.is_empty())
                        );
                    }
                }
                // The rank rerun forms the response Jacobian with forward sensitivities.
                if derivatives == FitDerivatives::Gradient {
                    let expected = if scheduled { 0.5 } else { 1. };
                    assert_affine_responses(
                        oracle,
                        2.,
                        &[expected, 1.],
                        &[if scheduled { 72. } else { 71. }, 0.],
                        &assessments,
                    );
                }
            })
            .await;
        }
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
        let frozen = prepared.problem.clone();
        let output_allowances = affine_output_allowances(&frozen, &prepared.assessments);
        let result = prepared.start().unwrap().wait().await.unwrap();
        let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
            panic!("missing fit")
        };
        // Least squares of (p − 2) and (p − 1).
        assert!(
            (report.candidate.as_ref().unwrap()[0] - 1.5).abs()
                <= parameter_allowance(&frozen, id(3)),
            "{report:?}"
        );
        assert_report_responses(&frozen, report, &[1., 1.], &output_allowances);
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
        let (problem, assessments) = package
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
        assert_affine_responses(
            &mut oracle,
            2.,
            &[if mode == 1 { 0.5 } else { 1. }],
            &[match mode {
                0 => 71.,
                1 => 72.,
                _ => 80.,
            }],
            &assessments,
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
        let frozen = prepared.problem.clone();
        let output_allowances = affine_output_allowances(&frozen, &prepared.assessments);
        let modeling::Assessment::Transient(simulation) = &prepared.assessments[0] else {
            panic!("missing original transient assessment")
        };
        assert_eq!(simulation.contract().states.len(), 1);
        // y=x+t-100 s has unit state gain. Its physical discrepancy therefore
        // uses the actual resolved x allowance, separately from response accuracy.
        let output_allowance = crate::workflow::tests::engineering_target(
            simulation.numerics(),
            NumericalTarget::Variable,
            simulation.contract().states[0],
        )
        .budget;
        let result = prepared.start().unwrap().wait().await.unwrap();
        let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
            panic!("missing fit")
        };
        assert!(
            (report.candidate.as_ref().unwrap()[0] - 3.).abs()
                <= parameter_allowance(&frozen, id(3)),
            "mode {mode}: {report:?}"
        );
        let trajectory = &report.trajectories[&InstanceId::from(id(74))];
        assert_eq!(trajectory.completed_time, 161.);
        let candidate = report.candidate.as_ref().unwrap()[0];
        let slope = if mode == 1 { 0.5 } else { 1. };
        assert!(
            (report.predictions[0].unwrap() - expected).abs()
                <= slope * parameter_allowance(&frozen, id(3)) + output_allowance
        );
        assert_report_responses(&frozen, report, &[slope], &output_allowances);
        let analytic = match mode {
            0 => 71. + candidate,
            1 => 72. + candidate / 2.,
            _ => 80. + candidate,
        };
        assert!(
            (report.predictions[0].unwrap() - analytic).abs() <= output_allowance,
            "physical prediction at accepted candidate {candidate}: {report:?}"
        );
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
        assert_eq!(exported[0].value, report.candidate.as_ref().unwrap()[0]);
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
    let frozen = prepared.problem.clone();
    let result = prepared.start().unwrap().wait().await.unwrap();
    let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
        panic!("missing fit")
    };
    assert!(
        (report.candidate.as_ref().unwrap()[0] - 3.).abs() <= parameter_allowance(&frozen, id(3)),
        "{report:?}"
    );
    assert!(result.usable(), "{report:?}");
}

#[tokio::test]
async fn contextual_closure_fit_factory_controls_preserve_explicit_experiment_profiles() {
    let source = r#"package p {
        entity kind source provenance {attribute title:Text;}
        enum role {given}
        entity source s {title="fit inventory control"}
        constant allowance:Time=1{s} provenance(s,role.given);
        annotation engineering_rule p.allowance;
        test Dynamic fixture {dof 0;route integrated;procedure integrate;
            integrate samples(160{s},161{s}) relative(global) normalized_absolute(global) step(1e-5{s});
        } {
            domain t:Time from 160{s} to 161{s};
            discretize grid on t using integrated(elements=1,order=1);
            param rate:Scalar=2;var x[i in t]:Time;
            annotation engineering_scale x[160{s}](kind=magnitude,value=10000{s});
            conserve stock[i in t]:Time on t inventory x[i] flux rate tolerance p.allowance;
            eq initial:x[160{s}]==10{s};
            annotation report x("inventory");
        }
    }"#;
    let mut rows = pse_authoring::language::parse(
        source,
        id(20),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Dynamic")
        .unwrap()
        .declaration_id;
    rows.extend(measured_rows(&[(id(71), "Time", Some(12.0), Some(1.0))]));
    let mut data = FitDeclarations::default();
    data.fits.push(serde_json::from_value(serde_json::json!({
        "fit_id": id(73),
        "parameters": [{"symbol_id": id(3), "fixed": false, "value": 2.0, "lower": 0.1, "upper": 10.0, "scale": 1.0}],
        "experiments": [{"experiment_id": id(74), "case_id": root, "route": "integrated", "bindings": [{"parameter_id": id(3), "path": "rate"}]}],
        "observations": [{"value_attribute": "value", "standard_deviation_attribute": "sigma", "observation_id": id(71), "experiment_id": id(74), "output_path": "x[160{s}]", "time": 161.0, "time_basis": "model_clock", "included": true, "importance": 1.0}]
    })).unwrap());
    let package = crate::workflow::tests::runtime_with_workspace(32 << 20)
        .modeling_package(rows, crate::workflow::tests::physical())
        .await
        .unwrap()
        .with_fit_declarations(data)
        .await
        .unwrap();
    let mut profile = FitProfile {
        solver: SolverProfile {
            intent: SolveIntent::Optimize,
            selection: native::solve::SolverSelection::Explicit(Backend::Ipopt),
            controls: native::solve::Controls {
                hessian: HessianMode::LimitedMemory,
                ..Default::default()
            },
            presolve: native::presolve::Policy::Off,
            numerics: Default::default(),
            convexity: Default::default(),
            backend: native::execution::BackendSettings::Default,
            sensitivity: None,
            composition: Default::default(),
            reconstruction: None,
        },
        simulations: BTreeMap::new(),
        rank_tolerance: 1e-8,
        max_cells: 100000,
        derivatives: FitDerivatives::Responses,
        uncertainty: None,
    };
    let cancel = crate::CancelSource::new();
    let (_, assessments) = package
        .prepare_fit_problem(
            FitId::from(id(73)),
            profile.clone(),
            compiler_profile(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    let modeling::Assessment::Transient(automatic) = &assessments[0] else {
        panic!("transient experiment required");
    };
    assert_eq!(automatic.profile().out_atol, vec![10.0]);
    assert!(
        automatic
            .contract()
            .balances
            .iter()
            .all(|balance| balance.tolerance == 10.0)
    );
    let mut explicit = automatic.profile().clone();
    explicit.out_atol = vec![1.0];
    profile
        .simulations
        .insert(InstanceId::from(id(74)), explicit);
    let (_, assessments) = package
        .prepare_fit_problem(
            FitId::from(id(73)),
            profile,
            compiler_profile(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    let modeling::Assessment::Transient(explicit) = &assessments[0] else {
        panic!("transient experiment required");
    };
    assert_eq!(explicit.profile().out_atol, vec![1.0]);
    assert!(
        explicit
            .contract()
            .balances
            .iter()
            .all(|balance| balance.tolerance == 10.0)
    );
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
        let frozen = prepared.problem.clone();
        let output_allowances = affine_output_allowances(&frozen, &prepared.assessments);
        let result = prepared.start().unwrap().wait().await.unwrap();
        let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
            panic!("missing fit")
        };
        assert!(
            (report.candidate.as_ref().unwrap()[0] - 2.).abs()
                <= parameter_allowance(&frozen, id(3)),
            "{report:?}"
        );
        let covariance = report.covariance.as_ref().unwrap();
        assert_eq!(
            covariance.approximation,
            CovarianceApproximation::GaussNewton
        );
        let values = covariance.values.as_ref().unwrap();
        assert_report_responses(&frozen, report, &[1.], &output_allowances);
        let response = report.responses.as_ref().unwrap()[(0, 0)];
        let action = values[0] * response * response;
        assert!(
            (action - 1.).abs() / (action.abs() + 1.)
                <= frozen.numerics.policy.linear_backward_error
        );
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
) -> (Arc<FitProblem>, Vec<f64>, Arc<crate::workflow::RunResult>) {
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
    let frozen = prepared.problem.clone();
    let output_allowances = affine_output_allowances(&frozen, &prepared.assessments);
    (
        frozen,
        output_allowances,
        prepared.start().unwrap().wait().await.unwrap(),
    )
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
    let (frozen, output_allowances, result) = transient_fit(&package, profile).await;
    let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
        panic!("missing fit")
    };
    assert!(report.candidate.is_some(), "{report:?}");
    assert!(
        (report.candidate.as_ref().unwrap()[0] - 2.).abs() <= parameter_allowance(&frozen, id(3)),
        "{report:?}"
    );
    let covariance = report.covariance.as_ref().unwrap();
    assert_eq!(covariance.approximation, CovarianceApproximation::Exact);
    let values = covariance.values.as_ref().unwrap();
    assert_report_responses(&frozen, report, &[1.], &output_allowances);
    let response = report.responses.as_ref().unwrap()[(0, 0)];
    let action = values[0] * response * response;
    assert!(
        (action - 1.).abs() / (action.abs() + 1.) <= frozen.numerics.policy.linear_backward_error
    );
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
    let (frozen, _, result) = transient_fit(&package, profile).await;
    let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
        panic!("missing fit")
    };
    assert!(report.quality.as_ref().unwrap().feasible(), "{report:?}");
    let variable = &frozen.contract.variables[0];
    assert_eq!(variable.id, id(3));
    let candidate = report.solve.as_ref().unwrap().candidate.as_ref().unwrap();
    let (lower, upper) = candidate.bound_dual.as_ref().unwrap();
    assert_eq!(
        native::kkt::bound_activity(
            candidate.primal[0],
            (variable.lower, variable.upper),
            frozen.tolerances.variables[0],
            (lower[0], upper[0]),
            (
                frozen.normalization.variables[0],
                frozen.normalization.objective
            ),
            frozen.accuracy.stationarity
        ),
        native::kkt::Activity::Strong(native::kkt::Side::Lower)
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
        "package p { test Decay fixture { dof 0; route integrated; procedure integrate; integrate samples(0{s},2{s}) relative(global) normalized_absolute(global) step(1e-5{s}); } { domain t: Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); param k: Scalar = 1; param a: Scalar = 2; var x[i in t]: Scalar; var z[i in t]: Scalar; eq rate[i in t]: d(x[i])/di == -z[i]/1{s}; eq closure[i in t]: z[i] == k*x[i]*x[i]; eq initial: x[0{s}] == a; annotation start x(1); annotation start z(1); annotation report x(\"state\"); annotation report z(\"closure\"); } }",
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
    let mut profile = FitProfile {
        solver: SolverProfile {
            intent: SolveIntent::Optimize,
            selection: native::solve::SolverSelection::Explicit(Backend::Ipopt),
            controls: native::solve::Controls {
                hessian: HessianMode::Exact,
                ..Default::default()
            },
            presolve: native::presolve::Policy::Off,
            numerics: Default::default(),
            convexity: Default::default(),
            backend: native::execution::BackendSettings::Default,
            sensitivity: None,
            composition: Default::default(),
            reconstruction: None,
        },
        simulations: BTreeMap::new(),
        rank_tolerance: 1e-8,
        max_cells: 100000,
        derivatives: FitDerivatives::Responses,
        uncertainty: None,
    };
    let runtime = crate::workflow::tests::runtime_with_workspace(32 << 20);
    let package = runtime
        .modeling_package(rows, physical)
        .await
        .unwrap()
        .with_fit_declarations(data)
        .await
        .unwrap();
    let simulation = package
        .declared_simulation(
            root,
            compiler_profile(),
            None,
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let mut integration = simulation.profile().clone();
    integration.method = method;
    profile
        .simulations
        .insert(InstanceId::from(id(84)), integration);
    (package, profile)
}

/// Closed form of x'=-k*x², x(0)=a, and z=k*x², differentiated
/// independently of the integrated worker. Time is in seconds, k/a are the actual
/// dimensionless coordinates, and each residual uses its admitted sigma/importance.
#[cfg(feature = "solver-idas")]
struct CurvedReference {
    values: Vec<f64>,
    objective: f64,
    gradient: [f64; 2],
    hessian: [[f64; 2]; 2],
    first: Vec<[f64; 2]>,
    gradient_allowance: [f64; 2],
    hessian_allowance: [[f64; 2]; 2],
}

#[cfg(feature = "solver-idas")]
fn curved_output_allowances(
    problem: &FitProblem,
    assessments: &[modeling::Assessment],
) -> Vec<f64> {
    problem
        .measurements
        .iter()
        .map(|observation| {
            let modeling::Assessment::Transient(simulation) = &assessments[observation.experiment]
            else {
                panic!("curved transient observation required")
            };
            assert_eq!(
                simulation.contract().outputs[observation.row],
                observation.port.id
            );
            let state = simulation
                .contract()
                .states
                .iter()
                .copied()
                .find(|&id| {
                    pse_compiler::workspace::ModelingOutput::Member(id).row_id()
                        == observation.port.id
                })
                .expect("exact observed x/z state identity");
            let target = crate::workflow::tests::engineering_target(
                simulation.numerics(),
                NumericalTarget::Variable,
                state,
            );
            assert_eq!(target.quantity, observation.port.quantity.as_id());
            assert_eq!(target.unit, observation.port.unit.as_id());
            target.budget
        })
        .collect()
}

/// Product composition of empirical primal, First and Second physical-output
/// allowances. These express meaningful output differences over the material
/// directions; they are not global error bounds inferred from native local controls.
#[cfg(feature = "solver-idas")]
fn curved_loss_derivatives(
    problem: &FitProblem,
    point: [f64; 2],
    allowances: &[f64],
) -> CurvedReference {
    let [k, a] = point;
    let mut gradient = [0.; 2];
    let mut hessian = [[0.; 2]; 2];
    let mut gradient_allowance = [0.; 2];
    let mut hessian_allowance = [[0.; 2]; 2];
    let mut responses = Vec::new();
    let mut values = Vec::new();
    let mut objective = 0.;
    assert_eq!(problem.measurements.len(), 5);
    for (row, observation) in problem.measurements.iter().enumerate() {
        assert_eq!(
            observation.id,
            [id(91), id(92), id(93), id(94), id(95)][row]
        );
        let t = observation.time.unwrap();
        let d = 1. + a * k * t;
        let x = a / d;
        let j = [-a * a * t / (d * d), 1. / (d * d)];
        let h = [
            [
                2. * a * a * a * t * t / (d * d * d),
                -2. * a * t / (d * d * d),
            ],
            [-2. * a * t / (d * d * d), -2. * k * t / (d * d * d)],
        ];
        let (value, first, second) = if observation.id == id(95) {
            let first = [x * x + 2. * k * x * j[0], 2. * k * x * j[1]];
            let second = [
                [
                    4. * x * j[0] + 2. * k * (j[0] * j[0] + x * h[0][0]),
                    2. * x * j[1] + 2. * k * (j[0] * j[1] + x * h[0][1]),
                ],
                [
                    2. * x * j[1] + 2. * k * (j[0] * j[1] + x * h[0][1]),
                    2. * k * (j[1] * j[1] + x * h[1][1]),
                ],
            ];
            (k * x * x, first, second)
        } else {
            (x, j, h)
        };
        let weight = observation.importance / observation.sigma.unwrap_or(1.).powi(2);
        let residual = value - observation.value.unwrap();
        values.push(value);
        objective += 0.5 * weight * residual * residual;
        let e = allowances[row];
        responses.push(first);
        for i in 0..2 {
            gradient[i] += weight * residual * first[i];
            let direction_i = 0.25 * problem.declaration.parameters[i].scale;
            let first_i = first[i] * direction_i;
            gradient_allowance[i] += weight * (residual.abs() * e + first_i.abs() * e + e * e)
                / objective_scale(problem);
            for j in 0..2 {
                hessian[i][j] += weight * (first[i] * first[j] + residual * second[i][j]);
                let direction_j = 0.25 * problem.declaration.parameters[j].scale;
                let first_j = first[j] * direction_j;
                let curvature = second[i][j] * direction_i * direction_j;
                hessian_allowance[i][j] += weight
                    * (first_i.abs() * e
                        + first_j.abs() * e
                        + e * e
                        + residual.abs() * e
                        + curvature.abs() * e
                        + e * e)
                    / objective_scale(problem);
            }
        }
    }
    CurvedReference {
        values,
        objective,
        gradient,
        hessian,
        first: responses,
        gradient_allowance,
        hessian_allowance,
    }
}

/// ADR-0110 item 4 through the fit: an exact Hessian is admitted for an IDAS transient
/// experiment, whose curvature block comes from second-order adjoint sensitivities, and
/// refused with a typed reason for a Diffsol one. Integrated First and Second
/// actions are checked against independent closed-form derivatives in the actual
/// weighted fit coordinates, composing the actual meaningful physical-output
/// allowances. The actual IDAS profile controls its sensitivity solves. These
/// empirical checks do not infer global error enclosures from local tolerances.
#[cfg(feature = "solver-idas")]
#[tokio::test]
async fn exact_transient_fit_hessian_satisfies_production_action_basis() {
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
    let (problem, assessments) = package
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
    let output_allowances = curved_output_allowances(&problem, &assessments);
    with_admitted_fit_oracle(problem.into(), move |oracle| {
        let pattern = oracle.hessian_pattern().unwrap().to_owned().unwrap();
        for x in [[1., 2.], [1.6, 1.5]] {
            for sigma in [1., 0.5] {
                let mut values = vec![0.; pattern.row_idx().len()];
                oracle.hessian(&x, sigma, &[], &mut values).unwrap();
                let h = SparseColMat::new(pattern.clone(), values).to_dense();
                let problem = oracle.prepared.clone();
                let analytic = curved_loss_derivatives(&problem, x, &output_allowances);
                let mut gradient = [0.; 2];
                oracle.gradient(&x, &mut gradient).unwrap();
                let responses = oracle.response_rank(&x).unwrap().responses;
                let actual_point = oracle.evaluate(&x).unwrap();
                for row in 0..problem.measurements.len() {
                    assert!(
                        (actual_point.predictions[row] - analytic.values[row]).abs()
                            <= output_allowances[row]
                    );
                }
                let loss_allowance = problem
                    .measurements
                    .iter()
                    .enumerate()
                    .map(|(row, observation)| {
                        let residual = analytic.values[row] - observation.value.unwrap();
                        let e = output_allowances[row];
                        observation.importance / observation.sigma.unwrap_or(1.).powi(2)
                            * (residual.abs() * e + 0.5 * e * e)
                    })
                    .sum::<f64>();
                assert!(
                    (oracle.objective(&x).unwrap() - analytic.objective).abs() <= loss_allowance
                );
                for i in 0..2 {
                    let scale_i = problem.declaration.parameters[i].scale;
                    assert!(
                        0.25 * scale_i
                            > parameter_allowance(
                                &problem,
                                problem.declaration.parameters[i].symbol_id
                            )
                    );
                    for row in 0..problem.measurements.len() {
                        assert_action(
                            responses[(row, i)] * 0.25 * scale_i,
                            analytic.first[row][i] * 0.25 * scale_i,
                            output_allowances[row],
                        );
                    }
                    assert_action(
                        gradient[i] * 0.25 * scale_i / objective_scale(&problem),
                        analytic.gradient[i] * 0.25 * scale_i / objective_scale(&problem),
                        analytic.gradient_allowance[i],
                    );
                    for j in 0..=i {
                        let action =
                            0.25 * scale_i * 0.25 * problem.declaration.parameters[j].scale
                                / objective_scale(&problem);
                        assert_action(
                            h[(i, j)] * action,
                            sigma * analytic.hessian[i][j] * action,
                            sigma * analytic.hessian_allowance[i][j],
                        );
                    }
                }
            }
        }
    })
    .await;
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
        let frozen = prepared.problem.clone();
        let output_allowances = curved_output_allowances(&frozen, &prepared.assessments);
        let result = prepared.start().unwrap().wait().await.unwrap();
        let crate::workflow::RunReport::Fit(report) = result.report().unwrap() else {
            panic!("missing fit")
        };
        let candidate = report.candidate.as_ref().unwrap();
        // Stationary KKT acceptance does not promise forward parameter error.
        // Independently assess the physical curve and weighted loss at the
        // actual accepted candidate against the authored noiseless optimum.
        assert!(report.quality.as_ref().unwrap().feasible(), "{report:?}");
        assert!(report.estimate_qualified() && result.usable(), "{report:?}");
        assert_eq!(report.hessian, HessianMode::Exact);
        let point = [candidate[0], candidate[1]];
        let analytic = curved_loss_derivatives(&frozen, point, &output_allowances);
        let objective_allowance = crate::workflow::tests::engineering_target(
            &frozen.numerics,
            NumericalTarget::Objective,
            SemanticId::NIL,
        )
        .budget;
        assert!(
            analytic.objective <= objective_allowance,
            "independent weighted loss at {point:?}: {}",
            analytic.objective
        );
        assert!(
            report.objective.unwrap() <= objective_allowance,
            "{report:?}"
        );
        let responses = report.responses.as_ref().unwrap();
        for row in 0..frozen.measurements.len() {
            assert!(
                (report.predictions[row].unwrap() - analytic.values[row]).abs()
                    <= output_allowances[row]
            );
            for i in 0..2 {
                let direction = 0.25 * frozen.declaration.parameters[i].scale;
                assert_action(
                    responses[(row, i)] * direction,
                    analytic.first[row][i] * direction,
                    output_allowances[row],
                );
            }
        }
        with_admitted_fit_oracle(frozen, move |actual| {
            let frozen = actual.prepared.clone();
            let pattern = actual.hessian_pattern().unwrap().to_owned().unwrap();
            let mut gradient = [0.; 2];
            actual.gradient(&point, &mut gradient).unwrap();
            let mut values = vec![0.; pattern.row_idx().len()];
            actual.hessian(&point, 1., &[], &mut values).unwrap();
            let h = SparseColMat::new(pattern.clone(), values).to_dense();
            for i in 0..2 {
                let scale_i = frozen.declaration.parameters[i].scale;
                assert_action(
                    gradient[i] * 0.25 * scale_i / objective_scale(&frozen),
                    analytic.gradient[i] * 0.25 * scale_i / objective_scale(&frozen),
                    analytic.gradient_allowance[i],
                );
                for j in 0..=i {
                    let action = 0.25 * scale_i * 0.25 * frozen.declaration.parameters[j].scale
                        / objective_scale(&frozen);
                    assert_action(
                        h[(i, j)] * action,
                        analytic.hessian[i][j] * action,
                        analytic.hessian_allowance[i][j],
                    );
                }
            }
        })
        .await;
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

/// The actual callback demand drives native passes and exact-bit point installation.
#[tokio::test]
async fn transient_fit_demand_cache_and_coherent_upgrade() {
    let (package, mut selected) = source(true, 73.).await;
    selected.derivatives = FitDerivatives::Gradient;
    let (problem, _) = package
        .prepare_fit_problem(
            FitId::from(id(73)),
            selected,
            compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    with_admitted_fit_oracle(problem.into(), move |oracle| {
        let mut first = [0.];
        oracle.gradient(&[2.5], &mut first).unwrap();
        assert_eq!(oracle.transient_passes, (1, 1));
        let report = oracle
            .point
            .as_ref()
            .unwrap()
            .trajectories
            .values()
            .next()
            .unwrap()
            .clone();
        let mut repeated = [0.];
        oracle.gradient(&[2.5], &mut repeated).unwrap();
        assert_eq!(first, repeated);
        assert_eq!(oracle.transient_passes, (1, 1));
        assert!(Arc::ptr_eq(
            &report,
            oracle
                .point
                .as_ref()
                .unwrap()
                .trajectories
                .values()
                .next()
                .unwrap()
        ));
        let earlier = oracle.objective(&[0.7]).unwrap();
        assert_eq!(oracle.transient_passes, (2, 1));
        assert!(oracle.point.as_ref().unwrap().adjoint.is_none());
        let prior = oracle
            .point
            .as_ref()
            .unwrap()
            .trajectories
            .values()
            .next()
            .unwrap()
            .clone();
        oracle.gradient(&[0.7], &mut repeated).unwrap();
        assert_eq!(oracle.transient_passes, (3, 2));
        assert!(!Arc::ptr_eq(
            &prior,
            oracle
                .point
                .as_ref()
                .unwrap()
                .trajectories
                .values()
                .next()
                .unwrap()
        ));
        assert!(earlier.is_finite());
        assert!(oracle.point.as_ref().unwrap().adjoint.is_some());
        oracle.gradient(&[0.7], &mut first).unwrap();
        assert_eq!(oracle.transient_passes, (3, 2));
        // A materially inconsistent prior observation refuses the upgrade and leaves
        // no derivative or mixed prediction cache installed.
        oracle.objective(&[2.]).unwrap();
        let old = oracle
            .point
            .as_mut()
            .unwrap()
            .trajectories
            .values_mut()
            .next()
            .unwrap();
        for sample in &mut Arc::get_mut(old)
            .expect("the actual cached report has one owner")
            .samples
        {
            for value in &mut sample.outputs {
                *value += 100.;
            }
        }
        assert!(oracle.gradient(&[2.], &mut first).is_err());
        assert!(oracle.point.is_none());
        oracle.gradient(&[2.], &mut first).unwrap();
        assert!(oracle.point.as_ref().unwrap().adjoint.is_some());
        // A nonfinite prediction in the actual earlier report also refuses atomically.
        oracle.objective(&[1.5]).unwrap();
        let old = oracle
            .point
            .as_mut()
            .unwrap()
            .trajectories
            .values_mut()
            .next()
            .unwrap();
        for sample in &mut Arc::get_mut(old)
            .expect("the actual cached report has one owner")
            .samples
        {
            sample.outputs.fill(f64::NAN);
        }
        assert!(oracle.gradient(&[1.5], &mut first).is_err());
        assert!(oracle.point.is_none());
        let memory = oracle.execution.memory.take();
        oracle.objective(&[1.3]).unwrap();
        assert!(oracle.gradient(&[1.3], &mut first).is_err());
        assert!(oracle.point.is_none());
        oracle.execution.memory = memory;
        oracle.gradient(&[1.3], &mut first).unwrap();
        // Signed zero has a distinct candidate key even with equal numerical values.
        oracle.objective(&[0.]).unwrap();
        let passes = oracle.transient_passes;
        oracle.objective(&[-0.]).unwrap();
        assert_eq!(oracle.transient_passes, (passes.0 + 1, passes.1));
        oracle
            .execution
            .cancel
            .store(true, std::sync::atomic::Ordering::Release);
        assert!(oracle.gradient(&[-0.], &mut first).is_err());
        assert!(oracle.point.is_none());
        oracle
            .execution
            .cancel
            .store(false, std::sync::atomic::Ordering::Release);
        // Candidate bits participate in the key; invalid values retire the cache.
        assert!(oracle.gradient(&[f64::NAN], &mut first).is_err());
        assert!(oracle.point.is_none());
    })
    .await;
}
