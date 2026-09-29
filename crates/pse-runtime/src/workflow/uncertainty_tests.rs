// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Propagation units (Plan 22 S4): `Σ_y = J·Σ_θ·Jᵀ` through a modeling step's parametric
//! sensitivities and through a fit's responses, valid as the conjunction of their inputs.
use super::*;
use crate::{
    math::settings::Propagation,
    workflow::{
        FitUncertainty, fitting::regression as fixture,
        modeling::sensitivity_tests::{LINEAR, QUADRATIC, solve_propagating},
    },
};
use pse_backend_native::solve::{Backend, HessianMode, SolverSelection};
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{local_validity, propagated_covariances},
};

/// `Σ_θ` over `(a, b)`.
const SIGMA_AB: [[f64; 2]; 2] = [[0.04, 0.01], [0.01, 0.09]];

fn bound(values: &[f64]) -> Vec<FiniteBound> {
    values
        .iter()
        .map(|v| FiniteBound::try_new(*v).unwrap())
        .collect()
}
/// `J·Σ·Jᵀ` of a row-major `J` over two parameters.
fn sandwich(j: &[[f64; 2]], sigma: [[f64; 2]; 2]) -> Vec<f64> {
    let m = j.len();
    (0..m * m)
        .map(|index| {
            let (i, r) = (index / m, index % m);
            (0..2)
                .flat_map(|k| (0..2).map(move |l| (k, l)))
                .map(|(k, l)| j[i][k] * sigma[k][l] * j[r][l])
                .sum()
        })
        .collect()
}
fn close(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() <= 1e-6 * (1.0 + e.abs()), "{actual:?} vs {expected:?}");
    }
}
fn validity(result: &RunResult) -> local_validity::Row {
    local_validity::Row::rows(&result.table("runtime.local_validity").unwrap())
        .unwrap()
        .into_iter()
        .find(|r| r.quantity == DerivedQuantity::PropagatedCovariance)
        .unwrap()
}
fn propagated(result: &RunResult) -> Vec<propagated_covariances::Row> {
    propagated_covariances::Row::rows(&result.table("runtime.propagated_covariances").unwrap())
        .unwrap()
}
fn predictions(level: f64) -> Option<FitUncertainty> {
    Some(FitUncertainty {
        predictions: true,
        ..FitUncertainty::wald(pse_model::scalars::Fraction::try_new(level).unwrap())
    })
}

/// ADR-0118 items 1 and 11: on outputs linear in the parameters the propagation is exact,
/// `J·Σ_θ·Jᵀ`, whether `J` is a modeling step's parametric sensitivities, with parameters
/// matched by identity, or a fit's responses to its own predictions.
#[tokio::test]
async fn uncertainty_propagation_linear_exact() {
    // x = (b + 2a)/3 and y = (a − b)/3 at the optimum of the quadratic case.
    let run = RunId::from(SemanticId::from_bytes([9; 16]));
    let (result, ids) = solve_propagating(
        QUADRATIC,
        SolverSelection::Explicit(Backend::Ipopt),
        &["x", "y"],
        |parameters, outputs| {
            Some(Propagation {
                // The covariance names b before a: parameters match by identity.
                covariance: ParameterCovariance {
                    run_id: run,
                    parameters: vec![parameters[1], parameters[0]],
                    values: bound(&[
                        SIGMA_AB[1][1],
                        SIGMA_AB[1][0],
                        SIGMA_AB[0][1],
                        SIGMA_AB[0][0],
                    ]),
                },
                outputs: outputs.to_vec(),
            })
        },
    )
    .await;
    let expected = sandwich(&[[2.0 / 3.0, 1.0 / 3.0], [1.0 / 3.0, -1.0 / 3.0]], SIGMA_AB);
    let rows = propagated(&result);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].covariance_run_id, run);
    assert_eq!(rows[0].parameters, vec![ids[1], ids[0]]);
    assert_eq!(rows[0].outputs, vec![ids[2], ids[3]]);
    close(&rows[0].values, &expected);
    let row = validity(&result);
    assert!(row.validity.certified, "{row:?}");
    // It holds at the KKT point its sensitivities were read from.
    assert_eq!(row.validity.second_order, Some(true));

    // A fit's covariance propagated to its own predictions through its responses.
    let (sigma, _) = fixture::analytic();
    let result = fixture::fit(
        &fixture::package(fixture::declared(), [1.0; 4]),
        fixture::profile(HessianMode::Exact, predictions(0.95)),
    )
    .await;
    let design: Vec<[f64; 2]> = fixture::X.iter().map(|x| [1.0, *x]).collect();
    let expected = sandwich(&design, [[sigma[0], sigma[1]], [sigma[2], sigma[3]]]);
    let rows = propagated(&result);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].covariance_run_id, result.run_id);
    let observations: Vec<SemanticId> = (40..44).map(|n| SemanticId::from_bytes([n; 16])).collect();
    assert_eq!(rows[0].outputs, observations);
    close(&rows[0].values, &expected);
    assert!(validity(&result).validity.certified);
    // The entry point: the run's covariance handed on, through the same Jacobian.
    let input = result.parameter_covariance().unwrap().unwrap();
    assert_eq!(input.run_id, result.run_id);
    let jacobian = Jacobian {
        outputs: observations,
        parameters: input.parameters.clone(),
        values: design.iter().flatten().copied().collect(),
    };
    let direct = propagate(Ok(&input), Ok(&jacobian)).unwrap().unwrap();
    close(&direct.values, &expected);
    // A covariance parameter the Jacobian does not differentiate is a caller error.
    let mut foreign = jacobian.clone();
    foreign.parameters[1] = SemanticId::from_bytes([99; 16]);
    assert!(propagate(Ok(&input), Ok(&foreign)).is_err());
}

