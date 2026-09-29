// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Covariance, Wald and profile-likelihood units (Plan 22 S3) on a weighted linear
//! regression `yᵢ = a + b·xᵢ`, whose covariance is `(XᵀWX)⁻¹` exactly.
use super::*;
use crate::workflow::{
    RunReport, RunResult,
    tests::{compiler_profile, id, physical, runtime},
};
use native::solve::Backend;
use pse_model::{
    scalar,
    scalars::{Fraction, PositiveCount},
};
use pse_relations::{
    columnar::RelationRow,
    generated::{
        enums::{IntervalEnd, IntervalMethod, IntervalOutcome},
        runtime::{
            local_validity, parameter_covariances, parameter_intervals, profile_points,
            response_directions,
        },
    },
};

const X: [f64; 4] = [1.0, 2.0, 3.0, 4.0];
const Y: [f64; 4] = [3.1, 4.9, 7.2, 8.8];
const SIGMA: [f64; 4] = [0.5, 1.0, 0.5, 2.0];
/// The parameters `a` and `b`, and their declared scales.
const A: u8 = 1;
const B: u8 = 2;
const D: u8 = 3;
const SCALES: [f64; 2] = [1.0, 2.0];

/// `(XᵀWX)⁻¹` row-major and the weighted least-squares estimate.
fn analytic() -> ([f64; 4], [f64; 2]) {
    let (mut s00, mut s01, mut s11, mut r0, mut r1) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for i in 0..4 {
        let w = 1.0 / (SIGMA[i] * SIGMA[i]);
        s00 += w;
        s01 += w * X[i];
        s11 += w * X[i] * X[i];
        r0 += w * Y[i];
        r1 += w * X[i] * Y[i];
    }
    let det = s00 * s11 - s01 * s01;
    let inverse = [s11 / det, -s01 / det, -s01 / det, s00 / det];
    let estimate = [
        inverse[0] * r0 + inverse[1] * r1,
        inverse[2] * r0 + inverse[3] * r1,
    ];
    (inverse, estimate)
}

