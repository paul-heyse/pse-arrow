// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Results conditional on a discrete assignment through the real pipeline (ADR-0118 items
//! 4 and 9; Plan 22 M2c): the HiGHS fixed-commitment LP and the SCIP fixed-assignment
//! re-solve price their candidate under one `Commitment`, which `runtime.solve_runs` states
//! once per step, and the local-analysis quantities read from those multipliers are
//! conditional on it.
use super::*;
use crate::math::{
    settings::SensitivityRequest,
    solves::{NumericalInputs, SolverProfile},
};
use crate::workflow::tests as fixture;
use pse_backend_native::{
    execution::BackendSettings,
    settings::highs,
    solve::{Backend, SolveIntent, SolverSelection},
};
use pse_compiler::workspace::ModelingCaseBindings;
use pse_kernels::DerivativeOrder;
use pse_relations::{
    columnar::RelationRow,
    generated::{
        enums::{DerivedQuantity, NativeBackend, NativeQualification, NumericalTarget},
        runtime::{local_validity, parametric_sensitivities, solve_constraints, solve_runs},
    },
};
use std::sync::Arc;

/// `max x + 2y` as `min −x − 2y` with `x + y ≤ 3.5 W` and `x ≤ 0.4 W`, integer y ∈ [0, 5]:
/// the MIP commits to y = 3, and under it only `x ≤ 0.4 W` binds.
const MILP: &str = "package p {
    def Root {
      param unit: Power = 1{W};
      var x: Power; var y: Count in integer;
      eq total: x + unit*y <= 3.5{W};
      eq cap: x <= 0.4{W};
      let cost: Power = -x - 2*unit*y;
      annotation objective cost(minimize);
      annotation bounds x(0{W}, 10{W}); annotation bounds y(0{1}, 5{1});
      annotation start x(0{W}); annotation start y(0{1}); } }";
/// `min ½x² + y² − b·x + ½z² + a·z (+ 3·build)  s.t.  x + y = a`, x, y ∈ [−10, 10] and
/// z ∈ [0, 10], at
/// (a, b) = (1, 2). With `build`, a binary at cost 3 W joins: a mixed-integer program whose
/// optimal assignment is build = 0 and whose continuous problem under it is the continuous
/// case, with λ = 2(b − a)/3 on the coupling row and dx/d(a, b) = (2/3, 1/3).
fn quadratic(build: bool) -> String {
    let (declaration, term) = if build {
        ("var build: Indicator in binary; annotation start build(1{1});", " + fee*build")
    } else {
        ("", "")
    };
    format!(
        "package p {{
    def Root {{
      param a: Scalar = 1; param b: Scalar = 2;
      param unit: Power = 1{{W}}; param fee: Power = 3{{W}};
      var x: Scalar; var y: Scalar; var z: Scalar; {declaration}
      eq coupling: x + y == a;
      let cost: Power = unit*(0.5*x*x + y*y - b*x + 0.5*z*z + a*z){term};
      annotation objective cost(minimize);
      annotation bounds x(-10, 10); annotation bounds y(-10, 10); annotation bounds z(0, 10);
      annotation start x(0.5); annotation start y(0.5); annotation start z(0.5); }} }}"
    )
}

