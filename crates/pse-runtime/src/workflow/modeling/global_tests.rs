// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored global, discrete and certified journeys through the real pipeline to SCIP
//! (Plan 22 G4, G5, G7).
use super::*;
use crate::math::solves::{NumericalInputs, Outcome, SolverProfile};
use crate::workflow::tests as fixture;
use pse_backend_native::{
    execution::{BackendSettings, ScipSettings},
    solve::{Assurance, Backend, Metric, Qualification, SolveIntent, SolverSelection},
};
use pse_compiler::workspace::ModelingCaseBindings;
use pse_kernels::DerivativeOrder;

/// A package on a runtime whose native jobs admit a foreign allowance SCIP takes as its
/// memory limit.
fn package(text: &str) -> (ModelingPackage, DeclarationId) {
    let physical = fixture::physical();
    let rows = pse_authoring::language::parse(
        text,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = fixture::runtime_with(16 << 20, 16 << 20, 2 << 30)
        .modeling_package(rows, physical)
        .unwrap();
    (package, root)
}
fn profile(intent: SolveIntent, selection: SolverSelection) -> SolverProfile {
    let mut profile = fixture::profile();
    profile.intent = intent;
    profile.selection = selection;
    profile
}
fn analysis(root: DeclarationId, solver: SolverProfile) -> ModelingAnalysis {
    ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: ModelingCaseBindings::default(),
        order: DerivativeOrder::First,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: NumericalInputs::default(),
    }
}
async fn solve(package: &ModelingPackage, analysis: &ModelingAnalysis) -> ModelingResult {
    let cancel = crate::CancelSource::new();
    let prepared = package.prepare_analysis(analysis, &cancel).await.unwrap();
    package
        .solve_case(prepared, analysis.compiler, &cancel)
        .await
        .unwrap()
}
fn report(result: &ModelingResult, label: &str) -> f64 {
    result
        .reports
        .iter()
        .find(|r| r.label == label)
        .unwrap()
        .value
}

