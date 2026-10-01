// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Global phase-stability certification of the authored tangent-plane model through the
//! real pipeline to SCIP (Plan 22 G6). Scientific equations stay in the authored packages.
use super::fixtures::*;
use pse_backend_native::solve::{Assurance, Backend, Qualification, SolveIntent, SolverSelection};
use pse_ids::SemanticId;
use pse_relations::generated::enums::ModelingCheckBasis;
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

/// The authored `intent certify;` fixture of an ideal mixture
/// (`phase_stability_fixtures.tpd_ideal` in the seed data).
const TPD_IDEAL: &str = "f622163224f64cecaa98b10d0952aa16";
/// The authored phase-stability fixtures package, whose equimolar benzene/toluene feed
/// lies inside the Peng-Robinson two-phase region.
const PHASE_STABILITY_FIXTURES: &str = "881a0e9fb03144108d64f181a79d1e9f";

/// The Peng-Robinson tangent-plane case at the two-phase feed, added to the authored
/// phase-stability fixtures at run time: an unstable feed fails the model's own stability
/// check, so it is no conformance fixture. The reference phase is the feed's vapor root; the
/// trial density stays below the smallest covolume limit 1/b of either component.
fn unstable_case(package: &ModelingPackage, header: &str) -> (ModelingPackage, SemanticId) {
    let name = "tpd_pr_two_phase";
    let source = format!(
        "@id(\"{PHASE_STABILITY_FIXTURES}\") package phase_stability_fixtures {{ use eos_data @\"1.0.0\"; test {name} fixture {{dof 1; route steady; procedure solve; value root.temperature=368{{K}}; value root.pressure=101325{{Pa}}; value root.reference_upper=200{{mol/m^3}}; value root.trial_upper=10500{{mol/m^3}}; value root.reference.rho=34{{mol/m^3}}; value root.trial.rho=9505.77{{mol/m^3}}; {header}}} {{ child root:phase_stability.TangentPlaneStability=phase_stability.TangentPlaneStability(selected=bt_ideal.aromatics,law=eos_data.potential,feed=bt_feed); }} }}"
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
    (package.with_declarations(rows).unwrap(), case.as_id())
}
/// The authored stability check `tpd > -tolerance` of the phase-stability model.
const TPD_CHECK: &str = "cf7deebc57f244f8a52ec8e924140225";
/// The solved tangent-plane distance and the authored stability check.
fn tpd(result: &pse_runtime::workflow::RunResult) -> (f64, &pse_runtime::workflow::ModelingCheck) {
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
        .find(|c| c.source_id.as_id() == SemanticId::parse_hex(TPD_CHECK).unwrap())
        .unwrap();
    (value, check)
}

#[tokio::test]
async fn tpd_certifies_stable_feed() {
    // An ideal mixture is stable everywhere: tpd is the Kullback-Leibler divergence of the
    // trial from the feed, minimized at the feed with value zero.
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    let case = SemanticId::parse_hex(TPD_IDEAL).unwrap();
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
    // The global lower bound certifies stability within the recorded tolerances, and the
    // authored stability check reads it rather than the point.
    let g = native.evidence.global.unwrap();
    assert!(g.dual_bound.unwrap() > -1e-6, "{g:?}");
    let (value, check) = tpd(&result);
    assert!(value.abs() < 1e-6, "{value}");
    assert!(check.satisfied, "{check:?}");
    assert_eq!(check.basis, ModelingCheckBasis::GlobalBound, "{check:?}");
    authored_success(&result);
}

#[tokio::test]
async fn tpd_detects_known_instability() {
    // Peng-Robinson benzene/toluene at 368 K and 101325 Pa, equimolar: against the feed's
    // vapor root, a stationary liquid-root trial phase has a tangent-plane distance of about
    // −0.110. A qualified trial point with a negative distance is conclusive evidence of
    // instability, whether or not a global bound exists, and the local route reaches the
    // reference's stationary point. (SCIP does not close this gap in bounded time: after
    // 10 minutes and 42 413 nodes its dual bound was −2.05 beside an incumbent at −0.1104.)
    let reference: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fixture("../pr-stability-reference.json")).unwrap(),
    )
    .unwrap();
    // teqp's canonical Peng-Robinson uses the exact constants; the reference's closed form,
    // checked against it, evaluates the authored rounded ones (0.45724, 0.07780).
    let teqp = reference["teqp"]["tpd"].as_f64().unwrap();
    let authored = &reference["authored"];
    let expected = authored["tpd"].as_f64().unwrap();
    assert!(
        expected < -0.1 && (expected - teqp).abs() < 1e-3,
        "{expected} {teqp}"
    );
    let owner = WorkflowRuntime::new().unwrap();
    let source = seed_package(&owner).await;
    let (package, case) = unstable_case(&source, "");
    let result = seed_prepare(
        &package,
        case,
        profile(Backend::Ipopt, true),
        &CancelSource::new(),
    )
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
        panic!("expected a native solve");
    };
    // The trial point is qualified in original coordinates ...
    assert!(native.quality.as_ref().unwrap().feasible(), "{native:?}");
    assert!(
        matches!(
            native.qualification,
            Qualification::Feasible | Qualification::Stationary
        ),
        "{native:?}"
    );
    // ... it is the reference's stationary liquid trial phase ...
    let (value, check) = tpd(&result);
    assert!((value - expected).abs() < 1e-8, "{value} vs {expected}");
    let benzene = authored["trial"][0].as_f64().unwrap();
    let primal = &native.candidate.as_ref().unwrap().primal;
    assert!(
        primal.iter().any(|x| (x - benzene).abs() < 1e-7),
        "{primal:?} lacks {benzene}"
    );
    // ... and its distance is negative, so the authored stability check fails there.
    assert!(!check.satisfied, "{check:?}");
    assert_eq!(check.basis, ModelingCheckBasis::Point, "{check:?}");
}

