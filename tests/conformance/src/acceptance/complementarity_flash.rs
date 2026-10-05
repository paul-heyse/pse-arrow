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
    math::solves::SolverProfile,
    workflow::{ModelingPackage, RunReport},
};
use std::collections::BTreeMap;
use std::time::Duration;

/// Select real complementarity fixture identities from the intact loaded package.
fn complementarity_fixtures(package: &ModelingPackage) -> BTreeMap<String, SemanticId> {
    let tests = package
        .declarations()
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
async fn solve(package: &ModelingPackage, case: SemanticId, solver: SolverProfile) -> (f64, f64) {
    let result = seed_prepare(package, case, solver, &CancelSource::new())
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap();
    let report = authored_success(&result);
    let value = |label: &str| {
        report
            .reports
            .iter()
            .find(|r| r.label == label)
            .unwrap()
            .value
    };
    assert!(matches!(result.report(), Ok(RunReport::Modeling(_))));
    (value("vapor fraction"), value("equilibrium temperature"))
}

#[tokio::test]
async fn flash_phase_disappearance_agrees_across_realizations() {
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    let cases = complementarity_fixtures(&package);
    for (feed, beta) in [("liquid", 0.0), ("two_phase", 0.3961), ("vapor", 1.0)] {
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
        for (name, (vapor, temperature)) in [("smooth", smooth), ("disjunctive", disjunctive)] {
            assert!(
                (vapor - beta).abs() < 1e-4,
                "{feed} {name}: vapor fraction {vapor}"
            );
            assert!(temperature.is_finite(), "{feed} {name}");
        }
        // The smoothed pair leaves the equilibrium temperature within O(eps²) of the
        // exact disjunctive one.
        assert!(
            (smooth.1 - disjunctive.1).abs() < 1e-3,
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
        let penalty = solve(&package, penalty_case, feasible(Backend::Pounce)).await;
        assert!(
            (penalty.0 - beta).abs() < 1e-4,
            "{feed} penalty: vapor fraction {}",
            penalty.0
        );
        // The exact penalty's solution is the complementarity point itself.
        assert!(
            (penalty.1 - disjunctive.1).abs() < 1e-4,
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
