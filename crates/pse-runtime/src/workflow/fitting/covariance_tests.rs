// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Covariance, Wald and profile-likelihood units (Plan 22 S3) on a weighted linear
//! regression `yᵢ = a + b·xᵢ`, whose covariance is `(XᵀWX)⁻¹` exactly.
use super::super::regression::*;
use super::*;
use crate::workflow::tests::{compiler_profile, id};
use pse_model::{
    scalar,
    scalars::{Fraction, PositiveCount},
};
use pse_relations::{
    columnar::RelationRow,
    generated::{
        enums::{IntervalEnd, IntervalMethod, IntervalOutcome},
        runtime::{
            parameter_covariances, parameter_intervals, profile_points, response_directions,
        },
    },
};

/// T11 and ADR-0118 item 8: an exact-Hessian fit's covariance is the inverse reduced
/// Hessian of its own KKT analysis, labelled exact; a quasi-Newton fit's is Gauss–Newton
/// from the response SVD. On a linear model both equal `(XᵀWX)⁻¹`.
#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn linear_regression_covariance_analytic() {
    let (_, estimate) = analytic();
    let package = package(declared(), [1.0; 4]).await;
    for (hessian, approximation) in [
        (HessianMode::Exact, CovarianceApproximation::Exact),
        (
            HessianMode::LimitedMemory,
            CovarianceApproximation::GaussNewton,
        ),
        (
            HessianMode::GaussNewton,
            CovarianceApproximation::GaussNewton,
        ),
    ] {
        let prepared = package
            .prepare_fit(
                id(32).into(),
                profile(hessian, None),
                compiler_profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let frozen = prepared.problem.clone();
        let result = prepared.start().unwrap().wait().await.unwrap();
        if hessian == HessianMode::Exact {
            crate::workflow::review_qualification_tests::assert_review_transport(
                &result,
                &[
                    ("/fit/constructor", "replay-fit-constructor.v3"),
                    ("/fit/context", "scientific-context.v1"),
                    ("/fit/runtime", "native-execution-snapshot.v1"),
                ],
            )
            .await;
        }
        let report = report(&result);
        let candidate = report.candidate.as_ref().unwrap();
        for k in 0..2 {
            assert!(
                (candidate[k] - estimate[k]).abs()
                    <= crate::workflow::tests::engineering_target(
                        &frozen.numerics,
                        NumericalTarget::Variable,
                        [id(A), id(B)][k],
                    )
                    .budget,
                "{hessian:?} {candidate:?}"
            );
        }
        let exported = pse_relations::generated::runtime::fitted_parameter_cells::Row::rows(
            &result.export_fit_parameters().unwrap(),
        )
        .unwrap();
        assert_eq!(exported.len(), 2);
        for (k, cell) in exported.iter().enumerate() {
            assert_eq!(cell.parameter_id, [id(A), id(B)][k]);
            assert_eq!(cell.value, candidate[k]);
            assert_eq!(cell.source_revision, package.revision.identity().as_id());
            assert_eq!(cell.run_id, result.run_id);
        }
        let covariance = report.covariance.as_ref().unwrap();
        assert_eq!(covariance.approximation, approximation);
        assert_eq!(covariance.parameters, vec![id(A), id(B)]);
        let values = covariance.values.as_ref().unwrap();
        // Independent information operator of the authored weighted linear
        // regression. Covariance has parameter-squared units; its inverse action
        // uses the actual dimensionless production linear backward-error budget.
        let mut information = [[0.; 2]; 2];
        for (x, sigma) in X.iter().zip(SIGMA) {
            let row = [1., *x];
            for i in 0..2 {
                for j in 0..2 {
                    information[i][j] += row[i] * row[j] / (sigma * sigma);
                }
            }
        }
        for (i, row) in information.iter().enumerate() {
            for j in 0..2 {
                let rhs = f64::from(i == j);
                let terms = [row[0] * values[j], row[1] * values[2 + j]];
                let residual = (terms.iter().sum::<f64>() - rhs).abs();
                let magnitude = rhs.abs() + terms.iter().map(|v| v.abs()).sum::<f64>();
                assert!(
                    residual <= frozen.numerics.policy.linear_backward_error * magnitude,
                    "{hessian:?} inverse information action [{i},{j}]: {values:?}"
                );
            }
        }
        // The published covariance and its certified validity.
        let rows = parameter_covariances::Row::rows(
            &result.table("runtime.parameter_covariances").unwrap(),
        )
        .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].approximation, approximation);
        assert_eq!(&rows[0].values, values);
        let validity = validity(&result);
        let row = validity
            .iter()
            .find(|r| r.quantity == DerivedQuantity::ParameterCovariance)
            .unwrap();
        assert!(row.validity.certified, "{row:?}");
        // The exact covariance states the verdicts of the KKT point it was read from.
        assert_eq!(
            row.validity.second_order,
            (hessian == HessianMode::Exact).then_some(true)
        );
        // Every direction is identifiable, over both parameters.
        let directions =
            response_directions::Row::rows(&result.table("runtime.response_directions").unwrap())
                .unwrap();
        assert_eq!(directions.len(), 4);
        assert!(directions.iter().all(|d| d.identifiable));
    }
}