/// ADR-0118 item 11: the validity of a propagated covariance is the conjunction of its
/// inputs'; a withheld input withholds it, with the input's reason in its detail.
#[tokio::test]
async fn propagation_withheld_when_upstream_withheld() {
    // The coefficient route runs no KKT-point analysis, so the sensitivities are withheld.
    let run = RunId::from(SemanticId::from_bytes([9; 16]));
    let (result, _) = solve_propagating(
        LINEAR,
        SolverSelection::Explicit(Backend::Highs),
        &["x"],
        |parameters, outputs| {
            Some(Propagation {
                covariance: ParameterCovariance {
                    run_id: run,
                    parameters: parameters.to_vec(),
                    values: bound(&[1.0, 0.0, 0.0, 1.0]),
                },
                outputs: outputs.to_vec(),
            })
        },
    )
    .await;
    let row = validity(&result);
    assert!(!row.validity.certified);
    assert_eq!(row.validity.reason, Some(WithheldReason::UpstreamWithheld));
    let detail = row.validity.detail.unwrap();
    assert!(
        detail.contains("parametric_sensitivity") && detail.contains("no_local_analysis"),
        "{detail}"
    );
    assert!(propagated(&result).is_empty());

    // A fit whose covariance is withheld, here for a non-unit importance, withholds the
    // propagation to its predictions.
    let result = fixture::fit(
        &fixture::package(fixture::declared(), [1.0, 1.0, 2.0, 1.0]),
        fixture::profile(HessianMode::Exact, predictions(0.95)),
    )
    .await;
    let row = validity(&result);
    assert_eq!(row.validity.reason, Some(WithheldReason::UpstreamWithheld));
    assert!(
        row.validity
            .detail
            .is_some_and(|d| d.contains("parameter_covariance") && d.contains("nonunit_importance"))
    );
    assert!(propagated(&result).is_empty());
    let upstream = result.parameter_covariance().unwrap().unwrap_err();
    assert_eq!(upstream.reason, WithheldReason::NonunitImportance);
    let jacobian = Jacobian {
        outputs: vec![SemanticId::from_bytes([40; 16])],
        parameters: vec![SemanticId::from_bytes([1; 16])],
        values: vec![1.0],
    };
    assert_eq!(
        propagate(Err(upstream.clone()), Ok(&jacobian)).unwrap(),
        Err(upstream)
    );
    // A covariance that is not square, symmetric and nonnegative on its diagonal is refused.
    let skew = ParameterCovariance {
        run_id: run,
        parameters: vec![SemanticId::from_bytes([1; 16]), SemanticId::from_bytes([2; 16])],
        values: bound(&[1.0, 0.5, -0.5, 1.0]),
    };
    assert!(skew.admit().is_err());
}
