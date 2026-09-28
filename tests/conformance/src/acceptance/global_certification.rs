// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Global phase-stability certification of the authored tangent-plane model through the
//! real pipeline to SCIP (Plan 22 G6). Scientific equations stay in the authored packages.
use super::fixtures::*;
use pse_backend_native::solve::{Assurance, Backend, Qualification, SolveIntent, SolverSelection};
use pse_ids::SemanticId;
use pse_runtime::{
    CancelSource,
    math::solves::{Outcome, SolverProfile},
    workflow::{ModelingPackage, RunReport},
};
use std::time::Duration;

/// The explicit certify intent on the certifying route.
fn certify(minutes: u64) -> SolverProfile {
    let mut solver = profile(Backend::Scip, true);
    solver.intent = SolveIntent::Certify;
    solver.selection = SolverSelection::Auto;
    solver.controls.time_limit = Duration::from_secs(60 * minutes);
    solver
}

/// A tangent-plane stability case of the authored `phase_stability` model, added to the
/// homogeneous fixtures, with `dof` degrees of freedom after the `header` specifications.
fn stability(
    package: &ModelingPackage,
    name: &str,
    law: &str,
    dof: usize,
    header: &str,
) -> (ModelingPackage, SemanticId) {
    let source = format!(
        "@id(\"b70ab2554b57594e8d2b75288e80da8e\") package homogeneous_fixtures {{ use phase_stability @\"1.0.0\"; use helmholtz @\"1.0.0\"; test {name} fixture {{dof {dof}; run steady; {header}}} {{ child root:phase_stability.TangentPlaneStability=phase_stability.TangentPlaneStability(selected=chem.alkanes,law={law},feed=vessel_fixtures.fraction); }} }}"
    );
    let extra = pse_authoring::language::parse(
        &source,
        pse_ids::named_id(SemanticId::NIL, &format!("tpd-{name}")),
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let case = extra
        .iter()
        .find(|r| r.name == name)
        .unwrap()
        .declaration_id;
    let mut rows = package.declarations().to_vec();
    rows.extend(
        extra
            .into_iter()
            .filter(|r| r.value.kind.as_str() != "package"),
    );
    (package.with_declarations(rows).unwrap(), case)
}
/// The solved tangent-plane distance and whether the authored stability check holds.
fn tpd(result: &pse_runtime::workflow::RunResult) -> (f64, bool) {
    let RunReport::Modeling(reports) = result.report().unwrap() else {
        panic!("expected a modeling report");
    };
    let report = &reports[0];
    let value = report
        .reports
        .iter()
        .find(|r| r.label == "tangent plane distance")
        .unwrap()
        .value;
    let check = report
        .checks
        .iter()
        .filter(|c| c.kind == pse_relations::generated::enums::ModelingCheckKind::Check)
        .all(|c| c.satisfied);
    (value, check)
}

#[tokio::test]
async fn tpd_certifies_stable_feed() {
    // An ideal mixture is stable everywhere: tpd is the Kullback-Leibler divergence of the
    // trial from the feed, minimized at the feed with value zero.
    let owner = WorkflowRuntime::new().unwrap();
    let source = seed_package(&owner).await;
    let (package, case) = stability(&source, "tpd_ideal", "helmholtz.ideal", 2, "");
    let result = seed_prepare(&package, case, certify(5), &CancelSource::new())
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap();
    let RunReport::Modeling(reports) = result.report().unwrap() else {
        panic!("expected a modeling report");
    };
    let Outcome::Native(native) = &reports[0].outcome else {
        panic!("expected a native certification");
    };
    assert_eq!(native.backend, Backend::Scip);
    assert_eq!(
        native.qualification,
        Qualification::GapQualified,
        "{native:?}"
    );
    assert_eq!(native.termination.assurance, Assurance::GlobalBound);
    // The global lower bound certifies stability within the recorded tolerances.
    let g = native.evidence.global.unwrap();
    assert!(g.dual_bound.unwrap() > -1e-6, "{g:?}");
    let (value, check) = tpd(&result);
    assert!(value.abs() < 1e-6 && check, "{value}");
    authored_success(&result);
}