/// The active-bound admission needs both complete original-coordinate multiplier
/// vectors. Mutate only that evidence in one genuinely executed Gauss–Newton fit;
/// its original candidate, responses and quality remain available.
#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn covariance_withheld_for_missing_or_malformed_bound_multipliers() {
    let package = package(declared(), [1.; 4]).await;
    let prepared = package
        .prepare_fit(
            id(32).into(),
            profile(HessianMode::GaussNewton, None),
            compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let problem = &prepared.problem;
    let mut report = problem
        .execute_profile(
            &problem.profile.solver,
            prepared.route(),
            pse_kernels::ExecutionScope::new(
                pse_diagnostics::Cancellation::default(),
                std::time::Instant::now().checked_add(problem.profile.solver.controls.time_limit),
            ),
            Arc::new(native::solve::Progress::new(16)),
            1,
            None,
        )
        .unwrap();
    problem.estimate_valid(&report).unwrap();
    assert!(report.quality.as_ref().unwrap().feasible(), "{report:?}");
    assert_eq!(report.rank, Some(2));
    assert!(report.responses.is_some());
    let original = report.solve.as_ref().unwrap().clone();
    let candidate = report.candidate.as_ref().unwrap().clone();
    let baseline = problem.covariance(&report).unwrap();
    assert_eq!(baseline.approximation, CovarianceApproximation::GaussNewton);
    assert!(baseline.values.is_ok(), "{baseline:?}");
    // The same real solve supplies every negative control; no altered evidence
    // is fed back to the solver and no new candidate or qualification is forged.
    for malformed in 0..3 {
        report.solve = Some(original.clone());
        let multipliers = &mut report
            .solve
            .as_mut()
            .unwrap()
            .candidate
            .as_mut()
            .unwrap()
            .bound_dual;
        match malformed {
            0 => *multipliers = None,
            1 => {
                assert!(multipliers.as_mut().unwrap().0.pop().is_some());
            }
            2 => {
                assert!(multipliers.as_mut().unwrap().1.pop().is_some());
            }
            _ => panic!("invalid malformed bound-dual fixture selector"),
        }
        problem.estimate_valid(&report).unwrap();
        assert_eq!(report.candidate.as_ref().unwrap(), &candidate);
        assert_eq!(report.rank, Some(2));
        assert!(report.responses.is_some());
        assert!(report.quality.as_ref().unwrap().feasible());
        let withheld = problem.covariance(&report).unwrap();
        assert!(
            matches!(
                &withheld.values,
                Err(FitWithheld::Local(Withheld::Multipliers))
            ),
            "{withheld:?}"
        );
    }
    report.solve = Some(original);
    assert_eq!(
        problem.covariance(&report).unwrap().values.unwrap(),
        baseline.values.unwrap()
    );
}

/// F02 (ADR-0118 item 2): the declared model needs a standard deviation for every included
/// observation. Admission refuses one without, so no fit exists whose covariance would
/// lack it.
#[tokio::test]
async fn covariance_withheld_without_declared_sigma() {
    let mut sigma = declared();
    sigma[1] = None;
    let error = package(sigma, [1.0; 4])
        .await
        .prepare_fit(
            id(32).into(),
            profile(HessianMode::Exact, None),
            compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap_err();
    assert!(
        matches!(error, WorkflowError::Input(ref message) if message == "included observations require finite values, positive difference-unit standard deviations and importance"),
        "{error:?}"
    );
}

#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn covariance_withheld_with_nonunit_importance() {
    let result = fit(
        &package(declared(), [1.0, 1.0, 2.0, 1.0]).await,
        profile(
            HessianMode::Exact,
            Some(FitUncertainty {
                level: scalar!(Fraction(0.95)),
                profile: None,
                predictions: false,
            }),
        ),
    )
    .await;
    let report = report(&result);
    let covariance = report.covariance.as_ref().unwrap();
    assert!(
        matches!(&covariance.values, Err(FitWithheld::NonunitImportance(o)) if *o == vec![id(42)]),
        "{covariance:?}"
    );
    // The Wald intervals it would have given are withheld upstream.
    assert!(matches!(
        report.wald,
        Some(Err(FitWithheld::Upstream(
            DerivedQuantity::ParameterCovariance
        )))
    ));
    let reasons: Vec<_> = validity(&result)
        .iter()
        .map(|r| (r.quantity, r.validity.reason))
        .collect();
    assert_eq!(
        reasons,
        vec![
            (
                DerivedQuantity::ParameterCovariance,
                Some(WithheldReason::NonunitImportance)
            ),
            (
                DerivedQuantity::WaldInterval,
                Some(WithheldReason::UpstreamWithheld)
            ),
        ]
    );
    assert_eq!(
        result
            .table("runtime.parameter_intervals")
            .unwrap()
            .batch()
            .num_rows(),
        0
    );
}

/// A fit whose dense response diagnostic exceeds its cell allowance keeps its candidate
/// and withholds the covariance for want of responses: 4 observations over 2 parameters
/// need 8 dense cells. The allowance also bounds preparation's sparse layout, so it drops
/// to 7 only on the prepared fit.
#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn covariance_withheld_without_responses() {
    let mut prepared = package(declared(), [1.0; 4])
        .await
        .prepare_fit(
            id(32).into(),
            profile(HessianMode::LimitedMemory, None),
            compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    Arc::get_mut(&mut prepared.problem)
        .unwrap()
        .profile
        .max_cells = 7;
    let result = prepared.start().unwrap().wait().await.unwrap();
    crate::workflow::review_qualification_tests::assert_review_transport(
        &result,
        &[("/fit/context", "scientific-context.v1")],
    )
    .await;
    let report = report(&result);
    assert!(report.candidate.is_some(), "{report:?}");
    let covariance = report.covariance.as_ref().unwrap();
    assert!(
        matches!(&covariance.values, Err(FitWithheld::Responses(Some(_)))),
        "{covariance:?}"
    );
    let reasons: Vec<_> = validity(&result)
        .iter()
        .map(|r| (r.quantity, r.validity.reason))
        .collect();
    assert_eq!(
        reasons,
        vec![(
            DerivedQuantity::ParameterCovariance,
            Some(WithheldReason::ResponsesUnavailable)
        )]
    );
}

