// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! POUNCE-convex through the real pipeline (Plan 22 N5): a study batched on the admitted
//! threads, quadratic-program sensitivities from the one KKT-point analysis, and
//! sum-of-squares bounds labelled non-rigorous.
use super::sensitivity_tests::{QUADRATIC, analysis, package, rows, solve};
use super::*;
use crate::math::solves::Outcome;
use crate::workflow::tests as fixture;
use pse_backend_native::solve::{Backend, Metric, SolverSelection};
use pse_model::generated::enums::{
    DerivedQuantity, NativeAssurance, NativeBackend, NumericalTarget,
};
use pse_relations::generated::runtime::{
    local_validity, parametric_sensitivities, resolved_numerics, solve_runs,
};

/// `min (x − a)² + (y − b)²  s.t.  x + y ≤ 1`, x, y ∈ [0, 5]: the projection of (a, b) onto
/// the simplex, a convex quadratic program the coefficient route solves. With b = 0.4 no
/// studied `a` puts (a, b) on the row (a = 0.6) or at the corner's degenerate edge
/// (a = 1.4), so every optimum is strictly complementary.
const PROJECTION: &str = "package p {
    def Root {
      param a: Scalar = 1; param b: Scalar = 0.4;
      var x: Scalar; var y: Scalar;
      eq budget: x + y <= 1;
      let cost: Scalar = (x - a)*(x - a) + (y - b)*(y - b);
      annotation objective cost(minimize);
      annotation bounds x(0, 5); annotation bounds y(0, 5);
      annotation start x(0); annotation start y(0); } }";

/// A study of the projection at eight values of `a`, each point independent.
async fn study(selection: SolverSelection, threads: usize) -> Vec<(f64, f64, ModelingResult)> {
    let (package, root) = package(PROJECTION).await;
    let mut base = analysis(root, selection);
    base.solver.controls.threads = threads;
    base.bindings.demand = vec!["x".into(), "y".into(), "a".into(), "b".into()];
    let values = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];
    let cancel = crate::CancelSource::new();
    let mut preparations = Vec::new();
    for a in values {
        let mut analysis = base.clone();
        analysis.case.values.insert("a".into(), a);
        preparations.push(package.prepare_analysis(&analysis, &cancel).await.unwrap());
    }
    let mut staged = crate::workflow::staged::Staged::open(&package.runtime, None).unwrap();
    let results = staged
        .batch(&preparations, assessment::Obligations::Final, &cancel)
        .await;
    staged.close().await;
    results
        .into_iter()
        .map(|result| {
            let result = result.unwrap();
            let paths = &result.prepared.model.model.compiled().model.paths;
            let at = |p: &str| result.values.scalars[&paths[p]];
            (at("x"), at("y"), result)
        })
        .collect()
}