#[tokio::test]
async fn fixture_intent_selects_certify() {
    // The run's own policy is an automatic local optimization; the authored
    // `intent certify;` of the stability fixture selects the certifying route for it.
    use pse_model::generated::enums::ModelingConformanceStatus;
    use pse_model::generated::identities::DeclarationId;
    let owner = WorkflowRuntime::new().unwrap();
    let source = seed_package(&owner).await;
    let fixture = DeclarationId::from(SemanticId::parse_hex(TPD_IDEAL).unwrap());
    // Only this fixture runs.
    let rows = source.declarations().to_vec();
    let mut solver = profile(Backend::Ipopt, true);
    solver.selection = SolverSelection::Auto;
    solver.controls.time_limit = Duration::from_secs(300);
    assert_eq!(solver.intent, SolveIntent::Optimize);
    let conform = |rows: Vec<_>| {
        let package = source.with_declarations(rows).unwrap();
        let solver = solver.clone();
        async move {
            package
                .conform(
                    pse_runtime::workflow::ModelingConformancePolicy {
                        compiler: compiler(),
                        solver,
                        numerical: pse_runtime::math::solves::NumericalInputs::default(),
                        limits: seed_limits(),
                        derivatives: pse_backend_native::derivative_diagnostics::Policy {
                            perturbation: 1e-6,
                            relative_tolerance: 1e-4,
                            maximum_cells: 100,
                        },
                        maximum_fixtures: 16,
                        maximum_checks: 10_000,
                        diagnostics: None,
                        fixtures: pse_runtime::workflow::ModelingFixtureSelection::Selected(
                            [fixture].into(),
                        ),
                    },
                    &CancelSource::new(),
                )
                .await
                .unwrap()
        }
    };
    let report = conform(rows.clone()).await;
    assert_eq!(report.fixtures(), [fixture].into());
    // The fixture's solve, its expectation and its checks pass (derivative sampling is
    // reported separately).
    use pse_model::generated::enums::ModelingConformanceKind as Kind;
    for kind in [Kind::StartToSolve, Kind::Expectation, Kind::Check] {
        let rows: Vec<_> = report
            .checks
            .iter()
            .filter(|c| c.fixture_id == fixture && c.kind == kind)
            .collect();
        assert!(!rows.is_empty(), "{kind:?}");
        assert!(
            rows.iter()
                .all(|c| c.status == ModelingConformanceStatus::Passed),
            "{rows:?}"
        );
    }
    let result = &report.results[&fixture];
    let Outcome::Native(native) = &result.outcome else {
        panic!("expected a native certification");
    };
    assert_eq!(native.backend, Backend::Scip);
    assert_eq!(native.qualification, Qualification::GapQualified);
    // The authored stability check reads the certified bound.
    let check = result
        .checks
        .iter()
        .find(|c| c.source_id.as_id() == SemanticId::parse_hex(TPD_CHECK).unwrap())
        .unwrap();
    assert!(check.satisfied, "{check:?}");
    assert_eq!(check.basis, ModelingCheckBasis::GlobalBound);
    // Control: without the declared intent the run's automatic optimization takes a local
    // route, and the check is evaluated at the point.
    let mut open = rows;
    for row in &mut open {
        if row.declaration_id == fixture
            && let Some(scope) = row.value.scope.as_mut()
            && let Some(authored) = scope.fixture.as_mut()
        {
            authored.intent = None;
        }
    }
    let report = conform(open).await;
    let result = &report.results[&fixture];
    let Outcome::Native(native) = &result.outcome else {
        panic!("expected a native solve");
    };
    assert_ne!(native.backend, Backend::Scip);
    let check = result
        .checks
        .iter()
        .find(|c| c.source_id.as_id() == SemanticId::parse_hex(TPD_CHECK).unwrap())
        .unwrap();
    assert_eq!(check.basis, ModelingCheckBasis::Point);
}