fn package(text: &str) -> (ModelingPackage, DeclarationId) {
    let physical = fixture::physical();
    let mut names = fixture::discrete_names();
    names.insert(
        "Scalar".into(),
        physical.quantities.neutral_dimensionless().unwrap(),
    );
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
        .modeling_package(rows, physical, names)
        .unwrap();
    (package, root)
}
fn analysis(root: DeclarationId, solver: SolverProfile, demand: &[&str]) -> ModelingAnalysis {
    ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings {
            demand: demand.iter().map(|p| (*p).into()).collect(),
            ..Bindings::default()
        },
        limits: Limits::default(),
        case: ModelingCaseBindings::default(),
        order: DerivativeOrder::Second,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: NumericalInputs::default(),
    }
}
fn profile(selection: SolverSelection, backend: BackendSettings) -> SolverProfile {
    let mut solver = fixture::profile();
    solver.intent = SolveIntent::Optimize;
    solver.selection = selection;
    solver.backend = backend;
    solver
}
/// Solve `analysis` as a one-step run; returns the result and the identities of `names`:
/// a demanded symbol path or an equation's name.
async fn solve(
    package: &ModelingPackage,
    analysis: &ModelingAnalysis,
    names: &[&str],
) -> (Arc<crate::workflow::RunResult>, Vec<SemanticId>) {
    let cancel = crate::CancelSource::new();
    let prepared = package.prepare_analysis(analysis, &cancel).await.unwrap();
    let model = &prepared.model.model.compiled().model;
    let ids = names
        .iter()
        .map(|name| {
            model.paths.get(*name).copied().unwrap_or_else(|| {
                model
                    .equations
                    .iter()
                    .find(|e| e.lineage.path.rsplit('.').next() == Some(*name))
                    .unwrap()
                    .id
            })
        })
        .collect();
    let result = prepared.start().unwrap().wait().await.unwrap();
    (result, ids)
}
fn rows<R: RelationRow>(result: &crate::workflow::RunResult, name: &str) -> Vec<R> {
    R::rows(&result.table(name).unwrap()).unwrap()
}
fn run(result: &crate::workflow::RunResult) -> solve_runs::Row {
    let mut runs: Vec<solve_runs::Row> = rows(result, "runtime.solve_runs");
    assert_eq!(runs.len(), 1);
    runs.remove(0)
}
fn backend(result: &crate::workflow::RunResult) -> NativeBackend {
    let run = run(result);
    assert_ne!(run.qualification, NativeQualification::Unqualified, "{run:?}");
    run.backend.unwrap()
}
/// The step's commitment as `(column, value)` pairs.
fn commitment(result: &crate::workflow::RunResult) -> Option<Vec<(SemanticId, f64)>> {
    run(result).commitment.as_ref().map(|c| {
        c.iter()
            .map(|item| (item.source_id, item.value))
            .collect()
    })
}
fn dual(result: &crate::workflow::RunResult, row: SemanticId) -> Option<f64> {
    rows::<solve_constraints::Row>(result, "runtime.solve_constraints")
        .into_iter()
        .find(|r| r.row_id == row)
        .unwrap()
        .dual
}