/// The price-taker with a quadratic operating cost: an MIQP. Enumerating the eight
/// commitments, only h3 is worth running: its interior optimum x = 1.2/0.01 = 120 W earns
/// 144 − 72 − 50 = 22 W. Running h2 alone loses 18 W and h2 with h3 shares the 150 W
/// budget at 55 W and 95 W for −2.25 W.
const PRICE_TAKER: &str = "package p {
    entity kind period {} entity period h1 {} entity period h2 {} entity period h3 {}
    set periods: Set<period> = {h1, h2, h3};
    table price[t: period]: Scalar complete_over(t in periods);
    dataset signal: price source \"synthetic price signal\" { [h1] = [-0.5]; [h2] = [0.8]; [h3] = [1.2]; }
    def Root {
      param capacity: Power = 150{W}; param minimum: Power = 40{W};
      param standby: Power = 50{W}; param budget: Power = 150{W};
      param scale: Power = 100{W}; param curvature: Power = 50{W};
      var on[t in periods]: Indicator in binary;
      var output[t in periods]: Power;
      var load[t in periods]: Scalar;
      eq most[t in periods]: output[t] <= capacity*on[t];
      eq least[t in periods]: output[t] >= minimum*on[t];
      eq energy: sum(t in periods | output[t]) <= budget;
      eq loading[t in periods]: output[t] == scale*load[t];
      let margin: Power = sum(t in periods | price[t]*output[t] - curvature*load[t]*load[t] - standby*on[t]);
      let peak: Power = output[h3];
      annotation objective margin(maximize);
      annotation bounds output(0{W}, 150{W}); annotation bounds load(0, 1.5);
      annotation start on(0{1}); annotation start output(0{W}); annotation start load(0);
      annotation report margin(\"margin\"); annotation report peak(\"h3\"); } }";

#[tokio::test]
async fn price_taker_quadratic_cost_miqp() {
    let (package, root) = package(PRICE_TAKER);
    // Only SCIP represents a mixed-integer quadratic program; HiGHS refuses the class.
    let explicit = analysis(
        root,
        profile(
            SolveIntent::Optimize,
            SolverSelection::Explicit(Backend::Highs),
        ),
    );
    assert!(
        package
            .prepare_analysis(&explicit, &crate::CancelSource::new())
            .await
            .is_err()
    );
    let result = solve(
        &package,
        &analysis(root, profile(SolveIntent::Optimize, SolverSelection::Auto)),
    )
    .await;
    let Outcome::Native(native) = &result.outcome else {
        panic!("{:?}", result.outcome);
    };
    assert_eq!(native.backend, Backend::Scip);
    assert!(result.accepted, "{:?}", result.validation_error);
    assert!(
        (report(&result, "margin") - 22.0).abs() < 1e-5,
        "{}",
        report(&result, "margin")
    );
    assert!((report(&result, "h3") - 120.0).abs() < 1e-4);
    assert_eq!(native.qualification, Qualification::GapQualified);
    assert_eq!(native.termination.assurance, Assurance::GlobalBound);
}

/// A nested implicit block: y solves y² = x on [0.5, 3]. The objective
/// (y − 1)²(y − 2.5)² + y/10 has two basins: its global minimum near y = 0.98 and a
/// higher one near y = 2.49.
const IMPLICIT: &str = "package p { def Root { var x: Scalar;
    implicit root { var y: Scalar; eq residual: y*y == x; annotation bounds y(0.5, 3); annotation start y(1); }
    realize policy on root using nested;
    let f: Scalar = (root.y-1)*(root.y-1)*(root.y-2.5)*(root.y-2.5) + 0.1*root.y;
    annotation bounds x(0.25, 9); annotation start x(2);
    annotation objective f(minimize); annotation report f(\"f\"); } }";
/// The minimum of the objective over y ∈ [lower, upper], by a dense grid refined by
/// golden-section search: the independent oracle of the certified value.
fn implicit_oracle(lower: f64, upper: f64) -> f64 {
    let f = |y: f64| (y - 1.0).powi(2) * (y - 2.5).powi(2) + 0.1 * y;
    let n = 100_000;
    let step = (upper - lower) / f64::from(n);
    let best = (0..=n)
        .map(|i| lower + step * f64::from(i))
        .min_by(|a, b| f(*a).total_cmp(&f(*b)))
        .unwrap();
    let (mut a, mut b) = ((best - step).max(lower), (best + step).min(upper));
    let ratio = (5.0_f64.sqrt() - 1.0) / 2.0;
    for _ in 0..200 {
        let (c, d) = (b - ratio * (b - a), a + ratio * (b - a));
        if f(c) < f(d) {
            b = d;
        } else {
            a = c;
        }
    }
    f(0.5 * (a + b))
}

#[tokio::test]
async fn certify_exports_implicit_residuals_exactly() {
    let (package, root) = package(IMPLICIT);
    let result = solve(
        &package,
        &analysis(root, profile(SolveIntent::Certify, SolverSelection::Auto)),
    )
    .await;
    let Outcome::Native(native) = &result.outcome else {
        panic!("{:?}", result.outcome);
    };
    assert_eq!(native.backend, Backend::Scip);
    // The block's residual is exported in place of its nested realization, so the
    // program is exact and the certificate is a gap claim.
    assert_eq!(
        native.metrics["export.fidelity"],
        Metric::Text("exact".into())
    );
    assert_eq!(native.qualification, Qualification::GapQualified);
    assert_eq!(native.termination.assurance, Assurance::GlobalBound);
    assert!(result.accepted, "{:?}", result.validation_error);
    let global = implicit_oracle(0.5, 3.0);
    assert!(
        (report(&result, "f") - global).abs() < 1e-5,
        "{} vs {global}",
        report(&result, "f")
    );
    // A case bound on the unknown replaces its bound hint in the exported interval: with
    // y ≥ 1.5 the certified minimum is the other basin's.
    let mut bounded = analysis(root, profile(SolveIntent::Certify, SolverSelection::Auto));
    bounded.case.variables.insert(
        "root.y".into(),
        pse_compiler::workspace::ModelingVariableState {
            lower: Some(Some(1.5)),
            ..Default::default()
        },
    );
    // The inner solve's deterministic start lies inside the case interval.
    bounded.case.values.insert("root.y".into(), 2.0);
    let result = solve(&package, &bounded).await;
    assert!(result.accepted, "{:?}", result.validation_error);
    let restricted = implicit_oracle(1.5, 3.0);
    assert!(restricted > global + 0.1);
    assert!(
        (report(&result, "f") - restricted).abs() < 1e-5,
        "{} vs {restricted}",
        report(&result, "f")
    );
}

const OBSTRUCTION: &str = "package p { def Root { var x:Scalar;
    eq lo: x*x >= 4; eq hi: x*x <= 1; eq spare: x*x <= 100;
    annotation start x(1.5); annotation bounds x(-3, 3); } }";

#[tokio::test]
async fn certified_infeasibility_beside_local_explanation() {
    let (package, root) = package(OBSTRUCTION);
    let mut solver = profile(SolveIntent::FeasiblePoint, SolverSelection::Auto);
    solver.backend = BackendSettings::Default;
    let analysis = analysis(root, solver);
    let certificate = package
        .certify_infeasibility(&analysis, &crate::CancelSource::new())
        .await
        .unwrap();
    // A global conclusion with its own assurance; the elastic explanation stays local.
    assert!(certificate.proven());
    assert_eq!(certificate.assurance, Assurance::ProvenInfeasible);
    assert_eq!(
        certificate.fidelity,
        Some(pse_math::factorable::Fidelity::Exact)
    );
    assert!(certificate.irreducible);
    let row = |name: &str| {
        certificate
            .result
            .prepared
            .model
            .model
            .compiled()
            .model
            .equations
            .iter()
            .find(|r| r.lineage.path.ends_with(name))
            .unwrap()
            .id
    };
    let names = certificate
        .result
        .prepared
        .model
        .model
        .compiled()
        .model
        .equations
        .iter()
        .filter(|r| certificate.rows.contains(&r.id))
        .map(|r| r.lineage.path.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        certificate.rows,
        [row(".lo"), row(".hi")].into_iter().collect(),
        "{names:?} {:?}",
        certificate.members
    );
    assert!(!certificate.result.accepted);
    // A feasible model yields no proof.
    let (feasible, root) = package_feasible();
    let certificate = feasible
        .certify_infeasibility(
            &self::analysis(
                root,
                profile(SolveIntent::FeasiblePoint, SolverSelection::Auto),
            ),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert!(!certificate.proven());
    assert!(certificate.members.is_empty());
}
fn package_feasible() -> (ModelingPackage, DeclarationId) {
    package(&OBSTRUCTION.replace("eq hi: x*x <= 1;", ""))
}

#[tokio::test]
async fn solution_pool_published() {
    use pse_relations::{columnar::RelationRow, generated::runtime::solution_pool};
    let (package, root) = package(PRICE_TAKER);
    let mut solver = profile(SolveIntent::Optimize, SolverSelection::Auto);
    solver.backend = BackendSettings::Scip(ScipSettings {
        pool: 3,
        ..ScipSettings::default()
    });
    let cancel = crate::CancelSource::new();
    let prepared = package
        .prepare_analysis(&analysis(root, solver), &cancel)
        .await
        .unwrap();
    let result = prepared.start().unwrap().wait().await.unwrap();
    let table = result.table("runtime.solution_pool").unwrap();
    let rows = solution_pool::Row::rows(&table).unwrap();
    assert!(!rows.is_empty());
    // Nine free variables per ranked solution, ranks from zero, every one re-qualified.
    let ranks: std::collections::BTreeSet<_> = rows.iter().map(|r| r.rank).collect();
    assert_eq!(ranks.iter().next(), Some(&0));
    for rank in &ranks {
        let solution: Vec<_> = rows.iter().filter(|r| r.rank == *rank).collect();
        assert_eq!(solution.len(), 9);
        assert!(solution.iter().all(|r| r.feasible == Some(true)));
    }
    let best = rows.iter().find(|r| r.rank == 0).unwrap();
    assert!((best.objective.unwrap() - 22.0).abs() < 1e-5);
    for pair in ranks.iter().collect::<Vec<_>>().windows(2) {
        let objective = |rank: i64| {
            rows.iter()
                .find(|r| r.rank == rank)
                .unwrap()
                .objective
                .unwrap()
        };
        assert!(objective(*pair[0]) >= objective(*pair[1]) - 1e-9);
    }
}