/// The foreign-library allowance the heater's certification declares for SCIP's search over
/// the PC-SAFT heater, which becomes SCIP's `limits/memory`. Measured on 2026-09-28: at
/// 512 MiB and 1 GiB SCIP stops with `SCIP_STATUS_MEMLIMIT` before its first node; at 2 GiB
/// it proves optimality in 91 nodes (the whole run 18 s, process peak 3.3 GB). The
/// deployment's allowance is 64 MiB; a declared one is the solve's own, so no other job or
/// program is charged it.
const SCIP_FOREIGN_BYTES: usize = 2 << 30;

#[tokio::test]
async fn heater_optimization_certified() {
    // The authored `intent certify;` fixture: heater_optimization over a declared
    // vapor-branch box, so the certified global minimum is the selected local one.
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    let case = SemanticId::parse_hex("e0d4fbe894134e60bb4e364dddae9c5f").unwrap();
    let mut solver = certify(10);
    solver.controls.foreign_bytes = Some(SCIP_FOREIGN_BYTES);
    let result = seed_prepare(&package, case, solver, &CancelSource::new())
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap();
    let report = authored_success(&result);
    let Outcome::Native(native) = &report.outcome else {
        panic!("expected a native certification");
    };
    assert_eq!(native.backend, Backend::Scip);
    // Both terminals carry the gap-qualified bound checked below. The authored
    // policy may close the configured gap before SCIP's exact optimality terminal.
    assert!(
        matches!(
            native.termination.name.as_str(),
            "SCIP_STATUS_OPTIMAL" | "SCIP_STATUS_GAPLIMIT"
        ),
        "{native:?}"
    );
    assert_eq!(
        native.qualification,
        Qualification::GapQualified,
        "{native:?}"
    );
    assert_eq!(native.termination.assurance, Assurance::GlobalBound);
    // The certified bound and the qualified candidate's fresh objective agree within the
    // recorded gap; the authored expectations (T = 350 K on the vapor root) hold.
    let g = native.evidence.global.unwrap();
    assert!(g.readback && g.nodes > 0, "{g:?}");
    let bound = g.dual_bound.unwrap();
    let objective = native.observation.as_ref().unwrap().objective.unwrap();
    assert!(
        (objective - bound).abs() <= g.gap_absolute + g.gap_relative * bound.abs(),
        "{objective} {bound}"
    );
}