/// A study whose points select POUNCE-convex runs as one parallel batch on two admitted
/// threads (Plan 22 N5): every point is accepted on its own, records the batch it ran in,
/// and both adapters reach the independent projection objective within their own
/// production-resolved allowance.
#[tokio::test]
async fn pounce_convex_batched_study() {
    let batched = study(SolverSelection::Explicit(Backend::PounceConvex), 2).await;
    let reference = study(SolverSelection::Auto, 1).await;
    assert_eq!(batched.len(), 8);
    assert_eq!(reference.len(), 8);
    for (k, ((x, y, result), (rx, ry, reference))) in batched.iter().zip(&reference).enumerate() {
        assert!(result.accepted, "{k}: {:?}", result.diagnostic());
        assert!(reference.accepted, "{k}: {:?}", reference.diagnostic());
        let Outcome::Native(native) = &result.outcome else {
            panic!("{:?}", result.outcome)
        };
        assert_eq!(native.backend, Backend::PounceConvex, "{k}");
        assert_eq!(native.metrics["batch"], Metric::Integer(8), "{k}");
        let Outcome::Native(highs) = &reference.outcome else {
            panic!("{:?}", reference.outcome)
        };
        assert_eq!(highs.backend, Backend::Highs, "{k}");
        assert_eq!(result.prepared.profile.controls.threads, 2);
        assert_eq!(reference.prepared.profile.controls.threads, 1);
        for (x, y, result, native) in [(*x, *y, result, native), (*rx, *ry, reference, highs)] {
            let model = &result.prepared.model.model.compiled().model;
            let at = |name: &str| result.values.scalars[&model.paths[name]];
            let a = at("a");
            let b = at("b");
            assert_eq!(a, [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0][k]);
            assert_eq!(b, 0.4);
            let (opt_x, opt_y) = if a <= 0.6 {
                (a, 0.4)
            } else if a < 1.4 {
                ((a + 0.6) / 2.0, (1.4 - a) / 2.0)
            } else {
                (1.0, 0.0)
            };
            let optimum = (opt_x - a).powi(2) + (opt_y - b).powi(2);
            let actual = (x - a).powi(2) + (y - b).powi(2);
            let numerics = result.prepared.solve.numerics();
            let objective_budget =
                fixture::engineering_target(numerics, NumericalTarget::Objective, SemanticId::NIL)
                    .budget;
            assert!(
                (actual - optimum).abs() <= objective_budget,
                "point {k}: actual={actual}, optimum={optimum}"
            );
            assert!(
                (native.candidate.as_ref().unwrap().objective.unwrap() - actual).abs()
                    <= objective_budget
            );
            let row = model
                .equations
                .iter()
                .find(|row| row.lineage.path.ends_with(".budget"))
                .unwrap()
                .id;
            assert!(
                x + y
                    <= 1.0
                        + fixture::engineering_target(numerics, NumericalTarget::Row, row).budget
            );
            for (name, value) in [("x", x), ("y", y)] {
                let allowance = fixture::engineering_target(
                    numerics,
                    NumericalTarget::Variable,
                    model.paths[name],
                )
                .budget;
                assert!(value >= -allowance && value <= 5.0 + allowance);
            }
            assert!(native.quality.as_ref().unwrap().feasible());
        }
    }
}

