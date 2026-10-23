// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pinned LM 0.15 nalgebra-0.35 revision over the actual unbounded fitting residual/response authority.
use super::*;
use levenberg_marquardt::{LeastSquaresProblem, LevenbergMarquardt, TerminationReason};
use nalgebra::{DMatrix, DVector, Dyn, storage::Owned};
use std::cell::{Cell, RefCell};
use std::time::Instant;

type FixtureResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const MATRIX_ALLOWANCE: usize = 1 << 20;

#[tokio::test]
async fn fit_covariance_owned_result_reconciles_decoded_content_and_refuses_tampering() {
    let seed = fixture("linear", [1.0; 2]).await.unwrap();
    let mut profile = seed.problem.profile.clone();
    profile.uncertainty = Some(FitUncertainty {
        level: Fraction::try_new(0.95).unwrap(),
        profile: None,
        predictions: false,
    });
    let fit = production_profile(&seed, profile).await.unwrap();
    let result = fit.start().unwrap().wait().await.unwrap();
    let original = result.parameter_covariance().unwrap().unwrap();
    let bytes = serde_json::to_vec(&original).unwrap();
    let mut decoded: crate::math::settings::ParameterCovariance =
        serde_json::from_slice(&bytes).unwrap();
    assert!(decoded.verified_fit().is_err());
    result.reconcile_parameter_covariance(&mut decoded).unwrap();
    assert!(decoded.verified_fit().unwrap().is_some());
    for field in 0..7 {
        let mut altered = decoded.clone();
        match field {
            0 => altered.values[0] = pse_model::scalars::FiniteBound::try_new(2.0).unwrap(),
            1 => altered.parameters.swap(0, 1),
            2 => altered.parameter_quantities.swap(0, 1),
            3 => altered.parameter_units[0] = SemanticId::from_bytes([99; 16]),
            4 => altered.scales[0] = pse_model::scalars::FiniteBound::try_new(3.0).unwrap(),
            5 => {
                altered.origin = crate::math::settings::CovarianceOrigin::QualifiedFit {
                    run_id: pse_model::generated::identities::RunId::from_bytes([99; 16]),
                    record: None,
                }
            }
            _ => {
                altered.origin = crate::math::settings::CovarianceOrigin::External {
                    label: "unverified external substitution".into(),
                    assumptions: vec!["caller input".into()],
                }
            }
        }
        // This fixture's parameters may share the same quantity; force an actual
        // change instead of mistaking equal physical contracts for a new input.
        if field == 2 {
            altered.parameter_quantities[0] = SemanticId::from_bytes([98; 16]);
        }
        assert!(
            result.reconcile_parameter_covariance(&mut altered).is_err(),
            "field {field}"
        );
    }
}