/// A small nonconvex regression certified globally (ADR-0102's named certify fixtures):
/// the authored least-squares first-order decay `y = a·exp(−k·t)` through five
/// observations, whose global optimum the fixture's expectations state.
#[tokio::test]
async fn small_regression_certified() {
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    let case = SemanticId::parse_hex("f767847e54e547d396cf0190032aa9d4").unwrap();
    let result = seed_prepare(&package, case, certify(5), &CancelSource::new())
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap();
    let report = authored_success(&result);
    let Outcome::Native(native) = &report.outcome else {
        panic!("expected a native certification");
    };
    assert_eq!(native.backend, Backend::Scip);
    assert_eq!(
        native.qualification,
        Qualification::GapQualified,
        "{native:?}"
    );
    assert_eq!(native.termination.assurance, Assurance::GlobalBound);
    // The certified lower bound meets the reference minimum within the recorded gap.
    let g = native.evidence.global.unwrap();
    let bound = g.dual_bound.unwrap();
    let objective = native.observation.as_ref().unwrap().objective.unwrap();
    assert!((objective - 0.001_262_75).abs() < 1e-7, "{objective}");
    assert!(
        (objective - bound).abs() <= g.gap_absolute + g.gap_relative * bound.abs(),
        "{objective} {bound}"
    );
}

/// The PC-SAFT tangent-plane case of the vessel mixture, added to the authored phase-stability
/// fixtures at run time, prepared under `limits`.
async fn pcsaft_tpd(
    package: &ModelingPackage,
    limits: pse_modeling::Limits,
) -> Result<pse_runtime::workflow::ModelingSolvePreparation, pse_runtime::workflow::WorkflowError> {
    let name = "tpd_pcsaft";
    let source = format!(
        "@id(\"{PHASE_STABILITY_FIXTURES}\") package phase_stability_fixtures {{ use pcsaft_data @\"1.0.0\"; test {name} fixture {{dof 2; route steady; procedure solve;}} {{ child root:phase_stability.TangentPlaneStability=phase_stability.TangentPlaneStability(selected=vessel_fixtures.alkanes,law=pcsaft_data.potential,feed=vessel_fixtures.fraction); }} }}"
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
    let package = package.with_declarations(rows).unwrap();
    let cancel = CancelSource::new();
    let analysis = package
        .declared_analysis(
            case,
            pse_relations::generated::enums::ModelingAnalysisRoute::Steady,
            compiler(),
            profile(Backend::Ipopt, true),
            Default::default(),
            limits,
            &cancel,
        )
        .await?;
    package.prepare_analysis(&analysis, &cancel).await
}

#[tokio::test]
async fn pcsaft_tpd_fits_the_formal_pool() {
    // The PC-SAFT tangent-plane body of three components needs 5510 formal slots (measured
    // 2026-09-29), above the former 4096 pool: it is refused within that allowance and
    // prepares and solves within the process-global pool.
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    let former = pse_modeling::Limits {
        body_slots: Some(4096),
        ..seed_limits()
    };
    let refused = pcsaft_tpd(&package, former).await.unwrap_err();
    assert!(refused.to_string().contains("body slots"), "{refused}");
    let prepared = pcsaft_tpd(&package, seed_limits()).await.unwrap();
    let result = prepared.start().unwrap().wait().await.unwrap();
    // The feed is its own stationary trial phase: the distance is zero there.
    let (value, check) = tpd(&result);
    assert!(value.abs() < 1e-6, "{value}");
    assert!(check.satisfied, "{check:?}");
}