/// The multipliers of a mixed-integer candidate are those of its continuous problem under
/// the assignment, and the step states that assignment: the HiGHS fixed-commitment LP
/// prices the MILP candidate, the SCIP fixed-assignment re-solve is the MIQP candidate.
/// Without either, no commitment is stated and a MIP candidate has no multipliers.
#[tokio::test]
async fn duals_conditional_on_assignment() {
    let (package, root) = package(MILP);
    let paths = ["y", "total", "cap"];
    let priced = BackendSettings::Highs(highs::Settings {
        diagnostics: highs::Request {
            fixed_lp: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let explicit = SolverSelection::Explicit(Backend::Highs);
    let (result, ids) = solve(
        &package,
        &analysis(root, profile(explicit, priced), &paths[..1]),
        &paths,
    )
    .await;
    let [y, total, cap] = ids[..] else {
        unreachable!()
    };
    assert_eq!(backend(&result), NativeBackend::Highs);
    assert_eq!(commitment(&result), Some(vec![(y, 3.0)]));
    // Under y = 3 the row x ≤ 0.4 W binds and x + y ≤ 3.5 W does not.
    let (d_total, d_cap) = (dual(&result, total).unwrap(), dual(&result, cap).unwrap());
    assert!(d_total.abs() < 1e-9 && (d_cap.abs() - 1.0).abs() < 1e-9, "{d_total} {d_cap}");
    // Without the fixed-commitment LP the MIP candidate has neither.
    let (plain, _) = solve(
        &package,
        &analysis(root, profile(explicit, BackendSettings::Default), &paths[..1]),
        &paths,
    )
    .await;
    assert_eq!(commitment(&plain), None);
    assert_eq!(dual(&plain, cap), None);

    // The MIQP routes to SCIP; its candidate is the re-solve under build = 0, whose coupling
    // multiplier is the continuous case's.
    let paths = ["build", "coupling"];
    let (package, root) = self::package(&quadratic(true));
    let auto = profile(SolverSelection::Auto, BackendSettings::Default);
    let (result, ids) = solve(&package, &analysis(root, auto, &paths[..1]), &paths).await;
    assert_eq!(backend(&result), NativeBackend::Scip);
    assert_eq!(commitment(&result), Some(vec![(ids[0], 0.0)]));
    let conditional = dual(&result, ids[1]).unwrap();
    let (package, root) = self::package(&quadratic(false));
    let ipopt = profile(
        SolverSelection::Explicit(Backend::Ipopt),
        BackendSettings::Default,
    );
    let (continuous, ids) = solve(&package, &analysis(root, ipopt, &[]), &paths[1..]).await;
    assert_eq!(backend(&continuous), NativeBackend::Ipopt);
    assert_eq!(commitment(&continuous), None);
    let reference = dual(&continuous, ids[0]).unwrap();
    assert!((reference.abs() - 2.0 / 3.0).abs() < 1e-6, "{reference}");
    assert!((conditional - reference).abs() < 1e-6, "{conditional} {reference}");
}

/// Sensitivities read at the SCIP re-solve's candidate equal those of the continuous
/// problem under its assignment, and their validity rows are conditional on the
/// commitment the step states (ADR-0118 item 4).
#[tokio::test]
async fn sensitivity_conditional_on_assignment() {
    async fn sensitivities(
        build: bool,
    ) -> (
        Arc<crate::workflow::RunResult>,
        Vec<SemanticId>,
        Vec<parametric_sensitivities::Row>,
    ) {
        let paths = ["a", "b", "x", "build"];
        let paths = if build { &paths[..] } else { &paths[..3] };
        let (package, root) = package(&quadratic(build));
        // The continuous reference runs on the NLP backend the re-solve routes to.
        let selection = if build {
            SolverSelection::Auto
        } else {
            SolverSelection::Explicit(Backend::Ipopt)
        };
        let mut analysis = analysis(root, profile(selection, BackendSettings::Default), paths);
        let plain = package
            .prepare_analysis(&analysis, &crate::CancelSource::new())
            .await
            .unwrap();
        let known = &plain.model.model.compiled().model.paths;
        let parameters = vec![known["a"], known["b"]];
        analysis.solver.sensitivity = Some(SensitivityRequest {
            parameters,
            reduced_hessian: true,
            propagation: None,
        });
        let (result, ids) = solve(&package, &analysis, paths).await;
        let rows = rows(&result, "runtime.parametric_sensitivities");
        (result, ids, rows)
    }
    let (result, ids, conditional) = sensitivities(true).await;
    assert_eq!(backend(&result), NativeBackend::Scip);
    assert_eq!(commitment(&result), Some(vec![(ids[3], 0.0)]));
    let validity: Vec<local_validity::Row> = rows(&result, "runtime.local_validity");
    assert_eq!(validity.len(), 2);
    for quantity in [
        DerivedQuantity::ParametricSensitivity,
        DerivedQuantity::ReducedHessian,
    ] {
        let row = validity.iter().find(|r| r.quantity == quantity).unwrap();
        assert!(row.validity.certified, "{:?}", row.validity);
        assert!(row.validity.conditional, "{quantity:?}");
    }
    let (continuous, unconditional, reference) = sensitivities(false).await;
    assert_eq!(backend(&continuous), NativeBackend::Ipopt);
    assert_eq!(commitment(&continuous), None);
    let validity: Vec<local_validity::Row> = rows(&continuous, "runtime.local_validity");
    assert!(validity.iter().all(|r| r.validity.certified && !r.validity.conditional));
    // dx/da = 2/3 and dx/db = 1/3 on both routes.
    let find = |rows: &[parametric_sensitivities::Row], ids: &[SemanticId], k: usize| {
        rows.iter()
            .find(|r| {
                r.parameter_id == ids[k]
                    && r.target_kind == NumericalTarget::Variable
                    && r.target_id == ids[2]
            })
            .and_then(|r| r.primal)
            .unwrap()
    };
    for (k, expected) in [(0, 2.0 / 3.0), (1, 1.0 / 3.0)] {
        let (a, b) = (
            find(&conditional, &ids, k),
            find(&reference, &unconditional, k),
        );
        assert!((a - expected).abs() < 1e-6 && (a - b).abs() < 1e-6, "{a} {b}");
    }
}