struct ResidualProblem {
    oracle: RefCell<FitOracle>,
    x: DVector<f64>,
    error: RefCell<Option<ProblemError>>,
    evaluations: Cell<usize>,
    jacobians: Cell<usize>,
    cap: usize,
}
impl ResidualProblem {
    fn new(
        problem: Arc<FitProblem>,
        execution: Execution,
        x: Vec<f64>,
        cap: usize,
    ) -> Result<Self, ProblemError> {
        if problem
            .contract
            .variables
            .iter()
            .any(|v| v.lower.is_finite() || v.upper.is_finite())
            || !problem.bounds.is_empty()
            || x.len() > 16
            || problem.measurements.len() > 32
        {
            return Err(ProblemError::unsupported(
                "private LM fixture is not bounded unboxed residual-only fitting",
            ));
        }
        Ok(Self {
            oracle: RefCell::new(FitOracle::new(problem, execution)?),
            x: DVector::from_vec(x),
            error: RefCell::new(None),
            evaluations: Cell::new(0),
            jacobians: Cell::new(0),
            cap,
        })
    }
    fn retain<T>(&self, result: Result<T, ProblemError>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                if self.error.borrow().is_none() {
                    *self.error.borrow_mut() = Some(error);
                }
                None
            }
        }
    }
}
impl LeastSquaresProblem<f64, Dyn, Dyn> for ResidualProblem {
    type ParameterStorage = Owned<f64, Dyn>;
    type ResidualStorage = Owned<f64, Dyn>;
    type JacobianStorage = Owned<f64, Dyn, Dyn>;
    fn set_params(&mut self, x: &DVector<f64>) {
        self.x.copy_from(x);
    }
    fn params(&self) -> DVector<f64> {
        self.x.clone()
    }
    fn residuals(&self) -> Option<DVector<f64>> {
        if self.error.borrow().is_some() {
            return None;
        }
        self.retain((|| {
            let mut oracle = self.oracle.borrow_mut();
            oracle.execution.check()?;
            if self.evaluations.get() >= self.cap {
                return Err(ProblemError::Limit {
                    kind: native::LimitKind::Work,
                    detail: "private LM original residual evaluation cap".into(),
                });
            }
            self.evaluations.set(self.evaluations.get() + 1);
            let problem = oracle.prepared.clone();
            let point = oracle.evaluate_demand(self.x.as_slice(), Demand::default())?;
            let values = problem
                .measurements
                .iter()
                .zip(&point.predictions)
                .filter(|(measurement, _)| measurement.included)
                .map(|(measurement, prediction)| {
                    FitOracle::residual(measurement, *prediction).map(|value| value.0)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(DVector::from_vec(values))
        })())
    }
    fn jacobian(&self) -> Option<DMatrix<f64>> {
        if self.error.borrow().is_some() {
            return None;
        }
        self.retain((|| {
            let mut oracle = self.oracle.borrow_mut();
            oracle.execution.check()?;
            self.jacobians.set(self.jacobians.get() + 1);
            let problem = oracle.prepared.clone();
            let point = oracle.evaluate_demand(self.x.as_slice(), Demand::responses())?;
            let mut mapping = vec![None; problem.measurements.len()];
            let mut rows = 0;
            for (index, (measurement, prediction)) in problem
                .measurements
                .iter()
                .zip(&point.predictions)
                .enumerate()
            {
                if measurement.included {
                    mapping[index] = Some((rows, FitOracle::residual(measurement, *prediction)?.1));
                    rows += 1;
                }
            }
            let source = point.responses.matrix();
            let mut result = DMatrix::zeros(rows, self.x.len());
            for column in 0..source.ncols() {
                for entry in source.col_range(column) {
                    if let Some((row, weight)) = mapping[source.row_idx()[entry]] {
                        result[(row, column)] = source.val()[entry] * weight;
                    }
                }
            }
            Ok(result)
        })())
    }
}

pub(in crate::workflow::fitting) async fn fixture(
    kind: &str,
    scales: [f64; 2],
) -> FixtureResult<PreparedFit> {
    fixture_on(kind, scales, crate::workflow::tests::runtime()).await
}
async fn fixture_on(
    kind: &str,
    scales: [f64; 2],
    runtime: crate::workflow::Runtime,
) -> FixtureResult<PreparedFit> {
    fixture_with_physical(kind, scales, runtime, crate::workflow::tests::physical()).await
}
async fn fixture_with_physical(
    kind: &str,
    scales: [f64; 2],
    runtime: crate::workflow::Runtime,
    physical: crate::workflow::PhysicalContext,
) -> FixtureResult<PreparedFit> {
    use crate::workflow::tests::{compiler_profile, id};
    let outputs = (1..=4)
        .map(|i| {
            if kind == "cosine-gap" {
                return match i {
                    1 => "let y1:Scalar=a;".into(),
                    2 => "let y2:Scalar=b;".into(),
                    _ => format!("let y{i}:Scalar=1e16;"),
                };
            }
            let column = if kind == "near-collinear" {
                1. + (i as f64) * 0.0001
            } else {
                i as f64
            };
            let intercept = if kind == "domain" { "sqrt(a)" } else { "a" };
            format!("let y{i}: Scalar = {intercept} + b*{column};")
        })
        .collect::<String>();
    let mut rows = pse_authoring::language::parse(
        &format!(
            "package p {{ def Root {{ param a: Scalar = 1; param b: Scalar = 1; {outputs} }} }}"
        ),
        id(20),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )?;
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .ok_or_else(|| WorkflowError::Input("fixture Root declaration missing".into()))?
        .declaration_id;
    if kind=="research-calibration" {
        let readings=(0..4).map(|i|format!("@id(\"{}\") entity reading observation{i} provenance(experiment,role.measured) {{raw={},sigma={}}}",id(40+i as u8),i+1,regression::SIGMA[i])).collect::<String>();
        rows.extend(pse_authoring::language::parse(&format!("package measurements {{entity kind origin provenance {{attribute title:Text;}} entity origin experiment {{title=\"independent affine calibration\"}} enum role {{measured facets(measured)}} entity kind reading {{attribute raw:Scalar;attribute sigma:Scalar;derived calibrated:Scalar=2*raw+3;derived calibrated_sigma:Scalar=2*sigma;require raw>0 : \"positive calibration domain\";}} {readings}}}"),id(222),pse_authoring::language::IdentityPolicy::Named,Default::default())?);
    }else{
    rows.extend(measured_rows(
        &(0..4)
            .map(|i| {
                (
                    id(40 + i as u8),
                    "Scalar",
                    Some(regression::Y[i]),
                    Some(regression::SIGMA[i]),
                )
            })
            .collect::<Vec<_>>(),
    ));
    }
    let parameters = (0..2)
        .map(|i| {
            serde_json::json!({"symbol_id":id(i as u8 + 1),"fixed":false,
        "value":1.,"lower":null,"upper":null,"scale":scales[i]})
        })
        .collect::<Vec<_>>();
    let observations = (0..4).map(|i| serde_json::json!({"value_attribute":if kind=="research-calibration"{"calibrated"}else{"value"},
        "standard_deviation_attribute":if kind=="research-calibration"{"calibrated_sigma"}else{"sigma"},"observation_id":id(40 + i as u8),"experiment_id":id(33),
        "output_path":format!("y{}",i+1),"time":null,"included":true,"importance":1.})).collect::<Vec<_>>();
    let declaration = serde_json::from_value(
        serde_json::json!({"fit_id":id(32),"parameters":parameters,
        "experiments":[{"experiment_id":id(33),"case_id":root,"route":"steady","bindings":[
            {"parameter_id":id(1),"path":"a"},{"parameter_id":id(2),"path":"b"}]}],"observations":observations}),
    )?;
    Ok(runtime
        .modeling_package(rows, physical)
        .await?
        .with_fit_declarations(FitDeclarations::from_owned_rows(vec![declaration], vec![]))
        .await?
        .prepare_fit(
            id(32).into(),
            regression::profile(HessianMode::GaussNewton, None),
            compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await?)
}
fn fresh(
    problem: Arc<FitProblem>,
    execution: Execution,
    x: &[f64],
) -> Result<(f64, Vec<f64>), ProblemError> {
    let mut oracle = FitOracle::new(problem, execution)?;
    let objective = oracle.objective(x)?;
    let mut gradient = vec![0.; x.len()];
    oracle.gradient(x, &mut gradient)?;
    Ok((objective, gradient))
}
/// This fixture is linear in (intercept, slope), including sqrt(a)'s positive
/// domain. Centering gives a stable independent quadratic identity even when
/// the two authored columns are nearly collinear.
struct QuadraticWitness {
    best: f64,
    gap: f64,
    scaled: f64,
    gradient: [f64; 2],
}
fn quadratic_witness(kind: &str, p: &FitProblem, x: &[f64], objective: f64) -> QuadraticWitness {
    if kind == "cosine-gap" {
        // This deliberately different authored model has two independent observed
        // parameters and two large constant residuals; its gradient is not the
        // intercept/slope gradient used by the other fixtures.
        let difference = [x[0] - regression::Y[0], x[1] - regression::Y[1]];
        let gradient = [
            difference[0] / regression::SIGMA[0].powi(2),
            difference[1] / regression::SIGMA[1].powi(2),
        ];
        let gap = 0.5
            * (difference[0].powi(2) / regression::SIGMA[0].powi(2)
                + difference[1].powi(2) / regression::SIGMA[1].powi(2));
        let best = 0.5
            * (2..4)
                .map(|i| ((1e16 - regression::Y[i]) / regression::SIGMA[i]).powi(2))
                .sum::<f64>();
        let scaled = gradient
            .iter()
            .zip(&p.normalization.variables)
            .map(|(g, scale)| (g * scale / p.normalization.objective).abs())
            .fold(0., f64::max);
        assert!((objective - best - gap).abs() <= 1e-7 * objective.abs().max(1.));
        return QuadraticWitness {
            best,
            gap,
            scaled,
            gradient,
        };
    }
    let columns: Vec<_> = (1..=4)
        .map(|i| {
            if kind == "near-collinear" {
                1. + i as f64 * 0.0001
            } else {
                i as f64
            }
        })
        .collect();
    let weights: Vec<_> = regression::SIGMA
        .iter()
        .map(|sigma| 1. / (sigma * sigma))
        .collect();
    let total = weights.iter().sum::<f64>();
    let mean_x = columns
        .iter()
        .zip(&weights)
        .map(|(x, w)| x * w)
        .sum::<f64>()
        / total;
    let mean_y = regression::Y
        .iter()
        .zip(&weights)
        .map(|(y, w)| y * w)
        .sum::<f64>()
        / total;
    let centered = columns
        .iter()
        .zip(&weights)
        .map(|(x, w)| w * (x - mean_x).powi(2))
        .sum::<f64>();
    let slope = columns
        .iter()
        .zip(regression::Y)
        .zip(&weights)
        .map(|((x, y), w)| w * (x - mean_x) * (y - mean_y))
        .sum::<f64>()
        / centered;
    let intercept = mean_y - slope * mean_x;
    let best = columns
        .iter()
        .zip(regression::Y)
        .zip(&weights)
        .map(|((x, y), w)| 0.5 * w * (intercept + slope * x - y).powi(2))
        .sum::<f64>();
    let actual_intercept = if kind == "domain" { x[0].sqrt() } else { x[0] };
    assert!(actual_intercept.is_finite());
    let du = actual_intercept - intercept;
    let db = x[1] - slope;
    let gap = 0.5 * (total * (du + mean_x * db).powi(2) + centered * db * db);
    let mut gradient = [0., 0.];
    for ((column, y), weight) in columns.iter().zip(regression::Y).zip(&weights) {
        let residual = actual_intercept + x[1] * column - y;
        gradient[0] += weight * residual;
        gradient[1] += weight * residual * column;
    }
    // 1/2 g^T H^-1 g in centered coordinates, with the exact authored
    // intercept derivative restored before physical/scaled stationarity.
    let gradient_gap = 0.5
        * (gradient[0].powi(2) / total + (gradient[1] - mean_x * gradient[0]).powi(2) / centered);
    assert!((gradient_gap - gap).abs() <= 1e-7 * (1. + gap));
    if kind == "domain" {
        gradient[0] /= 2. * actual_intercept;
    }
    let scaled = gradient
        .iter()
        .zip(&p.normalization.variables)
        .map(|(g, scale)| (g * scale / p.normalization.objective).abs())
        .fold(0., f64::max);
    assert!(
        (objective - best - gap).abs() <= 1e-7 * (1. + objective.abs()),
        "original objective must equal analytic optimum plus its exact current quadratic gap"
    );
    QuadraticWitness {
        best,
        gap,
        scaled,
        gradient,
    }
}
#[tokio::test]
async fn pinned_lm_actual_fit_residual_inquiry_preserves_original_weighted_objective() {
    for (kind, scales) in [
        ("linear", [1., 2.]),
        ("linear", [0.001, 1000.]),
        ("near-collinear", [1., 2.]),
    ] {
        let prepared = fixture(kind, scales).await.unwrap();
        let problem = prepared.problem.clone();
        let mut comparisons = Vec::new();
        for mode in [
            HessianMode::GaussNewton,
            HessianMode::Exact,
            HessianMode::LimitedMemory,
        ] {
            comparisons.push(
                prepared
                    .source
                    .prepare_fit(
                        prepared.declaration().fit_id,
                        regression::profile(mode, None),
                        crate::workflow::tests::compiler_profile(),
                        Default::default(),
                        &crate::CancelSource::new(),
                    )
                    .await
                    .unwrap(),
            );
        }
        let bytes = comparisons
            .iter()
            .map(|prepared| prepared.problem.bytes)
            .chain(std::iter::once(problem.bytes))
            .max()
            .unwrap()
            + MATRIX_ALLOWANCE;
        let service = problem.runtime.native().clone();
        service.job_scoped(1, bytes, Default::default(), None, move |flag| {
            let controls = problem.profile.solver.controls.clone();
            let execution = Execution::new(flag, &controls);
            let started = Instant::now();
            let adapter = ResidualProblem::new(problem.clone(), execution.clone(), problem.initial.clone(), 200)?;
            let (adapter, report) = LevenbergMarquardt::new().with_tol(0.001).with_patience(40).minimize(adapter);
            assert!(adapter.error.borrow().is_none(), "{:?}", adapter.error.borrow());
            let (objective, gradient) = fresh(problem.clone(), execution.clone(), adapter.x.as_slice())?;
            assert!(objective > 0., "the inquiry must retain nonzero residuals");
            assert!((objective - report.objective_function).abs() <= 0.001 * objective.abs().max(1.));
            assert!(gradient.iter().all(|value| value.abs() <= 0.001), "{kind}: {gradient:?}");
            println!("LM0.15 kind={kind} scales={scales:?} native={:?} original={objective} gradient={gradient:?} residual_calls={} jacobian_calls={} prepaid_matrix_bytes={MATRIX_ALLOWANCE} complete_ms={}",
                report.termination, adapter.evaluations.get(), adapter.jacobians.get(), started.elapsed().as_secs_f64()*1000.);
            for prepared in comparisons {
                let problem = &prepared.problem;
                let solver = &problem.profile.solver;
                let mode = solver.controls.hessian;
                let started = Instant::now();
                let native = problem.execute_profile(solver, prepared.route(), execution.scope()?,
                    Arc::new(native::solve::Progress::new(256)), 1, None)?;
                let candidate = native.candidate.as_ref().expect("current Ipopt route must retain a physical candidate");
                let (actual, _) = fresh(problem.clone(), execution.clone(), candidate)?;
                let QuadraticWitness { best, gap, scaled, gradient } = quadratic_witness(kind, problem, candidate, actual);
                println!("fit_quadratic kind={kind} x={candidate:?} physical_gradient={gradient:?} scaled_stationarity={scaled} analytic_best={best} exact_gap={gap}");
                assert!((objective-best).abs() <= 0.001*(1.+best.abs()), "LM original optimum: {kind}");
                assert!(scaled <= problem.accuracy.stationarity, "{kind} {mode:?}: stationarity {scaled}");
                if gap > 0.001*(1.+best.abs()) {
                    assert_eq!((kind, scales, mode), ("linear", [0.001,1000.], HessianMode::LimitedMemory),
                        "an unanticipated objective-gap classification needs its own causal evidence");
                }
                println!("Ipopt kind={kind} hessian={mode:?} original={actual} quality={:?} complete_ms={}", native.quality, started.elapsed().as_secs_f64()*1000.);
            }
            Ok(())
        }).await.unwrap();
    }
}
#[tokio::test]
async fn pinned_lm_actual_fit_refusal_cancel_and_evaluation_cap_remain_typed() {
    let prepared = fixture("domain", [1., 2.]).await.unwrap();
    let problem = prepared.problem.clone();
    let service = problem.runtime.native().clone();
    service
        .job_scoped(
            1,
            problem.bytes + MATRIX_ALLOWANCE,
            Default::default(),
            None,
            move |flag| {
                for (kind, x, cap) in [
                    ("domain", vec![-1., 1.], 40),
                    ("cancel", vec![1., 1.], 40),
                    ("cap", vec![1., 1.], 1),
                ] {
                    if flag.flag().load(std::sync::atomic::Ordering::Acquire) {
                        return Err(ProblemError::Cancelled.into());
                    }
                    let execution =
                        Execution::new(Default::default(), &problem.profile.solver.controls);
                    let adapter = ResidualProblem::new(problem.clone(), execution.clone(), x, cap)?;
                    if kind == "cancel" {
                        execution.cancellation_control().cancel();
                    }
                    let (adapter, report) = LevenbergMarquardt::new()
                        .with_tol(0.001)
                        .with_patience(20)
                        .minimize(adapter);
                    assert!(
                        matches!(report.termination, TerminationReason::User(_)),
                        "{kind}: {:?}",
                        report.termination
                    );
                    let error = adapter.error.borrow();
                    assert!(error.is_some(), "{kind}: retain the actual source refusal");
                    if kind == "cap" {
                        assert!(matches!(
                            error.as_ref(),
                            Some(ProblemError::Limit {
                                kind: native::LimitKind::Work,
                                ..
                            })
                        ));
                    }
                    if kind == "cancel" {
                        assert!(matches!(error.as_ref(), Some(ProblemError::Cancelled)));
                    }
                }
                Ok(())
            },
        )
        .await
        .unwrap();
}

async fn production_profile(
    prepared: &PreparedFit,
    mut profile: FitProfile,
) -> FixtureResult<PreparedFit> {
    profile.optimizer = FitOptimizer::Automatic;
    profile.solver.selection = native::solve::SolverSelection::Auto;
    profile.solver.controls.hessian = HessianMode::GaussNewton;
    Ok(prepared
        .source
        .prepare_fit(
            prepared.declaration().fit_id,
            profile,
            crate::workflow::tests::compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await?)
}
#[tokio::test]
async fn fitting_owned_residual_lm_uses_real_method_and_fresh_original_assessment() {
    for (scales, native_scaling) in [
        ([1., 2.], true),
        ([0.001, 1000.], true),
        ([0.001, 1000.], false),
    ] {
        let seed = fixture("linear", scales).await.unwrap();
        let mut profile = seed.problem.profile.clone();
        profile.solver.numerics.native_scaling = native_scaling;
        profile.uncertainty = Some(FitUncertainty {
            level: Fraction::try_new(0.95).unwrap(),
            profile: Some(Default::default()),
            predictions: false,
        });
        let prepared = production_profile(&seed, profile).await.unwrap();
        assert_eq!(prepared.route(), FitRoute::ResidualLm);
        assert_eq!(
            prepared.numerical_strategy().mechanisms[0]
                .profile
                .unwrap()
                .method,
            pse_model::strategy::NumericalMethod::ResidualLm
        );
        let result = prepared.start().unwrap().wait().await.unwrap();
        let report = regression::report(&result);
        assert!(report.solve.is_none());
        let residual = report.residual.as_ref().unwrap();
        assert!(residual.failure.is_none(), "{residual:?}");
        assert_eq!(
            residual.qualification,
            native::solve::Qualification::Stationary,
            "{residual:?}"
        );
        assert!(report.estimate_qualified(), "{report:?}");
        let (expected_covariance, estimate) = regression::analytic();
        let covariance = report.covariance.as_ref().unwrap();
        assert_eq!(
            covariance.approximation,
            pse_model::generated::enums::CovarianceApproximation::GaussNewton
        );
        let covariance = covariance.values.as_ref().unwrap();
        for (actual, expected) in covariance.iter().zip(expected_covariance) {
            assert!((actual - expected).abs() <= 0.001 * (1. + expected.abs()));
        }
        assert_eq!(report.wald.as_ref().unwrap().as_ref().unwrap().len(), 2);
        let objective = 0.5
            * (0..4)
                .map(|i| {
                    let residual = (estimate[0] + estimate[1] * regression::X[i]
                        - regression::Y[i])
                        / regression::SIGMA[i];
                    residual * residual
                })
                .sum::<f64>();
        assert!((report.objective.unwrap() - objective).abs() <= 0.001 * objective.max(1.));
        let completion = result.completion().unwrap();
        let computation = completion.computation.as_ref().unwrap();
        assert_eq!(
            computation.state,
            pse_model::generated::enums::NativeRunState::Native
        );
        assert_eq!(computation.backend, None);
        assert_eq!(computation.native_code, None);
        assert_eq!(
            computation.candidate_kind,
            Some(pse_model::generated::enums::NativeCandidateKind::FinalIterate)
        );
        assert!(
            computation
                .native_status
                .as_ref()
                .unwrap()
                .starts_with("residual_lm.")
        );
        assert!(matches!(
            report.execution_document(),
            FitExecutionDocument::ResidualLm { .. }
        ));
        for chain in report.profiles.as_ref().unwrap().as_ref().unwrap() {
            assert!(!chain.points.is_empty());
            assert!(chain.points.iter().all(|point| point.residual.is_some()));
        }
    }
}
#[tokio::test]
async fn fitting_residual_eligibility_preserves_required_native_contracts_before_callbacks() {
    let seed = fixture("linear", [1., 2.]).await.unwrap();
    let mut shared = seed.problem.profile.clone();
    shared.solver.selection = native::solve::SolverSelection::Auto;
    shared.solver.controls.hessian = HessianMode::GaussNewton;
    for intent in [
        SolveIntent::Root,
        SolveIntent::FeasiblePoint,
        SolveIntent::Initialize,
        SolveIntent::Certify,
    ] {
        shared.solver.intent = intent;
        assert!(
            !residual::request_eligible(&shared),
            "{intent:?} is a real requested purpose"
        );
    }
    shared.solver.intent = SolveIntent::Optimize;
    // The real typed request must not disappear into the residual binding.
    shared.solver.sensitivity = Some(crate::math::settings::SensitivityRequest {
        parameters: vec![crate::workflow::tests::id(1)],
        reduced_hessian: false,
        propagation: None,
    });
    assert!(!residual::request_eligible(&shared));
    shared.solver.sensitivity = None;
    shared.solver.composition.limits = Some(pse_model::strategy::WorkLimits {
        attempts: 1,
        evaluations: Some(256),
        iterations: None,
        factorizations: None,
        proof_steps: None,
    });
    assert!(
        residual::request_eligible(&shared),
        "actual residual/evaluation caps remain supported"
    );
    for request in [
        "required-presolve",
        "declared-composition",
        "exact-curvature",
        "explicit-backend",
        "native-iteration-cap",
        "native-factor-cap",
        "native-proof-cap",
    ] {
        let mut profile = seed.problem.profile.clone();
        profile.optimizer = FitOptimizer::Automatic;
        profile.solver.selection = native::solve::SolverSelection::Auto;
        profile.solver.controls.hessian = HessianMode::GaussNewton;
        match request {
            "required-presolve" => {
                profile.solver.presolve = native::presolve::Policy::from_native_options(
                    &Default::default(),
                    [native::presolve::Pass::AffineElimination].into(),
                )
                .unwrap()
            }
            "declared-composition" => {
                profile.solver.composition.policy = pse_model::strategy::CompositionPolicy::Declared
            }
            "exact-curvature" => profile.solver.controls.hessian = HessianMode::Exact,
            "native-iteration-cap" | "native-factor-cap" | "native-proof-cap" => {
                profile.solver.composition.limits = Some(pse_model::strategy::WorkLimits {
                    attempts: 1,
                    evaluations: None,
                    iterations: (request == "native-iteration-cap").then_some(64),
                    factorizations: (request == "native-factor-cap").then_some(64),
                    proof_steps: (request == "native-proof-cap").then_some(64),
                });
            }
            _ => {
                profile.solver.selection = native::solve::SolverSelection::Explicit(Backend::Ipopt)
            }
        }
        let native = seed
            .source
            .prepare_fit(
                seed.declaration().fit_id,
                profile.clone(),
                crate::workflow::tests::compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert!(matches!(native.route(), FitRoute::Native(_)), "{request}");
        profile.optimizer = FitOptimizer::ResidualLm;
        assert!(
            seed.source
                .prepare_fit(
                    seed.declaration().fit_id,
                    profile,
                    crate::workflow::tests::compiler_profile(),
                    Default::default(),
                    &crate::CancelSource::new()
                )
                .await
                .is_err(),
            "{request}"
        );
    }
}

/// Known dense workspace belongs to method eligibility before numerical products
/// or callbacks are prepared. The execution-time check remains a separate guard.
#[tokio::test]
async fn fitting_residual_dense_workspace_eligibility_precedes_numerical_preparation() {
    let seed = fixture("linear", [1., 2.]).await.unwrap();
    let m = seed
        .problem
        .measurements
        .iter()
        .filter(|m| m.included)
        .count();
    let n = seed.problem.contract.variables.len();
    let bytes = residual::matrix_bytes(m, n).unwrap();
    assert!(!residual::workspace_eligible(
        usize::MAX,
        2,
        usize::MAX,
        usize::MAX
    ));
    for allowance in [bytes - 1, bytes] {
        let mut profile = seed.problem.profile.clone();
        profile.optimizer = FitOptimizer::Automatic;
        profile.solver.selection = native::solve::SolverSelection::Auto;
        profile.solver.controls.hessian = HessianMode::GaussNewton;
        profile.solver.controls.foreign_bytes = Some(allowance);
        let prepared = seed
            .source
            .prepare_fit(
                seed.declaration().fit_id,
                profile.clone(),
                crate::workflow::tests::compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let Experiment::Steady(source) = &prepared.problem.experiments[0] else {
            panic!("steady fixture");
        };
        if allowance < bytes {
            assert!(matches!(prepared.route(), FitRoute::Native(_)));
            assert_eq!(
                source.case.assembly.order(),
                DerivativeOrder::Second,
                "NLP constraint curvature preparation must survive an ineligible dense LM route"
            );
        } else {
            assert_eq!(prepared.route(), FitRoute::ResidualLm);
            assert_eq!(source.case.assembly.order(), DerivativeOrder::First);
        }
        profile.optimizer = FitOptimizer::ResidualLm;
        let explicit = seed
            .source
            .prepare_fit(
                seed.declaration().fit_id,
                profile,
                crate::workflow::tests::compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await;
        if allowance < bytes {
            assert!(
                explicit
                    .unwrap_err()
                    .to_string()
                    .contains("dense workspace")
            );
        } else {
            assert_eq!(explicit.unwrap().route(), FitRoute::ResidualLm);
        }
    }
}

/// The physical assembly is unchanged while pins/free maps and current scopes are
/// rebuilt. Only the actual callback Drop releases its numeric buffers for reuse.
#[tokio::test]
async fn fit_neutral_scratch_returns_after_drop_and_rebinds_current_maps_and_controls() {
    let seed = fixture("linear", [1., 2.]).await.unwrap();
    let prepared = production_profile(&seed, seed.problem.profile.clone())
        .await
        .unwrap();
    let problem = prepared.problem.clone();
    let incompatible = fixture("domain", [1., 2.]).await.unwrap().problem.clone();
    let service = problem.runtime.native().clone();
    service
        .job_scoped(
            1,
            problem.bytes + MATRIX_ALLOWANCE,
            Default::default(),
            None,
            move |_| {
                let baseline = problem.runtime.native().test_admission_snapshot().0;
                let bank =
                    FitScratch::new(&problem).expect("the finite neutral population is admitted");
                let charged = problem.runtime.native().test_admission_snapshot().0;
                assert!(charged > baseline);
                let pointer = |oracle: &mut FitOracle, x: &[f64]| -> Result<usize, ProblemError> {
                    let Experiment::Steady(steady) = &problem.experiments[0] else {
                        panic!("steady fixture");
                    };
                    let mut values = steady.values.clone();
                    for &(id, column) in &steady.coordinates {
                        values.scalars.insert(id, x[column.get()]);
                    }
                    Ok(oracle.workers[0]
                        .as_mut()
                        .unwrap()
                        .jacobian(&values)?
                        .val()
                        .as_ptr() as usize)
                };
                let current = || {
                    let mut execution =
                        Execution::new(Default::default(), &problem.profile.solver.controls);
                    execution.memory = Some(MATRIX_ALLOWANCE);
                    execution
                };
                let prior = current();
                let mut first =
                    FitOracle::with_scratch(problem.clone(), prior.clone(), Some(bank.clone()))?;
                let value = first.objective(&[1., 2.])?;
                assert!(value.is_finite());
                assert_eq!(bank.steady_reuses(), 0);
                // A second callback while the first remains live gets its own fresh worker.
                let mut other =
                    FitOracle::with_scratch(problem.clone(), current(), Some(bank.clone()))?;
                assert_eq!(bank.steady_reuses(), 0);
                let returned_pointer = pointer(&mut other, &[1., 2.])?;
                drop(other);
                drop(first);
                prior.cancellation_control().cancel();
                drop(prior);
                let mut final_oracle =
                    FitOracle::with_scratch(problem.clone(), current(), Some(bank.clone()))?;
                assert_eq!(bank.steady_reuses(), 1);
                let actual = final_oracle.objective(&[2., 0.5])?;
                let (fresh, _) = fresh(problem.clone(), current(), &[2., 0.5])?;
                assert_eq!(
                    actual, fresh,
                    "no previous Point or old cancellation may survive reset"
                );
                assert_eq!(pointer(&mut final_oracle, &[2., 0.5])?, returned_pointer);
                assert_eq!(
                    problem.runtime.native().test_admission_snapshot().0,
                    charged
                );
                assert_ne!(actual, value);
                drop(final_oracle);
                let roles = &problem.declaration.experiments;
                let Experiment::Steady(different) = &incompatible.experiments[0] else {
                    panic!("steady fixture");
                };
                assert!(
                    bank.steady(roles[0].experiment_id, &different.case.assembly)
                        .is_none()
                );
                // Equal-length pins change the actual free-coordinate map. Reuse is tied to
                // the physical assembly, while every residual adapter rebuilds that map.
                for pin in [(0, 1.), (1, 2.)] {
                    let (evidence, candidate) = residual::solve(
                        &problem,
                        current(),
                        &[1., 1.],
                        Some(pin),
                        Some(bank.clone()),
                    )?;
                    assert!(evidence.failure.is_none(), "{evidence:?}");
                    assert_eq!(candidate[pin.0], pin.1);
                    let (objective, gradient, feasible) = residual::assess_pin(
                        &problem,
                        current(),
                        &candidate,
                        pin.0,
                        Some(bank.clone()),
                    )?;
                    assert!(
                        objective.is_finite()
                            && feasible
                            && gradient <= problem.accuracy.stationarity
                    );
                }
                assert!(bank.steady_reuses() >= 5);
                assert_eq!(
                    problem.runtime.native().test_admission_snapshot().0,
                    charged,
                    "fixed physical slots must not grow with the pin population"
                );
                // An actual current derivative refusal discards the whole optional bank;
                // it never makes an old numerical point available to another callback.
                let mut failed =
                    FitOracle::with_scratch(problem.clone(), current(), Some(bank.clone()))?;
                assert!(
                    failed
                        .evaluate_demand(&[f64::NAN, 1.], Demand::responses())
                        .is_err()
                );
                drop(failed);
                let Experiment::Steady(steady) = &problem.experiments[0] else {
                    panic!("steady fixture");
                };
                assert!(
                    bank.steady(roles[0].experiment_id, &steady.case.assembly)
                        .is_none()
                );
                drop(bank);
                assert_eq!(
                    problem.runtime.native().test_admission_snapshot().0,
                    baseline
                );
                Ok(())
            },
        )
        .await
        .unwrap();
}

/// Matched actual public operations, including independent final assessment,
/// report derivation, source physical checks and publication/join. Timing remains
/// observed evidence: no noisy wall-clock threshold changes scientific acceptance.
#[tokio::test]
async fn fitting_residual_production_matches_native_complete_operations_at_original_accuracy() {
    for (kind, scales) in [
        ("linear", [1., 2.]),
        ("linear", [0.001, 1000.]),
        ("near-collinear", [1., 2.]),
        ("domain", [1., 2.]),
    ] {
        let seed = fixture(kind, scales).await.unwrap();
        let mut objectives = Vec::new();
        for round in 0..2 {
            let mut routes = vec![
                (FitOptimizer::Automatic, HessianMode::GaussNewton),
                (FitOptimizer::NativeNlp, HessianMode::GaussNewton),
                (FitOptimizer::NativeNlp, HessianMode::Exact),
                (FitOptimizer::NativeNlp, HessianMode::LimitedMemory),
            ];
            if round == 1 {
                routes.reverse();
            }
            for (optimizer, curvature) in routes {
                let mut profile = seed.problem.profile.clone();
                profile.optimizer = optimizer;
                profile.solver.selection = native::solve::SolverSelection::Auto;
                profile.solver.controls.hessian = curvature;
                profile.uncertainty = Some(FitUncertainty {
                    level: Fraction::try_new(0.95).unwrap(),
                    profile: None,
                    predictions: true,
                });
                let started = Instant::now();
                let prepared = seed
                    .source
                    .prepare_fit(
                        seed.declaration().fit_id,
                        profile,
                        crate::workflow::tests::compiler_profile(),
                        Default::default(),
                        &crate::CancelSource::new(),
                    )
                    .await
                    .unwrap();
                let preparation = started.elapsed();
                let route = prepared.route();
                let result = prepared.start().unwrap().wait().await.unwrap();
                let elapsed = started.elapsed();
                let report = regression::report(&result);
                assert!(
                    report.checks_complete && report.checks.iter().all(|check| check.satisfied)
                );
                assert!(report.validation_error.is_none(), "{report:?}");
                assert!(report.quality.as_ref().unwrap().feasible(), "{report:?}");
                let covariance = report.covariance.as_ref().unwrap();
                assert!(
                    covariance
                        .values
                        .as_ref()
                        .unwrap()
                        .iter()
                        .all(|value| value.is_finite()),
                    "{covariance:?}"
                );
                assert_eq!(report.wald.as_ref().unwrap().as_ref().unwrap().len(), 2);
                let objective = report.objective.unwrap();
                assert!(objective.is_finite() && objective > 0.);
                if optimizer == FitOptimizer::Automatic {
                    assert_eq!(route, FitRoute::ResidualLm);
                    let evidence = report.residual.as_ref().unwrap();
                    assert!(evidence.failure.is_none(), "{evidence:?}");
                    assert_eq!(
                        evidence.qualification,
                        native::solve::Qualification::Stationary,
                        "{evidence:?}"
                    );
                    assert!(evidence.stationarity.unwrap() <= seed.problem.accuracy.stationarity);
                } else {
                    assert!(report.residual.is_none());
                    let native = report.solve.as_ref().unwrap();
                    assert!(
                        native.callback_failure().is_none()
                            && native.validation_failure().is_none(),
                        "{native:?}"
                    );
                }
                let QuadraticWitness {
                    best,
                    gap,
                    scaled,
                    gradient,
                } = quadratic_witness(
                    kind,
                    &seed.problem,
                    report.candidate.as_ref().unwrap(),
                    objective,
                );
                println!(
                    "fit_quadratic kind={kind} x={:?} physical_gradient={gradient:?} scaled_stationarity={scaled} analytic_best={best} exact_gap={gap}",
                    report.candidate.as_ref().unwrap()
                );
                assert!(
                    scaled <= seed.problem.accuracy.stationarity,
                    "{kind}: {scaled}"
                );
                if optimizer == FitOptimizer::Automatic {
                    assert!(
                        gap <= 0.001 * (1. + best.abs()),
                        "LM independent original optimum: {kind}"
                    );
                    objectives.push(objective);
                } else if gap > 0.001 * (1. + best.abs()) {
                    assert_eq!(
                        (kind, scales, curvature),
                        ("linear", [0.001, 1000.], HessianMode::LimitedMemory),
                        "only the causally established scaled-stationarity/objective-gap mismatch is expected"
                    );
                    println!(
                        "fit_objective_classification=stationary_without_objective_gap_guarantee"
                    );
                } else {
                    objectives.push(objective);
                }
                println!(
                    "fit_complete kind={kind} scales={scales:?} round={round} route={route:?} curvature={curvature:?} objective={objective} preparation_ms={} total_ms={} lm={:?} native_work={:?}",
                    preparation.as_secs_f64() * 1000.,
                    elapsed.as_secs_f64() * 1000.,
                    report.residual,
                    report.solve.as_ref().map(|report| report.evidence.work)
                );
            }
        }
        let reference = objectives[0];
        assert!(
            objectives
                .iter()
                .all(|value| (value - reference).abs() <= 0.001 * (1. + reference.abs())),
            "{kind}: {objectives:?}"
        );
    }
}

#[tokio::test]
async fn fit_neutral_scratch_discards_after_actual_derivative_upgrade_refusal() {
    let seed = fixture("domain", [1., 2.]).await.unwrap();
    let prepared = production_profile(&seed, seed.problem.profile.clone())
        .await
        .unwrap();
    let problem = prepared.problem.clone();
    let service = problem.runtime.native().clone();
    service
        .job_scoped(
            1,
            problem.bytes + MATRIX_ALLOWANCE,
            Default::default(),
            None,
            move |flag| {
                let bank = FitScratch::new(&problem).unwrap();
                let execution = Execution::new(flag, &problem.profile.solver.controls);
                let mut oracle = FitOracle::with_scratch(
                    problem.clone(),
                    execution.clone(),
                    Some(bank.clone()),
                )?;
                let value = oracle.objective(&[0., 1.])?;
                assert!(
                    value.is_finite(),
                    "sqrt(0) retains its independent Value meaning"
                );
                assert!(
                    oracle
                        .evaluate_demand(&[0., 1.], Demand::responses())
                        .is_err()
                );
                drop(oracle);
                let Experiment::Steady(steady) = &problem.experiments[0] else {
                    panic!("steady fixture");
                };
                assert!(
                    bank.steady(
                        problem.declaration.experiments[0].experiment_id,
                        &steady.case.assembly
                    )
                    .is_none()
                );
                let mut fresh =
                    FitOracle::with_scratch(problem.clone(), execution, Some(bank.clone()))?;
                assert!(fresh.objective(&[1., 1.])?.is_finite());
                assert_eq!(
                    bank.steady_reuses(),
                    0,
                    "failed derivative buffers cannot be offered to the next oracle"
                );
                Ok(())
            },
        )
        .await
        .unwrap();
}

/// The same public fixture crosses the actual canonical result boundary. Reopening
/// reads stored method/status/work; it never reconstructs a synthetic FitReport.
#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn fitting_residual_lm_public_result_reopens_actual_method_termination_and_profile() {
    let runtime = crate::workflow::tests::durable_runtime();
    let cancel = crate::CancelSource::new();
    let (documents, _) = crate::workflow::worker_tests::sources("package p {}");
    let physical = Box::pin(runtime.physical_from_sources(&documents, &cancel))
        .await
        .unwrap();
    let mut seed = fixture_with_physical("linear", [1., 2.], runtime.clone(), physical)
        .await
        .unwrap();
    let store = runtime.canonical_store().unwrap();
    store
        .create_project("fit-project-create", "fit-project", "Original fit context")
        .await
        .unwrap();
    let project = store.project("fit-project").await.unwrap().unwrap();
    let revision = Box::pin(runtime.project_fork(
        &project,
        "fit-branch",
        "fit-source",
        "fit-fork",
        &seed.source,
    ))
    .await
    .unwrap();
    let source = Box::pin(runtime.modeling_revision(
        revision,
        seed.source.physical.clone(),
        Default::default(),
    ))
    .await
    .unwrap();
    let project = store.project("fit-project").await.unwrap().unwrap();
    Box::pin(runtime.project_record_context(
        "fit-original",
        &project,
        "calibration",
        &source,
        0,
        None,
        None,
        false,
    ))
    .await
    .unwrap();
    let cut = store.project_context_cut("fit-project").await.unwrap();
    let selected = store
        .select_project_context(&cut, "calibration", 10)
        .await
        .unwrap()
        .unwrap();
    seed.source = Box::pin(runtime.project_context_reopen(&selected, &cancel))
        .await
        .unwrap();
    let prepared = production_profile(&seed, seed.problem.profile.clone())
        .await
        .unwrap();
    let strategy = prepared.numerical_strategy();
    assert_eq!(
        strategy.mechanisms[0].profile.unwrap().method,
        pse_model::strategy::NumericalMethod::ResidualLm
    );
    let handle = prepared.start().unwrap();
    let result = handle.wait().await.unwrap();
    let report = regression::report(&result);
    let FitExecutionDocument::ResidualLm { report: literal } = report.execution_document() else {
        panic!("real LM descriptor");
    };
    assert!(literal.failure.is_none());
    assert_eq!(
        literal.qualification,
        native::solve::Qualification::Stationary
    );
    let status = result
        .completion()
        .unwrap()
        .computation
        .as_ref()
        .unwrap()
        .native_status
        .clone();
    let termination = serde_json::to_string(&literal.termination).unwrap();
    let stopping = literal.stopping.unwrap();
    let (run, attempt) = (
        handle.canonical_run_key().unwrap().to_owned(),
        handle.canonical_attempt_key().unwrap().to_owned(),
    );
    drop(result);
    drop(handle);
    let reopened = runtime
        .operations()
        .unwrap()
        .record(&run, &attempt)
        .await
        .unwrap();
    let stored = reopened
        .completion
        .as_ref()
        .unwrap()
        .completion
        .as_ref()
        .unwrap();
    let computation = stored.computation.as_ref().unwrap();
    assert_eq!(
        computation.state,
        pse_model::generated::enums::NativeRunState::Native
    );
    assert_eq!(computation.backend, None);
    assert_eq!(computation.native_code, None);
    assert_eq!(computation.native_status, status);
    // The canonical receipt wraps the exact runtime selection bytes together
    // with its immutable source list; decode both actual receiving contracts.
    type SourceRevision = (String, String, u64, String);
    type RecordedSelection = (u8, Vec<u8>, Vec<SourceRevision>);
    let (receipt_version, selection, _sources): RecordedSelection =
        serde_json::from_slice(&reopened.run.as_ref().unwrap().source_selection).unwrap();
    assert_eq!(receipt_version, 1);
    let (selection_version, _identity, _physical, strategies, sources): (
        u8,
        serde_json::Value,
        ContentHash,
        Vec<pse_model::strategy::NumericalStrategy>,
        Vec<crate::workflow::OperationSource>,
    ) = serde_json::from_slice(&selection).unwrap();
    assert_eq!(selection_version, 3);
    assert_eq!(sources.len(), 1);
    assert_eq!(
        sources[0],
        crate::workflow::OperationSource::of(&prepared.source)
    );
    Box::pin(runtime.project_record_context(
        "fit-corrected",
        &project,
        "calibration",
        &source,
        0,
        None,
        Some("fit-original"),
        false,
    ))
    .await
    .unwrap();
    let latest = store.project_context_cut("fit-project").await.unwrap();
    assert_eq!(
        store
            .select_project_context(&latest, "calibration", 10)
            .await
            .unwrap()
            .unwrap()
            .context
            .key,
        "fit-corrected"
    );
    assert_eq!(
        sources[0].context.as_ref(),
        Some(&selected),
        "actual reopened fit receipt retains its original context after correction"
    );
    let after_correction = runtime
        .operations()
        .unwrap()
        .record(&run, &attempt)
        .await
        .unwrap();
    assert_eq!(
        after_correction.run.as_ref().unwrap().source_selection,
        reopened.run.as_ref().unwrap().source_selection,
        "later historical correction does not rewrite the stored fit consumption"
    );
    assert_eq!(strategies.len(), 1);
    let stored_strategy = &strategies[0];
    assert_eq!(
        stored_strategy.mechanisms[0].profile.unwrap().method,
        pse_model::strategy::NumericalMethod::ResidualLm
    );
    let mut cursor = runtime
        .results(
            &run,
            &attempt,
            "runtime.solve_metrics",
            0,
            u64::MAX,
            pse_columnar::CancellationToken::new(),
        )
        .await
        .unwrap();
    let mut metrics = Vec::new();
    while let Some(batch) = cursor.next_batch().await.unwrap() {
        metrics.extend(pse_relations::generated::runtime::solve_metrics::RuntimeSolveMetricsView::try_from_batch_with_registry(
            runtime.registry(), &batch, &runtime.validation_context()).unwrap().rows().unwrap());
    }
    assert!(metrics.iter().any(|row| row.namespace == "native"
        && row.name == "method"
        && row.text.as_deref() == Some("residual_lm")));
    assert!(metrics.iter().any(|row| row.namespace == "residual_lm"
        && row.name == "termination"
        && row.text.as_deref() == Some(termination.as_str())));
    assert!(!metrics.iter().any(|row| row.namespace == "native"
        && row.name == "backend"
        && row.text.as_deref() == Some("ipopt")));
    for (name, value) in [
        (
            "relative_cost_change",
            stopping.criteria.relative_cost_change(),
        ),
        ("relative_step", stopping.criteria.relative_step()),
        (
            "column_residual_cosine",
            stopping.criteria.column_residual_cosine(),
        ),
    ] {
        assert!(metrics.iter().any(|row| row.namespace == "residual_lm"
            && row.name == name
            && row.real.map(f64::to_bits) == Some(value.to_bits())));
    }
    for (name, value) in [
        ("criteria_version", i64::from(stopping.criteria.version())),
        (
            "parameter_update_limit",
            i64::try_from(stopping.parameter_update_limit).unwrap(),
        ),
        (
            "residual_evaluation_limit",
            i64::try_from(stopping.residual_evaluation_limit).unwrap(),
        ),
    ] {
        assert!(metrics.iter().any(|row| row.namespace == "residual_lm"
            && row.name == name
            && row.integer == Some(value)));
    }
}

#[tokio::test]
async fn fitting_lm_independent_stopping_preserves_original_qualification_and_covariance() {
    for (kind, scales) in [
        ("linear", [1., 2.]),
        ("linear", [0.001, 1000.]),
        ("near-collinear", [1., 2.]),
        ("domain", [1., 2.]),
        ("cosine-gap", [1., 2.]),
    ] {
        let seed = fixture(kind, scales).await.unwrap();
        let prepared = production_profile(&seed, seed.problem.profile.clone())
            .await
            .unwrap();
        let problem = prepared.problem.clone();
        let service = problem.runtime.native().clone();
        service.job_scoped(1, problem.bytes + MATRIX_ALLOWANCE, Default::default(), None, move |flag| {
            let execution = Execution::new(flag, &problem.profile.solver.controls);
            let mut baseline_qualified = false;
            let mut baseline_covariance = false;
            for (candidate, stopping) in [(false, ResidualLmCriteria::inquiry_baseline(problem.accuracy.stationarity)), (true, ResidualLmCriteria::current())] {
                let report = problem.execute_profile_with_lm(&problem.profile.solver, ProfileAlgorithm { route: FitRoute::ResidualLm, stopping },
                    execution.scope()?, Arc::new(native::solve::Progress::new(256)), 1, None)?;
                let residual = report.residual.as_ref().unwrap();
                assert!(residual.failure.is_none(), "{kind}: {residual:?}");
                let point = report.candidate.as_ref().unwrap();
                let (objective, _) = fresh(problem.clone(), execution.clone(), point)?;
                let witness = quadratic_witness(kind, &problem, point, objective);
                assert!((witness.scaled - residual.stationarity.unwrap()).abs() <= 1e-7 * witness.scaled.max(1.));
                let qualified = residual.qualification == native::solve::Qualification::Stationary;
                let covariance = report.covariance.as_ref().is_some_and(|covariance| covariance.values.is_ok());
                println!("LM stopping kind={kind} scales={scales:?} candidate={candidate} literal={:?} original_stationarity={} qualified={qualified} covariance={covariance} residual_calls={} jacobian_calls={} updates={}",
                    residual.termination, witness.scaled, residual.residual_evaluations, residual.jacobian_evaluations, residual.parameter_updates);
                if candidate {
                    assert!(!baseline_qualified || qualified, "candidate stopping loses an originally qualified {kind} fit");
                    assert!(!baseline_covariance || covariance, "candidate stopping newly withholds {kind} covariance");
                } else {
                    baseline_qualified = qualified;
                    baseline_covariance = covariance;
                }
            }
            Ok(())
        }).await.unwrap();
    }
}

#[tokio::test]
async fn fitting_lm_cosine_success_cannot_publish_nonstationary_covariance() {
    let seed = fixture("cosine-gap", [1., 2.]).await.unwrap();
    let prepared = production_profile(&seed, seed.problem.profile.clone())
        .await
        .unwrap();
    let problem = prepared.problem.clone();
    let service = problem.runtime.native().clone();
    service
        .job_scoped(
            1,
            problem.bytes + MATRIX_ALLOWANCE,
            Default::default(),
            None,
            move |flag| {
                let execution = Execution::new(flag, &problem.profile.solver.controls);
                let report = problem.execute_profile_with_lm(
                    &problem.profile.solver,
                    ProfileAlgorithm {
                        route: FitRoute::ResidualLm,
                        stopping: ResidualLmCriteria::current(),
                    },
                    execution.scope()?,
                    Arc::new(native::solve::Progress::new(256)),
                    1,
                    None,
                )?;
                let residual = report.residual.as_ref().unwrap();
                assert!(residual.failure.is_none());
                assert_eq!(residual.termination, FitResidualTermination::Orthogonal);
                assert!(residual.stationarity.unwrap() > problem.accuracy.stationarity);
                assert_eq!(
                    residual.qualification,
                    native::solve::Qualification::Unqualified
                );
                assert!(report.covariance.as_ref().unwrap().values.is_err());
                Ok(())
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn fitting_lm_current_policy_limits_and_cancellation_remain_typed() {
    let seed = fixture("linear", [1., 2.]).await.unwrap();
    let mut profile = seed.problem.profile.clone();
    profile.solver.controls.iterations = 1;
    let prepared = production_profile(&seed, profile).await.unwrap();
    let problem = prepared.problem.clone();
    let service = problem.runtime.native().clone();
    service
        .job_scoped(
            1,
            problem.bytes + MATRIX_ALLOWANCE,
            Default::default(),
            None,
            move |flag| {
                let execution = Execution::new(flag, &problem.profile.solver.controls);
                let report = problem.execute_profile(
                    &problem.profile.solver,
                    FitRoute::ResidualLm,
                    execution.scope()?,
                    Arc::new(native::solve::Progress::new(256)),
                    1,
                    None,
                )?;
                let residual = report.residual.as_ref().unwrap();
                let stopping = residual.stopping.unwrap();
                assert_eq!(stopping.criteria, ResidualLmCriteria::current());
                assert_eq!(stopping.parameter_update_limit, 1);
                assert_eq!(
                    stopping.residual_evaluation_limit,
                    problem.initial.len() + 1
                );
                assert!(residual.parameter_updates <= stopping.parameter_update_limit);
                assert!(residual.residual_evaluations <= stopping.residual_evaluation_limit);
                assert!(matches!(
                    residual.failure.as_ref().map(ProfileFailure::cause),
                    Some(ProblemError::Limit {
                        kind: native::LimitKind::Work,
                        ..
                    })
                ));
                let cancelled =
                    Execution::new(Default::default(), &problem.profile.solver.controls);
                cancelled.cancellation_control().cancel();
                assert!(matches!(
                    residual::solve(&problem, cancelled, &problem.initial, None, None),
                    Err(ProblemError::Cancelled)
                ));
                Ok(())
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn fitting_lm_recorded_criteria_bind_replay_and_actual_profile_identity() {
    let seed = fixture("linear", [1., 2.]).await.unwrap();
    let prepared = production_profile(&seed, seed.problem.profile.clone())
        .await
        .unwrap();
    let current = ResidualLmCriteria::current();
    let constructor = prepared.original_inputs.document(0);
    assert_eq!(constructor.residual_lm_criteria, current);
    let serialized = serde_json::to_value(&constructor).unwrap();
    assert_eq!(serialized["residual_lm_criteria"]["version"], 1);
    assert_eq!(
        serialized["residual_lm_criteria"]["relative_cost_change"],
        current.relative_cost_change()
    );
    assert_eq!(
        serialized["residual_lm_criteria"]["relative_step"],
        current.relative_step()
    );
    assert_eq!(
        serialized["residual_lm_criteria"]["column_residual_cosine"],
        current.column_residual_cosine()
    );
    let schema = schemars::schema_for!(crate::workflow::ReplayFitConstructor).to_value();
    assert!(
        schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == "residual_lm_criteria")
    );
    assert_eq!(
        prepared.problem.profile_key,
        preparation::profile_key(&prepared.problem.profile, current).unwrap()
    );
    let baseline = ResidualLmCriteria::inquiry_baseline(prepared.problem.accuracy.stationarity);
    assert_ne!(
        prepared.problem.profile_key,
        preparation::profile_key(&prepared.problem.profile, baseline).unwrap()
    );
    let replayed = constructor
        .reconstruct(&prepared.source, &crate::CancelSource::new())
        .await
        .unwrap();
    assert_eq!(replayed.identity(), prepared.identity());
    assert_eq!(replayed.problem.residual_lm_criteria, current);
    let mut unsupported = constructor;
    unsupported.residual_lm_criteria = baseline;
    assert!(
        unsupported
            .reconstruct(&prepared.source, &crate::CancelSource::new())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn fitting_lm_profile_chains_record_actual_criteria_and_effective_caps() {
    let seed = fixture("linear", [1., 2.]).await.unwrap();
    let mut profile = seed.problem.profile.clone();
    profile.uncertainty = Some(FitUncertainty {
        level: Fraction::try_new(0.95).unwrap(),
        profile: Some(Default::default()),
        predictions: false,
    });
    let prepared = production_profile(&seed, profile).await.unwrap();
    let updates = prepared.problem.profile.solver.controls.iterations as usize;
    let result = prepared.start().unwrap().wait().await.unwrap();
    let report = regression::report(&result);
    let estimate = report.residual.as_ref().unwrap().stopping.unwrap();
    assert_eq!(estimate.criteria, ResidualLmCriteria::current());
    assert_eq!(estimate.parameter_update_limit, updates);
    assert_eq!(
        estimate.residual_evaluation_limit,
        updates * (prepared.problem.initial.len() + 1)
    );
    for chain in report.profiles.as_ref().unwrap().as_ref().unwrap() {
        assert!(!chain.points.is_empty());
        for point in &chain.points {
            let residual = point.residual.as_ref().unwrap();
            let stopping = residual.stopping.unwrap();
            assert_eq!(stopping.criteria, estimate.criteria);
            assert_eq!(stopping.parameter_update_limit, updates);
            assert_eq!(
                stopping.residual_evaluation_limit,
                updates * prepared.problem.initial.len()
            );
            assert!(residual.parameter_updates <= stopping.parameter_update_limit);
            assert!(residual.residual_evaluations <= stopping.residual_evaluation_limit);
        }
    }
    let FitExecutionDocument::ResidualLm { report: execution } = report.execution_document() else {
        panic!("actual residual method");
    };
    assert_eq!(execution.stopping, Some(estimate));
}

/// SC01-SC03: authored affine calibration, immutable research selection, actual LM,
/// reopened qualified export and ordinary source publication are one composed journey.
#[cfg(feature="canonical-tests")]
#[tokio::test]
async fn research_calibrated_fit_reopens_and_publishes_original_cells() {
    use crate::workflow::{ResearchDataObject,ResearchDatasetRequest,ResearchSelectionRequest,ResearchObservation,ResearchPublicationRequest,ResearchParameterTarget};
    use crate::workflow::tests::{id,compiler_profile};
    use pse_relations::{columnar::RelationRow,generated::runtime::fitted_parameter_cells as cells};
    let runtime=crate::workflow::tests::durable_runtime();let cancel=crate::CancelSource::new();
    let (documents,_)=crate::workflow::worker_tests::sources("package p {}");
    let physical=Box::pin(runtime.physical_from_sources(&documents,&cancel)).await.unwrap();
    let mut seed=fixture_with_physical("research-calibration",[1.,2.],runtime.clone(),physical).await.unwrap();
    let store=runtime.canonical_store().unwrap();
    store.create_project("create","research","Calibration publication").await.unwrap();
    let project=store.project("research").await.unwrap().unwrap();
    let revision=Box::pin(runtime.project_fork(&project,"main","research-source","fork",&seed.source)).await.unwrap();
    let source=Box::pin(runtime.modeling_revision(revision,seed.source.physical.clone(),Default::default())).await.unwrap();
    let project=store.project("research").await.unwrap().unwrap();
    Box::pin(runtime.project_record_context("original-context",&project,"calibration",&source,0,None,None,false)).await.unwrap();
    let cut=store.project_context_cut("research").await.unwrap();
    let context=store.select_project_context(&cut,"calibration",10).await.unwrap().unwrap();
    let root=seed.declaration().experiments[0].case_id;
    let quantity=source.physical.quantities.neutral_dimensionless().unwrap();
    let unit=source.physical.quantities.quantity_type(quantity).unwrap().canonical_unit;
    let rows=source.declarations().await.unwrap();
    let derived=rows.iter().find(|r|r.name=="calibrated").unwrap().declaration_id;
    let sigma=rows.iter().find(|r|r.name=="calibrated_sigma").unwrap().declaration_id;
    let members=(0..4).map(|i|ResearchDataObject{root,record:id(40+i).into(),attribute:"raw".into(),standard_deviation_attribute:Some("sigma".into()),quantity:quantity.as_id(),acquisition_clock:"synthetic acquisition sequence, no wall-clock accuracy claim".into(),parent_member:None,calibration:vec![]}).collect::<Vec<_>>();
    let raw=ResearchDatasetRequest{key:"raw-v1".into(),project:project.clone(),context:context.clone(),parent:None,role:"raw".into(),members};
    let raw_row=Box::pin(runtime.research_dataset_create(&raw,&cancel)).await.unwrap();
    assert_eq!(Box::pin(runtime.research_dataset_create(&raw,&cancel)).await.unwrap(),raw_row);
    let mut wrong=raw.clone();wrong.key="wrong-physical".into();wrong.members[0].quantity=id(99);
    assert!(Box::pin(runtime.research_dataset_create(&wrong,&cancel)).await.is_err());
    let mut calibrated=raw.clone();calibrated.key="calibrated-v1".into();calibrated.role="derived".into();calibrated.parent=Some(raw.key.clone());
    for (i,m) in calibrated.members.iter_mut().enumerate(){m.parent_member=Some(i as u64);m.attribute="calibrated".into();m.standard_deviation_attribute=Some("calibrated_sigma".into());m.calibration=vec![derived,sigma];}
    Box::pin(runtime.research_dataset_create(&calibrated,&cancel)).await.unwrap();
    let selection=ResearchSelectionRequest{key:"selection-v1".into(),project:project.clone(),context:context.clone(),dataset:calibrated.key.clone(),fit:seed.declaration().fit_id,observations:(0..4).map(|i|ResearchObservation{observation:id(40+i).into(),member:i as u64,reason:"included: y=2x+3; explicitly authored sigma'=2sigma".into()}).collect()};
    let mut partial=selection.clone();partial.key="partial".into();partial.observations.pop();
    assert!(Box::pin(runtime.research_selection_create(&partial,&cancel)).await.is_err());
    Box::pin(runtime.research_selection_create(&selection,&cancel)).await.unwrap();
    let (selected,fit)=Box::pin(runtime.research_selection_open(&selection.key,&cancel)).await.unwrap();
    seed.source=selected;
    let prepared=production_profile(&seed,seed.problem.profile.clone()).await.unwrap();
    for (i,measurement) in prepared.problem.measurements.iter().enumerate(){
        assert_eq!(measurement.value,Some(2.*(i+1) as f64+3.));
        assert_eq!(measurement.sigma,Some(2.*regression::SIGMA[i]));
        let (residual,_)=FitOracle::residual(measurement,measurement.value.unwrap()+1.).unwrap();
        assert_eq!(residual,1./(2.*regression::SIGMA[i]),"actual weighted residual uses transformed standard uncertainty");
    }
    let handle=prepared.start().unwrap();let result=handle.wait().await.unwrap();
    assert!(regression::report(&result).estimate_qualified());
    let exported=cells::Row::rows(&result.export_fit_parameters().unwrap()).unwrap();
    assert_eq!(exported.len(),2);
    for c in &exported{let expected=if c.parameter_id==id(1){3.}else{2.};assert!((c.value-expected).abs()<1e-7);}
    let run=handle.canonical_run_key().unwrap().to_owned();let attempt=handle.canonical_attempt_key().unwrap().to_owned();
    let header=store.canonical_run(&run).await.unwrap().unwrap();
    assert_eq!(header.research_selection.as_deref(),Some("selection-v1"));
    assert_eq!(header.research_dataset.as_deref(),Some("calibrated-v1"));
    drop(result);drop(handle);drop(prepared);
    let mut reopened=runtime.results(&run,&attempt,"runtime.fitted_parameter_cells",0,u64::MAX,cancel.token()).await.unwrap();
    let mut actual=vec![];while let Some(batch)=reopened.next_relation_batch().await.unwrap(){actual.extend(cells::Row::rows(&batch).unwrap());}
    assert_eq!(actual,exported,"original producer-qualified cells survive dropping the live result");
    let branch=store.project_branch("main").await.unwrap().unwrap();
    let targets=[("a",id(1)),("b",id(2))].into_iter().map(|(name,parameter)|ResearchParameterTarget{parameter,declaration:rows.iter().find(|r|r.name==name).unwrap().declaration_id,root,instance:id(33).into(),path:name.into(),quantity:quantity.as_id(),unit:unit.as_id(),lower_bits:Some((-100f64).to_bits()),upper_bits:Some(100f64.to_bits())}).collect();
    let request=ResearchPublicationRequest{operation:"publish-fit".into(),selection:selection.key.clone(),run:run.clone(),attempt:attempt.clone(),project:project.clone(),branch,target:context.clone(),targets};
    let mut wrong=request.clone();wrong.attempt="absent-attempt".into();assert!(Box::pin(runtime.research_publication_preview(&wrong,&cancel)).await.is_err());
    let mut wrong=request.clone();wrong.targets[0].unit=id(99);assert!(Box::pin(runtime.research_publication_preview(&wrong,&cancel)).await.is_err());
    let mut wrong=request.clone();wrong.targets[0].upper_bits=Some(0f64.to_bits());assert!(Box::pin(runtime.research_publication_preview(&wrong,&cancel)).await.is_err());
    let preview=Box::pin(runtime.research_publication_preview(&request,&cancel)).await.unwrap();
    assert_eq!(preview.delta.replacements.len(),2);
    let intent:crate::workflow::ResearchPublicationIntent=serde_json::from_slice(&serde_json::to_vec(&preview.intent).unwrap()).unwrap();
    let receipt=Box::pin(runtime.research_publish(&intent,&cancel)).await.unwrap();
    assert_eq!(receipt.run,run);assert_eq!(receipt.attempt,attempt);assert_eq!(receipt.revision,"publish-fit");
    assert_eq!(Box::pin(runtime.research_publish(&intent,&cancel)).await.unwrap(),receipt,"replay settles original receipt after head changed");
    let mut stale=request.clone();stale.operation="stale-new-operation".into();
    let stale=Box::pin(runtime.research_publication_preview(&stale,&cancel)).await.unwrap();
    assert!(Box::pin(runtime.research_publish(&stale.intent,&cancel)).await.is_err());
    let downstream=Box::pin(runtime.research_publication_open("publish-fit",&cancel)).await.unwrap();
    assert_eq!(crate::workflow::OperationSource::of(&downstream).research.unwrap().publication.as_deref(),Some("publish-fit"));
    let downstream_model=downstream.prepare(root,id(33).into(),Default::default(),Default::default(),&cancel).await.unwrap();
    let target=downstream_model.compiled().model.paths["a"];
    assert_eq!(downstream_model.compiled().admitted.case().parameters().iter().find(|p|p.id==target).unwrap().quantity,quantity);
    // A backdated correction changes new discovery, never this old fit's source receipt.
    Box::pin(runtime.project_record_context("corrected-context",&project,"calibration",&source,0,None,Some("original-context"),false)).await.unwrap();
    let latest=store.project_context_cut("research").await.unwrap();
    assert_eq!(store.select_project_context(&latest,"calibration",10).await.unwrap().unwrap().context.key,"corrected-context");
    assert_eq!(store.research_dataset("raw-v1").await.unwrap().unwrap(),raw_row);
    let reopened=runtime.results(&run,&attempt,"runtime.fitted_parameter_cells",0,u64::MAX,cancel.token()).await.unwrap();
    assert_eq!(reopened.selection().run().revision,context.context.revision);
    let point=crate::workflow::StudyPoint {
        operation:crate::workflow::OperationRequest::DeclaredCase(crate::workflow::CaseOperation{case:root,route:pse_model::generated::enums::ModelingAnalysisRoute::Steady,settings:Default::default()}),
        preparation:crate::workflow::PreparationSettings{compiler:compiler_profile(),..Default::default()},
        overlay:Default::default(),
        policy:crate::workflow::StudyPointPolicy{key:pse_model::study::OccurrenceKey(0),dependencies:vec![],start:pse_model::study::StartPolicy::Fresh,attempt_limit:1},
    };
    let local=Box::pin(downstream.admit_study(&crate::workflow::StudyInputs::Explicit(vec![point]),&crate::workflow::StudyOutputDemand::Full{},&cancel)).await.unwrap();
    let definition=Box::pin(runtime.record_study(&local,&cancel)).await.unwrap();
    assert_eq!(definition.modeling_revision,"publish-fit");
    assert_eq!(definition.points[0].operation.source.research.as_ref().unwrap().selection,selection.key);
    let study=Box::pin(runtime.start_recorded_study(definition,&cancel)).await.unwrap();
    for _ in 0..4 {if study.result().await.unwrap().is_some(){break;} Box::pin(runtime.work_once()).await.unwrap();}
    let results=study.result().await.unwrap().expect("one downstream occurrence completed");
    assert_eq!(results.points.len(),1);
    let _=fit;
}
