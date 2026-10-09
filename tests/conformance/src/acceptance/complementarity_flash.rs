// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Phase disappearance authored as complementarity through the real pipeline (Plan 22 M5a,
//! ADR-0104 §5). The authored `ComplementarityVLE` of the reference `equilibrium` package
//! and its `bt_ideal` flash fixtures state every equation; the realizations only lower the
//! complementarity pairs.
use super::fixtures::*;
use pse_backend_native::solve::Backend;
use pse_ids::SemanticId;
use pse_model::generated::enums::ModelingStructuralRequirement as Requirement;
use pse_runtime::{
    CancelSource,
    math::solves::{Outcome, SolverProfile},
    workflow::{ModelingPackage, ModelingSolvePreparation, RunReport},
};
use std::collections::BTreeMap;
use std::time::Duration;

/// Select real complementarity fixture identities from the intact loaded package.
async fn complementarity_fixtures(package: &ModelingPackage) -> BTreeMap<String, SemanticId> {
    let tests = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.value.kind.as_str() == "test" && r.name.starts_with("complementarity_"))
        .map(|r| (r.name.clone(), r.declaration_id.as_id()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(tests.len(), 9, "3 feeds × 3 realizations");
    tests
}
/// An explicit route to a feasible point of the fixture's system.
fn feasible(backend: Backend) -> SolverProfile {
    let mut solver = profile(backend, false);
    solver.intent = pse_backend_native::solve::SolveIntent::FeasiblePoint;
    solver.controls.time_limit = Duration::from_secs(300);
    solver
}
/// Solve an authored fixture and return its vapor fraction and equilibrium temperature.
async fn solve(
    package: &ModelingPackage,
    case: SemanticId,
    solver: SolverProfile,
) -> (f64, f64, f64) {
    let prepared = seed_prepare(package, case, solver, &CancelSource::new())
        .await
        .unwrap();
    solve_prepared(&prepared).await
}
/// Execute the same admitted preparation whose route and closure were inspected.
async fn solve_prepared(prepared: &ModelingSolvePreparation) -> (f64, f64, f64) {
    let result = prepared.start().unwrap().wait().await.unwrap();
    if let Ok(RunReport::Modeling(reports)) = result.report() {
        for report in reports
            .iter()
            .filter(|report| !report.accepted || !result.usable())
        {
            for value in report.reports.iter().filter(|value| {
                matches!(
                    value.label.as_str(),
                    "vapor fraction" | "equilibrium temperature"
                )
            }) {
                eprintln!(
                    "flash rejected output: label={} value={:.17e}",
                    value.label, value.value
                );
            }
            for check in report
                .checks
                .iter()
                .filter(|check| !check.satisfied)
                .take(8)
            {
                eprintln!(
                    "flash rejected check: source={} target={} kind={:?} error={:.17e} tolerance={:?} satisfied={}",
                    check.source_id,
                    check.target_id,
                    check.kind,
                    check.value,
                    check.tolerance,
                    check.satisfied
                );
            }
            if let Outcome::Native(native) = &report.outcome {
                eprintln!(
                    "flash rejected native: backend={:?} code={} name={} termination={:?} qualification={:?} kkt={:?} work={:?}",
                    native.backend,
                    native.termination.code,
                    native.termination.name,
                    native.termination.category,
                    native.qualification,
                    native.evidence.kkt,
                    native.evidence.work
                );
                if let Some(quality) = &native.quality {
                    eprintln!(
                        "flash rejected native quality: normalized_max={:.17e}",
                        quality.normalized_max
                    );
                    for (kind, violation) in quality
                        .rows
                        .iter()
                        .map(|violation| ("row", violation))
                        .chain(quality.bounds.iter().map(|violation| ("bound", violation)))
                        .filter(|(_, violation)| violation.physical > violation.tolerance)
                        .take(8)
                    {
                        eprintln!(
                            "flash rejected native violation: kind={kind} source={} physical={:.17e} tolerance={:.17e}",
                            violation.id, violation.physical, violation.tolerance
                        );
                    }
                } else {
                    eprintln!("flash rejected native quality: unavailable");
                }
                for name in [
                    "tol",
                    "constr_viol_tol",
                    "dual_inf_tol",
                    "compl_inf_tol",
                    "l1_exact_penalty_barrier",
                    "hessian_approximation",
                    "max_wall_time",
                ] {
                    if let Some(value) = native.options.get(name) {
                        eprintln!("flash rejected native option: name={name} value={value:?}");
                    }
                }
            }
        }
    }
    let report = authored_success(&result);
    assert!(matches!(result.report(), Ok(RunReport::Modeling(_))));
    let Outcome::Native(native) = &report.outcome else {
        panic!("expected native complementarity realization");
    };
    assert!(
        native
            .quality
            .as_ref()
            .is_some_and(|quality| quality.feasible())
    );
    let vapor = report
        .reports
        .iter()
        .find(|r| r.label == "vapor fraction")
        .unwrap();
    let registry = &report.prepared.model.case.compiled().quantities;
    let numerics = report.prepared.solve.numerics();
    let vapor = vapor.value;
    let temperature = report
        .reports
        .iter()
        .find(|r| r.label == "equilibrium temperature")
        .unwrap();
    let temperature_allowance = resolved_physical_allowance(
        numerics,
        registry,
        temperature,
        SemanticId::parse_hex("13874d4b57684720bb42a48164325812").unwrap(),
    );
    let temperature = temperature.value;
    assert!(vapor.is_finite());
    assert!(temperature.is_finite());
    (vapor, temperature, temperature_allowance)
}

#[tokio::test]
async fn flash_phase_disappearance_agrees_across_realizations() {
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    let cases = complementarity_fixtures(&package).await;
    // Historical beta comparisons belong to the source-authored expectations.
    for feed in ["liquid", "two_phase", "vapor"] {
        // smooth(math.smooth_min, 1e-4): a square system on the NLP route.
        let smooth = solve(
            &package,
            cases[&format!("complementarity_{feed}_smooth")],
            feasible(Backend::Ipopt),
        )
        .await;
        // disjunctive: slack columns and a native SOS1, on SCIP.
        let disjunctive = solve(
            &package,
            cases[&format!("complementarity_{feed}_disjunctive")],
            feasible(Backend::Scip),
        )
        .await;
        // Empirical agreement uses the resolved shared temperature floor.
        // Bound feasibility budgets do not guarantee forward output accuracy.
        assert!(
            (smooth.1 - disjunctive.1).abs() <= smooth.2.min(disjunctive.2),
            "{feed}: smooth {} K, disjunctive {} K",
            smooth.1,
            disjunctive.1
        );
        // penalty(l1): nonnegative members, a product row and the l1 exact-penalty
        // requirement of its case structure, which selects POUNCE's l1 method: the
        // author's selection (ADR-0104 §5, Plan 22 M5b).
        let penalty_case = cases[&format!("complementarity_{feed}_penalty")];
        let prepared = seed_prepare(
            &package,
            penalty_case,
            feasible(Backend::Pounce),
            &CancelSource::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            prepared
                .model
                .case
                .compiled()
                .plan
                .structure()
                .requirements(),
            [Requirement::L1ExactPenalty]
        );
        let penalty = solve_prepared(&prepared).await;
        // The realizations agree within the shared physical output resolution.
        assert!(
            (penalty.1 - disjunctive.1).abs() <= penalty.2.min(disjunctive.2),
            "{feed}: penalty {} K, disjunctive {} K",
            penalty.1,
            disjunctive.1
        );
        // Another backend cannot honour the requirement.
        assert!(
            seed_prepare(
                &package,
                penalty_case,
                feasible(Backend::Ipopt),
                &CancelSource::new()
            )
            .await
            .is_err()
        );
    }
}
