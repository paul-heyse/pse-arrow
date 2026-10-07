// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Clarabel on authored linear programs, and typed infeasibility certificates published in
//! `runtime.infeasibility_certificates` (Plan 22 C4, I10–I11).
use super::*;
use crate::math::solves::{NumericalInputs, SolverProfile};
use crate::workflow::tests as fixture;
use pse_backend_native::solve::{
    Assurance, Backend, CertificateAccuracy, CertificateKind, RayCoordinate, SolveIntent,
    SolverSelection,
};
use pse_compiler::workspace::ModelingCaseBindings;
use pse_kernels::DerivativeOrder;
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{infeasibility_certificates, solve_metrics, solve_runs},
};

async fn package(text: &str) -> (ModelingPackage, DeclarationId) {
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
    let package = fixture::runtime()
        .modeling_package(rows, physical)
        .await
        .unwrap();
    (package, root)
}
fn analysis(root: DeclarationId, selection: SolverSelection) -> ModelingAnalysis {
    let solver = SolverProfile {
        intent: SolveIntent::Optimize,
        selection,
        ..fixture::profile()
    };
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
/// min x + y over x + y >= `cover` with x, y in [0, 1].
fn covering(cover: &str) -> String {
    format!(
        "package p {{ def Root {{ var x: Scalar; var y: Scalar;
        eq cover: x + y >= {cover};
        let total: Scalar = x + y;
        annotation objective total(minimize);
        annotation bounds x(0, 1); annotation bounds y(0, 1);
        annotation start x(0.5); annotation start y(0.5); }} }}"
    )
}
async fn run(text: &str, selection: SolverSelection) -> Arc<crate::workflow::RunResult> {
    let (package, root) = package(text).await;
    let prepared = package
        .prepare_analysis(&analysis(root, selection), &crate::CancelSource::new())
        .await
        .unwrap();
    prepared.start().unwrap().wait().await.unwrap()
}

/// Explicit Clarabel serves an authored LP through the coefficient runner's cone form,
/// while automatic routing keeps it on HiGHS when HiGHS is linked (ADR-0121).
#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
#[tokio::test]
async fn clarabel_serves_explicit_linear_program() {
    let text = covering("1.5");
    let clarabel = run(&text, SolverSelection::Explicit(Backend::Clarabel)).await;
    let rows = solve_runs::Row::rows(&clarabel.table("runtime.solve_runs").unwrap()).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].backend, Some(Backend::Clarabel));
    assert_eq!(rows[0].feasible, Some(true), "{rows:?}");
    let crate::workflow::RunReport::Modeling(reports) = clarabel.report().unwrap() else {
        panic!("expected modeling result")
    };
    let report = &reports[0];
    let allowance = fixture::engineering_target(
        report.prepared.solve.numerics(),
        pse_relations::generated::enums::NumericalTarget::Objective,
        SemanticId::NIL,
    )
    .engineering
    .as_ref()
    .unwrap()
    .budget;
    let model = &report.prepared.model.model.compiled().model;
    let member = |path: &str| {
        let members = model
            .symbols
            .iter()
            .filter(|(_, symbol)| symbol.lineage.path == path)
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        assert_eq!(members.len(), 1, "one original LP member at {path}");
        members[0]
    };
    let x = report.values.scalars[&member("Root.x")];
    let y = report.values.scalars[&member("Root.y")];
    assert_eq!(rows[0].objective.unwrap(), x + y);
    assert!(
        (rows[0].objective.unwrap() - 1.5).abs() <= allowance,
        "{rows:?}"
    );
    let certificates = infeasibility_certificates::Row::rows(
        &clarabel
            .table("runtime.infeasibility_certificates")
            .unwrap(),
    )
    .unwrap();
    assert!(certificates.is_empty());
    let automatic = run(&text, SolverSelection::Auto).await;
    let rows = solve_runs::Row::rows(&automatic.table("runtime.solve_runs").unwrap()).unwrap();
    if cfg!(feature = "solver-highs") {
        assert_eq!(rows[0].backend, Some(Backend::Highs));
    }
    assert_ne!(rows[0].backend, Some(Backend::Clarabel));
}

/// An infeasible authored LP on explicit Clarabel publishes its verified Farkas ray in
/// original coordinates, with the certificate assurance, and no string-keyed metrics.
#[tokio::test]
async fn infeasibility_certificate_published() {
    let result = run(&covering("3"), SolverSelection::Explicit(Backend::Clarabel)).await;
    let runs = solve_runs::Row::rows(&result.table("runtime.solve_runs").unwrap()).unwrap();
    assert_eq!(runs[0].assurance, Assurance::Certificate, "{runs:?}");
    let rows = infeasibility_certificates::Row::rows(
        &result.table("runtime.infeasibility_certificates").unwrap(),
    )
    .unwrap();
    assert_eq!(rows.len(), 1, "{rows:?}");
    let row = &rows[0];
    assert_eq!(row.backend, Backend::Clarabel);
    assert_eq!(row.kind, CertificateKind::PrimalInfeasible);
    assert_eq!(row.accuracy, CertificateAccuracy::Full);
    let verification = row.verification.as_ref().unwrap();
    assert!(
        verification.verified && verification.margin > 0.0,
        "{row:?}"
    );
    // The covering row's lower side, then both variables' bounds.
    assert_eq!(
        row.ray.iter().map(|e| e.coordinate).collect::<Vec<_>>(),
        [
            RayCoordinate::RowLower,
            RayCoordinate::VariableLower,
            RayCoordinate::VariableUpper,
            RayCoordinate::VariableLower,
            RayCoordinate::VariableUpper,
        ]
    );
    // Farkas: the covering row pushes x + y up, the upper bounds hold them at one.
    let value = |coordinate| {
        row.ray
            .iter()
            .filter(|e| e.coordinate == coordinate)
            .map(|e| e.value)
            .sum::<f64>()
    };
    assert!(value(RayCoordinate::RowLower) > 0.0);
    assert!(value(RayCoordinate::VariableUpper) > 0.0);
    let metrics =
        solve_metrics::Row::rows(&result.table("runtime.solve_metrics").unwrap()).unwrap();
    assert!(metrics.iter().all(|m| m.namespace != "certificate"));
}