/// On a linear model independent minimized weighted likelihoods establish the
/// profile/Wald endpoint agreement at production accuracy. The interval root uses
/// its default statistic control; serial and four-worker chains must agree.
#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn profile_likelihood_matches_wald_on_linear_model() {
    let (expected, estimate) = analytic();
    let profile = profile(
        HessianMode::Exact,
        Some(FitUncertainty {
            level: scalar!(Fraction(0.9)),
            profile: Some(ProfileControls {
                points: scalar!(PositiveCount(20)),
                workers: Some(scalar!(PositiveCount(2))),
                ..Default::default()
            }),
            predictions: false,
        }),
    );
    // Keep the ordinary fixture's memory capacity and explicitly admit the four
    // one-thread chains this control compares with serial execution.
    let runtime = crate::workflow::tests::runtime_on_threads(
        4 << 30,
        crate::math::MathPolicy {
            worker_bytes: 1 << 30,
            workspace_bytes: 1 << 30,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
        4,
    );
    assert_eq!(runtime.shared.math().cores(), 4);
    let package = package_on(runtime, false, declared(), [1.0; 4]).await;
    // The chains do not depend on how many run at once.
    let prepared = package
        .prepare_fit(
            id(32).into(),
            profile.clone(),
            compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(prepared.problem.profile.solver.controls.threads, 1);
    let chains = |workers| {
        let prepared = prepared.clone();
        async move {
            let service = prepared.problem.runtime.native().clone();
            let threads = prepared.problem.profile.solver.controls.threads;
            let bytes = prepared.problem.bytes;
            service
                .job(
                    threads,
                    bytes,
                    pse_diagnostics::Cancellation::default(),
                    move |flag| {
                        prepared.problem.execute_profile(
                            &prepared.problem.profile.solver,
                            prepared.route(),
                            pse_kernels::ExecutionScope::new(
                                flag,
                                std::time::Instant::now().checked_add(
                                    prepared.problem.profile.solver.controls.time_limit,
                                ),
                            ),
                            Arc::new(native::solve::Progress::new(16)),
                            workers,
                            None,
                        )
                    },
                )
                .await
                .unwrap()
                .profiles
                .unwrap()
                .unwrap()
        }
    };
    let serial = chains(1).await;
    let parallel = chains(4).await;
    assert!(serial.iter().all(|chain| chain.actual_parallelism == 1));
    assert!(parallel.iter().all(|chain| chain.actual_parallelism == 4));
    let frozen = prepared.problem.clone();
    assert_eq!(serial.len(), parallel.len());
    for (serial, parallel) in serial.iter().zip(&parallel) {
        assert_eq!(
            (serial.parameter, serial.end),
            (parallel.parameter, parallel.end)
        );
        assert_eq!(serial.bound.outcome, parallel.bound.outcome);
        assert!(serial.worker_failure.is_none() && parallel.worker_failure.is_none());
        assert!(serial.scheduling_failures.is_empty() && parallel.scheduling_failures.is_empty());
        let allowance = crate::workflow::tests::engineering_target(
            &frozen.numerics,
            NumericalTarget::Variable,
            serial.parameter,
        )
        .budget;
        assert!((serial.estimate - parallel.estimate).abs() <= allowance);
        assert!((serial.bound.value.unwrap() - parallel.bound.value.unwrap()).abs() <= allowance);
    }
    let uncertainty = frozen.profile.uncertainty.as_ref().unwrap();
    let controls = uncertainty.profile.as_ref().unwrap();
    let threshold = statrs::distribution::ChiSquared::new(1.)
        .unwrap()
        .inverse_cdf(uncertainty.level.into_inner())
        .sqrt();
    let statistic_allowance = controls.tolerance.into_inner() * threshold;
    let objective_allowance = crate::workflow::tests::engineering_target(
        &frozen.numerics,
        NumericalTarget::Objective,
        SemanticId::NIL,
    )
    .budget;
    // Independently minimize the authored weighted linear likelihood. This
    // objective allowance is checked empirically for the base and every accepted
    // pinned fit; the root tolerance is a separate dimensionless statistic control.
    let optimum = (0..X.len())
        .map(|i| 0.5 * ((estimate[0] + estimate[1] * X[i] - Y[i]) / SIGMA[i]).powi(2))
        .sum::<f64>();
    for chain in serial.iter().chain(&parallel) {
        let k = [id(A), id(B)]
            .iter()
            .position(|parameter| *parameter == chain.parameter)
            .unwrap();
        for point in chain.points.iter().filter(|point| point.accepted) {
            let analytic_objective =
                optimum + (point.value - estimate[k]).powi(2) / (2. * expected[3 * k]);
            assert!(
                (point.objective.unwrap() - analytic_objective).abs() <= objective_allowance,
                "independent serial/parallel pinned likelihood: {point:?}"
            );
        }
    }
    let result = prepared.start().unwrap().wait().await.unwrap();
    let report = report(&result);
    assert!(result.usable(), "{report:?}");
    assert!(
        (report.objective.unwrap() - optimum).abs() <= objective_allowance,
        "{report:?}"
    );
    let wald = report.wald.as_ref().unwrap().as_ref().unwrap();
    let chains = report.profiles.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(chains.len(), 4);
    for (k, interval) in wald.iter().enumerate() {
        let error = expected[3 * k].sqrt();
        let parameter_allowance = crate::workflow::tests::engineering_target(
            &frozen.numerics,
            NumericalTarget::Variable,
            [id(A), id(B)][k],
        )
        .budget;
        let half = interval.upper.value.unwrap() - interval.estimate;
        let center_error = (interval.estimate - estimate[k]).abs();
        let width_error = (half - threshold * error).abs();
        assert!(center_error <= parameter_allowance, "{interval:?}");
        assert!(width_error <= parameter_allowance, "{interval:?}");
        for chain in chains.iter().filter(|c| c.parameter == interval.parameter) {
            assert_eq!(chain.estimate, interval.estimate);
            assert_eq!(chain.bound.outcome, IntervalOutcome::Threshold, "{chain:?}");
            assert!(
                chain.points.len() <= controls.points.into_inner(),
                "{chain:?}"
            );
            for point in chain.points.iter().filter(|point| point.accepted) {
                let analytic_objective =
                    optimum + (point.value - estimate[k]).powi(2) / (2. * expected[3 * k]);
                assert!(
                    (point.objective.unwrap() - analytic_objective).abs() <= objective_allowance,
                    "independent pinned likelihood: {point:?}"
                );
                assert_eq!(
                    point.statistic.unwrap(),
                    (2. * (point.objective.unwrap() - report.objective.unwrap()).max(0.)).sqrt()
                );
                assert!(
                    matches!(
                        point.qualification,
                        Some(
                            Qualification::Stationary
                                | Qualification::OptimalWithinTolerance
                                | Qualification::GapQualified
                        )
                    ),
                    "{point:?}"
                );
            }
            let wald = if chain.end == IntervalEnd::Lower {
                &interval.lower
            } else {
                &interval.upper
            };
            let value = chain.bound.value.unwrap();
            let width_error =
                ((wald.value.unwrap() - interval.estimate).abs() - threshold * error).abs();
            assert!(width_error <= parameter_allowance, "{interval:?}");
            let endpoint = chain
                .points
                .iter()
                .find(|point| point.accepted && point.value == value)
                .unwrap();
            assert!(
                (endpoint.statistic.unwrap() - threshold).abs() <= statistic_allowance,
                "{chain:?}"
            );
            // Two checked likelihood errors compose into 4*allowance in the
            // squared statistic. Transform that discrepancy into parameter units,
            // then separately include the checked Wald center and width errors.
            let lower_radius = ((threshold - statistic_allowance).powi(2)
                - 4. * objective_allowance)
                .max(0.)
                .sqrt();
            let upper_radius =
                ((threshold + statistic_allowance).powi(2) + 4. * objective_allowance).sqrt();
            let endpoint_allowance =
                error * (threshold - lower_radius).max(upper_radius - threshold);
            let direction = if chain.end == IntervalEnd::Lower {
                -1.
            } else {
                1.
            };
            assert!(direction * (value - estimate[k]) > 0., "{chain:?}");
            assert!(
                (value - (estimate[k] + direction * threshold * error)).abs() <= endpoint_allowance,
                "{chain:?}"
            );
            assert!(
                (value - wald.value.unwrap()).abs()
                    <= endpoint_allowance + center_error + width_error,
                "{value} {wald:?} {chain:?}"
            );
        }
    }
    let rows =
        parameter_intervals::Row::rows(&result.table("runtime.parameter_intervals").unwrap())
            .unwrap();
    assert_eq!(rows.len(), 8);
    assert!(rows.iter().all(|r| r.level == 0.9 && r.value.is_some()));
    assert_eq!(
        rows.iter()
            .filter(|r| r.method == IntervalMethod::ProfileLikelihood)
            .count(),
        4
    );
    let validity = validity(&result);
    assert!(
        validity.iter().all(|r| r.validity.certified),
        "{validity:?}"
    );
    assert_eq!(validity.len(), 3);
}

/// PS-11: each pinned fit is seeded from its chain's latest accepted point below the
/// threshold, or from the estimate, and the seed is recorded with the point. The offset
/// `d` leaves `a + d` identifiable but not `a` or `d`: the covariance is withheld with the
/// identifiable directions reported, while the profile, which needs no full rank, is flat
/// along `a` and `d` up to their bounds and sized from the declared scale.
#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn profile_chain_seeds_from_predecessor() {
    let uncertainty = || {
        Some(FitUncertainty {
            level: scalar!(Fraction(0.95)),
            profile: Some(ProfileControls::default()),
            predictions: false,
        })
    };
    let result = fit(
        &package_with(true, declared(), [2.0, 1.0, 1.0, 1.0]).await,
        profile(HessianMode::LimitedMemory, uncertainty()),
    )
    .await;
    // The profile needs the declared model too.
    assert!(matches!(
        report(&result).profiles,
        Some(Err(FitWithheld::NonunitImportance(_)))
    ));
    let package = package_with(true, declared(), [1.0; 4]).await;
    let prepared = package
        .prepare_fit(
            id(32).into(),
            profile(HessianMode::LimitedMemory, uncertainty()),
            compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let frozen = prepared.problem.clone();
    let uncertainty = frozen.profile.uncertainty.as_ref().unwrap();
    let controls = uncertainty.profile.as_ref().unwrap();
    let threshold = statrs::distribution::ChiSquared::new(1.)
        .unwrap()
        .inverse_cdf(uncertainty.level.into_inner())
        .sqrt();
    let result = prepared.start().unwrap().wait().await.unwrap();
    let report = report(&result);
    assert!(
        matches!(
            report.covariance.as_ref().unwrap().values,
            Err(FitWithheld::RankDeficient {
                rank: 2,
                parameters: 3
            })
        ),
        "{report:?}"
    );
    // The unidentifiable direction is a − d: orthogonal to the identifiable a + d.
    let directions = report.directions.as_ref().unwrap();
    let unidentifiable = directions.col(2);
    assert!(
        (unidentifiable[0] + unidentifiable[2]).abs() < 1e-6,
        "{directions:?}"
    );
    assert!(unidentifiable[1].abs() < 1e-6, "{directions:?}");
    let chains = report.profiles.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(chains.len(), 6);
    assert!(
        !report.estimate_qualified(),
        "joint identifiability is not required for profile likelihood"
    );
    assert!(result.usable());
    let evidence = result.invocation_evidence().unwrap();
    assert!(
        evidence
            .products
            .iter()
            .any(|product| product.role == "profile_intervals"
                && product.availability == crate::invocation::ProductAvailability::Available)
    );
    assert!(
        evidence
            .products
            .iter()
            .any(|product| product.role == "parameter_covariance"
                && product.availability == crate::invocation::ProductAvailability::Unavailable)
    );
    let r = |p: &ProfilePoint| p.statistic.unwrap();
    for chain in chains {
        let flat = chain.parameter != id(B);
        let expected = if flat {
            IntervalOutcome::Bound
        } else {
            IntervalOutcome::Threshold
        };
        assert_eq!(chain.bound.outcome, expected, "{chain:?}");
        if flat {
            assert_eq!(chain.bound.value.map(f64::abs), Some(100.0), "{chain:?}");
        }
        assert!(chain.points.len() > 2, "{chain:?}");
        assert!(
            chain.points.len() <= controls.points.into_inner(),
            "{chain:?}"
        );
        assert_eq!(chain.points[0].seed, None, "{chain:?}");
        // The seed of every later point is the latest accepted point below the threshold.
        for (i, point) in chain.points.iter().enumerate().skip(1) {
            let predecessor = (0..i)
                .rev()
                .find(|&j| chain.points[j].accepted && r(&chain.points[j]) < threshold);
            assert_eq!(point.seed, predecessor, "{i} {chain:?}");
        }
        assert!(chain.points.iter().all(|p| p.accepted), "{chain:?}");
    }
    // Published with its seeds.
    let points =
        profile_points::Row::rows(&result.table("runtime.profile_points").unwrap()).unwrap();
    assert_eq!(
        points.len(),
        chains.iter().map(|c| c.points.len()).sum::<usize>()
    );
    for chain in chains {
        for (i, point) in chain.points.iter().enumerate() {
            let row = points
                .iter()
                .find(|r| {
                    r.parameter_id == chain.parameter && r.end == chain.end && r.point == i as i64
                })
                .unwrap();
            assert_eq!(row.seed, point.seed.map(|s| s as i64));
            assert_eq!(row.value, point.value);
        }
    }
    let reasons: Vec<_> = validity(&result)
        .iter()
        .map(|r| (r.quantity, r.validity.reason))
        .collect();
    assert_eq!(
        reasons,
        vec![
            (
                DerivedQuantity::ParameterCovariance,
                Some(WithheldReason::RankDeficient)
            ),
            (
                DerivedQuantity::WaldInterval,
                Some(WithheldReason::UpstreamWithheld)
            ),
            (DerivedQuantity::ProfileInterval, None),
        ]
    );
}

/// Admission: a level and profile controls in range, and no parametric sensitivity request.
#[tokio::test]
async fn fit_uncertainty_admission() {
    let package = package(declared(), [1.0; 4]).await;
    let prepare = |profile| {
        let package = package.clone();
        async move {
            package
                .prepare_fit_problem(
                    id(32).into(),
                    profile,
                    compiler_profile(),
                    Default::default(),
                    &crate::CancelSource::new(),
                )
                .await
        }
    };
    // Single-value domains refuse at decoding; the level must also lie below one.
    assert!(
        serde_json::from_str::<FitUncertainty>(r#"{"level": 0.95, "profile": {"points": 0}}"#)
            .is_err()
    );
    assert!(serde_json::from_str::<FitUncertainty>(r#"{"level": 0}"#).is_err());
    let decoded: FitUncertainty =
        serde_json::from_str(r#"{"level": 0.95, "profile": {}}"#).unwrap();
    assert_eq!(decoded.profile, Some(ProfileControls::default()));
    assert!(
        prepare(profile(
            HessianMode::Exact,
            Some(FitUncertainty::wald(scalar!(Fraction(1.0))))
        ))
        .await
        .is_err()
    );
    let mut sensitivity = profile(HessianMode::Exact, None);
    sensitivity.solver.sensitivity = Some(crate::math::settings::SensitivityRequest {
        parameters: vec![id(A)],
        reduced_hessian: false,
        propagation: None,
    });
    assert!(prepare(sensitivity).await.is_err());
    // The requested intervals enter the profile identity; their absence keeps it.
    let (plain, _) = prepare(profile(HessianMode::Exact, None)).await.unwrap();
    let (wald, _) = prepare(profile(
        HessianMode::Exact,
        Some(FitUncertainty::wald(scalar!(Fraction(0.95)))),
    ))
    .await
    .unwrap();
    assert_ne!(plain.profile_key, wald.profile_key);
    assert_eq!(plain.source_identity, wald.source_identity);
}

/// One original four-observation regression checks durable retention and descriptive
/// transport without expanding the scientific covariance/accuracy campaign.
#[cfg(all(feature = "canonical-tests", feature = "native-solvers"))]
#[tokio::test]
async fn review_qualification_durable_fit_preserves_original_response_and_rows() {
    use pse_relations::generated::runtime::fit_observations;
    let runtime = crate::workflow::durable_tests::durable_runtime();
    let package = package_on(runtime.clone(), false, declared(), [1.; 4]).await;
    let controls = profile(HessianMode::LimitedMemory, None);
    assert_eq!(
        controls.solver.numerics.engineering_relative_fraction,
        0.001
    );
    let prepared = package
        .prepare_fit(
            id(32).into(),
            controls,
            compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let handle = prepared.start().unwrap();
    let run = handle.canonical_run_key().unwrap().to_owned();
    let attempt = handle.canonical_attempt_key().unwrap().to_owned();
    let result = handle.wait().await.unwrap();
    assert!(result.usable(), "{:?}", result.report());
    let (_, expected) = analytic();
    let fitted = report(&result);
    for (actual, expected) in fitted.candidate.as_ref().unwrap().iter().zip(expected) {
        assert!((actual - expected).abs() <= 0.001 * expected.abs().max(1.));
    }
    assert_eq!(fitted.predictions.len(), 4);
    assert!(fitted.checks_complete);
    assert!(fitted.responses.is_some());
    let invocation = result.invocation_evidence().unwrap();
    assert_eq!(invocation.run_id, result.run_id);
    assert!(
        invocation
            .products
            .iter()
            .any(|product| product.role == "response_jacobian"
                && product.availability == crate::invocation::ProductAvailability::Available)
    );
    let original =
        fit_observations::Row::rows(&result.table("runtime.fit_observations").unwrap()).unwrap();
    let mut reopened = runtime
        .results(
            &run,
            &attempt,
            "runtime.fit_observations",
            0,
            u64::MAX,
            pse_columnar::CancellationToken::new(),
        )
        .await
        .unwrap();
    let mut rows = Vec::new();
    while let Some(batch) = reopened.next_batch().await.unwrap() {
        let view = fit_observations::View::try_from_batch_with_registry(
            runtime.registry(),
            &batch,
            &runtime.validation_context(),
        )
        .unwrap();
        rows.extend(view.rows().unwrap());
    }
    assert_eq!(rows, original);
    assert_eq!(rows.len(), 4);
    drop(reopened);
    crate::workflow::review_qualification_tests::assert_review_transport(
        &result,
        &[
            ("/fit/constructor", "replay-fit-constructor.v3"),
            ("/fit/context", "scientific-context.v1"),
        ],
    )
    .await;
}

// Pure policy controls intentionally construct neither Runtime nor prepared fit/native session.
#[test]
fn covariance_policy_estimate_qualification_and_statistical_refusal_order() {
    let diagnostic = None;
    let evidence =
        |candidate, residual_qualification, native_qualification, objective| EstimateEvidence {
            candidate,
            residual_qualification,
            native_qualification,
            objective,
            diagnostic: &diagnostic,
        };
    for qualification in [
        Qualification::Stationary,
        Qualification::OptimalWithinTolerance,
        Qualification::GapQualified,
    ] {
        assert!(
            estimate_policy(
                evidence(true, None, Some(qualification), true),
                std::iter::empty()
            )
            .is_ok()
        );
    }
    assert!(
        estimate_policy(
            evidence(true, Some(Qualification::Stationary), None, true),
            std::iter::empty()
        )
        .is_ok()
    );
    assert!(matches!(
        estimate_policy(
            evidence(false, Some(Qualification::Unqualified), None, false),
            [id(1)].into_iter()
        ),
        Err(FitWithheld::Local(Withheld::NoCandidate))
    ));
    // LM requires its independently established physical stationarity; native success
    // cannot substitute for that qualification, even when other premises also fail.
    for qualification in [
        Qualification::Unqualified,
        Qualification::OptimalWithinTolerance,
        Qualification::GapQualified,
    ] {
        assert!(matches!(estimate_policy(evidence(true, Some(qualification),
            Some(Qualification::Stationary), false), [id(1)].into_iter()),
            Err(FitWithheld::Local(Withheld::Unqualified(q))) if q == qualification));
    }
    assert!(matches!(
        estimate_policy(evidence(true, None, None, true), std::iter::empty()),
        Err(FitWithheld::Local(Withheld::NoCandidate))
    ));
    assert!(matches!(
        estimate_policy(
            evidence(true, None, Some(Qualification::Unqualified), true),
            std::iter::empty()
        ),
        Err(FitWithheld::Local(Withheld::Unqualified(
            Qualification::Unqualified
        )))
    ));
    assert!(
        matches!(estimate_policy(evidence(true, Some(Qualification::Stationary), None, false),
        [id(2), id(1)].into_iter()), Err(FitWithheld::NonunitImportance(ids)) if ids == [id(2), id(1)])
    );
    assert!(matches!(
        estimate_policy(
            evidence(true, Some(Qualification::Stationary), None, false),
            std::iter::empty()
        ),
        Err(FitWithheld::Responses(None))
    ));
}
#[test]
fn covariance_policy_requires_response_evidence_before_rank() {
    for (rank, directions) in [(None, true), (Some(0), false)] {
        assert!(matches!(
            response_policy(rank, directions, 2, &None),
            Err(FitWithheld::Responses(None))
        ));
    }
    assert!(matches!(
        response_policy(Some(1), true, 2, &None),
        Err(FitWithheld::RankDeficient {
            rank: 1,
            parameters: 2
        })
    ));
    assert!(response_policy(Some(2), true, 2, &None).is_ok());
}
#[test]
fn covariance_policy_uses_actual_bound_and_multiplier_evidence() {
    for (finite_variable_bounds, general_bounds) in [(true, false), (false, true)] {
        assert!(matches!(
            bound_policy(
                BoundEvidence::Residual {
                    finite_variable_bounds,
                    general_bounds
                },
                std::iter::empty()
            ),
            Err(FitWithheld::Local(Withheld::Multipliers))
        ));
    }
    assert!(
        bound_policy(
            BoundEvidence::Residual {
                finite_variable_bounds: false,
                general_bounds: false
            },
            std::iter::empty()
        )
        .is_ok()
    );
    let candidate = [0., 5.];
    let lower = [1., 0.];
    let upper = [0., 0.];
    for duals in [None, Some((&lower[..1], &upper[..]))] {
        assert!(matches!(
            bound_policy(
                BoundEvidence::Native {
                    candidate: &candidate,
                    duals,
                    objective_scale: 1.,
                    stationarity: 1e-8
                },
                std::iter::empty()
            ),
            Err(FitWithheld::Local(Withheld::Multipliers))
        ));
    }
    let parameter = |column| ParameterBoundEvidence {
        id: id(column as u8 + 1),
        column,
        bounds: (0., 10.),
        tolerance: 1e-8,
        scale: 1.,
    };
    assert!(
        matches!(bound_policy(BoundEvidence::Native { candidate: &candidate,
        duals: Some((&lower, &upper)), objective_scale: 1., stationarity: 1e-8 },
        [parameter(1), parameter(0)].into_iter()),
        Err(FitWithheld::AtBound(ids)) if ids == [id(1)])
    );
    assert!(
        bound_policy(
            BoundEvidence::Native {
                candidate: &candidate,
                duals: Some((&lower, &upper)),
                objective_scale: 1.,
                stationarity: 1e-8
            },
            [parameter(1)].into_iter()
        )
        .is_ok()
    );
}
#[test]
fn covariance_policy_preserves_exact_evidence_and_backsolve_refusals() {
    let mut block = native::kkt::InverseReducedHessian {
        columns: vec![OriginalCol::from(0)],
        values: vec![2.],
        normalized: vec![2.],
        coordinate_scales: vec![1.],
        objective_scale: 1.,
    };
    assert_eq!(
        exact_covariance_policy(Some(&Ok(block.clone())), 1).unwrap(),
        [2.]
    );
    assert!(matches!(
        exact_covariance_policy(None, 1),
        Err(FitWithheld::Local(Withheld::Analysis(_)))
    ));
    assert!(matches!(
        exact_covariance_policy(Some(&Ok(block.clone())), 2),
        Err(FitWithheld::Local(Withheld::Analysis(_)))
    ));
    assert!(matches!(
        exact_covariance_policy(Some(&Err(Withheld::Multipliers)), 1),
        Err(FitWithheld::Local(Withheld::Multipliers))
    ));
    // Nonfinite values take precedence over the wrong selected-column extent.
    block.values[0] = f64::NAN;
    assert!(matches!(
        exact_covariance_policy(Some(&Ok(block)), 2),
        Err(FitWithheld::Local(Withheld::Backsolve))
    ));
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            finite_covariance_policy(&[1., value]),
            Err(FitWithheld::Local(Withheld::Backsolve))
        ));
    }
    assert!(finite_covariance_policy(&[1., 0., 0., 2.]).is_ok());
}

/// An explicit state variable and nonlinear equality retain constraint curvature
/// in the fit KKT system. No polynomial substitution removes that boundary.
#[cfg(all(feature = "canonical-tests", feature = "native-solvers"))]
async fn nonlinear_state_fit_package(
    runtime: crate::workflow::Runtime,
    observed_x: f64,
) -> crate::workflow::ModelingPackage {
    let mut physical = crate::workflow::tests::physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    let mut source = pse_authoring::language::parse(
        "package nonlinear_fit { def Root {
            param x: Scalar = 0.8; var y: Scalar;
            eq state: y - x*x == 0;
            annotation start y(0.7);
            annotation engineering_scale y(kind=magnitude,value=7);
        } }",
        id(20),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = source
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    source.extend(measured_rows(&[
        (id(40), "Scalar", Some(observed_x), Some(1.)),
        (id(41), "Scalar", Some(0.), Some(1.)),
    ]));
    let mut declarations = FitDeclarations::default();
    declarations.fits.push(serde_json::from_value(serde_json::json!({
        "fit_id":id(32),
        "parameters":[{"symbol_id":id(A),"fixed":false,"value":0.8,"lower":null,"upper":null,"scale":2.0}],
        "experiments":[{"experiment_id":id(33),"case_id":root,"route":"steady","bindings":[{"parameter_id":id(A),"path":"x"}]}],
        "observations":[
            {"value_attribute":"value","standard_deviation_attribute":"sigma","observation_id":id(41),"experiment_id":id(33),"output_path":"y","time":null,"included":true,"importance":1.0},
            {"value_attribute":"value","standard_deviation_attribute":"sigma","observation_id":id(40),"experiment_id":id(33),"output_path":"x","time":null,"included":true,"importance":1.0}
        ]
    })).unwrap());
    runtime
        .modeling_package(source, physical)
        .await
        .unwrap()
        .with_fit_declarations(declarations)
        .await
        .unwrap()
}

/// Half absolute-sigma RSS gives exact information 1+6s² after the
/// lambda=-s² state constraint curvature. Response-only information is 1+4s².
/// The two nonzero-residual optima distinguish both formulas and a stale point.
#[cfg(all(feature = "canonical-tests", feature = "native-solvers"))]
#[tokio::test]
async fn nonlinear_state_constraint_exact_and_gauss_newton_covariance_reopen() {
    use crate::workflow::modeling::sensitivity_tests::reopened_rows;
    use pse_relations::generated::runtime::{
        fit_constraints, fit_observations, fit_parameters, fit_variables, local_validity,
    };
    let runtime = crate::workflow::durable_tests::durable_runtime();
    for (observed_x, s) in [(3., 1.), (18., 2.)] {
        let package = nonlinear_state_fit_package(runtime.clone(), observed_x).await;
        for (hessian, approximation, information) in [
            (
                HessianMode::Exact,
                CovarianceApproximation::Exact,
                1. + 6. * s * s,
            ),
            (
                HessianMode::GaussNewton,
                CovarianceApproximation::GaussNewton,
                1. + 4. * s * s,
            ),
        ] {
            let mut controls = profile(hessian, None);
            controls.solver.numerics.kkt.stationarity = 1e-9;
            controls.solver.numerics.kkt.complementarity = 1e-9;
            controls.solver.numerics.linear_backward_error = 1e-9;
            controls.solver.numerics.supplier_action_accuracy = 1e-9;
            controls.solver.numerics.engineering_relative_fraction = 1e-7;
            let prepared = package
                .prepare_fit(
                    id(32).into(),
                    controls,
                    compiler_profile(),
                    Default::default(),
                    &crate::CancelSource::new(),
                )
                .await
                .unwrap();
            // The native contract owns decisions; `bounds` owns constraint rows.
            // Keep the explicit state and nonlinear equality in the prepared NLP.
            let problem = &prepared.problem;
            assert_eq!(problem.contract.variables.len(), 2);
            let parameter = problem
                .contract
                .variables
                .iter()
                .find(|v| v.id == id(A))
                .unwrap();
            assert_eq!(
                (parameter.lower, parameter.upper),
                (f64::NEG_INFINITY, f64::INFINITY)
            );
            assert_eq!(problem.contract.rows.len(), 1);
            assert_eq!(problem.bounds, vec![(0., 0.)]);
            assert_eq!(problem.experiments.len(), 1);
            let Experiment::Steady(steady) = &problem.experiments[0] else {
                panic!("nonlinear covariance witness must retain its steady state");
            };
            assert_eq!(steady.local_states, 1);
            assert_eq!(steady.constraints.len(), 1);
            assert_eq!(steady.coordinates.len(), 2);
            let state_id = steady.coordinates.iter().next().unwrap().0;
            assert!(
                steady
                    .variables
                    .iter()
                    .any(|v| v.port.id == state_id && !v.fixed)
            );
            let result = prepared.start().unwrap().wait().await.unwrap();
            assert!(result.usable(), "{:?}", result.report());
            let fitted = report(&result);
            assert!(fitted.estimate_qualified(), "{fitted:?}");
            assert_eq!(fitted.rank, Some(1));
            assert_eq!(fitted.constraint_values.len(), 1);
            assert!(fitted.constraint_values[0].abs() < 1e-6, "{fitted:?}");
            // Publication separates the shared fitted parameter from the local state.
            let parameters =
                fit_parameters::Row::rows(&result.table("runtime.fit_parameters").unwrap())
                    .unwrap();
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].parameter_id, id(A));
            assert!((parameters[0].value.unwrap() - s).abs() < 1e-5);
            let states =
                fit_variables::Row::rows(&result.table("runtime.fit_variables").unwrap()).unwrap();
            assert_eq!(states.len(), 1);
            assert_eq!(states[0].symbol_id, state_id);
            assert!(!states[0].fixed);
            assert!((states[0].value.unwrap() - s * s).abs() < 1e-5);
            let constraints =
                fit_constraints::Row::rows(&result.table("runtime.fit_constraints").unwrap())
                    .unwrap();
            assert_eq!(constraints.len(), 1);
            assert_eq!(
                (constraints[0].lower, constraints[0].upper),
                (Some(0.), Some(0.))
            );
            assert!(constraints[0].value.unwrap().abs() <= constraints[0].tolerance);
            let predictions =
                fit_observations::Row::rows(&result.table("runtime.fit_observations").unwrap())
                    .unwrap();
            for (observation, expected) in [(id(40), s), (id(41), s * s)] {
                let row = predictions
                    .iter()
                    .find(|row| row.observation_id == observation)
                    .unwrap();
                assert!((row.prediction.unwrap() - expected).abs() < 1e-5, "{row:?}");
            }
            let expected_objective = 0.5 * (s - observed_x).powi(2) + 0.5 * s.powi(4);
            assert!((fitted.objective.unwrap() - expected_objective).abs() < 1e-5);
            // Scientific covariance assertion, independently of serialization/readback.
            let covariance = fitted.covariance.as_ref().unwrap();
            assert_eq!(covariance.approximation, approximation);
            assert_eq!(covariance.parameters, vec![id(A)]);
            assert_eq!(covariance.scales, vec![2.]);
            let variance = covariance.values.as_ref().unwrap();
            assert_eq!(variance.len(), 1);
            assert!(
                (variance[0] - 1. / information).abs() < 1e-5,
                "{observed_x} {hessian:?}: {covariance:?}"
            );
            let published = parameter_covariances::Row::rows(
                &result.table("runtime.parameter_covariances").unwrap(),
            )
            .unwrap();
            assert_eq!(published.len(), 1);
            assert_eq!(published[0].approximation, approximation);
            assert_eq!(published[0].parameters, vec![id(A)]);
            assert_eq!(published[0].values, *variance);
            let validity_rows = validity(&result);
            let validity = validity_rows
                .iter()
                .find(|row| row.quantity == DerivedQuantity::ParameterCovariance)
                .unwrap();
            assert!(
                validity.validity.certified && validity.validity.reason.is_none(),
                "{validity:?}"
            );
            assert_eq!(
                validity.validity.second_order,
                (hessian == HessianMode::Exact).then_some(true)
            );
            if hessian == HessianMode::Exact {
                assert_eq!(validity.validity.licq, Some(true));
                assert_eq!(validity.validity.strict_complementarity, Some(true));
            }
            // Result queries may order rows differently from the in-process writer.
            // Match each relation's declared primary key, retaining exact whole rows.
            let mut published = published;
            let mut reopened_covariance = reopened_rows::<parameter_covariances::Row>(
                &runtime,
                &result,
                "runtime.parameter_covariances",
            )
            .await;
            published.sort_by_key(|row| row.run_id);
            reopened_covariance.sort_by_key(|row| row.run_id);
            assert_eq!(reopened_covariance, published);
            let mut validity_rows = validity_rows;
            let mut reopened_validity =
                reopened_rows::<local_validity::Row>(&runtime, &result, "runtime.local_validity")
                    .await;
            validity_rows.sort_by_key(|row| (row.run_id, row.step, row.quantity.as_str()));
            reopened_validity.sort_by_key(|row| (row.run_id, row.step, row.quantity.as_str()));
            assert_eq!(reopened_validity, validity_rows);
            let mut predictions = predictions;
            let mut reopened_observations = reopened_rows::<fit_observations::Row>(
                &runtime,
                &result,
                "runtime.fit_observations",
            )
            .await;
            predictions.sort_by_key(|row| (row.run_id, row.observation_id));
            reopened_observations.sort_by_key(|row| (row.run_id, row.observation_id));
            assert_eq!(reopened_observations, predictions);
        }
    }
}