/// The regression package: one steady experiment observing `y₁…y₄`, with `sigma` and
/// `importance` per observation.
fn package(sigma: [Option<f64>; 4], importance: [f64; 4]) -> crate::workflow::ModelingPackage {
    package_with(false, sigma, importance)
}
/// With `offset`, a third parameter `d` enters every output only through `a + d`, so the
/// two are not separately identifiable.
fn package_with(
    offset: bool,
    sigma: [Option<f64>; 4],
    importance: [f64; 4],
) -> crate::workflow::ModelingPackage {
    let mut physical = physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    let quantity = physical.quantities.neutral_dimensionless().unwrap();
    let unit = physical
        .quantities
        .quantity_type(quantity)
        .unwrap()
        .canonical_unit
        .as_id();
    let intercept = if offset { "(a + d)" } else { "a" };
    let outputs = (1..=4)
        .map(|i| format!("let y{i}: Scalar = {intercept} + b*{i};"))
        .collect::<String>();
    let rows = pse_authoring::language::parse(
        &format!(
            "package p {{ def Root {{ param a: Scalar = 1; param b: Scalar = 1; param d: Scalar = 0; {outputs} }} }}"
        ),
        id(20),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let mut data = FitData::default();
    data.datasets.push(serde_json::from_value(serde_json::json!({"dataset_id":id(30),"name":"regression","source":"unit","content_hash":ContentHash::from_bytes([1;32])})).unwrap());
    let mut observations = Vec::new();
    for i in 0..4 {
        let observation = id(40 + i as u8);
        data.observations.push(serde_json::from_value(serde_json::json!({"observation_id":observation,"dataset_id":id(30),"target":"y","value":Y[i],"unit_id":unit,"std_dev":sigma[i],"timestamp":null,"tag":null,"source_span":{"document_id":id(30),"start":0,"end":0}})).unwrap());
        observations.push(serde_json::json!({"observation_id":observation,"experiment_id":id(33),"output_path":format!("y{}", i + 1),"time":null,"included":true,"importance":importance[i]}));
    }
    let mut parameters = vec![
        serde_json::json!({"symbol_id":id(A),"fixed":false,"value":1.0,"lower":-100.0,"upper":100.0,"scale":SCALES[0]}),
        serde_json::json!({"symbol_id":id(B),"fixed":false,"value":1.0,"lower":-100.0,"upper":100.0,"scale":SCALES[1]}),
    ];
    let mut bindings = vec![
        serde_json::json!({"parameter_id":id(A),"path":"a"}),
        serde_json::json!({"parameter_id":id(B),"path":"b"}),
    ];
    if offset {
        parameters.push(serde_json::json!({"symbol_id":id(D),"fixed":false,"value":0.0,"lower":-100.0,"upper":100.0,"scale":1.0}));
        bindings.push(serde_json::json!({"parameter_id":id(D),"path":"d"}));
    }
    data.fits.push(serde_json::from_value(serde_json::json!({"fit_id":id(32),"parameters":parameters,
        "experiments":[{"experiment_id":id(33),"case_id":root,"route":"steady","bindings":bindings}],
        "observations":observations})).unwrap());
    let mut names = crate::workflow::tests::discrete_names();
    names.insert("Scalar".into(), quantity);
    runtime()
        .modeling_package(rows, physical, names)
        .unwrap()
        .with_fit_data(data)
        .unwrap()
}
fn declared() -> [Option<f64>; 4] {
    SIGMA.map(Some)
}
fn profile(hessian: HessianMode, uncertainty: Option<FitUncertainty>) -> FitProfile {
    FitProfile {
        solver: SolverProfile {
            presolve: Default::default(),
            numerics: Default::default(),
            convexity: Default::default(),
            intent: SolveIntent::Optimize,
            selection: native::solve::SolverSelection::Explicit(Backend::Ipopt),
            controls: native::solve::Controls {
                hessian,
                ..Default::default()
            },
            backend: native::execution::BackendSettings::Default,
            sensitivity: None,
        },
        simulations: BTreeMap::new(),
        modes: BTreeMap::new(),
        rank_tolerance: 1e-8,
        max_cells: 100_000,
        uncertainty,
    }
}
async fn fit(package: &crate::workflow::ModelingPackage, profile: FitProfile) -> Arc<RunResult> {
    package
        .prepare_fit(
            id(32).into(),
            profile,
            compiler_profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap()
}
fn report(result: &RunResult) -> &FitReport {
    let RunReport::Fit(report) = result.report().unwrap() else {
        panic!("fit report")
    };
    report
}
fn close(actual: f64, expected: f64, relative: f64) -> bool {
    (actual - expected).abs() <= relative * (1.0 + expected.abs())
}
/// The fit's validity rows by quantity.
fn validity(result: &RunResult) -> Vec<local_validity::Row> {
    local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap()
}

/// T11 and ADR-0118 item 8: an exact-Hessian fit's covariance is the inverse reduced
/// Hessian of its own KKT analysis, labelled exact; a quasi-Newton fit's is Gauss–Newton
/// from the response SVD. On a linear model both equal `(XᵀWX)⁻¹`.
#[tokio::test]
async fn linear_regression_covariance_analytic() {
    let (expected, estimate) = analytic();
    let package = package(declared(), [1.0; 4]);
    for (hessian, approximation) in [
        (HessianMode::Exact, CovarianceApproximation::Exact),
        (HessianMode::LimitedMemory, CovarianceApproximation::GaussNewton),
        (HessianMode::GaussNewton, CovarianceApproximation::GaussNewton),
    ] {
        let result = fit(&package, profile(hessian, None)).await;
        let report = report(&result);
        let candidate = report.candidate.as_ref().unwrap();
        for k in 0..2 {
            assert!(close(candidate[k], estimate[k], 1e-6), "{hessian:?} {candidate:?}");
        }
        let covariance = report.covariance.as_ref().unwrap();
        assert_eq!(covariance.approximation, approximation);
        assert_eq!(covariance.parameters, vec![id(A), id(B)]);
        let values = covariance.values.as_ref().unwrap();
        for (actual, expected) in values.iter().zip(expected) {
            assert!(close(*actual, expected, 1e-6), "{hessian:?} {values:?}");
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
        let directions = response_directions::Row::rows(
            &result.table("runtime.response_directions").unwrap(),
        )
        .unwrap();
        assert_eq!(directions.len(), 4);
        assert!(directions.iter().all(|d| d.identifiable));
    }
}

#[tokio::test]
async fn covariance_withheld_without_declared_sigma() {
    let mut sigma = declared();
    sigma[1] = None;
    let result = fit(&package(sigma, [1.0; 4]), profile(HessianMode::Exact, None)).await;
    let report = report(&result);
    // The fit itself is defined: the undeclared observation is weighted by one.
    assert!(report.estimate_qualified(), "{report:?}");
    let covariance = report.covariance.as_ref().unwrap();
    assert!(
        matches!(&covariance.values, Err(FitWithheld::UndeclaredDeviation(o)) if *o == vec![id(41)]),
        "{covariance:?}"
    );
    let validity = validity(&result);
    assert_eq!(validity.len(), 1);
    assert!(!validity[0].validity.certified);
    assert_eq!(
        validity[0].validity.reason,
        Some(WithheldReason::UndeclaredDeviation)
    );
    // A withheld quantity has no data rows.
    assert_eq!(
        result
            .table("runtime.parameter_covariances")
            .unwrap()
            .batch()
            .num_rows(),
        0
    );
}

#[tokio::test]
async fn covariance_withheld_with_nonunit_importance() {
    let result = fit(
        &package(declared(), [1.0, 1.0, 2.0, 1.0]),
        profile(
            HessianMode::Exact,
            Some(FitUncertainty {
                level: scalar!(Fraction(0.95)),
                profile: None,
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
        Some(Err(FitWithheld::Upstream(DerivedQuantity::ParameterCovariance)))
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

/// On a linear model the signed root `√(2(f − f*))` is linear in the pinned value, so the
/// profile-likelihood ends are the Wald ends. Two chains run at once.
#[tokio::test]
async fn profile_likelihood_matches_wald_on_linear_model() {
    let (expected, _) = analytic();
    let profile = profile(
        HessianMode::Exact,
        Some(FitUncertainty {
            level: scalar!(Fraction(0.9)),
            profile: Some(ProfileControls {
                points: scalar!(PositiveCount(20)),
                tolerance: scalar!(Fraction(1e-6)),
                workers: Some(scalar!(PositiveCount(2))),
            }),
        }),
    );
    let package = package(declared(), [1.0; 4]);
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
    let chains = |workers| {
        let progress = Arc::new(native::solve::Progress::new(16));
        prepared
            .problem
            .execute(
                prepared.route(),
                Arc::new(std::sync::atomic::AtomicBool::new(false)),
                progress,
                workers,
            )
            .unwrap()
            .profiles
            .unwrap()
            .unwrap()
    };
    assert_eq!(chains(1), chains(4));
    let result = fit(&package, profile).await;
    let report = report(&result);
    let wald = report.wald.as_ref().unwrap().as_ref().unwrap();
    let chains = report.profiles.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(chains.len(), 4);
    let z = 1.6448536269514722;
    for (k, interval) in wald.iter().enumerate() {
        let error = expected[3 * k].sqrt();
        let half = interval.upper.value.unwrap() - interval.estimate;
        assert!(close(half, z * error, 1e-6), "{interval:?}");
        for chain in chains.iter().filter(|c| c.parameter == interval.parameter) {
            assert_eq!(chain.bound.outcome, IntervalOutcome::Threshold, "{chain:?}");
            let wald = if chain.end == IntervalEnd::Lower {
                &interval.lower
            } else {
                &interval.upper
            };
            let value = chain.bound.value.unwrap();
            assert!(
                (value - wald.value.unwrap()).abs() <= 1e-5 * error,
                "{value} {wald:?} {chain:?}"
            );
            // The secant in the signed root is exact: two pinned fits reach the end.
            assert!(chain.points.len() <= 3, "{chain:?}");
        }
    }
    let rows = parameter_intervals::Row::rows(&result.table("runtime.parameter_intervals").unwrap())
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
    assert!(validity.iter().all(|r| r.validity.certified), "{validity:?}");
    assert_eq!(validity.len(), 3);
}

/// PS-11: each pinned fit is seeded from its chain's latest accepted point below the
/// threshold, or from the estimate, and the seed is recorded with the point. The offset
/// `d` leaves `a + d` identifiable but not `a` or `d`: the covariance is withheld with the
/// identifiable directions reported, while the profile, which needs no full rank, is flat
/// along `a` and `d` up to their bounds and sized from the declared scale.
#[tokio::test]
async fn profile_chain_seeds_from_predecessor() {
    let uncertainty = || {
        Some(FitUncertainty {
            level: scalar!(Fraction(0.95)),
            profile: Some(ProfileControls {
                points: scalar!(PositiveCount(40)),
                tolerance: scalar!(Fraction(1e-2)),
                workers: None,
            }),
        })
    };
    let mut sigma = declared();
    sigma[0] = None;
    let result = fit(
        &package_with(true, sigma, [1.0; 4]),
        profile(HessianMode::LimitedMemory, uncertainty()),
    )
    .await;
    // The profile needs the declared model too.
    assert!(matches!(
        report(&result).profiles,
        Some(Err(FitWithheld::UndeclaredDeviation(_)))
    ));
    let result = fit(
        &package_with(true, declared(), [1.0; 4]),
        profile(HessianMode::LimitedMemory, uncertainty()),
    )
    .await;
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
    assert!((unidentifiable[0] + unidentifiable[2]).abs() < 1e-6, "{directions:?}");
    assert!(unidentifiable[1].abs() < 1e-6, "{directions:?}");
    let chains = report.profiles.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(chains.len(), 6);
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
        assert_eq!(chain.points[0].seed, None, "{chain:?}");
        // The seed of every later point is the latest accepted point below the threshold.
        for (i, point) in chain.points.iter().enumerate().skip(1) {
            let predecessor = (0..i)
                .rev()
                .find(|&j| chain.points[j].accepted && r(&chain.points[j]) < 1.959963984540054);
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
    let package = package(declared(), [1.0; 4]);
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
    assert!(serde_json::from_str::<FitUncertainty>(r#"{"level": 0.95, "profile": {"points": 0}}"#).is_err());
    assert!(serde_json::from_str::<FitUncertainty>(r#"{"level": 0}"#).is_err());
    let decoded: FitUncertainty = serde_json::from_str(r#"{"level": 0.95, "profile": {}}"#).unwrap();
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