/// A convex quadratic program with a sensitivity request (ADR-0118; I5): automatic routing
/// prefers an adapter whose candidate the one KKT-point analysis differentiates, so the
/// program that HiGHS would own routes to an NLP adapter and its sensitivities are
/// certified and analytic, `dx/da = 2/3` and `dx/db = 1/3`. No second, QP-specific
/// sensitivity mechanism exists.
#[tokio::test]
async fn qp_sensitivity_through_kkt_analysis() {
    let (package, root) = package(QUADRATIC).await;
    let plain = package
        .prepare_analysis(
            &analysis(root, SolverSelection::Auto),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(plain.solve.backend(), Some(Backend::Highs));
    let (result, [a, b, x]) = solve(QUADRATIC, SolverSelection::Auto).await;
    assert!(result.usable());
    let runs: Vec<solve_runs::Row> = rows(&result, "runtime.solve_runs");
    let backend = runs[0].backend.unwrap();
    assert!(
        matches!(backend, NativeBackend::Ipopt | NativeBackend::Pounce),
        "{backend:?}"
    );
    let sensitivities: Vec<parametric_sensitivities::Row> =
        rows(&result, "runtime.parametric_sensitivities");
    let primal = |parameter| {
        sensitivities
            .iter()
            .find(|r| r.parameter_id == parameter && r.target_id == x)
            .and_then(|r| r.primal)
            .unwrap()
    };
    let numerics: Vec<resolved_numerics::Row> = rows(&result, "runtime.resolved_numerics");
    let target = |id| {
        numerics
            .iter()
            .find(|row| row.target_kind == NumericalTarget::Variable && row.target_id == id)
            .unwrap()
    };
    let output = target(x);
    assert_eq!(
        output.engineering.as_ref().unwrap().relative_fraction,
        pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction
    );
    for (parameter, expected) in [(a, 2.0 / 3.0), (b, 1.0 / 3.0)] {
        let input = target(parameter);
        assert_eq!(
            input.engineering.as_ref().unwrap().relative_fraction,
            pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction
        );
        let actual = primal(parameter);
        let response_error =
            (actual - expected).abs() * input.coordinate_scale / output.coordinate_scale;
        assert!(
            actual.is_finite()
                && response_error
                    <= output.engineering.as_ref().unwrap().budget / output.coordinate_scale
        );
        let row = sensitivities
            .iter()
            .find(|row| {
                row.parameter_id == parameter
                    && row.target_kind == NumericalTarget::Variable
                    && row.target_id == x
            })
            .unwrap();
        assert_eq!(row.parameter_unit_id, input.unit_id);
        assert_eq!(row.target_unit_id, output.unit_id);
    }
    let validity: Vec<local_validity::Row> = rows(&result, "runtime.local_validity");
    let sensitivity = validity
        .iter()
        .find(|row| row.quantity == DerivedQuantity::ParametricSensitivity)
        .unwrap();
    assert!(sensitivity.validity.certified && !sensitivity.validity.conditional);
}

/// `min x⁴ − 3x² + x` over x ∈ [−2, 2], whose global minimum is ≈ −3.51391 at x ≈ −1.30:
/// the moment relaxation's empirical value agrees within its resolved objective
/// allowance and is labelled `sos_bound_nonrigorous`. The check makes no rigorous
/// direction claim, in either optimization sense. Non-polynomial programs are refused.
#[tokio::test]
async fn sos_bound_labelled_nonrigorous() {
    let quartic = |sense: &str, sign: &str| {
        format!(
            "package p {{ def Root {{ var x: Scalar;
              let cost: Scalar = {sign}(x*x*x*x - 3*x*x + x);
              annotation objective cost({sense});
              annotation bounds x(-2, 2); annotation start x(0); }} }}"
        )
    };
    // The global minimizer, by Newton on f′(x) = 4x³ − 6x + 1 from the left of its basin.
    let mut t: f64 = -1.3;
    for _ in 0..50 {
        t -= (4.0 * t.powi(3) - 6.0 * t + 1.0) / (12.0 * t * t - 6.0);
    }
    let minimum = t.powi(4) - 3.0 * t * t + t;
    assert!((minimum + 3.51391).abs() < 1e-4, "{minimum}");
    let cancel = crate::CancelSource::new();
    for (sense, sign, expected) in [("minimize", "", minimum), ("maximize", "-", -minimum)] {
        let (package, root) = package(&quartic(sense, sign)).await;
        let request = analysis(root, SolverSelection::Auto);
        let prepared = package.prepare_analysis(&request, &cancel).await.unwrap();
        let objective_budget = fixture::engineering_target(
            prepared.solve.numerics(),
            NumericalTarget::Objective,
            SemanticId::NIL,
        )
        .budget;
        let bound = package.sos_bound(&request, None, &cancel).await.unwrap();
        assert_eq!(
            bound.assurance,
            NativeAssurance::SosBoundNonrigorous,
            "{bound:?}"
        );
        assert_eq!(bound.order, 2);
        assert!(
            (bound.bound - expected).abs() <= objective_budget,
            "{sense}: {bound:?} against {expected}"
        );
    }
    let (package, root) = package(
        "package p { def Root { var x: Scalar;
          let cost: Scalar = exp(x) - x;
          annotation objective cost(minimize);
          annotation bounds x(-2, 2); annotation start x(0); } }",
    )
    .await;
    let refused = package
        .sos_bound(&analysis(root, SolverSelection::Auto), None, &cancel)
        .await
        .unwrap_err();
    assert!(refused.to_string().contains("polynomial"), "{refused}");
}
